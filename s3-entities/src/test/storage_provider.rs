use crate::{
    bucket::Bucket,
    storage_provider::{StorageErr, StorageProvider},
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use futures::future;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

const DEFAULT_REGION: &str = "Europe";

pub struct MockStorageProvider {
    buckets: Mutex<HashMap<String, MockBucket>>,
}

impl MockStorageProvider {
    pub fn new() -> Self {
        MockStorageProvider {
            buckets: Mutex::new(HashMap::new()),
        }
    }
}

pub fn get_mock_app_data() -> Arc<dyn StorageProvider> {
    let provider = MockStorageProvider::new();

    Arc::new(provider)
}

pub struct MockBucket {
    region: String,
    objects: Mutex<HashMap<String, Vec<u8>>>,
    creation_date: DateTime<Utc>,
}

#[async_trait]
impl StorageProvider for MockStorageProvider {
    async fn list_buckets(&self) -> Result<Vec<Bucket>, StorageErr> {
        let buckets = self
            .buckets
            .lock()
            .map_err(|err| StorageErr::IOErr(err.to_string().into()))?;

        Ok(buckets
            .iter()
            .map(|bucket| {
                let (bucket_name, data) = bucket;

                return Bucket {
                    name: bucket_name.into(),
                    region: data.region.clone(),
                    creation_date: data.creation_date,
                };
            })
            .collect::<Vec<_>>())
    }

    async fn create_bucket(&self, name: &str, region: Option<String>) -> Result<(), StorageErr> {
        let mut buckets = self
            .buckets
            .lock()
            .map_err(|err| StorageErr::IOErr(err.to_string().into()))?;

        if buckets.contains_key(name) {
            return Err(StorageErr::BucketAlreadyExists);
        }

        buckets.insert(
            name.into(),
            MockBucket {
                region: region.unwrap_or(DEFAULT_REGION.into()),
                objects: Mutex::new(HashMap::new()),
                creation_date: Utc::now(),
            },
        );

        Ok(())
    }

    async fn head_bucket(&self, name: &str) -> Result<(), StorageErr> {
        let buckets = self
            .buckets
            .lock()
            .map_err(|err| StorageErr::IOErr(err.to_string().into()))?;

        if !buckets.contains_key(name) {
            return Err(StorageErr::BucketNotFound);
        }

        Ok(())
    }

    async fn delete_bucket(&self, name: &str) -> Result<(), StorageErr> {
        let mut buckets = self
            .buckets
            .lock()
            .map_err(|err| StorageErr::IOErr(err.to_string().into()))?;

        if !buckets
            .get(name)
            .ok_or(StorageErr::BucketNotFound)?
            .objects
            .lock()
            .map_err(|err| StorageErr::IOErr(err.to_string().into()))?
            .is_empty()
        {
            return Err(StorageErr::BucketNotEmpty);
        }

        buckets.remove(name);

        Ok(())
    }

    async fn delete_object(&self, bucket_name: &str, object: &str) -> Result<(), StorageErr> {
        let buckets = self
            .buckets
            .lock()
            .map_err(|err| StorageErr::IOErr(err.to_string().into()))?;
        let bucket = buckets.get(bucket_name).ok_or(StorageErr::BucketNotFound)?;
        let mut bucket_objects = bucket
            .objects
            .lock()
            .map_err(|err| StorageErr::IOErr(err.to_string().into()))?;

        if bucket_objects.contains_key(object) {
            return Err(StorageErr::ObjectNotFound);
        }

        bucket_objects.remove(object);

        Ok(())
    }

    async fn delete_objects(
        &self,
        bucket_name: &str,
        objects: Vec<String>,
    ) -> Vec<Result<(), StorageErr>> {
        let results = objects
            .iter()
            .map(|object| self.delete_object(bucket_name, object))
            .collect::<Vec<_>>();

        future::join_all(results).await
    }
}
