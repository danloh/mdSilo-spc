CREATE TABLE channels (
  link VARCHAR NOT NULL UNIQUE,
  title VARCHAR NOT NULL,
  intro VARCHAR,
  published INTEGER NOT NULL,
  ty VARCHAR NOT NULL DEFAULT 'rss',
  is_hidden BOOLEAN DEFAULT FALSE -- for mod
);

CREATE TABLE feeds (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  title VARCHAR NOT NULL,
  channel_link VARCHAR NOT NULL,
  feed_url VARCHAR NOT NULL UNIQUE,
  audio_url VARCHAR NOT NULL DEFAULT '',
  published INTEGER NOT NULL,
  intro VARCHAR,
  content VARCHAR,
  author VARCHAR,
  img VARCHAR
);

CREATE TABLE subscriptions (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  uname VARCHAR NOT NULL,
  channel_link VARCHAR NOT NULL,
  channel_title VARCHAR NOT NULL,
  is_public BOOLEAN DEFAULT FALSE,
  tags VARCHAR,
  UNIQUE(uname, channel_link)
);

CREATE TABLE feed_status (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  uname VARCHAR NOT NULL,
  feed_url VARCHAR NOT NULL,
  read_status BOOLEAN DEFAULT FALSE,
  star_status BOOLEAN DEFAULT FALSE,
  UNIQUE(uname, feed_url)
);
