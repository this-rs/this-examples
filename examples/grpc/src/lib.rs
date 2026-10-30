use anyhow::Result;
use axum::Router;
use std::sync::Arc;

use billing::{BillingModule, BillingStores};
use test_data::populate_test_data;

use this::config::LinksConfig;
use this::core::module::Module;
use this::core::{EntityCreator, EntityFetcher};
use this::server::builder::ServerBuilder;
use this::server::entity_registry::EntityRegistry;
use this::server::{GrpcExposure, RestExposure};
use this::storage::InMemoryLinkService;

struct EventsConfigModule;

impl Module for EventsConfigModule {
    fn name(&self) -> &str {
        "events-config"
    }
    fn version(&self) -> &str {
        "0.1.0"
    }
    fn entity_types(&self) -> Vec<&str> {
        vec![]
    }
    fn register_entities(&self, _registry: &mut EntityRegistry) {}
    fn links_config(&self) -> Result<LinksConfig> {
        let config_path = concat!(env!("CARGO_MANIFEST_DIR"), "/config/events.yaml");
        LinksConfig::from_yaml_file(config_path)
    }
    fn get_entity_fetcher(&self, _: &str) -> Option<Arc<dyn EntityFetcher>> {
        None
    }
    fn get_entity_creator(&self, _: &str) -> Option<Arc<dyn EntityCreator>> {
        None
    }
}

/// Build the gRPC example's router (REST + gRPC, in-memory billing).
pub async fn app() -> Result<Router> {
    let stores = BillingStores::new_in_memory();
    let billing_module = BillingModule::new(stores);
    let link_service_arc = Arc::new(InMemoryLinkService::new());
    populate_test_data(&billing_module.stores, link_service_arc.clone()).await?;

    let host = Arc::new(
        ServerBuilder::new()
            .with_link_service((*link_service_arc).clone())
            .with_event_bus(1024)
            .register_module(billing_module)?
            .register_module(EventsConfigModule)?
            .build_host()?,
    );

    let rest_router = RestExposure::build_router(host.clone(), vec![])?;
    let grpc_router = GrpcExposure::build_router_unenforced(host.clone())?;
    Ok(rest_router.fallback_service(grpc_router))
}
