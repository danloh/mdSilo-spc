use axum_server::tls_rustls::RustlsConfig;
use bincode::config::standard;
use bincode::{Decode, Encode};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use sled::Db;
use std::fs::{read_to_string, File};
use std::io::Write;
use tracing::{error, warn};
use validator::Validate;

use crate::error::AppError;

/// App Config
pub(crate) static CONFIG: Lazy<Config> = Lazy::new(Config::load);

/// Config: set on startup.
#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Config {
  /// app name
  pub(crate) name: String,
  /// sqlite db URI
  pub(crate) db: String,
  /// sled db URI
  pub(crate) sled: String,
  /// listen address
  pub(crate) addr: String,
  /// the folder to store static icons
  pub(crate) icons_path: String,
  /// the folder to store users' avatar images
  pub(crate) avatars_path: String,
  /// the folder to store uploaded files
  pub(crate) upload_path: String,
  /// check if compress on uploading images
  pub(crate) if_compress_img: bool,
  /// customized serving static dirs: vec[(path, dir)]
  pub(crate) serve_dir: Vec<(String, String)>,
  /// secret key for encoding JWT
  pub(crate) secret_key: String,
  /// cert for enable https
  pub(crate) cert: String,
  /// key for enable https
  pub(crate) key: String,
  /// default admin username,
  pub(crate) admin_name: String,
  /// hours to clean up documents after collaboration inactivity.
  pub(crate) expiry_hours: u32,
}

impl Default for Config {
  fn default() -> Self {
    Config {
      name: "spc".into(),
      db: "spc.db".into(),
      sled: "spc.sled".into(),
      addr: "127.0.0.1:8080".into(),
      icons_path: "./data/icons".into(),
      avatars_path: "./data/avatars".into(),
      upload_path: "./data/upload".into(),
      if_compress_img: true,
      serve_dir: vec![
        ("pad".into(), "./dist".into()), // collaborative editor: index.html
        ("assets".into(), "./dist/assets".into()), // collaborative editor: js/css/wasm
      ],
      secret_key: "pL3AsG1v3AharDk3y".into(),
      cert: "".into(),
      key: "".into(),
      admin_name: "".into(),
      expiry_hours: 12,
    }
  }
}

impl Config {
  /// load config.toml, otherwise save the default config to config.toml
  fn load() -> Config {
    let cfg_file = "config.toml".to_owned();
    if let Ok(config_content) = read_to_string(cfg_file) {
      toml::from_str(&config_content).unwrap_or_default()
    } else {
      warn!("Config file not found, using default config.toml");
      let config = Config::default();
      // save the default config to file
      if let Ok(toml) = toml::to_string_pretty(&config) {
        if let Ok(mut cfg_file) = File::create("config.toml") {
          cfg_file.write_all(toml.as_bytes()).unwrap_or(());
        }
      }
      config
    }
  }

  pub(crate) async fn tls_config(&self) -> Option<RustlsConfig> {
    if let Ok(rustls_config) =
      RustlsConfig::from_pem_file(&CONFIG.cert, &CONFIG.key).await
    {
      Some(rustls_config)
    } else {
      error!("enable https failed, please check cert and key");
      None
    }
  }
}

/// SiteConfig can be set in runtime, stored in sled db.
#[derive(Serialize, Deserialize, Encode, Decode, Validate, Debug, Clone)]
pub struct SiteConfig {
  // site meta info
  #[validate(length(max = 128))]
  pub site_name: String,
  pub domain: String,
  #[validate(length(max = 512))]
  pub slogan: String,
  /// such as google site verification
  #[validate(length(max = 128))]
  pub verification: String,
  /// to customize landing page, should support markdown or HTML
  pub landing_page: String,
  /// to customize about page(terms, privacy)
  pub about_page: String,
  /// to customize style: css
  pub my_css: String,
  /// to add js plugin: js
  pub my_js: String,
  /// to add manifest.json
  pub my_manifest: String,
  // site status and setting
  pub read_only: u8,
  #[validate(range(max = 256))]
  pub title_max_length: usize,
  #[validate(range(max = 10000))]
  pub piece_max_length: usize,
  #[validate(range(max = 65535))]
  pub article_max_length: usize,
  #[validate(range(max = 65535))]
  pub comment_max_length: usize,
  #[validate(range(max = 3600))]
  pub post_interval: i64,
  #[validate(range(max = 3600))]
  pub upload_interval: i64,
  #[validate(range(max = 3600))]
  pub comment_interval: i64,
  pub per_page: usize,
  pub captcha_difficulty: String,
  pub captcha_name: String,
}

impl Default for SiteConfig {
  fn default() -> Self {
    SiteConfig {
      site_name: CONFIG.name.clone(),
      domain: "http://127.0.0.1:8080".into(),
      slogan: "Subscription, Publishing and Collaboration".into(),
      verification: "".into(),
      landing_page: "# Subscription, Publishing and collaboration \n A self-hosted online writing platform which comes as a single executable with feed subscription, publishing writing and live collaboration and many other features. \n Focus on the Markdown content, be it a blog, a knowledge base, a forum or a combination of them. Good fit for individual or small club. \n ## [Explore Here](/explore) \n\n \n ## [Collaborative Writing](/pad) \n ![](https://images.unsplash.com/photo-1675124516926-a0864dea0abd)".into(),
      about_page: "# About".into(),
      my_css: String::new(),
      my_js: String::new(),
      my_manifest: String::new(),
      read_only: 0,
      title_max_length: 100,
      piece_max_length: 512,
      article_max_length: 65_535,
      comment_max_length: 10_000,
      post_interval: 10,
      upload_interval: 10,
      comment_interval: 30,
      per_page: 42,
      captcha_difficulty: "Easy".into(),
      captcha_name: "Mila".into(),
    }
  }
}

/// get [SiteConfig]
pub fn get_site_config(db: &Db) -> Result<SiteConfig, AppError> {
  let site_config = &db.get("site_config").unwrap_or(None).unwrap_or_default();
  let (site_config, _): (SiteConfig, usize) =
    bincode::decode_from_slice(site_config, standard()).unwrap_or_default();
  Ok(site_config)
}

/// Static CSS styles file, can customize.
pub(crate) static CSS: Lazy<String> = Lazy::new(load_css);

/// load customized or default [CSS] file
fn load_css() -> String {
  let css_file = "my.css".to_string();
  if let Ok(css_content) = read_to_string(css_file) {
    css_content
  } else {
    include_str!("../static/css/styles.css").to_string()
  }
}

/// plugin js script
pub(crate) static JS: Lazy<String> = Lazy::new(load_js);

/// load customized or default [JS] file
fn load_js() -> String {
  let js_file = "my.js".to_string();
  if let Ok(js_script) = read_to_string(js_file) {
    js_script
  } else {
    String::new()
  }
}

/// Default favicon, can customize.
pub(crate) static FAVICON: Lazy<String> = Lazy::new(load_favicon);

/// load customized or default [FAVICON] file
fn load_favicon() -> String {
  let ico_file = "favicon.svg".to_string();
  if let Ok(ico_content) = read_to_string(ico_file) {
    ico_content
  } else {
    include_str!("../static/favicon.svg").to_string()
  }
}

/// Default manifest.json, can customize.
pub(crate) static MANIFEST: Lazy<String> = Lazy::new(load_manifest);

/// load customized or default [MANIFEST] file
fn load_manifest() -> String {
  let manifest_file = "manifest.json".to_string();
  if let Ok(manifest) = read_to_string(manifest_file) {
    manifest
  } else {
    include_str!("../static/manifest.json").to_string()
  }
}
