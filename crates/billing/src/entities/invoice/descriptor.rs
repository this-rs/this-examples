use std::sync::Arc;
use axum::routing::{get, post, put, delete};
use this::prelude::Router;
use this::server::entity_registry::EntityDescriptor;

use super::InvoiceStore;
use super::handlers::{InvoiceState, list_invoices, get_invoice, create_invoice, update_invoice, delete_invoice};

#[derive(Clone)]
pub struct InvoiceDescriptor { store: Arc<dyn InvoiceStore> }

impl InvoiceDescriptor {
    pub fn new(store: Arc<dyn InvoiceStore>) -> Self { Self { store } }
}

impl EntityDescriptor for InvoiceDescriptor {
    fn entity_type(&self) -> &str { "invoice" }
    fn plural(&self) -> &str { "invoices" }
    fn build_routes(&self) -> Router {
        let state = InvoiceState { store: self.store.clone() };
        Router::new()
            .route("/invoices", get(list_invoices).post(create_invoice))
.route("/invoices/{id}", get(get_invoice).put(update_invoice).delete(delete_invoice))
            .with_state(state)
    }
}
