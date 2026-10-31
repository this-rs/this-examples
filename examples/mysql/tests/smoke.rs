use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::Extension;
use testcontainers::runners::AsyncRunner;
use testcontainers_modules::mysql::Mysql;
use this::prelude::{AuthContext, Uuid};
use tower::ServiceExt;

#[ignore = "container-backed: needs Docker, run explicitly by the smoke CI job (--ignored)"]
#[tokio::test]
async fn mysql_serves_orders() {
    let container = Mysql::default()
        .start()
        .await
        .expect("start MySQL (Docker?)");
    let host = container.get_host().await.unwrap();
    let port = container.get_host_port_ipv4(3306).await.unwrap();
    let url = format!("mysql://root@{host}:{port}/test");

    let pool = sqlx::MySqlPool::connect(&url).await.expect("connect");
    this::storage::mysql::ensure_schema(&pool)
        .await
        .expect("migrations");

    let app = mysql_example::app(&url).await.expect("build app");
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
