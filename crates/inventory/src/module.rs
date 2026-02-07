use std::sync::Arc;
use this::core::module::Module;
use this::prelude::LinksConfig;
use this::prelude::{EntityCreator, EntityFetcher};
use this::server::entity_registry::EntityRegistry;

use crate::entities::activity::ActivityStore;
use crate::entities::activity::descriptor::ActivityDescriptor;
use crate::entities::stock_item::StockItemStore;
use crate::entities::stock_item::descriptor::StockItemDescriptor;
use crate::entities::stock_movement::StockMovementStore;
use crate::entities::stock_movement::descriptor::StockMovementDescriptor;
use crate::entities::store::StoreStore;
use crate::entities::store::descriptor::StoreDescriptor;
use crate::entities::usage::UsageStore;
use crate::entities::usage::descriptor::UsageDescriptor;
use crate::entities::warehouse::WarehouseStore;
use crate::entities::warehouse::descriptor::WarehouseDescriptor;

// Combined trait for entity stores
pub trait EntityStore: EntityFetcher + EntityCreator + Send + Sync {}

// Blanket implementation for any type that implements both traits
impl<T> EntityStore for T where T: EntityFetcher + EntityCreator + Send + Sync {}

pub struct InventoryStores {
    pub stores_store: Arc<dyn StoreStore>,
    pub stores_entity: Arc<dyn EntityStore>,
    pub activities_store: Arc<dyn ActivityStore>,
    pub activities_entity: Arc<dyn EntityStore>,
    pub warehouses_store: Arc<dyn WarehouseStore>,
    pub warehouses_entity: Arc<dyn EntityStore>,
    pub stock_items_store: Arc<dyn StockItemStore>,
    pub stock_items_entity: Arc<dyn EntityStore>,
    pub stock_movements_store: Arc<dyn StockMovementStore>,
    pub stock_movements_entity: Arc<dyn EntityStore>,
    pub usages_store: Arc<dyn UsageStore>,
    pub usages_entity: Arc<dyn EntityStore>,
}

pub struct InventoryModule {
    pub stores: InventoryStores,
}

impl InventoryModule {
    pub fn new(stores: InventoryStores) -> Self {
        Self { stores }
    }
}

impl Module for InventoryModule {
    fn name(&self) -> &str {
        "inventory"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    fn entity_types(&self) -> Vec<&str> {
        vec![
            "store",
            "activity",
            "warehouse",
            "stock_item",
            "stock_movement",
            "usage",
        ]
    }

    fn links_config(&self) -> Result<LinksConfig, anyhow::Error> {
        // Load configuration from YAML file
        let config_path = concat!(env!("CARGO_MANIFEST_DIR"), "/config/links.yaml");
        LinksConfig::from_yaml_file(config_path)
    }

    fn get_entity_fetcher(&self, entity_type: &str) -> Option<Arc<dyn EntityFetcher>> {
        match entity_type {
            "store" => Some(self.stores.stores_entity.clone()),
            "activity" => Some(self.stores.activities_entity.clone()),
            "warehouse" => Some(self.stores.warehouses_entity.clone()),
            "stock_item" => Some(self.stores.stock_items_entity.clone()),
            "stock_movement" => Some(self.stores.stock_movements_entity.clone()),
            "usage" => Some(self.stores.usages_entity.clone()),
            _ => None,
        }
    }

    fn get_entity_creator(&self, entity_type: &str) -> Option<Arc<dyn EntityCreator>> {
        match entity_type {
            "store" => Some(self.stores.stores_entity.clone()),
            "activity" => Some(self.stores.activities_entity.clone()),
            "warehouse" => Some(self.stores.warehouses_entity.clone()),
            "stock_item" => Some(self.stores.stock_items_entity.clone()),
            "stock_movement" => Some(self.stores.stock_movements_entity.clone()),
            "usage" => Some(self.stores.usages_entity.clone()),
            _ => None,
        }
    }

    fn register_entities(&self, registry: &mut EntityRegistry) {
        registry.register(Box::new(StoreDescriptor::new_with_creator(
            self.stores.stores_store.clone(),
            self.stores.stores_entity.clone(),
        )));
        registry.register(Box::new(ActivityDescriptor::new_with_creator(
            self.stores.activities_store.clone(),
            self.stores.activities_entity.clone(),
        )));
        registry.register(Box::new(WarehouseDescriptor::new_with_creator(
            self.stores.warehouses_store.clone(),
            self.stores.warehouses_entity.clone(),
        )));
        registry.register(Box::new(StockItemDescriptor::new_with_creator(
            self.stores.stock_items_store.clone(),
            self.stores.stock_items_entity.clone(),
        )));
        registry.register(Box::new(StockMovementDescriptor::new_with_creator(
            self.stores.stock_movements_store.clone(),
            self.stores.stock_movements_entity.clone(),
        )));
        registry.register(Box::new(UsageDescriptor::new_with_creator(
            self.stores.usages_store.clone(),
            self.stores.usages_entity.clone(),
        )));
    }
}





