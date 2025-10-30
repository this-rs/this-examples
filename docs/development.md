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
cargo run -p graphql_example --features graphql
```

- Playground: `http://127.0.0.1:4242/graphql/playground`
- REST endpoints: `GET /health`, `GET /order`, `GET /invoice`, `GET /payment`

REST-only:

```bash
cargo run -p rest_example
```

- Base URL: `http://0.0.0.0:4242`

## Logging

Both examples initialize `tracing_subscriber` for structured logs. Adjust with `RUST_LOG` env var, e.g.:

```bash
RUST_LOG=info cargo run -p graphql_example --features graphql
```

## Feature flags

- `graphql` (in `examples/graphql`): enables the GraphQL exposure and routes. If disabled, the binary will warn and exit.

## Hot reload tips

- For rapid iteration, use `cargo watch` (optional):

```bash
cargo install cargo-watch
cargo watch -x 'run -p graphql_example --features graphql'
```

## Troubleshooting

- Port already in use: stop the other process or change the bind address in the example.
- No data visible: ensure `test-data` seeding runs before the host is built (the examples already do this).
