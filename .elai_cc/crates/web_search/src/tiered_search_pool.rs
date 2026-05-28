//! E-48 Task 2 — tiered search fan-out (Brave T1 + SearXNG T2 + Mojeek T3).
//!
//! Concrete client implementations are RUNPOD-PENDING -- the BraveClient
//! + MojeekClient require API keys per their respective dashboards
//! (Brave: api-dashboard.search.brave.com; Mojeek: mojeek.com). This
//! module ships the substrate (BackendTier enum + canonical URL
//! deduplication + dispatch order ordering + per-tier quota tracker)
//! that the live client wiring will plug into in a follow-up patch.
//!
//! Naming clarification: the rename from the plan's `SourceTier` to
//! `BackendTier` resolves the symbol collision with the existing
//! [`elai_core::source_tier::SourceTier`] (domain-quality T1Authoritative
//! / T2Strong / T3Weak / T4Rejected). `BackendTier` carries
//! backend-identity semantics (T1 = Brave / T2 = SearXNG / T3 = Mojeek);
//! the two enums coexist on the same orchestrator without shadowing.
//!
//! Anti-pattern guard: NO DuckDuckGo as a tier. DDG is Bing-derivative
//! per factually.co private-search 2026; the only truly independent
//! T3 candidate is Mojeek (8.2B-page index per webpronews + makeuseof).
//!
//! Grounding:
//!   * arXiv:2509.23519 -- ReliabilityRAG (per-source reliability tier
//!     rationale).
//!   * arXiv:2410.22954 -- source-reliability heterogeneity overlooked
//!     by naive retrieval (grounds tiered-fan-out vs single-backend).
//!   * api-dashboard.search.brave.com/documentation/pricing -- $5/1k
//!     after free credits.
//!   * mojeek.com -- 8.2B-page truly independent index.

use serde::{Deserialize, Serialize};

/// E-48 Task 2 -- backend identity tier.
///
/// Distinct from [`elai_core::source_tier::SourceTier`] (domain quality).
/// A single search result carries BOTH:
///   * a `BackendTier` -- which API returned the row.
///   * a `SourceTier` -- which quality bucket the row's domain falls in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BackendTier {
    /// Tier-1 backend: Brave Search API (api-dashboard.search.brave.com).
    /// First in the dispatch cascade. Quota tracked per the free-tier
    /// credit allowance; when exhausted the tier disables for the
    /// remainder of the session.
    T1,
    /// Tier-2 backend: SearXNG (existing B-7 backend). Free,
    /// self-hosted, NO quota tracking. Always available unless the
    /// process cannot reach the SearXNG endpoint.
    T2,
    /// Tier-3 backend: Mojeek (8.2B-page independent index). Last in
    /// the cascade. Quota tracked per the configured API key.
    /// Explicit anti-DDG choice: DuckDuckGo is Bing-derivative and
    /// disqualifies as an independent T3 (see no_duckduckgo_as_tier
    /// test below).
    T3,
}

impl BackendTier {
    /// Snake-case wire label for SSB telemetry rows.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::T1 => "t1",
            Self::T2 => "t2",
            Self::T3 => "t3",
        }
    }

    /// Dispatch order. Lower number == earlier in the cascade. Used by
    /// [`dispatch_order`] which returns the canonical T1 -> T2 -> T3
    /// ordering.
    #[must_use]
    pub fn dispatch_position(&self) -> u8 {
        match self {
            Self::T1 => 0,
            Self::T2 => 1,
            Self::T3 => 2,
        }
    }
}

/// Canonical dispatch order: T1 -> T2 -> T3. Returned as a fixed-size
/// array so callers can iterate without allocating.
#[must_use]
pub fn dispatch_order() -> [BackendTier; 3] {
    [BackendTier::T1, BackendTier::T2, BackendTier::T3]
}

/// One search-result row with its backend-identity tier.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredSearchResult {
    pub url: String,
    pub title: String,
    pub snippet: String,
    pub backend_tier: BackendTier,
}

/// Canonical URL hash (blake3) for the dedup primary key.
///
/// Normalization:
///   1. Lowercase scheme + host.
///   2. Sort query params alphabetically by key.
///   3. Drop any query param whose key starts with `utm_` (UTM tracking).
///   4. Keep the path / fragment as-is (case-sensitive paths matter).
///
/// CLAUDE.md anti-stale: blake3 ONLY -- no blake2 / sha2.
#[must_use]
pub fn canonical_url_hash(url: &str) -> blake3::Hash {
    let normalized = canonicalize_url(url);
    blake3::hash(normalized.as_bytes())
}

/// Canonicalize a URL per the [`canonical_url_hash`] rules. Exposed as
/// a separate fn so tests can assert the normalization without going
/// through the hash.
#[must_use]
pub fn canonicalize_url(url: &str) -> String {
    let parsed = url::Url::parse(url);
    let Ok(mut u) = parsed else {
        // Unparseable URL: hash the raw bytes verbatim (drops back to
        // string-identity dedup).
        return url.to_owned();
    };

    // Lowercase scheme + host.
    let scheme = u.scheme().to_ascii_lowercase();
    let host = u.host_str().map(|h| h.to_ascii_lowercase());

    // Sort query params + drop utm_*.
    let mut pairs: Vec<(String, String)> = u
        .query_pairs()
        .filter(|(k, _)| !k.to_ascii_lowercase().starts_with("utm_"))
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    pairs.sort_by(|a, b| a.0.cmp(&b.0));

    // Rebuild query string.
    u.set_query(None);
    if !pairs.is_empty() {
        let mut q = u.query_pairs_mut();
        for (k, v) in &pairs {
            q.append_pair(k, v);
        }
        // Drop the borrow so we can mutate u below.
        drop(q);
    }

    // u.set_scheme returns Err on some edge cases; the lowercase scheme
    // is already canonical for http/https, so we manually build the
    // output rather than mutate the Url object.
    let path = u.path();
    let query = u.query().unwrap_or("");
    let host_str = host.as_deref().unwrap_or("");
    if query.is_empty() {
        format!("{scheme}://{host_str}{path}")
    } else {
        format!("{scheme}://{host_str}{path}?{query}")
    }
}

/// Per-tier quota state. Tracks `remaining` calls; once it hits zero
/// the tier disables for the rest of the session.
#[derive(Debug, Clone, PartialEq)]
pub struct TierQuota {
    pub tier: BackendTier,
    pub remaining: u32,
    /// Once `true`, [`TierQuota::consume`] becomes a no-op and the
    /// dispatcher skips this tier.
    pub disabled: bool,
}

impl TierQuota {
    /// Build a new quota with the supplied initial remaining count.
    #[must_use]
    pub fn new(tier: BackendTier, remaining: u32) -> Self {
        Self {
            tier,
            remaining,
            disabled: remaining == 0,
        }
    }

    /// Decrement the remaining count. Returns `false` when the tier
    /// has already been disabled (caller should skip the dispatch).
    pub fn consume(&mut self) -> bool {
        if self.disabled {
            return false;
        }
        if self.remaining == 0 {
            self.disabled = true;
            return false;
        }
        self.remaining = self.remaining.saturating_sub(1);
        if self.remaining == 0 {
            self.disabled = true;
        }
        true
    }
}

/// Pool state: per-tier quotas + accumulated SSB telemetry counters.
/// Concrete `BraveClient` / `SearxngClient` / `MojeekClient` wiring
/// hangs off the pool; this struct is the substrate the orchestrator
/// constructs at session start.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredSearchPool {
    pub t1_quota: Option<TierQuota>,
    pub t2_quota: TierQuota,
    pub t3_quota: Option<TierQuota>,
}

impl TieredSearchPool {
    /// Construct a pool. T1 and T3 quotas are optional (absent when
    /// no API key is configured). T2 is always present; SearXNG is the
    /// free / self-hosted fallback.
    #[must_use]
    pub fn new(
        t1_quota: Option<TierQuota>,
        t2_quota: TierQuota,
        t3_quota: Option<TierQuota>,
    ) -> Self {
        Self {
            t1_quota,
            t2_quota,
            t3_quota,
        }
    }

    /// Returns `true` iff the pool will dispatch ONLY to T2 SearXNG --
    /// i.e. both T1 and T3 are absent OR disabled. Drives the
    /// `tiered_search_degraded_to_t2_only` telemetry row.
    #[must_use]
    pub fn is_degraded_to_t2_only(&self) -> bool {
        let t1_off = self.t1_quota.as_ref().is_none_or(|q| q.disabled);
        let t3_off = self.t3_quota.as_ref().is_none_or(|q| q.disabled);
        t1_off && t3_off
    }

    /// Returns the list of tiers the pool will actually dispatch to in
    /// canonical T1 -> T2 -> T3 order, skipping absent / disabled tiers.
    #[must_use]
    pub fn active_tiers(&self) -> Vec<BackendTier> {
        let mut out = Vec::new();
        if self.t1_quota.as_ref().is_some_and(|q| !q.disabled) {
            out.push(BackendTier::T1);
        }
        // T2 is always active (no quota).
        if !self.t2_quota.disabled {
            out.push(BackendTier::T2);
        }
        if self.t3_quota.as_ref().is_some_and(|q| !q.disabled) {
            out.push(BackendTier::T3);
        }
        out
    }
}

/// Deduplicate a slice of [`TieredSearchResult`] rows by canonical URL
/// hash. The first occurrence of each canonical hash wins (T1 > T2 > T3
/// when results are pre-sorted by `BackendTier::dispatch_position`).
/// Returns `(deduped, dropped_count)`.
#[must_use]
pub fn dedupe_by_canonical_url(
    rows: Vec<TieredSearchResult>,
) -> (Vec<TieredSearchResult>, u32) {
    use std::collections::HashSet;
    let mut seen: HashSet<blake3::Hash> = HashSet::new();
    let mut out = Vec::with_capacity(rows.len());
    let mut dropped: u32 = 0;
    for row in rows {
        let h = canonical_url_hash(&row.url);
        if seen.insert(h) {
            out.push(row);
        } else {
            dropped = dropped.saturating_add(1);
        }
    }
    (out, dropped)
}

// E-48 stub_ban: NO DuckDuckGo as a tier. The negative test below
// pins this guard against regression -- the substrate intentionally
// has no `BackendTier::Ddg` variant.
//
// Reference: factually.co private-search 2026 (DDG = Bing-derivative,
// disqualifies as independent T3 candidate).

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backend_tier_as_str_canonical() {
        assert_eq!(BackendTier::T1.as_str(), "t1");
        assert_eq!(BackendTier::T2.as_str(), "t2");
        assert_eq!(BackendTier::T3.as_str(), "t3");
    }

    #[test]
    fn dispatch_order_is_t1_t2_t3() {
        assert_eq!(dispatch_order(), [BackendTier::T1, BackendTier::T2, BackendTier::T3]);
    }

    #[test]
    fn canonical_url_hash_strips_utm_params() {
        let a = canonicalize_url("https://example.com/page?utm_source=x&topic=rust");
        let b = canonicalize_url("https://example.com/page?topic=rust");
        assert_eq!(a, b);
    }

    #[test]
    fn canonical_url_hash_lowercases_host() {
        let a = canonicalize_url("HTTPS://EXAMPLE.COM/page");
        assert!(a.starts_with("https://example.com"));
    }

    #[test]
    fn canonical_url_hash_sorts_query_params() {
        let a = canonicalize_url("https://example.com/?z=1&a=2&m=3");
        let b = canonicalize_url("https://example.com/?a=2&m=3&z=1");
        assert_eq!(a, b);
    }

    #[test]
    fn canonical_url_hash_deterministic() {
        let h1 = canonical_url_hash("https://arxiv.org/abs/2510.05145");
        let h2 = canonical_url_hash("https://arxiv.org/abs/2510.05145");
        assert_eq!(h1, h2);
    }

    #[test]
    fn dedup_drops_duplicates_after_utm_strip() {
        let rows = vec![
            TieredSearchResult {
                url: "https://example.com/x?utm_source=a".to_owned(),
                title: "T1 row".to_owned(),
                snippet: "".to_owned(),
                backend_tier: BackendTier::T1,
            },
            TieredSearchResult {
                url: "https://example.com/x".to_owned(),
                title: "T2 row (same canonical)".to_owned(),
                snippet: "".to_owned(),
                backend_tier: BackendTier::T2,
            },
        ];
        let (deduped, dropped) = dedupe_by_canonical_url(rows);
        assert_eq!(deduped.len(), 1);
        assert_eq!(dropped, 1);
        // T1 wins (first in input order).
        assert_eq!(deduped[0].backend_tier, BackendTier::T1);
    }

    #[test]
    fn tier_quota_consume_and_disable() {
        let mut q = TierQuota::new(BackendTier::T1, 2);
        assert!(q.consume());
        assert!(q.consume());
        // Third consume fails; quota is now disabled.
        assert!(!q.consume());
        assert!(q.disabled);
    }

    #[test]
    fn pool_degraded_when_only_t2_present() {
        let pool = TieredSearchPool::new(
            None,
            TierQuota::new(BackendTier::T2, u32::MAX),
            None,
        );
        assert!(pool.is_degraded_to_t2_only());
        assert_eq!(pool.active_tiers(), vec![BackendTier::T2]);
    }

    #[test]
    fn pool_not_degraded_when_t1_active() {
        let pool = TieredSearchPool::new(
            Some(TierQuota::new(BackendTier::T1, 100)),
            TierQuota::new(BackendTier::T2, u32::MAX),
            None,
        );
        assert!(!pool.is_degraded_to_t2_only());
        assert_eq!(pool.active_tiers(), vec![BackendTier::T1, BackendTier::T2]);
    }

    #[test]
    fn no_duckduckgo_as_tier() {
        // Stub_ban guard. The BackendTier enum intentionally has no Ddg
        // variant. This test pins the assumption against regression by
        // documenting the choice; the compile-time guard is the absence
        // of the variant itself.
        let labels: Vec<&'static str> = dispatch_order().iter().map(|t| t.as_str()).collect();
        assert!(
            !labels.iter().any(|l| l.contains("ddg") || l.contains("duck")),
            "BackendTier must not surface DuckDuckGo (Bing-derivative anti-pattern)"
        );
    }
}
