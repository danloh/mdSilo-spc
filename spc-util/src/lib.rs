//! ## Utility helpers. 
//! can be used in spc-server and spc-wasm. 

use std::collections::BTreeSet;
use regex::Regex;
use once_cell::sync::Lazy;
use pulldown_cmark::{html, CodeBlockKind, Event, Options, Tag};
use syntect::{
  highlighting::ThemeSet, html::highlighted_html_for_string, parsing::SyntaxSet,
};

/// generate a new id with expiration time that is hex encoded.
/// format: "hex-timestamp_id"
pub fn gen_expirable_id(seconds: i64, key: &str) -> String {
  // let id = nanoid!();
  let exp = chrono::Utc::now().timestamp() + seconds;
  format!("{exp:x}_{key}")
}

/// get current timestamp(UTC)
pub fn get_utc_now() -> i64 {
  chrono::Utc::now().timestamp()
}

/// extract element from string.
/// re: regex
/// pat: char to split the extracted;
///
pub fn extract_element(input: &str, re: &str, pat: &str) -> BTreeSet<String> {
  let re_str = if re.trim().len() > 0 {
    re.trim()
  } else {
    r"[\s]+#[^\s#]+"
  };

  let extracted: BTreeSet<String> = if let Ok(reg) = Regex::new(re_str) {
    reg
      .find_iter(input)
      .map(|mat| mat.as_str().trim().replace(pat, ""))
      .filter(|s| !s.is_empty())
      .collect()
  } else {
    BTreeSet::new()
  };

  extracted
}

/// capture element from string via regex.
///
pub fn capture_element(input: &str, re: &str) -> BTreeSet<String> {
  let re_str = if re.trim().len() > 0 {
    re.trim()
  } else {
    r"\[\[[^\[\]]+\]\]"
  };

  if let Ok(reg) = Regex::new(re_str) {
    reg
      .find_iter(input)
      .map(|mat| mat.as_str().to_string())
      .collect()
  } else {
    BTreeSet::new()
  }
}

/// escape special char in html
pub fn escape_html(s: &str) -> String {
  let mut output = String::new();
  for c in s.chars() {
    match c {
      '<' => output.push_str("&lt;"),
      '>' => output.push_str("&gt;"),
      '"' => output.push_str("&quot;"),
      '&' => output.push_str("&amp;"),
      _ => output.push(c),
    }
  }
  output
}

struct SyntaxPreprocessor<'a, I: Iterator<Item = Event<'a>>> {
  parent: I,
}

impl<'a, I: Iterator<Item = Event<'a>>> SyntaxPreprocessor<'a, I> {
  /// Create a new syntax preprocessor from `parent`.
  const fn new(parent: I) -> Self {
    Self { parent }
  }
}

#[inline]
fn is_inline_latex(s: &str) -> bool {
  let s = s.as_bytes();
  s.len() > 1 && [s[0], s[s.len() - 1]] == [b'$', b'$']
}

static THEME_SET: Lazy<syntect::highlighting::ThemeSet> =
  Lazy::new(ThemeSet::load_defaults);
static SYNTAX_SET: Lazy<SyntaxSet> = Lazy::new(SyntaxSet::load_defaults_newlines);

impl<'a, I: Iterator<Item = Event<'a>>> Iterator for SyntaxPreprocessor<'a, I> {
  type Item = Event<'a>;

  #[inline]
  fn next(&mut self) -> Option<Self::Item> {
    let mut code = String::with_capacity(64);
    let lang = match self.parent.next()? {
      Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(lang))) => lang,
      Event::Code(c) if is_inline_latex(&c) => {
        return Some(Event::Html(
          latex2mathml::latex_to_mathml(
            &c[1..c.len() - 1],
            latex2mathml::DisplayStyle::Inline,
          )
          .unwrap_or_else(|e| e.to_string())
          .into(),
        ));
      }
      Event::Html(h) => {
        code.push_str(&h);
        "html".into()
      }
      other => return Some(other),
    };

    while let Some(Event::Text(text) | Event::Html(text)) = self.parent.next() {
      code.push_str(&text);
    }

    // println!(">>iter lang: {}, code: {}", lang, code);

    if lang.as_ref() == "math" {
      return Some(Event::Html(
        latex2mathml::latex_to_mathml(&code, latex2mathml::DisplayStyle::Block)
          .unwrap_or_else(|e| e.to_string())
          .into(),
      ));
    }

    if lang.as_ref() == "mermaid" {
      let mermaid_content = escape_html(&code);
      let mermaid_content = mermaid_content.replace("\r\n", "\n");
      let mermaid_code =
        format!("<pre class=\"mermaid\">{}</pre>\n\n", mermaid_content);
      return Some(Event::Html(mermaid_code.into()));
    }

    // to support raw html
    let res = if lang.as_ref() == "html" {
      return Some(Event::Html(code.into()));
    } else {
      let syntax_ref = SYNTAX_SET.find_syntax_by_name(lang.as_ref());
      let syntax = if let Some(syntax_name) = syntax_ref {
        syntax_name
      } else if let Some(syntax_ext) = SYNTAX_SET.find_syntax_by_extension("rs") {
        syntax_ext
      } else {
        return Some(Event::Html(code.into()));
      };

      let highlighted_html = highlighted_html_for_string(
        &code,
        &SYNTAX_SET,
        syntax,
        &THEME_SET.themes["InspiredGitHub"],
      );

      if let Ok(hl) = highlighted_html {
        hl
      } else {
        return Some(Event::Html(code.into()));
      }
    };

    Some(Event::Html(res.into()))
  }
}

const OPTIONS: Options = Options::all();

/// convert latex and markdown to html.
/// Inspired by [cmark-syntax](https://github.com/grego/cmark-syntax/blob/master/src/lib.rs)

// This file is part of cmark-syntax. This program comes with ABSOLUTELY NO WARRANTY;
// This is free software, and you are welcome to redistribute it under the
// conditions of the GNU General Public License version 3.0.
//
// You should have received a copy of the GNU General Public License
// along with cmark-syntax. If not, see <http://www.gnu.org/licenses/>
pub fn md2html(md: &str, wikilink_base: &str, tag_base: &str) -> String {
  let md = pre_process_md(md, wikilink_base, tag_base);
  let parser = pulldown_cmark::Parser::new_ext(&md, OPTIONS);
  let processed = SyntaxPreprocessor::new(parser);
  let mut html_output = String::with_capacity(md.len() * 2);
  html::push_html(&mut html_output, processed);
  html_output
}

/// unify the block format for math
/// maybe do more pre-process in the future 
fn pre_process_md(md: &str, wikilink_base: &str, tag_base: &str) -> String {
  let mut content = md.to_string();
  // process inline math code $math$
  let inline_maths = capture_element(&content, r"[\s]+\$[^$]+\$[\s]+");
  for inline_math in &inline_maths {
    let code = format!(" `{inline_math}` ");
    content = content.replace(inline_math, &code);
  }

  // process block math code $$math$$
  let maths = capture_element(&content, r"[\s]+\$\$[^$]+\$\$[\s]+");
  for math in &maths {
    let math_code = math.replace("$$", "");
    let math_block = format!("\n```math\n{math_code}\n```\n");
    content = content.replace(math, &math_block);
  }

  // Process tags
  let hashtags = extract_element(&content, "", "#");
  for tag in &hashtags {
    let tag_link = format!("[#{tag}](/{tag_base}/{tag})");
    content = content.replace(&format!("#{tag}"), &tag_link);
  }

  let wikilinks = capture_element(&content, "");
  for link in &wikilinks {
    let title = link.replace("[", "").replace("]", "");
    if title.trim().is_empty() {
      continue;
    }
    
    // custom wikilink title:  src_title and target_title
    let parts = title.split_once("|");
    let src_title = parts.and_then(|s| {
      let src = s.0.trim();
      if src.len() > 0 { Some(src) } else { None }
    })
    .unwrap_or(title.trim());

    let tar_title = parts.and_then(|s| {
      let tar = s.1.trim();
      if tar.len() > 0 { Some(tar) } else { None }
    })
    .unwrap_or(title.trim());
    
    let encoded_title = urlencoding::encode(tar_title);
    let wiki_link = format!("[{src_title}](/{wikilink_base}/{})", encoded_title);
    content = content.replace(link, &wiki_link);
  }

  return content;
}


/*

/// auto linkify

extern crate pulldown_cmark;
extern crate regex;

use pulldown_cmark::{CowStr, Event, LinkType, Tag};
use regex::Regex;

static URL_REGEX: &str = r#"((https?|ftp)://|www.)[^\s/$.?#].[^\s]*[^.^\s]"#;

enum LinkState {
    Open,
    Label,
    Close,
}

enum AutoLinkerState<'a> {
    Clear,
    Link(LinkState, CowStr<'a>, CowStr<'a>),
    TrailingText(CowStr<'a>),
}

pub struct AutoLinker<'a, I> {
    iter: I,
    state: AutoLinkerState<'a>,
    regex: Regex,
}

impl<'a, I> AutoLinker<'a, I> {
    pub fn new(iter: I) -> Self {
        Self {
            iter,
            state: AutoLinkerState::Clear,
            regex: Regex::new(URL_REGEX).unwrap(),
        }
    }
}

impl<'a, I> Iterator for AutoLinker<'a, I>
where
    I: Iterator<Item = Event<'a>>,
{
    type Item = Event<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let text = match std::mem::replace(&mut self.state, AutoLinkerState::Clear) {
            AutoLinkerState::Clear => match self.iter.next() {
                Some(Event::Text(text)) => text,
                x => return x,
            },
            AutoLinkerState::TrailingText(text) => text,
            AutoLinkerState::Link(link_state, link_text, trailing_text) => match link_state {
                LinkState::Open => {
                    self.state = AutoLinkerState::Link(
                        LinkState::Label,
                        link_text.clone(),
                        trailing_text.clone(),
                    );
                    return Some(Event::Start(Tag::Link(
                        LinkType::Inline,
                        link_text,
                        "".into(),
                    )));
                }
                LinkState::Label => {
                    self.state = AutoLinkerState::Link(
                        LinkState::Close,
                        link_text.clone(),
                        trailing_text.clone(),
                    );
                    return Some(Event::Text(link_text));
                }
                LinkState::Close => {
                    self.state = AutoLinkerState::TrailingText(trailing_text);
                    return Some(Event::End(Tag::Link(
                        LinkType::Inline,
                        link_text,
                        "".into(),
                    )));
                }
            },
        };

        match self.regex.find(&text) {
            Some(reg_match) => {
                let link_text = reg_match.as_str();
                let leading_text = &text.as_ref()[..reg_match.start()];
                let trailing_text = &text.as_ref()[reg_match.end()..];

                self.state = AutoLinkerState::Link(
                    LinkState::Open,
                    link_text.to_owned().into(),
                    trailing_text.to_owned().into(),
                );

                Some(Event::Text(leading_text.to_owned().into()))
            }
            None => Some(Event::Text(text)),
        }
    }
}

*/

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_extract_element() {
    assert_eq!(
      extract_element("# hello #world hola #world", "", "#"),
      BTreeSet::from(["world".to_string()])
    );
    assert_eq!(
      extract_element("#hello #世界", "", "#"),
      BTreeSet::from(["世界".to_string()])
    );
  }

  #[test]
  fn test_capture_element() {
    assert_eq!(
      capture_element("I [[just got]] and Till [[just got]].", ""),
      BTreeSet::from(["[[just got]]".to_string()])
    );
  }
}
