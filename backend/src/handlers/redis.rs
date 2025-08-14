use axum::http::StatusCode;
use redis::{AsyncCommands, pipe};
use tracing::instrument;

const KEY: &str = "counter";

// Post increment idx
#[instrument]
pub async fn redis_get_idx(
    redis: &mut redis::aio::MultiplexedConnection,
) -> Result<u64, (StatusCode, String)> {
    let (_, new_val): (u64, u64) = pipe()
        .atomic() // This makes it a transaction (MULTI/EXEC)
        .get(KEY)
        .incr(KEY, 1)
        .query_async(redis)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(new_val)
}
#[instrument]
pub async fn redis_set_ex(
    redis: &mut redis::aio::MultiplexedConnection,
    key: &str,
    value: &str,
    seconds: u64,
) -> Result<(), (StatusCode, String)> {
    let _: () = redis
        .set_ex(key, value, seconds)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(())
}
#[instrument]
pub async fn redis_get_key(
    redis: &mut redis::aio::MultiplexedConnection,
    key: &str,
) -> Result<Option<String>, (StatusCode, String)> {
    let value: Option<String> = redis
        .get(key)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(value)
}
