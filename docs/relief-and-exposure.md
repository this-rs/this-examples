# The relief, and the exposure chain

Two things a service built with this-rs publishes about itself: **what its
database contains** (the relief) and **who may reach it, by which door** (the
exposure).

## 1. The relief — `GET /graph`

| Section | What it says |
|---|---|
| `store` | **the matter**: node and edge totals, every label with its node count, every edge type with its count, the property-key catalogue, the declared indexes |
| `engine` | substrate or not, read-only, registry available, simplex configuration |
| `space` | the active named space, and the spaces that exist |
| `procedures` | the engine's procedures, **introspected** from the registry |
| `families` | dispatch families outside the registry — **declared**, and said to be |
| `identity` | the canonical identity contract: motives, index size, and the address of the full contract |
| `lease` | the write lease: armed or not, epoch, shard, authority |
| `journal` | durable journal availability |
| `iam` | **IAM as a native layer**: schema version, decision count, the property the enforcement reads at each request (`acl_perms_blob`), and every label, property and edge with its meaning and whether the framework *writes* it or only *reads* it |
| `surfaces` | which surface enforces the policies, which cannot, and what protects it otherwise |
| `traps` | known limits, each tied to `file:line` |
| `notes` | reading notes (provenance, features, costs) |

Everything in `store` is **introspected from the live base** — what a backend
cannot introspect is published empty, never estimated. `origin` says which.

Related reads: `GET /graph/identity` (the identity contract, `?counts=true` for a
bounded count), `GET /graph/identity/{uuid}` (one entity's identity chain),
`GET /graph/spaces`, `GET /graph/procedures`.

## 2. The multi-scale view — `GET /graph/hierarchy/{id}`

```
GET /graph/hierarchy/42?depth=3&limit=50          # JSON, three levels
GET /graph/hierarchy/42?depth=2&format=outline    # indented text, for a model
```

```
aletheia (2 enfant(s))
  - g1-0 (2)
    - g2-0 (2)
    - g2-1 (2)
  - g1-1 (2)
    ↳ /graph/hierarchy/17?depth=1
```

* one bounded level per call: `depth` (≤ 32), `limit` per level (≤ 1000);
* `children` on every node, so you know what is below **without loading it**;
* `total` and `truncated` per level: a cut is **never** silent;
* `drill_down` on every unfinished branch: that is how a client — or a model —
  walks the graph step by step;
* cycles are **reported, never followed**;
* traversal is by **adjacency**, filtered by containment edge type, so the cost
  follows what is *shown*, not the size of the database.

## 3. The exposure — `config/server.yaml`

```yaml
cors:    { enabled: true, origins: [...], credentials: true }
domains: { allowed: [...], sites: [...], trust_forwarded_host: true, tenants: {...} }
proxy:   { caddy: { enabled: true, upstream: ..., upstreams: {...} } }
dns:     { api_url: ..., api_key_env: PDNS_API_KEY, zone: ..., target: ..., }
```

One declaration, three delegations:

```
domains.sites ──► PowerDNS  (A / AAAA / CNAME of the declared hosts only)
              ──► Caddy     (TLS, routing, per-site upstreams, security headers)
              ──► this-rs   (CORS, host guard, authentication, policies)
```

Disabled by default, and **described rather than applied**: this-rs renders a
`Caddyfile` and a DNS **plan**; the operator reviews and applies. The DNS
delegation touches no MX, TXT or NS record, deletes nothing without
`prune: true`, and reads its API key from the environment — never from the
versioned file.

`ServerBuilder::with_server_config_file("config/server.yaml")` is the whole
wiring; `host.exposure()` returns counts and facts (never the lists) for a health
endpoint or a dashboard.
