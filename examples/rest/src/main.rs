use this::server::builder::ServerBuilder;
use this::storage::InMemoryLinkService;
use billing::{BillingStores, BillingModule};
use test_data::populate_test_data;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let stores = BillingStores::new_in_memory();
    
    // Populate test data
    let stores_for_seed = BillingStores {
        orders: stores.orders.clone(),
        invoices: stores.invoices.clone(),
        payments: stores.payments.clone(),
    };
    populate_test_data(stores_for_seed).await?;
    
    let billing_module = BillingModule::new(stores);

    let app = ServerBuilder::new()
        .with_link_service(InMemoryLinkService::new())
        .register_module(billing_module)?
        .build()?;

    println!("🚀 Server running on http://0.0.0.0:4242");
    
    // Utiliser la méthode de lancement du serveur
    let listener = tokio::net::TcpListener::bind("0.0.0.0:4242").await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}
