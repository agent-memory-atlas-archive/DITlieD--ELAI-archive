//! J-54 QuotaTracker — per-(KeyId, model) daily quota + cooldown tracking.
//!
//! KeyId stores ONLY a blake3 fingerprint (first 8 hex chars = 32 bits),
//! NEVER the raw API key (CB-1, invariant I-2). Day counters persist to
//! ~/.elai/free_pool/quota_state.json (atomic tmp+rename); per-minute
//! counters + cooldowns do NOT persist (interface map I-3).
//!
//! note_429 header priority (MINIMUM-wins):
//!   x-ratelimit-reset-requests > Retry-After >
//!   x-ratelimit-reset-tokens > 60s default

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

/// blake3 fingerprint of an API key — first 8 hex chars (32 bits).
/// Per-provider namespace keeps practical collision negligible for a
/// quota key (a collision over-counts one key, never leaks a key).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct KeyId {
    pub provider: String,
    pub fp: String,
}

impl KeyId {
    /// Consume the raw key and return a KeyId carrying ONLY the blake3
    /// fingerprint. The raw key is never stored (CB-1, invariant I-2).
    #[must_use]
    pub fn from_raw_key(provider: &str, raw: &str) -> Self {
        let hash = blake3::hash(raw.as_bytes());
        let hex = hash.to_hex();
        let fp = &hex.as_str()[..8];
        Self {
            provider: provider.to_string(),
            fp: fp.to_string(),
        }
    }
}

/// Per-(KeyId, model) quota counters. Only day-level counters persist.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
struct DayCounters {
    day_reqs: u64,
    day_tokens: u64,
}

/// Per-minute counters (not persisted).
#[derive(Debug, Clone, Default)]
struct MinuteCounters {
    minute_reqs: u64,
    minute_tokens: u64,
}

/// Cooldown expiry (unix seconds). Not persisted.
#[derive(Debug, Clone)]
struct Cooldown {
    expires_at: u64,
}

/// Quota limits for a (provider, model) pair.
#[derive(Debug, Clone, Copy)]
pub struct QuotaLimits {
    pub rpd: u64,
    pub tpd: u64,
    pub rpm: u64,
    pub tpm: u64,
}

impl QuotaLimits {
    fn unlimited() -> Self {
        Self {
            rpd: u64::MAX,
            tpd: u64::MAX,
            rpm: u64::MAX,
            tpm: u64::MAX,
        }
    }
}

/// Persisted quota state — only day counters + the UTC day they belong to.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct PersistedState {
    utc_day: u64,
    counters: HashMap<String, DayCounters>,
}

/// The quota tracker. Interior-mutable via Mutex.
pub struct QuotaTracker {
    inner: Mutex<Inner>,
    limits: HashMap<(String, String), QuotaLimits>,
    state_path: PathBuf,
}

struct Inner {
    utc_day: u64,
    day: HashMap<String, DayCounters>,
    minute: HashMap<String, MinuteCounters>,
    minute_minute: u64,
    cooldowns: HashMap<String, Cooldown>,
}

fn utc_day(now: SystemTime) -> u64 {
    let secs = now.duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    secs / 86_400
}

fn utc_minute(now: SystemTime) -> u64 {
    let secs = now.duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    secs / 60
}

fn quota_key(key: &KeyId, model: &str) -> String {
    format!("{}:{}:{}", key.provider, key.fp, model)
}

impl QuotaTracker {
    /// Build a QuotaTracker with the given limits map and default persist path.
    pub fn new(
        limits: HashMap<(String, String), QuotaLimits>,
        state_path: PathBuf,
    ) -> Self {
        let now = SystemTime::now();
        let day = utc_day(now);
        let persisted = read_persisted(&state_path).unwrap_or_default();
        let day = if persisted.utc_day == day {
            day
        } else {
            write_persisted(&state_path, &PersistedState {
                utc_day: day,
                counters: HashMap::new(),
            })
            .ok();
            day
        };
        Self {
            inner: Mutex::new(Inner {
                utc_day: day,
                day: persisted.counters,
                minute: HashMap::new(),
                minute_minute: utc_minute(now),
                cooldowns: HashMap::new(),
            }),
            limits,
            state_path,
        }
    }

    /// Build with no per-model limits (unlimited) and a given state path.
    pub fn unlimited(state_path: PathBuf) -> Self {
        Self::new(HashMap::new(), state_path)
    }

    fn roll_day(&self, inner: &mut Inner) {
        let now_day = utc_day(SystemTime::now());
        if now_day != inner.utc_day {
            inner.utc_day = now_day;
            inner.day.clear();
            inner.minute.clear();
            inner.cooldowns.clear();
            inner.minute_minute = utc_minute(SystemTime::now());
            let _ = write_persisted(
                &self.state_path,
                &PersistedState {
                    utc_day: now_day,
                    counters: HashMap::new(),
                },
            );
        }
    }

    fn roll_minute(&self, inner: &mut Inner) {
        let now_min = utc_minute(SystemTime::now());
        if now_min != inner.minute_minute {
            inner.minute_minute = now_min;
            inner.minute.clear();
        }
    }

    /// Check if a (key, model) has headroom under rpd/tpd/rpm/tpm + cooldown.
    pub fn has_headroom(&self, key: &KeyId, model: &str) -> bool {
        let mut inner = self.inner.lock().expect("quota mutex poisoned");
        self.roll_day(&mut inner);
        self.roll_minute(&mut inner);
        let qk = quota_key(key, model);
        if let Some(cd) = inner.cooldowns.get(&qk) {
            let now_secs = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            if now_secs < cd.expires_at {
                return false;
            }
        }
        let limits = self
            .limits
            .get(&(key.provider.clone(), model.to_string()))
            .copied()
            .unwrap_or_else(QuotaLimits::unlimited);
        let day = inner.day.get(&qk).cloned().unwrap_or_default();
        if day.day_reqs >= limits.rpd || day.day_tokens >= limits.tpd {
            return false;
        }
        let min = inner.minute.get(&qk).cloned().unwrap_or_default();
        if min.minute_reqs >= limits.rpm || min.minute_tokens >= limits.tpm {
            return false;
        }
        true
    }

    /// Record a successful completion's token usage.
    pub fn record_success(&self, key: &KeyId, model: &str, tokens: u64) {
        let mut inner = self.inner.lock().expect("quota mutex poisoned");
        self.roll_day(&mut inner);
        self.roll_minute(&mut inner);
        let qk = quota_key(key, model);
        let day = inner.day.entry(qk.clone()).or_default();
        day.day_reqs += 1;
        day.day_tokens += tokens;
        let min = inner.minute.entry(qk.clone()).or_default();
        min.minute_reqs += 1;
        min.minute_tokens += tokens;
        let persisted = PersistedState {
            utc_day: inner.utc_day,
            counters: inner.day.clone(),
        };
        let _ = write_persisted(&self.state_path, &persisted);
    }

    /// Record a 429, parsing reset headers (MINIMUM-wins priority).
    /// Returns the cooldown seconds that was applied.
    pub fn note_429(&self, key: &KeyId, model: &str, headers: &[(String, String)]) -> u64 {
        let cooldown_secs = parse_429_cooldown(headers);
        let mut inner = self.inner.lock().expect("quota mutex poisoned");
        self.roll_day(&mut inner);
        self.roll_minute(&mut inner);
        let now_secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let qk = quota_key(key, model);
        inner.cooldowns.insert(
            qk,
            Cooldown {
                expires_at: now_secs.saturating_add(cooldown_secs),
            },
        );
        cooldown_secs
    }

    /// Soonest reset across all active cooldowns + day/minute boundaries.
    /// Returns Duration until the soonest reset, or None if no limits active.
    pub fn soonest_reset(&self) -> Option<Duration> {
        let inner = self.inner.lock().expect("quota mutex poisoned");
        let now_secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let now_min = utc_minute(SystemTime::now());
        let mut soonest: Option<u64> = None;
        for cd in inner.cooldowns.values() {
            if cd.expires_at > now_secs {
                let wait = cd.expires_at - now_secs;
                soonest = Some(soonest.map_or(wait, |s| s.min(wait)));
            }
        }
        let next_minute = (now_min + 1) * 60;
        if next_minute > now_secs {
            let wait = next_minute - now_secs;
            soonest = Some(soonest.map_or(wait, |s| s.min(wait)));
        }
        let next_day = (inner.utc_day + 1) * 86_400;
        if next_day > now_secs {
            let wait = next_day - now_secs;
            soonest = Some(soonest.map_or(wait, |s| s.min(wait)));
        }
        soonest.map(Duration::from_secs)
    }

    /// Persist day counters to disk (atomic tmp+rename).
    pub fn persist(&self) -> std::io::Result<()> {
        let inner = self.inner.lock().expect("quota mutex poisoned");
        let persisted = PersistedState {
            utc_day: inner.utc_day,
            counters: inner.day.clone(),
        };
        write_persisted(&self.state_path, &persisted)
    }
}

/// Parse 429 cooldown from headers. Priority (MINIMUM-wins):
/// x-ratelimit-reset-requests > Retry-After > x-ratelimit-reset-tokens > 60s default.
fn parse_429_cooldown(headers: &[(String, String)]) -> u64 {
    let mut candidates: Vec<u64> = Vec::new();
    for (name, value) in headers {
        let lower = name.to_ascii_lowercase();
        if lower == "x-ratelimit-reset-requests" {
            if let Some(secs) = parse_reset_value(value) {
                candidates.push(secs);
            }
        }
    }
    for (name, value) in headers {
        let lower = name.to_ascii_lowercase();
        if lower == "retry-after" {
            if let Ok(secs) = value.trim().parse::<u64>() {
                candidates.push(secs);
            }
        }
    }
    for (name, value) in headers {
        let lower = name.to_ascii_lowercase();
        if lower == "x-ratelimit-reset-tokens" {
            if let Some(secs) = parse_reset_value(value) {
                candidates.push(secs);
            }
        }
    }
    candidates.into_iter().min().unwrap_or(60)
}

/// Parse a reset header value. Accepts plain seconds or "Xs" / "Xms" / "Xd" suffixes.
fn parse_reset_value(value: &str) -> Option<u64> {
    let trimmed = value.trim().to_ascii_lowercase();
    if let Ok(secs) = trimmed.parse::<u64>() {
        return Some(secs);
    }
    if let Some(rest) = trimmed.strip_suffix("ms") {
        let ms: u64 = rest.parse().ok()?;
        return Some(ms / 1000);
    }
    if let Some(rest) = trimmed.strip_suffix('s') {
        return rest.parse().ok();
    }
    if let Some(rest) = trimmed.strip_suffix('m') {
        let mins: u64 = rest.parse().ok()?;
        return Some(mins * 60);
    }
    if let Some(rest) = trimmed.strip_suffix('h') {
        let hours: u64 = rest.parse().ok()?;
        return Some(hours * 3600);
    }
    if let Some(rest) = trimmed.strip_suffix('d') {
        let days: u64 = rest.parse().ok()?;
        return Some(days * 86_400);
    }
    None
}

fn read_persisted(path: &PathBuf) -> std::io::Result<PersistedState> {
    let data = std::fs::read_to_string(path)?;
    serde_json::from_str(&data).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
}

fn write_persisted(path: &PathBuf, state: &PersistedState) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let data = serde_json::to_string_pretty(state)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, &data)?;
    std::fs::rename(&tmp, path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    fn tmp_state_path() -> PathBuf {
        let dir = std::env::temp_dir();
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        dir.join(format!("quota_test_{nanos}.json"))
    }

    #[test]
    fn key_id_never_stores_raw() {
        let raw = "sk-test-12345-secret-key";
        let key = KeyId::from_raw_key("groq", raw);
        assert!(!key.fp.contains(raw));
        assert_eq!(key.fp.len(), 8);
        assert_eq!(key.provider, "groq");
    }

    #[test]
    fn key_id_blake3_not_sha() {
        let key = KeyId::from_raw_key("test", "mykey");
        assert!(key.fp.chars().all(|c| c.is_ascii_hexdigit()));
        assert_eq!(key.fp.len(), 8);
    }

    #[test]
    fn day_persists_minute_does_not() {
        let path = tmp_state_path();
        {
            let tracker = QuotaTracker::unlimited(path.clone());
            let key = KeyId::from_raw_key("groq", "sk-test");
            tracker.record_success(&key, "llama-4-scout", 100);
            tracker.persist().ok();
        }
        let data = std::fs::read_to_string(&path).expect("persisted file");
        assert!(data.contains("day_reqs"));
        assert!(data.contains("day_tokens"));
        assert!(!data.contains("minute_reqs"));
        assert!(!data.contains("minute_tokens"));
        assert!(!data.contains("cooldown"));
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn note_429_takes_soonest_header() {
        let path = tmp_state_path();
        let tracker = QuotaTracker::unlimited(path.clone());
        let key = KeyId::from_raw_key("groq", "sk-test");
        let headers = vec![
            ("x-ratelimit-reset-requests".to_string(), "30".to_string()),
            ("Retry-After".to_string(), "90".to_string()),
            ("x-ratelimit-reset-tokens".to_string(), "120".to_string()),
        ];
        let cooldown = tracker.note_429(&key, "llama-4-scout", &headers);
        assert_eq!(cooldown, 30);
        assert!(!tracker.has_headroom(&key, "llama-4-scout"));
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn note_429_default_60_when_no_headers() {
        let path = tmp_state_path();
        let tracker = QuotaTracker::unlimited(path.clone());
        let key = KeyId::from_raw_key("groq", "sk-test");
        let cooldown = tracker.note_429(&key, "llama-4-scout", &[]);
        assert_eq!(cooldown, 60);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn raw_key_not_in_persisted_json() {
        let path = tmp_state_path();
        let raw = "sk-super-secret-abc123";
        {
            let tracker = QuotaTracker::unlimited(path.clone());
            let key = KeyId::from_raw_key("groq", raw);
            tracker.record_success(&key, "llama-4-scout", 50);
            tracker.persist().ok();
        }
        let data = std::fs::read_to_string(&path).expect("persisted file");
        assert!(!data.contains(raw), "raw key must not appear in persisted json");
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn has_headroom_respects_rpd_limit() {
        let path = tmp_state_path();
        let mut limits = HashMap::new();
        limits.insert(
            ("groq".to_string(), "llama-4-scout".to_string()),
            QuotaLimits {
                rpd: 2,
                tpd: u64::MAX,
                rpm: u64::MAX,
                tpm: u64::MAX,
            },
        );
        let tracker = QuotaTracker::new(limits, path.clone());
        let key = KeyId::from_raw_key("groq", "sk-test");
        assert!(tracker.has_headroom(&key, "llama-4-scout"));
        tracker.record_success(&key, "llama-4-scout", 10);
        tracker.record_success(&key, "llama-4-scout", 10);
        assert!(!tracker.has_headroom(&key, "llama-4-scout"));
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn soonest_reset_returns_minimum() {
        let path = tmp_state_path();
        let tracker = QuotaTracker::unlimited(path.clone());
        let key = KeyId::from_raw_key("groq", "sk-test");
        let headers = vec![("Retry-After".to_string(), "5".to_string())];
        tracker.note_429(&key, "llama-4-scout", &headers);
        let reset = tracker.soonest_reset().expect("should have a reset");
        assert!(reset.as_secs() <= 5);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn key_id_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<KeyId>();
        assert_send_sync::<Arc<QuotaTracker>>();
    }
}
