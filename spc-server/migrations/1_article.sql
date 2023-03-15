CREATE TABLE articles (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  uname VARCHAR NOT NULL,
  title VARCHAR UNIQUE NOT NULL, -- must be unique for wikilink
  cover TEXT NOT NULL DEFAULT '',
  content TEXT NOT NULL,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  is_locked BOOLEAN DEFAULT FALSE,
  is_hidden BOOLEAN DEFAULT FALSE
);

CREATE TABLE pieces (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  uname VARCHAR NOT NULL,
  content TEXT NOT NULL,
  created_at INTEGER NOT NULL,
  is_hidden BOOLEAN DEFAULT FALSE
);

CREATE TABLE notes (
  id VARCHAR PRIMARY KEY,
  uname VARCHAR NOT NULL,
  title VARCHAR UNIQUE NOT NULL,
  content TEXT NOT NULL,
  folder VARCHAR NOT NULL DEFAULT 'silo',
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL
);

CREATE TABLE tags (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  tname VARCHAR UNIQUE NOT NULL,
  cover VARCHAR NOT NULL DEFAULT '',
  content TEXT NOT NULL DEFAULT ''
);

CREATE TABLE tag_entry (
  tag_id INTEGER NOT NULL,
  on_ty VARCHAR NOT NULL, -- article|piece|item...
  on_id INTEGER NOT NULL,
  UNIQUE(tag_id, on_ty, on_id)
);

CREATE TABLE article_in (
  article_id INTEGER NOT NULL,
  in_ty VARCHAR NOT NULL, -- item...
  in_id INTEGER NOT NULL,
  UNIQUE(article_id, in_ty, in_id)
);

CREATE TABLE document(
  id VARCHAR PRIMARY KEY,
  text TEXT NOT NULL,
  language VARCHAR,
  updated_at INTEGER,
  article_id INTEGER
);
