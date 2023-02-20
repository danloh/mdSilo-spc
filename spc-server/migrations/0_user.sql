CREATE TABLE users (
  username VARCHAR PRIMARY KEY, -- uri friendly
  password_hash VARCHAR NOT NULL,
  nickname VARCHAR NOT NULL DEFAULT '',
  join_at INTEGER NOT NULL,
  permission INTEGER NOT NULL DEFAULT 3,
  karma INTEGER NOT NULL DEFAULT 1,
  about TEXT NOT NULL DEFAULT ''
);
