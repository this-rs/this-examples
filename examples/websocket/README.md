# WebSocket Example

Real-time event notifications using `this-rs` WebSocket exposure with the billing module.

## What this demonstrates

- **REST + WebSocket** on the same server using `RestExposure` + `WebSocketExposure`
- **EventBus** integration for broadcasting mutation events to connected clients
- **Subscription filters** — clients can subscribe to specific entity types, event types, or individual entities
- **HTML client** for visual debugging and demo

## Architecture

```
Client ──ws──▶ /ws ──▶ ConnectionManager
                              │
                     subscribe(filter)
                              │
           EventBus ──broadcast──▶ filter ──▶ Client

REST ──POST /orders──▶ Handler ──▶ EventBus.publish(Created)
```

When a REST mutation (POST, PUT, DELETE) occurs, the framework publishes an event to the `EventBus`. The `WebSocketExposure` subscribes to the bus and dispatches matching events to connected clients based on their subscription filters.

## Quick start

```bash
# Start the server (from this-examples root)
cargo run -p websocket_example

# Open the HTML client in your browser
open http://127.0.0.1:4243/static/ws-client.html

# In the client:
# 1. Click "Connect"
# 2. Click "Subscribe" (leave filters as * to receive all events)
# 3. Trigger events via curl in another terminal:
curl -X POST http://127.0.0.1:4243/orders \
  -H "Content-Type: application/json" \
  -d '{"name":"Test Order","status":"pending","number":"ORD-100","amount":42.0}'
```

## Endpoints

### REST API

| Method | Path | Description |
|--------|------|-------------|
| GET | `/health` | Health check |
| GET | `/orders` | List all orders |
| POST | `/orders` | Create an order |
| GET | `/orders/{id}` | Get an order |
| PUT | `/orders/{id}` | Update an order |
| DELETE | `/orders/{id}` | Delete an order |
| GET | `/invoices` | List all invoices |
| GET | `/payments` | List all payments |

### WebSocket

| Path | Description |
|------|-------------|
| `/ws` | WebSocket endpoint |

### Static files

| Path | Description |
|------|-------------|
| `/static/ws-client.html` | Interactive WebSocket client |

## WebSocket Protocol

### Client → Server

```json
// Subscribe to events (all filter fields are optional)
{"type": "subscribe", "filter": {"entity_type": "order", "event_type": "created"}}

// Unsubscribe
{"type": "unsubscribe", "subscription_id": "sub_abc123"}

// Keepalive
{"type": "ping"}
```

### Server → Client

```json
// Welcome (on connection)
{"type": "welcome", "connection_id": "..."}

// Subscription confirmed
{"type": "subscribed", "subscription_id": "sub_abc123", "filter": {...}}

// Event notification
{"type": "event", "subscription_id": "sub_abc123", "data": {"event": {...}, "timestamp": "..."}}

// Unsubscription confirmed
{"type": "unsubscribed", "subscription_id": "sub_abc123"}

// Keepalive response
{"type": "pong"}

// Error
{"type": "error", "message": "..."}
```

### Subscription Filters

All fields are optional. When omitted, the field acts as a wildcard (matches everything). Multiple fields are combined with AND logic.

| Field | Values | Description |
|-------|--------|-------------|
| `entity_type` | `order`, `invoice`, `payment` | Filter by entity type |
| `event_type` | `created`, `updated`, `deleted` | Filter by action |
| `kind` | `entity`, `link` | Filter entity vs link events |
| `entity_id` | UUID | Filter by specific entity |

### Examples

```bash
# Subscribe to all events
{"type": "subscribe", "filter": {}}

# Subscribe to order creations only
{"type": "subscribe", "filter": {"entity_type": "order", "event_type": "created"}}

# Subscribe to all deletions
{"type": "subscribe", "filter": {"event_type": "deleted"}}

# Subscribe to link events only
{"type": "subscribe", "filter": {"kind": "link"}}
```

## Key implementation details

- `ServerBuilder::with_event_bus(1024)` is **required** for WebSocket to broadcast events
- `populate_test_data()` must be called **before** `build_host()` (the builder consumes the link service)
- `RestExposure::build_router(host, vec![])` takes a `Vec<Router>` for custom routes (empty here)
- Static files are served via `tower_http::services::ServeDir`
