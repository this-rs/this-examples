use std::sync::Arc;
use this::core::module::Module;
use this::prelude::LinksConfig;
use this::prelude::{EntityCreator, EntityFetcher};
use this::server::entity_registry::EntityRegistry;

use crate::entities::category::CategoryStore;
use crate::entities::product::ProductStore;
use crate::entities::tag::TagStore;

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

    // Descriptor-only: build_host serves these from the descriptor (`DescriptorEntity`),
    // whose host-aware routes go through the common guard (SurfaceGuard).
    fn register_entities(&self, _registry: &mut EntityRegistry) {}

    /// Descriptor of the module, declared once (RFC « descriptor-only »). ADDITIVE.
    fn descriptor(&self) -> Option<this::descriptor::ModuleDescriptor> {
        let mut d = this::descriptor::ModuleDescriptor::new(self.name(), self.version());
        d.entities
            .push(crate::entities::product::Product::descriptor().clone());
        d.entities
            .push(crate::entities::category::Category::descriptor().clone());
        d.entities
            .push(crate::entities::tag::Tag::descriptor().clone());
        Some(d)
    }
}

#[cfg(test)]
mod descriptor_host {
    //! P4 (additive): catalog declares its descriptor once; build_host validates it.
    use super::*;
    use this::core::module::Module;
    use this::prelude::{InMemoryLinkService, ServerBuilder};

    #[test]
    fn catalog_descriptor_is_coherent_and_builds_a_host() {
        let module = CatalogModule::new(crate::CatalogStores::new_in_memory());
        let names: Vec<String> = module
            .descriptor()
            .expect("catalog descriptor")
            .entities
            .iter()
            .map(|e| e.name.clone())
            .collect();
        assert_eq!(names, vec!["product", "category", "tag"]);

        ServerBuilder::new()
            .with_link_service(InMemoryLinkService::new())
            .register_module(module)
            .expect("register")
            .build_host()
            .expect("build_host accepts the catalog descriptor");
    }
}
