use std::sync::Arc;

use anyhow::Result;
use axum::Router;
use tower_http::services::ServeDir;

use billing::{BillingModule, BillingStores};
use test_data::populate_test_data;

use this::server::builder::ServerBuilder;
use this::server::{RestExposure, WebSocketExposure};
use this::storage::InMemoryLinkService;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let stores = BillingStores::new_in_memory();
    let billing_module = BillingModule::new(stores);

    let link_service_arc = Arc::new(InMemoryLinkService::new());

    // Populate test data BEFORE building the host (builder consumes the link service)
    populate_test_data(&billing_module.stores, link_service_arc.clone()).await?;

    // Build the transport-agnostic host
    // CRITICAL: with_event_bus(1024) is required for WebSocket to broadcast events
    let host = Arc::new(
        ServerBuilder::new()
            .with_link_service((*link_service_arc).clone())
            .with_event_bus(1024)
            .register_module(billing_module)?
            .build_host()?,
    );

    // Build REST + WebSocket routers
    let rest_router = RestExposure::build_router(host.clone(), vec![])?;
    let ws_router = WebSocketExposure::build_router(host.clone())?;

    // Serve static files (for the HTML WebSocket client)
    let static_files = ServeDir::new("examples/websocket/static");

    // Merge all routers
    let app = Router::new()
        .merge(rest_router)
        .merge(ws_router)
        .nest_service("/static", static_files);

    println!("\n🌐 Server running on http://127.0.0.1:4243");
    println!("\n📚 Endpoints disponibles:");
    println!("\n  REST API:");
    println!("    GET    /health");
    println!("    GET    /orders");
    println!("    GET    /invoices");
    println!("    GET    /payments");
    println!("\n  WebSocket:");
    println!("    WS     /ws");
    println!("\n  Client de test:");
    println!("    GET    /static/ws-client.html");
    println!(
        "\n💡 Ouvrez le client HTML dans un navigateur, puis utilisez curl pour créer des entités."
    );
    println!("   Les événements apparaîtront en temps réel dans le client WebSocket.\n");

    let listener = tokio::net::TcpListener::bind("127.0.0.1:4243").await?;
    axum::serve(listener, app).await?;

    Ok(())
}
