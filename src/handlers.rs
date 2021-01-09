use actix_web::{web, HttpResponse, Responder};
use failure::bail;
use failure::format_err;
use chrono::NaiveDate;
use serde::Deserialize;
use serde_json::json;

/// Query params for the /pictures endpoint
#[derive(Clone, Deserialize)]
pub struct PicturesParams {
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
pub async fn pictures(q: web::Query<PicturesParams>) -> impl Responder {
    let (start, end) = match q.parse_and_validate_date_range() {
	Ok(dates) => dates,
	Err(e) => return HttpResponse::BadRequest().json(json!({"error": format!("{}", e)})),
    };
    HttpResponse::Ok().body("Hello, World!")
}
