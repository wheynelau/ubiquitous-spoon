use axum::http::StatusCode;
use redis::{AsyncCommands, pipe};
use std::sync::atomic::{AtomicU64, Ordering};
use tracing::instrument;

const KEY: &str = "counter";
static BATCH_UPPER: AtomicU64 = AtomicU64::new(0);
static BATCH_LOWER: AtomicU64 = AtomicU64::new(0);

pub async fn get_idx(
    redis: &mut redis::aio::MultiplexedConnection,
) -> Result<u64, (StatusCode, String)> {
    let curr = BATCH_LOWER.fetch_add(1, Ordering::SeqCst);
    if curr >= BATCH_UPPER.load(Ordering::SeqCst) {
        increment_redis_batch(redis).await?;
    }
    Ok(curr)
}

#[instrument]
async fn increment_redis_batch(
    redis: &mut redis::aio::MultiplexedConnection,
) -> Result<(), (StatusCode, String)> {
    // First check if the key exists, if not initialize it
    let exists: bool = redis
        .exists(KEY)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if !exists {
        let _: () = redis
            .set(KEY, 0)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    }

    let (old_val, new_val): (u64, u64) = pipe()
        .atomic() // This makes it a transaction (MULTI/EXEC)
        .get(KEY)
        .incr(KEY, 1000) // take a batch of 1000
        .query_async(redis)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    BATCH_UPPER.store(new_val, Ordering::SeqCst);
    BATCH_LOWER.store(old_val + 1, Ordering::SeqCst); // add one because we are incrementing the lower bound
    Ok(())
}

// Post increment idx
#[allow(dead_code)]
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

pub async fn initialize_counter(
    redis: &mut redis::aio::MultiplexedConnection,
) -> Result<(), (StatusCode, String)> {
    increment_redis_batch(redis).await
}
