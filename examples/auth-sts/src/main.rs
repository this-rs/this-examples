use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().with_env_filter("info").init();
    let app = auth_sts_example::app().await?;
    println!("\n  Auth STS Example running on http://127.0.0.1:4300");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:4300").await?;
    axum::serve(listener, app).await?;
    Ok(())
}
