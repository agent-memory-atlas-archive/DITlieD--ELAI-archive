# DEEP-H-006 exhaustive feature discovery

## Scope, method, and integrity

- Chunk: `DEEP-H-006`, family H, manifest section `DEEP-SOURCE-MANIFEST.md:742-757`.
- Permitted orientation only: `08-H-SUMMARY_INDEX.discovery.md`, specifically its global doctrine, the ten matching H entries, and its dependency/configuration/journal/correction inventories. It is not counted as an original source.
- Original-source boundary: exactly the ten manifest-listed files below were read completely; no other original H chunk, grounding map, plan index, code file, design document, or cited web source was opened.
- Extraction rule: every ledger atom below is sourced directly from one of those ten files. Cross-file deductions live only in **Inferences and interactions**. Unresolved wording lives only in **Ambiguities and conflicts**.
- Classification: `E-*` = explicit; `I-*` = inferred cross-source composition; `A-*` = ambiguous; `C-*` = conflict/correction. Level `M` = major capability/invariant, `m` = minor/supporting behavior.
- Category vocabulary: capability/action/surface; workflow/state; schema/data/lifecycle; flag/default/config; integration/event; permission/security/privacy/compliance; failure/fallback/retry/recovery; determinism/idempotency/concurrency; observability/evidence; test/acceptance/done-gate; constraint/non-goal; dependency/status/deferred.
- Anchor form is `filename:line` or an inclusive line span. Anchors refer to the verified source snapshot recorded in the manifest table.

## Exact processed manifest and verification

| # | Manifest class | Lines | Bytes | Verified SHA-256 | Exact absolute path | Result |
|---:|---|---:|---:|---|---|---|
| 1 | numbered/index original | 187 | 15448 | `e9769453ae7948564d79fd4fa19b77125cddf53f42998bfc1b521d60adf8ddd1` | `docs/thefinalglance/H-plan/H-6_observation_masking_memory_readback.md` | exact |
| 2 | numbered/index original | 229 | 18991 | `15db2f5950891147501f3554b3bb4765ccf8b4d065de97557a56337c6a8b32c9` | `docs/thefinalglance/H-plan/H-7_calibrated_routing_architect_editor_verifier.md` | exact |
| 3 | numbered/index original | 156 | 12103 | `0fbaed9c8aa1dbc2567d23a51d17393fd98fe3ccf2385bf5d1bf482cb24e5da7` | `docs/thefinalglance/H-plan/H-8_persist_oracle_answer_cache.md` | exact |
| 4 | numbered/index original | 196 | 16323 | `5db7b0a2cab23ca6ce9e1d39bd42167c8c395fb9e62329f782a5e4e146ba715f` | `docs/thefinalglance/H-plan/H-17_trial_ledger_keystone.md` | exact |
| 5 | numbered/index original | 169 | 13906 | `b4fe6353a73853013e6b33e73dab7ac538aaff47544d603fbf1fdcd81b66a705` | `docs/thefinalglance/H-plan/H-23_same_seed_rerun_hash_equality.md` | exact |
| 6 | numbered/index original | 182 | 16052 | `0e56d5c00322a65b4824767c75f862226b9e99503cf4a207b460783b3c38e340` | `docs/thefinalglance/H-plan/H-28_reward_tampering_harness_frozen.md` | exact |
| 7 | numbered/index original | 176 | 15510 | `ba6415c663e578116a81b2185e67de524e89231bfdbb0e794ec0db75d601af39` | `docs/thefinalglance/H-plan/H-29_drift_state_machine.md` | exact |
| 8 | numbered/index original | 81 | 20040 | `d19c6343e8a5691c828bee62302d29cf54eb4c18fd8f48d18f8b2e748ba955be` | `docs/thefinalglance/H-plan/H-30_staleness_intent_drift_metric.md` | exact |
| 9 | numbered/index original | 71 | 19556 | `0be01fb08340a04e15d98a15dc5bfbe4d5eda64484731ae079d91657516bd296` | `docs/thefinalglance/H-plan/H-36_completion_needs_input_notifications.md` | exact |
| 10 | numbered/index original | 48 | 11347 | `5bb488b582e5f87bdcaab6594fb45d22515e28f51988e0ddce219efb5c4fbee9` | `docs/thefinalglance/H-plan/H-51_sleeptime_handoff_digest_autodetect.md` | exact |
| **Total** |  | **1495** | **159276** |  | **10 files** | **matches manifest** |

Verification used byte-for-byte `sha256sum` plus `wc -l -c`; every per-file value and both totals match the manifest. “Logical lines” in this chunk equal newline-counted lines.

## Per-file explicit ledgers

### H-6 — deterministic observation masking and memory read-back

| Atomic ID | Level | Explicit type/categories | Feature or behavior | Exact anchor |
|---|:---:|---|---|---|
| H006-H6-001 | M | E-STATE; dependency/status | Plan is `PLANNED`, Wave 1, in-process activation of three existing-but-never-called symbols. | `H-6_observation_masking_memory_readback.md:3-14` |
| H006-H6-002 | M | E-LIMIT; evidence/constraint | The cited masking result is a literature claim (about 50.9–56.1% cost reduction while matching summarization), while ELAI has no LLM compaction to replace: `compact_session` is deterministic. | `H-6_observation_masking_memory_readback.md:24-32` |
| H006-H6-003 | M | E-STATE; workflow/observability | `adaptive_reduce` has three levels (`FilteredByRelevance`, `ReducedGranularity`, `ReducedScope + Terminal`), only test callers, and a write-capable but call-dark `ContextReduction` path. | `H-6_observation_masking_memory_readback.md:34-36` |
| H006-H6-004 | M | E-STATE; flag/default/workflow | `run_compaction_ladder` has a real `PreCompact` caller but is double-gated by `compaction_ladder_enabled()` and its own short circuit; `ELAI_E25_COMPACTION_LADDER_ENABLE` is default-off. | `H-6_observation_masking_memory_readback.md:37-39` |
| H006-H6-005 | m | E-STATE; capability/constraint | `ALL_COMPACTION_TIERS` already supplies six deterministic cheapest-first tiers, but current per-tier reductions are estimators rather than working-set mutations. | `H-6_observation_masking_memory_readback.md:39-40` |
| H006-H6-006 | M | E-STATE; security/data/lifecycle | `get_relevant_facts` is production-dark, trust/provenance filtered, exported, test-only, and bumps `last_observed_at_ms` on read. | `H-6_observation_masking_memory_readback.md:41-42` |
| H006-H6-007 | M | E-STATE; recovery/security | `recover_stubs`/`recover_single_stub` are production-dark but perform real blake3-verified SSB recovery and return `StubRecoveryFailed` for missing/corrupt content. | `H-6_observation_masking_memory_readback.md:43-44` |
| H006-H6-008 | m | E-STATE; lifecycle | `create_handoff` is the live stub producer on the 85%-budget branch; production creates stubs that are not read back. | `H-6_observation_masking_memory_readback.md:45` |
| H006-H6-009 | m | E-STATE; surface/workflow | `compact_session` is already live through `/compact` and `--resume`, whereas the live read-back at `live_cli.rs:1252-1264` inserts a literal `[stubbed]` placeholder. | `H-6_observation_masking_memory_readback.md:46-47` |
| H006-H6-010 | M | E-REQ; capability/default/determinism | Add a non-test production caller so `adaptive_reduce` becomes the default long-session manager once a budget threshold is exceeded. | `H-6_observation_masking_memory_readback.md:61-64` |
| H006-H6-011 | M | E-REQ; lifecycle/cache/determinism | Reduction must drop trailing observations only, never rewrite the stable prefix; dropped observations remain in SSB and re-fetchable. | `H-6_observation_masking_memory_readback.md:61-64` |
| H006-H6-012 | M | E-REQ; integration/event/schema | Emit the existing `ContextReduction` payload; creating a new action type is forbidden. | `H-6_observation_masking_memory_readback.md:64,89-94,147` |
| H006-H6-013 | M | E-REQ; flag/default | Promote the E-25 ladder off default-off; terminal behavior is default-on/kill-switch-only. | `H-6_observation_masking_memory_readback.md:66-70,102,130,140` |
| H006-H6-014 | M | E-REQ; capability/honesty | The cheapest applicable ladder tier must actually mutate the working set by dropping trailing observations, or be explicitly labeled estimate-only; it may not advertise an undelivered reduction. | `H-6_observation_masking_memory_readback.md:66-70` |
| H006-H6-015 | M | E-REQ; recovery/surface | Replace the live `/continue` placeholder with a `recover_stubs` call that rehydrates real content. | `H-6_observation_masking_memory_readback.md:72-74` |
| H006-H6-016 | M | E-REQ; failure/recovery | Missing/corrupt SSB must surface as a visible recovery failure; silent placeholder fallback is prohibited. | `H-6_observation_masking_memory_readback.md:72-74,99-104` |
| H006-H6-017 | M | E-REQ; integration/security | Add a non-test caller of `get_relevant_facts` on a memory-relevant turn, preserving its trust/provenance filtering and read timestamp bump. | `H-6_observation_masking_memory_readback.md:76-77` |
| H006-H6-018 | m | E-REQ; evidence/status | Create `workflow/reasoning/H-6-decisions.md` recording the no-LLM-compaction correction, dark symbols, prefix rule, estimator/mutation choice, and fail-closed stub behavior. | `H-6_observation_masking_memory_readback.md:79-81` |
| H006-H6-019 | M | E-REQ; workflow/integration | Production reachability is `task_runner.rs:9379 -> run_compaction_ladder -> ContextReduction` and `live_cli.rs:1252-1264 -> recover_stubs -> real content`. | `H-6_observation_masking_memory_readback.md:95-97` |
| H006-H6-020 | m | E-REQ; surface/integration | Frontend pair is the existing E-25 ladder-state panel through `get_compaction_ladder_state`; `cross_lang_diff` must pass. | `H-6_observation_masking_memory_readback.md:105-106` |
| H006-H6-021 | M | E-TEST; observability/done-gate | A real long session without an override must write `ContextReduction`, preserve H-2 byte-prefix equality, rehydrate real payloads in a fresh process, bump fact observation time, and prove SSB re-fetchability. | `H-6_observation_masking_memory_readback.md:107-114` |
| H006-H6-022 | m | E-REQ; determinism/dependency | The whole gate is Tier-1 in-process with no network; dependencies are shipped E-25, F-5, and the existing `ContextReduction` payload. | `H-6_observation_masking_memory_readback.md:16-19,148` |
| H006-H6-023 | m | E-REQ; security/config | Use blake3 and no production `.unwrap()`/`.expect()`; add no dependency. | `H-6_observation_masking_memory_readback.md:104,135-136` |
| H006-H6-024 | M | E-TEST; acceptance | `wiring_contract_check.py --run-trigger` must pass; grep must prove live callers, placeholder removal, gate promotion, cross-language parity, and decision-doc existence. | `H-6_observation_masking_memory_readback.md:116-135` |
| H006-H6-025 | M | E-TEST; failure/recovery | Contract tests must cover invocation/event emission, trailing-only byte stability, masked-row re-fetch, real recovery plus visible missing-SSB error, memory read bump, and a mutating-or-honestly-estimated cheapest tier. | `H-6_observation_masking_memory_readback.md:151-156` |
| H006-H6-026 | m | E-REQ; integration/status | Planned write set is four production modifications plus a new contract test and decision document, with approximate LOC called out. | `H-6_observation_masking_memory_readback.md:158-164` |
| H006-H6-027 | m | E-RISK; failure/recovery | A later-needed masked observation is the principal risk; passing requires re-fetchability, not merely evidence that masking occurred. | `H-6_observation_masking_memory_readback.md:174-176` |
| H006-H6-028 | m | E-RISK; dependency/observability | H-6 coordinates with H-2 for prefix stability and H-9 for masked-token cost reporting; it is parallel-safe with H-5/H-7/H-8. | `H-6_observation_masking_memory_readback.md:19,182-185` |

### H-7 — calibrated architect/editor/verifier routing

| Atomic ID | Level | Explicit type/categories | Feature or behavior | Exact anchor |
|---|:---:|---|---|---|
| H006-H7-001 | M | E-STATE; dependency/status | Plan is `PLANNED`, Wave 1, Tier-1 in-process BUILD and the heaviest Wave-1 session; role-dispatch work is not parallel-safe with other `task_runner`/`cloud_mode` routing edits. | `H-7_calibrated_routing_architect_editor_verifier.md:3,16-20` |
| H006-H7-002 | M | E-STATE; workflow | `PerRoleRuntime` and `RuntimeOrchestrator` exist but have zero task-runner dispatch references; their current callers are inspectors/bench/dashboard or non-dispatch consumers. | `H-7_calibrated_routing_architect_editor_verifier.md:5-10,35-39` |
| H006-H7-003 | M | E-STATE; flag/default/workflow | `RuntimeRoutingPolicy::evaluate` is default-off, observational, and cannot change `RoutedProvider`; it uses hardcoded `prior_audit_failures:0` and `budget_remaining_usd:25.0`, while VRAM is now real. | `H-7_calibrated_routing_architect_editor_verifier.md:40-43` |
| H006-H7-004 | M | E-STATE; integration/event | `emit_route_decision_log` only logs with `tracing::info`; existing `RouteDecisionEvent` arms `CloudRouteChosen`/`TierModelSwapped`/`LocalRouteKept` have no production `write_payload` emitter. | `H-7_calibrated_routing_architect_editor_verifier.md:44,50` |
| H006-H7-005 | M | E-STATE; workflow/fallback | The only live route leg is `CloudModeRouter::route/route_with_spend` cap enforcement plus `EscalateToCloud`, reached at four task-runner sites; `layer_failed:1` is hardcoded at the live probe. | `H-7_calibrated_routing_architect_editor_verifier.md:45-48` |
| H006-H7-006 | m | E-STATE; dependency/status | `HybridEscalationGate` and `confidence::compute_score` exist but are test-only. | `H-7_calibrated_routing_architect_editor_verifier.md:49` |
| H006-H7-007 | M | E-REQ; capability/workflow | Dispatch reasoning/planning to a flagship-cloud architect, cheaply checkable edits to a cheap/7B editor, and verification to an auditor in an independent context. | `H-7_calibrated_routing_architect_editor_verifier.md:64-67` |
| H006-H7-008 | M | E-REQ; security/correctness | Gate editor output with an off-agent auditor/test signal, never the editor’s self-confidence. | `H-7_calibrated_routing_architect_editor_verifier.md:64-72` |
| H006-H7-009 | M | E-REQ; determinism/workflow | Replace `layer_failed:1` with an externally calibrated RouteLLM/AutoMix-style signal, either by wiring `HybridEscalationGate::evaluate` or building the calibrated layer. | `H-7_calibrated_routing_architect_editor_verifier.md:69-72` |
| H006-H7-010 | M | E-REQ; workflow/data | Populate `RoutingContext` with live audit-failure and budget data, and make the resulting `RoutingDecision` change the dispatched provider. | `H-7_calibrated_routing_architect_editor_verifier.md:74-76` |
| H006-H7-011 | M | E-REQ; integration/event | `emit_route_decision_log` must write the existing `RouteDecisionEvent` to SSB; redefining that action type is prohibited. | `H-7_calibrated_routing_architect_editor_verifier.md:78-79,93-98,106` |
| H006-H7-012 | M | E-REQ; flag/default | Promote `ELAI_ROUTE_DECISION_SSB_ENABLE` off default-off; routing and evidence become default-on with kill-switch semantics only. | `H-7_calibrated_routing_architect_editor_verifier.md:78-80,107,133-135` |
| H006-H7-013 | M | E-REQ; cost/correctness | Route jointly on cost and correctness: a cheap call that fails its off-agent check and triggers cloud retry is not a saving. | `H-7_calibrated_routing_architect_editor_verifier.md:25-33,81-86` |
| H006-H7-014 | M | E-REQ; observability/default | Measure per-step-class pass rate and bound cheap-edit→cloud-retry over-escalation on at least three real runs before default-on. | `H-7_calibrated_routing_architect_editor_verifier.md:81-83,137` |
| H006-H7-015 | M | E-LIMIT; constraint/non-goal | Do not use BSP or parallel code writers; the second cloud call is an independent verifier, not another writer. | `H-7_calibrated_routing_architect_editor_verifier.md:30-33,102-103` |
| H006-H7-016 | m | E-LIMIT; evidence | The cited +9.6 percentage-point Aider result and RouteLLM/AutoMix figures are prior art, not ELAI measurements; ELAI’s pair must be calibrated locally. | `H-7_calibrated_routing_architect_editor_verifier.md:23-33,172-187` |
| H006-H7-017 | m | E-REQ; evidence/status | Create `workflow/reasoning/H-7-decisions.md` recording the external-signal, no-BSP, real-context, cost/correctness, over-escalation, and wire-or-replace decisions. | `H-7_calibrated_routing_architect_editor_verifier.md:84-86` |
| H006-H7-018 | M | E-REQ; workflow/integration | Production chain is the live `EscalateToCloud` probe → calibrated decision → role dispatch → SSB route event. | `H-7_calibrated_routing_architect_editor_verifier.md:99-100` |
| H006-H7-019 | m | E-REQ; surface | Add a route-decision drawer for the three existing `Kind` arms and pass `cross_lang_diff`. | `H-7_calibrated_routing_architect_editor_verifier.md:109-110` |
| H006-H7-020 | M | E-TEST; observability | A real no-override task must route a planning step to cloud architect and a checkable edit to local editor, run a separate-context verifier, emit journal-recomputable real-context route rows, and record over-escalation across at least three runs. | `H-7_calibrated_routing_architect_editor_verifier.md:111-119` |
| H006-H7-021 | M | E-TEST; acceptance/done-gate | Done requires run-trigger pass, live dispatch and SSB evidence, default-on behavior, drawer parity, measured over-escalation, reuse rather than rebuild, workspace/auditor green, and no new dependency. | `H-7_calibrated_routing_architect_editor_verifier.md:121-154` |
| H006-H7-022 | M | E-TEST; correctness | Contract cases must prove role separation, off-agent editor gating, effective provider change, SSB write rather than log-only behavior, live context values, and absence of parallel writers. | `H-7_calibrated_routing_architect_editor_verifier.md:156-161` |
| H006-H7-023 | m | E-REQ; integration/status | Planned write set spans task runner, cloud mode, escalation policy, cloud-route signal, dashboard, a contract test, and a decision doc. | `H-7_calibrated_routing_architect_editor_verifier.md:163-170` |
| H006-H7-024 | m | E-RISK; calibration | The central risk is double spend from an over-eager editor; calibration target and choice of `HybridEscalationGate` versus a new layer remain design decisions. | `H-7_calibrated_routing_architect_editor_verifier.md:181-187` |
| H006-H7-025 | M | E-REQ; failure/fallback/event | J-15 amendment: every attempted-and-failed backend hop in `cheap-tier -> cloud -> alternate cloud` must get its own journal row; recording only the final decision is insufficient. | `H-7_calibrated_routing_architect_editor_verifier.md:194-207` |
| H006-H7-026 | m | E-REQ; schema/event | Per-hop rows carry `(from_backend, to_backend, reason, timestamp)` so spend attribution, route-policy learning, and replay retain intermediate failures. | `H-7_calibrated_routing_architect_editor_verifier.md:199-207` |
| H006-H7-027 | M | E-REQ; schema/integration | Prefer a new `Kind::EscalationHopFailed` (final spelling chosen after live enum inspection) inside existing `RouteDecisionEvent`; add one kind only if no suitable kind exists, never a new action type. | `H-7_calibrated_routing_architect_editor_verifier.md:209-218` |
| H006-H7-028 | m | E-REQ; lifecycle/event | Failed-hop rows are additive; the final successful-route event still lands. | `H-7_calibrated_routing_architect_editor_verifier.md:214-215` |
| H006-H7-029 | m | E-REQ; dependency/status | H-7, not E-31, owns hop recording because it already owns the live fallback chain; H-7 lands first and is described by the amendment as `PARTIAL with live code`. | `H-7_calibrated_routing_architect_editor_verifier.md:220-227` |

### H-8 — persisted Oracle answer cache

| Atomic ID | Level | Explicit type/categories | Feature or behavior | Exact anchor |
|---|:---:|---|---|---|
| H006-H8-001 | M | E-STATE; dependency/status | Plan is `PLANNED`, Wave 1, in-process persistence BUILD and the smallest isolated session; it depends on shipped Oracle cache, REPL consumer, and PromptCache persistence. | `H-8_persist_oracle_answer_cache.md:3,12-15` |
| H006-H8-002 | M | E-STATE; data/lifecycle | `OracleAnswerCache` is blake3-keyed on `(query, project_root)`, has default capacity 512 and tier-derived TTL, but its `Mutex<CacheInner>` is lost at process exit. | `H-8_persist_oracle_answer_cache.md:5-10,25-26,28-32` |
| H006-H8-003 | M | E-STATE; integration/event | `/research` and `/ask` lazily initialize and use the cache; hits return early with `OracleCacheHit`, misses insert then emit `OracleCacheMiss`; CLI constructor `None` values are deliberate lazy-init, not wire-dark. | `H-8_persist_oracle_answer_cache.md:28-34` |
| H006-H8-004 | M | E-CORRECTION; data/lifecycle | `PromptCache` already persists `stats.json`, `session-state.json`, and `{hash}.json` beneath `$CLAUDE_CONFIG_HOME/cache/prompt-cache` or `$HOME/.elai/cache/prompt-cache`; the “ephemeral PromptCache” premise is wrong. | `H-8_persist_oracle_answer_cache.md:9-10,35-37` |
| H006-H8-005 | m | E-STATE; config/determinism | Existing Oracle TTLs are T1=30d, T2=7d, T3=24h, T4Rejected=never and use `cli.clock.now_secs`; PromptCache separately uses 30s completion and 5min prompt TTLs. | `H-8_persist_oracle_answer_cache.md:25-26,35-38` |
| H006-H8-006 | M | E-REQ; capability/data/lifecycle | On lazy init, load persisted Oracle entries from a deterministic Oracle-cache root; on insert and `evict_expired`, persist the entry set. | `H-8_persist_oracle_answer_cache.md:49-54` |
| H006-H8-007 | M | E-REQ; determinism | Preserve the blake3 key and tier TTLs and recompute expiry against `cli.clock.now_secs` during reload. | `H-8_persist_oracle_answer_cache.md:51-54` |
| H006-H8-008 | M | E-REQ; failure/recovery | Missing or corrupt cache files degrade to an empty cache and never panic; production IO may not use `.unwrap()`/`.expect()`. | `H-8_persist_oracle_answer_cache.md:54-55,75-80` |
| H006-H8-009 | M | E-REQ; security/correctness | Durable cache access is hard-confined to idempotent grounding (`/research`, `/ask`); code-generation requests must not reach durable `get` or `insert`. | `H-8_persist_oracle_answer_cache.md:57-59` |
| H006-H8-010 | m | E-REQ; evidence/status | Create `workflow/reasoning/H-8-decisions.md` documenting the process-only gap, PromptCache reuse, hard confinement, deterministic reload TTL, and chosen cache root. | `H-8_persist_oracle_answer_cache.md:61-63` |
| H006-H8-011 | M | E-REQ; integration/workflow | Production reachability is lazy init at `repl_command.rs:288`, hit at `:301`, and insert/write/miss at `:443`. | `H-8_persist_oracle_answer_cache.md:65-74` |
| H006-H8-012 | m | E-LIMIT; dependency/non-goal | Mirror the PromptCache persistence pattern but do not repurpose or rebuild PromptCache; add no new action type because `OracleCacheHit/Miss` already exist. | `H-8_persist_oracle_answer_cache.md:69-80,120-123` |
| H006-H8-013 | m | E-LIMIT; surface | No dashboard pair is required; a cache-stats line in `/research` is the human-visible signal. | `H-8_persist_oracle_answer_cache.md:81-83` |
| H006-H8-014 | M | E-TEST; observability/recovery | Two process runs of the same grounding question must turn process A’s miss/insert into process B’s `OracleCacheHit`, with no fresh grounding call and a cache file present. | `H-8_persist_oracle_answer_cache.md:84-90` |
| H006-H8-015 | M | E-TEST; security | The observable and contract gates must prove a code-generation query is never served by the durable cache. | `H-8_persist_oracle_answer_cache.md:89-90,102-108` |
| H006-H8-016 | M | E-TEST; acceptance/done-gate | Run-trigger, live REPL reachability, default-on/no override, mirrored persistence, confinement, deterministic TTL, corrupt-file recovery, build/auditor green, and no new dependency are all required. | `H-8_persist_oracle_answer_cache.md:92-123` |
| H006-H8-017 | M | E-TEST; determinism/idempotency | Contract tests cover cross-instance persistence, fixed-clock expiration (T3 at 25h), corrupt-file empty fallback, confinement, and stable blake3 `(query,project_root)` keys across restart. | `H-8_persist_oracle_answer_cache.md:125-130` |
| H006-H8-018 | m | E-REQ; integration/status | Planned write set is `oracle_cache.rs`, one new contract test, and one decision doc. | `H-8_persist_oracle_answer_cache.md:132-135` |
| H006-H8-019 | m | E-RISK; privacy/data | Cache layout must respect `$CLAUDE_CONFIG_HOME`/`$HOME` and prevent one project’s grounding data from leaking into another. | `H-8_persist_oracle_answer_cache.md:144-146` |
| H006-H8-020 | m | E-RISK; staleness/surface | Tier TTL mitigates stale grounding but does not eliminate it; surface cache age in `/research`. | `H-8_persist_oracle_answer_cache.md:147-148` |
| H006-H8-021 | m | E-REQ; lifecycle/concurrency | Reload must enforce the 512-entry cap by evicting oldest entries so an oversized disk cache cannot exhaust memory. | `H-8_persist_oracle_answer_cache.md:149` |
| H006-H8-022 | m | E-LIMIT; evidence | The cited 61.6–68.8% semantic-cache savings apply to factual QA, not code generation, and local prefix caching cannot reduce the cloud bill. | `H-8_persist_oracle_answer_cache.md:18-24,137-142` |
| H006-H8-023 | m | E-REQ; config/data | Candidate Oracle roots are `$CLAUDE_CONFIG_HOME/cache/oracle-cache` or `$HOME/.elai/cache/oracle-cache`, following PromptCache’s environment-root convention. | `H-8_persist_oracle_answer_cache.md:51-53,144-146` |

### H-17 — persisted untamperable trial ledger

| Atomic ID | Level | Explicit type/categories | Feature or behavior | Exact anchor |
|---|:---:|---|---|---|
| H006-H17-001 | M | E-STATE; dependency/status | Plan is `PLANNED`, Track-3/Wave-3 Tier-1 BUILD on the shipped WAL and a strict prerequisite for H-18/H-22/H-25/H-29. | `H-17_trial_ledger_keystone.md:3,15-21` |
| H006-H17-002 | M | E-REQ; security/correctness | Treat optimizer search as an unbounded multiple-testing source even when agents are honest; the keystone is cumulative trial count `N`, scoped per `(data_window, objective)`. | `H-17_trial_ledger_keystone.md:5-13,23-32` |
| H006-H17-003 | M | E-STATE; data/security | Shipped SSB is a single-writer append-only WAL with no exposed update/delete, debug `update_hook` panic, `ClockProvider` timestamps, and seed-deterministic dumps; no trial ledger exists. | `H-17_trial_ledger_keystone.md:34-38,45` |
| H006-H17-004 | m | E-STATE; permission/security | `gate_freeze_check.py`/`gate-manifest`, mutable=0 rule lifecycle, and `LockedKeysAuditor`/`LOCKED_KEYS_TOUCH` are shipped mechanisms to reuse. | `H-17_trial_ledger_keystone.md:39-44` |
| H006-H17-005 | M | E-REQ; schema/data | Add append-only `strategy_trials` with `trial_id`, `objective`, canonical `data_window`, `config_hash`, `seed`, `created_at_ms`, and `session_id`; `trial_id` and config hashes use blake3. | `H-17_trial_ledger_keystone.md:57-61` |
| H006-H17-006 | m | E-REQ; schema/performance | Extend the debug no-update invariant to the table and index `(objective,data_window)` for count reads; expose no update/delete method. | `H-17_trial_ledger_keystone.md:57-61` |
| H006-H17-007 | M | E-REQ; workflow/integration | Every backtest/strategy evaluation must call the single `register_trial()` choke point before reporting a metric. | `H-17_trial_ledger_keystone.md:63-64` |
| H006-H17-008 | M | E-REQ; determinism/idempotency | Re-registering the same `(config_hash,data_window,seed)` does not double-count, while a different config in the same search increments raw `N`. | `H-17_trial_ledger_keystone.md:64-68` |
| H006-H17-009 | M | E-REQ; data/lifecycle | Keep raw count monotone and derive `effective_trial_count()` by ONC-style clustering of near-duplicate/correlated configurations. | `H-17_trial_ledger_keystone.md:66-68,75-78` |
| H006-H17-010 | M | E-REQ; integration/event/schema | Emit `TrialRegistered` carrying `{objective,data_window,config_hash,seed,n_after}` so the ledger count is recomputable from SSB under `ELAI_SEED`. | `H-17_trial_ledger_keystone.md:70-73` |
| H006-H17-011 | m | E-REQ; schema/config | Authoring snapshot calls for baseline+1 from live 207/484 and correction of stale e45 count 482, but live schema/count must be reread at edit time. | `H-17_trial_ledger_keystone.md:70-72,154` |
| H006-H17-012 | M | E-REQ; capability/data | Expose `trial_count(objective,data_window)->u64` and `effective_trial_count(objective,data_window)->f64`; both read persisted rows and never accept an agent-supplied `N`. | `H-17_trial_ledger_keystone.md:75-78` |
| H006-H17-013 | M | E-REQ; failure/security | Zero matching rows yields the `trial_count_missing` sentinel and must fail closed at H-18; no agent delete/reset/update API may exist. | `H-17_trial_ledger_keystone.md:29-32,75-78` |
| H006-H17-014 | M | E-REQ; permission/security | Freeze the ledger schema and `register_trial` implementation in the gate manifest so a run editing them blocks. | `H-17_trial_ledger_keystone.md:80-82` |
| H006-H17-015 | M | E-REQ; flag/default | Ship the ledger default-on with no experimental flag; only a kill switch may disable it. | `H-17_trial_ledger_keystone.md:80-82,103-109` |
| H006-H17-016 | m | E-REQ; evidence/status | Create `H-17-decisions.md` explaining honest-search selection bias, raw versus effective N, single-write/no-reset design, and Tier-1 ledger versus Tier-2 backtest sidecar. | `H-17_trial_ledger_keystone.md:84-87` |
| H006-H17-017 | M | E-REQ; integration/workflow | Produced symbols are `strategy_trials`, `register_trial`, the two count APIs, and `SsbActionType::TrialRegistered`; H-18 consumes the count. | `H-17_trial_ledger_keystone.md:89-102` |
| H006-H17-018 | M | E-REQ; determinism/security | Use `ClockProvider`, not `SystemTime`, and blake3, not blake2/sha2; production may not use `.unwrap()`/`.expect()`. | `H-17_trial_ledger_keystone.md:95-109` |
| H006-H17-019 | m | E-LIMIT; surface | H-17 has no isolated frontend requirement; H-18/H-29 cockpit surfaces `N`, with an optional count strip. | `H-17_trial_ledger_keystone.md:110-112` |
| H006-H17-020 | M | E-TEST; observability/idempotency | A fixture registering at least three distinct configs must produce one row per distinct trial, a monotone journal-recomputable count, no duplicate increment, and a zero-count fail-closed downstream case. | `H-17_trial_ledger_keystone.md:113-120` |
| H006-H17-021 | M | E-TEST; acceptance/done-gate | Done requires real live-eval reachability, default-on behavior, frozen self-edit blocking, append-only/WAL reuse, byte-identical seeded rows/count, workspace/auditor green, and no new dependency. | `H-17_trial_ledger_keystone.md:122-156` |
| H006-H17-022 | M | E-TEST; determinism/security | Contract cases include append+event, duplicate idempotence, distinct-config increment, absence of mutation APIs, unseen-count zero, raw/effective correlated-count behavior, and two byte-identical `ELAI_SEED=42` runs. | `H-17_trial_ledger_keystone.md:158-164` |
| H006-H17-023 | m | E-REQ; integration/status | Planned files: new `trial_ledger.rs`; SSB/lock/count updates; live eval hook; frozen manifest; contract fixture/test; decision doc. | `H-17_trial_ledger_keystone.md:166-172` |
| H006-H17-024 | M | E-RISK; security/determinism | Canonicalize `(objective,data_window)` and freeze that canonicalizer, or an agent can fragment the scope key and effectively reset `N`. | `H-17_trial_ledger_keystone.md:182-185` |
| H006-H17-025 | m | E-RISK; config/fallback | ONC clustering threshold is locked and not agent-tunable; if clustering is uncertain, conservative raw `N` is the fail-safe. | `H-17_trial_ledger_keystone.md:186-187` |
| H006-H17-026 | M | E-REQ; lifecycle | Ledger lifetime is cross-session and per `(project,objective,data_window)`, not a per-session database. | `H-17_trial_ledger_keystone.md:188-189` |

### H-23 — same-seed rerun metric equality

| Atomic ID | Level | Explicit type/categories | Feature or behavior | Exact anchor |
|---|:---:|---|---|---|
| H006-H23-001 | M | E-STATE; dependency/status | Plan is `PLANNED`, Track-3/Wave-3 Cluster-D BUILD on shipped RRA/replay, `DriftComparator`, and rules predicates. | `H-23_same_seed_rerun_hash_equality.md:3,15-20` |
| H006-H23-002 | M | E-CORRECTION; state | RRA, `--emit-rra`, replay verification, replay resume, hardware/system pins, and first-divergence machinery are shipped; an older “RRA absent” statement is stale. | `H-23_same_seed_rerun_hash_equality.md:22-42` |
| H006-H23-003 | M | E-STATE; gap | `ELAI_SEED` records deterministic intent but no existing gate reruns the real experiment and compares declared metric outputs; that equality surface is absent. | `H-23_same_seed_rerun_hash_equality.md:5-13,42` |
| H006-H23-004 | M | E-REQ; schema/data | Extend `RunArtifact` evidence with a blake3 digest over experiment-declared metric artifacts, computed at run end; stdout is explicitly not the digest source. | `H-23_same_seed_rerun_hash_equality.md:55-57` |
| H006-H23-005 | M | E-REQ; workflow/determinism | Reinvoke the real entrypoint through replay under recorded environment plus `ELAI_SEED`, recompute the digest, and compare it with run 1. | `H-23_same_seed_rerun_hash_equality.md:59-62` |
| H006-H23-006 | M | E-REQ; failure/observability | Digest mismatch is FAIL and identifies the first diverging metric through `DriftComparator`; a missing rerun is also FAIL. | `H-23_same_seed_rerun_hash_equality.md:59-62` |
| H006-H23-007 | M | E-REQ; security/compliance | Add a Tier-1 `AST_PATTERN` that blocks unseeded `np.random`, PyTorch/random calls, `DataLoader(shuffle=True)` without generator+seed, and missing determinism environment variables at `TESTING_FIRST`. | `H-23_same_seed_rerun_hash_equality.md:64-66` |
| H006-H23-008 | m | E-REQ; config | Explicit determinism-variable examples are `PYTHONHASHSEED` and `CUBLAS_WORKSPACE_CONFIG`. | `H-23_same_seed_rerun_hash_equality.md:64-66` |
| H006-H23-009 | M | E-REQ; schema/security | `SchemaRequired` must force an environment-pin manifest containing lockfile hash, image `@sha256`, CUDA, driver, and Torch versions; missing manifest blocks. | `H-23_same_seed_rerun_hash_equality.md:68-70` |
| H006-H23-010 | M | E-REQ; integration/event | Emit Pattern-2 `RerunHashEquality` with `Equal`/`Divergent`, both digests, and first-divergence index. | `H-23_same_seed_rerun_hash_equality.md:72-74` |
| H006-H23-011 | m | E-REQ; schema/config | Authoring snapshot says baseline+1 from 207/484 with e45 count repair, but live counts must be reread at edit time. | `H-23_same_seed_rerun_hash_equality.md:72-73,129` |
| H006-H23-012 | m | E-REQ; evidence/status | Create `H-23-decisions.md` recording declared-output—not-stdout hashing, same-env scope, static early catch, and RRA/DriftComparator reuse. | `H-23_same_seed_rerun_hash_equality.md:76-78` |
| H006-H23-013 | M | E-REQ; integration/surface | Add `verify_rerun_digest`, seed/env rule files, the RunArtifact field, `elai replay --verify --metric-digest`, and an RRA-viewer digest/badge pair. | `H-23_same_seed_rerun_hash_equality.md:80-100` |
| H006-H23-014 | M | E-LIMIT; determinism | Scope is same-environment/same-seed equality only; cross-hardware GPU/floating-point deltas are recorded, never asserted bitwise. | `H-23_same_seed_rerun_hash_equality.md:12-13,30-32,94-98` |
| H006-H23-015 | M | E-REQ; flag/default | Gate is default-on with no override except a kill switch and runs from a live run-end/verify path. | `H-23_same_seed_rerun_hash_equality.md:115-130` |
| H006-H23-016 | M | E-TEST; observability | A hidden-unseeded-RNG fixture must emit `Divergent`, fail, and be caught statically; a clean seeded experiment hashes `Equal` twice. | `H-23_same_seed_rerun_hash_equality.md:101-104,115-121` |
| H006-H23-017 | M | E-TEST; failure/security | Contract cases include missing-rerun fail-closed, missing env manifest block, declared-output versus stdout proof, and same-env seeded equality. | `H-23_same_seed_rerun_hash_equality.md:132-137` |
| H006-H23-018 | m | E-REQ; integration/status/constraint | Planned write set includes rerun module, RRA extension, two immutable rule seeds, SSB/lock updates, CLI option, fixtures/test, and decision doc; no new dependency, RRA/DriftComparator reimplementation, non-blake3 digest, or production `.unwrap()`/`.expect()`. | `H-23_same_seed_rerun_hash_equality.md:94-98,121,139-146` |
| H006-H23-019 | m | E-RISK; workflow/cost | Full rerun is expensive and belongs pre-promotion/on-demand, while the seed-coverage AST gate is the per-edit early catch. | `H-23_same_seed_rerun_hash_equality.md:155-157` |
| H006-H23-020 | M | E-RISK; correctness/dependency | A sandbagging experiment can declare a trivial metric; H-25 must bind the reported number to provenance for the reproducibility claim to be complete. | `H-23_same_seed_rerun_hash_equality.md:158-160` |
| H006-H23-021 | m | E-RISK; security | A lying env manifest can defeat “same env”; its hash therefore belongs in the tamper-evident RRA bundle, while H-28 protects the harness. | `H-23_same_seed_rerun_hash_equality.md:161-162` |

### H-28 — reward-tampering gate and frozen harness

| Atomic ID | Level | Explicit type/categories | Feature or behavior | Exact anchor |
|---|:---:|---|---|---|
| H006-H28-001 | M | E-STATE; dependency/status | Plan is `PLANNED`, Track-3/Wave-3 Cluster-F capstone Tier-1 BUILD reusing the DGM promotion path, tripwire, WAL, rules, locked-key machinery; depends on H-17 and H-27. | `H-28_reward_tampering_harness_frozen.md:3,16-22` |
| H006-H28-002 | M | E-STATE; security/gap | Shipped `ArchiveVerifyResult` detects reward-function/eval-harness edits by blake3 mismatch, but not intact-hash manipulation of evaluator inputs/environment; the counter/input-tamper flag are absent. | `H-28_reward_tampering_harness_frozen.md:5-14,36-45` |
| H006-H28-003 | M | E-REQ; permission/security | Freeze evaluator, metric harness, H-18 DSR threshold, H-22 alpha, H-21 ULP bounds, and H-17 ledger in both gate manifest and locked-key surface; the self-mod agent may neither read nor mutate them. | `H-28_reward_tampering_harness_frozen.md:58-61` |
| H006-H28-004 | M | E-REQ; security/failure | Before self-mod admission, recompute harness blake3 and hard-block any mismatch by reusing `ArchiveVerifyResult` (deterministic leg 3a). | `H-28_reward_tampering_harness_frozen.md:63-65` |
| H006-H28-005 | M | E-REQ; security/failure | With hash intact, a post-self-mod score jump beyond a locked plausible band flags/blocks and routes to human Layer 3 (probabilistic RF-input leg 3b). | `H-28_reward_tampering_harness_frozen.md:67-70` |
| H006-H28-006 | M | E-REQ; workflow/invariant | A deterministic 3a hash-mismatch block can never be overridden by a 3b score-plausibility green; no mismatch path may promote. | `H-28_reward_tampering_harness_frozen.md:72-74` |
| H006-H28-007 | M | E-REQ; data/lifecycle/security | Persist an append-only, agent-unresettable `reward_tampering_counter`. | `H-28_reward_tampering_harness_frozen.md:76-80` |
| H006-H28-008 | M | E-REQ; integration/event/schema | Emit `RewardTampering` kinds `HarnessHashMismatch`, `ImplausibleScoreDelta`, and `StructuralSelfModRefused`, carrying harness hash and score-delta history. | `H-28_reward_tampering_harness_frozen.md:76-80` |
| H006-H28-009 | M | E-LIMIT; permission/non-goal | Structural changes to FSM, roles, or gate logic are never auto-approved; emit refusal and route to human Layer 3 because structural safety is undecidable. | `H-28_reward_tampering_harness_frozen.md:12-14,79-80` |
| H006-H28-010 | m | E-REQ; schema/config | Authoring snapshot says baseline+1 from live 207/484 and repair stale e45 count 482; live constants must be reread. | `H-28_reward_tampering_harness_frozen.md:76-78,139` |
| H006-H28-011 | m | E-REQ; evidence/status | Create `H-28-decisions.md` recording frozen-set scope, RF versus RF-input distinction, precedence, and the structural-human boundary. | `H-28_reward_tampering_harness_frozen.md:82-85` |
| H006-H28-012 | M | E-REQ; capability/integration | Add `check_reward_tampering`, counter, immutable `reward_tampering.toml`, and the event; call the check at `darwin_admit_loop -> two_gate_promote` before `insert_variant`. | `H-28_reward_tampering_harness_frozen.md:87-100` |
| H006-H28-013 | M | E-REQ; failure/default | Missing reward-tampering evidence blocks; the check runs on every self-mod admission default-on with only a kill switch. | `H-28_reward_tampering_harness_frozen.md:101-106,124-130` |
| H006-H28-014 | m | E-REQ; security/constraint | Reuse the tripwire, promotion, and locked-key machinery; use blake3, no production `.unwrap()`/`.expect()`, and no new dependency. | `H-28_reward_tampering_harness_frozen.md:93-106,130-131` |
| H006-H28-015 | m | E-REQ; surface/observability | Self-mod cockpit shows harness hash, score-delta history, and counter; blocked self-mod must be dashboard-visible and cross-language checks pass. | `H-28_reward_tampering_harness_frozen.md:107-108` |
| H006-H28-016 | M | E-TEST; observability/failure | Live fixture admissions must produce hard-block for harness edit, flag/block+Layer-3 for intact-hash score jump, promotion for clean config-only change, and refusal for structural change. | `H-28_reward_tampering_harness_frozen.md:109-112` |
| H006-H28-017 | M | E-TEST; acceptance/done-gate | Done requires run-trigger evidence, production-hook reachability, frozen manifest, precedence, human-only structural route, reuse, seeded recomputability, build/auditor green, and no dependency. | `H-28_reward_tampering_harness_frozen.md:114-140` |
| H006-H28-018 | M | E-TEST; security | Contract cases explicitly break both tamper arms, precedence, structural refusal, freeze protection, and counter reset resistance. | `H-28_reward_tampering_harness_frozen.md:142-148` |
| H006-H28-019 | m | E-REQ; integration/status | Planned files are reward-tampering module, immutable rule seed, SSB/lock updates, live admission hook, frozen manifest, fixtures/test, and decision doc. | `H-28_reward_tampering_harness_frozen.md:150-157` |
| H006-H28-020 | M | E-RISK; calibration/fallback | Plausible-delta band is locked, conservative, and operator-visible; an implausible gain should fail to human review rather than silently reject a legitimate gain. | `H-28_reward_tampering_harness_frozen.md:168-171` |
| H006-H28-021 | M | E-RISK; security/dependency | Freezing H-17 is essential because resetting `N` games H-18 at the source; H-27 plus a frozen H-28 validates and protects the gate itself. | `H-28_reward_tampering_harness_frozen.md:172-175` |

### H-29 — locked drift interlock

| Atomic ID | Level | Explicit type/categories | Feature or behavior | Exact anchor |
|---|:---:|---|---|---|
| H006-H29-001 | M | E-STATE; dependency/status | Plan is `PLANNED`, Track-3/Wave-3 Cluster-G Tier-1 BUILD depending on H-18 and composing shipped drift/canary/conformal/rules primitives. | `H-29_drift_state_machine.md:3,15-20` |
| H006-H29-002 | M | E-STATE; gap | G-10 supplies one model-drift canary; 90-day TTL, label-fed concept drift, and the locked resume interlock are absent. | `H-29_drift_state_machine.md:5-13,34-42` |
| H006-H29-003 | M | E-REQ; capability/data | Compute PSI on both inputs and outputs. | `H-29_drift_state_machine.md:53-59` |
| H006-H29-004 | M | E-REQ; schema/security | Compute a blake3 fingerprint of vendor-data schema; a fingerprint change is vendor drift. | `H-29_drift_state_machine.md:55-58` |
| H006-H29-005 | M | E-REQ; workflow/determinism | Compute rolling 14-day live-versus-backtest deviation over realized P&L using a `ClockProvider` window. | `H-29_drift_state_machine.md:55-58` |
| H006-H29-006 | M | E-REQ; lifecycle/dependency | Enforce a 90-day model/strategy deployment TTL that forces H-18 deflated-Sharpe re-evaluation at promotion. | `H-29_drift_state_machine.md:57-59` |
| H006-H29-007 | M | E-REQ; workflow/state | Build a locked mutable=0 `warning -> drift -> halt` composite: warning surfaces, drift escalates, halt stops trading and requires a full backtest rerun. | `H-29_drift_state_machine.md:61-63` |
| H006-H29-008 | M | E-REQ; failure/security | Missing evidence on any of the four axes means `HALT`, never `SKIP`. | `H-29_drift_state_machine.md:61-63` |
| H006-H29-009 | M | E-REQ; integration/workflow | Guard the actual FSM resume/keep-live transition; the interlock is not a passive monitor. | `H-29_drift_state_machine.md:65-67` |
| H006-H29-010 | M | E-REQ; failure/recovery | Vendor schema change halts until full backtest rerun; combined output-PSI plus 14-day-deviation breaches also halt. | `H-29_drift_state_machine.md:65-67` |
| H006-H29-011 | M | E-REQ; integration/event | Emit Pattern-2 `DriftInterlock` with `Warning`, `Drift`, or `Halt` and the per-axis signals/verdict; feature logs, realized P&L, and schema fingerprints are off-agent evidence. | `H-29_drift_state_machine.md:69-72` |
| H006-H29-012 | M | E-LIMIT; failure/fallback | Pre-label `P(Y|X)` concept drift is not a hard halt; route it to shipped conformal calibration as Tier-3 advisory. | `H-29_drift_state_machine.md:69-72` |
| H006-H29-013 | M | E-LIMIT; honesty | Guarantee only “halt within one realized-label cycle,” not “never trade a regime break.” | `H-29_drift_state_machine.md:10-13,77-80` |
| H006-H29-014 | M | E-REQ; permission/config | Freeze PSI band, 14-day deviation band, and 90-day TTL in mutable=0 rules and gate manifest. | `H-29_drift_state_machine.md:74-75` |
| H006-H29-015 | m | E-REQ; evidence/status | Create `H-29-decisions.md` covering axes/state machine, honest guarantee, conformal residual, primitive reuse, and missing-evidence halt. | `H-29_drift_state_machine.md:77-80` |
| H006-H29-016 | M | E-REQ; integration/dependency | Add `evaluate_drift_interlock` and `finance/drift_interlock.toml`; reuse `model_drift`, `CanaryFamily`, conformal `decide`, H-18 `evaluate_deflation`, compiled predicates, and the FSM edge. | `H-29_drift_state_machine.md:82-95` |
| H006-H29-017 | m | E-REQ; schema/config | Authoring snapshot says baseline+1 from 207/484 and repair e45 count 482; live schema/count must be reread. | `H-29_drift_state_machine.md:69-70,131` |
| H006-H29-018 | M | E-REQ; flag/default | Gate every resume default-on with no override beyond a kill switch. | `H-29_drift_state_machine.md:96-100,117-123` |
| H006-H29-019 | m | E-REQ; security/constraint | Reuse shipped drift/canary/conformal logic; use blake3 and no production `.unwrap()`/`.expect()`; PSI is closed-form and no new dependency is required. | `H-29_drift_state_machine.md:96-100,123` |
| H006-H29-020 | m | E-REQ; surface | Deployment cockpit renders per-axis warning/drift/halt and must visibly show vendor-schema halt; `cross_lang_diff` passes. | `H-29_drift_state_machine.md:101-102` |
| H006-H29-021 | M | E-TEST; observability/recovery | A real resume attempt with schema change, output-PSI breach, and 14-day breach must emit recomputable `Halt` rows and block until rerun; clean deployment resumes and TTL forces H-18. | `H-29_drift_state_machine.md:103-106` |
| H006-H29-022 | M | E-TEST; acceptance/done-gate | Done requires run-trigger evidence, default-on live edge, frozen bands/TTL, honest scope, primitive reuse, seeded recomputability, workspace/auditor green, and no dependency. | `H-29_drift_state_machine.md:108-132` |
| H006-H29-023 | M | E-TEST; failure/determinism | Contract cases cover each halt cause, clean resume, TTL recheck, missing feature-log halt, immutable bands, advisory residual, and identical seeded verdicts. | `H-29_drift_state_machine.md:134-139` |
| H006-H29-024 | m | E-REQ; integration/status | Planned files are drift module, immutable finance rule, SSB/lock updates, FSM edge, frozen manifest, fixtures/test, and decision doc. | `H-29_drift_state_machine.md:141-148` |
| H006-H29-025 | M | E-RISK; calibration/default | Bound false-halt rate on at least three real deployments before default-on; bias toward sensitivity because false halt is the safe failure. | `H-29_drift_state_machine.md:158-161` |
| H006-H29-026 | M | E-DEFER; deployment/status | Logic and fixture evidence are Tier-1, but actual live-trading resume is a separately gated Tier-2 deployment checkbox; no live resume ships before fixture proof. | `H-29_drift_state_machine.md:165-167` |

### H-30 — staleness and intent-drift instrument

| Atomic ID | Level | Explicit type/categories | Feature or behavior | Exact anchor |
|---|:---:|---|---|---|
| H006-H30-001 | M | E-STATE; dependency/status | Track-4/Wave-4 `BUILD`, strong/provable, blocks H-31 and H-32; no durable memory write/read is trusted until its instrument fires. | `H-30_staleness_intent_drift_metric.md:1-3` |
| H006-H30-002 | M | E-REQ; observability/correctness | Prove from the append-only journal that a superseded preference never reaches retrieval/prompt use and the latest belief is acted upon; cargo-green is insufficient. | `H-30_staleness_intent_drift_metric.md:2-3` |
| H006-H30-003 | M | E-REQ; capability/observability | Build runtime ratio `stale_fact_reached_prompt / total_retrievals` and a deterministic conflict-resolution harness with recognition-versus-application measurement. | `H-30_staleness_intent_drift_metric.md:5-9` |
| H006-H30-004 | m | E-REQ; determinism/performance | The steal/build section says to reimplement in Rust an order-independent blake3 fingerprint over sorted `(id,text,category)` and a low-signal read-time relevance semantic; never port the Python source. | `H-30_staleness_intent_drift_metric.md:9` |
| H006-H30-005 | m | E-LIMIT; dependency/non-goal | The >50%-deletion guard is a write-side H-31 responsibility; H-30 asserts metrics and does not delete. | `H-30_staleness_intent_drift_metric.md:9` |
| H006-H30-006 | M | E-STATE; schema/data | Existing `MemoryFact` has bi-temporal ingestion/validity and `superseded_by`; `FactStore` has atomic `supersede` and `soft_close`; `FactType` has eight variants and per-type thresholds. | `H-30_staleness_intent_drift_metric.md:11-14` |
| H006-H30-007 | M | E-STATE; workflow/event | `audit_conflicts` and `adjudicate_semantic` run on the live memory tick, transitively reach soft close, and reuse `memory_conflict_detected` with `conflict_kind="semantic_contradiction"`. | `H-30_staleness_intent_drift_metric.md:14` |
| H006-H30-008 | M | E-STATE; security/workflow | `get_relevant_facts` is test-only but already implements trust floor, nonempty-provenance filtering, safety-critical T2/T3 exclusion with `memory.safety_critical_blocked`, and read timestamp bump. | `H-30_staleness_intent_drift_metric.md:15` |
| H006-H30-009 | M | E-STATE; gap/integration | No prompt-assembly memory call exists, `ContextSlot` has no memory slot, and no fact-capture producer or staleness action type exists. | `H-30_staleness_intent_drift_metric.md:16` |
| H006-H30-010 | M | E-REQ; integration/forward-compatibility | Instrument the `get_relevant_facts` retrieval call itself now; H-32 later reuses the same counter around real prompt injection without metric rewrite. | `H-30_staleness_intent_drift_metric.md:16,18-25` |
| H006-H30-011 | M | E-REQ; schema/data | Each retrieval increments `total_retrievals`; each returned fact with `valid_to_ms.is_some()` or `superseded_by.is_some()` increments `stale_fact_reached_prompt`. | `H-30_staleness_intent_drift_metric.md:19-20` |
| H006-H30-012 | M | E-REQ; failure/observability | Clean retrieval should yield ratio zero by construction, while deliberately broken retrieval must make the metric nonzero. | `H-30_staleness_intent_drift_metric.md:20,23` |
| H006-H30-013 | M | E-REQ; capability/schema | Add `StalenessEvalRunner::run -> StalenessEvalReport` with computed accuracy, recognition, application, gap, stale/total counts, seed, and corpus blake3. | `H-30_staleness_intent_drift_metric.md:21` |
| H006-H30-014 | m | E-REQ; determinism/data | Embed fixtures via `include_str!`, pin corpus provenance, and derive `ELAI_SEED` through the shipped determinism module (default seed 0); numbers must be computed, never literals. | `H-30_staleness_intent_drift_metric.md:21` |
| H006-H30-015 | M | E-REQ; workflow/test | Each fixture stores preference X, contradicts it with Y through live adjudication, retrieves Y but never X, and checks that X links `superseded_by=Y`. | `H-30_staleness_intent_drift_metric.md:22` |
| H006-H30-016 | m | E-REQ; observability | Explicitly measure the store’s contradiction recognition separately from retrieval’s application and report their gap. | `H-30_staleness_intent_drift_metric.md:21-22` |
| H006-H30-017 | M | E-TEST; broken-arm | `--broken-retrieval` (or injected nonfiltering `RetrievalFilter`) must produce stale count >0 and lower accuracy; remaining at zero is itself failure. | `H-30_staleness_intent_drift_metric.md:23,64-67` |
| H006-H30-018 | M | E-REQ; surface/integration/event | Add `run_bench_staleness(json)` beside existing bench commands, write `benchmarks/staleness/<ts>.json`, and emit exactly one result beacon per bench run. | `H-30_staleness_intent_drift_metric.md:24` |
| H006-H30-019 | M | E-DEFER; workflow/status | Defer live memory `ContextSlot` and `conversation.rs` prompt wiring to H-32; H-30 is offline and uses `DeterministicNliJudge`, no 7B/cloud call. | `H-30_staleness_intent_drift_metric.md:25` |
| H006-H30-020 | M | E-REQ; lifecycle/constraint | Supersession remains bi-temporal newest-wins soft close; never delete, never narrate superseded facts as `[OUTDATED]`; returned stale rows are counted failures. | `H-30_staleness_intent_drift_metric.md:26,57-63` |
| H006-H30-021 | M | E-REQ; integration/schema | Add `staleness_runner.rs`, pure `retrieval_metrics.rs::RetrievalCounter/count_retrieval`, CLI dispatch, and one new SSB action type. | `H-30_staleness_intent_drift_metric.md:28-33` |
| H006-H30-022 | M | E-REQ; integration/event/schema | `StalenessEvalMeasured` payload contains accuracy, recognition, application, gap, stale/total counts, seed, corpus hash, and `broken_arm`; write through the single-writer MessagePack SSB path. | `H-30_staleness_intent_drift_metric.md:35` |
| H006-H30-023 | m | E-REQ; schema/config | Authoring snapshot is schema 220/variants 497 and proposes 221/498; stale 63/482 references are noted, and live constants must be confirmed cumulatively. | `H-30_staleness_intent_drift_metric.md:33-36` |
| H006-H30-024 | M | E-REQ; flag/default | Ship with no `ELAI_*_ENABLE` flag; CLI eval is default-available and counter is pure. If runtime emission must be gated, add experiment allowlist metadata and an ON-path smoke. | `H-30_staleness_intent_drift_metric.md:36` |
| H006-H30-025 | m | E-LIMIT; surface | No isolated frontend is required; H-9/H-33 may read the row later. | `H-30_staleness_intent_drift_metric.md:34` |
| H006-H30-026 | M | E-REQ; workflow/integration | Real reachability is `elai bench staleness --json -> run_bench_staleness`, then SSB write loop; test callers are not reachability witnesses. | `H-30_staleness_intent_drift_metric.md:38-56` |
| H006-H30-027 | M | E-LIMIT; security/privacy | No parallel store, delete/update, stale narration, non-blake3 hash, production unwrap/expect, SystemTime, persona fixture, or auditor/critic/adversary memory injection is permitted. | `H-30_staleness_intent_drift_metric.md:57-63` |
| H006-H30-028 | M | E-TEST; observability/determinism | Clean command emits one computed row with zero stale returns; a second seeded run must be byte-identically recomputable from the journal. | `H-30_staleness_intent_drift_metric.md:64-67` |
| H006-H30-029 | m | E-LIMIT; evidence | Literature rates motivate the threat but must never be emitted as ELAI measurements; ELAI’s rates are per-run computed. | `H-30_staleness_intent_drift_metric.md:69-73` |
| H006-H30-030 | M | E-REQ; dependency/done-gate | H-31 cannot ship until recognition and application pass; H-32 reuses the live ratio and must keep it at zero. | `H-30_staleness_intent_drift_metric.md:75-79` |

### H-36 — completion and needs-input notifications

| Atomic ID | Level | Explicit type/categories | Feature or behavior | Exact anchor |
|---|:---:|---|---|---|
| H006-H36-001 | M | E-STATE; dependency/status | Track-4/Wave-4 moderate-grounding BUILD, effort S, depends on H-35; producer-side notification dispatch and audit action are absent. | `H-36_completion_needs_input_notifications.md:1-7` |
| H006-H36-002 | M | E-STATE; workflow/event | Live completion trigger is `PhaseComplete{fsm_state:"complete"}` after fabrication-halt protection, with sibling web emitters; fabricated runs do not emit completion. | `H-36_completion_needs_input_notifications.md:13-16` |
| H006-H36-003 | M | E-CORRECTION; workflow/state | Needs-input is `AutonomyDecision::RequestHuman` persisted as `GateDecision`, not an `INTERVENTION_REQUIRED` FSM state; FSM has Idle/Executing/Complete and returns Complete→Idle. | `H-36_completion_needs_input_notifications.md:17-18` |
| H006-H36-004 | m | E-STATE; integration | SSB single-writer and `dispatch_stop` hook are live; hook-row write failures follow warn-on-failure behavior. | `H-36_completion_needs_input_notifications.md:19-20` |
| H006-H36-005 | M | E-STATE; surface | Dashboard completion/auditing toasts and webhook polling are live, but intervention handling at `NotificationManager.svelte:114-118` is an explicit no-op. | `H-36_completion_needs_input_notifications.md:21-23` |
| H006-H36-006 | M | E-REQ; capability/surface | `NotificationDispatcher` handles exactly two fixed event kinds: `Completion` and `NeedsInput`. | `H-36_completion_needs_input_notifications.md:25-30,39-43` |
| H006-H36-007 | M | E-REQ; surface/fallback | On terminal events, emit OSC 9 (`\x1b]9;<text>\x07`) when stdout is a TTY and always retain BEL (`\x07`) fallback. | `H-36_completion_needs_input_notifications.md:25-27` |
| H006-H36-008 | M | E-REQ; integration/fallback | Best-effort native dispatch uses Linux `notify-send`, macOS `osascript`, or Windows PowerShell via `std::process::Command`; missing binaries degrade to escape-only without panic. | `H-36_completion_needs_input_notifications.md:9,27,39-41` |
| H006-H36-009 | M | E-REQ; integration/event | Each dispatch writes `agent_notification_emitted` with event kind, attempted/succeeded surface bitsets, session id, and timestamp. | `H-36_completion_needs_input_notifications.md:27,46-48` |
| H006-H36-010 | M | E-REQ; workflow/integration | Completion calls notify from `InProcessHookDispatcher::dispatch_stop` only on genuine task completion, not every stop hook. | `H-36_completion_needs_input_notifications.md:28,42` |
| H006-H36-011 | M | E-REQ; workflow/integration | Needs-input calls notify immediately after live `emit_autonomous_decision` when the decision is `RequestHuman`. | `H-36_completion_needs_input_notifications.md:28,43` |
| H006-H36-012 | M | E-REQ; surface/integration | Replace dashboard intervention no-op by reading recent notification rows through a new Tauri command and firing a needs-input toast; retain existing completion toast. | `H-36_completion_needs_input_notifications.md:29,44` |
| H006-H36-013 | m | E-REQ; schema/config | Pattern-2 payload uses `AgentNotificationKind`, attempted bit0 terminal/bit1 OS, succeeded bitset, `session_id`, `ts_ms`; authoring proposes schema 220→221 and variants 497→498 in lockstep after live recheck. | `H-36_completion_needs_input_notifications.md:46-48` |
| H006-H36-014 | M | E-REQ; flag/default | `ELAI_NOTIFY_ENABLE` is default-off for OS process spawning and requires kill-switch allowlist metadata plus an ON-path mock-command smoke; terminal escape and SSB row remain unconditional. | `H-36_completion_needs_input_notifications.md:50` |
| H006-H36-015 | M | E-LIMIT; privacy/non-goal | Exactly two declarative dev events; no engagement framing, vacation mode, user-specific thresholds, quiet hours, frequency learning, schedules, or preference store. | `H-36_completion_needs_input_notifications.md:11,37` |
| H006-H36-016 | m | E-LIMIT; context/lifecycle | Notification text is fixed, never model-generated, adds zero inference tokens, and rows are append-only instantaneous audit events with no supersession/decay. | `H-36_completion_needs_input_notifications.md:31-35` |
| H006-H36-017 | M | E-DEFER; surface/status | Defer custom text, icons/deep links, sound customization, and rapid-event aggregation/coalescing. | `H-36_completion_needs_input_notifications.md:31` |
| H006-H36-018 | m | E-REQ; dependency/security | Use `std::io::IsTerminal` and `std::process::Command`; do not add `notify-rust` or another crate without explicit confirmation. | `H-36_completion_needs_input_notifications.md:39-41,65` |
| H006-H36-019 | M | E-REQ; failure/observability | No empty platform arm, discarded notify result, or silent SSB `.ok()`; OS failure is represented by a cleared success bit and SSB write failure warns. | `H-36_completion_needs_input_notifications.md:52-57` |
| H006-H36-020 | M | E-TEST; done-gate | Real `elai run` completion and separate fix-cap exhaustion must each produce a journal row from their production triggers; missing rows fail and cargo-green alone is insufficient. | `H-36_completion_needs_input_notifications.md:52-57` |
| H006-H36-021 | m | E-REQ; surface/security | Rust Tauri command and Svelte `invoke` must ship together under `cross_lang_diff`. | `H-36_completion_needs_input_notifications.md:44` |
| H006-H36-022 | m | E-REQ; dependency/workflow | H-35 must land first so a needs-input alert points to a real replay/rewind recovery surface. | `H-36_completion_needs_input_notifications.md:67-70` |

### H-51 — sleep-time grounded handoff and tiered resume

| Atomic ID | Level | Explicit type/categories | Feature or behavior | Exact anchor |
|---|:---:|---|---|---|
| H006-H51-001 | M | E-STATE; dependency/status | Track-9/Wave-9 strong-grounding BUILD-on-shipped, depends on H-50 and H-6, and composes H-35, SessionDag, and the live handoff rule. | `H-51_sleeptime_handoff_digest_autodetect.md:1-3` |
| H006-H51-002 | M | E-STATE; workflow/cost | Current `/continue` is manual and loads a trimmed transcript packed to roughly model window minus 5.1K, about 20–27K tokens on a 25–32K local model; fresh REPL is empty and auto-resume handles indexing only. | `H-51_sleeptime_handoff_digest_autodetect.md:3,9-10` |
| H006-H51-003 | M | E-STATE; integration | The handoff protocol already defines topic-keyed digest fields but is an unwired prose rule; journal, `SessionDag`, intervention/FileWrite evidence, and digest/lazy-loader primitives exist. | `H-51_sleeptime_handoff_digest_autodetect.md:3,7,11-13` |
| H006-H51-004 | M | E-REQ; workflow/capability | At session end/idle, a sleep-time pass authors or updates a stable problem-slug digest grounded in the SSB journal; the main LLM never hand-writes it. | `H-51_sleeptime_handoff_digest_autodetect.md:16-18` |
| H006-H51-005 | M | E-REQ; schema/data | Digest contains summary, decisions from intervention rows, files from `FileWrite`, FSM/plan cursor, and open follow-ups, targeting roughly 1–3K tokens. | `H-51_sleeptime_handoff_digest_autodetect.md:18` |
| H006-H51-006 | m | E-REQ; lifecycle | Append if topic digest exists, otherwise create; overwrite living `CURRENT STATUS` and `INDEX` fields in place. | `H-51_sleeptime_handoff_digest_autodetect.md:18` |
| H006-H51-007 | M | E-REQ; integration/event | Session-end authoring emits `HandoffLifecycleEvent::DigestWritten` as off-agent proof. | `H-51_sleeptime_handoff_digest_autodetect.md:18` |
| H006-H51-008 | M | E-REQ; workflow/state | Fresh startup detects `{open topic handoff + non-IDLE FSM + active DAG tip}` by current project/cwd; ambiguous matches list topics. | `H-51_sleeptime_handoff_digest_autodetect.md:19` |
| H006-H51-009 | M | E-REQ; surface/permission | Detection is automatic but continuation requires one-tap `[y/N]` confirmation; blind auto-continue is prohibited. | `H-51_sleeptime_handoff_digest_autodetect.md:19,28` |
| H006-H51-010 | M | E-REQ; integration/event | Detection and consent emit `ResumeOffered` then `ResumeConfirmed`. | `H-51_sleeptime_handoff_digest_autodetect.md:19` |
| H006-H51-011 | M | E-REQ; context/workflow | On confirmation, load only the 1–3K digest by default; keep transcript and tool-result stubs in SSB and selectively fetch details through H-50/H-6 `recover_stubs` only on request. | `H-51_sleeptime_handoff_digest_autodetect.md:20` |
| H006-H51-012 | M | E-REQ; integration/event/observability | Emit `TieredLoad` with measured loaded-token count, reducing default local resume from roughly 20–27K to 1–3K and making cost journal-recomputable. | `H-51_sleeptime_handoff_digest_autodetect.md:20,35` |
| H006-H51-013 | M | E-REQ; cache/determinism | Inject the digest after the cloud cache breakpoint or as stable-prefix material so it never busts the cached prefix. | `H-51_sleeptime_handoff_digest_autodetect.md:21` |
| H006-H51-014 | M | E-DEFER; status/surface | Defer multi-topic merge, digest-diff preview, 85%-budget auto-authoring, and cross-project suggestions. | `H-51_sleeptime_handoff_digest_autodetect.md:22` |
| H006-H51-015 | M | E-REQ; integration/schema | Add `handoff_digest.rs`, a topic index `{slug -> latest digest + DAG tip + FSM cursor}`, startup detection at `main.rs:712`, digest-first `/continue`, and raw lazy load. | `H-51_sleeptime_handoff_digest_autodetect.md:24-25` |
| H006-H51-016 | m | E-REQ; surface/integration | Add CommandWindow resume offer plus Tauri `resume_offer/confirm` in `commands_h51.rs`; cross-language diff must pass. | `H-51_sleeptime_handoff_digest_autodetect.md:26` |
| H006-H51-017 | M | E-REQ; integration/event/schema | Add one Pattern-2 `HandoffLifecycleEvent` with `DigestWritten`, `ResumeOffered`, `ResumeConfirmed`, and `TieredLoad`, in nine-site append-only lockstep. | `H-51_sleeptime_handoff_digest_autodetect.md:27` |
| H006-H51-018 | m | E-REQ; schema/config | Authoring snapshot is schema 224/variants 501; apply baseline+1 only after rereading the live cumulative values. | `H-51_sleeptime_handoff_digest_autodetect.md:14,27` |
| H006-H51-019 | M | E-REQ; flag/default | Detection is default-on, auto-continuation default-off, with `ELAI_HANDOFF_AUTORESUME_DISABLE` as the named kill switch. | `H-51_sleeptime_handoff_digest_autodetect.md:28` |
| H006-H51-020 | M | E-CORRECTION; integration | Wiring declaration corrects the earlier `compact_session` claim: shipped implementation does **not** call it and instead uses purpose-built `build_digest_body` to meet the 1–3K target. | `H-51_sleeptime_handoff_digest_autodetect.md:30-32` |
| H006-H51-021 | M | E-REQ; workflow/integration | Real entrypoints are session-end/idle, fresh REPL startup, and `/continue`/confirm dispatch; none may be a test shim. | `H-51_sleeptime_handoff_digest_autodetect.md:33` |
| H006-H51-022 | M | E-REQ; failure/security | Digest decisions/files must come from journal rows and fail closed if evidence is gone; LLM recollection is not acceptable grounding. | `H-51_sleeptime_handoff_digest_autodetect.md:34` |
| H006-H51-023 | M | E-REQ; context/failure | Default load above 5K is failure; selective recovery must not reinflate all stubs; dispatcher may not use `.unwrap()`. | `H-51_sleeptime_handoff_digest_autodetect.md:34` |
| H006-H51-024 | M | E-TEST; observability/done-gate | End a real seeded topic, verify row+digest against journal, start fresh and observe offer, confirm ≤~3K tiered load, and still recover an early detail. | `H-51_sleeptime_handoff_digest_autodetect.md:35` |
| H006-H51-025 | M | E-TEST; idempotency/no-op | Starting with no open session emits no offer and no lifecycle rows, distinguishing a correct no-op from a dead path. | `H-51_sleeptime_handoff_digest_autodetect.md:35` |
| H006-H51-026 | M | E-LIMIT; privacy/security | Digest stores work facts only, no persona/PII; auditor/adversary roles receive zero digest injection; lifecycle rows store slugs/hashes/cursors, never transcript bytes. | `H-51_sleeptime_handoff_digest_autodetect.md:40` |
| H006-H51-027 | M | E-REQ; staleness/recovery | If files drift after digest creation, flag and validate before use through H-30/H-31/H-32 rules. | `H-51_sleeptime_handoff_digest_autodetect.md:40,42-47` |
| H006-H51-028 | m | E-REQ; dependency/integration | Reuse H-50 selective recovery, H-6 read-back fix, H-35/SessionDag/RRA, and handoff protocol; implement shared recovery once. | `H-51_sleeptime_handoff_digest_autodetect.md:42-47` |
| H006-H51-029 | M | E-REQ; data/surface | Persist the topic digest at `workflow/handoffs/{slug}.md`. | `H-51_sleeptime_handoff_digest_autodetect.md:24-25,30-31` |

## Inferences and cross-source interactions

These are composition findings, not new feature requirements. Each is separated from the explicit ledgers.

| Atomic ID | Classification | Interaction/inference | Supporting source anchors |
|---|---|---|---|
| H006-I-001 | I-INTEGRATION | H-6’s generic `recover_stubs` activation is the shared primitive that H-51 later consumes through H-50’s selective wrapper; implementing separate recovery mechanisms would violate both plans’ reuse direction. | `H-6_observation_masking_memory_readback.md:72-77,182-185`; `H-51_sleeptime_handoff_digest_autodetect.md:11-12,42-46` |
| H006-I-002 | I-LIFECYCLE | H-6 addresses in-session tail masking and process read-back, while H-51 addresses cross-session digest-first loading; together they form lossless “small active context + journal detail” behavior at two lifecycle boundaries. | `H-6_observation_masking_memory_readback.md:61-77`; `H-51_sleeptime_handoff_digest_autodetect.md:16-21` |
| H006-I-003 | I-OBSERVABILITY | H-7 per-hop rows are necessary inputs to H-9-style cost attribution and faithful RRA replay, not merely richer route logs. | `H-7_calibrated_routing_architect_editor_verifier.md:194-218` |
| H006-I-004 | I-SECURITY | H-28’s frozen-set requirement is the enforcement layer that makes H-17’s “agent-unresettable” ledger credible against self-modification. | `H-17_trial_ledger_keystone.md:80-87,182-189`; `H-28_reward_tampering_harness_frozen.md:58-61,172-175` |
| H006-I-005 | I-SECURITY | H-23 protects experiment reproducibility inputs, while H-28 protects the evaluator/harness that interprets them; either alone leaves a tampering surface. | `H-23_same_seed_rerun_hash_equality.md:158-162`; `H-28_reward_tampering_harness_frozen.md:24-34` |
| H006-I-006 | I-DEPENDENCY | H-29’s 90-day H-18 recheck transitively depends on H-17’s persisted effective N, so live-resume trust inherits H-17 availability and freeze guarantees. | `H-17_trial_ledger_keystone.md:191-194`; `H-29_drift_state_machine.md:55-59,87-95` |
| H006-I-007 | I-FORWARD-COMPAT | H-30 deliberately measures the retrieval return set before prompt injection exists; H-51’s later digest validation should consume H-30/H-32 evidence without injecting digest content into auditor roles. | `H-30_staleness_intent_drift_metric.md:15-25,75-78`; `H-51_sleeptime_handoff_digest_autodetect.md:40,47` |
| H006-I-008 | I-RECOVERY | H-36’s needs-input alert is operationally useful only because H-35 supplies an inspect/rewind action; the dependency is a notification→human recovery flow. | `H-36_completion_needs_input_notifications.md:67-70` |
| H006-I-009 | I-SCHEMA | Schema snapshots 207/484, 220/497, and 224/501 are temporal baselines, not three compatible target totals; these ten plans require ship-order-cumulative lockstep updates. | `H-17_trial_ledger_keystone.md:70-73`; `H-30_staleness_intent_drift_metric.md:33-36`; `H-51_sleeptime_handoff_digest_autodetect.md:14,27` |
| H006-I-010 | I-HONESTY | Across H-6, H-23, H-29, and H-30, a recurring done-gate doctrine is to break the measured arm and require visible failure, rather than accept a healthy-looking metric. | `H-6_observation_masking_memory_readback.md:151-156`; `H-23_same_seed_rerun_hash_equality.md:132-137`; `H-29_drift_state_machine.md:134-139`; `H-30_staleness_intent_drift_metric.md:23,64-67` |
| H006-I-011 | I-PRIVACY | H-8’s project isolation, H-30’s work-fact-only fixtures, H-36’s no-preference rule, and H-51’s no-persona/no-transcript-row rule form a consistent boundary against cross-project or companion-style personalization leakage. | `H-8_persist_oracle_answer_cache.md:144-149`; `H-30_staleness_intent_drift_metric.md:63`; `H-36_completion_needs_input_notifications.md:11,37`; `H-51_sleeptime_handoff_digest_autodetect.md:40` |

## Ambiguities, internal conflicts, and corrections

| Atomic ID | Classification | Finding and required handling | Exact anchor |
|---|---|---|---|
| H006-C-001 | C-STATUS | H-7 header says `PLANNED`, but its later J-15 amendment says H-7 is `PARTIAL with live code`. Preserve both; current delivery state needs live-code audit. | `H-7_calibrated_routing_architect_editor_verifier.md:3`; `:220-227` |
| H006-A-002 | A-DESIGN | H-7 leaves `HybridEscalationGate` wire-versus-replace unresolved; no exact calibrated implementation is selected. | `H-7_calibrated_routing_architect_editor_verifier.md:69-72,181-187` |
| H006-A-003 | A-SCHEMA | `EscalationHopFailed` is illustrative; executor must inspect the live `RouteDecisionEvent` enum before choosing or adding the exact `Kind`. | `H-7_calibrated_routing_architect_editor_verifier.md:209-218` |
| H006-A-004 | A-CONFIG | H-8 gives example cache roots but leaves final per-project file layout open; the required invariant is no cross-project leakage. | `H-8_persist_oracle_answer_cache.md:51-53,144-146` |
| H006-C-005 | C-IDEMPOTENCY | H-17 scopes N per `(objective,data_window)` and describes `trial_id` as a hash of the row tuple, but states duplicate idempotency on `(config_hash,data_window,seed)` without `objective`; collision/domain behavior across objectives is unresolved. | `H-17_trial_ledger_keystone.md:5-10,57-68` |
| H006-A-006 | A-DATA | H-17 says the decision doc explains why raw and effective N are “both persisted,” while tasks define raw rows plus a computed `effective_trial_count()` read; whether an effective-N value is separately persisted is unclear. | `H-17_trial_ledger_keystone.md:66-68,75-87` |
| H006-A-007 | A-FAILURE | `trial_count_missing` is described for a “reported config,” but the declared count API accepts only `(objective,data_window)`; exact per-config missing-evidence check is underspecified. | `H-17_trial_ledger_keystone.md:75-78,113-120` |
| H006-A-008 | A-POLICY | H-28 alternates “FLAG/BLOCK” with “fail-to-human, not silent reject”; exact immediate machine verdict for intact-hash implausible delta must be made explicit, though human Layer-3 routing is consistent. | `H-28_reward_tampering_harness_frozen.md:67-70,109-112,168-171` |
| H006-C-009 | C-SUMMARY | H-29 detail requires output-PSI **plus** 14-day deviation to trip the stated combined halt, whereas orientation summary H29-014 says either breach independently blocks. The detail source does not establish both individual semantics. | `H-29_drift_state_machine.md:65-67,103-106`; orientation `08-H-SUMMARY_INDEX.discovery.md:606-625` |
| H006-C-010 | C-SCOPE | H-30’s opening says to reimplement fingerprint and relevance semantics, but revised integration produces neither; later COMPOSES-WITH assigns the fingerprint short circuit to H-31 and existing retrieval already has filtering. Treat these as referenced patterns unless executor resolves scope. | `H-30_staleness_intent_drift_metric.md:5-9,15,28-36,72,75-77` |
| H006-A-011 | A-METRIC | H-30 prose defines recognition/application gap narratively and once as “soft-closed cases minus cases retrieval served X,” which does not unambiguously specify rate normalization/sign. | `H-30_staleness_intent_drift_metric.md:21-22,35` |
| H006-C-012 | C-CONFIG | H-36 calls the only user control a binary “opt-out” gate, but integration names default-off `ELAI_NOTIFY_ENABLE`, which is an OS-notification opt-in. Unconditional terminal/event behavior is clear; user-control terminology is not. | `H-36_completion_needs_input_notifications.md:11`; `:50` |
| H006-C-013 | C-IMPLEMENTATION | H-51 initially says reuse `compact_session`; the wiring contract explicitly corrects that claim and says shipped `build_digest_body` is purpose-built. The correction supersedes the earlier integration claim. | `H-51_sleeptime_handoff_digest_autodetect.md:3,7,11,24-25`; `:30-32` |
| H006-C-014 | C-STATUS | H-51 is labeled BUILD-on-shipped with planned “SHIP NOW” work, but its correction speaks of a “shipped impl.” Current overall completion state cannot be derived from this file alone. | `H-51_sleeptime_handoff_digest_autodetect.md:2,16-18`; `:32` |
| H006-A-015 | A-CONFIG | `ELAI_HANDOFF_AUTORESUME_DISABLE` is named while detection is default-on and continuation default-off; whether the switch disables detection/offering or only an auto-resume behavior that is already off is not precisely stated. | `H-51_sleeptime_handoff_digest_autodetect.md:28` |
| H006-C-016 | C-TEMPORAL | H-6 plans to add a production `get_relevant_facts` caller, while the next-day H-30 code-read still finds zero production callers and assigns prompt wiring to H-32. This is consistent if H-6 remained unshipped, but the intended non-prompt H-6 caller versus H-32 caller boundary needs sequencing. | `H-6_observation_masking_memory_readback.md:76-77`; `H-30_staleness_intent_drift_metric.md:15-16,25` |
| H006-A-017 | A-SCOPE | H-29 motivates a label-fed concept-drift detector and promises a one-realized-label-cycle bound, but its build tasks specify four nonconditional axes plus a pre-label conformal advisory; ADWIN/DDM are described as scoped rather than an explicit produced symbol. Exact label-fed implementation is unresolved. | `H-29_drift_state_machine.md:5-13,53-80,150-166` |

## Summary additions and corrections

The orientation summary already captures the headline requirements. The following are every material detail-source addition or correction found in this chunk; references point back to the single canonical atom rather than duplicate it.

| Delta ID | Plan | Disposition versus `08-H-SUMMARY_INDEX.discovery.md` | Canonical atom(s) |
|---|---|---|---|
| H006-SD-001 | H-6 | ADD: ladder tiers are telemetry estimators until H-6 makes them mutate or labels them honestly. | H006-H6-004, H006-H6-005, H006-H6-014 |
| H006-SD-002 | H-6 | ADD/CORRECT: `recover_stubs` itself fails closed; the placeholder comes from the caller bypassing it, and missing/corrupt recovery must be visible. | H006-H6-007, H006-H6-015, H006-H6-016 |
| H006-SD-003 | H-6 | ADD: memory read-back mutates `last_observed_at_ms`; acceptance explicitly checks the bump and SSB re-fetchability. | H006-H6-006, H006-H6-021, H006-H6-025 |
| H006-SD-004 | H-7 | ADD: J-15 requires a row for every failed fallback hop and specifies fields plus additive final-success behavior. | H006-H7-025, H006-H7-026, H006-H7-028 |
| H006-SD-005 | H-7 | ADD: reuse/add a `RouteDecisionEvent` kind rather than an action type; exact kind remains live-enum dependent. | H006-H7-027, H006-A-003 |
| H006-SD-006 | H-7 | ADD/STATUS: H-7 owns hop journaling over E-31 and is later described as partial, conflicting with its planned header. | H006-H7-029, H006-C-001 |
| H006-SD-007 | H-7 | ADD: exact hardcoded context values and the unresolved `HybridEscalationGate` wire/replace fork. | H006-H7-003, H006-H7-009, H006-A-002 |
| H006-SD-008 | H-8 | ADD: exact tier TTLs, capacity 512, environment-aware cache-root candidates, cache-age surface, and oldest-on-reload eviction. | H006-H8-005, H006-H8-020, H006-H8-021, H006-H8-023 |
| H006-SD-009 | H-8 | ADD: corrupt/missing persistence degrades to empty, and disk layout must prevent cross-project leakage. | H006-H8-008, H006-H8-019 |
| H006-SD-010 | H-17 | ADD: exact table columns, hash/time rules, index, distinct-trial idempotence, and cross-session lifetime. | H006-H17-005 through H006-H17-009, H006-H17-018, H006-H17-026 |
| H006-SD-011 | H-17 | ADD: freeze canonicalization of the scope key; raw N is conservative fallback when ONC clustering is uncertain. | H006-H17-024, H006-H17-025 |
| H006-SD-012 | H-17 | AMBIGUITY: objective omission from duplicate key, effective-N persistence wording, and config-specific missing sentinel are unresolved. | H006-C-005, H006-A-006, H006-A-007 |
| H006-SD-013 | H-23 | ADD: missing rerun fails, exact env-pin fields and determinism variables are named, and rerun belongs late/on-demand. | H006-H23-006, H006-H23-008, H006-H23-009, H006-H23-019 |
| H006-SD-014 | H-23 | ADD: declared-output sandbagging is closed only with H-25, and manifest dishonesty is mitigated by RRA hash plus H-28. | H006-H23-020, H006-H23-021 |
| H006-SD-015 | H-28 | ADD: append-only unresettable counter and exact third kind `StructuralSelfModRefused`; the orientation inventory’s “clean” shorthand is not the detail enum. | H006-H28-007 through H006-H28-009 |
| H006-SD-016 | H-28 | ADD/AMBIGUITY: plausible-band handling is fail-to-human and conservative, but immediate flag-versus-block semantics need a decision. | H006-H28-020, H006-A-008 |
| H006-SD-017 | H-29 | CORRECT: detail demonstrates schema halt and a combined output-PSI+14d halt, not the orientation’s unqualified independent-OR wording. | H006-H29-010, H006-C-009 |
| H006-SD-018 | H-29 | ADD: false-halt rate requires three real deployments, sensitivity is preferred, and live resume remains a Tier-2 checkbox. | H006-H29-025, H006-H29-026 |
| H006-SD-019 | H-30 | CORRECT: `stale_fact_reached_prompt` initially counts stale rows returned by the retrieval call because no prompt site exists. | H006-H30-009 through H006-H30-012 |
| H006-SD-020 | H-30 | ADD: exact stale predicate, one timestamped report/beacon, no-persona constraint, and a broken arm that remaining-zero makes fail. | H006-H30-011, H006-H30-017, H006-H30-018, H006-H30-027 |
| H006-SD-021 | H-30 | CORRECT/CONFLICT: summary treats fingerprint and relevance reimplementation as H-30 requirements, but revised detail scope omits them and assigns fingerprint work to H-31. | H006-H30-004, H006-H30-005, H006-C-010 |
| H006-SD-022 | H-30 | AMBIGUITY: recognition/application gap normalization and sign need formal definition. | H006-A-011 |
| H006-SD-023 | H-36 | CORRECT: needs-input is a `RequestHuman` GateDecision, not an intervention FSM state. | H006-H36-003 |
| H006-SD-024 | H-36 | ADD: exact bitset schema, TTY/OS fallback behavior, OS-failure evidence, and platform command matrix. | H006-H36-007 through H006-H36-009, H006-H36-013, H006-H36-019 |
| H006-SD-025 | H-36 | CONFLICT: “binary opt-out” prose conflicts with default-off OS `ELAI_NOTIFY_ENABLE`; unconditional terminal/event legs remain clear. | H006-H36-014, H006-C-012 |
| H006-SD-026 | H-36 | ADD: finite exactly-two-event privacy boundary, deferred polish list, and H-35 recovery sequencing. | H006-H36-015 through H006-H36-018, H006-H36-022 |
| H006-SD-027 | H-51 | CORRECT: purpose-built shipped `build_digest_body`, not `compact_session`, is the declared digest implementation. | H006-H51-020, H006-C-013 |
| H006-SD-028 | H-51 | ADD: exact startup predicate, ambiguous-topic list, living-field overwrite semantics, and >5K failure threshold. | H006-H51-006, H006-H51-008, H006-H51-023 |
| H006-SD-029 | H-51 | ADD: no-open-session is a zero-row no-op; lifecycle rows exclude transcript bytes and auditor roles get no digest. | H006-H51-025 through H006-H51-027 |
| H006-SD-030 | H-51 | STATUS/CONFIG ambiguity: file says both BUILD and shipped implementation; autoresume-disable switch scope is unclear. | H006-C-014, H006-A-015 |
| H006-SD-031 | Cross-plan | ADD: H-6 versus H-30/H-32 production-memory-caller sequencing is temporally unresolved. | H006-C-016 |
| H006-SD-032 | H-29 | AMBIGUITY: the promised label-cycle concept-drift bound lacks an explicit label-fed produced symbol; ADWIN/DDM remain scoped in risk text. | H006-A-017 |

## Zero-omission and zero-duplicate audit

### Coverage matrix

Every requested extraction dimension is represented by at least one canonical atom for every file where the source makes an explicit statement. A dash means the source explicitly has no such surface or treats it as N/A, and that N/A is itself captured.

| Source | Capabilities/actions/surfaces | Workflows/states | Schema/data/lifecycle | Flags/defaults/config | Integrations/events | Security/privacy/compliance | Failure/recovery | Determinism/idempotency/concurrency | Observability/evidence | Tests/done gates | Constraints/non-goals | Dependencies/status/deferred |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| H-6 | 010,015,017 | 003-005,019 | 011-012 | 013 | 012,019-020 | 006-007,023 | 016,027 | 010-011,022-023 | 021 | 024-025 | 002,014 | 001,022,026,028 |
| H-7 | 007-010 | 002-005,018 | 010,026-028 | 003,012,014 | 011,025-028 | 008,015 | 005,024-028 | 009-010 | 014,020 | 021-022 | 015-016 | 001,006,017,023,029 |
| H-8 | 006,009 | 003,011 | 002,005-007,021 | 005,023 | 003,011-012 | 009,019 | 008,014 | 007,017 | 013-014,020 | 014-017 | 004,012,022 | 001,018 |
| H-17 | 005,007,012 | 007,017 | 005-013,026 | 015,025 | 010-011,017 | 002-004,013-015,018 | 013,025 | 008-009,018,020,022 | 010,019-020 | 020-022 | 019,025 | 001,016,023,026 |
| H-23 | 004-009 | 005,013 | 004,009-011 | 008-009,015 | 010,013 | 007,009,021 | 006,017 | 004-006,014,16-17 | 006,010,016 | 016-17 | 014,019-021 | 001-003,012,018 |
| H-28 | 003-008 | 004-006,012 | 007-010 | 010,013 | 008,012 | 002-006,009,013-014 | 004-006,013,016 | 004,017 | 008,015-016 | 016-018 | 009,014 | 001,011,019-021 |
| H-29 | 003-009 | 007-010 | 003-007,011,014 | 014,017-018 | 011,016 | 004,008,012-014 | 008-010,012-013 | 005,019,023 | 011,020-021 | 021-023 | 012-013,019 | 001-002,015,024-026 |
| H-30 | 003,010,013 | 006-010,015 | 006-08,011-16,022 | 023-024 | 007,018,021-022,026 | 008,020,027 | 012,017,020 | 004,014,028 | 002-003,012-018,028-029 | 017,028,030 | 005,019-020,025,027,029 | 001,005,019,030 |
| H-36 | 006-012 | 002-005,010-011 | 009,013,016 | 014 | 002-005,009-012 | 015-018 | 007-008,019 | — (instantaneous audit lifecycle in 016) | 009,019-020 | 020-021 | 015-018 | 001,017-018,022 |
| H-51 | 004-012,015-016 | 002-004,006,008-012,021 | 005-007,015,017-018,029 | 018-019 | 007,010,012,015-017 | 022,026-027 | 022-025,027 | 013,024-025 | 007,010,012,024-025 | 024-025 | 009,014,023,026 | 001-003,014,020,028 |

### Audit result

- Full-read coverage: `10/10` files, `1495/1495` lines, `159276/159276` bytes.
- Explicit ledger atoms: **255** (`H-6` 28; `H-7` 29; `H-8` 23; `H-17` 26; `H-23` 21; `H-28` 21; `H-29` 26; `H-30` 30; `H-36` 22; `H-51` 29).
- Inferred interaction atoms: **11**. Ambiguity/conflict atoms: **17**. Total atomic discovery IDs: **283**.
- Summary-delta audit records: **32**, all reference canonical atoms and deliberately do not restate them as additional feature atoms.
- Duplicate audit: every atomic discovery ID occurs exactly once as a defining row; cross-sections reference IDs rather than mint duplicate claims. Mechanical ID audit reports zero duplicate definitions and no numbering gaps within any per-file ledger.
- Omission audit: source line spans were checked end-to-end; headings, status/objective, existing state, tasks, wiring contract, observable effects, checks/acceptance, tests, files, grounding limits, risks, next/dependencies, amendments, revised scopes, integration, and composes-with sections are all represented or explicitly classified as evidence-only/non-feature material.
- No inferred requirement was promoted to explicit. No ambiguity was silently resolved. No summary-only claim was imported as a detail-source feature.

## Output integrity

- Canonical self-hash rule: compute SHA-256 over this UTF-8 file after replacing the hexadecimal value on the next line with the literal `<SELF>`. This avoids an impossible recursive file-hash fixed point while keeping the embedded digest reproducible.
- Canonical output SHA-256: `2c8b3ad15c4b36a3c94a2a275754539aad1ee6325cd095ede7b8576e2d72a207`
