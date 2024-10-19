extern crate self as s3_api;

use crate::generate_request_id;
use actix_web::{delete, web, HttpResponse};
use s3_derive::S3Error;
use s3_entities::storage_provider::{StorageErr, StorageProvider};

#[derive(Debug, S3Error)]
enum DeleteBucketError {
    #[error(status_code = 500, message = "An internal error occurred. Try again.")]
    InternalError {
        request_id: String,
        resource: String,
    },
    #[error(
        status_code = 409,
        message = "The bucket that you tried to delete is not empty."
    )]
    BucketNotEmpty {
        request_id: String,
        resource: String,
    },
    #[error(status_code = 404, message = "The specified bucket does not exist.")]
    NoSuchBucket {
        request_id: String,
        resource: String,
    },
}

#[delete("/{bucket}")]
pub async fn delete_bucket(
    path: web::Path<String>,
    storage_provider: web::Data<dyn StorageProvider>,
) -> Result<HttpResponse, DeleteBucketError> {
    let request_id = generate_request_id();
    let bucket = path.into_inner();

    storage_provider
        .into_inner()
        .delete_bucket(&bucket)
        .await
        .map_err(|e| {
            let bucket = String::clone(&bucket);
            match e {
                StorageErr::BucketNotFound => DeleteBucketError::NoSuchBucket {
                    request_id,
                    resource: bucket,
                },
                StorageErr::BucketNotEmpty => DeleteBucketError::BucketNotEmpty {
                    request_id,
                    resource: bucket,
                },
                _ => DeleteBucketError::InternalError {
                    request_id,
                    resource: bucket,
                },
            }
        })?;

    Ok(HttpResponse::NoContent().finish())
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{http, App};
    use s3_entities::test::storage_provider::get_mock_app_data;

    #[actix_web::test]
    async fn test_no_payload() {
        let bucket_name = "bucket";

        let provider = get_mock_app_data();
        provider.create_bucket(bucket_name, None).await.unwrap();

        let app = actix_web::test::init_service(
            App::new()
                .app_data(actix_web::web::Data::from(provider))
                .service(delete_bucket),
        )
        .await;
        let req = actix_web::test::TestRequest::delete()
            .uri(&format!("/{bucket_name}"))
            .to_request();
        let resp = actix_web::test::call_service(&app, req).await;

        assert_eq!(resp.status(), http::StatusCode::NO_CONTENT);
    }

    #[actix_web::test]
    async fn test_nonexisting_bucket() {
        let provider = get_mock_app_data();
        let app = actix_web::test::init_service(
            App::new()
                .app_data(actix_web::web::Data::from(provider))
                .service(delete_bucket),
        )
        .await;
        let req = actix_web::test::TestRequest::delete()
            .uri("/bucket")
            .to_request();
        let resp = actix_web::test::call_service(&app, req).await;

        assert_eq!(resp.status(), http::StatusCode::NOT_FOUND);
    }
}
