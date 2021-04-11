use std::collections::HashMap;

use crate::{
    Account, AccountStore, Amount, Client, Fallible, HelaError, Storage, Transaction,
    TransactionId, TransactionStore,
};

/// In Memory data store for Accounts and Transaction
#[derive(Debug, Clone)]
pub struct InMemoryStore {
    accounts: HashMap<Client, Account>,
    transactions: HashMap<TransactionId, Transaction>,
}

impl InMemoryStore {
    /// create an empty store
    pub fn new() -> InMemoryStore {
        InMemoryStore {
            accounts: HashMap::new(),
            transactions: HashMap::new(),
        }
    }

    fn get_account_mut(&mut self, client_id: Client) -> &mut Account {
        self.accounts.entry(client_id).or_insert(Account {
            client: client_id,
            ..Default::default()
        })
    }
}

impl AccountStore for InMemoryStore {
    fn is_locked(&self, client_id: Client) -> Fallible<bool> {
        self.get_account(client_id).map(|acc| acc.locked)
    }

    fn chargeback(&mut self, client_id: Client, amount: Amount) -> Fallible<()> {
        let acc = self.get_account_mut(client_id);
        acc.held -= amount;
        acc.total -= amount;
        acc.locked = true;
        if cfg!(debug_assertions) {
            acc.check_invariants();
        }
        Ok(())
    }

    fn deposit(&mut self, client_id: Client, amount: Amount) -> Fallible<()> {
        let acc = self.get_account_mut(client_id);
        acc.available += amount;
        acc.total += amount;
        Ok(())
    }

    fn dispute(&mut self, client_id: Client, amount: Amount) -> Fallible<()> {
        let acc = self.get_account_mut(client_id);
        acc.available -= amount;
        acc.held += amount;
        if cfg!(debug_assertions) {
            acc.check_invariants();
        }
        Ok(())
    }

    fn get_account(&self, client_id: Client) -> Fallible<Account> {
        let acc = *self
            .accounts
            .get(&client_id)
            .ok_or(HelaError::AccountNotFound(client_id))?;
        Ok(acc)
    }

    fn lock_account(&mut self, client_id: Client) -> Fallible<()> {
        let acc = self.get_account_mut(client_id);
        acc.locked = true;
        Ok(())
    }

    fn resolve(&mut self, client_id: Client, amount: Amount) -> Fallible<()> {
        let acc = self.get_account_mut(client_id);
        acc.held -= amount;
        acc.available += amount;
        if cfg!(debug_assertions) {
            acc.check_invariants();
        }
        Ok(())
    }

    fn store_account(&mut self, acc: Account) -> Fallible<()> {
        self.accounts.insert(acc.client, acc);
        Ok(())
    }

    fn withdraw(&mut self, client_id: Client, amount: Amount) -> Fallible<()> {
        let acc = self.get_account_mut(client_id);
        if acc.available < amount {
            return Err(HelaError::InsufficientAccountFunds(client_id));
        }
        acc.available -= amount;
        acc.total -= amount;

        if cfg!(debug_assertions) {
            acc.check_invariants();
        }

        Ok(())
    }

    fn get_accounts(&self) -> Fallible<Box<dyn Iterator<Item = Account> + '_>> {
        let iter = self.accounts.values().copied();
        Ok(Box::new(iter))
    }
}

impl TransactionStore for InMemoryStore {
    fn get_transaction(&self, id: TransactionId) -> Fallible<Transaction> {
        let txn = *self
            .transactions
            .get(&id)
            .ok_or(HelaError::TransactionNotFound(id))?;
        Ok(txn)
    }

    fn get_transaction_amount(&self, id: TransactionId) -> Fallible<Option<Amount>> {
        Ok(self.get_transaction(id)?.amount)
    }

    fn store_transaction(&mut self, txn: Transaction) -> Fallible<()> {
        self.transactions.insert(txn.id, txn);
        Ok(())
    }

    fn mark_transaction_as_disputed(&mut self, id: TransactionId) -> Fallible<()> {
        let txn = self
            .transactions
            .get_mut(&id)
            .ok_or(HelaError::TransactionNotFound(id))?;
        txn.disputed = true;
        Ok(())
    }

    fn mark_transaction_as_undisputed(&mut self, id: TransactionId) -> Fallible<()> {
        let txn = self
            .transactions
            .get_mut(&id)
            .ok_or(HelaError::TransactionNotFound(id))?;
        txn.disputed = false;
        Ok(())
    }
}

impl Storage for InMemoryStore {}

impl Default for InMemoryStore {
    fn default() -> Self {
        Self::new()
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn store_and_fetch() -> Fallible<()> {
        let mut store = InMemoryStore::new();
        let acc = Account {
            client: 0u16,
            available: 100f64,
            held: 0f64,
            total: 100f64,
            locked: false,
        };
        acc.check_invariants();
        store.store_account(acc.clone())?;
        let racc = store.get_account(acc.client)?;
        assert_eq!(acc, racc);
        Ok(())
    }
}
