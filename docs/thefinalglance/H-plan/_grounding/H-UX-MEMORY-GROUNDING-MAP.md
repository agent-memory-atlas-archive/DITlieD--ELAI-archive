# H-UX-MEMORY-GROUNDING-MAP (Track 4, H-30..H-38)
Generated 2026-06-04. W-7 grounding source for Track-4 authoring. Anchors grep-confirmed against live source
this session unless marked CONFIRM-AT-AUTHORING. Research provenance: run wf_c8950d72-45e (15-lens internet
sweep, 13/15 structured) + author run wf_02516dd9-e82; odysseus reference map docs/.source/odysseus-INDEX.md
+ docs/.source/odysseus_py.map (4,861 fns, MIT prior-art).

## CANONICAL CONSTANTS (LIVE 2026-06-04 — supersedes the 207/484 in H-0_INDEX)
- SCHEMA_VERSION = 220  (crates/orchestrator/src/ssb.rs:2990)
- TOTAL_VARIANTS = 497  (crates/orchestrator/tests/action_type_lock.rs:624)
- STALE pins (do NOT use as baseline): CLAUDE.md 191/464; H-0_INDEX 207/484; F-plan 205/482.
- LATENT DRIFT to fix on first Track-4 bump: e45_cross_language_drift_contract.rs (~:294, ~482 variant pin) will FAIL on re-run.

## MEMORY SUBSTRATE (already shipped; Track-4 ACTIVATES it — do NOT build a parallel store, GD-1)
- crates/memory/src/fact.rs:18 FactType (8 variants; Preference :35; per-type thresholds :81); :101 Scope (Project/Global, promotion deferred :95-98); MemoryFact :138 (ingested_at_ms:162 / valid_from_ms:164 / valid_to_ms:167 / superseded_by:179 / evidence_pointers:177)
- crates/memory/src/store.rs:208 supersede (atomic soft-close-old + insert-new, newest-wins, NO delete); :303 soft_close; :147 list_current (valid_to_ms IS NULL filter); :270 update_trust; :128 get; :84 insert; :319 iter_current
- crates/memory/src/mem0_dispatcher.rs:144 dispatch_one / :218 dispatch_batch (ADD / UPDATE-via-supersede / DELETE-via-soft_close / NOOP) — WIRE-DARK (only #[cfg(test)] callers :341-373)
- crates/memory/src/contradiction.rs:64 audit_conflicts — PROD-WIRED via live_cli.rs:2034 (its first production caller)
- crates/memory/src/semantic_contradiction.rs:392 adjudicate_semantic / :312 collect_cross_key_candidates / :115 DeterministicNliJudge (offline lexical, default-ON) / :98 NliJudge trait (Tier-2 ONNX behind feature nli-onnx, model NOT on disk) — PROD-WIRED via live_cli.rs:2076; emits MemoryConflictDetected conflict_kind="semantic_contradiction" (REUSE, no new variant)
- crates/memory/src/decay.rs:29 default_decay_rate (per-FactType half-lives) / :45 decayed_trust / :88 decay_tick; scheduler spawn_decay_scheduler (lib.rs:68) — the HALO backstop, already present
- crates/memory/src/retrieval.rs:46 get_relevant_facts + :17 RetrievalFilter (trust-floor + empty-evidence_pointers exclusion :97 + C-48 safety-critical T2/T3 exclusion :62-78 + last_observed bump :109-116) — WIRE-DARK (every caller a test; elai.map CALL_RESOLVED 281701-281705 all land in tests/)
- INJECTION SITE ABSENT (the central gap): context_budget.rs:1066 ContextSlot (8 slots: SystemPrompt/Rules/PhaseSpec/RepoMap/CodeContext/SsbEntries/GrcHistory/OutputReserve, SLOT_COUNT=8 at :10) has NO memory slot; conversation.rs has ZERO memory refs. H-31 adds ContextSlot::Memory + the conversation.rs get_relevant_facts call.
- READ-only UI today: commands_e5.rs:136 get_memory_facts -> store.list_current :150; MemoryFactsDrawer.svelte:83; semantic-contradiction viewer commands_e5.rs:194-195
- CLI verbs PARSED but UNROUTED: args_memory.rs:10 / enum args.rs:1310 (List/Inspect/Insert/Delete); main.rs:1019-1024 dispatch = EXPLICIT STUB ("handler being wired in s-31708; not yet routed") — H-33 de-stubs

## QOL SUBSTRATE
- H-34 MCP: live lane = STDIO McpServerManager mcp_stdio.rs:484/493/498, discover_tools_best_effort :559 wired at mcp_glue.rs:62/70, each McpTool carries input_schema verbatim :124, insert loop has NO defer gate :545-552; C-35 LazySchemaCache crates/plugins/src/mcp/lazy.rs:30 (resolve :50, hit/miss :96-98) + resolve_schema_on_demand mcp_bridge.rs:959 EXIST but hang off the WIRE-DARK HTTP bridge mcp_client.rs:26-38 (zero non-test callers). DISTINCT from H-4 (built-in catalog production_agent_rpc.rs:652 build_tool_catalog / :1221 injection).
- H-35 rewind: ssb.rs:15353 query_session_stream_asc (ASC replay); observability/replay.rs:84 ReplaySessionId::derive + ssb_resume_from (fail-closed LockedKeysAuditor) + :99 ReplayOutcome; ReplayResume ssb.rs:23345/23562 (NOT args.rs — brief corrected); Tauri commands_e29.rs:221 ssb_resume_from; session_dag.rs:1507 recover_stubs (blake3 verify :1543); rra.rs:384 build_rra / :474 verify_rra, seal run_subcommand.rs:449-459; compact.rs:96 compact_session
- H-36 notify: completion task_runner.rs:9610 PhaseComplete (siblings :2478/:4759); needs-input :1157 AutonomyDecision::RequestHuman -> emit_autonomous_decision :1162; SSB write_loop ssb.rs:15146; hook seam hook_dispatcher.rs:1146 dispatch_stop; dashboard NotificationManager.svelte:102-110 (completion toast) / :114-118 (intervention = explicit no-op, the B-36 gap)
- H-37 plan-gate: gate.rs:230 run_interactive_gate (APPROVE/REJECT/MODIFY/REDIRECT, WIRE-DARK — callers only #[cfg(test)] :733-956); CliInteractiveGate runtime_builder.rs:429 installed run_subcommand.rs:118-126 when gate_mode==Interactive; compliance pre-gate fsm.rs:925-928 but compliance_provider defaults None :570 (runtime-dark); AutonomousGate autonomous_gate.rs:88; emit_autonomous_decision task_runner.rs:14164 -> SsbPayload::GateDecision ssb.rs:8809; task_runner holds the gate as presence-only (`let _ = autonomous_gate.as_ref();` :1087)
- H-38 cost: cloud_mode.rs:646 SpendAccumulator / :521 CloudSpendSnapshot / :357 daily-cap; ssb_viewer.rs:5062 cloud_spend_summary day_usd; grounding_client.rs:848-905 post-hoc cap precedent; stop boundary hook_dispatcher.rs:1146 dispatch_stop (zero cost awareness). CORRECTION: pod_stop / pod_cost_status are HARNESS MCP tools, NOT ELAI Rust (corpus literal only corpusgen.rs:71) — the pod-cost half is a skill-doc concern.
- Eval-runner pattern (H-30 + H-34 mirror, GD-1): crates/orchestrator/src/benchmarks/seccodeplt_runner.rs (run:312, SecCodePltReport:150, include_str! corpus:65, seed:164, corpus_blake3:166, InsecureRateMeasured beacon:44, no-literals honesty :23); ELAI_SEED_ENV determinism.rs:48; single-writer write_loop ssb.rs:15146

## ODYSSEUS PRIOR-ART (MIT; REIMPLEMENT in Rust, never port; docs/.source/odysseus-main/ + odysseus-INDEX.md)
- src/memory.py:255 categorize_memory_by_relevance / :291 get_relevant_memories (read-time relevance gate, 0.9 identity floor, top-K cap) -> H-31 on-demand ranking
- src/memory_vector.py:116 find_similar(threshold=0.92) -> H-32 high-threshold dedup
- services/memory/memory_extractor.py:32-35 _fingerprint_entries (order-independent; reimpl blake3 over sorted (id,text,category)) / :441-443 short-circuit / :535-540 >50%-deletion safety guard (H-31 owns; armed when the H-32 tidy pass lands)
- src/teacher_escalation.py:133 _UNTRUSTED_TRACE_GUARD / :388 / :617 / :621-626 NO_SKILL distill gate (validate-before-persist -> H-32 inverts to validate-before-USE)
- src/tool_index.py:431 get_tools_for_query (always-floor :24/:435 UNION vector top-K :436-437 UNION keyword force-include :444-446) -> H-34 union assembler on ELAI's OWN ONNX-vector stack
- src/context_compactor.py:174 trim_for_context / :263 maybe_compact + src/context_budget.py:21 headroom-clamp -> H-35/H-33 (drop the 200K cloud constant for the 25-32K local ceiling)
- src/skill_extractor.py:66/:57 (write-path classifier + dedup) -> H-32 tier classifier
- src/task_scheduler.py, src/event_bus.py -> async/notification prior art (H-36; event-bus also the Track-5 SSB-event-trigger steal)

## KNOWN UNCONFIRMED / CONFIRM-AT-AUTHORING (per-author flags; the workflow result has the full per-file lists)
- arXiv future-dated, NOT web-verified this session (anti-slopsquat — WebSearch BEFORE any headline number; ELAI bad-cite history B-40/B-42/E-40): 2605.06527 (STALE), 2603.23848 (BeliefShift), 2512.16962 (MemoryGraft, H-32). The grounded-from-the-sweep set (re-verify IDs at execution): 2501.13956 (Zep), 2504.19413 (Mem0), 2505.07509 (HALO), 2510.09905 (Personalization-Trap).
- e45_cross_language_drift_contract.rs:294 exact variant-count line NOT grep-confirmed (stale ~482); reconcile at the first Track-4 SCHEMA bump.
- H-31 detail-file COMPOSES-WITH mischaracterises H-30 as a "capture/intake substrate" — H-30 is the staleness METRIC; the live FactStore handle comes from the live_cli.rs:2034 contradiction tick, NOT from H-30. Fix at execution.
- MemoryFact struct line: fact.rs:138 (the `pub struct` line; the ground bundle's :137 is the doc-comment above it).
- Several odysseus + a few ELAI line refs were taken from odysseus-INDEX.md / the ground bundle (prior-agent verified) and not re-opened by each author; the per-file unconfirmed_anchors lists in author run wf_02516dd9-e82 are the authoritative record the executor re-greps (W-7).
