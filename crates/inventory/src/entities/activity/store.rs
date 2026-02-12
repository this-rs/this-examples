use async_trait::async_trait;
use std::sync::Arc;
use this::prelude::*;

use super::Activity;

#[derive(Debug, thiserror::Error)]
pub enum ActivityStoreError {
    #[error("not found: {0}")]
    NotFound(String),
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("validation: {0}")]
    Validation(String),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[async_trait]
pub trait ActivityStore: Send + Sync {
    async fn create(&self, activity: Activity) -> Result<Activity, ActivityStoreError>;
    async fn get(&self, id: &Uuid) -> Result<Activity, ActivityStoreError>;
    async fn update(&self, activity: Activity) -> Result<Activity, ActivityStoreError>;
    async fn delete(&self, id: &Uuid) -> Result<(), ActivityStoreError>;
    async fn list(&self) -> Result<Vec<Activity>, ActivityStoreError>;
}

use tokio::sync::RwLock;

#[derive(Clone, Default)]
pub struct InMemoryActivityStore {
    inner: Arc<RwLock<Vec<Activity>>>,
}

#[async_trait::async_trait]
impl EntityFetcher for InMemoryActivityStore {
    async fn fetch_as_json(&self, entity_id: &Uuid) -> Result<serde_json::Value, anyhow::Error> {
        let activity = self
            .get(entity_id)
            .await
            .map_err(|_| anyhow::anyhow!("Activity not found: {}", entity_id))?;
        Ok(serde_json::to_value(activity)?)
    }

    async fn list_as_json(
        &self,
        limit: Option<i32>,
        offset: Option<i32>,
    ) -> Result<Vec<serde_json::Value>, anyhow::Error> {
        let all_activities = self.list().await?;
        let offset = offset.unwrap_or(0) as usize;
        let limit = limit.unwrap_or(20) as usize;

        let activities: Vec<Activity> = all_activities
            .into_iter()
            .skip(offset)
            .take(limit)
            .collect();
        activities
            .into_iter()
            .map(|activity| serde_json::to_value(activity).map_err(Into::into))
            .collect()
    }
}

#[async_trait::async_trait]
impl EntityCreator for InMemoryActivityStore {
    async fn create_from_json(
        &self,
        entity_data: serde_json::Value,
    ) -> Result<serde_json::Value, anyhow::Error> {
        let activity = Activity::new(
            entity_data["name"]
                .as_str()
                .unwrap_or("Activity")
                .to_string(),
            entity_data["status"]
                .as_str()
                .unwrap_or("active")
                .to_string(),
            entity_data["activity_type"].as_str().map(String::from),
            entity_data["description"].as_str().map(String::from),
        );

        self.create(activity.clone()).await?;
        Ok(serde_json::to_value(activity)?)
    }
}

#[async_trait::async_trait]
impl ActivityStore for InMemoryActivityStore {
    async fn create(&self, activity: Activity) -> Result<Activity, ActivityStoreError> {
        let mut g = self.inner.write().await;
        if g.iter().any(|a| a.id == activity.id) {
            return Err(ActivityStoreError::Conflict(activity.id.to_string()));
        }
        g.push(activity.clone());
        Ok(activity)
    }

    async fn get(&self, id: &Uuid) -> Result<Activity, ActivityStoreError> {
        let g = self.inner.read().await;
        g.iter()
            .find(|a| &a.id == id)
            .cloned()
            .ok_or_else(|| ActivityStoreError::NotFound(id.to_string()))
    }

    async fn update(&self, activity: Activity) -> Result<Activity, ActivityStoreError> {
        let mut g = self.inner.write().await;
        if let Some(x) = g.iter_mut().find(|a| a.id == activity.id) {
            *x = activity.clone();
            Ok(activity)
        } else {
            Err(ActivityStoreError::NotFound(activity.id.to_string()))
        }
    }

    async fn delete(&self, id: &Uuid) -> Result<(), ActivityStoreError> {
        let mut g = self.inner.write().await;
        let before = g.len();
        g.retain(|a| &a.id != id);
        if g.len() == before {
            return Err(ActivityStoreError::NotFound(id.to_string()));
        }
        Ok(())
    }

    async fn list(&self) -> Result<Vec<Activity>, ActivityStoreError> {
        Ok(self.inner.read().await.clone())
    }
}

#[cfg(feature = "dynamodb")]
use aws_sdk_dynamodb::Client as DynamoDBClient;
#[cfg(feature = "dynamodb")]
use this::storage::DynamoDBDataService;

#[cfg(feature = "dynamodb")]
#[derive(Clone)]
pub struct ActivityDynamoDBStore {
    service: Arc<DynamoDBDataService<Activity>>,
}

#[cfg(feature = "dynamodb")]
impl ActivityDynamoDBStore {
    pub fn new(client: DynamoDBClient, table_name: String) -> Self {
        Self {
            service: Arc::new(DynamoDBDataService::new(client, table_name)),
        }
    }
}

#[cfg(feature = "dynamodb")]
#[async_trait::async_trait]
impl EntityFetcher for ActivityDynamoDBStore {
    async fn fetch_as_json(&self, entity_id: &Uuid) -> Result<serde_json::Value, anyhow::Error> {
        let activity = self
            .get(entity_id)
            .await
            .map_err(|_| anyhow::anyhow!("Activity not found: {}", entity_id))?;
        Ok(serde_json::to_value(activity)?)
    }

    async fn list_as_json(
        &self,
        limit: Option<i32>,
        offset: Option<i32>,
    ) -> Result<Vec<serde_json::Value>, anyhow::Error> {
        let all_activities = self.list().await?;
        let offset = offset.unwrap_or(0) as usize;
        let limit = limit.unwrap_or(20) as usize;

        let activities: Vec<Activity> = all_activities
            .into_iter()
            .skip(offset)
            .take(limit)
            .collect();
        activities
            .into_iter()
            .map(|activity| serde_json::to_value(activity).map_err(Into::into))
            .collect()
    }
}

#[cfg(feature = "dynamodb")]
#[async_trait::async_trait]
impl EntityCreator for ActivityDynamoDBStore {
    async fn create_from_json(
        &self,
        entity_data: serde_json::Value,
    ) -> Result<serde_json::Value, anyhow::Error> {
        let activity = Activity::new(
            entity_data["name"]
                .as_str()
                .unwrap_or("Activity")
                .to_string(),
            entity_data["status"]
                .as_str()
                .unwrap_or("active")
                .to_string(),
            entity_data["activity_type"].as_str().map(String::from),
            entity_data["description"].as_str().map(String::from),
        );

        self.create(activity.clone()).await?;
        Ok(serde_json::to_value(activity)?)
    }
}

#[cfg(feature = "dynamodb")]
#[async_trait::async_trait]
impl ActivityStore for ActivityDynamoDBStore {
    async fn create(&self, activity: Activity) -> Result<Activity, ActivityStoreError> {
        self.service
            .create(activity.clone())
            .await
            .map_err(|e| ActivityStoreError::Other(anyhow::anyhow!(e)))
    }

    async fn get(&self, id: &Uuid) -> Result<Activity, ActivityStoreError> {
        self.service
            .get(id)
            .await
            .map_err(|e| ActivityStoreError::Other(anyhow::anyhow!(e)))?
            .ok_or_else(|| ActivityStoreError::NotFound(id.to_string()))
    }

    async fn update(&self, activity: Activity) -> Result<Activity, ActivityStoreError> {
        self.service
            .update(&activity.id, activity.clone())
            .await
            .map_err(|e| ActivityStoreError::Other(anyhow::anyhow!(e)))
    }

    async fn delete(&self, id: &Uuid) -> Result<(), ActivityStoreError> {
        self.service
            .delete(id)
            .await
            .map_err(|e| ActivityStoreError::Other(anyhow::anyhow!(e)))
    }

    async fn list(&self) -> Result<Vec<Activity>, ActivityStoreError> {
        self.service
            .list()
            .await
            .map_err(|e| ActivityStoreError::Other(anyhow::anyhow!(e)))
    }
}
