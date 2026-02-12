use axum::Json;
use axum::extract::{Path, State};
use std::sync::Arc;
use this::prelude::*;

use super::{Product, ProductStore};

#[derive(Clone)]
pub struct ProductState {
    pub store: Arc<dyn ProductStore>,
    pub entity_creator: Arc<dyn EntityCreator>,
}

pub async fn list_products(State(state): State<ProductState>) -> Json<serde_json::Value> {
    match state.store.list().await {
        Ok(items) => Json(serde_json::to_value(items).unwrap_or_else(|_| serde_json::json!([]))),
        Err(e) => {
            eprintln!("List products error: {:?}", e);
            Json(serde_json::json!({
                "error": "Failed to list products",
                "details": e.to_string()
            }))
        }
    }
}

pub async fn get_product(
    State(state): State<ProductState>,
    Path(id): Path<Uuid>,
) -> Json<Option<Product>> {
    let item = state.store.get(&id).await.ok();
    Json(item)
}

pub async fn create_product(
    State(state): State<ProductState>,
    Json(entity_data): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    eprintln!("Creating product with data: {:?}", entity_data);
    match state.entity_creator.create_from_json(entity_data).await {
        Ok(created) => {
            eprintln!("Product created successfully: {:?}", created);
            Json(created)
        }
        Err(e) => {
            eprintln!("Create product error: {:?}", e);
            Json(serde_json::json!({
                "error": "Failed to create product",
                "details": e.to_string()
            }))
        }
    }
}

pub async fn update_product(
    State(state): State<ProductState>,
    Json(product): Json<Product>,
) -> Json<Option<Product>> {
    let updated = state.store.update(product).await.ok();
    Json(updated)
}

pub async fn delete_product(State(state): State<ProductState>, Path(id): Path<Uuid>) -> Json<bool> {
    let ok = state.store.delete(&id).await.is_ok();
    Json(ok)
}
