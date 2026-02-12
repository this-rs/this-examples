use axum::routing::get;
use std::sync::Arc;
use this::prelude::Router;
use this::server::entity_registry::EntityDescriptor;

use super::ActivityStore;
use super::handlers::{
    ActivityState, create_activity, delete_activity, get_activity, list_activities, update_activity,
};

#[derive(Clone)]
pub struct ActivityDescriptor {
    store: Arc<dyn ActivityStore + Send + Sync>,
    entity_creator: Arc<dyn this::prelude::EntityCreator + Send + Sync>,
}

impl ActivityDescriptor {
    pub fn new(_store: Arc<dyn ActivityStore + Send + Sync>) -> Self {
        unimplemented!("Need to provide both store and entity_creator")
    }

    pub fn new_with_creator(
        store: Arc<dyn ActivityStore + Send + Sync>,
        entity_creator: Arc<dyn this::prelude::EntityCreator + Send + Sync>,
    ) -> Self {
        Self {
            store,
            entity_creator,
        }
    }
}

impl EntityDescriptor for ActivityDescriptor {
    fn entity_type(&self) -> &str {
        "activity"
    }

    fn plural(&self) -> &str {
        "activities"
    }

    fn build_routes(&self) -> Router {
        let state = ActivityState {
            store: self.store.clone(),
            entity_creator: self.entity_creator.clone(),
        };
        Router::new()
            .route("/activities", get(list_activities).post(create_activity))
            .route(
                "/activities/{id}",
                get(get_activity)
                    .put(update_activity)
                    .delete(delete_activity),
            )
            .with_state(state)
    }
}
