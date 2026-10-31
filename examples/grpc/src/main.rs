use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let app = grpc_example::app().await?;
    println!("\n🌐 Server running on http://127.0.0.1:4244");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:4244").await?;
    axum::serve(listener, app).await?;
    Ok(())
}
