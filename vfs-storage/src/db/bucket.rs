use sea_orm::*;

use super::entity::bucket;

pub async fn list_buckets(db: &DbConn) -> Result<Vec<bucket::Model>, DbErr> {
    bucket::Entity::find().all(db).await
}

pub async fn head_bucket(db: &DbConn, name: String) -> Result<(), DbErr> {
    let _: bucket::ActiveModel = bucket::Entity::find()
        .filter(bucket::Column::Name.eq(&name))
        .one(db)
        .await?
        .ok_or(DbErr::RecordNotFound(format!(
            "bucket '{}' doesn't exist",
            &name
        )))
        .map(Into::into)?;

    Ok(())
}

pub async fn create_bucket(db: &DbConn, name: String) -> Result<bucket::ActiveModel, DbErr> {
    bucket::ActiveModel {
        name: Set(name),
        ..Default::default()
    }
    .save(db)
    .await
}

pub async fn delete_bucket(db: &DbConn, name: String) -> Result<DeleteResult, DbErr> {
    let bucket: bucket::ActiveModel = bucket::Entity::find()
        .filter(bucket::Column::Name.eq(&name))
        .one(db)
        .await?
        .ok_or(DbErr::RecordNotFound(format!(
            "bucket '{}' doesn't exist",
            &name
        )))
        .map(Into::into)?;

    bucket.delete(db).await
}
