use async_trait::async_trait;
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

