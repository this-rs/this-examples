# Protocols: REST, GraphQL, gRPC, and WebSocket

The same transport-agnostic host is exposed via multiple protocols. You can run REST alone or combine any combination of protocols on the same server.

## REST exposure

- Base routes (examples):
  - `GET /health`
  - `GET /orders`
  - `GET /invoices`
  - `GET /payments`
- Handlers are derived from the entity descriptors and mapped to HTTP routes.
- The REST example binds to `0.0.0.0:4242` for easy testing from other devices.

## GraphQL exposure

- Endpoint: `POST /graphql`
- Playground UI: `GET /graphql/playground`
- Schema: `GET /graphql/schema`
- The GraphQL example binds to `127.0.0.1:4242` and merges REST + GraphQL routers.

## gRPC exposure

- Auto-generated Protocol Buffers with EntityService (5 RPCs) and LinkService (5 RPCs).
- Uses gRPC server reflection (works with `grpcurl`, Postman, etc.).
- Proto definitions are auto-generated from the entity model.

## WebSocket exposure

- WebSocket endpoint: `ws://127.0.0.1:4243/ws` (standalone) or `ws://127.0.0.1:4242/ws` (merged)
- Broadcasts entity events (create, update, delete) in real-time.
- Built-in EventBus with subscribe/unsubscribe filters by entity_type, entity_id, and event_type.

## Router composition

In the multi-module example, the app is constructed roughly as:

- Build the host with `ServerBuilder` and register modules.
- Create exposure routers: `RestExposure::build_router(host, vec![])`, `GraphQLExposure::build_router(host)`, etc.
- Merge all routers into a single Axum `Router` with `Router::new().merge(...)`.

This composition keeps the domain independent from transport concerns while offering multiple client options.
