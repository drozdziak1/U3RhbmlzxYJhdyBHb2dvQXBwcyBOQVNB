#[macro_use]
extern crate log;

mod handlers;

use actix_web::{error, web, App, HttpResponse, HttpServer};
use log::LevelFilter;
use serde_json::json;

use std::env;

use crate::handlers::pictures;

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
