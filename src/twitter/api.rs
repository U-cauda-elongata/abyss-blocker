pub const ACCOUNT_VERIFY_CREDENTIALS: &str =
    "https://api.twitter.com/1.1/account/verify_credentials.json";
pub const BLOCKS_CREATE: &str = "https://api.twitter.com/1.1/blocks/create.json";
pub const FOLLOWERS_LIST: &str = "https://api.twitter.com/1.1/followers/list.json";

#[derive(oauth::Authorize)]
pub struct AccountVerifyCredentials {
    pub include_entities: bool,
    pub skip_status: bool,
    pub include_email: bool,
}

#[derive(oauth::Authorize)]
pub struct BlocksCreate {
    pub user_id: i64,
    pub include_entities: bool,
    pub skip_status: bool,
}

#[derive(oauth::Authorize)]
pub struct FollowersList {
    pub user_id: i64,
    pub count: u64,
    pub skip_status: bool,
    pub include_user_entities: bool,
    pub cursor: i64,
}
