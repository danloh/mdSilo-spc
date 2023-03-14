//! models for note

use chrono::Utc;
use sqlx::FromRow;
use serde::Serialize;

use crate::{error::AppError, AppState};

#[derive(FromRow, Serialize, Debug, Default)]
pub struct Note {
  pub id: u32,
  pub uname: String,
  pub title: String,
  pub content: String,
  pub folder: String,
  pub created_at: i64,
  pub updated_at: i64,
}

impl Note {
  pub async fn get(ctx: &AppState, uname: &str, id: u32) -> Result<Note, AppError> {
    let note: Note = sqlx::query_as(
      r#"
      SELECT * FROM notes WHERE id = $1 AND uname = $2;
      "#,
    )
    .bind(id)
    .bind(uname)
    .fetch_one(&ctx.pool)
    .await?;

    Ok(note)
  }

  // pub async fn get_by_id_or_title(
  //   ctx: &AppState,
  //   name: &str,
  // ) -> Result<Note, AppError> {
  //   let id: u32 = name.parse().unwrap_or(0);
  //   let note: Note = sqlx::query_as(
  //     r#"
  //     SELECT * FROM notes WHERE id = $1 OR title = $2;
  //     "#,
  //   )
  //   .bind(id)
  //   .bind(name)
  //   .fetch_one(&ctx.pool)
  //   .await?;

  //   Ok(note)
  // }

  pub async fn new(
    ctx: &AppState, 
    uname: &str, 
    id: u32,
    title: &str, 
    content: &str,
  ) -> Result<Note, AppError> {
    let now = Utc::now().timestamp();
    // insert
    let new_note: Note = if id == 0 {
      sqlx::query_as(
        r#"
        INSERT OR IGNORE INTO notes 
        (title, uname, content, created_at, updated_at)
        VALUES
        ($1, $2, $3, $4, $5)
        RETURNING *;
        "#,
      )
      .bind(title)
      .bind(&uname)
      .bind(content)
      .bind(&now)
      .bind(&now)
      .fetch_one(&ctx.pool)
      .await?
    } else {
      sqlx::query_as(
        r#"
        UPDATE notes 
        SET title = $1, content = $2, updated_at = $3
        WHERE id = $4
        RETURNING *;
        "#,
      )
      .bind(title)
      .bind(content)
      .bind(&now)
      .bind(&id)
      .fetch_one(&ctx.pool)
      .await?
    };

    Ok(new_note)
  }

  pub async fn move_folder(
    ctx: &AppState, 
    uname: &str,
    id: u32, 
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

  pub async fn del(ctx: &AppState, uname: &str, id: u32) -> Result<Note, AppError> {
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
  pub id: u32,
  pub uname: String,
  pub title: String,
  pub folder: String,
  pub created_at: i64,
  pub updated_at: i64,
}

impl QueryNotes {
  pub async fn get(self, ctx: &AppState) -> Result<(Vec<NoteRes>, i64), AppError> {
    let note_list: Vec<NoteRes> = match self {
      QueryNotes::Index(uname) => {
        sqlx::query_as(
          r#"
          SELECT * FROM notes 
          WHERE uname = $1 
          ORDER BY id DESC; 
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
          ORDER BY id DESC; 
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
}
