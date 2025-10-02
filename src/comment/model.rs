use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Comment {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub post_id: ObjectId,   // links to Post
    pub author_id: ObjectId, // links to User
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
