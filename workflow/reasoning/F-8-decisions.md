F-8 SCF symbolic semantic conflict detector (self-mod pre-execution) - decision doc (session s-60824)

Scope (the plan's strong steer, confirmed)
SELF-MOD pre-execution path ONLY. DEFER the inter-agent BSP half (RUNPOD-PENDING: 27.9% FP degrades to noise in a cap=3 parallel loop; would collide with C-50 precondition/postcondition namespace; Graphiti LLM-judged contradiction detection is the future-F-N alternative for that half). Entirely Tier-1 in-process (no network/subprocess/RunPod at gate time).

What I will implement
A symbolic (LLM-free) conflict detector that flags a self-mod config variant which contradicts an active locked-key invariant BEFORE archive admission, emits symbolic_conflict_detected, and HOLDS the candidate pending (WARN-then-escalate), surfaced in the dashboard.

Key technical decision
Policy is WARN-then-escalate (held-pending in autonomous / operator-confirm in interactive), NEVER a hard fail-closed BLOCK - because SCF's 27.9% precision is simulation-only and unverified (NON-target); one config promotion per session bounds the escalation cost. The detector ships as a guard SIGNAL (a design hypothesis), not a claimed accuracy uplift; 27.9% must not appear as an ELAI target anywhere.

Expected test outcome
cargo check --workspace --tests GREEN; symbolic_conflict_contract asserts the 3 conflict kinds fire on crafted intents + clean variants pass; the SSB round-trip holds; live `elai self-mod apply --proposal-fixture tests/fixtures/contradictory_locked_variant.json` emits a real symbolic_conflict_detected row + reports the candidate held-pending.

Unknowns to verify empirically
1. The exact held-pending representation at the promotion seam (a non-blocking PromotionDecision outcome vs an emit+hold without widening the enum) - executor picks the minimal-blast option.
2. Live all_test_payloads / TOTAL_VARIANTS / golden ordinal (RE-COUNT; baseline 480->481, SCHEMA 203->204, all_test_payloads 400->401, golden alpha ~0404).
3. Whether a `self-mod` CLI group already exists to host the trigger, or a new subcommand is added.

================================================================================
THE W-3 ENTRYPOINT RESOLUTION (the cc HIGH-3 risk) - critical
================================================================================
The self-mod config-variant promotion path is largely WIRE-DARK: external_benches.rs:565 promote_after_bench_completed (the F-10 wrapper) is #[cfg(feature=darwinian-evolve)] with NO live caller; two_gate_promote (promotion_gate.rs:109) and config_archive::insert_variant (config_archive.rs:198) also have NO production caller (all promotion_gate callers are #[cfg(test)]). The live apply_proposal path (self_mod_layer1a.rs:831) admits DataTableProposal, NOT ConfigVariant, so the config-variant adapters have nothing to adapt there.
RESOLUTION: F-8's real W-3 production entrypoint is a NEW CLI subcommand `elai self-mod apply --proposal-fixture <json>` (W-3 explicitly counts a CLI subcommand as a production entrypoint). It drives a ConfigVariant through the REAL detect_conflicts + the admission decision, emits symbolic_conflict_detected, and reports the candidate held-pending. The detector is ALSO composed at external_benches.rs:619 (a 3rd early-check, mirroring F-10's security precondition at :586/:606) so it fires in the darwinian-evolve autonomous loop WHEN that path ships - this in-loop firing is a documented GATED CHECKBOX (the self-mod loop is RUNPOD-PENDING per E-7/E-9, the exact same posture as F-10's self-mod leg). The CLI is the proven live path; the autonomous-loop firing is gated. This is NOT laziness - the config-variant promotion path was wire-dark before F-8, and F-8 adds the detector + a live operator entrypoint + the future-loop composition.

================================================================================
Detector (symbolic_conflict.rs, LLM-free)
================================================================================
detect_conflicts(intent: &SymbolicIntent, model: &ProcessModel) -> Vec<SymbolicConflict>. ConflictKind { Contradictory, ResourceContention, CausalViolation } (3 symbolic types). SymbolicIntent { asserts, requires } + IntentCondition. intent_from_config_variant(ConfigVariant) (self_mod_types.rs:254) [+ intent_from_structural_change for completeness]. conflict_fingerprint -> blake3 prefix. current_process_model built from self_mod_safety::LOCKED_KEYS (pub const, 12 entries: score_weights, gate_definitions, auditor_config, benchmark_harness, test_suite, config_variants_schema, safety_locks, lock_list, cloud_monthly_budget, cloud_opus_allowed, cloud_escalation_threshold, ...) + is_locked (pub) - do NOT use the private locked_set. tripwire_quiescent_violated = ANY non-empty active set of the 5 TripwireKind variants {KlDivergence, CorrelationDrop, IqrOutlier, Perplexity, Ddm} (NOT the plan's 3; there is no Divergence variant). A config variant raising a LOCKED_KEYS-governed parameter while a tripwire is active is the canonical conflict.

================================================================================
SSB symbolic_conflict_detected (single flat beacon, 7-site cascade)
================================================================================
ONE SsbActionType::SymbolicConflictDetected (flat, no Kind) mirroring F-5 StalenessQuarantineBroadcast / F-10. 7-SITE cascade (NOT 5 - the 2 extra are the SQL CHECK allowlist + the from_str parse arm, which compile but break tests silently): enum arm + SQL CHECK allowlist append + as_sql_str + from_str + SsbPayload variant + action_type() dispatch (+ from-arm + to-payload arm) + all_test_payloads +1. SymbolicConflictPayload { fingerprint:String, kind:String, key:String, detail:String, candidate_id:String, fsm_substate:String }. SCHEMA 203->204; TOTAL_VARIANTS 480->481; all_test_payloads 400->401; golden alphabetical insert (~ordinal 0404, renumber trailing). Ssb::new INSIDE rt.block_on (gotcha:110 - construction spawns a write-loop; cargo-green + pure-fn tests hide the no-reactor panic; only the live trigger catches it). Round-trip golden fixture.

================================================================================
Compose-with-F-10 + dashboard + grounding
================================================================================
PromotionDecisionKind (promotion_gate.rs:69) has 4 arms (Promoted, BlockedBelowMargin, BlockedCapacityExceeded, BlockedSecurityRegression - the last added by F-10). F-8 does NOT add a 5th Blocked* arm (WARN-then-escalate is not fail-closed); it either adds a non-blocking held/escalate outcome or emits+holds without widening the enum - executor picks the minimal-blast path. two_gate_promote 4-arg sig stays LOCKED; compose at the external_benches.rs:619 caller exactly as F-10 did.
Dashboard (W-4/W-6): commands::get_symbolic_conflicts + ssb_viewer::symbolic_conflicts query + SymbolicConflictRow (mirror applied_proposals_layer1a:4150 / get_applied_proposals:1127) + a 'Symbolic Conflicts' section in ReasoningDrawer.svelte + types.ts/stub.ts + invoke_handler. (ReasoningDrawer is NOT one of the s-73316-owned windows - safe.)
C-50 ContractSpec: zero matches in the tree - NO namespace collision with the F-8 IntentNode/SymbolicIntent types (confirmed).
Grounding (sg): SCF 27.9% precision is a SIMULATION-only limitation; SCF is sim-only, 0 stars, empty tests/ (reliability LOW) - 27.9% is a NON-target and justifies WARN-then-escalate over fail-closed. The IntentNode pre/postcondition 3-conflict-type method is the grounded mechanism shape; F-8 thresholds are ELAI own.
HAZARDS: promotion_gate.rs / external_benches.rs are F-10-touched - re-grep their CURRENT state and compose, do not clobber. cargo check --tests must enable the darwinian-evolve / self-modification features if F-8 composes into the gated path. Build-isolate the concurrent knowledge RRF breakage (deep.rs/pipeline.rs) via git-stash; do NOT edit/commit those.
