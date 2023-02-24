//! ## Upload file

use crate::{
  config::{get_site_config, CONFIG},
  db::{
    sled::{get_status_timestamp, store_user_status},
    user::{ClaimCan, CREATE_PERMIT},
  },
  error::{AppError, SsrError},
  util::img::process_img,
};
use askama::Template;
use axum::{
  extract::{Multipart, State},
  response::IntoResponse,
};
use axum_macros::debug_handler;
use chrono::Utc;
use sled::Batch;
use tokio::fs;

use super::{into_response, PageData};
use crate::AppState as Ctx;

/// Page data: `upload.html`
#[derive(Template)]
#[template(path = "upload.html")]
struct UploadTmpl<'a> {
  page_data: PageData<'a>,
  fnames: Vec<String>,
  show_thumbnail: bool,
}

/// `GET /upload`
#[debug_handler]
pub(crate) async fn upload_page(
  State(ctx): State<Ctx>,
  check: ClaimCan<CREATE_PERMIT>,
) -> Result<impl IntoResponse, SsrError> {
  let site_config = get_site_config(&ctx.sled).unwrap_or_default();
  let claim = check.claim.unwrap_or_default();
  let uname = claim.uname.clone();
  let mut fnames = vec![];
  let uname_prefix = uname.as_bytes();
  let vals = ctx
    .sled
    .open_tree("user_uploads")
    .map_err(|_e| AppError::SledError)?
    .scan_prefix(uname_prefix)
    .values();
  for val in vals {
    let v = val.unwrap_or_default();
    let fname = String::from_utf8_lossy(&v).to_string();
    if fname.len() > 0 {
      fnames.push(fname);
    }
  }
  let page_data = PageData::new("Upload Files", &site_config, Some(claim), false);
  let page_upload = UploadTmpl {
    page_data,
    fnames,
    show_thumbnail: false,
  };

  Ok(into_response(&page_upload, "html"))
}

/// `POST /upload`
#[debug_handler]
pub(crate) async fn upload_file(
  State(ctx): State<Ctx>,
  check: ClaimCan<CREATE_PERMIT>,
  mut multipart: Multipart,
) -> Result<impl IntoResponse, SsrError> {
  let site_config = get_site_config(&ctx.sled).unwrap_or_default();
  let claim = check.claim.unwrap_or_default();
  let uname = claim.uname.clone();

  let now = Utc::now().timestamp();
  let last_upload =
    get_status_timestamp(&ctx.sled, &format!("{uname}_last_upload")).unwrap_or(0);
  if now - last_upload < site_config.upload_interval {
    return Err(AppError::WriteInterval.into());
  }

  let mut fnames = Vec::with_capacity(10);
  let mut batch = Batch::default();

  while let Some(field) = multipart
    .next_field()
    .await
    .map_err(|_e| AppError::MultiPartError)?
  {
    if fnames.len() > 10 {
      break;
    }

    let fname = field.file_name().unwrap_or_default().to_string();
    let data = field.bytes().await.map_err(|_e| AppError::MultiPartError)?;

    let file_data = if let Ok(img_format) = image::guess_format(&data) {
      if CONFIG.if_compress_img {
        if let Some(img_vec) = process_img(data, img_format) {
          img_vec
        } else {
          continue;
        }
      } else {
        data.to_vec()
      }
    } else {
      data.to_vec()
    };

    let real_fname = format!("{}-{}", uname, fname);
    let upload_path = format!("{}/{}", &CONFIG.upload_path, real_fname);

    fs::write(upload_path, &file_data).await.unwrap_or(());

    // uname bytes as prefix, so we can get fname later via scan_prefix
    let fname_bytes = fname.as_bytes();
    let key = [uname.as_bytes(), fname_bytes].concat();
    batch.insert(key, real_fname.as_bytes());
    // batch.insert(claim.uname.as_str(), fname.as_str());
    fnames.push(real_fname);
  }

  ctx
    .sled
    .open_tree("user_uploads")
    .map_err(|_e| AppError::SledError)?
    .apply_batch(batch)
    .unwrap_or(());
  store_user_status(&ctx.sled, &uname, "upload").unwrap_or(());

  let page_data = PageData::new("Uploaded Files", &site_config, Some(claim), false);
  let page_uploaded = UploadTmpl {
    page_data,
    fnames,
    show_thumbnail: true,
  };

  Ok(into_response(&page_uploaded, "html"))
}
