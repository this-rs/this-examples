use axum::routing::get;
use std::sync::Arc;
use this::prelude::Router;
use this::server::entity_registry::EntityDescriptor;

use super::OrderStore;
use super::handlers::{
    OrderState, create_order, delete_order, get_order, list_orders, update_order,
};

#[derive(Clone)]
pub struct OrderDescriptor {
    store: Arc<dyn OrderStore>,
}

impl OrderDescriptor {
    pub fn new(store: Arc<dyn OrderStore>) -> Self {
        Self { store }
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
