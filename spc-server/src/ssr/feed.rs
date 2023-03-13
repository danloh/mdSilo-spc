//! ## RSS or Atom feed
//! subscrible and read feed

use askama::Template;
use axum::Form;
use axum::{
  extract::{Path, Query, State},
  response::{IntoResponse, Redirect},
};
use serde::Deserialize;
use validator::Validate;

use super::{filters, into_response, PageData, QueryParams, ValidatedForm};
use crate::config::get_site_config;
use crate::db::feed::{Channel, Feed, Subscription};
use crate::db::sled::store_user_status;
use crate::db::user::{BASIC_PERMIT, MOD_PERMIT};
use crate::error::SsrError;
use crate::util::feed::process_feed;
use crate::{
  db::user::{ClaimCan, CREATE_PERMIT, READ_PERMIT},
  error::AppError,
  AppState as Ctx,
};

/// Page data: `channel_preload.html`
#[derive(Template)]
#[template(path = "channel_preload.html")]
struct ChannelPreloadTmpl<'a> {
  page_data: PageData<'a>,
}

/// `GET /channel_preload`
pub(crate) async fn channel_preload_page(
  State(ctx): State<Ctx>,
  check: ClaimCan<BASIC_PERMIT>,
) -> Result<impl IntoResponse, SsrError> {
  if !check.can() {
    return Err(AppError::NoPermission.into());
  }
  let claim = check.claim;
  let site_config = get_site_config(&ctx.sled).unwrap_or_default();
  let page_data = PageData::new("Add Channel", &site_config, claim, false);
  let preload_page = ChannelPreloadTmpl { page_data };

  Ok(into_response(&preload_page, "html"))
}

/// Form data: preload feed form
#[derive(Deserialize, Validate)]
pub(crate) struct ChannelPreloadForm {
  link: String,
}

/// `POST /channel_preload`
pub(crate) async fn channel_preload_form(
  State(ctx): State<Ctx>,
  check: ClaimCan<BASIC_PERMIT>,
  Form(form): Form<ChannelPreloadForm>,
) -> Result<impl IntoResponse, SsrError> {
  if !check.can() {
    return Err(AppError::NoPermission.into());
  }
  let claim = check.claim;

  // preload Channel from db or via request
  let channel_link = form.link;
  // get channel
  let channel = match Channel::get_by_link(&ctx, &channel_link).await {
    Ok(channel) => channel,
    _ => {
      // via request
      match process_feed(&channel_link, None, None).await {
        Some(res) => res.0,
        None => return Err(AppError::FeedError.into()),
      }
    }
  };

  let channel_form = ChannelAddForm {
    link: channel.link,
    title: channel.title,
    intro: channel.intro,
    ty: channel.ty,
    is_public: 1,
  };
  // then go to channel_add page
  let site_config = get_site_config(&ctx.sled).unwrap_or_default();
  let page_data = PageData::new("Add Channel", &site_config, claim, false);
  let add_page = ChannelAddTmpl {
    page_data,
    channel: &channel_form,
  };

  Ok(into_response(&add_page, "html"))
}

/// Form data: `/channel_add` add channel.
#[derive(Deserialize, Validate, Default)]
pub(crate) struct ChannelAddForm {
  link: String,
  #[validate(length(min = 1, max = 256))]
  title: String,
  #[validate(length(min = 1, max = 512))]
  intro: String,
  #[validate(length(min = 1, max = 16))]
  ty: String,
  is_public: u8, // 0 or 1
}

#[derive(Template)]
#[template(path = "channel_add.html")]
struct ChannelAddTmpl<'a> {
  page_data: PageData<'a>,
  channel: &'a ChannelAddForm,
}

/// `GET /channel_add`
pub(crate) async fn channel_add_page(
  State(ctx): State<Ctx>,
  check: ClaimCan<BASIC_PERMIT>,
) -> Result<impl IntoResponse, SsrError> {
  if !check.can() {
    return Err(AppError::NoPermission.into());
  }
  let claim = check.claim;
  let site_config = get_site_config(&ctx.sled).unwrap_or_default();
  let page_data = PageData::new("Add Channel", &site_config, claim, false);
  let add_page = ChannelAddTmpl {
    page_data,
    channel: &(ChannelAddForm::default()),
  };

  Ok(into_response(&add_page, "html"))
}

/// `POST /channel_add`
pub(crate) async fn channel_add_form(
  State(ctx): State<Ctx>,
  check: ClaimCan<BASIC_PERMIT>,
  ValidatedForm(input): ValidatedForm<ChannelAddForm>,
) -> Result<impl IntoResponse, SsrError> {
  if !check.can() {
    return Err(AppError::NoPermission.into());
  }
  let claim = check.claim;
  let uname = claim.unwrap_or_default().uname;

  let channel = Channel {
    link: input.link,
    title: input.title,
    intro: input.intro,
    ty: input.ty,
  };

  // upsert channel
  let new_channel = channel.new(&ctx).await?;

  // subscription
  let is_pub = if input.is_public == 0 { false } else { true };
  Subscription::new(&ctx, &uname, &new_channel.link, &new_channel.title, is_pub)
    .await?;
  // record action: post new piece
  store_user_status(&ctx.sled, &uname, "sub").unwrap_or(());

  Ok(Redirect::to("/feed_reader"))
}

/// `GET /channel_del/:link` delete subscription
pub(crate) async fn del_channel(
  State(ctx): State<Ctx>,
  Path(link): Path<String>,
  check: ClaimCan<MOD_PERMIT>,
) -> Result<impl IntoResponse, SsrError> {
  if !check.can() {
    return Err(AppError::NoPermission.into());
  }

  Channel::del(&ctx, &link).await?;

  Ok(Redirect::to("/feed_reader"))
}

/// `GET /unsubscribe/:id` delete subscription
pub(crate) async fn unsubscribe(
  State(ctx): State<Ctx>,
  Path(id): Path<u32>,
  check: ClaimCan<CREATE_PERMIT>,
) -> Result<impl IntoResponse, SsrError> {
  if !check.can() {
    return Err(SsrError::from(AppError::NoPermission));
  }
  let claim = check.claim.unwrap_or_default();
  let uname = claim.clone().uname;

  // check uid matched
  let sub = Subscription::get(&ctx, id).await?;
  if sub.uname == uname {
    Subscription::del(&ctx, id).await?;
  } else {
    return Err(AppError::NoPermission.into());
  }

  Ok(Redirect::to("/feed_reader"))
}

/// `GET /mod_subscription/:id/` mod subscription pub
pub(crate) async fn mod_subscription(
  State(ctx): State<Ctx>,
  Path(id): Path<u32>,
  check: ClaimCan<CREATE_PERMIT>,
) -> Result<impl IntoResponse, SsrError> {
  if !check.can() {
    return Err(SsrError::from(AppError::NoPermission));
  }
  let claim = check.claim.unwrap_or_default();
  let uname = claim.clone().uname;

  // check uid matched
  let sub = Subscription::get(&ctx, id).await?;
  if sub.uname == uname {
    let is_pub = !sub.is_public;
    Subscription::mod_public(&ctx, id, is_pub).await?;
  } else {
    return Err(AppError::NoPermission.into());
  }

  Ok(Redirect::to("/feed_reader"))
}

#[derive(Template)]
#[template(path = "feed_reader.html")]
struct FeedReaderTmpl<'a> {
  page_data: PageData<'a>,
  channels: Vec<Subscription>,
  feeds: Vec<Feed>,
  current_channel: &'a str, // channel link or all
}

/// `GET /feed_reader?channel=`
pub(crate) async fn feed_reader_page(
  State(ctx): State<Ctx>,
  Query(params): Query<QueryParams>,
  check: ClaimCan<READ_PERMIT>,
) -> Result<impl IntoResponse, SsrError> {
  let site_config = get_site_config(&ctx.sled).unwrap_or_default();
  let claim = check.claim;
  let uname = claim.clone().unwrap_or_default().uname;
  // let ord = params.ord.unwrap_or(String::from("desc"));
  let page = params.page.unwrap_or(1);
  let perpage = params.perpage.unwrap_or(42);
  let channel = params.tab.unwrap_or(String::from("All"));

  let channels = Subscription::get_list(&ctx, &uname, perpage, page).await?;

  let ch_links: Vec<String> =
    channels.iter().map(|ch| ch.channel_link.clone()).collect();
  refresh_feeds(&ctx, ch_links).await.unwrap_or(());

  let feeds = if channel == "All" {
    Feed::get_list_by_user(&ctx, &uname, false, perpage, page).await?
  } else {
    Feed::get_list_by_channel(&ctx, &channel, perpage, page).await?
  };

  let page_data = PageData::new("Feed Reader", &site_config, claim, false);
  let reader_page = FeedReaderTmpl {
    page_data,
    channels,
    feeds,
    current_channel: &channel,
  };

  Ok(into_response(&reader_page, "html"))
}

/// refresh_feeds
async fn refresh_feeds(ctx: &Ctx, channels: Vec<String>) -> Result<(), AppError> {
  for url in channels {
    if let Some(res) = process_feed(&url, None, None).await {
      let feeds = res.1;
      Feed::add_feeds(ctx, feeds).await?;
    }
  }

  Ok(())
}
