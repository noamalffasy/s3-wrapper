use sea_orm_migration::prelude::*;

mod m20230730_000001_create_objects_table;
mod m20230730_000002_create_bucket_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20230730_000001_create_objects_table::Migration),
            Box::new(m20230730_000002_create_bucket_table::Migration),
        ]
    }
}