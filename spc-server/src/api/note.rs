use axum::{extract::{State, Path}, response::IntoResponse, Json};
use axum::http::StatusCode;
use axum_macros::debug_handler;
use serde::Deserialize;

use crate::{
  AppState as Ctx,
  db::{
    note::{Note, QueryNotes}, 
    user::{ClaimCan, BASIC_PERMIT}
  }
};

#[derive(Deserialize)]
pub struct NewNote {
  id: String,
  title: String,
  content: String,
  folder: Option<String>,
}

/// Handler for the POST `/api/new_note` endpoint.
#[debug_handler]
pub async fn new_note(
  State(ctx): State<Ctx>,
  check: ClaimCan<BASIC_PERMIT>,
  Json(payload): Json<NewNote>,
) -> Result<impl IntoResponse, StatusCode> {
  if !check.can() {
    return Err(StatusCode::UNAUTHORIZED);
  }

  let claim = check.claim;
  let uname = claim.unwrap_or_default().uname;
  let folder = &payload.folder.unwrap_or(String::from("silo"));
  let new_note = Note::new(
    &ctx, &uname, &payload.id, &payload.title, &payload.content, folder
  )
  .await
  .map_err(|_e| StatusCode::BAD_REQUEST)?;
  
  return Ok(Json(new_note))
}

/// Handler for the GET `/api/move_note/:id/:folder` endpoint.
#[debug_handler]
pub async fn move_note(
  State(ctx): State<Ctx>,
  Path((id, folder)): Path<(String, String)>,
  check: ClaimCan<BASIC_PERMIT>,
) -> Result<impl IntoResponse, StatusCode> {
  if !check.can() {
    return Err(StatusCode::UNAUTHORIZED);
  }

  let claim = check.claim;
  let uname = claim.unwrap_or_default().uname;

  let new_note = Note::move_folder(
    &ctx, &uname, &id, &folder
  )
  .await
  .map_err(|_e| StatusCode::BAD_REQUEST)?;
  
  return Ok(Json(new_note))
}

/// Handler for the GET `/api/update_note` endpoint.
#[debug_handler]
pub async fn update_note(
  State(ctx): State<Ctx>,
  check: ClaimCan<BASIC_PERMIT>,
  Json(payload): Json<NewNote>,
) -> Result<impl IntoResponse, StatusCode> {
  if !check.can() {
    return Err(StatusCode::UNAUTHORIZED);
  }

  let claim = check.claim;
  let uname = claim.unwrap_or_default().uname;

  let new_note = Note::update(
    &ctx, &uname, &payload.id, &payload.content
  )
  .await
  .map_err(|_e| StatusCode::BAD_REQUEST)?;
  
  return Ok(Json(new_note))
}

/// Handler for the GET `/api/rename_note` endpoint.
#[debug_handler]
pub async fn rename_note(
  State(ctx): State<Ctx>,
  check: ClaimCan<BASIC_PERMIT>,
  Json(payload): Json<NewNote>,
) -> Result<impl IntoResponse, StatusCode> {
  if !check.can() {
    return Err(StatusCode::UNAUTHORIZED);
  }

  let claim = check.claim;
  let uname = claim.unwrap_or_default().uname;

  let new_note = Note::rename(
    &ctx, &uname, &payload.id, &payload.title
  )
  .await
  .map_err(|_e| StatusCode::BAD_REQUEST)?;
  
  return Ok(Json(new_note))
}

/// Handler for the GET `/api/del_note/:id` endpoint.
#[debug_handler]
pub async fn del_note(
  State(ctx): State<Ctx>,
  Path(id): Path<String>,
  check: ClaimCan<BASIC_PERMIT>,
) -> Result<impl IntoResponse, StatusCode> {
  if !check.can() {
    return Err(StatusCode::UNAUTHORIZED);
  }

  let claim = check.claim;
  let uname = claim.unwrap_or_default().uname;

  let note = Note::del(
    &ctx, &uname, &id
  )
  .await
  .map_err(|_e| StatusCode::BAD_REQUEST)?;
  
  return Ok(Json(note))
}

/// Handler for the GET `/api/get_note/:id` endpoint.
#[debug_handler]
pub async fn get_note(
  State(ctx): State<Ctx>,
  Path(id): Path<String>,
  check: ClaimCan<BASIC_PERMIT>,
) -> Result<impl IntoResponse, StatusCode> {
  if !check.can() {
    return Err(StatusCode::UNAUTHORIZED);
  }

  let claim = check.claim;
  let uname = claim.unwrap_or_default().uname;

  let note = Note::get(
    &ctx, &uname, &id
  )
  .await
  .map_err(|_e| StatusCode::BAD_REQUEST)?;
  
  return Ok(Json(note))
}

/// Handler for the GET `/api/get_notes` endpoint.
#[debug_handler]
pub async fn get_notes(
  State(ctx): State<Ctx>,
  check: ClaimCan<BASIC_PERMIT>,
) -> Result<impl IntoResponse, StatusCode> {
  if !check.can() {
    return Err(StatusCode::UNAUTHORIZED);
  }

  let claim = check.claim;
  let uname = claim.unwrap_or_default().uname;

  let notes = QueryNotes::Index(uname).get(&ctx)
  .await
  .map_err(|_e| StatusCode::BAD_REQUEST)?;
  
  return Ok(Json(notes))
}

/// Handler for the GET `/api/get_folder_notes/:folder` endpoint.
#[debug_handler]
pub async fn get_notes_by_folder(
  State(ctx): State<Ctx>,
  Path(folder): Path<String>,
  check: ClaimCan<BASIC_PERMIT>,
) -> Result<impl IntoResponse, StatusCode> {
  if !check.can() {
    return Err(StatusCode::UNAUTHORIZED);
  }

  let claim = check.claim;
  let uname = claim.unwrap_or_default().uname;

  let notes = QueryNotes::Folder(uname, folder).get(&ctx)
  .await
  .map_err(|_e| StatusCode::BAD_REQUEST)?;
  
  return Ok(Json(notes))
}

/// Handler for the GET `/api/get_folders` endpoint.
#[debug_handler]
pub async fn get_folders(
  State(ctx): State<Ctx>,
  check: ClaimCan<BASIC_PERMIT>,
) -> Result<impl IntoResponse, StatusCode> {
  if !check.can() {
    return Err(StatusCode::UNAUTHORIZED);
  }

  let claim = check.claim;
  let uname = claim.unwrap_or_default().uname;

  let folders = QueryNotes::get_folders(&ctx, &uname)
  .await
  .map_err(|_e| StatusCode::BAD_REQUEST)?;
  
  return Ok(Json(folders))
}
