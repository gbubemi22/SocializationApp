use std::env;

use crate::database::RedisService;
use crate::utils::error::CustomError;
use actix_web::{Error, HttpMessage, dev::ServiceRequest, web};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use jsonwebtoken::{DecodingKey, Validation, decode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub id: String,
    pub exp: usize,
}

/// Verify JWT token and validate session in Redis
pub async fn verify_token(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    let token = credentials.token();
    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "secret".to_string());

    // First decode the JWT
    let token_data = match decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    ) {
        Ok(data) => data,
        Err(_) => return Err((actix_web::error::ErrorUnauthorized("Invalid token"), req)),
    };

    let user_id = &token_data.claims.id;

    // Get Redis service from app data
    let redis_service = match req.app_data::<web::Data<RedisService>>() {
        Some(service) => service,
        None => {
            // If Redis is not available, just validate JWT (fallback mode)
            req.extensions_mut().insert(token_data.claims);
            return Ok(req);
        }
    };

    // Validate session in Redis
    match redis_service.validate_session(token).await {
        Ok(Some(stored_user_id)) => {
            // Check if the user_id matches
            if stored_user_id == *user_id {
                req.extensions_mut().insert(token_data.claims);
                Ok(req)
            } else {
                Err((actix_web::error::ErrorUnauthorized("Session mismatch"), req))
            }
        }
        Ok(None) => {
            // Token not found in Redis - session expired or user logged out
            Err((
                actix_web::error::ErrorUnauthorized("Session expired or invalid"),
                req,
            ))
        }
        Err(_) => {
            // Redis error - fallback to just JWT validation
            req.extensions_mut().insert(token_data.claims);
            Ok(req)
        }
    }
}

/// Create a JWT token and store session in Redis
pub async fn create_token_with_session(
    user_id: &str,
    redis_service: &RedisService,
) -> Result<String, Error> {
    let secret = env::var("JWT_SECRET")
        .map_err(|_| CustomError::UnauthorizedError("JWT_SECRET must be set".to_string()))?;

    // Token expires in 24 hours
    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::hours(24))
        .expect("valid timestamp")
        .timestamp() as usize;

    let claims = Claims {
        id: user_id.to_owned(),
        exp: expiration,
    };

    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|_| CustomError::BadRequestError("Token generation failed".to_string()))?;

    // Store session in Redis (24 hours = 86400 seconds)
    redis_service
        .store_session(user_id, &token, 86400)
        .await
        .map_err(|e| CustomError::InternalServerError(format!("Failed to store session: {}", e)))?;

    Ok(token)
}

/// Create a JWT token without Redis session (for backward compatibility)
pub async fn create_token(user_id: &str) -> Result<String, Error> {
    let secret = env::var("JWT_SECRET")
        .map_err(|_| CustomError::UnauthorizedError("JWT_SECRET must be set".to_string()))?;
    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::hours(24))
        .expect("valid timestamp")
        .timestamp() as usize;

    let claims = Claims {
        id: user_id.to_owned(),
        exp: expiration,
    };

    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|_| CustomError::BadRequestError("Token generation failed".to_string()))?;

    Ok(token)
}

/// Invalidate a user's session (logout)
pub async fn invalidate_session(user_id: &str, redis_service: &RedisService) -> Result<(), Error> {
    redis_service
        .invalidate_session(user_id)
        .await
        .map_err(|e| {
            CustomError::InternalServerError(format!("Failed to invalidate session: {}", e))
        })?;

    Ok(())
}

/// Get user ID from request extensions (use after auth middleware)
pub fn get_user_id_from_request(req: &actix_web::HttpRequest) -> Option<String> {
    req.extensions()
        .get::<Claims>()
        .map(|claims| claims.id.clone())
}
