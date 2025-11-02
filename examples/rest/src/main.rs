use billing::{BillingModule, BillingStores};
use std::sync::Arc;
use test_data::populate_test_data;
use this::server::builder::ServerBuilder;
use this::storage::InMemoryLinkService;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let link_service = Arc::new(InMemoryLinkService::new());

    // Create stores and populate with test data
    let stores = BillingStores::new_in_memory();
    populate_test_data(&stores, link_service.clone()).await?;

    let billing_module = BillingModule::new(stores);

    let app = ServerBuilder::new()
        .with_link_service((*link_service).clone())
        .register_module(billing_module)?
        .build()?;

    println!("🚀 Server running on http://0.0.0.0:4242");

    // Utiliser la méthode de lancement du serveur
    let listener = tokio::net::TcpListener::bind("0.0.0.0:4242").await?;
    axum::serve(listener, app).await?;

    Ok(())
}
