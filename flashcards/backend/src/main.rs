use axum::{
    response::{Html, IntoResponse},
    routing::get,
    Router,
    Json,
};
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, fs};
use tower_http::cors::{Any, CorsLayer};

#[derive(Deserialize, Serialize, Debug)]
struct Flashcard {
    word: String,
    pinyin: Option<String>,
    translation: String,
}

async fn import_csv() -> impl IntoResponse {
    let data = fs::read_to_string("flashcards.csv").unwrap_or_default();
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(data.as_bytes());
    let mut flashcards = Vec::new();

    for result in rdr.records() {
        if let Ok(record) = result {
            let word = record.get(0).unwrap_or("").to_string();
            let pinyin = record.get(1).map(|s| s.to_string());
            let translation = record.get(2).unwrap_or("").to_string();
            flashcards.push(Flashcard { word, pinyin, translation });
        }
    }

    Json(flashcards)
}

async fn health() -> impl IntoResponse {
    Html("<h1>Backend is running 🚀</h1>")
}

#[tokio::main]
async fn main() {
    let cors = CorsLayer::new().allow_origin(Any);

    let app = Router::new()
        .route("/", get(health))
        .route("/api/import", get(import_csv))
        .layer(cors);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    println!("Backend running on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    axum::serve(listener, app).await.unwrap();
}
