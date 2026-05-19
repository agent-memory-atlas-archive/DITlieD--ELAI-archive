//! Batch 2B — 5-tier validation contract tests.
//!
//! Pins the `DefaultTieredValidator` behaviour so the spec stays
//! executable:
//!   1. Every tier fires on a full clean run (5 results in declared order).
//!   2. `run_all` short-circuits on the first failing tier.
//!   3. Per-tier timeout turns a slow runner into a failed tier.
//!   4. Determinism hash mismatch fails the tier.
//!   5. Stress tier detects flakiness on the Unit repeat loop.
//!   6. Unsupported language reports `passed=true` with a "skipped" snippet.
//!   7. `elapsed_ms` stays inside the per-tier budget.

use std::future::Future;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;
use std::time::Duration;

use elai_core::tool_dispatcher::{ValidateRequest, ValidationTier};
use elai_tools::tiered_validation::{
    DefaultTieredValidator, DeterminismCommandRunner, IntegrationCommandRunner,
    IntegrationRunOutcome, LanguageSyntaxRunner, SyntaxCommandRunner, TieredValidator,
};

// ---------------------------------------------------------------------------
// Scripted runners
// ---------------------------------------------------------------------------

struct ScriptedIntegration {
    step_passed: bool,
    phase_passed: bool,
    concept_passed: bool,
    delay: Duration,
}

impl IntegrationCommandRunner for ScriptedIntegration {
    fn run_integration<'a>(
        &'a self,
        level: &'a str,
        _pd: &'a Path,
    ) -> Pin<Box<dyn Future<Output = IntegrationRunOutcome> + Send + 'a>> {
        Box::pin(async move {
            tokio::time::sleep(self.delay).await;
            let passed = match level {
                "step" => self.step_passed,
                "phase" => self.phase_passed,
                "concept" => self.concept_passed,
                _ => true,
            };
            IntegrationRunOutcome {
                passed,
                rules_fired: 1,
                stderr_snippet: if passed { None } else { Some(format!("fake {level} fail")) },
            }
        })
    }
}

struct ScriptedSyntax(bool);
impl SyntaxCommandRunner for ScriptedSyntax {
    fn run_syntax<'a>(
        &'a self,
        _l: &'a str,
        _pd: &'a Path,
        _cf: &'a [PathBuf],
    ) -> Pin<Box<dyn Future<Output = IntegrationRunOutcome> + Send + 'a>> {
        let p = self.0;
        Box::pin(async move {
            IntegrationRunOutcome {
                passed: p,
                rules_fired: 1,
                stderr_snippet: if p { None } else { Some("syntax fail".into()) },
            }
        })
    }
}

struct ScriptedDeterminism(bool);
impl DeterminismCommandRunner for ScriptedDeterminism {
    fn run_determinism<'a>(
        &'a self,
        _pd: &'a Path,
    ) -> Pin<Box<dyn Future<Output = IntegrationRunOutcome> + Send + 'a>> {
        let p = self.0;
        Box::pin(async move {
            IntegrationRunOutcome {
                passed: p,
                rules_fired: 1,
                stderr_snippet: if p {
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

fn validator(det: bool, syn: bool, int: (bool, bool, bool), delay: Duration) -> DefaultTieredValidator {
    DefaultTieredValidator::new(
        Arc::new(ScriptedDeterminism(det)),
        Arc::new(ScriptedSyntax(syn)),
        Arc::new(ScriptedIntegration {
            step_passed: int.0,
            phase_passed: int.1,
            concept_passed: int.2,
            delay,
        }),
    )
    .with_stress_unit_repeats(2)
}

// ---------------------------------------------------------------------------
// 1. Every tier fires on a full clean run
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn every_tier_fires_on_clean_run() {
    let v = validator(true, true, (true, true, true), Duration::ZERO);
    let out = v.run_all(&req(None)).await;
    // C-28: ValidationTier::ordered() grew to 6 elements (Mutation added
    // between Unit and Integration).
    assert_eq!(out.len(), 6);
    let tiers: Vec<_> = out.iter().map(|r| r.tier).collect();
    assert_eq!(tiers, ValidationTier::ordered().to_vec());
    assert!(out.iter().all(|r| r.passed));
}

// ---------------------------------------------------------------------------
// 2. `run_all` short-circuits on first failing tier
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn short_circuits_on_first_fail() {
    let v = validator(true, false, (true, true, true), Duration::ZERO);
    let out = v.run_all(&req(None)).await;
    assert_eq!(out.len(), 2, "expected determinism + quality_ast only");
    assert!(out[0].passed);
    assert!(!out[1].passed);
    assert_eq!(out[1].tier, ValidationTier::QualityAst);
}

// ---------------------------------------------------------------------------
// 3. Per-tier timeout fails gracefully
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn tier_timeout_fails_tier() {
    // Unit tier budget = 30s. Delay 40s → timeout. `start_paused` requires
    // the current_thread runtime flavor; tokio::time::advance is gated
    // behind the `test-util` feature which the contracts crate enables.
    let v = validator(true, true, (true, true, true), Duration::from_secs(40));
    let handle = tokio::spawn(async move {
        v.run_all(&req(Some(vec![ValidationTier::Unit]))).await
    });
    tokio::time::advance(Duration::from_secs(31)).await;
    let out = handle.await.unwrap();
    assert_eq!(out.len(), 1);
    assert!(!out[0].passed);
    assert!(out[0].stderr_snippet.as_deref().unwrap_or("").contains("timeout"));
}

// ---------------------------------------------------------------------------
// 4. Determinism hash mismatch fails the tier
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn determinism_hash_mismatch_fails_tier() {
    let v = validator(false, true, (true, true, true), Duration::ZERO);
    let out = v.run_all(&req(None)).await;
    assert_eq!(out.len(), 1);
    assert!(!out[0].passed);
    assert!(out[0]
        .stderr_snippet
        .as_deref()
        .unwrap_or("")
        .contains("hash mismatch"));
}

// ---------------------------------------------------------------------------
// 5. Stress tier flakiness detection
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn stress_flakiness_is_detected() {
    // Custom flaky runner: concept passes, first step passes, 2nd step fails.
    struct Flaky {
        step_counter: AtomicU8,
    }
    impl IntegrationCommandRunner for Flaky {
        fn run_integration<'a>(
            &'a self,
            level: &'a str,
            _pd: &'a Path,
        ) -> Pin<Box<dyn Future<Output = IntegrationRunOutcome> + Send + 'a>> {
            Box::pin(async move {
                match level {
                    "concept" => IntegrationRunOutcome {
                        passed: true,
                        rules_fired: 1,
                        stderr_snippet: None,
                    },
                    "step" => {
                        let n = self.step_counter.fetch_add(1, Ordering::SeqCst);
                        let passed = n < 1;
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
    let v = DefaultTieredValidator::new(
        Arc::new(ScriptedDeterminism(true)),
        Arc::new(ScriptedSyntax(true)),
        Arc::new(Flaky {
            step_counter: AtomicU8::new(0),
        }),
    )
    .with_stress_unit_repeats(3);
    let out = v.run_tier(ValidationTier::Stress, &req(None)).await;
    assert!(!out.passed);
    assert!(out.stderr_snippet.as_deref().unwrap_or("").contains("flake"));
}

// ---------------------------------------------------------------------------
// 6. Unsupported language skip
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn unsupported_language_reports_skipped() {
    let runner = LanguageSyntaxRunner;
    let out = runner.run_syntax("brainfuck", Path::new("."), &[]).await;
    assert!(out.passed);
    assert!(out
        .stderr_snippet
        .as_deref()
        .unwrap_or("")
        .contains("unsupported language"));
}

// ---------------------------------------------------------------------------
// 7. `elapsed_ms` stays inside per-tier budget
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn elapsed_ms_bounded_by_tier_budget() {
    let v = validator(true, true, (true, true, true), Duration::ZERO);
    let out = v.run_all(&req(None)).await;
    for r in &out {
        assert!(
            r.elapsed_ms <= r.tier.timeout_ms() + 1_000,
            "{:?} elapsed {} ms exceeds budget {} ms",
            r.tier,
            r.elapsed_ms,
            r.tier.timeout_ms(),
        );
    }
}
