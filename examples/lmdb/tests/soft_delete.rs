//! The soft-delete contract on LMDB: embedded, so no container, a fresh directory per run.

#[tokio::test]
async fn lmdb_meets_the_soft_delete_contract() {
    let dir = std::env::temp_dir().join(format!(
        "lmdb-soft-delete-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let stores = billing::BillingStores::new_lmdb(&dir).expect("open LMDB stores");
    soft_delete_example::contract::assert_soft_delete_contract(stores)
        .await
        .expect("the soft-delete contract holds on LMDB");
    let _ = std::fs::remove_dir_all(&dir);
}
