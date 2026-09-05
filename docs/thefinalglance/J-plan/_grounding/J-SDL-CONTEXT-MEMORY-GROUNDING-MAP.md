J-SDL-CONTEXT-MEMORY GROUNDING MAP (J-103..J-106)
Authored 2026-06-26 (main-LLM in-band, paper-to-plan PHASE-5 over the reasoning file
  workflow/reasoning/jplan-sdl-context-memory-readaccess-2026-06-26.md, which IS the PHASE-4 steal/gap analysis).
Lens: spec-eval AS-SPECIFIED. Scope: ELAI-the-product F:\.ELAI_workflow\.elai_cc (technical->J split; the
  K-native PROVE+PACKAGE+DISCIPLINE halves are out of scope here).
W-7 norm: every integration anchor grep-confirmed at authoring. Line numbers are 2026-06-26; re-grep at edit (W-9 for SSB).

LIVE SSB BASELINE (this session)
- SCHEMA_VERSION = SCHEMA_VERSION_DDL_EPOCH(-309) + SSB_ACTION_TYPES.len()  [crates/orchestrator/src/ssb.rs:1503/1508; SCHEMA_VERSION_PUBLIC:1513]
- TOTAL_VARIANTS = ALL_VARIANTS.len()  [crates/orchestrator/tests/action_type_lock.rs:775]
- map header carries ~268/577 (DERIVED, drifts); ALL four files PREFER-REUSE (net 0..+1), so the exact integer
  only matters if a +ReadAccessDecision / +OverlayDrained / +MemoryRoiAttributed / +OutputCommentaryRejected is
  chosen — in which case bump from the LIVE grep at edit, never from a doc number (W-9).

================================================================================
J-103 — READ-ACCESS LADDER + READ-TOOL INTERCEPTOR + METER (grep evidence)
================================================================================
- RULES ENGINE (the decision substrate; the READ-ARTIFACT predicate family extends it):
    crates/rules/src/predicate.rs, policy_engine.rs, rule_engine.rs, lifecycle.rs, project_rule_seeder.rs  [WIRED, phase-3F]
- I-46 INTERCEPTOR MECHANISM (widened to the Read tool):
    crates/tools/src/bash_validation.rs::validate_read_only:103 (refuse-with-redirect; write-redirection check :143-148);
    validate_mode:284, validate_sed:336, validate_paths:360  [PLANNED I-46, the hard tool-boundary gate]
- RUNGS:
    DKG symbol cards: crates/knowledge/src/dkg/mod.rs  [WIRED, phase-1A]
    B-29 traversal tools (impact_radius/call_flow/symbol_context/list): crates/knowledge (impact API)  [WIRED]
    E-17 adaptive AST chunking FAST/STANDARD/DEEP: crates/knowledge (phase-3K)  [WIRED]; J-25 section-aware read [PLANNED]
    full file = CodeWindow top rung
- PRESENCE CHECK: J-38 context-sufficiency gate [PLANNED] over C-29 IntentPair symbol-coverage [WIRED]
- DECLARED SCOPE: J-78 context-access spec [PLANNED] (the Conductor access_list, EXTENDS I-5)
- METER: J-29 LiveUsageGauge / ContextStatePanel [PLANNED, read-side]; CatRAG 55-87% accounting crates/knowledge/src/pipeline.rs (phase-1E)  [WIRED]
- SLOT: crates/orchestrator/src/context_budget.rs (CodeContext slot, phase-3E)  [WIRED]
- NEW: crates/orchestrator/src/read_ladder.rs
- FLAG: ELAI_READ_LADDER_ENABLE (default-OFF, allowlist row, promotion_plan_id=J-103-promotion)
- SSB: net 0 (ride J-29 read-side gauge) OR +ReadAccessDecision (Pattern-2, Kind {Approve,Downgrade,Deny,EscalateOnMiss})

================================================================================
J-104 — LIVE-INDEX OVERLAY + IDLE CHECKPOINT (grep evidence)
================================================================================
- B-31 IndexingService (the lock lifecycle + blake3 CASCADE the drain reuses):
    crates/orchestrator/src/indexing_service.rs  [WIRED]; kill-switch ELAI_DISABLE_INDEXING (governs the BASE)
- DURABLE STORE: crates/knowledge/src/dkg/mod.rs  [WIRED]
- RETRIEVAL READ ENTRYPOINT (overlay-first consumer): crates/knowledge/src/pipeline.rs  [WIRED]
- EDIT SIGNAL (the overlay capture source, no new hook): crates/tools/src/file_ops.rs
    read_to_string at :320/:354/:382; pre-image captured then discarded at :272/:300 (H-49)  [WIRED]
- IDLE TRIGGER (order-based, TZ-4, not wall-clock): J-100 T-5 watchdog promotion [PLANNED] / I-17 step-no-progress [WIRED]
- NEW: crates/orchestrator/src/index_overlay.rs
- FLAG: ELAI_LIVE_OVERLAY_ENABLE (default-OFF, allowlist row, promotion_plan_id=J-104-promotion)
- SSB: net 0 (ride B-31 indexing rows) OR +OverlayDrained (Pattern-2)
- INVARIANTS: overlay-wins-on-conflict (tombstone shadows base) + atomic drain (B-31 lock) = byte-consistent durable store (GD-2)

================================================================================
J-105 — MEMORY ROI ATTRIBUTION + ROI-DRIVEN CURATION + G-3 CENTRALITY OBSERVE-ONLY (grep evidence)
================================================================================
- crates/memory (E-5 substrate, PARTIAL): src/store.rs, mem0_dispatcher.rs, retrieval.rs (the E-5 retrieval-into-prompt path,
    the centrality rung rides it), fact.rs, decay.rs + decay_scheduler.rs (the J-21 TTL->activation-decay path the ROI score
    drives), contradiction.rs, citation_validity.rs, nli_onnx.rs, semantic_contradiction.rs, extractor.rs
- J-21 (DEPENDED-ON; the quarantine + commit-aware validation + the decay path) [PLANNED]
- OUTCOME SIGNAL: H-43 execution-grounded fitness (FitnessEvaluated 221) [PARTIAL]
- SCORE SHAPE: H-39 abstention_score.rs (AbstentionScored; abstain bit READ from D-16 RD-1) [PARTIAL]
- REUSE-VALUE: K-18 recurrence-hit-rate [PLANNED]
- CREDIT SUBSTRATE: C-45/C-47 OrchestratorCaseBaseSsbEmitter + [agent.credit_attribution] config [DARK]
- CENTRALITY SIGNAL: PageRank crates/knowledge/src/pagerank/ (phase-1D) [WIRED] + B-29 fan-in (impact API) [WIRED]
- QUERY-SHAPE CLASSIFIER (deterministic, the only condition the centrality rung fires): D-10 select_grounding_mode
    (grounding_mode.rs:157, locked_keys_registry.toml) [WIRED]
- AUDITOR FIREWALL (NON-NEGOTIABLE): H-31 auditor/critic/adversary get ZERO memory injection (arXiv:2510.09905) [PARTIAL]
- TRIM/DEMOTE ROW (prefer-reuse, payload never carries content): H-33 MemoryGovernanceEvent (211) / J-21 MemoryValidationEvent
- NEW: crates/memory/src/roi_attribution.rs
- FLAGS: ELAI_MEMORY_ROI_ATTRIBUTION_ENABLE + ELAI_MEMORY_CENTRALITY_RERANK_ENABLE (default-OFF, allowlist rows,
    promotion_plan_id=J-105-promotion; centrality stays observe-only longest)
- SSB: net 0 (H-33/J-21 rows) OR +MemoryRoiAttributed (Pattern-2, entry-hash + score, NEVER content)

================================================================================
J-106 — OUTPUT META-COMMENTARY ENFORCEMENT (op-emission roles) (grep evidence)
================================================================================
- I-32 ROLE SCOPING (the scope source; the OUTPUT-side twin extends it): crates/orchestrator/src/report_fns.rs (role-system-prompt
    build, the role classification) + role_contracts.rs (grep-confirmed present this session) + i32_role_scoping_contract.rs [PLANNED/BUILT]
- I-47 SCREAM PATH: crates/tools/src/prompt_hardening.rs (dropped_row_count; the dropped/violating-row idiom) [PLANNED]
- I-46 FROZEN-FIXUP-TABLE PATTERN (the banned-phrase table mirrors it): crates/tools/src/bash_validation.rs [PLANNED]
- E-35 eval-metric seam [PARTIAL] + E-49 coding metrics [PARTIAL] (the action-density corpus)
- DOCTRINE (the HOUSE evidence, not a paper): .claude/rules/output-formatting.md + the global "act, don't narrate" /
    measured-Fable-signature rule (result-first openings, ~4 tool-actions per prose block, low words/message)
- HD-3 BOUNDARY (load-bearing): grammars/gates ONLY on pure op-emission, NEVER constrained-decode reasoning roles
- NEW: crates/orchestrator/src/output_commentary_gate.rs
- FLAG: ELAI_OUTPUT_METACOMMENTARY_GATE_ENABLE (default-OFF, allowlist row, promotion_plan_id=J-106-promotion);
    observe-only until the reasoning-corpus false-positive rate is bounded at 0
- FROZEN TABLE in .claude/gate-manifest.txt (R8, mutable=0)
- SSB: net 0 (existing emission/violation row) OR +OutputCommentaryRejected (Pattern-2)

================================================================================
UNIVERSAL BINDING (all four; from the reasoning file §0 — not restated per-file)
================================================================================
- ID-5 / GD-1: every paper number (MutaGReP 95.7%, YoloFS 55%, Mem0 +2%, Cursor 10-min, +28.5pp, Ahad 87.5%) is DESIGN
    RATIONALE; the done-gate is ELAI-MEASURED on an ELAI fixture/gold set, NEVER a paper constant.
- TD-3: every policy/gate/ranking decision is deterministic; the LLM is advisory, never the load-bearing decision.
- Vertical-slice PRE_GATE: each unit ships producer + in-plan consumer + smoke through a REAL entrypoint (wiring-enforcement).
- Every default-OFF flag = a wire-dark-allowlist.toml row (type/owner/expiry/promotion_plan_id/smoke) or it FAILS R9.
- New SSB action_type = Pattern-2 single-variant-with-Kind + full 9-tuple golden-lock lockstep + TOTAL_VARIANTS bump from LIVE.
- NO sdl-mcp CODE copied (license bans embedding in a for-sale product; ELAI has the K-plan GTM). Design ideas only.

DROPPED / DEFERRED (recorded so never re-litigated — reasoning file §5):
- G-4 grounding-ratio/freshness SCORE: DROPPED from the product set (harness-scope + already-covered-in-spirit by
    elai_map_freshness.py + spec_link_check + I-39; report(11) §3 found no published score->accuracy correlation). RIDER:
    a future harness pass may flip elai_map_freshness.py advisory->fail-closed; OUT of these J files.
- G-5 end-to-end $/correct-answer metric: NOT its own file; the live savings meter folds into J-103's done-gate; the bigger
    cost-curve is K-18/J-82 bench territory (RIDER, avoid a GD-1 duplicate bench).
- G-3 standalone centrality file: NOT authored; folded into J-105 as a default-OFF observe-only rung (research downgrade).
- CARRY-FORWARD human-sim E2E tester: DEFERRED to its own intake (bigger + orthogonal to this context/memory cluster).

SOURCES
- workflow/reasoning/jplan-sdl-context-memory-readaccess-2026-06-26.md (the PHASE-4 analysis; §0 doctrine, §1..§4 the four files, §5 dropped/deferred, §6 authoring checklist).
- workflow/reasoning/elai-gaps-sdl-ai-setup-2026-06-26.md (the gap list G-1..G-7).
- Downloads/deep-research-report(11).md (G-3/G-7/G-4 grounding) + (12).md (G-1/G-2/G-5/G-6 grounding).
- docs/thefinalglance/ELAI-FEATURE-MAP.md (read in full this session; every composition anchor grep-confirmed).
