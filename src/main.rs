use actix_web::{web, App, HttpRequest, HttpServer, Responder};

/// Retrieves a collection of pictures from within the specified date range
async fn pictures(req: HttpRequest) -> impl Responder {
    let name = req.match_info().get("name").unwrap_or("World");
    format!("Hello {}!", &name)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/pictures", web::get().to(pictures))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
