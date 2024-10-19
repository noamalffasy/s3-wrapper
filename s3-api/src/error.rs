extern crate self as s3_api;

use s3_derive::S3Error;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename = "Error")]
pub struct XmlError {
    #[serde(rename = "Code")]
    pub code: String,
    #[serde(rename = "Message")]
    pub message: String,
    #[serde(rename = "Resource")]
    pub resource: String,
    #[serde(rename = "RequestID")]
    pub request_id: String,
}

#[derive(Debug, S3Error)]
pub enum BaseError {
    #[error(status_code = 500, message = "An internal error occurred. Try again.")]
    InternalError {
        request_id: String,
        resource: String,
    },
}
