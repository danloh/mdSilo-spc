//! ## sled db:   
//! to store some status or temp data,
//! like: page view count, captcha, site config, user status, user upload history, etc.  
//!

use crate::error::AppError;
use chrono::Utc;
use nanoid::nanoid;
use sled::{Db, IVec, Tree};

/// record users' action: such as how many articles/comments...
/// ty can be: post|upload|sub|comment
///
pub fn store_user_status(sled: &Db, uname: &str, ty: &str) -> Result<(), AppError> {
  let key = format!("{uname}_{ty}");
  let tree = sled
    .open_tree("user_status")
    .map_err(|_e| AppError::SledError)?;
  // count the action
  increase_id(&tree, key)?;
  // record last action timestamp
  let last_post = Utc::now().timestamp();
  tree
    .insert(
      &format!("{uname}_last_{ty}"),
      IVec::from(last_post.to_be_bytes().to_vec()),
    )
    .map_err(|_e| AppError::SledError)?;

  Ok(())
}

/// get the timestamo of the action performed
pub fn get_status_timestamp(sled: &Db, key: &str) -> Result<i64, AppError> {
  let tree = sled
    .open_tree("user_status")
    .map_err(|_e| AppError::SledError)?;
  let ts_ivec = tree.get(key).unwrap_or_default().unwrap_or_default();
  let timestamp =
    i64::from_be_bytes(ts_ivec.to_vec().as_slice().try_into().unwrap_or_default());

  Ok(timestamp)
}

/// get the count of post,subscription, comment
pub fn get_status_count(tree: &Tree, key: &str) -> Result<u32, AppError> {
  let post_count_ivec = tree.get(key).unwrap_or_default().unwrap_or_default();
  let count = ivec_to_u32(&post_count_ivec);

  Ok(count)
}

/// Background job: clean the expired storing.
///
/// The keys' format: `{exp-timestamp:x}_`.
pub(crate) async fn clear_invalid_job(
  db: &Db,
  tree_name: &str,
) -> Result<(), AppError> {
  let tree = db.open_tree(tree_name).map_err(|_e| AppError::SledError)?;
  for i in tree.iter() {
    let (k, _) = i.map_err(|_e| AppError::SledError)?;
    let k_str = std::str::from_utf8(&k)?;
    let time_stamp = k_str
      .split_once('_')
      .and_then(|s| i64::from_str_radix(s.0, 16).ok());
    if let Some(time_stamp) = time_stamp {
      if time_stamp < Utc::now().timestamp() {
        tree.remove(k).map_err(|_e| AppError::SledError)?;
      }
    }
  }
  Ok(())
}

/// Update the counter and return the new one. It is contiguous if every id is used.
///
pub fn increase_id<K>(tree: &Tree, key: K) -> Result<u32, AppError>
where
  K: AsRef<[u8]>,
{
  match tree
    .update_and_fetch(key, increment)
    .map_err(|_e| AppError::SledError)?
  {
    Some(ivec) => Ok(ivec_to_u32(&ivec)),
    None => return Err(AppError::SledError),
  }
}

// =========== sled helper ============

/// convert [IVec] to u32
pub fn ivec_to_u32(iv: &IVec) -> u32 {
  u32::from_be_bytes(iv.to_vec().as_slice().try_into().unwrap_or_default())
}

/// convert `u32` to [IVec]
pub fn u32_to_ivec(number: u32) -> IVec {
  IVec::from(number.to_be_bytes().to_vec())
}

/// work for [update_and_fetch](https://docs.rs/sled/latest/sled/struct.Db.html#method.update_and_fetch):
/// increment 1.
fn increment(old: Option<&[u8]>) -> Option<Vec<u8>> {
  let number = match old {
    Some(bytes) => {
      let array: [u8; 4] = bytes.try_into().unwrap_or_default();
      let number = u32::from_be_bytes(array);
      if let Some(new) = number.checked_add(1) {
        new
      } else {
        panic!("overflow")
      }
    }
    None => 1,
  };

  Some(number.to_be_bytes().to_vec())
}

/// generate a new id with expiration time that is hex encoded.
///
/// format: "hex-timestamp_id"
///
pub fn gen_expirable_id(seconds: i64, key: Option<String>) -> String {
  let id = key.unwrap_or(nanoid!());
  let exp = Utc::now().timestamp() + seconds;
  format!("{exp:x}_{id}")
}
