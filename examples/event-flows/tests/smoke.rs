use axum::body::{to_bytes, Body};
use axum::http::{Method, Request, StatusCode};
use axum::{Extension, Router};
use serde_json::{json, Value};
use this::prelude::{AuthContext, Uuid};
use tower::ServiceExt;

fn caller() -> AuthContext {
    AuthContext::User {
        user_id: Uuid::from_u128(1),
        tenant_id: Uuid::from_u128(9),
        roles: vec![],
        scopes: Vec::new(),
    }
}

async fn call(app: &Router, method: Method, uri: &str, body: Option<Value>) -> (StatusCode, Value) {
    let req = match body {
        Some(b) => Request::builder()
            .method(method)
            .uri(uri)
            .header("content-type", "application/json")
            .body(Body::from(b.to_string())),
        None => Request::builder()
            .method(method)
            .uri(uri)
            .body(Body::empty()),
    }
    .expect("request");
    let resp = app.clone().oneshot(req).await.expect("response");
    let status = resp.status();
    let bytes = to_bytes(resp.into_body(), usize::MAX).await.expect("body");
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}

/// The order API round-trips real content: a created order is echoed back and
/// listed next to the seeded billing data.
///
/// This does NOT assert the `notify-new-order` flow's delivery: in this demo
/// `/notifications/*` is refused by the auth policy (no `notifications` entity
/// is registered), so the notification cannot be read back over REST.
#[tokio::test]
async fn orders_round_trip_with_content() {
    let app = event_flows_example::app()
        .await
        .expect("build app")
        .layer(Extension(caller()));

    let (status, created) = call(
        &app,
        Method::POST,
        "/orders",
        Some(json!({
            "name": "Laptop Pro",
            "number": "ORD-100",
            "amount": 1299.0,
            "status": "pending",
        })),
    )
    .await;
    assert!(
        status.is_success(),
        "POST /orders returned {status}: {created}"
    );
    assert_eq!(
        created["name"], "Laptop Pro",
        "created order echoes its name"
    );

    let (status, orders) = call(&app, Method::GET, "/orders", None).await;
    assert_eq!(status, StatusCode::OK);
    let names: Vec<&str> = orders
        .get("data")
        .and_then(Value::as_array)
        .unwrap_or_else(|| panic!("GET /orders body has no data array: {orders}"))
        .iter()
        .filter_map(|o| o["name"].as_str())
        .collect();
    assert!(
        names.contains(&"Order 1"),
        "seeded billing data must be listed, got {names:?}"
    );
    assert!(
        names.contains(&"Laptop Pro"),
        "GET /orders must list the created order, got {names:?}"
    );
}
