use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use rocket::State;
use rocket::http::Status;
use rocket::serde::json::Json;
use tokio::sync::broadcast;
use tokio::sync::broadcast::Sender;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use serde_json::json;
use crate::AppState;
use crate::models::Message;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketMessage {
    pub message_type: String,
    pub content: String,
    pub from_user_id: i32,
    pub to_user_id: i32,
    pub file_path: Option<String>,
}

pub struct WebSocketState {
    connections: Arc<Mutex<HashMap<i32, Sender<WebSocketMessage>>>>,
}

impl WebSocketState {
    pub fn new() -> Self {
        Self {
            connections: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn add_connection(&self, user_id: i32) -> broadcast::Receiver<WebSocketMessage> {
        let (tx, rx) = broadcast::channel(100);
        self.connections.lock().unwrap().insert(user_id, tx);
        rx
    }

    pub fn remove_connection(&self, user_id: i32) {
        self.connections.lock().unwrap().remove(&user_id);
    }

    pub fn send_message(&self, msg: WebSocketMessage) {
        if let Some(tx) = self.connections.lock().unwrap().get(&msg.to_user_id) {
            let _ = tx.send(msg);
        }
    }
}

#[get("/ws/<user_id>")]
pub async fn ws_handler(
    user_id: i32,
    _state: &State<AppState>,
    ws_state: &State<WebSocketState>,
) -> Result<Json<serde_json::Value>, Status> {
    let _rx = ws_state.add_connection(user_id);
    Ok(Json(json!({ "status": "connected" })))
}

#[post("/ws/send", format = "json", data = "<message>")]
pub async fn send_ws_message(
    message: Json<WebSocketMessage>,
    ws_state: &State<WebSocketState>,
    state: &State<AppState>,
) -> Result<Json<serde_json::Value>, Status> {
    let db = &state.db;
    let db_message = Message {
        id: 0,
        from_user_id: message.from_user_id,
        to_user_id: message.to_user_id,
        content: message.content.clone(),
        message_type: message.message_type.clone(),
        file_path: message.file_path.clone(),
        created_at: chrono::Utc::now(),
    };

    match sqlx::query_as!(
        Message,
        r#"INSERT INTO messages (from_user_id, to_user_id, content, message_type, file_path)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, from_user_id as "from_user_id!",
            to_user_id as "to_user_id!", content,
            message_type, file_path,
            created_at as "created_at!: DateTime<Utc>""#,
        db_message.from_user_id,
        db_message.to_user_id,
        db_message.content,
        db_message.message_type,
        db_message.file_path
    )
        .fetch_one(db)
        .await {
        Ok(_) => {
            ws_state.send_message(message.0);
            Ok(Json(json!({ "status": "success" })))
        },
        Err(_) => Ok(Json(json!({
            "status": "error",
            "message": "Failed to save message"
        })))
    }
}