G-6 decisions — SAFEDELTA competitive safety-delta benchmark (Tier-1 in-process fixture scaffold)
Session s-56068 (resume). Written before EXECUTING. Binding on the executor + auditor.

WHAT
Ship the Tier-1 in-process FIXTURE harness for the gate-ON-vs-gate-OFF safety-delta ablation:
a safedelta_runner + the +safety_delta_measured SSB variant + `elai bench safedelta` CLI + a
signed independently-replayable run manifest reusing G-2's RRA. DEFER the live multi-suite run +
any external number to F-2b/RunPod (Tier-2, separately gated, W-10). NEVER publish a safety-delta
number before the F-2b dynamic floor (GD-5).

LIVE BASELINE (code is ground truth; plan/CLAUDE.md headers STALE)
SCHEMA_VERSION 216 (ssb.rs:2972) -> 217. TOTAL_VARIANTS 493 (tests/action_type_lock.rs:605) -> 494.

DD1 — DERIVE, do not re-measure (GD-1 compose-not-restate, BLOCKING).
SAFEDELTA = insecure_rate_before - insecure_rate_after, both already computed by the SHIPPED
SecCodePltRunner::run() (crates/orchestrator/src/.../seccodeplt_runner.rs:304, pub). insecure_rate_before
= gate-OFF residual, insecure_rate_after = gate-ON residual. G-6 DERIVES the delta over that report;
it re-implements NO part of F-2. Do NOT widen the module-private gate_blocks (seccodeplt_runner.rs:397) —
touching shipped F-2b breaks parallel-safety. Reuse the pub run() only.

DD2 — new SSB Pattern-2 single variant SafetyDeltaMeasured (W-9, SCHEMA bump named).
Payload carries: safety_delta (f64, before-after), per_cwe (map CWE->delta), ci_low/ci_high (seeded
bootstrap-resample CI), corpus_id, stdout_hash (the F-4 binding — anti-gaming), runpod_pending (bool,
TRUE for Tier-1). FULL CASCADE — ALL of these (the G-8 SQL-CHECK trap is a runtime-only site cargo check
is blind to):
  - SsbActionType enum variant + its Display/serde census string
  - the action_type IN(...) SQL CHECK list in ssb.rs (~:4706-4761) — DO NOT MISS THIS
  - tests/action_type_lock.rs: ALL_VARIANTS array entry + TOTAL_VARIANTS const 493 -> 494 + golden + census
  - test_typed_payloads golden-lock arm for the new payload
  - crates/contracts/tests/e45_cross_language_drift_contract.rs:294: 487 -> 494 (it is ALREADY RED at the
    stale 487 vs live 493 — drift carried silently across G-1..G-7; bumping to 494 fixes the carry AND
    accounts for G-6). e49/e16 carry COMMENT-ONLY refs (no runtime assertion) -> DO NOT edit them.

DD3 — `elai bench safedelta` CLI, mirroring the F-2b precedent bench_seccodeplt.rs.
Slots: args_bench.rs:29 (subcommand enum), args.rs:836 (dispatch), main.rs:901 (wire), bench_cmd.rs:139
(handler). The command: run SecCodePltRunner::run() on a deterministic fixture corpus under ELAI_SEED ->
derive SAFEDELTA + per-CWE + seeded bootstrap CI -> bind PASS to the F-4 stdout_hash -> emit ONE
SafetyDeltaMeasured SSB row (Ssb::new-inside-rt.block_on pattern) -> write a timestamped JSON manifest
under benchmarks/safedelta/ AND a SIGNED replayable run manifest via G-2 build_rra/write_rra
(rra.rs:384/443, blake3-only root, like run_subcommand.rs:520-566).

DD4 — CLI-ONLY, NO frontend_pair (W-4 N/A, justified).
G-6's observable_effect is a CLI/CI artifact (the bench command + the row + the manifest), not a live
dashboard tile (unlike G-2/G-5/G-10 which name panels). No dashboard surface in Tier-1 -> no ts-rs/Tauri
cascade. A SafedeltaPanel is a deferred follow-on. Justify W-4 N/A in the wiring contract.

DD5 — GD-5 honest deferral + GD-2 reproducibility + anti-floor-fake.
Tier-1 runs the REAL ablation path (SecCodePltRunner::run on a fixture corpus) — the in-process fail-closed
done-gate. Tier-2 (live SUSVIBES-200 / BaxBench-392 / SeCodePLT-full / AgentDojo multi-suite + any external
number) is RUNPOD-PENDING: mark with the canonical pattern (swebench_pro.rs:43 ::pending() + a RunpodPending
path + a live test leg #[ignore = "requires ELAI_E2E_*=1 + RunPod"]). runpod_pending stays TRUE; emit NO
significance/CI-tightness claim for the degenerate single-corpus Tier-1 (GD-5). GD-2: the bootstrap CI is
SEEDED by ELAI_SEED so safety_delta + CI recompute byte-identically from the SSB journal.
ANTI-FLOOR-FAKE (G-7 lesson): the fixture corpus MUST contain >=2 tasks with at least one gate-CAUGHT
insecure case so before > after and SAFEDELTA is NON-zero/non-constant — a hollow all-clean corpus that
yields a constant 0 is a stub_ban violation. Anti-gaming scaffolds (git-history strip, the Endor 163/200=81%
cheat-detector ratio, held-out-CWE partition) ship as fixture-level scaffolds via from_json_with_seed,
reusing held_out.rs (9-gram contamination) + the canary fixtures; the LIVE anti-gaming run is Tier-2.

CARRIED LANDMINE (BINDING): CyberSecEval 3 (arXiv:2408.01605) is OFFENSIVE-cyber, NOT the insecure-code-gen
source — cite CyberSecEval 1/2 (arXiv:2404.13161). Suite IDs (verifier-confirmed): SUSVIBES 2512.03262,
BaxBench 2502.11844, SeCodePLT 2410.11096, AgentDojo 2406.13352.

HARD RULES: write only under .elai_cc/; blake3 only; no .unwrap()/.expect()/as!-narrow in prod; tracing not
log; cargo check ONLY (NOT cargo test); no new deps without confirmation; zero lazy code.

EXPECTED TEST OUTCOME: cargo check --workspace green; the new test_typed_payloads + action_type_lock golden
arms compile; the safedelta fixture test yields a NON-zero SAFEDELTA. UNKNOWNS: bootstrap CI tightness
(degenerate at Tier-1 -> runpod_pending); the stdout_hash binding source (confirm F-4 stdout_hash producer
at authoring).
