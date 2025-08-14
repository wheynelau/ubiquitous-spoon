use mongodb::Collection;
use redis::aio::MultiplexedConnection;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Urls {
    #[serde(rename = "_id")]
    pub id: String, // hash serving as short code
    pub long_url: String,
    pub expiration_date: String, // placeholder, should ideally be a different format
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PostData {
    pub url: String,
    pub expiration_date: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UrlResponse {
    pub short_code: String,
}

#[derive(Clone, Debug)]
pub struct AppState {
    pub mongodb: Collection<Urls>,
    pub redis: MultiplexedConnection,
}
