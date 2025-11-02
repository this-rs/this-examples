use axum::routing::get;
use std::sync::Arc;
use this::prelude::Router;
use this::server::entity_registry::EntityDescriptor;

use super::PaymentStore;
use super::handlers::{
    PaymentState, create_payment, delete_payment, get_payment, list_payments, update_payment,
};

#[derive(Clone)]
pub struct PaymentDescriptor {
    store: Arc<dyn PaymentStore + Send + Sync>,
    entity_creator: Arc<dyn this::prelude::EntityCreator + Send + Sync>,
}

impl PaymentDescriptor {
    pub fn new(store: Arc<dyn PaymentStore + Send + Sync>) -> Self {
        unimplemented!("Need to provide both store and entity_creator")
    }
    
    pub fn new_with_creator(
        store: Arc<dyn PaymentStore + Send + Sync>,
        entity_creator: Arc<dyn this::prelude::EntityCreator + Send + Sync>,
    ) -> Self {
        Self { store, entity_creator }
    }
}

impl EntityDescriptor for PaymentDescriptor {
    fn entity_type(&self) -> &str {
        "payment"
    }
    fn plural(&self) -> &str {
        "payments"
    }
    fn build_routes(&self) -> Router {
        let state = PaymentState {
            store: self.store.clone(),
            entity_creator: self.entity_creator.clone(),
        };
        Router::new()
            .route("/payments", get(list_payments).post(create_payment))
            .route(
                "/payments/{id}",
                get(get_payment).put(update_payment).delete(delete_payment),
            )
            .with_state(state)
    }
}
