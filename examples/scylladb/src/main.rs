//! Billing API - ScyllaDB Example with This-RS

use std::env;
use std::sync::Arc;

use anyhow::Result;
use axum::Router;
use scylla::client::session::Session;
use scylla::client::session_builder::SessionBuilder;

use billing::{BillingModule, BillingStores};
use this::server::builder::ServerBuilder;
use this::server::RestExposure;
use this::storage::ScyllaLinkService;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Connect to ScyllaDB
    let scylla_uri =
        env::var("SCYLLA_URI").unwrap_or_else(|_| "localhost:9042".to_string());
    let keyspace = env::var("SCYLLA_KEYSPACE").unwrap_or_else(|_| "billing".to_string());

    let session: Arc<Session> = Arc::new(
        SessionBuilder::new()
            .known_node(&scylla_uri)
            .build()
            .await?,
    );
    println!("Connected to ScyllaDB at {}", scylla_uri);

    // Create keyspace if it doesn't exist
    session
        .query_unpaged(
            format!(
                "CREATE KEYSPACE IF NOT EXISTS {} WITH replication = {{'class': 'SimpleStrategy', 'replication_factor': 1}}",
                keyspace
            ),
            &[],
        )
        .await?;

    // Create ScyllaDB link service
    let link_service = ScyllaLinkService::new(session.clone(), keyspace.clone());

    // Create billing stores with ScyllaDB
    let stores = BillingStores::new_scylladb(session, keyspace);

    // Create the billing module
    let billing_module = BillingModule::new(stores);

    // Build the server host
    let host = Arc::new(
        ServerBuilder::new()
            .with_link_service(link_service)
            .register_module(billing_module)?
            .build_host()?,
    );

    // Build router
    let app = Router::new().merge(RestExposure::build_router(host.clone(), vec![])?);

    println!("\n🌐 Server running on http://0.0.0.0:4242");
    println!("\n📚 Available endpoints:");
    println!("    GET    /orders");
    println!("    POST   /orders");
    println!("    GET    /orders/{{id}}");
    println!("    DELETE /orders/{{id}}");
    println!("    GET    /invoices");
    println!("    POST   /invoices");
    println!("    GET    /invoices/{{id}}");
    println!("    DELETE /invoices/{{id}}");
    println!("    GET    /payments");
    println!("    POST   /payments");
    println!("    GET    /payments/{{id}}");
    println!("    DELETE /payments/{{id}}");

    let listener = tokio::net::TcpListener::bind("0.0.0.0:4242").await?;
    axum::serve(listener, app).await?;

    Ok(())
}
