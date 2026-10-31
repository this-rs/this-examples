use anyhow::Result;
use axum::Router;
use std::sync::Arc;

use billing::{BillingModule, BillingStores};
use this::server::builder::ServerBuilder;
use this::server::RestExposure;
use this::storage::LmdbLinkService;

/// Build the LMDB example's router (file-backed billing, no external service).
pub async fn app(data_dir: &str) -> Result<Router> {
    std::fs::create_dir_all(data_dir)?;

    let links_path = format!("{data_dir}/links");
    std::fs::create_dir_all(&links_path)?;
    let link_service = LmdbLinkService::open(&links_path)?;

    let stores_path = format!("{data_dir}/stores");
    std::fs::create_dir_all(&stores_path)?;
    let stores = BillingStores::new_lmdb(&stores_path)?;

    let billing_module = BillingModule::new(stores);

    let host = Arc::new(
        ServerBuilder::new()
            .with_link_service(link_service)
            .register_module(billing_module)?
            .build_host()?,
    );

    Ok(Router::new().merge(RestExposure::build_router(host.clone(), vec![])?))
}
