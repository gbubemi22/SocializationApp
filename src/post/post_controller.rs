use crate::middleware::auth::Claims;
use crate::post::post_model::CreatePostRequest;
use crate::post::post_service::PostService;
use crate::{post::post_model::Post, utils::error::CustomError};
use actix_web::{HttpMessage, HttpRequest, HttpResponse, web};
use mongodb::bson::oid::ObjectId;

pub async fn create_post(
    post_service: web::Data<PostService>,
    post: web::Json<CreatePostRequest>,
    req: HttpRequest, // ✅ Add HttpRequest parameter
) -> Result<HttpResponse, CustomError> {
    // ✅ Extract claims from request extensions
    let claims = req
        .extensions()
        .get::<Claims>()
        .ok_or_else(|| CustomError::UnauthorizedError("No claims found".into()))?
        .clone();

    // ✅ Author comes from token
    let author_id = match ObjectId::parse_str(&claims.id) {
        Ok(id) => id,
        Err(_) => {
            return Err(CustomError::BadRequestError(
                "Invalid user id in token".into(),
            ));
        }
    };

    // ✅ Create new post object
    let new_post = Post {
        id: ObjectId::new(),
        title: post.title.clone(),
        content: post.content.clone(),
        author_id,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    // ✅ Insert post using the service
    let inserted_post = post_service.create_post(new_post).await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "Post created successfully",
        "httpStatusCode": 200,
        "service": std::env::var("SERVICE_NAME").unwrap_or_else(|_| "Unknown".to_string()),
        "post": inserted_post
    })))
}

pub async fn get_post(
    post_id: web::Path<String>,
    post_service: web::Data<PostService>,
) -> Result<HttpResponse, CustomError> {
    let post_id = post_id.into_inner();
    let post = post_service.get_post(&post_id).await?;

    match post {
        Some(p) => Ok(HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "message": "Post fetched successfully",
            "httpStatusCode": 200,
            "service": std::env::var("SERVICE_NAME").unwrap_or_else(|_| "Unknown".to_string()),
            "post": p
        }))),
        None => Err(CustomError::NotFoundError("Post not found".into())),
    }
}

pub async fn delete_post(
    post_id: web::Path<String>,
    post_service: web::Data<PostService>,
) -> Result<HttpResponse, CustomError> {
    let post_id = post_id.into_inner();
    let deleted = post_service.delete_post(&post_id).await?;

    if deleted {
        Ok(HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "message": "Post deleted successfully",
            "httpStatusCode": 200,
            "service": std::env::var("SERVICE_NAME").unwrap_or_else(|_| "Unknown".to_string()),
        })))
    } else {
        Err(CustomError::NotFoundError("Post not found".into()))
    }
}
