# ScyllaDB Example

Billing API using ScyllaDB as the storage backend.

## Setup

1. Start ScyllaDB:
   ```bash
   docker-compose up -d
   ```

2. Wait for ScyllaDB to be ready (~30s), then run:
   ```bash
   cargo run -p scylladb_example
   ```

3. Test the API:
   ```bash
   curl http://localhost:4242/orders
   ```

## Configuration

| Variable          | Default              |
|-------------------|----------------------|
| `SCYLLA_URI`      | `localhost:9042`     |
| `SCYLLA_KEYSPACE` | `billing`            |
