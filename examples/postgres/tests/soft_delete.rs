use testcontainers::runners::AsyncRunner;
use testcontainers_modules::postgres::Postgres;

#[ignore = "container-backed: needs Docker, run explicitly by the smoke CI job (--ignored)"]
#[tokio::test]
async fn postgres_meets_the_soft_delete_contract() {
    let container = Postgres::default()
        .start()
        .await
        .expect("start Postgres container (Docker?)");
    let host = container.get_host().await.unwrap();
    let port = container.get_host_port_ipv4(5432).await.unwrap();
    let url = format!("postgres://postgres:postgres@{host}:{port}/postgres");

    let pool = sqlx::PgPool::connect(&url).await.expect("connect");
    this::storage::postgres::ensure_schema(&pool)
        .await
        .expect("run migrations");

    let stores = postgres_example::billing_stores(&url)
        .await
        .expect("stores");
    soft_delete_example::contract::assert_soft_delete_contract(stores)
        .await
        .expect("the soft-delete contract holds on PostgreSQL");
}
