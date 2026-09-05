Phase 4D.12 decisions

What: Add WebFetcher (HTTP fetching + robots.txt + per-domain rate limiting), ContentExtractor trait + ReadabilityExtractor (HTML to clean text), and supporting types (FetchedPage, ExtractedContent, PageType) to elai-web-search crate.

Key decisions:
1. readability-rust 0.1.0 confirmed on crates.io (cargo search + cargo info). Will try it first. If compilation fails, fall back to inline basic extraction (strip script/style/nav/header/footer, extract text from p/article/main/section/div).
2. ClockProvider has system_now() -> SystemTime. For robots cache TTL (epoch ms), use system_now().duration_since(UNIX_EPOCH).as_millis() as u64. Helper fn clock_ms(&dyn ClockProvider) -> u64.
3. url crate available as reqwest transitive dep (confirmed in Cargo.lock). Use url::Url::parse() for host_str() domain extraction.
4. Per-domain rate limiter: HashMap<String, RateLimiter> in WebFetcher, 500ms interval (2 req/s/domain).
5. Body size enforcement: reqwest bytes_stream() with running count, reject at MAX_BODY_BYTES (2MB).
6. FetchedPage.url = final post-redirect URL from response (response.url().to_string()).
7. ContentExtractor trait is object-safe, signature locked for 4D.17 consumption.
8. Add url as explicit dep (not just transitive) for API stability.

Expected test outcomes:
- Unit tests: fetcher (mock HTTP not needed for contract, test domain extraction + rate limiter wiring), extractor (HTML to text, empty page, basic extraction), robots (cache hit/miss/expiry/404).
- Adversarial: non-UTF8 lossy conversion, >2MB rejection, 404 robots.txt = allowed, JS-only page = Empty.
- Contract tests: struct field presence, error variant display, constant values.
- All tests GREEN after implementation.

Unknowns:
- readability-rust 0.1.0 compilation on current MSRV. Mitigation: try compile, fall back to inline.
- reqwest bytes_stream() availability with current feature flags. May need "stream" feature.
