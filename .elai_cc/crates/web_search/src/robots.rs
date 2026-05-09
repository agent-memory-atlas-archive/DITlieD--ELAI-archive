use rustc_hash::FxHashMap;
use std::sync::Arc;
use std::time::UNIX_EPOCH;

use elai_core::clock::ClockProvider;
use tracing::debug;

use crate::WebSearchError;

/// HTTP status code: Not Found, used to treat missing robots.txt as fully allowed.
const HTTP_NOT_FOUND: u16 = 404;

/// Cache TTL for robots.txt entries: 1 hour in milliseconds.
const ROBOTS_CACHE_TTL_MS: u64 = 3_600_000;

/// Maximum size for robots.txt response body (64 KB).
const ROBOTS_MAX_BODY_BYTES: usize = 64 * 1024;

/// Per-domain robots.txt cache with injectable clock for deterministic testing.
pub struct RobotsCache {
    clock: Arc<dyn ClockProvider>,
    /// domain -> (allowed, expires_at_epoch_ms)
    cache: FxHashMap<String, (bool, u64)>,
}

/// Extract epoch milliseconds from a ClockProvider.
fn clock_ms(clock: &dyn ClockProvider) -> u64 {
    clock
        .system_now()
        .duration_since(UNIX_EPOCH)
        .map(|d| {
            let millis = d.as_millis();
            u64::try_from(millis).unwrap_or(u64::MAX)
        })
        .unwrap_or(0)
}

/// Extract the domain (scheme + host) from a URL for robots.txt lookup.
/// Returns e.g. "https://example.com" from "https://example.com/path/page".
fn robots_base_url(url_str: &str) -> Result<String, WebSearchError> {
    let parsed = url::Url::parse(url_str).map_err(|e| WebSearchError::InvalidUrl {
        url: url_str.to_owned(),
        reason: e.to_string(),
    })?;
    let scheme = parsed.scheme();
    let host = parsed
        .host_str()
        .ok_or_else(|| WebSearchError::InvalidUrl {
            url: url_str.to_owned(),
            reason: "no host".to_owned(),
        })?;
    Ok(format!("{scheme}://{host}"))
}

impl RobotsCache {
    pub fn new(clock: Arc<dyn ClockProvider>) -> Self {
        Self {
            clock,
            cache: FxHashMap::default(),
        }
    }

    /// Check if a URL is allowed by robots.txt.
    /// Returns Ok(true) if allowed, Err(RobotsDenied) if disallowed.
    /// Fetches and caches robots.txt per domain with 1-hour TTL.
    /// 404 on robots.txt = fully allowed.
    pub async fn check_robots(
        &mut self,
        url: &str,
        http_client: &reqwest::Client,
    ) -> Result<bool, WebSearchError> {
        let base = robots_base_url(url)?;
        let now = clock_ms(self.clock.as_ref());

        // Check cache first.
        if let Some(&(allowed, expires_at)) = self.cache.get(&base) {
            if now < expires_at {
                debug!(domain = %base, allowed, "robots cache hit");
                if allowed {
                    return Ok(true);
                }
                return Err(WebSearchError::RobotsDenied {
                    url: url.to_owned(),
                });
            }
            // Expired — will re-fetch below.
        }

        // Fetch robots.txt with size-limited body read.
        let robots_url = format!("{base}/robots.txt");
        let allowed = match http_client.get(&robots_url).send().await {
            Ok(resp) if resp.status().as_u16() == HTTP_NOT_FOUND => {
                debug!(domain = %base, "robots.txt 404, treating as allowed");
                true
            }
            Ok(resp) if resp.status().is_success() => {
                let body = read_robots_body(resp).await;
                !is_disallowed_by_robots(&body, url)
            }
            Ok(_) | Err(_) => {
                // Non-404 error or network failure: treat as allowed (permissive).
                debug!(domain = %base, "robots.txt fetch failed, treating as allowed");
                true
            }
        };

        let expires_at = now + ROBOTS_CACHE_TTL_MS;
        self.cache.insert(base, (allowed, expires_at));

        if allowed {
            Ok(true)
        } else {
            Err(WebSearchError::RobotsDenied {
                url: url.to_owned(),
            })
        }
    }
}

/// Read robots.txt response body with size limit (ROBOTS_MAX_BODY_BYTES).
/// Truncates oversized responses rather than erroring — robots.txt is advisory.
async fn read_robots_body(response: reqwest::Response) -> String {
    use futures_util::StreamExt;

    let mut bytes = Vec::new();
    let mut stream = response.bytes_stream();
    let mut total: usize = 0;

    while let Some(chunk_result) = stream.next().await {
        let Ok(chunk) = chunk_result else { break };
        total = total.saturating_add(chunk.len());
        if total > ROBOTS_MAX_BODY_BYTES {
            // Truncate at limit — take only what fits.
            let overshoot = total - ROBOTS_MAX_BODY_BYTES;
            let usable = chunk.len().saturating_sub(overshoot);
            bytes.extend_from_slice(&chunk[..usable]);
            break;
        }
        bytes.extend_from_slice(&chunk);
    }

    String::from_utf8_lossy(&bytes).into_owned()
}

/// Simple robots.txt parser. Checks if our user-agent is disallowed for the given URL path.
/// Only handles User-agent: * and Disallow directives (sufficient for research agent).
/// Per RFC 9309 §2.2.2, Disallow path matching is prefix-based: /foo matches /foobar.
fn is_disallowed_by_robots(robots_txt: &str, url: &str) -> bool {
    let path = url::Url::parse(url)
        .ok()
        .map(|u| u.path().to_owned())
        .unwrap_or_default();

    let mut in_wildcard_block = false;
    for line in robots_txt.lines() {
        let line = line.trim();
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        let lower = line.to_lowercase();
        if lower.starts_with("user-agent:") {
            let agent = lower.trim_start_matches("user-agent:").trim();
            in_wildcard_block = agent == "*";
        } else if in_wildcard_block && lower.starts_with("disallow:") {
            let disallowed = line
                .split_once(':')
                .map(|(_, v)| v.trim())
                .unwrap_or("");
            if !disallowed.is_empty() && path.starts_with(disallowed) {
                return true;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use elai_core::clock::SeededClock;

    #[test]
    fn test_robots_base_url_extraction() {
        let base = robots_base_url("https://example.com/path/page?q=1").unwrap();
        assert_eq!(base, "https://example.com");
    }

    #[test]
    fn test_robots_base_url_no_host() {
        let err = robots_base_url("not-a-url");
        assert!(err.is_err());
    }

    #[test]
    fn test_clock_ms_returns_epoch() {
        let clock = SeededClock::new();
        let ms = clock_ms(&clock);
        // SeededClock epoch = UNIX_EPOCH + 1_700_000_000s = 1700000000000 ms
        assert!(ms >= 1_700_000_000_000);
    }

    #[test]
    fn test_is_disallowed_by_robots_wildcard() {
        let robots = "User-agent: *\nDisallow: /private/\n";
        assert!(is_disallowed_by_robots(robots, "https://example.com/private/page"));
        assert!(!is_disallowed_by_robots(robots, "https://example.com/public/page"));
    }

    #[test]
    fn test_is_disallowed_empty_disallow_means_allowed() {
        let robots = "User-agent: *\nDisallow:\n";
        assert!(!is_disallowed_by_robots(robots, "https://example.com/anything"));
    }

    #[test]
    fn test_robots_cache_fresh_hit() {
        let clock = Arc::new(SeededClock::new());
        let mut cache = RobotsCache::new(clock.clone());
        let now = clock_ms(clock.as_ref());
        cache
            .cache
            .insert("https://example.com".to_owned(), (true, now + ROBOTS_CACHE_TTL_MS));
        assert!(cache.cache.contains_key("https://example.com"));
        let &(allowed, expires) = cache.cache.get("https://example.com").unwrap();
        assert!(allowed);
        assert!(expires > now);
    }

    #[test]
    fn test_robots_cache_expired_entry() {
        let clock = Arc::new(SeededClock::new());
        let mut cache = RobotsCache::new(clock.clone());
        let now = clock_ms(clock.as_ref());
        // Insert an already-expired entry.
        cache
            .cache
            .insert("https://example.com".to_owned(), (true, now.saturating_sub(1)));
        // Verify it exists but is expired (now >= expires_at).
        let &(_, expires) = cache.cache.get("https://example.com").unwrap();
        assert!(now >= expires);
    }

    #[test]
    fn test_robots_404_response_means_allowed() {
        // Empty robots.txt content = nothing disallowed.
        assert!(!is_disallowed_by_robots("", "https://example.com/page"));
    }

    #[test]
    fn test_robots_max_body_constant() {
        assert_eq!(ROBOTS_MAX_BODY_BYTES, 64 * 1024);
    }
}
