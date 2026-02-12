use axum::Json;
use axum::extract::{Path, State};
use std::sync::Arc;
use this::prelude::*;

use super::{Activity, ActivityStore};

#[derive(Clone)]
pub struct ActivityState {
    pub store: Arc<dyn ActivityStore>,
    pub entity_creator: Arc<dyn EntityCreator>,
}

pub async fn list_activities(State(state): State<ActivityState>) -> Json<serde_json::Value> {
    match state.store.list().await {
        Ok(items) => Json(serde_json::to_value(items).unwrap_or_else(|_| serde_json::json!([]))),
        Err(e) => {
            eprintln!("List activities error: {:?}", e);
            Json(serde_json::json!({
                "error": "Failed to list activities",
                "details": e.to_string()
            }))
        }
    }
}

pub async fn get_activity(
    State(state): State<ActivityState>,
    Path(id): Path<Uuid>,
) -> Json<Option<Activity>> {
    let item = state.store.get(&id).await.ok();
    Json(item)
}

pub async fn create_activity(
    State(state): State<ActivityState>,
    Json(entity_data): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    eprintln!("Creating activity with data: {:?}", entity_data);
    match state.entity_creator.create_from_json(entity_data).await {
        Ok(created) => {
            eprintln!("Activity created successfully: {:?}", created);
            Json(created)
        }
        Err(e) => {
            eprintln!("Create activity error: {:?}", e);
            Json(serde_json::json!({
                "error": "Failed to create activity",
                "details": e.to_string()
            }))
        }
    }
}

pub async fn update_activity(
    State(state): State<ActivityState>,
    Json(activity): Json<Activity>,
) -> Json<Option<Activity>> {
    let updated = state.store.update(activity).await.ok();
    Json(updated)
}

pub async fn delete_activity(
    State(state): State<ActivityState>,
    Path(id): Path<Uuid>,
) -> Json<bool> {
    let ok = state.store.delete(&id).await.is_ok();
    Json(ok)
}
