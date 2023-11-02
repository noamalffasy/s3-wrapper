use sea_orm::{ConnectionTrait, Database, DatabaseConnection, DbBackend, DbErr, Statement};
use sea_orm_migration::{MigratorTrait, SchemaManager};

use super::migrator::Migrator;

async fn connect(db_url: String, db_name: String) -> Result<DatabaseConnection, DbErr> {
    let db = Database::connect(&db_url).await?;
    let db = match db.get_database_backend() {
        DbBackend::MySql => {
            db.execute(Statement::from_string(
                db.get_database_backend(),
                format!("CREATE DATABASE IF NOT EXISTS `{}`;", db_name),
            ))
            .await?;

            let url = format!("{}/{}", db_url, db_name);
            Database::connect(&url).await?
        }
        DbBackend::Postgres => {
            db.execute(Statement::from_string(
                db.get_database_backend(),
                format!("DROP DATABASE IF EXISTS \"{}\";", db_name),
            ))
            .await?;
            db.execute(Statement::from_string(
                db.get_database_backend(),
                format!("CREATE DATABASE \"{}\";", db_name),
            ))
            .await?;

            let url = format!("{}/{}", db_url, db_name);
            Database::connect(&url).await?
        }
        DbBackend::Sqlite => db,
    };

    Ok(db)
}

pub async fn run(db_url: String, db_name: String) -> Result<DatabaseConnection, DbErr> {
    let db = connect(db_url, db_name).await?;
    let schema_manager = SchemaManager::new(&db);

    Migrator::refresh(&db).await?;
    assert!(schema_manager.has_table("buckets").await?);
    assert!(schema_manager.has_table("objects").await?);

    Ok(db)
}
