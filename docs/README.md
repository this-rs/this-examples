# Documentation

Complete documentation for the this-examples project.

## Table of Contents

### Getting Started

- **[Overview](overview.md)** - High-level introduction to the project goals, layout, and how components fit together
- **[Quick Start](../README.md#quick-start)** - Running the examples (REST and GraphQL)

### Architecture & Design

- **[Architecture](architecture.md)** - Core components, data flow, host vs. exposure pattern
- **[Entities and Macros](entities-and-macros.md)** - Entity structure, file nomenclature, adding new entities
- **[Best Practices](best-practices.md)** - Project conventions, entity patterns, error handling, testing

### Protocols & APIs

- **[Protocols: REST and GraphQL](protocols.md)** - Dual protocol exposure, router composition
- **[REST API Guide](rest-api.md)** - REST endpoints, usage examples, testing
- **[Nested Routes](nested-routes.md)** - Entity relationships via auto-generated nested REST routes
- **[GraphQL Playground](graphql-playground.md)** - Using the GraphQL interface, queries, mutations

### Advanced Topics

- **[Links and Relations](links-and-relations.md)** - Managing relationships between entities
- **[Development Workflow](development.md)** - Adding features, iterating, debugging
- **[Testing and Seeded Data](testing.md)** - In-memory stores, test data provisioning

## Crate Documentation

- **[test-data](../crates/test-data/README.md)** - Data provisioning for demos and tests
- **[billing](../crates/billing/)** - Domain module with orders, invoices, and payments
- **[catalog](../crates/catalog/)** - Domain module with products, categories, and tags
- **[inventory](../crates/inventory/)** - Domain module with stores, activities, warehouses, stock, and usage tracking

## Examples

Every directory under `examples/` is a runnable crate with its own tests. Grouped by what each one shows:

**Protocols**
- **[rest](../examples/rest/)** - REST only, in-memory storage: the starting point
- **[graphql](../examples/graphql/)** - REST + GraphQL (schema derived from the modules, playground)
- **[grpc](../examples/grpc/)** - REST + gRPC on the same port, server reflection
- **[websocket](../examples/websocket/)** - REST + real-time entity events on `/ws`
- **[multi-module](../examples/multi-module/)** - billing, catalog and inventory in one host, all four transports, cross-module links

**Storage backends** (same billing domain, store swapped)
- **[dynamodb](../examples/dynamodb/)**, **[postgres](../examples/postgres/)**, **[mysql](../examples/mysql/)**, **[mongodb](../examples/mongodb/)**, **[neo4j](../examples/neo4j/)**, **[scylladb](../examples/scylladb/)**, **[lmdb](../examples/lmdb/)**

**Events**
- **[event-flows](../examples/event-flows/)** - YAML sinks and flows turning entity/link events into notifications, no Rust per flow
- **[cognitive-signals](../examples/cognitive-signals/)** - CognitiveNotificationBridge: anomalies, co-change, stigmergy, scars, episodes

**Authentication**
- **[auth-sts](../examples/auth-sts/)** - embedded WAMI STS: Ed25519 bootstrap, RBAC policies, custom resolvers, multi-tenant, GDPR erasure
- **[oidc](../examples/oidc/)** - resource server verifying tokens from an external IdP, with a mock IdP (discovery + JWKS) so it runs offline

**Contracts and data**
- **[descriptor](../examples/descriptor/)** - one `ModuleDescriptor`, the same validation, filters, policies and messages on REST, GraphQL and gRPC
- **[soft-delete](../examples/soft-delete/)** - test crate measuring `with_soft_delete()` on the guarded path, and pinning that the application's REST handlers sit outside it

**Modules**
- **[module-signing](../examples/module-signing/)** - a stored module loads only if its manifest is signed by a key of the trust anchor; the four outcomes measured, no server
- **[vfs-module](../examples/vfs-module/)** - load, replace and unload a declarative module in a `ModuleStore`, same cycle on an in-memory and a local `Vfs`, no server

## Quick Links

- [Main README](../README.md)
- [Repository Structure](overview.md#repository-layout)
- [GraphQL Schema](../README.md#run-the-graphql--rest-example)

---

Each guide is self-contained and can be read independently, but following the order above provides a natural learning path.
- [Relief and exposure](relief-and-exposure.md) — what the database contains, and who may reach it
