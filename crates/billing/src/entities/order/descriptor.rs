use axum::routing::get;
use std::sync::Arc;
use this::prelude::{EntityCreator, Router};
use this::server::entity_registry::EntityDescriptor;

use super::OrderStore;
use super::handlers::{
    OrderState, create_order, delete_order, get_order, list_orders, update_order,
};

#[derive(Clone)]
pub struct OrderDescriptor {
    store: Arc<dyn OrderStore + Send + Sync>,
    entity_creator: Arc<dyn EntityCreator + Send + Sync>,
}

impl OrderDescriptor {
    pub fn new(_store: Arc<dyn OrderStore + Send + Sync>) -> Self {
        // We know our stores also implement EntityCreator, but we need to handle it carefully
        // For now, let's create a simple wrapper or assume the caller provides both
        unimplemented!("Need to provide both store and entity_creator")
    }

    pub fn new_with_creator(
        store: Arc<dyn OrderStore + Send + Sync>,
        entity_creator: Arc<dyn EntityCreator + Send + Sync>,
    ) -> Self {
        Self {
            store,
            entity_creator,
        }
    }
}

impl EntityDescriptor for OrderDescriptor {
    fn entity_type(&self) -> &str {
        "order"
    }
    fn plural(&self) -> &str {
        "orders"
    }
    fn build_routes(&self) -> Router {
        let state = OrderState {
            store: self.store.clone(),
            entity_creator: self.entity_creator.clone(),
        };
        Router::new()
            .route("/orders", get(list_orders).post(create_order))
            .route(
                "/orders/{id}",
                get(get_order).put(update_order).delete(delete_order),
            )
            .with_state(state)
    }
}
