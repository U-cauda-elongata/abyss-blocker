use std::{collections::VecDeque, future::Future, task::Poll};

use diesel::{dsl::*, prelude::*};
use futures::{stream::FuturesUnordered, FutureExt, Stream, StreamExt};
use oauth;
use reqwest::{header::AUTHORIZATION, StatusCode};
use std::marker::Unpin;
use structopt::StructOpt;
use tokio::sync::mpsc::unbounded_channel;

use crate::common::{connect_database, instant_to_epoch, wait_until};
use crate::query;
use crate::schema::*;
use crate::twitter;

#[derive(StructOpt)]
pub struct Opts {
    #[structopt(about = "User ID of the user to search followers of")]
    user: i64,
    #[structopt(long, about = "User ID of the user to authorize")]
    login: Option<i64>,
    #[structopt(long, about = "Path to the database", default_value = "db.sqlite3")]
    database: String,
    #[structopt(short, long, about = "Do not block the users")]
    no_block: bool,
    #[structopt(long, about = "Search from the beginning instead of resuming")]
    reset: bool,
}

pub async fn run(opts: Opts) {
    let conn = connect_database(&opts.database).unwrap();

    let http = reqwest::Client::new();

    // Authenticated user
    let auth = if let Some(id) = opts.login {
        id
    } else {
        query::default_user(&conn).expect("`--login` option or default user is required")
    };

    let credentials = query::credentials(auth, &conn)
        .unwrap_or_else(|| panic!("credentials not found for user: {}", auth));

    let endpoint: i32 = if let Some(id) = endpoints::table
        .select(endpoints::id)
        .filter(endpoints::uri.eq(twitter::FOLLOWERS_LIST))
        .get_result::<i32>(&conn)
        .optional()
        .unwrap()
    {
        id
    } else {
        insert_into(endpoints::table)
            .values(endpoints::uri.eq(twitter::FOLLOWERS_LIST))
            .execute(&conn)
            .unwrap();
        endpoints::table
            .select(endpoints::id)
            .order(endpoints::id.desc())
            .get_result::<i32>(&conn)
            .unwrap()
    };

    let mut cursor = if opts.reset {
        replace_into(user_list_cursors::table)
            .values((
                user_list_cursors::endpoint.eq(endpoint),
                user_list_cursors::authenticated_user.eq(auth),
                user_list_cursors::user.eq(opts.user),
                user_list_cursors::cursor.eq(-1),
            ))
            .execute(&conn)
            .unwrap();
        -1
    } else {
        user_list_cursors::table
            .select(user_list_cursors::cursor)
            .find((endpoint, auth, opts.user))
            .get_result::<i64>(&conn)
            .optional()
            .unwrap()
            .unwrap_or(-1)
    };

    let (tx, rx) = unbounded_channel();

    // Receive user IDs from `searcher` and block them.
    let blocker = blocker(auth, rx, &credentials, &conn, &http);

    let borrow = (&credentials, &conn, &http);
    // Search for IDs of users who block `auth` and send them to `blocker`.
    let searcher = async move {
        let (credentials, conn, http) = borrow;

        log::info!("Started searching the followers of user {}", opts.user);

        while cursor != 0 {
            let oauth::Request {
                authorization,
                data: uri,
            } = oauth::Builder::new(credentials.client(), oauth::HmacSha1)
                .token(credentials.token())
                .get(
                    twitter::FOLLOWERS_LIST,
                    &twitter::FollowersList {
                        user_id: opts.user,
                        count: 200,
                        skip_status: true,
                        include_user_entities: false,
                        cursor,
                    },
                );

            log::info!("Retrieving the follower list with cursor = {}", cursor);
            let response = http
                .get(&uri)
                .header(AUTHORIZATION, authorization)
                .send()
                .await
                .unwrap();
            let rate_limit = twitter::rate_limit(response.headers());

            match response.status() {
                StatusCode::TOO_MANY_REQUESTS => {
                    log::warn!("Got a TooManyRequest error");
                    wait_until(rate_limit.unwrap().reset + 1).await;
                    continue;
                }
                StatusCode::NOT_FOUND => {
                    log::error!("The user was not found");
                    return;
                }
                s if s.is_success() => {}
                s => {
                    log::error!("Unexpected status code: {:?}", s);
                    log::error!("Response body: {:?}", response.text().await);
                    return;
                }
            }

            let users: twitter::Users = response.json().await.unwrap();

            let blocks: Vec<_> = users
                .users
                .iter()
                .filter(|u| u.blocked_by)
                .map(|u| (blocks::source.eq(u.id), blocks::target.eq(opts.user)))
                .collect();
            if !blocks.is_empty() {
                insert_or_ignore_into(blocks::table)
                    .values(blocks)
                    .execute(conn)
                    .unwrap();
            }
            for u in &users.users {
                if u.blocked_by {
                    log::info!("User {} has blocked {}", u.id, auth);
                    if !(opts.no_block || u.blocking) {
                        tx.send(u.id)
                            .expect("receiver half has been closed unexpectedly");
                    }
                }
            }

            cursor = users.next_cursor;
            replace_into(user_list_cursors::table)
                .values((
                    user_list_cursors::endpoint.eq(endpoint),
                    user_list_cursors::authenticated_user.eq(auth),
                    user_list_cursors::user.eq(opts.user),
                    user_list_cursors::cursor.eq(cursor),
                ))
                .execute(conn)
                .unwrap();

            if let Some(rl) = rate_limit {
                if rl.remaining == 0 {
                    log::info!("Rate limit exhausted");
                    wait_until(rl.reset + 1).await;
                }
            }
        }

        log::info!("Finished searching the followers of user {}", opts.user);
    };

    futures::future::join(searcher, blocker).await;
}

fn blocker<'a>(
    auth: i64,
    mut rx: impl Stream<Item = i64> + Unpin + 'a,
    credentials: &'a crate::auth::Token,
    conn: &'a SqliteConnection,
    http: &'a reqwest::Client,
) -> impl Future<Output = ()> + 'a {
    // User IDs to block
    let mut block_queue = VecDeque::new();
    // Stores `impl Future<Output = (http_response_future, user_id_to_block)>`
    let mut blocking = FuturesUnordered::new();
    // Timer to wait for the rate limit
    let mut timer = tokio::time::delay_until(tokio::time::Instant::now());

    let future = futures::future::poll_fn(move |cx| {
        log::trace!("Polled the blocker future");

        let rx_done = loop {
            match rx.poll_next_unpin(cx) {
                Poll::Ready(Some(id)) => block_queue.push_back(id),
                Poll::Ready(None) => break true,
                Poll::Pending => break false,
            }
        };

        while let Poll::Ready(Some((result, id))) = blocking.poll_next_unpin(cx) {
            let response: reqwest::Response = match result {
                Ok(response) => response,
                Err(e) => {
                    log::error!("HTTP client error: {:?}", e);
                    block_queue.push_front(id);
                    continue;
                }
            };
            match response.status() {
                s if s.is_success() => {}
                StatusCode::NOT_FOUND => continue,
                StatusCode::TOO_MANY_REQUESTS => {
                    log::warn!("Got a TooManyRequest error");
                    block_queue.push_front(id);
                    let reset = twitter::rate_limit(response.headers()).unwrap().reset;
                    if reset > instant_to_epoch(timer.deadline()) {
                        timer = wait_until(reset + 1);
                    }
                    continue;
                }
                s => {
                    log::error!("Unexpected status code: {:?}", s);
                    // TODO: limit the number of retrials
                    block_queue.push_front(id);
                    continue;
                }
            }

            insert_or_ignore_into(blocks::table)
                .values((blocks::source.eq(auth), blocks::target.eq(id)))
                .execute(conn)
                .unwrap();
        }

        if let Poll::Ready(()) = timer.poll_unpin(cx) {
            // TODO: limit the number of requests based on `rate-limit-remaining`
            for id in block_queue.drain(..) {
                blocking.push(block(id, credentials, &http).map(move |response| (response, id)));
            }
        }

        if rx_done && block_queue.is_empty() && blocking.is_empty() {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    });

    future
}

fn block(
    user: i64,
    credentials: &crate::auth::Token,
    http: &reqwest::Client,
) -> impl Future<Output = Result<reqwest::Response, reqwest::Error>> {
    let oauth::Request {
        authorization,
        data: uri,
    } = oauth::Builder::new(credentials.client(), oauth::HmacSha1)
        .token(credentials.token())
        .get(
            twitter::BLOCKS_CREATE,
            twitter::BlocksCreate {
                user_id: user,
                include_entities: false,
                skip_status: true,
            },
        );
    http.get(&uri).header(AUTHORIZATION, authorization).send()
}
