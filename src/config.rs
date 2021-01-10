use serde::Deserialize;

/// Custom settings for this app
#[derive(Clone, Deserialize)]
#[serde(default)]
pub struct Config {
    /// API key to use for NASA APOD
    api_key: String,
    /// How many requests should be happening at once?
    concurrent_requests: u32,
    /// Which port are we gonna use for HTTP?
    port: u16,
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
