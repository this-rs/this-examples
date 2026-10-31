#[tokio::test]
async fn cognitive_signals_runs() {
    cognitive_signals_example::run().await.expect("demo runs");
}
