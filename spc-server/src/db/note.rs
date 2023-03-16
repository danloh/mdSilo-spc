//! models for note

use chrono::Utc;
use sqlx::{FromRow, SqlitePool};
use serde::Serialize;

use crate::{error::AppError, AppState};

#[derive(FromRow, Serialize, Debug, Default)]
pub struct Note {
  pub id: String,
  pub uname: String,
  pub title: String,
  pub content: String,
  pub folder: String,
  pub created_at: i64,
  pub updated_at: i64,
}

impl Note {
  pub async fn get(
    ctx: &AppState, 
    uname: &str, 
    id: &str
  ) -> Result<Note, AppError> {
    Note::load(&ctx.pool, uname, id).await
  }

  pub async fn load(
    pool: &SqlitePool, 
    uname: &str, 
    id: &str
  ) -> Result<Note, AppError> {
    let note: Note = sqlx::query_as(
      r#"
      SELECT * FROM notes WHERE uname = $1 AND id = $2;
      "#,
    )
    .bind(uname)
    .bind(id)
    .fetch_one(pool)
    .await?;

    Ok(note)
  }

  pub async fn new(
    ctx: &AppState, 
    uname: &str, 
    id: &str,
    title: &str, 
    content: &str,
    folder: &str,
  ) -> Result<Note, AppError> {
    let now = Utc::now().timestamp();
    // insert
    let new_note: Note = sqlx::query_as(
      r#"
      INSERT OR IGNORE INTO notes 
      (id, title, uname, content, folder, created_at, updated_at)
      VALUES
      ($1, $2, $3, $4, $5, $6, $7)
      RETURNING *;
      "#,
    )
    .bind(id)
    .bind(title)
    .bind(&uname)
    .bind(content)
    .bind(folder)
    .bind(&now)
    .bind(&now)
    .fetch_one(&ctx.pool)
    .await?;

    Ok(new_note)
  }

  pub async fn rename(
    ctx: &AppState, 
    uname: &str, 
    id: &str,
    title: &str, 
  ) -> Result<Note, AppError> {
    let now = Utc::now().timestamp();
    // insert
    let new_note: Note = sqlx::query_as(
        r#"
        UPDATE notes 
        SET title = $1, updated_at = $2
        WHERE id = $3 AND uname = $4
        RETURNING *;
        "#,
      )
      .bind(title)
      .bind(&now)
      .bind(&id)
      .bind(uname)
      .fetch_one(&ctx.pool)
      .await?;

    Ok(new_note)
  }

  pub async fn update(
    ctx: &AppState, 
    uname: &str, 
    id: &str,
    content: &str, 
  ) -> Result<Note, AppError> {
    let now = Utc::now().timestamp();
    // insert
    let new_note: Note = sqlx::query_as(
        r#"
        UPDATE notes 
        SET content = $1, updated_at = $2
        WHERE id = $3 AND uname = $4
        RETURNING *;
        "#,
      )
      .bind(content)
      .bind(&now)
      .bind(&id)
      .bind(uname)
      .fetch_one(&ctx.pool)
      .await?;

    Ok(new_note)
  }

  pub async fn store(
    pool: &SqlitePool, 
    uname: &str, 
    id: &str,
    content: &str, 
  ) -> Result<Note, AppError> {
    let now = Utc::now().timestamp();
    // insert
    let new_note: Note = sqlx::query_as(
        r#"
        UPDATE notes 
        SET content = $1, updated_at = $2
        WHERE id = $3 AND uname = $4
        RETURNING *;
        "#,
      )
      .bind(content)
      .bind(&now)
      .bind(&id)
      .bind(uname)
      .fetch_one(pool)
      .await?;

    Ok(new_note)
  }

  pub async fn move_folder(
    ctx: &AppState, 
    uname: &str,
    id: &str, 
    folder: &str,
  ) -> Result<Note, AppError> {
    let new_note: Note = sqlx::query_as(
      r#"
      UPDATE notes 
      SET folder = $1 
      WHERE id = $2 AND uname = $3 
      RETURNING *;
      "#,
    )
    .bind(folder)
    .bind(&id)
    .bind(uname)
    .fetch_one(&ctx.pool)
    .await?;

    Ok(new_note)
  }

  pub async fn del(ctx: &AppState, uname: &str, id: &str) -> Result<Note, AppError> {
    let note: Note = sqlx::query_as(
      r#"
      DELETE FROM notes WHERE id = $1 AND uname = $2 RETURNING *;
      "#,
    )
    .bind(id)
    .bind(uname)
    .fetch_one(&ctx.pool)
    .await?;

    Ok(note)
  }
}

// #[derive(Debug, Clone)]
pub enum QueryNotes {
  Index(String), // uname, ... 
  // Tag(String, String, i64, i64),   // uname, tag, ..
  Folder(String, String), // uname, folder, ...
}

#[derive(FromRow, Serialize, Debug, Default)]
pub struct NoteRes {
  pub id: String,
  pub uname: String,
  pub title: String,
  pub folder: String,
  pub created_at: i64,
  pub updated_at: i64,
}

#[derive(FromRow, Serialize, Debug, Default)]
pub struct FolderRes {
  pub folder: String,
}

impl QueryNotes {
  pub async fn get(self, ctx: &AppState) -> Result<(Vec<NoteRes>, i64), AppError> {
    let note_list: Vec<NoteRes> = match self {
      QueryNotes::Index(uname) => {
        sqlx::query_as(
          r#"
          SELECT * FROM notes 
          WHERE uname = $1 
          ORDER BY updated_at DESC; 
          "#,
        )
        .bind(&uname)
        .fetch_all(&ctx.pool)
        .await
        .unwrap_or_default()
      }
      QueryNotes::Folder(uname, folder) => {
        sqlx::query_as(
          r#"
          SELECT * FROM notes 
          WHERE uname = $1 AND folder = $2 
          ORDER BY updated_at DESC; 
          "#,
        )
        .bind(&uname)
        .bind(&folder)
        .fetch_all(&ctx.pool)
        .await
        .unwrap_or_default()
      }
    };

    let note_count = note_list.len() as i64;

    Ok((note_list, note_count))
  }

  pub async fn get_folders(ctx: &AppState, uname: &str) -> Result<Vec<String>, AppError> {
    let folder_list: Vec<FolderRes> = sqlx::query_as(
      r#"
      SELECT folder FROM notes 
      WHERE uname = $1 
      GROUP BY folder; 
      "#,
    )
    .bind(&uname)
    .fetch_all(&ctx.pool)
    .await
    .unwrap_or_default();

    let folders: Vec<String> = folder_list
      .into_iter()
      .map(|n| n.folder)
      .collect();

    Ok(folders)
  }
}
