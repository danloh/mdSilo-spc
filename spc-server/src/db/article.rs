//! models for article

use chrono::Utc;
use sqlx::{FromRow, SqlitePool};
use serde::Serialize;

use super::{feed::Feed, tag::Tag, sled::gen_expirable_id};
use crate::{error::AppError, util::md::md2html, AppState, pad::document::PersistedDocument};

#[derive(FromRow, Serialize, Debug, Default)]
pub struct Article {
  pub id: u32,
  pub uname: String,
  pub title: String,
  pub cover: String,
  pub content: String,
  pub created_at: i64,
  pub updated_at: i64,
  pub is_locked: bool,
  pub is_hidden: bool,
}

impl Article {
  pub async fn get(ctx: &AppState, id: u32) -> Result<Article, AppError> {
    let article: Article = sqlx::query_as(
      r#"
      SELECT * FROM articles WHERE id = $1;
      "#,
    )
    .bind(id)
    .fetch_one(&ctx.pool)
    .await?;

    Ok(article)
  }

  pub async fn get_by_id_or_title(
    ctx: &AppState,
    name: &str,
  ) -> Result<Article, AppError> {
    let id: u32 = name.parse().unwrap_or(0);
    let article: Article = sqlx::query_as(
      r#"
      SELECT * FROM articles WHERE id = $1 OR title = $2;
      "#,
    )
    .bind(id)
    .bind(name)
    .fetch_one(&ctx.pool)
    .await?;

    Ok(article)
  }

  pub async fn new(&self, ctx: &AppState) -> Result<Article, AppError> {
    let id = self.id;
    let now = Utc::now().timestamp();
    // insert
    let new_article: Article = if id == 0 {
      sqlx::query_as(
        r#"
        INSERT INTO
        articles (title, uname, cover, content, created_at, updated_at)
        VALUES
        ($1, $2, $3, $4, $5, $6)
        RETURNING *;
        "#,
      )
      .bind(&self.title)
      .bind(&self.uname)
      .bind(&self.cover)
      .bind(&self.content)
      .bind(&now)
      .bind(&now)
      .fetch_one(&ctx.pool)
      .await?
    } else {
      sqlx::query_as(
        r#"
        UPDATE articles 
        SET title = $1, cover = $2, content = $3, updated_at = $4
        WHERE id = $5
        RETURNING *;
        "#,
      )
      .bind(&self.title)
      .bind(&self.cover)
      .bind(&self.content)
      .bind(&now)
      .bind(&self.id)
      .fetch_one(&ctx.pool)
      .await?
    };

    Ok(new_article)
  }

  pub async fn del(ctx: &AppState, id: u32) -> Result<Article, AppError> {
    let article: Article = sqlx::query_as(
      r#"
      DELETE FROM articles WHERE id = $1 RETURNING *;
      "#,
    )
    .bind(id)
    .fetch_one(&ctx.pool)
    .await?;

    // del TagEntry
    sqlx::query(
      r#"
      DELETE FROM tag_entry WHERE on_ty = 'article' AND on_id = $1;
      "#,
    )
    .bind(id)
    .execute(&ctx.pool)
    .await?;

    Ok(article)
  }

  /// gen a link for collaborative edit
  pub async fn gen_pad_link(
    ctx: &AppState, id: u32, uname: &str
  ) -> Result<String, AppError> {
    let article: Article = Article::get(ctx, id).await?;
    // check Author uname matched
    if article.uname != uname {
      return Err(AppError::NoPermission);
    }
    // num_char "\r" issue, leading OTError
    // operational_transform::OperationSeq
    // pub fn apply(&self, s: &str) -> Result<String, OTError>
    // Applies an operation to a string, returning a new string.
    // if num_chars(s.as_bytes()) != self.base_len
    // Returns an error if the operation cannot be applied due to length conflicts.
    let esc_content = article.content.replace("\r", "\n");
    // create a document, UNIQUE(article_id) if not NULL
    // check if existing already
    let doc_res: Result<PersistedDocument, AppError> = sqlx::query_as(
      r#"
      SELECT * FROM document WHERE article_id = $1;
      "#
    )
    .bind(&article.id)
    .fetch_one(&ctx.pool)
    .await
    .map_err(|_| AppError::NotFound);
    
    let generated_id = gen_expirable_id(60 * 60 * 24, None);
    let doc_id = match doc_res {
      Ok(doc) if doc.id.is_some() => doc.id.unwrap_or(generated_id),
      _ => generated_id
    };
    
    let result = sqlx::query(
      r#"
      INSERT INTO
        document (id, text, article_id)
      VALUES
        ($1, $2, $3)
      ON CONFLICT(id) DO UPDATE SET
        text = excluded.text
      "#,
    )
    .bind(&doc_id)
    .bind(&esc_content)
    .bind(&article.id)
    .execute(&ctx.pool)
    .await?;

    if result.rows_affected() != 1 {
      return Err(AppError::NotFound);
    }

    Ok(format!("/pad#{doc_id}"))
  }

  /// save the collaborative editing result to article. 
  /// the param `uname` is used to check author matched.   
  pub async fn save_doc_to_article(
    pool: &SqlitePool, doc_id: &str, uname: &str,
  ) -> Result<Article, AppError> {
    let doc = PersistedDocument::load(pool, doc_id).await?;
    
    // create a document
    if let Some(article_id) = doc.article_id {
      let article: Article = sqlx::query_as(
        r#"
        UPDATE articles 
        SET content = $1, updated_at = $2
        WHERE id = $3 AND uname = $4 
        RETURNING *;
        "#,
      )
      .bind(&doc.text)
      .bind(&Utc::now().timestamp())
      .bind(&article_id) // to get article
      .bind(&uname)      // to check author matched
      .fetch_one(pool)
      .await?;
      
      // del all the documents with this article_id
      sqlx::query(
        r#"
        DELETE FROM document WHERE article_id = $1;
        "#
      )
      .bind(article_id)
      .execute(pool)
      .await
      .unwrap_or_default();

      return Ok(article);
    } else {
      return Err(AppError::NotFound);
    }
  }
}

#[derive(FromRow, Debug, Default)]
pub struct Piece {
  pub id: u32,
  pub uname: String,
  pub content: String,
  pub created_at: i64,
  pub is_hidden: bool,
}

impl Piece {
  pub async fn get(ctx: &AppState, id: u32) -> Result<Piece, AppError> {
    let piece: Piece = sqlx::query_as(
      r#"
      SELECT * FROM pieces WHERE id = $1
      "#,
    )
    .bind(id)
    .fetch_one(&ctx.pool)
    .await?;

    Ok(piece)
  }

  pub async fn new(&self, ctx: &AppState) -> Result<Piece, AppError> {
    let id = self.id;
    let now = Utc::now().timestamp();
    // insert
    let new_piece: Piece = if id == 0 {
      sqlx::query_as(
        r#"
        INSERT INTO
        pieces (uname, content, created_at)
        VALUES
        ($1, $2, $3)
        RETURNING *;
        "#,
      )
      .bind(&self.uname)
      .bind(&self.content)
      .bind(&now)
      .fetch_one(&ctx.pool)
      .await?
    } else {
      sqlx::query_as(
        r#"
        UPDATE pieces 
        SET content = $1, created_at = $2
        WHERE id = $3
        RETURNING *;
        "#,
      )
      .bind(&self.content)
      .bind(&now)
      .bind(&self.id)
      .fetch_one(&ctx.pool)
      .await?
    };

    Ok(new_piece)
  }

  pub async fn del(ctx: &AppState, id: u32) -> Result<Piece, AppError> {
    let piece: Piece = sqlx::query_as(
      r#"
      DELETE FROM pieces WHERE id = $1 RETURNING *;
      "#,
    )
    .bind(id)
    .fetch_one(&ctx.pool)
    .await?;

    // del TagEntry
    sqlx::query(
      r#"
      DELETE FROM tag_entry WHERE on_ty = 'piece' AND on_id = $1;
      "#,
    )
    .bind(id)
    .execute(&ctx.pool)
    .await?;

    Ok(piece)
  }
}

#[derive(Debug, Default)]
pub struct Entry {
  pub ty: String,
  pub id: u32,
  pub title: String,
  pub cover: String,
  pub content: String,
  pub uname: String,
  pub created_at: i64,
  pub link: String,
}

impl From<Article> for Entry {
  fn from(a: Article) -> Self {
    Entry {
      ty: String::from("article"),
      id: a.id,
      title: a.title,
      cover: a.cover,
      content: a.content,
      uname: a.uname,
      created_at: a.created_at,
      link: format!("/article/{}/view", a.id),
    }
  }
}

impl From<Piece> for Entry {
  fn from(p: Piece) -> Self {
    Entry {
      ty: String::from("piece"),
      id: p.id,
      title: p.content.clone(),
      cover: String::from(""),
      content: md2html(&p.content),
      uname: p.uname,
      created_at: p.created_at,
      link: format!("/piece/{}", p.id),
    }
  }
}

impl From<Feed> for Entry {
  fn from(f: Feed) -> Self {
    Entry {
      ty: String::from("feed"),
      id: f.id,
      title: f.title,
      cover: String::from(""),
      content: f.content,
      uname: f.channel_link, // channel as uname
      created_at: f.published,
      link: f.feed_url,
    }
  }
}

impl From<Tag> for Entry {
  fn from(t: Tag) -> Self {
    Entry {
      ty: String::from("tag"),
      id: t.id,
      title: t.tname.clone(),
      cover: String::from(""),
      content: t.content,
      uname: String::from(""),
      created_at: 0,
      link: format!("/tag/{}", t.tname),
    }
  }
}

// #[derive(Debug, Clone)]
pub enum QueryArticles {
  Index(String, i64, i64), // ord, perpage, page
  Tag(String, i64, i64),   // tag, ..
  User(String, u8, i64, i64), // uname, action:1-by|2-like
  // Item(u32, String, i64, i64), // item id, ord, ..
  // Kw(String, i64, i64), // kw, ..
}

impl QueryArticles {
  pub async fn get(self, ctx: &AppState) -> Result<(Vec<Article>, i64), AppError> {
    let mut article_list: Vec<Article> = Vec::new();
    // let mut article_count: i64 = 0;
    match self {
      QueryArticles::Index(ord, perpage, page) => {
        let page_offset = std::cmp::max(0, page - 1);
        article_list = if ord.to_lowercase().trim() == "desc" {
          sqlx::query_as(
            r#"
            SELECT * FROM articles 
            ORDER BY id DESC
            LIMIT $1 
            OFFSET $2; 
            "#,
          )
          .bind(perpage)
          .bind(perpage * page_offset)
          .fetch_all(&ctx.pool)
          .await
          .unwrap_or_default()
        } else {
          sqlx::query_as(
            r#"
            SELECT * FROM articles 
            ORDER BY id ASC
            LIMIT $1 
            OFFSET $2; 
            "#,
          )
          .bind(perpage)
          .bind(perpage * page_offset)
          .fetch_all(&ctx.pool)
          .await
          .unwrap_or_default()
        };
      }
      QueryArticles::Tag(tname, perpage, page) => {
        let page_offset = std::cmp::max(0, page - 1);
        article_list = sqlx::query_as(
          r#"
          SELECT * FROM articles
          WHERE id IN (
            SELECT on_id FROM tag_entry
            WHERE on_ty = 'article' AND tag_id = (
              SELECT id FROM tags WHERE tname = $1
            )
            ORDER BY on_id DESC
            LIMIT $2 
            OFFSET $3 
          );
          "#,
        )
        .bind(&tname)
        .bind(perpage)
        .bind(perpage * page_offset)
        .fetch_all(&ctx.pool)
        .await
        .unwrap_or_default();
      }
      QueryArticles::User(uname, act, perpage, page) => {
        let page_offset = std::cmp::max(0, page - 1);
        if act == 1 {
          article_list = sqlx::query_as(
            r#"
            SELECT * FROM articles 
            WHERE uname = $1
            ORDER BY id DESC
            LIMIT $2 
            OFFSET $3; 
            "#,
          )
          .bind(&uname)
          .bind(perpage)
          .bind(perpage * page_offset)
          .fetch_all(&ctx.pool)
          .await
          .unwrap_or_default()
        }

        // article_count = article_list.len() as i64;
      } // _ => {} // TODO
    }

    let article_count = article_list.len() as i64;

    Ok((article_list, article_count))
  }
}

// #[derive(Debug, Clone)]
pub enum QueryPieces {
  Index(String, i64, i64), // ord, perpage, page
  Tag(String, i64, i64),   // tag, ..
  User(String, u8, i64, i64), // uname, action:1-by|2-like
  // Kw(String, i64, i64), // kw, ..
}

impl QueryPieces {
  pub async fn get(self, ctx: &AppState) -> Result<(Vec<Piece>, i64), AppError> {
    let mut piece_list: Vec<Piece> = Vec::new();
    // let mut piece_count: i64 = 0;
    match self {
      QueryPieces::Index(ord, perpage, page) => {
        let page_offset = std::cmp::max(0, page - 1);
        piece_list = if ord.to_lowercase().trim() == "desc" {
          sqlx::query_as(
            r#"
            SELECT * FROM pieces 
            ORDER BY id DESC
            LIMIT $1 
            OFFSET $2; 
            "#,
          )
          .bind(perpage)
          .bind(perpage * page_offset)
          .fetch_all(&ctx.pool)
          .await
          .unwrap_or_default()
        } else {
          sqlx::query_as(
            r#"
            SELECT * FROM pieces 
            ORDER BY id ASC
            LIMIT $1 
            OFFSET $2; 
            "#,
          )
          .bind(perpage)
          .bind(perpage * page_offset)
          .fetch_all(&ctx.pool)
          .await
          .unwrap_or_default()
        };
      }
      QueryPieces::Tag(tname, perpage, page) => {
        let page_offset = std::cmp::max(0, page - 1);
        piece_list = sqlx::query_as(
          r#"
          SELECT * FROM pieces
          WHERE id IN (
            SELECT on_id FROM tag_entry
            WHERE on_ty = 'piece' AND tag_id = (
              SELECT id FROM tags WHERE tname = $1
            )
            ORDER BY on_id DESC
            LIMIT $2 
            OFFSET $3 
          );
          "#,
        )
        .bind(&tname)
        .bind(perpage)
        .bind(perpage * page_offset)
        .fetch_all(&ctx.pool)
        .await
        .unwrap_or_default();
      }
      QueryPieces::User(uname, act, perpage, page) => {
        let page_offset = std::cmp::max(0, page - 1);
        if act == 1 {
          piece_list = sqlx::query_as(
            r#"
            SELECT * FROM pieces 
            WHERE uname = $1
            ORDER BY id DESC
            LIMIT $2 
            OFFSET $3; 
            "#,
          )
          .bind(&uname)
          .bind(perpage)
          .bind(perpage * page_offset)
          .fetch_all(&ctx.pool)
          .await
          .unwrap_or_default()
        }

        // piece_count = piece_list.len() as i64;
      } // _ => {} // TODO
    }

    let piece_count = piece_list.len() as i64;

    Ok((piece_list, piece_count))
  }
}
