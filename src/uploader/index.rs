use super::controller::{upload_multiple, upload_single};
use actix_web::web;

pub fn upload_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/upload")
            .route("/single", web::post().to(upload_single))
            .route("/multiple", web::post().to(upload_multiple)),
    );
}
