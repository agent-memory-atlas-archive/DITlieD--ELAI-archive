# Q-11: novelty-instrument canary + fixed-budget explore reporting + pre-change baselines — make the measuring instrument trustworthy and spend-honest before anything downstream changes

STATUS: PLANNED (WAVE AIDE2-1; instrument+bench trust wave; three sub-units in one file — C6 live-path novelty canary, C10 fixed-dollar explore reporting + do-not-win control arms, BASELINE capture of the research-lane/novelty surfaces. This file ships FIRST in the AIDE² deep-research steal; Q-12/Q-13 gate on its bench being green.)

Authoring provenance: paper-to-plan PHASE 5 over the AIDE² deep-research steal (conjecture engine run 2026-07-15; picks C6 + C10 + baselines, user-sequenced 2026-07-15). Auditors do NOT read the engine run artifact — anchors below are re-greppable in-tree (W-7).

- Track: AIDE2-RESEARCH-INSTRUMENT-ECONOMICS | lever: measuring-instrument-integrity-before-search-improvement | source: AIDE² reconstruction dossier §8 (public/private + fixed budget), §15 (three-layer anti-gaming + the silently-broken-Layer-3 incident), §19 (rejected-ideas noise band) | grounding: STRONG for the gap (premises grep-confirmed in-tree), [EXTERNAL] for every vendor number | nature: BUILD-on-shipped (C6/C10 harden shipped novelty/bench surfaces; baseline is a measurement) | effort: M | WAVE AIDE2-1
- PROOF_SPINE: feeds PS-4 (C10 fixed-dollar explore reporting = spend-normalized selection applied to the explore lane) + PS-3 (C6 canary + the baseline capture harden the novelty/explore measurement the hallucination-delta bench family reports) + PS-6 (the baseline is the cheapest longitudinal pre-stack snapshot, no live lane needed). The C6 canary sub-unit is instrument-discipline class (`none`) on its own; user-acked 2026-07-15 (in-session sequencing ruling: "improving search while the measuring instrument may be silently ineffective is backwards").

- Premise: ELAI already labels research/discovery novelty (`NoveltyClass`, `NoveltyClassifiedEmitter`) and runs an explore bench (`frontier_strategy_bench.py`) with a paired-CI95 within-noise gate. Two integrity holes remain. (1) The novelty path has NO live-path canary — `conjecture_triage.py --self-test` executes three fixture kills (demarcation/ledger/anchor) but never runs a known-answer pair through the LIVE classify+emit code, so a discrimination-dead classifier would emit dead verdicts silently, exactly the failure the AIDE² dossier reports (their statistical anti-gaming layer was live-broken and "had no impact", undetected because aggregate performance hid it) [EXTERNAL https://www.weco.ai/blog/first-evidence-of-recursive-self-improvement]. (2) The explore bench reports raw deltas but NOT per-dollar-at-a-locked-cap, so a policy that merely spends more can look better — the dossier's fixed-$ budget with two named do-not-win strategies (unbounded best-of-N, parallel-widening) exists precisely to reject spend-more wins [EXTERNAL, same URL]. And (3) nothing captures a frozen pre-change baseline of these surfaces, so a later behavior change (Q-12) cannot be attributed. This file closes all three before Q-12 touches the research lane.

## Steal/build

C6 — live-path novelty canary. Add a known-answer canary PAIR (one known-SAME, one known-DISTINCT fixture) executed through the LIVE `NoveltyClass` classify+emit path on every bench/novelty invocation, and the same pair through the `conjecture_triage.py --novelty` fold+verdict code (harness leg). A canary MISS (SAME judged DISTINCT or vice-versa) forces `instrument_strength=FAILED` and SUPPRESSES all novelty verdicts for that run rather than emitting them. Ships default-ON behind an `ELAI_NOVELTY_CANARY_DISABLE` kill-switch (W-2 kill-switch, never a default-OFF enable), so R9 does not bite.

C10 — fixed-dollar explore reporting + do-not-win control arms. The explore/novelty bench reports meaning-cluster coverage and best-verified-outcome PER-DOLLAR at a locked spend-cap, consuming cost straight from the `usage_turn_event` ledger row (never a re-derived number, GD-2). It carries two NAMED control arms — `unbounded_best_of_n` and `parallel_widening` — whose WIN raises a `spend_leak` flag instead of a promotion. This is the control-arm machinery Q-12's bandit is measured against.

BASELINE — pre-change capture. A one-shot `--baseline` run of the existing instruments (`frontier_strategy_bench.py`, `conjecture_triage.py --novelty`, the live novelty path) writes a FROZEN baseline artifact (blake3-identified) recording the current-state numbers, so Q-12's post-change deltas are attributable and the PS-6 longitudinal wire-dark/novelty posture has a t0.

EXCLUDED: any novelty-SURVIVAL re-judge scoring (that is deferred C5 below — it needs temporal-novelty semantics first); any change to what `NoveltyClass` MEANS (3 variants unchanged); any spend-cap that silently rescales the bench (the cap is locked and declared, not adaptive); an LLM judge as the canary oracle (the canary fixtures are deterministic known-answers).

## Existing ground truth (PLAN-asserted; executor re-greps per W-7)

- `NoveltyClass` (3 variants), `NoveltyClassifiedEmitter`, DARK `ELAI_NOVELTY_LABEL_ENABLE` in `.elai_cc/crates/orchestrator/src/novelty.rs`; `grep -icE "survival|re_verify|reverify"` = 0 (no re-judge path today).
- `.claude/tools/conjecture_triage.py` — `--self-test` executes exactly three fixture kills (demarcation/ledger/anchor), NO live-path novelty canary; `--novelty` degrades honestly to `equivalence=UNCERTAIN` + `instrument_strength=WEAK` with no judge.
- `.claude/tools/frontier_strategy_bench.py` — paired non-empty novelty/value deltas with a CI95-lower-bound>0 gate (within-noise = TIE); `grep -icE "per_dollar|usd|spend_cap"` = 0 (no per-dollar/locked-cap normalization today; the Q-10 G3 cost guard fired post-hoc, not in-bench).
- `usage_turn_event` cost row (J-28) in `.elai_cc/crates/orchestrator/src/ssb.rs`; `RoutingEconomicsMeasured` (schema 274) is the most recent per-dollar precedent to mirror, NOT re-implement.
- D-15 `NoveltyClass` honesty label + N-10 `NoveltyScore` consumer + the L-10 explore-arm novelty column are the downstream surfaces the canary protects.

## REVISED scope (W-10 tiered)

- SHIP FIRST (C6 canary): fixtures + the live-path classify+emit canary + the harness `--novelty` canary + the `instrument_strength=FAILED` suppression gate + its `ELAI_NOVELTY_CANARY_DISABLE` kill-switch. In-process; no network.
- SHIP SECOND (C10 reporting): the per-dollar-at-locked-cap columns + the two named control arms + the `spend_leak` flag, cost read from the `usage_turn_event` ledger. Offline bench; no runtime routing change.
- SHIP THIRD (baseline): the `--baseline` capture + the frozen blake3-identified baseline artifact. One-shot; no behavior change.

## Integration

- `crates/orchestrator/src/novelty.rs`: the live-path canary hook on the classify+emit path; the `instrument_strength` field/suppression.
- `.claude/tools/conjecture_triage.py`: the `--novelty` fold+verdict canary leg + a `--self-test-canary` self-test that FAILS if no live-path canary is present (dead-evidence-path screams, wiring R2).
- `.claude/tools/frontier_strategy_bench.py`: the per-dollar columns, locked-cap constraint, named control arms, `spend_leak` flag, and `--baseline` capture.
- `crates/orchestrator/src/ssb.rs`: reuse the `usage_turn_event` cost row; reuse `NoveltyClassifiedEmitter` + an `instrument_strength` FIELD (no new action_type if the field-add is schema-legal; else +1 — RECON at edit, W-9).
- frontend_pair: N/A with justification — Q-11 is offline instrument/bench discipline (auditor tooling + a novelty emitter field); no dashboard surface. Justified.

## MOVES (wargame)

M1 build the C6 canary fixtures + the live-path canary and run it through the LIVE novelty classify+emit path with `ELAI_NOVELTY_LABEL_ENABLE=1`.
EXPECTED_OBSERVATION: on a healthy classifier the known-SAME fixture classifies SAME and the known-DISTINCT fixture DISTINCT; `instrument_strength` is not FAILED and verdicts emit; `conjecture_triage.py --self-test-canary` now reports a present live-path canary.
ON_FAIL: the canary itself never fires on the live path (a shim called it instead) -> instrument is not actually guarded; re-anchor the canary at the real `NoveltyClassifiedEmitter` emit site and re-run, do not accept a test-shim canary (R7).

M2 inject a discrimination-dead classifier (fixture forcing SAME==DISTINCT) and run the canary.
EXPECTED_OBSERVATION: the canary MISSES, `instrument_strength=FAILED` is set, and ALL novelty verdicts for that run are suppressed (absent), not emitted-with-a-warning.
ON_FAIL: dead verdicts still emit alongside the FAILED flag -> the suppression is advisory not blocking; make suppression fail-closed (no verdict rows written when `instrument_strength=FAILED`).

M3 add the C10 per-dollar-at-locked-cap columns to the explore bench, reading cost from the `usage_turn_event` ledger, and run the two named control arms.
EXPECTED_OBSERVATION: the report shows coverage and best-verified-outcome per-dollar at the locked cap; `unbounded_best_of_n` and `parallel_widening` arms are present; when a control arm wins on raw score its report row carries `spend_leak=true` (a flag, not a promotion).
ON_FAIL: cost is re-derived in-bench instead of read from the ledger (numbers drift from GD-2) -> replace the derivation with a direct `usage_turn_event` read; a control-arm win that renders as a promotion -> the spend_leak flag is not wired, wire it before any arm can be declared a winner.

M4 run the `--baseline` capture over all three surfaces and freeze the artifact.
EXPECTED_OBSERVATION: a blake3-identified baseline artifact lands recording the current novelty-canary result, the per-dollar bench numbers, and the `conjecture_triage --novelty` state; re-running `--baseline` byte-reproduces it under the same seed.
ON_FAIL: the baseline is non-reproducible (an unseeded number leaked in) -> localize the non-determinism, seed or drop it; a baseline that cannot reproduce is not a baseline.

## FORK

FORK: if the `usage_turn_event` ledger does not carry a per-arm cost attribution the explore bench can read (only a run-total cost exists), take route B — attribute cost per control arm inside the bench from the per-call token counts already journaled, and record in the decision doc that per-dollar is arm-approximate; do NOT add a new cost ledger (reuse J-28, GD-1).

FORK: if adding `instrument_strength` as a FIELD on the existing emitter is not schema-legal (closed enum / lock), take route B — add at most ONE discriminated `NoveltyCanaryChecked` action_type (+1), re-read the live `SCHEMA_VERSION` at edit (W-9), and bump the action_type_lock golden by one row; do not reserve the bump if the field-add is legal.

## RECON_NEEDED

RECON_NEEDED-1: before building the C6 canary, re-grep `novelty.rs` for any FUNCTIONAL-equivalent canary/self-check already on the emit path (the engine's premise grep proved the NAMED implementation absent, not that no equivalent exists — inspect via `mcp codegraph-elai callers NoveltyClassifiedEmitter` + `.claude/tools/elai.map`); a live equivalent collapses C6 to wiring the suppression only.
RECON_NEEDED-2: re-read `frontier_strategy_bench.py` cost handling to confirm it does not already normalize per-dollar (the Q-10 G3 guard is post-hoc; confirm it is not an in-bench cap) before adding C10 columns.
RECON_NEEDED-3: re-read the live `SCHEMA_VERSION`/`UsageTurnEvent`/`RoutingEconomicsMeasured` sites in `ssb.rs` + `action_type_lock.rs` at edit time to choose reuse-field vs +1 (W-9); never hard-code the baseline.

## ABORT

ABORT: any proposal that (a) lets the canary suppression be bypassed while `instrument_strength=FAILED`, (b) makes the spend-cap adaptive so a spend-more arm can win, (c) re-derives cost instead of reading the `usage_turn_event` ledger, (d) turns the deterministic canary oracle into an LLM judge, or (e) changes what `NoveltyClass` means -> STOP and flag. These break the exact integrity the file exists to add.

## WIRING_CONTRACT
produces_symbols:
  - novelty.rs — the live-path novelty canary + `instrument_strength` FAILED-suppression on the classify+emit path
  - conjecture_triage.py — the `--novelty` canary leg + `--self-test-canary` (fails if no live-path canary present)
  - frontier_strategy_bench.py — per-dollar-at-locked-cap columns, `unbounded_best_of_n` + `parallel_widening` control arms, `spend_leak` flag, `--baseline` capture
  - the frozen blake3-identified baseline artifact
consumes_symbols:
  - novelty.rs::NoveltyClass / NoveltyClassifiedEmitter (harden; do NOT re-implement, GD-1)
  - ssb.rs::usage_turn_event cost row (J-28) — read, never re-derive
  - ssb.rs::RoutingEconomicsMeasured (schema 274) — mirror the per-dollar pattern, do NOT restate
  - frontier_strategy_bench.py paired-CI95 within-noise gate (extend, not replace)
reachable_from:
  - `elai bench <explore/novelty>` with `ELAI_NOVELTY_LABEL_ENABLE=1` [PRODUCTION bench entrypoint] fires the live-path canary
  - `python .claude/tools/frontier_strategy_bench.py` (+ `--baseline`) [PRODUCTION explore-bench CLI]
  - `python .claude/tools/conjecture_triage.py --novelty <artifact>` / `--self-test-canary` [PRODUCTION triage CLI]
  - a #[cfg(test)] caller or a test-shim canary NEVER satisfies reachability (R7)
stub_ban:
  - NO advisory-only suppression — `instrument_strength=FAILED` writes NO verdict rows (fail-closed)
  - NO re-derived cost — per-dollar reads the `usage_turn_event` ledger (GD-2)
  - NO adaptive/rescaling spend-cap; the cap is locked and declared
  - NO LLM judge as the canary oracle; fixtures are deterministic known-answers
  - NO new cost ledger, NO re-implementation of NoveltyClass, NO blake2/sha2 (blake3 for the baseline id)
  - NO default-OFF `ELAI_*_ENABLE` — the canary is a default-ON `ELAI_NOVELTY_CANARY_DISABLE` kill-switch (W-2); R9 N/A, no allowlist row
frontend_pair:
  N/A with justification — offline instrument/bench discipline, no dashboard surface. Justified.
observable_effect:
  trigger: "run `elai bench <explore/novelty>` (ELAI_NOVELTY_LABEL_ENABLE=1) then `python .claude/tools/frontier_strategy_bench.py --baseline`"
  expect_ssb: "the `NoveltyClassifiedEmitter` row carries `instrument_strength`; a healthy run emits verdicts, an injected discrimination-dead run emits ZERO verdict rows with `instrument_strength=FAILED`; per-dollar figures recompute from the `usage_turn_event` cost rows (GD-2)"
  expect_artifact: "a frozen blake3-identified baseline artifact + a bench report with per-dollar columns and the two control-arm rows. NEGATIVE: a control-arm raw-score win that renders as a promotion (missing `spend_leak`), a dead-classifier run that still emits verdicts, or a non-reproducible baseline must FAIL"

PRODUCTION_HOOK: producer C6 live-path canary -> consumer the `NoveltyClassifiedEmitter` classify+emit call site in `novelty.rs` (fires on the `elai bench` novelty path), wired same changeset; producer C10 per-dollar reporting -> consumer `frontier_strategy_bench.py`'s report/arm-selection path, wired same changeset; producer the baseline capture -> consumer the `--baseline` CLI arm, wired same changeset. No producer-only fn and no test-shim caller satisfies the hook.

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/Q-plan/Q-11_novelty_instrument_and_fixed_budget_bench.md --run-trigger
python .claude/tools/conjecture_triage.py --self-test-canary --json     # MUST report a present live-path canary (fails if absent, R2)
rg -c "instrument_strength" .elai_cc/crates/orchestrator/src/novelty.rs  # MUST be >0
rg -c "per_dollar|spend_cap|spend_leak" .claude/tools/frontier_strategy_bench.py  # MUST be >0
test -f workflow/reasoning/Q-11-decisions.md
```

## VERIFY

VERIFY: before declaring Q-11 done, the auditor runs off-agent —
`python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/Q-plan/Q-11_novelty_instrument_and_fixed_budget_bench.md --run-trigger`
`python .claude/tools/production_caller_check.py --files <changed .rs> --json`
`python .claude/tools/frontier_strategy_bench.py --baseline` (re-run twice; the blake3 id must match — reproducible baseline)
plus the M2 injected-dead-classifier run showing ZERO emitted verdict rows. Pass = the wire-checker clean (or a WIRE-DARK[Q-11] row), the canary present on the live path, per-dollar figures journal-recomputable (GD-2), and the fail-closed suppression proven. Cargo-green alone is necessary-not-sufficient.

INTEGRATION_SMOKE: an INDEPENDENT agent drives `elai bench <explore/novelty>` end-to-end on a HARD input (a full multi-candidate novelty run long enough to fire the emit path) with (a) a healthy classifier — verdicts emit + per-dollar columns populate from the ledger, and (b) an injected discrimination-dead classifier — verdicts are SUPPRESSED and `instrument_strength=FAILED`, and a control-arm spend-more win is flagged `spend_leak`, not promoted. Toy single-candidate runs do not satisfy R10.

## REDTEAM

REDTEAM: attack that FAILED — "let a silently-dead novelty classifier keep shipping verdicts because aggregate bench scores still look fine" (the exact AIDE² Layer-3 incident): FAILS because the live-path canary runs every invocation and a miss fail-closes verdict emission, so a dead classifier screams instead of hiding. Attack that FAILED — "win the explore bench by spending more (best-of-N / parallel widening)": FAILS because per-dollar is computed at a LOCKED cap and the two named control arms' wins raise `spend_leak`, never a promotion. Attack that SUCCEEDED against an earlier framing — "make the canary a cheap unit-test that calls the classifier directly": patched by anchoring the canary at the real `NoveltyClassifiedEmitter` emit site (R7) and by `--self-test-canary` failing when no live-path canary is present. Attack that FAILED — "quietly rescale the spend-cap so a favored arm fits": FAILS because the cap is declared and frozen in the bench manifest; a post-freeze change invalidates the run.

## DEFERRED-CONDITIONAL CANDIDATES (recorded here, NOT build units — do not schedule as sessions)

DEFERRED-C5 novelty-survival rate (novelty.rs, NoveltyClass): re-judge DiscoveryCandidate/DISTINCT verdicts later against the UPDATED archive; a verdict losing >=half its claimed distinctness counts as novelty-hacking, landing as a bench column. BLOCKED-ON: a formal TEMPORAL-NOVELTY semantics spec distinguishing novel-when-generated vs later-became-known vs wording-level vs useful-recombination. Without it the survival score PUNISHES success (a genuinely novel finding that others later reach would be scored as hacking). Do not build until those semantics are written down. Would feed PS-3 when unblocked.

DEFERRED-C9 hidden-probe split per Cosmos topic (cosmos_worker.rs, CosmosWorker/CosmosStage): each topic keeps a held-out probe-question set outside the fetch/summarize path; the briefing is scored on probe answer-coverage and the HIDDEN score drives topic decay. BLOCKED-ON: a probe-GENERATION quality story — weak/keyword-shaped probes select for superficial coverage, inverting the intent. Do not build until probe quality is specified and itself measurable. Research-lane class; needs its own user ack if promoted.

## Grounding

- AIDE² reconstruction dossier §8 (public/private score split + fixed-$ budget with the two named do-not-win strategies), §15 (three-layer anti-gaming; the Layer-3 statistical filter was live-broken in AIDE_85 and "had no impact"), §16 (63%->34% reward-hack-rate cut, other domain), §19 (run-to-run noise ~0.02-0.045). ALL vendor-reported, unreleased technical report, no ablation: [EXTERNAL https://www.weco.ai/blog/first-evidence-of-recursive-self-improvement]. These ground the DESIGN need (a canary + spend-honest reporting), never an ELAI benefit number.
- ELAI ground truth: `novelty.rs` (NoveltyClass/NoveltyClassifiedEmitter, no survival path), `conjecture_triage.py` (--self-test 3 fixture kills, no live canary; --novelty honest WEAK degrade), `frontier_strategy_bench.py` (paired CI95 gate, no per-dollar), `ssb.rs` usage_turn_event + RoutingEconomicsMeasured. Re-greppable in-tree (W-7).
- CLAUDE.md hard rules: blake3 for the baseline id; no .unwrap/.expect on prod; scoped tests; GD-2 journal-recomputable numbers; W-2 kill-switch; KD-1/HD-1 [EXTERNAL] tags never asserted as ELAI numbers.

## COMPOSES-WITH

D-15 NoveltyClass honesty label, N-10 NoveltyScore, the L-10 explore-arm novelty column, J-28 usage_turn_event, the schema-274 RoutingEconomicsMeasured per-dollar pattern, `frontier_strategy_bench.py`, `conjecture_triage.py`. Re-implements NONE (GD-1). Gates Q-12: the fixed-budget bench + baseline captured here are Q-12's control-arm apparatus and its attribution t0.

## Open questions / risks

- The canary fixtures must be genuinely discriminating yet stable across `NoveltyClass` revisions — decide the fixture pair at authoring against the live 3-variant enum (RECON-1).
- Per-arm cost attribution may be approximate if J-28 journals only run-total cost (FORK-1); the decision doc records the approximation honestly.

## Next

Closes the instrument-integrity + spend-honesty holes and captures the pre-change baseline. Pairs directly with Q-12 (which is gated on this bench being green) and with Q-13 (the receipt-binding trust sibling). Unblocks nothing in C5/C9 — those stay deferred until their scoring semantics land.

End of Q-11.
