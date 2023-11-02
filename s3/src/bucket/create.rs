use actix_web::{http::StatusCode, put, web, HttpResponse};
use actix_xml::Xml;

#[derive(serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CreateBucketConfiguration {
    pub location_constraint: Option<String>,
}

#[put("/{bucket}")]
pub async fn create(
    path: web::Path<String>,
    data: Option<Xml<CreateBucketConfiguration>>,
) -> HttpResponse {
    let bucket = path.into_inner();

    HttpResponse::Ok()
        .status(StatusCode::OK)
        .append_header(("Location", String::from("/") + &bucket))
        .finish()
}

#[cfg(test)]
mod tests {
    use actix_web::{
        http::{
            self,
            header::{ContentType, HeaderValue},
        },
        test, App,
    };

    use super::*;

    #[actix_web::test]
    async fn test_no_payload() {
        let app = test::init_service(App::new().service(create)).await;
        let req = test::TestRequest::put()
            .uri("/bucket")
            .insert_header(ContentType::xml())
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), http::StatusCode::OK);
        assert_eq!(
            resp.headers().get("Location").unwrap(),
            HeaderValue::from_static("/bucket")
        );
    }

    #[actix_web::test]
    async fn test_valid_payload_with_location() {
        let payload = r#"<CreateBucketConfiguration xmlns="http://s3.amazonaws.com/doc/2006-03-01/"><LocationConstraint>Europe</LocationConstraint></CreateBucketConfiguration>"#.as_bytes();
        let app = test::init_service(App::new().service(create)).await;
        let req = test::TestRequest::put()
            .uri("/bucket")
            .insert_header(ContentType::xml())
            .set_payload(payload)
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), http::StatusCode::OK);
        assert_eq!(
            resp.headers().get("Location").unwrap(),
            HeaderValue::from_static("/bucket")
        );
    }

    #[actix_web::test]
    async fn test_valid_payload_no_location() {
        let payload = r#"<CreateBucketConfiguration xmlns="http://s3.amazonaws.com/doc/2006-03-01/"></CreateBucketConfiguration>"#.as_bytes();
        let app = test::init_service(App::new().service(create)).await;
        let req = test::TestRequest::put()
            .uri("/bucket")
            .insert_header(ContentType::xml())
            .set_payload(payload)
            .to_request();
        let resp = test::call_service(&app, req).await;

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
        let app = test::init_service(App::new().service(create)).await;
        let req = test::TestRequest::put()
            .uri("/bucket")
            .insert_header(ContentType::xml())
            .set_payload(payload)
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), http::StatusCode::OK);
        assert_eq!(
            resp.headers().get("Location").unwrap(),
            HeaderValue::from_static("/bucket")
        );
    }
}
