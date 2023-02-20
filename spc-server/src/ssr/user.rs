//! ## User
//! Profile page, setting

use super::{filters, into_response, PageData, QueryParams, ValidatedForm};
use crate::{
  config::get_site_config,
  db::{
    article::{Entry, QueryArticles, QueryPieces},
    sled::get_status_count,
    user::{ClaimCan, PubUser, User, CREATE_PERMIT, READ_PERMIT},
  },
  error::{AppError, SsrError},
  AppState as Ctx,
};

use askama::Template;
use axum::{
  extract::{Path, Query, State},
  response::{IntoResponse, Redirect},
};
// use axum_macros::debug_handler;
use serde::Deserialize;
use validator::Validate;

#[derive(Template)]
#[template(path = "profile.html")]
struct ProfileTmpl<'a> {
  page_data: PageData<'a>,
  user: User,
  entries: Vec<Entry>,
  post_count: u32,
  upload_count: u32,
  feed_count: u32,
  is_self: bool,
  can_create: bool,
  page: i64,
}

/// `GET user/:uname` tag page
pub(crate) async fn profile_page(
  State(ctx): State<Ctx>,
  Path(uname): Path<String>,
  Query(params): Query<QueryParams>,
  check: ClaimCan<READ_PERMIT>,
) -> Result<impl IntoResponse, SsrError> {
  if !check.can() {
    return Err(AppError::NoPermission.into());
  }
  let site_config = get_site_config(&ctx.sled).unwrap_or_default();
  let claim = check.claim;
  let claim_uname = claim.clone().unwrap_or_default().uname;
  let is_self = claim_uname == uname;
  let can_create = claim.clone().unwrap_or_default().can(CREATE_PERMIT);
  // let ord = params.ord.unwrap_or(String::from("desc"));
  let page = params.page.unwrap_or(1);
  let perpage = params.page.unwrap_or(42);

  let user = User::get(&ctx, &uname).await?;

  let article_list = QueryArticles::User(uname.clone(), 1, perpage, page)
    .get(&ctx)
    .await?
    .0;
  let piece_list = QueryPieces::User(uname.clone(), 1, perpage, page)
    .get(&ctx)
    .await?
    .0;

  let mut entries: Vec<Entry> = article_list
    .into_iter()
    .map(|a| a.into())
    .into_iter()
    .chain(piece_list.into_iter().map(|p| p.into()).into_iter())
    .collect();
  // sort per created_at
  entries.sort_by(|a, b| b.created_at.cmp(&a.created_at));

  // get status
  let tree = ctx
    .sled
    .open_tree("user_status")
    .map_err(|_e| AppError::SledError)?;
  let post_count = get_status_count(&tree, &format!("{uname}_post"))?;
  let upload_count = get_status_count(&tree, &format!("{uname}_upload"))?;
  let feed_count = get_status_count(&tree, &format!("{uname}_sub"))?;

  let page_title = format!("{} 's Home", uname);
  let page_data = PageData::new(&page_title, &site_config, claim, false);
  let profile_page = ProfileTmpl {
    page_data,
    user,
    entries,
    post_count,
    upload_count,
    feed_count,
    is_self,
    can_create,
    page,
  };

  Ok(into_response(&profile_page, "html"))
}

#[derive(Template)]
#[template(path = "user_setting.html")]
struct UserSettingTmpl<'a> {
  page_data: PageData<'a>,
  user: &'a PubUser,
}

/// `GET /user/:uname/setting`
pub(crate) async fn user_setting_view(
  State(ctx): State<Ctx>,
  Path(uname): Path<String>,
  check: ClaimCan<READ_PERMIT>,
) -> Result<impl IntoResponse, SsrError> {
  if !check.can() {
    return Err(AppError::NoPermission.into());
  }
  let claim = check.claim;
  let claim_uname = claim.clone().unwrap_or_default().uname;
  if claim_uname != uname {
    return Err(AppError::NoPermission.into());
  }

  let site_config = get_site_config(&ctx.sled)?;
  let page_data = PageData::new("User Setting", &site_config, claim, false);
  let user = User::get(&ctx, &uname).await?;
  let user_setting_page = UserSettingTmpl {
    page_data,
    user: &user.into(),
  };
  Ok(into_response(&user_setting_page, "html"))
}

#[derive(Deserialize, Validate)]
pub(crate) struct UserInfoForm {
  #[validate(length(min = 1, max = 64))]
  nickname: String,
  #[validate(length(min = 1, max = 1024))]
  about: String,
}

/// `POST /user/:uname/setting`
pub(crate) async fn user_setting_form(
  State(ctx): State<Ctx>,
  Path(uname): Path<String>,
  check: ClaimCan<READ_PERMIT>,
  ValidatedForm(input): ValidatedForm<UserInfoForm>,
) -> Result<impl IntoResponse, SsrError> {
  if !check.can() {
    return Err(AppError::NoPermission.into());
  }
  let claim = check.claim;
  let claim_uname = claim.unwrap_or_default().uname;
  if claim_uname != uname {
    return Err(AppError::NoPermission.into());
  }

  User::update(&ctx, &uname, &input.nickname, &input.about).await?;
  let target = format!("/user/{}", uname);
  Ok(Redirect::to(&target))
}
