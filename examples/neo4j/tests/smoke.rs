use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::Extension;
use testcontainers::runners::AsyncRunner;
use testcontainers_modules::neo4j::Neo4j;
use this::prelude::{AuthContext, Uuid};
use tower::ServiceExt;

#[ignore = "container-backed: needs Docker, run explicitly by the smoke CI job (--ignored)"]
#[tokio::test]
async fn neo4j_serves_orders() {
    let container = Neo4j::default()
        .start()
        .await
        .expect("start Neo4j (Docker?)");
    let host = container.get_host().await.unwrap();
    let port = container.get_host_port_ipv4(7687).await.unwrap();
    // testcontainers-modules Neo4j sets NEO4J_AUTH=neo4j/password
    let bolt = format!("{host}:{port}");

    let app = neo4j_example::app(&bolt, "neo4j", "password")
        .await
        .expect("build app");
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
