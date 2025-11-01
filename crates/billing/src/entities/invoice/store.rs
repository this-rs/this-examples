use async_trait::async_trait;
use std::sync::Arc;
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

// ============================================================================
// InMemory Store Implementation
// ============================================================================

use tokio::sync::RwLock;

#[derive(Clone, Default)]
pub struct InMemoryInvoiceStore {
    inner: Arc<RwLock<Vec<Invoice>>>,
}

#[async_trait::async_trait]
impl EntityFetcher for InMemoryInvoiceStore {
    async fn fetch_as_json(&self, entity_id: &Uuid) -> Result<serde_json::Value, anyhow::Error> {
        let invoice = self
            .get(entity_id)
            .await
            .map_err(|_| anyhow::anyhow!("Invoice not found: {}", entity_id))?;
        Ok(serde_json::to_value(invoice)?)
    }

    async fn list_as_json(
        &self,
        limit: Option<i32>,
        offset: Option<i32>,
    ) -> Result<Vec<serde_json::Value>, anyhow::Error> {
        let all_invoices = self.list().await?;
        let offset = offset.unwrap_or(0) as usize;
        let limit = limit.unwrap_or(20) as usize;

        let invoices: Vec<Invoice> = all_invoices.into_iter().skip(offset).take(limit).collect();
        invoices
            .into_iter()
            .map(|invoice| serde_json::to_value(invoice).map_err(Into::into))
            .collect()
    }
}

#[async_trait::async_trait]
impl EntityCreator for InMemoryInvoiceStore {
    async fn create_from_json(
        &self,
        entity_data: serde_json::Value,
    ) -> Result<serde_json::Value, anyhow::Error> {
        let invoice = Invoice::new(
            entity_data["name"]
                .as_str()
                .unwrap_or("Invoice")
                .to_string(),
            entity_data["status"]
                .as_str()
                .unwrap_or("draft")
                .to_string(),
            entity_data["number"]
                .as_str()
                .unwrap_or("INV-000")
                .to_string(),
            entity_data["amount"].as_f64().unwrap_or(0.0),
            entity_data["due_date"].as_str().map(String::from),
            entity_data["paid_at"].as_str().map(String::from),
        );

        self.create(invoice.clone()).await?;
        Ok(serde_json::to_value(invoice)?)
    }
}

#[async_trait::async_trait]
impl InvoiceStore for InMemoryInvoiceStore {
    async fn create(&self, invoice: Invoice) -> Result<Invoice, InvoiceStoreError> {
        let mut g = self.inner.write().await;
        if g.iter().any(|o| o.id == invoice.id) {
            return Err(InvoiceStoreError::Conflict(invoice.id.to_string()));
        }
        g.push(invoice.clone());
        Ok(invoice)
    }

    async fn get(&self, id: &Uuid) -> Result<Invoice, InvoiceStoreError> {
        let g = self.inner.read().await;
        g.iter()
            .find(|o| &o.id == id)
            .cloned()
            .ok_or_else(|| InvoiceStoreError::NotFound(id.to_string()))
    }

    async fn update(&self, invoice: Invoice) -> Result<Invoice, InvoiceStoreError> {
        let mut g = self.inner.write().await;
        if let Some(x) = g.iter_mut().find(|o| o.id == invoice.id) {
            *x = invoice.clone();
            Ok(invoice)
        } else {
            Err(InvoiceStoreError::NotFound(invoice.id.to_string()))
        }
    }

    async fn delete(&self, id: &Uuid) -> Result<(), InvoiceStoreError> {
        let mut g = self.inner.write().await;
        let before = g.len();
        g.retain(|o| &o.id != id);
        if g.len() == before {
            return Err(InvoiceStoreError::NotFound(id.to_string()));
        }
        Ok(())
    }

    async fn list(&self) -> Result<Vec<Invoice>, InvoiceStoreError> {
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
/// DynamoDB store for Invoice entities
#[derive(Clone)]
pub struct InvoiceDynamoDBStore {
    service: Arc<DynamoDBDataService<Invoice>>,
}

#[cfg(feature = "dynamodb")]
impl InvoiceDynamoDBStore {
    pub fn new(client: DynamoDBClient, table_name: String) -> Self {
        Self {
            service: Arc::new(DynamoDBDataService::new(client, table_name)),
        }
    }
}

#[cfg(feature = "dynamodb")]
#[async_trait::async_trait]
impl EntityFetcher for InvoiceDynamoDBStore {
    async fn fetch_as_json(&self, entity_id: &Uuid) -> Result<serde_json::Value, anyhow::Error> {
        let invoice = self
            .get(entity_id)
            .await
            .map_err(|_| anyhow::anyhow!("Invoice not found: {}", entity_id))?;
        Ok(serde_json::to_value(invoice)?)
    }

    async fn list_as_json(
        &self,
        limit: Option<i32>,
        offset: Option<i32>,
    ) -> Result<Vec<serde_json::Value>, anyhow::Error> {
        let all_invoices = self.list().await?;
        let offset = offset.unwrap_or(0) as usize;
        let limit = limit.unwrap_or(20) as usize;

        let invoices: Vec<Invoice> = all_invoices.into_iter().skip(offset).take(limit).collect();
        invoices
            .into_iter()
            .map(|invoice| serde_json::to_value(invoice).map_err(Into::into))
            .collect()
    }
}

#[cfg(feature = "dynamodb")]
#[async_trait::async_trait]
impl EntityCreator for InvoiceDynamoDBStore {
    async fn create_from_json(
        &self,
        entity_data: serde_json::Value,
    ) -> Result<serde_json::Value, anyhow::Error> {
        let invoice = Invoice::new(
            entity_data["name"]
                .as_str()
                .unwrap_or("Invoice")
                .to_string(),
            entity_data["status"]
                .as_str()
                .unwrap_or("draft")
                .to_string(),
            entity_data["number"]
                .as_str()
                .unwrap_or("INV-000")
                .to_string(),
            entity_data["amount"].as_f64().unwrap_or(0.0),
            entity_data["due_date"].as_str().map(String::from),
            entity_data["paid_at"].as_str().map(String::from),
        );

        self.create(invoice.clone()).await?;
        Ok(serde_json::to_value(invoice)?)
    }
}

#[cfg(feature = "dynamodb")]
#[async_trait::async_trait]
impl InvoiceStore for InvoiceDynamoDBStore {
    async fn create(&self, invoice: Invoice) -> Result<Invoice, InvoiceStoreError> {
        self.service
            .create(invoice.clone())
            .await
            .map_err(|e| InvoiceStoreError::Other(anyhow::anyhow!(e)))
    }

    async fn get(&self, id: &Uuid) -> Result<Invoice, InvoiceStoreError> {
        self.service
            .get(id)
            .await
            .map_err(|e| InvoiceStoreError::Other(anyhow::anyhow!(e)))?
            .ok_or_else(|| InvoiceStoreError::NotFound(id.to_string()))
    }

    async fn update(&self, invoice: Invoice) -> Result<Invoice, InvoiceStoreError> {
        self.service
            .update(&invoice.id, invoice.clone())
            .await
            .map_err(|e| InvoiceStoreError::Other(anyhow::anyhow!(e)))
    }

    async fn delete(&self, id: &Uuid) -> Result<(), InvoiceStoreError> {
        self.service
            .delete(id)
            .await
            .map_err(|e| InvoiceStoreError::Other(anyhow::anyhow!(e)))
    }

    async fn list(&self) -> Result<Vec<Invoice>, InvoiceStoreError> {
        self.service
            .list()
            .await
            .map_err(|e| InvoiceStoreError::Other(anyhow::anyhow!(e)))
    }
}
