use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().with_env_filter("info").init();
    println!("\n  Cognitive Signals Example\n");
    let delivered = cognitive_signals_example::run().await?;
    println!("  Delivered to in_app: {delivered:?}");
    Ok(())
}
