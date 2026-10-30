use anyhow::Result;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let data_dir = env::var("LMDB_DATA_DIR").unwrap_or_else(|_| "./data".to_string());
    let app = lmdb_example::app(&data_dir).await?;

    println!("\n🌐 Server running on http://0.0.0.0:4242");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:4242").await?;
    axum::serve(listener, app).await?;

    Ok(())
}
