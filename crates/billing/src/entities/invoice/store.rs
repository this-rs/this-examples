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

// ============================================================================
// Macro for backend store implementations
// ============================================================================

/// Generates EntityFetcher, EntityCreator, and InvoiceStore implementations
/// for a backend store type that wraps a DataService.
macro_rules! impl_invoice_backend_store {
    ($store:ident) => {
        #[async_trait::async_trait]
        impl EntityFetcher for $store {
            async fn fetch_as_json(
                &self,
                entity_id: &Uuid,
            ) -> Result<serde_json::Value, anyhow::Error> {
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
                let invoices: Vec<Invoice> =
                    all_invoices.into_iter().skip(offset).take(limit).collect();
                invoices
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
        impl InvoiceStore for $store {
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
pub struct InvoicePostgresStore {
    service: Arc<PostgresDataService<Invoice>>,
}

#[cfg(feature = "postgres")]
impl InvoicePostgresStore {
    pub fn new(pool: PgPool) -> Self {
        Self {
            service: Arc::new(PostgresDataService::new(pool)),
        }
    }
}

#[cfg(feature = "postgres")]
impl_invoice_backend_store!(InvoicePostgresStore);

// ============================================================================
// MongoDB Store Implementation
// ============================================================================

#[cfg(feature = "mongodb_backend")]
use mongodb::Database as MongoDatabase;
#[cfg(feature = "mongodb_backend")]
use this::storage::MongoDataService;

#[cfg(feature = "mongodb_backend")]
#[derive(Clone)]
pub struct InvoiceMongoStore {
    service: Arc<MongoDataService<Invoice>>,
}

#[cfg(feature = "mongodb_backend")]
impl InvoiceMongoStore {
    pub fn new(database: MongoDatabase) -> Self {
        Self {
            service: Arc::new(MongoDataService::new(database)),
        }
    }
}

#[cfg(feature = "mongodb_backend")]
impl_invoice_backend_store!(InvoiceMongoStore);

// ============================================================================
// Neo4j Store Implementation
// ============================================================================

#[cfg(feature = "neo4j")]
use neo4rs::Graph;
#[cfg(feature = "neo4j")]
use this::storage::Neo4jDataService;

#[cfg(feature = "neo4j")]
#[derive(Clone)]
pub struct InvoiceNeo4jStore {
    service: Arc<Neo4jDataService<Invoice>>,
}

#[cfg(feature = "neo4j")]
impl InvoiceNeo4jStore {
    pub fn new(graph: Graph) -> Self {
        Self {
            service: Arc::new(Neo4jDataService::new(graph)),
        }
    }
}

#[cfg(feature = "neo4j")]
impl_invoice_backend_store!(InvoiceNeo4jStore);

// ============================================================================
// ScyllaDB Store Implementation
// ============================================================================

#[cfg(feature = "scylladb")]
use scylla::client::session::Session;
#[cfg(feature = "scylladb")]
use this::storage::ScyllaDataService;

#[cfg(feature = "scylladb")]
#[derive(Clone)]
pub struct InvoiceScyllaStore {
    service: Arc<ScyllaDataService<Invoice>>,
}

#[cfg(feature = "scylladb")]
impl InvoiceScyllaStore {
    pub fn new(session: Arc<Session>, keyspace: impl Into<String>) -> Self {
        Self {
            service: Arc::new(ScyllaDataService::new(session, keyspace)),
        }
    }
}

#[cfg(feature = "scylladb")]
impl_invoice_backend_store!(InvoiceScyllaStore);

// ============================================================================
// MySQL Store Implementation
// ============================================================================

#[cfg(feature = "mysql")]
use sqlx::MySqlPool;
#[cfg(feature = "mysql")]
use this::storage::MysqlDataService;

#[cfg(feature = "mysql")]
#[derive(Clone)]
pub struct InvoiceMysqlStore {
    service: Arc<MysqlDataService<Invoice>>,
}

#[cfg(feature = "mysql")]
impl InvoiceMysqlStore {
    pub fn new(pool: MySqlPool) -> Self {
        Self {
            service: Arc::new(MysqlDataService::new(pool)),
        }
    }
}

#[cfg(feature = "mysql")]
impl_invoice_backend_store!(InvoiceMysqlStore);

// ============================================================================
// LMDB Store Implementation
// ============================================================================

#[cfg(feature = "lmdb")]
use this::storage::LmdbDataService;

#[cfg(feature = "lmdb")]
#[derive(Clone)]
pub struct InvoiceLmdbStore {
    service: Arc<LmdbDataService<Invoice>>,
}

#[cfg(feature = "lmdb")]
impl InvoiceLmdbStore {
    pub fn open(path: impl AsRef<std::path::Path>) -> anyhow::Result<Self> {
        Ok(Self {
            service: Arc::new(LmdbDataService::open(path)?),
        })
    }
}

#[cfg(feature = "lmdb")]
impl_invoice_backend_store!(InvoiceLmdbStore);
