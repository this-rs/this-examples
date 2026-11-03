use testcontainers::runners::AsyncRunner;
use testcontainers_modules::mysql::Mysql;

#[ignore = "container-backed: needs Docker, run explicitly by the smoke CI job (--ignored)"]
#[tokio::test]
async fn mysql_meets_the_soft_delete_contract() {
    let container = Mysql::default()
        .start()
        .await
        .expect("start MySQL (Docker?)");
    let host = container.get_host().await.unwrap();
    let port = container.get_host_port_ipv4(3306).await.unwrap();
    let url = format!("mysql://root@{host}:{port}/test");

    let pool = sqlx::MySqlPool::connect(&url).await.expect("connect");
    this::storage::mysql::ensure_schema(&pool)
        .await
        .expect("migrations");

    let stores = mysql_example::billing_stores(&url).await.expect("stores");
    soft_delete_example::contract::assert_soft_delete_contract(stores)
        .await
        .expect("the soft-delete contract holds on MySQL");
}
