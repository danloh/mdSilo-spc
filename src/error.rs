//! General error: AppError,
//! into SsrError for ssr functions;
//! into ApiError for API functions.
//!
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use thiserror::Error;

use crate::ssr::error_page;

/// Genaral app error
#[derive(Error, Debug)]
pub enum AppError {
  // 5XX
  #[error("Sled db error")]
  SledError,
  #[error("sqlx error: {}", .0)]
  SqlxError(#[from] sqlx::Error),
  #[error("sqlx mig error: {}", .0)]
  MigError(#[from] sqlx::migrate::MigrateError),
  #[error("Bincode encode error: {}", .0)]
  BincodeEnError(#[from] bincode::error::EncodeError),
  #[error("Bincode decode error: {}", .0)]
  BincodeDeError(#[from] bincode::error::DecodeError),
  #[error(transparent)]
  Utf8Error(#[from] std::str::Utf8Error),
  #[error(transparent)]
  IoError(#[from] std::io::Error),
  #[error("str parse error")]
  StrParseError,
  #[error("upload error")]
  MultiPartError,
  #[error("feed related error")]
  FeedError,
  #[error(transparent)]
  ReqwestError(#[from] reqwest::Error),
  #[error("encode claim error")]
  EncodeClaimError,
  #[error("decode claim error")]
  DecodeClaimError,
  #[error("hash password error")]
  HashPasswordError,
  // 4XX
  #[error("Captcha Error")]
  CaptchaError,
  #[error("Name already exists")]
  NameExists,
  #[error("Username should not start with a number, should not contain '@' or '#'")]
  UsernameInvalid,
  #[error("wrong username or password")]
  AuthError,
  #[error("Too many attempts please try again later")]
  WriteInterval,
  #[error("Invalid input")]
  InvalidInput,
  #[error("unauthorized")]
  Unauthorized,
  #[error("NoPermission")]
  NoPermission,
  #[error("You have been banned")]
  Banned,
  #[error("The content was hidden or locked")]
  Moderated,
  #[error(transparent)]
  ImageError(#[from] image::ImageError),
  #[error("Apologies, but registrations are closed at the moment")]
  ReadOnly,
  #[error("Not found")]
  NotFound,
  #[error(transparent)]
  ValidationError(#[from] validator::ValidationErrors),
  #[error(transparent)]
  AxumFormRejection(#[from] axum::extract::rejection::FormRejection),
  
}

/// Specific error as returned error on ssr view functiom
#[derive(Error, Debug)]
pub struct SsrError {
  pub status: String,
  pub error: String,
}

impl std::fmt::Display for SsrError {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, "{}: {}", self.status, self.error)
  }
}

impl From<AppError> for SsrError {
  fn from(err: AppError) -> Self {
    let status = match err {
      AppError::NameExists
      | AppError::UsernameInvalid
      | AppError::AuthError
      | AppError::ImageError(_)
      | AppError::Moderated
      | AppError::ReadOnly
      | AppError::ValidationError(_)
      | AppError::InvalidInput
      | AppError::AxumFormRejection(_) => StatusCode::BAD_REQUEST,
      AppError::NotFound => StatusCode::NOT_FOUND,
      AppError::WriteInterval => StatusCode::TOO_MANY_REQUESTS,
      AppError::Unauthorized | AppError::NoPermission => StatusCode::UNAUTHORIZED,
      AppError::Banned => StatusCode::FORBIDDEN,
      _ => StatusCode::INTERNAL_SERVER_ERROR,
    };

    Self {
      status: status.to_string(),
      error: err.to_string(),
    }
  }
}

impl IntoResponse for SsrError {
  fn into_response(self) -> Response {
    error_page(self.status, self.error)
  }
}

// TODO: for api error
