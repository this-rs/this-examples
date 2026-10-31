use anyhow::Result;
use axum::Router;
use std::sync::Arc;

use billing::{BillingModule, BillingStores};
use this::server::builder::ServerBuilder;
use this::server::RestExposure;
use this::storage::MongoLinkService;

/// Build the MongoDB example's router (billing backed by MongoDB).
pub async fn app(mongodb_uri: &str, database_name: &str) -> Result<Router> {
    let client = mongodb::Client::with_uri_str(mongodb_uri).await?;
    let database = client.database(database_name);
    let link_service = MongoLinkService::new(database.clone());
    let stores = BillingStores::new_mongodb(database);
    let billing_module = BillingModule::new(stores);

    let host = Arc::new(
        ServerBuilder::new()
            .with_link_service(link_service)
            .register_module(billing_module)?
            .build_host()?,
    );

    Ok(Router::new().merge(RestExposure::build_router(host.clone(), vec![])?))
}
