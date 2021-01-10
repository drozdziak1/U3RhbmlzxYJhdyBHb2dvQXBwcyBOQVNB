use serde::Deserialize;

use crate::ErrBox;

/// Custom env-based settings for this app. Each struct member is
/// filled from corresponding upper-case environment variable.
#[derive(Clone, Debug, Deserialize)]
#[serde(default)]
pub struct Config {
    /// API key to use for NASA APOD
    api_key: String,
    /// How many requests should be happening at once?
    concurrent_requests: u32,
    /// Which port are we gonna use for HTTP?
    port: u16,
}

impl Config {
    /// Loads config from environment variables, including .env
    pub fn init() -> Result<Self, ErrBox> {
	dotenv::dotenv()?;
        Ok(envy::from_env()?)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            api_key: "DEMO_KEY".to_owned(),
            concurrent_requests: 5,
            port: 8080,
        }
    }
}
