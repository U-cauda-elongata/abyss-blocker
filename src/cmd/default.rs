use diesel::{dsl::*, prelude::*};
use structopt::StructOpt;

use crate::common::connect_database;
use crate::schema::*;

#[derive(StructOpt)]
pub struct Opts {
    #[structopt(about = "User ID")]
    user: i64,
    #[structopt(long, about = "Path to the database", default_value = "db.sqlite3")]
    database: String,
}

pub fn run(opts: Opts) {
    let conn = connect_database(&opts.database).unwrap();

    conn.transaction::<_, diesel::result::Error, _>(|| {
        delete(default_user::table).execute(&conn)?;
        insert_into(default_user::table)
            .values(default_user::user.eq(opts.user))
            .execute(&conn)?;
        Ok(())
    })
    .unwrap();
}
