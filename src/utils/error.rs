use actix_web::{HttpResponse, ResponseError, http::StatusCode};
use serde_json::json;
use thiserror::Error;

#[allow(dead_code)]
#[derive(Debug, Error)]
pub enum CustomError {
    #[error("Unauthorized: {0}")]
    UnauthorizedError(String),

    #[error("Bad Request: {0}")]
    BadRequestError(String),

    #[error("Conflict: {0}")]
    ConflictError(String),

    #[error("Internal Server Error: {0}")]
    InternalServerError(String),

    #[error("Unauthenticated: {0}")]
    UnauthenticatedError(String),

    #[error("Not Found: {0}")]
    NotFoundError(String),

    #[error("Validation Error: {0}")]
    ValidationError(String),
}

impl ResponseError for CustomError {
    fn status_code(&self) -> StatusCode {
        match *self {
            CustomError::UnauthorizedError(..) => StatusCode::UNAUTHORIZED,
            CustomError::BadRequestError(..) => StatusCode::BAD_REQUEST,
            CustomError::ConflictError(..) => StatusCode::CONFLICT,
            CustomError::InternalServerError(..) => StatusCode::INTERNAL_SERVER_ERROR,
            CustomError::UnauthenticatedError(..) => StatusCode::UNAUTHORIZED,
            CustomError::NotFoundError(..) => StatusCode::NOT_FOUND,
            CustomError::ValidationError(..) => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let error_message = json!({
            "success": false,
            "message": self.to_string(),
            "httpStatusCode": self.status_code().as_u16(),
            "error": match *self {
                CustomError::UnauthorizedError(..) => "UNAUTHORIZED_ERROR",
                CustomError::BadRequestError(..) => "BAD_REQUEST_ERROR",
                CustomError::ConflictError(..) => "CONFLICT_ERROR",
                CustomError::InternalServerError(..) => "INTERNAL_SERVER_ERROR",
                CustomError::UnauthenticatedError(..) => "UNAUTHENTICATED_ERROR",
                CustomError::NotFoundError(..) => "NOT_FOUND_ERROR",
                CustomError::ValidationError(..) => "VALIDATION_ERROR",
            },
            "service": std::env::var("SERVICE_NAME").unwrap_or_else(|_| "Unknown".to_string()),
        });

        HttpResponse::build(self.status_code()).json(error_message)
    }
}
