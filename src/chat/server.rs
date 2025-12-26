use crate::chat::model::ServerMessage;
use actix::prelude::*;
use std::collections::{HashMap, HashSet};

/// Message sent to chat server to connect a session
#[derive(Message)]
#[rtype(result = "()")]
pub struct Connect {
    pub session_id: String,
    pub user_id: String,
    pub addr: Recipient<WsMessage>,
}

/// Message sent to chat server when session disconnects
#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub session_id: String,
}

/// Message for joining a room
#[derive(Message)]
#[rtype(result = "()")]
pub struct JoinRoom {
    pub session_id: String,
    pub room_id: String,
}

/// Message for leaving a room
#[derive(Message)]
#[rtype(result = "()")]
pub struct LeaveRoom {
    pub session_id: String,
    pub room_id: String,
}

/// Message for broadcasting to a room
#[derive(Message)]
#[rtype(result = "()")]
pub struct RoomMessage {
    pub room_id: String,
    pub sender_session_id: String,
    pub message: ServerMessage,
}

/// WebSocket message wrapper
#[derive(Message)]
#[rtype(result = "()")]
pub struct WsMessage(pub String);

/// Session info
#[derive(Clone)]
pub struct SessionInfo {
    pub user_id: String,
    pub addr: Recipient<WsMessage>,
}

/// Chat server actor - manages rooms and sessions
pub struct ChatServer {
    /// Map of session_id -> session info
    sessions: HashMap<String, SessionInfo>,
    /// Map of room_id -> set of session_ids
    rooms: HashMap<String, HashSet<String>>,
    /// Map of user_id -> session_id (for direct messaging)
    user_sessions: HashMap<String, String>,
}

impl ChatServer {
    pub fn new() -> Self {
        ChatServer {
            sessions: HashMap::new(),
            rooms: HashMap::new(),
            user_sessions: HashMap::new(),
        }
    }

    /// Send message to all sessions in a room
    fn send_to_room(&self, room_id: &str, message: &ServerMessage, skip_session: Option<&str>) {
        if let Some(sessions) = self.rooms.get(room_id) {
            let msg_json = serde_json::to_string(message).unwrap_or_default();
            for session_id in sessions {
                if skip_session.map_or(true, |s| s != session_id) {
                    if let Some(session) = self.sessions.get(session_id) {
                        let _ = session.addr.do_send(WsMessage(msg_json.clone()));
                    }
                }
            }
        }
    }

    /// Send message to a specific session
    fn send_to_session(&self, session_id: &str, message: &ServerMessage) {
        if let Some(session) = self.sessions.get(session_id) {
            let msg_json = serde_json::to_string(message).unwrap_or_default();
            let _ = session.addr.do_send(WsMessage(msg_json));
        }
    }
}

impl Default for ChatServer {
    fn default() -> Self {
        Self::new()
    }
}

impl Actor for ChatServer {
    type Context = Context<Self>;
}

/// Handler for Connect message
impl Handler<Connect> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) {
        log::info!(
            "User {} connected with session {}",
            msg.user_id,
            msg.session_id
        );

        // Store session
        self.sessions.insert(
            msg.session_id.clone(),
            SessionInfo {
                user_id: msg.user_id.clone(),
                addr: msg.addr,
            },
        );

        // Map user to session
        self.user_sessions
            .insert(msg.user_id.clone(), msg.session_id.clone());

        // Send connected confirmation
        self.send_to_session(
            &msg.session_id,
            &ServerMessage::Connected {
                user_id: msg.user_id,
                session_id: msg.session_id,
            },
        );
    }
}

/// Handler for Disconnect message
impl Handler<Disconnect> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        log::info!("Session {} disconnected", msg.session_id);

        // Get user_id before removing session
        if let Some(session) = self.sessions.get(&msg.session_id) {
            let user_id = session.user_id.clone();

            // Remove from user_sessions
            self.user_sessions.remove(&user_id);

            // Remove from all rooms and notify
            for (room_id, sessions) in self.rooms.iter_mut() {
                if sessions.remove(&msg.session_id) {
                    // Notify room that user left
                    let msg = ServerMessage::UserLeft {
                        room_id: room_id.clone(),
                        user_id: user_id.clone(),
                    };
                    let msg_json = serde_json::to_string(&msg).unwrap_or_default();
                    for session_id in sessions.iter() {
                        if let Some(s) = self.sessions.get(session_id) {
                            let _ = s.addr.do_send(WsMessage(msg_json.clone()));
                        }
                    }
                }
            }
        }

        // Remove session
        self.sessions.remove(&msg.session_id);
    }
}

/// Handler for JoinRoom message
impl Handler<JoinRoom> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: JoinRoom, _: &mut Context<Self>) {
        log::info!("Session {} joining room {}", msg.session_id, msg.room_id);

        // Add session to room
        self.rooms
            .entry(msg.room_id.clone())
            .or_insert_with(HashSet::new)
            .insert(msg.session_id.clone());

        // Get user_id for notification
        let user_id = self
            .sessions
            .get(&msg.session_id)
            .map(|s| s.user_id.clone())
            .unwrap_or_default();

        // Notify room that user joined
        self.send_to_room(
            &msg.room_id,
            &ServerMessage::UserJoined {
                room_id: msg.room_id.clone(),
                user_id,
            },
            Some(&msg.session_id),
        );

        // Send joined confirmation to session
        self.send_to_session(
            &msg.session_id,
            &ServerMessage::Joined {
                room_id: msg.room_id,
            },
        );
    }
}

/// Handler for LeaveRoom message
impl Handler<LeaveRoom> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: LeaveRoom, _: &mut Context<Self>) {
        log::info!("Session {} leaving room {}", msg.session_id, msg.room_id);

        // Get user_id for notification
        let user_id = self
            .sessions
            .get(&msg.session_id)
            .map(|s| s.user_id.clone())
            .unwrap_or_default();

        // Remove session from room
        if let Some(sessions) = self.rooms.get_mut(&msg.room_id) {
            sessions.remove(&msg.session_id);
        }

        // Notify room that user left
        self.send_to_room(
            &msg.room_id,
            &ServerMessage::UserLeft {
                room_id: msg.room_id.clone(),
                user_id,
            },
            None,
        );

        // Send left confirmation to session
        self.send_to_session(
            &msg.session_id,
            &ServerMessage::Left {
                room_id: msg.room_id,
            },
        );
    }
}

/// Handler for RoomMessage
impl Handler<RoomMessage> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: RoomMessage, _: &mut Context<Self>) {
        self.send_to_room(&msg.room_id, &msg.message, None);
    }
}
