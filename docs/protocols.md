# Protocols: REST and GraphQL

The same transport-agnostic host is exposed via two protocols. You can run REST alone or combine REST and GraphQL on the same server.

## REST exposure

- Base routes (examples):
  - `GET /health`
  - `GET /order`
  - `GET /invoice`
  - `GET /payment`
- Handlers are derived from the entity descriptors and mapped to HTTP routes.
- The REST example binds to `0.0.0.0:4242` for easy testing from other devices.

## GraphQL exposure

- Endpoint: `POST /graphql`
- Playground UI: `GET /graphql/playground`
- Schema: `GET /graphql/schema`
- The GraphQL example binds to `127.0.0.1:4242` and merges REST + GraphQL routers.

## Router composition

In the GraphQL example, the app is constructed roughly as:

- Build the host with `ServerBuilder` and register the `BillingModule`.
- Create both REST and GraphQL routers from the host.
- Merge both routers into a single Axum `Router`.

This composition keeps the domain independent from transport concerns while offering multiple client options.
