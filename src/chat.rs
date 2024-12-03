use rocket::State;
use rocket::http::{CookieJar, Status};
use rocket::serde::json::Json;
use chrono::{DateTime, Utc};
use serde_json::json;
use crate::AppState;
use crate::models::{Message, MessageRequest, User};
use crate::auth::verify_jwt;

#[post("/message", format = "json", data = "<message>")]
pub async fn send_message(
    state: &State<AppState>,
    cookies: &CookieJar<'_>,
    message: Json<MessageRequest>
) -> Result<Json<serde_json::Value>, Status> {
    if let Some(cookie) = cookies.get("token") {
        if let Ok(claims) = verify_jwt(cookie.value()) {
            if claims.sub != message.from_user_id {
                return Ok(Json(json!({
                    "status": "error",
                    "message": "Unauthorized sender"
                })));
            }

            match sqlx::query_as!(
                Message,
                r#"INSERT INTO messages
                (from_user_id, to_user_id, content, message_type, file_path)
                VALUES ($1, $2, $3, $4, $5)
                RETURNING id, from_user_id as "from_user_id!",
                    to_user_id as "to_user_id!", content,
                    message_type, file_path,
                    created_at as "created_at!: DateTime<Utc>""#,
                message.from_user_id,
                message.to_user_id,
                message.content,
                message.message_type,
                message.file_path
            )
                .fetch_one(&state.db)
                .await {
                Ok(saved_message) => Ok(Json(json!({
                    "status": "success",
                    "message": saved_message
                }))),
                Err(e) => {
                    println!("Database error: {:?}", e);
                    Ok(Json(json!({
                        "status": "error",
                        "message": "Failed to save message"
                    })))
                }
            }
        } else {
            Err(Status::Unauthorized)
        }
    } else {
        Err(Status::Unauthorized)
    }
}

#[get("/messages/<other_user_id>")]
pub async fn get_messages(
    state: &State<AppState>,
    cookies: &CookieJar<'_>,
    other_user_id: i32
) -> Result<Json<serde_json::Value>, Status> {
    if let Some(cookie) = cookies.get("token") {
        if let Ok(claims) = verify_jwt(cookie.value()) {
            match sqlx::query_as!(
                Message,
                r#"SELECT id,
                    from_user_id as "from_user_id!",
                    to_user_id as "to_user_id!",
                    content,
                    message_type,
                    file_path,
                    created_at as "created_at!: DateTime<Utc>"
                FROM messages
                WHERE (from_user_id = $1 AND to_user_id = $2)
                    OR (from_user_id = $2 AND to_user_id = $1)
                ORDER BY created_at DESC
                LIMIT 50"#,
                claims.sub,
                other_user_id
            )
                .fetch_all(&state.db)
                .await {
                Ok(messages) => Ok(Json(json!({
                    "status": "success",
                    "messages": messages
                }))),
                Err(_) => Ok(Json(json!({
                    "status": "error",
                    "message": "Failed to fetch messages"
                })))
            }
        } else {
            Err(Status::Unauthorized)
        }
    } else {
        Err(Status::Unauthorized)
    }
}

#[get("/chats")]
pub async fn get_chats(
    state: &State<AppState>,
    cookies: &CookieJar<'_>
) -> Result<Json<serde_json::Value>, Status> {
    if let Some(cookie) = cookies.get("token") {
        if let Ok(claims) = verify_jwt(cookie.value()) {
            match sqlx::query_as!(
                User,
                r#"SELECT id, username, password, created_at as "created_at!: DateTime<Utc>"
                FROM users u
                WHERE EXISTS (
                    SELECT 1 FROM messages m
                    WHERE (m.from_user_id = $1 AND m.to_user_id = u.id)
                    OR (m.from_user_id = u.id AND m.to_user_id = $1)
                )
                AND u.id != $1"#,
                claims.sub
            )
                .fetch_all(&state.db)
                .await {
                Ok(users) => {
                    let chat_list = users.into_iter()
                        .map(|u| json!({
                            "id": u.id,
                            "username": u.username,
                            "created_at": u.created_at
                        }))
                        .collect::<Vec<_>>();

                    Ok(Json(json!({
                        "status": "success",
                        "chats": chat_list
                    })))
                },
                Err(_) => Ok(Json(json!({
                    "status": "error",
                    "message": "Failed to fetch chats"
                })))
            }
        } else {
            Err(Status::Unauthorized)
        }
    } else {
        Err(Status::Unauthorized)
    }
}