use anyhow::Result;
use aws_sdk_dynamodb::Client;
use axum::Router;
use std::sync::Arc;

use billing::{BillingModule, BillingStores};
use this::server::builder::ServerBuilder;
use this::server::RestExposure;
use this::storage::DynamoDBLinkService;

/// Build the DynamoDB example's router (billing backed by DynamoDB, local endpoint).
pub async fn app(endpoint_url: &str) -> Result<Router> {
    let config = aws_config::defaults(aws_config::BehaviorVersion::latest())
        .endpoint_url(endpoint_url)
        .region(aws_config::Region::new("us-east-1"))
        // DynamoDB Local accepts any credentials, but without some the SDK walks its
        // credential chain (IMDS...) and a scan fails as `dispatch failure`.
        .credentials_provider(aws_sdk_dynamodb::config::Credentials::new(
            "fakeMyKeyId",
            "fakeSecretAccessKey",
            None,
            None,
            "dynamodb-example",
        ))
        .load()
        .await;
    let client = Client::new(&config);

    let link_service = DynamoDBLinkService::new(client.clone(), "links".to_string());
    let stores = BillingStores::new_dynamodb(
        client,
        "orders".to_string(),
        "invoices".to_string(),
        "payments".to_string(),
    );
    let billing_module = BillingModule::new(stores);

    let host = Arc::new(
        ServerBuilder::new()
            .with_link_service(link_service)
            .register_module(billing_module)?
            .build_host()?,
    );

    Ok(Router::new().merge(RestExposure::build_router(host.clone(), vec![])?))
}

/// The billing stores on their own, for the soft-delete contract test (which needs stores, not a router).
/// Same client configuration as `app`.
pub async fn billing_stores(endpoint_url: &str) -> Result<BillingStores> {
    let config = aws_config::defaults(aws_config::BehaviorVersion::latest())
        .endpoint_url(endpoint_url)
        .region(aws_config::Region::new("us-east-1"))
        .credentials_provider(aws_sdk_dynamodb::config::Credentials::new(
            "fakeMyKeyId",
            "fakeSecretAccessKey",
            None,
            None,
            "dynamodb-example",
        ))
        .load()
        .await;
    let client = Client::new(&config);
    Ok(BillingStores::new_dynamodb(
        client,
        "orders".to_string(),
        "invoices".to_string(),
        "payments".to_string(),
    ))
}
