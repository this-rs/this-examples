use std::sync::Arc;

use anyhow::Result;
use axum::Router;

use billing::{BillingModule, BillingStores};
use test_data::populate_test_data;

use this::server::builder::ServerBuilder;
use this::server::{GraphQLExposure, RestExposure};
use this::storage::InMemoryLinkService;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let stores = BillingStores::new_in_memory();
    // Préparer le module ensuite pour l'enregistrer
    let billing_module = BillingModule::new(stores);

    #[cfg(feature = "graphql")]
    {
        let link_service_arc = Arc::new(InMemoryLinkService::new());

        // Populate test data BEFORE building the host
        populate_test_data(&billing_module.stores, link_service_arc.clone()).await?;

        // Construire l'hôte (agnostique au transport) ensuite
        let host = Arc::new(
            ServerBuilder::new()
                .with_link_service((*link_service_arc).clone())
                .register_module(billing_module)?
                .build_host()?,
        );

        // Construire les routeurs REST et GraphQL
        let rest_router = RestExposure::build_router(host.clone(), vec![])?;
        let graphql_router = GraphQLExposure::build_router(host.clone())?;

        // Fusionner les routeurs
        let app = Router::new().merge(rest_router).merge(graphql_router);

        println!("\n🌐 Server running on http://127.0.0.1:4242");
        println!("\n📚 Endpoints disponibles:");
        println!("\n  REST API:");
        println!("    GET    /health");
        println!("    GET    /order");
        println!("    GET    /invoice");
        println!("    GET    /payment");
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
        eprintln!("   Lancez avec: cargo run -p graphql_example --features graphql");
    }

    Ok(())
}
