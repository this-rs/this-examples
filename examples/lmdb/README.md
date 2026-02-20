# LMDB Example

Billing API using LMDB as the storage backend.

LMDB is an embedded key-value store — no Docker or external service needed. Data is stored in a local directory.

## Setup

1. Run the example directly:
   ```bash
   cargo run -p lmdb_example
   ```

2. Test the API:
   ```bash
   curl http://localhost:4242/orders
   ```

Data is stored in `./data/` by default.

## Configuration

| Variable        | Default  |
|-----------------|----------|
| `LMDB_DATA_DIR` | `./data` |
