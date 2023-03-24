//! ## Writing content
//! article: longer writing with title, content...;
//! piece: short writing like a mastodon toot.

use super::{filters, into_response, PageData, QueryParams, ValidatedForm};
use crate::config::get_site_config;
use crate::db::feed::Feed;
use crate::db::sled::{
  get_status_timestamp, increase_id, store_user_status, u32_to_ivec,
};
use crate::db::user::{EIDT_PERMIT, MOD_PERMIT};
use crate::error::SsrError;
use crate::util::md::md2html;
use crate::{
  db::{
    article::{Article, Entry, Piece, QueryArticles, QueryPieces},
    tag::{Tag, TagEntry},
    user::{ClaimCan, CREATE_PERMIT, READ_PERMIT},
  },
  error::AppError,
  util::helper::extract_element,
  AppState as Ctx,
};
use askama::Template;
use axum::{
  extract::{Path, Query, State},
  response::{IntoResponse, Redirect},
};

// use axum_macros::debug_handler;
use chrono::Utc;
use serde::Deserialize;
use spc_util::capture_element;
use validator::Validate;

/// Page data: `article_form.html`
#[derive(Template)]
#[template(path = "article_form.html")]
struct ArticleFormTmpl<'a> {
  page_data: PageData<'a>,
  article: Article,
}

/// `GET /article/:id/edit` article create/edit page
///
/// if articleid is 0, then create a new article
pub(crate) async fn edit_article_page(
  State(ctx): State<Ctx>,
  Path(articleid): Path<u32>,
  check: ClaimCan<CREATE_PERMIT>,
) -> Result<impl IntoResponse, SsrError> {
  if !check.can() {
    return Err(SsrError::from(AppError::Unauthorized));
  }
  let claim = check.claim;
  let site_config = get_site_config(&ctx.sled).unwrap_or_default();
  if articleid == 0 {
    let page_data = PageData::new("New Article", &site_config, claim, false);
    let article_new_page = ArticleFormTmpl {
      page_data,
      article: Article::default(),
    };

    Ok(into_response(&article_new_page, "html"))
  } else {
    let article: Article = Article::get(&ctx, articleid).await?;

    if article.uname != claim.clone().unwrap_or_default().uname {
      return Err(SsrError::from(AppError::Unauthorized));
    }

    let page_data = PageData::new("Edit Article", &site_config, claim, false);
    let article_edit_page = ArticleFormTmpl { page_data, article };

    Ok(into_response(&article_edit_page, "html"))
  }
}

/// Form data: article create/edit form
#[derive(Deserialize, Validate)]
pub(crate) struct ArticleForm {
  #[validate(length(min = 1, max = 256))]
  title: String,
  #[validate(length(max = 512))]
  cover: String,
  #[validate(length(min = 1, max = 65535))]
  content: String,
}

/// `POST /article/:id/edit` article create/edit page
///
/// if id is 0, then create a new article
pub(crate) async fn edit_article_form(
  State(ctx): State<Ctx>,
  Path(articleid): Path<u32>,
  check: ClaimCan<CREATE_PERMIT>,
  ValidatedForm(form): ValidatedForm<ArticleForm>,
) -> Result<impl IntoResponse, SsrError> {
  if !check.can() {
    return Err(AppError::NoPermission.into());
  }
  let site_config = get_site_config(&ctx.sled).unwrap_or_default();
  // check input length
  let title = form.title.trim();
  let content = form.content;
  if content.len() > site_config.article_max_length
    || title.len() > site_config.title_max_length
  {
    return Err(AppError::InvalidInput.into());
  }

  let claim = check.claim.unwrap_or_default();
  let uname = claim.clone().uname;
  // check post interval
  let now = Utc::now().timestamp();
  let last_post =
    get_status_timestamp(&ctx.sled, &format!("{uname}_last_post")).unwrap_or(0);
  if now - last_post < site_config.post_interval {
    return Err(AppError::WriteInterval.into());
  }

  let (created_at, updated_at, old_title) = if articleid > 0 {
    let old_article = Article::get(&ctx, articleid).await?;
    if old_article.uname != uname && !claim.can(EIDT_PERMIT) {
      return Err(AppError::NoPermission.into());
    }

    (old_article.created_at, now, old_article.title)
  } else {
    (now, now, String::new())
  };

  // process wikilink, work with db to get linked article id
  let wikilinks = capture_element(&content, "");
  let mut includes: Vec<u32> = Vec::new();
  for link in &wikilinks {
    let linked_title = link.replace("[", "").replace("]", "");
    if linked_title.trim().is_empty() {
      continue;
    }
    let tar_title = linked_title.split_once("|").and_then(|s| {
      let tar = s.1.trim();
      if tar.len() > 0 { Some(tar) } else { None }
    })
    .unwrap_or(linked_title.trim());
    // get article
    if let Ok(link_article) = Article::get_by_id_or_title(&ctx, tar_title).await {
      includes.push(link_article.id);
    }
  }

  // extact hashtags then save 
  let hashtags = extract_element(&content, "", "#");

  // save article to db
  let article = Article {
    id: articleid,
    uname: uname.clone(),
    title: title.to_string(),
    cover: form.cover,
    content,
    created_at,
    updated_at,
    is_locked: false,
    is_hidden: false,
  };
  let new_article = article.new(&ctx).await?;

  // record action: post new article
  if articleid == 0 {
    store_user_status(&ctx.sled, &uname, "post").unwrap_or(());
  }

  // if change title, need to update the wikilinks. who link to me
  if articleid > 0 && old_title.trim() != title {
    tokio::spawn(
      update_wikilinks(ctx.clone(), articleid, old_title, title.to_string())
    );
  }

  // save tags
  TagEntry::tag(&ctx, hashtags, "article", new_article.id).await?;
  // save backlinks, link to who
  for id in includes {
    Article::include(&ctx, id, "article", new_article.id).await.unwrap_or(0);
  }

  let target = format!("/article/{}/view", new_article.id);
  Ok(Redirect::to(&target))
}

async fn update_wikilinks(ctx: Ctx, id: u32, old_title: String, new_title: String) {
  Article::update_wikilinks(&ctx, id, &old_title, &new_title)
    .await
    .unwrap_or(0);
}

/// Page data: `article.html`
#[derive(Template)]
#[template(path = "article.html", escape = "none")]
struct ArticleViewTmpl<'a> {
  page_data: PageData<'a>,
  article: Article,
  pageview: u32,
  is_author: bool,
}

/// `GET /article/:id/view` Article page
pub(crate) async fn article_view(
  State(ctx): State<Ctx>,
  Path(articleid): Path<u32>,
  check: ClaimCan<READ_PERMIT>,
) -> Result<impl IntoResponse, SsrError> {
  let claim = check.claim;
  let site_config = get_site_config(&ctx.sled).unwrap_or_default();
  let uname = claim.clone().unwrap_or_default().uname;
  let article: Article = Article::get(&ctx, articleid).await?;
  // let user: User = User::get(&ctx, &article.uname).await?;
  // let author = article.uname.clone();
  let is_author = uname == article.uname;
  let content = article.content;

  let content = md2html(&content, "articlepage", "tag");
  let page_title = format!("{}", article.title);

  let article_view = Article { content, ..article };

  let pageview: u32 = increase_id(
    &ctx
      .sled
      .open_tree("article_pageviews")
      .map_err(|_| SsrError::from(AppError::SledError))?,
    u32_to_ivec(articleid),
  )
  .unwrap_or(1);

  let page_data = PageData::new(&page_title, &site_config, claim, false);
  let article_page = ArticleViewTmpl {
    page_data,
    article: article_view,
    pageview,
    is_author,
  };

  Ok(into_response(&article_page, "html"))
}

/// Page data: `article_not_found.html`
#[derive(Template)]
#[template(path = "article_not_found.html")]
struct ArticleNotFoundTmpl<'a> {
  page_data: PageData<'a>,
  title: &'a str,
}

/// `GET /article/:encoded_title` Article page
pub(crate) async fn view_article_by_title(
  State(ctx): State<Ctx>,
  Path(title): Path<String>,
  check: ClaimCan<READ_PERMIT>,
) -> Result<impl IntoResponse, SsrError> {
  let claim = check.claim;
  let site_config = get_site_config(&ctx.sled).unwrap_or_default();
  let uname = claim.clone().unwrap_or_default().uname;
  let decoded_title = urlencoding::decode(&title).unwrap_or_default();
  let page_data = PageData::new(decoded_title.as_ref(), &site_config, claim, false);

  if let Ok(article) = 
    Article::get_by_id_or_title(&ctx, decoded_title.as_ref()).await 
  {
    // let user: User = User::get(&ctx, &article.uname).await?;
    // let author = article.uname.clone();
    let is_author = uname == article.uname;
    let content = article.content;
    
    let content = md2html(&content, "articlepage", "tag");

    let article_view = Article { content, ..article };

    let pageview: u32 = increase_id(
      &ctx
        .sled
        .open_tree("article_pageviews")
        .map_err(|_| SsrError::from(AppError::SledError))?,
      u32_to_ivec(article.id),
    )
    .unwrap_or(1);

    let article_page = ArticleViewTmpl {
      page_data,
      article: article_view,
      pageview,
      is_author,
    };

    Ok(into_response(&article_page, "html"))
  } else {
    let not_found_page = ArticleNotFoundTmpl {
      page_data,
      title: decoded_title.as_ref(),
    };

    Ok(into_response(&not_found_page, "html"))
  }
}

/// Generate collaboration on when click only. 
/// `GET /article/:id/collaboration`
pub(crate) async fn gen_collaboration_link(
  State(ctx): State<Ctx>,
  Path(articleid): Path<u32>,
  check: ClaimCan<CREATE_PERMIT>,
) -> Result<impl IntoResponse, SsrError> {
  if !check.can() {
    return Err(SsrError::from(AppError::Unauthorized));
  }
  
  let uname = check.claim.unwrap_or_default().uname;
  let collaborative_link = Article::gen_pad_link(&ctx, articleid, &uname).await?;

  Ok(Redirect::to(&collaborative_link))
}

/// `GET /article/:id/delete` delete article
pub(crate) async fn article_delete(
  State(ctx): State<Ctx>,
  Path(articleid): Path<u32>,
  check: ClaimCan<CREATE_PERMIT>,
) -> Result<impl IntoResponse, SsrError> {
  if !check.can() {
    return Err(SsrError::from(AppError::Unauthorized));
  }
  let claim = check.claim.unwrap_or_default();
  let uname = claim.clone().uname;

  // check uname matched
  let article: Article = Article::get(&ctx, articleid).await?;
  if article.uname == uname || claim.can(EIDT_PERMIT) {
    Article::del(&ctx, articleid).await?;
  } else {
    return Err(AppError::NoPermission.into());
  }

  Ok(Redirect::to("/explore"))
}

/// Form data: `/new_piece` new piece.
#[derive(Deserialize, Validate)]
pub(crate) struct PieceForm {
  #[validate(length(min = 1, max = 1000))]
  content: String,
}

/// `POST /new_piece`
pub(crate) async fn new_piece_form(
  State(ctx): State<Ctx>,
  check: ClaimCan<CREATE_PERMIT>,
  ValidatedForm(input): ValidatedForm<PieceForm>,
) -> Result<impl IntoResponse, SsrError> {
  if !check.can() {
    return Err(SsrError::from(AppError::Unauthorized));
  }

  let site_config = get_site_config(&ctx.sled).unwrap_or_default();
  // check content length
  let content = input.content;
  if content.len() > site_config.piece_max_length {
    return Err(AppError::InvalidInput.into());
  }

  let claim = check.claim;
  let uname = claim.unwrap_or_default().uname;
  // check post interval
  let created_at = Utc::now().timestamp();
  let last_post =
    get_status_timestamp(&ctx.sled, &format!("{uname}_last_post")).unwrap_or(0);
  if created_at - last_post < site_config.post_interval {
    return Err(AppError::WriteInterval.into());
  }

  // Process tags
  let hashtags = extract_element(&content, "", "#");

  let piece = Piece {
    id: 0,
    uname: uname.clone(),
    content,
    created_at,
    is_hidden: false,
  };

  let new_piece = piece.new(&ctx).await?;

  // Save hashtags
  TagEntry::tag(&ctx, hashtags, "piece", new_piece.id).await?;

  // record action: post new piece
  store_user_status(&ctx.sled, &uname, "post").unwrap_or(());

  Ok(Redirect::to("/explore"))
}

/// `GET /piece/:id/delete` delete piece
pub(crate) async fn piece_delete(
  State(ctx): State<Ctx>,
  Path(id): Path<u32>,
  check: ClaimCan<CREATE_PERMIT>,
) -> Result<impl IntoResponse, SsrError> {
  if !check.can() {
    return Err(SsrError::from(AppError::NoPermission));
  }
  let claim = check.claim.unwrap_or_default();
  let uname = claim.clone().uname;

  // check uid matched
  let piece: Piece = Piece::get(&ctx, id).await?;
  if piece.uname == uname || claim.can(EIDT_PERMIT) {
    Piece::del(&ctx, id).await?;
  } else {
    return Err(AppError::NoPermission.into());
  }

  Ok(Redirect::to("/explore"))
}

/// `GET /tag/:id/delete` delete article
pub(crate) async fn tag_delete(
  State(ctx): State<Ctx>,
  Path(tagid): Path<u32>,
  check: ClaimCan<MOD_PERMIT>,
) -> Result<impl IntoResponse, SsrError> {
  if !check.can() {
    return Err(AppError::NoPermission.into());
  }

  Tag::del(&ctx, tagid).await?;

  Ok(Redirect::to("/explore"))
}

#[derive(Template)]
#[template(path = "explore.html")]
struct ExploreTmpl<'a> {
  page_data: PageData<'a>,
  entries: Vec<Entry>,
  tab: &'a str,
  can_create: bool,
  page: i64,
}

/// `GET /explore` explore page
pub(crate) async fn explore_page(
  State(ctx): State<Ctx>,
  Query(params): Query<QueryParams>,
  check: ClaimCan<READ_PERMIT>,
) -> Result<impl IntoResponse, SsrError> {
  let claim = check.claim;
  let site_config = get_site_config(&ctx.sled).unwrap_or_default();
  let can_create = claim.clone().unwrap_or_default().can(CREATE_PERMIT);

  let ord = params.ord.unwrap_or(String::from("desc"));
  let page = params.page.unwrap_or(1);
  let perpage = params.perpage.unwrap_or(42);
  let tab = params.tab.unwrap_or(String::from("posts"));

  let entries: Vec<Entry> = match tab.trim() {
    "posts" => {
      let article_list = QueryArticles::Index(ord.clone(), perpage, page)
        .get(&ctx)
        .await?
        .0;
      let piece_list = QueryPieces::Index(ord, perpage, page).get(&ctx).await?.0;

      let mut posts: Vec<Entry> = article_list
        .into_iter()
        .map(|a| a.into())
        .into_iter()
        .chain(piece_list.into_iter().map(|p| p.into()).into_iter())
        .collect();
      // sort per created_at
      posts.sort_by(|a, b| b.created_at.cmp(&a.created_at));
      posts
    }
    "tags" => {
      let tag_list = Tag::get_list(&ctx, &ord, perpage, page).await?;
      tag_list.into_iter().map(|t| t.into()).collect()
    }
    "feeds" => {
      let feeds = Feed::get_list(&ctx, perpage, page, true).await?;
      feeds.into_iter().map(|t| t.into()).collect()
    }
    _ => vec![],
  };

  let page_data = PageData::new("Exlpore", &site_config, claim, false);
  let explore_page = ExploreTmpl {
    page_data,
    entries,
    tab: tab.trim(),
    can_create,
    page,
  };

  Ok(into_response(&explore_page, "html"))
}

#[derive(Template)]
#[template(path = "tag.html")]
struct TagTmpl<'a> {
  page_data: PageData<'a>,
  tag: Tag,
  entries: Vec<Entry>,
  page: i64,
}

/// `GET tag/:tname` tag page
pub(crate) async fn tag_page(
  State(ctx): State<Ctx>,
  Path(tname): Path<String>,
  Query(params): Query<QueryParams>,
  check: ClaimCan<READ_PERMIT>,
) -> Result<impl IntoResponse, SsrError> {
  let site_config = get_site_config(&ctx.sled).unwrap_or_default();
  let claim = check.claim;

  // let ord = params.ord.unwrap_or(String::from("desc"));
  let page = params.page.unwrap_or(1);
  let perpage = params.perpage.unwrap_or(42);

  let tag = Tag::get(&ctx, &tname).await?;

  let article_list = QueryArticles::Tag(tname.clone(), perpage, page)
    .get(&ctx)
    .await?
    .0;
  let piece_list = QueryPieces::Tag(tname.clone(), perpage, page)
    .get(&ctx)
    .await?
    .0;

  let mut entries: Vec<Entry> = article_list
    .into_iter()
    .map(|a| a.into())
    .into_iter()
    .chain(piece_list.into_iter().map(|p| p.into()).into_iter())
    .collect();
  // sort per created_at
  entries.sort_by(|a, b| b.created_at.cmp(&a.created_at));

  let page_title = format!("Tag: {}", tname);
  let page_data = PageData::new(&page_title, &site_config, claim, false);
  let tag_page = TagTmpl {
    page_data,
    tag,
    entries,
    page,
  };

  Ok(into_response(&tag_page, "html"))
}
