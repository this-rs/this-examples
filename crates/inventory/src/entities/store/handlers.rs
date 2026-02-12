use axum::Json;
use axum::extract::{Path, State};
use std::sync::Arc;
use this::prelude::*;

use super::{Store, StoreStore};

#[derive(Clone)]
pub struct StoreState {
    pub store: Arc<dyn StoreStore>,
    pub entity_creator: Arc<dyn EntityCreator>,
}

pub async fn list_stores(State(state): State<StoreState>) -> Json<serde_json::Value> {
    match state.store.list().await {
        Ok(items) => Json(serde_json::to_value(items).unwrap_or_else(|_| serde_json::json!([]))),
        Err(e) => {
            eprintln!("List stores error: {:?}", e);
            Json(serde_json::json!({
                "error": "Failed to list stores",
                "details": e.to_string()
            }))
        }
    }
}

pub async fn get_store(
    State(state): State<StoreState>,
    Path(id): Path<Uuid>,
) -> Json<Option<Store>> {
    let item = state.store.get(&id).await.ok();
    Json(item)
}

pub async fn create_store(
    State(state): State<StoreState>,
    Json(entity_data): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    eprintln!("Creating store with data: {:?}", entity_data);
    match state.entity_creator.create_from_json(entity_data).await {
        Ok(created) => {
            eprintln!("Store created successfully: {:?}", created);
            Json(created)
        }
        Err(e) => {
            eprintln!("Create store error: {:?}", e);
            Json(serde_json::json!({
                "error": "Failed to create store",
                "details": e.to_string()
            }))
        }
    }
}

pub async fn update_store(
    State(state): State<StoreState>,
    Json(store): Json<Store>,
) -> Json<Option<Store>> {
    let updated = state.store.update(store).await.ok();
    Json(updated)
}

pub async fn delete_store(State(state): State<StoreState>, Path(id): Path<Uuid>) -> Json<bool> {
    let ok = state.store.delete(&id).await.is_ok();
    Json(ok)
}
