Phase 4D.11 decisions

What: new elai-web-search crate — SearxClient HTTP client for SearXNG with rate limiting, HTML stripping, retry.

Key decisions:
1. IPC scope: deferred SearchRequest/FetchRequest IPC variants to 4D.17. 4D.11 only creates the crate; orchestrator calls SearxClient directly.
2. reqwest rustls-tls: used default-features=false + rustls-tls to avoid Windows OpenSSL link failures.
3. ClockProvider + injectable sleep: rate limiter uses ClockProvider for measurement, injectable SleepFn for actual waits. Enables deterministic testing without real wall-clock sleeps.
4. Score non-finite safety: f64 score uses is_finite() guard (not just is_nan()) since serde_json rejects Infinity during serialization.
5. strip_html unclosed tag: preserves content after unclosed < to avoid silent data loss from malformed HTML.
6. Health client reuse: pre-built reqwest::Client with 2s timeout stored on SearxClient, not allocated per health_check call.
7. No bundled work from update-implementation-order.txt: design additions (SurgicalDecomposer, Oracle, FILE_PLAN, RA) are design notes spanning 4 design files, not immediate implementation.

Expected test outcome: 8 unit tests + 8 contract tests pass. Zero real-time sleeps in test suite.
Unknowns: SearXNG JSON response format may vary by version — struct uses serde(default) for resilience.
