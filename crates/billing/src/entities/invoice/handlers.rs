use axum::Json;
use axum::extract::{Path, State};
use std::sync::Arc;
use this::prelude::*;

use super::{Invoice, InvoiceStore};

#[derive(Clone)]
pub struct InvoiceState {
    pub store: Arc<dyn InvoiceStore>,
    pub entity_creator: Arc<dyn EntityCreator>,
}

pub async fn list_invoices(State(state): State<InvoiceState>) -> Json<serde_json::Value> {
    match state.store.list().await {
        Ok(items) => Json(serde_json::to_value(items).unwrap_or_else(|_| serde_json::json!([]))),
        Err(e) => {
            eprintln!("List invoices error: {:?}", e);
            Json(serde_json::json!({
                "error": "Failed to list invoices",
                "details": e.to_string()
            }))
        }
    }
}

pub async fn get_invoice(
    State(state): State<InvoiceState>,
    Path(id): Path<Uuid>,
) -> Json<Option<Invoice>> {
    let item = state.store.get(&id).await.ok();
    Json(item)
}

pub async fn create_invoice(
    State(state): State<InvoiceState>,
    Json(entity_data): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    eprintln!("Creating invoice with data: {:?}", entity_data);
    match state.entity_creator.create_from_json(entity_data).await {
        Ok(created) => {
            eprintln!("Invoice created successfully: {:?}", created);
            Json(created)
        }
        Err(e) => {
            eprintln!("Create invoice error: {:?}", e);
            Json(serde_json::json!({
                "error": "Failed to create invoice",
                "details": e.to_string()
            }))
        }
    }
}

pub async fn update_invoice(
    State(state): State<InvoiceState>,
    Json(invoice): Json<Invoice>,
) -> Json<Option<Invoice>> {
    let updated = state.store.update(invoice).await.ok();
    Json(updated)
}

pub async fn delete_invoice(State(state): State<InvoiceState>, Path(id): Path<Uuid>) -> Json<bool> {
    let ok = state.store.delete(&id).await.is_ok();
    Json(ok)
}
