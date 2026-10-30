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
cargo run  -p descriptor_example --features graphql,grpc
cargo test -p descriptor_example --features graphql,grpc
```

> **Branch caveat**: the `ModuleDescriptor` API lives on the framework's
> `integ/module-descriptor` branch (not yet merged into `main`). This example only
> compiles against that branch. To validate it against a framework worktree:
>
> ```sh
> # point the dependency at the integ worktree, then revert before committing
> sed -i '' 's#path = "../../../this"#path = "../../../_wt-md/integ"#' Cargo.toml
> cargo check -p descriptor_example --features graphql,grpc
> ```
