# O-1: research daemon — generalize Cosmos from designer-briefs to a user-editable typed topic set

STATUS: PLANNED (WAVE 1; PROOF_SPINE: none (product-surface class), owner-acked 2026-07-06; BUILD-on-shipped — the substrate)

- Track: RESEARCH-MACHINE | lever: manifest-generalization | source finding: L1 | grounding: MODERATE (index-derived, PLAN-asserted) | nature: BUILD-on-shipped | effort: M-L | WAVE 1

- Premise: CosmosWorker is already a scheduled research loop (manifest of artifacts, hourly tick, FlashResearch refresh, time-decay to DECAYED, <=1500-token briefing, CosmosStage in the QueryPipeline). It is 90% of a general research daemon with the manifest hard-bound to designer briefs. O-1 generalizes ONLY the manifest surface into a user-editable typed topic set and threads every fetched byte through ELAI's existing trust spine (taint + tool-filter + VCR). No new loop, no new scheduler — that is O-2.

## Steal/build
Generalize the Cosmos manifest to `[[research.topics]]`: `{ id, kind = steal-scan | leaderboard | discovery | monitor, sources[], cadence, confidence_threshold, sink }` — a CLOSED enum of topic kinds, never a free-form "custom topic". The per-topic pipeline is the one TEIKOKU proved live (handoff teikoku-research-machine.md): fetch -> enum-gated classify (untrusted preamble + item fence + per-kind schema; injection canaries as frozen fixtures) -> dup-precheck vs a rejected ledger -> file -> layered digest -> living leaderboard pages via guarded upsert. ELAI's delta over teikoku: route every fetched byte through G-1 taint provenance (untrusted-content -> destructive-sink flows already gate-blocked), apply the F-3 tool-filter to the research role, and record external bytes in the G-2 VCR cassette so a research run is RRA-replayable. Teikoku has canaries; ELAI has canaries + taint + replay.
EXCLUDED: a new scheduler (O-2 owns TaskSpec/triggers); a free-text "custom topic kind" (A3 generic-engine trap — new kinds ship as code); re-authoring the CosmosStage or the briefing compiler (GD-1, EXTEND only); any classifier output that reaches a prompt as free text (A1 — output is closed enum + confidence).

## Existing ground truth (PLAN-asserted; executor re-greps per W-7)
- `cosmos_worker` (~700 LOC), `VerificationMethod::CosmosCheck`, `CosmosStage` in QueryPipeline, `CosmosDrawer`, tick wired in `live_cli`; B-42, C-16 (feature-map "COSMOS ARTIFACT TIER [WIRED]").
- hourly refresh runs off the task_runner `tokio::interval` (C-9); FlashResearch refresh path is B-40.
- G-1 taint provenance (untrusted-content->destructive-sink block), F-3 tool-filter, G-2 VCR cassette/RRA-replay are SHIPPED trust-spine surfaces the daemon must consume, not re-build.
- teikoku enum-gated classify + canary fixtures + confidence-per-sink gate are LIVE-proven (handoff teikoku-research-machine.md: 3 injections neutralized, 0 out-of-enum).

## REVISED scope (W-10 tiered)
- SHIP NOW (Tier-1): the typed topic manifest + the enum-gated classify stage + confidence gate + dup-precheck + the taint/tool-filter/VCR threading, for 2 of the 4 kinds (steal-scan + monitor).
- DEFER (Tier-2): leaderboard + discovery kinds and the living-leaderboard upsert pages (land with O-3's vault, which owns the page surface).

## Integration
- crates/knowledge (or wherever cosmos_worker lives): generalize the manifest loader; add the typed topic schema + enum classify stage; consume G-1/F-3/G-2 at the fetch/classify/sink seams.
- frontend_pair: CosmosDrawer already renders artifacts — extend to render topic-kind + last-run trust tier; do NOT build a new drawer.

## MOVES (wargame)
M1 replace the designer-brief manifest struct with the typed topic schema (closed `kind` enum), keeping the existing loader signature so CosmosStage is unchanged.
EXPECTED_OBSERVATION: a config with `[[research.topics]] kind = "steal-scan"` loads and the daemon enumerates it; an unknown `kind` value is a hard parse-refusal, not a default.
ON_FAIL: unknown kind silently defaults to a catch-all -> the closed-vocabulary invariant is broken -> make the enum deserialize `#[serde(deny_unknown...)]`-strict and add the negative parse test.
M2 thread the fetched bytes through G-1 taint before they reach the classify prompt; classify emits `{class: enum, confidence: f32}`, never free text.
EXPECTED_OBSERVATION: a canary-injection fixture routes to `unsorted` at low confidence and emits a taint-labelled SSB row; zero out-of-enum classifier outputs.
ON_FAIL: a canary reaches the prompt unlabelled -> the fence/taint thread is bypassed -> re-anchor the classify call to consume only the taint-wrapped item, freeze the canary as a gate-manifest fixture (R8).
M3 record the external bytes in the G-2 VCR cassette keyed by run id.
EXPECTED_OBSERVATION: a research run replays byte-identically from its cassette with the network detached (RRA).
ON_FAIL: replay diverges -> a non-recorded network read leaked in -> route the remaining fetch through the VCR client, add the detached-network replay assertion.

## FORK
FORK: if the live cosmos_worker manifest is NOT a swappable struct (hard-inlined designer briefs) -> STOP the in-place generalization, author the topic manifest as a NEW loader feeding the existing CosmosStage via an adapter, and record the adapter seam in the decision doc. Do not fork the worker loop.

## RECON_NEEDED
RECON_NEEDED: the exact cosmos_worker manifest type + whether the classify stage already exists or is teikoku-only -> settle via `grep -rn "cosmos_worker\|CosmosStage\|manifest" .elai_cc/crates` + a codegraph_explore on `cosmos_worker` BEFORE the first edit.
RECON_NEEDED: whether G-1 taint exposes a wrap-untrusted-bytes API the classify path can call -> settle via `grep -rn "taint\|provenance\|untrusted" .elai_cc/crates/*/src` and read the one call site.

## ABORT
ABORT: if generalizing the manifest would require editing the G-2 VCR cassette format or the G-1 taint gate itself (frozen trust-spine surfaces) -> stop and flag; the daemon consumes those gates, it never modifies them.

## WIRING_CONTRACT
produces_symbols:
  - the typed research-topic manifest schema (closed `kind` enum) + its loader
  - the enum-gated classify stage (untrusted-preamble + item-fence + per-kind schema, `{class,confidence}` output)
  - the dup-precheck-vs-rejected-ledger step
consumes_symbols:
  - cosmos_worker / CosmosStage / briefing compiler (EXTEND, do not re-author — GD-1)
  - G-1 taint provenance, F-3 tool-filter, G-2 VCR cassette (consume the shipped gates)
reachable_from:
  - the live cosmos tick in live_cli drives a configured topic end-to-end  [PRODUCTION cosmos daemon tick]
  - a #[cfg(test)] caller NEVER satisfies this (R7)
stub_ban:
  - NO free-text classifier output reaching a prompt (A1)
  - NO open/custom topic kind (A3)
  - NO re-implementation of CosmosStage or the briefing compiler
  - NO fetch path that bypasses G-1 taint or the G-2 cassette
observable_effect:
  trigger: "run the cosmos daemon tick with a steal-scan topic pointed at a canary-injection fixture source"
  expect_ssb: "a classify SSB row carrying {class, confidence, taint_tier}; the canary row lands in unsorted, never a steal sink"
  expect_artifact: "the topic's briefing compiled <=1500 tokens with provenance tiers; NEGATIVE: an out-of-enum classifier output or a canary reaching a steal sink must FAIL the run"

PRODUCTION_HOOK: the cosmos tick in `live_cli` (C-16 tick seam) calls the generalized manifest loader + enum classify stage on every configured topic — the executor names the exact fn at RECON time and wires it in the SAME changeset.

## VERIFY
VERIFY: the auditor runs, off-agent:
`python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/O-plan/O-1_research_daemon_generalized_cosmos.md --run-trigger`
`python .claude/tools/production_caller_check.py --files <changed .rs> --json`   # DARK new pub fn = BLOCK
pass = the classify SSB row landed off-agent with a taint tier, the canary routed to unsorted, and the daemon tick is the production caller (not a test).

INTEGRATION_SMOKE: an independent agent drives the real cosmos tick on a HARD input (a live source whose payload contains a paraphrased prompt-injection, multi-item, long enough to fire the confidence gate + dup-precheck) and asserts the taint-labelled classify rows + the injection neutralized. Toy single-item happy-path does not count (R10).

## REDTEAM
REDTEAM: attack that FAILED against the plan — "smuggle an instruction through a fetched article so it reaches the briefing prompt": FAILS because the classify output is a closed enum + confidence, taint labels thread to every sink, and the canary fixture is frozen in the gate manifest (R8). Attack that SUCCEEDED and its patch — "a new topic kind added as free text to skip the code path": patched by making `kind` a `deny_unknown` enum (M1) so new kinds cannot exist without shipping code (A3).

## Grounding
- design digest workflow/reasoning/elai-personal-research-machine-2026-07-06.md L1 + A1 + A3 + the verbatim-steal list.
- handoff workflow/handoff/teikoku-research-machine.md (the live-proven enum-gated classify + canary + confidence-per-sink patterns).
- feature-map "COSMOS ARTIFACT TIER [WIRED]" (B-42, C-16); G-1/F-3/G-2 trust-spine entries.

## COMPOSES-WITH
CosmosStage/cosmos_worker/briefing-compiler (EXTEND), G-1 taint, F-3 tool-filter, G-2 VCR/RRA, FlashResearch (B-40). Re-implements NONE (GD-1). O-2 adds the scheduler; O-3 adds the vault sink; O-4 adds the canvas surface.

End of O-1.
