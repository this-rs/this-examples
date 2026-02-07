use std::sync::Arc;

use crate::module::InventoryStores;

// Import stores from entity modules
use crate::entities::activity::InMemoryActivityStore;
use crate::entities::stock_item::InMemoryStockItemStore;
use crate::entities::stock_movement::InMemoryStockMovementStore;
use crate::entities::store::InMemoryStoreStore;
use crate::entities::usage::InMemoryUsageStore;
use crate::entities::warehouse::InMemoryWarehouseStore;

#[cfg(feature = "dynamodb")]
use crate::entities::activity::ActivityDynamoDBStore;
#[cfg(feature = "dynamodb")]
use crate::entities::stock_item::StockItemDynamoDBStore;
#[cfg(feature = "dynamodb")]
use crate::entities::stock_movement::StockMovementDynamoDBStore;
#[cfg(feature = "dynamodb")]
use crate::entities::store::StoreDynamoDBStore;
#[cfg(feature = "dynamodb")]
use crate::entities::usage::UsageDynamoDBStore;
#[cfg(feature = "dynamodb")]
use crate::entities::warehouse::WarehouseDynamoDBStore;
#[cfg(feature = "dynamodb")]
use aws_sdk_dynamodb::Client as DynamoDBClient;

// ============================================================================
// Store Factories
// ============================================================================

impl InventoryStores {
    /// Create stores with in-memory implementations
    pub fn new_in_memory() -> Self {
        let stores = Arc::new(InMemoryStoreStore::default());
        let activities = Arc::new(InMemoryActivityStore::default());
        let warehouses = Arc::new(InMemoryWarehouseStore::default());
        let stock_items = Arc::new(InMemoryStockItemStore::default());
        let stock_movements = Arc::new(InMemoryStockMovementStore::default());
        let usages = Arc::new(InMemoryUsageStore::default());

        Self {
            stores_store: stores.clone(),
            stores_entity: stores,
            activities_store: activities.clone(),
            activities_entity: activities,
            warehouses_store: warehouses.clone(),
            warehouses_entity: warehouses,
            stock_items_store: stock_items.clone(),
            stock_items_entity: stock_items,
            stock_movements_store: stock_movements.clone(),
            stock_movements_entity: stock_movements,
            usages_store: usages.clone(),
            usages_entity: usages,
        }
    }

    #[cfg(feature = "dynamodb")]
    /// Create stores with DynamoDB implementations
    pub fn new_dynamodb(
        client: DynamoDBClient,
        stores_table: String,
        activities_table: String,
        warehouses_table: String,
        stock_items_table: String,
        stock_movements_table: String,
        usages_table: String,
    ) -> Self {
        let stores = Arc::new(StoreDynamoDBStore::new(client.clone(), stores_table));
        let activities = Arc::new(ActivityDynamoDBStore::new(client.clone(), activities_table));
        let warehouses = Arc::new(WarehouseDynamoDBStore::new(
            client.clone(),
            warehouses_table,
        ));
        let stock_items = Arc::new(StockItemDynamoDBStore::new(
            client.clone(),
            stock_items_table,
        ));
        let stock_movements = Arc::new(StockMovementDynamoDBStore::new(
            client.clone(),
            stock_movements_table,
        ));
        let usages = Arc::new(UsageDynamoDBStore::new(client, usages_table));

        Self {
            stores_store: stores.clone(),
            stores_entity: stores,
            activities_store: activities.clone(),
            activities_entity: activities,
            warehouses_store: warehouses.clone(),
            warehouses_entity: warehouses,
            stock_items_store: stock_items.clone(),
            stock_items_entity: stock_items,
            stock_movements_store: stock_movements.clone(),
            stock_movements_entity: stock_movements,
            usages_store: usages.clone(),
            usages_entity: usages,
        }
    }
}





