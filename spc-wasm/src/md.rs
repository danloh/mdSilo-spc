//! Preview markdown on frontend using pulldown-cmark.

use wasm_bindgen::prelude::*;
use spc_util::{md2html, gen_expirable_id, get_utc_now};

/// preview markdown using pulldown-cmark
#[wasm_bindgen]
pub fn preview_md(md: &str) -> String{
  md2html(md)
}

/// gen id
#[wasm_bindgen]
pub fn gen_id(seconds: i64, key: &str) -> String{
  gen_expirable_id(seconds, key)
}

/// gen current timestamp
#[wasm_bindgen]
pub fn get_now() -> i64{
  get_utc_now()
}
