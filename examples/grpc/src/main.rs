// NOTE: This example requires `protoc` (Protocol Buffers compiler) in your PATH.
// Install via: brew install protobuf (macOS) or apt install protobuf-compiler (Linux)

use std::sync::Arc;

use anyhow::Result;

use billing::{BillingModule, BillingStores};
use test_data::populate_test_data;

use this::server::builder::ServerBuilder;
use this::server::{GrpcExposure, RestExposure};
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
    let host = Arc::new(
        ServerBuilder::new()
            .with_link_service((*link_service_arc).clone())
            .register_module(billing_module)?
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
    println!("\n  gRPC Services (HTTP/2):");
    println!("    this_grpc.EntityService  (GetEntity, ListEntities, CreateEntity, UpdateEntity, DeleteEntity)");
    println!("    this_grpc.LinkService    (CreateLink, GetLink, FindLinksBySource, FindLinksByTarget, DeleteLink)");
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
