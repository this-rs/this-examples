use aws_sdk_dynamodb::types::{
    AttributeDefinition, KeySchemaElement, KeyType, ProvisionedThroughput, ScalarAttributeType,
};
use aws_sdk_dynamodb::Client;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::Extension;
use testcontainers::core::{IntoContainerPort, WaitFor};
use testcontainers::runners::AsyncRunner;
use testcontainers::GenericImage;
use this::prelude::{AuthContext, Uuid};
use tower::ServiceExt;

async fn create_table(client: &Client, name: &str) {
    client
        .create_table()
        .table_name(name)
        .key_schema(
            KeySchemaElement::builder()
                .attribute_name("tenant_id")
                .key_type(KeyType::Hash)
                .build()
                .unwrap(),
        )
        .key_schema(
            KeySchemaElement::builder()
                .attribute_name("id")
                .key_type(KeyType::Range)
                .build()
                .unwrap(),
        )
        .attribute_definitions(
            AttributeDefinition::builder()
                .attribute_name("tenant_id")
                .attribute_type(ScalarAttributeType::S)
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

#[tokio::test]
async fn dynamodb_serves_orders() {
    let container = GenericImage::new("amazon/dynamodb-local", "latest")
        .with_exposed_port(8000.tcp())
        .with_wait_for(WaitFor::message_on_stdout("Initializing DynamoDB Local"))
        .start()
        .await
        .expect("start DynamoDB Local (Docker?)");

    let host = container.get_host().await.unwrap();
    let port = container.get_host_port_ipv4(8000).await.unwrap();
    let endpoint = format!("http://{host}:{port}");

    // Credentials must be supplied explicitly. DynamoDB Local accepts anything,
    // but without them the SDK falls back to its credential chain and calls the
    // EC2 instance-metadata endpoint, which on a GitHub runner answers
    // `411 Length Required` and surfaces as `FailedToLoadToken` — a failure that
    // looks like a broken table but is really a missing fake credential.
    let config = aws_config::defaults(aws_config::BehaviorVersion::latest())
        .endpoint_url(&endpoint)
        .region(aws_config::Region::new("us-east-1"))
        .credentials_provider(aws_sdk_dynamodb::config::Credentials::new(
            "fakeMyKeyId",
            "fakeSecretAccessKey",
            None,
            None,
            "smoke-test",
        ))
        .load()
        .await;
    let client = Client::new(&config);

    for t in ["links", "orders", "invoices", "payments"] {
        create_table(&client, t).await;
    }

    let app = dynamodb_example::app(&endpoint).await.expect("build app");
    let app = app.layer(Extension(AuthContext::User {
        user_id: Uuid::from_u128(1),
        tenant_id: Uuid::from_u128(9),
        roles: vec![],
    }));
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/orders")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("request");
    assert_eq!(resp.status(), StatusCode::OK);
}
