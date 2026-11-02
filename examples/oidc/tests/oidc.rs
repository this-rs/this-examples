//! The resource server accepts exactly the tokens its IdP signed for it, and refuses every other.
//!
//! Each refusal is a separate test, so a regression names the check it broke: a wrong audience
//! is not a wrong signature, and "everything returns 401" would pass every one of them.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use oidc_example::idp::{claims, MockIdp};
use oidc_example::{app, AUDIENCE};
use tower::ServiceExt;

async fn get_orders(router: &axum::Router, bearer: Option<&str>) -> StatusCode {
    let mut req = Request::builder().uri("/orders");
    if let Some(token) = bearer {
        req = req.header("authorization", format!("Bearer {token}"));
    }
    router
        .clone()
        .oneshot(req.body(Body::empty()).unwrap())
        .await
        .expect("request")
        .status()
}

async fn setup() -> (MockIdp, axum::Router) {
    let idp = MockIdp::start_on("127.0.0.1:0".parse().unwrap())
        .await
        .expect("mock IdP");
    let router = app(&idp.discovery_url()).await.expect("resource server");
    (idp, router)
}

/// Authentication is armed: without a token, nothing opens.
#[tokio::test]
async fn no_token_is_refused() {
    let (_idp, router) = setup().await;
    assert_eq!(get_orders(&router, None).await, StatusCode::UNAUTHORIZED);
}

/// The one case that must pass: the IdP's token, for this audience, from this issuer.
#[tokio::test]
async fn a_token_from_the_idp_for_this_audience_opens_the_surface() {
    let (idp, router) = setup().await;
    let token = idp.mint(&claims(idp.issuer(), AUDIENCE, "alice", 900));
    assert_ne!(
        get_orders(&router, Some(&token)).await,
        StatusCode::UNAUTHORIZED,
        "a valid IdP token must be accepted"
    );
}

/// A token minted for another API. `aud` names this resource server or it is refused.
#[tokio::test]
async fn a_token_for_another_audience_is_refused() {
    let (idp, router) = setup().await;
    let token = idp.mint(&claims(idp.issuer(), "billing-api", "alice", 900));
    assert_eq!(
        get_orders(&router, Some(&token)).await,
        StatusCode::UNAUTHORIZED
    );
}

/// A token from another issuer. `iss` must be the discovered issuer, not merely a trusted key.
#[tokio::test]
async fn a_token_from_another_issuer_is_refused() {
    let (idp, router) = setup().await;
    let token = idp.mint(&claims("https://elsewhere.example", AUDIENCE, "alice", 900));
    assert_eq!(
        get_orders(&router, Some(&token)).await,
        StatusCode::UNAUTHORIZED
    );
}

/// Right issuer, right audience, wrong key: the signature is what makes the token the IdP's.
#[tokio::test]
async fn a_forged_signature_is_refused() {
    let (idp, router) = setup().await;
    let token = idp.mint_rogue(&claims(idp.issuer(), AUDIENCE, "alice", 900));
    assert_eq!(
        get_orders(&router, Some(&token)).await,
        StatusCode::UNAUTHORIZED
    );
}

/// An expired token, signed correctly. Tolerance is 60 s, so this one is well past it.
#[tokio::test]
async fn an_expired_token_is_refused() {
    let (idp, router) = setup().await;
    let token = idp.mint(&claims(idp.issuer(), AUDIENCE, "alice", -3600));
    assert_eq!(
        get_orders(&router, Some(&token)).await,
        StatusCode::UNAUTHORIZED
    );
}

/// A valid signature, audience and issuer are not enough: the token must name a tenant.
#[tokio::test]
async fn a_token_without_a_tenant_is_refused() {
    let (idp, router) = setup().await;
    let mut c = claims(idp.issuer(), AUDIENCE, "alice", 900);
    c.as_object_mut().unwrap().remove("tenant_id");
    let token = idp.mint(&c);
    assert_eq!(
        get_orders(&router, Some(&token)).await,
        StatusCode::UNAUTHORIZED
    );
}

/// The resource server must know who it is: an OIDC section without an audience does not start.
#[test]
fn oidc_without_an_audience_refuses_to_start() {
    let mut cfg = oidc_example::auth_config("http://127.0.0.1:1/.well-known/openid-configuration");
    cfg.oidc.as_mut().expect("oidc section").audience = None;
    let built = this::server::builder::ServerBuilder::new()
        .with_auth_config(cfg)
        .build_host();
    assert!(
        built.is_err(),
        "an OIDC section without `audience` must not build a host"
    );
}
