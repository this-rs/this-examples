use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    #[cfg(feature = "graphql")]
    {
        let app = graphql_example::app().await?;
        println!("\n🌐 Server running on http://127.0.0.1:4242");
        let listener = tokio::net::TcpListener::bind("127.0.0.1:4242").await?;
        axum::serve(listener, app).await?;
    }

    #[cfg(not(feature = "graphql"))]
    {
        eprintln!("❌ La feature GraphQL n'est pas activée !");
        eprintln!("   Lancez avec: cargo run -p graphql_example --features graphql");
    }

    Ok(())
}
