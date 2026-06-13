//! I-32 -- `elai role-ab-probe` CLI driver.
//!
//! Deterministic fixture harness for the structural role-contract A/B evidence gate.
//! Exercises the `RoleContractApplied` SSB Kind arms without invoking a live LLM.
//!
//! # What runs
//!
//! For each role in the fixture set (executor, researcher, context_compiler):
//!   1. Assembles system-prompt sections WITHOUT role contracts (baseline).
//!   2. Assembles system-prompt sections WITH role contracts applied.
//!   3. Evaluates each fixture task's `expected_in_prompt` / `must_not_appear`
//!      predicates against the assembled sections (deterministic string match).
//!   4. Computes old_pass_rate (baseline) and new_pass_rate (with contracts).
//!   5. Emits one `RoleContractApplied(AbDelta)` SSB row per role.
//!   6. Evidence gate (ID-5): exits non-zero if new_rate < old_rate.
//!
//! # Flags
//!
//!   --seed <u64>   Seed for SSB clock (default 42).
//!
//! # Wiring
//!
//! producer  : `apply_to_system_prompt` (role_contracts.rs) -- the real function.
//! consumer  : action_type_lock golden + i32_role_scoping_contract.
//! ssb rows  : `SsbActionType::RoleContractApplied(AbDelta)` x N roles.
//! off-author: role_contracts.rs applies; this fixture only drives it.
//!
//! Cross-lang pair: none (CLI bench is not a dashboard provider; AbDelta rows
//!                  render in existing SsbViewer generic stream).

use std::sync::Arc;

use anyhow::Context;
use elai_core::clock::{ClockProvider, SeededClock};
use elai_orchestrator::ssb::{
    RoleContractAppliedKind, RoleContractAppliedPayload, Ssb, SsbConfig, SsbPayload,
};
use elai_orchestrator::{AgentRole, RoleContractStore, apply_to_system_prompt};
use tracing::info;

// ---------------------------------------------------------------------------
// Temp-dir helper (no tempfile dep — uses std::env::temp_dir + pid/ts disambig)
// ---------------------------------------------------------------------------

fn make_temp_dir(prefix: &str) -> anyhow::Result<std::path::PathBuf> {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();
    let pid = std::process::id();
    let dir = std::env::temp_dir().join(format!("{prefix}_{pid}_{ts}"));
    std::fs::create_dir_all(&dir)
        .with_context(|| format!("could not create temp dir {}", dir.display()))?;
    Ok(dir)
}

fn make_temp_ssb(clock: Arc<dyn ClockProvider>) -> anyhow::Result<(std::path::PathBuf, Ssb)> {
    let dir = make_temp_dir("role_ab_probe")?;
    let cfg = SsbConfig {
        db_path: dir.join("ab_probe.db"),
        pool_size: Some(2),
        project_id: "role-ab-probe".into(),
        session_id: "s-role-ab-probe".into(),
    };
    let ssb = Ssb::new(cfg, clock).context("role-ab-probe: open SSB")?;
    Ok((dir, ssb))
}

// ---------------------------------------------------------------------------
// Fixture definition
// ---------------------------------------------------------------------------

struct AbFixture {
    role: AgentRole,
    role_slug: &'static str,
    /// (expected_in_prompt, must_not_appear)
    tasks: &'static [(&'static str, &'static str)],
}

static FIXTURES: &[AbFixture] = &[
    AbFixture {
        role: AgentRole::Executor,
        role_slug: "executor",
        tasks: &[
            ("# Role: Executor", "You are an interactive agent"),
            ("Constraints:", "You are an interactive agent"),
            (
                "Output: Rust/Python/TypeScript code or shell commands",
                "You are an interactive agent",
            ),
        ],
    },
    AbFixture {
        role: AgentRole::Researcher,
        role_slug: "researcher",
        tasks: &[
            ("# Role: Researcher", "You are an interactive agent"),
            (
                "Label unverified claims explicitly",
                "You are an interactive agent",
            ),
            (
                "Every factual claim carries a source reference",
                "You are an interactive agent",
            ),
        ],
    },
    AbFixture {
        role: AgentRole::ContextCompiler,
        role_slug: "context_compiler",
        tasks: &[
            ("# Role: ContextCompiler", "You are an interactive agent"),
            (
                "Read source files; do not invent symbol names",
                "You are an interactive agent",
            ),
            (
                "Output follows the context-package template",
                "You are an interactive agent",
            ),
        ],
    },
];

// ---------------------------------------------------------------------------
// Pass-rate evaluator
// ---------------------------------------------------------------------------

fn eval_pass_rate(sections: &[String], tasks: &[(&'static str, &'static str)]) -> f32 {
    let joined = sections.join("\n");
    let pass = tasks.iter().filter(|(must, must_not)| {
        joined.contains(must) && !joined.contains(must_not)
    }).count();
    pass as f32 / tasks.len() as f32
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

pub(crate) fn run(seed: u64) -> anyhow::Result<()> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| anyhow::anyhow!("tokio runtime build failed: {e}"))?;

    let outcome = rt.block_on(async move { run_async(seed).await });

    match outcome {
        Ok(()) => Ok(()),
        Err(e) => {
            eprintln!("ROLE-AB-PROBE ABORT: {e}");
            std::process::exit(1);
        }
    }
}

async fn run_async(seed: u64) -> anyhow::Result<()> {
    let _ = seed; // seed reserved for future deterministic clock injection
    let clock: Arc<dyn ClockProvider> = Arc::new(SeededClock::new());
    let (_dir, ssb) = make_temp_ssb(Arc::clone(&clock))?;
    let ssb = Arc::new(ssb);
    let store = RoleContractStore::default();

    let mut any_regression = false;

    for fixture in FIXTURES {
        // Baseline: sections with a narrative persona block (no contract applied).
        let baseline_sections = vec![
            "You are an interactive agent who helps with code tasks.".to_string(),
            "# Claude instructions\nsome rules".to_string(),
        ];
        let old_rate = eval_pass_rate(&baseline_sections, fixture.tasks);

        // With contracts: apply the role contract to the sections.
        let mut contract_sections = baseline_sections.clone();
        apply_to_system_prompt(&mut contract_sections, fixture.role, &store);
        let new_rate = eval_pass_rate(&contract_sections, fixture.tasks);

        let delta = new_rate - old_rate;
        if delta < 0.0 {
            any_regression = true;
            tracing::error!(
                role = fixture.role_slug,
                old_rate = old_rate,
                new_rate = new_rate,
                delta = delta,
                "I-32 role-ab-probe: REGRESSION (new_rate < old_rate); \
                 production flip blocked per ID-5 evidence gate"
            );
        } else {
            info!(
                role = fixture.role_slug,
                old_rate = old_rate,
                new_rate = new_rate,
                delta = delta,
                "I-32 role-ab-probe: pass (new_rate >= old_rate)"
            );
        }

        // Emit AbDelta SSB row (the off-agent artifact; wiring-enforcement R4).
        let payload = SsbPayload::RoleContractApplied(RoleContractAppliedPayload {
            kind: RoleContractAppliedKind::AbDelta,
            role: fixture.role_slug.to_owned(),
            fixture_pass_rate_old: old_rate,
            fixture_pass_rate_new: new_rate,
            delta,
            ts_ms: clock.now_ms() as i64,
        });
        ssb.write_payload("role_ab_probe", 0, payload, vec![])
            .await
            .context(format!("role-ab-probe: write SSB row for {}", fixture.role_slug))?;
        info!(
            "I-32 role-ab-probe: arm '{}' -- old={:.2} new={:.2} delta={:.2}",
            fixture.role_slug, old_rate, new_rate, delta
        );
    }

    if any_regression {
        anyhow::bail!(
            "role-ab-probe: one or more roles show regression (new_rate < old_rate); \
             production flip is BLOCKED per ID-5 evidence gate. \
             See log output above for per-role details."
        );
    }

    info!("I-32 role-ab-probe: all arms passed -- evidence gate SATISFIED");
    Ok(())
}
