use async_trait::async_trait;
use std::sync::Arc;
use this::prelude::*;

use super::StockMovement;

#[derive(Debug, thiserror::Error)]
pub enum StockMovementStoreError {
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
pub trait StockMovementStore: Send + Sync {
    async fn create(
        &self,
        stock_movement: StockMovement,
    ) -> Result<StockMovement, StockMovementStoreError>;
    async fn get(&self, id: &Uuid) -> Result<StockMovement, StockMovementStoreError>;
    async fn update(
        &self,
        stock_movement: StockMovement,
    ) -> Result<StockMovement, StockMovementStoreError>;
    async fn delete(&self, id: &Uuid) -> Result<(), StockMovementStoreError>;
    async fn list(&self) -> Result<Vec<StockMovement>, StockMovementStoreError>;
}

use tokio::sync::RwLock;

#[derive(Clone, Default)]
pub struct InMemoryStockMovementStore {
    inner: Arc<RwLock<Vec<StockMovement>>>,
}

#[async_trait::async_trait]
impl EntityFetcher for InMemoryStockMovementStore {
    async fn fetch_as_json(&self, entity_id: &Uuid) -> Result<serde_json::Value, anyhow::Error> {
        let stock_movement = self
            .get(entity_id)
            .await
            .map_err(|_| anyhow::anyhow!("StockMovement not found: {}", entity_id))?;
        Ok(serde_json::to_value(stock_movement)?)
    }

    async fn list_as_json(
        &self,
        limit: Option<i32>,
        offset: Option<i32>,
    ) -> Result<Vec<serde_json::Value>, anyhow::Error> {
        let all_stock_movements = self.list().await?;
        let offset = offset.unwrap_or(0) as usize;
        let limit = limit.unwrap_or(20) as usize;

        let stock_movements: Vec<StockMovement> = all_stock_movements
            .into_iter()
            .skip(offset)
            .take(limit)
            .collect();
        stock_movements
            .into_iter()
            .map(|stock_movement| serde_json::to_value(stock_movement).map_err(Into::into))
            .collect()
    }
}

#[async_trait::async_trait]
impl EntityCreator for InMemoryStockMovementStore {
    async fn create_from_json(
        &self,
        entity_data: serde_json::Value,
    ) -> Result<serde_json::Value, anyhow::Error> {
        let stock_movement = StockMovement::new(
            entity_data["name"]
                .as_str()
                .unwrap_or("StockMovement")
                .to_string(),
            entity_data["status"]
                .as_str()
                .unwrap_or("pending")
                .to_string(),
            entity_data["stock_item_id"]
                .as_str()
                .and_then(|s| Uuid::parse_str(s).ok())
                .unwrap_or_else(Uuid::new_v4),
            entity_data["movement_type"]
                .as_str()
                .unwrap_or("in")
                .to_string(),
            entity_data["quantity"].as_i64().unwrap_or(0) as i32,
            entity_data["reason"].as_str().map(String::from),
            entity_data["activity_id"]
                .as_str()
                .and_then(|s| Uuid::parse_str(s).ok()),
        );

        self.create(stock_movement.clone()).await?;
        Ok(serde_json::to_value(stock_movement)?)
    }
}

#[async_trait::async_trait]
impl StockMovementStore for InMemoryStockMovementStore {
    async fn create(
        &self,
        stock_movement: StockMovement,
    ) -> Result<StockMovement, StockMovementStoreError> {
        let mut g = self.inner.write().await;
        if g.iter().any(|s| s.id == stock_movement.id) {
            return Err(StockMovementStoreError::Conflict(
                stock_movement.id.to_string(),
            ));
        }
        g.push(stock_movement.clone());
        Ok(stock_movement)
    }

    async fn get(&self, id: &Uuid) -> Result<StockMovement, StockMovementStoreError> {
        let g = self.inner.read().await;
        g.iter()
            .find(|s| &s.id == id)
            .cloned()
            .ok_or_else(|| StockMovementStoreError::NotFound(id.to_string()))
    }

    async fn update(
        &self,
        stock_movement: StockMovement,
    ) -> Result<StockMovement, StockMovementStoreError> {
        let mut g = self.inner.write().await;
        if let Some(x) = g.iter_mut().find(|s| s.id == stock_movement.id) {
            *x = stock_movement.clone();
            Ok(stock_movement)
        } else {
            Err(StockMovementStoreError::NotFound(
                stock_movement.id.to_string(),
            ))
        }
    }

    async fn delete(&self, id: &Uuid) -> Result<(), StockMovementStoreError> {
        let mut g = self.inner.write().await;
        let before = g.len();
        g.retain(|s| &s.id != id);
        if g.len() == before {
            return Err(StockMovementStoreError::NotFound(id.to_string()));
        }
        Ok(())
    }

    async fn list(&self) -> Result<Vec<StockMovement>, StockMovementStoreError> {
        Ok(self.inner.read().await.clone())
    }
}

#[cfg(feature = "dynamodb")]
use aws_sdk_dynamodb::Client as DynamoDBClient;
#[cfg(feature = "dynamodb")]
use this::storage::DynamoDBDataService;

#[cfg(feature = "dynamodb")]
#[derive(Clone)]
pub struct StockMovementDynamoDBStore {
    service: Arc<DynamoDBDataService<StockMovement>>,
}

#[cfg(feature = "dynamodb")]
impl StockMovementDynamoDBStore {
    pub fn new(client: DynamoDBClient, table_name: String) -> Self {
        Self {
            service: Arc::new(DynamoDBDataService::new(client, table_name)),
        }
    }
}

#[cfg(feature = "dynamodb")]
#[async_trait::async_trait]
impl EntityFetcher for StockMovementDynamoDBStore {
    async fn fetch_as_json(&self, entity_id: &Uuid) -> Result<serde_json::Value, anyhow::Error> {
        let stock_movement = self
            .get(entity_id)
            .await
            .map_err(|_| anyhow::anyhow!("StockMovement not found: {}", entity_id))?;
        Ok(serde_json::to_value(stock_movement)?)
    }

    async fn list_as_json(
        &self,
        limit: Option<i32>,
        offset: Option<i32>,
    ) -> Result<Vec<serde_json::Value>, anyhow::Error> {
        let all_stock_movements = self.list().await?;
        let offset = offset.unwrap_or(0) as usize;
        let limit = limit.unwrap_or(20) as usize;

        let stock_movements: Vec<StockMovement> = all_stock_movements
            .into_iter()
            .skip(offset)
            .take(limit)
            .collect();
        stock_movements
            .into_iter()
            .map(|stock_movement| serde_json::to_value(stock_movement).map_err(Into::into))
            .collect()
    }
}

#[cfg(feature = "dynamodb")]
#[async_trait::async_trait]
impl EntityCreator for StockMovementDynamoDBStore {
    async fn create_from_json(
        &self,
        entity_data: serde_json::Value,
    ) -> Result<serde_json::Value, anyhow::Error> {
        let stock_movement = StockMovement::new(
            entity_data["name"]
                .as_str()
                .unwrap_or("StockMovement")
                .to_string(),
            entity_data["status"]
                .as_str()
                .unwrap_or("pending")
                .to_string(),
            entity_data["stock_item_id"]
                .as_str()
                .and_then(|s| Uuid::parse_str(s).ok())
                .unwrap_or_else(Uuid::new_v4),
            entity_data["movement_type"]
                .as_str()
                .unwrap_or("in")
                .to_string(),
            entity_data["quantity"].as_i64().unwrap_or(0) as i32,
            entity_data["reason"].as_str().map(String::from),
            entity_data["activity_id"]
                .as_str()
                .and_then(|s| Uuid::parse_str(s).ok()),
        );

        self.create(stock_movement.clone()).await?;
        Ok(serde_json::to_value(stock_movement)?)
    }
}

#[cfg(feature = "dynamodb")]
#[async_trait::async_trait]
impl StockMovementStore for StockMovementDynamoDBStore {
    async fn create(
        &self,
        stock_movement: StockMovement,
    ) -> Result<StockMovement, StockMovementStoreError> {
        self.service
            .create(stock_movement.clone())
            .await
            .map_err(|e| StockMovementStoreError::Other(anyhow::anyhow!(e)))
    }

    async fn get(&self, id: &Uuid) -> Result<StockMovement, StockMovementStoreError> {
        self.service
            .get(id)
            .await
            .map_err(|e| StockMovementStoreError::Other(anyhow::anyhow!(e)))?
            .ok_or_else(|| StockMovementStoreError::NotFound(id.to_string()))
    }

    async fn update(
        &self,
        stock_movement: StockMovement,
    ) -> Result<StockMovement, StockMovementStoreError> {
        self.service
            .update(&stock_movement.id, stock_movement.clone())
            .await
            .map_err(|e| StockMovementStoreError::Other(anyhow::anyhow!(e)))
    }

    async fn delete(&self, id: &Uuid) -> Result<(), StockMovementStoreError> {
        self.service
            .delete(id)
            .await
            .map_err(|e| StockMovementStoreError::Other(anyhow::anyhow!(e)))
    }

    async fn list(&self) -> Result<Vec<StockMovement>, StockMovementStoreError> {
        self.service
            .list()
            .await
            .map_err(|e| StockMovementStoreError::Other(anyhow::anyhow!(e)))
    }
}
