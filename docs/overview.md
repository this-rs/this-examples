# Overview

This repository showcases how to structure modular backends with the this-rs framework using real-world inspired domains (billing, catalog, inventory). It focuses on:

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
  catalog/           # Domain module (products, categories, tags)
  inventory/         # Domain module (stores, activities, warehouses, stock, usage)
  test-data/         # Data seeding helpers for demos & tests
examples/
  graphql/           # REST + GraphQL exposure, playground included
  rest/              # REST-only exposure
  dynamodb/          # DynamoDB backend example with local setup
  multi-module/      # Combines billing, catalog & inventory modules
```

## Domain modules

### Billing
Orders, invoices, and payments with a linear link chain (`order → invoice → payment`).

### Catalog
Products, categories, and tags. Features many-to-many relationships (`product ↔ category`, `product ↔ tag`) and reflexive hierarchical links (`category → category` for parent/child).

### Inventory
Multi-activity store management. A store can host multiple activities (e.g., bar + co-working), each with independent warehouses, stock items, stock movements, and usage tracking for refacturation. Cross-module links connect `stock_item` to catalog's `product`.

## How things fit together

- `crates/billing` defines the billing domain: orders, invoices, and payments.
- `crates/catalog` defines the catalog domain: products, categories (with hierarchy), and tags.
- `crates/inventory` defines the inventory domain: stores, activities, warehouses, stock items, movements, and usage tracking.
- `examples/*` assemble a server by registering modules into a host and attaching protocol exposures.
- `test-data` seeds in-memory stores with sample data to make the examples meaningful.
- The multi-module example demonstrates registering all three modules into a single host, including cross-module links (e.g., `stock_item → product`).
- The GraphQL example merges REST and GraphQL routers while keeping domain concerns isolated inside the module crate.
