use super::controller::{ws_chat, ws_chat_with_token};
use actix_web::web;

pub fn chat_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/ws")
            .route("/chat", web::get().to(ws_chat))
            .route("/chat/token", web::get().to(ws_chat_with_token)),
    );
}
