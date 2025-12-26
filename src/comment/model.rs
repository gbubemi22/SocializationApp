use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Comment {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub post_id: ObjectId,
    pub author_id: ObjectId,
    pub author_username: Option<String>,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct CreateCommentRequest {
    pub post_id: String,
    pub content: String,
}

#[derive(Deserialize)]
pub struct UpdateCommentRequest {
    pub content: String,
}
