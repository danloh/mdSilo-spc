//! models for feed

use chrono::Utc;
use serde::{Serialize, Deserialize};
use sqlx::FromRow;

use crate::{error::AppError, util::feed::process_feed, AppState};

#[derive(FromRow, Debug, Clone, Default, Serialize, Deserialize)]
pub struct Channel {
  pub title: String,
  pub link: String,
  pub intro: String,
  // pub published: i64,
  pub ty: String,
}

impl Channel {
  pub async fn get_list(
    ctx: &AppState,
    perpage: i64,
    page: Option<i64>,
  ) -> Result<Vec<Channel>, AppError> {
    let channels: Vec<Channel> = match page {
      Some(p) => {
        let page_offset = std::cmp::max(0, p - 1);
        sqlx::query_as(
          r#"
          SELECT * FROM channels 
          LIMIT $1 
          OFFSET $2;
          "#,
        )
        .bind(perpage)
        .bind(perpage * page_offset)
        .fetch_all(&ctx.pool)
        .await
        .unwrap_or_default()
      }
      None => sqlx::query_as(
        r#"
          SELECT * FROM channels;
          "#,
      )
      .fetch_all(&ctx.pool)
      .await
      .unwrap_or_default(),
    };

    Ok(channels)
  }

  // pub async fn get_list_by_user(
  //   ctx: &AppState, uname: &str, is_pub: bool, perpage: i64, page: i64,
  // ) -> Result<Vec<Channel>, AppError> {
  //   let query_str = if is_pub {
  //     r#"
  //     SELECT * FROM channels
  //     WHERE link IN (
  //       SELECT channel_link FROM subscriptions
  //       WHERE uname = $1 AND is_public = true
  //     )
  //     LIMIT $2
  //     OFFSET $3;
  //     "#
  //   } else {
  //     r#"
  //     SELECT * FROM channels
  //     WHERE link IN (
  //       SELECT channel_link FROM subscriptions
  //       WHERE uname = $1
  //     )
  //     LIMIT $2
  //     OFFSET $3;
  //     "#
  //   };
  //   let page_offset = std::cmp::max(0, page - 1);
  //   let channels: Vec<Channel> = sqlx::query_as(query_str)
  //     .bind(uname)
  //     .bind(perpage)
  //     .bind(perpage * page_offset)
  //     .fetch_all(&ctx.pool)
  //     .await
  //     .unwrap_or_default();

  //   Ok(channels)
  // }

  pub async fn get_by_link(ctx: &AppState, link: &str) -> Result<Channel, AppError> {
    let channel: Channel = sqlx::query_as(
      r#"
      SELECT * FROM channels WHERE link = $1;
      "#,
    )
    .bind(link)
    .fetch_one(&ctx.pool)
    .await?;

    Ok(channel)
  }

  pub async fn new(&self, ctx: &AppState) -> Result<Channel, AppError> {
    // insert
    let new_channel: Channel = sqlx::query_as(
      r#"
      INSERT INTO
      channels (title, link, intro, published, ty)
      VALUES
      ($1, $2, $3, $4, $5)
      RETURNING *;
      "#,
    )
    .bind(&self.title)
    .bind(&self.link)
    .bind(&self.intro)
    .bind(&Utc::now().timestamp())
    .bind(&self.ty)
    .fetch_one(&ctx.pool)
    .await?;

    Ok(new_channel)
  }

  pub async fn del(ctx: &AppState, link: &str) -> Result<Channel, AppError> {
    let channel: Channel = sqlx::query_as(
      r#"
      DELETE FROM channels WHERE link = $1 RETURNING *;
      "#,
    )
    .bind(link)
    .fetch_one(&ctx.pool)
    .await?;

    // del channel's feeds
    sqlx::query(
      r#"
      DELETE FROM feeds WHERE channel_link =  $1;
      "#,
    )
    .bind(link)
    .execute(&ctx.pool)
    .await?;

    Ok(channel)
  }
}

#[derive(FromRow, Debug, Default, Serialize, Deserialize)]
pub struct Feed {
  pub id: u32,
  pub title: String,
  pub channel_link: String,
  pub feed_url: String,
  pub audio_url: String,
  pub intro: String,
  pub published: i64,
  pub content: String,
  pub author: String,
  pub img: String,
}

impl Feed {
  pub async fn get_list(
    ctx: &AppState,
    perpage: i64,
    page: i64,
  ) -> Result<Vec<Feed>, AppError> {
    let page_offset = std::cmp::max(0, page - 1);
    let feeds: Vec<Feed> = sqlx::query_as(
      r#"
      SELECT * FROM feeds 
      ORDER BY published DESC
      LIMIT $1 
      OFFSET $2;
      "#,
    )
    .bind(perpage)
    .bind(perpage * page_offset)
    .fetch_all(&ctx.pool)
    .await
    .unwrap_or_default();

    Ok(feeds)
  }

  pub async fn get_list_by_channel(
    ctx: &AppState,
    channel_link: &str,
    perpage: i64,
    page: i64,
  ) -> Result<Vec<Feed>, AppError> {
    let page_offset = std::cmp::max(0, page - 1);
    let feeds: Vec<Feed> = sqlx::query_as(
      r#"
      SELECT * FROM feeds 
      WHERE channel_link = $1
      ORDER BY published DESC
      LIMIT $2 
      OFFSET $3;
      "#,
    )
    .bind(channel_link)
    .bind(perpage)
    .bind(perpage * page_offset)
    .fetch_all(&ctx.pool)
    .await
    .unwrap_or_default();

    Ok(feeds)
  }

  pub async fn get_list_by_user(
    ctx: &AppState,
    uname: &str,
    is_pub: bool,
    perpage: i64,
    page: i64,
  ) -> Result<Vec<Feed>, AppError> {
    let query_str = if is_pub {
      r#"
      SELECT * FROM feeds 
      WHERE channel_link IN (
        SELECT channel_link FROM subscriptions
        WHERE uname = $1 AND is_public = true
      )
      ORDER BY published DESC
      LIMIT $2 
      OFFSET $3;
      "#
    } else {
      r#"
      SELECT * FROM feeds 
      WHERE channel_link IN (
        SELECT channel_link FROM subscriptions
        WHERE uname = $1 
      )
      ORDER BY published DESC
      LIMIT $2 
      OFFSET $3;
      "#
    };
    let page_offset = std::cmp::max(0, page - 1);
    let feeds: Vec<Feed> = sqlx::query_as(query_str)
      .bind(uname)
      .bind(perpage)
      .bind(perpage * page_offset)
      .fetch_all(&ctx.pool)
      .await
      .unwrap_or_default();

    Ok(feeds)
  }

  pub async fn add_feeds(ctx: &AppState, feeds: Vec<Feed>) -> Result<u64, AppError> {
    // let mut values: Vec<String> = vec![];
    // for feed in feeds {
    //   let val = format!(
    //     r#"('{0}', '{1}', '{2}', '{3}', '{4}', {5}, '{6}', '{7}', '{8}')"#,
    //     feed.title, feed.channel_link, feed.feed_url, feed.audio_url, feed.intro, feed.published, feed.content, feed.author, feed.img
    //   );
    //   values.push(val);
    // }

    // let values_str = values.join(",");
    // let query_str = format!(r#"
    //   INSERT INTO feeds
    //   (title, channel_link, feed_url, audio_url, intro, published, content, author, img)
    //   VALUES {values_str}
    //   RETURNING *;
    //   "#);

    //   println!("query: {query_str}");
    // // insert
    // let res = sqlx::query(&query_str).execute(&ctx.pool).await?;
    let mut rows = 0;
    for feed in feeds {
      let res = sqlx::query(
        r#"
        INSERT INTO feeds 
        (title, channel_link, feed_url, audio_url, intro, published, content, author, img)
        VALUES
        ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING *;
        "#,
      )
      .bind(&feed.title)
      .bind(&feed.channel_link)
      .bind(&feed.feed_url)
      .bind(&feed.audio_url)
      .bind(&feed.intro)
      .bind(&feed.published)
      .bind(&feed.content)
      .bind(&feed.author)
      .bind(&feed.img)
      .execute(&ctx.pool)
      .await?;

      rows += res.rows_affected();
    }

    Ok(rows)
  }

  // pub async fn mod_status(
  //   ctx: &AppState,
  //   ty: &str,
  //   status: u8,
  //   url: &str,
  // ) -> Result<Feed, AppError> {
  //   let query_str = if ty == "read" {
  //     r#"
  //     UPDATE feed_status
  //     SET read_status = $1
  //     WHERE feed_url = $2
  //     RETURNING *;
  //     "#
  //   } else {
  //     r#"
  //     UPDATE feed_status
  //     SET star_status = $1
  //     WHERE feed_url = $2
  //     RETURNING *;
  //     "#
  //   };

  //   let new_feed: Feed =  sqlx::query_as(query_str)
  //     .bind(&status)
  //     .bind(&url)
  //     .fetch_one(&ctx.pool)
  //     .await?;

  //   Ok(new_feed)
  // }

  // pub async fn del(ctx: &AppState, url: &str) -> Result<Feed, AppError> {
  //   let feed: Feed = sqlx::query_as(
  //     r#"
  //     DELETE FROM feeds WHERE feed_url = $1 RETURNING *;
  //     "#,
  //   )
  //   .bind(url)
  //   .fetch_one(&ctx.pool)
  //   .await?;

  //   Ok(feed)
  // }
}

#[derive(FromRow, Debug, Default)]
pub struct Subscription {
  pub id: u32,
  pub uname: String,
  pub channel_link: String,
  pub channel_title: String,
  pub is_public: bool,
}

impl Subscription {
  pub async fn get(ctx: &AppState, id: u32) -> Result<Subscription, AppError> {
    let sub: Subscription = sqlx::query_as(
      r#"
      SELECT * FROM subscriptions WHERE id = $1;
      "#,
    )
    .bind(id)
    .fetch_one(&ctx.pool)
    .await?;

    Ok(sub)
  }

  pub async fn get_list(
    ctx: &AppState,
    uname: &str,
    perpage: i64,
    page: i64,
  ) -> Result<Vec<Subscription>, AppError> {
    let page_offset = std::cmp::max(0, page - 1);
    let subs: Vec<Subscription> = sqlx::query_as(
      r#"
      SELECT * FROM subscriptions 
      WHERE uname = $1 
      ORDER BY id DESC
      LIMIT $2 
      OFFSET $3;
      "#,
    )
    .bind(uname)
    .bind(perpage)
    .bind(perpage * page_offset)
    .fetch_all(&ctx.pool)
    .await
    .unwrap_or_default();

    Ok(subs)
  }

  pub async fn new(
    ctx: &AppState,
    uname: &str,
    link: &str,
    title: &str,
    is_pub: bool,
  ) -> Result<Subscription, AppError> {
    // insert
    let new_sub: Subscription = sqlx::query_as(
      r#"
      INSERT INTO
      subscriptions (uname, channel_link, channel_title, is_public)
      VALUES
      ($1, $2, $3, $4)
      RETURNING *;
      "#,
    )
    .bind(uname)
    .bind(link)
    .bind(title)
    .bind(is_pub)
    .fetch_one(&ctx.pool)
    .await?;

    Ok(new_sub)
  }

  pub async fn mod_public(
    ctx: &AppState,
    id: u32,
    is_pub: bool,
  ) -> Result<Subscription, AppError> {
    let new_sub: Subscription = sqlx::query_as(
      r#"
      UPDATE subscriptions 
      SET is_public = $1 
      WHERE id = $2
      RETURNING *;
      "#,
    )
    .bind(&is_pub)
    .bind(&id)
    .fetch_one(&ctx.pool)
    .await?;

    Ok(new_sub)
  }

  pub async fn del(ctx: &AppState, id: u32) -> Result<Subscription, AppError> {
    let sub: Subscription = sqlx::query_as(
      r#"
      DELETE FROM subscriptions WHERE id = $1 RETURNING *;
      "#,
    )
    .bind(id)
    .fetch_one(&ctx.pool)
    .await?;

    Ok(sub)
  }
}

pub async fn refresh_feeds_job(ctx: &AppState) -> Result<(), AppError> {
  let channels = Channel::get_list(ctx, 42, None).await?;
  for channel in channels {
    let url = channel.link;
    if let Some(res) = process_feed(&url).await {
      let feeds = res.1;
      Feed::add_feeds(ctx, feeds).await?;
    }
  }

  Ok(())
}
