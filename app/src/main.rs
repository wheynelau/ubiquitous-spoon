use axum::{
    Json, Router,
    extract::{Path, State},
    http::{StatusCode, header::LOCATION},
    response::Redirect,
    routing::{get, post},
};

use mongodb::{
    Client, Collection,
    bson::doc,
};

use serde::{Deserialize, Serialize};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use base62;
use rand::{self, Rng};

pub mod handlers;
pub mod services;
pub mod telemetry;

#[tokio::main]
async fn main() {
    // connecting to mongodb
    let db_connection_str = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "mongodb://admin:password@127.0.0.1:27017/?authSource=admin".to_string()
    });
    let client = Client::with_uri_str(db_connection_str).await.unwrap();

    // pinging the database
    client
        .database("axum-mongo")
        .run_command(doc! { "ping": 1 })
        .await
        .unwrap();
    println!("Pinged your database. Successfully connected to MongoDB!");

    // logging middleware
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!("{}=debug,tower_http=debug", env!("CARGO_CRATE_NAME")).into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // run it
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    tracing::debug!("Listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app(client)).await.unwrap();
}

// defining routes and state
fn app(client: Client) -> Router {
    let collection: Collection<Urls> = client.database("axum-mongo").collection("members");

    Router::new()
        .route("/shorten", post(create_url))
        .route("/{short_code}", get(redirect_url))
        .layer(TraceLayer::new_for_http())
        .with_state(collection)
}

// handler to create a new URL
async fn create_url(
    State(db): State<Collection<Urls>>,
    Json(input): Json<PostData>,
) -> Result<Json<UrlResponse>, (StatusCode, String)> {
    // we need the redis counter, but for now use rand id
    
    let random_id: u64 = {
        let mut rng = rand::rng();
        rng.random_range(0..1_000_000)
    };
    let short_code = base62::encode(random_id);
    let url = Urls {
        id: short_code.clone(),
        long_url: input.url,
        expiration_date: input.expiration_date,
    };
    db.insert_one(url).await.map_err(internal_error)?;
    let response = UrlResponse {
        short_code: short_code
    };
    Ok(Json(response))
}

// handler to redirect using short code
async fn redirect_url(
    State(db): State<Collection<Urls>>,
    Path(short_code): Path<String>,
) -> Result<Redirect, (StatusCode, String)> {
    let result = db
        .find_one(doc! { "_id": &short_code })
        .await
        .map_err(internal_error)?;

    match result {
        Some(url_doc) => {
            let url = if url_doc.long_url.starts_with("http://") || url_doc.long_url.starts_with("https://") {
                url_doc.long_url
            } else {
                format!("https://{}", url_doc.long_url)
            };
            Ok(Redirect::temporary(&url))
        },
        None => Err((StatusCode::NOT_FOUND, "Short URL not found".to_string())),
    }
}

fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}

// defining the url shortener structure
#[derive(Debug, Deserialize, Serialize)]
struct Urls {
    #[serde(rename = "_id")]
    id: String, // hash serving as short code
    long_url: String,
    expiration_date: String, // placeholder, should ideally be a different format
}

#[derive(Debug,Deserialize, Serialize)]
struct PostData {
    url: String,
    #[serde(rename = "expiration-date")]
    expiration_date: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct UrlResponse {
    short_code: String
}