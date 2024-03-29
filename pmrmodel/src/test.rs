use chrono::{DateTime, NaiveDateTime};
use std::cell::Cell;

thread_local! {
    static TIMESTAMP: Cell<i64> = Cell::new(1234567890);
}

pub struct Utc;

impl Utc {
    pub fn now() -> DateTime<chrono::Utc> {
        DateTime::<chrono::Utc>::from_naive_utc_and_offset(
            TIMESTAMP.with(|timestamp| NaiveDateTime::from_timestamp_opt(
                timestamp.get(),
                0,
            ).expect("a valid timestamp set")),
            chrono::Utc,
        )
    }
}

pub fn set_timestamp(timestamp: i64) {
    TIMESTAMP.with(|ts| ts.set(timestamp));
}
