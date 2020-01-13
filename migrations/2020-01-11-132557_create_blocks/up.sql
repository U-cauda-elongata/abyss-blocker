CREATE TABLE blocks (
  source INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  target INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  retrieved_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
  PRIMARY KEY (source, target)
);
