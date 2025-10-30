# Entities and Macros

Each entity follows the same predictable structure and naming. This uniformity makes the codebase easier to navigate, extend, and generate transports from.

## Standard entity files

- `model.rs`: Entity data model (fields, types) and core semantics.
- `store.rs`: Persistence abstraction (in-memory in the examples). Responsible for CRUD.
- `handlers.rs`: Application-facing operations the host invokes for this entity.
- `descriptor.rs`: Declares the entity to the framework (names, routes, capabilities), enabling auto-wiring and code generation.

Example entity directories:

```
crates/billing/src/entities/
  order/
    model.rs
    store.rs
    handlers.rs
    descriptor.rs
  invoice/
    ...
  payment/
    ...
```

## Macros and descriptors

The framework provides macros and helper types used by descriptors and handlers to:

- Register entities into a module (type-safe metadata)
- Derive REST and GraphQL shapes
- Generate resolvers/routers based on descriptor specifications

In this example, descriptors define the entity name, expose CRUD operations, and integrate with the link service. The GraphQL schema shown in the README is generated from these registrations.

## Adding a new entity

1. Create a new entity directory with the four files listed above.
2. Implement the `model`, `store`, and `handlers` for CRUD and any domain-specific logic.
3. Define the `descriptor` to register the entity with the module.
4. Register the entity in your module (e.g., `BillingModule`) and rebuild. REST routes and GraphQL schema will update accordingly.
