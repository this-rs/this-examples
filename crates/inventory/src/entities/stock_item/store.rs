use async_trait::async_trait;
use std::sync::Arc;
use this::prelude::*;

use super::StockItem;

#[derive(Debug, thiserror::Error)]
pub enum StockItemStoreError {
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
pub trait StockItemStore: Send + Sync {
    async fn create(&self, stock_item: StockItem) -> Result<StockItem, StockItemStoreError>;
    async fn get(&self, id: &Uuid) -> Result<StockItem, StockItemStoreError>;
    async fn update(&self, stock_item: StockItem) -> Result<StockItem, StockItemStoreError>;
    async fn delete(&self, id: &Uuid) -> Result<(), StockItemStoreError>;
    async fn list(&self) -> Result<Vec<StockItem>, StockItemStoreError>;
}

use tokio::sync::RwLock;

#[derive(Clone, Default)]
pub struct InMemoryStockItemStore {
    inner: Arc<RwLock<Vec<StockItem>>>,
}

#[async_trait::async_trait]
impl EntityFetcher for InMemoryStockItemStore {
    async fn fetch_as_json(&self, entity_id: &Uuid) -> Result<serde_json::Value, anyhow::Error> {
        let stock_item = self
            .get(entity_id)
            .await
            .map_err(|_| anyhow::anyhow!("StockItem not found: {}", entity_id))?;
        Ok(serde_json::to_value(stock_item)?)
    }

    async fn list_as_json(
        &self,
        limit: Option<i32>,
        offset: Option<i32>,
    ) -> Result<Vec<serde_json::Value>, anyhow::Error> {
        let all_stock_items = self.list().await?;
        let offset = offset.unwrap_or(0) as usize;
        let limit = limit.unwrap_or(20) as usize;

        let stock_items: Vec<StockItem> = all_stock_items
            .into_iter()
            .skip(offset)
            .take(limit)
            .collect();
        stock_items
            .into_iter()
            .map(|stock_item| serde_json::to_value(stock_item).map_err(Into::into))
            .collect()
    }
}

#[async_trait::async_trait]
impl EntityCreator for InMemoryStockItemStore {
    async fn create_from_json(
        &self,
        entity_data: serde_json::Value,
    ) -> Result<serde_json::Value, anyhow::Error> {
        let stock_item = StockItem::new(
            entity_data["name"]
                .as_str()
                .unwrap_or("StockItem")
                .to_string(),
            entity_data["status"]
                .as_str()
                .unwrap_or("available")
                .to_string(),
            entity_data["product_id"]
                .as_str()
                .and_then(|s| Uuid::parse_str(s).ok()),
            entity_data["quantity"].as_i64().unwrap_or(0) as i32,
            entity_data["warehouse_id"]
                .as_str()
                .and_then(|s| Uuid::parse_str(s).ok())
                .unwrap_or_else(Uuid::new_v4),
            entity_data["reserved_quantity"].as_i64().map(|v| v as i32),
        );

        self.create(stock_item.clone()).await?;
        Ok(serde_json::to_value(stock_item)?)
    }
}

#[async_trait::async_trait]
impl StockItemStore for InMemoryStockItemStore {
    async fn create(&self, stock_item: StockItem) -> Result<StockItem, StockItemStoreError> {
        let mut g = self.inner.write().await;
        if g.iter().any(|s| s.id == stock_item.id) {
            return Err(StockItemStoreError::Conflict(stock_item.id.to_string()));
        }
        g.push(stock_item.clone());
        Ok(stock_item)
    }

    async fn get(&self, id: &Uuid) -> Result<StockItem, StockItemStoreError> {
        let g = self.inner.read().await;
        g.iter()
            .find(|s| &s.id == id)
            .cloned()
            .ok_or_else(|| StockItemStoreError::NotFound(id.to_string()))
    }

    async fn update(&self, stock_item: StockItem) -> Result<StockItem, StockItemStoreError> {
        let mut g = self.inner.write().await;
        if let Some(x) = g.iter_mut().find(|s| s.id == stock_item.id) {
            *x = stock_item.clone();
            Ok(stock_item)
        } else {
            Err(StockItemStoreError::NotFound(stock_item.id.to_string()))
        }
    }

    async fn delete(&self, id: &Uuid) -> Result<(), StockItemStoreError> {
        let mut g = self.inner.write().await;
        let before = g.len();
        g.retain(|s| &s.id != id);
        if g.len() == before {
            return Err(StockItemStoreError::NotFound(id.to_string()));
        }
        Ok(())
    }

    async fn list(&self) -> Result<Vec<StockItem>, StockItemStoreError> {
        Ok(self.inner.read().await.clone())
    }
}

#[cfg(feature = "dynamodb")]
use aws_sdk_dynamodb::Client as DynamoDBClient;
#[cfg(feature = "dynamodb")]
use this::storage::DynamoDBDataService;

#[cfg(feature = "dynamodb")]
#[derive(Clone)]
pub struct StockItemDynamoDBStore {
    service: Arc<DynamoDBDataService<StockItem>>,
}

#[cfg(feature = "dynamodb")]
impl StockItemDynamoDBStore {
    pub fn new(client: DynamoDBClient, table_name: String) -> Self {
        Self {
            service: Arc::new(DynamoDBDataService::new(client, table_name)),
        }
    }
}

#[cfg(feature = "dynamodb")]
#[async_trait::async_trait]
impl EntityFetcher for StockItemDynamoDBStore {
    async fn fetch_as_json(&self, entity_id: &Uuid) -> Result<serde_json::Value, anyhow::Error> {
        let stock_item = self
            .get(entity_id)
            .await
            .map_err(|_| anyhow::anyhow!("StockItem not found: {}", entity_id))?;
        Ok(serde_json::to_value(stock_item)?)
    }

    async fn list_as_json(
        &self,
        limit: Option<i32>,
        offset: Option<i32>,
    ) -> Result<Vec<serde_json::Value>, anyhow::Error> {
        let all_stock_items = self.list().await?;
        let offset = offset.unwrap_or(0) as usize;
        let limit = limit.unwrap_or(20) as usize;

        let stock_items: Vec<StockItem> = all_stock_items
            .into_iter()
            .skip(offset)
            .take(limit)
            .collect();
        stock_items
            .into_iter()
            .map(|stock_item| serde_json::to_value(stock_item).map_err(Into::into))
            .collect()
    }
}

#[cfg(feature = "dynamodb")]
#[async_trait::async_trait]
impl EntityCreator for StockItemDynamoDBStore {
    async fn create_from_json(
        &self,
        entity_data: serde_json::Value,
    ) -> Result<serde_json::Value, anyhow::Error> {
        let stock_item = StockItem::new(
            entity_data["name"]
                .as_str()
                .unwrap_or("StockItem")
                .to_string(),
            entity_data["status"]
                .as_str()
                .unwrap_or("available")
                .to_string(),
            entity_data["product_id"]
                .as_str()
                .and_then(|s| Uuid::parse_str(s).ok()),
            entity_data["quantity"].as_i64().unwrap_or(0) as i32,
            entity_data["warehouse_id"]
                .as_str()
                .and_then(|s| Uuid::parse_str(s).ok())
                .unwrap_or_else(Uuid::new_v4),
            entity_data["reserved_quantity"].as_i64().map(|v| v as i32),
        );

        self.create(stock_item.clone()).await?;
        Ok(serde_json::to_value(stock_item)?)
    }
}

#[cfg(feature = "dynamodb")]
#[async_trait::async_trait]
impl StockItemStore for StockItemDynamoDBStore {
    async fn create(&self, stock_item: StockItem) -> Result<StockItem, StockItemStoreError> {
        self.service
            .create(stock_item.clone())
            .await
            .map_err(|e| StockItemStoreError::Other(anyhow::anyhow!(e)))
    }

    async fn get(&self, id: &Uuid) -> Result<StockItem, StockItemStoreError> {
        self.service
            .get(id)
            .await
            .map_err(|e| StockItemStoreError::Other(anyhow::anyhow!(e)))?
            .ok_or_else(|| StockItemStoreError::NotFound(id.to_string()))
    }

    async fn update(&self, stock_item: StockItem) -> Result<StockItem, StockItemStoreError> {
        self.service
            .update(&stock_item.id, stock_item.clone())
            .await
            .map_err(|e| StockItemStoreError::Other(anyhow::anyhow!(e)))
    }

    async fn delete(&self, id: &Uuid) -> Result<(), StockItemStoreError> {
        self.service
            .delete(id)
            .await
            .map_err(|e| StockItemStoreError::Other(anyhow::anyhow!(e)))
    }

    async fn list(&self) -> Result<Vec<StockItem>, StockItemStoreError> {
        self.service
            .list()
            .await
            .map_err(|e| StockItemStoreError::Other(anyhow::anyhow!(e)))
    }
}
