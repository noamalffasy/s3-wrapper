use sea_orm::{sea_query::Alias, *};
use sea_query::Expr;

use super::{entity::object, func};

const MAX_KEYS: u64 = 1000;

/// List objects in a bucket 
/// 
/// # Example
/// 
/// ```
/// use db;
/// 
/// assert_eq!(
///     db::list_objects(
///         &db,
///         "test".to_owned(),
///         delimiter: None,
///         marker: None,
///         max_keys: None,
///         prefix: None,
///     ).await?,
///     
/// )
/// ```
pub async fn list_objects(
    db: &DbConn,
    bucket_name: String,
    delimiter: Option<char>,
    marker: Option<String>,
    max_keys: Option<u64>,
    prefix: Option<String>,
) -> Result<Vec<object::Model>, DbErr> {
    let mut query = object::Entity::find()
        .filter(object::Column::BucketName.eq(bucket_name))
        .order_by_asc(object::Column::Key);

    if let Some(delimiter) = delimiter {
        let delimiter_position = func::instr(Expr::col(object::Column::Key), Expr::expr(delimiter));

        let common_prefixes = Expr::expr(
            Expr::case(
                Expr::expr(delimiter_position.clone()).gt(-1),
                func::substr(
                    Expr::col(object::Column::Key),
                    Expr::expr(1),
                    Expr::expr(delimiter_position.clone()),
                ),
            )
            .finally(Expr::col(object::Column::Key)),
        );

        query = query
            .column_as(common_prefixes, "common_prefixes")
            .group_by(Expr::col((object::Entity, Alias::new("common_prefixes"))))
    }

    if let Some(marker) = marker {
        query = query.filter(object::Column::Key.gt(marker))
    }

    if let Some(limit) = max_keys {
        query = query.limit(std::cmp::min(limit, MAX_KEYS));
    } else {
        query = query.limit(MAX_KEYS);
    }

    if let Some(prefix) = prefix {
        query = query.filter(object::Column::Key.starts_with(prefix));
    }

    let objects = query.all(db).await?;

    Ok(objects)
}

pub async fn copy_object(
    db: &DbConn,
    dest_bucket_name: String,
    dest_key: String,
    source_bucket_name: String,
    source_key: String,
) -> Result<object::Model, DbErr> {
    let source_object: object::Model = object::Entity::find()
        .filter(object::Column::Key.eq(source_key))
        .filter(object::Column::BucketName.eq(source_bucket_name))
        .one(db)
        .await?
        .ok_or(DbErr::Custom("Cannot find source object.".to_owned()))
        .map(Into::into)?;

    let dest_object: object::Model = object::ActiveModel {
        id: Set(uuid::Uuid::new_v4()),
        key: Set(dest_key),
        size: Set(source_object.size),
        etag: Set(source_object.etag),
        last_modified: Set(source_object.last_modified),
        bucket_name: Set(dest_bucket_name),
        file_id: Set(source_object.file_id),
    }
    .insert(db)
    .await?;

    Ok(dest_object)
}

pub async fn delete_object(
    db: &DbConn,
    bucket_name: String,
    key: String,
) -> Result<DeleteResult, DbErr> {
    let object: object::ActiveModel = object::Entity::find()
        .filter(object::Column::Key.eq(key))
        .filter(object::Column::BucketName.eq(bucket_name))
        .one(db)
        .await?
        .ok_or(DbErr::Custom("Cannot find object.".to_owned()))
        .map(Into::into)?;

    Ok(object.delete(db).await?)
}

pub async fn delete_objects(
    db: &DbConn,
    bucket_name: String,
    keys: Vec<String>,
) -> Result<DeleteResult, DbErr> {
    let result = object::Entity::delete_many()
        .filter(object::Column::Key.is_in(keys))
        .filter(object::Column::BucketName.eq(bucket_name))
        .exec(db)
        .await?;

    Ok(result)
}

#[cfg(test)]
mod tests {
    use sea_orm::{entity::prelude::*, DatabaseBackend, MockDatabase, Transaction, Values};

    use crate::db::{
        entity::object,
        object::{list_objects, MAX_KEYS},
    };

    fn setup_db() -> DatabaseConnection {
        MockDatabase::new(DatabaseBackend::Sqlite)
            .append_query_results([vec![object::Model {
                id: Uuid::new_v4(),
                key: String::from("sample.jpg"),
                size: 142863,
                etag: String::from("bf1d737a4d46a19f3bced6905cc8b902"),
                last_modified: DateTime::from_timestamp_millis(1662921288000)
                    .unwrap()
                    .and_utc(),
                bucket_name: String::from("test"),
                file_id: String::from(
                    "BQACAgQAAxkDAAMEZMOIYdfPpjwYEl05ZAN9HiXE2HMAAt4NAAIm-CBSDo0lBzNVdIgvBA",
                ),
            }]])
            .into_connection()
    }

    #[tokio::test]
    async fn test_list_objects_delimiter() -> Result<(), DbErr> {
        let db = setup_db();

        list_objects(&db, "test".to_owned(), Some('/'), None, None, None).await?;

        assert_eq!(
            db.into_transaction_log(),
            [Transaction::from_sql_and_values(
                DatabaseBackend::Sqlite,
                [
                    r#"SELECT"#,
                    r#""objects"."id", "objects"."key", "objects"."size", "objects"."etag", "objects"."last_modified", "objects"."bucket_name", "objects"."file_id","#,
                    r#"(CASE WHEN (INSTR("key", ?) > ?) THEN SUBSTR("key", ?, INSTR("key", ?)) ELSE "key" END) AS "common_prefixes""#,
                    r#"FROM "objects""#,
                    r#"WHERE "objects"."bucket_name" = ?"#,
                    r#"GROUP BY "objects"."common_prefixes""#,
                    r#"ORDER BY "objects"."key" ASC"#,
                    r#"LIMIT ?"#,
                ]
                .join(" "),
                Values(vec![
                    Value::Char(Some('/')),
                    Value::Int(Some(-1)),
                    Value::Int(Some(1)),
                    Value::Char(Some('/')),
                    Value::String(Some(Box::new("test".to_owned()))),
                    Value::BigUnsigned(Some(MAX_KEYS)),
                ])
            )]
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_list_objects_marker() -> Result<(), DbErr> {
        let db = setup_db();

        list_objects(
            &db,
            "test".to_owned(),
            None,
            Some("sample.jpg".to_owned()),
            None,
            None,
        )
        .await?;

        assert_eq!(
            db.into_transaction_log(),
            [Transaction::from_sql_and_values(
                DatabaseBackend::Sqlite,
                [
                    r#"SELECT"#,
                    r#""objects"."id", "objects"."key", "objects"."size", "objects"."etag", "objects"."last_modified", "objects"."bucket_name", "objects"."file_id""#,
                    r#"FROM "objects""#,
                    r#"WHERE "objects"."bucket_name" = ?"#,
                    r#"AND "objects"."key" > ?"#,
                    r#"ORDER BY "objects"."key" ASC"#,
                    r#"LIMIT ?"#,
                ]
                .join(" "),
                Values(vec![
                    Value::String(Some(Box::new("test".to_owned()))),
                    Value::String(Some(Box::new("sample.jpg".to_owned()))),
                    Value::BigUnsigned(Some(MAX_KEYS)),
                ])
            )]
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_list_objects_max_less_than_total_max() -> Result<(), DbErr> {
        let db = setup_db();
        let max = 10;

        list_objects(&db, "test".to_owned(), None, None, Some(max), None).await?;

        assert_eq!(
            db.into_transaction_log(),
            [Transaction::from_sql_and_values(
                DatabaseBackend::Sqlite,
                [
                    r#"SELECT"#,
                    r#""objects"."id", "objects"."key", "objects"."size", "objects"."etag", "objects"."last_modified", "objects"."bucket_name", "objects"."file_id""#,
                    r#"FROM "objects""#,
                    r#"WHERE "objects"."bucket_name" = ?"#,
                    r#"ORDER BY "objects"."key" ASC"#,
                    r#"LIMIT ?"#,
                ]
                .join(" "),
                Values(vec![
                    Value::String(Some(Box::new("test".to_owned()))),
                    Value::BigUnsigned(Some(max)),
                ])
            )]
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_list_objects_max_more_than_total_max() -> Result<(), DbErr> {
        let db = setup_db();
        let max = MAX_KEYS + 10;

        list_objects(&db, "test".to_owned(), None, None, Some(max), None).await?;

        assert_eq!(
            db.into_transaction_log(),
            [Transaction::from_sql_and_values(
                DatabaseBackend::Sqlite,
                [
                    r#"SELECT"#,
                    r#""objects"."id", "objects"."key", "objects"."size", "objects"."etag", "objects"."last_modified", "objects"."bucket_name", "objects"."file_id""#,
                    r#"FROM "objects""#,
                    r#"WHERE "objects"."bucket_name" = ?"#,
                    r#"ORDER BY "objects"."key" ASC"#,
                    r#"LIMIT ?"#,
                ]
                .join(" "),
                Values(vec![
                    Value::String(Some(Box::new("test".to_owned()))),
                    Value::BigUnsigned(Some(MAX_KEYS)),
                ])
            )]
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_list_objects_prefix() -> Result<(), DbErr> {
        let db = setup_db();
        let prefix = "sam".to_owned();

        list_objects(
            &db,
            "test".to_owned(),
            None,
            None,
            None,
            Some(prefix.clone()),
        )
        .await?;

        assert_eq!(
            db.into_transaction_log(),
            [Transaction::from_sql_and_values(
                DatabaseBackend::Sqlite,
                [
                    r#"SELECT"#,
                    r#""objects"."id", "objects"."key", "objects"."size", "objects"."etag", "objects"."last_modified", "objects"."bucket_name", "objects"."file_id""#,
                    r#"FROM "objects""#,
                    r#"WHERE "objects"."bucket_name" = ?"#,
                    r#"AND "objects"."key" LIKE ?"#,
                    r#"ORDER BY "objects"."key" ASC"#,
                    r#"LIMIT ?"#,
                ]
                .join(" "),
                Values(vec![
                    Value::String(Some(Box::new("test".to_owned()))),
                    Value::String(Some(Box::new(prefix.clone() + "%"))),
                    Value::BigUnsigned(Some(MAX_KEYS)),
                ])
            )]
        );

        Ok(())
    }
}
