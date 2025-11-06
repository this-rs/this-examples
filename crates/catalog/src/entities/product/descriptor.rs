use axum::routing::get;
use std::sync::Arc;
use this::prelude::Router;
use this::server::entity_registry::EntityDescriptor;

use super::ProductStore;
use super::handlers::{
    ProductState, create_product, delete_product, get_product, list_products, update_product,
};

#[derive(Clone)]
pub struct ProductDescriptor {
    store: Arc<dyn ProductStore + Send + Sync>,
    entity_creator: Arc<dyn this::prelude::EntityCreator + Send + Sync>,
}

impl ProductDescriptor {
    pub fn new(_store: Arc<dyn ProductStore + Send + Sync>) -> Self {
        unimplemented!("Need to provide both store and entity_creator")
    }

    pub fn new_with_creator(
        store: Arc<dyn ProductStore + Send + Sync>,
        entity_creator: Arc<dyn this::prelude::EntityCreator + Send + Sync>,
    ) -> Self {
        Self {
            store,
            entity_creator,
        }
    }
}

impl EntityDescriptor for ProductDescriptor {
    fn entity_type(&self) -> &str {
        "product"
    }
    
    fn plural(&self) -> &str {
        "products"
    }
    
    fn build_routes(&self) -> Router {
        let state = ProductState {
            store: self.store.clone(),
            entity_creator: self.entity_creator.clone(),
        };
        Router::new()
            .route("/products", get(list_products).post(create_product))
            .route(
                "/products/{id}",
                get(get_product).put(update_product).delete(delete_product),
            )
            .with_state(state)
    }
}

