use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let app = multi_module_example::app().await?;
    println!("\n🌐 Multi-Module Server running on http://127.0.0.1:4242");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:4242").await?;
    axum::serve(listener, app).await?;
    Ok(())
}
