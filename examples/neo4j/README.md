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

| Variable         | Default                    |
|------------------|----------------------------|
| `NEO4J_URI`      | `bolt://localhost:7687`    |
| `NEO4J_USER`     | `neo4j`                    |
| `NEO4J_PASSWORD` | `billing123`               |

## Neo4j Browser

Access the Neo4j browser at http://localhost:7474 to explore your data visually.
