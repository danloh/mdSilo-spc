use axum::{extract::{State, Query}, response::IntoResponse, Json};
use axum::http::StatusCode;
use axum_macros::debug_handler;
use serde::{Deserialize, Serialize};
use log::warn;

use crate::{
  AppState as Ctx, 
  util::feed::process_feed, 
  db::{
    feed::{Feed, Channel, Subscription, FeedStatus}, 
    user::{ClaimCan, BASIC_PERMIT}
  }
};

#[derive(Deserialize)]
pub struct ApiQuery {
  pub url: Option<String>,
  // perpage: Option<i64>,
  // page: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FeedResult {
  pub channel: Channel,
  pub articles: Vec<Feed>,
}

/// Handler for the GET `/api/fetchfeed?url=` endpoint.
pub async fn fetch_feed(
  Query(param): Query<ApiQuery>,
  check: ClaimCan<BASIC_PERMIT>,
) -> Result<impl IntoResponse, StatusCode> {
  if !check.can() {
    return Err(StatusCode::UNAUTHORIZED);
  }
  let url = param.url.unwrap_or_default();
  if url.trim().len() == 0 {
    return Err(StatusCode::BAD_REQUEST);
  }

  match process_feed(&url, None, None).await {
    Some(res) => {
      let channel = res.0;
      let articles = res.1;

      return Ok(Json(FeedResult { channel, articles }))
    }
    None => return Err(StatusCode::BAD_REQUEST),
  }
}

#[derive(Deserialize)]
pub struct NewChannel {
  url: String,
  title: String,
  ty: Option<String>,
}

/// Handler for the POST `/api/add_channel` endpoint.
#[debug_handler]
pub async fn add_channel(
  State(ctx): State<Ctx>,
  check: ClaimCan<BASIC_PERMIT>,
  Json(payload): Json<NewChannel>,
) -> Result<impl IntoResponse, StatusCode> {
  if !check.can() {
    return Err(StatusCode::UNAUTHORIZED);
  }

  // TODO: move process feed to wasm
  match process_feed(&payload.url, payload.ty, Some(payload.title)).await {
    Some(res) => {
      let channel = res.0;
      let articles = res.1;

      // upsert channel
      channel.new(&ctx)
        .await
        .map_err(|e| warn!("add channel: {}", e))
        .unwrap_or_default();
      Feed::add_feeds(&ctx, articles)
        .await
        .map_err(|e| warn!("add feeds: {}", e))
        .unwrap_or(0);
      // subscription
      // let is_pub = if input.is_public == 0 { false } else { true };
      let claim = check.claim;
      let uname = claim.unwrap_or_default().uname;
      Subscription::new(
        &ctx, &uname, &channel.link, &channel.title, false
      )
      .await
      .map_err(|e| warn!("add subscription: {}", e))
      .unwrap_or_default();

      return Ok(Json(1))
    }
    None => return Err(StatusCode::BAD_REQUEST),
  }
}

/// Handler for the GET `/api/get_channels` endpoint.
#[debug_handler]
pub async fn get_sub_channels(
  State(ctx): State<Ctx>,
  check: ClaimCan<BASIC_PERMIT>,
) -> Result<impl IntoResponse, StatusCode> {
  if !check.can() {
    return Err(StatusCode::UNAUTHORIZED);
  }

  let claim = check.claim;
  let uname = claim.unwrap_or_default().uname;
  let res = Subscription::get_channel_list(&ctx, &uname)
    .await
    .map_err(|_e| StatusCode::BAD_REQUEST)?;

  return Ok(Json(res))
}

/// Handler for the GET `/api/get_feeds` endpoint.
#[debug_handler]
pub async fn get_feeds(
  State(ctx): State<Ctx>,
  check: ClaimCan<BASIC_PERMIT>,
) -> Result<impl IntoResponse, StatusCode> {
  if !check.can() {
    return Err(StatusCode::UNAUTHORIZED);
  }

  let claim = check.claim;
  let uname = claim.unwrap_or_default().uname;
  let res = Feed::get_list_by_user(&ctx, &uname, false, 64, 1)
    .await
    .map_err(|_e| StatusCode::BAD_REQUEST)?;

  return Ok(Json(res))
}

/// Handler for the GET `/api/get_channel_feeds?url=` endpoint.
#[debug_handler]
pub async fn get_feeds_by_channel(
  State(ctx): State<Ctx>,
  Query(param): Query<ApiQuery>,
  check: ClaimCan<BASIC_PERMIT>,
) -> Result<impl IntoResponse, StatusCode> {
  if !check.can() {
    return Err(StatusCode::UNAUTHORIZED);
  }
  
  let url = param.url.unwrap_or_default();
  if url.trim().len() == 0 {
    return Err(StatusCode::BAD_REQUEST);
  }
  let res = Feed::get_list_by_channel(&ctx, &url, 64, 1)
    .await
    .map_err(|_e| StatusCode::BAD_REQUEST)?;

  return Ok(Json(res))
}

/// Handler for the GET `/api/get_star_feeds` endpoint.
#[debug_handler]
pub async fn get_star_feeds(
  State(ctx): State<Ctx>,
  check: ClaimCan<BASIC_PERMIT>,
) -> Result<impl IntoResponse, StatusCode> {
  if !check.can() {
    return Err(StatusCode::UNAUTHORIZED);
  }

  let claim = check.claim;
  let uname = claim.unwrap_or_default().uname;
  let res = FeedStatus::get_star_list(&ctx, &uname)
    .await
    .map_err(|_e| StatusCode::BAD_REQUEST)?;

  return Ok(Json(res))
}

/// Handler for the GET `/api/get_read_feeds` endpoint.
#[debug_handler]
pub async fn get_read_feeds(
  State(ctx): State<Ctx>,
  check: ClaimCan<BASIC_PERMIT>,
) -> Result<impl IntoResponse, StatusCode> {
  if !check.can() {
    return Err(StatusCode::UNAUTHORIZED);
  }

  let claim = check.claim;
  let uname = claim.unwrap_or_default().uname;
  let res = FeedStatus::get_read_list(&ctx, &uname)
    .await
    .map_err(|_e| StatusCode::BAD_REQUEST)?;

  return Ok(Json(res))
}

/// Handler for the GET `/api/star_feed?url=` endpoint.
#[debug_handler]
pub async fn star_feed(
  State(ctx): State<Ctx>,
  Query(param): Query<ApiQuery>,
  check: ClaimCan<BASIC_PERMIT>,
) -> Result<impl IntoResponse, StatusCode> {
  if !check.can() {
    return Err(StatusCode::UNAUTHORIZED);
  }
  let url = param.url.unwrap_or_default();
  if url.trim().len() == 0 {
    return Err(StatusCode::BAD_REQUEST);
  }

  let claim = check.claim;
  let uname = claim.unwrap_or_default().uname;
  let res = FeedStatus::new(&ctx, &uname, &url, 1, 1)
    .await
    .map_err(|_e| StatusCode::BAD_REQUEST)?;

  return Ok(Json(res))
}

/// Handler for the GET `/api/star_feed?url=` endpoint.
#[debug_handler]
pub async fn unstar_feed(
  State(ctx): State<Ctx>,
  Query(param): Query<ApiQuery>,
  check: ClaimCan<BASIC_PERMIT>,
) -> Result<impl IntoResponse, StatusCode> {
  if !check.can() {
    return Err(StatusCode::UNAUTHORIZED);
  }
  let url = param.url.unwrap_or_default();
  if url.trim().len() == 0 {
    return Err(StatusCode::BAD_REQUEST);
  }

  let claim = check.claim;
  let uname = claim.unwrap_or_default().uname;
  let res = FeedStatus::del(&ctx, &uname, &url)
    .await
    .map_err(|_e| StatusCode::BAD_REQUEST)?;

  return Ok(Json(res))
}

/// Handler for the GET `/api/read_feed?url=` endpoint.
#[debug_handler]
pub async fn read_feed(
  State(ctx): State<Ctx>,
  Query(param): Query<ApiQuery>,
  check: ClaimCan<BASIC_PERMIT>,
) -> Result<impl IntoResponse, StatusCode> {
  if !check.can() {
    return Err(StatusCode::UNAUTHORIZED);
  }
  let url = param.url.unwrap_or_default();
  if url.trim().len() == 0 {
    return Err(StatusCode::BAD_REQUEST);
  }

  let claim = check.claim;
  let uname = claim.unwrap_or_default().uname;
  let res = FeedStatus::new(&ctx, &uname, &url, 1, 1)
    .await
    .map_err(|_e| StatusCode::BAD_REQUEST)?;

  return Ok(Json(res))
}
