# MongoDB Example

Billing API using MongoDB as the storage backend.

## Setup

1. Start MongoDB:
   ```bash
   docker-compose up -d
   ```

2. Run the example:
   ```bash
   cargo run -p mongodb_example
   ```

3. Test the API:
   ```bash
   curl http://localhost:4242/orders
   ```

## Configuration

| Variable           | Default                        |
|--------------------|--------------------------------|
| `MONGODB_URI`      | `mongodb://localhost:27017`     |
| `MONGODB_DATABASE` | `billing`                      |
