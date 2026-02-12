use axum::routing::get;
use std::sync::Arc;
use this::prelude::Router;
use this::server::entity_registry::EntityDescriptor;

use super::StockMovementStore;
use super::handlers::{
    StockMovementState, create_stock_movement, delete_stock_movement, get_stock_movement,
    list_stock_movements, update_stock_movement,
};

#[derive(Clone)]
pub struct StockMovementDescriptor {
    store: Arc<dyn StockMovementStore + Send + Sync>,
    entity_creator: Arc<dyn this::prelude::EntityCreator + Send + Sync>,
}

impl StockMovementDescriptor {
    pub fn new(_store: Arc<dyn StockMovementStore + Send + Sync>) -> Self {
        unimplemented!("Need to provide both store and entity_creator")
    }

    pub fn new_with_creator(
        store: Arc<dyn StockMovementStore + Send + Sync>,
        entity_creator: Arc<dyn this::prelude::EntityCreator + Send + Sync>,
    ) -> Self {
        Self {
            store,
            entity_creator,
        }
    }
}

impl EntityDescriptor for StockMovementDescriptor {
    fn entity_type(&self) -> &str {
        "stock_movement"
    }

    fn plural(&self) -> &str {
        "stock_movements"
    }

    fn build_routes(&self) -> Router {
        let state = StockMovementState {
            store: self.store.clone(),
            entity_creator: self.entity_creator.clone(),
        };
        Router::new()
            .route(
                "/stock_movements",
                get(list_stock_movements).post(create_stock_movement),
            )
            .route(
                "/stock_movements/{id}",
                get(get_stock_movement)
                    .put(update_stock_movement)
                    .delete(delete_stock_movement),
            )
            .with_state(state)
    }
}
