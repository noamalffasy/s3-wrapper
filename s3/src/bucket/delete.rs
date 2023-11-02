use actix_web::{delete, web, HttpResponse};

#[delete("/{bucket}")]
pub async fn delete_bucket(path: web::Path<String>) -> HttpResponse {
    let bucket = path.into_inner();

    HttpResponse::NoContent().finish()
}

#[cfg(test)]
mod tests {
    use actix_web::{http, test, App};

    use super::*;

    #[actix_web::test]
    async fn test_no_payload() {
        let app = test::init_service(App::new().service(delete_bucket)).await;
        let req = test::TestRequest::delete().uri("/bucket").to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), http::StatusCode::NO_CONTENT);
    }

    #[actix_web::test]
    async fn test_nonexisting_bucket() {
        let app = test::init_service(App::new().service(delete_bucket)).await;
        let req = test::TestRequest::put().uri("/bucket").to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST);
    }
}
