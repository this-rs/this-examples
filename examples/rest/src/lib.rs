use billing::{BillingModule, BillingStores};
use std::sync::Arc;
use test_data::populate_test_data;
use this::server::builder::ServerBuilder;
use this::storage::InMemoryLinkService;

/// Build the REST example's router: in-memory billing, populated with test data.
pub async fn app() -> anyhow::Result<axum::Router> {
    let link_service = Arc::new(InMemoryLinkService::new());

    // Create stores and populate with test data
    let stores = BillingStores::new_in_memory();
    populate_test_data(&stores, link_service.clone()).await?;

    let billing_module = BillingModule::new(stores);

    let app = ServerBuilder::new()
        .with_link_service((*link_service).clone())
        .register_module(billing_module)?
        .build()?;

    Ok(app)
}
