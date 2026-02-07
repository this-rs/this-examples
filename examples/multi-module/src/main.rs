use std::sync::Arc;

use anyhow::Result;

use billing::{BillingModule, BillingStores};
use catalog::{CatalogModule, CatalogStores};
use inventory::{InventoryModule, InventoryStores};
use test_data::{populate_catalog_data, populate_inventory_data, populate_test_data};

#[cfg(feature = "graphql")]
use axum::Router;
#[cfg(feature = "graphql")]
use this::server::builder::ServerBuilder;
#[cfg(feature = "graphql")]
use this::server::{GraphQLExposure, RestExposure};
use this::storage::InMemoryLinkService;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let link_service = Arc::new(InMemoryLinkService::new());

    // Billing module
    let billing_stores = BillingStores::new_in_memory();
    populate_test_data(&billing_stores, link_service.clone()).await?;
    let _billing_module = BillingModule::new(billing_stores);

    // Catalog module
    let catalog_stores = CatalogStores::new_in_memory();
    populate_catalog_data(&catalog_stores, link_service.clone()).await?;
    let _catalog_module = CatalogModule::new(catalog_stores);

    // Inventory module
    let inventory_stores = InventoryStores::new_in_memory();
    populate_inventory_data(&inventory_stores, link_service.clone()).await?;
    let _inventory_module = InventoryModule::new(inventory_stores);

    #[cfg(feature = "graphql")]
    {
        // Build the host (transport-agnostic) with all three modules
        let host = Arc::new(
            ServerBuilder::new()
                .with_link_service((*link_service).clone())
                .register_module(_billing_module)?
                .register_module(_catalog_module)?
                .register_module(_inventory_module)?
                .build_host()?,
        );

        // Build REST and GraphQL routers
        let rest_router = RestExposure::build_router(host.clone(), vec![])?;
        let graphql_router = GraphQLExposure::build_router(host.clone())?;

        // Merge routers
        let app = Router::new().merge(rest_router).merge(graphql_router);

        println!("\n🌐 Server running on http://127.0.0.1:4242");
        println!("\n📚 Endpoints disponibles:");
        println!("\n  REST API - Billing:");
        println!("    GET    /order");
        println!("    GET    /invoice");
        println!("    GET    /payment");
        println!("    GET    /order/{{id}}/invoices");
        println!("    GET    /invoice/{{id}}/payments");
        println!("\n  REST API - Catalog:");
        println!("    GET    /product");
        println!("    GET    /category");
        println!("    GET    /tag");
        println!("    GET    /product/{{id}}/categories");
        println!("    GET    /category/{{id}}/products");
        println!("    GET    /product/{{id}}/tags");
        println!("    GET    /tag/{{id}}/products");
        println!("    GET    /category/{{id}}/children");
        println!("    GET    /category/{{id}}/parent");
        println!("\n  REST API - Inventory:");
        println!("    GET    /store");
        println!("    GET    /activity");
        println!("    GET    /warehouse");
        println!("    GET    /stock_item");
        println!("    GET    /stock_movement");
        println!("    GET    /usage");
        println!("    GET    /store/{{id}}/activities");
        println!("    GET    /activity/{{id}}/stores");
        println!("    GET    /store/{{id}}/warehouses");
        println!("    GET    /warehouse/{{id}}/stock_items");
        println!("    GET    /stock_item/{{id}}/movements");
        println!("    GET    /stock_item/{{id}}/product");
        println!("    GET    /activity/{{id}}/usages");
        println!("    GET    /usage/{{id}}/from_activity");
        println!("\n  GraphQL API:");
        println!("    POST   /graphql");
        println!("    GET    /graphql/playground");
        println!("    GET    /graphql/schema");

        let listener = tokio::net::TcpListener::bind("127.0.0.1:4242").await?;
        axum::serve(listener, app).await?;
    }

    #[cfg(not(feature = "graphql"))]
    {
        eprintln!("❌ La feature GraphQL n'est pas activée !");
        eprintln!("   Lancez avec: cargo run -p multi_module_example --features graphql");
    }

    Ok(())
}
