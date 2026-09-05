# P-14: explore-then-crystallize record->replay lane

STATUS: PLANNED (WAVE CU-2; EXTENSION of E-14 + O-2; PROOF_SPINE: feeds PS-5; execute after the proof spine per A-7)

- Track COMPUTER-USE-STEAL | lever agentic-run-mints-artifact | grounding STRONG (cua replay_env record->re-score verified HIGH in the eval doc; openclaw jsonl-replay is INPUT-through-mock NOT record->re-score, C4) | nature EXTENSION | effort M-L | WAVE CU-2 | depends P-13 | PROOF_SPINE feeds PS-5
- Premise: the strongest single design idea in the canvas eval. ELAI has the GENERAL form (C template crystallizer, J verified case graph, O-2 TaskSpecs) but no browser/GUI INSTANCE of "an agentic exploration run mints a deterministic replayable artifact". This adds it: an exploration run emits a deterministic script AS an O-2 TaskSpec stage + an RRA receipt binding script->minting-run; replay = the cheap zero-reasoning execution mode.

## Context — why this matters
Grounded in workflow/reasoning/canvas-computer-use-eval-2026-07-11.md ENTRY 2 + SYNTHESIS 2b + ENTRY 10/13. The industry split (from the eval's X-post source): EXPLORATION mode is agentic + expensive; EXECUTION mode emits a raw Playwright script of the exact steps and replays it forever at ~zero reasoning cost + ~100% reliability. cua's replay_env (ENTRY 10, environment.py) is the closest verified prior art — it records step:before/step:after into a typed HF Dataset and deterministically re-walks + re-scores from a recorded programmatic evaluate_task verdict; "determinism is a replay property, not a live-run property" is exactly ELAI's doctrine, independently converged. CORRECTION (C4, ENTRY 13): openclaw's jsonl-replay re-drives INPUT transcripts through a MOCK provider, it is NOT a record->re-score — cite cua, not openclaw, as the shape prior art.

## Existing ground truth (B..Q coverage; PLAN-asserted; executor re-greps W-7)
- E-14 Browser E2E (E-SUMMARY_INDEX.md:134): the agentic exploration run this crystallizes FROM.
- O-2 scheduled TaskSpec substrate (O-SUMMARY_INDEX.md:32; task_spec_fired row): the crystallized script becomes a TaskSpec STAGE, dispatched through the normal FSM (no new dispatch machinery).
- G-2 RRA (action stream + seed + gate-verdict hashes + `elai replay --verify`): the receipt that binds a script to the run that minted it.
- C compiled-pipeline + template crystallizer (C-plan, "cache a recurring pipeline shape"): the GENERAL form this instantiates for the browser/GUI lane — compose, do NOT fork.
- J-96 verified case graph (J-SUMMARY_INDEX.md:2086): a crystallized flow is a reuse artifact, adjacent to the case graph — feeds the same PS-5 compounding.

## Steal/build (REVISED scope, W-10 tiered)
- SHIP NOW (Tier-1): a crystallizer that takes an E-14 (browser) exploration trace and EMITS a deterministic Playwright script as an O-2 TaskSpec stage kind, with a G-2 RRA receipt binding the emitted script's hash to the exploration run_id that minted it; replay drives the real entrypoint with NO model in the loop.
- DEFER (Tier-2): the pyautogui (desktop) crystallizer over P-12 traces; the re-score-on-replay leg (cua evaluate_task analog — a programmatic per-task verifier re-derivable on replay); assertion synthesis from the exploration's observed AXTree deltas.
- EXCLUDED as vaporware: an openclaw-style jsonl "replay" that only re-feeds inputs through a mock (C4 — not a real record->re-score; do not frame it as one).

## Reads (mandatory before coding)
- workflow/reasoning/canvas-computer-use-eval-2026-07-11.md (ENTRY 2, 10, 13; SYNTHESIS 2b/P1)
- docs/thefinalglance/O-plan/O-2_*.md (TaskSpec stage kinds), docs/thefinalglance/E-plan/E-14_*.md (the trace source), docs/thefinalglance/G-plan (G-2 RRA binding)
- crates/orchestrator/src/task_runner.rs (the O-2 tick + stage dispatch)

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/crystallize/mod.rs (Crystallizer: E-14 trace -> deterministic Playwright script as an O-2 stage)
  - crates/orchestrator/src/crystallize/rra_bind.rs (script_hash -> minting run_id RRA receipt binder)
consumes_symbols:
  - crates/orchestrator/src/task_runner.rs::TaskSpecStage (O-2; the crystallized script IS a stage — EXTEND kinds)
  - crates/orchestrator/src/rra.rs::ReproducibleRunArtifact (G-2; the binding receipt)
reachable_from:
  - crates/cli/src/main.rs (`elai e2e crystallize` + replay via the task_runner tick) [PRODUCTION CLI + task_runner]
  - a #[cfg(test)] or contracts/ caller NEVER satisfies this (R7)
stub_ban:
  - NO crystallized stage without an RRA receipt binding it to a real minting run (a script with no minting-run receipt is refused)
  - NO model call in the replay path (replay is the zero-reasoning execution mode; any LLM invocation in the trace is a FAIL)
  - NO new dispatch runtime (the crystallized script is an O-2 TaskSpec stage through the normal FSM)
  - NO .unwrap()/.expect() on the crystallize/replay path; NO blake2/sha2 (blake3 for the script hash)
frontend_pair:
  a crystallized-stage node rendered on the O-4 canvas (Tier-2; decide at edit, W-4).
observable_effect:
  trigger: "elai e2e crystallize --from-run tests/fixtures/e2e/explore_login.trace && elai e2e replay --stage login"
  expect_ssb: "crystallized_stage_minted"
  expect_artifact: "a crystallized_stage_minted row {script_hash, minting_run_id, rra_hash} + the replay drives the real entrypoint with ZERO model calls in the trace and reproduces the exploration's observable outcome; NEGATIVE: a crystallized stage whose RRA does not bind to a real minting run, or a replay that silently invokes the model, FAILS"

## Wargame (plan-standard unit shape)
EXPECTED_OBSERVATION: crystallizing an E-14 exploration mints one crystallized_stage_minted row {script_hash, minting_run_id, rra_hash}; replaying that stage reproduces the exploration's observable outcome and the trace shows zero model calls.
ON_FAIL: the replay path invokes the model, or the stage has no minting-run receipt (CAUSE: the crystallizer emits a script that still calls the LLM for a step it could not deterministically capture, or the RRA binding is optional). COUNTER-MOVE: assert the emitted script is pure Playwright (no LLM client import) and the RRA receipt is a required output of crystallize; the negative test (LLM-in-replay / receiptless stage) must FAIL.
FORK: if the exploration trace has a genuinely non-deterministic step (a CAPTCHA, a value only the model could infer), crystallization DOES NOT emit that step as deterministic — route it to a marked "agentic-step" stage that re-enters exploration mode, and record the boundary in the RRA; do NOT silently bake a guessed value into the replay script.
RECON_NEEDED: confirm cua replay_env's record->re-score shape (environment.py + replay_env) via the cloned source at edit; confirm the exact O-2 TaskSpecStage enum to extend (avoid a second stage runtime). Settling check: source-read cua environment.py + `grep -n "enum .*Stage" crates/orchestrator/src/task_runner.rs`.
ABORT: if crystallization would produce a replay that is NOT byte-reproducible under the same seed (G-2 contract broken), STOP and flag — a "deterministic" replay that drifts defeats the entire lever.
VERIFY: the load-bearing proof is the off-agent crystallized_stage_minted row + a replay trace asserting zero model calls, driven by the real `elai e2e crystallize`/`replay` entrypoints. Run:
```bash
python .claude/tools/plan_gate_check.py --file docs/thefinalglance/P-plan/P-14_explore_then_crystallize_lane.md   # authoring gate (exit 0)
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/P-plan/P-14_explore_then_crystallize_lane.md   # execution done-gate
```
REDTEAM: surviving-attack seed — a crystallized script that "passes" by re-invoking the model on every step (defeating the cost lever) is caught by the zero-model-calls trace assertion. Attack that would succeed if unpatched: baking a run-specific value (a session token, a timestamp) into the replay so it works once then rots — PATCH: crystallization parameterizes run-specific values as stage inputs and the RRA records them; a hard-coded volatile literal is a stub_ban hit. Deep red-team DEFERRED to execution pre-flight.

## INTEGRATION_SMOKE
An independent agent runs a full E-14 exploration on a HARD multi-step SaaS journey (login + a multi-page form), crystallizes it, then replays the stage N times and asserts byte-identical outcomes under the same seed with zero model calls, and that the RRA `elai replay --verify` re-binds the script to its minting run. Toy single-step crystallize does NOT satisfy R10.

## Grounding / citations
- Source: workflow/reasoning/canvas-computer-use-eval-2026-07-11.md ENTRY 2/10/13, SYNTHESIS 2b/P1 (cua replay_env HIGH-reliability verified; openclaw jsonl-replay C4 correction).
- ELAI ground truth (PLAN-asserted, executor re-greps W-7): E-14, O-2, G-2 RRA, C crystallizer, J-96 per the summary indexes cited.
- CLAUDE.md hard rules: determinism doctrine (byte-reproducible replay under seed); blake3; no .unwrap on prod; no new dependency without confirmation.

## PRODUCTION_HOOK
crates/orchestrator/src/task_runner.rs — the O-2 TaskSpecStage enum gains a crystallized_script kind that the tick dispatches on replay; `elai e2e crystallize`/`elai e2e replay` in crates/cli/src/main.rs are the direct entrypoints. The Crystallizer + RRA binder are wired into both in this changeset.

## Next
Closes SYNTHESIS 2b (exploration->execution crystallization) and directly compounds per-customer reuse (PS-5): a recurring browser/GUI flow becomes a cheap, deterministic, replayable O-2 stage. Pairs with P-15 (both are reuse-artifact minters feeding K-18's case-reuse economics).

End of P-14.
