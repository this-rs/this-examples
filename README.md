# this-examples

A collection of runnable examples demonstrating how to build modular, protocol-agnostic backends with the this-rs framework.

- Domain modules are isolated as Rust crates under `crates/` for clean boundaries and data isolation.
- Each entity follows the same file nomenclature to keep development consistent and predictable (`model.rs`, `store.rs`, `handlers.rs`, `descriptor.rs`).
- Multi-protocol exposure: the exact same domain is exposed via REST, GraphQL, gRPC, and WebSocket.
- GraphQL schema is auto-generated from the registered entities.

## Repository layout

```
crates/
  billing/           # Domain module (orders, invoices, payments)
  catalog/           # Domain module (products, categories, tags)
  inventory/         # Domain module (stores, activities, warehouses, stock, usage tracking)
  test-data/         # Utilities to seed in-memory stores for demos/tests
examples/
  rest/              # REST-only server
  graphql/           # GraphQL + REST on the same server (playground included)
  grpc/              # gRPC server with reflection
  websocket/         # WebSocket real-time server with static web client
  dynamodb/          # DynamoDB backend example with local setup
  multi-module/      # Multi-module example combining all transports (REST + GraphQL + gRPC + WebSocket)
```

## Quick start

Prerequisites: Rust toolchain (stable), internet access for crates, and a free port (4242–4244 depending on example).

### Using this-cli (Recommended)

Install the CLI toolkit:

```bash
cargo install this-cli
```

Then navigate to any example directory and run:

```bash
cd examples/rest
this dev --api-only
```

The `this dev` command reads the `this.yaml` config in each example directory and starts the server with live-reload (requires `cargo-watch`, `watchexec`, or `bacon` installed).

Other useful commands:

```bash
this info       # Show project info (this-rs version, enabled features, entities)
this doctor     # Check your environment (Rust toolchain, dependencies, ports)
```

### Using the Makefile

```bash
# See all available commands
make help

# Run different examples
make rest      # REST-only server
make graphql   # GraphQL + REST server
make dynamodb  # DynamoDB example with local setup
```

### Manual commands

```bash
# REST-only
cargo run -p rest_example

# GraphQL + REST
cargo run -p graphql_example --features graphql

# gRPC
cargo run -p grpc_example --features grpc

# WebSocket
cargo run -p websocket_example --features websocket

# DynamoDB
cd examples/dynamodb && ./setup.sh
cargo run --features dynamodb

# Multi-module (all transports)
cargo run -p multi_module_example --features "graphql grpc websocket"
```

## Examples

### REST (`examples/rest/`)

A minimal REST server exposing billing entities.

```bash
cargo run -p rest_example
# or: cd examples/rest && this dev --api-only
```

- Server: `http://0.0.0.0:4242`
- Entity routes: `GET /order`, `GET /invoice`, `GET /payment`
- Nested routes: `GET /order/{id}/invoices`, `GET /invoice/{id}/payments`
- Multi-level chaining: `GET /order/{id}/invoices/{id}/payments`
- Health: `GET /health`

### GraphQL (`examples/graphql/`)

GraphQL + REST on the same server with an interactive playground.

```bash
cargo run -p graphql_example --features graphql
# or: cd examples/graphql && this dev --api-only
```

- Server: `http://127.0.0.1:4242`
- GraphQL Playground: `http://127.0.0.1:4242/graphql/playground`
- GraphQL endpoint: `POST http://127.0.0.1:4242/graphql`
- GraphQL schema: `GET http://127.0.0.1:4242/graphql/schema`
- REST endpoints also available at the same address

### gRPC (`examples/grpc/`)

A gRPC server with server reflection, auto-generated from entity descriptors.

```bash
cargo run -p grpc_example --features grpc
# or: cd examples/grpc && this dev --api-only
```

- Server: `http://127.0.0.1:4244`
- Uses gRPC server reflection (works with `grpcurl`, Postman, etc.)
- Proto definitions are auto-generated from the entity model

```bash
# List available services
grpcurl -plaintext 127.0.0.1:4244 list

# Describe a service
grpcurl -plaintext 127.0.0.1:4244 describe billing.OrderService
```

### WebSocket (`examples/websocket/`)

Real-time WebSocket server with a static web client for testing.

```bash
cargo run -p websocket_example --features websocket
# or: cd examples/websocket && this dev --api-only
```

- Server: `http://127.0.0.1:4243`
- WebSocket endpoint: `ws://127.0.0.1:4243/ws`
- Static web client: `http://127.0.0.1:4243/static/index.html`
- Broadcasts entity events (create, update, delete) in real-time

### DynamoDB (`examples/dynamodb/`)

Persistent storage backend using a local DynamoDB instance via Docker.

```bash
cd examples/dynamodb
./setup.sh                    # Start local DynamoDB + admin UI
cargo run --features dynamodb
# or: this dev --api-only
```

- Server: `http://0.0.0.0:4242`
- DynamoDB Admin UI: `http://localhost:8001`
- Same REST endpoints as other examples
- Uses local DynamoDB for persistent storage

### Multi-module (`examples/multi-module/`)

The flagship example: three domain modules combined in a single server, exposed over all four transports simultaneously.

```bash
cargo run -p multi_module_example --features "graphql grpc websocket"
# or: cd examples/multi-module && this dev --api-only
```

- Server: `http://127.0.0.1:4242`
- Combines **billing**, **catalog**, and **inventory** modules
- **REST**: entity routes for all three domains
- **GraphQL**: unified schema across all modules (`/graphql/playground`)
- **gRPC**: reflection-enabled gRPC on the same port (HTTP/2 content-type routing)
- **WebSocket**: real-time entity events (`/ws`)
- Demonstrates cross-module links (e.g., `stock_item → product` from inventory to catalog)

## Key concepts

- **Module isolation**: keep your business logic in `crates/<module>` and expose it via one or more protocols.
- **Uniform entity structure**: each entity directory contains `model`, `store`, `handlers`, and a `descriptor` describing the entity to the framework.
- **Transport-agnostic host**: build a host once, then compose one or many exposures (REST, GraphQL, gRPC, WebSocket) over it.
- **Links/relations**: a link service manages relationships across entities, automatically generating nested routes.
- **Link chaining**: define individual links in YAML; the framework automatically chains them to create multi-level routes (e.g., `/order/{id}/invoices/{id}/payments`).
- **Cross-module links**: entities from different modules can reference each other (e.g., inventory's `stock_item` links to catalog's `product`).
- **Multi-activity stores**: a single store can host multiple activities, each with independent stock and usage tracking.
- **Feature gating**: REST is always available; other transports require Cargo features (`graphql`, `grpc`, `websocket`).

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
