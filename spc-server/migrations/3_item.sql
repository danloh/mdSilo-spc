CREATE TABLE items (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  title VARCHAR UNIQUE NOT NULL,
  cover TEXT NOT NULL DEFAULT '',
  content TEXT NOT NULL DEFAULT '',
  category VARCHAR NOT NULL, -- story|book|movie|...
  uname VARCHAR NOT NULL,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  is_hidden BOOLEAN DEFAULT FALSE
);

CREATE TABLE item_attr (
  item_id INTEGER NOT NULL,
  attr_key VARCHAR NOT NULL,
  attr_icon VARCHAR NOT NULL DEFAULT '',
  attr_val VARCHAR NOT NULL,
  PRIMARY KEY (item_id, attr_key)
);

CREATE TABLE daysums (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  intro VARCHAR NOT NULL,
  sum_at INTEGER NOT NULL,
  sum_by VARCHAR NOT NULL,
  on_ty VARCHAR NOT NULL DEFAULT 'item',
  on_id INTEGER NOT NULL,
  UNIQUE(sum_at, on_ty, on_id)
);

CREATE TABLE revisions (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  content TEXT NOT NULL,
  on_ty VARCHAR NOT NULL,
  on_id INTEGER NOT NULL,
  rev_by VARCHAR NOT NULL,
  rev_at INTEGER NOT NULL,
  is_current BOOLEAN NOT NULL DEFAULT FALSE,
  UNIQUE(content, on_ty, on_id)
);
