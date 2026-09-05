I-36 cross-model prompt compilation — decisions

WHAT
Build per-(role,backend) compiled-prompt artifact store. CompiledPromptVariant / CompiledPromptStore /
promote_variant / rollback_on_regression. Emit PromptCompiled SSB rows. Backend substitution wired.
Per-role badge in SettingsWindow. GEPA offline optimizer DEFER behind ELAI_PROMPT_COMPILE_DISABLE.

KEY DECISIONS

D-1 WIRING SEAM: Option A (controller-confirmed, A187). Substitution at the providers adapter
boundary (anthropic_api.rs::complete), mirroring apply_anthropic_cache_markers at :232. Added
compiled_system_override: Option<String> to CompletionConfig (providers types.rs). The lookup runs
in the orchestrator/CLI layer (report_fns.rs build_system_prompt_for_role -> CompiledPromptStore::lookup)
and the resolved string is placed in compiled_system_override before passing the config to the adapter.
The adapter then prefers compiled_system_override over system in build_request. This avoids a circular
dep (providers cannot import AgentRole from orchestrator).

D-2 KILL-SWITCH POLARITY: ELAI_PROMPT_COMPILE_DISABLE is a DISABLE flag. The substitution path is
DEFAULT-ON (pass-through when no compiled prompt is stored, i.e. no-op). The DISABLE flag kills the
substitution entirely (stores are not loaded). The GEPA offline optimizer (which produces compiled
prompts) is Tier-2 DEFER. Flag type = kill_switch (ops, may be long-lived). This is consistent with
the plan note "Automated per-backend optimizer runs ... Tier-2 behind ELAI_PROMPT_COMPILE_DISABLE".

D-3 STORAGE KEY: (role: String, backend: String) keyed BTreeMap mirroring CompiledPipelineStore's
(String, u64) BTreeMap. Role and backend are snake-case strings consistent with agentRole + RuntimeBackend
display. No new ModelFamily type; uses existing RuntimeBackend string representation.

D-4 PROMOTE_VARIANT WIRING: promote_variant() is WIRE-DARK[I-36]. The darwin two_gate_promote
machinery is complex; threading a CompiledPromptVariant through the PromotionConfig/DescriptorEntry
bridge requires a Tier-2 task. A WIRE-DARK[I-36] marker is placed on promote_variant's pub fn
definition with a stubs.md row. The lookup + substitution (Tier-1) and rollback_on_regression are
genuinely wired.

D-5 TRANSFER GAP HARNESS: The measurement struct (TransferGapResult) is WIRE-DARK[I-36]. A live
comparison run against a real backend is budget-gated Tier-2. The SSB PromptCompiledKind::TransferGapMeasured
arm is registered and the payload struct is complete; the emitter is a dark pub fn.

D-6 SCHEMA: Adding PromptCompiled adds ONE entry to SSB_ACTION_TYPES. SCHEMA = -309 + 577 = 268
after this change. golden file adds one line "prompt_compiled" (alphabetically between "plan_state_derived"
and "prm_canary_injected"). Derived automatically; no hand-bump.

EXPECTED TEST OUTCOME
- cargo check --workspace EXIT 0
- action_type_lock regen PASS (+1 line "prompt_compiled" in golden)
- i36 contract: a-d PASS
- e45 cross-lang drift contract PASS (Tauri command + Svelte consumer both present)

UNKNOWNS
- promote_variant Tier-2: full ledger threading deferred (stubs.md row)
- live transfer-gap measurement: deferred until cloud optimizer budget
