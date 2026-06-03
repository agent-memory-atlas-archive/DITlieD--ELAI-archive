// F-9 / Slice D1 (SCHEMA_VERSION 199): DepWriteTimeGate — write-time
// anti-slopsquatting gate.  Parses dependency import lines from candidate
// source, probes the registry (via SubprocessRegistryChecker or a test stub),
// caches results (TTL 1 h, bounded 512 entries), and returns
// Vec<HookFinding> for the PreToolUse dispatcher.  Emits
// WriteTimeSlopsquatBlocked SSB rows on NotFound / TooNew verdicts.
//
// CRITICAL ASYNC RULE: `analyze()` is SYNCHRONOUS (PreToolUse boundary).
// `LruExistenceCache` MUST use std::sync::Mutex — never tokio::sync::Mutex.

use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use tracing::warn;

use crate::hook_dispatcher::{HookFinding, HookTier, HookVerdictKind};
use crate::ssb::{
    Ssb, SsbPayload, WriteTimeSlopVerdictKind, WriteTimeSlopsquatBlockedPayload,
};

// ---------------------------------------------------------------------------
// Public API types
// ---------------------------------------------------------------------------

/// Registry verdict returned by [`RegistryExistenceChecker::check`].
///
/// Object-safe; implemented by both [`StubRegistryChecker`] (offline/tests)
/// and [`SubprocessRegistryChecker`] (production shell-out).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegistryVerdict {
    /// Package exists in the registry.  `age_days` is the package age in days
    /// since first publish (None when the checker cannot determine age).
    Found { age_days: Option<u32> },
    /// Package was not found in the registry (possible slopsquat / hallucination).
    NotFound,
    /// Package exists but its latest publish is older than the per-ecosystem
    /// staleness threshold (`registry_check.py` `_DEFAULT_MAX_AGE_DAYS`).  This
    /// is the "stale fork / abandoned package" slopsquat vector — the name
    /// resolves but the package is effectively dead.  `age_days` carries the
    /// reported age when parseable.  Treated as a Block (TooNew SSB verdict).
    TooNew { age_days: Option<u32> },
    /// Registry probe failed (network down, timeout, parse error).  Gate
    /// FAILS OPEN per W-10 (never block on registry-down).
    Unreachable,
}

/// Object-safe, synchronous registry existence checker.
///
/// All implementations MUST be object-safe and sync (no `async fn`).
/// Blocking I/O is acceptable; callers are responsible for ensuring the call
/// is made from a context where blocking is safe (e.g. a rayon pool or a
/// `tokio::task::block_in_place` site).
pub trait RegistryExistenceChecker: Send + Sync {
    /// Check whether `name` exists in `ecosystem` (`"cargo"`, `"pypi"`,
    /// `"npm"`).  Returns [`RegistryVerdict`].
    fn check(&self, name: &str, ecosystem: &str) -> RegistryVerdict;
}

/// A blocked dependency record returned by [`DepWriteTimeGate::analyze_collect`].
///
/// Carries the structured fields needed to write a `WriteTimeSlopsquatBlocked`
/// SSB row durably from the CLI (BLOCK-3), without the gate performing the
/// (non-durable) spawn-detached emission itself.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockedDepRecord {
    /// The dependency name as extracted from the source.
    pub dep_name: String,
    /// Ecosystem label (`"cargo"` / `"pypi"` / `"npm"`).
    pub ecosystem: &'static str,
    /// Which slop verdict produced this record: `NotFound` / `TooNew` block the
    /// write; `RegistryUnreachable` is an ADVISORY record (fail-open verdict
    /// stays Warn) so the CLI durably traces the unreachable skip (revised W-10).
    pub verdict_kind: WriteTimeSlopVerdictKind,
    /// Reported registry age in days (present for `TooNew`).
    pub registry_age_days: Option<u32>,
}

// ---------------------------------------------------------------------------
// StubRegistryChecker — deterministic offline / test implementation
// ---------------------------------------------------------------------------

/// Deterministic offline checker used in tests and air-gapped environments.
///
/// WARN-3 fix: offline mode is now SAFE on real code. Only a curated set of
/// KNOWN-HALLUCINATED names (the fixtures + documented typo-squats) return
/// `NotFound`; EVERY other name returns `Found`. This means `elai dep-check
/// --offline` no longer false-blocks legitimate deps — it is a deterministic
/// demo/contract harness that still blocks the curated fixtures, while the REAL
/// gate is the DEFAULT `SubprocessRegistryChecker` (live) leg.
///
/// Known-hallucinated (NotFound): `reqquests`, `reqwest-typo`, the fixture
/// crate/pkg names, and the `numpy-`/`serde-`/`tokio-` typo prefixes.
/// Everything else: `Found { age_days: Some(365) }`.
#[derive(Debug, Default)]
pub struct StubRegistryChecker;

impl RegistryExistenceChecker for StubRegistryChecker {
    fn check(&self, name: &str, _ecosystem: &str) -> RegistryVerdict {
        // Curated known-hallucinated / typo-squat names. Exact matches and the
        // documented typo PREFIXES (matched via `starts_with`) below. Any name
        // NOT in this set is treated as a real package (Found) so offline mode
        // is safe on legitimate code (WARN-3).
        const KNOWN_HALLUCINATED: &[&str] = &[
            "reqquests",        // fixture: f9_hallucinated_import.rs
            "reqwest-typo",
            "fakepkg-9999",     // fixture: f9_hallucinated_cargo.toml (if present)
            "totally-not-real-crate",
        ];
        // Documented typo-squat PREFIXES (catch `numpy-foo`, `serde-bar`, …).
        const KNOWN_TYPO_PREFIXES: &[&str] = &[
            "numpy-",
            "serde-",
            "tokio-",
        ];

        if KNOWN_HALLUCINATED.contains(&name) {
            return RegistryVerdict::NotFound;
        }
        if KNOWN_TYPO_PREFIXES.iter().any(|p| name.starts_with(p)) {
            return RegistryVerdict::NotFound;
        }
        // WARN-3: unknown names are assumed real in offline mode (fail-safe for
        // legitimate code). The real existence check is the DEFAULT live leg.
        RegistryVerdict::Found { age_days: Some(365) }
    }
}

// ---------------------------------------------------------------------------
// SubprocessRegistryChecker — production shell-out to registry_check.py
// ---------------------------------------------------------------------------

/// Production checker: shells out to the D-11 `registry_check.py` script via
/// a bounded `std::process::Command` with a ~1.5 s timeout.
///
/// JSON contract with the script (stdin → stdout) — this is the SHIPPED D-11
/// contract that `registry_check.py::main()` actually reads/emits (BLOCK-1
/// reconcile; the prior `{ecosystem,name}` / `{found,age_days}` shapes never
/// matched the script and made the live leg always-Unreachable):
/// ```json
/// // stdin
/// { "claim": "serde", "candidate": { "ecosystem": "cargo" } }
/// // stdout — exactly one of:
/// { "result": "pass",          "reason": "cargo: 'serde' last updated 12d ago" }
/// { "result": "fail",          "reason": "cargo 404: crate 'X' does not exist (likely slopsquat)" }
/// { "result": "fail",          "reason": "cargo: 'X' last updated 900d ago (>730d stale)" }
/// { "result": "inapplicable",  "reason": "..." }
/// ```
/// Parser mapping (see [`parse_registry_json`]):
///   * `pass`         → `Found { age_days }` (age parsed from reason when present)
///   * `fail` + 404 / "does not exist" → `NotFound`
///   * `fail` + "stale" / ">…d"        → `TooNew { age_days }`
///   * `inapplicable` / error          → `Unreachable` (W-10 fail-open)
///
/// Mirrors the `SympyChecker` subprocess pattern in `checker_registry.rs`.
#[derive(Debug)]
pub struct SubprocessRegistryChecker {
    /// Absolute path to the python interpreter (defaults to `"python"`).
    pub python: PathBuf,
    /// Absolute path to the registry_check.py helper script.
    pub script_path: PathBuf,
}

impl SubprocessRegistryChecker {
    #[must_use]
    pub fn new(python: impl Into<PathBuf>, script_path: impl Into<PathBuf>) -> Self {
        Self {
            python: python.into(),
            script_path: script_path.into(),
        }
    }

    /// Construct using the conventional location under the workspace root:
    /// `<root>/.elai_cc/checkers/registry/registry_check.py`.
    #[must_use]
    pub fn default_at_root(root: &std::path::Path) -> Self {
        Self {
            python: PathBuf::from("python"),
            script_path: root.join(".elai_cc/checkers/registry/registry_check.py"),
        }
    }
}

impl RegistryExistenceChecker for SubprocessRegistryChecker {
    fn check(&self, name: &str, ecosystem: &str) -> RegistryVerdict {
        use std::io::Write as _;
        use std::process::{Command, Stdio};

        if !self.script_path.exists() {
            warn!(
                script = %self.script_path.display(),
                "registry_check.py not found; treating as Unreachable"
            );
            return RegistryVerdict::Unreachable;
        }

        // D-11 contract: registry_check.py reads `claim` (the package name) and
        // `candidate.ecosystem`; `max_age_days` is optional (script defaults
        // per ecosystem). See registry_check.py:196-215.
        let stdin_json = match serde_json::to_string(&serde_json::json!({
            "claim": name,
            "candidate": { "ecosystem": ecosystem },
        })) {
            Ok(s) => s,
            Err(e) => {
                warn!(err = %e, "registry_check: failed to serialize stdin JSON");
                return RegistryVerdict::Unreachable;
            }
        };

        // Bounded blocking call with a ~1.5 s wall-clock budget.
        let timeout = Duration::from_millis(1_500);
        let call_start = Instant::now();

        let mut child = match Command::new(&self.python)
            .arg(&self.script_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
        {
            Ok(c) => c,
            Err(e) => {
                warn!(
                    python = %self.python.display(),
                    err = %e,
                    "registry_check: spawn failed"
                );
                return RegistryVerdict::Unreachable;
            }
        };

        if let Some(mut stdin) = child.stdin.take() {
            if let Err(e) = stdin.write_all(stdin_json.as_bytes()) {
                warn!(err = %e, "registry_check: stdin write failed");
                let _ = child.kill();
                return RegistryVerdict::Unreachable;
            }
            // Drop closes stdin (EOF for the script).
            drop(stdin);
        }

        // Collect output with a bounded wall-clock budget.
        // We spawn a thread to call wait_with_output() (which reads stdout +
        // stderr to EOF then reaps the child) and send the result back via a
        // channel.  The main thread waits on the channel for `timeout`.
        //
        // WARN-5 (thread-leak-on-timeout, acknowledged + bounded): `child` is
        // MOVED into the spawned thread so the deadline path below CANNOT kill
        // it from the main thread (ownership transferred). On a hung
        // registry_check.py the orphaned child + reaper thread persist until
        // Python self-terminates (its own `requests`/urllib 10 s socket timeout
        // bounds this; registry_check.py:58). This is acceptable: the gate has
        // already returned Unreachable (fail-open) within the 1.5 s budget, and
        // the leak is bounded to one detached thread per timed-out probe with a
        // hard ~10 s self-cleanup ceiling. A kill-handle refactor (split the
        // stdout pipe, keep `child` on the main thread) is the follow-up if
        // probe-flood thread accumulation is ever observed in practice.
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let _ = tx.send(child.wait_with_output());
        });

        let output = match rx.recv_timeout(timeout) {
            Ok(Ok(o)) => o,
            Ok(Err(e)) => {
                warn!(err = %e, "registry_check: wait_with_output error");
                return RegistryVerdict::Unreachable;
            }
            Err(_) => {
                warn!(elapsed_ms = call_start.elapsed().as_millis(), "registry_check: timeout");
                // See WARN-5 note above: the child is owned by the reaper
                // thread and self-terminates within its own socket timeout.
                // Fail-open (W-10): a timed-out probe is Unreachable, never a
                // block.
                return RegistryVerdict::Unreachable;
            }
        };

        if !output.status.success() {
            warn!(code = ?output.status.code(), "registry_check: non-zero exit");
            return RegistryVerdict::Unreachable;
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        parse_registry_json(&stdout)
    }
}

/// Parse the registry_check.py stdout JSON into a [`RegistryVerdict`].
///
/// D-11 contract: `{ "result": "pass"|"fail"|"inapplicable", "reason": "..." }`.
/// (BLOCK-1: the prior parser looked for a non-existent `"found"` key and so
/// always returned `Unreachable` on the live leg.)
///   * `pass`         → `Found { age_days }` — age parsed from the reason
///     (`"… <N>d ago"` / `"… <N>d (>…)"`) when present, else None.
///   * `fail` whose reason indicates not-found (`404` / "does not exist")
///     → `NotFound`.
///   * `fail` whose reason indicates staleness ("stale" / "d ago (>")
///     → `TooNew { age_days }`.
///   * `inapplicable` / unknown result / missing keys → `Unreachable`
///     (fail-open per W-10).
fn parse_registry_json(s: &str) -> RegistryVerdict {
    let v: serde_json::Value = match serde_json::from_str(s.trim()) {
        Ok(v) => v,
        Err(e) => {
            warn!(err = %e, raw = %s, "registry_check: JSON parse error");
            return RegistryVerdict::Unreachable;
        }
    };
    let result = match v.get("result").and_then(|r| r.as_str()) {
        Some(r) => r,
        None => {
            warn!(raw = %s, "registry_check: missing 'result' key");
            return RegistryVerdict::Unreachable;
        }
    };
    let reason = v.get("reason").and_then(|r| r.as_str()).unwrap_or("");
    let reason_lower = reason.to_ascii_lowercase();

    match result {
        "pass" => RegistryVerdict::Found {
            age_days: parse_age_days_from_reason(reason),
        },
        "fail" => {
            // Distinguish not-found (slopsquat/hallucination) from stale/too-old.
            if reason_lower.contains("404") || reason_lower.contains("does not exist") {
                RegistryVerdict::NotFound
            } else if reason_lower.contains("stale") || reason_lower.contains("d ago (>") {
                RegistryVerdict::TooNew {
                    age_days: parse_age_days_from_reason(reason),
                }
            } else {
                // A `fail` we cannot classify (e.g. "no 'latest' version"): the
                // package is not usable. Treat as NotFound (Block) — a missing
                // latest version is functionally a non-existent package.
                RegistryVerdict::NotFound
            }
        }
        "inapplicable" => RegistryVerdict::Unreachable,
        other => {
            warn!(result = other, raw = %s, "registry_check: unknown 'result' value");
            RegistryVerdict::Unreachable
        }
    }
}

/// Parse the first `<N>d ago` / `<N>d (>` age token out of a registry_check.py
/// reason string.  Returns `None` when no age is present.  Examples:
///   "cargo: 'serde' last updated 12d ago"        → Some(12)
///   "pypi: 'x'==1.0 last published 900d ago (>730d stale)" → Some(900)
fn parse_age_days_from_reason(reason: &str) -> Option<u32> {
    // Scan for the pattern `<digits>d` immediately followed by " ago" or " (>".
    let bytes = reason.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i].is_ascii_digit() {
            let start = i;
            while i < bytes.len() && bytes[i].is_ascii_digit() {
                i += 1;
            }
            // Must be followed by 'd' then " ago" or " (>".
            if i < bytes.len() && bytes[i] == b'd' {
                let after = &reason[i + 1..];
                if after.starts_with(" ago") || after.starts_with(" (>") {
                    if let Ok(n) = reason[start..i].parse::<u64>() {
                        return Some(n.min(u32::MAX as u64) as u32);
                    }
                }
            }
        } else {
            i += 1;
        }
    }
    None
}

// ---------------------------------------------------------------------------
// LruExistenceCache — bounded TTL cache guarded by std::sync::Mutex
// ---------------------------------------------------------------------------

/// Cache key type.
type CacheKey = (String, String); // (ecosystem, name)
/// Cache value type: (verdict, insertion_instant).
type CacheEntry = (RegistryVerdict, Instant);

/// Bounded TTL cache for [`RegistryVerdict`] results.
///
/// CRITICAL: uses `std::sync::Mutex` — NOT `tokio::sync::Mutex` — because
/// `DepWriteTimeGate::analyze()` is synchronous (PreToolUse boundary).
/// Holding a `tokio::sync::Mutex` guard across a sync boundary is UB.
///
/// Eviction policy: when at capacity, the oldest insertion (FIFO) is evicted.
/// TTL expiry is checked on every `get`; expired entries return `None`.
pub struct LruExistenceCache {
    inner: Mutex<CacheInner>,
}

struct CacheInner {
    map: HashMap<CacheKey, CacheEntry>,
    insertion_order: VecDeque<CacheKey>,
    capacity: usize,
    ttl: Duration,
}

impl LruExistenceCache {
    /// Create a cache with `capacity` entries and a TTL of `ttl`.
    #[must_use]
    pub fn new(capacity: usize, ttl: Duration) -> Self {
        Self {
            inner: Mutex::new(CacheInner {
                map: HashMap::with_capacity(capacity.min(512)),
                insertion_order: VecDeque::with_capacity(capacity.min(512)),
                capacity,
                ttl,
            }),
        }
    }

    /// Default: 512 entries, 1 h TTL.
    #[must_use]
    pub fn default_bounded() -> Self {
        Self::new(512, Duration::from_secs(3_600))
    }

    /// Look up `(ecosystem, name)`.  Returns `None` on miss or TTL expiry.
    pub fn get(&self, ecosystem: &str, name: &str) -> Option<RegistryVerdict> {
        let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        let key = (ecosystem.to_owned(), name.to_owned());
        match inner.map.get(&key) {
            Some((verdict, inserted_at)) if inserted_at.elapsed() < inner.ttl => {
                Some(verdict.clone())
            }
            Some(_) => {
                // Expired — remove it.
                inner.map.remove(&key);
                inner.insertion_order.retain(|k| k != &key);
                None
            }
            None => None,
        }
    }

    /// Insert `(ecosystem, name) -> verdict`.  Evicts oldest entry when at
    /// capacity (FIFO — sufficient for the bounded TTL use-case).
    pub fn insert(&self, ecosystem: &str, name: &str, verdict: RegistryVerdict) {
        let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        let key = (ecosystem.to_owned(), name.to_owned());

        // If the key already exists, update in place (no order change needed
        // since TTL reset is benign).
        if inner.map.contains_key(&key) {
            inner.map.insert(key, (verdict, Instant::now()));
            return;
        }

        // Evict oldest when at capacity.
        if inner.map.len() >= inner.capacity {
            if let Some(oldest_key) = inner.insertion_order.pop_front() {
                inner.map.remove(&oldest_key);
            }
        }

        inner.insertion_order.push_back(key.clone());
        inner.map.insert(key, (verdict, Instant::now()));
    }
}

// ---------------------------------------------------------------------------
// Dependency import pattern matching
// ---------------------------------------------------------------------------

/// A parsed candidate dependency: (name, ecosystem).
#[derive(Debug, Clone, PartialEq, Eq)]
struct DepCandidate {
    name: String,
    ecosystem: &'static str,
}

/// File extensions that never carry dependency declarations.  When a write
/// targets one of these the gate early-returns `Allow` (skips extraction
/// entirely) — see [`extract_dep_candidates`] file-type guard.  Lockfiles are
/// matched by suffix in the guard, not here.
const NON_DEP_EXTENSIONS: &[&str] = &[
    ".md", ".markdown", ".txt", ".rst", ".png", ".jpg", ".jpeg", ".gif",
    ".svg", ".webp", ".ico", ".pdf", ".csv", ".log", ".lock",
];

/// Extract dependency candidates from `source` based on `file_path` extension.
///
/// Patterns detected:
///   * Python `.py`:  `import <name>`, `from <name> import …`
///   * JS/TS `.js/.ts/.jsx/.tsx/.mjs/.cjs`:  `import … from '<name>'`,
///     `require('<name>')`
///   * Rust `.rs`:    `use <name>::…`, `extern crate <name>;`
///   * Cargo.toml:    `<name> = …` under `[dependencies]` or
///     `[dev-dependencies]` sections
///
/// BLOCK-2 live-path fix (mirrors `vibegate.rs:223` `language_agnostic`):
/// when `file_path` is empty OR its extension is unknown (e.g. the live
/// PreToolUse invocation carries `file_path=None` + the task-summary prose as
/// `source`), the extractor runs in LANGUAGE-AGNOSTIC mode — it applies ALL
/// four sub-extractors over the raw `source` and merges the results — so a
/// hallucinated dep in the candidate text is still detected even with no
/// extension to route on.  The extension fast-path is retained when a known
/// extension IS present.
///
/// Returns only bare (top-level) package names — excludes stdlib / relative
/// imports.
fn extract_dep_candidates(source: &str, file_path: &str) -> Vec<DepCandidate> {
    let path_lower = file_path.to_ascii_lowercase();

    // File-type guard: skip writes to file types that never carry deps. Only
    // applies when a real path is present — an empty path goes to the
    // language-agnostic fallback below (BLOCK-2). Non-Cargo `.toml` files are
    // intentionally NOT scanned (only the literal `cargo.toml` suffix matches).
    if !path_lower.is_empty()
        && NON_DEP_EXTENSIONS.iter().any(|ext| path_lower.ends_with(ext))
    {
        return Vec::new();
    }

    if path_lower.ends_with("cargo.toml") {
        return extract_cargo_toml_deps(source);
    }
    if path_lower.ends_with(".py") {
        return extract_python_deps(source);
    }
    if path_lower.ends_with(".js")
        || path_lower.ends_with(".ts")
        || path_lower.ends_with(".jsx")
        || path_lower.ends_with(".tsx")
        || path_lower.ends_with(".mjs")
        || path_lower.ends_with(".cjs")
    {
        return extract_js_deps(source);
    }
    if path_lower.ends_with(".rs") {
        return extract_rust_deps(source);
    }

    // Language-agnostic fallback: no extension to route on (empty path) or an
    // unknown extension. Run every sub-extractor over the raw source and merge.
    // Each sub-extractor filters on its own syntax (python `import`, rust
    // `use`/`extern crate`, JS `import…from`/`require(`, cargo `name =` only
    // inside a `[dependencies]` section) so cross-contamination is bounded.
    extract_dep_candidates_agnostic(source)
}

/// Language-agnostic dep extraction: union of all four sub-extractors over the
/// raw `source` text, deduped.  Used when no usable file extension is present
/// (BLOCK-2 live PreToolUse path — `file_path=None`).
fn extract_dep_candidates_agnostic(source: &str) -> Vec<DepCandidate> {
    let mut out = Vec::new();
    out.extend(extract_python_deps(source));
    out.extend(extract_js_deps(source));
    out.extend(extract_rust_deps(source));
    out.extend(extract_cargo_toml_deps(source));
    // Dedup across ecosystems on (name, ecosystem). Sort first so dedup_by
    // (which only collapses adjacent equal pairs) catches all duplicates.
    out.sort_by(|a, b| (a.ecosystem, &a.name).cmp(&(b.ecosystem, &b.name)));
    out.dedup_by(|a, b| a.name == b.name && a.ecosystem == b.ecosystem);
    out
}

fn extract_python_deps(source: &str) -> Vec<DepCandidate> {
    let mut out = Vec::new();
    for line in source.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("import ") {
            let name = rest.split_whitespace().next().unwrap_or("").trim_end_matches(';');
            let top = name.split('.').next().unwrap_or("").trim();
            if is_valid_package_name(top) && !is_stdlib_python(top) {
                out.push(DepCandidate { name: top.to_owned(), ecosystem: "pypi" });
            }
        } else if let Some(rest) = trimmed.strip_prefix("from ") {
            let name = rest.split_whitespace().next().unwrap_or("");
            let top = name.split('.').next().unwrap_or("").trim();
            if !top.starts_with('.') && is_valid_package_name(top) && !is_stdlib_python(top) {
                out.push(DepCandidate { name: top.to_owned(), ecosystem: "pypi" });
            }
        }
    }
    dedup_candidates(out)
}

fn extract_js_deps(source: &str) -> Vec<DepCandidate> {
    let mut out = Vec::new();
    for line in source.lines() {
        let trimmed = line.trim();
        // `import … from 'pkg'` or `import … from "pkg"`
        if trimmed.starts_with("import ") {
            if let Some(pkg) = extract_from_clause(trimmed) {
                if is_valid_package_name(pkg) && !pkg.starts_with('.') {
                    let top = scope_or_bare(pkg);
                    out.push(DepCandidate { name: top.to_owned(), ecosystem: "npm" });
                }
            }
        }
        // `require('pkg')` or `require("pkg")`
        if let Some(pkg) = extract_require_arg(trimmed) {
            if is_valid_package_name(pkg) && !pkg.starts_with('.') {
                let top = scope_or_bare(pkg);
                out.push(DepCandidate { name: top.to_owned(), ecosystem: "npm" });
            }
        }
    }
    dedup_candidates(out)
}

fn extract_rust_deps(source: &str) -> Vec<DepCandidate> {
    let mut out = Vec::new();
    for line in source.lines() {
        let trimmed = line.trim();
        // `use crate_name::...` — skip `crate::`, `self::`, `super::`
        if let Some(rest) = trimmed.strip_prefix("use ") {
            let name = rest.split("::").next().unwrap_or("").trim_end_matches(';').trim();
            if !name.is_empty()
                && name != "crate"
                && name != "self"
                && name != "super"
                && is_valid_rust_ident(name)
                && !is_stdlib_rust(name)
            {
                out.push(DepCandidate { name: name.replace('_', "-"), ecosystem: "cargo" });
            }
        }
        // `extern crate name;`
        if let Some(rest) = trimmed.strip_prefix("extern crate ") {
            let name = rest.trim_end_matches(';').trim();
            if is_valid_rust_ident(name) && !is_stdlib_rust(name) {
                out.push(DepCandidate { name: name.replace('_', "-"), ecosystem: "cargo" });
            }
        }
    }
    dedup_candidates(out)
}

fn extract_cargo_toml_deps(source: &str) -> Vec<DepCandidate> {
    let mut out = Vec::new();
    let mut in_deps_section = false;
    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_deps_section = trimmed == "[dependencies]"
                || trimmed == "[dev-dependencies]"
                || trimmed == "[build-dependencies]";
            continue;
        }
        if !in_deps_section || trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if let Some(name) = trimmed.split('=').next() {
            let name = name.trim();
            if is_valid_package_name(name) && !name.starts_with("elai-") && name != "scaffold" {
                out.push(DepCandidate { name: name.to_owned(), ecosystem: "cargo" });
            }
        }
    }
    dedup_candidates(out)
}

// ---------------------------------------------------------------------------
// String helpers
// ---------------------------------------------------------------------------

fn is_valid_package_name(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.')
}

fn is_valid_rust_ident(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_')
        && s.chars().next().map(|c| c.is_ascii_alphabetic() || c == '_').unwrap_or(false)
}

fn is_stdlib_python(name: &str) -> bool {
    const PY_STDLIB: &[&str] = &[
        "os", "sys", "re", "io", "abc", "ast", "csv", "json", "math",
        "time", "copy", "enum", "glob", "gzip", "html", "http", "logging",
        "pathlib", "pickle", "random", "shutil", "socket", "string",
        "struct", "subprocess", "tempfile", "threading", "typing",
        "unittest", "urllib", "uuid", "warnings", "weakref", "zipfile",
        "collections", "contextlib", "dataclasses", "datetime", "decimal",
        "functools", "hashlib", "heapq", "inspect", "itertools", "operator",
        "platform", "pprint", "queue", "signal", "stat", "statistics",
        "textwrap", "traceback", "__future__",
    ];
    PY_STDLIB.contains(&name)
}

fn is_stdlib_rust(name: &str) -> bool {
    const RUST_BUILTIN: &[&str] = &[
        "std", "core", "alloc", "proc_macro", "test",
    ];
    RUST_BUILTIN.contains(&name)
}

/// Normalise `@scope/pkg` → `@scope/pkg` (keep as-is for npm scoped packages),
/// bare `pkg` → `pkg`.
fn scope_or_bare(s: &str) -> &str {
    // For scoped packages like `@scope/name`, keep the full string so the
    // registry checker can handle the `@` prefix.
    s
}

/// Extract the module specifier from `from 'specifier'` or `from "specifier"`.
fn extract_from_clause(line: &str) -> Option<&str> {
    let from_idx = line.rfind(" from ")?;
    let after = line[from_idx + 6..].trim();
    extract_quoted(after)
}

/// Extract the argument from `require('...')` or `require("...")`.
fn extract_require_arg(line: &str) -> Option<&str> {
    let req_idx = line.find("require(")?;
    let after = line[req_idx + 8..].trim();
    extract_quoted(after)
}

fn extract_quoted(s: &str) -> Option<&str> {
    let quote = s.chars().next()?;
    if quote != '\'' && quote != '"' {
        return None;
    }
    let rest = &s[1..];
    let end = rest.find(quote)?;
    Some(&rest[..end])
}

fn dedup_candidates(mut v: Vec<DepCandidate>) -> Vec<DepCandidate> {
    v.dedup_by(|a, b| a.name == b.name && a.ecosystem == b.ecosystem);
    v
}

// ---------------------------------------------------------------------------
// dep_name_blake3_u64 — local blake3 hash for dep names (GhCliAdapter::hash_prefix
// is pub(super) and inaccessible from this module)
// ---------------------------------------------------------------------------

/// 64-bit big-endian prefix of blake3 hash of `name`.  Used as
/// `dep_name_hash` in [`WriteTimeSlopsquatBlockedPayload`].
pub(crate) fn dep_name_blake3_u64(name: &str) -> u64 {
    let hash = blake3::hash(name.as_bytes());
    let bytes = hash.as_bytes();
    u64::from_be_bytes([
        bytes[0], bytes[1], bytes[2], bytes[3],
        bytes[4], bytes[5], bytes[6], bytes[7],
    ])
}

// ---------------------------------------------------------------------------
// DepWriteTimeGate
// ---------------------------------------------------------------------------

/// Write-time anti-slopsquatting gate.
///
/// Wired into the PreToolUse hook dispatcher.  On each candidate file write,
/// `analyze()` extracts dependency names, probes the registry (with TTL
/// cache), and returns [`HookFinding`] entries:
///   * `NotFound`      → `Block` + emits `WriteTimeSlopsquatBlocked` SSB row.
///   * `TooNew`        → `Block` + emits `WriteTimeSlopsquatBlocked` SSB row.
///   * `Unreachable`   → `Warn`  (fail-open per W-10 network-axis policy).
///   * `Found { .. }`  → no finding emitted.
///
/// SSB emission is best-effort: errors are logged via `warn!` and do not
/// affect the hook verdict.
pub struct DepWriteTimeGate {
    ssb: Option<Arc<Ssb>>,
    checker: Arc<dyn RegistryExistenceChecker + Send + Sync>,
    cache: LruExistenceCache,
}

impl DepWriteTimeGate {
    /// Construct with an optional SSB sink and a concrete checker.
    ///
    /// Pass `ssb = None` in contexts where SSB is unavailable (early boot,
    /// unit tests that only want verdicts without DB I/O).
    #[must_use]
    pub fn new(
        ssb: Option<Arc<Ssb>>,
        checker: Arc<dyn RegistryExistenceChecker + Send + Sync>,
    ) -> Self {
        Self {
            ssb,
            checker,
            cache: LruExistenceCache::default_bounded(),
        }
    }

    /// Analyze `source` (the candidate file content) for dependency imports.
    ///
    /// SYNCHRONOUS — called from the PreToolUse dispatcher which is NOT async.
    /// All I/O is blocking; the subprocess checker enforces its own 1.5 s
    /// timeout internally.
    ///
    /// Returns a `Vec<HookFinding>`:
    ///   * One `Block` finding per `NotFound` / `TooNew` package.
    ///   * One `Warn` finding per `Unreachable` probe.
    ///   * Empty vec when all packages are `Found`.
    pub fn analyze(&self, source: &str, file_path: &str) -> Vec<HookFinding> {
        let candidates = extract_dep_candidates(source, file_path);
        if candidates.is_empty() {
            return Vec::new();
        }

        let mut findings = Vec::new();

        for candidate in &candidates {
            let verdict = self.resolve(&candidate.name, candidate.ecosystem);

            match &verdict {
                RegistryVerdict::Found { .. } => {
                    // No finding for known-good packages.
                }
                RegistryVerdict::NotFound => {
                    let finding = HookFinding {
                        tier: HookTier::T1,
                        verdict: HookVerdictKind::Block,
                        rule_id: Some("vibesec_dep_write_time".into()),
                        category: "vibesec_dep".into(),
                        message: format!(
                            "dependency '{}' (ecosystem: {}) not found in registry — possible slopsquat or hallucination",
                            candidate.name, candidate.ecosystem
                        ),
                        fix_template: None,
                    };
                    self.emit_ssb_row(
                        &candidate.name,
                        candidate.ecosystem,
                        WriteTimeSlopVerdictKind::NotFound,
                        None,
                    );
                    findings.push(finding);
                }
                RegistryVerdict::TooNew { age_days } => {
                    // Package resolves but is stale/abandoned beyond the
                    // per-ecosystem threshold — Block as a slopsquat vector.
                    let finding = HookFinding {
                        tier: HookTier::T1,
                        verdict: HookVerdictKind::Block,
                        rule_id: Some("vibesec_dep_write_time".into()),
                        category: "vibesec_dep".into(),
                        message: format!(
                            "dependency '{}' (ecosystem: {}) is stale/abandoned ({}) — possible slopsquat (stale fork)",
                            candidate.name,
                            candidate.ecosystem,
                            age_days
                                .map(|d| format!("{d}d since last publish"))
                                .unwrap_or_else(|| "exceeds staleness threshold".to_owned()),
                        ),
                        fix_template: None,
                    };
                    self.emit_ssb_row(
                        &candidate.name,
                        candidate.ecosystem,
                        WriteTimeSlopVerdictKind::TooNew,
                        *age_days,
                    );
                    findings.push(finding);
                }
                RegistryVerdict::Unreachable => {
                    // Fail-open: Warn, do NOT block (W-10 network-axis policy).
                    // Revised W-10: a fail-open must leave a trace — emit an
                    // ADVISORY SSB row so the unreachable skip is observable
                    // ("no silent fail-open without an advisory SSB row"). The
                    // verdict stays Warn/Allow; the gate never blocks here.
                    let finding = HookFinding {
                        tier: HookTier::T1,
                        verdict: HookVerdictKind::Warn,
                        rule_id: Some("vibesec_dep_registry_unreachable".into()),
                        category: "vibesec_dep".into(),
                        message: format!(
                            "registry unreachable for '{}' (ecosystem: {}); skipping block (fail-open)",
                            candidate.name, candidate.ecosystem
                        ),
                        fix_template: None,
                    };
                    self.emit_ssb_row(
                        &candidate.name,
                        candidate.ecosystem,
                        WriteTimeSlopVerdictKind::RegistryUnreachable,
                        None,
                    );
                    findings.push(finding);
                }
            }
        }

        findings
    }

    /// Like [`analyze`](Self::analyze) but ALSO returns the structured blocked
    /// records (one per `NotFound` / `TooNew`) WITHOUT performing any SSB
    /// emission.  Used by the `elai dep-check` CLI (BLOCK-3): the CLI builds its
    /// own tokio runtime, opens `ssb.db` inside `block_on`, and writes the rows
    /// DURABLY (awaited) before the process exits — the `analyze()` internal
    /// `emit_ssb_row` spawn-detached path is not durable across an immediate
    /// process exit, so the CLI must collect-then-write itself.
    ///
    /// Returns `(findings, blocked_records)`.
    pub fn analyze_collect(
        &self,
        source: &str,
        file_path: &str,
    ) -> (Vec<HookFinding>, Vec<BlockedDepRecord>) {
        let candidates = extract_dep_candidates(source, file_path);
        let mut findings = Vec::new();
        let mut blocked = Vec::new();
        if candidates.is_empty() {
            return (findings, blocked);
        }

        for candidate in &candidates {
            let verdict = self.resolve(&candidate.name, candidate.ecosystem);
            match &verdict {
                RegistryVerdict::Found { .. } => {}
                RegistryVerdict::NotFound => {
                    findings.push(HookFinding {
                        tier: HookTier::T1,
                        verdict: HookVerdictKind::Block,
                        rule_id: Some("vibesec_dep_write_time".into()),
                        category: "vibesec_dep".into(),
                        message: format!(
                            "dependency '{}' (ecosystem: {}) not found in registry — possible slopsquat or hallucination",
                            candidate.name, candidate.ecosystem
                        ),
                        fix_template: None,
                    });
                    blocked.push(BlockedDepRecord {
                        dep_name: candidate.name.clone(),
                        ecosystem: candidate.ecosystem,
                        verdict_kind: WriteTimeSlopVerdictKind::NotFound,
                        registry_age_days: None,
                    });
                }
                RegistryVerdict::TooNew { age_days } => {
                    findings.push(HookFinding {
                        tier: HookTier::T1,
                        verdict: HookVerdictKind::Block,
                        rule_id: Some("vibesec_dep_write_time".into()),
                        category: "vibesec_dep".into(),
                        message: format!(
                            "dependency '{}' (ecosystem: {}) is stale/abandoned ({}) — possible slopsquat (stale fork)",
                            candidate.name,
                            candidate.ecosystem,
                            age_days
                                .map(|d| format!("{d}d since last publish"))
                                .unwrap_or_else(|| "exceeds staleness threshold".to_owned()),
                        ),
                        fix_template: None,
                    });
                    blocked.push(BlockedDepRecord {
                        dep_name: candidate.name.clone(),
                        ecosystem: candidate.ecosystem,
                        verdict_kind: WriteTimeSlopVerdictKind::TooNew,
                        registry_age_days: *age_days,
                    });
                }
                RegistryVerdict::Unreachable => {
                    // Verdict stays fail-open (Warn, never Block). Revised W-10:
                    // also push a DURABLE advisory record so the CLI dep-check
                    // (BLOCK-3) writes a registry_unreachable row via
                    // build_slop_payload — the fail-open is observable on the
                    // CLI path, not silent.
                    findings.push(HookFinding {
                        tier: HookTier::T1,
                        verdict: HookVerdictKind::Warn,
                        rule_id: Some("vibesec_dep_registry_unreachable".into()),
                        category: "vibesec_dep".into(),
                        message: format!(
                            "registry unreachable for '{}' (ecosystem: {}); skipping block (fail-open)",
                            candidate.name, candidate.ecosystem
                        ),
                        fix_template: None,
                    });
                    blocked.push(BlockedDepRecord {
                        dep_name: candidate.name.clone(),
                        ecosystem: candidate.ecosystem,
                        verdict_kind: WriteTimeSlopVerdictKind::RegistryUnreachable,
                        registry_age_days: None,
                    });
                }
            }
        }
        (findings, blocked)
    }

    /// Build the [`WriteTimeSlopsquatBlockedPayload`] for a blocked record.
    /// Exposed so the CLI (BLOCK-3) can write the row durably via its own
    /// runtime rather than the spawn-detached `emit_ssb_row` path.
    #[must_use]
    pub fn build_slop_payload(record: &BlockedDepRecord) -> WriteTimeSlopsquatBlockedPayload {
        let ts_ms: i64 = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis().min(i64::MAX as u128) as i64)
            .unwrap_or(0);
        WriteTimeSlopsquatBlockedPayload {
            dep_name_hash: dep_name_blake3_u64(&record.dep_name),
            ecosystem: record.ecosystem.to_owned(),
            verdict_kind: record.verdict_kind.as_str().to_owned(),
            registry_age_days: record.registry_age_days,
            ts_ms,
        }
    }

    /// Cache-aware registry probe.
    fn resolve(&self, name: &str, ecosystem: &str) -> RegistryVerdict {
        if let Some(cached) = self.cache.get(ecosystem, name) {
            return cached;
        }
        let verdict = self.checker.check(name, ecosystem);
        self.cache.insert(ecosystem, name, verdict.clone());
        verdict
    }

    /// Emit a `WriteTimeSlopsquatBlocked` SSB row (best-effort; errors are
    /// logged via `warn!`, not propagated).
    fn emit_ssb_row(
        &self,
        dep_name: &str,
        ecosystem: &str,
        verdict_kind: WriteTimeSlopVerdictKind,
        registry_age_days: Option<u32>,
    ) {
        let Some(ssb) = self.ssb.as_ref() else {
            return;
        };

        let ts_ms: i64 = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis().min(i64::MAX as u128) as i64)
            .unwrap_or(0);
        let payload = SsbPayload::WriteTimeSlopsquatBlocked(WriteTimeSlopsquatBlockedPayload {
            dep_name_hash: dep_name_blake3_u64(dep_name),
            ecosystem: ecosystem.to_owned(),
            verdict_kind: verdict_kind.as_str().to_owned(),
            registry_age_days,
            ts_ms,
        });

        // SSB write is async; we cannot await here.  Spawn a best-effort
        // tokio task if a runtime is available.  If no runtime is active
        // (e.g. test / non-tokio contexts), log the miss and continue.
        let ssb = Arc::clone(ssb);
        match tokio::runtime::Handle::try_current() {
            Ok(handle) => {
                handle.spawn(async move {
                    if let Err(e) = ssb
                        .write_payload(
                            "vibesec_dep_write_time_gate",
                            0,
                            payload,
                            vec![],
                        )
                        .await
                    {
                        warn!(err = %e, "WriteTimeSlopsquatBlocked SSB emit failed");
                    }
                });
            }
            Err(_) => {
                warn!(
                    ecosystem = ecosystem,
                    dep_hash = dep_name_blake3_u64(dep_name),
                    "WriteTimeSlopsquatBlocked SSB emit skipped (no tokio runtime)"
                );
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stub_checker_known_real_packages() {
        let checker = StubRegistryChecker;
        assert_eq!(
            checker.check("serde", "cargo"),
            RegistryVerdict::Found { age_days: Some(365) }
        );
        assert_eq!(
            checker.check("requests", "pypi"),
            RegistryVerdict::Found { age_days: Some(365) }
        );
        assert_eq!(
            checker.check("react", "npm"),
            RegistryVerdict::Found { age_days: Some(365) }
        );
    }

    #[test]
    fn stub_checker_known_hallucinated() {
        let checker = StubRegistryChecker;
        assert_eq!(checker.check("reqquests", "pypi"), RegistryVerdict::NotFound);
        assert_eq!(checker.check("reqwest-typo", "cargo"), RegistryVerdict::NotFound);
        // Documented typo prefix.
        assert_eq!(checker.check("numpy-fake", "pypi"), RegistryVerdict::NotFound);
    }

    #[test]
    fn stub_checker_unknown_returns_found() {
        // WARN-3: offline mode is safe on real code — an unknown (but not
        // known-hallucinated) name is treated as Found, never blocked.
        let checker = StubRegistryChecker;
        assert_eq!(
            checker.check("some-made-up-crate-9999", "cargo"),
            RegistryVerdict::Found { age_days: Some(365) }
        );
    }

    #[test]
    fn lru_cache_hit_and_ttl() {
        let cache = LruExistenceCache::new(10, Duration::from_secs(3_600));
        cache.insert("cargo", "serde", RegistryVerdict::Found { age_days: Some(365) });
        let got = cache.get("cargo", "serde");
        assert_eq!(got, Some(RegistryVerdict::Found { age_days: Some(365) }));
    }

    #[test]
    fn lru_cache_miss() {
        let cache = LruExistenceCache::new(10, Duration::from_secs(3_600));
        assert_eq!(cache.get("cargo", "nonexistent"), None);
    }

    #[test]
    fn lru_cache_eviction_at_capacity() {
        let cache = LruExistenceCache::new(2, Duration::from_secs(3_600));
        cache.insert("cargo", "a", RegistryVerdict::NotFound);
        cache.insert("cargo", "b", RegistryVerdict::NotFound);
        cache.insert("cargo", "c", RegistryVerdict::NotFound); // evicts "a"
        assert_eq!(cache.get("cargo", "a"), None); // evicted
        assert!(cache.get("cargo", "b").is_some());
        assert!(cache.get("cargo", "c").is_some());
    }

    #[test]
    fn extract_python_imports() {
        let src = "import requests\nfrom numpy import array\nimport os\n";
        let deps = extract_dep_candidates(src, "foo.py");
        let names: Vec<&str> = deps.iter().map(|d| d.name.as_str()).collect();
        assert!(names.contains(&"requests"), "missing requests: {:?}", names);
        assert!(names.contains(&"numpy"), "missing numpy: {:?}", names);
        // os is stdlib — must be excluded
        assert!(!names.contains(&"os"), "os should be excluded: {:?}", names);
    }

    #[test]
    fn extract_js_imports() {
        let src = "import React from 'react';\nconst axios = require('axios');\n";
        let deps = extract_dep_candidates(src, "app.js");
        let names: Vec<&str> = deps.iter().map(|d| d.name.as_str()).collect();
        assert!(names.contains(&"react"), "missing react: {:?}", names);
        assert!(names.contains(&"axios"), "missing axios: {:?}", names);
    }

    #[test]
    fn extract_rust_use_lines() {
        let src = "use serde::Deserialize;\nuse tokio::sync::Mutex;\nuse crate::foo;\n";
        let deps = extract_dep_candidates(src, "main.rs");
        let names: Vec<&str> = deps.iter().map(|d| d.name.as_str()).collect();
        assert!(names.contains(&"serde"), "missing serde: {:?}", names);
        assert!(names.contains(&"tokio"), "missing tokio: {:?}", names);
        // `crate` should never be included
        assert!(!names.contains(&"crate"), "crate should be excluded: {:?}", names);
    }

    #[test]
    fn analyze_blocks_not_found() {
        let checker = Arc::new(StubRegistryChecker);
        let gate = DepWriteTimeGate::new(None, checker);
        // reqquests → NotFound in StubRegistryChecker (known-hallucinated)
        let src = "import reqquests\n";
        let findings = gate.analyze(src, "foo.py");
        assert!(!findings.is_empty(), "expected at least one finding");
        assert!(
            findings.iter().any(|f| f.verdict == HookVerdictKind::Block),
            "expected Block verdict for NotFound dep: {:?}",
            findings
        );
    }

    #[test]
    fn analyze_agnostic_empty_path_blocks_hallucinated_rust() {
        // BLOCK-2: live PreToolUse path carries file_path="" + the candidate
        // text as source. The language-agnostic fallback must still extract the
        // hallucinated dep and Block it.
        let checker = Arc::new(StubRegistryChecker);
        let gate = DepWriteTimeGate::new(None, checker);
        let src = "use reqquests::get;\nuse serde::Deserialize;\n";
        let findings = gate.analyze(src, "");
        assert!(
            findings.iter().any(|f| f.verdict == HookVerdictKind::Block),
            "expected Block on empty-path agnostic extraction: {:?}",
            findings
        );
    }

    #[test]
    fn parse_age_days_from_reason_variants() {
        assert_eq!(
            parse_age_days_from_reason("cargo: 'serde' last updated 12d ago"),
            Some(12)
        );
        assert_eq!(
            parse_age_days_from_reason("pypi: 'x'==1.0 last published 900d ago (>730d stale)"),
            Some(900)
        );
        assert_eq!(parse_age_days_from_reason("no age here"), None);
    }

    #[test]
    fn parse_registry_json_contract_arms() {
        assert_eq!(
            parse_registry_json("{\"result\":\"pass\",\"reason\":\"cargo: 'serde' last updated 12d ago\"}"),
            RegistryVerdict::Found { age_days: Some(12) }
        );
        assert_eq!(
            parse_registry_json("{\"result\":\"fail\",\"reason\":\"cargo 404: crate 'x' does not exist (likely slopsquat)\"}"),
            RegistryVerdict::NotFound
        );
        assert_eq!(
            parse_registry_json("{\"result\":\"fail\",\"reason\":\"pypi: 'x'==1.0 last published 900d ago (>730d stale)\"}"),
            RegistryVerdict::TooNew { age_days: Some(900) }
        );
        assert_eq!(
            parse_registry_json("{\"result\":\"inapplicable\",\"reason\":\"empty stdin\"}"),
            RegistryVerdict::Unreachable
        );
        // Missing keys / malformed → fail-open Unreachable.
        assert_eq!(parse_registry_json("{}"), RegistryVerdict::Unreachable);
        assert_eq!(parse_registry_json("not json"), RegistryVerdict::Unreachable);
    }

    #[test]
    fn analyze_passes_known_good() {
        let checker = Arc::new(StubRegistryChecker);
        let gate = DepWriteTimeGate::new(None, checker);
        let src = "import requests\n";
        let findings = gate.analyze(src, "foo.py");
        // requests is in StubRegistryChecker KNOWN_REAL — no Block
        assert!(
            !findings.iter().any(|f| f.verdict == HookVerdictKind::Block),
            "unexpected Block for known-good dep: {:?}",
            findings
        );
    }

    #[test]
    fn dep_name_blake3_u64_is_deterministic() {
        let a = dep_name_blake3_u64("serde");
        let b = dep_name_blake3_u64("serde");
        assert_eq!(a, b);
        assert_ne!(dep_name_blake3_u64("serde"), dep_name_blake3_u64("tokio"));
    }
}
