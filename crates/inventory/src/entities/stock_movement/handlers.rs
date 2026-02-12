use axum::Json;
use axum::extract::{Path, State};
use std::sync::Arc;
use this::prelude::*;

use super::{StockMovement, StockMovementStore};

#[derive(Clone)]
pub struct StockMovementState {
    pub store: Arc<dyn StockMovementStore>,
    pub entity_creator: Arc<dyn EntityCreator>,
}

pub async fn list_stock_movements(
    State(state): State<StockMovementState>,
) -> Json<serde_json::Value> {
    match state.store.list().await {
        Ok(items) => Json(serde_json::to_value(items).unwrap_or_else(|_| serde_json::json!([]))),
        Err(e) => {
            eprintln!("List stock_movements error: {:?}", e);
            Json(serde_json::json!({
                "error": "Failed to list stock_movements",
                "details": e.to_string()
            }))
        }
    }
}

pub async fn get_stock_movement(
    State(state): State<StockMovementState>,
    Path(id): Path<Uuid>,
) -> Json<Option<StockMovement>> {
    let item = state.store.get(&id).await.ok();
    Json(item)
}

pub async fn create_stock_movement(
    State(state): State<StockMovementState>,
    Json(entity_data): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    eprintln!("Creating stock_movement with data: {:?}", entity_data);
    match state.entity_creator.create_from_json(entity_data).await {
        Ok(created) => {
            eprintln!("StockMovement created successfully: {:?}", created);
            Json(created)
        }
        Err(e) => {
            eprintln!("Create stock_movement error: {:?}", e);
            Json(serde_json::json!({
                "error": "Failed to create stock_movement",
                "details": e.to_string()
            }))
        }
    }
}

pub async fn update_stock_movement(
    State(state): State<StockMovementState>,
    Json(stock_movement): Json<StockMovement>,
) -> Json<Option<StockMovement>> {
    let updated = state.store.update(stock_movement).await.ok();
    Json(updated)
}

pub async fn delete_stock_movement(
    State(state): State<StockMovementState>,
    Path(id): Path<Uuid>,
) -> Json<bool> {
    let ok = state.store.delete(&id).await.is_ok();
    Json(ok)
}
