# Testing

The examples are designed to be test-friendly by default. Stores are in-memory and the `test-data` crate provides helpers to seed realistic data.

## Seeding data

Use the `test-data` crate to populate stores **before** building the host (the builder consumes the link service):

```rust
let link_service = Arc::new(InMemoryLinkService::new());
let stores = BillingStores::new_in_memory();
populate_test_data(&stores, link_service.clone()).await?;
```

This pattern is used in all examples to ensure the server starts with non-empty data and pre-linked entities.

## Unit tests

- Test handlers in isolation by constructing in-memory stores and a minimal module/host as needed.
- Validate business rules without standing up HTTP servers.

## Integration tests

- Spin up the Axum app using the same `ServerBuilder` and issue requests against it.
- Seed with `test-data` to make responses stable and meaningful.

## Tips

- Prefer deterministic IDs in tests when possible, or assert on semantic fields instead of raw IDs.
- Keep tests close to the domain crate (`crates/billing`) to avoid leaking transport details into domain tests.
