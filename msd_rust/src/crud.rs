/// An unusual name and perhaps an unusual approach but I wanted
/// keep it as much like the Python project as possible.
use diesel::pg::PgConnection;
use diesel::prelude::*;
use reqwest;
use serde::{Deserialize, Serialize};

use crate::date_helpers::{epoch, now, parse_date, today};
use crate::models::{DateTimePeriod, Instrument, Quote, Stats};
use crate::schema::instruments::dsl::*;
use crate::schema::quotes::dsl::*;

/*
This is the format we get back from the service.
{
    "symbol": "AAPL",
    "historical": [
        {
            "date": "2015-05-04",
            "open": 129.5,
            "high": 130.57,
            "low": 128.26,
            "close": 128.7,
            "adjClose": 118.44,
            "volume": 5.09883e7,
            "unadjustedVolume": 5.09883e7,
            "change": -0.8,
            "changePercent": -0.618,
            "vwap": 129.17667,
            "label": "May 04, 15",
            "changeOverTime": -0.00618,
        }
    ],
}

*/

/// The record type we expect from the service
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct FMPQuote {
    date: String,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    adj_close: f64,
    volume: f64,
    unadjusted_volume: f64,
    change: f64,
    change_percent: f64,
    vwap: f64,
    label: String,
    change_over_time: f64,
}

/// The entire response we expect from the service
#[derive(Serialize, Deserialize, Debug)]
struct FMPResponse {
    symbol: String,
    historical: Vec<FMPQuote>,
}

/// We need to re-fetch the quotes from the service on
/// a daily basis. So fetch if we don't have any records
/// or if they are not from today.
async fn fetch_if_necessary(
    connection: &PgConnection,
    sym: &str,
) -> Result<usize, Box<dyn std::error::Error>> {
    let qr = get_instrument(connection, sym);
    let mut need_fetch = false;
    let mut count = 0;

    match qr {
        Err(_) => {
            let new_instrument = Instrument {
                symbol: String::from(sym),
                fetched: epoch(),
            };
            diesel::insert_into(crate::schema::instruments::table)
                .values(&new_instrument)
                .execute(connection)
                .expect("Error saving new instrument");
            need_fetch = true;
        }
        Ok(inst) => {
            if inst.fetched < today() {
                need_fetch = true;
            }
        }
    }

    if need_fetch {
        let url = format!(
            "https://financialmodelingprep.com/api/v3/historical-price-full/{}",
            sym.to_uppercase()
        );

        let resp = reqwest::get(&url).await?.json::<FMPResponse>().await?;
        count = resp.historical.len();
        let mut qs = Vec::new();

        for h in resp.historical {
            let q = Quote {
                symbol: String::from(sym),
                date: parse_date(&h.date),
                open: h.open,
                high: h.high,
                low: h.low,
                close: h.adj_close,
                volume: 0,
            };
            qs.push(q);
        }

        /* Delete the old quotes, insert new ones, update the instrument. */
        diesel::delete(quotes)
            .filter(crate::schema::quotes::dsl::symbol.eq(sym))
            .execute(connection)
            .expect("Error deleting quotes");
        diesel::insert_into(quotes)
            .values(&qs)
            .execute(connection)
            .expect("Error inserting new quotes");
        diesel::update(instruments.filter(crate::schema::instruments::dsl::symbol.eq(sym)))
            .set(fetched.eq(now()))
            .get_result::<Instrument>(connection)
            .expect("Unable to update instrument");
    }
    Ok(count)
}

/// Get the record for a specific record
pub fn get_instrument(connection: &PgConnection, sym: &str) -> QueryResult<Instrument> {
    let result = instruments
        .filter(crate::schema::instruments::dsl::symbol.eq(sym))
        .first::<Instrument>(connection);
    result
}

/// Get all the instruments we have in the database.
pub fn get_instruments(connection: &PgConnection) -> Vec<Instrument> {
    let results = instruments
        .load::<Instrument>(connection)
        .expect("Error loading instruments");
    results
}

/// Get the quotes for a particular instrument
pub async fn get_quotes(connection: &PgConnection, sym: &str, dtp: &DateTimePeriod) -> Vec<Quote> {
    let _ = fetch_if_necessary(connection, sym).await;

    let results = quotes
        .filter(crate::schema::quotes::dsl::symbol.eq(sym))
        .filter(crate::schema::quotes::dsl::date.ge(dtp.start))
        .filter(crate::schema::quotes::dsl::date.le(dtp.end))
        .load::<Quote>(connection)
        .expect("Error loading instruments");
    results
}

/*
I decided not to implement this functionality in the Rust version but this is what it would look
like if we had kept it.

pub async fn get_quote_as_of(connection: &PgConnection, sym: &str, d: DateTime<Utc>) -> Quote {
    let _ = fetch_if_necessary(connection, sym).await;

    let results = quotes
        .filter(crate::schema::quotes::dsl::symbol.eq(sym))
        .filter(crate::schema::quotes::dsl::date.le(d))
        .first::<Quote>(connection)
        .expect("Error getting quote as of");
    results
}
*/

/// Calculate stats for an instrument for a date range. Note that I decided
/// to do the calculations in code here and in the db in the Python version
/// for reasons :)
pub async fn get_stats(connection: &PgConnection, sym: &str, dtp: DateTimePeriod) -> Stats {
    let _ = fetch_if_necessary(connection, sym).await;

    let closes: Vec<f64> = quotes
        .filter(crate::schema::quotes::dsl::symbol.eq(sym))
        .filter(crate::schema::quotes::dsl::date.ge(dtp.start))
        .filter(crate::schema::quotes::dsl::date.le(dtp.end))
        .select(crate::schema::quotes::dsl::close)
        .load::<f64>(connection)
        .expect("Could get quotes in date range.");
    let sum: f64 = closes.iter().sum();
    let mean = sum / closes.len() as f64;
    let max = closes.iter().cloned().fold(closes[0], f64::max);
    let min = closes.iter().cloned().fold(closes[0], f64::min);
    let last = *closes.last().unwrap();
    Stats {
        symbol: String::from(sym),
        start: dtp.start,
        end: dtp.end,
        duration: dtp.duration,
        count: closes.len() as i32,
        close: last,
        mean: mean,
        min: min,
        max: max,
    }
}
