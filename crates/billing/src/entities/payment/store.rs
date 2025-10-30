use async_trait::async_trait;
use this::prelude::*;

use super::Payment;

#[derive(Debug, thiserror::Error)]
pub enum PaymentStoreError {
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
pub trait PaymentStore: Send + Sync {
    async fn create(&self, payment: Payment) -> Result<Payment, PaymentStoreError>;
    async fn get(&self, id: &Uuid) -> Result<Payment, PaymentStoreError>;
    async fn update(&self, payment: Payment) -> Result<Payment, PaymentStoreError>;
    async fn delete(&self, id: &Uuid) -> Result<(), PaymentStoreError>;
    async fn list(&self) -> Result<Vec<Payment>, PaymentStoreError>;
}

