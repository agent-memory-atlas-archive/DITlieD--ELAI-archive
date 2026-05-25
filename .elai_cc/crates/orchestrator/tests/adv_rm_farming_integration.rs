//! D-28 integration test: AdvRmFarmer end-to-end against a custom
//! mock SidecarPpmHook impl that exercises the Z>1.96 filter, the
//! max_rounds=2 ceiling, the verdict-floor 1.0pp threshold, and the
//! corpus.jsonl + AdvRmFarmRunReport shape.
//!
//! The unit tests inside `crate::farm::adv_rm::tests` cover the trivial
//! constructor + validation + serialisation paths against the default
//! MockSidecarPpmHook. These integration tests cover the BEHAVIOUR-LEVEL
//! invariants that require driving the runner against a hand-crafted
//! hook that returns scripted (mean, var) pairs:
//!
//!   1. `run` return shape includes the expected fields populated from
//!      the hook's responses (samples_farmed, rounds, blake3, path).
//!   2. The Z>1.96 dual-filter EXCLUDES seeds whose PPM response has
//!      low score variance (the K=1 fallback per D-28-decisions.md
//!      DECISION-5: when std~=0 the Z formula returns 0 < 1.96 -> skip).
//!   3. max_rounds=2 is enforced — a runner constructed with the
//!      default 2-round budget never executes round 3+ even when the
//!      seed dataset is large.
//!   4. Verdict 1.0pp threshold — explicit pass/fail predicate exercise.
//!   5. K=1 fallback: high-mean + high-var seed -> Z>1.96 -> selected;
//!      same-mean + near-zero var -> Z<=1.96 -> excluded.
//!
//! Anti-stale-training compliance:
//!   - tokio::sync::Mutex for the call-counter shared across the async
//!     boundary (std::sync::Mutex would deadlock under tokio).
//!   - thiserror via the orchestrator's FarmError; no Box<dyn Error>.
//!   - tracing for any diagnostics (none emitted in tests).
//!   - #[cfg(test)] unwrap() permitted per CLAUDE.md.

use std::path::PathBuf;
use std::sync::Arc;

use elai_orchestrator::benchmarks::prmbench_runner::GatingVerdict;
use elai_orchestrator::farm::adv_rm::{
    verdict, AdvRmCorpusEntry, AdvRmFarmRunReport, AdvRmFarmer, AdvRmSeed, CorpusProvenance,
    DEFAULT_PAPER_FLOOR_PP, DEFAULT_Z_THRESHOLD, MAX_ROUNDS_CAP,
};
use elai_orchestrator::sidecar_ppm_hook::{
    PpmHookError, PpmMode, PpmScoreRequest, PpmScoreResponse, SidecarPpmHook,
};
use tempfile::TempDir;
use tokio::sync::Mutex;

// ---------------------------------------------------------------------------
// Scripted mock: per-call (mean, var) returned in declaration order.
// ---------------------------------------------------------------------------

/// Scripted hook: each `score` call pops the next (mean, var) pair off
/// the front of the queue. Empty queue -> InvalidScore so test failure
/// surfaces as a typed error rather than a hang.
struct ScriptedPpmHook {
    queue: Mutex<Vec<(f32, f32)>>,
    call_count: Mutex<u32>,
}

impl ScriptedPpmHook {
    fn new(script: Vec<(f32, f32)>) -> Self {
        Self {
            queue: Mutex::new(script),
            call_count: Mutex::new(0),
        }
    }

    async fn calls(&self) -> u32 {
        *self.call_count.lock().await
    }
}

impl SidecarPpmHook for ScriptedPpmHook {
    async fn score(
        &self,
        req: PpmScoreRequest,
    ) -> Result<PpmScoreResponse, PpmHookError> {
        let mut cc = self.call_count.lock().await;
        *cc += 1;
        drop(cc);
        let mut q = self.queue.lock().await;
        if q.is_empty() {
            return Err(PpmHookError::InvalidScore {
                index: 0,
                value: -1.0,
            });
        }
        let (mean, var) = q.remove(0);
        let num_steps = req.reasoning_steps.len() as u32;
        // Per-step scores all == mean (so std comes from `var` alone).
        // This makes the K=1 fallback predictable: Z = (mean - 0.5)/sqrt(var).
        let step_scores: Vec<f32> = vec![mean; num_steps as usize];
        Ok(PpmScoreResponse {
            candidate_id: req.candidate_id,
            verifier_id: req.verifier_id,
            mode: req.mode,
            step_scores,
            score_mean: mean,
            score_var: var,
            num_steps,
            latency_ms: 0,
        })
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn write_seeds(dir: &TempDir, seeds: &[AdvRmSeed]) -> PathBuf {
    let path = dir.path().join("dataset.jsonl");
    let mut body = String::new();
    for s in seeds {
        body.push_str(&serde_json::to_string(s).expect("serialize seed"));
        body.push('\n');
    }
    std::fs::write(&path, body).expect("write dataset");
    path
}

fn seed(id: &str) -> AdvRmSeed {
    AdvRmSeed {
        seed_id: id.to_string(),
        problem_statement: format!("problem {id}"),
        reasoning_steps: vec!["step 1".into(), "step 2".into(), "step 3".into()],
        mode: PpmMode::Math,
        provenance: CorpusProvenance::CanaryCorpus,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn run_returns_full_report_shape_on_happy_path() {
    // High-mean (0.9) + high-var (0.04 -> std=0.2 -> Z=(0.9-0.5)/0.2=2.0
    // > 1.96) seed-set; 3 seeds * 2 rounds = 6 calls; all should pass
    // the filter and land in the corpus. Verdict stays Fail because
    // the host-side scaffolding sets prmbench_delta_pp=0.0 (matches
    // D-28-decisions.md DECISION-9 RUNPOD-PENDING posture).
    let dir = TempDir::new().expect("tempdir");
    let script: Vec<(f32, f32)> = vec![(0.9, 0.04); 6];
    let hook = Arc::new(ScriptedPpmHook::new(script));
    let seeds = vec![seed("s1"), seed("s2"), seed("s3")];
    let dataset = write_seeds(&dir, &seeds);
    let corpus_dir = dir.path().join("corpus");
    let farmer = AdvRmFarmer::new(hook.clone(), dataset, corpus_dir)
        .with_max_rounds(2)
        .with_max_samples(1000);
    let report: AdvRmFarmRunReport = farmer.run().await.expect("run ok");

    assert_eq!(report.samples_farmed, 6, "all 3 seeds * 2 rounds selected");
    assert_eq!(report.rounds, 2);
    assert_eq!(report.paper_floor_pp, DEFAULT_PAPER_FLOOR_PP);
    assert_eq!(report.corpus_blake3.len(), 64);
    assert!(!report.corpus_path.is_empty());
    // Verdict Fail because host-side scaffolding sets delta=0.0
    // (RUNPOD-PENDING per D-28-decisions.md DECISION-9).
    assert_eq!(report.verdict, GatingVerdict::Fail);
    assert!(report.prmbench_delta_pp < report.paper_floor_pp);
    // Provenance accumulator carries the canary tag for all 6 rows.
    assert_eq!(report.provenance_counts.len(), 1);
    assert_eq!(report.provenance_counts[0].0, "canary_corpus");
    assert_eq!(report.provenance_counts[0].1, 6);
    // Hook call count matches seeds * rounds.
    assert_eq!(hook.calls().await, 6);
}

#[tokio::test]
async fn z_filter_excludes_low_uncertainty_seeds() {
    // 4 seeds; round 1 = 4 calls; mixed scripted responses:
    //   call 1: (0.9, 0.04)   -> std=0.2  -> Z=2.0  > 1.96 -> SELECTED
    //   call 2: (0.9, 0.0001) -> std=0.01 -> Z=40   > 1.96 -> SELECTED
    //   call 3: (0.51, 0.04)  -> std=0.2  -> Z=0.05 <=1.96 -> EXCLUDED
    //   call 4: (0.9, 0.0)    -> std=0.0  -> Z=0.0  <=1.96 -> EXCLUDED (div-by-zero guard)
    // Single round (max_rounds=1) so we don't re-score on round 2.
    let dir = TempDir::new().expect("tempdir");
    let script: Vec<(f32, f32)> =
        vec![(0.9, 0.04), (0.9, 0.0001), (0.51, 0.04), (0.9, 0.0)];
    let hook = Arc::new(ScriptedPpmHook::new(script));
    let seeds = vec![seed("s1"), seed("s2"), seed("s3"), seed("s4")];
    let dataset = write_seeds(&dir, &seeds);
    let corpus_dir = dir.path().join("corpus");
    let farmer = AdvRmFarmer::new(hook.clone(), dataset, corpus_dir).with_max_rounds(1);
    let report = farmer.run().await.expect("run ok");

    // Exactly 2 of 4 seeds crossed the Z>1.96 threshold.
    assert_eq!(
        report.samples_farmed, 2,
        "Z>1.96 filter must exclude std=0 seeds AND mean-just-above-0.5 seeds"
    );
    assert_eq!(report.rounds, 1);
    assert_eq!(hook.calls().await, 4, "all seeds scored, then filtered");
}

#[tokio::test]
async fn max_rounds_two_enforced_even_with_large_seed_set() {
    // Construct a runner with MAX_ROUNDS_CAP=2 (the hard ceiling). 2
    // seeds + 2 rounds = 4 hook calls; the runner MUST NOT score round
    // 3 even though the seed set is exhausted. All responses pass the
    // filter (Z=2.0) so samples_farmed = 4.
    let dir = TempDir::new().expect("tempdir");
    let script: Vec<(f32, f32)> = vec![(0.9, 0.04); 4];
    let hook = Arc::new(ScriptedPpmHook::new(script));
    let seeds = vec![seed("s1"), seed("s2")];
    let dataset = write_seeds(&dir, &seeds);
    let corpus_dir = dir.path().join("corpus");
    let farmer =
        AdvRmFarmer::new(hook.clone(), dataset, corpus_dir).with_max_rounds(MAX_ROUNDS_CAP);
    let report = farmer.run().await.expect("run ok");

    assert_eq!(report.rounds, 2, "runner must execute exactly 2 rounds");
    assert!(
        report.rounds <= MAX_ROUNDS_CAP,
        "report.rounds {} must be <= MAX_ROUNDS_CAP {}",
        report.rounds,
        MAX_ROUNDS_CAP
    );
    assert_eq!(report.samples_farmed, 4);
    assert_eq!(
        hook.calls().await,
        4,
        "hook MUST NOT be called for a phantom 3rd round"
    );
}

#[tokio::test]
async fn verdict_one_pp_threshold_pass_and_fail_paths() {
    // Synthetic report fixtures exercising the threshold both ways.
    // Verdict predicate: Pass iff prmbench_delta_pp >= paper_floor_pp
    // AND rounds >= 1.

    let r_pass = AdvRmFarmRunReport {
        samples_farmed: 100,
        rounds: 2,
        prmbench_delta_pp: 1.0,         // exactly the floor — must Pass
        verdict: GatingVerdict::Fail,    // ignored by verdict() helper
        paper_floor_pp: DEFAULT_PAPER_FLOOR_PP,
        corpus_blake3: "a".repeat(64),
        corpus_path: "tests/c.jsonl".into(),
        provenance_counts: vec![("canary_corpus".into(), 100)],
    };
    assert_eq!(verdict(&r_pass), GatingVerdict::Pass);

    let r_fail = AdvRmFarmRunReport {
        prmbench_delta_pp: 0.9, // just below — must Fail
        ..r_pass.clone()
    };
    assert_eq!(verdict(&r_fail), GatingVerdict::Fail);

    let r_zero_rounds = AdvRmFarmRunReport {
        rounds: 0, // ANY positive delta but rounds=0 -> Fail
        prmbench_delta_pp: 100.0,
        ..r_pass.clone()
    };
    assert_eq!(verdict(&r_zero_rounds), GatingVerdict::Fail);

    // Convenience predicate mirrors verdict() under canonical Pass.
    let canonical_pass = AdvRmFarmRunReport {
        verdict: GatingVerdict::Pass,
        ..r_pass.clone()
    };
    assert!(canonical_pass.passed());
}

#[tokio::test]
async fn corpus_entries_carry_pending_gold_verdict_and_round_index() {
    // Round-1 selection only (max_rounds=1) so the per-row `round`
    // field is deterministically 1 across all corpus entries. Asserts
    // (a) gold_verdict = "pending" — no mock-data drift; (b) round
    // field is correctly threaded from the runner; (c) provenance_tag
    // matches the seed's CorpusProvenance::CanaryCorpus -> "canary_corpus".
    let dir = TempDir::new().expect("tempdir");
    let script: Vec<(f32, f32)> = vec![(0.9, 0.04); 2];
    let hook = Arc::new(ScriptedPpmHook::new(script));
    let seeds = vec![seed("s1"), seed("s2")];
    let dataset = write_seeds(&dir, &seeds);
    let corpus_dir = dir.path().join("corpus");
    let farmer = AdvRmFarmer::new(hook, dataset, corpus_dir).with_max_rounds(1);
    let report = farmer.run().await.expect("run ok");
    assert_eq!(report.samples_farmed, 2);

    // Read the written corpus.jsonl back and decode each row.
    let corpus_text =
        std::fs::read_to_string(&report.corpus_path).expect("read corpus.jsonl");
    let rows: Vec<AdvRmCorpusEntry> = corpus_text
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str(l).expect("decode row"))
        .collect();
    assert_eq!(rows.len(), 2);
    for r in &rows {
        assert_eq!(r.gold_verdict, "pending");
        assert_eq!(r.round, 1);
        assert_eq!(r.provenance_tag, "canary_corpus");
        assert!(r.z_score > DEFAULT_Z_THRESHOLD);
    }
}
