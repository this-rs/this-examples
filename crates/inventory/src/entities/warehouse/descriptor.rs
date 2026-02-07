use axum::routing::get;
use std::sync::Arc;
use this::prelude::Router;
use this::server::entity_registry::EntityDescriptor;

use super::WarehouseStore;
use super::handlers::{
    WarehouseState, create_warehouse, delete_warehouse, get_warehouse, list_warehouses,
    update_warehouse,
};

#[derive(Clone)]
pub struct WarehouseDescriptor {
    store: Arc<dyn WarehouseStore + Send + Sync>,
    entity_creator: Arc<dyn this::prelude::EntityCreator + Send + Sync>,
}

impl WarehouseDescriptor {
    pub fn new(_store: Arc<dyn WarehouseStore + Send + Sync>) -> Self {
        unimplemented!("Need to provide both store and entity_creator")
    }

    pub fn new_with_creator(
        store: Arc<dyn WarehouseStore + Send + Sync>,
        entity_creator: Arc<dyn this::prelude::EntityCreator + Send + Sync>,
    ) -> Self {
        Self {
            store,
            entity_creator,
        }
    }
}

impl EntityDescriptor for WarehouseDescriptor {
    fn entity_type(&self) -> &str {
        "warehouse"
    }

    fn plural(&self) -> &str {
        "warehouses"
    }

    fn build_routes(&self) -> Router {
        let state = WarehouseState {
            store: self.store.clone(),
            entity_creator: self.entity_creator.clone(),
        };
        Router::new()
            .route("/warehouses", get(list_warehouses).post(create_warehouse))
            .route(
                "/warehouses/{id}",
                get(get_warehouse)
                    .put(update_warehouse)
                    .delete(delete_warehouse),
            )
            .with_state(state)
    }
}





