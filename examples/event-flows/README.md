# Event Flows Example

Demonstrates the **this-rs declarative event system**: YAML-configured event flows that automatically trigger notifications when entities and links are created.

## Architecture

```
REST/WS mutations
       │
       ▼
    EventBus ──────────► SSE /events/stream
       │                 WS  /ws
       ▼
    EventLog
       │
       ▼
   FlowRuntime
       │
       ├─ notify-new-order      (entity.created → filter → map → deliver)
       ├─ notify-invoice-linked  (link.created  → resolve → map → deliver)
       └─ notify-payment-received(link.created  → resolve → map → deliver)
       │
       ▼
   SinkRegistry
       │
       └─ in-app ──► NotificationStore ──► GET /notifications/:user_id
```

## Run

```bash
cargo run -p event_flows_example
```

Open http://127.0.0.1:4245/static/index.html for the interactive test client.

## Scope

The framework has 8 flow operators: `map`, `resolve`, `deliver`, `filter`, `batch`, `deduplicate`, `fan_out`, `rate_limit`.

This demo exercises **3 of 8**: `map`, `resolve` and `deliver`, used by the three flows in `config/links.yaml`. Step 2 of the Quick Test triggers `notify-new-order`; step 4 triggers `notify-invoice-linked`. It does not exercise `filter`, `batch`, `deduplicate`, `fan_out` or `rate_limit`, and does not claim to.

`cargo test -p event_flows_example` checks the order API (create, then list with the seeded data). It does not assert that the flows deliver notifications: `/notifications/*` is refused by this demo's auth policy, so the inbox cannot be read back over REST.

## Quick Test (curl)

```bash
# 1. Watch events in real-time
curl -N http://127.0.0.1:4245/events/stream

# 2. Create an order (triggers notify-new-order flow)
curl -X POST http://127.0.0.1:4245/orders \
  -H 'Content-Type: application/json' \
  -d '{"name": "Laptop Pro", "number": "ORD-100", "amount": 1299.0, "status": "pending"}'

# 3. Check notifications
curl http://127.0.0.1:4245/notifications/system

# 4. Create an invoice and link it to the order
ORDER_ID=$(curl -s http://127.0.0.1:4245/orders | jq -r '.[0].id')
INVOICE_ID=$(curl -s -X POST http://127.0.0.1:4245/invoices \
  -H 'Content-Type: application/json' \
  -d '{"name": "INV-001", "status": "draft"}' | jq -r '.id')

curl -X POST "http://127.0.0.1:4245/orders/$ORDER_ID/invoices" \
  -H 'Content-Type: application/json' \
  -d "{\"target_id\": \"$INVOICE_ID\"}"

# 5. Check notifications again — should have 2 now
curl http://127.0.0.1:4245/notifications/system

# 6. Unread count
curl http://127.0.0.1:4245/notifications/system/unread-count

# 7. Mark all as read
curl -X POST http://127.0.0.1:4245/notifications/system/read-all
```

## YAML Configuration

The event system is configured in `config/links.yaml`:

- **`sinks:`** — Define delivery destinations (in_app, webhook, push, websocket, counter)
- **`events.backend:`** — Event storage backend (memory, nats, kafka, redis)
- **`events.flows:`** — Declarative pipelines (trigger, then `map`, `resolve`, `deliver`; see Scope)

See `config/links.yaml` for the full configuration.
