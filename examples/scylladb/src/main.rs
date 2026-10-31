use anyhow::Result;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let scylla_uri = env::var("SCYLLA_URI").unwrap_or_else(|_| "localhost:9042".to_string());
    let keyspace = env::var("SCYLLA_KEYSPACE").unwrap_or_else(|_| "billing".to_string());
    let app = scylladb_example::app(&scylla_uri, &keyspace).await?;
    println!("\n🌐 Server running on http://0.0.0.0:4242");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:4242").await?;
    axum::serve(listener, app).await?;
    Ok(())
}
