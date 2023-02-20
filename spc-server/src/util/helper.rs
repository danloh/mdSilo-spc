//! # Helper functions

use once_cell::sync::Lazy;
use regex::Regex;

pub use spc_util::{extract_element, capture_element};

static RE_S: Lazy<Regex> = Lazy::new(|| Regex::new(r"https?://").unwrap()); // let fail in test
static RE_P: Lazy<Regex> = Lazy::new(|| Regex::new(r"/.*").unwrap()); // let fail in test

/// extract host of url
pub fn get_host(s: &str) -> String {
  let url_s = RE_S.replace_all(s, "");
  let url_p = RE_P.replace_all(&url_s, "");
  let host = url_p.replace("www.", "");
  host
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_get_host() {
    assert_eq!(
      get_host("https://www.propublica.org/article/irs-files-taxes-wash-sales-goldman-sachs"),
      String::from("propublica.org")
    );
    assert_eq!(
      get_host(
        "https://propublica.org/arti#cl$e/irs-files-taxes-wash-sales-goldman-sachs"
      ),
      String::from("propublica.org")
    );
  }
}
