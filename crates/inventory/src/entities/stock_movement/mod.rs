pub mod descriptor;
pub mod handlers;
pub mod model;
pub mod store;

pub use model::StockMovement;
pub use store::{InMemoryStockMovementStore, StockMovementStore, StockMovementStoreError};

#[cfg(feature = "dynamodb")]
pub use store::StockMovementDynamoDBStore;





