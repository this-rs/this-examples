use anyhow::Result;
use axum::Router;
use std::sync::Arc;

use billing::{BillingModule, BillingStores};
use test_data::populate_test_data;

use this::server::builder::ServerBuilder;
use this::server::exposure::rest::RestExposure;
use this::storage::InMemoryLinkService;

/// Build the auth-STS example's router (WAMI auth + REST billing).
pub async fn app() -> Result<Router> {
    let stores = BillingStores::new_in_memory();
    let billing_module = BillingModule::new(stores);
    let link_service = Arc::new(InMemoryLinkService::new());
    populate_test_data(&billing_module.stores, link_service.clone()).await?;

    let host = Arc::new(
        ServerBuilder::new()
            .with_link_service((*link_service).clone())
            .with_default_event_bus()
            .with_default_notification_store()
            .with_auth_config_file(concat!(env!("CARGO_MANIFEST_DIR"), "/config/auth.yaml"))?
            .with_auth_resolver("is_manager", |ctx, _resource_id| ctx.has_role("manager"))
            .with_auth_resolver("is_delegated", |ctx, _resource_id| {
                ctx.has_role("delegate") || ctx.is_admin()
            })
            .register_module(billing_module)?
            .build_host()?,
    );

    RestExposure::build_router(host, vec![])
}
