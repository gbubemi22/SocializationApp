use crate::database::RedisService;
use crate::middleware::auth::{get_user_id_from_request, invalidate_session};
use crate::user::model::{CreateUserRequest, ResendOtpRequest, VerifyEmailRequest};
use crate::user::service::UserService;
use crate::utils::error::CustomError;
use crate::utils::model::LoginRequests;
use actix_web::{HttpRequest, HttpResponse, web};

pub async fn register_user(
    user_service: web::Data<UserService>,
    user_info: web::Json<CreateUserRequest>,
) -> Result<HttpResponse, CustomError> {
    let user_id = user_service
        .create_user(
            user_info.username.clone(),
            user_info.email.clone(),
            user_info.password.clone(),
            user_info.phone_number.clone(),
        )
        .await
        .map_err(|arg0| arg0)?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "User created successfully. Please check your email for verification code.",
        "httpStatusCode": 200,
        "service": std::env::var("SERVICE_NAME").unwrap_or_else(|_| "Unknown".to_string()),
        "user_id": user_id.to_hex()
    })))
}

pub async fn verify_email(
    user_service: web::Data<UserService>,
    body: web::Json<VerifyEmailRequest>,
) -> Result<HttpResponse, CustomError> {
    user_service
        .verify_email(&body.email, &body.otp_code)
        .await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "Email verified successfully. You can now login.",
        "httpStatusCode": 200,
        "service": std::env::var("SERVICE_NAME").unwrap_or_else(|_| "Unknown".to_string())
    })))
}

pub async fn resend_otp(
    user_service: web::Data<UserService>,
    body: web::Json<ResendOtpRequest>,
) -> Result<HttpResponse, CustomError> {
    user_service.resend_otp(&body.email).await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "Verification code sent to your email.",
        "httpStatusCode": 200,
        "service": std::env::var("SERVICE_NAME").unwrap_or_else(|_| "Unknown".to_string())
    })))
}

pub async fn login_user(
    user_service: web::Data<UserService>,
    redis_service: web::Data<RedisService>,
    login_info: web::Json<LoginRequests>,
) -> Result<HttpResponse, CustomError> {
    let token = user_service
        .login_fn(login_info.into_inner(), Some(redis_service.get_ref()))
        .await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "Login successful",
        "httpStatusCode": 200,
        "service": std::env::var("SERVICE_NAME").unwrap_or_else(|_| "Unknown".to_string()),
        "token": token
    })))
}

pub async fn logout_user(
    req: HttpRequest,
    redis_service: web::Data<RedisService>,
) -> Result<HttpResponse, CustomError> {
    // Get user ID from request (set by auth middleware)
    let user_id = get_user_id_from_request(&req)
        .ok_or_else(|| CustomError::UnauthorizedError("Not authenticated".to_string()))?;

    // Invalidate session in Redis
    invalidate_session(&user_id, redis_service.get_ref())
        .await
        .map_err(|_| CustomError::InternalServerError("Failed to logout".to_string()))?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "Logged out successfully",
        "httpStatusCode": 200,
        "service": std::env::var("SERVICE_NAME").unwrap_or_else(|_| "Unknown".to_string())
    })))
}
