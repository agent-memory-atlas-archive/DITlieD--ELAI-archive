# H-25: Result manifest + re-derivation verifier (T-9) — bind the empirical NUMBER, not just the test

STATUS: PLANNED (TRACK 3 TRUST-HARDENING; WAVE 3 CLUSTER E; in-process BUILD extending the SHIPPED F-4 stdout_hash leg)

Objective: F-4 binds a test-PASS to an orchestrator-captured artifact (a fabricated PASS is detectable) — but
NOTHING binds an empirical RESULT (a reported Sharpe, P&L, plot) to its provenance. The Replit failure class (faked
tests + fabricated 4000 users) generalises: an autonomous system can REPORT a number that was never produced by the
pinned inputs. H-25 builds a SCHEMA_REQUIRED Result Manifest binding each reported number to its provenance
{value, captured_output_blake3, parser_expr, data_hash, code_commit, env_lock_hash, elai_seed, run_id, n_trials,
dsr_pvalue, pbo}, plus a T1 verifier that RE-PARSES the number from the captured output and RE-HASHES the pinned
inputs, asserting equality. The metric/backtest harness is in the gate-manifest (frozen); an AST_PATTERN forbids
strategy code from importing / redefining / monkeypatching the metric. This EXTENDS the SHIPPED F-4 stdout_hash leg
(task_runner.rs:16268, evidence.rs:845) from "the test PASS is honest" to "the empirical result is honest". The
DSR/PBO/N leg reuses H-17/H-18. Off-agent artifact: the captured output + the re-derived value.

Estimated effort: ~1 session (M, ~6 h; ~60% of primitives exist per the design). In-process (Tier-1).
Pre-requisites: F-4 stdout_hash SHIPPED (task_runner.rs:16268 emit_test_outcome stdout_hash; evidence.rs:845
  anti-fab filter; gate.rs:441); RRA evidence manifest SHIPPED (rra.rs:200); H-17 (n_trials) + H-18 (dsr_pvalue/pbo);
  the rules-engine SchemaRequired + AST_PATTERN + MetricGate SHIPPED.
Parallel-safe with: H-23/H-24 (shares the digest discipline), H-26.
Source: design T-9 + PART 0 (anti-fabrication); _grounding/H-TRUST-GROUNDING-MAP.md TS-6 + TS-5 + NN-9 + TD-1/2. Doctrine: H-0_INDEX.md.

## Context

ELAI's F-4 leg already proves a passing test is bound to an orchestrator-captured stdout_hash (a passing row with a
None hash is the fabrication signature — gate.rs:441, evidence.rs:845). H-25 generalises that discipline to an
empirical RESULT NUMBER: a reported Sharpe/P&L must carry a manifest binding it to {captured_output_blake3,
parser_expr, data_hash, code_commit, env_lock_hash, elai_seed, run_id, n_trials, dsr_pvalue, pbo}, and a verifier
RE-PARSES the number from the captured output and RE-HASHES the pinned inputs, asserting equality. A hand-edited
Sharpe whose captured-output digest does not re-derive to that value -> FAIL. An AST_PATTERN forbids the strategy
from importing/redefining/monkeypatching the metric (the harness is frozen — TD-2). The DSR/PBO/N fields reuse
H-17/H-18 (GD-1 — the result manifest is the place those numbers are bound, not re-computed).

## Existing state (B/C/D/E/F/G coverage)
- F-4 stdout_hash leg (crates/orchestrator/src/task_runner.rs:16268 emit_test_outcome stdout_hash; evidence.rs:845
  `.filter(|r| r.passed && r.stdout_hash.is_none())`; gate.rs:441 "a passing row with None hash is the fabrication
  signature"; TestIntegrityEvent SSB dashboard commands.rs:9610/9636) — SHIPPED. The exact pattern H-25 generalises
  to a result number (GD-1; do NOT restate the stdout_hash leg).
- RRA evidence manifest + blake3 (crates/orchestrator/src/rra.rs:200) — SHIPPED. The manifest extends this bundle.
- H-17 trial_count (n_trials) + H-18 DSR/PBO (dsr_pvalue/pbo) — the fields the manifest binds (prereq).
- rules-engine SchemaRequired + AST_PATTERN + MetricGate (predicate.rs:102; rule_engine.rs:98) — SHIPPED. The
  manifest schema + the monkeypatch ban + the re-derivation gate.
- a result-manifest schema + re-parse verifier — ABSENT (NN-9). The pure-BUILD surface (~60% of primitives exist).

## Reads (mandatory before coding)
- _grounding/H-TRUST-GROUNDING-MAP.md: TS-6, TS-5, NN-9, TD-1, TD-2, the H-25 symbol table.
- design T-9 + PART 0; H-18 (the DSR/PBO fields), H-17 (n_trials), H-23 (the digest discipline).
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\evidence.rs:840-850 (the anti-fabrication filter pattern)
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\gate.rs:435-445 (the None-hash fabrication-signature comment)
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\rra.rs:180-260 (the evidence manifest to extend)
- F:\.ELAI_workflow\.elai_cc\crates\rules\src\predicate.rs:100-120 (SchemaRequired + AstPattern + MetricGate)
- CLAUDE.md HARD RULES: blake3 (captured_output + data + env hashes), no .unwrap()/.expect(); TD-1, TD-2.

## Tasks (WAVE 3 — extend F-4 to empirical results)

1. **SCHEMA_REQUIRED Result Manifest (~60 LOC).** A typed manifest: {value, captured_output_blake3, parser_expr,
   data_hash, code_commit, env_lock_hash, elai_seed, run_id, n_trials (from H-17), dsr_pvalue + pbo (from H-18)}.
   Every reported empirical number (Sharpe/P&L/plot-derived stat) MUST carry one; a number without a manifest =
   BLOCK (W-1/R1 — the anti-fabrication signature, mirroring the F-4 None-hash rule).

2. **T1 re-derivation METRIC_GATE (~80 LOC).** A gate that (a) RE-PARSES the number from the captured output using
   parser_expr and asserts it equals `value`; (b) RE-HASHES the pinned inputs (data_hash, env_lock_hash, code_commit)
   and asserts they match; (c) checks n_trials/dsr_pvalue/pbo against H-17/H-18 (reuse, do NOT recompute). A
   hand-edited number whose captured-output digest does not re-derive -> FAIL. Missing the captured output = FAIL.

3. **AST_PATTERN: metric-harness integrity (~30 LOC).** An AstPattern forbidding strategy code from importing /
   redefining / monkeypatching the metric/backtest harness (the harness is in the gate-manifest frozen set — TD-2).
   Monkeypatching the metric fn = BLOCK.

4. **Off-agent artifact (~20 LOC).** Emit `result_manifest_verified` (Pattern-2 with Kind: ReDerived / FabricationDetected
   / MonkeypatchBlocked; baseline+1 from LIVE 207/484, re-read + bump e45:294). The captured output + the re-derived
   value are the off-agent artifacts (the agent did not author the re-parse).

5. **Freeze (TD-2).** Confirm the metric/backtest harness is in .claude/gate-manifest.txt (it is, per H-18/H-28);
   the AST ban enforces no in-strategy redefinition.

6. **Decision doc workflow/reasoning/H-25-decisions.md (~80 LOC).** Records: (a) extends F-4 (test-PASS) to an
   empirical RESULT, restating none (GD-1); (b) the manifest fields + the re-parse/re-hash verification; (c) the
   DSR/PBO/N fields are BOUND from H-17/H-18, not recomputed; (d) the monkeypatch AST ban + the frozen harness (TD-2).

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/result_manifest.rs::ResultManifest (the SCHEMA_REQUIRED typed manifest)
  - crates/orchestrator/src/result_manifest.rs::verify_result (the re-parse + re-hash verifier)
  - crates/rules/seed/finance/result_manifest.toml (the SchemaRequired manifest gate + the monkeypatch AST ban, mutable=0)
  - crates/orchestrator/src/ssb.rs::SsbActionType::ResultManifestVerified (Pattern-2 3-Kind; baseline+1)
consumes_symbols:
  - crates/orchestrator/src/evidence.rs::(the F-4 stdout_hash anti-fab pattern) + gate.rs:441 — REUSE the discipline, GD-1
  - crates/orchestrator/src/rra.rs::RunArtifact (the evidence manifest to extend — REUSE)
  - crates/orchestrator/src/trial_ledger.rs::trial_count (H-17 n_trials) + the H-18 dsr_pvalue/pbo (REUSE, do NOT recompute)
  - crates/rules/src/predicate.rs::{SchemaRequired, AstPattern, MetricGate} (REUSE)
reachable_from:
  - the result-reporting path (every reported empirical number routes through verify_result before it lands) [PRODUCTION gate]
  - verify_result -> the re-parse + re-hash -> ResultManifestVerified SSB row [off-agent artifact]
stub_ban:
  - NO reported number without a manifest (the anti-fabrication signature — missing manifest = BLOCK, W-1/R1)
  - NO in-strategy import/redefine/monkeypatch of the metric harness (AST ban — TD-2)
  - NO recomputation of DSR/PBO/N (bind from H-17/H-18 — GD-1)
  - NO re-implementation of the F-4 stdout_hash leg (extend it — GD-1 P0); blake3 only; no .unwrap()/.expect() in prod
frontend_pair:
  the reported number + its re-derived value + provenance in the reviewer cockpit (composes G-5 DSR cockpit). cross_lang_diff PASS. W-6: a fabrication-detected number visible in the dashboard.
observable_effect:
  trigger: "cd .elai_cc && cargo run -p elai-cli -- <report a fixture result whose manifest value was HAND-EDITED to differ from its captured-output digest, NO env override>"
  expect_ssb: "a ResultManifestVerified(FabricationDetected) row lands naming the number that did not re-derive, recomputable from the journal (GD-2)"
  expect_artifact: "the hand-edited Sharpe whose captured-output digest does not re-derive to that value is FAILED; a clean manifest re-derives ReDerived; a strategy that monkeypatches the metric fn is BLOCKed by the AST rule"

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/H-plan/H-25_result_manifest_rederivation.md --run-trigger
rg "verify_result|ResultManifest|stdout_hash" .elai_cc/crates/orchestrator/src --glob '!*test*'   # extends the F-4 leg
rg -n "result_manifest|monkeypatch|import.*metric" .elai_cc/crates/rules/seed/finance/result_manifest.toml  # the SchemaRequired + AST ban present
rg -c "blake2|sha2" .elai_cc/crates/orchestrator/src/result_manifest.rs                            # MUST be 0
test -f workflow/reasoning/H-25-decisions.md
```

## ACCEPTANCE
- [ ] W-1 done-gate: --run-trigger PASS (a hand-edited number that does not re-derive is FAILED; a clean manifest re-derives). cargo-green != DONE.
- [ ] W-2 default-ON: verify_result runs on every reported number with no env override; kill-switch only.
- [ ] W-3 production entrypoint: verify_result fires from the result-reporting path, not a #[cfg(test)] caller.
- [ ] TD-1: n_trials bound from H-17. TD-2: the metric harness is frozen + the monkeypatch AST ban active.
- [ ] GD-1: F-4 stdout_hash leg + RRA + H-17/H-18 REUSED (grep shows one stdout_hash discipline). GD-2: the re-derived value re-computes under ELAI_SEED.
- [ ] cargo check --workspace GREEN; auditor PASS. No new dependency.

## WIRING DONE-GATE (clause by clause)
- W-1: --run-trigger is ACCEPTANCE item 1; verify_result (produced) consumed on the result-reporting path (W-8) reached by reporting (W-3).
- W-2: no override; kill-switch only. W-3: live reporting path; not test-only.
- W-4: frontend_pair = reviewer cockpit (composes G-5); cross_lang_diff PASS. W-5: runnable fixture -> real row; Tier-1 in-process.
- W-6: fabrication-detected number visible in dashboard. W-7: evidence.rs:845 + gate.rs:441 + rra.rs:200 + H-17/H-18 grep-confirmed (TS-6/TS-5); manifest verifier absent (NN-9).
- W-8: verify_result (produced) consumed on reporting; the F-4 discipline + H-17/H-18 reused.
- W-9: +ResultManifestVerified Pattern-2 baseline+1; bump e45:294.
- W-10: Tier-1 in-process (re-parse + re-hash). TD-1/2: bound + frozen.

## Tests to add
- crates/contracts/tests/h25_result_manifest_contract.rs (~160 LOC): (a) hand-edited value that does not re-derive ->
  FabricationDetected; (b) clean manifest -> ReDerived; (c) reported number with no manifest -> Block; (d) strategy
  monkeypatching the metric fn -> MonkeypatchBlocked; (e) n_trials/dsr_pvalue/pbo are bound from H-17/H-18, not recomputed;
  (f) re-hash mismatch on data_hash -> Fail; (g) ELAI_SEED twice -> identical re-derived value.
- crates/contracts/tests/fixtures/result_manifest/ (a hand-edited result + a clean one + a monkeypatch attempt).

## Files touched
- CREATE: crates/orchestrator/src/result_manifest.rs (~140 LOC: ResultManifest + verify_result)
- CREATE: crates/rules/seed/finance/result_manifest.toml (SchemaRequired manifest + monkeypatch AST ban, mutable=0)
- MODIFY: crates/orchestrator/src/ssb.rs (+ResultManifestVerified Pattern-2) + action_type_lock.rs + e45 :294
- MODIFY: the result-reporting path (~20 LOC: route reported numbers through verify_result)
- MODIFY: .claude/gate-manifest.txt (confirm the metric/backtest harness is frozen)
- CREATE: crates/contracts/tests/h25_result_manifest_contract.rs (~160 LOC)
- CREATE: workflow/reasoning/H-25-decisions.md (~80 LOC)

## Citations / Grounding
- design T-9 + PART 0 (the Replit faked-tests + fabricated-4000-users class); DVC dvc.lock + Sigstore/Rekor
  append-only + in-toto (the provenance-binding precedent — composes with G-8 attestation).
- ELAI ground truth (TS-6/TS-5/NN-9, grep-confirmed 2026-06-04): evidence.rs:845 anti-fab filter; gate.rs:441
  None-hash signature; rra.rs:200 manifest; H-17 trial_count + H-18 DSR/PBO; result-manifest verifier ABSENT.
- CLAUDE.md HARD RULES: blake3, no .unwrap()/.expect(); TD-1/2; GD-1 (extend F-4, restate none).

## Open questions / risks
- parser_expr injection: the parser that re-extracts the number from captured output must itself be frozen + safe
  (a malicious parser_expr could "re-derive" any value). The parser_expr lives in the frozen manifest schema with a
  restricted grammar, not arbitrary code; verify the parser is sandboxed.
- "Captured output" provenance: the output must be orchestrator-captured (like F-4 stdout_hash), not agent-supplied —
  reuse the F-4 capture path exactly so the agent cannot author its own "captured" output.
- Plots/non-numeric results: a Sharpe is re-parseable; a plot is harder. Scope H-25 to numeric results first; visual
  results route to the media_validator VLM (B-24, shipped) for content check (HD-5 — a picture is valid-not-verified).

## Next
H-26 (runtime differential / shadow execution + financial-invariant MR library) catches WRONG VALUES (sign errors,
ULP cancellation) that static + wiring gates cannot see. H-25 + H-26 close CLUSTER E (anti-fabrication + dynamic
verification). Then CLUSTER F (the meta-layer): H-27 verifies the verifiers, H-28 guards self-mod reward tampering.

End of H-25.
