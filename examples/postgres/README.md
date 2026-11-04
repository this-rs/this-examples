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

| Variable       | Value                                                |
|----------------|------------------------------------------------------|
| `DATABASE_URL` | **Required**, e.g. `postgres://billing:<password>@127.0.0.1:5432/billing` |

`docker-compose.yml` publishes port 5432 on `127.0.0.1` only. Its password is a
local-development default (`billing`); to change it, set `POSTGRES_PASSWORD`
before `docker-compose up` and use the same value in `DATABASE_URL`.
