/// Our models /structs
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::convert::TryInto;

use crate::date_helpers::{parse_date, DEFAULT_DURATION};
use crate::schema::{instruments, quotes};

/// Instrument record - is inserted into db.
#[derive(Serialize, Deserialize, Debug, Queryable, Insertable)]
pub struct Instrument {
    pub symbol: String,
    pub fetched: DateTime<Utc>,
}

/// Our root route response format
#[derive(Serialize, Deserialize, Debug)]
pub struct Summary {
    pub language: String,
    pub instruments: Vec<Instrument>,
}

/// Convience structure to handle our date range specification
#[derive(Serialize, Deserialize, Debug)]
pub struct DateTimeParams {
    pub start: Option<String>,
    pub end: Option<String>,
    pub duration: Option<i32>,
}

/// Similar to date time params, window params add a small and
/// large window size.
#[derive(Serialize, Deserialize, Debug)]
pub struct DateTimeWindowParams {
    pub start: Option<String>,
    pub end: Option<String>,
    pub duration: Option<i32>,
    pub small: Option<i32>,
    pub large: Option<i32>,
}

/// The date time params converted to actual dates.
#[derive(Serialize, Deserialize, Debug)]
pub struct DateTimePeriod {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub duration: i32,
}

/// The struct/record for our quote data.
#[derive(Serialize, Deserialize, Debug, Queryable, Insertable)]
#[table_name = "quotes"]
pub struct Quote {
    pub symbol: String,
    pub date: DateTime<Utc>,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: i32,
}

/// Response format for quotes handler
#[derive(Serialize, Deserialize, Debug)]
pub struct Quotes {
    pub symbol: String,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub duration: i32,
    pub quotes: Vec<Quote>,
}

/// Response format for stats handler
#[derive(Serialize, Deserialize, Debug)]
pub struct Stats {
    pub symbol: String,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub duration: i32,
    pub count: i32,
    pub close: f64,
    pub mean: f64,
    pub min: f64,
    pub max: f64,
}

/// A record of an individual small, large window
/// crossover.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Signal {
    pub date: DateTime<Utc>,
    pub long: bool,
    pub short: bool,
    pub close: f64,
    pub sm_sma: f64,
    pub lg_sma: f64,
}

/// The response data structure for crossovers
#[derive(Serialize, Deserialize, Debug)]
pub struct CrossoverResponse {
    pub symbol: String,
    pub small_window: i32,
    pub large_window: i32,
    pub signals: Vec<Signal>,
}

/// Parse the params into a date time period. Review the python 
/// implementation for details but basically a user can specify 
/// a start date, end date or duration and we try to create an
/// appropriate date range.
pub fn parse_params(dtp: DateTimeParams) -> DateTimePeriod {
    let (start_date, end_date, duration) = match (dtp.start, dtp.end, dtp.duration) {
        (Some(s), Some(e), Some(d)) => {
            let sd = parse_date(&s);
            let ed = parse_date(&e);
            let cd: i32 = (ed - sd).num_days().try_into().unwrap();
            if cd != d {
                panic!("Calculated date duration does not match specified duration.")
            }
            (sd, ed, d)
        }
        (Some(s), Some(e), None) => {
            let sd = parse_date(&s);
            let ed = parse_date(&e);
            let d: i32 = (ed - sd).num_days().try_into().unwrap();
            (sd, ed, d)
        }
        (Some(s), None, Some(d)) => {
            let sd = parse_date(&s);
            let ed = sd + Duration::days(d.into());
            (sd, ed, d)
        }

        (Some(s), None, None) => {
            let sd = parse_date(&s);
            let ed = sd + Duration::days(DEFAULT_DURATION.into());
            (sd, ed, DEFAULT_DURATION)
        }
        (None, Some(e), Some(d)) => {
            let ed = parse_date(&e);
            let sd = ed - Duration::days(d.into());
            (sd, ed, d)
        }
        (None, Some(e), None) => {
            let ed = parse_date(&e);
            let sd = ed - Duration::days(DEFAULT_DURATION.into());
            (sd, ed, DEFAULT_DURATION)
        }
        (None, None, Some(d)) => {
            let ed = DateTime::from(Utc::now());
            let sd = ed - Duration::days(d.into());
            (sd, ed, d)
        }
        (None, None, None) => {
            let ed = DateTime::from(Utc::now());
            let sd = ed - Duration::days(DEFAULT_DURATION.into());
            (sd, ed, DEFAULT_DURATION)
        }
    };

    DateTimePeriod {
        start: start_date,
        end: end_date,
        duration: duration,
    }
}
