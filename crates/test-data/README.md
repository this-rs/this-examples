# Test Data

This crate provides test data provisioning for the billing examples.

## Purpose

Isolates the code responsible for creating in-memory test entities (Orders, Invoices, Payments) so it can be shared across multiple example applications (REST and GraphQL).

## Usage

```rust
use billing::BillingStores;
use test_data::populate_test_data;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let stores = BillingStores::new_in_memory();
    
    // Populate with test data
    populate_test_data(stores.clone()).await?;
    
    // ... use stores in your application
    Ok(())
}
```

## Test Data

The `populate_test_data` function creates:

- **2 Orders**: 
  - ORD-001 (pending, $999.99)
  - ORD-002 (paid, $4999.99)

- **3 Invoices**:
  - INV-001 (draft, $999.99)
  - INV-002 (paid, $999.99)
  - INV-003 (sent, $4999.99)

- **3 Payments**:
  - PAY-001 (completed, $999.99, credit_card)
  - PAY-002 (completed, $999.99, bank_transfer)
  - PAY-003 (pending, $4999.99, credit_card)
