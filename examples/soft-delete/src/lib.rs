use anyhow::Result;
use axum::Router;
use std::sync::Arc;

use billing::{BillingModule, BillingStores};
use test_data::populate_test_data;

use this::server::builder::ServerBuilder;
use this::storage::InMemoryLinkService;

/// The billing REST API, plus a second handle on the same stores so a test can look at the rows
/// underneath the API. `soft` turns on the framework's soft delete, which is opt-in on purpose.
pub async fn app(soft: bool) -> Result<(Router, BillingStores)> {
    let stores = BillingStores::new_in_memory();
    let handles = BillingStores {
        orders_store: stores.orders_store.clone(),
        orders_entity: stores.orders_entity.clone(),
        invoices_store: stores.invoices_store.clone(),
        invoices_entity: stores.invoices_entity.clone(),
        payments_store: stores.payments_store.clone(),
        payments_entity: stores.payments_entity.clone(),
    };
    let billing_module = BillingModule::new(stores);
    let link_service = Arc::new(InMemoryLinkService::new());
    populate_test_data(&billing_module.stores, link_service.clone()).await?;

    let builder = ServerBuilder::new()
        .with_link_service((*link_service).clone())
        .with_default_event_bus()
        .with_default_notification_store()
        .register_module(billing_module)?;
    let builder = if soft {
        builder.with_soft_delete()
    } else {
        builder
    };
    // Same entry point as `examples/rest`. Authentication is deliberately not part of this example:
    // `build_host` + `RestExposure` arms the default `authenticated` policy, which needs a provider.
    let router = builder.build()?;

    // Authentication is out of scope here: callers are a fixed user, as in `examples/rest`'s smoke
    // test (with the admin role, which deleting an order requires). The OIDC example is the one that verifies real tokens.
    let router = router.layer(axum::Extension(this::prelude::AuthContext::User {
        user_id: uuid::Uuid::from_u128(1),
        tenant_id: uuid::Uuid::from_u128(9),
        roles: vec!["admin".to_string()],
        scopes: Vec::new(),
    }));

    Ok((router, handles))
}

/// The same billing module, as a host: the path GraphQL, gRPC and the link routes take, where the
/// framework guards every operation itself. `soft` enables the framework's soft delete.
pub async fn host(soft: bool) -> Result<(Arc<this::server::host::ServerHost>, BillingStores)> {
    let stores = BillingStores::new_in_memory();
    let handles = BillingStores {
        orders_store: stores.orders_store.clone(),
        orders_entity: stores.orders_entity.clone(),
        invoices_store: stores.invoices_store.clone(),
        invoices_entity: stores.invoices_entity.clone(),
        payments_store: stores.payments_store.clone(),
        payments_entity: stores.payments_entity.clone(),
    };
    let billing_module = BillingModule::new(stores);
    let link_service = Arc::new(InMemoryLinkService::new());
    populate_test_data(&billing_module.stores, link_service.clone()).await?;
    let builder = ServerBuilder::new()
        .with_link_service((*link_service).clone())
        .with_default_event_bus()
        .with_default_notification_store()
        .register_module(billing_module)?;
    let builder = if soft {
        builder.with_soft_delete()
    } else {
        builder
    };
    Ok((Arc::new(builder.build_host()?), handles))
}
