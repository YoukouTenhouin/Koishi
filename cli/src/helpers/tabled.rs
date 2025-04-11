use chrono::{DateTime, Utc};

pub(crate) fn timestamp<T>(ts: &i64, _rec: &T) -> String {
    let date: DateTime<Utc> = DateTime::from_timestamp_millis(ts.to_owned()).unwrap();
    date.to_string()
}
