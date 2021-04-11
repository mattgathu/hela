# Hela Payments Engine

A simple toy payments engine.

Reads a series of transactions from a CSV, updates client accounts, handles disputes and chargebacks, and then outputs the state of clients accounts as a CSV.

## Overview
--
### Common Data Models:

Transaction: represents a payment transaction
```rust
pub struct Transaction {
    pub r#type: TransactionType,
    pub client: Client,
    pub id: TransactionId,
    pub amount: Option<Amount>,
}
```

Account: represents a client's account
```rust
pub struct Account {
    pub client: Client,
    pub available: Amount,
    pub held: Amount,
    pub total: Amount,
    pub locked: bool,
}
```

### Operations:

* Deposits
* Withdrawals
* Disputes
* Resolutions
* Charge backs

## Design and Code Layout
--

The implementation use a component based approach.

The core module defines common types and component interaces that are used in the actual
component implementations.

This approach allows for different implementation of components to be used with the payment engine
without requiring the engine to change. Components can be swapped out.

Project Layout:
- `src/core.rs` : common data types and components traits.
- `src/data.rs` : CSV data ingestion and presentation module.
- `src/engine.rs` : payment transactions processor.
- `src/errors.rs` : errors enumerations.
- `src/main.rs` : Command Line Interace.
- `src/storage.rs` : data storage backend implementation.

## Executing
--
- `cargo run -- transactions.csv > accounts.csv`


## Testing
--

Unit tests:
- `cargo test`

## Further Work
--
* Swap out the In Memory Store if a more robust data storage engine.

