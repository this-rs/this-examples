use anyhow::Result;
use axum::Router;
use std::sync::Arc;

use billing::{BillingModule, BillingStores};
use this::server::builder::ServerBuilder;
use this::server::RestExposure;
use this::storage::Neo4jLinkService;

/// Build the Neo4j example's router (billing backed by Neo4j).
pub async fn app(uri: &str, user: &str, password: &str) -> Result<Router> {
    let graph = neo4rs::Graph::new(uri, user, password).await?;
    let link_service = Neo4jLinkService::new(graph.clone());
    let stores = BillingStores::new_neo4j(graph);
    let billing_module = BillingModule::new(stores);

    let host = Arc::new(
        ServerBuilder::new()
            .with_link_service(link_service)
            .register_module(billing_module)?
            .build_host()?,
    );

    Ok(Router::new().merge(RestExposure::build_router(host.clone(), vec![])?))
}
