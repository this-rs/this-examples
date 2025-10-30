# Links and Relations

Cross-entity relationships are modeled via a link service. This keeps entities decoupled while enabling navigation and operations across them.

## Link service

- The examples use `InMemoryLinkService` for simplicity.
- The service stores generic links (source, target, type, metadata) and can be replaced by a persistent implementation.

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
