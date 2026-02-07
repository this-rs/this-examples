# Architecture

The examples are built around a clear separation of concerns enforced by this-rs:

- Module (domain): entities, stores, handlers, descriptors
- Host: transport-agnostic core assembled from modules
- Exposures: transport adapters (REST, GraphQL) mounted on the host

## Core components

- `BillingModule` / `BillingStores`: registers orders, invoices, and payments.
- `CatalogModule` / `CatalogStores`: registers products, categories, and tags.
- `InventoryModule` / `InventoryStores`: registers stores, activities, warehouses, stock items, stock movements, and usages.
- `ServerBuilder`: composes the host from one or more modules and attaches cross-cutting services like the link service.
- `InMemoryLinkService`: manages generic and typed links between entities (including cross-module links).

## Data flow

1. A request enters via an exposure (REST or GraphQL).
2. The exposure delegates to the host, which routes to the appropriate entity handler.
3. Handlers use the registered stores to read/write models.
4. The response flows back through the exposure to the client.

## Host vs. exposure

- The host is transport-agnostic: it contains the domain wiring (modules, stores, link service).
- Exposures are thin adapters that translate transport-specific requests to host calls.
- Multiple exposures can be merged on the same Axum router. In the GraphQL example, REST and GraphQL share the same host.

## Multi-module composition

The `multi-module` example demonstrates how multiple domain modules are composed into a single host:

```rust
ServerBuilder::new()
    .with_link_service(link_service)
    .register_module(billing_module)?
    .register_module(catalog_module)?
    .register_module(inventory_module)?
    .build_host()?;
```

Each module brings its own entities, link configuration, and store factories. The host merges them and the link service handles relationships both within and across modules (e.g., inventory's `stock_item` referencing catalog's `product`).

## Ports and listeners

- GraphQL example: binds to `127.0.0.1:4242` (local-only) and exposes both REST and GraphQL routes.
- REST example: binds to `0.0.0.0:4242` (all interfaces) and exposes REST routes.
- Multi-module example: binds to `127.0.0.1:4242` and exposes REST + GraphQL for all three modules.

## Seeding and observability

- `test-data` seeds in-memory stores so the server starts with realistic data.
- `tracing_subscriber` initializes structured logs for debugging and demos.
