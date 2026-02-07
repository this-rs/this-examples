pub mod descriptor;
pub mod handlers;
pub mod model;
pub mod store;

pub use model::Product;
pub use store::{InMemoryProductStore, ProductStore, ProductStoreError};

#[cfg(feature = "dynamodb")]
pub use store::ProductDynamoDBStore;





