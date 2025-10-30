# Test Data

This crate provides test data provisioning for the billing examples.

## Purpose

Isolates the code responsible for creating in-memory test entities (Orders, Invoices, Payments) and establishing links between them, so it can be shared across multiple example applications (REST and GraphQL).

## Usage

```rust
use std::sync::Arc;
use billing::BillingStores;
use test_data::populate_test_data;
use this::storage::InMemoryLinkService;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let stores = BillingStores::new_in_memory();
    let link_service = Arc::new(InMemoryLinkService::new());
    
    // Populate with test data and links
    populate_test_data(stores.clone(), link_service.clone()).await?;
    
    // ... use stores and link_service in your application
    Ok(())
}
```

## Test Data

The `populate_test_data` function creates:

### Entities

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

### Links

The function also creates links between entities:

- **Order → Invoice links**:
  - Order 1 → Invoice 1
  - Order 1 → Invoice 2
  - Order 2 → Invoice 3

- **Invoice → Payment links**:
  - Invoice 1 → Payment 1
  - Invoice 2 → Payment 2
  - Invoice 3 → Payment 3

These links enable nested routes like `/order/{id}/invoices` and `/invoice/{id}/payments`.
