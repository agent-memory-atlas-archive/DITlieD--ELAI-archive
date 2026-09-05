# DEEP-H-001 exhaustive feature discovery

Generated: 2026-08-11 (Asia/Tokyo)

## Scope and method

This is a source-faithful deep read of exactly the nine files assigned to manifest chunk DEEP-H-001. Every assigned file was read from line 1 through EOF. The numbered plans and the auxiliary/grounding/fixture files are all treated as evidence. No source from another original chunk was read. 08-H-SUMMARY_INDEX.discovery.md was consulted only as an orientation/comparison index; it is not evidence for the ledgers below. No live-code claim was independently re-verified, so words such as SHIPPED, ABSENT, WIRE-DARK, and PROD-WIRED retain the authored source's time-local meaning.

Classification:

- Explicit ledger atoms (HG-*, UX-*, VG-*, H9-*, H11-*, H20-*, H28W-*, H35-*, H46-*) are directly stated by the named source.
- Inferences (INF-*) are derived only after the explicit ledgers and are not promoted to requirements.
- Ambiguities/conflicts (AC-*) preserve unresolved choices or incompatible source claims.
- Summary deltas (SD-*) identify additions or corrections relative to the orientation summary.
- Repeated facts in separate sources remain separate source-occurrence atoms. Within a source ledger, later repeated acceptance/test wording is consolidated into the first atom and cross-referenced rather than counted twice.

## Exact processed manifest and verification

The exact DEEP-H-001 manifest block declares 9 files, 1,499 logical lines, and 197,472 bytes, status "under 1,800". Mechanical wc -l -c and sha256sum verification matched every row:

| # | Manifest class | Lines | Bytes | SHA-256 | Exact path | Verified |
|---:|---|---:|---:|---|---|---|
| 1 | auxiliary/grounding/fixture | 374 | 81,705 | 8eec614056da0e92512037a4cc5cdff519b758adb6537c8ecd2a2546ac679321 | docs/thefinalglance/H-plan/_grounding/H-GROUNDING-MAP.md | exact |
| 2 | auxiliary/grounding/fixture | 48 | 8,685 | 40ea6fe11bafaa2de2e24a8dd2971dc964f36430099a072d4f4bf05953d4e21b | docs/thefinalglance/H-plan/_grounding/H-UX-MEMORY-GROUNDING-MAP.md | exact |
| 3 | auxiliary/grounding/fixture | 163 | 16,337 | 30d032f5e560771c1ebb0920566e772ffb70210198108a4ef77c9f00a69ffebc | docs/thefinalglance/H-plan/_grounding/H-VERIFICATION-GROUNDING-MAP.md | exact |
| 4 | numbered/index original | 192 | 16,711 | 3e354b8538656e9d06dc777d75cf08e4d47714014754c6e0c0838738b9d0f95e | docs/thefinalglance/H-plan/H-9_cost_cockpit_dashboard.md | exact |
| 5 | numbered/index original | 185 | 16,503 | dae3993560602d022db730a739f91d606fbb40afa7ed303292d2ee4c2f8f730d | docs/thefinalglance/H-plan/H-11_live_layer_file_explorer_fs.md | exact |
| 6 | numbered/index original | 181 | 15,186 | c9f710a0e83b3f9d3a102d45efded686fbe0200b7ee04b49ea551488e372c052 | docs/thefinalglance/H-plan/H-20_leakage_taint_composite.md | exact |
| 7 | auxiliary/grounding/fixture | 124 | 11,060 | 79c5b79dd9e804f7b6081a0bf692a6cb267ee4a9cfc7a7a567528a41c8b565c0 | docs/thefinalglance/H-plan/H-28-WIRE-1_reward_gate_darwin_admit.md | exact |
| 8 | numbered/index original | 58 | 16,304 | 9837fb42e7f53899e34c4fa76b9c4d7685c5183ff1624ab2a0773c783ff171e3 | docs/thefinalglance/H-plan/H-35_rewind_over_ssb_journal.md | exact |
| 9 | numbered/index original | 174 | 14,981 | eaa8357ef13cec49031341d85fbd95b95fdaf97c5d3b5812984ab4ccd4514251 | docs/thefinalglance/H-plan/H-46_clade_metaproductivity_selection.md | exact |

## Explicit per-file ledgers

### 1. H-GROUNDING-MAP.md — 374/374 lines

- HG-001 — L1-3: This is the W-7 distilled grounding reference for a 16-session H plan, produced from 17 grounding streams (12 code readers and 5 web verifiers) read 2026-06-03; its file:line:status triples are declared verbatim and must not be invented.
- HG-002 — L5-10: The authoritative 2026-06-03 snapshot is SCHEMA_VERSION=207 at ssb.rs:2909 and TOTAL_VARIANTS=484 at action_type_lock.rs:549/558; competing 191/464 reports came from a stale CLAUDE.md header.
- HG-003 — L12-15: Stale baselines are CLAUDE.md 191/464 and F-plan 205/482; e45_cross_language_drift_contract.rs:294 pins 482 and is a latent pre-existing failing assertion against 484.
- HG-004 — L17: Each action-type-adding H session must re-read live constants, bump both SSB schema and golden variant count plus ALL_VARIANTS, and treat 207/484 as a snapshot rather than a frozen literal.
- HG-005 — L19-24: H-1 is an activation/repair, not marker creation: the main ProductionAgentRpcServer -> AnthropicClient::send_message/stream_message -> apply_anthropic_cache_markers path already stamps the last system block and tool definition, while AnthropicApiRuntime::build_request/complete hardcodes no markers for secondary production callers and the hit-rate monitor is default-off/call-dark.
- HG-006 — L25: The sixteen cache_control:None values inside build_tool_catalog are inert by design because send-time marker logic re-stamps the last tool definition; they are not an H-2 wiring gap.
- HG-007 — L26-27: H-2's breakpoint machinery computes blake3 prefix validity but only prints it, sends no marker, passes empty slots 2-7, and runs only under BudgetPolicy::Local; activation/build must target the cloud request path.
- HG-008 — L29: shortcoder_compress_text returns only an estimated character count for budget math; request bytes are unchanged, so the claimed 15-30% compression is not a live token saving.
- HG-009 — L31: H-3 streaming validation lives in the shared API/local Anthropic loop but is dark everywhere because the sole builder sets streaming_validator=None; on_chunk has no production caller and early termination lacks an SSB emitter.
- HG-010 — L33: DIFF always receives editor_fs_override=None and falls back to STRUCTURED; FIM is blocked by a default-off execute flag and Cloud-to-Structured Rule 1; the shipped B-20 three-tier matcher is a different structured-tool path.
- HG-011 — L35: H-6 corrects two premises: adaptive_reduce has no production caller, and recover_stubs performs real blake3-verified recovery and errors rather than returning a placeholder, but is call-dark while the live CLI emits a hardcoded placeholder; compact_session is the only fully live symbol in this cluster.
- HG-012 — L37: H-7's PerRoleRuntime/RuntimeOrchestrator do not route task execution; policy evaluation is default-off and observational with synthetic audit/budget fields, although VRAM is live; only CloudModeRouter cap enforcement and escalation are production-wired.
- HG-013 — L39: H-8's Oracle cache uses tier-derived per-entry TTLs (T1 30d, T2 7d, T3 24h, rejected never cached), is lazily live but process-only with capacity 512, while PromptCache is already disk-backed under the configured/home cache root.
- HG-014 — L41: H-9's spend row is end-to-end dark because cloud constructors never attach an SSB writer; CloudPanel therefore shows zeros, the cost/task feeder is test-only, and a dedicated token-tracker component is absent, although in-memory token recording exists on the cloud branch.
- HG-015 — L43: H-10's symbol call graph already reaches Canvas from outgoing_calls; the distinct DKG edge table never contains CALL edges, PageRank readers have no producer, and file-view cyclic, level, and status fields are hardcoded inert values.
- HG-016 — L45: The H-12/H-16 canvas-isolation gate passes: runes stores do not import Three/Troika/scene/orbit; corrected anchors include lib/state, src/styles/tokens.css, class construction for NodeMandala, and AppShell columns 48px 1fr 0.
- HG-017 — L47: H-11 has no webview FS capability/plugin/dependency; Rust-side project open/reindex use std::fs, and registered list_projects/create_project have no frontend callers.
- HG-018 — L49: H-4 finds no advanced-tool-use/tool-search/token-efficient/interleaved-thinking beta; API-key profile betas do not reach the live OAuth branch, so any new beta must also be joined with OAUTH_BETA_HEADER.
- HG-019 — L51: build_tool_catalog has 16 definitions, not the stale 14 asserted by its contract; four DKG definitions lack conversion and dispatch arms, so tool invocation fails, and the extension precedent is the coordinated catalog/convert/dispatch three-arm pattern.
- HG-020 — L53: H-5 is pure build: no Anthropic Message Batches transport exists; Cosmos refresh is the clearest sequential batch-eligible lane, web research is weaker/sequential, GEPA is caller-dark and stubbed, external evals are RunPod-pending, and fixture bench commands do not call a model.
- HG-021 — L55: H-13/H-15's E-37 provenance/capability gate and two evidence rows are defined, locked, and tested but have no live dispatch caller/emitter; F-3p1 committed-tool enforcement is live, its text taint is replay-only, G-1 taint flow/G-2 RRA/native canvas capture are absent, and the media-validation consumer is shipped.
- HG-022 — L59-71: AnthropicApiRuntime::build_request advertises caching support while emitting no markers; the main send_message/stream_message path caches unconditionally; measure_cache_hit_rate returns verdict code 0 at hit rate <=0.05 but has no production caller and is behind ELAI_CRITIC_OF_DESIGN_ENABLE=1.
- HG-023 — L73-85: H-2 has exactly two modeled breakpoints (PhaseSpec, CodeContext), but validity is only telemetry and the second hash is degenerate because slots 2-7 are empty; CumulativeTokenTracker.cache_read_tokens is the one shipped cluster symbol and cloud/API branches do not call breakpoint update.
- HG-024 — L87-104: Legacy two-argument FIM/DIFF APIs return missing-provider; replay_hunks is call-dark; the active structured edit route is handle_tool_request -> translate_tool_request(TOOL_EDIT) -> dispatch_edit -> try_edit_tiers; mode selection emits GenerationModeSelected even though Cloud is forced structured.
- HG-025 — L94-104: Streaming activation requires constructing Some(validator), invoking on_chunk around each 50-token boundary, and writing StreamingEarlyTerminated; DIFF activation requires relaxing Cloud Rule 1, threading a real BufferEditor, and promoting ELAI_FOCUS_TUNNEL_EXECUTE; Rust activation must preserve existing dashboard/sidecar wire mirrors.
- HG-026 — L106-119: H-4's 16-entry catalog sets every tool's cache control to none, conversion/dispatch cover only 12, six artifact dispatch tools are agent-invisible because absent from the catalog, and input-schema field names must match IPC request structs exactly.
- HG-027 — L114-119: Default request-profile betas are claude-code-20250219 and prompt-caching-scope-2026-01-05; with_beta affects only API-key requests unless the OAuth header branch is explicitly changed.
- HG-028 — L121-134: Message Batches must add a sibling to the synchronous /v1/messages chokepoint; Cosmos tick has live CLI/dashboard callers, web research has a live task/dashboard path but sequential dependency, and dry external benches remain unsuitable until their host scaffolds are real.
- HG-029 — L136-150: The compaction ladder is production-reachable but default-off via ELAI_E25_COMPACTION_LADDER_ENABLE; its six ordered tiers (BudgetReduction, Snip, Microcompact, ContextCollapse, AutoCompact, NarrativeSummary) currently estimate 5/8/12/20/35/60 percent reductions and do not mutate content.
- HG-030 — L142-150: adaptive_reduce has three named levels plus Terminal and writes ContextReduction only in tests; get_relevant_facts is trust/provenance filtered but test-only; compact_session is deterministic and live from /compact and --resume.
- HG-031 — L145-150: create_handoff produces tagged stubs at the 85% budget threshold, yet recover_stubs/recover_single_stub have only contract callers and the production read path substitutes a literal placeholder, creating a write-active/read-dark asymmetry.
- HG-032 — L152-166: PerRoleRuntime and RuntimeOrchestrator exist but never participate in task-runner dispatch; RuntimeRoutingPolicy::evaluate is behind ELAI_ROUTE_DECISION_SSB_ENABLE, observational, and fed hardcoded audit/budget values.
- HG-033 — L159-166: CloudModeRouter and EscalateToCloud are live, but layer_failed is hardcoded to 1; HybridEscalationGate and entropy confidence are test-only; live calibrated routing must replace that literal rather than duplicate the cap-enforcement leg.
- HG-034 — L163-166: RouteDecisionEvent has complete three-kind schema machinery (CloudRouteChosen, TierModelSwapped, LocalRouteKept) but no production writer; the live route code logs only.
- HG-035 — L168-176: OracleAnswerCache keys query+project root with blake3, bumps observation TTL through CLI clock, emits Hit/Miss rows, and lazy-initializes on /research or /ask, but answers disappear at process exit.
- HG-036 — L173-176: PromptCache uses FNV-1a exact-match keys, has 30-second completion and 5-minute prompt TTLs, short-circuits HTTP on hit, records usage, and persists JSON state/entries on disk; it is not the H-8 persistence gap.
- HG-037 — L178-190: cost_per_task_cents has live CLI capture plumbing but no production samples; CloudGroundingClient accrues spend only when cloud-selected yet gates row emission and daily-cap logic on an unattached writer.
- HG-038 — L184-190: Cumulative token recording is co-located with the cloud client and may feed precompaction, while the default sidecar leaves it absent; CloudPanel polls every five seconds and has a real read path over cloud_spend, but source rows and redaction counts are missing.
- HG-039 — L189-190: H-9 should attach the cloud writer using the shipped TemplateCrystallizer/store wiring bundle, while not reusing the crystallizer's writer object as if it were the client.
- HG-040 — L192-218: DKG persists only contains/imports/implements/staleness-quarantined edges; call graph operations instead consume symbol_file_payload.outgoing_calls, making query_callers/callees against CALLS structurally empty.
- HG-041 — L198-210: Impact radius, symbol context, entry-point detection, graph view/export, and bounded visualization are shipped; call-flow/list-symbol operations are agent-IPC-only; caps are 20,000 symbols, 60,000 edges, and 200 symbols/file.
- HG-042 — L205-218: get_pagerank_top, CLI rank, and viewer read dkg_nodes_pagerank, but no production writer creates it, so PageRank is a producer-dark build; Cytoscape file-mode selectors are also inert because cycle/level/status data is hardcoded.
- HG-043 — L220-235: open_project_from_folder canonicalizes and sets project root; reindex_now runs full indexing on a blocking worker and emits progress; both are live, while persisted projects.json list/create operations are UI-dark.
- HG-044 — L227-235: The dashboard capability is limited to core:default and dialog:default; Tauri 2.11/dialog 2.7 are present, FS is absent, and direct JavaScript file reads would require a verified 2.x FS plugin plus project-root-scoped read permissions, never fs:default.
- HG-045 — L230-235: ProjectSelector chooses an existing folder and uses in-memory recent projects; the graph focus/impact bus is a cleared-after-consume Svelte singleton keyed by node id/path; panel split persists to localStorage with 0.2-0.8 clamp, 250ms debounce, and reset 0.5.
- HG-046 — L237-248: Canvas window dispatch, GraphWindow registration, Tree status shapes, and the two-effective-column AppShell are shipped; planet-scan.ts is only partially traced and must be verified before reliance.
- HG-047 — L250-264: Provenance and allow_tool_with_origin have extensive definitions/tests but no production dispatch use; the only live provenance touch is a safety replay fixture, and the closest taint primitive is likewise replay-only.
- HG-048 — L258-264: CommittedToolSet is a default-on fail-closed production gate whose unset state passes until a commitment exists; media validation is lazily created for media-bearing specs, emits seven media action types, and has a mock Tier-2 backend with no live VLM.
- HG-049 — L260-264: Native canvas screenshot capture and taint_flow.rs/TaintFlow are absent; H-13 must build capture/live dispatch wiring and may reuse capability_flow::taint_text rather than assume G-1 exists.
- HG-050 — L269-280: New H-13/H-15 SSB rows must use the nine-site lockstep (enum, SQL string, parser, CHECK, payload, payload action type, test payload, all-test payloads, golden), bump both live constants, and reconcile the stale e45 pin; existing capability/provenance rows are schema-complete but emitter-dark.
- HG-051 — L282-300: The live Orbit render chain is workspace activity default -> WindowHost -> ActivityWindow -> OrbitCanvas -> Starfield -> StarCanvas -> scene/NodeMandala; seeded LCG helpers are production-used, and retirement means removing that chain/dependencies after the isolation gate, not untangling stores.
- HG-052 — L304-315: Prompt-cache semantics verified at 2026-06-03: only ephemeral cache type, five-minute default, one-hour via ttl:"1h" with no beta header, read/write multipliers 0.1x/1.25x/2x, and first-read break-even only for five-minute TTL (one-hour needs a second read).
- HG-053 — L309-315: Minimum cacheable prefix must be keyed to exact model id (4,096 for Mythos Preview and Opus 4.5-4.7; 1,024 for Opus 4/4.1/4.8 and Sonnet 4/4.5/4.6; 2,048 for Haiku 3.5), with at most four breakpoints and a 20-block lookback.
- HG-054 — L311-315: Cache changes invalidate tools before system before messages; H-2's stable layout must respect that hierarchy, and sending more than four breakpoints yields a 400.
- HG-055 — L316-325: H-4 should send umbrella beta advanced-tool-use-2025-11-20; older scoped tool-search-tool-2025-10-19 also exists; regex/BM25 tool types are tool_search_tool_regex_20251119 and tool_search_tool_bm25_20251119, and the search tool itself must not be deferred.
- HG-056 — L320-325: Regex searches cap patterns at 200 characters; the service supports up to 10,000 tools and returns 3-5; supported models are Sonnet/Opus 4+ and Haiku 4.5+; usage reports server_tool_use.tool_search_requests; it is ZDR-eligible and batch-compatible.
- HG-057 — L321-323: Corrected Tool Search evidence is about 77K to 8.7K tokens (85%), not 72K; programmatic tool calling was 43,588 to 27,297 (37%); these are external figures, not ELAI measurements.
- HG-058 — L323: A custom ANTHROPIC_BASE_URL gateway cannot assume server-side deferred-tool expansion; ELAI must perform its own deferral, and CLAUDE_CODE_DISABLE_EXPERIMENTAL_BETAS=1 does not disable advanced-tool-use.
- HG-059 — L326-337: Message Batches use POST /v1/messages/batches, halve pricing, support active models and most ordinary/tool/thinking request shapes, expire unfinished work after 24 hours, retain results 29 days, and cap a batch at 100,000 requests or 256 MB.
- HG-060 — L331-337: Batch forbids streaming, Fast mode, Threads store/previous-event fields, cache/context hints, zero max tokens, and active research preview; one-hour prompt-cache TTL is recommended for shared batch context.
- HG-061 — L339-354: Verified Canvas dependencies are Cytoscape 3.34.0, layout-utilities 1.1.1, and Sigma 3.0.3; Cytoscape WebGL is provisional and needs canvas fallback, layout-utilities is real but stale, and stable Sigma v4 does not exist (alphas are banned).
- HG-062 — L356-369: Recast 0.23.11 is the maintained format-preserving writeback printer; React 19 removed source/self debug location parameters, so stable element identity should use build-time AST-injected opaque data attributes.
- HG-063 — L364-369: Current Onlook uses an iframe/CodeSandbox/penpal architecture and in-repo parser, not a custom renderer or a current publishable Babel-plugin dependency; its data-oid is opaque with a server-side id-to-location map, so H-14 novelty is sandbox/proxy control and DOM attributes must not expose file location.
- HG-064 — L370-374: Dependency blockers are stable-Sigma-v4 absence, legacy/unfetchable @onlook/babel-plugin-react, stale layout-utilities, and provisional Cytoscape WebGL; plans must pin/reroute/fallback accordingly.

### 2. H-UX-MEMORY-GROUNDING-MAP.md — 48/48 lines

- UX-001 — L1-5: This is the 2026-06-04 W-7 Track-4 map for H-30..H-38, grounded in live anchors unless marked confirm-at-authoring, a 15-lens research run, an author run, and MIT Odysseus reference maps.
- UX-002 — L7-11: The authoritative Track-4 snapshot is schema 220 and variants 497; 191/464, 207/484, and 205/482 are stale, and the first Track-4 bump must reconcile the latent e45 ~482 pin.
- UX-003 — L13-15: Track 4 must activate the existing memory substrate, not build a parallel store; FactType has eight variants and thresholds, Scope is Project/Global with promotion deferred, and MemoryFact records ingestion/validity/supersession/evidence timestamps and links.
- UX-004 — L15: Store lifecycle is atomic supersede (soft-close old plus insert new), newest-wins, never delete; current reads filter valid_to_ms IS NULL, with explicit read/insert/trust/iteration APIs.
- UX-005 — L16: mem0_dispatcher supports ADD, UPDATE-via-supersede, DELETE-via-soft-close, and NOOP in single/batch dispatch, but all callers are tests.
- UX-006 — L17: Deterministic conflict auditing is production-wired through live_cli.rs:2034, its first production caller.
- UX-007 — L18: Semantic contradiction uses an offline lexical default-on deterministic NLI judge, optional ONNX feature with no model on disk, cross-key collection, and an existing MemoryConflictDetected(semantic_contradiction) row; it is production-wired and needs no new variant.
- UX-008 — L19: Per-fact-type half-life decay, trust decay, tick, and scheduler already supply the HALO backstop.
- UX-009 — L20: get_relevant_facts filters on trust floor, nonempty evidence, and safety-critical tier exclusions and updates last_observed; every caller is a test, making retrieval wire-dark.
- UX-010 — L21: The eight-slot context budget has no memory slot and conversation code has no memory references; H-31 must add ContextSlot::Memory and call get_relevant_facts.
- UX-011 — L22-23: Current memory UI is read-only; CLI List/Inspect/Insert/Delete parse but dispatch to an explicit stub, which H-33 must de-stub.
- UX-012 — L25-26: Live STDIO MCP eagerly inserts full schemas with no defer gate; lazy schema cache/on-demand resolution exists only on a call-dark HTTP bridge, and this external-MCP problem is distinct from H-4's built-in catalog.
- UX-013 — L27: H-35 can compose ascending journal replay, deterministic replay IDs, fail-closed locked-key resume, ReplayResume evidence, Tauri resume, blake3 stub recovery, RRA build/verify/seal, and compact_session.
- UX-014 — L28: H-36 completion signals exist at several PhaseComplete sites, needs-input originates from RequestHuman, SSB is single-writer, stop hooks exist, and dashboard completion toast is implemented while intervention is an explicit no-op.
- UX-015 — L29: H-37's interactive APPROVE/REJECT/MODIFY/REDIRECT gate is test-only; a CLI interactive gate can be installed, compliance provider defaults absent, autonomous gate evidence exists, and task runner currently holds the gate presence-only.
- UX-016 — L30: H-38 can compose spend accumulator/snapshot/daily cap, spend summary, post-hoc cap precedent, and stop dispatch, but stop has zero cost awareness; pod_stop/pod_cost_status are harness MCP tools rather than ELAI Rust.
- UX-017 — L31: H-30/H-34 should mirror the frozen SecCodePLT evaluator pattern: embedded corpus, explicit seed, corpus blake3, measured beacon, no-literals honesty, ELAI_SEED, and single-writer evidence.
- UX-018 — L33-34: Odysseus read-time memory relevance uses a 0.9 identity floor and top-K cap; H-31 should reimplement the ranking behavior in Rust.
- UX-019 — L35: Odysseus vector similarity uses a 0.92 threshold; this is prior art for H-32 high-threshold dedup.
- UX-020 — L36: Odysseus extraction uses an order-independent fingerprint over sorted identity/text/category, a no-change short circuit, and a >50%-deletion guard; H-31 owns the guard and arms it when H-32 tidy lands.
- UX-021 — L37: Odysseus's untrusted-trace/NO_SKILL validate-before-persist gate is prior art for H-32's inversion to validate-before-use.
- UX-022 — L38: H-34 should assemble tool candidates as always-floor union vector top-K union keyword force-includes using ELAI's own ONNX vector stack.
- UX-023 — L39: H-35/H-33 should reuse trimming/headroom semantics but replace a 200K cloud assumption with the 25-32K local ceiling.
- UX-024 — L40-41: Odysseus write-time classification/dedup informs H-32; task scheduler/event bus inform H-36 and a later Track-5 event-trigger idea; all are MIT prior art to reimplement in Rust, never port.
- UX-025 — L43-45: Several future-dated arXiv identifiers and all headline figures require WebSearch/paper re-verification before load-bearing use; the e45 exact pin was also not grep-confirmed.
- UX-026 — L46: H-31's detail incorrectly describes H-30 as capture/intake; H-30 is only the staleness metric, and the live FactStore handle comes from the contradiction tick.
- UX-027 — L47-48: MemoryFact begins at line 138 rather than the doc-comment at 137, and several Odysseus/ELAI anchors were inherited from prior-agent maps; authoring must re-grep the authoritative unconfirmed-anchor lists.

### 3. H-VERIFICATION-GROUNDING-MAP.md — 163/163 lines

- VG-001 — L1-7: This is the 2026-06-05 W-7 live-code-grounded map for H-43..H-48; authors must read their slice plus citations and VD-1..VD-5 before coding.
- VG-002 — L9-21: Track 7's explicit rationale is that recursive self-improvement collapses with a weak/gameable filter and can converge with a stronger real filter; ELAI's byte-hash fitness and keyword trap check put it on the weak branch, so the work replaces placeholders and strengthens selection/eval rather than inventing doctrine.
- VG-003 — L26-31: Candidate::derived_alpha honors fixture force_alpha then maps the first eight blake3 bytes of the config to [0,1]; it is shipped but performance-blind, consumed by run_one_generation and the scoring chain, and H-43 must replace it with execution-grounded fitness.
- VG-004 — L32-37: score_candidate_for_traps is a shipped substring detector for fake-log/verifier-capture phrases; it is explicitly unsound, and H-44 must replace it with behavioral detection plus measured TRACE recall.
- VG-005 — L38-43: two_gate_promote uses carried-forward unconfirmed DEFAULT_TAU/DEFAULT_K_M and candidate-own mean alpha; H-45 must extend its consumed score with worst-case, variance, lineage diversity, and calibrated tau without rewriting the two-gate structure.
- VG-006 — L44-50: config_archive uses sigmoid immediate performance times exploration bonus, maintains a DB-enforced active uniqueness/archive cap/ancestor protection, and H-46 must add descendant-clade deltas as the selection key without redesigning the archive.
- VG-007 — L51-54: Flat and hierarchical BAI structs/sampling exist, but the real reward-driven live loop is scaffold-only/RunPod-pending; H-48 activates it using H-43 reward rather than inventing a new bandit.
- VG-008 — L55-56: A held-out task sampler is shipped; H-47 consumes it while D-6/E-54 retain ownership of the store and contamination filters.
- VG-009 — L57-59: Frozen SAFEDELTA/SecCodePLT runner and InsecureRateMeasured supply the mandatory shape for every Track-7 measured off-agent row; no new runner architecture should be built.
- VG-010 — L64-66: H-43 ships execution fitness now and leaves later cross-family step-level PPM improvement to D-2; it does not duplicate the PPM sidecar.
- VG-011 — L67-69: H-48 is wiring over D-5/D-29 BAI, with discounted Thompson/successive-reject/curriculum prior art; it is not a new bandit.
- VG-012 — L70-74: H-47 composes the held-out store, n-gram/cosine filters, cutoffs, and awareness probes; its only net-new pieces are score-time template regeneration, a generation-date gate, and adaptive-budget binding.
- VG-013 — L75-77: H-47 must reuse H-19's HoldoutBudgetEvent because a self-improver's repeated score checks are adaptive holdout queries.
- VG-014 — L78-80: H-44/H-45 must use a different pretraining family via D-13/E-6; E-6's IsoBudgetReport belongs to H-40 and must not be duplicated in Track 7.
- VG-015 — L81-83: E-53 already owns the narrow Verus/Kani/Z3 proof ceiling; Track 7 only references it and must not re-author a proof gate.
- VG-016 — L84-86: H-43's off-agent execution signal unblocks E-19 Task 11's conditional cloud-hard-lane SE-Agent recombination operator; H-43 names the dependency without restating it.
- VG-017 — L87-89: H-43 provides correctness, H-39 adds abstention credit, and H-45 applies worst-case/lineage aggregation to the same reward; these are orthogonal layers.
- VG-018 — L92-98: VD-1 requires every optimized self-improvement signal (derived alpha, promotion reward, bandit arm) to be measured execution behavior rather than bytes/self-confidence; failing code must score low and byte-hash fitness default-on is a failure.
- VG-019 — L99-102: VD-2 requires behavioral trap checks using execution/spec conflict/isomorphic perturbation and measured frozen-corpus TRACE recall; a keyword-free bypass or unmeasured guarantee fails.
- VG-020 — L103-107: VD-3 requires worst-case/variance-penalized ensemble scoring, pretraining-lineage diversity, blocking cross-view disagreement, and PRMBench-calibrated tau; own-mean promotion or an unconfirmed default-on tau fails.
- VG-021 — L108-112: VD-4 requires execution/proof-grounded verification stronger than the generator and a verifier from a different pretraining family; equal-or-weaker verification is not a gate.
- VG-022 — L113-118: VD-5 requires explicit undecidable residue, narrow proof/human Layer-3 routing, sandbagging mitigation rather than immunity claims, and a dark/allowlisted self-improvement loop; cross-hardware FP identity and concept drift remain human-gated.
- VG-023 — L121-127: The authoritative 2026-06-05 schema snapshot is 224/501; all older baselines are stale, and the first Track-7 schema bump must reconcile e45's ~482 pin.
- VG-024 — L128-139: Track-7 additions are ship-order cumulative: H-43 FitnessEvaluated, H-44 TrapBehaviorDetected, H-45 PromotionScoreAggregated, H-46 CladeSelected, H-47 EvalFirewallEvent; H-48 should reuse BAI telemetry and adds BanditArmSampled only if no emitter exists.
- VG-025 — L130: FitnessEvaluated kinds are ExecutionPassRate, DualAgreement, and FitnessRegression.
- VG-026 — L131-132: TrapBehaviorDetected kinds are ImpossiblePassExploit, IsomorphMismatch, ExecutionHallucination, and KeywordFallback; recall, N, and corpus hash share the same payload rather than a second variant.
- VG-027 — L133: PromotionScoreAggregated kinds are WorstCaseScored, VariancePenalized, and LineageDiverseDisagree.
- VG-028 — L134: CladeSelected kinds are CladeMetaproductivity and ImmediateScoreFallback.
- VG-029 — L135-138: EvalFirewallEvent kinds are PreUpdateItemRejected, TemplateRegenerated, and IndistinguishabilityProbe; holdout spend reuses H-19; H-48's possible row is conditional.
- VG-030 — L139: If H-43..H-47 ship from the cited snapshot, the net is +5 (224->229 and 501->506), but every edit must re-read live values.
- VG-031 — L142-152: The map re-confirms CodeRM as H-43's execution-fitness pick, ImpossibleBench plus isomorphic/reward-hacking work for H-44, and HGM/CMP for H-46; HGM figures are external paper evidence, not ELAI measurements.
- VG-032 — L153-161: The remaining named research set was dossier/WebFetch-verified, while SCoRE and PSV-Verus are MEDIUM and require paper-ingestor re-verification; Self-Verification-Prevents-Collapse has no arXiv id and none may be invented.

### 4. H-9_cost_cockpit_dashboard.md — 192/192 lines

- H9-001 — L1-4: H-9 is PLANNED Wave 1, nature ACTIVATE + BUILD, and is the instrument intended to make H-1, H-2, and H-3 falsifiable.
- H9-002 — L5-10: SpendAccumulator, CumulativeTokenTracker, and CloudSpend construction exist, but all row work is gated by an optional writer that no cloud constructor attaches; the mounted five-second-polling CloudPanel consequently shows all zeros.
- H9-003 — L9-14: Production cost-per-task always remains None, the only feeder is test-only, and no dedicated token-tracker component exists; the planned surface shows per-response input/output/cached counts, cache expiry, dollars/resolved task, and cache hit rate.
- H9-004 — L10-19: The activation must attach CloudGroundingClient.with_ssb_writer following, but not conflating with, the shipped TemplateCrystallizer pattern; H-1 supplies measured hit rate and H-3 supplies input/output-ratio coordination.
- H9-005 — L16-20: Estimated scope is 1-2 sessions/6-10 hours; CloudPanel/read command and the dark CloudSpend emitter are prerequisites; it is parallel-safe with H-5 through H-8 but must coordinate H-1/H-3.
- H9-006 — L24-34: The governing efficiency unit is a model+harness configuration, not model rank; report ($/resolved-task, cache-read fraction, input:output token ratio, tokens/resolved-task) with paired bootstrap confidence interval and about a six-percentage-point noise floor, recomputable under ELAI_SEED.
- H9-007 — L26-34: Cache and output gains must remain separate rather than multiplied; Devin-style response breakdown/expiry is UX prior art, while ELAI's claimed differentiator is journal-recomputable rigor.
- H9-008 — L36-44: Spend and cache accumulation calls are live inside CloudGroundingClient, but CloudSpend emission and daily-cap checking are dormant because from_env/from_env_async build writer=None; token tracking is cloud-branch-only and sits inside the same writer gate.
- H9-009 — L45-50: elai metrics capture is wired but receives no production cost sample; CloudPanel reads SsbViewer cloud-spend rows, has a hardcoded zero redaction count, and admits the CLI cloud-mode bridge is follow-up work.
- H9-010 — L51-62: A token dashboard is a true creation; existing spend, cache, metrics, dashboard, and crystallizer anchors must be read before coding, along with hard rules HD-1/GD-2.
- H9-011 — L64-69: Task 1 attaches an SSB writer at the sole task-runner cloud constructor and threads it through from_env/from_env_async so CloudSpend rows actually land.
- H9-012 — L70-72: Task 2 creates a non-test record_cost_sample_cents feeder and requires its result to be journal-recomputable.
- H9-013 — L74-76: Task 3 creates a roughly 280-line Svelte/60-line Tauri token surface with input/output/cached tokens, cache-expiry timer, dollars/resolved-task, and H-1 hit rate, reading live CloudSpend rows and passing cross-language diff.
- H9-014 — L78-80: Task 4 removes false zero rendering and hardcoded redaction count; if G-9 is unavailable, zero may remain only with an honest unwired label.
- H9-015 — L81-83: Task 5 creates H-9-decisions.md recording the dark writer, attachment precedent, measured-per-configuration doctrine, UX steal, and four-component efficiency tuple.
- H9-016 — L85-97: Produced symbols are writer attachment/threading, a non-test cost feeder, token-tracker component/read command, and real redaction count; consumed symbols are CloudGroundingClient emitter, SpendAccumulator, CumulativeTokenTracker, TemplateCrystallizer pattern, get_cloud_spend, and measure_cache_hit_rate.
- H9-017 — L98-100: Production reachability is task_runner.rs:5840 -> CloudSpend rows and commands.rs:5088 -> CloudPanel/token component; neither a fixture nor a test-only path is sufficient.
- H9-018 — L101-108: Stub bans prohibit writerless cloud clients, test-only cost feeders, falsely claimed zero redactions, multiplied headline savings, non-blake3 hashing, and unwrap/expect in production; frontend and Tauri types must remain cross-language paired.
- H9-019 — L109-115: The live trigger is a real cloud-mode task with no extra override followed by opening CloudPanel; it must create input/output/cache-read/spend rows and visibly nonzero token/cost/hit-rate/expiry values.
- H9-020 — L111-115: Every displayed value must recompute byte-identically from journal evidence under ELAI_SEED; the pre-H-9 contrast is zero rows and all-zero rendering.
- H9-021 — L117-126: Deterministic static checks require run-trigger success, writer references in cloud construction, a non-test cost feeder, TokenTrackerPanel.svelte, non-hardcoded redaction handling, cross-language diff, and the decision document.
- H9-022 — L128-138: Acceptance requires default-on row emission within the existing cloud-lane selector, real production reachability, dashboard-smoker evidence, journal-recomputable tuple, honest redaction labeling, workspace/auditor/Svelte green, and no new dependency.
- H9-023 — L140-151: No new SSB action type is planned because CloudSpend already exists; done requires both Tier-1 in-process row activation and a W-6 dashboard render, not Cargo success alone.
- H9-024 — L153-158: The H9 contract must prove writer presence, real CloudSpend token/spend payload, byte-identical recomputation, real production cost/task feeding, and honest redaction; dashboard-smoker must prove both panels render nonzero values.
- H9-025 — L160-168: Planned file lifecycle is modify task runner, grounding client, metrics/collector, Tauri commands, and CloudPanel; create TokenTrackerPanel, H9 contract, and H-9 decision record.
- H9-026 — L170-177: External comparator figures and efficiency citations are grounding only; load-bearing ELAI facts are the dark writer, zero panel, test-only feeder, and shipped attachment precedent.
- H9-027 — L179-182: Because sidecar is the default backend, a session without cloud work should render the explicit empty state "no cloud spend this session", not unexplained zeros.
- H9-028 — L183-185: The cockpit must resist reducing its tuple to an X-percent-cheaper headline, and until G-9 ships it must say redaction is not wired rather than imply zero secrets.
- H9-029 — L187-190: H-9 closes Wave 1 by consuming H-1 hit rate, H-2 cache stability, and H-3 output ratio under the same paired-CI/journal discipline as SAFEDELTA; H-10's Canvas B0 follows.

### 5. H-11_live_layer_file_explorer_fs.md — 185/185 lines

- H11-001 — L1-4: H-11 is PLANNED Wave 2 BUILD and owns the CanvasActivityEvent live-overlay transport.
- H11-002 — L5-8: The overlay applies to the currently active Canvas mode, gives the active node a halo, traversed edges dash-flow, and exposes a follow-agent attention-centering toggle.
- H11-003 — L8-12: File explorer behavior is VS Code-style and bidirectionally synchronized over the existing graph focus bus; the plan builds project-scoped FS access and activates UI-dark project list/create commands.
- H11-004 — L13-18: Estimated effort is two sessions/10-14 hours; H-10 must land first, H-12 can proceed in parallel only with shell coordination, and shipped prerequisites are project open/reindex, focus bus, Tree, and panelSplit.
- H11-005 — L20-28: CanvasActivityEvent must carry target node id plus raw file_path, symbol_name, and mode fallback so resolution failure never vanishes; an unmatched event creates an unverified transient ghost near the nearest resolvable ancestor.
- H11-006 — L28-32: A ghost reconciles to a real node when indexing catches up or fades after 30 seconds into a dropped marker; selecting a Canvas file/symbol highlights its tree file and clicking a file focuses/centers its Canvas node.
- H11-007 — L29-32: The focus bus key is path and the explorer is another publisher/consumer; FS authority is root-scoped, never a whole-disk capability.
- H11-008 — L34-44: Project open and reindex are shipped; project list/create are registered/wrapped but have zero frontend callers; current permissions are core/dialog only, FS dependency/plugin is absent, and the existing AppShell has no explorer column.
- H11-009 — L35-43: reindex_now uses spawn_blocking and emits reindex-progress; the graph bus exposes focus/consume and impact/consume with clear-after-consume; panel split clamps 0.2-0.8 with 250ms persistence; Tree's current status-oriented shape needs a filesystem variant.
- H11-010 — L46-52: Mandatory reads include H-grounding C13, Canvas design live/failure/FS/multi-root/grid rules, current commands and Tauri capability/dependency registration, focus/panel state, and live SSB constants/plugin version.
- H11-011 — L54-60: Add exactly one Pattern-2 CanvasActivityEvent with spatial target, file/symbol/mode fallback, touch kind (reading/editing/validating/researching), and short label; H-11 owns it before H-13 and must use live baseline+1 plus e45 reconciliation if first bump.
- H11-012 — L62-66: canvasActivity.svelte.ts subscribes to CanvasActivityEvent and existing FSM/GRC/evidence/bubble signals, choosing among active-mode glow, non-active-mode off-frame indicator, or fallback ghost; follow-agent defaults on.
- H11-013 — L65-66: Overlay motion ports halo-pulse, dash-flow, and bubble-ring, while ghost lifecycle remains reconcile-or-30-second-drop.
- H11-014 — L68-71: FileExplorer and fileTree build a collapsible Tree with filesystem nodes {path, kind=file|dir, no status}, add a persisted resizable AppShell column outside the 48px icon rail, and reuse the focus bus both directions.
- H11-015 — L73-77: The Tauri surface is read_dir, read_file, open_in_editor, and mount_root, all scoped per mounted root; multi-root extends today's single session project.
- H11-016 — L73-77: tauri-plugin-fs is conditional on direct Svelte reads, its current 2.x version must be verified, and permissions if installed are allow-read-dir/read-text-file plus project-root scope, never fs:default.
- H11-017 — L79: list_projects/create_project must gain real explorer/project-switcher callers rather than remain registered wrappers.
- H11-018 — L81-82: H-11-decisions.md records ghost behavior, root-only FS, the new column, existing bus reuse, and multi-root policy.
- H11-019 — L84-95: Produced artifacts are CanvasActivityEvent/Kind/Payload, overlay store, explorer/tree state, commands_h11 FS calls, scoped capability, plugin registration, and layout column; they consume the existing graph bus, project commands, panel persistence, and Tree primitive.
- H11-020 — L96-99: Reachability must be Canvas window -> overlay subscriber, AppShell explorer -> FileExplorer -> real FS commands, and live agent task -> event rows -> overlay marker.
- H11-021 — L101-108: Stub bans prohibit silent resolution gaps, whole-disk grants, assumed plugin versions, fixture activity, hardcoded schema counts, non-blake3 hashing, and unwrap/expect in production; frontend/Tauri pairs and dashboard E2E are required.
- H11-022 — L109-115: The live trigger opens a real project and runs an agent touching files; rows must contain file/symbol/mode/kind and the overlay must consume them live.
- H11-023 — L112-115: A within-root node highlights its file, a cross-root monorepo sibling exposes and resolves through mount, an external node such as node_modules shows an explicit external state, live focus moves node-by-node, and an unindexed file shows the full ghost lifecycle.
- H11-024 — L117-128: Deterministic checks require run-trigger, event schema/golden, FileExplorer, commands_h11, scoped FS grants, reuse of focus_node, cross-language diff, and the decision file.
- H11-025 — L130-139: Acceptance requires default-on overlay/explorer with no override, real rows and FS calls, dashboard-smoker, Pattern-2 roundtrip/golden/schema drift repair, path-failure behavior, root scope, verified plugin version, workspace/auditor/Svelte green.
- H11-026 — L141-151: The done gate is Tier-1 in-process plus dashboard proof; produced overlay consumes its event and produced explorer consumes FS/focus, and cargo-green alone is insufficient.
- H11-027 — L153-158: Contract cases are event fallback roundtrip, active/non-active/missing-node triage, ghost reconciliation/timeout, traversal escape rejection, and reuse of the existing focus bus rather than a new bus.
- H11-028 — L160-168: Planned lifecycle modifies SSB/golden/e45, capabilities/dependencies/registration/layout/workspace; creates overlay/explorer/tree state, commands_h11, contract test, and decision record.
- H11-029 — L170-174: Canvas design evidence is motivating rather than causal proof; the grounded FS absence, orphan commands, focus bus, and exact grid are the implementation facts.
- H11-030 — L176-179: Ghost placement depends on H-10 folder/LOD hierarchy; any symlink resolving outside a root must be canonicalized and rejected; multi-root nodes need mount affordance while true externals get a badge.
- H11-031 — L181-183: H-12 must hand over a single SSB consumer so Orbit and Canvas do not double-consume, then H-13 layers model operations on the transport H-11 owns.

### 6. H-20_leakage_taint_composite.md — 181/181 lines

- H20-001 — L1-4: H-20 is PLANNED Track-3 Wave-3 Cluster-B, an in-process BUILD on a shipped-but-partial taint primitive with G-1 explicitly unresolved.
- H20-002 — L5-13: The target failures are fit-before-split, normalization across the boundary, full-series PCA/feature selection, and missing signal-to-execution lag; compile/test success is structurally blind to them.
- H20-003 — L8-13: Tier-1 is a COMPOSITE dataflow proof: no fit site may reach both train and test sinks, no normalization statistic may taint earlier timestamps, and no full-series dimension reduction/selection may occur before split.
- H20-004 — L10-13: A SCHEMA_REQUIRED typed split boundary gives the pass a partition, Tier-2 AST_PATTERN detects missing shift(1), and all failures are fail-closed with registered suppression rather than silent skip.
- H20-005 — L13-16: The off-agent artifact is the reachability proof, not agent-authored prose; claimed soundness is limited to fit/normalize/PCA classes, while target-derived and sampling-bias cases route to Tier-3 advisory.
- H20-006 — L15-22: Estimated effort is one M-L session/8-10 hours; taint primitives and AST/Composite/SchemaRequired engine exist, G-1 live taint is absent, and H-21/H-22 are parallel-safe.
- H20-007 — L24-34: Static analysis is necessary but insufficient: dynamic dispatch and target-derived features remain residue, and H-21's full-vs-truncated runtime differential is the authoritative backstop rather than textual name matching.
- H20-008 — L36-45: Provenance/capability/text-taint definitions exist but propagation is replay-only; committed tool categories are live; rule predicates and AstEvaluator are shipped; G-1 flow and leakage reachability are absent.
- H20-009 — L47-54: Authoring must read the trust map, design mapping, H-21, taint/rules source, and G-1 plan, then obey blake3/no-unwrap and honest Tier-3-residue rules.
- H20-010 — L56-61: Before implementation, H-20-decisions must choose either shipped G-1 live taint or a finance/leakage-scoped DKG dataflow pass with provenance labels; assuming G-1 is forbidden.
- H20-011 — L63-66: Every executable backtest/strategy module must declare a typed train/test partition boundary or decision time; absence blocks because an undeclared split cannot be analyzed.
- H20-012 — L68-72: The COMPOSITE recognizes fit/fit_transform/train sites, dual sink reachability, cross-time normalization/scaling, and pre-split full-series PCA/feature selection using abstract interpretation over disjoint splits.
- H20-013 — L71-72: A flagged pandas idiom can pass only through a frozen registered allowlist row carrying a reason.
- H20-014 — L74-76: Missing shift detection identifies a signal consumed by execution/position on the same bar; unambiguous same-bar use blocks, while ambiguous cases warn and route to Tier 3.
- H20-015 — L78-81: Add Pattern-2 LeakageBlocked with kinds FitReachesBothSinks, NormalizationAcrossBoundary, FullSeriesPCA, and MissingShift; payload includes tainting file:line:symbol and the suppression registry is frozen/non-agent-editable.
- H20-016 — L78-81: The cited baseline is live 207/484 only as an authored snapshot; implementation re-reads and bumps schema/golden/e45 in lockstep.
- H20-017 — L83-86: H-20-decisions records dataflow choice, sound-vs-advisory classes, suppression policy, and H-21 authority.
- H20-018 — L88-97: Produced artifacts are evaluate_leakage, locked finance/leakage.toml, LeakageBlocked, and the registry; consumed artifacts are provenance/text taint, AstPattern/Composite/SchemaRequired, DKG/AstEvaluator, and conditionally G-1.
- H20-019 — L98-100: Production reachability is PreToolUse and/or pre-promotion on backtest/strategy modules, followed by evaluate_leakage -> reachability -> evidence row.
- H20-020 — L101-108: Stub bans prohibit silent suppression, blocking without a dataflow proof, soundness claims for residue, duplicate taint/rules engines, non-blake3 hashing, and unwrap; the dashboard drawer must display the fit site and reachability path.
- H20-021 — L109-113: A no-override StandardScaler-before-split fixture must emit LeakageBlocked(NormalizationAcrossBoundary), name the fit site, block execution, allow split-then-fit, and require a registered row for an intentional idiom.
- H20-022 — L111: Leakage evidence and reachability must recompute from the journal under ELAI_SEED.
- H20-023 — L115-122: Deterministic checks require run-trigger, a production taint/composite caller, locked finance seed, zero blake2/sha2 use in the new evaluator, and the decision record.
- H20-024 — L124-132: Acceptance requires default-on execution with kill-switch only, production pre-tool/promotion reachability, honest Tier-3 routing, documented G-1 resolution, reused primitives, seeded reproducibility, workspace/auditor green, and no dependency.
- H20-025 — L134-141: Done requires cross-language leakage drawer, dashboard visibility, a Pattern-2 schema bump, and explicit separation from H-21; cargo-green is not enough.
- H20-026 — L143-148: Contract cases are fit-before-split block with site, pre-split PCA block, missing-lag block, clean split-then-fit allow, registered suppression, target-derived advisory, and identical two-seed executions, with a frozen fixture family.
- H20-027 — L150-157: Planned lifecycle creates evaluator, locked finance rule file, suppression registry/gate-manifest entry, contract/fixtures, and decisions; modifies SSB/golden/e45 and live pre-tool/promotion routing.
- H20-028 — L159-165: Dataflow research figures require paper ingestion before implementation; load-bearing code facts are the partial taint source, shipped predicate engine, and absence of G-1/leakage predicate.
- H20-029 — L167-170: If G-1 remains absent, implementing the scoped DKG pass itself may be L effort; that material scope choice must be made before committing, with H-21 as runtime fallback.
- H20-030 — L171-174: Pandas false positives must be bounded on at least three real backtest modules before default-on; static Python dataflow is generally unsound and H-21 must ship for a trustworthy leakage claim.
- H20-031 — L176-179: H-21 compares full versus future-truncated feature columns and H-22 tests edge persistence under shuffled future; together with H-20 they close TB-2.

### 7. H-28-WIRE-1_reward_gate_darwin_admit.md — 124/124 lines

- H28W-001 — L1-9: This 2026-07-07 supplement classifies H-28 production wiring as a low-blast-radius ACTIVATION in darwin_admit_loop.rs: the guard, CLI proof, frozen manifest, and admit seam already exist but have zero production guard callers.
- H28W-002 — L5-9: It explicitly excludes H-29 and limits scope to one orchestrator file plus one wire-dark allowlist row, one unit test, and one smoke; impact radius was reported as zero at-risk.
- H28W-003 — L11-12: The slice feeds proof-spine PS-6 by converting a built-but-CLI-only capstone gate into a production caller and also exercises the free-worker implementation leg.
- H28W-004 — L17-24: check_reward_tampering accepts SSB, DB path, expected/observed harness blake3, prior/new reward, band, run id, and seed, returns RewardTamperingOutcome/Error, classifies, and emits exactly one RewardTampering row as the off-agent artifact.
- H28W-005 — L24: DEFAULT_PLAUSIBLE_REWARD_BAND is a public module constant already consumed by the CLI and must replace any invented literal.
- H28W-006 — L25-29: The GuardOutcome::Admitted arm immediately precedes archive.insert_variant and already has the loop's Arc<Ssb> because it emits admit and clade-selection rows; this is the single production interception seam.
- H28W-007 — L30-33: The loop reportedly holds last admitted fitness, current execution-grounded fitness, seed, run identity, and frozen corpus; the existing guard is CLI-proven and frozen, so this wiring changes callers rather than gate math.
- H28W-008 — L36-40: The vertical slice places a default-off interception at the top of the Admitted arm before insert_variant; a Tampering verdict skips insertion and emits a reject/non-admit beacon.
- H28W-009 — L42-47: Parameter mapping is fixed: SSB is the existing row-emitter handle, prior reward is last_admitted_fitness, new reward is the same candidate execution fitness used by the guard chain, and band is DEFAULT_PLAUSIBLE_REWARD_BAND.
- H28W-010 — L48-53: Expected and observed harness hashes both use frozen-corpus blake3 in this loop, run id and seed reuse existing row fields, and the active branch is reward-band detection; harness-swap detection remains present and should fire only if corpus hash drifts.
- H28W-011 — L54-56: New flag ELAI_H28_REWARD_GATE_ENABLE defaults OFF; OFF preserves exact current behavior, ON blocks insertion for Tampering and records rejection.
- H28W-012 — L58-60: Exact field/accessor identifiers remain mechanical reconnaissance around the Admitted arm and run_one_generation; their semantic mapping is not open for redesign.
- H28W-013 — L60,72-75: If both real prior/new reward signals are not available at the seam, the executor must abort and hand off because plumbing them through signatures exceeds this activation slice; placeholder reward pairs are forbidden.
- H28W-014 — L62-65: ON with an implausible upward reward jump must emit RewardTampering(Tampering) and no Admitted beacon; OFF must admit exactly as main and emit no RewardTampering row.
- H28W-015 — L66-70: A compile/type failure triggers re-reading exact fields and borrowing Arc<Ssb> like emit_admit_row; a missing ON-path row means the call is too late or on the wrong branch and must move before parent selection/insertion.
- H28W-016 — L71: No implementation forks are authorized apart from the explicit abort tripline.
- H28W-017 — L76-79: Auditor verification is production_caller_check, focused orchestrator tests, and assertion that the ON smoke lands the off-agent row; executor stdout is not evidence.
- H28W-018 — L80-84: The integration smoke must call real run_one_generation over a hard candidate set, prove an implausible candidate is not admitted, prove its Tampering row, and prove flag-unset behavior matches main.
- H28W-019 — L86-95: The flag's wiring contract is runtime run_one_generation reachability and a wire-dark allowlist row with type=kill_switch, owner=maintainer, expiry_date=2026-10-01, promotion_plan_id=H-28-WIRE-1, and smoke name.
- H28W-020 — L92-95: Observable semantics are ON -> reject plus row and OFF -> unchanged; flag, call, allowlist, unit test, and smoke must ship together with no TODO, placeholder value, or "wire later".
- H28W-021 — L96-99: gate_freeze_check must remain green and grep must find the allowlist row; an ON-default or missing five-field allowlist is an R9 violation repaired by restoring OFF plus registration.
- H28W-022 — L102-108: Red-team patch 1 specifically rejects 0.0/0.0 or constant reward mapping: the hard candidate must produce a real Tampering verdict from current execution fitness.
- H28W-023 — L109-111: Existing Darwin tests remain behaviorally stable because they exercise the default-OFF path; only new tests exercise ON.
- H28W-024 — L112-116: The activation claim is protected by the no-real-pair abort; the author's grounding expected both signals to exist, but the tripline remains mandatory.
- H28W-025 — L118-124: The executable-blind criterion is that all semantic decisions, seam, flag, and artifacts are named; only identifier lookup remains, every alternative is none/abort, and authority rests on the row/checker/test rather than worker narrative.

### 8. H-35_rewind_over_ssb_journal.md — 58/58 lines

- H35-001 — L1-3: H-35 is Track-4 Wave-4, strong-grounded M-effort BUILD-UI-over-journal, depends on H-33 window governance, and composes G-2 RRA.
- H35-002 — L3: User intent is restoration to a known-good point after a bad edit/turn/refactor without losing audit history, scoped conversation-only, code-only, or both, with optional summarize-from-here.
- H35-003 — L3,10: The implementation must be a logical UI/restore layer over seeded append-only SSB replay, never a separate checkpoint store or shadow Git; new surfaces are CLI, dashboard control, SessionRewound, and a dispatcher.
- H35-004 — L5-6: Claude Code's measured adoption is cited only to justify stealing the affordance/scope, not its out-of-band checkpoint implementation or to claim an ELAI measurement.
- H35-005 — L7-9: Odysseus trim and sorted fingerprint are MIT prior art to reimplement, never port; ELAI must consume compact_session and use blake3 over tail (seq, action_type, event_blake3) for idempotence.
- H35-006 — L12-14: The source claims the journal/replay/stub substrate is all production-wired and only affordance is missing; SSB reads untruncated ascending streams and appends through a single writer using MessagePack payloads.
- H35-007 — L14: Rewind must never update/delete rows; it is a logical pointer move followed by a new beacon, preserving pre-target history.
- H35-008 — L14: The author-time schema snapshot is 220/497 and CLAUDE.md 191/464 is stale.
- H35-009 — L15: ReplaySessionId::derive hashes checkpoint plus canonical-JSON patch with blake3; ssb_resume_from is fail-closed through LockedKeysAuditor and returns PredicateUnavailable when the registry is absent.
- H35-010 — L15: Code restore emits AgenticObservabilityEventKind::ReplayResume via existing constructors, and a Tauri ssb_resume_from producer emits one such observability event per call.
- H35-011 — L16: recover_stubs fetches by stored SSB sequence, blake3-verifies payload integrity, and restores in place; the plan labels it production-wired.
- H35-012 — L16: There is no replay_resume function in session_dag.rs; ReplayResume is the SSB action/kind and earlier wording to the contrary is corrected.
- H35-013 — L17: RRA build reads the ascending stream; root and event hashes are recomputed with blake3; verdicts are Verified, Divergence@event, or Tampered.
- H35-014 — L17: Run-end RRA sealing is default-on through emit_run_artifact and has kill-switch ELAI_RRA_DISABLE; the RRA is the off-agent proof after restore.
- H35-015 — L18: Existing CLI Rra/Replay action/parser/driver paths are the extension template; elai replay --verify means honest divergence verification, not bit-identical model-token replay.
- H35-016 — L19: compact_session is the existing narrative compactor; summarize mode folds the truncated tail into one summary and frees local context.
- H35-017 — L20: CommandWindow is the command surface, CanvasWindow the canvas, and RraViewerPanel the adjacent place for rewind UI.
- H35-018 — L21: The exact production gap is target selection, three-way scope choice, orchestration of replay/recovery/compaction, and one audit row.
- H35-019 — L23-25: Tier-1 CLI is elai rewind <target> with optional scope and summarize; targets support raw SSB sequence, nth user turn via --turn, or prior RRA label via --checkpoint, all resolved over the ascending stream.
- H35-020 — L26: Conversation rewind drops messages after target, optionally summarizes that tail, and rehydrates every stub at or before target.
- H35-021 — L27: Code rewind derives a replay session from target checkpoint plus empty patch and calls fail-closed ssb_resume_from to reconstruct file state.
- H35-022 — L28: Both-scope order is conversation trim first, code restore second.
- H35-023 — L29: Every effective successful rewind emits exactly one Pattern-2 SessionRewound as the off-agent done-gate.
- H35-024 — L30: target==current head is an idempotent no-op, recognized by an empty-tail blake3 fingerprint, and emits zero rows.
- H35-025 — L31: Each transcript turn gets a CommandWindow "Rewind to here" control that invokes rewind_session(session_id,target_seq,scope,summarize), drives the same dispatcher, and emits the same row as CLI.
- H35-026 — L32: Tier-2 defers retaining both branch timelines, multi-checkpoint diff preview, and cloud-model code restore when RRA guarantee is VerifyDivergence; cloud restore is best-effort/flagged rather than proven.
- H35-027 — L32: Rewind injects no prompt content by default; only summarize changes the window and it shrinks it; append-only history needs no supersession narrative.
- H35-028 — L34-35: Add rewind_cmd.rs, CliAction::Rewind including target/scope/summarize/output_format, RewindScope Conversation/Code/Both, parser, and main dispatch by mirroring RRA/Replay.
- H35-029 — L36: The dispatcher consumes ascending query, compact_session, recover_stubs, ReplaySessionId::derive, ssb_resume_from, and ReplayOutcome; reimplementation of any is forbidden.
- H35-030 — L37: A new commands_h35.rs Tauri command must be registered and call the same dispatcher as CLI so both share one evidence gate.
- H35-031 — L38: SessionRewoundPayload fields are session_id, target_seq, head_seq_before, snake_case scope, summarized, optional replay_session_id, replay_outcome string, and tail_fingerprint_blake3.
- H35-032 — L38: The source's authored bump is 220->221 and 497->498 using append-only enum/array/string/parser lockstep, but live constants must be confirmed at implementation.
- H35-033 — L39: Rewind is default-on behind opt-out ELAI_REWIND_DISABLE, not an enable flag or wire-dark allowlist row; code restore mutates the working tree but LockedKeysAuditor must refuse frozen-surface changes.
- H35-034 — L41-44: SessionRewound is always produced for effective scope; code/both additionally produce replay_resume transitively; real entrypoints are CLI and CommandWindow Tauri, with seeded both-scope fixture as the trigger.
- H35-035 — L45: No TODO/placeholder restore or hardcoded Accepted is allowed; the dispatcher must propagate Refused, PredicateUnavailable, or Accepted ReplayOutcome, use Result rather than unwrap, and actually call ssb_resume_from.
- H35-036 — L46: The live done gate requires exactly one row with matching target/head plus post-rewind build_rra/verify_rra == Verified under fixed ELAI_SEED; missing row or non-Verified fails.
- H35-037 — L46: The smoke manifest must distinguish a correct no-op's zero rows from a dead code path.
- H35-038 — L48-52: Byte-identical replay is only a design claim until the done gate proves it; the event contains sequences, hashes, and scope but hard-excludes transcript/message contents, persona, and PII.
- H35-039 — L54-58: H-35 composes G-2 RRA verification, H-33 local 25-32K window governance, E-25 NarrativeSummary, and C-44/D-9 LockedKeysAuditor; restore cannot route around frozen surfaces.

### 9. H-46_clade_metaproductivity_selection.md — 174/174 lines

- H46-001 — L1-4: H-46 is PLANNED Track-7 Wave-7, with Tier-1 in-process clade aggregation/selection/evidence and a separately gated Tier-2 multi-generation measurement.
- H46-002 — L5-14: Current parent selection uses sigmoid_perf_score(immediate) times exploration bonus; the planned key aggregates measured descendant success so productive lineages beat high-scoring evolutionary dead ends.
- H46-003 — L7-14: The Metaproductivity-Performance Mismatch and HGM results are external evidence; ELAI must compose config_archive and H-43 deltas, not redesign the archive.
- H46-004 — L10-14: Cited paper outcomes are SWE-Verified 56.7 vs 53.3, Polyglot 30.5 vs 27.1, and about 2.38x less compute (517 vs 1,231 CPU hours); these are never ELAI measurements.
- H46-005 — L16-21: Estimated Tier-1 effort is one M session/6-8 hours; H-43 execution-grounded per-node deltas are a hard prerequisite, D-5 composes orthogonally, and H-45/H-47 are parallel-safe.
- H46-006 — L25-36: CMP walks the archive's existing parent/child tree and aggregates descendant benchmark deltas; archive uniqueness, capacity, and ancestor protection remain intact.
- H46-007 — L33-36: CMP must aggregate H-43 execution-grounded fitness under a cross-family verifier, never config-byte hashes; a node without descendants uses ImmediateScoreFallback.
- H46-008 — L38-43: sigmoid score, exploration bonus, select_parent/select_parent_with, insertion, child counts, and ancestor protection are shipped; H-43 and D-5/D-29 are planned; the CMP aggregator itself is absent.
- H46-009 — L45-51: Mandatory reads include verification DG/PC/VD/schema/citations, archive selection source, H-43 execution fitness, the dossier, HGM paper/repo, and deterministic/no-unwrap hard rules.
- H46-010 — L53-58: Create deterministic clade_metaproductivity over archive tree and per-node H-43 deltas, exposing a CladeScore of cmp, clade_size, and immediate_score with no wall-clock input.
- H46-011 — L60-62: select_parent and select_parent_with use CMP when a measured clade exists while retaining the existing exploration bonus and existing archive election machinery.
- H46-012 — L64-66: clade_size==0 uses sigmoid immediate selection to avoid starving cold-start leaves; CMP overrides only after lineage evidence exists.
- H46-013 — L68-70: Emit CladeSelected with node_id, cmp, clade_size, immediate_score, key_used, and seed; kinds are CladeMetaproductivity and ImmediateScoreFallback.
- H46-014 — L68-70: The authored schema snapshot is 224/501; implementation uses live baseline+1 and reconciles e45 if this is the first Track-7 bump.
- H46-015 — L72-74: H-46-decisions records mismatch, CMP/fallback reasoning, paper-vs-ELAI attribution, and honest dependence on both fitness quality and verifier gap.
- H46-016 — L76-85: Produced artifacts are clade_metaproductivity, CladeScore, and CladeSelected; consumed artifacts are config_archive selection/sigmoid, H-43 execution_grounded_fitness, and SAFEDELTA measured-row pattern.
- H46-017 — L86-87: Production reachability is run_one_generation -> config_archive::select_parent -> clade_metaproductivity -> CladeSelected under the existing Darwinian kill switch/allowlist.
- H46-018 — L88-95: Stub bans reject immediate-only selection when clade evidence exists, byte-hash CMP, and non-determinism; the broken --select-immediate-score arm must choose the dead end, and dashboard must show CMP/immediate/lineage.
- H46-019 — L96-101: The fixed archive fixture has A with high immediate/regressing descendants and B with mediocre immediate/improving descendants; CMP must select B, a fresh leaf must fallback, and broken immediate selection must select A.
- H46-020 — L99-101: CladeSelected must show cmp(B)>cmp(A) despite immediate(A)>immediate(B) and be recomputable under ELAI_SEED.
- H46-021 — L103-109: Deterministic checks require run-trigger, production source references to CMP/row, preserved select_parent, and the decision file.
- H46-022 — L111-121: Acceptance requires default CMP under the existing Darwinian kill switch with no new enable gate, live selection reachability, execution deltas, archive reuse, seeded byte-identical aggregation, broken-arm falsifiability, Tier-1/Tier-2 separation, workspace/auditor green.
- H46-023 — L123-134: Done also requires cross-language dashboard pairing, real journal evidence from a frozen fixture, dashboard visibility, Pattern-2/e45 handling, and explicit GD-1/GD-2/VD-1/VD-4 compliance.
- H46-024 — L136-141: Contract cases compare A/B CMP, live selected parent, no-lineage fallback, broken immediate arm, and two identical seeded selections over a frozen archive tree/delta fixture.
- H46-025 — L143-149: Planned lifecycle creates the aggregator, contract, fixture, and decision record; modifies archive selection, SSB/golden/e45, and dashboard/types.
- H46-026 — L151-157: HGM/DGM/Solver-Verifier research is grounding; the execution-fitness symbol is planned H-43 input, and paper compute must be attributed under HD-1.
- H46-027 — L159-162: Early-run leaves make fallback dominant; ELAI benefit is not proven by Tier-1 and belongs to the Tier-2 multi-generation measurement.
- H46-028 — L163-165: Cloud/70B HGM results do not establish benefit for ELAI's local 7B lane; measure at least three runs before a lift claim and never rebrand the 2.38x paper number.
- H46-029 — L166-167: Sum versus mean versus decay is an open design choice; implementation must select HGM's documented aggregate, freeze it, and ablate it against immediate selection.
- H46-030 — L169-172: H-48's BAI arm sampling is orthogonal to H-46 parent selection; together they form the intended regret-bounded selection, reusing D-5/D-29 and optionally ExIt curriculum.

## Explicit cross-source interactions

- IX-001 — H-GROUNDING L178-190 + H-9 L64-100: H-9 closes writer -> CloudSpend -> SsbViewer/get_cloud_spend -> CloudPanel/token component and cost collector -> metrics capture; H-1 hit rate and H-3 token ratio are explicit upstream inputs.
- IX-002 — H-9 L17-20,187-190: H-9 is both H-1/H-2/H-3's falsifiability instrument and the spend dependency later consumed by H-38 stop confirmation.
- IX-003 — H-GROUNDING L220-235 + H-11 L54-99: H-11 composes the existing project commands, path-keyed focus bus, Tree, and persisted panel split into a new live-event overlay and root-scoped explorer.
- IX-004 — H-11 L13-18,54-60,181-183: H-10 must supply the Canvas shell/LOD hierarchy first; H-11 owns CanvasActivityEvent before H-13; H-12 must enforce the single-consumer handover.
- IX-005 — H-20 L56-100: The leakage route is typed split declaration -> taint/DKG reachability COMPOSITE plus missing-lag AST -> pre-tool or promotion decision -> LeakageBlocked/drawer.
- IX-006 — H-20 L21,33-34,167-179: H-20 catches named static leakage classes, H-21 is the runtime causality backstop, and H-22 adds trading null relations; none substitutes for the others.
- IX-007 — H-28-WIRE-1 L25-56,80-95: The real Darwin flow is guard outcome Admitted -> optional H-28 interception -> reject evidence or archive insertion/admit evidence; the default-off flag and allowlist make the activation explicitly reversible.
- IX-008 — H-35 L23-46: Rewind resolves a journal target, restores conversation and/or code in fixed order, emits one SessionRewound for an effective action, then rebuilds/verifies the existing RRA.
- IX-009 — H-35 L54-58: H-35 shares compact_session with H-33/E-25, RRA with G-2, and locked-key enforcement with C-44/D-9; it creates neither a compactor, proof format, nor lock system.
- IX-010 — H-VERIFICATION L44-59 + H-46 L53-101: H-43 execution deltas flow into CMP, archive selection consumes CMP while preserving fallback/exploration, and CladeSelected supplies measured evidence.
- IX-011 — H-VERIFICATION L67-89 + H-46 L169-172: H-46 changes which parent is expanded, H-48 changes bandit arm sampling, H-45 changes score aggregation, and H-39 changes abstention credit; these are explicitly orthogonal layers over H-43 correctness.
- IX-012 — All three grounding maps' schema blocks plus H-11/H-20/H-35/H-46 bump clauses: action-type work is append-only, nine-site/golden locked, ship-order cumulative, and must repair the stale e45 pin once rather than applying every source snapshot literally.
- IX-013 — H-GROUNDING L269-280, H-UX L31, H-VERIFICATION L57-59: SSB is the shared off-agent observability plane; new rows mirror established Pattern-2/single-writer/frozen-runner evidence rather than bespoke logging.
- IX-014 — H-GROUNDING L136-150 + H-9 L43-44: Cloud CumulativeTokenTracker can feed the precompaction ladder; H-9 makes its spend/token evidence visible while H-6 owns actual context lifecycle activation.

## Inferences (not explicit requirements)

- INF-001 — From H-28W-009 and H46-007: H-43's execution-grounded fitness becomes a shared trust root for both reward-jump interception and clade selection; corrupt fitness would bias both gates. Neither source explicitly mandates a joint integrity check.
- INF-002 — From H28W-006/008 and H46-006/017: Because H-28 can reject before archive insertion and CMP walks the archive, enabling H-28 changes the lineage population H-46 later scores. No source specifies a compensating selection policy.
- INF-003 — From HG-037 and H9-011: Attaching the writer may activate the daily-cap block as well as CloudSpend emission because both are described under the same optional-writer region; H-9 explicitly targets evidence, so the cap behavior must be checked rather than assumed.
- INF-004 — From H11-015/016: If FileExplorer calls only Rust read_dir/read_file commands, tauri-plugin-fs may be unnecessary; direct JavaScript reads make it necessary. The plan intentionally leaves that architecture conditional.
- INF-005 — From H35-023/024 and H11-005/006: Both UX features distinguish effective state changes from non-events (one row for effective rewind, zero for no-op; ghost/dropped evidence for unresolved activity), suggesting a shared "no silent gaps/no evidence spam" convention.
- INF-006 — From the dated 207/484, 220/497, and 224/501 snapshots: at least 17 action types landed between the earliest and latest maps. The sources do not enumerate that intervening ship order here, so this cannot be used to calculate today's constants.
- INF-007 — From H9-006 and H46-028: Both cost and self-improvement claims require configuration/lane-specific repeated measurement; paper/vendor figures are context, not promotable product claims.

## Ambiguity and conflict register

- AC-001 [time-local, resolved procedure] — H-GROUNDING says 207/484 (2026-06-03), UX says 220/497 (2026-06-04), and verification says 224/501 (2026-06-05); H-11/H-20/H-35/H-46 repeat the relevant author-time snapshot. These are not safe current literals. Every source converges on re-reading live constants and repairing e45 once.
- AC-002 [conflict] — H-GROUNDING L35/L145-149 says recover_stubs and recover_single_stub have zero production callers and the live CLI substitutes a hardcoded placeholder. H-35 L13-17 says the whole substrate, including recover_stubs, is PROD-WIRED. The one-day-newer H-35 claim may reflect drift or may conflate implemented/write-capable with invoked; only a live caller audit can resolve it.
- AC-003 [temporal conflict] — H-GROUNDING L55 says G-2 RRA is absent on 2026-06-03, while H-35 L17/L55 says RRA build/verify/run-end seal is production-wired on 2026-06-04. Treat H-35 as later authored state but verify live before relying.
- AC-004 [newer-state correction] — The orientation summary describes H-28 as BUILD/reuse with an input-tamper gate absent. The July H-28-WIRE-1 source says check_reward_tampering, evidence emission, CLI proof, and freeze are already built; only the Darwin caller is dark. Current work is activation unless its abort tripline fires.
- AC-005 [semantic ambiguity] — H-28W L48-51 maps expected and observed harness hashes to the same frozen-corpus blake3, yet says the harness-mismatch leg would fire if corpus blake3 drifts. Passing the same accessor/value to both arguments cannot detect drift; implementation needs distinct frozen-expected versus observed-current sources or must honestly declare that leg inert.
- AC-006 [identifier ambiguity] — H-28W L38-40 requires a reject/non-admit beacon but does not name its exact action type or existing emitter; only RewardTampering is fully named.
- AC-007 [recon-required] — H-28W L58-60 fixes semantics but leaves exact prior/current fitness, run-id, and corpus-hash field names unresolved; absence of a real pair is a mandatory abort, not freedom to invent fields.
- AC-008 [implementation fork] — H-20 must choose live G-1 taint if shipped or a leakage-scoped DKG pass if not. Effort, dependencies, and exact data representation differ materially.
- AC-009 [entrypoint ambiguity] — H-20 says PreToolUse / pre-promotion and elsewhere "before tool use and/or promotion"; it does not decide whether both hooks are mandatory or one is sufficient.
- AC-010 [policy boundary] — Missing shift behavior is block for unambiguous same-bar use and warn/Tier-3 for ambiguous cases; the plan supplies no formal ambiguity classifier.
- AC-011 [ownership ambiguity] — H-9 asks both the sole task-runner constructor and from_env/from_env_async to chain with_ssb_writer, but the exact writer ownership/signature change is not specified.
- AC-012 [deferred feeder] — H-9's redaction count is real if G-9 has shipped, otherwise an honest "not wired" label; the final producer is explicitly conditional.
- AC-013 [conditional dependency] — H-11 says to add/register tauri-plugin-fs only if Svelte reads directly, but its Files touched section lists the dependency/plugin modification unconditionally. Rust-only command architecture may remove that dependency.
- AC-014 [cross-plan dependency] — Ghost placement near a nearest resolvable ancestor depends on H-10's folder super-node/LOD hierarchy, whose exact API is not specified here.
- AC-015 [classification ambiguity] — A path outside the opened root may be a mountable monorepo sibling or a true external such as node_modules; the rule for deciding "mount affordance" versus "external badge" is not formalized.
- AC-016 [CLI grammar] — H-35 presents positional <target> while also offering --turn and --checkpoint target forms. Mutual exclusion, precedence, and whether positional target becomes optional are not defined.
- AC-017 [proof not yet established] — H-35 claims empty-patch replay to a target reconstructs exact code state and later explicitly says byte-identical replay is only proven when post-rewind RRA verifies. Treat it as an acceptance claim, not current capability.
- AC-018 [outcome semantics] — H-35 calls one SessionRewound per successful rewind but also requires replay outcomes Refused/PredicateUnavailable/Accepted in the beacon. It does not state whether a refused code/both operation is "successful" enough to emit SessionRewound.
- AC-019 [open algorithm] — H-46 leaves CMP aggregation as sum vs mean vs decayed and delegates to the HGM definition; the choice must be frozen and ablated before implementation is determinate.
- AC-020 [unspecified gate] — H-46 calls live multi-generation measurement a separately gated checkbox but does not name its flag, default, owner, expiry, or UI location.
- AC-021 [planned dependency] — H-46 names execution_fitness.rs/H-43 as input even though its own existing-state section calls H-43 PLANNED; the file/symbol may not exist until H-43 lands.
- AC-022 [lane wording] — H-9 says no env override but also requires the existing ELAI_GROUNDING_BACKEND=cloud lane selector. The intended invariant is no new H-9 enable flag, not that cloud selection is unnecessary.
- AC-023 [source-status caution] — Grounding maps label anchors live/authoritative as of June, while H-28-WIRE-1 is a July planning-time judgment with RECON_NEEDED identifiers. Recency does not replace the explicit live re-grep gates.
- AC-024 [external evidence limit] — Several dependency/API/paper details were verified at authoring time, but UX marks some arXiv/anchor items unconfirmed and verification marks two papers MEDIUM. None should be promoted to a current implementation fact without the mandated re-check.
- AC-025 [temporal conflict] — H-11 L3/L56-60 calls CanvasActivityEvent planned/new, while the next-day H-35 L38 cites concrete live ssb.rs locations for CanvasActivityEvent as a Pattern-2 precedent. The later source suggests the event schema landed, but says nothing about the explorer/overlay; live audit must split schema status from full H-11 status.
- AC-026 [temporal conflict] — H-46 L3/L41-43/L68-70 calls CMP and CladeSelected planned/absent, while the later H-28-WIRE-1 L26-31 says the Darwin arm already calls emit_clade_selected_row. The row emitter appears newer than H-46's snapshot, but that does not by itself prove CMP selection is live.
- AC-027 [temporal conflict] — H-VERIFICATION L26-31 and H-46 L19/L41 describe execution-grounded fitness as H-43 work not yet landed, while H-28-WIRE-1 L30-31/L45-46 calls the current candidate fitness execution-grounded and already in scope. This may be later implementation drift or loose terminology; verify the actual fitness producer before trusting the reward mapping.

## Summary additions and corrections

The orientation summary already captures each numbered plan's headline outcome. The following are the deep-source details that it omits, narrows, or must correct.

- SD-001 [addition, H-GROUNDING] — The cache baseline includes exact send-path split, two precise non-caching secondary caller classes, the harmless 16 catalog None values, two-only/local-only/empty-slot breakpoints, and model-id-specific prefix minima (HG-005..007, HG-022..025, HG-052..054).
- SD-002 [addition, H-GROUNDING] — H-3's stream loop is shared by cloud and local, on_chunk is wholly call-dark, DIFF always receives no editor, and B-20 edits ship through a separate structured route (HG-009..010, HG-024..025).
- SD-003 [addition, H-GROUNDING] — H-4 has 16 catalog definitions but only 12 convert/dispatch arms, six agent-invisible artifact arms, OAuth-specific beta loss, exact tool-search type ids/limits, and custom-gateway deferral constraints (HG-018..019, HG-026..027, HG-055..058).
- SD-004 [addition, H-GROUNDING] — H-5's precise batch lifecycle, endpoint, unsupported fields, 24-hour expiry, 29-day retention, 100k/256MB cap, and eligible versus noneligible lanes are absent from the short summary (HG-020, HG-028, HG-059..060).
- SD-005 [addition, H-GROUNDING] — H-6's six compaction tiers are estimators that do not mutate content, handoff is write-active/read-dark at 85%, and retrieval/stub recovery caller states are more specific than the summary (HG-011, HG-029..031).
- SD-006 [addition, H-GROUNDING] — H-7's route context includes hardcoded audit/budget values and layer_failed=1; HybridEscalationGate/confidence are test-only and RouteDecisionEvent has three kinds but no writer (HG-012, HG-032..034).
- SD-007 [addition, H-GROUNDING] — H-8's exact tier TTLs/capacity and PromptCache's FNV/disk/30s/5m behavior distinguish two caches more precisely than the summary (HG-013, HG-035..036).
- SD-008 [addition, H-GROUNDING] — H-10 has two disjoint graph stores, exact visualization caps, agent-IPC-only APIs, and hardcoded file attributes; H-16 has an exact live Orbit render/removal chain (HG-015, HG-040..042, HG-051).
- SD-009 [addition, H-GROUNDING] — H-13's media consumer emits seven rows with mock VLM while screenshot/taint producer is absent; provenance evidence is schema-complete but emitter-dark (HG-021, HG-047..050).
- SD-010 [correction, H-GROUNDING] — One-hour Anthropic cache TTL is GA with no beta header, Tool Search's before figure is about 77K rather than 72K, stable Sigma v4 does not exist, and Onlook also uses an iframe with opaque IDs (HG-052, HG-057, HG-061..064).
- SD-011 [addition, UX map] — Track-4 memory schema/lifecycle includes atomic soft-close supersession, eight fact types, evidence filtering, last-observed mutation, default lexical NLI, and no memory context slot (UX-003..010).
- SD-012 [addition, UX map] — Live STDIO MCP is eager while lazy loading hangs from a dark HTTP bridge; H-36 intervention is an explicit frontend no-op; H-37 is presence-only in task runner; H-38 stop lacks cost awareness (UX-012, UX-014..016).
- SD-013 [addition, UX map] — Odysseus-derived thresholds/fingerprints/deletion guard/union assembler/local-window correction and their reimplement-never-port boundary are summary-silent (UX-018..024).
- SD-014 [correction, UX map] — H-30 is a metric, not H-31's capture substrate; MemoryFact starts at line 138; several anchors and future-dated citations require re-grep/reverification (UX-025..027).
- SD-015 [addition, verification map] — Track 7's current placeholders are exact byte-hash and literal substring implementations, tau/k_m are unconfirmed, and selection is sigmoid*exploration over immediate score (VG-003..007).
- SD-016 [addition, verification map] — The stronger-than-generator/cross-pretraining-family rule, explicit sandbagging residue, exact action-type kinds/payload reuse, and conditional H-48 schema behavior exceed the summary (VG-018..030).
- SD-017 [addition, H-9] — The cockpit's statistical contract is a four-value per-configuration tuple with paired bootstrap CI and about a six-point noise floor, not merely a set of dashboard values (H9-006..007).
- SD-018 [addition, H-9] — TokenTrackerPanel/read command, honest unwired redaction label, explicit no-cloud empty state, no-new-action-type decision, exact smoke/contract, and file lifecycle are summary-silent (H9-013..015, H9-021..028).
- SD-019 [addition, H-9] — Writer attachment may also expose a previously dormant daily-cap branch and token tracking remains cloud-client-co-located/default-sidecar-absent (H9-008..009; see INF-003).
- SD-020 [addition, H-11] — CanvasActivityEvent's fallback fields, four touch kinds, short label, non-active-mode indicator, default-on follow toggle, three motion names, and 30-second ghost timeout are not in the summary (H11-005..006, H11-011..013).
- SD-021 [addition, H-11] — File nodes have path/kind/no-status, the explorer gets a persisted new grid column, and exact commands are read_dir/read_file/open_in_editor/mount_root (H11-014..016).
- SD-022 [addition, H-11] — Conditional plugin need, exact scoped permissions, symlink canonicalization/rejection, mountable sibling vs external state, no-double-consumer sequencing, and detailed contract cases extend the summary (H11-016, H11-023..031).
- SD-023 [addition, H-20] — Missing split declarations block, missing-lag ambiguity warns/Tier-3 while unambiguous use blocks, and the proof names file:line:symbol (H20-011..015).
- SD-024 [addition, H-20] — The suppression registry is frozen/gate-manifested, dataflow choice is a pre-implementation decision, PreToolUse versus promotion hook is unresolved, and zero non-blake3 hashes are statically checked (H20-010, H20-013, H20-018..023).
- SD-025 [addition, H-20] — Frozen fixtures cover seven cases, false-positive rate must be bounded on at least three real backtests, and H-21 must ship before a trustworthy class guarantee (H20-026, H20-029..031).
- SD-026 [correction, H-28W] — The later supplement changes H-28's delivery state from an absent/build gate to a built, CLI-proven, frozen, row-emitting guard with only its Darwin caller dark (H28W-001..007; AC-004).
- SD-027 [addition, H-28W] — It introduces ELAI_H28_REWARD_GATE_ENABLE default-off, a fully pinned prior/new/band/hash/run/seed mapping, before-insert placement, ON/OFF evidence semantics, and an abort if real rewards are unavailable (H28W-008..018).
- SD-028 [addition, H-28W] — The allowlist metadata (kill_switch, maintainer, 2026-10-01, plan id, smoke), five-piece atomic delivery, production-caller check, hard-candidate red team, and counter-moves are wholly summary-silent (H28W-019..025).
- SD-029 [ambiguity/correction, H-28W] — Mapping expected and observed hashes to the same corpus value makes the stated harness-drift leg questionable; the summary's broader harness-mismatch promise is not proven by this wire mapping (AC-005).
- SD-030 [addition, H-35] — Targets include raw seq, nth turn, and labeled RRA checkpoint; conversation restore precedes code restore; CLI has output_format; the exact eight-field payload and three ReplayOutcome strings are absent from the summary (H35-019..022, H35-028..035).
- SD-031 [addition, H-35] — The no-op fingerprint hashes tail triples, UI is per transcript turn, privacy hard-excludes message bytes/persona/PII, and cloud VerifyDivergence restore is only best-effort Tier-2 (H35-005, H35-024..027, H35-038).
- SD-032 [conflict, H-35] — The summary follows H-35 in calling restore primitives live, but H-GROUNDING explicitly calls recover_stubs production-dark; this must remain unresolved pending a live caller audit (AC-002).
- SD-033 [qualification, H-35] — Byte-identical replay is not a current fact; H-35 itself says the post-rewind RRA done gate must prove it (H35-036..038).
- SD-034 [addition, H-46] — CladeScore exposes cmp/clade_size/immediate_score, the evidence payload adds node/key/seed, and existing exploration bonus remains active alongside no-lineage sigmoid fallback (H46-010..014).
- SD-035 [addition, H-46] — Tier-1 fixture evidence and Tier-2 multi-generation measurement are separate, local-7B benefit requires at least three runs, and aggregate sum/mean/decay must follow HGM then be frozen/ablated (H46-001, H46-027..029).
- SD-036 [addition, H-46] — The exact A-dead-end/B-productive broken-arm fixture, dashboard visibility, file lifecycle, and no-new-enable-gate acceptance are more specific than the summary (H46-018..025).
- SD-037 [later-state correction, H-35 over H-11] — H-35's concrete CanvasActivityEvent precedent indicates at least H-11's event schema may have shipped after the H-11 plan was authored; the summary's blanket PLANNED state should be decomposed into schema versus UI/FS status (AC-025).
- SD-038 [later-state correction, H-28W over Track 7] — The July Darwin source reports an existing CladeSelected emitter and execution-grounded candidate fitness, newer than the June H-43/H-46 planned snapshots. The summary should not continue to label every H-43/H-46 primitive absent without a live status audit (AC-026..027).

## Dimension and lifecycle coverage

| Requested dimension | Where exhaustively captured |
|---|---|
| Capabilities, actions, surfaces | H9-013..018; H11-011..019; H20-011..019; H28W-004..011; H35-019..031; H46-010..017 |
| Workflows and states | HG-005..051; UX-003..017; VG-003..017; IX-001..014 |
| Schemas, data, lifecycle | HG-002..004/050; UX-002..010; VG-023..030; H11-011; H20-015; H35-007/031..032; H46-013..014 |
| Flags, defaults, configuration | HG-022/025/029/032; H9-022; H11-012/025; H20-024; H28W-011/019; H35-014/033; H46-017/022 |
| Integrations and events | IX-001..014 and all WIRING_CONTRACT-derived atoms |
| Permissions, security, privacy, compliance | H11-007/015..016/021/030; H20-004/013/020/024; H28W-013/020; H35-027/033/035/038 |
| Failure, fallback, retry, recovery | H11-005..006/012/023/027; H20-005/010/014/029; H28W-013..018/021..024; H35-009..012/020..026/035..037; H46-007/012/018..019. No generic retry/backoff protocol is specified by any assigned source. |
| Determinism, idempotency, concurrency | HG-029..031/045/048/051; UX-017/020; VG-018..022; H9-006/020; H11-009; H20-022/026; H35-005/007/013/024/036..037; H46-010/020/022/024. Concurrency statements are limited to spawn_blocking indexing and the SSB single-writer. |
| Observability and evidence | HG-034/037..039/050; UX-007/013..017; VG-009/024..029; H9-019..024; H11-020..027; H20-015/019..026; H28W-004/014/017..020; H35-023/034..038; H46-013/017..024 |
| Tests, acceptance, done gates | H9-019..025; H11-022..028; H20-021..027; H28W-014..025; H35-034..037; H46-019..025 |
| Constraints, non-goals, dependencies, status, deferred | Every plan's opening/status and closing risks/next atoms; especially H20-005..010/029..031, H28W-001..003/013, H35-003/026..027/039, H46-001/005/027..030 |

## Zero-omission and duplicate audit

| Source | Lines read | Explicit atoms | Coverage audit |
|---|---:|---:|---|
| H-GROUNDING-MAP.md | 374/374 | 64 | Canonical constants, C1-C17 corrections, H-1..H-16 symbol map, all five web-verification blocks, and dependency blockers represented. |
| H-UX-MEMORY-GROUNDING-MAP.md | 48/48 | 27 | Header/provenance, constants, memory substrate, QOL substrate, evaluator pattern, every Odysseus bullet, and all confirm-at-authoring cautions represented. |
| H-VERIFICATION-GROUNDING-MAP.md | 163/163 | 32 | Meta-finding, DG-1..DG-7, PC-1..PC-8, VD-1..VD-5, schema/event block, and citation confidence/status represented. |
| H-9_cost_cockpit_dashboard.md | 192/192 | 29 | Status/objective, state, tasks, complete wiring contract, trigger/check/acceptance/done gate/tests/files, evidence, risks, and next represented. |
| H-11_live_layer_file_explorer_fs.md | 185/185 | 31 | Status/objective, state, all six tasks, transport/payload, UI/FS/security, complete wiring/trigger/check/acceptance/tests/files, risks, and next represented. |
| H-20_leakage_taint_composite.md | 181/181 | 31 | Status/objective/scope, state, all six tasks, dataflow/schema/allowlist, complete wiring/trigger/check/acceptance/tests/files, risks, and next represented. |
| H-28-WIRE-1_reward_gate_darwin_admit.md | 124/124 | 25 | Scope/proof spine, every grounding item, both moves, fixed mapping, recon/abort/on-fail/forks, verification/smoke, allowlist, all red-team attacks, and blind self-check represented. |
| H-35_rewind_over_ssb_journal.md | 58/58 | 39 | Premise/prior art, every ground-truth subsystem, complete Tier-1/Tier-2 scope, CLI/UI/schema/flag/wiring/done gate/privacy, and all composition edges represented. |
| H-46_clade_metaproductivity_selection.md | 174/174 | 30 | Status/objective/context/state, all five tasks, complete wiring/trigger/check/acceptance/tests/files, citations, every risk/open choice, and next represented. |
| **Total** | **1,499/1,499** | **308** | **9/9 exact sources; zero unread source lines. Blank lines, separators, headings, and repeated restatements do not create extra atoms.** |

Mechanical ID audit scope is every bullet beginning with an atomic prefix. Expected registry counts after finalization: 308 explicit source atoms + 14 explicit interaction atoms + 7 inference atoms + 27 ambiguity/conflict atoms + 38 summary-delta atoms = 394 unique atomic IDs. Duplicate identifiers: expected 0. Duplicate semantic restatements within a source: consolidated; cross-source corroborations remain intentionally distinct evidence occurrences and are not double-counted in the 308-source total.

Output body SHA-256 (all bytes before this line, including the preceding newline): 2c2ca961282f49449828df63aa7371df480b107757fe02a3f1ecfcec7eb3bd3d
