use axum::Json;
use axum::extract::{Path, State};
use std::sync::Arc;
use this::prelude::*;

use super::{StockItem, StockItemStore};

#[derive(Clone)]
pub struct StockItemState {
    pub store: Arc<dyn StockItemStore>,
    pub entity_creator: Arc<dyn EntityCreator>,
}

pub async fn list_stock_items(State(state): State<StockItemState>) -> Json<serde_json::Value> {
    match state.store.list().await {
        Ok(items) => Json(serde_json::to_value(items).unwrap_or_else(|_| serde_json::json!([]))),
        Err(e) => {
            eprintln!("List stock_items error: {:?}", e);
            Json(serde_json::json!({
                "error": "Failed to list stock_items",
                "details": e.to_string()
            }))
        }
    }
}

pub async fn get_stock_item(
    State(state): State<StockItemState>,
    Path(id): Path<Uuid>,
) -> Json<Option<StockItem>> {
    let item = state.store.get(&id).await.ok();
    Json(item)
}

pub async fn create_stock_item(
    State(state): State<StockItemState>,
    Json(entity_data): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    eprintln!("Creating stock_item with data: {:?}", entity_data);
    match state.entity_creator.create_from_json(entity_data).await {
        Ok(created) => {
            eprintln!("StockItem created successfully: {:?}", created);
            Json(created)
        }
        Err(e) => {
            eprintln!("Create stock_item error: {:?}", e);
            Json(serde_json::json!({
                "error": "Failed to create stock_item",
                "details": e.to_string()
            }))
        }
    }
}

pub async fn update_stock_item(
    State(state): State<StockItemState>,
    Json(stock_item): Json<StockItem>,
) -> Json<Option<StockItem>> {
    let updated = state.store.update(stock_item).await.ok();
    Json(updated)
}

pub async fn delete_stock_item(
    State(state): State<StockItemState>,
    Path(id): Path<Uuid>,
) -> Json<bool> {
    let ok = state.store.delete(&id).await.is_ok();
    Json(ok)
}
