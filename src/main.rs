#[macro_use]
extern crate log;

use actix_web::{error, web, App, HttpResponse, HttpServer, Responder};
use failure::bail;
use failure::format_err;
use chrono::NaiveDate;
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

impl PicturesParams {
    pub fn parse_and_validate_date_range(&self) -> Result<(NaiveDate, NaiveDate), failure::Error> {
	let start = NaiveDate::parse_from_str(&self.start_date, "%Y-%m-%d").map_err(|e| format_err!("start_date: {}", e))?;
	let end = NaiveDate::parse_from_str(&self.end_date, "%Y-%m-%d").map_err(|e| format_err!("end_date: {}", e))?;

	if !(start < end) {
	    bail!("Start date must be before end date!");
	}

	Ok((start, end))
    }
}

/// Retrieves a collection of pictures from NASA APOD within the specified date range
async fn pictures(q: web::Query<PicturesParams>) -> impl Responder {
    let (start, end) = match q.parse_and_validate_date_range() {
	Ok(dates) => dates,
	Err(e) => return HttpResponse::BadRequest().json(json!({"error": format!("{}", e)})),
    };
    HttpResponse::Ok().body("Hello, World!")
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
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
