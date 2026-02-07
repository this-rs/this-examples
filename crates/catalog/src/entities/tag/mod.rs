pub mod descriptor;
pub mod handlers;
pub mod model;
pub mod store;

pub use model::Tag;
pub use store::{InMemoryTagStore, TagStore, TagStoreError};

#[cfg(feature = "dynamodb")]
pub use store::TagDynamoDBStore;





