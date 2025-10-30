use std::sync::Arc;
use axum::extract::{Path, State};
use axum::Json;
use this::prelude::*;

use super::{Order, OrderStore};

#[derive(Clone)]
pub struct OrderState { pub store: Arc<dyn OrderStore> }

pub async fn list_orders(State(state): State<OrderState>) -> Json<Vec<Order>> {
    let items = state.store.list().await.unwrap_or_default();
    Json(items)
}

pub async fn get_order(State(state): State<OrderState>, Path(id): Path<Uuid>) -> Json<Option<Order>> {
    let item = state.store.get(&id).await.ok();
    Json(item)
}

pub async fn create_order(State(state): State<OrderState>, Json(order): Json<Order>) -> Json<Order> {
    let created = state.store.create(order.clone()).await.unwrap_or(order);
    Json(created)
}

pub async fn update_order(State(state): State<OrderState>, Json(order): Json<Order>) -> Json<Option<Order>> {
    let updated = state.store.update(order).await.ok();
    Json(updated)
}

pub async fn delete_order(State(state): State<OrderState>, Path(id): Path<Uuid>) -> Json<bool> {
    let ok = state.store.delete(&id).await.is_ok();
    Json(ok)
}
