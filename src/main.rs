use actix_web::{error, web, App, HttpResponse, HttpServer, Responder};
use serde::Deserialize;
use serde_json::json;

/// Query params for the /pictures endpoint
#[derive(Deserialize)]
struct PictureParams {
    start_date: String,
    end_date: String,
}

/// Retrieves a collection of pictures from NASA APOD within the specified date range
async fn pictures(q: web::Query<PictureParams>) -> impl Responder {
    "Hello, World!"
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
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
