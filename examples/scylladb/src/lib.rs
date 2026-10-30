use anyhow::Result;
use axum::Router;
use scylla::client::session::Session;
use scylla::client::session_builder::SessionBuilder;
use std::sync::Arc;

use billing::{BillingModule, BillingStores};
use this::server::builder::ServerBuilder;
use this::server::RestExposure;
use this::storage::ScyllaLinkService;

/// Build the ScyllaDB example's router (billing backed by ScyllaDB/Cassandra).
pub async fn app(scylla_uri: &str, keyspace: &str) -> Result<Router> {
    let session: Arc<Session> = Arc::new(
        SessionBuilder::new()
            .known_node(scylla_uri)
            .build()
            .await?,
    );

    session
        .query_unpaged(
            format!(
                "CREATE KEYSPACE IF NOT EXISTS {keyspace} WITH replication = {{'class': 'SimpleStrategy', 'replication_factor': 1}}"
            ),
            &[],
        )
        .await?;

    let link_service = ScyllaLinkService::new(session.clone(), keyspace.to_string());
    let stores = BillingStores::new_scylladb(session, keyspace.to_string());
    let billing_module = BillingModule::new(stores);

    let host = Arc::new(
        ServerBuilder::new()
            .with_link_service(link_service)
            .register_module(billing_module)?
            .build_host()?,
    );

    Ok(Router::new().merge(RestExposure::build_router(host.clone(), vec![])?))
}
