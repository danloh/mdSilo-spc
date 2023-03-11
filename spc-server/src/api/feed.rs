use axum::{extract::{State, Path}, response::IntoResponse, Json};
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};

use crate::{util::feed::process_feed, db::{feed::{Feed, Channel}, user::{ClaimCan, BASIC_PERMIT}}};

#[derive(Debug, Serialize, Deserialize)]
pub struct FeedResult {
  pub channel: Channel,
  pub articles: Vec<Feed>,
}

/// Handler for the `/api/fetchfeed/{url}` endpoint.
pub async fn fetch_feed(
  Path(url): Path<String>,
  check: ClaimCan<BASIC_PERMIT>,
) -> Result<impl IntoResponse, StatusCode> {
  if !check.can() {
    return Err(StatusCode::UNAUTHORIZED);
  }
  match process_feed(&url).await {
    Some(res) => {
      let channel = res.0;
      let articles = res.1;

      return Ok(Json(FeedResult { channel, articles }))
    }
    None => return Err(StatusCode::BAD_REQUEST),
  }
}

