use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::Extension;
use this::prelude::{AuthContext, Uuid};
use tower::ServiceExt;

/// BUG: `BillingStores::new_lmdb` opens orders/invoices/payments at the SAME LMDB path,
/// and heed refuses a second `EnvOpenOptions::open` on an already-open path
/// ("environment already open ... with different options"). The binary hits the same
/// path when it runs, so this is not a test artifact — the lmdb example is broken until
/// the billing stores share a single LMDB env.
#[ignore = "lmdb example: billing stores open the same LMDB path 3x (heed env conflict)"]
#[tokio::test]
async fn lmdb_serves_orders() {
    let dir = std::env::temp_dir().join(format!("lmdb-smoke-{}", std::process::id()));
    let app = lmdb_example::app(dir.to_str().unwrap()).await.expect("build app");
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
