mod api;
mod models;

pub use api::*;
pub use models::*;

use atoi::atoi;
use reqwest::header::{HeaderMap, HeaderName};

pub struct RateLimit {
    pub remaining: u64,
    pub reset: u64,
}

pub fn rate_limit(headers: &HeaderMap) -> Option<RateLimit> {
    let x_rate_limit_remaining = &HeaderName::from_static("x-rate-limit-remaining");
    let x_rate_limit_reset = &HeaderName::from_static("x-rate-limit-reset");

    headers.get(x_rate_limit_remaining).and_then(|v| {
        atoi(v.as_bytes()).and_then(|remaining| {
            headers
                .get(x_rate_limit_reset)
                .and_then(|v| atoi(v.as_bytes()).map(|reset| RateLimit { remaining, reset }))
        })
    })
}
