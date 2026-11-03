use testcontainers::runners::AsyncRunner;
use testcontainers_modules::neo4j::Neo4j;

#[ignore = "container-backed: needs Docker, run explicitly by the smoke CI job (--ignored)"]
#[tokio::test]
async fn neo4j_meets_the_soft_delete_contract() {
    let container = Neo4j::default()
        .start()
        .await
        .expect("start Neo4j (Docker?)");
    let host = container.get_host().await.unwrap();
    let port = container.get_host_port_ipv4(7687).await.unwrap();
    // testcontainers-modules Neo4j sets NEO4J_AUTH=neo4j/password
    let bolt = format!("{host}:{port}");

    let stores = neo4j_example::billing_stores(&bolt, "neo4j", "password")
        .await
        .expect("stores");
    soft_delete_example::contract::assert_soft_delete_contract(stores)
        .await
        .expect("the soft-delete contract holds on Neo4j");
}
