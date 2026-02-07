use axum::routing::get;
use std::sync::Arc;
use this::prelude::Router;
use this::server::entity_registry::EntityDescriptor;

use super::StockItemStore;
use super::handlers::{
    StockItemState, create_stock_item, delete_stock_item, get_stock_item, list_stock_items,
    update_stock_item,
};

#[derive(Clone)]
pub struct StockItemDescriptor {
    store: Arc<dyn StockItemStore + Send + Sync>,
    entity_creator: Arc<dyn this::prelude::EntityCreator + Send + Sync>,
}

impl StockItemDescriptor {
    pub fn new(_store: Arc<dyn StockItemStore + Send + Sync>) -> Self {
        unimplemented!("Need to provide both store and entity_creator")
    }

    pub fn new_with_creator(
        store: Arc<dyn StockItemStore + Send + Sync>,
        entity_creator: Arc<dyn this::prelude::EntityCreator + Send + Sync>,
    ) -> Self {
        Self {
            store,
            entity_creator,
        }
    }
}

impl EntityDescriptor for StockItemDescriptor {
    fn entity_type(&self) -> &str {
        "stock_item"
    }

    fn plural(&self) -> &str {
        "stock_items"
    }

    fn build_routes(&self) -> Router {
        let state = StockItemState {
            store: self.store.clone(),
            entity_creator: self.entity_creator.clone(),
        };
        Router::new()
            .route(
                "/stock_items",
                get(list_stock_items).post(create_stock_item),
            )
            .route(
                "/stock_items/{id}",
                get(get_stock_item)
                    .put(update_stock_item)
                    .delete(delete_stock_item),
            )
            .with_state(state)
    }
}





