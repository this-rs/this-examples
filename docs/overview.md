# Overview

This repository showcases how to structure modular backends with the this-rs framework using a real-world inspired billing domain. It focuses on:

- Isolating domain logic in crates under `crates/`.
- Enforcing a uniform entity structure across the domain for predictability.
- Building a protocol-agnostic host and exposing it via REST and GraphQL.
- Demonstrating links/relations between entities using a link service.

## Goals

- Provide copy-paste runnable examples to kickstart new services.
- Encourage clean boundaries between domain, storage, and transport layers.
- Showcase dual exposure (REST and GraphQL) for the same domain model.
- Promote testable design through in-memory stores and seeded data.

## Repository layout

```
crates/
  billing/           # Domain module (orders, invoices, payments)
  test-data/         # Data seeding helpers for demos & tests
examples/
  graphql/           # REST + GraphQL exposure, playground included
  rest/              # REST-only exposure
```

## How things fit together

- `crates/billing` defines the domain: entities, stores, handlers, and descriptors.
- `examples/*` assemble a server by registering modules into a host and attaching protocol exposures.
- `test-data` seeds in-memory stores with sample data to make the examples meaningful.
- The GraphQL example merges REST and GraphQL routers while keeping domain concerns isolated inside the module crate.
