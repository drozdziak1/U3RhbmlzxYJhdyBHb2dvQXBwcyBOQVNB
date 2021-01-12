//! Apod request code,

use actix_web::client::Client;
use diesel::prelude::*;
use futures_intrusive::sync::Semaphore;
use serde::{Deserialize, Serialize};

use std::{sync::Arc, time::Duration};

use crate::ErrBox;

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
#[derive(Debug, Deserialize, Queryable)]
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

    /// Retrieves APOD urls specified by `query` from DB cache or asks
    /// NASA if anything's missing.
    pub async fn get_date_range(
        &self,
        db_conn: &PgConnection,
        query: &ApodQuery,
    ) -> Result<Vec<Url>, ErrBox> {
        unimplemented!();
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
