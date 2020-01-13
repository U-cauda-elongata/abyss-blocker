use oauth;

pub struct Token {
    pub client: oauth::Credentials<Box<str>>,
    pub token: oauth::Credentials<Box<str>>,
}

impl Token {
    pub fn client(&self) -> oauth::Credentials<&str> {
        self.client.as_ref()
    }

    pub fn token(&self) -> oauth::Credentials<&str> {
        self.token.as_ref()
    }
}
