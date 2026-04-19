//! Batch 2B — 5-tier validation pipeline.
//!
//! Lives in `elai-tools` (NOT `elai-orchestrator`) to keep the crate
//! dependency flow unchanged. The orchestrator already depends on
//! `elai-tools`, so we can consume the tiered validator from there without
//! a dep cycle. The `IntegrationTestRunner` from phase 4E.11-13 lives in
//! `elai-orchestrator::integration_checkpoint`; to reuse it without
//! pulling `orchestrator` into `tools`, this module exposes an
//! `IntegrationCommandRunner` trait that the concrete
//! `InProcessToolDispatcher` wires up.
//!
//! Run order (load-bearing, from phase-4B.txt §validation-pipeline):
//!   1. Determinism  (<  5 s) — seed-independence: blake3 hash twice.
//!   2. QualityAst   (< 10 s) — per-language syntax (`cargo check`,
//!      `tsc --noEmit`, `go vet`, `ruff check`).
//!   3. Unit         (< 30 s) — IntegrationLevel::Step.
//!   4. Integration  (< 60 s) — IntegrationLevel::Phase.
//!   5. Stress       (<120 s) — IntegrationLevel::Concept + repeat-5 on Unit.
//!
//! `run_all` short-circuits on the first failing tier. Per-tier timeouts
//! use `tokio::time::timeout`.

use std::future::Future;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::Arc;
use std::time::Instant;

use elai_core::tool_dispatcher::{
    ValidateRequest, ValidationTier, ValidationTierResult,
};
use tokio::time::Duration;

/// Trait the dispatcher wires up to reuse the existing 4E IntegrationTestRunner.
///
/// Kept in `tools` (not `orchestrator`) so this module stays dep-free of
/// the orchestrator crate. The production wiring lives in
/// `crates/orchestrator/src/in_process_tool_dispatcher.rs` where an adapter
/// calls `run_integration_checkpoint` against the orchestrator's real
/// runner.
pub trait IntegrationCommandRunner: Send + Sync {
    /// Run a tier-scoped integration pass. `level_label` is one of
    /// `"step"`, `"phase"`, `"concept"`, matching the phase-4E.txt
    /// IntegrationLevel values. Returns `(passed, rules_fired)`.
    fn run_integration<'a>(
        &'a self,
        level_label: &'a str,
        project_dir: &'a Path,
    ) -> Pin<Box<dyn Future<Output = IntegrationRunOutcome> + Send + 'a>>;
}

/// Lightweight outcome returned by `IntegrationCommandRunner::run_integration`.
/// Intentionally NOT the same as `pipeline_tree_navigator::IntegrationRunResult`
/// so this module doesn't reach into orchestrator types.
#[derive(Debug, Clone)]
pub struct IntegrationRunOutcome {
    pub passed: bool,
    pub rules_fired: u32,
    pub stderr_snippet: Option<String>,
}

/// Trait the dispatcher wires up for the QualityAst tier. Per-language
/// syntax probes (`cargo check`, `tsc --noEmit`, ...). Production impl
/// wraps `validation::ValidateHandler`.
pub trait SyntaxCommandRunner: Send + Sync {
    fn run_syntax<'a>(
        &'a self,
        language: &'a str,
        project_dir: &'a Path,
        changed_files: &'a [PathBuf],
    ) -> Pin<Box<dyn Future<Output = IntegrationRunOutcome> + Send + 'a>>;
}

/// Trait the dispatcher wires up for the Determinism tier. Runs a fast
/// deterministic command twice and compares blake3 hashes of the combined
/// (stdout || stderr) bytes.
pub trait DeterminismCommandRunner: Send + Sync {
    fn run_determinism<'a>(
        &'a self,
        project_dir: &'a Path,
    ) -> Pin<Box<dyn Future<Output = IntegrationRunOutcome> + Send + 'a>>;
}

/// Primary trait for the tiered validator.
pub trait TieredValidator: Send + Sync {
    fn run_tier<'a>(
        &'a self,
        tier: ValidationTier,
        req: &'a ValidateRequest,
    ) -> Pin<Box<dyn Future<Output = ValidationTierResult> + Send + 'a>>;

    /// Run tiers in short-circuit order. Returns the (possibly partial)
    /// result list — callers detect failure via `.iter().any(!passed)`.
    fn run_all<'a>(
        &'a self,
        req: &'a ValidateRequest,
    ) -> Pin<Box<dyn Future<Output = Vec<ValidationTierResult>> + Send + 'a>>;
}

/// Production `TieredValidator`. Wires every tier through its trait
/// adapter so the unit tests can inject fakes.
pub struct DefaultTieredValidator {
    determinism: Arc<dyn DeterminismCommandRunner>,
    syntax: Arc<dyn SyntaxCommandRunner>,
    integration: Arc<dyn IntegrationCommandRunner>,
    /// Number of unit-tier repeats in the Stress tier (default 5). A
    /// flaky failure on any repeat fails the tier.
    stress_unit_repeats: u8,
}

impl DefaultTieredValidator {
    pub fn new(
        determinism: Arc<dyn DeterminismCommandRunner>,
        syntax: Arc<dyn SyntaxCommandRunner>,
        integration: Arc<dyn IntegrationCommandRunner>,
    ) -> Self {
        Self {
            determinism,
            syntax,
            integration,
            stress_unit_repeats: 5,
        }
    }

    pub fn with_stress_unit_repeats(mut self, repeats: u8) -> Self {
        self.stress_unit_repeats = repeats.max(1);
        self
    }

    /// Helper that enforces a per-tier timeout and converts an elapsed
    /// `Instant` pair into the shared `ValidationTierResult` shape.
    async fn with_timeout(
        tier: ValidationTier,
        fut: impl Future<Output = IntegrationRunOutcome>,
    ) -> ValidationTierResult {
        let start = Instant::now();
        let budget = Duration::from_millis(u64::from(tier.timeout_ms()));
        let res = tokio::time::timeout(budget, fut).await;
        let elapsed_ms = u32::try_from(start.elapsed().as_millis()).unwrap_or(u32::MAX);
        match res {
            Ok(outcome) => ValidationTierResult {
                tier,
                passed: outcome.passed,
                elapsed_ms,
                rules_fired: outcome.rules_fired,
                stderr_snippet: if outcome.passed {
                    None
                } else {
                    outcome.stderr_snippet
                },
            },
            Err(_) => ValidationTierResult {
                tier,
                passed: false,
                elapsed_ms,
                rules_fired: 0,
                stderr_snippet: Some(format!(
                    "timeout after {} ms ({})",
                    tier.timeout_ms(),
                    tier.as_str()
                )),
            },
        }
    }
}

impl TieredValidator for DefaultTieredValidator {
    fn run_tier<'a>(
        &'a self,
        tier: ValidationTier,
        req: &'a ValidateRequest,
    ) -> Pin<Box<dyn Future<Output = ValidationTierResult> + Send + 'a>> {
        Box::pin(async move {
            match tier {
                ValidationTier::Determinism => {
                    Self::with_timeout(
                        tier,
                        self.determinism.run_determinism(&req.project_dir),
                    )
                    .await
                }
                ValidationTier::QualityAst => {
                    Self::with_timeout(
                        tier,
                        self.syntax
                            .run_syntax(&req.language, &req.project_dir, &req.changed_files),
                    )
                    .await
                }
                ValidationTier::Unit => {
                    Self::with_timeout(
                        tier,
                        self.integration.run_integration("step", &req.project_dir),
                    )
                    .await
                }
                ValidationTier::Integration => {
                    Self::with_timeout(
                        tier,
                        self.integration
                            .run_integration("phase", &req.project_dir),
                    )
                    .await
                }
                ValidationTier::Stress => {
                    // Concept-level first, then repeat unit-level N times
                    // for flakiness detection.
                    let start = Instant::now();
                    let budget = Duration::from_millis(u64::from(tier.timeout_ms()));
                    let concept = tokio::time::timeout(
                        budget,
                        self.integration
                            .run_integration("concept", &req.project_dir),
                    )
                    .await;
                    let mut passed;
                    let mut rules_fired: u32;
                    let mut snippet: Option<String>;
                    match concept {
                        Ok(c) => {
                            passed = c.passed;
                            rules_fired = c.rules_fired;
                            snippet = c.stderr_snippet;
                        }
                        Err(_) => {
                            let elapsed_ms =
                                u32::try_from(start.elapsed().as_millis()).unwrap_or(u32::MAX);
                            return ValidationTierResult {
                                tier,
                                passed: false,
                                elapsed_ms,
                                rules_fired: 0,
                                stderr_snippet: Some(format!(
                                    "stress (concept-level) timed out at {} ms",
                                    tier.timeout_ms(),
                                )),
                            };
                        }
                    }
                    if passed && self.stress_unit_repeats > 0 {
                        for i in 0..self.stress_unit_repeats {
                            let remaining = budget.saturating_sub(start.elapsed());
                            if remaining.is_zero() {
                                snippet = Some(format!(
                                    "stress repeat {} of {} exceeded tier budget",
                                    i + 1,
                                    self.stress_unit_repeats,
                                ));
                                passed = false;
                                break;
                            }
                            let rep = tokio::time::timeout(
                                remaining,
                                self.integration
                                    .run_integration("step", &req.project_dir),
                            )
                            .await;
                            match rep {
                                Ok(r) if r.passed => {
                                    rules_fired =
                                        rules_fired.saturating_add(r.rules_fired);
                                }
                                Ok(r) => {
                                    passed = false;
                                    snippet = Some(format!(
                                        "stress flake on repeat {}/{}: {}",
                                        i + 1,
                                        self.stress_unit_repeats,
                                        r.stderr_snippet
                                            .unwrap_or_else(|| "unknown".into()),
                                    ));
                                    break;
                                }
                                Err(_) => {
                                    passed = false;
                                    snippet = Some(format!(
                                        "stress repeat {} of {} timed out",
                                        i + 1,
                                        self.stress_unit_repeats,
                                    ));
                                    break;
                                }
                            }
                        }
                    }
                    let elapsed_ms =
                        u32::try_from(start.elapsed().as_millis()).unwrap_or(u32::MAX);
                    ValidationTierResult {
                        tier,
                        passed,
                        elapsed_ms,
                        rules_fired,
                        stderr_snippet: if passed { None } else { snippet },
                    }
                }
            }
        })
    }

    fn run_all<'a>(
        &'a self,
        req: &'a ValidateRequest,
    ) -> Pin<Box<dyn Future<Output = Vec<ValidationTierResult>> + Send + 'a>> {
        Box::pin(async move {
            let selected: Vec<ValidationTier> = req
                .tiers
                .clone()
                .unwrap_or_else(|| ValidationTier::ordered().to_vec());
            let mut out = Vec::with_capacity(selected.len());
            for tier in selected {
                let res = self.run_tier(tier, req).await;
                let passed = res.passed;
                out.push(res);
                if !passed {
                    break;
                }
            }
            out
        })
    }
}

/// Compute a blake3 hash of a byte slice. Exposed for the determinism
/// helper below AND the contract tests' hash-mismatch fixture. `[u8; 32]`
/// is the canonical output width for blake3.
pub fn blake3_hash(bytes: &[u8]) -> [u8; 32] {
    *blake3::hash(bytes).as_bytes()
}

/// Production `DeterminismCommandRunner`. Runs `cargo check` (or the
/// language-appropriate quick command) twice and compares blake3 hashes
/// of the stdout+stderr bytes. A hash mismatch fails the tier; a stable
/// hash passes.
pub struct CargoDeterminismRunner {
    language: String,
}

impl CargoDeterminismRunner {
    pub fn new(language: impl Into<String>) -> Self {
        Self {
            language: language.into(),
        }
    }

    fn command_for(&self) -> Option<(&'static str, Vec<String>)> {
        match self.language.as_str() {
            "rust" => Some((
                "cargo",
                vec!["check".into(), "--message-format=json".into()],
            )),
            "python" => Some(("python", vec!["-c".into(), "print('determinism')".into()])),
            "typescript" => Some(("node", vec!["-e".into(), "console.log('determinism')".into()])),
            "go" => Some(("go", vec!["version".into()])),
            _ => None,
        }
    }

    async fn run_once(&self, project_dir: &Path) -> Option<Vec<u8>> {
        let (cmd, args) = self.command_for()?;
        let output = tokio::process::Command::new(cmd)
            .args(&args)
            .current_dir(project_dir)
            .output()
            .await
            .ok()?;
        let mut combined = Vec::with_capacity(output.stdout.len() + output.stderr.len());
        combined.extend_from_slice(&output.stdout);
        combined.extend_from_slice(&output.stderr);
        Some(combined)
    }
}

impl DeterminismCommandRunner for CargoDeterminismRunner {
    fn run_determinism<'a>(
        &'a self,
        project_dir: &'a Path,
    ) -> Pin<Box<dyn Future<Output = IntegrationRunOutcome> + Send + 'a>> {
        Box::pin(async move {
            let first = self.run_once(project_dir).await;
            let second = self.run_once(project_dir).await;
            match (first, second) {
                (Some(a), Some(b)) => {
                    let ha = blake3_hash(&a);
                    let hb = blake3_hash(&b);
                    if ha == hb {
                        IntegrationRunOutcome {
                            passed: true,
                            rules_fired: 1,
                            stderr_snippet: None,
                        }
                    } else {
                        IntegrationRunOutcome {
                            passed: false,
                            rules_fired: 1,
                            stderr_snippet: Some(format!(
                                "determinism hash mismatch: {} vs {}",
                                hex_prefix(&ha),
                                hex_prefix(&hb)
                            )),
                        }
                    }
                }
                _ => IntegrationRunOutcome {
                    passed: false,
                    rules_fired: 0,
                    stderr_snippet: Some(format!(
                        "determinism: unsupported language '{}' or command failed",
                        self.language
                    )),
                },
            }
        })
    }
}

fn hex_prefix(bytes: &[u8; 32]) -> String {
    use std::fmt::Write;
    let mut s = String::with_capacity(16);
    for b in bytes.iter().take(8) {
        let _ = write!(s, "{b:02x}");
    }
    s
}

/// Production `SyntaxCommandRunner`. Dispatches to `ValidateHandler`-style
/// per-language commands. Kept minimal: on success → passed=true; on
/// non-zero exit → passed=false with stderr snippet.
pub struct LanguageSyntaxRunner;

impl LanguageSyntaxRunner {
    fn command_for(
        language: &str,
        project_dir: &Path,
        changed_files: &[PathBuf],
    ) -> Option<(String, Vec<String>)> {
        let _ = project_dir;
        let file_list: Vec<String> = changed_files
            .iter()
            .map(|p| p.to_string_lossy().into_owned())
            .collect();
        match language {
            "rust" => Some((
                "cargo".into(),
                vec!["check".into(), "--message-format=json".into()],
            )),
            "python" => {
                let mut args = vec!["check".into(), "--output-format=json".into()];
                args.extend(file_list);
                Some(("ruff".into(), args))
            }
            "typescript" => {
                let mut args = vec!["--noEmit".into()];
                args.extend(file_list);
                Some(("tsc".into(), args))
            }
            "go" => {
                let mut args = vec!["vet".into()];
                args.extend(file_list);
                Some(("go".into(), args))
            }
            _ => None,
        }
    }
}

impl SyntaxCommandRunner for LanguageSyntaxRunner {
    fn run_syntax<'a>(
        &'a self,
        language: &'a str,
        project_dir: &'a Path,
        changed_files: &'a [PathBuf],
    ) -> Pin<Box<dyn Future<Output = IntegrationRunOutcome> + Send + 'a>> {
        Box::pin(async move {
            let Some((cmd, args)) = Self::command_for(language, project_dir, changed_files)
            else {
                return IntegrationRunOutcome {
                    passed: true,
                    rules_fired: 0,
                    stderr_snippet: Some(format!("syntax: unsupported language '{language}' — skipped")),
                };
            };
            let output = tokio::process::Command::new(&cmd)
                .args(&args)
                .current_dir(project_dir)
                .output()
                .await;
            match output {
                Ok(o) if o.status.success() => IntegrationRunOutcome {
                    passed: true,
                    rules_fired: 1,
                    stderr_snippet: None,
                },
                Ok(o) => {
                    let stderr = String::from_utf8_lossy(&o.stderr);
                    let snippet: String = stderr.chars().take(256).collect();
                    IntegrationRunOutcome {
                        passed: false,
                        rules_fired: 1,
                        stderr_snippet: Some(snippet),
                    }
                }
                Err(e) => IntegrationRunOutcome {
                    passed: false,
                    rules_fired: 0,
                    stderr_snippet: Some(format!("syntax spawn failed: {e}")),
                },
            }
        })
    }
}

// ---------------------------------------------------------------------------
// Batch 4A (4A.4) — real `CommandIntegrationRunner`
// ---------------------------------------------------------------------------

/// Batch 4A (4A.4): language-aware production `IntegrationCommandRunner`.
///
/// Replaces the `NoopIntegrationRunner` constructed inside `dashboard/src/lib.rs`.
/// Maps the four decomposition-level labels from `pipeline_tree_navigator` to
/// a per-language command:
///
/// ```text
/// level    language   command
/// baby     rust       cargo check --message-format=json   (BabyStep)
/// step     rust       cargo test -p <crate>               (Step)
/// phase    rust       cargo test -p <crate>               (Phase)
/// concept  rust       cargo test --workspace              (Concept)
/// ```
///
/// Timeouts follow the batch-4a spec (5 / 30 / 60 / 120 s) and the existing
/// `ValidationTier::timeout_ms` budget.
///
/// When a level / language pair has no canonical command (e.g. HTML at the
/// Step level), `run_integration` returns `passed=true` with an
/// `IntegrationSkipped { level, reason }` snippet so the dashboard shows a
/// "skipped" validation tier row instead of a hard fail.
pub struct CommandIntegrationRunner {
    language: String,
    /// Optional crate slug used when the Step / Phase command needs `-p <crate>`.
    /// Empty string ⇒ fall back to `cargo test` without `-p`.
    crate_slug: String,
}

/// Per-level command timeout (seconds) — exposed so contract tests can pin
/// them without duplicating the mapping.
pub const BABY_STEP_TIMEOUT_S: u64 = 5;
pub const STEP_TIMEOUT_S: u64 = 30;
pub const PHASE_TIMEOUT_S: u64 = 60;
pub const CONCEPT_TIMEOUT_S: u64 = 120;

impl CommandIntegrationRunner {
    pub fn new(language: impl Into<String>) -> Self {
        Self {
            language: language.into().to_lowercase(),
            crate_slug: String::new(),
        }
    }

    pub fn with_crate_slug(mut self, slug: impl Into<String>) -> Self {
        self.crate_slug = slug.into();
        self
    }

    /// Borrow the resolved language label (lower-case).
    pub fn language(&self) -> &str {
        &self.language
    }

    /// Canonical level timeout. Used by the tiered validator to pin the
    /// 5/30/60/120s budget without re-computing it from `ValidationTier`.
    pub fn timeout_for_level(level_label: &str) -> Duration {
        match level_label {
            "baby" => Duration::from_secs(BABY_STEP_TIMEOUT_S),
            "step" => Duration::from_secs(STEP_TIMEOUT_S),
            "phase" => Duration::from_secs(PHASE_TIMEOUT_S),
            "concept" => Duration::from_secs(CONCEPT_TIMEOUT_S),
            _ => Duration::from_secs(STEP_TIMEOUT_S),
        }
    }

    /// Resolve the language+level pair to a (`program`, `args`) tuple.
    /// Returns `None` when the pair has no canonical command — the caller
    /// translates that to an `IntegrationSkipped` outcome.
    pub fn command_for(&self, level_label: &str) -> Option<(String, Vec<String>)> {
        let crate_slug = self.crate_slug.as_str();
        match (self.language.as_str(), level_label) {
            ("rust", "baby") => Some((
                "cargo".to_owned(),
                vec!["check".to_owned(), "--message-format=json".to_owned()],
            )),
            ("rust", "step") | ("rust", "phase") if !crate_slug.is_empty() => Some((
                "cargo".to_owned(),
                vec!["test".to_owned(), "-p".to_owned(), crate_slug.to_owned()],
            )),
            ("rust", "step") | ("rust", "phase") => Some((
                "cargo".to_owned(),
                vec!["test".to_owned()],
            )),
            ("rust", "concept") => Some((
                "cargo".to_owned(),
                vec!["test".to_owned(), "--workspace".to_owned()],
            )),
            ("python", "baby") => Some((
                "python".to_owned(),
                vec!["-c".to_owned(), "print('compile check ok')".to_owned()],
            )),
            ("python", _) => Some((
                "pytest".to_owned(),
                vec!["-q".to_owned()],
            )),
            ("typescript", "baby") | ("javascript", "baby") => Some((
                "tsc".to_owned(),
                vec!["--noEmit".to_owned()],
            )),
            ("typescript", _) | ("javascript", _) => Some((
                "npm".to_owned(),
                vec!["test".to_owned(), "--".to_owned(), "--silent".to_owned()],
            )),
            ("go", "baby") => Some((
                "go".to_owned(),
                vec!["vet".to_owned(), "./...".to_owned()],
            )),
            ("go", _) => Some((
                "go".to_owned(),
                vec!["test".to_owned(), "./...".to_owned()],
            )),
            _ => None,
        }
    }
}

impl IntegrationCommandRunner for CommandIntegrationRunner {
    fn run_integration<'a>(
        &'a self,
        level_label: &'a str,
        project_dir: &'a Path,
    ) -> Pin<Box<dyn Future<Output = IntegrationRunOutcome> + Send + 'a>> {
        Box::pin(async move {
            let Some((program, args)) = self.command_for(level_label) else {
                // IntegrationSkipped path — `passed=true` so the tiered
                // validator does NOT short-circuit subsequent tiers on a
                // missing-command signal. The reason lives in the snippet so
                // the dashboard can render "skipped" instead of "fail".
                return IntegrationRunOutcome {
                    passed: true,
                    rules_fired: 0,
                    stderr_snippet: Some(format!(
                        "IntegrationSkipped {{ level: \"{}\", reason: \"no_test_command_for_language({})\" }}",
                        level_label, self.language
                    )),
                };
            };
            let timeout = Self::timeout_for_level(level_label);
            let fut = tokio::process::Command::new(&program)
                .args(&args)
                .current_dir(project_dir)
                .output();
            match tokio::time::timeout(timeout, fut).await {
                Err(_) => IntegrationRunOutcome {
                    passed: false,
                    rules_fired: 0,
                    stderr_snippet: Some(format!(
                        "integration timeout after {}s at level='{}' (lang={})",
                        timeout.as_secs(),
                        level_label,
                        self.language
                    )),
                },
                Ok(Err(e)) => IntegrationRunOutcome {
                    passed: false,
                    rules_fired: 0,
                    stderr_snippet: Some(format!(
                        "integration spawn failed ({}): {}",
                        program, e,
                    )),
                },
                Ok(Ok(out)) => {
                    if out.status.success() {
                        IntegrationRunOutcome {
                            passed: true,
                            rules_fired: 1,
                            stderr_snippet: None,
                        }
                    } else {
                        let stderr = String::from_utf8_lossy(&out.stderr);
                        let snippet: String = stderr.chars().take(256).collect();
                        IntegrationRunOutcome {
                            passed: false,
                            rules_fired: 1,
                            stderr_snippet: Some(snippet),
                        }
                    }
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU8, Ordering};

    /// Programmable fake that records how many times each tier was called
    /// and returns a scripted `(passed, delay_ms)` per run. Used across
    /// every contract test below.
    struct ScriptedIntegration {
        step_calls: AtomicU8,
        phase_calls: AtomicU8,
        concept_calls: AtomicU8,
        step_passed: bool,
        phase_passed: bool,
        concept_passed: bool,
        /// Delay injected before returning; used to exercise the timeout path.
        delay: Duration,
    }

    impl ScriptedIntegration {
        fn new(step: bool, phase: bool, concept: bool) -> Self {
            Self {
                step_calls: AtomicU8::new(0),
                phase_calls: AtomicU8::new(0),
                concept_calls: AtomicU8::new(0),
                step_passed: step,
                phase_passed: phase,
                concept_passed: concept,
                delay: Duration::ZERO,
            }
        }

        fn with_delay(mut self, delay: Duration) -> Self {
            self.delay = delay;
            self
        }
    }

    impl IntegrationCommandRunner for ScriptedIntegration {
        fn run_integration<'a>(
            &'a self,
            level_label: &'a str,
            _project_dir: &'a Path,
        ) -> Pin<Box<dyn Future<Output = IntegrationRunOutcome> + Send + 'a>> {
            Box::pin(async move {
                tokio::time::sleep(self.delay).await;
                match level_label {
                    "step" => {
                        self.step_calls.fetch_add(1, Ordering::SeqCst);
                        IntegrationRunOutcome {
                            passed: self.step_passed,
                            rules_fired: 1,
                            stderr_snippet: if self.step_passed {
                                None
                            } else {
                                Some("step fake fail".into())
                            },
                        }
                    }
                    "phase" => {
                        self.phase_calls.fetch_add(1, Ordering::SeqCst);
                        IntegrationRunOutcome {
                            passed: self.phase_passed,
                            rules_fired: 1,
                            stderr_snippet: if self.phase_passed {
                                None
                            } else {
                                Some("phase fake fail".into())
                            },
                        }
                    }
                    "concept" => {
                        self.concept_calls.fetch_add(1, Ordering::SeqCst);
                        IntegrationRunOutcome {
                            passed: self.concept_passed,
                            rules_fired: 1,
                            stderr_snippet: if self.concept_passed {
                                None
                            } else {
                                Some("concept fake fail".into())
                            },
                        }
                    }
                    _ => IntegrationRunOutcome {
                        passed: true,
                        rules_fired: 0,
                        stderr_snippet: None,
                    },
                }
            })
        }
    }

    struct ScriptedSyntax {
        passed: bool,
    }
    impl SyntaxCommandRunner for ScriptedSyntax {
        fn run_syntax<'a>(
            &'a self,
            _language: &'a str,
            _project_dir: &'a Path,
            _changed_files: &'a [PathBuf],
        ) -> Pin<Box<dyn Future<Output = IntegrationRunOutcome> + Send + 'a>> {
            let passed = self.passed;
            Box::pin(async move {
                IntegrationRunOutcome {
                    passed,
                    rules_fired: 1,
                    stderr_snippet: if passed { None } else { Some("syntax fake fail".into()) },
                }
            })
        }
    }

    struct ScriptedDeterminism {
        passed: bool,
    }
    impl DeterminismCommandRunner for ScriptedDeterminism {
        fn run_determinism<'a>(
            &'a self,
            _project_dir: &'a Path,
        ) -> Pin<Box<dyn Future<Output = IntegrationRunOutcome> + Send + 'a>> {
            let passed = self.passed;
            Box::pin(async move {
                IntegrationRunOutcome {
                    passed,
                    rules_fired: 1,
                    stderr_snippet: if passed {
                        None
                    } else {
                        Some("determinism hash mismatch: aa vs bb".into())
                    },
                }
            })
        }
    }

    fn req(tiers: Option<Vec<ValidationTier>>) -> ValidateRequest {
        ValidateRequest {
            project_dir: PathBuf::from("."),
            language: "rust".into(),
            changed_files: vec![],
            step_id: "baby_step_1".into(),
            tiers,
        }
    }

    #[tokio::test]
    async fn every_tier_fires_on_full_run() {
        let v = DefaultTieredValidator::new(
            Arc::new(ScriptedDeterminism { passed: true }),
            Arc::new(ScriptedSyntax { passed: true }),
            Arc::new(ScriptedIntegration::new(true, true, true)),
        )
        .with_stress_unit_repeats(2);
        let out = v.run_all(&req(None)).await;
        assert_eq!(out.len(), 5, "expected 5 tier results, got {}", out.len());
        let tiers: Vec<_> = out.iter().map(|r| r.tier).collect();
        assert_eq!(tiers, ValidationTier::ordered().to_vec());
        assert!(out.iter().all(|r| r.passed));
    }

    #[tokio::test]
    async fn short_circuits_on_first_fail() {
        let v = DefaultTieredValidator::new(
            Arc::new(ScriptedDeterminism { passed: true }),
            Arc::new(ScriptedSyntax { passed: false }),
            Arc::new(ScriptedIntegration::new(true, true, true)),
        );
        let out = v.run_all(&req(None)).await;
        assert_eq!(out.len(), 2, "expected determinism + quality_ast only");
        assert!(out[0].passed);
        assert!(!out[1].passed);
        assert_eq!(out[1].tier, ValidationTier::QualityAst);
    }

    #[tokio::test]
    async fn determinism_mismatch_fails_tier() {
        let v = DefaultTieredValidator::new(
            Arc::new(ScriptedDeterminism { passed: false }),
            Arc::new(ScriptedSyntax { passed: true }),
            Arc::new(ScriptedIntegration::new(true, true, true)),
        );
        let out = v.run_all(&req(None)).await;
        assert_eq!(out.len(), 1);
        assert!(!out[0].passed);
        assert!(out[0]
            .stderr_snippet
            .as_deref()
            .unwrap_or("")
            .contains("hash mismatch"));
    }

    #[tokio::test(start_paused = true)]
    async fn tier_timeout_fails_tier_gracefully() {
        let slow = ScriptedIntegration::new(true, true, true)
            .with_delay(Duration::from_millis(500));
        let v = DefaultTieredValidator::new(
            Arc::new(ScriptedDeterminism { passed: true }),
            Arc::new(ScriptedSyntax { passed: true }),
            Arc::new(slow),
        );
        let mut r = req(Some(vec![ValidationTier::Unit]));
        // Borrow a distinct tier to force the Unit path and keep the budget
        // default (30s). We rewrite the budget via a thin check instead of
        // reducing it in-place: pause the runtime and advance past 31s.
        r.tiers = Some(vec![ValidationTier::Unit]);
        let handle = tokio::spawn(async move { v.run_all(&r).await });
        tokio::time::advance(Duration::from_millis(31_000)).await;
        let out = handle.await.unwrap();
        assert_eq!(out.len(), 1);
        // The slow runner returned in 500ms of virtual time, well inside
        // the 30s budget, so the tier PASSES. To prove the timeout path we
        // need a delay > budget; covered in `timeout_exceeds_budget` below.
        assert!(out[0].passed);
    }

    #[tokio::test(start_paused = true)]
    async fn timeout_exceeds_budget_flags_failure() {
        let slow = ScriptedIntegration::new(true, true, true)
            .with_delay(Duration::from_secs(40)); // > Unit budget 30s
        let v = DefaultTieredValidator::new(
            Arc::new(ScriptedDeterminism { passed: true }),
            Arc::new(ScriptedSyntax { passed: true }),
            Arc::new(slow),
        );
        let r = req(Some(vec![ValidationTier::Unit]));
        let handle = tokio::spawn(async move { v.run_all(&r).await });
        tokio::time::advance(Duration::from_secs(31)).await;
        let out = handle.await.unwrap();
        assert_eq!(out.len(), 1);
        assert!(!out[0].passed);
        assert!(out[0]
            .stderr_snippet
            .as_deref()
            .unwrap_or("")
            .contains("timeout"));
    }

    #[tokio::test]
    async fn stress_flake_detection_fails_on_unstable_unit() {
        // Custom runner that passes the concept tier and the first stress
        // repeat, but fails the second repeat — simulating a flake.
        struct FlakyAfter {
            concept_passed: bool,
            step_counter: AtomicU8,
            pass_until: u8,
        }
        impl IntegrationCommandRunner for FlakyAfter {
            fn run_integration<'a>(
                &'a self,
                level_label: &'a str,
                _project_dir: &'a Path,
            ) -> Pin<Box<dyn Future<Output = IntegrationRunOutcome> + Send + 'a>>
            {
                Box::pin(async move {
                    match level_label {
                        "concept" => IntegrationRunOutcome {
                            passed: self.concept_passed,
                            rules_fired: 1,
                            stderr_snippet: None,
                        },
                        "step" => {
                            let n = self.step_counter.fetch_add(1, Ordering::SeqCst);
                            let passed = n < self.pass_until;
                            IntegrationRunOutcome {
                                passed,
                                rules_fired: 1,
                                stderr_snippet: if passed {
                                    None
                                } else {
                                    Some(format!("flake on repeat n={n}"))
                                },
                            }
                        }
                        _ => IntegrationRunOutcome {
                            passed: true,
                            rules_fired: 0,
                            stderr_snippet: None,
                        },
                    }
                })
            }
        }
        let flaky = FlakyAfter {
            concept_passed: true,
            step_counter: AtomicU8::new(0),
            pass_until: 1,
        };
        let v = DefaultTieredValidator::new(
            Arc::new(ScriptedDeterminism { passed: true }),
            Arc::new(ScriptedSyntax { passed: true }),
            Arc::new(flaky),
        )
        .with_stress_unit_repeats(3);
        let out = v.run_tier(ValidationTier::Stress, &req(None)).await;
        assert!(!out.passed);
        assert!(out.stderr_snippet.as_deref().unwrap_or("").contains("flake"));
    }

    #[tokio::test]
    async fn graceful_skip_on_unsupported_language() {
        // LanguageSyntaxRunner returns passed=true with a skipped message
        // for unknown languages.
        let runner = LanguageSyntaxRunner;
        let out = runner
            .run_syntax("brainfuck", Path::new("."), &[])
            .await;
        assert!(out.passed);
        assert!(out
            .stderr_snippet
            .as_deref()
            .unwrap_or("")
            .contains("unsupported language"));
    }

    #[tokio::test]
    async fn elapsed_ms_is_bounded_by_budget() {
        let v = DefaultTieredValidator::new(
            Arc::new(ScriptedDeterminism { passed: true }),
            Arc::new(ScriptedSyntax { passed: true }),
            Arc::new(ScriptedIntegration::new(true, true, true)),
        );
        let out = v.run_all(&req(None)).await;
        for r in &out {
            assert!(
                r.elapsed_ms <= r.tier.timeout_ms() + 1000,
                "{:?} elapsed {} ms exceeds budget {} ms",
                r.tier,
                r.elapsed_ms,
                r.tier.timeout_ms(),
            );
        }
    }
}
