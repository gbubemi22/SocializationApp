use actix_web::http::StatusCode;
use actix_web::middleware::ErrorHandlerResponse;
use actix_web::{HttpResponse, Result, dev::ServiceResponse};
use serde_json::json;

pub fn not_found<B>(res: ServiceResponse<B>) -> Result<ErrorHandlerResponse<B>> {
    let new_response = HttpResponse::build(StatusCode::NOT_FOUND).json(json!({
        "success": false,
        "message": "Route does not exist",
        "httpStatusCode": StatusCode::NOT_FOUND.as_u16(),
        "error": "NOT_FOUND_ERROR",
        "service": std::env::var("SERVICE_NAME").unwrap_or_else(|_| "Unknown".to_string()),
    }));
    let (req, _) = res.into_parts();
    let res = ServiceResponse::new(req, new_response.map_into_right_body());

    Ok(ErrorHandlerResponse::Response(res))
}
