use async_trait::async_trait;
use std::sync::Arc;
use this::prelude::*;

use super::Warehouse;

#[derive(Debug, thiserror::Error)]
pub enum WarehouseStoreError {
    #[error("not found: {0}")]
    NotFound(String),
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("validation: {0}")]
    Validation(String),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[async_trait]
pub trait WarehouseStore: Send + Sync {
    async fn create(&self, warehouse: Warehouse) -> Result<Warehouse, WarehouseStoreError>;
    async fn get(&self, id: &Uuid) -> Result<Warehouse, WarehouseStoreError>;
    async fn update(&self, warehouse: Warehouse) -> Result<Warehouse, WarehouseStoreError>;
    async fn delete(&self, id: &Uuid) -> Result<(), WarehouseStoreError>;
    async fn list(&self) -> Result<Vec<Warehouse>, WarehouseStoreError>;
}

use tokio::sync::RwLock;

#[derive(Clone, Default)]
pub struct InMemoryWarehouseStore {
    inner: Arc<RwLock<Vec<Warehouse>>>,
}

#[async_trait::async_trait]
impl EntityFetcher for InMemoryWarehouseStore {
    async fn fetch_as_json(&self, entity_id: &Uuid) -> Result<serde_json::Value, anyhow::Error> {
        let warehouse = self
            .get(entity_id)
            .await
            .map_err(|_| anyhow::anyhow!("Warehouse not found: {}", entity_id))?;
        Ok(serde_json::to_value(warehouse)?)
    }

    async fn list_as_json(
        &self,
        limit: Option<i32>,
        offset: Option<i32>,
    ) -> Result<Vec<serde_json::Value>, anyhow::Error> {
        let all_warehouses = self.list().await?;
        let offset = offset.unwrap_or(0) as usize;
        let limit = limit.unwrap_or(20) as usize;

        let warehouses: Vec<Warehouse> = all_warehouses
            .into_iter()
            .skip(offset)
            .take(limit)
            .collect();
        warehouses
            .into_iter()
            .map(|warehouse| serde_json::to_value(warehouse).map_err(Into::into))
            .collect()
    }
}

#[async_trait::async_trait]
impl EntityCreator for InMemoryWarehouseStore {
    async fn create_from_json(
        &self,
        entity_data: serde_json::Value,
    ) -> Result<serde_json::Value, anyhow::Error> {
        let warehouse = Warehouse::new(
            entity_data["name"]
                .as_str()
                .unwrap_or("Warehouse")
                .to_string(),
            entity_data["status"]
                .as_str()
                .unwrap_or("active")
                .to_string(),
            entity_data["location"].as_str().map(String::from),
            entity_data["store_id"]
                .as_str()
                .and_then(|s| Uuid::parse_str(s).ok())
                .unwrap_or_else(Uuid::new_v4),
        );

        self.create(warehouse.clone()).await?;
        Ok(serde_json::to_value(warehouse)?)
    }
}

#[async_trait::async_trait]
impl WarehouseStore for InMemoryWarehouseStore {
    async fn create(&self, warehouse: Warehouse) -> Result<Warehouse, WarehouseStoreError> {
        let mut g = self.inner.write().await;
        if g.iter().any(|w| w.id == warehouse.id) {
            return Err(WarehouseStoreError::Conflict(warehouse.id.to_string()));
        }
        g.push(warehouse.clone());
        Ok(warehouse)
    }

    async fn get(&self, id: &Uuid) -> Result<Warehouse, WarehouseStoreError> {
        let g = self.inner.read().await;
        g.iter()
            .find(|w| &w.id == id)
            .cloned()
            .ok_or_else(|| WarehouseStoreError::NotFound(id.to_string()))
    }

    async fn update(&self, warehouse: Warehouse) -> Result<Warehouse, WarehouseStoreError> {
        let mut g = self.inner.write().await;
        if let Some(x) = g.iter_mut().find(|w| w.id == warehouse.id) {
            *x = warehouse.clone();
            Ok(warehouse)
        } else {
            Err(WarehouseStoreError::NotFound(warehouse.id.to_string()))
        }
    }

    async fn delete(&self, id: &Uuid) -> Result<(), WarehouseStoreError> {
        let mut g = self.inner.write().await;
        let before = g.len();
        g.retain(|w| &w.id != id);
        if g.len() == before {
            return Err(WarehouseStoreError::NotFound(id.to_string()));
        }
        Ok(())
    }

    async fn list(&self) -> Result<Vec<Warehouse>, WarehouseStoreError> {
        Ok(self.inner.read().await.clone())
    }
}

#[cfg(feature = "dynamodb")]
use aws_sdk_dynamodb::Client as DynamoDBClient;
#[cfg(feature = "dynamodb")]
use this::storage::DynamoDBDataService;

#[cfg(feature = "dynamodb")]
#[derive(Clone)]
pub struct WarehouseDynamoDBStore {
    service: Arc<DynamoDBDataService<Warehouse>>,
}

#[cfg(feature = "dynamodb")]
impl WarehouseDynamoDBStore {
    pub fn new(client: DynamoDBClient, table_name: String) -> Self {
        Self {
            service: Arc::new(DynamoDBDataService::new(client, table_name)),
        }
    }
}

#[cfg(feature = "dynamodb")]
#[async_trait::async_trait]
impl EntityFetcher for WarehouseDynamoDBStore {
    async fn fetch_as_json(&self, entity_id: &Uuid) -> Result<serde_json::Value, anyhow::Error> {
        let warehouse = self
            .get(entity_id)
            .await
            .map_err(|_| anyhow::anyhow!("Warehouse not found: {}", entity_id))?;
        Ok(serde_json::to_value(warehouse)?)
    }

    async fn list_as_json(
        &self,
        limit: Option<i32>,
        offset: Option<i32>,
    ) -> Result<Vec<serde_json::Value>, anyhow::Error> {
        let all_warehouses = self.list().await?;
        let offset = offset.unwrap_or(0) as usize;
        let limit = limit.unwrap_or(20) as usize;

        let warehouses: Vec<Warehouse> = all_warehouses
            .into_iter()
            .skip(offset)
            .take(limit)
            .collect();
        warehouses
            .into_iter()
            .map(|warehouse| serde_json::to_value(warehouse).map_err(Into::into))
            .collect()
    }
}

#[cfg(feature = "dynamodb")]
#[async_trait::async_trait]
impl EntityCreator for WarehouseDynamoDBStore {
    async fn create_from_json(
        &self,
        entity_data: serde_json::Value,
    ) -> Result<serde_json::Value, anyhow::Error> {
        let warehouse = Warehouse::new(
            entity_data["name"]
                .as_str()
                .unwrap_or("Warehouse")
                .to_string(),
            entity_data["status"]
                .as_str()
                .unwrap_or("active")
                .to_string(),
            entity_data["location"].as_str().map(String::from),
            entity_data["store_id"]
                .as_str()
                .and_then(|s| Uuid::parse_str(s).ok())
                .unwrap_or_else(Uuid::new_v4),
        );

        self.create(warehouse.clone()).await?;
        Ok(serde_json::to_value(warehouse)?)
    }
}

#[cfg(feature = "dynamodb")]
#[async_trait::async_trait]
impl WarehouseStore for WarehouseDynamoDBStore {
    async fn create(&self, warehouse: Warehouse) -> Result<Warehouse, WarehouseStoreError> {
        self.service
            .create(warehouse.clone())
            .await
            .map_err(|e| WarehouseStoreError::Other(anyhow::anyhow!(e)))
    }

    async fn get(&self, id: &Uuid) -> Result<Warehouse, WarehouseStoreError> {
        self.service
            .get(id)
            .await
            .map_err(|e| WarehouseStoreError::Other(anyhow::anyhow!(e)))?
            .ok_or_else(|| WarehouseStoreError::NotFound(id.to_string()))
    }

    async fn update(&self, warehouse: Warehouse) -> Result<Warehouse, WarehouseStoreError> {
        self.service
            .update(&warehouse.id, warehouse.clone())
            .await
            .map_err(|e| WarehouseStoreError::Other(anyhow::anyhow!(e)))
    }

    async fn delete(&self, id: &Uuid) -> Result<(), WarehouseStoreError> {
        self.service
            .delete(id)
            .await
            .map_err(|e| WarehouseStoreError::Other(anyhow::anyhow!(e)))
    }

    async fn list(&self) -> Result<Vec<Warehouse>, WarehouseStoreError> {
        self.service
            .list()
            .await
            .map_err(|e| WarehouseStoreError::Other(anyhow::anyhow!(e)))
    }
}
