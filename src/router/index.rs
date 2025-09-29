use actix_web::web;
use crate::user::index::user_routes;
// later you’ll add: use crate::post::index::post_routes; etc.

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.configure(user_routes);
    // cfg.configure(post_routes);
    // cfg.configure(comment_routes);
}