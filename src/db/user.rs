//! models for user

use argon2::{
  password_hash::{
    rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString,
  },
  Argon2,
};
use axum::{
  async_trait, extract::FromRequestParts, headers::Cookie, http::request::Parts,
  RequestPartsExt, TypedHeader,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use ring::rand::{self, SecureRandom};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::util::identicon::Identicon;
use crate::{config::CONFIG, error::AppError, AppState};

pub(crate) const COOKIE_NAME: &str = "_SPC_ID_";

// user permission and mod permission
pub const READ_PERMIT: u8 = 0x01; // read...
pub const BASIC_PERMIT: u8 = 0x03; // star, follow, ...
pub const CREATE_PERMIT: u8 = 0x07; // create, edit self created and wiki-style content...
pub const EIDT_PERMIT: u8 = 0x0f; // edit or del others' creates
pub const MOD_PERMIT: u8 = 0x1f; // mod role
pub const ADMIN_PERMIT: u8 = 0xff; // admin

/// user
#[derive(FromRow, Default, Serialize, Debug)]
pub struct User {
  pub username: String,
  pub password_hash: String,
  pub nickname: String,
  pub join_at: i64,
  pub permission: u8,
  pub karma: u32,
  pub about: String,
}

/// user w/o password_hash
#[derive(Default, Serialize, Debug)]
pub struct PubUser {
  pub username: String,
  pub nickname: String,
  pub join_at: i64,
  pub permission: u8,
  pub karma: u32,
  pub about: String,
}

impl From<User> for PubUser {
  fn from(user: User) -> Self {
    PubUser {
      username: user.username,
      nickname: user.nickname,
      join_at: user.join_at,
      permission: user.permission,
      karma: user.karma,
      about: user.about,
    }
  }
}

impl User {
  pub async fn get(ctx: &AppState, uname: &str) -> Result<User, AppError> {
    let usr: User = sqlx::query_as(
      r#"
      SELECT username, password_hash, nickname, join_at, permission, karma, about 
      FROM users 
      WHERE username = $1
      "#,
    )
    .bind(uname)
    .fetch_one(&ctx.pool)
    .await?;

    Ok(usr)
  }

  pub async fn get_list(
    ctx: &AppState,
    ord: &str,
    perpage: i64,
    page: i64,
  ) -> Result<Vec<PubUser>, AppError> {
    let page_offset = std::cmp::max(0, page - 1);
    let users: Vec<User> = match ord.to_lowercase().trim() {
      "desc" => sqlx::query_as(
        r#"
          SELECT * FROM users 
          ORDER BY join_at DESC
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
          SELECT * FROM users 
          ORDER BY join_at ASC
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

    let pub_user_list: Vec<PubUser> = users.into_iter().map(|u| u.into()).collect();

    Ok(pub_user_list)
  }

  pub async fn update(
    ctx: &AppState,
    uname: &str,
    nickname: &str,
    about: &str,
  ) -> Result<PubUser, AppError> {
    if let Ok(_usr) = User::get(ctx, uname).await {
      // update
      let new_user: User = sqlx::query_as(
        r#"
        UPDATE users 
        SET nickname = $1, about = $2
        WHERE username = $3
        RETURNING *;
        "#,
      )
      .bind(&nickname)
      .bind(&about)
      .bind(&uname)
      .fetch_one(&ctx.pool)
      .await?;

      Ok(new_user.into())
    } else {
      return Err(AppError::NotFound);
    }
  }

  pub async fn mod_permission(
    ctx: &AppState,
    uname: &str,
    permission: u8,
  ) -> Result<(), AppError> {
    if let Ok(_usr) = User::get(ctx, uname).await {
      // mod
      let res = sqlx::query(
        r#"
        UPDATE users 
        SET permission = $1 
        WHERE username = $2;
        "#,
      )
      .bind(&permission)
      .bind(&uname)
      .execute(&ctx.pool)
      .await?;

      if res.rows_affected() == 0 {
        return Err(AppError::NotFound);
      }

      Ok(())
    } else {
      return Err(AppError::NotFound);
    }
  }
}

#[derive(Deserialize)]
pub(crate) struct AuthUser {
  pub username: String,
  pub password: String,
}

impl AuthUser {
  pub async fn register(&self, ctx: &AppState) -> Result<PubUser, AppError> {
    // check duplicate
    if let Ok(_check) = User::get(ctx, &self.username).await {
      return Err(AppError::NameExists);
    }
    // check admin
    let admin_name = CONFIG.admin_name.clone();
    let permission = if admin_name == self.username {
      ADMIN_PERMIT
    } else {
      BASIC_PERMIT
    };
    // hash password
    let psw_hash: String = hash_password(&self.password)?;
    // insert
    let result = sqlx::query(
      r#"
      INSERT INTO
      users (username, password_hash, join_at, permission)
      VALUES
      ($1, $2, $3, $4)
      ON CONFLICT(username) DO NOTHING
      "#,
    )
    .bind(&self.username)
    .bind(&psw_hash)
    .bind(&Utc::now().timestamp())
    .bind(&permission)
    .execute(&ctx.pool)
    .await?;

    if result.rows_affected() != 1 {
      return Err(AppError::NotFound);
    }

    // save avatar
    let avatar = format!("{}/{}.png", &CONFIG.avatars_path, &self.username);
    Identicon::new(&generate_salt(), 420).image().save(avatar)?;

    let new_user = User::get(ctx, &self.username).await?;

    Ok(new_user.into())
  }

  pub async fn auth(
    ctx: &AppState,
    uname: &str,
    psw: &str,
  ) -> Result<PubUser, AppError> {
    // check duplicate
    let check = User::get(ctx, uname).await?;
    if check.username != uname {
      return Err(AppError::UsernameInvalid);
    }
    // verify password
    if verify_password(psw, &check.password_hash) {
      return Ok(check.into());
    } else {
      return Err(AppError::Unauthorized);
    }
  }

  pub async fn change_password(
    ctx: &AppState,
    uname: &str,
    old: &str,
    new: &str,
  ) -> Result<(), AppError> {
    // check duplicate
    let check = User::get(ctx, &uname).await?;
    if check.username != uname {
      return Err(AppError::UsernameInvalid);
    }
    // verify old password
    if verify_password(&old, &check.password_hash) {
      let new_psw_hash: String = hash_password(&new)?;
      let res = sqlx::query(
        r#"
        UPDATE users 
        SET password_hash = $1 
        WHERE username = $2
        RETURNING *;
        "#,
      )
      .bind(&new_psw_hash)
      .bind(&uname)
      .execute(&ctx.pool)
      .await?;

      if res.rows_affected() == 0 {
        return Err(AppError::Unauthorized);
      }

      Ok(())
    } else {
      return Err(AppError::Unauthorized);
    }
  }
}

// # for jwt auth
// # encode authed user info as token w/ secret key,
// # then send to client as cookie;
// # request w/ such token to server,
// # decode token to get authed user info w/ secret key
//
// jwt Token auth: Claim, token
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Claim {
  pub iss: String, // issuer
  pub sub: String, // subject
  pub iat: i64,    // issued at
  pub exp: i64,    // expiry
  pub uname: String,
  pub permission: u8,
  // more column can be added if needed
}

// claims's constructor
impl Claim {
  pub fn new(uname: &str, permit: u8) -> Self {
    Claim {
      iss: (&CONFIG.name).into(),
      sub: "auth".into(),
      iat: Utc::now().timestamp(),
      exp: (Utc::now() + Duration::hours(24 * 14)).timestamp(),
      uname: uname.to_owned(),
      permission: permit,
    }
  }

  // check permission
  pub fn can(&self, permission: u8) -> bool {
    (self.permission & permission) == permission
  }

  /// generate cookie
  pub fn generate_cookie(user: PubUser) -> Result<String, AppError> {
    let seconds = 2 * 7 * 24 * 3600; // todo config

    let claim = encode_token(&user)?;
    let cookie = format!(
        "{COOKIE_NAME}={claim}; SameSite=Strict; Path=/; Secure; HttpOnly; Max-Age={seconds}"
      );

    Ok(cookie)
  }
}

// just try to get claim, not return err,
// will return default claim but not break if no cookie
#[async_trait]
impl<S> FromRequestParts<S> for Claim
where
  S: Send + Sync,
{
  type Rejection = ();

  async fn from_request_parts(
    parts: &mut Parts,
    _: &S,
  ) -> Result<Self, Self::Rejection> {
    let cookie: Option<TypedHeader<Cookie>> = parts.extract().await.unwrap_or(None);
    let tok_cookie = cookie.as_ref().and_then(|cookie| cookie.get(COOKIE_NAME));
    let claim = if let Some(tok) = tok_cookie {
      match decode_token(tok) {
        Ok(t_claim) => t_claim,
        _ => return Ok(Claim::default()),
      }
    } else {
      return Ok(Claim::default());
    };

    // check expire
    let now = Utc::now();
    if claim.exp < now.timestamp() {
      return Ok(Claim::default());
    }

    Ok(claim)
  }
}
#[derive(Default, Debug)]
pub struct ClaimCan<const CAN: u8> {
  pub claim: Option<Claim>,
}

impl<const CAN: u8> ClaimCan<CAN> {
  pub fn can(&self) -> bool {
    if let Some(ref claim) = self.claim {
      (claim.permission & CAN) == CAN
    } else {
      false
    }
  }
}

// just try to get claim and check permission, not return err,
// will return default claim but not break if no cookie
#[async_trait]
impl<S, const CAN: u8> FromRequestParts<S> for ClaimCan<CAN>
where
  S: Send + Sync,
{
  type Rejection = ();

  async fn from_request_parts(
    parts: &mut Parts,
    _: &S,
  ) -> Result<Self, Self::Rejection> {
    let cookie: Option<TypedHeader<Cookie>> = parts.extract().await.unwrap_or(None);
    let tok_cookie = cookie.as_ref().and_then(|cookie| cookie.get(COOKIE_NAME));
    let claim = if let Some(tok) = tok_cookie {
      match decode_token(tok) {
        Ok(t_claim) => t_claim,
        _ => return Ok(ClaimCan::default()),
      }
    } else {
      return Ok(ClaimCan::default());
    };

    // check expire
    let now = Utc::now();
    if claim.exp < now.timestamp() {
      // TODO: log
      return Ok(ClaimCan::default());
    }

    // check permission
    // FIXME: it is not permission in db,
    // a potential issue: mod user not affect immediately
    if !claim.can(CAN) {
      // TODO: log
      return Ok(ClaimCan::default());
    }

    Ok(ClaimCan { claim: Some(claim) })
  }
}

// Helper function for auth
//
fn get_secret() -> String {
  let cfg_secret = CONFIG.secret_key.clone();
  if cfg_secret.trim().len() > 16 {
    cfg_secret
  } else {
    "AH8aR9uyS37s5SeCREkY".into()
  }
}

// User info -> token
fn encode_token(data: &PubUser) -> Result<String, AppError> {
  let claim = Claim::new(data.username.as_str(), data.permission);
  encode(
    &Header::default(),
    &claim,
    &EncodingKey::from_secret(get_secret().as_ref()),
  )
  .map_err(|_err| AppError::EncodeClaimError)
}
// token -> claim including user info
fn decode_token(token: &str) -> Result<Claim, AppError> {
  decode::<Claim>(
    token,
    &DecodingKey::from_secret(get_secret().as_ref()),
    &Validation::default(),
  )
  .map(|data| Ok(data.claims))
  .map_err(|_err| AppError::DecodeClaimError)?
}

fn hash_password(plain: &str) -> Result<String, AppError> {
  let salt = SaltString::generate(&mut OsRng);
  let hash_password = Argon2::default()
    .hash_password(plain.as_bytes(), &salt)
    .map_err(|_| AppError::HashPasswordError)?
    .to_string();

  Ok(hash_password)
}

fn verify_password(psw: &str, hash_psw: &str) -> bool {
  if let Ok(parsed_hash) = PasswordHash::new(hash_psw) {
    Argon2::default()
      .verify_password(psw.as_bytes(), &parsed_hash)
      .is_ok()
  } else {
    false
  }
}

fn generate_salt() -> [u8; 64] {
  let rng = rand::SystemRandom::new();
  let mut salt = [0_u8; 64];
  rng.fill(&mut salt).unwrap_or(());
  salt
}
