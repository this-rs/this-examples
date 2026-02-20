# PostgreSQL Example

Billing API using PostgreSQL as the storage backend.

## Setup

1. Start PostgreSQL:
   ```bash
   docker-compose up -d
   ```

2. Run the example:
   ```bash
   cargo run -p postgres_example
   ```

3. Test the API:
   ```bash
   curl http://localhost:4242/orders
   ```

## Configuration

| Variable       | Default                                          |
|----------------|--------------------------------------------------|
| `DATABASE_URL` | `postgres://billing:billing@localhost:5432/billing` |
