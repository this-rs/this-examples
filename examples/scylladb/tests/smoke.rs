use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::Extension;
use scylla::client::session_builder::SessionBuilder;
use testcontainers::core::{IntoContainerPort, WaitFor};
use testcontainers::runners::AsyncRunner;
use testcontainers::{GenericImage, ImageExt};
use this::prelude::{AuthContext, Uuid};
use tower::ServiceExt;

#[tokio::test]
async fn scylladb_serves_orders() {
    // Cassandra is used as a CQL-compatible stand-in (ScyllaDB needs AIO slots
    // unavailable under Docker Desktop).
    let container = GenericImage::new("cassandra", "4.1")
        .with_exposed_port(9042.tcp())
        .with_wait_for(WaitFor::message_on_stdout("Startup complete"))
        .with_env_var("MAX_HEAP_SIZE", "256M")
        .with_env_var("HEAP_NEWSIZE", "50M")
        .start()
        .await
        .expect("start Cassandra (Docker?)");

    let host = container.get_host().await.unwrap();
    let port = container.get_host_port_ipv4(9042).await.unwrap();
    let uri = format!("{host}:{port}");

    // Wait for CQL readiness
    let mut session = None;
    for _ in 0..60 {
        let c = tokio::time::timeout(
            std::time::Duration::from_secs(5),
            SessionBuilder::new().known_node(&uri).build(),
        )
        .await;
        if let Ok(Ok(s)) = c {
            let ping = tokio::time::timeout(
                std::time::Duration::from_secs(5),
                s.query_unpaged("SELECT now() FROM system.local", ()),
            )
            .await;
            if matches!(ping, Ok(Ok(_))) {
                session = Some(s);
                break;
            }
        }
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
    let session = session.expect("CQL not ready");

    this::storage::scylladb::ensure_schema(&session, "billing")
        .await
        .expect("schema");

    let app = scylladb_example::app(&uri, "billing").await.expect("build app");
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
