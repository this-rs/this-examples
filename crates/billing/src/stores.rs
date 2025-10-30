use std::sync::Arc;
use this::prelude::*;
use tokio::sync::RwLock;

use crate::entities::invoice::{Invoice, InvoiceStore, InvoiceStoreError};
use crate::entities::order::{Order, OrderStore, OrderStoreError};
use crate::entities::payment::{Payment, PaymentStore, PaymentStoreError};

use crate::module::BillingStores;

impl BillingStores {
    pub fn new_in_memory() -> Self {
        Self {
            orders: Arc::new(InMemoryOrderStore::default()),
            invoices: Arc::new(InMemoryInvoiceStore::default()),
            payments: Arc::new(InMemoryPaymentStore::default()),
        }
    }
}

#[derive(Clone, Default)]
pub struct InMemoryOrderStore {
    inner: Arc<RwLock<Vec<Order>>>,
}

#[async_trait::async_trait]
impl EntityFetcher for InMemoryOrderStore {
    async fn fetch_as_json(&self, entity_id: &Uuid) -> Result<serde_json::Value, anyhow::Error> {
        let order = self
            .get(entity_id)
            .await
            .map_err(|_| anyhow::anyhow!("Order not found: {}", entity_id))?;
        Ok(serde_json::to_value(order)?)
    }

    async fn list_as_json(
        &self,
        limit: Option<i32>,
        offset: Option<i32>,
    ) -> Result<Vec<serde_json::Value>, anyhow::Error> {
        let all_orders = self.list().await?;
        let offset = offset.unwrap_or(0) as usize;
        let limit = limit.unwrap_or(20) as usize;

        let orders: Vec<Order> = all_orders.into_iter().skip(offset).take(limit).collect();
        orders
            .into_iter()
            .map(|order| serde_json::to_value(order).map_err(Into::into))
            .collect()
    }
}

#[async_trait::async_trait]
impl EntityCreator for InMemoryOrderStore {
    async fn create_from_json(
        &self,
        entity_data: serde_json::Value,
    ) -> Result<serde_json::Value, anyhow::Error> {
        let order = Order::new(
            entity_data["name"].as_str().unwrap_or("Order").to_string(),
            entity_data["status"]
                .as_str()
                .unwrap_or("pending")
                .to_string(),
            entity_data["number"]
                .as_str()
                .unwrap_or("ORD-000")
                .to_string(),
            entity_data["amount"].as_f64().unwrap_or(0.0),
            entity_data["customer_name"].as_str().map(String::from),
            entity_data["notes"].as_str().map(String::from),
        );

        self.create(order.clone()).await?;
        Ok(serde_json::to_value(order)?)
    }
}

#[async_trait::async_trait]
impl OrderStore for InMemoryOrderStore {
    async fn create(&self, order: Order) -> Result<Order, OrderStoreError> {
        let mut g = self.inner.write().await;
        if g.iter().any(|o| o.id == order.id) {
            return Err(OrderStoreError::Conflict(order.id.to_string()));
        }
        g.push(order.clone());
        Ok(order)
    }

    async fn get(&self, id: &Uuid) -> Result<Order, OrderStoreError> {
        let g = self.inner.read().await;
        g.iter()
            .find(|o| &o.id == id)
            .cloned()
            .ok_or_else(|| OrderStoreError::NotFound(id.to_string()))
    }

    async fn update(&self, order: Order) -> Result<Order, OrderStoreError> {
        let mut g = self.inner.write().await;
        if let Some(x) = g.iter_mut().find(|o| o.id == order.id) {
            *x = order.clone();
            Ok(order)
        } else {
            Err(OrderStoreError::NotFound(order.id.to_string()))
        }
    }

    async fn delete(&self, id: &Uuid) -> Result<(), OrderStoreError> {
        let mut g = self.inner.write().await;
        let before = g.len();
        g.retain(|o| &o.id != id);
        if g.len() == before {
            return Err(OrderStoreError::NotFound(id.to_string()));
        }
        Ok(())
    }

    async fn list(&self) -> Result<Vec<Order>, OrderStoreError> {
        Ok(self.inner.read().await.clone())
    }
}

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
