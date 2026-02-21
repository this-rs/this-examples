//! Billing API - MySQL Example with This-RS

use std::env;
use std::sync::Arc;

use anyhow::Result;
use axum::Router;

use billing::{BillingModule, BillingStores};
use this::server::builder::ServerBuilder;
use this::server::RestExposure;
use this::storage::MysqlLinkService;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Connect to MySQL
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "mysql://billing:billing@localhost:3306/billing".to_string());

    let pool = sqlx::MySqlPool::connect(&database_url).await?;
    println!("Connected to MySQL at {}", database_url);

    // Create MySQL link service
    let link_service = MysqlLinkService::new(pool.clone());

    // Create billing stores with MySQL
    let stores = BillingStores::new_mysql(pool);

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
