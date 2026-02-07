use async_trait::async_trait;
use std::sync::Arc;
use this::prelude::*;

use super::Store;

#[derive(Debug, thiserror::Error)]
pub enum StoreStoreError {
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
pub trait StoreStore: Send + Sync {
    async fn create(&self, store: Store) -> Result<Store, StoreStoreError>;
    async fn get(&self, id: &Uuid) -> Result<Store, StoreStoreError>;
    async fn update(&self, store: Store) -> Result<Store, StoreStoreError>;
    async fn delete(&self, id: &Uuid) -> Result<(), StoreStoreError>;
    async fn list(&self) -> Result<Vec<Store>, StoreStoreError>;
}

use tokio::sync::RwLock;

#[derive(Clone, Default)]
pub struct InMemoryStoreStore {
    inner: Arc<RwLock<Vec<Store>>>,
}

#[async_trait::async_trait]
impl EntityFetcher for InMemoryStoreStore {
    async fn fetch_as_json(&self, entity_id: &Uuid) -> Result<serde_json::Value, anyhow::Error> {
        let store = self
            .get(entity_id)
            .await
            .map_err(|_| anyhow::anyhow!("Store not found: {}", entity_id))?;
        Ok(serde_json::to_value(store)?)
    }

    async fn list_as_json(
        &self,
        limit: Option<i32>,
        offset: Option<i32>,
    ) -> Result<Vec<serde_json::Value>, anyhow::Error> {
        let all_stores = self.list().await?;
        let offset = offset.unwrap_or(0) as usize;
        let limit = limit.unwrap_or(20) as usize;

        let stores: Vec<Store> = all_stores.into_iter().skip(offset).take(limit).collect();
        stores
            .into_iter()
            .map(|store| serde_json::to_value(store).map_err(Into::into))
            .collect()
    }
}

#[async_trait::async_trait]
impl EntityCreator for InMemoryStoreStore {
    async fn create_from_json(
        &self,
        entity_data: serde_json::Value,
    ) -> Result<serde_json::Value, anyhow::Error> {
        let store = Store::new(
            entity_data["name"].as_str().unwrap_or("Store").to_string(),
            entity_data["status"]
                .as_str()
                .unwrap_or("active")
                .to_string(),
            entity_data["address"].as_str().map(String::from),
        );

        self.create(store.clone()).await?;
        Ok(serde_json::to_value(store)?)
    }
}

#[async_trait::async_trait]
impl StoreStore for InMemoryStoreStore {
    async fn create(&self, store: Store) -> Result<Store, StoreStoreError> {
        let mut g = self.inner.write().await;
        if g.iter().any(|s| s.id == store.id) {
            return Err(StoreStoreError::Conflict(store.id.to_string()));
        }
        g.push(store.clone());
        Ok(store)
    }

    async fn get(&self, id: &Uuid) -> Result<Store, StoreStoreError> {
        let g = self.inner.read().await;
        g.iter()
            .find(|s| &s.id == id)
            .cloned()
            .ok_or_else(|| StoreStoreError::NotFound(id.to_string()))
    }

    async fn update(&self, store: Store) -> Result<Store, StoreStoreError> {
        let mut g = self.inner.write().await;
        if let Some(x) = g.iter_mut().find(|s| s.id == store.id) {
            *x = store.clone();
            Ok(store)
        } else {
            Err(StoreStoreError::NotFound(store.id.to_string()))
        }
    }

    async fn delete(&self, id: &Uuid) -> Result<(), StoreStoreError> {
        let mut g = self.inner.write().await;
        let before = g.len();
        g.retain(|s| &s.id != id);
        if g.len() == before {
            return Err(StoreStoreError::NotFound(id.to_string()));
        }
        Ok(())
    }

    async fn list(&self) -> Result<Vec<Store>, StoreStoreError> {
        Ok(self.inner.read().await.clone())
    }
}

#[cfg(feature = "dynamodb")]
use aws_sdk_dynamodb::Client as DynamoDBClient;
#[cfg(feature = "dynamodb")]
use this::storage::DynamoDBDataService;

#[cfg(feature = "dynamodb")]
#[derive(Clone)]
pub struct StoreDynamoDBStore {
    service: Arc<DynamoDBDataService<Store>>,
}

#[cfg(feature = "dynamodb")]
impl StoreDynamoDBStore {
    pub fn new(client: DynamoDBClient, table_name: String) -> Self {
        Self {
            service: Arc::new(DynamoDBDataService::new(client, table_name)),
        }
    }
}

#[cfg(feature = "dynamodb")]
#[async_trait::async_trait]
impl EntityFetcher for StoreDynamoDBStore {
    async fn fetch_as_json(&self, entity_id: &Uuid) -> Result<serde_json::Value, anyhow::Error> {
        let store = self
            .get(entity_id)
            .await
            .map_err(|_| anyhow::anyhow!("Store not found: {}", entity_id))?;
        Ok(serde_json::to_value(store)?)
    }

    async fn list_as_json(
        &self,
        limit: Option<i32>,
        offset: Option<i32>,
    ) -> Result<Vec<serde_json::Value>, anyhow::Error> {
        let all_stores = self.list().await?;
        let offset = offset.unwrap_or(0) as usize;
        let limit = limit.unwrap_or(20) as usize;

        let stores: Vec<Store> = all_stores.into_iter().skip(offset).take(limit).collect();
        stores
            .into_iter()
            .map(|store| serde_json::to_value(store).map_err(Into::into))
            .collect()
    }
}

#[cfg(feature = "dynamodb")]
#[async_trait::async_trait]
impl EntityCreator for StoreDynamoDBStore {
    async fn create_from_json(
        &self,
        entity_data: serde_json::Value,
    ) -> Result<serde_json::Value, anyhow::Error> {
        let store = Store::new(
            entity_data["name"].as_str().unwrap_or("Store").to_string(),
            entity_data["status"]
                .as_str()
                .unwrap_or("active")
                .to_string(),
            entity_data["address"].as_str().map(String::from),
        );

        self.create(store.clone()).await?;
        Ok(serde_json::to_value(store)?)
    }
}

#[cfg(feature = "dynamodb")]
#[async_trait::async_trait]
impl StoreStore for StoreDynamoDBStore {
    async fn create(&self, store: Store) -> Result<Store, StoreStoreError> {
        self.service
            .create(store.clone())
            .await
            .map_err(|e| StoreStoreError::Other(anyhow::anyhow!(e)))
    }

    async fn get(&self, id: &Uuid) -> Result<Store, StoreStoreError> {
        self.service
            .get(id)
            .await
            .map_err(|e| StoreStoreError::Other(anyhow::anyhow!(e)))?
            .ok_or_else(|| StoreStoreError::NotFound(id.to_string()))
    }

    async fn update(&self, store: Store) -> Result<Store, StoreStoreError> {
        self.service
            .update(&store.id, store.clone())
            .await
            .map_err(|e| StoreStoreError::Other(anyhow::anyhow!(e)))
    }

    async fn delete(&self, id: &Uuid) -> Result<(), StoreStoreError> {
        self.service
            .delete(id)
            .await
            .map_err(|e| StoreStoreError::Other(anyhow::anyhow!(e)))
    }

    async fn list(&self) -> Result<Vec<Store>, StoreStoreError> {
        self.service
            .list()
            .await
            .map_err(|e| StoreStoreError::Other(anyhow::anyhow!(e)))
    }
}





