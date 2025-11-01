use async_trait::async_trait;
use std::sync::Arc;
use this::prelude::*;

use super::Order;

#[derive(Debug, thiserror::Error)]
pub enum OrderStoreError {
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
pub trait OrderStore: Send + Sync {
    async fn create(&self, order: Order) -> Result<Order, OrderStoreError>;
    async fn get(&self, id: &Uuid) -> Result<Order, OrderStoreError>;
    async fn update(&self, order: Order) -> Result<Order, OrderStoreError>;
    async fn delete(&self, id: &Uuid) -> Result<(), OrderStoreError>;
    async fn list(&self) -> Result<Vec<Order>, OrderStoreError>;
}

// ============================================================================
// InMemory Store Implementation
// ============================================================================

use tokio::sync::RwLock;

#[derive(Clone, Default)]
pub struct InMemoryOrderStore {
    inner: Arc<RwLock<Vec<Order>>>,
}

#[async_trait::async_trait]
impl EntityFetcher for InMemoryOrderStore {
    async fn fetch_as_json(&self, entity_id: &Uuid) -> Result<serde_json::Value, anyhow::Error> {
        let order = self
            .get(entity_id)
            .await
            .map_err(|_| anyhow::anyhow!("Order not found: {}", entity_id))?;
        Ok(serde_json::to_value(order)?)
    }

    async fn list_as_json(
        &self,
        limit: Option<i32>,
        offset: Option<i32>,
    ) -> Result<Vec<serde_json::Value>, anyhow::Error> {
        let all_orders = self.list().await?;
        let offset = offset.unwrap_or(0) as usize;
        let limit = limit.unwrap_or(20) as usize;

        let orders: Vec<Order> = all_orders.into_iter().skip(offset).take(limit).collect();
        orders
            .into_iter()
            .map(|order| serde_json::to_value(order).map_err(Into::into))
            .collect()
    }
}

#[async_trait::async_trait]
impl EntityCreator for InMemoryOrderStore {
    async fn create_from_json(
        &self,
        entity_data: serde_json::Value,
    ) -> Result<serde_json::Value, anyhow::Error> {
        let order = Order::new(
            entity_data["name"].as_str().unwrap_or("Order").to_string(),
            entity_data["status"]
                .as_str()
                .unwrap_or("pending")
                .to_string(),
            entity_data["number"]
                .as_str()
                .unwrap_or("ORD-000")
                .to_string(),
            entity_data["amount"].as_f64().unwrap_or(0.0),
            entity_data["customer_name"].as_str().map(String::from),
            entity_data["notes"].as_str().map(String::from),
        );

        self.create(order.clone()).await?;
        Ok(serde_json::to_value(order)?)
    }
}

#[async_trait::async_trait]
impl OrderStore for InMemoryOrderStore {
    async fn create(&self, order: Order) -> Result<Order, OrderStoreError> {
        let mut g = self.inner.write().await;
        if g.iter().any(|o| o.id == order.id) {
            return Err(OrderStoreError::Conflict(order.id.to_string()));
        }
        g.push(order.clone());
        Ok(order)
    }

    async fn get(&self, id: &Uuid) -> Result<Order, OrderStoreError> {
        let g = self.inner.read().await;
        g.iter()
            .find(|o| &o.id == id)
            .cloned()
            .ok_or_else(|| OrderStoreError::NotFound(id.to_string()))
    }

    async fn update(&self, order: Order) -> Result<Order, OrderStoreError> {
        let mut g = self.inner.write().await;
        if let Some(x) = g.iter_mut().find(|o| o.id == order.id) {
            *x = order.clone();
            Ok(order)
        } else {
            Err(OrderStoreError::NotFound(order.id.to_string()))
        }
    }

    async fn delete(&self, id: &Uuid) -> Result<(), OrderStoreError> {
        let mut g = self.inner.write().await;
        let before = g.len();
        g.retain(|o| &o.id != id);
        if g.len() == before {
            return Err(OrderStoreError::NotFound(id.to_string()));
        }
        Ok(())
    }

    async fn list(&self) -> Result<Vec<Order>, OrderStoreError> {
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
/// DynamoDB store for Order entities
#[derive(Clone)]
pub struct OrderDynamoDBStore {
    service: Arc<DynamoDBDataService<Order>>,
}

#[cfg(feature = "dynamodb")]
impl OrderDynamoDBStore {
    pub fn new(client: DynamoDBClient, table_name: String) -> Self {
        Self {
            service: Arc::new(DynamoDBDataService::new(client, table_name)),
        }
    }
}

#[cfg(feature = "dynamodb")]
#[async_trait::async_trait]
impl EntityFetcher for OrderDynamoDBStore {
    async fn fetch_as_json(&self, entity_id: &Uuid) -> Result<serde_json::Value, anyhow::Error> {
        let order = self
            .get(entity_id)
            .await
            .map_err(|_| anyhow::anyhow!("Order not found: {}", entity_id))?;
        Ok(serde_json::to_value(order)?)
    }

    async fn list_as_json(
        &self,
        limit: Option<i32>,
        offset: Option<i32>,
    ) -> Result<Vec<serde_json::Value>, anyhow::Error> {
        let all_orders = self.list().await?;
        let offset = offset.unwrap_or(0) as usize;
        let limit = limit.unwrap_or(20) as usize;

        let orders: Vec<Order> = all_orders.into_iter().skip(offset).take(limit).collect();
        orders
            .into_iter()
            .map(|order| serde_json::to_value(order).map_err(Into::into))
            .collect()
    }
}

#[cfg(feature = "dynamodb")]
#[async_trait::async_trait]
impl EntityCreator for OrderDynamoDBStore {
    async fn create_from_json(
        &self,
        entity_data: serde_json::Value,
    ) -> Result<serde_json::Value, anyhow::Error> {
        let order = Order::new(
            entity_data["name"].as_str().unwrap_or("Order").to_string(),
            entity_data["status"]
                .as_str()
                .unwrap_or("pending")
                .to_string(),
            entity_data["number"]
                .as_str()
                .unwrap_or("ORD-000")
                .to_string(),
            entity_data["amount"].as_f64().unwrap_or(0.0),
            entity_data["customer_name"].as_str().map(String::from),
            entity_data["notes"].as_str().map(String::from),
        );

        self.create(order.clone()).await?;
        Ok(serde_json::to_value(order)?)
    }
}

#[cfg(feature = "dynamodb")]
#[async_trait::async_trait]
impl OrderStore for OrderDynamoDBStore {
    async fn create(&self, order: Order) -> Result<Order, OrderStoreError> {
        self.service
            .create(order.clone())
            .await
            .map_err(|e| OrderStoreError::Other(anyhow::anyhow!(e)))
    }

    async fn get(&self, id: &Uuid) -> Result<Order, OrderStoreError> {
        self.service
            .get(id)
            .await
            .map_err(|e| OrderStoreError::Other(anyhow::anyhow!(e)))?
            .ok_or_else(|| OrderStoreError::NotFound(id.to_string()))
    }

    async fn update(&self, order: Order) -> Result<Order, OrderStoreError> {
        self.service
            .update(&order.id, order.clone())
            .await
            .map_err(|e| OrderStoreError::Other(anyhow::anyhow!(e)))
    }

    async fn delete(&self, id: &Uuid) -> Result<(), OrderStoreError> {
        self.service
            .delete(id)
            .await
            .map_err(|e| OrderStoreError::Other(anyhow::anyhow!(e)))
    }

    async fn list(&self) -> Result<Vec<Order>, OrderStoreError> {
        self.service
            .list()
            .await
            .map_err(|e| OrderStoreError::Other(anyhow::anyhow!(e)))
    }
}
