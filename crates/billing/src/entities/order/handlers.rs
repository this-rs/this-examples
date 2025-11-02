use axum::Json;
use axum::extract::{Path, State};
use std::sync::Arc;
use this::prelude::*;

use super::{Order, OrderStore};

#[derive(Clone)]
pub struct OrderState {
    pub store: Arc<dyn OrderStore>,
    pub entity_creator: Arc<dyn EntityCreator>,
}

pub async fn list_orders(State(state): State<OrderState>) -> Json<serde_json::Value> {
    match state.store.list().await {
        Ok(items) => Json(serde_json::to_value(items).unwrap_or_else(|_| serde_json::json!([]))),
        Err(e) => {
            eprintln!("List orders error: {:?}", e);
            Json(serde_json::json!({
                "error": "Failed to list orders",
                "details": e.to_string()
            }))
        }
    }
}

pub async fn get_order(
    State(state): State<OrderState>,
    Path(id): Path<Uuid>,
) -> Json<Option<Order>> {
    let item = state.store.get(&id).await.ok();
    Json(item)
}

pub async fn create_order(
    State(state): State<OrderState>,
    Json(entity_data): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    eprintln!("Creating order with data: {:?}", entity_data);
    match state.entity_creator.create_from_json(entity_data).await {
        Ok(created) => {
            eprintln!("Order created successfully: {:?}", created);
            Json(created)
        },
        Err(e) => {
            eprintln!("Create order error: {:?}", e);
            Json(serde_json::json!({
                "error": "Failed to create order",
                "details": e.to_string()
            }))
        }
    }
}

pub async fn update_order(
    State(state): State<OrderState>,
    Json(order): Json<Order>,
) -> Json<Option<Order>> {
    let updated = state.store.update(order).await.ok();
    Json(updated)
}

pub async fn delete_order(State(state): State<OrderState>, Path(id): Path<Uuid>) -> Json<bool> {
    let ok = state.store.delete(&id).await.is_ok();
    Json(ok)
}
