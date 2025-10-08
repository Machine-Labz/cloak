use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Rate limiter using sliding window algorithm
#[derive(Debug, Clone)]
pub struct RateLimiter {
    requests: Arc<RwLock<HashMap<String, Vec<Instant>>>>,
    window_size: Duration,
    max_requests: usize,
}

impl RateLimiter {
    /// Create a new rate limiter
    /// 
    /// # Arguments
    /// * `window_size` - Time window for rate limiting (e.g., 1 minute)
    /// * `max_requests` - Maximum requests allowed per window per client
    pub fn new(window_size: Duration, max_requests: usize) -> Self {
        Self {
            requests: Arc::new(RwLock::new(HashMap::new())),
            window_size,
            max_requests,
        }
    }

    /// Check if a request is allowed for the given client
    /// 
    /// # Arguments
    /// * `client_id` - Unique identifier for the client (e.g., IP address)
    /// 
    /// # Returns
    /// * `true` if request is allowed
    /// * `false` if rate limit exceeded
    pub async fn is_allowed(&self, client_id: &str) -> bool {
        let now = Instant::now();
        let cutoff = now - self.window_size;

        let mut requests = self.requests.write().await;
        
        // Get or create client request history
        let client_requests = requests.entry(client_id.to_string()).or_insert_with(Vec::new);
        
        // Remove old requests outside the window
        client_requests.retain(|&time| time > cutoff);
        
        // Check if we're under the limit
        if client_requests.len() < self.max_requests {
            client_requests.push(now);
            true
        } else {
            false
        }
    }

    /// Get remaining requests for a client in the current window
    pub async fn remaining_requests(&self, client_id: &str) -> usize {
        let now = Instant::now();
        let cutoff = now - self.window_size;

        let requests = self.requests.read().await;
        
        if let Some(client_requests) = requests.get(client_id) {
            let valid_requests = client_requests.iter().filter(|&&time| time > cutoff).count();
            self.max_requests.saturating_sub(valid_requests)
        } else {
            self.max_requests
        }
    }

    /// Get time until next request is allowed
    pub async fn time_until_reset(&self, client_id: &str) -> Option<Duration> {
        let now = Instant::now();
        let cutoff = now - self.window_size;

        let requests = self.requests.read().await;
        
        if let Some(client_requests) = requests.get(client_id) {
            let valid_requests = client_requests.iter().filter(|&&time| time > cutoff).count();
            
            if valid_requests >= self.max_requests {
                // Find the oldest request in the window
                if let Some(oldest_request) = client_requests.iter()
                    .filter(|&&time| time > cutoff)
                    .min() 
                {
                    let reset_time = *oldest_request + self.window_size;
                    return Some(reset_time.duration_since(now));
                }
            }
        }
        
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_rate_limiter_allows_requests_within_limit() {
        let limiter = RateLimiter::new(Duration::from_secs(60), 5);
        
        // Should allow first 5 requests
        for i in 0..5 {
            assert!(limiter.is_allowed("test_client").await, "Request {} should be allowed", i);
        }
        
        // 6th request should be denied
        assert!(!limiter.is_allowed("test_client").await, "6th request should be denied");
    }

    #[tokio::test]
    async fn test_rate_limiter_resets_after_window() {
        let limiter = RateLimiter::new(Duration::from_millis(100), 2);
        
        // Use up the limit
        assert!(limiter.is_allowed("test_client").await);
        assert!(limiter.is_allowed("test_client").await);
        assert!(!limiter.is_allowed("test_client").await);
        
        // Wait for window to reset
        sleep(Duration::from_millis(150)).await;
        
        // Should allow requests again
        assert!(limiter.is_allowed("test_client").await);
    }

    #[tokio::test]
    async fn test_remaining_requests() {
        let limiter = RateLimiter::new(Duration::from_secs(60), 5);
        
        // Initially should have 5 remaining
        assert_eq!(limiter.remaining_requests("test_client").await, 5);
        
        // After 2 requests, should have 3 remaining
        limiter.is_allowed("test_client").await;
        limiter.is_allowed("test_client").await;
        assert_eq!(limiter.remaining_requests("test_client").await, 3);
    }
}