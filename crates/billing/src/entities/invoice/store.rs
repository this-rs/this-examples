use async_trait::async_trait;
use this::prelude::*;

use super::Invoice;

#[derive(Debug, thiserror::Error)]
pub enum InvoiceStoreError {
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
pub trait InvoiceStore: Send + Sync {
    async fn create(&self, invoice: Invoice) -> Result<Invoice, InvoiceStoreError>;
    async fn get(&self, id: &Uuid) -> Result<Invoice, InvoiceStoreError>;
    async fn update(&self, invoice: Invoice) -> Result<Invoice, InvoiceStoreError>;
    async fn delete(&self, id: &Uuid) -> Result<(), InvoiceStoreError>;
    async fn list(&self) -> Result<Vec<Invoice>, InvoiceStoreError>;
}

