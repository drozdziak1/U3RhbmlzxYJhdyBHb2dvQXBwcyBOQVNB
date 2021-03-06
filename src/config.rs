//! Configuration and defaults
use serde::Deserialize;

use crate::ErrBox;

/// Custom env-based settings for this app. Each struct member is
/// filled from corresponding upper-case environment variable.
#[derive(Clone, Debug, Deserialize)]
#[serde(default)]
pub struct Config {
    /// API key to use for NASA APOD
    pub api_key: String,
    /// How many requests should be happening at once?
    pub concurrent_requests: usize,
    /// How to connect to the Postgres db?
    pub database_url: String,
    /// Which host are we gonna use for this service?
    pub host: String,
    /// Which port are we gonna use for HTTP?
    pub port: u16,
}

impl Config {
    /// Loads config from environment variables, including .env
    pub fn init() -> Result<Self, ErrBox> {
        Ok(envy::from_env()?)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            api_key: "DEMO_KEY".to_owned(),
            concurrent_requests: 5,
	    database_url: "SETME".to_owned(),
	    host: "0.0.0.0".to_owned(),
            port: 8080,
        }
    }
}
