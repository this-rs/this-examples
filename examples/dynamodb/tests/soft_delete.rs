use aws_sdk_dynamodb::Client;

mod support;
use testcontainers::core::{IntoContainerPort, WaitFor};
use testcontainers::runners::AsyncRunner;
use testcontainers::GenericImage;

#[ignore = "container-backed: needs Docker, run explicitly by the smoke CI job (--ignored)"]
#[tokio::test]
async fn dynamodb_meets_the_soft_delete_contract() {
    let container = GenericImage::new("amazon/dynamodb-local", "latest")
        .with_exposed_port(8000.tcp())
        .with_wait_for(WaitFor::message_on_stdout("Initializing DynamoDB Local"))
        .start()
        .await
        .expect("start DynamoDB Local (Docker?)");
    let host = container.get_host().await.unwrap();
    let port = container.get_host_port_ipv4(8000).await.unwrap();
    support::wait_until_serving(&host.to_string(), port).await;
    let endpoint = format!("http://{host}:{port}");

    let config = aws_config::defaults(aws_config::BehaviorVersion::latest())
        .endpoint_url(&endpoint)
        .region(aws_config::Region::new("us-east-1"))
        .credentials_provider(aws_sdk_dynamodb::config::Credentials::new(
            "fakeMyKeyId",
            "fakeSecretAccessKey",
            None,
            None,
            "soft-delete-test",
        ))
        .load()
        .await;
    let client = Client::new(&config);
    for t in ["links", "orders", "invoices", "payments"] {
        support::create_table(&client, t).await;
    }

    let stores = dynamodb_example::billing_stores(&endpoint)
        .await
        .expect("stores");
    soft_delete_example::contract::assert_soft_delete_contract(stores)
        .await
        .expect("the soft-delete contract holds on DynamoDB");
}
