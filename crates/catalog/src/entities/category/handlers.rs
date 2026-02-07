use axum::Json;
use axum::extract::{Path, State};
use std::sync::Arc;
use this::prelude::*;

use super::{Category, CategoryStore};

#[derive(Clone)]
pub struct CategoryState {
    pub store: Arc<dyn CategoryStore>,
    pub entity_creator: Arc<dyn EntityCreator>,
}

pub async fn list_categories(State(state): State<CategoryState>) -> Json<serde_json::Value> {
    match state.store.list().await {
        Ok(items) => Json(serde_json::to_value(items).unwrap_or_else(|_| serde_json::json!([]))),
        Err(e) => {
            eprintln!("List categories error: {:?}", e);
            Json(serde_json::json!({
                "error": "Failed to list categories",
                "details": e.to_string()
            }))
        }
    }
}

pub async fn get_category(
    State(state): State<CategoryState>,
    Path(id): Path<Uuid>,
) -> Json<Option<Category>> {
    let item = state.store.get(&id).await.ok();
    Json(item)
}

pub async fn create_category(
    State(state): State<CategoryState>,
    Json(entity_data): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    eprintln!("Creating category with data: {:?}", entity_data);
    match state.entity_creator.create_from_json(entity_data).await {
        Ok(created) => {
            eprintln!("Category created successfully: {:?}", created);
            Json(created)
        }
        Err(e) => {
            eprintln!("Create category error: {:?}", e);
            Json(serde_json::json!({
                "error": "Failed to create category",
                "details": e.to_string()
            }))
        }
    }
}

pub async fn update_category(
    State(state): State<CategoryState>,
    Json(category): Json<Category>,
) -> Json<Option<Category>> {
    let updated = state.store.update(category).await.ok();
    Json(updated)
}

pub async fn delete_category(
    State(state): State<CategoryState>,
    Path(id): Path<Uuid>,
) -> Json<bool> {
    let ok = state.store.delete(&id).await.is_ok();
    Json(ok)
}





