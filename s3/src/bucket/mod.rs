use actix_web::web;

mod create;
mod delete;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("")
            .service(create::create)
            .service(delete::delete_bucket)
    );
}
