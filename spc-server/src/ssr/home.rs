//! ## Home
//! landing page, about page...

use askama::Template;
use axum::{
  body::{self, BoxBody, Empty},
  extract::State,
  headers::HeaderName,
  http::{HeaderMap, HeaderValue, StatusCode},
  response::{IntoResponse, Response},
  routing::{get_service, MethodRouter},
};
use tower_http::services::{ServeDir, ServeFile};

use crate::{
  config::{get_site_config, CSS, JS, FAVICON, MANIFEST},
  db::user::{ClaimCan, READ_PERMIT},
  error::SsrError,
  util::md::md2html,
  AppState as Ctx,
};

use super::{into_response, PageData};

/// Page data: `article_new.html`
#[derive(Template)]
#[template(path = "home.html")]
struct HomeTmpl<'a> {
  page_data: PageData<'a>,
  page_content: String,
  as_page: &'a str, // home or about, for style
}

/// `GET /` index page
pub(crate) async fn home_page(
  State(ctx): State<Ctx>,
  check: ClaimCan<READ_PERMIT>,
) -> Result<impl IntoResponse, SsrError> {
  let cfg = get_site_config(&ctx.sled).unwrap_or_default();
  let page_data = PageData::new("Home", &cfg, check.claim, false);
  let landing_page = md2html(&cfg.landing_page, "articlepage", "tag");
  let home_page = HomeTmpl {
    page_data,
    page_content: landing_page,
    as_page: "home",
  };
  Ok(into_response(&home_page, "html"))
}

/// `GET /about` index page
pub(crate) async fn about_page(
  State(ctx): State<Ctx>,
  check: ClaimCan<READ_PERMIT>,
) -> Result<impl IntoResponse, SsrError> {
  let cfg = get_site_config(&ctx.sled).unwrap_or_default();
  let page_data = PageData::new("About", &cfg, check.claim, false);
  let about = md2html(&cfg.about_page, "articlepage", "tag");
  let about_page = HomeTmpl {
    page_data,
    page_content: about,
    as_page: "about",
  };
  Ok(into_response(&about_page, "html"))
}

/// `GET /health_check`
pub(crate) async fn health_check() -> Response<BoxBody> {
  Response::builder()
    .status(StatusCode::OK)
    .body(body::boxed(Empty::new()))
    .unwrap_or_default()
}

/// serve static directory
pub(crate) async fn serve_dir(dir: &str) -> MethodRouter {
  let fallback = get_service(ServeFile::new(format!("{dir}/index.html")));
  get_service(ServeDir::new(dir).precompressed_gzip().fallback(fallback))
}

/// serve style css file
pub(crate) async fn static_style() -> (HeaderMap, &'static str) {
  let mut headers = HeaderMap::new();

  headers.insert(
    HeaderName::from_static("content-type"),
    HeaderValue::from_static("text/css"),
  );
  headers.insert(
    HeaderName::from_static("cache-control"),
    HeaderValue::from_static("public, max-age=1209600, s-maxage=86400"),
  );

  (headers, &CSS)
}

/// serve js file
pub(crate) async fn static_js() -> (HeaderMap, &'static str) {
  let mut headers = HeaderMap::new();

  headers.insert(
    HeaderName::from_static("content-type"),
    HeaderValue::from_static("text/javascript"),
  );
  headers.insert(
    HeaderName::from_static("cache-control"),
    HeaderValue::from_static("public, max-age=1209600, s-maxage=86400"),
  );

  (headers, &JS)
}

/// serve favicon
pub(crate) async fn favicon() -> (HeaderMap, &'static str) {
  let mut headers = HeaderMap::new();

  headers.insert(
    HeaderName::from_static("content-type"),
    HeaderValue::from_static("image/svg+xml"),
  );
  headers.insert(
    HeaderName::from_static("cache-control"),
    HeaderValue::from_static("public, max-age=1209600, s-maxage=86400"),
  );

  (headers, &FAVICON)
}

/// serve manifest file 
pub(crate) async fn manifest() -> (HeaderMap, &'static str) {
  let mut headers = HeaderMap::new();

  headers.insert(
    HeaderName::from_static("content-type"),
    HeaderValue::from_static("application/json"),
  );
  headers.insert(
    HeaderName::from_static("cache-control"),
    HeaderValue::from_static("public, max-age=1209600, s-maxage=86400"),
  );

  (headers, &MANIFEST)
}
