#![doc = include_str!("../README.md")]

mod config;
mod db;
mod error;
mod pad;
mod router;
mod ssr;
mod util;

use crate::{
  db::feed::refresh_feeds_job, db::sled::clear_invalid_job, router::router,
};
use config::CONFIG;
use error::AppError;
use sled::Db as Sledb;
use sqlx::{sqlite::SqliteConnectOptions, ConnectOptions, SqlitePool};
use std::{fs, path::Path, str::FromStr};
use tokio::signal;
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// App state wrapping a pool connection and sled db.
#[derive(Clone, Debug)]
pub struct AppState {
  pub pool: SqlitePool,
  pub sled: Sledb,
}

impl AppState {
  /// Construct a new state from db URIs.
  pub async fn new(uri: &str) -> Result<Self, AppError> {
    {
      // Create sqlite database file if missing, and run migrations.
      let mut conn = SqliteConnectOptions::from_str(uri)?
        .create_if_missing(true)
        .connect()
        .await?;
      sqlx::migrate!("./migrations").run(&mut conn).await?;
    }

    // init sled db
    let sled_url = &CONFIG.sled;
    info!("sled DB URI: {}", sled_url);
    let sled_config = sled::Config::default().path(sled_url).use_compression(true);
    let sled_db = sled_config.open().expect("sled db error");

    Ok(AppState {
      pool: SqlitePool::connect(uri).await?,
      sled: sled_db,
    })
  }
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
  tracing_subscriber::registry()
    .with(tracing_subscriber::EnvFilter::new("info"))
    .with(tracing_subscriber::fmt::layer())
    .init();

  let db_url = &CONFIG.db;
  info!("sqlite DB uri: {}", db_url);
  let ctx = AppState::new(db_url).await.expect("error on new app state");

  prepare_path(&CONFIG.icons_path);
  prepare_path(&CONFIG.avatars_path);
  prepare_path(&CONFIG.upload_path);

  // background job in other thread
  let ctx1 = ctx.clone();
  tokio::spawn(async move {
    loop {
      if let Err(e) = clear_invalid_job(&ctx1.sled, "captcha").await {
        error!(%e);
      }
      if let Err(e) = clear_invalid_job(&ctx1.sled, "sessions").await {
        error!(%e);
      }
      if let Err(e) = refresh_feeds_job(&ctx1).await {
        error!(%e);
      }
      sleep_seconds(3600 * 8).await;
    }
  });

  let app = router(ctx).await;
  let addr = CONFIG.addr.parse().expect("addr parse error");

  match CONFIG.tls_config().await {
    Some(tls_config) => {
      info!("listening on https://{}", addr);
      axum_server::bind_rustls(addr, tls_config)
        .serve(app.into_make_service())
        .await
        .expect("error on run serve");
    }
    None => {
      info!("listening on http://{}", addr);
      axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("error on run serve");
    }
  }

  Ok(())
}

/// prepare the folders to get ready for uploading
fn prepare_path(path_str: &str) {
  let path = Path::new(path_str);
  if !path.exists() {
    fs::create_dir_all(path).expect("create dir error");
  }
  info!("static path {path_str}");
}

async fn sleep_seconds(seconds: u64) {
  tokio::time::sleep(std::time::Duration::from_secs(seconds)).await
}

async fn shutdown_signal() {
  let ctrl_c = async {
    signal::ctrl_c()
      .await
      .expect("failed to install Ctrl+C handler");
  };

  #[cfg(unix)]
  let terminate = async {
    signal::unix::signal(signal::unix::SignalKind::terminate())
      .expect("failed to install signal handler")
      .recv()
      .await;
  };

  #[cfg(not(unix))]
  let terminate = std::future::pending::<()>();

  tokio::select! {
      _ = ctrl_c => {},
      _ = terminate => {},
  }

  println!("starting shutdown...");
}
