use anyhow::Result;
use billing::entities::invoice::Invoice;
use billing::entities::order::Order;
use billing::entities::payment::Payment;
use billing::BillingStores;
use std::sync::Arc;
use this::core::LinkService;
use this::prelude::{InMemoryLinkService, LinkEntity};

/// Populate test data in the billing stores and create links between entities
pub async fn populate_test_data(
    stores: &BillingStores,
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
    let order1_result = stores.orders_store.create(order1.clone()).await.ok();
    let order2_result = stores.orders_store.create(order2.clone()).await.ok();

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
    let invoice1_result = stores.invoices_store.create(invoice1.clone()).await.ok();
    let invoice2_result = stores.invoices_store.create(invoice2.clone()).await.ok();
    let invoice3_result = stores.invoices_store.create(invoice3.clone()).await.ok();

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
    let payment1_result = stores.payments_store.create(payment1.clone()).await.ok();
    let payment2_result = stores.payments_store.create(payment2.clone()).await.ok();
    let payment3_result = stores.payments_store.create(payment3.clone()).await.ok();

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

/// Populate test data in the catalog stores and create links between entities
pub async fn populate_catalog_data(
    stores: &catalog::CatalogStores,
    link_service: Arc<InMemoryLinkService>,
) -> Result<()> {
    use catalog::entities::product::Product;
    use catalog::entities::category::Category;
    use catalog::entities::tag::Tag;

    // Categories
    let category1 = Category::new(
        "Electronics".into(),
        "active".into(),
        "electronics".into(),
        Some("Electronic products".into()),
    );
    let category2 = Category::new(
        "Clothing".into(),
        "active".into(),
        "clothing".into(),
        Some("Clothing items".into()),
    );
    let category3 = Category::new(
        "Laptops".into(),
        "active".into(),
        "laptops".into(),
        Some("Laptop computers".into()),
    );
    let category1_result = stores.categories_store.create(category1.clone()).await.ok();
    let category2_result = stores.categories_store.create(category2.clone()).await.ok();
    let category3_result = stores.categories_store.create(category3.clone()).await.ok();

    // Tags
    let tag1 = Tag::new(
        "featured".into(),
        "active".into(),
        Some("#FF5733".into()),
        Some("Featured products".into()),
    );
    let tag2 = Tag::new(
        "new".into(),
        "active".into(),
        Some("#33FF57".into()),
        Some("New arrivals".into()),
    );
    let tag3 = Tag::new(
        "sale".into(),
        "active".into(),
        Some("#3357FF".into()),
        Some("On sale".into()),
    );
    let tag1_result = stores.tags_store.create(tag1.clone()).await.ok();
    let tag2_result = stores.tags_store.create(tag2.clone()).await.ok();
    let tag3_result = stores.tags_store.create(tag3.clone()).await.ok();

    // Products
    let product1 = Product::new(
        "Laptop Pro".into(),
        "active".into(),
        "LAP-001".into(),
        1299.99,
        10,
        Some("High-performance laptop".into()),
    );
    let product2 = Product::new(
        "T-Shirt Basic".into(),
        "active".into(),
        "TSH-001".into(),
        19.99,
        50,
        Some("Basic cotton t-shirt".into()),
    );
    let product3 = Product::new(
        "Smartphone X".into(),
        "active".into(),
        "PHN-001".into(),
        899.99,
        25,
        Some("Latest smartphone model".into()),
    );
    let product1_result = stores.products_store.create(product1.clone()).await.ok();
    let product2_result = stores.products_store.create(product2.clone()).await.ok();
    let product3_result = stores.products_store.create(product3.clone()).await.ok();

    // Create links between entities
    // Product → Category (many-to-many)
    if let (Some(p1), Some(c1)) = (product1_result.as_ref(), category1_result.as_ref()) {
        let link = LinkEntity::new(
            "has_category",
            p1.id,
            c1.id,
            Some(serde_json::json!({
                "created_by": "test-data",
                "primary": true
            })),
        );
        link_service.create(link).await.ok();
    }

    if let (Some(p1), Some(c3)) = (product1_result.as_ref(), category3_result.as_ref()) {
        let link = LinkEntity::new(
            "has_category",
            p1.id,
            c3.id,
            Some(serde_json::json!({
                "created_by": "test-data",
                "primary": false
            })),
        );
        link_service.create(link).await.ok();
    }

    if let (Some(p2), Some(c2)) = (product2_result.as_ref(), category2_result.as_ref()) {
        let link = LinkEntity::new(
            "has_category",
            p2.id,
            c2.id,
            Some(serde_json::json!({
                "created_by": "test-data",
                "primary": true
            })),
        );
        link_service.create(link).await.ok();
    }

    if let (Some(p3), Some(c1)) = (product3_result.as_ref(), category1_result.as_ref()) {
        let link = LinkEntity::new(
            "has_category",
            p3.id,
            c1.id,
            Some(serde_json::json!({
                "created_by": "test-data",
                "primary": true
            })),
        );
        link_service.create(link).await.ok();
    }

    // Product → Tag (many-to-many)
    if let (Some(p1), Some(t1)) = (product1_result.as_ref(), tag1_result.as_ref()) {
        let link = LinkEntity::new(
            "has_tag",
            p1.id,
            t1.id,
            Some(serde_json::json!({
                "created_by": "test-data"
            })),
        );
        link_service.create(link).await.ok();
    }

    if let (Some(p1), Some(t2)) = (product1_result.as_ref(), tag2_result.as_ref()) {
        let link = LinkEntity::new(
            "has_tag",
            p1.id,
            t2.id,
            Some(serde_json::json!({
                "created_by": "test-data"
            })),
        );
        link_service.create(link).await.ok();
    }

    if let (Some(p2), Some(t3)) = (product2_result.as_ref(), tag3_result.as_ref()) {
        let link = LinkEntity::new(
            "has_tag",
            p2.id,
            t3.id,
            Some(serde_json::json!({
                "created_by": "test-data"
            })),
        );
        link_service.create(link).await.ok();
    }

    if let (Some(p3), Some(t1)) = (product3_result.as_ref(), tag1_result.as_ref()) {
        let link = LinkEntity::new(
            "has_tag",
            p3.id,
            t1.id,
            Some(serde_json::json!({
                "created_by": "test-data"
            })),
        );
        link_service.create(link).await.ok();
    }

    // Category → Category (hierarchical - reflexive)
    if let (Some(c3), Some(c1)) = (category3_result.as_ref(), category1_result.as_ref()) {
        let link = LinkEntity::new(
            "has_parent",
            c3.id,
            c1.id,
            Some(serde_json::json!({
                "created_by": "test-data",
                "level": 1
            })),
        );
        link_service.create(link).await.ok();
    }

    Ok(())
}
