use anyhow::Result;
use std::sync::Arc;
use uuid::Uuid;

use this::core::events::{CognitiveSignal, EventBus, FrameworkEvent};
use this::events::bridge::cognitive::{CognitiveNotificationBridge, CognitiveNotificationConfig};
use this::events::SinkRegistry;

/// Run the cognitive-signals demo: set up the bridge and publish sample signals
/// (one above threshold -> routed, one below -> filtered).
pub async fn run() -> Result<()> {
    let event_bus = Arc::new(EventBus::new(1024));
    let sink_registry = Arc::new(SinkRegistry::new());
    let config = CognitiveNotificationConfig::default();
    let bridge = CognitiveNotificationBridge::new(event_bus.clone(), sink_registry, config);
    let _handle = bridge.start();

    // Above threshold (0.8 > 0.5) -> routed
    event_bus.publish(FrameworkEvent::Cognitive(
        CognitiveSignal::AnomalyDetected {
            node_id: Uuid::new_v4(),
            anomaly_type: "price_spike".to_string(),
            score: 0.8,
        },
    ));
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    // Below threshold (0.3 < 0.7) -> filtered
    event_bus.publish(FrameworkEvent::Cognitive(
        CognitiveSignal::CoChangeDetected {
            nodes: vec![Uuid::new_v4(), Uuid::new_v4()],
            strength: 0.3,
        },
    ));
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    Ok(())
}
