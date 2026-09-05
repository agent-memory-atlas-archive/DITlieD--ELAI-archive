H-1 DECISION DOC — prompt caching on the production path
session s-66424 | 2026-06-03 | orchestrate H-plan WAVE 1

WHAT
H-1 was framed as "close the RuntimeAdapter cache split-brain + activate the dark hit-rate monitor".
Grounded source reading refutes half the premise and narrows H-1 to one real deliverable.

GROUNDED FINDING (split-brain premise REFUTED)
AnthropicApiRuntime::complete (anthropic_api.rs:207) calls self.client.send_message (:215).
AnthropicClient::send_message (anthropic.rs:440) calls apply_anthropic_cache_markers UNCONDITIONALLY (:448).
apply_anthropic_cache_markers (anthropic.rs:1026) marks the last system block when cache_control.is_none() + the last tool def.
=> The RuntimeAdapter path ALREADY ships ephemeral markers via the shared send_message. The cache_control:None at
   anthropic_api.rs:137 is on per-message InputContentBlock content, which the marker fn never touches, so it is inert.
=> H-1 Task 1 ("unify markers onto the RuntimeAdapter path") is largely a NO-OP for correctness.
Independent confirmation: crates/contracts/tests/cache_control_marker_contract.rs already asserts turn-2
   cache_read_input_tokens > 0 on the cloud path (a live passing contract). Markers work.

KEY DECISIONS
D1 (consume contract, defensive): promote apply_anthropic_cache_markers to pub(crate) and call it idempotently in
   complete() before send_message. Rationale: makes the RuntimeAdapter path's marker-consume EXPLICIT at the adapter
   boundary (satisfies WIRING_CONTRACT consumes + reachable_from by name, not only via send_message's internals), and
   guards the path against a future send_message refactor. Idempotent (is_none() guard), zero behavior change today.
   GD-1: REUSE the shipped stamper, never re-implement (done-gate grep must show ONE stamper).
D2 (THE deliverable): add a PRODUCTION caller of measure_cache_hit_rate (cache_check.rs:32, wire-dark, zero prod
   callers). Site: grounding_client.rs CloudGroundingClient::send, inside the existing `if let Some(ssb)` block, right
   after the CloudSpend write (:942), where ssb + response.usage + agent_id("cloud_grounding_client") + superstep(0)
   are all in scope. This is a real production method, not a test shim (W-3).
D3 (snapshot mapping): CacheCheckSnapshot{ hits = usage.cache_read_input_tokens, misses = usage.input_tokens }.
   hit_rate = cache_read / (cache_read + fresh_input) = the per-call cache-read fraction; non-zero iff cache_read>0;
   recomputable from the journal (challenger_score field), GD-2. Per-call emit (mirrors the per-call CloudSpend row).
   Definition is documented + pinned by the contract test so it is self-consistent and recomputable.
D4 (no schema bump): measure_cache_hit_rate emits the EXISTING CriticDecisionEvent / CriticTilePhase::PromptCacheHitRate
   row (action_type prompt_cache_hit_rate). No new SsbActionType. SCHEMA_VERSION stays 209 / TOTAL_VARIANTS 486.
   (CacheEfficiencySampled is NOT added — reuse path taken, per H-0 conditional guidance "prefer reuse, minimise bumps".)
D5 (default-ON, W-2): the new caller is NOT wrapped in ELAI_CRITIC_OF_DESIGN_ENABLE. It fires whenever the CloudSpend
   ssb block fires (writer attached). No env override to enable.

CROSS-SESSION (GD-1 boundary)
The prod ctor .with_ssb_writer attach (which lights the `if let Some(ssb)` block in production) is H-9's owned
deliverable (CORRECTION C10: "the sole cloud client ctor never chains .with_ssb_writer; CloudPanel renders all-zeros").
H-1 does NOT pre-empt it. H-1 wires the monitor CALL (reachable from CloudGroundingClient::send) and proves it via the
Tier-1 fixture (which attaches a test writer). When H-9 attaches the writer in prod, BOTH CloudSpend and the H-1 monitor
fire live together. H-9 then owns the dashboard surface (CloudPanel render + token-tracker UI + $/task). Documented to H-9.

DONE-GATE (Tier-1, per W-5/W-10; sidecar live = deferred Tier-2)
crates/contracts/tests/h1_caching_production_contract.rs: constructs CloudGroundingClient with a test Ssb writer + a
stub provider returning a usage with cache_read_input_tokens > 0, drives the REAL send path, asserts (a) a CloudSpend row
and (b) a prompt_cache_hit_rate (CriticDecisionEvent/PromptCacheHitRate) row both land, with challenger_score ==
cache_read/(cache_read+input). Plus a static assert that complete() carries markers (the consume). Plus a non-test-caller
grep proof. cargo check --workspace green + auditor PASS on -p elai-providers -p elai-orchestrator + wiring_contract_check
static legs. The LIVE two-turn cloud run (real cache_read>0 over the wire) is a Tier-2 deferred checkbox (sidecar-gated;
user asleep, cannot boot the hardware-heavy sidecar).

EXPECTED TEST OUTCOME
The new contract test PASSES (rows land, challenger_score matches the formula). cargo check --workspace compiles.
The pre-existing e45_cross_language_drift_contract.rs:294 pin (482) vs live (486) is a LATENT FAIL independent of H-1
(no SSB lockstep touched here) — flagged to the auditor, NOT fixed in H-1.

RISK / FAILURE MODES (top 3)
R1 visibility-promotion ripple: promoting apply_anthropic_cache_markers to pub(crate) is additive (0 external callers
   today per impact_radius) — no blast radius.
R2 borrow/lifetime on the ssb call: measure_cache_hit_rate takes &Arc<Ssb>; `ssb` is already &Arc<Ssb> in the block — direct pass, no clone needed.
R3 snapshot semantics: input_tokens vs total-input ambiguity is sidestepped by pinning the exact formula in the test
   (cache-read vs fresh-input ratio), so the number is self-consistent + recomputable regardless of SDK aggregation.

UNKNOWNS / DEFERRED
- Live cache_read>0 over the wire (Tier-2, sidecar-gated) — DEFERRED.
- Prod .with_ssb_writer attach — H-9.
