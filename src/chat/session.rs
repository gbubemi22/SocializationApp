use actix::{Actor, ActorContext, Addr, AsyncContext, Handler, Running, StreamHandler};
use actix_web_actors::ws;
use std::time::{Duration, Instant};
use uuid::Uuid;

use crate::chat::model::{ClientMessage, ServerMessage};
use crate::chat::server::{
    ChatServer, Connect, Disconnect, JoinRoom, LeaveRoom, RoomMessage, WsMessage,
};

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

/// WebSocket session actor
pub struct WsSession {
    /// Unique session id
    pub session_id: String,
    /// User id (from JWT auth)
    pub user_id: String,
    /// Chat server address
    pub server_addr: Addr<ChatServer>,
    /// Last heartbeat timestamp
    pub last_heartbeat: Instant,
}

impl WsSession {
    pub fn new(user_id: String, server_addr: Addr<ChatServer>) -> Self {
        WsSession {
            session_id: Uuid::new_v4().to_string(),
            user_id,
            server_addr,
            last_heartbeat: Instant::now(),
        }
    }

    /// Start heartbeat process
    fn start_heartbeat(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            // Check client heartbeat
            if Instant::now().duration_since(act.last_heartbeat) > CLIENT_TIMEOUT {
                log::warn!("WebSocket client heartbeat timeout, disconnecting");
                ctx.stop();
                return;
            }
            ctx.ping(b"");
        });
    }

    /// Handle incoming client message
    fn handle_message(&mut self, msg: ClientMessage, ctx: &mut ws::WebsocketContext<Self>) {
        match msg {
            ClientMessage::Join { room_id } => {
                self.server_addr.do_send(JoinRoom {
                    session_id: self.session_id.clone(),
                    room_id,
                });
            }
            ClientMessage::Leave { room_id } => {
                self.server_addr.do_send(LeaveRoom {
                    session_id: self.session_id.clone(),
                    room_id,
                });
            }
            ClientMessage::Message { room_id, content } => {
                let message = ServerMessage::Message {
                    room_id: room_id.clone(),
                    sender_id: self.user_id.clone(),
                    sender_username: None, // TODO: fetch username
                    content,
                    timestamp: chrono::Utc::now().to_rfc3339(),
                };
                self.server_addr.do_send(RoomMessage {
                    room_id,
                    sender_session_id: self.session_id.clone(),
                    message,
                });
            }
            ClientMessage::Typing { room_id } => {
                let message = ServerMessage::UserTyping {
                    room_id: room_id.clone(),
                    user_id: self.user_id.clone(),
                };
                self.server_addr.do_send(RoomMessage {
                    room_id,
                    sender_session_id: self.session_id.clone(),
                    message,
                });
            }
            ClientMessage::StopTyping { room_id } => {
                let message = ServerMessage::UserStopTyping {
                    room_id: room_id.clone(),
                    user_id: self.user_id.clone(),
                };
                self.server_addr.do_send(RoomMessage {
                    room_id,
                    sender_session_id: self.session_id.clone(),
                    message,
                });
            }
            ClientMessage::Ping => {
                self.send_message(&ServerMessage::Pong, ctx);
            }
        }
    }

    /// Send message to WebSocket client
    fn send_message(&self, msg: &ServerMessage, ctx: &mut ws::WebsocketContext<Self>) {
        if let Ok(json) = serde_json::to_string(msg) {
            ctx.text(json);
        }
    }
}

impl Actor for WsSession {
    type Context = ws::WebsocketContext<Self>;

    /// Called when actor starts
    fn started(&mut self, ctx: &mut Self::Context) {
        // Start heartbeat
        self.start_heartbeat(ctx);

        // Register with chat server
        let addr = ctx.address();
        self.server_addr.do_send(Connect {
            session_id: self.session_id.clone(),
            user_id: self.user_id.clone(),
            addr: addr.recipient(),
        });
    }

    /// Called when actor is stopping
    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        // Notify chat server of disconnect
        self.server_addr.do_send(Disconnect {
            session_id: self.session_id.clone(),
        });
        Running::Stop
    }
}

/// Handler for WsMessage from chat server
impl Handler<WsMessage> for WsSession {
    type Result = ();

    fn handle(&mut self, msg: WsMessage, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}

/// Handler for WebSocket messages
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                self.last_heartbeat = Instant::now();
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {
                self.last_heartbeat = Instant::now();
            }
            Ok(ws::Message::Text(text)) => {
                self.last_heartbeat = Instant::now();

                // Parse client message
                match serde_json::from_str::<ClientMessage>(&text) {
                    Ok(client_msg) => {
                        self.handle_message(client_msg, ctx);
                    }
                    Err(e) => {
                        log::warn!("Failed to parse WebSocket message: {}", e);
                        self.send_message(
                            &ServerMessage::Error {
                                message: format!("Invalid message format: {}", e),
                            },
                            ctx,
                        );
                    }
                }
            }
            Ok(ws::Message::Binary(_)) => {
                log::warn!("Binary messages not supported");
            }
            Ok(ws::Message::Close(reason)) => {
                log::info!("WebSocket close: {:?}", reason);
                ctx.close(reason);
                ctx.stop();
            }
            _ => ctx.stop(),
        }
    }
}
