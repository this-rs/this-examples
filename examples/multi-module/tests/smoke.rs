use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::Extension;
use this::prelude::{AuthContext, Uuid};
use tower::ServiceExt;

#[tokio::test]
async fn multi_module_serves_orders_and_products() {
    let app = multi_module_example::app().await.expect("build app");
    let app = app.layer(Extension(AuthContext::User {
        user_id: Uuid::from_u128(1),
        tenant_id: Uuid::from_u128(9),
        roles: vec![],
        scopes: Vec::new(),
    }));

    for uri in ["/orders", "/products", "/stores"] {
        let resp = app
            .clone()
            .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
            .await
            .expect("request");
        assert_eq!(resp.status(), StatusCode::OK, "uri {uri}");
    }
}

/// Shipping depends on two other modules: a shipment links to a catalog product and
/// to a billing order. Each link is created then read back from both sides.
#[tokio::test]
async fn shipping_links_to_catalog_and_billing_both_ways() {
    use serde_json::{json, Value};

    let app = multi_module_example::app().await.expect("build app");
    let app = app.layer(Extension(AuthContext::User {
        user_id: Uuid::from_u128(1),
        tenant_id: Uuid::from_u128(9),
        roles: vec![],
        scopes: Vec::new(),
    }));

    async fn call(
        app: &axum::Router,
        method: &str,
        uri: &str,
        body: Option<Value>,
    ) -> (StatusCode, Value) {
        let mut req = Request::builder().method(method).uri(uri);
        let body = match body {
            Some(v) => {
                req = req.header("content-type", "application/json");
                Body::from(v.to_string())
            }
            None => Body::empty(),
        };
        let resp = app
            .clone()
            .oneshot(req.body(body).unwrap())
            .await
            .expect("request");
        let status = resp.status();
        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .expect("body");
        let json = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
        (status, json)
    }

    let (s, product) = call(
        &app,
        "POST",
        "/products",
        Some(json!({
            "sku": "SHP-PRD-1", "price": 9.5, "stock_quantity": 3, "status": "active"
        })),
    )
    .await;
    assert!(s.is_success(), "create product: {s} {product}");
    let product_id = product["id"].as_str().expect("product id").to_string();

    let (s, order) = call(
        &app,
        "POST",
        "/orders",
        Some(json!({
            "number": "ORD-SHP-1", "amount": 19.0, "status": "pending"
        })),
    )
    .await;
    assert!(s.is_success(), "create order: {s} {order}");
    let order_id = order["id"].as_str().expect("order id").to_string();

    let (s, shipment) = call(
        &app,
        "POST",
        "/shipments",
        Some(json!({
            "number": "SHP-001", "carrier": "DHL", "status": "preparing"
        })),
    )
    .await;
    assert!(s.is_success(), "create shipment: {s} {shipment}");
    let shipment_id = shipment["id"].as_str().expect("shipment id").to_string();

    let (s, _) = call(
        &app,
        "POST",
        &format!("/shipments/{shipment_id}/products/{product_id}"),
        Some(json!({})),
    )
    .await;
    assert!(s.is_success(), "link shipment→product: {s}");
    let (s, _) = call(
        &app,
        "POST",
        &format!("/shipments/{shipment_id}/orders/{order_id}"),
        Some(json!({})),
    )
    .await;
    assert!(s.is_success(), "link shipment→order: {s}");

    // Forward: the shipment lists its products and its orders.
    let (s, fwd) = call(
        &app,
        "GET",
        &format!("/shipments/{shipment_id}/products"),
        None,
    )
    .await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(
        fwd["data"].as_array().map(Vec::len),
        Some(1),
        "products of shipment: {fwd}"
    );
    let (s, fwd) = call(
        &app,
        "GET",
        &format!("/shipments/{shipment_id}/orders"),
        None,
    )
    .await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(
        fwd["data"].as_array().map(Vec::len),
        Some(1),
        "orders of shipment: {fwd}"
    );

    // Reverse: the product and the order each see the shipment.
    let (s, rev) = call(
        &app,
        "GET",
        &format!("/products/{product_id}/shipments"),
        None,
    )
    .await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(
        rev["data"].as_array().map(Vec::len),
        Some(1),
        "shipments of product: {rev}"
    );
    let (s, rev) = call(&app, "GET", &format!("/orders/{order_id}/shipments"), None).await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(
        rev["data"].as_array().map(Vec::len),
        Some(1),
        "shipments of order: {rev}"
    );
}
