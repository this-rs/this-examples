use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::Extension;
use testcontainers::runners::AsyncRunner;
use testcontainers_modules::postgres::Postgres;
use this::prelude::{AuthContext, Uuid};
use tower::ServiceExt;

#[tokio::test]
async fn postgres_serves_orders() {
    let container = Postgres::default()
        .start()
        .await
        .expect("start Postgres container (Docker?)");

    let host = container.get_host().await.unwrap();
    let port = container.get_host_port_ipv4(5432).await.unwrap();
    let url = format!("postgres://postgres:postgres@{host}:{port}/postgres");

    let pool = sqlx::PgPool::connect(&url).await.expect("connect");
    this::storage::postgres::ensure_schema(&pool)
        .await
        .expect("run migrations");

    let app = postgres_example::app(&url).await.expect("build app");
    let app = app.layer(Extension(AuthContext::User {
        user_id: Uuid::from_u128(1),
        tenant_id: Uuid::from_u128(9),
        roles: vec![],
    }));
    let resp = app
        .oneshot(Request::builder().uri("/orders").body(Body::empty()).unwrap())
        .await
        .expect("request");
    assert_eq!(resp.status(), StatusCode::OK);
}
