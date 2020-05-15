/// The main server
#[macro_use]
extern crate diesel;
extern crate dotenv;
extern crate log;
extern crate reqwest;

use actix_web::middleware::Logger;
use actix_web::web::Data;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use chrono::Duration;
use dotenv::dotenv;
use env_logger::Env;
use listenfd::ListenFd;

mod crud;
mod database;
mod date_helpers;
mod models;
mod schema;

use models::{
    parse_params, CrossoverResponse, DateTimeParams, DateTimePeriod, DateTimeWindowParams, Quotes,
    Signal, Summary,
};

const DBPOOL_ERROR: &'static str = "Could not get db connection from pool.";

/// Handler for the root/home route. Returns a list of instruments so we
/// know the system and database are up and running.
async fn system_summary(pool: Data<database::DBPool>) -> impl Responder {
    let connection = pool.get().expect(DBPOOL_ERROR);
    let instruments = crud::get_instruments(&connection);
    let summary = Summary {
        language: String::from("Rust"),
        instruments,
    };
    HttpResponse::Ok().json(summary)
}

/// Handler that gets quotes for a particular symbol
async fn get_quotes(
    pool: Data<database::DBPool>,
    symbol: web::Path<String>,
    web::Query(dtparams): web::Query<DateTimeParams>,
) -> impl Responder {
    let connection = pool.get().expect(DBPOOL_ERROR);
    let dtp = parse_params(dtparams);
    let s = symbol.to_string();
    let quotes = crud::get_quotes(&connection, &s, &dtp).await;

    let res = Quotes {
        symbol: String::from(s),
        start: dtp.start,
        end: dtp.end,
        duration: dtp.duration,
        quotes,
    };
    HttpResponse::Ok().json(res)
}

/// Handler to get stats for a particular symbol.
async fn get_stats(
    pool: Data<database::DBPool>,
    symbol: web::Path<String>,
    web::Query(dtparams): web::Query<DateTimeParams>,
) -> impl Responder {
    let connection = pool.get().expect(DBPOOL_ERROR);
    let dtp = parse_params(dtparams);
    let s = symbol.to_string();
    let stats = crud::get_stats(&connection, &s, dtp).await;
    HttpResponse::Ok().json(stats)
}

/// Extra functionality not covered in the article.
/// Calculates when the moving average for two
/// different sized windows cross each other.
async fn get_crossovers(
    pool: Data<database::DBPool>,
    symbol: web::Path<String>,
    web::Query(dtwp): web::Query<DateTimeWindowParams>,
) -> impl Responder {
    let connection = pool.get().expect(DBPOOL_ERROR);
    let dtp = parse_params(DateTimeParams {
        start: dtwp.start,
        end: dtwp.end,
        duration: dtwp.duration,
    });
    let sym = symbol.to_string();
    let sm_window = dtwp.small.unwrap_or(7);
    let lg_window = dtwp.large.unwrap_or(21);
    let mut sm_stats = Vec::new();
    let mut lg_stats = Vec::new();
    let first = dtp.start;
    let last = dtp.end;

    for offset in 0..(last - first).num_days() {
        let end_date = first + Duration::days(offset);
        let sm_start = end_date - Duration::days(sm_window as i64);
        let lg_start = end_date - Duration::days(lg_window as i64);
        let sdtp = DateTimePeriod {
            start: sm_start,
            end: end_date,
            duration: sm_window as i32,
        };
        sm_stats.push(crud::get_stats(&connection, &sym, sdtp).await);

        let ldtp = DateTimePeriod {
            start: lg_start,
            end: end_date,
            duration: lg_window as i32,
        };
        lg_stats.push(crud::get_stats(&connection, &sym, ldtp).await);
    }

    let mut signals = Vec::new();
    for (sm, lg) in sm_stats.iter().zip(lg_stats.iter()) {
        let signal = Signal {
            date: sm.end,
            long: sm.mean > lg.mean,
            short: sm.mean < lg.mean,
            close: sm.close,
            sm_sma: sm.mean,
            lg_sma: lg.mean,
        };
        signals.push(signal);
    }

    let mut crossovers: Vec<Signal> = Vec::new();
    let mut long = false;
    let mut short = false;

    for s in signals.iter() {
        if s.long != long || s.short != short {
            crossovers.push(s.clone());
            long = s.long;
            short = s.short;
        }
    }

    let resp = CrossoverResponse {
        symbol: sym,
        small_window: sm_window,
        large_window: lg_window,
        signals: crossovers,
    };

    HttpResponse::Ok().json(resp)
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::from_env(Env::default().default_filter_or("debug")).init();
    let mut listenfd = ListenFd::from_env();
    let mut server = HttpServer::new(|| {
        App::new()
            .data(database::create_pool().clone())
            .wrap(Logger::default())
            .route("/", web::get().to(system_summary))
            .route("/instruments/{symbol}", web::get().to(get_quotes))
            .route("/instruments/{symbol}/stats", web::get().to(get_stats))
            .route(
                "/instruments/{symbol}/crossovers",
                web::get().to(get_crossovers),
            )
    });

    server = if let Some(l) = listenfd.take_tcp_listener(0).unwrap() {
        server.listen(l)?
    } else {
        server.bind("127.0.0.1:8000")?
    };
    server.run().await
}
