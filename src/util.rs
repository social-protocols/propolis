use timediff::*;
use chrono::Utc;


pub fn human_relative_time(timestamp: &i64) -> String {
    let now: i64 = Utc::now().timestamp();
    let string = format!("{}s", timestamp - now); // TODO: WTF? How to calculate a relative
    // duration without constructing and parsing a string?
    TimeDiff::to_diff(string).parse().unwrap()
}

