use super::bucket::Bucket;
use async_trait::async_trait;
use std::error;
use thiserror::Error;

#[async_trait]
pub trait StorageProvider: Send + Sync {
    async fn list_buckets(&self) -> Result<Vec<Bucket>, StorageErr>;
    async fn create_bucket(&self, name: &str, region: Option<String>) -> Result<(), StorageErr>;
    async fn head_bucket(&self, name: &str) -> Result<(), StorageErr>;
    async fn delete_bucket(&self, name: &str) -> Result<(), StorageErr>;
    async fn delete_object(&self, bucket_name: &str, object: &str) -> Result<(), StorageErr>;
    async fn delete_objects(
        &self,
        bucket_name: &str,
        objects: Vec<String>,
    ) -> Vec<Result<(), StorageErr>>;
}

#[derive(Error, Debug)]
pub enum StorageErr {
    #[error("bucket not found")]
    BucketNotFound,
    #[error("bucket not empty")]
    BucketNotEmpty,
    #[error("bucket already exists")]
    BucketAlreadyExists,
    #[error("object not found")]
    ObjectNotFound,
    #[error("failed due to IO error: {0}")]
    IOErr(#[from] Box<dyn error::Error + Send + Sync>),
}
