# this-examples

A collection of runnable examples demonstrating how to build modular, protocol-agnostic backends with the this-rs framework.

- Domain modules are isolated as Rust crates under `crates/` for clean boundaries and data isolation.
- Each entity follows the same file nomenclature to keep development consistent and predictable (`model.rs`, `store.rs`, `handlers.rs`, `descriptor.rs`).
- Dual protocol exposure: the exact same domain is exposed via REST and GraphQL.
- GraphQL schema is auto-generated from the registered entities.

## Repository layout

```
crates/
  billing/           # Domain module (orders, invoices, payments)
  test-data/         # Utilities to seed in-memory stores for demos/tests
examples/
  graphql/           # GraphQL + REST on the same server (playground included)
  rest/              # REST-only server
```

## Quick start

Prerequisites: Rust toolchain (stable), internet access for crates, and a free port 4242.

### Run the GraphQL + REST example

```
cargo run -p graphql_example --features graphql
```

- Server: `http://127.0.0.1:4242`
- GraphQL Playground: `http://127.0.0.1:4242/graphql/playground`
- GraphQL endpoint: `POST http://127.0.0.1:4242/graphql`
- GraphQL schema: `GET http://127.0.0.1:4242/graphql/schema`
- REST endpoints: `GET /health`, `GET /order`, `GET /invoice`, `GET /payment`

### Run the REST-only example

```
cargo run -p rest_example
```

- Server: `http://0.0.0.0:4242`
- REST endpoints: `GET /health`, `GET /order`, `GET /invoice`, `GET /payment`

Both examples use in-memory stores and are seeded with sample data from the `test-data` crate at startup.

## Key concepts

- Module isolation: keep your business logic in `crates/<module>` and expose it via one or more protocols.
- Uniform entity structure: each entity directory contains `model`, `store`, `handlers`, and a `descriptor` describing the entity to the framework.
- Transport-agnostic host: build a host once, then compose one or many exposures (REST/GraphQL) over it.
- Links/relations: a link service manages generic and typed relationships across entities.

## Documentation

- Overview: `docs/overview.md`
- Architecture: `docs/architecture.md`
- Entities and macros: `docs/entities-and-macros.md`
- Protocols (REST/GraphQL): `docs/protocols.md`
- GraphQL Playground guide: `docs/graphql-playground.md`
- REST API guide: `docs/rest-api.md`
- Best practices: `docs/best-practices.md`
- Development workflow: `docs/development.md`
- Testing and seeded data: `docs/testing.md`
- Links and relations: `docs/links-and-relations.md`

## License

MIT or the license of the upstream project. See the root project for details.
