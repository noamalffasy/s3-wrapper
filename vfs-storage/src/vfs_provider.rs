use crate::db;
use async_trait::async_trait;
use s3::storage_provider::StorageErr;
use sea_orm::{DatabaseConnection, DbErr};
use std::env;

#[async_trait]
pub trait VfsProvider: Sync {
    async fn upload_file(&self) -> Result<(), StorageErr>;
    async fn download_file(&self) -> Result<(), StorageErr>;

    async fn connect_to_db(&self) -> DatabaseConnection {
        dotenvy::dotenv().ok();
        let db_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set in .env file");
        let db_name = env::var("DATABASE_NAME").expect("DATABASE_NAME is not set in .env file");

        db::run(db_url, db_name).await.unwrap()
    }
}

#[async_trait]
impl s3::storage_provider::StorageProvider for dyn VfsProvider {
    async fn create_bucket(&self, name: String) -> Result<(), StorageErr> {
        let conn = self.connect_to_db().await;
        let _ = db::create_bucket(&conn, name)
            .await
            .map_err(|err| StorageErr::IOErr(Box::new(err)))?;

        Ok(())
    }

    async fn head_bucket(&self, name: String) -> Result<(), StorageErr> {
        let conn = self.connect_to_db().await;

        db::head_bucket(&conn, name).await.map_err(|err| match err {
            DbErr::RecordNotFound(_) => StorageErr::BucketNotFound,
            other => StorageErr::IOErr(Box::new(other)),
        })
    }

    async fn delete_bucket(&self, name: String) -> Result<(), StorageErr> {
        let conn = self.connect_to_db().await;
        let _ = db::create_bucket(&conn, name)
            .await
            .map_err(|err| StorageErr::IOErr(Box::new(err)))?;

        Ok(())
    }

    async fn delete_object(&self, bucket_name: String, key: String) -> Result<(), StorageErr> {
        let conn = self.connect_to_db().await;
        let _ = db::delete_object(&conn, bucket_name, key)
            .await
            .map_err(|err| StorageErr::IOErr(Box::new(err)))?;

        Ok(())
    }

    async fn delete_objects(
        &self,
        bucket_name: String,
        objects: Vec<String>,
    ) -> Result<(), StorageErr> {
        let conn = self.connect_to_db().await;
        let _ = db::delete_objects(&conn, bucket_name, objects)
            .await
            .map_err(|err| StorageErr::IOErr(Box::new(err)))?;

        Ok(())
    }
}
