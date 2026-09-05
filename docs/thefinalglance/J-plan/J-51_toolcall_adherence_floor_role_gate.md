# J-51: tool-call adherence floor — measure per-local-model semantic tool-call reliability under the grammar mask, gate tool-heavy role assignment on it

STATUS: PLANNED (WAVE GUAVA-1; Tier-1 bench + profile + advisory role-gate now; nature BUILD-on-shipped over B-10 ModelCapabilityProfile + PerRoleRuntime)

- Track J (Guava intake) | lever PROVABLE+SMARTER-ORCHESTRATION | grounding STRONG (Guava arXiv:2606.18363 Table 3, measured; ELAI substrate PLAN-asserted, executor re-greps W-7) | nature BUILD-on-shipped | effort M | WAVE GUAVA-1 | parallel-safe with J-50 (disjoint files) | composes B-10 / E-31 / J-34
- Premise: Guava (arXiv:2606.18363) ran ONE harness across many models (Table 3): Gemini-3.1-Pro 89.8%, Claude-Sonnet-4.6 88.9%, GPT-5.4 70.2%, Qwen3.5-2B 2.2% — the 2B model FAILS by hallucinating tool calls and not holding the structured format; the harness is universal only ABOVE a ~2-4B tool-call-adherence floor. ELAI's local lane is Qwen2.5-Coder-7B (primary) + 3B (alternate, NF4); the 3B sits on that floor for tool-heavy roles. ELAI already MITIGATES the FORMAT failure Guava's 2B hit: the sidecar XGrammar LogitsProcessor mask forces valid JSON tool calls (server.py), and HD-3 (arXiv:2408.02442) correctly scopes constrained decoding to TOOL-CALL FORMAT only (never reasoning/coder/planner). So the FORMAT layer is handled. What is UNMEASURED is the RESIDUAL SEMANTIC layer: under the mask (valid JSON guaranteed), does the small local model pick the RIGHT tool with the RIGHT arguments? J-51 measures that residual per-(local model), surfaces it on ModelCapabilityProfile, and lets the role-router prefer a higher-adherence model for tool-heavy roles — the J-34 doctrine ("measure the per-model number, stop trusting folklore") applied to tool-call reliability instead of context fill.

## Steal/build (3 legs, mirrors J-34's bench->profile->consume shape)

1. ADHERENCE BENCH (`elai bench tool-call-adherence`): a fixed fixture set of tool-requiring tasks, each with a KNOWN-correct (tool, args) ground truth, run per (local model, lane) UNDER the production grammar mask (the bench exercises the real masked path, not a bypass). Score the deterministic tool-selection-correct rate + args-schema-valid + args-semantically-correct rate (gate/schema verdicts, NEVER LLM-judge — TD-3). Output per (model): an adherence score in [0,1] + a metrics TOML, byte-recomputable from the journal under ELAI_SEED (GD-2). Guava's Table-3 spread becomes ELAI's own measured, CI-carrying, per-local-model number.
2. CONSUME ON THE PROFILE: ModelCapabilityProfile (context_budget.rs:292 — the SAME struct J-34 extends with fill curves) gains tool_call_adherence: Option<f32> (absent = today's behavior; no field = no gate).
3. ROLE-GATE (advisory, default-ON kill-switch): the role-router (PerRoleRuntime / RuntimeOrchestrator::try_complete, runtime_orchestrator.rs) consults tool_call_adherence when assigning a TOOL-HEAVY role (executor/researcher — roles that fire many tool calls); when the chosen local model is below a named const TOOLCALL_ADHERENCE_FLOOR it PREFERS a higher-adherence resident model, else emits a FloorBreached decision and (per the I-7 cascade) advises cloud-spill / warns — it does NOT hard-kill the run (the grammar mask still rescues format, so a sub-floor model degrades, it does not crash). Disable via ELAI_TOOLCALL_ADHERENCE_GATE_DISABLE (kill-switch, default-ON — W-2/R-f, NOT a new ELAI_*_ENABLE).

HARD boundaries (negative space): the bench measures SEMANTIC adherence (right tool/args) under the mask, NOT format validity (XGrammar already guarantees that — re-measuring it is redundant and would over-credit the local lane). NOT constrained decoding on reasoning roles (HD-3 ban stands; the mask is tool-call-format-only). NOT a hard block (advisory + cascade, W-2 graceful default). NOT an LLM-judge score (TD-3 — deterministic ground-truth (tool,args) match only). NOT a claim that ELAI reproduces Guava's exact percentages (different domain; Guava is the PHENOMENON anchor, ELAI measures its own).

## Existing ground truth (PLAN-asserted from FEATURE-MAP 2026-06-18 / J-34; executor re-greps per W-7)

- B-10 ModelCapabilityProfile WIRED (context_budget.rs:292 per J-34) — the descriptor struct; J-34 adds fill_degradation_curve/positional_value_curve, J-51 adds tool_call_adherence (parallel additive fields, disjoint legs).
- PerRoleRuntime + RuntimeOrchestrator + resolve_backend WIRED (B-37 / phase-4M; RuntimeOrchestrator::try_complete runtime_orchestrator.rs:185 per FEATURE-MAP) — the role-assignment seam the gate hooks.
- XGrammar tool-call mask SHIPPED (PYTHON SIDECAR, server.py LogitsProcessor chain incl XGrammar mask per FEATURE-MAP) — the format mitigation the bench runs UNDER; do NOT re-implement, do NOT bypass.
- RouteDecisionEvent (E-31, cloud_mode.rs; CloudRouteChosen/TierModelSwapped/LocalRouteKept) — the existing route-decision event; the FloorBreached swap can reuse TierModelSwapped, OR J-51 owns the adherence event (see SSB note).
- I-7 EscalationCascade WIRED (local->cloud waterfall) — the cascade the advisory swap rides; do NOT re-implement.
- HD-3 (arXiv:2408.02442): constrained decoding degrades reasoning — the rule that scopes the mask to tool-call format only.
- J-34 context_calibration.rs + `elai bench context-fill` (PLANNED) — the bench-arm + metrics-TOML + model-identity-keyed-artifact + G-10-invalidation SHAPE J-51's bench MIRRORS (parallel, disjoint metric).

## REVISED scope (W-10 tiered)

Tier-1: legs 1+2 for the LOCAL lane (the bench costs only local inference) + the advisory role-gate (leg 3) consuming the profile field only when present. The cloud models in Guava's table are a reference point, not a thing ELAI must re-bench (cloud tool-call adherence is high and not the risk).
Tier-2: per-role-class adherence curves (needs row volume); auto-recalibration on a G-10 model-identity change (until G-10: manual `elai bench tool-call-adherence --recalibrate` on model swap); a journal-regression leg refining the fixture number with real production tool-call outcomes once those rows carry (model, role, correct?) keys.

## Integration

- NEW: crates/orchestrator/src/toolcall_adherence.rs (~160 LOC: adherence-score type, fixture runner under the masked path, model-identity-keyed artifact store, floor lookup) + crates/cli bench arm `tool-call-adherence` (mirrors J-34's `context-fill` arm shape, args_bench.rs)
- MODIFY: context_budget.rs (ModelCapabilityProfile += tool_call_adherence: Option<f32>); runtime_orchestrator.rs (the advisory floor consult in try_complete role assignment)
- SSB: +ToolCallAdherenceMeasured (Pattern-2 single-variant; Kind: AdherenceMeasured / RoleAdvised / FloorBreached) — measurement + the two consumption decisions all journal-visible (a floor the router used but never logged = R1 FAIL). baseline+1; SCHEMA_VERSION COMPUTED (`-309 + SSB_ACTION_TYPES.len()`, ssb.rs:1500, auto-follows); full 9-tuple lockstep; re-grep BOTH at edit (W-9).
- frontend_pair: a per-local-model adherence tile on the model/cost-cockpit panel family (the same "believe-the-number" family J-34's curve tile joins — "this model picks the right tool X% of the time"); #[tauri::command] + ts-rs pair; cross_lang_diff.py PASS.

## WIRING_CONTRACT

produces_symbols:
  - crates/orchestrator/src/toolcall_adherence.rs::ToolCallAdherenceBench (fixture runner under the masked path; deterministic (tool,args) ground-truth scoring; model-identity-keyed artifact)
  - ModelCapabilityProfile.tool_call_adherence: Option<f32> (context_budget.rs)
  - the advisory floor consult in RuntimeOrchestrator::try_complete (runtime_orchestrator.rs)
  - ToolCallAdherenceMeasured SSB action_type (Kind: AdherenceMeasured / RoleAdvised / FloorBreached)

consumes_symbols:
  - B-10 ModelCapabilityProfile (context_budget.rs:292 — extend the struct; J-34 also extends it, disjoint fields) — GD-1
  - B-37 PerRoleRuntime / RuntimeOrchestrator::try_complete (the role-assignment seam) — GD-1
  - sidecar XGrammar tool-call mask (server.py LogitsProcessor — the bench runs UNDER it; never bypass, never re-implement) — GD-1
  - I-7 EscalationCascade (the cloud-spill advisory rides the existing waterfall) — GD-1
  - RouteDecisionEvent (E-31) is the precedent route-event; J-51 owns ToolCallAdherenceMeasured for the adherence-specific reason (the floor value), reused-RouteDecisionEvent loses that field

reachable_from:
  - `elai bench tool-call-adherence --model <local_model>` (CLI entrypoint)  [PRODUCTION CLI bench surface]
  - RuntimeOrchestrator::try_complete role assignment on a real task run (the FloorBreached/RoleAdvised path)  [PRODUCTION role-routing seam]
  - ToolCallAdherenceMeasured rows + the metrics TOML are the off-agent artifacts; a #[cfg(test)] caller never satisfies this (R7)

stub_ban:
  - NO LLM-judge scoring — adherence is a deterministic match against known (tool, args) ground truth (TD-3)
  - NO re-measuring FORMAT validity as adherence (XGrammar guarantees format; adherence = SEMANTIC tool/args correctness under the mask)
  - NO bypassing the grammar mask in the bench (the fixture runs the REAL masked path or it measures the wrong thing)
  - NO hard kill on a sub-floor model — advisory swap + cascade + WARN; the run degrades, never crashes (W-2 graceful)
  - NO new ELAI_*_ENABLE — the gate is default-ON behind ELAI_TOOLCALL_ADHERENCE_GATE_DISABLE (W-2/R-f); absent profile field = today's behavior
  - NO fabricated adherence numbers (ID-5); NO .unwrap()/.expect() on prod paths; NO blake2/sha2; NO silent fallback (a no-artifact model logs the fallback)

frontend_pair:
  per-local-model adherence tile on the model/cost-cockpit panel family, fed by ToolCallAdherenceMeasured rows; #[tauri::command] + ts-rs + cross_lang_diff.py PASS. Justified: the adherence number is the operator's "is this local model safe for tool-heavy roles" instrument (the believe-the-number surface, same family as J-34's curve tile).

observable_effect:
  trigger: "elai bench tool-call-adherence --model qwen2.5-coder-3b   (then a real task run assigning a tool-heavy role)"
  expect_ssb: "ToolCallAdherenceMeasured::AdherenceMeasured row carrying the model identity + measured score (recomputable byte-identically from the journal under ELAI_SEED, GD-2); on a subsequent run that assigns a tool-heavy role to a sub-floor model, a RoleAdvised or FloorBreached row carrying the model + measured score + TOOLCALL_ADHERENCE_FLOOR"
  expect_artifact: "the per-model adherence metrics TOML + the panel tile rendering the score; NEGATIVE (R2): a model with NO adherence artifact uses today's role assignment AND logs the no-calibration fallback (no silent pseudo-gate); the gate disabled via ELAI_TOOLCALL_ADHERENCE_GATE_DISABLE logs that it is disabled (not a silent no-op); a broken arm scoring via LLM-judge instead of (tool,args) match FAILS the contract test (TD-3)"

## Grounding

- Guava arXiv:2606.18363 Appendix D / Table 3 (identical-harness multi-model spread: Gemini-3.1-Pro 89.8, Claude-Sonnet-4.6 88.9, GPT-5.4 70.2, Qwen3.5-2B 2.2; capability floor ~2-4B for tool-call adherence). VERIFIED at authoring (workflow/papers/guava-interface.md; watermark arXiv:2606.18363v1 [cs.RO] 16 Jun 2026). Scope caveat (GD-5/HD-1): EMBODIED-MANIPULATION numbers; ELAI borrows the PHENOMENON (small models drop on tool-call adherence; there is a floor) and measures its OWN per-model number on its OWN fixtures — Guava's percentages are directional, never claimed as ELAI's.
- HD-3 (arXiv:2408.02442): constrained decoding degrades reasoning — scopes the XGrammar mask to tool-call format only; the bench measures the residual the mask does not cover.
- ELAI ground truth (FEATURE-MAP 2026-06-18, PLAN-asserted, re-grep W-7): B-10 ModelCapabilityProfile WIRED; B-37 PerRoleRuntime/RuntimeOrchestrator WIRED; XGrammar sidecar mask SHIPPED; I-7 cascade WIRED; J-34 bench/profile/artifact shape (PLANNED, the mirror).
- house ordering 7B>3B>2B for tool-call reliability (memory project_qwen3b_baseline / project_vram_tier_plan) — Guava Table 3 is the external confirmation; the 3B-NF4 borderline is why the floor gate matters.
- SSB: +ToolCallAdherenceMeasured; SCHEMA_VERSION COMPUTED -> baseline 268 / TOTAL_VARIANTS 577 as of LIVE re-grep 2026-06-16 (J-0_INDEX constants); NEVER hard-code, re-grep BOTH at edit (W-9/R-c).
- CLAUDE.md: blake3 only, no .unwrap in prod, scoped -p tests, W-2 kill-switch-not-enable, ID-5 no fabricated stats, TD-3 LLM-advisory-never-gate.

## COMPOSES-WITH

- B-10 ModelCapabilityProfile: ADD a field; do NOT re-define the struct (J-34 also extends it — coordinate field additions, disjoint metrics).
- B-37 PerRoleRuntime / RuntimeOrchestrator: hook the advisory consult into try_complete; do NOT re-implement role routing or the runtime adapter.
- sidecar XGrammar mask: run the bench UNDER it; do NOT re-implement or bypass the mask.
- I-7 EscalationCascade: the advisory cloud-spill rides the existing waterfall; do NOT add a second escalation path.
- J-34 EMPIRICAL CONTEXT CALIBRATION: mirror the bench->profile->artifact->G-10-invalidation shape; do NOT merge the metrics (context-fill and tool-call-adherence are orthogonal per-model numbers).

## Riders (recorded, not in-scope)

- Publishing a per-local-model adherence table = K-1 claims-policy material once numbers exist; recorded.
- Per-role-class adherence (executor vs researcher fire different tool mixes) = Tier-2, needs row volume; recorded.
- Using adherence to drive the J-48 LocalServingRouter tier choice (don't warm a sub-floor model for a tool-heavy queue) = a J-48 rider once both ship; recorded.

End of J-51.
