use std::time::{Duration, SystemTime, UNIX_EPOCH};

use diesel::prelude::*;
use tokio::time::Delay;

pub fn connect_database(uri: &str) -> ConnectionResult<SqliteConnection> {
    SqliteConnection::establish(uri)
}

pub fn instant_to_epoch(t: tokio::time::Instant) -> u64 {
    let t = t.into_std();
    let now_s = SystemTime::now();
    let now_i = std::time::Instant::now();
    let t_s = if t > now_i {
        (now_s + (t - now_i))
    } else {
        (now_s - (now_i - t))
    };
    t_s.duration_since(UNIX_EPOCH)
        .expect("`Instant` must be after Unix epoch")
        .as_secs()
}

/// Returns a future to wait until the specified Unix time.
pub fn wait_until(until: u64) -> Delay {
    let until = Duration::from_secs(until);
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    if until > now {
        let wait = until - now;
        log::debug!("Sleeping for {} secs", wait.as_secs());
        tokio::time::delay_for(wait)
    } else {
        tokio::time::delay_until(tokio::time::Instant::now())
    }
}
