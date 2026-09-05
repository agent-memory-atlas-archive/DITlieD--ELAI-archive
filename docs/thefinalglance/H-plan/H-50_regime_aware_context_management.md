# H-50 -- regime-aware in-session context management (local unload/load vs cloud cache-stable; one window truth)
- Track 9 (CONTEXT LIFECYCLE) | lever CHEAPER/STEERING | grounding STRONG | nature BUILD-on-shipped (window-truth fix + route the SHIPPED ladder/masking into the interactive local turn) | effort M-L | WAVE 9 | composes H-1/H-2 (cloud caching, leave intact), H-6 (deterministic masking + recover_stubs), H-35/H-49 (SessionDag + selective recovery + RV-2 lossless-view)
- Premise: a local model (window 25-32K, NO cross-request cache in production) and a cloud model (200K, caching live) need OPPOSITE context policies, and ELAI runs ONE cloud-shaped policy for both. The result, grounded this session: a long LOCAL interactive session is effectively unprotected -- nothing shrinks the re-sent context before the window overflows, and it eventually dies at a hard context_window_blocked error instead of gracefully unloading. H-50 makes in-session context management regime-aware: ONE model-window source of truth feeding window-derived trim thresholds, a LOCAL unload/load policy (drop->mask->summarize at ~70-75% of the REAL window, keep stubs unloaded, reload selectively on demand) and a CLOUD keep-the-cache-stable-prefix policy, selected by BackendMode. Full rationale: docs/thefinalglance/CONTEXT-LIFECYCLE-REGIME-AWARE-AND-HANDOFF-DESIGN-2026-06-05.md.

## Steal/build
Prior art (WebSearch-verified 2026-06-05): Anthropic effective-context-engineering (Sep 2025) -- the policy must match the window; compaction + structured note-taking + clean windows. vLLM automatic prefix caching vs HuggingFace transformers -- HF transformers has NO native cross-request KV reuse (the sidecar's lever is window-minimization, not caching). The economics: cloud caching amortizes a 1.25x write over many 0.1x reads, so cloud WANTS a big stable prefix; local has nothing to amortize and no room, so it must MINIMIZE the prefix. These are opposite, and the regime must pick the policy.
Reuse-from-shipped (this is mostly WIRING, not invention -- GD-1): adaptive_reduce + compaction_ladder already exist (the ladder is even default-ON, just wired to the autonomous path); recover_stubs already does selective recovery by stub.ssb_seq; H-6 is the deterministic-masking activation. H-50 = fix the window truth + route the ladder/masking into the interactive LOCAL turn + make recovery SELECTIVE + leave the cloud caching path intact.

## Existing ground truth (code-read 2026-06-05, file:line confirmed via subagent greps)
- FLAT auto-compact: `orchestrator/src/conversation.rs:22` DEFAULT_AUTO_COMPACTION_INPUT_TOKENS_THRESHOLD = 100_000 (set once from one env var, conversation.rs:238/924, never model-derived); `conversation.rs:789 maybe_auto_compact` compares cumulative input tokens to it. For a 32K local model this NEVER fires before overflow. DEAD for local.
- WRONG-WINDOW handoff gate: `cli/src/live_cli.rs:1238` computes model_window * 0.85, but model_window = `cli/src/cli_consts.rs:20-26 max_tokens_for_model` = 64_000 for ANY non-"opus" model name, IGNORING `ELAI_LOCAL_MAX_CONTEXT`. So a Qwen local gets a ~54.4K trigger, ABOVE its 32K window. And it disagrees with the `BudgetPolicy::Local` window (`cli/src/runtime_client.rs:338`, from ELAI_LOCAL_MAX_CONTEXT) which only caps OUTPUT + sizes slots, never trims input.
- RE-INFLATE defeats the trim: even when the gate fires, the SSB-present path calls `session_dag.rs:1551 recover_stubs` which restores every stubbed tool-result to FULL bytes before the model call -> net live trim ~0. Only User/Assistant text is never stubbed; tool stubs are restored.
- DEAD reducers: `adaptive_reduce` (context_budget.rs:1525) has ZERO production callers; `compaction_ladder` (compaction_ladder.rs:55, default-ON kill-switch) runs only in `task_runner.rs:9604` (autonomous PreCompact, over a summary string), NOT the interactive REPL. The only live local input shrinkage is ShortCoder code-text compression (~15-30%, code blocks only, runtime_client.rs:360).
- HARD-ERROR backstop: `providers/src/providers/mod.rs:257` preflight reject -> context_window_blocked. So local fails closed with an error, not graceful compaction.
- CLOUD caching is LIVE (leave it): `providers/src/providers/anthropic.rs:1094 apply_anthropic_cache_markers` at all 3 dispatch sites incl `runtime/anthropic_api.rs:219` (the H-1 split-brain is REFUTED -- None set at :137, re-stamped at :219); 1h TTL `types.rs:93 ephemeral_1h` gated on a byte-stable prefix (runtime_client.rs:509-535); guard `session_dag.rs:20/934 PROMPT_CACHE_PREFIX_BOUNDARY=3` keeps trim from busting the prefix. SCHEMA_VERSION=224 / TOTAL_VARIANTS=501 (ssb.rs:3010 / action_type_lock.rs:645), 2026-06-05; re-read at edit (W-9).

## REVISED scope (W-10 tiered)
SHIP NOW (Tier-1, the vertical slice -- producer + consumer + smoke together, R6):
- ONE window truth: `effective_context_window(backend)` (local = ELAI_LOCAL_MAX_CONTEXT / the BudgetPolicy::Local ceiling; cloud = the cloud ceiling; api = the model's real window). Replace the flat 100_000 (conversation.rs:22) AND the 64K-non-opus gate (cli_consts.rs:20) with `trim_at = floor(effective_window * TRIM_FRACTION)` (local ~0.70-0.75, cloud ~0.85). Both maybe_auto_compact and the live_cli handoff gate read this one value.
- `ContextPolicy` chosen by BackendMode. LOCAL: at trim_at, run the cheapest-first ladder IN the interactive turn (drop dead trailing observations via H-6 masking -> mask/stub bulky tool-results -> compact_session the oldest turns) and DO NOT auto-recover; `recover_stubs` becomes SELECTIVE (called only for a specific stub the model asks to re-read). CLOUD: keep the cache-stable prefix, compact only near the real window, preserve the PROMPT_CACHE_PREFIX_BOUNDARY guard -- no behavior change to the live caching path.
- Measured off-agent proof: the model call after a local trim carries FEWER input tokens than before (from the journal), a ContextReduction row lands, and the session that previously hit context_window_blocked now completes.
DEFER (Tier-2): the OPTIONAL local free-lunch -- unbypass the sidecar's own PrefixKVCache (`python/elai_inference/server.py:1819`, real cross-request cache that already honors cache_control but is skipped on the streaming path at server.py:2320 while the live client sends stream:true at runtime_client.rs:547): teach generate_streaming to lease()/admit() OR route local above a size threshold to the non-streaming endpoint. Output-preserving by construction (assert byte-identical generations with/without). This reduces COMPUTE, not token count -- §5.1/§5.2 are still primary. Behind a kill-switch.

## Integration
- `effective_context_window` + `ContextPolicy` (new, small) in the orchestrator context layer; consumed by `conversation.rs:789` (replace the flat threshold) and `cli/src/live_cli.rs:1238` (replace max_tokens_for_model in the gate). Route the LOCAL ladder through the existing `compaction_ladder.rs` + H-6 masking + `compact.rs:96 compact_session`; make the `live_cli.rs:1317` recover_stubs call SELECTIVE (drop the unconditional whole-set re-inflate for local). Leave `anthropic.rs`/`runtime_client.rs` cloud caching untouched.
- frontend_pair: a context-budget gauge per regime + an "unloaded N items (reloadable)" indicator in the dashboard (CommandWindow / a small ContextPanel); cross_lang_diff PASS.
- SSB (W-9): REUSE the existing `ContextReduction` payload for unload events (GD-1, no new variant); add a Pattern-2 `WindowPolicyApplied` ONLY if the gate needs a distinct off-agent regime-decision row (author decides; prefer reuse, minimise bumps). If added, baseline+1 from the live 224/501.
- Default-ON + kill-switch `ELAI_CONTEXT_POLICY_DISABLE` (W-2). The §Tier-2 sidecar cache-unbypass ships behind its own kill-switch.

## WIRING_CONTRACT
- produces: a `ContextReduction` SSB row on each local unload (reused payload); the post-trim model request carrying FEWER input tokens than the pre-trim request (the measurable off-agent artifact); optional `WindowPolicyApplied` row.
- consumes: `effective_context_window`/BudgetPolicy (runtime_client.rs:338), `compaction_ladder` (compaction_ladder.rs:55), `adaptive_reduce` (context_budget.rs:1525, activate), `compact::compact_session` (compact.rs:96), `session_dag::recover_stubs` (session_dag.rs:1507, now selective), H-6 masking.
- reachable_from: REAL entrypoint = the interactive turn loop `live_cli.rs prepare_turn_runtime` (every local turn) + `conversation.rs maybe_auto_compact`; NOT a test shim (R3/R7).
- stub_ban: no placeholder trim; the local path MUST actually reduce the re-sent message tokens (a trim that nets ~0 after re-inflate is the CURRENT bug and is a FAIL); recover_stubs MUST be selective on local (an unconditional whole-set re-inflate = FAIL); no `.unwrap()` in the policy code.
- observable_effect (W-1 done-gate, a LIVE trigger leaving an off-agent artifact; missing = FAIL per R1/R2): drive a real local session (ELAI_LOCAL_MAX_CONTEXT=32000, seeded fixture) past trim_at; (a) a ContextReduction row lands that the executing agent did not author, (b) the NEXT model request's input-token count (from the journal) is < the window AND < the pre-trim count, and (c) a control run with the policy disabled hits context_window_blocked while the enabled run COMPLETES. CL-4: a "caching on" claim is FAIL if the journal shows cache_read=0 on that path (local streaming = 0 today). GD-4: the false-drop rate (context the model still needed) is bounded on >=3 runs before default-ON.

## Grounding
- Prior art (WebSearch 2026-06-05): Anthropic context engineering (anthropic.com/engineering/effective-context-engineering-for-ai-agents); vLLM APC vs HF transformers no-native-reuse (docs.vllm.ai/en/stable/design/prefix_caching/). Borrowed numbers are [DESIGN], re-measured per HD-1.
- ELAI source (subagent greps 2026-06-05): conversation.rs:22/789, cli_consts.rs:20-26, live_cli.rs:1238/1317/1551(via session_dag), session_dag.rs:1507/1551/20/934, runtime_client.rs:338/360/509-535/547, compaction_ladder.rs:55, context_budget.rs:1525, compact.rs:96, anthropic.rs:1094, anthropic_api.rs:137/219, types.rs:93, providers/mod.rs:257, server.py:1819/2320/3816, ssb.rs:3010, action_type_lock.rs:645. Full design + 2-approach analysis (regime-aware vs vLLM-swap): the design doc above.
- HONEST RESIDUE (CL-4 / GD-5): local caching stays a no-op until the Tier-2 unbypass ships; do not advertise a local cache hit before then. Caching reduces cost-per-token, not token-count -- a small window is still small; window-minimization is the primary local lever.

## COMPOSES-WITH
- H-1/H-2 (cloud caching): confirmed LIVE; H-50 leaves it intact (the cloud half of the regime split).
- H-6 (deterministic masking + recover_stubs + adaptive_reduce activation): H-50 is WHERE H-6's masking + adaptive_reduce actually run in the interactive local turn at a window-derived threshold -- shared work, implement once.
- H-35 / H-49 (SessionDag, recover_stubs, RV-2 lossless-view): the selective-recovery + keep-unloaded discipline is shared.
- H-51 (cross-session handoff): H-50 is the in-session base H-51's tiered load sits on; both share the selective recover_stubs.
- ADDS CL-1..CL-4 (canon: the design doc + H-0_INDEX).

## J-11 AMENDMENT — S-5 TOKEN-EFFICIENCY CONTEXTMANAGER LAYERS (source-cited)
Grounding: Arbor `src/coordinator/context.py:255-334` (ContextManager), `src/coordinator/context_prune.py:1-219`
(ContextPruner), read directly per W-3.
Arbor's ContextManager is a working existence proof of the layered context-pruning pattern H-50's LOCAL
unload/load policy needs. Three stealable mechanics (GD-1 prior-art pointer, compose not restate — H-50 owns
the regime-aware policy):

### 1. Three-tier observation pruning (`context_prune.py:1-219`)
Arbor's ContextPruner applies three tiers in order (`context.py:295` prune_context):
- **Tier 1 — drop old observations** (`context_prune.py:40` _prune_old_observations): remove observations
  older than a configurable window, keeping only the N most recent.
- **Tier 2 — truncate long observations** (`context_prune.py:90` _truncate_long_observations): cap each
  observation at a max character count, keeping head + tail with a truncation marker.
- **Tier 3 — summarize compacted observations** (`context_prune.py:150` _summarize_observations): replace
  the oldest observations with a single summary line.

This maps to H-50's cheapest-first ladder: drop dead trailing observations (H-6 masking) → mask/stub bulky
tool-results → compact_session the oldest turns. The ORDER matters — cheapest first, most expensive last.
Arbor's implementation is Unix-only and single-threaded; H-50's is regime-aware and runs in the interactive
turn. Compose the tier ordering, do NOT restate the implementation.

### 2. Token-count-aware pruning (`context.py:310` _estimate_tokens)
Arbor estimates token count via a simple heuristic (chars/4) and prunes to a target budget
(`context.py:315` target_tokens). H-50 already has the real token count from the journal
(`observable_effect`: "the NEXT model request's input-token count from the journal") — use the REAL count,
not Arbor's chars/4 heuristic. The heuristic is a fallback for when no tokenizer is available; ELAI always
has the journal count.

### 3. Protected content preservation (`context_prune.py:190` _is_protected)
Arbor marks certain content as protected (system prompts, current task description, recent observations
within the window) and never prunes it. H-50's PROMPT_CACHE_PREFIX_BOUNDARY guard (session_dag.rs:20/934)
is the analog for the CLOUD path — the cache-stable prefix is protected. For the LOCAL path, the system
prompt + rules + current phase spec are protected (the Context Budget Allocator's floors, context_budget.rs).
Compose with the existing allocator floors; do NOT re-implement the protection logic.

NOTE: Arbor's ContextManager runs in a persistent 500-turn ReAct loop (M-1 DO-NOT-PORT). H-50's runs in the
interactive REPL turn. The pruning MECHANICS are stealable; the loop shape is not.
