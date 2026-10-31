use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().with_env_filter("info").init();
    println!("\n  Cognitive Signals Example\n");
    cognitive_signals_example::run().await
}
