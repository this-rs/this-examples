use async_trait::async_trait;
use std::sync::Arc;
use this::prelude::*;

use super::Tag;

#[derive(Debug, thiserror::Error)]
pub enum TagStoreError {
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
pub trait TagStore: Send + Sync {
    async fn create(&self, tag: Tag) -> Result<Tag, TagStoreError>;
    async fn get(&self, id: &Uuid) -> Result<Tag, TagStoreError>;
    async fn update(&self, tag: Tag) -> Result<Tag, TagStoreError>;
    async fn delete(&self, id: &Uuid) -> Result<(), TagStoreError>;
    async fn list(&self) -> Result<Vec<Tag>, TagStoreError>;
}

// ============================================================================
// InMemory Store Implementation
// ============================================================================

use tokio::sync::RwLock;

#[derive(Clone, Default)]
pub struct InMemoryTagStore {
    inner: Arc<RwLock<Vec<Tag>>>,
}

#[async_trait::async_trait]
impl EntityFetcher for InMemoryTagStore {
    async fn fetch_as_json(&self, entity_id: &Uuid) -> Result<serde_json::Value, anyhow::Error> {
        let tag = self
            .get(entity_id)
            .await
            .map_err(|_| anyhow::anyhow!("Tag not found: {}", entity_id))?;
        Ok(serde_json::to_value(tag)?)
    }

    async fn list_as_json(
        &self,
        limit: Option<i32>,
        offset: Option<i32>,
    ) -> Result<Vec<serde_json::Value>, anyhow::Error> {
        let all_tags = self.list().await?;
        let offset = offset.unwrap_or(0) as usize;
        let limit = limit.unwrap_or(20) as usize;

        let tags: Vec<Tag> = all_tags.into_iter().skip(offset).take(limit).collect();
        tags
            .into_iter()
            .map(|tag| serde_json::to_value(tag).map_err(Into::into))
            .collect()
    }
}

#[async_trait::async_trait]
impl EntityCreator for InMemoryTagStore {
    async fn create_from_json(
        &self,
        entity_data: serde_json::Value,
    ) -> Result<serde_json::Value, anyhow::Error> {
        let tag = Tag::new(
            entity_data["name"]
                .as_str()
                .unwrap_or("Tag")
                .to_string(),
            entity_data["status"]
                .as_str()
                .unwrap_or("active")
                .to_string(),
            entity_data["color"].as_str().map(String::from),
            entity_data["description"].as_str().map(String::from),
        );

        self.create(tag.clone()).await?;
        Ok(serde_json::to_value(tag)?)
    }
}

#[async_trait::async_trait]
impl TagStore for InMemoryTagStore {
    async fn create(&self, tag: Tag) -> Result<Tag, TagStoreError> {
        let mut g = self.inner.write().await;
        if g.iter().any(|t| t.id == tag.id) {
            return Err(TagStoreError::Conflict(tag.id.to_string()));
        }
        g.push(tag.clone());
        Ok(tag)
    }

    async fn get(&self, id: &Uuid) -> Result<Tag, TagStoreError> {
        let g = self.inner.read().await;
        g.iter()
            .find(|t| &t.id == id)
            .cloned()
            .ok_or_else(|| TagStoreError::NotFound(id.to_string()))
    }

    async fn update(&self, tag: Tag) -> Result<Tag, TagStoreError> {
        let mut g = self.inner.write().await;
        if let Some(x) = g.iter_mut().find(|t| t.id == tag.id) {
            *x = tag.clone();
            Ok(tag)
        } else {
            Err(TagStoreError::NotFound(tag.id.to_string()))
        }
    }

    async fn delete(&self, id: &Uuid) -> Result<(), TagStoreError> {
        let mut g = self.inner.write().await;
        let before = g.len();
        g.retain(|t| &t.id != id);
        if g.len() == before {
            return Err(TagStoreError::NotFound(id.to_string()));
        }
        Ok(())
    }

    async fn list(&self) -> Result<Vec<Tag>, TagStoreError> {
        Ok(self.inner.read().await.clone())
    }
}

// ============================================================================
// DynamoDB Store Implementation
// ============================================================================

#[cfg(feature = "dynamodb")]
use aws_sdk_dynamodb::Client as DynamoDBClient;
#[cfg(feature = "dynamodb")]
use this::storage::DynamoDBDataService;

#[cfg(feature = "dynamodb")]
/// DynamoDB store for Tag entities
#[derive(Clone)]
pub struct TagDynamoDBStore {
    service: Arc<DynamoDBDataService<Tag>>,
}

#[cfg(feature = "dynamodb")]
impl TagDynamoDBStore {
    pub fn new(client: DynamoDBClient, table_name: String) -> Self {
        Self {
            service: Arc::new(DynamoDBDataService::new(client, table_name)),
        }
    }
}

#[cfg(feature = "dynamodb")]
#[async_trait::async_trait]
impl EntityFetcher for TagDynamoDBStore {
    async fn fetch_as_json(&self, entity_id: &Uuid) -> Result<serde_json::Value, anyhow::Error> {
        let tag = self
            .get(entity_id)
            .await
            .map_err(|_| anyhow::anyhow!("Tag not found: {}", entity_id))?;
        Ok(serde_json::to_value(tag)?)
    }

    async fn list_as_json(
        &self,
        limit: Option<i32>,
        offset: Option<i32>,
    ) -> Result<Vec<serde_json::Value>, anyhow::Error> {
        let all_tags = self.list().await?;
        let offset = offset.unwrap_or(0) as usize;
        let limit = limit.unwrap_or(20) as usize;

        let tags: Vec<Tag> = all_tags.into_iter().skip(offset).take(limit).collect();
        tags
            .into_iter()
            .map(|tag| serde_json::to_value(tag).map_err(Into::into))
            .collect()
    }
}

#[cfg(feature = "dynamodb")]
#[async_trait::async_trait]
impl EntityCreator for TagDynamoDBStore {
    async fn create_from_json(
        &self,
        entity_data: serde_json::Value,
    ) -> Result<serde_json::Value, anyhow::Error> {
        let tag = Tag::new(
            entity_data["name"]
                .as_str()
                .unwrap_or("Tag")
                .to_string(),
            entity_data["status"]
                .as_str()
                .unwrap_or("active")
                .to_string(),
            entity_data["color"].as_str().map(String::from),
            entity_data["description"].as_str().map(String::from),
        );

        self.create(tag.clone()).await?;
        Ok(serde_json::to_value(tag)?)
    }
}

#[cfg(feature = "dynamodb")]
#[async_trait::async_trait]
impl TagStore for TagDynamoDBStore {
    async fn create(&self, tag: Tag) -> Result<Tag, TagStoreError> {
        self.service
            .create(tag.clone())
            .await
            .map_err(|e| TagStoreError::Other(anyhow::anyhow!(e)))
    }

    async fn get(&self, id: &Uuid) -> Result<Tag, TagStoreError> {
        self.service
            .get(id)
            .await
            .map_err(|e| TagStoreError::Other(anyhow::anyhow!(e)))?
            .ok_or_else(|| TagStoreError::NotFound(id.to_string()))
    }

    async fn update(&self, tag: Tag) -> Result<Tag, TagStoreError> {
        self.service
            .update(&tag.id, tag.clone())
            .await
            .map_err(|e| TagStoreError::Other(anyhow::anyhow!(e)))
    }

    async fn delete(&self, id: &Uuid) -> Result<(), TagStoreError> {
        self.service
            .delete(id)
            .await
            .map_err(|e| TagStoreError::Other(anyhow::anyhow!(e)))
    }

    async fn list(&self) -> Result<Vec<Tag>, TagStoreError> {
        self.service
            .list()
            .await
            .map_err(|e| TagStoreError::Other(anyhow::anyhow!(e)))
    }
}

