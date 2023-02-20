//! Backend SQLite database handlers for persisting documents.

use chrono::Utc;
use sqlx::SqlitePool;

use crate::error::AppError;

/// Represents a document persisted in database storage.
#[derive(sqlx::FromRow, PartialEq, Eq, Clone, Debug)]
pub struct PersistedDocument {
  /// id, must generate before insert
  pub id: Option<String>,
  /// Text content of the document.
  pub text: String,
  /// Language of the document for editor syntax highlighting.
  pub language: Option<String>,
  /// Last modified timestamp 
  pub updated_at: Option<i64>,
  /// id of the linked article
  pub article_id: Option<u32>,
}

impl PersistedDocument {
  /// Load the text of a document from the database.
  pub async fn load(
    pool: &SqlitePool,
    document_id: &str,
  ) -> Result<PersistedDocument, AppError> {
    sqlx::query_as(
      r#"
      SELECT * FROM document WHERE id = $1;
      "#
    )
    .bind(document_id)
    .fetch_one(pool)
    .await
    .map_err(|e| e.into())
  }

  /// Store the text of a document in the database.
  pub async fn store(
    pool: &SqlitePool,
    document_id: &str,
    document: &PersistedDocument,
  ) -> Result<(), AppError> {
    let now = Utc::now().timestamp();
    let result = sqlx::query(
      r#"
      INSERT INTO
        document (id, text, language, updated_at)
      VALUES
        ($1, $2, $3, $4)
      ON CONFLICT(id) DO UPDATE SET
        text = excluded.text,
        language = excluded.language,
        updated_at = $5 
      "#
    )
    .bind(document_id)
    .bind(&document.text)
    .bind(&document.language)
    .bind(&now)
    .bind(&now)
    .execute(pool)
    .await?;

    if result.rows_affected() != 1 {
      return Err(AppError::NotFound);
    }
    Ok(())
  }

  /// Count the number of documents in the database.
  pub async fn count(pool: &SqlitePool) -> Result<usize, AppError> {
    let row: (i64,) = sqlx::query_as("SELECT count(*) FROM document;")
      .fetch_one(pool)
      .await?;
    Ok(row.0 as usize)
  }
}
