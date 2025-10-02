use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Post {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub title: String,
    pub content: String,
    pub author_id: ObjectId,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct CreatePostRequest {
    pub title: String,
    pub content: String,
}
