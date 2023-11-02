use std::error;

use async_trait::async_trait;
use thiserror::Error;

#[async_trait]
pub trait StorageProvider {
    async fn create_bucket(&self, name: String) -> Result<(), StorageErr>;
    async fn head_bucket(&self, name: String) -> Result<(), StorageErr>;
    async fn delete_bucket(&self, name: String) -> Result<(), StorageErr>;
    async fn delete_object(&self, bucket_name: String, object: String) -> Result<(), StorageErr>;
    async fn delete_objects(
        &self,
        bucket_name: String,
        objects: Vec<String>,
    ) -> Result<(), StorageErr>;
}

#[derive(Error, Debug)]
pub enum StorageErr {
    #[error("bucket not found")]
    BucketNotFound,
    #[error("object not found")]
    ObjectNotFound,
    #[error(transparent)]
    IOErr(#[from] Box<dyn error::Error>),
}
