# Documentation

Complete documentation for the this-examples project.

## Table of Contents

### Getting Started

- **[Overview](overview.md)** - High-level introduction to the project goals, layout, and how components fit together
- **[Quick Start](../README.md#quick-start)** - Running the examples (REST and GraphQL)

### Architecture & Design

- **[Architecture](architecture.md)** - Core components, data flow, host vs. exposure pattern
- **[Entities and Macros](entities-and-macros.md)** - Entity structure, file nomenclature, adding new entities
- **[Best Practices](best-practices.md)** - Project conventions, entity patterns, error handling, testing

### Protocols & APIs

- **[Protocols: REST and GraphQL](protocols.md)** - Dual protocol exposure, router composition
- **[REST API Guide](rest-api.md)** - REST endpoints, usage examples, testing
- **[Nested Routes](nested-routes.md)** - Entity relationships via auto-generated nested REST routes
- **[GraphQL Playground](graphql-playground.md)** - Using the GraphQL interface, queries, mutations

### Advanced Topics

- **[Links and Relations](links-and-relations.md)** - Managing relationships between entities
- **[Development Workflow](development.md)** - Adding features, iterating, debugging
- **[Testing and Seeded Data](testing.md)** - In-memory stores, test data provisioning

## Crate Documentation

- **[test-data](../crates/test-data/README.md)** - Data provisioning for demos and tests
- **[billing](../crates/billing/)** - Domain module with orders, invoices, and payments

## Quick Links

- [Main README](../README.md)
- [Repository Structure](overview.md#repository-layout)
- [GraphQL Schema](../README.md#run-the-graphql--rest-example)

---

Each guide is self-contained and can be read independently, but following the order above provides a natural learning path.
