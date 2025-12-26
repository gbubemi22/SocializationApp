use crate::chat::index::chat_routes;
use crate::comment::index::comment_routes;
use crate::post::post_index::post_routes;
use crate::uploader::index::upload_routes;
use crate::user::index::user_routes;
use actix_web::web;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.configure(user_routes);
    cfg.configure(post_routes);
    cfg.configure(upload_routes);
    cfg.configure(comment_routes);
    cfg.configure(chat_routes);
}
