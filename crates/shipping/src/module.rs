use std::sync::Arc;

use this::core::module::Module;
use this::prelude::LinksConfig;
use this::prelude::{EntityCreator, EntityFetcher};
use this::server::entity_registry::EntityRegistry;

use crate::entities::shipment::{InMemoryShipmentStore, ShipmentStore};

/// Combined trait for the shipment store's two faces (fetch and create).
pub trait EntityStore: EntityFetcher + EntityCreator + Send + Sync {}
impl<T> EntityStore for T where T: EntityFetcher + EntityCreator + Send + Sync {}

pub struct ShippingStores {
    pub shipments_store: Arc<dyn ShipmentStore>,
    pub shipments_entity: Arc<dyn EntityStore>,
}

impl ShippingStores {
    pub fn new_in_memory() -> Self {
        let shipments = Arc::new(InMemoryShipmentStore::default());
        Self {
            shipments_store: shipments.clone(),
            shipments_entity: shipments,
        }
    }
}

pub struct ShippingModule {
    pub stores: ShippingStores,
}

impl ShippingModule {
    pub fn new(stores: ShippingStores) -> Self {
        Self { stores }
    }
}

impl Module for ShippingModule {
    fn name(&self) -> &str {
        "shipping"
    }

    fn version(&self) -> &str {
        "0.0.1"
    }

    fn entity_types(&self) -> Vec<&str> {
        vec!["shipment"]
    }

    fn links_config(&self) -> Result<LinksConfig, anyhow::Error> {
        let config_path = concat!(env!("CARGO_MANIFEST_DIR"), "/config/links.yaml");
        LinksConfig::from_yaml_file(config_path)
    }

    fn get_entity_fetcher(&self, entity_type: &str) -> Option<Arc<dyn EntityFetcher>> {
        match entity_type {
            "shipment" => Some(self.stores.shipments_entity.clone()),
            _ => None,
        }
    }

    fn get_entity_creator(&self, entity_type: &str) -> Option<Arc<dyn EntityCreator>> {
        match entity_type {
            "shipment" => Some(self.stores.shipments_entity.clone()),
            _ => None,
        }
    }

    // Descriptor-only, as in catalog: build_host serves these from the descriptor.
    fn register_entities(&self, _registry: &mut EntityRegistry) {}

    fn descriptor(&self) -> Option<this::descriptor::ModuleDescriptor> {
        let mut d = this::descriptor::ModuleDescriptor::new(self.name(), self.version());
        d.entities
            .push(crate::entities::shipment::Shipment::descriptor().clone());
        // Cross-module dependencies, by module name: a shipment links to a product
        // (catalog) and to an order (billing). Without these the host refuses the
        // links with `this.module.reference_unknown`.
        for required in ["catalog", "billing"] {
            d.requires.push(this::descriptor::ModuleReq {
                name: required.to_string(),
                version: "^0.1".to_string(),
                optional: false,
            });
        }
        Some(d)
    }
}

#[cfg(test)]
mod descriptor_host {
    //! Standalone checks only. `build_host` needs the modules this one requires
    //! (catalog, billing): it is exercised by `multi-module`'s smoke test, which
    //! registers all three.
    use super::*;

    #[test]
    fn shipping_descriptor_declares_its_entity_and_its_two_requirements() {
        let module = ShippingModule::new(ShippingStores::new_in_memory());
        let d = module.descriptor().expect("shipping descriptor");
        let entities: Vec<String> = d.entities.iter().map(|e| e.name.clone()).collect();
        assert_eq!(entities, vec!["shipment"]);
        let requires: Vec<String> = d.requires.iter().map(|r| r.name.clone()).collect();
        assert_eq!(requires, vec!["catalog", "billing"]);
    }
}
