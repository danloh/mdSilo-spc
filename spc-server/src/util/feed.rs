//! ## Process Feed 
//! Support RSS and Atom

use axum::body::Bytes;
use chrono::{DateTime, Utc};

use crate::db::feed::{Channel, Feed};

// # process feed
//
// 0- get content
async fn get_content(url: &str) -> Option<Bytes> {
  let client = reqwest::Client::builder().build();

  let response = match client {
    Ok(cl) => cl.get(url).send().await,
    Err(_e) => return None,
  };

  match response {
    Ok(response) => match response.status() {
      reqwest::StatusCode::OK => {
        let content = match response.bytes().await {
          Ok(ctn) => ctn,
          Err(_e) => return None,
        };
        Some(content)
      }
      _status => None,
    },
    Err(_e) => None,
  }
}

// 1.1- fetch: rss typed
async fn process_rss(
  url: &str, 
  ty: Option<String>, 
  title: Option<String>,
) -> Option<(Channel, Vec<Feed>)> {
  if let Some(content) = get_content(url).await {
    match rss::Channel::read_from(&content[..]).map(|channel| channel) {
      Ok(channel) => {
        let rss_channel = Channel {
          link: String::from(url),
          title: title.unwrap_or(channel.title),
          intro: channel.description,
          ty: ty.unwrap_or(String::from("rss")),
        };

        let mut feeds: Vec<Feed> = vec![];
        for item in channel.items {
          let published = if let Some(pub_date) = item.pub_date {
            DateTime::parse_from_rfc2822(&pub_date)
              .unwrap_or_default()
              .timestamp()
          } else {
            Utc::now().timestamp()
          };
          let intro = item.description.unwrap_or_default();
          // get audio 
          let enclosure = item.enclosure.clone().unwrap_or_default();
          let audio_url = if enclosure.mime_type.starts_with("audio/") {
            enclosure.url
          } else {
            String::new()
          };

          let feed = Feed {
            id: 0,
            title: item.title.unwrap_or_default(),
            channel_link: url.to_string(),
            feed_url: item.link.unwrap_or_default(),
            audio_url,
            intro: intro.clone(),
            published,
            content: item.content.unwrap_or(intro),
            author: item.author.unwrap_or_default(),
            img: String::from(""),
          };
          feeds.push(feed);
        }
        Some((rss_channel, feeds))
      }
      Err(_e) => None,
    }
  } else {
    None
  }
}

// 1.2- fetch: atom typed
async fn process_atom(
  url: &str, 
  title: Option<String>,
) -> Option<(Channel, Vec<Feed>)> {
  if let Some(content) = get_content(url).await {
    match atom_syndication::Feed::read_from(&content[..]) {
      Ok(atom) => {
        let channel = Channel {
          link: String::from(url),
          title: title.unwrap_or(atom.title.to_string()),
          intro: atom.subtitle.unwrap_or_default().to_string(),
          ty: String::from("atom"),
        };

        let mut feeds: Vec<Feed> = vec![];
        for item in atom.entries {
          let feed_url = if let Some(link) = item.links.first() {
            link.to_owned().href
          } else {
            String::new()
          };
          let intro = item.summary.unwrap_or_default().to_string();
          let feed = Feed {
            id: 0,
            title: item.title.to_string(),
            channel_link: url.to_string(),
            feed_url,
            audio_url: String::from(""),
            intro: intro.clone(),
            published: item.updated.timestamp(),
            content: item.content.unwrap_or_default().value.unwrap_or(intro),
            author: String::from(""),
            img: String::from(""),
          };
          feeds.push(feed);
        }
        Some((channel, feeds))
      }
      Err(_) => {
        return None;
      }
    }
  } else {
    None
  }
}

// 1: get channel and feeds
pub async fn process_feed(
  url: &str, 
  ty: Option<String>,
  title: Option<String>
) -> Option<(Channel, Vec<Feed>)> {
  match process_rss(url, ty, title.clone()).await {
    Some(ch) => Some(ch),
    None => process_atom(url, title).await,
  }
}
