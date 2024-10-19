extern crate self as s3_api;

use crate::xml::{BucketInfo, LocationInfo};
use actix_web::{http::StatusCode, put, web, HttpResponse};
use actix_xml::Xml;
use rand::distributions::DistString;
use s3_derive::S3Error;
use s3_entities::storage_provider::{StorageErr, StorageProvider};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename = "CreateBucketConfiguration")]
pub struct CreateBucketConfiguration {
    #[serde(rename = "LocationConstraint")]
    pub location_constraint: Option<String>,
    #[serde(rename = "Location")]
    pub location: Option<LocationInfo>,
    #[serde(rename = "Bucket")]
    pub bucket: Option<BucketInfo>,
}

#[derive(Debug, S3Error)]
enum CreateBucketError {
    #[error(status_code = 500, message = "An internal error occurred. Try again.")]
    InternalError {
        request_id: String,
        resource: String,
    },
    #[error(
        status_code = 409,
        message = "The requested bucket name is not available. The bucket namespace is shared by all users of the system. Specify a different name and try again."
    )]
    BucketAlreadyExists {
        request_id: String,
        resource: String,
    },
    #[error(
        status_code = 409,
        message = "The bucket that you tried to create already exists, and you own it. Amazon S3 returns this error in all AWS Regions except in the US East (N. Virginia) Region (us-east-1). For legacy compatibility, if you re-create an existing bucket that you already own in us-east-1, Amazon S3 returns 200 OK and resets the bucket access control lists (ACLs). For Amazon S3 on Outposts, the bucket that you tried to create already exists in your Outpost and you own it."
    )]
    BucketAlreadyOwnedByYou {
        request_id: String,
        resource: String,
    },
}

#[put("/{bucket}")]
pub async fn create(
    path: web::Path<String>,
    storage_provider: web::Data<dyn StorageProvider>,
    payload: Option<Xml<CreateBucketConfiguration>>,
) -> Result<HttpResponse, CreateBucketError> {
    let request_id = rand::distributions::Alphanumeric
        .sample_string(&mut rand::thread_rng(), 16)
        .to_uppercase();
    let bucket = path.into_inner();

    let location = payload
        .and_then(|payload| payload.into_inner().location)
        .and_then(|location| location.name);

    storage_provider
        .into_inner()
        .create_bucket(&bucket, location)
        .await
        .map_err(|e| {
            let bucket = String::clone(&bucket);
            match e {
                StorageErr::BucketAlreadyExists => CreateBucketError::BucketAlreadyExists {
                    request_id,
                    resource: bucket,
                },
                _ => CreateBucketError::InternalError {
                    request_id,
                    resource: bucket,
                },
            }
        })?;

    Ok(HttpResponse::Ok()
        .status(StatusCode::OK)
        .append_header(("Location", String::from("/") + &bucket))
        .finish())
}

#[cfg(test)]
mod tests {
    use std::env;

    use actix_web::{
        http::{
            self,
            header::{ContentType, HeaderValue},
        },
        App,
    };

    use s3_entities::test::storage_provider::get_mock_app_data;

    use super::*;

    #[actix_web::test]
    async fn test_no_payload() {
        env::set_var("RUST_LOG", "debug");
        env::set_var("RUST_BACKTRACE", "full");
        env_logger::init();

        let storage = get_mock_app_data();
        let app = actix_web::test::init_service(
            App::new()
                .app_data(web::Data::from(storage))
                .service(create),
        )
        .await;
        let req = actix_web::test::TestRequest::put()
            .uri("/bucket")
            .insert_header(ContentType::xml())
            .to_request();
        let resp = actix_web::test::call_service(&app, req).await;

        println!("{:?}", resp.response().body());
        assert_eq!(resp.status(), http::StatusCode::OK);
        assert_eq!(
            resp.headers().get("Location").unwrap(),
            HeaderValue::from_static("/bucket")
        );
    }

    #[actix_web::test]
    async fn test_valid_payload_with_location() {
        let payload = r#"<CreateBucketConfiguration xmlns="http://s3.amazonaws.com/doc/2006-03-01/"><LocationConstraint>Europe</LocationConstraint></CreateBucketConfiguration>"#.as_bytes();
        let storage = get_mock_app_data();
        let app = actix_web::test::init_service(
            App::new()
                .app_data(web::Data::from(storage))
                .service(create),
        )
        .await;
        let req = actix_web::test::TestRequest::put()
            .uri("/bucket")
            .insert_header(ContentType::xml())
            .set_payload(payload)
            .to_request();
        let resp = actix_web::test::call_service(&app, req).await;

        assert_eq!(resp.status(), http::StatusCode::OK);
        assert_eq!(
            resp.headers().get("Location").unwrap(),
            HeaderValue::from_static("/bucket")
        );
    }

    #[actix_web::test]
    async fn test_valid_payload_no_location() {
        let payload = r#"<CreateBucketConfiguration xmlns="http://s3.amazonaws.com/doc/2006-03-01/"></CreateBucketConfiguration>"#.as_bytes();
        let storage = get_mock_app_data();
        let app = actix_web::test::init_service(
            App::new()
                .app_data(web::Data::from(storage))
                .service(create),
        )
        .await;
        let req = actix_web::test::TestRequest::put()
            .uri("/bucket")
            .insert_header(ContentType::xml())
            .set_payload(payload)
            .to_request();
        let resp = actix_web::test::call_service(&app, req).await;

        assert_eq!(resp.status(), http::StatusCode::OK);
        assert_eq!(
            resp.headers().get("Location").unwrap(),
            HeaderValue::from_static("/bucket")
        );
    }

    // Should ignore the bad payload and assume there's none provided
    #[actix_web::test]
    async fn test_invalid_payload() {
        let payload =
            r#"<CreateBucketConfiguration xmlns="http://s3.amazonaws.com/doc/2006-03-01/">"#
                .as_bytes();
        let storage = get_mock_app_data();
        let app = actix_web::test::init_service(
            App::new()
                .app_data(web::Data::from(storage))
                .service(create),
        )
        .await;
        let req = actix_web::test::TestRequest::put()
            .uri("/bucket")
            .insert_header(ContentType::xml())
            .set_payload(payload)
            .to_request();
        let resp = actix_web::test::call_service(&app, req).await;

        assert_eq!(resp.status(), http::StatusCode::OK);
        assert_eq!(
            resp.headers().get("Location").unwrap(),
            HeaderValue::from_static("/bucket")
        );
    }
}
