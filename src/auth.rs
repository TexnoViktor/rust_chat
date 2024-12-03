use rocket::State;
use rocket::http::{Cookie, CookieJar, Status};
use rocket::serde::json::Json;
use bcrypt::{hash, verify, DEFAULT_COST};
use jsonwebtoken::{encode, decode, Header, EncodingKey, DecodingKey, Validation};
use chrono::{DateTime, Utc};
use serde_json::json;
use crate::AppState;
use crate::models::{User, LoginRequest, Claims};

const JWT_SECRET: &[u8] = b"your-secret-key";

#[post("/login", format = "json", data = "<login>")]
pub async fn login(
    state: &State<AppState>,
    login: Json<LoginRequest>,
    cookies: &CookieJar<'_>
) -> Result<Json<serde_json::Value>, Status> {
    let db = &state.db;

    match sqlx::query_as!(
        User,
        r#"SELECT id, username, password, created_at as "created_at!: DateTime<Utc>"
        FROM users WHERE username = $1"#,
        login.username
    )
        .fetch_one(db)
        .await {
        Ok(user) => {
            if verify(&login.password, &user.password).unwrap_or(false) {
                let token = create_jwt(user.id);
                cookies.add(Cookie::new("token", token));

                Ok(Json(json!({
                    "status": "success",
                    "user": json!({
                        "id": user.id,
                        "username": user.username,
                        "created_at": user.created_at
                    })
                })))
            } else {
                Ok(Json(json!({
                    "status": "error",
                    "message": "Invalid password"
                })))
            }
        }
        Err(_) => Ok(Json(json!({
            "status": "error",
            "message": "User not found"
        })))
    }
}

pub fn create_jwt(user_id: i32) -> String {
    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::days(1))
        .unwrap()
        .timestamp() as usize;

    let claims = Claims {
        sub: user_id,
        exp: expiration,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(JWT_SECRET)
    ).unwrap()
}

#[post("/register", format = "json", data = "<register>")]
pub async fn register(
    state: &State<AppState>,
    register: Json<LoginRequest>
) -> Result<Json<serde_json::Value>, Status> {
    let db = &state.db;
    let hashed_password = hash(register.password.as_bytes(), DEFAULT_COST)
        .map_err(|_| Status::InternalServerError)?;

    match sqlx::query_as!(
        User,
        r#"INSERT INTO users (username, password)
        VALUES ($1, $2)
        RETURNING id, username, password, created_at as "created_at!: DateTime<Utc>""#,
        register.username,
        hashed_password
    )
        .fetch_one(db)
        .await {
        Ok(user) => Ok(Json(json!({
            "status": "success",
            "user": json!({
                "id": user.id,
                "username": user.username,
                "created_at": user.created_at
            })
        }))),
        Err(_) => Ok(Json(json!({
            "status": "error",
            "message": "Username already taken"
        })))
    }
}

#[get("/users")]
pub async fn get_users(
    state: &State<AppState>,
    cookies: &CookieJar<'_>
) -> Result<Json<serde_json::Value>, Status> {
    if let Some(cookie) = cookies.get("token") {
        if let Ok(claims) = verify_jwt(cookie.value()) {
            let db = &state.db;

            match sqlx::query_as!(
                User,
                r#"SELECT id, username, password, created_at as "created_at!: DateTime<Utc>"
                FROM users
                WHERE id != $1"#,
                claims.sub
            )
                .fetch_all(db)
                .await {
                Ok(users) => Ok(Json(json!({
                    "status": "success",
                    "users": users.into_iter().map(|u| json!({
                        "id": u.id,
                        "username": u.username,
                        "created_at": u.created_at
                    })).collect::<Vec<_>>()
                }))),
                Err(_) => Ok(Json(json!({
                    "status": "error",
                    "message": "Failed to fetch users"
                })))
            }
        } else {
            Err(Status::Unauthorized)
        }
    } else {
        Err(Status::Unauthorized)
    }
}

pub fn verify_jwt(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(JWT_SECRET),
        &Validation::default()
    ).map(|data| data.claims)
}