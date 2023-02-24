//! Server side rendering function

use askama::Template;
use axum::body::{self, BoxBody, Full};
use axum::extract::rejection::FormRejection;
use axum::extract::FromRequest;
use axum::http::{Request, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::{async_trait, Form};
use serde::de::DeserializeOwned;
use serde::Deserialize;
use validator::Validate;

use crate::config::SiteConfig;
use crate::db::user::Claim;
use crate::error::{AppError, SsrError};

pub mod admin;
pub mod article;
pub mod auth;
pub mod feed;
pub mod home;
pub mod upload;
pub mod user;

pub fn into_response<T: Template>(t: &T, ext: &str) -> Response<BoxBody> {
  match t.render() {
    Ok(body) => Response::builder()
      .status(StatusCode::OK)
      .header("content-type", ext)
      .body(body::boxed(Full::from(body)))
      .unwrap_or_default(),
    Err(err) => Response::builder()
      .status(StatusCode::INTERNAL_SERVER_ERROR)
      .body(body::boxed(Full::from(format!("{err}"))))
      .unwrap_or_default(),
  }
}

#[derive(Template)]
#[template(path = "error.html")]
struct ErrorTmpl<'a> {
  page_data: PageData<'a>,
  status: String,
  error: String,
}

pub(crate) fn error_page(status: String, error: String) -> Response {
  let site_config = SiteConfig::default();
  let page_data = PageData::new("Error", &site_config, None, false);
  let page_error = ErrorTmpl {
    page_data,
    status,
    error,
  };

  into_response(&page_error, "html")
}

pub(crate) async fn handler_404() -> impl IntoResponse {
  SsrError::from(AppError::NotFound).into_response()
}

pub(super) struct ValidatedForm<T>(pub(super) T);

#[async_trait]
impl<T, S, B> FromRequest<S, B> for ValidatedForm<T>
where
  T: DeserializeOwned + Validate,
  S: Send + Sync,
  Form<T>: FromRequest<S, B, Rejection = FormRejection>,
  B: Send + 'static,
{
  type Rejection = SsrError;

  async fn from_request(
    req: Request<B>,
    state: &S,
  ) -> Result<Self, Self::Rejection> {
    match Form::<T>::from_request(req, state).await {
      Ok(Form(val)) if val.validate().is_ok() => Ok(ValidatedForm(val)),
      _ => {
        return Err(SsrError {
          status: StatusCode::BAD_REQUEST.to_string(),
          error: "Invalid Form, Please check input".into(),
        })
      }
    }
  }
}

// #[derive(Default)]
pub struct PageData<'a> {
  pub title: &'a str,
  pub claim: Option<Claim>,
  pub has_unread: bool,
  pub site_name: &'a str,
  pub site_slogan: &'a str,
  pub site_verification: &'a str,
}

impl<'a> PageData<'a> {
  pub fn new(
    title: &'a str,
    site_config: &'a SiteConfig,
    claim: Option<Claim>,
    has_unread: bool,
  ) -> Self {
    Self {
      title,
      claim,
      has_unread,
      site_name: &site_config.site_name,
      site_slogan: &site_config.slogan,
      site_verification: &site_config.verification,
    }
  }
}

#[derive(Deserialize)]
pub(crate) struct QueryParams {
  ord: Option<String>,
  perpage: Option<i64>,
  page: Option<i64>,
  // tag: Option<String>,
  // usr: Option<String>,
  tab: Option<String>,
}

/// custom filters for askama Templete
//
pub(super) mod filters {
  use crate::util::helper::{get_host, rm_html_tag};
  use askama::Result as TmplResult;
  use chrono::{NaiveDateTime, Utc};

  pub fn must_https(s: &str) -> TmplResult<String> {
    Ok(s.replacen("http:", "https:", 1))
  }

  pub fn num_unit(num: &u32) -> TmplResult<String> {
    let x = *num;
    let (n, u): (u32, &str) = if x > 1_000_000 {
      (x / 1_000_000, "M+")
    } else if x >= 1_000 {
      (x / 1_000, "K+")
    } else {
      (x, "")
    };
    let res = format!("{}{}", n, u);

    Ok(res)
  }

  pub fn calc_read_min(content: &str) -> TmplResult<usize> {
    let res = content.len() / 4 / 275;

    Ok(std::cmp::max(res, 1))
  }

  pub fn pluralize(num: &u32, pl: &str, sl: &str) -> TmplResult<String> {
    let res = if num != &1 {
      format!("{} {}", num, pl)
    } else {
      format!("{} {}", num, sl)
    };

    Ok(res)
  }

  pub fn host(s: &str) -> TmplResult<String> {
    let s_host = get_host(s);
    Ok(s_host)
  }

  pub fn inner_text(s: &str) -> TmplResult<String> {
    let text = rm_html_tag(s);
    Ok(text)
  }

  pub fn ts_date(timestamp: &i64, fmt: &str) -> TmplResult<String> {
    let now = Utc::now().naive_utc();
    let dt = NaiveDateTime::from_timestamp_opt(*timestamp, 0).unwrap_or(now);
    let off = (now - dt).num_minutes();
    let formatted = if off > 60 * 24 {
      // https://docs.rs/chrono/0.4.15/chrono/format/strftime/index.html
      let format: &str = if fmt.len() > 0 { fmt } else { "%a %Y-%m-%d" };
      dt.format(format).to_string()
    } else if off > 60 {
      format!("{}h ago", off / 60)
    } else if off < 1 {
      format!("Just Now")
    } else {
      format!("{}m ago", off)
    };

    Ok(formatted)
  }

  #[cfg(test)]
  mod tests {
    use super::*;

    #[test]
    fn test_ts_date() {
      assert_eq!(
        ts_date(&1673976654, "").unwrap(),
        String::from("Tue 2023-01-17")
      );
    }

    #[test]
    fn test_num_unit() {
      assert_eq!(num_unit(&13976654).unwrap(), String::from("13M+"));
      assert_eq!(num_unit(&96654).unwrap(), String::from("96K+"));
      assert_eq!(num_unit(&654).unwrap(), String::from("654"));
    }

    #[test]
    fn test_pluralize() {
      assert_eq!(pluralize(&1, "ms", "m").unwrap(), String::from("1 m"));
      assert_eq!(pluralize(&0, "ms", "m").unwrap(), String::from("0 ms"));
      assert_eq!(pluralize(&2, "ms", "m").unwrap(), String::from("2 ms"));
    }

    #[test]
    fn test_host() {
      assert_eq!(
        host("https://mdsilo.com/jh/#jhdkj").unwrap(),
        String::from("mdsilo.com")
      );
      assert_eq!(
        host("https://app.mdsilo.com/jh").unwrap(),
        String::from("app.mdsilo.com")
      );
      assert_eq!(
        host("https://www.mdsilo.com/jh/#jhdkj").unwrap(),
        String::from("mdsilo.com")
      );
    }

    #[test]
    fn test_inner_text() {
      assert_eq!(
        inner_text(r#"<a href="https://mdsilo.com/spc">mdsilo</a>"#).unwrap(),
        String::from("mdsilo")
      );
    }
  }
}
