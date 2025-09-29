use actix_web::web;
use super::controller::{login_user, register_user};

pub fn user_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/auth/user")
            .route("/register", web::post().to(register_user))
            .route("/login", web::post().to(login_user)),
    );
}
