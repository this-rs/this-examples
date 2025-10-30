# Links and Relations

Cross-entity relationships are modeled via a link service. This keeps entities decoupled while enabling navigation and operations across them.

## Link service

- The examples use `InMemoryLinkService` for simplicity.
- The service stores generic links (source, target, type, metadata) and can be replaced by a persistent implementation.
- Links are configured in `crates/billing/config/links.yaml` with forward and reverse route names.

## Nested Routes

The framework automatically generates nested REST routes based on link configuration:

### Forward routes
- `GET /order/{id}/invoices` - List all invoices linked to an order
- `GET /invoice/{id}/payments` - List all payments linked to an invoice

### Reverse routes
- `GET /invoice/{id}/order` - Get the order linked to an invoice
- `GET /payment/{id}/invoice` - Get the invoice linked to a payment

**Note**: The `{id}` placeholders are dynamically replaced with actual entity IDs at runtime. IDs are auto-generated UUIDs.

### Link Chaining

The framework automatically chains links to create multi-level routes. When you define:

```yaml
order → invoice  (has_invoice link)
invoice → payment (payment link)
```

You get both individual routes AND the chained route:
```
GET /order/{id}/invoices
GET /invoice/{id}/payments
GET /order/{id}/invoices/{id}/payments  ← automatically generated!
```

Example:
```bash
curl http://localhost:4242/order/0052a960-6707-4028-aa8f-29de26e47106/invoices/2d7a08d8-1b05-4396-af1d-bc4ea29d5ac8/payments/adfe6522-7b0e-4829-9472-420d011fa539
```

This follows two links:
1. Order → Invoice (validates the invoice belongs to that order)
2. Invoice → Payment (validates the payment belongs to that invoice)

See **[Nested Routes](nested-routes.md)** for complete documentation.

## Generic links

- Use `createLink` and `deleteLink` GraphQL mutations to manage links at runtime.
- Payload includes `sourceId`, `targetId`, `linkType`, and optional `metadata`.

Example mutation:

```graphql
mutation LinkEntities($sourceId: ID!, $targetId: ID!) {
  createLink(sourceId: $sourceId, targetId: $targetId, linkType: "relates_to", metadata: { note: "demo" }) {
    id
    sourceId
    targetId
    linkType
  }
}
```

## Typed links

- When the domain requires stronger semantics, define typed links at the module level.
- Typed links typically constrain `linkType` and shape of `metadata` and may add helper resolvers/handlers.

## Best practices

- Start with generic links to keep the model flexible.
- Introduce typed links only when the domain benefits from explicit constraints and navigation.
- Keep link logic in the module layer; avoid leaking link internals into entity models.
