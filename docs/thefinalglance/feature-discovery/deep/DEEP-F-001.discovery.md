# DEEP-F-001 deep-source feature discovery

## Scope and evidence contract

This artifact is a source-isolated deep read of exactly the eight original files assigned to DEEP-F-001. The orientation source was 06-F-SUMMARY_INDEX.discovery.md; it was used only to locate summary claims and never to upgrade an original-plan assertion into implementation fact. No original from any other deep-read chunk was opened.

Evidence labels:

- E — explicit in the named original source. “Shipped,” “confirmed,” or “as-built” remains a source assertion because implementation files were outside this chunk.
- I — bounded composition inference across the eight sources.
- A — unresolved ambiguity or missing contract.
- C — internal conflict, stale reconciliation, or mutually incompatible explicit statements.
- D — addition or correction relative to 06-F-SUMMARY_INDEX.discovery.md.

Anchors are one-based line numbers in the exact hashed files below. Repeated source language is consolidated into one atomic record with every corroborating anchor; this avoids counting the same behavior twice.

## Exact processed manifest and integrity verification

Manifest section: DEEP-SOURCE-MANIFEST.md L611-L625.

| File | Manifest lines | Manifest bytes | Verified SHA-256 | Result |
|---|---:|---:|---|---|
| docs/thefinalglance/F-plan/F-0_INDEX.md | 233 | 17485 | e3ee054392549bdfe5d1b2a98d1c6655fa69315af91690fdcc34e0970eb0d868 | exact |
| docs/thefinalglance/F-plan/F-4_test_integrity_cant_lie.md | 217 | 36324 | ae65098ff39db55e08a04dc05ac63a536ec198e2eb0589e8d378659628f72dc5 | exact |
| docs/thefinalglance/F-plan/F-8_scf_symbolic_conflict_selfmod.md | 242 | 42472 | 79b45385f3006784902c4f74f680ff8dbff3b8658cba86ec7dd74836b50c0e02 | exact |
| docs/thefinalglance/F-plan/F-9_hallucinated_dep_writetime_gate.md | 238 | 41399 | f6ec6d435203788582eabe709ac8fa32771d435839f5b92a402b971818c0d29f | exact |
| docs/thefinalglance/F-plan/F-10_security_monotonicity_iter_guard.md | 220 | 39238 | 25c9f983cbf1d39ec8e497dbdde633755f02de9130f6d1d962062065bba4d95f | exact |
| docs/thefinalglance/F-plan/F-11_close_one_config_generation.md | 258 | 50021 | 5605f2402d271d79445f0dfab3d0f88a5ece101a715100b0f7f18cdbdd1f0c3c | exact |
| docs/thefinalglance/F-plan/F-12_semantic_contradiction_local_nli.md | 267 | 55558 | c4ea77cbd0e5f0d0b4650b35ea2c32e2c5746cb678ee206795415d2e1e0cc922 | exact |
| docs/thefinalglance/F-plan/F-13_session_archive_schema_bump.md | 513 | 39521 | 2ec94c5367fa7955b34fb38c36583e06dff6836a3b8a8a37e41f729199c67ad6 | exact |
| **Total** | **2188** | **322018** | — | exact |

Verification used sha256sum and wc -l -c against every path. All eight hashes, all sixteen per-file counts, and both totals match the manifest.

## Per-file explicit ledgers

### F-0_INDEX.md — doctrine, inventory, sequencing, and extension registry

| Atomic ID | Evidence | Explicit feature or behavior | Source anchor |
|---|---|---|---|
| DF001-F0-E001 | E | The index is dated 2026-06-01, synthesizes ten grounded VibeSec/SHARD proposals, and names verification run wf_b959963a-db6 plus B/C/D/E predecessors. | L1-L7 |
| DF001-F0-E002 | E | Every F-N execution has a mandatory read order: synthesis rationale, architecture summary, E-30 format template, night-run handoff, this index, then F-SUMMARY_INDEX. | L9-L15 |
| DF001-F0-E003 | E | The motivating audit reports approximately 52% shipped-wired, 15% shipped-wire-dark, 17% shipped-env-gated, 10% partial, 5% not shipped, with roughly 40–45% collectively dark/gated/partial and about 232 producer-only backend signals. | L16-L26 |
| DF001-F0-E004 | E | F-plan’s sole job is to make wiring and integration a hard, executed, non-negotiable done gate for ten mechanisms. | L28-L30 |
| DF001-F0-E005 | E | Status vocabulary is PLANNED, IN PROGRESS, DONE, and DEFERRED; all ten were PLANNED at generation, DONE means the full doctrine passes, and DEFERRED names RunPod/external gating. | L32-L36 |
| DF001-F0-E006 | E | W-1 makes python .claude/tools/wiring_contract_check.py --bn-file <F-N>.md --run-trigger returning exit 0 a mandatory completion gate. | L47-L55 |
| DF001-F0-E007 | E | W-1 requires every produces_symbol to resolve as DEF/SCIP_DEF in elai.map and every consumes_symbol to have a real CALL/REF. | L47-L51 |
| DF001-F0-E008 | E | W-1 requires each reachable_from anchor to be reachable by CALL_RESOLVED BFS within depth 8 and stub_ban to find no forbidden literal. | L49-L52 |
| DF001-F0-E009 | E | W-1 requires the live observable trigger to run and emit the expected SSB row and artifact; cargo-check green is necessary but insufficient, and the FSM may not transition to COMPLETE without validator PASS. | L51-L55 |
| DF001-F0-E010 | E | W-2 permits a disable-only kill switch but requires the default production path to be on and wired. | L57-L62 |
| DF001-F0-E011 | E | A staged default-off rollout must flip to default-on and smoke within the same F-N; completion requires an observable run without an environment override. | L57-L62 |
| DF001-F0-E012 | E | W-3 requires reachability from a real CLI/slash command, FSM transition/evidence gate, PreToolUse/PostToolUse hook, or Tauri command; test-only callers are wire-dark. | L64-L68 |
| DF001-F0-E013 | E | W-4 requires every human-visible SSB action or signal to name a Svelte/Tauri frontend pair and pass cross_lang_diff; only explicitly justified pure-internal gates are exempt. | L70-L75 |
| DF001-F0-E014 | E | W-5 defines observable_effect as a human-runnable command or dashboard action that produces both a live ssb.db row and an inspectable file/stdout artifact; a unit test is not sufficient. | L77-L83 |
| DF001-F0-E015 | E | A RunPod-gated feature must still ship a deterministic fixture harness through the same production code path and keep the live run as a separate gated checkbox. | L81-L83 |
| DF001-F0-E016 | E | W-6 requires at least one dashboard-verified acceptance item, with a one-line N/A justification only for truly headless work. | L85-L88 |
| DF001-F0-E017 | E | W-7 requires authoring-time elai.map confirmation of every existing consumed symbol, reachability anchor, and modified target, followed by validator re-check. | L90-L94 |
| DF001-F0-E018 | E | W-8 requires produced callable symbols to be consumed or reachable inside the same F-N; later wiring is not acceptable. | L96-L98 |
| DF001-F0-E019 | E | W-9 requires new SSB actions to traverse the golden lock, name a schema bump, pass action_type_lock and cross-language drift, and prefer Pattern-2 one-variant-with-kind collapse; schema numbers are expressed relative to baseline 191. | L100-L104 |
| DF001-F0-E020 | E | W-10 requires network/subprocess gates to split an in-process, genuinely fail-closed Tier 1 from a separately gated Tier 2 and forbids advertising fail-closed behavior for a fail-open axis. | L106-L110 |
| DF001-F0-E021 | E | Global completion requires cargo check, sole-auditor PASS, wiring trigger PASS, cross-language PASS when applicable, dashboard E2E, and a memory-writer run. | L112-L115 |
| DF001-F0-E022 | E | The ten-session catalog maps C1..C10 identically to F-1..F-10 and records title, steal/new classification, novelty, and tier for each. | L117-L134 |
| DF001-F0-E023 | E | F-1 is a genuinely-new hybrid Tier-1 unified fail-closed write gate. | L121-L123 |
| DF001-F0-E024 | E | F-2 is a genuinely-new hybrid with Tier-1 rules and a Tier-2 measured insecure-rate number. | L123-L124 |
| DF001-F0-E025 | E | F-3 is a genuinely-new ToolFilter-first capability with full CaMeL deferred to Tier 3. | L124-L125 |
| DF001-F0-E026 | E | F-4 is a partially-covered hybrid Tier-1 test-integrity gate. | L125-L126 |
| DF001-F0-E027 | E | F-5 is a genuinely-new hybrid Tier-3 staleness cascade sequenced after B-30. | L126-L127 |
| DF001-F0-E028 | E | F-6 is a partially-covered hybrid Tier-3 learned-skill library. | L127-L128 |
| DF001-F0-E029 | E | F-7 is a genuinely-new hybrid Tier-3 split between lattice proof and CBR ranking. | L128-L129 |
| DF001-F0-E030 | E | F-8 is a genuinely-new hybrid Tier-3 symbolic-conflict check scoped to self-modification. | L129-L130 |
| DF001-F0-E031 | E | F-9 is a partially-covered hybrid Tier-2 dependency gate sequenced after D-11. | L130-L131 |
| DF001-F0-E032 | E | F-10 is a genuinely-new Tier-2 security-monotonicity guard with classifier-first sequencing. | L131-L134 |
| DF001-F0-E033 | E | The schema registry reserves F-1 VibeGateBlocked kinds for code, secret, dependency, and skill axes, with only code/secret in Tier 1. | L136-L141 |
| DF001-F0-E034 | E | The schema registry names security_rule_blocked and insecure_rate_measured for F-2, and tool_filter_committed, untrusted_var_quarantined, plus a reserved capability_flow_blocked for F-3. | L141-L143 |
| DF001-F0-E035 | E | The schema registry names test_integrity_verified and anti_fabrication_block plus TestOutcomePayload.stdout_hash for F-4. | L143-L144 |
| DF001-F0-E036 | E | The schema registry names staleness_quarantine_broadcast for F-5 and skill_acquired, skill_trust_promoted, and skill_demoted_stale for F-6. | L144-L146 |
| DF001-F0-E037 | E | The schema registry names safety_lattice_verified and template_utility_ranked for F-7, symbolic_conflict_detected for F-8, write_time_slopsquat_blocked for F-9, and security_class_regression_blocked for F-10. | L146-L149 |
| DF001-F0-E038 | E | Tier-1 sequencing is F-2a plus F-1 co-ship, then F-4, then F-3 phase 1; these gates require no network, RunPod, or subprocess at gate time. | L151-L156 |
| DF001-F0-E039 | E | Tier-2 sequencing is F-2b, F-9 after D-11, and F-10 classifier-first; Tier 3 sequences F-6, F-5, F-8, F-7, then F-3 phase 2 under its overhead gate. | L157-L163 |
| DF001-F0-E040 | E | The headline move is F-1 Tier 1 plus F-2a into F-2b, producing an auditable and reproducible pre-write before/after insecure-rate grounded by SSB and ELAI_SEED. | L165-L170 |
| DF001-F0-E041 | E | All F work inherits BLAKE3-only hashing, no production unwrap/expect, model.generate plus LogitsProcessor, and FA2 on the 7B model. | L172-L178 |
| DF001-F0-E042 | E | Every dependency must be verified for name, version, and last-published date; a new external dependency requires user confirmation, and CLI subprocesses require Tier-C isolation. | L178-L181 |
| DF001-F0-E043 | E | Work proceeds one change at a time; the auditor alone runs tests and the main LLM does not. External-behavior claims require grounded citations and known overstatements must not be reintroduced. | L181-L185 |
| DF001-F0-E044 | E | The session log begins empty and, on shipment, records context compiler, integration explorer, spec grounder, executor, auditor/fixer, wiring validator, dashboard smoker, memory writer, and committing. | L187-L192 |
| DF001-F0-E045 | E | Four post-F-10 extensions were added 2026-06-02 and are explicitly PLANNED/implementation-pending. | L194-L200 |
| DF001-F0-E046 | E | F-2c adds four low-false-positive CWE rules on registered Python/TypeScript grammars, then runs the existing SeCodePLT bench without a new dependency, ast-adapter edit, or SSB action. | L201-L204 |
| DF001-F0-E047 | E | F-11 closes one config-only DGM generation, explicitly calls the otherwise-bypassed security filter, remains behind darwinian-evolve plus ELAI_DARWINIAN_LOOP_ENABLE, and adds darwinian_admit. | L205-L211 |
| DF001-F0-E048 | E | F-12 adds a local deterministic-argmax NLI pre-filter, reuses MemoryConflictDetected with semantic_contradiction, gives audit_conflicts its first production caller, keeps the model ops-gated, and rejects a cloud judge. | L212-L217 |
| DF001-F0-E049 | E | F-13 minimally archives stale SSB state before the destructive schema-drop, adds no periodic committer/git/unified store, leaves W-2 open, and adds session_archived. | L218-L224 |
| DF001-F0-E050 | E | Extension schema effects are F-11 plus one event, F-13 plus one event, F-12 no bump by field reuse, and F-2c no event. | L223-L224 |
| DF001-F0-E051 | E | The prescribed next step is the Tier-1 F-2a/F-1 headline move followed by F-2b when a 2x3090 RunPod window exists, always under W-1..W-10. | L226-L233 |

### F-4_test_integrity_cant_lie.md — artifact-bound test claims

| Atomic ID | Evidence | Explicit feature or behavior | Source anchor |
|---|---|---|---|
| DF001-F4-E001 | E | F-4 adds TestOutcomePayload.stdout_hash: Option<String>, captures an orchestrator-managed artifact hash, and blocks an unproved claimed PASS at AUDITING to MEMORIZING. | L1-L5 |
| DF001-F4-E002 | E | The scoped gate is Tier 1, in-process and fail-closed, estimated at 6–9 hours; C-22/C-27/C-28 contribute hashing, replay stability, and mutation evidence but none is a hard blocker for the field-plus-gate. | L4-L6 |
| DF001-F4-E003 | E | F-4 is parallel-safe with F-1/F-2 subject to coordinated SSB edits, but not with concurrent evidence-gate-registry or TestOutcomePayload edits. | L6 |
| DF001-F4-E004 | E | Existing TestOutcomePayload contains test_name, passed, and duration_ms but no captured-output binding. | L13-L15 |
| DF001-F4-E005 | E | emit_test_outcome is the single funnel with a baby-step caller driven by an auditor narrative and an integration-checkpoint caller driven by an orchestrator-run result. | L15-L16 |
| DF001-F4-E006 | E | The plan’s process-slot question is declared pre-resolved: the integration checkpoint already exposes captured output; the job is to hash it and treat the baby-step None as visible failure input. | L16-L20 |
| DF001-F4-E007 | E | AntiFabricationGate uses EvidenceGate on the Auditing to Memorizing edge; journal-side TestOutcomePayload is distinct from in-memory evidence::TestOutcome and is intended as the human-queryable truth. | L17-L18 |
| DF001-F4-E008 | E | The feature explicitly does not determine whether output satisfies task semantics; that requires spec-pinned expectations in a later F-N. | L20, L61-L64 |
| DF001-F4-E009 | E | C-22 supplies the exact BLAKE3 pattern and additive Option-field contract; C-27 makes hashes replayable but replay is not an F-4 done gate; C-28 is orthogonal. | L22-L26 |
| DF001-F4-E010 | E | E-30 supplies golden-lock/Pattern-2 precedent, E-52 supplies evidence-gate registration precedent, and the auditor remains the sole test runner. | L27-L29 |
| DF001-F4-E011 | E | Task 1 updates nine SSB lockstep sites, defaults stdout_hash to None for old rows, and documents None plus passed=true as lack of captured proof. | L47-L50 |
| DF001-F4-E012 | E | Task 2 hashes integration output with BLAKE3, extends emit_test_outcome with an Option hash, and passes None from the baby-step site. | L51-L52 |
| DF001-F4-E013 | E | AntiFabricationGate has name anti_fabrication, transition Auditing to Memorizing, and blocks an integration-suffixed passed row when stdout_hash is absent. | L53 |
| DF001-F4-E014 | E | A block returns EvidenceError with a Block-severity AuditFinding; success emits test_integrity_verified and failure emits anti_fabrication_block. | L53-L55 |
| DF001-F4-E015 | E | The gate is registered in all_gates, changes the expected gate-name count from 9 to 10, has no disable/bypass branch, and forbids production unwrap/expect. | L53 |
| DF001-F4-E016 | E | The two logical outcomes are collapsed into SsbActionType::TestIntegrityEvent plus TestIntegrityEventKind Verified or FabricationBlocked, with payload test_name, passed, optional hash, and a baseline-plus-one schema bump. | L54-L55, L68-L78 |
| DF001-F4-E017 | E | Dashboard surface is get_test_integrity_events plus TestIntegrityEventJson and AuditTimelineWindow fabricationBlocks/red badge; Rust/TS bindings and stub must pass cross-language validation. | L57, L77-L78, L107-L109 |
| DF001-F4-E018 | E | The decision record must explain process provenance, semantic deferment, fail-closed/no-flag behavior, BLAKE3, and Pattern-2/schema choice. | L59 |
| DF001-F4-E019 | E | Giving baby-step tests their own orchestrator-managed slot is explicitly deferred; the current plan only makes their None observable. | L61-L65 |
| DF001-F4-E020 | E | The reconciled consumes contract corrects the planned result type to IntegrationRunResult.output_capture and CriterionResult rather than reasoning::types::TestRunResult. | L79-L92 |
| DF001-F4-E021 | E | The reconciled production route is drive_task’s post-loop gate check before PhaseComplete, plus registry exposure and the Tauri read path; fsm.rs transition is explicitly rejected because it lacks integration rows. | L93-L99 |
| DF001-F4-E022 | E | Stub bans forbid placeholder text, a default-off gate, an LLM override, silent warning on a missing slot hash, non-BLAKE3 hashing, and silent pass. | L100-L106 |
| DF001-F4-E023 | E | The observable fixture exercises the real checkpoint-outcome to emit-outcome to gate route with no environment override, expects a same-superstep failure row or clean success row, blocks the FSM transition, writes an audit artifact, and renders a red badge. | L110-L113 |
| DF001-F4-E024 | E | Deterministic checks require the wiring trigger, real symbols, production registration, BLAKE3 capture, golden/schema update, frontend pair, cross-language parity, and decision document. | L115-L146 |
| DF001-F4-E025 | E | Acceptance requires W-1, default-on behavior, dashboard E2E, a real-path deterministic fixture, nine-site backward-compatible field migration, gate registration, BLAKE3/no unwrap, action round-trip, cargo/auditor green, and no new dependency. | L148-L160 |
| DF001-F4-E026 | E | The contract suite covers Option defaults/round-trip, hashed/unhashed gate decisions, golden action coverage, real-path mismatch blocking, and direct evidence-gate allow/block cases. | L161-L165 |
| DF001-F4-E027 | E | The declared change set spans ssb.rs, task_runner.rs, evidence.rs, Tauri commands, AuditTimelineWindow, TypeScript types/stub, golden lock, two contract files, and a decision file. | L167-L178 |
| DF001-F4-E028 | E | F-4 claims only structural detectability of an integration PASS without captured output, not general impossibility of lying or semantic correctness. | L180-L188, L205-L211 |
| DF001-F4-E029 | E | The integration-name suffix is a pinned correlation discriminator; renaming it could silently disable the gate and must break a contract test. | L205-L207 |
| DF001-F4-E030 | E | A captured but empty stdout must hash the empty byte string so None means no capture; the source flags its own earlier non-empty-only C-22 pattern as potentially wrong here. | L208 |
| DF001-F4-E031 | E | run.output being raw captured stdout versus a summary remains UNVERIFIED and must be documented after reading the implementation. | L209-L210 |
| DF001-F4-E032 | E | F-4 composes with F-1 as “cannot lie/cannot wreck,” can later gain replay confirmation from C-27, and may export events through E-29. | L213-L215 |

### F-8_scf_symbolic_conflict_selfmod.md — symbolic self-mod review

| Atomic ID | Evidence | Explicit feature or behavior | Source anchor |
|---|---|---|---|
| DF001-F8-E001 | E | F-8 adds an LLM-free rule-based pre-admission detector for config/structural-change conflicts against locked keys, parameter bounds, and active tripwires. | L1-L7 |
| DF001-F8-E002 | E | It emits Contradictory, ResourceContention, or CausalViolation and uses WARN-then-escalate on self-modification only. | L3, L9-L15 |
| DF001-F8-E003 | E | The source treats SCF’s 27.9% precision as simulation-only and unvalidated outside its synthetic scaffold, so it is not an ELAI target or hard-block basis. | L7-L12 |
| DF001-F8-E004 | E | The inter-agent BSP half is RunPod-pending and out of scope because parallel cap-3 false positives become noise; Graphiti’s cloud-lane contradiction judge is a future alternative. | L14-L15 |
| DF001-F8-E005 | E | The self-mod stack and apply path are asserted shipped, while C-50 ContractSpec is asserted absent; the new names avoid a future precondition/postcondition namespace collision. | L5, L17-L29 |
| DF001-F8-E006 | E | Existing name-based locked-key checks, numeric promotion/consensus/tripwire gates, and interface set comparison do not answer semantic postcondition conflict. | L11-L13, L19-L27 |
| DF001-F8-E007 | E | F-8 is a pre-gate before two_gate_promote and before ConfigArchive admission; it augments rather than replaces the name-match safety gate. | L19-L23 |
| DF001-F8-E008 | E | ConflictKind has three arms and SymbolicIntent stores asserts and requires vectors of IntentCondition. | L49-L54 |
| DF001-F8-E009 | E | IntentCondition reuses ParameterKey and supports Equals, GreaterThan, LessThan, Locked, Unlocked, and TripwireQuiescent predicates. | L53-L55 |
| DF001-F8-E010 | E | ProcessModel snapshots locked keys, active TripwireKind values, and current numeric bounds. | L54-L56 |
| DF001-F8-E011 | E | detect_conflicts is pure, deterministic, returns all conflicts, and produces SymbolicConflict kind/key/detail records. | L55-L57 |
| DF001-F8-E012 | E | Two adapters translate StructuralChange and ConfigVariant into symbolic intents; conflict_fingerprint produces a BLAKE3 prefix; production code has no unwrap/expect or unnamed thresholds. | L57-L59 |
| DF001-F8-E013 | E | On any conflict the system emits one symbolic_conflict_detected row per conflict. Interactive mode requires operator confirmation; autonomous mode holds pending and escalates. | L61-L63 |
| DF001-F8-E014 | E | A detected conflict cannot be overridden by prompt/candidate content and never silently applies; only an explicit operator-confirm row admits it. | L62-L63, L112-L118 |
| DF001-F8-E015 | E | SymbolicConflictPayload carries fingerprint, kind, key, detail, candidate_id, and fsm_substate in one schema-baseline-plus-one action. | L65 |
| DF001-F8-E016 | E | symbolic_conflicts returns SymbolicConflictRow with seq, time, fingerprint, kind, key, detail, and candidate_id for dashboard consumption. | L67 |
| DF001-F8-E017 | E | get_symbolic_conflicts has feature-on and empty-array feature-off twins; ReasoningDrawer renders per-kind tone badges and the TS stub/invoke handler complete the frontend pair. | L69 |
| DF001-F8-E018 | E | current_process_model and tripwire_quiescent_violated implement the ELAI-specific rule that increased self-mod activity violates an active tripwire’s quiescence requirement. | L71 |
| DF001-F8-E019 | E | The decision document must capture terminal WARN/escalate policy, BSP deferment, collision-safe names, rule definitions, grounding status, and fully in-process tiering. | L73 |
| DF001-F8-E020 | E | The produced-symbol contract enumerates detector types/adapters/helpers, viewer query/row, SSB action/payload, and dashboard command. | L75-L94 |
| DF001-F8-E021 | E | The consumed-symbol contract names live apply/safety/type/tripwire/viewer/dashboard anchors and corrects observe_post_consensus_tripwires to agent_pool.rs. | L95-L105 |
| DF001-F8-E022 | E | A later as-built reachability reconciliation routes CLI run_apply through screen_on_runtime and screen_config_variant_for_conflicts, with screen_config_variant_for_conflicts emitting the event before admission. | L107-L110 |
| DF001-F8-E023 | E | The live trigger applies contradictory_locked_variant.json through CLI or dashboard, expects a pending-not-applied candidate, a matching BLAKE3-prefix row, and ReasoningDrawer rendering. | L125-L128 |
| DF001-F8-E024 | E | Deterministic checks verify core symbols, live admission wiring, five SSB sites, viewer, frontend and TS binding, drift, default-on/no bypass, and the decision record. | L130-L162 |
| DF001-F8-E025 | E | Acceptance requires all three conflict fixtures plus a clean case, operator-confirm semantics, no C-50 names, no BSP scope, live dashboard display, golden/schema parity, cargo/auditor green, and no new dependency. | L164-L177 |
| DF001-F8-E026 | E | Tests cover each conflict kind, clean behavior, stable fingerprint, real apply/hold behavior, SSB round-trip, and an end-to-end temp-database smoke through the production path. | L179-L184 |
| DF001-F8-E027 | E | The declared artifacts include symbolic_conflict.rs, four fixtures, three contract suites, an observable smoke, decision doc, apply/SSB/viewer/module edits, and complete dashboard wiring. | L186-L204 |
| DF001-F8-E028 | E | Before any tightening to hard block, the decision record must measure false-positive rate on at least 20 real self-mod proposals. | L215-L217 |
| DF001-F8-E029 | E | Process-model staleness is mitigated by reading the latest tripwire outcome inside admission; a smoke test must prove freshness. | L218 |
| DF001-F8-E030 | E | Semantic conflict accuracy remains a design hypothesis; SHARD semantic performance underperformed syntactic performance and no ELAI catch-rate may be claimed before measurement. | L220-L221 |
| DF001-F8-E031 | E | F-7 can later supply a proven locked-key set, while F-10 composes as a separate CWE-class monotonicity axis. | L238-L240 |

### F-9_hallucinated_dep_writetime_gate.md — cached registry gate

| Atomic ID | Evidence | Explicit feature or behavior | Source anchor |
|---|---|---|---|
| DF001-F9-E001 | E | F-9 extends a real PyPI/npm existence-and-age checker from PR time to Edit/Write PreToolUse time, adds crates.io, per-ecosystem thresholds, a local cache, live block telemetry, CLI, and dashboard. | L1-L5 |
| DF001-F9-E002 | E | D-11 is explicitly PLANNED/not shipped and is a hard blocker; E-41/E-11 are shipped composition surfaces and B-19 latency budgets are planned. | L7-L13 |
| DF001-F9-E003 | E | The source corrects “shape-only” to a real HTTP checker and narrows the true gaps to invocation timing, cargo coverage, and a hard-coded 730-day age threshold. | L17-L25 |
| DF001-F9-E004 | E | Network lookup is explicitly not a fail-closed hot-path guarantee: cache hits are in-process; misses are Tier-2 best effort and may fail open under registry unavailability. | L27-L29 |
| DF001-F9-E005 | E | Socket Firewall is defense in depth at install time but is moved to an operations runbook and contributes no plan code. | L29 |
| DF001-F9-E006 | E | Existing E-41 dep parsing is a cheap deterministic PR prefilter and lacks Cargo section awareness; F-9 preserves it while adding registry-grounded write-time enforcement. | L31-L39 |
| DF001-F9-E007 | E | _check_cargo calls crates.io through the existing HTTP helper, derives age from updated_at/newest created_at, keeps the existing tuple shape, accepts ecosystem in stdin JSON, and maps network failure to inapplicable. | L54-L57 |
| DF001-F9-E008 | E | Default maximum ages become PyPI 730, npm 730, and cargo 1825 days; an explicit claim overrides the default, and 1825 is a tunable unverified design choice. | L58 |
| DF001-F9-E009 | E | Cargo parsing becomes section-aware for dependencies, dev-dependencies, and build-dependencies and returns DepCandidate{name, Ecosystem} for Pypi/Npm/Cargo while keeping the PR leg compatible. | L60 |
| DF001-F9-E010 | E | DepWriteTimeGate fires only for Edit/Write dependency content/path patterns and returns immediately with no finding on other writes. | L62-L64 |
| DF001-F9-E011 | E | LruExistenceCache is a bounded HashMap plus VecDeque with about a one-hour TTL and no new lru dependency. | L64-L66 |
| DF001-F9-E012 | E | A known-good hit allows without network; a miss invokes RegistryCheck; NotFound or TooNew blocks and emits write_time_slopsquat_blocked; Ok populates the cache. | L64-L65 |
| DF001-F9-E013 | E | RegistryUnreachable allows the write but must emit an advisory row; it may never silently pass. | L65, L109-L115 |
| DF001-F9-E014 | E | The gate registers in build_production_hook_dispatcher/dispatch_pre_tool_use and is default-on; ELAI_DEP_WRITETIME_GATE_DISABLE=1 is the only permitted operator kill switch. | L68, L98-L108 |
| DF001-F9-E015 | E | WriteTimeSlopsquatBlockedPayload stores a BLAKE3 dep-name hash, ecosystem, NotFound/TooNew/RegistryUnreachable kind, optional registry age, and timestamp in one baseline-plus-one action. | L70 |
| DF001-F9-E016 | E | write_time_slop_status aggregates recent rows by ecosystem and verdict; DepsSecurityPanel shows daily block/advisory counts and registers a DrawerKey, Tauri DTO, and TS stub. | L72 |
| DF001-F9-E017 | E | The decision document must name cache capacity, one-hour TTL, fail-open/advisory semantics, cargo threshold uncertainty, Socket deferral, real-checker correction, and E-41 composition. | L74 |
| DF001-F9-E018 | E | The explicit tier split says known-good allows and known-bad blocks on cache hit at T1; cache-miss subprocess is the separately gated, fail-open-with-advisory Tier 2. | L76-L78 |
| DF001-F9-E019 | E | The reconciled produced contract adds RegistryExistenceChecker, StubRegistryChecker, SubprocessRegistryChecker, RegistryVerdict, an agnostic extractor, dep_name_blake3_u64, a dedicated dep-check CLI action, dashboard command, and drawer. | L80-L108 |
| DF001-F9-E020 | E | The real hook is bootstrapped from CLI task and dashboard paths; elai dep-check <path> [--offline] reuses analyze_collect and writes the same event. | L99-L108 |
| DF001-F9-E021 | E | The live fixture uses a Rust use of reqquests, expects a NotFound/cargo row within two seconds, a BLAKE3 prefix, a BLOCK line, and refusal before disk. | L119-L122 |
| DF001-F9-E022 | E | Deterministic checks cover wiring trigger, produced types, Pattern-2 event, hook registration, disable-only default, frontend/drift, BLAKE3, and decision doc. | L124-L159 |
| DF001-F9-E023 | E | Acceptance separates a fixed-not_found real-subprocess fixture from a network-gated 50/50 confirmed-404 and zero-false-positive-on-20-real-deps live run. | L161-L170 |
| DF001-F9-E024 | E | Acceptance also requires Cargo extraction/checking, event round-trip/schema, no new dependency, and backward-compatible E-41 PR contracts. | L168-L171 |
| DF001-F9-E025 | E | Tests cover non-dependency no-op, no-network cache hit, real subprocess block, unreachable advisory plus allow, prompt bypass resistance, event round-trip, Cargo parsing, mock-clock threshold, and dashboard DTO parity. | L188-L194 |
| DF001-F9-E026 | E | The declared change set spans the checker, cargo-aware parser, new hook/cache, hook bootstrap, SSB, CLI, two fixtures, two contracts, decisions, drawer/Tauri/stub, and golden lock. | L196-L214 |
| DF001-F9-E027 | E | The 1825-day floor must be validated empirically and remains explicitly unverified. | L226-L228 |
| DF001-F9-E028 | E | False-fire mitigation restricts Cargo extraction to dependency sections; cache-miss hangs use the checker’s ten-second timeout then fail open with an advisory. | L229-L231 |
| DF001-F9-E029 | E | F-9 must not reimplement D-11, add an LRU crate, or silently unify with F-1; co-sequencing must decide whether to emit both event names or merge under one schema bump. | L232-L234 |
| DF001-F9-E030 | E | Lifecycle composition is write-time registry check, PR-time cheap prefilter, and out-of-plan install-time Socket wrapping. | L236-L238 |

### F-10_security_monotonicity_iter_guard.md — per-loop CWE regression control

| Atomic ID | Evidence | Explicit feature or behavior | Source anchor |
|---|---|---|---|
| DF001-F10-E001 | E | F-10 fingerprints each generated delta by CWE class and blocks/escalates any class present at iteration N but absent at N-1 across REFINE, auto-advance, and self-modification. | L1-L5 |
| DF001-F10-E002 | E | It is a two-session plan: Phase 1 builds/calibrates the classifier and Phase 2 wires the gate as post-write async auditor evidence rather than a per-write blocking hook. | L3-L7, L14-L15 |
| DF001-F10-E003 | E | B-39, B-22/C-9, D-4, and E-7 are asserted shipped prerequisites; the CWE rule data is a new in-repo dependency surface but no crate may be added without confirmation. | L5-L6 |
| DF001-F10-E004 | E | The source motivates the invariant with externally measured +37.6% critical vulnerabilities over five iterations and approximately 11x LLM-versus-human vulnerability injection, while forbidding their use as ELAI targets. | L9-L15, L201-L207 |
| DF001-F10-E005 | E | Existing correctness/score gates are explicitly blind to code-delta security class, and no current classifier, comparison gate, tripwire kind, autonomy axis, or promotion precondition is asserted in the initial-state section. | L17-L25 |
| DF001-F10-E006 | E | Planned classify_delta returns a deterministically ordered BTreeSet of CweClass for a CodeDelta and Language. | L42-L47 |
| DF001-F10-E007 | E | Planned classes include CWE-78, 95, 502, and 22 plus SQL injection, XSS, weak cryptography, and missing authorization; patterns cover Rust, Python, and TypeScript with confidence metadata. | L46-L47 |
| DF001-F10-E008 | E | Planned implementation is in-process GREP/AST patterning, uses BLAKE3, forbids production unwrap, and does not claim an exhaustive CWE taxonomy. | L46-L49 |
| DF001-F10-E009 | E | Calibration uses 50 known-clean historical patches per language and requires false-positive rate below 5% separately for Rust, Python, and TypeScript. | L48-L49 |
| DF001-F10-E010 | E | If any language misses the threshold, patterns are tightened and remeasured; after three failed tightening passes the workflow escalates rather than spirals. | L48 |
| DF001-F10-E011 | E | SecurityMonotonicityGate set-diffs prior versus current classes and returns Monotone or Regressed{introduced, prior}; state is keyed by turn_id, step_id, or variant_id. | L51-L53 |
| DF001-F10-E012 | E | Classifier errors become UNKNOWN and escalate to the auditor rather than silently becoming Monotone. | L92-L99 |
| DF001-F10-E013 | E | Planned REFINE wiring consumes the event stream after candidate commit, schedules a background task, emits a block row, and writes workflow/sessions/{id}/security-monotonicity.log without synchronously stalling the pipeline. | L54 |
| DF001-F10-E014 | E | Planned auto-advance wiring adds a security evidence axis whose regression verdict prevents phase advancement. | L55 |
| DF001-F10-E015 | E | Planned self-mod wiring checks the archive baseline before tau-margin and blocks a variant introducing a new CWE class. | L56 |
| DF001-F10-E016 | E | One SecurityClassRegressionBlocked action carries delta_blake3, introduced/prior classes, loop_kind Refine/AutoAdvance/SelfMod, loop_id, and language under a baseline-plus-one bump. | L57 |
| DF001-F10-E017 | E | SecurityMonotonicityDrawer plus get_security_monotonicity_blocks and ReflectEvent::SecurityRegressed expose both queryable history and live REFINE updates. | L58 |
| DF001-F10-E018 | E | The contract suite requires a regressing and clean delta, nonblocking REFINE scheduling, failed auto-advance evidence, blocked self-mod admission, event round-trip, and calibration regression protection. | L59 |
| DF001-F10-E019 | E | Default operation is active; ELAI_SECURITY_MONOTONICITY_DISABLE=1 is a disable-only kill switch, and prompts/candidates cannot bypass compiled classification. | L60, L92-L99 |
| DF001-F10-E020 | E | The later reconciled as-built contract places classify_delta and CweClass in crates/orchestrator/src/cwe_classifier.rs, returns BTreeSet directly, and uses in-module LazyLock regexes rather than the planned safety-crate TOML. | L62-L76 |
| DF001-F10-E021 | E | As-built REFINE reachability is live_cli.rs:1583 to run_refine_leg to check_monotonicity, then an event row and session log. | L86-L89 |
| DF001-F10-E022 | E | As-built auto-advance reachability is derive_security_monotonicity_evidence querying regression rows and failing the axis when one exists. | L88 |
| DF001-F10-E023 | E | As-built CLI reachability is elai safety monotonicity --fixture <PATH> through safety_cmd.rs. | L89 |
| DF001-F10-E024 | E | The self-mod leg remains wire-dark behind darwinian-evolve and an explicit allowlist entry owned by maintainer, expiring 2026-09-01, with F-11 as the promotion plan. | L90 |
| DF001-F10-E025 | E | The dashboard as-built path is commands_f10.rs::get_security_monotonicity_blocks to SecurityMonotonicityDrawer. | L91 |
| DF001-F10-E026 | E | The observable fixture introduces os.system CWE-78 during REFINE with no env override, expects one event within an auditor cycle, a populated delta hash, a matching session log, and dashboard rendering. | L103-L106 |
| DF001-F10-E027 | E | The retained deterministic checks still look for the obsolete safety-crate classifier/TOML and chat_reflect/task_runner/promotion_gate call sites despite the as-built reconciliation. | L108-L140 |
| DF001-F10-E028 | E | Acceptance requires measured per-language calibration, dashboard E2E, all three loop legs, single-variant schema parity, deterministic fixture, separately gated live Qwen run, cargo/auditor/memory-writer completion, and no external dependency. | L142-L154 |
| DF001-F10-E029 | E | The live model checkbox remains RunPod/sidecar-pending and is satisfied only by an organically introduced CWE class caught during a remote REFINE turn. | L150-L152 |
| DF001-F10-E030 | E | The declared file plan still names safety-crate module/TOML/bench paths, while as-built reconciliation says those artifacts do not exist and the classifier lives in orchestrator. | L178-L200, L64-L70 |
| DF001-F10-E031 | E | The architecture choice between a new TripwireKind and a standalone gate remains open; default recommendation is a standalone gate emitting through existing tripwire/SSB paths. | L209-L211 |
| DF001-F10-E032 | E | If Rust cannot meet the false-positive gate, the plan permits Python/TypeScript first with Rust warn-only rather than indefinite tuning. | L212 |
| DF001-F10-E033 | E | Async ordering permits a turn to commit before classification, but the auditor is expected to block the superstep before Memorizing. | L213 |
| DF001-F10-E034 | E | F-1/F-2 predicates may later replace duplicated classifier rules; E-29 can export events and MAST can classify them downstream. | L218-L220 |

### F-11_close_one_config_generation.md — one guarded DGM generation

| Atomic ID | Evidence | Explicit feature or behavior | Source anchor |
|---|---|---|---|
| DF001-F11-E001 | E | F-11 replaces an inert synthetic Darwin loop with one closed config-only propose, real-evaluate, guard, insert, select-parent, and mutate generation. | L1-L5 |
| DF001-F11-E002 | E | Acceptance is loop closure plus a guard catching a trap mutant, never a positive score or capability claim. | L3-L5, L25-L27 |
| DF001-F11-E003 | E | Existing ConfigArchive math uses lambda 10, alpha_0 0.5, weights s_i times h_i, and retains active-variant ancestors during capacity pruning. | L7-L8, L19-L23 |
| DF001-F11-E004 | E | insert_variant, select_parent, and sample_variant are asserted to have only module, contract-test, and re-export references and no production caller. | L8-L9, L21-L23 |
| DF001-F11-E005 | E | Current darwin-loop is a live CLI but uses deterministic alternating synthetic reward, terminates RUNPOD-PENDING, and never calls ConfigArchive. | L9, L23-L25 |
| DF001-F11-E006 | E | Existing guards are BLAKE3 tripwire, live-candidate trap scoring, the separate-path is_security_sensitive keyword filter, and two_gate_promote. | L10-L13, L31-L40 |
| DF001-F11-E007 | E | is_security_sensitive covers auth, credential, password, secret, token, ssh, tls, ssl, private_key, and api_key, but existing archive admission bypasses it. | L11, L34-L37 |
| DF001-F11-E008 | E | Mutation is restricted to ConfigVariant config/template changes, never StructuralChange source rewrite, preserving bounded capacity and avoiding Layer-3 human_approved by construction. | L3, L27, L37, L64-L70 |
| DF001-F11-E009 | E | F-11 is not parallel-safe with concurrent ConfigArchive, Darwin CLI, promotion-gate, or trap-task edits; F-8 is orthogonal if schema and apply-path edits remain separate. | L15 |
| DF001-F11-E010 | E | Parent proposal uses select_parent_with and EntropySource for ELAI_SEED replay or seeds a root variant when the archive is empty. | L62-L66 |
| DF001-F11-E011 | E | Evaluation calls one real external_benches suite and derives alpha in [0,1], with no mock or retained synthetic fallback. | L64-L70 |
| DF001-F11-E012 | E | Guard order is tripwire pre-exec, trap scoring on live config/eval log, is_security_sensitive, then two_gate_promote; order is normative. | L67-L68, L80 |
| DF001-F11-E013 | E | Any guard rejection emits RejectedByGuard and prevents insertion; only all-pass candidates enter the archive, after which select_parent proves loop closure. | L67-L70 |
| DF001-F11-E014 | E | One DarwinianAdmit action uses kind Admitted, RejectedByGuard, or LoopClosed and payload variant_id, parent_id, alpha, optional guard name, candidate BLAKE3, and seed. | L74 |
| DF001-F11-E015 | E | darwinian_admits returns seq/time/kind/variant/parent/alpha/rejecting-guard/fingerprint for the dashboard. | L76 |
| DF001-F11-E016 | E | get_darwinian_admits plus ReasoningDrawer “Darwinian Admits,” tone badges, TS stub, invoke registration, and feature-off empty array form the frontend pair. | L78 |
| DF001-F11-E017 | E | The decision record must preserve config-only scope, security-bypass fix, guard order, real trap input, unconfirmed tau/K_m, no-uplift acceptance, grounding table, and deferred multi-generation tier. | L80 |
| DF001-F11-E018 | E | Production activation deliberately requires both cargo feature darwinian-evolve and ELAI_DARWINIAN_LOOP_ENABLE=1. | L72, L110-L113 |
| DF001-F11-E019 | E | The source declares this dual opt-in a deliberate W-2 deviation for autonomous self-modification rather than an unfinished default-off wire. | L72, L177-L180, L236, L243-L244 |
| DF001-F11-E020 | E | A reviewer-rejected W-2 exception has a stated fallback: default-on dry-run mode where insertion is preview-only without the flag. | L243-L244 |
| DF001-F11-E021 | E | The observable command fixes seed 42, three mutants, one generation, a feature build, and two environment variables, including an absolute conductor-worktree fixture path. | L129-L132 |
| DF001-F11-E022 | E | Expected evidence is at least one guard rejection and exactly one LoopClosed row, each with a 16-hex BLAKE3 prefix, plus ReasoningDrawer rendering. | L129-L132 |
| DF001-F11-E023 | E | Deterministic checks require driver existence, synthetic-line absence, archive calls, all four guards, action/viewer/frontend parity, BLAKE3, decision doc, and feature-enabled workspace check. | L134-L175 |
| DF001-F11-E024 | E | Acceptance requires the first production archive caller, real benchmark, ordered guards, config-only mutation, unconfirmed threshold disclosure, UI E2E, golden/schema parity, and no new dependency. | L177-L193 |
| DF001-F11-E025 | E | tau defaults to 0.05 while tau/K_m are explicitly unconfirmed; calibration on at least 20 real self-mod proposals is a follow-up prerequisite for tightening. | L10, L31-L34, L187, L230-L233 |
| DF001-F11-E026 | E | Contract tests cover deterministic loop closure, a fixture for each guard rejection, ordered execution, event round-trip, real CLI smoke with at least two rows, and absence of source rewrite. | L195-L201 |
| DF001-F11-E027 | E | The declared change set modifies darwin_loop_cmd and SSB/viewer/dashboard files and creates four mutant fixtures, four contracts, a CLI smoke, and a decision record. | L203-L219 |
| DF001-F11-E028 | E | No Springdrift/DGM uplift is claimed: DGM’s published gains came from many iterations and deceptive dips, unlike this one-generation config-only scope. | L221-L228, L235 |
| DF001-F11-E029 | E | Multi-generation population mutation, cross-domain canary, promotion voting, and deceptive-dip study remain RunPod 2x3090 deferred. | L5, L191, L237, L252, L254-L256 |
| DF001-F11-E030 | E | F-8 may add a semantic admission review and F-11 the archive guard chain; threshold calibration must precede any hard capability-gain claim. | L254-L256 |

### F-12_semantic_contradiction_local_nli.md — local NLI trigger over memory facts

| Atomic ID | Evidence | Explicit feature or behavior | Source anchor |
|---|---|---|---|
| DF001-F12-E001 | E | F-12 adds a second-pass local NLI cross-encoder for semantically opposed facts with different keys, while preserving the existing non-destructive store. | L1-L5 |
| DF001-F12-E002 | E | Existing audit_conflicts groups current facts by byte-identical project/scope/key and compares different JSON values; exact-key tier dominance may supersede, otherwise Surface preserves both rows. | L7-L10, L17-L23 |
| DF001-F12-E003 | E | audit_conflicts is asserted wire-dark with three test-only callers; F-12 gives it its first production call in the live memory tick. | L29-L32, L72-L75 |
| DF001-F12-E004 | E | Existing supersede is transactional replacement and soft_close is non-destructive single-row closure; F-12 changes the trigger, not the storage schema/path. | L19-L23, L31-L33 |
| DF001-F12-E005 | E | Existing MemoryConflictDetected has free-form conflict_kind, so semantic_contradiction reuses it without a new action or schema bump. | L10, L35, L38 |
| DF001-F12-E006 | E | The local model reuses ort/ONNX/tokenizer infrastructure and deterministic-compute settings; only the NLI model data file is new. | L9, L25, L36 |
| DF001-F12-E007 | E | Cloud LLM judgment is rejected because temp-zero/fixed-seed inference is not replay deterministic; local inference is described only as practically stable, not bit-identical. | L23-L27 |
| DF001-F12-E008 | E | F-5 remains an orthogonal age-based sibling pass; DKG alias resolution, cross-scope resolution, and mem0 write-path changes are out of scope. | L11-L13, L21, L27, L33-L34 |
| DF001-F12-E009 | E | NliJudge exposes classify(premise,hypothesis) returning Result<NliVerdict, MemoryError>; the fake/live implementations share this seam. | L58-L62 |
| DF001-F12-E010 | E | NliLabel is Entailment, Neutral, or Contradiction and NliVerdict includes confidence. | L62 |
| DF001-F12-E011 | E | ContradictionCandidate holds older/newer facts; collect_cross_key_candidates pairs different keys only within the same project and scope, using list_current and avoiding a full cross product. | L63 |
| DF001-F12-E012 | E | adjudicate_semantic renders stable text, invokes the judge, and only closes the older row on Contradiction at or above threshold. | L64-L68 |
| DF001-F12-E013 | E | Event-time closure sets older.valid_to_ms to newer.valid_from_ms, not now_ms. | L64, L85, L186-L197 |
| DF001-F12-E014 | E | Neutral, entailment, low confidence, or judge error yields SurfaceToUser and never auto-closes. | L64-L68, L126-L133 |
| DF001-F12-E015 | E | SEMANTIC_CONTRADICTION_CONFIDENCE_THRESHOLD is 0.85 and is an ELAI design value requiring measured precision before loosening. | L66, L87, L253-L257 |
| DF001-F12-E016 | E | pair_fingerprint uses a BLAKE3 hex prefix; production paths forbid unwrap/expect. | L67-L68 |
| DF001-F12-E017 | E | OnnxNliJudge stores session/tokenizer/label order, uses deterministic compute, one intra-op thread, full graph optimization, model-dir convention, three-logit argmax, and max-softmax confidence. | L70 |
| DF001-F12-E018 | E | Model variant is configuration-selected by VRAM tier: xsmall for 8 GB and base for 24 GB, using the cross-encoder/nli-deberta-v3 family. | L70 |
| DF001-F12-E019 | E | Model construction fails loudly if its files are absent, but the production tick catches absence, logs once, and skips the semantic pass while exact-key audit_conflicts continues. | L70, L72, L81-L87 |
| DF001-F12-E020 | E | The live tick runs audit_conflicts first, then cross-key adjudication, and emits one MemoryConflictDetected semantic_contradiction row for both InvalidatedOlder and SurfaceToUser outcomes. | L72-L75 |
| DF001-F12-E021 | E | ELAI_SEMANTIC_CONTRADICTION_KILL is disable-only; default behavior runs when the model is present, with no enable flag. | L72, L126-L133 |
| DF001-F12-E022 | E | get_semantic_contradictions filters MemoryConflictDetected history and MemoryFactsDrawer renders older/newer IDs, label, confidence, action, and a disposition tone. | L77 |
| DF001-F12-E023 | E | The decision record must cover cloud rejection, practical determinism, unchanged storage, confidence-safe behavior, event reuse, deferred axes, and measured/ported/unverified grounding. | L79 |
| DF001-F12-E024 | E | Tier 2 stages ONNX plus tokenizer under models/onnx, records label order and tokenizer, and keeps the code path fail-safe while absent. | L81-L83 |
| DF001-F12-E025 | E | The fixture harness uses a deterministic fake judge but the real candidate, adjudication, and soft-close path; it verifies event-time closure, neutral survival, and fingerprint stability. | L85 |
| DF001-F12-E026 | E | The separately gated live run requires a staged model, a known cross-key pair, live event/UI evidence, and measured precision on at least 20 real candidates before threshold widening. | L87 |
| DF001-F12-E027 | E | The wiring contract enumerates detector types, threshold, fingerprint, live ONNX judge, dashboard command, and deferred model artifact. | L89-L105 |
| DF001-F12-E028 | E | The production route is live_cli’s post-reflect memory tick; dashboard route is MemoryFactsDrawer to Tauri to filtered SSB viewer. | L107-L124 |
| DF001-F12-E029 | E | Stub bans prohibit cloud routing, below-threshold close, default-off enable flags, destructive delete, and production unwrap/expect. | L126-L133 |
| DF001-F12-E030 | E | The stated observable effect itself requires the staged model and expects a semantic event, event-time closure, a pair fingerprint, and drawer rendering. | L140-L143 |
| DF001-F12-E031 | E | Deterministic checks require the live ONNX implementation and production tick wiring even though the model data artifact is deferred. | L145-L184 |
| DF001-F12-E032 | E | Acceptance separates fake-judge and live-model boxes and also requires UI E2E, no new action/schema, event-time correctness, determinism-honest copy, no cloud branch, safe fallback, scope discipline, cargo/auditor green, and no new crate. | L186-L202 |
| DF001-F12-E033 | E | Tests cover real-path closure, threshold/error fallback, static cloud-branch absence, existing-event round-trip/no-new-action, and dashboard parity. | L219-L225 |
| DF001-F12-E034 | E | The declared artifact set adds semantic_contradiction.rs, nli_onnx.rs, four tests, a decision record, live tick/dashboard edits, and an ops-staged model plus tokenizer. | L227-L241 |
| DF001-F12-E035 | E | No private fact text is sent to a cloud service by design; inference is local, but the plan states no separate retention/redaction/consent policy. | L23-L27, L68-L70, L253-L260 |
| DF001-F12-E036 | E | Same-scope-only pairing is the conservative default; cross-scope contradiction detection is a documented future item to avoid quadratic model calls. | L260-L261 |
| DF001-F12-E037 | E | Future DKG alias work is gated on measured pair precision; cloud judgment remains rejected unless a determinism-fixed lane exists. | L263-L265 |

### F-13_session_archive_schema_bump.md — archive before destructive migration

| Atomic ID | Evidence | Explicit feature or behavior | Source anchor |
|---|---|---|---|
| DF001-F13-E001 | E | The file explicitly says STATUS PLANNED, promoted canonically as F-13 on 2026-06-02, EXTEND_EXISTING/high confidence, and deliberately minimal. | L1-L9 |
| DF001-F13-E002 | E | The objective is to archive stale SSB state before the single destructive schema-mismatch drop without adopting Springdrift’s whole-state Backup Actor. | L11-L26 |
| DF001-F13-E003 | E | Intended output copies stale ssb.db and precautionary session_dag.db under workflow/sessions/{id}/archive/ssb-v{N}.sqlite, behind ELAI_SSB_ARCHIVE_ON_BUMP_ENABLE. | L14-L22 |
| DF001-F13-E004 | E | No periodic commit, remote push, unified store, git dependency, network, subprocess, or RunPod is added. | L20-L26 |
| DF001-F13-E005 | E | SSB run_migration is asserted to drop ssb_files and ssb when user_version is stale; session-DAG migration is explicitly corrected as non-destructive. | L28-L36, L80-L90 |
| DF001-F13-E006 | E | Existing checkpoints write five byte files and metadata-only hashes/sizes to SSB and have a lazy migrator; SSB has no equivalent migrator. | L56-L73, L118-L123 |
| DF001-F13-E007 | E | Recovery today is replay/re-derivation under ELAI_SEED, session-DAG git pointers, and separate checkpoint files rather than atomic byte restore. | L56-L78 |
| DF001-F13-E008 | E | Four gaps are named: checkpoint hashes without bytes, destructive SSB bump, no unified whole-state committer, and working-tree content represented by external-git pointers. | L80-L98 |
| DF001-F13-E009 | E | Only gap 2 is in scope; gaps 1, 3, and 4 are deferred, and document-the-gap/build-nothing is the fallback. | L100-L106 |
| DF001-F13-E010 | E | Archive-on-bump is entirely Tier 1 local file I/O and estimated at 3–4 hours. | L24-L26, L156-L159 |
| DF001-F13-E011 | E | ARCHIVE_ON_BUMP_ENV names ELAI_SSB_ARCHIVE_ON_BUMP_ENABLE; archive_on_bump_enabled returns false unless truthy. | L160-L170 |
| DF001-F13-E012 | E | archive_stale_ssb returns Ok(None) when disabled and otherwise copies via rusqlite online Backup API or std::fs::copy, with the correct method explicitly deferred to implementation-time connection-state analysis. | L171-L175 |
| DF001-F13-E013 | E | archive_session_dag performs a same-flag precautionary copy despite non-destructive migration. | L176-L178 |
| DF001-F13-E014 | E | archive_manifest_digest writes a BLAKE3 prefix into event evidence; ArchiveError uses thiserror and production code forbids unwrap/expect and unnamed magic values. | L179-L183 |
| DF001-F13-E015 | E | run_migration calls archive_stale_ssb before DROP and emits session_archived only after new tables are created. | L185-L195 |
| DF001-F13-E016 | E | Archive failure while enabled logs and continues the destructive drop so startup is not bricked; disabled mode is intended to be behaviorally identical. | L189-L195 |
| DF001-F13-E017 | E | One SessionArchived action carries session_id, stale_version, archive_path, manifest_blake3, and store_kind ssb or session_dag under a baseline-plus-one bump. | L197-L203 |
| DF001-F13-E018 | E | session_archives returns seq/time/session/version/path/digest/store kind for read-side visibility. | L205-L207 |
| DF001-F13-E019 | E | get_session_archives plus an implement-time-resolved existing recovery drawer and TS stub form the frontend pair; the source refuses to guess a Svelte filename. | L209-L217 |
| DF001-F13-E020 | E | The decision record must document minimality, no Backup Actor, W-2 tension, fail-open archive failure, session-DAG correction, AGPL clean-room constraint, and deferred gaps. | L219-L226 |
| DF001-F13-E021 | E | The wiring contract produces archive functions/error/env const, viewer query/row, event/payload, and Tauri command. | L228-L242 |
| DF001-F13-E022 | E | Production reachability is every stale SSB open through run_migration, but the exact constructor caller chain is left for implementation-time map confirmation. | L243-L259 |
| DF001-F13-E023 | E | Stub bans preserve unconditional DROP, forbid periodic tick/git invocation, forbid unwrap/expect, and require BLAKE3-only digesting. | L261-L271 |
| DF001-F13-E024 | E | The live trigger opens a reused stale-version SSB with the flag set, expects archive then drop/recreate, a matching digest row, and dashboard display. | L279-L288 |
| DF001-F13-E025 | E | Deterministic checks verify module symbols, ordering before DROP, event/golden, viewer, Tauri/TS pair, drift, no tick/git, BLAKE3, and decision record. | L290-L325 |
| DF001-F13-E026 | E | Acceptance requires an explicit W-2 exception or default-on flip, unchanged DROP, byte-faithful valid SQLite archive, matching digest, inert disabled path, no Backup Actor creep, UI E2E, golden/schema parity, AGPL clean-room, cargo/auditor green, and no dependency. | L327-L362 |
| DF001-F13-E027 | E | Tests cover enabled/disabled migration, valid archive/digest, preserved destructive recreation, event round-trip, and a real-run_migration observable smoke. | L364-L378 |
| DF001-F13-E028 | E | The declared change set adds one archive module, decision record, four tests, and edits SSB, viewer, lib, golden lock, Tauri, an unresolved drawer, and stub. | L380-L397 |
| DF001-F13-E029 | E | Springdrift evidence is treated as vendor-reported, single-author, n=1, unreproduced, 45-star evidence too weak for a heavy build. | L399-L409, L452-L466 |
| DF001-F13-E030 | E | Springdrift is AGPL-3.0; only the archive-before-destroy concept may be reused and no source code may be copied. | L401-L409, L475-L478 |
| DF001-F13-E031 | E | The W-2 resolution options are keep as staging/not numbered, flip default on plus smoke, or approve an explicit internal-forensics exception; silent default-off shipment is forbidden. | L467-L474 |
| DF001-F13-E032 | E | No persisted checkpoint msgpack is asserted present, so checkpoint-loss risk is code-derived rather than demonstrated by a captured live failure. | L479-L484 |
| DF001-F13-E033 | E | The copy is additive and may never suppress the expected clean-db DROP. | L485-L487 |
| DF001-F13-E034 | E | Archive failure is best-effort and not guaranteed; the dashboard must not imply guaranteed recovery. | L493-L496 |
| DF001-F13-E035 | E | Archive retention/garbage collection is absent and deferred; cold files accumulate per session. | L497-L499 |
| DF001-F13-E036 | E | The fallback documentation would state that SSB journals are forensic only within a session and working-file durability remains external git plus seeded replay. | L501-L510 |

## Ambiguity ledger

These are not implementation facts. They are contracts that the originals leave open.

| Atomic ID | Evidence | Ambiguity requiring resolution | Source anchor |
|---|---|---|---|
| DF001-F0-A001 | A | The index is a plan/status doctrine, not a code audit; apart from later “as-built” comments in individual files, implementation state cannot be inferred from the index catalog. | F-0 L32-L36, L187-L200 |
| DF001-F0-A002 | A | W-4’s “pure internal enforcement” exception has no approval authority, duration, or evidence format. | F-0 L70-L75 |
| DF001-F0-A003 | A | Baseline-plus-N schema coordination is required but no concurrency/merge protocol is defined for simultaneous F edits. | F-0 L100-L104, L136-L149 |
| DF001-F4-A001 | A | Exact hash input remains unspecified: raw stdout versus captured summary, stderr inclusion, exit status, command/env, encoding, truncation, and canonicalization. | F-4 L16, L51, L205-L210 |
| DF001-F4-A002 | A | Same-superstep row lookup and integration-name matching lack a durable correlation key beyond test_name suffix. | F-4 L53, L110-L113, L205-L207 |
| DF001-F4-A003 | A | Behavior for existing passed=true rows deserialized with None during migration/backfill is not defined. | F-4 L49-L55, L154 |
| DF001-F4-A004 | A | Concurrent checkpoint outcomes and the SSB snapshot boundary used by the gate are not specified. | F-4 L53, L97-L99 |
| DF001-F4-A005 | A | Hashes of potentially sensitive stdout are journaled, but retention, redaction, access control, and side-channel policy are unstated. | F-4 L49-L57, L110-L113 |
| DF001-F8-A001 | A | The adapters from arbitrary ConfigVariant/StructuralChange values into typed numeric predicates are not fully specified. | F-8 L51-L59 |
| DF001-F8-A002 | A | ResourceContention normally compares competing intents, but the model carries one candidate plus locks/bounds/tripwires and names no resource-ownership snapshot. | F-8 L51-L57 |
| DF001-F8-A003 | A | Operator-confirm row type, authorization, expiry, replay behavior, and who may approve are not defined. | F-8 L61-L63, L112-L118 |
| DF001-F8-A004 | A | The pending proposal store/status transition and idempotency of repeated conflict rows are not specified. | F-8 L61-L69, L125-L128 |
| DF001-F8-A005 | A | Snapshot freshness is required, but locking/atomicity between tripwire read and admission is not. | F-8 L71, L218 |
| DF001-F9-A001 | A | The title promises provenance, but no provenance source, predicate, payload field, or acceptance assertion is defined. | F-9 L1-L3, L54-L74 |
| DF001-F9-A002 | A | Cache capacity, eviction tie-breaking, persistence, poisoning defense, per-project isolation, and prewarming are deferred to a decision doc. | F-9 L64-L66, L74 |
| DF001-F9-A003 | A | Negative-cache insertion is not defined even though the tier registry says known-bad cache hits block. | F-9 L64-L65, L76-L78 |
| DF001-F9-A004 | A | Offline mode appears in as-built CLI reachability but its semantics versus cache miss and advisory behavior are not specified. | F-9 L105-L108 |
| DF001-F9-A005 | A | Registry rate limits, retries, proxying, namespace/alias normalization, git/path dependencies, and cache-key ecosystem isolation are unstated. | F-9 L54-L68, L226-L234 |
| DF001-F9-A006 | A | Registry queries cross an external data boundary, but request logging, dependency-name privacy, proxy policy, and advisory retention are unstated. | F-9 L21-L29, L62-L72 |
| DF001-F10-A001 | A | First-iteration baseline, removal then reintroduction, severity changes within one CWE, generated-delta boundaries, and renamed loop IDs are undefined. | F-10 L1-L3, L51-L57 |
| DF001-F10-A002 | A | State storage keyed by loop ID has no lifecycle, eviction, crash recovery, or concurrent-update contract. | F-10 L53-L56 |
| DF001-F10-A003 | A | The exact calibration corpus, labeling authority, class balance, confidence thresholds, and reproducible report schema are not defined. | F-10 L46-L49, L142-L147 |
| DF001-F10-A004 | A | UNKNOWN escalation has no timeout or explicit auditor-to-FSM completion contract beyond the intended pre-Memorizing block. | F-10 L92-L99, L209-L214 |
| DF001-F10-A005 | A | Whether the standalone gate or TripwireKind design is normative remains a user-facing architecture decision. | F-10 L209-L211 |
| DF001-F10-A006 | A | The self-mod allowlist expiry behavior after 2026-09-01 and promotion criteria through F-11 are unstated. | F-10 L90 |
| DF001-F11-A001 | A | The real benchmark suite name, input receipt, timeout, determinism envelope, and conversion from BenchmarkResult to alpha are unspecified. | F-11 L12, L64-L67 |
| DF001-F11-A002 | A | Archive insert plus later parent selection lacks a transaction, rollback, partial-failure, retry, or duplicate-variant contract. | F-11 L64-L70, L195-L200 |
| DF001-F11-A003 | A | The mutation operator after parent selection and what exactly “next generation’s parent” means in a one-generation run are unspecified. | F-11 L3, L64-L69 |
| DF001-F11-A004 | A | Two-Gate descriptor construction, capacity K_m, and security-sensitive conversion of structured config to text remain underspecified. | F-11 L67, L80, L230-L234 |
| DF001-F11-A005 | A | Environment/feature enablement grants autonomous mutation authority, but operator identity, audit approval, isolation, and rollback permissions are unstated. | F-11 L72, L177-L180, L236 |
| DF001-F12-A001 | A | “Approximately two rows” has no explicit bound or candidate ranking rule inside a same-project/scope group. | F-12 L60-L64 |
| DF001-F12-A002 | A | NLI label order is model-specific and promised in a decision doc, but no pinned variant, model hash, tokenizer hash, license receipt, or provenance record is fixed. | F-12 L70, L81-L87, L241, L250 |
| DF001-F12-A003 | A | Tick budget, batching, deduplication, pair retry, per-user scheduling, and behavior on a large current-fact set are not defined. | F-12 L63, L72-L75 |
| DF001-F12-A004 | A | Concurrent fact insertion/closure can race the candidate snapshot; transaction isolation and idempotent repeated soft-close semantics are unstated. | F-12 L63-L68, L72-L75 |
| DF001-F12-A005 | A | Local processing avoids cloud disclosure, but permissions for raw fact text, model logs, SSB fact identifiers, retention, and user consent are unstated. | F-12 L23-L27, L68-L77 |
| DF001-F12-A006 | A | The emitted existing event contains only fact IDs and conflict kind, so the source does not define where NLI label/confidence/action/pair fingerprint are durably stored. | F-12 L10, L35, L67, L72-L77 |
| DF001-F13-A001 | A | Online Backup API versus std::fs::copy is unresolved; WAL consistency, open-handle locks, atomic rename, fsync, disk-full, and partial-copy cleanup are not specified. | F-13 L171-L181, L341-L343 |
| DF001-F13-A002 | A | session_id and stale_version source, sanitization, path traversal defense, archive-directory permissions, and symlink policy are not defined. | F-13 L171-L180, L279-L288 |
| DF001-F13-A003 | A | Reopening the same stale version can collide with ssb-v{N}.sqlite; overwrite, idempotency, suffixing, and retry behavior are unspecified. | F-13 L171-L180, L279-L288 |
| DF001-F13-A004 | A | No restoration command, validation-before-restore, replay workflow, or recovery SLA consumes an archive. | F-13 L100-L106, L205-L217, L501-L510 |
| DF001-F13-A005 | A | Archives may contain sensitive journal/session data, but encryption, access mode, redaction, retention, deletion, consent, and compliance policy are absent. | F-13 L17-L22, L285-L288, L497-L499 |
| DF001-F13-A006 | A | The exact existing Svelte host and migration constructor caller are intentionally unresolved, so W-3/W-4 cannot yet be mechanically complete. | F-13 L209-L217, L252-L259 |

## Conflict and stale-reconciliation ledger

| Atomic ID | Evidence | Conflict or correction internal to an original | Source anchor |
|---|---|---|---|
| DF001-F0-C001 | C | The title and core catalog say ten sessions, while the same index canonically appends four planned extensions; this is ten original sessions plus extensions, not a literal ten-item folder. | F-0 L1, L117-L134, L194-L224 |
| DF001-F0-C002 | C | Global W-2 forbids terminal default-off gates, while F-11 and F-13 explicitly require or contemplate permanent opt-in exceptions. | F-0 L57-L62, L205-L221 |
| DF001-F4-C001 | C | Early prose describes blocking any passed=true plus None row, but Task 3 scopes blocking to rows with the _integration suffix that “should have a slot.” | F-4 L3, L53 |
| DF001-F4-C002 | C | Tasks and deterministic checks refer to run.output/TestRunResult, while the reconciled contract says the real type is IntegrationRunResult.output_capture and explicitly rejects the reasoning type. | F-4 L16, L51, L79-L88, L132-L134, L163-L164 |
| DF001-F4-C003 | C | The action design collapses two outcomes into one TestIntegrityEvent variant, but checks and observable expectations continue to talk as if test_integrity_verified and anti_fabrication_block are independent action types/action_type strings. | F-4 L54-L55, L68-L78, L110-L113, L126-L128, L202 |
| DF001-F4-C004 | C | The C-22-inspired non-empty-output hash yields None for captured empty stdout, while the risk section requires hashing empty bytes so None exclusively means no capture. | F-4 L51, L208 |
| DF001-F4-C005 | C | The reconciled production gate is drive_task, but the doctrine explanation still names emit_checkpoint_outcome and get_per_level_verdicts rather than the reconciled gate/command path. | F-4 L93-L99, L194-L201 |
| DF001-F8-C001 | C | Tasks, tests, and acceptance require wiring inside DefaultLayer1aProcessor::apply_proposal, while the later as-built reachability claims CLI run_apply to screen_config_variant_for_conflicts instead. | F-8 L61-L69, L107-L110, L166-L184 |
| DF001-F8-C002 | C | The file contains later as-built anchors and shipped-real symbols while all acceptance boxes remain unchecked and no explicit DONE status is declared. | F-8 L75-L110, L164-L177 |
| DF001-F9-C001 | C | D-11 is labeled PLANNED/not shipped and a hard blocker, yet the same source asserts its checker file/functions and RegistryCheck enum exist and are authoring-time confirmed. | F-9 L7-L8, L21-L23, L31-L39 |
| DF001-F9-C002 | C | The source calls this the first fail-closed enforcement of the human rule, but cache-miss network failures explicitly allow the write. | F-9 L27-L29, L39, L65, L76-L78 |
| DF001-F9-C003 | C | Tier text promises known-bad cache-hit blocking, but the task logic only explicitly inserts successful lookups and never defines negative-cache insertion. | F-9 L64-L65, L76-L78 |
| DF001-F9-C004 | C | RegistryUnreachable uses an action named write_time_slopsquat_blocked even though the write is allowed, creating observability semantics that conflate a block with degradation. | F-9 L65, L70-L72, L109-L122 |
| DF001-F9-C005 | C | The checker configuration is framed as maximum package age/staleness, but the event discriminator is TooNew; the exact policy direction for old versus freshly published packages is not consistently specified. | F-9 L19-L29, L56-L70 |
| DF001-F10-C001 | C | Initial ground truth and Phase-1 heading say the classifier does not exist; the later reconciled contract says it ships as-built in orchestrator. | F-10 L15, L25, L44-L49, L64-L70 |
| DF001-F10-C002 | C | As-built classifier paths/types/pattern storage contradict the task, deterministic-check, consumes-symbol, test, and file-plan references to crates/safety and CweClassSet/TOML. | F-10 L46-L49, L64-L85, L114-L118, L173-L199 |
| DF001-F10-C003 | C | As-built reachability moves REFINE to run_refine_leg, auto-advance to a row-query function, and CLI to safety_cmd, while deterministic checks still demand direct chat_reflect/task_runner/promotion_gate calls. | F-10 L86-L91, L126-L135 |
| DF001-F10-C004 | C | Acceptance says all three loop legs block, but the as-built self-mod leg is intentionally wire-dark behind darwinian-evolve. | F-10 L90, L142-L152 |
| DF001-F10-C005 | C | The feature is described as post-write async and non-stalling, yet also as blocking before completion; the exact commit/rollback boundary is not consistent across REFINE prose. | F-10 L3, L15, L53-L56, L148, L209-L214 |
| DF001-F11-C001 | C | F-11 explicitly violates the inherited W-2 terminal-default-on rule while still claiming doctrine completion through a justified opt-in posture. | F-11 L72, L177-L180, L236, L243-L244 |
| DF001-F11-C002 | C | One observable run expects a rejected guard candidate and a LoopClosed insertion/select receipt; the source implies multiple mutants but never defines correlation/order proving a different clean candidate closed the loop. | F-11 L64-L70, L129-L132, L177-L200 |
| DF001-F11-C003 | C | Tier 1 is called entirely in-process/no subprocess, but the “real external benchmark” execution mechanism is not established in this source. | F-11 L5, L12, L62-L70, L252 |
| DF001-F11-C004 | C | The observable command hard-codes a Windows conductor worktree path, while the artifact is otherwise presented as a reusable production done trigger. | F-11 L129-L132 |
| DF001-F12-C001 | C | Objective/task language says NLI only handles audit_conflicts Surface residue and near-key neighbors, but collect_cross_key_candidates independently pairs cross-key current rows and the tick emits events for every SurfaceToUser outcome. | F-12 L3, L63-L75 |
| DF001-F12-C002 | C | Tier-1 is said to be the done gate without a staged model, but W-1 observable_effect and its run-trigger require the live model-staged path. | F-12 L5, L58-L59, L81-L87, L104-L105, L140-L149, L208-L217 |
| DF001-F12-C003 | C | MemoryConflictDetectedPayload has only fact_id_a, fact_id_b, conflict_kind, while the dashboard and observable evidence require label, confidence, action, and fingerprint with no named persistence source. | F-12 L10, L35, L67, L72-L77, L140-L143 |
| DF001-F12-C004 | C | OnnxNliJudge construction is required to fail loudly when the model is absent, while production must catch absence and quietly skip after one log; the ownership of that distinction is not fully specified. | F-12 L70-L72, L81-L83 |
| DF001-F13-C001 | C | The file says it was promoted to canonical numbered F-13, but repeatedly says a full F-N is unwarranted, default disposition is not promotion, and W-2 may not bind if it remains staging. | F-13 L1-L9, L219-L226, L467-L474, L501-L510 |
| DF001-F13-C002 | C | Objective and produced-symbol lists promise both SSB and session-DAG copies, but run_migration wiring and reachable_from call only archive_stale_ssb; archive_session_dag has no in-plan caller. | F-13 L17-L22, L176-L178, L185-L195, L230-L259 |
| DF001-F13-C003 | C | “One file-copy + one-line guard” conflicts with two database-copy functions, one new action/payload, viewer, frontend, and multiple tests. | F-13 L17-L25, L156-L226, L364-L397 |
| DF001-F13-C004 | C | The objective says the guard converts destroyed-forever into archived evidence, but archive failure intentionally continues destruction, so preservation is best-effort rather than guaranteed. | F-13 L11-L22, L185-L195, L493-L496 |
| DF001-F13-C005 | C | A permanent default-off ENABLE flag directly conflicts with W-2 and remains unresolved among three disposition options. | F-13 L160-L170, L327-L337, L467-L474 |

## Cross-source interactions and bounded inferences

| Atomic ID | Evidence | Interaction or inference | Anchors |
|---|---|---|---|
| DF001-X-I001 | I | F-4’s test-integrity evidence and F-10’s async security evidence both need a reliable pre-Memorizing completion barrier; otherwise either can arrive after the truth-bound state transition. | F-4 L53, L97-L99; F-10 L53-L56, L209-L214 |
| DF001-X-I002 | I | F-8 symbolic review, F-10 CWE monotonicity, and F-11’s four-guard chain are distinct self-mod axes, but the sources do not define one canonical order or transaction across all three. | F-8 L61-L71, L238-L240; F-10 L56, L90; F-11 L64-L80, L254-L256 |
| DF001-X-I003 | I | F-9 is the write-time registry leg, E-41 the PR prefilter, and Socket the install-time operational leg; F-1 may later fan the same dependency evaluator into VibeGate. | F-9 L29-L39, L232-L238 |
| DF001-X-I004 | I | F-12 explicitly orders exact-key audit, age quarantine, and semantic adjudication on one memory tick, but concurrent facts can change between those passes without a stated snapshot transaction. | F-12 L72-L75, L107-L124 |
| DF001-X-I005 | I | Every new SSB action increases the chance that F-13’s migration trigger runs on an old database; therefore archive behavior is operationally coupled to F-4/F-8/F-9/F-10/F-11 schema bumps even though no plan names that interaction. | F-0 L136-L149, L223-L224; F-13 L185-L203 |
| DF001-X-I006 | I | F-4’s additive None default means legacy PASS rows could become false fabrication evidence if queried without a “written under new schema” discriminator. | F-4 L49-L55, L154 |
| DF001-X-I007 | I | F-9’s advisory-on-unreachable keeps availability but cannot satisfy “zero hallucinated dependencies reach disk” during outages; the live 50/50 claim is necessarily conditioned on registry availability or a fixture. | F-9 L65, L161-L170, L226-L231 |
| DF001-X-I008 | I | F-10’s reconciled as-built section supports “partially implemented/wired” more strongly than the orientation summary’s “classifier does not exist,” but unchecked acceptance and a dark self-mod leg prevent a source-only DONE conclusion. | F-10 L64-L106, L142-L154 |
| DF001-X-I009 | I | F-11’s dual gate is a permission boundary for autonomous mutation, not merely configuration; its dry-run fallback is the only stated way to reconcile discoverability with least authority. | F-11 L72, L177-L180, L243-L244 |
| DF001-X-I010 | I | F-12’s local model minimizes fact-text disclosure, but the dashboard/SSB identifier trail still creates a privacy and retention surface not covered by “no cloud judge.” | F-12 L68-L77, L253-L260 |
| DF001-X-I011 | I | F-13 archives the very audit database that can contain security/test/memory events, so filesystem permissions and deletion policy are part of the effective security contract even though absent from the plan. | F-13 L17-L22, L285-L288, L497-L499 |
| DF001-X-I012 | I | F-0’s “one change at a time” and per-file non-parallel-safe notes are the only concurrency control for schema/golden edits; no merge-safe allocation mechanism exists. | F-0 L136-L149, L181-L182; F-4 L6; F-9 L13 |

## Summary additions and corrections

Each row compares an original with 06-F-SUMMARY_INDEX.discovery.md. “Addition” means source detail absent from the orientation summary; “correction” means the original materially changes or qualifies the summary claim.

| Atomic ID | Evidence | Addition or correction | Original anchor |
|---|---|---|---|
| DF001-D-D001 | D | Addition: F-0 defines exact W-1 mechanics including DEF/REF resolution, depth-8 BFS, live artifact, FSM completion veto, dashboard check, and memory-writer; the summary’s global contract is directionally right but less operationally complete. | F-0 L47-L115 |
| DF001-D-D002 | D | Addition: F-0’s schema registry includes success-side and reserved events such as test_integrity_verified and Tier-2 VibeGate kinds, not only the headline failure events. | F-0 L136-L149 |
| DF001-D-D003 | D | Correction: F-4’s C-27/C-28 dependencies are explicitly not hard blockers for the field/gate, and the process-slot design question is declared pre-resolved. | F-4 L4-L6, L16-L20, L24-L26 |
| DF001-D-D004 | D | Correction: F-4’s current contract is one TestIntegrityEvent action with a kind discriminator, not two independent SSB variants. | F-4 L54-L55, L68-L78 |
| DF001-D-D005 | D | Addition: F-4 later reconciles the actual source type to IntegrationRunResult.output_capture and the production enforcement point to drive_task rather than the originally planned TestRunResult/FSM route. | F-4 L79-L99 |
| DF001-D-D006 | D | Addition: F-4 explicitly identifies empty captured stdout as a false-positive hazard and requires hashing empty bytes. | F-4 L205-L210 |
| DF001-D-D007 | D | Addition: F-8 specifies the full symbolic schema—three conflict kinds, six predicate forms, ProcessModel fields, two adapters, all-conflicts output, and tripwire-quiescence rule. | F-8 L49-L73 |
| DF001-D-D008 | D | Correction: F-8’s later as-built reachability is a CLI screening function, not the apply_proposal insertion point repeated in tasks/acceptance. | F-8 L107-L110, L164-L184 |
| DF001-D-D009 | D | Addition: F-8’s permission behavior is interactive operator-confirm versus autonomous pending/queue, not merely “warn then escalate.” | F-8 L61-L63 |
| DF001-D-D010 | D | Correction: F-9 explicitly says D-11 is PLANNED/not shipped even while its component checker exists; F-9 is blocked until the prerequisite is considered shipped. | F-9 L7-L8, L31-L39 |
| DF001-D-D011 | D | Addition: F-9 fixes concrete defaults—730 days for PyPI/npm, 1825 for cargo—and defines RegistryUnreachable as Allow plus advisory. | F-9 L56-L65 |
| DF001-D-D012 | D | Correction: F-9 is not generally fail-closed at write time; only cache-resolved decisions are T1, while registry-unreachable misses fail open. | F-9 L27-L29, L65, L76-L78 |
| DF001-D-D013 | D | Addition: F-9 has a reconciled production bootstrap, dedicated elai dep-check [--offline] CLI, checker trait implementations, and dashboard module names missing from the summary. | F-9 L80-L108 |
| DF001-D-D014 | D | Correction: F-9’s title-level provenance promise remains unimplemented in the body. | F-9 L1-L3, L54-L74 |
| DF001-D-D015 | D | Major correction: F-10’s later as-built ledger says classify_delta now ships in orchestrator with LazyLock regexes and BTreeSet, contradicting the orientation summary’s “classifier does not exist.” | F-10 L64-L76 |
| DF001-D-D016 | D | Addition: F-10 has as-built REFINE, auto-advance, CLI, dashboard, and allowlisted dark self-mod routes; the summary does not capture this later reconciliation. | F-10 L86-L106 |
| DF001-D-D017 | D | Correction: F-10’s remaining task/check/file sections are stale against the as-built classifier location and call graph, so no single uncontested implementation ledger exists inside the source. | F-10 L108-L140, L171-L200 |
| DF001-D-D018 | D | Addition: F-10 defines exact initial CWE classes, three-language calibration, per-language 5% threshold, 50 clean patches each, and three-pass anti-spiral escalation. | F-10 L44-L49 |
| DF001-D-D019 | D | Addition: F-11’s exact guard order includes the otherwise-bypassed is_security_sensitive check and requires trap scoring on live candidate bytes rather than fixtures. | F-11 L64-L80 |
| DF001-D-D020 | D | Addition: F-11’s one action payload, three dispositions, BLAKE3/seed evidence, exact feature/env permission boundary, and dry-run fallback are richer than the summary. | F-11 L72-L80, L129-L132, L243-L244 |
| DF001-D-D021 | D | Addition: F-11’s done trigger hard-codes a transient Windows conductor-worktree fixture path, which weakens portability of the stated live acceptance. | F-11 L129-L132 |
| DF001-D-D022 | D | Addition: F-12 fixes the threshold at 0.85, defines label/error fallback, model VRAM variants, same-project/scope candidate boundary, and event-time closure. | F-12 L60-L70 |
| DF001-D-D023 | D | Correction: F-12’s existing MemoryConflictDetected payload cannot directly carry the dashboard’s promised label/confidence/action/fingerprint fields; an additional data source is missing. | F-12 L10, L35, L67, L72-L77 |
| DF001-D-D024 | D | Correction: F-12 calls Tier 1 independently shippable, yet the mandatory W-1 observable trigger requires the staged Tier-2 model. | F-12 L58-L59, L81-L87, L140-L149 |
| DF001-D-D025 | D | Addition: F-12 emits semantic_contradiction rows for SurfaceToUser outcomes as well as auto-invalidations, so an event is not itself proof of closure. | F-12 L72-L77 |
| DF001-D-D026 | D | Correction: F-13 explicitly labels itself both canonical planned F-13 and a draft that should not be a full F-N; the summary’s simple planned extension status hides this normative conflict. | F-13 L1-L9, L467-L474, L501-L510 |
| DF001-D-D027 | D | Addition: F-13 archive failure logs then still destroys the stale tables, so recovery preservation is best-effort, not guaranteed. | F-13 L185-L195, L493-L496 |
| DF001-D-D028 | D | Addition: F-13 promises a session-DAG archive function and event store_kind, but no task/reachability call wires that function. | F-13 L176-L178, L197-L203, L230-L259 |
| DF001-D-D029 | D | Addition: F-13 explicitly imposes AGPL clean-room constraints and records weak, unreproduced prior-art evidence as a reason not to build a heavy mechanism. | F-13 L399-L409, L452-L478 |
| DF001-D-D030 | D | Addition: F-13 leaves WAL-consistent copy method, dashboard host, caller chain, retention, and restore behavior unresolved. | F-13 L171-L181, L209-L217, L252-L259, L497-L510 |

## Category coverage map

| Required dimension | Where exhaustively represented |
|---|---|
| Capabilities, actions, and surfaces | Per-file E ledgers; especially F-4 E011-E023, F-8 E008-E023, F-9 E007-E021, F-10 E006-E026, F-11 E010-E022, F-12 E009-E030, F-13 E011-E024 |
| Workflows and states | F-0 E006-E021/E038-E044; F-4 E013/E021-E025; F-8 E013-E014/E022-E025; F-9 E010-E014/E018-E024; F-10 E011-E026; F-11 E010-E024; F-12 E012-E026; F-13 E015-E026 |
| Schemas, data, and lifecycle | F-0 E019/E033-E037/E050; each feature’s payload/viewer/cache/store/archive atoms |
| Flags, defaults, and configuration | F-0 E010-E011/E041-E043; F-8 E014; F-9 E008/E011/E014; F-10 E009/E019/E024; F-11 E003/E018-E020/E025; F-12 E015/E018-E021; F-13 E011/E026/E031 |
| Integrations and events | F-0 schema registry plus each per-file produced/consumed/reachable/observable atom |
| Permissions, security, privacy, compliance | F-0 E042; F-8 E013-E014; F-9 E014 and A006; F-10 E019; F-11 E018-E020 and A005; F-12 E007/E035 and A005; F-13 E029-E034 and A005 |
| Failure, fallback, retry, recovery | F-4 E014/E022/E030; F-8 E013-E014/E028-E029; F-9 E013/E028-E029; F-10 E010/E012/E033; F-11 E013/E020/E029; F-12 E014/E019/E024-E026; F-13 E009/E016/E034-E036 |
| Determinism, idempotency, concurrency | F-0 E019/E040-E043; feature hash/seed/cache/async atoms and every A row about correlation, state, transaction, collision, or race |
| Observability and evidence | All observable-effect, viewer, frontend, SSB, artifact, and dashboard atoms |
| Tests, acceptance, and done gates | F-0 E006-E021; F-4 E024-E027; F-8 E024-E027; F-9 E022-E026; F-10 E027-E030; F-11 E023-E027; F-12 E031-E034; F-13 E025-E028 |
| Constraints and non-goals | F-0 W rules; F-4 E008/E019; F-8 E004/E030; F-9 E005/E029; F-10 E032; F-11 E008/E028-E029; F-12 E008/E035-E037; F-13 E004/E009/E029-E036 |
| Dependencies, status, and deferred work | F-0 E001-E005/E022-E032/E038-E051; each file’s prerequisite, parallel-safety, as-built/planned, acceptance, risk, and Next atoms |

## Zero-omission and duplicate audit

All assigned originals were read sequentially from line 1 through their explicit end markers. Blank lines, Markdown headings, separators, mandatory-read path inventories, citation bibliographies, and repeated doctrine restatements were read but are not independently counted as features. A repeated clause is represented once with all corroborating anchors. Mandatory reads/citations are reflected through the exact dependency, symbol, integration, grounding, and constraint atoms they support.

| Original | Lines read | Explicit E | Ambiguous A | Conflicting C | Source-ledger atoms |
|---|---:|---:|---:|---:|---:|
| F-0_INDEX.md | 233/233 | 51 | 3 | 2 | 56 |
| F-4_test_integrity_cant_lie.md | 217/217 | 32 | 5 | 5 | 42 |
| F-8_scf_symbolic_conflict_selfmod.md | 242/242 | 31 | 5 | 2 | 38 |
| F-9_hallucinated_dep_writetime_gate.md | 238/238 | 30 | 6 | 5 | 41 |
| F-10_security_monotonicity_iter_guard.md | 220/220 | 34 | 6 | 5 | 45 |
| F-11_close_one_config_generation.md | 258/258 | 30 | 5 | 4 | 39 |
| F-12_semantic_contradiction_local_nli.md | 267/267 | 37 | 6 | 4 | 47 |
| F-13_session_archive_schema_bump.md | 513/513 | 36 | 6 | 5 | 47 |
| **Assigned-source subtotal** | **2188/2188** | **281** | **42** | **32** | **355** |
| Cross-source interactions | — | — | — | — | 12 inferred I |
| Summary additions/corrections | — | — | — | — | 30 delta D |
| **Grand atomic inventory** | **8 files / 2188 lines** | **281 E** | **42 A** | **32 C** | **397 total** |

Mechanical ID audit:

- ID grammar: DF001-<namespace>-<E|A|C|I|D><three digits>.
- Atomic ID occurrences: 397.
- Unique atomic IDs: 397.
- Duplicate IDs: 0.
- Missing sequence numbers within every namespace/evidence series: 0.
- Original files outside DEEP-F-001 opened: 0.
- Assigned original files omitted: 0.
- Assigned source lines unread: 0.
- Manifest hash/count mismatches: 0.
- Summary claims promoted to implementation fact without original evidence: 0.

## Output checksum

Because a file cannot contain its own ordinary SHA-256 without changing that digest, this artifact uses a reproducible normalized self-checksum. To verify it, replace the 64 hexadecimal characters after normalized_output_sha256 below with 64 ASCII zeroes, preserve all other UTF-8 bytes and the final newline, then run SHA-256.

normalized_output_sha256: 18e6dd1a077acfd667c7b12516d79c85f83bfff122c7eb01c467d4eedd7ee87c

End of DEEP-F-001 discovery.
