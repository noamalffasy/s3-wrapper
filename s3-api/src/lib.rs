use actix_web::{web, App, HttpServer};
use rand::distributions::DistString;
use s3_entities::storage_provider::StorageProvider;
use std::sync::Arc;

pub mod bucket;
mod error;
mod xml;

fn generate_request_id() -> String {
    rand::distributions::Alphanumeric
        .sample_string(&mut rand::thread_rng(), 16)
        .to_uppercase()
}

#[actix_web::main]
async fn start(
    storage_provider: impl StorageProvider + Send + Sync + 'static,
) -> std::io::Result<()> {
    let arc_provider: Arc<dyn StorageProvider> = Arc::new(storage_provider);
    let provider: web::Data<dyn StorageProvider> = web::Data::from(arc_provider);

    HttpServer::new(move || {
        App::new()
            .app_data(provider.clone())
            .configure(bucket::config)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}

pub fn main(
    storage_provider: impl StorageProvider + Send + Sync + 'static,
) -> Result<(), std::io::Error> {
    start(storage_provider)
}
