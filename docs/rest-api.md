# REST API Guide

This guide shows how to run the REST-only example and interact with the REST endpoints using curl.

## Start the server

```
cargo run -p rest_example
```

- Base URL: `http://0.0.0.0:4242`
- Endpoints (examples):
  - `GET /health`
  - `GET /order`
  - `GET /invoice`
  - `GET /payment`

The server uses in-memory stores and is pre-seeded with sample data.

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
