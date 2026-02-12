# gRPC Example

REST + gRPC coexisting on the same server using `this-rs` with the billing module.

## What this demonstrates

- **REST + gRPC** on the same port using `RestExposure` + `GrpcExposure`
- **Generic CRUD via gRPC** — `EntityService` handles any registered entity type dynamically
- **Link management via gRPC** — `LinkService` for entity relationships
- **Dynamic `.proto` export** — typed proto definitions generated from registered entity types
- **Schemaless data** — entity payloads use `google.protobuf.Struct` (JSON-like)

## Prerequisites

- **protoc** (Protocol Buffers compiler) — required for compilation
  ```bash
  # macOS
  brew install protobuf

  # Linux (Debian/Ubuntu)
  apt install protobuf-compiler
  ```

- **grpcurl** (optional, for testing gRPC endpoints)
  ```bash
  # macOS
  brew install grpcurl

  # Linux — see https://github.com/fullstorydev/grpcurl
  ```

## Architecture

```
                         ┌─────────────────────────────────┐
                         │         127.0.0.1:4244          │
                         │                                 │
  HTTP/1.1 requests ────▶│  RestExposure (health, CRUD)    │
                         │         │                       │
                         │         │ fallback_service       │
                         │         ▼                       │
  HTTP/2 gRPC calls ────▶│  GrpcExposure (tonic)           │
                         │    ├─ EntityService (CRUD)      │
                         │    ├─ LinkService (relations)   │
                         │    └─ GET /grpc/proto (export)  │
                         └─────────────────────────────────┘
                                       │
                                  ServerHost
                            (transport-agnostic core)
```

REST routes are matched first. Unmatched requests fall through to the gRPC router via `fallback_service` — this is necessary because both REST (link routes) and tonic install a fallback handler, and axum forbids merging two routers with fallback.

## Quick start

```bash
# Start the server (from this-examples root)
cargo run -p grpc_example

# Test REST
curl http://127.0.0.1:4244/health
curl http://127.0.0.1:4244/orders

# Test gRPC (see "Testing with grpcurl" section below)
```

## Endpoints

### REST API

| Method | Path | Description |
|--------|------|-------------|
| GET | `/health` | Health check |
| GET | `/orders` | List all orders |
| POST | `/orders` | Create an order |
| GET | `/orders/{id}` | Get an order |
| PUT | `/orders/{id}` | Update an order |
| DELETE | `/orders/{id}` | Delete an order |
| GET | `/invoices` | List all invoices |
| GET | `/payments` | List all payments |

### gRPC Services

The server exposes two services defined in `this_grpc.proto`:

**EntityService** — Generic CRUD for any registered entity type

| RPC | Request | Response | Description |
|-----|---------|----------|-------------|
| `GetEntity` | `GetEntityRequest` | `EntityResponse` | Get entity by type + ID |
| `ListEntities` | `ListEntitiesRequest` | `ListEntitiesResponse` | List with pagination |
| `CreateEntity` | `CreateEntityRequest` | `EntityResponse` | Create new entity |
| `UpdateEntity` | `UpdateEntityRequest` | `EntityResponse` | Update existing entity |
| `DeleteEntity` | `DeleteEntityRequest` | `DeleteEntityResponse` | Delete entity |

**LinkService** — Relationship management

| RPC | Request | Response | Description |
|-----|---------|----------|-------------|
| `CreateLink` | `CreateLinkRequest` | `LinkResponse` | Create a link between entities |
| `GetLink` | `GetLinkRequest` | `LinkResponse` | Get link by ID |
| `FindLinksBySource` | `FindLinksRequest` | `LinkListResponse` | Find outgoing links |
| `FindLinksByTarget` | `FindLinksRequest` | `LinkListResponse` | Find incoming links |
| `DeleteLink` | `DeleteLinkRequest` | `DeleteLinkResponse` | Delete a link |

### Proto export

| Method | Path | Description |
|--------|------|-------------|
| GET | `/grpc/proto` | Dynamic typed `.proto` with per-entity services |

## Testing with grpcurl

The server does **not** expose gRPC reflection. You need the `.proto` file to use grpcurl. The base proto is at `this_grpc.proto` in the framework.

```bash
# Set the proto import path (adjust to your this-rs checkout)
PROTO_PATH="/path/to/this-rs/this/proto"

# List available services
grpcurl -plaintext -import-path $PROTO_PATH -proto this_grpc.proto \
  127.0.0.1:4244 list

# List RPCs for EntityService
grpcurl -plaintext -import-path $PROTO_PATH -proto this_grpc.proto \
  127.0.0.1:4244 describe this_grpc.EntityService

# List all orders (seeded test data)
grpcurl -plaintext -import-path $PROTO_PATH -proto this_grpc.proto \
  -d '{"entity_type": "order", "limit": 10}' \
  127.0.0.1:4244 this_grpc.EntityService/ListEntities

# Get a specific order by ID
grpcurl -plaintext -import-path $PROTO_PATH -proto this_grpc.proto \
  -d '{"entity_type": "order", "entity_id": "<UUID>"}' \
  127.0.0.1:4244 this_grpc.EntityService/GetEntity

# Create a new order via gRPC
grpcurl -plaintext -import-path $PROTO_PATH -proto this_grpc.proto \
  -d '{
    "entity_type": "order",
    "data": {
      "name": "gRPC Order",
      "number": "ORD-GRPC",
      "status": "pending",
      "amount": 99.99,
      "customer_name": "gRPC Client",
      "notes": "Created via gRPC"
    }
  }' \
  127.0.0.1:4244 this_grpc.EntityService/CreateEntity

# Create a link between two entities
grpcurl -plaintext -import-path $PROTO_PATH -proto this_grpc.proto \
  -d '{
    "link_type": "order_invoice",
    "source_id": "<order-UUID>",
    "target_id": "<invoice-UUID>"
  }' \
  127.0.0.1:4244 this_grpc.LinkService/CreateLink

# Find links from a source entity
grpcurl -plaintext -import-path $PROTO_PATH -proto this_grpc.proto \
  -d '{"entity_id": "<order-UUID>"}' \
  127.0.0.1:4244 this_grpc.LinkService/FindLinksBySource
```

## Dynamic proto export

The server generates a **typed** `.proto` file based on registered entity types, available at `GET /grpc/proto`. This is different from the base `this_grpc.proto` (which uses generic `google.protobuf.Struct`).

```bash
# Export the typed proto
curl -s http://127.0.0.1:4244/grpc/proto > billing.proto
```

The exported proto contains per-entity services (`OrderService`, `InvoiceService`, `PaymentService`) with typed messages. A snapshot is saved in [`proto/this_billing.proto`](proto/this_billing.proto).

### Generating clients in other languages

Use the exported `.proto` to generate typed clients:

```bash
# Go
protoc --go_out=. --go-grpc_out=. billing.proto

# Python
python -m grpc_tools.protoc -I. --python_out=. --grpc_python_out=. billing.proto

# TypeScript (via ts-proto)
protoc --plugin=protoc-gen-ts_proto --ts_proto_out=. billing.proto

# Java
protoc --java_out=. --grpc-java_out=. billing.proto
```

## Key implementation details

- `GrpcExposure::build_router(host)` takes the same `Arc<ServerHost>` as all other exposures
- `rest_router.fallback_service(grpc_router)` is required to combine routers (both install fallback handlers)
- `with_event_bus()` is **not required** for gRPC (unlike WebSocket) — gRPC works without it
- `populate_test_data()` must be called **before** `build_host()` (the builder consumes the link service)
- Entity data is schemaless — gRPC uses `google.protobuf.Struct` for dynamic JSON-like payloads
- The base `this_grpc.proto` uses a generic `EntityService` for all types; the dynamic proto at `/grpc/proto` generates typed per-entity services
