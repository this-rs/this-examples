// NOTE: This example requires `protoc` (Protocol Buffers compiler) in your PATH
// for gRPC support. Install via: brew install protobuf (macOS) or apt install protobuf-compiler (Linux)

use std::sync::Arc;

use anyhow::Result;
use axum::Router;
use tower_http::services::ServeDir;

use billing::{BillingModule, BillingStores};
use catalog::{CatalogModule, CatalogStores};
use inventory::{InventoryModule, InventoryStores};
use test_data::{populate_catalog_data, populate_inventory_data, populate_test_data};

use this::config::LinksConfig;
use this::core::module::Module;
use this::core::{EntityCreator, EntityFetcher};
use this::server::builder::ServerBuilder;
use this::server::entity_registry::EntityRegistry;
use this::server::{GraphQLExposure, GrpcExposure, RestExposure, WebSocketExposure};
use this::storage::InMemoryLinkService;

/// Config-only module that adds event flows and sinks to the multi-module server.
/// Entities/links come from the domain modules (billing, catalog, inventory).
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

    let link_service = Arc::new(InMemoryLinkService::new());

    // Billing module
    let billing_stores = BillingStores::new_in_memory();
    populate_test_data(&billing_stores, link_service.clone()).await?;
    let billing_module = BillingModule::new(billing_stores);

    // Catalog module
    let catalog_stores = CatalogStores::new_in_memory();
    populate_catalog_data(&catalog_stores, link_service.clone()).await?;
    let catalog_module = CatalogModule::new(catalog_stores);

    // Inventory module
    let inventory_stores = InventoryStores::new_in_memory();
    populate_inventory_data(&inventory_stores, link_service.clone()).await?;
    let inventory_module = InventoryModule::new(inventory_stores);

    // Build the transport-agnostic host with all modules + event config
    //
    // The EventsConfigModule adds events/sinks from config/events.yaml.
    // ServerBuilder merges all configs and auto-wires the full event pipeline:
    //   EventBus → EventLog → FlowRuntime → SinkRegistry → InAppSink
    let host = Arc::new(
        ServerBuilder::new()
            .with_link_service((*link_service).clone())
            .with_event_bus(1024)
            .register_module(billing_module)?
            .register_module(catalog_module)?
            .register_module(inventory_module)?
            .register_module(EventsConfigModule)?
            .build_host()?,
    );

    // Build all exposure routers
    let rest_router = RestExposure::build_router(host.clone(), vec![])?;
    let graphql_router = GraphQLExposure::build_router(host.clone())?;
    let grpc_router = GrpcExposure::build_router(host.clone())?;
    let ws_router = WebSocketExposure::build_router(host.clone())?;

    // Serve static files (for the HTML WebSocket client)
    let static_files = ServeDir::new("examples/websocket/static");

    // Merge routers:
    // 1. Start with REST + GraphQL + WebSocket (standard merge)
    // 2. Use fallback_service for gRPC because both REST (link routes)
    //    and gRPC (tonic) install a fallback handler
    let app = Router::new()
        .merge(rest_router)
        .merge(graphql_router)
        .merge(ws_router)
        .nest_service("/static", static_files)
        .fallback_service(grpc_router);

    println!("\n🌐 Multi-Module Server running on http://127.0.0.1:4242");
    println!("\n📚 Endpoints:");
    println!("\n  REST API - Billing:");
    println!("    GET    /orders");
    println!("    GET    /invoices");
    println!("    GET    /payments");
    println!("    GET    /orders/{{id}}/invoices");
    println!("    GET    /invoices/{{id}}/payments");
    println!("\n  REST API - Catalog:");
    println!("    GET    /products");
    println!("    GET    /categories");
    println!("    GET    /tags");
    println!("    GET    /products/{{id}}/categories");
    println!("    GET    /categories/{{id}}/products");
    println!("    GET    /products/{{id}}/tags");
    println!("    GET    /tags/{{id}}/products");
    println!("    GET    /categories/{{id}}/children");
    println!("    GET    /categories/{{id}}/parent");
    println!("\n  REST API - Inventory:");
    println!("    GET    /stores");
    println!("    GET    /activities");
    println!("    GET    /warehouses");
    println!("    GET    /stock_items");
    println!("    GET    /stock_movements");
    println!("    GET    /usages");
    println!("    GET    /stores/{{id}}/activities");
    println!("    GET    /activities/{{id}}/stores");
    println!("    GET    /stores/{{id}}/warehouses");
    println!("    GET    /warehouses/{{id}}/stock_items");
    println!("    GET    /stock_items/{{id}}/movements");
    println!("    GET    /stock_items/{{id}}/product");
    println!("    GET    /activities/{{id}}/usages");
    println!("    GET    /usages/{{id}}/from_activity");
    println!("\n  Events (real-time):");
    println!("    GET    /events/stream              (SSE)");
    println!("    WS     /ws                         (WebSocket)");
    println!("\n  Notifications:");
    println!("    GET    /notifications/system");
    println!("    GET    /notifications/system/unread-count");
    println!("    POST   /notifications/system/read-all");
    println!("\n  GraphQL API:");
    println!("    POST   /graphql");
    println!("    GET    /graphql/playground");
    println!("    GET    /graphql/schema");
    println!("\n  gRPC Services (HTTP/2):");
    println!("    this_grpc.EntityService  (GetEntity, ListEntities, CreateEntity, UpdateEntity, DeleteEntity)");
    println!("    this_grpc.LinkService    (CreateLink, GetLink, FindLinksBySource, FindLinksByTarget, DeleteLink)");
    println!("    this_grpc.NotificationService (ListNotifications, MarkAsRead, GetUnreadCount)");
    println!("    GET    /grpc/proto");
    println!("\n  Client de test WebSocket:");
    println!("    GET    /static/ws-client.html");
    println!("\n💡 Quick test:");
    println!("   curl http://127.0.0.1:4242/orders");
    println!("   curl http://127.0.0.1:4242/notifications/system");
    println!();

    let listener = tokio::net::TcpListener::bind("127.0.0.1:4242").await?;
    axum::serve(listener, app).await?;

    Ok(())
}
