# MySQL Example

Billing API using MySQL as the storage backend.

## Setup

1. Start MySQL:
   ```bash
   docker-compose up -d
   ```

2. Run the example:
   ```bash
   cargo run -p mysql_example
   ```

3. Test the API:
   ```bash
   curl http://localhost:4242/orders
   ```

## Configuration

| Variable       | Value                                                |
|----------------|------------------------------------------------------|
| `DATABASE_URL` | **Required**, e.g. `mysql://billing:<password>@127.0.0.1:3306/billing` |

`docker-compose.yml` publishes port 3306 on `127.0.0.1` only. Its passwords are
local-development defaults (`root` and `billing`); to change them, set
`MYSQL_ROOT_PASSWORD` / `MYSQL_PASSWORD` before `docker-compose up` and use the
same `MYSQL_PASSWORD` in `DATABASE_URL`.
