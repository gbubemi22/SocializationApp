use crate::comment::model::{CreateCommentRequest, UpdateCommentRequest};
use crate::comment::service::CommentService;
use crate::middleware::auth::get_user_id_from_request;
use crate::utils::error::CustomError;
use actix_web::{HttpRequest, HttpResponse, web};
use mongodb::bson::oid::ObjectId;
use serde_json::json;

/// Create a new comment on a post
/// POST /comments
pub async fn create_comment(
    req: HttpRequest,
    comment_service: web::Data<CommentService>,
    body: web::Json<CreateCommentRequest>,
) -> Result<HttpResponse, CustomError> {
    // Get user ID from auth middleware
    let user_id_str = get_user_id_from_request(&req)
        .ok_or_else(|| CustomError::UnauthorizedError("Not authenticated".to_string()))?;

    let author_id = ObjectId::parse_str(&user_id_str)
        .map_err(|_| CustomError::BadRequestError("Invalid user ID".to_string()))?;

    let post_id = ObjectId::parse_str(&body.post_id)
        .map_err(|_| CustomError::BadRequestError("Invalid post ID".to_string()))?;

    if body.content.trim().is_empty() {
        return Err(CustomError::BadRequestError(
            "Comment content cannot be empty".to_string(),
        ));
    }

    let comment_id = comment_service
        .add_comment(post_id, author_id, None, body.content.clone())
        .await?;

    Ok(HttpResponse::Created().json(json!({
        "success": true,
        "message": "Comment created successfully",
        "httpStatusCode": 201,
        "comment_id": comment_id.to_hex()
    })))
}

/// Get all comments for a post
/// GET /comments/post/{post_id}
pub async fn get_post_comments(
    comment_service: web::Data<CommentService>,
    path: web::Path<String>,
) -> Result<HttpResponse, CustomError> {
    let post_id = ObjectId::parse_str(path.into_inner())
        .map_err(|_| CustomError::BadRequestError("Invalid post ID".to_string()))?;

    let comments = comment_service.get_comments_for_post(&post_id).await?;
    let count = comments.len();

    Ok(HttpResponse::Ok().json(json!({
        "success": true,
        "message": "Comments retrieved successfully",
        "httpStatusCode": 200,
        "count": count,
        "data": comments
    })))
}

/// Get a single comment by ID
/// GET /comments/{comment_id}
pub async fn get_comment(
    comment_service: web::Data<CommentService>,
    path: web::Path<String>,
) -> Result<HttpResponse, CustomError> {
    let comment_id = ObjectId::parse_str(path.into_inner())
        .map_err(|_| CustomError::BadRequestError("Invalid comment ID".to_string()))?;

    let comment = comment_service
        .get_comment_by_id(&comment_id)
        .await?
        .ok_or_else(|| CustomError::NotFoundError("Comment not found".to_string()))?;

    Ok(HttpResponse::Ok().json(json!({
        "success": true,
        "message": "Comment retrieved successfully",
        "httpStatusCode": 200,
        "data": comment
    })))
}

/// Update a comment
/// PUT /comments/{comment_id}
pub async fn update_comment(
    req: HttpRequest,
    comment_service: web::Data<CommentService>,
    path: web::Path<String>,
    body: web::Json<UpdateCommentRequest>,
) -> Result<HttpResponse, CustomError> {
    let user_id_str = get_user_id_from_request(&req)
        .ok_or_else(|| CustomError::UnauthorizedError("Not authenticated".to_string()))?;

    let author_id = ObjectId::parse_str(&user_id_str)
        .map_err(|_| CustomError::BadRequestError("Invalid user ID".to_string()))?;

    let comment_id = ObjectId::parse_str(path.into_inner())
        .map_err(|_| CustomError::BadRequestError("Invalid comment ID".to_string()))?;

    if body.content.trim().is_empty() {
        return Err(CustomError::BadRequestError(
            "Comment content cannot be empty".to_string(),
        ));
    }

    comment_service
        .update_comment(&comment_id, &author_id, body.content.clone())
        .await?;

    Ok(HttpResponse::Ok().json(json!({
        "success": true,
        "message": "Comment updated successfully",
        "httpStatusCode": 200
    })))
}

/// Delete a comment
/// DELETE /comments/{comment_id}
pub async fn delete_comment(
    req: HttpRequest,
    comment_service: web::Data<CommentService>,
    path: web::Path<String>,
) -> Result<HttpResponse, CustomError> {
    let user_id_str = get_user_id_from_request(&req)
        .ok_or_else(|| CustomError::UnauthorizedError("Not authenticated".to_string()))?;

    let author_id = ObjectId::parse_str(&user_id_str)
        .map_err(|_| CustomError::BadRequestError("Invalid user ID".to_string()))?;

    let comment_id = ObjectId::parse_str(path.into_inner())
        .map_err(|_| CustomError::BadRequestError("Invalid comment ID".to_string()))?;

    comment_service
        .delete_comment(&comment_id, &author_id)
        .await?;

    Ok(HttpResponse::Ok().json(json!({
        "success": true,
        "message": "Comment deleted successfully",
        "httpStatusCode": 200
    })))
}

/// Get comment count for a post
/// GET /comments/count/{post_id}
pub async fn get_comment_count(
    comment_service: web::Data<CommentService>,
    path: web::Path<String>,
) -> Result<HttpResponse, CustomError> {
    let post_id = ObjectId::parse_str(path.into_inner())
        .map_err(|_| CustomError::BadRequestError("Invalid post ID".to_string()))?;

    let count = comment_service.get_comment_count(&post_id).await?;

    Ok(HttpResponse::Ok().json(json!({
        "success": true,
        "message": "Comment count retrieved successfully",
        "httpStatusCode": 200,
        "count": count
    })))
}
