use axum::Json;
use axum::extract::{Path, State};
use std::sync::Arc;
use this::prelude::*;

use super::{Invoice, InvoiceStore};

#[derive(Clone)]
pub struct InvoiceState {
    pub store: Arc<dyn InvoiceStore>,
}

pub async fn list_invoices(State(state): State<InvoiceState>) -> Json<Vec<Invoice>> {
    let items = state.store.list().await.unwrap_or_default();
    Json(items)
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
    Json(invoice): Json<Invoice>,
) -> Json<Invoice> {
    let created = state.store.create(invoice.clone()).await.unwrap_or(invoice);
    Json(created)
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
