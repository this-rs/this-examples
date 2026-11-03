pub mod model;
pub mod store;

pub use model::Warehouse;
pub use store::{InMemoryWarehouseStore, WarehouseStore, WarehouseStoreError};

#[cfg(feature = "dynamodb")]
pub use store::WarehouseDynamoDBStore;
