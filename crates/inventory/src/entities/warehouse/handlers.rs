use axum::Json;
use axum::extract::{Path, State};
use std::sync::Arc;
use this::prelude::*;

use super::{Warehouse, WarehouseStore};

#[derive(Clone)]
pub struct WarehouseState {
    pub store: Arc<dyn WarehouseStore>,
    pub entity_creator: Arc<dyn EntityCreator>,
}

pub async fn list_warehouses(State(state): State<WarehouseState>) -> Json<serde_json::Value> {
    match state.store.list().await {
        Ok(items) => Json(serde_json::to_value(items).unwrap_or_else(|_| serde_json::json!([]))),
        Err(e) => {
            eprintln!("List warehouses error: {:?}", e);
            Json(serde_json::json!({
                "error": "Failed to list warehouses",
                "details": e.to_string()
            }))
        }
    }
}

pub async fn get_warehouse(
    State(state): State<WarehouseState>,
    Path(id): Path<Uuid>,
) -> Json<Option<Warehouse>> {
    let item = state.store.get(&id).await.ok();
    Json(item)
}

pub async fn create_warehouse(
    State(state): State<WarehouseState>,
    Json(entity_data): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    eprintln!("Creating warehouse with data: {:?}", entity_data);
    match state.entity_creator.create_from_json(entity_data).await {
        Ok(created) => {
            eprintln!("Warehouse created successfully: {:?}", created);
            Json(created)
        }
        Err(e) => {
            eprintln!("Create warehouse error: {:?}", e);
            Json(serde_json::json!({
                "error": "Failed to create warehouse",
                "details": e.to_string()
            }))
        }
    }
}

pub async fn update_warehouse(
    State(state): State<WarehouseState>,
    Json(warehouse): Json<Warehouse>,
) -> Json<Option<Warehouse>> {
    let updated = state.store.update(warehouse).await.ok();
    Json(updated)
}

pub async fn delete_warehouse(
    State(state): State<WarehouseState>,
    Path(id): Path<Uuid>,
) -> Json<bool> {
    let ok = state.store.delete(&id).await.is_ok();
    Json(ok)
}





