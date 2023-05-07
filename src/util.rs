//! Utiliy fns that do not fit anywhere else

use chrono::Utc;
use http::{
    header::{HOST, REFERER},
    HeaderMap,
};
use timediff::*;

pub fn human_relative_time(timestamp: i64) -> String {
    let now: i64 = Utc::now().timestamp();
    let string = format!("{}s", timestamp - now); // TODO: WTF? How to calculate a relative
                                                  // duration without constructing and parsing a string?
    TimeDiff::to_diff(string).parse().unwrap()
}

/// Returns http(s)://domain, depending on what is used inside the headers
pub fn base_url(headers: &HeaderMap) -> String {
    let referer = headers
        .get(REFERER)
        .map_or("https://", |header_value| header_value.to_str().unwrap());
    let splits: Vec<&str> = referer.split(':').collect();
    let proto = match splits[..] {
        [proto, ..] => proto,
        _ => "http",
    };
    let host = headers[HOST].to_str().expect("Unable te get HOST header");
    format!("{proto}://{host}")
}
