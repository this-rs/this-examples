use std::sync::Arc;
use this::core::module::Module;
use this::prelude::LinksConfig;
use this::prelude::{EntityCreator, EntityFetcher};
use this::server::entity_registry::EntityRegistry;

use crate::entities::invoice::InvoiceStore;
use crate::entities::invoice::descriptor::InvoiceDescriptor;
use crate::entities::order::OrderStore;
use crate::entities::order::descriptor::OrderDescriptor;
use crate::entities::payment::PaymentStore;
use crate::entities::payment::descriptor::PaymentDescriptor;

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

    fn register_entities(&self, registry: &mut EntityRegistry) {
        registry.register(Box::new(OrderDescriptor::new_with_creator(
            self.stores.orders_store.clone(),
            self.stores.orders_entity.clone(),
        )));
        registry.register(Box::new(InvoiceDescriptor::new_with_creator(
            self.stores.invoices_store.clone(),
            self.stores.invoices_entity.clone(),
        )));
        registry.register(Box::new(PaymentDescriptor::new_with_creator(
            self.stores.payments_store.clone(),
            self.stores.payments_entity.clone(),
        )));
    }
}
