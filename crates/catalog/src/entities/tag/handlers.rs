use axum::Json;
use axum::extract::{Path, State};
use std::sync::Arc;
use this::prelude::*;

use super::{Tag, TagStore};

#[derive(Clone)]
pub struct TagState {
    pub store: Arc<dyn TagStore>,
    pub entity_creator: Arc<dyn EntityCreator>,
}

pub async fn list_tags(State(state): State<TagState>) -> Json<serde_json::Value> {
    match state.store.list().await {
        Ok(items) => Json(serde_json::to_value(items).unwrap_or_else(|_| serde_json::json!([]))),
        Err(e) => {
            eprintln!("List tags error: {:?}", e);
            Json(serde_json::json!({
                "error": "Failed to list tags",
                "details": e.to_string()
            }))
        }
    }
}

pub async fn get_tag(State(state): State<TagState>, Path(id): Path<Uuid>) -> Json<Option<Tag>> {
    let item = state.store.get(&id).await.ok();
    Json(item)
}

pub async fn create_tag(
    State(state): State<TagState>,
    Json(entity_data): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    eprintln!("Creating tag with data: {:?}", entity_data);
    match state.entity_creator.create_from_json(entity_data).await {
        Ok(created) => {
            eprintln!("Tag created successfully: {:?}", created);
            Json(created)
        }
        Err(e) => {
            eprintln!("Create tag error: {:?}", e);
            Json(serde_json::json!({
                "error": "Failed to create tag",
                "details": e.to_string()
            }))
        }
    }
}

pub async fn update_tag(State(state): State<TagState>, Json(tag): Json<Tag>) -> Json<Option<Tag>> {
    let updated = state.store.update(tag).await.ok();
    Json(updated)
}

pub async fn delete_tag(State(state): State<TagState>, Path(id): Path<Uuid>) -> Json<bool> {
    let ok = state.store.delete(&id).await.is_ok();
    Json(ok)
}





