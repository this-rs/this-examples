use anyhow::Result;
use billing::entities::invoice::{Invoice, InvoiceStore};
use billing::entities::order::{Order, OrderStore};
use billing::entities::payment::{Payment, PaymentStore};
use billing::BillingStores;
use std::sync::Arc;
use this::core::LinkService;
use this::prelude::{InMemoryLinkService, LinkEntity};

/// Populate test data in the billing stores and create links between entities
pub async fn populate_test_data(
    stores: BillingStores,
    link_service: Arc<InMemoryLinkService>,
) -> Result<()> {
    // Orders
    let order1 = Order::new(
        "Order 1".into(),
        "pending".into(),
        "ORD-001".into(),
        999.99,
        Some("Customer 1".into()),
        Some("Test order 1".into()),
    );
    let order2 = Order::new(
        "Order 2".into(),
        "paid".into(),
        "ORD-002".into(),
        4999.99,
        Some("Customer 2".into()),
        Some("Test order 2".into()),
    );
    let order1_result = stores.orders.create(order1.clone()).await.ok();
    let order2_result = stores.orders.create(order2.clone()).await.ok();

    // Invoices
    let invoice1 = Invoice::new(
        "Invoice 1".into(),
        "draft".into(),
        "INV-001".into(),
        999.99,
        Some("2025-12-31".into()),
        None,
    );
    let invoice2 = Invoice::new(
        "Invoice 2".into(),
        "paid".into(),
        "INV-002".into(),
        999.99,
        Some("2025-12-31".into()),
        Some("2025-01-15".into()),
    );
    let invoice3 = Invoice::new(
        "Invoice 3".into(),
        "sent".into(),
        "INV-003".into(),
        4999.99,
        Some("2025-12-31".into()),
        None,
    );
    let invoice1_result = stores.invoices.create(invoice1.clone()).await.ok();
    let invoice2_result = stores.invoices.create(invoice2.clone()).await.ok();
    let invoice3_result = stores.invoices.create(invoice3.clone()).await.ok();

    // Payments
    let payment1 = Payment::new(
        "Payment 1".into(),
        "completed".into(),
        "PAY-001".into(),
        999.99,
        "credit_card".into(),
        Some("txn_001".into()),
    );
    let payment2 = Payment::new(
        "Payment 2".into(),
        "completed".into(),
        "PAY-002".into(),
        999.99,
        "bank_transfer".into(),
        Some("txn_002".into()),
    );
    let payment3 = Payment::new(
        "Payment 3".into(),
        "pending".into(),
        "PAY-003".into(),
        4999.99,
        "credit_card".into(),
        Some("txn_003".into()),
    );
    let payment1_result = stores.payments.create(payment1.clone()).await.ok();
    let payment2_result = stores.payments.create(payment2.clone()).await.ok();
    let payment3_result = stores.payments.create(payment3.clone()).await.ok();

    // Create links between entities
    // order1 -> invoice1
    if let (Some(o1), Some(i1)) = (order1_result.as_ref(), invoice1_result.as_ref()) {
        let link = LinkEntity::new(
            "has_invoice",
            o1.id,
            i1.id,
            Some(serde_json::json!({
                "created_by": "test-data",
                "invoice_type": "standard"
            })),
        );
        link_service.create(link).await.ok();
    }

    // order1 -> invoice2
    if let (Some(o1), Some(i2)) = (order1_result.as_ref(), invoice2_result.as_ref()) {
        let link = LinkEntity::new(
            "has_invoice",
            o1.id,
            i2.id,
            Some(serde_json::json!({
                "created_by": "test-data",
                "invoice_type": "partial"
            })),
        );
        link_service.create(link).await.ok();
    }

    // order2 -> invoice3
    if let (Some(o2), Some(i3)) = (order2_result.as_ref(), invoice3_result.as_ref()) {
        let link = LinkEntity::new(
            "has_invoice",
            o2.id,
            i3.id,
            Some(serde_json::json!({
                "created_by": "test-data",
                "invoice_type": "standard"
            })),
        );
        link_service.create(link).await.ok();
    }

    // invoice1 -> payment1
    if let (Some(i1), Some(p1)) = (invoice1_result.as_ref(), payment1_result.as_ref()) {
        let link = LinkEntity::new(
            "payment",
            i1.id,
            p1.id,
            Some(serde_json::json!({
                "payment_method": "credit_card",
                "transaction_id": "txn_001"
            })),
        );
        link_service.create(link).await.ok();
    }

    // invoice2 -> payment2
    if let (Some(i2), Some(p2)) = (invoice2_result.as_ref(), payment2_result.as_ref()) {
        let link = LinkEntity::new(
            "payment",
            i2.id,
            p2.id,
            Some(serde_json::json!({
                "payment_method": "bank_transfer",
                "transaction_id": "txn_002"
            })),
        );
        link_service.create(link).await.ok();
    }

    // invoice3 -> payment3
    if let (Some(i3), Some(p3)) = (invoice3_result.as_ref(), payment3_result.as_ref()) {
        let link = LinkEntity::new(
            "payment",
            i3.id,
            p3.id,
            Some(serde_json::json!({
                "payment_method": "credit_card",
                "transaction_id": "txn_003"
            })),
        );
        link_service.create(link).await.ok();
    }

    Ok(())
}
