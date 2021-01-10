//! Apod request code, 

use futures_intrusive::sync::Semaphore;

/// Data global to all request jobs
pub struct ApodState {
    /// Job slots for concurrent requests
    pub sema: Semaphore,
    /// How many requests are left for our API key
    pub req_cnt_left: Option<u32>,
}

impl ApodState {
    pub fn new(concurrent_requests: usize) -> Self {
	Self {
	    sema: Semaphore::new(false, concurrent_requests),
	    req_cnt_left: None,
	}
    }
}
