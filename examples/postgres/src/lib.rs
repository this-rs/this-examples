use anyhow::Result;
use axum::Router;
use std::sync::Arc;

use billing::{BillingModule, BillingStores};
use this::server::builder::ServerBuilder;
use this::server::RestExposure;
use this::storage::PostgresLinkService;

/// Build the PostgreSQL example's router (billing backed by Postgres).
pub async fn app(database_url: &str) -> Result<Router> {
    let pool = sqlx::PgPool::connect(database_url).await?;
    let link_service = PostgresLinkService::new(pool.clone());
    let stores = BillingStores::new_postgres(pool);
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
pub async fn billing_stores(database_url: &str) -> anyhow::Result<billing::BillingStores> {
    let pool = sqlx::PgPool::connect(database_url).await?;
    Ok(billing::BillingStores::new_postgres(pool))
}
