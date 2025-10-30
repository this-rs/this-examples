# REST API Guide

This guide shows how to run the REST-only example and interact with the REST endpoints using curl.

## Start the server

```
cargo run -p rest_example
```

- Base URL: `http://0.0.0.0:4242`
- Entity endpoints:
  - `GET /order` - List all orders
  - `GET /invoice` - List all invoices
  - `GET /payment` - List all payments
  - `GET /health` - Health check
- Nested routes (entity relationships via links):
  - `GET /order/{id}/invoices` - List invoices for a specific order
  - `GET /invoice/{id}/payments` - List payments for a specific invoice
  - `GET /invoice/{id}/order` - Get the order for a specific invoice
  - `GET /payment/{id}/invoice` - Get the invoice for a specific payment

The server uses in-memory stores and is pre-seeded with sample data and links between entities.

## Examples with curl

List orders:

```bash
curl -s http://0.0.0.0:4242/order | jq
```

List invoices and payments:

```bash
curl -s http://0.0.0.0:4242/invoice | jq
curl -s http://0.0.0.0:4242/payment | jq
```

### Nested routes (relationships)

Get invoices for a specific order:

```bash
# First, get an order ID
ORDER_ID=$(curl -s http://0.0.0.0:4242/order | jq -r '.data[0].id')

# Then list its invoices
curl -s "http://0.0.0.0:4242/order/$ORDER_ID/invoices" | jq
```

Get payments for a specific invoice:

```bash
# First, get an invoice ID
INVOICE_ID=$(curl -s http://0.0.0.0:4242/invoice | jq -r '.data[0].id')

# Then list its payments
curl -s "http://0.0.0.0:4242/invoice/$INVOICE_ID/payments" | jq
```

Reverse navigation - get order from invoice:

```bash
# Get the order for a specific invoice
curl -s "http://0.0.0.0:4242/invoice/$INVOICE_ID/order" | jq
```

### Multi-level nested routes

The framework automatically chains links to create deep nested routes:

```bash
# Navigate through the complete chain: Order → Invoice → Payment
ORDER_ID=$(curl -s http://0.0.0.0:4242/order | jq -r '.data[0].id')
INVOICE_ID=$(curl -s "http://0.0.0.0:4242/order/$ORDER_ID/invoices" | jq -r '.data[0].id')
PAYMENT_ID=$(curl -s "http://0.0.0.0:4242/invoice/$INVOICE_ID/payments" | jq -r '.data[0].id')

# Access the payment through the full chain
curl -s "http://0.0.0.0:4242/order/$ORDER_ID/invoices/$INVOICE_ID/payments/$PAYMENT_ID" | jq
```

Example with real UUIDs:
```bash
curl http://localhost:4242/order/0052a960-6707-4028-aa8f-29de26e47106/invoices/2d7a08d8-1b05-4396-af1d-bc4ea29d5ac8/payments/adfe6522-7b0e-4829-9472-420d011fa539
```

This works because:
1. `order` → `invoice` link is defined in `config/links.yaml`
2. `invoice` → `payment` link is defined in `config/links.yaml`
3. The framework chains them automatically

See [Nested Routes](nested-routes.md) for detailed explanation of the chaining mechanism.

### Creating and updating entities

Create an order (if the REST handlers support POST in your setup, otherwise use GraphQL for mutations):

```bash
curl -s -X POST http://0.0.0.0:4242/order \
  -H 'Content-Type: application/json' \
  -d '{
    "number": "ORD-2025-001",
    "name": "Example order",
    "amount": 199.99,
    "status": "PENDING",
    "customer_name": "Jane Doe",
    "notes": "Created from curl"
  }' | jq
```

Update an order:

```bash
curl -s -X PUT http://0.0.0.0:4242/order/<id> \
  -H 'Content-Type: application/json' \
  -d '{ "status": "PAID" }' | jq
```

Delete an order:

```bash
curl -s -X DELETE http://0.0.0.0:4242/order/<id>
```

Note: Exact REST routes and verbs depend on how your descriptors map handlers to HTTP; the examples above illustrate typical usage. For full CRUD, prefer the GraphQL example which exposes all mutations out of the box.
