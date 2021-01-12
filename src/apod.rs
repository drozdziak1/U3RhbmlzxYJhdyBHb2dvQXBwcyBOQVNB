//! Apod request code,

use actix_web::client::Client;
use chrono::{Duration as ChronoDuration, NaiveDate};
use diesel::prelude::*;
use futures::lock::Mutex;
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
    pub requests_left: Arc<Mutex<u16>>,
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
            requests_left: Arc::new(Mutex::new(1000u16)),
        }
    }

    /// Retrieves APOD urls specified by `query` from DB cache or
    /// falls back to requesting from NASA.
    pub async fn get_date_range(
        &self,
        db_mut: Arc<Mutex<PgConnection>>,
        query: &ApodQuery,
    ) -> Result<Vec<Url>, ErrBox> {
        use schema::urls::dsl::*;

        let mut records = {
            let db_conn = db_mut.lock().await;
            urls.filter(date.between(&query.start_date, &query.end_date))
                .order(date.asc())
                .load::<Url>(&*db_conn)?
        };

        info!("Records cached in the DB: {:#?}", records);

        let start_date = NaiveDate::parse_from_str(&query.start_date, "%Y-%m-%d")?;
        let end_date = NaiveDate::parse_from_str(&query.end_date, "%Y-%m-%d")?;

        let ranges_todo = compute_missing_ranges(records.as_slice(), start_date, end_date)?;

        info!("Computed ranges to fetch from API: {:#?}", ranges_todo);

        for range in ranges_todo {
            let mut received_urls = self
                .do_get_date_range(&ApodQuery {
                    start_date: range.0.format("%Y-%m-%d").to_string(),
                    end_date: range.1.format("%Y-%m-%d").to_string(),
                    api_key: query.api_key.clone(),
                })
                .await?;

            let db_conn = db_mut.lock().await;

            diesel::insert_into(schema::urls::table)
                .values(&received_urls)
                .execute(&*db_conn)?;

            records.append(&mut received_urls);
        }

        // Should be good for APOD, might need to be more clever for bigger services
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

/// Returns a list of date ranges not covered by ascendingly sorted
/// `Url` slice `records`. `records` must be in `start_date` and
/// `end_date` range.
pub fn compute_missing_ranges(
    records: &[Url],
    start_date: NaiveDate,
    end_date: NaiveDate,
) -> Result<Vec<(NaiveDate, NaiveDate)>, ErrBox> {
    let mut next_expected_date = start_date;
    let mut ranges_todo = vec![];

    if records.is_empty() {
        return Ok(vec![(start_date, end_date)]);
    }
    for record in records.iter() {
        let r_date = NaiveDate::parse_from_str(&record.date, "%Y-%m-%d")?;

        // If we have a hole in our cache, compute the range and
        // add it to the todo list.
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

    // Cover the rightmost gap if before end_date
    if let Some(last) = records.last() {
        let last_record_date = NaiveDate::parse_from_str(&last.date, "%Y-%m-%d")?;

        if last_record_date < end_date {
            let diff = end_date - last_record_date;

            let last_range_start = last_record_date + ChronoDuration::days(1);
            let last_range_end = last_range_start + diff - ChronoDuration::days(1);

            ranges_todo.push((last_range_start, last_range_end));
        }
    }

    Ok(ranges_todo)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_missing_ranges_missing_edges() -> Result<(), ErrBox> {
        let records = vec![Url {
            date: "2020-01-02".to_owned(),
            url: "".to_owned(),
        }];
        let start = NaiveDate::from_ymd(2020, 1, 1);
        let end = NaiveDate::from_ymd(2020, 1, 3);

        let expected = vec![(start.clone(), start.clone()), (end.clone(), end.clone())];

        assert_eq!(
            compute_missing_ranges(records.as_slice(), start, end)?,
            expected
        );

        Ok(())
    }

    #[test]
    fn test_compute_missing_ranges_missing_middle() -> Result<(), ErrBox> {
        let records = vec![
            Url {
                date: "2020-01-01".to_owned(),
                url: "".to_owned(),
            },
            Url {
                date: "2020-01-03".to_owned(),
                url: "".to_owned(),
            },
        ];
        let start = NaiveDate::from_ymd(2020, 1, 1);
        let end = NaiveDate::from_ymd(2020, 1, 3);

        let expected = vec![(
            NaiveDate::from_ymd(2020, 1, 2),
            NaiveDate::from_ymd(2020, 1, 2),
        )];

        assert_eq!(
            compute_missing_ranges(records.as_slice(), start, end)?,
            expected
        );

        Ok(())
    }

    #[test]
    fn test_compute_missing_ranges_empty_records() -> Result<(), ErrBox> {
        let records = vec![];
        let start = NaiveDate::from_ymd(2020, 1, 1);
        let end = NaiveDate::from_ymd(2020, 1, 3);

        let expected = vec![(start.clone(), end.clone())];

        assert_eq!(
            compute_missing_ranges(records.as_slice(), start, end)?,
            expected
        );

        Ok(())
    }
}
