use async_trait::async_trait;
use std::sync::Arc;
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

// ============================================================================
// InMemory Store Implementation
// ============================================================================

use tokio::sync::RwLock;

#[derive(Clone, Default)]
pub struct InMemoryPaymentStore {
    inner: Arc<RwLock<Vec<Payment>>>,
}

#[async_trait::async_trait]
impl EntityFetcher for InMemoryPaymentStore {
    async fn fetch_as_json(&self, entity_id: &Uuid) -> Result<serde_json::Value, anyhow::Error> {
        let payment = self
            .get(entity_id)
            .await
            .map_err(|_| anyhow::anyhow!("Payment not found: {}", entity_id))?;
        Ok(serde_json::to_value(payment)?)
    }

    async fn list_as_json(
        &self,
        limit: Option<i32>,
        offset: Option<i32>,
    ) -> Result<Vec<serde_json::Value>, anyhow::Error> {
        let all_payments = self.list().await?;
        let offset = offset.unwrap_or(0) as usize;
        let limit = limit.unwrap_or(20) as usize;

        let payments: Vec<Payment> = all_payments.into_iter().skip(offset).take(limit).collect();
        payments
            .into_iter()
            .map(|payment| serde_json::to_value(payment).map_err(Into::into))
            .collect()
    }
}

#[async_trait::async_trait]
impl EntityCreator for InMemoryPaymentStore {
    async fn create_from_json(
        &self,
        entity_data: serde_json::Value,
    ) -> Result<serde_json::Value, anyhow::Error> {
        let payment = Payment::new(
            entity_data["name"]
                .as_str()
                .unwrap_or("Payment")
                .to_string(),
            entity_data["status"]
                .as_str()
                .unwrap_or("pending")
                .to_string(),
            entity_data["number"]
                .as_str()
                .unwrap_or("PAY-000")
                .to_string(),
            entity_data["amount"].as_f64().unwrap_or(0.0),
            entity_data["method"]
                .as_str()
                .unwrap_or("credit_card")
                .to_string(),
            entity_data["transaction_id"].as_str().map(String::from),
        );

        self.create(payment.clone()).await?;
        Ok(serde_json::to_value(payment)?)
    }
}

#[async_trait::async_trait]
impl PaymentStore for InMemoryPaymentStore {
    async fn create(&self, payment: Payment) -> Result<Payment, PaymentStoreError> {
        let mut g = self.inner.write().await;
        if g.iter().any(|o| o.id == payment.id) {
            return Err(PaymentStoreError::Conflict(payment.id.to_string()));
        }
        g.push(payment.clone());
        Ok(payment)
    }

    async fn get(&self, id: &Uuid) -> Result<Payment, PaymentStoreError> {
        let g = self.inner.read().await;
        g.iter()
            .find(|o| &o.id == id)
            .cloned()
            .ok_or_else(|| PaymentStoreError::NotFound(id.to_string()))
    }

    async fn update(&self, payment: Payment) -> Result<Payment, PaymentStoreError> {
        let mut g = self.inner.write().await;
        if let Some(x) = g.iter_mut().find(|o| o.id == payment.id) {
            *x = payment.clone();
            Ok(payment)
        } else {
            Err(PaymentStoreError::NotFound(payment.id.to_string()))
        }
    }

    async fn delete(&self, id: &Uuid) -> Result<(), PaymentStoreError> {
        let mut g = self.inner.write().await;
        let before = g.len();
        g.retain(|o| &o.id != id);
        if g.len() == before {
            return Err(PaymentStoreError::NotFound(id.to_string()));
        }
        Ok(())
    }

    async fn list(&self) -> Result<Vec<Payment>, PaymentStoreError> {
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
/// DynamoDB store for Payment entities
#[derive(Clone)]
pub struct PaymentDynamoDBStore {
    service: Arc<DynamoDBDataService<Payment>>,
}

#[cfg(feature = "dynamodb")]
impl PaymentDynamoDBStore {
    pub fn new(client: DynamoDBClient, table_name: String) -> Self {
        Self {
            service: Arc::new(DynamoDBDataService::new(client, table_name)),
        }
    }
}

#[cfg(feature = "dynamodb")]
#[async_trait::async_trait]
impl EntityFetcher for PaymentDynamoDBStore {
    async fn fetch_as_json(&self, entity_id: &Uuid) -> Result<serde_json::Value, anyhow::Error> {
        let payment = self
            .get(entity_id)
            .await
            .map_err(|_| anyhow::anyhow!("Payment not found: {}", entity_id))?;
        Ok(serde_json::to_value(payment)?)
    }

    async fn list_as_json(
        &self,
        limit: Option<i32>,
        offset: Option<i32>,
    ) -> Result<Vec<serde_json::Value>, anyhow::Error> {
        let all_payments = self.list().await?;
        let offset = offset.unwrap_or(0) as usize;
        let limit = limit.unwrap_or(20) as usize;

        let payments: Vec<Payment> = all_payments.into_iter().skip(offset).take(limit).collect();
        payments
            .into_iter()
            .map(|payment| serde_json::to_value(payment).map_err(Into::into))
            .collect()
    }
}

#[cfg(feature = "dynamodb")]
#[async_trait::async_trait]
impl EntityCreator for PaymentDynamoDBStore {
    async fn create_from_json(
        &self,
        entity_data: serde_json::Value,
    ) -> Result<serde_json::Value, anyhow::Error> {
        let payment = Payment::new(
            entity_data["name"]
                .as_str()
                .unwrap_or("Payment")
                .to_string(),
            entity_data["status"]
                .as_str()
                .unwrap_or("pending")
                .to_string(),
            entity_data["number"]
                .as_str()
                .unwrap_or("PAY-000")
                .to_string(),
            entity_data["amount"].as_f64().unwrap_or(0.0),
            entity_data["method"]
                .as_str()
                .unwrap_or("credit_card")
                .to_string(),
            entity_data["transaction_id"].as_str().map(String::from),
        );

        self.create(payment.clone()).await?;
        Ok(serde_json::to_value(payment)?)
    }
}

#[cfg(feature = "dynamodb")]
#[async_trait::async_trait]
impl PaymentStore for PaymentDynamoDBStore {
    async fn create(&self, payment: Payment) -> Result<Payment, PaymentStoreError> {
        self.service
            .create(payment.clone())
            .await
            .map_err(|e| PaymentStoreError::Other(anyhow::anyhow!(e)))
    }

    async fn get(&self, id: &Uuid) -> Result<Payment, PaymentStoreError> {
        self.service
            .get(id)
            .await
            .map_err(|e| PaymentStoreError::Other(anyhow::anyhow!(e)))?
            .ok_or_else(|| PaymentStoreError::NotFound(id.to_string()))
    }

    async fn update(&self, payment: Payment) -> Result<Payment, PaymentStoreError> {
        self.service
            .update(&payment.id, payment.clone())
            .await
            .map_err(|e| PaymentStoreError::Other(anyhow::anyhow!(e)))
    }

    async fn delete(&self, id: &Uuid) -> Result<(), PaymentStoreError> {
        self.service
            .delete(id)
            .await
            .map_err(|e| PaymentStoreError::Other(anyhow::anyhow!(e)))
    }

    async fn list(&self) -> Result<Vec<Payment>, PaymentStoreError> {
        self.service
            .list()
            .await
            .map_err(|e| PaymentStoreError::Other(anyhow::anyhow!(e)))
    }
}
