use anyhow::{anyhow, Result};
use serde_json::Value;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use uuid::Uuid;

use this::config::sinks::SinkType;
use this::core::events::{CognitiveSignal, EventBus, FrameworkEvent};
use this::events::bridge::cognitive::{CognitiveNotificationBridge, CognitiveNotificationConfig};
use this::events::{Sink, SinkRegistry};
use this::prelude::async_trait;

/// In-memory sink that records every notification the bridge delivers to it.
///
/// The default cognitive config routes to the `in_app` sink. Nothing else in
/// this demo registers one, so this sink is registered under that name: it
/// lets the demo (and its test) observe what was actually routed.
#[derive(Debug, Default)]
pub struct CollectingSink {
    received: Mutex<Vec<Value>>,
}

impl CollectingSink {
    /// Signal types (e.g. `anomaly_detected`) of the notifications received so far.
    pub fn signal_types(&self) -> Vec<String> {
        self.received
            .lock()
            .unwrap()
            .iter()
            .filter_map(|p| {
                p.get("signal_type")
                    .and_then(Value::as_str)
                    .map(String::from)
            })
            .collect()
    }
}

#[async_trait]
impl Sink for CollectingSink {
    async fn deliver(
        &self,
        payload: Value,
        _recipient_id: Option<&str>,
        _context_vars: &std::collections::HashMap<String, Value>,
    ) -> Result<()> {
        self.received.lock().unwrap().push(payload);
        Ok(())
    }

    fn name(&self) -> &str {
        "in_app"
    }

    fn sink_type(&self) -> SinkType {
        SinkType::InApp
    }
}

/// Wait (bounded) until the sink has received at least `expected` notifications.
async fn wait_for_deliveries(sink: &CollectingSink, expected: usize) -> Result<()> {
    tokio::time::timeout(Duration::from_secs(2), async {
        while sink.signal_types().len() < expected {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .map_err(|_| anyhow!("expected {expected} notification(s), none routed within 2s"))
}

/// Run the cognitive-signals demo and return the signal types the sink received.
///
/// Publishes one signal above its threshold (routed) and one below (filtered).
/// Errors if the above-threshold notification is not delivered.
pub async fn run() -> Result<Vec<String>> {
    let event_bus = Arc::new(EventBus::new(1024));
    let sink = Arc::new(CollectingSink::default());
    let sink_registry = Arc::new(SinkRegistry::new());
    sink_registry.register("in_app", sink.clone());

    let config = CognitiveNotificationConfig::default();
    let bridge = CognitiveNotificationBridge::new(event_bus.clone(), sink_registry, config);
    let handle = bridge.start();
    // `start()` spawns the listener task, which subscribes to the EventBus only
    // when it first runs. Let it run before publishing, or the signal is sent to
    // no subscriber and silently dropped.
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Above threshold (0.8 > 0.5) -> must be routed to the sink.
    event_bus.publish(FrameworkEvent::Cognitive(
        CognitiveSignal::AnomalyDetected {
            node_id: Uuid::new_v4(),
            anomaly_type: "price_spike".to_string(),
            score: 0.8,
        },
    ));
    wait_for_deliveries(&sink, 1).await?;

    // Below threshold (0.3 < 0.7) -> must be filtered out.
    event_bus.publish(FrameworkEvent::Cognitive(
        CognitiveSignal::CoChangeDetected {
            nodes: vec![Uuid::new_v4(), Uuid::new_v4()],
            strength: 0.3,
        },
    ));
    tokio::time::sleep(Duration::from_millis(50)).await;

    handle.abort();
    Ok(sink.signal_types())
}
