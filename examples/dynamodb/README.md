# DynamoDB Example

This example demonstrates how to use This-RS with DynamoDB as the storage backend.

## Prerequisites

- Docker and Docker Compose
- AWS CLI (optional, for real AWS usage)

## Local Development Setup

### 1. Start Local DynamoDB

```bash
docker-compose up -d
```

This will start:
- **DynamoDB Local** on port 8000
- **DynamoDB Admin UI** on port 8001 (http://localhost:8001)

### 2. Configure Environment Variables

Create a `.env` file or set these environment variables:

```bash
# For local development
export AWS_ENDPOINT_URL=http://localhost:8000
export AWS_ACCESS_KEY_ID=dummy
export AWS_SECRET_ACCESS_KEY=dummy
export AWS_DEFAULT_REGION=us-east-1

# Table names (optional, will use defaults if not set)
export ORDERS_TABLE_NAME=orders
export INVOICES_TABLE_NAME=invoices
export PAYMENTS_TABLE_NAME=payments
export LINKS_TABLE_NAME=links
```

### 3. Create Tables (Optional)

The application will create tables automatically, but you can create them manually if needed:

```bash
# Create orders table
aws dynamodb create-table \
    --endpoint-url http://localhost:8000 \
    --table-name orders \
    --attribute-definitions AttributeName=id,AttributeType=S \
    --key-schema AttributeName=id,KeyType=HASH \
    --provisioned-throughput ReadCapacityUnits=5,WriteCapacityUnits=5

# Create invoices table
aws dynamodb create-table \
    --endpoint-url http://localhost:8000 \
    --table-name invoices \
    --attribute-definitions AttributeName=id,AttributeType=S \
    --key-schema AttributeName=id,KeyType=HASH \
    --provisioned-throughput ReadCapacityUnits=5,WriteCapacityUnits=5

# Create payments table
aws dynamodb create-table \
    --endpoint-url http://localhost:8000 \
    --table-name payments \
    --attribute-definitions AttributeName=id,AttributeType=S \
    --key-schema AttributeName=id,KeyType=HASH \
    --provisioned-throughput ReadCapacityUnits=5,WriteCapacityUnits=5

# Create links table
aws dynamodb create-table \
    --endpoint-url http://localhost:8000 \
    --table-name links \
    --attribute-definitions AttributeName=id,AttributeType=S \
    --key-schema AttributeName=id,KeyType=HASH \
    --provisioned-throughput ReadCapacityUnits=5,WriteCapacityUnits=5
```

### 4. Run the Application

```bash
cargo run --features dynamodb
```

## Usage

### REST API Endpoints

- **Orders**
  - `GET /order` - List all orders
  - `POST /order` - Create a new order
  - `GET /order/{id}` - Get order by ID
  - `DELETE /order/{id}` - Delete order

- **Invoices**
  - `GET /invoice` - List all invoices
  - `POST /invoice` - Create a new invoice
  - `GET /invoice/{id}` - Get invoice by ID
  - `DELETE /invoice/{id}` - Delete invoice

- **Payments**
  - `GET /payment` - List all payments
  - `POST /payment` - Create a new payment
  - `GET /payment/{id}` - Get payment by ID
  - `DELETE /payment/{id}` - Delete payment

### GraphQL (if enabled)

- `POST /graphql` - GraphQL endpoint
- `GET /graphql/playground` - GraphQL Playground
- `GET /graphql/schema` - GraphQL schema

## DynamoDB Admin UI

Access the DynamoDB Admin UI at http://localhost:8001 to:
- View tables and data
- Run queries
- Manage table schemas

## Real AWS Usage

To use with real AWS DynamoDB:

1. Remove or comment out `AWS_ENDPOINT_URL`
2. Set proper AWS credentials:
   ```bash
   export AWS_ACCESS_KEY_ID=your_access_key
   export AWS_SECRET_ACCESS_KEY=your_secret_key
   export AWS_DEFAULT_REGION=your_region
   ```
3. Make sure the tables exist in your AWS account
4. Run the application

## Cleanup

```bash
# Stop local DynamoDB
docker-compose down

# Remove volumes (optional)
docker-compose down -v
```

## Troubleshooting

### Connection Issues

If you get connection errors:
1. Make sure DynamoDB Local is running: `docker-compose ps`
2. Check if port 8000 is available: `lsof -i :8000`
3. Verify environment variables are set correctly

### Table Creation Issues

If tables aren't created automatically:
1. Check AWS credentials and permissions
2. Create tables manually using the AWS CLI commands above
3. Verify the endpoint URL is correct

### Data Not Persisting

DynamoDB Local runs in-memory by default. To persist data:
1. Remove the `-inMemory` flag from docker-compose.yml
2. Add a volume mount for data persistence