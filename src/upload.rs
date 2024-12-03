use rocket::Data;
use rocket::serde::json::{Json, Value};
use std::path::{Path, PathBuf};
use rocket::data::ToByteUnit;
use serde_json::json;
use uuid::Uuid;
use tokio::fs::{File, create_dir_all};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[post("/upload", data = "<file>")]
pub async fn upload_file(mut file: Data<'_>) -> Result<Json<Value>, std::io::Error> {
    let uploads_dir = Path::new("uploads");
    if !uploads_dir.exists() {
        create_dir_all(uploads_dir).await?;
    }

    let file_name = format!("{}.webm", Uuid::new_v4());
    let file_path = uploads_dir.join(&file_name);

    let mut buffer = Vec::new();
    file.open(512.kibibytes()).read_to_end(&mut buffer).await?;

    let mut f = File::create(&file_path).await?;
    f.write_all(&buffer).await?;

    Ok(Json(json!({
        "status": "success",
        "file_path": format!("/uploads/{}", file_name)
    })))
}