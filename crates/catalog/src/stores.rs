use std::sync::Arc;

use crate::module::CatalogStores;

// Import stores from entity modules
use crate::entities::category::InMemoryCategoryStore;
use crate::entities::product::InMemoryProductStore;
use crate::entities::tag::InMemoryTagStore;

#[cfg(feature = "dynamodb")]
use crate::entities::category::CategoryDynamoDBStore;
#[cfg(feature = "dynamodb")]
use crate::entities::product::ProductDynamoDBStore;
#[cfg(feature = "dynamodb")]
use crate::entities::tag::TagDynamoDBStore;
#[cfg(feature = "dynamodb")]
use aws_sdk_dynamodb::Client as DynamoDBClient;

// ============================================================================
// Store Factories
// ============================================================================

impl CatalogStores {
    /// Create stores with in-memory implementations
    pub fn new_in_memory() -> Self {
        let products = Arc::new(InMemoryProductStore::default());
        let categories = Arc::new(InMemoryCategoryStore::default());
        let tags = Arc::new(InMemoryTagStore::default());

        Self {
            products_store: products.clone(),
            products_entity: products,
            categories_store: categories.clone(),
            categories_entity: categories,
            tags_store: tags.clone(),
            tags_entity: tags,
        }
    }

    #[cfg(feature = "dynamodb")]
    /// Create stores with DynamoDB implementations
    pub fn new_dynamodb(
        client: DynamoDBClient,
        products_table: String,
        categories_table: String,
        tags_table: String,
    ) -> Self {
        let products = Arc::new(ProductDynamoDBStore::new(client.clone(), products_table));
        let categories = Arc::new(CategoryDynamoDBStore::new(client.clone(), categories_table));
        let tags = Arc::new(TagDynamoDBStore::new(client, tags_table));

        Self {
            products_store: products.clone(),
            products_entity: products,
            categories_store: categories.clone(),
            categories_entity: categories,
            tags_store: tags.clone(),
            tags_entity: tags,
        }
    }
}





