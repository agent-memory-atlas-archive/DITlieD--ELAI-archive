# G-6: SAFEDELTA competitive safety-delta benchmark

STATUS: PLANNED (WAVE 3; DEFERRED behind F-2b LIVE + G-2; RunPod-gated for the live model+task arm)

Objective: a gate-ON-vs-gate-OFF ABLATION (GD-3) measuring the gate's MARGINAL residual-insecure-rate that
reaches a human, on git-stripped held-out tasks, with a secondary head-to-head vs competitor HARNESSES.
ELAI ALREADY SHIPS the static-gate half: SecCodePltRunner (seccodeplt_runner.rs:304) computes
insecure_rate_before (gate-OFF, :348) vs insecure_rate_after (gate-ON residual, :349) + gate_recall (:350)
over an EMBEDDED CWE corpus, emitting InsecureRateMeasured (ssb.rs:8317) — but that is static-gate-over-a-fixed-
corpus, and the bench has NEVER been RUN live (the SSB emit at bench_seccodeplt.rs:192 only fires inside a real
`elai bench seccodeplt` invocation). G-6 generalizes that ablation to a LIVE model+task matrix with dynamic
exploit execution, anti-gaming controls, and a multi-suite split, producing the headline "N fewer
dynamically-exploitable diffs per 100 tasks reach a reviewer" with a 95% bootstrap CI and a per-CWE breakdown —
NEVER a SWE-bench rank (GD-3). Estimated effort: ~XL (multi-session). DEFERRED: no external safety-delta claim
ships before ELAI's own single-harness dynamic floor (F-2b LIVE) is measured, and the matrix needs RunPod for
the competitor-harness + live-model arms (W-10 Tier-2; the in-process static-gate Tier-1 is the fail-closed part).

Pre-requisites: F-2b SecCodePltRunner SHIPPED (grep-confirmed 2026-06-02: seccodeplt_runner.rs run:304 /
from_embedded:223 / gate_blocks->evaluate_vibegate:397-398 / insecure_rate_before:348 / after:349 / gate_recall:350)
+ InsecureRateMeasured SSB (ssb.rs:8317, "insecure_rate_measured":9118, payload:25427, FLAT beacon) + the CLI
`elai bench seccodeplt` (bench_seccodeplt.rs:104 / emit_insecure_rate_measured:192, NEVER RUN live). determinism.rs
ELAI_SEED_ENV:48 + HardwareClass:78 SHIPPED. F-4 TestOutcomePayload.stdout_hash:9887 SHIPPED. F-2b LIVE keystone
(the dynamic single-harness extension of seccodeplt_runner) MUST land first — G-6 does NOT redefine F-2. G-2 RRA
is a FORWARD DEPENDENCY (rra.rs absent).
Parallel-safe with: G-3, G-4 (disjoint files), but G-6 EXECUTION is gated on F-2b LIVE + G-2. NOT parallel-safe
with any plan editing seccodeplt_runner.rs / the bench CLI dispatch in the same window.
Source: workflow/reasoning/ELAI-TRUST-MAXIMIZATION-2026-06-02.md R6. Doctrine: G-0_INDEX.md (W-1..W-10 + GD-1..GD-5).

## Context

Per the trust-max synthesis R6: the one credibility move no competitor makes is a MEASURED, reproducible,
anti-gameable safety-delta — the gate's marginal contribution, same model, gate-ON vs gate-OFF. ELAI ships the
static-gate ablation already: SecCodePltRunner::run (seccodeplt_runner.rs:304) computes the gate-OFF baseline
insecure_rate_before (:348), the gate-ON residual insecure_rate_after (:349, = vulnerable completions
evaluate_vibegate did NOT block, :397-398), and gate_recall (:350) over an embedded SeCodePLT-structured CWE
corpus (from_embedded:223), emitting the FLAT InsecureRateMeasured beacon (ssb.rs:8317 / payload:25427, SCHEMA
201). The InsecureRateMeasuredPayload doc (ssb.rs:25401-25415) is explicit that this is the HONEST STATIC-gate
rate, DISTINCT from the dynamic sandbox-executed rate which is "RunPod-gated and NOT carried here." That dynamic,
live-model, competitor-comparable measurement is exactly the G-6 gap.

SAFEDELTA adds, on top of the shipped static ablation: (1) DYNAMIC exploit execution (BaxBench-style: a diff is
"insecure" only if an exploit actually fires in a sandbox, not if a pattern matches); (2) a LIVE model+task arm
under a pinned ELAI_SEED + HardwareClass; (3) anti-gaming controls — git-history strip + a cheat-detector cloned
from Endor's finding that SWE-Agent+Opus exploited git history in 163/200 = 81% of tasks, plus a held-out CWE
split; (4) a multi-suite split (the static seccodeplt corpus alone is too narrow); (5) SAFEDELTA's own PASS bound
to an F-4 stdout_hash so the bench result is itself fabrication-resistant. The headline is framed gate-ON-vs-OFF
(the gate's marginal residual-insecure-rate reaching a reviewer), 95% bootstrap CI, per-CWE — NEVER a SWE-bench-
Verified rank or a perfect-code promise (GD-3; a green replay proves REPRODUCED not RIGHT).

ELAI ground truth (grep+source-confirmed 2026-06-02): the static-gate ablation core is
crates/orchestrator/src/benchmarks/seccodeplt_runner.rs::run (:304, before:348/after:349/gate_recall:350,
from_embedded:223, gate_blocks->evaluate_vibegate:397-398). The CLI is `elai bench seccodeplt [--json]`:
parse_bench_seccodeplt_args (crates/cli/src/args_bench.rs:744 — NO --suite flag, only --json) routed by
parse_bench_subcommand_args (args_bench.rs:29, sub-verbs :31-58: prmbench|mode-routing|conformal|arc|rag|sycophancy|
canary-regression|tamper|seccodeplt), dispatched main.rs:854 CliAction::BenchSeccodeplt -> bench_cmd::run_bench_seccodeplt
(crates/cli/src/bench_cmd.rs:139) -> crate::bench_seccodeplt::run (bench_seccodeplt.rs:104), emit_insecure_rate_measured
(bench_seccodeplt.rs:192, the never-RUN live emit). There is NO crates/cli/src/benchmark.rs (the bench harness is
bench_cmd.rs + per-suite files). The SSB substrate is SsbActionType::InsecureRateMeasured (ssb.rs:8317, sql:9118,
FLAT beacon not Pattern-2 at ssb.rs:2799) + InsecureRateMeasuredPayload (ssb.rs:25427), round-trip-proven by
f2_security_ssb_roundtrip_contract.rs:269/342, surfaced by commands_f2.rs::get_insecure_rate_latest (:180). The
determinism module is crates/orchestrator/src/determinism.rs (ELAI_SEED_ENV:48, HardwareClass:78 — same-class
comparison only; already imported by the runner at seccodeplt_runner.rs:55, seed round-trips, test asserts seed==42).
SAFEDELTA's own PASS binds to TestOutcomePayload.stdout_hash (ssb.rs:9887). bench_cmd.rs::run_tier_eval (:878) is the
only existing bench taking a --suites list; run_bench_ablation (:870) is the existing gate-on/off ablation tile. The
G-2 RRA wrapper (rra.rs) is ABSENT — G-6 is deferred behind it. SCHEMA_VERSION=205 (ssb.rs:2898), TOTAL_VARIANTS=482
(tests/action_type_lock.rs:534). NOTE: e45_cross_language_drift_contract.rs:294 still asserts TOTAL_VARIANTS=394 (a
pre-existing STALE E-45 snapshot assertion, already out of sync with 482, NOT a G-6 regression).

## Existing state (B/C/D/E/F coverage)
- F-2b SecCodePltRunner (crates/orchestrator/src/benchmarks/seccodeplt_runner.rs) — SHIPPED static-gate ablation:
  run:304, from_embedded:223, gate_blocks->evaluate_vibegate:397-398, before:348/after:349/gate_recall:350. G-6
  generalizes to a live+dynamic matrix; does NOT redefine F-2 (GD-1). NEVER RUN live (the SSB emit is invocation-only).
- InsecureRateMeasured SSB (ssb.rs:8317, "insecure_rate_measured":9118, payload:25427, FLAT beacon :2799, SCHEMA 201)
  — SHIPPED. Round-trip-proven (f2_security_ssb_roundtrip_contract.rs:269/342). The static-gate beacon G-6's matrix extends.
- CLI bench dispatch: parse_bench_subcommand_args (args_bench.rs:29, sub-verbs :31-58); parse_bench_seccodeplt_args
  (args_bench.rs:744, NO --suite); bench_cmd.rs run_bench_seccodeplt:139 / run_bench_ablation:870 / run_tier_eval:878
  (the --suites taker); main.rs:854 dispatch (fn run:388). A new `safedelta` sub-verb attaches as a NEW branch here.
- determinism.rs (crates/orchestrator) — ELAI_SEED_ENV:48, HardwareClass:78. The matrix runs under a pinned seed + class.
- F-4 TestOutcomePayload.stdout_hash (ssb.rs:9877/:9887) — SHIPPED. SAFEDELTA's own PASS binds to it (anti-fabrication).
- commands_f2.rs::get_insecure_rate_latest:180 — SHIPPED. The dashboard surface G-6 mirrors for safety_delta_measured.
- G-2 RRA wrapper (rra.rs) — ABSENT. FORWARD DEPENDENCY (G-6 wraps each bench run in an RRA so the delta is replayable).
- ssb.rs — SCHEMA 205 (:2898), TOTAL_VARIANTS 482 (tests/action_type_lock.rs:534).

## Reads (mandatory before coding)
- workflow/reasoning/ELAI-TRUST-MAXIMIZATION-2026-06-02.md R6 + GD-3 (ablation framing, never a SWE-bench rank)
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\benchmarks\seccodeplt_runner.rs (full — the static-gate ablation core G-6 generalizes)
- F:\.ELAI_workflow\.elai_cc\crates\cli\src\bench_seccodeplt.rs + bench_cmd.rs:130-150,860-890 + args_bench.rs:29-60,744 (the bench CLI dispatch + run_tier_eval --suites pattern)
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\determinism.rs:40-100 (ELAI_SEED_ENV + HardwareClass)
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\ssb.rs:8317,25427,9877 (InsecureRateMeasured + the F-4 stdout_hash bind)
- docs/thefinalglance/F-plan/F-SUMMARY_INDEX.md F-2 + F-2b + F-2c (the keystone G-6 sits on; do NOT redefine F-2)
- docs/thefinalglance/G-plan/G-2_reproducible_run_artifact.md (the RRA wrapper — forward dep)
- CLAUDE.md HARD RULES: blake3, no .unwrap()/.expect() in prod, anti-slopsquatting, SCHEMA baseline+N.

## Tasks (WAVE 3 — Tier-1 in-process static matrix; Tier-2 live+dynamic RunPod-gated)

1. **Add the SAFEDELTA matrix runner (~240 LOC in a new crates/orchestrator/src/benchmarks/safedelta_runner.rs).**
   `pub fn run_safedelta(suites: &[Suite], arms: &[Arm], seed: u64, hw: HardwareClass) -> SafeDeltaReport` running
   the gate-ON-vs-gate-OFF ablation per arm (Arm::{ElaiGateOn, ElaiGateOff, CursorFrozenModel, ClaudeCode, Copilot, Cline})
   over the suite set, reusing SecCodePltRunner's gate_blocks primitive (seccodeplt_runner.rs:397) for the static
   tier. `pub struct SafeDeltaReport { per_suite: Vec<SuiteResult>, per_cwe: Vec<PerCweCell>, safety_delta: f64,
   ci_low: f64, ci_high: f64, dynamically_exploitable_before: u32, after: u32 }`. The headline = (before - after)
   exploitable diffs per 100 tasks, 95% bootstrap CI. NO `.unwrap()`/`.expect()`. Tier-1 = the static gate over the
   embedded/held-out corpus; Tier-2 (dynamic exploit exec + competitor harnesses) is RunPod-gated (W-10).

2. **Anti-gaming controls (~120 LOC).** git-history strip (the task checkout carries no `.git` so the model cannot
   read the fix — the Endor 163/200=81% exploit); a cheat-detector that flags a completion reading git history /
   the held-out answer; a held-out CWE split (train CWEs disjoint from eval CWEs); DYNAMIC exploit execution
   (a diff counts insecure only if an exploit fires in the sandbox, BaxBench-style). NO static-pattern-only scoring
   for the dynamic suites (stub_ban).

3. **Bind SAFEDELTA's own PASS to an F-4 stdout_hash (~40 LOC).** The bench's own self-test PASS records its stdout
   via TestOutcomePayload.stdout_hash (ssb.rs:9887) so the SAFEDELTA result is itself fabrication-resistant (a
   fabricated bench PASS is detectable). GD-3: the report states "REPRODUCED not RIGHT."

4. **Add the `safety_delta_measured` SSB action_type (~70 LOC in ssb.rs, W-9).** Following the FLAT-beacon precedent
   of InsecureRateMeasured (ssb.rs:2799, NOT Pattern-2 — the safety-delta is a single measured tuple), or a 2-arm
   Pattern-2 if a gate-ON/gate-OFF discriminator is needed (the nearest Pattern-2 is ToolFilterEvent, 51st): the choice
   is a design decision, recorded in the decision doc. `SafetyDeltaMeasuredPayload { suite_set: String, model_id: String,
   safety_delta: f64, ci_low: f64, ci_high: f64, n_tasks: u32, seed: u64, corpus_blake3: String, gate_recall: f64 }`.
   sql_str + parse + CHECK + round-trip golden + action_type_lock. SCHEMA bump baseline 205 (ssb.rs:2898) -> 206;
   TOTAL_VARIANTS 482 (tests/action_type_lock.rs:534) -> 483 + ALL_VARIANTS row + golden regen.

5. **CLI surface `elai bench safedelta` (~110 LOC).** A NEW sub-verb branch in parse_bench_subcommand_args (args_bench.rs:29)
   — NOT a --suite bolted onto seccodeplt (parse_bench_seccodeplt_args:744 explicitly rejects --suite) — with its own
   parse_bench_safedelta_args (accepting --suites like run_tier_eval:878), a new CliAction variant, a new
   bench_cmd::run_bench_safedelta wrapper, and a main.rs:854-style dispatch arm. Emits safety_delta_measured + a signed,
   independently-replayable run manifest (the G-2 RRA wrapper, once shipped).

6. **Frontend pair: SafeDeltaPanel (~220 LOC svelte + ~110 LOC tauri) (W-4).** New SafeDeltaPanel.svelte rendering the
   per-suite / per-CWE safety-delta + the 95% CI; Tauri `get_safety_delta_latest` (mirror commands_f2.rs::get_insecure_rate_latest:180).
   cross_lang_diff.py PASS.

7. **Decision doc workflow/reasoning/G-6-decisions.md (~120 LOC).** Records: (a) ablation framing — gate-ON-vs-OFF
   marginal residual-insecure-rate, NEVER a SWE-bench rank (GD-3); (b) the suite citation set (below) with BaxBench
   flagged DYNAMIC; (c) the anti-gaming controls (git-history strip + cheat-detector + held-out CWE split + dynamic
   exploit exec) grounded in the Endor 163/200 finding; (d) the CyberSecEval landmine (cite 2404.13161 NOT 2408.01605);
   (e) the FLAT-beacon-vs-Pattern-2 SSB choice; (f) the DEFERRED gate — no external claim before F-2b LIVE + the CI
   strategy is resolved (small per-CWE cells); (g) the pre-existing stale e45 assertion (:294 TOTAL_VARIANTS=394) is
   NOT a G-6 regression.

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/benchmarks/safedelta_runner.rs::run_safedelta
  - crates/orchestrator/src/benchmarks/safedelta_runner.rs::SafeDeltaReport
  - crates/orchestrator/src/ssb.rs::SsbActionType::SafetyDeltaMeasured
  - crates/orchestrator/src/ssb.rs::SafetyDeltaMeasuredPayload
  - crates/cli/src/bench_safedelta.rs::run
  - crates/cli/src/args_bench.rs::parse_bench_safedelta_args
  - crates/dashboard-v2/src-tauri/src/commands_g6.rs::get_safety_delta_latest
  - crates/dashboard-v2/src/lib/drawers/SafeDeltaPanel.svelte
consumes_symbols:
  - crates/orchestrator/src/benchmarks/seccodeplt_runner.rs::gate_blocks (the static-gate primitive, :397) + run (:304)
  - crates/orchestrator/src/benchmarks/seccodeplt_runner.rs::from_embedded (:223)
  - crates/orchestrator/src/determinism.rs::ELAI_SEED_ENV (:48) + HardwareClass (:78)
  - crates/orchestrator/src/ssb.rs::TestOutcomePayload (F-4 stdout_hash self-PASS bind, :9877/:9887)
  - crates/orchestrator/src/ssb.rs::InsecureRateMeasuredPayload (the FLAT-beacon precedent, :25427)
  - crates/cli/src/args_bench.rs::parse_bench_subcommand_args (the bench sub-verb dispatch, :29)
  - crates/cli/src/bench_cmd.rs::run_tier_eval (the --suites pattern, :878) + run_bench_ablation (:870)
  - the G-2 RRA wrapper (FORWARD DEPENDENCY — rra.rs absent; G-6 wraps each run once G-2 ships, W-7 do-not-cite-as-present)
reachable_from:
  - crates/cli/src/args_bench.rs -> `elai bench safedelta --suites ...` -> bench_cmd::run_bench_safedelta ->
    run_safedelta -> safety_delta_measured row + replayable manifest  [PRODUCTION bench path; Tier-2 live arm RunPod-gated]
  - crates/dashboard-v2/src-tauri/src/commands_g6.rs::get_safety_delta_latest -> SafeDeltaPanel.svelte  [dashboard]
stub_ban:
  - NO SWE-bench-rank / model-capability / perfect-code framing — the headline is gate-ON-vs-OFF marginal residual rate (GD-3)
  - NO static-pattern-only scoring for the DYNAMIC suites — a diff is insecure only if an exploit fires (BaxBench-style)
  - NO --suite bolted onto seccodeplt (parse_bench_seccodeplt_args:744 rejects it) — safedelta is its OWN sub-verb
  - NO fabricated bench PASS — SAFEDELTA's own PASS binds to an F-4 stdout_hash
  - NO advertised live-safety claim while RunPod-pending — carry the honest runpod_pending state (GD-5); the static-gate
    Tier-1 number is honestly static-gate, the dynamic/competitor arm is gated
  - blake3 only (corpus_blake3); no blake2/sha2
  - do NOT cite crates/cli/src/benchmark.rs (does not exist) or crates/orchestrator/src/rra.rs (G-2 forward dep, absent)
  - cite CyberSecEval 1/2 = arXiv:2404.13161 (NOT 2408.01605, which is the offensive-cyber CyberSecEval 3)
frontend_pair:
  svelte: crates/dashboard-v2/src/lib/drawers/SafeDeltaPanel.svelte
  tauri: crates/dashboard-v2/src-tauri/src/commands_g6.rs::get_safety_delta_latest
observable_effect:
  trigger: "cd .elai_cc && cargo run -p elai-cli -- bench safedelta --suites seccodeplt --seed 42 (Tier-1 static
    ablation, NO env override); the live+dynamic+competitor arm is a SEPARATELY-gated RunPod checkbox"
  expect_ssb: "a safety_delta_measured row with safety_delta + ci_low/ci_high + n_tasks + corpus_blake3 (blake3) + seed
    + gate_recall in the live ssb.db; the safety_delta re-computes from the per-task before/after rows under the pinned seed (GD-2)"
  expect_artifact: "a signed, independently-replayable run manifest (the G-2 RRA wrapper once shipped); SafeDeltaPanel
    renders the per-suite/per-CWE delta + the 95% CI; the report states REPRODUCED-not-RIGHT (GD-3)"

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/G-plan/G-6_safedelta_benchmark.md --run-trigger
rg "pub fn run_safedelta|pub struct SafeDeltaReport" .elai_cc/crates/orchestrator/src/benchmarks/safedelta_runner.rs
rg "parse_bench_safedelta_args" .elai_cc/crates/cli/src/args_bench.rs                 # new sub-verb (NOT --suite on seccodeplt)
rg "SafetyDeltaMeasured|safety_delta_measured" .elai_cc/crates/orchestrator/src/ssb.rs
rg "safety_delta_measured" .elai_cc/crates/orchestrator/tests/action_type_lock.golden
rg -c "2408.01605" .elai_cc/crates/orchestrator/src/benchmarks/safedelta_runner.rs    # MUST be 0 (cite 2404.13161 for insecure-code)
rg -c "blake2|sha2" .elai_cc/crates/orchestrator/src/benchmarks/safedelta_runner.rs   # MUST be 0
test -f .elai_cc/crates/dashboard-v2/src/lib/drawers/SafeDeltaPanel.svelte
python .claude/tools/cross_lang_diff.py
test -f workflow/reasoning/G-6-decisions.md
```

## ACCEPTANCE
- [ ] W-1 done-gate (Tier-1): wiring_contract_check.py --run-trigger PASS; `elai bench safedelta --suites seccodeplt`
  RAN the static-gate ablation + emitted a safety_delta_measured row. cargo-green != DONE.
- [ ] W-2 default-ON: the Tier-1 static ablation runs with NO env override (the bench is invoked explicitly, but the
  gate it measures is default-ON); no default-off ENABLE flag on the gate.
- [ ] W-3 production entrypoint: `elai bench safedelta` is a real CLI sub-verb (args_bench.rs:29 dispatch), not test-only.
- [ ] W-6 dashboard E2E: SafeDeltaPanel renders the per-suite/per-CWE delta + CI via dashboard-smoker.
- [ ] GD-2: the safety_delta re-computes from the per-task before/after SSB rows under the pinned seed.
- [ ] GD-3: the report + decision doc frame the number as gate-ON-vs-OFF marginal residual rate, NEVER a SWE-bench rank;
  a contract test asserts no "rank"/"#1"/"beats" framing in the report serialization.
- [ ] GD-5: a RunPod-pending / dynamic / competitor-harness arm carries the honest runpod_pending state; the Tier-1
  static-gate number is labelled static-gate (synthetic-corpus caveat explicit), NOT a live dynamic claim.
- [ ] Anti-gaming: a contract test asserts the git-history-strip removes `.git` and the cheat-detector flags a
  git-history-reading completion (the Endor 163/200 exploit); the held-out CWE split is disjoint.
- [ ] safety_delta_measured round-trips via the SSB roundtrip contract + action_type_lock golden; SCHEMA bump named
  baseline 205 -> 206, TOTAL_VARIANTS 482 -> 483.
- [ ] cargo check --workspace GREEN; auditor PASS on -p elai-orchestrator -p elai-cli -p contracts.
- [ ] No new dependency for Tier-1 (seccodeplt_runner + determinism + blake3 in-tree); any RunPod/competitor-harness
  Tier-2 dep is anti-slopsquatting-verified + feature-gated.

## WIRING DONE-GATE (clause by clause)
- W-1: wiring_contract_check --run-trigger is ACCEPTANCE 1 (Tier-1 static ablation); run_safedelta (produced) is consumed
  by the bench CLI (W-8) reached by `elai bench safedelta` (W-3).
- W-2: the gate measured is default-ON; the bench is an explicit invocation (a measurement tool, not a gate); no default-off ENABLE on the gate.
- W-3: the `elai bench safedelta` sub-verb (args_bench.rs:29); not test-only.
- W-4: safety_delta_measured is human-surfaceable -> SafeDeltaPanel + get_safety_delta_latest; cross_lang_diff PASS.
- W-5: observable_effect Tier-1 is a runnable CLI -> a live ssb.db row; the dynamic+competitor Tier-2 ships a deterministic
  fixture harness over the real path + a SEPARATELY-gated live RunPod checkbox (W-10).
- W-6: ACCEPTANCE verifies the delta+CI THROUGH SafeDeltaPanel via dashboard-smoker.
- W-7: seccodeplt_runner.rs (run:304/gate_blocks:397), bench CLI (args_bench.rs:29/:744, bench_cmd.rs:139/:870/:878,
  main.rs:854), determinism.rs (:48/:78), InsecureRateMeasured (ssb.rs:8317/:25427), TestOutcomePayload.stdout_hash (:9887)
  grep-confirmed 2026-06-02; benchmark.rs + rra.rs confirmed ABSENT and NOT cited; the validator re-greps.
- W-8: run_safedelta (produced) called by bench_safedelta::run (Task 5); SafetyDeltaMeasuredPayload emitted (Task 4) +
  consumed by the Tauri command (Task 6). No produced symbol caller-less.
- W-9: ONE SafetyDeltaMeasured action_type (FLAT beacon precedent InsecureRateMeasured, or 2-arm Pattern-2 — design
  choice in the decision doc); action_type golden + cross_lang_diff; bump 205->206 / 482->483.
- W-10: TIERED — Tier-1 static-gate ablation is in-process + fail-closed (the done-gate); Tier-2 (dynamic exploit exec +
  competitor harnesses + live model) is network/subprocess/RunPod and SEPARATELY gated, never advertised fail-closed.
- GD-1: COMPOSES-WITH F-2b (the static runner, generalized — NOT redefined), F-4 (self-PASS bind), G-2 (RRA wrapper,
  forward dep). GD-3: ablation framing, never a SWE-bench rank. GD-5: runpod_pending state carried honestly.

## Tests to add
- crates/contracts/tests/g6_safedelta_static_contract.rs (~180 LOC): (a) run_safedelta Tier-1 over the embedded corpus
  -> a safety_delta matching the seccodeplt before/after; (b) the safety_delta re-computes from the per-task rows under
  seed=42 (GD-2); (c) the report carries NO rank/beats framing (GD-3); (d) the FLAT-beacon (or Pattern-2) round-trips.
- crates/contracts/tests/g6_anti_gaming_contract.rs (~140 LOC): (a) git-history-strip removes `.git`; (b) the
  cheat-detector flags a git-history-reading completion; (c) the held-out CWE split is disjoint (train ∩ eval = ∅);
  (d) a static-pattern match on a DYNAMIC suite without a fired exploit is NOT counted insecure.
- crates/contracts/tests/fixtures/g6/ (held-out tasks + git-stripped checkouts + a cheat fixture).

## Files touched
- CREATE: crates/orchestrator/src/benchmarks/safedelta_runner.rs (~240 LOC)
- CREATE: crates/cli/src/bench_safedelta.rs (~110 LOC)
- CREATE: crates/dashboard-v2/src-tauri/src/commands_g6.rs (~110 LOC)
- CREATE: crates/dashboard-v2/src/lib/drawers/SafeDeltaPanel.svelte (~220 LOC)
- CREATE: crates/contracts/tests/g6_safedelta_static_contract.rs (~180 LOC)
- CREATE: crates/contracts/tests/g6_anti_gaming_contract.rs (~140 LOC)
- CREATE: crates/contracts/tests/fixtures/g6/*.json (fixtures)
- CREATE: workflow/reasoning/G-6-decisions.md (~120 LOC)
- MODIFY: crates/orchestrator/src/benchmarks/mod.rs (~2 LOC: pub mod safedelta_runner;)
- MODIFY: crates/orchestrator/src/ssb.rs (~70 LOC: SafetyDeltaMeasured variant + payload + sql_str + parse + CHECK + golden)
- MODIFY: crates/orchestrator/tests/action_type_lock.rs (~2 LOC: TOTAL_VARIANTS 482->483 + ALL_VARIANTS row) + tests/action_type_lock.golden (~1 row)
- MODIFY: crates/cli/src/args_bench.rs (~30 LOC: safedelta sub-verb + parse_bench_safedelta_args) + args.rs (CliAction variant) + bench_cmd.rs (wrapper) + main.rs (dispatch)
- MODIFY: crates/dashboard-v2/src/lib/api/types.ts + DrawerHost.svelte (~6 LOC: DrawerKey 'safe-delta')

## Citations / Grounding
- workflow/reasoning/ELAI-TRUST-MAXIMIZATION-2026-06-02.md R6 (the ablation + the anti-gaming controls). Suite citation
  set (all IDs verifier-CONFIRMED): SUSVIBES-200 arXiv:2512.03262, BaxBench-392 arXiv:2502.11844 (DYNAMIC exploit exec),
  SeCodePLT arXiv:2410.11096 (the shipped embedded corpus family), AgentDojo arXiv:2406.13352, optional MSB arXiv:2510.15994.
  Anti-gaming git-history strip + cheat-detector grounded in Endor's finding (SWE-Agent+Opus exploited git history in
  163/200=81% of tasks). LANDMINE (binding): CyberSecEval 1/2 (arXiv:2404.13161) is the insecure-code-GENERATION suite;
  CyberSecEval 3 (arXiv:2408.01605) is OFFENSIVE-cyber — do NOT cite 2408.01605 as the insecure-code source. GD-3: a green
  replay proves REPRODUCED not RIGHT (DFAH arXiv:2601.15322).
- ELAI ground truth (grep-confirmed 2026-06-02): seccodeplt_runner.rs run:304 / from_embedded:223 / gate_blocks:397 /
  before:348 / after:349 / gate_recall:350; InsecureRateMeasured ssb.rs:8317 (FLAT beacon :2799) / payload:25427 (SCHEMA 201);
  bench CLI args_bench.rs:29/:744, bench_cmd.rs:139/:870/:878, main.rs:854, bench_seccodeplt.rs:104/:192 (never RUN live);
  benchmark.rs ABSENT; determinism.rs ELAI_SEED_ENV:48 / HardwareClass:78; TestOutcomePayload.stdout_hash ssb.rs:9887;
  commands_f2.rs get_insecure_rate_latest:180; round-trip f2_security_ssb_roundtrip_contract.rs:269/342. rra.rs ABSENT
  (G-2 forward dep). SCHEMA_VERSION 205 (ssb.rs:2898), TOTAL_VARIANTS 482 (tests/action_type_lock.rs:534). NOTE
  e45_cross_language_drift_contract.rs:294 asserts TOTAL_VARIANTS=394 (pre-existing stale E-45 snapshot, NOT a G-6 regression).
- CLAUDE.md HARD RULES: blake3, no .unwrap()/.expect() in prod, anti-slopsquatting, SCHEMA baseline+N.

## Open questions / risks
- CI tightness (whats_missing): with 200-392 tasks + small per-CWE cells, a significance claim is unresolved. The shipped
  runner emits per-CWE cells (PerCweReport) but has NO confidence-interval machinery (purely point estimates) — G-6 must
  add bootstrap CI and may need to POOL suites; cross-harness methodological defensibility is itself unverified. Resolve
  the CI strategy before any external claim (the DEFERRED gate).
- The competitor-harness arm (Cursor/Claude Code/Copilot/Cline) needs each harness run under matched conditions — that is
  the RunPod + access cost and the GD-5 honest-state surface (a harness ELAI cannot run is reported as not-run, not estimated).
- F-2b LIVE keystone MUST land first: G-6 does NOT redefine F-2 (GD-1). No external safety-delta claim before ELAI's own
  single-harness dynamic floor (F-2b) is measured.
- Do NOT regress the carried ground truth: benchmark.rs does not exist (bench harness is bench_cmd.rs + per-suite files);
  parse_bench_seccodeplt_args rejects --suite (safedelta is its own sub-verb); the F-2b bench was NEVER RUN live (only the
  contract round-trip is proven); the e45:294 TOTAL_VARIANTS=394 assertion is pre-existing-stale, not a G-6 regression;
  SCHEMA/TV are 205/482.

## Next
THE ONE HEADLINE MOVE (G-0): couple G-6 (SAFEDELTA) with G-2 (RRA) — the artifact makes the benchmark independently
replayable; the benchmark gives the artifact a published number. Together = ELAI-DIRECTION R1 (a real measured number on
ELAI's own engine, made FALSIFIABLE) + R2 (the anti-Replit distribution demo). G-6's per-CWE residual-rate also feeds the
G-5 cockpit (which CWE classes still leak through) and binds via F-4 (the bench's own PASS is fabrication-resistant).

End of G-6.
