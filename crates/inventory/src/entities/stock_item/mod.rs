pub mod descriptor;
pub mod handlers;
pub mod model;
pub mod store;

pub use model::StockItem;
pub use store::{InMemoryStockItemStore, StockItemStore, StockItemStoreError};

#[cfg(feature = "dynamodb")]
pub use store::StockItemDynamoDBStore;





