use mongodb::bson::{doc, oid::ObjectId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub username: String,
    pub email: String,
    pub phone_number: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct LoginRequests {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
    pub phone_number: String,
    pub password: String,
}

#[derive(serde::Serialize)]
pub struct LoginResponse {
    pub success: bool,
    pub message: String,
    pub token: String,
}
