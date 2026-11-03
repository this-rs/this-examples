use aws_sdk_dynamodb::types::{
    AttributeDefinition, KeySchemaElement, KeyType, ProvisionedThroughput, ScalarAttributeType,
};
use aws_sdk_dynamodb::Client;
use testcontainers::core::{IntoContainerPort, WaitFor};
use testcontainers::runners::AsyncRunner;
use testcontainers::GenericImage;

/// The table layout `setup.sh` creates: a single `id` hash key (no `tenant_id`), which is what the
/// billing stores write. The smoke test's fixture uses a `tenant_id` + `id` key; billing never writes
/// `tenant_id`, so that layout cannot store an order.
async fn create_table(client: &Client, name: &str) {
    client
        .create_table()
        .table_name(name)
        .key_schema(
            KeySchemaElement::builder()
                .attribute_name("id")
                .key_type(KeyType::Hash)
                .build()
                .unwrap(),
        )
        .attribute_definitions(
            AttributeDefinition::builder()
                .attribute_name("id")
                .attribute_type(ScalarAttributeType::S)
                .build()
                .unwrap(),
        )
        .provisioned_throughput(
            ProvisionedThroughput::builder()
                .read_capacity_units(5)
                .write_capacity_units(5)
                .build()
                .unwrap(),
        )
        .send()
        .await
        .unwrap();
}

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
        create_table(&client, t).await;
    }

    let stores = dynamodb_example::billing_stores(&endpoint)
        .await
        .expect("stores");
    soft_delete_example::contract::assert_soft_delete_contract(stores)
        .await
        .expect("the soft-delete contract holds on DynamoDB");
}
