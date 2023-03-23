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

/// Handler for the GET `/api/del_subscription?url=` endpoint.
#[debug_handler]
pub async fn del_subscription(
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

  let res = Subscription::del(&ctx, &uname, &url)
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
    .map_err(|_e| StatusCode::NOT_FOUND)?;

  return Ok(Json(res))
}

/// Handler for the GET `/api/get_audio_feeds` endpoint.
#[debug_handler]
pub async fn get_audio_feeds(
  State(ctx): State<Ctx>,
  check: ClaimCan<BASIC_PERMIT>,
) -> Result<impl IntoResponse, StatusCode> {
  if !check.can() {
    return Err(StatusCode::UNAUTHORIZED);
  }

  let claim = check.claim;
  let uname = claim.unwrap_or_default().uname;
  let res = Subscription::get_audio_feeds(&ctx, &uname)
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
  let res = FeedStatus::new(&ctx, &uname, &url, "star", 1)
    .await
    .map_err(|e| warn!("star feed: {}", e))
    .unwrap_or_default();

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
  let res = FeedStatus::new(&ctx, &uname, &url, "star", 0)
    .await
    .map_err(|e| warn!("unstar feed: {}", e))
    .unwrap_or_default();

  return Ok(Json(res))
}

/// Handler for the GET `/api/check_star?url=` endpoint.
#[debug_handler]
pub async fn check_star(
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
  let res = FeedStatus::check_star(&ctx, &uname, &url)
    .await
    .map_err(|e| warn!("check star: {}", e))
    .unwrap_or_default();

  return Ok(Json(res))
}

/// Handler for the GET `/api/check_star?url=` endpoint.
#[debug_handler]
pub async fn check_read(
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
  let res = FeedStatus::check_read(&ctx, &uname, &url)
    .await
    .map_err(|e| warn!("check read: {}", e))
    .unwrap_or_default();

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
  let res = FeedStatus::new(&ctx, &uname, &url, "read", 1)
    .await
    .map_err(|e| warn!("read status: {}", e))
    .unwrap_or_default();

  return Ok(Json(res))
}

/// Handler for the GET `/proxy/gethtml?url=` endpoint. 
/// as proxy to extract article on client side
#[debug_handler]
pub async fn get_html_proxy(
  Query(param): Query<ApiQuery>,
) -> Result<impl IntoResponse, StatusCode> {
  let url = param.url.unwrap_or_default();
  if url.trim().len() == 0 {
    return Err(StatusCode::BAD_REQUEST);
  }
  // println!("via proxy: {}", url);
  let client = reqwest::Client::builder().build();
  let response = match client {
    Ok(cl) => cl.get(url).send().await,
    Err(_e) => return Err(StatusCode::BAD_REQUEST),
  };

  let resp = match response {
    Ok(response) => match response.status() {
      reqwest::StatusCode::OK => {
        let content = match response.text().await {
          Ok(ctn) => ctn,
          Err(_e) => return Err(StatusCode::BAD_REQUEST),
        };
        content
      }
      status => return Err(status),
    },
    Err(_e) => return Err(StatusCode::BAD_REQUEST),
  };

  return Ok(resp)
}
