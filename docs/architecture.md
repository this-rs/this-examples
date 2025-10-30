# Architecture

The examples are built around a clear separation of concerns enforced by this-rs:

- Module (domain): entities, stores, handlers, descriptors
- Host: transport-agnostic core assembled from modules
- Exposures: transport adapters (REST, GraphQL) mounted on the host

## Core components

- `BillingModule`: registers the billing domain's entities and their handlers.
- `BillingStores`: bundles the in-memory stores for `Order`, `Invoice`, and `Payment`.
- `ServerBuilder`: composes the host and attaches cross-cutting services like the link service.
- `InMemoryLinkService`: manages generic and typed links between entities.

## Data flow

1. A request enters via an exposure (REST or GraphQL).
2. The exposure delegates to the host, which routes to the appropriate entity handler.
3. Handlers use the registered stores to read/write models.
4. The response flows back through the exposure to the client.

## Host vs. exposure

- The host is transport-agnostic: it contains the domain wiring (modules, stores, link service).
- Exposures are thin adapters that translate transport-specific requests to host calls.
- Multiple exposures can be merged on the same Axum router. In the GraphQL example, REST and GraphQL share the same host.

## Ports and listeners

- GraphQL example: binds to `127.0.0.1:4242` (local-only) and exposes both REST and GraphQL routes.
- REST example: binds to `0.0.0.0:4242` (all interfaces) and exposes REST routes.

## Seeding and observability

- `test-data` seeds in-memory stores so the server starts with realistic data.
- `tracing_subscriber` initializes structured logs for debugging and demos.
