use async_trait::async_trait;
use std::sync::Arc;
use this::prelude::*;
use tokio::sync::RwLock;

use super::Shipment;

#[derive(Debug, thiserror::Error)]
pub enum ShipmentStoreError {
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
pub trait ShipmentStore: Send + Sync {
    async fn create(&self, shipment: Shipment) -> Result<Shipment, ShipmentStoreError>;
    async fn get(&self, id: &Uuid) -> Result<Shipment, ShipmentStoreError>;
    async fn update(&self, shipment: Shipment) -> Result<Shipment, ShipmentStoreError>;
    async fn delete(&self, id: &Uuid) -> Result<(), ShipmentStoreError>;
    async fn list(&self) -> Result<Vec<Shipment>, ShipmentStoreError>;
}

// ============================================================================
// InMemory Store Implementation
// ============================================================================

#[derive(Clone, Default)]
pub struct InMemoryShipmentStore {
    inner: Arc<RwLock<Vec<Shipment>>>,
}

#[async_trait]
impl EntityFetcher for InMemoryShipmentStore {
    async fn fetch_as_json(&self, entity_id: &Uuid) -> Result<serde_json::Value, anyhow::Error> {
        let shipment = self
            .get(entity_id)
            .await
            .map_err(|_| anyhow::anyhow!("Shipment not found: {}", entity_id))?;
        Ok(serde_json::to_value(shipment)?)
    }

    async fn list_as_json(
        &self,
        limit: Option<i32>,
        offset: Option<i32>,
    ) -> Result<Vec<serde_json::Value>, anyhow::Error> {
        let all = self.list().await?;
        let offset = offset.unwrap_or(0) as usize;
        let limit = limit.unwrap_or(20) as usize;

        all.into_iter()
            .skip(offset)
            .take(limit)
            .map(|s| serde_json::to_value(s).map_err(Into::into))
            .collect()
    }
}

#[async_trait]
impl EntityCreator for InMemoryShipmentStore {
    async fn create_from_json(
        &self,
        entity_data: serde_json::Value,
    ) -> Result<serde_json::Value, anyhow::Error> {
        let shipment = Shipment::new(
            entity_data["name"]
                .as_str()
                .unwrap_or("Shipment")
                .to_string(),
            entity_data["status"]
                .as_str()
                .unwrap_or("preparing")
                .to_string(),
            entity_data["number"].as_str().unwrap_or("").to_string(),
            entity_data["carrier"].as_str().unwrap_or("").to_string(),
            entity_data["tracking"].as_str().map(String::from),
        );

        self.create(shipment.clone()).await?;
        Ok(serde_json::to_value(shipment)?)
    }
}

#[async_trait]
impl ShipmentStore for InMemoryShipmentStore {
    async fn create(&self, shipment: Shipment) -> Result<Shipment, ShipmentStoreError> {
        let mut g = self.inner.write().await;
        if g.iter().any(|s| s.id == shipment.id) {
            return Err(ShipmentStoreError::Conflict(shipment.id.to_string()));
        }
        g.push(shipment.clone());
        Ok(shipment)
    }

    async fn get(&self, id: &Uuid) -> Result<Shipment, ShipmentStoreError> {
        let g = self.inner.read().await;
        g.iter()
            .find(|s| &s.id == id)
            .cloned()
            .ok_or_else(|| ShipmentStoreError::NotFound(id.to_string()))
    }

    async fn update(&self, shipment: Shipment) -> Result<Shipment, ShipmentStoreError> {
        let mut g = self.inner.write().await;
        if let Some(existing) = g.iter_mut().find(|s| s.id == shipment.id) {
            *existing = shipment.clone();
            Ok(shipment)
        } else {
            Err(ShipmentStoreError::NotFound(shipment.id.to_string()))
        }
    }

    async fn delete(&self, id: &Uuid) -> Result<(), ShipmentStoreError> {
        let mut g = self.inner.write().await;
        let before = g.len();
        g.retain(|s| &s.id != id);
        if g.len() == before {
            return Err(ShipmentStoreError::NotFound(id.to_string()));
        }
        Ok(())
    }

    async fn list(&self) -> Result<Vec<Shipment>, ShipmentStoreError> {
        Ok(self.inner.read().await.clone())
    }
}
