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

use this::server::builder::ServerBuilder;
use this::server::{GraphQLExposure, GrpcExposure, RestExposure, WebSocketExposure};
use this::storage::InMemoryLinkService;

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

    // Build the transport-agnostic host with all three modules
    // CRITICAL: with_event_bus(1024) is required for WebSocket to broadcast events
    let host = Arc::new(
        ServerBuilder::new()
            .with_link_service((*link_service).clone())
            .with_event_bus(1024)
            .register_module(billing_module)?
            .register_module(catalog_module)?
            .register_module(inventory_module)?
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
    println!("\n📚 Endpoints disponibles:");
    println!("\n  REST API - Billing:");
    println!("    GET    /order");
    println!("    GET    /invoice");
    println!("    GET    /payment");
    println!("    GET    /order/{{id}}/invoices");
    println!("    GET    /invoice/{{id}}/payments");
    println!("\n  REST API - Catalog:");
    println!("    GET    /product");
    println!("    GET    /category");
    println!("    GET    /tag");
    println!("    GET    /product/{{id}}/categories");
    println!("    GET    /category/{{id}}/products");
    println!("    GET    /product/{{id}}/tags");
    println!("    GET    /tag/{{id}}/products");
    println!("    GET    /category/{{id}}/children");
    println!("    GET    /category/{{id}}/parent");
    println!("\n  REST API - Inventory:");
    println!("    GET    /store");
    println!("    GET    /activity");
    println!("    GET    /warehouse");
    println!("    GET    /stock_item");
    println!("    GET    /stock_movement");
    println!("    GET    /usage");
    println!("    GET    /store/{{id}}/activities");
    println!("    GET    /activity/{{id}}/stores");
    println!("    GET    /store/{{id}}/warehouses");
    println!("    GET    /warehouse/{{id}}/stock_items");
    println!("    GET    /stock_item/{{id}}/movements");
    println!("    GET    /stock_item/{{id}}/product");
    println!("    GET    /activity/{{id}}/usages");
    println!("    GET    /usage/{{id}}/from_activity");
    println!("\n  GraphQL API:");
    println!("    POST   /graphql");
    println!("    GET    /graphql/playground");
    println!("    GET    /graphql/schema");
    println!("\n  gRPC Services (HTTP/2):");
    println!("    this_grpc.EntityService  (GetEntity, ListEntities, CreateEntity, UpdateEntity, DeleteEntity)");
    println!("    this_grpc.LinkService    (CreateLink, GetLink, FindLinksBySource, FindLinksByTarget, DeleteLink)");
    println!("    GET    /grpc/proto");
    println!("\n  WebSocket:");
    println!("    WS     /ws");
    println!("\n  Client de test WebSocket:");
    println!("    GET    /static/ws-client.html");
    println!("\n💡 Exemples:");
    println!("   curl http://127.0.0.1:4242/order");
    println!("   curl -X POST http://127.0.0.1:4242/graphql -H 'Content-Type: application/json' -d '{{\"query\": \"{{ orders {{ id name }} }}\"}}' ");
    println!("   curl -s http://127.0.0.1:4242/grpc/proto > /tmp/this.proto && grpcurl -plaintext -proto /tmp/this.proto 127.0.0.1:4242 this_grpc.EntityService/ListEntities");
    println!();

    let listener = tokio::net::TcpListener::bind("127.0.0.1:4242").await?;
    axum::serve(listener, app).await?;

    Ok(())
}
