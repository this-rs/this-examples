use std::sync::Arc;
use this::core::module::Module;
use this::prelude::LinksConfig;
use this::prelude::{EntityCreator, EntityFetcher};
use this::server::entity_registry::EntityRegistry;

use crate::entities::category::CategoryStore;
use crate::entities::category::descriptor::CategoryDescriptor;
use crate::entities::product::ProductStore;
use crate::entities::product::descriptor::ProductDescriptor;
use crate::entities::tag::TagStore;
use crate::entities::tag::descriptor::TagDescriptor;

// Combined trait for entity stores
pub trait EntityStore: EntityFetcher + EntityCreator + Send + Sync {}

// Blanket implementation for any type that implements both traits
impl<T> EntityStore for T where T: EntityFetcher + EntityCreator + Send + Sync {}

pub struct CatalogStores {
    pub products_store: Arc<dyn ProductStore>,
    pub products_entity: Arc<dyn EntityStore>,
    pub categories_store: Arc<dyn CategoryStore>,
    pub categories_entity: Arc<dyn EntityStore>,
    pub tags_store: Arc<dyn TagStore>,
    pub tags_entity: Arc<dyn EntityStore>,
}

pub struct CatalogModule {
    pub stores: CatalogStores,
}

impl CatalogModule {
    pub fn new(stores: CatalogStores) -> Self {
        Self { stores }
    }
}

impl Module for CatalogModule {
    fn name(&self) -> &str {
        "catalog"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    fn entity_types(&self) -> Vec<&str> {
        vec!["product", "category", "tag"]
    }

    fn links_config(&self) -> Result<LinksConfig, anyhow::Error> {
        // Load configuration from YAML file
        let config_path = concat!(env!("CARGO_MANIFEST_DIR"), "/config/links.yaml");
        LinksConfig::from_yaml_file(config_path)
    }

    fn get_entity_fetcher(&self, entity_type: &str) -> Option<Arc<dyn EntityFetcher>> {
        match entity_type {
            "product" => Some(self.stores.products_entity.clone()),
            "category" => Some(self.stores.categories_entity.clone()),
            "tag" => Some(self.stores.tags_entity.clone()),
            _ => None,
        }
    }

    fn get_entity_creator(&self, entity_type: &str) -> Option<Arc<dyn EntityCreator>> {
        match entity_type {
            "product" => Some(self.stores.products_entity.clone()),
            "category" => Some(self.stores.categories_entity.clone()),
            "tag" => Some(self.stores.tags_entity.clone()),
            _ => None,
        }
    }

    fn register_entities(&self, registry: &mut EntityRegistry) {
        registry.register(Box::new(ProductDescriptor::new_with_creator(
            self.stores.products_store.clone(),
            self.stores.products_entity.clone(),
        )));
        registry.register(Box::new(CategoryDescriptor::new_with_creator(
            self.stores.categories_store.clone(),
            self.stores.categories_entity.clone(),
        )));
        registry.register(Box::new(TagDescriptor::new_with_creator(
            self.stores.tags_store.clone(),
            self.stores.tags_entity.clone(),
        )));
    }
}
