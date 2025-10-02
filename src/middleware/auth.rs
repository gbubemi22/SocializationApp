use std::env;

use crate::utils::error::CustomError;
use actix_web::{Error, HttpMessage, dev::ServiceRequest};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use jsonwebtoken::{DecodingKey, Validation, decode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub id: String,
    pub exp: usize,
}

pub async fn verify_token(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    let token = credentials.token();
    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "secret".to_string());

    match decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    ) {
        Ok(token_data) => {
            // ✅ Insert claims into request extensions
            req.extensions_mut().insert(token_data.claims);
            Ok(req)
        }
        Err(_) => Err((actix_web::error::ErrorUnauthorized("Invalid token"), req)),
    }
}

// pub async fn verify_token(req: &HttpRequest) -> Result<String, Error> {
//     let token = req
//         .headers()
//         .get("Authorization")
//         .and_then(|header| header.to_str().ok())
//         .and_then(|auth_header| auth_header.strip_prefix("Bearer "));

//         debug!("Extracted token: {:?}", token);

//     match token {
//         Some(token) => {
//             let secret = env::var("JWT_SECRET").map_err(|_| {
//                 CustomError::UnauthorizedError("JWT_SECRET must be set".to_string())
//             })?;
//             let key = DecodingKey::from_secret(secret.as_bytes());
//             let validation = Validation::default();

//             match decode::<Claims>(token, &key, &validation) {
//                 Ok(token_data) => Ok(token_data.claims.id),
//                 Err(_) => Err(CustomError::UnauthorizedError("Invalid token".to_string()).into()),
//             }
//         }
//         None => Err(CustomError::UnauthorizedError(
//             "Authorization header is missing or invalid".to_string(),
//         )
//         .into()),
//     }
// }

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

    //  let claims = Claims {
    //         id: user.id.unwrap().to_string(),
    //         exp: (Utc::now() + Duration::hours(24)).timestamp() as usize,
    //     };

    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|_| CustomError::BadRequestError("Token generation failed".to_string()))?;

    Ok(token)
}
