use async_trait::async_trait;
use std::sync::Arc;
use this::prelude::*;

use super::Category;

#[derive(Debug, thiserror::Error)]
pub enum CategoryStoreError {
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
pub trait CategoryStore: Send + Sync {
    async fn create(&self, category: Category) -> Result<Category, CategoryStoreError>;
    async fn get(&self, id: &Uuid) -> Result<Category, CategoryStoreError>;
    async fn update(&self, category: Category) -> Result<Category, CategoryStoreError>;
    async fn delete(&self, id: &Uuid) -> Result<(), CategoryStoreError>;
    async fn list(&self) -> Result<Vec<Category>, CategoryStoreError>;
}

// ============================================================================
// InMemory Store Implementation
// ============================================================================

use tokio::sync::RwLock;

#[derive(Clone, Default)]
pub struct InMemoryCategoryStore {
    inner: Arc<RwLock<Vec<Category>>>,
}

#[async_trait::async_trait]
impl EntityFetcher for InMemoryCategoryStore {
    async fn fetch_as_json(&self, entity_id: &Uuid) -> Result<serde_json::Value, anyhow::Error> {
        let category = self
            .get(entity_id)
            .await
            .map_err(|_| anyhow::anyhow!("Category not found: {}", entity_id))?;
        Ok(serde_json::to_value(category)?)
    }

    async fn list_as_json(
        &self,
        limit: Option<i32>,
        offset: Option<i32>,
    ) -> Result<Vec<serde_json::Value>, anyhow::Error> {
        let all_categories = self.list().await?;
        let offset = offset.unwrap_or(0) as usize;
        let limit = limit.unwrap_or(20) as usize;

        let categories: Vec<Category> = all_categories.into_iter().skip(offset).take(limit).collect();
        categories
            .into_iter()
            .map(|category| serde_json::to_value(category).map_err(Into::into))
            .collect()
    }
}

#[async_trait::async_trait]
impl EntityCreator for InMemoryCategoryStore {
    async fn create_from_json(
        &self,
        entity_data: serde_json::Value,
    ) -> Result<serde_json::Value, anyhow::Error> {
        let category = Category::new(
            entity_data["name"]
                .as_str()
                .unwrap_or("Category")
                .to_string(),
            entity_data["status"]
                .as_str()
                .unwrap_or("active")
                .to_string(),
            entity_data["slug"]
                .as_str()
                .unwrap_or("category")
                .to_string(),
            entity_data["description"].as_str().map(String::from),
        );

        self.create(category.clone()).await?;
        Ok(serde_json::to_value(category)?)
    }
}

#[async_trait::async_trait]
impl CategoryStore for InMemoryCategoryStore {
    async fn create(&self, category: Category) -> Result<Category, CategoryStoreError> {
        let mut g = self.inner.write().await;
        if g.iter().any(|c| c.id == category.id) {
            return Err(CategoryStoreError::Conflict(category.id.to_string()));
        }
        g.push(category.clone());
        Ok(category)
    }

    async fn get(&self, id: &Uuid) -> Result<Category, CategoryStoreError> {
        let g = self.inner.read().await;
        g.iter()
            .find(|c| &c.id == id)
            .cloned()
            .ok_or_else(|| CategoryStoreError::NotFound(id.to_string()))
    }

    async fn update(&self, category: Category) -> Result<Category, CategoryStoreError> {
        let mut g = self.inner.write().await;
        if let Some(x) = g.iter_mut().find(|c| c.id == category.id) {
            *x = category.clone();
            Ok(category)
        } else {
            Err(CategoryStoreError::NotFound(category.id.to_string()))
        }
    }

    async fn delete(&self, id: &Uuid) -> Result<(), CategoryStoreError> {
        let mut g = self.inner.write().await;
        let before = g.len();
        g.retain(|c| &c.id != id);
        if g.len() == before {
            return Err(CategoryStoreError::NotFound(id.to_string()));
        }
        Ok(())
    }

    async fn list(&self) -> Result<Vec<Category>, CategoryStoreError> {
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
/// DynamoDB store for Category entities
#[derive(Clone)]
pub struct CategoryDynamoDBStore {
    service: Arc<DynamoDBDataService<Category>>,
}

#[cfg(feature = "dynamodb")]
impl CategoryDynamoDBStore {
    pub fn new(client: DynamoDBClient, table_name: String) -> Self {
        Self {
            service: Arc::new(DynamoDBDataService::new(client, table_name)),
        }
    }
}

#[cfg(feature = "dynamodb")]
#[async_trait::async_trait]
impl EntityFetcher for CategoryDynamoDBStore {
    async fn fetch_as_json(&self, entity_id: &Uuid) -> Result<serde_json::Value, anyhow::Error> {
        let category = self
            .get(entity_id)
            .await
            .map_err(|_| anyhow::anyhow!("Category not found: {}", entity_id))?;
        Ok(serde_json::to_value(category)?)
    }

    async fn list_as_json(
        &self,
        limit: Option<i32>,
        offset: Option<i32>,
    ) -> Result<Vec<serde_json::Value>, anyhow::Error> {
        let all_categories = self.list().await?;
        let offset = offset.unwrap_or(0) as usize;
        let limit = limit.unwrap_or(20) as usize;

        let categories: Vec<Category> = all_categories.into_iter().skip(offset).take(limit).collect();
        categories
            .into_iter()
            .map(|category| serde_json::to_value(category).map_err(Into::into))
            .collect()
    }
}

#[cfg(feature = "dynamodb")]
#[async_trait::async_trait]
impl EntityCreator for CategoryDynamoDBStore {
    async fn create_from_json(
        &self,
        entity_data: serde_json::Value,
    ) -> Result<serde_json::Value, anyhow::Error> {
        let category = Category::new(
            entity_data["name"]
                .as_str()
                .unwrap_or("Category")
                .to_string(),
            entity_data["status"]
                .as_str()
                .unwrap_or("active")
                .to_string(),
            entity_data["slug"]
                .as_str()
                .unwrap_or("category")
                .to_string(),
            entity_data["description"].as_str().map(String::from),
        );

        self.create(category.clone()).await?;
        Ok(serde_json::to_value(category)?)
    }
}

#[cfg(feature = "dynamodb")]
#[async_trait::async_trait]
impl CategoryStore for CategoryDynamoDBStore {
    async fn create(&self, category: Category) -> Result<Category, CategoryStoreError> {
        self.service
            .create(category.clone())
            .await
            .map_err(|e| CategoryStoreError::Other(anyhow::anyhow!(e)))
    }

    async fn get(&self, id: &Uuid) -> Result<Category, CategoryStoreError> {
        self.service
            .get(id)
            .await
            .map_err(|e| CategoryStoreError::Other(anyhow::anyhow!(e)))?
            .ok_or_else(|| CategoryStoreError::NotFound(id.to_string()))
    }

    async fn update(&self, category: Category) -> Result<Category, CategoryStoreError> {
        self.service
            .update(&category.id, category.clone())
            .await
            .map_err(|e| CategoryStoreError::Other(anyhow::anyhow!(e)))
    }

    async fn delete(&self, id: &Uuid) -> Result<(), CategoryStoreError> {
        self.service
            .delete(id)
            .await
            .map_err(|e| CategoryStoreError::Other(anyhow::anyhow!(e)))
    }

    async fn list(&self) -> Result<Vec<Category>, CategoryStoreError> {
        self.service
            .list()
            .await
            .map_err(|e| CategoryStoreError::Other(anyhow::anyhow!(e)))
    }
}

