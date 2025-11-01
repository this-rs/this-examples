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
}
