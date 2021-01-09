#[macro_use]
extern crate log;

use actix_web::{error, web, App, HttpResponse, HttpServer, Responder};
use log::LevelFilter;
use serde::Deserialize;
use serde_json::json;

use std::env;

/// Query params for the /pictures endpoint
#[derive(Deserialize)]
struct PicturesParams {
    start_date: String,
    end_date: String,
}

/// Retrieves a collection of pictures from NASA APOD within the specified date range
async fn pictures(q: web::Query<PicturesParams>) -> impl Responder {
    "Hello, World!"
}

/// This helper function initializes logging on the supplied level unless RUST_LOG was specified
pub fn init_logging(default_lvl: LevelFilter) {
    match env::var("RUST_LOG") {
        Ok(_) => env_logger::init(),
        Err(_) => env_logger::Builder::new().filter_level(default_lvl).init(),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    init_logging(LevelFilter::Info);
    HttpServer::new(|| {
        App::new().service(
            web::resource("/pictures")
                .app_data(
		    // Respond to bad GET queries with with status 400
		    // and JSON body: {"error": "<description>"}
                    web::QueryConfig::default().error_handler(|err, _req| {
			let err_json = json!({"error": format!("{}", err) });
                        error::InternalError::from_response(
                            err,
			    HttpResponse::BadRequest().json(err_json),
			    )
                        .into()
                    }),
                )
                .route(web::get().to(pictures)),
        )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
