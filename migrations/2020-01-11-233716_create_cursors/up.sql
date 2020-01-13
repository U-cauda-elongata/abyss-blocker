CREATE TABLE user_list_cursors (
  endpoint INTEGER NOT NULL REFERENCES endpoints(id) ON DELETE RESTRICT ON UPDATE CASCADE,
  authenticated_user INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  user INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  cursor INTEGER NOT NULL,
  PRIMARY KEY (endpoint, authenticated_user, user)
);
