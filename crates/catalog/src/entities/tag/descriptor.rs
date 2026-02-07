use axum::routing::get;
use std::sync::Arc;
use this::prelude::Router;
use this::server::entity_registry::EntityDescriptor;

use super::TagStore;
use super::handlers::{TagState, create_tag, delete_tag, get_tag, list_tags, update_tag};

#[derive(Clone)]
pub struct TagDescriptor {
    store: Arc<dyn TagStore + Send + Sync>,
    entity_creator: Arc<dyn this::prelude::EntityCreator + Send + Sync>,
}

impl TagDescriptor {
    pub fn new(_store: Arc<dyn TagStore + Send + Sync>) -> Self {
        unimplemented!("Need to provide both store and entity_creator")
    }

    pub fn new_with_creator(
        store: Arc<dyn TagStore + Send + Sync>,
        entity_creator: Arc<dyn this::prelude::EntityCreator + Send + Sync>,
    ) -> Self {
        Self {
            store,
            entity_creator,
        }
    }
}

impl EntityDescriptor for TagDescriptor {
    fn entity_type(&self) -> &str {
        "tag"
    }

    fn plural(&self) -> &str {
        "tags"
    }

    fn build_routes(&self) -> Router {
        let state = TagState {
            store: self.store.clone(),
            entity_creator: self.entity_creator.clone(),
        };
        Router::new()
            .route("/tags", get(list_tags).post(create_tag))
            .route(
                "/tags/{id}",
                get(get_tag).put(update_tag).delete(delete_tag),
            )
            .with_state(state)
    }
}





