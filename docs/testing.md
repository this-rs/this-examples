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

## Container-backed tests

The storage-backend examples (`postgres`, `mysql`, `mongodb`, `neo4j`, `dynamodb`, `scylladb`)
prove their guarantees against a **real** database, so their tests are `#[ignore]`d: a plain
`cargo test` never starts a container and stays fast. Run them explicitly, one backend at a
time, with a single thread — two containers of the same kind would pick the same resource:

```sh
cargo test -p postgres_example --test smoke          -- --ignored --test-threads=1
cargo test -p postgres_example --test soft_delete    -- --ignored --test-threads=1
```

Docker must be running. `dynamodb` expects the tables of its own `setup.sh`; `scylladb` is the
slowest (a Cassandra container needs minutes before it accepts CQL) and is opt-in in CI.

The **soft-delete contract** is written once, in `soft_delete_example::contract`, and each
backend runs it against its own stores: create, delete through the guarded path, the row must
stay with `deleted_at`, and a guarded read must hide it. Adding a backend means writing the
glue that builds its `BillingStores`, not a second copy of the contract.

## Tips

- Prefer deterministic IDs in tests when possible, or assert on semantic fields instead of raw IDs.
- Keep tests close to the domain crate (`crates/billing`) to avoid leaking transport details into domain tests.
