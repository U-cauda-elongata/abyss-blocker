CREATE TABLE default_user (
  id INTEGER NOT NULL PRIMARY KEY,
  user INTEGER NOT NULL REFERENCES users(id) ON DELETE RESTRICT
);
