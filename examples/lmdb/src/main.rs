//! Billing API - LMDB Example with This-RS
//!
//! LMDB is an embedded key-value store — no external service needed.
//! Data is stored in a local directory.

use std::env;
use std::sync::Arc;

use anyhow::Result;
use axum::Router;

use billing::{BillingModule, BillingStores};
use this::server::builder::ServerBuilder;
use this::server::RestExposure;
use this::storage::LmdbLinkService;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Configure LMDB data directory
    let data_dir = env::var("LMDB_DATA_DIR").unwrap_or_else(|_| "./data".to_string());

    // Create data directory if it doesn't exist
    std::fs::create_dir_all(&data_dir)?;
    println!("Using LMDB data directory: {}", data_dir);

    // Create LMDB link service
    let links_path = format!("{}/links", data_dir);
    std::fs::create_dir_all(&links_path)?;
    let link_service = LmdbLinkService::open(&links_path)?;

    // Create billing stores with LMDB
    let stores_path = format!("{}/stores", data_dir);
    std::fs::create_dir_all(&stores_path)?;
    let stores = BillingStores::new_lmdb(&stores_path)?;

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
