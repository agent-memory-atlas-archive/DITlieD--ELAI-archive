//! C-27 — Testing-Trust Tier T-0: WiringContract foundation.
//!
//! Implements the three deterministic auditors that decide whether a B-N /
//! C-N plan's WIRING_CONTRACT block is honoured by the actual code:
//!
//!   1. [`StubBanAuditor`] — regex-scans production-path source for forbidden
//!      patterns (todo!() / unimplemented!() / "deferred" / "stub" /
//!      exact-line `Ok(Default::default())`). False positives are tolerated;
//!      false negatives are not.
//!   2. [`DkgReachabilityAuditor`] — given an in-memory symbol set (assembled
//!      by the audit driver from the C-17 SymbolGraph), confirms that each
//!      declared `produces_symbol` is BFS-reachable from at least one
//!      configured `[wiring.entrypoints]`.
//!   3. [`ObservableEffectRunner`] — polls the SSB DB at a fixed cadence for
//!      a row whose `action_type` matches the contract's declared
//!      `observable_effect.expect_ssb`; on timeout emits a
//!      `WiringViolation(BrokenObservable)` instead of panicking.
//!
//! In addition, the module hosts:
//!   * [`WiringContract`] — the parsed-from-markdown form of a single
//!     WIRING_CONTRACT block.
//!   * [`StubBanEntry`] + [`ObservableEffect`] — value types referenced by
//!     the contract.
//!   * [`WiringEntrypointSet`] + [`WiringEntrypointSetLoader`] — reads
//!     `[wiring.entrypoints]` from `elai_cc.toml`. Missing section is non-
//!     fatal (returns empty set + tracing::warn!, no panic).
//!   * [`WiringAuditCmd`] + [`WiringAuditReport`] + [`WiringAuditError`] —
//!     orchestration types consumed by the CLI driver in
//!     `crates/cli/src/wiring_audit.rs`.
//!
//! Grounding sources:
//!   * docs/thefinalglance/verification/reports/guide/testing-trust-design.report.md §Layer 0
//!   * .claude/tools/wiring_contract_check.py (Python prototype — same shape,
//!     algorithm ported here).
//!   * workflow/reasoning/C-15-decisions.md D7 (entropy-allow precedent for
//!     audit_id Uuid::new_v4()).
//!   * workflow/reasoning/C-17-decisions.md (DKG cache + traversal patterns).

use std::collections::{BTreeSet, HashSet, VecDeque};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use regex::Regex;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::mpsc::UnboundedSender;
use tracing::{debug, warn};
use walkdir::WalkDir;

use crate::ssb::{
    SsbPayload, WiringEvidencePayload, WiringViolationKind, WiringViolationPayload,
};
// SP2 (WAVE 2): real reachability-graph source. SymbolGraphCache lives in
// THIS crate (crate::symbol_graph, E-40 orchestrator-rooted JSON view) — NOT
// in elai_knowledge. Verified: crates/orchestrator/src/symbol_graph/cache.rs
// is the def (load() :167, iter_nodes() :350, SymbolNode :60-76). Same-crate
// use means no Cargo.toml change and no new dependency.
use crate::symbol_graph::{SymbolGraphCache, SymbolGraphCacheError};

// ---------------------------------------------------------------------------
// Skip set (DRIFT-6) — mirrors check_entropy_quarantine.sh +
// .claude/tools/wiring_contract_check.py SKIP_DIRS exactly. New entries here
// MUST be appended to both companion lists as well.
// ---------------------------------------------------------------------------

const SKIP_DIR_FRAGMENTS: &[&str] = &[
    "/tests/",
    "/target/",
    "/node_modules/",
    "/__pycache__/",
    "/.bkp_old/",
    "/.elai/",
    "/benches/",
    "/examples/",
    "/.venv/",
    "/.venv-staging/",
    "/site-packages/",
    "/dist-info/",
    "/.tox/",
    "\\tests\\",
    "\\target\\",
    "\\node_modules\\",
    "\\__pycache__\\",
    "\\.bkp_old\\",
    "\\.elai\\",
    "\\benches\\",
    "\\examples\\",
    "\\.venv\\",
    "\\.venv-staging\\",
    "\\site-packages\\",
    "\\dist-info\\",
    "\\.tox\\",
];

/// Path-prefix forms used when a path is relative (no leading separator) —
/// e.g., `target/debug/foo.rs` vs `crates/foo/target/debug/foo.rs`. We match
/// the canonical first-segment forms on both Unix and Windows separators.
const SKIP_DIR_PREFIXES: &[&str] = &[
    "tests/",
    "target/",
    "node_modules/",
    "__pycache__/",
    ".bkp_old/",
    "benches/",
    "examples/",
    ".venv/",
    ".venv-staging/",
    "site-packages/",
    ".tox/",
    "tests\\",
    "target\\",
    "node_modules\\",
    "__pycache__\\",
    ".bkp_old\\",
    "benches\\",
    "examples\\",
    ".venv\\",
    ".venv-staging\\",
    "site-packages\\",
    ".tox\\",
];

/// File-name suffixes (case-sensitive) that mark test scaffolding which
/// MUST NOT be scanned for stub patterns. Matches anywhere in the file
/// basename so `foo_test.rs` and `bar_contract.rs` are both excluded.
const SKIP_FILE_SUFFIXES: &[&str] = &["_test.rs", "_contract.rs"];

/// Extensions that StubBanAuditor scans by default.
const SCANNABLE_EXTENSIONS: &[&str] = &["rs", "py", "ts", "tsx", "js", "svelte"];

// ---------------------------------------------------------------------------
// WiringContract value types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StubBanEntry {
    pub pattern: String,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObservableEffect {
    pub trigger: String,
    #[serde(default)]
    pub expect_ssb: Option<String>,
    #[serde(default)]
    pub expect_artifact: Option<String>,
    pub timeout_secs: u32,
}

/// Parsed WIRING_CONTRACT block from a plan markdown file (B-N or C-N).
/// The Python prototype at `.claude/tools/wiring_contract_check.py` was
/// the algorithmic reference.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WiringContract {
    pub bn_id: String,
    #[serde(default)]
    pub produces_symbols: Vec<String>,
    #[serde(default)]
    pub consumes_symbols: Vec<String>,
    #[serde(default)]
    pub reachable_from: Vec<String>,
    #[serde(default)]
    pub stub_ban: Vec<StubBanEntry>,
    #[serde(default)]
    pub frontend_pair: Option<String>,
    #[serde(default)]
    pub observable_effects: Vec<ObservableEffect>,
}

#[derive(Debug, Error)]
pub enum WiringContractError {
    #[error("wiring contract parse error: {0}")]
    ParseError(String),
    #[error(transparent)]
    IoError(#[from] io::Error),
    #[error("regex compile error: {0}")]
    RegexError(String),
    #[error("sqlite error: {0}")]
    SqliteError(String),
    /// SP1 R1/R2 fail-close: `[wiring.entrypoints].symbols` is present in
    /// the config but resolves to an empty list. Wiring enforcement treats
    /// an empty entrypoint set as "never proven reachable", not "nothing to
    /// check" — missing evidence is a hard FAIL (R1).
    #[error("[wiring.entrypoints].symbols is present but empty in {0}; an empty entrypoint set cannot prove reachability (R1 fail-close)")]
    EmptyEntrypointSet(PathBuf),
    /// SP1 R1/R2 fail-close: the `[wiring.entrypoints]` section is absent
    /// entirely. Per fail-closed doctrine an absent gate input is FAIL, not
    /// a silent empty default (R2 dead-evidence-path-must-scream).
    #[error("[wiring.entrypoints] section absent from {0}; wiring audit requires a declared entrypoint set (R2 fail-close)")]
    AbsentEntrypointSet(PathBuf),
}

impl From<rusqlite::Error> for WiringContractError {
    fn from(e: rusqlite::Error) -> Self {
        WiringContractError::SqliteError(e.to_string())
    }
}

// ---------------------------------------------------------------------------
// WiringEntrypointSet
// ---------------------------------------------------------------------------

/// The closed entrypoint list read from `[wiring.entrypoints]` in
/// `elai_cc.toml`. Names are bare symbol paths (e.g.
/// `"crates/cli/src/main.rs::main"`); the auditor does NOT interpret them
/// beyond string-equality matches against the DKG node id space.
///
/// DRIFT-4 — named `WiringEntrypointSet` (not `EntrypointSet`) to avoid
/// collision with `elai_core::EntryPoint`,
/// `knowledge::symbol_graph::EntryPoint`, and
/// `SsbActionType::EntryPointsDetected`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WiringEntrypointSet {
    pub entrypoints: Vec<String>,
}

impl WiringEntrypointSet {
    pub fn new(entrypoints: Vec<String>) -> Self {
        Self { entrypoints }
    }

    pub fn is_empty(&self) -> bool {
        self.entrypoints.is_empty()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, String> {
        self.entrypoints.iter()
    }
}

/// Loader for `[wiring.entrypoints]` out of an `elai_cc.toml` file.
/// Missing section is NON-FATAL — returns an empty `WiringEntrypointSet`
/// plus a tracing::warn!. No panic on absent file either; that returns a
/// typed IO error so the audit driver can decide whether to bail.
pub struct WiringEntrypointSetLoader {
    config_path: PathBuf,
}

impl WiringEntrypointSetLoader {
    pub fn new(config_path: impl Into<PathBuf>) -> Self {
        Self {
            config_path: config_path.into(),
        }
    }

    pub fn load(&self) -> Result<WiringEntrypointSet, WiringContractError> {
        let raw = fs::read_to_string(&self.config_path)?;
        let parsed: toml::Value = toml::from_str(&raw)
            .map_err(|e: toml::de::Error| WiringContractError::ParseError(e.to_string()))?;
        let entrypoints = parsed
            .get("wiring")
            .and_then(|w| w.get("entrypoints"))
            .and_then(|e| e.get("symbols"))
            .and_then(|s| s.as_array());
        match entrypoints {
            Some(arr) => {
                let collected: Vec<String> = arr
                    .iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_owned()))
                    .collect();
                if collected.is_empty() {
                    // SP1 R1 fail-close: present-but-empty symbols list is a
                    // hard FAIL. An empty entrypoint set cannot prove any
                    // produces_symbol reachable, so it is missing evidence,
                    // not an empty work set.
                    return Err(WiringContractError::EmptyEntrypointSet(
                        self.config_path.clone(),
                    ));
                }
                Ok(WiringEntrypointSet::new(collected))
            }
            None => {
                // SP1 R2 fail-close: absent [wiring.entrypoints] section is a
                // hard FAIL. A dead gate input must scream, not return a
                // silent empty default.
                Err(WiringContractError::AbsentEntrypointSet(
                    self.config_path.clone(),
                ))
            }
        }
    }
}

// ---------------------------------------------------------------------------
// StubBanAuditor
// ---------------------------------------------------------------------------

/// Opt-out token: any line containing this literal is exempt from stub-ban
/// scanning. Mirrors the `// entropy-allow: <reason>` precedent from C-15.
/// Callers tag legitimate uses with a brief reason, e.g.
/// `// stub-ban-allow: doc comment describing the stub_marker default regex`.
pub const STUB_BAN_ALLOW_TOKEN: &str = "stub-ban-allow:";

/// Default forbidden patterns. Tightened in C-27 BLOCK-1 follow-up: the
/// bare-word `\bstub\b` / `\bdeferred\b` were dropped because they
/// false-positived on doc comments, log messages, and the auditor module's
/// OWN literal pattern strings (930 self-trips across 233 files). Default
/// patterns now match only:
///   * function-body macros: `todo!()` / `unimplemented!()`
///   * `panic!("not yet ...")` / `panic!("not implemented ...")` style aborts
///   * explicit comment markers: `// STUB:` / `// TODO:` / `// FIXME:`
///     and the equivalent block-comment forms (`/* STUB:`)
///   * exact-line `Ok(Default::default())` returns (DRIFT-8 / D8)
/// Callers wanting stricter word-level scans can pass extras via the
/// per-contract `stub_ban:` block (see `parse_wiring_contract_from_markdown`).
fn default_forbidden_patterns() -> Vec<(Regex, StubBanEntry)> {
    let raw: &[(&str, &str, &str)] = &[
        // Statement-context only. Bare `todo!()` / `unimplemented!()` on a
        // line by itself (with optional trailing `;` and optional comment).
        // String-literal occurrences inside writeln!/format!/grep-pattern
        // fixtures are NOT flagged — those uses are documentation, not
        // unfinished code. The wiring auditor cares about UNFINISHED
        // function bodies, not regex testbed strings.
        (
            r"^\s*todo!\s*\(\s*\)\s*;?\s*(?://.*)?$",
            "todo_macro",
            "bare todo!() in production code",
        ),
        (
            r"^\s*unimplemented!\s*\(\s*\)\s*;?\s*(?://.*)?$",
            "unimplemented_macro",
            "bare unimplemented!() in production code",
        ),
        (
            r#"^\s*panic!\s*\(\s*"(?:not yet|not implemented|TODO|stub)"#,
            "panic_not_implemented",
            r#"panic!("not yet ..." / "not implemented ..." / "TODO ..." / "stub ...") at statement position"#,
        ),
        // Comment-anchored markers ONLY. The leading `^\s*` plus the explicit
        // `//` or `/*` requires the marker to be in a comment at the start of
        // the line. Documentation prose and string literals are not flagged.
        (
            r"^\s*(?://+|/\*+)\s*STUB(?::|\b)",
            "stub_marker_comment",
            "`// STUB:` marker in production code",
        ),
        (
            r"^\s*(?://+|/\*+)\s*TODO(?::|\b)",
            "todo_marker_comment",
            "`// TODO:` marker in production code",
        ),
        (
            r"^\s*(?://+|/\*+)\s*FIXME(?::|\b)",
            "fixme_marker_comment",
            "`// FIXME:` marker in production code",
        ),
        // Exact-line anchored — see DRIFT-8 / C-27-decisions.md D8.
        (
            r"^\s*Ok\(Default::default\(\)\)\s*$",
            "ok_default_default_exact_line",
            "exact-line return Ok(Default::default()) in production code",
        ),
    ];
    raw.iter()
        .map(|(pat, name, desc)| {
            let re = Regex::new(pat).unwrap_or_else(|e| {
                // Compile error on a literal regex is a static bug — emit a
                // never-matching regex so the audit limps along rather than
                // panicking. Loud trace plus the WiringViolation row makes
                // this visible in dashboards.
                warn!(
                    target: "elai_orchestrator::wiring_contract",
                    "default stub_ban regex {pat:?} failed to compile: {e}"
                );
                Regex::new(r"^\xFFnever-matches\xFF$").unwrap_or_else(|_| Regex::new("a").expect("trivial regex"))
            });
            (
                re,
                StubBanEntry {
                    pattern: (*name).to_owned(),
                    description: (*desc).to_owned(),
                },
            )
        })
        .collect()
}

/// Sink used by the auditors to publish violations / evidence rows to SSB.
/// Production sinks emit through an mpsc channel that the audit driver
/// drains and writes to the SSB DB; tests use an in-memory capture vec.
pub enum AuditorSink {
    Channel(UnboundedSender<SsbPayload>),
    #[cfg(test)]
    Capture(Arc<std::sync::Mutex<Vec<SsbPayload>>>),
}

impl AuditorSink {
    fn send(&self, payload: SsbPayload) {
        match self {
            Self::Channel(tx) => {
                if let Err(err) = tx.send(payload) {
                    warn!(
                        target: "elai_orchestrator::wiring_contract",
                        "auditor SSB channel closed: {err}"
                    );
                }
            }
            #[cfg(test)]
            Self::Capture(buf) => match buf.lock() {
                Ok(mut g) => g.push(payload),
                Err(poisoned) => poisoned.into_inner().push(payload),
            },
        }
    }
}

pub struct StubBanAuditor {
    patterns: Vec<(Regex, StubBanEntry)>,
    sink: AuditorSink,
    audit_id: uuid::Uuid,
}

impl StubBanAuditor {
    /// Construct an auditor seeded with the default forbidden patterns plus
    /// any contract-level extras.
    pub fn new(
        contract_patterns: &[StubBanEntry],
        sink: AuditorSink,
        audit_id: uuid::Uuid,
    ) -> Result<Self, WiringContractError> {
        let mut patterns = default_forbidden_patterns();
        for entry in contract_patterns {
            let re = Regex::new(&entry.pattern)
                .map_err(|e| WiringContractError::RegexError(e.to_string()))?;
            patterns.push((re, entry.clone()));
        }
        Ok(Self {
            patterns,
            sink,
            audit_id,
        })
    }

    /// Decide whether a path should be skipped from production scanning.
    /// Conservative — false positives only suppress test scaffolding, never
    /// real production code (DRIFT-7).
    pub fn should_skip_path(path: &Path) -> bool {
        let s = path.to_string_lossy();
        if SKIP_DIR_FRAGMENTS.iter().any(|frag| s.contains(frag)) {
            return true;
        }
        if SKIP_DIR_PREFIXES.iter().any(|pre| s.starts_with(pre)) {
            return true;
        }
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if SKIP_FILE_SUFFIXES.iter().any(|suf| name.ends_with(suf)) {
                return true;
            }
        }
        false
    }

    fn extension_allowed(path: &Path) -> bool {
        match path.extension().and_then(|e| e.to_str()) {
            Some(ext) => SCANNABLE_EXTENSIONS.iter().any(|allowed| *allowed == ext),
            None => false,
        }
    }

    /// Scan a single file's text contents and emit one
    /// `WiringViolation(StubPattern)` per forbidden-pattern line match.
    /// Returns the number of violations emitted.
    ///
    /// Lines containing the `stub-ban-allow:` opt-out token are skipped
    /// entirely — mirrors the `// entropy-allow: <reason>` convention from
    /// C-15. Use sparingly and always with a written justification.
    pub fn scan_file_contents(&self, path: &Path, contents: &str) -> u32 {
        if Self::should_skip_path(path) {
            return 0;
        }
        let mut count = 0u32;
        for (line_no_zero, line) in contents.lines().enumerate() {
            if line.contains(STUB_BAN_ALLOW_TOKEN) {
                continue;
            }
            for (re, entry) in &self.patterns {
                if re.is_match(line) {
                    let payload = WiringViolationPayload {
                        violation_kind: WiringViolationKind::StubPattern,
                        file: path.to_string_lossy().to_string(),
                        line: (line_no_zero as u32) + 1,
                        pattern: entry.pattern.clone(),
                        audit_id: self.audit_id,
                    };
                    self.sink.send(SsbPayload::WiringViolation(payload));
                    count += 1;
                    // One violation per (line, pattern) — multiple patterns
                    // on the same line are individually reported because
                    // operators care about each rule independently.
                }
            }
        }
        count
    }

    /// Walk `root` (default to current dir) using stdlib WalkDir + an
    /// extension filter (no globset dep — DRIFT-3 / C-27-decisions.md D3).
    /// Returns the total violation count.
    pub fn scan_production_paths(&self, root: &Path) -> Result<u32, WiringContractError> {
        let mut total = 0u32;
        for entry in WalkDir::new(root).follow_links(false) {
            let entry = match entry {
                Ok(e) => e,
                Err(e) => {
                    debug!(
                        target: "elai_orchestrator::wiring_contract",
                        "walkdir error under {root:?}: {e}"
                    );
                    continue;
                }
            };
            let path = entry.path();
            if !entry.file_type().is_file() {
                continue;
            }
            if Self::should_skip_path(path) {
                continue;
            }
            if !Self::extension_allowed(path) {
                continue;
            }
            match fs::read_to_string(path) {
                Ok(contents) => {
                    total += self.scan_file_contents(path, &contents);
                }
                Err(e) => {
                    // Non-UTF8 files (e.g., binaries dropped into src) are
                    // skipped silently — the WiringContract is concerned
                    // with textual stub patterns only.
                    debug!(
                        target: "elai_orchestrator::wiring_contract",
                        "skipping {path:?} read error: {e}"
                    );
                }
            }
        }
        Ok(total)
    }
}

// ---------------------------------------------------------------------------
// DkgReachabilityAuditor
// ---------------------------------------------------------------------------

/// A trimmed view of the DKG that the reachability auditor needs. The audit
/// driver assembles this from
/// `elai_knowledge::symbol_graph::SymbolGraphCache` so this module stays
/// independent of the on-disk schema.
#[derive(Debug, Clone, Default)]
pub struct ReachabilityGraph {
    /// Adjacency: symbol_id -> set of symbol_ids it can directly reach.
    pub forward_edges: std::collections::BTreeMap<String, BTreeSet<String>>,
    /// Set of symbol_ids known to the graph (used to disambiguate
    /// "unknown symbol" from "known but unreachable" in callers).
    pub known_symbols: BTreeSet<String>,
}

impl ReachabilityGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_edge(&mut self, from: impl Into<String>, to: impl Into<String>) {
        let f = from.into();
        let t = to.into();
        self.known_symbols.insert(f.clone());
        self.known_symbols.insert(t.clone());
        self.forward_edges.entry(f).or_default().insert(t);
    }

    pub fn add_symbol(&mut self, sym: impl Into<String>) {
        self.known_symbols.insert(sym.into());
    }
}

pub struct DkgReachabilityAuditor {
    graph: Arc<ReachabilityGraph>,
    sink: AuditorSink,
    audit_id: uuid::Uuid,
}

impl DkgReachabilityAuditor {
    pub fn new(graph: Arc<ReachabilityGraph>, sink: AuditorSink, audit_id: uuid::Uuid) -> Self {
        Self {
            graph,
            sink,
            audit_id,
        }
    }

    /// BFS from each entrypoint until `target` is hit, short-circuiting on
    /// the first match. Returns `Some(entrypoint)` on hit, `None` on miss.
    pub fn is_reachable(
        &self,
        entrypoints: &WiringEntrypointSet,
        target: &str,
    ) -> Option<String> {
        // BFS preferred over DFS for bounded-depth walks on large DKGs (D-bfs).
        for ep in entrypoints.iter() {
            if ep == target {
                return Some(ep.clone());
            }
            let mut visited = HashSet::new();
            let mut queue: VecDeque<&str> = VecDeque::new();
            queue.push_back(ep.as_str());
            visited.insert(ep.clone());
            while let Some(cur) = queue.pop_front() {
                if cur == target {
                    return Some(ep.clone());
                }
                if let Some(neighbors) = self.graph.forward_edges.get(cur) {
                    for n in neighbors {
                        if visited.insert(n.clone()) {
                            queue.push_back(n.as_str());
                        }
                    }
                }
            }
        }
        None
    }

    /// Audit a single contract against the configured entrypoint set. For
    /// each produces_symbol: emit one WiringEvidence row if reachable, one
    /// WiringViolation(UnreachableSymbol) otherwise.
    pub fn audit_contract(
        &self,
        contract: &WiringContract,
        entrypoints: &WiringEntrypointSet,
    ) -> DkgAuditOutcome {
        let mut verified_by_entrypoint: std::collections::BTreeMap<String, Vec<String>> =
            std::collections::BTreeMap::new();
        let mut violation_count = 0u32;
        for sym in &contract.produces_symbols {
            match self.is_reachable(entrypoints, sym) {
                Some(ep) => {
                    verified_by_entrypoint
                        .entry(ep)
                        .or_default()
                        .push(sym.clone());
                }
                None => {
                    let payload = WiringViolationPayload {
                        violation_kind: WiringViolationKind::UnreachableSymbol,
                        file: contract.bn_id.clone(),
                        line: 0,
                        pattern: sym.clone(),
                        audit_id: self.audit_id,
                    };
                    self.sink.send(SsbPayload::WiringViolation(payload));
                    violation_count += 1;
                }
            }
        }
        let mut evidence_count = 0u32;
        for (ep, verified) in &verified_by_entrypoint {
            let payload = WiringEvidencePayload {
                entrypoint: ep.clone(),
                produces_symbols_verified: verified.clone(),
                reachable_count: verified.len() as u32,
                audit_id: self.audit_id,
            };
            self.sink.send(SsbPayload::WiringEvidence(payload));
            evidence_count += 1;
        }
        DkgAuditOutcome {
            evidence_emitted: evidence_count,
            violations_emitted: violation_count,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DkgAuditOutcome {
    pub evidence_emitted: u32,
    pub violations_emitted: u32,
}

// ---------------------------------------------------------------------------
// ObservableEffectRunner
// ---------------------------------------------------------------------------

pub struct ObservableEffectRunner {
    ssb_db_path: PathBuf,
    sink: AuditorSink,
    audit_id: uuid::Uuid,
    poll_interval: Duration,
}

impl ObservableEffectRunner {
    /// Construct with a default 1s poll cadence — see C-27-decisions.md D-poll
    /// for the choice rationale (SQLite has no NOTIFY; polling is
    /// deterministic and unit-testable).
    pub fn new(ssb_db_path: impl Into<PathBuf>, sink: AuditorSink, audit_id: uuid::Uuid) -> Self {
        Self {
            ssb_db_path: ssb_db_path.into(),
            sink,
            audit_id,
            poll_interval: Duration::from_secs(1),
        }
    }

    /// Override the poll interval. Used by tests to keep timeouts fast.
    pub fn with_poll_interval(mut self, interval: Duration) -> Self {
        self.poll_interval = interval;
        self
    }

    /// Run a single observable_effect declaration to completion.
    ///
    /// `bn_id` is the declaring plan id (B-N / C-N) — surfaced inside the
    /// `WiringViolation.file` field on failure for triage.
    ///
    /// On `expect_ssb=Some(action_type)` — polls the SSB DB until at least
    /// one row with that snake_case `action_type` appears OR `timeout_secs`
    /// elapses. Timeout emits `WiringViolation(BrokenObservable)` and
    /// returns `Err(WiringContractError::ParseError(...))` so callers
    /// distinguish success from violation; the SSB row is the durable
    /// signal.
    ///
    /// On `expect_artifact=Some(path)` — checks `path::exists()` after the
    /// timeout deadline; absent triggers `WiringViolation(BrokenObservable)`.
    pub async fn run_effect(
        &self,
        effect: &ObservableEffect,
        bn_id: &str,
    ) -> Result<(), WiringContractError> {
        let deadline = tokio::time::Instant::now() + Duration::from_secs(effect.timeout_secs as u64); // entropy-allow: tokio-mockable-time-required-for-tokio-time-sleep-pairing

        if let Some(action_type) = effect.expect_ssb.as_deref() {
            // SP4(e) fail-close: track whether the SSB DB ever materialised in
            // the poll window. If the deadline is reached and the DB never
            // appeared at all, that is a distinct, louder failure than "db
            // present but row absent" — the producing entrypoint never ran.
            let mut db_ever_existed = false;
            loop {
                if self.ssb_db_path.exists() {
                    db_ever_existed = true;
                }
                if Self::ssb_has_row(&self.ssb_db_path, action_type)? {
                    return Ok(());
                }
                if tokio::time::Instant::now() >= deadline { // entropy-allow: tokio-mockable-time-paired-with-deadline-above
                    self.emit_broken_observable(bn_id, effect.trigger.as_str());
                    let detail = if db_ever_existed {
                        format!(
                            "expect_ssb={action_type:?} not seen within {}s",
                            effect.timeout_secs
                        )
                    } else {
                        format!(
                            "SSB db at {} never appeared within {}s (producing entrypoint never ran) expect_ssb={action_type:?}",
                            self.ssb_db_path.display(),
                            effect.timeout_secs
                        )
                    };
                    return Err(WiringContractError::ParseError(format!(
                        "observable_effect timeout: {detail}"
                    )));
                }
                tokio::time::sleep(self.poll_interval).await;
            }
        }

        if let Some(artifact_path) = effect.expect_artifact.as_deref() {
            // Sleep until deadline then check exists() — the artifact is
            // expected to be produced by the trigger; we give it the full
            // timeout to materialize.
            tokio::time::sleep_until(deadline).await;
            if Path::new(artifact_path).exists() {
                return Ok(());
            }
            self.emit_broken_observable(bn_id, effect.trigger.as_str());
            return Err(WiringContractError::ParseError(format!(
                "observable_effect timeout: expect_artifact={artifact_path:?} not present after {}s",
                effect.timeout_secs
            )));
        }

        // No expect_ssb and no expect_artifact -> contract is malformed but
        // not panic-worthy. Treat as a violation so it surfaces during audit.
        self.emit_broken_observable(bn_id, effect.trigger.as_str());
        Err(WiringContractError::ParseError(
            "observable_effect declared neither expect_ssb nor expect_artifact".to_owned(),
        ))
    }

    fn emit_broken_observable(&self, bn_id: &str, trigger: &str) {
        let payload = WiringViolationPayload {
            violation_kind: WiringViolationKind::BrokenObservable,
            file: bn_id.to_owned(),
            line: 0,
            pattern: trigger.to_owned(),
            audit_id: self.audit_id,
        };
        self.sink.send(SsbPayload::WiringViolation(payload));
    }

    /// Open the SSB DB read-only and check whether at least one row of
    /// `action_type` exists. Returns `Ok(false)` for a missing DB so callers
    /// can poll across the boot window.
    fn ssb_has_row(db_path: &Path, action_type: &str) -> Result<bool, WiringContractError> {
        // SP4(e): a missing DB returns Ok(false) here so the caller keeps
        // polling across the producer's boot window. The fail-close decision
        // is made by run_effect at the DEADLINE (see `db_ever_existed`
        // tracking) — "db never appeared by deadline" is an explicit FAIL,
        // not silent poll-exhaustion.
        if !db_path.exists() {
            return Ok(false);
        }
        let conn = match Connection::open_with_flags(
            db_path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY
                | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
        ) {
            Ok(c) => c,
            Err(e) => {
                // DB still being created by the producer — treat as "not yet".
                debug!(
                    target: "elai_orchestrator::wiring_contract",
                    "SSB DB open failed during observable poll: {e}"
                );
                return Ok(false);
            }
        };
        let mut stmt = conn
            .prepare("SELECT COUNT(*) FROM ssb WHERE action_type = ?1")
            .map_err(|e| WiringContractError::SqliteError(e.to_string()))?;
        let count: i64 = stmt
            .query_row([action_type], |row| row.get(0))
            .unwrap_or(0);
        Ok(count > 0)
    }
}

// ---------------------------------------------------------------------------
// Audit driver types
// ---------------------------------------------------------------------------

/// One-shot audit run command struct. The CLI driver constructs this from
/// parsed args; the orchestrator core executes via [`WiringAuditCmd::run`].
#[derive(Debug, Clone)]
pub struct WiringAuditCmd {
    pub config_path: PathBuf,
    pub contracts_dir: PathBuf,
    pub source_root: PathBuf,
    pub ssb_db_path: Option<PathBuf>,
    /// SP2 (WAVE 2): path to the offline-generated symbol graph
    /// (`.claude/tools/elai_graph.json`) used to build the REAL reachability
    /// graph. `None` triggers a walk-up from `source_root` through its
    /// ancestors for the first `.claude/tools/elai_graph.json` that exists
    /// (the canonical artifact lives at the OUTER workflow root, one level
    /// above `.elai_cc`); see [`WiringAuditCmd::graph_path`].
    pub graph_path: Option<PathBuf>,
    pub seed: u64,
    pub scope: WiringAuditScope,
}

impl WiringAuditCmd {
    /// SP2 (WAVE 6 residual fix): resolve the symbol-graph JSON path. An
    /// explicit `graph_path` always wins. Otherwise WALK UP from `source_root`
    /// through its ancestors, returning the first
    /// `<ancestor>/.claude/tools/elai_graph.json` that exists. The canonical
    /// `elai_graph.json` is generated at the OUTER workflow root (it maps
    /// `.elai_cc` but lives one level up at `F:\.ELAI_workflow\.claude\tools\`),
    /// so a `source_root` of `.elai_cc` would not find an inner copy; the
    /// walk-up locates the real artifact regardless of which level the audit
    /// is invoked from. If no ancestor carries the artifact, return the
    /// `source_root`-anchored path so the reachability leg fails CLOSED with a
    /// clear "failed to load symbol graph at <path>" error (R2: missing
    /// evidence is a hard FAIL, never a silent pass). Panic-free: pure
    /// `Option`/iterator resolution, no `unwrap`/`expect`.
    fn graph_path(&self) -> PathBuf {
        if let Some(explicit) = self.graph_path.clone() {
            return explicit;
        }
        self.resolve_default_graph_path()
    }

    /// Walk `source_root` and its ancestors looking for
    /// `.claude/tools/elai_graph.json`; return the first hit, else the
    /// `source_root`-anchored fail-closed default. Split out so the resolution
    /// order is unit-testable independent of an explicit override.
    fn resolve_default_graph_path(&self) -> PathBuf {
        let candidate_under = |dir: &Path| {
            dir.join(".claude").join("tools").join("elai_graph.json")
        };
        // `Path::ancestors` yields self, then each parent up to the filesystem
        // root, so this checks source_root first and stops at the first
        // existing artifact (closest-wins).
        for ancestor in self.source_root.ancestors() {
            let candidate = candidate_under(ancestor);
            if candidate.exists() {
                return candidate;
            }
        }
        // No artifact found anywhere up the tree: return the source_root form
        // so build_reachability_graph_from_source reports a clear hard error
        // against a concrete, expected path (R2 dead-evidence-path-must-scream)
        // rather than silently passing.
        candidate_under(&self.source_root)
    }
}

impl WiringAuditCmd {
    /// Execute the audit. Synchronous wrapper that owns a `current_thread`
    /// tokio runtime for the ObservableEffectRunner. Loads
    /// `[wiring.entrypoints]`, enumerates `*.md` WIRING_CONTRACT blocks,
    /// runs each scope-selected auditor, and returns an aggregated
    /// [`WiringAuditReport`]. The CLI driver renders the report; orchestrator
    /// callers persist it.
    ///
    /// Sink wiring: rows are captured in an in-memory vec and split into
    /// evidence / violations on return. CLI exit codes follow
    /// `WiringAuditReport::is_clean()`.
    pub fn run(&self) -> Result<WiringAuditReport, WiringAuditError> {
        if !self.config_path.exists() {
            return Err(WiringAuditError::ConfigMissing(self.config_path.clone()));
        }
        if !self.contracts_dir.exists() {
            return Err(WiringAuditError::ContractsDirMissing(
                self.contracts_dir.clone(),
            ));
        }

        // Per-audit-run correlator; uuid::Uuid::new_v4 is the accepted boundary
        // for non-deterministic ids in audit output. The determinism harness
        // operates on action_type sequences, not audit ids.
        let audit_id = uuid::Uuid::new_v4(); // entropy-allow: per-audit-run correlator-id-not-replay-compared
        let captured: Arc<std::sync::Mutex<Vec<SsbPayload>>> =
            Arc::new(std::sync::Mutex::new(Vec::new()));

        let entrypoints = WiringEntrypointSetLoader::new(&self.config_path).load()?;
        let contracts_md = collect_contract_markdown_files(&self.contracts_dir)?;

        let mut all_contracts: Vec<WiringContract> = Vec::new();
        for path in &contracts_md {
            let raw = match fs::read_to_string(path) {
                Ok(t) => t,
                Err(e) => {
                    return Err(WiringAuditError::Contract(
                        WiringContractError::IoError(e),
                    ));
                }
            };
            let bn_id = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string();
            if let Ok(c) = parse_wiring_contract_from_markdown(&bn_id, &raw) {
                // Skip contracts that did not actually parse a WIRING_CONTRACT
                // body (parse_wiring_contract returns Err on missing heading).
                // Empty produces + observable_effects + stub_ban + reachable_from
                // is a "found heading but no body" — treat as no-op rather
                // than synthetic evidence.
                let empty = c.produces_symbols.is_empty()
                    && c.consumes_symbols.is_empty()
                    && c.reachable_from.is_empty()
                    && c.stub_ban.is_empty()
                    && c.observable_effects.is_empty()
                    && c.frontend_pair.is_none();
                if !empty {
                    all_contracts.push(c);
                }
            }
        }

        // SP1(c) R1 fail-close (ArchUnit failOnEmptyShould): the contracts
        // dir exists (checked above) but parsed to zero non-empty
        // WIRING_CONTRACT blocks. A wiring gate that finds nothing to check
        // must FAIL, not pass vacuously.
        if all_contracts.is_empty() {
            return Err(WiringAuditError::ZeroContracts(self.contracts_dir.clone()));
        }

        if matches!(self.scope, WiringAuditScope::All | WiringAuditScope::StubBan) {
            let sink = AuditorSink::Channel(spawn_capture_drain(captured.clone()));
            let auditor = StubBanAuditor::new(&[], sink, audit_id)?;
            auditor.scan_production_paths(&self.source_root)?;
        }

        if matches!(
            self.scope,
            WiringAuditScope::All | WiringAuditScope::DkgReachability
        ) {
            // SP1(b) fail-close: an empty entrypoint set can no longer reach
            // this point — WiringEntrypointSetLoader::load now returns
            // Err(EmptyEntrypointSet/AbsentEntrypointSet) and the `?` at the
            // load site above has already aborted run(). This debug_assert
            // documents the invariant for future maintainers; it is NOT the
            // gate (the gate is the loader Err).
            debug_assert!(
                !entrypoints.is_empty(),
                "reachability leg reached with an empty entrypoint set — loader fail-close regressed"
            );

            // SP2 BUILD-NEW: derive the reachability graph from the REAL
            // source call graph (SymbolGraphCache over elai_graph.json),
            // excluding test-file callers (R7/W-8). Replaces the prior
            // fabricated entrypoint->every-produces edge set that made every
            // declared symbol trivially reachable (a self-certifying stub).
            // On graph-load failure the leg FAILS (R2 dead-evidence-path-must-
            // scream) rather than falling back to a self-certifying graph.
            let graph = build_reachability_graph_from_source(
                &self.graph_path(),
                &entrypoints,
            )?;
            let sink = AuditorSink::Channel(spawn_capture_drain(captured.clone()));
            let auditor = DkgReachabilityAuditor::new(Arc::new(graph), sink, audit_id);
            for contract in &all_contracts {
                auditor.audit_contract(contract, &entrypoints);
            }
        }

        if matches!(
            self.scope,
            WiringAuditScope::All | WiringAuditScope::ObservableEffect
        ) {
            let ssb_path = self
                .ssb_db_path
                .clone()
                .unwrap_or_else(|| self.source_root.join(".elai").join("ssb.db"));
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(WiringContractError::IoError)?;
            let observed_failures: usize = runtime.block_on(async {
                let mut failures = 0usize;
                for contract in &all_contracts {
                    for effect in &contract.observable_effects {
                        let sink =
                            AuditorSink::Channel(spawn_capture_drain(captured.clone()));
                        let runner = ObservableEffectRunner::new(&ssb_path, sink, audit_id)
                            .with_poll_interval(Duration::from_secs(1));
                        // SP4(d) fail-close: a failing observable effect emits
                        // a WiringViolation(BrokenObservable) AND is counted
                        // here so it forces a nonzero audit exit. We finish
                        // the remaining effects (to surface every broken
                        // observable in one run) but do NOT discard the Err.
                        if runner.run_effect(effect, &contract.bn_id).await.is_err() {
                            failures += 1;
                        }
                    }
                }
                failures
            });
            if observed_failures > 0 {
                // SP4(d) R4: a BrokenObservable is load-bearing evidence of a
                // wire-dark feature. Propagate it as a hard FAIL rather than
                // relying solely on the violation row being counted later.
                return Err(WiringAuditError::ViolationsPresent(
                    // narrowing guard: observed_failures is bounded by the
                    // contract/effect count, well within u32.
                    u32::try_from(observed_failures).unwrap_or(u32::MAX),
                ));
            }
        }

        let (evidence, violations) = split_captured_payloads(&captured);
        Ok(WiringAuditReport {
            audit_id,
            evidence_count: evidence.len() as u32,
            violation_count: violations.len() as u32,
            evidence,
            violations,
        })
    }
}

/// Spawn the channel-drain thread used by the audit driver to bridge
/// AuditorSink::Channel into the captured vec. Returns the sender. The
/// thread terminates when the sender is dropped (or all clones go out of
/// scope).
fn spawn_capture_drain(
    captured: Arc<std::sync::Mutex<Vec<SsbPayload>>>,
) -> UnboundedSender<SsbPayload> {
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<SsbPayload>();
    std::thread::spawn(move || {
        while let Some(payload) = rx.blocking_recv() {
            if let Ok(mut g) = captured.lock() {
                g.push(payload);
            }
        }
    });
    tx
}

/// SP2 WAVE-3 SCHEMA FIX (R5/R6/R7): build a NAME-keyed [`ReachabilityGraph`]
/// from the real source call graph. Replaces the prior fabricated
/// entrypoint->every-produces edge set (self-certifying), AND fixes a SECOND
/// dead leg discovered downstream: the prior wave read per-node
/// `SymbolNode.calls`, but `codebase_mapper.py` emits EVERY function node with
/// an EMPTY `calls` vec (verified 25491/25491 nodes empty on the live graph).
/// The 206k real call edges live in a SEPARATE TOP-LEVEL `calls` array of
/// `{caller, callee, file, line, ...}` records (bare-name keyed). Reading
/// per-node `calls` therefore yielded ZERO edges => only the entrypoint seed
/// names were reachable => every `produces_symbol` falsely flagged
/// `UnreachableSymbol`. This function now loads the TOP-LEVEL `calls` array.
///
/// POSTURE — OVER-PERMISSIVE (necessary-not-sufficient, by design): edges are
/// keyed by bare symbol name (`caller` -> `callee`, each reduced to its
/// trailing name so `blake3::hash` and a local `hash` collapse to `hash`).
/// No method-resolution or bare-name disambiguation is attempted: if ANY
/// `hash`-named symbol is reachable from an entrypoint, ALL `hash` produces
/// count as reachable. This is intentional. Per the wiring research
/// (docs/strategy/WIRING-ENFORCEMENT-RESEARCH-2026-06-02.md), the Layer-3
/// runtime-coverage leg is the load-bearing BLOCKING backstop; this static
/// leg is a cheap necessary condition that only catches symbols reachable
/// from NO entrypoint at all (genuinely-orphaned). Over-permissive here means
/// false-NEGATIVES (a wire-dark symbol that shares a name with a live one
/// slips this leg) but NOT false-positives (a genuinely-orphaned symbol still
/// fires). Residual: bare-name ambiguity, and method-entrypoints
/// (`Type::method`) collapse to the bare method name.
///
/// R7/W-8 test-caller exclusion (EDGE-LEVEL): because the top-level `calls`
/// record carries the CALLER's `file`, edges whose caller file matches the
/// module's existing [`SKIP_DIR_FRAGMENTS`] / [`SKIP_DIR_PREFIXES`] /
/// [`SKIP_FILE_SUFFIXES`] (same set [`StubBanAuditor`] uses, via
/// [`is_test_source_path`]) are dropped, so a caller that only exists in test
/// code cannot satisfy production reachability. This is finer-grained than
/// the prior wave's node-level exclusion.
///
/// R2 fail-close: a missing or unparseable graph file returns `Err` — the
/// reachability leg never falls back to a self-certifying graph. A non-empty
/// graph file that yields ZERO non-test edges ALSO returns `Err` (R2 dead-
/// evidence-path-must-scream): zero edges over a populated graph means the
/// schema parse silently mismatched, and a 0-edge graph would falsely flag
/// every produces_symbol unreachable.
fn build_reachability_graph_from_source(
    graph_path: &Path,
    entrypoints: &WiringEntrypointSet,
) -> Result<ReachabilityGraph, WiringAuditError> {
    // R2: load via SymbolGraphCache first. Its load() Err is the canonical
    // missing/unparseable-graph fail-close path (missing file, bad JSON, etc.).
    // We use the cache to seed `known_symbols` with every node name; the EDGES
    // come from the top-level `calls` array, which the cache discards.
    let cache = SymbolGraphCache::load(graph_path).map_err(|e| {
        WiringAuditError::Contract(WiringContractError::ParseError(format!(
            "reachability: failed to load symbol graph at {}: {e}",
            graph_path.display()
        )))
    })?;

    let mut graph = ReachabilityGraph::new();

    // Seed entrypoint names so a contract whose produces_symbol IS an
    // entrypoint is trivially reachable (is_reachable checks `ep == target`).
    for ep in entrypoints.iter() {
        graph.add_symbol(entrypoint_match_key(ep));
    }

    // Seed every known node name so `known_symbols` distinguishes
    // "unknown symbol" from "known-but-unreachable" for downstream callers.
    for (_id, node) in cache.iter_nodes() {
        if !node.name.is_empty() {
            graph.add_symbol(symbol_trailing_name(&node.name));
        }
    }

    // Load the REAL edges from the top-level `calls` array. Each record is
    // a `{caller, callee, file, line, ...}` row keyed by bare name; the
    // `file` is the CALLER's file (used for edge-level test exclusion).
    let edge_count = load_top_level_call_edges(graph_path, &mut graph).map_err(|e| {
        WiringAuditError::Contract(WiringContractError::ParseError(format!(
            "reachability: failed to parse top-level `calls` edges at {}: {e}",
            graph_path.display()
        )))
    })?;

    // R2 fail-close on zero edges over a non-empty graph file: a populated
    // graph that produced no non-test edges means the schema parse silently
    // mismatched (the exact failure mode this wave fixes). A 0-edge graph
    // would mark every produces_symbol unreachable, so SCREAM rather than
    // emit a false-positive flood. (cache.node_count() > 0 proves the file
    // was non-empty; an empty file is its own legitimate 0-edge case only if
    // it also has zero nodes — which a real graph never does.)
    if edge_count == 0 && cache.node_count() > 0 {
        return Err(WiringAuditError::Contract(WiringContractError::ParseError(
            format!(
                "reachability: graph at {} has {} nodes but ZERO non-test call edges parsed from the top-level `calls` array; this means the edge schema did not match (fail-closed per R2 — a 0-edge graph would falsely flag every produces_symbol unreachable)",
                graph_path.display(),
                cache.node_count()
            ),
        )));
    }

    Ok(graph)
}

/// SP2 WAVE-3: parse the TOP-LEVEL `calls` array out of the symbol-graph JSON
/// and add one `caller_name -> callee_name` edge per non-test record. Returns
/// the number of edges added.
///
/// The record shape (verified against the live
/// `.claude/tools/elai_graph.json`) is:
/// `{ "caller": <bare name>, "callee": <bare name>, "file": <caller file>,
///    "line": u32, "call_type": ..., "lang": ... }`. Only `caller`, `callee`,
/// and `file` are load-bearing here; all other fields are ignored
/// (forward-compatible). A record missing `caller` or `callee` is skipped.
///
/// R7/W-8 (EDGE-LEVEL): the record's `file` is the CALLER's file, so an edge
/// is dropped when that file is a test path. Both endpoints are reduced to
/// their trailing names so the edge name space matches the entrypoint /
/// produces_symbol convention (`blake3::hash` -> `hash`).
///
/// R2: a parse error (missing key, malformed JSON) propagates as `Err`. A
/// graph whose root is not an object, or which has no `calls` key, yields 0
/// edges (the caller's zero-edge fail-close then decides whether that is a
/// hard FAIL based on node count).
fn load_top_level_call_edges(
    graph_path: &Path,
    graph: &mut ReachabilityGraph,
) -> Result<usize, SymbolGraphCacheError> {
    // One bare-name call edge from the top-level `calls` array. Unknown fields
    // (call_type, caller_type, callee_type, lang, line) are ignored.
    #[derive(Deserialize)]
    struct RawCallEdge {
        #[serde(default)]
        caller: String,
        #[serde(default)]
        callee: String,
        #[serde(default)]
        file: String,
    }
    // Only deserialize the `calls` array, not the whole graph, to bound peak
    // memory — `functions`/`modules`/`structs`/`reverse_calls` are skipped.
    #[derive(Deserialize)]
    struct GraphCallsOnly {
        #[serde(default)]
        calls: Vec<RawCallEdge>,
    }

    let raw = fs::read_to_string(graph_path).map_err(|source| SymbolGraphCacheError::Io {
        path: graph_path.to_path_buf(),
        source,
    })?;
    let parsed: GraphCallsOnly =
        serde_json::from_str(&raw).map_err(|source| SymbolGraphCacheError::Parse {
            path: graph_path.to_path_buf(),
            source,
        })?;

    let mut added = 0usize;
    for edge in &parsed.calls {
        if edge.caller.is_empty() || edge.callee.is_empty() {
            continue;
        }
        // R7/W-8 edge-level: drop edges whose CALLER file is a test path.
        if is_test_source_path(&edge.file) {
            continue;
        }
        let caller = symbol_trailing_name(&edge.caller);
        let callee = symbol_trailing_name(&edge.callee);
        graph.add_edge(caller, callee);
        added += 1;
    }
    Ok(added)
}

/// SP2: reduce an entrypoint string to its match key. `[wiring.entrypoints]`
/// symbols may be a path form (`crates/cli/src/main.rs::main`) or a bare
/// `main`; the BFS matches on the trailing symbol name either way.
fn entrypoint_match_key(ep: &str) -> String {
    symbol_trailing_name(ep)
}

/// SP2: take the trailing symbol name from an id/path. Handles both `::`
/// (Rust path) and the `<file>:<name>:<line>` id form by splitting on the
/// last `::` first, then stripping a trailing `:<digits>` line suffix and
/// taking the segment after the preceding `:`.
fn symbol_trailing_name(s: &str) -> String {
    if let Some(idx) = s.rfind("::") {
        return s[idx + 2..].to_string();
    }
    // `<file>:<name>:<line>` — strip a trailing `:<digits>` then take the
    // segment after the preceding `:`.
    let trimmed = match s.rsplit_once(':') {
        Some((head, tail))
            if !tail.is_empty() && tail.chars().all(|c| c.is_ascii_digit()) =>
        {
            head
        }
        _ => s,
    };
    match trimmed.rsplit_once(':') {
        Some((_, name)) => name.to_string(),
        None => trimmed.to_string(),
    }
}

/// SP2 R7/W-8: is this source path a test file that must be EXCLUDED from
/// the reachability proof? Reuses the module's existing skip constants so
/// the test-exclusion set stays consistent with [`StubBanAuditor`]. File
/// heuristic only — `#[cfg(test)]` inline modules inside a production file
/// are out of scope (the mapper generally omits test code from the graph;
/// see SP2 residual-risk note in workflow/sessions/.../specs/D-WC.json).
fn is_test_source_path(file: &str) -> bool {
    let norm = file.replace('\\', "/");
    if SKIP_DIR_FRAGMENTS.iter().any(|frag| {
        let f = frag.replace('\\', "/");
        norm.contains(&f)
    }) {
        return true;
    }
    if SKIP_DIR_PREFIXES.iter().any(|p| {
        let p = p.replace('\\', "/");
        norm.starts_with(&p)
    }) {
        return true;
    }
    let basename = norm.rsplit('/').next().unwrap_or(norm.as_str());
    SKIP_FILE_SUFFIXES.iter().any(|suf| basename.ends_with(suf))
}

fn split_captured_payloads(
    captured: &Arc<std::sync::Mutex<Vec<SsbPayload>>>,
) -> (Vec<WiringEvidencePayload>, Vec<WiringViolationPayload>) {
    let guard = match captured.lock() {
        Ok(g) => g,
        Err(p) => p.into_inner(),
    };
    let mut evidence = Vec::new();
    let mut violations = Vec::new();
    for p in guard.iter() {
        match p {
            SsbPayload::WiringEvidence(e) => evidence.push(e.clone()),
            SsbPayload::WiringViolation(v) => violations.push(v.clone()),
            _ => {}
        }
    }
    (evidence, violations)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WiringAuditScope {
    All,
    StubBan,
    DkgReachability,
    ObservableEffect,
}

impl WiringAuditScope {
    pub fn parse(value: &str) -> Result<Self, String> {
        match value.to_ascii_lowercase().as_str() {
            "all" => Ok(Self::All),
            "stub-ban" | "stub_ban" => Ok(Self::StubBan),
            "dkg-reachability" | "dkg_reachability" => Ok(Self::DkgReachability),
            "observable-effect" | "observable_effect" => Ok(Self::ObservableEffect),
            other => Err(format!(
                "unknown wiring-audit scope {other:?}; expected all|stub-ban|dkg-reachability|observable-effect"
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WiringAuditReport {
    pub audit_id: uuid::Uuid,
    pub evidence_count: u32,
    pub violation_count: u32,
    #[serde(default)]
    pub violations: Vec<WiringViolationPayload>,
    #[serde(default)]
    pub evidence: Vec<WiringEvidencePayload>,
}

impl WiringAuditReport {
    pub fn is_clean(&self) -> bool {
        self.violation_count == 0
    }
}

#[derive(Debug, Error)]
pub enum WiringAuditError {
    #[error(transparent)]
    Contract(#[from] WiringContractError),
    #[error("config file {0} does not exist")]
    ConfigMissing(PathBuf),
    #[error("contracts dir {0} does not exist")]
    ContractsDirMissing(PathBuf),
    /// SP1(c) R1 / ArchUnit failOnEmptyShould: the contracts dir is present
    /// but contains zero parseable WIRING_CONTRACT blocks. A gate that finds
    /// nothing to check must FAIL, not pass vacuously.
    #[error("contracts dir {0} contains no parseable WIRING_CONTRACT blocks; a gate with zero contracts is a fail-closed error (R1 failOnEmptyShould)")]
    ZeroContracts(PathBuf),
    /// SP2/SP4: a reachability or observable leg detected wiring violations
    /// that must propagate as a nonzero exit. Distinct from a clean report
    /// with violation_count>0 so the CLI can render the cause; carries the
    /// violation count for the operator.
    #[error("wiring audit found {0} reachability/observable violation(s); see report for detail")]
    ViolationsPresent(u32),
}

// ---------------------------------------------------------------------------
// Minimal Markdown WIRING_CONTRACT parser
// ---------------------------------------------------------------------------

/// Parse a single WIRING_CONTRACT block from markdown content. Mirrors the
/// shape recognised by `.claude/tools/wiring_contract_check.py` (the
/// canonical Python prototype).
///
/// Recognises top-level keys:
///   `produces_symbols:`     followed by `  - <symbol>` lines
///   `consumes_symbols:`     same shape
///   `reachable_from:`       same shape
///   `stub_ban:`             same shape (each `- ` entry becomes a regex)
///   `frontend_pair:`        single-line scalar
///   `observable_effect:`    indented block with `trigger:` / `expect_ssb:` /
///                           `expect_artifact:` / `timeout_secs:`
///
/// `bn_id` is supplied by the caller (typically the markdown filename without
/// extension).
pub fn parse_wiring_contract_from_markdown(
    bn_id: &str,
    markdown: &str,
) -> Result<WiringContract, WiringContractError> {
    // Locate the WIRING_CONTRACT heading. Accept both `## WIRING_CONTRACT`
    // and `## WIRING_CONTRACT { ... }` and case-insensitive variants.
    let lower = markdown.to_ascii_lowercase();
    let start = lower
        .find("wiring_contract")
        .ok_or_else(|| WiringContractError::ParseError("no WIRING_CONTRACT block".into()))?;
    let after = &markdown[start..];
    // Stop at next top-level section (line starting with `## ` other than
    // the WIRING_CONTRACT one) or end of file.
    let mut block_end = after.len();
    for (idx, line) in after.split_inclusive('\n').enumerate() {
        if idx == 0 {
            continue;
        }
        if line.starts_with("## ") {
            let off: usize = after.split_inclusive('\n').take(idx).map(|s| s.len()).sum();
            block_end = off;
            break;
        }
    }
    let block = &after[..block_end];

    let mut contract = WiringContract {
        bn_id: bn_id.to_owned(),
        ..Default::default()
    };

    let mut current_list: Option<&str> = None;
    let mut current_effect: Option<ObservableEffect> = None;

    for raw_line in block.lines() {
        let trimmed = raw_line.trim_end();
        let leading = trimmed.trim_start();
        let lower_trim = leading.to_ascii_lowercase();

        // List markers (key match on lowercase; value extracted from cased form).
        if lower_trim.starts_with("produces_symbols:") {
            finish_effect_block(&mut contract, &mut current_effect);
            current_list = Some("produces");
            continue;
        }
        if lower_trim.starts_with("consumes_symbols:") {
            finish_effect_block(&mut contract, &mut current_effect);
            current_list = Some("consumes");
            continue;
        }
        if lower_trim.starts_with("reachable_from:") {
            finish_effect_block(&mut contract, &mut current_effect);
            current_list = Some("reachable_from");
            continue;
        }
        if lower_trim.starts_with("stub_ban:") {
            finish_effect_block(&mut contract, &mut current_effect);
            current_list = Some("stub_ban");
            continue;
        }
        if lower_trim.starts_with("frontend_pair:") {
            finish_effect_block(&mut contract, &mut current_effect);
            current_list = None;
            // Slice the cased `leading` at the colon to preserve mixed case.
            let after_colon = leading
                .find(':')
                .map(|i| &leading[i + 1..])
                .unwrap_or("");
            let value = after_colon.trim();
            if !value.is_empty() {
                contract.frontend_pair = Some(value.to_owned());
            }
            continue;
        }
        if lower_trim.starts_with("observable_effect:") {
            finish_effect_block(&mut contract, &mut current_effect);
            current_list = None;
            current_effect = Some(ObservableEffect {
                trigger: String::new(),
                expect_ssb: None,
                expect_artifact: None,
                timeout_secs: 30,
            });
            continue;
        }

        // Observable-effect interior keys — value from cased form.
        if let Some(effect) = current_effect.as_mut() {
            let cased_after = |prefix_lower: &str| -> Option<&str> {
                if !lower_trim.starts_with(prefix_lower) {
                    return None;
                }
                // Walk to the colon in the original cased text.
                leading.find(':').map(|i| leading[i + 1..].trim())
            };
            if let Some(value) = cased_after("trigger:") {
                effect.trigger = value.trim_matches('"').to_owned();
                continue;
            }
            if let Some(value) = cased_after("expect_ssb:") {
                let v = value.trim_matches('"');
                if !v.is_empty() {
                    effect.expect_ssb = Some(v.to_owned());
                }
                continue;
            }
            if let Some(value) = cased_after("expect_artifact:") {
                let v = value.trim_matches('"');
                if !v.is_empty() {
                    effect.expect_artifact = Some(v.to_owned());
                }
                continue;
            }
            if let Some(value) = cased_after("timeout_secs:") {
                if let Ok(n) = value.parse::<u32>() {
                    effect.timeout_secs = n;
                }
                continue;
            }
        }

        // List entries.
        let entry_text = match trimmed.trim_start().strip_prefix("- ") {
            Some(text) => text,
            None => continue,
        };
        match current_list {
            Some("produces") => contract.produces_symbols.push(entry_text.to_owned()),
            Some("consumes") => contract.consumes_symbols.push(entry_text.to_owned()),
            Some("reachable_from") => contract.reachable_from.push(entry_text.to_owned()),
            Some("stub_ban") => contract.stub_ban.push(StubBanEntry {
                pattern: entry_text.to_owned(),
                description: format!("{bn_id} contract stub_ban: {entry_text}"),
            }),
            _ => {}
        }
    }
    finish_effect_block(&mut contract, &mut current_effect);

    Ok(contract)
}

// Helper kept at module scope so the parser stays readable. Underscore
// prefix prevents clippy from flagging "may shadow" with a free-fn name.
fn finish_effect_block(contract: &mut WiringContract, current_effect: &mut Option<ObservableEffect>) {
    if let Some(effect) = current_effect.take() {
        if !effect.trigger.is_empty() {
            contract.observable_effects.push(effect);
        }
    }
}

// ---------------------------------------------------------------------------
// Markdown contracts directory scanner (stdlib walk + extension filter — no
// globset; DRIFT-3 / D3).
// ---------------------------------------------------------------------------

/// Enumerate `*.md` files under `dir` (recursive) using stdlib WalkDir.
/// Returns paths in sorted order so audit output is deterministic.
pub fn collect_contract_markdown_files(dir: &Path) -> Result<Vec<PathBuf>, WiringContractError> {
    let mut out = Vec::new();
    for entry in WalkDir::new(dir).follow_links(false) {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                debug!(
                    target: "elai_orchestrator::wiring_contract",
                    "walkdir error: {e}"
                );
                continue;
            }
        };
        let path = entry.path();
        if !entry.file_type().is_file() {
            continue;
        }
        match path.extension().and_then(|e| e.to_str()) {
            Some(ext) if ext.eq_ignore_ascii_case("md") => out.push(path.to_path_buf()),
            _ => continue,
        }
    }
    out.sort();
    Ok(out)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::sync::Mutex;
    use tempfile::TempDir;

    fn capture_sink() -> (AuditorSink, Arc<Mutex<Vec<SsbPayload>>>) {
        let buf = Arc::new(Mutex::new(Vec::<SsbPayload>::new()));
        (AuditorSink::Capture(buf.clone()), buf)
    }

    // -----------------------------------------------------------------------
    // StubBanAuditor
    // -----------------------------------------------------------------------

    #[test]
    fn stub_ban_skips_test_paths() {
        let p = Path::new("crates/foo/tests/bar.rs");
        assert!(StubBanAuditor::should_skip_path(p));
        let q = Path::new("crates/foo/src/bar_contract.rs");
        assert!(StubBanAuditor::should_skip_path(q));
        let r = Path::new("crates/foo/src/bar_test.rs");
        assert!(StubBanAuditor::should_skip_path(r));
        let s = Path::new("target/debug/foo.rs");
        assert!(StubBanAuditor::should_skip_path(s));
        let t = Path::new("crates/foo/src/lib.rs");
        assert!(!StubBanAuditor::should_skip_path(t));
    }

    #[test]
    fn stub_ban_detects_todo_macro() {
        let (sink, buf) = capture_sink();
        let auditor = StubBanAuditor::new(&[], sink, uuid::Uuid::nil()).expect("auditor");
        let count = auditor.scan_file_contents(
            Path::new("crates/foo/src/lib.rs"),
            "fn a() {\n    todo!()\n}\n",
        );
        assert_eq!(count, 1);
        let captured = buf.lock().expect("lock");
        assert_eq!(captured.len(), 1);
        match &captured[0] {
            SsbPayload::WiringViolation(p) => {
                assert_eq!(p.violation_kind, WiringViolationKind::StubPattern);
                assert_eq!(p.line, 2);
            }
            other => panic!("unexpected payload {other:?}"),
        }
    }

    #[test]
    fn stub_ban_detects_unimplemented_macro() {
        let (sink, buf) = capture_sink();
        let auditor = StubBanAuditor::new(&[], sink, uuid::Uuid::nil()).expect("auditor");
        // Statement-context unimplemented!() — bare on its own line. Inline
        // forms (`fn a() { unimplemented!() }`) are intentionally not
        // flagged after C-27 BLOCK-1: string-literal occurrences inside
        // writeln!/format! generators were producing 19 false-positives
        // across the live codebase. The auditor cares about unfinished
        // function bodies in canonical layout.
        let count = auditor.scan_file_contents(
            Path::new("crates/foo/src/lib.rs"),
            "fn a() {\n    unimplemented!()\n}\n",
        );
        assert_eq!(count, 1);
        assert_eq!(buf.lock().expect("lock").len(), 1);
    }

    #[test]
    fn stub_ban_detects_exact_line_ok_default_default() {
        let (sink, _buf) = capture_sink();
        let auditor = StubBanAuditor::new(&[], sink, uuid::Uuid::nil()).expect("auditor");
        // Exact-line match (anchored).
        let count1 = auditor.scan_file_contents(
            Path::new("crates/foo/src/lib.rs"),
            "fn a() -> Result<X, E> {\n    Ok(Default::default())\n}\n",
        );
        assert_eq!(count1, 1, "exact-line Ok(Default::default()) must match");
    }

    #[test]
    fn stub_ban_ignores_inline_default_default() {
        let (sink, buf) = capture_sink();
        let auditor = StubBanAuditor::new(&[], sink, uuid::Uuid::nil()).expect("auditor");
        // Not exact-line (additional content on the line).
        auditor.scan_file_contents(
            Path::new("crates/foo/src/lib.rs"),
            "let x = Ok(Default::default()); // legit\n",
        );
        // The 'stub'/'deferred' patterns also don't appear here. Only
        // the Ok(Default::default()) rule could fire; it shouldn't.
        let captured = buf.lock().expect("lock");
        for p in captured.iter() {
            if let SsbPayload::WiringViolation(v) = p {
                assert_ne!(
                    v.pattern, "ok_default_default_exact_line",
                    "inline Ok(Default::default()) must not trip the exact-line rule"
                );
            }
        }
    }

    #[test]
    fn stub_ban_skips_test_path_even_when_pattern_present() {
        let (sink, buf) = capture_sink();
        let auditor = StubBanAuditor::new(&[], sink, uuid::Uuid::nil()).expect("auditor");
        let count = auditor.scan_file_contents(
            Path::new("crates/foo/tests/bar.rs"),
            "fn x() { todo!() }\n",
        );
        assert_eq!(count, 0);
        assert!(buf.lock().expect("lock").is_empty());
    }

    #[test]
    fn stub_ban_walk_respects_extension_allowlist() {
        let tmp = TempDir::new().expect("tempdir");
        let prod_path = tmp.path().join("src").join("lib.rs");
        fs::create_dir_all(prod_path.parent().expect("parent")).expect("mkdir");
        let mut f = fs::File::create(&prod_path).expect("create");
        // Bare-statement todo!() — the post-BLOCK-1 regex only matches
        // when the macro is on a line by itself (with optional `;` and
        // trailing comment).
        writeln!(f, "fn a() {{").expect("write");
        writeln!(f, "    todo!()").expect("write");
        writeln!(f, "}}").expect("write");
        // Add a non-allowlisted extension that ALSO contains the pattern —
        // it should be skipped because of the extension filter.
        let bin_path = tmp.path().join("src").join("blob.bin");
        let mut g = fs::File::create(&bin_path).expect("create");
        writeln!(g, "    todo!()").expect("write");

        let (sink, buf) = capture_sink();
        let auditor = StubBanAuditor::new(&[], sink, uuid::Uuid::nil()).expect("auditor");
        let count = auditor.scan_production_paths(tmp.path()).expect("scan");
        assert_eq!(count, 1, "exactly one violation from src/lib.rs");
        assert_eq!(buf.lock().expect("lock").len(), 1);
    }

    // -----------------------------------------------------------------------
    // WiringEntrypointSetLoader
    // -----------------------------------------------------------------------

    #[test]
    fn entrypoint_loader_absent_section_is_err() {
        // SP1 R2 fail-close: an absent [wiring.entrypoints] section is now a
        // hard FAIL (was Ok(empty) pre-fix). A dead gate input must scream.
        let tmp = TempDir::new().expect("tempdir");
        let cfg = tmp.path().join("elai_cc.toml");
        fs::write(&cfg, "[tools.routing_rules]\nenabled = true\n").expect("write");
        let loader = WiringEntrypointSetLoader::new(&cfg);
        let err = loader.load().expect_err("absent section must be Err");
        assert!(
            matches!(err, WiringContractError::AbsentEntrypointSet(_)),
            "expected AbsentEntrypointSet, got {err:?}"
        );
    }

    #[test]
    fn entrypoint_loader_empty_symbols_is_err() {
        // SP1 R1 fail-close: a present-but-empty symbols list is now a hard
        // FAIL (was Ok(empty) pre-fix). An empty entrypoint set cannot prove
        // reachability, so it is missing evidence, not an empty work set.
        let tmp = TempDir::new().expect("tempdir");
        let cfg = tmp.path().join("elai_cc.toml");
        fs::write(&cfg, "[wiring.entrypoints]\nsymbols = []\n").expect("write");
        let loader = WiringEntrypointSetLoader::new(&cfg);
        let err = loader.load().expect_err("empty symbols must be Err");
        assert!(
            matches!(err, WiringContractError::EmptyEntrypointSet(_)),
            "expected EmptyEntrypointSet, got {err:?}"
        );
    }

    #[test]
    fn entrypoint_loader_present_section_returns_symbols() {
        let tmp = TempDir::new().expect("tempdir");
        let cfg = tmp.path().join("elai_cc.toml");
        fs::write(
            &cfg,
            "[wiring.entrypoints]\nsymbols = [\"crates/cli/src/main.rs::main\", \"foo::bar\"]\n",
        )
        .expect("write");
        let loader = WiringEntrypointSetLoader::new(&cfg);
        let set = loader.load().expect("load");
        assert_eq!(set.entrypoints.len(), 2);
        assert_eq!(set.entrypoints[0], "crates/cli/src/main.rs::main");
        assert_eq!(set.entrypoints[1], "foo::bar");
    }

    // -----------------------------------------------------------------------
    // DkgReachabilityAuditor
    // -----------------------------------------------------------------------

    #[test]
    fn dkg_reachable_emits_evidence() {
        let (sink, buf) = capture_sink();
        let mut g = ReachabilityGraph::new();
        g.add_edge("ep1", "DeterminismConfig");
        let auditor = DkgReachabilityAuditor::new(Arc::new(g), sink, uuid::Uuid::nil());
        let contract = WiringContract {
            bn_id: "C-27".into(),
            produces_symbols: vec!["DeterminismConfig".into()],
            ..Default::default()
        };
        let entrypoints = WiringEntrypointSet::new(vec!["ep1".into()]);
        let outcome = auditor.audit_contract(&contract, &entrypoints);
        assert_eq!(outcome.evidence_emitted, 1);
        assert_eq!(outcome.violations_emitted, 0);
        let captured = buf.lock().expect("lock");
        assert_eq!(captured.len(), 1);
        assert!(matches!(captured[0], SsbPayload::WiringEvidence(_)));
    }

    #[test]
    fn dkg_unreachable_emits_violation() {
        let (sink, buf) = capture_sink();
        let g = ReachabilityGraph::new();
        let auditor = DkgReachabilityAuditor::new(Arc::new(g), sink, uuid::Uuid::nil());
        let contract = WiringContract {
            bn_id: "C-27".into(),
            produces_symbols: vec!["NotInGraph".into()],
            ..Default::default()
        };
        let entrypoints = WiringEntrypointSet::new(vec!["ep1".into()]);
        let outcome = auditor.audit_contract(&contract, &entrypoints);
        assert_eq!(outcome.evidence_emitted, 0);
        assert_eq!(outcome.violations_emitted, 1);
        let captured = buf.lock().expect("lock");
        assert_eq!(captured.len(), 1);
        match &captured[0] {
            SsbPayload::WiringViolation(v) => {
                assert_eq!(v.violation_kind, WiringViolationKind::UnreachableSymbol);
                assert_eq!(v.pattern, "NotInGraph");
            }
            other => panic!("unexpected payload {other:?}"),
        }
    }

    // -----------------------------------------------------------------------
    // build_reachability_graph_from_source (SP2 WAVE-3 top-level `calls` fix)
    // -----------------------------------------------------------------------

    /// Write a minimal symbol-graph JSON whose shape matches the real
    /// `.claude/tools/elai_graph.json`: a `functions` map (node names) plus a
    /// TOP-LEVEL `calls` array of `{caller, callee, file, ...}` records. Each
    /// function node carries an EMPTY per-node `calls` vec, exactly like the
    /// live mapper output — proving the reachability builder must read the
    /// top-level array, not per-node `calls`.
    fn write_graph_fixture(dir: &Path) -> PathBuf {
        let p = dir.join("elai_graph.json");
        let json = serde_json::json!({
            "functions": {
                "src/lib.rs:main": {
                    "name": "main", "file": "src/lib.rs", "line": 1,
                    "kind": "function", "calls": []
                },
                "src/lib.rs:do_work": {
                    "name": "do_work", "file": "src/lib.rs", "line": 9,
                    "kind": "function", "calls": []
                },
                "src/lib.rs:leaf": {
                    "name": "leaf", "file": "src/lib.rs", "line": 20,
                    "kind": "function", "calls": []
                },
                "src/lib.rs:only_called_from_test": {
                    "name": "only_called_from_test", "file": "src/lib.rs",
                    "line": 30, "kind": "function", "calls": []
                }
            },
            // The real edges live HERE, top-level, bare-name keyed.
            "calls": [
                { "caller": "main", "callee": "do_work",
                  "file": "src/lib.rs", "line": 3, "call_type": "direct",
                  "lang": "rust" },
                { "caller": "do_work", "callee": "leaf",
                  "file": "src/lib.rs", "line": 12, "call_type": "direct",
                  "lang": "rust" },
                // A `path::name` callee must collapse to its trailing name.
                { "caller": "do_work", "callee": "blake3::hash",
                  "file": "src/lib.rs", "line": 13, "call_type": "direct",
                  "lang": "rust" },
                // R7: this edge's CALLER file is a test path -> excluded.
                { "caller": "test_helper", "callee": "only_called_from_test",
                  "file": "crates/foo/tests/it.rs", "line": 4,
                  "call_type": "direct", "lang": "rust" }
            ],
            "reverse_calls": {},
            "modules": {},
            "structs": {},
            "metadata": { "total_functions": 4, "total_calls": 4 }
        });
        fs::write(&p, serde_json::to_string(&json).expect("ser")).expect("write");
        p
    }

    #[test]
    fn reachability_loads_top_level_call_edges() {
        // Regression guard for the WAVE-3 schema bug: per-node `calls` are
        // empty, so the builder MUST read the top-level `calls` array. If it
        // reads per-node `calls` again, this graph has zero edges and the
        // assertions below fail.
        let tmp = TempDir::new().expect("tempdir");
        let graph_path = write_graph_fixture(tmp.path());
        let entrypoints = WiringEntrypointSet::new(vec!["main".into()]);

        let graph = build_reachability_graph_from_source(&graph_path, &entrypoints)
            .expect("graph builds from top-level calls");

        // >0 edges loaded (the whole point of the fix).
        let edge_total: usize = graph
            .forward_edges
            .values()
            .map(|s| s.len())
            .sum();
        assert!(
            edge_total > 0,
            "expected real call edges from the top-level `calls` array, got 0 \
             (builder regressed to empty per-node calls)"
        );
        // main -> do_work and do_work -> {leaf, hash} must exist.
        assert!(graph.forward_edges.get("main").is_some_and(|s| s.contains("do_work")));
        let dw = graph.forward_edges.get("do_work").expect("do_work edges");
        assert!(dw.contains("leaf"));
        // `blake3::hash` collapsed to trailing name `hash`.
        assert!(dw.contains("hash"));

        // R7 edge-level test-caller exclusion: the test-file caller's edge
        // (`test_helper -> only_called_from_test`) must NOT be present.
        assert!(
            graph.forward_edges.get("test_helper").is_none(),
            "test-file caller edge must be excluded (R7/W-8)"
        );

        // End-to-end reachability over the REAL edges: leaf + hash reachable
        // from `main`, but `only_called_from_test` is NOT (its only edge was
        // a test-file caller, now excluded).
        let auditor = DkgReachabilityAuditor::new(
            Arc::new(graph),
            capture_sink().0,
            uuid::Uuid::nil(),
        );
        assert!(auditor.is_reachable(&entrypoints, "leaf").is_some());
        assert!(auditor.is_reachable(&entrypoints, "hash").is_some());
        assert!(
            auditor.is_reachable(&entrypoints, "only_called_from_test").is_none(),
            "symbol reachable only via an excluded test-file caller must be unreachable"
        );
    }

    #[test]
    fn reachability_fails_closed_on_zero_edges_with_nodes() {
        // R2 dead-evidence-path-must-scream: a populated graph (nodes present)
        // whose top-level `calls` array is empty/absent => ZERO edges => hard
        // FAIL, not a 0-edge false-positive machine.
        let tmp = TempDir::new().expect("tempdir");
        let p = tmp.path().join("elai_graph.json");
        let json = serde_json::json!({
            "functions": {
                "src/lib.rs:main": {
                    "name": "main", "file": "src/lib.rs", "line": 1,
                    "kind": "function", "calls": []
                }
            },
            "calls": []
        });
        fs::write(&p, serde_json::to_string(&json).expect("ser")).expect("write");
        let entrypoints = WiringEntrypointSet::new(vec!["main".into()]);
        let result = build_reachability_graph_from_source(&p, &entrypoints);
        assert!(
            result.is_err(),
            "0 edges over a non-empty node set must fail closed (R2)"
        );
    }

    #[test]
    fn reachability_fails_closed_on_missing_graph() {
        // R2: a missing graph file is a dead evidence path -> Err.
        let entrypoints = WiringEntrypointSet::new(vec!["main".into()]);
        let result = build_reachability_graph_from_source(
            Path::new("does/not/exist/elai_graph.json"),
            &entrypoints,
        );
        assert!(result.is_err(), "missing graph must fail closed (R2)");
    }

    // -----------------------------------------------------------------------
    // ObservableEffectRunner
    // -----------------------------------------------------------------------

    #[tokio::test(flavor = "current_thread", start_paused = true)]
    async fn observable_runner_timeout_emits_broken_observable() {
        let tmp = TempDir::new().expect("tempdir");
        let db_path = tmp.path().join("ssb.db");
        // Don't create the DB — runner must handle missing DB gracefully.
        let (sink, buf) = capture_sink();
        let runner = ObservableEffectRunner::new(db_path, sink, uuid::Uuid::nil())
            .with_poll_interval(Duration::from_millis(10));
        let effect = ObservableEffect {
            trigger: "elai wiring audit".into(),
            expect_ssb: Some("nonexistent_action".into()),
            expect_artifact: None,
            timeout_secs: 1,
        };
        let result = runner.run_effect(&effect, "C-27").await;
        assert!(result.is_err(), "must error on timeout");
        let captured = buf.lock().expect("lock");
        assert_eq!(captured.len(), 1);
        match &captured[0] {
            SsbPayload::WiringViolation(v) => {
                assert_eq!(v.violation_kind, WiringViolationKind::BrokenObservable);
                assert_eq!(v.pattern, "elai wiring audit");
            }
            other => panic!("unexpected payload {other:?}"),
        }
    }

    #[tokio::test(flavor = "current_thread", start_paused = true)]
    async fn observable_runner_artifact_present_succeeds() {
        let tmp = TempDir::new().expect("tempdir");
        let artifact = tmp.path().join("artifact.txt");
        fs::write(&artifact, "ok").expect("write");
        let (sink, buf) = capture_sink();
        let runner = ObservableEffectRunner::new(
            tmp.path().join("ssb.db"),
            sink,
            uuid::Uuid::nil(),
        )
        .with_poll_interval(Duration::from_millis(10));
        let effect = ObservableEffect {
            trigger: "elai wiring audit".into(),
            expect_ssb: None,
            expect_artifact: Some(artifact.to_string_lossy().to_string()),
            timeout_secs: 1,
        };
        let result = runner.run_effect(&effect, "C-27").await;
        assert!(result.is_ok(), "must succeed when artifact present");
        assert!(buf.lock().expect("lock").is_empty());
    }

    // -----------------------------------------------------------------------
    // Markdown WIRING_CONTRACT parser
    // -----------------------------------------------------------------------

    #[test]
    fn parser_extracts_produces_consumes_and_stub_ban() {
        let md = r#"
# Foo

## WIRING_CONTRACT

produces_symbols:
  - DeterminismConfig
  - WiringContract
consumes_symbols:
  - SseededClock
reachable_from:
  - elai_cc.toml [wiring.entrypoints]
stub_ban:
  - prefix_cache_key MUST use blake3
frontend_pair: dashboard-v2/src/WiringPanel.svelte
observable_effect:
  trigger: "elai wiring audit"
  expect_ssb: "wiring_evidence"
  timeout_secs: 30

## Next

C-28.
"#;
        let parsed = parse_wiring_contract_from_markdown("C-27", md).expect("parse");
        assert_eq!(parsed.produces_symbols.len(), 2);
        assert_eq!(parsed.consumes_symbols.len(), 1);
        assert_eq!(parsed.reachable_from.len(), 1);
        assert_eq!(parsed.stub_ban.len(), 1);
        assert_eq!(
            parsed.frontend_pair.as_deref(),
            Some("dashboard-v2/src/WiringPanel.svelte")
        );
        assert_eq!(parsed.observable_effects.len(), 1);
        assert_eq!(parsed.observable_effects[0].trigger, "elai wiring audit");
        assert_eq!(
            parsed.observable_effects[0].expect_ssb.as_deref(),
            Some("wiring_evidence")
        );
    }
}
