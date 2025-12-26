use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Chat message stored in database
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatMessage {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub room_id: String,
    pub sender_id: String,
    pub sender_username: Option<String>,
    pub content: String,
    pub message_type: MessageType,
    pub created_at: DateTime<Utc>,
}

/// Type of message
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MessageType {
    Text,
    Image,
    File,
    System,
}

/// Chat room
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatRoom {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub room_id: String,
    pub name: String,
    pub room_type: RoomType,
    pub participants: Vec<String>, // user IDs
    pub created_by: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Type of chat room
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum RoomType {
    Direct, // 1-to-1 chat
    Group,  // Group chat
    Public, // Public room
}

/// WebSocket message from client
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    /// Join a chat room
    Join { room_id: String },
    /// Leave a chat room
    Leave { room_id: String },
    /// Send a message
    Message { room_id: String, content: String },
    /// Typing indicator
    Typing { room_id: String },
    /// Stop typing indicator
    StopTyping { room_id: String },
    /// Ping to keep connection alive
    Ping,
}

/// WebSocket message to client
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    /// Connection established
    Connected { user_id: String, session_id: String },
    /// Joined a room
    Joined { room_id: String },
    /// Left a room
    Left { room_id: String },
    /// New message in room
    Message {
        room_id: String,
        sender_id: String,
        sender_username: Option<String>,
        content: String,
        timestamp: String,
    },
    /// User started typing
    UserTyping { room_id: String, user_id: String },
    /// User stopped typing
    UserStopTyping { room_id: String, user_id: String },
    /// User joined room
    UserJoined { room_id: String, user_id: String },
    /// User left room
    UserLeft { room_id: String, user_id: String },
    /// Error message
    Error { message: String },
    /// Pong response
    Pong,
}

/// Request to create a chat room
#[derive(Debug, Deserialize)]
pub struct CreateRoomRequest {
    pub name: String,
    pub room_type: RoomType,
    pub participants: Vec<String>,
}

/// Request to send a message (REST endpoint)
#[derive(Debug, Deserialize)]
pub struct SendMessageRequest {
    pub room_id: String,
    pub content: String,
}
