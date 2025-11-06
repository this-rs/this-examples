pub mod model;
pub mod store;
pub mod handlers;
pub mod descriptor;

pub use model::Tag;
pub use store::{TagStore, TagStoreError, InMemoryTagStore};

#[cfg(feature = "dynamodb")]
pub use store::TagDynamoDBStore;

