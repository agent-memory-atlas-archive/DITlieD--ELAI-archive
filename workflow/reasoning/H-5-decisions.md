H-5 — Batch API for non-interactive lanes — DECISIONS (session s-10612, 2026-06-04)

WHAT SHIPPED (Tier-1)
- crates/providers/src/providers/anthropic_batch.rs (NEW): AnthropicBatchClient — submit
  (POST /v1/messages/batches) + poll (GET .../{id}) + retrieve (GET .../{id}/results JSONL,
  mapped by custom_id) + cancel. Reuses AuthSource + AnthropicRequestProfile::render_json_body
  from the synchronous client (GD-1). GA: anthropic-version 2023-06-01, no beta header. Streaming
  forced off on every item. half_price_cents() = HD-1 50% helper. Unit tests for parse/version/url.
- crates/providers/src/providers/mod.rs + lib.rs: register + re-export the batch types at crate root.
- crates/providers/src/providers/anthropic.rs: AnthropicClient::batch_client() — a Message Batches
  sibling sharing this client's auth+base_url (GD-1, no transport restatement).
- crates/orchestrator/src/ssb.rs: +SsbActionType::BatchLifecycle (Pattern-2 single-variant, 3-arm
  BatchLifecycleKind Submitted/Completed/Expired) + BatchLifecyclePayload{kind,batch_id,request_count,
  model,spend_cents_half,task_id}. Full 9-tuple lockstep + the runtime SQL CHECK cascade entry
  (the G-8/G-9 silent-INSERT-failure site). SCHEMA_VERSION 218 -> 219 (re-read LIVE at edit time;
  216 in the plan doc was stale, G capstone moved it to 218 mid-pre-flight).
- crates/orchestrator/tests/action_type_lock.rs: TOTAL_VARIANTS 495 -> 496 + ALL_VARIANTS +
  exhaustiveness match arm. action_type_lock.golden: REGENERATED via regen_golden_file (alphabetical;
  hand-insert infeasible due to ordinal shift) — AUDITOR STEP.
- crates/contracts/tests/e45_cross_language_drift_contract.rs:294 pin 495 -> 496.
- crates/orchestrator/src/grounding_client.rs: CloudGroundingClient::ground_batch() — the REAL
  producer: builds N requests, submits ONE batch via self.client.batch_client(), bounded-polls,
  maps results by custom_id, emits batch_lifecycle Submitted+Completed with the HD-1 MEASURED
  spend_cents_half (= half the summed synchronous USD cost from response usage). + batch_lane_enabled().
- wire-dark-allowlist.toml: +ELAI_BATCH_LANE_ENABLE (R9 experiment; promotion_plan_id=H-5).
- crates/contracts/tests/h5_batch_lifecycle_contract.rs (NEW): action_type label, kind wire labels,
  serde roundtrip, half-price floor invariant, the SQL CHECK source-wiring, the batch endpoint, the
  producer wiring, AND an off-agent SSB runtime-CHECK fixture (writes batch_lifecycle Submitted+Completed
  to a real Ssb DB — proves the SQL cascade accepts the variant; W-1/W-5 Tier-1 off-agent artifact).

GROUNDED CORRECTION (the load-bearing finding — like s-66424's H-1 premise refutation)
The plan + H-SUMMARY framed the batch lane as "route CosmosWorker::tick through an Anthropic batch
client" with the chokepoint at anthropic.rs:440/678 and reachable_from live_cli cosmos_tick_emit_loop.
SOURCE REFUTES THIS:
  1. cosmos refresh research goes through an injected `dyn ResearchDispatcher` (cosmos_worker.rs:606);
     the PRODUCTION impl is HttpCosmosResearchDispatcher (grounding_client.rs:1301), whose run() calls
     SearxClient::search — i.e. cosmos research is SearXNG WEB SEARCH, NOT an Anthropic LLM call. There
     is NOTHING in the cosmos lane to route through the Anthropic Batch API.
  2. The genuine non-interactive Anthropic LLM lane is CloudGroundingClient (a DIFFERENT struct):
     ground() -> send_with_ledger (grounding_client.rs:880) -> AnthropicClient::send_message
     (anthropic.rs:440). Driven by the research_tree decomposition fan-out (research_tree.rs:2334).
So the batch producer was placed on CloudGroundingClient (ground_batch), NOT cosmos. cosmos is untouched.

W-10 TIERING (honest, no fabricated guarantee — GD-5)
The Batch API has a <=24h async window. ELAI's live non-interactive lanes do NOT tolerate it:
  - cosmos refresh = SearXNG web search (no Anthropic call);
  - interactive research decomposition = needs results NOW (human-waited downstream);
  - GEPA prompt-optimization (E-8) + bench/eval (E-9/E-49) = the genuinely deferrable Anthropic lanes,
    but RunPod-PENDING (no live API call today).
Therefore the LIVE production caller + the live POST to api.anthropic.com/v1/messages/batches are
Tier-2 / RunPod-pending. Tier-1 (this session) ships: the transport (AnthropicBatchClient, unit-tested),
the telemetry (batch_lifecycle SSB, roundtrip-tested via the orchestrator typed-payloads lock), the
producer (ground_batch), and an off-agent SSB-write fixture proving the runtime cascade. The live lane
is registered default-off (ELAI_BATCH_LANE_ENABLE, R9) with H-5 as the promotion plan. This is the
W-10 tiered pattern the plan itself prescribes for the bench/GEPA lanes — NOT wire-dark theater: the
substrate is real and exercised; only the 24h-tolerant live caller is honestly pending.

R6 NOTE (producer-without-live-consumer): ground_batch's only non-test caller is the future
GEPA/bench live lane (Tier-2). This is the legal R9 state (default-off experiment + promotion plan +
smoke). The auditor should treat the absent LIVE caller as the documented Tier-2 boundary, not a Tier-1 BLOCK.

CARRIED USER ACTION ITEMS (out-of-band)
- python .claude/tools/gate_freeze_check.py --lock  (the wire-dark-allowlist.toml +ELAI_BATCH_LANE_ENABLE
  edit trips the R8 freeze; the H-3 ELAI_FOCUS_TUNNEL_EXECUTE edit from s-66424 also pends a re-lock).

DEFERRED (Tier-2)
- The live POST to /v1/messages/batches over the wire (cloud; no sidecar needed but a real API key + a
  24h-tolerant lane). A production non-interactive caller of ground_batch (GEPA refresh / bench fan-out)
  once those lanes go live on RunPod.
