// NOTE: This example requires `protoc` (Protocol Buffers compiler) in your PATH.
// Install via: brew install protobuf (macOS) or apt install protobuf-compiler (Linux)

use std::sync::Arc;

use anyhow::Result;

use billing::{BillingModule, BillingStores};
use test_data::populate_test_data;

use this::config::LinksConfig;
use this::core::module::Module;
use this::core::{EntityCreator, EntityFetcher};
use this::server::builder::ServerBuilder;
use this::server::entity_registry::EntityRegistry;
use this::server::{GrpcExposure, RestExposure};
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
    // EventsConfigModule adds event flows (notify on entity creation) + in-app sink
    let host = Arc::new(
        ServerBuilder::new()
            .with_link_service((*link_service_arc).clone())
            .with_event_bus(1024)
            .register_module(billing_module)?
            .register_module(EventsConfigModule)?
            .build_host()?,
    );

    // Build REST + gRPC routers
    let rest_router = RestExposure::build_router(host.clone(), vec![])?;
    let grpc_router = GrpcExposure::build_router(host.clone())?;

    // Merge routers — REST and gRPC coexist on the same port.
    // Both REST (link routes) and gRPC (tonic) install a fallback handler,
    // so we must use fallback_service to combine them without conflict.
    let app = rest_router.fallback_service(grpc_router);

    println!("\n🌐 Server running on http://127.0.0.1:4244");
    println!("\n📚 Endpoints disponibles:");
    println!("\n  REST API:");
    println!("    GET    /health");
    println!("    GET    /orders");
    println!("    POST   /orders");
    println!("    GET    /orders/{{id}}");
    println!("    PUT    /orders/{{id}}");
    println!("    DELETE /orders/{{id}}");
    println!("    GET    /invoices");
    println!("    GET    /payments");
    println!("\n  Events (real-time):");
    println!("    GET    /events/stream              (SSE)");
    println!("\n  Notifications:");
    println!("    GET    /notifications/system");
    println!("    GET    /notifications/system/unread-count");
    println!("    POST   /notifications/system/read-all");
    println!("\n  gRPC Services (HTTP/2):");
    println!("    this_grpc.EntityService  (GetEntity, ListEntities, CreateEntity, UpdateEntity, DeleteEntity)");
    println!("    this_grpc.LinkService    (CreateLink, GetLink, FindLinksBySource, FindLinksByTarget, DeleteLink)");
    println!("    this_grpc.NotificationService (ListNotifications, MarkAsRead, GetUnreadCount)");
    println!("\n  Proto export:");
    println!("    GET    /grpc/proto");
    println!("\n💡 Test with grpcurl:");
    println!("   curl -s http://127.0.0.1:4244/grpc/proto > /tmp/this.proto");
    println!("   grpcurl -plaintext -proto /tmp/this.proto 127.0.0.1:4244 this_grpc.EntityService/ListEntities");
    println!();

    let listener = tokio::net::TcpListener::bind("127.0.0.1:4244").await?;
    axum::serve(listener, app).await?;

    Ok(())
}
