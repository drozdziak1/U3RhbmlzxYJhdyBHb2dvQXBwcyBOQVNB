#[macro_use]
extern crate log;

mod apod;
mod config;
mod handlers;

use actix_web::{error, web, App, HttpResponse, HttpServer};
use failure::format_err;
use log::LevelFilter;
use serde_json::json;

use std::{env, io};

use crate::{apod::ApodState, config::Config, handlers::pictures};

pub type ErrBox = Box<dyn std::error::Error>;

/// Initializes logging on the supplied level unless RUST_LOG was specified
pub fn init_logging(default_lvl: LevelFilter) {
    match env::var("RUST_LOG") {
        Ok(_) => env_logger::init(),
        Err(_) => env_logger::Builder::new().filter_level(default_lvl).init(),
    }
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    init_logging(LevelFilter::Info);

    let cfg = Config::init().map_err(|e| {
        io::Error::new(
            io::ErrorKind::Other,
            format_err!("Could not get config from envs: {}", e),
        )
    })?;

    let apod_state = ApodState::new(cfg.concurrent_requests);

    // WARNING: shows api key, we assume the app stays below DEBUG
    // logging in prod.
    debug!("Config:\n{:#?}", cfg);

    let cfg4app_data = cfg.clone();
    HttpServer::new(move || {
        App::new().service(
            web::resource("/pictures")
                .app_data(
                    // Respond to bad GET queries with status 400
                    // and JSON body: {"error": "<description>"}
                    web::QueryConfig::default().error_handler(|err, _req| {
                        let err_json = json!({ "error": format!("{}", err) });
                        error::InternalError::from_response(
                            err,
                            HttpResponse::BadRequest().json(err_json),
                        )
                        .into()
                    }),
                )
                .app_data(web::Data::new(cfg4app_data.clone()))
                .app_data(web::Data::new(apod_state.clone()))
                .route(web::get().to(pictures)),
        )
    })
    .bind((cfg.host.as_str(), cfg.port))?
    .run()
    .await
}
