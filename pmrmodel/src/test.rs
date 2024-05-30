use chrono::DateTime;
use std::cell::Cell;

thread_local! {
    static TIMESTAMP: Cell<i64> = Cell::new(1234567890);
}

pub struct Utc;

impl Utc {
    pub fn now() -> DateTime<chrono::Utc> {
        TIMESTAMP.with(|timestamp| DateTime::<chrono::Utc>::from_timestamp(
            timestamp.get(),
            0,
        )).expect("need a valid timestamp set")
    }
}

pub fn set_timestamp(timestamp: i64) {
    TIMESTAMP.with(|ts| ts.set(timestamp));
}
