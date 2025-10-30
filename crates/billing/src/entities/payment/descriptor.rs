use std::sync::Arc;
use axum::routing::{get, post, put, delete};
use this::prelude::Router;
use this::server::entity_registry::EntityDescriptor;

use super::PaymentStore;
use super::handlers::{PaymentState, list_payments, get_payment, create_payment, update_payment, delete_payment};

#[derive(Clone)]
pub struct PaymentDescriptor { store: Arc<dyn PaymentStore> }

impl PaymentDescriptor {
    pub fn new(store: Arc<dyn PaymentStore>) -> Self { Self { store } }
}

impl EntityDescriptor for PaymentDescriptor {
    fn entity_type(&self) -> &str { "payment" }
    fn plural(&self) -> &str { "payments" }
    fn build_routes(&self) -> Router {
        let state = PaymentState { store: self.store.clone() };
        Router::new()
            .route("/payments", get(list_payments).post(create_payment))
            .route("/payments/{id}", get(get_payment).put(update_payment).delete(delete_payment))
            .with_state(state)
    }
}
