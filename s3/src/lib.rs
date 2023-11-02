#![allow(clippy::all)]

use actix_web::{App, HttpServer};

pub mod bucket;
pub mod storage_provider;

#[actix_web::main]
async fn start() -> std::io::Result<()> {
    HttpServer::new(|| 
        App::new()
            .configure(bucket::config)
    )
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}

pub fn main() -> Result<(), std::io::Error> {
    start()
}