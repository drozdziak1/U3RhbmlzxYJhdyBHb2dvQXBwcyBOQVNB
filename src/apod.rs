//! Apod request code,

use actix_web::client::Client;
use futures_intrusive::sync::Semaphore;
use serde::{Deserialize, Serialize};

use std::{sync::Arc, time::Duration};

use crate::ErrBox;

#[derive(Serialize)]
pub struct ApodQuery {
    pub api_key: String,
    pub start_date: String,
    pub end_date: String,
}

/// Concurrent job synchronization state
#[derive(Clone)]
pub struct ApodState {
    pub sema: Arc<Semaphore>,
}

/// An APOD picture metadata
#[derive(Debug, Deserialize)]
pub struct ApodRecord {
    pub date: String,
    pub url: String,
}

impl ApodState {
    pub fn new(concurrent_requests: usize) -> Self {
        Self {
            sema: Arc::new(Semaphore::new(true, concurrent_requests)),
        }
    }
    /// Retrieves APOD images specified by `query`.
    pub async fn do_get_date_range(&self, query: &ApodQuery) -> Result<Vec<ApodRecord>, ErrBox> {
        // Acquire a job slot
        let _permit = self.sema.acquire(1).await;

        // Take the first client that
        let client = Client::builder().timeout(Duration::from_secs(60)).finish();

        let mut res = client
            .get("https://api.nasa.gov/planetary/apod")
            .query(query)?
            .send()
            .await?;

        let parsed_json: Vec<ApodRecord> = res.json().await?;

        Ok(parsed_json)
    }
}
