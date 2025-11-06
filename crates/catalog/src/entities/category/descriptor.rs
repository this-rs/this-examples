use axum::routing::get;
use std::sync::Arc;
use this::prelude::Router;
use this::server::entity_registry::EntityDescriptor;

use super::CategoryStore;
use super::handlers::{
    CategoryState, create_category, delete_category, get_category, list_categories, update_category,
};

#[derive(Clone)]
pub struct CategoryDescriptor {
    store: Arc<dyn CategoryStore + Send + Sync>,
    entity_creator: Arc<dyn this::prelude::EntityCreator + Send + Sync>,
}

impl CategoryDescriptor {
    pub fn new(_store: Arc<dyn CategoryStore + Send + Sync>) -> Self {
        unimplemented!("Need to provide both store and entity_creator")
    }

    pub fn new_with_creator(
        store: Arc<dyn CategoryStore + Send + Sync>,
        entity_creator: Arc<dyn this::prelude::EntityCreator + Send + Sync>,
    ) -> Self {
        Self {
            store,
            entity_creator,
        }
    }
}

impl EntityDescriptor for CategoryDescriptor {
    fn entity_type(&self) -> &str {
        "category"
    }
    
    fn plural(&self) -> &str {
        "categories"
    }
    
    fn build_routes(&self) -> Router {
        let state = CategoryState {
            store: self.store.clone(),
            entity_creator: self.entity_creator.clone(),
        };
        Router::new()
            .route("/categories", get(list_categories).post(create_category))
            .route(
                "/categories/{id}",
                get(get_category).put(update_category).delete(delete_category),
            )
            .with_state(state)
    }
}

