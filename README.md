# this-examples

[![CI](https://github.com/this-rs/this-examples/actions/workflows/ci.yml/badge.svg)](https://github.com/this-rs/this-examples/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/this-rs/this-examples/graph/badge.svg)](https://codecov.io/gh/this-rs/this-examples)

A collection of runnable examples demonstrating how to build modular, protocol-agnostic backends with the [this-rs](https://crates.io/crates/this-rs) framework (v0.0.9).

- Domain modules are isolated as Rust crates under `crates/` for clean boundaries and data isolation.
- Each entity follows the same file nomenclature to keep development consistent and predictable (`model.rs`, `store.rs`, `handlers.rs`, `descriptor.rs`).
- Multi-protocol exposure: the exact same domain is exposed via REST, GraphQL, gRPC, and WebSocket.
- **9 storage backends**: In-Memory, PostgreSQL, MongoDB, Neo4j, ScyllaDB, MySQL, DynamoDB, LMDB, and Obrain.
- GraphQL schema is auto-generated from the registered entities.
- **EventBus + Event Flows**: declarative YAML pipelines with 8 operators and multi-sink delivery.
- **Cognitive Signals**: anomaly detection, co-change tracking, stigmergy, scars, and episode learning.
- **WAMI Auth STS**: JWT authentication with Ed25519, RBAC policies, custom resolvers, multi-tenant, GDPR erasure.

## Repository layout

```
crates/
  billing/           # Domain module (orders, invoices, payments)
  catalog/           # Domain module (products, categories, tags)
  inventory/         # Domain module (stores, activities, warehouses, stock, usage tracking)
  test-data/         # Utilities to seed in-memory stores for demos/tests
examples/
  rest/              # REST-only server (in-memory)
  graphql/           # GraphQL + REST on the same server (playground included)
  grpc/              # gRPC server with reflection
  websocket/         # WebSocket real-time server with static web client
  dynamodb/          # DynamoDB backend example with local setup
  multi-module/      # Multi-module example combining all transports
  postgres/          # PostgreSQL storage backend
  mongodb/           # MongoDB storage backend
  neo4j/             # Neo4j graph database storage backend
  scylladb/          # ScyllaDB storage backend
  mysql/             # MySQL storage backend
  lmdb/              # LMDB embedded storage backend
  auth-sts/          # WAMI Auth STS — JWT bootstrap, login, tenant isolation, GDPR erasure
  cognitive-signals/ # Cognitive Signals — bridge + thresholds, anomaly detection demo
```

## Quick start

Prerequisites: Rust toolchain (stable), internet access for crates, and a free port (4242-4244 depending on example).

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
# REST-only (in-memory)
cargo run -p rest_example

# GraphQL + REST
cargo run -p graphql_example

# gRPC (requires protoc in PATH)
cargo run -p grpc_example

# WebSocket
cargo run -p websocket_example

# DynamoDB (start local DynamoDB first)
cd examples/dynamodb && ./setup.sh && cd ../..
cargo run -p dynamodb_example

# Multi-module (all transports)
cargo run -p multi_module_example

# Storage backends
cargo run -p postgres_example     # PostgreSQL
cargo run -p mongodb_example      # MongoDB
cargo run -p neo4j_example        # Neo4j
cargo run -p scylladb_example     # ScyllaDB
cargo run -p mysql_example        # MySQL
cargo run -p lmdb_example         # LMDB (embedded, no server needed)

# Auth & Cognitive
cargo run -p auth_sts_example             # WAMI Auth STS (JWT, RBAC, GDPR)
cargo run -p cognitive_signals_example    # Cognitive Signals (anomaly, co-change)
```

## Examples

### REST (`examples/rest/`)

A minimal REST server exposing billing entities.

```bash
cargo run -p rest_example
# or: cd examples/rest && this dev --api-only
```

- Server: `http://0.0.0.0:4242`
- Entity routes: `GET /orders`, `GET /invoices`, `GET /payments`
- Nested routes: `GET /orders/{id}/invoices`, `GET /invoices/{id}/payments`
- Multi-level chaining: `GET /orders/{id}/invoices/{id}/payments`
- Health: `GET /health`

### GraphQL (`examples/graphql/`)

GraphQL + REST on the same server with an interactive playground.

```bash
cargo run -p graphql_example
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
cargo run -p grpc_example
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
cargo run -p websocket_example
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
cd ../..
cargo run -p dynamodb_example
# or: cd examples/dynamodb && this dev --api-only
```

- Server: `http://0.0.0.0:4242`
- DynamoDB Admin UI: `http://localhost:8001`
- Same REST endpoints as other examples
- Uses local DynamoDB for persistent storage

### Multi-module (`examples/multi-module/`)

The flagship example: three domain modules combined in a single server, exposed over all four transports simultaneously.

```bash
cargo run -p multi_module_example
# or: cd examples/multi-module && this dev --api-only
```

- Server: `http://127.0.0.1:4242`
- Combines **billing**, **catalog**, and **inventory** modules
- **REST**: entity routes for all three domains
- **GraphQL**: unified schema across all modules (`/graphql/playground`)
- **gRPC**: reflection-enabled gRPC on the same port (HTTP/2 content-type routing)
- **WebSocket**: real-time entity events (`/ws`)
- Demonstrates cross-module links (e.g., `stock_item -> product` from inventory to catalog)

### Storage backend examples

Each backend example uses the **billing** module with the same REST API but swaps the storage layer. This demonstrates the `DataService` / `LinkService` abstraction: same business logic, different database.

#### PostgreSQL (`examples/postgres/`)

```bash
# Start PostgreSQL
docker run -d --name this-postgres -p 5432:5432 -e POSTGRES_PASSWORD=password postgres:16

# Run the example
cargo run -p postgres_example
```

#### MongoDB (`examples/mongodb/`)

```bash
# Start MongoDB
docker run -d --name this-mongo -p 27017:27017 mongo:7

# Run the example
cargo run -p mongodb_example
```

#### Neo4j (`examples/neo4j/`)

```bash
# Start Neo4j
docker run -d --name this-neo4j -p 7687:7687 -e NEO4J_AUTH=neo4j/password neo4j:5

# Run the example
cargo run -p neo4j_example
```

#### ScyllaDB (`examples/scylladb/`)

```bash
# Start ScyllaDB
docker run -d --name this-scylladb -p 9042:9042 scylladb/scylla:latest

# Run the example
cargo run -p scylladb_example
```

#### MySQL (`examples/mysql/`)

```bash
# Start MySQL
docker run -d --name this-mysql -p 3306:3306 -e MYSQL_ROOT_PASSWORD=password -e MYSQL_DATABASE=this mysql:8

# Run the example
cargo run -p mysql_example
```

#### LMDB (`examples/lmdb/`)

LMDB is an embedded key-value store - no external server required.

```bash
cargo run -p lmdb_example
```

- Data is stored in a local `data/` directory
- Zero-copy reads for high performance
- Great for development and single-node deployments

### Auth STS (`examples/auth-sts/`)

WAMI Auth STS demo with JWT bootstrap, login, tenant isolation, and GDPR erasure.

```bash
cargo run -p auth_sts_example
```

- Server: `http://127.0.0.1:4242`
- Bootstrap mode: auto-generates Ed25519 key pairs at startup
- `POST /auth/token` — JWT token issuance
- `GET /auth/keys` — Public key endpoint
- `POST /auth/refresh` — Token refresh
- `POST /auth/revoke` — Token revocation
- `DELETE /tenants/:tenant_id/data` — GDPR erasure cascade
- Per-entity RBAC policies with custom resolvers (`resolver:is_manager`)
- Multi-tenant isolation via `tenant_id` claim

### Cognitive Signals (`examples/cognitive-signals/`)

Cognitive NotificationBridge demo with anomaly detection and threshold-based signal routing.

```bash
cargo run -p cognitive_signals_example
```

- Server: `http://127.0.0.1:4242`
- CognitiveNotificationBridge subscribes to EventBus
- 5 signal types: AnomalyDetected, CoChangeDetected, StigmergyLockIn, ScarCreated, EpisodeLearned
- Configurable thresholds per signal type
- Routes signals to SinkRegistry (webhook, in-app notification, SSE)
- Create entities and watch cognitive signals fire in real-time

## Key concepts

- **Module isolation**: keep your business logic in `crates/<module>` and expose it via one or more protocols.
- **Uniform entity structure**: each entity directory contains `model`, `store`, `handlers`, and a `descriptor` describing the entity to the framework.
- **Transport-agnostic host**: build a host once, then compose one or many exposures (REST, GraphQL, gRPC, WebSocket) over it.
- **Storage backends**: swap the storage layer without changing business logic. Implement `DataService` and `LinkService` for any database (9 backends included).
- **Links/relations**: a link service manages relationships across entities, automatically generating nested routes.
- **Link chaining**: define individual links in YAML; the framework automatically chains them to create multi-level routes (e.g., `/orders/{id}/invoices/{id}/payments`).
- **Cross-module links**: entities from different modules can reference each other (e.g., inventory's `stock_item` links to catalog's `product`).
- **Multi-activity stores**: a single store can host multiple activities, each with independent stock and usage tracking.
- **Feature gating**: REST is always available; other transports require Cargo features (`graphql`, `grpc`, `websocket`).
- **EventBus**: built-in non-blocking broadcast on all CRUD operations, powers SSE, WebSocket, GraphQL subscriptions, and Event Flows.
- **Event Flows**: declarative YAML pipelines with 8 operators (filter, map, batch, deduplicate, rate_limit, fan_out, resolve, deliver) and multi-sink delivery.
- **Cognitive Signals**: CognitiveNotificationBridge monitors EventBus for anomalies, co-changes, stigmergy lock-ins, scars, and episodes with configurable thresholds.
- **WAMI Auth**: JWT authentication with Ed25519 key pairs, bootstrap mode, per-entity RBAC policies, custom resolvers (`resolver:name`), multi-tenant isolation, GDPR erasure cascade.

## Documentation

See the [docs directory](docs/) for the complete list of guides covering GraphQL playground, REST API, testing, links/relations, and development workflow.

## License

MIT or the license of the upstream project. See the root project for details.
