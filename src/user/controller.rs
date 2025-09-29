use actix_web::{HttpResponse, Responder, web};
use crate::user::model::CreateUserRequest;
use crate::utils::model::LoginRequests;
use crate::user::service::UserService;

pub async fn register_user(
    user_service: web::Data<UserService>,
    user_info: web::Json<CreateUserRequest>,
) -> impl Responder {
    match user_service
        .create_user(
            user_info.username.clone(),
            user_info.email.clone(),
            user_info.phone_number.clone(),
            user_info.password.clone(),
        )
        .await
    {
        Ok(user_id) => HttpResponse::Ok().json(serde_json::json!({
            "message": "User created successfully",
            "user_id": user_id.to_hex()
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "message": "Failed to create user",
            "error": e.to_string()
        })),
    }
}

pub async fn login_user(
    user_service: web::Data<UserService>,
    login_info: web::Json<LoginRequests>,
) -> impl Responder {
    match user_service.login_fn(login_info.into_inner()).await {
        Ok(token) => HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "message": "Login successful".to_string(),
            "token": token
        })),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({
            "success": false,
            "error": e.to_string()
        })),
    }
}
