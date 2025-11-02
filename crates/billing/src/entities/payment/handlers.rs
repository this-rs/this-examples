use axum::Json;
use axum::extract::{Path, State};
use std::sync::Arc;
use this::prelude::*;

use super::{Payment, PaymentStore};

#[derive(Clone)]
pub struct PaymentState {
    pub store: Arc<dyn PaymentStore>,
    pub entity_creator: Arc<dyn EntityCreator>,
}

pub async fn list_payments(State(state): State<PaymentState>) -> Json<serde_json::Value> {
    match state.store.list().await {
        Ok(items) => Json(serde_json::to_value(items).unwrap_or_else(|_| serde_json::json!([]))),
        Err(e) => {
            eprintln!("List payments error: {:?}", e);
            Json(serde_json::json!({
                "error": "Failed to list payments",
                "details": e.to_string()
            }))
        }
    }
}

pub async fn get_payment(
    State(state): State<PaymentState>,
    Path(id): Path<Uuid>,
) -> Json<Option<Payment>> {
    let item = state.store.get(&id).await.ok();
    Json(item)
}

pub async fn create_payment(
    State(state): State<PaymentState>,
    Json(entity_data): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    eprintln!("Creating payment with data: {:?}", entity_data);
    match state.entity_creator.create_from_json(entity_data).await {
        Ok(created) => {
            eprintln!("Payment created successfully: {:?}", created);
            Json(created)
        }
        Err(e) => {
            eprintln!("Create payment error: {:?}", e);
            Json(serde_json::json!({
                "error": "Failed to create payment",
                "details": e.to_string()
            }))
        }
    }
}

pub async fn update_payment(
    State(state): State<PaymentState>,
    Json(payment): Json<Payment>,
) -> Json<Option<Payment>> {
    let updated = state.store.update(payment).await.ok();
    Json(updated)
}

pub async fn delete_payment(State(state): State<PaymentState>, Path(id): Path<Uuid>) -> Json<bool> {
    let ok = state.store.delete(&id).await.is_ok();
    Json(ok)
}
