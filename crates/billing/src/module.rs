use std::sync::Arc;
use this::core::module::Module;
use this::prelude::LinksConfig;
use this::prelude::{EntityCreator, EntityFetcher};
use this::server::entity_registry::EntityRegistry;

use crate::entities::invoice::InvoiceStore;
use crate::entities::order::OrderStore;
use crate::entities::payment::PaymentStore;

// Combined trait for entity stores
pub trait EntityStore: EntityFetcher + EntityCreator + Send + Sync {}

// Blanket implementation for any type that implements both traits
impl<T> EntityStore for T where T: EntityFetcher + EntityCreator + Send + Sync {}

pub struct BillingStores {
    pub orders_store: Arc<dyn OrderStore>,
    pub orders_entity: Arc<dyn EntityStore>,
    pub invoices_store: Arc<dyn InvoiceStore>,
    pub invoices_entity: Arc<dyn EntityStore>,
    pub payments_store: Arc<dyn PaymentStore>,
    pub payments_entity: Arc<dyn EntityStore>,
}

pub struct BillingModule {
    pub stores: BillingStores,
}

impl BillingModule {
    pub fn new(stores: BillingStores) -> Self {
        Self { stores }
    }
}

impl Module for BillingModule {
    fn name(&self) -> &str {
        "billing"
    }
    fn version(&self) -> &str {
        "0.1.0"
    }

    fn entity_types(&self) -> Vec<&str> {
        vec!["order", "invoice", "payment"]
    }

    fn links_config(&self) -> Result<LinksConfig, anyhow::Error> {
        // Load configuration from YAML file
        let config_path = concat!(env!("CARGO_MANIFEST_DIR"), "/config/links.yaml");
        LinksConfig::from_yaml_file(config_path)
    }

    fn get_entity_fetcher(&self, entity_type: &str) -> Option<Arc<dyn EntityFetcher>> {
        match entity_type {
            "order" => Some(self.stores.orders_entity.clone()),
            "invoice" => Some(self.stores.invoices_entity.clone()),
            "payment" => Some(self.stores.payments_entity.clone()),
            _ => None,
        }
    }

    fn get_entity_creator(&self, entity_type: &str) -> Option<Arc<dyn EntityCreator>> {
        match entity_type {
            "order" => Some(self.stores.orders_entity.clone()),
            "invoice" => Some(self.stores.invoices_entity.clone()),
            "payment" => Some(self.stores.payments_entity.clone()),
            _ => None,
        }
    }

    // Descriptor-only: build_host serves these from the descriptor (`DescriptorEntity`),
    // whose host-aware routes go through the common guard (SurfaceGuard).
    fn register_entities(&self, _registry: &mut EntityRegistry) {}

    /// Descriptor of the module, declared once (RFC « descriptor-only »). ADDITIVE: the
    /// hand-written `register_entities` and `links_config` stay, so the routes are unchanged;
    /// the descriptor feeds schemas, contracts, policies. Remove `register_entities` to let
    /// `build_host` serve the entities from the descriptor (`DescriptorEntity`).
    fn descriptor(&self) -> Option<this::descriptor::ModuleDescriptor> {
        let mut d = this::descriptor::ModuleDescriptor::new(self.name(), self.version());
        d.entities
            .push(crate::entities::order::Order::descriptor().clone());
        d.entities
            .push(crate::entities::invoice::Invoice::descriptor().clone());
        d.entities
            .push(crate::entities::payment::Payment::descriptor().clone());
        Some(d)
    }
}

#[cfg(test)]
mod descriptor_host {
    //! P4 of the descriptor-only RFC, on billing: adding `descriptor()` is ADDITIVE. The
    //! descriptor is coherent (identity, plural, check) and `build_host` accepts it, while
    //! the hand-written routes stay (legacy) because `register_entities` still yields entities.
    use super::*;
    use this::core::module::Module;
    use this::prelude::{InMemoryLinkService, ServerBuilder};

    #[test]
    fn billing_descriptor_is_coherent_and_builds_a_host() {
        let module = BillingModule::new(crate::BillingStores::new_in_memory());

        let names: Vec<String> = module
            .descriptor()
            .expect("billing declares a descriptor")
            .entities
            .iter()
            .map(|e| e.name.clone())
            .collect();
        assert_eq!(names, vec!["order", "invoice", "payment"]);

        // The real proof: build_host runs identity/plural/check on the descriptor.
        ServerBuilder::new()
            .with_link_service(InMemoryLinkService::new())
            .register_module(module)
            .expect("register")
            .build_host()
            .expect("build_host accepts the billing descriptor");
    }
}
