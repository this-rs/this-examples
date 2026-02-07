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
  catalog/           # Domain module (products, categories, tags)
  inventory/         # Domain module (stores, activities, warehouses, stock, usage tracking)
  test-data/         # Utilities to seed in-memory stores for demos/tests
examples/
  graphql/           # GraphQL + REST on the same server (playground included)
  rest/              # REST-only server
  dynamodb/          # DynamoDB backend example with local setup
  multi-module/      # Multi-module example combining billing, catalog & inventory
```

## Quick start

Prerequisites: Rust toolchain (stable), internet access for crates, and a free port 4242.

### Using the Makefile (Recommended)

```bash
# See all available commands
make help

# Run different examples
make rest      # REST-only server
make graphql   # GraphQL + REST server  
make dynamodb  # DynamoDB example with local setup
```

### Manual commands

### Run the GraphQL + REST example

```
cargo run -p graphql_example --features graphql
```

- Server: `http://127.0.0.1:4242`
- GraphQL Playground: `http://127.0.0.1:4242/graphql/playground`
- GraphQL endpoint: `POST http://127.0.0.1:4242/graphql`
- GraphQL schema: `GET http://127.0.0.1:4242/graphql/schema`
- REST endpoints:
  - Entity routes: `GET /order`, `GET /invoice`, `GET /payment`
  - Nested routes: `GET /order/{id}/invoices`, `GET /invoice/{id}/payments`
  - Multi-level chaining: `GET /order/{id}/invoices/{id}/payments`
  - Health: `GET /health`

### Run the REST-only example

```
cargo run -p rest_example
```

- Server: `http://0.0.0.0:4242`
- REST endpoints:
  - Entity routes: `GET /order`, `GET /invoice`, `GET /payment`
  - Nested routes: `GET /order/{id}/invoices`, `GET /invoice/{id}/payments`
  - Multi-level chaining: `GET /order/{id}/invoices/{id}/payments`
  - Health: `GET /health`

### Run the DynamoDB example

```bash
# Setup local DynamoDB with Docker
cd examples/dynamodb
./setup.sh

# Run the application
cargo run --features dynamodb
```

- Server: `http://0.0.0.0:4242`
- DynamoDB Admin UI: `http://localhost:8001`
- Same REST and GraphQL endpoints as other examples
- Uses local DynamoDB for persistent storage

### Run the multi-module example

```
cargo run -p multi_module_example --features graphql
```

- Server: `http://127.0.0.1:4242`
- Combines **billing**, **catalog**, and **inventory** modules in a single server
- REST endpoints for all three domains + GraphQL endpoint
- Demonstrates cross-module links (e.g., `stock_item → product` from inventory to catalog)

The REST and GraphQL examples use in-memory stores, while the DynamoDB example provides persistent storage. All are seeded with sample data and entity links from the `test-data` crate at startup.

## Key concepts

- **Module isolation**: keep your business logic in `crates/<module>` and expose it via one or more protocols.
- **Uniform entity structure**: each entity directory contains `model`, `store`, `handlers`, and a `descriptor` describing the entity to the framework.
- **Transport-agnostic host**: build a host once, then compose one or many exposures (REST/GraphQL) over it.
- **Links/relations**: a link service manages relationships across entities, automatically generating nested routes.
- **Link chaining**: define individual links in YAML; the framework automatically chains them to create multi-level routes (e.g., `/order/{id}/invoices/{id}/payments`).
- **Cross-module links**: entities from different modules can reference each other (e.g., inventory's `stock_item` links to catalog's `product`).
- **Multi-activity stores**: a single store can host multiple activities, each with independent stock and usage tracking.

## Documentation

📚 **[Complete Documentation](docs/README.md)** - Full documentation with organized table of contents

### Quick Links

- **[Overview](docs/overview.md)** - Project goals and structure
- **[Architecture](docs/architecture.md)** - Core components and data flow
- **[Entities and Macros](docs/entities-and-macros.md)** - Entity structure and conventions
- **[Protocols](docs/protocols.md)** - REST and GraphQL exposure
- **[Best Practices](docs/best-practices.md)** - Development conventions

See the [docs directory](docs/) for the complete list of guides covering GraphQL playground, REST API, testing, links/relations, and development workflow.

## License

MIT or the license of the upstream project. See the root project for details.
