use std::sync::Arc;
use this::server::entity_registry::EntityRegistry;
use this::core::module::Module;
use this::prelude::LinksConfig;
use this::prelude::{EntityFetcher, EntityCreator};

use crate::entities::order::descriptor::OrderDescriptor;
use crate::entities::invoice::descriptor::InvoiceDescriptor;
use crate::entities::payment::descriptor::PaymentDescriptor;
use crate::stores::{InMemoryOrderStore, InMemoryInvoiceStore, InMemoryPaymentStore};

pub struct BillingStores {
    pub orders: Arc<InMemoryOrderStore>,
    pub invoices: Arc<InMemoryInvoiceStore>,
    pub payments: Arc<InMemoryPaymentStore>,
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
    fn name(&self) -> &str { "billing" }
    fn version(&self) -> &str { "0.1.0" }

    fn entity_types(&self) -> Vec<&str> {
        vec!["order", "invoice", "payment"]
    }

    fn links_config(&self) -> Result<LinksConfig, anyhow::Error> {
        // Return empty configuration for now
        Ok(LinksConfig { links: vec![], entities: vec![], validation_rules: None })
    }

    fn get_entity_fetcher(&self, entity_type: &str) -> Option<Arc<dyn EntityFetcher>> {
        match entity_type {
            "order" => Some(self.stores.orders.clone()),
            "invoice" => Some(self.stores.invoices.clone()),
            "payment" => Some(self.stores.payments.clone()),
            _ => None,
        }
    }

    fn get_entity_creator(&self, entity_type: &str) -> Option<Arc<dyn EntityCreator>> {
        match entity_type {
            "order" => Some(self.stores.orders.clone()),
            "invoice" => Some(self.stores.invoices.clone()),
            "payment" => Some(self.stores.payments.clone()),
            _ => None,
        }
    }

    fn register_entities(&self, registry: &mut EntityRegistry) {
        registry.register(Box::new(
            OrderDescriptor::new(self.stores.orders.clone())
        ));
        registry.register(Box::new(
            InvoiceDescriptor::new(self.stores.invoices.clone())
        ));
        registry.register(Box::new(
            PaymentDescriptor::new(self.stores.payments.clone())
        ));
    }
}
