use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Users {
    pub users: Vec<User>,
    pub next_cursor: i64,
    pub previous_cursor: i64,
}

#[derive(Debug, Deserialize)]
pub struct User {
    pub id: i64,
    /// An undocumented attribute that indicates whether the authenticated user is blocks this user.
    pub blocking: bool,
    /// An undocumented attribute that indicates whether the authenticated user is blocked by this user.
    pub blocked_by: bool,
}
