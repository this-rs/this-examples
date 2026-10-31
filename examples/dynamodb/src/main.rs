use anyhow::Result;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let endpoint_url =
        env::var("AWS_ENDPOINT_URL").unwrap_or_else(|_| "http://localhost:8000".to_string());
    let app = dynamodb_example::app(&endpoint_url).await?;
    println!("\n🌐 Server running on http://0.0.0.0:4242");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:4242").await?;
    axum::serve(listener, app).await?;
    Ok(())
}
