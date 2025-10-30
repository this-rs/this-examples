use std::sync::Arc;
use axum::extract::{Path, State};
use axum::Json;
use this::prelude::*;

use super::{Payment, PaymentStore};

#[derive(Clone)]
pub struct PaymentState { pub store: Arc<dyn PaymentStore> }

pub async fn list_payments(State(state): State<PaymentState>) -> Json<Vec<Payment>> {
    let items = state.store.list().await.unwrap_or_default();
    Json(items)
}

pub async fn get_payment(State(state): State<PaymentState>, Path(id): Path<Uuid>) -> Json<Option<Payment>> {
    let item = state.store.get(&id).await.ok();
    Json(item)
}

pub async fn create_payment(State(state): State<PaymentState>, Json(payment): Json<Payment>) -> Json<Payment> {
    let created = state.store.create(payment.clone()).await.unwrap_or(payment);
    Json(created)
}

pub async fn update_payment(State(state): State<PaymentState>, Json(payment): Json<Payment>) -> Json<Option<Payment>> {
    let updated = state.store.update(payment).await.ok();
    Json(updated)
}

pub async fn delete_payment(State(state): State<PaymentState>, Path(id): Path<Uuid>) -> Json<bool> {
    let ok = state.store.delete(&id).await.is_ok();
    Json(ok)
}
