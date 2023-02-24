//! models for tag

use sqlx::FromRow;
use std::collections::BTreeSet;

use crate::{error::AppError, AppState};

// use super::article::{Article, Entry, Piece};

#[derive(FromRow, Debug, Default)]
pub struct Tag {
  pub id: u32,
  pub tname: String,
  pub content: String,
}

impl Tag {
  pub async fn get(ctx: &AppState, name: &str) -> Result<Tag, AppError> {
    let id: u32 = name.parse().unwrap_or(0);
    let tag: Tag = sqlx::query_as(
      r#"
      SELECT * FROM tags WHERE id = $1 OR tname = $2;
      "#,
    )
    .bind(id)
    .bind(name)
    .fetch_one(&ctx.pool)
    .await?;

    Ok(tag)
  }

  pub async fn get_list(
    ctx: &AppState,
    ord: &str,
    perpage: i64,
    page: i64,
  ) -> Result<Vec<Tag>, AppError> {
    let page_offset = std::cmp::max(0, page - 1);
    let tags: Vec<Tag> = match ord.to_lowercase().trim() {
      "desc" => sqlx::query_as(
        r#"
          SELECT * FROM tags 
          ORDER BY id DESC
          LIMIT $1 
          OFFSET $2; 
          "#,
      )
      .bind(perpage)
      .bind(perpage * page_offset)
      .fetch_all(&ctx.pool)
      .await
      .unwrap_or_default(),
      "asc" => sqlx::query_as(
        r#"
          SELECT * FROM tags 
          ORDER BY id ASC
          LIMIT $1 
          OFFSET $2; 
          "#,
      )
      .bind(perpage)
      .bind(perpage * page_offset)
      .fetch_all(&ctx.pool)
      .await
      .unwrap_or_default(),
      _ => vec![],
    };

    Ok(tags)
  }

  pub async fn new(
    ctx: &AppState,
    id: u32,
    tname: &str,
    content: &str,
  ) -> Result<Tag, AppError> {
    // insert
    let _new_tag: Tag = if id == 0 {
      sqlx::query_as(
        r#"
        INSERT INTO tags (tname, content)
        VALUES ($1, $2)
        ON CONFLICT (tname) DO NOTHING 
        RETURNING *;
        "#,
      )
      .bind(tname)
      .bind(content)
      .fetch_one(&ctx.pool)
      .await?
    } else {
      sqlx::query_as(
        r#"
        UPDATE tags 
        SET tname = $1, content = $2
        WHERE id = $3
        RETURNING *;
        "#,
      )
      .bind(tname)
      .bind(content)
      .bind(id)
      .fetch_one(&ctx.pool)
      .await?
    };

    let tag: Tag = sqlx::query_as(
      r#"
      SELECT * FROM tags WHERE tname = $1;
      "#,
    )
    .bind(tname)
    .fetch_one(&ctx.pool)
    .await?;

    Ok(tag)
  }

  pub async fn del(ctx: &AppState, id: u32) -> Result<Tag, AppError> {
    let tag: Tag = sqlx::query_as(
      r#"
      DELETE FROM tags WHERE id = $1 RETURNING *;
      "#,
    )
    .bind(id)
    .fetch_one(&ctx.pool)
    .await?;

    // del TagEntry
    sqlx::query(
      r#"
      DELETE FROM tag_entry WHERE tag_id = $1;
      "#,
    )
    .bind(id)
    .execute(&ctx.pool)
    .await?;

    Ok(tag)
  }
}

#[derive(FromRow, Debug, Default)]
pub struct TagEntry {
  pub tag_id: u32,
  pub on_ty: String,
  pub on_id: u32,
}

impl TagEntry {
  pub async fn get_tags(
    ctx: &AppState,
    ty: &str,
    id: u32,
  ) -> Result<Vec<Tag>, AppError> {
    let tags: Vec<Tag> = sqlx::query_as(
      r#"
      SELECT * FROM tags 
      WHERE id IN (
        SELECT tag_id FROM tag_entry
        WHERE on_ty = $1 AND on_id = $2
      )
      "#,
    )
    .bind(ty)
    .bind(id)
    .fetch_all(&ctx.pool)
    .await?;

    Ok(tags)
  }

  // pub async fn get_entries(
  //   ctx: &AppState,
  //   tag_id: u32,
  //   ty: &str,
  // ) -> Result<Vec<Entry>, AppError> {
  //   let mut entries: Vec<Entry> = Vec::new();
  //   match ty.to_lowercase().trim() {
  //     "article" => {
  //       let articles: Vec<Article> = sqlx::query_as(
  //         r#"
  //         SELECT * FROM articles WHERE id IN (
  //           SELECT on_id FROM tag_entry
  //           WHERE on_ty = 'article' AND tag_id = $1
  //         );
  //         "#,
  //       )
  //       .bind(tag_id)
  //       .fetch_all(&ctx.pool)
  //       .await?;

  //       entries = articles.into_iter().map(|a| a.into()).collect();
  //     }
  //     "piece" => {
  //       let pieces: Vec<Piece> = sqlx::query_as(
  //         r#"
  //         SELECT * FROM pieces WHERE id IN (
  //           SELECT on_id FROM tag_entry
  //           WHERE on_ty = 'piece' AND tag_id = $1
  //         );
  //         "#,
  //       )
  //       .bind(tag_id)
  //       .fetch_all(&ctx.pool)
  //       .await?;

  //       entries = pieces.into_iter().map(|a| a.into()).collect();
  //     }
  //     _ => {}
  //   }

  //   Ok(entries)
  // }

  pub async fn new(
    ctx: &AppState,
    tag_id: u32,
    on_ty: &str,
    on_id: u32,
  ) -> Result<TagEntry, AppError> {
    // insert
    let new_tag: TagEntry = sqlx::query_as(
      r#"
        INSERT INTO
        tag_entry (tag_id, on_ty, on_id)
        VALUES
        ($1, $2, $3)
        RETURNING *;
        "#,
    )
    .bind(tag_id)
    .bind(on_ty)
    .bind(on_id)
    .fetch_one(&ctx.pool)
    .await?;

    Ok(new_tag)
  }

  pub async fn del(
    ctx: &AppState,
    tag_id: u32,
    on_ty: &str,
    on_id: u32,
  ) -> Result<(), AppError> {
    sqlx::query(
      r#"
      DELETE FROM tag_entry 
      WHERE tag_id = $1 AND on_ty = $2 AND on_id = $1 
      RETURNING *;
      "#,
    )
    .bind(tag_id)
    .bind(on_ty)
    .bind(on_id)
    .execute(&ctx.pool)
    .await?;

    Ok(())
  }

  pub async fn tag(
    ctx: &AppState,
    tnames: BTreeSet<String>,
    on_ty: &str,
    on_id: u32,
  ) -> Result<(), AppError> {
    let old_tags: Vec<Tag> = TagEntry::get_tags(&ctx, on_ty, on_id).await?;
    let old_tnames: Vec<String> = old_tags.into_iter().map(|t| t.tname).collect();
    let old_set: BTreeSet<String> = old_tnames
      .into_iter()
      .map(|s| s.trim().to_string())
      .filter(|s| !s.is_empty())
      .collect();

    let to_del: Vec<String> = old_set.difference(&tnames).cloned().collect();
    let to_add: Vec<String> = tnames.difference(&old_set).cloned().collect();

    // add
    for t in to_add {
      let new_tag = Tag::new(&ctx, 0, &t, "").await?;
      TagEntry::new(&ctx, new_tag.id, on_ty, on_id).await?;
    }

    // del
    for t in to_del {
      let del_tag = Tag::get(&ctx, &t).await?;
      TagEntry::del(&ctx, del_tag.id, on_ty, on_id).await?;
    }

    Ok(())
  }
}
