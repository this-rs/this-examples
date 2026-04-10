//! Auth STS Example — WAMI authentication with multi-tenant isolation
//!
//! Demonstrates:
//! 1. **Bootstrap STS**: Auto-generated Ed25519 keys at startup
//! 2. **JWT auth endpoints**: /auth/token, /auth/keys, /auth/refresh, /auth/revoke
//! 3. **Multi-tenant isolation**: Events and data scoped by tenant_id from JWT
//! 4. **GDPR erasure**: DELETE /tenants/:tenant_id/data cascades all entity + link data
//!
//! # Running
//!
//! ```sh
//! cargo run -p auth_sts_example
//! ```
//!
//! # Quick test
//!
//! ```sh
//! # 1. Get JWKS public key
//! curl http://127.0.0.1:4300/auth/keys | jq
//!
//! # 2. Create an order (no auth required in bootstrap mode — auth provider
//! #    logs a warning but doesn't block requests)
//! curl -X POST http://127.0.0.1:4300/orders \
//!   -H 'Content-Type: application/json' \
//!   -d '{"name": "Auth Test Order", "status": "pending", "owner_id": "alice"}'
//!
//! # 3. List orders
//! curl http://127.0.0.1:4300/orders | jq
//!
//! # 4. Check SSE stream (in another terminal)
//! curl -N http://127.0.0.1:4300/events/stream
//! ```

use std::sync::Arc;

use anyhow::Result;

use billing::{BillingModule, BillingStores};
use test_data::populate_test_data;

use this::server::builder::ServerBuilder;
use this::server::exposure::rest::RestExposure;
use this::storage::InMemoryLinkService;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    let stores = BillingStores::new_in_memory();
    let billing_module = BillingModule::new(stores);

    let link_service = Arc::new(InMemoryLinkService::new());

    // Populate test data before building host
    populate_test_data(&billing_module.stores, link_service.clone()).await?;

    // Build server with auth config from external YAML file
    //
    // The auth config is loaded separately from links.yaml:
    // - config/auth.yaml → AuthConfig (provider, tenant, GDPR, wami settings)
    // - Module's links_config() → entities, links, events, sinks
    //
    // ServerBuilder merges both and auto-wires:
    // - WamiAuthProvider (JWT verification) — skipped in bootstrap without public_key
    // - StsService (token issuance) — auto-generated keys in bootstrap mode
    // - STS endpoints (/auth/token, /auth/keys, /auth/refresh, /auth/revoke)
    // - GDPR erasure endpoint (DELETE /tenants/:tenant_id/data)
    // - EventBus for SSE + real-time events
    let host = Arc::new(
        ServerBuilder::new()
            .with_link_service((*link_service).clone())
            .with_default_event_bus()
            .with_default_notification_store()
            .with_auth_config_file(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/config/auth.yaml"
            ))?
            .register_module(billing_module)?
            .build_host()?,
    );

    let app = RestExposure::build_router(host, vec![])?;

    println!("\n  Auth STS Example running on http://127.0.0.1:4300");
    println!("\n  Auth endpoints (auto-wired from config/auth.yaml):");
    println!("    POST   /auth/token    Login (username/password -> JWT)");
    println!("    GET    /auth/keys     JWKS public key (Ed25519)");
    println!("    POST   /auth/refresh  Refresh access token");
    println!("    POST   /auth/revoke   Revoke token by JTI");
    println!("\n  GDPR:");
    println!("    DELETE /tenants/:tenant_id/data   Erasure cascade");
    println!("\n  REST API (Billing module):");
    println!("    GET/POST   /orders");
    println!("    GET/POST   /invoices");
    println!("    GET/POST   /payments");
    println!("\n  Events:");
    println!("    GET        /events/stream    (SSE)");
    println!("    GET        /notifications/system");
    println!();

    let listener = tokio::net::TcpListener::bind("127.0.0.1:4300").await?;
    axum::serve(listener, app).await?;

    Ok(())
}
