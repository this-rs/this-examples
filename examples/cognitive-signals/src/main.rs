//! Cognitive Signals Example — Backend-agnostic cognitive notifications
//!
//! Demonstrates the CognitiveNotificationBridge which:
//! 1. Listens to EventBus for cognitive signal events
//! 2. Applies threshold-based rules per signal type
//! 3. Routes notifications to configured sinks
//!
//! This works with ANY storage backend (in-memory, postgres, etc.)
//! — it only depends on core types (EventBus, SinkRegistry, CognitiveSignal).
//!
//! # Running
//!
//! ```sh
//! cargo run -p cognitive_signals_example
//! ```

use std::sync::Arc;

use anyhow::Result;
use uuid::Uuid;

use this::core::events::{EventBus, FrameworkEvent, CognitiveSignal};
use this::events::bridge::cognitive::{
    CognitiveNotificationBridge, CognitiveNotificationConfig,
};
use this::events::SinkRegistry;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    println!("\n  Cognitive Signals Example");
    println!("  ========================\n");

    // 1. Create an EventBus — the backbone for all events
    let event_bus = Arc::new(EventBus::new(1024));

    // 2. Create a SinkRegistry (empty — no real sinks in this demo)
    let sink_registry = Arc::new(SinkRegistry::new());

    // 3. Configure cognitive notification rules
    //
    // Default thresholds (from CognitiveNotificationConfig::default()):
    //   - stigmergy_lock_in > 0.8  → notify
    //   - co_change > 0.7          → notify
    //   - anomaly > 0.5            → notify
    //   - scar / episode           → always notify (threshold 0.0)
    let config = CognitiveNotificationConfig::default();

    println!("  Default thresholds:");
    for (signal_type, rule) in &config.rules {
        match rule.threshold {
            Some(t) => println!("    {:<25} > {:.1}", signal_type, t),
            None => println!("    {:<25}   always", signal_type),
        }
    }
    println!();

    // 4. Start the bridge — it subscribes to EventBus in the background
    let bridge = CognitiveNotificationBridge::new(
        event_bus.clone(),
        sink_registry,
        config,
    );
    let _handle = bridge.start();

    println!("  Bridge started, listening for cognitive signals...\n");

    // 5. Simulate cognitive signals being published

    // 5a. Anomaly detected (score 0.8 > threshold 0.5) → should trigger notification
    println!("  [SEND] AnomalyDetected (score: 0.8 — above threshold 0.5)");
    event_bus.publish(FrameworkEvent::Cognitive(CognitiveSignal::AnomalyDetected {
        node_id: Uuid::new_v4(),
        anomaly_type: "price_spike".to_string(),
        score: 0.8,
    }));

    // Give the bridge time to process
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // 5b. Co-change signal (strength 0.3 < threshold 0.7) → should be filtered out
    println!("  [SEND] CoChangeDetected (strength: 0.3 — below threshold 0.7, filtered)");
    event_bus.publish(FrameworkEvent::Cognitive(CognitiveSignal::CoChangeDetected {
        nodes: vec![Uuid::new_v4(), Uuid::new_v4()],
        strength: 0.3,
    }));

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // 5c. Strong co-change (strength 0.9 > threshold 0.7) → should trigger
    println!("  [SEND] CoChangeDetected (strength: 0.9 — above threshold 0.7)");
    event_bus.publish(FrameworkEvent::Cognitive(CognitiveSignal::CoChangeDetected {
        nodes: vec![Uuid::new_v4(), Uuid::new_v4()],
        strength: 0.9,
    }));

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // 5d. Scar signal (threshold 0.0 → always notifies)
    println!("  [SEND] ScarCreated (always triggers — threshold 0.0)");
    event_bus.publish(FrameworkEvent::Cognitive(CognitiveSignal::ScarCreated {
        node_id: Uuid::new_v4(),
        scar_type: "repeated_failure".to_string(),
        description: "Payment processing failed 3 times in a row".to_string(),
    }));

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // 5e. Stigmergy lock-in (intensity 0.95 > threshold 0.8) → should trigger
    println!("  [SEND] StigmergyLockIn (intensity: 0.95 — above threshold 0.8)");
    event_bus.publish(FrameworkEvent::Cognitive(CognitiveSignal::StigmergyLockIn {
        path: vec![Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4()],
        intensity: 0.95,
    }));

    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    println!("\n  Summary:");
    println!("    5 signals sent, expected 4 notifications (1 filtered)");
    println!("\n  In a real server, these would be delivered via configured sinks:");
    println!("    - in-app   : NotificationStore (REST API at /notifications/:user_id)");
    println!("    - webhook  : HTTP POST to configured URL");
    println!("    - push     : Mobile push notifications");
    println!("\n  Done.");

    Ok(())
}
