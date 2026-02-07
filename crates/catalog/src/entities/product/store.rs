use async_trait::async_trait;
use std::sync::Arc;
use this::prelude::*;

use super::Product;

#[derive(Debug, thiserror::Error)]
pub enum ProductStoreError {
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
pub trait ProductStore: Send + Sync {
    async fn create(&self, product: Product) -> Result<Product, ProductStoreError>;
    async fn get(&self, id: &Uuid) -> Result<Product, ProductStoreError>;
    async fn update(&self, product: Product) -> Result<Product, ProductStoreError>;
    async fn delete(&self, id: &Uuid) -> Result<(), ProductStoreError>;
    async fn list(&self) -> Result<Vec<Product>, ProductStoreError>;
}

// ============================================================================
// InMemory Store Implementation
// ============================================================================

use tokio::sync::RwLock;

#[derive(Clone, Default)]
pub struct InMemoryProductStore {
    inner: Arc<RwLock<Vec<Product>>>,
}

#[async_trait::async_trait]
impl EntityFetcher for InMemoryProductStore {
    async fn fetch_as_json(&self, entity_id: &Uuid) -> Result<serde_json::Value, anyhow::Error> {
        let product = self
            .get(entity_id)
            .await
            .map_err(|_| anyhow::anyhow!("Product not found: {}", entity_id))?;
        Ok(serde_json::to_value(product)?)
    }

    async fn list_as_json(
        &self,
        limit: Option<i32>,
        offset: Option<i32>,
    ) -> Result<Vec<serde_json::Value>, anyhow::Error> {
        let all_products = self.list().await?;
        let offset = offset.unwrap_or(0) as usize;
        let limit = limit.unwrap_or(20) as usize;

        let products: Vec<Product> = all_products.into_iter().skip(offset).take(limit).collect();
        products
            .into_iter()
            .map(|product| serde_json::to_value(product).map_err(Into::into))
            .collect()
    }
}

#[async_trait::async_trait]
impl EntityCreator for InMemoryProductStore {
    async fn create_from_json(
        &self,
        entity_data: serde_json::Value,
    ) -> Result<serde_json::Value, anyhow::Error> {
        let product = Product::new(
            entity_data["name"]
                .as_str()
                .unwrap_or("Product")
                .to_string(),
            entity_data["status"]
                .as_str()
                .unwrap_or("active")
                .to_string(),
            entity_data["sku"].as_str().unwrap_or("SKU-000").to_string(),
            entity_data["price"].as_f64().unwrap_or(0.0),
            entity_data["stock_quantity"].as_i64().unwrap_or(0) as i32,
            entity_data["description"].as_str().map(String::from),
        );

        self.create(product.clone()).await?;
        Ok(serde_json::to_value(product)?)
    }
}

#[async_trait::async_trait]
impl ProductStore for InMemoryProductStore {
    async fn create(&self, product: Product) -> Result<Product, ProductStoreError> {
        let mut g = self.inner.write().await;
        if g.iter().any(|p| p.id == product.id) {
            return Err(ProductStoreError::Conflict(product.id.to_string()));
        }
        g.push(product.clone());
        Ok(product)
    }

    async fn get(&self, id: &Uuid) -> Result<Product, ProductStoreError> {
        let g = self.inner.read().await;
        g.iter()
            .find(|p| &p.id == id)
            .cloned()
            .ok_or_else(|| ProductStoreError::NotFound(id.to_string()))
    }

    async fn update(&self, product: Product) -> Result<Product, ProductStoreError> {
        let mut g = self.inner.write().await;
        if let Some(x) = g.iter_mut().find(|p| p.id == product.id) {
            *x = product.clone();
            Ok(product)
        } else {
            Err(ProductStoreError::NotFound(product.id.to_string()))
        }
    }

    async fn delete(&self, id: &Uuid) -> Result<(), ProductStoreError> {
        let mut g = self.inner.write().await;
        let before = g.len();
        g.retain(|p| &p.id != id);
        if g.len() == before {
            return Err(ProductStoreError::NotFound(id.to_string()));
        }
        Ok(())
    }

    async fn list(&self) -> Result<Vec<Product>, ProductStoreError> {
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
/// DynamoDB store for Product entities
#[derive(Clone)]
pub struct ProductDynamoDBStore {
    service: Arc<DynamoDBDataService<Product>>,
}

#[cfg(feature = "dynamodb")]
impl ProductDynamoDBStore {
    pub fn new(client: DynamoDBClient, table_name: String) -> Self {
        Self {
            service: Arc::new(DynamoDBDataService::new(client, table_name)),
        }
    }
}

#[cfg(feature = "dynamodb")]
#[async_trait::async_trait]
impl EntityFetcher for ProductDynamoDBStore {
    async fn fetch_as_json(&self, entity_id: &Uuid) -> Result<serde_json::Value, anyhow::Error> {
        let product = self
            .get(entity_id)
            .await
            .map_err(|_| anyhow::anyhow!("Product not found: {}", entity_id))?;
        Ok(serde_json::to_value(product)?)
    }

    async fn list_as_json(
        &self,
        limit: Option<i32>,
        offset: Option<i32>,
    ) -> Result<Vec<serde_json::Value>, anyhow::Error> {
        let all_products = self.list().await?;
        let offset = offset.unwrap_or(0) as usize;
        let limit = limit.unwrap_or(20) as usize;

        let products: Vec<Product> = all_products.into_iter().skip(offset).take(limit).collect();
        products
            .into_iter()
            .map(|product| serde_json::to_value(product).map_err(Into::into))
            .collect()
    }
}

#[cfg(feature = "dynamodb")]
#[async_trait::async_trait]
impl EntityCreator for ProductDynamoDBStore {
    async fn create_from_json(
        &self,
        entity_data: serde_json::Value,
    ) -> Result<serde_json::Value, anyhow::Error> {
        let product = Product::new(
            entity_data["name"]
                .as_str()
                .unwrap_or("Product")
                .to_string(),
            entity_data["status"]
                .as_str()
                .unwrap_or("active")
                .to_string(),
            entity_data["sku"].as_str().unwrap_or("SKU-000").to_string(),
            entity_data["price"].as_f64().unwrap_or(0.0),
            entity_data["stock_quantity"].as_i64().unwrap_or(0) as i32,
            entity_data["description"].as_str().map(String::from),
        );

        self.create(product.clone()).await?;
        Ok(serde_json::to_value(product)?)
    }
}

#[cfg(feature = "dynamodb")]
#[async_trait::async_trait]
impl ProductStore for ProductDynamoDBStore {
    async fn create(&self, product: Product) -> Result<Product, ProductStoreError> {
        self.service
            .create(product.clone())
            .await
            .map_err(|e| ProductStoreError::Other(anyhow::anyhow!(e)))
    }

    async fn get(&self, id: &Uuid) -> Result<Product, ProductStoreError> {
        self.service
            .get(id)
            .await
            .map_err(|e| ProductStoreError::Other(anyhow::anyhow!(e)))?
            .ok_or_else(|| ProductStoreError::NotFound(id.to_string()))
    }

    async fn update(&self, product: Product) -> Result<Product, ProductStoreError> {
        self.service
            .update(&product.id, product.clone())
            .await
            .map_err(|e| ProductStoreError::Other(anyhow::anyhow!(e)))
    }

    async fn delete(&self, id: &Uuid) -> Result<(), ProductStoreError> {
        self.service
            .delete(id)
            .await
            .map_err(|e| ProductStoreError::Other(anyhow::anyhow!(e)))
    }

    async fn list(&self) -> Result<Vec<Product>, ProductStoreError> {
        self.service
            .list()
            .await
            .map_err(|e| ProductStoreError::Other(anyhow::anyhow!(e)))
    }
}





