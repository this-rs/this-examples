#[tokio::test]
async fn cognitive_signals_routes_above_threshold_only() {
    let delivered = cognitive_signals_example::run()
        .await
        .expect("above-threshold anomaly must be routed to the in_app sink");

    assert!(
        delivered.iter().any(|t| t == "anomaly_detected"),
        "expected an anomaly_detected notification, got {delivered:?}"
    );
    assert!(
        !delivered.iter().any(|t| t == "co_change_detected"),
        "co_change_detected (strength 0.3 < 0.7) must be filtered, got {delivered:?}"
    );
}
