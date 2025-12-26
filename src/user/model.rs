use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub username: String,
    pub email: String,
    pub password: String,
    pub phone_number: String,
    pub profile_picture: Option<String>,
    pub is_email_verified: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
    pub password: String,
    pub phone_number: String,
}

/// OTP model for email verification
#[derive(Debug, Serialize, Deserialize)]
pub struct Otp {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub user_id: ObjectId,
    pub email: String,
    pub code: String,
    pub expires_at: DateTime<Utc>,
    pub is_used: bool,
    pub created_at: DateTime<Utc>,
}

/// Request body for email verification
#[derive(Deserialize)]
pub struct VerifyEmailRequest {
    pub email: String,
    pub otp_code: String,
}

/// Request body for resending OTP
#[derive(Deserialize)]
pub struct ResendOtpRequest {
    pub email: String,
}
