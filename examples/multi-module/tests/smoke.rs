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
