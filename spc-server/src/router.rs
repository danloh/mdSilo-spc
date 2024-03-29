//! App routers
//!
use crate::{
  config::CONFIG,
  pad::{ws_server, WsConfig},
  api::{
    feed::{
      fetch_feed, add_channel, get_sub_channels, get_feeds, 
      get_feeds_by_channel, star_feed, unstar_feed, read_feed, 
      get_read_feeds, get_star_feeds, check_star, check_read, 
      get_audio_feeds, del_subscription, get_html_proxy
    },
    note::{
      new_note, get_note, get_notes, get_notes_by_folder, get_folders,
      move_note, del_note, update_note, rename_note
    }
  },
  ssr::{
    admin::{
      mod_user, save_site_config, site_config_view, user_list_page, 
      channel_list_page, mod_channel
    },
    article::{
      article_delete, article_view, gen_collaboration_link, 
      edit_article_form, edit_article_page, explore_page, 
      new_piece_form, piece_delete, tag_delete, tag_page, view_article_by_title, 
    },
    auth::{
      change_psw_form, change_psw_page, sign_out, signin_form, signin_page,
      signup_form, signup_page,
    },
    feed::{
      channel_add_form, channel_add_page, channel_preload_form,
      channel_preload_page, del_channel, feed_reader_page, mod_subscription,
      unsubscribe, refresh_scribled_feeds,
    },
    handler_404,
    home::{
      about_page, health_check, home_page, serve_dir, 
      static_js, static_style, favicon, manifest,
    },
    upload::{upload_file, upload_page},
    user::{profile_page, user_setting_form, user_setting_view},
  },
  AppState,
};
use axum::{
  error_handling::HandleErrorLayer,
  extract::DefaultBodyLimit,
  handler::Handler, // trait for .layer
  http::StatusCode,
  routing::{get, post},
  BoxError,
  Router, response::Redirect,
};
use std::time::Duration;
use tower::{timeout::TimeoutLayer, ServiceBuilder};
use tower_http::{
  compression::CompressionLayer,
  trace::{DefaultMakeSpan, TraceLayer},
};
use tracing::{info, Level};

const UPLOAD_LIMIT: usize = 20 * 1024 * 1024;

pub async fn router(ctx: AppState) -> Router {
  let middleware_stack = ServiceBuilder::new()
    .layer(HandleErrorLayer::new(|_: BoxError| async {
      StatusCode::REQUEST_TIMEOUT
    }))
    .layer(TimeoutLayer::new(Duration::from_secs(10)))
    .layer(CompressionLayer::new())
    .layer(
      TraceLayer::new_for_http()
        .make_span_with(DefaultMakeSpan::new().level(Level::INFO)),
    );

  let ws_config = WsConfig {
    expiry_hours: CONFIG.expiry_hours,
    pool: ctx.pool.clone(),
  };

  let ws_route = ws_server(ws_config).await;

  let router_api = Router::new()
    // feed reader
    .route("/api/fetch_feed", get(fetch_feed))
    .route("/api/add_channel", post(add_channel))
    .route("/api/get_channels", get(get_sub_channels))
    .route("/api/del_subscription", get(del_subscription))
    .route("/api/get_feeds", get(get_feeds))
    .route("/api/get_channel_feeds", get(get_feeds_by_channel))
    .route("/api/check_star", get(check_star))
    .route("/api/star_feed", get(star_feed))
    .route("/api/unstar_feed", get(unstar_feed))
    .route("/api/check_read", get(check_read))
    .route("/api/read_feed", get(read_feed))
    .route("/api/get_read_feeds", get(get_read_feeds))
    .route("/api/get_star_feeds", get(get_star_feeds))
    .route("/api/get_audio_feeds", get(get_audio_feeds))
    // note
    .route("/api/new_note", post(new_note))
    .route("/api/update_note", post(update_note))
    .route("/api/rename_note", post(rename_note))
    .route("/api/get_note/:id", get(get_note))
    .route("/api/get_notes", get(get_notes))
    .route("/api/get_folder_notes/:folder", get(get_notes_by_folder))
    .route("/api/get_folders", get(get_folders))
    .route("/api/move_note/:id/:folder", get(move_note))
    .route("/api/del_note/:id", get(del_note))
    .route("/proxy/gethtml", get(get_html_proxy))
    .with_state(ctx.clone());

  let router_ssr = Router::new()
    .route("/", get(home_page))
    .route("/explore", get(explore_page))
    .route("/about", get(about_page))
    // auth and user
    .route("/signin", get(signin_page).post(signin_form))
    .route("/signup", get(signup_page).post(signup_form))
    .route("/signout", get(sign_out))
    .route(
      "/change_password",
      get(change_psw_page).post(change_psw_form),
    )
    .route("/user/:uname", get(profile_page))
    .route(
      "/user/:uname/setting",
      get(user_setting_view).post(user_setting_form),
    )
    // content
    .route("/articlepage/:title", get(view_article_by_title))
    .route("/article/:id/view", get(article_view))
    .route("/article/:id/collaboration", get(gen_collaboration_link))
    .route(
      "/article/:id/edit",
      get(edit_article_page).post(edit_article_form),
    )
    .route("/new", get(|| async { Redirect::permanent("/article/0/edit") }))
    .route("/article/:id/delete", get(article_delete))
    .route("/new_piece", post(new_piece_form))
    .route("/piece/:id/delete", get(piece_delete))
    .route("/tag/:tag", get(tag_page))
    .route("/delete_tag/:id", get(tag_delete))
    // admin
    .route("/admin/user_list", get(user_list_page))
    .route("/admin/:uname/mod/:permission", get(mod_user))
    .route("/admin/channel_list", get(channel_list_page))
    .route("/admin/mod_channel/:hidden", get(mod_channel))
    .route("/siteconfig", get(site_config_view).post(save_site_config))
    // upload and media center
    .route(
      "/upload",
      get(upload_page).post(upload_file.layer(DefaultBodyLimit::max(UPLOAD_LIMIT))),
    )
    // feed
    .route(
      "/channel_preload",
      get(channel_preload_page).post(channel_preload_form),
    )
    .route("/channel_add", get(channel_add_page).post(channel_add_form))
    .route("/channel_del/:id", get(del_channel))
    .route("/unsubscribe/:id", get(unsubscribe))
    .route("/mod_subscription/:id", get(mod_subscription))
    .route("/feed_reader", get(feed_reader_page))
    .route("/refresh_scribled_feeds", get(refresh_scribled_feeds))
    .with_state(ctx);

  let mut router_static = Router::new()
    .route("/health_check", get(health_check))
    .route("/static/style.css", get(static_style))
    .route("/static/script.js", get(static_js))
    .route("/static/favicon.svg", get(favicon))
    .route("/static/manifest.json", get(manifest))
    .nest_service("/static/icon", serve_dir(&CONFIG.icons_path).await)
    .nest_service("/static/avatars", serve_dir(&CONFIG.avatars_path).await)
    .nest_service("/static/upload", serve_dir(&CONFIG.upload_path).await);

  for (path, dir) in &CONFIG.serve_dir {
    let path = format!("/{path}");
    info!("serve dir: {} -> {}", path, dir);
    router_static = router_static.nest_service(&path, serve_dir(dir).await);
  }

  let app = router_static.merge(router_ssr).merge(ws_route).merge(router_api);
  app.layer(middleware_stack).fallback(handler_404)
}
