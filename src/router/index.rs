use crate::post::post_index::post_routes;
use crate::user::index::user_routes;
use actix_web::web;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.configure(user_routes);
    cfg.configure(post_routes);
    // cfg.configure(comment_routes);
}
