use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let app = event_flows_example::app().await?;
    println!("\n🚀 Event Flows Example running on http://127.0.0.1:4245");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:4245").await?;
    axum::serve(listener, app).await?;
    Ok(())
}
