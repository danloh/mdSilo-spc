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
  pub is_hidden: bool,
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
        r#"SELECT * FROM channels;"#,
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
      INSERT OR IGNORE INTO channels 
      (title, link, intro, published, ty)
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

  pub async fn del(ctx: &AppState, id: u32) -> Result<Channel, AppError> {
    let channel: Channel = sqlx::query_as(
      r#"
      DELETE FROM channels WHERE id = $1 RETURNING *;
      "#,
    )
    .bind(id)
    .fetch_one(&ctx.pool)
    .await?;

    // del channel's feeds
    sqlx::query(
      r#"
      DELETE FROM feeds WHERE channel_link =  $1;
      "#,
    )
    .bind(&channel.link)
    .execute(&ctx.pool)
    .await?;

    Ok(channel)
  }

  pub async fn mod_channel(
    ctx: &AppState,
    link: &str,
    is_hidden: bool,
  ) -> Result<Channel, AppError> {
    let channel: Channel = sqlx::query_as(
      r#"
      UPDATE channels 
      SET is_hidden = $1 
      WHERE link = $2
      RETURNING *;
      "#,
    )
    .bind(&is_hidden)
    .bind(link)
    .fetch_one(&ctx.pool)
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
    pub_only: bool,
  ) -> Result<Vec<Feed>, AppError> {
    let page_offset = std::cmp::max(0, page - 1);
    let feeds: Vec<Feed> = if pub_only {
      sqlx::query_as(
        r#"
        SELECT * FROM feeds 
        WHERE channel_link IN (
          SELECT link FROM channels WHERE is_hidden = false
        )
        ORDER BY published DESC
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
      .unwrap_or_default()
    };

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
        INSERT OR IGNORE INTO feeds 
        (title, channel_link, feed_url, audio_url, published, intro, content, author, img)
        VALUES
        ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING *;
        "#,
      )
      .bind(&feed.title)
      .bind(&feed.channel_link)
      .bind(&feed.feed_url)
      .bind(&feed.audio_url)
      .bind(&feed.published)
      .bind("") // &feed.intro
      .bind("") // &feed.content // extract on client side, not save 
      .bind(&feed.author)
      .bind(&feed.img)
      .execute(&ctx.pool)
      .await?;

      rows += res.rows_affected();
    }

    Ok(rows)
  }
}

#[derive(FromRow, Debug, Default, Serialize)]
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

  pub async fn get_by_link(
    ctx: &AppState, 
    link: &str, 
    uname: &str
  ) -> Result<Subscription, AppError> {
    let sub: Subscription = sqlx::query_as(
      r#"
      SELECT * FROM subscriptions WHERE channel_link = $1 AND uname = $2;
      "#,
    )
    .bind(link)
    .bind(uname)
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

  pub async fn get_channel_list(
    ctx: &AppState,
    uname: &str,
  ) -> Result<Vec<Channel>, AppError> {
    let subs: Vec<Channel> = sqlx::query_as(
      r#"
      SELECT * FROM channels 
      WHERE link IN (
        SELECT channel_link FROM subscriptions  
        WHERE uname = $1 
      )
      "#,
    )
    .bind(uname)
    .fetch_all(&ctx.pool)
    .await
    .unwrap_or_default();

    Ok(subs)
  }

  pub async fn get_audio_feeds(
    ctx: &AppState,
    uname: &str,
  ) -> Result<Vec<Feed>, AppError> {
    let audiolist: Vec<Feed> = sqlx::query_as(
      r#"
      SELECT * FROM feeds 
      WHERE audio_url != '' AND feed_url IN (
        SELECT feed_url FROM subscriptions  
        WHERE uname = $1 
      );
      "#,
    )
    .bind(uname)
    .fetch_all(&ctx.pool)
    .await
    .unwrap_or_default();

    Ok(audiolist)
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
      INSERT OR IGNORE INTO subscriptions 
      (uname, channel_link, channel_title, is_public)
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

  pub async fn del(
    ctx: &AppState, 
    uname: &str, 
    channel_link: &str,
  ) -> Result<Subscription, AppError> {
    let sub: Subscription = sqlx::query_as(
      r#"
      DELETE FROM subscriptions WHERE uname = $1 AND channel_link = $2 RETURNING *;
      "#,
    )
    .bind(uname)
    .bind(channel_link)
    .fetch_one(&ctx.pool)
    .await?;

    Ok(sub)
  }

  pub async fn del_by_id(
    ctx: &AppState, 
    uname: &str, 
    id: u32,
  ) -> Result<Subscription, AppError> {
    let sub: Subscription = sqlx::query_as(
      r#"
      DELETE FROM subscriptions WHERE uname = $1 AND id = $2 RETURNING *;
      "#,
    )
    .bind(uname)
    .bind(id)
    .fetch_one(&ctx.pool)
    .await?;

    Ok(sub)
  }
}

#[derive(FromRow, Debug, Default, Serialize, Deserialize)]
pub struct FeedStatus {
  pub id: u32,
  pub uname: String,
  pub feed_url: String,
  pub read_status: u8,
  pub star_status: u8,
}

impl FeedStatus {
  // pub async fn del(
  //   ctx: &AppState,
  //   uname: &str,
  //   link: &str,
  // ) -> Result<FeedStatus, AppError> {
  //   // insert
  //   let del_status: FeedStatus = sqlx::query_as(
  //     r#"
  //     DELETE FROM feed_status WHERE uname = $1 AND feed_url = $2 RETURNING *;
  //     "#,
  //   )
  //   .bind(uname)
  //   .bind(link)
  //   .fetch_one(&ctx.pool)
  //   .await?;

  //   Ok(del_status)
  // }

  pub async fn new(
    ctx: &AppState,
    uname: &str,
    link: &str,
    action: &str,
    status: u8,
  ) -> Result<FeedStatus, AppError> {
    let new_status: FeedStatus = if action == "read" {
      sqlx::query_as(
        r#"
        INSERT INTO feed_status 
        (uname, feed_url, read_status)
        VALUES
        ($1, $2, $3)
        ON CONFLICT(uname, feed_url) DO UPDATE SET
          read_status = excluded.read_status
        RETURNING *;
        "#,
      )
      .bind(uname)
      .bind(link)
      .bind(status)
      .fetch_one(&ctx.pool)
      .await?
    } else {
      sqlx::query_as(
        r#"
        INSERT INTO feed_status 
        (uname, feed_url, star_status)
        VALUES
        ($1, $2, $3)
        ON CONFLICT(uname, feed_url) DO UPDATE SET
          star_status = excluded.star_status
        RETURNING *;
        "#,
      )
      .bind(uname)
      .bind(link)
      .bind(status)
      .fetch_one(&ctx.pool)
      .await?
    };

    Ok(new_status)
  }

  pub async fn get_star_list(
    ctx: &AppState,
    uname: &str,
  ) -> Result<Vec<Feed>, AppError> {
    let feeds: Vec<Feed> = sqlx::query_as(
      r#"
      SELECT * FROM feeds 
      WHERE feed_url IN (
        SELECT feed_url FROM feed_status
        WHERE uname = $1 AND star_status = $2
      );
      "#,
    )
    .bind(uname)
    .bind(1)
    .fetch_all(&ctx.pool)
    .await
    .unwrap_or_default();

    Ok(feeds)
  }

  pub async fn get_read_list(
    ctx: &AppState,
    uname: &str,
  ) -> Result<Vec<Feed>, AppError> {
    let feeds: Vec<Feed> = sqlx::query_as(
      r#"
      SELECT * FROM feeds 
      WHERE feed_url IN (
        SELECT feed_url FROM feed_status
        WHERE uname = $1 AND read_status = $2
      );
      "#,
    )
    .bind(uname)
    .bind(1)
    .fetch_all(&ctx.pool)
    .await
    .unwrap_or_default();

    Ok(feeds)
  }

  pub async fn check_star(
    ctx: &AppState,
    uname: &str,
    url: &str,
  ) -> Result<bool, AppError> {
    let res: FeedStatus = sqlx::query_as(
      r#"
      SELECT * FROM feed_status
      WHERE uname = $1 AND feed_url = $2 AND star_status = $3;
      "#,
    )
    .bind(uname)
    .bind(url)
    .bind(1)
    .fetch_one(&ctx.pool)
    .await?;
    
    println!("check res: {:?}", res);
    return Ok(true);
  }

  pub async fn check_read(
    ctx: &AppState,
    uname: &str,
    url: &str,
  ) -> Result<bool, AppError> {
    let res: FeedStatus = sqlx::query_as(
      r#"
      SELECT * FROM feed_status
      WHERE uname = $1 AND feed_url = $2 AND read_status = $3;
      "#,
    )
    .bind(uname)
    .bind(url)
    .bind(1)
    .fetch_one(&ctx.pool)
    .await?;
    
    println!("check res: {:?}", res);
    return Ok(true);
  }
}

pub async fn refresh_feeds_job(ctx: &AppState) -> Result<(), AppError> {
  let channels = Channel::get_list(ctx, 42, None).await?;
  for channel in channels {
    let url = channel.link;
    if let Some(res) = process_feed(&url, None, None).await {
      let feeds = res.1;
      Feed::add_feeds(ctx, feeds).await?;
    }
  }

  Ok(())
}
