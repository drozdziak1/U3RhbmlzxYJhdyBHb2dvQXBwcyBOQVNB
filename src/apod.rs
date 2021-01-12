//! Apod request code,

use actix_web::client::Client;
use chrono::{Duration as ChronoDuration, NaiveDate};
use diesel::prelude::*;
use futures_intrusive::sync::Semaphore;
use serde::{Deserialize, Serialize};

use std::{sync::Arc, time::Duration};

use crate::{
    db::schema::{self, urls},
    ErrBox,
};

/// A GET query struct for requests to APOD
#[derive(Serialize)]
pub struct ApodQuery {
    pub api_key: String,
    pub start_date: String,
    pub end_date: String,
}

/// Concurrent APOD request dispatch
#[derive(Clone)]
pub struct ApodState {
    /// Job slots
    pub sema: Arc<Semaphore>,
}

/// An APOD picture metadata
#[derive(Clone, Debug, Deserialize, Insertable, Queryable, Ord, PartialOrd, Eq, PartialEq)]
pub struct Url {
    pub date: String,
    pub url: String,
}

impl ApodState {
    pub fn new(concurrent_requests: usize) -> Self {
        Self {
            sema: Arc::new(Semaphore::new(true, concurrent_requests)),
        }
    }

    /// Retrieves APOD urls specified by `query` from DB cache or
    /// falls back to requesting from NASA.
    pub async fn get_date_range(
        &self,
        db_conn: &PgConnection,
        query: &ApodQuery,
    ) -> Result<Vec<Url>, ErrBox> {
        use schema::urls::dsl::*;

        let mut records = urls
            .filter(date.between(&query.start_date, &query.end_date))
            .order(date.asc())
            .load::<Url>(db_conn)?;

        info!("Records: {:#?}", records);

        let start_date = NaiveDate::parse_from_str(&query.start_date, "%Y-%m-%d")?;
        let end_date = NaiveDate::parse_from_str(&query.end_date, "%Y-%m-%d")?;

        let ranges_todo = compute_missing_date_ranges(records.as_slice(), start_date, end_date)?;

        for range in ranges_todo {
            let mut received_urls = self
                .do_get_date_range(&ApodQuery {
                    start_date: range.0.format("%Y-%m-%d").to_string(),
                    end_date: range.1.format("%Y-%m-%d").to_string(),
                    api_key: query.api_key.clone(),
                })
                .await?;

            diesel::insert_into(schema::urls::table)
                .values(&received_urls)
                .execute(db_conn)?;

            records.append(&mut received_urls);
        }

        records.sort();

        return Ok(records);
    }

    /// Retrieves APOD images specified by `query`.
    async fn do_get_date_range(&self, query: &ApodQuery) -> Result<Vec<Url>, ErrBox> {
        // Acquire a job slot
        let _permit = self.sema.acquire(1).await;

        // Take the first client that
        let client = Client::builder().timeout(Duration::from_secs(60)).finish();

        let mut res = client
            .get("https://api.nasa.gov/planetary/apod")
            .query(query)?
            .send()
            .await?;

        let parsed_json: Vec<Url> = res.json().await?;

        Ok(parsed_json)
    }
}

pub fn compute_missing_date_ranges(
    records: &[Url],
    start_date: NaiveDate,
    end_date: NaiveDate,
) -> Result<Vec<(NaiveDate, NaiveDate)>, ErrBox> {
    let mut next_expected_date = start_date;
    let mut ranges_todo = vec![];

    if records.is_empty() {
        ranges_todo.push((start_date, end_date));
    }
    for record in records.iter() {
        let r_date = NaiveDate::parse_from_str(&record.date, "%Y-%m-%d")?;

        // If we have a hole in our cache, compute the range and
        // add it to our todo list.
        if r_date > next_expected_date {
            let diff = r_date - next_expected_date;

            let range_start = next_expected_date;
            let range_end = range_start + diff - ChronoDuration::days(1);

            ranges_todo.push((range_start, range_end));

            // Catch up with current retrieved date
            next_expected_date = r_date;
        }
        next_expected_date += ChronoDuration::days(1);
    }

    info!("Ranges: {:#?}", ranges_todo);

    Ok(ranges_todo)
}
