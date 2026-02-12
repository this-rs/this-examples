use axum::Json;
use axum::extract::{Path, State};
use std::sync::Arc;
use this::prelude::*;

use super::{Usage, UsageStore};

#[derive(Clone)]
pub struct UsageState {
    pub store: Arc<dyn UsageStore>,
    pub entity_creator: Arc<dyn EntityCreator>,
}

pub async fn list_usages(State(state): State<UsageState>) -> Json<serde_json::Value> {
    match state.store.list().await {
        Ok(items) => Json(serde_json::to_value(items).unwrap_or_else(|_| serde_json::json!([]))),
        Err(e) => {
            eprintln!("List usages error: {:?}", e);
            Json(serde_json::json!({
                "error": "Failed to list usages",
                "details": e.to_string()
            }))
        }
    }
}

pub async fn get_usage(
    State(state): State<UsageState>,
    Path(id): Path<Uuid>,
) -> Json<Option<Usage>> {
    let item = state.store.get(&id).await.ok();
    Json(item)
}

pub async fn create_usage(
    State(state): State<UsageState>,
    Json(entity_data): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    eprintln!("Creating usage with data: {:?}", entity_data);
    match state.entity_creator.create_from_json(entity_data).await {
        Ok(created) => {
            eprintln!("Usage created successfully: {:?}", created);
            Json(created)
        }
        Err(e) => {
            eprintln!("Create usage error: {:?}", e);
            Json(serde_json::json!({
                "error": "Failed to create usage",
                "details": e.to_string()
            }))
        }
    }
}

pub async fn update_usage(
    State(state): State<UsageState>,
    Json(usage): Json<Usage>,
) -> Json<Option<Usage>> {
    let updated = state.store.update(usage).await.ok();
    Json(updated)
}

pub async fn delete_usage(State(state): State<UsageState>, Path(id): Path<Uuid>) -> Json<bool> {
    let ok = state.store.delete(&id).await.is_ok();
    Json(ok)
}
