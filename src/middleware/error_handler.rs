use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
// use actix_web::dev::ServiceResponse;
// use actix_web::middleware::ErrorHandlerResponse;
use serde_json::json;
use std::fmt;
#[allow(dead_code)]
#[derive(Debug)]
pub enum CustomError {
    ValidationError(String),
    DuplicateError(String),
    NotFoundError(String),
    InternalServerError(String),
}

impl fmt::Display for CustomError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CustomError::ValidationError(msg) => write!(f, "Validation Error: {}", msg),
            CustomError::DuplicateError(msg) => write!(f, "Duplicate Error: {}", msg),
            CustomError::NotFoundError(msg) => write!(f, "Not Found Error: {}", msg),
            CustomError::InternalServerError(msg) => write!(f, "Internal Server Error: {}", msg),
        }
    }
}

impl ResponseError for CustomError {
    fn status_code(&self) -> StatusCode {
        match *self {
            CustomError::ValidationError(_) => StatusCode::BAD_REQUEST,
            CustomError::DuplicateError(_) => StatusCode::CONFLICT,
            CustomError::NotFoundError(_) => StatusCode::NOT_FOUND,
            CustomError::InternalServerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let status_code = self.status_code();
        let error_message = self.to_string();
        let error_type = match self {
            CustomError::ValidationError(_) => "VALIDATION_ERROR",
            CustomError::DuplicateError(_) => "DUPLICATE_ERROR",
            CustomError::NotFoundError(_) => "NOT_FOUND_ERROR",
            CustomError::InternalServerError(_) => "INTERNAL_SERVER_ERROR",
        };

        HttpResponse::build(status_code).json(json!({
            "success": false,
            "message": error_message,
            "httpStatusCode": status_code.as_u16(),
            "error": error_type,
            "service": std::env::var("SERVICE_NAME").unwrap_or_else(|_| "Unknown".to_string()),
        }))
    }
}

// pub fn handle_error<B>(res: ServiceResponse<B>) -> Result<ErrorHandlerResponse<B>> {
//      let error_message = res.response().error().map(|e| e.to_string()).unwrap_or_else(|| "Unknown error".to_string());
//      let status_code = res.response().status();

//      let new_response = HttpResponse::build(status_code)
//          .json(json!({
//              "success": false,
//              "message": error_message,
//              "httpStatusCode": status_code.as_u16(),
//              "error": status_code.canonical_reason().unwrap_or("Unknown"),
//              "service": std::env::var("SERVICE_NAME").unwrap_or_else(|_| "Unknown".to_string()),
//          }));

//      let (req, _) = res.into_parts();
//      let res = ServiceResponse::new(
//          req,
//          new_response.map_into_right_body()
//      );

//      Ok(ErrorHandlerResponse::Response(res))
//  }
