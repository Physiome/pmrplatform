use chrono::{DateTime, NaiveDateTime};

pub struct Utc;

impl Utc {
    pub fn now() -> DateTime<chrono::Utc> {
        DateTime::<chrono::Utc>::from_utc(
            NaiveDateTime::from_timestamp(1234567890, 0),
            chrono::Utc,
        )
    }
}
