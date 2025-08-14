use axum::{
    Json, Router,
    extract::{Path, State},
    http::{StatusCode, Method, HeaderValue, header},
    response::Redirect,
    routing::{get, post},
};

use base62;
use tower_http::{trace::TraceLayer, cors::CorsLayer};
use tracing::instrument;

use super::common::{build_url, internal_error};
use super::db::{mongodb_lookup, mongodb_put};
use super::redis::{redis_get_idx, redis_get_key, redis_set_ex};
use crate::models::{AppState, PostData, UrlResponse, Urls};

// defining routes and state
pub fn app(state: AppState) -> Router {
    Router::new()
        .route("/shorten", post(create_url))
        .route("/{short_code}", get(redirect_url))
        .layer(
            CorsLayer::new()
                .allow_origin(
                    std::env::var("FRONTEND_URL")
                        .unwrap_or_else(|_| "http://localhost:8080".into())
                        .parse::<HeaderValue>()
                        .unwrap()
                )
                .allow_methods([Method::GET, Method::POST])
                .allow_headers([header::CONTENT_TYPE])
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

    let id = redis_get_idx(&mut state.redis).await?;
    let short_code = base62::encode(id);
    let url = Urls {
        id: short_code.clone(),
        long_url: build_url(&input.url),
        expiration_date: input.expiration_date,
    };
    mongodb_put(&state.mongodb, url)
        .await
        .map_err(internal_error)?;
    // build url with header
    let short_url = format!(
        "{}/{}",
        std::env::var("BASE_URL").unwrap_or_else(|_| "http://localhost:3000".into()),
        short_code
    );
    let response = UrlResponse { short_code: short_url };
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

    let long_url = url_doc.long_url.clone();
    tokio::spawn(async move {
        if let Err(e) = redis_set_ex(&mut state.redis, &short_code, &url_doc.long_url, 600).await {
            // Log error but don't fail the request
            tracing::warn!("Failed to update Redis cache: {:?}", e);
        }
    });

    Ok(Redirect::temporary(&long_url))
}
