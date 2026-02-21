# Nested Routes

The this-rs framework automatically generates nested REST routes based on link configurations, enabling intuitive navigation through entity relationships.

## Overview

Nested routes allow you to navigate entity relationships directly through URLs:

- **Forward navigation**: Follow relationships from parent to child entities
- **Reverse navigation**: Navigate backwards from child to parent entities
- **Multi-level nesting**: Chain multiple relationships in a single URL

## Configuration

Links are defined in `crates/billing/config/links.yaml`:

```yaml
links:
  - link_type: has_invoice
    source_type: order
    target_type: invoice
    forward_route_name: invoices  # Creates /orders/{id}/invoices
    reverse_route_name: order     # Creates /invoices/{id}/order

  - link_type: payment
    source_type: invoice
    target_type: payment
    forward_route_name: payments  # Creates /invoices/{id}/payments
    reverse_route_name: invoice   # Creates /payments/{id}/invoice
```

## Generated Routes

### Entity Routes (flat)

Standard CRUD endpoints for each entity:

```
GET    /orders           - List all orders
GET    /invoices         - List all invoices
GET    /payments         - List all payments
```

### Nested Routes (relationships)

#### Order -> Invoices (forward)

```bash
# List all invoices for a specific order
GET /orders/{order_id}/invoices

# Example
curl -s http://127.0.0.1:4242/orders/550e8400-e29b-41d4-a716-446655440000/invoices | jq
```

#### Invoice -> Payments (forward)

```bash
# List all payments for a specific invoice
GET /invoices/{invoice_id}/payments

# Example
curl -s http://127.0.0.1:4242/invoices/123e4567-e89b-12d3-a456-426614174000/payments | jq
```

#### Invoice -> Order (reverse)

```bash
# Get the parent order for an invoice
GET /invoices/{invoice_id}/order

# Example
curl -s http://127.0.0.1:4242/invoices/123e4567-e89b-12d3-a456-426614174000/order | jq
```

#### Payment -> Invoice (reverse)

```bash
# Get the parent invoice for a payment
GET /payments/{payment_id}/invoice

# Example
curl -s http://127.0.0.1:4242/payments/987fcdeb-51a2-43f7-b123-456789abcdef/invoice | jq
```

## Working with Dynamic IDs

Entity IDs are auto-generated UUIDs. Here's how to work with them:

### Get an ID from a list

```bash
# Get the first order ID (entity list returns a flat JSON array)
ORDER_ID=$(curl -s http://127.0.0.1:4242/orders | jq -r '.[0].id')

# Use it to get invoices
curl -s "http://127.0.0.1:4242/orders/$ORDER_ID/invoices" | jq
```

### Chain multiple calls

```bash
# Get order -> invoice -> payments
ORDER_ID=$(curl -s http://127.0.0.1:4242/orders | jq -r '.[0].id')
INVOICE_ID=$(curl -s "http://127.0.0.1:4242/orders/$ORDER_ID/invoices" | jq -r '.data[0].target_id')
curl -s "http://127.0.0.1:4242/invoices/$INVOICE_ID/payments" | jq
```

## Multi-level Nesting

The framework supports unlimited depth for nested routes by **chaining link definitions**.

### How Link Chaining Works

The YAML configuration defines individual links:

```yaml
# Link 1: Order -> Invoice
links:
  - link_type: has_invoice
    source_type: order
    target_type: invoice
    forward_route_name: invoices

# Link 2: Invoice -> Payment
  - link_type: payment
    source_type: invoice
    target_type: payment
    forward_route_name: payments
```

The framework **automatically chains** these links to create multi-level routes:

```
order -> invoice (link 1) -> payment (link 2)
```

This generates the route:
```
GET /orders/{order_id}/invoices/{invoice_id}/payments
```

### Real Example

With the billing configuration, you can navigate through the complete chain:

```bash
# Access a specific payment through its parent invoice and grandparent order
curl http://localhost:4242/orders/0052a960-6707-4028-aa8f-29de26e47106/invoices/2d7a08d8-1b05-4396-af1d-bc4ea29d5ac8/payments/adfe6522-7b0e-4829-9472-420d011fa539
```

This URL has **two levels of nesting**:
1. **Level 1**: `orders/{id}/invoices/{id}` - follows the `has_invoice` link
2. **Level 2**: `invoices/{id}/payments/{id}` - follows the `payment` link

### Automatic Validation

Each level in the chain is validated:
- The order must exist
- The invoice must exist AND be linked to that order
- The payment must exist AND be linked to that invoice

If any link in the chain is broken, the request returns a 404.

### Unlimited Depth

You can chain as many levels as needed. If you had:
```yaml
order -> invoice -> payment -> refund -> audit_log
```

The framework would generate:
```
GET /orders/{id}/invoices/{id}/payments/{id}/refunds/{id}/audit_logs
```

**No code changes required** - just define the links in YAML!

## Response Format

Nested link routes return a paginated enriched response:

```json
{
  "data": [
    {
      "id": "link-uuid",
      "type": "invoice",
      "link_type": "has_invoice",
      "source_id": "order-uuid",
      "target_id": "invoice-uuid",
      "target": {
        "id": "invoice-uuid",
        "number": "INV-001",
        "amount": 1500.00,
        "status": "paid"
      },
      "metadata": null,
      "created_at": "2025-01-01T00:00:00Z",
      "updated_at": "2025-01-01T00:00:00Z",
      "status": "active"
    }
  ],
  "pagination": {
    "page": 1,
    "per_page": 50,
    "total": 1,
    "total_pages": 1
  },
  "link_type": "has_invoice",
  "direction": "Forward",
  "description": null
}
```

Each item in `data` is an enriched link containing the full target entity (not just its ID).

## Link Metadata

Links can carry additional metadata (payment method, creation date, etc.). This metadata is accessible through the link service API.

## Adding New Nested Routes

To add new nested routes:

1. Define the link in `config/links.yaml`
2. Specify `forward_route_name` and `reverse_route_name`
3. Restart the server - routes are auto-generated

No code changes required!
