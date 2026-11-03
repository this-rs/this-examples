//! The contract, run against the in-memory stores: the reference the other backends are held to.

#[tokio::test]
async fn in_memory_meets_the_soft_delete_contract() {
    let stores = billing::BillingStores::new_in_memory();
    soft_delete_example::contract::assert_soft_delete_contract(stores)
        .await
        .expect("the contract holds on the in-memory stores");
}
