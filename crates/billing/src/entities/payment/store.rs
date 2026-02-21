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

// ============================================================================
// Macro for backend store implementations
// ============================================================================

/// Generates EntityFetcher, EntityCreator, and PaymentStore implementations
/// for a backend store type that wraps a DataService.
macro_rules! impl_payment_backend_store {
    ($store:ident) => {
        #[async_trait::async_trait]
        impl EntityFetcher for $store {
            async fn fetch_as_json(
                &self,
                entity_id: &Uuid,
            ) -> Result<serde_json::Value, anyhow::Error> {
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
                let payments: Vec<Payment> =
                    all_payments.into_iter().skip(offset).take(limit).collect();
                payments
                    .into_iter()
                    .map(|o| serde_json::to_value(o).map_err(Into::into))
                    .collect()
            }
        }

        #[async_trait::async_trait]
        impl EntityCreator for $store {
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
        impl PaymentStore for $store {
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
    };
}

// ============================================================================
// PostgreSQL Store Implementation
// ============================================================================

#[cfg(feature = "postgres")]
use sqlx::PgPool;
#[cfg(feature = "postgres")]
use this::storage::PostgresDataService;

#[cfg(feature = "postgres")]
#[derive(Clone)]
pub struct PaymentPostgresStore {
    service: Arc<PostgresDataService<Payment>>,
}

#[cfg(feature = "postgres")]
impl PaymentPostgresStore {
    pub fn new(pool: PgPool) -> Self {
        Self {
            service: Arc::new(PostgresDataService::new(pool)),
        }
    }
}

#[cfg(feature = "postgres")]
impl_payment_backend_store!(PaymentPostgresStore);

// ============================================================================
// MongoDB Store Implementation
// ============================================================================

#[cfg(feature = "mongodb_backend")]
use mongodb::Database as MongoDatabase;
#[cfg(feature = "mongodb_backend")]
use this::storage::MongoDataService;

#[cfg(feature = "mongodb_backend")]
#[derive(Clone)]
pub struct PaymentMongoStore {
    service: Arc<MongoDataService<Payment>>,
}

#[cfg(feature = "mongodb_backend")]
impl PaymentMongoStore {
    pub fn new(database: MongoDatabase) -> Self {
        Self {
            service: Arc::new(MongoDataService::new(database)),
        }
    }
}

#[cfg(feature = "mongodb_backend")]
impl_payment_backend_store!(PaymentMongoStore);

// ============================================================================
// Neo4j Store Implementation
// ============================================================================

#[cfg(feature = "neo4j")]
use neo4rs::Graph;
#[cfg(feature = "neo4j")]
use this::storage::Neo4jDataService;

#[cfg(feature = "neo4j")]
#[derive(Clone)]
pub struct PaymentNeo4jStore {
    service: Arc<Neo4jDataService<Payment>>,
}

#[cfg(feature = "neo4j")]
impl PaymentNeo4jStore {
    pub fn new(graph: Graph) -> Self {
        Self {
            service: Arc::new(Neo4jDataService::new(graph)),
        }
    }
}

#[cfg(feature = "neo4j")]
impl_payment_backend_store!(PaymentNeo4jStore);

// ============================================================================
// ScyllaDB Store Implementation
// ============================================================================

#[cfg(feature = "scylladb")]
use scylla::client::session::Session;
#[cfg(feature = "scylladb")]
use this::storage::ScyllaDataService;

#[cfg(feature = "scylladb")]
#[derive(Clone)]
pub struct PaymentScyllaStore {
    service: Arc<ScyllaDataService<Payment>>,
}

#[cfg(feature = "scylladb")]
impl PaymentScyllaStore {
    pub fn new(session: Arc<Session>, keyspace: impl Into<String>) -> Self {
        Self {
            service: Arc::new(ScyllaDataService::new(session, keyspace)),
        }
    }
}

#[cfg(feature = "scylladb")]
impl_payment_backend_store!(PaymentScyllaStore);

// ============================================================================
// MySQL Store Implementation
// ============================================================================

#[cfg(feature = "mysql")]
use sqlx::MySqlPool;
#[cfg(feature = "mysql")]
use this::storage::MysqlDataService;

#[cfg(feature = "mysql")]
#[derive(Clone)]
pub struct PaymentMysqlStore {
    service: Arc<MysqlDataService<Payment>>,
}

#[cfg(feature = "mysql")]
impl PaymentMysqlStore {
    pub fn new(pool: MySqlPool) -> Self {
        Self {
            service: Arc::new(MysqlDataService::new(pool)),
        }
    }
}

#[cfg(feature = "mysql")]
impl_payment_backend_store!(PaymentMysqlStore);

// ============================================================================
// LMDB Store Implementation
// ============================================================================

#[cfg(feature = "lmdb")]
use this::storage::LmdbDataService;

#[cfg(feature = "lmdb")]
#[derive(Clone)]
pub struct PaymentLmdbStore {
    service: Arc<LmdbDataService<Payment>>,
}

#[cfg(feature = "lmdb")]
impl PaymentLmdbStore {
    pub fn open(path: impl AsRef<std::path::Path>) -> anyhow::Result<Self> {
        Ok(Self {
            service: Arc::new(LmdbDataService::open(path)?),
        })
    }
}

#[cfg(feature = "lmdb")]
impl_payment_backend_store!(PaymentLmdbStore);
