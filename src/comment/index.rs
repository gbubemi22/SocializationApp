use super::controller::{
    create_comment, delete_comment, get_comment, get_comment_count, get_post_comments,
    update_comment,
};
use actix_web::web;

pub fn comment_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/comments")
            .route("", web::post().to(create_comment))
            .route("/post/{post_id}", web::get().to(get_post_comments))
            .route("/count/{post_id}", web::get().to(get_comment_count))
            .route("/{comment_id}", web::get().to(get_comment))
            .route("/{comment_id}", web::put().to(update_comment))
            .route("/{comment_id}", web::delete().to(delete_comment)),
    );
}
