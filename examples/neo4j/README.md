# Neo4j Example

Billing API using Neo4j as the storage backend.

## Setup

1. Start Neo4j:
   ```bash
   docker-compose up -d
   ```

2. Run the example:
   ```bash
   cargo run -p neo4j_example
   ```

3. Test the API:
   ```bash
   curl http://localhost:4242/orders
   ```

## Configuration

| Variable         | Value                                   |
|------------------|-----------------------------------------|
| `NEO4J_URI`      | `bolt://localhost:7687` (default)       |
| `NEO4J_USER`     | `neo4j` (default)                       |
| `NEO4J_PASSWORD` | **Required**: the password of the Neo4j instance |

`docker-compose.yml` publishes ports 7474 and 7687 on `127.0.0.1` only. Its
password is a local-development default (`billing123`); to change it, set
`NEO4J_PASSWORD` before `docker-compose up` and export the same value when
running the example.

## Neo4j Browser

Access the Neo4j browser at http://localhost:7474 to explore your data visually.
