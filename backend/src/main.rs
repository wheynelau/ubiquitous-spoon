use mongodb::{Client, bson::doc};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use redis::AsyncCommands;
use redis::Client as RedisClient;
use redis::aio::prefer_tokio;

pub mod handlers;
pub mod models;
pub mod services;
pub mod telemetry;

use models::AppState;

#[tokio::main]
async fn main() {
    // use tokio
    let _ = prefer_tokio();

    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1/".into());
    let redis_client = RedisClient::open(redis_url).unwrap();
    let mut redis_connection = redis_client
        .get_multiplexed_async_connection()
        .await
        .unwrap();

    // connecting to mongodb
    let db_connection_str = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "mongodb://admin:password@127.0.0.1:27017/?authSource=admin".to_string()
    });
    let mongodb_client = Client::with_uri_str(db_connection_str).await.unwrap();

    // pinging the database
    mongodb_client
        .database("axum-mongo")
        .run_command(doc! { "ping": 1 })
        .await
        .unwrap();
    println!("Pinged your database. Successfully connected to MongoDB!");

    // Check redis?
    let _: () = redis_connection.set("key", "value").await.unwrap();

    println!("Connected to Redis!");

    let state = AppState {
        mongodb: mongodb_client.database("axum-mongo").collection("members"),
        redis: redis_connection,
    };

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
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::debug!("Listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, handlers::app(state)).await.unwrap();
}
