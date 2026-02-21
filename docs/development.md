# Development

This document covers local setup, running the examples, and useful flags.

## Prerequisites

- Rust stable toolchain (via rustup)
- A free TCP port 4242

## Install dependencies

The examples use only crates from crates.io and the local workspace; `cargo run` will fetch what is needed automatically.

## Running the servers

GraphQL + REST:

```bash
cargo run -p graphql_example
```

- Playground: `http://127.0.0.1:4242/graphql/playground`
- REST endpoints: `GET /health`, `GET /orders`, `GET /invoices`, `GET /payments`

REST-only:

```bash
cargo run -p rest_example
```

- Base URL: `http://0.0.0.0:4242`

gRPC:

```bash
cargo run -p grpc_example
```

- Server: `http://127.0.0.1:4244`

WebSocket:

```bash
cargo run -p websocket_example
```

- Server: `http://127.0.0.1:4243`
- WebSocket: `ws://127.0.0.1:4243/ws`

Multi-module (all transports):

```bash
cargo run -p multi_module_example
```

## Logging

All examples initialize `tracing_subscriber` for structured logs. Adjust with `RUST_LOG` env var, e.g.:

```bash
RUST_LOG=info cargo run -p graphql_example
```

## Feature flags

Feature flags are configured per-example in each example's `Cargo.toml`. You don't need to pass `--features` manually — each example already enables the features it needs.

## Hot reload tips

- For rapid iteration, use `cargo watch` (optional):

```bash
cargo install cargo-watch
cargo watch -x 'run -p graphql_example'
```

Or use `this-cli`:

```bash
cargo install this-cli
cd examples/graphql
this dev --api-only
```

## Troubleshooting

- Port already in use: stop the other process or change the bind address in the example.
- No data visible: ensure `test-data` seeding runs before the host is built (the examples already do this).
