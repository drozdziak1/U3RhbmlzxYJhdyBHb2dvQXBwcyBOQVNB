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

    pub fn parse_and_validate(&self) -> Result<(NaiveDate, NaiveDate), failure::Error> {
	let start = NaiveDate::parse_from_str(&self.start_date, "%Y-%m-%d").map_err(|e| format_err!("start_date: {}", e))?;
	let end = NaiveDate::parse_from_str(&self.end_date, "%Y-%m-%d").map_err(|e| format_err!("end_date: {}", e))?;

	if !(start <= end) {
	    bail!("Start date must be before end date!");
	}

	Ok((start, end))
    }
}

/// Retrieves a collection of pictures from NASA APOD within the specified date range
pub async fn pictures(q: web::Query<PicturesParams>) -> impl Responder {
    let (start, end) = match q.parse_and_validate() {
	Ok(dates) => dates,
	Err(e) => return HttpResponse::BadRequest().json(json!({"error": format!("{}", e)})),
    };
    HttpResponse::Ok().body("Hello, World!")
}

#[cfg(test)]
mod tests {
    use super::*;

    impl PicturesParams {
	/// Quality-of-life constructor for testing
	pub fn new(s: &str, e: &str) -> Self {
	    Self {
		start_date: s.to_owned(),
		end_date: e.to_owned(),
	    }	
	}
    }

    #[test]
    fn test_parse_and_validate_correct() -> Result<(), failure::Error> {
	let ok = PicturesParams::new("2021-01-01", "2021-01-02");

	let (ok_start, ok_end) = ok.parse_and_validate()?;

	assert_eq!(ok_start, NaiveDate::from_ymd(2021, 1, 1));
	assert_eq!(ok_end, NaiveDate::from_ymd(2021, 1, 2));

	Ok(())
    }

    #[test]
    fn test_parse_and_validate_same_day() -> Result<(), failure::Error> {
	let ok = PicturesParams::new("2021-01-01", "2021-01-01");

	let (ok_start, ok_end) = ok.parse_and_validate()?;

	assert_eq!(ok_start, NaiveDate::from_ymd(2021, 1, 1));
	assert_eq!(ok_end, NaiveDate::from_ymd(2021, 1, 1));

	Ok(())
    }
    
    #[test]
    fn test_parse_and_validate_wrong_start() {
	let ok = PicturesParams::new("Gibberish", "2021-01-02");

	assert!(ok.parse_and_validate().is_err());
    }
    
    #[test]
    fn test_parse_and_validate_wrong_end() {
	let ok = PicturesParams::new("2021-01-01", "My birthday");

	assert!(ok.parse_and_validate().is_err());
    }
    
    #[test]
    fn test_parse_and_validate_start_after_end() {
	let ok = PicturesParams::new("2021-01-03", "2021-01-02");

	assert!(ok.parse_and_validate().is_err());
    }
}
