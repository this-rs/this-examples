use anyhow::Result;
use billing::BillingStores;
use billing::entities::order::{Order, OrderStore};
use billing::entities::invoice::{Invoice, InvoiceStore};
use billing::entities::payment::{Payment, PaymentStore};

/// Populate test data in the billing stores
pub async fn populate_test_data(stores: BillingStores) -> Result<()> {
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
    stores.orders.create(order1.clone()).await.ok();
    stores.orders.create(order2.clone()).await.ok();

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
    stores.invoices.create(invoice1.clone()).await.ok();
    stores.invoices.create(invoice2.clone()).await.ok();
    stores.invoices.create(invoice3.clone()).await.ok();

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
    stores.payments.create(payment1.clone()).await.ok();
    stores.payments.create(payment2.clone()).await.ok();
    stores.payments.create(payment3.clone()).await.ok();

    Ok(())
}
