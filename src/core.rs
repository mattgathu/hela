//! A module providing types for common data and traits for components.

use serde::{Deserialize, Serialize, Serializer};

use crate::errors::Fallible;

/// Client Identifier
pub type Client = u16;
/// Monetary Amount
pub type Amount = f64;
/// Transaction Identifier
pub type TransactionId = u32;

/// Transaction Type
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TransactionType {
    /// Withdrawal
    Withdrawal,
    /// Deposit
    Deposit,
    /// Dispute
    Dispute,
    /// Dispute Resolution
    Resolve,
    /// Chargeback
    Chargeback,
}

/// Transaction
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Transaction {
    /// Transaction Type
    pub r#type: TransactionType,
    /// Client
    pub client: Client,
    /// Transaction Identifier
    pub id: TransactionId,
    /// Optional Amount
    #[serde(default)]
    pub amount: Option<Amount>,
    /// Transaction is disputed
    #[serde(default)]
    #[serde(skip)]
    pub disputed: bool,
}

/// Account
#[derive(Copy, Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Account {
    /// Client
    pub client: Client,
    /// Available amount
    #[serde(serialize_with = "ser_float")]
    pub available: Amount,
    /// Held amount
    #[serde(serialize_with = "ser_float")]
    pub held: Amount,
    /// Total Amount
    #[serde(serialize_with = "ser_float")]
    pub total: Amount,
    /// Locked status
    pub locked: bool,
}

impl Account {
    #[cfg(debug_assertions)]
    /// check that account invariants are not violated
    pub fn check_invariants(&self) {
        assert!(self.total >= self.available);
        assert!((self.total - (self.available + self.held)).abs() < f64::EPSILON);
    }
}

/// A combined storage interface for Accounts and Transactions
pub trait Storage: AccountStore + TransactionStore {}

/// Trait providing interface to be implemented by storage backend
pub trait AccountStore {
    /// Check if an account has been locked.
    fn is_locked(&self, client_id: Client) -> Fallible<bool>;

    /// A chargeback is the final state of a dispute and represents the client reversing a transaction.
    /// Funds that were held have now been withdrawn.
    ///
    /// This means that the clients held funds and total funds should decrease by the amount previously disputed.
    /// If a chargeback occurs the client's account should be immediately frozen.
    fn chargeback(&mut self, client_id: Client, amount: Amount) -> Fallible<()>;

    /// A deposit is a credit to the client's asset account, meaning it should increase
    /// the available and total funds of the client account
    fn deposit(&mut self, client_id: Client, amount: Amount) -> Fallible<()>;

    /// A dispute represents a client's claim that a transaction was erroneous and should be reversed.
    ///
    /// This means that the clients available funds should decrease by the amount
    /// disputed, their held funds should increase by the amount disputed,
    /// while their total funds should remain the same.
    fn dispute(&mut self, client_id: Client, amount: Amount) -> Fallible<()>;

    /// Get a client's Account
    fn get_account(&self, client_id: Client) -> Fallible<Account>;

    /// Locks an Account
    fn lock_account(&mut self, client_id: Client) -> Fallible<()>;

    /// A resolve represents a resolution to a dispute, releasing the associated held funds.
    /// Funds that were previously disputed are no longer disputed.
    ///
    /// This means that the clients held funds should decrease by the amount no longer disputed,
    /// their available funds should increase by the amount no longer disputed,
    /// and their total funds should remain the same.
    fn resolve(&mut self, client_id: Client, amount: Amount) -> Fallible<()>;

    /// Persist account in the storage backend
    fn store_account(&mut self, acc: Account) -> Fallible<()>;

    /// A withdraw is a debit to the client's asset account, meaning it
    /// should decrease the available and total funds of the client account
    fn withdraw(&mut self, client_id: Client, amount: Amount) -> Fallible<()>;

    /// All accounts stored by the storage backend.
    fn get_accounts(&self) -> Fallible<Box<dyn Iterator<Item = Account> + '_>>;
}

/// An interface implemented by Transactions storage backends
pub trait TransactionStore {
    /// Fetch a Transaction
    fn get_transaction(&self, id: TransactionId) -> Fallible<Transaction>;

    /// Fetch a Transaction Amount
    fn get_transaction_amount(&self, id: TransactionId) -> Fallible<Option<Amount>>;

    /// Persist a transaction in the storage backend
    fn store_transaction(&mut self, txn: Transaction) -> Fallible<()>;

    /// Mark a transaction as disputed
    fn mark_transaction_as_disputed(&mut self, id: TransactionId) -> Fallible<()>;

    /// Mark a transaction as undisputed
    fn mark_transaction_as_undisputed(&mut self, id: TransactionId) -> Fallible<()>;
}

/// Serialize floats
pub fn ser_float<S: Serializer>(float: &f64, serializer: S) -> Result<S::Ok, S::Error> {
    let float_as_str = format!("{:.2}", float);
    serializer.serialize_str(&&float_as_str)
}
