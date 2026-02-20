//! Billing API - Neo4j Example with This-RS

use std::env;
use std::sync::Arc;

use anyhow::Result;
use axum::Router;

use billing::{BillingModule, BillingStores};
use this::server::builder::ServerBuilder;
use this::server::RestExposure;
use this::storage::Neo4jLinkService;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Connect to Neo4j
    let neo4j_uri = env::var("NEO4J_URI").unwrap_or_else(|_| "bolt://localhost:7687".to_string());
    let neo4j_user = env::var("NEO4J_USER").unwrap_or_else(|_| "neo4j".to_string());
    let neo4j_password = env::var("NEO4J_PASSWORD").unwrap_or_else(|_| "billing123".to_string());

    let graph = neo4rs::Graph::new(&neo4j_uri, &neo4j_user, &neo4j_password).await?;
    println!("Connected to Neo4j at {}", neo4j_uri);

    // Create Neo4j link service
    let link_service = Neo4jLinkService::new(graph.clone());

    // Create billing stores with Neo4j
    let stores = BillingStores::new_neo4j(graph);

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
