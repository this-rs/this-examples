use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use axum::Router;
use billing::{BillingModule, BillingStores};
use hyper::body::Incoming;
use hyper::{Request, Response};
use hyper_util::client::legacy::{connect::HttpConnector, Client};
use http_body_util::{BodyExt, Full};
use hyper::body::Bytes;
use test_data::populate_test_data;
use this::server::builder::ServerBuilder;
use this::server::{GraphQLExposure, RestExposure};
use this::storage::{InMemoryLinkService, DynamoDBLinkService};
use tokio::net::TcpListener;
use aws_sdk_dynamodb::Client as DynamoClient;
use aws_config::BehaviorVersion;
pub type HttpClient = Client<HttpConnector, Full<Bytes>>;

/// Test server configuration
#[derive(Clone)]
pub struct TestServer {
    pub base_url: String,
    pub client: HttpClient,
}

impl TestServer {
    pub fn new(port: u16) -> Self {
        let base_url = format!("http://127.0.0.1:{}", port);
        let client = Client::builder(hyper_util::rt::TokioExecutor::new())
            .pool_idle_timeout(Duration::from_secs(30))
            .build_http();
        
        Self { base_url, client }
    }

    pub async fn get(&self, path: &str) -> Result<Response<Incoming>> {
        let uri = format!("{}{}", self.base_url, path);
        let req = Request::builder()
            .method("GET")
            .uri(uri)
            .header("content-type", "application/json")
            .body(Full::new(Bytes::new()))?;
        
        Ok(self.client.request(req).await?)
    }

    pub async fn post_json(&self, path: &str, body: serde_json::Value) -> Result<Response<Incoming>> {
        let uri = format!("{}{}", self.base_url, path);
        let body_bytes = serde_json::to_vec(&body)?;
        
        let req = Request::builder()
            .method("POST")
            .uri(uri)
            .header("content-type", "application/json")
            .body(Full::new(Bytes::from(body_bytes)))?;
        
        Ok(self.client.request(req).await?)
    }

    pub async fn response_body_string(response: Response<Incoming>) -> Result<String> {
        let body_bytes = response.collect().await?.to_bytes();
        Ok(String::from_utf8(body_bytes.to_vec())?)
    }
}

/// Create a REST server with in-memory storage
pub async fn create_rest_server_in_memory() -> Result<Router> {
    let link_service = Arc::new(InMemoryLinkService::new());
    let stores = BillingStores::new_in_memory();
    
    // Populate test data
    populate_test_data(&stores, link_service.clone()).await?;
    
    let billing_module = BillingModule::new(stores);
    let host = Arc::new(
        ServerBuilder::new()
            .with_link_service((*link_service).clone())
            .register_module(billing_module)?
            .build_host()?,
    );
    
    let router = RestExposure::build_router(host, vec![])?;
    Ok(router)
}

/// Create a GraphQL server with in-memory storage
pub async fn create_graphql_server_in_memory() -> Result<Router> {
    let link_service = Arc::new(InMemoryLinkService::new());
    let stores = BillingStores::new_in_memory();
    
    // Populate test data
    populate_test_data(&stores, link_service.clone()).await?;
    
    let billing_module = BillingModule::new(stores);
    let host = Arc::new(
        ServerBuilder::new()
            .with_link_service((*link_service).clone())
            .register_module(billing_module)?
            .build_host()?,
    );
    
    let rest_router = RestExposure::build_router(host.clone(), vec![])?;
    let graphql_router = GraphQLExposure::build_router(host)?;
    let router = Router::new().merge(rest_router).merge(graphql_router);
    
    Ok(router)
}

/// Create a REST server with DynamoDB storage
pub async fn create_rest_server_dynamodb() -> Result<Router> {
    // Configure DynamoDB client for local testing
    let mut config_loader = aws_config::defaults(BehaviorVersion::latest());
    
    // Use local DynamoDB if available
    if let Ok(endpoint_url) = std::env::var("AWS_ENDPOINT_URL") {
        config_loader = config_loader.endpoint_url(&endpoint_url);
    } else {
        // Default to local DynamoDB for benchmarks
        config_loader = config_loader.endpoint_url("http://localhost:8000");
    }
    
    let config = config_loader.load().await;
    let client = DynamoClient::new(&config);
    
    // Create DynamoDB link service
    let link_service = DynamoDBLinkService::new(
        client.clone(),
        "bench_links".to_string(),
    );
    
    // Create billing stores with DynamoDB
    let stores = BillingStores::new_dynamodb(
        client,
        "bench_orders".to_string(),
        "bench_invoices".to_string(),
        "bench_payments".to_string(),
    );
    
    let billing_module = BillingModule::new(stores);
    let host = Arc::new(
        ServerBuilder::new()
            .with_link_service(link_service)
            .register_module(billing_module)?
            .build_host()?,
    );
    
    let router = RestExposure::build_router(host, vec![])?;
    Ok(router)
}

/// Create a GraphQL server with DynamoDB storage
pub async fn create_graphql_server_dynamodb() -> Result<Router> {
    // Configure DynamoDB client for local testing
    let mut config_loader = aws_config::defaults(BehaviorVersion::latest());
    
    // Use local DynamoDB if available
    if let Ok(endpoint_url) = std::env::var("AWS_ENDPOINT_URL") {
        config_loader = config_loader.endpoint_url(&endpoint_url);
    } else {
        // Default to local DynamoDB for benchmarks
        config_loader = config_loader.endpoint_url("http://localhost:8000");
    }
    
    let config = config_loader.load().await;
    let client = DynamoClient::new(&config);
    
    // Create DynamoDB link service
    let link_service = DynamoDBLinkService::new(
        client.clone(),
        "bench_links".to_string(),
    );
    
    // Create billing stores with DynamoDB
    let stores = BillingStores::new_dynamodb(
        client,
        "bench_orders".to_string(),
        "bench_invoices".to_string(),
        "bench_payments".to_string(),
    );
    
    let billing_module = BillingModule::new(stores);
    let host = Arc::new(
        ServerBuilder::new()
            .with_link_service(link_service)
            .register_module(billing_module)?
            .build_host()?,
    );
    
    let rest_router = RestExposure::build_router(host.clone(), vec![])?;
    let graphql_router = GraphQLExposure::build_router(host)?;
    let router = Router::new().merge(rest_router).merge(graphql_router);
    
    Ok(router)
}

/// Start a test server on a dynamic port
pub async fn start_test_server(router: Router) -> Result<TestServer> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    let test_server = TestServer::new(addr.port());
    
    tokio::spawn(async move {
        let app = router.into_make_service();
        axum::serve(listener, app).await.unwrap();
    });
    
    // Give the server a moment to start
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    Ok(test_server)
}

/// Sample data generators for benchmarks
pub mod data {
    use serde_json::{json, Value};
    use uuid::Uuid;

    pub fn sample_order() -> Value {
        json!({
            "name": "Sample Order",
            "status": "pending",
            "number": format!("ORD-{}", Uuid::new_v4().to_string().split('-').next().unwrap().to_uppercase()),
            "amount": 1299.99,
            "customer_name": "John Doe",
            "notes": "Sample order for benchmarking"
        })
    }

    pub fn sample_invoice() -> Value {
        json!({
            "name": "Sample Invoice",
            "status": "draft",
            "number": format!("INV-{}", Uuid::new_v4().to_string().split('-').next().unwrap().to_uppercase()),
            "amount": 2599.50,
            "due_date": "2024-12-31",
            "paid_at": null
        })
    }

    pub fn sample_payment() -> Value {
        json!({
            "name": "Sample Payment",
            "status": "pending", 
            "number": format!("PAY-{}", Uuid::new_v4().to_string().split('-').next().unwrap().to_uppercase()),
            "amount": 1299.99,
            "method": "credit_card",
            "transaction_id": format!("txn_{}", Uuid::new_v4().to_string().replace('-', "")[..16].to_string())
        })
    }

    pub fn sample_graphql_query() -> Value {
        json!({
            "query": "query { orders { id name number amount status } }"
        })
    }

    pub fn sample_graphql_mutation() -> Value {
        let order = sample_order();
        json!({
            "query": "mutation CreateOrder($input: OrderInput!) { createOrder(input: $input) { id name number amount status } }",
            "variables": {
                "input": {
                    "name": order["name"],
                    "status": order["status"],
                    "number": order["number"],
                    "amount": order["amount"],
                    "customer_name": order["customer_name"],
                    "notes": order["notes"]
                }
            }
        })
    }
}