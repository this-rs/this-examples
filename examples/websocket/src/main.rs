use std::sync::Arc;

use anyhow::Result;
use axum::Router;
use tower_http::services::ServeDir;

use billing::{BillingModule, BillingStores};
use test_data::populate_test_data;

use this::config::LinksConfig;
use this::core::module::Module;
use this::core::{EntityCreator, EntityFetcher};
use this::server::builder::ServerBuilder;
use this::server::entity_registry::EntityRegistry;
use this::server::{RestExposure, WebSocketExposure};
use this::storage::InMemoryLinkService;

/// Config-only module that adds event flows and sinks.
/// Entities come from BillingModule; this module only provides event configuration.
struct EventsConfigModule;

impl Module for EventsConfigModule {
    fn name(&self) -> &str {
        "events-config"
    }
    fn version(&self) -> &str {
        "0.1.0"
    }
    fn entity_types(&self) -> Vec<&str> {
        vec![]
    }
    fn register_entities(&self, _registry: &mut EntityRegistry) {}
    fn links_config(&self) -> Result<LinksConfig> {
        let config_path = concat!(env!("CARGO_MANIFEST_DIR"), "/config/events.yaml");
        LinksConfig::from_yaml_file(config_path)
    }
    fn get_entity_fetcher(&self, _: &str) -> Option<Arc<dyn EntityFetcher>> {
        None
    }
    fn get_entity_creator(&self, _: &str) -> Option<Arc<dyn EntityCreator>> {
        None
    }
}

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
    // EventsConfigModule adds event flows (notify on entity creation) + in-app sink
    let host = Arc::new(
        ServerBuilder::new()
            .with_link_service((*link_service_arc).clone())
            .with_event_bus(1024)
            .register_module(billing_module)?
            .register_module(EventsConfigModule)?
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
    println!("\n  Events (real-time):");
    println!("    GET    /events/stream              (SSE)");
    println!("    WS     /ws                         (WebSocket)");
    println!("\n  Notifications:");
    println!("    GET    /notifications/system");
    println!("    GET    /notifications/system/unread-count");
    println!("    POST   /notifications/system/read-all");
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
