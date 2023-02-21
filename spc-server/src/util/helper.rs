//! # Helper functions

use once_cell::sync::Lazy;
use regex::Regex;

pub use spc_util::{extract_element, capture_element};

// let fail in test
static RE_HTTP: Lazy<Regex> = Lazy::new(|| Regex::new(r"https?://").unwrap()); 
static RE_PATH: Lazy<Regex> = Lazy::new(|| Regex::new(r"/.*").unwrap());
static RE_TAG: Lazy<Regex> = Lazy::new(|| Regex::new(r"<[^<>]+>").unwrap());

/// extract host of url
pub fn get_host(s: &str) -> String {
  let url_s = RE_HTTP.replace_all(s, "");
  let url_p = RE_PATH.replace_all(&url_s, "");
  let host = url_p.replace("www.", "");
  host
}

/// replace html tag
pub fn rm_html_tag(s: &str) -> String {
  RE_TAG.replace_all(s, "").to_string()
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
        "https://mdsilo.com/arti#cl$e/irs"
      ),
      String::from("mdsilo.com")
    );
  }

  #[test]
  fn test_rm_html_tag() {
    assert_eq!(
      rm_html_tag(r#"<p class="byline"> By <span class="name">Dan</span>, <a href="http://mdsilo.com/">Mind Silo</a>"#),
      String::from(" By Dan, Mind Silo")
    );
  }
}
