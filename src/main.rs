#[macro_use] extern crate rocket;

mod auth;
mod chat;
mod db;
mod websocket;
mod models;
mod upload;

use rocket::fs::{FileServer, relative};
use rocket::response::content::RawHtml;
use rocket::http::Status;
use sqlx::postgres::PgPoolOptions;
use dotenv::dotenv;
use std::fs;

pub struct AppState {
    db: sqlx::PgPool,
}

#[get("/")]
async fn index() -> Result<RawHtml<String>, Status> {
    match fs::read_to_string(relative!("static/index.html")) {
        Ok(content) => Ok(RawHtml(content)),
        Err(_) => Err(Status::NotFound)
    }
}


#[launch]
async fn rocket() -> _ {
    dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to create pool");

    // println!("Running database migrations...");
    // match sqlx::migrate!("./migrations")
    //     .run(&pool)
    //     .await {
    //     Ok(_) => println!("Migrations completed successfully"),
    //     Err(e) => {
    //         eprintln!("Error running migrations: {}", e);
    //         std::process::exit(1);
    //     }
    // }

    let state = AppState { db: pool };
    let ws_state = websocket::WebSocketState::new();

    rocket::build()
        .manage(state)
        .manage(ws_state)
        .mount("/", routes![
            index,
            upload::upload_file,
            auth::login,
            auth::register,
            auth::get_users,
            chat::get_messages,
            chat::send_message,
            chat::get_chats,
            websocket::ws_handler,
            websocket::send_ws_message
        ])
        .mount("/static", FileServer::from(relative!("static")))
        .mount("/uploads", FileServer::from("uploads"))
}