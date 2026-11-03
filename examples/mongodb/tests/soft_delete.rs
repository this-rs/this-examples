use testcontainers::runners::AsyncRunner;
use testcontainers_modules::mongo::Mongo;

#[ignore = "container-backed: needs Docker, run explicitly by the smoke CI job (--ignored)"]
#[tokio::test]
async fn mongodb_meets_the_soft_delete_contract() {
    let container = Mongo::default()
        .start()
        .await
        .expect("start Mongo (Docker?)");
    let host = container.get_host().await.unwrap();
    let port = container.get_host_port_ipv4(27017).await.unwrap();
    let url = format!("mongodb://{host}:{port}");

    let stores = mongodb_example::billing_stores(&url, "billing")
        .await
        .expect("stores");
    soft_delete_example::contract::assert_soft_delete_contract(stores)
        .await
        .expect("the soft-delete contract holds on MongoDB");
}
