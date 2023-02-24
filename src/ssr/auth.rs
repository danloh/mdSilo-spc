//! ## Auth:
//! sign up, sign in...

use super::{into_response, PageData, ValidatedForm};
use askama::Template;
use axum::{
  extract::{Form, State},
  http::{header::SET_COOKIE, HeaderMap},
  response::{IntoResponse, Redirect},
};
// use axum_macros::debug_handler;
use captcha::{CaptchaName, Difficulty};
use serde::Deserialize;
use validator::Validate;

use crate::{
  config::get_site_config,
  db::{
    sled::gen_expirable_id,
    user::{AuthUser, Claim, ClaimCan, COOKIE_NAME, READ_PERMIT},
  },
  error::{AppError, SsrError},
  AppState as Ctx,
};

/// Page data: `signin.html`
#[derive(Template)]
#[template(path = "signin.html")]
struct SigninTmpl<'a> {
  page_data: PageData<'a>,
}

/// `GET /signin`
pub(crate) async fn signin_page(
  State(ctx): State<Ctx>,
  check: ClaimCan<READ_PERMIT>,
) -> Result<impl IntoResponse, SsrError> {
  let claim = check.claim;
  if claim.is_some() {
    return Ok(Redirect::to("/explore").into_response());
  }

  let site_config = get_site_config(&ctx.sled).unwrap_or_default();
  let page_data = PageData::new("Sign in", &site_config, claim, false);
  let page_signin = SigninTmpl { page_data };
  Ok(into_response(&page_signin, "html"))
}

/// Form data: `/signin`
#[derive(Deserialize)]
pub(crate) struct SigninForm {
  username: String,
  password: String,
}

/// `POST /signin`
pub(crate) async fn signin_form(
  State(ctx): State<Ctx>,
  Form(input): Form<SigninForm>,
) -> impl IntoResponse {
  if let Ok(usr) = AuthUser::auth(&ctx, &input.username, &input.password).await {
    let site_config = get_site_config(&ctx.sled).unwrap_or_default();
    if site_config.read_only > 0 && usr.permission != u8::MAX {
      return Err(SsrError::from(AppError::ReadOnly));
    }
    let mut headers = HeaderMap::new();
    let cookie = Claim::generate_cookie(usr)?;
    if let Ok(ck) = cookie.parse() {
      headers.insert(SET_COOKIE, ck);
    }
    if headers.is_empty() {
      return Err(SsrError::from(AppError::AuthError));
    }
    Ok((headers, Redirect::to("/explore")))
  } else {
    Err(SsrError::from(AppError::AuthError))
  }
}

/// Page data: `signup.html`
#[derive(Template)]
#[template(path = "signup.html")]
struct SignupTmpl<'a> {
  page_data: PageData<'a>,
  captcha_id: String,
  captcha_img: String,
}

/// `GET /signup`
pub(crate) async fn signup_page(
  State(ctx): State<Ctx>,
  check: ClaimCan<READ_PERMIT>,
) -> Result<impl IntoResponse, SsrError> {
  let site_config = get_site_config(&ctx.sled).unwrap_or_default();
  if site_config.read_only > 0 {
    return Err(SsrError::from(AppError::ReadOnly));
  }

  let claim = check.claim;
  if claim.is_some() {
    return Ok(Redirect::to("/explore").into_response());
  }

  let page_data = PageData::new("Sign up", &site_config, None, false);
  let captcha_difficulty = match site_config.captcha_difficulty.as_str() {
    "Easy" => Difficulty::Easy,
    "Medium" => Difficulty::Medium,
    _ => Difficulty::Hard,
  };

  let captcha_name = match site_config.captcha_name.as_str() {
    "Amelia" => CaptchaName::Amelia,
    "Lucy" => CaptchaName::Lucy,
    _ => CaptchaName::Mila,
  };

  let captcha = captcha::by_name(captcha_difficulty, captcha_name);
  let captcha_id = gen_expirable_id(60, None);

  ctx
    .sled
    .open_tree("captcha")
    .map_err(|_e| AppError::SledError)?
    .insert(&captcha_id, &*captcha.chars_as_string())
    .map_err(|_e| AppError::SledError)?;

  let page_signup = SignupTmpl {
    page_data,
    captcha_id,
    captcha_img: captcha.as_base64().unwrap_or_default(),
  };
  Ok(into_response(&page_signup, "html"))
}

/// Form data: `/signup`
#[derive(Deserialize, Validate)]
pub(crate) struct SignupForm {
  #[validate(length(min = 1, max = 64))]
  username: String,
  #[validate(must_match(other = "password2", message = "not match"))]
  password: String,
  #[validate(length(min = 7))]
  password2: String,
  captcha_id: String,
  captcha_val: String,
}

/// `POST /signup`
pub(crate) async fn signup_form(
  State(ctx): State<Ctx>,
  ValidatedForm(input): ValidatedForm<SignupForm>,
) -> Result<impl IntoResponse, SsrError> {
  let site_config = get_site_config(&ctx.sled).unwrap_or_default();
  if site_config.read_only > 0 {
    return Err(SsrError::from(AppError::ReadOnly));
  }
  let check = input
    .username
    .chars()
    .next()
    .unwrap_or_default()
    .is_numeric()
    || input.username.chars().any(char::is_control)
    || input.username.contains(['@', '#']);
  if check {
    return Err(SsrError::from(AppError::UsernameInvalid));
  }

  let captcha_char = ctx
    .sled
    .open_tree("captcha")
    .map_err(|_e| AppError::SledError)?
    .remove(&input.captcha_id)
    .map_err(|_e| AppError::SledError)?
    .ok_or(AppError::CaptchaError)?;
  let captcha_val = String::from_utf8(captcha_char.to_vec()).unwrap_or_default();

  // println!("captcha val: {:?}", captcha_val);
  // println!("input captcha: {:?}", input.captcha_val);
  if captcha_val != input.captcha_val {
    return Err(SsrError::from(AppError::CaptchaError));
  }

  let reg_user = AuthUser {
    username: input.username,
    password: input.password,
  };
  let user = reg_user.register(&ctx).await?;
  let cookie = Claim::generate_cookie(user)?;
  let mut headers = HeaderMap::new();
  headers.insert(
    SET_COOKIE,
    cookie.parse().map_err(|_e| AppError::StrParseError)?,
  );

  Ok((headers, Redirect::to("/explore")))
}

/// `GET /signout`
pub(crate) async fn sign_out() -> Result<impl IntoResponse, SsrError> {
  let mut headers = HeaderMap::new();
  let ck = format!(
    "{COOKIE_NAME}=deleted; SameSite=Strict; Path=/; Secure; HttpOnly; expires=Thu, 01 Jan 1970 00:00:00 GMT"
  );
  headers.insert(
    SET_COOKIE,
    ck.parse().map_err(|_e| AppError::StrParseError)?,
  );
  Ok((headers, Redirect::to("/explore")))
}

/// Page data: `change_psw.html`
#[derive(Template)]
#[template(path = "change_psw.html")]
struct ChangePswTmpl<'a> {
  page_data: PageData<'a>,
}

/// `GET /change_password`
pub(crate) async fn change_psw_page(
  State(ctx): State<Ctx>,
  check: ClaimCan<READ_PERMIT>,
) -> Result<impl IntoResponse, SsrError> {
  let claim = check.claim;
  if claim.is_some() {
    return Ok(Redirect::to("/explore").into_response());
  }

  let site_config = get_site_config(&ctx.sled).unwrap_or_default();
  let page_data = PageData::new("Change Password", &site_config, None, false);

  let page_signup = ChangePswTmpl { page_data };
  Ok(into_response(&page_signup, "html"))
}

/// Form data: `/change_password`
#[derive(Deserialize, Validate)]
pub(crate) struct ChangePswForm {
  #[validate(length(min = 7))]
  old_psw: String,
  #[validate(length(min = 7))]
  new_psw: String,
}

/// `POST /change_password`
pub(crate) async fn change_psw_form(
  State(ctx): State<Ctx>,
  check: ClaimCan<READ_PERMIT>,
  ValidatedForm(input): ValidatedForm<ChangePswForm>,
) -> Result<impl IntoResponse, SsrError> {
  let claim = check.claim;
  if claim.is_some() {
    return Err(AppError::Unauthorized.into());
  }
  let uname = claim.unwrap_or_default().uname;

  AuthUser::change_password(&ctx, &uname, &input.old_psw, &input.new_psw).await?;

  let mut headers = HeaderMap::new();
  let ck = format!(
    "{COOKIE_NAME}=deleted; SameSite=Strict; Path=/; Secure; HttpOnly; expires=Thu, 01 Jan 1970 00:00:00 GMT"
  );
  headers.insert(
    SET_COOKIE,
    ck.parse().map_err(|_e| AppError::StrParseError)?,
  );

  Ok((headers, Redirect::to("/signin")))
}
