pub mod model;
pub mod store;
pub mod handlers;
pub mod descriptor;

pub use model::Product;
pub use store::{ProductStore, ProductStoreError, InMemoryProductStore};

#[cfg(feature = "dynamodb")]
pub use store::ProductDynamoDBStore;

