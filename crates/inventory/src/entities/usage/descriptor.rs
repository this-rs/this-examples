use axum::routing::get;
use std::sync::Arc;
use this::prelude::Router;
use this::server::entity_registry::EntityDescriptor;

use super::UsageStore;
use super::handlers::{
    UsageState, create_usage, delete_usage, get_usage, list_usages, update_usage,
};

#[derive(Clone)]
pub struct UsageDescriptor {
    store: Arc<dyn UsageStore + Send + Sync>,
    entity_creator: Arc<dyn this::prelude::EntityCreator + Send + Sync>,
}

impl UsageDescriptor {
    pub fn new(_store: Arc<dyn UsageStore + Send + Sync>) -> Self {
        unimplemented!("Need to provide both store and entity_creator")
    }

    pub fn new_with_creator(
        store: Arc<dyn UsageStore + Send + Sync>,
        entity_creator: Arc<dyn this::prelude::EntityCreator + Send + Sync>,
    ) -> Self {
        Self {
            store,
            entity_creator,
        }
    }
}

impl EntityDescriptor for UsageDescriptor {
    fn entity_type(&self) -> &str {
        "usage"
    }

    fn plural(&self) -> &str {
        "usages"
    }

    fn build_routes(&self) -> Router {
        let state = UsageState {
            store: self.store.clone(),
            entity_creator: self.entity_creator.clone(),
        };
        Router::new()
            .route("/usages", get(list_usages).post(create_usage))
            .route(
                "/usages/{id}",
                get(get_usage).put(update_usage).delete(delete_usage),
            )
            .with_state(state)
    }
}





