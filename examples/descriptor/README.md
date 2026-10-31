# ModuleDescriptor — one declaration, three protocols

A complete described module, declared **once** in a single `ModuleDescriptor`, and
served on REST, GraphQL and gRPC by the same host.

The `billing` module declares:

- an entity `invoice` with validators (`number`: 3..50 chars, `amount`: > 0),
  input transforms (trim + uppercase on `number`) and typed query capabilities;
- a typed filter: `amount` is comparable with `eq`/`gt`, `number` with `eq`, both
  are sortable; any other field or operator is refused with a stable code;
- an entity `customer` and the link `owner` (`invoice -> customer`);
- a policy: creating an invoice needs `role:admin`, everything else only an identity;
- messages in French and English (a module bundle on top of the framework catalog).

The same host then answers the same questions on the three protocols: an invalid
payload gets the **same contract code**, a forbidden caller the **same refusal**, a
filter the **same rows**, and the message comes back in the **caller's language**.

## Run / test

```sh
cargo run  -p descriptor_example
cargo test -p descriptor_example
```

> **Framework version**: this example demonstrates APIs that shipped in
> **this-rs v0.0.10** (the `ModuleDescriptor` integration, PR #71 merged into
> `main`). It depends on the framework through the workspace's single source of
> truth — `[workspace.dependencies] this-rs = { path = "../this" }` — so it
> always runs against the local checkout, with no version to bump.
>
> GraphQL and gRPC are **default features** of the example: the demo serves all
> three protocols at once, and an example that only builds when you remember a
> flag is a bad example. `--no-default-features` gives a REST-only run.
