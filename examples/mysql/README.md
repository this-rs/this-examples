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

| Variable       | Default                                      |
|----------------|----------------------------------------------|
| `DATABASE_URL` | `mysql://billing:billing@localhost:3306/billing` |
