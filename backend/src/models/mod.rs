use mongodb::Collection;
use redis::aio::MultiplexedConnection;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Urls {
    #[serde(rename = "_id")]
    pub id: String, // hash serving as short code
    pub long_url: String,
    pub expiration_date: mongodb::bson::DateTime,
}

#[derive(Debug, Validate, Deserialize, Serialize)]
pub struct PostData {
    pub url: String,
    #[validate(range(min = 1, exclusive_max = 365))]
    #[serde(default = "default_expiration_days")]
    pub expiration_days: u64,
}

fn default_expiration_days() -> u64 {
    7
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
