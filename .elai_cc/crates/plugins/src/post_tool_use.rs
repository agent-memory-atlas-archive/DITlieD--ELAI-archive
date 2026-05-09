//! B-19A (T6) — ruff PostToolUse subprocess hook.
//!
//! Spec: docs/thefinalglance/audit-implementation/phase-4C.md item 4 — "ruff
//! syntax check on file_write (subprocess, ~50-200ms). Circuit breaker: skip
//! on files >10000 lines." Adds the missing rust-side glue between the hook
//! dispatcher and the `ruff` binary so PostToolUse runs lint+syntax on
//! agent-written Python sources.
//!
//! Detection strategy (cross-platform, no new deps):
//!   1. Try `Command::new("ruff").arg("--version")`.
//!   2. On `ErrorKind::NotFound` (and Windows) retry with `"ruff.exe"`.
//!   3. If both fail, the runner returns `RuffOutcome::NoOp { reason:
//!      BinaryMissing }`, emits ONE `tracing::warn` for the lifetime of
//!      the process (subsequent calls stay silent via `std::sync::Once`),
//!      and never errors.
//!
//! Cache: keyed on `blake3(file_content)`. Bounded BTreeMap with 256-entry
//! cap (FIFO eviction by insertion order) — `lru` crate is not in tree per
//! workspace deps, and CLAUDE.md bans new dependencies without confirmation.
//! BTreeMap is NOT std HashMap so it satisfies the "no std HashMap" rule.
//!
//! Timeout: 5 seconds via `tokio::time::timeout` per spec. Overruns return
//! `RuffOutcome::Timeout`.

use std::collections::BTreeMap;
use std::io::ErrorKind;
use std::process::Stdio;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, Once};

use blake3::Hasher;
use elai_core::clock::{ClockProvider, OsClock};
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use tokio::time::{timeout, Duration};
use tracing::warn;

/// Default timeout for a single ruff invocation, in milliseconds.
/// Source: B-19 plan §"T6 ruff PostToolUse" — 5 seconds. Anything beyond
/// indicates a stuck binary (ruff itself is single-digit ms on small files).
pub const RUFF_DEFAULT_TIMEOUT_MS: u64 = 5_000;

/// Cap on the in-process result cache. After this many distinct file
/// content hashes, oldest insertion is evicted on the next write.
pub const RUFF_CACHE_MAX_ENTRIES: usize = 256;

/// Outcome of a single ruff PostToolUse run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuffOutcome {
    /// Ruff was unavailable, the file was outside scope, or the line cap
    /// circuit breaker fired. Carries the reason so the caller can render
    /// the appropriate dashboard / log breadcrumb.
    NoOp { reason: RuffSkipReason },
    /// Ruff ran and produced findings. `exit_code` follows ruff's contract
    /// (0 = clean, 1 = findings produced, other = ruff internal error).
    Findings {
        exit_code: i32,
        findings: Vec<RuffFinding>,
        elapsed_ms: u64,
    },
    /// Ruff exceeded the timeout window.
    Timeout { elapsed_ms: u64 },
}

/// Why a ruff invocation was skipped without producing findings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuffSkipReason {
    /// Neither `ruff` nor `ruff.exe` was found on PATH at startup.
    BinaryMissing,
    /// File extension is not `.py` / `.pyi`.
    NonPythonFile,
    /// File exceeded `POST_TOOL_USE_LINE_LIMIT` (10_000 lines).
    LineLimitExceeded,
    /// File path was empty.
    EmptyPath,
}

/// One finding parsed out of ruff's text output. We do not call ruff with
/// `--output-format=json` because that requires a newer ruff version we
/// can't pin without a workspace dep change; instead we record the raw
/// stderr/stdout lines so the caller can surface them to the dashboard.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuffFinding {
    /// Severity tier — ruff reports "error" / "warning" via prefixes; we
    /// classify on best-effort and default to "warning" if ambiguous.
    pub severity: String,
    /// File path as ruff reported it.
    pub file_path: String,
    /// 1-based line number.
    pub line: u32,
    /// 1-based column number.
    pub column: u32,
    /// Ruff rule code (e.g. "E501", "F401"). Empty if not parseable.
    pub rule_code: String,
    /// Human-readable description.
    pub message: String,
}

/// Internal cache entry. `insertion_seq` is used for FIFO eviction; we
/// drop the entry with the smallest seq when the cache is at cap.
#[derive(Debug, Clone)]
struct CacheEntry {
    outcome: RuffOutcome,
    insertion_seq: u64,
}

/// Ruff PostToolUse runner — owns the binary detection + result cache.
///
/// Construct via `RuffPostToolUseRunner::new()`. Multiple instances within
/// the same process share nothing; tests construct fresh runners per test.
///
/// B-19A cycle 2 (BLOCK-3): all timing now flows through an injected
/// `Arc<dyn ClockProvider>` per the entropy quarantine rule. Production
/// callers get [`OsClock`]; tests inject [`elai_core::clock::SeededClock`].
pub struct RuffPostToolUseRunner {
    /// Detected binary name, or `None` when ruff is missing on PATH.
    binary: Option<String>,
    /// `true` once `BinaryMissing` was warned about — silences subsequent
    /// invocations to stop log spam in long-running agents.
    warn_once: Once,
    /// Tracking flag for tests: `Once` itself is opaque so we mirror its
    /// state into an atomic the test harness can query.
    warn_once_fired: std::sync::atomic::AtomicBool,
    /// Cache hit counter. Incremented on every cache-served call.
    cache_hits: AtomicU64,
    /// Result cache keyed by blake3 of file content. Bounded at
    /// RUFF_CACHE_MAX_ENTRIES via FIFO eviction.
    cache: Mutex<BTreeMap<[u8; 32], CacheEntry>>,
    /// Monotonic insertion counter for FIFO eviction ordering.
    insertion_counter: AtomicU64,
    /// Injected monotonic clock used to bound `elapsed_ms` reporting.
    /// Defaults to [`OsClock`] when constructed via [`Self::new`]; tests
    /// pass a [`elai_core::clock::SeededClock`] via [`Self::with_clock`].
    clock: Arc<dyn ClockProvider>,
}

impl std::fmt::Debug for RuffPostToolUseRunner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RuffPostToolUseRunner")
            .field("binary", &self.binary)
            .field("warn_once_fired", &self.warn_once_fired)
            .field("cache_hits", &self.cache_hits)
            .field("cache_len", &self.cache_len())
            .finish_non_exhaustive()
    }
}

impl Default for RuffPostToolUseRunner {
    fn default() -> Self {
        Self::new()
    }
}

impl RuffPostToolUseRunner {
    /// Construct a new runner. Probes for the ruff binary up-front via
    /// `--version`; the result is cached for the runner's lifetime so
    /// subsequent `run` calls don't re-probe on every invocation.
    ///
    /// Production timing uses [`OsClock`]. For deterministic tests, build
    /// via [`Self::with_clock`] passing a [`elai_core::clock::SeededClock`].
    #[must_use]
    pub fn new() -> Self {
        Self::with_clock(Arc::new(OsClock))
    }

    /// Construct a runner with an injected [`ClockProvider`]. Used by tests
    /// to swap in [`elai_core::clock::SeededClock`] for deterministic
    /// `elapsed_ms` values.
    #[must_use]
    pub fn with_clock(clock: Arc<dyn ClockProvider>) -> Self {
        let binary = detect_ruff_binary();
        Self {
            binary,
            warn_once: Once::new(),
            warn_once_fired: std::sync::atomic::AtomicBool::new(false),
            cache_hits: AtomicU64::new(0),
            cache: Mutex::new(BTreeMap::new()),
            insertion_counter: AtomicU64::new(0),
            clock,
        }
    }

    /// Returns `true` once the BinaryMissing warning has fired.
    #[must_use]
    pub fn has_warned_once(&self) -> bool {
        self.warn_once_fired.load(Ordering::SeqCst)
    }

    /// Cache hit count for this runner's lifetime.
    #[must_use]
    pub fn cache_hits(&self) -> u64 {
        self.cache_hits.load(Ordering::SeqCst)
    }

    /// Current cache entry count. Useful for tests; production callers
    /// should not depend on a precise number.
    #[must_use]
    pub fn cache_len(&self) -> usize {
        self.cache.lock().map(|c| c.len()).unwrap_or(0)
    }

    /// Run ruff on `file_path` with the given file contents. Returns a
    /// [`RuffOutcome`] describing what happened.
    ///
    /// Errors propagate only when subprocess plumbing fails in ways that
    /// indicate a runtime bug (not "ruff missing" — that's NoOp). Spec:
    /// callers MUST treat `Err` as "skip + log" rather than block the
    /// agent's PostToolUse path.
    pub async fn run(
        &self,
        file_path: &str,
        content: &str,
    ) -> Result<RuffOutcome, RuffRunnerError> {
        if file_path.is_empty() {
            return Ok(RuffOutcome::NoOp {
                reason: RuffSkipReason::EmptyPath,
            });
        }
        // Out-of-scope file types — Python only.
        let lower = file_path.to_ascii_lowercase();
        if !(lower.ends_with(".py") || lower.ends_with(".pyi")) {
            return Ok(RuffOutcome::NoOp {
                reason: RuffSkipReason::NonPythonFile,
            });
        }
        // Circuit-breaker: very large files skip per ipc_hooks.rs:31
        // (POST_TOOL_USE_LINE_LIMIT = 10_000).
        const POST_TOOL_USE_LINE_LIMIT: usize = 10_000;
        if content.lines().count() > POST_TOOL_USE_LINE_LIMIT {
            return Ok(RuffOutcome::NoOp {
                reason: RuffSkipReason::LineLimitExceeded,
            });
        }

        // Binary missing? Warn ONCE per process and return NoOp.
        let Some(binary) = self.binary.as_deref() else {
            self.warn_once.call_once(|| {
                warn!(
                    "ruff PostToolUse: binary not found on PATH — Python lint disabled. \
                     Install via `pip install ruff` or `uv pip install ruff`."
                );
                self.warn_once_fired.store(true, Ordering::SeqCst);
            });
            return Ok(RuffOutcome::NoOp {
                reason: RuffSkipReason::BinaryMissing,
            });
        };

        // Cache lookup by blake3(content).
        let key = blake3_hash_bytes(content);
        if let Ok(cache) = self.cache.lock() {
            if let Some(entry) = cache.get(&key) {
                self.cache_hits.fetch_add(1, Ordering::SeqCst);
                return Ok(entry.outcome.clone());
            }
        }

        // B-19A cycle 2 (BLOCK-1): we MUST lint the in-memory `content` the
        // hook just observed, NOT whatever happens to be on disk. Disk state
        // is racy (the file may not exist yet at PostToolUse time, or may
        // have been overwritten by a concurrent writer), and a missing file
        // produces ruff's E902 error which we'd otherwise cache as a
        // "valid Findings" entry — a correctness bug.
        //
        // Fix: invoke `ruff check --stdin-filename <path> -` and pipe
        // `content` to ruff's stdin. The `-` arg signals "read source from
        // stdin"; `--stdin-filename` carries the logical path so ruff's
        // findings still reference the real file. This was added in
        // ruff 0.1+ (verified at https://docs.astral.sh/ruff/configuration/
        // and `ruff check --help`).
        //
        // Timing uses the injected ClockProvider per BLOCK-3 (entropy
        // quarantine: no `tokio::time::Instant::now` in production code).
        let started = self.clock.now();
        let mut cmd = Command::new(binary);
        cmd.arg("check")
            .arg("--no-cache")
            .arg("--exit-zero")
            .arg("--stdin-filename")
            .arg(file_path)
            .arg("-")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = cmd
            .spawn()
            .map_err(|e| RuffRunnerError::Spawn(format!("{e}")))?;
        // Take stdin BEFORE awaiting so the child sees EOF after the write.
        // Failing to capture stdin here would mean we configured `Stdio::piped()`
        // but the child handle never grabbed it — a wiring bug, not a runtime
        // error, hence `Spawn` rather than a new variant.
        let mut child_stdin = child
            .stdin
            .take()
            .ok_or_else(|| RuffRunnerError::Spawn("ruff child stdin unavailable".to_string()))?;
        // Write the source bytes, then drop stdin so ruff observes EOF.
        // Both write and shutdown errors propagate as Spawn to match the
        // existing error-shape; they only fire on plumbing failure.
        if let Err(e) = child_stdin.write_all(content.as_bytes()).await {
            return Err(RuffRunnerError::Spawn(format!("ruff stdin write: {e}")));
        }
        if let Err(e) = child_stdin.shutdown().await {
            return Err(RuffRunnerError::Spawn(format!("ruff stdin close: {e}")));
        }
        drop(child_stdin);

        let timeout_dur = Duration::from_millis(RUFF_DEFAULT_TIMEOUT_MS);
        let output = match timeout(timeout_dur, child.wait_with_output()).await {
            Ok(Ok(out)) => out,
            Ok(Err(e)) => return Err(RuffRunnerError::Spawn(format!("{e}"))),
            Err(_) => {
                // NARROWING-OK: clamped to u64::MAX before cast; u128->u64 is safe here.
                let elapsed_ms =
                    self.clock.elapsed_since(started).as_millis().min(u128::from(u64::MAX)) as u64;
                let outcome = RuffOutcome::Timeout { elapsed_ms };
                self.insert_cache(key, outcome.clone());
                return Ok(outcome);
            }
        };

        // NARROWING-OK: clamped to u64::MAX before cast; u128->u64 is safe here.
        let elapsed_ms =
            self.clock.elapsed_since(started).as_millis().min(u128::from(u64::MAX)) as u64;
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let findings = parse_ruff_output(&stdout, &stderr);
        let exit_code = output.status.code().unwrap_or(-1);
        let outcome = RuffOutcome::Findings {
            exit_code,
            findings,
            elapsed_ms,
        };
        self.insert_cache(key, outcome.clone());
        Ok(outcome)
    }

    fn insert_cache(&self, key: [u8; 32], outcome: RuffOutcome) {
        let seq = self.insertion_counter.fetch_add(1, Ordering::SeqCst);
        let Ok(mut cache) = self.cache.lock() else {
            return;
        };
        // FIFO eviction when at cap: drop entry with smallest insertion_seq.
        if cache.len() >= RUFF_CACHE_MAX_ENTRIES {
            let oldest_key = cache
                .iter()
                .min_by_key(|(_, v)| v.insertion_seq)
                .map(|(k, _)| *k);
            if let Some(k) = oldest_key {
                cache.remove(&k);
            }
        }
        cache.insert(
            key,
            CacheEntry {
                outcome,
                insertion_seq: seq,
            },
        );
    }

    // ---- Test seams (cfg(any) so they're available to integration tests) ----

    /// Test seam — directly insert a synthetic cache entry without spawning
    /// a subprocess. Public so integration tests in `tests/` can drive the
    /// LRU bookkeeping without a real ruff binary.
    pub fn test_cache_insert(&self, key: [u8; 32], outcome: RuffOutcome) {
        self.insert_cache(key, outcome);
    }
}

/// Errors that can occur in the ruff runner. These are runtime bugs in the
/// subprocess plumbing — the missing-binary path is NOT an error.
#[derive(Debug, thiserror::Error)]
pub enum RuffRunnerError {
    #[error("failed to spawn ruff subprocess: {0}")]
    Spawn(String),
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Probe for a ruff binary on PATH. Returns the binary name to invoke.
fn detect_ruff_binary() -> Option<String> {
    // Try the standard names. On Windows, prefer `ruff.exe` because PATHEXT
    // resolution is more deterministic with the explicit suffix.
    #[cfg(windows)]
    let candidates: &[&str] = &["ruff.exe", "ruff"];
    #[cfg(not(windows))]
    let candidates: &[&str] = &["ruff"];

    for name in candidates {
        match std::process::Command::new(name)
            .arg("--version")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
        {
            Ok(_) => return Some((*name).to_string()),
            Err(e) if e.kind() == ErrorKind::NotFound => continue,
            Err(_) => continue,
        }
    }
    None
}

/// Compute blake3-256 of UTF-8 content. Returns the 32-byte digest.
fn blake3_hash_bytes(content: &str) -> [u8; 32] {
    let mut hasher = Hasher::new();
    hasher.update(content.as_bytes());
    *hasher.finalize().as_bytes()
}

/// Parse ruff's textual output into structured findings. Best-effort:
/// ruff's default text format is `<file>:<line>:<col>: <code> <message>`.
fn parse_ruff_output(stdout: &str, stderr: &str) -> Vec<RuffFinding> {
    let combined = if stdout.is_empty() { stderr } else { stdout };
    combined
        .lines()
        .filter_map(|line| {
            // Format: path:line:col: CODE message
            let mut parts = line.splitn(4, ':');
            let path = parts.next()?.trim();
            let line_no = parts.next()?.trim().parse::<u32>().ok()?;
            let col_no = parts.next()?.trim().parse::<u32>().ok()?;
            let rest = parts.next()?.trim();
            // rest is "CODE message"
            let mut code_msg = rest.splitn(2, ' ');
            let rule_code = code_msg.next().unwrap_or("").to_string();
            let message = code_msg.next().unwrap_or("").to_string();
            // Severity classification: anything starting with E or F is error,
            // W is warning. Default to warning when ambiguous.
            let severity = match rule_code.chars().next() {
                Some('E') | Some('F') => "error".to_string(),
                _ => "warning".to_string(),
            };
            Some(RuffFinding {
                severity,
                file_path: path.to_string(),
                line: line_no,
                column: col_no,
                rule_code,
                message,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_ruff_output_handles_standard_format() {
        let stdout = "src/foo.py:5:1: E501 line too long (88 > 79)\n\
                      src/foo.py:7:3: F401 'os' imported but unused";
        let findings = parse_ruff_output(stdout, "");
        assert_eq!(findings.len(), 2);
        assert_eq!(findings[0].rule_code, "E501");
        assert_eq!(findings[0].line, 5);
        assert_eq!(findings[0].column, 1);
        assert_eq!(findings[0].severity, "error");
        assert_eq!(findings[1].rule_code, "F401");
    }

    #[test]
    fn parse_ruff_output_handles_no_findings() {
        let findings = parse_ruff_output("", "");
        assert!(findings.is_empty());
    }

    #[test]
    fn blake3_is_deterministic() {
        let a = blake3_hash_bytes("hello");
        let b = blake3_hash_bytes("hello");
        assert_eq!(a, b);
        let c = blake3_hash_bytes("hello!");
        assert_ne!(a, c);
    }

    #[test]
    fn timeout_constant_is_5_seconds() {
        assert_eq!(RUFF_DEFAULT_TIMEOUT_MS, 5_000);
    }
}
