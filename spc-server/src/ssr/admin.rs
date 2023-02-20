//! ## Admin:
//! mod user, config site

use super::{filters, into_response, PageData, QueryParams, ValidatedForm};
use crate::{
  config::{get_site_config, SiteConfig},
  db::user::{ClaimCan, PubUser, User, ADMIN_PERMIT, MOD_PERMIT},
  error::{AppError, SsrError},
  AppState as Ctx,
};

use askama::Template;
use axum::{
  extract::{Path, Query, State},
  response::{IntoResponse, Redirect},
};
// use axum_macros::debug_handler;
use bincode::config::standard;

#[derive(Template)]
#[template(path = "site_config.html")]
struct SiteConfigTmpl<'a> {
  site_config: &'a SiteConfig,
  page_data: PageData<'a>,
}

/// `GET /siteconfig`
pub(crate) async fn site_config_view(
  State(ctx): State<Ctx>,
  check: ClaimCan<ADMIN_PERMIT>,
) -> Result<impl IntoResponse, SsrError> {
  if !check.can() {
    return Err(AppError::NoPermission.into());
  }
  let site_config = get_site_config(&ctx.sled)?;
  let claim = check.claim;
  let page_data = PageData::new("Site Config", &site_config, claim, false);
  let site_config_page = SiteConfigTmpl {
    site_config: &site_config,
    page_data,
  };
  Ok(into_response(&site_config_page, "html"))
}

/// `POST /siteconfig`
pub(crate) async fn save_site_config(
  State(ctx): State<Ctx>,
  check: ClaimCan<ADMIN_PERMIT>,
  ValidatedForm(input): ValidatedForm<SiteConfig>,
) -> Result<impl IntoResponse, SsrError> {
  if !check.can() {
    return Err(AppError::NoPermission.into());
  }

  let site_config =
    bincode::encode_to_vec(&input, standard()).map_err(|_e| AppError::SledError)?;
  ctx
    .sled
    .insert("site_config", site_config)
    .map_err(|_e| AppError::SledError)?;
  Ok(Redirect::to("/siteconfig"))
}

#[derive(Template)]
#[template(path = "user_list.html")]
struct UserListTmpl<'a> {
  page_data: PageData<'a>,
  users: Vec<PubUser>,
  admin: PubUser,
  page: i64,
}

/// `GET /admin/user_list` admin page
pub(crate) async fn user_list_page(
  State(ctx): State<Ctx>,
  Query(params): Query<QueryParams>,
  check: ClaimCan<MOD_PERMIT>,
) -> Result<impl IntoResponse, SsrError> {
  if !check.can() {
    return Err(AppError::NoPermission.into());
  }
  let claim = check.claim;
  let uname = claim.clone().unwrap_or_default().uname;
  // check the permission in server db
  let admin = User::get(&ctx, &uname).await?;
  if admin.permission & MOD_PERMIT != MOD_PERMIT {
    return Err(AppError::NoPermission.into());
  }

  let ord = params.ord.unwrap_or(String::from("desc"));
  let page = params.page.unwrap_or(1);
  let perpage = params.perpage.unwrap_or(42);

  let site_config = get_site_config(&ctx.sled).unwrap_or_default();
  let page_data = PageData::new("Admin: mod users", &site_config, claim, false);
  let users = User::get_list(&ctx, &ord, perpage, page).await?;
  let userlist_page = UserListTmpl {
    page_data,
    users,
    admin: admin.into(),
    page,
  };

  Ok(into_response(&userlist_page, "html"))
}

/// MOD USER.
/// `GET /admin/:uname/mod/:permission` mod user's permission
pub(crate) async fn mod_user(
  State(ctx): State<Ctx>,
  Path((uname, permission)): Path<(String, u8)>,
  check: ClaimCan<MOD_PERMIT>,
) -> Result<impl IntoResponse, SsrError> {
  if !check.can() {
    return Err(AppError::NoPermission.into());
  }
  let claim = check.claim;
  let admin_uname = claim.clone().unwrap_or_default().uname;
  // check the permission in server db
  let admin = User::get(&ctx, &admin_uname).await?;
  let admin_permit = admin.permission;
  if admin_permit & MOD_PERMIT != MOD_PERMIT || admin_permit <= permission {
    return Err(AppError::NoPermission.into());
  }

  User::mod_permission(&ctx, &uname, permission).await?;

  Ok(Redirect::to("/admin/user_list"))
}
