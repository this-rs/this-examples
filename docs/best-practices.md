# Best Practices

These conventions help maintain clarity, consistency, and extensibility across modules and entities.

## Project structure

- Keep each domain isolated in its own crate under `crates/`.
- Expose domains via transport adapters in `examples/` or in your application crate.
- Avoid coupling domain logic to transport concerns; the host composes them.

## Entity conventions

- Use the standard 4-file structure per entity: `model`, `store`, `handlers`, `descriptor`.
- Keep naming consistent across entities (ids, timestamps, numbers, status, etc.).
- Prefer explicit and typed fields in `model.rs`; avoid opaque `JSON` fields in the domain.

## Stores and data access

- Start with in-memory stores for fast iteration; introduce persistent stores behind the same interface later.
- Keep stores focused on CRUD and simple queries; move business rules to handlers.

## Error handling and tracing

- Use `anyhow` or typed errors where appropriate.
- Initialize `tracing_subscriber` early and add meaningful instrumentation at boundaries.

## Testing and data seeding

- Use `test-data` to seed representative data for demos and integration tests.
- Prefer testing handlers against in-memory stores for speed and determinism.

## Links/relations

- Model cross-entity relations via the link service; start with generic links.
- Introduce typed links only when the domain needs stronger semantics.

## Extensibility

- New entities should follow the same structure and register via descriptors.
- New protocols can be added as exposures without changing the domain.
