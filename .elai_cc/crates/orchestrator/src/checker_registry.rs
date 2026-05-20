//! D-11 — Constructive-checker registry (Mode A operational substrate).
//!
//! When the D-10 mode-routing predicate selects
//! [`crate::grounding_mode::GroundingMode::Constructive`] for a claim,
//! the orchestrator dispatches the claim through this registry. Each
//! registered checker is an external, deterministic verifier whose
//! Pass/Fail/Inapplicable verdict is *ground truth* — the policy model
//! does not get a vote (Cohen Boxed Myopic / Everitt current-RF).
//!
//! Grounding:
//!   * docs/specs/guide/real-problem-solving-grounded-2026-05-19.md §4.2
//!     (checker layout) + §1 (Mode A constructive grading).
//!   * arxiv:2310.04406 (constructive proof checking — every operational
//!     claim must terminate in a syntactic check the checker accepts).
//!   * arxiv:2211.12588 Program-of-Thought (T1, TMLR 2023) — Python
//!     interpreter as ground truth (+12pp over CoT on numerical
//!     benchmarks).
//!   * arxiv:2402.01030 CodeAct (T1, ICML 2024) — code execution as the
//!     agent's unified action space.
//!
//! Closed taxonomy: [`CheckerKind`] is a frozen enum. Adding a variant
//! is a plan-level schema change (the D-9 L3 locked-keys registry
//! freezes the surface). The agent cannot extend the closed set at
//! runtime — every registered checker maps to exactly one
//! [`CheckerKind`] variant.
//!
//! Object-safety: [`Checker`] is a trait object via a boxed-future
//! return type (`Pin<Box<dyn Future + Send>>`) because native async-fn
//! in traits is not yet dyn-compatible. The same erasure pattern lives
//! in D-3 (multi-verifier consensus) — see
//! `crates/orchestrator/src/agent_pool.rs::ClaimRoutedEmitter`.
//!
//! Wiring:
//!   * producer: `crate::agent_pool::dispatch_constructive_checkers`
//!     iterates [`CheckerRegistry::run_all_applicable`] and emits one
//!     `checker_result_emitted` SSB row per [`CheckerResult`].
//!   * consumer: dashboard `CheckerRegistryPanel.svelte` reads via the
//!     `get_checker_results_recent` + `get_checker_registry_state`
//!     Tauri commands.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use crate::claim::{Claim, ClaimType};

/// Closed taxonomy of supported checker kinds. Frozen per D-9 locked
/// keys — adding a variant is a plan-level schema change. Naming is
/// the on-wire snake_case label used in SSB rows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckerKind {
    /// SymPy symbolic + numeric equality check (shell-out to
    /// `.elai_cc/checkers/math/sympy.py`).
    Sympy,
    /// Z3 SMT solver (shell-out to `.elai_cc/checkers/math/z3.py`).
    /// Deferred — registry slot reserved; runtime returns
    /// `Inapplicable { reason: "checker script not yet deployed" }`
    /// until the helper lands.
    Z3,
    /// `cargo check` on a candidate Rust file (shell-out to
    /// `.elai_cc/checkers/code/compile_rust.sh`).
    CompileRust,
    /// Pure-Rust `regex` match against a candidate string. In-process,
    /// no subprocess — the regex crate is already in the workspace.
    RegexMatch,
    /// JSON-schema validation. Reserved slot; no `jsonschema` crate is
    /// vendored — runtime returns Inapplicable until either a vendored
    /// pure-Rust validator is wired or the dep is approved.
    JsonSchemaValidate,
    /// Numeric range / interval predicate on a candidate value. Pure
    /// in-process arithmetic — no subprocess.
    NumericRange,
}

impl CheckerKind {
    /// Stable snake-case wire label (matches the serde tag).
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Sympy => "sympy",
            Self::Z3 => "z3",
            Self::CompileRust => "compile_rust",
            Self::RegexMatch => "regex_match",
            Self::JsonSchemaValidate => "json_schema_validate",
            Self::NumericRange => "numeric_range",
        }
    }

    /// Every variant in declaration order. Used by the dashboard to
    /// render the registry table without hard-coding the list.
    #[must_use]
    pub const fn all() -> [CheckerKind; 6] {
        [
            Self::Sympy,
            Self::Z3,
            Self::CompileRust,
            Self::RegexMatch,
            Self::JsonSchemaValidate,
            Self::NumericRange,
        ]
    }
}

/// Stable identifier for a registered checker entry. The producer (the
/// orchestrator) assigns the id at register time; consumers reference
/// the id when correlating SSB rows to specific checkers.
pub type CheckerId = String;

/// Closed outcome of a single checker invocation. Carries enough
/// context for the dashboard to render Pass/Fail/Inapplicable/Error
/// without re-running the checker.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CheckerResult {
    /// Checker verified the claim — Mode A ground truth.
    Pass,
    /// Checker rejected the claim. `reason` is a bounded human-readable
    /// string the dashboard renders; never empty.
    Fail { reason: String },
    /// Checker is not applicable to this claim (wrong claim type, missing
    /// candidate field, helper script not deployed). Not a verdict —
    /// the orchestrator should try the next applicable checker.
    Inapplicable { reason: String },
    /// Infrastructure failure (subprocess spawn, timeout, JSON parse
    /// error). Distinct from Fail: a Fail is a definitive verdict,
    /// an Error means the checker could not produce one.
    Error { source: String },
}

impl CheckerResult {
    /// Stable snake-case kind label for SSB rows + dashboard rendering.
    #[must_use]
    pub fn result_kind(&self) -> &'static str {
        match self {
            Self::Pass => "pass",
            Self::Fail { .. } => "fail",
            Self::Inapplicable { .. } => "inapplicable",
            Self::Error { .. } => "error",
        }
    }

    /// True iff this is a definitive Pass verdict.
    #[must_use]
    pub fn is_pass(&self) -> bool {
        matches!(self, Self::Pass)
    }
}

/// Erased async checker trait. The return type is a boxed pinned
/// future to keep the trait dyn-compatible; concrete implementations
/// can be written with `async fn` and wrapped via `Box::pin`.
///
/// Implementations MUST be deterministic on their `(claim,
/// candidate)` inputs — the registry calls `check` from the
/// orchestrator's audit loop, and non-determinism would invalidate
/// the Mode A "checker is ground truth" property.
pub trait Checker: Send + Sync {
    /// The closed-enum kind of this checker — used by the dashboard +
    /// SSB row to group results by kind without inspecting the
    /// implementation type.
    fn kind(&self) -> CheckerKind;

    /// True iff the checker is applicable to `claim`. The registry
    /// filters by this predicate before calling `check` — non-
    /// applicable claims must not produce a Fail.
    fn applies_to(&self, claim: &Claim) -> bool;

    /// Run the checker. Returning `Inapplicable` is allowed even when
    /// `applies_to` returned true (e.g. the candidate field is empty
    /// in this specific claim).
    fn check<'a>(
        &'a self,
        claim: &'a Claim,
    ) -> Pin<Box<dyn std::future::Future<Output = CheckerResult> + Send + 'a>>;
}

/// Process-bound registry mapping [`CheckerId`] -> boxed [`Checker`].
/// Constructed at orchestrator startup; populated via
/// [`CheckerRegistry::register`]. Read-only after population in the
/// common path — but additional registrations are allowed during
/// tests.
pub struct CheckerRegistry {
    entries: HashMap<CheckerId, Box<dyn Checker>>,
}

impl Default for CheckerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for CheckerRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CheckerRegistry")
            .field("entries_len", &self.entries.len())
            .field(
                "ids",
                &self
                    .entries
                    .iter()
                    .map(|(k, v)| (k.as_str(), v.kind()))
                    .collect::<Vec<_>>(),
            )
            .finish()
    }
}

impl CheckerRegistry {
    /// Construct an empty registry. Callers must register checkers
    /// before [`Self::run_all_applicable`] returns anything useful.
    #[must_use]
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    /// Register `checker` under `id`. Returns the previously-registered
    /// boxed checker if `id` was already present (caller can inspect
    /// the displaced entry — useful in tests that swap mocks).
    pub fn register(
        &mut self,
        id: impl Into<CheckerId>,
        checker: Box<dyn Checker>,
    ) -> Option<Box<dyn Checker>> {
        self.entries.insert(id.into(), checker)
    }

    /// Look up a registered checker by id. Borrowed reference so the
    /// caller can call `check` without acquiring ownership.
    #[must_use]
    pub fn lookup(&self, id: &str) -> Option<&dyn Checker> {
        self.entries.get(id).map(|b| b.as_ref())
    }

    /// Number of registered checkers.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// True iff no checkers are registered.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// All registered ids in deterministic order (sorted). Used by the
    /// dashboard's `get_checker_registry_state` command + tests that
    /// want a stable iteration order.
    #[must_use]
    pub fn ids_sorted(&self) -> Vec<CheckerId> {
        let mut out: Vec<CheckerId> = self.entries.keys().cloned().collect();
        out.sort();
        out
    }

    /// Iterate `(id, kind)` pairs in sorted-id order.
    #[must_use]
    pub fn entries_sorted(&self) -> Vec<(CheckerId, CheckerKind)> {
        let mut out: Vec<(CheckerId, CheckerKind)> = self
            .entries
            .iter()
            .map(|(k, v)| (k.clone(), v.kind()))
            .collect();
        out.sort_by(|a, b| a.0.cmp(&b.0));
        out
    }

    /// Run every checker for which `applies_to(claim)` returns true,
    /// in sorted-id order so the output is deterministic. Returns one
    /// `(id, kind, result)` triple per call. Empty `Vec` when no
    /// checker is applicable — the caller can treat that as a routing
    /// gap (typically falls back to a non-constructive mode).
    pub async fn run_all_applicable(
        &self,
        claim: &Claim,
    ) -> Vec<(CheckerId, CheckerKind, CheckerResult)> {
        let mut out = Vec::new();
        let ids = self.ids_sorted();
        for id in ids {
            // Safe: id came from self.entries via ids_sorted.
            let Some(entry) = self.entries.get(&id) else {
                continue;
            };
            if !entry.applies_to(claim) {
                continue;
            }
            let result = entry.check(claim).await;
            out.push((id, entry.kind(), result));
        }
        out
    }
}

// ---------------------------------------------------------------------------
// Built-in checker implementations.
//
// Each builtin is a minimal in-tree impl. Subprocess-based checkers
// (sympy, compile_rust) shell out to scripts under .elai_cc/checkers/
// via std::process::Command, isolated from the orchestrator's process
// state. Pure-Rust checkers (regex_match, numeric_range) run inline.
// Deferred checkers (z3, json_schema_validate) are registered as
// stubs that always return Inapplicable — the slot exists so the
// dashboard can render the full closed-enum table, but the actual
// verification is gated on a future plan that ships the helper.
// ---------------------------------------------------------------------------

/// Maximum subprocess wall-clock budget for shell-out checkers. The
/// checker MUST terminate within this — if it doesn't, the runner
/// returns `Error { source: "timeout" }`.
const SHELLOUT_TIMEOUT: Duration = Duration::from_secs(30);

/// SymPy checker — shells out to `.elai_cc/checkers/math/sympy.py`.
/// The helper script reads `{ "claim": "<expr>", "candidate": null }`
/// on stdin and returns `{ "result": "pass" | "fail", "reason":
/// "..." }` on stdout.
///
/// Applies to claims of type `Computational` or `Factual` whose `text`
/// starts with a numeric-equality form (`x ... = ...`). The applies-to
/// predicate is intentionally conservative — false-negatives are
/// preferable to spurious dispatches.
pub struct SympyChecker {
    /// Absolute path to the python interpreter (defaults to "python").
    pub python: PathBuf,
    /// Absolute path to the sympy.py helper script.
    pub script_path: PathBuf,
}

impl SympyChecker {
    /// Construct with explicit paths. Use [`Self::default_at_root`]
    /// when the orchestrator knows the workspace root.
    #[must_use]
    pub fn new(python: impl Into<PathBuf>, script_path: impl Into<PathBuf>) -> Self {
        Self {
            python: python.into(),
            script_path: script_path.into(),
        }
    }

    /// Construct using the conventional location under the workspace
    /// root: `<root>/.elai_cc/checkers/math/sympy.py`.
    #[must_use]
    pub fn default_at_root(root: &Path) -> Self {
        Self {
            python: PathBuf::from("python"),
            script_path: root.join(".elai_cc/checkers/math/sympy.py"),
        }
    }
}

impl Checker for SympyChecker {
    fn kind(&self) -> CheckerKind {
        CheckerKind::Sympy
    }

    fn applies_to(&self, claim: &Claim) -> bool {
        matches!(
            claim.claim_type,
            ClaimType::Computational | ClaimType::Factual
        ) && claim.text.contains('=')
    }

    fn check<'a>(
        &'a self,
        claim: &'a Claim,
    ) -> Pin<Box<dyn std::future::Future<Output = CheckerResult> + Send + 'a>> {
        Box::pin(async move {
            if !self.script_path.exists() {
                return CheckerResult::Inapplicable {
                    reason: format!(
                        "sympy helper script missing at {}",
                        self.script_path.display()
                    ),
                };
            }
            let stdin_json = serde_json::json!({
                "claim": claim.text,
                "candidate": serde_json::Value::Null,
            })
            .to_string();
            run_shellout_check(&self.python, &self.script_path, &stdin_json).await
        })
    }
}

/// `cargo check` checker — shells out to
/// `.elai_cc/checkers/code/compile_rust.sh`. The script reads
/// `{ "candidate": "<rust src>" }` on stdin and writes
/// `{ "result": "pass" | "fail", "reason": "..." }` to stdout.
pub struct CompileRustChecker {
    /// Absolute path to the helper shell script.
    pub script_path: PathBuf,
}

impl CompileRustChecker {
    #[must_use]
    pub fn new(script_path: impl Into<PathBuf>) -> Self {
        Self {
            script_path: script_path.into(),
        }
    }

    #[must_use]
    pub fn default_at_root(root: &Path) -> Self {
        Self {
            script_path: root.join(".elai_cc/checkers/code/compile_rust.sh"),
        }
    }
}

impl Checker for CompileRustChecker {
    fn kind(&self) -> CheckerKind {
        CheckerKind::CompileRust
    }

    fn applies_to(&self, claim: &Claim) -> bool {
        // Only computational claims that mention "rust" or look like
        // a code snippet. Heuristic — the registry caller can override
        // by registering a custom narrower checker under a different id.
        matches!(
            claim.claim_type,
            ClaimType::Computational | ClaimType::Procedural
        ) && (claim.text.contains("fn ")
            || claim.text.contains("let ")
            || claim.text.to_ascii_lowercase().contains("rust"))
    }

    fn check<'a>(
        &'a self,
        claim: &'a Claim,
    ) -> Pin<Box<dyn std::future::Future<Output = CheckerResult> + Send + 'a>> {
        Box::pin(async move {
            if !self.script_path.exists() {
                return CheckerResult::Inapplicable {
                    reason: format!(
                        "compile_rust helper script missing at {}",
                        self.script_path.display()
                    ),
                };
            }
            let stdin_json = serde_json::json!({
                "candidate": claim.text,
            })
            .to_string();
            // Shell scripts run via `sh` on POSIX; on Windows we still
            // attempt `sh` (Git-Bash / WSL provides it) and surface the
            // error if absent. Production deployment is Linux.
            run_shellout_check(
                &PathBuf::from(if cfg!(windows) { "sh" } else { "sh" }),
                &self.script_path,
                &stdin_json,
            )
            .await
        })
    }
}

/// Pure-Rust regex checker. The candidate `pattern` is stored at
/// register time; `applies_to` returns true only for claims whose
/// `text` is the string under test. Uses the workspace's vendored
/// `regex` crate (already a dep — verified by grepping Cargo.toml).
pub struct RegexMatchChecker {
    pattern: regex::Regex,
    /// Human-readable label for SSB / dashboard rendering.
    pub label: String,
    /// True = must match anywhere; false = must match fully (anchored).
    pub contains: bool,
}

impl RegexMatchChecker {
    /// Construct, failing if `pattern` does not compile.
    pub fn new(
        pattern: &str,
        label: impl Into<String>,
        contains: bool,
    ) -> Result<Self, regex::Error> {
        let p = regex::Regex::new(pattern)?;
        Ok(Self {
            pattern: p,
            label: label.into(),
            contains,
        })
    }
}

impl Checker for RegexMatchChecker {
    fn kind(&self) -> CheckerKind {
        CheckerKind::RegexMatch
    }

    fn applies_to(&self, _claim: &Claim) -> bool {
        // Regex is universally applicable — the registry caller scopes
        // it via id naming + which claim_types the producer feeds.
        true
    }

    fn check<'a>(
        &'a self,
        claim: &'a Claim,
    ) -> Pin<Box<dyn std::future::Future<Output = CheckerResult> + Send + 'a>> {
        Box::pin(async move {
            let hit = if self.contains {
                self.pattern.is_match(&claim.text)
            } else {
                self.pattern
                    .find(&claim.text)
                    .map(|m| m.start() == 0 && m.end() == claim.text.len())
                    .unwrap_or(false)
            };
            if hit {
                CheckerResult::Pass
            } else {
                CheckerResult::Fail {
                    reason: format!("regex {} did not match claim.text", self.label),
                }
            }
        })
    }
}

/// Inclusive numeric range check on a parsed `f64`. The candidate is
/// the claim's `text` (parsed at `check` time). Out-of-range yields
/// Fail; non-parseable yields Inapplicable. Pure in-process — no
/// subprocess.
pub struct NumericRangeChecker {
    pub low: f64,
    pub high: f64,
}

impl NumericRangeChecker {
    #[must_use]
    pub fn new(low: f64, high: f64) -> Self {
        Self { low, high }
    }
}

impl Checker for NumericRangeChecker {
    fn kind(&self) -> CheckerKind {
        CheckerKind::NumericRange
    }

    fn applies_to(&self, claim: &Claim) -> bool {
        matches!(
            claim.claim_type,
            ClaimType::Computational | ClaimType::Factual
        )
    }

    fn check<'a>(
        &'a self,
        claim: &'a Claim,
    ) -> Pin<Box<dyn std::future::Future<Output = CheckerResult> + Send + 'a>> {
        Box::pin(async move {
            let Ok(value) = claim.text.trim().parse::<f64>() else {
                return CheckerResult::Inapplicable {
                    reason: "claim.text does not parse as f64".into(),
                };
            };
            if value >= self.low && value <= self.high {
                CheckerResult::Pass
            } else {
                CheckerResult::Fail {
                    reason: format!(
                        "{} not in inclusive range [{}, {}]",
                        value, self.low, self.high
                    ),
                }
            }
        })
    }
}

/// Z3 checker — deferred. Slot reserved; runtime returns Inapplicable
/// until the helper script lands. The slot exists so the closed
/// CheckerKind enum + registry table is complete from D-11 day one
/// (avoids a later schema bump just to expose the kind).
pub struct Z3DeferredChecker;

impl Checker for Z3DeferredChecker {
    fn kind(&self) -> CheckerKind {
        CheckerKind::Z3
    }
    fn applies_to(&self, _claim: &Claim) -> bool {
        false
    }
    fn check<'a>(
        &'a self,
        _claim: &'a Claim,
    ) -> Pin<Box<dyn std::future::Future<Output = CheckerResult> + Send + 'a>> {
        Box::pin(async move {
            CheckerResult::Inapplicable {
                reason: "z3 helper not yet deployed".into(),
            }
        })
    }
}

/// JSON-schema-validate checker — deferred (no jsonschema dep
/// approved yet). Same rationale as [`Z3DeferredChecker`].
pub struct JsonSchemaDeferredChecker;

impl Checker for JsonSchemaDeferredChecker {
    fn kind(&self) -> CheckerKind {
        CheckerKind::JsonSchemaValidate
    }
    fn applies_to(&self, _claim: &Claim) -> bool {
        false
    }
    fn check<'a>(
        &'a self,
        _claim: &'a Claim,
    ) -> Pin<Box<dyn std::future::Future<Output = CheckerResult> + Send + 'a>> {
        Box::pin(async move {
            CheckerResult::Inapplicable {
                reason: "json-schema validator not vendored".into(),
            }
        })
    }
}

// ---------------------------------------------------------------------------
// Subprocess runner shared by SympyChecker + CompileRustChecker.
// ---------------------------------------------------------------------------

/// Run a single shellout checker. Spawns `bin <script>`, writes
/// `stdin_json` to stdin, waits up to [`SHELLOUT_TIMEOUT`], parses the
/// stdout as `{ "result": "pass"|"fail"|"inapplicable", "reason":
/// "..." }`, and maps to a [`CheckerResult`].
///
/// Failure modes:
///   * spawn error            → CheckerResult::Error
///   * timeout                → CheckerResult::Error { source: "timeout" }
///   * non-zero exit + stderr → CheckerResult::Error
///   * stdout parse error     → CheckerResult::Error
async fn run_shellout_check(
    bin: &Path,
    script: &Path,
    stdin_json: &str,
) -> CheckerResult {
    use tokio::io::AsyncWriteExt;
    use tokio::process::Command;

    let start = Instant::now();

    let spawn_result = Command::new(bin)
        .arg(script)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn();
    let mut child = match spawn_result {
        Ok(c) => c,
        Err(e) => {
            return CheckerResult::Error {
                source: format!("spawn {}: {}", bin.display(), e),
            };
        }
    };

    if let Some(mut stdin) = child.stdin.take() {
        if let Err(e) = stdin.write_all(stdin_json.as_bytes()).await {
            return CheckerResult::Error {
                source: format!("stdin write: {}", e),
            };
        }
        // Closing stdin signals EOF to the helper.
        drop(stdin);
    }

    let wait = tokio::time::timeout(SHELLOUT_TIMEOUT, child.wait_with_output()).await;
    let output = match wait {
        Ok(Ok(o)) => o,
        Ok(Err(e)) => {
            return CheckerResult::Error {
                source: format!("wait: {}", e),
            };
        }
        Err(_) => {
            return CheckerResult::Error {
                source: format!("timeout after {:?}", start.elapsed()),
            };
        }
    };

    if !output.status.success() {
        let stderr_snippet = String::from_utf8_lossy(&output.stderr);
        let trimmed: String = stderr_snippet.chars().take(512).collect();
        return CheckerResult::Error {
            source: format!("exit {:?}: {}", output.status.code(), trimmed),
        };
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_shellout_stdout(&stdout)
}

/// Parse the helper script's stdout JSON into a [`CheckerResult`].
/// Public-in-crate for testability — the shape is part of the helper
/// contract.
pub(crate) fn parse_shellout_stdout(s: &str) -> CheckerResult {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return CheckerResult::Error {
            source: "helper stdout empty".into(),
        };
    }
    let v: serde_json::Value = match serde_json::from_str(trimmed) {
        Ok(v) => v,
        Err(e) => {
            return CheckerResult::Error {
                source: format!("stdout parse: {}", e),
            };
        }
    };
    let kind = v
        .get("result")
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    let reason = v
        .get("reason")
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .to_string();
    match kind.as_str() {
        "pass" => CheckerResult::Pass,
        "fail" => CheckerResult::Fail {
            reason: if reason.is_empty() {
                "checker reported fail with no reason".into()
            } else {
                reason
            },
        },
        "inapplicable" => CheckerResult::Inapplicable {
            reason: if reason.is_empty() {
                "checker reported inapplicable with no reason".into()
            } else {
                reason
            },
        },
        other => CheckerResult::Error {
            source: format!("unknown result kind: {}", other),
        },
    }
}

// ---------------------------------------------------------------------------
// Tests — 3 mock checkers (Pass / Fail / Inapplicable) + builtin smoke.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::claim::ClaimType;

    fn mk_claim(text: &str, ty: ClaimType) -> Claim {
        Claim {
            claim_id: format!("c-{}", text.len()),
            text: text.into(),
            claim_type: ty,
            evidence: vec![],
            ts_ms: 1_700_000_000_000,
        }
    }

    /// Mock checker that always returns the configured result.
    struct MockChecker {
        kind: CheckerKind,
        result: CheckerResult,
        applies: bool,
    }

    impl Checker for MockChecker {
        fn kind(&self) -> CheckerKind {
            self.kind
        }
        fn applies_to(&self, _claim: &Claim) -> bool {
            self.applies
        }
        fn check<'a>(
            &'a self,
            _claim: &'a Claim,
        ) -> Pin<Box<dyn std::future::Future<Output = CheckerResult> + Send + 'a>> {
            let r = self.result.clone();
            Box::pin(async move { r })
        }
    }

    #[tokio::test]
    async fn run_all_applicable_returns_one_row_per_applicable_mock() {
        let mut reg = CheckerRegistry::new();
        reg.register(
            "mock-pass",
            Box::new(MockChecker {
                kind: CheckerKind::RegexMatch,
                result: CheckerResult::Pass,
                applies: true,
            }),
        );
        reg.register(
            "mock-fail",
            Box::new(MockChecker {
                kind: CheckerKind::NumericRange,
                result: CheckerResult::Fail {
                    reason: "boom".into(),
                },
                applies: true,
            }),
        );
        reg.register(
            "mock-inapplicable",
            Box::new(MockChecker {
                kind: CheckerKind::Z3,
                result: CheckerResult::Inapplicable {
                    reason: "skip".into(),
                },
                // Inapplicable mock is filtered out by applies_to ->
                // returns false; the registry must NOT include it.
                applies: false,
            }),
        );

        let claim = mk_claim("x = 1", ClaimType::Computational);
        let out = reg.run_all_applicable(&claim).await;

        // 2 applicable checkers -> 2 rows. The third was filtered.
        assert_eq!(out.len(), 2, "got {:?}", out);
        // Sorted by id -> mock-fail comes before mock-pass.
        assert_eq!(out[0].0, "mock-fail");
        assert_eq!(out[0].2.result_kind(), "fail");
        assert_eq!(out[1].0, "mock-pass");
        assert!(out[1].2.is_pass());
    }

    #[tokio::test]
    async fn run_all_applicable_returns_inapplicable_when_checker_reports_so() {
        let mut reg = CheckerRegistry::new();
        reg.register(
            "mock-inapplicable-reported",
            Box::new(MockChecker {
                kind: CheckerKind::JsonSchemaValidate,
                result: CheckerResult::Inapplicable {
                    reason: "no schema".into(),
                },
                applies: true,
            }),
        );
        let claim = mk_claim("anything", ClaimType::Factual);
        let out = reg.run_all_applicable(&claim).await;
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].2.result_kind(), "inapplicable");
    }

    #[test]
    fn checker_kind_as_str_matches_serde() {
        for k in CheckerKind::all() {
            let s = serde_json::to_string(&k).expect("serialize");
            let stripped = s.trim_matches('"');
            assert_eq!(stripped, k.as_str(), "as_str/serde mismatch for {k:?}");
        }
    }

    #[test]
    fn checker_result_round_trips_through_json() {
        for r in [
            CheckerResult::Pass,
            CheckerResult::Fail { reason: "x".into() },
            CheckerResult::Inapplicable {
                reason: "y".into(),
            },
            CheckerResult::Error { source: "z".into() },
        ] {
            let s = serde_json::to_string(&r).expect("ser");
            let back: CheckerResult = serde_json::from_str(&s).expect("de");
            assert_eq!(back, r);
        }
    }

    #[tokio::test]
    async fn regex_match_checker_pass_and_fail() {
        let ck =
            RegexMatchChecker::new(r"^hello\b", "hello-prefix", true).expect("regex compiles");
        let mut reg = CheckerRegistry::new();
        reg.register("regex-hello", Box::new(ck));

        let pass_claim = mk_claim("hello world", ClaimType::Factual);
        let pass = reg.run_all_applicable(&pass_claim).await;
        assert_eq!(pass.len(), 1);
        assert!(pass[0].2.is_pass());

        let fail_claim = mk_claim("goodbye world", ClaimType::Factual);
        let fail = reg.run_all_applicable(&fail_claim).await;
        assert_eq!(fail.len(), 1);
        assert_eq!(fail[0].2.result_kind(), "fail");
    }

    #[tokio::test]
    async fn numeric_range_checker_pass_fail_inapplicable() {
        let mut reg = CheckerRegistry::new();
        reg.register(
            "nr-0-10",
            Box::new(NumericRangeChecker::new(0.0, 10.0)),
        );
        let pass = reg
            .run_all_applicable(&mk_claim("5", ClaimType::Computational))
            .await;
        assert!(pass[0].2.is_pass());

        let fail = reg
            .run_all_applicable(&mk_claim("42", ClaimType::Computational))
            .await;
        assert_eq!(fail[0].2.result_kind(), "fail");

        let inap = reg
            .run_all_applicable(&mk_claim("not-a-number", ClaimType::Computational))
            .await;
        assert_eq!(inap[0].2.result_kind(), "inapplicable");
    }

    #[test]
    fn parse_shellout_stdout_handles_all_kinds() {
        assert!(parse_shellout_stdout(r#"{"result":"pass"}"#).is_pass());
        assert_eq!(
            parse_shellout_stdout(r#"{"result":"fail","reason":"bad"}"#).result_kind(),
            "fail"
        );
        assert_eq!(
            parse_shellout_stdout(r#"{"result":"inapplicable","reason":"nope"}"#).result_kind(),
            "inapplicable"
        );
        assert_eq!(parse_shellout_stdout("").result_kind(), "error");
        assert_eq!(parse_shellout_stdout("not-json").result_kind(), "error");
        assert_eq!(
            parse_shellout_stdout(r#"{"result":"weird"}"#).result_kind(),
            "error"
        );
    }

    #[test]
    fn registry_entries_sorted_is_deterministic() {
        let mut reg = CheckerRegistry::new();
        reg.register(
            "zeta",
            Box::new(NumericRangeChecker::new(0.0, 1.0)),
        );
        reg.register(
            "alpha",
            Box::new(NumericRangeChecker::new(0.0, 1.0)),
        );
        reg.register(
            "mu",
            Box::new(NumericRangeChecker::new(0.0, 1.0)),
        );
        let ids: Vec<String> = reg.entries_sorted().into_iter().map(|(k, _)| k).collect();
        assert_eq!(ids, vec!["alpha".to_string(), "mu".into(), "zeta".into()]);
    }

    #[tokio::test]
    async fn missing_sympy_script_returns_inapplicable() {
        let ck = SympyChecker::new("python", PathBuf::from("/nonexistent/sympy.py"));
        let mut reg = CheckerRegistry::new();
        reg.register("sympy-missing", Box::new(ck));
        let claim = mk_claim("x = 1", ClaimType::Computational);
        let out = reg.run_all_applicable(&claim).await;
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].2.result_kind(), "inapplicable");
    }
}
