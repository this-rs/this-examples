use rest_example::app;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let app = app().await?;

    println!("🚀 Server running on http://0.0.0.0:4242");

    let listener = tokio::net::TcpListener::bind("0.0.0.0:4242").await?;
    axum::serve(listener, app).await?;

    Ok(())
}
