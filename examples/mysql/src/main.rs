use anyhow::Result;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "mysql://billing:billing@localhost:3306/billing".to_string());
    let app = mysql_example::app(&database_url).await?;
    println!("\n🌐 Server running on http://0.0.0.0:4242");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:4242").await?;
    axum::serve(listener, app).await?;
    Ok(())
}
