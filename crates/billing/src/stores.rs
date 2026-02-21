use std::sync::Arc;

use crate::module::BillingStores;

// Import stores from entity modules
use crate::entities::invoice::InMemoryInvoiceStore;
use crate::entities::order::InMemoryOrderStore;
use crate::entities::payment::InMemoryPaymentStore;

#[cfg(feature = "dynamodb")]
use crate::entities::invoice::InvoiceDynamoDBStore;
#[cfg(feature = "dynamodb")]
use crate::entities::order::OrderDynamoDBStore;
#[cfg(feature = "dynamodb")]
use crate::entities::payment::PaymentDynamoDBStore;
#[cfg(feature = "dynamodb")]
use aws_sdk_dynamodb::Client as DynamoDBClient;

#[cfg(feature = "postgres")]
use crate::entities::invoice::InvoicePostgresStore;
#[cfg(feature = "postgres")]
use crate::entities::order::OrderPostgresStore;
#[cfg(feature = "postgres")]
use crate::entities::payment::PaymentPostgresStore;
#[cfg(feature = "postgres")]
use sqlx::PgPool;

#[cfg(feature = "mongodb_backend")]
use crate::entities::invoice::InvoiceMongoStore;
#[cfg(feature = "mongodb_backend")]
use crate::entities::order::OrderMongoStore;
#[cfg(feature = "mongodb_backend")]
use crate::entities::payment::PaymentMongoStore;
#[cfg(feature = "mongodb_backend")]
use mongodb::Database as MongoDatabase;

#[cfg(feature = "neo4j")]
use crate::entities::invoice::InvoiceNeo4jStore;
#[cfg(feature = "neo4j")]
use crate::entities::order::OrderNeo4jStore;
#[cfg(feature = "neo4j")]
use crate::entities::payment::PaymentNeo4jStore;
#[cfg(feature = "neo4j")]
use neo4rs::Graph;

#[cfg(feature = "scylladb")]
use crate::entities::invoice::InvoiceScyllaStore;
#[cfg(feature = "scylladb")]
use crate::entities::order::OrderScyllaStore;
#[cfg(feature = "scylladb")]
use crate::entities::payment::PaymentScyllaStore;
#[cfg(feature = "scylladb")]
use scylla::client::session::Session;

#[cfg(feature = "mysql")]
use crate::entities::invoice::InvoiceMysqlStore;
#[cfg(feature = "mysql")]
use crate::entities::order::OrderMysqlStore;
#[cfg(feature = "mysql")]
use crate::entities::payment::PaymentMysqlStore;
#[cfg(feature = "mysql")]
use sqlx::MySqlPool;

#[cfg(feature = "lmdb")]
use crate::entities::invoice::InvoiceLmdbStore;
#[cfg(feature = "lmdb")]
use crate::entities::order::OrderLmdbStore;
#[cfg(feature = "lmdb")]
use crate::entities::payment::PaymentLmdbStore;

// ============================================================================
// Store Factories
// ============================================================================

impl BillingStores {
    /// Create stores with in-memory implementations
    pub fn new_in_memory() -> Self {
        let orders = Arc::new(InMemoryOrderStore::default());
        let invoices = Arc::new(InMemoryInvoiceStore::default());
        let payments = Arc::new(InMemoryPaymentStore::default());

        Self {
            orders_store: orders.clone(),
            orders_entity: orders,
            invoices_store: invoices.clone(),
            invoices_entity: invoices,
            payments_store: payments.clone(),
            payments_entity: payments,
        }
    }

    #[cfg(feature = "dynamodb")]
    /// Create stores with DynamoDB implementations
    pub fn new_dynamodb(
        client: DynamoDBClient,
        orders_table: String,
        invoices_table: String,
        payments_table: String,
    ) -> Self {
        let orders = Arc::new(OrderDynamoDBStore::new(client.clone(), orders_table));
        let invoices = Arc::new(InvoiceDynamoDBStore::new(client.clone(), invoices_table));
        let payments = Arc::new(PaymentDynamoDBStore::new(client, payments_table));

        Self {
            orders_store: orders.clone(),
            orders_entity: orders,
            invoices_store: invoices.clone(),
            invoices_entity: invoices,
            payments_store: payments.clone(),
            payments_entity: payments,
        }
    }

    #[cfg(feature = "postgres")]
    /// Create stores with PostgreSQL implementations
    pub fn new_postgres(pool: PgPool) -> Self {
        let orders = Arc::new(OrderPostgresStore::new(pool.clone()));
        let invoices = Arc::new(InvoicePostgresStore::new(pool.clone()));
        let payments = Arc::new(PaymentPostgresStore::new(pool));

        Self {
            orders_store: orders.clone(),
            orders_entity: orders,
            invoices_store: invoices.clone(),
            invoices_entity: invoices,
            payments_store: payments.clone(),
            payments_entity: payments,
        }
    }

    #[cfg(feature = "mongodb_backend")]
    /// Create stores with MongoDB implementations
    pub fn new_mongodb(database: MongoDatabase) -> Self {
        let orders = Arc::new(OrderMongoStore::new(database.clone()));
        let invoices = Arc::new(InvoiceMongoStore::new(database.clone()));
        let payments = Arc::new(PaymentMongoStore::new(database));

        Self {
            orders_store: orders.clone(),
            orders_entity: orders,
            invoices_store: invoices.clone(),
            invoices_entity: invoices,
            payments_store: payments.clone(),
            payments_entity: payments,
        }
    }

    #[cfg(feature = "neo4j")]
    /// Create stores with Neo4j implementations
    pub fn new_neo4j(graph: Graph) -> Self {
        let orders = Arc::new(OrderNeo4jStore::new(graph.clone()));
        let invoices = Arc::new(InvoiceNeo4jStore::new(graph.clone()));
        let payments = Arc::new(PaymentNeo4jStore::new(graph));

        Self {
            orders_store: orders.clone(),
            orders_entity: orders,
            invoices_store: invoices.clone(),
            invoices_entity: invoices,
            payments_store: payments.clone(),
            payments_entity: payments,
        }
    }

    #[cfg(feature = "scylladb")]
    /// Create stores with ScyllaDB implementations
    pub fn new_scylladb(session: Arc<Session>, keyspace: impl Into<String>) -> Self {
        let ks: String = keyspace.into();
        let orders = Arc::new(OrderScyllaStore::new(session.clone(), ks.clone()));
        let invoices = Arc::new(InvoiceScyllaStore::new(session.clone(), ks.clone()));
        let payments = Arc::new(PaymentScyllaStore::new(session, ks));

        Self {
            orders_store: orders.clone(),
            orders_entity: orders,
            invoices_store: invoices.clone(),
            invoices_entity: invoices,
            payments_store: payments.clone(),
            payments_entity: payments,
        }
    }

    #[cfg(feature = "mysql")]
    /// Create stores with MySQL implementations
    pub fn new_mysql(pool: MySqlPool) -> Self {
        let orders = Arc::new(OrderMysqlStore::new(pool.clone()));
        let invoices = Arc::new(InvoiceMysqlStore::new(pool.clone()));
        let payments = Arc::new(PaymentMysqlStore::new(pool));

        Self {
            orders_store: orders.clone(),
            orders_entity: orders,
            invoices_store: invoices.clone(),
            invoices_entity: invoices,
            payments_store: payments.clone(),
            payments_entity: payments,
        }
    }

    #[cfg(feature = "lmdb")]
    /// Create stores with LMDB implementations
    pub fn new_lmdb(path: impl AsRef<std::path::Path>) -> anyhow::Result<Self> {
        let orders = Arc::new(OrderLmdbStore::open(path.as_ref())?);
        let invoices = Arc::new(InvoiceLmdbStore::open(path.as_ref())?);
        let payments = Arc::new(PaymentLmdbStore::open(path.as_ref())?);

        Ok(Self {
            orders_store: orders.clone(),
            orders_entity: orders,
            invoices_store: invoices.clone(),
            invoices_entity: invoices,
            payments_store: payments.clone(),
            payments_entity: payments,
        })
    }
}
