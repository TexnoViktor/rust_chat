use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i32,
    pub exp: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserResponse {
    pub id: i32,
    pub username: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub password: String,
    pub created_at: DateTime<Utc>,
}

impl User {
    pub fn into_response(self) -> UserResponse {
        UserResponse {
            id: self.id,
            username: self.username,
            created_at: self.created_at,
        }
    }
}

// Повна структура повідомлення з бази даних
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Message {
    pub id: i32,
    pub from_user_id: i32,
    pub to_user_id: i32,
    pub content: String,
    pub message_type: String,
    pub file_path: Option<String>,
    pub created_at: DateTime<Utc>,
}

// Структура для прийому повідомлення від клієнта
#[derive(Debug, Serialize, Deserialize)]
pub struct MessageRequest {
    pub from_user_id: i32,
    pub to_user_id: i32,
    pub content: String,
    pub message_type: String,
    pub file_path: Option<String>,
}