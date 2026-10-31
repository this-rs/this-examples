use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::Extension;
use testcontainers::runners::AsyncRunner;
use testcontainers_modules::mongo::Mongo;
use this::prelude::{AuthContext, Uuid};
use tower::ServiceExt;

#[ignore = "container-backed: needs Docker, run explicitly by the smoke CI job (--ignored)"]
#[tokio::test]
async fn mongodb_serves_orders() {
    let container = Mongo::default()
        .start()
        .await
        .expect("start Mongo (Docker?)");
    let host = container.get_host().await.unwrap();
    let port = container.get_host_port_ipv4(27017).await.unwrap();
    let url = format!("mongodb://{host}:{port}");

    let app = mongodb_example::app(&url, "billing")
        .await
        .expect("build app");
    let app = app.layer(Extension(AuthContext::User {
        user_id: Uuid::from_u128(1),
        tenant_id: Uuid::from_u128(9),
        roles: vec![],
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
