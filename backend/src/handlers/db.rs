use tracing::instrument;

use mongodb::{Collection, bson::doc, results::InsertOneResult};

use crate::models::Urls;

#[instrument]
pub(super) async fn mongodb_lookup(
    state: &Collection<Urls>,
    short_code: &str,
) -> Result<Option<Urls>, mongodb::error::Error> {
    state.find_one(doc! { "_id": short_code }).await
}

#[instrument]
pub(super) async fn mongodb_put(
    state: &Collection<Urls>,
    url: Urls,
) -> Result<InsertOneResult, mongodb::error::Error> {
    state.insert_one(url).await
}
