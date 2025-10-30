# GraphQL Playground Guide

This guide shows how to run the GraphQL example, open the Playground, and try common queries and mutations. The schema is generated from the registered entities.

## Start the server

```
cargo run -p graphql_example --features graphql
```

- Playground: `http://127.0.0.1:4242/graphql/playground`
- Endpoint: `http://127.0.0.1:4242/graphql`
- Schema (SDL): `http://127.0.0.1:4242/graphql/schema`

No authentication is required for the demo; use the default headers.

## Query examples

List orders with pagination:

```graphql
query ListOrders {
  orders(limit: 10, offset: 0) {
    id
    number
    name
    amount
    status
    created_at
  }
}
```

Get one order by id:

```graphql
query GetOrder($id: ID!) {
  order(id: $id) {
    id
    number
    name
    amount
    customer_name
    notes
    status
    created_at
    updated_at
  }
}
```

List invoices and payments:

```graphql
query Lists {
  invoices(limit: 5) { id number amount status due_date }
  payments(limit: 5) { id number amount status method transaction_id }
}
```

## Mutation examples

Create/update/delete Order:

```graphql
mutation CreateOrder($data: JSON!) {
  createOrder(data: $data) {
    id
    number
    name
    amount
    status
  }
}
```

```graphql
mutation UpdateOrder($id: ID!, $data: JSON!) {
  updateOrder(id: $id, data: $data) {
    id
    name
    amount
    status
  }
}
```

```graphql
mutation DeleteOrder($id: ID!) {
  deleteOrder(id: $id)
}
```

Create/update/delete Invoice:

```graphql
mutation CreateInvoice($data: JSON!) {
  createInvoice(data: $data) { id number amount status due_date }
}
```

Create/update/delete Payment:

```graphql
mutation CreatePayment($data: JSON!) {
  createPayment(data: $data) { id number amount status method transaction_id }
}
```

### Variable examples

```json
{
  "data": {
    "number": "ORD-2025-001",
    "name": "Example order",
    "amount": 199.99,
    "status": "PENDING",
    "customer_name": "Jane Doe",
    "notes": "Demo order created from Playground"
  }
}
```

## Links (generic)

The schema includes generic link mutations:

```graphql
mutation LinkEntities($sourceId: ID!, $targetId: ID!) {
  createLink(sourceId: $sourceId, targetId: $targetId, linkType: "relates_to", metadata: { note: "example" }) {
    id
    sourceId
    targetId
    linkType
  }
}
```

```graphql
mutation Unlink($id: ID!) {
  deleteLink(id: $id)
}
```

Adjust `linkType` and `metadata` to your needs; typed links can be added in the module when needed.
