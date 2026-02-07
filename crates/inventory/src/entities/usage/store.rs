use async_trait::async_trait;
use std::sync::Arc;
use this::prelude::*;

use super::Usage;

#[derive(Debug, thiserror::Error)]
pub enum UsageStoreError {
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
pub trait UsageStore: Send + Sync {
    async fn create(&self, usage: Usage) -> Result<Usage, UsageStoreError>;
    async fn get(&self, id: &Uuid) -> Result<Usage, UsageStoreError>;
    async fn update(&self, usage: Usage) -> Result<Usage, UsageStoreError>;
    async fn delete(&self, id: &Uuid) -> Result<(), UsageStoreError>;
    async fn list(&self) -> Result<Vec<Usage>, UsageStoreError>;
}

use tokio::sync::RwLock;

#[derive(Clone, Default)]
pub struct InMemoryUsageStore {
    inner: Arc<RwLock<Vec<Usage>>>,
}

#[async_trait::async_trait]
impl EntityFetcher for InMemoryUsageStore {
    async fn fetch_as_json(&self, entity_id: &Uuid) -> Result<serde_json::Value, anyhow::Error> {
        let usage = self
            .get(entity_id)
            .await
            .map_err(|_| anyhow::anyhow!("Usage not found: {}", entity_id))?;
        Ok(serde_json::to_value(usage)?)
    }

    async fn list_as_json(
        &self,
        limit: Option<i32>,
        offset: Option<i32>,
    ) -> Result<Vec<serde_json::Value>, anyhow::Error> {
        let all_usages = self.list().await?;
        let offset = offset.unwrap_or(0) as usize;
        let limit = limit.unwrap_or(20) as usize;

        let usages: Vec<Usage> = all_usages.into_iter().skip(offset).take(limit).collect();
        usages
            .into_iter()
            .map(|usage| serde_json::to_value(usage).map_err(Into::into))
            .collect()
    }
}

#[async_trait::async_trait]
impl EntityCreator for InMemoryUsageStore {
    async fn create_from_json(
        &self,
        entity_data: serde_json::Value,
    ) -> Result<serde_json::Value, anyhow::Error> {
        let usage = Usage::new(
            entity_data["name"].as_str().unwrap_or("Usage").to_string(),
            entity_data["status"]
                .as_str()
                .unwrap_or("pending")
                .to_string(),
            entity_data["activity_id"]
                .as_str()
                .and_then(|s| Uuid::parse_str(s).ok())
                .unwrap_or_else(Uuid::new_v4),
            entity_data["usage_type"]
                .as_str()
                .unwrap_or("espace_utilise")
                .to_string(),
            entity_data["quantity"].as_f64().unwrap_or(0.0),
            entity_data["unit"].as_str().map(String::from),
            entity_data["from_activity_id"]
                .as_str()
                .and_then(|s| Uuid::parse_str(s).ok()),
            entity_data["date"].as_str().map(String::from),
        );

        self.create(usage.clone()).await?;
        Ok(serde_json::to_value(usage)?)
    }
}

#[async_trait::async_trait]
impl UsageStore for InMemoryUsageStore {
    async fn create(&self, usage: Usage) -> Result<Usage, UsageStoreError> {
        let mut g = self.inner.write().await;
        if g.iter().any(|u| u.id == usage.id) {
            return Err(UsageStoreError::Conflict(usage.id.to_string()));
        }
        g.push(usage.clone());
        Ok(usage)
    }

    async fn get(&self, id: &Uuid) -> Result<Usage, UsageStoreError> {
        let g = self.inner.read().await;
        g.iter()
            .find(|u| &u.id == id)
            .cloned()
            .ok_or_else(|| UsageStoreError::NotFound(id.to_string()))
    }

    async fn update(&self, usage: Usage) -> Result<Usage, UsageStoreError> {
        let mut g = self.inner.write().await;
        if let Some(x) = g.iter_mut().find(|u| u.id == usage.id) {
            *x = usage.clone();
            Ok(usage)
        } else {
            Err(UsageStoreError::NotFound(usage.id.to_string()))
        }
    }

    async fn delete(&self, id: &Uuid) -> Result<(), UsageStoreError> {
        let mut g = self.inner.write().await;
        let before = g.len();
        g.retain(|u| &u.id != id);
        if g.len() == before {
            return Err(UsageStoreError::NotFound(id.to_string()));
        }
        Ok(())
    }

    async fn list(&self) -> Result<Vec<Usage>, UsageStoreError> {
        Ok(self.inner.read().await.clone())
    }
}

#[cfg(feature = "dynamodb")]
use aws_sdk_dynamodb::Client as DynamoDBClient;
#[cfg(feature = "dynamodb")]
use this::storage::DynamoDBDataService;

#[cfg(feature = "dynamodb")]
#[derive(Clone)]
pub struct UsageDynamoDBStore {
    service: Arc<DynamoDBDataService<Usage>>,
}

#[cfg(feature = "dynamodb")]
impl UsageDynamoDBStore {
    pub fn new(client: DynamoDBClient, table_name: String) -> Self {
        Self {
            service: Arc::new(DynamoDBDataService::new(client, table_name)),
        }
    }
}

#[cfg(feature = "dynamodb")]
#[async_trait::async_trait]
impl EntityFetcher for UsageDynamoDBStore {
    async fn fetch_as_json(&self, entity_id: &Uuid) -> Result<serde_json::Value, anyhow::Error> {
        let usage = self
            .get(entity_id)
            .await
            .map_err(|_| anyhow::anyhow!("Usage not found: {}", entity_id))?;
        Ok(serde_json::to_value(usage)?)
    }

    async fn list_as_json(
        &self,
        limit: Option<i32>,
        offset: Option<i32>,
    ) -> Result<Vec<serde_json::Value>, anyhow::Error> {
        let all_usages = self.list().await?;
        let offset = offset.unwrap_or(0) as usize;
        let limit = limit.unwrap_or(20) as usize;

        let usages: Vec<Usage> = all_usages.into_iter().skip(offset).take(limit).collect();
        usages
            .into_iter()
            .map(|usage| serde_json::to_value(usage).map_err(Into::into))
            .collect()
    }
}

#[cfg(feature = "dynamodb")]
#[async_trait::async_trait]
impl EntityCreator for UsageDynamoDBStore {
    async fn create_from_json(
        &self,
        entity_data: serde_json::Value,
    ) -> Result<serde_json::Value, anyhow::Error> {
        let usage = Usage::new(
            entity_data["name"].as_str().unwrap_or("Usage").to_string(),
            entity_data["status"]
                .as_str()
                .unwrap_or("pending")
                .to_string(),
            entity_data["activity_id"]
                .as_str()
                .and_then(|s| Uuid::parse_str(s).ok())
                .unwrap_or_else(Uuid::new_v4),
            entity_data["usage_type"]
                .as_str()
                .unwrap_or("espace_utilise")
                .to_string(),
            entity_data["quantity"].as_f64().unwrap_or(0.0),
            entity_data["unit"].as_str().map(String::from),
            entity_data["from_activity_id"]
                .as_str()
                .and_then(|s| Uuid::parse_str(s).ok()),
            entity_data["date"].as_str().map(String::from),
        );

        self.create(usage.clone()).await?;
        Ok(serde_json::to_value(usage)?)
    }
}

#[cfg(feature = "dynamodb")]
#[async_trait::async_trait]
impl UsageStore for UsageDynamoDBStore {
    async fn create(&self, usage: Usage) -> Result<Usage, UsageStoreError> {
        self.service
            .create(usage.clone())
            .await
            .map_err(|e| UsageStoreError::Other(anyhow::anyhow!(e)))
    }

    async fn get(&self, id: &Uuid) -> Result<Usage, UsageStoreError> {
        self.service
            .get(id)
            .await
            .map_err(|e| UsageStoreError::Other(anyhow::anyhow!(e)))?
            .ok_or_else(|| UsageStoreError::NotFound(id.to_string()))
    }

    async fn update(&self, usage: Usage) -> Result<Usage, UsageStoreError> {
        self.service
            .update(&usage.id, usage.clone())
            .await
            .map_err(|e| UsageStoreError::Other(anyhow::anyhow!(e)))
    }

    async fn delete(&self, id: &Uuid) -> Result<(), UsageStoreError> {
        self.service
            .delete(id)
            .await
            .map_err(|e| UsageStoreError::Other(anyhow::anyhow!(e)))
    }

    async fn list(&self) -> Result<Vec<Usage>, UsageStoreError> {
        self.service
            .list()
            .await
            .map_err(|e| UsageStoreError::Other(anyhow::anyhow!(e)))
    }
}
