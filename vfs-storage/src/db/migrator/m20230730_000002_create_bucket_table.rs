use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m_20230730_000002_create_buckets_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Bucket::Table)
                    .col(ColumnDef::new(Bucket::Id).uuid().not_null().primary_key())
                    .col(
                        ColumnDef::new(Bucket::Name)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-bucket-name")
                    .table(Bucket::Table)
                    .col(Bucket::Name)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(Index::drop().name("idx-bucket-name").to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Bucket::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
pub enum Bucket {
    Table,
    Id,
    Name,
}
