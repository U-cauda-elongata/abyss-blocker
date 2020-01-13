use std::io::{stdin, stdout, BufRead, Write};

use diesel::{dsl::*, prelude::*};
use oauth;
use reqwest::{self, header::AUTHORIZATION};
use structopt::StructOpt;

use crate::common::connect_database;
use crate::schema::*;
use crate::twitter;

#[derive(StructOpt)]
pub struct Opts {
    #[structopt(long, about = "Path to the database", default_value = "db.sqlite3")]
    database: String,
    #[structopt(short, long, about = "Do not check validity of the credentials")]
    no_verify: bool,
}

pub async fn run(opts: Opts) {
    let conn = connect_database(&opts.database).unwrap();

    let stdin_isatty = atty::is(atty::Stream::Stdin);
    let stdin = stdin();
    let mut lines = stdin.lock().lines();
    let stdout = stdout();
    let mut stdout = stdout.lock();

    macro_rules! gets {
        () => {
            match lines.next() {
                Some(s) => s.unwrap(),
                None => {
                    eprintln!("Unexpected end of input");
                    return;
                }
            }
        };
    }

    macro_rules! prompt {
        ($($args:tt)*) => {
            if stdin_isatty {
                write!(stdout, $($args)*).unwrap();
                stdout.flush().unwrap();
            }
        };
    }

    prompt!("Consumer key: ");
    let consumer_key = gets!();

    prompt!("Consumer secret: ");
    let consumer_secret = gets!();

    prompt!("Access token: ");
    let access_token = gets!();
    let user: i64 = if let Ok(id) = access_token.split('-').next().unwrap().parse() {
        id
    } else {
        eprintln!("Unrecognized token format");
        return;
    };

    prompt!("Access token secret: ");
    let token_secret = gets!();

    if !opts.no_verify {
        write!(stdout, "Verifying the credentials... ").unwrap();

        let client = oauth::Credentials {
            identifier: &*consumer_key,
            secret: &*consumer_secret,
        };
        let token = oauth::Credentials {
            identifier: &*access_token,
            secret: &*token_secret,
        };

        let oauth::Request {
            authorization,
            data: uri,
        } = oauth::Builder::new(client, oauth::HmacSha1)
            .token(token)
            .get(
                twitter::ACCOUNT_VERIFY_CREDENTIALS,
                twitter::AccountVerifyCredentials {
                    include_entities: false,
                    skip_status: false,
                    include_email: false,
                },
            );
        let response = reqwest::Client::new()
            .get(&uri)
            .header(AUTHORIZATION, authorization)
            .send()
            .await
            .unwrap();

        if !response.status().is_success() {
            writeln!(stdout, "").unwrap();
            eprintln!("Unable to verify the credentials");
            return;
        }

        writeln!(stdout, "Success").unwrap();
    }

    insert_or_ignore_into(users::table)
        .values(users::id.eq(user))
        .execute(&conn)
        .unwrap();
    insert_or_ignore_into(credentials::table)
        .values((
            credentials::identifier.eq(&consumer_key),
            credentials::secret.eq(&consumer_secret),
        ))
        .execute(&conn)
        .unwrap();
    insert_or_ignore_into(credentials::table)
        .values((
            credentials::identifier.eq(&access_token),
            credentials::secret.eq(&token_secret),
        ))
        .execute(&conn)
        .unwrap();
    let client: i32 = credentials::table
        .select(credentials::id)
        .filter(credentials::identifier.eq(&consumer_key))
        .get_result(&conn)
        .unwrap();
    let token: i32 = credentials::table
        .select(credentials::id)
        .filter(credentials::identifier.eq(&access_token))
        .get_result(&conn)
        .unwrap();
    insert_or_ignore_into(tokens::table)
        .values((
            tokens::client.eq(client),
            tokens::token.eq(token),
            tokens::user.eq(user),
        ))
        .execute(&conn)
        .unwrap();
}
