#[macro_use]
extern crate diesel;

use structopt::StructOpt;

mod auth;
mod cmd;
mod common;
mod query;
mod schema;
mod twitter;

#[derive(StructOpt)]
enum Cmd {
    #[structopt(about = "Register a set of API keys to the database")]
    Authorize(cmd::authorize::Opts),
    #[structopt(about = "Set the default user")]
    Default(cmd::default::Opts),
    #[structopt(about = "Search the list of followers of a user for users who blocks you")]
    Followers(cmd::followers::Opts),
}

#[tokio::main]
async fn main() {
    env_logger::init();

    match Cmd::from_args() {
        Cmd::Authorize(opts) => cmd::authorize::run(opts).await,
        Cmd::Default(opts) => cmd::default::run(opts),
        Cmd::Followers(opts) => cmd::followers::run(opts).await,
    }
}
