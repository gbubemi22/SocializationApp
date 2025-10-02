use super::post_controller::{create_post, delete_post, get_post};
use crate::middleware::auth::verify_token;
use actix_web::web;
use actix_web_httpauth::middleware::HttpAuthentication;

pub fn post_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/posts")
            .wrap(HttpAuthentication::bearer(verify_token))
            .route("", web::post().to(create_post))
            .route("/{id}", web::get().to(get_post))
            .route("/{id}", web::delete().to(delete_post)),
    );
}
