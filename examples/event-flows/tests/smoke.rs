use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::Extension;
use this::prelude::{AuthContext, Uuid};
use tower::ServiceExt;

#[tokio::test]
async fn event_flows_serves_orders() {
    let app = event_flows_example::app().await.expect("build app");
    let app = app.layer(Extension(AuthContext::User {
        user_id: Uuid::from_u128(1),
        tenant_id: Uuid::from_u128(9),
        roles: vec![],
        scopes: Vec::new(),
    }));
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/orders")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("request");
    assert_eq!(resp.status(), StatusCode::OK);
}
