use anyhow::Result;
use axum::Router;
use std::sync::Arc;

use billing::{BillingModule, BillingStores};
use test_data::populate_test_data;

use this::config::auth::{AuthConfig, AuthProviderType, OidcConfig};
use this::server::builder::ServerBuilder;
use this::server::exposure::rest::RestExposure;
use this::storage::InMemoryLinkService;

pub mod idp;

/// The audience this resource server accepts. A token minted for another API is refused here.
pub const AUDIENCE: &str = "orders-api";

/// The auth section, built in code because the IdP's address is only known once it is serving.
///
/// `audience` is required: without it `this` refuses to accept any token, by design.
pub fn auth_config(discovery_url: &str) -> AuthConfig {
    AuthConfig {
        provider: AuthProviderType::Oidc,
        default_policy: "authenticated".to_string(),
        oidc: Some(OidcConfig {
            discovery_url: discovery_url.to_string(),
            audience: Some(AUDIENCE.to_string()),
            user_claim: "sub".to_string(),
            roles_claim: "roles".to_string(),
        }),
        ..AuthConfig::default()
    }
}

/// The OIDC example's router: REST billing, every route armed, tokens verified against the IdP
/// found at `discovery_url`. This crate never issues a token: it is a resource server.
pub async fn app(discovery_url: &str) -> Result<Router> {
    let stores = BillingStores::new_in_memory();
    let billing_module = BillingModule::new(stores);
    let link_service = Arc::new(InMemoryLinkService::new());
    populate_test_data(&billing_module.stores, link_service.clone()).await?;

    let host = Arc::new(
        ServerBuilder::new()
            .with_link_service((*link_service).clone())
            .with_default_event_bus()
            .with_default_notification_store()
            .with_auth_config(auth_config(discovery_url))
            .register_module(billing_module)?
            .build_host()?,
    );

    RestExposure::build_router(host, vec![])
}
