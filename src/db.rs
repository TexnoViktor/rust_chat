use sqlx::PgPool;
use chrono::{DateTime, Utc};
use crate::models::{User, Message, UserResponse};

pub struct Database {
    pool: PgPool,
}

impl Database {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_user(&self, username: &str, password: &str) -> Result<User, sqlx::Error> {
        sqlx::query_as!(
            User,
            r#"INSERT INTO users (username, password)
            VALUES ($1, $2)
            RETURNING id, username, password, created_at as "created_at!: DateTime<Utc>""#,
            username,
            password
        )
            .fetch_one(&self.pool)
            .await
    }

    pub async fn get_user_by_username(&self, username: &str) -> Result<User, sqlx::Error> {
        sqlx::query_as!(
            User,
            r#"SELECT id, username, password, created_at as "created_at!: DateTime<Utc>"
            FROM users WHERE username = $1"#,
            username
        )
            .fetch_one(&self.pool)
            .await
    }

    pub async fn save_message(&self, message: &Message) -> Result<Message, sqlx::Error> {
        sqlx::query_as!(
            Message,
            r#"INSERT INTO messages (from_user_id, to_user_id, content, message_type, file_path)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, from_user_id as "from_user_id!",
                to_user_id as "to_user_id!", content, message_type,
                file_path, created_at as "created_at!: DateTime<Utc>""#,
            message.from_user_id,
            message.to_user_id,
            message.content,
            message.message_type,
            message.file_path
        )
            .fetch_one(&self.pool)
            .await
    }

    pub async fn get_messages(&self, user1_id: i32, user2_id: i32) -> Result<Vec<Message>, sqlx::Error> {
        sqlx::query_as!(
            Message,
            r#"SELECT id,
                from_user_id as "from_user_id!",
                to_user_id as "to_user_id!",
                content, message_type,
                file_path, created_at as "created_at!: DateTime<Utc>"
            FROM messages
            WHERE (from_user_id = $1 AND to_user_id = $2)
                OR (from_user_id = $2 AND to_user_id = $1)
            ORDER BY created_at DESC
            LIMIT 50"#,
            user1_id,
            user2_id
        )
            .fetch_all(&self.pool)
            .await
    }

    pub async fn get_user_chats(&self, user_id: i32) -> Result<Vec<UserResponse>, sqlx::Error> {
        let users = sqlx::query_as!(
            User,
            r#"SELECT DISTINCT u.id, u.username, u.password,
            u.created_at as "created_at!: DateTime<Utc>"
            FROM users u
            INNER JOIN messages m
            ON (m.from_user_id = u.id OR m.to_user_id = u.id)
            WHERE (m.from_user_id = $1 OR m.to_user_id = $1)
            AND u.id != $1"#,
            user_id
        )
            .fetch_all(&self.pool)
            .await?;

        Ok(users.into_iter().map(|u| u.into_response()).collect())
    }
}