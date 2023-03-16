//! Server backend for the collaborative text editor.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use std::sync::Arc;
use std::time::{Duration, SystemTime};

use axum::http::StatusCode;
use axum::routing::get;
use axum::Router;
use axum::{
  extract::{ws::WebSocketUpgrade, Path, State},
  response::IntoResponse,
  Json,
};
use sqlx::SqlitePool;

use dashmap::DashMap;
use log::{error, info};
use rand::Rng;
use serde::Serialize;
use tokio::time::{self, Instant};

use document::PersistedDocument as StoreDoc;
use mdpad::Pad;

use crate::db::article::Article;
use crate::db::note::Note;
use crate::db::user::{ClaimCan, CREATE_PERMIT, BASIC_PERMIT};

pub mod document;
pub mod mdpad;
pub mod ot;

/// An entry stored in the global server map.
///
/// Each entry corresponds to a single document. This is garbage collected by a
/// background task after one day of inactivity, to avoid server memory usage
/// growing without bound.
struct Document {
  last_accessed: Instant,
  pad: Arc<Pad>,
}

impl Document {
  fn new(pad: Arc<Pad>) -> Self {
    Self {
      last_accessed: Instant::now(),
      pad,
    }
  }
}

impl Drop for Document {
  fn drop(&mut self) {
    self.pad.kill();
  }
}

/// The shared state of the server, accessible from within request handlers.
#[derive(Clone)]
struct ServerState {
  /// Concurrent map storing in-memory documents.
  documents: Arc<DashMap<String, Document>>,
  /// Connection to the database pool, if persistence is enabled.
  pool: SqlitePool,
  /// System time when the server started, in seconds since Unix epoch.
  start_time: u64,
}

/// Statistics about the server, returned from an API endpoint.
#[derive(Serialize)]
struct Stats {
  /// System time when the server started, in seconds since Unix epoch.
  start_time: u64,
  /// Number of documents currently tracked by the server.
  num_documents: usize,
  /// Number of documents persisted in the database.
  database_size: usize,
}

/// Server configuration.
#[derive(Clone, Debug)]
pub struct WsConfig {
  /// Number of hours to clean up documents after inactivity.
  pub expiry_hours: u32,
  /// Database object, for persistence if desired.
  pub pool: SqlitePool,
}

/// router
pub async fn ws_server(config: WsConfig) -> Router {
  let start_time = SystemTime::now()
    .duration_since(SystemTime::UNIX_EPOCH)
    .expect("SystemTime returned before UNIX_EPOCH")
    .as_secs();
  let state = ServerState {
    documents: Default::default(),
    pool: config.pool,
    start_time,
  };
  tokio::spawn(cleaner(state.clone(), config.expiry_hours));

  let router_ws = Router::new()
    .route("/api/socket/:id", get(socket_handler)) // WEBSOCKET
    .route("/api/text/:id", get(text_handler))
    .route("/api/stats", get(stats_handler))
    .route("/api/savetoarticle/:id", get(save_handler))
    .with_state(state);

  router_ws
}

/// Handler for the `/api/socket/:id` endpoint.
async fn socket_handler(
  State(state): State<ServerState>,
  Path(id): Path<String>,
  ws: WebSocketUpgrade,
  check: ClaimCan<BASIC_PERMIT>,
) -> Result<impl IntoResponse, StatusCode> {
  if id.starts_with("note_") && !check.can() {
    return Err(StatusCode::UNAUTHORIZED);
  }
  
  use dashmap::mapref::entry::Entry;

  let mut entry = match state.documents.entry(id.clone()) {
    Entry::Occupied(e) => e.into_ref(),
    Entry::Vacant(e) => {
      let pool = state.pool;

      if id.starts_with("note_") {
        let claim = check.claim.unwrap_or_default();
        let uname = claim.clone().uname;
        let pad = Arc::new(Note::load(&pool, &uname, &id)
          .await
          .map(Pad::from)
          .unwrap_or_default()
        );
        tokio::spawn(persister(id, uname, Arc::clone(&pad), pool.clone()));
        e.insert(Document::new(pad))
      } else {
        let pad = Arc::new(StoreDoc::load(&pool, &id)
          .await
          .map(Pad::from)
          .unwrap_or_default()
        );
        tokio::spawn(persister(id, String::new(), Arc::clone(&pad), pool.clone()));
        e.insert(Document::new(pad))
      }
    }
  };

  let value = entry.value_mut();
  value.last_accessed = Instant::now();
  let pad = Arc::clone(&value.pad);
  Ok(ws.on_upgrade(|socket| async move { pad.on_connection(socket).await }))
}

/// Handler for the `/api/text/:id` endpoint.
async fn text_handler(
  State(state): State<ServerState>,
  Path(id): Path<String>,
) -> Result<impl IntoResponse, StatusCode> {
  Ok(match state.documents.get(&id) {
    Some(value) => value.pad.text(),
    None => {
      StoreDoc::load(
        &state.pool, 
        &id,
      )
      .await
      .map(|document| document.text)
      .unwrap_or_default()
    }
  })
}

/// Handler for the `/api/stats` endpoint.
async fn stats_handler(
  State(state): State<ServerState>,
) -> Result<impl IntoResponse, StatusCode> {
  let num_documents = state.documents.len();
  let database_size = match StoreDoc::count(&state.pool).await {
    Ok(size) => size,
    Err(_e) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
  };
  Ok(Json(Stats {
    start_time: state.start_time,
    num_documents,
    database_size,
  }))
}

/// Handler for the `/api/savetoarticle/:id` endpoint.
async fn save_handler(
  State(state): State<ServerState>,
  Path(id): Path<String>,
  check: ClaimCan<CREATE_PERMIT>,
) -> Result<impl IntoResponse, StatusCode> {
  if !check.can() {
    return Err(StatusCode::UNAUTHORIZED);
  }
  let claim = check.claim.unwrap_or_default();
  let uname = claim.clone().uname;
  let article = Article::save_doc_to_article(&state.pool, &id, &uname)
    .await
    .map_err(|_e| StatusCode::BAD_REQUEST)?;
  
  return Ok(Json(article));
}

const HOUR: Duration = Duration::from_secs(3600);

/// Reclaims memory for documents.
async fn cleaner(state: ServerState, expiry_hours: u32) {
  loop {
    time::sleep(HOUR).await;
    let mut keys = Vec::new();
    for entry in &*state.documents {
      if entry.last_accessed.elapsed() > HOUR * expiry_hours {
        keys.push(entry.key().clone());
      }
    }
    info!("cleaner removing keys: {:?}", keys);
    for key in keys {
      state.documents.remove(&key);
    }
  }
}

const PERSIST_INTERVAL: Duration = Duration::from_secs(3);
const PERSIST_INTERVAL_JITTER: Duration = Duration::from_secs(1);

/// Persists changed documents after a fixed time interval.
async fn persister(id: String, uname: String, pad: Arc<Pad>, db: SqlitePool) {
  let mut last_revision = 0;
  while !pad.killed() {
    let interval = PERSIST_INTERVAL
      + rand::thread_rng().gen_range(Duration::ZERO..=PERSIST_INTERVAL_JITTER);
    time::sleep(interval).await;
    let revision = pad.revision();
    if revision > last_revision {
      info!("persisting revision {} for id = {}", revision, id);
      if id.starts_with("note_") {
        if let Err(e) = Note::store(&db, &uname, &id, &pad.snapshot().text).await {
          error!("when persisting document {}: {}", id, e);
        } else {
          last_revision = revision;
        }
      } else {
        if let Err(e) = StoreDoc::store(&db, &id, &pad.snapshot()).await {
          error!("when persisting document {}: {}", id, e);
        } else {
          last_revision = revision;
        }
      }
    }
  }
}
