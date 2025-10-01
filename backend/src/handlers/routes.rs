use std::time::Duration;

use axum::{
    Json, Router,
    extract::{Path, State},
    http::{HeaderValue, Method, StatusCode, header},
    response::Redirect,
    routing::{get, post},
};

use base62;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::instrument;

use super::common::{build_url, internal_error};
use super::db::{mongodb_lookup, mongodb_put};
use super::redis::{get_idx, redis_get_key, redis_set_ex};
use crate::models::{AppState, PostData, UrlResponse, Urls};
use crate::{BASE_URL, FRONTEND_URL};

// defining routes and state
pub fn app(state: AppState) -> Router {
    Router::new()
        .route("/shorten", post(create_url))
        .route("/{short_code}", get(redirect_url))
        .layer(
            CorsLayer::new()
                .allow_origin(FRONTEND_URL.parse::<HeaderValue>().unwrap())
                .allow_methods([Method::GET, Method::POST])
                .allow_headers([header::CONTENT_TYPE]),
        )
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

// handler to create a new URL
#[instrument]
async fn create_url(
    State(mut state): State<AppState>,
    Json(input): Json<PostData>,
) -> Result<Json<UrlResponse>, (StatusCode, String)> {
    // we need the redis counter, but for now use rand id
    tracing::debug!("Creating URL for: {}", input.url);

    let id = get_idx(&mut state.redis).await?;
    let short_code = format!("{:0>8}", base62::encode(id));
    let duration = Duration::from_secs(input.expiration_days * 24 * 60 * 60);
    let long_url = build_url(&input.url);
    if long_url.contains(FRONTEND_URL.as_str()) || long_url.contains(BASE_URL.as_str()) {
        return Err((StatusCode::BAD_REQUEST, "Circular redirect".to_string()));
    }
    let url = Urls {
        id: short_code.clone(),
        long_url,
        // Use expiration_days from input, convert to seconds
        expiration_date: mongodb::bson::DateTime::now().saturating_add_duration(duration),
    };
    mongodb_put(&state.mongodb, url)
        .await
        .map_err(internal_error)?;
    // build url with header
    let short_url = format!("{}/{}", *BASE_URL, short_code);
    let response = UrlResponse {
        short_code: short_url,
    };
    Ok(Json(response))
}

// handler to redirect using short code
#[instrument]
async fn redirect_url(
    State(mut state): State<AppState>,
    Path(short_code): Path<String>,
) -> Result<Redirect, (StatusCode, String)> {
    // Try Redis cache first
    if let Ok(Some(url)) = redis_get_key(&mut state.redis, &short_code).await {
        return Ok(Redirect::temporary(&url));
    }

    // Fallback to MongoDB
    let url_doc = mongodb_lookup(&state.mongodb, &short_code)
        .await
        .map_err(internal_error)?
        .ok_or((StatusCode::NOT_FOUND, "Short URL not found".to_string()))?;

    // should we block this?
    // immediate use of url after shortening
    let long_url = url_doc.long_url.clone();
    tokio::spawn(async move {
        if let Err(e) = redis_set_ex(&mut state.redis, &short_code, &url_doc.long_url, 600).await {
            // Log error but don't fail the request
            tracing::warn!("Failed to update Redis cache: {:?}", e);
        }
    });

    Ok(Redirect::temporary(&long_url))
}
