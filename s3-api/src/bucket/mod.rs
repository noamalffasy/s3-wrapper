use actix_web::web;

mod create;
mod delete;
mod list;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("")
            .service(list::list)
            .service(create::create)
            .service(delete::delete_bucket),
    );
}
