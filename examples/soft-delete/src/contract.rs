//! The soft-delete contract, stated once and run against any billing store.
//!
//! The guarded path (GraphQL, gRPC, link routes) is `SoftDeleteCreator` over the store's creator,
//! read through `SoftDeleteFetcher`. This module runs exactly those two wrappers over the stores a
//! backend hands in, so each backend is measured by the same assertions rather than its own copy.

use std::sync::Arc;

use anyhow::{ensure, Context, Result};
use billing::{BillingModule, BillingStores};
use serde_json::json;
use this::core::module::{EntityCreator, EntityFetcher, Module};
use this::core::soft_delete::{SoftDeleteCreator, SoftDeleteFetcher};
use uuid::Uuid;

/// Runs the contract on orders and invoices. Any error names the entity and the step that failed.
pub async fn assert_soft_delete_contract(stores: BillingStores) -> Result<()> {
    let module = BillingModule::new(stores);
    for entity in ["order", "invoice"] {
        check(&module, entity)
            .await
            .with_context(|| format!("soft delete contract on `{entity}`"))?;
    }
    Ok(())
}

async fn check(module: &BillingModule, entity: &str) -> Result<()> {
    let creator: Arc<dyn EntityCreator> = module
        .get_entity_creator(entity)
        .context("the module exposes no creator")?;
    let fetcher: Arc<dyn EntityFetcher> = module
        .get_entity_fetcher(entity)
        .context("the module exposes no fetcher")?;

    let created = creator
        .create_from_json(json!({
            "name": "contract",
            "status": "pending",
            "number": format!("SD-{}", Uuid::new_v4().simple()),
            "amount": 1.0,
        }))
        .await
        .context("create")?;
    let id: Uuid = serde_json::from_value(created["id"].clone()).context("the created id")?;
    fetcher
        .fetch_as_json(&id)
        .await
        .context("the created row must be readable before the delete")?;

    SoftDeleteCreator::new(creator.clone(), fetcher.clone())
        .delete(&id)
        .await
        .context("soft delete must succeed on a store that carries the mark")?;

    // The row is kept, and it carries the mark.
    let stored = fetcher
        .fetch_as_json(&id)
        .await
        .context("the row must still be stored after a soft delete")?;
    ensure!(
        stored["deleted_at"].is_string(),
        "the stored row must carry `deleted_at`, found {}",
        stored["deleted_at"]
    );

    // Guarded reads no longer return it.
    ensure!(
        SoftDeleteFetcher::new(fetcher.clone())
            .fetch_as_json(&id)
            .await
            .is_err(),
        "a marked row must be hidden from guarded reads"
    );
    Ok(())
}
