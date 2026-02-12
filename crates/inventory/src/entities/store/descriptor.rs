use axum::routing::get;
use std::sync::Arc;
use this::prelude::Router;
use this::server::entity_registry::EntityDescriptor;

use super::StoreStore;
use super::handlers::{
    StoreState, create_store, delete_store, get_store, list_stores, update_store,
};

#[derive(Clone)]
pub struct StoreDescriptor {
    store: Arc<dyn StoreStore + Send + Sync>,
    entity_creator: Arc<dyn this::prelude::EntityCreator + Send + Sync>,
}

impl StoreDescriptor {
    pub fn new(_store: Arc<dyn StoreStore + Send + Sync>) -> Self {
        unimplemented!("Need to provide both store and entity_creator")
    }

    pub fn new_with_creator(
        store: Arc<dyn StoreStore + Send + Sync>,
        entity_creator: Arc<dyn this::prelude::EntityCreator + Send + Sync>,
    ) -> Self {
        Self {
            store,
            entity_creator,
        }
    }
}

impl EntityDescriptor for StoreDescriptor {
    fn entity_type(&self) -> &str {
        "store"
    }

    fn plural(&self) -> &str {
        "stores"
    }

    fn build_routes(&self) -> Router {
        let state = StoreState {
            store: self.store.clone(),
            entity_creator: self.entity_creator.clone(),
        };
        Router::new()
            .route("/stores", get(list_stores).post(create_store))
            .route(
                "/stores/{id}",
                get(get_store).put(update_store).delete(delete_store),
            )
            .with_state(state)
    }
}
