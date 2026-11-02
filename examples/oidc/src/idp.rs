//! A mock identity provider: the discovery document and JWKS a real IdP (Keycloak, Auth0, Entra,
//! Dex) would serve, plus the means to mint tokens it signs.
//!
//! Keys are generated **at start-up**, never committed: nothing secret lives in the repository, and
//! every run is a fresh issuer, as a real provider's rotated keys would be.

use anyhow::{Context, Result};
use axum::{routing::get, Json, Router};
use base64::{engine::general_purpose::STANDARD, Engine};
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use ring::{
    rand::SystemRandom,
    signature::{Ed25519KeyPair, KeyPair},
};
use serde_json::{json, Value};
use std::net::SocketAddr;
use std::sync::Arc;
use this::core::auth::sts::StsService;

/// The tenant every token carries. `this` refuses a token without a tenant UUID.
pub const TENANT_ID: &str = "00000000-0000-4000-8000-0000000000a1";

/// A fresh Ed25519 key: the PKCS#8 document that signs, and the SPKI PEM that verifies.
struct Keys {
    signing: EncodingKey,
    public_pem: String,
}

fn generate_keys() -> Result<Keys> {
    let rng = SystemRandom::new();
    let pkcs8 =
        Ed25519KeyPair::generate_pkcs8(&rng).map_err(|_| anyhow::anyhow!("key generation"))?;
    let pair =
        Ed25519KeyPair::from_pkcs8(pkcs8.as_ref()).map_err(|_| anyhow::anyhow!("key parse"))?;

    // SubjectPublicKeyInfo for Ed25519 (RFC 8410): a fixed 12-byte prefix, then the 32-byte key.
    let mut spki = vec![
        0x30, 0x2a, 0x30, 0x05, 0x06, 0x03, 0x2b, 0x65, 0x70, 0x03, 0x21, 0x00,
    ];
    spki.extend_from_slice(pair.public_key().as_ref());
    let body = STANDARD.encode(&spki);
    let wrapped: Vec<&str> = body
        .as_bytes()
        .chunks(64)
        .map(|c| std::str::from_utf8(c).unwrap())
        .collect();
    let public_pem = format!(
        "-----BEGIN PUBLIC KEY-----\n{}\n-----END PUBLIC KEY-----\n",
        wrapped.join("\n")
    );

    Ok(Keys {
        signing: EncodingKey::from_ed_der(pkcs8.as_ref()),
        public_pem,
    })
}

/// A running mock IdP.
pub struct MockIdp {
    issuer: String,
    kid: String,
    keys: Keys,
    rogue: Keys,
}

impl MockIdp {
    /// Serve the IdP on `addr` (use port 0 for an ephemeral port).
    pub async fn start_on(addr: SocketAddr) -> Result<Self> {
        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .context("binding the mock IdP")?;
        let issuer = format!("http://{}", listener.local_addr()?);

        let keys = generate_keys()?;
        // Same JWKS shape as wami and every real provider: one OKP/Ed25519 key, `kid` = RFC 7638
        // thumbprint. The `kid` is read back from it so tokens name the key the JWKS publishes.
        let jwks = StsService::jwks_response(&keys.public_pem)?;
        let kid = jwks["keys"][0]["kid"]
            .as_str()
            .context("the JWKS names its key")?
            .to_string();

        let discovery = Arc::new(json!({
            "issuer": issuer,
            "jwks_uri": format!("{issuer}/jwks"),
        }));
        let jwks = Arc::new(jwks);
        let app = Router::new()
            .route(
                "/.well-known/openid-configuration",
                get({
                    let d = discovery.clone();
                    move || {
                        let d = d.clone();
                        async move { Json((*d).clone()) }
                    }
                }),
            )
            .route(
                "/jwks",
                get({
                    let j = jwks.clone();
                    move || {
                        let j = j.clone();
                        async move { Json((*j).clone()) }
                    }
                }),
            );
        tokio::spawn(async move { axum::serve(listener, app).await.expect("mock IdP serves") });

        Ok(Self {
            issuer,
            kid,
            keys,
            rogue: generate_keys()?,
        })
    }

    /// The discovery URL `this` is configured with (`auth.oidc.discovery_url`).
    pub fn discovery_url(&self) -> String {
        format!("{}/.well-known/openid-configuration", self.issuer)
    }

    /// The issuer this IdP stamps into `iss`.
    pub fn issuer(&self) -> &str {
        &self.issuer
    }

    /// Sign `claims` with the IdP's published key.
    pub fn mint(&self, claims: &Value) -> String {
        sign(&self.keys.signing, Some(&self.kid), claims)
    }

    /// Sign `claims` under this IdP's `kid` with a key it does not publish: a forged signature.
    pub fn mint_rogue(&self, claims: &Value) -> String {
        sign(&self.rogue.signing, Some(&self.kid), claims)
    }
}

fn sign(key: &EncodingKey, kid: Option<&str>, claims: &Value) -> String {
    let mut header = Header::new(Algorithm::EdDSA);
    header.kid = kid.map(str::to_string);
    encode(&header, claims, key).expect("token signs")
}

/// Claims for a well-formed access token, valid for `ttl_secs` from now.
pub fn claims(issuer: &str, audience: &str, subject: &str, ttl_secs: i64) -> Value {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock after epoch")
        .as_secs() as i64;
    json!({
        "iss": issuer,
        "aud": audience,
        "sub": subject,
        "iat": now,
        "exp": now + ttl_secs,
        "roles": ["reader"],
        // Required: a token with no tenant UUID is refused (auth.tenant.claim_field).
        "tenant_id": TENANT_ID,
    })
}
