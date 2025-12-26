use actix::Addr;
use actix_web::{HttpRequest, HttpResponse, web};
use actix_web_actors::ws;

use crate::chat::server::ChatServer;
use crate::chat::session::WsSession;
use crate::middleware::auth::Claims;
use crate::utils::error::CustomError;

/// WebSocket connection handler
/// GET /ws/chat
pub async fn ws_chat(
    req: HttpRequest,
    stream: web::Payload,
    server: web::Data<Addr<ChatServer>>,
) -> Result<HttpResponse, actix_web::Error> {
    // Get user_id from auth (JWT claims in request extensions)
    let user_id = req
        .extensions()
        .get::<Claims>()
        .map(|claims| claims.id.clone())
        .unwrap_or_else(|| "anonymous".to_string());

    log::info!("WebSocket connection request from user: {}", user_id);

    // Create WebSocket session
    let session = WsSession::new(user_id, server.get_ref().clone());

    // Start WebSocket connection
    ws::start(session, &req, stream)
}

/// WebSocket connection with token in query parameter (for clients that can't set headers)
/// GET /ws/chat?token=<jwt_token>
pub async fn ws_chat_with_token(
    req: HttpRequest,
    stream: web::Payload,
    server: web::Data<Addr<ChatServer>>,
    query: web::Query<TokenQuery>,
) -> Result<HttpResponse, actix_web::Error> {
    // Validate JWT token from query parameter
    let user_id = validate_token(&query.token).unwrap_or_else(|_| "anonymous".to_string());

    log::info!("WebSocket connection request from user: {}", user_id);

    // Create WebSocket session
    let session = WsSession::new(user_id, server.get_ref().clone());

    // Start WebSocket connection
    ws::start(session, &req, stream)
}

#[derive(serde::Deserialize)]
pub struct TokenQuery {
    pub token: String,
}

/// Validate JWT token and extract user_id
fn validate_token(token: &str) -> Result<String, CustomError> {
    use jsonwebtoken::{DecodingKey, Validation, decode};

    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "secret".to_string());

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|_| CustomError::UnauthorizedError("Invalid token".to_string()))?;

    Ok(token_data.claims.id)
}
