use crate::{error::BaseError, generate_request_id};
use actix_web::{get, http::StatusCode, web, HttpResponse};
use s3_entities::storage_provider::StorageProvider;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename = "ListAllMyBucketsResult")]
struct ListAllMyBucketsResult {
    #[serde(rename = "Buckets")]
    buckets: Buckets,
    #[serde(rename = "Owner")]
    owner: Owner,
}

#[derive(Debug, Serialize, Deserialize)]
struct Buckets {
    #[serde(rename = "Bucket")]
    buckets: Vec<Bucket>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Bucket {
    #[serde(rename = "CreationDate")]
    creation_date: String,
    #[serde(rename = "String")]
    name: String,
}

#[derive(Serialize, Deserialize)]
struct Owner {
    #[serde(rename = "ID")]
    id: ID,
    #[serde(rename = "DisplayName")]
    display_name: DisplayName,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename = "ID")]
struct ID {
    #[serde(rename = "$text")]
    content: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename = "DisplayName")]
struct DisplayName {
    #[serde(rename = "$text")]
    content: String,
}

#[get("/")]
pub async fn list(
    storage_provider: web::Data<dyn StorageProvider>,
) -> Result<HttpResponse, BaseError> {
    let request_id = generate_request_id();
    let buckets: Vec<Bucket> = storage_provider
        .into_inner()
        .list_buckets()
        .await
        .map_err(|_| BaseError::InternalError {
            request_id,
            resource: "/".into(),
        })?
        .iter()
        .map(|bucket| Bucket {
            name: bucket.name.clone(),
            creation_date: bucket.creation_date.to_rfc3339(),
        })
        .collect();

    Ok(HttpResponse::Ok().status(StatusCode::OK).body(
        quick_xml::se::to_string(&ListAllMyBucketsResult {
            buckets: Buckets { buckets },
            owner: Owner {
                id: ID {
                    content: "AIDACKCEVSQ6C2EXAMPLE".into(),
                },
                display_name: DisplayName {
                    content: "Account+Name".into(),
                },
            },
        })
        .unwrap(),
    ))
}

#[cfg(test)]
mod tests {
    use actix_web::{http, App};

    use s3_entities::test::storage_provider::get_mock_app_data;

    use super::*;

    #[actix_web::test]
    async fn no_payload() {
        let provider = get_mock_app_data();
        provider
            .create_bucket("DOC-EXAMPLE-BUCKET", Some("Europe".into()))
            .await
            .unwrap();
        provider
            .create_bucket("DOC-EXAMPLE-BUCKET2", Some("Europe".into()))
            .await
            .unwrap();

        let app = actix_web::test::init_service(
            App::new()
                .app_data(actix_web::web::Data::from(provider))
                .service(list),
        )
        .await;
        let req = actix_web::test::TestRequest::get()
            .uri(&format!("/"))
            .to_request();
        let resp = actix_web::test::call_service(&app, req).await;

        assert_eq!(resp.status(), http::StatusCode::OK);
        println!("{:?}", resp.response().body())
    }

    #[test]
    fn test_serialization() {
        let res = quick_xml::se::to_string(&ListAllMyBucketsResult {
            buckets: Buckets {
                buckets: vec![
                    Bucket {
                        creation_date: "2019-12-11T23:32:47+00:00".into(),
                        name: "DOC-EXAMPLE-BUCKET".into(),
                    },
                    Bucket {
                        creation_date: "2019-12-11T23:32:47+00:00".into(),
                        name: "DOC-EXAMPLE-BUCKET2".into(),
                    },
                ],
            },
            owner: Owner {
                id: ID {
                    content: "AIDACKCEVSQ6C2EXAMPLE".into(),
                },
                display_name: DisplayName {
                    content: "Account+Name".into(),
                },
            },
        })
        .unwrap();

        assert_eq!(
            res,
            "<ListAllMyBucketsResult>\
                <Buckets>\
                    <Bucket>\
                        <CreationDate>2019-12-11T23:32:47+00:00</CreationDate>\
                        <String>DOC-EXAMPLE-BUCKET</String>\
                    </Bucket>\
                    <Bucket>\
                        <CreationDate>2019-12-11T23:32:47+00:00</CreationDate>\
                        <String>DOC-EXAMPLE-BUCKET2</String>\
                    </Bucket>\
                </Buckets>\
                <Owner>\
                    <ID>AIDACKCEVSQ6C2EXAMPLE</ID>\
                    <DisplayName>Account+Name</DisplayName>\
                </Owner>\
            </ListAllMyBucketsResult>"
        )
    }
}
