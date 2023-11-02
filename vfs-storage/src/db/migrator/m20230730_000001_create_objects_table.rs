use sea_orm_migration::prelude::*;

use super::m20230730_000002_create_bucket_table::Bucket;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m_20230730_000001_create_objects_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Object::Table)
                    .col(ColumnDef::new(Object::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Object::Key).string().not_null().unique_key())
                    .col(ColumnDef::new(Object::Size).big_integer().not_null())
                    .col(ColumnDef::new(Object::ETag).string().not_null())
                    .col(ColumnDef::new(Object::LastModified).timestamp().not_null())
                    .col(ColumnDef::new(Object::BucketId).uuid().not_null())
                    .col(ColumnDef::new(Object::FileId).string().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-object-bucket_id")
                            .from(Object::Table, Object::BucketId)
                            .to(Bucket::Table, Bucket::Id),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-object-key")
                    .table(Object::Table)
                    .col(Object::Key)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Object::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum Object {
    Table,
    Id,
    Key,
    Size,
    #[iden = "etag"]
    ETag,
    LastModified,
    BucketId,
    FileId,
}
