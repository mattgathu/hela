use crate::{Account, Fallible, HelaError, Storage, Transaction, TransactionType};

/// Payments transcations processor
pub struct PaymentEngine {
    store: Box<dyn Storage>,
}

impl PaymentEngine {
    /// create a new engine
    pub fn new(store: Box<dyn Storage>) -> PaymentEngine {
        PaymentEngine { store }
    }

    /// Execute a single transcation
    pub fn execute_transaction(&mut self, txn: Transaction) -> Fallible<()> {
        match txn.r#type {
            TransactionType::Deposit => {
                debug_assert!(txn.amount.is_some());
                self.store.deposit(txn.client, txn.amount.unwrap())?;
                self.store.store_transaction(txn)?;
            }
            // Spec: If a client does not have sufficient available funds the withdrawal
            // should fail and the total amount of funds should not change.
            //
            // What does fail mean here?
            //
            // Assumption is the account state doesn't change.
            // The engine suppresses the InsufficientAccountFunds error.
            TransactionType::Withdrawal => {
                debug_assert!(txn.amount.is_some());
                match self.store.withdraw(txn.client, txn.amount.unwrap()) {
                    Err(HelaError::InsufficientAccountFunds(_)) => {}
                    Err(e) => return Err(e),
                    Ok(_) => {
                        self.store.store_transaction(txn)?;
                    }
                }
            }
            TransactionType::Chargeback => {
                //  Spec: if the tx specified doesn't exist, or the tx isn't under dispute,
                //  you can ignore chargeback and assume this is an error on our partner's side.
                //
                //  How do you know tx is under dispute?
                //
                debug_assert!(txn.amount.is_none());
                if let Ok(prev_txn) = self.store.get_transaction(txn.id) {
                    debug_assert!(prev_txn.client == txn.client);
                    debug_assert!(prev_txn.amount.is_some());
                    if prev_txn.disputed {
                        self.store
                            .chargeback(txn.client, prev_txn.amount.unwrap())?;
                    }
                }
            }
            TransactionType::Dispute => {
                debug_assert!(txn.amount.is_none());
                if let Ok(prev_txn) = self.store.get_transaction(txn.id) {
                    debug_assert!(prev_txn.client == txn.client);
                    debug_assert!(prev_txn.amount.is_some());
                    self.store.dispute(txn.client, prev_txn.amount.unwrap())?;
                    self.store.mark_transaction_as_disputed(prev_txn.id)?;
                }
            }
            TransactionType::Resolve => {
                debug_assert!(txn.amount.is_none());
                if let Ok(prev_txn) = self.store.get_transaction(txn.id) {
                    debug_assert!(prev_txn.client == txn.client);
                    debug_assert!(prev_txn.amount.is_some());
                    if prev_txn.disputed {
                        self.store.resolve(txn.client, prev_txn.amount.unwrap())?;
                        self.store.mark_transaction_as_undisputed(prev_txn.id)?;
                    }
                }
            }
        }

        Ok(())
    }

    /// Execute a stream of transcations
    pub fn execute_transactions<I>(&mut self, txns: I) -> Fallible<()>
    where
        I: Iterator<Item = Fallible<Transaction>>,
    {
        for txn in txns {
            self.execute_transaction(txn?)?;
        }
        Ok(())
    }

    /// Get a stream if accounts from the storage backend
    pub fn accounts(&self) -> Fallible<Box<dyn Iterator<Item = Account> + '_>> {
        self.store.get_accounts()
    }
}
