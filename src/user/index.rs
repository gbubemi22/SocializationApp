use super::controller::{login_user, logout_user, register_user, resend_otp, verify_email};
use actix_web::web;

pub fn user_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/auth/user")
            .route("/register", web::post().to(register_user))
            .route("/verify-email", web::post().to(verify_email))
            .route("/resend-otp", web::post().to(resend_otp))
            .route("/login", web::post().to(login_user))
            .route("/logout", web::post().to(logout_user)),
    );
}
