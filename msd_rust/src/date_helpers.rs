/// Utility functions to help with dates.
use chrono::prelude::*;

pub const DEFAULT_DURATION: i32 = 10;

pub fn parse_date(ods: &str) -> DateTime<Utc> {
    let split = ods.split(&['-', ' ', ':'][..]).collect::<Vec<&str>>();
    let ndt = match split.as_slice() {
        [year, month, day, ..] => NaiveDate::from_ymd(
            year.parse().unwrap(),
            month.parse().unwrap(),
            day.parse().unwrap(),
        )
        .and_hms(0, 0, 0),
        _ => panic!(),
    };
    DateTime::<Utc>::from_utc(ndt, Utc)
}

pub fn now() -> DateTime<Utc> {
    Utc::now()
}

pub fn today() -> DateTime<Utc> {
    let today = Utc::today().and_hms(0, 0, 0);
    today
}

pub fn epoch() -> DateTime<Utc> {
    Utc.ymd(1970, 1, 1).and_hms(0, 0, 0)
}
