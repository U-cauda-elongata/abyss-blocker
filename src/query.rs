use diesel::deserialize::FromSql;
use diesel::prelude::*;
use diesel::sql_types::{BigInt, Text};
use diesel::{sql_query, Connection};

use crate::auth::Token;
use crate::schema::default_user;

pub fn credentials<Conn>(user: i64, conn: &Conn) -> Option<Token>
where
    Conn: Connection,
    String: FromSql<Text, Conn::Backend>,
{
    #[derive(QueryableByName)]
    struct QueriedToken {
        #[sql_type = "Text"]
        client_identifier: String,
        #[sql_type = "Text"]
        client_secret: String,
        #[sql_type = "Text"]
        token_identifier: String,
        #[sql_type = "Text"]
        token_secret: String,
    }

    sql_query(
        "SELECT \
         client.identifier AS client_identifier, \
         client.secret AS client_secret, \
         token.identifier AS token_identifier, \
         token.secret AS token_secret \
         FROM tokens \
         JOIN credentials AS client ON tokens.client = client.id \
         JOIN credentials AS token ON tokens.token = token.id \
         WHERE tokens.user = ? \
         LIMIT 1",
    )
    .bind::<BigInt, _>(user)
    .get_result::<QueriedToken>(conn)
    .optional()
    .unwrap()
    .map(|t| {
        let client = oauth::Credentials {
            identifier: t.client_identifier.into(),
            secret: t.client_secret.into(),
        };
        let token = oauth::Credentials {
            identifier: t.token_identifier.into(),
            secret: t.token_secret.into(),
        };
        Token { client, token }
    })
}

pub fn default_user<Conn>(conn: &Conn) -> Option<i64>
where
    Conn: Connection,
    i64: FromSql<BigInt, Conn::Backend>,
{
    default_user::table
        .order(default_user::id.desc())
        .select(default_user::user)
        .limit(1)
        .get_result(conn)
        .optional()
        .unwrap()
}
