F-10 security-monotonicity iteration guard - decision doc (session s-60824, continues s-6344)

What I will implement
Part A (classifier, deterministic, ships-now): crates/orchestrator/src/cwe_classifier.rs - classify_delta(source, lang) -> BTreeSet<CweClass> that runs the existing F-1 evaluate_vibegate scanner over candidate source and buckets each HookFinding.rule_id into a CWE class via cwe_from_rule_id (pub-exported from vibegate.rs:454). Single source of truth: it reuses the F-1/F-2a mutable=0 security seed corpus, no parallel pattern tables, no new crate, no network.
Part B (gate + 3 legs): crates/orchestrator/src/security_monotonicity.rs - SecurityMonotonicityGate::check_monotonicity(loop_kind, loop_id, prior_set, current_source, lang) -> MonotonicityVerdict {Monotone | Regressed{introduced_classes}}. On Regressed the caller emits one flat Pattern-2 SsbActionType::SecurityClassRegressionBlocked (payload: delta_blake3, introduced_classes, prior_classes, loop_kind:LoopKind{Refine,AutoAdvance,SelfMod}, loop_id, lang) plus a Finding row. Wired at all three production iteration boundaries.

Key technical decision
The monotonicity gate is orchestrator-local and standalone (not a new TripwireKind), reusing the F-1 scanner; the three loop legs share one SSB variant discriminated by a loop_kind payload field, default-ON with a single ELAI_SECURITY_MONOTONICITY_DISABLE kill-switch (W-2).

Expected test outcome
cargo check --workspace GREEN; security_monotonicity_contract.rs round-trips the new SSB variant and asserts Regressed fires when iteration N introduces a CWE class absent at N-1; live `elai security-monotonicity --fixture tests/fixtures/f10/cwe78_regression.json --no-env-override` writes a real security_class_regression_blocked row to ssb.db (W-5).

Unknowns to verify empirically
1. Per-language false-positive rate of the reused scanner on a clean ELAI corpus (target FP<5%/lang, measured in F-10-classifier-calibration.md; Rust high-FP -> WARN-only degrade, no spiral).
2. Exact live count for all_test_payloads / TOTAL_VARIANTS (RE-COUNT at edit time; baseline 476->477, SCHEMA 199->200).
3. Whether `elai` exposes a refine/security subcommand group to host the W-5 fixture trigger, or a new safety subcommand must be added.

================================================================================
FORK 1 (anti-sycophancy, surfaced) - delta source in the REFINE loop
================================================================================
The integration-explorer found (HIGH) that the F-10 plan wording "fingerprint the per-iteration generated DELTA" is imprecise: chat_reflect.rs's REFINE loop refines candidate SELECTION (best_index), not code; final_reply is the whole chosen reply text, with NO within-call code diff.
Options considered:
  (a) re-target F-10 at the self_mod_* loop only (real per-proposal deltas) - REJECTED: drops the REFINE + auto-advance legs the plan explicitly names.
  (b) fingerprint the whole emitted code per ITERATION (a turn for REFINE, a level for auto-advance, a proposal for self-mod) and diff the CWE-class SET against the prior iteration's set (queried from SSB by loop_id) - CHOSEN.
Rationale: "iteration" in Shukla arXiv:2506.11022 (the +37.6% motivation, GROUNDED) means an AI-improvement round, i.e. across turns/proposals - not a sub-step inside one call. The cc payload {introduced_classes, prior_classes} already models exactly a per-iteration class-set difference. So the plan's intent is preserved; only the word "delta" is clarified to mean a CWE-class set difference, not a text diff.

================================================================================
FORK 2 (anti-sycophancy, surfaced) - classifier crate placement
================================================================================
The context-compiler proposed crates/safety/src/cwe_classifier.rs.
Options considered:
  (a) crates/safety crate - REJECTED: requires a new in-workspace dependency edge orchestrator->elai-safety (CLAUDE.md: no new deps without user confirmation; user is asleep), and it would invert deps because the scanner it must reuse (evaluate_vibegate) lives in crates/orchestrator. A self-contained safety classifier would also duplicate the F-1 CWE patterns.
  (b) orchestrator-local cwe_classifier.rs reusing evaluate_vibegate + pub cwe_from_rule_id - CHOSEN: zero new deps, no dep inversion, one source of truth for CWE patterns (the F-1 seed corpus), classifier co-located with the gate.
Rationale: the F-plan thesis is real wiring to existing symbols (W-7/W-8); reusing the already-shipped, already-low-FP F-1 scanner is strictly better than a parallel implementation, and it removes the only user-confirmation blocker while the user is asleep.

================================================================================
THREE PRODUCTION LEGS (W-3 - wire all real entrypoints)
================================================================================
REFINE leg: call-site after the refine pipeline returns final_reply - live_cli.rs:1554 (CLI) AND dashboard-v2 commands.rs:5676 (dashboard); tokio::spawn fire-and-forget (W-10 off the hot path); NO change to crates/reasoning chat_reflect.rs (avoids the reflect_bridge.rs:122 exhaustive-match E0004 and the reasoning->orchestrator dep inversion). loop_kind=Refine, loop_id=conversation/turn id; prior set from an SSB query.
AUTO-ADVANCE leg: a new Evidence "security_monotonicity" check field (autonomous_gate.rs:208) that VACUOUS-DEFAULTS TRUE (Default + serde default so the 25 cold-start gate contracts do not break), derived by derive_security_monotonicity_evidence and appended at the Evidence struct-literal task_runner.rs:13197; feeds all_passed + failed_checks (NOT summary()); a regression flips the check false -> auto-advance blocked. loop_kind=AutoAdvance.
SELF-MOD leg: a precondition at the two_gate_promote CALLER external_benches.rs:557 (the 4-arg promotion_gate.rs:103 signature is LOCKED - do not change it); a regressed proposal blocks promotion before archive admission. loop_kind=SelfMod.

================================================================================
SSB 10-SITE LOCKSTEP CASCADE (gotcha #6 - move together or the golden breaks)
================================================================================
enum arm (ssb.rs ~8101) + SQL CHECK alpha-insert (~4441) + as_sql_str (:8148) + ssb_action_type_from_str (~8919 + ~9551) + SsbPayload variant (~12923) + payload->action_type() arm (~13593) + write builder (~28605) + all_test_payloads +1 (:29452, RE-COUNT live len) + ALL_VARIANTS (action_type_lock.rs:510) + golden alpha-insert (golden:415, between search_trigger_fired and security_rule_blocked, renumber trailing) + SCHEMA_VERSION 199->200 (ssb.rs:2780) + TOTAL_VARIANTS 476->477 (action_type_lock.rs:501). Round-trip golden fixture in security_monotonicity_contract.rs.

Grounding (sg, GROUNDED): Shukla 2506.11022 +37.6%/5-iters and Sajadi 2507.02976v3 11x/135-vs-12 are MOTIVATION-ONLY - no numeric threshold/reward/budget derives from either; the gate is binary CWE-class presence. Pair FP<5% with KNOWN-VULN recall fixtures so it cannot be gamed by under-detecting (Goodhart guard). Cite Sajadi as v3 (not v1 185/9x/20).

================================================================================
SLICE A5 (dashboard pair, W-4/W-6) - SHIPPED s-60824
================================================================================
Mirrored the F-2a precedent (commands_f2.rs) over F-9 because F-2a deserializes the full SsbPayload and matches the arm + uses entry.ts/entry.seq from SsbActionDetail, which is the exact shape needed for the SecurityClassRegressionBlockedPayload struct (delta_blake3, introduced_classes, prior_classes, loop_kind: LoopKind, loop_id, lang). F-9 instead hand-reads JSON keys, less robust for a Vec<String>/enum payload.
DrawerKey convention is KEBAB-case (deps-security, security-rules), NOT camelCase as the briefing text said; used 'security-monotonicity' to stay consistent.
Files created: crates/dashboard-v2/src-tauri/src/commands_f10.rs (get_security_monotonicity_blocks -> SecurityMonotonicityResponse{rows: Vec<SecurityMonotonicityRow>, env_active}; env_active = !truthy(ELAI_SECURITY_MONOTONICITY_DISABLE_ENV)); crates/dashboard-v2/src/lib/drawers/SecurityMonotonicityDrawer.svelte (5s poll, loop-kind badge + introduced/prior CWE chips + ts; client-side per-loop-kind aggregation via $derived).
4-file DrawerKey sync: types.ts (DrawerKey union + SecurityMonotonicityRow/Response DTOs), DrawerHost.svelte (import + else-if), Drawer.svelte (DRAWER_LABELS), Rail.svelte (DrawerKeyUpper union + drawerItems entry, Icon ShieldAlert mirroring sibling security drawers). loop_kind serialized String-side via serde_json::to_value(LoopKind) (snake_case rename) so TS carries a plain string, no enum re-import.
cross_lang_diff.py: registered pair security_monotonicity_drawer (commands_f10.rs <-> SecurityMonotonicityDrawer.svelte). commands_f10.rs carries a cross-lang-skip: tauri_commands header (matches F-9/F-2a precedent; F-9 pair did NOT wire stub.ts, so neither did A5).
Verify: cargo check -p elai-dashboard-v2 GREEN (dashboard-v2 clean; only pre-existing orchestrator warnings). svelte-check 0 ERRORS / 22 WARNINGS (all pre-existing, none in the 4 touched files); tsc --noEmit exit 0. (pnpm not on PATH locally; ran node_modules svelte-check + tsc directly, equivalent to `pnpm run check`.)
