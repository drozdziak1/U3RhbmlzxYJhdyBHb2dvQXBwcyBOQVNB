#[macro_use]
extern crate log;

mod config;
mod handlers;

use actix_web::{error, web, App, HttpResponse, HttpServer};
use failure::format_err;
use log::LevelFilter;
use serde_json::json;

use std::{env, io, sync::Mutex};

use crate::{config::Config, handlers::pictures};

/// This helper function initializes logging on the supplied level unless RUST_LOG was specified
pub fn init_logging(default_lvl: LevelFilter) {
    match env::var("RUST_LOG") {
        Ok(_) => env_logger::init(),
        Err(_) => env_logger::Builder::new().filter_level(default_lvl).init(),
    }
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    init_logging(LevelFilter::Info);

    let cfg: Config = envy::from_env().map_err(|e| {
        io::Error::new(
            io::ErrorKind::Other,
            format_err!("Could not get config from envs: {}", e),
        )
    })?;

    HttpServer::new(move || {
        App::new().service(
            web::resource("/pictures")
                .app_data(
                    // Respond to bad GET queries with with status 400
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
                .app_data(web::Data::new(cfg.clone()))
                .route(web::get().to(pictures)),
        )
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
