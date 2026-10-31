use anyhow::Result;
use axum::Router;
use std::sync::Arc;
use tower_http::services::ServeDir;

use billing::{BillingModule, BillingStores};
use catalog::{CatalogModule, CatalogStores};
use inventory::{InventoryModule, InventoryStores};
use test_data::{populate_catalog_data, populate_inventory_data, populate_test_data};

use this::config::LinksConfig;
use this::core::module::Module;
use this::core::{EntityCreator, EntityFetcher};
use this::server::builder::ServerBuilder;
use this::server::entity_registry::EntityRegistry;
use this::server::{GraphQLExposure, GrpcExposure, RestExposure, WebSocketExposure};
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

/// Build the multi-module example's router (billing + catalog + inventory, all surfaces).
pub async fn app() -> Result<Router> {
    let link_service = Arc::new(InMemoryLinkService::new());

    let billing_stores = BillingStores::new_in_memory();
    populate_test_data(&billing_stores, link_service.clone()).await?;
    let billing_module = BillingModule::new(billing_stores);

    let catalog_stores = CatalogStores::new_in_memory();
    populate_catalog_data(&catalog_stores, link_service.clone()).await?;
    let catalog_module = CatalogModule::new(catalog_stores);

    let inventory_stores = InventoryStores::new_in_memory();
    populate_inventory_data(&inventory_stores, link_service.clone()).await?;
    let inventory_module = InventoryModule::new(inventory_stores);

    let host = Arc::new(
        ServerBuilder::new()
            .with_link_service((*link_service).clone())
            .with_event_bus(1024)
            .register_module(billing_module)?
            .register_module(catalog_module)?
            .register_module(inventory_module)?
            .register_module(EventsConfigModule)?
            .build_host()?,
    );

    let rest_router = RestExposure::build_router(host.clone(), vec![])?;
    let graphql_router = GraphQLExposure::build_router_unenforced(host.clone())?;
    let grpc_router = GrpcExposure::build_router_unenforced(host.clone())?;
    let ws_router = WebSocketExposure::build_router_unenforced(host.clone())?;
    let static_files = ServeDir::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../websocket/static"));

    Ok(Router::new()
        .merge(rest_router)
        .merge(graphql_router)
        .merge(ws_router)
        .nest_service("/static", static_files)
        .fallback_service(grpc_router))
}
