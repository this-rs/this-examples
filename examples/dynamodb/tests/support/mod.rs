//! What both DynamoDB tests need, in one place.
//!
//! Two things: the table layout `setup.sh` creates, and a way to wait for a
//! container that says it is initialising before it actually accepts
//! connections.

use std::time::{Duration, Instant};

use aws_sdk_dynamodb::types::{
    AttributeDefinition, KeySchemaElement, KeyType, ProvisionedThroughput, ScalarAttributeType,
};
use aws_sdk_dynamodb::Client;
use tokio::net::TcpStream;

/// DynamoDB Local prints `Initializing DynamoDB Local` **before** it binds its
/// port. The wait strategy stops there, so the first request can hit a socket
/// that nobody listens on yet: measured three times in CI as
/// `DispatchFailure(TransientError, IncompleteMessage)` on `create_table`,
/// always on the first call, always on a PR unrelated to DynamoDB.
///
/// Waiting for the port to accept a connection is the condition we actually
/// depend on, and it does not care which line the image logs first.
pub async fn wait_until_serving(host: &str, port: u16) {
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        match TcpStream::connect((host, port)).await {
            Ok(_) => return,
            Err(_) if Instant::now() < deadline => {
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
            Err(error) => {
                panic!("DynamoDB Local never accepted a connection on {host}:{port}: {error}")
            }
        }
    }
}

/// The table layout `setup.sh` creates: a single `id` hash key (no `tenant_id`),
/// which is what the billing stores write. The smoke test's fixture uses a
/// `tenant_id` + `id` key; billing never writes `tenant_id`, so that layout
/// cannot store an order.
pub async fn create_table(client: &Client, name: &str) {
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
        .expect("create a DynamoDB table");
}
