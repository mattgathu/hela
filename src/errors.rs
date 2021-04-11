use thiserror::Error;

use crate::{Client, TransactionId};

/// A result wrapper around HelaError
pub type Fallible<T> = Result<T, HelaError>;

#[derive(Debug, Error)]
/// Enumeration of all possible Hela errors
pub enum HelaError {
    /// DataStore Locking
    #[error("DataStore locking error")]
    DataStoreLockError,

    /// Missing Transaction
    #[error("Transaction Not Found for ID: {0}")]
    TransactionNotFound(TransactionId),

    /// Missing Account
    #[error("Acount Not Found for Client: {0}")]
    AccountNotFound(Client),

    /// Insufficient Account Funds
    #[error("Insufficient Funds in Acount for Client: {0}")]
    InsufficientAccountFunds(Client),

    /// CSV Data Error
    #[error("Error when processing CSV data: {0}")]
    CsvError(csv::Error),

    /// IO Errors
    #[error(transparent)]
    IoError(#[from] std::io::Error),

    /// String Parsing error
    #[error(transparent)]
    Utf8Error(#[from] std::string::FromUtf8Error),
}
