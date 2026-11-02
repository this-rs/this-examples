use anyhow::Result;
use std::net::SocketAddr;

use oidc_example::idp::{claims, MockIdp};
use oidc_example::{app, AUDIENCE};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().with_env_filter("info").init();

    // The IdP on a fixed port, so the example is reproducible from the command line.
    let idp = MockIdp::start_on("127.0.0.1:4301".parse::<SocketAddr>()?).await?;
    let router = app(&idp.discovery_url()).await?;

    // A token a client would obtain from the IdP, valid for 15 minutes.
    let token = idp.mint(&claims(idp.issuer(), AUDIENCE, "alice", 900));

    println!("\n  OIDC Example — resource server on http://127.0.0.1:4300");
    println!("  Mock IdP        on {}", idp.issuer());
    println!("\n  curl -H \"Authorization: Bearer {token}\" http://127.0.0.1:4300/orders\n");

    let listener = tokio::net::TcpListener::bind("127.0.0.1:4300").await?;
    axum::serve(listener, router).await?;
    Ok(())
}
