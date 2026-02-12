//! Billing API - DynamoDB Example with This-RS

use std::env;
use std::sync::Arc;

use anyhow::Result;
use axum::Router;

use aws_sdk_dynamodb::Client;

use billing::{BillingModule, BillingStores};
use this::server::builder::ServerBuilder;
#[cfg(feature = "graphql")]
use this::server::GraphQLExposure;
use this::server::RestExposure;
use this::storage::DynamoDBLinkService;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Load AWS config with local DynamoDB endpoint if specified
    let mut config_loader = aws_config::defaults(aws_config::BehaviorVersion::latest());

    // Configure endpoint for local DynamoDB if AWS_ENDPOINT_URL is set
    if let Ok(endpoint_url) = env::var("AWS_ENDPOINT_URL") {
        config_loader = config_loader.endpoint_url(&endpoint_url);
        println!("Using DynamoDB endpoint: {}", endpoint_url);
    }

    let config = config_loader.load().await;
    let client = Client::new(&config);

    // Create DynamoDB link service
    let link_service = DynamoDBLinkService::new(
        client.clone(),
        env::var("LINKS_TABLE_NAME").unwrap_or_else(|_| "links".to_string()),
    );

    // Create billing stores with DynamoDB - each entity has its own table
    let stores = BillingStores::new_dynamodb(
        client,
        env::var("ORDERS_TABLE_NAME").unwrap_or_else(|_| "orders".to_string()),
        env::var("INVOICES_TABLE_NAME").unwrap_or_else(|_| "invoices".to_string()),
        env::var("PAYMENTS_TABLE_NAME").unwrap_or_else(|_| "payments".to_string()),
    );

    // Create the billing module
    let billing_module = BillingModule::new(stores);

    // Build the server host
    let host = Arc::new(
        ServerBuilder::new()
            .with_link_service(link_service)
            .register_module(billing_module)?
            .build_host()?,
    );

    // Build routers
    let rest_router = RestExposure::build_router(host.clone(), vec![])?;

    #[cfg(feature = "graphql")]
    let graphql_router = GraphQLExposure::build_router(host.clone())?;

    // Merge routers
    #[cfg(feature = "graphql")]
    let app = Router::new().merge(rest_router).merge(graphql_router);

    #[cfg(not(feature = "graphql"))]
    let app = Router::new().merge(rest_router);

    println!("\n🌐 Server running on http://0.0.0.0:4242");
    println!("\n📚 Endpoints disponibles:");
    println!("\n  REST API:");
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

    #[cfg(feature = "graphql")]
    {
        println!("\n  GraphQL API:");
        println!("    POST   /graphql");
        println!("    GET    /graphql/playground");
        println!("    GET    /graphql/schema");
    }

    let listener = tokio::net::TcpListener::bind("0.0.0.0:4242").await?;
    axum::serve(listener, app).await?;

    Ok(())
}
