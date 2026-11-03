# Overview

This repository showcases how to structure modular backends with the this-rs framework using real-world inspired domains (billing, catalog, inventory). It focuses on:

- Isolating domain logic in crates under `crates/`.
- Enforcing a uniform entity structure across the domain for predictability.
- Building a protocol-agnostic host and exposing it via REST, GraphQL, gRPC, and WebSocket.
- Demonstrating links/relations between entities using a link service.
- Showing multiple storage backends (in-memory, LMDB, PostgreSQL, MongoDB, Neo4j, ScyllaDB, MySQL, DynamoDB).

## Goals

- Provide copy-paste runnable examples to kickstart new services.
- Encourage clean boundaries between domain, storage, and transport layers.
- Showcase multi-protocol exposure (REST, GraphQL, gRPC, WebSocket) for the same domain model.
- Demonstrate pluggable storage backends from development to production.
- Promote testable design through in-memory stores and seeded data.

## Repository layout

```
crates/
  billing/           # Domain module (orders, invoices, payments)
  catalog/           # Domain module (products, categories, tags)
  inventory/         # Domain module (stores, activities, warehouses, stock, usage)
  test-data/         # Data seeding helpers for demos & tests
examples/
  rest/              # REST-only exposure (in-memory storage)
  graphql/           # REST + GraphQL exposure, playground included
  grpc/              # REST + gRPC exposure with reflection
  websocket/         # REST + WebSocket real-time events
  multi-module/      # Combines billing, catalog & inventory modules (all transports)
  dynamodb/          # DynamoDB backend example with local setup
  postgres/          # PostgreSQL backend example
  mongodb/           # MongoDB backend example
  neo4j/             # Neo4j backend example
  scylladb/          # ScyllaDB backend example
  mysql/             # MySQL backend example
  lmdb/              # LMDB embedded backend example
  event-flows/       # Declarative YAML event pipelines → in-app notifications (REST + WebSocket)
  cognitive-signals/ # CognitiveNotificationBridge: anomalies, co-change, stigmergy, scars, episodes
  auth-sts/          # WAMI STS: Ed25519 bootstrap, RBAC policies, resolvers, multi-tenant, GDPR
  oidc/              # OIDC resource server verifying an external IdP (mock IdP included)
  soft-delete/       # with_soft_delete() measured on the guarded path; REST boundary pinned
  descriptor/        # ModuleDescriptor declared once, served on REST, GraphQL and gRPC
  module-signing/    # A stored module loads only if signed by a key of the trust anchor (test crate)
  vfs-module/        # Load, replace, unload a declarative module in a ModuleStore (test crate)
benches/
  endpoint-benchmarks/
```

## Domain modules

### Billing
Orders, invoices, and payments with a linear link chain (`order → invoice → payment`).

### Catalog
Products, categories, and tags. Features many-to-many relationships (`product ↔ category`, `product ↔ tag`) and reflexive hierarchical links (`category → category` for parent/child).

### Inventory
Multi-activity store management. A store can host multiple activities (e.g., bar + co-working), each with independent warehouses, stock items, stock movements, and usage tracking for refacturation. Cross-module links connect `stock_item` to catalog's `product`.

## How things fit together

- `crates/billing` defines the billing domain: orders, invoices, and payments.
- `crates/catalog` defines the catalog domain: products, categories (with hierarchy), and tags.
- `crates/inventory` defines the inventory domain: stores, activities, warehouses, stock items, movements, and usage tracking.
- `examples/*` assemble a server by registering modules into a host and attaching protocol exposures.
- `test-data` seeds in-memory stores with sample data to make the examples meaningful.
- The **rest** example shows the simplest setup: REST-only with in-memory storage.
- The **graphql** example merges REST and GraphQL routers while keeping domain concerns isolated inside the module crate.
- The **grpc** example adds gRPC exposure with server reflection alongside REST.
- The **websocket** example adds real-time WebSocket events (create/update/delete broadcasts).
- The **multi-module** example demonstrates registering all three domain modules into a single host with all four transports, including cross-module links (e.g., `stock_item → product`).
- The **storage backend** examples (dynamodb, postgres, mongodb, neo4j, scylladb, mysql, lmdb) show the same billing domain with different databases, demonstrating the pluggable storage architecture.
- The **event-flows** example declares event pipelines in YAML (sinks + flows) and turns entity and link events into notifications without Rust code per flow; **cognitive-signals** plugs the CognitiveNotificationBridge on the same EventBus.
- The **auth-sts** example runs the embedded WAMI STS (the host issues and verifies its own tokens); the **oidc** example is the other posture — a resource server that only verifies tokens from an external IdP, with a mock IdP so it runs offline.
- The **soft-delete** example is a test crate: it measures what `with_soft_delete()` guarantees on the guarded path, and pins the boundary that the application's own REST handlers sit outside of it.
- The **descriptor** example declares a module once (`ModuleDescriptor`) and gets the same validation, filters, policies and messages on REST, GraphQL and gRPC.
- The **module-signing** example measures the four outcomes of module signing (accepted, bad signature, untrusted key, unsigned once an anchor exists); the **vfs-module** example walks the load / replace / unload cycle of a declarative module on an in-memory and a local `Vfs`. Both are test crates: no server, the store's contract is what they assert.
