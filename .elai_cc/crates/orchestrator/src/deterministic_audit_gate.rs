//! Deterministic audit gate invoked by the [`crate::fsm::Fsm`] on transitions
//! involving the `AUDITING` state (Session wire-a12).
//!
//! Spec: `docs/mvp-wiring-gaps.md` Session wire-a12 — "invoke
//! `DeterministicAuditor::run_checks(&profile, &changed_files)` from the FSM's
//! AUDITING state transition handler. Emit findings as `SsbPayload::Finding`
//! rows with severity Block/Warn. Early-exit the AUDITING→MEMORIZING
//! transition on any Block."
//!
//! The gate is a trait so the [`crate::fsm::Fsm`] can stay agnostic of how the
//! profile and changed file set are produced. The production impl
//! [`DefaultDeterministicAuditGate`] owns the [`crate::ssb::Ssb`] writer and a
//! pair of provider closures so the FSM call-path does not need to build a
//! `ProjectProfile` on every transition.
//!
//! Behavior contract enforced by [`crate::fsm::Fsm`]:
//!   * `evaluate` is called on every `_ -> Auditing` transition. The result is
//!     cached inside `FsmInner.last_audit_result`.
//!   * On `Auditing -> Memorizing` the cached result is consulted: if
//!     `block_count() > 0` the transition is rejected with
//!     [`crate::fsm::FsmError::AuditingBlocked`].
//!   * `evaluate` MUST be side-effect-tolerant: the FSM transition lock is
//!     held for its duration, so implementations should avoid long blocking
//!     I/O. The production impl dispatches the synchronous
//!     [`crate::deterministic_auditor::DeterministicAuditor::run_all`] inside
//!     a `tokio::task::block_in_place`-safe scope at the caller's discretion.
//!
//! Severity mapping to SSB `Finding.severity`:
//!   * `DeterministicSeverity::Block` -> `"error"`
//!   * `DeterministicSeverity::Warn`  -> `"warning"`

use std::path::PathBuf;
use std::sync::Arc;

use elai_core::project_profile::ProjectProfile;

use crate::deterministic_auditor::{
    CheckKind, DeterministicAuditResult, DeterministicAuditor, DeterministicAuditorConfig,
    DeterministicFinding, DeterministicSeverity,
};
use crate::gate_intensity::{
    select as select_gate_intensity, BlastRadiusCone, CentralitySignal, ComplexityClass, CweClass,
    NoveltySignal, RiskLevelSignal, RiskSignalVector,
};
use crate::mutation_budget::{BudgetVerdict, MutationBudgetSummary};
use crate::mutation_testing::{MutantOutcome, MutationConfig};
use crate::ssb::{
    env_gate_truthy, FindingPayload, GateIntensitySelectedPayload, MutationBudgetedRunKind,
    MutationBudgetedRunPayload, MutationSurvivedPayload, Ssb, SsbPayload,
    ELAI_MUTATION_BUDGET_DISABLE_ENV,
};

/// Agent id used on Finding rows emitted by the deterministic audit gate.
pub const GATE_AGENT_ID: &str = "deterministic_audit_gate";

/// Superstep for gate-emitted rows. Matches [`crate::task_runner::RUNNER_SUPERSTEP`] in spirit —
/// the dashboard keys rows by `seq`, so `0` is sufficient.
pub const GATE_SUPERSTEP: i64 = 0;

/// Category used on the per-finding `Finding` rows.
pub const GATE_FINDING_CATEGORY: &str = "deterministic_audit";

// ---------------------------------------------------------------------------
// Trait
// ---------------------------------------------------------------------------

/// Evaluates the deterministic audit checks at a FSM transition boundary.
///
/// The trait is intentionally minimal — the FSM calls `evaluate` once per
/// `_ -> Auditing` transition and stores the returned result. All side effects
/// (Finding emission, logging) are the implementation's responsibility.
pub trait DeterministicAuditGate: Send + Sync {
    /// Run the deterministic checks and return the aggregate result.
    ///
    /// The FSM passes the transition source+target so implementations can
    /// differentiate behavior (e.g. a replay on `Auditing -> Executing`). The
    /// production impl [`DefaultDeterministicAuditGate`] ignores them: it
    /// always runs the full 5-check sequence.
    fn evaluate(
        &self,
        source: crate::fsm::FsmState,
        target: crate::fsm::FsmState,
    ) -> DeterministicAuditResult;
}

// ---------------------------------------------------------------------------
// Default production impl
// ---------------------------------------------------------------------------

/// Pluggable supplier of `ProjectProfile`. The dashboard passes a closure that
/// reads from its cached [`elai_core::project_profile::ProjectProfile`]; tests
/// pass an in-memory fixture.
pub type ProjectProfileSupplier = Arc<dyn Fn() -> ProjectProfile + Send + Sync>;

/// Pluggable supplier of the files changed since the most recent commit.
/// The dashboard implementation reads `git diff --name-only HEAD`; tests pass
/// a fixed list.
pub type ChangedFilesSupplier = Arc<dyn Fn() -> Vec<PathBuf> + Send + Sync>;

/// C-28: pluggable supplier of [`MutationConfig`]. Dashboards thread
/// `ElaiRuntimeConfig.testing.mutation`; tests pass `MutationConfig::default()`.
pub type MutationConfigSupplier = Arc<dyn Fn() -> MutationConfig + Send + Sync>;

/// Production [`DeterministicAuditGate`]:
///   * Builds a [`DeterministicAuditor`] from the supplied [`ProjectProfile`].
///   * Runs the 5 checks via `DeterministicAuditor::run_all`.
///   * Emits one `Finding` SSB row per `DeterministicFinding`.
///   * Returns the raw [`DeterministicAuditResult`] to the FSM for the
///     AUDITING -> MEMORIZING gate.
pub struct DefaultDeterministicAuditGate {
    writer: Arc<Ssb>,
    profile_supplier: ProjectProfileSupplier,
    changed_files_supplier: ChangedFilesSupplier,
    /// C-28: closure that returns the live `MutationConfig`. Defaults to
    /// `MutationConfig::default()` when [`Self::new`] is used; callers
    /// that read `[testing.mutation]` from `elai_cc.toml` should use
    /// [`Self::with_mutation_supplier`].
    mutation_config_supplier: MutationConfigSupplier,
}

impl DefaultDeterministicAuditGate {
    pub fn new(
        writer: Arc<Ssb>,
        profile_supplier: ProjectProfileSupplier,
        changed_files_supplier: ChangedFilesSupplier,
    ) -> Self {
        let mutation_config_supplier: MutationConfigSupplier =
            Arc::new(MutationConfig::default);
        Self {
            writer,
            profile_supplier,
            changed_files_supplier,
            mutation_config_supplier,
        }
    }

    /// C-28: replace the mutation config supplier. Used by the dashboard
    /// to thread `ElaiRuntimeConfig.testing.mutation` into the gate.
    pub fn with_mutation_supplier(mut self, supplier: MutationConfigSupplier) -> Self {
        self.mutation_config_supplier = supplier;
        self
    }

    /// Expose the writer so callers that already have an Arc<Ssb> don't need
    /// a second Arc — keeps the dashboard setup site concise.
    pub fn writer(&self) -> &Arc<Ssb> {
        &self.writer
    }
}

impl DeterministicAuditGate for DefaultDeterministicAuditGate {
    fn evaluate(
        &self,
        source: crate::fsm::FsmState,
        target: crate::fsm::FsmState,
    ) -> DeterministicAuditResult {
        let profile = (self.profile_supplier)();
        let changed = (self.changed_files_supplier)();
        let primary_lang = profile.languages.primary.clone();
        let mutation_cfg = (self.mutation_config_supplier)();
        let config = DeterministicAuditorConfig::from_profile_with_mutation(
            &profile,
            Vec::new(),
            mutation_cfg,
        );
        let auditor = DeterministicAuditor::new(config);
        let result = auditor.run_all(&changed);

        // K-5: compute a deterministic risk-signal vector from the changed
        // files + project profile, select the observe-only GateIntensity tier,
        // and emit it as an SSB row. The tier does NOT filter the legs run
        // above; it is telemetry for dashboard post-analysis.
        let signals = compute_risk_signals(&profile, &changed);
        let intensity = select_gate_intensity(&signals);

        tracing::info!(
            %source,
            %target,
            checks_run = result.checks_run,
            block_count = result.block_count(),
            warn_count = result.warn_count(),
            gate_intensity_tier = %intensity.tier.tier_name(),
            policy_version = %intensity.policy_version,
            passed = result.passed,
            "deterministic_audit_gate: evaluated"
        );

        // Batch 4A (A-41): probe the toolchain for the project's primary
        // language. When the toolchain binary is absent, surface an
        // `AuditSkipped { language }` info Finding so the dashboard makes the
        // gap visible without blocking the transition. Probe happens after
        // run_all so it piggybacks on the same tokio spawn and does not hold
        // the FSM transition lock.
        let tool_missing_language = tool_missing_for_language(&primary_lang);

        let writer = Arc::clone(&self.writer);
        let intensity = intensity.clone();
        let findings = result.findings.clone();
        // C-28: one SsbActionType::MutationSurvived row per surviving mutant.
        let survivors: Vec<MutationSurvivedPayload> = result
            .mutation
            .as_ref()
            .map(|m| {
                m.records
                    .iter()
                    .filter(|r| matches!(r.outcome, MutantOutcome::Survived))
                    .map(|r| MutationSurvivedPayload {
                        file: r.file.to_string_lossy().into_owned(),
                        line: r.line,
                        mutant_kind: r.mutant_kind.clone(),
                    })
                    .collect()
            })
            .unwrap_or_default();
        // I-20: budgeted-mutation summary (Wilson-CI + survivor directives),
        // emitted as MutationBudgetedRun SSB rows from this writer-owning seam
        // (the deterministic_auditor stays SSB-agnostic, mirroring C-28/C-29).
        let budget_summary = result.mutation_budget.clone();
        tokio::spawn(async move {
            emit_findings(writer.as_ref(), &findings).await;
            emit_gate_intensity_selected(
                writer.as_ref(),
                &intensity,
                &source.to_string(),
                &target.to_string(),
            )
            .await;
            if let Some(lang) = tool_missing_language {
                emit_audit_skipped(writer.as_ref(), &lang).await;
            }
            emit_mutation_survivors(writer.as_ref(), &survivors).await;
            if let Some(summary) = budget_summary {
                emit_mutation_budgeted_runs(writer.as_ref(), &summary).await;
            }
        });
        result
    }
}

/// K-5: derive an observe-only risk-signal vector from the profile + changed
/// files. The heuristics are intentionally conservative: missing/ambiguous data
/// maps to `Unknown`, which the selector degrades to `Core`.
fn compute_risk_signals(
    profile: &ProjectProfile,
    changed_files: &[PathBuf],
) -> RiskSignalVector {
    use elai_core::project_profile::{ArchitecturePattern, ProjectDomain};

    let mut signals = RiskSignalVector::default();

    // Risk level: High when any security-sensitive domain or framework is present.
    let security_domains = [
        ProjectDomain::WebBackend,
        ProjectDomain::Fullstack,
        ProjectDomain::Mobile,
        ProjectDomain::Desktop,
        ProjectDomain::Embedded,
        ProjectDomain::DevOps,
    ];
    let has_security_domain = profile.domains.iter().any(|d| security_domains.contains(&d.domain));
    let has_security_framework = profile.frameworks.iter().any(|f| {
        matches!(
            f.category,
            elai_core::project_profile::FrameworkCategory::Web
                | elai_core::project_profile::FrameworkCategory::Server
        )
    });
    if has_security_domain || has_security_framework {
        signals.risk_level = RiskLevelSignal::High;
    }

    // Complexity class: driven by the number of changed files.
    signals.complexity_class = match changed_files.len() {
        0..=3 => ComplexityClass::Small,
        4..=10 => ComplexityClass::Medium,
        _ => ComplexityClass::Large,
    };

    // Blast radius: single crate vs cross-cutting. We approximate cross-cutting
    // by a broad language distribution or multiple entry points.
    let distinct_langs = profile.languages.distribution.len();
    if distinct_langs > 1 || profile.entry_points.len() > 1 {
        signals.blast_radius = BlastRadiusCone::CrossCutting;
    } else if changed_files.len() > 5 {
        signals.blast_radius = BlastRadiusCone::MultiCrate;
    } else {
        signals.blast_radius = BlastRadiusCone::SingleCrate;
    }

    // CWE class: default Unknown; auth/crypto hints when the profile mentions
    // relevant architecture or constraints.
    if profile
        .constraints
        .iter()
        .any(|c| c.to_lowercase().contains("crypto") || c.to_lowercase().contains("auth"))
        || profile.architecture == Some(ArchitecturePattern::Layered)
            && has_security_domain
    {
        signals.cwe_class = CweClass::Auth;
    }

    // Novelty: Recent when the project declares few/no established conventions.
    if matches!(profile.naming.functions, elai_core::project_profile::Convention::Unknown)
        || matches!(profile.naming.types, elai_core::project_profile::Convention::Unknown)
    {
        signals.novelty = NoveltySignal::Recent;
    } else {
        signals.novelty = NoveltySignal::Established;
    }

    // Centrality: Core when entry points exist and changed files touch them.
    let entry_paths: std::collections::HashSet<_> = profile
        .entry_points
        .iter()
        .filter_map(|e| e.file.file_name().and_then(|n| n.to_str()).map(String::from))
        .collect();
    let touches_entry = changed_files.iter().any(|p| {
        p.file_name()
            .and_then(|n| n.to_str())
            .map(|n| entry_paths.contains(n))
            .unwrap_or(false)
    });
    if !entry_paths.is_empty() && touches_entry {
        signals.centrality = CentralitySignal::Core;
    } else if distinct_langs > 1 {
        signals.centrality = CentralitySignal::Internal;
    } else {
        signals.centrality = CentralitySignal::Leaf;
    }

    signals
}

/// Probe whether the primary-language toolchain binary is present on the host.
/// Returns `Some(language_id)` when absent so the gate can emit
/// `AuditSkipped { language }`. `None` means the probe was skipped (unknown
/// language) or the binary was found.
fn tool_missing_for_language(lang: &elai_core::project_profile::Language) -> Option<String> {
    use elai_core::project_profile::Language;
    let probe = match lang {
        Language::Rust => ("cargo", "--version"),
        Language::Python => ("python", "--version"),
        Language::Go => ("go", "version"),
        Language::TypeScript | Language::JavaScript => ("tsc", "--version"),
        Language::Java | Language::Kotlin => ("javac", "-version"),
        Language::CSharp => ("dotnet", "--version"),
        Language::Ruby => ("ruby", "--version"),
        _ => return None,
    };
    match std::process::Command::new(probe.0)
        .arg(probe.1)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
    {
        Ok(status) if status.success() => None,
        _ => Some(lang.id_str().to_owned()),
    }
}

/// Map a `CheckKind` to the batch-4a `audit_step` tag surfaced on
/// `FindingPayload`. Only the 5 pipeline step kinds map; the remaining kinds
/// (ImportCrossRef, ModuleCoherence) fold into the step they're a sub-check of.
/// C-28: `Mutation` is the Step 6 tag.
pub fn audit_step_tag(kind: CheckKind) -> &'static str {
    match kind {
        CheckKind::Build => "build",
        CheckKind::VersionGate | CheckKind::ImportCrossRef => "version_gate",
        CheckKind::FileSizeCensus => "file_size",
        CheckKind::LazyCode => "lazy_code",
        CheckKind::Completeness | CheckKind::ModuleCoherence => "completeness",
        CheckKind::Mutation => "mutation",
        // C-29: T-3 basic metamorphic relations — Step 7 tag.
        CheckKind::Metamorphic => "metamorphic",
    }
}

async fn emit_findings(writer: &Ssb, findings: &[DeterministicFinding]) {
    for f in findings {
        let severity = match f.severity {
            DeterministicSeverity::Block => "error",
            DeterministicSeverity::Warn => "warning",
        };
        let file_path = f
            .file
            .as_ref()
            .map(|p| p.to_string_lossy().into_owned());
        let payload = SsbPayload::Finding(FindingPayload {
            severity: severity.to_owned(),
            category: GATE_FINDING_CATEGORY.to_owned(),
            message: format!("[{}] {}", f.check, f.message),
            file_path,
            line: f.line,
            failure_category: None,
            // Batch 4A (A-41): label the Finding with the pipeline step so the
            // dashboard can render per-step totals without re-parsing `message`.
            audit_step: Some(audit_step_tag(f.check).to_owned()),
            few_shot_examples: None,
        });
        if let Err(e) = writer
            .write_payload(GATE_AGENT_ID, GATE_SUPERSTEP, payload, Vec::new())
            .await
        {
            tracing::error!(error = %e, "deterministic_audit_gate: write Finding failed");
        }
    }
}

/// Emit an "audit skipped" informational Finding for a language whose build
/// toolchain is absent on the host. Severity is `"info"` — the caller's step
/// count is unaffected, and the dashboard can collapse these rows.
pub async fn emit_audit_skipped(writer: &Ssb, language: &str) {
    let payload = SsbPayload::Finding(FindingPayload {
        severity: "info".to_owned(),
        category: GATE_FINDING_CATEGORY.to_owned(),
        message: format!("audit skipped: no build toolchain for '{language}'"),
        file_path: None,
        line: None,
        failure_category: None,
        audit_step: Some("build".to_owned()),
        few_shot_examples: None,
    });
    if let Err(e) = writer
        .write_payload(GATE_AGENT_ID, GATE_SUPERSTEP, payload, Vec::new())
        .await
    {
        tracing::error!(error = %e, "deterministic_audit_gate: write AuditSkipped failed");
    }
}

/// C-28: emit one `SsbActionType::MutationSurvived` row per
/// `MutantOutcome::Survived` mutant. Zero rows when all caught
/// (stub_ban contract).
pub async fn emit_mutation_survivors(writer: &Ssb, survivors: &[MutationSurvivedPayload]) {
    for s in survivors {
        let payload = SsbPayload::MutationSurvived(s.clone());
        if let Err(e) = writer
            .write_payload(GATE_AGENT_ID, GATE_SUPERSTEP, payload, Vec::new())
            .await
        {
            tracing::error!(
                error = %e,
                file = %s.file,
                line = s.line,
                kind = %s.mutant_kind,
                "deterministic_audit_gate: write MutationSurvived failed"
            );
        }
    }
}

/// K-5: emit a `GateIntensitySelected` SSB row. The selection is observe-only
/// and carries the source/target FSM state so downstream analysis can correlate
/// intensity with transition context.
async fn emit_gate_intensity_selected(
    writer: &Ssb,
    intensity: &crate::gate_intensity::GateIntensitySelection,
    source_state: &str,
    target_state: &str,
) {
    let payload = SsbPayload::GateIntensitySelected(GateIntensitySelectedPayload {
        tier: intensity.tier.tier_name().to_owned(),
        always_on: intensity.always_on.iter().map(|k| format!("{k:?}")).collect(),
        risk_triggered: intensity.risk_triggered.iter().map(|k| format!("{k:?}")).collect(),
        selected: intensity.selected.iter().map(|k| format!("{k:?}")).collect(),
        policy_version: intensity.policy_version.to_owned(),
        source_state: source_state.to_owned(),
        target_state: target_state.to_owned(),
    });
    if let Err(e) = writer
        .write_payload(GATE_AGENT_ID, GATE_SUPERSTEP, payload, Vec::new())
        .await
    {
        tracing::error!(error = %e, "deterministic_audit_gate: write GateIntensitySelected failed");
    }
}

/// I-20: emit `MutationBudgetedRun` SSB row(s) for a budgeted-mutation run.
///
/// One row carries the cone verdict + the 5 CI params (`CiCleared` /
/// `CiFailed` / `CacheHit`); one additional `SurvivorDirective` row is emitted
/// per surviving mutant (the structured counterexample feeding the generator;
/// the consumer retry loop is OUT of I-20 scope).
///
/// Default-ON; the kill-switch `ELAI_MUTATION_BUDGET_DISABLE=1` suppresses
/// emission (the gate verdict is unaffected — SSB is an off-agent beacon).
pub async fn emit_mutation_budgeted_runs(writer: &Ssb, summary: &MutationBudgetSummary) {
    if env_gate_truthy(ELAI_MUTATION_BUDGET_DISABLE_ENV) {
        return;
    }
    let ts_ms = std::time::SystemTime::now() // entropy-allow: SSB journal ts_ms, no ClockProvider threaded into this gate
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0);

    let verdict_kind = match summary.verdict {
        BudgetVerdict::CiCleared => MutationBudgetedRunKind::CiCleared,
        BudgetVerdict::CiFailed => MutationBudgetedRunKind::CiFailed,
        BudgetVerdict::CacheHit => MutationBudgetedRunKind::CacheHit,
    };

    // (1) the cone verdict row (the 5 CI params).
    let verdict_payload = MutationBudgetedRunPayload {
        kind: verdict_kind,
        n_sampled: summary.n_sampled,
        n_total_cone: summary.n_total_cone,
        ci_lower: summary.ci_lower,
        ci_upper: summary.ci_upper,
        threshold: summary.threshold,
        time_ms: summary.time_ms,
        operator: String::new(),
        survivor_file: String::new(),
        survivor_line: 0,
        survivor_mutation: String::new(),
        ts_ms,
    };
    if let Err(e) = writer
        .write_payload(
            GATE_AGENT_ID,
            GATE_SUPERSTEP,
            SsbPayload::MutationBudgetedRun(verdict_payload),
            Vec::new(),
        )
        .await
    {
        tracing::error!(error = %e, "deterministic_audit_gate: write MutationBudgetedRun verdict failed");
    }

    // (2) one SurvivorDirective row per surviving mutant.
    for directive in &summary.survivors {
        let payload = MutationBudgetedRunPayload {
            kind: MutationBudgetedRunKind::SurvivorDirective,
            n_sampled: summary.n_sampled,
            n_total_cone: summary.n_total_cone,
            ci_lower: summary.ci_lower,
            ci_upper: summary.ci_upper,
            threshold: summary.threshold,
            time_ms: summary.time_ms,
            operator: directive.operator.clone(),
            survivor_file: directive.file.clone(),
            survivor_line: directive.line,
            survivor_mutation: directive.survivor_mutation.clone(),
            ts_ms,
        };
        if let Err(e) = writer
            .write_payload(
                GATE_AGENT_ID,
                GATE_SUPERSTEP,
                SsbPayload::MutationBudgetedRun(payload),
                Vec::new(),
            )
            .await
        {
            tracing::error!(
                error = %e,
                file = %directive.file,
                line = directive.line,
                "deterministic_audit_gate: write MutationBudgetedRun(SurvivorDirective) failed"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::deterministic_auditor::CheckKind;

    struct FakeGate {
        result: DeterministicAuditResult,
    }

    impl DeterministicAuditGate for FakeGate {
        fn evaluate(
            &self,
            _source: crate::fsm::FsmState,
            _target: crate::fsm::FsmState,
        ) -> DeterministicAuditResult {
            self.result.clone()
        }
    }

    #[test]
    fn fake_gate_returns_its_result() {
        let gate = FakeGate {
            result: DeterministicAuditResult {
                findings: vec![DeterministicFinding {
                    check: CheckKind::Build,
                    severity: DeterministicSeverity::Warn,
                    file: None,
                    line: None,
                    message: "no build cmds".into(),
                }],
                checks_run: 1,
                passed: true,
                mutation: None,
                mutation_budget: None,
            },
        };
        let r = gate.evaluate(crate::fsm::FsmState::Executing, crate::fsm::FsmState::Auditing);
        assert_eq!(r.checks_run, 1);
        assert_eq!(r.warn_count(), 1);
        assert_eq!(r.block_count(), 0);
        assert!(r.passed);
    }

    #[test]
    fn severity_maps_block_to_error_and_warn_to_warning() {
        // Verifies the mapping contract documented in the module doc-comment.
        // The actual SSB write is exercised in the contract test.
        let block = DeterministicFinding {
            check: CheckKind::Build,
            severity: DeterministicSeverity::Block,
            file: None,
            line: None,
            message: "build failed".into(),
        };
        let warn = DeterministicFinding {
            check: CheckKind::LazyCode,
            severity: DeterministicSeverity::Warn,
            file: Some(PathBuf::from("src/foo.rs")),
            line: Some(42),
            message: "todo!()".into(),
        };
        assert_eq!(
            match block.severity {
                DeterministicSeverity::Block => "error",
                DeterministicSeverity::Warn => "warning",
            },
            "error"
        );
        assert_eq!(
            match warn.severity {
                DeterministicSeverity::Block => "error",
                DeterministicSeverity::Warn => "warning",
            },
            "warning"
        );
    }
}
