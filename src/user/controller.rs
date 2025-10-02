use crate::user::service::UserService;
use crate::utils::model::LoginRequests;
use crate::{user::model::CreateUserRequest, utils::error::CustomError};
use actix_web::{HttpResponse, web};
pub async fn register_user(
    user_service: web::Data<UserService>,
    user_info: web::Json<CreateUserRequest>,
) -> Result<HttpResponse, CustomError> {
    let user_id = user_service
        .create_user(
            user_info.username.clone(),
            user_info.email.clone(),
            user_info.password.clone(), // password should be 3rd
            user_info.phone_number.clone(),
        )
        .await
        .map_err(|arg0| arg0)?; // Use the same CustomError type throughout

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "User created successfully",
        "httpStatusCode": 200,
        "service": std::env::var("SERVICE_NAME").unwrap_or_else(|_| "Unknown".to_string()),
        "user_id": user_id.to_hex()
    })))
}

pub async fn login_user(
    user_service: web::Data<UserService>,
    login_info: web::Json<LoginRequests>,
) -> Result<HttpResponse, CustomError> {
    let token = user_service.login_fn(login_info.into_inner()).await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "Login successful",
        "httpStatusCode": 200,
        "service": std::env::var("SERVICE_NAME").unwrap_or_else(|_| "Unknown".to_string()),
        "token": token
    })))
}
