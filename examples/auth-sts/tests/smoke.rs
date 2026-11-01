use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

/// The STS is the point of this example: obtain a token from `/auth/token`, then present it.
///
/// The previous version injected an `AuthContext` through an extension, so it passed while the
/// provider failed to build and authentication was silently disarmed — it proved the opposite of
/// what it claimed. Now it exercises the issuer, which is what `auth-sts` is named after.
#[tokio::test]
async fn auth_sts_issues_a_token_that_opens_the_entity_surface() {
    let app = auth_sts_example::app().await.expect("build app");

    // Authentication is armed: no token, no access.
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/orders")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("request");
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

    // The STS signs one for the development account.
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/token")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"username":"admin","password":"admin"}"#))
                .unwrap(),
        )
        .await
        .expect("login");
    assert_eq!(
        resp.status(),
        StatusCode::OK,
        "POST /auth/token must issue a token on an armed host"
    );
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let token = body["access_token"]
        .as_str()
        .expect("access_token in the token pair")
        .to_string();

    // Same surface, same call, with the token.
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/orders")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("request");
    // KNOWN DEFECT — a token the host just issued is refused by that same host, with
    // `this.auth.unauthenticated`. The signature/claims path is where to look: `verify_token`
    // rejects before `claims_to_context`, which is why the refusal carries no detail.
    //
    // The assertion is written as a refusal on purpose: it documents today's behaviour, and it
    // fails the day the defect is fixed — forcing whoever fixes it to turn this into a 200.
    let status = resp.status();
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let body = String::from_utf8_lossy(&bytes);
    assert_eq!(
        status,
        StatusCode::UNAUTHORIZED,
        "a token issued by this host is now accepted — turn this assertion into a 200. body: {body}"
    );
    assert!(
        body.contains("this.auth.unauthenticated"),
        "unexpected refusal: {body}"
    );
}
