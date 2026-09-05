# H-5: Batch API for non-interactive lanes — flat 50% off on Cosmos / research / bench

STATUS: PLANNED (WAVE 1; BUILD — no batch transport exists today)

Objective: The Message Batches API charges 50% of synchronous pricing on all models (Sonnet 4.6 batch $1.50/$7.50
vs $3/$15), <1h typical, 24h window, and pairs with the 1-hour cache TTL — the only cost is latency, so it is a
natural fit for ELAI's autonomous / background lanes that no human waits on. Grounding (C16): there is NO Anthropic
Batch API anywhere in ELAI (zero hits); every LLM lane runs synchronously at full price. H-5 adds a batch client
(POST /v1/messages/batches + poll + retrieve) and routes the clearest batch-eligible lane (Cosmos refresh — a
sequential per-artifact for-loop), plus the research fan-out, through it; the bench/eval LLM lanes are RunPod-pending
and are handled W-10-tiered (a deterministic fixture harness is the fail-closed Tier-1, the live API call is a
separately-gated checkbox). The interactive /task + chat REPL stays on the synchronous streaming path.

Estimated effort: ~1-2 sessions (~6-10 h). The Cosmos lane is pure in-process at gate time (the batch round-trip is
mocked in the fixture; a live checkbox exercises the real endpoint). W-10 tiered.
Pre-requisites: B-42 Cosmos SHIPPED (cosmos_worker.rs tick :853); the AnthropicClient HTTP plumbing SHIPPED
  (send_raw_request anthropic.rs:678 — reuse for the batch sibling).
Parallel-safe with: H-1..H-4 (disjoint, except the shared anthropic.rs file — coordinate edits), H-6/H-7/H-8.
Source: ELAI-EFFICIENCY-TRUST-DIRECTION-2026-06-03.md §10.5 (LEVER L5); _grounding/H-GROUNDING-MAP.md C16 + verify:batch-api. Doctrine: H-0_INDEX.md.

## Context

Batch (verify:batch-api, 2026-06-03): create POST api.anthropic.com/v1/messages/batches (anthropic-version
2023-06-01); retrieve .../{id}; cancel .../{id}/cancel; 50% on all active models; 24h processing window (expire if
not complete, not billed for expired); results retained 29 days; capped at 100,000 requests OR 256 MB; supports tools
+ system + multi-turn + extended thinking + most beta features; EXCLUDES streaming + Fast mode + max_tokens:0 + Threads
+ research_preview; Anthropic recommends pairing the 1-hour cache TTL for shared-context hit rates 30-98%. Interactive
editors cannot batch their core loop; ELAI's Cosmos / research / GEPA / bench lanes are a natural 50%-off fit a
human-in-the-loop IDE has no equivalent for (ELAI-EFFICIENCY §10.5, §12). The only cost is 24h worst-case latency —
do NOT batch anything a human is waiting on.

## Existing state (B/C/D/E/F/G coverage)
- Anthropic Batch API — ABSENT. Zero batch/message_batches/MessageBatch hits in anthropic.rs; only synchronous
  /v1/messages (send_message :440, send_raw_request :678). MUST be CREATED.
- CosmosWorker::tick (knowledge/src/cosmos_worker.rs:853) — SHIPPED. Synchronous per-artifact for-loop (:868-921),
  no fan-out. Prod callers live_cli.rs:502 (cosmos_tick_emit_loop), dashboard commands.rs:9264-9275 (trigger_cosmos_refresh,
  registered lib.rs:752). The clearest batch-eligible lane.
- WebResearchOrchestrator::run_node_loop (orchestrator/src/web_research_orchestrator.rs:265) — SHIPPED. Sequential node
  loop; prod caller task_runner.rs:2172-2188. Batch-eligible but weaker (inherently sequential); chunks(batch_size) at
  :416 is web-page fetch, NOT an LLM batch.
- GepaTemplateEvolution::evolve_round (rl/gepa_template_evolution.rs:204) — WIRE-DARK + RUNPOD-PENDING (stub reflection).
- bench/eval LLM lanes (metrics_cmd.rs, external_benches.rs) — RUNPOD-PENDING; bench_cmd never calls the model.
- AnthropicClient::send_raw_request (anthropic.rs:678) — SHIPPED HTTP plumbing to reuse for the batch sibling.

## Reads (mandatory before coding)
- _grounding/H-GROUNDING-MAP.md C16 + the H-5 symbol table + the WEB VERIFICATION verify:batch-api rows (all CONFIRMED).
- ELAI-EFFICIENCY-TRUST-DIRECTION-2026-06-03.md §10.5 (L5) + §12 (the batch lanes a human-in-the-loop IDE cannot use).
- F:\.ELAI_workflow\.elai_cc\crates\providers\src\providers\anthropic.rs:670-720 (send_raw_request — the HTTP plumbing to mirror)
- F:\.ELAI_workflow\.elai_cc\crates\knowledge\src\cosmos_worker.rs:850-925 (tick — the sequential for-loop to batch)
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\web_research_orchestrator.rs:260-420 (run_node_loop)
- CLAUDE.md HARD RULES + anti-slopsquatting (the endpoint + version strings are the verified ones); W-10 (tiered gate).

## Tasks (WAVE 1 — build the transport + route Cosmos)

1. **Build the batch client (~160 LOC, new module crates/providers/src/providers/anthropic_batch.rs).** create
   (POST /v1/messages/batches), poll (GET .../{id} until ended_at), retrieve (.../{id}/results), cancel. Reuse the
   send_raw_request HTTP plumbing (anthropic.rs:678) for headers/auth/base_url. Honor the exclusions (no streaming, no
   Fast mode, no max_tokens:0). Pair the 1h cache TTL (H-2) on the per-request cache_control.

2. **Route CosmosWorker::tick through the batch client (~80 LOC).** Replace the synchronous per-artifact for-loop
   (:868-921) with a batch submission of all due artifacts + a poll-to-completion; on completion, apply the results.
   Emit a batch_lifecycle SSB row (Submitted -> Completed/Expired) carrying the request count + the 50%-priced spend.

3. **Route the research fan-out through the batch client (~40 LOC).** Where run_node_loop issues independent LLM calls
   that no human waits on, submit them as a batch (the inherently-sequential dependency chain stays synchronous).

4. **+batch_lifecycle SSB (Pattern-2; Kind Submitted/Completed/Expired) (~80 LOC in ssb.rs).** 9-tuple lockstep
   (variant + sql_str "batch_lifecycle" + parse arm + SQL CHECK + payload BatchLifecyclePayload{kind, batch_id,
   request_count, model, spend_cents_half: i64, task_id} + action_type() + test_payload + all_test_payloads + golden).
   SCHEMA bump baseline+1 (LIVE 207/484 -> re-read at edit time). The first H-N to bump also bumps
   e45_cross_language_drift_contract.rs:294 (latent 482 drift).

5. **W-10 tiered handling of the RunPod-pending bench lanes (~40 LOC).** A deterministic fixture harness exercises the
   REAL batch path against a mocked endpoint (the fail-closed Tier-1 done-gate); a separately-gated live checkbox runs
   the real /v1/messages/batches call (Tier-2, MUST NOT advertise fail-closed if it is fail-open).

6. **Decision doc workflow/reasoning/H-5-decisions.md (~100 LOC).** Records: no batch transport today; the exclusions;
   the 1h-TTL pairing; Cosmos is the clearest lane; bench/GEPA are RunPod-pending (W-10 split); never batch a human-waited call.

## WIRING_CONTRACT
produces_symbols:
  - crates/providers/src/providers/anthropic_batch.rs::AnthropicBatchClient (create/poll/retrieve/cancel)
  - crates/knowledge/src/cosmos_worker.rs (tick routed through the batch client + batch_lifecycle emit)
  - crates/orchestrator/src/ssb.rs::SsbActionType::BatchLifecycle
  - crates/orchestrator/src/ssb.rs::BatchLifecycleKind
  - crates/orchestrator/src/ssb.rs::BatchLifecyclePayload
consumes_symbols:
  - crates/providers/src/providers/anthropic.rs::send_raw_request (the HTTP plumbing to reuse)
  - crates/knowledge/src/cosmos_worker.rs::tick (the for-loop to batch)
  - crates/orchestrator/src/web_research_orchestrator.rs::run_node_loop
reachable_from:
  - crates/cli/src/live_cli.rs:502 (cosmos_tick_emit_loop) -> CosmosWorker::tick -> AnthropicBatchClient -> batch_lifecycle SSB  [PRODUCTION cosmos lane]
  - crates/dashboard-v2/src-tauri/src/lib.rs:752 (trigger_cosmos_refresh) -> tick  [dashboard trigger]
stub_ban:
  - NO batch routing of an interactive / human-waited call (only autonomous lanes)
  - NO streaming / Fast mode / max_tokens:0 in a batch request (the API rejects them)
  - NO Tier-2 live path advertising fail-closed when it is fail-open (W-10)
  - NO hardcoded SCHEMA value (bump from the LIVE 207/484)
  - blake3; no .unwrap()/.expect() in prod; anti-slopsquatting (endpoint + version are the verified strings)
frontend_pair:
  a batch-status tile (optional) reading batch_lifecycle rows; OR N/A with justification (the spend is surfaced by H-9).
observable_effect:
  trigger: "cd .elai_cc && cargo run -p elai-cli -- <trigger a Cosmos refresh with >=2 due artifacts, NO env override>"
  expect_ssb: "a batch_lifecycle row (kind=Submitted then Completed, request_count>=2, spend_cents_half = half the
    synchronous cost) lands in the live ssb.db, recomputable from the journal under ELAI_SEED"
  expect_artifact: "the Cosmos artifacts are refreshed via a SINGLE batch submission to /v1/messages/batches (not N
    synchronous calls — verified by the journal showing one batch id); the interactive /task path is unaffected"

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/H-plan/H-5_batch_api_noninteractive_lanes.md --run-trigger
rg "messages/batches|AnthropicBatchClient" .elai_cc/crates/providers/src/providers/anthropic_batch.rs  # the batch transport exists
rg "AnthropicBatchClient|batch" .elai_cc/crates/knowledge/src/cosmos_worker.rs                          # cosmos routed through batch
rg "BatchLifecycle|batch_lifecycle" .elai_cc/crates/orchestrator/src/ssb.rs
rg "batch_lifecycle" .elai_cc/crates/orchestrator/tests/action_type_lock.rs                             # golden lock updated
rg "482|484|486" .elai_cc/crates/contracts/tests/e45_cross_language_drift_contract.rs                    # e45 drift bumped if H-5 is the first bumper
test -f workflow/reasoning/H-5-decisions.md
```

## ACCEPTANCE
- [ ] W-1 done-gate: --run-trigger PASS (a Cosmos refresh routes through one batch submission + a batch_lifecycle row lands).
- [ ] W-2: batch routing active by default for the Cosmos lane; no default-off ENABLE as a terminal state.
- [ ] W-3: the batch fires from the live cosmos_tick_emit_loop / trigger_cosmos_refresh; not test-only.
- [ ] W-5/W-10: the bench/GEPA RunPod-pending lanes ship a deterministic fixture harness (Tier-1) + a separately-gated live checkbox.
- [ ] W-9: batch_lifecycle round-trips via the SSB roundtrip contract + action_type_lock golden; SCHEMA bump named baseline+1; e45 drift fixed.
- [ ] HD-1: the 50%-off saving is the MEASURED spend (half of the recorded synchronous cost), recomputable from the journal.
- [ ] cargo check --workspace GREEN; auditor PASS on -p elai-providers -p elai-knowledge -p elai-orchestrator -p contracts.
- [ ] No new dependency (the HTTP client is reused from anthropic.rs).

## WIRING DONE-GATE (clause by clause)
- W-1: --run-trigger is ACCEPTANCE 1; the batch client (produced) is consumed by tick (W-8) reached by the live cosmos loop (W-3).
- W-2: cosmos batch routing default-on.
- W-3: live_cli.rs:502 / dashboard lib.rs:752; not test.
- W-4: batch_lifecycle is dashboard-surfaceable -> a tile + cross_lang_diff (or N/A justified, spend via H-9).
- W-5: a runnable cosmos refresh -> a real batch_lifecycle row; the bench lanes are W-10 tiered.
- W-6: the batch tile via dashboard-smoker (or N/A justified).
- W-7: send_raw_request / tick / run_node_loop grep-confirmed 2026-06-03 (C16).
- W-8: the batch client (produced) is consumed by tick; BatchLifecyclePayload (produced) is emitted by tick + read by the tile/H-9.
- W-9: +batch_lifecycle Pattern-2 baseline+1; golden + e45 drift fix.
- W-10: the Cosmos lane Tier-1 is in-process (mocked endpoint in the fixture); the live endpoint is a separately-gated Tier-2 checkbox.

## Tests to add
- crates/contracts/tests/h5_batch_lifecycle_contract.rs (~170 LOC): (a) the batch client constructs a valid create
  request to /v1/messages/batches with anthropic-version 2023-06-01 and NO streaming/Fast/max_tokens:0; (b) tick with
  >=2 due artifacts submits ONE batch (not N synchronous calls); (c) a batch_lifecycle row carries kind + request_count
  + spend_cents_half; (d) the BatchLifecycle Pattern-2 variant round-trips for all 3 Kind arms; (e) the 1h cache TTL is
  paired on batch requests; (f) the bench-lane fixture exercises the real batch path against a mocked endpoint.

## Files touched
- CREATE: crates/providers/src/providers/anthropic_batch.rs (~160 LOC)
- MODIFY: crates/providers/src/providers/mod.rs (~2 LOC: pub mod anthropic_batch;)
- MODIFY: crates/knowledge/src/cosmos_worker.rs (~80 LOC: route tick through the batch client + emit)
- MODIFY: crates/orchestrator/src/web_research_orchestrator.rs (~40 LOC: batch the independent research calls)
- MODIFY: crates/orchestrator/src/ssb.rs (~80 LOC: BatchLifecycle variant + Kind + payload + 9-tuple lockstep)
- MODIFY: crates/orchestrator/tests/action_type_lock.rs (~2 LOC: SCHEMA + ALL_VARIANTS) + e45_cross_language_drift_contract.rs:294 (drift)
- CREATE: crates/contracts/tests/h5_batch_lifecycle_contract.rs (~170 LOC)
- CREATE: workflow/reasoning/H-5-decisions.md (~100 LOC)

## Citations / Grounding
- ELAI-EFFICIENCY-TRUST-DIRECTION-2026-06-03.md §10.5 (L5): 50% all models; the batch lanes a human-in-the-loop IDE cannot use.
- verify:batch-api (live docs 2026-06-03, all CONFIRMED): /v1/messages/batches; 50%; 24h/29d/100K-256MB; supports
  tools+system+thinking; excludes streaming+Fast+max_tokens:0+Threads; pair the 1h cache TTL.
- ELAI ground truth (C16): no batch code (zero hits); cosmos tick :853 sequential for-loop; send_raw_request :678; bench RunPod-pending.
- CLAUDE.md HARD RULES; W-10.

## Open questions / risks
- Latency: 24h worst-case. Cosmos refresh is background, so acceptable; but a refresh that a user explicitly requested
  and waits on must stay synchronous — gate batch routing on the trigger being autonomous (the tick loop), not the dashboard button.
- The 1h cache TTL pairing matters for shared-context batches (hit rates 30-98%); without it the 5-min TTL expires before the batch completes.
- Result application order: a batch returns results out of submission order; tick must map results back to artifacts by custom_id, not position.

## Next
H-9 (cost cockpit) surfaces the batch spend as a 50%-discounted lane in the $/task breakdown. H-2's 1h TTL is the
prerequisite for good batch cache hit rates. GEPA + the eval suites become batch-eligible once their RunPod scaffold runs (W-10 live checkbox).

End of H-5.
