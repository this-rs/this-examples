use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let app = websocket_example::app().await?;
    println!("\n🌐 Server running on http://127.0.0.1:4243");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:4243").await?;
    axum::serve(listener, app).await?;
    Ok(())
}
