# H-plan — Summary Index of All 54 sessions (Tracks 1-4 + Track 6 GROUNDED-REASONING + Track 7 DGM-VERIFICATION + Track 8 REVERSIBILITY + Track 9 CONTEXT-LIFECYCLE + Track 10 REVIEW-ANCHORING + Track 11 EXTERNAL-STEALS; Track 5 = odysseus, reserved/unauthored)
Generated: 2026-06-03 (Track 1/2); 2026-06-04 (Track 3 — TRUST-HARDENING; Track 4 — UX/MEMORY; Track 6 — GROUNDED-REASONING); 2026-06-05 (Track 7 — DGM-VERIFICATION; Track 8 — REVERSIBILITY; Track 9 — CONTEXT-LIFECYCLE; Track 10 — REVIEW-ANCHORING). Mirrors G-SUMMARY_INDEX format.
Each entry preserves the track + lever, the grounding strength, the scope, the wire-dark/shipped ELAI
substrate it activates (or the absent code it builds), the prior-plan artifact it COMPOSES-WITH (GD-1), the
production integration point, and the observable_effect that is the wiring done-gate (W-1). Canonical why +
doctrine: H-0_INDEX.md (inherits F-0 W-1..W-10 + G-0 GD-1/2/4/5 + adds HD-1..HD-5; Track 3 ADDS TD-1..TD-5).
Grounded symbol maps (W-7): _grounding/H-GROUNDING-MAP.md (Track 1/2); _grounding/H-TRUST-GROUNDING-MAP.md (Track 3).

## Premise
Track 1 (H-1..H-9) makes ELAI provably CHEAPER on the SAME cloud model than Claude Code / Cursor / Codex,
mostly by ACTIVATING wire-dark caching/output/routing substrate; Track 2 (H-10..H-16) replaces the watch-
not-work 3D orbit with a 2D generative canvas the model drives and the user points at, whose MOAT is the
journaled + reproducible + taint-tagged op-stream underneath (G-1/G-2); Track 3 (H-17..H-29) makes an
AUTONOMOUS trading-strategy search/build loop TRUSTABLE by deterministic ELAI gates reading off-agent
artifacts — closing TB-1 selection-bias-under-search (the SEARCH ITSELF ships overfit edge even with honest
agents), TB-2 leakage, and TB-3 self-evaluator capture, with the irreducible undecidable residue named and
routed to human Layer-3 (TD-5). The differentiator vs F/G is GD-1 (COMPOSE-NOT-RESTATE) + HD-1 (every
efficiency number MEASURED per config, never multiplied) + HD-4 (every canvas gate auditor-driven, off-agent)
+ TD-1..TD-5 (Track 3: persisted-N keystone / frozen evaluator / LLM-is-T3-advisory / verify-the-verifiers /
honest-residue). A session is not DONE until wiring_contract_check.py --run-trigger PASSes; default-off as a
terminal state is banned (W-2). Track 3 is BUILD-ON-SHIPPED-substrate (the fail-closed off-agent-artifact
substrate is already live; the finance/search-specific gates are the genuine BUILD — see H-TRUST-GROUNDING-MAP).

================================================================================
TRACK 1 — EFFICIENCY
================================================================================

### H-1 — Prompt caching on the production path (close the RuntimeAdapter split-brain + activate the hit-rate monitor)
- Track 1 | lever CHEAPER | grounding STRONG | nature ACTIVATION | effort M | WAVE 1
- Steal/build: the main coding-agent loop ALREADY caches (handle_infer_request -> AnthropicClient::send_message
  -> apply_anthropic_cache_markers, ephemeral on last system block + last tool def). Two gaps: (a) the parallel
  AnthropicApiRuntime::complete RuntimeAdapter path (anthropic_api.rs:101) hardcodes cache_control:None and ships
  ZERO markers for its secondary callers (hook_prompt, focus_tunnel orchestrator, rlaif_judge) despite
  supports_caching:true; (b) cache_check.rs (hit-rate monitor) is wire-dark (no prod caller, default-OFF). H-1
  unifies marker logic onto the RuntimeAdapter path and lights up the live hit-rate measurement.
- COMPOSES-WITH: B-27/C-5 cloud mode, E-39 cost gate; consumes apply_anthropic_cache_markers (shipped — do NOT
  re-implement, GD-1).
- Integration: anthropic_api.rs:137/189 (the None + the false-advertised capability); cache_check.rs:32; the
  RuntimeAdapter callers hook_prompt.rs:242 / focus_tunnel/orchestrator.rs:218/280. frontend_pair: hit-rate tile (H-9).
- observable_effect: a real two-turn task shows the second turn's request carries cache_control markers on the
  RuntimeAdapter path AND the live hit-rate monitor reports a non-zero cache-read fraction recomputable from the
  journal (GD-2); a single-turn fixture shows the first write at 1.25x. Default-ON, no env override.

### H-2 — Cache-stable layout + breakpoints + 1h TTL (place real markers on the cloud path; assert byte-stability)
- Track 1 | lever CHEAPER | grounding STRONG | nature BUILD | effort M | WAVE 1
- Steal/build: ELAI's allocator specifies 2 breakpoints (after PhaseSpec, after CodeContext) and computes a
  blake3 byte-prefix stability flag (bp.valid) — but the whole mechanism is INERT (bp.valid consumed only by an
  eprintln!), fed EMPTY slices for slots 2-7, and reachable only under BudgetPolicy::Local (Cloud/ApiDefault
  never call it). H-2 feeds real slot bytes, consumes bp.valid to PLACE cache_control markers on the cloud path,
  adds a SECOND tail breakpoint (20-block lookback), opts into 1h TTL via "ttl":"1h" (NO beta header — CORRECTED),
  keys min-prefix on the exact model id, and adds the HD-2 debug_assert (a cache-bust is a test failure).
- COMPOSES-WITH: B-10/Phase-3E context budget; consumes update_breakpoints + bp.valid + CumulativeTokenTracker.
- Integration: context_budget.rs:1481 (update_breakpoints) / 1191-1200 (the 2 bps) / 1507 (bp.valid);
  runtime_client.rs:376-385 (the empty-slice + Local-only site). frontend_pair via H-9 cache tile.
- observable_effect: on a real multi-turn cloud task, the cached byte-prefix is byte-identical turn-to-turn (the
  debug_assert holds), markers are placed on a verified-stable prefix only, and the measured hit-rate (H-9) rises;
  a deliberate prefix perturbation FAILS the assert (the cache-bust guard fires).

### H-3 — Output-token stack: diff/FIM emission + streaming early-abort (the 5x-billed lever)
- Track 1 | lever CHEAPER (5x) | grounding STRONG | nature BUILD + 1-line ACTIVATE | effort L | WAVE 1
- Steal/build: output bills 5x input and caching never touches it. FocusTunnel DIFF never emits in prod
  (task_runner.rs:7871 passes editor_fs_override=None -> always UnsupportedMode -> whole-output STRUCTURED); FIM
  is double-gated dark; streaming early-abort is dark on ALL paths (premise "sidecar-only" FALSE) because
  streaming_validator is hardcoded None at runtime_builder.rs:306. H-3: relax ModeSelector Rule 1 + thread a real
  BufferEditor into ResolvedConfig so DIFF emits diffs on cloud; construct Some(validator) at runtime_builder.rs:306
  + call on_chunk every ~50 tokens; write the StreamingEarlyTerminated SSB row (currently only tracing::warn).
  B-20's 3-tier matcher (try_edit_tiers) IS shipped via the STRUCTURED path — REUSE it (GD-1), do not rebuild.
- COMPOSES-WITH: B-28 focus-tunnel, B-20 edit-matching (try_edit_tiers shipped).
- Integration: focus_tunnel/orchestrator.rs:266 (run_diff_step), mode.rs:184 (Rule 1), task_runner.rs:7762/7871,
  runtime_builder.rs:306 (the streaming-validator decision point), runtime_client.rs:508-531 (the early-abort hook).
- observable_effect: a real cloud edit task emits a unified diff / SEARCH-REPLACE (not a whole-file rewrite) with
  fewer output tokens, verified against the journal; a streaming run that hits an early-abort condition writes a
  StreamingEarlyTerminated SSB row and stops decoding. (HD-3: never force-JSON the coder role.)

### H-4 — Tool Search Tool + advanced-tool-use beta header (per-request fixed-cost cut)
- Track 1 | lever CHEAPER | grounding STRONG | nature BUILD | effort M | WAVE 1
- Steal/build: ELAI sends NO advanced-tool-use header and loads its full 16-tool catalog into every request. The
  OAuth-bearer production path (anthropic.rs:695-701) emits ONLY oauth-2025-04-20 and ignores profile betas, so a
  beta added via with_beta is DEAD on the live OAuth path. H-4 joins advanced-tool-use-2025-11-20 onto the OAuth
  beta header AND moves the catalog behind the Tool Search Tool (tool_search_tool_regex_20251119), gated on catalog
  size; since a custom ANTHROPIC_BASE_URL disables server-side deferral, ELAI implements its own (it already does
  via ToolSearch-emulation). Also fixes the C-30 dkg_* tools (catalog-only, no convert/dispatch arm) via the 3-arm
  pattern. ~77K->8.7K tool-def reduction (CORRECTED from 72K).
- COMPOSES-WITH: B-37/E-18 tool catalog, E-40 (symbol_dsl_query 3-arm extension pattern).
- Integration: production_agent_rpc.rs:266 (build_tool_catalog) / :970 (convert) / :1163 (dispatch); anthropic.rs:696-701
  (OAuth beta — the MANDATORY edit) + telemetry_types.rs:69-72 (profile betas).
- observable_effect: a real task with the catalog behind tool-search sends the beta header on the OAuth path
  (verified on the wire), loads tool definitions on demand, and the measured tool-def token count per request drops;
  the catalog-size gate keeps a tiny tool set inline.

### H-5 — Batch API for non-interactive lanes (flat 50% off, zero quality cost)
- Track 1 | lever CHEAPER | grounding STRONG | nature BUILD (no transport exists) | effort M | WAVE 1
- Steal/build: NO Anthropic Batch API anywhere (zero hits); every LLM lane runs synchronously at full price. H-5
  adds a batch client (POST /v1/messages/batches, 50% pricing, 1h cache TTL) and routes the non-interactive lanes
  through it: Cosmos refresh (cosmos_worker tick() — the clearest case, a sequential per-artifact for-loop), web-
  research fan-out, and the bench/eval lanes (RunPod-gated — W-10 tiered: deterministic fixture harness + a
  separately-gated live checkbox). Keep the interactive /task + chat REPL on the synchronous streaming path.
- COMPOSES-WITH: B-42 Cosmos, B-7 research, E-8 GEPA, E-9/E-49 bench (all batch-eligible, all paying 2x today).
- Integration: anthropic.rs:440/678 (the synchronous chokepoint to be given a batch sibling); cosmos_worker.rs:853
  (tick); task_runner.rs:2172-2188 (research run). frontend_pair: batch-status tile (optional).
- observable_effect: `elai` routes a Cosmos refresh batch to /v1/messages/batches, polls to completion, and a
  batch_lifecycle SSB row (Submitted->Completed) lands with the 50%-priced spend recomputable from the journal;
  the interactive path is unaffected (a human-waited call never batches).

### H-6 — Deterministic observation masking + memory read-back (cache-preserving context efficiency)
- Track 1 | lever CHEAPER | grounding STRONG | nature ACTIVATION | effort M | WAVE 1
- Steal/build: deterministic observation masking halves cost while matching LLM-summarization and PRESERVES the
  cache prefix (HD-2). Three real-but-never-called symbols: adaptive_reduce (context_budget.rs:1525, zero non-test
  callers), recover_stubs (session_dag.rs:1507, real blake3 SSB recovery but prod substitutes a hardcoded
  "[stubbed]" placeholder by NOT calling it), get_relevant_facts (retrieval.rs:46, memory read-back, read-dark).
  H-6 makes adaptive_reduce the default long-session manager, un-gates the E-25 ladder (default-on cheapest-first),
  and wires recover_stubs + get_relevant_facts into the live read paths.
- COMPOSES-WITH: E-25 compaction ladder, F-5 memory read; reuses the existing ContextReduction SSB payload (NO new
  action_type — GD-1).
- Integration: live_cli.rs:1252-1264 (the placeholder read-back — the ACTIVATE target for recover_stubs);
  task_runner.rs:9379 (PreCompact ladder branch); compaction_ladder.rs:45 (the default-OFF gate to promote).
- observable_effect: a long real session masks trailing observations (ContextReduction rows land) WITHOUT busting
  the cache prefix (HD-2 assert holds), a /continue rehydrates real content via recover_stubs (not the placeholder),
  and a memory-relevant turn reads back facts via get_relevant_facts; the masked tokens stay re-fetchable from the journal.

### H-7 — Calibrated routing: architect / editor / verifier (smarter orchestration, not a smarter model)
- Track 1 | lever SMARTER-ORCH | grounding STRONG | nature BUILD | effort L | WAVE 1
- Steal/build: the per-role RuntimeOrchestrator + PerRoleRuntime are built but UNWIRED (zero task_runner dispatch
  refs); RuntimeRoutingPolicy::evaluate is doubly inert (default-OFF + observational, does not change the routed
  provider); RouteDecisionEvent has full machinery but NO production emitter; the live leg is only CloudModeRouter
  cap-enforcement + EscalateToCloud. H-7 wires the architect (cloud flagship, reasons/plans) + editor (cheap 7B,
  cheaply-checkable edits) + independent-context verifier (the auditor) on top of the live cap leg, replaces the
  hardcoded layer_failed:1 with a CALIBRATED signal (RouteLLM/AutoMix-style external layer, never 7B self-
  confidence), feeds evaluate a REAL RoutingContext, and emits RouteDecisionEvent. Architect/editor split = +9.6pp
  Aider. Strictly cost-AND-correctness gated (a cheap edit failing an off-agent check is not cheaper). GD-4: the
  over-escalation rate is bounded on >=3 runs before default-on.
- COMPOSES-WITH: B-37D RuntimeOrchestrator, E-31 RouteDecisionEvent (wires the dark emitter, restates neither).
- Integration: runtime_setup.rs:533-536 (the live CloudModeRouter to build on); task_runner.rs:7606 (layer_failed:1);
  escalation_policy.rs:261 (evaluate); cloud_mode.rs:416 (emit_route_decision_log — wire Ssb::write_payload).
- observable_effect: a real task routes a cheaply-checkable edit to the editor and the plan/reason step to the
  architect, the verifier runs in a separate context, a RouteDecisionEvent SSB row records the calibrated decision
  (recomputable, GD-2), and the over-escalation rate is measured on >=3 runs.

### H-8 — Persist OracleAnswerCache (+ idempotent grounding cache)
- Track 1 | lever CHEAPER | grounding STRONG | nature BUILD (persistence) | effort S | WAVE 1
- Steal/build: OracleAnswerCache (blake3-keyed, tier-derived TTL) is wired into /research|/ask but IN-PROCESS ONLY
  (Mutex<CacheInner>, lost on exit). H-8 disk-backs it so idempotent grounding answers survive restart. PromptCache
  ALREADY persists to disk ($HOME/.elai/cache/prompt-cache) — do NOT re-implement it (GD-1; the design "ephemeral"
  framing of PromptCache is wrong). Confine durable caching to IDEMPOTENT grounding queries only, never context-
  dependent code generation (staleness = correctness hazard).
- COMPOSES-WITH: C-30 oracle cache; contrasts PromptCache (already disk-backed).
- Integration: oracle_cache.rs:173 (the in-process struct); repl_command.rs:286-459 (the live get/insert consumer +
  Hit/Miss SSB; cli.clock drives TTL, C-15 deterministic).
- observable_effect: a /research answer cached in one process is a CACHE HIT in a fresh process (the answer survives
  restart, OracleCacheHit SSB row), and a code-gen query is NEVER served from the durable cache (the confinement guard).

### H-9 — Cost cockpit dashboard (activate the dark spend ledger; the believe-the-number instrument)
- Track 1 | lever PROVABLE | grounding STRONG | nature ACTIVATE + BUILD | effort M | WAVE 1 | the H-1/H-2 instrument
- Steal/build: the cloud spend ledger is dark END-TO-END — SpendAccumulator + CloudSpend SSB emit are gated
  `if let Some(ssb)`, and the sole cloud client ctor never chains .with_ssb_writer, so CloudPanel polls and renders
  ALL-ZEROS; cost_per_task_cents is always None (test-only feeder); there is no token-tracker UI. H-9 attaches
  .with_ssb_writer on the cloud client (the pattern TemplateCrystallizer already uses correctly), builds the cost
  feeder, and adds the absent component: a per-response input/output/CACHED breakdown + a cache-expiry timer +
  $/resolved-task + cache-hit-rate live (stealing Devin Desktop's per-response UX). This is the instrument that
  makes H-1/H-2 falsifiable.
- COMPOSES-WITH: C-5 CloudPanel, C-25 CumulativeTokenTracker, E-49 metrics; consumes the dark CloudSpend emit.
- Integration: grounding_client.rs:887-940 (the dark CloudSpend block — ACTIVATE by attaching the writer);
  commands.rs:5088 get_cloud_spend -> CloudPanel.svelte (the zero-rendering read path). frontend_pair: the new token-tracker component.
- observable_effect: a real cloud task makes CloudPanel render NON-zero input/output/cached token counts + live
  $/task + cache-hit-rate, every number recomputable from the journal (GD-2); dashboard-smoker confirms the panel updates.

================================================================================
TRACK 2 — CANVAS-WORKBENCH
================================================================================

### H-10 — Canvas shell + CODE-GRAPH (read-only) [the thesis slice]
- Track 2 | lever STEERING | grounding STRONG | nature BUILD | effort L | WAVE 2 | BLOCKING B0
- Steal/build: a 2D generative canvas as a new WindowHost window type rendering the codebase graph, dead code
  visibly disconnected. CORRECTION C11 (INVERTED): the symbol CALL graph IS already on canvas (build_symbol_view ->
  get_graph_view_data mode='symbol' -> GraphWindow); the producer-dark piece is PageRank (get_pagerank_top reads a
  dkg_nodes_pagerank table NOTHING materialises) and dkg_edges carries NO call edges. B0 (blocking, no code ships
  after a fail): measure the real worst-case DKG node+edge count and pick cytoscape-WebGL (3.34, provisional —
  graceful fallback) vs sigma@3.0.3 (v4 is alpha = BLOCK) by the §4 rule. Then semantic zoom + LOD + the dead-code
  island read with the entrypoints-map supplement + coverage_count backstop (HD-5: topology-suspected rendered
  distinct from coverage-confirmed-dead). Build the persistent PageRank materialiser.
- COMPOSES-WITH: B-29 symbol-graph, B-33 graph commands (get_graph_view_data/VizData), B-34 entry-points, Project-Intelligence/DKG.
- Integration: commands.rs:2507 (get_graph_view_data), views.rs:175 (build_symbol_view), entry_points/detect.rs:142;
  WindowHost.svelte (new window type). frontend_pair: Canvas.svelte (cytoscape WebGL host).
- observable_effect (HD-4/HD-5): open a project, the codebase renders on canvas at the measured >=60fps for the B0
  worst-case (the showFps HUD is the off-agent artifact); a known true orphan islands apart AND a Tauri-invoked-only
  function does NOT render dead (entrypoints map attached it); the PageRank table is materialised and the pagerank
  panel is non-empty.

### H-11 — Live SSB layer + file explorer + scoped fs surface
- Track 2 | lever STEERING | grounding STRONG | nature BUILD | effort M | WAVE 2
- Steal/build: (a) a CanvasActivityEvent SSB overlay drawn on the active mode — active-node halo + traversed-edge
  dash-flow + follow-agent toggle + the ghost-node resolution-failure path (un-indexed file -> transient ghost ->
  reconcile or fade to dropped-marker, never a silent gap); (b) a VS Code-style file explorer with bidirectional
  sync over the EXISTING graph focus bus (focus_node/consume_focus, keyed on path). The hard blocker: NO fs surface
  exists (no tauri-plugin-fs dep, no fs capability, dialog-only) — BUILD it scoped per root (verify the plugin
  version, do NOT assume). ACTIVATE the orphaned list_projects/create_project (registered, zero UI callers).
- COMPOSES-WITH: C-4 project-open (open_project_from_folder/reindex_now shipped; list_projects/create_project dark),
  B-29 (the focus bus graph.svelte.ts).
- Integration: graph.svelte.ts:31 (focus_node bus — REUSE), commands.rs:594/5936 (project open/reindex), default.json:6
  (the capability grant to add), Cargo.toml:22 (the fs dep to add), lib.rs:435 (the plugin to register).
- observable_effect: open a folder, the tree renders from disk via the new scoped fs commands; selecting a within-
  root canvas node highlights the file, a cross-root node shows the mount affordance, an external node shows the
  external state (not a silent no-op); a live task's focus marker travels the graph from real SSB rows and an un-
  indexed file renders a reconciling ghost.

### H-12 — PLAN + RESEARCH modes + shared chrome (orbit feature-parity, flat)
- Track 2 | lever STEERING | grounding STRONG | nature BUILD (port) | effort M | WAVE 2
- Steal/build: port OrbitMode1 (plan) and OrbitMode3 (research) to flat clipping-frame modes on the same canvas:
  PLAN off plan.svelte.ts (concept>phase>step>babystep as contains-nested nodes, FSM-active node lit), RESEARCH off
  the research tree (cites + red contradiction edges). Shared chrome: one zoom, one search, one inspector, one mode
  bar (a plain flag on canvas.svelte.ts, NOT an FSM substate — CANVAS §7). Make Canvas the default window behind a
  workspace flag with the SINGLE-CONSUMER SSB handover (active window owns the subscription; orbit paused) — the
  canvas-store isolation gate already PASSES (C12), so this is clean.
- COMPOSES-WITH: plan.svelte.ts + the research tree (OrbitMode1/3 parity); workspace.svelte.ts default-panel switch.
- Integration: WindowHost.svelte (the activity/plan/graph dispatch), workspace.svelte.ts:82 (default panel),
  the canvas mode bar. frontend_pair: PLAN/RESEARCH frames + mode bar.
- observable_effect: the live decomposition tree renders flat with the FSM-active node lit from real plan state; a
  real research run renders sources + contradiction edges; Canvas is the default and ONLY one window holds the live
  SSB subscription at a time (no double-application into a shared store on switch).

### H-13 — Model drives the canvas (ops + dispatcher + SSB action_types + ARCHITECTURE mode)
- Track 2 | lever GENERATIVE | grounding STRONG | nature BUILD | effort L | WAVE 2 | HD-4 off-agent gate
- Steal/build: the canvas as a tool the model calls. An incremental op stream (createNode/connect/updateNode/
  annotate/fork...) via canvas_* ToolDefinitions added through the E-40 3-arm pattern (catalog + convert + dispatch);
  a CanvasDispatcher (mode-namespace gating + sanitizeAction repair — rejects dangling-id ops, never silently fixes);
  ONE Pattern-2 CanvasOpApplied + CanvasActivityEvent SSB action_type (9-tuple lockstep, baseline+2); ARCHITECTURE
  mode (canvas_render_diagram + mermaid import bridge); per-mode constrained grammars with a CI compile ceiling +
  grammar_unavailable fallback (HD-3); canvas_screenshot -> Tauri-native capture -> media_validator Tier2 (the Tier2
  consumer IS shipped — REUSE, GD-1).
- COMPOSES-WITH: the tool catalog (H-4 3-arm pattern), the 9-tuple SSB lockstep, media_validator Tier2 (shipped),
  F-3p1 CommittedToolSet (live).
- Integration: production_agent_rpc.rs:266/970/1163 (the 3 tool arms); ssb.rs (the 9 lockstep sites, SCHEMA baseline+2);
  task_runner.rs:6208 (the media_validator wire). frontend_pair: CanvasPanel applying canvas_* rows live.
- observable_effect (HD-4, AUDITOR-driven, gate frozen before the slice): an auditor drives a real ELAI task
  authored BEFORE H-13 began; the canvas_op_applied SSB rows land AND the nodes render, observed by the auditor (the
  off-agent artifact), NOT the implementer's stdout; the gate FAILS if the dispatcher is wired to nothing in prod
  even when a unit path passes; a forced grammar-timeout surfaces a blocked op with a reason (never silent free-form).

### H-14 — APP/WEBSITE DESIGN mode (web-first; RN/Expo deferred)
- Track 2 | lever GENERATIVE | grounding MODERATE | nature BUILD (net-new, riskiest) | effort XL | WAVE 2 | GD-4/GD-5
- Steal/build: a design.page node renders a live preview of a real app screen, nav edges wire pages, click an
  element inside a page to edit it. B0 (prerequisite): the dev-preview proxy serving a frameable origin (relax
  X-Frame-Options / frame-ancestors + scope the Service Worker). Click-to-edit: build-time data-oid OPAQUE-id
  tagging mirroring @onlook/parser ids.ts in-repo (NOT the phantom @onlook/babel-plugin-react; store id->{file,range,
  scope} SERVER-SIDE, do NOT encode line/col in the DOM attr — CORRECTED). Two-tier writeback: deterministic recast
  (0.23.11, MIT) patches for style/text, the LLM only for structural — gated on a recast prove-it spike against a
  real large React codebase (current prior art cannot diff-against-existing — GD-5 honest limit). NOVELTY re-grounded
  on the SANDBOX/PROXY axis (Onlook is iframe-based — REFUTED renderer novelty).
- COMPOSES-WITH: recast (verified), @onlook/parser ids.ts (mirror, not depend); the canvas op stream (H-13).
- Integration: a new dev-preview proxy + DesignPageNode.svelte + the build-time tagging step + a recast writeback
  command. frontend_pair: DesignPageNode (proxied-iframe live preview).
- observable_effect (HD-4/GD-4/GD-5): a real app's pages render as preview nodes verified by an off-agent screenshot
  of an actually-painted preview (not a 200); clicking an element resolves to its AST range via the opaque id; a
  style edit round-trips as a deterministic recast patch with NO model call, a structural edit round-trips through
  the model with recast preserving formatting, both applied to a real codebase; the writeback corruption rate is bounded.

### H-15 — spawn_view reply channel + taint-gated deixis + snapshot/delta [the trust-spine moat]
- Track 2 | lever GENERATIVE | grounding STRONG | nature BUILD | effort L | WAVE 2 | depends H-13, G-1, G-2
- Steal/build: the canvas as the model's general-purpose visual reply channel — a spawn_view{view_type,seed_ops[]}
  verb (the field-converged content-type routing), composed from existing ops. ONE Pattern-2 CanvasViewEvent SSB
  (ViewSpawned/UiAction/StateCheckpoint, baseline+1). Three slices: H15-1 comparison/branch board (cheapest high-
  value view, pure existing ops); H15-2 the inbound CanvasUiAction::Select with tiered read-back GATED by a G-1
  taint check (a web-sourced node's content flagged at the handoff — the part Artifacts/Canvas cannot do); H15-3
  snapshot+delta reconciliation per tab (sequence numbers + periodic checkpoints so a tab rebuilds from the journal
  tail). The MOAT = the journaled (SSB) + reproducible (G-2 RRA) + taint-tagged (G-1) op-stream the consumer
  products lack.
- COMPOSES-WITH: G-1 taint_flow (gates Select — restate none), G-2 RRA (a view is "produced by run X, reproduces
  from the journal"), H-13 (the op transport + dispatcher); the AG-UI / MCP-Apps convergence (internal contract only,
  not interop — CANVAS §20.3).
- Integration: production_agent_rpc.rs (the spawn_view verb, 3-arm); ssb.rs (CanvasViewEvent, baseline+1);
  G-1 capability_flow::taint_text (reuse) at the Select handoff. frontend_pair: the renderer registry + comparison board.
- observable_effect (HD-4, auditor-driven): a real reply path emits a ViewSpawned row and the comparison board
  renders (off-agent SSB row + Layer-4 trace assert, not a unit test); selecting a node hands its EXACT current
  state to the model and a tainted (web-sourced) node is flagged at the handoff; kill+reopen the dashboard mid-run
  and the tab reconstructs from the SSB tail.

### H-16 — Retire the orbit
- Track 2 | lever CLEANUP | grounding STRONG | nature DELETE | effort S | WAVE 2 | LAST
- Steal/build: delete the three.js scene/ layer + OrbitMode1/2/3 ONLY after Canvas-as-default ships a full release
  with no regressions. The canvas-store isolation gate already PASSES (C12 — no store imports three.js), so removal
  is clean: drop scene/three-scene.ts + three-orbit*.ts + StarCanvas.svelte + sky-dome.ts + star-points.ts +
  orbit/node-mandala.ts + OrbitCanvas + ActivityWindow's orbit render + the three/troika/@types/three deps; the
  starfield is already 2D by then. The cleanest, most mechanical, ungameable gate in the plan.
- COMPOSES-WITH: H-12 (Canvas-as-default + single-consumer handover must have shipped first).
- Integration: workspace.svelte.ts:82 (default panel — must already be canvas), WindowHost.svelte:81 (activity
  dispatch), package.json:23 (the three deps to drop), the scene/ + orbit/ three.js graph.
- observable_effect: build is GREEN with three / troika-three-text / @types/three removed and no orbit imports
  remain (the off-agent artifact = a clean build + a zero-grep for `from.*scene/three`); the dashboard renders the
  canvas with no regression.

================================================================================
TRACK 3 — TRUST-HARDENING (autonomous-trading-trust gates; T-1..T-13)
================================================================================

### H-17 — Persisted untamperable trial ledger (T-1) [the selection-bias KEYSTONE]
- Track 3 | cluster A SELECTION-BIAS (TB-1) | grounding STRONG | nature BUILD on shipped SSB WAL | effort S-M | WAVE 3 | BLOCKING
- Steal/build: an autonomous optimizer ships overfit edge even with perfectly honest agents — the max of N noisy
  Sharpe ratios is upward-biased (an in-sample Sharpe>1.0 after ~7 noise trials). The single deterministic fix is a
  persisted, append-only, agent-unresettable cumulative trial count N. BUILD strategy_trials (a new table on the
  SHIPPED SSB WAL) + a single register_trial() choke-point every backtest/eval passes through + a trial_count() read
  API. No agent delete/reset/update; ONC effective-N clustering for correlated trials. register_trial/strategy_trials
  are ABSENT (NN-1, zero hits) — pure BUILD.
- COMPOSES-WITH: the SSB append-only WAL (no UPDATE/DELETE; reuse the discipline, GD-1); gate_freeze_check.py +
  gate-manifest (TD-2 — the ledger + register fn join the frozen set); the rules-engine mutable=0 tier (H-18 reads N).
- Integration: new crates/orchestrator/src/trial_ledger.rs; the backtest/eval path routes through register_trial;
  +TrialRegistered SSB (baseline+1). frontend_pair: N surfaced by the H-18/H-29 cockpit.
- observable_effect: a fixture search registers >=3 distinct configs -> N TrialRegistered rows + a monotone
  trial_count recomputable from the journal (GD-2); a config with zero rows fails closed downstream (trial_count_missing, TD-1).

### H-18 — Deflated-Sharpe / PBO / MinBTL promotion gate (T-2) [#1 net-new trust gate]
- Track 3 | cluster A SELECTION-BIAS (TB-1) | grounding STRONG | nature BUILD (W-10 tiered) | effort M | WAVE 3 | depends H-17
- Steal/build: ELAI has NO finance-deflation evaluator wired to promotion (NN-2). BUILD a LOCKED COMPOSITE rules-engine
  gate that BLOCKs promotion unless Deflated Sharpe Ratio + PBO(CSCV) + MinBTL clear thresholds, with N read from H-17,
  failing CLOSED on trial_count_missing. The DSR/PBO/MinBTL math runs in a FROZEN numpy/scipy sidecar (Tier-2); the Rust
  METRIC_GATE enforces the verdict (Tier-1, deterministic — TD-3). Thresholds LOCKED mutable=0 (TD-2; Harvey-Liu t>3.0 default).
- COMPOSES-WITH: H-17 trial_count (N); the rules-engine COMPOSITE/MetricGate (rule_engine.rs:98); the SecCodePltRunner/
  safedelta_runner.rs frozen-evaluator + measured-SSB-row pattern (mirror, GD-1).
- Integration: new crates/orchestrator/src/deflation_gate.rs + a numpy/scipy sidecar + crates/rules/seed/finance/
  deflation_promotion.toml (mutable=0); the strategy promotion path; +DeflationGateEvaluated SSB. frontend_pair: the {DSR,PBO,N} drawer.
- observable_effect: a strategy that beats raw Sharpe but fails DSR-at-N is BLOCKED with the triple persisted; a
  genuine low-N strategy PASSES; a config with zero trial rows is BLOCKED (fail-closed). DSR is NECESSARY not SUFFICIENT (TD-5).

### H-19 — Reusable-holdout query budget (T-3)
- Track 3 | cluster A SELECTION-BIAS (TB-1) | grounding STRONG | nature BUILD | effort M | WAVE 3 | depends H-17 | deployment-trust half
- Steal/build: nothing bounds adaptive OOS-holdout reuse — a searcher mines the holdout as an unlimited analyst.
  BUILD a differentially-private fresh-holdout budget (Thresholdout/Ladder, ABSENT NN-3): a LOCKED append-only
  budget counter on the SSB (mirrors H-17's no-reset discipline) + a Thresholdout-style DP OOS query; after the budget
  is spent, OOS queries are REFUSED (budget_exhausted) until a fresh holdout is provisioned (frozen/human path only).
- COMPOSES-WITH: H-17's append-only-counter discipline; the rules-engine MetricGate (the refusal). Closed-form DP noise (no sidecar).
- Integration: new crates/orchestrator/src/holdout_budget.rs + crates/rules/seed/finance/holdout_budget.toml (mutable=0);
  the OOS-evaluation path; +HoldoutBudgetEvent SSB (Pattern-2 QuerySpent/Exhausted/Provisioned). frontend_pair: a budget gauge.
- observable_effect: budget_total OOS queries spend, the next is REFUSED (Exhausted row); a fresh holdout provisions only
  via the frozen path (an agent reset does not compile); spend recomputable from the journal (GD-2).

### H-20 — Leakage static-analysis COMPOSITE on the taint graph (T-4) [the trading-specific killer]
- Track 3 | cluster B LEAKAGE (TB-2) | grounding STRONG | nature BUILD on taint primitive | effort M-L | WAVE 3 | G-1 prereq flagged
- Steal/build: "compiles + tests pass" is structurally blind to fit-before-split / cross-boundary normalisation /
  full-series-PCA / missing signal->execution shift. BUILD a T1 COMPOSITE over ELAI's taint/provenance graph asserting
  no fit-site reaches both train+test sinks, no normalisation taints earlier-timestamp data, no full-series PCA before
  split; + a SCHEMA_REQUIRED split-boundary declaration; + a T2 AST_PATTERN for the missing .shift(1). FAIL-CLOSED with
  a REGISTERED suppression allowlist (never silent). Sound for fit/normalise/PCA; target-derived routes to T3 (TD-3/5).
- COMPOSES-WITH: the SHIPPED taint primitive (provenance.rs:145, capability_flow taint_text:534 — wire-dark/partial per C17);
  the rules-engine AST_PATTERN/COMPOSITE/SchemaRequired. PREREQ: G-1 live taint (taint_flow.rs) is ABSENT — compose with G-1 OR build a leakage-scoped DKG dataflow pass (TD-5, decide at authoring).
- Integration: new crates/rules/src/leakage_composite.rs + crates/rules/seed/finance/leakage.toml (mutable=0);
  PreToolUse/pre-promotion; +LeakageBlocked SSB (Pattern-2 4-Kind). frontend_pair: the tainting fit-site drawer.
- observable_effect: a fit-before-split pipeline is BLOCKED with the fit-site reported; a clean split-then-fit PASSES;
  a flagged pandas idiom requires a REGISTERED suppression (no silent skip). The runtime backstop for what static misses is H-21.

### H-21 — Runtime look-ahead / causality differential gate (T-5) [the un-gameable leakage backstop]
- Track 3 | cluster B LEAKAGE (TB-2) | grounding STRONG | nature BUILD (W-10 tiered) | effort M | WAVE 3
- Steal/build: static leakage analysis is blind to dynamic look-ahead (a name is not a dataflow edge). BUILD the
  perturb-and-diff backstop (ABSENT NN-5; Freqtrade look-ahead-analysis): run the feature/signal pipeline TWICE — full
  data vs future-truncated — and assert the output columns are IDENTICAL (ulp/relative tolerance). Any column that
  changes when the future is removed uses the future. Tier-2 frozen feature-runner; Tier-1 Rust column-diff.
- COMPOSES-WITH: DriftComparator's first-divergence-index reporting (determinism.rs:317 — reuse the shape, GD-1); the rules-engine MetricGate.
- Integration: new crates/orchestrator/src/lookahead_differential.rs + a frozen feature-runner sidecar; pre-promotion;
  +LookaheadDifferential SSB (Pattern-2 Causal/FutureLeak). frontend_pair: the first-changing-column in the leakage drawer.
- observable_effect: an iloc[t+1] feature is BLOCKED because its column differs under future-truncation; a causal
  pipeline diffs to zero and PASSES. Cross-hardware FP residue is honest (TD-5 — relative tolerance, not exact bitwise).

### H-22 — Trading metamorphic null-test COMPOSITE (T-6) [does the edge survive perturbation]
- Track 3 | cluster C PERTURBATION (TB-1+TB-2) | grounding STRONG | nature BUILD (W-10 tiered) | effort L | WAVE 3 | depends H-17
- Steal/build: a leaking/overfit backtest passes every existing gate. BUILD a frozen COMPOSITE of named metamorphic
  relations (ABSENT NN-6) — future-shuffle (edge must collapse to noise), time-reversal, zero-drift permutation null
  (+ DSR vs the null, N from H-17), scale & additive invariance, cost monotone+concave (sqrt-law), seed-determinism
  (equity-curve blake3 hash). ADD the named relations to the SHIPPED RelationLibrary (metamorphic_testing.rs); reuse
  MrViolated for failures. Only the human-set alpha is LOCKED (TD-2); the relations are mutation-tested by H-27 (TD-4).
- COMPOSES-WITH: the SHIPPED metamorphic_testing.rs (MetamorphicRelation/RelationLibrary/MetamorphicTestRunner/MrViolated, GD-1); H-17 N; the rules-engine MetricGate.
- Integration: extend metamorphic_testing.rs (new relations) + a permutation null-band sidecar + crates/rules/seed/
  finance/trading_nulls.toml (mutable=0); pre-promotion + a commit hook; +TradingNullTest SSB (Pattern-2 4-Kind). frontend_pair: the null-band.
- observable_effect: a leaky strategy whose edge survives the shuffled future is BLOCKED; a genuine strategy's edge
  collapses to noise and PASSES; the permutation null-band is the persisted off-agent artifact (missing = FAIL).

### H-23 — Same-seed re-run hash-equality METRIC_GATE (T-7) [reproducibility centerpiece]
- Track 3 | cluster D REPRODUCIBILITY | grounding STRONG | nature BUILD on shipped RRA/determinism | effort M | WAVE 3
- Steal/build: ELAI_SEED asserts INTENT but nothing RE-RUNS and compares the DECLARED METRIC OUTPUTS. BUILD a gate
  (ABSENT NN-7) that re-invokes the real entrypoint under the recorded env+ELAI_SEED (via the SHIPPED RRA/replay),
  blake3-hashes the DECLARED outputs (not stdout), and asserts equality vs run#1 (DriftComparator first-divergence) +
  a seed-coverage AST gate (unseeded np/torch/random/DataLoader-shuffle) + a SCHEMA_REQUIRED env-pin manifest. SCOPE
  (TD-5): same-env/same-seed ONLY; cross-hardware FP is physics — record, never assert.
- COMPOSES-WITH: the SHIPPED RRA (rra.rs:200, --emit-rra, replay run_subcommand.rs:584 — G-2; G-summary "ABSENT" STALE) +
  DriftComparator (determinism.rs:317); the rules-engine AST_PATTERN/SchemaRequired. Restate neither (GD-1).
- Integration: new crates/orchestrator/src/rerun_equality.rs + extend rra.rs (metric-output digest) + crates/rules/
  seed/repro/{seed_coverage,env_pin_manifest}.toml; elai replay --verify --metric-digest; +RerunHashEquality SSB (Equal/Divergent). frontend_pair: the RRA viewer digest badge.
- observable_effect: an experiment with a hidden unseeded RNG FAILS (run#2 digest != run#1); a clean one hashes Equal
  twice; the seed-coverage AST gate BLOCKs the unseeded call at TESTING_FIRST.

### H-24 — Sim-to-real parity gate (T-8)
- Track 3 | cluster D PARITY | grounding STRONG | nature BUILD (A+C build, B deferred) | effort M | WAVE 3
- Steal/build: nothing asserts research-path == live-path, nor replay == golden. BUILD (ABSENT NN-8): (A) a T1
  AST_PATTERN banning mode-divergent branches (`if live:`/`if backtest:`, duplicate fill/router) + a SCHEMA_REQUIRED
  single-shared-entrypoint manifest; (C) a T1 METRIC_GATE that replays the recorded event log under ELAI_SEED and
  asserts blake3(replay)==blake3(golden), reporting the first mismatch. (B) behavioral numeric parity = a later Tier-2
  gate needing a live/paper run + LOCKED thresholds, DEFERRED (GD-5 — no parity claim before it).
- COMPOSES-WITH: replay_resume (ssb.rs:23502) + RRA replay (run_subcommand.rs:584); the rules-engine AST_PATTERN/SchemaRequired/MetricGate; DriftComparator. Restate neither (GD-1).
- Integration: new crates/orchestrator/src/sim_real_parity.rs + crates/rules/seed/repro/sim_real_parity.toml (mutable=0);
  PreToolUse (A) + pre-promotion (C); +SimRealParity SSB (Pattern-2 3-Kind). frontend_pair: the parity drawer.
- observable_effect: an `if backtest:` fast-path is BLOCKED at build; a divergent replay FAILs the golden hash with the
  first mismatch; a single-entrypoint strategy with a clean replay PASSES. B (numeric parity) is honestly deferred Tier-2.

### H-25 — Result manifest + re-derivation verifier (T-9) [bind the empirical NUMBER, not just the test]
- Track 3 | cluster E ANTI-FABRICATION | grounding STRONG | nature BUILD extending F-4 | effort M | WAVE 3
- Steal/build: F-4 binds a test-PASS to a captured artifact, but NOTHING binds an empirical RESULT (Sharpe/P&L) to its
  provenance (the Replit faked-results class). BUILD (ABSENT NN-9) a SCHEMA_REQUIRED Result Manifest {value,
  captured_output_blake3, parser_expr, data_hash, code_commit, env_lock_hash, elai_seed, run_id, n_trials, dsr_pvalue,
  pbo} + a T1 verifier that RE-PARSES the number from the captured output and RE-HASHES the pinned inputs; + an
  AST_PATTERN forbidding strategy code from importing/redefining/monkeypatching the (frozen) metric harness. Extends
  the SHIPPED F-4 stdout_hash discipline from "test PASS honest" to "result honest"; DSR/PBO/N fields bound from H-17/H-18.
- COMPOSES-WITH: the F-4 stdout_hash leg (evidence.rs:845, gate.rs:441 — extend, restate none, GD-1); RRA evidence manifest; H-17/H-18 (the bound fields).
- Integration: new crates/orchestrator/src/result_manifest.rs + crates/rules/seed/finance/result_manifest.toml (mutable=0);
  the result-reporting path; +ResultManifestVerified SSB (Pattern-2 ReDerived/FabricationDetected/MonkeypatchBlocked). frontend_pair: the reviewer cockpit (composes G-5).
- observable_effect: a hand-edited Sharpe whose captured-output digest does not re-derive is FAILED; a clean manifest
  re-derives; a strategy that monkeypatches the metric fn is BLOCKed by the AST rule. Visual results route to the media_validator VLM (HD-5).

### H-26 — Runtime differential / shadow exec + financial-invariant MR library (T-10)
- Track 3 | cluster E DYNAMIC-VERIFICATION | grounding STRONG | nature BUILD (W-10 tiered) | effort M-L | WAVE 3
- Steal/build: static + wiring gates cannot see WRONG VALUES (a flipped P&L sign, ULP cancellation, a put-call-parity
  violation). BUILD (ABSENT NN-10): a seeded shadow executor running two impls (new vs prior / vs QuantLib oracle) on
  shared inputs, gating on bounded ULP/relative divergence (FPDiff); + a financial-invariant MR pack (put-call parity,
  strike-monotonicity, P&L additivity, scale-invariance, numeric-stability — new named relations); + an AST ban on
  naked float ==. Daikon-mined invariants enter PROPOSED and must kill a seeded mutant first (TD-4).
- COMPOSES-WITH: the SHIPPED metamorphic_testing.rs (new financial MRs) + mutation_testing.rs (the kill-a-mutant admission, GD-1); the rules-engine MetricGate/AST_PATTERN.
- Integration: new crates/orchestrator/src/shadow_exec.rs + a Tier-2 shadow runner + 5 new MRs in metamorphic_testing.rs
  + crates/rules/seed/finance/financial_invariants.toml (mutable=0); pre-promotion + a refactor hook; +DifferentialDivergence SSB (Pattern-2 3-Kind). frontend_pair: the divergence drawer.
- observable_effect: a sign-flipped refactor diverges beyond the ULP bound -> FAIL; a clean refactor diffs within ULP;
  a put-call-parity violation is caught; a naked float == is BLOCKed at build. QuantLib is a Tier-2 ADVISORY oracle (TD-3), not a hard gate.

### H-27 — Gate self-validation (T-11) [verify-the-verifiers; the GBM-band fix generalized]
- Track 3 | cluster F META-LAYER (TB-3) | grounding STRONG | nature BUILD on shipped mutation/metamorphic/trap | effort L | WAVE 3 | capstone
- Steal/build: a self-improving optimizer routes itself to PASS through a MIS-SPECIFIED gate (the literal REGRET
  symmetric-vs-one-sided-band incident). BUILD (ABSENT NN-11) a harness that, per PROPOSED->ACTIVE gate revision,
  mechanically proves the gate (a) kills 100% of a FROZEN out-of-band mutant corpus of its OWN logic, (b) satisfies
  its declared MRs INCLUDING a mandatory null_band_sidedness MR, (c) rejects every frozen trap; any escape blocks
  promotion + increments a persisted escaped-mutant counter. + a non-defaultable null_band_sidedness SCHEMA field +
  an AST ban on abs(x)<band for a one-sided-declared gate. Mechanical, no LLM verdict (TD-3/TD-4). Only sound TB-3 defense.
- COMPOSES-WITH: the SHIPPED mutation_testing.rs (MutationRunner) + metamorphic_testing.rs (RelationLibrary) + trap_tasks.rs (TrapTask:184), turned RECURSIVELY onto the gates (GD-1); the rules-engine lifecycle SHADOW->ACTIVE hook (lifecycle.rs:871).
- Integration: new crates/orchestrator/src/gate_self_validation.rs + a NullBandSidedness MR + crates/rules/seed/meta/
  gate_self_validation.toml (mutable=0) + frozen per-gate mutant corpora; hooks SHADOW->ACTIVE; +GateSelfValidation SSB (Pattern-2 4-Kind). frontend_pair: the meta cockpit.
- observable_effect: a newly-proposed gate that uses abs(x)<band where one-sided was declared FAILS its own
  self-validation and CANNOT go ACTIVE; the escaped-mutant counter increments; a correctly one-sided gate kills 100% of its corpus and promotes.

### H-28 — Current-RF reward-tampering gate + harness-in-the-frozen-set (T-12) [self-mod safety capstone]
- Track 3 | cluster F META-LAYER (TB-3) | grounding STRONG | nature BUILD reusing blake3 tripwire + two_gate | effort S | WAVE 3 | capstone
- Steal/build: ELAI's blake3 tripwire catches RF tampering (harness edit -> hash mismatch) but NOT RF-INPUT tampering
  (score jumps with the hash intact). BUILD (ABSENT NN-12): put the evaluator + metric harness + gate thresholds + the
  H-17 trial ledger ALL in the gate-freeze LOCKED set (Everitt current-RF — the highest-leverage move, TD-2); + a
  COMPOSITE reward-tampering gate that HARD-BLOCKs on an eval-harness blake3 mismatch (reuse the shipped tripwire) and
  FLAGS/BLOCKS an implausible post-self-mod score delta with the hash INTACT; a deterministic 3a BLOCK can never be
  overridden by a probabilistic 3b green (TD-3). Structural self-mod stays human Layer-3 only (TD-5, undecidable).
- COMPOSES-WITH: the SHIPPED blake3 tripwire (ArchiveVerifyResult ssb.rs:30840) + two_gate_promote (promotion_gate.rs:109,
  darwin_admit_loop.rs:298) + LockedKeysAuditor + is_security_sensitive (self_mod_layer1a.rs:383). Restate none (GD-1).
- Integration: new crates/orchestrator/src/reward_tampering.rs + crates/rules/seed/meta/reward_tampering.toml (mutable=0)
  + add the harness+thresholds+H-17 ledger to gate-manifest; darwin_admit_loop before insert_variant; +RewardTampering SSB (Pattern-2 3-Kind). frontend_pair: the self-mod cockpit.
- observable_effect: a harness-editing variant is HARD-BLOCKED on hash mismatch; a score-jump-with-hash-intact variant
  is FLAGGED/BLOCKED for Layer-3; a clean config variant promotes; a structural self-mod is refused to human Layer-3.

### H-29 — LOCKED fail-closed drift state machine (T-13)
- Track 3 | cluster G DEPLOYMENT-TRUST | grounding STRONG | nature BUILD composing shipped signals | effort M | WAVE 3 | depends H-18 | LAST
- Steal/build: the model-drift canary (G-10) is one signal, not the full interlock; no 90-day TTL; the concept-drift
  half is not label-fed. BUILD (ABSENT NN-13) a LOCKED(mutable=0) warning->drift->halt interlock across FOUR axes:
  PSI on inputs AND outputs; schema-fingerprint (vendor drift); rolling-14d live-vs-backtest deviation; 90-day model
  TTL + deflated-Sharpe-at-promotion (H-18). It gates the FSM transition that resumes/continues live trading. Missing
  evidence = HALT (not SKIP). HONEST GUARANTEE (TD-5): "halt within ONE realized-label cycle," NOT "never trade a regime break".
- COMPOSES-WITH: the SHIPPED model_drift.rs (G-10) + canary.rs CanaryFamily (D-7) + conformal_calibration.rs (the T3
  concept-drift residual advisory, D-16/G-7) + H-18 (the TTL DSR re-check). Restate none (GD-1).
- Integration: new crates/orchestrator/src/drift_interlock.rs + crates/rules/seed/finance/drift_interlock.toml (mutable=0);
  the FSM resume/keep-live edge; +DriftInterlock SSB (Pattern-2 Warning/Drift/Halt). frontend_pair: the deployment cockpit.
- observable_effect: a vendor schema change + an output-PSI breach + a 14d-deviation breach HALTS the resume edge until
  a full backtest re-run; a clean deployment within all bands resumes; the 90d TTL forces an H-18 DSR re-check; the P(Y|X) residual routes to the conformal T3 advisory (TD-5).

## Cross-cutting (what makes H-plan different from F/G)
- Every H-N COMPOSES-WITH a prior-plan artifact by consuming its symbol/verdict, or builds a genuinely-absent
  efficiency/canvas surface (GD-1); re-stating apply_anthropic_cache_markers / try_edit_tiers / RouteDecisionEvent /
  CloudSpend / media_validator Tier2 / G-1 taint / G-2 RRA / the 9-tuple lockstep is a P0.
- Every efficiency number (hit-rate, $/task, token ratio, over-block) is MEASURED per config and recomputable from
  the journal under ELAI_SEED (HD-1 + GD-2), never a literature figure and never a product of multiplied levers.
- No change busts the byte-stable cache prefix; masking drops trailing observations, never rewrites the prefix (HD-2).
- Output-token wins count 5x and are never conflated with cache wins; reasoning roles are never force-JSON'd (HD-3).
- Every canvas / model-driving gate is auditor-driven against a pre-authored frozen task; the off-agent SSB row +
  rendered artifact is the proof, never the implementer's stdout (HD-4).
- A rendered picture is not proof; coverage_count is the dead-code backstop; a model-drawn diagram is valid-not-
  verified until VLM/executed-artifact checks its content (HD-5).
- No gate flips default-ON until its FP / over-block / over-escalation / writeback-corruption rate is bounded on
  >=3 real runs (GD-4); no fail-open / cloud-opaque path advertises a guarantee it does not deliver (GD-5).
- (Track 3) Every trust number deflates by a persisted agent-unresettable N (TD-1); the evaluator + harness +
  thresholds + the trial ledger are frozen LOCKED (TD-2); LLM/auditor judgment is T3-advisory, never the promotion
  gate (TD-3); each gate kills its own frozen mutant corpus + a null_band_sidedness MR before ACTIVE (TD-4); the
  irreducible undecidable residue (cross-hardware FP, pre-label concept drift, target-derived leakage, genuine-vs-
  lucky, self-mod safety) is named and routed to human Layer-3 — never faked (TD-5). Track 3 is BUILD-ON-SHIPPED:
  the off-agent-artifact substrate is already live; the finance/search-specific gates are the genuine BUILD.


================================================================================
TRACK 4 -- UX + USER-MEMORY (instrument-first; H-30..H-38)
================================================================================
Generated 2026-06-04 from grounded internet research (run wf_c8950d72-45e, 13/15 lenses)
+ a read of the shipped crates/memory substrate. Premise: ELAI engineers every LLM-problem
but ships almost no user-facing UX and ZERO durable USER memory. Track 4 closes that with:
an INSTRUMENT-FIRST keystone (H-30) that MEASURES a non-stale, token-cheap memory before
H-31/H-32 are trusted; a two-tier user memory that ACTIVATES the already-shipped but wire-dark
crates/memory bi-temporal substrate (soft_close / audit_conflicts / FactType::Preference /
valid_from_ms-valid_to_ms / mem0_dispatcher); and 5 host-side "free-win" QOL surfaces that ride
existing substrate at ~zero local-token cost. Differentiator vs F/G (HD-1): every number is
MEASURED-per-config and journal-recomputable under ELAI_SEED. Staleness is fixed at WRITE time
via bi-temporal supersession (retrieval HARD-FILTERS superseded rows, NEVER narrates [OUTDATED]
= the measured Graphiti +68%-token trap); contradiction-detection runs on the CLOUD model on
WRITES ONLY (spare the 7B); and the AUDITOR / CRITIC / ADVERSARY roles receive ZERO user-memory
injection (Personalization-Trap arXiv:2510.09905) to protect ELAI's trust product.

SCHEMA: live baseline is 220 / 497 (ssb.rs:2990 / action_type_lock.rs:624). CLAUDE.md's 191/464
AND the Track-1/2/3 header's 207/484 are STALE. Each H-file's SCHEMA / TOTAL_VARIANTS bump is a
per-file delta off 220/497 and is SHIP-ORDER-CUMULATIVE (they cannot all be 221/498 if shipped
together); H-34 adds TWO action_types, the rest add one each. Confirm the live const at authoring (W-7).

CITATION CAVEAT (anti-slopsquat, feedback_paper_citation_verify): the future-dated arXiv IDs
2605.06527 (STALE), 2603.23848 (BeliefShift), and 2512.16962 (MemoryGraft, H-32) are beyond the
2026-01 knowledge cutoff and were NOT web-verified during authoring (flagged CLAIM in-file);
WebSearch-confirm each before its number headlines any gate. 2501.13956 (Zep), 2504.19413 (Mem0),
2505.07509 (HALO), 2510.09905 (Personalization-Trap) came grounded from the research sweep --
re-verify IDs at execution given ELAI's prior bad-cite history (B-40/B-42/E-40).

### H-30 -- staleness / intent-drift INSTRUMENT (the measure-before-trust keystone)
- Track 4 (UX + USER-MEMORY) | lever PROVABLE | grounding STRONG | nature BUILD | effort M | WAVE 4 | BLOCKING (gates H-31 + H-32)
- Steal/build: build the staleness instrument BEFORE trusting any durable-user-memory write/read (W-1/R4 measure-the-real-objective). Two instruments: (a) a runtime ratio stale_fact_reached_prompt/total_retrievals emitted as SSB rows, recomputable under ELAI_SEED; (b) a BeliefShift/STALE-style Conflict-Resolution-Accuracy eval harness -- a fixture states X, then contradicting Y, and asserts retrieval acts on Y and NEVER surfaces X, measuring the recognition-vs-application gap. Reimplement (MIT odysseus) the order-independent fingerprint short-circuit (memory_extractor.py:32-35,441-443) with blake3 and the read-time relevance gate (memory.py:255/:291). Mirror the seccodeplt_runner.rs eval-runner shape (NOT a port).
- Existing ground truth (code-read 2026-06-04): the store + supersession + contradiction + decay + trust-filter retrieval ALL EXIST -- MemoryFact (fact.rs:138; valid_to_ms:167/superseded_by:179), FactStore.supersede(store.rs:208)/soft_close(store.rs:303), audit_conflicts(contradiction.rs:64)+adjudicate_semantic(semantic_contradiction.rs:392) PROD-WIRED via live_cli.rs:2034/:2076. get_relevant_facts(retrieval.rs:46) is WIRE-DARK (zero prod callers). conversation.rs has ZERO memory refs and ContextSlot(context_budget.rs:1066, SLOT_COUNT=8) has no memory slot -- so the injection site does not exist yet. H-30 instruments the retrieval CHOKEPOINT (get_relevant_facts), not the not-yet-built prompt slice; the SAME shim wraps the live path when H-32 wires it.
- REVISED scope (W-10 Tier-1 whole, offline, no model/sidecar): a count_retrieval shim (returns a row with valid_to_ms.is_some() OR superseded_by.is_some() == a stale leak) + a deterministic StalenessEvalRunner with an include_str! fixture corpus, a COMPUTED StalenessEvalReport (conflict_resolution_accuracy + recognition_rate + application_rate + gap + stale_fact_reached_prompt + total_retrievals + seed + corpus_blake3), and a --broken-retrieval arm that MUST make stale_fact_reached_prompt>0 (R2: a metric that stays 0 even when broken is itself a FAIL). DEFERRED to H-32: the live retrieval-into-prompt wiring (memory ContextSlot + conversation.rs call). The CLOUD contradiction pass is a WRITE-side H-31 concern; H-30 uses only the offline DeterministicNliJudge (spare the 7B). Default-ON, NO ELAI_*_ENABLE flag (W-2: an eval needs none).
- Integration: NEW crates/orchestrator/src/benchmarks/staleness_runner.rs + crates/memory/src/retrieval_metrics.rs; MODIFY crates/cli/src/bench_cmd.rs (+run_bench_staleness, mirrors :139/:149) + ssb.rs (+StalenessEvalMeasured action_type, Pattern-2; SCHEMA baseline 220+1=221, confirm ssb.rs:2990 at edit) + action_type_lock.rs:624 (497->498, lockstep) + the e45 cross-lang drift contract (stale 482). frontend_pair: none at WAVE 4 (surfaced read-only by the H-9/H-33 cockpit; W-4 producer-only N/A).
- observable_effect: `cargo run -p elai-cli -- bench staleness --json` (no env override) -> ONE StalenessEvalMeasured SSB row with conflict_resolution_accuracy COMPUTED from fixtures AND stale_fact_reached_prompt==0 on the clean path, byte-recomputable from the journal under ELAI_SEED (GD-2); `--broken-retrieval` makes stale_fact_reached_prompt>0 and drops accuracy (proves the metric is not vacuously 0). Gates H-31/H-32: neither ships until this eval reports recognition AND application on the supersession path.

### H-31 -- Tier-B user-preference bi-temporal ledger (activate wire-dark crates/memory into a production user-decision path)
- Track 4 (UX + USER-MEMORY) | lever STEERING/TRUST | grounding STRONG | nature ACTIVATION | effort M-L | WAVE 4 | depends H-30
- Premise: a user states a durable WORK preference once ("prefer tracing over log crate", "auditor before commit") and ELAI honours it across sessions without re-asking, then quietly retires it the instant the user states the opposite -- never narrating the dead rule, never silently auto-learning one, never leaking a stale/flattering rule into an auditor's prompt.
- Steal/build: ACTIVATION not BUILD. The bi-temporal memory crate already exists; the write-side contradiction/decay reconcile is already PROD-WIRED in the live tick (live_cli.rs:2034/2076). Builds exactly 3 missing wires + ZERO new storage: (a) capture PRODUCER (/remember + propose-on-repeated-correction) into the test-only `dispatch_batch`; (b) retrieval CONSUMER calling the wire-dark `get_relevant_facts` into prompt-assembly behind a new ContextSlot; (c) one supersede SSB beacon + a frozen smoke test. odysseus MIT prior-art for read-time relevance gate (memory.py:291/255), high-threshold dedup (memory_vector.py:116), >50%-deletion guard (memory_extractor.py:535-540) -- reimplement in Rust, never port.
- Existing ground truth (code-read): store.rs:208 `supersede` (atomic soft-close older + insert newer, DELETE never issued), store.rs:303 `soft_close`, store.rs:147 `list_current` (the `valid_to_ms IS NULL` filter). fact.rs:35 `FactType::Preference` (threshold 0.5, fact.rs:81). mem0_dispatcher.rs:144 `dispatch_one` (ADD/UPDATE-via-supersede/DELETE-via-soft_close/NOOP) WIRE-DARK (test-only callers). retrieval.rs:46 `get_relevant_facts` WIRE-DARK (every caller a test) -- superseded rows STRUCTURALLY excluded at SQL, never narrated. context_budget.rs:1066 `ContextSlot` has 8 slots, NO memory slot (slot 2 is `PhaseSpec`, not "Telescope"). conversation.rs has ZERO memory references. Live SCHEMA_VERSION=220 (ssb.rs:2990), TOTAL_VARIANTS=497 (action_type_lock.rs:624).
- REVISED scope (W-10): SHIPS NOW -- explicit-only capture (/remember + confirmed propose, NO silent auto-write per Cursor lesson); write-time supersession via existing dispatcher UPDATE path (older.valid_to_ms==newer.valid_from_ms); on-demand retrieval via `get_relevant_facts` with the hard `valid_to_ms IS NULL` filter (avoids Graphiti +68% narrate-stale trap), HALO decay backstop; ~100-500 token Memory slot (never always-injected); HARD-EXCLUDE secrets/PII/file-contents at capture; high-stakes-flip (HARD RULE/dep/routing) Y/N-gated via InteractiveGate else auto-supersede; AUDITOR-OBJECTIVITY FIREWALL (auditor/critic/adversary get ZERO memory injection, Personalization-Trap arXiv:2510.09905); default-OFF `ELAI_USER_MEMORY_ENABLE` + allowlist row + frozen smoke. DEFERRED to H-32: DKG/rerank ranking, scope promotion, periodic tidy pass + >50%-deletion guard armed.
- Integration: `/remember` handler -> new crates/cli/src/user_memory_capture.rs -> `dispatch_one`; add `ContextSlot::Memory` (context_budget.rs:1066) + `get_relevant_facts` call in conversation.rs (role+flag guarded); new Pattern-2 single-variant `UserDecisionSuperseded` (3-arm UserDecisionEventKind Superseded/Captured/ProposalSurfaced), SCHEMA 220->221 + TOTAL_VARIANTS 497->498 in lockstep; payload carries {old_fact_id,new_fact_id,fact_type,key} NOT value (no secrets in journal); frontend_pair MemoryFactsDrawer.svelte:83; wire-dark-allowlist.toml [[flag]] row (promotion_plan_id=H-31, smoke superseded_fact_never_reaches_assembled_prompt).
- observable_effect: state X via /remember then contradicting Y -> older row `valid_to_ms == newer.valid_from_ms` (queried from live FactStore, not the producer return) + a `UserDecisionSuperseded` SSB row in-window; the FROZEN auditor-authored smoke (h31_user_memory_contract.rs) drives the REAL retrieval+prompt-assembly path with ELAI_USER_MEMORY_ENABLE=1 and asserts X is ABSENT from the assembled context window AND an auditor-role assembly contains ZERO user_decision/memory rows; missing ssb.db / zero rows = FAIL (R1/R2), never SKIP.

### H-32 -- Tier A code-grounded fact memory (validate-citation-before-use)
- Track 4 (UX + USER-MEMORY) | lever TRUST | grounding STRONG | nature BUILD (on DKG) | effort M | WAVE 4 | depends H-31
- Premise: a durable code claim ("repo bans .unwrap() in prod", "swiftide rejected") must still be TRUE about current code; a fact citing a renamed/deleted symbol must re-validate before use, else surface a confirmation + an off-agent SSB row instead of silently steering the model.
- Steal/build: REIMPLEMENT (Rust, not port) odysseus teacher_escalation.py:133 _UNTRUSTED_TRACE_GUARD + :621-626 NO_SKILL distill gate (validate-before-persist -> ELAI inverts to validate-before-USE) and skill_extractor.py:66/:57 (write-path classifier + dedup). GitHub Copilot agentic memory (github.blog 2026, MEASURED +7% PR-merge, 28d TTL, "only validated facts used") is the production proof of the two-axis design.
- Existing ground truth (code-read, 2026-06-04): CITATION CARRIER ALREADY EXISTS = MemoryFact.evidence_pointers (fact.rs:177); retrieval already HARD-EXCLUDES empty-evidence rows (retrieval.rs:97). get_relevant_facts (retrieval.rs:46) is WIRE-DARK (tests-only callers, elai.map:281701-281705) -> H-31 makes it prod, H-32 gates it. Tier-A maps to FactType DomainKnowledge/Mistake (fact.rs:36-37/41-43); Tier-B = preference (H-31). DKG re-resolution surface EXISTS: ensure_name_index (cache.rs:276), find_symbol_at (detect.rs:104), node_by_id (dkg/mod.rs:818), SymbolNode (types.rs:74). GAP: RECENCY (last_observed_at_ms) and VALIDITY are conflated; no re-validation call; no fact_validation_failed beacon; the Mem0 write classifier (mem0_dispatcher.rs:144) is wire-dark.
- REVISED scope (W-10): Tier-1 ships add last_validated_at_ms (validity axis != recency), revalidate_citation (fail-CLOSED on Unknown), write-path classify_fact_tier, read-side validity gate on get_relevant_facts (Stale -> exclude + emit SSB + confirm), unused-28d-TTL soft_close with reset-on-validated-use. DEFER ONNX fuzzy citation match + Scope::Global promotion. Staleness handled at READ (excluded, NOT narrated [OUTDATED] = Graphiti trap) + WRITE (soft_close/supersede, never DELETE). Auditor/critic get ZERO Tier-A injection (Personalization-Trap arXiv:2510.09905); secrets/PII hard-excluded; citation is a symbol POINTER only.
- Integration: new crates/memory/src/citation_validity.rs; extend fact.rs:138 MemoryFact + retrieval.rs:46/:17; consume DKG read-only; maintenance pass on the live tick live_cli.rs:1991-2126; read gate rides H-31's ContextSlot memory slot + conversation.rs call. NEW SSB action_type fact_validation_failed (Pattern-2), SCHEMA 220->221 + TOTAL_VARIANTS 497->498 in lockstep (ssb.rs:2990 + action_type_lock.rs:624, confirm live const). Prefer NO new flag (fold into H-31's gate); if ELAI_FACT_VALIDATION_ENABLE added it needs a wire-dark-allowlist row + ON-path smoke (W-2/R9). frontend_pair: per-fact validity badge on MemoryFactsDrawer.svelte (commands_e5.rs:136).
- observable_effect: store a Tier-A fact citing a real symbol, rename/delete it + rebuild DKG index, drive a real model-dispatch turn (H-31 path) -> a fact_validation_failed SSB row (off-agent) is appended and the fact is ABSENT from the injected slice + the model gets a confirmation instead of acting; a still-valid citation injects + stamps last_validated_at_ms (resets TTL); an unread Tier-A fact past 28d is soft-closed (never deleted).

### H-33 -- memory hub, governance, and provenance surface
- Track 4 (UX + USER-MEMORY) | lever STEERING/TRUST | grounding STRONG | nature ACTIVATION | effort M | WAVE 4 | depends H-31
- Premise: the user can see/edit/forget/pin every stored decision, see which stored decisions will influence a run BEFORE it acts, and on /continue get a resume-summary that flags SUPERSEDED decisions (defuses intent-drift). Strictly "your saved decisions (editable)", never "remembers you" (personal_agent_risk: low) -- reads the SSB journal + FactStore directly, ZERO tokens injected into any model prompt.
- Steal/build: ACTIVATION-dominant. The `elai memory` CLI verbs, bi-temporal supersession, soft_close, decay scheduler, and the F-12 read-only memory drawer ALL EXIST. Light up the parsed-but-unrouted dispatch, add the one missing verb (`pin`), add two pure-read surfaces (provenance + resume-summary). odysseus MIT prior-art for context-budget math (context_compactor.py:174/263, context_budget.py:21) -- reimplement headroom-clamp in Rust, drop 200K max.
- Existing ground truth (code-read): CLI verbs parsed (args_memory.rs:10, enum args.rs:1310 List/Inspect/Insert/Delete) but main.rs:1019-1024 dispatch is an EXPLICIT STUB ("handler being wired in s-31708; not yet routed") -- THE activation gap. store.rs:303 soft_close, store.rs:147 list_current, store.rs:128 get; fact.rs:179 superseded_by + fact.rs:167 valid_to_ms give history; MemoryFact has NO pinned/decay-exempt column. decay.rs:88 decay_tick. F-12 drawer MemoryFactsDrawer.svelte:83 -> get_memory_facts commands_e5.rs:136 -> list_current; F-12 semantic-contradiction viewer-read commands_e5.rs:194-195. Live SCHEMA 220 (ssb.rs:2990) / TOTAL_VARIANTS 497 (action_type_lock.rs:624).
- REVISED scope (W-10): SHIP = (1) route the main.rs:1019 stub into a new memory_cmd.rs driver (list/inspect/forget/insert against existing store primitives); (2) add pin/unpin WITHOUT a sqlite migration -- append-only MemoryGovernanceEvent beacon, decay scheduler hard-skips pinned fact_ids (fail-toward-decay if beacon read fails); (3) provenance "which prefs fired" read surface (trust-floor predicate recomputed read-only, zero tokens); (4) /continue resume-summary flagging superseded rows. DEFER = standalone MemoryHub panel beyond extending F-12 drawer; vector-ranked provenance.
- Integration: de-stub main.rs:1019; new crates/cli/src/memory_cmd.rs; extend MemorySubcommandKind (args.rs:1310) + parser (args_memory.rs) with Pin/Unpin/Provenance; decay_scheduler pin-filter; extend F-12 MemoryFactsDrawer.svelte + new Tauri commands memory_forget/memory_pin/get_memory_facts_with_history. SSB: ONE new Pattern-2 single-variant MemoryGovernanceEvent (3-arm Forgotten/Pinned/Unpinned), SCHEMA 220->221, TOTAL_VARIANTS 497->498 in lockstep (confirm live const at authoring). Decay pin-FILTER behind ELAI_MEMORY_PIN_ENABLE (default-OFF) + wire-dark-allowlist row (type=experiment, expiry +90d, promotion_plan=H-33, smoke=pin-on-path); reads ship default-ON. Auditor-firewall: provenance/resume-summary NEVER injected into auditor/critic prompts (arXiv:2510.09905).
- observable_effect: `elai memory list` prints active+superseded decisions FROM THE JOURNAL with zero tokens injected into any model prompt; `elai memory forget <id>` sets valid_to_ms non-NULL AND lands a MemoryGovernanceEvent::Forgotten SSB row the agent did not author (missing row = FAIL, R1); `elai memory pin <id>` + a decay tick with the flag ON leaves the pinned row's trust_score unchanged while a control row decays (R9 ON-path smoke); on /continue the resume-summary lists influencing decisions and flags superseded ones before the run acts. Done-gate = wiring_contract_check --run-trigger PASS, not cargo-green.

### H-34 -- MCP defer-loading + Tool-Search emulation for the live STDIO MCP servers
- Track 4 (UX) | lever CHEAPER | grounding STRONG | nature BUILD | effort M | WAVE 4 | depends none (composes C-35 + distinct from H-4)
- Premise: the live STDIO MCP lane (McpServerManager) discovers EVERY server tool's full inputSchema eagerly and carries them per turn -- the largest fixed tax on the 25-32K local window. FIRST measure the per-turn MCP-schema token baseline, THEN defer-load so only an always-floor + RAG-selected tools carry full schemas; the rest resolve on demand. Delta is MEASURED + recomputable from the SSB journal.
- Steal/build: ACTIVATE C-35 LazySchemaCache (crates/plugins/src/mcp/lazy.rs:30, resolve at :50, hit/miss counters :96-98) + resolve_schema_on_demand (mcp_bridge.rs:959) -- they exist but hang off the WIRE-DARK HTTP bridge (mcp_client.rs:26-38, zero non-test callers), not the live STDIO lane. Reimplement odysseus tool_index.py union assembler in Rust (get_tools_for_query :431 = always-floor :435 UNION vector top-K :436-437 UNION keyword force-include :444-446; ALWAYS_AVAILABLE floor :24, small per :32) against ELAI's OWN ONNX-vector stack -- MIT reference, not a Python port; blake3 not hashlib.
- Existing ground truth (code-read): live lane = STDIO McpServerManager (mcp_stdio.rs:484/493/498), discover_tools_best_effort (mcp_stdio.rs:559) wired at mcp_glue.rs:62/70, REPL re-export live_cli.rs:13; each McpTool carries input_schema verbatim (mcp_stdio.rs:124), tool_index insert loop has NO size/defer gate (mcp_stdio.rs:545-552). The 3 named servers are HARNESS config, never in ELAI Rust (server-agnostic from_runtime_config). Gap: no per-turn schema-token measurement, eager-all-schemas on the live lane, lazy cache bolted to wire-dark HTTP bridge, no deferred-load SSB action_type.
- REVISED scope (W-10): SHIP -- (1) MEASURE FIRST via deterministic offline mcp_schema_budget_runner (include_str! corpus seccodeplt_runner.rs:65, COMPUTED baseline tokens, seed ELAI_SEED determinism.rs:48, corpus_blake3 :166, no literals :23/:146-148); (2) defer-load behind ELAI_MCP_DEFER_SCHEMA_ENABLE (default-OFF, W-2 allowlist row) -- inject always-floor (<=6) UNION RAG-top-K UNION keyword, hold withheld schemas in Arc<LazySchemaCache>, resolve on first use; (3) AFTER runner computes injected tokens + delta to benchmarks/mcp_schema_budget/<ts>.json. DEFER: schema dedup, cross-server sharing, default-ON promotion. Always-injected slice capped <=~500 tokens (runner asserts). Staleness N/A (server-current).
- Integration: gate-guarded branch in discover_tools_best_effort (mcp_stdio.rs:559) + insert loop (:545-552); deferred-resolve at the live call_tool path (mcp_glue.rs); runner module crates/orchestrator/src/benchmarks/mcp_schema_budget_runner.rs + run_bench_mcp_schema_budget in bench_cmd.rs (alongside :139/:149). TWO new SSB action_types: mcp_tool_deferred (Pattern-2 single-variant MultiAgentSpan::McpToolDeferred, per deferred resolve) + mcp_schema_budget_measured (flat result beacon mirroring InsecureRateMeasured ssb.rs:8513/9510). SCHEMA bump 220->222, TOTAL_VARIANTS 497->499 in lockstep (live consts ssb.rs:2990 / action_type_lock.rs:624; CLAUDE.md 191/464 STALE). Gate needs wire-dark-allowlist.toml row {experiment, owner, expiry, promotion_plan, ON-path smoke}.
- observable_effect: wiring_contract_check --run-trigger PASS -- elai bench mcp-schema-budget computes baseline tokens (off-agent journal number); a real --via-mcp task with ELAI_MCP_DEFER_SCHEMA_ENABLE=1 names a withheld tool -> mcp_tool_deferred SSB row lands + runner reports injected < baseline with floor present and slice under cap. Missing row OR non-positive delta = FAIL (R1). Distinct from H-4 (built-in catalog, production_agent_rpc.rs:652/:1221) -- H-34 targets external MCP-server schemas.

### H-35 -- /rewind over the SSB journal
- Track 4 (UX + USER-MEMORY) | lever STEERING | grounding STRONG | nature BUILD-UI-over-journal | effort M | WAVE 4 | depends H-33, composes G-2
- Steal/build: STEAL Claude Code's rewind affordance + conversation/code/both scoping (MEASURED most-loved feature, 46% vs Cursor 19%, Pragmatic Engineer 2026). BUILD it over ELAI's own seeded append-only SSB journal -- strictly stronger than a shadow-git because it REPLAYS deterministically under ELAI_SEED. Reimplement (not port) odysseus context_compactor.py:174 trim_for_context (summarize-from-here) + memory_extractor.py:32 _fingerprint_entries (blake3 idempotence short-circuit).
- Existing ground truth (code-read): ALL restore primitives PROD-WIRED, only the affordance is missing. Journal ASC-replayable ssb.rs:15353 query_session_stream_asc (SCHEMA 220 ssb.rs:2990 / TV 497 action_type_lock.rs:624; 191/464 STALE). Replay-resume observability/replay.rs:84 ReplaySessionId::derive + ssb_resume_from (fail-closed LockedKeysAuditor) + ReplayOutcome replay.rs:99, emits ReplayResume ssb.rs:23345/23562; Tauri commands_e29.rs:221 ssb_resume_from. Stub rehydrate session_dag.rs:1507 recover_stubs (blake3 verify :1543; NO replay_resume FN here -- brief wrong). RRA proof rra.rs:384 build_rra/:474 verify_rra, seal run_subcommand.rs:449-459. Summarize compact.rs:96 compact_session. CLI args.rs:427/437/1363/1372/3621/3652 + main.rs:488/493 rra_cmd. Frontend CommandWindow.svelte/CanvasWindow.svelte/RraViewerPanel.svelte.
- REVISED scope (W-10): SHIP elai rewind <target> [--scope conversation|code|both] [--summarize] driving compact_session (conv trim) + recover_stubs + ssb_resume_from (code restore, fail-closed); one session_rewound beacon; idempotence guard (target==head = NO-OP, zero rows). DEFER branch-fork, multi-checkpoint diff, cloud-model VerifyDivergence restore. /rewind injects NOTHING into prompt; summarize SHRINKS window; append-only so no superseded rows to narrate.
- Integration: new rewind_cmd.rs + CliAction::Rewind + RewindScope enum + parse_rewind_subcommand_args; frontend_pair CommandWindow.svelte + new commands_h35.rs rewind_session Tauri cmd. ONE new SSB variant SessionRewound (Pattern-2, H-11 CanvasActivityEvent precedent): SCHEMA 220->221, TV 497->498 lockstep (confirm live const at authoring). DEFAULT-ON kill-switch ELAI_REWIND_DISABLE (NOT default-OFF graveyard; no allowlist row needed).
- observable_effect: elai rewind <seq> --scope both on a seeded fixture under fixed ELAI_SEED lands exactly one off-agent session_rewound row (target_seq/head_seq_before match trigger) AND re-build_rra + verify_rra over the post-rewind stream returns Verified. wiring_contract_check --run-trigger PASS needs row present AND verdict Verified; missing row or non-Verified = FAIL (R1/R2). No-op rewind = ZERO rows (manifest distinguishes from dead path).

### H-36 -- completion / needs-input dev-event notifications
- Track 4 (UX + USER-MEMORY) | lever STEERING | grounding MODERATE | nature BUILD | effort S | WAVE 4 | depends H-35
- Premise: a dev who fires a long autonomous run walks away; nothing tells them when it FINISHED or is BLOCKED on a human decision. Ships a terminal BEL/OSC9 escape + an OS notification driven FROM a real SSB row, on BOTH completion and needs-input.
- Steal/build: BUILD. FSM/RequestHuman/PhaseComplete/write_loop/NotificationManager all PROD-WIRED, but no code converts a terminal event into a terminal-escape or OS notification, and no action_type records that one fired. Prior art: codeagentswarm OSC9+BEL+native-OS dispatch matrix (reimplement in Rust). REJECT engagement/vacation-mode/learned-threshold framing (personal_agent_risk: companion drift + would need a stale-pref store) -- declarative DEV-EVENT alerts only, single binary opt-out gate.
- Existing ground truth (code-read): completion = task_runner.rs:9610 PhaseComplete (fsm_state="complete" :9613, siblings :2478/:4759); needs-input = task_runner.rs:1157 AutonomyDecision::RequestHuman -> emit_autonomous_decision :1162 (NO INTERVENTION_REQUIRED FSM state). SSB write_loop ssb.rs:15146, SCHEMA_VERSION=220 (ssb.rs:2990), TOTAL_VARIANTS=497 (action_type_lock.rs:624). Hook seam hook_dispatcher.rs:1146 dispatch_stop. Dashboard NotificationManager.svelte:102-110 fires completion toasts; :114-118 intervention is an explicit no-op (B-36 gap) = the dashboard free-win.
- REVISED scope: SHIP a NotificationDispatcher (OSC9 + BEL fallback + std::process::Command OS-notify, no new crate) wired onto dispatch_stop (completion) and the RequestHuman site (needs-input), emit agent_notification_emitted SSB row, activate the dashboard no-op by reading needs_input rows. New action_type bumps SCHEMA 220->221 / TOTAL_VARIANTS 497->498. ELAI_NOTIFY_ENABLE default-OFF kill_switch (terminal escape + SSB row fire unconditionally) requires a wire-dark-allowlist.toml row + ON-path smoke test (R9). Zero local-token cost; no staleness/decay (instantaneous fire-and-forget).
- Integration: new crates/orchestrator/src/notify_dispatch.rs; triggers at hook_dispatcher.rs:1146 + task_runner.rs:1162; frontend_pair NotificationManager.svelte:114-118 + a new Tauri command in commands_e5-style module registered in dashboard-v2/src-tauri/src/lib.rs (cross-lang pair gate).
- observable_effect: a long `elai run` finishing OR hitting the fix_and_rerun cap (needs-input) emits an agent_notification_emitted SSB row the feature agent did not author, written by the live notifier reached from the real production stop-hook / RequestHuman path; done-gate is wiring_contract_check --run-trigger PASS, missing rows = FAIL (R1).

### H-37 -- plan-approval gate at PLANNING -> TESTING_FIRST
- Track 4 (UX + USER-MEMORY) | lever STEERING | grounding STRONG | nature BUILD-on-existing-pre-gate | effort S-M | WAVE 4 | depends H-36 (soft, notification reuse)
- Premise: catch a wrong plan assumption at one FSM transition, before it cascades into 30 edits. Adds an explicit approve/redirect/reject decision riding the EXISTING compliance pre-gate edge; industry-convergent (Claude Code plan-mode, Devin Interactive Planning).
- Steal/build: BUILD-on-existing, three live seams composed. (1) gate.rs::run_interactive_gate (gate.rs:230) — full APPROVE/REJECT/MODIFY/REDIRECT protocol, WIRE-DARK (callers only #[cfg(test)] gate.rs:733-956). (2) CliInteractiveGate (runtime_builder.rs:429) already installed onto RuntimeWiring.interactive_gate (run_subcommand.rs:118-126) when gate_mode==Interactive. (3) AutonomousGate + GateMode (autonomous_gate.rs:88) for the B-22 LOW-auto / MEDIUM+-surface path. odysseus prior-art teacher_escalation.py:621-626 (NO_SKILL distill gate) informs the fail-closed REJECT/REDIRECT invariant — reimplement in Rust.
- Existing ground truth (code-read): compliance pre-gate PROD machinery at fsm.rs:925-928 but compliance_provider defaults None (fsm.rs:570) = runtime-dark at install (no CLI install site found). run_interactive_gate consumed only in tests; task_runner holds the gate as a presence signal only (`let _ = autonomous_gate.as_ref();` task_runner.rs:1087, "reserved for follow-on wiring" :1084) = THE activation gap. Existing emit_autonomous_decision (task_runner.rs:14164) maps onto SsbPayload::GateDecision(GateDecisionPayload) action_type "gate_decision" (ssb.rs:8809) — the prior-art payload shape. Live SCHEMA_VERSION=220 (ssb.rs:2990), TOTAL_VARIANTS=497 (action_type_lock.rs:624); CLAUDE.md 191/464 STALE.
- REVISED scope (W-10): ships the live plan-approval decision at PLANNING/FEASIBILITY_PROBE->TESTING_FIRST; INTERACTIVE mode prompts via the installed CliInteractiveGate, AUTONOMOUS routes through AutonomousGate::decide (LOW auto, MEDIUM/HIGH surface). REJECT refuses the transition (FSM stays PLANNING), REDIRECT validates target. Deferred: multi-round MODIFY, learn-from-rejection memory write, installing a real ComplianceProvider. Local-token: only the EvidenceSummary surfaces, no always-injected block, ZERO user-memory in the gate (auditor-objectivity firewall).
- Integration: decision computed in orchestrator at the fsm.rs:925-928 edge (reachable from real FSM transition, not a CLI shim); risk via default_risk_level(Planning,TestingFirst)=MEDIUM (gate.rs:196). New SSB action_type plan_decision_event (Pattern-2 single-variant, PlanDecisionKind discriminator Approve/Redirect/Reject/AutoAdvance), payload mirrors GateDecisionPayload; SCHEMA 220+1=221 / TOTAL_VARIANTS 497+1=498 at action_type_lock.rs:624 in lockstep (confirm live const at authoring). default-OFF kill switch ELAI_PLAN_APPROVAL_GATE_ENABLE needs a wire-dark-allowlist.toml row (kill_switch / owner H-37 / expiry 2026-09-04 / promotion H-37 / ON-path smoke). frontend_pair: PlanApprovalCard.svelte (sibling to components/FeasibilityPanel.svelte).
- observable_effect: drive a real `elai run` to PLANNING->TESTING_FIRST with the gate enabled; assert a plan_decision_event SSB row landed (agent did not author it, queried back under ELAI_SEED) AND a REJECT leaves FSM at PLANNING (no TESTING_FIRST PhaseTransition follows). wiring_contract_check --run-trigger PASS; missing row = FAIL (R1).

### H-38 -- cost-confirmation gate on TaskStop / pod_stop
- Track 4 | lever SAFETY/STEERING | grounding MODERATE | nature BUILD | effort S | WAVE 4 | depends H-9; composes H-37 + orchestrate T1
- Premise: a stop today silently kills billable pods + unsaved cloud work (Claude Code #28875: ~$30 + ~1h lost). Surface cost-so-far + unsaved progress and require confirm before a billable/unsaved stop; pass a $0 saved stop with zero friction (FP guard).
- Steal/build: BUILD. Composes H-9 spend ledger + the in-runtime stop hook + the H-37 CliInteractiveGate present_gate + the orchestrate T1 tripwire (does NOT restate them, GD-1). odysseus reimplement-ref = teacher_escalation.py:621 NO_SKILL distill gate (validate-before-act posture); reimplement predicate in Rust, no Python port.
- Existing ground truth (code-read): spend ledger PROD-WIRED (cloud_mode.rs:646 SpendAccumulator, :521 CloudSpendSnapshot, :357 daily-cap; ssb_viewer.rs:5062 cloud_spend_summary day_usd; grounding_client.rs:848-905 POST-hoc cap precedent). Stop boundary hook_dispatcher.rs:1146 dispatch_stop has ZERO cost awareness (B-19 non-IDLE block only). H-37 CliInteractiveGate (gate.rs:126/129, runtime_builder.rs:429, installed run_subcommand.rs:120) is the confirm seam. CORRECTION: pod_stop/pod_cost_status are HARNESS MCP tools, NOT ELAI Rust (only a corpus literal at corpusgen.rs:71) — pod-cost half is a skill-doc concern. SCHEMA 220 (ssb.rs:2990) / TOTAL_VARIANTS 497 (action_type_lock.rs:624) live.
- REVISED scope: Tier-1 ships NEW stop_cost_gate.rs (pure evaluate_stop -> Pass | ConfirmRequired) wired into dispatch_stop reading day_usd from cloud_spend_summary; on ConfirmRequired reuse present_gate (Reject=hold, headless=fail-closed-hold); emit ONE StopConfirmationEvent Pattern-2 beacon (3-arm: ConfirmedProceed/RejectedHeld/PassedNoFriction). Extend orchestrate T1 list for billable pod_stop. Defer RunPod billing reconcile + dashboard modal + auto-checkpoint. Cost figure read fresh (never stale, never injected into a prompt).
- Integration: stop_cost_gate.rs + lib.rs re-export + hook_dispatcher.rs:1146 wire + CloudPanel.svelte frontend_pair (no new command). New SsbActionType StopConfirmationEvent (ssb.rs:8753/9557/10265/13743/14451 + payload ~:27005), SCHEMA 220->221, TOTAL_VARIANTS 497->498 lockstep. Beacon default-OFF behind ELAI_STOP_COST_CONFIRM_ENABLE (W-2/R9 allowlist row: type=experiment, owner=H-38, expiry 2026-09-04, promotion H-38-promote + ON-path smoke); the Y/N safety behavior is ON unconditionally so it cannot fail-open.
- observable_effect (W-1 done-gate): a billable/unsaved stop surfaces cost-so-far + unsaved progress and requires confirmation AND a stop_confirmation_event SSB row lands with the cost figure (recomputable from the spend ledger via cloud_spend_summary); a non-billable saved stop proceeds frictionless + lands a PassedNoFriction row. wiring_contract_check --run-trigger PASS (real dispatch_stop ran, off-agent row appeared); missing row = FAIL (R1). NOT cargo-green.

## Track-4 cross-cutting (what makes it different)
- Every H-N done-gate = wiring_contract_check --run-trigger PASS (a real entrypoint left an
  off-agent SSB row), never cargo-green (W-1); missing evidence = FAIL (R1); the broken arm screams (R2).
- Memory is RETRIEVAL-ON-DEMAND (~100-500 token slice), never an always-injected block; ~zero
  always-on cost on the 25-32K local window.
- Staleness fixed at WRITE time (soft_close, newest-wins, DELETE BANNED for SSB replay), never
  trusted to read-time model detection (STALE: best model ~55%, most frameworks <10%).
- Two-tier (the approved Option 6): Tier-B preferences (H-31, bi-temporal supersession) + Tier-A
  code-grounded facts (H-32, validate-citation-before-use on the DKG); a write-path classifier routes each.
- The 5 QOL free-wins (H-34..H-38) are host-side / journal-read = ~zero local-token cost; they
  surface capability ELAI already owns (MCP defer-load, /rewind, notifications, plan-gate, cost-confirm).
- KNOWN NITS for the executor: (1) re-verify the future-dated arXiv IDs above before any headline;
  (2) H-31 detail COMPOSES-WITH mischaracterizes H-30 as a "capture/intake substrate" -- H-30 is the
  METRIC; the live FactStore handle comes from the live_cli.rs:2034 tick, not from H-30; (3) reconcile
  the SHIP-ORDER-CUMULATIVE SCHEMA bump across H-30..H-38 + the stale e45 cross-lang drift variant count.
- Track 5 (the 6 odysseus steals + minor refinements, each referencing docs/.source/odysseus-main +
  docs/.source/odysseus-INDEX.md) is SEQUENCED AFTER Track 4 per the approved instrument-first ordering.

================================================================================
TRACK 6 -- GROUNDED-REASONING HARDENING (hallucination-limits-derived; H-39..H-42)
================================================================================
Generated 2026-06-04 from the Hallucination-Stations research thread: the YouTube explainer
(AIYQp1n51ZI "They Lied to You About AI", 376k views) + its source paper arXiv:2507.07505
("Hallucination Stations", Sikka & Sikka) ingested, then a 3-angle deep-research sweep over the
formal transformer-expressivity / hallucination-inevitability / verification-vs-generation
literature (2310.07923, 2402.12875, 2511.07378, 2503.01544, 2502.02393, 2509.04664, 2506.18203,
2402.08115, 2401.11817, 2502.12187 -- all id-verified; interface maps in workflow/papers/).
VERDICT (anti-sycophantic): ELAI's trust spine is ALREADY strongly aligned with the rigorous
2024-2026 literature -- D-10 select_grounding_mode already routes sound-checker-first
(grounding_mode.rs:157) and firewalls the auditor, D-11 checker_registry / D-16 conformal refusal /
D-17 refusal_sla / E-6 iso_budget_harness / E-19 self_certainty+micro_reflection are all SHIPPED.
Track 6 is FOUR small ACTIVATION/EXTENSION refinements + instruments on that shipped substrate, NOT
a redesign. The 4 grounded gaps + their fixes are in _grounding/H-REASONING-GROUNDING-MAP.md
(RG-39..RG-42 + the RD-1..RD-4 delta rules). The 3 inevitability papers (2509.04664 / 2401.11817 /
2502.12187) were evaluated for standalone plans and DECLINED: 2509.04664 GROUNDS H-39 (the exact
penalty=t/(1-t) abstention-grading formula), 2401.11817 REINFORCES the external-oracle/checker design
(its Table-3 complexity taxonomy is FOLDED INTO H-41), 2502.12187 REINFORCES GD-4 FP-bounding. None
warranted a separate file.

TRACK NUMBERING: Track 5 (the 6 odysseus steals) remains RESERVED + UNAUTHORED; Track 6's files take
H-39..H-42 and Track 7's take H-43..H-48, so when Track 5 is authored it starts at H-49 (collision-safe).
Track 6 inherits W-1..W-10 + GD-1/2/4/5 + HD-1..HD-5 and ADDS RD-1..RD-4 (canon: H-0_INDEX.md + H-REASONING-GROUNDING-MAP.md).

SCHEMA: LIVE baseline 220/497 (ssb.rs:2990 / action_type_lock.rs:624; 207/484 and CLAUDE.md 191/464
are STALE). Track-6 deltas: H-39 +AbstentionScored, H-41 +CheckerGapDetected, H-42 +ReasoningBudgetScaled
(baseline+1 each, ship-order-cumulative); H-40 reuses the shipped iso_budget_measured row (NO new variant).
The FIRST Track-6 bump reconciles the stale e45_cross_language_drift_contract variant pin (~482).

### H-39 -- abstention-aware promotion + eval scoring [TRACK 6 KEYSTONE]
- Track 6 | lever PROVABLE/TRUST | grounding STRONG | nature BUILD | effort M | WAVE 6 | BLOCKING (the reward the loop climbs)
- Gap (RG-39): the OPTIMIZED reward (promotion_gate::two_gate_promote / bai::sample / coding_metrics pass_at_k / the
  bench scorers) is ABSTENTION-BLIND; the abstain primitive (conformal RefusalDecision, D-16) + the refusal SLA (D-17)
  exist but are unfed -- so the self-mod loop (darwin_admit_loop) + BAI + GEPA SELECT FOR confident guessing while
  D-16/D-17 try to suppress it at inference = a cross-subsystem MISALIGNMENT. Grounded in OpenAI 2509.04664 Observation 1.
- Steal/build: one AbstentionAwareScore (correct=+1, abstain=0, confident-wrong=-t/(1-t) with t LOCKED, default 0.75->2x
  per 2509.04664), reading the abstain bit from the shipped RefusalDecision (GD-1); feed it into the promotion reward +
  the coding_metrics accumulator + the bench scorers (safedelta/seccodeplt pattern); emit +AbstentionScored SSB.
- observable_effect: a frozen eval fixture with a confident-wrong V1 + a correct-abstaining V2 -> score(V2)>score(V1) AND
  the promotion path ranks V2 above V1; an AbstentionScored row recomputable under ELAI_SEED; the --reward-confident-guessing
  broken arm flips it (R2). Composes D-16/D-17/D-5/E-49/G-6.

### H-40 -- decorrelation-keyed verifier/role retention
- Track 6 | lever TRUST | grounding STRONG | nature EXTENSION (E-6 iso_budget_harness) | effort S-M | WAVE 6
- Gap (RG-40): iso_budget_harness flags a role by MARGINAL LIFT (insufficient_lift:93); Weaver 2506.18203 proves a
  decorrelated below-accuracy verifier still raises an ensemble -- so a lift-only deletion gate WRONGLY prunes a valuable
  decorrelated verifier/auditor. Add a conditional-error-DECORRELATION (unique-catch) axis; prune only on both-low.
- Steal/build: per-role unique_catch_rate + decorrelation_score on IsoBudgetReport; insufficient_contribution =
  low_lift AND low_unique_catch; reuse the shipped iso_budget_measured row (NO new variant, GD-1).
- observable_effect: a fixture where role R is low-lift/high-unique-catch and control C is low/low -> R RETAINED, C prunable;
  the --prune-on-lift-only broken arm prunes both (R2). Honest ceiling = the Pass@K oracle (RD-2/GD-5).

### H-41 -- live checker-coverage + checker-gap INSTRUMENT
- Track 6 | lever PROVABLE | grounding STRONG | nature BUILD (instrument over shipped D-10/D-11) | effort M | WAVE 6
- Gap (RG-41): the sound-checker-FIRST preference is ALREADY shipped (grounding_mode.rs:157) and D-21 has an OFFLINE F1
  benchmark, but there is NO LIVE signal for the Mode-A (Constructive) fraction of real claims nor a ranked CHECKER-GAP
  backlog naming claim-classes that fell to Mode B/refusal for LACK of a checker. Per Kambhampati 2402.08115, maximizing
  Mode-A coverage is THE lever; the Xu 2401.11817 Table-3 taxonomy (NP-complete/undecidable -> solver-or-refuse, never
  LLM-grade) ranks the backlog. (NOTE: H-41 does NOT add the A>B preference -- it is shipped; it INSTRUMENTS it.)
- Steal/build: a live mode_a_coverage roll-up (over shipped grounding-mode + CheckerResultEmitted rows; no variant for the
  ratio) + a +CheckerGapDetected signal at the agent_pool.rs:2030 routing site, classified by an Xu-Table-3 complexity taxon;
  `elai checker-gaps` ranks the backlog by frequency x taxon-priority.
- observable_effect: a Factual claim WITH a checker routes Constructive + no gap row; one WITHOUT emits CheckerGapDetected
  {claim_class, complexity_taxon, fell_to_mode}; mode_a_fraction recomputable under ELAI_SEED; the --ignore-checker-gaps
  broken arm mis-reports (R2). Coverage = REACH to a sound checker, not its correctness (GD-5).

### H-42 -- serial-depth-scaled reasoning budget + the depth-not-width guard
- Track 6 | lever SMARTER-ORCH/TRUST | grounding STRONG | nature EXTENSION (Phase-2C GRC + E-19) | effort M | WAVE 6
- Gap (RG-42): GRC budget is per-ROLE STATIC (agent_grc_config_for_role); the CoT-step lower bounds (2503.01544 ~O(n);
  2502.02393 tight) say required SERIAL steps scale with a sub-task's serial DEPTH, and the single pass is ~TC0
  (2310.07923) so context buys ACCESS not compute-DEPTH. A flat budget under-budgets a deep serial sub-task and FORCES
  truncation; a context lift is the wrong fix.
- Steal/build: estimate_serial_depth (decomposition-tree depth / DKG reachability-chain length) -> scale_grc_budget(base, d)
  bounded by a LOCKED mutable=0 ceiling (floor = the base per-role config) -> classify_failure DepthBound vs AccessBound
  (the depth-not-width guard) -> +ReasoningBudgetScaled SSB. Formally grounds the "smarter orchestration not model" thesis (H-7).
- observable_effect: a deep serial sub-task gets a larger budget than a shallow one (same role, both <= ceiling); a high-depth
  exhaustion emits failure_class=DepthBound (remedy=steps, not context); the --flat-budget broken arm equalizes them (R2).

## Track-6 cross-cutting (what makes it different)
- Every H-N done-gate = wiring_contract_check --run-trigger PASS (a real entrypoint left an off-agent SSB row), never
  cargo-green (W-1); the broken arm screams (R2 -- an instrument/metric that stays right when broken is itself a FAIL).
- The finding is MOSTLY CONFIRMATION: the rigorous literature validates ELAI's shipped trust spine (off-agent sound
  checkers, cross-family decorrelated verification, conformal refusal, orchestration-as-serial-compute). Track 6 is four
  small refinements, NOT a redesign -- the anti-sycophantic read.
- RD-1 abstention-credited reward; RD-2 decorrelation-over-accuracy (oracle-ceiling honest); RD-3 sound-checker-first +
  coverage-is-the-lever; RD-4 depth-not-width. All four encode a specific peer-reviewed/formal-proof result against a
  specific shipped ELAI symbol (see H-REASONING-GROUNDING-MAP.md RG/RS tables).
- Positioning: 2507.07505's "verification is as hard as the task" pessimism is true ONLY for a CORRELATED verifier;
  ELAI's cross-family + off-agent + sound-checker design is the engineering refutation -- the cant-lie wedge.

================================================================================
TRACK 7 -- DGM SELF-IMPROVEMENT VERIFICATION HARDENING (self-verification-dossier-derived; H-43..H-48)
================================================================================
Generated 2026-06-05 from the self-verification dossier (workflow/reasoning/ELAI-SELF-VERIFICATION-RESEARCH-DESIGN-
2026-06-05.md -- 6 background research agents, every arXiv id + GitHub repo WebFetch-verified by the agent because the
research is ABOUT not building on hallucinations; DGM code read directly: darwin_admit_loop.rs, config_archive.rs,
promotion_gate.rs, trap_tasks.rs, darwinian_bai.rs) + the E-19 SE-Agent L4 relocation. The 3 keystone cites (CodeRM
2501.01054, ImpossibleBench 2510.20270, HGM 2510.21614) + the 2 newest (2604.15149, 2605.02964) were re-confirmed via
WebSearch this session (title + repo match). META-FINDING (the theorem the track rotates around): A RECURSIVE SELF-
IMPROVEMENT LOOP WITHOUT A REAL, UN-GAMEABLE, STRONGER-THAN-GENERATOR VERIFICATION FILTER PROVABLY COLLAPSES; WITH ONE,
IT PROVABLY CONVERGES (Self-Verification-Prevents-Collapse NeurIPS X5Hk8aMs6w + Solver-Verifier Gap 2507.00075 + Skalse
2209.13085 + Co-rewarding 2508.00410 40-pt self-judge collapse). ELAI's DGM-as-shipped is the DIVERGENT branch: its
filter is TWO placeholders grep-confirmed LIVE -- derived_alpha = blake3(config_bytes) (GAP-1, P0) + score_candidate_for_
traps = keyword grep the code self-flags "unsound" (GAP-2, P0) -- plus an UNCONFIRMED tau (GAP-3), a scaffold bandit on a
placeholder reward (GAP-4), and eval-trust == the placeholder (GAP-5). Track 7 replaces the FILTER. THE CENTRAL FINDING
(anti-sycophantic): most of the dossier's PART D is ALREADY PLANNED (D-2 PPM, D-5/D-29 BAI, D-6 held-out store, D-13
adversary, E-6 multi-agent, E-53 Verus proof gate, E-54 corpus) -- so Track 7 is replacing TWO PLACEHOLDERS + a few
compositional upgrades, the genuine net-new being the execution-grounded fitness + the behavioral detector. The gaps +
fixes + symbol anchors are in _grounding/H-VERIFICATION-GROUNDING-MAP.md (DG-1..DG-7 + PC-1..PC-8 + VD-1..VD-5).

Track 7 inherits W-1..W-10 + GD-1/2/4/5 and ADDS VD-1..VD-5 (fitness-is-execution-grounded-not-bytes / detector-is-
behavioral-with-measured-recall / score-worst-case-not-mean+lineage-diverse / stronger-than-generator-verifier /
honest-undecidable-residue). E-53 (Verus/Kani) is COMPOSE-NOT-AUTHOR -- the narrow CSQ-Critical proof ceiling (VERINA
4.9% / CLEVER 0.6% = never routine, VD-5); SE-Agent recombination stays E-19 Task 11 (H-43's off-agent fitness makes it
runnable). All H-N sit behind the existing cfg(feature="darwinian-evolve")+ELAI_DARWINIAN_LOOP_ENABLE kill-switch
(allowlisted, W-2). W-10-tiered: a deterministic frozen-fixture done-gate now; the live multi-generation bench/regret run
a separately-gated checkbox (RUNPOD-PENDING per the dossier PART F -- 1+2+5 are MEANINGFUL only once the Tier-2 bench exists).

SCHEMA: LIVE baseline 224/501 (ssb.rs:3010 / action_type_lock.rs:645; 207/484 and 220/497 are EARLIER snapshots, now
STALE). Track-7 deltas: H-43 +FitnessEvaluated, H-44 +TrapBehaviorDetected, H-45 +PromotionScoreAggregated, H-46
+CladeSelected, H-47 +EvalFirewallEvent (its budget leg REUSES H-19's HoldoutBudgetEvent -- no new variant), H-48 reuses
D-5 telemetry (+BanditArmSampled only if no emitter). baseline+1 each, ship-order-cumulative; the FIRST Track-7 bump
reconciles the stale e45_cross_language_drift_contract pin (~482). Net +5 if H-43..H-47 ship (224->229, 501->506).

### H-43 -- execution-grounded DGM fitness [TRACK 7 KEYSTONE]
- Track 7 | lever TRUST/PROVABLE | grounding STRONG | nature BUILD | effort M-L | WAVE 7 | BLOCKING (the signal the whole loop climbs)
- Gap (GAP-1, DG-1): derived_alpha (darwin_admit_loop.rs:186) returns blake3(config_blob)->[0,1] -- a function of candidate
  BYTES, not PERFORMANCE; a failing-code config can score 0.99. The divergent branch (Skalse unhackable-proxy).
- Steal/build: ExecutionGroundedFitness = CodeRM pass-rate over a FROZEN generated-test set (2501.01054) + CodeT dual-
  agreement (2207.10397) with LOCKED weights; derived_alpha calls it (keep force_alpha + signature); cross-family guard on
  any model-graded blend (VD-4); +FitnessEvaluated SSB (ExecutionPassRate/DualAgreement/FitnessRegression).
- observable_effect: a frozen fixture with C_good (passes generated tests) + C_bad (fails) -> fitness(C_good)>fitness(C_bad),
  C_bad carries FitnessRegression, promotion ranks C_good above C_bad; the --fitness-from-bytes broken arm inverts it (R2).
  Composes D-2 (PPM upgrade path) / E-19 PRM-lite / E-53 (proof ceiling); enables E-19 Task 11 + H-39's "correct" bit.

### H-44 -- behavioral trap detection + measured recall
- Track 7 | lever TRUST | grounding STRONG | nature BUILD | effort M-L | WAVE 7 | parallel with H-43
- Gap (GAP-2, DG-2): score_candidate_for_traps (trap_tasks.rs:123) is substring grep over keyword arrays (:139/:143) the
  code self-flags "unsound"; a keyword-free verifier-gaming candidate passes untouched.
- Steal/build: behavioral detect_trap_behavior = ImpossibleBench spec-test conflict canary (2510.20270; "STOP if tests
  flawed" 93%->1%) + Isomorphic-Perturbation re-verify (2604.15149) + CodeHalu execution-classification (2405.00253, reusing
  H-43's harness); cross-view gate (>=2 different-family views, Co-rewarding); MEASURE recall on the frozen TRACE corpus
  (2601.20103); the keyword grep survives as a KeywordFallback advisory only; +TrapBehaviorDetected SSB.
- observable_effect: a keyword-free hack H1 is flagged + BLOCKED, a clean C1 passes, recall reported on TRACE recomputable
  under ELAI_SEED; the --trap-keyword-only broken arm misses H1 (R2). Composes D-24 canary / D-13 cross-family / E-54 awareness.

### H-45 -- worst-case + lineage-diverse promotion scoring + PRMBench tau
- Track 7 | lever TRUST | grounding STRONG | nature BUILD (extend two_gate_promote) | effort M | WAVE 7 | depends H-43
- Gap (GAP-3, DG-3): two_gate_promote admits on the candidate's OWN mean alpha vs tau, and tau is "UNCONFIRMED carries
  forward" -- the overoptimization regime (Gao 2210.10760); a single self-judge collapses 40pt (Co-rewarding).
- Steal/build: aggregate_promotion_score = min(verdicts)-kappa*var over LINEAGE-diverse views (Coste 2310.02743 WCO/UWO;
  Eisenstein 2312.09244 pretrain-lineage > seed); cross-view disagreement BLOCKS; tau CALIBRATED on PRMBench (2501.03124) +
  LOCKED with a date; wire the aggregate into two_gate_promote's margin (extend, not rewrite the gate -- GD-1); +PromotionScoreAggregated SSB.
- observable_effect: M (high mean, high cross-family variance) is BLOCKED, S (lower mean, low variance) admitted on the
  worst-case margin; the --score-own-mean broken arm flips it (R2). Composes D-5 / D-13; orthogonal to H-39's abstention dimension.

### H-46 -- HGM clade-metaproductivity parent selection
- Track 7 | lever SMARTER-ORCH | grounding STRONG | nature BUILD (extend config_archive select_parent) | effort M | WAVE 7 | depends H-43
- Gap (DG-4): config_archive selects the next parent by the IMMEDIATE sigmoid_perf_score (config_archive.rs:49/274); HGM
  2510.21614 proves the Metaproductivity-Performance Mismatch -- immediate score weakly predicts descendant productivity.
- Steal/build: clade_metaproductivity = the aggregated measured deltas of a node's descendant CLADE (HGM's CMP, on H-43's
  execution deltas) as the select_parent key; sigmoid retained for the no-lineage fallback (ImmediateScoreFallback); +CladeSelected SSB.
- observable_effect: node B (mediocre immediate, improving clade) selected over A (high immediate, regressing clade); a fresh
  leaf uses the fallback; the --select-immediate-score broken arm expands the dead-end A (R2). HGM 56.7 vs DGM 53.3 at ~2.38x less compute (PAPER's number, HD-1).

### H-47 -- DGM eval firewall (date-gate + template-regen + holdout-budget binding)
- Track 7 | lever PROVABLE | grounding STRONG | nature BUILD (compose D-6/E-54/H-19) | effort M | WAVE 7 | depends H-43
- Gap (GAP-5): even with a real fitness, the loop's eval-trust == the eval's trust; a self-improver firing thousands of
  score-queries IS adaptive analysis that overfits a static holdout / date-leaked items / re-used template instances / a
  recognizable eval it can sandbag.
- Steal/build: a DATE-GATE tying D-6 date-tags to the darwin generation clock (PreUpdateItemRejected; LiveCodeBench
  2403.07974) + GSM-Symbolic score-time template REGENERATION (2410.05229) + BINDING H-19's Thresholdout DP budget
  (1506.02629) to the loop's queries + an eval-awareness IndistinguishabilityProbe (2505.23836/2510.20487); +EvalFirewallEvent
  SSB (budget leg REUSES H-19's HoldoutBudgetEvent). COMPOSES D-6/E-54/H-19, restates none (GD-1 P0).
- observable_effect: a pre-update item P rejected, a post-update Q scored, a templated T regenerated fresh, the budget
  binds + refuses after Exhausted; the --eval-ignore-dates broken arm scores P + inflates the fitness (R2). Residue: a strong improver can still sandbag (VD-5).

### H-48 -- wire the BAI bandit on the real reward
- Track 7 | lever SMARTER-ORCH | grounding STRONG | nature ACTIVATION (compose D-5/D-29) | effort M | WAVE 7 | LAST | depends H-43 + H-45
- Gap (GAP-4, DG-5): the BAI structs ship (bai.rs:286 sample, bai_hierarchical D-29) but the darwinian_bai sample/update/run
  loop is SCAFFOLD on a placeholder reward -- it could not run, the reward WAS a byte-hash.
- Steal/build: ACTIVATE Discounted-TS (2305.10718, non-stationary) + Successive-Rejects (Audibert COLT 2010) selection on
  H-45's aggregated execution reward; seed the arm/guard-task curriculum from ExIt partial histories (2509.04575); REUSE
  D-5's telemetry (+BanditArmSampled only if no emitter). COMPOSES D-5/D-29, not a new bandit (GD-1).
- observable_effect: selection converges to the high-execution-fitness arm G over a fixed budget; the --bandit-reward-from-bytes
  broken arm converges to the failing arm B (R2). The live multi-arm regret number is RUNPOD-PENDING Tier-2.

## Track-7 cross-cutting (what makes it different)
- Every H-N done-gate = wiring_contract_check --run-trigger PASS (a real loop entrypoint left an off-agent SSB row over a
  frozen fixture), never cargo-green (W-1); the broken arm screams (R2) -- a fitness/detector/score that stays right when
  fed a byte-hash / a keyword-free hack / the candidate's own mean is itself a FAIL.
- The convergence theorem is the spine: the loop PROVABLY collapses without a real un-gameable stronger-than-generator
  filter and PROVABLY converges with one (Self-Verification-Prevents-Collapse + Solver-Verifier Gap). Track 7 BUILDS that filter.
- VD-1 execution-grounded-not-bytes; VD-2 behavioral-detector-with-measured-recall; VD-3 worst-case+lineage-diverse scoring;
  VD-4 stronger-than-generator (cross-family) verifier; VD-5 honest-undecidable-residue (proof gates rarely pass; sandbagging
  is mitigated not cured; loop stays dark/allowlisted -- SAFER, not a headline). Each encodes a peer-reviewed/proof result
  against a specific shipped DGM symbol (DG-1..DG-7).
- Anti-sycophantic read: ELAI's instincts are right across PART D (already-planned D/E gates); the work is replacing TWO
  placeholders (hash-fitness, keyword-traps) with measured code-backed mechanisms + a few compositional upgrades, NOT a redesign.

================================================================================
TRACK 8 — REVERSIBILITY / TIME-TRAVEL
================================================================================

### H-49 — reversible LLM edits: undo/redo/branching file time-travel + lossless session history
- Track 8 | lever STEERING/TRUST | grounding STRONG | nature BUILD-on-shipped | effort L | WAVE 8 | depends H-35 + E-25 + H-6
- Steal/build: every LLM file edit reversible AND re-appliable (redo) at ANY node, never losing a timeline, PLUS a lossless paged
  transcript so a long session's START is always reachable (the Claude-Code "can only see the tail" complaint). This is the
  follow-on H-35 EXPLICITLY DEFERS (branch-and-fork keeping both timelines + redo + cloud-model code-restore). Prior art
  WebSearch-verified 2026-06-05: Cursor (snapshot-before-edit, per-message restore, AI-edits-only FOOTGUN), Cline/Roo (shadow-git
  core.worktree, Restore-Files-Only, but DISABLED with no git repo + nested-git data loss), Aider (commit-per-edit + commits the
  user's work FIRST), undo-tree (branching history, never lose redo). The mechanism: a content-addressed (blake3 + zstd, dedup)
  object store WRAPPING the shipped-but-dark edit_checkpoints, an undo-tree HEAD over the SessionDag (undo=back, redo=forward-edge,
  undo-then-edit=fork), the RV-1 divergence guard (snapshot-before-restore so a user's manual edits are NEVER clobbered), the 3
  user-selectable grains (edit/turn/babystep), CAS-default + shadow-git-option behind a RestoreEngine trait, and the lossless paged
  transcript. Snapshot-restore (NOT replay) is load-bearing — model-agnostic, works on the production CLOUD lane.
- Existing state (grounded 2026-06-05 vs live source): the file-edit pre-image is ALREADY captured at file_ops.rs:272/300 then
  DISCARDED; edit_checkpoints.rs (snapshot_pre_edit/rewind_to, test-proven rewind_restores_file_content) is gated OFF + path-
  heuristic + full-copy; SessionDag is a real parent_id DAG (session_dag.rs:213); SSB FileWritePayload is {path,size,hash} with NO
  emitter (ssb.rs:10644); NO redo exists anywhere; TranscriptDrawer hardcaps ENTRY_LIMIT=50, handoff drops old messages, recover_stubs
  has zero prod callers. ~70% shipped, fragmented + wire-dark.
- COMPOSES-WITH: H-35 (delivers its 3 DEFERRED items; reuses SessionRewound + rewind_cmd), E-25 edit_checkpoints (wraps the engine,
  flips default-ON, retires the GB-eviction open-question), H-6 (shared recover_stubs activation), E-29 (paged ssb_transcript), C-23
  (BranchTree timeline), E-44 (saga compensation = the RV-4 irreversibility warnings), G-2 (RRA verify post-restore), F-13 (archive
  schema bump). Restates none (GD-1).
- Integration: new crates/orchestrator/src/architecture/timetravel.rs (CAS store + FileTree manifest + RestoreEngine trait + undo-tree
  HEAD + RV-1 guard); session_dag file_manifest+head columns (lazy ALTER); a capture hook at the FileWrite-row build site consuming
  file_ops.original_file; new crates/cli/src/timetravel_cmd.rs (undo/redo/history/jump) + commands_h49.rs (Tauri) +
  CommandWindow/BranchTree/TranscriptDrawer frontend_pair. SSB: reuse SessionRewound (H-35) for undo + ONE new Pattern-2
  EditHistoryEvent (SnapshotCaptured/Deduped/Redo/Branched/Jumped/TooLargeSkipped) for capture+navigation; baseline+1 from LIVE
  224/501 (re-read W-9). Default-ON + kill-switch ELAI_TIMETRAVEL_DISABLE (W-2).
- observable_effect: on a seeded fixture under ELAI_SEED, a real agent edit lands a SnapshotCaptured row (off-agent) + a blake3-
  verified pre-image blob; `elai undo --scope code` restores byte-exact (on-disk blake3 == pre-hash) + a SessionRewound row; `elai
  redo` re-applies (== post-hash) + EditHistoryEvent kind=Redo; an edit-after-undo creates a SIBLING node (undo target now has 2
  children) kind=Branched (the redo timeline is NOT destroyed — RV-3); a hand-edit-then-undo case FIRST forks (auto:pre-restore,
  RV-1). Missing row / wrong bytes / lost sibling = FAIL; a no-op (HEAD==target) emits ZERO rows. GD-4: the divergence-guard
  false-fork rate is bounded on >=3 runs before default-ON. Inherits W-1..W-10 + GD-1/2/4/5 + HD-1..HD-5 and ADDS RV-1..RV-4.

================================================================================
TRACK 9 — CONTEXT LIFECYCLE
================================================================================

### H-50 — regime-aware in-session context management (local unload/load vs cloud cache-stable; one window truth)
- Track 9 | lever CHEAPER/STEERING | grounding STRONG | nature BUILD-on-shipped | effort M-L | WAVE 9 | composes H-1/H-2, H-6, H-35/H-49
- Steal/build: cloud and local need OPPOSITE context policies and ELAI runs ONE cloud-shaped policy for both. Anthropic
  context-engineering (the policy must match the window) + vLLM-APC-vs-HF-transformers (HF has no cross-request KV reuse, so the
  LOCAL lever is window-minimization, not caching). Reuse-from-shipped (GD-1): adaptive_reduce + compaction_ladder (default-ON
  but mis-wired to the autonomous path) + recover_stubs (make selective) + H-6 masking. H-50 = fix the window truth + route the
  ladder/masking into the interactive LOCAL turn + make recovery selective + leave the cloud caching path intact.
- Existing state (grounded 2026-06-05): auto-compact is a FLAT 100K (conversation.rs:22) -> never fires for a 32K local model;
  the 85% gate reads a 64K-non-opus window (cli_consts.rs:20) IGNORING ELAI_LOCAL_MAX_CONTEXT -> triggers at ~54K above the
  window; even when it fires, recover_stubs re-inflates the trim to ~0 net (session_dag.rs:1551); adaptive_reduce has zero prod
  callers; the live backstop is a HARD context_window_blocked error (providers/mod.rs:257). Cloud caching IS live (anthropic.rs:1094
  at all 3 sites; H-1 split-brain REFUTED). LIVE SCHEMA 224/501.
- COMPOSES-WITH: H-1/H-2 (cloud caching, left intact), H-6 (masking + recover_stubs + adaptive_reduce activation — shared work),
  H-35/H-49 (SessionDag + selective recovery + RV-2 lossless-view), H-51 (the in-session base its tiered load rides). Restates none.
- Integration: effective_context_window(backend) + ContextPolicy (new, small) consumed by conversation.rs:789 (replace flat 100K)
  + live_cli.rs:1238 (replace max_tokens_for_model); route the LOCAL ladder via compaction_ladder.rs + H-6 + compact_session; make
  live_cli.rs:1317 recover_stubs SELECTIVE. frontend_pair: per-regime context gauge + "unloaded N (reloadable)". SSB: REUSE
  ContextReduction (GD-1, no variant); optional +1 WindowPolicyApplied. Default-ON + kill-switch ELAI_CONTEXT_POLICY_DISABLE.
- observable_effect: drive a real local session (ELAI_LOCAL_MAX_CONTEXT=32000, seeded) past trim_at -> a ContextReduction row
  lands (off-agent), the NEXT request's input tokens (from the journal) < window AND < pre-trim count, and a policy-DISABLED
  control run hits context_window_blocked while the ENABLED run COMPLETES. CL-4: a "caching on" claim is FAIL if the journal shows
  cache_read=0 (local streaming = 0 today). GD-4: false-drop rate bounded on >=3 runs. Inherits W/GD/HD + ADDS CL-1..CL-4.

### H-51 — cross-session sleep-time handoff: grounded topic-digest + auto-detect + tiered load
- Track 9 | lever CHEAPER/STEERING | grounding STRONG | nature BUILD-on-shipped | effort M | WAVE 9 | depends H-50, H-6; composes H-35 + the handoff rule
- Steal/build: today's cross-session resume is MANUAL (/continue) AND reloads the packed trimmed transcript (~20-27K on local =
  the pollution). The user's just-shipped .claude/rules/handoff-protocol.md produces the RIGHT artifact (a ~40-100-line topic-keyed
  curated digest) but only as a prose rule, unwired. Prior art WebSearch-verified 2026-06-05: Letta sleep-time (consolidate
  offline, 11K vs 20K same accuracy), Mem0 (lazy salient-fact load, >90% token savings, +26% LOCOMO), A-MEM (Zettelkasten
  linking), Anthropic note-taking. Reuse (GD-1): compact_session (digest engine), the SSB journal (off-agent ground truth),
  SessionDag (topic/tip), H-50 selective recover_stubs (lazy detail).
- Existing state (grounded 2026-06-05): /continue (repl_command.rs:97 -> handoff.rs:403) loads load_handoff -> the trimmed
  transcript capped at window-5.1K (session_dag.rs:909-912); a fresh REPL starts empty (main.rs:712); the only startup auto-resume
  is INDEXING (startup.rs:272). compact_session (compact.rs:96) + query_session_stream_asc (ssb.rs:15353) exist; recover_stubs
  (session_dag.rs:1507) is read-dark (H-6). The user's rule is live but unwired.
- COMPOSES-WITH: .claude/rules/handoff-protocol.md (wires it into the runtime — auto-author + auto-detect), H-50 (selective
  recovery + lossless view), H-6 (recover_stubs read-back), H-35 + SessionDag (topic/tip + RRA-verifiable resume), H-30/H-31/H-32
  (staleness + auditor firewall + validate-before-use). Restates none.
- Integration: new crates/orchestrator/src/handoff_digest.rs (sleep-time author from the journal via compact_session -> the
  topic-keyed workflow/handoffs/{slug}.md) + a topic-index {slug -> digest + DAG tip + FSM cursor}; the DETECT hook at main.rs:712
  + the OFFER before the first prompt; /continue re-pointed to the tiered (digest-first) load. frontend_pair: the resume offer in
  CommandWindow + commands_h51.rs. SSB: +1 Pattern-2 HandoffLifecycleEvent (DigestWritten/ResumeOffered/ResumeConfirmed/
  TieredLoad), baseline+1 from 224/501. Default-ON DETECT + default-OFF AUTO-CONTINUE, kill-switch ELAI_HANDOFF_AUTORESUME_DISABLE.
- observable_effect: end a real seeded session on topic T -> a DigestWritten row + a workflow/handoffs/{T}.md whose
  DECISIONS/FILES match the journal's intervention/FileWrite rows (off-agent, NOT recollection). A FRESH elai in the same project
  auto-surfaces the resume offer (ResumeOffered); on confirm, a TieredLoad row records loaded_tokens <= ~3K (NOT the ~20-27K
  transcript) and an early detail is still pullable via selective recover_stubs. No open session -> NO offer, ZERO rows. GD-2:
  resume token cost recomputable from the journal. Inherits W/GD/HD + ADDS CL-1..CL-4.

================================================================================
TRACK 10 — REVIEW-ANCHORING (external steal: alibaba/open-code-review)
================================================================================
Generated 2026-06-05 from an external-codebase STEAL exercise: databow (columnar.tech/blog/introducing-databow) +
anthropics/defending-code-reference-harness + alibaba/open-code-review were fetched; the two code repos cloned to
docs/.source + mapped (codebase_mapper.py -> _codebase.map) + explored by per-repo agents (summaries at
docs/.source/{repo}/_SUMMARY.md). VERDICT (anti-sycophantic): databow = an ADBC/Arrow data-engineering CLI with NO ELAI
fit (no steal); the Anthropic harness's genuinely-good patterns (execution-grounded fitness, fresh-adversary
re-attack-the-fix, two-container off-agent verification, sandbox-enforced-in-code) are ALREADY covered by Track 7
(H-43..H-48) + E-15/C-11 — porting them would RESTATE (GD-1 P0); the ONE net-new steal is open-code-review's
deterministic comment-anchoring resolver. Inherits W-1..W-10 + GD-1/2/4/5 + HD-1..HD-5; ADDS no new delta family.
NUMBERING: the Track 5 (odysseus) reservation bumped H-52 -> H-55 (Track 10 took H-52, Track 11 took H-53/H-54). LIVE SCHEMA 224/501 (2026-06-05);
H-52 +FindingAnchored baseline+1, ship-order-cumulative, re-read the live integer at edit (W-9).

### H-52 — deterministic snippet->hunk finding-anchoring + line-less finding schema (the reviewer cockpit's missing primitive)
- Track 10 | lever VERIFIED-BEFORE-SEEN | grounding STRONG | nature BUILD | effort M | WAVE 10 | depends G-5 + C-7; composes E-41/C-40 (deferred PR leg) + F-4
- Steal/build: alibaba/open-code-review's resolver.go (Apache-2.0, cloned to docs/.source/open-code-review/): the model
  emits a VERBATIM code snippet and NEVER a line number; deterministic code normalizes + matches a consecutive run
  against the hunk NEW-then-OLD side + falls back to a whole-new-file scan; 0,0 = can't-anchor -> file-fallback (the CI
  action posts a SUMMARY, not an inline line). The line-less code_comment schema makes a hallucinated line STRUCTURALLY
  impossible. Port WITH the two fixes the source LACKS: a duplicate-snippet -> Ambiguous guard + a hunk-proximity
  tiebreak (resolver.go is literal-match + first-occurrence-wins). EXCLUDED as grep-verified VAPORWARE: "smart bundling"
  + "reflection module"; the RE-LOCATION LLM retry is DEFERRED Tier-2.
- Why it is net-new: G-5's reviewer cockpit (DSR = lines-needing-attention / total-changed) has an OWN Open-Question #2
  — "confirm the FileWrite diff + the TestOutcome/gate rows join to the SAME hunk_id, or vouching attaches at the WRONG
  granularity." H-52 IS the resolver that closes it; the same primitive E-41's cross-family PR reviewer needs to anchor
  comments without trusting a model line. Neither plan specifies it.
- Existing ground truth (PLAN-grounded; ELAI code unfinished, executor re-greps W-7): G-5 score_review_surface / HunkVouch
  / review_surface_scored + its Open-Question #2; C-7 ChangeDecisionPayload.hunk_id (the `@@`-header hash); F-4
  TestOutcome stdout_hash; E-41 PR reviewer over C-40. Cited from the plan files, NOT fresh elai code greps.
- Integration: new crates/orchestrator/src/review_anchor.rs (anchor_finding -> AnchorResult{hunk_id?, line_range?,
  AnchorKind: Anchored / FileFallback / Ambiguous / Unanchorable / Relocated}); the ReviewFinding line-less schema; ONE
  Pattern-2 FindingAnchored SSB (baseline+1); the additive G-5 score_review_surface wire (join at hunk granularity);
  surfaced through the EXISTING `elai review --cockpit` entrypoint + the G-5 CockpitWindow frontend_pair. Default-ON +
  kill-switch ELAI_REVIEW_ANCHOR_DISABLE. Reuse ELAI's existing diff parser (do NOT re-author one — GD-1 / W-7).
- observable_effect: under ELAI_SEED, `elai review --cockpit` over a change where a finding's snippet occurs ONCE ->
  FindingAnchored kind=Anchored + inline render + a hunk_id that JOINS a C-7 ChangeDecision row (OQ#2 closed); occurs
  TWICE -> kind=Ambiguous + file-fallback (NOT a guessed line); absent -> Unanchorable + file-fallback. The
  --trust-model-line-numbers BROKEN ARM mis-anchors the duplicate case -> FAIL (R2). No findings -> ZERO rows. GD-2: the
  anchor re-computes byte-identically from snippet + diff. GD-4: over-fallback rate bounded on >=3 runs before default-ON.
- COMPOSES-WITH: G-5 (scoring consumer, closes its OQ#2), C-7 (hunk_id space), F-4/F-1/C-1 (vouch rows to anchor),
  E-41/C-40 (deferred PR-comment leg). Restates none (GD-1). Detail:
  docs/thefinalglance/H-plan/H-52_deterministic_finding_anchoring_cockpit.md.

================================================================================
TRACK 11 — EXTERNAL-SYSTEM RESEARCH STEALS (external steal: Perplexity AI + AlphaProof Nexus)
================================================================================
Generated 2026-06-06 from a research exercise on two external SYSTEMS (not codebases): Perplexity AI (oracle openness
run, workflow/sessions/oracle/2026-06-06-perplexity-ai-openness.md) + AlphaProof Nexus (DeepMind, WebFetch-verified
paper arXiv:2605.22763 + the proof corpus cloned to docs/.source/alphaproof-nexus-results @ 0647711, mapped + explored:
workflow/papers/alphaproof-nexus-{interface,HOW-IT-WORKS}.md). VERDICT (anti-sycophantic): two net-new cant-lie
hardenings, each on an already-strong ELAI surface, each grounded in ELAI's OWN substrate — the external system supplies
the SHAPE, never the guarantee. Both inherit W-1..W-10 + GD-1/2/4/5 + HD-1..HD-5 (H-54 also the Track-7 VD-1..VD-5); ADD
no new delta family. NUMBERING: Track 5 (odysseus) reservation bumped H-53 -> H-55 (Track 11 took H-53/H-54). LIVE SCHEMA
224/501 (2026-06-05); both PREFER REUSE/COMPOSE (H-53 extends E-48 citation_verified; H-54 extends H-44
TrapBehaviorDetected) so net SCHEMA delta is 0..+2, re-read the live integer at edit (W-9). Reasoning digests:
workflow/reasoning/PERPLEXITY-CITATION-SYNTHESIS-STEAL-2026-06-06.md + ALPHAPROOF-DEPENDENCY-CLOSURE-STEAL-2026-06-06.md.

### H-53 — citation pre-binding + constrained synthesis on the answer path (ghost citations made structurally impossible)
- Track 11 | lever VERIFIED-BEFORE-SEEN/cant-lie | grounding STRONG (on ELAI's D-11/E-48; the Perplexity mechanism is MEDIUM) | nature BUILD | effort M-L | WAVE 11 | depends E-48 + E-35/C-26; composes H-52, D-16, 3E
- Steal/build: ELAI's answer path is DETECT-AFTER (E-48 GhostCite hardening + Phase 2B verify run AFTER the model emits
  freeform [N]). Perplexity's documented answer stage embeds numbered ranked evidence anchors INTO the prompt BEFORE
  generation and constrains synthesis to cite-as-you-generate ("citations predetermined, not retrofitted"). H-53 moves
  citation assignment to a first-class PRE-GENERATION stage: freeze the ranked evidence as an EvidenceAnchorSet (rank-stable
  ids under ELAI_SEED, blake3-pinned excerpts) injected AFTER the last cache breakpoint (HD-2); the answer's [N] are checked
  against the frozen set (closed-set membership+support — E-48's verifier degenerates from an open-world ghost hunt to a
  membership check); an out-of-set [N] is held, an ungroundable claim routes to D-16 refusal, never an invented cite.
  REUSE H-52's verbatim->anchor resolver + E-48's verifier (GD-1). DEFER the E-35/C-26 constrained-decode grammar (Tier-2;
  marker positions only, HD-3; makes an out-of-set [N] mechanically unemittable) + the local-window budgeting (coordinate H-50).
- GROUNDING HONESTY (GD-5): the Perplexity-SPECIFIC mechanism is analyst-reported in the oracle doc + FLAGGED there as NOT
  officially confirmed (tier-2/3, MEDIUM); the steal rests on ELAI's D-11 constructive-grounding (arXiv:2211.12588 PoT /
  2402.01030 CodeAct) + E-48's GhostCite framing, NEVER a Perplexity number. ELAI publishes only its OWN measured
  ghost-citation-rate (->0) + refusal band, recomputable from the journal under ELAI_SEED (GD-2).
- COMPOSES-WITH: E-48 (verify -> closed-set; citation_verified extended +anchor_id +pre_bound), H-52 (the resolver), E-35/C-26
  (Tier-2 grammar), D-16/D-17 (refusal + SLA band), 3E + H-50 (the Evidence slot, window-budgeted per backend). Restates none (GD-1).
- Integration: new crates/orchestrator/src/citation_prebind.rs; inserted on the repl_command /research|/ask + researcher-oracle
  + E-48 answer paths before synthesis; SSB PREFER REUSE E-48 citation_verified (a CitationAnchorsBound Pattern-2 only if a
  distinct pre-binding row is needed). Default-ON + kill-switch ELAI_CITATION_PREBIND_DISABLE (W-2).
- observable_effect (W-1 done-gate): on a seeded fixture, `elai /research` over a known-answerable + known-unanswerable query ->
  (a) every [N] resolves to a pre-bound anchor (citation_verified pre_bound=true rows); (b) an out-of-set cite is BLOCKED + the
  unanswerable sub-claim refuses (D-16), no ghost shipped; (c) byte-identical [N] numbering under ELAI_SEED (GD-2). The
  --allow-freeform-citations BROKEN ARM lets a ghost ship -> FAIL (R2). GD-4: over-refusal bounded on >=3 runs (cross-check D-17).
  HONEST RESIDUE: hardens cite->evidence LINK, not evidence TRUTH (novel claims still D-15 + human review).

### H-54 — deterministic obligation-closure escape-hatch gate (the relocated-sorry / hallucinated-known-result defense)
- Track 11 | lever TRUST/PROVABLE/cant-lie | grounding STRONG | nature BUILD | effort M | WAVE 11 | depends E-53; composes H-44, R5/R7/R8
- Steal/build: ELAI's H-44 NAMES its own weakness — score_candidate_for_traps (trap_tasks.rs:123) is a keyword substring grep
  the code self-flags "unsound" (GAP-2 P0): a keyword-FREE verifier-gaming candidate passes. AlphaProof documents EXACTLY this
  evasion in a formal setting — (1) relocate the unproven `sorry` into a renamed helper lemma re-stating the target; (2) a
  hallucinated false "known-result" lemma — and ships the deterministic fix: inspect the WHOLE dependency closure, a
  sorry/admit/axiom/assume ANYWHERE disqualifies (the corpus confirms — 128 sorries confined to 2 LABELLED files, 69 fully
  closed). H-54 ports a deterministic obligation-closure scan: BFS the verifier-exposed obligation DAG, an UNREGISTERED escape
  hatch anywhere -> BLOCK; per-verifier HatchKind map (Verus assume/external_body, Kani vacuous-assume, Dafny assume/axiom,
  Z3 goal-restated-assert, Lean sorry/admit/axiom); a REGISTERED, R8-frozen allowlist for declared-open obligations (mirrors the
  corpus's labelled-open discipline). It is the deterministic FIRST leg of trap detection (the keyword grep demoted to
  KeywordFallback advisory — H-44's plan) and the deterministic core H-44's grep was a stand-in for; it does NOT replace H-44's
  behavioral legs (ImpossibleBench/isomorphic/CodeHalu) for non-formal classes with no proof DAG. It is wiring R7 applied to a
  proof DAG (admitted != proved, as test-root != production-reachable). DEFER a cross-verifier unified DAG abstraction (Tier-2).
- COMPOSES-WITH: H-44 (the deterministic first leg; extend TrapBehaviorDetected +ClosureEscapeHatch Kind, GD-1), E-53
  (Z3/Kani/Verus/Dafny — the lane the scan guards; reuse each verifier's obligation exposure; respect the HARD-AVOID of routine
  Lean), H-43 + Track-7 META-FINDING + TD-3 (AlphaProof = external grounding that the loop's filter must be an off-agent
  un-gameable artifact, the compiler not bytes; its Gemini critic is a search heuristic, the compiler is the gate), R5/R7/R8
  (unsound-label + admitted!=proved + frozen-allowlist). Restates none (GD-1). Inherits Track-7 VD-1..VD-5.
- Integration: new crates/orchestrator/src/verifier/closure_scan.rs (BFS + HatchKind map + registered-exception check, REUSING
  each E-53 verifier's obligation exposure, W-7); obligation-closure-allowlist.toml (R8-frozen via gate_freeze_check.py); wired as
  the cheapest FIRST leg of the E-53 lane + H-44's first leg; SSB PREFER COMPOSE (H-44 TrapBehaviorDetected +ClosureEscapeHatch
  Kind; ObligationClosureScanned standalone only if H-44 unshipped). Default-ON + kill-switch ELAI_CLOSURE_SCAN_DISABLE (W-2).
- observable_effect (W-1 done-gate): on a seeded fixture, the E-53 lane over — (a) a clean closure -> ClosureClean, verifier
  proceeds; (b) a proof that "passes" at the top because the hard obligation was RELOCATED into a renamed admitted helper ->
  EscapeHatchAnywhere naming the escape obligation id + BLOCK (the AlphaProof failure mode 1:1); (c) a declared-open obligation
  WITH a registered row -> RegisteredException (recorded, not BLOCKed); (d) the same hole WITHOUT a row -> BLOCK. The
  --top-statement-only BROKEN ARM passes (b) -> FAIL (R2). GD-2: the closure fingerprint re-computes byte-identically; GD-4:
  false-block bounded on >=3 runs. HONEST RESIDUE (VD-5): hardens "no hidden hole", NOT statement-conformance (-> C-29/F-8);
  sound only where the verifier exposes a DAG (boolean-only -> ClosureUnavailable + runtime backstop, R5); formal proof stays a
  narrow CSQ-Critical ceiling (~2.5-9% pass — steal the scan, never the search).

## Track-11 cross-cutting (what makes it different)
- Two external SYSTEMS researched (not cloned codebases like Track 10): Perplexity (closed — only the documented SHAPE is
  stealable, MEDIUM/GD-5) + AlphaProof (open corpus + paper — the mechanism is corpus-grounded + deterministic).
- Both are cant-lie hardenings that convert a DETECT-AFTER guard into a PREVENT-BEFORE / deterministic structural invariant:
  H-53 makes a ghost citation structurally impossible on the answer path; H-54 makes a relocated-admit structurally caught by a
  closure walk. Both are the off-agent-artifact doctrine pushed down into a SCHEMA / a graph fact.
- Both COMPOSE existing ELAI substrate (H-53: E-48/H-52/E-35/D-16; H-54: E-53/H-44) — composition, not invention; ADD no new
  delta family. AlphaProof additionally GROUNDS the standing Track-7 doctrine (H-43 execution-fitness, TD-3) + E-53's Lean avoidance.
- The external systems independently CONVERGE on ELAI's own thesis (Perplexity's stated moat = index+ranking+orchestration not
  the model; AlphaProof's load-bearing filter = the compiler not a learned reward) — recorded as a strategy confidence signal.

End of H-SUMMARY_INDEX.md.
