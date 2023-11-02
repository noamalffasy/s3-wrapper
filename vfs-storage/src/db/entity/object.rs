use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "objects")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    #[sea_orm(unique, indexed)]
    pub key: String,
    pub size: i64,
    pub etag: String,
    pub last_modified: DateTimeUtc,
    pub bucket_name: String,
    pub file_id: String
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::bucket::Entity",
        from = "Column::BucketName",
        to = "super::bucket::Column::Name"
    )]
    Bucket,
}

impl Related<super::bucket::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Bucket.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}