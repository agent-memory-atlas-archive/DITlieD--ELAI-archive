# J-28: token ledger — usage unification, the per-turn usage event, and the rollup core (the data half of the token-consumption window)

STATUS: PLANNED (WAVE GSW-1; .elai_cc runtime + CLI consumer; BUILD+EXTENSION — fixes the local-lane capture bug, normalizes cloud/local usage semantics, ships ONE uniform per-turn usage row both lanes emit, reprices the spend accumulator off the real catalog, and gives the CLI a dollar-aware report; the dashboard window is J-29)

- Track J (repo-steal extension) | lever PROVABLE + EFFICIENT | grounding STRONG (ELAI anchors
  grep-confirmed live 2026-06-12 THIS session; goose source via goose-main-INDEX.md, reviewed
  source = MEDIUM; reference digest = WEAK, ideas only) | nature BUILD+EXTENSION | effort M-L |
  WAVE GSW-1

- Premise: the user's #1 deliverable is a COMPLETE token-consumption window — per model, per
  task, per session, real-time, cloud + local. The audit (workflow/papers/
  elai-token-usage-current-state.md, X1 2026-06-12) shows the substrate is SHIPPED but scattered
  and partly wrong: streaming-local turns lose input_tokens (the SSE consumer only reads
  MessageDelta), local and cloud input_tokens mean DIFFERENT things (full-prompt vs suffix-only),
  the spend accumulator prices every model at one hardcoded rate, and no existing row carries
  enough keys to roll up per-(model,task,session). J-28 is the data layer that makes the window
  possible; it is ACTIVATION + NORMALIZATION + AGGREGATION, not from-scratch. Steal file:
  workflow/reasoning/steal-goose-factory-2026-06-12.md (S-01..S-16, S-31/S-32, S-50).

## Steal/build
T1 STREAMING USAGE ASSEMBLY FIX (S-31, the keystone). anthropic.rs::observe_event reacts ONLY to
  StreamEvent::MessageDelta (anthropic.rs:1313-1316, grep-confirmed); the sidecar's streaming
  message_start carries input_tokens (server.py:4728-4733, audit) which never reaches
  latest_usage (anthropic.rs:1275), so streaming-local usage records input_tokens=0 (audit L-1).
  Fix: accumulate MessageStart usage into latest_usage and merge with the MessageDelta
  output-side (the Anthropic spec splits usage across the two events by design). VERIFY pre-gate
  at edit: confirm the live assembly path on both anchors before coding (audit flags the
  runtime-layer (tokens_in, tokens_out) tuple as a second read path — map both).
T2 NORMALIZED USAGE + LANE SEMANTICS TAG (S-32). server.py:4959-4963 (audit) documents the
  intentional divergence: sidecar input_tokens = FULL prompt; Anthropic spec = uncached suffix
  only. Ship NormalizedUsage carrying {raw TokenUsage, lane, semantics tag, normalized fields}.
  Two candidate fixes, decided at edit with the cross-language pair rule (server.py +
  runtime_client.rs move together): (a) sidecar emits spec-compliant suffix-only + cache fields,
  (b) client-side normalization keyed by lane tag. Either way: cloud sums and local sums are
  NEVER naively added — every aggregate is per-lane or explicitly dual-labeled.
T3 THE PER-TURN USAGE EVENT (S-08 + S-16 + S-50). One new SSB action_type UsageTurnEvent emitted
  by BOTH lanes at their existing post-turn seams (cloud: the CloudSpend emit site,
  grounding_client.rs:950-969 audit; local: the BackendCostAccrued emit site,
  runtime_orchestrator.rs:331-395, grep-confirmed both arms). Payload keys: identity (String,
  default "local" — S-50: a field, never multi-user infrastructure), lane, backend, model,
  task_id, session_id, the four token counters, est_cost_micros, elapsed_ms (S-16; the sidecar
  already returns elapsed_ms, server.py:4984 audit, ELAI drops it), semantics tag. Pattern-2
  single-variant, 3-arm Kind: TurnRecorded / EstimateApplied / NormalizationApplied. EXISTING
  CloudSpend + BackendCostAccrued rows are KEPT untouched (no removal); UsageTurnEvent is the
  uniform join they cannot provide (BackendCostAccruedPayload has no model/task/session keys —
  ssb.rs:18477, audit).
T4 SPEND ACCUMULATOR REPRICING (S-05, audit C-1). SpendAccumulator::for_model() matches ONLY the
  literal "claude-sonnet-4-6" and silently prices every other id at sonnet rates
  (cloud_mode.rs:683-691, grep-confirmed). Fix: consult pricing_for_model/the I-51 ModelCatalog
  (pricing.rs:82 + estimate_cost_usd_with_pricing:143, grep-confirmed; audit notes pricing.rs is
  already a VIEW over models.toml) with family-alias normalization on lookup; unknown model =
  explicit None-cost (surfaced as "no rate"), never a silent wrong rate.
T5 SESSION + ROLLUP READ-SIDE (S-04, audit C-2). A rollup module (usage_ledger) folding
  UsageTurnEvent rows by (identity, model, task_id, session_id, lane) with Option-semantics
  accumulation (S-02: None+None=None — adopt goose's sum_optionals idea in THIS layer only; the
  shipped u32 TokenUsage at usage.rs:15 is NOT churned). Closes "session total" (today task_usd =
  most-recent row only, audit 2.2).
T6 ESTIMATOR FALLBACK (S-03, audit L-3). When a backend reports no/partial usage, count
  client-side (per-lane tokenizer; blake3-keyed LRU cache — goose token_counter.rs:22-73 idea)
  and emit Kind=EstimateApplied. Estimates are LABELED approximations: the per-lane tokenizer
  differs from the provider's, and the rollup keeps estimated counters distinguishable.
T7 COMPACTION ACCOUNTING RULE (S-06). Summarization output tokens become new input-context
  tokens; cumulative fields keep growing (goose reply_parts.rs:529-539 idea). Lands as a rule in
  the ledger accumulation; recorded as a rider on H-50's ContextPolicy (composition, not a
  re-author).
T8 CLI SURFACE — the in-plan consumer (S-11, audit C-6). format_cost_report (resume.rs:328,
  grep-confirmed; printed via live_cli.rs:2683 audit) gains a dollar line via estimate_cost_usd
  (pricing.rs:163) + per-lane token split; plus a small read-side `elai usage` report verb
  rendering the T5 rollup (per-model / per-task / per-session, cloud and local separately
  labeled). Tokens-only display where no rate exists (local lane; S-12 rule).
RIDERS (recorded, not tasks): S-15 OTEL counters = E-29 OTLP exporter path (ledger
  observability verdict); S-06 also referenced from H-50; S-36 XML tool-call fallback = E-35;
  S-38 RunPod real-cost attribution = future pod-lifecycle rider.

## Existing ground truth (grep-confirmed live 2026-06-12 this session unless marked audit)
- crates/core/src/usage.rs:15 TokenUsage (4 counters, u32) + :34 UsageTracker
  (cumulative+latest_turn) — KEEP, compose.
- crates/providers/src/providers/anthropic.rs:1275 latest_usage field; :1313-1316 observe_event
  captures usage from MessageDelta ONLY (the L-1 bug-class anchor).
- crates/orchestrator/src/cloud_mode.rs:47 DEFAULT_CLOUD_MODEL_ID; :683-691 for_model sonnet-only
  match (the C-1 anchor).
- crates/providers/src/pricing.rs:82 pricing_for_model; :143 estimate_cost_usd_with_pricing; :163
  estimate_cost_usd.
- crates/cli/src/resume.rs:328 format_cost_report (tokens-only today).
- crates/orchestrator/src/runtime_orchestrator.rs:331-395 BackendCostAccrued emission, paid arm +
  local est_cost_micros=0 arm (the J-28 local emit seam).
- AUDIT-VERIFIED (X1 2026-06-12; re-grep at edit, W-7): grounding_client.rs:950-969 CloudSpend
  emit site; server.py:4728-4733 message_start usage / :4856-4863 message_delta usage /
  :4959-4968 divergence comment + non-streaming block / :4984 elapsed_ms;
  context_budget.rs:795 CumulativeTokenTracker (cloud-path-only population, audit L-4);
  observability/pricing_rollup.rs DARK no consumer (J-29 activates); ssb.rs:9190 cloud_spend /
  :9527 backend_cost_accrued / :11522 CloudSpendPayload / :18477 BackendCostAccruedPayload.
- Goose ground truth (W-3, via docs/.source/goose-main-INDEX.md): token_usage.rs:26-107 Option
  counters + Add; usage_estimator.rs:9-47 ensure-usage fallback; token_counter.rs:22-171 cache +
  schema cost model; reply_parts.rs:499-575 accumulation + pricing; session_manager.rs:56-89
  dual fields.
- CORRECTIONS (RED-TEAM grep-verified live 2026-07-09; the 2026-06-12 anchors above drifted, design
  conclusions all survive — re-grep at edit per W-7/W-9 regardless): cloud_mode.rs for_model is at
  :761-769 (not 683-691; for_claude_sonnet_4_6 at :741-754); pricing.rs pricing_for_model :91,
  estimate_cost_usd_with_pricing :154, estimate_cost_usd :174 (consistent +11 vs the 82/143/163
  cited); ssb.rs was MODULE-SPLIT — cloud_spend/backend_cost_accrued action-types now in
  ssb/action_type.rs, CloudSpendPayload/BackendCostAccruedPayload in ssb/payloads.rs (the
  9190/9527/11522/18477 anchors are pre-split; BackendCostAccruedPayload STILL has no
  model/task/session key, ssb/payloads.rs:6867-6882, so the T3 rationale holds); server.py
  message_start :5040-5044, message_delta :5181-5185, elapsed_ms ~:5216-5311 (not 4728/4856/4984).
  EXACT (no drift, spot-verified): anthropic.rs:1275 + :1313-1316, grounding_client.rs:950-969,
  resume.rs:328, context_budget.rs:795, usage.rs:15/34, runtime_orchestrator.rs:331-395 (~336-390).

## REVISED scope (W-10 tiers)
SHIP NOW Tier-1 (in-process, fixture-driven): T1 (fixture SSE streams), T2 (both fix candidates
  are fixture-testable; the server.py leg lands as code but its LIVE confirmation is gated), T3
  emission at both seams, T4 repricing, T5 rollup, T6 estimator, T7 rule, T8 CLI.
DEFER Tier-2 (remote-sidecar-gated per W-10 + sidecar-protocol): the LIVE streaming confirmation
  runs (boot the remote sidecar, drive a real local turn, assert the UsageTurnEvent row carries
  non-zero input_tokens + the lane tag). Flag before any boot (HARDWARE-HEAVY house rule).

## Integration
- crates/providers/src/providers/anthropic.rs (~40 LOC: MessageStart usage accumulation).
- python/elai_inference/server.py + crates/orchestrator/src/runtime_client.rs (T2 candidate (a):
  cross-lang pair ships together, cross_lang_diff applies).
- NEW crates/orchestrator/src/usage_ledger.rs (~300 LOC: NormalizedUsage, lane tag, rollup,
  estimator seam, Option-semantics accumulation).
- crates/orchestrator/src/ssb.rs + tests/action_type_lock.rs (+UsageTurnEvent, full 6-part
  variant discipline, 9-tuple golden-lock lockstep).
  NOTE (REDTEAM patch 2): ssb.rs is a THIN AGGREGATOR now (mod action_type; mod payloads; mod
  store), ~1622 lines. The new SsbActionType::UsageTurnEvent arm lands in ssb/action_type.rs; the
  UsageTurnEventPayload struct + SsbPayload::UsageTurnEvent arm land in ssb/payloads.rs; only the
  SSB_ACTION_TYPES registration entry + the SCHEMA_VERSION derivation + the golden-lock stay in
  ssb.rs (mirror the RecurrenceHitRateMeasured / GateTaxMeasured precedent entries). The
  UsageTurnEvent grep vs ssb.rs still hits the registration entry, so the DETERMINISTIC_CHECK holds.
- grounding_client.rs + runtime_orchestrator.rs (emit UsageTurnEvent beside the existing rows).
- crates/orchestrator/src/cloud_mode.rs (~30 LOC: for_model -> catalog lookup).
- crates/cli/src/resume.rs + live_cli.rs + the small `elai usage` verb (~120 LOC).
- frontend_pair: J-29 is the dashboard pair (one wave later); within THIS unit the CLI report is
  the consumer (W-4 satisfied by the J-29 hand-off being explicit, not silent).

## WIRING_CONTRACT
produces_symbols:
- UsageTurnEvent SSB action_type + UsageTurnEventPayload (identity/lane/backend/model/task/
  session/4 counters/est_cost_micros/elapsed_ms/semantics tag; Kind: TurnRecorded /
  EstimateApplied / NormalizationApplied)
- usage_ledger module (NormalizedUsage + lane semantics tag + rollup by
  (identity,model,task,session,lane) + estimator fallback)
- SpendAccumulator catalog-priced rates (for_model consults pricing_for_model/ModelCatalog)
- CLI: dollar-aware cost report + `elai usage` rollup verb
consumes_symbols:
- crates/core/src/usage.rs::UsageTracker (the per-turn capture — never re-implemented)
- crates/providers/src/pricing.rs::pricing_for_model + estimate_cost_usd_with_pricing (I-51
  surface; the ONLY rate source)
- grounding_client.rs CloudSpend emit seam + runtime_orchestrator.rs BackendCostAccrued seam
  (existing rows KEPT; UsageTurnEvent emitted beside them)
- context_budget.rs::CumulativeTokenTracker (read-composed; its L-4 cloud-only population is
  NOT fixed here — honest residue)
reachable_from:
- the production cloud send path (task_runner -> CloudGroundingClient::send_message ->
  grounding_client.rs:950 seam)  [PRODUCTION cloud turn]
- RuntimeOrchestrator::try_complete (runtime_orchestrator.rs:331 seam)  [PRODUCTION local turn]
- `elai usage` / the live CLI cost print (resume.rs:328 path)  [PRODUCTION CLI]
- contracts/-only callers never satisfy this (R7)
stub_ban:
- NO naive cloud+local summation anywhere (every aggregate per-lane or dual-labeled)
- NO silent estimation (estimator fires => Kind=EstimateApplied, never TurnRecorded)
- NO new hardcoded per-model rate tables (catalog lookup only; unknown model = explicit None)
- NO mid-stream SSB writes (the per-turn row is the only journal write; live ticks are J-29's
  in-memory gauge)
- NO key/secret material in any payload (G-9)
- NO .unwrap()/.expect() on prod paths; NO blake2/sha2 (estimator cache = blake3)
- NO new env gate (ships ON — S-46 DO-NOT-COPY: goose gates cost display off by default)
frontend_pair:
  J-29 UsageWindow (next wave, explicit hand-off); this unit's consumer is the CLI report.
  Justified.
observable_effect:
  trigger: "one cloud fixture turn through the real task path AND one local fixture turn through
    RuntimeOrchestrator::try_complete, then `elai usage`"
  expect_ssb: "two UsageTurnEvent rows (one per lane) with correct lane tags + non-zero
    input_tokens on the streaming-local row (the L-1 fix proven); a haiku-model cloud fixture's
    cost derived from haiku catalog rates, not sonnet defaults (the C-1 fix proven)"
  expect_artifact: "`elai usage` renders the rollup grouped by (model, task, session) with cloud
    and local sums separately labeled. NEGATIVE (R2): a fixture stream with no usage in
    message_delta yields Kind=EstimateApplied (never silent zeros); a contract test summing
    cloud+local without the lane tag FAILS; for_model('claude-haiku-4-5') returning sonnet rates
    FAILS."

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/J-plan/J-28_token_ledger_usage_unification.md --run-trigger
rg "UsageTurnEvent" .elai_cc/crates/orchestrator/src/ssb.rs                          # variant registered
rg "MessageStart" .elai_cc/crates/providers/src/providers/anthropic.rs               # input-side capture present (T1)
# T4 (REDTEAM patch 1): the load-bearing proof is the negative unit test (haiku->haiku rate,
# unknown->None; VERIFY-1). This grep is the lexical corroborant only — assertable (exit 0 iff the
# catalog lookup is present where it was 0 matches pre-fix); the old anchored for_claude_sonnet_4_6()
# grep was not pass/fail-decidable (never matched the real ;-terminated call site) and is dropped:
test "$(rg -c 'pricing_for_model' .elai_cc/crates/orchestrator/src/cloud_mode.rs)" -ge 1
test -f workflow/reasoning/J-28-decisions.md
```

## Grounding
- ELAI anchors: grep-confirmed live 2026-06-12 this session (list in Existing ground truth);
  audit-tagged anchors re-grepped at edit (W-7). Audit doc: workflow/papers/
  elai-token-usage-current-state.md.
- Goose: docs/.source/goose-main clone (Apache-2.0, reviewed source = MEDIUM) via
  goose-main-INDEX.md; ideas only, clean-room names (UsageTurnEvent / NormalizedUsage /
  usage_ledger are ELAI-native). Reference digest: workflow/papers/
  reference-cli-feature-digest.md (WEAK by its own header; nominates features, grounds nothing).
- SSB baseline: the 2026-06-12 snapshot (SCHEMA_VERSION 234 / TOTAL_VARIANTS 543) is STALE —
  RED-TEAM verified live 2026-07-09 the value is SCHEMA_VERSION 271 (ssb.rs derives it as
  -309 + SSB_ACTION_TYPES.len(); HEAD commit d1eb94a shipped 271) after 37 versions of drift.
  J-28 = live-baseline+1, computed FROM THE GREP at edit (FORK-2), NEVER hard-coded (W-9; the J-0
  constants table logs five stale snapshots in three days — this is the sixth).
- No new dependency. No arXiv citations (repo steal, not paper steal).
- Doctrine that bites: W-1/W-9/W-10, GD-1/GD-2/GD-5, HD-1 (no goose-figure claims), R7, G-9,
  J-22 (if landed first, the new payload ships with its field-shape snapshot).

## COMPOSES-WITH
H-9 cost cockpit (extends: CloudSpend stays, repricing fixes its ledger), I-51 ModelCatalog (the
rate source), phase-3J sidecar (T2 pair), H-50 (T7 rider), C-14 TokenUsage (untouched), J-22
payload lock (snapshot on the new payload), J-29 (the window this feeds), G-9 redaction.
Restates none; CloudSpend / BackendCostAccrued / UsageTracker / pricing.rs are consumed, never
re-implemented.

## Open questions / risks
- The runtime layer's (tokens_in, tokens_out) tuple vs the SSE-layer Usage (audit 1.4): map BOTH
  read paths at edit before choosing where NormalizedUsage is assembled.
- T2 fix choice (sidecar-side vs client-side): sidecar-side is spec-cleaner but touches the
  remote deployment; decide at edit, cross-lang pair either way.
- Windows file semantics for nothing here (key store is J-30); the ledger writes only SSB.
- elapsed_ms for the CLOUD lane needs a wall-clock at the send seam (sidecar already returns it).

## Wargame (refine-before-execute injection, 2026-07-09 s-j28-refine; plan-standard.md unit shape)
The T1-T8 moves above are the actions; each carries an EXPECTED_OBSERVATION (what the executor
sees on success) and an ON_FAIL (the likely failure, the cause it signals, the counter-move).
Grounding is unchanged (anchors grep-confirmed 2026-06-12, re-grep at edit per W-7/W-9); this
section adds the wargame markers the plan lacked, nothing about the design changed.

EXPECTED_OBSERVATION T1: with MessageStart usage accumulated into latest_usage, a fixture SSE
  stream (message_start input_tokens=N, message_delta output_tokens=M) driven through observe_event
  leaves latest_usage = {input_tokens=N (non-zero), output_tokens=M}; a unit test over the fixture
  asserts input_tokens != 0 (the L-1 fix).
ON_FAIL T1: latest_usage.input_tokens stays 0 -> the MessageStart arm never merged, OR the merge
  clobbered the MessageDelta output side. CAUSE: observe_event still matches MessageDelta only, or
  the two arms overwrite instead of accumulate. COUNTER-MOVE: re-grep anthropic.rs:1313 for the new
  MessageStart arm; assert MessageStart populates the input side and MessageDelta merges the output
  side with neither clobbering the other; also map the runtime (tokens_in,tokens_out) tuple read
  path (audit 1.4) so the fix lands on the path production actually reads.
EXPECTED_OBSERVATION T2: NormalizedUsage carries {raw TokenUsage, lane, semantics_tag, normalized};
  a cloud row and a local row with identical numeric input_tokens carry DIFFERENT semantics tags
  (full-prompt vs suffix-only) and no aggregate path adds them without a lane label.
ON_FAIL T2: a cloud+local input_tokens sum compiles and runs unlabeled -> the stub_ban (no naive
  summation) is violated. CAUSE: the rollup folded across lanes. COUNTER-MOVE: the NEGATIVE contract
  test "sum cloud+local without the lane tag" must FAIL (already named in observable_effect); wire it
  so an unlabeled cross-lane aggregate cannot type-check or cannot pass the test.
EXPECTED_OBSERVATION T3: one UsageTurnEvent SSB row emitted at EACH lane seam (cloud
  grounding_client.rs:950, local runtime_orchestrator.rs:331); the action_type golden lock
  regenerates by EXACTLY +1 variant and SCHEMA_VERSION by +1 off the live grep (never hardcoded).
ON_FAIL T3: only one lane emits, or the golden diff is not exactly +1. CAUSE: the emit call was
  added at one seam only, or a hardcoded schema constant drifted (the J-0 table logged 5 stale
  snapshots in 3 days). COUNTER-MOVE: grep BOTH seams for the emit call; regen golden and diff
  exactly +1; recompute the constant from the live grep, do not carry a snapshot.
EXPECTED_OBSERVATION T4: for_model('claude-haiku-4-5') returns haiku catalog rates (not sonnet);
  an unknown model id returns explicit None-cost surfaced as "no rate", never a silent sonnet
  default.
ON_FAIL T4: haiku is still priced at sonnet rates. CAUSE: for_model still literal-matches
  "claude-sonnet-4-6" only. COUNTER-MOVE: route for_model through pricing_for_model/the I-51
  ModelCatalog with family-alias normalization; the negative test (haiku->sonnet rate) FAILS before
  and PASSES after; unknown-model returns None, asserted.
EXPECTED_OBSERVATION T5: usage_ledger folds UsageTurnEvent rows by (identity,model,task,session,
  lane) with Option-semantics (None+None=None); a multi-row fixture yields a session total that is
  the SUM across rows, not the most-recent row (closes C-2).
ON_FAIL T5: session total equals the last row only. CAUSE: the fold returned latest instead of
  accumulating. COUNTER-MOVE: multi-row fixture; assert session_sum > any single row and ==
  hand-computed total; assert None+None stays None (not 0).
EXPECTED_OBSERVATION T6: a fixture stream with no usage in message_delta yields Kind=EstimateApplied
  (labeled), never a TurnRecorded row with all-zero counters.
ON_FAIL T6: a zero-usage stream produces TurnRecorded(0,0) -> a silent zero (R2 dead-evidence-path
  violation, the worst class). CAUSE: the estimator seam never fired / never set the Kind.
  COUNTER-MOVE: estimator fires => Kind=EstimateApplied; the NEGATIVE test asserts NO TurnRecorded
  carries an all-zero counter set.
EXPECTED_OBSERVATION T7: after a summarization turn, cumulative input-context tokens grow (summary
  output becomes next-turn input); the rule lands as an accumulation rider (a rider on H-50
  ContextPolicy), no churn of usage.rs.
ON_FAIL T7: cumulative fields reset or double-count across a compaction turn. CAUSE: the rule was
  not applied at the accumulation seam. COUNTER-MOVE: a fixture with a compaction turn asserts
  monotonic cumulative growth across the boundary.
EXPECTED_OBSERVATION T8: format_cost_report prints a dollar line (via estimate_cost_usd) + a
  per-lane token split; the `elai usage` verb renders the T5 rollup grouped by (model,task,session)
  with cloud and local separately labeled; tokens-only where no rate exists.
ON_FAIL T8: the cost line shows $0 / a wrong rate, or `elai usage` is unreachable. CAUSE: the CLI
  consumer never reaches pricing.rs / the rollup (a wire-dark verb). COUNTER-MOVE:
  production_caller_check on the new verb (0 DARK); drive `elai usage` on a fixture db and assert
  non-empty grouped output with both lane labels.

FORK-1 (T2 fix locus, settled at edit): TRIGGER — if mapping the runtime (tokens_in,tokens_out)
  tuple vs the SSE-layer Usage shows the sidecar is the single clean assembly point AND the remote
  deployment can be re-rsynced this session, take ROUTE A (sidecar emits spec-compliant suffix-only
  + cache fields; cross-lang pair server.py + runtime_client.rs move together, cross_lang_diff
  applies). ELSE take ROUTE B (client-side normalization keyed by the lane tag; no remote touch).
  DEFAULT ROUTE B — it is Tier-1 shippable with zero remote dependency; choose A only if the audit's
  spec-cleanliness win is explicitly wanted and a Tier-2 remote confirmation is already scheduled.
FORK-2 (schema constant, settled at edit): TRIGGER — if re-grep shows SCHEMA_VERSION != 234 or
  TOTAL_VARIANTS != 543, do NOT hardcode +1; recompute baseline+1 from the live grep. ELSE proceed
  234->235 / 543->544. (RED-TEAM: as of 2026-07-09 the live value IS 271/580-ish, 37 versions past
  the 234 the Grounding header still shows — this fork fires, recompute from the grep.)
FORK-3 (T7 compaction-rule home, settled at edit — REDTEAM patch 3): TRIGGER — if H-50's
  context_policy.rs already threads per-turn token deltas at the seam H-50 uses, EXTEND the
  monotonic-growth rule THERE (composition, T7's "rider on ContextPolicy" framing) and name that
  file:line in the decisions doc. ELSE land the growth check directly in usage_ledger's fold (T5's
  module). DO NOT do both — two independent cumulative-growth numbers can diverge per lane with no
  test catching it. Whichever home is chosen, the INTEGRATION_SMOKE cross-check below is mandatory.

RECON_NEEDED-1: the live streaming assembly path on BOTH read anchors (anthropic.rs:1275
  latest_usage AND the runtime (tokens_in,tokens_out) tuple, audit 1.4) — SETTLE by a scoped read
  of anthropic.rs:1270-1320 + a grep for the tuple read path BEFORE choosing where NormalizedUsage
  is assembled (T1/T2 pre-gate). Uncheckable-at-plan-time -> this is the landmine with its tripline.
RECON_NEEDED-2: the two emit seams still live at grounding_client.rs:950-969 and
  runtime_orchestrator.rs:331-395 — SETTLE by re-grep at edit (audit-verified 2026-06-12; W-7
  mandates re-grep, line numbers drift).
RECON_NEEDED-3: the live SSB baseline (SCHEMA_VERSION at ssb.rs:3096, TOTAL_VARIANTS at
  action_type_lock.rs:676) — SETTLE by grep at edit; never hardcode (W-9).
RECON_NEEDED-4: elapsed_ms for the CLOUD lane needs a wall-clock at the send seam — SETTLE by
  confirming the sidecar returns elapsed_ms (server.py:4984) and locating the cloud send-seam
  timestamp source before adding the field.

ABORT-1: if the T2 sidecar route (A) would require BOOTING the remote sidecar to satisfy a Tier-1
  check — STOP. Tier-1 is in-process fixture-only (W-10); the LIVE streaming confirmation is Tier-2,
  remote-gated, and flagged before any boot (HARDWARE-HEAVY house rule). Never boot mid-build.
ABORT-2: if regenerating the action_type golden lock changes MORE than +1 variant, or touches an
  unrelated action_type — STOP; a wider golden diff is an unplanned schema change (ABORT-4 class,
  K-4 precedent). Surface, do not absorb.
ABORT-3: if any required aggregate cannot be expressed without summing cloud+local UNLABELED —
  STOP and re-scope T5; the no-naive-summation stub_ban is load-bearing.
ABORT-4: if proving the number would need real `elai run` dogfood rows — STOP. J-28 is a
  fixture-driven data-layer build; the LIVE aggregate is usage-gated (same posture as PS-4/PS-5).
  Ship the plumbing + the fixture proof; never fabricate a measured aggregate.

VERIFY-1 (auditor off-agent, sole test runner; each cargo ONE at a time on this OOM-prone box, -j 1,
  never two concurrent): the wiring-contract run-trigger + the DETERMINISTIC_CHECK block. PASS = two
  UsageTurnEvent SSB rows land in a fresh db driven by the REAL seams (sqlite3-confirmed, artifact
  the executor did not author), non-zero input_tokens on the streaming-local row, haiku cost from
  haiku rates.
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/J-plan/J-28_token_ledger_usage_unification.md --run-trigger
cargo test -p elai-orchestrator --lib usage_ledger -j 1
cargo test -p elai-orchestrator --test action_type_lock -j 1
python .claude/tools/production_caller_check.py --files <changed .rs>
python .claude/tools/maintainability_check.py --files <changed .rs> --base HEAD
```
VERIFY-2 (cross-lang, only if FORK-1 ROUTE A): `python .claude/tools/cross_lang_diff.py` shows
  server.py + runtime_client.rs moved together (no drift), else BLOCK.

INTEGRATION_SMOKE (after T3+T5 land, and again before DONE — R10): an INDEPENDENT auditor drives
  BOTH real entrypoints on a HARD input — one cloud fixture turn through the production cloud send
  path (task_runner -> CloudGroundingClient::send_message -> grounding_client.rs:950) AND one local
  fixture turn through RuntimeOrchestrator::try_complete (runtime_orchestrator.rs:331), MULTI-TURN
  so the T7 compaction rule and the T5 cross-row rollup both fire, then `elai usage`. PASS = two
  lane-tagged UsageTurnEvent rows + a grouped rollup with cloud and local SEPARATELY labeled + the
  streaming-local row's input_tokens non-zero (L-1 proven end-to-end, not just per-unit). A
  single-turn happy-path smoke does NOT satisfy this.
  ALSO (REDTEAM patch 3, T7/T5 seam): the multi-turn run MUST cross a compaction boundary, and the
  smoke asserts T7's cumulative-growth number EQUALS T5's rollup session-sum across that boundary on
  BOTH lanes (per lane) — catches the FORK-3 divergence where the compaction rule and the rollup
  count from two different homes.

REDTEAM (non-author adversarial-auditor mode=plan, s-j28-refine 2026-07-09; grep-verified every
  anchor against live .elai_cc; immutable record). VERDICT: PLAN-BLOCK on v1-of-refine -> patched
  below to PLAN-PASS.
  ATTACKS THAT FAILED (plan already defends, cited): T3-emits-one-seam-only killed by move T3's
    observation/on-fail pair ("grep BOTH seams") + INTEGRATION_SMOKE (both turns);
    cloud+local-silent-sum killed by stub_ban + observable_effect NEGATIVE + ON_FAIL T2;
    silent-TurnRecorded(0,0) killed by T6 + its NEGATIVE test; haiku-priced-at-sonnet killed by T4
    (and the pricing_for_model grep is a genuine before/after flip — verified 0 current matches in
    cloud_mode.rs, not a tautology); schema-constant-staleness neutralized by FORK-2 (re-grep
    mandated, never hardcode); T2-forces-remote-boot killed by ABORT-1 + FORK-1 default ROUTE B;
    golden-drift-wider-than-+1 killed by ABORT-2.
  ATTACKS THAT SUCCEEDED -> PATCHED (this session):
    (1) DETERMINISTIC_CHECK T4 line `rg -c "for_claude_sonnet_4_6\(\)$"` was not pass/fail-decidable
        (the anchored `()` pattern does not even match the real call site at cloud_mode.rs:765 which
        ends in `;`; count stays 0 pass-or-fail). PATCH: replaced with an assertable one-call-site
        check + reliance on the T4 negative unit test. (Applied in DETERMINISTIC_CHECK below.)
    (2) Integration/WIRING_CONTRACT did not name the ssb/ module split — ssb.rs is now a thin
        aggregator (mod action_type; mod payloads; mod store), 1622 lines; the new SsbActionType arm
        lands in ssb/action_type.rs, the UsageTurnEventPayload struct + SsbPayload arm in
        ssb/payloads.rs, only the SSB_ACTION_TYPES registration entry + SCHEMA_VERSION derivation +
        golden-lock stay in ssb.rs (verified vs the RecurrenceHitRateMeasured/GateTaxMeasured
        precedent entries). PATCH: ssb-split NOTE added to Integration. (Not a check-breaker — the
        UsageTurnEvent grep vs ssb.rs still passes on the registration entry — but a real
        executable-blind gap.)
    (3) T7 had no file:line seam anchor and an UNDECLARED fork between two candidate homes
        (usage_ledger.rs fold vs H-50's context_policy.rs, a real distinct module); the two
        cumulative-growth numbers could diverge per lane with no test catching it. PATCH: FORK-3
        added (deterministic trigger for which module owns the monotonic-growth logic, do-not-do-
        both) + INTEGRATION_SMOKE PASS extended to assert T7's growth number == T5's session-sum
        across a compaction boundary on BOTH lanes.
  STALE/WRONG ANCHORS (grep-verified live 2026-07-09; DESIGN conclusions all survive, only citations
    drifted — corrected in the "Existing ground truth CORRECTIONS" note; the executor re-greps at
    edit per W-7/W-9/RECON_NEEDED-2/3 regardless): cloud_mode.rs for_model 683-691 -> 761-769
    (for_claude_sonnet_4_6 741-754); pricing.rs 82/143/163 -> 91/154/174 (consistent +11);
    ssb.rs 9190/9527/11522/18477 -> moved into ssb/action_type.rs + ssb/payloads.rs (pre-split
    anchors; BackendCostAccruedPayload-has-no-model/task/session key STILL TRUE at
    ssb/payloads.rs:6867-6882); server.py message_start 4728-4733 -> 5040-5044, message_delta
    4856-4863 -> 5181-5185, elapsed_ms 4984 -> ~5216-5311. ACCURATE (no drift): anthropic.rs
    1275/1313, grounding_client.rs:950, resume.rs:328, context_budget.rs:795, usage.rs:15/34,
    runtime_orchestrator.rs:331-395 (~336-390). Grounding-section SCHEMA_VERSION 234/543 is 37
    versions stale (live 271) — FORK-2 neutralizes at execution; header refreshed below so a skimmer
    does not anchor on it.

## Next
J-28 closes audit gaps C-1, C-2, C-6, L-1 (capture), L-5 (normalization), L-3 (estimator) and
produces the uniform row J-29's window + canvas + live meter render. J-30 (BYOK) is independent;
its custom backends inherit the ledger for free because emission sits at the orchestrator seams.

End of J-28.
