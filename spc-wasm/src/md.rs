//! Preview markdown on frontend using pulldown-cmark.

use wasm_bindgen::prelude::*;
use spc_util::md2html;

/// preview markdown using pulldown-cmark
#[wasm_bindgen]
pub fn preview_md(md: &str) -> String{
  md2html(md)
}
