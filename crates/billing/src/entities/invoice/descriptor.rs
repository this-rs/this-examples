use axum::routing::get;
use std::sync::Arc;
use this::prelude::Router;
use this::server::entity_registry::EntityDescriptor;

use super::InvoiceStore;
use super::handlers::{
    InvoiceState, create_invoice, delete_invoice, get_invoice, list_invoices, update_invoice,
};

#[derive(Clone)]
pub struct InvoiceDescriptor {
    store: Arc<dyn InvoiceStore + Send + Sync>,
    entity_creator: Arc<dyn this::prelude::EntityCreator + Send + Sync>,
}

impl InvoiceDescriptor {
    pub fn new(store: Arc<dyn InvoiceStore + Send + Sync>) -> Self {
        unimplemented!("Need to provide both store and entity_creator")
    }

    pub fn new_with_creator(
        store: Arc<dyn InvoiceStore + Send + Sync>,
        entity_creator: Arc<dyn this::prelude::EntityCreator + Send + Sync>,
    ) -> Self {
        Self {
            store,
            entity_creator,
        }
    }
}

impl EntityDescriptor for InvoiceDescriptor {
    fn entity_type(&self) -> &str {
        "invoice"
    }
    fn plural(&self) -> &str {
        "invoices"
    }
    fn build_routes(&self) -> Router {
        let state = InvoiceState {
            store: self.store.clone(),
            entity_creator: self.entity_creator.clone(),
        };
        Router::new()
            .route("/invoices", get(list_invoices).post(create_invoice))
            .route(
                "/invoices/{id}",
                get(get_invoice).put(update_invoice).delete(delete_invoice),
            )
            .with_state(state)
    }
}
