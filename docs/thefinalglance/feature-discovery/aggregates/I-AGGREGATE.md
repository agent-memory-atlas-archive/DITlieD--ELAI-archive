# Family I aggregate — ELAI deep feature discovery

## Aggregate metadata

- Family: **I** (54-session feature-engineering plan over wiring bedrock: Tracks 12–20 — fault isolation, evidence-ranked search, cache economics, deterministic gates, routing/budgets, open-weight flywheel, deterministic-over-prompt, design critique/trust claims, competitor-steal ports)
- Inputs merged: DEEP-I-001 through DEEP-I-006 and `09-I-SUMMARY_INDEX.discovery.md`
- Rule: every distinct atomic record is preserved with its ID and source anchor
- Namespace: ledger-local IX/SD/AC/CON/INF IDs are written as `I-00N/<ID>`; summary atoms as `SUMMARY/<ID>`
- Source-plan atoms keep their ledger IDs under that namespace
- Newer-plan precedence: explicit SUPERSEDED/replacement/CORRECT marker in a source wins; else higher plan number wins **target state**; losers retained under Superseded notes. Deep ledgers supersede the summary orientation pass where they conflict.
- Status labels (BUILD/EXTENSION/ACTIVATION/PLANNED/SHIPPED/WIRE-DARK/ABSENT) are plan-recorded claims unless noted; deep passes did not independently re-verify live code
- Generated: 2026-08-12 (I-family aggregator pass)

## Provenance

| Ledger | Source path | Declared (footer) | Extracted atoms |
|---|---|---|---:|
| I-001 | `docs/thefinalglance/feature-discovery/deep/DEEP-I-001.discovery.md` | 348 primary + 20 IX + 53 SD + 14 AC = 435 | 435 |
| I-002 | `docs/thefinalglance/feature-discovery/deep/DEEP-I-002.discovery.md` | 588 primary + 20 IX + 46 SD = 654 | 654 |
| I-003 | `docs/thefinalglance/feature-discovery/deep/DEEP-I-003.discovery.md` | 367 primary + 12 IX + 17 SD + 19 REG = 415 | 415 |
| I-004 | `docs/thefinalglance/feature-discovery/deep/DEEP-I-004.discovery.md` | 351 primary + 30 IX + 64 SD + 24 AC = 469 | 469 |
| I-005 | `docs/thefinalglance/feature-discovery/deep/DEEP-I-005.discovery.md` | 380 unique finding defs (333 E + 47 non-E; INT/SD tables are ID-less cross-refs) | 380 |
| I-006 | `docs/thefinalglance/feature-discovery/deep/DEEP-I-006.discovery.md` | 339 primary + 28 IX + 64 SD + 23 AC = 454 | 454 |
| SUMMARY | `docs/thefinalglance/feature-discovery/09-I-SUMMARY_INDEX.discovery.md` | 1001 session + 21 CROSS = 1022 inventory atoms | 1022 |
| **Extracted total** | — | — | **3829** |

## Family map (session inventory)

| Track | Sessions | Deep ledger owners (primary) |
|---|---|---|
| Track 12 Fault isolation | I-1..I-3 | I-003 (I-1), I-004 (I-2,I-3) |
| Track 13 Evidence-ranked search | I-4..I-9 | I-003 (I-4,I-9), I-002 (I-5), I-005 (I-6,I-7), I-001 (I-8) |
| Track 14 Cache economics | I-10..I-16 | I-002 (I-10), I-005 (I-11,I-14), I-004 (I-12), I-001 (I-13,I-15), I-006 (I-16) |
| Track 15 Deterministic gates | I-17..I-24 | I-006 (I-17,I-19,I-24), I-001 (I-18,I-23), I-003 (I-20), I-004 (I-21), I-005 (I-22) |
| Track 16 Routing/budgets | I-25..I-29 | I-005 (I-25), I-006 (I-26), I-004 (I-27), I-001 (I-28), I-002 (I-29) |
| Track 17 Open-weight flywheel | I-30..I-31 | I-006 (I-30), I-005 (I-31) |
| Track 18 Deterministic-over-prompt | I-32..I-40 | I-003 (I-32,I-33,I-38), I-004 (I-34,I-35,I-40), I-001 (I-36,I-37), I-005 (I-39) |
| Track 19 Design critique / trust claims | I-41..I-44 | I-003 (I-41), I-004 (I-42), I-006 (I-43), I-005 (I-44) |
| Track 20 Competitor-steal ports | I-45..I-54 | I-004 (I-45), I-002 (I-46,I-54), I-006 (I-47,I-49,I-53), I-001 (I-48), I-003 (I-50,I-52), I-005 (I-51) |
| Index / grounding | I-0 + maps | I-001 (I-GROUNDING-MAP), I-002 (I-0, I-STEAL-GROUNDING-MAP) |
| Summary orientation | all | SUMMARY |

## Notation

- Namespaced ID form: `I-00N/<local-id>` or `SUMMARY/<local-id>`.
- Cross-file interactions, summary deltas, and ambiguity/conflict registers keep local IDs under their ledger namespace (e.g. `I-001/IX-003`, `I-004/SD-I21-005`, `I-006/AC-003`).
- Kind tags: `[SRC]` source explicit/feature, `[IX]` cross-source interaction, `[SD]` summary delta, `[AC]` ambiguity, `[CON]` conflict/correction, `[INF]` inference (not a requirement).
- `SURFACE` marks user-facing windows, panels, commands, CLI verbs, flags, status displays, notifications (feeds dashboard DESIGN.md).
- Anchor form: `(ledger-file:L<line>)` points at the deep-ledger definition line; unit and source anchors from the ledger text are preserved.

## User-facing surfaces

Every atom that names a window, panel, drawer, command, CLI verb, flag, status display, or notification. Full text is retained in ledger sections below; this subsection indexes them by namespaced ID for dashboard DESIGN.md feed.

Surface-tagged atom count: **445**.

### Surfaces — CROSS (2)

- **SUMMARY/I.CROSS.08** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L61| unit=CROSS SURFACE): [E] Paper numbers are not adopted as product thresholds. ELAI measures its own baselines and records losing or non-monotonic results honestly.
- **SUMMARY/I.CROSS.14** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L71| unit=CROSS SURFACE): [E] Track 14 inherits the local window-minimization versus cloud cache-stability split rather than reopening it.

### Surfaces — I-0 (13)

- **I-002/I002-IDX-006** [CON] (DEEP-I-002.discovery.md:L86| unit=I-0 SURFACE): [E][C][non-goal] A wholesale gajae provider-layer port is rejected because B-37 plus CLI bridges/OAuth/local serving is broader. Anchors: lines 15-17, 151-165.
- **I-002/I002-IDX-023** [SRC] (DEEP-I-002.discovery.md:L106| unit=I-0 SURFACE): [E][constraint] Every Track-19 LLM critique surface is advisory and never load-bearing; only the off-agent spine decides DONE. Anchor: lines 136-149.
- **I-002/I002-IDX-032** [SRC] (DEEP-I-002.discovery.md:L121| unit=I-0 SURFACE): [E][UI][done-gate] W-4 requires `frontend_pair` and `cross_lang_diff` PASS for dashboard signals. Anchor: line 197.
- **I-002/I002-IDX-034** [SRC] (DEEP-I-002.discovery.md:L123| unit=I-0 SURFACE): [E][UI][done-gate] W-6 makes the dashboard surface part of done unless its absence is justified. Anchor: line 199.
- **I-002/I002-IDX-055** [SRC] (DEEP-I-002.discovery.md:L144| unit=I-0 SURFACE): [E][done-gate] COMPLETE requires workspace cargo check, auditor PASS, live wiring-contract PASS, applicable cross-language PASS, shipped/justified dashboard, applicable GD-4 bound, and delegated `memory-writer`. Anchors: lines 251-258.
- **I-002/I002-IDX-099** [SRC] (DEEP-I-002.discovery.md:L191| unit=I-0 SURFACE): [E][I-12][surface] Curated content exports to `CLAUDE.md`, `AGENTS.md`, and skills. Anchor: line 296.
- **I-002/I002-IDX-155** [SRC] (DEEP-I-002.discovery.md:L247| unit=I-0 SURFACE): [E][I-27][UI] It exposes a Pareto cost/accuracy cockpit tile. Anchor: line 342.
- **I-002/I002-IDX-196** [SRC] (DEEP-I-002.discovery.md:L288| unit=I-0 SURFACE): [E][I-37][UI] The cockpit exposes per-requirement verified/unverified state. Anchors: lines 377-378.
- **I-002/I002-IDX-202** [SRC] (DEEP-I-002.discovery.md:L294| unit=I-0 SURFACE): [E][I-39][status] `elai plan status` derives plan state from the journal and wiring fragments. Anchors: lines 382-384.
- **I-002/I002-IDX-212** [SRC] (DEEP-I-002.discovery.md:L304| unit=I-0 SURFACE): [E][I-41][surface] It integrates with the B-40 idea-critic loop and `elai critique`. Anchor: line 392.
- **I-002/I002-IDX-340** [SRC] (DEEP-I-002.discovery.md:L440| unit=I-0 SURFACE): [E][security] Grammars are pinned control-plane artifacts; third-party grammars and untrusted enum-bearing schemas are rejected because constrained decoding is an attack surface. Anchors: lines 565-567.
- **I-002/I002-IDX-346** [SRC] (DEEP-I-002.discovery.md:L446| unit=I-0 SURFACE): [E][product] Evidence/cockpit work targets review-time bottlenecks, not raw generation speed. Anchors: lines 577-578.
- **I-002/I002-IDX-349** [SRC] (DEEP-I-002.discovery.md:L449| unit=I-0 SURFACE): [E][next-step] The prescribed first command checks I-39, then I-10 follows as the economics keystone. Anchors: lines 595-601. I-0 ambiguity/conflict register:

### Surfaces — I-1 (4)

- **I-003/DI003-I01-016** [SRC] (DEEP-I-003.discovery.md:L81| unit=I-1 SURFACE): [E][deferred/security] The credential vault/key-management surface is a separate Tier-2 session. Anchor: I-1_directive_supervisor_enforcement_plane.md:42-45.
- **I-003/DI003-I01-018** [SRC] (DEEP-I-003.discovery.md:L85| unit=I-1 SURFACE): [E][surface] Create directive.rs, the I-1 contract, and I-1-decisions.md; modify task_runner dispatch and hook_dispatcher. Anchor: I-1_directive_supervisor_enforcement_plane.md:47-54.
- **I-003/DI003-I01-024** [SRC] (DEEP-I-003.discovery.md:L99| unit=I-1 SURFACE): [E][UI] DirectiveExecuted rows render in an existing SSB-backed drawer; no new Tier-1 panel is required. Anchor: I-1_directive_supervisor_enforcement_plane.md:55-56,83-86.
- **I-003/DI003-I01-032** [AC] (DEEP-I-003.discovery.md:L118| unit=I-1 SURFACE): [A][UI] The exact existing drawer is intentionally unresolved until EXECUTING. Anchor: I-1_directive_supervisor_enforcement_plane.md:55-56.

### Surfaces — I-10 (4)

- **I-002/I002-I10-026** [SRC] (DEEP-I-002.discovery.md:L533| unit=I-10 SURFACE): [E][UI] TTL/economics fields extend the H-9 cost tile in the CostDrawer/CachePanel family. Anchors: lines 50, 77-79.
- **I-002/I002-I10-032** [SRC] (DEEP-I-002.discovery.md:L539| unit=I-10 SURFACE): [E][UI][done-gate] The user-visible planner decision/guard signal makes a frontend pair mandatory. Anchors: lines 77-79.
- **I-002/I002-I10-033** [SRC] (DEEP-I-002.discovery.md:L540| unit=I-10 SURFACE): [E][trigger] The observable is a two-turn CLI run emitting SSB. Anchors: lines 80-82.
- **I-002/I002-I10-041** [AC] (DEEP-I-002.discovery.md:L551| unit=I-10 SURFACE): [A][UI] `CostDrawer` versus `CachePanel` is plan-asserted naming and must be re-grepped. Anchor: line 50.

### Surfaces — I-11 (1)

- **I-005/DII005-I11-021** [SRC] (DEEP-I-005.discovery.md:L120| unit=I-11 SURFACE): `[E][surface]` Add a required `BeliefState` inspector row in `SessionsDrawer`; `HonestyDelta` is a user-visible economics signal. — I11:L38, I11:L47, I11:L70-L71

### Surfaces — I-12 (7)

- **I-004/I12-E007** [SRC] (DEEP-I-004.discovery.md:L136| unit=I-12 SURFACE): E/SURFACE | `elai playbook export` writes verified entries in CLAUDE.md, AGENTS.md, and skills-compatible formats with embedded supporting evidence-row IDs. | anchor=I-12:L18-L20, L37-L39, L49-L49, L53-L56, L79-L82
- **I-004/I12-E013** [SRC] (DEEP-I-004.discovery.md:L142| unit=I-12 SURFACE): E/INT | The CLI subcommand is added under the PLAN-asserted `skills_cmd` family and must be re-grepped at execution. | anchor=I-12:L46-L50
- **I-004/I12-E016** [SRC] (DEEP-I-004.discovery.md:L145| unit=I-12 SURFACE): E/SURFACE | Evidence count, lift, and demotion events are user-actionable rows in the existing `RulesDrawer` playbook panel. | anchor=I-12:L49-L50, L77-L78
- **I-004/I12-E017** [SRC] (DEEP-I-004.discovery.md:L146| unit=I-12 SURFACE): E/INT | Qualifying production paths are CLI `export_verified()` and scheduler `run_promotion_tick()`; test-only/contracts callers do not establish R7 reachability. | anchor=I-12:L64-L67
- **I-004/SD-I12-002** [SD] (DEEP-I-004.discovery.md:L567| unit=I-12): **ADD** full `PlaybookEvent` kinds, required validation-before-promotion order, automatic stale/regression demotion, and exporter-as-read lifecycle boundary. | 09-I:L366-L384 | anchor=I12-E005, I12-E008, I12-E011, I12-E018–I12-E020, I12-I001
- **I-004/SD-I12-003** [SD] (DEEP-I-004.discovery.md:L568| unit=I-12 SURFACE): **ADD** exact module/production entrypoints, configurable Layer-1A schedule, event-driven RulesDrawer fields, and decision artifact. | 09-I:L376-L383 | anchor=I12-E012–I12-E017, I12-E026
- **I-004/SD-I12-006** [SD] (DEEP-I-004.discovery.md:L571| unit=I-12): **EXPAND** the summary’s `k` ambiguity to include schedule, lift statistics, and export block/collision schema. | 09-I:L385-L386 | anchor=I12-A001–I12-A004

### Surfaces — I-13 (6)

- **I-001/I13-E010** [SRC] (DEEP-I-001.discovery.md:L172| unit=I-13 SURFACE): E/DATA | ToolResult gains to_ui_dict for the full dashboard payload and to_llm_content for condensed model context. | anchor=I-13:L16-L18, L32-L34, L42-L45, L49-L54
- **I-001/I13-E011** [SRC] (DEEP-I-001.discovery.md:L173| unit=I-13 SURFACE): E/FLOW | Every production tool result entering context must pass through to_llm_content; adapters are updated while the full UI form remains intact. | anchor=I-13:L32-L34, L43-L45, L61-L70
- **I-001/I13-E012** [SRC] (DEEP-I-001.discovery.md:L174| unit=I-13 SURFACE): E/OBS | DualSerialized records the token delta between dashboard and context representations in the journal. | anchor=I-13:L32-L35, L47-L47, L76-L79
- **I-001/I13-E019** [SRC] (DEEP-I-001.discovery.md:L181| unit=I-13 SURFACE): E/SURFACE | MemoryFactsDrawer must show guard pattern class and serialization token delta; it is a required user-visible security/efficiency surface. | anchor=I-13:L47-L47, L74-L75
- **I-001/I13-E022** [SRC] (DEEP-I-001.discovery.md:L184| unit=I-13 SURFACE): E/TEST | A 500-file listing must remain complete in the dashboard while the model receives a condensed form and the journal records the token delta. | anchor=I-13:L76-L79
- **I-001/SD-I13-004** [SD] (DEEP-I-001.discovery.md:L551| unit=I-13 SURFACE): ADD negative-arm and full-vs-condensed 500-file artifact proof; FLAG unspecified FP threshold, path authorization, and pattern catalog. | anchor=I13-E021–I13-E025, I13-A001–I13-A003

### Surfaces — I-14 (1)

- **I-005/DII005-I14-021** [SRC] (DEEP-I-005.discovery.md:L155| unit=I-14 SURFACE): `[E][surface]` Per-transform receipt rows in the Transcript drawer are required and show token delta plus reversibility link. — I14:L52, I14:L77-L78

### Surfaces — I-15 (3)

- **I-001/I15-E012** [SRC] (DEEP-I-001.discovery.md:L211| unit=I-15 SURFACE): E/SURFACE | Research answers show a required cache-source badge with blake3_exact, semantic_vector, or miss. | anchor=I-15:L37-L37, L43-L47, L70-L71
- **I-001/I15-E014** [SRC] (DEEP-I-001.discovery.md:L213| unit=I-15 SURFACE): E/DEFER | Similarity-threshold auto-tuning is deferred until ELAI measures its own research-corpus precision/recall baseline. | anchor=I-15:L40-L41
- **I-001/SD-I15-004** [SD] (DEEP-I-001.discovery.md:L560| unit=I-15 SURFACE): ADD exact two-run/broken-arm evidence and FLAG undefined canonicalization/vector threshold. | anchor=I15-E016–I15-E020, I15-A002–I15-A003

### Surfaces — I-16 (12)

- **I-006/I16-E002** [SRC] (DEEP-I-006.discovery.md:L37| unit=I-16 SURFACE): E/GAP | The absent surfaces are an ELAI-fixture evaluator, a `KvCacheTuned` decision row, and the `elai bench kv-tune` entry point. | anchor=I-16:L6-L8
- **I-006/I16-E008** [SRC] (DEEP-I-006.discovery.md:L43| unit=I-16 SURFACE): E/EVIDENCE | SnapKV/ThinK paper results remain design evidence only; ELAI measurements control promotion, while W-10 requires the deterministic harness to ship even if the RunPod leg does not. | anchor=I-16:L18-L20, L64-L66, L83-L85
- **I-006/I16-E009** [SRC] (DEEP-I-006.discovery.md:L44| unit=I-16 SURFACE): E/DEP | The evaluator composes E-26/E-27/E-28 serving and FA2/`LogitsProcessor`, E-32 context-lift, B-37 transport for the live leg, and the `BenchLongcot` CLI/fixture pattern. | anchor=I-16:L22-L28, L79-L91
- **I-006/I16-E012** [SRC] (DEEP-I-006.discovery.md:L47| unit=I-16 SURFACE): E/CLI | Production CLI wiring is `CliAction::BenchKvTune`, `bench_kv_tune.rs`, and the `main.rs` dispatch arm; the command supports dry-run and `--emit-ssb`. | anchor=I-16:L35-L35, L43-L45, L54-L55
- **I-006/I16-E016** [SRC] (DEEP-I-006.discovery.md:L51| unit=I-16 SURFACE): E/UI | There is deliberately no frontend pair or dashboard: the product surface is the CLI plus metrics TOML. | anchor=I-16:L47-L47, L72-L73, L122-L124
- **I-006/I16-E017** [SRC] (DEEP-I-006.discovery.md:L52| unit=I-16 SURFACE): E/SYMBOL | The produced Python symbol is `python/elai_inference/eval_kv_tune.py::KvTuneEval`; the produced Rust/SSB symbols are the CLI dispatch and `KvCacheTuned`. | anchor=I-16:L49-L55
- **I-006/I16-E019** [SRC] (DEEP-I-006.discovery.md:L54| unit=I-16 SURFACE): E/REACH | Required production reachability is `cargo run -p elai-cli -- bench kv-tune --dry-run --emit-ssb` to the Python evaluator. | anchor=I-16:L60-L62, L74-L77
- **I-006/I16-E024** [SRC] (DEEP-I-006.discovery.md:L59| unit=I-16 SURFACE): E/OBS | The no-GPU command must emit `KvCacheTuned{vram_gb:null,tok_s:null,fixture_pass_rate:1.0,config_hash:blake3(...),adopt_decision:DryRun}` and the matching metrics file; `Adopt` with null VRAM fails. | anchor=I-16:L74-L77, L93-L99
- **I-006/I16-E025** [SRC] (DEEP-I-006.discovery.md:L60| unit=I-16 SURFACE): E/RECOVERY | Missing beacon, absent metrics, or silent FA2 disablement diagnoses unwired evaluator/CLI dispatch; the counter-move is to re-read the full `BenchLongcot` dispatch and Pattern-2 SSB lockstep, then rerun smoke. | anchor=I-16:L100-L102
- **I-006/I16-E028** [SRC] (DEEP-I-006.discovery.md:L63| unit=I-16 SURFACE): E/VERIFY | Done gates include plan-gate check, `cargo check -p elai-cli --all-targets`, orchestrator `action_type_lock`, the production CLI smoke, and `production_caller_check` on new dispatch symbols. | anchor=I-16:L111-L116
- **I-006/I16-E029** [SRC] (DEEP-I-006.discovery.md:L64| unit=I-16 SURFACE): E/REDTEAM | The red-team arms require failure for an Adopt row lacking live VRAM/pass-rate measurement and ban a duplicate-flag implementation that replaces shipped levers. | anchor=I-16:L118-L120
- **I-006/SD-I16-04** [SD] (DEEP-I-006.discovery.md:L532| unit=I-16 SURFACE): ADDITION | Exact CLI/module symbols, subprocess fallback, no-dashboard decision, schema auto-derivation, action-lock checks, and clean-tree cardinality gates were omitted. | anchor=I16-E012, I16-E015–I16-E020, I16-E026–I16-E030

### Surfaces — I-17 (2)

- **I-006/I17-E023** [SRC] (DEEP-I-006.discovery.md:L99| unit=I-17 SURFACE): E/UI | Existing cockpit branch cards gain verdict chips sourced from SSB viewer/subscription wiring; no new drawer is allowed and cross-language diff must pass. | anchor=I-17:L51-L52, L76-L79
- **I-006/SD-I17-04** [SD] (DEEP-I-006.discovery.md:L544| unit=I-17 SURFACE): ADDITION | `DisagreementEscalated`, exact production callers, no-new-drawer IPC behavior, no-soft-veto/no-copy bans, and mandatory full-form expansion were omitted. | anchor=I17-E009, I17-E015–I17-E023, I17-E027

### Surfaces — I-18 (9)

- **I-001/I18-E015** [SRC] (DEEP-I-001.discovery.md:L247| unit=I-18 SURFACE): E/CLI | elai verify-attestation --chain walks the chain and reports the first broken link and exact sequence number. | anchor=I-18:L41-L43, L49-L55, L70-L73
- **I-001/I18-E019** [SRC] (DEEP-I-001.discovery.md:L251| unit=I-18 SURFACE): E/INT | Changes target ssb.rs single-writer insert, rra.rs build/verify/CLI chain leg, attestation.rs AI-BOM emission, and the CLI verify subcommand; G-2/G-8/F-4 are composed, not redefined. | anchor=I-18:L49-L55, L59-L73, L99-L103
- **I-001/I18-E020** [SRC] (DEEP-I-001.discovery.md:L252| unit=I-18 SURFACE): E/SURFACE | The existing RRA viewer gains a chain badge, not a new drawer; it subscribes to RowChainSealed and shows CHAIN OK or BROKEN plus break seq. | anchor=I-18:L56-L57, L80-L83
- **I-001/I18-E021** [SRC] (DEEP-I-001.discovery.md:L253| unit=I-18 SURFACE): E/TEST | cross_lang_diff.py must pass for the RRA-viewer row/badge extension. | anchor=I-18:L56-L57, L80-L83
- **I-001/I18-C001** [CON] (DEEP-I-001.discovery.md:L262| unit=I-18 SURFACE): C/DEFER | The orientation summary presents Rekor push and chain/Rekor CLI options together; the numbered source explicitly makes the push Tier 2 and only scaffolds --rekor now. | anchor=I-18:L44-L47; orientation:L529-L535
- **I-001/SD-I18-002** [SD] (DEEP-I-001.discovery.md:L567| unit=I-18 SURFACE): ADD exact RowChainSealed kinds, CLI first-break behavior, badge subscription/state, and complete negative row-edit/Cargo-lock tests. | anchor=I18-E015–I18-E024
- **I-001/SD-I18-003** [SD] (DEEP-I-001.discovery.md:L568| unit=I-18 SURFACE): QUALIFY Rekor as separately deferred push with scaffolded UI/flag, approval/anti-slopsquat dependency choice, and fail-closed requested behavior. | anchor=I18-E011–I18-E014, I18-A001, I18-C001
- **I-001/SD-I18-004** [SD] (DEEP-I-001.discovery.md:L569| unit=I-18 SURFACE): ADD execution-time CycloneDX/Rekor verification requirements and FLAG schema-extension wording ambiguity. | anchor=I18-E009–I18-E010, I18-A002–I18-A003
- **I-001/SD-I18-005** [SD] (DEEP-I-001.discovery.md:L570| unit=I-18 SURFACE): REMOVE/FLAG the orientation-only I-45 coordination claim, which the numbered source does not support. | anchor=I18-C002

### Surfaces — I-19 (4)

- **I-006/I19-E017** [SRC] (DEEP-I-006.discovery.md:L135| unit=I-19 SURFACE): E/UI | TDD block reasons use the existing F-1 gate panel and IPC path, add no drawer/endpoint, and require cross-language diff. | anchor=I-19:L57-L59, L80-L83
- **I-006/I19-A002** [AC] (DEEP-I-006.discovery.md:L148| unit=I-19 SURFACE): A/LANGUAGE | TypeScript is in the parse/reachability scope, but no TypeScript syntax-only command is named alongside Rust/Python. | anchor=I-19:L11-L12, L16-L18, L36-L36, L71-L72
- **I-006/SD-I19-04** [SD] (DEEP-I-006.discovery.md:L554| unit=I-19 SURFACE): ADDITION | Exact predicate symbol, FP TOML, action-lock/cross-language/UI reuse, positive failing-test arm, and skip-preflight broken arm were omitted. | anchor=I19-E016–I19-E027
- **I-006/SD-I19-05** [SD] (DEEP-I-006.discovery.md:L555| unit=I-19 SURFACE): AMBIGUITY | The deep source reveals the non-blocking `RefactorExempted` discriminator under a “Blocked” action family; the summary reports it without flagging this consumer-contract ambiguity. | anchor=I19-A004

### Surfaces — I-2 (4)

- **I-004/I2-E015** [SRC] (DEEP-I-004.discovery.md:L51| unit=I-2 SURFACE): E/SURFACE | Existing C-23 `BranchTree` is extended with workspace badges from `Spawned`/`Merged`/`EscapeBlocked` rows through its existing Tauri read; no new branch UI is built, and `cross_lang_diff.py` must pass. | anchor=I-2:L29-L30, L60-L61, L74-L75, L91-L94, L119-L120
- **I-004/I2-E021** [SRC] (DEEP-I-004.discovery.md:L57| unit=I-2 SURFACE): E/DEP | E-15 Firecracker pool, C-11 sandbox seam, and C-23 BranchTree are PLAN-asserted shipped dependencies that must be re-grepped at execution and composed, not rebuilt. | anchor=I-2:L27-L30, L73-L75, L118-L120
- **I-004/I2-A004** [AC] (DEEP-I-004.discovery.md:L78| unit=I-2 SURFACE): A/SURFACE | `ConflictEscalated` is an event arm but is omitted from the explicit BranchTree badge arm list, leaving its exact operator-visible rendering unresolved. | anchor=I-2:L43-L45, L60-L61, L67-L68, L91-L94
- **I-004/SD-I2-003** [SD] (DEEP-I-004.discovery.md:L546| unit=I-2 SURFACE): **ADD** complete event family, pre-filesystem escape guarantee, rollback chain, missing-row hard failure, and unresolved conflict-event UI rendering. | 09-I:L136-L141 | anchor=I2-E010, I2-E013–I2-E018, I2-A004

### Surfaces — I-20 (3)

- **I-003/DI003-I20-022** [SRC] (DEEP-I-003.discovery.md:L392| unit=I-20 SURFACE): [E][deferred] Exhaustive mutation is an offline opt-in CLI flag only. Anchor: I-20_budgeted_mutation_scheduler.md:55-57.
- **I-003/DI003-I20-024** [SRC] (DEEP-I-003.discovery.md:L396| unit=I-20 SURFACE): [E][surface] MutationBudget wraps MutationRunner without changing its internals and is called by I-17 and the search ladder. Anchor: I-20_budgeted_mutation_scheduler.md:59-64.
- **I-003/DI003-I20-026** [SRC] (DEEP-I-003.discovery.md:L400| unit=I-20 SURFACE): [E][UI] An existing branch-board/audit pane gets a per-hunk chip showing CI, n, verdict, or survivor path; no new drawer is added. Anchor: I-20_budgeted_mutation_scheduler.md:66-67,92-95.

### Surfaces — I-21 (3)

- **I-004/I21-E027** [SRC] (DEEP-I-004.discovery.md:L197| unit=I-21 SURFACE): E/SURFACE | A Sprint Contract card beside `PlanApprovalCard` shows contract fields, criterion-determinism badges, verdict/objection, and loop count via existing SSB subscription; `cross_lang_diff.py` must pass. | anchor=I-21:L65-L67, L89-L90
- **I-004/I21-E030** [SRC] (DEEP-I-004.discovery.md:L200| unit=I-21 SURFACE): E/TEST | The acceptance trigger runs `elai run --sprint --task-fixture fixtures/vague_contract.toml --emit-ssb`. | anchor=I-21:L91-L92
- **I-004/SD-I21-007** [SD] (DEEP-I-004.discovery.md:L583| unit=I-21): **ADD** multi-contract and auto-criteria deferrals, full-form requirement, external-figure caution, and unresolved criterion resolver. | 09-I:L601-L624 | anchor=I21-E001–I21-E002, I21-E024–I21-E025, I21-A002

### Surfaces — I-22 (5)

- **I-005/DII005-I22-007** [SRC] (DEEP-I-005.discovery.md:L171| unit=I-22 SURFACE): `[E][measurement/default]` Reward weights are measured on ELAI fixtures and written by first deployment to `metrics/progress_reward_weights.toml`; no paper vector is a gate. — I22:L16-L20, I22:L50, I22:L71, I22:L84
- **I-005/DII005-I22-013** [SRC] (DEEP-I-005.discovery.md:L177| unit=I-22 SURFACE): `[E][config/measurement]` D is measured on the ELAI fixture corpus and must not be an unbounded or hard-coded source constant. — I22:L25-L28, I22:L49-L50, I22:L85
- **I-005/DII005-I22-016** [SRC] (DEEP-I-005.discovery.md:L180| unit=I-22 SURFACE): `[E][composition]` This per-branch search mechanism is a sibling of, and does not replace or interact with, the main-session anti-spiral CLI skill. — I22:L29-L31, I22:L39, I22:L103, I22:L112-L113
- **I-005/DII005-I22-022** [SRC] (DEEP-I-005.discovery.md:L186| unit=I-22 SURFACE): `[E][deferred]` Exponential/adaptive decay schedules remain deferred until ELAI measurements justify them. — I22:L54
- **I-005/DII005-I22-024** [SRC] (DEEP-I-005.discovery.md:L188| unit=I-22 SURFACE): `[E][surface]` The I-4 branch board renders a per-branch `phi` sparkline with decay regions distinguished, consuming existing SSB subscriptions; `cross_lang_diff` must pass. — I22:L63-L64, I22:L87-L91

### Surfaces — I-23 (4)

- **I-001/I23-E009** [SRC] (DEEP-I-001.discovery.md:L281| unit=I-23 SURFACE): E/CFG | The uncertainty threshold is measured on ELAI fixtures, persisted in metrics/calibration_fork_threshold.toml, and cannot be a paper constant. | anchor=I-23:L19-L21, L25-L28, L31-L33, L43-L48, L64-L68, L79-L84
- **I-001/I23-E015** [SRC] (DEEP-I-001.discovery.md:L287| unit=I-23 SURFACE): E/SURFACE | No new dashboard component is added; existing D-16 calibration/refusal tiles remain and I-4's existing branch board surfaces forks. | anchor=I-23:L60-L62, L85-L88
- **I-001/SD-I23-003** [SD] (DEEP-I-001.discovery.md:L578| unit=I-23 SURFACE): ADD local threshold artifact, divergent-hypothesis rule, no-new-UI details, and deferred multi-model tau support. | anchor=I23-E008–I23-E010, I23-E015–I23-E016
- **I-001/SD-I23-004** [SD] (DEEP-I-001.discovery.md:L579| unit=I-23 SURFACE): FLAG undefined CI-width calibration procedure and exact future I-4 seam. | anchor=I23-A001–I23-A002

### Surfaces — I-24 (4)

- **I-006/I24-E005** [SRC] (DEEP-I-006.discovery.md:L164| unit=I-24 SURFACE): E/INVARIANT | Transformations may alter surface style but not control flow, data flow, or type constraints. | anchor=I-24:L16-L20
- **I-006/I24-E017** [SRC] (DEEP-I-006.discovery.md:L176| unit=I-24 SURFACE): E/UI | The existing branch board/scoring panel gains `SCORER OK`, `BIAS DETECTED`, and `NEUTRALIZED` health states via SSB subscription; no new surface is created and cross-language diff must pass. | anchor=I-24:L66-L67, L92-L97
- **I-006/I24-E028** [SRC] (DEEP-I-006.discovery.md:L187| unit=I-24 SURFACE): E/TEST | Every new relation has an L1-verified semantic-equivalence test; the UI path requires cross-language diff. | anchor=I-24:L47-L50, L92-L97
- **I-006/SD-I24-06** [SD] (DEEP-I-006.discovery.md:L566| unit=I-24 SURFACE): AMBIGUITY | Signed rather than absolute score delta and the calibration/version lifecycle are unresolved in the source but not surfaced by the summary. | anchor=I24-A002–I24-A003

### Surfaces — I-25 (7)

- **I-005/DII005-I25-002** [SRC] (DEEP-I-005.discovery.md:L198| unit=I-25 SURFACE): `[E][learning]` Train routing on ELAI journal verifier outcomes keyed by task features, lane, and cost rather than human/proxy labels. — I25:L4-L8, I25:L10-L13
- **I-005/DII005-I25-015** [SRC] (DEEP-I-005.discovery.md:L211| unit=I-25 SURFACE): `[E][flag/default]` The existing emission remains default-off behind `ELAI_ROUTE_DECISION_SSB_ENABLE`. — I25:L27-L28, I25:L80-L82
- **I-005/DII005-I25-035** [SRC] (DEEP-I-005.discovery.md:L231| unit=I-25 SURFACE): `[E][deferred/UI]` Route-explanation dashboard/Svelte work is deferred by `owner-ui-rework-hold`; this unit emits backend details only. — I25:L55-L56, I25:L92-L93, I25:L177
- **I-005/DII005-I25-037** [SRC] (DEEP-I-005.discovery.md:L233| unit=I-25 SURFACE): `[E][reachability]` Production reachability is `elai run <task>` → task-runner pre-cloud path → `run_cost_router_preclassify` → `CostProfiler`. — I25:L80-L82
- **I-005/DII005-I25-044** [SRC] (DEEP-I-005.discovery.md:L240| unit=I-25 SURFACE): `[E][honesty]` Winning DeNovoSWE weights are unpublished, so ELAI calibrates weights on its own rollouts. — I25:L129-L131
- **I-005/DII005-I25-045** [SRC] (DEEP-I-005.discovery.md:L241| unit=I-25 SURFACE): `[E][cross-language adaptation]` Candidate ELAI structural substitutes are SCIP function count, cross-language bindings touched, and SSB action-type variants added per task. — I25:L132-L134
- **I-005/DII005-I25-047** [SRC] (DEEP-I-005.discovery.md:L243| unit=I-25 SURFACE): `[E][measurement constraint]` The cited +1.2 percentage-point result is rationale only and never an ELAI target. — I25:L135-L136

### Surfaces — I-26 (2)

- **I-006/I26-E019** [SRC] (DEEP-I-006.discovery.md:L218| unit=I-26 SURFACE): E/REACH | Production reach is `elai run <task>` → I-4 ladder → `BudgetController` at each node, with SSB rows as off-agent evidence. | anchor=I-26:L66-L68
- **I-006/I26-E021** [SRC] (DEEP-I-006.discovery.md:L220| unit=I-26 SURFACE): E/UI | Existing H-15/I-4 branch cards show a per-node remaining-budget bar from `depth_level`, `budget_remaining`, and `plateau_fired`; no new dashboard crate, and cross-language diff covers Tauri→Svelte. | anchor=I-26:L53-L53, L76-L77

### Surfaces — I-27 (8)

- **I-004/I27-E006** [SRC] (DEEP-I-004.discovery.md:L223| unit=I-27 SURFACE): E/SURFACE | A Pareto view overlays cost per additional verified task as a second axis so an operator can choose a cost/accuracy operating point. | anchor=I-27:L4-L7, L13-L15, L43-L47, L71-L72
- **I-004/I27-E011** [SRC] (DEEP-I-004.discovery.md:L228| unit=I-27 SURFACE): E/DEP | I-27 imports E-49 `$ / CTS` and `STABLE-Pass`, mirrors E-9’s fixture-runner shape, lands in H-9’s cockpit family, reads I-4 `SearchNodeEvent`, and shares I-38’s statistical method. | anchor=I-27:L22-L28, L43-L58, L79-L91
- **I-004/I27-E013** [SRC] (DEEP-I-004.discovery.md:L230| unit=I-27 SURFACE): E/CAP | Tier 1 creates `crates/cli/src/bench_convergence.rs::ConvergenceBench` for sweeping, aggregation, curve computation, and row emission. | anchor=I-27:L30-L36, L43-L52
- **I-004/I27-E014** [SRC] (DEEP-I-004.discovery.md:L231| unit=I-27 SURFACE): E/SURFACE | Tier 1 creates/extends a dashboard-v2 convergence tile in the H-9 cost cockpit, rendering both curve and Pareto view through a Tauri-to-Svelte pair checked by `cross_lang_diff.py`. | anchor=I-27:L30-L35, L43-L47, L49-L53, L71-L72
- **I-004/I27-E018** [SRC] (DEEP-I-004.discovery.md:L235| unit=I-27 SURFACE): E/SURFACE | Production CLI entry is `elai bench convergence --suite <fixture>`; the beacon is the off-agent artifact read by the cockpit. | anchor=I-27:L60-L63
- **I-004/I27-E021** [SRC] (DEEP-I-004.discovery.md:L238| unit=I-27 SURFACE): E/TEST | The acceptance run uses at least five seeds: `elai bench convergence --suite <fixture-set> --seeds 5`. | anchor=I-27:L74-L76
- **I-004/I27-E024** [SRC] (DEEP-I-004.discovery.md:L241| unit=I-27 SURFACE): E/INT | E-49 definitions, E-9 runner skeleton, H-9 cockpit, I-4 telemetry, and I-38 bootstrap must be imported/composed rather than redefined. | anchor=I-27:L54-L58, L86-L91
- **I-004/SD-I27-005** [SD] (DEEP-I-004.discovery.md:L593| unit=I-27 SURFACE): **FLAG** compute-step, statistic, knee, cost, and date-normalization choices absent from the summary and source. | 09-I:L738-L751 | anchor=I27-A001–I27-A004

### Surfaces — I-28 (9)

- **I-001/I28-E010** [SRC] (DEEP-I-001.discovery.md:L313| unit=I-28 SURFACE): E/CFG | The production path is default-off behind ELAI_DAG_CREDIT_ENABLE and also requires the search-ladder flag. | anchor=I-28:L19-L21, L32-L32, L47-L50
- **I-001/I28-E011** [SRC] (DEEP-I-001.discovery.md:L314| unit=I-28 SURFACE): E/OBS | With ELAI_SEARCH_LADDER_ENABLE=1 and ELAI_DAG_CREDIT_ENABLE=1 credit rows appear; with the credit flag unset there must be zero rows. | anchor=I-28:L47-L50, L76-L84, L110-L117
- **I-001/I28-E013** [SRC] (DEEP-I-001.discovery.md:L316| unit=I-28 SURFACE): E/TEST | Slice 1 adds crates/contracts/tests/i28_dag_credit_contract.rs covering CreditWalk, schema floor, sql_str round-trip, flag-on, and flag-unset paths. | anchor=I-28:L19-L21, L76-L84, L103-L108, L117-L117
- **I-001/I28-E023** [SRC] (DEEP-I-001.discovery.md:L326| unit=I-28 SURFACE): E/SURFACE | No dashboard work ships in this unit; SSB rows are the proof, and a credit-heat view is deferred until trajectory volume justifies it. | anchor=I-28:L61-L63
- **I-001/I28-E025** [SRC] (DEEP-I-001.discovery.md:L328| unit=I-28 SURFACE): E/FAIL | If both flags are enabled and no CreditWalk appears, dispatcher wiring is fixed and the test repeated; rows with the credit flag unset abort as an environment-gate leak. | anchor=I-28:L84-L84
- **I-001/I28-C005** [CON] (DEEP-I-001.discovery.md:L345| unit=I-28 SURFACE): C/SURFACE | The summary allows an optional plan-view heat map; the refreshed execution unit explicitly does no dashboard work and defers heat until trajectory volume justifies it. | anchor=I-28:L61-L63; orientation:L767-L767
- **I-001/SD-I28-003** [SD] (DEEP-I-001.discovery.md:L587| unit=I-28 SURFACE): ADD default-off flag, wire-dark promotion row, production-caller policy, contract/schema-roundtrip gates, and refreshed registry derivation. | anchor=I28-E010–I28-E015
- **I-001/SD-I28-006** [SD] (DEEP-I-001.discovery.md:L590| unit=I-28 SURFACE): CORRECT optional immediate heat UI to no-dashboard unit/deferred heat; ADD I-30 reader-or-wire-dark artifact rule. | anchor=I28-E019, I28-E023, I28-A002, I28-C005
- **I-001/SD-I28-008** [SD] (DEEP-I-001.discovery.md:L592| unit=I-28 SURFACE): FLAG unresolved DPO action shape and source-level temporal conflict with dated grounding constants. | anchor=I28-A004, I28-C006

### Surfaces — I-29 (3)

- **I-002/I002-I29-029** [SRC] (DEEP-I-002.discovery.md:L586| unit=I-29 SURFACE): [E][UI] Generation cards display a draft-lane badge with `DraftAccepted`, `DraftVetoed`, and `CrdtMerged`; `cross_lang_diff.py` covers Tauri-to-Svelte. Anchors: lines 60, 85-86.
- **I-002/I002-I29-036** [SRC] (DEEP-I-002.discovery.md:L593| unit=I-29 SURFACE): [E][trigger] Tier 1 uses the wiring-contract fixture harness; gated Tier 2 uses `elai run --diffusion-lane`. Anchors: lines 88-90.
- **I-002/I002-I29-041** [AC] (DEEP-I-002.discovery.md:L598| unit=I-29 SURFACE): [E][evidence][A] The speed claim is moderate/[DESIGN]/[WEAK] until Tier 2 lands and ELAI measures it; SSB planning is strong. Anchors: lines 93-100.

### Surfaces — I-3 (4)

- **I-004/I3-E020** [SRC] (DEEP-I-004.discovery.md:L108| unit=I-3 SURFACE): E/SURFACE | I-3 intentionally adds no live UI: its observable artifacts are the metrics TOML and crash-fixture result, with future cockpit consumption assigned to I-27. | anchor=I-3:L62-L64, L93-L96
- **I-004/I3-I001** [INF] (DEEP-I-004.discovery.md:L115| unit=I-3 SURFACE): I/DATA | Because I-27 is named as the future cockpit consumer, `metrics/phase_I3.toml` implicitly needs a stable machine-readable schema even though I-3 specifies only one field. | anchor=I-3:L60-L64, L93-L96, L97-L105
- **I-004/SD-I3-002** [SD] (DEEP-I-004.discovery.md:L556| unit=I-3 SURFACE): **ADD** exact `ELAI_IO_URING_ENABLE` flag, allowlist/promotion gate, and separately deferred multi-writer fanout after I-4 contention measurement. | 09-I:L155-L158 | anchor=I3-E008, I3-E018
- **I-004/SD-I3-005** [SD] (DEEP-I-004.discovery.md:L559| unit=I-3): **QUALIFY** summary status ambiguity with the source’s full unknowns: pragma values/sites, benchmark normalization, and cross-platform atomic-write details remain undefined. | 09-I:L163-L164 | anchor=I3-A001–I3-A004

### Surfaces — I-30 (12)

- **I-006/I30-E019** [SRC] (DEEP-I-006.discovery.md:L256| unit=I-30 SURFACE): E/CLI | Tier 1 creates `crates/cli/src/sft_exhaust.rs` and `elai dataset export --format sft`. | anchor=I-30:L44-L48, L59-L63
- **I-006/I30-E021** [SRC] (DEEP-I-006.discovery.md:L258| unit=I-30 SURFACE): E/UI | The dataset-card drawer is explicitly deferred under `owner-ui-rework-hold-2026-08-06`; only backend `SftExhaustEvent` ships in Tier 1. | anchor=I-30:L49-L51, L59-L64, L89-L90
- **I-006/I30-E027** [SRC] (DEEP-I-006.discovery.md:L264| unit=I-30 SURFACE): E/REACH | Production reach is the dataset-export CLI; SSB rows and JSONL are the off-agent artifacts. | anchor=I-30:L77-L79
- **I-006/I30-E033** [SRC] (DEEP-I-006.discovery.md:L270| unit=I-30 SURFACE): E/UI | Tier-1 implementation must not add the deferred UI drawer. | anchor=I-30:L89-L90, L173-L180
- **I-006/I30-E035** [SRC] (DEEP-I-006.discovery.md:L272| unit=I-30 SURFACE): E/EVIDENCE | J-11 amendment adopts difficulty-banded inclusion as design rationale from a cited DeNovoSWE ablation; its reported gain is never an ELAI performance target. | anchor=I-30:L114-L120
- **I-006/I30-E046** [SRC] (DEEP-I-006.discovery.md:L283| unit=I-30 SURFACE): E/ABORT | Abort on unsealed ingestion, DPO/PPO/GRPO logic, promotion bypass, fabricated values, non-blake3 hashing, production unwrap/expect, or Tier-1 UI. | anchor=I-30:L173-L180
- **I-006/I30-E050** [SRC] (DEEP-I-006.discovery.md:L287| unit=I-30 SURFACE): E/STATUS | No recon question is declared open, but execution must re-grep RRA, task runner, split SSB, CLI, gate-chain, and action-lock seams. | anchor=I-30:L198-L198
- **I-006/I30-C003** [CON] (DEEP-I-006.discovery.md:L290| unit=I-30 SURFACE): C/THRESHOLD | The amendment says the external numeric table is design rationale and not an ELAI target, yet then makes those exact floors Tier-1 inclusion behavior and requires preserving them when switching to I-25. | anchor=I-30:L114-L120, L122-L144, L168-L170
- **I-006/SD-I30-02** [SD] (DEEP-I-006.discovery.md:L584| unit=I-30 SURFACE): CORRECTION | Summary L819 says I-30 adds a dataset card; the source explicitly defers the drawer under the owner UI hold and forbids Tier-1 UI. | anchor=I30-E021, I30-E033
- **I-006/SD-I30-06** [SD] (DEEP-I-006.discovery.md:L588| unit=I-30 SURFACE): CONFLICT | `TriLayerVerdict(Promoted)` is used by I-30 but absent from I-17’s declared schema; the summary does not flag the cross-plan mismatch. | anchor=I30-C002, IX-009
- **I-006/SD-I30-08** [SD] (DEEP-I-006.discovery.md:L590| unit=I-30 SURFACE): CONFLICT | The amendment’s exact external floors are both “not an ELAI target” and binding/preserved Tier-1 behavior; this conflicts with local-threshold doctrine. | anchor=I30-C003, IX-028
- **I-006/SD-I30-09** [SD] (DEEP-I-006.discovery.md:L591| unit=I-30 SURFACE): AMBIGUITY | Verified-COMPLETE winners versus qualifying partials, gate block/warn/error semantics, JSONL/event schema, and `d_i` calibration remain unresolved. | anchor=I30-A001–I30-A005

### Surfaces — I-31 (2)

- **I-005/DII005-I31-037** [SRC] (DEEP-I-005.discovery.md:L293| unit=I-31 SURFACE): `[E][surface]` The flywheel panel shows checkpoint admits, H-28 blocks, and per-episode KL trend; it consumes `RewardServed`, `PromotionBlocked`, and `PromotionLedgered` rows and must pass `cross_lang_diff.py`. — I31:L87, I31:L117-L118
- **I-005/DII005-I31-038** [SRC] (DEEP-I-005.discovery.md:L294| unit=I-31 SURFACE): `[E][reachability]` Tier-2 production reachability is `elai train grpo --init <sft-checkpoint>`; Tier-1 contract-only invocation does not itself satisfy production R7. — I31:L104-L107

### Surfaces — I-32 (2)

- **I-003/DI003-I32-019** [SD] (DEEP-I-003.discovery.md:L485| unit=I-32 SURFACE): [E][UI] This is prompt-internal; AbDelta uses the existing governance metrics stream and no new Svelte component. Anchor: I-32_structural_role_scoping.md:55-56,75-76.
- **I-003/DI003-I32-020** [SRC] (DEEP-I-003.discovery.md:L487| unit=I-32 SURFACE): [E][reachability] dispatch_agent → apply_to_system_prompt → SSB row and the role-ab-probe CLI are production paths; tests alone do not satisfy R7. Anchor: I-32_structural_role_scoping.md:67-70.

### Surfaces — I-33 (3)

- **I-003/DI003-I33-015** [SRC] (DEEP-I-003.discovery.md:L547| unit=I-33 SURFACE): [E][tier-1/identifiers] Ship CeflRequest, I6qValidator, CommitRevealRecord, wrapped call sites, action rows, fixtures, UI tile, and decision doc. Anchor: I-33_mechanical_decision_governance.md:43-51.
- **I-003/DI003-I33-021** [SRC] (DEEP-I-003.discovery.md:L561| unit=I-33 SURFACE): [E][UI] A mandatory new governance metrics chip in MonitorWindow/SettingsWindow renders CDL/DIU/FSR through Tauri. Anchor: I-33_mechanical_decision_governance.md:49,62-63,85-86.
- **I-003/DI003-I33-026** [SRC] (DEEP-I-003.discovery.md:L572| unit=I-33 SURFACE): [E][acceptance] Reversing field order must yield the same decision, out-of-order commit exits nonzero, and the UI passes cross_lang_diff. Anchor: I-33_mechanical_decision_governance.md:88-90.

### Surfaces — I-34 (7)

- **I-004/I34-E017** [SRC] (DEEP-I-004.discovery.md:L272| unit=I-34 SURFACE): E/SURFACE | The existing decode panel gains a grammar-pin status chip through Svelte and Tauri; `cross_lang_diff.py` must pass. | anchor=I-34:L49-L50, L63-L63, L87-L92
- **I-004/I34-E037** [SRC] (DEEP-I-004.discovery.md:L292| unit=I-34 SURFACE): E/LIMIT | `xgrammar-rs` was considered and rejected as a C++ binding with far lower reported adoption that duplicates the existing XGrammar surface. | anchor=I-34:L147-L149
- **I-004/SD-I34-002** [SD] (DEEP-I-004.discovery.md:L600| unit=I-34 SURFACE): **REMOVE/FLAG** summary claim that Tier 1 records a tight-grammar arm’s measured tax; the numbered source only carries an external premise and requires CRANE pass rate >= FreeText in its fixture. | 09-I:L925-L940 | anchor=I34-E003, I34-E016, I34-E024, I34-A005
- **I-004/SD-I34-003** [SD] (DEEP-I-004.discovery.md:L601| unit=I-34 SURFACE): **ADD** complete event family, enum-source rejection, R8 freeze, exact production routes, D-16 refusal, and cross-language/UI evidence. | 09-I:L928-L943 | anchor=I34-E007–I34-E017, I34-E020–I34-E023
- **I-004/SD-I34-004** [SD] (DEEP-I-004.discovery.md:L602| unit=I-34): **ADD** cross-language verifier-pack deferral, module-path TBD, commit-point/manifest/trust ambiguities, and local fixture-statistics gap. | 09-I:L919-L943 | anchor=I34-E019, I34-E021, I34-A001–I34-A005
- **I-004/SD-I34-005** [SD] (DEEP-I-004.discovery.md:L603| unit=I-34): **ADD** J-15 Option-A amendment: plan-time llguidance `ParserFactory` compile/validate/pin with XGrammar unchanged at inference. | 09-I:L919-L943 | anchor=I34-E026–I34-E031
- **I-004/SD-I34-008** [SD] (DEEP-I-004.discovery.md:L606| unit=I-34 SURFACE): **FLAG CONFLICT** reasoning-heavy CRANE/final-only constraints versus amendment HD-3 role ban must be resolved; the summary does not surface this. | 09-I:L923-L940 | anchor=I34-E004–I34-E005, I34-E032, I34-C001

### Surfaces — I-35 (5)

- **I-004/I35-E021** [SRC] (DEEP-I-004.discovery.md:L329| unit=I-35 SURFACE): E/SURFACE | Sequence-verdict rows render in the existing gate/MonitorWindow panel through Svelte plus Tauri, with `cross_lang_diff.py` required; a reasoning decision document is also required. | anchor=I-35:L47-L50, L56-L62, L84-L89
- **I-004/I35-A005** [AC] (DEEP-I-004.discovery.md:L340| unit=I-35 SURFACE): A/SEC | “CDA-immune by construction” is an explicit design claim, but this plan defines no adversarial acceptance test proving row provenance cannot be forged through another tool surface. | anchor=I-35:L5-L9, L25-L27, L86-L89
- **I-004/SD-I35-001** [SD] (DEEP-I-004.discovery.md:L612| unit=I-35): **CORRECT** summary’s “taint check has run”: source requires an intervening `taint_check PASS`, specifically before a write-class call. | 09-I:L951-L954 | anchor=I35-E005
- **I-004/SD-I35-003** [SD] (DEEP-I-004.discovery.md:L614| unit=I-35 SURFACE): **ADD/FLAG** require-confirmation and substitute are explicit rule actions but lack Tier-1 events/fixtures/continuation semantics. | 09-I:L955-L957 | anchor=I35-E007–I35-E008, I35-A002
- **I-004/SD-I35-006** [SD] (DEEP-I-004.discovery.md:L617| unit=I-35 SURFACE): **FLAG** undefined TOML DSL/class mapping/history boundary and untested absolute CDA-immunity claim. | 09-I:L949-L972 | anchor=I35-A001, I35-A003–I35-A005

### Surfaces — I-36 (3)

- **I-001/I36-E016** [SRC] (DEEP-I-001.discovery.md:L371| unit=I-36 SURFACE): E/SURFACE | SettingsWindow shows a per-role compiled-variant Svelte badge via a Tauri read and must pass cross_lang_diff.py. | anchor=I-36:L43-L50, L58-L65, L89-L94
- **I-001/I36-E026** [SRC] (DEEP-I-001.discovery.md:L381| unit=I-36 SURFACE): E/STATE | B-37 routes across 11 backends, which is the stated portability surface motivating per-backend compilation. | anchor=I-36:L5-L9
- **I-001/SD-I36-004** [SD] (DEEP-I-001.discovery.md:L601| unit=I-36 SURFACE): ADD UI/cross-language gate, decision artifact, web-citation caution, and FLAG undefined fixture floor. | anchor=I36-E016, I36-E019, I36-E021–I36-E025, I36-A003

### Surfaces — I-37 (6)

- **I-001/I37-E011** [SRC] (DEEP-I-001.discovery.md:L406| unit=I-37 SURFACE): E/INTEROP | Spec-Kit and Kiro-style YAML/Markdown-frontmatter requirement lists round-trip into typed records at planning time; imported files express intent while ELAI proves satisfaction. | anchor=I-37:L26-L29
- **I-001/I37-E019** [SRC] (DEEP-I-001.discovery.md:L414| unit=I-37 SURFACE): E/SURFACE | G-5 cockpit renders per-requirement status chips per hunk via H-52 anchoring and Tauri/Svelte pairing; cross_lang_diff.py must pass. | anchor=I-37:L31-L36, L41-L43, L47-L54, L62-L68, L84-L99
- **I-001/I37-E028** [SRC] (DEEP-I-001.discovery.md:L423| unit=I-37 SURFACE): E/LIMIT | I-37 must not reimplement intent capture, metamorphic testing, ContractSpec gate machinery, cockpit anchoring, or mutation scoring. | anchor=I-37:L111-L116
- **I-001/I37-A004** [AC] (DEEP-I-001.discovery.md:L429| unit=I-37 SURFACE): A/STATE | The source uses VERIFIED/UNVERIFIED in prose/headline but the typed enum is Satisfied/Unverified/Regressed; whether “VERIFIED” is a UI label or alias is unstated. | anchor=I-37:L1-L11, L23-L24
- **I-001/SD-I37-002** [SD] (DEEP-I-001.discovery.md:L608| unit=I-37 SURFACE): ADD exact planning/audit/cockpit lifecycle, event kinds, real evidence joins, and full three-state fixture. | anchor=I37-E009–I37-E021
- **I-001/SD-I37-006** [SD] (DEEP-I-001.discovery.md:L612| unit=I-37 SURFACE): FLAG example-only mutation floor, canonicalization, and VERIFIED-versus-Satisfied naming. | anchor=I37-E022, I37-A001, I37-A003–I37-A004

### Surfaces — I-38 (5)

- **I-003/DI003-I38-011** [SRC] (DEEP-I-003.discovery.md:L611| unit=I-38 SURFACE): [E][corpus/privacy] Runs use date-stamped task cuts and surface D-20 memorization probe results as contamination evidence. Anchor: I-38_qdelta_statistical_floor.md:23.
- **I-003/DI003-I38-016** [SD] (DEEP-I-003.discovery.md:L621| unit=I-38 SURFACE): [E][tier-1/identifiers] Ship QdeltaConfig, run_qdelta, floor_check, bootstrap_ci, deterministic methodology generation, QdeltaMeasured, fixtures, probe field, UI tile, and decision doc. Anchor: I-38_qdelta_statistical_floor.md:41-51.
- **I-003/DI003-I38-023** [SD] (DEEP-I-003.discovery.md:L636| unit=I-38 SURFACE): [E][UI] A new QDELTA Svelte/Tauri tile sits beside SAFEDELTA and must pass cross_lang_diff. Anchor: I-38_qdelta_statistical_floor.md:63,88-89.
- **I-003/DI003-I38-024** [SD] (DEEP-I-003.discovery.md:L638| unit=I-38 SURFACE): [E][reachability] elai bench qdelta reaches run_qdelta; floor_check runs at entry; tests alone do not satisfy R7. Anchor: I-38_qdelta_statistical_floor.md:78-81.
- **I-003/DI003-I38-028** [SRC] (DEEP-I-003.discovery.md:L646| unit=I-38 SURFACE): [E][acceptance] Refusal names the floor violation; methodology regenerates deterministically and names D-20; the UI passes cross-language checking. Anchor: I-38_qdelta_statistical_floor.md:93.

### Surfaces — I-39 (5)

- **I-005/DII005-I39-003** [SRC] (DEEP-I-005.discovery.md:L309| unit=I-39 SURFACE): `[E][capability]` Add CLI `elai plan status [--letter X] [--id I-N]`. — I39:L7-L9, I39:L35-L41
- **I-005/DII005-I39-008** [SRC] (DEEP-I-005.discovery.md:L314| unit=I-39 SURFACE): `[E][lifecycle]` On first run, manual tallies become build artifacts; mechanically derived command output is the later source of truth. — I39:L19-L21
- **I-005/DII005-I39-010** [SRC] (DEEP-I-005.discovery.md:L316| unit=I-39 SURFACE): `[E][capability]` Create `crates/cli/src/plan_status_cmd.rs::derive_plan_state()`, `format_status_line()`, and `run_plan_status()`. — I39:L35-L38, I39:L47-L48, I39:L55-L59
- **I-005/DII005-I39-012** [SRC] (DEEP-I-005.discovery.md:L318| unit=I-39 SURFACE): `[E][reachability]` Production status reachability is CLI invocation → `run_plan_status()` → `derive_plan_state()` → `PlanStateDerived`. — I39:L67-L69
- **I-005/DII005-I39-021** [SRC] (DEEP-I-005.discovery.md:L327| unit=I-39 SURFACE): `[E][surface]` No dashboard is added; CLI output plus documentation are the developer-facing artifacts. — I39:L53, I39:L76-L77

### Surfaces — I-4 (6)

- **I-003/DI003-I04-023** [SRC] (DEEP-I-003.discovery.md:L175| unit=I-4 SURFACE): [E][tier-1/surface] Ship search/ladder.rs, search/mod.rs, best-of-N dispatch, decision doc, and SearchNodeEvent registration. Anchor: I-4_evidence_ranked_search_ladder.md:40-48,54-55.
- **I-003/DI003-I04-029** [SRC] (DEEP-I-003.discovery.md:L187| unit=I-4 SURFACE): [E][UI] H-15 branch-board comparison is the intended SearchNodeEvent consumer, but V1 may ship without that deferred Svelte surface. Anchor: I-4_evidence_ranked_search_ladder.md:60-62,89-92.
- **I-003/DI003-I04-046** [SRC] (DEEP-I-003.discovery.md:L224| unit=I-4 SURFACE): [E][fail-closed] Missing held-out eval command must BLOCK; there is no fallback to an LLM-reported test score. Anchor: I-4_evidence_ranked_search_ladder.md:143-146.
- **I-003/DI003-I04-054** [SRC] (DEEP-I-003.discovery.md:L240| unit=I-4 SURFACE): [E][platform/non-goal] Arbor has no pool and is Unix-only; ELAI keeps I-2's warm, Windows-safe pool instead of copying that implementation. Anchor: I-4_evidence_ranked_search_ladder.md:171-173.
- **I-003/DI003-I04-055** [SRC] (DEEP-I-003.discovery.md:L243| unit=I-4 SURFACE): [E][non-goal] Do not port persistent ReAct, a 500-turn loop, premature-stop nudges, or LLM-selected nodes; ELAI FSM plus deterministic leaf selection replaces them. Anchor: I-4_evidence_ranked_search_ladder.md:175-179.
- **I-003/DI003-I04-060** [AC] (DEEP-I-003.discovery.md:L255| unit=I-4 SURFACE): [A][UI] H-15 and SessionDag are plan-asserted seams requiring re-grep; their exact live APIs/surface cannot be treated as confirmed here. Anchors: I-4_evidence_ranked_search_ladder.md:38,60-62.

### Surfaces — I-40 (15)

- **I-004/I40-E002** [SRC] (DEEP-I-004.discovery.md:L351| unit=I-40 SURFACE): E/CAP | It exposes ELAI’s existing taint/write-gate/RRA/attestation trust layer around non-ELAI external agents as an adoption wedge. | anchor=I-40:L5-L9, L13-L35
- **I-004/I40-E003** [SRC] (DEEP-I-004.discovery.md:L352| unit=I-40 SURFACE): E/SURFACE | Tier 1 publishes a versioned open format at `docs/spec/rra-spec-v1.md` and gives external implementers a conformance target. | anchor=I-40:L16-L20, L50-L54, L64-L65, L75-L80
- **I-004/I40-E005** [SRC] (DEEP-I-004.discovery.md:L354| unit=I-40 SURFACE): E/SURFACE | `elai rra verify <path>` checks a bundle against the published spec and must reject tampering with nonzero exit. | anchor=I-40:L16-L20, L50-L54, L66-L67, L89-L103
- **I-004/I40-E006** [SRC] (DEEP-I-004.discovery.md:L355| unit=I-40 SURFACE): E/SURFACE | `elai wrap -- <agent cmd>` is a minimal MCP/stdio proxy that intercepts an external agent’s tool traffic. | anchor=I-40:L22-L23, L50-L57, L64-L67, L89-L91
- **I-004/I40-E009** [SRC] (DEEP-I-004.discovery.md:L358| unit=I-40 SURFACE): E/OBS | Each wrapped tool call emits through the existing `rra.rs` surface, yielding a wrapped-run RRA. | anchor=I-40:L22-L27, L41-L47, L68-L72, L81-L90
- **I-004/I40-E013** [SRC] (DEEP-I-004.discovery.md:L362| unit=I-40 SURFACE): E/STATE | The published strategy records wrapper/open standard as the on-ramp and deep ELAI orchestration as the premium destination. | anchor=I-40:L34-L35, L50-L53
- **I-004/I40-E018** [SRC] (DEEP-I-004.discovery.md:L367| unit=I-40 SURFACE): E/DATA | Planned implementation files are `crates/cli/src/wrap_cmd.rs`, `crates/cli/src/rra_verify_cmd.rs`, the spec document, and `workflow/reasoning/I-40-decisions.md`. | anchor=I-40:L50-L57, L64-L67, L75-L80
- **I-004/I40-E020** [SRC] (DEEP-I-004.discovery.md:L369| unit=I-40 SURFACE): E/SURFACE | Wrapped-run rows reuse the existing RRA viewer; no new Svelte component is planned, and `cross_lang_diff.py` applies only if execution-time re-grep confirms a live Svelte counterpart. | anchor=I-40:L54-L57, L73-L73, L98-L103
- **I-004/I40-E024** [SRC] (DEEP-I-004.discovery.md:L373| unit=I-40 SURFACE): E/INT | Production chains are wrap CLI -> taint -> write gate -> RRA emit -> redaction and verify CLI -> conformance checker; test-only callers do not satisfy R7. | anchor=I-40:L89-L92
- **I-004/I40-E027** [SRC] (DEEP-I-004.discovery.md:L376| unit=I-40 SURFACE): E/TEST | A wrapper-produced bundle must pass `elai rra verify`, a tampered bundle must fail, and no planted raw secret bytes may appear in export. | anchor=I-40:L100-L103
- **I-004/I40-A004** [AC] (DEEP-I-004.discovery.md:L381| unit=I-40 SURFACE): A/SURFACE | The RRA viewer’s live Svelte pairing is explicitly unknown until execution-time re-grep. | anchor=I-40:L73-L73, L98-L99
- **I-004/I40-A005** [AC] (DEEP-I-004.discovery.md:L382| unit=I-40 SURFACE): A/PRIV | G-9 owns redaction, but I-40 supplies no redaction failure policy, secret canary schema, or rule for redacting command output before versus after row sealing. | anchor=I-40:L22-L27, L37-L38, L68-L72, L100-L103
- **I-004/SD-I40-003** [SD] (DEEP-I-004.discovery.md:L625| unit=I-40): **ADD** exact open-spec contents (`chain_prev_hash`, taint schema, write-gate schema, envelope), I-18 ordering ambiguity, and normative schema gaps. | 09-I:L1083-L1096 | anchor=I40-E003–I40-E005, I40-E017, I40-A001–I40-A002
- **I-004/SD-I40-006** [SD] (DEEP-I-004.discovery.md:L628| unit=I-40 SURFACE): **QUALIFY** RRA viewer surface: source says Svelte pairing/cross-language gate is conditional on execution-time re-grep, not established. | 09-I:L1099-L1099 | anchor=I40-E020, I40-A004
- **I-004/SD-I40-007** [SD] (DEEP-I-004.discovery.md:L629| unit=I-40): **ADD ORIENTATION-ONLY STATUS** the discovery summary reports `docs/spec/rra-spec-v1.md` already exists and requires conformance inspection; that fact is not established by the numbered source and was not promoted to a source atom. | 09-I:L1702-L1705 | anchor=I40-E003, I40-A002

### Surfaces — I-41 (6)

- **I-003/DI003-I41-024** [SRC] (DEEP-I-003.discovery.md:L711| unit=I-41 SURFACE): [E][CLI] elai critique accepts a spec path, optional lens list, and simple/heavy risk. Anchor: I-41_spec_time_critique_lenses.md:48-49.
- **I-003/DI003-I41-025** [SRC] (DEEP-I-003.discovery.md:L713| unit=I-41 SURFACE): [E][artifact] CLI output is workflow/reasoning/<artifact>-critique.md plus advisory SSB rows. Anchor: I-41_spec_time_critique_lenses.md:48-49.
- **I-003/DI003-I41-032** [SRC] (DEEP-I-003.discovery.md:L727| unit=I-41 SURFACE): [E][surface] Create lenses.rs, critique_cmd.rs, the advisory-only contract, and decision doc; modify idea_framing. Anchor: I-41_spec_time_critique_lenses.md:61-67.
- **I-003/DI003-I41-033** [SRC] (DEEP-I-003.discovery.md:L729| unit=I-41 SURFACE): [E][UI/human-loop] Existing ReasoningDrawer or framing panel shows nonblocking findings; BLOCKER-CANDIDATE routes to a human and never auto-blocks. Anchor: I-41_spec_time_critique_lenses.md:68-70,97-100.
- **I-003/DI003-I41-036** [SRC] (DEEP-I-003.discovery.md:L737| unit=I-41 SURFACE): [E][reachability] Both CLI and idea-framing production paths emit findings; test-only reachability does not satisfy R7. Anchor: I-41_spec_time_critique_lenses.md:87-90.
- **I-003/DI003-I41-043** [AC] (DEEP-I-003.discovery.md:L752| unit=I-41 SURFACE): [A][UI] The exact existing drawer is unresolved between ReasoningDrawer and B-40's framing panel. Anchor: I-41_spec_time_critique_lenses.md:68-70.

### Surfaces — I-42 (7)

- **I-004/I42-E018** [SRC] (DEEP-I-004.discovery.md:L409| unit=I-42 SURFACE): E/SURFACE | CLI selection is `elai run --profile critique-only`; ordinary orchestrate dispatch also checks `RunProfile` before every executor spawn. | anchor=I-42:L41-L46, L55-L58, L78-L81
- **I-004/I42-E020** [SRC] (DEEP-I-004.discovery.md:L411| unit=I-42 SURFACE): E/DEFER | A Settings/dashboard profile switcher is deferred; CLI ships first. | anchor=I-42:L50-L52
- **I-004/I42-E021** [SRC] (DEEP-I-004.discovery.md:L412| unit=I-42 SURFACE): E/DEFER | Per-profile review-time versus generation-time telemetry waits for the G-5 cockpit. | anchor=I-42:L50-L53
- **I-004/I42-E023** [SRC] (DEEP-I-004.discovery.md:L414| unit=I-42 SURFACE): E/INT | CLI args/run dispatch thread the profile into the PLAN-asserted `task_runner.rs` executor/fixer spawn site. | anchor=I-42:L55-L59, L72-L80
- **I-004/I42-E024** [SRC] (DEEP-I-004.discovery.md:L415| unit=I-42 SURFACE): E/SURFACE | The active profile reuses B-22’s existing StatusBar mode-badge cluster; no new component is created. | anchor=I-42:L61-L62, L88-L90
- **I-004/I42-E026** [SRC] (DEEP-I-004.discovery.md:L417| unit=I-42 SURFACE): E/INT | Production reachability is both critique-only CLI dispatch and normal orchestrate dispatch before spawn; tests-only callers do not satisfy R7. | anchor=I-42:L78-L81
- **I-004/SD-I42-006** [SD] (DEEP-I-004.discovery.md:L641| unit=I-42 SURFACE): **FLAG** permissions for non-executor agents, operator-edit ingestion, and how differing write behavior yields identical verdict bytes. | 09-I:L1147-L1165 | anchor=I42-A001, I42-A003, I42-A005

### Surfaces — I-43 (6)

- **I-006/I43-E012** [SRC] (DEEP-I-006.discovery.md:L316| unit=I-43 SURFACE): E/CLI | One `elai bench claim-scope` subcommand exposes pilot `--suite … --n 50` and divergence `--mode divergence` modes. | anchor=I-43:L14-L16, L43-L46
- **I-006/I43-E022** [SRC] (DEEP-I-006.discovery.md:L326| unit=I-43 SURFACE): E/UI | An H-9-family Phase-0 cockpit tile is optional; W-6 permits no UI because bench TOML plus memo are the load-bearing surfaces. | anchor=I-43:L69-L71, L98-L101
- **I-006/I43-E025** [SRC] (DEEP-I-006.discovery.md:L329| unit=I-43 SURFACE): E/REACH | Both CLI modes must traverse `bench_claim_scope::run` to SSB; test-only callers never satisfy R7. | anchor=I-43:L87-L90
- **I-006/I43-A001** [AC] (DEEP-I-006.discovery.md:L336| unit=I-43 SURFACE): A/CFG | Prose says default pilot N is “50–100,” while the concrete Tier-1 command fixes `--n 50`; default/range validation is unclear. | anchor=I-43:L5-L8, L14-L17, L43-L46
- **I-006/SD-I43-02** [SD] (DEEP-I-006.discovery.md:L599| unit=I-43 SURFACE): ADDITION | Exact two-mode CLI, event arms, mock-runner/canned-RRA real-path fixture, and no-test-only R7 rule were omitted. | anchor=I43-E012, I43-E015, I43-E017–I43-E018, I43-E025
- **I-006/SD-I43-06** [SD] (DEEP-I-006.discovery.md:L603| unit=I-43 SURFACE): AMBIGUITY | Default N, statistical/trap criteria, unnamed bench TOML, and divergence/claim schemas remain open beyond the summary’s Track-19 kill-gate qualification. | anchor=I43-A001–I43-A005

### Surfaces — I-44 (4)

- **I-005/DII005-I44-016** [SRC] (DEEP-I-005.discovery.md:L350| unit=I-44 SURFACE): `[E][verification invariant]` `elai replay --verify` must still pass for hash-checkable steps after purge; verification cannot depend on purged bytes. — I44:L24-L26, I44:L51-L52, I44:L103-L107
- **I-005/DII005-I44-020** [SRC] (DEEP-I-005.discovery.md:L354| unit=I-44 SURFACE): `[E][CLI]` Add `elai run --zero-retention` and check policy at the seal point. — I44:L48, I44:L84-L86
- **I-005/DII005-I44-022** [SRC] (DEEP-I-005.discovery.md:L356| unit=I-44 SURFACE): `[E][reachability]` Production reachability is the zero-retention CLI/run seal path and I-40 export path; test-only callers do not satisfy R7. — I44:L84-L87
- **I-005/DII005-I44-025** [SRC] (DEEP-I-005.discovery.md:L359| unit=I-44 SURFACE): `[E][surface]` Render receipts in the existing RRA viewer/session-retention drawer, exposing purge/retained classes and retained-root blake3. — I44:L67-L68, I44:L96-L98

### Surfaces — I-45 (9)

- **I-004/I45-E001** [SRC] (DEEP-I-004.discovery.md:L438| unit=I-45 SURFACE): E/STATUS | I-45 is a Track-20 can’t-lie EXTENSION, effort S-M, Wave 1, strong-grounded on ELAI and moderate-grounded on the roach/SL-11 semantics, bound to ID-4. | anchor=I-45:L1-L4
- **I-004/I45-E007** [SRC] (DEEP-I-004.discovery.md:L444| unit=I-45 SURFACE): E/FLOW | Every receipt read path rehashes the current objective, explicitly including `elai verify-attestation`, replay, and plan-status consumers. | anchor=I-45:L19-L21, L38-L49
- **I-004/I45-E009** [SRC] (DEEP-I-004.discovery.md:L446| unit=I-45 SURFACE): E/LIMIT | I-45 adds no verifier, gate, objective-versioning UI, or historical-receipt migration. | anchor=I-45:L22-L23
- **I-004/I45-E016** [SRC] (DEEP-I-004.discovery.md:L453| unit=I-45 SURFACE): E/SURFACE | The existing RRA viewer gets a stale-receipt badge; no new component is created. | anchor=I-45:L38-L41
- **I-004/I45-E018** [SRC] (DEEP-I-004.discovery.md:L455| unit=I-45 SURFACE): E/INT | Production reachability is `elai run --emit-rra` at the RRA seal site and `elai verify-attestation` at the read site; contracts-only callers do not satisfy R7. | anchor=I-45:L48-L49
- **I-004/SD-I45-001** [SD] (DEEP-I-004.discovery.md:L647| unit=I-45): **ADD** exact canonical sources/order/whitespace/blake3 requirements, orchestrator-only ownership, and shared-serializer inference across every read path. | 09-I:L1238-L1253 | anchor=I45-E004–I45-E007, I45-E019–I45-E020, I45-I001
- **I-004/SD-I45-003** [SD] (DEEP-I-004.discovery.md:L649| unit=I-45 SURFACE): **ADD/QUALIFY** complete `ObjectiveSealed` kind list and flag `ReceiptStale` event-versus-error ambiguity. | 09-I:L1250-L1253 | anchor=I45-E017, I45-E021–I45-E023, I45-A004
- **I-004/SD-I45-005** [SD] (DEEP-I-004.discovery.md:L651| unit=I-45): **ADD** per-requirement/cross-session deferrals and unresolved LEGACY/missing-field semantics. | 09-I:L1238-L1253 | anchor=I45-E010, I45-E015, I45-A003, I45-A005
- **I-004/SD-I45-006** [SD] (DEEP-I-004.discovery.md:L652| unit=I-45 SURFACE): **FLAG** every-PASS versus every-sealed-verdict scope and undefined canonicalization/source precedence. | 09-I:L1238-L1244 | anchor=I45-A001–I45-A002

### Surfaces — I-46 (8)

- **I-002/I002-I46-002** [SRC] (DEEP-I-002.discovery.md:L613| unit=I-46 SURFACE): [E][compliance] Before execution, read gajae `bash-interceptor.ts`, `bash-command-fixup.ts`, and `bash-allowed-prefixes.ts`. Anchors: lines 12-14, 59-61.
- **I-002/I002-I46-003** [SRC] (DEEP-I-002.discovery.md:L614| unit=I-46 SURFACE): [E][action] The interceptor parses/classifies each bash command before existing bash validation. Anchors: lines 15-16, 41-45.
- **I-002/I002-I46-006** [SRC] (DEEP-I-002.discovery.md:L617| unit=I-46 SURFACE): [E][fixup] A small frozen deterministic table repairs known-broken command shapes such as flag order and missing quoting. Anchors: lines 17-18.
- **I-002/I002-I46-019** [SRC] (DEEP-I-002.discovery.md:L630| unit=I-46 SURFACE): [E][UI] Redirect rows appear in the existing tool-call drawer; no component is added. Anchor: line 39.
- **I-002/I002-I46-023** [SRC] (DEEP-I-002.discovery.md:L634| unit=I-46 SURFACE): [E][security] No bypass flag may skip interception. Anchors: lines 48-49.
- **I-002/I002-I46-030** [SRC] (DEEP-I-002.discovery.md:L641| unit=I-46 SURFACE): [E][acceptance] Redirected commands create no process in the journal window. Anchors: lines 55-57.
- **I-002/I002-I46-031** [SRC] (DEEP-I-002.discovery.md:L642| unit=I-46 SURFACE): [E][negative-test] An allowed-prefix command executes unchanged with zero redirect rows. Anchors: lines 55-57.
- **I-002/I002-I46-032** [SRC] (DEEP-I-002.discovery.md:L643| unit=I-46 SURFACE): [E][broken-arm] A redirect row plus a spawned grep in the same window fails. Anchor: line 57.

### Surfaces — I-47 (7)

- **I-006/I47-E012** [SRC] (DEEP-I-006.discovery.md:L361| unit=I-47 SURFACE): E/UI | Prompt assembly is internal, so W-6 explicitly justifies no frontend pair. | anchor=I-47:L41-L44
- **I-006/I47-E016** [SRC] (DEEP-I-006.discovery.md:L365| unit=I-47 SURFACE): E/LIMIT | Per-task rule text, model-authored tag content, and a skip flag are forbidden. | anchor=I-47:L53-L55
- **I-006/I47-E019** [SRC] (DEEP-I-006.discovery.md:L368| unit=I-47 SURFACE): E/FAIL | A no-tags build flag must be structurally absent; enabled hardening with zero evidence rows is an R1 failure. | anchor=I-47:L63-L65
- **I-006/I47-A002** [AC] (DEEP-I-006.discovery.md:L373| unit=I-47 SURFACE): A/DEFAULT | Reachability says behavior is unconditional/no-skip, yet failure prose refers to “while the feature is ON”; no feature flag/default/migration path is defined. | anchor=I-47:L51-L55, L63-L65
- **I-006/SD-I47-03** [SD] (DEEP-I-006.discovery.md:L611| unit=I-47 SURFACE): ADDITION | Wrapping/rules are unconditional by role; no skip flag, model-authored tags, classification copy, or artifact truncation is allowed. | anchor=I47-E015–I47-E017
- **I-006/SD-I47-05** [SD] (DEEP-I-006.discovery.md:L613| unit=I-47 SURFACE): ADDITION | No-UI justification, exact malicious fixture bytes, structurally absent no-tags flag, and missing-row failure were omitted. | anchor=I47-E012, I47-E018–I47-E019
- **I-006/SD-I47-06** [SD] (DEEP-I-006.discovery.md:L614| unit=I-47 SURFACE): AMBIGUITY | Tag/frame canonical bytes, frozen-table schema, actual prompt modules, and “feature ON” versus unconditional/no-skip activation are unresolved. | anchor=I47-A001–I47-A003

### Surfaces — I-48 (6)

- **I-001/I48-E011** [SRC] (DEEP-I-001.discovery.md:L451| unit=I-48 SURFACE): E/DEFER | Python-sidecar prompt strings are a separate cross-language slice and dashboard-side copy is deferred. | anchor=I-48:L31-L34
- **I-001/I48-E015** [SRC] (DEEP-I-001.discovery.md:L455| unit=I-48 SURFACE): E/SURFACE | No UI changes ship because this is a build-time discipline refactor. | anchor=I-48:L36-L38
- **I-001/I48-A002** [AC] (DEEP-I-001.discovery.md:L463| unit=I-48 SURFACE): A/SCOPE | “Every dispatch prompt template” is the headline target, but current Tier 1 explicitly excludes Python and dashboard text; completion of the global headline spans later slices. | anchor=I-48:L1-L11, L31-L34
- **I-001/I48-C001** [CON] (DEEP-I-001.discovery.md:L465| unit=I-48 SURFACE): C/SCOPE | The orientation presents every dispatch prompt as moved now; the numbered source limits ship-now extraction to orchestrator/providers and defers sidecar/dashboard material. | anchor=I-48:L31-L34; orientation:L1309-L1311
- **I-001/SD-I48-003** [SD] (DEEP-I-001.discovery.md:L620| unit=I-48 SURFACE): CORRECT global “every prompt now” scope to orchestrator/providers Tier 1 with Python/dashboard slices deferred. | anchor=I48-E010–I48-E011, I48-A002, I48-C001
- **I-001/SD-I48-005** [SD] (DEEP-I-001.discovery.md:L622| unit=I-48 SURFACE): FLAG approximate threshold and incomplete golden role/provider inventory. | anchor=I48-A001, I48-A003

### Surfaces — I-49 (7)

- **I-006/I49-E002** [SRC] (DEEP-I-006.discovery.md:L385| unit=I-49 SURFACE): E/NON-GOAL | The wholesale gajae provider layer is rejected; existing B-37, CLI bridges, OAuth, and local serving remain the architectural base. | anchor=I-49:L8-L13
- **I-006/I49-E011** [SRC] (DEEP-I-006.discovery.md:L394| unit=I-49 SURFACE): E/NON-GOAL | GitLab Duo, Kimi, Vertex, replacement/removal of adapters or CLI bridges, and API-key storage changes are outside scope. | anchor=I-49:L28-L29
- **I-006/I49-E012** [SRC] (DEEP-I-006.discovery.md:L395| unit=I-49 SURFACE): E/STATUS | Existing runtime adapters include Anthropic API, multiple CLI bridges, `genai_adapter`, Hermes, and local serving; `oauth.rs` stays. | anchor=I-49:L31-L34
- **I-006/I49-E016** [SRC] (DEEP-I-006.discovery.md:L399| unit=I-49 SURFACE): E/UI | `elai runtime list` and the existing `RuntimePanel` list both expose `bedrock` and `azure-openai`; no new component. | anchor=I-49:L44-L47, L60-L67
- **I-006/I49-E019** [SRC] (DEEP-I-006.discovery.md:L402| unit=I-49 SURFACE): E/REACH | Selecting either backend traverses the real PerRoleRuntime request builder; `elai runtime list` proves registry reach and contracts-only callers do not satisfy R7. | anchor=I-49:L54-L55
- **I-006/I49-E029** [SRC] (DEEP-I-006.discovery.md:L412| unit=I-49 SURFACE): E/INTEG | Composition retains B-37 as the frame, OAuth/CLI bridges as keep-list paths, I-50 as future credential supplier, and I-51 as catalog owner. | anchor=I-49:L69-L77
- **I-006/SD-I49-06** [SD] (DEEP-I-006.discovery.md:L625| unit=I-49 SURFACE): AMBIGUITY | Shim placement, AWS/Azure API/dependency choice, credential supply, backend/error schemas, J-14 compatibility, and production-vs-fixture reach remain open. | anchor=I49-A001–I49-A006

### Surfaces — I-5 (3)

- **I-002/I002-I05-021** [SRC] (DEEP-I-002.discovery.md:L482| unit=I-5 SURFACE): [E][deferred][UI] The ReasoningDrawer vividness-tax tile is Tier 2, wire-dark until at least three A/B runs show a consistent delta and the GD-4 bound is measured. Anchors: lines 47-49, 57-58, 81-83.
- **I-002/I002-I05-026** [SRC] (DEEP-I-002.discovery.md:L487| unit=I-5 SURFACE): [E][UI][done-gate] If the dashboard tile ships in the H-15 era, `cross_lang_diff.py PASS` is mandatory; absence is W-6-justified for v1. Anchors: lines 81-83.
- **I-002/I002-I05-037** [AC] (DEEP-I-002.discovery.md:L501| unit=I-5 SURFACE): [A][UI] The exact live UI timing depends on H-15-era surfaces and measured stability; v1 explicitly permits no tile. Anchors: lines 47-49, 81-83.

### Surfaces — I-50 (4)

- **I-003/DI003-I50-011** [SRC] (DEEP-I-003.discovery.md:L785| unit=I-50 SURFACE): [E][non-goal] CLI-bridge authentication remains vendor-CLI-based and unchanged. Anchor: I-50_subscription_auth_broker.md:23.
- **I-003/DI003-I50-016** [SRC] (DEEP-I-003.discovery.md:L796| unit=I-50 SURFACE): [E][UI] Existing RuntimePanel/SettingsWindow rows show per-provider authentication status; no new component is required. Anchor: I-50_subscription_auth_broker.md:35-36.
- **I-003/DI003-I50-017** [SRC] (DEEP-I-003.discovery.md:L798| unit=I-50 SURFACE): [E][UI/security] OpenCode may inspire Providers/Connect/verified-status layout only; implementation is clean-room and credential replay is forbidden. Anchor: I-50_subscription_auth_broker.md:37-43.
- **I-003/DI003-I50-018** [SRC] (DEEP-I-003.discovery.md:L801| unit=I-50 SURFACE): [E][coordination] J-30 owns the wider BYOK/custom-endpoint panel sharing the surface. Anchor: I-50_subscription_auth_broker.md:42-43.

### Surfaces — I-51 (1)

- **I-005/DII005-I51-022** [SRC] (DEEP-I-005.discovery.md:L391| unit=I-51 SURFACE): `[E][surface]` No UI is added; existing cost and tier surfaces remain unchanged. — I51:L38-L41

### Surfaces — I-52 (6)

- **I-003/DI003-I52-002** [SRC] (DEEP-I-003.discovery.md:L837| unit=I-52 SURFACE): [E][activation] Existing edit-checkpoint machinery and CLI/contracts ship, but agents cannot call it; I-52 exposes it rather than building a store. Anchor: I-52_checkpoint_rewind_tool.md:6-12.
- **I-003/DI003-I52-010** [SRC] (DEEP-I-003.discovery.md:L854| unit=I-52 SURFACE): [E][coordination] If H-49 ships first, I-52 becomes a tool surface over H-49's store and reuses EditHistoryEvent. Anchor: I-52_checkpoint_rewind_tool.md:20-23.
- **I-003/DI003-I52-012** [SRC] (DEEP-I-003.discovery.md:L858| unit=I-52 SURFACE): [E][non-goal] Redo/branching semantics, git-worktree integration, cross-session travel, and new UI are excluded. Anchor: I-52_checkpoint_rewind_tool.md:24-25.
- **I-003/DI003-I52-014** [SRC] (DEEP-I-003.discovery.md:L862| unit=I-52 SURFACE): [E][flag/default] If the substrate is still OFF, activation flips it per W-2: default-ON behind an ELAI_*_DISABLE flag or an allowlist row. Anchor: I-52_checkpoint_rewind_tool.md:30-32.
- **I-003/DI003-I52-019** [SRC] (DEEP-I-003.discovery.md:L873| unit=I-52 SURFACE): [E][UI] Checkpoint rows use the existing session/transcript drawer; no component is added. Anchor: I-52_checkpoint_rewind_tool.md:40-41.
- **I-003/DI003-I52-031** [AC] (DEEP-I-003.discovery.md:L898| unit=I-52 SURFACE): [A][flag] The exact disable-flag name or allowlist-row schema is not specified. Anchor: I-52_checkpoint_rewind_tool.md:30-32.

### Surfaces — I-53 (2)

- **I-006/I53-E015** [SRC] (DEEP-I-006.discovery.md:L442| unit=I-53 SURFACE): E/UI | Existing tool-call drawer rows render the three tools; no new component. | anchor=I-53:L41-L42
- **I-006/SD-I53-06** [SD] (DEEP-I-006.discovery.md:L637| unit=I-53 SURFACE): AMBIGUITY | Recipe syntax/arguments, JSON cap/path contract, archive bomb/path protections, and actual dependency inventory remain unspecified and were not surfaced by the summary. | anchor=I53-A001–I53-A004

### Surfaces — I-54 (2)

- **I-002/I002-I54-017** [IX] (DEEP-I-002.discovery.md:L673| unit=I-54 SURFACE): [E][interaction] Harness-side `impact_radius.py`/`elai_graph` are blast-radius siblings; preview is the runtime-side equivalent surface. Anchors: lines 30-31.
- **I-002/I002-I54-022** [SRC] (DEEP-I-002.discovery.md:L678| unit=I-54 SURFACE): [E][UI] Existing tool-call drawer rows show preview and apply, including the operator-visible diff; no new component is added. Anchors: lines 41-42.

### Surfaces — I-6 (4)

- **I-005/DII005-I6-009** [SRC] (DEEP-I-005.discovery.md:L46| unit=I-6 SURFACE): `[E][enforcement]` The I-1 supervisor validates tag presence in the first reply-token window. — I6:L16, I6:L33-L34, I6:L41-L42, I6:L65
- **I-005/DII005-I6-015** [SRC] (DEEP-I-005.discovery.md:L52| unit=I-6 SURFACE): `[E][surface]` I-6 creates no dashboard surface; the anchor is prompt-internal and is measured through fixtures. — I6:L54-L56, I6:L77-L78
- **I-005/DII005-I6-019** [SRC] (DEEP-I-005.discovery.md:L56| unit=I-6 SURFACE): `[E][deferred]` The constraint-adherence cockpit tile is Tier 2 under the same GD-4 bound gate. — I6:L48
- **I-005/DII005-I6-023** [SRC] (DEEP-I-005.discovery.md:L60| unit=I-6 SURFACE): `[E][acceptance/evidence]` The response journal must show the acknowledgement tag in its first token window. — I6:L84-L85

### Surfaces — I-7 (1)

- **I-005/DII005-I7-021** [SRC] (DEEP-I-005.discovery.md:L88| unit=I-7 SURFACE): `[E][deferred/UI]` The cost-cockpit cascade-trace tile is deferred until the GD-4 bound is measured; `cross_lang_diff.py` becomes required when it ships. — I7:L54, I7:L61-L62, I7:L87-L90

### Surfaces — I-8 (4)

- **I-001/I8-E015** [SRC] (DEEP-I-001.discovery.md:L137| unit=I-8 SURFACE): E/DEFER | The ReasoningDrawer abort/dead-end strip is deferred/wire-dark behind a GD-4 bound and is W-6-justified for v1; when shipped it must pass cross_lang_diff.py. | anchor=I-8:L53-L56, L64-L65, L94-L96
- **I-001/I8-C001** [CON] (DEEP-I-001.discovery.md:L153| unit=I-8 SURFACE): C/SURFACE | The orientation summary presents the ReasoningDrawer strip as an unqualified feature; the numbered source explicitly defers it behind a GD-4 bound and justifies no v1 frontend. | anchor=I-8:L53-L56, L64-L65, L94-L96; orientation:L275-L275
- **I-001/SD-I8-004** [SD] (DEEP-I-001.discovery.md:L541| unit=I-8 SURFACE): CORRECT the summary's unconditional ReasoningDrawer surface: it is deferred/wire-dark behind GD-4 and W-6-justified for v1. | anchor=I8-E015, I8-C001
- **I-001/SD-I8-005** [SD] (DEEP-I-001.discovery.md:L542| unit=I-8 SURFACE): FLAG undefined approach canonicalization, tool alias/handler naming, and configuration-key identity. | anchor=I8-A001–I8-A003

### Surfaces — I-9 (6)

- **I-003/DI003-I09-012** [SRC] (DEEP-I-003.discovery.md:L288| unit=I-9 SURFACE): [E][CLI] The command is elai bench error-amplification. Anchor: I-9_error_amplification_telemetry.md:21,47.
- **I-003/DI003-I09-018** [SRC] (DEEP-I-003.discovery.md:L300| unit=I-9 SURFACE): [E][tier-1/surface] Ship error_amplification_runner.rs, CLI wiring, the bounded temperature hook, action registration, and I-9-decisions.md. Anchor: I-9_error_amplification_telemetry.md:42-53.
- **I-003/DI003-I09-021** [SRC] (DEEP-I-003.discovery.md:L307| unit=I-9 SURFACE): [E][deferred/UI] The cockpit alpha-trend tile waits for at least three consistent baseline runs. Anchor: I-9_error_amplification_telemetry.md:57-58.
- **I-003/DI003-I09-023** [SRC] (DEEP-I-003.discovery.md:L311| unit=I-9 SURFACE): [E][identifiers] Produced identifiers are ErrorPropagationHarness::run, ErrorAmplificationMeasured, and the benchmark CLI. Anchor: I-9_error_amplification_telemetry.md:71-75.
- **I-003/DI003-I09-024** [SRC] (DEEP-I-003.discovery.md:L314| unit=I-9 SURFACE): [E][reachability] CLI dispatch reaches the harness; a task_runner mid-run alpha event reaches temperature tightening; tests alone do not satisfy R7. Anchor: I-9_error_amplification_telemetry.md:81-86.
- **I-003/DI003-I09-031** [CON] (DEEP-I-003.discovery.md:L332| unit=I-9 SURFACE): [C][summary status] Grammar tightening and the alpha tile are architectural features but explicitly Tier-2, not Tier-1 deliverables. Anchors: I-9_error_amplification_telemetry.md:54-60.

### Surfaces — I-GROUNDING (3)

- **I-001/IG-E023** [SRC] (DEEP-I-001.discovery.md:L59| unit=I-GROUNDING SURFACE): E/STATE | The workspace already contains safety, memory, eval-corpus, ssb-otel-export, web_search, web_validator, agent, ipc-types, scaffold, dashboard-v2, and parked/backup dashboard crates. | anchor=IG:L59-L60
- **I-001/IG-E036** [SRC] (DEEP-I-001.discovery.md:L72| unit=I-GROUNDING SURFACE): E/STATUS | No elai plan status repo-derived command exists; I-39 is a build. | anchor=IG:L85-L85
- **I-001/IG-E072** [SRC] (DEEP-I-001.discovery.md:L108| unit=I-GROUNDING SURFACE): E/INT | Track 18 anchors I-32 to agent configs, I-33 to rules/refusal, I-34 to decoding/grammar substrates, I-35 to temporal/per-call gates, I-36 to compiled-store/routing/optimizer, I-37 to intent/contract/cockpit/anchor, I-38 to quality/corpus probes, I-39 to repo/SSB/wiring/archive, and I-40 to RRA/provenance/adapters/bridge. | anchor=IG:L201-L205

### Surfaces — I-STEAL (5)

- **I-002/I002-GM-002** [SRC] (DEEP-I-002.discovery.md:L40| unit=I-STEAL SURFACE): [E][evidence] The map says `elai.map` staleness was unknown, so every ELAI anchor was confirmed directly by glob/grep in live source; detail files cite `SL-n` IDs instead of restating facts. Anchor: lines 4-6.
- **I-002/I002-GM-005** [SRC] (DEEP-I-002.discovery.md:L43| unit=I-STEAL SURFACE): [E][dependency] I-46's mandatory gajae references are `bash-interceptor.ts`, `bash-command-fixup.ts`, and `bash-allowed-prefixes.ts`. Anchor: lines 19-21.
- **I-002/I002-GM-012** [SRC] (DEEP-I-002.discovery.md:L50| unit=I-STEAL SURFACE): [E][status] E-25 edit checkpoints, its CLI, and both named checkpoint contracts are shipped and grep-confirmed. Anchor: lines 37-40.
- **I-002/I002-GM-014** [SRC] (DEEP-I-002.discovery.md:L52| unit=I-STEAL SURFACE): [E][integration][fallback] If H-49 ships first, I-52 becomes a tool surface over H-49's store and should reuse its `EditHistoryEvent` row. Anchor: lines 40-42.
- **I-002/I002-GM-019** [SRC] (DEEP-I-002.discovery.md:L57| unit=I-STEAL SURFACE): [E][scope] I-49 adds direct clients only for SigV4+EventStream, Azure Responses, and the OpenAI-as-Anthropic shim seam—protocol shapes a CLI bridge cannot supply. Anchor: lines 51-53.

### Surfaces — IX (5)

- **I-002/I002-X-003** [IX] (DEEP-I-002.discovery.md:L707| unit=IX SURFACE): [E][measurement] E-19 is the I-5 same-choice A/B surface; H-9's hit monitor is I-10 planner input; both convert a design claim into journal-derived evidence before rollout. Sources: I-5 lines 32-44; I-10 lines 92-97.
- **I-002/I002-X-007** [IX] (DEEP-I-002.discovery.md:L711| unit=IX SURFACE): [E][tool-discovery] I-46 obtains substitute targets from the registered tool surface/H-4, and I-53 later supplies recipe targets; I-46 does not invent or execute the target. Sources: I-46 lines 41-45, 64-66; I-0 lines 428-430.
- **I-002/I002-X-013** [IX] (DEEP-I-002.discovery.md:L720| unit=IX SURFACE): [I][failure-propagation] A broken I-5 prefix-preservation implementation can surface as an I-10 prefix-diff/guard trip; the shared `update_breakpoints` seam makes this a plausible cross-feature diagnostic, though no direct event join is specified. Sources: I-5 lines 24-27, 64-66; I-10 lines 60-66, 80-83.
- **I-002/I002-X-018** [IX] (DEEP-I-002.discovery.md:L728| unit=IX SURFACE): [A][UI] I-5's tile is deferred/wire-dark, I-10's exact cost-panel component name is plan-asserted, and I-46/I-54 reuse an existing drawer; only the interaction contract is stable, not every current frontend symbol.
- **I-002/I002-X-020** [IX] (DEEP-I-002.discovery.md:L730| unit=IX SURFACE): [C][summary-scope] High-level phrases such as “adds a diffusion LLM” and “adds a vividness tile” overstate Tier-1 delivery unless read with the detail files: both live dLLM access and the I-5 tile are gated/deferred.

### Surfaces — RELATIONAL (53)

- **I-001/IX-003** [IX] (DEEP-I-001.discovery.md:L477| unit=RELATIONAL SURFACE): I-8 abort outcomes feed the single calibration surface that I-23 unifies; neither plan duplicates refusal or scoring math. | anchor=I8-E004, I8-E012–I8-E013, I23-E002, I23-E017
- **I-001/IX-016** [IX] (DEEP-I-001.discovery.md:L490| unit=RELATIONAL SURFACE): I-37's evidence lifecycle is planning import/authorship -> bound tests/MRs -> I-20 sampled coverage -> audit status -> per-hunk cockpit chip; deleting evidence creates Regressed rather than erasing the requirement. | anchor=I37-E004–I37-E020
- **I-001/AC-004** [AC] (DEEP-I-001.discovery.md:L505| unit=RELATIONAL SURFACE): rollout | I-8 lacks the named halting-bound key, and its UI is deferred despite the summary's unconditional presentation. | anchor=I8-A003, I8-C001
- **I-001/AC-011** [SD] (DEEP-I-001.discovery.md:L512| unit=RELATIONAL SURFACE): refreshed I-28 correction | The summary's node/chunk/template walk, L2-scored veto negative, PhaseComplete hook, three-abort demotion, and optional heat UI conflict with the refreshed sliced plan. | anchor=I28-C001–I28-C005
- **I-001/AC-013** [AC] (DEEP-I-001.discovery.md:L514| unit=RELATIONAL): requirements | I-37 leaves the mutation floor, canonical text, Kiro-specific parser, and VERIFIED/Satisfied label mapping unresolved; deletion status conflicts with the summary. | anchor=I37-A001–I37-A004, I37-C001–I37-C002
- **I-003/SD-I01-02** [SD] (DEEP-I-003.discovery.md:L979| unit=RELATIONAL SURFACE): [ADDITION] Exact Directive fields, event arms, production reachability, Tier-1 UI posture, and full acceptance/bans were absent. Refs: DI003-I01-003, DI003-I01-014, DI003-I01-021–026.
- **I-003/SD-I04-03** [SD] (DEEP-I-003.discovery.md:L986| unit=RELATIONAL SURFACE): [ADDITION] The full J-11 held-out merge gate—hidden B_test, detached pool, fail-closed command, frozen manifest, deterministic parser, merge recovery, path/output guards—was omitted. Refs: DI003-I04-040–048.
- **I-003/SD-I09-01** [SD] (DEEP-I-003.discovery.md:L991| unit=RELATIONAL SURFACE): [CORRECTION] Summary I-9.07/I-9.11 present grammar tightening and the tile without their Tier-2 gates; V1 is temperature-only and the tile needs three baselines. Refs: DI003-I09-021–022, DI003-I09-031.
- **I-003/SD-I33-01** [SD] (DEEP-I-003.discovery.md:L1006| unit=RELATIONAL SURFACE): [ADDITION] Orchestrator-owned I6Q config, exact commitment formula/order proof, selection-by-negation guarantee, D-16 routing, mandatory UI, and deferred calibration/correlation were omitted. Refs: DI003-I33-004, DI003-I33-007–009, DI003-I33-018–021.
- **I-003/SD-I41-02** [SD] (DEEP-I-003.discovery.md:L1015| unit=RELATIONAL SURFACE): [ADDITION] Deferred planning-skill injection, eval corpora, GEPA optimization, exact production reachability, and UI/dedup ambiguities were omitted. Refs: DI003-I41-029–036, DI003-I41-043–044.
- **I-003/SD-I50-01** [SD] (DEEP-I-003.discovery.md:L1018| unit=RELATIONAL SURFACE): [ADDITION] No CLI-bridge change/keychain dependency, existing-location rule, AWS/Azure deferral, ProviderLinked arm, clean-room UI constraint, all-adapter reachability, and no-hang failure evidence were omitted. Refs: DI003-I50-008–011, DI003-I50-014, DI003-I50-017, DI003-I50-019–024.
- **I-004/IX-005** [IX] (DEEP-I-004.discovery.md:L480| unit=RELATIONAL SURFACE): E | I-2 backend state is surfaced by extending C-23 BranchTree, while the underlying sandbox/pool implementation composes C-11/E-15. | anchor=I2-E015, I2-E021
- **I-004/IX-007** [IX] (DEEP-I-004.discovery.md:L482| unit=RELATIONAL SURFACE): E | I-3 intentionally produces a TOML rather than UI; I-27 is named as the later cockpit consumer while separately sourcing compute telemetry from I-4. | anchor=I3-E006–I3-E007, I3-E020, I3-I001; I27-E011, I27-E014
- **I-004/IX-008** [IX] (DEEP-I-004.discovery.md:L483| unit=RELATIONAL SURFACE): E | I-12 is a gated extension chain: receipt count/lift -> C-28 mutation validation -> existing Phase-3F lifecycle -> RulesDrawer/export, with F-5 demotion and IS-16 freeze controls. | anchor=I12-E002–I12-E016, I12-I001
- **I-004/AC-002** [AC] (DEEP-I-004.discovery.md:L512| unit=RELATIONAL): ambiguity | I-2 does not fully define “last-seq wins” for non-overlapping patches or canonical patch application. | anchor=I2-A001
- **I-004/AC-003** [AC] (DEEP-I-004.discovery.md:L513| unit=RELATIONAL SURFACE): ambiguity/security | I-2’s absolute-path escape rule omits symlink/junction/canonicalization cases and its conflict badge is not mapped to UI. | anchor=I2-A002, I2-A004
- **I-004/AC-006** [AC] (DEEP-I-004.discovery.md:L516| unit=RELATIONAL): ambiguity/config | I-3 leaves pragma values, remaining checkpoint sites, benchmark workload, and acceptable performance largely unspecified. | anchor=I3-A001–I3-A003
- **I-004/AC-007** [AC] (DEEP-I-004.discovery.md:L517| unit=RELATIONAL): ambiguity/recovery | I-3 cites POSIX rename without defining non-POSIX replacement, directory fsync, cleanup, or rename retry. | anchor=I3-A004
- **I-004/AC-008** [AC] (DEEP-I-004.discovery.md:L518| unit=RELATIONAL): ambiguity/measurement | I-12 does not fix `k`, schedule interval, or statistically valid lift method. | anchor=I12-A001–I12-A003
- **I-004/AC-009** [AC] (DEEP-I-004.discovery.md:L519| unit=RELATIONAL): ambiguity/data | I-12 names three export formats without specifying their block schemas or collision/update semantics. | anchor=I12-A004
- **I-004/AC-010** [AC] (DEEP-I-004.discovery.md:L520| unit=RELATIONAL): ambiguity/schema | I-21 uses both `bounded_iteration_count` and `iteration_count`, and its deterministic-criterion reference grammar is undefined. | anchor=I21-A001–I21-A002
- **I-004/AC-011** [AC] (DEEP-I-004.discovery.md:L521| unit=RELATIONAL): ambiguity/dependency | I-21 permits an unspecified temporary I-1 isolation shim and leaves replace-versus-supplement outer retry unresolved. | anchor=I21-A003–I21-A004
- **I-004/AC-012** [AC] (DEEP-I-004.discovery.md:L522| unit=RELATIONAL): ambiguity/state | I-21 does not fully define the final third-failure event/stop ordering. | anchor=I21-A005
- **I-004/AC-013** [AC] (DEEP-I-004.discovery.md:L523| unit=RELATIONAL): ambiguity/measurement | I-27 leaves compute stepping, bootstrap confidence, knee logic, cost normalization, and report-date determinism unresolved. | anchor=I27-A001–I27-A004
- **I-004/AC-015** [AC] (DEEP-I-004.discovery.md:L525| unit=RELATIONAL): ambiguity/implementation | I-34 leaves module ownership, commit-point transform, manifest schema, untrusted-source test, and fixture statistics unresolved. | anchor=I34-A001–I34-A005
- **I-004/AC-017** [AC] (DEEP-I-004.discovery.md:L527| unit=RELATIONAL): ambiguity/workflow | I-35 defines confirmation/substitution but supplies Tier-1 evidence only for allow/halt; its TOML DSL, class registry, and session-history semantics remain open. | anchor=I35-A001–I35-A004
- **I-004/AC-020** [AC] (DEEP-I-004.discovery.md:L530| unit=RELATIONAL): ambiguity/security/privacy | I-40 does not define proxy bypass coverage or redaction-stage failure semantics; its viewer pairing is also unverified. | anchor=I40-A003–I40-A005
- **I-004/AC-021** [AC] (DEEP-I-004.discovery.md:L531| unit=RELATIONAL): ambiguity/permissions | I-42 narrows its executable invariant to executor/fixer writes although prose says all spawned agents only critique; actor attribution and exact gate site are assumed/unresolved. | anchor=I42-I001, I42-A001–I42-A002, I42-A004
- **I-004/AC-022** [AC] (DEEP-I-004.discovery.md:L532| unit=RELATIONAL): ambiguity/determinism | I-42 requires Orchestrate-only writes and identical verdict bytes but does not specify an artifact-equivalent fixture; operator edit ingestion is also undefined. | anchor=I42-A003, I42-A005
- **I-004/AC-023** [AC] (DEEP-I-004.discovery.md:L533| unit=RELATIONAL): ambiguity/scope | I-45 alternates between every PASS and every sealed verdict, and its event/error identity plus LEGACY success semantics are unresolved. | anchor=I45-A001, I45-A003–I45-A005
- **I-004/AC-024** [AC] (DEEP-I-004.discovery.md:L534| unit=RELATIONAL): ambiguity/determinism | I-45 requires canonical bytes across multiple sources/consumers but omits canonical field/Unicode/precedence/version rules. | anchor=I45-I001, I45-A002
- **I-005/DII005-AMB-001** [AC] (DEEP-I-005.discovery.md:L413| unit=RELATIONAL SURFACE): `[A][I6 protocol]` The acknowledgement tag's exact grammar and the size of the “first reply token window” are unspecified. — I6:L15-L16, I6:L41-L42, I6:L84-L87
- **I-005/DII005-AMB-009** [AC] (DEEP-I-005.discovery.md:L421| unit=RELATIONAL SURFACE): `[A][I25 enable flag]` `ELAI_ROUTE_DECISION_SSB_ENABLE` explicitly gates event emission, while the wargame also mentions a default-off profiler config without naming whether it is the same or a separate switch. — I25:L27-L28, I25:L80-L82, I25:L140-L143
- **I-005/DII005-CON-002** [CON] (DEEP-I-005.discovery.md:L437| unit=RELATIONAL SURFACE): `[C][I6 protocol]` Summary `I-6.06` loses “highest-priority,” supervisor validation, first-token-window, and retry-on-missing semantics. It must be narrowed/expanded to the original contract. — summary:L230-L231; I6:L15-L16, I6:L41-L42
- **I-005/DII005-CON-003** [CON] (DEEP-I-005.discovery.md:L438| unit=RELATIONAL SURFACE): `[C][I7 UI status]` Summary `I-7.09` presents the cost-cockpit trace as an active feature, but the revised original explicitly defers it until the GD-4 bound is measured. — summary:L253; I7:L51-L54, I7:L87-L90
- **I-005/DII005-CON-009** [CON] (DEEP-I-005.discovery.md:L444| unit=RELATIONAL SURFACE): `[C][I25 UI status]` Summary `I-25.12` says route explanations are shown in the cockpit; the source defers all dashboard/Svelte work under `owner-ui-rework-hold`. — summary:L704; I25:L52-L56, I25:L92-L93
- **I-005/DII005-CON-011** [CON] (DEEP-I-005.discovery.md:L446| unit=RELATIONAL SURFACE): `[C][I25 run count]` Summary `I-25.14` requires at least three runs; the original requires an ELAI mixed-batch comparison but gives no three-run minimum. — summary:L707-L708; I25:L15-L18, I25:L100-L105
- **I-005/DII005-CON-015** [CON] (DEEP-I-005.discovery.md:L450| unit=RELATIONAL SURFACE): `[C][I39 event trigger]` Summary `I-39.15` attaches `PlanStateDerived` to successful archive replay; the original attaches it to the production `elai plan status` query and says migration replay reuses existing rows. — summary:L1074; I39:L39, I39:L59, I39:L67-L70
- **I-006/IX-019** [IX] (DEEP-I-006.discovery.md:L480| unit=RELATIONAL SURFACE): E | I-53 recipe commands must pass E-30/bash validation and are only a future I-46 redirect target; neither feature may bypass the existing command gate. | anchor=I53-E014, I53-E017, I53-E021, I53-E023
- **I-006/IX-024** [IX] (DEEP-I-006.discovery.md:L485| unit=RELATIONAL SURFACE): E | UI posture is additive/minimal: I-17/I-19/I-24/I-26/I-49/I-53 reuse existing surfaces, I-16/I-47 justify none, I-30 defers its drawer, and I-43 makes its tile optional. | anchor=I16-E016; I17-E023; I19-E017; I24-E017; I26-E021; I30-E021, I30-E033; I43-E022; I47-E012; I49-E016; I53-E015
- **I-006/AC-002** [AC] (DEEP-I-006.discovery.md:L498| unit=RELATIONAL): ambiguity/measurement | I-16 leaves adoption band/VRAM threshold and matrix-to-single-artifact aggregation unspecified. | anchor=I16-A001–I16-A002
- **I-006/AC-003** [AC] (DEEP-I-006.discovery.md:L499| unit=RELATIONAL): ambiguity/recovery | I-17 omits flake rerun count, persistence, rehabilitation, and L3 variance threshold. | anchor=I17-A001–I17-A002
- **I-006/AC-004** [AC] (DEEP-I-006.discovery.md:L500| unit=RELATIONAL): ambiguity/integration | I-17’s live hook locations, temporary calibration stub lifecycle, and `HaltingBound` schema remain unresolved. | anchor=I17-A003–I17-A004
- **I-006/AC-006** [AC] (DEEP-I-006.discovery.md:L502| unit=RELATIONAL SURFACE): ambiguity/language | I-19 names TypeScript scope without a TS dry-run command and leaves attribution/name matching undefined. | anchor=I19-A002–I19-A003
- **I-006/AC-007** [AC] (DEEP-I-006.discovery.md:L503| unit=RELATIONAL): ambiguity/event | `RefactorExempted` is non-blocking but lives under `TddPreflightBlocked`. | anchor=I19-A004
- **I-006/AC-008** [AC] (DEEP-I-006.discovery.md:L504| unit=RELATIONAL): ambiguity/config | I-24 omits `p`, `N`, threshold calibration/version, reset, and reverse-direction score-delta behavior. | anchor=I24-A001–I24-A003
- **I-006/AC-010** [AC] (DEEP-I-006.discovery.md:L506| unit=RELATIONAL): ambiguity/schema | I-26’s budget constants, schema migration, collapse event owner, and provider-version mapping are open. | anchor=I26-A001–I26-A003
- **I-006/AC-014** [AC] (DEEP-I-006.discovery.md:L510| unit=RELATIONAL): ambiguity/eligibility | I-30’s verified-COMPLETE winner model is not reconciled with hard-band qualifying partial trajectories. | anchor=I30-A002
- **I-006/AC-015** [AC] (DEEP-I-006.discovery.md:L511| unit=RELATIONAL): ambiguity/gate | I-30 alternates among block, warning, and error for missing ID-6 evidence; JSONL/event schemas and calibration remain incomplete. | anchor=I30-A001, I30-A003–I30-A005
- **I-006/AC-017** [AC] (DEEP-I-006.discovery.md:L513| unit=RELATIONAL): ambiguity/kill | I-43 does not quantify rough/flat/block-everything criteria, reconcile N=50 with 50–100, or define how off-tool F0.3 completes the memo. | anchor=I43-A001–I43-A003
- **I-006/AC-018** [AC] (DEEP-I-006.discovery.md:L514| unit=RELATIONAL): ambiguity/artifact | I-43 leaves bench artifact and divergence/wording schemas undefined. | anchor=I43-A004–I43-A005
- **I-006/AC-020** [AC] (DEEP-I-006.discovery.md:L516| unit=RELATIONAL): ambiguity/protocol | I-49 leaves shim placement, API/crate choices, credential acquisition, IDs/errors, J-14 compatibility, and fixture-vs-production reach unresolved. | anchor=I49-A001–I49-A006
- **I-006/AC-021** [AC] (DEEP-I-006.discovery.md:L517| unit=RELATIONAL): ambiguity/tooling | I-53 leaves recipe parsing/execution semantics, JSON caps, archive bomb/path handling, and dependency fallback scope unresolved. | anchor=I53-A001–I53-A004

### Surfaces — SD (2)

- **I-002/I002-SD-012** [SD] (DEEP-I-002.discovery.md:L747| unit=SD SURFACE): [correction][I-5 UI] The summary says I-5 adds a ReasoningDrawer tile; the detail file makes it Tier 2, wire-dark, and W-6-justifiably absent in v1 until at least three consistent A/B runs and GD-4 evidence. Deep anchors: I-5 lines 47-50, 57-58, 81-83.
- **I-002/I002-SD-029** [SD] (DEEP-I-002.discovery.md:L764| unit=SD SURFACE): [addition][I-29 event] The full event enum/badge includes `DraftAccepted`, which the oriented summary's observability bullet omitted while naming only veto and merge. Deep anchors: I-29 lines 85-90, 99.

### Surfaces — Track 12 (3)

- **SUMMARY/I-1.13** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L110| unit=Track 12 SURFACE): [E][UI] Shows supervisor verdict rows in an existing drawer.
- **SUMMARY/I-2.12** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L136| unit=Track 12 SURFACE): [E][UI] Displays branch-workspace badges on BranchTree.
- **SUMMARY/I-3.09** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L159| unit=Track 12 SURFACE): [E][UI] Has no live UI; the metrics TOML is the justified observable surface.

### Surfaces — Track 13 (7)

- **SUMMARY/I-4.14** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L189| unit=Track 13 SURFACE): [E][UI] Feeds SearchNodeEvent rows to the H15-1 branch-board comparison view.
- **SUMMARY/I-5.09** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L213| unit=Track 13 SURFACE): [E][UI] Adds a vividness-tax tile to ReasoningDrawer.
- **SUMMARY/I-6.09** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L234| unit=Track 13 SURFACE): [E][UI] Has no separate UI because it is prompt-internal.
- **SUMMARY/I-7.09** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L253| unit=Track 13 SURFACE): [E][UI] Shows the cascade trace in the cost cockpit.
- **SUMMARY/I-8.11** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L275| unit=Track 13 SURFACE): [E][UI] Displays an abort/dead-end strip in ReasoningDrawer.
- **SUMMARY/I-9.09** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L293| unit=Track 13 SURFACE): [E][CLI] Adds elai bench error-amplification.
- **SUMMARY/I-9.11** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L295| unit=Track 13 SURFACE): [E][UI] Adds an alpha-trend cockpit tile.

### Surfaces — Track 14 (10)

- **SUMMARY/I-10.17** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L327| unit=Track 14 SURFACE): [E][UI] Adds TTL and economics fields to the H-9 cost tile.
- **SUMMARY/I-11.15** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L353| unit=Track 14 SURFACE): [E][UI] Adds a BeliefState inspector row to SessionsDrawer.
- **SUMMARY/I-12.12** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L378| unit=Track 14 SURFACE): [E][UI] Displays playbook rows in RulesDrawer.
- **SUMMARY/I-12.16** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L382| unit=Track 14 SURFACE): [E][CLI] elai playbook export writes an AGENTS.md block.
- **SUMMARY/I-13.09** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L400| unit=Track 14 SURFACE): [E][serialization] Defines to_ui_dict as the full dashboard representation.
- **SUMMARY/I-13.13** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L404| unit=Track 14 SURFACE): [E][UI] Adds guard badges to MemoryFactsDrawer.
- **SUMMARY/I-13.16** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L407| unit=Track 14 SURFACE): [E][behavior] A 500-file listing remains complete in the dashboard but enters model context in condensed form.
- **SUMMARY/I-14.13** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L431| unit=Track 14 SURFACE): [E][UI] Displays transform receipts in the Transcript drawer.
- **SUMMARY/I-15.09** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L450| unit=Track 14 SURFACE): [E][UI] Shows a cache-source badge on research answers.
- **SUMMARY/I-16.12** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L473| unit=Track 14 SURFACE): [E][CLI] elai bench kv-tune writes KvCacheTuned with vram_gb, tok_s, and fixture_pass_rate.

### Surfaces — Track 15 (10)

- **SUMMARY/I-17.20** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L507| unit=Track 15 SURFACE): [E][UI] Shows per-branch verdict chips.
- **SUMMARY/I-18.14** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L535| unit=Track 15 SURFACE): [E][CLI] Extends elai verify-attestation with chain and Rekor options.
- **SUMMARY/I-18.15** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L536| unit=Track 15 SURFACE): [E][UI] Adds a chain badge to the RRA viewer.
- **SUMMARY/I-19.12** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L562| unit=Track 15 SURFACE): [E][UI] Uses the existing gate panel for block reasons.
- **SUMMARY/I-20.17** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L592| unit=Track 15 SURFACE): [E][UI] Shows a per-hunk mutation chip.
- **SUMMARY/I-20.21** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L597| unit=Track 15 SURFACE): [E][measurement] Thresholds come from ELAI measurements, not paper constants.
- **SUMMARY/I-21.16** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L620| unit=Track 15 SURFACE): [E][UI] Adds a Sprint Contract card beside PlanApprovalCard.
- **SUMMARY/I-22.13** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L643| unit=Track 15 SURFACE): [E][UI] Shows a per-branch progress sparkline.
- **SUMMARY/I-23.08** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L660| unit=Track 15 SURFACE): [E][UI] Adds no new UI and preserves existing refusal surfaces.
- **SUMMARY/I-24.10** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L681| unit=Track 15 SURFACE): [E][UI] Adds a scorer-health badge.

### Surfaces — Track 16 (7)

- **SUMMARY/I-25.12** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L704| unit=Track 16 SURFACE): [E][UI] Shows route explanations in the cost cockpit.
- **SUMMARY/I-26.12** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L728| unit=Track 16 SURFACE): [E][UI] Shows a branch-board budget bar.
- **SUMMARY/I-27.09** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L746| unit=Track 16 SURFACE): [E][CLI] Adds elai bench convergence with a fixture suite argument.
- **SUMMARY/I-27.10** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L747| unit=Track 16 SURFACE): [E][UI] Adds a convergence-curve cockpit tile.
- **SUMMARY/I-28.11** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L767| unit=Track 16 SURFACE): [E][UI] Optionally displays credit heat on the plan view; absence may be W-6 justified.
- **SUMMARY/I-29.10** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L790| unit=Track 16 SURFACE): [E][UI] Adds a draft-lane badge to generation cards.
- **SUMMARY/I-29.13** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L794| unit=Track 16 SURFACE): [E][feature-flag] With the lane flag absent, existing paths are untouched.

### Surfaces — Track 17 (3)

- **SUMMARY/I-30.11** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L817| unit=Track 17 SURFACE): [E][CLI] Adds elai dataset export --format sft.
- **SUMMARY/I-30.13** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L819| unit=Track 17 SURFACE): [E][UI] Adds a dataset card with counts and provenance.
- **SUMMARY/I-31.23** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L856| unit=Track 17 SURFACE): [E][UI] Shows flywheel admits, blocks, and KL trend.

### Surfaces — Track 18 (18)

- **SUMMARY/I-32.13** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L883| unit=Track 18 SURFACE): [E][UI] Adds no UI because the change is prompt-internal.
- **SUMMARY/I-33.17** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L912| unit=Track 18 SURFACE): [E][UI] Adds a governance-metrics tile.
- **SUMMARY/I-34.15** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L937| unit=Track 18 SURFACE): [E][UI] Shows grammar-pin status in the existing decode panel.
- **SUMMARY/I-35.14** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L965| unit=Track 18 SURFACE): [E][UI] Shows sequence-verdict rows in the gate panel.
- **SUMMARY/I-36.05** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L982| unit=Track 18 SURFACE): [E][validation] Validates variants on ELAI fixtures.
- **SUMMARY/I-36.12** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L989| unit=Track 18 SURFACE): [E][UI] Shows a per-role compiled-variant badge in SettingsWindow.
- **SUMMARY/I-37.07** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1007| unit=Track 18 SURFACE): [E][UI] Surfaces requirement status per hunk in the reviewer cockpit.
- **SUMMARY/I-37.10** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1010| unit=Track 18 SURFACE): [E][trust] Treats imported formats as intent sources while ELAI separately proves satisfaction.
- **SUMMARY/I-38.17** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1045| unit=Track 18 SURFACE): [E][UI] Places a QDELTA tile beside SAFEDELTA.
- **SUMMARY/I-38.18** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1046| unit=Track 18 SURFACE): [E][CLI] elai bench qdelta emits QdeltaMeasured with on/off delta and confidence interval.
- **SUMMARY/I-39.01** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1057| unit=Track 18 SURFACE): [E][CLI] Adds elai plan status.
- **SUMMARY/I-39.11** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1068| unit=Track 18 SURFACE): [E][UI] Adds no separate UI because the CLI and generated document are the load-bearing artifacts.
- **SUMMARY/I-39.12** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1070| unit=Track 18 SURFACE): [E][CLI] Letter-filtered status output carries fragment path, checker verdict, and journal row for each session.
- **SUMMARY/I-40.04** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1086| unit=Track 18 SURFACE): [E][wrapper] Wraps an external agent through a minimal proxy/MCP surface.
- **SUMMARY/I-40.05** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1087| unit=Track 18 SURFACE): [E][wrapper] Routes external-agent tool traffic through ELAI taint tagging.
- **SUMMARY/I-40.06** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1088| unit=Track 18 SURFACE): [E][wrapper] Routes external-agent writes through ELAI write gates.
- **SUMMARY/I-40.14** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1097| unit=Track 18 SURFACE): [E][CLI] Adds elai wrap -- <agent command>.
- **SUMMARY/I-40.16** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1099| unit=Track 18 SURFACE): [E][UI] Renders wrapped-run rows in the existing RRA viewer.

### Surfaces — Track 19 (9)

- **SUMMARY/I-41.23** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1135| unit=Track 19 SURFACE): [E][CLI] Adds elai critique <spec>.
- **SUMMARY/I-41.24** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1136| unit=Track 19 SURFACE): [E][UI] Shows nonblocking advisory findings in ReasoningDrawer or the framing panel.
- **SUMMARY/I-42.15** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1161| unit=Track 19 SURFACE): [E][CLI] Adds elai run --profile critique-only.
- **SUMMARY/I-42.16** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1162| unit=Track 19 SURFACE): [E][UI] Shows the active profile in the existing StatusBar mode cluster.
- **SUMMARY/I-43.18** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1189| unit=Track 19 SURFACE): [E][CLI] Adds elai bench claim-scope.
- **SUMMARY/I-43.19** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1190| unit=Track 19 SURFACE): [E][UI] Optionally shows a Phase-0 go/no-go cockpit panel; the memo may justify no UI.
- **SUMMARY/I-44.13** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1219| unit=Track 19 SURFACE): [E][invariant] elai replay --verify must still pass for hash-checkable steps.
- **SUMMARY/I-44.16** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1222| unit=Track 19 SURFACE): [E][CLI] Adds elai run --zero-retention.
- **SUMMARY/I-44.17** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1223| unit=Track 19 SURFACE): [E][UI] Shows RetentionPurged rows in the RRA viewer or retention drawer.

### Surfaces — Track 20 (17)

- **SUMMARY/I-45.12** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1249| unit=Track 20 SURFACE): [E][UI] Shows a stale-receipt badge in the RRA viewer.
- **SUMMARY/I-46.08** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1267| unit=Track 20 SURFACE): [E][fixup] Applies a frozen deterministic table for known-broken command shapes.
- **SUMMARY/I-46.14** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1273| unit=Track 20 SURFACE): [E][UI] Shows redirect rows in the tool-call drawer.
- **SUMMARY/I-46.16** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1275| unit=Track 20 SURFACE): [E][test] A redirected command must spawn no process.
- **SUMMARY/I-47.15** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1299| unit=Track 20 SURFACE): [E][UI] Adds no UI because the behavior is prompt-internal.
- **SUMMARY/I-48.12** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1320| unit=Track 20 SURFACE): [E][UI] Adds no UI because it is a build-time refactor.
- **SUMMARY/I-49.15** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1342| unit=Track 20 SURFACE): [E][constraint] Leaves oauth and existing CLI bridges on the keep list.
- **SUMMARY/I-49.16** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1343| unit=Track 20 SURFACE): [E][integration] Registers bedrock.rs, azure_openai.rs, and a shim under the runtime adapter surface.
- **SUMMARY/I-49.17** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1345| unit=Track 20 SURFACE): [E][UI] Lists both backends in the existing RuntimePanel.
- **SUMMARY/I-49.18** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1346| unit=Track 20 SURFACE): [E][CLI] elai runtime list reports both lanes.
- **SUMMARY/I-50.15** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1373| unit=Track 20 SURFACE): [E][UI] Shows auth status in the existing RuntimePanel or Settings.
- **SUMMARY/I-51.02** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1386| unit=Track 20 SURFACE): [E][field] Stores context window.
- **SUMMARY/I-51.18** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1402| unit=Track 20 SURFACE): [E][UI] Adds no UI behind preserved APIs.
- **SUMMARY/I-52.14** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1426| unit=Track 20 SURFACE): [E][feature-flag] Inherits W-2 posture if the checkpoint substrate remains off.
- **SUMMARY/I-52.15** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1427| unit=Track 20 SURFACE): [E][UI] Shows checkpoint rows in the existing session drawer.
- **SUMMARY/I-53.17** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1453| unit=Track 20 SURFACE): [E][UI] Uses the existing tool-call drawer.
- **SUMMARY/I-54.21** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1485| unit=Track 20 SURFACE): [E][UI] Shows preview and apply rows in the existing tool-call drawer.

### Surfaces — misc (5)

- **I-003/DI003-X-011** [IX] (DEEP-I-003.discovery.md:L933| unit=misc SURFACE): [E][UI reuse/new surfaces] I-1, I-20, I-32, I-41, I-50, and I-52 reuse existing surfaces; I-33 and I-38 require new metric tiles; I-4 and I-9 defer their principal visualizations. Refs: DI003-I01-024, DI003-I04-029, DI003-I09-021, DI003-I20-026, DI003-I32-019, DI003-I33-021, DI003-I38-023, DI003-I41-033, DI003-I50-016, DI003-I52-019.
- **I-003/REG-002** [AC] (DEEP-I-003.discovery.md:L946| unit=misc SURFACE): [A] I-1's exact existing drawer. Ref: DI003-I01-032.
- **I-003/REG-006** [CON] (DEEP-I-003.discovery.md:L950| unit=misc SURFACE): [C] I-9 grammar and UI are deferred despite summary-level capability wording. Ref: DI003-I09-031.
- **I-003/REG-017** [AC] (DEEP-I-003.discovery.md:L964| unit=misc SURFACE): [A] I-41 drawer and dedup normalization are unresolved. Refs: DI003-I41-043, DI003-I41-044.
- **I-003/REG-019** [AC] (DEEP-I-003.discovery.md:L967| unit=misc SURFACE): [A] I-52 event-family and flag schema depend on edit-time state. Refs: DI003-I52-030, DI003-I52-031.

## Per-ledger atomic records

Feature and unit-local records grouped by ledger then unit. Cross-file interactions, summary deltas, and ambiguity/conflict registers are included under each ledger (namespaced).

### Ledger I-001 — I-GROUNDING-MAP, I-8, I-13, I-15, I-18, I-23, I-28, I-36, I-37, I-48

Source: `docs/thefinalglance/feature-discovery/deep/DEEP-I-001.discovery.md`
Extracted atoms: **435**

#### I-001 / unit `I-GROUNDING` (85 records)

- **I-001/IG-E001** [SRC] (DEEP-I-001.discovery.md:L37| unit=I-GROUNDING): E/STATUS | The map was grounded 2026-06-10 from live read-only source greps, the 13-file prefable-research corpus, B–H summaries, the wire-remediation handoff, and the finish index. | anchor=IG:L1-L6
- **I-001/IG-E002** [SRC] (DEEP-I-001.discovery.md:L38| unit=I-GROUNDING): E/TEST | elai.map was intentionally not regenerated because it was stale versus concurrent H edits; every load-bearing anchor was source-grep confirmed and executors must re-grep at EXECUTING under W-7. | anchor=IG:L3-L7
- **I-001/IG-E003** [SRC] (DEEP-I-001.discovery.md:L39| unit=I-GROUNDING): E/DATA | The dated live constants were SCHEMA_VERSION=221 at ssb.rs:2981 and SCHEMA_VERSION_PUBLIC at :2986. | anchor=IG:L9-L12
- **I-001/IG-E004** [SRC] (DEEP-I-001.discovery.md:L40| unit=I-GROUNDING): E/DATA | The dated action registry held TOTAL_VARIANTS=532 at action_type_lock.rs:601 and ALL_VARIANTS at :610. | anchor=IG:L12-L13
- **I-001/IG-E005** [SRC] (DEEP-I-001.discovery.md:L41| unit=I-GROUNDING): E/CFG | Prior values 191/464, 207/484, 220/497, 224/501, 234/511, 236/513, 240/517, 209/520, and 522 are explicitly stale; every I-N must express a baseline-relative bump from a freshly reread live pair. | anchor=IG:L14-L16
- **I-001/IG-E006** [SRC] (DEEP-I-001.discovery.md:L42| unit=I-GROUNDING): E/FAIL | The pre-existing ssb_schema_v97_payloads::schema_version_is_97 test still asserts 97; its fix belongs to a patch-N, not an I-N. | anchor=IG:L17-L18
- **I-001/IG-E007** [SRC] (DEEP-I-001.discovery.md:L43| unit=I-GROUNDING): E/STATE | The SSB schema is golden-locked and append-only. | anchor=IG:L21-L23
- **I-001/IG-E008** [SRC] (DEEP-I-001.discovery.md:L44| unit=I-GROUNDING): E/STATE | darwin_admit_loop.rs currently derives fitness as blake3(config bytes), a placeholder that H-43 replaces; I-31 consumes execution-grounded fitness and never the byte hash. | anchor=IG:L24-L25
- **I-001/IG-E009** [SRC] (DEEP-I-001.discovery.md:L45| unit=I-GROUNDING): E/STATE | trap_tasks.rs currently uses keyword-grep trap scoring; H-44 replaces it and I-24/I-31 must compose H-44. | anchor=IG:L26-L26
- **I-001/IG-E010** [SRC] (DEEP-I-001.discovery.md:L46| unit=I-GROUNDING): E/STATE | config_archive.rs already exposes insert_variant, select_parent, and select_parent_with for the Darwin archive consumed by I-28/I-31. | anchor=IG:L27-L27
- **I-001/IG-E011** [SRC] (DEEP-I-001.discovery.md:L47| unit=I-GROUNDING): E/STATE | Anthropic CacheControl and ephemeral markers exist with a four-marker maximum, but the RuntimeAdapter path still supplied cache_control=None at grounding time; I-10 is only the TTL/economics/guard layer if H-1 lands first. | anchor=IG:L28-L30
- **I-001/IG-E012** [SRC] (DEEP-I-001.discovery.md:L48| unit=I-GROUNDING): E/STATE | context_budget.rs already provides blake3 prefix stability, adaptive_reduce, and nine ContextSlot values including Memory; I-6 adds its anchor slot last and I-5/I-14 must preserve breakpoint stability. | anchor=IG:L31-L33
- **I-001/IG-E013** [SRC] (DEEP-I-001.discovery.md:L49| unit=I-GROUNDING): E/STATE | Banded canary deltas and conformal calibration/bootstrap consumers already exist as the D-16/D-30 substrate for I-23. | anchor=IG:L34-L36
- **I-001/IG-E014** [SRC] (DEEP-I-001.discovery.md:L50| unit=I-GROUNDING): E/STATE | RelationLibrary, MutationRunner, and a blake3-keyed 30-day EquivalentMutantCache already exist; I-20 extends both testing systems and I-24 extends the relation library. | anchor=IG:L37-L39
- **I-001/IG-E015** [SRC] (DEEP-I-001.discovery.md:L51| unit=I-GROUNDING): E/RECOVERY | bsp/engine.rs recover_from_checkpoint and a test exist, correcting the stale “stub” framing; production reachability remained unverified and must be retraced. | anchor=IG:L40-L41
- **I-001/IG-E016** [SRC] (DEEP-I-001.discovery.md:L52| unit=I-GROUNDING): E/STATE | build_rra/verify_rra and attestation.rs already exist; I-18 extends them with row chaining, Rekor push, and AI-BOM rather than rebuilding them. | anchor=IG:L42-L43
- **I-001/IG-E017** [SRC] (DEEP-I-001.discovery.md:L53| unit=I-GROUNDING): E/SEC | Taint/provenance substrate exists in capability_flow.rs and the dedicated safety crate; G-1 ingest liveness is the composition seam for I-13/I-35/I-40. | anchor=IG:L44-L46
- **I-001/IG-E018** [SRC] (DEEP-I-001.discovery.md:L54| unit=I-GROUNDING): E/STATE | DecodingMode::Grammar already carries ebnf_path and crane_adaptive, but crane_adaptive only affects eligibility because the CRANE runtime is absent; I-34 builds that runtime leg. | anchor=IG:L47-L49
- **I-001/IG-E019** [SRC] (DEEP-I-001.discovery.md:L55| unit=I-GROUNDING): E/STATE | TemplateCrystallizer and CompiledPipelineStore already ship; they provide the zero-token curator seam for I-12 and the storage precedent for I-36. | anchor=IG:L50-L51
- **I-001/IG-E020** [SRC] (DEEP-I-001.discovery.md:L56| unit=I-GROUNDING): E/INT | production_agent_rpc.rs build_tool_catalog exposes the three-arm convert/dispatch tool-array pattern; I-10 must preserve its prefix stability and I-35 gates it. | anchor=IG:L52-L53
- **I-001/IG-E021** [SRC] (DEEP-I-001.discovery.md:L57| unit=I-GROUNDING): E/STATE | IntentPair, IntentPairSet, and IntentPairsGate already ship as the substrate generalized by I-37. | anchor=IG:L54-L55
- **I-001/IG-E022** [SRC] (DEEP-I-001.discovery.md:L58| unit=I-GROUNDING): E/STATE | The named Track-3/4 ledger, budget, deflation, self-validation, drift, leakage, null, and manifest gate files exist; ID-6 binds weight/prompt promotions to them. | anchor=IG:L56-L58
- **I-001/IG-E023** [SRC] (DEEP-I-001.discovery.md:L59| unit=I-GROUNDING SURFACE): E/STATE | The workspace already contains safety, memory, eval-corpus, ssb-otel-export, web_search, web_validator, agent, ipc-types, scaffold, dashboard-v2, and parked/backup dashboard crates. | anchor=IG:L59-L60
- **I-001/IG-E024** [SRC] (DEEP-I-001.discovery.md:L60| unit=I-GROUNDING): E/STATUS | streaming_validator, batch-message, memory retrieval, and conformal-calibration tokens exist at file level, but their wiring depth was not verified and must be retraced under R7. | anchor=IG:L61-L64
- **I-001/IG-E025** [SRC] (DEEP-I-001.discovery.md:L61| unit=I-GROUNDING): E/OBS | Wiring and freeze tooling exists: wiring_contract_check.py, gate_freeze_check.py, wire-dark allowlist, failure_taxonomy.py, and impact_radius.py. | anchor=IG:L65-L66
- **I-001/IG-E026** [SRC] (DEEP-I-001.discovery.md:L62| unit=I-GROUNDING): E/STATUS | The dated static D/E/F/G tally was 34 wired, 21 dark, 50 partial, and 3 empty-contract, with no clean wiring flips left; remaining work was feature engineering, GPU, or new dependencies. | anchor=IG:L66-L69
- **I-001/IG-E027** [SRC] (DEEP-I-001.discovery.md:L63| unit=I-GROUNDING): E/STATUS | No MCTS module exists; I-4 is a genuine ladder build rather than a day-one tree. | anchor=IG:L71-L75
- **I-001/IG-E028** [SRC] (DEEP-I-001.discovery.md:L64| unit=I-GROUNDING): E/STATUS | No typed BeliefState exists; I-11 is a build. | anchor=IG:L76-L76
- **I-001/IG-E029** [SRC] (DEEP-I-001.discovery.md:L65| unit=I-GROUNDING): E/STATUS | No AnchorPane/privileged-last slot exists among the nine ContextSlots; I-6 is a build. | anchor=IG:L77-L77
- **I-001/IG-E030** [SRC] (DEEP-I-001.discovery.md:L66| unit=I-GROUNDING): E/STATUS | No sprint_contract module exists; I-21 builds one on H-37/H-38 seams. | anchor=IG:L78-L78
- **I-001/IG-E031** [SRC] (DEEP-I-001.discovery.md:L67| unit=I-GROUNDING): E/STATUS | No per-row chain_prev_hash exists; I-18 builds that field. | anchor=IG:L79-L79
- **I-001/IG-E032** [SRC] (DEEP-I-001.discovery.md:L68| unit=I-GROUNDING): E/STATUS | No compression middleware exists; I-14 is a deterministic-first build. | anchor=IG:L80-L80
- **I-001/IG-E033** [SRC] (DEEP-I-001.discovery.md:L69| unit=I-GROUNDING): E/STATUS | No EARS requirement object exists; I-37 is a build. | anchor=IG:L81-L81
- **I-001/IG-E034** [SRC] (DEEP-I-001.discovery.md:L70| unit=I-GROUNDING): E/STATUS | No per-backend compiled-prompt artifact store exists; I-36 builds it using CompiledPipelineStore as precedent. | anchor=IG:L82-L82
- **I-001/IG-E035** [SRC] (DEEP-I-001.discovery.md:L71| unit=I-GROUNDING): E/STATUS | No QDELTA quality harness exists though G-6 SAFEDELTA exists; I-38 builds the sibling quality harness. | anchor=IG:L83-L84
- **I-001/IG-E036** [SRC] (DEEP-I-001.discovery.md:L72| unit=I-GROUNDING SURFACE): E/STATUS | No elai plan status repo-derived command exists; I-39 is a build. | anchor=IG:L85-L85
- **I-001/IG-E037** [SRC] (DEEP-I-001.discovery.md:L73| unit=I-GROUNDING): E/STATUS | No CEFL externalization, I6Q rationale-quality, or commit-reveal entropy primitives exist; I-33 builds them. | anchor=IG:L86-L86
- **I-001/IG-E038** [SRC] (DEEP-I-001.discovery.md:L74| unit=I-GROUNDING): E/STATUS | FSM masks, argv allowlists, and tool-set commitments are per-call gates; no temporal ordering-over-history rule exists, so I-35 builds it. | anchor=IG:L87-L88
- **I-001/IG-E039** [SRC] (DEEP-I-001.discovery.md:L75| unit=I-GROUNDING): E/STATUS | No cache-economics planner, TTL selector, or cache-hit hard guard exists; I-10 builds them on H-1/H-2. | anchor=IG:L89-L90
- **I-001/IG-E040** [SRC] (DEEP-I-001.discovery.md:L76| unit=I-GROUNDING): E/STATUS | No error-amplification alpha_t harness exists; I-9 builds it from drift and mutation components. | anchor=IG:L91-L91
- **I-001/IG-E041** [SD] (DEEP-I-001.discovery.md:L77| unit=I-GROUNDING): E/CORRECTION | I-7 interactive Batch API racing is architecturally rejected because batch runs take 10–45 minutes, have a 24-hour SLA, and cancellation does not refund queued work; corrected behavior is local-first, condense-then-escalate, with batch confined to offline lanes. | anchor=IG:L93-L100
- **I-001/IG-E042** [SD] (DEEP-I-001.discovery.md:L78| unit=I-GROUNDING): E/CORRECTION | I-14 is deterministic LSP slicing and tree-sitter folding first, replacing irrelevant bodies with an elision marker while preserving signatures; LLMLingua-2 is a gated prose-only Tier 2 because its XLM-R encoder consumes VRAM. | anchor=IG:L101-L104
- **I-001/IG-E043** [SRC] (DEEP-I-001.discovery.md:L79| unit=I-GROUNDING): E/LIMIT | CompactPrompt/n-gram code-symbol abbreviation is killed as neural poison; only narrow numeric quantization for tabular data survives as a Tier-2 option. | anchor=IG:L105-L106
- **I-001/IG-E044** [SD] (DEEP-I-001.discovery.md:L80| unit=I-GROUNDING): E/CORRECTION | Cache micro-heartbeats are last-resort only; I-10 primarily uses explicit one-hour frozen-prefix and five-minute volatile-tail TTLs plus wide-local/burst-cloud scheduling. | anchor=IG:L107-L110
- **I-001/IG-E045** [SD] (DEEP-I-001.discovery.md:L81| unit=I-GROUNDING): E/CORRECTION | H-4 tool deferral and I-10 prefix invariance are both valid levers; catalog choice must be based on journal-measured cache-hit-times-token cost, with either inline-frozen small catalogs or byte-stable deferral structure for large catalogs. | anchor=IG:L111-L114
- **I-001/IG-E046** [SD] (DEEP-I-001.discovery.md:L82| unit=I-GROUNDING): E/CORRECTION | I-4 is a value-yielding ladder: v1 best-of-N with deterministic verifier ranking, v2 tournament, v3 PUCT with virtual loss; the XL tree does not block Tracks 14/15. | anchor=IG:L115-L117
- **I-001/IG-E047** [SD] (DEEP-I-001.discovery.md:L83| unit=I-GROUNDING): E/CORRECTION | Track 17 is resequenced: verifier-ranked rejection-sampling I-30 precedes I-31 GRPO, and I-31 is blocked by canaries, tamper gates, ledger promotion, KL penalty, and an evaluator outside the write boundary. | anchor=IG:L118-L122
- **I-001/IG-E048** [SRC] (DEEP-I-001.discovery.md:L84| unit=I-GROUNDING): E/DEP | The canonical local model remains Qwen2.5-Coder; any MoE retarget is a RunPod-gated evaluation, never an assumed swap. | anchor=IG:L120-L122
- **I-001/IG-E049** [SD] (DEEP-I-001.discovery.md:L85| unit=I-GROUNDING): E/CORRECTION | I-11's differentiator over commodity BeliefState patterns is typed, DKG-linked, journaled, and honestly measured against periodic full-history reruns. | anchor=IG:L123-L124
- **I-001/IG-E050** [SD] (DEEP-I-001.discovery.md:L86| unit=I-GROUNDING): E/CORRECTION | I-2 is cross-platform: overlayfs on Linux, pooled warm git worktrees on Windows/macOS, and an optional cloud-sandbox adapter behind one trait. | anchor=IG:L125-L127
- **I-001/IG-E051** [SD] (DEEP-I-001.discovery.md:L87| unit=I-GROUNDING): E/CORRECTION | I-13 must compose H-31/H-32, E-37, and memory MINJA; net-new scope is write-time deterministic scanning, dual serialization, and TraceQuery, with regex/Aho-Corasick before any approved YARA dependency. | anchor=IG:L128-L131
- **I-001/IG-E052** [SD] (DEEP-I-001.discovery.md:L88| unit=I-GROUNDING): E/CORRECTION | I-18 is an extension of shipped RRA/attestation: per-row previous-hash chain, five-layer stack hash, optional Rekor, and AI-BOM for the named AISVS expectation. | anchor=IG:L132-L134
- **I-001/IG-E053** [SD] (DEEP-I-001.discovery.md:L89| unit=I-GROUNDING): E/CORRECTION | I-19 does not rebuild TDD; it adds a symbol-level pre-tool source-edit gate with a coverage-delta refactor exemption and must bound false positives before default-on. | anchor=IG:L135-L137
- **I-001/IG-E054** [SD] (DEEP-I-001.discovery.md:L90| unit=I-GROUNDING): E/CORRECTION | I-23 unifies shipped conformal calibration across refusal, claim scores, and I-4 priors; it introduces no Brier implementation or new math. | anchor=IG:L138-L140
- **I-001/IG-E055** [SD] (DEEP-I-001.discovery.md:L91| unit=I-GROUNDING): E/CORRECTION | I-20's real gaps are diff-cone mutant selection, time-boxed sampled scoring with a confidence interval, and surviving-mutant regeneration directives because an equivalent-mutant cache already ships. | anchor=IG:L141-L143
- **I-001/IG-E056** [SD] (DEEP-I-001.discovery.md:L92| unit=I-GROUNDING): E/CORRECTION | The old checkpoint-recovery “stub” claim is stale; remaining work is production reachability and resume-from-journal UX owned by H-35/H-51. | anchor=IG:L144-L145
- **I-001/IG-E057** [SRC] (DEEP-I-001.discovery.md:L93| unit=I-GROUNDING): E/STATUS | Trust-spine wiring is substantially executed; I sessions must not restate it, must put trust-reinforcing work before search/flywheel, and must bind I-31 to its gates. | anchor=IG:L146-L150
- **I-001/IG-E058** [SRC] (DEEP-I-001.discovery.md:L94| unit=I-GROUNDING): E/SEC | Narrative personas are removed from factual/code roles but retained for alignment-dependent hostile-critic/safety roles; I-32 mechanizes that split. The carried evidence is -3.6 to -5.3 percentage points on MMLU and -0.65 MT-Bench coding, not independently reverified here. | anchor=IG:L151-L153
- **I-001/IG-E059** [SRC] (DEEP-I-001.discovery.md:L95| unit=I-GROUNDING): E/SEC | I-34 must use CRANE-style free-thinking-between-commitments, constrain final output only in reflection loops, post-parse revalidate, and pin grammars as control-plane artifacts. Carried risks include 10–15-point reasoning loss, 38 XGrammar silent failures, and CDA attack success of 96.2% / 75.8–99.5%; none was independently reverified here. | anchor=IG:L154-L159
- **I-001/IG-E060** [SRC] (DEEP-I-001.discovery.md:L96| unit=I-GROUNDING): E/EVIDENCE | The 13 research files are themselves web-grounded but this session did not independently verify their identifiers or vendor numbers; every cited arXiv ID must be web-confirmed before heading a gate or metric. | anchor=IG:L161-L172
- **I-001/IG-E061** [SRC] (DEEP-I-001.discovery.md:L97| unit=I-GROUNDING): E/CFG | Dated cache economics: Anthropic's default TTL changed from one hour to five minutes on 2026-03-06, one-hour writes cost 2x and break even around three reads, while five-minute writes cost 1.25x and break even at one read. | anchor=IG:L173-L176
- **I-001/IG-E062** [SRC] (DEEP-I-001.discovery.md:L98| unit=I-GROUNDING): E/CFG | TPM is input plus max_tokens; max_tokens must be clamped to schema size, and 60 RPM is enforced sub-second at roughly one request per second. | anchor=IG:L174-L176
- **I-001/IG-E063** [SRC] (DEEP-I-001.discovery.md:L99| unit=I-GROUNDING): E/CFG | Opus 4.7+ rejects manual thinking budget_tokens and uses effort; batch lanes are 10–45 minutes with a 24-hour SLA. | anchor=IG:L176-L177
- **I-001/IG-E064** [SRC] (DEEP-I-001.discovery.md:L100| unit=I-GROUNDING): E/DET | temperature=0 is explicitly non-deterministic, with cited variance figures carried only as dated research landmines. | anchor=IG:L177-L178
- **I-001/IG-E065** [SRC] (DEEP-I-001.discovery.md:L101| unit=I-GROUNDING): E/EVIDENCE | SWE-bench Verified is treated as retired/contaminated, K-Prize as contamination-free but low-performing, and OPRO-style self-optimization as unsuitable below roughly 100B; these figures are carried, not independently verified. | anchor=IG:L177-L178
- **I-001/IG-E066** [SRC] (DEEP-I-001.discovery.md:L102| unit=I-GROUNDING): E/INT | Track 12 anchors I-1 to task-runner/hook/safety seams, I-2 to BSP/sandbox/worktrees, and I-3 to SSB pragmas/checkpoint writes. | anchor=IG:L180-L184
- **I-001/IG-E067** [SRC] (DEEP-I-001.discovery.md:L103| unit=I-GROUNDING): E/INT | Track 13 anchors I-4 to SessionDag/verifiers/catalog, I-5/I-6 to ContextBudget, I-7 to batches/routing, I-8 to calibration/refusal/abstention, and I-9 to mutation/drift comparison. | anchor=IG:L185-L187
- **I-001/IG-E068** [SRC] (DEEP-I-001.discovery.md:L104| unit=I-GROUNDING): E/INT | Track 14 anchors I-10 to cache substrates, I-11 to BeliefState/context/routing, I-12 to compiled pipelines/rules/GEPA, I-13 to safety/memory, I-14 to LSP/tree-sitter, I-15 to Oracle cache/A-RAG, and I-16 to the Python sidecar. | anchor=IG:L188-L191
- **I-001/IG-E069** [SRC] (DEEP-I-001.discovery.md:L105| unit=I-GROUNDING): E/INT | Track 15 anchors I-17 to verifier/gate/mutation seams, I-18 to RRA/SSB, I-19 to pre-tool hooks, I-20 to mutation/impact radius, I-21 to interactive gate/adversary, I-22 to PPM/undo-tree, I-23 to conformal calibration, and I-24 to relations/trap detection. | anchor=IG:L192-L195
- **I-001/IG-E070** [SRC] (DEEP-I-001.discovery.md:L106| unit=I-GROUNDING): E/INT | Track 16 anchors I-25 to escalation/cloud routing, I-26 to serial depth/cost, I-27 to metrics/benchmarks, I-28 to archive/Phase 4H, and I-29 to MiniCritic/research with media/CRDT absent. | anchor=IG:L196-L198
- **I-001/IG-E071** [SRC] (DEEP-I-001.discovery.md:L107| unit=I-GROUNDING): E/INT | Track 17 anchors I-30 to RRA/ledger/SFT and I-31 to corrected fitness/trap scoring, ledger, held-out evaluation, and SFT. | anchor=IG:L199-L200
- **I-001/IG-E072** [SRC] (DEEP-I-001.discovery.md:L108| unit=I-GROUNDING SURFACE): E/INT | Track 18 anchors I-32 to agent configs, I-33 to rules/refusal, I-34 to decoding/grammar substrates, I-35 to temporal/per-call gates, I-36 to compiled-store/routing/optimizer, I-37 to intent/contract/cockpit/anchor, I-38 to quality/corpus probes, I-39 to repo/SSB/wiring/archive, and I-40 to RRA/provenance/adapters/bridge. | anchor=IG:L201-L205
- **I-001/IG-E073** [SRC] (DEEP-I-001.discovery.md:L109| unit=I-GROUNDING): E/EVIDENCE | The unverified citation list carried exact IDs for CaMeL 2503.18813, FIDES 2505.23643, MAST 2503.13657, scaling-agents 2512.08296, sycophancy-propagation 2604.02668, Compiled-AI 2604.05150, AgentSpec 2503.18666, Agent-C 2512.23738, Pro2Guard 2508.00500, CRANE 2502.09061, structure-snowballing 2604.06066, Let-Me-Speak-Freely 2408.02442, XGrammar 2411.15100, JSONSchemaBench 2501.10868, CFGzip 2605.04066, SELF-DISCOVER 2402.03620, and METR 2507.09089. | anchor=IG:L164-L172
- **I-001/IG-E074** [SRC] (DEEP-I-001.discovery.md:L110| unit=I-GROUNDING): E/EVIDENCE | The same unverified list carries LLMLingua-2, PABU/ACON at 26.9% steps, ACE +10.6%, DSPy/MIPROv2 +13%, StateFlow +13–28% and 3–5x, and persona-evaluation deltas. | anchor=IG:L167-L172
- **I-001/IG-I001** [INF] (DEEP-I-001.discovery.md:L111| unit=I-GROUNDING): I/STATUS | Because the map repeatedly labels live values as dated and mandates re-grep, none of its implementation-status or numeric-baseline statements can safely be treated as current after 2026-06-10. | anchor=IG:L2-L7, L9-L18, L61-L69
- **I-001/IG-A001** [AC] (DEEP-I-001.discovery.md:L112| unit=I-GROUNDING): A/EVIDENCE | The map lists external papers and vendor figures but explicitly leaves their identity, revision, and current accuracy unverified. | anchor=IG:L161-L178
- **I-001/IG-A002** [AC] (DEEP-I-001.discovery.md:L113| unit=I-GROUNDING): A/STATUS | File-level token presence for streaming, batches, memory retrieval, and calibration does not resolve production reachability. | anchor=IG:L61-L64
- **I-001/SD-IG-001** [SD] (DEEP-I-001.discovery.md:L525| unit=I-GROUNDING): ADD exact grounding method, stale-map warning, dated 221/532 constants, stale baseline list, and v97 test residue. | anchor=IG-E001–IG-E006
- **I-001/SD-IG-002** [SD] (DEEP-I-001.discovery.md:L526| unit=I-GROUNDING): ADD shipped-substrate ledger for SSB, Darwin, traps, cache, context, calibration, mutation, recovery, RRA, provenance, decoding, compiled stores, tool catalog, intent, gates, crates, and wiring tools. | anchor=IG-E007–IG-E026
- **I-001/SD-IG-003** [SD] (DEEP-I-001.discovery.md:L527| unit=I-GROUNDING): ADD 14 confirmed absence/build-status records for I-4/I-6/I-9/I-10/I-11/I-14/I-18/I-21/I-33/I-35/I-37/I-38/I-39 and related gaps. | anchor=IG-E027–IG-E040
- **I-001/SD-IG-004** [SD] (DEEP-I-001.discovery.md:L528| unit=I-GROUNDING): ADD binding design corrections for I-7/I-10/I-11/I-13/I-14 and the cache/tool-layout tradeoffs. | anchor=IG-E041–IG-E045, IG-E049–IG-E051
- **I-001/SD-IG-005** [SD] (DEEP-I-001.discovery.md:L529| unit=I-GROUNDING): ADD binding sequencing/status corrections for I-4/I-19/I-20/I-23, Track 17, checkpoint recovery, trust wiring, persona scope, and constrained decoding. | anchor=IG-E046–IG-E048, IG-E052–IG-E059
- **I-001/SD-IG-006** [SD] (DEEP-I-001.discovery.md:L530| unit=I-GROUNDING): ADD exact unverified citation/evidence inventory plus dated cache, rate-limit, extended-thinking, batch, nondeterminism, benchmark, and optimizer landmines. | anchor=IG-E060–IG-E065, IG-E073–IG-E074, IG-A001
- **I-001/SD-IG-007** [SD] (DEEP-I-001.discovery.md:L531| unit=I-GROUNDING): ADD exact Track-12–18 per-session substrate/production-anchor map. | anchor=IG-E066–IG-E072
- **I-001/SD-IG-008** [SD] (DEEP-I-001.discovery.md:L532| unit=I-GROUNDING): QUALIFY every grounding-map status as dated/necessary-not-sufficient and leave file-token reachability unresolved. | anchor=IG-I001, IG-A002

#### I-001 / unit `I-8` (36 records)

- **I-001/I8-E001** [SRC] (DEEP-I-001.discovery.md:L123| unit=I-8): E/STATUS | I-8 is a strong-grounded Track-13 BUILD, effort M, Wave 1, CANT-LIE lever, bound to RD-1. | anchor=I-8:L1-L3
- **I-001/I8-E002** [SRC] (DEEP-I-001.discovery.md:L124| unit=I-8): E/CAP | It adds acknowledge_infeasible / abort_trajectory to the production tool schema because the shipped abstention scorer lacks a structural invocation mechanism. | anchor=I-8:L5-L15
- **I-001/I8-E003** [SRC] (DEEP-I-001.discovery.md:L125| unit=I-8): E/FLOW | The tool is registered through the existing three-arm catalog pattern and runs through normal production tool conversion/dispatch. | anchor=I-8:L13-L15, L28-L29, L58-L63, L68-L83
- **I-001/I8-E004** [SRC] (DEEP-I-001.discovery.md:L126| unit=I-8): E/EVENT | An abort calls the existing H-39 scorer and emits AbstentionScored with abort_reason Voluntary, DeadEndHash, or HaltingBound; I-8 never reimplements reward/credit logic. | anchor=I-8:L13-L16, L49-L51, L72-L77, L85-L87
- **I-001/I8-E005** [SRC] (DEEP-I-001.discovery.md:L127| unit=I-8): E/RECOVERY | Voluntary abort rolls the trajectory back to the last good H-49 trunk node and journals the restored node/sequence identity. | anchor=I-8:L16-L16, L37-L39, L62-L63, L100-L106
- **I-001/I8-E006** [SRC] (DEEP-I-001.discovery.md:L128| unit=I-8): E/DATA | An aborted AST or approach shape is fingerprinted as a blake3 hash. | anchor=I-8:L16-L18, L44-L46
- **I-001/I8-E007** [SRC] (DEEP-I-001.discovery.md:L129| unit=I-8): E/DATA | DeadEndCache is a bounded LRU in crates/orchestrator/src/dead_end_cache.rs with check_and_insert; v1 persistence is session-local. | anchor=I-8:L17-L18, L45-L46, L54-L55, L59-L59, L68-L70
- **I-001/I8-E008** [SRC] (DEEP-I-001.discovery.md:L130| unit=I-8): E/FLOW | Every branch expansion checks the dead-end cache before a model call; an isomorphic match is a final prune costing zero model calls/tokens. | anchor=I-8:L18-L19, L24-L25, L45-L46, L88-L89
- **I-001/I8-E009** [SRC] (DEEP-I-001.discovery.md:L131| unit=I-8): E/FAIL | A deterministic check running past the empirical 15-second default triggers automatic abort and a HaltingBound row through the I-17 runner seam. | anchor=I-8:L19-L21, L47-L48, L61-L63, L77-L83
- **I-001/I8-E010** [SRC] (DEEP-I-001.discovery.md:L132| unit=I-8): E/CFG | The 15-second bound must be a named configurable key whose default is sourced from I-17's specification; it cannot be an unnamed hard-coded constant. | anchor=I-8:L19-L21, L90-L91
- **I-001/I8-E011** [SRC] (DEEP-I-001.discovery.md:L133| unit=I-8): E/EVENT | I-8 registers TrajectoryAborted with kinds Voluntary, DeadEndHash, and HaltingBound in ssb.rs/action_type_lock.rs as a live-baseline-relative +1. | anchor=I-8:L40-L40, L49-L49, L68-L71, L119-L120
- **I-001/I8-E012** [SRC] (DEEP-I-001.discovery.md:L134| unit=I-8): E/DEP | D-16/D-17 supply refusal/SLA substrate, H-39 supplies scoring, H-49 supplies rollback, IS-14 supplies tool registration, and I-17 supplies the watchdog seam. | anchor=I-8:L27-L39, L58-L63, L109-L130
- **I-001/I8-E013** [SRC] (DEEP-I-001.discovery.md:L135| unit=I-8): E/STATE | Wave 1 ships the tool/cache before H-49 and I-17 are necessarily live; rollback and watchdog integrations become fully exercised when those later seams land. | anchor=I-8:L35-L39, L129-L130
- **I-001/I8-E014** [SRC] (DEEP-I-001.discovery.md:L136| unit=I-8): E/DEFER | Durable cross-session dead-end persistence is deferred until at least three sessions show repeated isomorphic retries. | anchor=I-8:L53-L55
- **I-001/I8-E015** [SRC] (DEEP-I-001.discovery.md:L137| unit=I-8 SURFACE): E/DEFER | The ReasoningDrawer abort/dead-end strip is deferred/wire-dark behind a GD-4 bound and is W-6-justified for v1; when shipped it must pass cross_lang_diff.py. | anchor=I-8:L53-L56, L64-L65, L94-L96
- **I-001/I8-E016** [SRC] (DEEP-I-001.discovery.md:L138| unit=I-8): E/LIMIT | A learned infeasibility classifier is excluded because no ground-truth corpus exists; the structural tool is first. | anchor=I-8:L21-L22
- **I-001/I8-E017** [SRC] (DEEP-I-001.discovery.md:L139| unit=I-8): E/LIMIT | A whole-task hard timeout is excluded because context budget already governs tasks; I-8 is trajectory-local. | anchor=I-8:L22-L23
- **I-001/I8-E018** [SRC] (DEEP-I-001.discovery.md:L140| unit=I-8): E/LIMIT | Resurrecting or overriding an aborted/isomorphic path in the same session is explicitly declined. | anchor=I-8:L23-L25, L88-L89
- **I-001/I8-E019** [SRC] (DEEP-I-001.discovery.md:L141| unit=I-8): E/DET | Dead-end hashing must use blake3, not blake2/sha2; production paths prohibit unwrap/expect and fabricated measurements. | anchor=I-8:L92-L93
- **I-001/I8-E020** [SRC] (DEEP-I-001.discovery.md:L142| unit=I-8): E/DATA | The required decision artifact is workflow/reasoning/I-8-decisions.md. | anchor=I-8:L52-L52
- **I-001/I8-E021** [SRC] (DEEP-I-001.discovery.md:L143| unit=I-8): E/TEST | The live acceptance trigger runs impossible_task and spin_task with a 15-second halting bound. | anchor=I-8:L97-L100
- **I-001/I8-E022** [SRC] (DEEP-I-001.discovery.md:L144| unit=I-8): E/OBS | impossible_task must emit TrajectoryAborted(Voluntary), AbstentionScored(Voluntary), and a journal-verifiable trunk rollback. | anchor=I-8:L100-L102, L105-L106
- **I-001/I8-E023** [SRC] (DEEP-I-001.discovery.md:L145| unit=I-8): E/OBS | spin_task must emit TrajectoryAborted(HaltingBound) within budget. | anchor=I-8:L99-L104
- **I-001/I8-E024** [SRC] (DEEP-I-001.discovery.md:L146| unit=I-8): E/TEST | An isomorphic second attempt must emit DeadEndHash with journal token/model-call count exactly zero. | anchor=I-8:L102-L104
- **I-001/I8-E025** [SRC] (DEEP-I-001.discovery.md:L147| unit=I-8): E/TEST | The broken no-cache arm must exceed three isomorphic iterations and fail the spiral detector. | anchor=I-8:L105-L107
- **I-001/I8-E026** [SRC] (DEEP-I-001.discovery.md:L148| unit=I-8): E/INT | The production chains are model tool call -> catalog dispatch -> abort handler -> cache plus score, and check-runner timeout -> abort handler -> HaltingBound; test-only callers do not satisfy reachability. | anchor=I-8:L78-L84
- **I-001/I8-E027** [SRC] (DEEP-I-001.discovery.md:L149| unit=I-8): E/LIMIT | D-16/D-17 refusal logic, H-39 scoring, H-49 rollback logic, and the E-35 catalog are composed and must not be duplicated. | anchor=I-8:L85-L87, L122-L130
- **I-001/I8-A001** [AC] (DEEP-I-001.discovery.md:L150| unit=I-8): A/DATA | The source alternates between AST shape, approach fingerprint, and approach hash without defining the canonicalization algorithm whose equality decides isomorphism. | anchor=I-8:L8-L9, L16-L18, L45-L46
- **I-001/I8-A002** [AC] (DEEP-I-001.discovery.md:L151| unit=I-8): A/CAP | acknowledge_infeasible and abort_trajectory are written as a slash pair, but the source does not resolve whether they are one catalog name, aliases, or tool-plus-handler names. | anchor=I-8:L7-L8, L13-L14, L44-L44
- **I-001/I8-A003** [AC] (DEEP-I-001.discovery.md:L152| unit=I-8): A/CFG | The requirement says the bound must have a named key but does not name that configuration key. | anchor=I-8:L90-L91
- **I-001/I8-C001** [CON] (DEEP-I-001.discovery.md:L153| unit=I-8 SURFACE): C/SURFACE | The orientation summary presents the ReasoningDrawer strip as an unqualified feature; the numbered source explicitly defers it behind a GD-4 bound and justifies no v1 frontend. | anchor=I-8:L53-L56, L64-L65, L94-L96; orientation:L275-L275
- **I-001/SD-I8-001** [SD] (DEEP-I-001.discovery.md:L538| unit=I-8): ADD bounded in-session LRU semantics, pre-call final prune, blake3-only rule, and cross-session persistence gate. | anchor=I8-E006–I8-E008, I8-E014, I8-E019
- **I-001/SD-I8-002** [SD] (DEEP-I-001.discovery.md:L539| unit=I-8): ADD exact tool/score/rollback/watchdog production chains and staged Wave-1 versus H-49/I-17 activation. | anchor=I8-E002–I8-E005, I8-E009–I8-E013, I8-E026–I8-E027
- **I-001/SD-I8-003** [SD] (DEEP-I-001.discovery.md:L540| unit=I-8): ADD learned-classifier/global-timeout exclusions, same-session resurrection ban, named-config requirement, decision artifact, and complete broken-arm evidence. | anchor=I8-E016–I8-E025
- **I-001/SD-I8-004** [SD] (DEEP-I-001.discovery.md:L541| unit=I-8 SURFACE): CORRECT the summary's unconditional ReasoningDrawer surface: it is deferred/wire-dark behind GD-4 and W-6-justified for v1. | anchor=I8-E015, I8-C001
- **I-001/SD-I8-005** [SD] (DEEP-I-001.discovery.md:L542| unit=I-8 SURFACE): FLAG undefined approach canonicalization, tool alias/handler naming, and configuration-key identity. | anchor=I8-A001–I8-A003

#### I-001 / unit `I-13` (32 records)

- **I-001/I13-E001** [SRC] (DEEP-I-001.discovery.md:L163| unit=I-13): E/STATUS | I-13 is a strong-grounded Track-14 BUILD, effort M, Wave 2, SECURITY-EFFICIENCY lever, bound to IC-10. | anchor=I-13:L1-L3
- **I-001/I13-E002** [SRC] (DEEP-I-001.discovery.md:L164| unit=I-13): E/CAP | It ships three orthogonal mechanisms together: deterministic memory-write guard, dual tool-result serialization, and TraceQuery expansion. | anchor=I-13:L4-L8
- **I-001/I13-E003** [SRC] (DEEP-I-001.discovery.md:L165| unit=I-13): E/SEC | Every Recall, Archival, and playbook memory write is scanned at write time for injection markers, invisible-Unicode/multibyte tricks, and secret patterns. | anchor=I-13:L11-L19, L30-L36
- **I-001/I13-E004** [SRC] (DEEP-I-001.discovery.md:L166| unit=I-13): E/SEC | Tier 1 uses regex plus Aho-Corasick first. | anchor=I-13:L13-L15, L31-L32
- **I-001/I13-E005** [SRC] (DEEP-I-001.discovery.md:L167| unit=I-13): E/DEP | A YARA engine is a new dependency requiring package anti-slopsquat verification and explicit user approval; it may be added only after measurement shows the first-line scanner insufficient. | anchor=I-13:L14-L15, L28-L28, L38-L40, L66-L68
- **I-001/I13-E006** [SRC] (DEEP-I-001.discovery.md:L168| unit=I-13): E/INT | The scanner composes H-31/H-32 memory objectivity, E-37 PromptArmor, MINJA, G-9 redaction precedent, H-6 masking, and safety Provenance; it must not re-filter their scopes. | anchor=I-13:L10-L20, L22-L28, L55-L60, L81-L94
- **I-001/I13-E007** [SRC] (DEEP-I-001.discovery.md:L169| unit=I-13): E/FLOW | MemoryWriteScanner is inserted at mem0_dispatcher.rs/store.rs write choke points and registers as a write-side provenance hook. | anchor=I-13:L31-L33, L42-L46, L49-L64
- **I-001/I13-E008** [SRC] (DEEP-I-001.discovery.md:L170| unit=I-13): E/FAIL | A detected poison write is refused, never silently swallowed, and produces no store entry. | anchor=I-13:L66-L71, L76-L79
- **I-001/I13-E009** [SRC] (DEEP-I-001.discovery.md:L171| unit=I-13): E/EVENT | MemoryGuardEvent(PoisonBlocked) records the named pattern class. | anchor=I-13:L31-L35, L49-L54, L76-L79
- **I-001/I13-E010** [SRC] (DEEP-I-001.discovery.md:L172| unit=I-13 SURFACE): E/DATA | ToolResult gains to_ui_dict for the full dashboard payload and to_llm_content for condensed model context. | anchor=I-13:L16-L18, L32-L34, L42-L45, L49-L54
- **I-001/I13-E011** [SRC] (DEEP-I-001.discovery.md:L173| unit=I-13 SURFACE): E/FLOW | Every production tool result entering context must pass through to_llm_content; adapters are updated while the full UI form remains intact. | anchor=I-13:L32-L34, L43-L45, L61-L70
- **I-001/I13-E012** [SRC] (DEEP-I-001.discovery.md:L174| unit=I-13 SURFACE): E/OBS | DualSerialized records the token delta between dashboard and context representations in the journal. | anchor=I-13:L32-L35, L47-L47, L76-L79
- **I-001/I13-E013** [SRC] (DEEP-I-001.discovery.md:L175| unit=I-13): E/CAP | TraceQuery is a production tool that expands an exact on-disk line range on demand when condensed content is insufficient. | anchor=I-13:L16-L18, L33-L35, L42-L45
- **I-001/I13-E014** [SRC] (DEEP-I-001.discovery.md:L176| unit=I-13): E/INT | TraceQuery is implemented in crates/tools/src/trace_query.rs and registered through build_tool_catalog's three-arm pattern. | anchor=I-13:L34-L35, L45-L45, L49-L64
- **I-001/I13-E015** [SRC] (DEEP-I-001.discovery.md:L177| unit=I-13): E/EVENT | TraceQuery expansion emits MemoryGuardEvent(TraceQueryExpanded) with the requested byte/range evidence. | anchor=I-13:L34-L35, L54-L54, L76-L79
- **I-001/I13-E016** [SRC] (DEEP-I-001.discovery.md:L178| unit=I-13): E/EVENT | MemoryGuardEvent has kinds PoisonBlocked, DualSerialized, and TraceQueryExpanded and is a live-baseline-relative +1 SSB addition. | anchor=I-13:L35-L35, L49-L54, L86-L86
- **I-001/I13-E017** [SRC] (DEEP-I-001.discovery.md:L179| unit=I-13): E/CFG | The write guard cannot become default-on until a clean corpus across at least three real runs establishes a GD-4 false-positive bound. | anchor=I-13:L19-L19, L35-L36, L66-L71, L81-L86
- **I-001/I13-E018** [SRC] (DEEP-I-001.discovery.md:L180| unit=I-13): E/DEFER | Automated secret-pattern updates from a versioned threat feed are deferred for governance. | anchor=I-13:L38-L40
- **I-001/I13-E019** [SRC] (DEEP-I-001.discovery.md:L181| unit=I-13 SURFACE): E/SURFACE | MemoryFactsDrawer must show guard pattern class and serialization token delta; it is a required user-visible security/efficiency surface. | anchor=I-13:L47-L47, L74-L75
- **I-001/I13-E020** [SRC] (DEEP-I-001.discovery.md:L182| unit=I-13): E/FLOW | Required production reachability is memory write -> scan, ToolResult assembly -> to_llm_content, and model tool call -> TraceQuery.expand; test-only callers do not qualify. | anchor=I-13:L61-L65
- **I-001/I13-E021** [SRC] (DEEP-I-001.discovery.md:L183| unit=I-13): E/TEST | A seeded invisible-Unicode fixture must emit PoisonBlocked and leave no memory-store entry. | anchor=I-13:L76-L79
- **I-001/I13-E022** [SRC] (DEEP-I-001.discovery.md:L184| unit=I-13 SURFACE): E/TEST | A 500-file listing must remain complete in the dashboard while the model receives a condensed form and the journal records the token delta. | anchor=I-13:L76-L79
- **I-001/I13-E023** [SRC] (DEEP-I-001.discovery.md:L185| unit=I-13): E/TEST | A TraceQuery call must emit TraceQueryExpanded for the exact requested range. | anchor=I-13:L76-L79
- **I-001/I13-E024** [SRC] (DEEP-I-001.discovery.md:L186| unit=I-13): E/TEST | A --no-guard negative arm accepts the malicious write and must fail the fixture check. | anchor=I-13:L79-L79
- **I-001/I13-E025** [SRC] (DEEP-I-001.discovery.md:L187| unit=I-13): E/DET | Production scan/serialization paths prohibit unwrap/expect; hash use is blake3-only. | anchor=I-13:L66-L73
- **I-001/I13-A001** [AC] (DEEP-I-001.discovery.md:L188| unit=I-13): A/CFG | The source requires a quantified false-positive bound but provides neither threshold nor exact clean corpus beyond “at least three real runs.” | anchor=I-13:L19-L19, L35-L36, L71-L71
- **I-001/I13-A002** [AC] (DEEP-I-001.discovery.md:L189| unit=I-13): A/DATA | TraceQuery is described in line ranges but the event example uses byte_range; encoding, authorization, and path-boundary semantics are not specified. | anchor=I-13:L17-L18, L34-L35, L78-L78
- **I-001/I13-A003** [AC] (DEEP-I-001.discovery.md:L190| unit=I-13): A/SEC | “Secret patterns” and “injection markers” are required classes, but the source supplies no concrete pattern catalog/versioning scheme for Tier 1. | anchor=I-13:L13-L15, L31-L32
- **I-001/SD-I13-001** [SD] (DEEP-I-001.discovery.md:L548| unit=I-13): ADD exact composition boundary with memory/safety systems, write-side Provenance registration, hard refusal/no-store semantics, and no-refilter rule. | anchor=I13-E003–I13-E009
- **I-001/SD-I13-002** [SD] (DEEP-I-001.discovery.md:L549| unit=I-13): ADD exact dual-serialization and TraceQuery production chains plus the three MemoryGuardEvent discriminators. | anchor=I13-E010–I13-E016, I13-E020
- **I-001/SD-I13-003** [SD] (DEEP-I-001.discovery.md:L550| unit=I-13): ADD YARA's “only if measured first-line scanner insufficient” gate and deferred governed threat-feed updates. | anchor=I13-E005, I13-E017–I13-E018
- **I-001/SD-I13-004** [SD] (DEEP-I-001.discovery.md:L551| unit=I-13 SURFACE): ADD negative-arm and full-vs-condensed 500-file artifact proof; FLAG unspecified FP threshold, path authorization, and pattern catalog. | anchor=I13-E021–I13-E025, I13-A001–I13-A003

#### I-001 / unit `I-15` (28 records)

- **I-001/I15-E001** [SRC] (DEEP-I-001.discovery.md:L200| unit=I-15): E/STATUS | I-15 is a strong-grounded Track-14 EXTENSION, effort S, Wave 1, CHEAPER-SPEED lever. | anchor=I-15:L1-L3
- **I-001/I15-E002** [SRC] (DEEP-I-001.discovery.md:L201| unit=I-15): E/CAP | It extends the existing C-30/H-8 OracleAnswerCache to cache intermediate DKG chunk summaries and research extracts, not only final answers. | anchor=I-15:L4-L13, L24-L29
- **I-001/I15-E003** [SRC] (DEEP-I-001.discovery.md:L202| unit=I-15): E/FLOW | Semantically similar follow-up queries may reuse intermediate summaries and skip re-summarization. | anchor=I-15:L11-L15
- **I-001/I15-E004** [SRC] (DEEP-I-001.discovery.md:L203| unit=I-15): E/PERF | An O(1) lookup keyed by blake3(canonical_query_bytes) must execute before vector cosine work or any ONNX embedding call. | anchor=I-15:L7-L8, L14-L15, L31-L36, L49-L60
- **I-001/I15-E005** [SRC] (DEEP-I-001.discovery.md:L204| unit=I-15): E/FALLBACK | An exact-key miss proceeds to the normal semantic-vector/reranker path. | anchor=I-15:L14-L15, L46-L47
- **I-001/I15-E006** [SRC] (DEEP-I-001.discovery.md:L205| unit=I-15): E/SEC | Durable cache confinement is limited to idempotent grounding content; code-generation output, context-dependent tool results, and session-specific reasoning chains must never enter it. | anchor=I-15:L16-L18, L62-L65
- **I-001/I15-E007** [SRC] (DEEP-I-001.discovery.md:L206| unit=I-15): E/DEP | The existing H-8 store, TTL logic, and confinement test are inherited and extended, never reimplemented or replaced. | anchor=I-15:L10-L11, L24-L29, L43-L46, L54-L57, L77-L89
- **I-001/I15-E008** [SRC] (DEEP-I-001.discovery.md:L207| unit=I-15): E/CFG | Intermediate summaries default to H-8's volatile five-minute tier, but I-10 CacheEconomicsPlanner may override per call class; I-15 must read the plan rather than hard-code TTL. | anchor=I-15:L31-L35, L54-L57, L84-L88
- **I-001/I15-E009** [SRC] (DEEP-I-001.discovery.md:L208| unit=I-15): E/EVENT | Preferred observability is reuse of OracleCacheHit/Miss with a semantic_cache discriminator and sources blake3_exact or semantic_vector. | anchor=I-15:L19-L22, L31-L36, L49-L53
- **I-001/I15-E010** [SRC] (DEEP-I-001.discovery.md:L209| unit=I-15): E/EVENT | Only if the existing payload cannot carry that discriminator may SemanticCacheEvent be added; the reuse-versus-new-event decision is deferred to edit time and must be recorded. | anchor=I-15:L19-L22, L49-L53, L77-L82
- **I-001/I15-E011** [SRC] (DEEP-I-001.discovery.md:L210| unit=I-15): E/INT | The production insertion point is knowledge/src/arag.rs DKG/research chunk summarization; the exact-hit path bypasses the Phase-1G reranker/ONNX call. | anchor=I-15:L24-L27, L31-L37, L43-L47, L58-L61
- **I-001/I15-E012** [SRC] (DEEP-I-001.discovery.md:L211| unit=I-15 SURFACE): E/SURFACE | Research answers show a required cache-source badge with blake3_exact, semantic_vector, or miss. | anchor=I-15:L37-L37, L43-L47, L70-L71
- **I-001/I15-E013** [SRC] (DEEP-I-001.discovery.md:L212| unit=I-15): E/DEFER | Cross-session prewarming is deferred until H-51 cross-session topology is live. | anchor=I-15:L39-L40
- **I-001/I15-E014** [SRC] (DEEP-I-001.discovery.md:L213| unit=I-15 SURFACE): E/DEFER | Similarity-threshold auto-tuning is deferred until ELAI measures its own research-corpus precision/recall baseline. | anchor=I-15:L40-L41
- **I-001/I15-E015** [SRC] (DEEP-I-001.discovery.md:L214| unit=I-15): E/LIMIT | The research dossier's 50–60% token-reduction figure remains design-only; I-15 measures its own two-run input-token delta and cannot advertise a hard-coded similarity target. | anchor=I-15:L39-L41, L62-L65, L72-L75, L77-L82
- **I-001/I15-E016** [SRC] (DEEP-I-001.discovery.md:L215| unit=I-15): E/TEST | Running the same research query twice must make the second run an exact blake3 hit with trace-verified ONNX call count zero. | anchor=I-15:L72-L75
- **I-001/I15-E017** [SRC] (DEEP-I-001.discovery.md:L216| unit=I-15): E/OBS | The first/second runs journal cache source, latency/call count, and measured RAG input-token delta. | anchor=I-15:L72-L75
- **I-001/I15-E018** [SRC] (DEEP-I-001.discovery.md:L217| unit=I-15): E/TEST | A code-generation query must create no durable entry; the inherited H-8 confinement test and a broken caching arm must enforce this. | anchor=I-15:L63-L65, L73-L75
- **I-001/I15-E019** [SRC] (DEEP-I-001.discovery.md:L218| unit=I-15): E/DET | Hashing uses blake3, not blake2/sha2, and production lookup/insert paths prohibit unwrap/expect. | anchor=I-15:L67-L69
- **I-001/I15-E020** [SRC] (DEEP-I-001.discovery.md:L219| unit=I-15): E/INT | Exact production chains are A-RAG summarize -> exact lookup and DKG summarize -> intermediate insert; tests alone do not satisfy R7. | anchor=I-15:L58-L61
- **I-001/I15-A001** [AC] (DEEP-I-001.discovery.md:L220| unit=I-15): A/EVENT | The final SSB schema is intentionally unresolved between extending OracleCacheHit/Miss and adding SemanticCacheEvent. | anchor=I-15:L19-L22, L49-L53, L82-L82
- **I-001/I15-A002** [AC] (DEEP-I-001.discovery.md:L221| unit=I-15): A/DATA | canonical_query_bytes is the exact-key input, but canonicalization rules are not defined in the source. | anchor=I-15:L14-L15
- **I-001/I15-A003** [AC] (DEEP-I-001.discovery.md:L222| unit=I-15): A/CFG | The semantic-vector similarity threshold and its eventual tuning method remain deliberately unspecified pending local measurement. | anchor=I-15:L40-L41, L65-L65
- **I-001/I15-C001** [CON] (DEEP-I-001.discovery.md:L223| unit=I-15): C/EVENT | The orientation summary says I-15 reuses OracleCacheHit/Miss; the numbered source makes reuse preferred but conditionally permits a new SemanticCacheEvent if payload compatibility fails. | anchor=I-15:L19-L22, L49-L53, L82-L82; orientation:L449-L449
- **I-001/SD-I15-001** [SD] (DEEP-I-001.discovery.md:L557| unit=I-15): ADD exact canonical-query fast-path ordering, semantic miss fallback, production A-RAG/DKG seams, and broader exclusion of tool/reasoning content. | anchor=I15-E003–I15-E007, I15-E011
- **I-001/SD-I15-002** [SD] (DEEP-I-001.discovery.md:L558| unit=I-15): ADD volatile five-minute default under I-10 override and defer cross-session warming plus threshold tuning. | anchor=I15-E008, I15-E013–I15-E015
- **I-001/SD-I15-003** [SD] (DEEP-I-001.discovery.md:L559| unit=I-15): QUALIFY summary event reuse: OracleCacheHit/Miss is preferred, but SemanticCacheEvent remains a conditional edit-time choice. | anchor=I15-E009–I15-E010, I15-A001, I15-C001
- **I-001/SD-I15-004** [SD] (DEEP-I-001.discovery.md:L560| unit=I-15 SURFACE): ADD exact two-run/broken-arm evidence and FLAG undefined canonicalization/vector threshold. | anchor=I15-E016–I15-E020, I15-A002–I15-A003

#### I-001 / unit `I-18` (36 records)

- **I-001/I18-E001** [SRC] (DEEP-I-001.discovery.md:L233| unit=I-18): E/STATUS | I-18 is a strong-grounded Track-15 EXTENSION, effort M, Wave 3, AUDITABLE lever, bound to IC-11; full-form expansion is required before execution. | anchor=I-18:L1-L3, L105-L105
- **I-001/I18-E002** [SRC] (DEEP-I-001.discovery.md:L234| unit=I-18): E/STATE | G-2 RRA build/verify and G-8 attestation already ship; the absent capability is sequence-wide row tamper evidence. | anchor=I-18:L4-L9, L28-L34
- **I-001/I18-E003** [SRC] (DEEP-I-001.discovery.md:L235| unit=I-18): E/DATA | Every SSB insert adds chain_prev_hash equal to blake3(prior_row_bytes). | anchor=I-18:L11-L14, L36-L39, L49-L55
- **I-001/I18-E004** [SRC] (DEEP-I-001.discovery.md:L236| unit=I-18): E/CONC | The SSB single writer owns the rolling hash, and no async/batch path may reorder inserts or bypass previous-hash order. | anchor=I-18:L12-L14, L24-L26, L50-L51, L74-L79
- **I-001/I18-E005** [SRC] (DEEP-I-001.discovery.md:L237| unit=I-18): E/SCHEMA | chain_prev_hash extends the append-only/golden-locked row; the executor must coordinate all six lockstep bump sites. | anchor=I-18:L12-L14, L29-L31, L49-L51, L89-L93
- **I-001/I18-E006** [SRC] (DEEP-I-001.discovery.md:L238| unit=I-18): E/DATA | RRA StackHash has five blake3 layers: STATE=SSB snapshot, DEPS=Cargo.lock, PROCESS=execution environment, RESULT=output artifact, VERIFY=verifier-stack outcome. | anchor=I-18:L15-L18, L36-L40, L59-L64
- **I-001/I18-E007** [SRC] (DEEP-I-001.discovery.md:L239| unit=I-18): E/FAIL | A Cargo.lock mutation during a run invalidates DEPS immediately and hard-aborts with RowChainSealed(ChainBroken). | anchor=I-18:L17-L18, L39-L40, L84-L87
- **I-001/I18-E008** [SRC] (DEEP-I-001.discovery.md:L240| unit=I-18): E/ARTIFACT | build_rra seals the five-layer hash into the existing RRA and the verifier gains a chain-verification leg. | anchor=I-18:L15-L18, L39-L39, L49-L54, L59-L68
- **I-001/I18-E009** [SRC] (DEEP-I-001.discovery.md:L241| unit=I-18): E/ARTIFACT | Every sealed attestation gains an adjacent AI-BOM/CycloneDX-ML component/file expressing the cited OWASP AISVS AC.8 expectation. | anchor=I-18:L22-L23, L36-L43, L52-L55, L89-L96
- **I-001/I18-E010** [SRC] (DEEP-I-001.discovery.md:L242| unit=I-18): E/TEST | The AI-BOM must validate against the published CycloneDX schema whose URL/version is web-confirmed at execution. | anchor=I-18:L22-L23, L84-L87, L93-L95
- **I-001/I18-E011** [SRC] (DEEP-I-001.discovery.md:L243| unit=I-18): E/CFG | Offline blake3-Merkle verification is the complete default; external Rekor transparency is opt-in. | anchor=I-18:L19-L21, L44-L47
- **I-001/I18-E012** [SRC] (DEEP-I-001.discovery.md:L244| unit=I-18): E/DEP | Rekor uses cosign keyless via a binary or cosign-rs, but exact dependency identity/version/repository/license must pass anti-slopsquat checks and user approval before landing. | anchor=I-18:L19-L21, L34-L34, L44-L47, L89-L95
- **I-001/I18-E013** [SRC] (DEEP-I-001.discovery.md:L245| unit=I-18): E/DEFER | The --rekor interface/checkbox may be scaffolded now, but the actual push is a separately gated Tier-2 capability. | anchor=I-18:L44-L47
- **I-001/I18-E014** [SRC] (DEEP-I-001.discovery.md:L246| unit=I-18): E/FAIL | If --rekor is passed and the push fails, the run halts with a clear error; silent SKIP is forbidden. | anchor=I-18:L74-L79
- **I-001/I18-E015** [SRC] (DEEP-I-001.discovery.md:L247| unit=I-18 SURFACE): E/CLI | elai verify-attestation --chain walks the chain and reports the first broken link and exact sequence number. | anchor=I-18:L41-L43, L49-L55, L70-L73
- **I-001/I18-E016** [SRC] (DEEP-I-001.discovery.md:L248| unit=I-18): E/EVENT | RowChainSealed kinds are ChainAdvanced, RekorPushed, AibomEmitted, and ChainBroken, registered as a live-baseline-relative +1 action type. | anchor=I-18:L41-L43, L54-L55, L59-L64, L97-L97
- **I-001/I18-E017** [SRC] (DEEP-I-001.discovery.md:L249| unit=I-18): E/IDEM | Once ChainAdvanced seals a sequence, the row bytes are immutable; editing a sealed row is the primary banned design. | anchor=I-18:L24-L26, L74-L76
- **I-001/I18-E018** [SRC] (DEEP-I-001.discovery.md:L250| unit=I-18): E/SEC | Chain, stack, and AI-BOM integrity fields are blake3-only; sha256/blake2 are prohibited. | anchor=I-18:L24-L26, L74-L77
- **I-001/I18-E019** [SRC] (DEEP-I-001.discovery.md:L251| unit=I-18 SURFACE): E/INT | Changes target ssb.rs single-writer insert, rra.rs build/verify/CLI chain leg, attestation.rs AI-BOM emission, and the CLI verify subcommand; G-2/G-8/F-4 are composed, not redefined. | anchor=I-18:L49-L55, L59-L73, L99-L103
- **I-001/I18-E020** [SRC] (DEEP-I-001.discovery.md:L252| unit=I-18 SURFACE): E/SURFACE | The existing RRA viewer gains a chain badge, not a new drawer; it subscribes to RowChainSealed and shows CHAIN OK or BROKEN plus break seq. | anchor=I-18:L56-L57, L80-L83
- **I-001/I18-E021** [SRC] (DEEP-I-001.discovery.md:L253| unit=I-18 SURFACE): E/TEST | cross_lang_diff.py must pass for the RRA-viewer row/badge extension. | anchor=I-18:L56-L57, L80-L83
- **I-001/I18-E022** [SRC] (DEEP-I-001.discovery.md:L254| unit=I-18): E/TEST | A normal emit-RRA run must produce ChainAdvanced rows for every insert, AibomEmitted on seal, and verify-attestation --chain exit 0. | anchor=I-18:L84-L87
- **I-001/I18-E023** [SRC] (DEEP-I-001.discovery.md:L255| unit=I-18): E/TEST | Editing database row N must make chain verification fail at N and name the exact break. | anchor=I-18:L84-L87
- **I-001/I18-E024** [SRC] (DEEP-I-001.discovery.md:L256| unit=I-18): E/TEST | The opt-in Rekor arm must yield an inclusion proof verifiable without rerunning ELAI. | anchor=I-18:L84-L87
- **I-001/I18-E025** [SRC] (DEEP-I-001.discovery.md:L257| unit=I-18): E/LIMIT | Existing RRA format/build logic, attestation write path, SSB writer, and F-4 stdout_hash are composition boundaries and may not be forked/reimplemented. | anchor=I-18:L65-L69, L99-L103
- **I-001/I18-E026** [SRC] (DEEP-I-001.discovery.md:L258| unit=I-18): E/DET | Production paths prohibit unwrap/expect and fabricated chain-verification results. | anchor=I-18:L78-L79
- **I-001/I18-A001** [AC] (DEEP-I-001.discovery.md:L259| unit=I-18): A/DEP | The source leaves the Rekor implementation choice open between a cosign binary and cosign-rs. | anchor=I-18:L19-L21
- **I-001/I18-A002** [AC] (DEEP-I-001.discovery.md:L260| unit=I-18): A/SCHEMA | It calls chain_prev_hash “an extend, not a schema change” while also requiring golden-lock/schema coordination; the exact storage migration semantics are not resolved. | anchor=I-18:L12-L14, L29-L31, L49-L55
- **I-001/I18-A003** [AC] (DEEP-I-001.discovery.md:L261| unit=I-18): A/COMP | The CycloneDX-ML version/schema URL and exact Rekor endpoint are deliberately deferred to execution-time web verification. | anchor=I-18:L22-L23, L34-L34, L89-L95
- **I-001/I18-C001** [CON] (DEEP-I-001.discovery.md:L262| unit=I-18 SURFACE): C/DEFER | The orientation summary presents Rekor push and chain/Rekor CLI options together; the numbered source explicitly makes the push Tier 2 and only scaffolds --rekor now. | anchor=I-18:L44-L47; orientation:L529-L535
- **I-001/I18-C002** [CON] (DEEP-I-001.discovery.md:L263| unit=I-18): C/DEP | The orientation adds coordinated sealed-record layout with I-45, but this numbered source names G-2, G-8, SSB, and F-4 composition and contains no I-45 dependency. The summary-only coordination claim is not imported. | anchor=I-18:L65-L69, L99-L103; orientation:L542-L543
- **I-001/SD-I18-001** [SD] (DEEP-I-001.discovery.md:L566| unit=I-18): ADD exact previous-row-byte hash, single-writer/no-async invariant, append-lock coordination, layer inputs, and sealed immutability. | anchor=I18-E003–I18-E008, I18-E017–I18-E018
- **I-001/SD-I18-002** [SD] (DEEP-I-001.discovery.md:L567| unit=I-18 SURFACE): ADD exact RowChainSealed kinds, CLI first-break behavior, badge subscription/state, and complete negative row-edit/Cargo-lock tests. | anchor=I18-E015–I18-E024
- **I-001/SD-I18-003** [SD] (DEEP-I-001.discovery.md:L568| unit=I-18 SURFACE): QUALIFY Rekor as separately deferred push with scaffolded UI/flag, approval/anti-slopsquat dependency choice, and fail-closed requested behavior. | anchor=I18-E011–I18-E014, I18-A001, I18-C001
- **I-001/SD-I18-004** [SD] (DEEP-I-001.discovery.md:L569| unit=I-18 SURFACE): ADD execution-time CycloneDX/Rekor verification requirements and FLAG schema-extension wording ambiguity. | anchor=I18-E009–I18-E010, I18-A002–I18-A003
- **I-001/SD-I18-005** [SD] (DEEP-I-001.discovery.md:L570| unit=I-18 SURFACE): REMOVE/FLAG the orientation-only I-45 coordination claim, which the numbered source does not support. | anchor=I18-C002

#### I-001 / unit `I-23` (26 records)

- **I-001/I23-E001** [SRC] (DEEP-I-001.discovery.md:L273| unit=I-23): E/STATUS | I-23 is a strong-grounded Track-15 EXTENSION/UNIFICATION, effort S–M, Wave 1, SMARTER-ORCH lever, bound to IC-13. | anchor=I-23:L1-L3
- **I-001/I23-E002** [SRC] (DEEP-I-001.discovery.md:L274| unit=I-23): E/STATE | conformal_calibration.rs and calibration_bootstrap.rs with warm_start_tau already ship; the task is structural unification, not Brier or new mathematics. | anchor=I-23:L4-L9, L29-L41, L94-L101
- **I-001/I23-E003** [SRC] (DEEP-I-001.discovery.md:L275| unit=I-23): E/CAP | A thin CalibrationFacade exposes typed get_tau(CalibrationRequest)->TauResult, update_tau(ObservedOutcome), and warm_start(). | anchor=I-23:L11-L15, L43-L48, L64-L68
- **I-001/I23-E004** [SRC] (DEEP-I-001.discovery.md:L276| unit=I-23): E/INT | After migration the facade is the only import path; no consumer may import conformal_calibration.rs directly. | anchor=I-23:L12-L15, L22-L24, L64-L84
- **I-001/I23-E005** [SRC] (DEEP-I-001.discovery.md:L277| unit=I-23): E/TEST | A structural contract greps direct conformal-calibration imports and requires a zero result after migration. | anchor=I-23:L22-L24, L43-L48, L79-L84
- **I-001/I23-E006** [SRC] (DEEP-I-001.discovery.md:L278| unit=I-23): E/INT | D-16 refusal migrates only its import/call site; refusal logic and all existing observable effects remain unchanged. | anchor=I-23:L16-L18, L35-L38, L43-L47, L54-L62, L89-L92
- **I-001/I23-E007** [SRC] (DEEP-I-001.discovery.md:L279| unit=I-23): E/DEP | G-7 claim_score is a planned consumer, but its migration is deferred to and owned by G-7; I-23 does not block on it. | anchor=I-23:L18-L18, L38-L40, L50-L52, L69-L74, L94-L100
- **I-001/I23-E008** [SRC] (DEEP-I-001.discovery.md:L280| unit=I-23): E/INT | I-4 PUCT consumes facade uncertainty; when TauResult confidence-interval width exceeds a locally measured threshold it inserts a divergent exploration branch rather than repeating the current hypothesis. | anchor=I-23:L19-L28, L40-L40
- **I-001/I23-E009** [SRC] (DEEP-I-001.discovery.md:L281| unit=I-23 SURFACE): E/CFG | The uncertainty threshold is measured on ELAI fixtures, persisted in metrics/calibration_fork_threshold.toml, and cannot be a paper constant. | anchor=I-23:L19-L21, L25-L28, L31-L33, L43-L48, L64-L68, L79-L84
- **I-001/I23-E010** [SRC] (DEEP-I-001.discovery.md:L282| unit=I-23): E/STATE | The facade can ship in Wave 1; I-4 fork logic is scaffolded as a conditional and becomes live only when Wave-4 PUCT exists. | anchor=I-23:L40-L40, L54-L59
- **I-001/I23-E011** [SRC] (DEEP-I-001.discovery.md:L283| unit=I-23): E/EVENT | I-23 adds no SSB type and reuses RefusalDecision and CalibrationUpdate, carrying a shared calibration_id/tau identity. | anchor=I-23:L43-L49, L54-L60, L79-L84, L89-L101
- **I-001/I23-E012** [SRC] (DEEP-I-001.discovery.md:L284| unit=I-23): E/OBS | A single run triggering refusal and a search fork must join both reused rows on the same calibration_id, proving one tau source. | anchor=I-23:L89-L92
- **I-001/I23-E013** [SRC] (DEEP-I-001.discovery.md:L285| unit=I-23): E/TEST | A split-calibrations negative arm with independent refusal/search tau values must fail i23_unified_calibration_contract.rs. | anchor=I-23:L89-L92
- **I-001/I23-E014** [SRC] (DEEP-I-001.discovery.md:L286| unit=I-23): E/TEST | The D-16 suite is rerun verbatim as a regression guard. | anchor=I-23:L92-L92
- **I-001/I23-E015** [SRC] (DEEP-I-001.discovery.md:L287| unit=I-23 SURFACE): E/SURFACE | No new dashboard component is added; existing D-16 calibration/refusal tiles remain and I-4's existing branch board surfaces forks. | anchor=I-23:L60-L62, L85-L88
- **I-001/I23-E016** [SRC] (DEEP-I-001.discovery.md:L288| unit=I-23): E/DEFER | Multi-model tau divergence detection for cloud/local model families is deferred; a single tau ships first. | anchor=I-23:L50-L53
- **I-001/I23-E017** [SRC] (DEEP-I-001.discovery.md:L289| unit=I-23): E/LIMIT | The facade may not inline I-4 fork logic, absorb G-7 logic, change refusal logic, or reimplement H-39 scoring/calibration math. | anchor=I-23:L103-L107
- **I-001/I23-E018** [SRC] (DEEP-I-001.discovery.md:L290| unit=I-23): E/DET | Production paths prohibit unwrap/expect and fabricated tau/CI values. | anchor=I-23:L79-L84
- **I-001/I23-E019** [SRC] (DEEP-I-001.discovery.md:L291| unit=I-23): E/FLOW | Production reachability is the existing D-16 refusal path via the facade plus the I-4 PUCT expansion caller; contract-only callers do not satisfy R7. | anchor=I-23:L75-L78
- **I-001/I23-A001** [AC] (DEEP-I-001.discovery.md:L292| unit=I-23): A/CFG | The confidence-interval width metric, fixture corpus, and threshold-selection procedure are not specified beyond local measurement and a TOML artifact. | anchor=I-23:L19-L21, L25-L28, L48-L48
- **I-001/I23-A002** [AC] (DEEP-I-001.discovery.md:L293| unit=I-23): A/INT | The exact I-4 file/seam is PLAN-asserted as ladder/PUCT and must be re-grepped once I-4 exists. | anchor=I-23:L54-L59
- **I-001/I23-C001** [CON] (DEEP-I-001.discovery.md:L294| unit=I-23): C/STATUS | The orientation says I-23 “uses” the facade for G-7; the numbered source says G-7 is planned and explicitly defers that call-site migration to G-7. | anchor=I-23:L18-L18, L38-L40, L50-L52, L74-L74; orientation:L654-L654
- **I-001/SD-I23-001** [SD] (DEEP-I-001.discovery.md:L576| unit=I-23): ADD typed facade methods, zero-direct-import invariant/test, exact reused-event identity, and D-16 unchanged regression gate. | anchor=I23-E003–I23-E006, I23-E011–I23-E014
- **I-001/SD-I23-002** [SD] (DEEP-I-001.discovery.md:L577| unit=I-23): QUALIFY G-7 as a future-owned migration and I-4 fork as Wave-1 scaffolding activated only with Wave-4 PUCT. | anchor=I23-E007–I23-E010, I23-C001
- **I-001/SD-I23-003** [SD] (DEEP-I-001.discovery.md:L578| unit=I-23 SURFACE): ADD local threshold artifact, divergent-hypothesis rule, no-new-UI details, and deferred multi-model tau support. | anchor=I23-E008–I23-E010, I23-E015–I23-E016
- **I-001/SD-I23-004** [SD] (DEEP-I-001.discovery.md:L579| unit=I-23 SURFACE): FLAG undefined CI-width calibration procedure and exact future I-4 seam. | anchor=I23-A001–I23-A002

#### I-001 / unit `I-28` (51 records)

- **I-001/I28-E001** [SRC] (DEEP-I-001.discovery.md:L304| unit=I-28): E/STATUS | I-28 is a strong-grounded Track-16 BUILD, effort M–L, Wave 5, SELF-MOD lever, blocked on I-4; its refreshed verdict is ratified BUILD-WITH-SEAM and requires full-form expansion. | anchor=I-28:L1-L6, L17-L18, L69-L74
- **I-001/I28-E002** [SRC] (DEEP-I-001.discovery.md:L305| unit=I-28): E/STATE | I-4 SearchLadderOutcome supplies a winner and ranked siblings but v1 lacks per-node exclusion and per-branch chunk provenance. | anchor=I-28:L4-L4, L8-L11, L23-L25
- **I-001/I28-E003** [SRC] (DEEP-I-001.discovery.md:L306| unit=I-28): E/STATE | The live task_runner drive_task calls dispatch_search_ladder before maybe_run_escalation_waterfall, but currently discards SearchLadderOutcome. | anchor=I-28:L8-L11
- **I-001/I28-E004** [SRC] (DEEP-I-001.discovery.md:L307| unit=I-28): E/FLOW | dispatch_search_ladder must become an Option<SearchLadderOutcome> producer and pass the value directly to dispatch_dag_credit; credit must not reconstruct it by rereading SSB. | anchor=I-28:L10-L11, L27-L30
- **I-001/I28-E005** [SRC] (DEEP-I-001.discovery.md:L308| unit=I-28): E/FLOW | dispatch_dag_credit is inserted immediately after dispatch_search_ladder and before maybe_run_escalation_waterfall. | anchor=I-28:L19-L21, L27-L29, L47-L50
- **I-001/I28-E006** [SRC] (DEEP-I-001.discovery.md:L309| unit=I-28): E/CAP | Tier-1 Slice 1 adds DagCreditAssigner and deterministic branch-grain credit. | anchor=I-28:L17-L21, L34-L38
- **I-001/I28-E007** [SRC] (DEEP-I-001.discovery.md:L310| unit=I-28): E/DET | Slice-1 credit is winner score minus sibling score, clamped to the contract range; no LLM-authored credit score is permitted. | anchor=I-28:L52-L54, L110-L111
- **I-001/I28-E008** [SRC] (DEEP-I-001.discovery.md:L311| unit=I-28): E/EVENT | Slice 1 emits DagCreditAssigned rows with Kind=CreditWalk, one per ranked sibling. | anchor=I-28:L19-L21, L34-L38, L64-L67
- **I-001/I28-E009** [SRC] (DEEP-I-001.discovery.md:L312| unit=I-28): E/DATA | Each CreditWalk row carries winner score, sibling score, ranked position, and deterministic credit delta. | anchor=I-28:L64-L66
- **I-001/I28-E010** [SRC] (DEEP-I-001.discovery.md:L313| unit=I-28 SURFACE): E/CFG | The production path is default-off behind ELAI_DAG_CREDIT_ENABLE and also requires the search-ladder flag. | anchor=I-28:L19-L21, L32-L32, L47-L50
- **I-001/I28-E011** [SRC] (DEEP-I-001.discovery.md:L314| unit=I-28 SURFACE): E/OBS | With ELAI_SEARCH_LADDER_ENABLE=1 and ELAI_DAG_CREDIT_ENABLE=1 credit rows appear; with the credit flag unset there must be zero rows. | anchor=I-28:L47-L50, L76-L84, L110-L117
- **I-001/I28-E012** [SRC] (DEEP-I-001.discovery.md:L315| unit=I-28): E/DEP | New public Rust symbols require a real production caller or a wire-dark allowlist row; Slice 1 adds one with promotion_plan_id=I-28-promotion. | anchor=I-28:L15-L15, L19-L21, L96-L101, L103-L108
- **I-001/I28-E013** [SRC] (DEEP-I-001.discovery.md:L316| unit=I-28 SURFACE): E/TEST | Slice 1 adds crates/contracts/tests/i28_dag_credit_contract.rs covering CreditWalk, schema floor, sql_str round-trip, flag-on, and flag-unset paths. | anchor=I-28:L19-L21, L76-L84, L103-L108, L117-L117
- **I-001/I28-E014** [SRC] (DEEP-I-001.discovery.md:L317| unit=I-28): E/SCHEMA | DagCreditAssigned is appended after ScopeCeilingAdvisory using the existing flat action-type plus payload-kind pattern. | anchor=I-28:L27-L31
- **I-001/I28-E015** [SRC] (DEEP-I-001.discovery.md:L318| unit=I-28): E/SCHEMA | At refresh time the registry had 606 entries and SCHEMA_VERSION_DDL_EPOCH=-309, deriving schema 297; implementation must recount and derive rather than hard-code a future value. | anchor=I-28:L30-L32, L69-L73
- **I-001/I28-E016** [SRC] (DEEP-I-001.discovery.md:L319| unit=I-28): E/STATE | I-17 verdicts and frozen VerdictRow are read-only; a HardVeto branch cannot be assigned or invented an L2 score, and VETO_SCORE/ID-4 may not change. | anchor=I-28:L12-L12, L40-L44, L52-L59, L84-L86, L96-L101
- **I-001/I28-E017** [SRC] (DEEP-I-001.discovery.md:L320| unit=I-28): E/CAP | Tier-1 Slice 2, only after FORK-1, extends SearchLadderOutcome with pruned verdict data and extracts DPO pairs. | anchor=I-28:L19-L22, L34-L38, L64-L67
- **I-001/I28-E018** [SRC] (DEEP-I-001.discovery.md:L321| unit=I-28): E/EVENT | Slice 2 emits DpoPairExtracted with verifiable provenance from a sealed winner and selected pruned sibling; an unsealed RRA yields zero pairs. | anchor=I-28:L21-L22, L52-L56, L64-L67, L76-L86, L110-L113
- **I-001/I28-E019** [SRC] (DEEP-I-001.discovery.md:L322| unit=I-28): E/DEP | A DPO JSONL writer must ship with I-30's first reader or remain explicitly wire-dark with promotion_plan_id=I-30. | anchor=I-28:L14-L15, L21-L22, L40-L45, L90-L94
- **I-001/I28-E020** [SRC] (DEEP-I-001.discovery.md:L323| unit=I-28): E/DEFER | Slice 3 chunk demotion is prohibited until each branch has chunk_id or equivalent provenance and the owning layer is decided. | anchor=I-28:L23-L25, L52-L57, L64-L67, L82-L88, L94-L101
- **I-001/I28-E021** [SRC] (DEEP-I-001.discovery.md:L324| unit=I-28): E/DEFER | Node-grain credit waits for v3 PUCT plus provenance capture. | anchor=I-28:L23-L25, L90-L93
- **I-001/I28-E022** [SRC] (DEEP-I-001.discovery.md:L325| unit=I-28): E/LIMIT | Shapley/SHARP attribution remains excluded and requires a separate RL-infrastructure decision. | anchor=I-28:L23-L25, L52-L57, L96-L99
- **I-001/I28-E023** [SRC] (DEEP-I-001.discovery.md:L326| unit=I-28 SURFACE): E/SURFACE | No dashboard work ships in this unit; SSB rows are the proof, and a credit-heat view is deferred until trajectory volume justifies it. | anchor=I-28:L61-L63
- **I-001/I28-E024** [SRC] (DEEP-I-001.discovery.md:L327| unit=I-28): E/DEP | IS-4 archive and H-46 clade scoring are downstream context only and must not be reimplemented. | anchor=I-28:L13-L13, L40-L45
- **I-001/I28-E025** [SRC] (DEEP-I-001.discovery.md:L328| unit=I-28 SURFACE): E/FAIL | If both flags are enabled and no CreditWalk appears, dispatcher wiring is fixed and the test repeated; rows with the credit flag unset abort as an environment-gate leak. | anchor=I-28:L84-L84
- **I-001/I28-E026** [SRC] (DEEP-I-001.discovery.md:L329| unit=I-28): E/FAIL | If a DPO extractor accepts unsealed RRA or an L2-scored hard-veto branch, extraction stops while frozen verdict contracts are preserved. | anchor=I-28:L85-L86
- **I-001/I28-E027** [SRC] (DEEP-I-001.discovery.md:L330| unit=I-28): E/FAIL | Missing branch chunk provenance keeps Slice 3 deferred; implementation may not synthesize a chunk identifier. | anchor=I-28:L87-L88, L110-L115
- **I-001/I28-E028** [SRC] (DEEP-I-001.discovery.md:L331| unit=I-28): E/DET | Production paths prohibit unwrap/expect, fabricated measurements, stale SSB baselines, and synthetic/neural attribution. | anchor=I-28:L52-L59
- **I-001/I28-E029** [SRC] (DEEP-I-001.discovery.md:L332| unit=I-28): E/TEST | Off-agent verification runs plan_gate_check, the I-28 contracts, production_caller_check, wiring_contract_check, action_type_lock, schema derivation, allowlist promotion ID, and default-off confirmation. | anchor=I-28:L103-L108
- **I-001/I28-E030** [SRC] (DEEP-I-001.discovery.md:L333| unit=I-28): E/SEC | Red-team arms cover fabricated credit, unsealed RRA, veto score, environment-gate leakage, and provenance-free demotion. | anchor=I-28:L110-L115
- **I-001/I28-E031** [SRC] (DEEP-I-001.discovery.md:L334| unit=I-28): E/TEST | Integration smoke repeats the seeded ladder with both flags on and then credit off, checking rows, schema floor, SQL round-trip, and production task-runner reachability. | anchor=I-28:L117-L117
- **I-001/I28-E032** [SRC] (DEEP-I-001.discovery.md:L335| unit=I-28): E/STATUS | Recon is closed, but the executor must re-grep task runner, ladder, verifier, registry, and allowlist immediately before implementation. | anchor=I-28:L119-L119
- **I-001/I28-E033** [SRC] (DEEP-I-001.discovery.md:L336| unit=I-28): E/LIMIT | This source is a plan-only refresh unit: it must not edit product Rust, SSB, task runner, frozen gates, or evaluator files, and the worker cannot self-certify DONE. | anchor=I-28:L121-L124
- **I-001/I28-A001** [AC] (DEEP-I-001.discovery.md:L337| unit=I-28): A/DATA | FORK-1 leaves the DPO-negative key unresolved among preserved progress_score, progress_score plus gate id, or first veto by generation order; L2 score is disallowed in all branches. | anchor=I-28:L90-L92
- **I-001/I28-A002** [AC] (DEEP-I-001.discovery.md:L338| unit=I-28): A/ARTIFACT | FORK-3 leaves JSONL writer timing unresolved between paired I-30 reader and wire-dark emission. | anchor=I-28:L93-L93
- **I-001/I28-A003** [AC] (DEEP-I-001.discovery.md:L339| unit=I-28): A/DEP | FORK-4 leaves chunk-provenance ownership unresolved between pristine-context construction and retrieval. | anchor=I-28:L94-L94
- **I-001/I28-A004** [AC] (DEEP-I-001.discovery.md:L340| unit=I-28): A/SCHEMA | The source names DagCreditAssigned as the append target and DpoPairExtracted as a row but does not say whether the latter is a distinct action type or payload kind under an existing type. | anchor=I-28:L30-L30, L34-L38
- **I-001/I28-C001** [CON] (DEEP-I-001.discovery.md:L341| unit=I-28): C/CAP | The orientation summary claims backward DAG/node credit to chunks, subplans, and templates. The refreshed source explicitly limits Slice 1 to branch-grain sibling deltas and defers node-grain/chunk provenance. | anchor=I-28:L4-L4, L17-L25, L52-L57, L90-L94; orientation:L757-L760
- **I-001/I28-C002** [CON] (DEEP-I-001.discovery.md:L342| unit=I-28): C/DATA | The summary chooses the highest-Layer-2-scoring vetoed sibling; the refreshed source freezes HardVeto semantics, forbids any L2 score for a vetoed branch, and leaves a non-L2 FORK-1 choice open. | anchor=I-28:L12-L12, L52-L57, L90-L92, L96-L98; orientation:L763-L763
- **I-001/I28-C003** [CON] (DEEP-I-001.discovery.md:L343| unit=I-28): C/FLOW | The summary says credit runs at PhaseComplete; the refreshed production seam is drive_task immediately after search-ladder dispatch and before escalation. | anchor=I-28:L27-L30, L47-L50; orientation:L766-L766
- **I-001/I28-C004** [CON] (DEEP-I-001.discovery.md:L344| unit=I-28): C/DEFER | The summary specifies demotion after three abort-preceding appearances and a changed next ranking; the refreshed source emits no demotion row until per-branch chunk provenance and ownership exist. | anchor=I-28:L23-L25, L64-L67, L82-L88, L94-L101; orientation:L764-L772
- **I-001/I28-C005** [CON] (DEEP-I-001.discovery.md:L345| unit=I-28 SURFACE): C/SURFACE | The summary allows an optional plan-view heat map; the refreshed execution unit explicitly does no dashboard work and defers heat until trajectory volume justifies it. | anchor=I-28:L61-L63; orientation:L767-L767
- **I-001/I28-C006** [CON] (DEEP-I-001.discovery.md:L346| unit=I-28): C/DATA | I-GROUNDING-MAP's dated 221/532 baseline is superseded within this source by the refreshed 606-entry/derived-297 registry observation; both demand reread at implementation. | anchor=I-28:L30-L32, L69-L73; IG:L9-L18
- **I-001/SD-I28-001** [SD] (DEEP-I-001.discovery.md:L585| unit=I-28): CORRECT summary DAG/node attribution to ratified default-off Slice-1 deterministic branch-grain sibling deltas. | anchor=I28-E001–I28-E011, I28-C001
- **I-001/SD-I28-002** [SD] (DEEP-I-001.discovery.md:L586| unit=I-28): ADD exact task-runner Option<SearchLadderOutcome> pass-through and placement before escalation; CORRECT PhaseComplete. | anchor=I28-E003–I28-E005, I28-C003
- **I-001/SD-I28-003** [SD] (DEEP-I-001.discovery.md:L587| unit=I-28 SURFACE): ADD default-off flag, wire-dark promotion row, production-caller policy, contract/schema-roundtrip gates, and refreshed registry derivation. | anchor=I28-E010–I28-E015
- **I-001/SD-I28-004** [SD] (DEEP-I-001.discovery.md:L588| unit=I-28): CORRECT L2-scored vetoed negative as forbidden; DPO selection is post-FORK-1 and must preserve frozen hard-veto semantics. | anchor=I28-E016–I28-E019, I28-A001, I28-C002
- **I-001/SD-I28-005** [SD] (DEEP-I-001.discovery.md:L589| unit=I-28): CORRECT chunk demotion/three-abort threshold to fully deferred Slice 3 pending real per-branch provenance/owner; node-grain waits for PUCT. | anchor=I28-E020–I28-E022, I28-A003, I28-C004
- **I-001/SD-I28-006** [SD] (DEEP-I-001.discovery.md:L590| unit=I-28 SURFACE): CORRECT optional immediate heat UI to no-dashboard unit/deferred heat; ADD I-30 reader-or-wire-dark artifact rule. | anchor=I28-E019, I28-E023, I28-A002, I28-C005
- **I-001/SD-I28-007** [SD] (DEEP-I-001.discovery.md:L591| unit=I-28): ADD detailed fail/abort/red-team/off-agent verification and plan-refresh execution boundaries. | anchor=I28-E024–I28-E033
- **I-001/SD-I28-008** [SD] (DEEP-I-001.discovery.md:L592| unit=I-28 SURFACE): FLAG unresolved DPO action shape and source-level temporal conflict with dated grounding constants. | anchor=I28-A004, I28-C006

#### I-001 / unit `I-36` (35 records)

- **I-001/I36-E001** [SRC] (DEEP-I-001.discovery.md:L356| unit=I-36): E/STATUS | I-36 is a strong-grounded Track-18 BUILD, effort M, Wave 4, CHEAPER-PORTABLE lever, bound to ID-3 and ID-6. | anchor=I-36:L1-L3
- **I-001/I36-E002** [SRC] (DEEP-I-001.discovery.md:L357| unit=I-36): E/CAP | It creates a versioned compiled-prompt artifact store keyed by role and backend so prompts can be promoted, rolled back, and replayed as configs. | anchor=I-36:L5-L12
- **I-001/I36-E003** [SRC] (DEEP-I-001.discovery.md:L358| unit=I-36): E/DATA | CompiledPromptVariant is a plain-string artifact, allowing existing call sites to substitute text without refactor. | anchor=I-36:L12-L17, L43-L46, L67-L72
- **I-001/I36-E004** [SRC] (DEEP-I-001.discovery.md:L359| unit=I-36): E/FLOW | Offline compilation searches instructions and exemplars for each role/backend against the C-28 fixture suite. | anchor=I-36:L14-L16
- **I-001/I36-E005** [SRC] (DEEP-I-001.discovery.md:L360| unit=I-36): E/DEP | Only a cloud-lane MIPROv2-class optimizer may compile variants; local-lane self-optimization is prohibited because the cited OPRO pattern fails below roughly 100B. | anchor=I-36:L14-L17, L34-L41, L84-L88
- **I-001/I36-E006** [SRC] (DEEP-I-001.discovery.md:L361| unit=I-36): E/INT | Routing applies I-32 role_contracts first, then looks up a promoted variant by B-37 PerRoleRuntime role/backend key. | anchor=I-36:L43-L46, L58-L64, L73-L82, L108-L113
- **I-001/I36-E007** [SRC] (DEEP-I-001.discovery.md:L362| unit=I-36): E/INT | CompiledPipelineStore/TemplateCrystallizer are storage precedents, E-8 GEPA is the optimizer lane, and I-12/IS-16 supply the promotion gates; none is reimplemented. | anchor=I-36:L34-L41, L58-L64, L73-L78, L96-L113
- **I-001/I36-E008** [SRC] (DEEP-I-001.discovery.md:L363| unit=I-36): E/OBS | Transfer-gap measurement compares source prompt versus compiled prompt on the same target backend and records the fixture pass-rate delta. | anchor=I-36:L19-L23, L43-L48, L91-L94
- **I-001/I36-E009** [SRC] (DEEP-I-001.discovery.md:L364| unit=I-36): E/DET | TransferGapMeasured is recomputable from the same ELAI_SEED. | anchor=I-36:L19-L23, L91-L94
- **I-001/I36-E010** [SRC] (DEEP-I-001.discovery.md:L365| unit=I-36): E/RECOVERY | A compiled variant below its fixture floor automatically rolls back and emits Rolledback. | anchor=I-36:L19-L23, L43-L49, L79-L82, L91-L94
- **I-001/I36-E011** [SRC] (DEEP-I-001.discovery.md:L366| unit=I-36): E/SEC | Compiler constraints remove explicit step-by-step/CoT scaffolds when extended thinking is enabled. | anchor=I-36:L25-L28, L43-L50, L84-L86
- **I-001/I36-E012** [SRC] (DEEP-I-001.discovery.md:L367| unit=I-36): E/CFG | Compiled prompts respect each backend's temperature defaults and do not treat temperature zero as determinism. | anchor=I-36:L25-L28
- **I-001/I36-E013** [SRC] (DEEP-I-001.discovery.md:L368| unit=I-36): E/COMP | A compiled variant promotes only through H-17 persisted-N, H-18 deflation, D-6 held-out, and D-7 canary gates, with the evaluator outside the optimizer write boundary. | anchor=I-36:L30-L32, L43-L52, L84-L88
- **I-001/I36-E014** [SRC] (DEEP-I-001.discovery.md:L369| unit=I-36): E/EVENT | PromptCompiled has kinds Compiled, TransferGapMeasured, and Rolledback and is registered as a live-baseline-relative +1 event. | anchor=I-36:L43-L49, L67-L72, L91-L106
- **I-001/I36-E015** [SRC] (DEEP-I-001.discovery.md:L370| unit=I-36): E/OBS | Compiled rows name variant hash and backend; transfer and rollback receipts establish the exact served/promoted artifact. | anchor=I-36:L67-L72, L91-L94, L96-L106
- **I-001/I36-E016** [SRC] (DEEP-I-001.discovery.md:L371| unit=I-36 SURFACE): E/SURFACE | SettingsWindow shows a per-role compiled-variant Svelte badge via a Tauri read and must pass cross_lang_diff.py. | anchor=I-36:L43-L50, L58-L65, L89-L94
- **I-001/I36-E017** [SRC] (DEEP-I-001.discovery.md:L372| unit=I-36): E/DEFER | Automated per-backend optimizer runs are Tier 2 because they need cloud budget and sit behind the ELAI_PROMPT_COMPILE_DISABLE kill switch. | anchor=I-36:L54-L56
- **I-001/I36-E018** [SRC] (DEEP-I-001.discovery.md:L373| unit=I-36): E/DEFER | Cross-family transfer-gap curve analysis across local/cloud model families waits for post-Wave-4 data. | anchor=I-36:L54-L57
- **I-001/I36-E019** [SRC] (DEEP-I-001.discovery.md:L374| unit=I-36): E/DATA | Tier 1 produces prompt_artifacts.rs with CompiledPromptVariant, CompiledPromptStore, promote_variant, and rollback_on_regression, plus workflow/reasoning/I-36-decisions.md. | anchor=I-36:L43-L52, L58-L60, L67-L72
- **I-001/I36-E020** [SRC] (DEEP-I-001.discovery.md:L375| unit=I-36): E/FLOW | Production chains are prompt routing -> lookup -> Compiled, fixture runner -> TransferGapMeasured -> ledger, and regression detection -> rollback -> Rolledback; test-only callers do not satisfy R7. | anchor=I-36:L79-L83
- **I-001/I36-E021** [SRC] (DEEP-I-001.discovery.md:L376| unit=I-36): E/TEST | A two-backend/seeded run with a promoted variant must emit all applicable PromptCompiled receipts under ELAI_SEED. | anchor=I-36:L91-L94
- **I-001/I36-E022** [SRC] (DEEP-I-001.discovery.md:L377| unit=I-36): E/TEST | A cloud extended-thinking request must contain zero explicit CoT/step-by-step scaffold, verifiable from the compiled string. | anchor=I-36:L84-L86, L91-L94
- **I-001/I36-E023** [SRC] (DEEP-I-001.discovery.md:L378| unit=I-36): E/DET | Production paths prohibit unwrap/expect and require blake3 rather than blake2/sha2. | anchor=I-36:L84-L88
- **I-001/I36-E024** [SRC] (DEEP-I-001.discovery.md:L379| unit=I-36): E/LIMIT | Variant promotion without ID-6 gates is P0, and optimization logic, backend adapters, crystallizer pipeline, and ledger implementation remain composition boundaries. | anchor=I-36:L84-L88, L108-L113
- **I-001/I36-E025** [SRC] (DEEP-I-001.discovery.md:L380| unit=I-36): E/EVIDENCE | SELF-DISCOVER, MIPROv2/DSPy, and Compiled-AI citations must be web-confirmed before headline use. | anchor=I-36:L96-L106
- **I-001/I36-E026** [SRC] (DEEP-I-001.discovery.md:L381| unit=I-36 SURFACE): E/STATE | B-37 routes across 11 backends, which is the stated portability surface motivating per-backend compilation. | anchor=I-36:L5-L9
- **I-001/I36-A001** [AC] (DEEP-I-001.discovery.md:L382| unit=I-36): A/DEFER | Tier 1 requires compiled-store substitution and transfer measurement while automated compilation is Tier 2; the source does not specify how initial Tier-1 compiled variants are produced or admitted. | anchor=I-36:L43-L56
- **I-001/I36-A002** [AC] (DEEP-I-001.discovery.md:L383| unit=I-36): A/CFG | ELAI_PROMPT_COMPILE_DISABLE is named as a Tier-2 kill switch, but its default value and relationship to manually served Tier-1 variants are not stated. | anchor=I-36:L54-L56
- **I-001/I36-A003** [AC] (DEEP-I-001.discovery.md:L384| unit=I-36): A/TEST | The fixture-floor metric and rollback threshold are required but not numerically or procedurally defined. | anchor=I-36:L19-L23, L47-L48, L93-L94
- **I-001/I36-I001** [INF] (DEEP-I-001.discovery.md:L385| unit=I-36): I/IDEM | Versioned role/backend keys plus exact variant hashes make serving a promoted variant replay-identifiable, but byte-idempotent store updates are not explicitly specified. | anchor=I-36:L5-L9, L67-L72, L91-L94
- **I-001/I36-C001** [CON] (DEEP-I-001.discovery.md:L386| unit=I-36): C/STATUS | The orientation describes offline production of variants as a present workflow; the numbered source defers automated per-backend optimizer runs to Tier 2, leaving Tier-1 store/harness operation partially manual or unspecified. | anchor=I-36:L43-L56; orientation:L979-L980
- **I-001/SD-I36-001** [SD] (DEEP-I-001.discovery.md:L598| unit=I-36): ADD exact artifact/store methods, 11-backend motive, post-role-contract routing order, cloud-only optimizer boundary, and no-reimplementation seams. | anchor=I36-E002–I36-E007, I36-E019–I36-E020, I36-E026
- **I-001/SD-I36-002** [SD] (DEEP-I-001.discovery.md:L599| unit=I-36): ADD deterministic transfer/rollback lifecycle, exact event kinds, promotion gates, and evaluator write-boundary isolation. | anchor=I36-E008–I36-E015
- **I-001/SD-I36-003** [SD] (DEEP-I-001.discovery.md:L600| unit=I-36): QUALIFY offline compilation: automated per-backend optimizer runs are Tier 2 behind a kill switch, leaving Tier-1 variant creation unspecified. | anchor=I36-E017, I36-A001–I36-A002, I36-C001
- **I-001/SD-I36-004** [SD] (DEEP-I-001.discovery.md:L601| unit=I-36 SURFACE): ADD UI/cross-language gate, decision artifact, web-citation caution, and FLAG undefined fixture floor. | anchor=I36-E016, I36-E019, I36-E021–I36-E025, I36-A003

#### I-001 / unit `I-37` (42 records)

- **I-001/I37-E001** [SRC] (DEEP-I-001.discovery.md:L396| unit=I-37): E/STATUS | I-37 is a strong-grounded Track-18 BUILD, effort M–L, Wave 3, VERIFIED-INTENT lever, requiring full-form expansion before execution. | anchor=I-37:L1-L3, L13-L13
- **I-001/I37-E002** [SRC] (DEEP-I-001.discovery.md:L397| unit=I-37): E/CAP | It turns each natural-language requirement into a typed EarsRequirement linked to deterministic evidence and a live status. | anchor=I-37:L5-L11, L15-L24
- **I-001/I37-E003** [SRC] (DEEP-I-001.discovery.md:L398| unit=I-37): E/DATA | pattern is an enum of five EARS stereotypes: Ubiquitous, Event-driven, State-driven, Option, and Unwanted-behaviour; freeform strings are prohibited. | anchor=I-37:L15-L24, L89-L93
- **I-001/I37-E004** [SRC] (DEEP-I-001.discovery.md:L399| unit=I-37): E/DATA | content_hash is blake3 of canonical requirement text and detects drift. | anchor=I-37:L18-L20
- **I-001/I37-E005** [SRC] (DEEP-I-001.discovery.md:L400| unit=I-37): E/DATA | bound_tests is Vec<TestSymbol> naming tests claimed to cover the requirement. | anchor=I-37:L20-L22
- **I-001/I37-E006** [SRC] (DEEP-I-001.discovery.md:L401| unit=I-37): E/DATA | mutation_coverage stores I-20's sampled score for bound tests at the latest audit. | anchor=I-37:L21-L24, L38-L46
- **I-001/I37-E007** [SRC] (DEEP-I-001.discovery.md:L402| unit=I-37): E/DATA | metamorphic_relations is Vec<MrId> resolved through the existing RelationLibrary. | anchor=I-37:L22-L24, L38-L45
- **I-001/I37-E008** [SRC] (DEEP-I-001.discovery.md:L403| unit=I-37): E/STATE | verified_status has Satisfied, Unverified, and Regressed; satisfaction requires mutation coverage at/above the applicable floor and all bound metamorphic relations to hold. | anchor=I-37:L23-L24, L31-L32
- **I-001/I37-E009** [SRC] (DEEP-I-001.discovery.md:L404| unit=I-37): E/HONESTY | A requirement lacking verifiable evidence is shown as Unverified and can never be silently dropped. | anchor=I-37:L31-L32, L47-L55, L89-L99
- **I-001/I37-E010** [SRC] (DEEP-I-001.discovery.md:L405| unit=I-37): E/INT | IntentPair/IntentPairsGate are generalized rather than rebuilt; the simplest IntentPair maps to an Ubiquitous record with one bound test and one relation. | anchor=I-37:L26-L29, L38-L45, L62-L67, L77-L83, L101-L116
- **I-001/I37-E011** [SRC] (DEEP-I-001.discovery.md:L406| unit=I-37 SURFACE): E/INTEROP | Spec-Kit and Kiro-style YAML/Markdown-frontmatter requirement lists round-trip into typed records at planning time; imported files express intent while ELAI proves satisfaction. | anchor=I-37:L26-L29
- **I-001/I37-E012** [SRC] (DEEP-I-001.discovery.md:L407| unit=I-37): E/CAP | Tier 1 explicitly provides parse_speckit_file()->Vec<EarsRequirement>. | anchor=I-37:L47-L50, L62-L64, L70-L76
- **I-001/I37-E013** [SRC] (DEEP-I-001.discovery.md:L408| unit=I-37): E/FLOW | Requirements are authored/imported at the C-50 ContractSpec PLANNING gate before the TESTING_FIRST transition. | anchor=I-37:L47-L53, L62-L65, L77-L87
- **I-001/I37-E014** [SRC] (DEEP-I-001.discovery.md:L409| unit=I-37): E/DATA | requirements.rs supplies EarsRequirement, EarsRequirementSet, verify_requirement, and parse_speckit_file. | anchor=I-37:L47-L50, L62-L64, L70-L76
- **I-001/I37-E015** [SRC] (DEEP-I-001.discovery.md:L410| unit=I-37): E/EVENT | RequirementVerified kinds are Satisfied, Unverified, and Regressed and are registered as a live-baseline-relative +1 SSB type. | anchor=I-37:L47-L54, L70-L76, L96-L109
- **I-001/I37-E016** [SRC] (DEEP-I-001.discovery.md:L411| unit=I-37): E/FLOW | Audit cycles call verify_requirement, join I-20 mutation evidence and metamorphic results, then emit RequirementVerified. | anchor=I-37:L62-L67, L84-L87, L96-L99
- **I-001/I37-E017** [SRC] (DEEP-I-001.discovery.md:L412| unit=I-37): E/LIFECYCLE | Deleting a bound test and reaudit flips the affected requirement to Regressed. | anchor=I-37:L47-L55, L96-L99
- **I-001/I37-E018** [SRC] (DEEP-I-001.discovery.md:L413| unit=I-37): E/IDEM | Imported Spec-Kit data round-trips idempotently on content_hash. | anchor=I-37:L96-L99
- **I-001/I37-E019** [SRC] (DEEP-I-001.discovery.md:L414| unit=I-37 SURFACE): E/SURFACE | G-5 cockpit renders per-requirement status chips per hunk via H-52 anchoring and Tauri/Svelte pairing; cross_lang_diff.py must pass. | anchor=I-37:L31-L36, L41-L43, L47-L54, L62-L68, L84-L99
- **I-001/I37-E020** [SRC] (DEEP-I-001.discovery.md:L415| unit=I-37): E/OBS | Satisfied chips point to real test and mutation rows, with joins recomputable from I-20 SSB evidence under ELAI_SEED. | anchor=I-37:L89-L99
- **I-001/I37-E021** [SRC] (DEEP-I-001.discovery.md:L416| unit=I-37): E/TEST | The fixture contains three EARS requirements, bound tests, and mutation scores; it exercises Satisfied, deleted-test Regressed, and no-bound-test Unverified. | anchor=I-37:L47-L55, L96-L99
- **I-001/I37-E022** [SRC] (DEEP-I-001.discovery.md:L417| unit=I-37): E/LIMIT | The stated “88% sampled mutation plus MR-3” is an example requirement expression, not a declared universal threshold. | anchor=I-37:L31-L32
- **I-001/I37-E023** [SRC] (DEEP-I-001.discovery.md:L418| unit=I-37): E/DEFER | Formal-obligation closure/H-54 verifier integration is deferred until after Wave 3. | anchor=I-37:L57-L59
- **I-001/I37-E024** [SRC] (DEEP-I-001.discovery.md:L419| unit=I-37): E/DEFER | Hierarchical/cross-requirement dependency graphs are research-grade and deferred until after an I-20 full-corpus run. | anchor=I-37:L58-L60
- **I-001/I37-E025** [SRC] (DEEP-I-001.discovery.md:L420| unit=I-37): E/DEFER | Webhook-triggered automated Spec-Kit synchronization is deferred as infrastructure rather than correctness. | anchor=I-37:L59-L60
- **I-001/I37-E026** [SRC] (DEEP-I-001.discovery.md:L421| unit=I-37): E/DET | Production paths prohibit fabricated mutation coverage, unwrap/expect, and blake2/sha2; hashes use blake3. | anchor=I-37:L89-L93
- **I-001/I37-E027** [SRC] (DEEP-I-001.discovery.md:L422| unit=I-37): E/DATA | The required decision artifact is workflow/reasoning/I-37-decisions.md. | anchor=I-37:L54-L55
- **I-001/I37-E028** [SRC] (DEEP-I-001.discovery.md:L423| unit=I-37 SURFACE): E/LIMIT | I-37 must not reimplement intent capture, metamorphic testing, ContractSpec gate machinery, cockpit anchoring, or mutation scoring. | anchor=I-37:L111-L116
- **I-001/I37-E029** [SRC] (DEEP-I-001.discovery.md:L424| unit=I-37): E/EVIDENCE | The METR citation and related review-economics headline must be web-confirmed before use. | anchor=I-37:L101-L109
- **I-001/I37-E030** [SRC] (DEEP-I-001.discovery.md:L425| unit=I-37): E/EVIDENCE | The motivation carries review-bottleneck figures of a 19% METR slowdown and AI-PR review waits 2.5–5.3x longer; these are evidence claims, not product thresholds. | anchor=I-37:L31-L36, L101-L108
- **I-001/I37-A001** [AC] (DEEP-I-001.discovery.md:L426| unit=I-37): A/CFG | The mutation floor used by verify_requirement is not specified; the only number is explicitly framed as an example. | anchor=I-37:L23-L24, L31-L32
- **I-001/I37-A002** [AC] (DEEP-I-001.discovery.md:L427| unit=I-37): A/INTEROP | The source broadly promises Kiro-style round-trip interop, but Tier-1 symbols name only parse_speckit_file and define no distinct Kiro parser/format contract. | anchor=I-37:L26-L29, L47-L50, L70-L76
- **I-001/I37-A003** [AC] (DEEP-I-001.discovery.md:L428| unit=I-37): A/DATA | Canonical requirement-text normalization for content_hash is not defined. | anchor=I-37:L18-L20
- **I-001/I37-A004** [AC] (DEEP-I-001.discovery.md:L429| unit=I-37 SURFACE): A/STATE | The source uses VERIFIED/UNVERIFIED in prose/headline but the typed enum is Satisfied/Unverified/Regressed; whether “VERIFIED” is a UI label or alias is unstated. | anchor=I-37:L1-L11, L23-L24
- **I-001/I37-C001** [CON] (DEEP-I-001.discovery.md:L430| unit=I-37): C/STATE | The orientation describes deletion as flipping to Unverified, while the numbered source's explicit typed state and fixture require Regressed. | anchor=I-37:L23-L24, L47-L55, L96-L99; orientation:L1018-L1019
- **I-001/I37-C002** [CON] (DEEP-I-001.discovery.md:L431| unit=I-37): C/INTEROP | The orientation lists Spec-Kit and Kiro imports equally; the numbered Tier-1 API explicitly implements only parse_speckit_file, leaving Kiro-specific import support ambiguous. | anchor=I-37:L26-L29, L47-L50, L70-L76; orientation:L1008-L1009
- **I-001/SD-I37-001** [SD] (DEEP-I-001.discovery.md:L607| unit=I-37): ADD exact typed fields/enums, blake3 drift key, Satisfied predicate, and content-hash idempotent round-trip. | anchor=I37-E003–I37-E008, I37-E018
- **I-001/SD-I37-002** [SD] (DEEP-I-001.discovery.md:L608| unit=I-37 SURFACE): ADD exact planning/audit/cockpit lifecycle, event kinds, real evidence joins, and full three-state fixture. | anchor=I37-E009–I37-E021
- **I-001/SD-I37-003** [SD] (DEEP-I-001.discovery.md:L609| unit=I-37): CORRECT deletion state from summary Unverified to source Regressed. | anchor=I37-E017, I37-C001
- **I-001/SD-I37-004** [SD] (DEEP-I-001.discovery.md:L610| unit=I-37): QUALIFY Kiro import: prose promises Kiro-style round-trip, but Tier-1 API exposes only parse_speckit_file. | anchor=I37-E011–I37-E012, I37-A002, I37-C002
- **I-001/SD-I37-005** [SD] (DEEP-I-001.discovery.md:L611| unit=I-37): ADD formal closure, dependency graph, and webhook-sync deferrals plus composition/non-reimplementation boundaries and review-bottleneck evidence scope. | anchor=I37-E023–I37-E030
- **I-001/SD-I37-006** [SD] (DEEP-I-001.discovery.md:L612| unit=I-37 SURFACE): FLAG example-only mutation floor, canonicalization, and VERIFIED-versus-Satisfied naming. | anchor=I37-E022, I37-A001, I37-A003–I37-A004

#### I-001 / unit `I-48` (30 records)

- **I-001/I48-E001** [SRC] (DEEP-I-001.discovery.md:L441| unit=I-48): E/STATUS | I-48 is a strong-grounded Track-20 REFACTOR, effort S, Wave 2, STEERING lever and Tier-1 competitor steal. | anchor=I-48:L1-L5, L58-L60
- **I-001/I48-E002** [SRC] (DEEP-I-001.discovery.md:L442| unit=I-48): E/CAP | Runtime dispatch prompt templates move from inline literals to static Markdown files under crates/<crate>/prompts and are imported with include_str!. | anchor=I-48:L5-L10, L13-L20
- **I-001/I48-E003** [SRC] (DEEP-I-001.discovery.md:L443| unit=I-48): E/REVIEW | File artifacts make prompt changes diffable and code-reviewable and make I-36 compiled variants/I-47 frozen blocks pinnable. | anchor=I-48:L5-L11, L62-L66
- **I-001/I48-E004** [SRC] (DEEP-I-001.discovery.md:L444| unit=I-48): E/FLOW | The first task inventories dispatch/agent/role modules for prompt-like multiline or role-framing literals above an approximate 200-character threshold. | anchor=I-48:L16-L20, L25-L29
- **I-001/I48-E005** [SRC] (DEEP-I-001.discovery.md:L445| unit=I-48): E/FLOW | Extraction proceeds per crate, preserving the existing typed substitution path and auditing each batch under one-change-at-a-time discipline. | anchor=I-48:L16-L23, L58-L60
- **I-001/I48-E006** [SRC] (DEEP-I-001.discovery.md:L446| unit=I-48): E/TEST | A contract scans dispatch modules and fails on new long inline prompt literals; only frozen non-prompt strings may be allowlisted. | anchor=I-48:L16-L20, L40-L49
- **I-001/I48-E007** [SRC] (DEEP-I-001.discovery.md:L447| unit=I-48): E/LIMIT | Extraction is byte-preserving and may not rewrite prompt content. | anchor=I-48:L21-L23, L40-L49
- **I-001/I48-E008** [SRC] (DEEP-I-001.discovery.md:L448| unit=I-48): E/DEP | No new template-engine dependency is allowed; include_str! feeds the unchanged D-1 typed substitution system. | anchor=I-48:L21-L23, L40-L49, L62-L66
- **I-001/I48-E009** [SRC] (DEEP-I-001.discovery.md:L449| unit=I-48): E/LIMIT | .claude/agents/*.md is excluded because it is harness rather than runtime prompt scope. | anchor=I-48:L21-L23
- **I-001/I48-E010** [SRC] (DEEP-I-001.discovery.md:L450| unit=I-48): E/SCOPE | Ship-now scope is orchestrator/providers dispatch prompts plus the ban test. | anchor=I-48:L31-L34, L36-L38
- **I-001/I48-E011** [SRC] (DEEP-I-001.discovery.md:L451| unit=I-48 SURFACE): E/DEFER | Python-sidecar prompt strings are a separate cross-language slice and dashboard-side copy is deferred. | anchor=I-48:L31-L34
- **I-001/I48-E012** [SRC] (DEEP-I-001.discovery.md:L452| unit=I-48): E/STATUS | Inline-literal count is initially unknown; inventory revises the estimate, and a blast radius above 40 files pauses for impact mapping. | anchor=I-48:L25-L29
- **I-001/I48-E013** [SRC] (DEEP-I-001.discovery.md:L453| unit=I-48): E/FLOW | Production reachability is preserved because every dispatch call site assembles byte-identical prompt output from a file instead of its former literal. | anchor=I-48:L40-L46
- **I-001/I48-E014** [SRC] (DEEP-I-001.discovery.md:L454| unit=I-48): E/EVENT | I-48 adds no action type; assembled prompt bytes remain in existing journaled requests, with later provenance attributable to I-36 PromptCompiled. | anchor=I-48:L50-L55, L62-L66
- **I-001/I48-E015** [SRC] (DEEP-I-001.discovery.md:L455| unit=I-48 SURFACE): E/SURFACE | No UI changes ship because this is a build-time discipline refactor. | anchor=I-48:L36-L38
- **I-001/I48-E016** [SRC] (DEEP-I-001.discovery.md:L456| unit=I-48): E/TEST | Executor and auditor prompt assemblies are compared before/after using blake3 over journaled request bodies and must be byte-identical. | anchor=I-48:L45-L55
- **I-001/I48-E017** [SRC] (DEEP-I-001.discovery.md:L457| unit=I-48): E/TEST | A seeded inline-literal reintroduction must fail the ban contract. | anchor=I-48:L50-L56
- **I-001/I48-E018** [SRC] (DEEP-I-001.discovery.md:L458| unit=I-48): E/REVIEW | Extracted prompt files must render as ordinary git diffs. | anchor=I-48:L50-L56
- **I-001/I48-E019** [SRC] (DEEP-I-001.discovery.md:L459| unit=I-48): E/LIMIT | Partial extraction cannot “pass” by allowlisting prompt literals across half the codebase. | anchor=I-48:L47-L49
- **I-001/I48-E020** [SRC] (DEEP-I-001.discovery.md:L460| unit=I-48): E/DEP | E-8 GEPA reads/writes file artifacts, I-36 compiles/diffs them, and I-47's frozen injection block becomes one; none of those systems is restated. | anchor=I-48:L62-L66
- **I-001/I48-E021** [SRC] (DEEP-I-001.discovery.md:L461| unit=I-48): E/EVIDENCE | The gajae prompt-directory pattern is the competitor source and SL-10 original-read rule applies before execution even though the directory shape already exposes the discipline. | anchor=I-48:L13-L15, L58-L60
- **I-001/I48-A001** [AC] (DEEP-I-001.discovery.md:L462| unit=I-48): A/CFG | The inline-literal threshold is approximate (“~200 chars or multiline with role framing”), not a fixed contract value. | anchor=I-48:L16-L20
- **I-001/I48-A002** [AC] (DEEP-I-001.discovery.md:L463| unit=I-48 SURFACE): A/SCOPE | “Every dispatch prompt template” is the headline target, but current Tier 1 explicitly excludes Python and dashboard text; completion of the global headline spans later slices. | anchor=I-48:L1-L11, L31-L34
- **I-001/I48-A003** [AC] (DEEP-I-001.discovery.md:L464| unit=I-48): A/TEST | The source names executor and auditor comparison in the trigger but does not enumerate every role/provider prompt that must have a golden pre-extraction body. | anchor=I-48:L50-L55
- **I-001/I48-C001** [CON] (DEEP-I-001.discovery.md:L465| unit=I-48 SURFACE): C/SCOPE | The orientation presents every dispatch prompt as moved now; the numbered source limits ship-now extraction to orchestrator/providers and defers sidecar/dashboard material. | anchor=I-48:L31-L34; orientation:L1309-L1311
- **I-001/SD-I48-001** [SD] (DEEP-I-001.discovery.md:L618| unit=I-48): ADD inventory-first workflow, approximate literal heuristic, per-crate extraction discipline, impact-map pause, and frozen non-prompt allowlist. | anchor=I48-E004–I48-E006, I48-E012, I48-E019
- **I-001/SD-I48-002** [SD] (DEEP-I-001.discovery.md:L619| unit=I-48): ADD explicit exclusions: no content rewrite, template dependency, or harness-agent edit; D-1 remains unchanged; SL-10 original competitor read still applies. | anchor=I48-E007–I48-E009, I48-E021
- **I-001/SD-I48-003** [SD] (DEEP-I-001.discovery.md:L620| unit=I-48 SURFACE): CORRECT global “every prompt now” scope to orchestrator/providers Tier 1 with Python/dashboard slices deferred. | anchor=I48-E010–I48-E011, I48-A002, I48-C001
- **I-001/SD-I48-004** [SD] (DEEP-I-001.discovery.md:L621| unit=I-48): ADD production byte-identity reachability, existing-journal/no-event behavior, seeded negative arm, and ordinary git-diff proof. | anchor=I48-E013–I48-E018
- **I-001/SD-I48-005** [SD] (DEEP-I-001.discovery.md:L622| unit=I-48 SURFACE): FLAG approximate threshold and incomplete golden role/provider inventory. | anchor=I48-A001, I48-A003

#### I-001 / unit `RELATIONAL` (34 records)

- **I-001/IX-001** [IX] (DEEP-I-001.discovery.md:L475| unit=RELATIONAL): The grounding map is a dated substrate/absence/correction contract, not a current-state oracle; every numbered executor must re-grep live seams and baselines. | anchor=IG-E001–IG-E006, IG-E024–IG-E026, IG-I001
- **I-001/IX-002** [IX] (DEEP-I-001.discovery.md:L476| unit=RELATIONAL): I-8 turns H-39's abstention scoring into an explicit tool path, uses H-49 for rollback, and attaches its watchdog to I-17; the tool/cache can precede the later integrations. | anchor=I8-E002–I8-E013
- **I-001/IX-003** [IX] (DEEP-I-001.discovery.md:L477| unit=RELATIONAL SURFACE): I-8 abort outcomes feed the single calibration surface that I-23 unifies; neither plan duplicates refusal or scoring math. | anchor=I8-E004, I8-E012–I8-E013, I23-E002, I23-E017
- **I-001/IX-004** [IX] (DEEP-I-001.discovery.md:L478| unit=RELATIONAL): I-13's three layers are complementary: write-time refusal protects stored memory, condensed ToolResult protects context budget, and TraceQuery recovers exact source ranges on demand. | anchor=I13-E002–I13-E015
- **I-001/IX-005** [IX] (DEEP-I-001.discovery.md:L479| unit=RELATIONAL): I-13 extends the shipped safety/memory stack at write time and must not duplicate H-31/H-32, E-37, MINJA, G-9, or H-6. | anchor=IG-E017, IG-E051, I13-E005–I13-E008
- **I-001/IX-006** [IX] (DEEP-I-001.discovery.md:L480| unit=RELATIONAL): I-15 exact-key lookup precedes semantic lookup; I-10 supplies TTL choices, H-8 supplies store/confinement, and H-51 gates future cross-session warmup. | anchor=I15-E002–I15-E014
- **I-001/IX-007** [IX] (DEEP-I-001.discovery.md:L481| unit=RELATIONAL): I-18 chains the SSB evidence on which I-28's sealed DPO provenance relies; an unsealed record cannot enter the I-30 training-data path. | anchor=I18-E003–I18-E008, I28-E017–I28-E019
- **I-001/IX-008** [IX] (DEEP-I-001.discovery.md:L482| unit=RELATIONAL): I-18's security boundary is offline-first: local chain/stack/AI-BOM ships independently, while user-approved, anti-slopsquat-checked Rekor remains optional and fail-closed when requested. | anchor=I18-E009–I18-E014
- **I-001/IX-009** [IX] (DEEP-I-001.discovery.md:L483| unit=RELATIONAL): I-23 supplies a single tau identity to existing refusal and future I-4 search; I-4 activation is later, and G-7 owns its own planned migration. | anchor=I23-E003–I23-E012
- **I-001/IX-010** [IX] (DEEP-I-001.discovery.md:L484| unit=RELATIONAL): I-28 depends on I-4 returning SearchLadderOutcome and I-17 frozen verdicts; its first consumer-facing training artifact is gated on I-30 reader/wire-dark coordination. | anchor=I28-E001–I28-E006, I28-E016–I28-E019
- **I-001/IX-011** [IX] (DEEP-I-001.discovery.md:L485| unit=RELATIONAL): I-28's lifecycle is strictly sliced: branch credit now/default-off, pruned-verdict DPO only after FORK-1, and node/chunk credit only after PUCT/provenance. | anchor=I28-E006–I28-E023, I28-A001–I28-A003
- **I-001/IX-012** [IX] (DEEP-I-001.discovery.md:L486| unit=RELATIONAL): I-28 sequencing preserves veto supremacy: a hard-vetoed sibling may be selected only by non-L2 metadata after FORK-1, never by an invented verifier score. | anchor=I28-E016–I28-E018, I28-A001, I28-C002
- **I-001/IX-013** [IX] (DEEP-I-001.discovery.md:L487| unit=RELATIONAL): I-48 file extraction provides reviewable source artifacts; I-36 subsequently compiles per-backend variants from them, and I-47 can pin a frozen injection block as another file. | anchor=I48-E002–I48-E003, I48-E020, I36-E002–I36-E003
- **I-001/IX-014** [IX] (DEEP-I-001.discovery.md:L488| unit=RELATIONAL): I-36 serves a compiled variant only after I-32 role scoping and uses B-37 for the role/backend key; its optimizer and promotion evaluator stay outside each other's write boundaries. | anchor=I36-E006–I36-E007, I36-E013
- **I-001/IX-015** [IX] (DEEP-I-001.discovery.md:L489| unit=RELATIONAL): I-36's delivery lifecycle is compile -> fixture comparison -> gated promotion -> served hash receipt -> automatic rollback on regression. | anchor=I36-E004, I36-E008–I36-E015
- **I-001/IX-016** [IX] (DEEP-I-001.discovery.md:L490| unit=RELATIONAL SURFACE): I-37's evidence lifecycle is planning import/authorship -> bound tests/MRs -> I-20 sampled coverage -> audit status -> per-hunk cockpit chip; deleting evidence creates Regressed rather than erasing the requirement. | anchor=I37-E004–I37-E020
- **I-001/IX-017** [IX] (DEEP-I-001.discovery.md:L491| unit=RELATIONAL): I-37 composes C-29 intent, C-50 planning gates, I-20 mutation evidence, RelationLibrary, and G-5/H-52 presentation without owning those subsystems. | anchor=I37-E010, I37-E013–I37-E020, I37-E028
- **I-001/IX-018** [IX] (DEEP-I-001.discovery.md:L492| unit=RELATIONAL): The chunk's hash doctrine is consistently blake3 for dead-end/query/integrity/requirement/prompt evidence; sources explicitly ban blake2/sha2 on new paths. | anchor=I8-E006, I8-E019, I15-E004, I15-E019, I18-E003, I18-E006, I18-E018, I36-E023, I37-E004, I37-E026, I48-E016
- **I-001/IX-019** [IX] (DEEP-I-001.discovery.md:L493| unit=RELATIONAL): Optional/new external dependencies never become silent defaults: I-13 YARA and I-18 cosign/Rekor need explicit approval/verification, while I-36 cloud optimization is budget-gated. | anchor=I13-E005, I18-E011–I18-E014, I36-E005, I36-E017
- **I-001/IX-020** [IX] (DEEP-I-001.discovery.md:L494| unit=RELATIONAL): Evidence ownership stays outside the model: I-28 credit is deterministic, I-36 promotion passes frozen ledger gates, I-37 coverage is journal-recomputable, and I-18 makes the resulting record tamper-evident. | anchor=I28-E007, I28-E016, I36-E009, I36-E013, I37-E020, I18-E003–I18-E008
- **I-001/AC-001** [AC] (DEEP-I-001.discovery.md:L502| unit=RELATIONAL): temporal | Grounding baselines and wiring observations are dated and must not be promoted to current status. | anchor=IG-I001, IG-A002, I28-C006
- **I-001/AC-002** [AC] (DEEP-I-001.discovery.md:L503| unit=RELATIONAL): external evidence | Research identifiers, vendor figures, CycloneDX version, and Rekor/cosign details require execution-time verification. | anchor=IG-A001, I18-A001, I18-A003
- **I-001/AC-003** [AC] (DEEP-I-001.discovery.md:L504| unit=RELATIONAL): dead-end identity | The canonical AST/approach shape and exact tool-name relationship are undefined. | anchor=I8-A001–I8-A002
- **I-001/AC-004** [AC] (DEEP-I-001.discovery.md:L505| unit=RELATIONAL SURFACE): rollout | I-8 lacks the named halting-bound key, and its UI is deferred despite the summary's unconditional presentation. | anchor=I8-A003, I8-C001
- **I-001/AC-005** [AC] (DEEP-I-001.discovery.md:L506| unit=RELATIONAL): guard | I-13 lacks a numeric false-positive floor, concrete Tier-1 pattern catalog, and TraceQuery path/range authorization contract. | anchor=I13-A001–I13-A003
- **I-001/AC-006** [AC] (DEEP-I-001.discovery.md:L507| unit=RELATIONAL): cache | I-15 leaves canonical query normalization, vector threshold, and final event schema open. | anchor=I15-A001–I15-A003, I15-C001
- **I-001/AC-007** [AC] (DEEP-I-001.discovery.md:L508| unit=RELATIONAL): attestation | Rekor implementation, field-migration semantics, and external schema identity remain choices; Rekor push is deferred despite broad summary wording. | anchor=I18-A001–I18-A003, I18-C001
- **I-001/AC-008** [AC] (DEEP-I-001.discovery.md:L509| unit=RELATIONAL): summary-only dependency | I-45 coordination appears only in orientation and is absent from the numbered I-18 composition contract. | anchor=I18-C002
- **I-001/AC-009** [AC] (DEEP-I-001.discovery.md:L510| unit=RELATIONAL): calibration | Local threshold derivation and the future I-4 seam are open; G-7 migration is future-owned, not current. | anchor=I23-A001–I23-A002, I23-C001
- **I-001/AC-010** [AC] (DEEP-I-001.discovery.md:L511| unit=RELATIONAL): credit model | I-28's negative selection, writer activation, provenance owner, and DpoPairExtracted registry shape are unresolved. | anchor=I28-A001–I28-A004
- **I-001/AC-011** [SD] (DEEP-I-001.discovery.md:L512| unit=RELATIONAL SURFACE): refreshed I-28 correction | The summary's node/chunk/template walk, L2-scored veto negative, PhaseComplete hook, three-abort demotion, and optional heat UI conflict with the refreshed sliced plan. | anchor=I28-C001–I28-C005
- **I-001/AC-012** [AC] (DEEP-I-001.discovery.md:L513| unit=RELATIONAL): prompt compilation | Tier-1 variant provenance, kill-switch semantics, and fixture floor are open because automated compilation is deferred. | anchor=I36-A001–I36-A003, I36-C001
- **I-001/AC-013** [AC] (DEEP-I-001.discovery.md:L514| unit=RELATIONAL): requirements | I-37 leaves the mutation floor, canonical text, Kiro-specific parser, and VERIFIED/Satisfied label mapping unresolved; deletion status conflicts with the summary. | anchor=I37-A001–I37-A004, I37-C001–I37-C002
- **I-001/AC-014** [AC] (DEEP-I-001.discovery.md:L515| unit=RELATIONAL): prompt extraction | I-48's approximate threshold, global completion boundary, and golden role inventory remain open; the summary overstates immediate all-dispatch scope. | anchor=I48-A001–I48-A003, I48-C001

### Ledger I-002 — I-STEAL-GROUNDING-MAP, I-0_INDEX, I-5, I-10, I-29, I-46, I-54

Source: `docs/thefinalglance/feature-discovery/deep/DEEP-I-002.discovery.md`
Extracted atoms: **654**

#### I-002 / unit `I-0` (353 records)

- **I-002/I002-IDX-001** [SRC] (DEEP-I-002.discovery.md:L81| unit=I-0): [E][scope] I-plan contains 54 sessions across Tracks 12-20 with no reserved I track. Anchor: line 1.
- **I-002/I002-IDX-002** [SRC] (DEEP-I-002.discovery.md:L82| unit=I-0): [E][provenance] Tracks 12-18 were authored from the named consolidated/review/dossier corpus on 2026-06-10. Anchors: lines 3, 19-37.
- **I-002/I002-IDX-003** [SRC] (DEEP-I-002.discovery.md:L83| unit=I-0): [E][scope][non-goal] Track 19 adds only design-phase critique, critique-only operation, cheap Phase-0 falsifiers, and zero-retention export; token-efficiency restatements and the already-covered verifier wedge are excluded. Anchor: lines 4-11.
- **I-002/I002-IDX-004** [SRC] (DEEP-I-002.discovery.md:L84| unit=I-0): [E][non-goal] GTM/willingness-to-pay and SOC 2 are deliberately not engineering sessions. Anchor: lines 10-11.
- **I-002/I002-IDX-005** [SRC] (DEEP-I-002.discovery.md:L85| unit=I-0): [E][scope] Track 20 derives from the STEAL-LIST Tier-1/Tier-2 residue; Tier-3 overlaps are reference material, not duplicated sessions. Anchor: lines 12-17.
- **I-002/I002-IDX-006** [CON] (DEEP-I-002.discovery.md:L86| unit=I-0 SURFACE): [E][C][non-goal] A wholesale gajae provider-layer port is rejected because B-37 plus CLI bridges/OAuth/local serving is broader. Anchors: lines 15-17, 151-165.
- **I-002/I002-IDX-007** [SRC] (DEEP-I-002.discovery.md:L87| unit=I-0): [E][evidence] The main grounding map carries `IS-1..IS-19`, `IN-1..IN-14`, `IC-1..IC-18`, and live 221/532 constants; its anchors were re-grepped because `elai.map` was stale. Anchor: lines 43-47.
- **I-002/I002-IDX-008** [SRC] (DEEP-I-002.discovery.md:L88| unit=I-0): [E][evidence] The Track-20 map carries `SL-1..SL-11` and `SLC-1..SLC-3`, including shipped-RRA, shipped-checkpoint, wrap-not-delete, activation, and ast-grep-ban corrections. Anchor: lines 48-51.
- **I-002/I002-IDX-009** [SRC] (DEEP-I-002.discovery.md:L89| unit=I-0): [E][dependency] I composes with B through H and restates none; track numbering continues H's 1-11 while I session numbering restarts at I-1. Anchor: lines 53-56.
- **I-002/I002-IDX-010** [SRC] (DEEP-I-002.discovery.md:L90| unit=I-0): [E][workflow] Every I-session execution must read, in order, its grounding slice; F wiring canon and R1-R9; G deltas; applicable H families; I-0; I summary; then its detail file. Anchor: lines 58-68.
- **I-002/I002-IDX-011** [SRC] (DEEP-I-002.discovery.md:L91| unit=I-0): [E][done-gate] Every L/XL compressed detail file must be expanded to full form before execution and every anchor re-grepped. Anchor: lines 66-68.
- **I-002/I002-IDX-012** [SRC] (DEEP-I-002.discovery.md:L92| unit=I-0): [E][dependency] Track-specific mandatory reads are declared for Tracks 13/16, 14, 15, 17, 18, and 20. Anchor: lines 69-74.
- **I-002/I002-IDX-013** [SRC] (DEEP-I-002.discovery.md:L93| unit=I-0): [E][compliance] Track-20 ports must read named competitor sources; scope based only on repository summaries violates W-3. Anchor: lines 72-74.
- **I-002/I002-IDX-014** [SRC] (DEEP-I-002.discovery.md:L97| unit=I-0): [E][architecture] Track 12 separates pure `Intent -> Directive` reasoning from a sole side-effect supervisor, isolates parallel workspaces, deterministically merges, and hardens SSB/checkpoints. Anchor: lines 78-85.
- **I-002/I002-IDX-015** [SRC] (DEEP-I-002.discovery.md:L98| unit=I-0): [E][security] Track 12's supervisor is the structural boundary all later tracks rely on; gates run before side effects and the generator cannot reach gate code. Anchor: lines 79-85.
- **I-002/I002-IDX-016** [SRC] (DEEP-I-002.discovery.md:L99| unit=I-0): [E][architecture] Track 13 demotes the LLM to stateless hypothesis generation, gives the orchestrator state-tree ownership, prunes failed branches from context, and ranks by receipts. Anchor: lines 87-92.
- **I-002/I002-IDX-017** [SRC] (DEEP-I-002.discovery.md:L100| unit=I-0): [E][rollout] Track 13's MCTS capability must ship as best-of-N, then tournament, then tree; I-5 measures the vividness trap before claiming it. Anchor: lines 89-92.
- **I-002/I002-IDX-018** [SRC] (DEEP-I-002.discovery.md:L101| unit=I-0): [E][architecture] Track 14 supplies byte-frozen prefixes, TTL economics, max-token clamping, belief-state handoffs, deterministic compression, per-config measurement, and a receipt for every middleware transform. Anchor: lines 94-100.
- **I-002/I002-IDX-019** [SRC] (DEEP-I-002.discovery.md:L102| unit=I-0): [E][architecture] Track 15 composes shipped verification into an in-loop tri-layer veto, bounded mutation, independent contracts, progress rewards, unified calibration, hash chains, and scorer canaries. Anchor: lines 102-109.
- **I-002/I002-IDX-020** [SRC] (DEEP-I-002.discovery.md:L103| unit=I-0): [E][architecture] Track 16 trains routing from journaled verifier outcomes, scales budgets by serial depth, stops exploration on marginal verified-pass-per-dollar collapse, instruments the convergence curve, and harvests DPO pairs. Anchor: lines 111-116.
- **I-002/I002-IDX-021** [SRC] (DEEP-I-002.discovery.md:L104| unit=I-0): [E][rollout][security] Track 17 performs rejection-sampling SFT before GRPO; GRPO is last and blocked by guards, with all learned weights treated as configuration promotions. Anchor: lines 118-123.
- **I-002/I002-IDX-022** [SRC] (DEEP-I-002.discovery.md:L105| unit=I-0): [E][architecture] Track 18 replaces prompt-shaped controls with nine named deterministic mechanisms, including schema roles, decision governance, pinned grammars, sequence rules, compiled prompts, EARS requirements, QDELTA, repo-derived state, and an RRA wrapper/spec. Anchor: lines 125-134.
- **I-002/I002-IDX-023** [SRC] (DEEP-I-002.discovery.md:L106| unit=I-0 SURFACE): [E][constraint] Every Track-19 LLM critique surface is advisory and never load-bearing; only the off-agent spine decides DONE. Anchor: lines 136-149.
- **I-002/I002-IDX-024** [SRC] (DEEP-I-002.discovery.md:L107| unit=I-0): [E][architecture] Track 20 is primarily extension/activation over shipped code; only I-49 and I-54 are builds and both are binding-read reference ports. Anchor: lines 151-165.
- **I-002/I002-IDX-025** [SRC] (DEEP-I-002.discovery.md:L111| unit=I-0): [E][non-goal] Declined: voice-first input, Shapley/SHARP before I-28-era RL infrastructure, proactive DTMC without a trace corpus, full TTD-DR, local-lane OPRO optimization, and routine Lean gating. Anchor: lines 167-171.
- **I-002/I002-IDX-026** [SRC] (DEEP-I-002.discovery.md:L112| unit=I-0): [E][non-goal] Track 20 declines the wholesale provider layer, redundant rate-limit/backoff, niche clients absent demand, hindsight memory, BM25 tool search, generator-side review, overlapping media tools, agent cron/monitor/jobs, receipt-only returns, duplicate conflict detection, duplicate experiment isolation, TUI port, a standalone lineage session, LSP-as-verifier duplication, and Stop-hook continuation. Anchor: lines 172-182.
- **I-002/I002-IDX-027** [SRC] (DEEP-I-002.discovery.md:L113| unit=I-0): [E][status] Valid session states are `PLANNED`, `IN PROGRESS`, `DONE`, and `DEFERRED`. Anchor: lines 184-185.
- **I-002/I002-IDX-028** [SRC] (DEEP-I-002.discovery.md:L114| unit=I-0): [E][done-gate] `DONE` means live `wiring_contract_check.py --run-trigger PASS`; cargo-green alone is insufficient. Anchor: lines 184-187.
- **I-002/I002-IDX-029** [SRC] (DEEP-I-002.discovery.md:L118| unit=I-0): [E][done-gate] W-1 defines the done gate as the run-trigger wiring check. Anchor: lines 192-193.
- **I-002/I002-IDX-030** [SRC] (DEEP-I-002.discovery.md:L119| unit=I-0): [E][flags][rollout] W-2 bans terminal default-OFF env gates; I-29/I-31 may use existing allowlisted kill-switch lanes, while new gates ship default-on with `ELAI_*_DISABLE`. Anchor: lines 194-195.
- **I-002/I002-IDX-031** [SRC] (DEEP-I-002.discovery.md:L120| unit=I-0): [E][traceability] W-3 requires every plan claim to map to a grounding-map/source ID. Anchor: line 196.
- **I-002/I002-IDX-032** [SRC] (DEEP-I-002.discovery.md:L121| unit=I-0 SURFACE): [E][UI][done-gate] W-4 requires `frontend_pair` and `cross_lang_diff` PASS for dashboard signals. Anchor: line 197.
- **I-002/I002-IDX-033** [SRC] (DEEP-I-002.discovery.md:L122| unit=I-0): [E][observability] W-5 requires a live end-to-end `observable_effect`. Anchor: line 198.
- **I-002/I002-IDX-034** [SRC] (DEEP-I-002.discovery.md:L123| unit=I-0 SURFACE): [E][UI][done-gate] W-6 makes the dashboard surface part of done unless its absence is justified. Anchor: line 199.
- **I-002/I002-IDX-035** [SRC] (DEEP-I-002.discovery.md:L124| unit=I-0): [E][evidence] W-7 requires execution-time and audit-time re-grep of every file/line anchor. Anchor: line 200.
- **I-002/I002-IDX-036** [SRC] (DEEP-I-002.discovery.md:L125| unit=I-0): [E][scope] W-8 makes one session one shippable unit with one done gate. Anchor: line 201.
- **I-002/I002-IDX-037** [SRC] (DEEP-I-002.discovery.md:L126| unit=I-0): [E][schema] W-9 requires live re-reading of `SCHEMA_VERSION` and `TOTAL_VARIANTS` for every baseline-plus-N claim. Anchor: line 202.
- **I-002/I002-IDX-038** [SRC] (DEEP-I-002.discovery.md:L127| unit=I-0): [E][rollout] W-10 ships deterministic in-process legs now and separately gates network/GPU/subprocess legs with deterministic fixtures exercising the real path. Anchors: lines 203-204.
- **I-002/I002-IDX-039** [SRC] (DEEP-I-002.discovery.md:L128| unit=I-0): [E][architecture] GD-1 makes reimplementation of a shipped consumed symbol a P0; sessions compose rather than restate. Anchors: lines 206-208.
- **I-002/I002-IDX-040** [SRC] (DEEP-I-002.discovery.md:L129| unit=I-0): [E][determinism] GD-2 requires every emitted number to recompute from SSB under `ELAI_SEED`. Anchor: line 208.
- **I-002/I002-IDX-041** [SRC] (DEEP-I-002.discovery.md:L130| unit=I-0): [E][rollout] GD-4 forbids default-on before false-positive/over-block rates are bounded on at least three real runs. Anchors: lines 208-209.
- **I-002/I002-IDX-042** [SRC] (DEEP-I-002.discovery.md:L131| unit=I-0): [E][truthfulness] GD-5 forbids guarantees on fail-open, RunPod-pending, or cloud-opaque paths. Anchors: lines 209-210.
- **I-002/I002-IDX-043** [SRC] (DEEP-I-002.discovery.md:L132| unit=I-0): [E][measurement] HD-1 requires efficiency numbers measured per configuration, never multiplied. Anchors: lines 212-213.
- **I-002/I002-IDX-044** [SRC] (DEEP-I-002.discovery.md:L133| unit=I-0): [E][cache] HD-2 makes any byte-stable-prefix cache bust a test failure. Anchors: lines 213-214.
- **I-002/I002-IDX-045** [SRC] (DEEP-I-002.discovery.md:L134| unit=I-0): [E][generation] HD-3 says output-token wins are 5x and reasoning roles are never force-JSON encoded. Anchor: line 214.
- **I-002/I002-IDX-046** [SRC] (DEEP-I-002.discovery.md:L135| unit=I-0): [E][security] HD-4 makes model-driving gates auditor-driven and off-agent. Anchors: lines 214-215.
- **I-002/I002-IDX-047** [SRC] (DEEP-I-002.discovery.md:L136| unit=I-0): [E][evidence] HD-5 distinguishes valid/rendered/claimed artifacts from verified artifacts. Anchor: line 215.
- **I-002/I002-IDX-048** [SRC] (DEEP-I-002.discovery.md:L137| unit=I-0): [E][dependency] TD, VD, RD, RV, and CL rule families bind the named I sessions by concern. Anchors: lines 216-219.
- **I-002/I002-IDX-049** [SRC] (DEEP-I-002.discovery.md:L138| unit=I-0): [E][state][security] ID-1 gives canonical state to the orchestrator, hard-excludes failed siblings/raw tracebacks/dead hypotheses, permits only a compressed contrastive constraint, and requires measuring the effect. Anchors: lines 221-227.
- **I-002/I002-IDX-050** [SRC] (DEEP-I-002.discovery.md:L139| unit=I-0): [E][concurrency][security] ID-2 forbids shared mutable workspaces for branches, permits merges only at deterministic barriers, and blocks root escape. Anchors: lines 228-230.
- **I-002/I002-IDX-051** [SRC] (DEEP-I-002.discovery.md:L140| unit=I-0): [E][lifecycle][observability] ID-3 requires a reversible SSB receipt for every context-to-wire transform and exact replay of model-visible bytes. Anchors: lines 231-234.
- **I-002/I002-IDX-052** [SRC] (DEEP-I-002.discovery.md:L141| unit=I-0): [E][selection][security] ID-4 restricts selection/pruning/promotion to off-agent evidence; neural scores may order survivors but never revive a vetoed branch, and the SSB records why the winner won. Anchors: lines 235-239.
- **I-002/I002-IDX-053** [SRC] (DEEP-I-002.discovery.md:L142| unit=I-0): [E][measurement] ID-5 forbids paper constants as gates, acceptance criteria, or headlines before local engine/corpus measurement; paper values remain `[DESIGN]/[WEAK]` hypotheses. Anchors: lines 240-244.
- **I-002/I002-IDX-054** [SRC] (DEEP-I-002.discovery.md:L143| unit=I-0): [E][lifecycle][security] ID-6 treats every learned/tuned artifact as a config promotion requiring the named ledger gates and an evaluator outside the loop's write boundary. Anchors: lines 245-249.
- **I-002/I002-IDX-055** [SRC] (DEEP-I-002.discovery.md:L144| unit=I-0 SURFACE): [E][done-gate] COMPLETE requires workspace cargo check, auditor PASS, live wiring-contract PASS, applicable cross-language PASS, shipped/justified dashboard, applicable GD-4 bound, and delegated `memory-writer`. Anchors: lines 251-258.
- **I-002/I002-IDX-056** [SRC] (DEEP-I-002.discovery.md:L148| unit=I-0): [E][I-1][data] I-1 defines pure `Intent -> Directive` transitions. Anchors: lines 265-267.
- **I-002/I002-IDX-057** [SRC] (DEEP-I-002.discovery.md:L149| unit=I-0): [E][I-1][security] Its supervisor alone executes side effects and does so only after gates; generator reachability to gate code is forbidden. Anchors: lines 265-267.
- **I-002/I002-IDX-058** [SRC] (DEEP-I-002.discovery.md:L150| unit=I-0): [E][I-1][test] A contract test proves the “no gate satisfiable from inside the boundary” invariant. Anchors: lines 266-267.
- **I-002/I002-IDX-059** [SRC] (DEEP-I-002.discovery.md:L151| unit=I-0): [E][I-2][platform] I-2 uses Linux overlayfs and pooled warm worktrees on Windows. Anchors: lines 268-269.
- **I-002/I002-IDX-060** [SRC] (DEEP-I-002.discovery.md:L152| unit=I-0): [E][I-2][concurrency] Branch workspaces are isolated and merge deterministically at a barrier. Anchors: lines 268-269.
- **I-002/I002-IDX-061** [SRC] (DEEP-I-002.discovery.md:L153| unit=I-0): [E][I-2][extension] A cloud-sandbox adapter is optional. Anchor: line 269.
- **I-002/I002-IDX-062** [SRC] (DEEP-I-002.discovery.md:L154| unit=I-0): [E][I-2][dependency] I-2 composes the BSP, E-15, C-11, and C-23 substrates in Wave 2. Anchor: line 268.
- **I-002/I002-IDX-063** [SRC] (DEEP-I-002.discovery.md:L155| unit=I-0): [E][I-3][performance] I-3 hardens SSB pragmas and measures actual insert throughput. Anchors: lines 270-271.
- **I-002/I002-IDX-064** [SRC] (DEEP-I-002.discovery.md:L156| unit=I-0): [E][I-3][durability] Checkpoints use `.tmp`/`fsync`/`rename` atomic writes. Anchors: lines 270-271.
- **I-002/I002-IDX-065** [SRC] (DEEP-I-002.discovery.md:L157| unit=I-0): [E][I-3][flags] `io_uring` is Linux-gated rather than a universal path. Anchor: line 271.
- **I-002/I002-IDX-066** [SRC] (DEEP-I-002.discovery.md:L158| unit=I-0): [E][I-3][nature] I-3 is a small Wave-1 extension of SSB and H-44 durability. Anchor: line 270.
- **I-002/I002-IDX-067** [SRC] (DEEP-I-002.discovery.md:L159| unit=I-0): [E][I-4][workflow] I-4 progresses from receipt-ranked best-of-N to tournament to PUCT with virtual loss. Anchors: lines 273-275.
- **I-002/I002-IDX-068** [SRC] (DEEP-I-002.discovery.md:L160| unit=I-0): [E][I-4][dependency] It depends on I-2 isolation and I-17 evaluation. Anchor: line 275.
- **I-002/I002-IDX-069** [SRC] (DEEP-I-002.discovery.md:L161| unit=I-0): [E][I-4][status] I-4 is the Track-13 keystone and blocks I-22, I-28, and I-30. Anchor: line 274.
- **I-002/I002-IDX-070** [SRC] (DEEP-I-002.discovery.md:L162| unit=I-0): [E][I-4][integration] It composes SessionDag/H-49, D-3, and the supporting receipt stack rather than delegating selection to an LLM. Anchors: lines 274-275.
- **I-002/I002-IDX-071** [SRC] (DEEP-I-002.discovery.md:L163| unit=I-0): [E][I-5][context] I-5 hard-excludes failed siblings and raw tracebacks and retains a contrastive constraint. Anchors: lines 276-277.
- **I-002/I002-IDX-072** [SRC] (DEEP-I-002.discovery.md:L164| unit=I-0): [E][I-5][measurement] It A/B measures the vividness tax and is the ID-1 instrument. Anchor: line 277.
- **I-002/I002-IDX-073** [SRC] (DEEP-I-002.discovery.md:L165| unit=I-0): [E][I-5][dependency] It composes H-6, H-50, and the E-19 probe in Wave 2. Anchor: line 276.
- **I-002/I002-IDX-074** [SRC] (DEEP-I-002.discovery.md:L166| unit=I-0): [E][I-6][context] I-6 adds a privileged final context slot containing TaskSpec and active constraints. Anchors: lines 278-279.
- **I-002/I002-IDX-075** [SRC] (DEEP-I-002.discovery.md:L167| unit=I-0): [E][I-6][workflow] The slot appears after the last cache breakpoint and requires an acknowledgement tag. Anchor: line 279.
- **I-002/I002-IDX-076** [SRC] (DEEP-I-002.discovery.md:L168| unit=I-0): [E][I-6][dependency] It composes the existing ContextSlot substrate in Wave 2. Anchor: line 278.
- **I-002/I002-IDX-077** [SRC] (DEEP-I-002.discovery.md:L169| unit=I-0): [E][I-7][routing] I-7 uses a local-first cascade and escalates only after condensation through I-11. Anchors: lines 280-281.
- **I-002/I002-IDX-078** [SRC] (DEEP-I-002.discovery.md:L170| unit=I-0): [E][I-7][constraint] The Batch API is confined to offline lanes. Anchor: line 281.
- **I-002/I002-IDX-079** [SRC] (DEEP-I-002.discovery.md:L171| unit=I-0): [E][I-7][dependency] I-7 depends on I-11 and composes H-5/H-7 in Wave 4. Anchors: lines 280-281.
- **I-002/I002-IDX-080** [CON] (DEEP-I-002.discovery.md:L172| unit=I-0): [E][I-7][C] Its cascade embodies IC-1's rejection of interactive batch racing. Anchor: line 281.
- **I-002/I002-IDX-081** [SRC] (DEEP-I-002.discovery.md:L173| unit=I-0): [E][I-8][action] I-8 adds an `acknowledge_infeasible` tool. Anchors: lines 282-283.
- **I-002/I002-IDX-082** [SRC] (DEEP-I-002.discovery.md:L174| unit=I-0): [E][I-8][state] It detects dead ends through AST hashing. Anchor: line 283.
- **I-002/I002-IDX-083** [SRC] (DEEP-I-002.discovery.md:L175| unit=I-0): [E][I-8][workflow] It applies an empirical halting bound and credits abstention under RD-1. Anchor: line 283.
- **I-002/I002-IDX-084** [SRC] (DEEP-I-002.discovery.md:L176| unit=I-0): [E][I-8][dependency] It composes D-16/D-17, H-39, and E-35 in Wave 1. Anchor: line 282.
- **I-002/I002-IDX-085** [SRC] (DEEP-I-002.discovery.md:L177| unit=I-0): [E][I-9][measurement] I-9 measures `alpha_t` with injected corruption and recovery rate. Anchors: lines 284-285.
- **I-002/I002-IDX-086** [SRC] (DEEP-I-002.discovery.md:L178| unit=I-0): [E][I-9][control] It actively anneals when `alpha > 1`. Anchor: line 285.
- **I-002/I002-IDX-087** [SRC] (DEEP-I-002.discovery.md:L179| unit=I-0): [E][I-9][constraint] Local baselines precede any external threshold under ID-5. Anchor: line 285.
- **I-002/I002-IDX-088** [SRC] (DEEP-I-002.discovery.md:L180| unit=I-0): [E][I-10][cache] I-10 freezes prefixes, including tool-array handling, and sets explicit one-hour/five-minute TTL tiers. Anchors: lines 287-290.
- **I-002/I002-IDX-089** [SRC] (DEEP-I-002.discovery.md:L181| unit=I-0): [E][I-10][planning] A cache-economics planner uses journaled inter-call-gap statistics. Anchors: lines 289-290.
- **I-002/I002-IDX-090** [SRC] (DEEP-I-002.discovery.md:L182| unit=I-0): [E][I-10][failure] Unexplained misses over 10% halt execution and dump a blake3 prefix diff. Anchor: line 290.
- **I-002/I002-IDX-091** [SRC] (DEEP-I-002.discovery.md:L183| unit=I-0): [E][I-10][dependency] This Track-14 keystone builds on H-1/H-2/H-4/H-9 and E-39. Anchor: line 288.
- **I-002/I002-IDX-092** [SRC] (DEEP-I-002.discovery.md:L184| unit=I-0): [E][I-11][schema] I-11 defines a typed, versioned `BeliefState` for cross-model and cross-tier handoffs. Anchors: lines 291-293.
- **I-002/I002-IDX-093** [SRC] (DEEP-I-002.discovery.md:L185| unit=I-0): [E][I-11][measurement] It periodically compares full-history and BeliefState reruns and journals the delta. Anchors: lines 292-293.
- **I-002/I002-IDX-094** [SRC] (DEEP-I-002.discovery.md:L186| unit=I-0): [E][I-11][observability] BeliefState handoffs are ID-3 receipt-bearing middleware transforms. Anchor: line 293.
- **I-002/I002-IDX-095** [SRC] (DEEP-I-002.discovery.md:L187| unit=I-0): [E][I-11][dependency] It composes H-7, H-51, DKG, and I-7 in Wave 2. Anchor: line 291.
- **I-002/I002-IDX-096** [SRC] (DEEP-I-002.discovery.md:L188| unit=I-0): [E][I-12][lifecycle] I-12 promotes playbook content only after at least `k` SSB-verified outcomes and measured lift. Anchors: lines 294-296.
- **I-002/I002-IDX-097** [SRC] (DEEP-I-002.discovery.md:L189| unit=I-0): [E][I-12][lifecycle] The same ledger can demote entries. Anchor: line 296.
- **I-002/I002-IDX-098** [SRC] (DEEP-I-002.discovery.md:L190| unit=I-0): [E][I-12][validation] DSPy-style validation runs against the C-28 suite. Anchor: line 296.
- **I-002/I002-IDX-099** [SRC] (DEEP-I-002.discovery.md:L191| unit=I-0 SURFACE): [E][I-12][surface] Curated content exports to `CLAUDE.md`, `AGENTS.md`, and skills. Anchor: line 296.
- **I-002/I002-IDX-100** [SRC] (DEEP-I-002.discovery.md:L192| unit=I-0): [E][I-12][security] Learned playbook entries are governed by ID-6 promotion discipline. Anchor: line 296.
- **I-002/I002-IDX-101** [SRC] (DEEP-I-002.discovery.md:L193| unit=I-0): [E][I-13][security] I-13 performs a deterministic scan at memory-write time. Anchors: lines 297-299.
- **I-002/I002-IDX-102** [SRC] (DEEP-I-002.discovery.md:L194| unit=I-0): [E][I-13][schema] It splits `to_llm_content` from `to_ui_dict`. Anchor: line 298.
- **I-002/I002-IDX-103** [SRC] (DEEP-I-002.discovery.md:L195| unit=I-0): [E][I-13][capability] It expands `TraceQuery`. Anchor: line 298.
- **I-002/I002-IDX-104** [AC] (DEEP-I-002.discovery.md:L196| unit=I-0): [E][I-13][dependency][A] YARA is an optional new-dependency decision; regex/Aho-Corasick is the first implementation. Anchor: line 299.
- **I-002/I002-IDX-105** [SRC] (DEEP-I-002.discovery.md:L197| unit=I-0): [E][I-14][workflow] I-14 applies LSP slicing and tree-sitter AST folding before any learned compression. Anchors: lines 300-302.
- **I-002/I002-IDX-106** [SRC] (DEEP-I-002.discovery.md:L198| unit=I-0): [E][I-14][deferred] LLMLingua-2 is a gated Tier-2 path. Anchor: line 301.
- **I-002/I002-IDX-107** [SRC] (DEEP-I-002.discovery.md:L199| unit=I-0): [E][I-14][non-goal] N-gram ciphers are banned. Anchor: line 302.
- **I-002/I002-IDX-108** [SRC] (DEEP-I-002.discovery.md:L200| unit=I-0): [E][I-14][lifecycle] Every compression is journaled and reversible under ID-3. Anchor: line 302.
- **I-002/I002-IDX-109** [SRC] (DEEP-I-002.discovery.md:L201| unit=I-0): [E][I-15][cache] I-15 caches durable intermediate DKG/research summaries. Anchors: lines 303-304.
- **I-002/I002-IDX-110** [SRC] (DEEP-I-002.discovery.md:L202| unit=I-0): [E][I-15][performance] An exact-hash O(1) lookup precedes vector search. Anchor: line 304.
- **I-002/I002-IDX-111** [SRC] (DEEP-I-002.discovery.md:L203| unit=I-0): [E][I-16][evaluation] I-16 evaluates sink-protected SnapKV/ThinK and Int4 KV on the sidecar. Anchors: lines 305-307.
- **I-002/I002-IDX-112** [SRC] (DEEP-I-002.discovery.md:L204| unit=I-0): [E][I-16][flags] Evaluation is RunPod-gated. Anchor: line 306.
- **I-002/I002-IDX-113** [CON] (DEEP-I-002.discovery.md:L205| unit=I-0): [E][I-16][C][constraint] MoE-sidecar retargeting is an evaluation, never an assumed swap. Anchors: lines 306-307.
- **I-002/I002-IDX-114** [SRC] (DEEP-I-002.discovery.md:L206| unit=I-0): [E][I-16][measurement] ID-5 requires local evidence before adoption claims. Anchor: line 307.
- **I-002/I002-IDX-115** [SRC] (DEEP-I-002.discovery.md:L207| unit=I-0): [E][I-17][workflow] I-17 applies a supreme Layer-1 hard veto, a calibrated Layer-2 score to survivors, and Layer-3 adversarial escalation on disagreement. Anchors: lines 309-312.
- **I-002/I002-IDX-116** [SRC] (DEEP-I-002.discovery.md:L208| unit=I-0): [E][I-17][failure] It uses an empirical 15-second halting bound. Anchor: line 311.
- **I-002/I002-IDX-117** [SRC] (DEEP-I-002.discovery.md:L209| unit=I-0): [E][I-17][recovery] Flakes enter quarantine with ledger evidence. Anchors: lines 311-312.
- **I-002/I002-IDX-118** [SRC] (DEEP-I-002.discovery.md:L210| unit=I-0): [E][I-17][deferred] Cross-language verifier packs are a Tier-2 seam. Anchor: line 312.
- **I-002/I002-IDX-119** [SRC] (DEEP-I-002.discovery.md:L211| unit=I-0): [E][I-17][dependency] I-17 is blocking for I-4. Anchor: line 310.
- **I-002/I002-IDX-120** [SRC] (DEEP-I-002.discovery.md:L212| unit=I-0): [E][I-18][integrity] I-18 adds a per-row `chain_prev_hash` and a five-layer stack hash. Anchors: lines 313-315.
- **I-002/I002-IDX-121** [SRC] (DEEP-I-002.discovery.md:L213| unit=I-0): [E][I-18][integration] Rekor transparency publishing is opt-in. Anchor: line 314.
- **I-002/I002-IDX-122** [SRC] (DEEP-I-002.discovery.md:L214| unit=I-0): [E][I-18][compliance] It emits an AI-BOM/AISVS AC.8 artifact. Anchors: lines 314-315.
- **I-002/I002-IDX-123** [SRC] (DEEP-I-002.discovery.md:L215| unit=I-0): [E][I-18][fallback] Offline blake3-Merkle sealing is the default. Anchor: line 315.
- **I-002/I002-IDX-124** [SD] (DEEP-I-002.discovery.md:L216| unit=I-0): [E][I-19][security] I-19 blocks source edits that lack a symbol-level test delta at PreToolUse. Anchors: lines 316-318.
- **I-002/I002-IDX-125** [SRC] (DEEP-I-002.discovery.md:L217| unit=I-0): [E][I-19][validation] It performs a syntax-only dry run. Anchor: line 317.
- **I-002/I002-IDX-126** [SD] (DEEP-I-002.discovery.md:L218| unit=I-0): [E][I-19][exception] Refactors may use a coverage-delta exemption. Anchors: lines 317-318.
- **I-002/I002-IDX-127** [SRC] (DEEP-I-002.discovery.md:L219| unit=I-0): [E][I-19][rollout] False-positive rate must be bounded before default-on. Anchor: line 318.
- **I-002/I002-IDX-128** [SRC] (DEEP-I-002.discovery.md:L220| unit=I-0): [E][I-20][selection] I-20 selects mutants inside the diff cone using reachability. Anchors: lines 319-322.
- **I-002/I002-IDX-129** [SRC] (DEEP-I-002.discovery.md:L221| unit=I-0): [E][I-20][budget] Mutation is time-boxed and sampled with a confidence interval and early stop. Anchors: lines 320-321.
- **I-002/I002-IDX-130** [SRC] (DEEP-I-002.discovery.md:L222| unit=I-0): [E][I-20][cache] Results are cached by `(fn-hash, test-hash)`. Anchor: line 321.
- **I-002/I002-IDX-131** [SRC] (DEEP-I-002.discovery.md:L223| unit=I-0): [E][I-20][recovery] Surviving mutants become structured regeneration Directives. Anchors: lines 321-322.
- **I-002/I002-IDX-132** [SRC] (DEEP-I-002.discovery.md:L224| unit=I-0): [E][I-20][integration] PBT relation adapters sit behind the D-3 trait. Anchor: line 322.
- **I-002/I002-IDX-133** [SRC] (DEEP-I-002.discovery.md:L225| unit=I-0): [E][I-20][status] I-20 is the Track-15 keystone and a Wave-3 extension. Anchor: line 319.
- **I-002/I002-IDX-134** [SRC] (DEEP-I-002.discovery.md:L226| unit=I-0): [E][I-21][security] I-21 gives generator and evaluator zero shared context. Anchors: lines 323-325.
- **I-002/I002-IDX-135** [SRC] (DEEP-I-002.discovery.md:L227| unit=I-0): [E][I-21][workflow] Sprint contracts require structured approval. Anchor: line 324.
- **I-002/I002-IDX-136** [SRC] (DEEP-I-002.discovery.md:L228| unit=I-0): [E][I-21][retry] Trace reinjection retries are bounded. Anchors: lines 324-325.
- **I-002/I002-IDX-137** [SRC] (DEEP-I-002.discovery.md:L229| unit=I-0): [E][I-21][fallback] A third loop escalates to a cross-family adversary. Anchor: line 325.
- **I-002/I002-IDX-138** [SD] (DEEP-I-002.discovery.md:L230| unit=I-0): [E][I-22][scoring] I-22 scores delta progress. Anchors: lines 326-327.
- **I-002/I002-IDX-139** [SRC] (DEEP-I-002.discovery.md:L231| unit=I-0): [E][I-22][failure] It truncates at the first error and decays reward on no-progress loops. Anchor: line 327.
- **I-002/I002-IDX-140** [SRC] (DEEP-I-002.discovery.md:L232| unit=I-0): [E][I-22][dependency] I-22 depends on I-4 and follows ID-4. Anchors: lines 326-327.
- **I-002/I002-IDX-141** [SRC] (DEEP-I-002.discovery.md:L233| unit=I-0): [E][I-23][architecture] I-23 unifies calibration in one implementation. Anchors: lines 328-329.
- **I-002/I002-IDX-142** [SRC] (DEEP-I-002.discovery.md:L234| unit=I-0): [E][I-23][integration] Refusal, `claim_score`, and search prior consume that implementation. Anchor: line 329.
- **I-002/I002-IDX-143** [SRC] (DEEP-I-002.discovery.md:L235| unit=I-0): [E][I-23][workflow] Calibrated uncertainty can fork execution. Anchor: line 329.
- **I-002/I002-IDX-144** [SRC] (DEEP-I-002.discovery.md:L236| unit=I-0): [E][I-24][security] I-24 injects AST-preserving obfuscation canaries during scoring. Anchors: lines 330-332.
- **I-002/I002-IDX-145** [SRC] (DEEP-I-002.discovery.md:L237| unit=I-0): [E][I-24][failure] It detects style bias and neutralizes the scorer's weight. Anchors: lines 331-332.
- **I-002/I002-IDX-146** [SRC] (DEEP-I-002.discovery.md:L238| unit=I-0): [E][I-24][dependency] It composes H-44's detector rather than restating it. Anchor: line 332.
- **I-002/I002-IDX-147** [SRC] (DEEP-I-002.discovery.md:L239| unit=I-0): [E][I-25][learning] I-25 trains a difficulty/router classifier on ELAI's own `(task-features, lane, cost, verifier outcome)` evidence. Anchors: lines 334-337.
- **I-002/I002-IDX-148** [SRC] (DEEP-I-002.discovery.md:L240| unit=I-0): [E][I-25][routing] Execution-cost profiling pre-routes known-expensive task shapes. Anchors: lines 336-337.
- **I-002/I002-IDX-149** [SRC] (DEEP-I-002.discovery.md:L241| unit=I-0): [E][I-25][security] Learned routing is governed by ID-5 local baselines and ID-6 promotion discipline. Anchor: line 337.
- **I-002/I002-IDX-150** [SRC] (DEEP-I-002.discovery.md:L242| unit=I-0): [E][I-26][budget] I-26 scales thinking budgets with serial depth. Anchors: lines 338-340.
- **I-002/I002-IDX-151** [SRC] (DEEP-I-002.discovery.md:L243| unit=I-0): [E][I-26][failure] It applies an 85% collapse threshold and a plateau early stop. Anchor: line 339.
- **I-002/I002-IDX-152** [SRC] (DEEP-I-002.discovery.md:L244| unit=I-0): [E][I-26][integration] Cloud lanes map budgets to provider effort parameters. Anchors: lines 339-340.
- **I-002/I002-IDX-153** [SRC] (DEEP-I-002.discovery.md:L245| unit=I-0): [E][I-26][compatibility] Opus 4.7+ rejecting manual `budget_tokens` is a binding provider constraint. Anchor: line 340.
- **I-002/I002-IDX-154** [SRC] (DEEP-I-002.discovery.md:L246| unit=I-0): [E][I-27][measurement] I-27 plots verified-outcome rate against compute. Anchors: lines 341-343.
- **I-002/I002-IDX-155** [SRC] (DEEP-I-002.discovery.md:L247| unit=I-0 SURFACE): [E][I-27][UI] It exposes a Pareto cost/accuracy cockpit tile. Anchor: line 342.
- **I-002/I002-IDX-156** [SRC] (DEEP-I-002.discovery.md:L248| unit=I-0): [E][I-27][constraint] Anytime monotonicity is measured, never asserted. Anchors: lines 342-343.
- **I-002/I002-IDX-157** [SRC] (DEEP-I-002.discovery.md:L249| unit=I-0): [E][I-28][credit] I-28 assigns backward causal credit from COMPLETE. Anchors: lines 344-346.
- **I-002/I002-IDX-158** [SRC] (DEEP-I-002.discovery.md:L250| unit=I-0): [E][I-28][dataset] It extracts a DPO pair from a winner and the highest-scoring failed branch. Anchors: lines 345-346.
- **I-002/I002-IDX-159** [SRC] (DEEP-I-002.discovery.md:L251| unit=I-0): [E][I-28][retrieval] It demotes retrieval chunks associated with failure. Anchor: line 346.
- **I-002/I002-IDX-160** [SRC] (DEEP-I-002.discovery.md:L252| unit=I-0): [E][I-28][non-goal] Shapley attribution is declined as research-grade; I-28 depends on I-4. Anchors: lines 344-346.
- **I-002/I002-IDX-161** [SRC] (DEEP-I-002.discovery.md:L253| unit=I-0): [E][I-29][generation] I-29 makes a Mercury-class dLLM an untrusted draft generator behind the full veto stack. Anchors: lines 347-349.
- **I-002/I002-IDX-162** [SRC] (DEEP-I-002.discovery.md:L254| unit=I-0): [E][I-29][concurrency] CRDT semantics merge multi-agent drafts. Anchors: lines 348-349.
- **I-002/I002-IDX-163** [SRC] (DEEP-I-002.discovery.md:L255| unit=I-0): [E][I-29][permissions] New dependency and network access require user approval. Anchor: line 349.
- **I-002/I002-IDX-164** [CON] (DEEP-I-002.discovery.md:L256| unit=I-0): [E][I-29][C][scope] Full TTD-DR is re-scoped to the narrow draft lane; I-29 is L effort and must be full-form-expanded. Anchors: lines 347-349, 167-171.
- **I-002/I002-IDX-165** [SRC] (DEEP-I-002.discovery.md:L257| unit=I-0): [E][I-30][dataset] I-30 extracts winning paths from sealed RRAs into SFT JSONL. Anchors: lines 351-354.
- **I-002/I002-IDX-166** [SRC] (DEEP-I-002.discovery.md:L258| unit=I-0): [E][I-30][dataset] It pairs fluent-but-vetoed outputs as negatives. Anchors: lines 353-354.
- **I-002/I002-IDX-167** [SRC] (DEEP-I-002.discovery.md:L259| unit=I-0): [E][I-30][rollout] Rejection-sampling LoRA SFT ships before GRPO. Anchor: line 354.
- **I-002/I002-IDX-168** [SRC] (DEEP-I-002.discovery.md:L260| unit=I-0): [E][I-30][security] Promotion uses ID-6 ledger gates and depends on I-4. Anchors: lines 352-354.
- **I-002/I002-IDX-169** [SRC] (DEEP-I-002.discovery.md:L261| unit=I-0): [E][I-31][learning] I-31 performs GRPO from a typed endpoint serving execution-grounded rewards. Anchors: lines 355-358.
- **I-002/I002-IDX-170** [SRC] (DEEP-I-002.discovery.md:L262| unit=I-0): [E][I-31][security] Blocking prerequisites include I-24 canaries, H-28 reward-tampering defense, and H-17/H-18 ledger controls. Anchors: lines 355-358.
- **I-002/I002-IDX-171** [SRC] (DEEP-I-002.discovery.md:L263| unit=I-0): [E][I-31][constraint] KL penalty is mandatory and the evaluator remains outside the write boundary. Anchors: lines 356-358.
- **I-002/I002-IDX-172** [SRC] (DEEP-I-002.discovery.md:L264| unit=I-0): [E][I-31][flags] The capability stays inside the existing `darwinian-evolve` kill-switched lane. Anchor: line 358.
- **I-002/I002-IDX-173** [SRC] (DEEP-I-002.discovery.md:L265| unit=I-0): [E][I-31][status] I-31 is explicitly last. Anchor: line 355.
- **I-002/I002-IDX-174** [SRC] (DEEP-I-002.discovery.md:L266| unit=I-0): [E][I-32][roles] I-32 removes personas from factual/code roles. Anchors: lines 360-363.
- **I-002/I002-IDX-175** [SRC] (DEEP-I-002.discovery.md:L267| unit=I-0): [E][I-32][roles] It retains personas for alignment-dependent auditor-style and anti-spiral roles. Anchors: lines 362-363.
- **I-002/I002-IDX-176** [SRC] (DEEP-I-002.discovery.md:L268| unit=I-0): [E][I-32][schema] Roles become schema-first contracts. Anchor: line 363.
- **I-002/I002-IDX-177** [SRC] (DEEP-I-002.discovery.md:L269| unit=I-0): [E][I-32][measurement] Each role receives a measured A/B. Anchor: line 363.
- **I-002/I002-IDX-178** [SRC] (DEEP-I-002.discovery.md:L270| unit=I-0): [E][I-33][governance] I-33 externalizes candidates for decision-type LLM calls with CEFL. Anchors: lines 364-366.
- **I-002/I002-IDX-179** [SRC] (DEEP-I-002.discovery.md:L271| unit=I-0): [E][I-33][validation] I6Q validates rationale quality. Anchor: line 365.
- **I-002/I002-IDX-180** [SRC] (DEEP-I-002.discovery.md:L272| unit=I-0): [E][I-33][integrity] Commit-reveal protects entropy integrity on stochastic gate choices. Anchors: lines 365-366.
- **I-002/I002-IDX-181** [SRC] (DEEP-I-002.discovery.md:L273| unit=I-0): [E][I-33][observability] CDL, DIU, and FSR metrics are ledgered. Anchor: line 366.
- **I-002/I002-IDX-182** [SRC] (DEEP-I-002.discovery.md:L274| unit=I-0): [E][I-34][generation] I-34 applies CRANE thinking-token grammar augmentation on local lanes. Anchors: lines 367-369.
- **I-002/I002-IDX-183** [SRC] (DEEP-I-002.discovery.md:L275| unit=I-0): [E][I-34][security] Grammars are blake3-pinned control-plane artifacts. Anchor: line 368.
- **I-002/I002-IDX-184** [SRC] (DEEP-I-002.discovery.md:L276| unit=I-0): [E][I-34][validation] Outputs undergo post-parse schema revalidation. Anchors: lines 368-369.
- **I-002/I002-IDX-185** [SRC] (DEEP-I-002.discovery.md:L277| unit=I-0): [E][I-34][failure] The revalidation backstops silently noncompliant constrained-decoding output. Anchor: line 369.
- **I-002/I002-IDX-186** [SRC] (DEEP-I-002.discovery.md:L278| unit=I-0): [E][I-35][workflow] I-35 defines trigger/predicate/enforcement rules over tool-call history. Anchors: lines 370-372.
- **I-002/I002-IDX-187** [SRC] (DEEP-I-002.discovery.md:L279| unit=I-0): [E][I-35][security] Enforcement runs at the I-1 supervisor plane. Anchor: line 371.
- **I-002/I002-IDX-188** [SRC] (DEEP-I-002.discovery.md:L280| unit=I-0): [E][I-35][capability] It supports deterministic “X-before-Y” sequence classes that are CDA-immune. Anchors: lines 371-372.
- **I-002/I002-IDX-189** [SRC] (DEEP-I-002.discovery.md:L281| unit=I-0): [E][I-35][deferred] Pro2Guard DTMC monitoring is Tier 2 and deferred pending evidence. Anchor: line 372.
- **I-002/I002-IDX-190** [SRC] (DEEP-I-002.discovery.md:L282| unit=I-0): [E][I-36][artifact] I-36 compiles per-backend prompt artifacts. Anchors: lines 373-375.
- **I-002/I002-IDX-191** [SRC] (DEEP-I-002.discovery.md:L283| unit=I-0): [E][I-36][measurement] It measures transfer gaps from the ledger. Anchor: line 374.
- **I-002/I002-IDX-192** [SRC] (DEEP-I-002.discovery.md:L284| unit=I-0): [E][I-36][constraint] Deliberative lanes receive no explicit chain-of-thought instruction under extended thinking. Anchors: lines 374-375.
- **I-002/I002-IDX-193** [SRC] (DEEP-I-002.discovery.md:L285| unit=I-0): [E][I-36][compatibility] Backend temperature constraints remain binding; outputs follow ID-3 receipts and ID-6 promotion discipline. Anchor: line 375.
- **I-002/I002-IDX-194** [SRC] (DEEP-I-002.discovery.md:L286| unit=I-0): [E][I-37][schema] I-37 creates EARS-typed requirements with content hashes. Anchors: lines 376-378.
- **I-002/I002-IDX-195** [SRC] (DEEP-I-002.discovery.md:L287| unit=I-0): [E][I-37][traceability] Each requirement binds tests and mutation coverage. Anchors: lines 377-378.
- **I-002/I002-IDX-196** [SRC] (DEEP-I-002.discovery.md:L288| unit=I-0 SURFACE): [E][I-37][UI] The cockpit exposes per-requirement verified/unverified state. Anchors: lines 377-378.
- **I-002/I002-IDX-197** [SRC] (DEEP-I-002.discovery.md:L289| unit=I-0): [E][I-37][integration] Spec-kit import interop is included. Anchor: line 378.
- **I-002/I002-IDX-198** [SD] (DEEP-I-002.discovery.md:L290| unit=I-0): [E][I-38][metric] I-38 defines QDELTA as defective diffs reaching review per 100 tasks. Anchors: lines 379-381.
- **I-002/I002-IDX-199** [SRC] (DEEP-I-002.discovery.md:L291| unit=I-0): [E][I-38][experiment] Gates-on and gates-off arms are compared with bootstrap confidence intervals. Anchors: lines 380-381.
- **I-002/I002-IDX-200** [SRC] (DEEP-I-002.discovery.md:L292| unit=I-0): [E][I-38][statistical-gate] The floor requires paired seeds of at least five, SEM, pass^k, and dollars/task. Anchor: line 381.
- **I-002/I002-IDX-201** [SRC] (DEEP-I-002.discovery.md:L293| unit=I-0): [E][I-38][compliance] Contamination discipline surfaces the D-20 probe; ID-5 binds the metric. Anchor: line 381.
- **I-002/I002-IDX-202** [SRC] (DEEP-I-002.discovery.md:L294| unit=I-0 SURFACE): [E][I-39][status] `elai plan status` derives plan state from the journal and wiring fragments. Anchors: lines 382-384.
- **I-002/I002-IDX-203** [SRC] (DEEP-I-002.discovery.md:L295| unit=I-0): [E][I-39][non-goal] Hand-maintained tallies are demoted to build artifacts. Anchor: line 383.
- **I-002/I002-IDX-204** [SRC] (DEEP-I-002.discovery.md:L296| unit=I-0): [E][I-39][compatibility-test] One migration-replay test runs an old `ssb.db` through current code to detect RRA rot. Anchors: lines 383-384.
- **I-002/I002-IDX-205** [SRC] (DEEP-I-002.discovery.md:L297| unit=I-0): [E][I-40][artifact] I-40 publishes `.rra` as an open specification. Anchors: lines 385-387.
- **I-002/I002-IDX-206** [SRC] (DEEP-I-002.discovery.md:L298| unit=I-0): [E][I-40][integration] A minimal proxy/MCP wrapper adds taint, write gates, and RRA evidence around external agents. Anchors: lines 386-387.
- **I-002/I002-IDX-207** [SRC] (DEEP-I-002.discovery.md:L299| unit=I-0): [E][I-40][product] The wrapper is the wedge/standard tier; the orchestrator remains premium. Anchor: line 387.
- **I-002/I002-IDX-208** [SRC] (DEEP-I-002.discovery.md:L300| unit=I-0): [E][I-40][status] I-40 is an L-effort, separately gated Wave-5 build. Anchors: lines 385-387, 530-533.
- **I-002/I002-IDX-209** [SRC] (DEEP-I-002.discovery.md:L301| unit=I-0): [E][I-41][capability] I-41 runs eight advisory critique lenses over prose/specification. Anchors: lines 389-392.
- **I-002/I002-IDX-210** [SRC] (DEEP-I-002.discovery.md:L302| unit=I-0): [E][I-41][evidence] Findings without anchors are dropped. Anchor: line 391.
- **I-002/I002-IDX-211** [SRC] (DEEP-I-002.discovery.md:L303| unit=I-0): [E][I-41][security] `advisory=true`; critique never yields accept/block. Anchors: lines 391-392.
- **I-002/I002-IDX-212** [SRC] (DEEP-I-002.discovery.md:L304| unit=I-0 SURFACE): [E][I-41][surface] It integrates with the B-40 idea-critic loop and `elai critique`. Anchor: line 392.
- **I-002/I002-IDX-213** [SRC] (DEEP-I-002.discovery.md:L305| unit=I-0): [E][I-42][schema] I-42 adds `RunProfile { Orchestrate, CritiqueOnly }`. Anchors: lines 393-395.
- **I-002/I002-IDX-214** [SRC] (DEEP-I-002.discovery.md:L306| unit=I-0): [E][I-42][permissions] CritiqueOnly structurally disables executor/fixer spawning while the operator writes. Anchors: lines 394-395.
- **I-002/I002-IDX-215** [SRC] (DEEP-I-002.discovery.md:L307| unit=I-0): [E][I-42][invariant] The profile must produce zero executor-attributed `file_write` rows. Anchor: line 395.
- **I-002/I002-IDX-216** [SRC] (DEEP-I-002.discovery.md:L308| unit=I-0): [E][I-42][architecture] Both profiles use one engine and one verification spine; I-42 depends on I-41. Anchors: lines 393-395.
- **I-002/I002-IDX-217** [SRC] (DEEP-I-002.discovery.md:L309| unit=I-0): [E][I-43][experiment] I-43 runs a reduced-N gates-on/off pilot. Anchors: lines 396-398.
- **I-002/I002-IDX-218** [SRC] (DEEP-I-002.discovery.md:L310| unit=I-0): [E][I-43][failure] The pilot detects the block-everything trap. Anchor: line 397.
- **I-002/I002-IDX-219** [SRC] (DEEP-I-002.discovery.md:L311| unit=I-0): [E][I-43][truthfulness] A cross-lane scoper limits reproducibility claims. Anchors: lines 397-398.
- **I-002/I002-IDX-220** [SRC] (DEEP-I-002.discovery.md:L312| unit=I-0): [E][I-43][rollout] A go/no-go memo precedes the G-6 XL build; fixture runs ship now and the live leg is gated. Anchor: line 398.
- **I-002/I002-IDX-221** [SRC] (DEEP-I-002.discovery.md:L313| unit=I-0): [E][I-44][schema] I-44 defines `RetentionPolicy { Full, ZeroAfterVerify }`. Anchors: lines 399-401.
- **I-002/I002-IDX-222** [SRC] (DEEP-I-002.discovery.md:L314| unit=I-0): [E][I-44][privacy] ZeroAfterVerify purges raw inputs after a sealed verdict but retains redacted evidence. Anchors: lines 400-401.
- **I-002/I002-IDX-223** [SRC] (DEEP-I-002.discovery.md:L315| unit=I-0): [E][I-44][recovery-test] `replay --verify` must still pass after purge. Anchor: line 401.
- **I-002/I002-IDX-224** [SRC] (DEEP-I-002.discovery.md:L316| unit=I-0): [E][I-44][flags] Zero retention is opt-in and the default remains unchanged under GD-4; I-44 depends on G-9. Anchors: lines 399-401.
- **I-002/I002-IDX-225** [SRC] (DEEP-I-002.discovery.md:L317| unit=I-0): [E][I-45][integrity] I-45 stamps `blake3(canonical objective)` into every sealed PASS. Anchors: lines 403-406.
- **I-002/I002-IDX-226** [SRC] (DEEP-I-002.discovery.md:L318| unit=I-0): [E][I-45][lifecycle] Mutating the goal after PASS makes the receipt stale on every read path. Anchors: lines 405-406.
- **I-002/I002-IDX-227** [SRC] (DEEP-I-002.discovery.md:L319| unit=I-0): [E][I-45][failure] Stale receipts hard-fail. Anchor: line 406.
- **I-002/I002-IDX-228** [SRC] (DEEP-I-002.discovery.md:L320| unit=I-0): [E][I-45][status] I-45 is an extension over shipped RRA/attestation substrate. Anchors: lines 404-406.
- **I-002/I002-IDX-229** [SRC] (DEEP-I-002.discovery.md:L321| unit=I-0): [E][I-46][action] I-46 refuses and redirects bare grep/cat/find to registered tools. Anchors: lines 407-409.
- **I-002/I002-IDX-230** [SRC] (DEEP-I-002.discovery.md:L322| unit=I-0): [E][I-46][fixup] It uses a deterministic fixup table and bounded allowed prefixes. Anchors: lines 408-409.
- **I-002/I002-IDX-231** [SRC] (DEEP-I-002.discovery.md:L323| unit=I-0): [E][I-46][security] Redirect substitutes are never auto-executed. Anchor: line 409.
- **I-002/I002-IDX-232** [SRC] (DEEP-I-002.discovery.md:L324| unit=I-0): [E][I-46][invariant] Interception narrows and never widens existing validation. Anchor: line 409.
- **I-002/I002-IDX-233** [SRC] (DEEP-I-002.discovery.md:L325| unit=I-0): [E][I-47][security] I-47 tags auditor inputs as untrusted and adds an anti-self-attestation frame. Anchors: lines 410-412.
- **I-002/I002-IDX-234** [SRC] (DEEP-I-002.discovery.md:L326| unit=I-0): [E][I-47][dispatch] A frozen per-role worker-discipline block is injected at dispatch. Anchors: lines 411-412.
- **I-002/I002-IDX-235** [SRC] (DEEP-I-002.discovery.md:L327| unit=I-0): [E][I-47][cache] The injected block is byte-stable under HD-2. Anchor: line 412.
- **I-002/I002-IDX-236** [SRC] (DEEP-I-002.discovery.md:L328| unit=I-0): [E][I-48][artifact] I-48 moves every dispatch prompt to reviewed `crates/*/prompts/*.md` files loaded by `include_str!`. Anchors: lines 413-415.
- **I-002/I002-IDX-237** [SRC] (DEEP-I-002.discovery.md:L329| unit=I-0): [E][I-48][compatibility] Extraction must preserve prompt bytes. Anchors: lines 414-415.
- **I-002/I002-IDX-238** [SRC] (DEEP-I-002.discovery.md:L330| unit=I-0): [E][I-48][test] A contract test bans inline prompt literals, enabling diffable and pinnable prompts. Anchor: line 415.
- **I-002/I002-IDX-239** [SRC] (DEEP-I-002.discovery.md:L331| unit=I-0): [E][I-49][integration] I-49 adds direct Bedrock SigV4+EventStream, Azure Responses, and OpenAI-to-Anthropic shim protocol lanes. Anchors: lines 416-418.
- **I-002/I002-IDX-240** [SRC] (DEEP-I-002.discovery.md:L332| unit=I-0): [E][I-49][rollout] Protocol logic and replay fixtures ship now; live calls are separate checkboxes. Anchors: lines 417-418.
- **I-002/I002-IDX-241** [SRC] (DEEP-I-002.discovery.md:L333| unit=I-0): [E][I-49][non-goal] It is the only provider steal; a wholesale port remains rejected. Anchor: line 418.
- **I-002/I002-IDX-242** [SRC] (DEEP-I-002.discovery.md:L334| unit=I-0): [E][I-49][status] I-49 is an L-effort Wave-4 build and therefore requires full-form expansion and source rereads. Anchors: lines 416-418, 66-68.
- **I-002/I002-IDX-243** [SRC] (DEEP-I-002.discovery.md:L335| unit=I-0): [E][I-50][architecture] I-50 defines an `AuthProvider` trait and broker generalizing Anthropic-only `oauth.rs`. Anchors: lines 419-421.
- **I-002/I002-IDX-244** [SRC] (DEEP-I-002.discovery.md:L336| unit=I-0): [E][I-50][compatibility] Existing Anthropic OAuth flow must remain byte-compatible. Anchors: lines 420-421.
- **I-002/I002-IDX-245** [SRC] (DEEP-I-002.discovery.md:L337| unit=I-0): [E][I-50][privacy] Token material is never journaled. Anchor: line 421.
- **I-002/I002-IDX-246** [SRC] (DEEP-I-002.discovery.md:L338| unit=I-0): [E][I-50][dependency] I-50 composes B-37 and I-49 in Wave 4. Anchors: lines 419-421.
- **I-002/I002-IDX-247** [SRC] (DEEP-I-002.discovery.md:L339| unit=I-0): [E][I-51][data] I-51 establishes one `models.toml` containing per-model context, pricing, thinking, and VRAM-tier data. Anchors: lines 422-424.
- **I-002/I002-IDX-248** [SRC] (DEEP-I-002.discovery.md:L340| unit=I-0): [E][I-51][compatibility] Existing modules become views while their APIs are preserved. Anchors: lines 423-424.
- **I-002/I002-IDX-249** [CON] (DEEP-I-002.discovery.md:L341| unit=I-0): [E][I-51][C][constraint] The catalog consolidates duplicated names but wraps and never deletes measured VRAM-tier logic. Anchor: line 424.
- **I-002/I002-IDX-250** [SRC] (DEEP-I-002.discovery.md:L342| unit=I-0): [E][I-51][integration] E-39/E-31 and later routing consume the catalog. Anchors: lines 422-424.
- **I-002/I002-IDX-251** [SRC] (DEEP-I-002.discovery.md:L343| unit=I-0): [E][I-52][tool] I-52 exposes save, rewind, and diff over the shipped E-25 checkpoint store. Anchors: lines 425-427.
- **I-002/I002-IDX-252** [SRC] (DEEP-I-002.discovery.md:L344| unit=I-0): [E][I-52][scope] Checkpoints are session-scoped. Anchor: line 426.
- **I-002/I002-IDX-253** [SRC] (DEEP-I-002.discovery.md:L345| unit=I-0): [E][I-52][failure] Rewind refuses divergent state. Anchor: line 426.
- **I-002/I002-IDX-254** [CON] (DEEP-I-002.discovery.md:L346| unit=I-0): [E][I-52][C][integration] If H-49 ships, I-52 coordinates with/reuses that store and never creates a second one. Anchors: lines 426-427.
- **I-002/I002-IDX-255** [SRC] (DEEP-I-002.discovery.md:L347| unit=I-0): [E][I-53][tool] I-53 discovers and gates Justfile, Makefile, and cargo-alias recipes. Anchors: lines 428-430.
- **I-002/I002-IDX-256** [SRC] (DEEP-I-002.discovery.md:L348| unit=I-0): [E][I-53][tool] It adds a read-only archive reader. Anchor: line 429.
- **I-002/I002-IDX-257** [SRC] (DEEP-I-002.discovery.md:L349| unit=I-0): [E][I-53][tool] It adds a capped JSON-tree explorer. Anchors: lines 429-430.
- **I-002/I002-IDX-258** [SRC] (DEEP-I-002.discovery.md:L350| unit=I-0): [E][I-53][non-goal] Cron, monitor, and background-job tools are declined because the supervisor/orchestrator owns scheduling. Anchor: line 430.
- **I-002/I002-IDX-259** [SRC] (DEEP-I-002.discovery.md:L351| unit=I-0): [E][I-53][dependency] Recipe execution composes bash validation, E-30, I-46, and the tool catalog. Anchors: lines 428-430.
- **I-002/I002-IDX-260** [SRC] (DEEP-I-002.discovery.md:L352| unit=I-0): [E][I-54][tool] I-54 performs tree-sitter-query structural search, preview, and apply using in-tree grammars. Anchors: lines 431-433.
- **I-002/I002-IDX-261** [SRC] (DEEP-I-002.discovery.md:L353| unit=I-0): [E][I-54][workflow] Preview must precede apply. Anchor: line 432.
- **I-002/I002-IDX-262** [SRC] (DEEP-I-002.discovery.md:L354| unit=I-0): [E][I-54][security] Apply traverses the same edit gates as ordinary Edit. Anchor: line 433.
- **I-002/I002-IDX-263** [SRC] (DEEP-I-002.discovery.md:L355| unit=I-0): [E][I-54][dependency] Any ast-grep dependency is banned. Anchor: line 433.
- **I-002/I002-IDX-264** [SRC] (DEEP-I-002.discovery.md:L356| unit=I-0): [E][I-54][scope] E-52 remains the separate semantic-edit sibling. Anchors: lines 431-433.
- **I-002/I002-IDX-265** [SRC] (DEEP-I-002.discovery.md:L362| unit=I-0): [E][schema][I-1] Add `DirectiveExecuted { Executed, Discarded, CompensationFired }`, baseline +1. Anchor: line 444.
- **I-002/I002-IDX-266** [CON] (DEEP-I-002.discovery.md:L363| unit=I-0): [E][schema][I-2] Add `BranchWorkspaceEvent { Spawned, Merged, ConflictResolved, EscapeBlocked }`, baseline +1. Anchor: line 445.
- **I-002/I002-IDX-267** [SRC] (DEEP-I-002.discovery.md:L364| unit=I-0): [E][schema][I-3] Add no action type; reuse checkpoint/memory-monitoring rows and put throughput in metrics TOML. Anchor: line 446.
- **I-002/I002-IDX-268** [SRC] (DEEP-I-002.discovery.md:L365| unit=I-0): [E][schema][I-4] Add `SearchNodeEvent { Expanded, Pruned, Promoted, BudgetCollapsed }`, baseline +1. Anchor: line 447.
- **I-002/I002-IDX-269** [SRC] (DEEP-I-002.discovery.md:L366| unit=I-0): [E][schema][I-5] Add `ContextPruned { SiblingExcluded, TracebackContrasted, VividnessProbe }`, baseline +1. Anchor: line 448.
- **I-002/I002-IDX-270** [SRC] (DEEP-I-002.discovery.md:L367| unit=I-0): [E][schema][I-6] Add no action type; reuse `ContextReduction`, because anchor presence is a prompt fact. Anchor: line 449.
- **I-002/I002-IDX-271** [SRC] (DEEP-I-002.discovery.md:L368| unit=I-0): [E][schema][I-7] Add `EscalationCascade { LocalWin, Condensed, Escalated, BatchRouted }`, baseline +1. Anchor: line 450.
- **I-002/I002-IDX-272** [SRC] (DEEP-I-002.discovery.md:L369| unit=I-0): [E][schema][I-8] Add `TrajectoryAborted { Voluntary, DeadEndHash, HaltingBound }`, baseline +1. Anchor: line 451.
- **I-002/I002-IDX-273** [SRC] (DEEP-I-002.discovery.md:L370| unit=I-0): [E][schema][I-9] Add flat beacon `ErrorAmplificationMeasured`, baseline +1. Anchor: line 452.
- **I-002/I002-IDX-274** [SRC] (DEEP-I-002.discovery.md:L371| unit=I-0): [E][schema][I-10] Add `CacheEconomicsEvent { TtlPlanned, HitRateGuardTripped, PrefixDiffDumped }`, baseline +1. Anchor: line 453.
- **I-002/I002-IDX-275** [SD] (DEEP-I-002.discovery.md:L372| unit=I-0): [E][schema][I-11] Add `BeliefStateEvent { Condensed, Handoff, HonestyDelta }`, baseline +1. Anchor: line 454.
- **I-002/I002-IDX-276** [SRC] (DEEP-I-002.discovery.md:L373| unit=I-0): [E][schema][I-12] Add `PlaybookEvent { Promoted, Demoted, ValidationRun, Exported }`, baseline +1. Anchor: line 455.
- **I-002/I002-IDX-277** [SRC] (DEEP-I-002.discovery.md:L374| unit=I-0): [E][schema][I-13] Add `MemoryGuardEvent { PoisonBlocked, DualSerialized, TraceQueryExpanded }`, baseline +1. Anchor: line 456.
- **I-002/I002-IDX-278** [SRC] (DEEP-I-002.discovery.md:L375| unit=I-0): [E][schema][I-14] Add ID-3 receipt `ContextTransformReceipt { LspSlice, AstFold, Tier2Compressed }`, baseline +1. Anchor: line 457.
- **I-002/I002-IDX-279** [AC] (DEEP-I-002.discovery.md:L376| unit=I-0): [A][schema][I-15] Prefer reusing `OracleCacheHit/Miss` with a `semantic_cache` discriminator; add `SemanticCacheEvent` only if the existing payload lacks source. Decide at edit. Anchors: lines 458-459.
- **I-002/I002-IDX-280** [SRC] (DEEP-I-002.discovery.md:L377| unit=I-0): [E][schema][I-16] Add flat beacon `KvCacheTuned`, baseline +1. Anchor: line 460.
- **I-002/I002-IDX-281** [SRC] (DEEP-I-002.discovery.md:L378| unit=I-0): [E][schema][I-17] Add `TriLayerVerdict { HardVeto, ScoredSurvivor, DisagreementEscalated, FlakeQuarantined }`, baseline +1. Anchor: line 461.
- **I-002/I002-IDX-282** [SRC] (DEEP-I-002.discovery.md:L379| unit=I-0): [E][schema][I-18] Add `RowChainSealed { ChainAdvanced, RekorPushed, AibomEmitted, ChainBroken }`, baseline +1. Anchor: line 462.
- **I-002/I002-IDX-283** [SD] (DEEP-I-002.discovery.md:L380| unit=I-0): [E][schema][I-19] Add `TddPreflightBlocked { MissingTestDelta, SyntaxDryRunFailed, RefactorExempted }`, baseline +1. Anchor: line 463.
- **I-002/I002-IDX-284** [SRC] (DEEP-I-002.discovery.md:L381| unit=I-0): [E][schema][I-20] Add `MutationBudgetedRun { CiCleared, CiFailed, SurvivorDirective, CacheHit }`, baseline +1. Anchor: line 464.
- **I-002/I002-IDX-285** [SRC] (DEEP-I-002.discovery.md:L382| unit=I-0): [E][schema][I-21] Add `SprintContractEvent { Proposed, Approved, Rejected, RalphReinjected, AdversaryEscalated }`, baseline +1. Anchor: line 465.
- **I-002/I002-IDX-286** [SD] (DEEP-I-002.discovery.md:L383| unit=I-0): [E][schema][I-22] Add `ProgressRewardScored { Delta, FirstErrorTruncated, DecayApplied }`, baseline +1. Anchor: line 466.
- **I-002/I002-IDX-287** [SRC] (DEEP-I-002.discovery.md:L384| unit=I-0): [E][schema][I-23] Add no action type; reuse `RefusalDecision` and `CalibrationUpdate`. Anchor: line 467.
- **I-002/I002-IDX-288** [SRC] (DEEP-I-002.discovery.md:L385| unit=I-0): [E][schema][I-24] Add `PrmCanaryEvent { CanaryInjected, StyleBiasDetected, ScorerNeutralized }`, baseline +1. Anchor: line 468.
- **I-002/I002-IDX-289** [SRC] (DEEP-I-002.discovery.md:L386| unit=I-0): [E][schema][I-25] Add no action type; emit the existing E-31 `RouteDecisionEvent`. Anchor: line 469.
- **I-002/I002-IDX-290** [SRC] (DEEP-I-002.discovery.md:L387| unit=I-0): [E][schema][I-26] Add no action type; reuse `ReasoningBudgetScaled` and extend its payload only if necessary. Anchor: line 470.
- **I-002/I002-IDX-291** [SRC] (DEEP-I-002.discovery.md:L388| unit=I-0): [E][schema][I-27] Add flat beacon `ConvergenceCurveMeasured`, baseline +1. Anchor: line 471.
- **I-002/I002-IDX-292** [SRC] (DEEP-I-002.discovery.md:L389| unit=I-0): [E][schema][I-28] Add `DagCreditAssigned { CreditWalk, DpoPairExtracted, ChunkDemoted }`, baseline +1. Anchor: line 472.
- **I-002/I002-IDX-293** [SRC] (DEEP-I-002.discovery.md:L390| unit=I-0): [E][schema][I-29] Add `DiffusionDraftEvent { DraftAccepted, DraftVetoed, CrdtMerged }`, baseline +1. Anchor: line 473.
- **I-002/I002-IDX-294** [SRC] (DEEP-I-002.discovery.md:L391| unit=I-0): [E][schema][I-30] Add `SftExhaustEvent { PathExtracted, NegativePaired, DatasetSealed }`, baseline +1. Anchor: line 474.
- **I-002/I002-IDX-295** [SRC] (DEEP-I-002.discovery.md:L392| unit=I-0): [E][schema][I-31] Add `GrpoRewardComputed { RewardServed, PromotionBlocked, PromotionLedgered }`, baseline +1. Anchor: line 475.
- **I-002/I002-IDX-296** [SRC] (DEEP-I-002.discovery.md:L393| unit=I-0): [E][schema][I-32] Add `RoleContractApplied { PersonaStripped, PersonaRetained, AbDelta }`, baseline +1. Anchor: line 476.
- **I-002/I002-IDX-297** [SRC] (DEEP-I-002.discovery.md:L394| unit=I-0): [E][schema][I-33] Add `GovernancePrimitiveFired { CeflExternalized, I6qRejected, CommitRevealSealed }`, baseline +1. Anchor: line 477.
- **I-002/I002-IDX-298** [SRC] (DEEP-I-002.discovery.md:L395| unit=I-0): [E][schema][I-34] Add `GrammarGateEvent { CranePassed, PinViolation, PostParseRejected }`, baseline +1. Anchor: line 478.
- **I-002/I002-IDX-299** [SRC] (DEEP-I-002.discovery.md:L396| unit=I-0): [E][schema][I-35] Add `ToolSequenceVerdict { Allowed, OrderViolation, HaltEnforced }`, baseline +1. Anchor: line 479.
- **I-002/I002-IDX-300** [SRC] (DEEP-I-002.discovery.md:L397| unit=I-0): [E][schema][I-36] Add `PromptCompiled { Compiled, TransferGapMeasured, Rolledback }`, baseline +1. Anchor: line 480.
- **I-002/I002-IDX-301** [SRC] (DEEP-I-002.discovery.md:L398| unit=I-0): [E][schema][I-37] Add `RequirementVerified { Satisfied, Unverified, Regressed }`, baseline +1. Anchor: line 481.
- **I-002/I002-IDX-302** [SD] (DEEP-I-002.discovery.md:L399| unit=I-0): [E][schema][I-38] Add flat beacon `QdeltaMeasured`, baseline +1. Anchor: line 482.
- **I-002/I002-IDX-303** [SRC] (DEEP-I-002.discovery.md:L400| unit=I-0): [E][schema][I-39] Add flat beacon `PlanStateDerived`, baseline +1; migration replay reuses existing rows. Anchor: line 483.
- **I-002/I002-IDX-304** [SRC] (DEEP-I-002.discovery.md:L401| unit=I-0): [E][schema][I-40] Add no action type for the spec; wrapper reuses taint-flow and run-artifact-emitted rows. Anchor: line 484.
- **I-002/I002-IDX-305** [SRC] (DEEP-I-002.discovery.md:L402| unit=I-0): [E][schema][I-41] Add advisory `CritiqueLensFinding { AdvisoryFinding, LensRun, DroppedNoAnchor }`, baseline +1. Anchor: line 485.
- **I-002/I002-IDX-306** [SRC] (DEEP-I-002.discovery.md:L403| unit=I-0): [E][schema][I-42] Add no action type; reuse `phase_transition` and prove CritiqueOnly from absent executor `file_write` rows. Anchors: lines 486-487.
- **I-002/I002-IDX-307** [SD] (DEEP-I-002.discovery.md:L404| unit=I-0): [E][schema][I-43] Add `ClaimScopeMeasured { PilotDelta, CrossLaneDivergence, ClaimScoped }`, baseline +1. Anchor: line 488.
- **I-002/I002-IDX-308** [SRC] (DEEP-I-002.discovery.md:L405| unit=I-0): [E][schema][I-44] Add `RetentionPurged { RawInputsPurged, EvidenceRetained, PurgeVerified }`, baseline +1. Anchor: line 489.
- **I-002/I002-IDX-309** [SRC] (DEEP-I-002.discovery.md:L406| unit=I-0): [E][schema][I-45] Add `ObjectiveSealed { Sealed, MutationDetected, ReceiptStale }`, baseline +1. Anchor: line 490.
- **I-002/I002-IDX-310** [SRC] (DEEP-I-002.discovery.md:L407| unit=I-0): [E][schema][I-46] Add `BashRedirected { Redirected, FixedUp, PrefixBlocked }`, baseline +1. Anchor: line 491.
- **I-002/I002-IDX-311** [SRC] (DEEP-I-002.discovery.md:L408| unit=I-0): [E][schema][I-47] Add `PromptHardeningApplied { InputsTagged, RulesInjected, SelfAttestationStripped }`, baseline +1. Anchor: line 492.
- **I-002/I002-IDX-312** [SRC] (DEEP-I-002.discovery.md:L409| unit=I-0): [E][schema][I-48] Add no action type; inspect existing journaled request bytes and later reuse I-36 `PromptCompiled` provenance. Anchors: lines 493-494.
- **I-002/I002-IDX-313** [SRC] (DEEP-I-002.discovery.md:L410| unit=I-0): [E][schema][I-49] Add no action type; reuse route/runtime rows with new backend IDs. Anchor: line 495.
- **I-002/I002-IDX-314** [SRC] (DEEP-I-002.discovery.md:L411| unit=I-0): [E][schema][I-50] Add `AuthBrokerEvent { TokenAcquired, Refreshed, ProviderLinked, RefreshFailed }`, baseline +1. Anchor: line 496.
- **I-002/I002-IDX-315** [SRC] (DEEP-I-002.discovery.md:L412| unit=I-0): [E][schema][I-51] Add no action type; existing cost/route rows carry catalog-derived values. Anchor: line 497.
- **I-002/I002-IDX-316** [AC] (DEEP-I-002.discovery.md:L413| unit=I-0): [A][schema][I-52] Add `CheckpointToolEvent { Saved, Rewound, Diffed, RefusedDiverged }`, baseline +1, unless H-49 ships first, in which case reuse `EditHistoryEvent`. Anchors: lines 498-499.
- **I-002/I002-IDX-317** [SRC] (DEEP-I-002.discovery.md:L414| unit=I-0): [E][schema][I-53] Add no action type; all tools use existing `tool_call` rows. Anchor: line 500.
- **I-002/I002-IDX-318** [SRC] (DEEP-I-002.discovery.md:L415| unit=I-0): [E][schema][I-54] Add `StructuralEditApplied { Matched, Previewed, Applied, RefusedStale, ParseFailed }`, baseline +1. Anchor: line 501.
- **I-002/I002-IDX-319** [SRC] (DEEP-I-002.discovery.md:L419| unit=I-0): [E][sequencing] Wave 1 is I-3/I-8/I-10/I-15/I-23/I-34/I-38/I-39/I-43/I-45/I-51: cheap, trust-reinforcing, instrument-first, and search-independent. Anchors: lines 505-513.
- **I-002/I002-IDX-320** [SRC] (DEEP-I-002.discovery.md:L420| unit=I-0): [E][sequencing] I-43 falsifies before the G-6/I-38 XL measurement build; I-45 is the cheapest live-RRA trust steal; I-51 feeds later routing. Anchors: lines 510-513.
- **I-002/I002-IDX-321** [SRC] (DEEP-I-002.discovery.md:L421| unit=I-0): [E][sequencing] Wave 2 is the isolation/context/governance/dispatch spine: I-1/I-2/I-5/I-6/I-11/I-13/I-14/I-32/I-33/I-35/I-41/I-46/I-47/I-48/I-53. Anchors: lines 514-519.
- **I-002/I002-IDX-322** [SRC] (DEEP-I-002.discovery.md:L422| unit=I-0): [E][sequencing] Wave 3 builds receipt-ranked verification and gated edit tools; I-4 cannot ship before I-17 and I-20, I-42 depends on I-41, I-44 depends on G-9, and I-52/I-54 reuse edit gates. Anchors: lines 520-525.
- **I-002/I002-IDX-323** [SRC] (DEEP-I-002.discovery.md:L423| unit=I-0): [E][sequencing] Wave 4 contains search, routing, measurement, and provider/auth reach; I-49/I-50 wait until router/catalog substrate exists. Anchors: lines 526-529.
- **I-002/I002-IDX-324** [SRC] (DEEP-I-002.discovery.md:L424| unit=I-0): [E][sequencing] Wave 5 contains separately gated frontier/flywheel/wedge work; distillation consumes Wave-4 trajectories, GRPO is last, and the wrapper waits for end-to-end evidence. Anchors: lines 530-533.
- **I-002/I002-IDX-325** [SRC] (DEEP-I-002.discovery.md:L425| unit=I-0): [E][implementation] Non-RFC hashing uses blake3 only; production code forbids `.unwrap()`/`.expect()` except tests/main/const-init. Anchors: lines 535-539.
- **I-002/I002-IDX-326** [SRC] (DEEP-I-002.discovery.md:L426| unit=I-0): [E][implementation] Production uses `tracing`, `thiserror`/`anyhow`, `tokio::sync::Mutex` in async, native async traits, and `FxHashMap`/`BTreeMap` over standard `HashMap`. Anchors: lines 538-540.
- **I-002/I002-IDX-327** [SRC] (DEEP-I-002.discovery.md:L427| unit=I-0): [E][determinism] Time and entropy come through `ClockProvider`/`EntropySource`; raw time/entropy is quarantined. Anchor: line 540.
- **I-002/I002-IDX-328** [SRC] (DEEP-I-002.discovery.md:L428| unit=I-0): [E][test] Scoped per-crate tests are for development; final validation is `cargo check --workspace`, with full suites run by the auditor. Anchors: lines 540-542.
- **I-002/I002-IDX-329** [SRC] (DEEP-I-002.discovery.md:L429| unit=I-0): [E][permissions][security] Every dependency must pass registry/repository/last-publish/license checks and receive user approval; known candidates are YARA, dLLM client, and RL stack. Anchors: lines 541-543.
- **I-002/I002-IDX-330** [SRC] (DEEP-I-002.discovery.md:L430| unit=I-0): [E][compatibility] Version-specific APIs require fresh web verification. Anchors: lines 543-544.
- **I-002/I002-IDX-331** [SRC] (DEEP-I-002.discovery.md:L431| unit=I-0): [E][permissions] Removal/simplification requires user confirmation; changes land one at a time. Anchor: line 544.
- **I-002/I002-IDX-332** [SRC] (DEEP-I-002.discovery.md:L432| unit=I-0): [E][artifact] Every session gets `workflow/reasoning/I-N-decisions.md`; commit cadence is one commit per shipped session, with maintainer voice, no AI trailers, and only when asked. Anchors: lines 544-546.
- **I-002/I002-IDX-333** [SRC] (DEEP-I-002.discovery.md:L433| unit=I-0): [E][cache][compatibility] Anthropic's default TTL changed from one hour to five minutes on 2026-03-06; TTL must always be explicit and long-TTL entries precede shorter ones. Anchors: lines 549-552.
- **I-002/I002-IDX-334** [SRC] (DEEP-I-002.discovery.md:L434| unit=I-0): [E][rate-limit] TPM reserves input plus `max_tokens`; schema-bound clamping prevents a 4096-token reservation for tiny scores. Anchors: lines 553-554.
- **I-002/I002-IDX-335** [SRC] (DEEP-I-002.discovery.md:L435| unit=I-0): [E][concurrency][rate-limit] A nominal 60 RPM limit is enforced sub-second, so parallel branch submissions must jitter/spread. Anchor: line 555.
- **I-002/I002-IDX-336** [AC] (DEEP-I-002.discovery.md:L436| unit=I-0): [E][economics][A] Five-minute writes default to 1.25x/breakeven one read and one-hour writes to 2x/breakeven about three reads, but pricing must be re-verified at edit. Anchors: lines 556-557.
- **I-002/I002-IDX-337** [SRC] (DEEP-I-002.discovery.md:L437| unit=I-0): [E][constraint] Batch work takes roughly 10-45 minutes with 24-hour SLA, cancellation does not refund queued work, and batch never runs interactively/search-loop. Anchors: lines 558-559.
- **I-002/I002-IDX-338** [SRC] (DEEP-I-002.discovery.md:L438| unit=I-0): [E][compatibility] Opus 4.7+ rejects manual `budget_tokens`; use effort and automatic interleaved thinking. Anchors: lines 560-561.
- **I-002/I002-IDX-339** [SRC] (DEEP-I-002.discovery.md:L439| unit=I-0): [E][validation] Constrained-decoding engines can silently violate hard schemas and lose coverage on complex schemas; post-parse revalidation is mandatory. Anchors: lines 562-564.
- **I-002/I002-IDX-340** [SRC] (DEEP-I-002.discovery.md:L440| unit=I-0 SURFACE): [E][security] Grammars are pinned control-plane artifacts; third-party grammars and untrusted enum-bearing schemas are rejected because constrained decoding is an attack surface. Anchors: lines 565-567.
- **I-002/I002-IDX-341** [SRC] (DEEP-I-002.discovery.md:L441| unit=I-0): [E][non-goal] Cipher/abbreviation substitution in code prompts is banned because it breaks BPE priors. Anchor: line 568.
- **I-002/I002-IDX-342** [SRC] (DEEP-I-002.discovery.md:L442| unit=I-0): [E][determinism][truthfulness] Temperature zero is not deterministic; reproducibility claims remain action-stream-scoped. Anchors: lines 569-570.
- **I-002/I002-IDX-343** [SD] (DEEP-I-002.discovery.md:L443| unit=I-0): [E][metric][non-goal] SWE-bench Verified is rejected as a headline; QDELTA is ablation-framed and never a leaderboard rank. Anchors: lines 571-572.
- **I-002/I002-IDX-344** [SRC] (DEEP-I-002.discovery.md:L444| unit=I-0): [E][routing] OPRO-style optimizers run on cloud models, though their target can be local. Anchors: lines 573-574.
- **I-002/I002-IDX-345** [SRC] (DEEP-I-002.discovery.md:L445| unit=I-0): [E][roles] Personas are harmful for factual/code accuracy and retained only for alignment-dependent work. Anchors: lines 575-576.
- **I-002/I002-IDX-346** [SRC] (DEEP-I-002.discovery.md:L446| unit=I-0 SURFACE): [E][product] Evidence/cockpit work targets review-time bottlenecks, not raw generation speed. Anchors: lines 577-578.
- **I-002/I002-IDX-347** [SD] (DEEP-I-002.discovery.md:L447| unit=I-0): [E][C][governance] All `IC-1..IC-18` corrections are binding; highlighted replacements include no batch racing, deterministic compression first, cipher ban, TTL primary, measured tool-array layout, laddered search, SFT-before-GRPO, evaluation-only MoE retargeting, trust-spine reuse, and corrected persona/grammar posture. Anchors: lines 580-588.
- **I-002/I002-IDX-348** [SRC] (DEEP-I-002.discovery.md:L448| unit=I-0): [E][status] The session log was empty at generation; the index does not claim any I session DONE. Anchors: lines 590-593.
- **I-002/I002-IDX-349** [SRC] (DEEP-I-002.discovery.md:L449| unit=I-0 SURFACE): [E][next-step] The prescribed first command checks I-39, then I-10 follows as the economics keystone. Anchors: lines 595-601. I-0 ambiguity/conflict register:
- **I-002/I002-IDX-350** [AC] (DEEP-I-002.discovery.md:L453| unit=I-0): [A][status] “Shipped” predecessors in detail prose are plan-time assertions unless re-grepped under W-7; the index itself does not establish their current 2026 status. Anchors: lines 43-74, 200.
- **I-002/I002-IDX-351** [AC] (DEEP-I-002.discovery.md:L454| unit=I-0): [A][schema] I-15 and I-52 retain conditional action-row decisions; the advertised net `~+41` is approximate and depends on I-52 reuse. Anchors: lines 458-459, 498-503.
- **I-002/I002-IDX-352** [CON] (DEEP-I-002.discovery.md:L455| unit=I-0): [C][scope] I-29 narrows full TTD-DR to an untrusted draft lane; I-49 narrows the provider steal to three otherwise-unreachable protocols; I-54 takes only the structural-edit idea and retains the ast-grep ban. Anchors: lines 167-182, 416-433.
- **I-002/I002-IDX-353** [IX] (DEEP-I-002.discovery.md:L456| unit=I-0): [I][interaction] Because W-7/W-9 force live rechecks and the session log is empty, all baseline and shipped-substrate claims are implementation inputs, not proof that the corresponding features currently ship. Anchors: lines 184-204, 438-441, 590-593.

#### I-002 / unit `I-5` (38 records)

- **I-002/I002-I05-001** [SRC] (DEEP-I-002.discovery.md:L462| unit=I-5): [E][premise] Failed-sibling tracebacks/raw errors are hypothesized to consume context and bias the next step; the claim is instrumented rather than imported from literature. Anchors: lines 5-9.
- **I-002/I002-I05-002** [SRC] (DEEP-I-002.discovery.md:L463| unit=I-5): [E][workflow] `PristinePathFilter::build_branch_context` rebuilds branch context from the parent chain at expansion time. Anchors: lines 12-17, 61-63.
- **I-002/I002-I05-003** [SRC] (DEEP-I-002.discovery.md:L464| unit=I-5): [E][context] Failed sibling payloads are hard-excluded. Anchors: lines 13-18, 39-42.
- **I-002/I002-I05-004** [SRC] (DEEP-I-002.discovery.md:L465| unit=I-5): [E][context] Raw tracebacks are hard-excluded. Anchors: lines 13-18, 73-75.
- **I-002/I002-I05-005** [SRC] (DEEP-I-002.discovery.md:L466| unit=I-5): [E][schema] The only permitted negative signal is a structured contrastive constraint of at most 30 tokens, exemplified as attempt/line/type/avoid. Anchors: lines 14-17, 73-75.
- **I-002/I002-I05-006** [SRC] (DEEP-I-002.discovery.md:L467| unit=I-5): [E][measurement] A sampled same-decision A/B compares pristine rebuild against accumulated context via the E-19 reasoning-quality probe. Anchors: lines 5-9, 16-17, 43-44.
- **I-002/I002-I05-007** [SD] (DEEP-I-002.discovery.md:L468| unit=I-5): [E][observability] The A/B outcome delta is ledgered in a `VividnessProbe` row and must recompute under `ELAI_SEED`. Anchors: lines 16-17, 43-44, 87-89.
- **I-002/I002-I05-008** [SRC] (DEEP-I-002.discovery.md:L469| unit=I-5): [E][non-goal] Carrying raw tracebacks is explicitly the anti-pattern. Anchor: line 18.
- **I-002/I002-I05-009** [SRC] (DEEP-I-002.discovery.md:L470| unit=I-5): [E][non-goal] Truncation is rejected because it preserves a smaller attention sink; context must be rebuilt. Anchors: lines 18-20.
- **I-002/I002-I05-010** [SRC] (DEEP-I-002.discovery.md:L471| unit=I-5): [E][architecture] I-5 extends H-6 turn masking and never creates a parallel masking/turn-drop layer. Anchors: lines 20-21, 28-29, 54-56, 76-77.
- **I-002/I002-I05-011** [SRC] (DEEP-I-002.discovery.md:L472| unit=I-5): [E][constraint] No numeric vividness improvement may be claimed before local A/B measurement. Anchors: lines 21-22, 78-79, 100-101.
- **I-002/I002-I05-012** [SRC] (DEEP-I-002.discovery.md:L473| unit=I-5): [E][integration] The rebuild hooks `context_budget.rs` assembly while preserving `update_breakpoints` byte-prefix stability and using `ContextSlot`. Anchors: lines 24-27, 53-56, 64-66.
- **I-002/I002-I05-013** [AC] (DEEP-I-002.discovery.md:L474| unit=I-5): [E][dependency][A] H-6 and H-50 are plan-asserted Wave-2 predecessors and must be re-grepped before execution. Anchors: lines 28-31, 97-99.
- **I-002/I002-I05-014** [AC] (DEEP-I-002.discovery.md:L475| unit=I-5): [E][dependency][A] E-19 is a plan-asserted existing probe that I-5 calls rather than rebuilds; it must be re-grepped. Anchors: lines 32-33, 68, 99.
- **I-002/I002-I05-015** [SRC] (DEEP-I-002.discovery.md:L476| unit=I-5): [E][schema][status] `ContextPruned` is described as net-new and confirmed absent as a named action type at authoring. Anchors: lines 34-35.
- **I-002/I002-I05-016** [SRC] (DEEP-I-002.discovery.md:L477| unit=I-5): [E][scope] Tier 1 creates `crates/orchestrator/src/pristine_path_filter.rs` and logs byte deltas from exclusion/compression. Anchors: lines 37-40, 52-53.
- **I-002/I002-I05-017** [SRC] (DEEP-I-002.discovery.md:L478| unit=I-5): [E][schema] Register `ContextPruned` with `SiblingExcluded`, `TracebackContrasted`, and `VividnessProbe`; baseline +1. Anchors: lines 41-44, 61-63, 105-106.
- **I-002/I002-I05-018** [SRC] (DEEP-I-002.discovery.md:L479| unit=I-5): [E][sampling] Only a sampled fraction of expansions runs both context arms. Anchors: lines 43-44.
- **I-002/I002-I05-019** [SRC] (DEEP-I-002.discovery.md:L480| unit=I-5): [E][integration] The production caller is I-4 `SearchLadder::run_best_of_n -> PristinePathFilter::build_branch_context`. Anchors: lines 45, 69-72.
- **I-002/I002-I05-020** [SRC] (DEEP-I-002.discovery.md:L481| unit=I-5): [E][artifact] Tier 1 requires `workflow/reasoning/I-5-decisions.md`. Anchor: line 46.
- **I-002/I002-I05-021** [SRC] (DEEP-I-002.discovery.md:L482| unit=I-5 SURFACE): [E][deferred][UI] The ReasoningDrawer vividness-tax tile is Tier 2, wire-dark until at least three A/B runs show a consistent delta and the GD-4 bound is measured. Anchors: lines 47-49, 57-58, 81-83.
- **I-002/I002-I05-022** [SRC] (DEEP-I-002.discovery.md:L483| unit=I-5): [E][deferred] Per-task-class contrastive-token-budget auto-tuning is deferred until after measurement. Anchor: line 50.
- **I-002/I002-I05-023** [SRC] (DEEP-I-002.discovery.md:L484| unit=I-5): [E][integration] H-50 supplies regime-aware assembly; I-5 shares that path instead of forking it. Anchors: lines 30-31, 67, 110.
- **I-002/I002-I05-024** [SRC] (DEEP-I-002.discovery.md:L485| unit=I-5): [E][reachability] Contract-test-only callers do not satisfy production reachability R7. Anchor: line 72.
- **I-002/I002-I05-025** [SRC] (DEEP-I-002.discovery.md:L486| unit=I-5): [E][implementation] Production paths forbid `.unwrap()`/`.expect()`, blake2/sha2, and fabricated measurements. Anchor: line 80.
- **I-002/I002-I05-026** [SRC] (DEEP-I-002.discovery.md:L487| unit=I-5 SURFACE): [E][UI][done-gate] If the dashboard tile ships in the H-15 era, `cross_lang_diff.py PASS` is mandatory; absence is W-6-justified for v1. Anchors: lines 81-83.
- **I-002/I002-I05-027** [SRC] (DEEP-I-002.discovery.md:L488| unit=I-5): [E][trigger] The live observable uses the retry-branch fixture with search ladder after I-4 v1 ships. Anchors: lines 84-86.
- **I-002/I002-I05-028** [SRC] (DEEP-I-002.discovery.md:L489| unit=I-5): [E][observability] The SSB must contain `SiblingExcluded` for the failed payload and `TracebackContrasted` for the compressed constraint. Anchors: lines 87-89.
- **I-002/I002-I05-029** [SRC] (DEEP-I-002.discovery.md:L490| unit=I-5): [E][acceptance] The outbound retry payload must contain zero bytes of the prior 5K-line traceback and must contain the <=30-token constraint phrase. Anchors: lines 90-92.
- **I-002/I002-I05-030** [SRC] (DEEP-I-002.discovery.md:L491| unit=I-5): [E][broken-arm] `--carry-tracebacks` must re-inflate the payload to at least 5K tokens and fail. Anchors: lines 91-92.
- **I-002/I002-I05-031** [SRC] (DEEP-I-002.discovery.md:L492| unit=I-5): [E][grounding] ID-1 is the stateless-generator/trajectory-pruning basis and requires instrument-first treatment. Anchors: lines 94-96.
- **I-002/I002-I05-032** [SRC] (DEEP-I-002.discovery.md:L493| unit=I-5): [E][grounding] IS-6 identifies `ContextSlot` and `update_breakpoints`; CL-1 binds regime awareness. Anchors: lines 97, 102.
- **I-002/I002-I05-033** [SRC] (DEEP-I-002.discovery.md:L494| unit=I-5): [E][evidence] Any arXiv identifier placed in the decision document must be web-verified before becoming a headline. Anchors: lines 103-104.
- **I-002/I002-I05-034** [AC] (DEEP-I-002.discovery.md:L495| unit=I-5): [E][schema][A] The 221/532 baseline is dated provenance and must be re-read under W-9 at edit. Anchors: lines 105-106.
- **I-002/I002-I05-035** [SRC] (DEEP-I-002.discovery.md:L496| unit=I-5): [E][dependency] I-5 is a Wave-2 building block that I-4 consumes in Wave 4. Anchors: lines 108-113. I-5 ambiguity/inference register:
- **I-002/I002-I05-036** [AC] (DEEP-I-002.discovery.md:L500| unit=I-5): [A][threshold] `<=30` tokens is a hard fixture/contract ceiling, but optimal per-task-class budgets remain deferred rather than product constants. Anchors: lines 14-17, 47-50.
- **I-002/I002-I05-037** [AC] (DEEP-I-002.discovery.md:L501| unit=I-5 SURFACE): [A][UI] The exact live UI timing depends on H-15-era surfaces and measured stability; v1 explicitly permits no tile. Anchors: lines 47-49, 81-83.
- **I-002/I002-I05-038** [INF] (DEEP-I-002.discovery.md:L502| unit=I-5): [I][privacy] Hard-excluding sibling tracebacks also limits accidental replay of sensitive raw error content, but the source frames this as context quality, not a privacy guarantee. Anchors: lines 13-18, 90-92.

#### I-002 / unit `I-10` (42 records)

- **I-002/I002-I10-001** [SRC] (DEEP-I-002.discovery.md:L508| unit=I-10): [E][premise] Timestamps, mutable endpoint text, and provider-default TTLs can silently invalidate cloud prompt prefixes and force retransfers. Anchors: lines 4-7.
- **I-002/I002-I10-002** [SRC] (DEEP-I-002.discovery.md:L509| unit=I-10): [E][cache] Prefix bytes are frozen end-to-end. Anchors: lines 6-7, 12-13.
- **I-002/I002-I10-003** [SRC] (DEEP-I-002.discovery.md:L510| unit=I-10): [E][constraint] Cached prefix regions reject timestamps, UUIDs, and mutable endpoint strings. Anchors: lines 12-13, 38, 68-70.
- **I-002/I002-I10-004** [SRC] (DEEP-I-002.discovery.md:L511| unit=I-10): [E][tools] Dynamic tool restrictions use `tool_choice`/`allowed_tools` payload attributes, never prompt-text mutation. Anchors: lines 12-15, 38.
- **I-002/I002-I10-005** [AC] (DEEP-I-002.discovery.md:L512| unit=I-10): [E][measurement][A] Inline-frozen versus H-4-deferred tool arrays are chosen per catalog size from measured cache-hit-by-token cost; neither layout is preselected. Anchors: lines 14-15, 71, 95.
- **I-002/I002-I10-006** [SRC] (DEEP-I-002.discovery.md:L513| unit=I-10): [E][cache] Frozen-prefix breakpoints use an explicit one-hour TTL. Anchors: lines 16-18.
- **I-002/I002-I10-007** [SRC] (DEEP-I-002.discovery.md:L514| unit=I-10): [E][cache] Volatile tails use an explicit five-minute TTL. Anchors: lines 16-18.
- **I-002/I002-I10-008** [SRC] (DEEP-I-002.discovery.md:L515| unit=I-10): [E][ordering] Long-TTL entries must precede shorter-TTL entries in a request. Anchors: lines 17-18.
- **I-002/I002-I10-009** [AC] (DEEP-I-002.discovery.md:L516| unit=I-10): [E][economics][A] Planner defaults use one-hour 2x/~3-read breakeven and five-minute 1.25x/one-read breakeven, but pricing must be re-verified and never treated as fixed. Anchors: lines 19-20, 88-89.
- **I-002/I002-I10-010** [SRC] (DEEP-I-002.discovery.md:L517| unit=I-10): [E][planning] `CacheEconomicsPlanner` reads journaled inter-call gaps and selects a TTL tier per call class with a write-premium breakeven check. Anchors: lines 16-20, 36-37, 53-55.
- **I-002/I002-I10-011** [SRC] (DEEP-I-002.discovery.md:L518| unit=I-10): [E][rate-limit] `max_tokens_clamp` bounds every cached scoring call to its output schema. Anchors: lines 21-22, 36-37, 56.
- **I-002/I002-I10-012** [SRC] (DEEP-I-002.discovery.md:L519| unit=I-10): [E][failure] `PrefixInvarianceGuard` halts on unexplained misses over 10% for identical task classes. Anchors: lines 23-24, 36-37, 55.
- **I-002/I002-I10-013** [SRC] (DEEP-I-002.discovery.md:L520| unit=I-10): [E][diagnostics] Guard trip emits a blake3 prefix diff naming the mutation and leaking byte range. Anchors: lines 23-24, 36-37, 55.
- **I-002/I002-I10-014** [SRC] (DEEP-I-002.discovery.md:L521| unit=I-10): [E][scheduling] Burst-amortized scheduling is the primary TTL-preservation strategy. Anchors: lines 23-26.
- **I-002/I002-I10-015** [SRC] (DEEP-I-002.discovery.md:L522| unit=I-10): [E][fallback] Heartbeat traffic survives only as a last resort when burst scheduling cannot apply. Anchors: lines 24-26.
- **I-002/I002-I10-016** [CON] (DEEP-I-002.discovery.md:L523| unit=I-10): [E][dependency][C] I-10 adds only TTL/economics/guard functionality if H-1/H-2 already ship and must re-grep the open gap before coding. Anchors: lines 9-11, 28-33, 98-99.
- **I-002/I002-I10-017** [AC] (DEEP-I-002.discovery.md:L524| unit=I-10): [E][status][A] At authoring, CacheControl plumbing existed, Anthropic marker sites still had `cache_control: None`, and the planner/selector/guard were absent; each is plan-asserted and rechecked under W-7. Anchors: lines 28-33.
- **I-002/I002-I10-018** [SRC] (DEEP-I-002.discovery.md:L525| unit=I-10): [E][scope] Tier 1 creates `crates/orchestrator/src/cache_economics.rs` with planner, guard, and clamp helper. Anchors: lines 35-37, 47-49.
- **I-002/I002-I10-019** [SRC] (DEEP-I-002.discovery.md:L526| unit=I-10): [E][validation] Tier 1 validates frozen prefixes at prompt assembly and rejects dynamic text. Anchor: line 38.
- **I-002/I002-I10-020** [SRC] (DEEP-I-002.discovery.md:L527| unit=I-10): [E][integration] Every `apply_anthropic_cache_markers` site receives TTL annotation; the shipped marker implementation is never reimplemented. Anchors: lines 39, 61-62, 72.
- **I-002/I002-I10-021** [SRC] (DEEP-I-002.discovery.md:L528| unit=I-10): [E][schema] Add `CacheEconomicsEvent { TtlPlanned, HitRateGuardTripped, PrefixDiffDumped }`, baseline +1. Anchors: lines 40, 53-57, 90.
- **I-002/I002-I10-022** [SRC] (DEEP-I-002.discovery.md:L529| unit=I-10): [E][test] A deterministic seeded two-turn fixture proves turn-2 cache read and an injected-timestamp guard trip. Anchor: line 41.
- **I-002/I002-I10-023** [SRC] (DEEP-I-002.discovery.md:L530| unit=I-10): [E][deferred] Default-on planner rollout to all task classes waits for at least three real-run false-positive/over-block measurements. Anchors: lines 43-45.
- **I-002/I002-I10-024** [SRC] (DEEP-I-002.discovery.md:L531| unit=I-10): [E][deferred] Automated pricing-breakpoint reverification is a monitoring job, not a gate. Anchor: line 45.
- **I-002/I002-I10-025** [SRC] (DEEP-I-002.discovery.md:L532| unit=I-10): [E][integration] Marker types, context breakpoints, Anthropic cache-control paths, and `build_tool_catalog` are the production seams. Anchors: lines 47-50, 58-66.
- **I-002/I002-I10-026** [SRC] (DEEP-I-002.discovery.md:L533| unit=I-10 SURFACE): [E][UI] TTL/economics fields extend the H-9 cost tile in the CostDrawer/CachePanel family. Anchors: lines 50, 77-79.
- **I-002/I002-I10-027** [SRC] (DEEP-I-002.discovery.md:L534| unit=I-10): [E][reachability] Production reachability is Anthropic cache-control to `plan_ttl()` and context assembly to `check_miss_rate()`; tests alone do not satisfy R7. Anchors: lines 64-67.
- **I-002/I002-I10-028** [SRC] (DEEP-I-002.discovery.md:L535| unit=I-10): [E][constraint] TTL may never be left to provider default. Anchor: line 70.
- **I-002/I002-I10-029** [SRC] (DEEP-I-002.discovery.md:L536| unit=I-10): [E][architecture] Neither `apply_anthropic_cache_markers` nor `update_breakpoints` may be duplicated. Anchors: lines 71-73.
- **I-002/I002-I10-030** [SRC] (DEEP-I-002.discovery.md:L537| unit=I-10): [E][implementation] Production forbids `.unwrap()`/`.expect()`, blake2/sha2, and fabricated miss/cost measurements; blake3 is mandatory. Anchors: lines 74-76.
- **I-002/I002-I10-031** [SRC] (DEEP-I-002.discovery.md:L538| unit=I-10): [E][measurement] Cost/miss measurements are per configuration and never multiplied. Anchor: line 76.
- **I-002/I002-I10-032** [SRC] (DEEP-I-002.discovery.md:L539| unit=I-10 SURFACE): [E][UI][done-gate] The user-visible planner decision/guard signal makes a frontend pair mandatory. Anchors: lines 77-79.
- **I-002/I002-I10-033** [SRC] (DEEP-I-002.discovery.md:L540| unit=I-10 SURFACE): [E][trigger] The observable is a two-turn CLI run emitting SSB. Anchors: lines 80-82.
- **I-002/I002-I10-034** [SRC] (DEEP-I-002.discovery.md:L541| unit=I-10): [E][schema] `TtlPlanned` includes `ttl_seconds: 3600|300` and `breakeven_reads`. Anchor: line 82.
- **I-002/I002-I10-035** [SRC] (DEEP-I-002.discovery.md:L542| unit=I-10): [E][acceptance] Turn 2 must journal `cache_read_input_tokens > 0` over the frozen prefix. Anchor: line 83.
- **I-002/I002-I10-036** [SRC] (DEEP-I-002.discovery.md:L543| unit=I-10): [E][acceptance] An injected timestamp must trip the guard within one session and identify the leaking mutation/byte range. Anchors: lines 82-83.
- **I-002/I002-I10-037** [SRC] (DEEP-I-002.discovery.md:L544| unit=I-10): [E][broken-arm] Disabling the guard must fail because the injected mutation no longer trips. Anchor: line 83.
- **I-002/I002-I10-038** [SRC] (DEEP-I-002.discovery.md:L545| unit=I-10): [E][integration] H-9's wire-dark hit-rate monitor becomes planner data, and E-39 consumes economics signals. Anchors: lines 92-97.
- **I-002/I002-I10-039** [SRC] (DEEP-I-002.discovery.md:L546| unit=I-10): [E][done-gate] Full-form expansion is mandatory before execution, including live re-grep of IS-5 and the IC-5 choice. Anchors: lines 98-99. I-10 ambiguity/conflict register:
- **I-002/I002-I10-040** [AC] (DEEP-I-002.discovery.md:L550| unit=I-10): [A][layout] Inline versus deferred tool catalogs remains a runtime measurement decision, not a source-level constant. Anchors: lines 14-15, 71, 95.
- **I-002/I002-I10-041** [AC] (DEEP-I-002.discovery.md:L551| unit=I-10 SURFACE): [A][UI] `CostDrawer` versus `CachePanel` is plan-asserted naming and must be re-grepped. Anchor: line 50.
- **I-002/I002-I10-042** [CON] (DEEP-I-002.discovery.md:L552| unit=I-10): [C][strategy] The plan corrects heartbeat-primary to burst/TTL-primary and corrects any assumption that H-4 deferral must be overridden. Anchors: lines 14-15, 23-26, 87.

#### I-002 / unit `I-29` (46 records)

- **I-002/I002-I29-001** [SRC] (DEEP-I-002.discovery.md:L558| unit=I-29): [E][generation] A Mercury-class diffusion LLM is a fast, explicitly untrusted draft generator with no privileged status. Anchors: lines 1-8.
- **I-002/I002-I29-002** [SRC] (DEEP-I-002.discovery.md:L559| unit=I-29): [E][security] Every diffusion output traverses the identical I-17/I-20 verification/mutation stack used by autoregressive output. Anchors: lines 4-6, 19-21.
- **I-002/I002-I29-003** [CON] (DEEP-I-002.discovery.md:L560| unit=I-29): [E][non-goal][C] The full TTD-DR denoising/research loop is declined; the narrow steal is draft speed, not autonomy. Anchors: lines 7-9, 28-30, 79.
- **I-002/I002-I29-004** [SRC] (DEEP-I-002.discovery.md:L561| unit=I-29): [E][permissions] No new dependency or network code may land before user approval. Anchors: lines 8-9, 49-53.
- **I-002/I002-I29-005** [SRC] (DEEP-I-002.discovery.md:L562| unit=I-29): [E][done-gate] L effort requires full-form expansion before execution. Anchor: line 11.
- **I-002/I002-I29-006** [SRC] (DEEP-I-002.discovery.md:L563| unit=I-29): [E][integration] The dLLM client adds a `RuntimeAdapter` variant and must not fork the trait. Anchors: lines 13-15, 57, 65, 102-104.
- **I-002/I002-I29-007** [SRC] (DEEP-I-002.discovery.md:L564| unit=I-29): [E][security] Before naming a client crate, executor must check npm/PyPI/cargo, repository URL, last-published date, and license, then obtain user confirmation. Anchors: lines 14-16, 49-51, 80.
- **I-002/I002-I29-008** [SRC] (DEEP-I-002.discovery.md:L565| unit=I-29): [E][workflow] The orchestrator assigns a subtask to the diffusion lane. Anchors: lines 16-17.
- **I-002/I002-I29-009** [SRC] (DEEP-I-002.discovery.md:L566| unit=I-29): [E][workflow] The dLLM creates a one-pass or few-step parallel-fill draft. Anchors: lines 17-18.
- **I-002/I002-I29-010** [SRC] (DEEP-I-002.discovery.md:L567| unit=I-29): [E][workflow] Multiple sub-agent patches flow through `draft_merge.rs`; the merged result then enters I-17 and surviving output enters I-20. Anchors: lines 18-21.
- **I-002/I002-I29-011** [SRC] (DEEP-I-002.discovery.md:L568| unit=I-29): [E][concurrency] Parallel agents append patch operations to a shared CRDT log. Anchors: lines 21-23.
- **I-002/I002-I29-012** [SRC] (DEEP-I-002.discovery.md:L569| unit=I-29): [E][determinism] Merge is deterministic and commutative. Anchors: lines 21-23.
- **I-002/I002-I29-013** [CON] (DEEP-I-002.discovery.md:L570| unit=I-29): [E][conflict-resolution] Character/line conflicts use last-write-wins on a content hash, never LLM judgment. Anchors: lines 22-24, 83.
- **I-002/I002-I29-014** [SRC] (DEEP-I-002.discovery.md:L571| unit=I-29): [E][validation] C-32 MiniCritic runs at every semantic merge checkpoint. Anchors: lines 23-24, 42-43, 69.
- **I-002/I002-I29-015** [SRC] (DEEP-I-002.discovery.md:L572| unit=I-29): [E][failure] A compile-failing diffusion draft produces the same hard-veto/pruning outcome as an AR draft. Anchors: lines 19-20.
- **I-002/I002-I29-016** [SRC] (DEEP-I-002.discovery.md:L573| unit=I-29): [E][integrity] Each `DraftVetoed` row must reference a `TriLayerVerdict(HardVeto)` row; diffusion has no shortcut gate. Anchors: lines 24-26, 45-46, 78, 90-91.
- **I-002/I002-I29-017** [SRC] (DEEP-I-002.discovery.md:L574| unit=I-29): [E][rollout] Default-on is forbidden until a corruption rate is bounded over at least three real runs. Anchors: lines 26-28, 38, 52, 81.
- **I-002/I002-I29-018** [SRC] (DEEP-I-002.discovery.md:L575| unit=I-29): [E][metric] Corruption rate is defined as the fraction of drafts reaching Layer-1 hard veto that are genuinely bad rather than merely slow. Anchors: lines 26-27.
- **I-002/I002-I29-019** [SRC] (DEEP-I-002.discovery.md:L576| unit=I-29): [E][flags] Until promotion, the lane is an allowlisted experiment in `wire-dark-allowlist.toml`, following `darwinian-evolve`. Anchors: lines 27-28, 44, 58, 97.
- **I-002/I002-I29-020** [AC] (DEEP-I-002.discovery.md:L577| unit=I-29): [E][dependency][A] B-37, C-32, and E-48 are plan-asserted seams that must be re-grepped. Anchors: lines 32-35.
- **I-002/I002-I29-021** [SRC] (DEEP-I-002.discovery.md:L578| unit=I-29): [E][dependency] I-17 is a blocking prerequisite for live gate parity. Anchor: line 36.
- **I-002/I002-I29-022** [SRC] (DEEP-I-002.discovery.md:L579| unit=I-29): [E][status] At authoring, neither dLLM client nor merge engine was production-reachable. Anchor: line 37.
- **I-002/I002-I29-023** [SRC] (DEEP-I-002.discovery.md:L580| unit=I-29): [E][scope] Tier 1 creates an approximately 100-LOC, in-process, dependency-free `DraftMerger` skeleton with patch log, content-hash resolution, and MiniCritic call sites. Anchors: lines 40-43, 55-56, 63-64.
- **I-002/I002-I29-024** [SRC] (DEEP-I-002.discovery.md:L581| unit=I-29): [E][config] Tier 1 adds an allowlist row containing `type=experiment`, owner, expiry date, `promotion_plan=I-29`, and smoke-test path. Anchor: line 44.
- **I-002/I002-I29-025** [SRC] (DEEP-I-002.discovery.md:L582| unit=I-29): [E][test] Tier 1 adds `i29_diffusion_draft_lane_contract.rs` with a seeded veto-parity fixture. Anchors: lines 45-46.
- **I-002/I002-I29-026** [SRC] (DEEP-I-002.discovery.md:L583| unit=I-29): [E][artifact] Tier 1 adds `workflow/reasoning/I-29-decisions.md`. Anchor: line 47.
- **I-002/I002-I29-027** [SRC] (DEEP-I-002.discovery.md:L584| unit=I-29): [E][deferred] Tier 2 defers the approved dLLM `DiffusionAdapter` and live endpoint wiring. Anchors: lines 49-52, 57.
- **I-002/I002-I29-028** [SRC] (DEEP-I-002.discovery.md:L585| unit=I-29): [E][deferred] Default-on promotion and a parallel-volume test remain gated on live client/evidence. Anchors: lines 52-53.
- **I-002/I002-I29-029** [SRC] (DEEP-I-002.discovery.md:L586| unit=I-29 SURFACE): [E][UI] Generation cards display a draft-lane badge with `DraftAccepted`, `DraftVetoed`, and `CrdtMerged`; `cross_lang_diff.py` covers Tauri-to-Svelte. Anchors: lines 60, 85-86.
- **I-002/I002-I29-030** [SRC] (DEEP-I-002.discovery.md:L587| unit=I-29): [E][schema] Produce `DiffusionDraftEvent { DraftAccepted, DraftVetoed, CrdtMerged }`, baseline +1. Anchors: lines 85-86, 99.
- **I-002/I002-I29-031** [SRC] (DEEP-I-002.discovery.md:L588| unit=I-29): [E][integration] Consumed symbols are B-37 `RuntimeAdapter`, C-32 MiniCritic, I-17 tri-layer veto, and I-20 budgeted mutation, each reused rather than rebuilt. Anchors: lines 67-71, 102-107.
- **I-002/I002-I29-032** [SRC] (DEEP-I-002.discovery.md:L589| unit=I-29): [E][reachability] The eventual production chain is search ladder/task runner -> `DiffusionAdapter` -> `DraftMerger` -> I-17. Anchor: line 74.
- **I-002/I002-I29-033** [SRC] (DEEP-I-002.discovery.md:L590| unit=I-29): [E][reachability] Direct Tier-1 contract calls do not satisfy R7; only the gated Tier-2 chain establishes production reachability. Anchor: line 75.
- **I-002/I002-I29-034** [SRC] (DEEP-I-002.discovery.md:L591| unit=I-29): [E][constraint] No placeholder/TODO for the declined TTD-DR loop may appear. Anchors: lines 77-80.
- **I-002/I002-I29-035** [SRC] (DEEP-I-002.discovery.md:L592| unit=I-29): [E][implementation] Production forbids `.unwrap()`/`.expect()` and blake2/sha2. Anchor: line 82.
- **I-002/I002-I29-036** [SRC] (DEEP-I-002.discovery.md:L593| unit=I-29 SURFACE): [E][trigger] Tier 1 uses the wiring-contract fixture harness; gated Tier 2 uses `elai run --diffusion-lane`. Anchors: lines 88-90.
- **I-002/I002-I29-037** [SRC] (DEEP-I-002.discovery.md:L594| unit=I-29): [E][acceptance] A two-agent patch fixture emits `CrdtMerged`, and a clean draft emits `DraftAccepted`. Anchor: line 90.
- **I-002/I002-I29-038** [SRC] (DEEP-I-002.discovery.md:L595| unit=I-29): [E][determinism-test] Two seeded runs of parallel patches must produce byte-identical output and the same content hash. Anchor: line 91.
- **I-002/I002-I29-039** [SRC] (DEEP-I-002.discovery.md:L596| unit=I-29): [E][regression] With the diffusion lane off, every existing generation path remains untouched. Anchor: line 91.
- **I-002/I002-I29-040** [SRC] (DEEP-I-002.discovery.md:L597| unit=I-29): [E][negative-test] Gate bypass is structurally impossible; the contract asserts the `DraftVetoed -> HardVeto` join. Anchor: line 91.
- **I-002/I002-I29-041** [AC] (DEEP-I-002.discovery.md:L598| unit=I-29 SURFACE): [E][evidence][A] The speed claim is moderate/[DESIGN]/[WEAK] until Tier 2 lands and ELAI measures it; SSB planning is strong. Anchors: lines 93-100.
- **I-002/I002-I29-042** [SRC] (DEEP-I-002.discovery.md:L599| unit=I-29): [E][evidence] Any Mercury-class paper citation must be web-verified before headlining; no paper is asserted for the hand-rolled CRDT. Anchor: line 100. I-29 ambiguity/conflict register:
- **I-002/I002-I29-043** [AC] (DEEP-I-002.discovery.md:L603| unit=I-29): [A][implementation] The detail file resolves the summary's “Automerge-class versus hand-rolled” choice toward an in-process hand-rolled Tier-1 skeleton with no external dependency. Anchors: lines 42-43, 98-100.
- **I-002/I002-I29-044** [CON] (DEEP-I-002.discovery.md:L604| unit=I-29): [C][status] Contrary to a broad reading of “adds a diffusion LLM,” only the deterministic merge skeleton/test/config ship in Tier 1; the actual client and network lane are explicitly deferred pending approval. Anchors: lines 40-53.
- **I-002/I002-I29-045** [AC] (DEEP-I-002.discovery.md:L605| unit=I-29): [A][threshold] The source says default-on needs `>=3` real runs, while its stub-ban phrase “`<=3` runs are insufficient” could imply strictly more than three; implementation should preserve the clearer `>=3` rule or resolve the wording. Anchors: lines 26-28, 38, 52, 81.
- **I-002/I002-I29-046** [INF] (DEEP-I-002.discovery.md:L606| unit=I-29): [I][failure] A missing referenced hard-veto row necessarily makes gate-parity evidence invalid, even though the source expresses this through the contract join rather than a named runtime recovery. Anchors: lines 24-26, 90-91.

#### I-002 / unit `I-46` (37 records)

- **I-002/I002-I46-001** [SRC] (DEEP-I-002.discovery.md:L612| unit=I-46): [E][premise] Existing validation allows/denies, while interception can recover dead tool calls, reduce raw-output context floods, and journal substitutions. Anchors: lines 6-10.
- **I-002/I002-I46-002** [SRC] (DEEP-I-002.discovery.md:L613| unit=I-46 SURFACE): [E][compliance] Before execution, read gajae `bash-interceptor.ts`, `bash-command-fixup.ts`, and `bash-allowed-prefixes.ts`. Anchors: lines 12-14, 59-61.
- **I-002/I002-I46-003** [SRC] (DEEP-I-002.discovery.md:L614| unit=I-46 SURFACE): [E][action] The interceptor parses/classifies each bash command before existing bash validation. Anchors: lines 15-16, 41-45.
- **I-002/I002-I46-004** [SRC] (DEEP-I-002.discovery.md:L615| unit=I-46): [E][redirect] Bare grep/find/cat/head/tail covered by registered search/read tools are refused with redirect. Anchors: lines 15-16.
- **I-002/I002-I46-005** [SRC] (DEEP-I-002.discovery.md:L616| unit=I-46): [E][response] Redirect response includes the exact registered substitute tool call. Anchor: line 16.
- **I-002/I002-I46-006** [SRC] (DEEP-I-002.discovery.md:L617| unit=I-46 SURFACE): [E][fixup] A small frozen deterministic table repairs known-broken command shapes such as flag order and missing quoting. Anchors: lines 17-18.
- **I-002/I002-I46-007** [SRC] (DEEP-I-002.discovery.md:L618| unit=I-46): [E][constraint] Fixup is allowed only when semantics-preserving. Anchor: line 18.
- **I-002/I002-I46-008** [SRC] (DEEP-I-002.discovery.md:L619| unit=I-46): [E][allowlist] A bounded frozen allowed-prefix table identifies commands that pass unchanged. Anchors: lines 18-19, 31-33.
- **I-002/I002-I46-009** [SRC] (DEEP-I-002.discovery.md:L620| unit=I-46): [E][non-goal] The interceptor performs no LLM-driven rewrite. Anchors: lines 20-22, 48.
- **I-002/I002-I46-010** [SRC] (DEEP-I-002.discovery.md:L621| unit=I-46): [E][security] It never auto-executes a proposed substitute; the agent must reissue it so the supervisor/audit trail remains intact. Anchors: lines 20-22, 48.
- **I-002/I002-I46-011** [SRC] (DEEP-I-002.discovery.md:L622| unit=I-46): [E][invariant] Interception may narrow, never widen, existing validation. Anchors: lines 21-22.
- **I-002/I002-I46-012** [AC] (DEEP-I-002.discovery.md:L623| unit=I-46): [E][status][A] `bash_validation.rs` is shipped, while redirect absence is STEAL-LIST-asserted and must be re-grepped. Anchors: lines 24-26.
- **I-002/I002-I46-013** [SRC] (DEEP-I-002.discovery.md:L624| unit=I-46): [E][architecture] E-30 argv rules, planned I-35 sequence rules, and I-46 substitution are disjoint enforcement dimensions. Anchors: lines 27-29.
- **I-002/I002-I46-014** [SRC] (DEEP-I-002.discovery.md:L625| unit=I-46): [E][scope] Ship now: interceptor, fixup table, prefix table, and read/search-family redirects in the existing validation path. Anchors: lines 31-33.
- **I-002/I002-I46-015** [SRC] (DEEP-I-002.discovery.md:L626| unit=I-46): [E][deferred] Recipe-runner redirects wait for I-53. Anchor: line 34.
- **I-002/I002-I46-016** [SRC] (DEEP-I-002.discovery.md:L627| unit=I-46): [E][deferred] Per-project custom prefix configuration is deferred. Anchor: line 34.
- **I-002/I002-I46-017** [SRC] (DEEP-I-002.discovery.md:L628| unit=I-46): [E][integration] Extend `crates/tools/src/bash_validation.rs` and bash dispatch; do not fork validation. Anchors: lines 36-38.
- **I-002/I002-I46-018** [SRC] (DEEP-I-002.discovery.md:L629| unit=I-46): [E][integrity] Fixup/prefix tables are R8-frozen repo artifacts and a run cannot edit its own rewrite rules. Anchors: lines 37-38.
- **I-002/I002-I46-019** [SRC] (DEEP-I-002.discovery.md:L630| unit=I-46 SURFACE): [E][UI] Redirect rows appear in the existing tool-call drawer; no component is added. Anchor: line 39.
- **I-002/I002-I46-020** [SRC] (DEEP-I-002.discovery.md:L631| unit=I-46): [E][symbols] Produce `tools::bash_interceptor`, frozen tables, and `BashRedirected` SSB rows. Anchors: lines 41-43.
- **I-002/I002-I46-021** [SRC] (DEEP-I-002.discovery.md:L632| unit=I-46): [E][integration] Consume the existing validation verdict path, tool registry, and unchanged downstream E-30 argv allowlist. Anchors: lines 44-45.
- **I-002/I002-I46-022** [SRC] (DEEP-I-002.discovery.md:L633| unit=I-46): [E][reachability] Every production agent bash call traverses the interceptor; tests alone do not satisfy R7. Anchors: lines 46-47.
- **I-002/I002-I46-023** [SRC] (DEEP-I-002.discovery.md:L634| unit=I-46 SURFACE): [E][security] No bypass flag may skip interception. Anchors: lines 48-49.
- **I-002/I002-I46-024** [SRC] (DEEP-I-002.discovery.md:L635| unit=I-46): [E][parsing] Regex-only “looks like grep” detection is forbidden; argv parsing is required. Anchors: lines 49-50.
- **I-002/I002-I46-025** [SRC] (DEEP-I-002.discovery.md:L636| unit=I-46): [E][implementation] Production paths forbid `.unwrap()`. Anchor: line 50.
- **I-002/I002-I46-026** [SRC] (DEEP-I-002.discovery.md:L637| unit=I-46): [E][trigger] The live trigger is an agent bash call such as `grep -rn pattern crates/`. Anchors: lines 51-53.
- **I-002/I002-I46-027** [SRC] (DEEP-I-002.discovery.md:L638| unit=I-46): [E][observability] That call emits `BashRedirected(Redirected)` naming its substitute. Anchors: lines 52-54.
- **I-002/I002-I46-028** [SRC] (DEEP-I-002.discovery.md:L639| unit=I-46): [E][observability] A known-fixup fixture emits `BashRedirected(FixedUp)`. Anchors: lines 53-54.
- **I-002/I002-I46-029** [SRC] (DEEP-I-002.discovery.md:L640| unit=I-46): [E][observability] A novel non-allowlisted prefix emits `BashRedirected(PrefixBlocked)`. Anchor: line 54.
- **I-002/I002-I46-030** [SRC] (DEEP-I-002.discovery.md:L641| unit=I-46 SURFACE): [E][acceptance] Redirected commands create no process in the journal window. Anchors: lines 55-57.
- **I-002/I002-I46-031** [SRC] (DEEP-I-002.discovery.md:L642| unit=I-46 SURFACE): [E][negative-test] An allowed-prefix command executes unchanged with zero redirect rows. Anchors: lines 55-57.
- **I-002/I002-I46-032** [SRC] (DEEP-I-002.discovery.md:L643| unit=I-46 SURFACE): [E][broken-arm] A redirect row plus a spawned grep in the same window fails. Anchor: line 57.
- **I-002/I002-I46-033** [SRC] (DEEP-I-002.discovery.md:L644| unit=I-46): [E][schema] Add `BashRedirected { Redirected, FixedUp, PrefixBlocked }`, baseline +1, and re-read live constants. Anchors: lines 64-68.
- **I-002/I002-I46-034** [IX] (DEEP-I-002.discovery.md:L645| unit=I-46): [E][interaction] H-4 discovers substitute tools; I-53 supplies future recipe targets; I-35 remains a separate sequence layer. Anchors: lines 64-66. I-46 ambiguity/inference register:
- **I-002/I002-I46-035** [AC] (DEEP-I-002.discovery.md:L649| unit=I-46): [A][config] The initial allowed-prefix/fixup table contents are not enumerated; only the read/search redirect family and the requirement that tables be bounded/frozen are explicit. Anchors: lines 15-19, 31-34.
- **I-002/I002-I46-036** [AC] (DEEP-I-002.discovery.md:L650| unit=I-46): [A][evidence] Redirect absence and gajae behavioral parity remain medium-confidence until the mandatory source read/re-grep. Anchors: lines 3-4, 13-14, 24-26.
- **I-002/I002-I46-037** [INF] (DEEP-I-002.discovery.md:L651| unit=I-46): [I][failure] A blocked prefix returns a journaled refusal rather than falling through, inferred from `PrefixBlocked` plus the no-bypass/narrowing invariants. Anchors: lines 21-22, 48-54.

#### I-002 / unit `I-54` (40 records)

- **I-002/I002-I54-001** [SRC] (DEEP-I-002.discovery.md:L657| unit=I-54): [E][premise] I-54 steals structural-search/edit ergonomics but builds on shipped tree-sitter 0.26 and nine in-tree grammars. Anchors: lines 1-12.
- **I-002/I002-I54-002** [SRC] (DEEP-I-002.discovery.md:L658| unit=I-54): [E][compliance] Before execution, read gajae `ast-edit.ts` and `ast-grep.ts` for interface behavior only. Anchors: lines 14-16.
- **I-002/I002-I54-003** [SRC] (DEEP-I-002.discovery.md:L659| unit=I-54): [E][parser] The engine uses tree-sitter query syntax and introduces no new pattern DSL. Anchors: lines 9-12.
- **I-002/I002-I54-004** [SRC] (DEEP-I-002.discovery.md:L660| unit=I-54): [E][rewrite] Rewrites use capture templates. Anchors: lines 10-12.
- **I-002/I002-I54-005** [SRC] (DEEP-I-002.discovery.md:L661| unit=I-54): [E][tool] Search maps a tree-sitter query to matches with file, line, and capture bindings. Anchors: lines 17-18.
- **I-002/I002-I54-006** [SRC] (DEEP-I-002.discovery.md:L662| unit=I-54): [E][tool] Preview maps query plus rewrite template to a would-be diff and writes nothing. Anchors: lines 18-19.
- **I-002/I002-I54-007** [SRC] (DEEP-I-002.discovery.md:L663| unit=I-54): [E][tool] Apply executes only the previewed edit set. Anchors: lines 18-20.
- **I-002/I002-I54-008** [SRC] (DEEP-I-002.discovery.md:L664| unit=I-54): [E][atomicity] Apply is atomic per file. Anchors: lines 18-20.
- **I-002/I002-I54-009** [SRC] (DEEP-I-002.discovery.md:L665| unit=I-54): [E][staleness] Apply refuses if any target file changed since preview. Anchor: line 19.
- **I-002/I002-I54-010** [SRC] (DEEP-I-002.discovery.md:L666| unit=I-54): [E][security] Structural apply traverses the identical PreToolUse edit gates used by ordinary Edit. Anchors: lines 20-21, 38-40, 47-48.
- **I-002/I002-I54-011** [CON] (DEEP-I-002.discovery.md:L667| unit=I-54): [E][dependency][C] Any dependency named `ast-grep` is a review-blocking violation. Anchors: lines 22-24, 51-53.
- **I-002/I002-I54-012** [SRC] (DEEP-I-002.discovery.md:L668| unit=I-54): [E][scope] A single call does not perform cross-language rewrite; queries are per language. Anchors: lines 22-23.
- **I-002/I002-I54-013** [SRC] (DEEP-I-002.discovery.md:L669| unit=I-54): [E][scope] The tool is syntax-only; semantic/type-aware renaming remains E-52 LSP territory. Anchors: lines 23-24.
- **I-002/I002-I54-014** [SRC] (DEEP-I-002.discovery.md:L670| unit=I-54): [E][workflow] Apply without a preceding preview row is forbidden. Anchors: lines 23-24, 51-52.
- **I-002/I002-I54-015** [SRC] (DEEP-I-002.discovery.md:L671| unit=I-54): [E][parser][status] The nine grammars are Rust, Python, Go, TypeScript, Java, C#, Ruby, JavaScript, and C. Anchors: lines 26-29.
- **I-002/I002-I54-016** [SRC] (DEEP-I-002.discovery.md:L672| unit=I-54): [E][architecture] Tools consume a public knowledge-crate parser/grammar API; no other crate may pin a second tree-sitter version. Anchors: lines 27-29, 38-40, 52-53.
- **I-002/I002-I54-017** [IX] (DEEP-I-002.discovery.md:L673| unit=I-54 SURFACE): [E][interaction] Harness-side `impact_radius.py`/`elai_graph` are blast-radius siblings; preview is the runtime-side equivalent surface. Anchors: lines 30-31.
- **I-002/I002-I54-018** [SRC] (DEEP-I-002.discovery.md:L674| unit=I-54): [E][scope] Tier 1 ships catalog-registered, gated search/preview/apply for Rust, Python, and TypeScript. Anchors: lines 33-35.
- **I-002/I002-I54-019** [SRC] (DEEP-I-002.discovery.md:L675| unit=I-54): [E][deferred] The remaining six grammars are enabled only on demand. Anchor: line 36.
- **I-002/I002-I54-020** [SRC] (DEEP-I-002.discovery.md:L676| unit=I-54): [E][deferred] A rewrite-template library is deferred. Anchor: line 36.
- **I-002/I002-I54-021** [SRC] (DEEP-I-002.discovery.md:L677| unit=I-54): [E][integration] Create `crates/tools/src/structural_edit.rs`, register it in the tool catalog, and route it through the edit-gate path. Anchors: lines 38-40.
- **I-002/I002-I54-022** [SRC] (DEEP-I-002.discovery.md:L678| unit=I-54 SURFACE): [E][UI] Existing tool-call drawer rows show preview and apply, including the operator-visible diff; no new component is added. Anchors: lines 41-42.
- **I-002/I002-I54-023** [SRC] (DEEP-I-002.discovery.md:L679| unit=I-54): [E][symbols] Produce `tools::structural_edit`, the staleness guard, and `StructuralEditApplied` rows. Anchors: lines 44-46.
- **I-002/I002-I54-024** [SRC] (DEEP-I-002.discovery.md:L680| unit=I-54): [E][integration] Consume the single knowledge parser version, F-1 PreToolUse edit gates, and the tool registry. Anchors: lines 47-48.
- **I-002/I002-I54-025** [SRC] (DEEP-I-002.discovery.md:L681| unit=I-54): [E][reachability] Production reachability is a live agent invoking search -> preview -> apply on a real crate; contracts alone do not satisfy R7. Anchors: lines 49-50.
- **I-002/I002-I54-026** [SRC] (DEEP-I-002.discovery.md:L682| unit=I-54): [E][failure] Parse failure returns typed `ParseFailed`; regex/string-replace fallback is forbidden. Anchors: lines 51-53.
- **I-002/I002-I54-027** [SRC] (DEEP-I-002.discovery.md:L683| unit=I-54): [E][security] Edit-gate bypass is forbidden. Anchor: line 53.
- **I-002/I002-I54-028** [SRC] (DEEP-I-002.discovery.md:L684| unit=I-54): [E][implementation] Production paths forbid `.unwrap()`/`.expect()`. Anchor: line 53.
- **I-002/I002-I54-029** [SRC] (DEEP-I-002.discovery.md:L685| unit=I-54): [E][trigger] The live fixture structurally renames a function and its call sites across a fixture crate. Anchors: lines 54-55.
- **I-002/I002-I54-030** [SRC] (DEEP-I-002.discovery.md:L686| unit=I-54): [E][observability] `Matched` records match count, `Previewed` records the diff hash, and `Applied` records per-file results. Anchors: lines 56-57.
- **I-002/I002-I54-031** [SRC] (DEEP-I-002.discovery.md:L687| unit=I-54): [E][failure] A mid-flight external file change emits `RefusedStale`. Anchor: line 57.
- **I-002/I002-I54-032** [SRC] (DEEP-I-002.discovery.md:L688| unit=I-54): [E][acceptance] The edited fixture must compile. Anchors: lines 58-59.
- **I-002/I002-I54-033** [SRC] (DEEP-I-002.discovery.md:L689| unit=I-54): [E][acceptance] Post-apply bytes may touch only previewed spans and must equal the preview diff. Anchors: lines 58-60.
- **I-002/I002-I54-034** [SRC] (DEEP-I-002.discovery.md:L690| unit=I-54): [E][negative-test] A syntactically broken rewrite template fails during preview with `ParseFailed` and writes zero bytes. Anchors: lines 59-60.
- **I-002/I002-I54-035** [SRC] (DEEP-I-002.discovery.md:L691| unit=I-54): [E][broken-arm] Apply without a preview row fails under R1. Anchors: lines 60-61.
- **I-002/I002-I54-036** [SRC] (DEEP-I-002.discovery.md:L692| unit=I-54): [E][schema] Add `StructuralEditApplied { Matched, Previewed, Applied, RefusedStale, ParseFailed }`, baseline +1, with live W-9 reread. Anchors: lines 68-73.
- **I-002/I002-I54-037** [IX] (DEEP-I-002.discovery.md:L693| unit=I-54): [E][interaction] Survivor Directives from I-20 may use structural edit as the fix vehicle; E-52 and impact mapping remain disjoint siblings. Anchors: lines 68-71. I-54 ambiguity/inference register:
- **I-002/I002-I54-038** [AC] (DEEP-I-002.discovery.md:L697| unit=I-54): [A][claim] The premise calls the ergonomics “rename-safe,” but the same file explicitly limits the tool to syntax-only edits and excludes semantic rename; safety means preview/gate/staleness integrity, not type-aware rename correctness. Anchors: lines 6-12, 22-24.
- **I-002/I002-I54-039** [AC] (DEEP-I-002.discovery.md:L698| unit=I-54): [A][API] The required knowledge-crate public parser API is named architecturally but no exact symbol is specified. Anchors: lines 27-29, 38-40.
- **I-002/I002-I54-040** [INF] (DEEP-I-002.discovery.md:L699| unit=I-54): [I][idempotency] Reapplying an already-consumed preview should be refused or produce no unpreviewed changes; this follows from preview-bound bytes/staleness but no explicit preview-consumption policy is named. Anchors: lines 18-24, 51-61.

#### I-002 / unit `I-STEAL` (32 records)

- **I-002/I002-GM-001** [SRC] (DEEP-I-002.discovery.md:L39| unit=I-STEAL): [E][provenance] The map covers Track 20 (`I-45..I-54`), was grepped on 2026-06-10 in session `s-iplan-track20-2026-06-10`, and derives from `docs/.source/STEAL-LIST.md`. Anchor: lines 1-4.
- **I-002/I002-GM-002** [SRC] (DEEP-I-002.discovery.md:L40| unit=I-STEAL SURFACE): [E][evidence] The map says `elai.map` staleness was unknown, so every ELAI anchor was confirmed directly by glob/grep in live source; detail files cite `SL-n` IDs instead of restating facts. Anchor: lines 4-6.
- **I-002/I002-GM-003** [SRC] (DEEP-I-002.discovery.md:L41| unit=I-STEAL): [E][schema][lifecycle] Live constants were `SCHEMA_VERSION = 221` at `ssb.rs:2981` and `TOTAL_VARIANTS = 532` at `action_type_lock.rs:601`; both must be re-read at edit and never hard-coded. Anchor: lines 12-15.
- **I-002/I002-GM-004** [SRC] (DEEP-I-002.discovery.md:L42| unit=I-STEAL): [E][capability][status] `crates/tools/src/bash_validation.rs` exists, while redirect/rewrite absence is a STEAL-LIST assertion that I-46 must re-grep before building. Anchor: lines 17-19.
- **I-002/I002-GM-005** [SRC] (DEEP-I-002.discovery.md:L43| unit=I-STEAL SURFACE): [E][dependency] I-46's mandatory gajae references are `bash-interceptor.ts`, `bash-command-fixup.ts`, and `bash-allowed-prefixes.ts`. Anchor: lines 19-21.
- **I-002/I002-GM-006** [SRC] (DEEP-I-002.discovery.md:L44| unit=I-STEAL): [E][integration][status] `crates/providers/src/oauth.rs` exists and is Anthropic/Claude-plan OAuth; the source reports no other subscription-auth broker under `crates/providers/`. Anchor: lines 23-25.
- **I-002/I002-GM-007** [SRC] (DEEP-I-002.discovery.md:L45| unit=I-STEAL): [E][dependency] The corresponding gajae auth references are `packages/ai/src/auth-broker/`, `auth-gateway/`, and `auth-storage.ts`. Anchor: lines 25-26.
- **I-002/I002-GM-008** [SRC] (DEEP-I-002.discovery.md:L46| unit=I-STEAL): [E][data][status] Model capability/pricing data is scattered across `providers/pricing.rs`, two distinct `vram_tier.rs` modules, and `pricing_rollup.rs`; no single capability catalog exists. Anchor: lines 27-30.
- **I-002/I002-GM-009** [SRC] (DEEP-I-002.discovery.md:L47| unit=I-STEAL): [E][dependency] The catalog reference port names `api-registry.ts`, `models.json`, and `model-manager.ts`. Anchor: lines 29-30.
- **I-002/I002-GM-010** [CON] (DEEP-I-002.discovery.md:L48| unit=I-STEAL): [E][C][status] `rra.rs`, `attestation.rs`, and `rra_cmd.rs` are shipped, correcting the stale G-plan memory that `rra.rs` was absent; I-45 is therefore an extension, not a forward dependency. Anchor: lines 32-35.
- **I-002/I002-GM-011** [SRC] (DEEP-I-002.discovery.md:L49| unit=I-STEAL): [E][dependency] F-4 `stdout_hash` is the adjacent precedent for I-45. Anchor: lines 34-35.
- **I-002/I002-GM-012** [SRC] (DEEP-I-002.discovery.md:L50| unit=I-STEAL SURFACE): [E][status] E-25 edit checkpoints, its CLI, and both named checkpoint contracts are shipped and grep-confirmed. Anchor: lines 37-40.
- **I-002/I002-GM-013** [CON] (DEEP-I-002.discovery.md:L51| unit=I-STEAL): [E][C][scope] H-49 is planned rather than shipped; I-52 activates the shipped E-25 store as an agent tool instead of creating a second snapshot store. Anchor: lines 39-42, 72-74.
- **I-002/I002-GM-014** [SRC] (DEEP-I-002.discovery.md:L52| unit=I-STEAL SURFACE): [E][integration][fallback] If H-49 ships first, I-52 becomes a tool surface over H-49's store and should reuse its `EditHistoryEvent` row. Anchor: lines 40-42.
- **I-002/I002-GM-015** [SRC] (DEEP-I-002.discovery.md:L53| unit=I-STEAL): [E][parser][status] The knowledge crate already pins tree-sitter 0.26 and grammars for Rust, Python, Go, TypeScript, Java, C#, Ruby, JavaScript, and C. Anchor: lines 44-46.
- **I-002/I002-GM-016** [CON] (DEEP-I-002.discovery.md:L54| unit=I-STEAL): [E][C][dependency] The rejection of an `ast-grep` dependency remains binding; I-54 must build only on the shipped grammars. Anchor: lines 45-46, 78-79.
- **I-002/I002-GM-017** [SRC] (DEEP-I-002.discovery.md:L55| unit=I-STEAL): [E][status] No Bedrock or Azure implementation exists in `crates/`; the grep is accepted as high-confidence and same-day. Anchor: lines 48-49.
- **I-002/I002-GM-018** [SRC] (DEEP-I-002.discovery.md:L56| unit=I-STEAL): [E][integration] Existing provider reach comprises the ten named runtime adapters, local serving, and `oauth.rs`. Anchor: lines 49-51.
- **I-002/I002-GM-019** [SRC] (DEEP-I-002.discovery.md:L57| unit=I-STEAL SURFACE): [E][scope] I-49 adds direct clients only for SigV4+EventStream, Azure Responses, and the OpenAI-as-Anthropic shim seam—protocol shapes a CLI bridge cannot supply. Anchor: lines 51-53.
- **I-002/I002-GM-020** [SRC] (DEEP-I-002.discovery.md:L58| unit=I-STEAL): [E][status] No recipe runner exists in `crates/tools/src`; `just`/`makefile`/`recipe` hits were only incidental. Anchor: lines 55-57.
- **I-002/I002-GM-021** [SRC] (DEEP-I-002.discovery.md:L59| unit=I-STEAL): [E][dependency] I-53's gajae references are the recipe directory, `archive-reader.ts`, and `json-tree.ts`. Anchor: lines 56-57.
- **I-002/I002-GM-022** [SRC] (DEEP-I-002.discovery.md:L60| unit=I-STEAL): [E][evidence][constraint] Gajae behavior is medium confidence because file existence was verified but semantics came from repository summaries. Anchor: lines 59-60.
- **I-002/I002-GM-023** [SRC] (DEEP-I-002.discovery.md:L61| unit=I-STEAL): [E][compliance][done-gate] Executors of I-46, I-49, I-50, I-51, and I-53 must read their named gajae sources before execution and rescope if summaries differ; summary-only scope is a W-3 violation. Anchor: lines 60-62.
- **I-002/I002-GM-024** [SRC] (DEEP-I-002.discovery.md:L62| unit=I-STEAL): [E][evidence][constraint] Roach-pi/lazycodex semantics are likewise medium confidence. Anchor: lines 64-65.
- **I-002/I002-GM-025** [SRC] (DEEP-I-002.discovery.md:L63| unit=I-STEAL): [E][compliance] I-45 and I-47 must read the summary-cited roach-pi extension sources before execution. Anchor: lines 64-67.
- **I-002/I002-GM-026** [CON] (DEEP-I-002.discovery.md:L64| unit=I-STEAL): [E][C][architecture] SLC-2 requires a unified capability catalog to wrap—not delete—the measured 8/12/16/24/48GB VRAM-tier plan; simplification/removal needs explicit user approval. Anchor: lines 75-77.
- **I-002/I002-GM-027** [SRC] (DEEP-I-002.discovery.md:L65| unit=I-STEAL): [E][schema] Track 20 plans at most six new variants, from I-45/I-46/I-47/I-50/I-52/I-54; I-48/I-49/I-51/I-53 reuse existing rows. Anchor: lines 82-86.
- **I-002/I002-GM-028** [SRC] (DEEP-I-002.discovery.md:L66| unit=I-STEAL): [E][schema][done-gate] Variant bumps must follow I-0 lockstep discipline. Anchor: lines 84-86.
- **I-002/I002-GM-029** [SRC] (DEEP-I-002.discovery.md:L67| unit=I-STEAL): [E][status] No web verification was required at map time. Anchor: lines 89-92.
- **I-002/I002-GM-030** [SRC] (DEEP-I-002.discovery.md:L68| unit=I-STEAL): [E][deferred][security] I-49 defers current AWS/Azure protocol verification to authoring time; the executor must web-search current docs and anti-slopsquat-check candidate `aws-*` crates before porting. Anchor: lines 91-94. Grounding-map ambiguity/conflict register:
- **I-002/I002-GM-031** [AC] (DEEP-I-002.discovery.md:L72| unit=I-STEAL): [A][status] Several negative findings are source-asserted rather than independently re-grepped by this map (`redirect` absence, non-Anthropic broker absence, and no capability catalog); the relevant executor re-check is part of the requirement. Anchors: lines 17-30.
- **I-002/I002-GM-032** [CON] (DEEP-I-002.discovery.md:L73| unit=I-STEAL): [C][status] The shipped RRA and E-25 checkpoint facts supersede older “absent/planned-only” memories and change I-45/I-52 from greenfield builds to extension/activation. Anchors: lines 32-42, 72-74.

#### I-002 / unit `IX` (20 records)

- **I-002/I002-X-001** [IX] (DEEP-I-002.discovery.md:L705| unit=IX): [E][context/search] I-5 is a Wave-2 production dependency of I-4: each best-of-N expansion calls the pristine-context builder, while tests alone do not establish reachability. Sources: I-0 lines 273-277; I-5 lines 69-72, 108-113.
- **I-002/I002-X-002** [IX] (DEEP-I-002.discovery.md:L706| unit=IX): [E][context/cache] I-5 must reuse H-6/H-50 context machinery and simultaneously preserve the `update_breakpoints` prefix invariant that I-10 guards. Sources: I-5 lines 24-33, 64-68; I-10 lines 58-66.
- **I-002/I002-X-003** [IX] (DEEP-I-002.discovery.md:L707| unit=IX SURFACE): [E][measurement] E-19 is the I-5 same-choice A/B surface; H-9's hit monitor is I-10 planner input; both convert a design claim into journal-derived evidence before rollout. Sources: I-5 lines 32-44; I-10 lines 92-97.
- **I-002/I002-X-004** [IX] (DEEP-I-002.discovery.md:L708| unit=IX): [E][search/security] I-29 can enter production only through search/task dispatch and must route merged drafts through I-17 then I-20 without a special verdict path. Sources: I-0 lines 309-322, 347-349; I-29 lines 67-75.
- **I-002/I002-X-005** [IX] (DEEP-I-002.discovery.md:L709| unit=IX): [E][rollout] I-29 uses the existing `darwinian-evolve` allowlist/kill-switch precedent, satisfying the index's W-2 exception for an experimental lane while default-on waits for GD-4 evidence. Sources: I-0 lines 194-195; I-29 lines 26-28, 44, 97.
- **I-002/I002-X-006** [IX] (DEEP-I-002.discovery.md:L710| unit=IX): [E][tool-security] I-46 runs before bash validation, then leaves E-30 argv checks downstream; I-35 sequence enforcement is a separate later layer. Sources: I-0 lines 370-372, 407-409; I-46 lines 24-29, 41-45, 64-66.
- **I-002/I002-X-007** [IX] (DEEP-I-002.discovery.md:L711| unit=IX SURFACE): [E][tool-discovery] I-46 obtains substitute targets from the registered tool surface/H-4, and I-53 later supplies recipe targets; I-46 does not invent or execute the target. Sources: I-46 lines 41-45, 64-66; I-0 lines 428-430.
- **I-002/I002-X-008** [IX] (DEEP-I-002.discovery.md:L712| unit=IX): [E][edit-security] I-54 search is read-only, preview is write-free, and apply reuses F-1 PreToolUse edit gates; I-20 Directives may select structural edit as their fix vehicle. Sources: I-54 lines 17-24, 44-50, 68-71.
- **I-002/I002-X-009** [IX] (DEEP-I-002.discovery.md:L713| unit=IX): [E][parser-ownership] The knowledge crate is the single tree-sitter version/grammar owner; the tools crate consumes a public API and may not add an ast-grep or second tree-sitter dependency. Sources: grounding map lines 44-46, 78-79; I-54 lines 26-29, 38-53.
- **I-002/I002-X-010** [IX] (DEEP-I-002.discovery.md:L714| unit=IX): [E][schema] All five detailed sessions add one Pattern-2 action variant, so actual shipping order must reconcile the shared 221/532 baseline rather than independently adding “baseline +1.” Sources: I-0 lines 438-453, 473, 491, 501-503.
- **I-002/I002-X-011** [IX] (DEEP-I-002.discovery.md:L715| unit=IX): [E][evidence] Grounding-map “shipped/absent” facts and detail-file plan assertions never waive W-7/W-9 live rechecks. Sources: grounding map lines 5-6, 12-21; I-0 lines 192-204.
- **I-002/I002-X-012** [IX] (DEEP-I-002.discovery.md:L716| unit=IX): [E][permissions] I-29's client is blocked on dependency approval; I-54 explicitly avoids a dependency; I-46 depends only on shipped tables/validation. This produces three distinct dependency postures despite all being ports/builds. Sources: I-0 lines 538-544; I-29 lines 49-53; I-46 lines 31-38; I-54 lines 22-29.
- **I-002/I002-X-013** [IX] (DEEP-I-002.discovery.md:L720| unit=IX SURFACE): [I][failure-propagation] A broken I-5 prefix-preservation implementation can surface as an I-10 prefix-diff/guard trip; the shared `update_breakpoints` seam makes this a plausible cross-feature diagnostic, though no direct event join is specified. Sources: I-5 lines 24-27, 64-66; I-10 lines 60-66, 80-83.
- **I-002/I002-X-014** [IX] (DEEP-I-002.discovery.md:L721| unit=IX): [I][determinism] I-29's content-hash merge and I-54's preview diff hash both bind later mutation to earlier bytes, but they are separate mechanisms and share no stated hash schema. Sources: I-29 lines 21-24, 88-91; I-54 lines 18-20, 54-60.
- **I-002/I002-X-015** [IX] (DEEP-I-002.discovery.md:L722| unit=IX): [I][observability] Because I-46 refuses shell grep and returns a registered search call, subsequent evidence should appear under the substitute tool's normal rows in addition to `BashRedirected`; the exact correlation field is unspecified. Sources: I-46 lines 15-22, 41-57.
- **I-002/I002-X-016** [IX] (DEEP-I-002.discovery.md:L726| unit=IX): [A][status] The corpus was authored against 2026-06-10 live state; no statement in this chunk proves the same modules/constants still hold at implementation time. W-7/W-9 explicitly demand reread.
- **I-002/I002-X-017** [IX] (DEEP-I-002.discovery.md:L727| unit=IX): [A][rollout] I-29 says both “at least three runs” and “three or fewer are insufficient”; the threshold wording conflicts and needs resolution before promotion.
- **I-002/I002-X-018** [IX] (DEEP-I-002.discovery.md:L728| unit=IX SURFACE): [A][UI] I-5's tile is deferred/wire-dark, I-10's exact cost-panel component name is plan-asserted, and I-46/I-54 reuse an existing drawer; only the interaction contract is stable, not every current frontend symbol.
- **I-002/I002-X-019** [IX] (DEEP-I-002.discovery.md:L729| unit=IX): [A][idempotency] The corpus is explicit about deterministic replay, CRDT merge, atomic file application, and stale preview rejection, but it does not define replay deduplication keys or single-consumption semantics for I-29/I-54 operations.
- **I-002/I002-X-020** [IX] (DEEP-I-002.discovery.md:L730| unit=IX SURFACE): [C][summary-scope] High-level phrases such as “adds a diffusion LLM” and “adds a vividness tile” overstate Tier-1 delivery unless read with the detail files: both live dLLM access and the I-5 tile are gated/deferred.

#### I-002 / unit `SD` (46 records)

- **I-002/I002-SD-001** [SD] (DEEP-I-002.discovery.md:L736| unit=SD): [addition][scope] The exact DEEP-I-002 subset is now integrity-verified and fully read: one Track-20 grounding map, I-0, and I-5/I-10/I-29/I-46/I-54—1,169 lines/99,250 bytes. The summary explicitly left deep sources uninspected.
- **I-002/I002-SD-002** [SD] (DEEP-I-002.discovery.md:L737| unit=SD): [addition][evidence] The auxiliary map is direct evidence for shipped/absent seams, mandatory competitor-source reads, SLC corrections, and the dated live constants; it is not merely an inventory row. Deep anchors: grounding map lines 1-94.
- **I-002/I002-SD-003** [SD] (DEEP-I-002.discovery.md:L738| unit=SD): [addition][workflow] I-0 specifies a seven-step predecessor-reading order and mandatory full-form expansion for L/XL sessions. Deep anchors: I-0 lines 58-74.
- **I-002/I002-SD-004** [SD] (DEEP-I-002.discovery.md:L739| unit=SD): [addition][status] I-0's session log is empty, and DONE has a seven-part gate including delegated memory writing; this prevents treating planned features as current implementation status. Deep anchors: I-0 lines 184-187, 251-258, 590-593.
- **I-002/I002-SD-005** [SD] (DEEP-I-002.discovery.md:L740| unit=SD): [addition][flags] W-2's terminal-gate rule is more exact: new I gates ship default-on behind disable flags, while I-29/I-31 reuse existing allowlisted kill-switch lanes. Deep anchors: I-0 lines 192-204.
- **I-002/I002-SD-006** [SD] (DEEP-I-002.discovery.md:L741| unit=SD): [addition][evidence] W-7/W-9 require both execution and audit re-greps plus live schema recounts, qualifying every dated “shipped” and 221/532 claim. Deep anchors: I-0 lines 200-204, 438-441.
- **I-002/I002-SD-007** [SD] (DEEP-I-002.discovery.md:L742| unit=SD): [addition][lifecycle] ID-3 requires reversible receipts for every context-to-wire transform and exact reconstruction of what the model saw. Deep anchors: I-0 lines 231-234.
- **I-002/I002-SD-008** [SD] (DEEP-I-002.discovery.md:L743| unit=SD): [addition][schema] I-0 supplies a complete 54-session action-type ledger, including exact Kind enums, reuse decisions, and two conditional decisions (I-15/I-52). Deep anchors: I-0 lines 443-503.
- **I-002/I002-SD-009** [SD] (DEEP-I-002.discovery.md:L744| unit=SD): [addition][sequencing] I-0 records five exact waves and additional dependency order, including I-4 behind I-17/I-20 and provider/auth work behind router/catalog substrate. Deep anchors: I-0 lines 505-533.
- **I-002/I002-SD-010** [SD] (DEEP-I-002.discovery.md:L745| unit=SD): [addition][implementation] The deep index adds production coding constraints, auditor/test boundaries, dependency approval, no-removal permission, decision-doc, and commit-cadence rules. Deep anchors: I-0 lines 535-546.
- **I-002/I002-SD-011** [SD] (DEEP-I-002.discovery.md:L746| unit=SD): [addition][compatibility] The index adds explicit sub-second RPM jittering, cache-entry order, constrained-decoding attack posture, action-stream-scoped reproducibility, and review-time—not generation-time—product focus. Deep anchors: I-0 lines 549-578.
- **I-002/I002-SD-012** [SD] (DEEP-I-002.discovery.md:L747| unit=SD SURFACE): [correction][I-5 UI] The summary says I-5 adds a ReasoningDrawer tile; the detail file makes it Tier 2, wire-dark, and W-6-justifiably absent in v1 until at least three consistent A/B runs and GD-4 evidence. Deep anchors: I-5 lines 47-50, 57-58, 81-83.
- **I-002/I002-SD-013** [SD] (DEEP-I-002.discovery.md:L748| unit=SD): [addition][I-5 scope] Tier 1's exact deliverables are `pristine_path_filter.rs`, the three-Kind `ContextPruned` row, sampled dual-arm probe wiring, I-4 integration, and an I-5 decision document. Deep anchors: I-5 lines 37-46.
- **I-002/I002-SD-014** [SD] (DEEP-I-002.discovery.md:L749| unit=SD): [addition][I-5 sampling] The A/B runs only on a sampled fraction of branch expansions; it is not a mandatory dual generation on every branch. Deep anchors: I-5 lines 43-44.
- **I-002/I002-SD-015** [SD] (DEEP-I-002.discovery.md:L750| unit=SD): [addition][I-5 non-goals] Truncation is rejected separately from raw-trace carrying, and a new masking layer is forbidden. Deep anchors: I-5 lines 18-21, 73-80.
- **I-002/I002-SD-016** [SD] (DEEP-I-002.discovery.md:L751| unit=SD): [addition][I-5 status] H-6/H-50/E-19 are plan-asserted/re-grep dependencies rather than current-state proof in this file. Deep anchors: I-5 lines 24-35, 94-106.
- **I-002/I002-SD-017** [SD] (DEEP-I-002.discovery.md:L752| unit=SD): [addition][I-10 rate-limit] I-10 includes `max_tokens_clamp` for schema-bound scoring calls, a material Tier-1 capability absent from the oriented I-10 summary ledger. Deep anchors: I-10 lines 21-22, 36-37, 53-57.
- **I-002/I002-SD-018** [SD] (DEEP-I-002.discovery.md:L753| unit=SD): [addition][I-10 ordering] Long-TTL cache entries must precede short-TTL entries in each request. Deep anchors: I-10 lines 16-18.
- **I-002/I002-SD-019** [SD] (DEEP-I-002.discovery.md:L754| unit=SD): [addition][I-10 economics] The planner's dated write-premium/breakeven defaults and mandatory pricing reverification are explicit. Deep anchors: I-10 lines 19-20, 85-90.
- **I-002/I002-SD-020** [SD] (DEEP-I-002.discovery.md:L755| unit=SD): [addition][I-10 rollout] Global default-on waits for three real runs, and automated price reverification is deferred as monitoring rather than a gate. Deep anchors: I-10 lines 43-45.
- **I-002/I002-SD-021** [SD] (DEEP-I-002.discovery.md:L756| unit=SD): [addition][I-10 observability] `TtlPlanned` carries `ttl_seconds` and `breakeven_reads`; the acceptance fixture requires `cache_read_input_tokens > 0` on turn 2. Deep anchors: I-10 lines 80-83.
- **I-002/I002-SD-022** [SD] (DEEP-I-002.discovery.md:L757| unit=SD): [addition][I-10 status] Cache plumbing existed but Anthropic markers remained `None`, while the planner/selector/guard were absent at authoring and remain W-7 recheck facts. Deep anchors: I-10 lines 28-33.
- **I-002/I002-SD-023** [SD] (DEEP-I-002.discovery.md:L758| unit=SD): [correction][I-29 scope] The summary's broad “adds a diffusion LLM” is not Tier-1 delivery: only the in-process merge skeleton, allowlist row, contract, and decision doc ship now; client/network/default-on/volume test are deferred and approval-gated. Deep anchors: I-29 lines 40-53.
- **I-002/I002-SD-024** [SD] (DEEP-I-002.discovery.md:L759| unit=SD): [correction][I-29 implementation] The summary leaves Automerge versus hand-rolled merge open; the detail file selects a roughly 100-LOC dependency-free in-process skeleton with deterministic content-hash resolution. Deep anchors: I-29 lines 42-43, 55-65, 98-100.
- **I-002/I002-SD-025** [SD] (DEEP-I-002.discovery.md:L760| unit=SD): [addition][I-29 config] The allowlist row has required owner, expiry, promotion plan, and smoke-test fields. Deep anchor: I-29 line 44.
- **I-002/I002-SD-026** [SD] (DEEP-I-002.discovery.md:L761| unit=SD): [addition][I-29 concurrency] Parallel patches append to a shared CRDT log and conflicts resolve at character/line level with content-hash last-write-wins, never an LLM. Deep anchors: I-29 lines 21-24, 77-83.
- **I-002/I002-SD-027** [SD] (DEEP-I-002.discovery.md:L762| unit=SD): [addition][I-29 reachability] Tier-1 contract calls do not satisfy R7; production reachability exists only after the Tier-2 adapter path is live. Deep anchors: I-29 lines 73-75.
- **I-002/I002-SD-028** [SD] (DEEP-I-002.discovery.md:L763| unit=SD): [conflict][I-29 threshold] `>=3` real runs and “<=3 insufficient” are inconsistent threshold wordings and require resolution. Deep anchors: I-29 lines 26-28, 38, 52, 81.
- **I-002/I002-SD-029** [SD] (DEEP-I-002.discovery.md:L764| unit=SD SURFACE): [addition][I-29 event] The full event enum/badge includes `DraftAccepted`, which the oriented summary's observability bullet omitted while naming only veto and merge. Deep anchors: I-29 lines 85-90, 99.
- **I-002/I002-SD-030** [SD] (DEEP-I-002.discovery.md:L765| unit=SD): [addition][I-46 compliance] Execution must first read three named gajae source files; medium-confidence summary semantics are not enough. Deep anchors: I-46 lines 12-14, 59-61.
- **I-002/I002-SD-031** [SD] (DEEP-I-002.discovery.md:L766| unit=SD): [addition][I-46 parsing] Fixups must be semantics-preserving; argv parsing is mandatory; regex-only classification and bypass flags are banned. Deep anchors: I-46 lines 17-18, 48-50.
- **I-002/I002-SD-032** [SD] (DEEP-I-002.discovery.md:L767| unit=SD): [addition][I-46 deferred] Recipe redirects and per-project prefix configuration are explicitly deferred. Deep anchor: I-46 line 34.
- **I-002/I002-SD-033** [SD] (DEEP-I-002.discovery.md:L768| unit=SD): [addition][I-46 events] Detail acceptance requires `FixedUp` and `PrefixBlocked` rows as well as `Redirected`, plus no-process and allowed-prefix negative arms. Deep anchors: I-46 lines 51-57, 67-68.
- **I-002/I002-SD-034** [SD] (DEEP-I-002.discovery.md:L769| unit=SD): [addition][I-46 integrity] Fixup/prefix tables are R8-frozen and a run may not alter its own redirect policy. Deep anchors: I-46 lines 36-38.
- **I-002/I002-SD-035** [SD] (DEEP-I-002.discovery.md:L770| unit=SD): [addition][I-54 compliance] Execution must read the two named gajae AST tools for interface semantics while retaining a different engine. Deep anchors: I-54 lines 14-16.
- **I-002/I002-SD-036** [SD] (DEEP-I-002.discovery.md:L771| unit=SD): [addition][I-54 schema] Search results include file/line/capture bindings and rewrites are capture-template based. Deep anchors: I-54 lines 9-12, 17-19.
- **I-002/I002-SD-037** [SD] (DEEP-I-002.discovery.md:L772| unit=SD): [addition][I-54 scope] Cross-language rewrites in one call are forbidden; each query is per-language. Deep anchors: I-54 lines 22-24.
- **I-002/I002-SD-038** [SD] (DEEP-I-002.discovery.md:L773| unit=SD): [addition][I-54 deferred] Six remaining named grammars and a rewrite-template library are deferred. Deep anchors: I-54 lines 26-36.
- **I-002/I002-SD-039** [SD] (DEEP-I-002.discovery.md:L774| unit=SD): [addition][I-54 events] The complete event enum adds `RefusedStale` and `ParseFailed` to Matched/Previewed/Applied. Deep anchors: I-54 lines 54-57, 68-73.
- **I-002/I002-SD-040** [SD] (DEEP-I-002.discovery.md:L775| unit=SD): [addition][I-54 failure] Parse failure is typed and may never fall back to regex/string replacement; tools may not pin a second tree-sitter version. Deep anchors: I-54 lines 26-29, 51-53.
- **I-002/I002-SD-041** [SD] (DEEP-I-002.discovery.md:L776| unit=SD): [ambiguity][I-54 claim] “Rename-safe” is limited by the explicit syntax-only/non-semantic scope; no type-aware rename guarantee is earned. Deep anchors: I-54 lines 6-12, 22-24.
- **I-002/I002-SD-042** [SD] (DEEP-I-002.discovery.md:L777| unit=SD): [addition][grounding status] The map confirms shipped RRA and checkpoint substrate, correcting stale absence/build assumptions and making I-45 an extension and I-52 an activation. Deep anchors: grounding map lines 32-42, 72-74.
- **I-002/I002-SD-043** [SD] (DEEP-I-002.discovery.md:L778| unit=SD): [addition][grounding gaps] The map records Anthropic-only OAuth, scattered catalog data with duplicate VRAM-tier module names, no Bedrock/Azure, and no recipe runner. Deep anchors: grounding map lines 23-30, 48-57.
- **I-002/I002-SD-044** [SD] (DEEP-I-002.discovery.md:L779| unit=SD): [addition][source-read gate] Named Track-20 executors have binding direct-source-read obligations for both gajae and roach-pi/lazycodex references. Deep anchors: grounding map lines 59-67.
- **I-002/I002-SD-045** [SD] (DEEP-I-002.discovery.md:L780| unit=SD): [addition][schema] Track 20's maximum is six new variants; four sessions reuse existing rows. Deep anchors: grounding map lines 81-86.
- **I-002/I002-SD-046** [SD] (DEEP-I-002.discovery.md:L781| unit=SD): [addition][I-49 deferred verification] AWS/Azure protocol and candidate-crate verification is deliberately deferred to I-49 authoring time. Deep anchors: grounding map lines 89-94.

### Ledger I-003 — I-1, I-4, I-9, I-20, I-32, I-33, I-38, I-41, I-50, I-52

Source: `docs/thefinalglance/feature-discovery/deep/DEEP-I-003.discovery.md`
Extracted atoms: **415**

#### I-003 / unit `I-1` (33 records)

- **I-003/DI003-I01-001** [SRC] (DEEP-I-003.discovery.md:L50| unit=I-1): [E][status] Track 12; CANT-WRECK; STRONG; BUILD; effort L; Wave 2; binds ID-2. Anchor: I-1_directive_supervisor_enforcement_plane.md:3.
- **I-003/DI003-I01-002** [SRC] (DEEP-I-003.discovery.md:L52| unit=I-1): [E][gap/security] Existing PreToolUse enforcement shares the generator process; no Directive type or structural supervisor boundary exists. Anchor: I-1_directive_supervisor_enforcement_plane.md:4-9.
- **I-003/DI003-I01-003** [SRC] (DEEP-I-003.discovery.md:L54| unit=I-1): [E][schema] Tier-1 Directive is a pure value containing tool_name, args, intent_id, and initiator. Anchor: I-1_directive_supervisor_enforcement_plane.md:8-9,35.
- **I-003/DI003-I01-004** [SRC] (DEEP-I-003.discovery.md:L56| unit=I-1): [E][invariant] The generator emits Directives and must not directly call a side-effect sink. Anchor: I-1_directive_supervisor_enforcement_plane.md:12-16.
- **I-003/DI003-I01-005** [SRC] (DEEP-I-003.discovery.md:L58| unit=I-1): [E][security] The target architecture makes a Supervisor the sole side-effect executor and owner of credentials and egress paths. Anchor: I-1_directive_supervisor_enforcement_plane.md:8-9.
- **I-003/DI003-I01-006** [SRC] (DEEP-I-003.discovery.md:L60| unit=I-1): [E][workflow] Supervisor execution runs every applicable gate before the side effect and executes only on PASS. Anchor: I-1_directive_supervisor_enforcement_plane.md:13-16.
- **I-003/DI003-I01-007** [SRC] (DEEP-I-003.discovery.md:L62| unit=I-1): [E][integration] The pre-check chain composes F-1 VibeGate, E-30 argv allowlist, and G-3 blast-radius rather than reimplementing them. Anchor: I-1_directive_supervisor_enforcement_plane.md:14-15,27-28.
- **I-003/DI003-I01-008** [SRC] (DEEP-I-003.discovery.md:L64| unit=I-1): [E][recovery] A Supervisor crash invokes E-44 saga rollback to undo partial state. Anchor: I-1_directive_supervisor_enforcement_plane.md:15-16,38.
- **I-003/DI003-I01-009** [SRC] (DEEP-I-003.discovery.md:L66| unit=I-1): [E][test/security] A contract must prove that a generator-authored function cannot satisfy a gate or reach a sink/egress path. Anchor: I-1_directive_supervisor_enforcement_plane.md:16-17,39.
- **I-003/DI003-I01-010** [SRC] (DEEP-I-003.discovery.md:L68| unit=I-1): [E][non-goal] A mere async-task/thread split is declined because it retains the heap and fault domain. Anchor: I-1_directive_supervisor_enforcement_plane.md:17-18.
- **I-003/DI003-I01-011** [SRC] (DEEP-I-003.discovery.md:L70| unit=I-1): [E][non-goal] Capability passing without OS isolation is declined because the handle could leak back to the generator. Anchor: I-1_directive_supervisor_enforcement_plane.md:18-19.
- **I-003/DI003-I01-012** [SRC] (DEEP-I-003.discovery.md:L72| unit=I-1): [E][dependency] The existing Phase 4A IPC seam is the required generator/Supervisor channel; a second process seam is forbidden. Anchor: I-1_directive_supervisor_enforcement_plane.md:26,66.
- **I-003/DI003-I01-013** [SRC] (DEEP-I-003.discovery.md:L74| unit=I-1): [E][tier-1] Ship Directive, a Supervisor trait/process abstraction, and an in-process implementation sufficient for tests. Anchor: I-1_directive_supervisor_enforcement_plane.md:34-38,48.
- **I-003/DI003-I01-014** [SRC] (DEEP-I-003.discovery.md:L76| unit=I-1): [E][observability] Register DirectiveExecuted with CompensationFired; clean execution also produces Succeeded and rejected execution produces Refused. Anchor: I-1_directive_supervisor_enforcement_plane.md:40,62-63,88-91.
- **I-003/DI003-I01-015** [SRC] (DEEP-I-003.discovery.md:L79| unit=I-1): [E][deferred] A distinct OS PID and IPC channel are Tier-2 and process/network-launch gated. Anchor: I-1_directive_supervisor_enforcement_plane.md:42-44.
- **I-003/DI003-I01-016** [SRC] (DEEP-I-003.discovery.md:L81| unit=I-1 SURFACE): [E][deferred/security] The credential vault/key-management surface is a separate Tier-2 session. Anchor: I-1_directive_supervisor_enforcement_plane.md:42-45.
- **I-003/DI003-I01-017** [SRC] (DEEP-I-003.discovery.md:L83| unit=I-1): [E][deferred/integration] A cloud-sandbox adapter may reuse the trait but is optional and never a dependency. Anchor: I-1_directive_supervisor_enforcement_plane.md:45.
- **I-003/DI003-I01-018** [SRC] (DEEP-I-003.discovery.md:L85| unit=I-1 SURFACE): [E][surface] Create directive.rs, the I-1 contract, and I-1-decisions.md; modify task_runner dispatch and hook_dispatcher. Anchor: I-1_directive_supervisor_enforcement_plane.md:47-54.
- **I-003/DI003-I01-019** [SRC] (DEEP-I-003.discovery.md:L87| unit=I-1): [E][identifiers] Produced identifiers are Directive, Supervisor, DirectiveExecuted, and i1_directive_supervisor_contract.rs. Anchor: I-1_directive_supervisor_enforcement_plane.md:58-64.
- **I-003/DI003-I01-020** [SRC] (DEEP-I-003.discovery.md:L90| unit=I-1): [E][dependency] The Supervisor consumes E-44 rollback and safety Provenance in addition to the three gates. Anchor: I-1_directive_supervisor_enforcement_plane.md:65-71.
- **I-003/DI003-I01-021** [SRC] (DEEP-I-003.discovery.md:L92| unit=I-1): [E][reachability] Production reachability must be task_runner → execute_directive and hook_dispatcher → run_gates; test-only callers do not satisfy R7. Anchor: I-1_directive_supervisor_enforcement_plane.md:72-75.
- **I-003/DI003-I01-022** [SRC] (DEEP-I-003.discovery.md:L95| unit=I-1): [E][constraint] Production paths ban unwrap/expect, blake2/sha2, fabricated measurements, and invented file/line evidence. Anchor: I-1_directive_supervisor_enforcement_plane.md:76-80.
- **I-003/DI003-I01-023** [SRC] (DEEP-I-003.discovery.md:L97| unit=I-1): [E][constraint/status] Tier-1 must not claim OS process separation, and may not reimplement F-1/E-30/G-3 logic. Anchor: I-1_directive_supervisor_enforcement_plane.md:81-82.
- **I-003/DI003-I01-024** [SRC] (DEEP-I-003.discovery.md:L99| unit=I-1 SURFACE): [E][UI] DirectiveExecuted rows render in an existing SSB-backed drawer; no new Tier-1 panel is required. Anchor: I-1_directive_supervisor_enforcement_plane.md:55-56,83-86.
- **I-003/DI003-I01-025** [SRC] (DEEP-I-003.discovery.md:L101| unit=I-1): [E][acceptance] Injected OOM/panic mid-tool must emit CompensationFired and restore pre-tool bytes exactly; clean execution emits Succeeded. Anchor: I-1_directive_supervisor_enforcement_plane.md:87-94.
- **I-003/DI003-I01-026** [SRC] (DEEP-I-003.discovery.md:L104| unit=I-1): [E][failure/evidence] A direct-sink attempt emits Refused with the full Directive logged; a missing required SSB row is failure. Anchor: I-1_directive_supervisor_enforcement_plane.md:89-94.
- **I-003/DI003-I01-027** [SRC] (DEEP-I-003.discovery.md:L106| unit=I-1): [E][status] DirectiveExecuted is specified as baseline+1 from LIVE 221/532 on 2026-06-10 and must be re-read at edit. Anchor: I-1_directive_supervisor_enforcement_plane.md:62-63,102-103.
- **I-003/DI003-I01-028** [SRC] (DEEP-I-003.discovery.md:L108| unit=I-1): [E][dependency] I-2 supplies the workspace boundary that the Supervisor enforces. Anchor: I-1_directive_supervisor_enforcement_plane.md:113.
- **I-003/DI003-I01-029** [SRC] (DEEP-I-003.discovery.md:L110| unit=I-1): [E][dependency] I-1 is the Supervisor-side prerequisite for I-35 sequence rules and the I-31 reward endpoint. Anchor: I-1_directive_supervisor_enforcement_plane.md:114-115.
- **I-003/DI003-I01-030** [SRC] (DEEP-I-003.discovery.md:L112| unit=I-1): [E][done-gate] Full-form expansion is mandatory before EXECUTING. Anchor: I-1_directive_supervisor_enforcement_plane.md:105.
- **I-003/DI003-I01-031** [CON] (DEEP-I-003.discovery.md:L114| unit=I-1): [C][tier conflict] The architectural premise says process-separated Supervisor and Supervisor-held credentials, but Revised Scope ships an in-process stub and defers OS separation and the credential vault. Tier-1 claims must follow the latter. Anchors: I-1_directive_supervisor_enforcement_plane.md:8-9,13,33-45,81.
- **I-003/DI003-I01-032** [AC] (DEEP-I-003.discovery.md:L118| unit=I-1 SURFACE): [A][UI] The exact existing drawer is intentionally unresolved until EXECUTING. Anchor: I-1_directive_supervisor_enforcement_plane.md:55-56.
- **I-003/DI003-I01-033** [INF] (DEEP-I-003.discovery.md:L120| unit=I-1): [I][privacy] Because refusal logging includes the full Directive and args, downstream redaction requirements may be needed; this source does not specify them. Basis: I-1_directive_supervisor_enforcement_plane.md:35,89-91.

#### I-003 / unit `I-4` (61 records)

- **I-003/DI003-I04-001** [SRC] (DEEP-I-003.discovery.md:L129| unit=I-4): [E][status] Track 13; SMARTER-ORCH; STRONG; BUILD; effort XL; Wave 4; depends on I-2 and I-17; blocks I-22/I-28/I-30; binds ID-1/ID-2/ID-4. Anchor: I-4_evidence_ranked_search_ladder.md:3.
- **I-003/DI003-I04-002** [SRC] (DEEP-I-003.discovery.md:L132| unit=I-4): [E][gap] No public tree search ships; current behavior is wide parallelism plus an LLM-judge selector. Anchor: I-4_evidence_ranked_search_ladder.md:5-9.
- **I-003/DI003-I04-003** [SRC] (DEEP-I-003.discovery.md:L134| unit=I-4): [E][workflow] V1 is best-of-N: generate K divergent candidates, isolate them, evaluate them, rank by receipts, and return a winner. Anchor: I-4_evidence_ranked_search_ladder.md:11-16,41-48.
- **I-003/DI003-I04-004** [SRC] (DEEP-I-003.discovery.md:L136| unit=I-4): [E][concurrency] Each candidate uses an I-2 BranchWorkspace; mutable scratch may never be shared across branches. Anchor: I-4_evidence_ranked_search_ladder.md:13,36-37,70,85-86.
- **I-003/DI003-I04-005** [SRC] (DEEP-I-003.discovery.md:L138| unit=I-4): [E][ranking] Receipt inputs are the I-17 tri-layer verdict, I-20 mutation evidence, and progress score. Anchor: I-4_evidence_ranked_search_ladder.md:13-15,71-72.
- **I-003/DI003-I04-006** [SRC] (DEEP-I-003.discovery.md:L140| unit=I-4): [E][determinism] V1 ranking precedence is tri-layer verdict first, mutation kill-rate second, progress score third. Anchor: I-4_evidence_ranked_search_ladder.md:47.
- **I-003/DI003-I04-007** [SRC] (DEEP-I-003.discovery.md:L142| unit=I-4): [E][observability] The SSB records why each branch won or lost. Anchor: I-4_evidence_ranked_search_ladder.md:14-15.
- **I-003/DI003-I04-008** [SRC] (DEEP-I-003.discovery.md:L144| unit=I-4): [E][workflow/deferred] V2 is a tournament among ranked survivors. Anchor: I-4_evidence_ranked_search_ladder.md:15,49-51.
- **I-003/DI003-I04-009** [SRC] (DEEP-I-003.discovery.md:L146| unit=I-4): [E][workflow/deferred] V3 adds PUCT over SessionDag. Anchor: I-4_evidence_ranked_search_ladder.md:15-16,49-52.
- **I-003/DI003-I04-010** [SRC] (DEEP-I-003.discovery.md:L148| unit=I-4): [E][concurrency/deferred] V3 uses virtual loss for safe parallel expansion. Anchor: I-4_evidence_ranked_search_ladder.md:16.
- **I-003/DI003-I04-011** [SRC] (DEEP-I-003.discovery.md:L150| unit=I-4): [E][generation] C-26 temperature schedules generate the K divergent hypotheses. Anchor: I-4_evidence_ranked_search_ladder.md:16-17,124.
- **I-003/DI003-I04-012** [SRC] (DEEP-I-003.discovery.md:L152| unit=I-4): [E][ownership] The generator is stateless with respect to the search tree; the orchestrator owns it. Anchor: I-4_evidence_ranked_search_ladder.md:6-8,16-17.
- **I-003/DI003-I04-013** [SRC] (DEEP-I-003.discovery.md:L154| unit=I-4): [E][hard-veto] A neural scorer may order survivors but may never resurrect an I-17 HardVeto branch. Anchor: I-4_evidence_ranked_search_ladder.md:17-19,80-83.
- **I-003/DI003-I04-014** [SRC] (DEEP-I-003.discovery.md:L156| unit=I-4): [E][non-goal] LLM-judge-only selection that ignores verifier receipts is banned. Anchor: I-4_evidence_ranked_search_ladder.md:18-20,79-81.
- **I-003/DI003-I04-015** [SRC] (DEEP-I-003.discovery.md:L158| unit=I-4): [E][non-goal] Batch API parallel expansion is offline-only and banned from live search loops. Anchor: I-4_evidence_ranked_search_ladder.md:19-20,84.
- **I-003/DI003-I04-016** [SRC] (DEEP-I-003.discovery.md:L160| unit=I-4): [E][rollout] V1 must demonstrate measured value before v3/tree code ships. Anchor: I-4_evidence_ranked_search_ladder.md:20-22.
- **I-003/DI003-I04-017** [SRC] (DEEP-I-003.discovery.md:L162| unit=I-4): [E][status] No mcts module exists; I-4 is a genuine BUILD. Anchor: I-4_evidence_ranked_search_ladder.md:24-26.
- **I-003/DI003-I04-018** [SRC] (DEEP-I-003.discovery.md:L164| unit=I-4): [E][dependency] Per-branch context rebuilding must preserve ContextSlot and blake3 prefix breakpoints through update_breakpoints/adaptive_reduce. Anchor: I-4_evidence_ranked_search_ladder.md:27-28,74.
- **I-003/DI003-I04-019** [SRC] (DEEP-I-003.discovery.md:L167| unit=I-4): [E][dependency] Existing RelationLibrary, MutationRunner, and the 30-day blake3-keyed EquivalentMutantCache supply verifier receipts. Anchor: I-4_evidence_ranked_search_ladder.md:29-31.
- **I-003/DI003-I04-020** [SRC] (DEEP-I-003.discovery.md:L169| unit=I-4): [E][integration] The ladder mirrors the production_agent_rpc three-arm dispatch shape at baby-step dispatch. Anchor: I-4_evidence_ranked_search_ladder.md:32-33,75.
- **I-003/DI003-I04-021** [SRC] (DEEP-I-003.discovery.md:L171| unit=I-4): [E][recovery] Existing recover_from_checkpoint is a live/tested primitive inherited by the branch store, not a stub to rebuild. Anchor: I-4_evidence_ranked_search_ladder.md:34-35.
- **I-003/DI003-I04-022** [SRC] (DEEP-I-003.discovery.md:L173| unit=I-4): [E][dependency/status] SessionDag/H-49 is the planned branch store and must be re-grepped at execution. Anchor: I-4_evidence_ranked_search_ladder.md:38.
- **I-003/DI003-I04-023** [SRC] (DEEP-I-003.discovery.md:L175| unit=I-4 SURFACE): [E][tier-1/surface] Ship search/ladder.rs, search/mod.rs, best-of-N dispatch, decision doc, and SearchNodeEvent registration. Anchor: I-4_evidence_ranked_search_ladder.md:40-48,54-55.
- **I-003/DI003-I04-024** [SRC] (DEEP-I-003.discovery.md:L177| unit=I-4): [E][schema/observability] SearchNodeEvent kinds are Expanded, Pruned, Promoted, and BudgetCollapsed. Anchor: I-4_evidence_ranked_search_ladder.md:44,68.
- **I-003/DI003-I04-025** [SRC] (DEEP-I-003.discovery.md:L179| unit=I-4): [E][rollout] Linear retry remains until the ladder has a measured first-win on at least three real runs. Anchor: I-4_evidence_ranked_search_ladder.md:45-46.
- **I-003/DI003-I04-026** [SD] (DEEP-I-003.discovery.md:L181| unit=I-4): [E][deferred] Tournament code is gated on a measured V1 pass-rate delta on a real held-out fixture. Anchor: I-4_evidence_ranked_search_ladder.md:49-51.
- **I-003/DI003-I04-027** [SD] (DEEP-I-003.discovery.md:L183| unit=I-4): [E][deferred] MCTS/PUCT code is gated on the V2 delta and may not block Tracks 14/15. Anchor: I-4_evidence_ranked_search_ladder.md:51.
- **I-003/DI003-I04-028** [SRC] (DEEP-I-003.discovery.md:L185| unit=I-4): [E][deferred] Cross-language verifier packs are outside V1. Anchor: I-4_evidence_ranked_search_ladder.md:52.
- **I-003/DI003-I04-029** [SRC] (DEEP-I-003.discovery.md:L187| unit=I-4 SURFACE): [E][UI] H-15 branch-board comparison is the intended SearchNodeEvent consumer, but V1 may ship without that deferred Svelte surface. Anchor: I-4_evidence_ranked_search_ladder.md:60-62,89-92.
- **I-003/DI003-I04-030** [SRC] (DEEP-I-003.discovery.md:L189| unit=I-4): [E][identifiers] Production entry is SearchLadder::run_best_of_n; search/mod.rs reexports it. Anchor: I-4_evidence_ranked_search_ladder.md:64-68.
- **I-003/DI003-I04-031** [SRC] (DEEP-I-003.discovery.md:L191| unit=I-4): [E][dependency] Concrete calls are BranchWorkspace::spawn, TriLayerStack::evaluate, MutationScheduler::score, SessionDag::fork_branch, update_breakpoints, and build_tool_catalog. Anchor: I-4_evidence_ranked_search_ladder.md:69-75.
- **I-003/DI003-I04-032** [SRC] (DEEP-I-003.discovery.md:L194| unit=I-4): [E][reachability] task_runner baby-step dispatch must reach run_best_of_n; contracts-only callers do not satisfy R7. Anchor: I-4_evidence_ranked_search_ladder.md:76-78.
- **I-003/DI003-I04-033** [SRC] (DEEP-I-003.discovery.md:L196| unit=I-4): [E][constraint] Production code bans unwrap/expect, blake2/sha2, and fabricated measurements. Anchor: I-4_evidence_ranked_search_ladder.md:87-88.
- **I-003/DI003-I04-034** [SRC] (DEEP-I-003.discovery.md:L198| unit=I-4): [E][acceptance] The simple_fix fixture must emit at least one Pruned compile-fail branch and one Promoted survivor carrying verifier evidence, all seed-recomputable. Anchor: I-4_evidence_ranked_search_ladder.md:93-97.
- **I-003/DI003-I04-035** [SRC] (DEEP-I-003.discovery.md:L201| unit=I-4): [E][acceptance] The winner merge must be journal-verified as the highest-receipt survivor. Anchor: I-4_evidence_ranked_search_ladder.md:98-99.
- **I-003/DI003-I04-036** [SRC] (DEEP-I-003.discovery.md:L203| unit=I-4): [E][broken-arm] An LLM-judge-only arm choosing a compile-fail branch must exit nonzero with zero Promoted rows. Anchor: I-4_evidence_ranked_search_ladder.md:99-100.
- **I-003/DI003-I04-037** [SRC] (DEEP-I-003.discovery.md:L205| unit=I-4): [E][done-gate] V2/V3 code must be absent from the binary before its gates open. Anchor: I-4_evidence_ranked_search_ladder.md:100-101.
- **I-003/DI003-I04-038** [SRC] (DEEP-I-003.discovery.md:L207| unit=I-4): [E][status] SearchNodeEvent is baseline+1 from LIVE 221/532 on 2026-06-10 and must be re-read at edit. Anchor: I-4_evidence_ranked_search_ladder.md:114-115.
- **I-003/DI003-I04-039** [SRC] (DEEP-I-003.discovery.md:L209| unit=I-4): [E][done-gate] Full-form expansion is mandatory before EXECUTING. Anchor: I-4_evidence_ranked_search_ladder.md:21-22.
- **I-003/DI003-I04-040** [SRC] (DEEP-I-003.discovery.md:L211| unit=I-4): [E][held-out security] The merge gate reruns B_test off-agent in a fresh detached worktree from a pool separate from the executor worktree. Anchor: I-4_evidence_ranked_search_ladder.md:127-141.
- **I-003/DI003-I04-041** [SRC] (DEEP-I-003.discovery.md:L214| unit=I-4): [E][privacy/anti-gaming] The executor sees B_dev only and never touches B_test. Anchor: I-4_evidence_ranked_search_ladder.md:130-133,139-141.
- **I-003/DI003-I04-042** [SRC] (DEEP-I-003.discovery.md:L216| unit=I-4): [E][evidence] LLM-reported scores may be logged but only the independently verified score governs merge. Anchor: I-4_evidence_ranked_search_ladder.md:133-134.
- **I-003/DI003-I04-043** [SRC] (DEEP-I-003.discovery.md:L218| unit=I-4): [E][failure] No improvement against trunk/baseline rejects the candidate with metric-direction awareness. Anchor: I-4_evidence_ranked_search_ladder.md:134-135.
- **I-003/DI003-I04-044** [CON] (DEEP-I-003.discovery.md:L220| unit=I-4): [E][merge lifecycle] Accepted merge uses no-fast-forward; conflicts abort the merge and restore the original checkout. Anchor: I-4_evidence_ranked_search_ladder.md:135-136.
- **I-003/DI003-I04-045** [SRC] (DEEP-I-003.discovery.md:L222| unit=I-4): [E][security] Merge checks protected-path patterns and required outputs. Anchor: I-4_evidence_ranked_search_ladder.md:136-137.
- **I-003/DI003-I04-046** [SRC] (DEEP-I-003.discovery.md:L224| unit=I-4 SURFACE): [E][fail-closed] Missing held-out eval command must BLOCK; there is no fallback to an LLM-reported test score. Anchor: I-4_evidence_ranked_search_ladder.md:143-146.
- **I-003/DI003-I04-047** [SRC] (DEEP-I-003.discovery.md:L226| unit=I-4): [E][integrity] Eval commands belong in a frozen gate manifest and must not be agent-writable state. Anchor: I-4_evidence_ranked_search_ladder.md:147-150.
- **I-003/DI003-I04-048** [SRC] (DEEP-I-003.discovery.md:L228| unit=I-4): [E][determinism] Eval-score parsing is deterministic only; an LLM parser is not evidence. Anchor: I-4_evidence_ranked_search_ladder.md:151-154.
- **I-003/DI003-I04-049** [SRC] (DEEP-I-003.discovery.md:L230| unit=I-4): [E][branch lifecycle] A candidate branch starts from current trunk HEAD so it includes prior merges. Anchor: I-4_evidence_ranked_search_ladder.md:156-160.
- **I-003/DI003-I04-050** [SRC] (DEEP-I-003.discovery.md:L232| unit=I-4): [E][artifact lifecycle] Finalize-commit filters generated artifacts from commit-worthy paths. Anchor: I-4_evidence_ranked_search_ladder.md:161-162.
- **I-003/DI003-I04-051** [SRC] (DEEP-I-003.discovery.md:L234| unit=I-4): [E][branch lifecycle] Removing a candidate worktree preserves its branch for later merge. Anchor: I-4_evidence_ranked_search_ladder.md:163-164.
- **I-003/DI003-I04-052** [SRC] (DEEP-I-003.discovery.md:L236| unit=I-4): [E][concurrency] Merge-eval and executor worktrees use distinct pools. Anchor: I-4_evidence_ranked_search_ladder.md:165-167.
- **I-003/DI003-I04-053** [SRC] (DEEP-I-003.discovery.md:L238| unit=I-4): [E][recovery/concurrency] Stale worktrees are force-removed at creation and naming collisions receive a timestamp suffix. Anchor: I-4_evidence_ranked_search_ladder.md:168-169.
- **I-003/DI003-I04-054** [SRC] (DEEP-I-003.discovery.md:L240| unit=I-4 SURFACE): [E][platform/non-goal] Arbor has no pool and is Unix-only; ELAI keeps I-2's warm, Windows-safe pool instead of copying that implementation. Anchor: I-4_evidence_ranked_search_ladder.md:171-173.
- **I-003/DI003-I04-055** [SRC] (DEEP-I-003.discovery.md:L243| unit=I-4 SURFACE): [E][non-goal] Do not port persistent ReAct, a 500-turn loop, premature-stop nudges, or LLM-selected nodes; ELAI FSM plus deterministic leaf selection replaces them. Anchor: I-4_evidence_ranked_search_ladder.md:175-179.
- **I-003/DI003-I04-056** [SRC] (DEEP-I-003.discovery.md:L246| unit=I-4): [E][advisory] High-impact/evidenced/recoverable/diverse SELECT guidance may be used only as an advisory tie-break; deterministic scoring remains authoritative. Anchor: I-4_evidence_ranked_search_ladder.md:180-183.
- **I-003/DI003-I04-057** [SRC] (DEEP-I-003.discovery.md:L249| unit=I-4): [E][prompt] B_dev iteration/B_test milestone and divergence-investigation guidance may enter the executor/eval prompt. Anchor: I-4_evidence_ranked_search_ladder.md:184-185.
- **I-003/DI003-I04-058** [SRC] (DEEP-I-003.discovery.md:L251| unit=I-4): [E][prompt/dependency] IDEATE prompt material is cloud-lane-only; executor method text is diffed and only deltas are stolen. Anchor: I-4_evidence_ranked_search_ladder.md:186-189.
- **I-003/DI003-I04-059** [AC] (DEEP-I-003.discovery.md:L253| unit=I-4): [A][configuration] The prose mixes best-of-N with K candidates and supplies no fixed N/K product value. Anchors: I-4_evidence_ranked_search_ladder.md:12-13,16,42.
- **I-003/DI003-I04-060** [AC] (DEEP-I-003.discovery.md:L255| unit=I-4 SURFACE): [A][UI] H-15 and SessionDag are plan-asserted seams requiring re-grep; their exact live APIs/surface cannot be treated as confirmed here. Anchors: I-4_evidence_ranked_search_ladder.md:38,60-62.
- **I-003/DI003-I04-061** [INF] (DEEP-I-003.discovery.md:L257| unit=I-4): [I][status] The three-rung descriptions are architectural capability, but only V1 is current ship scope; treating V2/V3 as current features would collapse explicit gates. Basis: I-4_evidence_ranked_search_ladder.md:40-52.

#### I-003 / unit `I-9` (32 records)

- **I-003/DI003-I09-001** [SRC] (DEEP-I-003.discovery.md:L266| unit=I-9): [E][status] Track 13; PROVABLE; STRONG; BUILD; effort M; Wave 4; binds ID-5. Anchor: I-9_error_amplification_telemetry.md:3.
- **I-003/DI003-I09-002** [SRC] (DEEP-I-003.discovery.md:L268| unit=I-9): [E][metric] alpha_t distinguishes amplification above 1 from contraction below 1 after a corrupt/low-quality step. Anchor: I-9_error_amplification_telemetry.md:5-10.
- **I-003/DI003-I09-003** [SRC] (DEEP-I-003.discovery.md:L270| unit=I-9): [E][workflow] ErrorPropagationHarness corrupts step t in a frozen fixture pipeline. Anchor: I-9_error_amplification_telemetry.md:12-15.
- **I-003/DI003-I09-004** [SRC] (DEEP-I-003.discovery.md:L272| unit=I-9): [E][dependency] C-28 MutationRunner supplies corruption operators; no new corruption primitives are built. Anchor: I-9_error_amplification_telemetry.md:13-14,28-31.
- **I-003/DI003-I09-005** [SRC] (DEEP-I-003.discovery.md:L274| unit=I-9): [E][measurement] The harness measures the fraction of downstream steps that recover versus propagate error. Anchor: I-9_error_amplification_telemetry.md:14-16.
- **I-003/DI003-I09-006** [SRC] (DEEP-I-003.discovery.md:L276| unit=I-9): [E][determinism] ErrorAmplificationMeasured carries alpha_t recomputable under ELAI_SEED. Anchor: I-9_error_amplification_telemetry.md:15-16.
- **I-003/DI003-I09-007** [SRC] (DEEP-I-003.discovery.md:L278| unit=I-9): [E][control] A mid-run alpha above 1 tightens C-26 temperature toward its lower bound. Anchor: I-9_error_amplification_telemetry.md:16-18.
- **I-003/DI003-I09-008** [SRC] (DEEP-I-003.discovery.md:L280| unit=I-9): [E][control/config] Temperature tightening is bounded; runaway cooling is forbidden. Anchor: I-9_error_amplification_telemetry.md:48-49.
- **I-003/DI003-I09-009** [SRC] (DEEP-I-003.discovery.md:L282| unit=I-9): [E][control/dependency] Grammar narrowing is optional through the I-34 seam if/when I-34 is live. Anchor: I-9_error_amplification_telemetry.md:17-18.
- **I-003/DI003-I09-010** [SRC] (DEEP-I-003.discovery.md:L284| unit=I-9): [E][evidence policy] ELAI's own baseline must be recorded before any contraction claim; literature alpha values cannot become thresholds. Anchor: I-9_error_amplification_telemetry.md:18-20,25-26.
- **I-003/DI003-I09-011** [SRC] (DEEP-I-003.discovery.md:L286| unit=I-9): [E][determinism] The benchmark mirrors the seccodeplt runner and uses embedded include_str fixtures. Anchor: I-9_error_amplification_telemetry.md:20.
- **I-003/DI003-I09-012** [SRC] (DEEP-I-003.discovery.md:L288| unit=I-9 SURFACE): [E][CLI] The command is elai bench error-amplification. Anchor: I-9_error_amplification_telemetry.md:21,47.
- **I-003/DI003-I09-013** [SRC] (DEEP-I-003.discovery.md:L290| unit=I-9): [E][non-goal] Always-on per-production-trajectory monitoring is excluded due to latency and no current consumer. Anchor: I-9_error_amplification_telemetry.md:21-24.
- **I-003/DI003-I09-014** [SRC] (DEEP-I-003.discovery.md:L292| unit=I-9): [E][non-goal] Replacing C-28 operators and publishing a pre-baseline target are declined. Anchor: I-9_error_amplification_telemetry.md:23-26.
- **I-003/DI003-I09-015** [SRC] (DEEP-I-003.discovery.md:L294| unit=I-9): [E][status] No alpha harness exists; this is a genuine BUILD. Anchor: I-9_error_amplification_telemetry.md:32.
- **I-003/DI003-I09-016** [SRC] (DEEP-I-003.discovery.md:L296| unit=I-9): [E][dependency] DriftComparator supplies the first-divergence measurement shape and must be re-grepped. Anchor: I-9_error_amplification_telemetry.md:35-36.
- **I-003/DI003-I09-017** [SRC] (DEEP-I-003.discovery.md:L298| unit=I-9): [E][dependency] I-4 branches are the Wave-4 trajectories/fixture pipeline measured by I-9. Anchor: I-9_error_amplification_telemetry.md:37-38,133-134.
- **I-003/DI003-I09-018** [SRC] (DEEP-I-003.discovery.md:L300| unit=I-9 SURFACE): [E][tier-1/surface] Ship error_amplification_runner.rs, CLI wiring, the bounded temperature hook, action registration, and I-9-decisions.md. Anchor: I-9_error_amplification_telemetry.md:42-53.
- **I-003/DI003-I09-019** [SRC] (DEEP-I-003.discovery.md:L303| unit=I-9): [E][lifecycle] The first artifact in I-9-decisions.md is the local baseline, preceding every contraction claim. Anchor: I-9_error_amplification_telemetry.md:51-53.
- **I-003/DI003-I09-020** [SRC] (DEEP-I-003.discovery.md:L305| unit=I-9): [E][deferred] Per-task production alpha estimation is Tier-2 until the baseline shows the metric is stable enough for in-loop use. Anchor: I-9_error_amplification_telemetry.md:54-56.
- **I-003/DI003-I09-021** [SRC] (DEEP-I-003.discovery.md:L307| unit=I-9 SURFACE): [E][deferred/UI] The cockpit alpha-trend tile waits for at least three consistent baseline runs. Anchor: I-9_error_amplification_telemetry.md:57-58.
- **I-003/DI003-I09-022** [SRC] (DEEP-I-003.discovery.md:L309| unit=I-9): [E][deferred] V1 annealing is temperature-only; the grammar-tighten arm waits for I-34. Anchor: I-9_error_amplification_telemetry.md:59-60,95.
- **I-003/DI003-I09-023** [SRC] (DEEP-I-003.discovery.md:L311| unit=I-9 SURFACE): [E][identifiers] Produced identifiers are ErrorPropagationHarness::run, ErrorAmplificationMeasured, and the benchmark CLI. Anchor: I-9_error_amplification_telemetry.md:71-75.
- **I-003/DI003-I09-024** [SRC] (DEEP-I-003.discovery.md:L314| unit=I-9 SURFACE): [E][reachability] CLI dispatch reaches the harness; a task_runner mid-run alpha event reaches temperature tightening; tests alone do not satisfy R7. Anchor: I-9_error_amplification_telemetry.md:81-86.
- **I-003/DI003-I09-025** [SRC] (DEEP-I-003.discovery.md:L317| unit=I-9): [E][constraint] Production paths ban premature literature thresholds, custom mutation operators, premature production estimation, unwrap/expect, non-blake3 hashes, and fabricated measurements. Anchor: I-9_error_amplification_telemetry.md:87-96.
- **I-003/DI003-I09-026** [SRC] (DEEP-I-003.discovery.md:L320| unit=I-9): [E][acceptance] The fixture row and any annealing temperature event must be seed-recomputable. Anchor: I-9_error_amplification_telemetry.md:100-105.
- **I-003/DI003-I09-027** [SRC] (DEEP-I-003.discovery.md:L322| unit=I-9): [E][broken-arm] A no-repair arm must produce alpha above 1 and a repaired arm below 1. Anchor: I-9_error_amplification_telemetry.md:106-109.
- **I-003/DI003-I09-028** [SRC] (DEEP-I-003.discovery.md:L324| unit=I-9): [E][status] The contraction proposition remains DESIGN until the local baseline runs. Anchor: I-9_error_amplification_telemetry.md:118-120.
- **I-003/DI003-I09-029** [SRC] (DEEP-I-003.discovery.md:L326| unit=I-9): [E][status] ErrorAmplificationMeasured is baseline+1 from LIVE 221/532 on 2026-06-10 and must be re-read at edit. Anchor: I-9_error_amplification_telemetry.md:123-124.
- **I-003/DI003-I09-030** [CON] (DEEP-I-003.discovery.md:L328| unit=I-9): [C][lifecycle] Revised Scope simultaneously ships a production task_runner mid-run-alpha hook and defers per-task live alpha estimation. The source does not explain what supplies the Tier-1 live measurement, so implementation status is internally unresolved. Anchors: I-9_error_amplification_telemetry.md:48-49,54-56,81-85.
- **I-003/DI003-I09-031** [CON] (DEEP-I-003.discovery.md:L332| unit=I-9 SURFACE): [C][summary status] Grammar tightening and the alpha tile are architectural features but explicitly Tier-2, not Tier-1 deliverables. Anchors: I-9_error_amplification_telemetry.md:54-60.
- **I-003/DI003-I09-032** [AC] (DEEP-I-003.discovery.md:L334| unit=I-9): [A][configuration] The temperature lower bound, tightening step, and baseline stability criterion are not numerically specified. Anchors: I-9_error_amplification_telemetry.md:17-18,48-49,55-58.

#### I-003 / unit `I-20` (43 records)

- **I-003/DI003-I20-001** [SRC] (DEEP-I-003.discovery.md:L342| unit=I-20): [E][status] Track 15 keystone; VERIFIED; STRONG; EXTENSION; effort M-L; Wave 3; ID-5 and IC-14. Anchor: I-20_budgeted_mutation_scheduler.md:3.
- **I-003/DI003-I20-002** [SD] (DEEP-I-003.discovery.md:L344| unit=I-20): [E][correction/gap] MutationRunner and EquivalentMutantCache already ship; the real gaps are diff-cone selection, sampled CI scoring, and survivor-to-Directive feedback. Anchor: I-20_budgeted_mutation_scheduler.md:4-10,43.
- **I-003/DI003-I20-003** [SRC] (DEEP-I-003.discovery.md:L347| unit=I-20): [E][vertical slice] Selector, sampler, and Directive emitter ship as one unit so mutation can run inside the search loop. Anchor: I-20_budgeted_mutation_scheduler.md:7-10.
- **I-003/DI003-I20-004** [SRC] (DEEP-I-003.discovery.md:L349| unit=I-20): [E][selection] Given a hunk, traverse elai_graph reachability or impact_radius to enumerate data-flow-reachable functions and mutate only them. Anchor: I-20_budgeted_mutation_scheduler.md:12-16.
- **I-003/DI003-I20-005** [SRC] (DEEP-I-003.discovery.md:L352| unit=I-20): [E][cache] Diff-cone reachability is cached in the SSB for the session by blake3(diff). Anchor: I-20_budgeted_mutation_scheduler.md:15-16,48.
- **I-003/DI003-I20-006** [SRC] (DEEP-I-003.discovery.md:L354| unit=I-20): [E][sampling] Sample without replacement from the cone-selected mutants using an adaptive sampler. Anchor: I-20_budgeted_mutation_scheduler.md:17-18.
- **I-003/DI003-I20-007** [SRC] (DEEP-I-003.discovery.md:L356| unit=I-20): [E][statistics] Compute a Wilson-score confidence interval after each batch and report the interval plus sample count, not a raw count. Anchor: I-20_budgeted_mutation_scheduler.md:18-20.
- **I-003/DI003-I20-008** [SRC] (DEEP-I-003.discovery.md:L358| unit=I-20): [E][budget] Stop when CI width clears its criterion or the time budget expires. Anchor: I-20_budgeted_mutation_scheduler.md:18-20.
- **I-003/DI003-I20-009** [SRC] (DEEP-I-003.discovery.md:L360| unit=I-20): [E][control] The design calls for both fail-early and pass-early exits based on CI bounds. Anchor: I-20_budgeted_mutation_scheduler.md:21-22,49.
- **I-003/DI003-I20-010** [SRC] (DEEP-I-003.discovery.md:L362| unit=I-20): [E][evidence policy] The gate threshold comes from ELAI's own fixed-fixture baseline, never a paper kill-rate constant. Anchor: I-20_budgeted_mutation_scheduler.md:19-20,32-34.
- **I-003/DI003-I20-011** [SRC] (DEEP-I-003.discovery.md:L364| unit=I-20): [E][cache schema] Mutation results reuse EquivalentMutantCache with key blake3(function-content-hash, bound-test-set-hash). Anchor: I-20_budgeted_mutation_scheduler.md:23-25.
- **I-003/DI003-I20-012** [SRC] (DEEP-I-003.discovery.md:L367| unit=I-20): [E][cache migration] Adding bound-test-set hash is backwards-compatible: old entries miss and rerun once. Anchor: I-20_budgeted_mutation_scheduler.md:50.
- **I-003/DI003-I20-013** [SRC] (DEEP-I-003.discovery.md:L369| unit=I-20): [E][performance] Unchanged code returns CacheHit and runs zero mutants. Anchor: I-20_budgeted_mutation_scheduler.md:24-25.
- **I-003/DI003-I20-014** [SRC] (DEEP-I-003.discovery.md:L371| unit=I-20): [E][feedback/schema] A surviving mutant emits SurvivorDirective naming the behavioral distinction, operator, source line, and surviving replacement. Anchor: I-20_budgeted_mutation_scheduler.md:26-28,51.
- **I-003/DI003-I20-015** [SRC] (DEEP-I-003.discovery.md:L374| unit=I-20): [E][ownership] I-20 emits the Directive but does not own the generator retry loop or retry count. Anchor: I-20_budgeted_mutation_scheduler.md:27-28.
- **I-003/DI003-I20-016** [SRC] (DEEP-I-003.discovery.md:L376| unit=I-20): [E][integration] Proptest and Hypothesis relation adapters register as a parallel RelationLibrary lane behind D-3; no new dispatcher is needed. Anchor: I-20_budgeted_mutation_scheduler.md:29-31.
- **I-003/DI003-I20-017** [SRC] (DEEP-I-003.discovery.md:L379| unit=I-20): [E][lifecycle/artifact] First deploy measures the local baseline and writes metrics/mutation_baseline.toml before enabling a threshold. Anchor: I-20_budgeted_mutation_scheduler.md:32-34,52.
- **I-003/DI003-I20-018** [SRC] (DEEP-I-003.discovery.md:L382| unit=I-20): [E][non-goal] Exhaustive hot-path mutation, paper-derived thresholds, and a second mutant cache are excluded. Anchor: I-20_budgeted_mutation_scheduler.md:35-36.
- **I-003/DI003-I20-019** [SRC] (DEEP-I-003.discovery.md:L384| unit=I-20): [E][dependency/status] MutationRunner, EquivalentMutantCache, and RelationLibrary are shipped substrates to extend, not rebuild. Anchor: I-20_budgeted_mutation_scheduler.md:38-41.
- **I-003/DI003-I20-020** [AC] (DEEP-I-003.discovery.md:L386| unit=I-20): [A][integration] Execution must choose between an in-process graph API and an impact_radius subprocess after re-grep, preferring in-process if it exists. Anchor: I-20_budgeted_mutation_scheduler.md:42.
- **I-003/DI003-I20-021** [SRC] (DEEP-I-003.discovery.md:L389| unit=I-20): [E][tier-1] Ship mutation_budget.rs selector/sampler, cache-key extension, SurvivorDirective, baseline fixture, MutationBudgetedRun, and PBT adapters. Anchor: I-20_budgeted_mutation_scheduler.md:46-54.
- **I-003/DI003-I20-022** [SRC] (DEEP-I-003.discovery.md:L392| unit=I-20 SURFACE): [E][deferred] Exhaustive mutation is an offline opt-in CLI flag only. Anchor: I-20_budgeted_mutation_scheduler.md:55-57.
- **I-003/DI003-I20-023** [SRC] (DEEP-I-003.discovery.md:L394| unit=I-20): [E][deferred/dependency] PITest and cosmic-ray adapters require separate per-language approval. Anchor: I-20_budgeted_mutation_scheduler.md:57.
- **I-003/DI003-I20-024** [SRC] (DEEP-I-003.discovery.md:L396| unit=I-20 SURFACE): [E][surface] MutationBudget wraps MutationRunner without changing its internals and is called by I-17 and the search ladder. Anchor: I-20_budgeted_mutation_scheduler.md:59-64.
- **I-003/DI003-I20-025** [SRC] (DEEP-I-003.discovery.md:L398| unit=I-20): [E][schema/observability] MutationBudgetedRun kinds are CiCleared, CiFailed, SurvivorDirective, and CacheHit. Anchor: I-20_budgeted_mutation_scheduler.md:65,69-74.
- **I-003/DI003-I20-026** [SRC] (DEEP-I-003.discovery.md:L400| unit=I-20 SURFACE): [E][UI] An existing branch-board/audit pane gets a per-hunk chip showing CI, n, verdict, or survivor path; no new drawer is added. Anchor: I-20_budgeted_mutation_scheduler.md:66-67,92-95.
- **I-003/DI003-I20-027** [SRC] (DEEP-I-003.discovery.md:L402| unit=I-20): [E][identifiers] Produced symbols are MutationBudget, SurvivorDirective, MutationBudgetedRun, and mutation_baseline.toml. Anchor: I-20_budgeted_mutation_scheduler.md:69-74.
- **I-003/DI003-I20-028** [SRC] (DEEP-I-003.discovery.md:L404| unit=I-20): [E][dependency] Consumers include MutationRunner, EquivalentMutantCache, RelationLibrary, C-36 MUCOCO operators, graph reachability, and I-17. Anchor: I-20_budgeted_mutation_scheduler.md:75-81.
- **I-003/DI003-I20-029** [SRC] (DEEP-I-003.discovery.md:L407| unit=I-20): [E][reachability] Production paths are I-17 run_verdict and I-4 audit expansion; contracts-only callers do not satisfy R7. Anchor: I-20_budgeted_mutation_scheduler.md:82-85.
- **I-003/DI003-I20-030** [SRC] (DEEP-I-003.discovery.md:L409| unit=I-20): [E][constraint] Hot paths require cone selection and a time box; exhaustive mode is offline-only. Anchor: I-20_budgeted_mutation_scheduler.md:86-87.
- **I-003/DI003-I20-031** [SRC] (DEEP-I-003.discovery.md:L411| unit=I-20): [E][constraint] A numeric hard-coded threshold is a BLOCK; threshold must read the baseline TOML. Anchor: I-20_budgeted_mutation_scheduler.md:88.
- **I-003/DI003-I20-032** [SRC] (DEEP-I-003.discovery.md:L413| unit=I-20): [E][constraint] A second cache or MutationRunner-internal modification is forbidden. Anchor: I-20_budgeted_mutation_scheduler.md:89-90.
- **I-003/DI003-I20-033** [SRC] (DEEP-I-003.discovery.md:L415| unit=I-20): [E][constraint] Production paths ban unwrap/expect, non-blake3 hashes, and fabricated CI values. Anchor: I-20_budgeted_mutation_scheduler.md:91.
- **I-003/DI003-I20-034** [SRC] (DEEP-I-003.discovery.md:L417| unit=I-20): [E][observability] CiCleared records n_sampled, n_total_cone, ci_lower, ci_upper, and time_ms under ELAI_SEED. Anchor: I-20_budgeted_mutation_scheduler.md:96-98.
- **I-003/DI003-I20-035** [SRC] (DEEP-I-003.discovery.md:L419| unit=I-20): [E][acceptance] An unchanged rerun must produce CacheHit with zero mutation runs; survivors must identify operator and line. Anchor: I-20_budgeted_mutation_scheduler.md:97-98.
- **I-003/DI003-I20-036** [SRC] (DEEP-I-003.discovery.md:L421| unit=I-20): [E][broken-arm] A contract must reject a literal paper threshold and prove the value comes from mutation_baseline.toml. Anchor: I-20_budgeted_mutation_scheduler.md:99.
- **I-003/DI003-I20-037** [SRC] (DEEP-I-003.discovery.md:L423| unit=I-20): [E][status] MutationBudgetedRun is baseline+1 from LIVE 221/532 on 2026-06-10 and must be re-read at edit. Anchor: I-20_budgeted_mutation_scheduler.md:109.
- **I-003/DI003-I20-038** [SRC] (DEEP-I-003.discovery.md:L425| unit=I-20): [E][done-gate] Full-form expansion is mandatory before EXECUTING. Anchor: I-20_budgeted_mutation_scheduler.md:118.
- **I-003/DI003-I20-039** [CON] (DEEP-I-003.discovery.md:L427| unit=I-20): [C][statistics] The prose says fail early when the CI lower bound already fails the threshold; conventional decisive failure normally depends on the upper bound being below threshold. The intended formula is ambiguous and must be resolved, not silently corrected. Anchor: I-20_budgeted_mutation_scheduler.md:21-22.
- **I-003/DI003-I20-040** [CON] (DEEP-I-003.discovery.md:L431| unit=I-20): [C][summary ownership] Bounded retries are a surrounding-generator property; I-20 explicitly does not own the retry loop. Anchor: I-20_budgeted_mutation_scheduler.md:27-28.
- **I-003/DI003-I20-041** [CON] (DEEP-I-003.discovery.md:L433| unit=I-20): [C][summary cache] Two caches are distinct: SSB session cache for diff-cone reachability and EquivalentMutantCache for execution results. Anchors: I-20_budgeted_mutation_scheduler.md:15-16,23-25.
- **I-003/DI003-I20-042** [AC] (DEEP-I-003.discovery.md:L436| unit=I-20): [A][configuration] Time-box duration, batch/sample policy, retry bound, CI-width criterion, and gate threshold are not numerically fixed. Anchors: I-20_budgeted_mutation_scheduler.md:17-22,27-28,32-34.
- **I-003/DI003-I20-043** [INF] (DEEP-I-003.discovery.md:L439| unit=I-20): [I][lifecycle] Because cache-key migration intentionally makes old entries miss, the first post-migration audit repays mutation cost once; this is implied by the explicit compatibility rule. Basis: I-20_budgeted_mutation_scheduler.md:50.

#### I-003 / unit `I-32` (29 records)

- **I-003/DI003-I32-001** [SRC] (DEEP-I-003.discovery.md:L448| unit=I-32): [E][status] Track 18; STEERING-TRUST; STRONG; REFACTOR; effort M; Wave 2; binds IC-17. Anchor: I-32_structural_role_scoping.md:3.
- **I-003/DI003-I32-002** [SRC] (DEEP-I-003.discovery.md:L450| unit=I-32): [E][policy] Persona removal is evidence-gated with a per-role fixture A/B before any production change. Anchor: I-32_structural_role_scoping.md:5-9.
- **I-003/DI003-I32-003** [SRC] (DEEP-I-003.discovery.md:L452| unit=I-32): [E][scope] Executor, context-compiler, researcher, and factual/code-generation agents lose narrative persona blocks. Anchor: I-32_structural_role_scoping.md:11-16.
- **I-003/DI003-I32-004** [SRC] (DEEP-I-003.discovery.md:L454| unit=I-32): [E][schema] Replacement role contracts are schema-first with typed input/output and explicit output constraints. Anchor: I-32_structural_role_scoping.md:14-15.
- **I-003/DI003-I32-005** [SRC] (DEEP-I-003.discovery.md:L456| unit=I-32): [E][prompt constraint] Affected roles have no “you are X who Y” opener. Anchor: I-32_structural_role_scoping.md:14-15.
- **I-003/DI003-I32-006** [SRC] (DEEP-I-003.discovery.md:L458| unit=I-32): [E][retention] Auditor, critic, adversary, and anti-spiral's five lateral personas retain alignment-dependent framing. Anchor: I-32_structural_role_scoping.md:15-17.
- **I-003/DI003-I32-007** [SRC] (DEEP-I-003.discovery.md:L460| unit=I-32): [E][config] Persona blocks are set max-tokens=0 and replaced by typed constraint and I/O schema. Anchor: I-32_structural_role_scoping.md:19-21.
- **I-003/DI003-I32-008** [SRC] (DEEP-I-003.discovery.md:L462| unit=I-32): [E][testing] The old and new prompt templates run against E-19 fixtures per role. Anchor: I-32_structural_role_scoping.md:20-22.
- **I-003/DI003-I32-009** [SRC] (DEEP-I-003.discovery.md:L464| unit=I-32): [E][promotion] Each affected role flips only if new pass rate meets or exceeds its old baseline. Anchor: I-32_structural_role_scoping.md:21-22.
- **I-003/DI003-I32-010** [SRC] (DEEP-I-003.discovery.md:L466| unit=I-32): [E][non-goal] Wholesale persona removal is excluded because alignment-dependent roles are the carve-out. Anchor: I-32_structural_role_scoping.md:23-28.
- **I-003/DI003-I32-011** [SD] (DEEP-I-003.discovery.md:L468| unit=I-32): [E][status/correction] IC-17 is binding and corrects the overbroad source-dossier reading. Anchor: I-32_structural_role_scoping.md:26-28.
- **I-003/DI003-I32-012** [SRC] (DEEP-I-003.discovery.md:L470| unit=I-32): [E][dependency/status] Phase 2C role configs, agent templates, and C-41 routing are plan-asserted and require re-grep; streaming role plumbing is live. Anchor: I-32_structural_role_scoping.md:30-35.
- **I-003/DI003-I32-013** [SRC] (DEEP-I-003.discovery.md:L473| unit=I-32): [E][tier-1] Tier-1 names executor, context-compiler, and researcher templates plus the A/B harness and retained-role proof rows. Anchor: I-32_structural_role_scoping.md:37-43.
- **I-003/DI003-I32-014** [SD] (DEEP-I-003.discovery.md:L475| unit=I-32): [E][observability] RoleContractApplied emits seeded AbDelta per role and PersonaRetained proof for unaffected roles. Anchor: I-32_structural_role_scoping.md:40-41.
- **I-003/DI003-I32-015** [SRC] (DEEP-I-003.discovery.md:L477| unit=I-32): [E][deferred] Cross-role executor→auditor chain effects wait for the I-21 sprint context. Anchor: I-32_structural_role_scoping.md:45-47.
- **I-003/DI003-I32-016** [SRC] (DEEP-I-003.discovery.md:L479| unit=I-32): [E][deferred] Per-commit regression A/B for new persona blocks waits for a post-Wave-3 CI harness. Anchor: I-32_structural_role_scoping.md:47.
- **I-003/DI003-I32-017** [SRC] (DEEP-I-003.discovery.md:L481| unit=I-32): [E][identifiers] Produce RoleContract, RoleContractStore, and RoleContractApplied with PersonaStripped/PersonaRetained/AbDelta kinds. Anchor: I-32_structural_role_scoping.md:49-62.
- **I-003/DI003-I32-018** [SRC] (DEEP-I-003.discovery.md:L483| unit=I-32): [E][integration] agent_dispatch prompt building switches affected roles to apply_to_system_prompt and reuses E-19 fixtures. Anchor: I-32_structural_role_scoping.md:52-54,63-66.
- **I-003/DI003-I32-019** [SD] (DEEP-I-003.discovery.md:L485| unit=I-32 SURFACE): [E][UI] This is prompt-internal; AbDelta uses the existing governance metrics stream and no new Svelte component. Anchor: I-32_structural_role_scoping.md:55-56,75-76.
- **I-003/DI003-I32-020** [SRC] (DEEP-I-003.discovery.md:L487| unit=I-32 SURFACE): [E][reachability] dispatch_agent → apply_to_system_prompt → SSB row and the role-ab-probe CLI are production paths; tests alone do not satisfy R7. Anchor: I-32_structural_role_scoping.md:67-70.
- **I-003/DI003-I32-021** [SRC] (DEEP-I-003.discovery.md:L490| unit=I-32): [E][constraint] After flip, affected prompts contain zero narrative, and no flip occurs without a non-regressing AbDelta row. Anchor: I-32_structural_role_scoping.md:71-74.
- **I-003/DI003-I32-022** [SRC] (DEEP-I-003.discovery.md:L492| unit=I-32): [E][constraint] Production paths ban unwrap/expect, non-blake3 hashes, and fabricated measurements. Anchor: I-32_structural_role_scoping.md:74.
- **I-003/DI003-I32-023** [SRC] (DEEP-I-003.discovery.md:L494| unit=I-32): [E][acceptance] Seeded probe rows must show stripped affected roles and retained auditor/critic/adversary/lateral roles. Anchor: I-32_structural_role_scoping.md:77-80.
- **I-003/DI003-I32-024** [SRC] (DEEP-I-003.discovery.md:L496| unit=I-32): [E][failure] Any per-role pass-rate regression blocks production flip; journal evidence proves the executor prompt has a schema and no narrative. Anchor: I-32_structural_role_scoping.md:78-80.
- **I-003/DI003-I32-025** [SRC] (DEEP-I-003.discovery.md:L498| unit=I-32): [E][status] RoleContractApplied is baseline+1 from LIVE 221/532 and must be re-read at edit. Anchor: I-32_structural_role_scoping.md:62,88.
- **I-003/DI003-I32-026** [SRC] (DEEP-I-003.discovery.md:L500| unit=I-32): [E][dependency] Compose Phase 2C dispatch, in-place agent templates, E-19 loading, C-41 routing, and I-21's retained evaluator persona. Anchor: I-32_structural_role_scoping.md:90-95.
- **I-003/DI003-I32-027** [AC] (DEEP-I-003.discovery.md:L502| unit=I-32): [A][scope] The general rule covers all factual/code roles, while the concrete Tier-1 list names only three; treatment of any additional factual/code roles is not enumerated. Anchors: I-32_structural_role_scoping.md:13-16,38-41.
- **I-003/DI003-I32-028** [AC] (DEEP-I-003.discovery.md:L505| unit=I-32): [A][measurement] The local fixture gate is pass-rate ≥ baseline, but fixture composition, sample size, and statistical tolerance are unspecified. Anchor: I-32_structural_role_scoping.md:20-22,42.
- **I-003/DI003-I32-029** [INF] (DEEP-I-003.discovery.md:L507| unit=I-32): [I][status] Persona-performance numbers motivate the refactor but are grounding claims, not acceptance thresholds; local E-19 results own the flip. Basis: I-32_structural_role_scoping.md:5-9,82-87.

#### I-003 / unit `I-33` (31 records)

- **I-003/DI003-I33-001** [SRC] (DEEP-I-003.discovery.md:L516| unit=I-33): [E][status] Track 18; TRUST; STRONG; BUILD; effort M; Wave 2. Anchor: I-33_mechanical_decision_governance.md:3.
- **I-003/DI003-I33-002** [SRC] (DEEP-I-003.discovery.md:L518| unit=I-33): [E][scope] CEFL, I6Q, and commit-reveal apply to all orchestrator decision-type LLM calls such as mode picks, gate advisories, and classifications. Anchor: I-33_mechanical_decision_governance.md:5-11.
- **I-003/DI003-I33-003** [SRC] (DEEP-I-003.discovery.md:L521| unit=I-33): [E][CEFL] The orchestrator authors the complete candidate set and the model only scores or ranks it. Anchor: I-33_mechanical_decision_governance.md:13-16.
- **I-003/DI003-I33-004** [SRC] (DEEP-I-003.discovery.md:L523| unit=I-33): [E][schema/security] The response schema makes model-invented options structurally unparseable, eliminating selection-by-negation at the type level. Anchor: I-33_mechanical_decision_governance.md:13-16.
- **I-003/DI003-I33-005** [SRC] (DEEP-I-003.discovery.md:L526| unit=I-33): [E][I6Q] A deterministic validator enforces minimum rationale length and lexical diversity before acceptance. Anchor: I-33_mechanical_decision_governance.md:18-20.
- **I-003/DI003-I33-006** [SRC] (DEEP-I-003.discovery.md:L528| unit=I-33): [E][failure] Vacuous rationales are rejected with GovernancePrimitiveFired(I6qRejected). Anchor: I-33_mechanical_decision_governance.md:19-21.
- **I-003/DI003-I33-007** [SRC] (DEEP-I-003.discovery.md:L530| unit=I-33): [E][config/security] I6Q thresholds are orchestrator-owned configuration and cannot be set by the model. Anchor: I-33_mechanical_decision_governance.md:21.
- **I-003/DI003-I33-008** [SRC] (DEEP-I-003.discovery.md:L532| unit=I-33): [E][integrity] Every stochastic gate choice commits blake3(seed || choice_index) before revealing the choice. Anchor: I-33_mechanical_decision_governance.md:23-25.
- **I-003/DI003-I33-009** [SRC] (DEEP-I-003.discovery.md:L535| unit=I-33): [E][evidence] SSB ordering proves commit-before-reveal; reveal-before-commit is an R2 failure. Anchor: I-33_mechanical_decision_governance.md:24-25.
- **I-003/DI003-I33-010** [SRC] (DEEP-I-003.discovery.md:L537| unit=I-33): [E][non-goal] Model-authored candidates and reveal-before-commit are explicitly excluded. Anchor: I-33_mechanical_decision_governance.md:27-29.
- **I-003/DI003-I33-011** [SRC] (DEEP-I-003.discovery.md:L539| unit=I-33): [E][metrics] Each seeded run ledgers CDL cosmetic-deadlock rate, DIU deferral-information utilization, and FSR framing-success rate. Anchor: I-33_mechanical_decision_governance.md:31-33.
- **I-003/DI003-I33-012** [SRC] (DEEP-I-003.discovery.md:L541| unit=I-33): [E][evidence policy] Governance decoupling under task stress is measured locally before it is claimed. Anchor: I-33_mechanical_decision_governance.md:31-33.
- **I-003/DI003-I33-013** [SRC] (DEEP-I-003.discovery.md:L543| unit=I-33): [E][status] CEFL/I6Q/commit-reveal are grep-confirmed absent, so I-33 is a BUILD. Anchor: I-33_mechanical_decision_governance.md:35-36.
- **I-003/DI003-I33-014** [SRC] (DEEP-I-003.discovery.md:L545| unit=I-33): [E][dependency/status] Phase 3F, D-16, B-22/H-37, and C-26 are plan-asserted; E-25 prompt_decision is present at file level. Anchor: I-33_mechanical_decision_governance.md:37-41.
- **I-003/DI003-I33-015** [SRC] (DEEP-I-003.discovery.md:L547| unit=I-33 SURFACE): [E][tier-1/identifiers] Ship CeflRequest, I6qValidator, CommitRevealRecord, wrapped call sites, action rows, fixtures, UI tile, and decision doc. Anchor: I-33_mechanical_decision_governance.md:43-51.
- **I-003/DI003-I33-016** [SRC] (DEEP-I-003.discovery.md:L550| unit=I-33): [E][schema/observability] GovernancePrimitiveFired kinds are CeflExternalized, I6qRejected, and CommitRevealSealed. Anchor: I-33_mechanical_decision_governance.md:45-48,65-70.
- **I-003/DI003-I33-017** [SRC] (DEEP-I-003.discovery.md:L552| unit=I-33): [E][testing] Fixtures cover framing inversion, vacuous rationale, and out-of-order commit. Anchor: I-33_mechanical_decision_governance.md:50.
- **I-003/DI003-I33-018** [SRC] (DEEP-I-003.discovery.md:L554| unit=I-33): [E][deferred] Cross-gate DIU correlation requires at least three real runs. Anchor: I-33_mechanical_decision_governance.md:53-55.
- **I-003/DI003-I33-019** [SRC] (DEEP-I-003.discovery.md:L556| unit=I-33): [E][deferred] Adaptive I6Q calibration waits for local ID-5 measurement. Anchor: I-33_mechanical_decision_governance.md:55.
- **I-003/DI003-I33-020** [SRC] (DEEP-I-003.discovery.md:L558| unit=I-33): [E][integration] Every gate-advisory/mode/classification prompt_decision call is wrapped by CEFL and I6Q; Phase 3F routes rows and D-16 handles refusal. Anchor: I-33_mechanical_decision_governance.md:57-63,71-75.
- **I-003/DI003-I33-021** [SRC] (DEEP-I-003.discovery.md:L561| unit=I-33 SURFACE): [E][UI] A mandatory new governance metrics chip in MonitorWindow/SettingsWindow renders CDL/DIU/FSR through Tauri. Anchor: I-33_mechanical_decision_governance.md:49,62-63,85-86.
- **I-003/DI003-I33-022** [SRC] (DEEP-I-003.discovery.md:L563| unit=I-33): [E][reachability] Production prompt_decision and stochastic-gate paths must emit rows; tests alone do not satisfy R7. Anchor: I-33_mechanical_decision_governance.md:76-79.
- **I-003/DI003-I33-023** [SRC] (DEEP-I-003.discovery.md:L565| unit=I-33): [E][constraint] Parsing a model-invented option or revealing before commit proves the wrapper broken. Anchor: I-33_mechanical_decision_governance.md:80-82.
- **I-003/DI003-I33-024** [SRC] (DEEP-I-003.discovery.md:L567| unit=I-33): [E][constraint] Production bans unwrap/expect, non-blake3 hashes, and fabricated CDL/DIU/FSR. Anchor: I-33_mechanical_decision_governance.md:83-84.
- **I-003/DI003-I33-025** [SRC] (DEEP-I-003.discovery.md:L569| unit=I-33): [E][acceptance] Every decision row carries the candidate list; vacuous input produces I6qRejected; stochastic choices have commit_row_id < reveal_row_id under the same seed. Anchor: I-33_mechanical_decision_governance.md:87-90.
- **I-003/DI003-I33-026** [SRC] (DEEP-I-003.discovery.md:L572| unit=I-33 SURFACE): [E][acceptance] Reversing field order must yield the same decision, out-of-order commit exits nonzero, and the UI passes cross_lang_diff. Anchor: I-33_mechanical_decision_governance.md:88-90.
- **I-003/DI003-I33-027** [SRC] (DEEP-I-003.discovery.md:L574| unit=I-33): [E][status] GovernancePrimitiveFired is baseline+1 from LIVE 221/532 and must be re-read at edit. Anchor: I-33_mechanical_decision_governance.md:70,97.
- **I-003/DI003-I33-028** [SRC] (DEEP-I-003.discovery.md:L576| unit=I-33): [E][dependency] Compose prompt_decision, Phase 3F routing, D-16 refusal, and C-26 schema; do not fork any. Anchor: I-33_mechanical_decision_governance.md:100-104.
- **I-003/DI003-I33-029** [AC] (DEEP-I-003.discovery.md:L578| unit=I-33): [A][configuration] Minimum rationale length, lexical-diversity threshold, and later adaptive calibration rule are unspecified. Anchors: I-33_mechanical_decision_governance.md:18-21,53-55.
- **I-003/DI003-I33-030** [AC] (DEEP-I-003.discovery.md:L580| unit=I-33): [A][scope] “All decision-type calls” is categorical, but only three call families and plan-asserted sites are enumerated; a complete call-site census is deferred to execution. Anchors: I-33_mechanical_decision_governance.md:11,37-41,47.
- **I-003/DI003-I33-031** [INF] (DEEP-I-003.discovery.md:L583| unit=I-33): [I][security] Commit-reveal proves ordering/integrity, not unbiased randomness; no entropy source beyond seed is specified. Basis: I-33_mechanical_decision_governance.md:23-25.

#### I-003 / unit `I-38` (33 records)

- **I-003/DI003-I38-001** [SRC] (DEEP-I-003.discovery.md:L591| unit=I-38): [E][status] Track 18; PROVABLE; STRONG; BUILD; effort M; Wave 1; binds ID-5. Anchor: I-38_qdelta_statistical_floor.md:3.
- **I-003/DI003-I38-002** [SD] (DEEP-I-003.discovery.md:L593| unit=I-38): [E][gap] SAFEDELTA exists, but QDELTA and mechanical statistical-floor enforcement do not. Anchor: I-38_qdelta_statistical_floor.md:5-9.
- **I-003/DI003-I38-003** [SD] (DEEP-I-003.discovery.md:L595| unit=I-38): [E][metric] QDELTA is defective diffs reaching a reviewer per 100 tasks, compared with gates ON versus OFF. Anchor: I-38_qdelta_statistical_floor.md:11-14.
- **I-003/DI003-I38-004** [SRC] (DEEP-I-003.discovery.md:L597| unit=I-38): [E][schema] A defective diff includes held-out test failures, post-merge surviving I-20 mutants, and regressions. Anchor: I-38_qdelta_statistical_floor.md:12-14.
- **I-003/DI003-I38-005** [SRC] (DEEP-I-003.discovery.md:L599| unit=I-38): [E][statistics] Confidence intervals are bootstrapped and clustered by repository. Anchor: I-38_qdelta_statistical_floor.md:13-14.
- **I-003/DI003-I38-006** [SRC] (DEEP-I-003.discovery.md:L601| unit=I-38): [E][framing/non-goal] Reporting is gate ablation only; SWE-bench Verified/leaderboard rank is explicitly declined. Anchor: I-38_qdelta_statistical_floor.md:14-16.
- **I-003/DI003-I38-007** [SRC] (DEEP-I-003.discovery.md:L603| unit=I-38): [E][policy] Every gates-ON run requires a gates-OFF counterpart on the same tasks. Anchor: I-38_qdelta_statistical_floor.md:18-20.
- **I-003/DI003-I38-008** [SRC] (DEEP-I-003.discovery.md:L605| unit=I-38): [E][policy] Each condition requires at least five seeds and reports SEM. Anchor: I-38_qdelta_statistical_floor.md:20.
- **I-003/DI003-I38-009** [SRC] (DEEP-I-003.discovery.md:L607| unit=I-38): [E][policy] Multi-step reliability claims report pass^k and log k explicitly. Anchor: I-38_qdelta_statistical_floor.md:21.
- **I-003/DI003-I38-010** [SRC] (DEEP-I-003.discovery.md:L609| unit=I-38): [E][economics] Every headline reports dollars per task beside accuracy. Anchor: I-38_qdelta_statistical_floor.md:22.
- **I-003/DI003-I38-011** [SRC] (DEEP-I-003.discovery.md:L611| unit=I-38 SURFACE): [E][corpus/privacy] Runs use date-stamped task cuts and surface D-20 memorization probe results as contamination evidence. Anchor: I-38_qdelta_statistical_floor.md:23.
- **I-003/DI003-I38-012** [SRC] (DEEP-I-003.discovery.md:L613| unit=I-38): [E][fail-closed] floor_check validates config at startup and exits nonzero before compute on violations. Anchor: I-38_qdelta_statistical_floor.md:25-27.
- **I-003/DI003-I38-013** [SD] (DEEP-I-003.discovery.md:L615| unit=I-38): [E][contamination] QdeltaConfig has contamination_probe and executes D-20 before calling a run clean. Anchor: I-38_qdelta_statistical_floor.md:29-30.
- **I-003/DI003-I38-014** [SD] (DEEP-I-003.discovery.md:L617| unit=I-38): [E][status] IN-9 confirms QDELTA absent; SAFEDELTA is only a shape precedent. Anchor: I-38_qdelta_statistical_floor.md:32-34.
- **I-003/DI003-I38-015** [SD] (DEEP-I-003.discovery.md:L619| unit=I-38): [E][dependency/status] D-6/E-54 corpus, D-20, E-49, and SAFEDELTA are plan-asserted and re-grepped; I-20 supplies mutant data. Anchor: I-38_qdelta_statistical_floor.md:34-39.
- **I-003/DI003-I38-016** [SD] (DEEP-I-003.discovery.md:L621| unit=I-38 SURFACE): [E][tier-1/identifiers] Ship QdeltaConfig, run_qdelta, floor_check, bootstrap_ci, deterministic methodology generation, QdeltaMeasured, fixtures, probe field, UI tile, and decision doc. Anchor: I-38_qdelta_statistical_floor.md:41-51.
- **I-003/DI003-I38-017** [SD] (DEEP-I-003.discovery.md:L624| unit=I-38): [E][schema/observability] QdeltaMeasured includes ON/OFF delta, CI, seed_count, paired, dollars/task, and contamination-probe result. Anchor: I-38_qdelta_statistical_floor.md:45-47.
- **I-003/DI003-I38-018** [SRC] (DEEP-I-003.discovery.md:L626| unit=I-38): [E][testing] Ship a valid paired ≥5-seed fixture and a 3-seed unpaired fixture the harness refuses. Anchor: I-38_qdelta_statistical_floor.md:48.
- **I-003/DI003-I38-019** [SRC] (DEEP-I-003.discovery.md:L628| unit=I-38): [E][deferred] Full-corpus evaluation is RunPod-gated; Tier-1 is fixture-scale. Anchor: I-38_qdelta_statistical_floor.md:53-55.
- **I-003/DI003-I38-020** [SRC] (DEEP-I-003.discovery.md:L630| unit=I-38): [E][deferred] Cross-repository CI-width analysis waits for at least three repositories of real data. Anchor: I-38_qdelta_statistical_floor.md:55.
- **I-003/DI003-I38-021** [SD] (DEEP-I-003.discovery.md:L632| unit=I-38): [E][integration] QdeltaConfig mirrors SafedeltaConfig and shares D-6/E-54 corpus plus 13-gram filtering. Anchor: I-38_qdelta_statistical_floor.md:57-60.
- **I-003/DI003-I38-022** [SRC] (DEEP-I-003.discovery.md:L634| unit=I-38): [E][integration] D-20 supplies contamination, I-20 rows supply surviving mutants, and E-49 supplies metrics plumbing. Anchor: I-38_qdelta_statistical_floor.md:61-62,72-77.
- **I-003/DI003-I38-023** [SD] (DEEP-I-003.discovery.md:L636| unit=I-38 SURFACE): [E][UI] A new QDELTA Svelte/Tauri tile sits beside SAFEDELTA and must pass cross_lang_diff. Anchor: I-38_qdelta_statistical_floor.md:63,88-89.
- **I-003/DI003-I38-024** [SD] (DEEP-I-003.discovery.md:L638| unit=I-38 SURFACE): [E][reachability] elai bench qdelta reaches run_qdelta; floor_check runs at entry; tests alone do not satisfy R7. Anchor: I-38_qdelta_statistical_floor.md:78-81.
- **I-003/DI003-I38-025** [SRC] (DEEP-I-003.discovery.md:L640| unit=I-38): [E][constraint] Unpaired or fewer-than-five-seed runs may emit no headline/QdeltaMeasured row. Anchor: I-38_qdelta_statistical_floor.md:82-85.
- **I-003/DI003-I38-026** [SRC] (DEEP-I-003.discovery.md:L642| unit=I-38): [E][constraint] Methodology starts with ablation-only framing; CI is seed-recomputable; production bans unwrap/expect and non-blake3 hashes. Anchor: I-38_qdelta_statistical_floor.md:85-87.
- **I-003/DI003-I38-027** [SRC] (DEEP-I-003.discovery.md:L644| unit=I-38): [E][acceptance] A valid run emits all required payload fields and is byte-recomputable; the underpowered run emits no QdeltaMeasured. Anchor: I-38_qdelta_statistical_floor.md:90-92.
- **I-003/DI003-I38-028** [SRC] (DEEP-I-003.discovery.md:L646| unit=I-38 SURFACE): [E][acceptance] Refusal names the floor violation; methodology regenerates deterministically and names D-20; the UI passes cross-language checking. Anchor: I-38_qdelta_statistical_floor.md:93.
- **I-003/DI003-I38-029** [SD] (DEEP-I-003.discovery.md:L649| unit=I-38): [E][status] QdeltaMeasured is baseline+1 from LIVE 221/532 and must be re-read at edit. Anchor: I-38_qdelta_statistical_floor.md:70,103.
- **I-003/DI003-I38-030** [SD] (DEEP-I-003.discovery.md:L651| unit=I-38): [E][dependency] SAFEDELTA, corpus loading, memorization probe, mutation scoring, and metrics plumbing must be composed, never reimplemented. Anchor: I-38_qdelta_statistical_floor.md:105-110.
- **I-003/DI003-I38-031** [AC] (DEEP-I-003.discovery.md:L653| unit=I-38): [A][statistics] Bootstrap resample count, CI level, SEM presentation, defect weighting, and cost-accounting method are not specified. Anchors: I-38_qdelta_statistical_floor.md:11-23,43-47.
- **I-003/DI003-I38-032** [AC] (DEEP-I-003.discovery.md:L655| unit=I-38): [A][contamination] The source requires D-20 to run but does not define which probe result rejects a task cut. Anchor: I-38_qdelta_statistical_floor.md:29-30.
- **I-003/DI003-I38-033** [SD] (DEEP-I-003.discovery.md:L657| unit=I-38): [I][lifecycle] Because floor_check exits before compute, absence of QdeltaMeasured is the fail-closed evidence rather than a failure event row. Basis: I-38_qdelta_statistical_floor.md:25-27,91-93.

#### I-003 / unit `I-41` (45 records)

- **I-003/DI003-I41-001** [SRC] (DEEP-I-003.discovery.md:L665| unit=I-41): [E][status] Track 19; PLURAL-DOUBT; MODERATE; BUILD; effort M; Wave 2; binds ID-4 and the ban on load-bearing LLM judges. Anchor: I-41_spec_time_critique_lenses.md:3.
- **I-003/DI003-I41-002** [SRC] (DEEP-I-003.discovery.md:L667| unit=I-41): [E][timing] Lenses run on prose/specifications before deterministic artifacts exist. Anchor: I-41_spec_time_critique_lenses.md:4-8.
- **I-003/DI003-I41-003** [SRC] (DEEP-I-003.discovery.md:L669| unit=I-41): [E][trust] LLM-judged findings are noisy advisory input only; they never accept or block and do not replace off-agent gates. Anchor: I-41_spec_time_critique_lenses.md:8-11.
- **I-003/DI003-I41-004** [SRC] (DEEP-I-003.discovery.md:L671| unit=I-41): [E][lens] L1 Assumption Excavator names interface assumptions, classifies GROUNDED/VERIFY/INGEST, and verifies nothing itself. Anchor: I-41_spec_time_critique_lenses.md:14-16.
- **I-003/DI003-I41-005** [SRC] (DEEP-I-003.discovery.md:L673| unit=I-41): [E][lens] L2 is Implied Completeness Detector. Anchor: I-41_spec_time_critique_lenses.md:16.
- **I-003/DI003-I41-006** [AC] (DEEP-I-003.discovery.md:L675| unit=I-41): [E][lens] L3 Ambiguity Mapper composes B-38: the lens provides a human map while the scorer supplies the gate-time number. Anchor: I-41_spec_time_critique_lenses.md:16-18.
- **I-003/DI003-I41-007** [SRC] (DEEP-I-003.discovery.md:L677| unit=I-41): [E][lens] L4 is Gap Analyzer. Anchor: I-41_spec_time_critique_lenses.md:18.
- **I-003/DI003-I41-008** [SRC] (DEEP-I-003.discovery.md:L678| unit=I-41): [E][lens] L5 Pre-Implementation Architect flags a producer lacking an in-plan consumer and smoke trigger. Anchor: I-41_spec_time_critique_lenses.md:18-19.
- **I-003/DI003-I41-009** [SRC] (DEEP-I-003.discovery.md:L680| unit=I-41): [E][lens] L6 Public-Interface/API-Contract Validator mirrors cross_lang_diff and the E-45 action-type golden lock at design time. Anchor: I-41_spec_time_critique_lenses.md:19-20.
- **I-003/DI003-I41-010** [SRC] (DEEP-I-003.discovery.md:L682| unit=I-41): [E][lens] L7 is Documentation Validator. Anchor: I-41_spec_time_critique_lenses.md:20-21.
- **I-003/DI003-I41-011** [SRC] (DEEP-I-003.discovery.md:L684| unit=I-41): [E][lens] L8 Anxiety Reader is the catch-all and runs last. Anchor: I-41_spec_time_critique_lenses.md:21.
- **I-003/DI003-I41-012** [SRC] (DEEP-I-003.discovery.md:L686| unit=I-41): [E][schema] Finding fields are lens, severity, location, claim, why_it_matters, suggested_resolution, and confidence. Anchor: I-41_spec_time_critique_lenses.md:21-23.
- **I-003/DI003-I41-013** [SRC] (DEEP-I-003.discovery.md:L688| unit=I-41): [E][schema] Severity values are BLOCKER-CANDIDATE, CONCERN, and NIT. Anchor: I-41_spec_time_critique_lenses.md:21-22.
- **I-003/DI003-I41-014** [SRC] (DEEP-I-003.discovery.md:L690| unit=I-41): [E][grounding] Location is mandatory and may be a spec heading, file:line, symbol, or quoted phrase. Anchor: I-41_spec_time_critique_lenses.md:22-24.
- **I-003/DI003-I41-015** [SRC] (DEEP-I-003.discovery.md:L692| unit=I-41): [E][fail-safe] Anchorless findings are dropped rather than surfaced. Anchor: I-41_spec_time_critique_lenses.md:23-24.
- **I-003/DI003-I41-016** [SRC] (DEEP-I-003.discovery.md:L694| unit=I-41): [E][budget] Each lens emits at most eight findings per pass. Anchor: I-41_spec_time_critique_lenses.md:24.
- **I-003/DI003-I41-017** [SRC] (DEEP-I-003.discovery.md:L696| unit=I-41): [E][dedup] Findings deduplicate across lenses by location plus claim. Anchor: I-41_spec_time_critique_lenses.md:24-25.
- **I-003/DI003-I41-018** [SRC] (DEEP-I-003.discovery.md:L698| unit=I-41): [E][routing/config] SIMPLE uses 2–3 lenses; HEAVY uses the full eight. Anchor: I-41_spec_time_critique_lenses.md:25-26.
- **I-003/DI003-I41-019** [SRC] (DEEP-I-003.discovery.md:L700| unit=I-41): [E][non-goal] Do not import agents-and-pipelines; lenses are clean native reauthors to avoid the E-43 import gate. Anchor: I-41_spec_time_critique_lenses.md:26-28.
- **I-003/DI003-I41-020** [SRC] (DEEP-I-003.discovery.md:L702| unit=I-41): [E][non-goal] Do not run these prose-time lenses on shipped code. Anchor: I-41_spec_time_critique_lenses.md:28.
- **I-003/DI003-I41-021** [SRC] (DEEP-I-003.discovery.md:L704| unit=I-41): [E][integration/status] B-40, B-38, E-45/cross_lang_diff, and Phase 2B are shipped or plan-asserted composition seams; D-13/E-6/E-10 are distinct critique surfaces. Anchor: I-41_spec_time_critique_lenses.md:30-40.
- **I-003/DI003-I41-022** [SRC] (DEEP-I-003.discovery.md:L707| unit=I-41): [E][status] No LensSet/spec-time critique module exists; I-41 is net-new. Anchor: I-41_spec_time_critique_lenses.md:41-42.
- **I-003/DI003-I41-023** [SRC] (DEEP-I-003.discovery.md:L709| unit=I-41): [E][tier-1] Ship lenses.rs with eight prompts, LensSet, Finding, anchor drop, caps, and dedup. Anchor: I-41_spec_time_critique_lenses.md:44-47.
- **I-003/DI003-I41-024** [SRC] (DEEP-I-003.discovery.md:L711| unit=I-41 SURFACE): [E][CLI] elai critique accepts a spec path, optional lens list, and simple/heavy risk. Anchor: I-41_spec_time_critique_lenses.md:48-49.
- **I-003/DI003-I41-025** [SRC] (DEEP-I-003.discovery.md:L713| unit=I-41 SURFACE): [E][artifact] CLI output is workflow/reasoning/<artifact>-critique.md plus advisory SSB rows. Anchor: I-41_spec_time_critique_lenses.md:48-49.
- **I-003/DI003-I41-026** [SRC] (DEEP-I-003.discovery.md:L715| unit=I-41): [E][integration] B-40 idea-critic calls LensSet only as an advisory surfacer and never uses it for framing accept/reject. Anchor: I-41_spec_time_critique_lenses.md:50-51.
- **I-003/DI003-I41-027** [SRC] (DEEP-I-003.discovery.md:L717| unit=I-41): [E][schema/observability] CritiqueLensFinding is advisory=true with AdvisoryFinding, LensRun, and DroppedNoAnchor arms. Anchor: I-41_spec_time_critique_lenses.md:52,72-78.
- **I-003/DI003-I41-028** [SRC] (DEEP-I-003.discovery.md:L719| unit=I-41): [E][test/invariant] A contract proves no lens output reaches any gate-verdict path. Anchor: I-41_spec_time_critique_lenses.md:53.
- **I-003/DI003-I41-029** [SRC] (DEEP-I-003.discovery.md:L721| unit=I-41): [E][deferred] Automatic injection into the nine-step planning skill follows runtime proof. Anchor: I-41_spec_time_critique_lenses.md:55-57.
- **I-003/DI003-I41-030** [SRC] (DEEP-I-003.discovery.md:L723| unit=I-41): [E][deferred] Per-lens positive/negative eval JSONL waits for accumulated findings. Anchor: I-41_spec_time_critique_lenses.md:58.
- **I-003/DI003-I41-031** [SRC] (DEEP-I-003.discovery.md:L725| unit=I-41): [E][deferred] GEPA self-optimization waits for a labelled corpus. Anchor: I-41_spec_time_critique_lenses.md:59.
- **I-003/DI003-I41-032** [SRC] (DEEP-I-003.discovery.md:L727| unit=I-41 SURFACE): [E][surface] Create lenses.rs, critique_cmd.rs, the advisory-only contract, and decision doc; modify idea_framing. Anchor: I-41_spec_time_critique_lenses.md:61-67.
- **I-003/DI003-I41-033** [SRC] (DEEP-I-003.discovery.md:L729| unit=I-41 SURFACE): [E][UI/human-loop] Existing ReasoningDrawer or framing panel shows nonblocking findings; BLOCKER-CANDIDATE routes to a human and never auto-blocks. Anchor: I-41_spec_time_critique_lenses.md:68-70,97-100.
- **I-003/DI003-I41-034** [SRC] (DEEP-I-003.discovery.md:L732| unit=I-41): [E][identifiers] Produced symbols are LensSet, Finding, critique_cmd::run, CritiqueLensFinding, and the advisory-only contract. Anchor: I-41_spec_time_critique_lenses.md:72-79.
- **I-003/DI003-I41-035** [SRC] (DEEP-I-003.discovery.md:L735| unit=I-41): [E][dependency] LensSet consumes B-38, E-45/cross_lang_diff, Phase 2B VerificationMethod, B-40, and B-37 runtime. Anchor: I-41_spec_time_critique_lenses.md:80-86.
- **I-003/DI003-I41-036** [SRC] (DEEP-I-003.discovery.md:L737| unit=I-41 SURFACE): [E][reachability] Both CLI and idea-framing production paths emit findings; test-only reachability does not satisfy R7. Anchor: I-41_spec_time_critique_lenses.md:87-90.
- **I-003/DI003-I41-037** [SRC] (DEEP-I-003.discovery.md:L739| unit=I-41): [E][constraint] No verdict path, anchorless surfacing, primitive reimplementation, or upstream repository import is allowed. Anchor: I-41_spec_time_critique_lenses.md:91-96.
- **I-003/DI003-I41-038** [SRC] (DEEP-I-003.discovery.md:L741| unit=I-41): [E][constraint] Production paths ban unwrap/expect and non-blake3 hashes. Anchor: I-41_spec_time_critique_lenses.md:96.
- **I-003/DI003-I41-039** [SRC] (DEEP-I-003.discovery.md:L743| unit=I-41): [E][acceptance] Planted L1 and L4 issues emit anchored AdvisoryFinding rows; anchorless output emits DroppedNoAnchor; every row is advisory. Anchor: I-41_spec_time_critique_lenses.md:101-105.
- **I-003/DI003-I41-040** [SRC] (DEEP-I-003.discovery.md:L746| unit=I-41): [E][acceptance] Markdown omits anchorless findings and the run emits zero gate-verdict rows. Anchor: I-41_spec_time_critique_lenses.md:106-108.
- **I-003/DI003-I41-041** [SRC] (DEEP-I-003.discovery.md:L748| unit=I-41): [E][status] CritiqueLensFinding is baseline+1 from LIVE 221/532 and must be re-read at edit. Anchor: I-41_spec_time_critique_lenses.md:77-78,119.
- **I-003/DI003-I41-042** [SRC] (DEEP-I-003.discovery.md:L750| unit=I-41): [E][dependency] I-42 consumes LensSet; L8 remains distinct from trajectory/ship-time critics. Anchor: I-41_spec_time_critique_lenses.md:121-127.
- **I-003/DI003-I41-043** [AC] (DEEP-I-003.discovery.md:L752| unit=I-41 SURFACE): [A][UI] The exact existing drawer is unresolved between ReasoningDrawer and B-40's framing panel. Anchor: I-41_spec_time_critique_lenses.md:68-70.
- **I-003/DI003-I41-044** [AC] (DEEP-I-003.discovery.md:L754| unit=I-41): [A][dedup] The source gives a location/claim key but no normalization, cross-pass persistence, or semantic-duplicate policy. Anchor: I-41_spec_time_critique_lenses.md:24-25.
- **I-003/DI003-I41-045** [INF] (DEEP-I-003.discovery.md:L756| unit=I-41): [I][trust] BLOCKER-CANDIDATE is severity/routing metadata, not a gate outcome; the human may act, but the lens itself cannot block. Basis: I-41_spec_time_critique_lenses.md:21-24,68-70.

#### I-003 / unit `I-50` (29 records)

- **I-003/DI003-I50-001** [SRC] (DEEP-I-003.discovery.md:L764| unit=I-50): [E][status] Track 20; REACH; STRONG for SL-3 and MEDIUM for gajae semantics/SL-10; EXTENSION; effort M; Wave 4. Anchor: I-50_subscription_auth_broker.md:3-4.
- **I-003/DI003-I50-002** [SRC] (DEEP-I-003.discovery.md:L766| unit=I-50): [E][capability] One broker/gateway/storage abstraction owns token acquisition, refresh, storage, and provider linkage. Anchor: I-50_subscription_auth_broker.md:6-11.
- **I-003/DI003-I50-003** [SRC] (DEEP-I-003.discovery.md:L768| unit=I-50): [E][dependency/done-gate] Executor must read the named gajae auth-broker, auth-gateway, and auth-storage sources before EXECUTING. Anchor: I-50_subscription_auth_broker.md:13-15.
- **I-003/DI003-I50-004** [SRC] (DEEP-I-003.discovery.md:L771| unit=I-50): [E][API] AuthProvider exposes acquire, refresh, revoke, and status. Anchor: I-50_subscription_auth_broker.md:16-17.
- **I-003/DI003-I50-005** [SRC] (DEEP-I-003.discovery.md:L773| unit=I-50): [E][lifecycle] auth_broker.rs owns token lifecycle and storage through a broker/storage facade. Anchor: I-50_subscription_auth_broker.md:16-20,45-47.
- **I-003/DI003-I50-006** [SRC] (DEEP-I-003.discovery.md:L775| unit=I-50): [E][migration] Existing Anthropic oauth.rs becomes the first implementation behind the trait. Anchor: I-50_subscription_auth_broker.md:17-18.
- **I-003/DI003-I50-007** [SRC] (DEEP-I-003.discovery.md:L777| unit=I-50): [E][compatibility] Migration preserves token-file bytes and refresh semantics so existing users observe no behavior change. Anchor: I-50_subscription_auth_broker.md:18-20.
- **I-003/DI003-I50-008** [SRC] (DEEP-I-003.discovery.md:L779| unit=I-50): [E][storage/security] Credentials remain in the current oauth.rs storage location unless the user confirms a new backend. Anchor: I-50_subscription_auth_broker.md:19-20.
- **I-003/DI003-I50-009** [SRC] (DEEP-I-003.discovery.md:L781| unit=I-50): [E][non-goal] This session adds no AWS/Azure or second-provider flow. Anchor: I-50_subscription_auth_broker.md:21-22.
- **I-003/DI003-I50-010** [SRC] (DEEP-I-003.discovery.md:L783| unit=I-50): [E][non-goal/dependency] No keychain/credential-manager dependency is added. Anchor: I-50_subscription_auth_broker.md:22-23.
- **I-003/DI003-I50-011** [SRC] (DEEP-I-003.discovery.md:L785| unit=I-50 SURFACE): [E][non-goal] CLI-bridge authentication remains vendor-CLI-based and unchanged. Anchor: I-50_subscription_auth_broker.md:23.
- **I-003/DI003-I50-012** [SRC] (DEEP-I-003.discovery.md:L787| unit=I-50): [E][status] oauth.rs is shipped and Anthropic-only; existing runtimes/adapters are kept, not replaced. Anchor: I-50_subscription_auth_broker.md:25-27.
- **I-003/DI003-I50-013** [SRC] (DEEP-I-003.discovery.md:L789| unit=I-50): [E][tier-1] Ship the trait, broker, and byte-compatible Anthropic migration. Anchor: I-50_subscription_auth_broker.md:29-31.
- **I-003/DI003-I50-014** [SRC] (DEEP-I-003.discovery.md:L791| unit=I-50): [E][deferred] AWS/Azure credentials and any second subscription provider are deferred with I-49 Tier-2. Anchor: I-50_subscription_auth_broker.md:31.
- **I-003/DI003-I50-015** [SRC] (DEEP-I-003.discovery.md:L793| unit=I-50): [E][integration] Refactor oauth.rs behind auth_broker, migrate adapter call sites, and make runtime adapters read credentials through the broker. Anchor: I-50_subscription_auth_broker.md:33-36.
- **I-003/DI003-I50-016** [SRC] (DEEP-I-003.discovery.md:L796| unit=I-50 SURFACE): [E][UI] Existing RuntimePanel/SettingsWindow rows show per-provider authentication status; no new component is required. Anchor: I-50_subscription_auth_broker.md:35-36.
- **I-003/DI003-I50-017** [SRC] (DEEP-I-003.discovery.md:L798| unit=I-50 SURFACE): [E][UI/security] OpenCode may inspire Providers/Connect/verified-status layout only; implementation is clean-room and credential replay is forbidden. Anchor: I-50_subscription_auth_broker.md:37-43.
- **I-003/DI003-I50-018** [SRC] (DEEP-I-003.discovery.md:L801| unit=I-50 SURFACE): [E][coordination] J-30 owns the wider BYOK/custom-endpoint panel sharing the surface. Anchor: I-50_subscription_auth_broker.md:42-43.
- **I-003/DI003-I50-019** [SRC] (DEEP-I-003.discovery.md:L803| unit=I-50): [E][schema/observability] AuthBrokerEvent kinds are TokenAcquired, Refreshed, ProviderLinked, and RefreshFailed. Anchor: I-50_subscription_auth_broker.md:45-47,69-73.
- **I-003/DI003-I50-020** [SRC] (DEEP-I-003.discovery.md:L805| unit=I-50): [E][reachability] Every authenticating adapter's production request builder, at minimum Anthropic API, must flow through the broker; contracts-only callers do not satisfy R7. Anchor: I-50_subscription_auth_broker.md:48-52.
- **I-003/DI003-I50-021** [SRC] (DEEP-I-003.discovery.md:L808| unit=I-50): [E][privacy] Journal rows contain provider ID and event kind but never token material. Anchor: I-50_subscription_auth_broker.md:53-55.
- **I-003/DI003-I50-022** [SRC] (DEEP-I-003.discovery.md:L810| unit=I-50): [E][failure] Refresh failures are typed, emit RefreshFailed, and may not be silent. Anchor: I-50_subscription_auth_broker.md:53-55.
- **I-003/DI003-I50-023** [SRC] (DEEP-I-003.discovery.md:L812| unit=I-50): [E][constraint] Production paths ban behavior drift, new storage locations, unwrap, and expect. Anchor: I-50_subscription_auth_broker.md:53-55.
- **I-003/DI003-I50-024** [SRC] (DEEP-I-003.discovery.md:L814| unit=I-50): [E][acceptance] A real Anthropic request emits TokenAcquired/Refreshed; an expired fixture refreshes then succeeds; a failed refresh emits a typed error without hanging. Anchor: I-50_subscription_auth_broker.md:56-60.
- **I-003/DI003-I50-025** [SRC] (DEEP-I-003.discovery.md:L817| unit=I-50): [E][acceptance] Token-store fixtures remain byte-identical, token-material journal grep is empty, and successful auth without a broker row fails. Anchor: I-50_subscription_auth_broker.md:61-63.
- **I-003/DI003-I50-026** [SRC] (DEEP-I-003.discovery.md:L820| unit=I-50): [E][dependency] Auth events join I-18's chained record; I-49 consumes future credential implementations. Anchor: I-50_subscription_auth_broker.md:69-71.
- **I-003/DI003-I50-027** [SRC] (DEEP-I-003.discovery.md:L822| unit=I-50): [E][status] AuthBrokerEvent is baseline+1 from LIVE 221/532 and must be re-read at edit. Anchor: I-50_subscription_auth_broker.md:71-73.
- **I-003/DI003-I50-028** [AC] (DEEP-I-003.discovery.md:L824| unit=I-50): [A][API/schema] AuthProvider method signatures, token status model, storage facade schema, and refresh retry policy are not specified. Anchors: I-50_subscription_auth_broker.md:16-20,45-49.
- **I-003/DI003-I50-029** [INF] (DEEP-I-003.discovery.md:L826| unit=I-50): [I][scope] “Multi-provider” describes the extensible seam, not current provider breadth; the only shipped implementation in this session is Anthropic. Basis: I-50_subscription_auth_broker.md:16-23,29-31.

#### I-003 / unit `I-52` (31 records)

- **I-003/DI003-I52-001** [SRC] (DEEP-I-003.discovery.md:L835| unit=I-52): [E][status] Track 20; CONCURRENCY-SAFETY; STRONG SL-6; ACTIVATION; effort S-M; Wave 3. Anchor: I-52_checkpoint_rewind_tool.md:3-4.
- **I-003/DI003-I52-002** [SRC] (DEEP-I-003.discovery.md:L837| unit=I-52 SURFACE): [E][activation] Existing edit-checkpoint machinery and CLI/contracts ship, but agents cannot call it; I-52 exposes it rather than building a store. Anchor: I-52_checkpoint_rewind_tool.md:6-12.
- **I-003/DI003-I52-003** [SRC] (DEEP-I-003.discovery.md:L840| unit=I-52): [E][scope] The tool supports cheap single-agent, in-branch speculation; I-2 remains the parallel-branch mechanism. Anchor: I-52_checkpoint_rewind_tool.md:10-12.
- **I-003/DI003-I52-004** [SRC] (DEEP-I-003.discovery.md:L842| unit=I-52): [E][dependency/done-gate] Executor reads gajae checkpoint.ts for ergonomics before EXECUTING. Anchor: I-52_checkpoint_rewind_tool.md:14-16.
- **I-003/DI003-I52-005** [SRC] (DEEP-I-003.discovery.md:L844| unit=I-52): [E][tool] save creates a labelled snapshot through edit_checkpoints. Anchor: I-52_checkpoint_rewind_tool.md:16-18.
- **I-003/DI003-I52-006** [SRC] (DEEP-I-003.discovery.md:L846| unit=I-52): [E][tool] rewind restores a label subject to divergence and session guards. Anchor: I-52_checkpoint_rewind_tool.md:17-19.
- **I-003/DI003-I52-007** [SRC] (DEEP-I-003.discovery.md:L848| unit=I-52): [E][tool] diff reports changes since a label. Anchor: I-52_checkpoint_rewind_tool.md:18.
- **I-003/DI003-I52-008** [SRC] (DEEP-I-003.discovery.md:L850| unit=I-52): [E][safety] Rewind refuses if files outside the checkpointed set diverged. Anchor: I-52_checkpoint_rewind_tool.md:17-18.
- **I-003/DI003-I52-009** [SRC] (DEEP-I-003.discovery.md:L852| unit=I-52): [E][security] An agent may rewind only checkpoints from its own session. Anchor: I-52_checkpoint_rewind_tool.md:18-19.
- **I-003/DI003-I52-010** [SRC] (DEEP-I-003.discovery.md:L854| unit=I-52 SURFACE): [E][coordination] If H-49 ships first, I-52 becomes a tool surface over H-49's store and reuses EditHistoryEvent. Anchor: I-52_checkpoint_rewind_tool.md:20-23.
- **I-003/DI003-I52-011** [SRC] (DEEP-I-003.discovery.md:L856| unit=I-52): [E][invariant] No implementation order permits a second snapshot store. Anchor: I-52_checkpoint_rewind_tool.md:21-23.
- **I-003/DI003-I52-012** [SRC] (DEEP-I-003.discovery.md:L858| unit=I-52 SURFACE): [E][non-goal] Redo/branching semantics, git-worktree integration, cross-session travel, and new UI are excluded. Anchor: I-52_checkpoint_rewind_tool.md:24-25.
- **I-003/DI003-I52-013** [SRC] (DEEP-I-003.discovery.md:L860| unit=I-52): [E][status] edit_checkpoints.rs, checkpoint_cmd.rs, and two E-25 contract tests exist. Anchor: I-52_checkpoint_rewind_tool.md:27-30.
- **I-003/DI003-I52-014** [SRC] (DEEP-I-003.discovery.md:L862| unit=I-52 SURFACE): [E][flag/default] If the substrate is still OFF, activation flips it per W-2: default-ON behind an ELAI_*_DISABLE flag or an allowlist row. Anchor: I-52_checkpoint_rewind_tool.md:30-32.
- **I-003/DI003-I52-015** [SRC] (DEEP-I-003.discovery.md:L865| unit=I-52): [E][lifecycle/status] Executor must recheck whether file_ops still discards preimages and whether the gate remains off. Anchor: I-52_checkpoint_rewind_tool.md:30-32.
- **I-003/DI003-I52-016** [SRC] (DEEP-I-003.discovery.md:L867| unit=I-52): [E][tier-1] Ship the in-process three-arm tool, session guard, divergence refusal, and contract test. Anchor: I-52_checkpoint_rewind_tool.md:34-36.
- **I-003/DI003-I52-017** [SRC] (DEEP-I-003.discovery.md:L869| unit=I-52): [E][deferred] H-49 store migration and automatic checkpoints before risky edits are deferred. Anchor: I-52_checkpoint_rewind_tool.md:36.
- **I-003/DI003-I52-018** [SRC] (DEEP-I-003.discovery.md:L871| unit=I-52): [E][integration] Register through the existing three-arm catalog and extend edit_checkpoints only with label/scope lookup. Anchor: I-52_checkpoint_rewind_tool.md:38-40.
- **I-003/DI003-I52-019** [SRC] (DEEP-I-003.discovery.md:L873| unit=I-52 SURFACE): [E][UI] Checkpoint rows use the existing session/transcript drawer; no component is added. Anchor: I-52_checkpoint_rewind_tool.md:40-41.
- **I-003/DI003-I52-020** [SRC] (DEEP-I-003.discovery.md:L875| unit=I-52): [E][schema/observability] Default event is CheckpointToolEvent with Saved, Rewound, Diffed, and RefusedDiverged; it is conditional on H-49 event reuse. Anchor: I-52_checkpoint_rewind_tool.md:43-45,65-70.
- **I-003/DI003-I52-021** [SRC] (DEEP-I-003.discovery.md:L878| unit=I-52): [E][dependency] Consume edit_checkpoints store, catalog registration, and file_ops preimage capture; never reimplement the store. Anchor: I-52_checkpoint_rewind_tool.md:46-47.
- **I-003/DI003-I52-022** [SRC] (DEEP-I-003.discovery.md:L880| unit=I-52): [E][reachability] A live worker's production tool dispatch must reach the tool; contracts-only callers do not satisfy R7. Anchor: I-52_checkpoint_rewind_tool.md:48-49.
- **I-003/DI003-I52-023** [SRC] (DEEP-I-003.discovery.md:L882| unit=I-52): [E][failure] Divergence refusal names the complete divergent-path set and may not silently overwrite files. Anchor: I-52_checkpoint_rewind_tool.md:50-52,55-59.
- **I-003/DI003-I52-024** [SRC] (DEEP-I-003.discovery.md:L884| unit=I-52): [E][constraint] Production paths ban unwrap/expect and claiming activation while the substrate remains gated off. Anchor: I-52_checkpoint_rewind_tool.md:50-52.
- **I-003/DI003-I52-025** [SRC] (DEEP-I-003.discovery.md:L886| unit=I-52): [E][observability] save then rewind emits Saved then Rewound with the file set; divergence emits a refusal naming the path. Anchor: I-52_checkpoint_rewind_tool.md:53-56.
- **I-003/DI003-I52-026** [SRC] (DEEP-I-003.discovery.md:L888| unit=I-52): [E][acceptance] Post-rewind bytes must blake3-match checkpoint preimages. Anchor: I-52_checkpoint_rewind_tool.md:57.
- **I-003/DI003-I52-027** [SRC] (DEEP-I-003.discovery.md:L890| unit=I-52): [E][broken-arm] Cross-session labels are refused; silent overwrite of a diverged file fails R2. Anchor: I-52_checkpoint_rewind_tool.md:57-59.
- **I-003/DI003-I52-028** [SRC] (DEEP-I-003.discovery.md:L892| unit=I-52): [E][dependency] H-49 supplies future divergence semantics, I-2 is the parallel sibling, and I-8 rollback-to-trunk remains outside this tool. Anchor: I-52_checkpoint_rewind_tool.md:65-68.
- **I-003/DI003-I52-029** [SRC] (DEEP-I-003.discovery.md:L894| unit=I-52): [E][status] CheckpointToolEvent is baseline+1 from LIVE 221/532 unless H-49 reuse fires; action registration must be re-read at edit. Anchor: I-52_checkpoint_rewind_tool.md:69-70.
- **I-003/DI003-I52-030** [AC] (DEEP-I-003.discovery.md:L896| unit=I-52): [A][event schema] Whether I-52 adds CheckpointToolEvent or reuses EditHistoryEvent depends on landing order and is explicitly decided at edit. Anchors: I-52_checkpoint_rewind_tool.md:20-23,43-45.
- **I-003/DI003-I52-031** [AC] (DEEP-I-003.discovery.md:L898| unit=I-52 SURFACE): [A][flag] The exact disable-flag name or allowlist-row schema is not specified. Anchor: I-52_checkpoint_rewind_tool.md:30-32.

#### I-003 / unit `misc` (31 records)

- **I-003/DI003-X-001** [IX] (DEEP-I-003.discovery.md:L903| unit=misc): [E][enforcement] I-1 creates the Supervisor-side execution boundary later consumed by I-35 and I-31; its Tier-1 process isolation remains incomplete. Refs: DI003-I01-013, DI003-I01-015, DI003-I01-029.
- **I-003/DI003-X-002** [IX] (DEEP-I-003.discovery.md:L906| unit=misc): [E][search chain] I-4 requires I-2 isolation and I-17 verdicts, consumes I-20 mutation receipts, and supplies real trajectories to I-9. Refs: DI003-I04-001, DI003-I04-004, DI003-I04-005, DI003-I09-017.
- **I-003/DI003-X-003** [IX] (DEEP-I-003.discovery.md:L909| unit=misc): [E][quality chain] I-20's sampled-survivor records both rank I-4 branches and populate the mutant leg of I-38 QDELTA. Refs: DI003-I20-024, DI003-I38-004, DI003-I38-022.
- **I-003/DI003-X-004** [IX] (DEEP-I-003.discovery.md:L911| unit=misc): [E][gated rollout] I-4 V1 precedes tournament/PUCT; I-9 baseline precedes production-wide live estimation; I-32 local A/B precedes prompt flip; I-38 fixture tier precedes full-corpus RunPod work. Refs: DI003-I04-025–027, DI003-I09-019–022, DI003-I32-008–009, DI003-I38-019.
- **I-003/DI003-X-005** [IX] (DEEP-I-003.discovery.md:L914| unit=misc): [E][local-baseline doctrine] I-9, I-20, I-32, I-33, and I-38 all prohibit promoting external/paper claims into policy before local evidence. Refs: DI003-I09-010, DI003-I20-010, DI003-I32-002, DI003-I33-012, DI003-I38-016.
- **I-003/DI003-X-006** [IX] (DEEP-I-003.discovery.md:L917| unit=misc): [E][determinism] I-4 search receipts, I-9 alpha, I-20 mutation CI, I-33 governance metrics, and I-38 QDELTA evidence are required to be seed-recomputable. Refs: DI003-I04-034, DI003-I09-006, DI003-I20-034, DI003-I33-025, DI003-I38-027.
- **I-003/DI003-X-007** [IX] (DEEP-I-003.discovery.md:L920| unit=misc): [E][SSB lifecycle] Every source introduces or conditionally reuses an SSB action family; all baseline counts are provisional and must be re-read at edit. Refs: DI003-I01-027, DI003-I04-038, DI003-I09-029, DI003-I20-037, DI003-I32-025, DI003-I33-027, DI003-I38-029, DI003-I41-041, DI003-I50-027, DI003-I52-029.
- **I-003/DI003-X-008** [IX] (DEEP-I-003.discovery.md:L924| unit=misc): [E][advisory boundary] I-41 findings never decide, while I-4 and I-38 make selection/reporting authoritative only through deterministic receipts and floor checks. Refs: DI003-I41-003, DI003-I04-013–014, DI003-I38-012.
- **I-003/DI003-X-009** [IX] (DEEP-I-003.discovery.md:L927| unit=misc): [E][provider chain] I-50 preserves current Anthropic subscription auth while providing the future credential seam for I-49; auth events join I-18's chained record. Refs: DI003-I50-006–008, DI003-I50-014, DI003-I50-026.
- **I-003/DI003-X-010** [IX] (DEEP-I-003.discovery.md:L930| unit=misc): [E][rewind coordination] I-52 is single-agent/in-branch, I-2 remains parallel isolation, H-49 may own the shared store/events, and I-8 remains rollback-to-trunk. Refs: DI003-I52-003, DI003-I52-010–012, DI003-I52-028.
- **I-003/DI003-X-011** [IX] (DEEP-I-003.discovery.md:L933| unit=misc SURFACE): [E][UI reuse/new surfaces] I-1, I-20, I-32, I-41, I-50, and I-52 reuse existing surfaces; I-33 and I-38 require new metric tiles; I-4 and I-9 defer their principal visualizations. Refs: DI003-I01-024, DI003-I04-029, DI003-I09-021, DI003-I20-026, DI003-I32-019, DI003-I33-021, DI003-I38-023, DI003-I41-033, DI003-I50-016, DI003-I52-019.
- **I-003/DI003-X-012** [IX] (DEEP-I-003.discovery.md:L937| unit=misc): [E][concurrency] I-4 uses isolated candidate and detached eval worktrees; I-52 explicitly avoids worktree overhead and cannot replace I-2 parallelism. Refs: DI003-I04-004, DI003-I04-040, DI003-I04-052, DI003-I52-003, DI003-I52-012.
- **I-003/REG-001** [CON] (DEEP-I-003.discovery.md:L945| unit=misc): [C] I-1's target architecture versus Tier-1 process/credential scope. Ref: DI003-I01-031.
- **I-003/REG-002** [AC] (DEEP-I-003.discovery.md:L946| unit=misc SURFACE): [A] I-1's exact existing drawer. Ref: DI003-I01-032.
- **I-003/REG-003** [AC] (DEEP-I-003.discovery.md:L947| unit=misc): [A] I-4's N/K notation and unspecified candidate count. Ref: DI003-I04-059.
- **I-003/REG-004** [AC] (DEEP-I-003.discovery.md:L948| unit=misc): [A] I-4's plan-asserted H-15/SessionDag seams. Ref: DI003-I04-060.
- **I-003/REG-005** [CON] (DEEP-I-003.discovery.md:L949| unit=misc): [C] I-9 ships a mid-run hook while deferring live per-task estimation. Ref: DI003-I09-030.
- **I-003/REG-006** [CON] (DEEP-I-003.discovery.md:L950| unit=misc SURFACE): [C] I-9 grammar and UI are deferred despite summary-level capability wording. Ref: DI003-I09-031.
- **I-003/REG-007** [AC] (DEEP-I-003.discovery.md:L952| unit=misc): [A] I-9 annealing/stability values are unspecified. Ref: DI003-I09-032.
- **I-003/REG-008** [AC] (DEEP-I-003.discovery.md:L953| unit=misc): [A] I-20 graph API is selected only at execution. Ref: DI003-I20-020.
- **I-003/REG-009** [AC] (DEEP-I-003.discovery.md:L954| unit=misc): [C] I-20's fail-early CI-bound wording is statistically ambiguous. Ref: DI003-I20-039.
- **I-003/REG-010** [CON] (DEEP-I-003.discovery.md:L955| unit=misc): [C] I-20 does not own bounded retries. Ref: DI003-I20-040.
- **I-003/REG-011** [CON] (DEEP-I-003.discovery.md:L956| unit=misc): [C] I-20 uses two different cache surfaces. Ref: DI003-I20-041.
- **I-003/REG-012** [AC] (DEEP-I-003.discovery.md:L957| unit=misc): [A] I-20 numeric budgets/thresholds are unspecified. Ref: DI003-I20-042.
- **I-003/REG-013** [AC] (DEEP-I-003.discovery.md:L958| unit=misc): [A] I-32's broad factual/code rule versus three-role Tier-1 list. Ref: DI003-I32-027.
- **I-003/REG-014** [AC] (DEEP-I-003.discovery.md:L959| unit=misc): [A] I-32 A/B statistical details are unspecified. Ref: DI003-I32-028.
- **I-003/REG-015** [AC] (DEEP-I-003.discovery.md:L960| unit=misc): [A] I-33 I6Q numeric configuration and complete call-site census are unspecified. Refs: DI003-I33-029, DI003-I33-030.
- **I-003/REG-016** [AC] (DEEP-I-003.discovery.md:L962| unit=misc): [A] I-38 statistical and contamination rejection details are unspecified. Refs: DI003-I38-031, DI003-I38-032.
- **I-003/REG-017** [AC] (DEEP-I-003.discovery.md:L964| unit=misc SURFACE): [A] I-41 drawer and dedup normalization are unresolved. Refs: DI003-I41-043, DI003-I41-044.
- **I-003/REG-018** [AC] (DEEP-I-003.discovery.md:L966| unit=misc): [A] I-50 method/schema/retry details are unresolved. Ref: DI003-I50-028.
- **I-003/REG-019** [AC] (DEEP-I-003.discovery.md:L967| unit=misc SURFACE): [A] I-52 event-family and flag schema depend on edit-time state. Refs: DI003-I52-030, DI003-I52-031.

#### I-003 / unit `RELATIONAL` (17 records)

- **I-003/SD-I01-01** [SD] (DEEP-I-003.discovery.md:L976| unit=RELATIONAL): [CORRECTION] Summary I-1.02/I-1.05 flatten the target architecture into current scope; Tier-1 is an in-process stub and the OS split/credential vault are deferred. Refs: DI003-I01-013, DI003-I01-015–016, DI003-I01-031.
- **I-003/SD-I01-02** [SD] (DEEP-I-003.discovery.md:L979| unit=RELATIONAL SURFACE): [ADDITION] Exact Directive fields, event arms, production reachability, Tier-1 UI posture, and full acceptance/bans were absent. Refs: DI003-I01-003, DI003-I01-014, DI003-I01-021–026.
- **I-003/SD-I04-01** [SD] (DEEP-I-003.discovery.md:L981| unit=RELATIONAL): [CORRECTION] Candidate notation is not a declared N constant, and V2/V3 are gated—not current ship scope. Refs: DI003-I04-025–028, DI003-I04-059, DI003-I04-061.
- **I-003/SD-I04-02** [SD] (DEEP-I-003.discovery.md:L983| unit=RELATIONAL): [ADDITION] Deterministic V1 ranking precedence, all SearchNodeEvent kinds, exact rollout evidence, and binary-absence gate were omitted. Refs: DI003-I04-006, DI003-I04-024–027, DI003-I04-034–037.
- **I-003/SD-I04-03** [SD] (DEEP-I-003.discovery.md:L986| unit=RELATIONAL SURFACE): [ADDITION] The full J-11 held-out merge gate—hidden B_test, detached pool, fail-closed command, frozen manifest, deterministic parser, merge recovery, path/output guards—was omitted. Refs: DI003-I04-040–048.
- **I-003/SD-I04-04** [SD] (DEEP-I-003.discovery.md:L989| unit=RELATIONAL): [ADDITION] J-11 worktree lifecycle and prompt do/do-not-port rules were omitted. Refs: DI003-I04-049–058.
- **I-003/SD-I09-01** [SD] (DEEP-I-003.discovery.md:L991| unit=RELATIONAL SURFACE): [CORRECTION] Summary I-9.07/I-9.11 present grammar tightening and the tile without their Tier-2 gates; V1 is temperature-only and the tile needs three baselines. Refs: DI003-I09-021–022, DI003-I09-031.
- **I-003/SD-I09-02** [SD] (DEEP-I-003.discovery.md:L994| unit=RELATIONAL): [ADDITION] Embedded-fixture determinism, bounded cooling, baseline-first artifact, explicit non-goals, and the live-estimation contradiction were omitted. Refs: DI003-I09-008, DI003-I09-011, DI003-I09-013–014, DI003-I09-019, DI003-I09-030.
- **I-003/SD-I20-01** [SD] (DEEP-I-003.discovery.md:L997| unit=RELATIONAL): [CORRECTION] Summary I-20.08 conflates reachability and mutant caches; summary I-20.14 over-assigns retry ownership. Refs: DI003-I20-005, DI003-I20-011, DI003-I20-015, DI003-I20-040–041.
- **I-003/SD-I20-02** [SD] (DEEP-I-003.discovery.md:L1000| unit=RELATIONAL): [ADDITION] Sampling-without-replacement, Wilson CI, cache migration, first-deploy baseline, exact event arms, deferred engines, and CI-bound ambiguity were omitted. Refs: DI003-I20-006–012, DI003-I20-017, DI003-I20-022–025, DI003-I20-039.
- **I-003/SD-I32-01** [SD] (DEEP-I-003.discovery.md:L1003| unit=RELATIONAL): [ADDITION] The broad factual/code-role rule versus concrete three-role Tier-1 list, max-tokens=0 mechanism, complete retained roles/event kinds, and deferred cross-role/CI work were omitted. Refs: DI003-I32-003, DI003-I32-006–007, DI003-I32-013–017, DI003-I32-027.
- **I-003/SD-I33-01** [SD] (DEEP-I-003.discovery.md:L1006| unit=RELATIONAL SURFACE): [ADDITION] Orchestrator-owned I6Q config, exact commitment formula/order proof, selection-by-negation guarantee, D-16 routing, mandatory UI, and deferred calibration/correlation were omitted. Refs: DI003-I33-004, DI003-I33-007–009, DI003-I33-018–021.
- **I-003/SD-I38-01** [SD] (DEEP-I-003.discovery.md:L1009| unit=RELATIONAL): [ADDITION] Pre-compute refusal/no-row evidence, exact payload/config identifiers, D-20 pre-clean execution, 13-gram reuse, full-corpus/cross-repo gates, and unresolved statistical parameters were omitted. Refs: DI003-I38-012–013, DI003-I38-016–022, DI003-I38-025–032.
- **I-003/SD-I41-01** [SD] (DEEP-I-003.discovery.md:L1012| unit=RELATIONAL): [ADDITION] Per-lens semantics, complete Finding/severity schema, 2–3 versus 8 risk routing, LensRun arm, output artifact, and native-reauthor/code non-goals were omitted. Refs: DI003-I41-004–020, DI003-I41-024–027.
- **I-003/SD-I41-02** [SD] (DEEP-I-003.discovery.md:L1015| unit=RELATIONAL SURFACE): [ADDITION] Deferred planning-skill injection, eval corpora, GEPA optimization, exact production reachability, and UI/dedup ambiguities were omitted. Refs: DI003-I41-029–036, DI003-I41-043–044.
- **I-003/SD-I50-01** [SD] (DEEP-I-003.discovery.md:L1018| unit=RELATIONAL SURFACE): [ADDITION] No CLI-bridge change/keychain dependency, existing-location rule, AWS/Azure deferral, ProviderLinked arm, clean-room UI constraint, all-adapter reachability, and no-hang failure evidence were omitted. Refs: DI003-I50-008–011, DI003-I50-014, DI003-I50-017, DI003-I50-019–024.
- **I-003/SD-I52-01** [SD] (DEEP-I-003.discovery.md:L1022| unit=RELATIONAL): [ADDITION] Exact W-2 default posture, preimage/gate recheck, conditional EditHistoryEvent reuse, Diffed/RefusedDiverged arms, redo/worktree exclusions, and auto-checkpoint deferral were omitted. Refs: DI003-I52-010–017, DI003-I52-020.

### Ledger I-004 — I-2, I-3, I-12, I-21, I-27, I-34, I-35, I-40, I-42, I-45

Source: `docs/thefinalglance/feature-discovery/deep/DEEP-I-004.discovery.md`
Extracted atoms: **469**

#### I-004 / unit `I-2` (49 records)

- **I-004/I2-E001** [SRC] (DEEP-I-004.discovery.md:L37| unit=I-2): E/STATUS | I-2 is a strong-grounded Track-12 BUILD, concurrency lever, effort M-L, Wave 2, bound to ID-2, and dependent on I-1. | anchor=I-2:L1-L3
- **I-004/I2-E002** [SRC] (DEEP-I-004.discovery.md:L38| unit=I-2): E/CONC | The motivating defect is that parallel tool-calling branches share a mutable working directory, can silently overwrite intermediate state, and therefore make wide search and merge results unsafe/non-deterministic. | anchor=I-2:L4-L9
- **I-004/I2-E003** [SRC] (DEEP-I-004.discovery.md:L39| unit=I-2): E/CAP | Tier 1 creates `crates/orchestrator/src/sandbox/branch_workspace.rs::BranchWorkspace` with `spawn`, `escape_check`, and `barrier_merge`, plus `WorktreePoolImpl`. | anchor=I-2:L38-L45, L53-L58, L63-L69
- **I-004/I2-E004** [SRC] (DEEP-I-004.discovery.md:L40| unit=I-2): E/FLOW | Every branch/agent receives a private workspace root; production branch dispatch is `task_runner.rs -> Supervisor -> BranchWorkspace::spawn`. | anchor=I-2:L6-L9, L56-L57, L77-L81
- **I-004/I2-E005** [SRC] (DEEP-I-004.discovery.md:L41| unit=I-2): E/CFG | The cross-platform Tier-1 default is a pooled warm git-worktree implementation with shared Cargo target-dir: source is isolated while build artifacts are shared. | anchor=I-2:L12-L15, L40-L42
- **I-004/I2-E006** [SRC] (DEEP-I-004.discovery.md:L42| unit=I-2): E/DEFER | Linux overlayfs/btrfs-reflink CoW is a separately gated Tier-2 implementation behind the same trait, not a cross-platform assumption. | anchor=I-2:L12-L14, L47-L48, L83-L85
- **I-004/I2-E007** [SRC] (DEEP-I-004.discovery.md:L43| unit=I-2): E/SEC | The cloud-sandbox/E2B-class adapter is optional, never a hard dependency, Tier-2, and user-approval gated. | anchor=I-2:L14-L15, L47-L50, L84-L85
- **I-004/I2-E008** [SRC] (DEEP-I-004.discovery.md:L44| unit=I-2): E/FLOW | Merges occur only at the existing explicit BSP barrier; `bsp/engine.rs` must compose `BranchWorkspace::barrier_merge` rather than reimplement barrier logic. | anchor=I-2:L8-L9, L15-L17, L52-L55, L72-L73, L79-L80, L89-L90
- **I-004/I2-E009** [SRC] (DEEP-I-004.discovery.md:L45| unit=I-2): E/DET | Barrier inputs are ordered exclusively by SSB sequence; any other merge order is a block. | anchor=I-2:L15-L17, L42-L43, L90-L90
- **I-004/I2-E010** [SRC] (DEEP-I-004.discovery.md:L46| unit=I-2): E/RECOVERY | Overlapping hunks produce `ConflictEscalated`, halt the merge, and restore the trunk through the existing checkpoint/H-49 restore semantics. | anchor=I-2:L16-L18, L31-L34, L43-L44, L71-L76, L97-L103, L116-L122
- **I-004/I2-E011** [SRC] (DEEP-I-004.discovery.md:L47| unit=I-2): E/FLOW | The stated non-overlap policy is “last-seq wins on non-overlapping hunks”; overlap is never auto-resolved. | anchor=I-2:L15-L18
- **I-004/I2-E012** [SRC] (DEEP-I-004.discovery.md:L48| unit=I-2): E/DET | An identical-seed rerun must produce a byte-identical merged file under GD-2. | anchor=I-2:L17-L19, L44-L45, L96-L103
- **I-004/I2-E013** [SRC] (DEEP-I-004.discovery.md:L49| unit=I-2): E/SEC | An absolute-path write outside a branch root must be intercepted by the Supervisor escape gate before filesystem mutation and emit `EscapeBlocked`. | anchor=I-2:L8-L9, L18-L19, L40-L45, L56-L57, L96-L103
- **I-004/I2-E014** [SRC] (DEEP-I-004.discovery.md:L50| unit=I-2): E/EVENT | I-2 adds one `BranchWorkspaceEvent` action type with discriminator arms `Spawned`, `Merged`, `EscapeBlocked`, and `ConflictEscalated`, expressed as baseline+1 from the dated 221/532 registry. | anchor=I-2:L42-L45, L63-L69, L112-L113
- **I-004/I2-E015** [SRC] (DEEP-I-004.discovery.md:L51| unit=I-2 SURFACE): E/SURFACE | Existing C-23 `BranchTree` is extended with workspace badges from `Spawned`/`Merged`/`EscapeBlocked` rows through its existing Tauri read; no new branch UI is built, and `cross_lang_diff.py` must pass. | anchor=I-2:L29-L30, L60-L61, L74-L75, L91-L94, L119-L120
- **I-004/I2-E016** [SRC] (DEEP-I-004.discovery.md:L52| unit=I-2): E/TEST | The contract test must prove disjoint roots, byte-identical deterministic reruns, and a pre-write-blocked escape fixture. | anchor=I-2:L44-L45, L58-L58, L96-L103
- **I-004/I2-E017** [SRC] (DEEP-I-004.discovery.md:L53| unit=I-2): E/TEST | A seeded-conflict run must emit `ConflictEscalated` followed by trunk rollback; a clean two-branch run must emit two `Spawned` rows and an SSB-ordered `Merged` row. | anchor=I-2:L95-L103
- **I-004/I2-E018** [SRC] (DEEP-I-004.discovery.md:L54| unit=I-2): E/OBS | Per-branch root prefixes are journaled so root disjointness is row-verifiable; a missing event row or non-deterministic merged output is an R1/R2 failure. | anchor=I-2:L95-L103
- **I-004/I2-E019** [SRC] (DEEP-I-004.discovery.md:L55| unit=I-2): E/STATUS | `bsp/engine.rs::recover_from_checkpoint` and a test exist, but production reachability was not verified in this plan pass and must be retraced under R7. | anchor=I-2:L23-L26, L71-L72, L105-L108
- **I-004/I2-E020** [SRC] (DEEP-I-004.discovery.md:L56| unit=I-2): E/STATUS | No branch-workspace module/layer existed in the cited workspace inventory, making this a genuine build. | anchor=I-2:L6-L7, L23-L26, L35-L36
- **I-004/I2-E021** [SRC] (DEEP-I-004.discovery.md:L57| unit=I-2 SURFACE): E/DEP | E-15 Firecracker pool, C-11 sandbox seam, and C-23 BranchTree are PLAN-asserted shipped dependencies that must be re-grepped at execution and composed, not rebuilt. | anchor=I-2:L27-L30, L73-L75, L118-L120
- **I-004/I2-E022** [SRC] (DEEP-I-004.discovery.md:L58| unit=I-2): E/DEP | H-49 divergence/restore semantics are PLAN-asserted planned; they own post-conflict rollback and must not be reimplemented by I-2. | anchor=I-2:L31-L32, L76-L76, L121-L121
- **I-004/I2-E023** [SRC] (DEEP-I-004.discovery.md:L59| unit=I-2): E/DEP | The Phase-3D BSP aggregator/barrier is PLAN-asserted shipped; its exact production barrier site must be re-grepped at execution. | anchor=I-2:L33-L34, L54-L55, L72-L73, L106-L108
- **I-004/I2-E024** [SRC] (DEEP-I-004.discovery.md:L60| unit=I-2): E/DATA | Tier 1 creates `crates/contracts/tests/i2_cow_workspace_contract.rs` and `workflow/reasoning/I-2-decisions.md` in addition to the runtime module. | anchor=I-2:L53-L59, L63-L69
- **I-004/I2-E025** [SRC] (DEEP-I-004.discovery.md:L61| unit=I-2): E/INT | Qualifying production reachability is the parallel task launch into `spawn` and the BSP barrier into `barrier_merge`; a test-only/contracts caller never satisfies R7. | anchor=I-2:L78-L81
- **I-004/I2-E026** [SRC] (DEEP-I-004.discovery.md:L62| unit=I-2): E/LIMIT | Shared mutable source workspaces, cross-platform overlayfs assumptions, and hard cloud-sandbox dependencies are prohibited. | anchor=I-2:L20-L21, L82-L85
- **I-004/I2-E027** [SRC] (DEEP-I-004.discovery.md:L63| unit=I-2): E/LIMIT | Production paths prohibit `.unwrap()`/`.expect()`, blake2/sha2, fabricated measurements/anchors, duplicate BSP barrier logic, and non-SSB merge order. | anchor=I-2:L86-L90
- **I-004/I2-E028** [SRC] (DEEP-I-004.discovery.md:L64| unit=I-2): E/DEFER | Cross-branch CRDT merge behavior beyond the sequence-ordered policy is deferred to I-29 diffusion drafts. | anchor=I-2:L47-L50
- **I-004/I2-E029** [SRC] (DEEP-I-004.discovery.md:L65| unit=I-2): E/DEP | I-4 parallel evidence-ranked search is explicitly blocked on I-2 workspace isolation. | anchor=I-2:L4-L6, L122-L123
- **I-004/I2-E030** [SRC] (DEEP-I-004.discovery.md:L66| unit=I-2): E/FLOW | The J-11 amendment adopts Arbor’s branch start point: each new worktree begins at current trunk HEAD so it includes prior merges, not original repository HEAD. | anchor=I-2:L125-L132
- **I-004/I2-E031** [SRC] (DEEP-I-004.discovery.md:L67| unit=I-2): E/DATA | Arbor prior art filters generated `.coordinator/**`, `results/**`, `models/**`, `logs/**`, and similar artifacts from finalize commits via `filter_commit_paths`. | anchor=I-2:L125-L135
- **I-004/I2-E032** [SRC] (DEEP-I-004.discovery.md:L68| unit=I-2): E/STATE | Worktree teardown removes the ephemeral worktree but preserves the branch as the durable later-merge artifact. | anchor=I-2:L133-L137
- **I-004/I2-E033** [SRC] (DEEP-I-004.discovery.md:L69| unit=I-2): E/CONC | Merge evaluation runs in a detached worktree pool distinct from the executor worktree pool. | anchor=I-2:L138-L140
- **I-004/I2-E034** [SRC] (DEEP-I-004.discovery.md:L70| unit=I-2): E/RECOVERY | Stale worktrees are force-removed before creation, and name collisions receive timestamp suffixes. | anchor=I-2:L141-L142
- **I-004/I2-E035** [SRC] (DEEP-I-004.discovery.md:L71| unit=I-2): E/OBS | Arbor’s configured after-executor hook snapshots the submission artifact; the amendment identifies this as I-2’s barrier-merge analogue. | anchor=I-2:L143-L144
- **I-004/I2-E036** [SRC] (DEEP-I-004.discovery.md:L72| unit=I-2): E/LIMIT | Arbor itself is Unix-only and does not pool worktrees; I-2 deliberately retains Windows-safe warm pooling as an improvement. | anchor=I-2:L145-L147
- **I-004/I2-E037** [SRC] (DEEP-I-004.discovery.md:L73| unit=I-2): E/EVIDENCE | The J-11 amendment is source-cited to exact Arbor executor/git-artifact/git-ops anchors and says it grounds lifecycle semantics without changing I-2 scope. | anchor=I-2:L125-L149
- **I-004/I2-I001** [INF] (DEEP-I-004.discovery.md:L74| unit=I-2): I/STATE | The amendment’s preserved branch is the implied durable handoff from an ephemeral executor worktree to later barrier/merge evaluation; no separate handoff object is named. | anchor=I-2:L133-L144
- **I-004/I2-A001** [AC] (DEEP-I-004.discovery.md:L75| unit=I-2): A/DET | “Last-seq wins on non-overlapping hunks” does not define what sequence precedence changes when hunks do not overlap; exact patch application/canonical merge semantics remain unspecified. | anchor=I-2:L15-L18
- **I-004/I2-A002** [AC] (DEEP-I-004.discovery.md:L76| unit=I-2): A/SEC | The escape requirement names absolute-path writes but does not define symlink, `..`, mount, junction, or path-canonicalization behavior. | anchor=I-2:L8-L9, L18-L19, L40-L45, L101-L103
- **I-004/I2-A003** [AC] (DEEP-I-004.discovery.md:L77| unit=I-2): A/DEP | Tier-1 acceptance requires rollback after `ConflictEscalated`, while H-49 restore semantics are still PLAN-asserted planned; the temporary behavior if H-49 has not landed is not specified. | anchor=I-2:L31-L32, L43-L45, L76-L76, L97-L103
- **I-004/I2-A004** [AC] (DEEP-I-004.discovery.md:L78| unit=I-2 SURFACE): A/SURFACE | `ConflictEscalated` is an event arm but is omitted from the explicit BranchTree badge arm list, leaving its exact operator-visible rendering unresolved. | anchor=I-2:L43-L45, L60-L61, L67-L68, L91-L94
- **I-004/I2-C001** [CON] (DEEP-I-004.discovery.md:L79| unit=I-2): C/DEFER | The opening design says Linux “gets” overlayfs/btrfs reflink, but the controlling revised scope moves that implementation to Tier 2; Tier 1 is the worktree default. | anchor=I-2:L12-L15, L38-L50
- **I-004/SD-I2-001** [SD] (DEEP-I-004.discovery.md:L544| unit=I-2): **CORRECT** summary implementation scope: overlayfs/btrfs is not a Tier-1 implementation; the worktree pool is the default and Linux CoW is deferred. | 09-I:L123-L134 | anchor=I2-E005–I2-E006, I2-C001
- **I-004/SD-I2-002** [SD] (DEEP-I-004.discovery.md:L545| unit=I-2): **CORRECT** “adjacent-code conflict resolves deterministically”: an overlapping conflict emits `ConflictEscalated`, halts, and rolls back; only non-overlap follows the stated sequence policy. | 09-I:L140-L141 | anchor=I2-E009–I2-E012, I2-A001
- **I-004/SD-I2-003** [SD] (DEEP-I-004.discovery.md:L546| unit=I-2 SURFACE): **ADD** complete event family, pre-filesystem escape guarantee, rollback chain, missing-row hard failure, and unresolved conflict-event UI rendering. | 09-I:L136-L141 | anchor=I2-E010, I2-E013–I2-E018, I2-A004
- **I-004/SD-I2-004** [SD] (DEEP-I-004.discovery.md:L547| unit=I-2): **ADD** exact `BranchWorkspace` methods, production reachability, test-only exclusion, no-reimplementation/stub bans, and PLAN-asserted composition status. | 09-I:L123-L143 | anchor=I2-E003–I2-E004, I2-E019–I2-E027
- **I-004/SD-I2-005** [SD] (DEEP-I-004.discovery.md:L548| unit=I-2): **ADD/QUALIFY** cloud adapter is user-approval gated, CRDT merge is deferred to I-29, and H-49 readiness/path-escape semantics remain unresolved. | 09-I:L127-L143 | anchor=I2-E007, I2-E022, I2-E028–I2-E029, I2-A002–I2-A003
- **I-004/SD-I2-006** [SD] (DEEP-I-004.discovery.md:L549| unit=I-2): **ADD** the entire J-11 Arbor lifecycle amendment: current-trunk start, artifact filtering, preserved branch, separate eval pool, stale cleanup, snapshot hook, and Windows-safe pooling improvement. | 09-I:L119-L143 | anchor=I2-E030–I2-E037, I2-I001

#### I-004 / unit `I-3` (38 records)

- **I-004/I3-E001** [SRC] (DEEP-I-004.discovery.md:L89| unit=I-3): E/STATUS | I-3 is a strong-grounded Track-12 performance EXTENSION, effort S, Wave 1. | anchor=I-3:L1-L3
- **I-004/I3-E002** [SRC] (DEEP-I-004.discovery.md:L90| unit=I-3): E/CONC | The SSB is the single append-only audit log for I-plan rows; search-era fanout motivates hardening it against write load and mid-write crashes. | anchor=I-3:L4-L9
- **I-004/I3-E003** [SRC] (DEEP-I-004.discovery.md:L91| unit=I-3): E/FLOW | Tier 1 hardens the existing **single-writer SQLite** append path; concurrent multi-writer fanout is not part of the immediate implementation. | anchor=I-3:L11-L13, L49-L53
- **I-004/I3-E004** [SRC] (DEEP-I-004.discovery.md:L92| unit=I-3): E/CFG | The Phase-3B pragma block must be audited/extended to include WAL mode, `busy_timeout`, `cache_size`, and `wal_autocheckpoint` exactly as the Phase-3B spec requires. | anchor=I-3:L11-L14, L38-L42, L55-L56, L66-L70
- **I-004/I3-E005** [SRC] (DEEP-I-004.discovery.md:L93| unit=I-3): E/RECOVERY | Every non-atomic checkpoint write becomes `<path>.tmp -> fsync -> atomic rename`; readers must observe either the old complete file or the new complete file, never a torn file. | anchor=I-3:L13-L16, L42-L45, L57-L59, L68-L70
- **I-004/I3-E006** [SRC] (DEEP-I-004.discovery.md:L94| unit=I-3): E/OBS | Actual insert throughput is measured from the real SSB write loop under seeded fixture load and stored in `metrics/phase_I3.toml`, rather than taken from a synthetic microbenchmark or paper. | anchor=I-3:L16-L20, L42-L44, L60-L60, L68-L70
- **I-004/I3-E007** [SRC] (DEEP-I-004.discovery.md:L95| unit=I-3): E/DET | The throughput measurement must be recomputable under `ELAI_SEED` and becomes the local ID-5 baseline for future claims. | anchor=I-3:L16-L20, L42-L44, L102-L105, L111-L116
- **I-004/I3-E008** [SRC] (DEEP-I-004.discovery.md:L96| unit=I-3): E/CFG | `io_uring` is a Linux-only Tier-2 experiment behind `ELAI_IO_URING_ENABLE`, with a `wire-dark-allowlist.toml` row/promotion plan and a separate enablement gate; it is never default. | anchor=I-3:L17-L20, L36-L36, L49-L51, L88-L88
- **I-004/I3-E009** [SRC] (DEEP-I-004.discovery.md:L97| unit=I-3): E/LIMIT | No external/paper throughput number may become a target; the measured TOML value is the baseline. | anchor=I-3:L8-L9, L19-L20, L86-L86, L111-L116
- **I-004/I3-E010** [SRC] (DEEP-I-004.discovery.md:L98| unit=I-3): E/LIMIT | Switching storage engines and asynchronous batched SSB writes are declined because SQLite+WAL and append-only golden locking are established invariants. | anchor=I-3:L21-L23
- **I-004/I3-E011** [SRC] (DEEP-I-004.discovery.md:L99| unit=I-3): E/STATUS | The cited dated live registry was `SCHEMA_VERSION=221` and `TOTAL_VARIANTS=532` at `ssb.rs:2981` and `action_type_lock.rs:601`. | anchor=I-3:L25-L27, L107-L108
- **I-004/I3-E012** [SRC] (DEEP-I-004.discovery.md:L100| unit=I-3): E/STATUS | Phase-3B pragmas and Phase-3C atomic writes are PLAN-asserted; some sites may already comply, so execution must re-grep/audit every actual checkpoint site. | anchor=I-3:L28-L31, L34-L35, L38-L43, L107-L110
- **I-004/I3-E013** [SRC] (DEEP-I-004.discovery.md:L101| unit=I-3): E/DEP | H-44 durability is PLAN-asserted planned and composes with this atomic-write pattern; I-3 must not absorb H-44’s checkpoint scope. | anchor=I-3:L32-L33, L77-L77, L118-L122
- **I-004/I3-E014** [SRC] (DEEP-I-004.discovery.md:L102| unit=I-3): E/STATUS | No `io_uring` integration existed in the cited tooling/crate inventory. | anchor=I-3:L36-L36
- **I-004/I3-E015** [SRC] (DEEP-I-004.discovery.md:L103| unit=I-3): E/TEST | `cargo check` must confirm that the pragma/write-loop edit introduces no regression. | anchor=I-3:L38-L42
- **I-004/I3-E016** [SRC] (DEEP-I-004.discovery.md:L104| unit=I-3): E/TEST | A kill-9 crash fixture must prove structural old-or-new completeness; the non-atomic broken arm must exhibit a torn file and fail. | anchor=I-3:L45-L45, L97-L105
- **I-004/I3-E017** [SRC] (DEEP-I-004.discovery.md:L105| unit=I-3): E/EVENT | I-3 adds no action type and must reuse existing SSB append infrastructure, leaving the dated 221/532 baseline unchanged. | anchor=I-3:L45-L47, L67-L71, L85-L90, L100-L101, L115-L116
- **I-004/I3-E018** [SRC] (DEEP-I-004.discovery.md:L106| unit=I-3): E/DEFER | Multi-writer fanout hardening waits until I-4 ships and real contention has been measured, with the decision deferred to full-form expansion. | anchor=I-3:L49-L53
- **I-004/I3-E019** [SRC] (DEEP-I-004.discovery.md:L107| unit=I-3): E/INT | The planned edit set is the pragma/write loop in `crates/orchestrator/src/ssb.rs`, all remaining checkpoint sites in `session_dag.rs` and `orchestrator/`, plus `workflow/reasoning/I-3-decisions.md`. | anchor=I-3:L55-L61
- **I-004/I3-E020** [SRC] (DEEP-I-004.discovery.md:L108| unit=I-3 SURFACE): E/SURFACE | I-3 intentionally adds no live UI: its observable artifacts are the metrics TOML and crash-fixture result, with future cockpit consumption assigned to I-27. | anchor=I-3:L62-L64, L93-L96
- **I-004/I3-E021** [SRC] (DEEP-I-004.discovery.md:L109| unit=I-3): E/INT | Production reachability is the SSB write loop used by real row-emitting runs and the `session_dag.rs` checkpoint call; test-only/contracts callers do not satisfy R7. | anchor=I-3:L80-L84
- **I-004/I3-E022** [SRC] (DEEP-I-004.discovery.md:L110| unit=I-3): E/LIMIT | Production edits prohibit `.unwrap()`/`.expect()`, blake2/sha2, fabricated locations, default-path `io_uring`, a new I-3 action type, and any weakening of append-only golden locking. | anchor=I-3:L85-L92
- **I-004/I3-E023** [SRC] (DEEP-I-004.discovery.md:L111| unit=I-3): E/TEST | The acceptance trigger runs a seeded real SSB fixture, kills the process mid-checkpoint, and parses `insert_throughput_per_sec` from `metrics/phase_I3.toml`, which must be greater than zero. | anchor=I-3:L97-L100
- **I-004/I3-E024** [SRC] (DEEP-I-004.discovery.md:L112| unit=I-3): E/OBS | The normal fixture must emit existing SSB rows without regression and must emit no I-3-specific variant. | anchor=I-3:L100-L101
- **I-004/I3-E025** [SRC] (DEEP-I-004.discovery.md:L113| unit=I-3): E/TEST | Crash completeness is checked using file length plus blake3 against the pre-crash snapshot; the negative arm without temp/fsync/rename must fail. | anchor=I-3:L102-L105
- **I-004/I3-E026** [SRC] (DEEP-I-004.discovery.md:L114| unit=I-3): E/DEP | I-3 Wave 1 precedes I-1 Wave 2 and I-4 Wave 4; durable Supervisor rows and high-fanout search appends consume its hardened path. | anchor=I-3:L123-L127
- **I-004/I3-I001** [INF] (DEEP-I-004.discovery.md:L115| unit=I-3 SURFACE): I/DATA | Because I-27 is named as the future cockpit consumer, `metrics/phase_I3.toml` implicitly needs a stable machine-readable schema even though I-3 specifies only one field. | anchor=I-3:L60-L64, L93-L96, L97-L105
- **I-004/I3-A001** [AC] (DEEP-I-004.discovery.md:L116| unit=I-3): A/CFG | The source names the required pragmas but supplies no concrete values for timeout, cache size, or autocheckpoint interval. | anchor=I-3:L11-L14, L38-L42
- **I-004/I3-A002** [AC] (DEEP-I-004.discovery.md:L117| unit=I-3): A/STATUS | Exact non-atomic checkpoint locations and which pragma clauses already ship are deliberately unresolved until execution-time audit. | anchor=I-3:L28-L35, L42-L43, L57-L59
- **I-004/I3-A003** [AC] (DEEP-I-004.discovery.md:L118| unit=I-3): A/TEST | Fixture workload, duration, hardware normalization, and any minimum acceptable throughput beyond `>0` are not defined. | anchor=I-3:L16-L20, L97-L105
- **I-004/I3-A004** [AC] (DEEP-I-004.discovery.md:L119| unit=I-3): A/RECOVERY | The text invokes POSIX rename but does not specify cross-platform replacement behavior, parent-directory fsync, temp-file cleanup, or retry after a failed rename. | anchor=I-3:L13-L16, L42-L45
- **I-004/I3-C001** [CON] (DEEP-I-004.discovery.md:L120| unit=I-3): C/CONC | The premise mentions concurrent writers, but the revised Tier-1 design remains single-writer and explicitly defers multi-writer hardening; this is durability/throughput hardening, not immediate multi-writer support. | anchor=I-3:L4-L9, L11-L13, L49-L53
- **I-004/SD-I3-001** [SD] (DEEP-I-004.discovery.md:L555| unit=I-3): **ADD** WAL mode (omitted from the summary pragma list), explicit single-writer scope, `cargo check`, and no-action-type registry invariant. | 09-I:L149-L164 | anchor=I3-E003–I3-E004, I3-E015, I3-E017
- **I-004/SD-I3-002** [SD] (DEEP-I-004.discovery.md:L556| unit=I-3 SURFACE): **ADD** exact `ELAI_IO_URING_ENABLE` flag, allowlist/promotion gate, and separately deferred multi-writer fanout after I-4 contention measurement. | 09-I:L155-L158 | anchor=I3-E008, I3-E018
- **I-004/SD-I3-003** [SD] (DEEP-I-004.discovery.md:L557| unit=I-3): **ADD** seeded real-loop methodology, TOML-as-ID-5-baseline semantics, `insert_throughput_per_sec > 0`, and file-length+blake3 crash proof. | 09-I:L153-L162 | anchor=I3-E006–I3-E007, I3-E009, I3-E023–I3-E025
- **I-004/SD-I3-004** [SD] (DEEP-I-004.discovery.md:L558| unit=I-3): **ADD** declined storage-engine/async batching alternatives, H-44/I-1/I-4 composition boundaries, production reachability, and write-path bans. | 09-I:L145-L164 | anchor=I3-E010–I3-E014, I3-E019–I3-E022, I3-E026
- **I-004/SD-I3-005** [SD] (DEEP-I-004.discovery.md:L559| unit=I-3): **QUALIFY** summary status ambiguity with the source’s full unknowns: pragma values/sites, benchmark normalization, and cross-platform atomic-write details remain undefined. | 09-I:L163-L164 | anchor=I3-A001–I3-A004
- **I-004/SD-I3-006** [SD] (DEEP-I-004.discovery.md:L560| unit=I-3): **CORRECT/CLARIFY** concurrency framing: Tier 1 does not deliver concurrent multi-writer support despite the motivating language. | 09-I:L145-L164 | anchor=I3-E003, I3-E018, I3-C001

#### I-004 / unit `I-12` (38 records)

- **I-004/I12-E001** [SRC] (DEEP-I-004.discovery.md:L130| unit=I-12): E/STATUS | I-12 is a strong-grounded Track-14 steering BUILD, effort M, Wave 3, governed by ID-6. | anchor=I-12:L1-L3
- **I-004/I12-E002** [SRC] (DEEP-I-004.discovery.md:L131| unit=I-12): E/FLOW | `PlaybookCurator` may promote a rule/playbook entry only when it links to at least `k` SSB-verified successful outcomes. | anchor=I-12:L4-L8, L10-L13, L34-L37, L52-L56
- **I-004/I12-E003** [SRC] (DEEP-I-004.discovery.md:L132| unit=I-12): E/OBS | Each candidate carries measured lift from a cheap on/off-injection job run on an offline tick; promoted content retains that measured lift. | anchor=I-12:L6-L8, L10-L14, L35-L40, L52-L55
- **I-004/I12-E004** [SRC] (DEEP-I-004.discovery.md:L133| unit=I-12): E/TEST | A merged rule must pass the C-28 mutation suite with zero regressions before any ACTIVE promotion. | anchor=I-12:L12-L14, L35-L37, L60-L60, L69-L76, L79-L82
- **I-004/I12-E005** [SRC] (DEEP-I-004.discovery.md:L134| unit=I-12): E/STATE | The curator feeds, and never replaces, the existing Phase-3F `PROPOSED -> SHADOW -> ACTIVE` lifecycle. | anchor=I-12:L15-L17, L26-L26, L38-L40, L47-L49, L58-L58, L91-L92
- **I-004/I12-E006** [SRC] (DEEP-I-004.discovery.md:L135| unit=I-12): E/DEP | `TemplateCrystallizer`/`CompiledPipelineStore` provide the compiled-AI zero-token storage precedent; C-25 contributes the earlier success-count pattern. | anchor=I-12:L15-L17, L24-L27, L47-L48, L59-L59, L84-L86, L93-L93
- **I-004/I12-E007** [SRC] (DEEP-I-004.discovery.md:L136| unit=I-12 SURFACE): E/SURFACE | `elai playbook export` writes verified entries in CLAUDE.md, AGENTS.md, and skills-compatible formats with embedded supporting evidence-row IDs. | anchor=I-12:L18-L20, L37-L39, L49-L49, L53-L56, L79-L82
- **I-004/I12-E008** [SRC] (DEEP-I-004.discovery.md:L137| unit=I-12): E/STATE | The same ledger automatically demotes stale or regressing ACTIVE entries without requiring a human edit. | anchor=I-12:L19-L22, L35-L37, L52-L55, L79-L82
- **I-004/I12-E009** [SRC] (DEEP-I-004.discovery.md:L138| unit=I-12): E/DEP | F-5 staleness demotion is the predecessor gate; I-12 extends it with evidence count rather than replacing it. | anchor=I-12:L21-L22, L30-L30, L62-L62, L91-L97
- **I-004/I12-E010** [SRC] (DEEP-I-004.discovery.md:L139| unit=I-12): E/SEC | ID-6 treats learned weights/rules as configuration: promotion must traverse trial-ledger gates and the evaluator must remain outside the loop’s write boundary under R8 gate freeze. | anchor=I-12:L31-L32, L61-L61, L68-L76, L84-L89
- **I-004/I12-E011** [SRC] (DEEP-I-004.discovery.md:L140| unit=I-12): E/EVENT | I-12 adds one `PlaybookEvent` action type with `Promoted`, `Demoted`, `ValidationRun`, and `Exported` kinds, baseline+1 from the dated 221/532 registry. | anchor=I-12:L35-L38, L52-L56, L79-L82, L84-L89
- **I-004/I12-E012** [SRC] (DEEP-I-004.discovery.md:L141| unit=I-12): E/CAP | Tier 1 creates `crates/orchestrator/src/rule_curator.rs::PlaybookCurator` with evidence-count promotion, demotion, lift measurement, and validation behavior. | anchor=I-12:L34-L37, L46-L48, L52-L55
- **I-004/I12-E013** [SRC] (DEEP-I-004.discovery.md:L142| unit=I-12 SURFACE): E/INT | The CLI subcommand is added under the PLAN-asserted `skills_cmd` family and must be re-grepped at execution. | anchor=I-12:L46-L50
- **I-004/I12-E014** [SRC] (DEEP-I-004.discovery.md:L143| unit=I-12): E/CFG | Promotion evaluation runs through a Layer-1A offline scheduler entry whose interval is configurable. | anchor=I-12:L35-L40, L64-L67
- **I-004/I12-E015** [SRC] (DEEP-I-004.discovery.md:L144| unit=I-12): E/DEP | The curator composes C-28 `MutationRunner`/`EquivalentMutantCache`, IS-16 `trial_ledger.rs`, E-8 GEPA’s metric shape, and the Phase-3F engine. | anchor=I-12:L28-L32, L47-L49, L57-L63, L84-L97
- **I-004/I12-E016** [SRC] (DEEP-I-004.discovery.md:L145| unit=I-12 SURFACE): E/SURFACE | Evidence count, lift, and demotion events are user-actionable rows in the existing `RulesDrawer` playbook panel. | anchor=I-12:L49-L50, L77-L78
- **I-004/I12-E017** [SRC] (DEEP-I-004.discovery.md:L146| unit=I-12 SURFACE): E/INT | Qualifying production paths are CLI `export_verified()` and scheduler `run_promotion_tick()`; test-only/contracts callers do not establish R7 reachability. | anchor=I-12:L64-L67
- **I-004/I12-E018** [SRC] (DEEP-I-004.discovery.md:L147| unit=I-12): E/FAIL | A zero-evidence attempt is refused with `ValidationRun{outcome_count:0}` and no ACTIVE transition. | anchor=I-12:L68-L70, L79-L82
- **I-004/I12-E019** [SRC] (DEEP-I-004.discovery.md:L148| unit=I-12): E/STATE | Successful promotion evidence includes `ValidationRun{regression_count:0}` before `PlaybookEvent{kind:Promoted,...}`. | anchor=I-12:L79-L82
- **I-004/I12-E020** [SRC] (DEEP-I-004.discovery.md:L149| unit=I-12): E/FAIL | A candidate with at least one C-28 regression emits a positive `regression_count` and is refused. | anchor=I-12:L69-L70, L79-L82
- **I-004/I12-E021** [SRC] (DEEP-I-004.discovery.md:L150| unit=I-12): E/LIMIT | I-12 must not fork/reimplement the Phase-3F promotion state machine. | anchor=I-12:L15-L17, L38-L40, L68-L72, L91-L92
- **I-004/I12-E022** [SRC] (DEEP-I-004.discovery.md:L151| unit=I-12): E/EVIDENCE | ACE +10.6% and DSPy/MIPROv2 +13% are DESIGN-tagged prior-art figures requiring web confirmation and local measurement; neither may become a threshold. | anchor=I-12:L10-L14, L68-L73, L84-L89
- **I-004/I12-E023** [SRC] (DEEP-I-004.discovery.md:L152| unit=I-12): E/LIMIT | Production promotion paths prohibit `.unwrap()`/`.expect()`, blake2/sha2, and self-evaluation inside the mutable loop boundary. | anchor=I-12:L68-L76
- **I-004/I12-E024** [SRC] (DEEP-I-004.discovery.md:L153| unit=I-12): E/DEFER | Cross-corpus transfer validation waits for an I-38 QDELTA baseline. | anchor=I-12:L42-L44
- **I-004/I12-E025** [SRC] (DEEP-I-004.discovery.md:L154| unit=I-12): E/DEFER | Automatic CLAUDE.md update is deferred behind user-approved PR-diff preview, `ELAI_PLAYBOOK_AUTOUPDATE_ENABLE`, a wire-dark allowlist row, and a promotion plan. | anchor=I-12:L42-L45
- **I-004/I12-E026** [SRC] (DEEP-I-004.discovery.md:L155| unit=I-12): E/DATA | The required reasoning artifact is `workflow/reasoning/I-12-decisions.md`. | anchor=I-12:L46-L50
- **I-004/I12-E027** [SRC] (DEEP-I-004.discovery.md:L156| unit=I-12): E/STATUS | IS-13 storage, IS-8 mutation, and IS-16 ledger seams are cited as existing/PLAN-asserted and require execution-time re-grep where noted. | anchor=I-12:L24-L32, L84-L89
- **I-004/I12-I001** [INF] (DEEP-I-004.discovery.md:L157| unit=I-12): I/STATE | Because export is restricted to verified/promoted entries and promotion is owned by Phase 3F, export is a read/serialization path rather than an alternate path to ACTIVE. | anchor=I-12:L15-L20, L37-L40, L53-L66
- **I-004/I12-A001** [AC] (DEEP-I-004.discovery.md:L158| unit=I-12): A/CFG | The required success threshold `k` has no value, derivation rule, or configuration-key name. | anchor=I-12:L6-L7, L12-L13, L35-L37
- **I-004/I12-A002** [AC] (DEEP-I-004.discovery.md:L159| unit=I-12): A/CFG | The offline scheduler interval is configurable but has no key name, unit, or default. | anchor=I-12:L35-L40
- **I-004/I12-A003** [AC] (DEEP-I-004.discovery.md:L160| unit=I-12): A/OBS | The on/off lift estimator does not define its corpus, pairing/randomization, minimum sample, confidence interval, or promotion threshold. | anchor=I-12:L10-L14, L35-L37
- **I-004/I12-A004** [AC] (DEEP-I-004.discovery.md:L161| unit=I-12): A/DATA | The exact serialized export block schemas, merge/collision rules, and behavior when target files already contain generated blocks are not specified. | anchor=I-12:L18-L20, L37-L39, L53-L56, L79-L82
- **I-004/SD-I12-001** [SD] (DEEP-I-004.discovery.md:L566| unit=I-12): **QUALIFY** summary “interleaved on/off injection”: the numbered source says cheap on/off injection on an offline tick but does not explicitly specify interleaving. | 09-I:L369-L371 | anchor=I12-E003, I12-A003
- **I-004/SD-I12-002** [SD] (DEEP-I-004.discovery.md:L567| unit=I-12): **ADD** full `PlaybookEvent` kinds, required validation-before-promotion order, automatic stale/regression demotion, and exporter-as-read lifecycle boundary. | 09-I:L366-L384 | anchor=I12-E005, I12-E008, I12-E011, I12-E018–I12-E020, I12-I001
- **I-004/SD-I12-003** [SD] (DEEP-I-004.discovery.md:L568| unit=I-12 SURFACE): **ADD** exact module/production entrypoints, configurable Layer-1A schedule, event-driven RulesDrawer fields, and decision artifact. | 09-I:L376-L383 | anchor=I12-E012–I12-E017, I12-E026
- **I-004/SD-I12-004** [SD] (DEEP-I-004.discovery.md:L569| unit=I-12): **ADD** ID-6/R8 evaluator-write-boundary rule, C-28/IS-16/E-8/F-5 composition seams, and DESIGN-number ban. | 09-I:L366-L386 | anchor=I12-E004, I12-E009–I12-E010, I12-E015, I12-E021–I12-E023, I12-E027
- **I-004/SD-I12-005** [SD] (DEEP-I-004.discovery.md:L570| unit=I-12): **ADD** cross-corpus/I-38 deferral and user-approved `ELAI_PLAYBOOK_AUTOUPDATE_ENABLE` PR-preview lane. | 09-I:L362-L386 | anchor=I12-E024–I12-E025
- **I-004/SD-I12-006** [SD] (DEEP-I-004.discovery.md:L571| unit=I-12): **EXPAND** the summary’s `k` ambiguity to include schedule, lift statistics, and export block/collision schema. | 09-I:L385-L386 | anchor=I12-A001–I12-A004

#### I-004 / unit `I-21` (45 records)

- **I-004/I21-E001** [SRC] (DEEP-I-004.discovery.md:L171| unit=I-21): E/STATUS | I-21 is a strong-grounded Track-15 trust/anti-bias BUILD, effort M, Wave 3, with IC-17 persona consistency; full-form expansion is mandatory before EXECUTING. | anchor=I-21:L1-L3, L111-L111
- **I-004/I21-E002** [SRC] (DEEP-I-004.discovery.md:L172| unit=I-21): E/EVIDENCE | The motivating +0.12 self-review-bias figure is carried from research/drafts and must be web-confirmed before use as a headline gate number. | anchor=I-21:L4-L9, L96-L101
- **I-004/I21-E003** [SRC] (DEEP-I-004.discovery.md:L173| unit=I-21): E/DATA | The initial Sprint Contract shape is `{goal, in_scope_files, acceptance_criteria, bounded_iteration_count, out_of_scope_marker}`. | anchor=I-21:L11-L16
- **I-004/I21-E004** [SRC] (DEEP-I-004.discovery.md:L174| unit=I-21): E/DATA | A Sprint Contract is machine-readable structured data, not a free-text plan. | anchor=I-21:L11-L16
- **I-004/I21-E005** [SRC] (DEEP-I-004.discovery.md:L175| unit=I-21): E/TEST | Each acceptance criterion must deterministically reference a test symbol, compile status, or wiring-contract field. | anchor=I-21:L13-L16, L49-L52, L83-L88
- **I-004/I21-E006** [SRC] (DEEP-I-004.discovery.md:L176| unit=I-21): E/FAIL | Vague/open-ended criteria such as “implementation feels complete” are rejected before execution/at approval. | anchor=I-21:L13-L16, L49-L52, L83-L86, L91-L94
- **I-004/I21-E007** [SRC] (DEEP-I-004.discovery.md:L177| unit=I-21): E/SEC | The isolated evaluator receives only the sealed Sprint Contract and task spec—no generator turns, intermediate outputs, or scratchpad. | anchor=I-21:L17-L21, L35-L36, L83-L85
- **I-004/I21-E008** [SRC] (DEEP-I-004.discovery.md:L178| unit=I-21): E/FLOW | I-1 Supervisor enforces isolation by spawning an evaluator Directive whose payload is sealed and whose generator context is pruned. | anchor=I-21:L17-L20, L44-L45, L52-L52, L74-L79
- **I-004/I21-E009** [SRC] (DEEP-I-004.discovery.md:L179| unit=I-21): E/CFG | The evaluator runs the D-13 adversarial posture at temperature zero with a retained hostile-critic persona because the work is alignment-dependent. | anchor=I-21:L20-L21, L30-L34, L96-L101
- **I-004/I21-E010** [SRC] (DEEP-I-004.discovery.md:L180| unit=I-21): E/EVENT | Receipt of a contract emits `SprintContractEvent(Proposed)`. | anchor=I-21:L22-L24, L49-L56
- **I-004/I21-E011** [SRC] (DEEP-I-004.discovery.md:L181| unit=I-21): E/STATE | Approval emits `Approved`; rejection emits `Rejected` with structured, concrete missing/ambiguous/scope-creep objections rather than a generic response. | anchor=I-21:L22-L24, L49-L56, L65-L67
- **I-004/I21-E012** [SRC] (DEEP-I-004.discovery.md:L182| unit=I-21): E/FLOW | At generated-output exit, I-17 `TriLayerStack` checks the criteria; the reinjected trace must identify the exact failing gate and line. | anchor=I-21:L25-L29, L45-L45, L53-L55, L77-L79
- **I-004/I21-E013** [SRC] (DEEP-I-004.discovery.md:L183| unit=I-21): E/RETRY | A failed check reinjects its trace for one retry and emits `RalphReinjected`. | anchor=I-21:L25-L29, L53-L56, L91-L94
- **I-004/I21-E014** [SRC] (DEEP-I-004.discovery.md:L184| unit=I-21): E/RETRY | The Ralph loop is capped at **three total iterations**, meaning at most two reinjections. | anchor=I-21:L25-L29, L35-L36, L53-L55, L83-L86
- **I-004/I21-E015** [SRC] (DEEP-I-004.discovery.md:L185| unit=I-21): E/RECOVERY | The third failure escalates to the existing D-13 cross-family adversary, emits `AdversaryEscalated`, and requires a concrete counterexample rather than only a score. | anchor=I-21:L27-L29, L41-L43, L53-L56, L74-L79, L91-L94
- **I-004/I21-E016** [SRC] (DEEP-I-004.discovery.md:L186| unit=I-21): E/FAIL | The loop counter is SSB-tracked; overflow is a BLOCK, and H-38 provides stop-gate enforcement at the cap. | anchor=I-21:L27-L29, L41-L42, L53-L55, L74-L79, L83-L88
- **I-004/I21-E017** [SRC] (DEEP-I-004.discovery.md:L187| unit=I-21): E/LIMIT | Shared generator/evaluator context, an unbounded loop, and non-deterministically checkable criteria are explicitly excluded. | anchor=I-21:L35-L36
- **I-004/I21-E018** [SRC] (DEEP-I-004.discovery.md:L188| unit=I-21): E/STATUS | No sprint-contract module existed in the cited inventory; I-21 is a genuine build. | anchor=I-21:L4-L6, L38-L40, L96-L98
- **I-004/I21-E019** [SRC] (DEEP-I-004.discovery.md:L189| unit=I-21): E/DEP | H-37 gate, H-38 stop gate, D-13 adversary, and C-32 MiniCritic posture are PLAN-asserted shipped composition seams, not code to duplicate. | anchor=I-21:L38-L46, L74-L79, L105-L109
- **I-004/I21-E020** [SRC] (DEEP-I-004.discovery.md:L190| unit=I-21): E/RECOVERY | If I-1 has not landed, Tier 1 permits a temporary evaluator-isolation shim that is later replaced. | anchor=I-21:L44-L44, L52-L52
- **I-004/I21-E021** [SRC] (DEEP-I-004.discovery.md:L191| unit=I-21): E/DEP | I-17 `TriLayerStack` is the sole acceptance-check trigger; I-21 must not add an ad-hoc duplicate checker. | anchor=I-21:L45-L45, L53-L55, L77-L79, L105-L109
- **I-004/I21-E022** [SRC] (DEEP-I-004.discovery.md:L192| unit=I-21): E/CAP | Tier 1 creates `crates/orchestrator/src/sprint_contract.rs::SprintContract` and `::RalphLoop`, including validation, Directive construction, loop counting, and escalation. | anchor=I-21:L48-L56, L61-L73
- **I-004/I21-E023** [SRC] (DEEP-I-004.discovery.md:L193| unit=I-21): E/EVENT | I-21 adds one `SprintContractEvent` action type with `Proposed`, `Approved`, `Rejected`, `RalphReinjected`, and `AdversaryEscalated`, baseline+1 from the dated 221/532 registry. | anchor=I-21:L49-L56, L65-L73, L103-L103
- **I-004/I21-E024** [SRC] (DEEP-I-004.discovery.md:L194| unit=I-21): E/DEFER | Multi-contract sprints are deferred; Tier 1 only leaves a contract-chaining extension seam. | anchor=I-21:L57-L59
- **I-004/I21-E025** [SRC] (DEEP-I-004.discovery.md:L195| unit=I-21): E/DEFER | Automatic generation of acceptance criteria is deferred; manually supplied deterministic criteria ship first. | anchor=I-21:L57-L59
- **I-004/I21-E026** [SRC] (DEEP-I-004.discovery.md:L196| unit=I-21): E/INT | I-21 extends H-37 `run_interactive_gate` with a new payload and modifies `search/ladder.rs` or `task_runner.rs` at an execution-time-re-grepped outer retry seam. | anchor=I-21:L61-L65
- **I-004/I21-E027** [SRC] (DEEP-I-004.discovery.md:L197| unit=I-21 SURFACE): E/SURFACE | A Sprint Contract card beside `PlanApprovalCard` shows contract fields, criterion-determinism badges, verdict/objection, and loop count via existing SSB subscription; `cross_lang_diff.py` must pass. | anchor=I-21:L65-L67, L89-L90
- **I-004/I21-E028** [SRC] (DEEP-I-004.discovery.md:L198| unit=I-21): E/INT | Production reachability is task-runner/search-ladder sprint dispatch; contracts-only callers never satisfy R7. | anchor=I-21:L80-L82
- **I-004/I21-E029** [SRC] (DEEP-I-004.discovery.md:L199| unit=I-21): E/LIMIT | Production code forbids generator context in the evaluator, counterless loops, free-text criteria, duplicated D-13 logic, `.unwrap()`/`.expect()`, and fabricated verdict/count data. | anchor=I-21:L83-L88
- **I-004/I21-E030** [SRC] (DEEP-I-004.discovery.md:L200| unit=I-21 SURFACE): E/TEST | The acceptance trigger runs `elai run --sprint --task-fixture fixtures/vague_contract.toml --emit-ssb`. | anchor=I-21:L91-L92
- **I-004/I21-E031** [SRC] (DEEP-I-004.discovery.md:L201| unit=I-21): E/OBS | The fixture must show `Rejected` with a concrete ambiguity objection, `RalphReinjected` after the first failed exit, and `AdversaryEscalated` on the third failure. | anchor=I-21:L92-L94
- **I-004/I21-E032** [SRC] (DEEP-I-004.discovery.md:L202| unit=I-21): E/TEST | A journal join by session/role must prove the evaluator saw only the sealed payload and zero generator rows; a `--shared-context` arm must change verdict distribution and fail isolation. | anchor=I-21:L92-L94
- **I-004/I21-E033** [SRC] (DEEP-I-004.discovery.md:L203| unit=I-21): E/DET | Approval and execution rows must byte-recompute under `ELAI_SEED`. | anchor=I-21:L93-L94
- **I-004/I21-A001** [AC] (DEEP-I-004.discovery.md:L204| unit=I-21): A/DATA | The proposal uses field name `bounded_iteration_count`, while revised Tier 1 uses `iteration_count`; the canonical serialized field name is unresolved. | anchor=I-21:L12-L13, L49-L51
- **I-004/I21-A002** [AC] (DEEP-I-004.discovery.md:L205| unit=I-21): A/TEST | “Reference to” an allowed symbol/status/field has no specified grammar, resolver, repository snapshot, or behavior for renamed/missing symbols. | anchor=I-21:L13-L16, L49-L52, L83-L86
- **I-004/I21-A003** [AC] (DEEP-I-004.discovery.md:L206| unit=I-21): A/DEP | I-1 is called the isolation substrate/hard sequencing need, yet a temporary shim is allowed; the shim’s interface, evidence, and removal gate are unspecified. | anchor=I-21:L17-L20, L44-L44, L52-L52, L77-L78
- **I-004/I21-A004** [AC] (DEEP-I-004.discovery.md:L207| unit=I-21): A/INT | The existing outer retry loop may be “replaced or supplemented”; the plan does not choose one or define nested-budget interaction. | anchor=I-21:L63-L64
- **I-004/I21-A005** [AC] (DEEP-I-004.discovery.md:L208| unit=I-21): A/STATE | The exact event/verdict sequence on the third failure—whether a final `Rejected`, stop event, or other row accompanies `AdversaryEscalated`—is not defined. | anchor=I-21:L22-L29, L53-L56, L91-L94
- **I-004/SD-I21-001** [SD] (DEEP-I-004.discovery.md:L577| unit=I-21): **CORRECT** summary “vague goal is rejected”: the explicit rejection target is a vague/open-ended **acceptance criterion**, with permitted reference categories. | 09-I:L605-L624 | anchor=I21-E003–I21-E006
- **I-004/SD-I21-002** [SD] (DEEP-I-004.discovery.md:L578| unit=I-21): **ADD** exact contract fields including `out_of_scope_marker`, while flagging `bounded_iteration_count` versus `iteration_count` schema drift. | 09-I:L606-L610 | anchor=I21-E003–I21-E005, I21-A001
- **I-004/SD-I21-003** [SD] (DEEP-I-004.discovery.md:L579| unit=I-21): **QUALIFY** zero-shared-context wording: evaluator still receives task spec plus sealed contract, but no generator history/output/scratchpad; Supervisor pruning is the enforcement. | 09-I:L605-L619 | anchor=I21-E007–I21-E009
- **I-004/SD-I21-004** [SD] (DEEP-I-004.discovery.md:L580| unit=I-21): **ADD** `Proposed`/`Approved`/`Rejected` event arms, structured concrete objection fields, SSB loop counter, overflow BLOCK, and H-38 stop behavior. | 09-I:L611-L624 | anchor=I21-E010–I21-E011, I21-E016, I21-E023, I21-E027, I21-A005
- **I-004/SD-I21-005** [SD] (DEEP-I-004.discovery.md:L581| unit=I-21): **ADD** exact retry semantics: one trace per failed exit, gate+line payload, three total iterations/two reinjections, then D-13 counterexample. | 09-I:L612-L624 | anchor=I21-E012–I21-E016, I21-E031–I21-E033
- **I-004/SD-I21-006** [SD] (DEEP-I-004.discovery.md:L582| unit=I-21): **ADD** temporary I-1 isolation shim, unresolved replace/supplement outer retry, production reachability, and negative shared-context proof. | 09-I:L618-L624 | anchor=I21-E020–I21-E022, I21-E026, I21-E028–I21-E032, I21-A003–I21-A004
- **I-004/SD-I21-007** [SD] (DEEP-I-004.discovery.md:L583| unit=I-21): **ADD** multi-contract and auto-criteria deferrals, full-form requirement, external-figure caution, and unresolved criterion resolver. | 09-I:L601-L624 | anchor=I21-E001–I21-E002, I21-E024–I21-E025, I21-A002

#### I-004 / unit `I-27` (34 records)

- **I-004/I27-E001** [SRC] (DEEP-I-004.discovery.md:L218| unit=I-27): E/STATUS | I-27 is a strong-grounded Track-16 provability BUILD, effort S-M, Wave 4. | anchor=I-27:L1-L3
- **I-004/I27-E002** [SRC] (DEEP-I-004.discovery.md:L219| unit=I-27): E/CAP | It builds a public curve of W-1 verified-outcome rate versus total test-time compute for each configuration. | anchor=I-27:L4-L7, L9-L17
- **I-004/I27-E003** [SRC] (DEEP-I-004.discovery.md:L220| unit=I-27): E/DATA | `bench_convergence.rs` sweeps a fixture suite through increasing compute envelopes expressed as nodes, tokens, and monetary spend. | anchor=I-27:L9-L12, L30-L36
- **I-004/I27-E004** [SRC] (DEEP-I-004.discovery.md:L221| unit=I-27): E/DATA | Only journaled W-1 verified outcomes count; pass@k is explicitly excluded. | anchor=I-27:L10-L13, L17-L20, L64-L68
- **I-004/I27-E005** [SRC] (DEEP-I-004.discovery.md:L222| unit=I-27): E/OBS | Each curve point is a mean with bootstrap confidence intervals over at least five paired seeds, inheriting I-38’s statistical floor. | anchor=I-27:L11-L13, L30-L36, L79-L84
- **I-004/I27-E006** [SRC] (DEEP-I-004.discovery.md:L223| unit=I-27 SURFACE): E/SURFACE | A Pareto view overlays cost per additional verified task as a second axis so an operator can choose a cost/accuracy operating point. | anchor=I-27:L4-L7, L13-L15, L43-L47, L71-L72
- **I-004/I27-E007** [SRC] (DEEP-I-004.discovery.md:L224| unit=I-27): E/OBS | The report identifies and names the knee where marginal gain collapses. | anchor=I-27:L13-L15, L74-L77
- **I-004/I27-E008** [SRC] (DEEP-I-004.discovery.md:L225| unit=I-27): E/EVIDENCE | Any adjacent-level dip must be explicitly flagged as non-monotonic and never smoothed or generalized beyond the measured fixture sample. | anchor=I-27:L15-L20, L64-L68, L79-L82
- **I-004/I27-E009** [SRC] (DEEP-I-004.discovery.md:L226| unit=I-27): E/EVENT | I-27 adds one flat `ConvergenceCurveMeasured` beacon/action type, baseline+1 from the dated 221/532 registry. | anchor=I-27:L15-L17, L74-L84
- **I-004/I27-E010** [SRC] (DEEP-I-004.discovery.md:L227| unit=I-27): E/LIMIT | SWE-bench branding and external/paper compute-efficiency targets are prohibited; ELAI’s fixture suite is the corpus. | anchor=I-27:L18-L20, L64-L69, L79-L84
- **I-004/I27-E011** [SRC] (DEEP-I-004.discovery.md:L228| unit=I-27 SURFACE): E/DEP | I-27 imports E-49 `$ / CTS` and `STABLE-Pass`, mirrors E-9’s fixture-runner shape, lands in H-9’s cockpit family, reads I-4 `SearchNodeEvent`, and shares I-38’s statistical method. | anchor=I-27:L22-L28, L43-L58, L79-L91
- **I-004/I27-E012** [SRC] (DEEP-I-004.discovery.md:L229| unit=I-27): E/SEC | Under R8, the measured session may not modify the benchmark harness; the harness is measurement infrastructure, not a mutable gate. | anchor=I-27:L27-L27
- **I-004/I27-E013** [SRC] (DEEP-I-004.discovery.md:L230| unit=I-27 SURFACE): E/CAP | Tier 1 creates `crates/cli/src/bench_convergence.rs::ConvergenceBench` for sweeping, aggregation, curve computation, and row emission. | anchor=I-27:L30-L36, L43-L52
- **I-004/I27-E014** [SRC] (DEEP-I-004.discovery.md:L231| unit=I-27 SURFACE): E/SURFACE | Tier 1 creates/extends a dashboard-v2 convergence tile in the H-9 cost cockpit, rendering both curve and Pareto view through a Tauri-to-Svelte pair checked by `cross_lang_diff.py`. | anchor=I-27:L30-L35, L43-L47, L49-L53, L71-L72
- **I-004/I27-E015** [SRC] (DEEP-I-004.discovery.md:L232| unit=I-27): E/TEST | Tier 1 includes `crates/contracts/tests/i27_convergence_instrumentation_contract.rs` and `workflow/reasoning/I-27-decisions.md`. | anchor=I-27:L35-L37
- **I-004/I27-E016** [SRC] (DEEP-I-004.discovery.md:L233| unit=I-27): E/DEFER | Live/non-batch refresh waits for sustained I-4 trajectory volume. | anchor=I-27:L39-L41
- **I-004/I27-E017** [SRC] (DEEP-I-004.discovery.md:L234| unit=I-27): E/DEFER | Cross-config overlays require at least three sufficiently sampled configurations and are RunPod-gated for GPU configurations. | anchor=I-27:L39-L42
- **I-004/I27-E018** [SRC] (DEEP-I-004.discovery.md:L235| unit=I-27 SURFACE): E/SURFACE | Production CLI entry is `elai bench convergence --suite <fixture>`; the beacon is the off-agent artifact read by the cockpit. | anchor=I-27:L60-L63
- **I-004/I27-E019** [SRC] (DEEP-I-004.discovery.md:L236| unit=I-27): E/DATA | The required report is `workflow/benchmarks/convergence_<fixture>_<date>.toml` with curve points, confidence intervals, Pareto data, knee label, and non-monotonic flags. | anchor=I-27:L74-L77
- **I-004/I27-E020** [SRC] (DEEP-I-004.discovery.md:L237| unit=I-27): E/DET | `curve_points` must recompute byte-identically from the journal under `ELAI_SEED`; the row also carries CI bounds and `knee_compute_level`. | anchor=I-27:L74-L77
- **I-004/I27-E021** [SRC] (DEEP-I-004.discovery.md:L238| unit=I-27 SURFACE): E/TEST | The acceptance run uses at least five seeds: `elai bench convergence --suite <fixture-set> --seeds 5`. | anchor=I-27:L74-L76
- **I-004/I27-E022** [SRC] (DEEP-I-004.discovery.md:L239| unit=I-27): E/FAIL | A `--skip-verification` broken arm must inflate `verified_count`; the contract detects that inflation and fails. | anchor=I-27:L74-L77
- **I-004/I27-E023** [SRC] (DEEP-I-004.discovery.md:L240| unit=I-27): E/LIMIT | Production code forbids pass@k, fabricated monotonicity, paper-derived targets, SWE-bench headlines, `.unwrap()`/`.expect()`, and blake2/sha2. | anchor=I-27:L64-L69
- **I-004/I27-E024** [SRC] (DEEP-I-004.discovery.md:L241| unit=I-27 SURFACE): E/INT | E-49 definitions, E-9 runner skeleton, H-9 cockpit, I-4 telemetry, and I-38 bootstrap must be imported/composed rather than redefined. | anchor=I-27:L54-L58, L86-L91
- **I-004/I27-I001** [INF] (DEEP-I-004.discovery.md:L242| unit=I-27): I/DATA | `ConvergenceCurveMeasured` is implicitly an aggregate per suite/configuration rather than a per-node event because its payload must contain the complete recomputable curve and knee. | anchor=I-27:L9-L17, L74-L77
- **I-004/I27-A001** [AC] (DEEP-I-004.discovery.md:L243| unit=I-27): A/CFG | The fixture suite, exact compute-level sequence, repetitions per level, token/cost accounting boundaries, and configuration identity are not defined. | anchor=I-27:L9-L17, L30-L36
- **I-004/I27-A002** [AC] (DEEP-I-004.discovery.md:L244| unit=I-27): A/OBS | The bootstrap procedure, confidence level, knee-detection algorithm, and formal non-monotonic significance rule are unspecified. | anchor=I-27:L11-L17, L30-L36, L74-L84
- **I-004/I27-A003** [AC] (DEEP-I-004.discovery.md:L245| unit=I-27): A/DATA | Monetary denomination/provider normalization and the exact meaning of “cost per additional verified task” are left to imported E-49 definitions or later implementation. | anchor=I-27:L11-L15, L22-L23, L54-L56
- **I-004/I27-A004** [AC] (DEEP-I-004.discovery.md:L246| unit=I-27): A/DET | The row is seed-recomputable, but the report filename embeds `<date>` and no normalization rule is given for byte-stable artifact naming. | anchor=I-27:L74-L77
- **I-004/SD-I27-001** [SD] (DEEP-I-004.discovery.md:L589| unit=I-27): **ADD** mean curve points with bootstrap confidence intervals and at least five paired seeds under I-38’s statistical floor. | 09-I:L738-L751 | anchor=I27-E003–I27-E005, I27-E021
- **I-004/SD-I27-002** [SD] (DEEP-I-004.discovery.md:L590| unit=I-27): **ADD** exact beacon/report payload, artifact path, knee field, seed byte-recomputation, non-monotonic flags, and immutable-harness R8 rule. | 09-I:L745-L751 | anchor=I27-E007–I27-E009, I27-E012, I27-E019–I27-E020
- **I-004/SD-I27-003** [SD] (DEEP-I-004.discovery.md:L591| unit=I-27): **ADD** E-49/E-9/H-9/I-4/I-38 non-reimplementation dependencies, decision/contract artifacts, and production/off-agent reachability. | 09-I:L734-L751 | anchor=I27-E011, I27-E013–I27-E015, I27-E018, I27-E024
- **I-004/SD-I27-004** [SD] (DEEP-I-004.discovery.md:L592| unit=I-27): **ADD** live-refresh and >=3-config/RunPod overlay deferrals plus explicit paper/SWE-bench constraints. | 09-I:L734-L751 | anchor=I27-E010, I27-E016–I27-E017, I27-E023
- **I-004/SD-I27-005** [SD] (DEEP-I-004.discovery.md:L593| unit=I-27 SURFACE): **FLAG** compute-step, statistic, knee, cost, and date-normalization choices absent from the summary and source. | 09-I:L738-L751 | anchor=I27-A001–I27-A004

#### I-004 / unit `I-34` (52 records)

- **I-004/I34-E001** [SRC] (DEEP-I-004.discovery.md:L256| unit=I-34): E/STATUS | I-34 is a strong-grounded Track-18 trust/can’t-lie BUILD, effort M, Wave 1, bound to IC-18. | anchor=I-34:L1-L3
- **I-004/I34-E002** [SRC] (DEEP-I-004.discovery.md:L257| unit=I-34): E/STATUS | `providers/src/decoding.rs::DecodingMode::Grammar{ebnf_path, crane_adaptive}` exists at the cited lines, but `crane_adaptive` has no runtime leg. | anchor=I-34:L5-L9, L33-L40, L94-L96
- **I-004/I34-E003** [SRC] (DEEP-I-004.discovery.md:L258| unit=I-34): E/EVIDENCE | The premise carries three externally sourced risks—10–15pp constrained-reasoning loss, 38 silent JSONSchemaBench failures, and 96.2% EnumAttack ASR—which require web confirmation before headline use. | anchor=I-34:L5-L9, L94-L102
- **I-004/I34-E004** [SRC] (DEEP-I-004.discovery.md:L259| unit=I-34): E/FLOW | On the local sidecar lane, CRANE permits free thinking tokens between structural commitment points and constrains the model when it commits to structured tokens. | anchor=I-34:L11-L18, L43-L49, L65-L68
- **I-004/I34-E005** [SRC] (DEEP-I-004.discovery.md:L260| unit=I-34): E/SEC | Reflection/self-correction constrains only FINAL output; intermediate reasoning remains unconstrained to avoid structure snowballing. | anchor=I-34:L14-L18, L82-L86
- **I-004/I34-E006** [SRC] (DEEP-I-004.discovery.md:L261| unit=I-34): E/DEP | C-37 re-evaluation is a consumer of the CRANE runtime; I-34 does not absorb C-37. | anchor=I-34:L17-L18, L37-L37, L61-L62, L104-L109
- **I-004/I34-E007** [SRC] (DEEP-I-004.discovery.md:L262| unit=I-34): E/DATA | Every grammar is a blake3-pinned control-plane artifact recorded in a manifest modeled on E-24 MCP-schema pinning. | anchor=I-34:L20-L23, L43-L47, L57-L59, L65-L75
- **I-004/I34-E008** [SRC] (DEEP-I-004.discovery.md:L263| unit=I-34): E/SEC | Third-party grammars are never loaded, and enum-bearing schemas from untrusted sources are rejected at load. | anchor=I-34:L20-L23, L30-L31, L82-L84
- **I-004/I34-E009** [SRC] (DEEP-I-004.discovery.md:L264| unit=I-34): E/SEC | The grammar manifest is R8-frozen; tampering emits `GrammarGateEvent(PinViolation)`. | anchor=I-34:L20-L23, L43-L49, L77-L80
- **I-004/I34-E010** [SRC] (DEEP-I-004.discovery.md:L265| unit=I-34): E/TEST | Every constrained generation is raw-reparsed by `re_validate_output()` against its schema after decoding. | anchor=I-34:L25-L28, L43-L49, L57-L58, L65-L70
- **I-004/I34-E011** [SRC] (DEEP-I-004.discovery.md:L266| unit=I-34): E/FAIL | Silent noncompliance emits `PostParseRejected` and routes to the existing D-16 refusal path; skipping validation is the `--trust-mask` broken arm. | anchor=I-34:L25-L31, L47-L49, L60-L61, L76-L80, L89-L92
- **I-004/I34-E012** [SRC] (DEEP-I-004.discovery.md:L267| unit=I-34): E/LIMIT | Loading a third-party grammar or trusting mask output without post-parse revalidation is explicitly excluded. | anchor=I-34:L30-L31
- **I-004/I34-E013** [SRC] (DEEP-I-004.discovery.md:L268| unit=I-34): E/STATUS | E-35/E-18 XGrammar, C-26 `DecodingMode`, C-37, E-24 pins, and D-16 refusal are PLAN-asserted composition seams whose exact locations require execution-time re-grep. | anchor=I-34:L33-L41, L71-L80, L94-L109
- **I-004/I34-E014** [SRC] (DEEP-I-004.discovery.md:L269| unit=I-34): E/CAP | Tier 1 implements the Rust/provider CRANE runtime, post-parse validator, grammar store/manifest/load verification, and `workflow/reasoning/I-34-decisions.md`. | anchor=I-34:L43-L51, L57-L70
- **I-004/I34-E015** [SRC] (DEEP-I-004.discovery.md:L270| unit=I-34): E/EVENT | I-34 adds one `GrammarGateEvent` action type with `CranePassed`, `PinViolation`, and `PostParseRejected`, baseline+1 from the dated 221/532 registry. | anchor=I-34:L43-L49, L65-L70, L89-L102
- **I-004/I34-E016** [SRC] (DEEP-I-004.discovery.md:L271| unit=I-34): E/TEST | Tier-1 fixtures cover a mutated grammar, seeded silent noncompliance, and reasoning-heavy CRANE whose measured pass rate must be at least its FreeText arm under local ID-5 measurement. | anchor=I-34:L47-L50, L89-L92
- **I-004/I34-E017** [SRC] (DEEP-I-004.discovery.md:L272| unit=I-34 SURFACE): E/SURFACE | The existing decode panel gains a grammar-pin status chip through Svelte and Tauri; `cross_lang_diff.py` must pass. | anchor=I-34:L49-L50, L63-L63, L87-L92
- **I-004/I34-E018** [SRC] (DEEP-I-004.discovery.md:L273| unit=I-34): E/DEFER | Python-sidecar mask-layer CRANE integration is Tier 2 and requires a separately gated, hardware-heavy sidecar boot. | anchor=I-34:L53-L55, L57-L60
- **I-004/I34-E019** [SRC] (DEEP-I-004.discovery.md:L274| unit=I-34): E/DEFER | A cross-language grammar-compliance verifier pack is Tier 2 under I-17. | anchor=I-34:L53-L55
- **I-004/I34-E020** [SRC] (DEEP-I-004.discovery.md:L275| unit=I-34): E/INT | Production reachability covers `DecodingMode::Grammar -> crane_adaptive`, grammar-manifest load verification, and post-parse validation into refusal; tests-only callers do not satisfy R7. | anchor=I-34:L77-L81
- **I-004/I34-E021** [SRC] (DEEP-I-004.discovery.md:L276| unit=I-34): E/DATA | The new grammar-store module path is deliberately TBD until execution-time re-grep. | anchor=I-34:L65-L70
- **I-004/I34-E022** [SRC] (DEEP-I-004.discovery.md:L277| unit=I-34): E/LIMIT | Production paths forbid third-party grammars, skipped validation, intermediate-reasoning constraints, `.unwrap()`/`.expect()`, and blake2/sha2. | anchor=I-34:L82-L86
- **I-004/I34-E023** [SRC] (DEEP-I-004.discovery.md:L278| unit=I-34): E/OBS | The seeded fixture must emit `CranePassed`, `PinViolation`, and `PostParseRejected`; the status chip and broken-arm failure are required off-agent evidence. | anchor=I-34:L89-L92
- **I-004/I34-E024** [SRC] (DEEP-I-004.discovery.md:L279| unit=I-34): E/EVIDENCE | CRANE, structure-snowballing, XGrammar, and JSONSchemaBench identifiers are carried from dossiers and explicitly require web confirmation before headline use. | anchor=I-34:L94-L102
- **I-004/I34-E025** [SRC] (DEEP-I-004.discovery.md:L280| unit=I-34): E/DEP | The main design composes XGrammar’s mask layer, E-24’s pin protocol, D-16 refusal, and the existing decoding enum rather than reimplementing them. | anchor=I-34:L104-L109
- **I-004/I34-E026** [SRC] (DEEP-I-004.discovery.md:L281| unit=I-34): E/EVIDENCE | The J-15 amendment cites a determinism-triage ledger, a dated crates.io anti-slopsquat snapshot, and a detLLM interface note as its grounding. | anchor=I-34:L111-L115
- **I-004/I34-E027** [SRC] (DEEP-I-004.discovery.md:L282| unit=I-34): E/DEP | The amendment selects `llguidance` crates.io v1.7.6/MIT as a new orchestrator-side dependency, subject to edit-time reverification and user confirmation. | anchor=I-34:L117-L123, L141-L149
- **I-004/I34-E028** [SRC] (DEEP-I-004.discovery.md:L283| unit=I-34): E/FLOW | Selected Option A compiles, validates, and blake3-pins a grammar with Rust `ParserFactory` at plan time; malformed grammars fail before any inference call. | anchor=I-34:L117-L123
- **I-004/I34-E029** [SRC] (DEEP-I-004.discovery.md:L284| unit=I-34): E/DET | Grammar pins use the same blake3 stack as RRA so plan/execution drift can be detected by `replay --verify`. | anchor=I-34:L119-L123
- **I-004/I34-E030** [SRC] (DEEP-I-004.discovery.md:L285| unit=I-34): E/INT | The Python sidecar’s XGrammar remains the inference-time masker; plan-time Rust parsing/pinning and runtime Python masking stay separate. | anchor=I-34:L124-L132
- **I-004/I34-E031** [SRC] (DEEP-I-004.discovery.md:L286| unit=I-34): E/LIMIT | Option B—Rust-side token masking with llguidance—is rejected because it would redesign the Python logits loop, duplicate XGrammar, and violate the existing substrate contract. | anchor=I-34:L124-L132
- **I-004/I34-E032** [SRC] (DEEP-I-004.discovery.md:L287| unit=I-34): E/LIMIT | The amendment cites HD-3 as banning constrained decoding on reasoning/coder/planner roles; the Rust addition is compile/validate only. | anchor=I-34:L128-L132
- **I-004/I34-E033** [SRC] (DEEP-I-004.discovery.md:L288| unit=I-34): E/STATE | Compiled llguidance parser state is not serialized; the durable artifact is grammar spec plus blake3 pin, and the parser is recompiled on every boot. | anchor=I-34:L134-L139
- **I-004/I34-E034** [SRC] (DEEP-I-004.discovery.md:L289| unit=I-34): E/EVIDENCE | Sub-second JSON-schema compilation is a README-derived performance claim, not a locally defined acceptance threshold. | anchor=I-34:L134-L138
- **I-004/I34-E035** [SRC] (DEEP-I-004.discovery.md:L290| unit=I-34): E/SEC | Before adding the dependency, the executor must re-check crates.io identity/license/maintainer/download evidence and obtain explicit user confirmation. | anchor=I-34:L141-L147
- **I-004/I34-E036** [SRC] (DEEP-I-004.discovery.md:L291| unit=I-34): E/EVIDENCE | The dated snapshot reports v1.7.6, MIT, roughly 525K downloads, and guidance-ai/Moskal/MS origin; it is expressly non-authoritative at edit time. | anchor=I-34:L141-L147
- **I-004/I34-E037** [SRC] (DEEP-I-004.discovery.md:L292| unit=I-34 SURFACE): E/LIMIT | `xgrammar-rs` was considered and rejected as a C++ binding with far lower reported adoption that duplicates the existing XGrammar surface. | anchor=I-34:L147-L149
- **I-004/I34-A001** [AC] (DEEP-I-004.discovery.md:L293| unit=I-34): A/INT | The grammar-store module path and ownership are TBD, including whether it lives under providers or orchestrator. | anchor=I-34:L57-L60, L65-L70
- **I-004/I34-A002** [AC] (DEEP-I-004.discovery.md:L294| unit=I-34): A/SEC | “Untrusted source” and enum detection/trust provenance have no exact schema-loader classification algorithm. | anchor=I-34:L20-L23
- **I-004/I34-A003** [AC] (DEEP-I-004.discovery.md:L295| unit=I-34): A/FLOW | Commit-point identification, thinking-token grammar augmentation, and transition semantics are not specified as an algorithm or grammar transform. | anchor=I-34:L14-L18, L43-L49
- **I-004/I34-A004** [AC] (DEEP-I-004.discovery.md:L296| unit=I-34): A/DATA | Manifest serialization, pin entry schema, update authorization, and freeze-manifest nesting are not defined. | anchor=I-34:L20-L23, L43-L47, L57-L59
- **I-004/I34-A005** [AC] (DEEP-I-004.discovery.md:L297| unit=I-34): A/TEST | The reasoning fixture, pass-rate denominator, number of seeds, and treatment of ties with FreeText are unspecified. | anchor=I-34:L47-L50, L89-L92
- **I-004/I34-A006** [AC] (DEEP-I-004.discovery.md:L298| unit=I-34): A/DEP | Exact dependency version policy is unclear: the amendment names v1.7.6 but also commands edit-time reverification rather than trusting that snapshot. | anchor=I-34:L117-L123, L141-L147
- **I-004/I34-C001** [CON] (DEEP-I-004.discovery.md:L299| unit=I-34): C/SCOPE | The main body requires reasoning-heavy CRANE/final-only constraints, while the amendment says HD-3 bans constrained decoding on reasoning/coder/planner roles; role applicability must be reconciled before execution. | anchor=I-34:L14-L18, L47-L50, L128-L132
- **I-004/SD-I34-001** [SD] (DEEP-I-004.discovery.md:L599| unit=I-34): **CORRECT** summary claim that implementation adds “the sidecar mask layer”: Python-sidecar CRANE integration is explicitly Tier 2; Tier 1 is provider/Rust runtime plus validation/pinning. | 09-I:L923-L943 | anchor=I34-E014, I34-E018, I34-C001
- **I-004/SD-I34-002** [SD] (DEEP-I-004.discovery.md:L600| unit=I-34 SURFACE): **REMOVE/FLAG** summary claim that Tier 1 records a tight-grammar arm’s measured tax; the numbered source only carries an external premise and requires CRANE pass rate >= FreeText in its fixture. | 09-I:L925-L940 | anchor=I34-E003, I34-E016, I34-E024, I34-A005
- **I-004/SD-I34-003** [SD] (DEEP-I-004.discovery.md:L601| unit=I-34 SURFACE): **ADD** complete event family, enum-source rejection, R8 freeze, exact production routes, D-16 refusal, and cross-language/UI evidence. | 09-I:L928-L943 | anchor=I34-E007–I34-E017, I34-E020–I34-E023
- **I-004/SD-I34-004** [SD] (DEEP-I-004.discovery.md:L602| unit=I-34): **ADD** cross-language verifier-pack deferral, module-path TBD, commit-point/manifest/trust ambiguities, and local fixture-statistics gap. | 09-I:L919-L943 | anchor=I34-E019, I34-E021, I34-A001–I34-A005
- **I-004/SD-I34-005** [SD] (DEEP-I-004.discovery.md:L603| unit=I-34): **ADD** J-15 Option-A amendment: plan-time llguidance `ParserFactory` compile/validate/pin with XGrammar unchanged at inference. | 09-I:L919-L943 | anchor=I34-E026–I34-E031
- **I-004/SD-I34-006** [SD] (DEEP-I-004.discovery.md:L604| unit=I-34): **ADD** nonserializable parser state, spec+pin persistence, boot recompilation, replay drift detection, and README-only compile-performance evidence. | 09-I:L919-L943 | anchor=I34-E029, I34-E033–I34-E034
- **I-004/SD-I34-007** [SD] (DEEP-I-004.discovery.md:L605| unit=I-34): **ADD** new-dependency user approval/current anti-slopsquat recheck, dated v1.7.6 snapshot caveat, and xgrammar-rs rejection. | 09-I:L919-L943 | anchor=I34-E027, I34-E035–I34-E037, I34-A006
- **I-004/SD-I34-008** [SD] (DEEP-I-004.discovery.md:L606| unit=I-34 SURFACE): **FLAG CONFLICT** reasoning-heavy CRANE/final-only constraints versus amendment HD-3 role ban must be resolved; the summary does not surface this. | 09-I:L923-L940 | anchor=I34-E004–I34-E005, I34-E032, I34-C001

#### I-004 / unit `I-35` (38 records)

- **I-004/I35-E001** [SRC] (DEEP-I-004.discovery.md:L309| unit=I-35): E/STATUS | I-35 is a strong-grounded Track-18 can’t-wreck BUILD, effort M, Wave 2, hard-dependent on I-1. | anchor=I-35:L1-L3
- **I-004/I35-E002** [SRC] (DEEP-I-004.discovery.md:L310| unit=I-35): E/CAP | It adds the absent fourth enforcement dimension—ordering over tool-call history—on top of state masks, argv shape, and committed tool sets. | anchor=I-35:L5-L9, L32-L39, L101-L106
- **I-004/I35-E003** [SRC] (DEEP-I-004.discovery.md:L311| unit=I-35): E/DATA | A `SequenceRule` consists of tool-class trigger, deterministic predicate over the current session’s ordered SSB/journal history, and an enforcement action. | anchor=I-35:L11-L20, L41-L45, L64-L70
- **I-004/I35-E004** [SRC] (DEEP-I-004.discovery.md:L312| unit=I-35): E/TEST | One supported predicate requires a `test_run` PASS since the most recent `src_edit`. | anchor=I-35:L13-L17
- **I-004/I35-E005** [SRC] (DEEP-I-004.discovery.md:L313| unit=I-35): E/SEC | One supported predicate forbids a write-class call after `untrusted_ingest` until an intervening `taint_check` PASS exists. | anchor=I-35:L13-L17, L41-L48
- **I-004/I35-E006** [SRC] (DEEP-I-004.discovery.md:L314| unit=I-35): E/SEC | One supported predicate requires an in-session `auditor_PASS` before `git_push`. | anchor=I-35:L13-L17, L41-L48
- **I-004/I35-E007** [SRC] (DEEP-I-004.discovery.md:L315| unit=I-35): E/FLOW | The rule model defines three enforcement actions: halt/refuse, require-confirmation advisory, or substitute with a safer tool class. | anchor=I-35:L18-L20
- **I-004/I35-E008** [SRC] (DEEP-I-004.discovery.md:L316| unit=I-35): E/FAIL | Halt refuses the call and emits `ToolSequenceVerdict(OrderViolation)`; Tier-1 halt fixtures also emit `HaltEnforced`. | anchor=I-35:L18-L20, L45-L48, L86-L89
- **I-004/I35-E009** [SRC] (DEEP-I-004.discovery.md:L317| unit=I-35): E/SEC | `SequenceRuleSet` is TOML-deserialized and R8-frozen; its blake3 hash is carried in the gate-freeze manifest and the running session may not edit it. | anchor=I-35:L21-L23, L41-L50, L56-L62, L80-L83
- **I-004/I35-E010** [SRC] (DEEP-I-004.discovery.md:L318| unit=I-35): E/DET | Replay re-evaluates the same concrete ordered row list and must derive identical sequence verdicts under GD-2. | anchor=I-35:L21-L27, L86-L89
- **I-004/I35-E011** [SRC] (DEEP-I-004.discovery.md:L319| unit=I-35): E/SEC | The source claims CDA/token-level attack immunity because predicates consume orchestrator-owned action-history rows rather than generated token content. | anchor=I-35:L5-L9, L25-L27
- **I-004/I35-E012** [SRC] (DEEP-I-004.discovery.md:L320| unit=I-35): E/DEFER | Pro2Guard-style probabilistic DTMC reachability is deferred until a trace corpus exists; deterministic sequence rules ship first. | anchor=I-35:L29-L30, L52-L54, L80-L83, L91-L99
- **I-004/I35-E013** [SRC] (DEEP-I-004.discovery.md:L321| unit=I-35): E/STATUS | Per-state Phase-3A masks, E-30 argv gating, and F-3 committed tool sets are PLAN-asserted existing; temporal ordering was absent. | anchor=I-35:L32-L39, L71-L76, L91-L106
- **I-004/I35-E014** [SRC] (DEEP-I-004.discovery.md:L322| unit=I-35): E/DEP | IS-11 taint/provenance rows and IS-14’s production tool catalog are existing predicate/dispatch inputs to compose, not rebuild. | anchor=I-35:L37-L39, L56-L62, L71-L76, L101-L106
- **I-004/I35-E015** [SRC] (DEEP-I-004.discovery.md:L323| unit=I-35): E/DEP | I-1 Supervisor must exist before the rule gate can wire into production dispatch. | anchor=I-35:L3-L3, L39-L39, L71-L79, L97-L102
- **I-004/I35-E016** [SRC] (DEEP-I-004.discovery.md:L324| unit=I-35): E/CAP | Tier 1 creates `crates/orchestrator/src/tool_sequence.rs::{SequenceRule,SequenceRuleSet,evaluate_sequence_rule}`. | anchor=I-35:L41-L44, L56-L59, L64-L70
- **I-004/I35-E017** [SRC] (DEEP-I-004.discovery.md:L325| unit=I-35): E/CFG | Tier 1 creates frozen `rules/tool_sequence_core.toml` covering at least `git_push`, `file_write`, and `untrusted_ingest` classes. | anchor=I-35:L41-L49, L56-L61, L64-L70
- **I-004/I35-E018** [SRC] (DEEP-I-004.discovery.md:L326| unit=I-35): E/EVENT | I-35 adds one `ToolSequenceVerdict` action type with `Allowed`, `OrderViolation`, and `HaltEnforced`, baseline+1 from the dated 221/532 registry. | anchor=I-35:L41-L48, L64-L70, L86-L99
- **I-004/I35-E019** [SRC] (DEEP-I-004.discovery.md:L327| unit=I-35): E/FLOW | The sequence check executes at I-1 Supervisor dispatch **after** existing per-call gates and **before** tool execution. | anchor=I-35:L41-L47, L56-L59, L77-L79
- **I-004/I35-E020** [SRC] (DEEP-I-004.discovery.md:L328| unit=I-35): E/TEST | Required fixtures cover a compliant sequence, push without auditor PASS, and write after untrusted ingest without taint-check PASS. | anchor=I-35:L41-L48, L86-L89
- **I-004/I35-E021** [SRC] (DEEP-I-004.discovery.md:L329| unit=I-35 SURFACE): E/SURFACE | Sequence-verdict rows render in the existing gate/MonitorWindow panel through Svelte plus Tauri, with `cross_lang_diff.py` required; a reasoning decision document is also required. | anchor=I-35:L47-L50, L56-L62, L84-L89
- **I-004/I35-E022** [SRC] (DEEP-I-004.discovery.md:L330| unit=I-35): E/DEFER | Rules spanning multiple sessions are deferred; Tier 1 is session-scoped. | anchor=I-35:L52-L55
- **I-004/I35-E023** [SRC] (DEEP-I-004.discovery.md:L331| unit=I-35): E/INT | Production chain is `supervisor::dispatch_tool -> tool_sequence::evaluate_sequence_rule -> ToolSequenceVerdict`; test-only callers do not satisfy R7. | anchor=I-35:L77-L79
- **I-004/I35-E024** [SRC] (DEEP-I-004.discovery.md:L332| unit=I-35): E/LIMIT | Production code forbids mutable self-edited rule sets, Tier-1 probabilistic reachability, `.unwrap()`/`.expect()`, and blake2/sha2. | anchor=I-35:L80-L83
- **I-004/I35-E025** [SRC] (DEEP-I-004.discovery.md:L333| unit=I-35): E/OBS | Violation payloads must name the unsatisfied predicate; compliant flow emits `Allowed`, while both negative sequences emit `OrderViolation` plus `HaltEnforced`. | anchor=I-35:L86-L89
- **I-004/I35-E026** [SRC] (DEEP-I-004.discovery.md:L334| unit=I-35): E/TEST | Editing `tool_sequence_core.toml` during a run must make `gate_freeze_check.py` exit nonzero and block the run. | anchor=I-35:L80-L81, L86-L89
- **I-004/I35-E027** [SRC] (DEEP-I-004.discovery.md:L335| unit=I-35): E/EVIDENCE | AgentSpec/Agent-C supply the trigger/predicate/enforcement shape and Pro2Guard supplies deferred prior art; their arXiv IDs require web confirmation before headline use. | anchor=I-35:L91-L99
- **I-004/I35-A001** [AC] (DEEP-I-004.discovery.md:L336| unit=I-35): A/DATA | The TOML schema, predicate DSL/AST, history-query language, validation errors, and rule precedence are not defined. | anchor=I-35:L11-L23, L41-L45
- **I-004/I35-A002** [AC] (DEEP-I-004.discovery.md:L337| unit=I-35): A/FLOW | Require-confirmation and substitute are part of the rule model but have no Tier-1 event kinds, fixtures, production acceptance evidence, or exact continuation semantics. | anchor=I-35:L18-L20, L41-L48, L64-L70, L86-L89
- **I-004/I35-A003** [AC] (DEEP-I-004.discovery.md:L338| unit=I-35): A/CFG | Mapping concrete tools to `git_push`, `file_write`, `untrusted_ingest`, and “safer” substitute classes is not specified. | anchor=I-35:L13-L20, L41-L48
- **I-004/I35-A004** [AC] (DEEP-I-004.discovery.md:L339| unit=I-35): A/STATE | Session boundary, replay-history truncation, and behavior after checkpoint restore are not defined for sequence predicates. | anchor=I-35:L14-L17, L21-L27, L52-L55
- **I-004/I35-A005** [AC] (DEEP-I-004.discovery.md:L340| unit=I-35 SURFACE): A/SEC | “CDA-immune by construction” is an explicit design claim, but this plan defines no adversarial acceptance test proving row provenance cannot be forged through another tool surface. | anchor=I-35:L5-L9, L25-L27, L86-L89
- **I-004/SD-I35-001** [SD] (DEEP-I-004.discovery.md:L612| unit=I-35): **CORRECT** summary’s “taint check has run”: source requires an intervening `taint_check PASS`, specifically before a write-class call. | 09-I:L951-L954 | anchor=I35-E005
- **I-004/SD-I35-002** [SD] (DEEP-I-004.discovery.md:L613| unit=I-35): **CORRECT** summary’s “mutable equal to zero”: the numbered source specifies an R8-frozen rule hash but no `mutable` field/value. | 09-I:L963-L970 | anchor=I35-E009, I35-E026
- **I-004/SD-I35-003** [SD] (DEEP-I-004.discovery.md:L614| unit=I-35 SURFACE): **ADD/FLAG** require-confirmation and substitute are explicit rule actions but lack Tier-1 events/fixtures/continuation semantics. | 09-I:L955-L957 | anchor=I35-E007–I35-E008, I35-A002
- **I-004/SD-I35-004** [SD] (DEEP-I-004.discovery.md:L615| unit=I-35): **ADD** exact post-existing-gates/pre-execution placement, current-session scope, full event kinds, class fixtures, and cross-session deferral. | 09-I:L949-L972 | anchor=I35-E003–I35-E006, I35-E017–I35-E023
- **I-004/SD-I35-005** [SD] (DEEP-I-004.discovery.md:L616| unit=I-35): **ADD** taint/catalog composition, production reachability, gate-freeze negative test, arXiv caution, and no-reimplementation constraints. | 09-I:L949-L972 | anchor=I35-E012–I35-E016, I35-E024–I35-E027
- **I-004/SD-I35-006** [SD] (DEEP-I-004.discovery.md:L617| unit=I-35 SURFACE): **FLAG** undefined TOML DSL/class mapping/history boundary and untested absolute CDA-immunity claim. | 09-I:L949-L972 | anchor=I35-A001, I35-A003–I35-A005

#### I-004 / unit `I-40` (41 records)

- **I-004/I40-E001** [SRC] (DEEP-I-004.discovery.md:L350| unit=I-40): E/STATUS | I-40 is a strong-grounded Track-18 wedge BUILD, effort L, Wave 5; full-form expansion is required before EXECUTING. | anchor=I-40:L1-L3, L11-L11
- **I-004/I40-E002** [SRC] (DEEP-I-004.discovery.md:L351| unit=I-40 SURFACE): E/CAP | It exposes ELAI’s existing taint/write-gate/RRA/attestation trust layer around non-ELAI external agents as an adoption wedge. | anchor=I-40:L5-L9, L13-L35
- **I-004/I40-E003** [SRC] (DEEP-I-004.discovery.md:L352| unit=I-40 SURFACE): E/SURFACE | Tier 1 publishes a versioned open format at `docs/spec/rra-spec-v1.md` and gives external implementers a conformance target. | anchor=I-40:L16-L20, L50-L54, L64-L65, L75-L80
- **I-004/I40-E004** [SRC] (DEEP-I-004.discovery.md:L353| unit=I-40): E/DATA | The spec includes I-18 `chain_prev_hash`, taint-flow payload schema, write-gate record schema, and the RRA bundle envelope. | anchor=I-40:L16-L20
- **I-004/I40-E005** [SRC] (DEEP-I-004.discovery.md:L354| unit=I-40 SURFACE): E/SURFACE | `elai rra verify <path>` checks a bundle against the published spec and must reject tampering with nonzero exit. | anchor=I-40:L16-L20, L50-L54, L66-L67, L89-L103
- **I-004/I40-E006** [SRC] (DEEP-I-004.discovery.md:L355| unit=I-40 SURFACE): E/SURFACE | `elai wrap -- <agent cmd>` is a minimal MCP/stdio proxy that intercepts an external agent’s tool traffic. | anchor=I-40:L22-L23, L50-L57, L64-L67, L89-L91
- **I-004/I40-E007** [SRC] (DEEP-I-004.discovery.md:L356| unit=I-40): E/SEC | Any untrusted ingest in wrapped traffic goes through existing G-1 taint tagging. | anchor=I-40:L22-L27, L68-L70, L81-L90, L100-L103
- **I-004/I40-E008** [SRC] (DEEP-I-004.discovery.md:L357| unit=I-40): E/SEC | Destructive wrapped tool classes go through existing F-1/F-9/G-3 write gates. | anchor=I-40:L22-L27, L68-L71, L81-L90, L100-L103
- **I-004/I40-E009** [SRC] (DEEP-I-004.discovery.md:L358| unit=I-40 SURFACE): E/OBS | Each wrapped tool call emits through the existing `rra.rs` surface, yielding a wrapped-run RRA. | anchor=I-40:L22-L27, L41-L47, L68-L72, L81-L90
- **I-004/I40-E010** [SRC] (DEEP-I-004.discovery.md:L359| unit=I-40): E/PRIV | G-9 redaction is a mandatory pre-export step; an exported bundle containing raw secret bytes is a P0/stub-ban violation. | anchor=I-40:L5-L9, L22-L27, L37-L38, L43-L47, L68-L72, L93-L103
- **I-004/I40-E011** [SRC] (DEEP-I-004.discovery.md:L360| unit=I-40): E/LIMIT | The wrapper adds no new trust primitives or forked gate logic; it composes the shipped substrate. | anchor=I-40:L29-L32, L93-L97, L115-L121
- **I-004/I40-E012** [SRC] (DEEP-I-004.discovery.md:L361| unit=I-40): E/TEST | Tier 1 proves the wrapper with exactly one controllable external-agent fixture rather than a real-server matrix. | anchor=I-40:L29-L32, L50-L57, L93-L97
- **I-004/I40-E013** [SRC] (DEEP-I-004.discovery.md:L362| unit=I-40 SURFACE): E/STATE | The published strategy records wrapper/open standard as the on-ramp and deep ELAI orchestration as the premium destination. | anchor=I-40:L34-L35, L50-L53
- **I-004/I40-E014** [SRC] (DEEP-I-004.discovery.md:L363| unit=I-40): E/STATUS | `rra.rs::build_rra`/`verify_rra` and attestation are cited as shipped; the wrapper must extend/compose them, never fork them. | anchor=I-40:L40-L43, L68-L68, L81-L83, L105-L107, L115-L116
- **I-004/I40-E015** [SRC] (DEEP-I-004.discovery.md:L364| unit=I-40): E/STATUS | `capability_flow.rs::taint_text` and `safety/src/provenance.rs` are cited as shipped taint substrates. | anchor=I-40:L41-L43, L68-L70, L81-L84, L105-L108
- **I-004/I40-E016** [SRC] (DEEP-I-004.discovery.md:L365| unit=I-40): E/STATUS | F-1/F-9/G-3 gates, G-9 redaction, B-37 adapters, and E-24 MCP hardening are PLAN-asserted seams that require execution-time re-grep. | anchor=I-40:L43-L47, L68-L72, L81-L88, L105-L121
- **I-004/I40-E017** [SRC] (DEEP-I-004.discovery.md:L366| unit=I-40): E/DEP | I-18’s per-row chain is required for the documented chained format; the source says I-18 must ship first or in Wave-5 parallel. | anchor=I-40:L16-L18, L48-L48, L81-L88, L105-L121
- **I-004/I40-E018** [SRC] (DEEP-I-004.discovery.md:L367| unit=I-40 SURFACE): E/DATA | Planned implementation files are `crates/cli/src/wrap_cmd.rs`, `crates/cli/src/rra_verify_cmd.rs`, the spec document, and `workflow/reasoning/I-40-decisions.md`. | anchor=I-40:L50-L57, L64-L67, L75-L80
- **I-004/I40-E019** [SRC] (DEEP-I-004.discovery.md:L368| unit=I-40): E/EVENT | I-40 adds no SSB action type and reuses existing `taint_flow` and `run_artifact_emitted` rows, leaving the dated 221/532 baseline unchanged. | anchor=I-40:L50-L56, L75-L80, L89-L90, L105-L113
- **I-004/I40-E020** [SRC] (DEEP-I-004.discovery.md:L369| unit=I-40 SURFACE): E/SURFACE | Wrapped-run rows reuse the existing RRA viewer; no new Svelte component is planned, and `cross_lang_diff.py` applies only if execution-time re-grep confirms a live Svelte counterpart. | anchor=I-40:L54-L57, L73-L73, L98-L103
- **I-004/I40-E021** [SRC] (DEEP-I-004.discovery.md:L370| unit=I-40): E/DEFER | A matrix of multiple real MCP server implementations is Tier 2 and user-approval/network-dependency gated. | anchor=I-40:L29-L32, L59-L61
- **I-004/I40-E022** [SRC] (DEEP-I-004.discovery.md:L371| unit=I-40): E/DEFER | Commercial packaging/distribution is deferred as a post-validation business decision. | anchor=I-40:L59-L62
- **I-004/I40-E023** [SRC] (DEEP-I-004.discovery.md:L372| unit=I-40): E/DEFER | Sigstore/Rekor publication for external-agent RRAs waits for I-18. | anchor=I-40:L59-L63
- **I-004/I40-E024** [SRC] (DEEP-I-004.discovery.md:L373| unit=I-40 SURFACE): E/INT | Production chains are wrap CLI -> taint -> write gate -> RRA emit -> redaction and verify CLI -> conformance checker; test-only callers do not satisfy R7. | anchor=I-40:L89-L92
- **I-004/I40-E025** [SRC] (DEEP-I-004.discovery.md:L374| unit=I-40): E/LIMIT | Production code forbids unredacted export, new wrapper-native trust primitives, expansion beyond one Tier-1 fixture, `.unwrap()`/`.expect()`, and blake2/sha2. | anchor=I-40:L93-L97
- **I-004/I40-E026** [SRC] (DEEP-I-004.discovery.md:L375| unit=I-40): E/TEST | The fixture performs untrusted ingest plus a seeded malicious write; expected rows prove taint propagation and a write-gate BLOCK outside ELAI’s native generator loop. | anchor=I-40:L100-L103
- **I-004/I40-E027** [SRC] (DEEP-I-004.discovery.md:L376| unit=I-40 SURFACE): E/TEST | A wrapper-produced bundle must pass `elai rra verify`, a tampered bundle must fail, and no planted raw secret bytes may appear in export. | anchor=I-40:L100-L103
- **I-004/I40-E028** [SRC] (DEEP-I-004.discovery.md:L377| unit=I-40): E/EVIDENCE | Competitive/trust reviews are cited as converging on wrapper-as-wedge; IC-16 says the trust spine is substantially wired. | anchor=I-40:L105-L113
- **I-004/I40-A001** [AC] (DEEP-I-004.discovery.md:L378| unit=I-40): A/DEP | “I-18 must ship first or in Wave-5 parallel” does not define how I-40 can document/verify the chained format before I-18’s record layout is final. | anchor=I-40:L16-L18, L48-L48, L62-L63, L88-L88, L121-L121
- **I-004/I40-A002** [AC] (DEEP-I-004.discovery.md:L379| unit=I-40): A/DATA | The plan lists required spec sections but does not define normative field types, canonical serialization, version negotiation, compatibility policy, or conformance error taxonomy. | anchor=I-40:L16-L20, L50-L54
- **I-004/I40-A003** [AC] (DEEP-I-004.discovery.md:L380| unit=I-40): A/SEC | MCP/stdio interception completeness, process/platform handling, unsupported tool channels, and bypass behavior are not specified. | anchor=I-40:L22-L32, L50-L54
- **I-004/I40-A004** [AC] (DEEP-I-004.discovery.md:L381| unit=I-40 SURFACE): A/SURFACE | The RRA viewer’s live Svelte pairing is explicitly unknown until execution-time re-grep. | anchor=I-40:L73-L73, L98-L99
- **I-004/I40-A005** [AC] (DEEP-I-004.discovery.md:L382| unit=I-40 SURFACE): A/PRIV | G-9 owns redaction, but I-40 supplies no redaction failure policy, secret canary schema, or rule for redacting command output before versus after row sealing. | anchor=I-40:L22-L27, L37-L38, L68-L72, L100-L103
- **I-004/SD-I40-001** [SD] (DEEP-I-004.discovery.md:L623| unit=I-40): **QUALIFY** summary’s “write/dependency/blast-radius gates” expansion: the numbered plan groups F-1/F-9/G-3 as destructive-class write gates and does not separately specify three wrapper stages. | 09-I:L1086-L1095 | anchor=I40-E007–I40-E011, I40-E016
- **I-004/SD-I40-002** [SD] (DEEP-I-004.discovery.md:L624| unit=I-40): **QUALIFY** summary “proxies wrapped tool input and output”: the numbered source explicitly says intercepts external tool traffic at MCP/stdio, without separately defining output-proxy semantics. | 09-I:L1097-L1099 | anchor=I40-E006, I40-A003
- **I-004/SD-I40-003** [SD] (DEEP-I-004.discovery.md:L625| unit=I-40): **ADD** exact open-spec contents (`chain_prev_hash`, taint schema, write-gate schema, envelope), I-18 ordering ambiguity, and normative schema gaps. | 09-I:L1083-L1096 | anchor=I40-E003–I40-E005, I40-E017, I40-A001–I40-A002
- **I-004/SD-I40-004** [SD] (DEEP-I-004.discovery.md:L626| unit=I-40): **ADD** no-new-action reuse of `taint_flow`/`run_artifact_emitted`, exact production chains, one-fixture limit, and new-trust-primitive ban. | 09-I:L1086-L1104 | anchor=I40-E012, I40-E018–I40-E019, I40-E024–I40-E027
- **I-004/SD-I40-005** [SD] (DEEP-I-004.discovery.md:L627| unit=I-40): **ADD** real MCP matrix/network approval, commercial packaging, and Rekor publication deferrals. | 09-I:L1079-L1104 | anchor=I40-E021–I40-E023
- **I-004/SD-I40-006** [SD] (DEEP-I-004.discovery.md:L628| unit=I-40 SURFACE): **QUALIFY** RRA viewer surface: source says Svelte pairing/cross-language gate is conditional on execution-time re-grep, not established. | 09-I:L1099-L1099 | anchor=I40-E020, I40-A004
- **I-004/SD-I40-007** [SD] (DEEP-I-004.discovery.md:L629| unit=I-40): **ADD ORIENTATION-ONLY STATUS** the discovery summary reports `docs/spec/rra-spec-v1.md` already exists and requires conformance inspection; that fact is not established by the numbered source and was not promoted to a source atom. | 09-I:L1702-L1705 | anchor=I40-E003, I40-A002
- **I-004/SD-I40-008** [SD] (DEEP-I-004.discovery.md:L630| unit=I-40): **ADD** redaction-stage/bypass uncertainties and explicit no-raw-secret P0 boundary. | 09-I:L1092-L1104 | anchor=I40-E010, I40-E025–I40-E027, I40-A003, I40-A005

#### I-004 / unit `I-42` (43 records)

- **I-004/I42-E001** [SRC] (DEEP-I-004.discovery.md:L392| unit=I-42): E/STATUS | I-42 is a moderate-grounded Track-19 human-as-verifier BUILD, effort M, Wave 3, dependent on I-41 and bound to the §6 ban-LLM-judge rule. | anchor=I-42:L1-L3
- **I-004/I42-E002** [SRC] (DEEP-I-004.discovery.md:L393| unit=I-42): E/CFG | `RunProfile` has `Orchestrate` and `CritiqueOnly`; `Orchestrate` remains the default and the profile is selected at dispatch. | anchor=I-42:L12-L15, L41-L46, L64-L68
- **I-004/I42-E003** [SRC] (DEEP-I-004.discovery.md:L394| unit=I-42): E/STATE | Both profiles use one orchestration engine and one identical off-agent verification spine; profile changes only who writes and whether executors/fixers spawn. | anchor=I-42:L4-L10, L20-L25, L82-L86, L100-L116
- **I-004/I42-E004** [SRC] (DEEP-I-004.discovery.md:L395| unit=I-42): E/FLOW | In `CritiqueOnly`, the human operator is the sole code writer; spawned agents are for critique rather than implementation. | anchor=I-42:L4-L10, L12-L19
- **I-004/I42-E005** [SRC] (DEEP-I-004.discovery.md:L396| unit=I-42): E/SEC | Executor/fixer spawn is structurally gated off so no spawned executor/fixer touches disk. | anchor=I-42:L4-L10, L12-L16, L41-L48, L64-L71
- **I-004/I42-E006** [SRC] (DEEP-I-004.discovery.md:L397| unit=I-42): E/SEC | The generator may emit no disk-mutating Directive in `CritiqueOnly`; this is an execution/spawn gate, not a prompt request. | anchor=I-42:L12-L16, L20-L25, L36-L38, L41-L48, L82-L84
- **I-004/I42-E007** [SRC] (DEEP-I-004.discovery.md:L398| unit=I-42): E/FLOW | I-41 `LensSet` runs at design time and surfaces findings for operator revision. | anchor=I-42:L12-L18, L30-L30, L41-L48, L72-L74
- **I-004/I42-E008** [SRC] (DEEP-I-004.discovery.md:L399| unit=I-42): E/FLOW | The shipped auditor plus F-1/F-4/G-2/B-41 off-agent gate spine run at ship time on the operator’s diff. | anchor=I-42:L16-L19, L31-L32, L41-L48, L72-L75
- **I-004/I42-E009** [SRC] (DEEP-I-004.discovery.md:L400| unit=I-42): E/STATE | The profile loop is Design critique -> operator revision/implementation -> ship-time auditor/spine -> human-judged iteration to convergence. | anchor=I-42:L16-L19
- **I-004/I42-E010** [SRC] (DEEP-I-004.discovery.md:L401| unit=I-42): E/EVIDENCE | The source positions this profile for high-stakes, regulated, or low-trust-in-autonomy work, carrying the source-design claim that agent-spawned writes caused more harm than good. | anchor=I-42:L20-L23, L100-L106
- **I-004/I42-E011** [SRC] (DEEP-I-004.discovery.md:L402| unit=I-42): E/OBS | Core invariant: the SSB set of `file_write` rows attributable to spawned executor/fixer agents is empty in `CritiqueOnly`. | anchor=I-42:L20-L23, L41-L49, L64-L71, L91-L98
- **I-004/I42-E012** [SRC] (DEEP-I-004.discovery.md:L403| unit=I-42): E/LIMIT | Prompt-only “do not write” and a separate engine fork are explicitly declined. | anchor=I-42:L24-L25, L82-L87
- **I-004/I42-E013** [SRC] (DEEP-I-004.discovery.md:L404| unit=I-42): E/STATUS | The orchestrate executor/fixer dispatch, auditor/spine, and B-22 mode-plumbing pattern are PLAN-asserted shipped and require execution-time re-grep. | anchor=I-42:L27-L38, L55-L62, L100-L108
- **I-004/I42-E014** [SRC] (DEEP-I-004.discovery.md:L405| unit=I-42): E/CFG | `RunProfile` is orthogonal to B-22 `GateMode`; it must not overload the gate’s autonomous/interactive choice. | anchor=I-42:L33-L35, L72-L77, L82-L87, L110-L114
- **I-004/I42-E015** [SRC] (DEEP-I-004.discovery.md:L406| unit=I-42): E/RECOVERY | If I-1 is shipped, Supervisor rejects executor-authored side-effect Directives; otherwise Tier 1 gates directly at the spawn site, with the exact seam chosen at edit time. | anchor=I-42:L36-L38, L43-L48, L72-L77
- **I-004/I42-E016** [SRC] (DEEP-I-004.discovery.md:L407| unit=I-42): E/STATUS | No `RunProfile` concept existed in the cited inventory. | anchor=I-42:L39-L39
- **I-004/I42-E017** [SRC] (DEEP-I-004.discovery.md:L408| unit=I-42): E/CAP | Tier 1 creates `crates/orchestrator/src/run_profile.rs::{RunProfile,from_cli_args}` and the executor/fixer spawn gate. | anchor=I-42:L41-L48, L55-L60, L64-L71
- **I-004/I42-E018** [SRC] (DEEP-I-004.discovery.md:L409| unit=I-42 SURFACE): E/SURFACE | CLI selection is `elai run --profile critique-only`; ordinary orchestrate dispatch also checks `RunProfile` before every executor spawn. | anchor=I-42:L41-L46, L55-L58, L78-L81
- **I-004/I42-E019** [SRC] (DEEP-I-004.discovery.md:L410| unit=I-42): E/TEST | Tier 1 includes `crates/contracts/tests/i42_critique_only_no_executor_writes_contract.rs` and `workflow/reasoning/I-42-decisions.md`. | anchor=I-42:L47-L49, L55-L60, L64-L71
- **I-004/I42-E020** [SRC] (DEEP-I-004.discovery.md:L411| unit=I-42 SURFACE): E/DEFER | A Settings/dashboard profile switcher is deferred; CLI ships first. | anchor=I-42:L50-L52
- **I-004/I42-E021** [SRC] (DEEP-I-004.discovery.md:L412| unit=I-42 SURFACE): E/DEFER | Per-profile review-time versus generation-time telemetry waits for the G-5 cockpit. | anchor=I-42:L50-L53
- **I-004/I42-E022** [SRC] (DEEP-I-004.discovery.md:L413| unit=I-42): E/DEFER | Future review-only/pair profile variants are reserved but out of this session. | anchor=I-42:L50-L53
- **I-004/I42-E023** [SRC] (DEEP-I-004.discovery.md:L414| unit=I-42 SURFACE): E/INT | CLI args/run dispatch thread the profile into the PLAN-asserted `task_runner.rs` executor/fixer spawn site. | anchor=I-42:L55-L59, L72-L80
- **I-004/I42-E024** [SRC] (DEEP-I-004.discovery.md:L415| unit=I-42 SURFACE): E/SURFACE | The active profile reuses B-22’s existing StatusBar mode-badge cluster; no new component is created. | anchor=I-42:L61-L62, L88-L90
- **I-004/I42-E025** [SRC] (DEEP-I-004.discovery.md:L416| unit=I-42): E/EVENT | I-42 adds no action type; profile is surfaced through existing `phase_transition` rows and the key proof is absence of executor-attributed writes. | anchor=I-42:L64-L71, L100-L108
- **I-004/I42-E026** [SRC] (DEEP-I-004.discovery.md:L417| unit=I-42 SURFACE): E/INT | Production reachability is both critique-only CLI dispatch and normal orchestrate dispatch before spawn; tests-only callers do not satisfy R7. | anchor=I-42:L78-L81
- **I-004/I42-E027** [SRC] (DEEP-I-004.discovery.md:L418| unit=I-42): E/LIMIT | Production code prohibits prompt-only gating, an engine fork, lens output on a verdict path, `GateMode` overloading, `.unwrap()`/`.expect()`, and blake2/sha2. | anchor=I-42:L82-L87
- **I-004/I42-E028** [SRC] (DEEP-I-004.discovery.md:L419| unit=I-42): E/OBS | Expected critique-only evidence is I-41 design-time finding rows, ship-time auditor/spine verdicts, and zero executor/fixer-attributed file writes. | anchor=I-42:L91-L96
- **I-004/I42-E029** [SRC] (DEEP-I-004.discovery.md:L420| unit=I-42): E/TEST | The same fixture under default `Orchestrate` must produce executor file writes, proving the profile gate is not a no-op. | anchor=I-42:L95-L98
- **I-004/I42-E030** [SRC] (DEEP-I-004.discovery.md:L421| unit=I-42): E/DET | Off-agent spine verdict bytes must be identical across both profiles. | anchor=I-42:L95-L98
- **I-004/I42-E031** [SRC] (DEEP-I-004.discovery.md:L422| unit=I-42): E/INT | I-40 is the external-agent trust-wrapper sibling; I-42 is the internal operator-writes profile and must not restate I-40. | anchor=I-42:L110-L116
- **I-004/I42-I001** [INF] (DEEP-I-004.discovery.md:L423| unit=I-42): I/DATA | Proving an empty executor/fixer write set implicitly requires existing `file_write` rows to carry stable actor-role attribution, although I-42 does not define that schema. | anchor=I-42:L20-L23, L47-L49, L64-L71, L91-L98
- **I-004/I42-A001** [AC] (DEEP-I-004.discovery.md:L424| unit=I-42): A/SEC | The prose says all spawned agents are critique-only, but the structural invariant/gate is specifically executor/fixer-attributed writes; permissions for other spawned critic/auditor agents are not separately specified. | anchor=I-42:L4-L10, L12-L23, L41-L49
- **I-004/I42-A002** [AC] (DEEP-I-004.discovery.md:L425| unit=I-42): A/DATA | The `phase_transition` field/discriminator that encodes `RunProfile` is not identified. | anchor=I-42:L64-L71, L100-L108
- **I-004/I42-A003** [AC] (DEEP-I-004.discovery.md:L426| unit=I-42): A/DET | The fixture must produce executor writes only in Orchestrate yet identical off-agent verdict bytes across profiles; the plan does not define how resulting diffs/artifacts are held constant for that comparison. | anchor=I-42:L91-L98
- **I-004/I42-A004** [AC] (DEEP-I-004.discovery.md:L427| unit=I-42): A/INT | Exact enforcement location remains edit-time conditional between I-1 Supervisor rejection and direct spawn-site refusal. | anchor=I-42:L36-L38, L43-L48, L72-L77
- **I-004/I42-A005** [AC] (DEEP-I-004.discovery.md:L428| unit=I-42): A/FLOW | The mechanics by which an operator’s external edits are detected/ingested between design and ship phases are not specified. | anchor=I-42:L16-L19, L91-L98
- **I-004/SD-I42-001** [SD] (DEEP-I-004.discovery.md:L636| unit=I-42): **QUALIFY** summary “spawned agents may run auditor/spine”: source says spawned agents are critique-only and the shipped auditor/spine runs at ship time, without explicitly assigning that spine to spawned agent authorship. | 09-I:L1149-L1152 | anchor=I42-E004, I42-E007–I42-E009, I42-A001
- **I-004/SD-I42-002** [SD] (DEEP-I-004.discovery.md:L637| unit=I-42): **ADD** generator-level no-side-effect-Directive rule, exact design/operator/ship loop, high-stakes rationale, and one-engine/no-prompt bans. | 09-I:L1147-L1165 | anchor=I42-E003–I42-E012, I42-E027
- **I-004/SD-I42-003** [SD] (DEEP-I-004.discovery.md:L638| unit=I-42): **ADD** no-new-action/phase-transition reuse, actor-attribution inference, production reachability, and edit-time Supervisor-versus-spawn-site choice. | 09-I:L1152-L1165 | anchor=I42-E015, I42-E023, I42-E025–I42-E026, I42-I001, I42-A002, I42-A004
- **I-004/SD-I42-004** [SD] (DEEP-I-004.discovery.md:L639| unit=I-42): **ADD** deferred Settings switch, per-profile telemetry, and future profile variants. | 09-I:L1142-L1165 | anchor=I42-E020–I42-E022
- **I-004/SD-I42-005** [SD] (DEEP-I-004.discovery.md:L640| unit=I-42): **ADD** exact contract/decision artifacts and complete acceptance evidence: critique rows, zero writes, Orchestrate control arm, byte-identical spine verdicts. | 09-I:L1147-L1165 | anchor=I42-E017–I42-E019, I42-E024, I42-E028–I42-E030
- **I-004/SD-I42-006** [SD] (DEEP-I-004.discovery.md:L641| unit=I-42 SURFACE): **FLAG** permissions for non-executor agents, operator-edit ingestion, and how differing write behavior yields identical verdict bytes. | 09-I:L1147-L1165 | anchor=I42-A001, I42-A003, I42-A005

#### I-004 / unit `I-45` (37 records)

- **I-004/I45-E001** [SRC] (DEEP-I-004.discovery.md:L438| unit=I-45 SURFACE): E/STATUS | I-45 is a Track-20 can’t-lie EXTENSION, effort S-M, Wave 1, strong-grounded on ELAI and moderate-grounded on the roach/SL-11 semantics, bound to ID-4. | anchor=I-45:L1-L4
- **I-004/I45-E002** [SRC] (DEEP-I-004.discovery.md:L439| unit=I-45): E/CAP | It binds a sealed verifier receipt to the objective by hashing canonical objective bytes, preventing post-PASS goal/acceptance mutation from retaining a fresh-looking receipt. | anchor=I-45:L6-L11
- **I-004/I45-E003** [SRC] (DEEP-I-004.discovery.md:L440| unit=I-45): E/DEP | Execution must read the cited original roach-pi extension source under `docs/.source/roach-pi/`; the technique is stolen, not its code. | anchor=I-45:L13-L16, L62-L64
- **I-004/I45-E004** [SRC] (DEEP-I-004.discovery.md:L441| unit=I-45): E/DATA | `canonical_objective_bytes()` serializes orchestrator-owned TaskSpec, baby-step acceptance text, and future I-37 EARS records. | anchor=I-45:L17-L20, L25-L31, L43-L47
- **I-004/I45-E005** [SRC] (DEEP-I-004.discovery.md:L442| unit=I-45): E/DET | Canonicalization requires stable field order, normalized whitespace, and blake3. | anchor=I-45:L17-L20
- **I-004/I45-E006** [SRC] (DEEP-I-004.discovery.md:L443| unit=I-45): E/DATA | Verdict sealing stamps `objective_hash` into both the RRA seal and attestation. | anchor=I-45:L17-L21, L33-L40, L43-L47
- **I-004/I45-E007** [SRC] (DEEP-I-004.discovery.md:L444| unit=I-45 SURFACE): E/FLOW | Every receipt read path rehashes the current objective, explicitly including `elai verify-attestation`, replay, and plan-status consumers. | anchor=I-45:L19-L21, L38-L49
- **I-004/I45-E008** [SRC] (DEEP-I-004.discovery.md:L445| unit=I-45): E/FAIL | Current hash mismatch produces `ReceiptStale` and hard-fails; warn-and-continue is forbidden. | anchor=I-45:L19-L21, L43-L60
- **I-004/I45-E009** [SRC] (DEEP-I-004.discovery.md:L446| unit=I-45 SURFACE): E/LIMIT | I-45 adds no verifier, gate, objective-versioning UI, or historical-receipt migration. | anchor=I-45:L22-L23
- **I-004/I45-E010** [SRC] (DEEP-I-004.discovery.md:L447| unit=I-45): E/COMP | Historical receipts without the field are explicitly marked `LEGACY`; they may never be silently reported as fresh. | anchor=I-45:L22-L23, L50-L52
- **I-004/I45-E011** [SRC] (DEEP-I-004.discovery.md:L448| unit=I-45): E/STATUS | `rra.rs`, `attestation.rs`, and `rra_cmd.rs` are cited as shipped, correcting a stale absence note; I-45 extends them in place. | anchor=I-45:L25-L29, L38-L49
- **I-004/I45-E012** [SRC] (DEEP-I-004.discovery.md:L449| unit=I-45): E/DEP | F-4 `stdout_hash` is the sibling artifact-hash precedent; I-18 `chain_prev_hash` shares the record and requires field-layout coordination if it lands first. | anchor=I-45:L9-L11, L25-L31, L43-L47, L66-L70
- **I-004/I45-E013** [SRC] (DEEP-I-004.discovery.md:L450| unit=I-45): E/DEP | B-40 TaskSpec and baby-step blocks are current objective sources; I-37 EARS is a forward composition source, not a prerequisite. | anchor=I-45:L30-L31, L43-L47, L66-L68
- **I-004/I45-E014** [SRC] (DEEP-I-004.discovery.md:L451| unit=I-45): E/CAP | Tier 1 is serializer, seal-time stamp, read-time staleness check, and an in-process deterministic contract test. | anchor=I-45:L33-L35
- **I-004/I45-E015** [SRC] (DEEP-I-004.discovery.md:L452| unit=I-45): E/DEFER | Per-requirement hashing waits for I-37, and cross-session objective lineage is deferred. | anchor=I-45:L33-L36
- **I-004/I45-E016** [SRC] (DEEP-I-004.discovery.md:L453| unit=I-45 SURFACE): E/SURFACE | The existing RRA viewer gets a stale-receipt badge; no new component is created. | anchor=I-45:L38-L41
- **I-004/I45-E017** [SRC] (DEEP-I-004.discovery.md:L454| unit=I-45): E/EVENT | I-45 adds one `ObjectiveSealed` action type with `Sealed`, `MutationDetected`, and `ReceiptStale`, baseline+1 from the dated 221/532 registry. | anchor=I-45:L43-L45, L53-L60, L66-L70
- **I-004/I45-E018** [SRC] (DEEP-I-004.discovery.md:L455| unit=I-45 SURFACE): E/INT | Production reachability is `elai run --emit-rra` at the RRA seal site and `elai verify-attestation` at the read site; contracts-only callers do not satisfy R7. | anchor=I-45:L48-L49
- **I-004/I45-E019** [SRC] (DEEP-I-004.discovery.md:L456| unit=I-45): E/SEC | The hash is never model-settable; serialization reads orchestrator-owned state only. | anchor=I-45:L50-L52
- **I-004/I45-E020** [SRC] (DEEP-I-004.discovery.md:L457| unit=I-45): E/LIMIT | Production paths forbid blake2/sha2, model-supplied hashes, warn-and-continue, silent legacy freshness, and `.unwrap()`/`.expect()`. | anchor=I-45:L50-L52
- **I-004/I45-E021** [SRC] (DEEP-I-004.discovery.md:L458| unit=I-45): E/TEST | Acceptance seals an RRA, mutates fixture objective text, and verifies the bundle. | anchor=I-45:L53-L55
- **I-004/I45-E022** [SRC] (DEEP-I-004.discovery.md:L459| unit=I-45): E/OBS | Seal emits `ObjectiveSealed(Sealed)`; post-mutation verify emits `ObjectiveSealed(MutationDetected)` plus `ReceiptStale` and names both hashes. | anchor=I-45:L53-L60
- **I-004/I45-E023** [SRC] (DEEP-I-004.discovery.md:L460| unit=I-45): E/TEST | The unmodified objective is the negative/control arm and must verify PASS; any verifier that still passes after mutation is the broken arm and fails R2. | anchor=I-45:L58-L60
- **I-004/I45-E024** [SRC] (DEEP-I-004.discovery.md:L461| unit=I-45): E/DEP | I-39 plan status is an explicit downstream consumer that must recheck hashes. | anchor=I-45:L66-L68
- **I-004/I45-E025** [SRC] (DEEP-I-004.discovery.md:L462| unit=I-45): E/SEC | The staleness check joins the frozen off-agent gate set under R1/R2/R7/R8. | anchor=I-45:L62-L64
- **I-004/I45-I001** [INF] (DEEP-I-004.discovery.md:L463| unit=I-45): I/DET | All seal/read consumers implicitly must share the same canonical serializer/version; otherwise unchanged objectives could disagree across RRA, attestation, replay, and plan status. | anchor=I-45:L17-L21, L38-L49, L66-L68
- **I-004/I45-A001** [AC] (DEEP-I-004.discovery.md:L464| unit=I-45): A/SCOPE | Title/premise say stamp every PASS, while the build paragraph says every sealed verdict; whether non-PASS seals also require `objective_hash` is unresolved. | anchor=I-45:L1-L1, L6-L11, L17-L21
- **I-004/I45-A002** [AC] (DEEP-I-004.discovery.md:L465| unit=I-45): A/DATA | Canonical field inclusion, TaskSpec/baby-step/EARS precedence, Unicode/newline normalization, and missing-source behavior are not defined. | anchor=I-45:L17-L20, L30-L31
- **I-004/I45-A003** [AC] (DEEP-I-004.discovery.md:L466| unit=I-45): A/COMP | “Verify as legacy” does not say whether LEGACY returns success, a distinct non-fresh verdict, or a policy-controlled failure. | anchor=I-45:L22-L23, L50-L52
- **I-004/I45-A004** [AC] (DEEP-I-004.discovery.md:L467| unit=I-45): A/EVENT | Acceptance says `ObjectiveSealed(MutationDetected) + ReceiptStale`, while the event list makes `ReceiptStale` an `ObjectiveSealed` kind; error versus event identity is ambiguous. | anchor=I-45:L43-L45, L53-L57, L66-L70
- **I-004/I45-A005** [AC] (DEEP-I-004.discovery.md:L468| unit=I-45): A/STATE | Detection/versioning for receipts lacking `objective_hash` is not specified beyond the LEGACY marker. | anchor=I-45:L22-L23
- **I-004/SD-I45-001** [SD] (DEEP-I-004.discovery.md:L647| unit=I-45): **ADD** exact canonical sources/order/whitespace/blake3 requirements, orchestrator-only ownership, and shared-serializer inference across every read path. | 09-I:L1238-L1253 | anchor=I45-E004–I45-E007, I45-E019–I45-E020, I45-I001
- **I-004/SD-I45-002** [SD] (DEEP-I-004.discovery.md:L648| unit=I-45): **ADD** the full read-path list (`verify-attestation`, replay, plan status), I-39 downstream role, and hard-fail/no-warning behavior. | 09-I:L1242-L1253 | anchor=I45-E007–I45-E008, I45-E018, I45-E024–I45-E025
- **I-004/SD-I45-003** [SD] (DEEP-I-004.discovery.md:L649| unit=I-45 SURFACE): **ADD/QUALIFY** complete `ObjectiveSealed` kind list and flag `ReceiptStale` event-versus-error ambiguity. | 09-I:L1250-L1253 | anchor=I45-E017, I45-E021–I45-E023, I45-A004
- **I-004/SD-I45-004** [SD] (DEEP-I-004.discovery.md:L650| unit=I-45): **ADD** original roach-pi read-before-execution rule, shipped-substrate correction, F-4/I-18 record coordination, and no-new-verifier/gate constraint. | 09-I:L1233-L1253 | anchor=I45-E003, I45-E009, I45-E011–I45-E014
- **I-004/SD-I45-005** [SD] (DEEP-I-004.discovery.md:L651| unit=I-45): **ADD** per-requirement/cross-session deferrals and unresolved LEGACY/missing-field semantics. | 09-I:L1238-L1253 | anchor=I45-E010, I45-E015, I45-A003, I45-A005
- **I-004/SD-I45-006** [SD] (DEEP-I-004.discovery.md:L652| unit=I-45 SURFACE): **FLAG** every-PASS versus every-sealed-verdict scope and undefined canonicalization/source precedence. | 09-I:L1238-L1244 | anchor=I45-A001–I45-A002

#### I-004 / unit `RELATIONAL` (54 records)

- **I-004/IX-001** [IX] (DEEP-I-004.discovery.md:L476| unit=RELATIONAL): E | Track-12 safe parallelism is layered: I-2 isolates branch source state while I-3 makes the shared audit/checkpoint substrate durable under later search load. | anchor=I2-E002–I2-E012; I3-E002–I3-E007
- **I-004/IX-002** [IX] (DEEP-I-004.discovery.md:L477| unit=RELATIONAL): E | I-4 parallel search is directly blocked on I-2 workspace isolation and later consumes I-3’s high-rate append path. | anchor=I2-E029; I3-E018, I3-E026
- **I-004/IX-003** [IX] (DEEP-I-004.discovery.md:L478| unit=RELATIONAL): E | I-1 Supervisor is both I-2’s spawn/escape enforcement plane and I-35’s sequence-gate plane; I-3 precedes it so Supervisor rows land durably. | anchor=I2-E004, I2-E013; I3-E026; I35-E015, I35-E019, I35-E023
- **I-004/IX-004** [IX] (DEEP-I-004.discovery.md:L479| unit=RELATIONAL): E | I-2 conflict recovery composes BSP checkpoint recovery and H-49 trunk restore; it owns neither checkpoint nor rollback implementation. | anchor=I2-E008–I2-E010, I2-E019, I2-E022–I2-E023
- **I-004/IX-005** [IX] (DEEP-I-004.discovery.md:L480| unit=RELATIONAL SURFACE): E | I-2 backend state is surfaced by extending C-23 BranchTree, while the underlying sandbox/pool implementation composes C-11/E-15. | anchor=I2-E015, I2-E021
- **I-004/IX-006** [IX] (DEEP-I-004.discovery.md:L481| unit=RELATIONAL): E | I-2’s deterministic sequence merge is the Tier-1 policy; richer cross-branch CRDT semantics are explicitly handed to I-29. | anchor=I2-E008–I2-E012, I2-E028
- **I-004/IX-007** [IX] (DEEP-I-004.discovery.md:L482| unit=RELATIONAL SURFACE): E | I-3 intentionally produces a TOML rather than UI; I-27 is named as the later cockpit consumer while separately sourcing compute telemetry from I-4. | anchor=I3-E006–I3-E007, I3-E020, I3-I001; I27-E011, I27-E014
- **I-004/IX-008** [IX] (DEEP-I-004.discovery.md:L483| unit=RELATIONAL SURFACE): E | I-12 is a gated extension chain: receipt count/lift -> C-28 mutation validation -> existing Phase-3F lifecycle -> RulesDrawer/export, with F-5 demotion and IS-16 freeze controls. | anchor=I12-E002–I12-E016, I12-I001
- **I-004/IX-009** [IX] (DEEP-I-004.discovery.md:L484| unit=RELATIONAL): E | I-12 cross-corpus transfer is blocked on I-38’s baseline; I-27 already consumes I-38’s paired-seed/bootstrap floor for a different measurement. | anchor=I12-E024; I27-E005, I27-E011
- **I-004/IX-010** [IX] (DEEP-I-004.discovery.md:L485| unit=RELATIONAL): E | I-21’s execution path composes H-37 approval, I-1 context isolation, I-17 deterministic checks, H-38 stop, and D-13 counterexample escalation in that order. | anchor=I21-E007–I21-E016, I21-E019–I21-E023
- **I-004/IX-011** [IX] (DEEP-I-004.discovery.md:L486| unit=RELATIONAL): E | I-21’s bounded retry is structurally distinct from I-3 durability recovery and I-2 rollback: it reinjects a failing verifier trace into generation rather than repeating I/O or restoring a branch. | anchor=I21-E012–I21-E016; I2-E010; I3-E005
- **I-004/IX-012** [IX] (DEEP-I-004.discovery.md:L487| unit=RELATIONAL): E | I-27 imports pass/cost definitions from E-49, runner shape from E-9, I-4 node telemetry, I-38 statistics, and H-9 display; none may be redefined locally. | anchor=I27-E011, I27-E024
- **I-004/IX-013** [IX] (DEEP-I-004.discovery.md:L488| unit=RELATIONAL): E | I-34’s runtime chain deliberately separates XGrammar token masking, Rust/provider CRANE control, mandatory post-parse validation, and D-16 refusal. | anchor=I34-E004–I34-E006, I34-E010–I34-E013, I34-E025, I34-E030–I34-E031
- **I-004/IX-014** [IX] (DEEP-I-004.discovery.md:L489| unit=RELATIONAL): E | I-34’s amendment aligns grammar spec/pin hashing with the RRA stack so replay can detect plan/execution grammar drift. | anchor=I34-E027–I34-E030, I34-E033
- **I-004/IX-015** [IX] (DEEP-I-004.discovery.md:L490| unit=RELATIONAL): E | C-37 consumes I-34’s CRANE runtime; the Python-sidecar CRANE mask work and cross-language pack remain separately deferred. | anchor=I34-E006, I34-E018–I34-E019
- **I-004/IX-016** [IX] (DEEP-I-004.discovery.md:L491| unit=RELATIONAL): E | I-35 is the temporal dimension layered after Phase-3A masks, E-30 argv checks, and F-3 committed sets, immediately before Supervisor execution. | anchor=I35-E002, I35-E013, I35-E019
- **I-004/IX-017** [IX] (DEEP-I-004.discovery.md:L492| unit=RELATIONAL): E | I-35 consumes IS-11 taint/provenance rows to make untrusted-ingest ordering enforceable at action history rather than token text. | anchor=I35-E005, I35-E011, I35-E014
- **I-004/IX-018** [IX] (DEEP-I-004.discovery.md:L493| unit=RELATIONAL): E | I-40’s spec and conformance checker depend on I-18’s chain format, while external Rekor publication is explicitly postponed until I-18 ships. | anchor=I40-E004–I40-E005, I40-E017, I40-E023, I40-A001
- **I-004/IX-019** [IX] (DEEP-I-004.discovery.md:L494| unit=RELATIONAL): E | I-40’s wrapped run must invoke G-9 redaction after trust processing and before bundle export; redaction is the publication/privacy precondition, not an optional display filter. | anchor=I40-E007–I40-E010, I40-E025–I40-E027
- **I-004/IX-020** [IX] (DEEP-I-004.discovery.md:L495| unit=RELATIONAL): E | I-42 consumes I-41 only for advisory design critique and preserves the shipped deterministic spine as the ship-time authority; lenses never become a DONE verdict. | anchor=I42-E001, I42-E003, I42-E007–I42-E009, I42-E027–I42-E030
- **I-004/IX-021** [IX] (DEEP-I-004.discovery.md:L496| unit=RELATIONAL): E | I-42 explicitly pairs with I-40 without overlap: I-42 governs internal operator-written runs; I-40 wraps external-agent tool traffic. | anchor=I42-E031; I40-E002, I40-E006
- **I-004/IX-022** [IX] (DEEP-I-004.discovery.md:L497| unit=RELATIONAL): E | I-45 and I-18 modify the same RRA/attestation record: objective hash and previous-row chain layout require coordination rather than parallel independent schemas. | anchor=I45-E006, I45-E012; I40-E004, I40-E017
- **I-004/IX-023** [IX] (DEEP-I-004.discovery.md:L498| unit=RELATIONAL): E | I-45’s objective source expands forward to I-37 EARS, and I-39 is a read-path consumer required to rerun the staleness check. | anchor=I45-E004, I45-E013, I45-E015, I45-E024
- **I-004/IX-024** [IX] (DEEP-I-004.discovery.md:L499| unit=RELATIONAL): I | Once I-45’s `objective_hash` lands, I-40’s versioned open RRA spec/conformance checker will need a coordinated version/schema decision even though neither source explicitly assigns that integration. | anchor=I40-E003–I40-E005, I40-A002; I45-E006–I45-E008
- **I-004/IX-025** [IX] (DEEP-I-004.discovery.md:L500| unit=RELATIONAL): E | Seven sources add baseline-relative action types—`BranchWorkspaceEvent`, `PlaybookEvent`, `SprintContractEvent`, `ConvergenceCurveMeasured`, `GrammarGateEvent`, `ToolSequenceVerdict`, and `ObjectiveSealed`; all require edit-time registry reread rather than independently assuming the dated 221/532 baseline. | anchor=I2-E014; I12-E011; I21-E023; I27-E009; I34-E015; I35-E018; I45-E017
- **I-004/IX-026** [IX] (DEEP-I-004.discovery.md:L501| unit=RELATIONAL): E | I-3, I-40, and I-42 explicitly add no action type and reuse existing row families; their acceptance evidence depends on existing rows and, for I-42, meaningful absence. | anchor=I3-E017, I3-E024; I40-E019; I42-E011, I42-E025, I42-I001
- **I-004/IX-027** [IX] (DEEP-I-004.discovery.md:L502| unit=RELATIONAL): E | Frozen/off-agent controls recur at different layers: I-12 freezes evaluator independence, I-27 freezes the harness, I-34 freezes grammar pins, I-35 freezes rule files, and I-45 joins the frozen receipt gate set. | anchor=I12-E010; I27-E012; I34-E009; I35-E009, I35-E026; I45-E025
- **I-004/IX-028** [IX] (DEEP-I-004.discovery.md:L503| unit=RELATIONAL): E | User/new-authority gates remain isolated: I-2 cloud adapter, I-12 auto-update, I-34 new Cargo dependency, and I-40 real MCP matrix each require explicit approval rather than silently entering Tier 1. | anchor=I2-E007; I12-E025; I34-E027, I34-E035; I40-E021
- **I-004/IX-029** [IX] (DEEP-I-004.discovery.md:L504| unit=RELATIONAL): E | All new hashing in this chunk uses blake3; I-2/I-3/I-12/I-34/I-35/I-40/I-42/I-45 explicitly ban blake2/sha2, and I-27 does so for its production bench path. | anchor=I2-E027; I3-E022; I12-E023; I27-E023; I34-E022; I35-E024; I40-E025; I42-E027; I45-E005, I45-E020
- **I-004/IX-030** [IX] (DEEP-I-004.discovery.md:L505| unit=RELATIONAL): I | The seven simultaneous action-family additions plus I-18/I-45 shared RRA fields make registry/schema edits a concurrency hotspot; serialization and golden-lock conflicts must be resolved from freshly reread live state. | anchor=IX-022, IX-025; I3-E011–I3-E012, I3-E017
- **I-004/AC-001** [CON] (DEEP-I-004.discovery.md:L511| unit=RELATIONAL): conflict | I-2’s opening platform description sounds immediate, but revised scope controls: worktrees are Tier 1 and Linux overlay/reflink is Tier 2. | anchor=I2-C001
- **I-004/AC-002** [AC] (DEEP-I-004.discovery.md:L512| unit=RELATIONAL): ambiguity | I-2 does not fully define “last-seq wins” for non-overlapping patches or canonical patch application. | anchor=I2-A001
- **I-004/AC-003** [AC] (DEEP-I-004.discovery.md:L513| unit=RELATIONAL SURFACE): ambiguity/security | I-2’s absolute-path escape rule omits symlink/junction/canonicalization cases and its conflict badge is not mapped to UI. | anchor=I2-A002, I2-A004
- **I-004/AC-004** [AC] (DEEP-I-004.discovery.md:L514| unit=RELATIONAL): dependency | I-2 requires rollback while its H-49 restore owner is only PLAN-asserted planned; pre-H-49 fallback is missing. | anchor=I2-A003
- **I-004/AC-005** [AC] (DEEP-I-004.discovery.md:L515| unit=RELATIONAL): qualification | I-3 addresses a concurrency-motivated write path but remains single-writer in Tier 1; multi-writer behavior is deferred. | anchor=I3-C001
- **I-004/AC-006** [AC] (DEEP-I-004.discovery.md:L516| unit=RELATIONAL): ambiguity/config | I-3 leaves pragma values, remaining checkpoint sites, benchmark workload, and acceptable performance largely unspecified. | anchor=I3-A001–I3-A003
- **I-004/AC-007** [AC] (DEEP-I-004.discovery.md:L517| unit=RELATIONAL): ambiguity/recovery | I-3 cites POSIX rename without defining non-POSIX replacement, directory fsync, cleanup, or rename retry. | anchor=I3-A004
- **I-004/AC-008** [AC] (DEEP-I-004.discovery.md:L518| unit=RELATIONAL): ambiguity/measurement | I-12 does not fix `k`, schedule interval, or statistically valid lift method. | anchor=I12-A001–I12-A003
- **I-004/AC-009** [AC] (DEEP-I-004.discovery.md:L519| unit=RELATIONAL): ambiguity/data | I-12 names three export formats without specifying their block schemas or collision/update semantics. | anchor=I12-A004
- **I-004/AC-010** [AC] (DEEP-I-004.discovery.md:L520| unit=RELATIONAL): ambiguity/schema | I-21 uses both `bounded_iteration_count` and `iteration_count`, and its deterministic-criterion reference grammar is undefined. | anchor=I21-A001–I21-A002
- **I-004/AC-011** [AC] (DEEP-I-004.discovery.md:L521| unit=RELATIONAL): ambiguity/dependency | I-21 permits an unspecified temporary I-1 isolation shim and leaves replace-versus-supplement outer retry unresolved. | anchor=I21-A003–I21-A004
- **I-004/AC-012** [AC] (DEEP-I-004.discovery.md:L522| unit=RELATIONAL): ambiguity/state | I-21 does not fully define the final third-failure event/stop ordering. | anchor=I21-A005
- **I-004/AC-013** [AC] (DEEP-I-004.discovery.md:L523| unit=RELATIONAL): ambiguity/measurement | I-27 leaves compute stepping, bootstrap confidence, knee logic, cost normalization, and report-date determinism unresolved. | anchor=I27-A001–I27-A004
- **I-004/AC-014** [CON] (DEEP-I-004.discovery.md:L524| unit=RELATIONAL): conflict/scope | I-34’s reasoning-heavy CRANE requirement must be reconciled with the amendment’s HD-3 ban on constrained decoding for reasoning/coder/planner roles. | anchor=I34-C001
- **I-004/AC-015** [AC] (DEEP-I-004.discovery.md:L525| unit=RELATIONAL): ambiguity/implementation | I-34 leaves module ownership, commit-point transform, manifest schema, untrusted-source test, and fixture statistics unresolved. | anchor=I34-A001–I34-A005
- **I-004/AC-016** [AC] (DEEP-I-004.discovery.md:L526| unit=RELATIONAL): dependency | I-34’s named llguidance version is only a dated snapshot; user approval and current anti-slopsquat verification control. | anchor=I34-A006, I34-E035–I34-E036
- **I-004/AC-017** [AC] (DEEP-I-004.discovery.md:L527| unit=RELATIONAL): ambiguity/workflow | I-35 defines confirmation/substitution but supplies Tier-1 evidence only for allow/halt; its TOML DSL, class registry, and session-history semantics remain open. | anchor=I35-A001–I35-A004
- **I-004/AC-018** [AC] (DEEP-I-004.discovery.md:L528| unit=RELATIONAL): evidence | I-35’s absolute CDA-immunity claim has no corresponding adversarial provenance-forgery test in this plan. | anchor=I35-A005
- **I-004/AC-019** [AC] (DEEP-I-004.discovery.md:L529| unit=RELATIONAL): dependency/schema | I-40 documents I-18’s chain before its layout is final and omits normative serialization/version compatibility details. | anchor=I40-A001–I40-A002
- **I-004/AC-020** [AC] (DEEP-I-004.discovery.md:L530| unit=RELATIONAL): ambiguity/security/privacy | I-40 does not define proxy bypass coverage or redaction-stage failure semantics; its viewer pairing is also unverified. | anchor=I40-A003–I40-A005
- **I-004/AC-021** [AC] (DEEP-I-004.discovery.md:L531| unit=RELATIONAL): ambiguity/permissions | I-42 narrows its executable invariant to executor/fixer writes although prose says all spawned agents only critique; actor attribution and exact gate site are assumed/unresolved. | anchor=I42-I001, I42-A001–I42-A002, I42-A004
- **I-004/AC-022** [AC] (DEEP-I-004.discovery.md:L532| unit=RELATIONAL): ambiguity/determinism | I-42 requires Orchestrate-only writes and identical verdict bytes but does not specify an artifact-equivalent fixture; operator edit ingestion is also undefined. | anchor=I42-A003, I42-A005
- **I-004/AC-023** [AC] (DEEP-I-004.discovery.md:L533| unit=RELATIONAL): ambiguity/scope | I-45 alternates between every PASS and every sealed verdict, and its event/error identity plus LEGACY success semantics are unresolved. | anchor=I45-A001, I45-A003–I45-A005
- **I-004/AC-024** [AC] (DEEP-I-004.discovery.md:L534| unit=RELATIONAL): ambiguity/determinism | I-45 requires canonical bytes across multiple sources/consumers but omits canonical field/Unicode/precedence/version rules. | anchor=I45-I001, I45-A002

### Ledger I-005 — I-6, I-7, I-11, I-14, I-22, I-25, I-31, I-39, I-44, I-51

Source: `docs/thefinalglance/feature-discovery/deep/DEEP-I-005.discovery.md`
Extracted atoms: **380**

#### I-005 / unit `I-6` (27 records)

- **I-005/DII005-I6-001** [SRC] (DEEP-I-005.discovery.md:L38| unit=I-6): `[E][status]` I-6 is Track 13, lever `STEERING`, `STRONG`, `BUILD`, effort S, Wave 2. — I6:L1-L3
- **I-005/DII005-I6-002** [SRC] (DEEP-I-005.discovery.md:L39| unit=I-6): `[E][problem]` Long contexts degrade effective attention to the task specification as context fills. — I6:L5-L9
- **I-005/DII005-I6-003** [SRC] (DEEP-I-005.discovery.md:L40| unit=I-6): `[E][capability]` Add the tenth enum variant with the exact identifier `context_budget.rs::ContextSlot::AnchorPane`; it is an immutable privileged-last slot. — I6:L12-L18, I6:L25-L27, I6:L38-L40, I6:L59-L61
- **I-005/DII005-I6-004** [SRC] (DEEP-I-005.discovery.md:L41| unit=I-6): `[E][layout/invariant]` Append `AnchorPane` after the final cache breakpoint, without displacing or rebalancing the existing nine slots. — I6:L12-L20, I6:L27-L30, I6:L73-L75
- **I-005/DII005-I6-005** [SRC] (DEEP-I-005.discovery.md:L42| unit=I-6): `[E][data]` The anchor serializes the original `TaskSpec` bytes and keeps those bytes frozen. — I6:L13-L15, I6:L39-L40
- **I-005/DII005-I6-006** [SRC] (DEEP-I-005.discovery.md:L43| unit=I-6): `[E][data]` The anchor serializes unresolved claims originating in I-17/I-20 verification passes. — I6:L14-L15, I6:L39-L40
- **I-005/DII005-I6-007** [SRC] (DEEP-I-005.discovery.md:L44| unit=I-6): `[E][data]` The anchor serializes the active constraint list. — I6:L14-L16, I6:L39-L40
- **I-005/DII005-I6-008** [SRC] (DEEP-I-005.discovery.md:L45| unit=I-6): `[E][generation]` The reply must open with a short acknowledgement tag reflecting the highest-priority active constraint. — I6:L15-L16
- **I-005/DII005-I6-009** [SRC] (DEEP-I-005.discovery.md:L46| unit=I-6 SURFACE): `[E][enforcement]` The I-1 supervisor validates tag presence in the first reply-token window. — I6:L16, I6:L33-L34, I6:L41-L42, I6:L65
- **I-005/DII005-I6-010** [SRC] (DEEP-I-005.discovery.md:L47| unit=I-6): `[E][failure/retry]` A missing acknowledgement tag causes a retry prompt with explicit clarification; it is logged and is not a hard error. — I6:L41-L42
- **I-005/DII005-I6-011** [SRC] (DEEP-I-005.discovery.md:L48| unit=I-6): `[E][schema]` Anchor presence reuses the existing `ContextReduction` payload family; I-6 adds no SSB `action_type`. — I6:L21-L22, I6:L43, I6:L70-L72, I6:L81-L83, I6:L94-L95, I6:L102-L103
- **I-005/DII005-I6-012** [SRC] (DEEP-I-005.discovery.md:L49| unit=I-6): `[E][integration]` Extend `context_budget.rs::assemble_context` with the anchor serialization branch, following H-31's lazy-ALTER/slot-add pattern. — I6:L31-L32, I6:L50-L53, I6:L59-L64
- **I-005/DII005-I6-013** [SRC] (DEEP-I-005.discovery.md:L50| unit=I-6): `[E][determinism/cache]` `context_budget.rs::update_breakpoints` must retain its `bp.valid`/blake3 prefix-stability invariant; any cache bust is a test failure. — I6:L25-L30, I6:L63, I6:L96-L97
- **I-005/DII005-I6-014** [SRC] (DEEP-I-005.discovery.md:L51| unit=I-6): `[E][reachability]` Production reachability is `task_runner.rs` context assembly → `assemble_context` → `ContextSlot::AnchorPane` serialization; test-only callers do not establish R7 reachability. — I6:L67-L69
- **I-005/DII005-I6-015** [SRC] (DEEP-I-005.discovery.md:L52| unit=I-6 SURFACE): `[E][surface]` I-6 creates no dashboard surface; the anchor is prompt-internal and is measured through fixtures. — I6:L54-L56, I6:L77-L78
- **I-005/DII005-I6-016** [SRC] (DEEP-I-005.discovery.md:L53| unit=I-6): `[E][scope]` The implementation is a targeted enum extension and assembly hook: no new crate, module, or context serialization layer. — I6:L18-L21, I6:L50-L53
- **I-005/DII005-I6-017** [SRC] (DEEP-I-005.discovery.md:L54| unit=I-6): `[E][constraint]` A prompt instruction such as “remember the task” is not an acceptable substitute for the structural slot. — I6:L19-L20, I6:L74-L75
- **I-005/DII005-I6-018** [SRC] (DEEP-I-005.discovery.md:L55| unit=I-6): `[E][deferred]` Adaptive constraint prioritization is Tier 2 and gated on at least three measured adherence runs showing the fixed list insufficient. — I6:L45-L47
- **I-005/DII005-I6-019** [SRC] (DEEP-I-005.discovery.md:L56| unit=I-6 SURFACE): `[E][deferred]` The constraint-adherence cockpit tile is Tier 2 under the same GD-4 bound gate. — I6:L48
- **I-005/DII005-I6-020** [SRC] (DEEP-I-005.discovery.md:L57| unit=I-6): `[E][artifact]` Ship `workflow/reasoning/I-6-decisions.md`. — I6:L44
- **I-005/DII005-I6-021** [SRC] (DEEP-I-005.discovery.md:L58| unit=I-6): `[E][acceptance]` The long-context trigger is `cargo run --bin elai -- run --task fixtures/search/long_context_25k.toml`. — I6:L79-L80
- **I-005/DII005-I6-022** [SRC] (DEEP-I-005.discovery.md:L59| unit=I-6): `[E][acceptance/evidence]` The journaled assembled request must show `AnchorPane` as the final block (`anchor_pane` is the stated byte-search marker), with `bp.valid` passing. — I6:L80-L83
- **I-005/DII005-I6-023** [SRC] (DEEP-I-005.discovery.md:L60| unit=I-6 SURFACE): `[E][acceptance/evidence]` The response journal must show the acknowledgement tag in its first token window. — I6:L84-L85
- **I-005/DII005-I6-024** [SRC] (DEEP-I-005.discovery.md:L61| unit=I-6): `[E][measurement]` Constraint adherence is measured—not asserted—against a `--no-anchor` control under the local-baseline-before-paper-threshold rule. — I6:L84-L87, I6:L99-L101
- **I-005/DII005-I6-025** [SRC] (DEEP-I-005.discovery.md:L62| unit=I-6): `[E][negative test]` The `--no-anchor` broken arm must open without the tag and fail the tag-presence check. — I6:L85-L87
- **I-005/DII005-I6-026** [SRC] (DEEP-I-005.discovery.md:L63| unit=I-6): `[E][dependency]` I-6 composes H-2 breakpoint discipline, H-31 slot-add precedent, and I-1 supervisor enforcement rather than reimplementing them. — I6:L29-L34, I6:L105-L110
- **I-005/DII005-I6-027** [SRC] (DEEP-I-005.discovery.md:L64| unit=I-6): `[E][safety]` Production paths ban `.unwrap()`/`.expect()`, blake2/sha2, and fabricated measurements. — I6:L76

#### I-005 / unit `I-7` (29 records)

- **I-005/DII005-I7-001** [SRC] (DEEP-I-005.discovery.md:L68| unit=I-7): `[E][status/dependency]` I-7 is Track 13, `CHEAPER-SPEED`, `STRONG`, `BUILD`, effort M, Wave 4; it depends on I-11 and binds IC-1. — I7:L1-L3
- **I-005/DII005-I7-002** [SD] (DEEP-I-005.discovery.md:L69| unit=I-7): `[E][correction]` The former speculative Batch-API racing design is dead: live-loop batch jobs take 10–45 minutes with a 24-hour SLA and cancellation does not refund queued work. — I7:L5-L11, I7:L13-L15, I7:L103-L107
- **I-005/DII005-I7-003** [SRC] (DEEP-I-005.discovery.md:L70| unit=I-7): `[E][workflow]` The local lane first fans the task out to K branches in I-2 isolated workspaces. — I7:L15-L16, I7:L72
- **I-005/DII005-I7-004** [SRC] (DEEP-I-005.discovery.md:L71| unit=I-7): `[E][workflow]` `I-17 TriLayerStack::evaluate` runs deterministic checks on every local branch. — I7:L16, I7:L73
- **I-005/DII005-I7-005** [SRC] (DEEP-I-005.discovery.md:L72| unit=I-7): `[E][state transition]` Escalation begins only when all local branches fail. — I7:L16-L17
- **I-005/DII005-I7-006** [SRC] (DEEP-I-005.discovery.md:L73| unit=I-7): `[E][data]` The failure set is condensed into I-11 `BeliefState` fields `tried`, `failed-because`, and `constraints`, targeting no more than about 2K tokens. — I7:L17-L21, I7:L59-L60, I7:L84-L85
- **I-005/DII005-I7-007** [SRC] (DEEP-I-005.discovery.md:L74| unit=I-7): `[E][routing]` The condensed payload escalates synchronously to the H-7 cloud architect. — I7:L17-L20, I7:L31-L32
- **I-005/DII005-I7-008** [SRC] (DEEP-I-005.discovery.md:L75| unit=I-7): `[E][workflow]` The cloud architect response becomes the next task-runner candidate. — I7:L18-L19
- **I-005/DII005-I7-009** [SRC] (DEEP-I-005.discovery.md:L76| unit=I-7): `[E][privacy/cost]` A local solution never leaves the machine and must incur zero cloud spend through the E-39 cost gate. — I7:L20-L22, I7:L37, I7:L71
- **I-005/DII005-I7-010** [SRC] (DEEP-I-005.discovery.md:L77| unit=I-7): `[E][offline integration]` H-5 retains the Batch API only for offline Cosmos/research/bench lanes; I-7 does not reimplement its client or scheduler. — I7:L10-L11, I7:L19-L20, I7:L33-L34, I7:L83, I7:L115-L117
- **I-005/DII005-I7-011** [SRC] (DEEP-I-005.discovery.md:L78| unit=I-7): `[E][security/architecture]` Live `escalation_waterfall.rs` must have no import path to the batch client, making batch routing structurally unreachable from interactive/search-loop execution. — I7:L45-L47, I7:L77-L80
- **I-005/DII005-I7-012** [SRC] (DEEP-I-005.discovery.md:L79| unit=I-7): `[E][capability]` Create `crates/orchestrator/src/escalation_waterfall.rs::EscalationWaterfall::run` as the local-first dispatcher. — I7:L42-L43, I7:L56-L57, I7:L65-L67
- **I-005/DII005-I7-013** [SRC] (DEEP-I-005.discovery.md:L80| unit=I-7): `[E][schema/events]` Add `EscalationCascade` to `ssb.rs` and `action_type_lock.rs`, baseline +1, with `LocalWin`, `Condensed`, `Escalated`, and `BatchRouted` kinds. — I7:L38, I7:L43-L44, I7:L65-L67, I7:L112-L113
- **I-005/DII005-I7-014** [SRC] (DEEP-I-005.discovery.md:L81| unit=I-7): `[E][event constraint]` `BatchRouted` exists only for offline-path journal completeness and never on a live search loop. — I7:L78-L80
- **I-005/DII005-I7-015** [SRC] (DEEP-I-005.discovery.md:L82| unit=I-7): `[E][integration]` Wire the waterfall at `cloud_mode.rs` / `escalation_policy.rs` and call I-11 `BeliefState::condense`. — I7:L48-L50, I7:L56-L60, I7:L68-L71
- **I-005/DII005-I7-016** [SRC] (DEEP-I-005.discovery.md:L83| unit=I-7): `[E][reachability]` Production reachability is task-runner dispatch → `EscalationWaterfall::run`; test callers do not satisfy R7. — I7:L74-L76
- **I-005/DII005-I7-017** [SRC] (DEEP-I-005.discovery.md:L84| unit=I-7): `[E][constraint]` Raw history is forbidden in cloud escalation; the condensed `BeliefState` is mandatory. — I7:L84-L85
- **I-005/DII005-I7-018** [SRC] (DEEP-I-005.discovery.md:L85| unit=I-7): `[E][constraint]` Async/speculative local+cloud submission that relies on cancellation refunds is forbidden. — I7:L22-L25, I7:L81-L82
- **I-005/DII005-I7-019** [SRC] (DEEP-I-005.discovery.md:L86| unit=I-7): `[E][constraint]` Heartbeat-primary routing is excluded from this feature. — I7:L22-L24
- **I-005/DII005-I7-020** [SRC] (DEEP-I-005.discovery.md:L87| unit=I-7): `[E][deferred]` Multi-hop `local → cloud-small → cloud-large` escalation is Tier 2, gated on at least three runs showing single-hop insufficiency. — I7:L51-L53
- **I-005/DII005-I7-021** [SRC] (DEEP-I-005.discovery.md:L88| unit=I-7 SURFACE): `[E][deferred/UI]` The cost-cockpit cascade-trace tile is deferred until the GD-4 bound is measured; `cross_lang_diff.py` becomes required when it ships. — I7:L54, I7:L61-L62, I7:L87-L90
- **I-005/DII005-I7-022** [SRC] (DEEP-I-005.discovery.md:L89| unit=I-7): `[E][artifact]` Ship `workflow/reasoning/I-7-decisions.md`. — I7:L50
- **I-005/DII005-I7-023** [SRC] (DEEP-I-005.discovery.md:L90| unit=I-7): `[E][acceptance]` Run paired fixtures `fixtures/search/local_solvable.toml` and `fixtures/search/local_fail.toml` with `--waterfall`. — I7:L91-L93
- **I-005/DII005-I7-024** [SRC] (DEEP-I-005.discovery.md:L91| unit=I-7): `[E][acceptance/evidence]` The solvable fixture emits `EscalationCascade(LocalWin)` and journals zero cloud API calls. — I7:L94-L95
- **I-005/DII005-I7-025** [SRC] (DEEP-I-005.discovery.md:L92| unit=I-7): `[E][acceptance/evidence]` The failing fixture emits `Condensed` followed by `Escalated`, and the journal proves a typed payload of at most 2K tokens. — I7:L95-L97
- **I-005/DII005-I7-026** [SRC] (DEEP-I-005.discovery.md:L93| unit=I-7): `[E][negative test]` `rg 'batch_client\|MessageBatch' crates/orchestrator/src/escalation_waterfall.rs` must return zero. — I7:L98-L100
- **I-005/DII005-I7-027** [SRC] (DEEP-I-005.discovery.md:L94| unit=I-7): `[E][negative test]` An injected live-waterfall call to the batch client must fail to compile. — I7:L100-L101
- **I-005/DII005-I7-028** [SRC] (DEEP-I-005.discovery.md:L95| unit=I-7): `[E][status/evidence]` Existing `batches` tokens are confirmed only at file level; their wiring depth remains unverified and must be retraced before implementation. — I7:L27-L30
- **I-005/DII005-I7-029** [SRC] (DEEP-I-005.discovery.md:L96| unit=I-7): `[E][safety]` Production paths ban `.unwrap()`/`.expect()`, blake2/sha2, and fabricated measurements. — I7:L86

#### I-005 / unit `I-11` (32 records)

- **I-005/DII005-I11-001** [SRC] (DEEP-I-005.discovery.md:L100| unit=I-11): `[E][status]` I-11 is Track 14, `CHEAPER-ORCH`, `STRONG`, `BUILD`, effort M, Wave 2, bound by ID-3 and IC-8. — I11:L1-L3
- **I-005/DII005-I11-002** [SRC] (DEEP-I-005.discovery.md:L101| unit=I-11): `[E][problem]` Existing cross-model/tier handoffs send raw history, increasing cloud payloads and hiding condensation cost. — I11:L4-L7
- **I-005/DII005-I11-003** [SRC] (DEEP-I-005.discovery.md:L102| unit=I-11): `[E][schema]` Create exact symbol `crates/orchestrator/src/belief_state.rs::BeliefState`, typed and versioned. — I11:L13-L14, I11:L34, I11:L45, I11:L50-L53
- **I-005/DII005-I11-004** [SRC] (DEEP-I-005.discovery.md:L103| unit=I-11): `[E][schema field]` `BeliefState` carries what was tried. — I11:L13
- **I-005/DII005-I11-005** [SRC] (DEEP-I-005.discovery.md:L104| unit=I-11): `[E][schema field]` `BeliefState` carries why attempts failed. — I11:L13
- **I-005/DII005-I11-006** [SRC] (DEEP-I-005.discovery.md:L105| unit=I-11): `[E][schema field]` `BeliefState` carries current AST state. — I11:L13
- **I-005/DII005-I11-007** [SRC] (DEEP-I-005.discovery.md:L106| unit=I-11): `[E][schema field]` `BeliefState` carries active constraints. — I11:L13
- **I-005/DII005-I11-008** [SRC] (DEEP-I-005.discovery.md:L107| unit=I-11): `[E][schema field]` `BeliefState` carries DKG-keyed entity links. — I11:L11-L14, I11:L51
- **I-005/DII005-I11-009** [SRC] (DEEP-I-005.discovery.md:L108| unit=I-11): `[E][failure]` Decode on schema-version mismatch fails closed with typed `BeliefStateVersionMismatch`, never a silent parse. — I11:L13-L14, I11:L34, I11:L66, I11:L74-L75
- **I-005/DII005-I11-010** [SRC] (DEEP-I-005.discovery.md:L109| unit=I-11): `[E][workflow]` Handoff sites are I-7 local-to-cloud escalation, H-7 architect-to-editor routing, and H-51 session resume. — I11:L15-L16
- **I-005/DII005-I11-011** [SRC] (DEEP-I-005.discovery.md:L110| unit=I-11): `[E][privacy/context]` The escalation payload becomes prefix + `BeliefState`, never raw history; this removes raw history from escalation, not from session storage. — I11:L21-L22, I11:L37, I11:L64
- **I-005/DII005-I11-012** [SRC] (DEEP-I-005.discovery.md:L111| unit=I-11): `[E][events]` Add `BeliefStateEvent` to `ssb.rs`/`action_type_lock.rs`, baseline +1, with `Condensed`, `Handoff`, and `HonestyDelta` kinds. — I11:L17-L18, I11:L35-L36, I11:L53, I11:L82
- **I-005/DII005-I11-013** [SRC] (DEEP-I-005.discovery.md:L112| unit=I-11): `[E][observability]` Every condensation emits an ID-3 receipt carrying `schema_version` and `token_delta`. — I11:L17-L18, I11:L30, I11:L35, I11:L63-L65
- **I-005/DII005-I11-014** [SRC] (DEEP-I-005.discovery.md:L113| unit=I-11): `[E][replay]` Journal evidence must reconstruct exactly what the escalated model received. — I11:L17-L18
- **I-005/DII005-I11-015** [SRC] (DEEP-I-005.discovery.md:L114| unit=I-11): `[E][capability]` Create `crates/orchestrator/src/belief_state.rs::HonestyJob` for periodic full-history-versus-BeliefState reruns and delta ledgering. — I11:L36, I11:L45, I11:L52
- **I-005/DII005-I11-016** [SRC] (DEEP-I-005.discovery.md:L115| unit=I-11): `[E][config/default]` The honesty interval is configurable and defaults to after every 10 escalations. — I11:L36
- **I-005/DII005-I11-017** [SD] (DEEP-I-005.discovery.md:L116| unit=I-11): `[E][determinism]` Honesty deltas must be recomputable under `ELAI_SEED`. — I11:L36
- **I-005/DII005-I11-018** [SRC] (DEEP-I-005.discovery.md:L117| unit=I-11): `[E][integration]` Reuse `context_budget.rs::adaptive_reduce` as the reduction engine whose result the typed state represents. — I11:L26, I11:L46, I11:L56
- **I-005/DII005-I11-019** [SRC] (DEEP-I-005.discovery.md:L118| unit=I-11): `[E][integration]` Share H-51's digest engine while keeping I-11 live-handoff scope distinct from H-51 sleep-time scope. — I11:L16, I11:L19, I11:L28, I11:L57, I11:L84-L87
- **I-005/DII005-I11-020** [SRC] (DEEP-I-005.discovery.md:L119| unit=I-11): `[E][recovery]` The receiver uses `H-6::recover_stubs` for detail re-expansion; I-11 does not reimplement it. — I11:L20, I11:L58, I11:L87
- **I-005/DII005-I11-021** [SRC] (DEEP-I-005.discovery.md:L120| unit=I-11 SURFACE): `[E][surface]` Add a required `BeliefState` inspector row in `SessionsDrawer`; `HonestyDelta` is a user-visible economics signal. — I11:L38, I11:L47, I11:L70-L71
- **I-005/DII005-I11-022** [SRC] (DEEP-I-005.discovery.md:L121| unit=I-11): `[E][reachability]` Production encode reachability is `escalation_policy.rs::condense_and_escalate()` → `BeliefState::encode()`. — I11:L59-L60
- **I-005/DII005-I11-023** [SRC] (DEEP-I-005.discovery.md:L122| unit=I-11): `[E][reachability]` Production decode reachability is `session_dag.rs::resume()` → `BeliefState::decode()`; test-only callers do not establish R7. — I11:L61-L62
- **I-005/DII005-I11-024** [SRC] (DEEP-I-005.discovery.md:L123| unit=I-11): `[E][deferred]` Automated staleness detection for DKG links is deferred until live-graph update cadence exists. — I11:L40-L41
- **I-005/DII005-I11-025** [SRC] (DEEP-I-005.discovery.md:L124| unit=I-11): `[E][deferred]` Schema migration tooling beyond fail-closed mismatch is deferred until a v2 schema exists. — I11:L42
- **I-005/DII005-I11-026** [SRC] (DEEP-I-005.discovery.md:L125| unit=I-11): `[E][acceptance/evidence]` A local-fail run emits `Condensed` with `schema_version: 1`/token delta and `Handoff` on dispatch; the honesty job emits after at least 10 escalations or `--force-honesty-check`. — I11:L72-L75
- **I-005/DII005-I11-027** [SRC] (DEEP-I-005.discovery.md:L126| unit=I-11): `[E][measurement]` The journal must show an approximately ≤2K-token escalation payload versus a ≥20K raw-history control. — I11:L75
- **I-005/DII005-I11-028** [SRC] (DEEP-I-005.discovery.md:L127| unit=I-11): `[E][negative test]` A version-bumped blob must fail before any model call. — I11:L75
- **I-005/DII005-I11-029** [SRC] (DEEP-I-005.discovery.md:L128| unit=I-11): `[E][negative test]` A raw-history bypass produces a payload over 10K tokens and fails the payload-size gate. — I11:L75
- **I-005/DII005-I11-030** [SRC] (DEEP-I-005.discovery.md:L129| unit=I-11): `[E][dependency]` DKG must be reachable at decode time. — I11:L88
- **I-005/DII005-I11-031** [SRC] (DEEP-I-005.discovery.md:L130| unit=I-11): `[E][safety]` Production decode bans `.unwrap()`/`.expect()` and uses blake3, not blake2/sha2. — I11:L68-L69
- **I-005/DII005-I11-032** [SRC] (DEEP-I-005.discovery.md:L131| unit=I-11): `[E][status/evidence]` The typed module is confirmed absent, so this is a net-new build; routing/digest seams remain plan-asserted pending re-grep. — I11:L24-L30, I11:L77-L82

#### I-005 / unit `I-14` (27 records)

- **I-005/DII005-I14-001** [SRC] (DEEP-I-005.discovery.md:L135| unit=I-14): `[E][status]` I-14 is Track 14, `CHEAPER`, `STRONG`, `BUILD`, effort M, Wave 2, bound by ID-3 and IC-2/IC-3. — I14:L1-L3
- **I-005/DII005-I14-002** [SRC] (DEEP-I-005.discovery.md:L136| unit=I-14): `[E][problem]` No middleware prevents full compiler panics, tracebacks, or AST dumps from entering context and destabilizing the cache prefix. — I14:L4-L8, I14:L27-L28
- **I-005/DII005-I14-003** [SRC] (DEEP-I-005.discovery.md:L137| unit=I-14): `[E][capability]` Create `crates/orchestrator/src/context_transform.rs::ContextTransformer` with deterministic Tier-1 and gated Tier-2 pipeline semantics. — I14:L35-L41, I14:L48-L49, I14:L54-L59
- **I-005/DII005-I14-004** [SRC] (DEEP-I-005.discovery.md:L138| unit=I-14): `[E][strategy]` `LspSliceTransform` uses headless rust-analyzer and compiler locations to extract the exact failing line, AST node, or declaration. — I14:L11-L14, I14:L37, I14:L57
- **I-005/DII005-I14-005** [SRC] (DEEP-I-005.discovery.md:L139| unit=I-14): `[E][strategy]` `AstFoldTransform` uses tree-sitter to replace irrelevant function bodies with `/* elided */`. — I14:L14-L15, I14:L37, I14:L58
- **I-005/DII005-I14-006** [SRC] (DEEP-I-005.discovery.md:L140| unit=I-14): `[E][semantic invariant]` AST folding preserves signatures and types so BPE semantic priors remain intact. — I14:L14-L15, I14:L37
- **I-005/DII005-I14-007** [SRC] (DEEP-I-005.discovery.md:L141| unit=I-14): `[E][deferred strategy]` An LLMLingua-2-class encoder is gated Tier 2 for prose-only payloads. — I14:L11-L16, I14:L43-L45, I14:L71
- **I-005/DII005-I14-008** [SRC] (DEEP-I-005.discovery.md:L142| unit=I-14): `[E][resource/failure]` Tier-2 neural compression requires a VRAM-headroom check and refuses closed when headroom is insufficient. — I14:L16, I14:L44, I14:L72, I14:L82
- **I-005/DII005-I14-009** [SRC] (DEEP-I-005.discovery.md:L143| unit=I-14): `[E][dependency/security]` Adding LLMLingua-2 requires availability, license, current-version, and anti-slopsquat verification plus user approval. — I14:L44, I14:L91
- **I-005/DII005-I14-010** [SRC] (DEEP-I-005.discovery.md:L144| unit=I-14): `[E][deferred strategy]` Numeric quantization survives only as a narrow Tier-2 option for tabular payloads. — I14:L17-L19
- **I-005/DII005-I14-011** [SRC] (DEEP-I-005.discovery.md:L145| unit=I-14): `[E][constraint]` N-gram abbreviation/cipher substitution for code identifiers is unconditionally banned as “neural poison.” — I14:L17-L19, I14:L24-L25, I14:L68-L71
- **I-005/DII005-I14-012** [SRC] (DEEP-I-005.discovery.md:L146| unit=I-14): `[E][cache invariant]` Transforms operate only on volatile-tail content and must never touch the blake3-frozen prefix. — I14:L20-L21, I14:L31, I14:L63, I14:L73
- **I-005/DII005-I14-013** [SRC] (DEEP-I-005.discovery.md:L147| unit=I-14): `[E][events]` Add `ContextTransformReceipt` to `ssb.rs`/`action_type_lock.rs`, baseline +1, with `LspSlice`, `AstFold`, and `Tier2Compressed` kinds. — I14:L22-L23, I14:L38-L39, I14:L59, I14:L92
- **I-005/DII005-I14-014** [SRC] (DEEP-I-005.discovery.md:L148| unit=I-14): `[E][receipt schema]` Every transform receipt carries before/after token counts and a blake3 reverse pointer to original bytes. — I14:L22-L23, I14:L38
- **I-005/DII005-I14-015** [SRC] (DEEP-I-005.discovery.md:L149| unit=I-14): `[E][replay]` The reverse pointer must re-expand to byte-identical original content under a blake3 match. — I14:L22-L23, I14:L41, I14:L81-L82, I14:L88
- **I-005/DII005-I14-016** [SRC] (DEEP-I-005.discovery.md:L150| unit=I-14): `[E][failure]` Any transform without a receipt is banned and must be caught by a post-transform receipt-presence check. — I14:L24-L25, I14:L33, I14:L70, I14:L81-L82
- **I-005/DII005-I14-017** [SRC] (DEEP-I-005.discovery.md:L151| unit=I-14): `[E][integration]` Hook after tool-result finalization and before `context_budget.rs::adaptive_reduce` at the tool-result → context-assembly seam. — I14:L40, I14:L47-L49, I14:L65-L67
- **I-005/DII005-I14-018** [SRC] (DEEP-I-005.discovery.md:L152| unit=I-14): `[E][dependency]` Share the E-52 headless rust-analyzer client; never reimplement the LSP wire protocol. — I14:L29, I14:L50, I14:L61, I14:L74
- **I-005/DII005-I14-019** [SRC] (DEEP-I-005.discovery.md:L153| unit=I-14): `[E][dependency]` Share `knowledge::tree_sitter` grammar files; never duplicate them. — I14:L30, I14:L51, I14:L62
- **I-005/DII005-I14-020** [SRC] (DEEP-I-005.discovery.md:L154| unit=I-14): `[E][composition]` H-6 masking drops turns while I-14 compresses payloads; the layers coexist without replacing one another. — I14:L32, I14:L64, I14:L99
- **I-005/DII005-I14-021** [SRC] (DEEP-I-005.discovery.md:L155| unit=I-14 SURFACE): `[E][surface]` Per-transform receipt rows in the Transcript drawer are required and show token delta plus reversibility link. — I14:L52, I14:L77-L78
- **I-005/DII005-I14-022** [SRC] (DEEP-I-005.discovery.md:L156| unit=I-14): `[E][fixture/acceptance]` The seeded fixture reduces a 10K-token compiler panic to a ≤500-token LSP slice and emits its receipt. — I14:L41, I14:L79-L82
- **I-005/DII005-I14-023** [SRC] (DEEP-I-005.discovery.md:L157| unit=I-14): `[E][measurement]` Fixed-fixture pass rate must remain unchanged versus uncompressed context. — I14:L82
- **I-005/DII005-I14-024** [SRC] (DEEP-I-005.discovery.md:L158| unit=I-14): `[E][negative test]` The unjournaled-transform broken arm must fail the receipt-presence check. — I14:L82
- **I-005/DII005-I14-025** [SRC] (DEEP-I-005.discovery.md:L159| unit=I-14): `[E][deferred]` Python/TypeScript tree-sitter grammars beyond Rust are deferred pending grammar acquisition. — I14:L45
- **I-005/DII005-I14-026** [SRC] (DEEP-I-005.discovery.md:L160| unit=I-14): `[E][safety]` Production transform/receipt paths ban `.unwrap()`/`.expect()` and reverse pointers must use blake3, not blake2/sha2. — I14:L75-L76
- **I-005/DII005-I14-027** [SRC] (DEEP-I-005.discovery.md:L161| unit=I-14): `[E][done gate]` Full-form expansion is required before execution because this effort-M feature spans multiple crates. — I14:L93

#### I-005 / unit `I-22` (29 records)

- **I-005/DII005-I22-001** [SRC] (DEEP-I-005.discovery.md:L165| unit=I-22): `[E][status/dependency]` I-22 is Track 15, `ANTI-GAMING`, `STRONG`, `BUILD`, effort M, Wave 4; it depends on I-4 and binds ID-4. — I22:L1-L3
- **I-005/DII005-I22-002** [SD] (DEEP-I-005.discovery.md:L166| unit=I-22): `[E][reward]` The step reward is the potential delta `phi(h_t) - phi(h_{t-1})`, not an absolute score. — I22:L4-L10, I22:L12-L17
- **I-005/DII005-I22-003** [SRC] (DEEP-I-005.discovery.md:L167| unit=I-22): `[E][reward field]` `phi` includes test-pass count. — I22:L13-L15, I22:L46-L47
- **I-005/DII005-I22-004** [SRC] (DEEP-I-005.discovery.md:L168| unit=I-22): `[E][reward field]` `phi` includes inverted compile-error count. — I22:L13-L15, I22:L46-L47
- **I-005/DII005-I22-005** [SRC] (DEEP-I-005.discovery.md:L169| unit=I-22): `[E][reward field]` `phi` includes requirements-covered count from I-37 EARS when available, otherwise wiring-contract fields. — I22:L14-L15, I22:L46-L47
- **I-005/DII005-I22-006** [SRC] (DEEP-I-005.discovery.md:L170| unit=I-22): `[E][anti-gaming]` Holding position earns zero, improvement earns positive reward, and regression earns negative reward. — I22:L15-L17
- **I-005/DII005-I22-007** [SRC] (DEEP-I-005.discovery.md:L171| unit=I-22 SURFACE): `[E][measurement/default]` Reward weights are measured on ELAI fixtures and written by first deployment to `metrics/progress_reward_weights.toml`; no paper vector is a gate. — I22:L16-L20, I22:L50, I22:L71, I22:L84
- **I-005/DII005-I22-008** [SRC] (DEEP-I-005.discovery.md:L172| unit=I-22): `[E][determinism]` Every `phi` component is a deterministic journal-enumerable observable, and the component breakdown is reproducible under `ELAI_SEED`. — I22:L9-L10, I22:L18-L20, I22:L82, I22:L94-L95
- **I-005/DII005-I22-009** [SRC] (DEEP-I-005.discovery.md:L173| unit=I-22): `[E][constraint]` LLM opinion, self-report, plausible-looking output without observable state change, and other nondeterministic `phi` signals are structurally excluded. — I22:L18-L20, I22:L32-L33, I22:L82
- **I-005/DII005-I22-010** [SRC] (DEEP-I-005.discovery.md:L174| unit=I-22): `[E][localization]` The first `I-17 TriLayerStack` `HardVeto` at step k is the first-error trigger. — I22:L21-L22, I22:L41, I22:L75
- **I-005/DII005-I22-011** [SRC] (DEEP-I-005.discovery.md:L175| unit=I-22): `[E][recovery]` On that veto, truncate to k−1, preserve the last passing state as an H-49 fork node, and reinvest generation from that prefix. — I22:L21-L24, I22:L48, I22:L69, I22:L74-L75
- **I-005/DII005-I22-012** [SD] (DEEP-I-005.discovery.md:L176| unit=I-22): `[E][anti-spiral]` After D consecutive zero-delta steps, multiply the next reward by a decay factor less than one. — I22:L25-L27, I22:L49
- **I-005/DII005-I22-013** [SRC] (DEEP-I-005.discovery.md:L177| unit=I-22 SURFACE): `[E][config/measurement]` D is measured on the ELAI fixture corpus and must not be an unbounded or hard-coded source constant. — I22:L25-L28, I22:L49-L50, I22:L85
- **I-005/DII005-I22-014** [SRC] (DEEP-I-005.discovery.md:L178| unit=I-22): `[E][default algorithm]` Decay is linear-step until measured behavior supports another curve. — I22:L27-L28
- **I-005/DII005-I22-015** [SRC] (DEEP-I-005.discovery.md:L179| unit=I-22): `[E][behavior]` Accumulated decay breaks isomorphic-state spirals, causing backtracking and sibling expansion in the stated fixture. — I22:L27-L28, I22:L93-L95
- **I-005/DII005-I22-016** [SRC] (DEEP-I-005.discovery.md:L180| unit=I-22 SURFACE): `[E][composition]` This per-branch search mechanism is a sibling of, and does not replace or interact with, the main-session anti-spiral CLI skill. — I22:L29-L31, I22:L39, I22:L103, I22:L112-L113
- **I-005/DII005-I22-017** [SD] (DEEP-I-005.discovery.md:L181| unit=I-22): `[E][capability]` Create `crates/orchestrator/src/search/score.rs::DeltaProgressScorer` for potential, shaped reward, and decay. — I22:L46, I22:L56-L59, I22:L67-L68
- **I-005/DII005-I22-018** [SRC] (DEEP-I-005.discovery.md:L182| unit=I-22): `[E][capability]` Create `crates/orchestrator/src/search/score.rs::FirstErrorTruncator` for H-49 fork-at-k−1 logic. — I22:L60-L61, I22:L69
- **I-005/DII005-I22-019** [SRC] (DEEP-I-005.discovery.md:L183| unit=I-22): `[E][integration]` Wire the scorer into I-4 `search/ladder.rs` at the PUCT expansion/value-function seam. — I22:L38, I22:L57-L59, I22:L76, I22:L78-L80
- **I-005/DII005-I22-020** [SRC] (DEEP-I-005.discovery.md:L184| unit=I-22): `[E][integration nuance]` I-22 shapes the input received by shipped D-2 `PpmClient` Layer 2; it does not reimplement or replace the PPM output. — I22:L35-L36, I22:L73
- **I-005/DII005-I22-021** [SRC] (DEEP-I-005.discovery.md:L185| unit=I-22): `[E][deferred/stub lifecycle]` The I-37 EARS requirements component ships as a registered zero-contribution stub with `will_resolve_in=i-37` until I-37 lands. — I22:L52-L54, I22:L77
- **I-005/DII005-I22-022** [SRC] (DEEP-I-005.discovery.md:L186| unit=I-22 SURFACE): `[E][deferred]` Exponential/adaptive decay schedules remain deferred until ELAI measurements justify them. — I22:L54
- **I-005/DII005-I22-023** [SD] (DEEP-I-005.discovery.md:L187| unit=I-22): `[E][events]` Add `ProgressRewardScored`, baseline +1, with `Delta`, `FirstErrorTruncated`, and `DecayApplied` discriminators. — I22:L51, I22:L62, I22:L70, I22:L106
- **I-005/DII005-I22-024** [SRC] (DEEP-I-005.discovery.md:L188| unit=I-22 SURFACE): `[E][surface]` The I-4 branch board renders a per-branch `phi` sparkline with decay regions distinguished, consuming existing SSB subscriptions; `cross_lang_diff` must pass. — I22:L63-L64, I22:L87-L91
- **I-005/DII005-I22-025** [SD] (DEEP-I-005.discovery.md:L189| unit=I-22): `[E][observability]` Emit `Delta` rows per step with `test_pass`, `compile_err_inv`, and `req_covered` breakdown. — I22:L89-L90, I22:L93-L95
- **I-005/DII005-I22-026** [SD] (DEEP-I-005.discovery.md:L190| unit=I-22): `[E][observability]` Emit `DecayApplied` after D zero-delta steps and `FirstErrorTruncated` with the H-49 k−1 node identifier. — I22:L93-L95
- **I-005/DII005-I22-027** [SRC] (DEEP-I-005.discovery.md:L191| unit=I-22): `[E][negative test]` An `--llm-phi` broken arm must fail the contract asserting fully deterministic `phi` components. — I22:L95
- **I-005/DII005-I22-028** [SRC] (DEEP-I-005.discovery.md:L192| unit=I-22): `[E][safety]` Production paths ban `.unwrap()`/`.expect()`, fabricated `phi` measurements, and blake2/sha2. — I22:L86
- **I-005/DII005-I22-029** [SRC] (DEEP-I-005.discovery.md:L193| unit=I-22): `[E][done gate]` Full-form expansion is required before execution. — I22:L115

#### I-005 / unit `I-25` (57 records)

- **I-005/DII005-I25-001** [SRC] (DEEP-I-005.discovery.md:L197| unit=I-25): `[E][status]` I-25 is Track 16, `CHEAPER-SPEED`, `STRONG`, `EXTENSION`, effort M, Wave 4, bound by ID-5/ID-6. — I25:L1-L3
- **I-005/DII005-I25-002** [SRC] (DEEP-I-005.discovery.md:L198| unit=I-25 SURFACE): `[E][learning]` Train routing on ELAI journal verifier outcomes keyed by task features, lane, and cost rather than human/proxy labels. — I25:L4-L8, I25:L10-L13
- **I-005/DII005-I25-003** [SRC] (DEEP-I-005.discovery.md:L199| unit=I-25): `[E][feature]` The base feature family includes task class expressed through AST shape. — I25:L12-L14
- **I-005/DII005-I25-004** [SRC] (DEEP-I-005.discovery.md:L200| unit=I-25): `[E][feature]` The base feature family includes DKG centrality. — I25:L12-L14
- **I-005/DII005-I25-005** [SRC] (DEEP-I-005.discovery.md:L201| unit=I-25): `[E][feature]` The base feature family includes file count. — I25:L12-L14
- **I-005/DII005-I25-006** [SRC] (DEEP-I-005.discovery.md:L202| unit=I-25): `[E][feature]` The base feature family includes historical cost shape per task class. — I25:L13-L14
- **I-005/DII005-I25-007** [SRC] (DEEP-I-005.discovery.md:L203| unit=I-25): `[E][feature]` The base feature family includes recent pass rate per lane. — I25:L13-L14
- **I-005/DII005-I25-008** [SRC] (DEEP-I-005.discovery.md:L204| unit=I-25): `[E][model]` The router is deliberately small; the revised Slice 2 selects a logistic classifier, while neural routing is deferred. — I25:L14-L15, I25:L46-L48, I25:L52-L54
- **I-005/DII005-I25-009** [SRC] (DEEP-I-005.discovery.md:L205| unit=I-25): `[E][determinism]` Offline training uses `ELAI_SEED`-sealed journal exports. — I25:L15
- **I-005/DII005-I25-010** [SRC] (DEEP-I-005.discovery.md:L206| unit=I-25): `[E][measurement]` Over-escalation is measured on ELAI's own mixed batch; RouteLLM paper numbers are forbidden as thresholds. — I25:L16-L18, I25:L87, I25:L104
- **I-005/DII005-I25-011** [SRC] (DEEP-I-005.discovery.md:L207| unit=I-25): `[E][promotion]` A learned router that does not beat the H-7 heuristic baseline is not promoted; the promotion row must cite the ledger comparison. — I25:L16-L18, I25:L36, I25:L85, I25:L105
- **I-005/DII005-I25-012** [SRC] (DEEP-I-005.discovery.md:L208| unit=I-25): `[E][routing]` Before ordinary routing, a historically expensive task-shape fingerprint match pre-routes to the deep lane. — I25:L18-L20
- **I-005/DII005-I25-013** [SRC] (DEEP-I-005.discovery.md:L209| unit=I-25): `[E][events]` Reuse the live E-31 `RouteDecisionEvent`; do not add an action type or schema bump. — I25:L20-L21, I25:L27-L28, I25:L76, I25:L86, I25:L107, I25:L153-L154
- **I-005/DII005-I25-014** [SRC] (DEEP-I-005.discovery.md:L210| unit=I-25): `[E][event schema]` Extend only `RouteDecisionEventPayload.details_json` with `feature_vec`, `predicted_lane`, and `reason`. — I25:L20-L21, I25:L43-L44, I25:L76, I25:L97
- **I-005/DII005-I25-015** [SRC] (DEEP-I-005.discovery.md:L211| unit=I-25 SURFACE): `[E][flag/default]` The existing emission remains default-off behind `ELAI_ROUTE_DECISION_SSB_ENABLE`. — I25:L27-L28, I25:L80-L82
- **I-005/DII005-I25-016** [SD] (DEEP-I-005.discovery.md:L212| unit=I-25): `[E][seam correction]` The ratified primary live hook is exact function `task_runner.rs::run_cost_router_preclassify`, called on pre-cloud dispatch. — I25:L25-L31, I25:L60, I25:L71, I25:L74, I25:L80-L82
- **I-005/DII005-I25-017** [SRC] (DEEP-I-005.discovery.md:L213| unit=I-25): `[E][seam constraint]` `crates/orchestrator/src/escalation_policy.rs` is E-32 handoff policy and must not be treated as the I-25 hook. — I25:L29-L31
- **I-005/DII005-I25-018** [SRC] (DEEP-I-005.discovery.md:L214| unit=I-25): `[E][fallback]` `providers/src/runtime/escalation_policy.rs::RuntimeRoutingPolicy::evaluate` is a later fallback composition target and currently has no production callers; it cannot be Slice 1's sole hook. — I25:L31-L32, I25:L55, I25:L77, I25:L90
- **I-005/DII005-I25-019** [SRC] (DEEP-I-005.discovery.md:L215| unit=I-25): `[E][gap/status]` `CostProfiler` is confirmed absent; `crates/orchestrator/src/routing/cost_profile.rs` is a genuine build gap alongside the existing C-46 routing modules. — I25:L33-L34
- **I-005/DII005-I25-020** [SRC] (DEEP-I-005.discovery.md:L216| unit=I-25): `[E][capability/slice 1]` Create `crates/orchestrator/src/routing/cost_profile.rs::CostProfiler` as an approximately 80-LOC deterministic task-shape classifier with a `BTreeMap` cache. — I25:L39-L45, I25:L58-L60, I25:L68-L71
- **I-005/DII005-I25-021** [SRC] (DEEP-I-005.discovery.md:L217| unit=I-25): `[E][routing order]` Consult `CostProfiler` in `run_cost_router_preclassify` before `route_for_difficulty_and_budget`; non-match/default-off preserves E-39. — I25:L42-L45, I25:L71, I25:L75, I25:L97, I25:L111
- **I-005/DII005-I25-022** [SRC] (DEEP-I-005.discovery.md:L218| unit=I-25): `[E][event behavior]` An expensive match emits `details_json.reason=cost_profile_expensive_shape` plus predicted lane and feature vector. — I25:L43-L45, I25:L95-L98, I25:L140-L143
- **I-005/DII005-I25-023** [SRC] (DEEP-I-005.discovery.md:L219| unit=I-25): `[E][test/slice 1]` Create deterministic in-process contract `crates/contracts/tests/i25_ledger_trained_router_contract.rs`. — I25:L44-L45, I25:L62, I25:L96, I25:L175-L177
- **I-005/DII005-I25-024** [SRC] (DEEP-I-005.discovery.md:L220| unit=I-25): `[E][capability/slice 2]` Create `workflow/training/router/train_router.py::RouterTrainer`, approximately 150 LOC of Python stdlib. — I25:L46-L48, I25:L63, I25:L69-L70
- **I-005/DII005-I25-025** [SRC] (DEEP-I-005.discovery.md:L221| unit=I-25): `[E][data/security]` The trainer opens `ssb.db` read-only via `sqlite3`. — I25:L46-L48
- **I-005/DII005-I25-026** [SRC] (DEEP-I-005.discovery.md:L222| unit=I-25): `[E][training data]` The trainer joins `RouteDecisionEvent` to verifier rows and `UsageTurnEvent`. — I25:L47-L48
- **I-005/DII005-I25-027** [SRC] (DEEP-I-005.discovery.md:L223| unit=I-25): `[E][artifact]` The trainer writes a TOML model artifact and `workflow/training/router/eval_report.toml`. — I25:L47-L48, I25:L98
- **I-005/DII005-I25-028** [SRC] (DEEP-I-005.discovery.md:L224| unit=I-25): `[E][failure]` Insufficient labeled rows produce an explicit `REFUSED` report and no silent model. — I25:L48, I25:L144-L145
- **I-005/DII005-I25-029** [SRC] (DEEP-I-005.discovery.md:L225| unit=I-25): `[E][capability/slice 3]` Load a router artifact from `elai_cc.toml` only when a trial-ledger row records a win. — I25:L49-L50
- **I-005/DII005-I25-030** [SRC] (DEEP-I-005.discovery.md:L226| unit=I-25): `[E][fallback/recovery]` A blocked/losing artifact is never loaded; H-7/E-39 heuristics remain the fallback. — I25:L49-L50, I25:L85, I25:L98, I25:L146
- **I-005/DII005-I25-031** [SRC] (DEEP-I-005.discovery.md:L227| unit=I-25): `[E][security/frozen boundary]` `trial_ledger.rs`, `deflation_gate.rs`, `holdout_budget.rs`, and `gate_self_validation.rs` are R8-frozen consume-only files; touching any is an abort. — I25:L35-L36, I25:L64, I25:L78-L79, I25:L89, I25:L158-L159
- **I-005/DII005-I25-032** [SRC] (DEEP-I-005.discovery.md:L228| unit=I-25): `[E][ownership]` The Python trainer never writes ledger rows; `TrialLedger::register_trial` remains the Rust writer. — I25:L69-L70, I25:L153-L156
- **I-005/DII005-I25-033** [SRC] (DEEP-I-005.discovery.md:L229| unit=I-25): `[E][deferred]` The neural router is RunPod/VRAM-gated Tier 2. — I25:L22-L23, I25:L52-L54
- **I-005/DII005-I25-034** [SRC] (DEEP-I-005.discovery.md:L230| unit=I-25): `[E][deferred]` Continuous online retraining waits for I-4's high-volume trajectory stream. — I25:L54
- **I-005/DII005-I25-035** [SRC] (DEEP-I-005.discovery.md:L231| unit=I-25 SURFACE): `[E][deferred/UI]` Route-explanation dashboard/Svelte work is deferred by `owner-ui-rework-hold`; this unit emits backend details only. — I25:L55-L56, I25:L92-L93, I25:L177
- **I-005/DII005-I25-036** [SRC] (DEEP-I-005.discovery.md:L232| unit=I-25): `[E][integration]` Create/modify only the listed Slice files: `cost_profile.rs`, `task_runner.rs`, `routing/mod.rs`, the contract, and the Python trainer. — I25:L58-L64
- **I-005/DII005-I25-037** [SRC] (DEEP-I-005.discovery.md:L233| unit=I-25 SURFACE): `[E][reachability]` Production reachability is `elai run <task>` → task-runner pre-cloud path → `run_cost_router_preclassify` → `CostProfiler`. — I25:L80-L82
- **I-005/DII005-I25-038** [SRC] (DEEP-I-005.discovery.md:L234| unit=I-25): `[E][acceptance/evidence]` Contract/live fixtures require an expensive match event with the exact reason and a non-match leaving E-39 unchanged. — I25:L95-L98, I25:L140-L143
- **I-005/DII005-I25-039** [SRC] (DEEP-I-005.discovery.md:L235| unit=I-25): `[E][artifact integrity]` `eval_report.toml` carries ELAI-measured over-escalation/mis-route rates, seed, and input blake3; the loser artifact path is absent when blocked. — I25:L98, I25:L144-L146
- **I-005/DII005-I25-040** [SRC] (DEEP-I-005.discovery.md:L236| unit=I-25): `[E][difficulty fusion]` The J-11 amendment adds DeNovoSWE's convex feature `d_i = w_s·d_i^struct + w_g·ℓ_i^(g) + w_q·ℓ_i^(q)` with nonnegative weights summing to one. — I25:L118-L123
- **I-005/DII005-I25-041** [SRC] (DEEP-I-005.discovery.md:L237| unit=I-25): `[E][difficulty feature]` The structural reference signal is executable Python lines, transformed by `log(1+x)` and min-max normalized against the 5th/95th percentiles. — I25:L124-L126
- **I-005/DII005-I25-042** [SRC] (DEEP-I-005.discovery.md:L238| unit=I-25): `[E][difficulty feature]` Two independent LLM judges, g and q, each provide a five-level qualitative difficulty signal. — I25:L127-L128
- **I-005/DII005-I25-043** [SRC] (DEEP-I-005.discovery.md:L239| unit=I-25): `[E][calibration]` Choose fusion weights by maximizing absolute Pearson correlation with empirical pass rate over a dense two-dimensional simplex grid. — I25:L129-L131
- **I-005/DII005-I25-044** [SRC] (DEEP-I-005.discovery.md:L240| unit=I-25 SURFACE): `[E][honesty]` Winning DeNovoSWE weights are unpublished, so ELAI calibrates weights on its own rollouts. — I25:L129-L131
- **I-005/DII005-I25-045** [SRC] (DEEP-I-005.discovery.md:L241| unit=I-25 SURFACE): `[E][cross-language adaptation]` Candidate ELAI structural substitutes are SCIP function count, cross-language bindings touched, and SSB action-type variants added per task. — I25:L132-L134
- **I-005/DII005-I25-046** [SRC] (DEEP-I-005.discovery.md:L242| unit=I-25): `[E][authority]` Difficulty fusion is merely a router feature; ID-6 ledger gates remain authoritative. — I25:L134-L135
- **I-005/DII005-I25-047** [SRC] (DEEP-I-005.discovery.md:L243| unit=I-25 SURFACE): `[E][measurement constraint]` The cited +1.2 percentage-point result is rationale only and never an ELAI target. — I25:L135-L136
- **I-005/DII005-I25-048** [SRC] (DEEP-I-005.discovery.md:L244| unit=I-25): `[E][failure/recovery]` Missing event, wrong reason, or default-off E-39 regression directs the executor to re-read the live hook/emission sites and rerun the I-25 contract. — I25:L148-L151
- **I-005/DII005-I25-049** [SRC] (DEEP-I-005.discovery.md:L245| unit=I-25): `[E][fork decision]` FORK-1 fixes payload evolution to `details_json` keys with no schema/action-type change. — I25:L153-L154
- **I-005/DII005-I25-050** [SRC] (DEEP-I-005.discovery.md:L246| unit=I-25): `[E][fork decision]` FORK-2 fixes Slice 1 to the live `run_cost_router_preclassify` hook and leaves `RuntimeRoutingPolicy::evaluate` for later. — I25:L154-L155
- **I-005/DII005-I25-051** [SRC] (DEEP-I-005.discovery.md:L247| unit=I-25): `[E][fork decision]` FORK-3 keeps the ledger writer in Rust `TrialLedger::register_trial`. — I25:L155-L156
- **I-005/DII005-I25-052** [SRC] (DEEP-I-005.discovery.md:L248| unit=I-25): `[E][abort]` Stop rather than invent `SCHEMA_VERSION` if live SSB integers cannot be reread; inventing a new `RouteDecisionEvent` variant is also a stop condition. — I25:L158-L161
- **I-005/DII005-I25-053** [SRC] (DEEP-I-005.discovery.md:L249| unit=I-25): `[E][done gates]` Off-agent verification includes `plan_gate_check.py`, the contract test, orchestrator all-target check, production-caller checks, Python byte-compilation, and `python_wire_check`. — I25:L163-L169
- **I-005/DII005-I25-054** [SRC] (DEEP-I-005.discovery.md:L250| unit=I-25): `[E][red team]` Promotion without a ledger row, silent losing-artifact adoption, new action type, and a dark-only hook must all fail. — I25:L170-L173
- **I-005/DII005-I25-055** [SRC] (DEEP-I-005.discovery.md:L251| unit=I-25): `[E][status/recon]` The source reports no open seam reconnaissance after rereading the live hook, routing inventory, live event, and trial writer. — I25:L179-L180
- **I-005/DII005-I25-056** [SRC] (DEEP-I-005.discovery.md:L252| unit=I-25): `[E][composition]` I-25 composes E-39, H-7, E-31, C-46, E-20 privacy routing, and IS-16 without reimplementing them. — I25:L110-L116
- **I-005/DII005-I25-057** [SRC] (DEEP-I-005.discovery.md:L253| unit=I-25): `[E][safety]` Production routing bans `.unwrap()`/`.expect()`, fabricated measurements, and paper thresholds. — I25:L84-L90

#### I-005 / unit `I-31` (47 records)

- **I-005/DII005-I31-001** [SRC] (DEEP-I-005.discovery.md:L257| unit=I-31): `[E][status/dependency]` I-31 is Track 17, `SELF-IMPROVEMENT`, `STRONG`, `BUILD`, effort XL, Wave 5, explicitly LAST; it depends on I-30, H-43/H-44/H-28, and I-24 and binds ID-6/IC-7. — I31:L1-L3
- **I-005/DII005-I31-002** [SRC] (DEEP-I-005.discovery.md:L258| unit=I-31): `[E][security premise]` A self-improving loop is invalid if it can influence its evaluator; the evaluator must be stronger than and unreachable by the generator. — I31:L4-L7
- **I-005/DII005-I31-003** [SRC] (DEEP-I-005.discovery.md:L259| unit=I-31): `[E][kill switch]` The lane stays behind the existing `darwinian-evolve` kill-switched allowlist. — I31:L7-L8, I31:L64, I31:L132, I31:L144
- **I-005/DII005-I31-004** [SRC] (DEEP-I-005.discovery.md:L260| unit=I-31): `[E][constraint]` DPO is inherited as E-55 `HARD-AVOID`; PPO is also excluded because its value-network overhead is unjustified here. — I31:L8-L9, I31:L44-L45, I31:L112, I31:L133
- **I-005/DII005-I31-005** [SRC] (DEEP-I-005.discovery.md:L261| unit=I-31): `[E][done gate]` Full-form expansion is required before execution. — I31:L11
- **I-005/DII005-I31-006** [SRC] (DEEP-I-005.discovery.md:L262| unit=I-31): `[E][algorithm]` GRPO generates K completions, scores them, normalizes reward within the group, and updates the policy. — I31:L13-L15
- **I-005/DII005-I31-007** [SRC] (DEEP-I-005.discovery.md:L263| unit=I-31): `[E][service]` Create typed local supervisor-side reward service exact symbol `crates/training/grpo_harness.rs::RewardEndpoint`; Tier 1 chooses an Axum handler. — I31:L15-L17, I31:L40-L43, I31:L68-L69, I31:L90-L93
- **I-005/DII005-I31-008** [SRC] (DEEP-I-005.discovery.md:L264| unit=I-31): `[E][security boundary]` The supervisor owns the endpoint address and reward code; the trained policy can receive only returned reward values. — I31:L15-L17, I31:L92, I31:L111
- **I-005/DII005-I31-009** [SRC] (DEEP-I-005.discovery.md:L265| unit=I-31): `[E][reward]` Execution-grounded reward includes the I-17 L1 compile pass/fail. — I31:L17-L18
- **I-005/DII005-I31-010** [SRC] (DEEP-I-005.discovery.md:L266| unit=I-31): `[E][reward]` Execution-grounded reward includes I-17 L1 test pass-count/total. — I31:L18
- **I-005/DII005-I31-011** [SRC] (DEEP-I-005.discovery.md:L267| unit=I-31): `[E][reward]` Execution-grounded reward includes the I-20 CI-cleared mutation score. — I31:L18-L19
- **I-005/DII005-I31-012** [SRC] (DEEP-I-005.discovery.md:L268| unit=I-31): `[E][reward]` Execution-grounded reward includes I-8's null-test halting/abort signal. — I31:L19
- **I-005/DII005-I31-013** [SRC] (DEEP-I-005.discovery.md:L269| unit=I-31): `[E][reward]` XGrammar conformance contributes an additive format penalty. — I31:L19-L20
- **I-005/DII005-I31-014** [SRC] (DEEP-I-005.discovery.md:L270| unit=I-31): `[E][constraint]` There is no neural reward model; only execution gates supply fitness. — I31:L20
- **I-005/DII005-I31-015** [SRC] (DEEP-I-005.discovery.md:L271| unit=I-31): `[E][anti-collapse/config]` Apply `KL(policy || I-30 SFT init checkpoint)` with a configurable coefficient; skip the update when KL exceeds its threshold. — I31:L20-L22
- **I-005/DII005-I31-016** [SRC] (DEEP-I-005.discovery.md:L272| unit=I-31): `[E][training]` Inject I-30 fluent-but-vetoed JSONL negative pairs as masked negative examples. — I31:L22-L24
- **I-005/DII005-I31-017** [SRC] (DEEP-I-005.discovery.md:L273| unit=I-31): `[E][constraint]` Old `derived_alpha = blake3(config bytes)` fitness is retired and structurally forbidden. — I31:L24-L25, I31:L29, I31:L49-L52, I31:L109-L110
- **I-005/DII005-I31-018** [SRC] (DEEP-I-005.discovery.md:L274| unit=I-31): `[E][anti-gaming]` Score AST-preserving I-24 canary pairs; divergence above threshold emits `ScorerNeutralized` and skips that batch's update. — I31:L25-L27
- **I-005/DII005-I31-019** [SRC] (DEEP-I-005.discovery.md:L275| unit=I-31): `[E][blocking prerequisite]` H-43 execution-grounded fitness must be live with no byte-hash path. — I31:L28-L30, I31:L50-L52, I31:L95
- **I-005/DII005-I31-020** [SRC] (DEEP-I-005.discovery.md:L276| unit=I-31): `[E][blocking prerequisite]` H-44 behavioral `score_candidate_for_traps` must be live, replacing the retired keyword-grep detector. — I31:L29-L30, I31:L53-L55, I31:L96
- **I-005/DII005-I31-021** [SRC] (DEEP-I-005.discovery.md:L277| unit=I-31): `[E][blocking prerequisite]` I-24 canaries must already appear as `PrmCanaryEvent` rows in a live scoring run. — I31:L31, I31:L60, I31:L98
- **I-005/DII005-I31-022** [SRC] (DEEP-I-005.discovery.md:L278| unit=I-31): `[E][blocking prerequisite/security]` H-28 reward-tampering gate plus evaluator/harness/thresholds must be in the R8 frozen set and pass `gate_freeze_check.py`. — I31:L32-L33, I31:L57-L58, I31:L70, I31:L97
- **I-005/DII005-I31-023** [SRC] (DEEP-I-005.discovery.md:L279| unit=I-31): `[E][blocking prerequisite/promotion]` H-17 trial-N persistence and H-18 deflation must pass before production loads a checkpoint. — I31:L34-L35, I31:L99
- **I-005/DII005-I31-024** [SRC] (DEEP-I-005.discovery.md:L280| unit=I-31): `[E][blocking prerequisite]` D-6 held-out evaluation uses data never exposed to training. — I31:L36
- **I-005/DII005-I31-025** [SRC] (DEEP-I-005.discovery.md:L281| unit=I-31): `[E][blocking prerequisite]` D-7 adds a cross-domain promotion canary. — I31:L37
- **I-005/DII005-I31-026** [SRC] (DEEP-I-005.discovery.md:L282| unit=I-31): `[E][operations]` Ship a deterministic small-fixture harness first, exercising the real reward endpoint, canary injection, and KL computation. — I31:L38-L40, I31:L66-L74
- **I-005/DII005-I31-027** [SRC] (DEEP-I-005.discovery.md:L283| unit=I-31): `[E][dependency/security]` Any RL stack (TRL, veRL, or custom) requires package/repository/date/license anti-slopsquat verification and explicit user confirmation before import. — I31:L40-L42, I31:L76-L78, I31:L114
- **I-005/DII005-I31-028** [SRC] (DEEP-I-005.discovery.md:L284| unit=I-31): `[E][dependency]` The reward endpoint itself adds no external dependency. — I31:L42-L43
- **I-005/DII005-I31-029** [SRC] (DEEP-I-005.discovery.md:L285| unit=I-31): `[E][capability]` Create `crates/training/grpo_harness.rs::GrpoHarness` with reward handling, KL penalty, canary hook, masking, and SSB emission. — I31:L68-L69, I31:L83-L84, I31:L89-L93
- **I-005/DII005-I31-030** [SRC] (DEEP-I-005.discovery.md:L286| unit=I-31): `[E][workspace lifecycle]` Create the `crates/training/` workspace crate only if absent; otherwise modify the existing crate after re-grep. — I31:L83-L84
- **I-005/DII005-I31-031** [SRC] (DEEP-I-005.discovery.md:L287| unit=I-31): `[E][allowlist lifecycle]` Reuse the `darwinian-evolve` allowlist row if it covers GRPO; otherwise add a promotion row under R9. — I31:L85
- **I-005/DII005-I31-032** [SRC] (DEEP-I-005.discovery.md:L288| unit=I-31): `[E][test]` Create `crates/contracts/tests/i31_grpo_flywheel_contract.rs` with a seeded H-28 reward hack, missing trial-N rejection, and byte-hash-absence check. — I31:L72-L74
- **I-005/DII005-I31-033** [SRC] (DEEP-I-005.discovery.md:L289| unit=I-31): `[E][artifact]` Ship `workflow/reasoning/I-31-decisions.md` with the blocking-prerequisite checklist. — I31:L71-L74
- **I-005/DII005-I31-034** [SRC] (DEEP-I-005.discovery.md:L290| unit=I-31): `[E][deferred]` The live multi-night GRPO run is a user-approved, dependency-approved RunPod Tier-2 checkbox. — I31:L38-L42, I31:L76-L79
- **I-005/DII005-I31-035** [SRC] (DEEP-I-005.discovery.md:L291| unit=I-31): `[E][deferred]` Full checkpoint promotion waits for a training-produced checkpoint. — I31:L79
- **I-005/DII005-I31-036** [SRC] (DEEP-I-005.discovery.md:L292| unit=I-31): `[E][deferred]` Online D-5/D-29 bandit integration waits for live-run volume. — I31:L80
- **I-005/DII005-I31-037** [SRC] (DEEP-I-005.discovery.md:L293| unit=I-31 SURFACE): `[E][surface]` The flywheel panel shows checkpoint admits, H-28 blocks, and per-episode KL trend; it consumes `RewardServed`, `PromotionBlocked`, and `PromotionLedgered` rows and must pass `cross_lang_diff.py`. — I31:L87, I31:L117-L118
- **I-005/DII005-I31-038** [SRC] (DEEP-I-005.discovery.md:L294| unit=I-31 SURFACE): `[E][reachability]` Tier-2 production reachability is `elai train grpo --init <sft-checkpoint>`; Tier-1 contract-only invocation does not itself satisfy production R7. — I31:L104-L107
- **I-005/DII005-I31-039** [SRC] (DEEP-I-005.discovery.md:L295| unit=I-31): `[E][events]` Add `GrpoRewardComputed`, baseline +1, with `RewardServed`, `PromotionBlocked`, and `PromotionLedgered` kinds. — I31:L107, I31:L117-L118, I31:L120-L123, I31:L134
- **I-005/DII005-I31-040** [SRC] (DEEP-I-005.discovery.md:L296| unit=I-31): `[E][traceability]` `RewardServed` rows cite execution-gate verdicts, including linked `TriLayerVerdict`/`MutationBudgetedRun` rows. — I31:L120-L123
- **I-005/DII005-I31-041** [SRC] (DEEP-I-005.discovery.md:L297| unit=I-31): `[E][promotion evidence]` `PromotionLedgered` must cite trial-N, deflation, held-out, and D-7 canary row identifiers. — I31:L113, I31:L120-L123
- **I-005/DII005-I31-042** [SRC] (DEEP-I-005.discovery.md:L298| unit=I-31): `[E][negative test]` Grep for `derived_alpha` or `blake3(config` in the harness must return zero. — I31:L109-L110, I31:L123
- **I-005/DII005-I31-043** [SRC] (DEEP-I-005.discovery.md:L299| unit=I-31): `[E][negative test]` A seeded harness-edit/reward-hack fixture must yield `PromotionBlocked`, never `PromotionLedgered`. — I31:L72-L73, I31:L122-L123
- **I-005/DII005-I31-044** [SRC] (DEEP-I-005.discovery.md:L300| unit=I-31): `[E][negative test]` A candidate lacking the complete promotion chain must not emit `PromotionLedgered`. — I31:L123
- **I-005/DII005-I31-045** [SRC] (DEEP-I-005.discovery.md:L301| unit=I-31): `[E][security]` The evaluator endpoint and all gate code stay outside the training loop's write boundary. — I31:L46-L47, I31:L58, I31:L111
- **I-005/DII005-I31-046** [SRC] (DEEP-I-005.discovery.md:L302| unit=I-31): `[E][composition]` Consume H-43..H-48, I-30, I-24, H-28, IS-16, D-5/D-29, and E-55 without reauthoring their pipelines or gates. — I31:L137-L145
- **I-005/DII005-I31-047** [SRC] (DEEP-I-005.discovery.md:L303| unit=I-31): `[E][safety]` Production paths ban `.unwrap()`/`.expect()` and blake2/sha2. — I31:L115

#### I-005 / unit `I-39` (25 records)

- **I-005/DII005-I39-001** [SRC] (DEEP-I-005.discovery.md:L307| unit=I-39): `[E][status]` I-39 is Track 18, `ANTI-DRIFT`, `STRONG`, `BUILD`, effort S–M, Wave 1. — I39:L1-L3
- **I-005/DII005-I39-002** [SRC] (DEEP-I-005.discovery.md:L308| unit=I-39): `[E][problem/data]` The five drifting manual status sources are `CLAUDE.md`, session handoffs, plan index files, grounding maps, and memory-bank. — I39:L5-L7
- **I-005/DII005-I39-003** [SRC] (DEEP-I-005.discovery.md:L309| unit=I-39 SURFACE): `[E][capability]` Add CLI `elai plan status [--letter X] [--id I-N]`. — I39:L7-L9, I39:L35-L41
- **I-005/DII005-I39-004** [SRC] (DEEP-I-005.discovery.md:L310| unit=I-39): `[E][derivation]` Source one is `wiring_contract_check.py --static`, mapping its verdict to `PLANNED`/`IN-PROGRESS`/`DONE`/`BLOCKED`. — I39:L14-L16, I39:L37, I39:L50, I39:L63
- **I-005/DII005-I39-005** [SRC] (DEEP-I-005.discovery.md:L311| unit=I-39): `[E][derivation]` Source two is presence/absence of each plan's `observable_effect` SSB rows. — I39:L16-L17
- **I-005/DII005-I39-006** [SRC] (DEEP-I-005.discovery.md:L312| unit=I-39): `[E][derivation]` Source three is plan-status fragments from the `s-plan-audit-2026-06-04` FINISH files. — I39:L17-L18, I39:L31, I39:L66
- **I-005/DII005-I39-007** [SRC] (DEEP-I-005.discovery.md:L313| unit=I-39): `[E][traceability]` Every status line carries a `verified against <commit-hash/map-date>` anchor. — I39:L19
- **I-005/DII005-I39-008** [SRC] (DEEP-I-005.discovery.md:L314| unit=I-39 SURFACE): `[E][lifecycle]` On first run, manual tallies become build artifacts; mechanically derived command output is the later source of truth. — I39:L19-L21
- **I-005/DII005-I39-009** [SRC] (DEEP-I-005.discovery.md:L315| unit=I-39): `[E][determinism]` The same journal snapshot must produce byte-identical output. — I39:L20-L21, I39:L79-L81
- **I-005/DII005-I39-010** [SRC] (DEEP-I-005.discovery.md:L316| unit=I-39 SURFACE): `[E][capability]` Create `crates/cli/src/plan_status_cmd.rs::derive_plan_state()`, `format_status_line()`, and `run_plan_status()`. — I39:L35-L38, I39:L47-L48, I39:L55-L59
- **I-005/DII005-I39-011** [SRC] (DEEP-I-005.discovery.md:L317| unit=I-39): `[E][events]` Add flat beacon `PlanStateDerived`, baseline +1, carrying `letter`, `session_id`, `state`, and `evidence_anchor`. — I39:L39, I39:L56-L60, I39:L80, I39:L90
- **I-005/DII005-I39-012** [SRC] (DEEP-I-005.discovery.md:L318| unit=I-39 SURFACE): `[E][reachability]` Production status reachability is CLI invocation → `run_plan_status()` → `derive_plan_state()` → `PlanStateDerived`. — I39:L67-L69
- **I-005/DII005-I39-013** [SRC] (DEEP-I-005.discovery.md:L319| unit=I-39): `[E][migration guard]` Create `crates/contracts/tests/i39_plan_state_migration_contract.rs` and open a real F-13 archived old-schema `ssb.db` through the current E-44 decode path. — I39:L23-L26, I39:L40, I39:L51-L52, I39:L60, I39:L69-L70
- **I-005/DII005-I39-014** [SRC] (DEEP-I-005.discovery.md:L320| unit=I-39): `[E][migration acceptance]` Decode N rows to N well-formed results with no silent skip. — I39:L23-L26, I39:L40, I39:L69-L70, I39:L79-L81
- **I-005/DII005-I39-015** [SRC] (DEEP-I-005.discovery.md:L321| unit=I-39): `[E][failure]` Any corrupted/undecodable archive fails loudly and exits nonzero with a named error. — I39:L25-L26, I39:L40, I39:L72-L73, I39:L81
- **I-005/DII005-I39-016** [SRC] (DEEP-I-005.discovery.md:L322| unit=I-39): `[E][test reachability exception]` Contract callers satisfy R7 only for the migration guard because the real archived DB—not a production entrypoint—is the guard's subject. — I39:L69-L70
- **I-005/DII005-I39-017** [SRC] (DEEP-I-005.discovery.md:L323| unit=I-39): `[E][constraint]` A mock `ssb.db` is forbidden for the migration contract. — I39:L71-L72
- **I-005/DII005-I39-018** [SRC] (DEEP-I-005.discovery.md:L324| unit=I-39): `[E][constraint]` Hard-coded/manual status fields are forbidden; state must be derived mechanically. — I39:L74
- **I-005/DII005-I39-019** [SRC] (DEEP-I-005.discovery.md:L325| unit=I-39): `[E][integration]` Execute `wiring_contract_check.py` as a subprocess and parse exit code/stdout rather than reimplementing it. — I39:L49-L50, I39:L63, I39:L94
- **I-005/DII005-I39-020** [SRC] (DEEP-I-005.discovery.md:L326| unit=I-39): `[E][integration]` Reuse SSB viewer read paths, the F-13 archive, E-44 migration decode, and audit fragments; do not copy/reencode or reimplement these substrates. — I39:L47-L52, I39:L61-L66, I39:L92-L97
- **I-005/DII005-I39-021** [SRC] (DEEP-I-005.discovery.md:L327| unit=I-39 SURFACE): `[E][surface]` No dashboard is added; CLI output plus documentation are the developer-facing artifacts. — I39:L53, I39:L76-L77
- **I-005/DII005-I39-022** [SRC] (DEEP-I-005.discovery.md:L328| unit=I-39): `[E][artifact]` Ship `workflow/reasoning/I-39-decisions.md`. — I39:L41
- **I-005/DII005-I39-023** [SRC] (DEEP-I-005.discovery.md:L329| unit=I-39): `[E][deferred]` Automatic sync on each git push is deferred as infrastructure rather than correctness. — I39:L43-L45
- **I-005/DII005-I39-024** [SRC] (DEEP-I-005.discovery.md:L330| unit=I-39): `[E][deferred]` Cross-plan dependency visualization waits for post-Wave-4 data. — I39:L45
- **I-005/DII005-I39-025** [SRC] (DEEP-I-005.discovery.md:L331| unit=I-39): `[E][safety]` Production paths ban `.unwrap()`/`.expect()` and use blake3, not blake2/sha2. — I39:L75

#### I-005 / unit `I-44` (32 records)

- **I-005/DII005-I44-001** [SRC] (DEEP-I-005.discovery.md:L335| unit=I-44): `[E][status/dependency]` I-44 is Track 19, `AUDITABLE/TABLE-STAKES`, `STRONG`, `BUILD`, effort S, Wave 3; it depends on G-9 and composes G-2/F-13/I-40. — I44:L1-L3
- **I-005/DII005-I44-002** [SRC] (DEEP-I-005.discovery.md:L336| unit=I-44): `[E][scope]` Self-host, zero retention, and SOC 2 are framed as table stakes; this feature implements only the post-verification retention leg. — I44:L4-L11
- **I-005/DII005-I44-003** [SRC] (DEEP-I-005.discovery.md:L337| unit=I-44): `[E][policy/default]` Define `crates/orchestrator/src/retention.rs::RetentionPolicy` with `Full` as default and opt-in `ZeroAfterVerify`. — I44:L13-L15, I44:L44-L48, I44:L70-L73
- **I-005/DII005-I44-004** [SRC] (DEEP-I-005.discovery.md:L338| unit=I-44): `[E][timing/state]` `ZeroAfterVerify` purges only after G-2 RRA and off-agent verdicts are committed/sealed. — I44:L14-L16, I44:L48, I44:L62
- **I-005/DII005-I44-005** [SRC] (DEEP-I-005.discovery.md:L339| unit=I-44): `[E][purge class]` Purge raw prompt bytes. — I44:L16-L18
- **I-005/DII005-I44-006** [SRC] (DEEP-I-005.discovery.md:L340| unit=I-44): `[E][purge class]` Purge raw tool stdin and stdout. — I44:L16-L18
- **I-005/DII005-I44-007** [SRC] (DEEP-I-005.discovery.md:L341| unit=I-44): `[E][purge class]` Purge raw model input and output. — I44:L16-L18
- **I-005/DII005-I44-008** [SRC] (DEEP-I-005.discovery.md:L342| unit=I-44): `[E][granularity/constraint]` Because SSB is an append-only WAL with no row update/delete, retention purges whole expired session DB files using F-13's archive-then-purge primitive. — I44:L16-L18, I44:L31-L39, I44:L79, I44:L83, I44:L93
- **I-005/DII005-I44-009** [SRC] (DEEP-I-005.discovery.md:L343| unit=I-44): `[E][retained evidence]` Retain F-1 `source_blake3`, F-4 `stdout_hash`, and F-9 `dep_name_hash`. — I44:L19-L20
- **I-005/DII005-I44-010** [SRC] (DEEP-I-005.discovery.md:L344| unit=I-44): `[E][retained evidence]` Retain the G-2 evidence manifest and blake3 root. — I44:L19-L21, I44:L36-L37
- **I-005/DII005-I44-011** [SRC] (DEEP-I-005.discovery.md:L345| unit=I-44): `[E][retained evidence]` Retain the G-8 signed attestation/verdict. — I44:L20-L21, I44:L36-L37
- **I-005/DII005-I44-012** [SRC] (DEEP-I-005.discovery.md:L346| unit=I-44): `[E][retained evidence/privacy]` Retain only G-9-redacted summaries, with G-9 redaction enforced before purge. — I44:L20-L21, I44:L26-L27, I44:L32-L33, I44:L47
- **I-005/DII005-I44-013** [SRC] (DEEP-I-005.discovery.md:L347| unit=I-44): `[E][events]` Add SSB action type `RetentionPurged`, baseline +1, with `RawInputsPurged`, `EvidenceRetained`, and `PurgeVerified` arms. — I44:L22-L23, I44:L49-L50, I44:L74-L75, I44:L118
- **I-005/DII005-I44-014** [SRC] (DEEP-I-005.discovery.md:L348| unit=I-44): `[E][receipt schema]` The purge receipt records what was purged, what was retained, and a blake3 over the retained set. — I44:L22-L23, I44:L49-L50, I44:L97-L98
- **I-005/DII005-I44-015** [SRC] (DEEP-I-005.discovery.md:L349| unit=I-44): `[E][privacy invariant]` Post-purge grep of the exported bundle for raw-input markers must be empty. — I44:L24-L25, I44:L51-L52, I44:L100-L106
- **I-005/DII005-I44-016** [SRC] (DEEP-I-005.discovery.md:L350| unit=I-44 SURFACE): `[E][verification invariant]` `elai replay --verify` must still pass for hash-checkable steps after purge; verification cannot depend on purged bytes. — I44:L24-L26, I44:L51-L52, I44:L103-L107
- **I-005/DII005-I44-017** [SRC] (DEEP-I-005.discovery.md:L351| unit=I-44): `[E][constraint]` Never purge evidence hashes; zero retention removes raw source, not off-agent evidence. — I44:L27-L28, I44:L88-L90
- **I-005/DII005-I44-018** [SRC] (DEEP-I-005.discovery.md:L352| unit=I-44): `[E][safety/default]` Purge is destructive and irreversible, so it cannot be default-on and default behavior must remain unchanged. — I44:L28-L29, I44:L92
- **I-005/DII005-I44-019** [SRC] (DEEP-I-005.discovery.md:L353| unit=I-44): `[E][capability]` Create `crates/orchestrator/src/retention.rs::purge_after_verify` as the post-verdict driver. — I44:L45-L47, I44:L60-L64, I44:L71-L74
- **I-005/DII005-I44-020** [SRC] (DEEP-I-005.discovery.md:L354| unit=I-44 SURFACE): `[E][CLI]` Add `elai run --zero-retention` and check policy at the seal point. — I44:L48, I44:L84-L86
- **I-005/DII005-I44-021** [SRC] (DEEP-I-005.discovery.md:L355| unit=I-44): `[E][export integration]` I-40 wrapper export checks `RetentionPolicy` before emitting a purge-eligible bundle. — I44:L40-L41, I44:L82, I44:L86
- **I-005/DII005-I44-022** [SRC] (DEEP-I-005.discovery.md:L356| unit=I-44 SURFACE): `[E][reachability]` Production reachability is the zero-retention CLI/run seal path and I-40 export path; test-only callers do not satisfy R7. — I44:L84-L87
- **I-005/DII005-I44-023** [SRC] (DEEP-I-005.discovery.md:L357| unit=I-44): `[E][test]` Create `crates/contracts/tests/i44_zero_retention_verify_survives_contract.rs`. — I44:L51-L52, I44:L65, I44:L76
- **I-005/DII005-I44-024** [SRC] (DEEP-I-005.discovery.md:L358| unit=I-44): `[E][artifact]` Create `workflow/reasoning/I-44-decisions.md`. — I44:L66
- **I-005/DII005-I44-025** [SRC] (DEEP-I-005.discovery.md:L359| unit=I-44 SURFACE): `[E][surface]` Render receipts in the existing RRA viewer/session-retention drawer, exposing purge/retained classes and retained-root blake3. — I44:L67-L68, I44:L96-L98
- **I-005/DII005-I44-026** [SRC] (DEEP-I-005.discovery.md:L360| unit=I-44): `[E][acceptance/evidence]` A planted-secret fixture must produce all three receipt arms and an export with neither raw markers nor the secret in retained summaries. — I44:L99-L106
- **I-005/DII005-I44-027** [SRC] (DEEP-I-005.discovery.md:L361| unit=I-44): `[E][determinism]` Full and ZeroAfterVerify runs over the same fixture differ only in raw-byte presence; evidence and verdict bytes are identical. — I44:L106-L107, I44:L114-L115
- **I-005/DII005-I44-028** [SRC] (DEEP-I-005.discovery.md:L362| unit=I-44): `[E][deferred/non-goal]` SOC 2 is process/audit work outside engineering scope. — I44:L11, I44:L54-L55, I44:L116-L117
- **I-005/DII005-I44-029** [SRC] (DEEP-I-005.discovery.md:L363| unit=I-44): `[E][deferred]` Configurable retention TTL (N days rather than immediate post-verdict) is deferred. — I44:L56
- **I-005/DII005-I44-030** [SRC] (DEEP-I-005.discovery.md:L364| unit=I-44): `[E][deferred]` Self-host packaging/distribution is deferred as a business decision; I-40 already provides the local wrapper. — I44:L57-L58
- **I-005/DII005-I44-031** [SRC] (DEEP-I-005.discovery.md:L365| unit=I-44): `[E][composition]` Compose, never reimplement, G-9 redaction, F-13 session archive, G-2 RRA, and G-8 attestation. — I44:L63-L64, I44:L77-L83, I44:L94, I44:L120-L124
- **I-005/DII005-I44-032** [SRC] (DEEP-I-005.discovery.md:L366| unit=I-44): `[E][safety]` Production paths ban `.unwrap()`/`.expect()` and use blake3, not blake2/sha2. — I44:L95

#### I-005 / unit `I-51` (28 records)

- **I-005/DII005-I51-001** [SRC] (DEEP-I-005.discovery.md:L370| unit=I-51): `[E][status]` I-51 is Track 20, `SMARTER-ORCH`, `STRONG (SL-4)`, `EXTENSION/consolidation`, effort S–M, Wave 1. — I51:L1-L4
- **I-005/DII005-I51-002** [SRC] (DEEP-I-005.discovery.md:L371| unit=I-51): `[E][problem]` Model metadata is scattered across provider pricing, two `vram_tier.rs` modules, pricing rollup, and adapter defaults, causing duplicated/hard-coded derivations. — I51:L6-L12, I51:L27-L31
- **I-005/DII005-I51-003** [SRC] (DEEP-I-005.discovery.md:L372| unit=I-51): `[E][capability]` Create `crates/providers/src/model_catalog.rs` plus one repo-artifact `models.toml` keyed by model ID. — I51:L14-L18, I51:L33-L39, I51:L43-L44
- **I-005/DII005-I51-004** [SRC] (DEEP-I-005.discovery.md:L373| unit=I-51): `[E][schema field]` Catalog entries carry `context_window`. — I51:L10-L12
- **I-005/DII005-I51-005** [SRC] (DEEP-I-005.discovery.md:L374| unit=I-51): `[E][schema field]` Catalog entries carry input pricing. — I51:L10-L12
- **I-005/DII005-I51-006** [SRC] (DEEP-I-005.discovery.md:L375| unit=I-51): `[E][schema field]` Catalog entries carry output pricing. — I51:L10-L12
- **I-005/DII005-I51-007** [SRC] (DEEP-I-005.discovery.md:L376| unit=I-51): `[E][schema field]` Catalog entries carry cache pricing. — I51:L10-L12
- **I-005/DII005-I51-008** [SRC] (DEEP-I-005.discovery.md:L377| unit=I-51): `[E][schema field]` Catalog entries carry thinking support. — I51:L10-L12
- **I-005/DII005-I51-009** [SRC] (DEEP-I-005.discovery.md:L378| unit=I-51): `[E][schema field]` Catalog entries carry thinking-effort mapping. — I51:L10-L12
- **I-005/DII005-I51-010** [SRC] (DEEP-I-005.discovery.md:L379| unit=I-51): `[E][schema field]` Catalog entries carry default `max_tokens`. — I51:L10-L12
- **I-005/DII005-I51-011** [SRC] (DEEP-I-005.discovery.md:L380| unit=I-51): `[E][schema field]` Catalog entries carry VRAM tier. — I51:L10-L12
- **I-005/DII005-I51-012** [SRC] (DEEP-I-005.discovery.md:L381| unit=I-51): `[E][freshness]` Every catalog entry has a dated staleness stamp so pricing can be reverified at edit time. — I51:L17-L21, I51:L60-L62
- **I-005/DII005-I51-013** [SRC] (DEEP-I-005.discovery.md:L382| unit=I-51): `[E][security/config]` Gate-relevant `models.toml` fields are diffable repo data and R8-frozen. — I51:L17-L18
- **I-005/DII005-I51-014** [SRC] (DEEP-I-005.discovery.md:L383| unit=I-51): `[E][migration]` Preserve `pricing.rs` public APIs but turn their data implementation into a catalog view. — I51:L18-L20, I51:L34-L35, I51:L39, I51:L45-L46
- **I-005/DII005-I51-015** [SRC] (DEEP-I-005.discovery.md:L384| unit=I-51): `[E][migration]` Preserve both `vram_tier.rs` public APIs but re-source them as catalog views. — I51:L18-L20, I51:L34-L35, I51:L39, I51:L45-L46
- **I-005/DII005-I51-016** [SRC] (DEEP-I-005.discovery.md:L385| unit=I-51): `[E][compatibility]` Existing call sites must continue compiling against preserved APIs. — I51:L19-L20, I51:L47-L48, I51:L50-L51
- **I-005/DII005-I51-017** [SRC] (DEEP-I-005.discovery.md:L386| unit=I-51): `[E][constraint]` Wrap and preserve the measured 8/12/16/24/48GB VRAM-tier semantics; deletion requires explicit user confirmation. — I51:L22-L23, I51:L49
- **I-005/DII005-I51-018** [SRC] (DEEP-I-005.discovery.md:L387| unit=I-51): `[E][constraint]` The catalog performs no live price fetch. — I51:L24-L25, I51:L49-L50
- **I-005/DII005-I51-019** [SRC] (DEEP-I-005.discovery.md:L388| unit=I-51): `[E][non-goal]` I-51 makes no router changes and adds/removes no models. — I51:L24-L25
- **I-005/DII005-I51-020** [SRC] (DEEP-I-005.discovery.md:L389| unit=I-51): `[E][deferred]` Migrating every per-adapter default is deferred to follow-on patch work after the catalog is proven. — I51:L33-L36, I51:L66
- **I-005/DII005-I51-021** [SRC] (DEEP-I-005.discovery.md:L390| unit=I-51): `[E][integration]` `pricing_rollup` and existing cost/tier consumers read through preserved views; I-10 and I-25 are future clean-input consumers. — I51:L30-L31, I51:L45-L48, I51:L64-L66
- **I-005/DII005-I51-022** [SRC] (DEEP-I-005.discovery.md:L391| unit=I-51 SURFACE): `[E][surface]` No UI is added; existing cost and tier surfaces remain unchanged. — I51:L38-L41
- **I-005/DII005-I51-023** [SRC] (DEEP-I-005.discovery.md:L392| unit=I-51): `[E][events]` Add no SSB action type; existing cost/route rows carry catalog-derived values. — I51:L52-L55, I51:L67
- **I-005/DII005-I51-024** [SRC] (DEEP-I-005.discovery.md:L393| unit=I-51): `[E][behavioral invariant]` Fixture cost and tier output must be byte-identical before and after consolidation. — I51:L52-L58
- **I-005/DII005-I51-025** [SRC] (DEEP-I-005.discovery.md:L394| unit=I-51): `[E][test]` A drift contract requires the two VRAM-tier modules to agree byte-for-byte through the catalog and fails on seeded view divergence. — I51:L34-L35, I51:L43-L44, I51:L56-L58
- **I-005/DII005-I51-026** [SRC] (DEEP-I-005.discovery.md:L395| unit=I-51): `[E][test]` Grep must find zero hard-coded price constants outside `models.toml` in migrated modules. — I51:L49-L50, I51:L57-L58
- **I-005/DII005-I51-027** [SRC] (DEEP-I-005.discovery.md:L396| unit=I-51): `[E][dependency]` Before execution, read the gajae `api-registry.ts`, `models.json`, and `model-manager.ts` catalog shape under SL-10. — I51:L14-L16
- **I-005/DII005-I51-028** [SRC] (DEEP-I-005.discovery.md:L397| unit=I-51): `[E][safety]` Production paths ban `unwrap`/`expect` and API breakage. — I51:L49-L51

#### I-005 / unit `RELATIONAL` (47 records)

- **I-005/DII005-INF-001** [INF] (DEEP-I-005.discovery.md:L401| unit=RELATIONAL): `[I][ordering]` I-6 cannot enforce acknowledgement tags until I-1's supervisor gate exists, so same-wave implementation needs I-1 before the I-6 enforcement slice even though both are labeled Wave 2. — inferred from I6:L33-L34, I6:L41-L42
- **I-005/DII005-INF-002** [INF] (DEEP-I-005.discovery.md:L402| unit=RELATIONAL): `[I][routing cardinality]` With multi-hop deferred, the Tier-1 I-7 failure path appears intended to make one synchronous cloud-architect call, but the original never states a numeric “exactly one” acceptance assertion. — inferred from I7:L17-L20, I7:L51-L53, I7:L91-L97
- **I-005/DII005-INF-003** [INF] (DEEP-I-005.discovery.md:L403| unit=RELATIONAL): `[I][context ordering]` Because I-14 transforms tool-result volatile-tail content before context assembly and I-6 appends a frozen final anchor, the intended composed order appears to keep `AnchorPane` outside compression. No assigned source states that cross-feature order directly. — inferred from I14:L40, I14:L65-L73 and I6:L12-L18
- **I-005/DII005-INF-004** [INF] (DEEP-I-005.discovery.md:L404| unit=RELATIONAL): `[I][routing layers]` I-25 preclassification and I-7 post-local-failure escalation are distinct routing stages and can compose without being alternate implementations: I-25 acts pre-cloud, while I-7 acts after all local branches fail. — inferred from I25:L29-L31, I25:L80-L82 and I7:L15-L20
- **I-005/DII005-INF-005** [INF] (DEEP-I-005.discovery.md:L405| unit=RELATIONAL): `[I][promotion pattern]` I-25 router artifacts and I-31 checkpoints share the same architectural rule: evaluators/trainers do not own the frozen promotion writer, and only an IS-16 ledger win can activate an artifact. — inferred from I25:L31-L36, I25:L49-L50, I25:L155-L156 and I31:L32-L37, I31:L99-L101, I31:L141-L142
- **I-005/DII005-INF-006** [INF] (DEEP-I-005.discovery.md:L406| unit=RELATIONAL): `[I][future integration]` I-51 can eventually normalize pricing/VRAM inputs consumed by I-25, but I-25 does not currently declare `model_catalog` as a direct dependency; this is future composition, not present wiring. — inferred from I51:L64-L66 and I25:L73-L79
- **I-005/DII005-INF-007** [INF] (DEEP-I-005.discovery.md:L407| unit=RELATIONAL): `[I][journal lifecycle]` I-39's plan status can use new action rows from the other sessions only after their `observable_effect` events exist; it does not confer DONE merely because a plan file exists. — inferred from I39:L14-L21
- **I-005/DII005-INF-008** [INF] (DEEP-I-005.discovery.md:L408| unit=RELATIONAL): `[I][retention/replay]` Under I-44 `ZeroAfterVerify`, I-14 reverse pointers and I-11 exact-received-payload replay may become hash-verifiable but non-reexpandable once their raw backing bytes are purged. The sources do not define an exemption. — inferred from I14:L22-L23, I11:L17-L18, and I44:L16-L26
- **I-005/DII005-INF-009** [INF] (DEEP-I-005.discovery.md:L409| unit=RELATIONAL): `[I][behavior-neutral scope]` I-51's byte-identical claim applies to consolidation against a fixed catalog snapshot; a later reviewed pricing/staleness update would intentionally change cost output. — inferred from I51:L20-L21, I51:L52-L58
- **I-005/DII005-AMB-001** [AC] (DEEP-I-005.discovery.md:L413| unit=RELATIONAL SURFACE): `[A][I6 protocol]` The acknowledgement tag's exact grammar and the size of the “first reply token window” are unspecified. — I6:L15-L16, I6:L41-L42, I6:L84-L87
- **I-005/DII005-AMB-002** [AC] (DEEP-I-005.discovery.md:L414| unit=RELATIONAL): `[A][I7 config]` K (local branch width) and the precise tokenizer/rounding semantics behind “≤~2K tokens” are not specified. — I7:L15-L18, I7:L84-L85
- **I-005/DII005-AMB-003** [AC] (DEEP-I-005.discovery.md:L415| unit=RELATIONAL): `[A][I11 scheduling]` The honesty job says “periodic” and defaults to ten escalations, but persistence, scheduler ownership, and behavior across restarts are unspecified. — I11:L35-L36, I11:L45-L46
- **I-005/DII005-AMB-004** [AC] (DEEP-I-005.discovery.md:L416| unit=RELATIONAL): `[A][I11 failure recovery]` A DKG must be reachable at decode time, but the typed failure/fallback when it is unavailable is not defined. — I11:L88
- **I-005/DII005-AMB-005** [AC] (DEEP-I-005.discovery.md:L417| unit=RELATIONAL): `[A][I14 storage]` The receipt points to original bytes by blake3, but storage location, retention duration, and authorization for re-expansion are unspecified. — I14:L22-L23, I14:L38, I14:L81-L82
- **I-005/DII005-AMB-006** [AC] (DEEP-I-005.discovery.md:L418| unit=RELATIONAL): `[A][I14 event typing]` Numeric-quantization Tier 2 survives, but the source does not say whether it uses generic `Tier2Compressed` or a distinct discriminator. — I14:L17-L23, I14:L38-L39
- **I-005/DII005-AMB-007** [AC] (DEEP-I-005.discovery.md:L419| unit=RELATIONAL): `[A][I22 parameters]` The source deliberately leaves weight values, D, decay factor, and the threshold that triggers sibling backtracking to measurement; their config schemas/ranges are not defined. — I22:L16-L17, I22:L25-L28, I22:L49-L50, I22:L93-L95
- **I-005/DII005-AMB-008** [AC] (DEEP-I-005.discovery.md:L420| unit=RELATIONAL): `[A][I22 fallback field]` The “wiring contract fallback” for requirements coverage is named but no exact field identifier/schema is given. — I22:L14-L15
- **I-005/DII005-AMB-009** [AC] (DEEP-I-005.discovery.md:L421| unit=RELATIONAL SURFACE): `[A][I25 enable flag]` `ELAI_ROUTE_DECISION_SSB_ENABLE` explicitly gates event emission, while the wargame also mentions a default-off profiler config without naming whether it is the same or a separate switch. — I25:L27-L28, I25:L80-L82, I25:L140-L143
- **I-005/DII005-AMB-010** [AC] (DEEP-I-005.discovery.md:L422| unit=RELATIONAL): `[A][I25 fingerprint]` The exact task-shape fingerprint schema, expensive-match threshold, cache lifecycle, and model-artifact TOML schema are not specified. — I25:L18-L20, I25:L40-L48
- **I-005/DII005-AMB-011** [AC] (DEEP-I-005.discovery.md:L423| unit=RELATIONAL): `[A][I31 canary event]` `ScorerNeutralized` is required on canary divergence, but the source neither registers it as a new action type nor identifies an existing owning event family. — I31:L25-L27, I31:L66-L74, I31:L134
- **I-005/DII005-AMB-012** [AC] (DEEP-I-005.discovery.md:L424| unit=RELATIONAL): `[A][I31 endpoint protocol]` The mechanism section permits typed local gRPC/HTTP, while revised Tier 1 names Axum; the revised scope strongly implies HTTP, but the external contract is not stated. — I31:L15-L17, I31:L40-L43, I31:L68-L69
- **I-005/DII005-AMB-013** [AC] (DEEP-I-005.discovery.md:L425| unit=RELATIONAL): `[A][I31 allowlist]` Coverage by the existing `darwinian-evolve` row must be verified and may require a new R9 promotion row, so “existing row covers the lane” is not yet established. — I31:L64, I31:L85, I31:L144
- **I-005/DII005-AMB-014** [AC] (DEEP-I-005.discovery.md:L426| unit=RELATIONAL): `[A][I39 precedence]` When static-check, journal, and fragment evidence disagree, no precedence rule for the four `PlanState` values is specified. — I39:L14-L21, I39:L37
- **I-005/DII005-AMB-015** [AC] (DEEP-I-005.discovery.md:L427| unit=RELATIONAL): `[A][I39 fixture]` N and the exact repo-relative archived database path are left to implementation-time re-grep. — I39:L23-L26, I39:L31-L33, I39:L40, I39:L51-L52
- **I-005/DII005-AMB-016** [AC] (DEEP-I-005.discovery.md:L428| unit=RELATIONAL): `[A][I44 deletion boundary]` “Archive-then-purge” does not state whether the archive itself is immediately deleted, encrypted, moved out of the export, or retained under another policy. — I44:L16-L18, I44:L34-L35, I44:L79
- **I-005/DII005-AMB-017** [AC] (DEEP-I-005.discovery.md:L429| unit=RELATIONAL): `[A][I44 erasure claim]` Acceptance proves absence from the exported bundle and purges session DB files, but does not define deletion guarantees for backups, filesystem snapshots, caches, or remanence. — I44:L22-L26, I44:L100-L107
- **I-005/DII005-AMB-018** [AC] (DEEP-I-005.discovery.md:L430| unit=RELATIONAL): `[A][I51 artifact path]` The source names `models.toml` but does not give its exact repository path relative to `crates/providers/src/model_catalog.rs`. — I51:L17-L18, I51:L38-L39
- **I-005/DII005-AMB-019** [AC] (DEEP-I-005.discovery.md:L431| unit=RELATIONAL): `[A][I51 governance]` Gate-relevant catalog fields are R8-frozen, but the authorized update/reverification workflow is not specified. — I51:L17-L21
- **I-005/DII005-AMB-020** [AC] (DEEP-I-005.discovery.md:L432| unit=RELATIONAL): `[A][I51 test symbol]` A duplication-drift contract is required, but its exact test file and symbol are not named. — I51:L34-L35, I51:L43-L44
- **I-005/DII005-CON-001** [CON] (DEEP-I-005.discovery.md:L436| unit=RELATIONAL): `[C][I6 identifier]` Summary `I-6.01` says `PrivilegedAnchor`; the original's exact and repeated identifier is `ContextSlot::AnchorPane`. Replace the summary name. — summary:L225; I6:L25-L27, I6:L38, I6:L59-L61
- **I-005/DII005-CON-002** [CON] (DEEP-I-005.discovery.md:L437| unit=RELATIONAL SURFACE): `[C][I6 protocol]` Summary `I-6.06` loses “highest-priority,” supervisor validation, first-token-window, and retry-on-missing semantics. It must be narrowed/expanded to the original contract. — summary:L230-L231; I6:L15-L16, I6:L41-L42
- **I-005/DII005-CON-003** [CON] (DEEP-I-005.discovery.md:L438| unit=RELATIONAL SURFACE): `[C][I7 UI status]` Summary `I-7.09` presents the cost-cockpit trace as an active feature, but the revised original explicitly defers it until the GD-4 bound is measured. — summary:L253; I7:L51-L54, I7:L87-L90
- **I-005/DII005-CON-004** [CON] (DEEP-I-005.discovery.md:L439| unit=RELATIONAL): `[C][I7 cardinality]` Summary `I-7.11` asserts a local-failure fixture escalates “exactly once”; the original specifies a single-hop Tier-1 sequence and defers multi-hop but never states that numeric acceptance assertion. Mark it inferred or restate as `Condensed` then `Escalated`. — summary:L255; I7:L17-L20, I7:L51-L53, I7:L95-L97
- **I-005/DII005-CON-005** [CON] (DEEP-I-005.discovery.md:L440| unit=RELATIONAL): `[C][I11 receipt wording]` Summary `I-11.10` says every handoff is an ID-3 receipt; the original says every condensation emits the receipt and separately emits `Handoff` when dispatched. — summary:L347; I11:L17-L18, I11:L35, I11:L73-L75
- **I-005/DII005-CON-006** [CON] (DEEP-I-005.discovery.md:L441| unit=RELATIONAL): `[C][I14 rollout]` Summary `I-14.06` reads as though Tier-2 token classification ships, but the revised original defers it behind dependency approval, prose-only, and VRAM gates. — summary:L423-L424; I14:L43-L45, I14:L71-L72, I14:L91
- **I-005/DII005-CON-007** [CON] (DEEP-I-005.discovery.md:L442| unit=RELATIONAL): `[C][I22 PPM semantics]` Summary `I-22.12` can be read as shaping the D-2 PPM score; the original explicitly says I-22 shapes the L2 input, not PPM output. — summary:L642; I22:L35-L36, I22:L73
- **I-005/DII005-CON-008** [CON] (DEEP-I-005.discovery.md:L443| unit=RELATIONAL): `[C][I25 live seam]` Summary `I-25.11` points to `escalation_policy/cloud_mode`; the ratified source explicitly moves Slice 1 to `task_runner.rs::run_cost_router_preclassify` and warns against the former/dark hook. — summary:L703; I25:L25-L32, I25:L74-L82
- **I-005/DII005-CON-009** [CON] (DEEP-I-005.discovery.md:L444| unit=RELATIONAL SURFACE): `[C][I25 UI status]` Summary `I-25.12` says route explanations are shown in the cockpit; the source defers all dashboard/Svelte work under `owner-ui-rework-hold`. — summary:L704; I25:L52-L56, I25:L92-L93
- **I-005/DII005-CON-010** [CON] (DEEP-I-005.discovery.md:L445| unit=RELATIONAL): `[C][I25 event payload]` Summary `I-25.13` says `RouteDecisionEvent` contains predicted-versus-actual outcome; the source's live payload keys are only `feature_vec`, `predicted_lane`, and `reason`. Actual verifier outcome is joined offline by the trainer. — summary:L705-L706; I25:L20-L21, I25:L43-L48, I25:L76
- **I-005/DII005-CON-011** [CON] (DEEP-I-005.discovery.md:L446| unit=RELATIONAL SURFACE): `[C][I25 run count]` Summary `I-25.14` requires at least three runs; the original requires an ELAI mixed-batch comparison but gives no three-run minimum. — summary:L707-L708; I25:L15-L18, I25:L100-L105
- **I-005/DII005-CON-012** [CON] (DEEP-I-005.discovery.md:L447| unit=RELATIONAL): `[C][I25 loss record]` Summary `I-25.15` asserts an “honest loss” record; the original specifies an eval report and no promotion/load for a loser, but no event or ledger row with that name. — summary:L709; I25:L47-L50, I25:L98, I25:L144-L146
- **I-005/DII005-CON-013** [CON] (DEEP-I-005.discovery.md:L448| unit=RELATIONAL): `[C][I25 blocked ledger]` Summary `I-25.16` says the losing-router block is ledgered. The source only requires promotion to depend on a ledger win and the loser path to remain absent; the Python trainer expressly cannot write ledger rows. — summary:L710; I25:L29-L32, I25:L49-L50, I25:L155-L156
- **I-005/DII005-CON-014** [CON] (DEEP-I-005.discovery.md:L449| unit=RELATIONAL): `[C][I31 allowlist certainty]` Summary `I-31.19` states the existing allowlist definitely covers GRPO; the source requires coverage verification and allows adding an R9 promotion row if it does not. — summary:L850-L851; I31:L64, I31:L85
- **I-005/DII005-CON-015** [CON] (DEEP-I-005.discovery.md:L450| unit=RELATIONAL SURFACE): `[C][I39 event trigger]` Summary `I-39.15` attaches `PlanStateDerived` to successful archive replay; the original attaches it to the production `elai plan status` query and says migration replay reuses existing rows. — summary:L1074; I39:L39, I39:L59, I39:L67-L70
- **I-005/DII005-CON-016** [CON] (DEEP-I-005.discovery.md:L451| unit=RELATIONAL): `[C][I39 five-source ambiguity]` Summary `I-39.17` calls the five drifting sources unenumerated, but the assigned original enumerates all five explicitly. — summary:L1076-L1077; I39:L5-L7
- **I-005/DII005-CON-017** [CON] (DEEP-I-005.discovery.md:L452| unit=RELATIONAL): `[C][I44 granularity]` Summary `I-44.07` can imply each raw input is a separate file; the original's enforceable unit is the whole expired session DB because row deletion is forbidden. — summary:L1213-L1214; I44:L16-L18, I44:L38-L39, I44:L93
- **I-005/DII005-CON-018** [CON] (DEEP-I-005.discovery.md:L453| unit=RELATIONAL): `[C][I51 current consumers]` Summary `I-51.17` includes runtime defaults as though migrated now; the original defers per-adapter defaults and names I-10/I-25 as future consumers. — summary:L1401; I51:L24-L25, I51:L33-L36, I51:L64-L66

### Ledger I-006 — I-16, I-17, I-19, I-24, I-26, I-30, I-43, I-47, I-49, I-53

Source: `docs/thefinalglance/feature-discovery/deep/DEEP-I-006.discovery.md`
Extracted atoms: **454**

#### I-006 / unit `I-16` (40 records)

- **I-006/I16-E001** [SRC] (DEEP-I-006.discovery.md:L35| unit=I-16): E/META | Track 14 `CACHE-INVARIANT ECONOMICS MIDDLEWARE`; lever `LOCAL-VRAM`; moderate grounding; `ACTIVATION+EVAL`; effort M; Wave 5; live leg RunPod-gated; binds ID-5 and IC-7. | anchor=I-16:L3
- **I-006/I16-C001** [CON] (DEEP-I-006.discovery.md:L36| unit=I-16): C/STATUS | Ratified correction: fp8/fp16 `--kv-cache-dtype`, Int4-class `--turboquant-bits`, `KVCompressor`/`_KVQuantCache`, and the `ELAI_STREAMING_LLM_SINKS` sink mask already ship; I-16 is an evaluation layer, not a new-lever implementation. | anchor=I-16:L4-L12
- **I-006/I16-E002** [SRC] (DEEP-I-006.discovery.md:L37| unit=I-16 SURFACE): E/GAP | The absent surfaces are an ELAI-fixture evaluator, a `KvCacheTuned` decision row, and the `elai bench kv-tune` entry point. | anchor=I-16:L6-L8
- **I-006/I16-E003** [SRC] (DEEP-I-006.discovery.md:L38| unit=I-16): E/SCOPE | Tier 1 is a deterministic no-GPU dry-run evaluator; it neither adds duplicate pruning/quant flags nor changes serving defaults. | anchor=I-16:L8-L10, L30-L36
- **I-006/I16-E004** [SRC] (DEEP-I-006.discovery.md:L39| unit=I-16): E/DEFER | True SnapKV/ThinK-class eviction and new eviction flags remain Tier 2 because `position_ids` re-indexing is missing. | anchor=I-16:L10-L10, L37-L40
- **I-006/I16-E005** [SRC] (DEEP-I-006.discovery.md:L40| unit=I-16): E/PROMOTION | MoE retargeting is a separate IC-7 checkbox; live adoption requires an ELAI-measured pass-rate band plus lower VRAM. | anchor=I-16:L11-L12, L37-L40, L107-L109
- **I-006/I16-E006** [SRC] (DEEP-I-006.discovery.md:L41| unit=I-16): E/CFG | The dry-run matrix is `turboquant_bits ∈ {0,4,8}` × `kv_cache_dtype ∈ {fp16,fp8_e4m3,fp8_e5m2}` × `sinks ∈ {0,4}`: 18 explicit configurations. | anchor=I-16:L14-L18
- **I-006/I16-E007** [SRC] (DEEP-I-006.discovery.md:L42| unit=I-16): E/FLOW | Each dry-run configuration instantiates the real `KVCompressor`, FA2 auto-detect, and `LogitsProcessor` assembly paths without booting a GPU model. | anchor=I-16:L16-L20
- **I-006/I16-E008** [SRC] (DEEP-I-006.discovery.md:L43| unit=I-16 SURFACE): E/EVIDENCE | SnapKV/ThinK paper results remain design evidence only; ELAI measurements control promotion, while W-10 requires the deterministic harness to ship even if the RunPod leg does not. | anchor=I-16:L18-L20, L64-L66, L83-L85
- **I-006/I16-E009** [SRC] (DEEP-I-006.discovery.md:L44| unit=I-16 SURFACE): E/DEP | The evaluator composes E-26/E-27/E-28 serving and FA2/`LogitsProcessor`, E-32 context-lift, B-37 transport for the live leg, and the `BenchLongcot` CLI/fixture pattern. | anchor=I-16:L22-L28, L79-L91
- **I-006/I16-E010** [SRC] (DEEP-I-006.discovery.md:L45| unit=I-16): E/ARTIFACT | `python/elai_inference/eval_kv_tune.py::KvTuneEval` sweeps shipped levers, writes `metrics/kv_tune_<blake3>.toml`, and rejects an FA2-disabled configuration with a typed error. | anchor=I-16:L30-L34, L43-L44, L49-L53
- **I-006/I16-E011** [SRC] (DEEP-I-006.discovery.md:L46| unit=I-16): E/SCHEMA | New flat SSB beacon `KvCacheTuned` carries `vram_gb`, `tok_s`, `fixture_pass_rate`, `config_hash`, and `adopt_decision`. | anchor=I-16:L34-L35, L49-L53
- **I-006/I16-E012** [SRC] (DEEP-I-006.discovery.md:L47| unit=I-16 SURFACE): E/CLI | Production CLI wiring is `CliAction::BenchKvTune`, `bench_kv_tune.rs`, and the `main.rs` dispatch arm; the command supports dry-run and `--emit-ssb`. | anchor=I-16:L35-L35, L43-L45, L54-L55
- **I-006/I16-E013** [SRC] (DEEP-I-006.discovery.md:L48| unit=I-16): E/DEFER | Live RunPod measurement and its Adopt/Reject decision, the MoE evaluation, and real eviction are independently gated Tier-2 work. | anchor=I-16:L37-L40
- **I-006/I16-E014** [SRC] (DEEP-I-006.discovery.md:L49| unit=I-16): E/NON-GOAL | `server.py` receives no new argparse flags; evaluation sweeps the already-shipped `--turboquant-bits`, `--kv-cache-dtype`, and sink controls. | anchor=I-16:L42-L45
- **I-006/I16-E015** [SRC] (DEEP-I-006.discovery.md:L50| unit=I-16): E/SCHEMA | The SSB action registry and payload must change in lockstep; `SCHEMA_VERSION` is derived from live `SSB_ACTION_TYPES.len()` and must not be hand-bumped or restated as a stale integer. | anchor=I-16:L46-L46, L52-L53, L83-L85, L107-L108
- **I-006/I16-E016** [SRC] (DEEP-I-006.discovery.md:L51| unit=I-16 SURFACE): E/UI | There is deliberately no frontend pair or dashboard: the product surface is the CLI plus metrics TOML. | anchor=I-16:L47-L47, L72-L73, L122-L124
- **I-006/I16-E017** [SRC] (DEEP-I-006.discovery.md:L52| unit=I-16 SURFACE): E/SYMBOL | The produced Python symbol is `python/elai_inference/eval_kv_tune.py::KvTuneEval`; the produced Rust/SSB symbols are the CLI dispatch and `KvCacheTuned`. | anchor=I-16:L49-L55
- **I-006/I16-E018** [SRC] (DEEP-I-006.discovery.md:L53| unit=I-16): E/INVARIANT | Every matrix arm consumes shipped server levers and asserts that the E-26 FA2 and E-28 `LogitsProcessor` paths remain active. | anchor=I-16:L56-L59, L95-L98
- **I-006/I16-E019** [SRC] (DEEP-I-006.discovery.md:L54| unit=I-16 SURFACE): E/REACH | Required production reachability is `cargo run -p elai-cli -- bench kv-tune --dry-run --emit-ssb` to the Python evaluator. | anchor=I-16:L60-L62, L74-L77
- **I-006/I16-E020** [SRC] (DEEP-I-006.discovery.md:L55| unit=I-16): E/DONE | Test-only callers do not satisfy R7 production reachability. | anchor=I-16:L61-L62
- **I-006/I16-E021** [SRC] (DEEP-I-006.discovery.md:L56| unit=I-16): E/LIMIT | Duplicate `--kv-pruning-budget`/`--kv-quant-bits` and any serving-default change without a live `KvCacheTuned(adopt_decision: Adopt)` are banned. | anchor=I-16:L63-L66
- **I-006/I16-E022** [SRC] (DEEP-I-006.discovery.md:L57| unit=I-16): E/LIMIT | Paper accuracy as a gate, assumed MoE retarget, FA2 disablement, and `LogitsProcessor` bypass are banned. | anchor=I-16:L66-L68
- **I-006/I16-E023** [SRC] (DEEP-I-006.discovery.md:L58| unit=I-16): E/SEC | `config_hash` uses blake3 only; dry-run must leave VRAM/throughput null rather than fabricate them; production parse paths may not use `.unwrap()`/`.expect()`. | anchor=I-16:L69-L71
- **I-006/I16-E024** [SRC] (DEEP-I-006.discovery.md:L59| unit=I-16 SURFACE): E/OBS | The no-GPU command must emit `KvCacheTuned{vram_gb:null,tok_s:null,fixture_pass_rate:1.0,config_hash:blake3(...),adopt_decision:DryRun}` and the matching metrics file; `Adopt` with null VRAM fails. | anchor=I-16:L74-L77, L93-L99
- **I-006/I16-E025** [SRC] (DEEP-I-006.discovery.md:L60| unit=I-16 SURFACE): E/RECOVERY | Missing beacon, absent metrics, or silent FA2 disablement diagnoses unwired evaluator/CLI dispatch; the counter-move is to re-read the full `BenchLongcot` dispatch and Pattern-2 SSB lockstep, then rerun smoke. | anchor=I-16:L100-L102
- **I-006/I16-E026** [SRC] (DEEP-I-006.discovery.md:L61| unit=I-16): E/FALLBACK | The default dry-run transport is subprocess `python -m elai_inference.eval_kv_tune --dry-run`; HTTP through `SidecarProvider` is deferred to the live leg. | anchor=I-16:L104-L105
- **I-006/I16-E027** [SRC] (DEEP-I-006.discovery.md:L62| unit=I-16): E/ABORT | Stop rather than invent schema integers if the live action registry cannot be reread; stop and retain subprocess transport if dry-run would otherwise require a booted model. | anchor=I-16:L107-L109
- **I-006/I16-E028** [SRC] (DEEP-I-006.discovery.md:L63| unit=I-16 SURFACE): E/VERIFY | Done gates include plan-gate check, `cargo check -p elai-cli --all-targets`, orchestrator `action_type_lock`, the production CLI smoke, and `production_caller_check` on new dispatch symbols. | anchor=I-16:L111-L116
- **I-006/I16-E029** [SRC] (DEEP-I-006.discovery.md:L64| unit=I-16 SURFACE): E/REDTEAM | The red-team arms require failure for an Adopt row lacking live VRAM/pass-rate measurement and ban a duplicate-flag implementation that replaces shipped levers. | anchor=I-16:L118-L120
- **I-006/I16-E030** [SRC] (DEEP-I-006.discovery.md:L65| unit=I-16): E/IDEMP | Clean-tree integration smoke produces exactly one DryRun beacon and one metrics artifact, with `config_hash` equal to blake3 of the relevant matrix entry. | anchor=I-16:L122-L124
- **I-006/I16-A001** [AC] (DEEP-I-006.discovery.md:L66| unit=I-16): A/THRESHOLD | The acceptable live pass-rate band and required VRAM decrease are not quantified. | anchor=I-16:L11-L12, L64-L66, L107-L109
- **I-006/I16-A002** [AC] (DEEP-I-006.discovery.md:L67| unit=I-16): A/DATA | The source defines an 18-arm matrix but the smoke requires exactly one row/file and a hash of “the matrix entry”; result aggregation, selected entry, and per-arm artifact cardinality are unspecified. | anchor=I-16:L16-L18, L95-L98, L122-L124
- **I-006/SD-I16-01** [SD] (DEEP-I-006.discovery.md:L529| unit=I-16): CORRECTION | Summary L462-L464 describes sink-protected pruning and pinning the first four tokens; the ratified source instead evaluates the shipped sink control across `sinks={0,4}` and does not mandate four sinks as the serving default. | anchor=I16-C001, I16-E003, I16-E006
- **I-006/SD-I16-02** [SD] (DEEP-I-006.discovery.md:L530| unit=I-16): CORRECTION | Summary L472 says I-16 adds sidecar flags. The source expressly bans duplicate pruning/quant flags because the relevant dtype/turboquant/sink levers already ship. | anchor=I16-C001, I16-E014, I16-E021
- **I-006/SD-I16-03** [SD] (DEEP-I-006.discovery.md:L531| unit=I-16): ADDITION | The exact 18-arm matrix, real-config-path dry run, metrics filename, null VRAM/throughput policy, `DryRun` decision, and FA2 typed refusal were omitted. | anchor=I16-E006–I16-E007, I16-E010, I16-E023–I16-E024
- **I-006/SD-I16-04** [SD] (DEEP-I-006.discovery.md:L532| unit=I-16 SURFACE): ADDITION | Exact CLI/module symbols, subprocess fallback, no-dashboard decision, schema auto-derivation, action-lock checks, and clean-tree cardinality gates were omitted. | anchor=I16-E012, I16-E015–I16-E020, I16-E026–I16-E030
- **I-006/SD-I16-05** [SD] (DEEP-I-006.discovery.md:L533| unit=I-16): ADDITION | True eviction is blocked on `position_ids` re-indexing; HTTP transport, live Adopt/Reject, and MoE remain distinct Tier-2 work. | anchor=I16-E004–I16-E005, I16-E013, I16-E026
- **I-006/SD-I16-06** [SD] (DEEP-I-006.discovery.md:L534| unit=I-16): UNSUPPORTED | Summary L468 requires comparison with an E-27 AWQ baseline; the deep source composes E-27 serving/parity patterns but contains no explicit AWQ-baseline comparison acceptance requirement. | anchor=I16-E009
- **I-006/SD-I16-07** [SD] (DEEP-I-006.discovery.md:L535| unit=I-16): ADDITION | The source exposes an unresolved 18-arm-versus-one-row/file aggregation contract beyond the summary’s already-noted missing pass-rate band. | anchor=I16-A001–I16-A002

#### I-006 / unit `I-17` (38 records)

- **I-006/I17-E001** [SRC] (DEEP-I-006.discovery.md:L77| unit=I-17): E/META | Track 15 `DETERMINISTIC GATES + ANTI-GAMING VERIFICATION`; `TRUST-PROVABLE`; strong grounding; composition build; effort M; Wave 3; ID-4; blocking for I-4; IC-11 explicitly does not bind. | anchor=I-17:L3-L3, L85-L88
- **I-006/I17-E002** [SRC] (DEEP-I-006.discovery.md:L78| unit=I-17): E/GAP | D-3/D-11/F-1/F-9/IS-8 substrate ships, but no single veto-first compositor exists; currently a hallucinated-import candidate can score nonzero before L1. | anchor=I-17:L4-L7, L24-L30
- **I-006/I17-E003** [SRC] (DEEP-I-006.discovery.md:L79| unit=I-17): E/CAP | L1 composes compile, tests, F-1 VibeGate, F-9 dependency gate, and D-11 checkers. | anchor=I-17:L9-L12, L34-L37
- **I-006/I17-E004** [SRC] (DEEP-I-006.discovery.md:L80| unit=I-17): E/INVARIANT | Any L1 failure is final score `0.0`; neither L2 nor L3 may rescore or resurrect the branch, and a soft `0.001` substitute is forbidden. | anchor=I-17:L10-L12, L21-L22, L70-L74
- **I-006/I17-E005** [SRC] (DEEP-I-006.discovery.md:L81| unit=I-17): E/FAIL | A deterministic check exceeding the empirical 15-second bound aborts the branch and incurs a `HaltingBound` penalty; the timer must be the common `halt::BoundTimer` shared with I-8. | anchor=I-17:L12-L14, L34-L38, L98-L100
- **I-006/I17-E006** [SRC] (DEEP-I-006.discovery.md:L82| unit=I-17): E/RELIABILITY | Oracles are rerun under seed variation; a result flip creates an SSB-backed flake quarantine keyed by `blake3(oracle-id + seed-range)`. | anchor=I-17:L14-L15, L34-L39, L55-L58
- **I-006/I17-E007** [SRC] (DEEP-I-006.discovery.md:L83| unit=I-17): E/LIFECYCLE | A quarantined oracle is barred from both vetoing and vouching until rehabilitation; vote exclusion is a hard gate checked before every oracle read. | anchor=I-17:L14-L16, L70-L73
- **I-006/I17-E008** [SRC] (DEEP-I-006.discovery.md:L84| unit=I-17): E/FLOW | L2 runs only on L1 survivors, using D-2 PPM plus I-23 unified conformal uncertainty; until I-23 lands it may call raw `conformal_calibration` through a registered stub, while E-19 PRM-lite is the pre-PPM scorer. | anchor=I-17:L15-L17, L28-L30, L38-L40, L64-L65
- **I-006/I17-E009** [SRC] (DEEP-I-006.discovery.md:L85| unit=I-17): E/ESCALATION | L3 fires when survivor L2 variance exceeds a threshold and routes to D-13 for a concrete counterexample. | anchor=I-17:L17-L18, L40-L41, L65-L65
- **I-006/I17-E010** [SRC] (DEEP-I-006.discovery.md:L86| unit=I-17): E/DEFER | The D-3 seam for TypeScript/Python verifier packs ships now, but the packs are Tier 2 and require per-language harness confirmation plus a W-10 checkbox. | anchor=I-17:L19-L20, L42-L44
- **I-006/I17-E011** [SRC] (DEEP-I-006.discovery.md:L87| unit=I-17): E/NON-GOAL | L2-on-veto, soft veto, and absorption/reimplementation of D-3/D-11/F-1/F-9 are explicitly excluded. | anchor=I-17:L21-L22, L70-L75
- **I-006/I17-E012** [SRC] (DEEP-I-006.discovery.md:L88| unit=I-17): E/STATUS | D-3, D-11, F-1, F-9, IS-8, IS-7, and D-13 are asserted shipped; exact dispatch/hook locations remain execution-time W-7 re-grep obligations. | anchor=I-17:L24-L32
- **I-006/I17-E013** [SRC] (DEEP-I-006.discovery.md:L89| unit=I-17): E/API | New `crates/orchestrator/src/verifier_stack.rs::TriLayerStack` exposes `run_verdict(&Branch) -> VerdictRow`. | anchor=I-17:L46-L50, L54-L57
- **I-006/I17-E014** [SRC] (DEEP-I-006.discovery.md:L90| unit=I-17): E/API | `FlakeQuarantineLedger` provides SSB read/write operations; its stable identity is the blake3 oracle/seed-range key. | anchor=I-17:L54-L58, L75-L75
- **I-006/I17-E015** [SRC] (DEEP-I-006.discovery.md:L91| unit=I-17): E/EVENT | `TriLayerVerdict` kinds are exactly `HardVeto`, `ScoredSurvivor`, `DisagreementEscalated`, and `FlakeQuarantined`. | anchor=I-17:L41-L41, L49-L50, L58-L58
- **I-006/I17-E016** [SRC] (DEEP-I-006.discovery.md:L92| unit=I-17): E/INTEG | The compositor calls `ConsensusVerifier`, `run_all_checkers`, `VibeGate::evaluate`, and `run_dep_gate`; it must not copy their logic. | anchor=I-17:L59-L64, L94-L99
- **I-006/I17-E017** [SRC] (DEEP-I-006.discovery.md:L93| unit=I-17): E/INTEG | It consumes I-23/raw calibration for uncertainty and `crates/orchestrator/src/adversary.rs` for L3. | anchor=I-17:L64-L65, L97-L100
- **I-006/I17-E018** [SRC] (DEEP-I-006.discovery.md:L94| unit=I-17): E/REACH | I-4 `search/ladder.rs::rank_candidates` invokes one verdict per branch on the production search path. | anchor=I-17:L66-L68
- **I-006/I17-E019** [SRC] (DEEP-I-006.discovery.md:L95| unit=I-17): E/REACH | The existing `task_runner.rs` baby-step audit dispatch is the second production caller and replaces inline gate calls. | anchor=I-17:L47-L49, L67-L69
- **I-006/I17-E020** [SRC] (DEEP-I-006.discovery.md:L96| unit=I-17): E/DONE | A `#[cfg(test)]` or contracts-only caller never satisfies R7; both production chains must be retraced at execution. | anchor=I-17:L67-L69
- **I-006/I17-E021** [SRC] (DEEP-I-006.discovery.md:L97| unit=I-17): E/SEC | A hard-vetoed branch may trigger no L2/L3 call, and a quarantined oracle may participate in no vote. | anchor=I-17:L70-L73
- **I-006/I17-E022** [SRC] (DEEP-I-006.discovery.md:L98| unit=I-17): E/LIMIT | Production paths ban copied verifier logic, `.unwrap()`/`.expect()`, blake2/sha2, and fabricated measurements. | anchor=I-17:L73-L75
- **I-006/I17-E023** [SRC] (DEEP-I-006.discovery.md:L99| unit=I-17 SURFACE): E/UI | Existing cockpit branch cards gain verdict chips sourced from SSB viewer/subscription wiring; no new drawer is allowed and cross-language diff must pass. | anchor=I-17:L51-L52, L76-L79
- **I-006/I17-E024** [SRC] (DEEP-I-006.discovery.md:L100| unit=I-17): E/OBS | The hallucinated-import fixture must emit `TriLayerVerdict(HardVeto)` with `gate=L1_compile` before any L2 call and journal L2 `call_count=0`. | anchor=I-17:L80-L83
- **I-006/I17-E025** [SRC] (DEEP-I-006.discovery.md:L101| unit=I-17): E/OBS | L1 survivors emit `ScoredSurvivor`; a seeded flipping oracle emits `FlakeQuarantined`. | anchor=I-17:L80-L83
- **I-006/I17-E026** [SRC] (DEEP-I-006.discovery.md:L102| unit=I-17): E/REDTEAM | The broken arm that turns HardVeto into a strong negative weight must resurrect the wrong branch and make the contract fail; the unbroken arm selects an L1 survivor. | anchor=I-17:L82-L83
- **I-006/I17-E027** [SRC] (DEEP-I-006.discovery.md:L103| unit=I-17): E/STATUS | Because effort is M, I-0 requires a full-form expansion before execution. | anchor=I-17:L102-L102
- **I-006/I17-E028** [SRC] (DEEP-I-006.discovery.md:L104| unit=I-17): E/EVIDENCE | J-11’s source-cited Arbor amendment supplies prior art: missing held-out verified score or failure of `is_improvement` rejects merge regardless of the LLM-reported score. | anchor=I-17:L104-L112
- **I-006/I17-E029** [SRC] (DEEP-I-006.discovery.md:L105| unit=I-17): E/TEST | The same amendment reinforces separate iterative `B_dev` and milestone-only `B_test` held-out discipline. | anchor=I-17:L112-L114
- **I-006/I17-A001** [AC] (DEEP-I-006.discovery.md:L106| unit=I-17): A/CFG | `n` reruns, the seed range, quarantine persistence, and rehabilitation criteria are not defined. | anchor=I-17:L14-L16, L38-L39
- **I-006/I17-A002** [AC] (DEEP-I-006.discovery.md:L107| unit=I-17): A/THRESHOLD | The survivor-variance threshold that triggers L3 is unspecified. | anchor=I-17:L17-L18
- **I-006/I17-A003** [AC] (DEEP-I-006.discovery.md:L108| unit=I-17): A/STATUS | The exact I-4/task-runner hook locations and the registered I-23 fallback’s removal/upgrade lifecycle are deferred to execution-time discovery. | anchor=I-17:L32-L32, L38-L40, L47-L49, L67-L69
- **I-006/I17-A004** [AC] (DEEP-I-006.discovery.md:L109| unit=I-17): A/SCHEMA | The source names a `HaltingBound` penalty but does not assign it an event kind, payload, score interaction beyond branch abort, or retry/recovery behavior. | anchor=I-17:L12-L14, L37-L38
- **I-006/SD-I17-01** [SD] (DEEP-I-006.discovery.md:L541| unit=I-17): ADDITION | The 15-second timer must be shared with I-8 and charges `HaltingBound`; the summary records abort only, not timer ownership or penalty. | anchor=I17-E005, I17-A004
- **I-006/SD-I17-02** [SD] (DEEP-I-006.discovery.md:L542| unit=I-17): ADDITION | Quarantine is SSB-backed and blake3-keyed by oracle plus seed range, with a mandatory check before every vote; rerun/rehabilitation parameters remain open. | anchor=I17-E006–I17-E007, I17-A001
- **I-006/SD-I17-03** [SD] (DEEP-I-006.discovery.md:L543| unit=I-17): QUALIFICATION | The source’s pre-I-23 fallback is a registered direct conformal-calibration call, while E-19 is the PRM-lite scorer; summary L496 compresses these two fallback roles. | anchor=I17-E008, I17-A003
- **I-006/SD-I17-04** [SD] (DEEP-I-006.discovery.md:L544| unit=I-17 SURFACE): ADDITION | `DisagreementEscalated`, exact production callers, no-new-drawer IPC behavior, no-soft-veto/no-copy bans, and mandatory full-form expansion were omitted. | anchor=I17-E009, I17-E015–I17-E023, I17-E027
- **I-006/SD-I17-05** [SD] (DEEP-I-006.discovery.md:L545| unit=I-17): ADDITION | J-11 adds source-cited Arbor veto prior art and held-out `B_dev`/`B_test` discipline, absent from the summary. | anchor=I17-E028–I17-E029

#### I-006 / unit `I-19` (37 records)

- **I-006/I19-E001** [SRC] (DEEP-I-006.discovery.md:L119| unit=I-19): E/META | Track 15; lever `CANT-WRECK`; strong grounding; extension; effort S-M; Wave 3; binds IC-12. | anchor=I-19:L3-L3
- **I-006/I19-E002** [SRC] (DEEP-I-006.discovery.md:L120| unit=I-19): E/SCOPE | IC-12 correction limits I-19 to the missing symbol-level PreToolUse predicate; shipped `TESTING_FIRST`, E-13 anti-over-mock, and F-1 remain owners of their existing gates. | anchor=I-19:L4-L8, L26-L27, L31-L38
- **I-006/I19-E003** [SRC] (DEEP-I-006.discovery.md:L121| unit=I-19): E/REACH | The predicate runs before writes to `.rs`, `.py`, or `.ts` source files while the FSM is `EXECUTING`. | anchor=I-19:L10-L12, L71-L73
- **I-006/I19-E004** [SRC] (DEEP-I-006.discovery.md:L122| unit=I-19): E/ANALYSIS | Tree-sitter parses the pending diff and extracts modified/added Rust `pub fn`/`fn`, Python `def`/`class`, and corresponding TypeScript public symbols. | anchor=I-19:L11-L13, L40-L43
- **I-006/I19-E005** [SRC] (DEEP-I-006.discovery.md:L123| unit=I-19): E/GATE | A test file must contain a symbol whose name shares a substring with the modified symbol or is linked by an optional test-attribution map. | anchor=I-19:L12-L15, L42-L44
- **I-006/I19-E006** [SRC] (DEEP-I-006.discovery.md:L124| unit=I-19): E/GATE | Before disk write, staged Rust runs `cargo check --tests` and staged Python runs `pytest --collect-only`; syntax/import/type failure blocks. | anchor=I-19:L16-L18, L43-L44, L70-L70
- **I-006/I19-E007** [SRC] (DEEP-I-006.discovery.md:L125| unit=I-19): E/NON-GOAL | Preflight is compile/collection only; it may not run the actual test suite because scoped tests remain auditor-owned. | anchor=I-19:L16-L18, L28-L29, L74-L76
- **I-006/I19-E008** [SRC] (DEEP-I-006.discovery.md:L126| unit=I-19): E/EXEMPT | A diff with no new public symbol is a pure-refactor candidate and bypasses the symbol requirement; a line-delta versus name-delta cross-check disambiguates structural moves/renames. | anchor=I-19:L19-L21, L44-L45
- **I-006/I19-E009** [SRC] (DEEP-I-006.discovery.md:L127| unit=I-19): E/DEFAULT | `ELAI_TDD_PREFLIGHT_ENABLE` ships default-off in `wire-dark-allowlist.toml` as `type=experiment`, with expiry date and `promotion_plan_id=i-19`. | anchor=I-19:L22-L25, L45-L47, L53-L57, L63-L65
- **I-006/I19-E010** [SRC] (DEEP-I-006.discovery.md:L128| unit=I-19): E/PROMOTION | Default-on requires measured false-positive rate on at least three real orchestrate runs where the gate fires on a correct edit; results go to `metrics/tdd_preflight_fp.toml`. | anchor=I-19:L22-L25, L47-L50, L84-L87
- **I-006/I19-E011** [SRC] (DEEP-I-006.discovery.md:L129| unit=I-19): E/NON-GOAL | Full per-edit coverage instrumentation, forced test generation, and any implementation that blocks refactor edits are excluded. | anchor=I-19:L28-L29
- **I-006/I19-E012** [SRC] (DEEP-I-006.discovery.md:L130| unit=I-19): E/STATUS | `TESTING_FIRST`, E-13, the compound F-1 hook tiers, and workspace tree-sitter grammars are asserted shipped; exact hook locations must be regrepped under W-7. | anchor=I-19:L31-L38
- **I-006/I19-E013** [SRC] (DEEP-I-006.discovery.md:L131| unit=I-19): E/SCOPE | Tier 1 ships the tree-sitter diff parser, symbol cross-reference, Rust/Python syntax-only staging checks, refactor exemption, allowlist registration, event, and FP harness. | anchor=I-19:L40-L48
- **I-006/I19-E014** [SRC] (DEEP-I-006.discovery.md:L132| unit=I-19): E/EVENT | `TddPreflightBlocked` discriminators are `MissingTestDelta`, `SyntaxDryRunFailed`, and `RefactorExempted`. | anchor=I-19:L46-L48, L57-L57, L63-L65
- **I-006/I19-E015** [SRC] (DEEP-I-006.discovery.md:L133| unit=I-19): E/DEFER | Both default-on promotion and full line-level coverage-delta instrumentation are Tier 2; promotion is a separate patch-N. | anchor=I-19:L49-L51
- **I-006/I19-E016** [SRC] (DEEP-I-006.discovery.md:L134| unit=I-19): E/INTEG | New `crates/orchestrator/src/tdd_preflight.rs::TddPreflight::check(&Edit) -> PreflightResult` is added as a predicate after existing F-1 axes; the hook is not forked. | anchor=I-19:L53-L56, L61-L69
- **I-006/I19-E017** [SRC] (DEEP-I-006.discovery.md:L135| unit=I-19 SURFACE): E/UI | TDD block reasons use the existing F-1 gate panel and IPC path, add no drawer/endpoint, and require cross-language diff. | anchor=I-19:L57-L59, L80-L83
- **I-006/I19-E018** [SRC] (DEEP-I-006.discovery.md:L136| unit=I-19): E/SYMBOL | Produced durable artifacts are the predicate, `TddPreflightBlocked`, the allowlist row, and the FP TOML. | anchor=I-19:L61-L65, L84-L87
- **I-006/I19-E019** [SRC] (DEEP-I-006.discovery.md:L137| unit=I-19): E/DEP | The implementation consumes `hooks/run_pre_tool_use`, E-13’s neighboring axis, existing knowledge-crate grammar bindings, and external compiler/collector commands; none is reimplemented. | anchor=I-19:L66-L70, L98-L102
- **I-006/I19-E020** [SRC] (DEEP-I-006.discovery.md:L138| unit=I-19): E/DONE | Only the production PreToolUse dispatch satisfies R7; contracts-only callers do not. | anchor=I-19:L71-L73
- **I-006/I19-E021** [SRC] (DEEP-I-006.discovery.md:L139| unit=I-19): E/LIMIT | `cargo test` in preflight, unmeasured default-on, blocking pure renames, and rebundling tree-sitter grammars are banned. | anchor=I-19:L74-L79
- **I-006/I19-E022** [SRC] (DEEP-I-006.discovery.md:L140| unit=I-19): E/SEC | Production paths ban `.unwrap()`/`.expect()` and fabricated false-positive measurements. | anchor=I-19:L79-L79
- **I-006/I19-E023** [SRC] (DEEP-I-006.discovery.md:L141| unit=I-19): E/FAIL | Adding a new public function without a matching test-symbol delta refuses the edit before disk and emits `MissingTestDelta`. | anchor=I-19:L84-L87
- **I-006/I19-E024** [SRC] (DEEP-I-006.discovery.md:L142| unit=I-19): E/OBS | Staged syntax failure emits `SyntaxDryRunFailed`; rename-only passage emits `RefactorExempted`. | anchor=I-19:L84-L87
- **I-006/I19-E025** [SRC] (DEEP-I-006.discovery.md:L143| unit=I-19): E/TEST | A matching failing test is a positive arm and permits the edit; a `--skip-preflight` broken arm bypasses the gate and must fail the contract. | anchor=I-19:L84-L87
- **I-006/I19-E026** [SRC] (DEEP-I-006.discovery.md:L144| unit=I-19): E/ARTIFACT | The three-run report at `metrics/tdd_preflight_fp.toml` must exist before promotion. | anchor=I-19:L84-L87
- **I-006/I19-E027** [SRC] (DEEP-I-006.discovery.md:L145| unit=I-19): E/INTEG | The feature composes the FSM state precondition, F-1/E-13 compound axes, and existing grammars; only the predicate and FP harness are net-new. | anchor=I-19:L89-L102
- **I-006/I19-C001** [CON] (DEEP-I-006.discovery.md:L146| unit=I-19): C/SCOPE | The source resolves “coverage-delta exemption” into a Tier-1 no-new-public-symbol plus line/name heuristic and explicitly defers full line-level coverage instrumentation. | anchor=I-19:L19-L21, L49-L51
- **I-006/I19-A001** [AC] (DEEP-I-006.discovery.md:L147| unit=I-19): A/THRESHOLD | No numerical acceptable false-positive bound is supplied after the minimum three runs. | anchor=I-19:L22-L25, L49-L50
- **I-006/I19-A002** [AC] (DEEP-I-006.discovery.md:L148| unit=I-19 SURFACE): A/LANGUAGE | TypeScript is in the parse/reachability scope, but no TypeScript syntax-only command is named alongside Rust/Python. | anchor=I-19:L11-L12, L16-L18, L36-L36, L71-L72
- **I-006/I19-A003** [AC] (DEEP-I-006.discovery.md:L149| unit=I-19): A/SCHEMA | The optional attribution-map path/schema, substring normalization, collision handling, and test-file discovery rules are not defined. | anchor=I-19:L12-L15, L42-L44
- **I-006/I19-A004** [AC] (DEEP-I-006.discovery.md:L150| unit=I-19): A/EVENT | `RefactorExempted` is carried by an action type named `TddPreflightBlocked` even though the edit is allowed; consumer semantics for “non-blocking blocked-row” are not specified. | anchor=I-19:L46-L48, L57-L57, L84-L87
- **I-006/SD-I19-01** [SD] (DEEP-I-006.discovery.md:L551| unit=I-19): CORRECTION | Summary L558 says the exemption is determined through coverage delta; the source narrows Tier 1 to no-new-public-symbol plus line/name heuristic and defers full line-level coverage instrumentation. | anchor=I19-E008, I19-E015, I19-C001
- **I-006/SD-I19-02** [SD] (DEEP-I-006.discovery.md:L552| unit=I-19): ADDITION | Default-off `ELAI_TDD_PREFLIGHT_ENABLE` allowlist registration, experiment/expiry/promotion fields, and separate patch-N promotion were omitted. | anchor=I19-E009–I19-E010, I19-E015
- **I-006/SD-I19-03** [SD] (DEEP-I-006.discovery.md:L553| unit=I-19): ADDITION | TypeScript parse/reach scope, substring/attribution matching, `SyntaxDryRunFailed`, and the staged-content compile-only/no-test constraint were omitted. | anchor=I19-E003–I19-E007, I19-E014, I19-A002–I19-A003
- **I-006/SD-I19-04** [SD] (DEEP-I-006.discovery.md:L554| unit=I-19 SURFACE): ADDITION | Exact predicate symbol, FP TOML, action-lock/cross-language/UI reuse, positive failing-test arm, and skip-preflight broken arm were omitted. | anchor=I19-E016–I19-E027
- **I-006/SD-I19-05** [SD] (DEEP-I-006.discovery.md:L555| unit=I-19 SURFACE): AMBIGUITY | The deep source reveals the non-blocking `RefactorExempted` discriminator under a “Blocked” action family; the summary reports it without flagging this consumer-contract ambiguity. | anchor=I19-A004

#### I-006 / unit `I-24` (37 records)

- **I-006/I24-E001** [SRC] (DEEP-I-006.discovery.md:L160| unit=I-24): E/META | Track 15; `ANTI-GAMING`; strong grounding; extension; effort S-M; Wave 4; binds ID-4. | anchor=I-24:L3-L3
- **I-006/I24-E002** [SRC] (DEEP-I-006.discovery.md:L161| unit=I-24): E/CAP | The feature detects scorer-side style bias in I-17 L2 and complements, rather than copies, H-44 candidate-side trap detection. | anchor=I-24:L4-L9, L25-L28
- **I-006/I24-E003** [SRC] (DEEP-I-006.discovery.md:L162| unit=I-24): E/DET | With probability `p`, sampled uniformly and deterministically under `ELAI_SEED`, the L2 path injects the original candidate plus a frozen isomorphic variant. | anchor=I-24:L11-L15, L47-L53
- **I-006/I24-E004** [SRC] (DEEP-I-006.discovery.md:L163| unit=I-24): E/TRANSFORM | Tier-1 relation types are `ForWhileEquiv`, `RenameIdempotent`, and `ConstantExtraction`. | anchor=I-24:L13-L18, L47-L50
- **I-006/I24-E005** [SRC] (DEEP-I-006.discovery.md:L164| unit=I-24 SURFACE): E/INVARIANT | Transformations may alter surface style but not control flow, data flow, or type constraints. | anchor=I-24:L16-L20
- **I-006/I24-E006** [SRC] (DEEP-I-006.discovery.md:L165| unit=I-24): E/VALIDATION | Both variants run through L1; differing L1 outcomes identify a malformed canary/obfuscation bug, not scorer bias. | anchor=I-24:L18-L20, L85-L88
- **I-006/I24-E007** [SRC] (DEEP-I-006.discovery.md:L166| unit=I-24): E/MEASURE | Bias is a score delta above an ELAI-measured known-unbiased-scorer threshold stored at `metrics/canary_bias_threshold.toml`; paper thresholds are prohibited. | anchor=I-24:L21-L24, L49-L52, L89-L89
- **I-006/I24-E008** [SRC] (DEEP-I-006.discovery.md:L167| unit=I-24): E/LIFECYCLE | One positive detection is a warning; `N` consecutive detections in one run set the scorer’s weight to zero for the remainder of that run and emit neutralization. | anchor=I-24:L21-L24, L51-L52
- **I-006/I24-E009** [SRC] (DEEP-I-006.discovery.md:L168| unit=I-24): E/INTEG | H-44 and I-24 run in parallel through L2 and jointly inform the run verdict, while H-44 remains sole owner of its trap logic. | anchor=I-24:L25-L28, L76-L81
- **I-006/I24-E010** [SRC] (DEEP-I-006.discovery.md:L169| unit=I-24): E/SEC | `fixtures/canary_pairs.json` is static, git-versioned, frozen out of band, listed in `.claude/gate-manifest.txt`, and covered by `gate_freeze_check.py`; the evaluated model never authors canaries. | anchor=I-24:L29-L32, L52-L55, L63-L64
- **I-006/I24-E011** [SRC] (DEEP-I-006.discovery.md:L170| unit=I-24): E/NON-GOAL | Runtime model-generated canaries, semantic-changing pairs, and any H-44 reimplementation are excluded. | anchor=I-24:L33-L35
- **I-006/I24-E012** [SRC] (DEEP-I-006.discovery.md:L171| unit=I-24): E/STATUS | IS-8 `RelationLibrary`/`MutationRunner`, H-44, I-17’s L2 seam, D-2 `PpmClient`, and the freeze checker are asserted shipped/available composition points; H-22/C-29 relationships require W-7 re-grep. | anchor=I-24:L37-L45, L103-L113
- **I-006/I24-E013** [SRC] (DEEP-I-006.discovery.md:L172| unit=I-24): E/SCOPE | Tier 1 adds the three Rust relations, seeded injector, measured threshold, run-local neutralizer, frozen fixture, freeze coverage, and SSB event. | anchor=I-24:L47-L55
- **I-006/I24-E014** [SRC] (DEEP-I-006.discovery.md:L173| unit=I-24): E/DEFER | New-language canary generation waits for per-language AST verification and dependency approval; cross-scorer correlation waits for multiple live scorers. | anchor=I-24:L56-L58
- **I-006/I24-E015** [SRC] (DEEP-I-006.discovery.md:L174| unit=I-24): E/INTEG | `canary_injector.rs` is called immediately before PPM scoring, RelationLibrary is extended in place, and the frozen fixture is added to the existing manifest. | anchor=I-24:L60-L65
- **I-006/I24-E016** [SRC] (DEEP-I-006.discovery.md:L175| unit=I-24): E/EVENT | `PrmCanaryEvent` kinds are `CanaryInjected`, `StyleBiasDetected`, and `ScorerNeutralized`. | anchor=I-24:L55-L55, L65-L65, L69-L75
- **I-006/I24-E017** [SRC] (DEEP-I-006.discovery.md:L176| unit=I-24 SURFACE): E/UI | The existing branch board/scoring panel gains `SCORER OK`, `BIAS DETECTED`, and `NEUTRALIZED` health states via SSB subscription; no new surface is created and cross-language diff must pass. | anchor=I-24:L66-L67, L92-L97
- **I-006/I24-E018** [SRC] (DEEP-I-006.discovery.md:L177| unit=I-24): E/SYMBOL | Produced symbols/artifacts are `CanaryInjector`, the three relations, the frozen JSON pair set, `PrmCanaryEvent`, and the threshold TOML. | anchor=I-24:L69-L75
- **I-006/I24-E019** [SRC] (DEEP-I-006.discovery.md:L178| unit=I-24): E/DEP | Consumers are IS-8 RelationLibrary, H-44 output, I-17’s single L2 invocation, D-2 PPM, and the existing freeze checker. | anchor=I-24:L76-L81, L115-L120
- **I-006/I24-E020** [SRC] (DEEP-I-006.discovery.md:L179| unit=I-24): E/REACH | Production reach is `TriLayerStack::run_verdict`’s L2 path; contracts-only callers do not satisfy R7. | anchor=I-24:L82-L84
- **I-006/I24-E021** [SRC] (DEEP-I-006.discovery.md:L180| unit=I-24): E/LIMIT | Model-authored or semantic-changing canaries are hard blocks. | anchor=I-24:L85-L88
- **I-006/I24-E022** [SRC] (DEEP-I-006.discovery.md:L181| unit=I-24): E/LIMIT | The injector may neither duplicate H-44 nor use paper thresholds nor let neutralization alter L1 HardVeto supremacy. | anchor=I-24:L88-L90
- **I-006/I24-E023** [SRC] (DEEP-I-006.discovery.md:L182| unit=I-24): E/SEC | Production paths ban `.unwrap()`/`.expect()`, blake2/sha2, and fabricated bias results. | anchor=I-24:L91-L91
- **I-006/I24-E024** [SRC] (DEEP-I-006.discovery.md:L183| unit=I-24): E/OBS | A seeded biased fixture emits injection, then style-bias, then neutralization after `N` consecutive detections. | anchor=I-24:L98-L101
- **I-006/I24-E025** [SRC] (DEEP-I-006.discovery.md:L184| unit=I-24): E/DET | Canary pair choice and both scores must recompute under `ELAI_SEED`. | anchor=I-24:L99-L101
- **I-006/I24-E026** [SRC] (DEEP-I-006.discovery.md:L185| unit=I-24): E/TEST | A known-unbiased scorer must emit only `CanaryInjected`, proving the detector does not overfire. | anchor=I-24:L100-L101
- **I-006/I24-E027** [SRC] (DEEP-I-006.discovery.md:L186| unit=I-24): E/REDTEAM | A runtime dynamic-canary broken arm must fail the frozen-gate check. | anchor=I-24:L100-L101
- **I-006/I24-E028** [SRC] (DEEP-I-006.discovery.md:L187| unit=I-24 SURFACE): E/TEST | Every new relation has an L1-verified semantic-equivalence test; the UI path requires cross-language diff. | anchor=I-24:L47-L50, L92-L97
- **I-006/I24-A001** [AC] (DEEP-I-006.discovery.md:L188| unit=I-24): A/CFG | Injection probability `p`, warning threshold, consecutive-detection count `N`, and reset timing within a run are unspecified. | anchor=I-24:L11-L12, L21-L24
- **I-006/I24-A002** [AC] (DEEP-I-006.discovery.md:L189| unit=I-24): A/MEASURE | The prose uses signed `score(original)-score(obfuscated) > threshold`; it does not say whether reverse-direction or absolute divergence also constitutes style bias. | anchor=I-24:L21-L24
- **I-006/I24-A003** [AC] (DEEP-I-006.discovery.md:L190| unit=I-24): A/LIFECYCLE | The known-unbiased calibration corpus, recalibration trigger, threshold versioning, and behavior after the run ends are not specified. | anchor=I-24:L21-L24, L49-L52, L74-L75
- **I-006/SD-I24-01** [SD] (DEEP-I-006.discovery.md:L561| unit=I-24): ADDITION | `ConstantExtraction`, seeded probability `p`, uniform deterministic sampling, and L1 semantic-equivalence validation were omitted. | anchor=I24-E003–I24-E006
- **I-006/SD-I24-02** [SD] (DEEP-I-006.discovery.md:L562| unit=I-24): ADDITION | A single detection is only a warning; neutralization requires `N` consecutive detections, with `p`, `N`, and threshold left unbound. | anchor=I24-E007–I24-E008, I24-A001
- **I-006/SD-I24-03** [SD] (DEEP-I-006.discovery.md:L563| unit=I-24): CORRECTION | Summary L678 says neutralization lasts until recalibration; the source says weight zero for the current run and does not define post-run/recalibration persistence. | anchor=I24-E008, I24-A003
- **I-006/SD-I24-04** [SD] (DEEP-I-006.discovery.md:L564| unit=I-24): ADDITION | Exact frozen fixture/manifest/checker artifacts, three event kinds, production L2 caller, and dynamic-canary freeze-check broken arm were omitted. | anchor=I24-E010, I24-E015–I24-E020, I24-E024–I24-E028
- **I-006/SD-I24-05** [SD] (DEEP-I-006.discovery.md:L565| unit=I-24): ADDITION | Tier 1 covers Rust; new-language adapters and multi-scorer correlation are separately deferred. | anchor=I24-E013–I24-E014
- **I-006/SD-I24-06** [SD] (DEEP-I-006.discovery.md:L566| unit=I-24 SURFACE): AMBIGUITY | Signed rather than absolute score delta and the calibration/version lifecycle are unresolved in the source but not surfaced by the summary. | anchor=I24-A002–I24-A003

#### I-006 / unit `I-26` (35 records)

- **I-006/I26-E001** [SRC] (DEEP-I-006.discovery.md:L200| unit=I-26): E/META | Track 16 `ROUTING + BUDGETS + MEASUREMENT`; `STEERING`; strong grounding; extension; effort M; Wave 4. | anchor=I-26:L1-L3
- **I-006/I26-E002** [SRC] (DEEP-I-006.discovery.md:L201| unit=I-26): E/CAP | I-26 extends H-42’s serial-depth budget discipline from the main GRC chain to every I-4 search-node expansion. | anchor=I-26:L4-L9, L11-L14
- **I-006/I26-E003** [SRC] (DEEP-I-006.discovery.md:L202| unit=I-26): E/ALGO | `BudgetController` computes `node_budget(depth) = base * depth_factor^depth`, capped by the supreme E-39 hard cost ceiling. | anchor=I-26:L13-L15, L55-L64
- **I-006/I26-E004** [SRC] (DEEP-I-006.discovery.md:L203| unit=I-26): E/COLLAPSE | When accumulated spend across all active siblings reaches the 85%-collapse condition, new expansions stop. | anchor=I-26:L15-L16, L35-L38
- **I-006/I26-E005** [SRC] (DEEP-I-006.discovery.md:L204| unit=I-26): E/FLOW | Collapse scores the current frontier with I-17 L2, synthesizes the best result, and emits I-27 `ConvergenceCurveMeasured` evidence. | anchor=I-26:L15-L18
- **I-006/I26-E006** [SRC] (DEEP-I-006.discovery.md:L205| unit=I-26): E/EARLY-STOP | If the top L2 surviving score fails to improve by `delta_min` for five consecutive supersteps, the search collapses early. | anchor=I-26:L17-L18, L35-L38
- **I-006/I26-E007** [SRC] (DEEP-I-006.discovery.md:L206| unit=I-26): E/PROVIDER | Cloud architecture/reasoning requests use `effort` (`auto`/`expert`); `budget_tokens` must be structurally absent on Opus 4.7+ paths. | anchor=I-26:L19-L21, L35-L41, L55-L59
- **I-006/I26-E008** [SRC] (DEEP-I-006.discovery.md:L207| unit=I-26): E/INVARIANT | Reasoning or extended-thinking roles may never be force-wrapped as JSON. | anchor=I-26:L21-L25, L70-L74
- **I-006/I26-E009** [SRC] (DEEP-I-006.discovery.md:L208| unit=I-26): E/SCHEMA | No action type is added: H-42 `ReasoningBudgetScaled` is reused, with in-place `depth_level`/`plateau_fired` payload extension only if live schema lacks them. | anchor=I-26:L22-L23, L60-L61, L84-L90
- **I-006/I26-E010** [SRC] (DEEP-I-006.discovery.md:L209| unit=I-26): E/STATUS | H-42 event, E-39 ceiling, and E-19 confidence gate are plan-asserted shipped; the executor must re-grep their live definitions, and I-4 is the Wave-4 predecessor. | anchor=I-26:L27-L32
- **I-006/I26-E011** [SRC] (DEEP-I-006.discovery.md:L210| unit=I-26): E/SEC | The collapse controller cannot be generator-reachable under R8, and mutable budget values may not enter the HD-2 frozen cache prefix. | anchor=I-26:L32-L33
- **I-006/I26-E012** [SRC] (DEEP-I-006.discovery.md:L211| unit=I-26): E/SCOPE | Tier 1 creates `crates/orchestrator/src/routing/budget_controller.rs` with depth scaling, 85%/plateau collapse, and provider effort mapping. | anchor=I-26:L35-L38, L48-L52
- **I-006/I26-E013** [SRC] (DEEP-I-006.discovery.md:L212| unit=I-26): E/TEST | `i26_dynamic_thinking_budgets_contract.rs` checks Opus request-builder field absence and `ReasoningBudgetScaled` depth/collapse metadata. | anchor=I-26:L39-L41
- **I-006/I26-E014** [SRC] (DEEP-I-006.discovery.md:L213| unit=I-26): E/ARTIFACT | Tier 1 also writes `workflow/reasoning/I-26-decisions.md`. | anchor=I-26:L42-L42
- **I-006/I26-E015** [SRC] (DEEP-I-006.discovery.md:L214| unit=I-26): E/DEFER | Journal-trained `depth_factor` and effort levels beyond `auto`/`expert` wait for I-27 trajectory volume and measured cost/quality deltas. | anchor=I-26:L44-L46
- **I-006/I26-E016** [SRC] (DEEP-I-006.discovery.md:L215| unit=I-26): E/INTEG | The controller hooks I-4 search expansion/collapse and modifies the existing cloud request builder (for example `providers/src/anthropic_api.rs`) rather than forking either. | anchor=I-26:L48-L53, L92-L96
- **I-006/I26-E017** [SRC] (DEEP-I-006.discovery.md:L216| unit=I-26): E/API | Produced API is `routing::budget_controller::BudgetController`; provider output replaces Opus `budget_tokens` with `effort`. | anchor=I-26:L55-L59
- **I-006/I26-E018** [SRC] (DEEP-I-006.discovery.md:L217| unit=I-26): E/DEP | The controller consumes H-42 event schema, E-39 cap, E-19 confidence prior, and the I-4 expansion loop through their existing interfaces. | anchor=I-26:L60-L64
- **I-006/I26-E019** [SRC] (DEEP-I-006.discovery.md:L218| unit=I-26 SURFACE): E/REACH | Production reach is `elai run <task>` → I-4 ladder → `BudgetController` at each node, with SSB rows as off-agent evidence. | anchor=I-26:L66-L68
- **I-006/I26-E020** [SRC] (DEEP-I-006.discovery.md:L219| unit=I-26): E/LIMIT | Opus manual budgets, reasoning-role force-JSON, paper-derived threshold constants, `.unwrap()`/`.expect()`, blake2, and sha2 are banned. | anchor=I-26:L70-L74
- **I-006/I26-E021** [SRC] (DEEP-I-006.discovery.md:L220| unit=I-26 SURFACE): E/UI | Existing H-15/I-4 branch cards show a per-node remaining-budget bar from `depth_level`, `budget_remaining`, and `plateau_fired`; no new dashboard crate, and cross-language diff covers Tauri→Svelte. | anchor=I-26:L53-L53, L76-L77
- **I-006/I26-E022** [SRC] (DEEP-I-006.discovery.md:L221| unit=I-26): E/OBS | Deep-fixture runs journal per-node depth; cloud Opus rows contain effort and zero `budget_tokens` request bytes. | anchor=I-26:L79-L82
- **I-006/I26-E023** [SRC] (DEEP-I-006.discovery.md:L222| unit=I-26): E/IDEMP | Once a collapse marker lands, no later `SearchNodeEvent(Expanded)` may exist; journal order makes post-collapse expansion structurally testable. | anchor=I-26:L79-L82
- **I-006/I26-E024** [SRC] (DEEP-I-006.discovery.md:L223| unit=I-26): E/FRESHNESS | Current Anthropic effort-parameter API shape must be web-verified before modifying the builder. | anchor=I-26:L84-L87
- **I-006/I26-E025** [SRC] (DEEP-I-006.discovery.md:L224| unit=I-26): E/DEP | RD-1..RD-4 bind because this controls search reasoning budgets; the action-type delta is explicitly baseline +0. | anchor=I-26:L88-L90
- **I-006/I26-C001** [CON] (DEEP-I-006.discovery.md:L225| unit=I-26): C/THRESHOLD | The source alternates among spend “exceeds 85%,” observable `budget_pct>=85`, and the title/premise’s “at 85%”; exact boundary equality is internally inconsistent. | anchor=I-26:L5-L7, L15-L16, L79-L82
- **I-006/I26-A001** [AC] (DEEP-I-006.discovery.md:L226| unit=I-26): A/CFG | `base`, `depth_factor`, `delta_min`, total-envelope calculation, token accounting unit, and synthesize-best tie-break are unspecified. | anchor=I-26:L13-L18
- **I-006/I26-A002** [AC] (DEEP-I-006.discovery.md:L227| unit=I-26): A/SCHEMA | Whether `ReasoningBudgetScaled` actually lacks fields, what an in-place migration looks like, and whether `ConvergenceCurveMeasured` or the reused row is the canonical collapse marker are edit-time decisions. | anchor=I-26:L17-L18, L22-L23, L40-L41, L79-L82
- **I-006/I26-A003** [AC] (DEEP-I-006.discovery.md:L228| unit=I-26): A/PROVIDER | Exact model-version detection and the current request enum/schema for `effort` remain external-document and live-code discoveries. | anchor=I-26:L19-L21, L39-L41, L48-L50, L84-L87
- **I-006/SD-I26-01** [SD] (DEEP-I-006.discovery.md:L572| unit=I-26): ADDITION | The exact exponential `base*depth_factor^depth` formula, all-active-sibling spend aggregation, I-17 frontier call, and I-27 event handoff were omitted. | anchor=I26-E003–I26-E006
- **I-006/SD-I26-02** [SD] (DEEP-I-006.discovery.md:L573| unit=I-26): CONFLICT | Summary L719 says “at 85 percent”; the source itself alternates `>85`, `>=85`, and “at,” so the boundary is not settled. | anchor=I26-C001
- **I-006/SD-I26-03** [SD] (DEEP-I-006.discovery.md:L574| unit=I-26): ADDITION | Generator inaccessibility, cache-prefix invariance, exact module/decision doc, production path, conditional row migration, and no-new-action decision were omitted. | anchor=I26-E009–I26-E014, I26-E016–I26-E019
- **I-006/SD-I26-04** [SD] (DEEP-I-006.discovery.md:L575| unit=I-26): ADDITION | Adaptive depth-factor and extra effort levels are explicitly deferred pending I-27 measured data. | anchor=I26-E015
- **I-006/SD-I26-05** [SD] (DEEP-I-006.discovery.md:L576| unit=I-26): UNSUPPORTED | Summary L730 says a plateau fixture reports measured token savings; the source requires depth/collapse/request evidence but contains no explicit token-savings acceptance output. | anchor=I26-E013, I26-E022–I26-E023
- **I-006/SD-I26-06** [SD] (DEEP-I-006.discovery.md:L577| unit=I-26): ADDITION | Unspecified budget constants, collapse-event ownership/schema migration, and provider-version/API discovery were not recorded as open issues. | anchor=I26-A001–I26-A003

#### I-006 / unit `I-30` (68 records)

- **I-006/I30-E001** [SRC] (DEEP-I-006.discovery.md:L238| unit=I-30): E/META | Track 17 `OPEN-WEIGHT FLYWHEEL`; `ALIGNMENT`; strong grounding; build; effort L; Wave 5; depends on I-4; binds ID-6 and IC-7; full-form expansion is mandatory before execution. | anchor=I-30:L1-L10
- **I-006/I30-E002** [SRC] (DEEP-I-006.discovery.md:L239| unit=I-30): E/STRATEGY | Rejection-sampling SFT is the first distillation step because it captures verifier-ranked search gains without RL instability; I-31 GRPO follows only after blocking guards. | anchor=I-30:L4-L8, L21-L24
- **I-006/I30-E003** [SRC] (DEEP-I-006.discovery.md:L240| unit=I-30): E/ELIGIBILITY | `SftExhaust` walks only sealed RRAs; in-progress or manually closed RRAs cannot yield training rows. | anchor=I-30:L12-L18, L44-L48
- **I-006/I30-E004** [SRC] (DEEP-I-006.discovery.md:L241| unit=I-30): E/SELECTION | For each sealed run, the nominal winner is a root-to-verified-`COMPLETE` path selected by `TriLayerVerdict(Promoted)` or highest-L2 `PhaseComplete` among L1 passers. | anchor=I-30:L13-L17
- **I-006/I30-E005** [SRC] (DEEP-I-006.discovery.md:L242| unit=I-30): E/SCHEMA | Each JSONL record contains system prompt, every trajectory turn, and final verified output. | anchor=I-30:L16-L17
- **I-006/I30-E006** [SRC] (DEEP-I-006.discovery.md:L243| unit=I-30): E/PROVENANCE | Every record carries `blake3(RRA_bytes)` and fails verification if it does not match the sealed RRA on disk. | anchor=I-30:L17-L18, L81-L83
- **I-006/I30-E007** [SRC] (DEEP-I-006.discovery.md:L244| unit=I-30): E/NEGATIVE | Each winner is paired with the highest-L2 fluent sibling that received an L1 deterministic veto. | anchor=I-30:L18-L21
- **I-006/I30-E008** [SRC] (DEEP-I-006.discovery.md:L245| unit=I-30): E/SCHEMA | Positive and negative examples share the JSONL and are distinguished by a label field. | anchor=I-30:L20-L21
- **I-006/I30-E009** [SRC] (DEEP-I-006.discovery.md:L246| unit=I-30): E/NON-GOAL | I-30 exposes SFT extraction only: DPO fine-tuning, PPO, and GRPO are forbidden; inherited E-55 `HARD-AVOID` is not reopened. | anchor=I-30:L21-L23, L29-L31, L81-L84
- **I-006/I30-E010** [SRC] (DEEP-I-006.discovery.md:L247| unit=I-30): E/TIER | Tier 1 ships offline dataset extraction and gate linting; the actual LoRA GPU run is a separate RunPod Tier-2 checkbox. | anchor=I-30:L23-L24, L44-L57
- **I-006/I30-E011** [SRC] (DEEP-I-006.discovery.md:L248| unit=I-30): E/MODEL | The canonical target remains Qwen2.5-Coder; a MoE/Qwen3-Coder-Next retarget is a separate measured RunPod evaluation, never an assumed swap. | anchor=I-30:L25-L26, L54-L57, L84-L86
- **I-006/I30-E012** [SRC] (DEEP-I-006.discovery.md:L249| unit=I-30): E/PROMOTION | Any LoRA checkpoint must pass H-17 trial-N, H-18 deflation, H-28 reward-tampering, D-6 held-out, and D-7 cross-domain-canary gates under ID-6. | anchor=I-30:L26-L31, L33-L42
- **I-006/I30-E013** [SRC] (DEEP-I-006.discovery.md:L250| unit=I-30): E/SEC | The checkpoint evaluator/harness and training scripts are frozen artifacts outside the training loop’s write boundary under R8. | anchor=I-30:L27-L29, L42-L42
- **I-006/I30-E014** [SRC] (DEEP-I-006.discovery.md:L251| unit=I-30): E/NON-GOAL | Unsealed inputs and promotion without the complete ID-6 chain are explicitly excluded. | anchor=I-30:L29-L31
- **I-006/I30-E015** [SRC] (DEEP-I-006.discovery.md:L252| unit=I-30): E/DEP | The sealed source is existing IS-10 `build_rra`, `write_rra`, `read_rra`, `verify_rra`, and attestation; RRA construction/verification must not be copied. | anchor=I-30:L33-L35, L97-L108
- **I-006/I30-E016** [SRC] (DEEP-I-006.discovery.md:L253| unit=I-30): E/DEP | Existing IS-16 `trial_ledger`, `deflation_gate`, `holdout_budget`, and reward/self-validation gates provide the promotion chain; I-4 provides trajectories. | anchor=I-30:L35-L39, L71-L75
- **I-006/I30-E017** [SRC] (DEEP-I-006.discovery.md:L254| unit=I-30): E/INTEG | When I-28 is available, the sealed walker calls `DpoPairExtractor::run` with `DpoProvenance{rra_sealed:true}` and must not consume current live `DpoPairExtracted` rows because `task_runner` hardcodes false. | anchor=I-30:L39-L42, L71-L75, L106-L110
- **I-006/I30-E018** [SRC] (DEEP-I-006.discovery.md:L255| unit=I-30): E/ARTIFACT | Training scripts/evaluator are frozen outside the loop; the dataset itself is a config-class, provenance-bearing artifact. | anchor=I-30:L36-L42, L97-L103
- **I-006/I30-E019** [SRC] (DEEP-I-006.discovery.md:L256| unit=I-30 SURFACE): E/CLI | Tier 1 creates `crates/cli/src/sft_exhaust.rs` and `elai dataset export --format sft`. | anchor=I-30:L44-L48, L59-L63
- **I-006/I30-E020** [SRC] (DEEP-I-006.discovery.md:L257| unit=I-30): E/GATE | Export checks whether the result is tied to the IS-16 chain and names missing required rows; it may not claim promotion. | anchor=I-30:L48-L50, L81-L86, L189-L194
- **I-006/I30-E021** [SRC] (DEEP-I-006.discovery.md:L258| unit=I-30 SURFACE): E/UI | The dataset-card drawer is explicitly deferred under `owner-ui-rework-hold-2026-08-06`; only backend `SftExhaustEvent` ships in Tier 1. | anchor=I-30:L49-L51, L59-L64, L89-L90
- **I-006/I30-E022** [SRC] (DEEP-I-006.discovery.md:L259| unit=I-30): E/TEST | Tier 1 includes `crates/contracts/tests/i30_sft_exhaust_contract.rs` and `workflow/reasoning/I-30-decisions.md`. | anchor=I-30:L51-L52
- **I-006/I30-E023** [SRC] (DEEP-I-006.discovery.md:L260| unit=I-30): E/DEFER | LoRA training, checkpoint promotion after a produced checkpoint, and MoE retarget are separately gated Tier-2 items; any new LoRA dependency needs anti-slopsquat review and user confirmation. | anchor=I-30:L54-L57
- **I-006/I30-E024** [SRC] (DEEP-I-006.discovery.md:L261| unit=I-30): E/STATUS | `workflow/training/sft/` training scripts are a Tier-2 integration location and do not ship until dependency approval. | anchor=I-30:L59-L64
- **I-006/I30-E025** [SRC] (DEEP-I-006.discovery.md:L262| unit=I-30): E/API | `SftExhaust` is the sealed-RRA walker/JSONL extractor/gate checker; output is `workflow/training/sft/<run_id>.jsonl`. | anchor=I-30:L66-L75
- **I-006/I30-E026** [SRC] (DEEP-I-006.discovery.md:L263| unit=I-30): E/EVENT | `SftExhaustEvent` emits `PathExtracted` per winning path, `NegativePaired` per negative, and final `DatasetSealed` with `record_count` and `provenance_verified_count`. | anchor=I-30:L77-L79, L92-L95
- **I-006/I30-E027** [SRC] (DEEP-I-006.discovery.md:L264| unit=I-30 SURFACE): E/REACH | Production reach is the dataset-export CLI; SSB rows and JSONL are the off-agent artifacts. | anchor=I-30:L77-L79
- **I-006/I30-E028** [SRC] (DEEP-I-006.discovery.md:L265| unit=I-30): E/FAIL | An unsealed RRA is rejected at the walker, yields no `PathExtracted`, and produces a visible lint/rejection. | anchor=I-30:L92-L95, L148-L164
- **I-006/I30-E029** [SRC] (DEEP-I-006.discovery.md:L266| unit=I-30): E/FAIL | Provenance mismatch fails closed, emits `DatasetSealed(provenance_mismatch)` or contract-equivalent failure, and may not silently accept/write a training row. | anchor=I-30:L92-L95, L150-L165
- **I-006/I30-E030** [SRC] (DEEP-I-006.discovery.md:L267| unit=I-30): E/TRACE | Negative pairs must cite real vetoed branches by `TriLayerVerdict(HardVeto)` row ID. | anchor=I-30:L92-L95
- **I-006/I30-E031** [SRC] (DEEP-I-006.discovery.md:L268| unit=I-30): E/LIMIT | Dataset inputs must be sealed, no DPO/GRPO/PPO training may enter this unit, and no checkpoint may bypass the full ledger chain or assume a model swap. | anchor=I-30:L81-L86
- **I-006/I30-E032** [SRC] (DEEP-I-006.discovery.md:L269| unit=I-30): E/SEC | Counts/statistics must derive from real RRAs; production paths ban `.unwrap()`/`.expect()` and blake2/sha2. | anchor=I-30:L85-L87, L173-L180
- **I-006/I30-E033** [SRC] (DEEP-I-006.discovery.md:L270| unit=I-30 SURFACE): E/UI | Tier-1 implementation must not add the deferred UI drawer. | anchor=I-30:L89-L90, L173-L180
- **I-006/I30-E034** [SRC] (DEEP-I-006.discovery.md:L271| unit=I-30): E/INTEG | Live SSB uses split `ssb/{action_type,payloads,schema,store,tests}.rs`; a monolithic `ssb.rs` or `ssb/action_type_lock.rs` must not be treated as current source. | anchor=I-30:L97-L104
- **I-006/I30-E035** [SRC] (DEEP-I-006.discovery.md:L272| unit=I-30 SURFACE): E/EVIDENCE | J-11 amendment adopts difficulty-banded inclusion as design rationale from a cited DeNovoSWE ablation; its reported gain is never an ELAI performance target. | anchor=I-30:L114-L120
- **I-006/I30-E036** [SRC] (DEEP-I-006.discovery.md:L273| unit=I-30): E/CFG | The amendment records five band floors: difficulty `[0,.2)→.90`, `[.2,.4)→.85`, `[.4,.6)→.80`, `[.6,.8)→.70`, `[.8,1]→.60`. | anchor=I-30:L122-L131
- **I-006/I30-E037** [SRC] (DEEP-I-006.discovery.md:L274| unit=I-30): E/ELIGIBILITY | There is no global pass/fail inclusion gate; a trajectory is included only if its score meets its difficulty-band floor. | anchor=I-30:L133-L136
- **I-006/I30-E038** [SRC] (DEEP-I-006.discovery.md:L275| unit=I-30): E/RANK | Among floor-qualified trajectories, a harder-band partial is preferred to an easier-band/trivial pass because it preserves more repository-level reasoning. | anchor=I-30:L136-L141
- **I-006/I30-E039** [SRC] (DEEP-I-006.discovery.md:L276| unit=I-30): E/FEATURE | Difficulty `d_i` comes from I-25 when shipped or I-30 calibration; candidate structural signals include SCIP function count, cross-language bindings touched, and SSB variants—not Python-only executable-line count. | anchor=I-30:L142-L144
- **I-006/I30-E040** [SRC] (DEEP-I-006.discovery.md:L277| unit=I-30): E/TEST | Contract cases must cover sealed, unsealed, tampered-provenance, and every difficulty band, writing only sealed/provenance-valid records. | anchor=I-30:L146-L156
- **I-006/I30-E041** [SRC] (DEEP-I-006.discovery.md:L278| unit=I-30): E/TEST | At/beyond-floor scores include, below-floor scores exclude, and qualifying hard-band trajectories remain eligible without a global gate. | anchor=I-30:L150-L156
- **I-006/I30-E042** [SRC] (DEEP-I-006.discovery.md:L279| unit=I-30): E/RECOVERY | Missing events/count equality require inspecting `dataset_cmd.rs` and the sealed walker; missing fixture cases must be added; unsealed/tampered/below-floor acceptance requires restoring the guard rather than caller filtering. | anchor=I-30:L158-L166
- **I-006/I30-E043** [SRC] (DEEP-I-006.discovery.md:L280| unit=I-30): E/FORK | Negative-source default is the sealed direct I-28 call; live SSB pairs remain disallowed until `task_runner` carries seal status. | anchor=I-30:L168-L170
- **I-006/I30-E044** [SRC] (DEEP-I-006.discovery.md:L281| unit=I-30): E/FORK | Tier-1 difficulty defaults to fixture/calibration constants; switching to I-25 later must preserve the same inclusion-floor contract fixture. | anchor=I-30:L169-L170
- **I-006/I30-E045** [SRC] (DEEP-I-006.discovery.md:L282| unit=I-30): E/FORK | Module default is sibling `dataset_cmd.rs` plus `sft_exhaust.rs`; the dataset path must not be appended to `rra_cmd.rs`. | anchor=I-30:L170-L171
- **I-006/I30-E046** [SRC] (DEEP-I-006.discovery.md:L283| unit=I-30 SURFACE): E/ABORT | Abort on unsealed ingestion, DPO/PPO/GRPO logic, promotion bypass, fabricated values, non-blake3 hashing, production unwrap/expect, or Tier-1 UI. | anchor=I-30:L173-L180
- **I-006/I30-E047** [SRC] (DEEP-I-006.discovery.md:L284| unit=I-30): E/VERIFY | Required certification covers plan gate, plain-Cargo contract test, production caller check, wiring contract check, and action-type lock; off-agent verifier and serialized integrator results are recorded. | anchor=I-30:L182-L187
- **I-006/I30-E048** [SRC] (DEEP-I-006.discovery.md:L285| unit=I-30): E/REDTEAM | Red-team arms cover unsealed smuggling, tampered hash, count falsification, DPO-training exposure, and absent ID-6 lint. | anchor=I-30:L189-L194
- **I-006/I30-E049** [SRC] (DEEP-I-006.discovery.md:L286| unit=I-30): E/DONE | Integration smoke asserts `PathExtracted`, `DatasetSealed`, equal counts, and all sealed/unsealed/tampered/band fixtures under off-agent ownership. | anchor=I-30:L196-L196
- **I-006/I30-E050** [SRC] (DEEP-I-006.discovery.md:L287| unit=I-30 SURFACE): E/STATUS | No recon question is declared open, but execution must re-grep RRA, task runner, split SSB, CLI, gate-chain, and action-lock seams. | anchor=I-30:L198-L198
- **I-006/I30-C001** [CON] (DEEP-I-006.discovery.md:L288| unit=I-30): C/FORK | Early prose permits re-deriving negative pairs from the search tree when I-28 is absent, but the later binding integration/amendment says call I-28 directly and “do not re-derive pairs.” | anchor=I-30:L18-L21, L74-L75, L106-L110
- **I-006/I30-C002** [CON] (DEEP-I-006.discovery.md:L289| unit=I-30): C/SCHEMA | Winner selection names `TriLayerVerdict(Promoted)`, but I-17’s exhaustive kind list contains no `Promoted` discriminator. | anchor=I-30:L13-L16; I-17:L41-L41, L49-L50, L58-L58
- **I-006/I30-C003** [CON] (DEEP-I-006.discovery.md:L290| unit=I-30 SURFACE): C/THRESHOLD | The amendment says the external numeric table is design rationale and not an ELAI target, yet then makes those exact floors Tier-1 inclusion behavior and requires preserving them when switching to I-25. | anchor=I-30:L114-L120, L122-L144, L168-L170
- **I-006/I30-A001** [AC] (DEEP-I-006.discovery.md:L291| unit=I-30): A/SCHEMA | JSONL field names/version, turn/tool encoding, label vocabulary, canonical RRA-byte serialization, and duplicate-run/idempotent rewrite policy are unspecified. | anchor=I-30:L13-L21, L66-L75
- **I-006/I30-A002** [AC] (DEEP-I-006.discovery.md:L292| unit=I-30): A/ELIGIBILITY | Root-to-verified-`COMPLETE` winner language conflicts conceptually with the later admission of qualifying hard-task partials; which partial trajectory endpoint becomes an export record is not defined. | anchor=I-30:L13-L17, L133-L141
- **I-006/I30-A003** [AC] (DEEP-I-006.discovery.md:L293| unit=I-30): A/GATE | Export-time ID-6 handling is variously “block,” “lint warning,” and “lint error,” while the dataset export records only gate-chain lint status; exact success/exit-code semantics are unresolved. | anchor=I-30:L48-L50, L73-L74, L81-L86, L97-L103, L189-L194
- **I-006/I30-A004** [AC] (DEEP-I-006.discovery.md:L294| unit=I-30): A/EVENT | `DatasetSealed(provenance_mismatch)` is used as a failure shape although the event’s status/error field and whether a failed dataset is semantically “sealed” are not specified. | anchor=I-30:L92-L95, L148-L165
- **I-006/I30-A005** [AC] (DEEP-I-006.discovery.md:L295| unit=I-30): A/MEASURE | I-30-owned calibration of `d_i`, normalization of the proposed structural signals, ties across bands, and score provenance are left undefined. | anchor=I-30:L133-L144, L168-L170
- **I-006/SD-I30-01** [SD] (DEEP-I-006.discovery.md:L583| unit=I-30): CORRECTION | Summary L813 reads as an I-30 LoRA training delivery; the source makes the GPU run Tier 2 and ships only offline extraction/gate linting now. | anchor=I30-E010, I30-E023–I30-E024
- **I-006/SD-I30-02** [SD] (DEEP-I-006.discovery.md:L584| unit=I-30 SURFACE): CORRECTION | Summary L819 says I-30 adds a dataset card; the source explicitly defers the drawer under the owner UI hold and forbids Tier-1 UI. | anchor=I30-E021, I30-E033
- **I-006/SD-I30-03** [SD] (DEEP-I-006.discovery.md:L585| unit=I-30): ADDITION | Exact JSONL content, blake3-over-RRA bytes, three event kinds, equal record/provenance counts, and unsealed/tampered failure evidence were omitted. | anchor=I30-E005–I30-E008, I30-E026–I30-E030
- **I-006/SD-I30-04** [SD] (DEEP-I-006.discovery.md:L586| unit=I-30): ADDITION | Direct sealed `DpoPairExtractor` invocation and prohibition on current live rows were omitted. | anchor=I30-E017, I30-E043
- **I-006/SD-I30-05** [SD] (DEEP-I-006.discovery.md:L587| unit=I-30): CONFLICT | The source first allows re-derivation without I-28, then later forbids it; the summary gives neither fork nor conflict. | anchor=I30-C001
- **I-006/SD-I30-06** [SD] (DEEP-I-006.discovery.md:L588| unit=I-30 SURFACE): CONFLICT | `TriLayerVerdict(Promoted)` is used by I-30 but absent from I-17’s declared schema; the summary does not flag the cross-plan mismatch. | anchor=I30-C002, IX-009
- **I-006/SD-I30-07** [SD] (DEEP-I-006.discovery.md:L589| unit=I-30): ADDITION | J-11’s five difficulty bands, no-global-gate rule, hard-partial preference, I-25/calibration source, and structural features are entirely absent. | anchor=I30-E035–I30-E041
- **I-006/SD-I30-08** [SD] (DEEP-I-006.discovery.md:L590| unit=I-30 SURFACE): CONFLICT | The amendment’s exact external floors are both “not an ELAI target” and binding/preserved Tier-1 behavior; this conflicts with local-threshold doctrine. | anchor=I30-C003, IX-028
- **I-006/SD-I30-09** [SD] (DEEP-I-006.discovery.md:L591| unit=I-30 SURFACE): AMBIGUITY | Verified-COMPLETE winners versus qualifying partials, gate block/warn/error semantics, JSONL/event schema, and `d_i` calibration remain unresolved. | anchor=I30-A001–I30-A005
- **I-006/SD-I30-10** [SD] (DEEP-I-006.discovery.md:L592| unit=I-30): ADDITION | Full fork/abort/recovery/red-team/verification gates, sibling `dataset_cmd.rs` placement, split SSB warning, and mandatory full-form expansion were omitted. | anchor=I30-E001, I30-E034, I30-E042–I30-E050

#### I-006 / unit `I-43` (42 records)

- **I-006/I43-E001** [SRC] (DEEP-I-006.discovery.md:L305| unit=I-43): E/META | Track 19 `DESIGN-PHASE CRITIQUE + TRUST-CLAIM DISCIPLINE`; `PROVABLE/FALSIFY-FIRST`; strong grounding; build; effort M; Wave 1; ID-5; composes I-38/G-6/H-23. | anchor=I-43:L1-L3
- **I-006/I43-E002** [SRC] (DEEP-I-006.discovery.md:L306| unit=I-43): E/STRATEGY | Cheap reduced-N falsifiers must run before expensive G-6 SAFEDELTA work so build/claim investment is evidence-gated rather than faith-based. | anchor=I-43:L4-L12
- **I-006/I43-E003** [SRC] (DEEP-I-006.discovery.md:L307| unit=I-43): E/PILOT | F0.1 drives the shipped SecCodePLT/SAFEDELTA runner gate-on versus gate-off over 50–100 git-stripped tasks with dynamic-exploit scoring. | anchor=I-43:L5-L8, L14-L20
- **I-006/I43-E004** [SRC] (DEEP-I-006.discovery.md:L308| unit=I-43): E/MEASURE | Pilot output includes residual-insecure-rate delta sign, rough magnitude, and reviewer-reaching rate under gate-on. | anchor=I-43:L6-L8, L16-L20
- **I-006/I43-E005** [SRC] (DEEP-I-006.discovery.md:L309| unit=I-43): E/KILL | The pilot fails if the security delta is flat/negative or the apparent win comes from blocking nearly everything before review. | anchor=I-43:L6-L8, L16-L20
- **I-006/I43-E006** [SRC] (DEEP-I-006.discovery.md:L310| unit=I-43): E/CLAIM | Reduced-N output is a private go/no-go memo, not a headline or external marketing claim. | anchor=I-43:L19-L20, L91-L96
- **I-006/I43-E007** [SRC] (DEEP-I-006.discovery.md:L311| unit=I-43): E/SCOPER | F0.2 runs about 20 identical security tasks with fixed seed on local Qwen2.5-Coder and cloud Claude lanes. | anchor=I-43:L8-L12, L21-L26
- **I-006/I43-E008** [SRC] (DEEP-I-006.discovery.md:L312| unit=I-43): E/MEASURE | It measures action-stream divergence on security-critical paths; divergence above 5% forces trace-authentication-only wording. | anchor=I-43:L21-L26, L102-L110
- **I-006/I43-E009** [SRC] (DEEP-I-006.discovery.md:L313| unit=I-43): E/CLAIM | Cloud may claim trace authentication/signed evidence, not deterministic replay; full recomputation is reserved for open-weight mode. | anchor=I-43:L8-L10, L21-L29
- **I-006/I43-E010** [SRC] (DEEP-I-006.discovery.md:L314| unit=I-43): E/DET | Cross-lane claim scoping is distinct from H-23 same-environment replay; every measured number must recompute byte-identically from SSB under `ELAI_SEED`. | anchor=I-43:L25-L27, L102-L110
- **I-006/I43-E011** [SRC] (DEEP-I-006.discovery.md:L315| unit=I-43): E/NON-GOAL | A single large-N run and cloud “deterministic replay” language are declined. | anchor=I-43:L27-L29, L91-L97
- **I-006/I43-E012** [SRC] (DEEP-I-006.discovery.md:L316| unit=I-43 SURFACE): E/CLI | One `elai bench claim-scope` subcommand exposes pilot `--suite … --n 50` and divergence `--mode divergence` modes. | anchor=I-43:L14-L16, L43-L46
- **I-006/I43-E013** [SRC] (DEEP-I-006.discovery.md:L317| unit=I-43): E/DEP | The implementation composes I-38 QDELTA, shipped G-6/SecCodePLT/SAFEDELTA dynamic-exploit/git-strip path, G-2 RRA capture, H-23 comparator shape, and B-37 lane routing. | anchor=I-43:L31-L40, L61-L68
- **I-006/I43-E014** [SRC] (DEEP-I-006.discovery.md:L318| unit=I-43): E/STATUS | A reduced-N go/no-go driver and cross-lane divergence scoper are explicitly absent/net-new. | anchor=I-43:L40-L41
- **I-006/I43-E015** [SRC] (DEEP-I-006.discovery.md:L319| unit=I-43): E/SCOPE | Tier 1 creates `bench_claim_scope.rs` with both modes, trap detection, claim wording, memo writer, and deterministic fixture path. | anchor=I-43:L43-L53
- **I-006/I43-E016** [SRC] (DEEP-I-006.discovery.md:L320| unit=I-43): E/ARTIFACT | `workflow/reasoning/I-43-phase0-memo.md` records either three Phase-0 greens or a rescope. | anchor=I-43:L49-L53, L73-L80
- **I-006/I43-E017** [SRC] (DEEP-I-006.discovery.md:L321| unit=I-43): E/EVENT | New `ClaimScopeMeasured` arms are `PilotDelta`, `CrossLaneDivergence`, and `ClaimScoped`. | anchor=I-43:L51-L53, L73-L80
- **I-006/I43-E018** [SRC] (DEEP-I-006.discovery.md:L322| unit=I-43): E/TEST | A mock runner plus canned RRA traces exercises the real production code path deterministically without GPU/cloud. | anchor=I-43:L52-L53, L65-L68
- **I-006/I43-E019** [SRC] (DEEP-I-006.discovery.md:L323| unit=I-43): E/DEFER | Real SecCodePLT/BaxBench/RunPod and real Claude/Qwen cross-lane runs are separate Tier-2 checkboxes. | anchor=I-43:L55-L57
- **I-006/I43-E020** [SRC] (DEEP-I-006.discovery.md:L324| unit=I-43): E/NON-GOAL | F0.3 willingness-to-pay calls are business validation outside engineering scope, represented only as the memo’s third green. | anchor=I-43:L58-L59
- **I-006/I43-E021** [SRC] (DEEP-I-006.discovery.md:L325| unit=I-43): E/INTEG | Tier 1 also creates the fixture contract and `I-43-decisions.md`; all production instrumentation is composed rather than rebuilt. | anchor=I-43:L61-L68
- **I-006/I43-E022** [SRC] (DEEP-I-006.discovery.md:L326| unit=I-43 SURFACE): E/UI | An H-9-family Phase-0 cockpit tile is optional; W-6 permits no UI because bench TOML plus memo are the load-bearing surfaces. | anchor=I-43:L69-L71, L98-L101
- **I-006/I43-E023** [SRC] (DEEP-I-006.discovery.md:L327| unit=I-43): E/API | Produced capabilities are `bench_claim_scope::run`, reduced-N/trap driver, cross-lane scoper/wording emitter, event, and memo. | anchor=I-43:L73-L80
- **I-006/I43-E024** [SRC] (DEEP-I-006.discovery.md:L328| unit=I-43): E/DEP | Consumers are I-38, shipped SecCodePLT/SAFEDELTA, G-2 RRA, H-23 first-divergence pattern, and B-37 `PerRoleRuntime`. | anchor=I-43:L81-L86
- **I-006/I43-E025** [SRC] (DEEP-I-006.discovery.md:L329| unit=I-43 SURFACE): E/REACH | Both CLI modes must traverse `bench_claim_scope::run` to SSB; test-only callers never satisfy R7. | anchor=I-43:L87-L90
- **I-006/I43-E026** [SRC] (DEEP-I-006.discovery.md:L330| unit=I-43): E/LIMIT | Production bans big-N masquerading as pilot, replay wording on cloud, copied instrumentation, fabricated live results, `.unwrap()`/`.expect()`, and non-blake3 hashes. | anchor=I-43:L91-L97
- **I-006/I43-E027** [SRC] (DEEP-I-006.discovery.md:L331| unit=I-43): E/OBS | Expected rows contain delta sign/magnitude/reviewer rate, cross-lane divergence percentage, and the selected honest wording. | anchor=I-43:L102-L107
- **I-006/I43-E028** [SRC] (DEEP-I-006.discovery.md:L332| unit=I-43): E/REDTEAM | A gate-on fixture that blocks nearly everything must visibly fail; a >5% divergence fixture must emit trace-authentication-only wording. | anchor=I-43:L107-L110
- **I-006/I43-E029** [SRC] (DEEP-I-006.discovery.md:L333| unit=I-43): E/EVIDENCE | External benchmark/security figures in grounding require web confirmation before headline use. | anchor=I-43:L112-L117
- **I-006/I43-E030** [SRC] (DEEP-I-006.discovery.md:L334| unit=I-43): E/INTEG | I-40 consumes I-43’s scoped wording and must not advertise cloud replay. | anchor=I-43:L124-L130
- **I-006/I43-C001** [CON] (DEEP-I-006.discovery.md:L335| unit=I-43): C/THRESHOLD | The source simultaneously says no paper constant is a threshold and hard-codes a >5% divergence wording threshold; provenance and promotion of that constant are not reconciled. | anchor=I-43:L21-L27, L91-L95, L102-L110, L120-L121
- **I-006/I43-A001** [AC] (DEEP-I-006.discovery.md:L336| unit=I-43 SURFACE): A/CFG | Prose says default pilot N is “50–100,” while the concrete Tier-1 command fixes `--n 50`; default/range validation is unclear. | anchor=I-43:L5-L8, L14-L17, L43-L46
- **I-006/I43-A002** [AC] (DEEP-I-006.discovery.md:L337| unit=I-43): A/KILL | “Rough magnitude,” “flat/negative,” and “nearly everything” have no statistical/operational threshold or confidence rule. | anchor=I-43:L5-L8, L16-L20
- **I-006/I43-A003** [AC] (DEEP-I-006.discovery.md:L338| unit=I-43): A/LIFECYCLE | The memo requires three greens although F0.3 is off-tool business work; ownership, representation, and whether either engineering mode can emit final GO are unspecified. | anchor=I-43:L49-L59
- **I-006/I43-A004** [AC] (DEEP-I-006.discovery.md:L339| unit=I-43): A/ARTIFACT | The plan calls bench TOML a primary artifact but gives no filename, schema, or write/idempotency rules for it. | anchor=I-43:L98-L101
- **I-006/I43-A005** [AC] (DEEP-I-006.discovery.md:L340| unit=I-43): A/SCHEMA | Exact action-stream normalization, security-critical-path selection, divergence denominator, and serialized claim-wording vocabulary are undefined. | anchor=I-43:L21-L26, L49-L52, L73-L86
- **I-006/SD-I43-01** [SD] (DEEP-I-006.discovery.md:L598| unit=I-43): ADDITION | Git-stripped tasks, explicit flat/negative kill, and the reduced-N-as-private-memo constraint were omitted. | anchor=I43-E003–I43-E006
- **I-006/SD-I43-02** [SD] (DEEP-I-006.discovery.md:L599| unit=I-43 SURFACE): ADDITION | Exact two-mode CLI, event arms, mock-runner/canned-RRA real-path fixture, and no-test-only R7 rule were omitted. | anchor=I43-E012, I43-E015, I43-E017–I43-E018, I43-E025
- **I-006/SD-I43-03** [SD] (DEEP-I-006.discovery.md:L600| unit=I-43): ADDITION | The memo requires three greens, while F0.3 is off-tool business validation; this completion dependency was omitted. | anchor=I43-E016, I43-E020, I43-A003
- **I-006/SD-I43-04** [SD] (DEEP-I-006.discovery.md:L601| unit=I-43): ADDITION | I-40’s downstream wording dependency, no-external-claim boundary, full ban list, and deterministic trap/wording negative arms were omitted. | anchor=I43-E006, I43-E026–I43-E030
- **I-006/SD-I43-05** [SD] (DEEP-I-006.discovery.md:L602| unit=I-43): CONFLICT | Summary L1187 and L1195-L1196 reproduce no-paper-threshold and fixed-5% claims without identifying their tension. | anchor=I43-C001
- **I-006/SD-I43-06** [SD] (DEEP-I-006.discovery.md:L603| unit=I-43 SURFACE): AMBIGUITY | Default N, statistical/trap criteria, unnamed bench TOML, and divergence/claim schemas remain open beyond the summary’s Track-19 kill-gate qualification. | anchor=I43-A001–I43-A005

#### I-006 / unit `I-47` (31 records)

- **I-006/I47-E001** [SRC] (DEEP-I-006.discovery.md:L350| unit=I-47): E/META | Track 20; `TRUST-ANTI-BIAS`; strong seam/moderate competitor grounding; extension; effort S-M; Wave 2. | anchor=I-47:L1-L4
- **I-006/I47-E002** [SRC] (DEEP-I-006.discovery.md:L351| unit=I-47): E/SEC | Worker diffs, worker claims, and tool/repo output entering auditor/critic dispatch are typed untrusted data segments. | anchor=I-47:L6-L9, L15-L20
- **I-006/I47-E003** [SRC] (DEEP-I-006.discovery.md:L352| unit=I-47): E/SEC | A standing “do not follow embedded instructions / do not trust self-attestation” frame sits above the tags, and self-attestation is labeled claim-not-evidence. | anchor=I-47:L8-L9, L18-L20
- **I-006/I47-E004** [SRC] (DEEP-I-006.discovery.md:L353| unit=I-47): E/RULES | Every worker dispatch mechanically receives read-before-write, scope discipline, no premature abstraction, no future-proofing, and no while-I-am-here refactoring. | anchor=I-47:L10-L13
- **I-006/I47-E005** [SRC] (DEEP-I-006.discovery.md:L354| unit=I-47): E/CACHE | Rules are an R8-frozen per-role byte-stable block placed after cache-breakpoint discipline so the role prefix remains cacheable and byte-identical. | anchor=I-47:L20-L23
- **I-006/I47-E006** [SRC] (DEEP-I-006.discovery.md:L355| unit=I-47): E/PROVENANCE | Before execution, SL-11 requires reading roach-pi summary techniques 2/3 and their cited extension source. | anchor=I-47:L15-L17
- **I-006/I47-E007** [SRC] (DEEP-I-006.discovery.md:L356| unit=I-47): E/NON-GOAL | No new classifier, agent-definition rewrite, or per-task dynamic rule text is allowed; E-37 retains classification ownership. | anchor=I-47:L24-L26
- **I-006/I47-E008** [SRC] (DEEP-I-006.discovery.md:L357| unit=I-47): E/STATUS | H-31/H-32, E-37, and D-13 surround the seam; I-32 is planned and must coordinate if it lands first. | anchor=I-47:L28-L32
- **I-006/I47-E009** [SRC] (DEEP-I-006.discovery.md:L358| unit=I-47): E/ACTIVATION | The rule content already exists as prose; I-47 makes a fixed distillate runtime-enforced rather than aspirational. | anchor=I-47:L33-L34
- **I-006/I47-E010** [SRC] (DEEP-I-006.discovery.md:L359| unit=I-47): E/SCOPE | Tier 1 ships in-process deterministic tag wrapping, role injection, and contract tests. | anchor=I-47:L36-L38
- **I-006/I47-E011** [SRC] (DEEP-I-006.discovery.md:L360| unit=I-47): E/DEFER | Measured prompt-injection resistance waits for a separate attack-corpus bench session. | anchor=I-47:L39-L39
- **I-006/I47-E012** [SRC] (DEEP-I-006.discovery.md:L361| unit=I-47 SURFACE): E/UI | Prompt assembly is internal, so W-6 explicitly justifies no frontend pair. | anchor=I-47:L41-L44
- **I-006/I47-E013** [SRC] (DEEP-I-006.discovery.md:L362| unit=I-47): E/EVENT | Produced outputs are the tag wrapper, frozen role block, and `PromptHardeningApplied` rows. | anchor=I-47:L46-L48
- **I-006/I47-E014** [SRC] (DEEP-I-006.discovery.md:L363| unit=I-47): E/DEP | The feature consumes the agent/auditor prompt builders, upstream E-37 classifier unchanged, and H-31/H-32 firewall objective. | anchor=I-47:L49-L50
- **I-006/I47-E015** [SRC] (DEEP-I-006.discovery.md:L364| unit=I-47): E/REACH | Wrapping is unconditional on every auditor/critic production dispatch; rule injection is unconditional on every worker production dispatch. | anchor=I-47:L51-L52
- **I-006/I47-E016** [SRC] (DEEP-I-006.discovery.md:L365| unit=I-47 SURFACE): E/LIMIT | Per-task rule text, model-authored tag content, and a skip flag are forbidden. | anchor=I-47:L53-L55
- **I-006/I47-E017** [SRC] (DEEP-I-006.discovery.md:L366| unit=I-47): E/LIMIT | PromptArmor may not be copied and worker artifacts may not be stripped/truncated: tags wrap the actual content. | anchor=I-47:L54-L55
- **I-006/I47-E018** [SRC] (DEEP-I-006.discovery.md:L367| unit=I-47): E/OBS | A malicious fixture emits `InputsTagged` and `RulesInjected`; journal bytes must show malicious instructions beneath the frame, claims labeled, and two identical worker role blocks. | anchor=I-47:L56-L65
- **I-006/I47-E019** [SRC] (DEEP-I-006.discovery.md:L368| unit=I-47 SURFACE): E/FAIL | A no-tags build flag must be structurally absent; enabled hardening with zero evidence rows is an R1 failure. | anchor=I-47:L63-L65
- **I-006/I47-E020** [SRC] (DEEP-I-006.discovery.md:L369| unit=I-47): E/SCHEMA | The declared event kinds are `InputsTagged`, `RulesInjected`, and `SelfAttestationStripped`, with baseline +1 and mandatory live-registry reread. | anchor=I-47:L73-L78
- **I-006/I47-E021** [SRC] (DEEP-I-006.discovery.md:L370| unit=I-47): E/INTEG | It composes E-37, H-31/H-32, I-32, D-13, and I-21 without restating their classifier, role-contract, adversary, or separation logic. | anchor=I-47:L67-L77
- **I-006/I47-C001** [CON] (DEEP-I-006.discovery.md:L371| unit=I-47): C/SCHEMA | The prose requires self-attestation to be retained and labeled inside wrapped data, while the event vocabulary calls it `SelfAttestationStripped`; truncation is expressly banned. | anchor=I-47:L18-L20, L53-L55, L61-L65, L73-L78
- **I-006/I47-A001** [AC] (DEEP-I-006.discovery.md:L372| unit=I-47): A/SCHEMA | Exact XML/tag names, escaping/canonicalization, nesting rules, hardening-frame bytes, role-table schema, and freeze-manifest path are omitted. | anchor=I-47:L18-L23, L41-L48
- **I-006/I47-A002** [AC] (DEEP-I-006.discovery.md:L373| unit=I-47 SURFACE): A/DEFAULT | Reachability says behavior is unconditional/no-skip, yet failure prose refers to “while the feature is ON”; no feature flag/default/migration path is defined. | anchor=I-47:L51-L55, L63-L65
- **I-006/I47-A003** [AC] (DEEP-I-006.discovery.md:L374| unit=I-47): A/INTEG | Concrete prompt-builder modules, role taxonomy, and coordination order if I-32 changes those seams are deliberately unnamed. | anchor=I-47:L28-L32, L41-L50
- **I-006/SD-I47-01** [SD] (DEEP-I-006.discovery.md:L609| unit=I-47): ADDITION | The injected rule set also includes “no future-proofing,” omitted from summary L1290-L1293. | anchor=I47-E004
- **I-006/SD-I47-02** [SD] (DEEP-I-006.discovery.md:L610| unit=I-47): ADDITION | SL-11’s mandatory original competitor read, existing-prose activation status, and deferred injection-resistance corpus evaluation were omitted. | anchor=I47-E006, I47-E008–I47-E011
- **I-006/SD-I47-03** [SD] (DEEP-I-006.discovery.md:L611| unit=I-47 SURFACE): ADDITION | Wrapping/rules are unconditional by role; no skip flag, model-authored tags, classification copy, or artifact truncation is allowed. | anchor=I47-E015–I47-E017
- **I-006/SD-I47-04** [SD] (DEEP-I-006.discovery.md:L612| unit=I-47): CONFLICT | The summary mentions labeled self-attestation but omits the source’s `SelfAttestationStripped` kind, which conflicts with the preserve/wrap rule. | anchor=I47-C001
- **I-006/SD-I47-05** [SD] (DEEP-I-006.discovery.md:L613| unit=I-47 SURFACE): ADDITION | No-UI justification, exact malicious fixture bytes, structurally absent no-tags flag, and missing-row failure were omitted. | anchor=I47-E012, I47-E018–I47-E019
- **I-006/SD-I47-06** [SD] (DEEP-I-006.discovery.md:L614| unit=I-47 SURFACE): AMBIGUITY | Tag/frame canonical bytes, frozen-table schema, actual prompt modules, and “feature ON” versus unconditional/no-skip activation are unresolved. | anchor=I47-A001–I47-A003

#### I-006 / unit `I-49` (42 records)

- **I-006/I49-E001** [SRC] (DEEP-I-006.discovery.md:L384| unit=I-49): E/META | Track 20; `REACH`; strong gap/moderate competitor grounding; reference-port build; effort L; Wave 4; live legs network-gated; full-form expansion and anchor re-grep are mandatory before execution. | anchor=I-49:L1-L6
- **I-006/I49-E002** [SRC] (DEEP-I-006.discovery.md:L385| unit=I-49 SURFACE): E/NON-GOAL | The wholesale gajae provider layer is rejected; existing B-37, CLI bridges, OAuth, and local serving remain the architectural base. | anchor=I-49:L8-L13
- **I-006/I49-E003** [SRC] (DEEP-I-006.discovery.md:L386| unit=I-49): E/PROVIDER | AWS Bedrock direct support implements SigV4 request signing and binary EventStream decoding for accounts without Anthropic API keys. | anchor=I-49:L10-L12, L19-L21
- **I-006/I49-E004** [SRC] (DEEP-I-006.discovery.md:L387| unit=I-49): E/PROVIDER | Azure OpenAI direct support implements the Responses API shape for Azure deployments. | anchor=I-49:L11-L13, L19-L23
- **I-006/I49-E005** [SRC] (DEEP-I-006.discovery.md:L388| unit=I-49): E/SHIM | An OpenAI-as-Anthropic normalization pattern presents OpenAI-shaped backends through the Anthropic-shaped internal seam. | anchor=I-49:L12-L13, L19-L23
- **I-006/I49-E006** [SRC] (DEEP-I-006.discovery.md:L389| unit=I-49): E/PROVENANCE | Executor must first read `amazon-bedrock.ts`, `aws-sigv4.ts`, `aws-eventstream.ts`, `aws-credentials.ts`, `azure-openai-responses.ts`, and `openai-anthropic-shim.ts` in the named gajae provider directory. | anchor=I-49:L15-L18
- **I-006/I49-E007** [SRC] (DEEP-I-006.discovery.md:L390| unit=I-49): E/API | Rust implementation lives in `crates/providers/src/runtime/bedrock.rs`, `azure_openai.rs`, and a shared normalization-shim module, modeled on `anthropic_api.rs`. | anchor=I-49:L19-L23, L44-L47
- **I-006/I49-E008** [SRC] (DEEP-I-006.discovery.md:L391| unit=I-49): E/FRESHNESS | Before coding, current AWS InvokeModelWithResponseStream/Converse and Azure Responses documentation must be researched rather than trusted from training. | anchor=I-49:L24-L27
- **I-006/I49-E009** [SRC] (DEEP-I-006.discovery.md:L392| unit=I-49): E/DEP | Candidate AWS crates require registry, repository, last-publish, and license checks under the anti-slopsquat protocol. | anchor=I-49:L24-L27
- **I-006/I49-E010** [SRC] (DEEP-I-006.discovery.md:L393| unit=I-49): E/PERM | Every new dependency needs user approval; hand-roll protocol logic if candidate dependencies outweigh it. | anchor=I-49:L26-L27
- **I-006/I49-E011** [SRC] (DEEP-I-006.discovery.md:L394| unit=I-49 SURFACE): E/NON-GOAL | GitLab Duo, Kimi, Vertex, replacement/removal of adapters or CLI bridges, and API-key storage changes are outside scope. | anchor=I-49:L28-L29
- **I-006/I49-E012** [SRC] (DEEP-I-006.discovery.md:L395| unit=I-49 SURFACE): E/STATUS | Existing runtime adapters include Anthropic API, multiple CLI bridges, `genai_adapter`, Hermes, and local serving; `oauth.rs` stays. | anchor=I-49:L31-L34
- **I-006/I49-E013** [SRC] (DEEP-I-006.discovery.md:L396| unit=I-49): E/GAP | Source records a dated grep finding of zero Bedrock/Azure symbols in `crates/`. | anchor=I-49:L35-L35
- **I-006/I49-E014** [SRC] (DEEP-I-006.discovery.md:L397| unit=I-49): E/SCOPE | Tier 1 ships all protocol logic, deterministic recorded EventStream/Responses replay, and B-37 registry wiring without network. | anchor=I-49:L37-L40
- **I-006/I49-E015** [SRC] (DEEP-I-006.discovery.md:L398| unit=I-49): E/DEFER | Real Bedrock and Azure calls are separate Tier-2 checkboxes requiring credentials/deployment and documented environment. | anchor=I-49:L41-L42
- **I-006/I49-E016** [SRC] (DEEP-I-006.discovery.md:L399| unit=I-49 SURFACE): E/UI | `elai runtime list` and the existing `RuntimePanel` list both expose `bedrock` and `azure-openai`; no new component. | anchor=I-49:L44-L47, L60-L67
- **I-006/I49-E017** [SRC] (DEEP-I-006.discovery.md:L400| unit=I-49): E/SYMBOL | Produced surfaces are Bedrock SigV4/EventStream, Azure Responses, OpenAI→Anthropic normalization, and replay fixtures. | anchor=I-49:L49-L51
- **I-006/I49-E018** [SRC] (DEEP-I-006.discovery.md:L401| unit=I-49): E/DEP | The adapters consume B-37 `RuntimeAdapter`/registry, conditionally `genai_adapter`, and I-51 pricing/catalog metadata when available. | anchor=I-49:L52-L53
- **I-006/I49-E019** [SRC] (DEEP-I-006.discovery.md:L402| unit=I-49 SURFACE): E/REACH | Selecting either backend traverses the real PerRoleRuntime request builder; `elai runtime list` proves registry reach and contracts-only callers do not satisfy R7. | anchor=I-49:L54-L55
- **I-006/I49-E020** [SRC] (DEEP-I-006.discovery.md:L403| unit=I-49): E/TEST | SigV4 fixtures sign real canonical requests with recorded keys and must reproduce a documented known-good signature; mock signing is forbidden. | anchor=I-49:L56-L57, L60-L67
- **I-006/I49-E021** [SRC] (DEEP-I-006.discovery.md:L404| unit=I-49): E/FAIL | Malformed EventStream frames fail closed with typed errors and may not panic or silently fall back to a different backend. | anchor=I-49:L56-L59, L65-L67
- **I-006/I49-E022** [SRC] (DEEP-I-006.discovery.md:L405| unit=I-49): E/SEC | Production paths ban unwrap/expect and unverified dependencies. | anchor=I-49:L56-L59
- **I-006/I49-E023** [SRC] (DEEP-I-006.discovery.md:L406| unit=I-49): E/EVENT | No action type is added; existing `RouteDecisionEvent`/runtime rows carry backend IDs. | anchor=I-49:L60-L64, L74-L77
- **I-006/I49-E024** [SRC] (DEEP-I-006.discovery.md:L407| unit=I-49): E/DONE | Recorded frames and Responses payloads decode byte-exactly; corruption is the negative arm, and each enabled live checkbox must land one real completion. | anchor=I-49:L37-L42, L60-L67
- **I-006/I49-E025** [SRC] (DEEP-I-006.discovery.md:L408| unit=I-49): E/INTEG | J-15 amendment requires fixtures to come from J-14 `rra export-fixture` repro packs rather than a private format. | anchor=I-49:L79-L87
- **I-006/I49-E026** [SRC] (DEEP-I-006.discovery.md:L409| unit=I-49): E/DET | Replay matching key is `(model_id, blake3(message_content), tool_name)`; all three fields must match and positional indices are forbidden. | anchor=I-49:L85-L92
- **I-006/I49-E027** [SRC] (DEEP-I-006.discovery.md:L410| unit=I-49): E/REDTEAM | Positional matching is a do-not-copy exhibit because insertion, retry-with-different-content, or skipped calls shift/mask subsequent fixtures. | anchor=I-49:L94-L104
- **I-006/I49-E028** [SRC] (DEEP-I-006.discovery.md:L411| unit=I-49): E/PRIV | Fixtures must contain no auth headers; J-14’s export scrub is the inherited counter-design, and any header-bearing fixture is a credential leak. | anchor=I-49:L106-L110
- **I-006/I49-E029** [SRC] (DEEP-I-006.discovery.md:L412| unit=I-49 SURFACE): E/INTEG | Composition retains B-37 as the frame, OAuth/CLI bridges as keep-list paths, I-50 as future credential supplier, and I-51 as catalog owner. | anchor=I-49:L69-L77
- **I-006/I49-A001** [AC] (DEEP-I-006.discovery.md:L413| unit=I-49): A/INTEG | The shim extends `genai_adapter` only if its live seam fits; otherwise module ownership is unresolved. | anchor=I-49:L19-L23, L52-L53, L74-L77
- **I-006/I49-A002** [AC] (DEEP-I-006.discovery.md:L414| unit=I-49): A/PROTOCOL | InvokeModelWithResponseStream versus Converse, candidate-crate versus hand-roll, retry/backoff, streaming cancellation, and Azure API-version choices remain pre-task decisions. | anchor=I-49:L24-L27
- **I-006/I49-A003** [AC] (DEEP-I-006.discovery.md:L415| unit=I-49): A/AUTH | Bedrock consumes credentials and a competitor `aws-credentials.ts` shape, but storage changes are excluded and I-50 is future; the Tier-1 credential acquisition seam is not defined. | anchor=I-49:L15-L18, L28-L29, L41-L42, L74-L77
- **I-006/I49-A004** [AC] (DEEP-I-006.discovery.md:L416| unit=I-49): A/SCHEMA | Stable backend IDs, normalized event schema, typed error variants, and runtime-row payload fields are not specified. | anchor=I-49:L44-L67
- **I-006/I49-A005** [AC] (DEEP-I-006.discovery.md:L417| unit=I-49): A/DEP | J-14 fixture availability/version compatibility and scrub-verification behavior are assumed, not status-checked in this source. | anchor=I-49:L79-L110
- **I-006/I49-A006** [AC] (DEEP-I-006.discovery.md:L418| unit=I-49): A/REACH | Tier 1 requires the production request-builder path to be reachable while all real network calls are deferred; how fixture transport is selected without a test-only caller is unspecified. | anchor=I-49:L37-L42, L54-L55, L60-L67
- **I-006/SD-I49-01** [SD] (DEEP-I-006.discovery.md:L620| unit=I-49): ADDITION | All six mandatory TypeScript source files and the full registry/repository/publish/license dependency checks were omitted. | anchor=I49-E006, I49-E009–I49-E010
- **I-006/SD-I49-02** [SD] (DEEP-I-006.discovery.md:L621| unit=I-49): ADDITION | Hand-roll-if-heavier policy; GitLab/Kimi/Vertex, storage, and adapter-removal exclusions; and the existing adapter/keep-list inventory were omitted. | anchor=I49-E002, I49-E010–I49-E013
- **I-006/SD-I49-03** [SD] (DEEP-I-006.discovery.md:L622| unit=I-49): ADDITION | No silent backend fallback, real rather than mock canonical signing, user-approved deps, and typed no-panic failure are stronger than the summary’s corrupt-frame sentence. | anchor=I49-E020–I49-E022
- **I-006/SD-I49-04** [SD] (DEEP-I-006.discovery.md:L623| unit=I-49): ADDITION | J-15 makes J-14 repro packs and `(model_id, blake3(content), tool_name)` matching binding; positional fixture matching is an explicit do-not-copy pattern. | anchor=I49-E025–I49-E027
- **I-006/SD-I49-05** [SD] (DEEP-I-006.discovery.md:L624| unit=I-49): ADDITION | Fixture auth headers are prohibited and inherited J-14 scrubbing is a privacy/credential acceptance rule. | anchor=I49-E028
- **I-006/SD-I49-06** [SD] (DEEP-I-006.discovery.md:L625| unit=I-49 SURFACE): AMBIGUITY | Shim placement, AWS/Azure API/dependency choice, credential supply, backend/error schemas, J-14 compatibility, and production-vs-fixture reach remain open. | anchor=I49-A001–I49-A006
- **I-006/SD-I49-07** [SD] (DEEP-I-006.discovery.md:L626| unit=I-49): QUALIFICATION | Summary L1343-L1344 implies all three register as adapters; the source describes two adapter modules plus a shared normalization module whose `genai_adapter` ownership is conditional. | anchor=I49-E007, I49-E017–I49-E018, I49-A001

#### I-006 / unit `I-53` (33 records)

- **I-006/I53-E001** [SRC] (DEEP-I-006.discovery.md:L428| unit=I-53): E/META | Track 20; `TOOLS`; strong gap/moderate competitor grounding; build; effort S-M; Wave 2. | anchor=I-53:L1-L4
- **I-006/I53-E002** [SRC] (DEEP-I-006.discovery.md:L429| unit=I-53): E/PREMISE | Repository recipes are owner-sanctioned real entry points; first-class tools replace arbitrary, undiscoverable/riskier bash shapes. | anchor=I-53:L6-L12
- **I-006/I53-E003** [SRC] (DEEP-I-006.discovery.md:L430| unit=I-53): E/RECIPE | `recipe.rs` discovers recipes with descriptions and runs a selected named recipe through existing validation. | anchor=I-53:L14-L20
- **I-006/I53-E004** [SRC] (DEEP-I-006.discovery.md:L431| unit=I-53): E/CAP | Tier-1 discovery covers Justfile targets, Makefile targets, and Cargo aliases. | anchor=I-53:L15-L18, L34-L36
- **I-006/I53-E005** [SRC] (DEEP-I-006.discovery.md:L432| unit=I-53): E/ARCHIVE | `archive_reader.rs` lists and reads zip/tar.gz entries through read-only streams and performs no extraction to disk. | anchor=I-53:L19-L21, L51-L53
- **I-006/I53-E006** [SRC] (DEEP-I-006.discovery.md:L433| unit=I-53): E/JSON | `json_tree.rs` performs path queries and bounded depth/size subtree rendering, explicitly marking truncation. | anchor=I-53:L20-L21, L51-L53
- **I-006/I53-E007** [SRC] (DEEP-I-006.discovery.md:L434| unit=I-53): E/PROVENANCE | Before execution, SL-10 requires reading gajae’s recipe directory, `archive-reader.ts`, and `json-tree.ts`. | anchor=I-53:L14-L16
- **I-006/I53-E008** [SRC] (DEEP-I-006.discovery.md:L435| unit=I-53): E/API | New files are `crates/tools/src/{recipe.rs,archive_reader.rs,json_tree.rs}`. | anchor=I-53:L17-L21, L38-L40
- **I-006/I53-E009** [SRC] (DEEP-I-006.discovery.md:L436| unit=I-53 SURFACE): E/DECLINE | Cron, monitor, and background-job tools are permanently declined—not deferred—because orchestrator/Layer-1A schedules own them. | anchor=I-53:L22-L24
- **I-006/I53-E010** [SRC] (DEEP-I-006.discovery.md:L437| unit=I-53): E/PERM | Existing archive dependencies must be checked first; a new format dependency requires anti-slopsquat review plus user approval, with hand-rolled tar listing as the fallback. | anchor=I-53:L25-L27
- **I-006/I53-E011** [SRC] (DEEP-I-006.discovery.md:L438| unit=I-53): E/GAP | A dated SL-9 grep found no recipe/Just/Make runner in the intentionally lean tools crate. | anchor=I-53:L29-L32
- **I-006/I53-E012** [SRC] (DEEP-I-006.discovery.md:L439| unit=I-53): E/SCOPE | Tier 1 ships and catalog-registers all three tools through existing validation gates. | anchor=I-53:L34-L35
- **I-006/I53-E013** [SRC] (DEEP-I-006.discovery.md:L440| unit=I-53): E/DEFER | `taskfile.yml` and npm-scripts discovery wait until a target repository needs them. | anchor=I-53:L36-L36
- **I-006/I53-E014** [SRC] (DEEP-I-006.discovery.md:L441| unit=I-53): E/INTEG | Catalog wiring follows the existing three-arm pattern; recipe commands flow through `bash_validation` and E-30 argv, and may become an I-46 redirect target later. | anchor=I-53:L38-L42
- **I-006/I53-E015** [SRC] (DEEP-I-006.discovery.md:L442| unit=I-53 SURFACE): E/UI | Existing tool-call drawer rows render the three tools; no new component. | anchor=I-53:L41-L42
- **I-006/I53-E016** [SRC] (DEEP-I-006.discovery.md:L443| unit=I-53): E/SYMBOL | Produced namespaces are `tools::recipe::{list,run}`, `tools::archive_reader::{list,read}`, and `tools::json_tree::{query,render}` plus registrations. | anchor=I-53:L44-L46
- **I-006/I53-E017** [SRC] (DEEP-I-006.discovery.md:L444| unit=I-53): E/DEP | Recipe execution consumes existing `tools::bash_validation`, E-30 argv allowlist, and the tool registry without bypass or duplication. | anchor=I-53:L47-L48
- **I-006/I53-E018** [SRC] (DEEP-I-006.discovery.md:L445| unit=I-53): E/REACH | Production agent tool dispatch on real repositories must reach each tool; contracts-only callers do not satisfy R7. | anchor=I-53:L49-L50
- **I-006/I53-E019** [SRC] (DEEP-I-006.discovery.md:L446| unit=I-53): E/LIMIT | Recipe gate bypass, archive extraction, unbounded JSON output, unapproved dependencies, and production unwrap/expect are banned. | anchor=I-53:L51-L54
- **I-006/I53-E020** [SRC] (DEEP-I-006.discovery.md:L447| unit=I-53): E/EVENT | No action type is added; existing `tool_call` rows carry the three tool IDs. | anchor=I-53:L55-L59, L68-L70
- **I-006/I53-E021** [SRC] (DEEP-I-006.discovery.md:L448| unit=I-53): E/TEST | A live fixture must list Justfile targets and run one recipe with a journaled bash-validation verdict; absence of that verdict is the broken arm and fails R1. | anchor=I-53:L55-L62
- **I-006/I53-E022** [SRC] (DEEP-I-006.discovery.md:L449| unit=I-53): E/TEST | Archive-read leaves the fixture directory byte-for-byte unwritten, while a 5 MB JSON query stays below the cap and marks truncation on the oversized arm. | anchor=I-53:L55-L62
- **I-006/I53-E023** [SRC] (DEEP-I-006.discovery.md:L450| unit=I-53): E/INTEG | H-4 tool search supplies discoverability; E-35 supplies catalog pattern; I-46 is only a future redirect consumer. | anchor=I-53:L64-L70
- **I-006/I53-A001** [AC] (DEEP-I-006.discovery.md:L451| unit=I-53): A/RECIPE | Parsing rules for phony/pattern/private targets and Cargo config variants, recipe arguments/environment/cwd, descriptions, target validation, and duplicate names are unspecified. | anchor=I-53:L6-L10, L14-L20
- **I-006/I53-A002** [AC] (DEEP-I-006.discovery.md:L452| unit=I-53): A/CFG | JSON path syntax, depth/byte caps, encoding, error shape, and whether limits are configurable are unspecified. | anchor=I-53:L20-L21, L51-L53, L55-L62
- **I-006/I53-A003** [AC] (DEEP-I-006.discovery.md:L453| unit=I-53): A/SEC | Archive size/entry caps, decompression-bomb protection, symlink/path handling, encrypted entries, and malformed-archive recovery are not defined despite read-only operation. | anchor=I-53:L19-L21, L51-L62
- **I-006/I53-A004** [AC] (DEEP-I-006.discovery.md:L454| unit=I-53): A/DEP | Which archive crates already exist and exactly what “hand-roll tar listing” supports are execution-time dependency-tree discoveries. | anchor=I-53:L25-L27
- **I-006/SD-I53-01** [SD] (DEEP-I-006.discovery.md:L632| unit=I-53): ADDITION | Recipe descriptions and owner-sanctioned-entrypoint rationale were omitted; taskfile/npm support is explicitly deferred despite taskfile appearing in the premise. | anchor=I53-E002–I53-E004, I53-E013
- **I-006/SD-I53-02** [SD] (DEEP-I-006.discovery.md:L633| unit=I-53): ADDITION | Exact Rust modules/symbols, catalog three-arm wiring, H-4 discoverability, and future I-46 redirect relationship were omitted. | anchor=I53-E008, I53-E014, I53-E016–I53-E018, I53-E023
- **I-006/SD-I53-03** [SD] (DEEP-I-006.discovery.md:L634| unit=I-53): ADDITION | Archive scope is zip/tar.gz streamed read-only; the JSON fixture is 5 MB and must stay capped with a truncation marker. | anchor=I53-E005–I53-E006, I53-E022
- **I-006/SD-I53-04** [SD] (DEEP-I-006.discovery.md:L635| unit=I-53): QUALIFICATION | Cron/monitor/background jobs are declined permanently, not deferred backlog. | anchor=I53-E009
- **I-006/SD-I53-05** [SD] (DEEP-I-006.discovery.md:L636| unit=I-53): ADDITION | The dependency fallback may hand-roll tar listing, and production unwrap/expect plus archive extraction are explicitly banned. | anchor=I53-E010, I53-E019
- **I-006/SD-I53-06** [SD] (DEEP-I-006.discovery.md:L637| unit=I-53 SURFACE): AMBIGUITY | Recipe syntax/arguments, JSON cap/path contract, archive bomb/path protections, and actual dependency inventory remain unspecified and were not surfaced by the summary. | anchor=I53-A001–I53-A004

#### I-006 / unit `RELATIONAL` (51 records)

- **I-006/IX-001** [IX] (DEEP-I-006.discovery.md:L462| unit=RELATIONAL): E | I-16 evaluates the already-shipped local inference levers without altering their serving defaults; only a later live measured Adopt row may change configuration. | anchor=I16-C001, I16-E003–I16-E008, I16-E021–I16-E024
- **I-006/IX-002** [IX] (DEEP-I-006.discovery.md:L463| unit=RELATIONAL): E | I-17 is the receipt-producing center of this chunk: I-24 injects into its L2 path, I-26 uses its L2 to rank a collapse frontier, and I-30 consumes its pass/veto evidence. | anchor=I17-E003–I17-E009; I24-E015, I24-E020; I26-E005; I30-E004, I30-E007, I30-E030
- **I-006/IX-003** [IX] (DEEP-I-006.discovery.md:L464| unit=RELATIONAL): I | I-19’s pre-write predicate composes F-1, which I-17 later consumes as an L1 axis; therefore a source edit rejected by I-19 should never become an I-17 candidate, while an allowed failing-test edit may still be hard-vetoed later. | anchor=I19-E002, I19-E016, I19-E023–I19-E025; I17-E003, I17-E016
- **I-006/IX-004** [IX] (DEEP-I-006.discovery.md:L465| unit=RELATIONAL): E | I-24 neutralizes only L2 scorer weight and can never weaken I-17’s L1 supremacy; malformed canaries are caught by L1 before any scorer-bias conclusion. | anchor=I24-E006–I24-E009, I24-E022; I17-E004
- **I-006/IX-005** [IX] (DEEP-I-006.discovery.md:L466| unit=RELATIONAL): I | Frozen canaries (I-24) and the off-loop frozen checkpoint evaluator (I-30) are complementary anti-self-evaluation controls at scorer and training-promotion layers. | anchor=I24-E010, I24-E021; I30-E013
- **I-006/IX-006** [IX] (DEEP-I-006.discovery.md:L467| unit=RELATIONAL): E | I-26 collapse stops exploration first, then invokes I-17 L2 on only the current frontier and forbids any later expansion row. | anchor=I26-E004–I26-E006, I26-E023; I17-E008
- **I-006/IX-007** [IX] (DEEP-I-006.discovery.md:L468| unit=RELATIONAL): E | I-26 owns budget control but emits/feeds I-27 convergence evidence rather than adding a new event family; adaptive tuning waits for I-27 data. | anchor=I26-E005, I26-E009, I26-E015, I26-A002
- **I-006/IX-008** [IX] (DEEP-I-006.discovery.md:L469| unit=RELATIONAL): E | I-30 cannot operate until I-4 yields sealed search trajectories, and its positive/negative extraction presumes I-17 L1/L2 verdict evidence. | anchor=I30-E001, I30-E003–I30-E007, I30-E016
- **I-006/IX-009** [IX] (DEEP-I-006.discovery.md:L470| unit=RELATIONAL): C | I-30’s `TriLayerVerdict(Promoted)` selector is not representable by I-17’s enumerated four-kind schema without a schema or selector correction. | anchor=I30-C002, I17-E015
- **I-006/IX-010** [IX] (DEEP-I-006.discovery.md:L471| unit=RELATIONAL): C | I-30’s negative-source lifecycle has two incompatible branches: early fallback re-derivation if I-28 is absent versus the later direct-only sealed I-28 call. | anchor=I30-E017, I30-E043, I30-C001
- **I-006/IX-011** [IX] (DEEP-I-006.discovery.md:L472| unit=RELATIONAL): E | I-30 is SFT-only and produces the initialization/data substrate; I-31 GRPO remains later/last and cannot be smuggled into this unit. | anchor=I30-E002, I30-E009–I30-E010, I30-E031, I30-E046
- **I-006/IX-012** [IX] (DEEP-I-006.discovery.md:L473| unit=RELATIONAL): E | Both I-16 and I-30 keep any MoE retarget as an independent RunPod measurement and forbid assuming a model swap. | anchor=I16-E005, I16-E013, I16-E022; I30-E011, I30-E023
- **I-006/IX-013** [IX] (DEEP-I-006.discovery.md:L474| unit=RELATIONAL): E | I-43 runs cheap I-38/G-6 instrumentation first and can kill/rescope the expensive trust build or its claim without replacing the product DONE spine. | anchor=I43-E002–I43-E006, I43-E013, I43-E026
- **I-006/IX-014** [IX] (DEEP-I-006.discovery.md:L475| unit=RELATIONAL): E | I-43 produces the cloud/open-weight wording contract consumed by I-40: cloud trace authentication is allowed, deterministic replay is not. | anchor=I43-E008–I43-E010, I43-E030
- **I-006/IX-015** [IX] (DEEP-I-006.discovery.md:L476| unit=RELATIONAL): E | I-47 hardens every worker artifact/tool result before an auditor or critic sees it, preserving I-17/D-13 evaluation authority from embedded instructions and self-attestation. | anchor=I47-E002–I47-E003, I47-E014–I47-E018; I17-E017
- **I-006/IX-016** [IX] (DEEP-I-006.discovery.md:L477| unit=RELATIONAL): I | I-53’s new tool outputs become untrusted tool-result segments under I-47 on later auditor dispatch, so adding utilities does not enlarge the auditor’s trusted-data plane. | anchor=I53-E016–I53-E020; I47-E002–I47-E003
- **I-006/IX-017** [IX] (DEEP-I-006.discovery.md:L478| unit=RELATIONAL): I | I-49 adds cloud backends that will eventually need I-26 provider-budget compatibility and I-51 capability metadata; neither source defines the cross-backend reasoning-effort mapping. | anchor=I49-E018, I49-E029, I49-A004; I26-E007, I26-A003
- **I-006/IX-018** [IX] (DEEP-I-006.discovery.md:L479| unit=RELATIONAL): E | I-49 keeps credential storage outside scope and composes future I-50 auth plus I-51 catalog rather than absorbing either concern. | anchor=I49-E011, I49-E018, I49-E029, I49-A003
- **I-006/IX-019** [IX] (DEEP-I-006.discovery.md:L480| unit=RELATIONAL SURFACE): E | I-53 recipe commands must pass E-30/bash validation and are only a future I-46 redirect target; neither feature may bypass the existing command gate. | anchor=I53-E014, I53-E017, I53-E021, I53-E023
- **I-006/IX-020** [IX] (DEEP-I-006.discovery.md:L481| unit=RELATIONAL): E | Seven sources add action families (`KvCacheTuned`, `TriLayerVerdict`, `TddPreflightBlocked`, `PrmCanaryEvent`, `SftExhaustEvent`, `ClaimScopeMeasured`, `PromptHardeningApplied`); I-26, I-49, and I-53 explicitly reuse existing rows. | anchor=I16-E011; I17-E015; I19-E014; I24-E016; I30-E026; I43-E017; I47-E020; I26-E009; I49-E023; I53-E020
- **I-006/IX-021** [IX] (DEEP-I-006.discovery.md:L482| unit=RELATIONAL): I | Those seven baseline-relative registry additions are a schema/concurrency hotspot; each executor must reread and serialize against live action locks rather than independently applying dated `+1` assumptions. | anchor=IX-020; I16-E015, I17-E012, I24-E012, I43-E017, I47-E020
- **I-006/IX-022** [IX] (DEEP-I-006.discovery.md:L483| unit=RELATIONAL): E | New integrity keys use blake3: I-16 config hash, I-17 quarantine key, I-30 RRA provenance, and I-49 fixture-content hash; multiple plans expressly ban blake2/sha2. | anchor=I16-E023, I16-E030; I17-E006, I17-E022; I30-E006, I30-E032; I49-E026
- **I-006/IX-023** [IX] (DEEP-I-006.discovery.md:L484| unit=RELATIONAL): E | Live/GPU/network or new-authority work remains isolated behind checkboxes/approval: I-16 live eval, I-24 new languages, I-30 LoRA/MoE/deps, I-43 live lanes, I-49 live calls/deps, and I-53 archive deps. | anchor=I16-E013; I24-E014; I30-E023; I43-E019; I49-E010, I49-E015; I53-E010
- **I-006/IX-024** [IX] (DEEP-I-006.discovery.md:L485| unit=RELATIONAL SURFACE): E | UI posture is additive/minimal: I-17/I-19/I-24/I-26/I-49/I-53 reuse existing surfaces, I-16/I-47 justify none, I-30 defers its drawer, and I-43 makes its tile optional. | anchor=I16-E016; I17-E023; I19-E017; I24-E017; I26-E021; I30-E021, I30-E033; I43-E022; I47-E012; I49-E016; I53-E015
- **I-006/IX-025** [IX] (DEEP-I-006.discovery.md:L486| unit=RELATIONAL): E | Frozen/off-agent boundaries occur at canary authorship (I-24), budget-controller authority (I-26), checkpoint evaluation/training artifacts (I-30), and role-rule injection (I-47). | anchor=I24-E010, I24-E021; I26-E011; I30-E013; I47-E005
- **I-006/IX-026** [IX] (DEEP-I-006.discovery.md:L487| unit=RELATIONAL): E | Every source except I-47 explicitly bans production unwrap/expect or equivalent unsafe failure; typed fail-closed behavior is load-bearing in I-16, I-17, I-19, I-24, I-26, I-30, I-43, I-49, and I-53. | anchor=I16-E023; I17-E022; I19-E022; I24-E023; I26-E020; I30-E032; I43-E026; I49-E021–I49-E022; I53-E019
- **I-006/IX-027** [IX] (DEEP-I-006.discovery.md:L488| unit=RELATIONAL): E | Deterministic offline fixtures are the common delivery strategy for gated live systems: I-16 no-GPU matrix, I-17 seeded flake/veto, I-19 staged edit, I-24 seeded canary, I-26 collapse, I-30 sealed RRA, I-43 canned traces, I-49 recorded protocol frames, and I-53 repository/archive/JSON fixtures. | anchor=I16-E024; I17-E024–I17-E026; I19-E023–I19-E026; I24-E024–I24-E028; I26-E022–I26-E023; I30-E040–I30-E049; I43-E018, I43-E027–I43-E028; I49-E020, I49-E024; I53-E021–I53-E022
- **I-006/IX-028** [IX] (DEEP-I-006.discovery.md:L489| unit=RELATIONAL): C | “No paper constant as product threshold” recurs, but I-30 makes a cited external five-band table binding and I-43 fixes 5%; both require explicit reconciliation rather than silent promotion. | anchor=I16-E008; I24-E007; I26-E020; I30-C003; I43-C001
- **I-006/AC-001** [SD] (DEEP-I-006.discovery.md:L497| unit=RELATIONAL): correction/status | I-16 is evaluation over shipped KV levers, not new pruning/quant flags or default changes. | anchor=I16-C001, I16-E002–I16-E005
- **I-006/AC-002** [AC] (DEEP-I-006.discovery.md:L498| unit=RELATIONAL): ambiguity/measurement | I-16 leaves adoption band/VRAM threshold and matrix-to-single-artifact aggregation unspecified. | anchor=I16-A001–I16-A002
- **I-006/AC-003** [AC] (DEEP-I-006.discovery.md:L499| unit=RELATIONAL): ambiguity/recovery | I-17 omits flake rerun count, persistence, rehabilitation, and L3 variance threshold. | anchor=I17-A001–I17-A002
- **I-006/AC-004** [AC] (DEEP-I-006.discovery.md:L500| unit=RELATIONAL): ambiguity/integration | I-17’s live hook locations, temporary calibration stub lifecycle, and `HaltingBound` schema remain unresolved. | anchor=I17-A003–I17-A004
- **I-006/AC-005** [SD] (DEEP-I-006.discovery.md:L501| unit=RELATIONAL): correction/scope | I-19 Tier 1 uses symbol/line-name heuristics; full line-level coverage instrumentation is deferred. | anchor=I19-C001
- **I-006/AC-006** [AC] (DEEP-I-006.discovery.md:L502| unit=RELATIONAL SURFACE): ambiguity/language | I-19 names TypeScript scope without a TS dry-run command and leaves attribution/name matching undefined. | anchor=I19-A002–I19-A003
- **I-006/AC-007** [AC] (DEEP-I-006.discovery.md:L503| unit=RELATIONAL): ambiguity/event | `RefactorExempted` is non-blocking but lives under `TddPreflightBlocked`. | anchor=I19-A004
- **I-006/AC-008** [AC] (DEEP-I-006.discovery.md:L504| unit=RELATIONAL): ambiguity/config | I-24 omits `p`, `N`, threshold calibration/version, reset, and reverse-direction score-delta behavior. | anchor=I24-A001–I24-A003
- **I-006/AC-009** [CON] (DEEP-I-006.discovery.md:L505| unit=RELATIONAL): conflict/threshold | I-26’s 85% boundary is variously `>`, `>=`, and “at.” | anchor=I26-C001
- **I-006/AC-010** [AC] (DEEP-I-006.discovery.md:L506| unit=RELATIONAL): ambiguity/schema | I-26’s budget constants, schema migration, collapse event owner, and provider-version mapping are open. | anchor=I26-A001–I26-A003
- **I-006/AC-011** [CON] (DEEP-I-006.discovery.md:L507| unit=RELATIONAL): conflict/dependency | I-30 both permits and forbids re-deriving negatives when I-28 is unavailable. | anchor=I30-C001
- **I-006/AC-012** [CON] (DEEP-I-006.discovery.md:L508| unit=RELATIONAL): conflict/schema | I-30 selects `TriLayerVerdict(Promoted)`, which I-17 does not define. | anchor=I30-C002, IX-009
- **I-006/AC-013** [CON] (DEEP-I-006.discovery.md:L509| unit=RELATIONAL): conflict/threshold | I-30 treats cited band numbers as non-target rationale and binding Tier-1 floors simultaneously. | anchor=I30-C003
- **I-006/AC-014** [AC] (DEEP-I-006.discovery.md:L510| unit=RELATIONAL): ambiguity/eligibility | I-30’s verified-COMPLETE winner model is not reconciled with hard-band qualifying partial trajectories. | anchor=I30-A002
- **I-006/AC-015** [AC] (DEEP-I-006.discovery.md:L511| unit=RELATIONAL): ambiguity/gate | I-30 alternates among block, warning, and error for missing ID-6 evidence; JSONL/event schemas and calibration remain incomplete. | anchor=I30-A001, I30-A003–I30-A005
- **I-006/AC-016** [CON] (DEEP-I-006.discovery.md:L512| unit=RELATIONAL): conflict/threshold | I-43’s no-paper-threshold rule coexists with a hard 5% wording cutoff. | anchor=I43-C001
- **I-006/AC-017** [AC] (DEEP-I-006.discovery.md:L513| unit=RELATIONAL): ambiguity/kill | I-43 does not quantify rough/flat/block-everything criteria, reconcile N=50 with 50–100, or define how off-tool F0.3 completes the memo. | anchor=I43-A001–I43-A003
- **I-006/AC-018** [AC] (DEEP-I-006.discovery.md:L514| unit=RELATIONAL): ambiguity/artifact | I-43 leaves bench artifact and divergence/wording schemas undefined. | anchor=I43-A004–I43-A005
- **I-006/AC-019** [CON] (DEEP-I-006.discovery.md:L515| unit=RELATIONAL): conflict/security | I-47 says label/preserve self-attestation while its event says `SelfAttestationStripped`; tag/table bytes and enablement semantics are also open. | anchor=I47-C001, I47-A001–I47-A003
- **I-006/AC-020** [AC] (DEEP-I-006.discovery.md:L516| unit=RELATIONAL): ambiguity/protocol | I-49 leaves shim placement, API/crate choices, credential acquisition, IDs/errors, J-14 compatibility, and fixture-vs-production reach unresolved. | anchor=I49-A001–I49-A006
- **I-006/AC-021** [AC] (DEEP-I-006.discovery.md:L517| unit=RELATIONAL): ambiguity/tooling | I-53 leaves recipe parsing/execution semantics, JSON caps, archive bomb/path handling, and dependency fallback scope unresolved. | anchor=I53-A001–I53-A004
- **I-006/AC-022** [CON] (DEEP-I-006.discovery.md:L518| unit=RELATIONAL): conflict/cross-cutting | Seven action families plus reused-row payload changes require serialized live-schema reconciliation; dated baseline arithmetic is not independently safe. | anchor=IX-020–IX-021
- **I-006/AC-023** [CON] (DEEP-I-006.discovery.md:L519| unit=RELATIONAL): conflict/cross-cutting | External numeric thresholds in I-30/I-43 conflict with the chunk-wide local-measurement doctrine. | anchor=IX-028

### Ledger SUMMARY — orientation inventory

Source: `docs/thefinalglance/feature-discovery/09-I-SUMMARY_INDEX.discovery.md`
Extracted atoms: **1022**

#### SUMMARY / unit `CROSS` (21 records)

- **SUMMARY/I.CROSS.01** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L47| unit=CROSS): [E] I is described as the first letter authored after the wiring bedrock was reached; every session is intended as feature engineering over an already wiring-disciplined substrate.
- **SUMMARY/I.CROSS.02** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L49| unit=CROSS): [E] Every summary entry records track and lever, grounding strength, nature, the existing substrate or absent code, a COMPOSES-WITH chain, a production integration point, and an observable-effect wiring done-gate.
- **SUMMARY/I.CROSS.03** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L52| unit=CROSS): [E] Tracks 12-18 derive from the docs/prefableresearch corpus; Track 19 derives from two grok-elai-trust documents plus the token-efficiency plan; Track 20 derives from competitor code and STEAL-LIST.md.
- **SUMMARY/I.CROSS.04** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L55| unit=CROSS): [E] The generator proposes while the orchestrator owns state and receipt-ranked selection.
- **SUMMARY/I.CROSS.05** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L56| unit=CROSS): [E] Missing required evidence rows are failures, not silent skips, wherever R1 is named.
- **SUMMARY/I.CROSS.06** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L57| unit=CROSS): [E] Same-seed recomputation and byte-identical replay are recurring done-gates wherever GD-2 is named.
- **SUMMARY/I.CROSS.07** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L59| unit=CROSS): [E] Existing event/action types are reused where possible; several sessions explicitly prohibit introducing a new action type.
- **SUMMARY/I.CROSS.08** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L61| unit=CROSS SURFACE): [E] Paper numbers are not adopted as product thresholds. ELAI measures its own baselines and records losing or non-monotonic results honestly.
- **SUMMARY/I.CROSS.09** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L63| unit=CROSS): [E] User approval, anti-slopsquat checks, and separate network/RunPod checkboxes gate optional dependencies or live external evaluations.
- **SUMMARY/I.CROSS.10** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L65| unit=CROSS): [E] The I-1 supervisor is the enforcement boundary used later by I-35 sequence rules and the I-31 reward endpoint.
- **SUMMARY/I.CROSS.11** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L67| unit=CROSS): [E] I-2 isolated workspaces make I-4 parallel search safe.
- **SUMMARY/I.CROSS.12** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L68| unit=CROSS): [E] I-17 is blocking for I-4; I-4 in turn blocks I-22, I-28, and I-30.
- **SUMMARY/I.CROSS.13** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L69| unit=CROSS): [E] I-30 plus the H safety guards and I-24 are blocking prerequisites for I-31, which is explicitly last.
- **SUMMARY/I.CROSS.14** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L71| unit=CROSS SURFACE): [E] Track 14 inherits the local window-minimization versus cloud cache-stability split rather than reopening it.
- **SUMMARY/I.CROSS.15** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L73| unit=CROSS): [E] Track 15 exists to make I-4 selection receipts trustworthy: hard-veto supremacy, bounded mutation evidence, independent contracts, scorer canaries, and tamper-evident records.
- **SUMMARY/I.CROSS.16** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L75| unit=CROSS): [E] Track 17 may run only on a stronger-than-generator, execution-grounded filter; every learned weight is handled as a ledger-gated configuration artifact.
- **SUMMARY/I.CROSS.17** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L77| unit=CROSS): [E] Track 18 replaces prompt-shaped controls with deterministic mechanisms and measures each replacement.
- **SUMMARY/I.CROSS.18** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L79| unit=CROSS): [E] Track 19 LLM critique is advisory only and never determines accept/block; the off-agent F/G spine remains the sole DONE arbiter.
- **SUMMARY/I.CROSS.19** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L81| unit=CROSS): [E] Track 20 requires each executor to read the named competitor source file before execution. A summary-only port violates W-3.
- **SUMMARY/I.CROSS.20** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L83| unit=CROSS): [E] Track 20 corrections are binding: activation-not-build where substrate ships, wrap-never-delete for the measured VRAM tier plan, and idea-not-dependency for structural editing.
- **SUMMARY/I.CROSS.21** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L85| unit=CROSS): [E] The wholesale gajae provider-layer port is rejected; Tier-3 competitor items stay reference material and recorded declines must not be re-litigated.

#### SUMMARY / unit `Track 12` (49 records)

- **SUMMARY/I-1.01** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L96| unit=Track 12): [E][capability] Introduces pure Intent-to-Directive structs emitted by the generator.
- **SUMMARY/I-1.02** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L97| unit=Track 12): [E][security] Places Directive execution in a process-separated Supervisor.
- **SUMMARY/I-1.03** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L98| unit=Track 12): [E][invariant] Makes the Supervisor the only executor of side effects.
- **SUMMARY/I-1.04** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L99| unit=Track 12): [E][workflow] Runs every applicable gate before the Supervisor executes a Directive.
- **SUMMARY/I-1.05** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L100| unit=Track 12): [E][security] Makes the Supervisor, not the generator, hold credentials.
- **SUMMARY/I-1.06** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L101| unit=Track 12): [E][security] Prevents the generator from reaching gate code.
- **SUMMARY/I-1.07** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L102| unit=Track 12): [E][security] Prevents the generator from reaching the egress path or calling a sink directly.
- **SUMMARY/I-1.08** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L103| unit=Track 12): [E][lifecycle] Uses saga-style compensation after a crash to restore pre-tool state.
- **SUMMARY/I-1.09** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L104| unit=Track 12): [E][dependency] Reuses the existing Phase 4A IPC process seam rather than creating another.
- **SUMMARY/I-1.10** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L105| unit=Track 12): [E][integration] Moves F-1 VibeGate, E-30 argv, and G-3 blast-radius enforcement onto the supervisor plane.
- **SUMMARY/I-1.11** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L107| unit=Track 12): [E][integration] Integrates at task_runner dispatch and hook_dispatcher seams and proposes crates/orchestrator/src/directive.rs.
- **SUMMARY/I-1.12** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L109| unit=Track 12): [E][test] Adds a contract test that the generator cannot author or reach gate code.
- **SUMMARY/I-1.13** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L110| unit=Track 12 SURFACE): [E][UI] Shows supervisor verdict rows in an existing drawer.
- **SUMMARY/I-1.14** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L111| unit=Track 12): [E][failure] An injected OOM or panic during a tool call must restore exact pre-tool state.
- **SUMMARY/I-1.15** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L112| unit=Track 12): [E][observability] Successful compensation emits DirectiveExecuted(CompensationFired).
- **SUMMARY/I-1.16** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L113| unit=Track 12): [E][failure] A generator attempt to modify gate code or directly call a sink is refused.
- **SUMMARY/I-1.17** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L114| unit=Track 12): [E][observability] A refused Directive is logged in full.
- **SUMMARY/I-1.18** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L115| unit=Track 12): [E][fail-closed] A missing required supervisor row is a failure.
- **SUMMARY/I-1.19** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L116| unit=Track 12): [E][dependency] This boundary is the later execution home for I-35 temporal rules and I-31 reward serving.
- **SUMMARY/I-2.01** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L123| unit=Track 12): [E][capability] Gives each parallel branch or agent an isolated workspace.
- **SUMMARY/I-2.02** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L124| unit=Track 12): [E][platform] Uses overlayfs or btrfs reflinks on Linux.
- **SUMMARY/I-2.03** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L125| unit=Track 12): [E][platform] Uses pooled warm git worktrees with a shared target directory on Windows and macOS.
- **SUMMARY/I-2.04** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L127| unit=Track 12): [E][constraint] Does not assume cross-platform copy-on-write support.
- **SUMMARY/I-2.05** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L128| unit=Track 12): [E][workflow] Permits merges only at an explicit deterministic BSP barrier.
- **SUMMARY/I-2.06** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L129| unit=Track 12): [E][determinism] Orders merges by SSB sequence.
- **SUMMARY/I-2.07** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L130| unit=Track 12): [E][security] Blocks a branch that escapes its workspace root.
- **SUMMARY/I-2.08** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L131| unit=Track 12): [E][extension] Allows an optional cloud-sandbox adapter behind the same trait.
- **SUMMARY/I-2.09** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L132| unit=Track 12): [E][dependency] Makes the cloud sandbox optional rather than a required dependency.
- **SUMMARY/I-2.10** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L133| unit=Track 12): [E][integration] Adds a branch-workspace trait plus overlayfs and worktree-pool implementations under crates/orchestrator/src/sandbox/branch_workspace.rs.
- **SUMMARY/I-2.11** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L135| unit=Track 12): [E][integration] Connects merging at the BSP aggregator merge site.
- **SUMMARY/I-2.12** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L136| unit=Track 12 SURFACE): [E][UI] Displays branch-workspace badges on BranchTree.
- **SUMMARY/I-2.13** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L137| unit=Track 12): [E][observability] Journals per-branch root prefixes so disjoint paths are verifiable.
- **SUMMARY/I-2.14** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L138| unit=Track 12): [E][observability] Emits BranchWorkspaceEvent states Spawned and Merged.
- **SUMMARY/I-2.15** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L139| unit=Track 12): [E][failure] Emits EscapeBlocked for an absolute-path write outside the branch root.
- **SUMMARY/I-2.16** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L140| unit=Track 12): [E][test] A seeded adjacent-code conflict must resolve deterministically at the barrier.
- **SUMMARY/I-2.17** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L141| unit=Track 12): [E][test] A same-seed rerun must produce a byte-identical merge.
- **SUMMARY/I-2.18** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L142| unit=Track 12): [E][interaction] Supplies safe parallelism to the I-4 search ladder while remaining distinct from I-52 single-agent in-branch rewind.
- **SUMMARY/I-3.01** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L149| unit=Track 12): [E][performance] Verifies or extends busy_timeout, cache_size, and wal_autocheckpoint for search-era SSB write rates.
- **SUMMARY/I-3.02** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L151| unit=Track 12): [E][durability] Uses temporary-file, fsync, and rename atomic writes at every checkpoint site.
- **SUMMARY/I-3.03** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L152| unit=Track 12): [E][scope] Closes only checkpoint sites that are not already atomic.
- **SUMMARY/I-3.04** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L153| unit=Track 12): [E][measurement] Measures real SSB insert throughput rather than claiming a paper number.
- **SUMMARY/I-3.05** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L154| unit=Track 12): [E][data] Records throughput in metrics/phase_I3.toml.
- **SUMMARY/I-3.06** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L155| unit=Track 12): [E][platform] Exposes io_uring only as a Linux opt-in experiment.
- **SUMMARY/I-3.07** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L156| unit=Track 12): [E][constraint] Keeps io_uring allowlisted and never the default path.
- **SUMMARY/I-3.08** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L157| unit=Track 12): [E][integration] Touches the ssb.rs write-loop pragma block and checkpoint sites in the orchestrator and session DAG.
- **SUMMARY/I-3.09** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L159| unit=Track 12 SURFACE): [E][UI] Has no live UI; the metrics TOML is the justified observable surface.
- **SUMMARY/I-3.10** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L160| unit=Track 12): [E][failure] A kill -9 during a checkpoint must leave either the old or new complete file, never a torn file.
- **SUMMARY/I-3.11** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L162| unit=Track 12): [E][test] The no-atomic broken arm must produce a torn file and fail.
- **SUMMARY/I-3.12** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L163| unit=Track 12): [A][status] The summary requires re-verifying which pragmas and checkpoint sites already ship; it does not enumerate the remaining gaps.

#### SUMMARY / unit `Track 13` (88 records)

- **SUMMARY/I-4.01** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L173| unit=Track 13): [E][capability] Selects search candidates by off-agent receipts rather than an LLM judge.
- **SUMMARY/I-4.02** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L174| unit=Track 13): [E][workflow] Version 1 runs best-of-N search.
- **SUMMARY/I-4.03** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L175| unit=Track 13): [E][workflow] Version 1 fans N candidates into isolated workspaces.
- **SUMMARY/I-4.04** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L176| unit=Track 13): [E][workflow] A tri-layer veto, budgeted mutation evidence, and progress score produce the survivor ranking.
- **SUMMARY/I-4.05** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L178| unit=Track 13): [E][observability] The SSB records why a branch won.
- **SUMMARY/I-4.06** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L179| unit=Track 13): [E][workflow] Version 2 upgrades selection to a tournament.
- **SUMMARY/I-4.07** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L180| unit=Track 13): [E][workflow] Version 3 builds a PUCT tree over SessionDag.
- **SUMMARY/I-4.08** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L181| unit=Track 13): [E][concurrency] PUCT uses virtual loss for parallel expansion.
- **SUMMARY/I-4.09** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L182| unit=Track 13): [E][generation] The generator proposes K divergent candidates using C-26 temperature schedules.
- **SUMMARY/I-4.10** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L184| unit=Track 13): [E][invariant] The orchestrator, not the generator, owns the search tree.
- **SUMMARY/I-4.11** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L185| unit=Track 13): [E][dependency] Reuses SessionDag/H-49 as the branch store.
- **SUMMARY/I-4.12** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L186| unit=Track 13): [E][integration] Adds crates/orchestrator/src/search/ladder.rs first and mcts.rs only at v3.
- **SUMMARY/I-4.13** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L187| unit=Track 13): [E][rollout] Replaces linear retry at baby-step dispatch only after the ladder demonstrates a measured win.
- **SUMMARY/I-4.14** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L189| unit=Track 13 SURFACE): [E][UI] Feeds SearchNodeEvent rows to the H15-1 branch-board comparison view.
- **SUMMARY/I-4.15** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L190| unit=Track 13): [E][failure] Compile failure creates a HardVeto and Pruned row.
- **SUMMARY/I-4.16** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L191| unit=Track 13): [E][observability] Promoted survivor rows include concrete receipts such as mutant-kill count and progress score.
- **SUMMARY/I-4.17** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L193| unit=Track 13): [E][workflow] The winning branch merges after ranking.
- **SUMMARY/I-4.18** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L194| unit=Track 13): [E][test] The LLM-judge-only broken arm must be able to pick a vetoed branch and fail.
- **SUMMARY/I-4.19** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L195| unit=Track 13): [E][rollout] Version 1 must ship before any tree-search code.
- **SUMMARY/I-4.20** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L196| unit=Track 13): [A][configuration] N and K are configurable or unspecified in the summary; the observable fixture uses three candidates but does not declare three as the product constant.
- **SUMMARY/I-5.01** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L203| unit=Track 13): [E][context] Rebuilds branch context from the parent chain at expansion time.
- **SUMMARY/I-5.02** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L204| unit=Track 13): [E][context] Hard-excludes failed siblings.
- **SUMMARY/I-5.03** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L205| unit=Track 13): [E][context] Hard-excludes raw tracebacks.
- **SUMMARY/I-5.04** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L206| unit=Track 13): [E][context] Preserves failure information only as a compressed contrastive constraint.
- **SUMMARY/I-5.05** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L207| unit=Track 13): [E][measurement] Samples pristine-versus-accumulated context A/B decisions on the same choice.
- **SUMMARY/I-5.06** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L208| unit=Track 13): [E][measurement] Uses the E-19 reasoning-quality probe and records the measured vividness-tax delta.
- **SUMMARY/I-5.07** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L210| unit=Track 13): [E][integration] Extends H-6 masking/H-50 regime machinery without remasking.
- **SUMMARY/I-5.08** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L211| unit=Track 13): [E][integration] Hooks the context budget and prompt-builder assembly seam through a pristine_path_filter.
- **SUMMARY/I-5.09** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L213| unit=Track 13 SURFACE): [E][UI] Adds a vividness-tax tile to ReasoningDrawer.
- **SUMMARY/I-5.10** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L214| unit=Track 13): [E][test] A retry payload must contain zero bytes from a prior 5K-line traceback.
- **SUMMARY/I-5.11** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L215| unit=Track 13): [E][constraint] The retained contrastive constraint is at most 30 tokens in the stated observable fixture.
- **SUMMARY/I-5.12** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L217| unit=Track 13): [E][observability] Emits ContextPruned rows.
- **SUMMARY/I-5.13** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L218| unit=Track 13): [E][determinism] The A/B delta must be recomputable under ELAI_SEED.
- **SUMMARY/I-5.14** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L219| unit=Track 13): [E][test] A carry-tracebacks broken arm must reinflate context and fail.
- **SUMMARY/I-6.01** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L225| unit=Track 13): [E][context] Adds an immutable PrivilegedAnchor context slot.
- **SUMMARY/I-6.02** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L226| unit=Track 13): [E][layout] Appends the anchor last, after the final cache breakpoint.
- **SUMMARY/I-6.03** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L227| unit=Track 13): [E][data] Carries the TaskSpec in the anchor.
- **SUMMARY/I-6.04** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L228| unit=Track 13): [E][data] Carries unresolved claims in the anchor.
- **SUMMARY/I-6.05** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L229| unit=Track 13): [E][data] Carries active constraints in the anchor.
- **SUMMARY/I-6.06** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L230| unit=Track 13): [E][generation] Requires the model to emit a short acknowledgement tag reflecting the active constraint before generation.
- **SUMMARY/I-6.07** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L232| unit=Track 13): [E][integration] Extends ContextSlot with AnchorPane without rebalancing other slots.
- **SUMMARY/I-6.08** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L233| unit=Track 13): [E][observability] Reuses ContextReduction rows and introduces no new action type.
- **SUMMARY/I-6.09** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L234| unit=Track 13 SURFACE): [E][UI] Has no separate UI because it is prompt-internal.
- **SUMMARY/I-6.10** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L235| unit=Track 13): [E][test] On a 25K-plus-token task the final journaled prompt block must be the anchor.
- **SUMMARY/I-6.11** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L236| unit=Track 13): [E][measurement] Measures constraint-adherence improvement against a no-anchor arm rather than asserting it.
- **SUMMARY/I-6.12** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L238| unit=Track 13): [E][invariant] The cache-prefix stability assertion must continue to pass.
- **SUMMARY/I-7.01** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L245| unit=Track 13): [E][workflow] Runs wide cheap branches on the local lane first.
- **SUMMARY/I-7.02** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L246| unit=Track 13): [E][workflow] Applies deterministic checks locally.
- **SUMMARY/I-7.03** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L247| unit=Track 13): [E][routing] Escalates only after local failure.
- **SUMMARY/I-7.04** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L248| unit=Track 13): [E][context] Condenses the local failure set into I-11 BeliefState before escalation.
- **SUMMARY/I-7.05** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L249| unit=Track 13): [E][routing] Escalates synchronously to the cloud architect.
- **SUMMARY/I-7.06** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L250| unit=Track 13): [E][constraint] Keeps the Batch API in H-5 offline lanes.
- **SUMMARY/I-7.07** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L251| unit=Track 13): [E][constraint] Makes interactive-path batch routing structurally impossible.
- **SUMMARY/I-7.08** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L252| unit=Track 13): [E][integration] Hooks cloud_mode/escalation_policy and emits EscalationCascade rows.
- **SUMMARY/I-7.09** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L253| unit=Track 13 SURFACE): [E][UI] Shows the cascade trace in the cost cockpit.
- **SUMMARY/I-7.10** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L254| unit=Track 13): [E][privacy/cost] A locally solvable task produces LocalWin and zero journaled cloud spend.
- **SUMMARY/I-7.11** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L255| unit=Track 13): [E][workflow] A local-failure fixture escalates exactly once.
- **SUMMARY/I-7.12** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L256| unit=Track 13): [E][constraint] The observable escalated payload is at most 2K tokens.
- **SUMMARY/I-7.13** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L257| unit=Track 13): [E][observability] Records Condensed then Escalated rows and derives payload size from the journal.
- **SUMMARY/I-7.14** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L259| unit=Track 13): [E][correction] Explicitly rejects speculative racing against the Batch API as impossible.
- **SUMMARY/I-8.01** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L265| unit=Track 13): [E][tool] Adds an acknowledge_infeasible/abort_trajectory tool to the tool schema.
- **SUMMARY/I-8.02** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L266| unit=Track 13): [E][reward] Credits voluntary abort through the existing abstention-aware H-39/RD-1 score.
- **SUMMARY/I-8.03** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L267| unit=Track 13): [E][lifecycle] Rolls an aborted trajectory back to the last good H-49 trunk node.
- **SUMMARY/I-8.04** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L268| unit=Track 13): [E][cache] Hashes the aborted AST or approach shape.
- **SUMMARY/I-8.05** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L269| unit=Track 13): [E][cache] Stores aborted-path hashes in a dead-end cache.
- **SUMMARY/I-8.06** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L270| unit=Track 13): [E][routing] Prunes an isomorphic future branch before an LLM call.
- **SUMMARY/I-8.07** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L271| unit=Track 13): [E][safety] Aborts and penalizes deterministic checks that spin longer than 15 seconds.
- **SUMMARY/I-8.08** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L272| unit=Track 13): [E][integration] Adds the tool through the existing three-arm catalog pattern.
- **SUMMARY/I-8.09** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L273| unit=Track 13): [E][integration] Hooks the search/retry loop.
- **SUMMARY/I-8.10** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L274| unit=Track 13): [E][data] Keys the dead-end cache by blake3 of AST shape.
- **SUMMARY/I-8.11** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L275| unit=Track 13 SURFACE): [E][UI] Displays an abort/dead-end strip in ReasoningDrawer.
- **SUMMARY/I-8.12** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L276| unit=Track 13): [E][observability] Impossible tasks emit TrajectoryAborted(Voluntary) and a journaled rollback.
- **SUMMARY/I-8.13** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L277| unit=Track 13): [E][observability] Isomorphic reattempts emit DeadEndHash and must incur zero model calls.
- **SUMMARY/I-8.14** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L278| unit=Track 13): [E][observability] A spin fixture emits HaltingBound within budget.
- **SUMMARY/I-9.01** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L284| unit=Track 13): [E][measurement] Defines alpha_t as whether a corrupted step amplifies or contracts downstream error.
- **SUMMARY/I-9.02** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L286| unit=Track 13): [E][test] Corrupts step t in a frozen fixture pipeline.
- **SUMMARY/I-9.03** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L287| unit=Track 13): [E][dependency] Reuses C-28 mutation operators as the corruption source.
- **SUMMARY/I-9.04** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L288| unit=Track 13): [E][measurement] Measures downstream recovery after corruption.
- **SUMMARY/I-9.05** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L289| unit=Track 13): [E][control] Actively anneals when live alpha exceeds 1.
- **SUMMARY/I-9.06** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L290| unit=Track 13): [E][control] Annealing lowers generation temperature.
- **SUMMARY/I-9.07** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L291| unit=Track 13): [E][control] Annealing tightens the grammar.
- **SUMMARY/I-9.08** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L292| unit=Track 13): [E][measurement] Measures ELAI's own alpha baseline before claiming contraction.
- **SUMMARY/I-9.09** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L293| unit=Track 13 SURFACE): [E][CLI] Adds elai bench error-amplification.
- **SUMMARY/I-9.10** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L294| unit=Track 13): [E][integration] Adds an error_amplification_runner using embedded fixtures.
- **SUMMARY/I-9.11** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L295| unit=Track 13 SURFACE): [E][UI] Adds an alpha-trend cockpit tile.
- **SUMMARY/I-9.12** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L296| unit=Track 13): [E][observability] Emits ErrorAmplificationMeasured with a seed-recomputable alpha.
- **SUMMARY/I-9.13** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L297| unit=Track 13): [E][test] A no-repair arm must yield alpha greater than 1 while the repaired arm contracts.
- **SUMMARY/I-9.14** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L298| unit=Track 13): [E][observability] A live alpha-greater-than-1 fixture must journal the resulting temperature drop.

#### SUMMARY / unit `Track 14` (126 records)

- **SUMMARY/I-10.01** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L307| unit=Track 14): [E][cache] Enforces a byte-frozen prompt prefix end to end.
- **SUMMARY/I-10.02** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L308| unit=Track 14): [E][constraint] Excludes timestamps from the frozen prefix.
- **SUMMARY/I-10.03** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L309| unit=Track 14): [E][constraint] Excludes UUIDs from the frozen prefix.
- **SUMMARY/I-10.04** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L310| unit=Track 14): [E][constraint] Excludes mutable endpoints from the frozen prefix.
- **SUMMARY/I-10.05** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L311| unit=Track 14): [E][tools] Restricts tools dynamically through tool_choice or allowed_tools payload attributes.
- **SUMMARY/I-10.06** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L313| unit=Track 14): [E][constraint] Never mutates prompt text to express dynamic tool restriction.
- **SUMMARY/I-10.07** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L314| unit=Track 14): [E][measurement] Chooses inline-frozen versus H-4-deferred tool arrays by measured cache-hit and token cost for each catalog size.
- **SUMMARY/I-10.08** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L316| unit=Track 14): [E][cache] Uses a one-hour TTL at the frozen-prefix breakpoint.
- **SUMMARY/I-10.09** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L317| unit=Track 14): [E][cache] Uses a five-minute TTL for volatile tails.
- **SUMMARY/I-10.10** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L318| unit=Track 14): [E][planning] A cache-economics planner selects tiers using inter-call gap statistics from the journal.
- **SUMMARY/I-10.11** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L320| unit=Track 14): [E][guard] Halts when unexplained misses exceed 10 percent for identical task classes.
- **SUMMARY/I-10.12** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L321| unit=Track 14): [E][diagnostics] Dumps a blake3 prefix diff on guard trip.
- **SUMMARY/I-10.13** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L322| unit=Track 14): [E][diagnostics] Names the leaking mutation and byte range.
- **SUMMARY/I-10.14** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L323| unit=Track 14): [E][scheduling] Uses burst-amortized scheduling for long local validation loops.
- **SUMMARY/I-10.15** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L324| unit=Track 14): [E][fallback] Demotes heartbeat traffic to a last resort.
- **SUMMARY/I-10.16** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L325| unit=Track 14): [E][integration] Extends shipped H-1/H-2/H-9 machinery only with the TTL, planner, and guard layer if those pieces already ship.
- **SUMMARY/I-10.17** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L327| unit=Track 14 SURFACE): [E][UI] Adds TTL and economics fields to the H-9 cost tile.
- **SUMMARY/I-10.18** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L328| unit=Track 14): [E][observability] Emits CacheEconomicsEvent TtlPlanned for real cloud calls.
- **SUMMARY/I-10.19** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L329| unit=Track 14): [E][test] A deliberately injected prefix timestamp trips the guard within one session.
- **SUMMARY/I-10.20** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L330| unit=Track 14): [E][observability] Emits HitRateGuardTripped and PrefixDiffDumped.
- **SUMMARY/I-10.21** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L331| unit=Track 14): [A][decision] Inline versus deferred tool-array layout is deliberately unresolved until measured per catalog size.
- **SUMMARY/I-11.01** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L338| unit=Track 14): [E][data] Defines a typed, versioned BeliefState.
- **SUMMARY/I-11.02** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L339| unit=Track 14): [E][data] Records what was tried.
- **SUMMARY/I-11.03** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L340| unit=Track 14): [E][data] Records why attempts failed.
- **SUMMARY/I-11.04** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L341| unit=Track 14): [E][data] Records current AST state.
- **SUMMARY/I-11.05** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L342| unit=Track 14): [E][data] Records active constraints.
- **SUMMARY/I-11.06** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L343| unit=Track 14): [E][knowledge] Links entities to the DKG.
- **SUMMARY/I-11.07** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L344| unit=Track 14): [E][workflow] Replaces raw history on local-to-cloud escalation.
- **SUMMARY/I-11.08** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L345| unit=Track 14): [E][workflow] Replaces raw history on architect-to-editor handoff.
- **SUMMARY/I-11.09** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L346| unit=Track 14): [E][workflow] Replaces raw history on session resume.
- **SUMMARY/I-11.10** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L347| unit=Track 14): [E][observability] Journals every handoff as an ID-3 receipt.
- **SUMMARY/I-11.11** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L348| unit=Track 14): [E][measurement] Periodically reruns tasks from full history versus BeliefState.
- **SUMMARY/I-11.12** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L349| unit=Track 14): [E][measurement] Records the measured success delta as compression cost.
- **SUMMARY/I-11.13** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L350| unit=Track 14): [E][integration] Shares the digest engine with H-51 while keeping cross-session scope distinct.
- **SUMMARY/I-11.14** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L352| unit=Track 14): [E][recovery] Uses H-6 recover_stubs to re-expand detail.
- **SUMMARY/I-11.15** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L353| unit=Track 14 SURFACE): [E][UI] Adds a BeliefState inspector row to SessionsDrawer.
- **SUMMARY/I-11.16** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L354| unit=Track 14): [E][constraint] Local-to-cloud escalation carries only prefix plus BeliefState.
- **SUMMARY/I-11.17** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L355| unit=Track 14): [E][measurement] The summary's observable compares an approximately 2K-token handoff with a 20K-or-more-token history control.
- **SUMMARY/I-11.18** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L357| unit=Track 14): [E][observability] Emits BeliefStateEvent(Condensed/Handoff) with schema version.
- **SUMMARY/I-11.19** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L358| unit=Track 14): [E][observability] Emits seed-recomputable HonestyDelta results.
- **SUMMARY/I-11.20** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L359| unit=Track 14): [E][fail-closed] A schema-version mismatch yields a typed closed failure, never a silent parse.
- **SUMMARY/I-12.01** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L366| unit=Track 14): [E][learning] Curates playbook/rule entries in an ACE-shaped learning loop.
- **SUMMARY/I-12.02** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L367| unit=Track 14): [E][promotion] Promotes an entry only when it links to at least k SSB-verified successful outcomes.
- **SUMMARY/I-12.03** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L369| unit=Track 14): [E][measurement] Stores each promoted entry's measured lift.
- **SUMMARY/I-12.04** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L370| unit=Track 14): [E][measurement] Measures lift through cheap interleaved on/off injection.
- **SUMMARY/I-12.05** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L371| unit=Track 14): [E][lifecycle] Demotes entries using the same evidence ledger.
- **SUMMARY/I-12.06** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L372| unit=Track 14): [E][guard] Refuses merged rules that regress the C-28 mutation suite.
- **SUMMARY/I-12.07** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L373| unit=Track 14): [E][interop] Exports verified content to CLAUDE.md.
- **SUMMARY/I-12.08** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L374| unit=Track 14): [E][interop] Exports verified content to AGENTS.md.
- **SUMMARY/I-12.09** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L375| unit=Track 14): [E][interop] Exports verified content to skill formats.
- **SUMMARY/I-12.10** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L376| unit=Track 14): [E][dependency] Feeds the existing Phase 3F proposed-to-shadow-to-active rule lifecycle.
- **SUMMARY/I-12.11** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L377| unit=Track 14): [E][operations] Runs the curator from an offline Layer-1A tick.
- **SUMMARY/I-12.12** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L378| unit=Track 14 SURFACE): [E][UI] Displays playbook rows in RulesDrawer.
- **SUMMARY/I-12.13** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L379| unit=Track 14): [E][observability] PlaybookEvent(Promoted) carries the supporting outcome row IDs.
- **SUMMARY/I-12.14** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L380| unit=Track 14): [E][failure] A mutation-regressing merge is refused by a failed ValidationRun.
- **SUMMARY/I-12.15** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L381| unit=Track 14): [E][lifecycle] Stale entries demote.
- **SUMMARY/I-12.16** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L382| unit=Track 14 SURFACE): [E][CLI] elai playbook export writes an AGENTS.md block.
- **SUMMARY/I-12.17** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L383| unit=Track 14): [E][traceability] Every exported entry cites its evidence rows.
- **SUMMARY/I-12.18** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L384| unit=Track 14): [E][fail-closed] Zero-evidence promotion fails.
- **SUMMARY/I-12.19** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L385| unit=Track 14): [A][configuration] The minimum success count k is named but not assigned a value in the summary.
- **SUMMARY/I-13.01** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L392| unit=Track 14): [E][security] Scans every Recall memory write at write time.
- **SUMMARY/I-13.02** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L393| unit=Track 14): [E][security] Scans every Archival memory write at write time.
- **SUMMARY/I-13.03** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L394| unit=Track 14): [E][security] Scans every playbook memory write at write time.
- **SUMMARY/I-13.04** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L395| unit=Track 14): [E][security] Detects injection patterns deterministically.
- **SUMMARY/I-13.05** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L396| unit=Track 14): [E][security] Detects invisible-Unicode and multibyte tricks.
- **SUMMARY/I-13.06** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L397| unit=Track 14): [E][security] Detects secrets.
- **SUMMARY/I-13.07** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L398| unit=Track 14): [E][implementation] Uses regex and Aho-Corasick first.
- **SUMMARY/I-13.08** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L399| unit=Track 14): [E][dependency] Allows a YARA engine only as a user-approved dependency.
- **SUMMARY/I-13.09** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L400| unit=Track 14 SURFACE): [E][serialization] Defines to_ui_dict as the full dashboard representation.
- **SUMMARY/I-13.10** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L401| unit=Track 14): [E][serialization] Defines to_llm_content as the condensed context representation.
- **SUMMARY/I-13.11** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L402| unit=Track 14): [E][tool] Adds TraceQuery for on-demand expansion of exact disk line ranges.
- **SUMMARY/I-13.12** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L403| unit=Track 14): [E][integration] Hooks memory write choke points and crates/tools tool_result.rs.
- **SUMMARY/I-13.13** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L404| unit=Track 14 SURFACE): [E][UI] Adds guard badges to MemoryFactsDrawer.
- **SUMMARY/I-13.14** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L405| unit=Track 14): [E][failure] Blocks a seeded invisible-Unicode poison attempt.
- **SUMMARY/I-13.15** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L406| unit=Track 14): [E][observability] Emits MemoryGuardEvent(PoisonBlocked) with the pattern class.
- **SUMMARY/I-13.16** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L407| unit=Track 14 SURFACE): [E][behavior] A 500-file listing remains complete in the dashboard but enters model context in condensed form.
- **SUMMARY/I-13.17** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L409| unit=Track 14): [E][observability] Journals the serialization token delta and DualSerialized.
- **SUMMARY/I-13.18** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L410| unit=Track 14): [E][observability] TraceQuery emits TraceQueryExpanded for the exact requested range.
- **SUMMARY/I-13.19** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L411| unit=Track 14): [E][rollout] Requires false-positive bounds on at least three real runs before default-on.
- **SUMMARY/I-14.01** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L417| unit=Track 14): [E][compression] Makes Tier 1 compression deterministic.
- **SUMMARY/I-14.02** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L418| unit=Track 14): [E][compression] Uses headless LSP slicing to extract the exact AST node, declaration, or failing line referenced by a compiler error.
- **SUMMARY/I-14.03** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L420| unit=Track 14): [E][compression] Uses tree-sitter AST folding.
- **SUMMARY/I-14.04** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L421| unit=Track 14): [E][compression] Replaces irrelevant function bodies with an elision marker.
- **SUMMARY/I-14.05** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L422| unit=Track 14): [E][invariant] Preserves signatures and types to retain BPE priors.
- **SUMMARY/I-14.06** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L423| unit=Track 14): [E][compression] Allows Tier 2 token-classification compression only for prose payloads.
- **SUMMARY/I-14.07** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L424| unit=Track 14): [E][resource] Checks VRAM budget before Tier 2 use.
- **SUMMARY/I-14.08** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L425| unit=Track 14): [E][constraint] Bans cipher or abbreviation substitution.
- **SUMMARY/I-14.09** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L426| unit=Track 14): [E][replay] Emits a reversible ContextTransformReceipt for every transform.
- **SUMMARY/I-14.10** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L427| unit=Track 14): [E][replay] Enables replay to reconstruct the exact bytes the model saw.
- **SUMMARY/I-14.11** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L428| unit=Track 14): [E][dependency] Shares the E-52 headless rust-analyzer client and existing tree-sitter grammars.
- **SUMMARY/I-14.12** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L430| unit=Track 14): [E][integration] Hooks the tool-result-to-context seam through context_transform.rs.
- **SUMMARY/I-14.13** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L431| unit=Track 14 SURFACE): [E][UI] Displays transform receipts in the Transcript drawer.
- **SUMMARY/I-14.14** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L432| unit=Track 14): [E][measurement] Compresses the stated 10K-token panic fixture to at most 500 tokens.
- **SUMMARY/I-14.15** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L433| unit=Track 14): [E][recovery] A receipt's reverse pointer expands to bytes matching the original blake3.
- **SUMMARY/I-14.16** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L434| unit=Track 14): [E][measurement] Requires fixture pass rate to remain unchanged versus uncompressed input.
- **SUMMARY/I-14.17** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L435| unit=Track 14): [E][fail-closed] Refuses Tier 2 use without VRAM headroom.
- **SUMMARY/I-14.18** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L436| unit=Track 14): [E][fail-closed] Running transforms without receipts is a failure.
- **SUMMARY/I-15.01** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L442| unit=Track 14): [E][cache] Caches intermediate DKG chunk summaries.
- **SUMMARY/I-15.02** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L443| unit=Track 14): [E][cache] Caches intermediate research extracts.
- **SUMMARY/I-15.03** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L444| unit=Track 14): [E][cache] Serves intermediate summaries for semantically similar queries.
- **SUMMARY/I-15.04** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L445| unit=Track 14): [E][performance] Runs an exact blake3 lookup before any vector math.
- **SUMMARY/I-15.05** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L446| unit=Track 14): [E][performance] Makes exact-repeat lookup O(1).
- **SUMMARY/I-15.06** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L447| unit=Track 14): [E][security/scope] Restricts durable caching to idempotent grounding content.
- **SUMMARY/I-15.07** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L448| unit=Track 14): [E][integration] Extends C-30/H-8 OracleAnswerCache and hooks A-RAG summarization.
- **SUMMARY/I-15.08** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L449| unit=Track 14): [E][observability] Reuses OracleCacheHit and OracleCacheMiss rows.
- **SUMMARY/I-15.09** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L450| unit=Track 14 SURFACE): [E][UI] Shows a cache-source badge on research answers.
- **SUMMARY/I-15.10** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L451| unit=Track 14): [E][measurement] Measures run-two RAG input-token reduction instead of claiming the paper's percentage.
- **SUMMARY/I-15.11** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L453| unit=Track 14): [E][test] Exact repeats must issue zero ONNX calls.
- **SUMMARY/I-15.12** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L454| unit=Track 14): [E][observability] Measures exact-path latency and call count from traces.
- **SUMMARY/I-15.13** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L455| unit=Track 14): [E][guard] Code-generation queries can never hit the durable cache.
- **SUMMARY/I-16.01** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L462| unit=Track 14): [E][evaluation] Evaluates sink-protected KV pruning rather than assuming adoption.
- **SUMMARY/I-16.02** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L463| unit=Track 14): [E][cache] Pins the first four tokens during pruning.
- **SUMMARY/I-16.03** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L464| unit=Track 14): [E][evaluation] Evaluates Int4 KV quantization.
- **SUMMARY/I-16.04** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L465| unit=Track 14): [E][measurement] Measures fixture pass rate per configuration.
- **SUMMARY/I-16.05** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L466| unit=Track 14): [E][measurement] Measures VRAM per configuration.
- **SUMMARY/I-16.06** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L467| unit=Track 14): [E][measurement] Measures throughput per configuration.
- **SUMMARY/I-16.07** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L468| unit=Track 14): [E][baseline] Compares with the E-27 AWQ baseline.
- **SUMMARY/I-16.08** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L469| unit=Track 14): [E][evaluation] Keeps the MoE sidecar retarget as a separate checkbox.
- **SUMMARY/I-16.09** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L470| unit=Track 14): [E][constraint] Does not assume a model swap because prior swaps lost on measurement.
- **SUMMARY/I-16.10** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L471| unit=Track 14): [E][invariant] Preserves the FA2 mandate and LogitsProcessor chain.
- **SUMMARY/I-16.11** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L472| unit=Track 14): [E][integration] Adds sidecar flags and an E-27-shaped evaluation harness.
- **SUMMARY/I-16.12** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L473| unit=Track 14 SURFACE): [E][CLI] elai bench kv-tune writes KvCacheTuned with vram_gb, tok_s, and fixture_pass_rate.
- **SUMMARY/I-16.13** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L474| unit=Track 14): [E][promotion] Adopts a configuration only if pass rate remains within band and VRAM falls.
- **SUMMARY/I-16.14** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L475| unit=Track 14): [E][observability] Records the adopt/reject decision as a row.
- **SUMMARY/I-16.15** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L476| unit=Track 14): [E][operations] Ships a no-GPU deterministic fixture harness immediately while the real RunPod evaluation remains separately gated.
- **SUMMARY/I-16.16** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L478| unit=Track 14): [A][threshold] The acceptable pass-rate band is not specified in the summary.

#### SUMMARY / unit `Track 15` (148 records)

- **SUMMARY/I-17.01** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L487| unit=Track 15): [E][evaluation] Defines Layer 1 as a supreme hard veto.
- **SUMMARY/I-17.02** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L488| unit=Track 15): [E][evaluation] Layer 1 includes compile results.
- **SUMMARY/I-17.03** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L489| unit=Track 15): [E][evaluation] Layer 1 includes test results.
- **SUMMARY/I-17.04** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L490| unit=Track 15): [E][evaluation] Layer 1 includes F-1 VibeGate.
- **SUMMARY/I-17.05** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L491| unit=Track 15): [E][evaluation] Layer 1 includes F-9 dependency gate.
- **SUMMARY/I-17.06** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L492| unit=Track 15): [E][evaluation] Layer 1 includes D-11 constructive checkers.
- **SUMMARY/I-17.07** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L493| unit=Track 15): [E][invariant] Any Layer-1 failure forces score 0.0.
- **SUMMARY/I-17.08** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L494| unit=Track 15): [E][invariant] No later layer can override a hard veto.
- **SUMMARY/I-17.09** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L495| unit=Track 15): [E][evaluation] Layer 2 scores survivors only.
- **SUMMARY/I-17.10** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L496| unit=Track 15): [E][evaluation] Layer 2 uses the D-2 PPM seam when live and PRM-lite/E-19 until then.
- **SUMMARY/I-17.11** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L497| unit=Track 15): [E][evaluation] Layer 3 escalates disagreement to a cross-family D-13 adversary.
- **SUMMARY/I-17.12** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L498| unit=Track 15): [E][safety] Applies a 15-second empirical halting bound to deterministic Layer-1 checks.
- **SUMMARY/I-17.13** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L499| unit=Track 15): [E][workflow] Aborts the branch when a deterministic check exceeds that bound.
- **SUMMARY/I-17.14** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L500| unit=Track 15): [E][reliability] Reruns candidate oracles under seed variation to detect flakes.
- **SUMMARY/I-17.15** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L501| unit=Track 15): [E][quarantine] An oracle whose result flips is quarantined.
- **SUMMARY/I-17.16** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L502| unit=Track 15): [E][invariant] A quarantined oracle can neither veto nor vouch until rehabilitated.
- **SUMMARY/I-17.17** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L503| unit=Track 15): [E][observability] The flake decision and excluded vote are ledgered.
- **SUMMARY/I-17.18** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L504| unit=Track 15): [E][extension] Defines a Tier-2 cross-language verifier-pack seam for TypeScript and Python behind the D-3 adapter trait.
- **SUMMARY/I-17.19** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L506| unit=Track 15): [E][integration] Adds verifier_stack.rs for use by the I-4 ranker and baby-step audit seam.
- **SUMMARY/I-17.20** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L507| unit=Track 15 SURFACE): [E][UI] Shows per-branch verdict chips.
- **SUMMARY/I-17.21** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L508| unit=Track 15): [E][observability] A hallucinated import emits TriLayerVerdict HardVeto before any Layer-2 call.
- **SUMMARY/I-17.22** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L510| unit=Track 15): [E][test] A hard-vetoed candidate must show Layer-2 call count zero.
- **SUMMARY/I-17.23** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L511| unit=Track 15): [E][observability] Survivor ordering emits ScoredSurvivor.
- **SUMMARY/I-17.24** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L512| unit=Track 15): [E][observability] A seeded flaky test emits FlakeQuarantined.
- **SUMMARY/I-17.25** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L513| unit=Track 15): [E][test] The Layer-2-overrides-Layer-1 broken arm must resurrect a vetoed branch and fail.
- **SUMMARY/I-17.26** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L514| unit=Track 15): [A][deferred] TypeScript/Python verifier packs are described only as Tier 2, with no delivery threshold in the summary.
- **SUMMARY/I-18.01** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L521| unit=Track 15): [E][integrity] Adds chain_prev_hash to each SSB row.
- **SUMMARY/I-18.02** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L522| unit=Track 15): [E][integrity] Uses the SSB single writer to make row order tamper-evident.
- **SUMMARY/I-18.03** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L523| unit=Track 15): [E][integrity] Computes a hierarchical STATE stack hash.
- **SUMMARY/I-18.04** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L524| unit=Track 15): [E][integrity] Computes a hierarchical DEPS stack hash.
- **SUMMARY/I-18.05** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L525| unit=Track 15): [E][integrity] Computes a hierarchical PROCESS stack hash.
- **SUMMARY/I-18.06** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L526| unit=Track 15): [E][integrity] Computes a hierarchical RESULT stack hash.
- **SUMMARY/I-18.07** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L527| unit=Track 15): [E][integrity] Computes a hierarchical VERIFY stack hash.
- **SUMMARY/I-18.08** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L528| unit=Track 15): [E][artifact] Seals the five-layer hash into the RRA.
- **SUMMARY/I-18.09** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L529| unit=Track 15): [E][attestation] Offers an opt-in Rekor transparency-log push.
- **SUMMARY/I-18.10** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L530| unit=Track 15): [E][dependency] Uses cosign keyless only as an opt-in dependency after anti-slopsquat and approval.
- **SUMMARY/I-18.11** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L532| unit=Track 15): [E][artifact] Adds an AI-BOM/CycloneDX-ML entry beside the attestation.
- **SUMMARY/I-18.12** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L533| unit=Track 15): [E][interop] Makes the artifact express the cited OWASP AISVS AC.8 expectation.
- **SUMMARY/I-18.13** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L534| unit=Track 15): [E][offline] Keeps local blake3 Merkle verification as the offline default.
- **SUMMARY/I-18.14** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L535| unit=Track 15 SURFACE): [E][CLI] Extends elai verify-attestation with chain and Rekor options.
- **SUMMARY/I-18.15** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L536| unit=Track 15 SURFACE): [E][UI] Adds a chain badge to the RRA viewer.
- **SUMMARY/I-18.16** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L537| unit=Track 15): [E][failure] Editing any mid-run row breaks chain verification at that exact row.
- **SUMMARY/I-18.17** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L538| unit=Track 15): [E][diagnostics] RowChainSealed(ChainBroken) names the failing sequence number.
- **SUMMARY/I-18.18** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L539| unit=Track 15): [E][test] The AI-BOM must validate against the CycloneDX schema.
- **SUMMARY/I-18.19** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L540| unit=Track 15): [E][offline] A Rekor inclusion proof must verify without rerunning ELAI.
- **SUMMARY/I-18.20** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L541| unit=Track 15): [E][fail-closed] Cargo.lock mutation during a run invalidates the DEPS layer and hard-halts.
- **SUMMARY/I-18.21** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L542| unit=Track 15): [E][coordination] The chain field shares a sealed-record layout with I-45 objective hash and therefore requires coordinated schema changes.
- **SUMMARY/I-19.01** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L549| unit=Track 15): [E][gate] Adds only the missing symbol-level PreToolUse leg rather than rebuilding the existing TESTING_FIRST FSM or anti-over-mock machinery.
- **SUMMARY/I-19.02** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L551| unit=Track 15): [E][scope] Applies to source-file edits during EXECUTING.
- **SUMMARY/I-19.03** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L552| unit=Track 15): [E][gate] Requires a corresponding test-symbol delta before the write lands.
- **SUMMARY/I-19.04** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L553| unit=Track 15): [E][analysis] Uses tree-sitter AST to verify a test references the modified function.
- **SUMMARY/I-19.05** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L554| unit=Track 15): [E][gate] Requires a syntax-only dry run before the write lands.
- **SUMMARY/I-19.06** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L555| unit=Track 15): [E][language] Uses cargo check --tests for Rust.
- **SUMMARY/I-19.07** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L556| unit=Track 15): [E][language] Uses pytest --collect-only for Python.
- **SUMMARY/I-19.08** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L557| unit=Track 15): [E][exemption] Exempts pure structural refactors.
- **SUMMARY/I-19.09** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L558| unit=Track 15): [E][analysis] Determines structural-refactor exemption through coverage delta.
- **SUMMARY/I-19.10** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L559| unit=Track 15): [E][rollout] Bounds false positives on at least three real runs before default-on.
- **SUMMARY/I-19.11** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L560| unit=Track 15): [E][integration] Adds the axis to F-1 VibeGate's compound PreToolUse evaluator rather than forking the hook.
- **SUMMARY/I-19.12** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L562| unit=Track 15 SURFACE): [E][UI] Uses the existing gate panel for block reasons.
- **SUMMARY/I-19.13** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L563| unit=Track 15): [E][failure] Editing in a new public function without a test-symbol delta is refused before disk.
- **SUMMARY/I-19.14** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L565| unit=Track 15): [E][observability] Emits TddPreflightBlocked MissingTestDelta.
- **SUMMARY/I-19.15** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L566| unit=Track 15): [E][behavior] The same edit with a matching failing test is allowed to land.
- **SUMMARY/I-19.16** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L567| unit=Track 15): [E][observability] Rename-only refactors emit RefactorExempted.
- **SUMMARY/I-19.17** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L568| unit=Track 15): [A][threshold] The acceptable false-positive bound is not numerically specified.
- **SUMMARY/I-20.01** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L574| unit=Track 15): [E][testing] Runs mutation testing inside the generation/audit loop.
- **SUMMARY/I-20.02** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L575| unit=Track 15): [E][selection] Selects mutants only from the changed hunk's data-flow cone.
- **SUMMARY/I-20.03** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L576| unit=Track 15): [E][analysis] Computes reachability through symbol-graph impact_radius.
- **SUMMARY/I-20.04** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L577| unit=Track 15): [E][budget] Time-boxes mutation work per hunk.
- **SUMMARY/I-20.05** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L578| unit=Track 15): [E][sampling] Samples mutants rather than requiring exhaustive enumeration.
- **SUMMARY/I-20.06** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L579| unit=Track 15): [E][measurement] Reports a confidence-interval mutation score.
- **SUMMARY/I-20.07** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L580| unit=Track 15): [E][control] Stops early when the confidence interval clears or fails the threshold.
- **SUMMARY/I-20.08** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L581| unit=Track 15): [E][cache] Caches results in the SSB.
- **SUMMARY/I-20.09** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L582| unit=Track 15): [E][cache-key] Keys cache entries by function content hash.
- **SUMMARY/I-20.10** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L583| unit=Track 15): [E][cache-key] Also binds cache entries to the test-set hash.
- **SUMMARY/I-20.11** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L584| unit=Track 15): [E][performance] Unchanged code does not repay mutation execution.
- **SUMMARY/I-20.12** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L585| unit=Track 15): [E][feedback] Converts surviving mutants into structured counterexample Directives.
- **SUMMARY/I-20.13** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L586| unit=Track 15): [E][feedback] Counterexamples identify the mutation distinction, operator, and source line.
- **SUMMARY/I-20.14** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L587| unit=Track 15): [E][control] Bounds generator retries after survivor feedback.
- **SUMMARY/I-20.15** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L588| unit=Track 15): [E][extension] Supports proptest and Hypothesis relation adapters as a parallel lane behind the D-3 trait.
- **SUMMARY/I-20.16** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L590| unit=Track 15): [E][integration] Wraps the existing mutation runner with mutation_budget.rs at audit and search seams.
- **SUMMARY/I-20.17** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L592| unit=Track 15 SURFACE): [E][UI] Shows a per-hunk mutation chip.
- **SUMMARY/I-20.18** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L593| unit=Track 15): [E][observability] Emits MutationBudgetedRun CiCleared with recomputable parameters and sample.
- **SUMMARY/I-20.19** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L595| unit=Track 15): [E][observability] Unchanged-function reaudit emits CacheHit and runs zero mutants.
- **SUMMARY/I-20.20** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L596| unit=Track 15): [E][observability] A survivor emits SurvivorDirective with operator and line.
- **SUMMARY/I-20.21** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L597| unit=Track 15 SURFACE): [E][measurement] Thresholds come from ELAI measurements, not paper constants.
- **SUMMARY/I-20.22** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L598| unit=Track 15): [A][configuration] Time-box length, sample policy, retry bound, and CI threshold are not numerically fixed in the summary.
- **SUMMARY/I-21.01** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L605| unit=Track 15): [E][separation] Gives generator and evaluator zero shared context.
- **SUMMARY/I-21.02** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L606| unit=Track 15): [E][planning] The generator proposes a structured Sprint Contract.
- **SUMMARY/I-21.03** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L607| unit=Track 15): [E][data] A Sprint Contract contains the goal.
- **SUMMARY/I-21.04** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L608| unit=Track 15): [E][data] A Sprint Contract contains files in scope.
- **SUMMARY/I-21.05** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L609| unit=Track 15): [E][data] A Sprint Contract contains acceptance conditions.
- **SUMMARY/I-21.06** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L610| unit=Track 15): [E][data] A Sprint Contract contains bounded scope.
- **SUMMARY/I-21.07** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L611| unit=Track 15): [E][workflow] An isolated evaluator approves or rejects before execution.
- **SUMMARY/I-21.08** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L612| unit=Track 15): [E][loop] Failed exit checks reinject the trace into a bounded Ralph loop.
- **SUMMARY/I-21.09** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L613| unit=Track 15): [E][budget] The Ralph loop runs at most three times.
- **SUMMARY/I-21.10** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L614| unit=Track 15): [E][escalation] The third failure escalates to the D-13 cross-family adversary.
- **SUMMARY/I-21.11** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L615| unit=Track 15): [E][output] The adversary must produce a concrete counterexample.
- **SUMMARY/I-21.12** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L616| unit=Track 15): [E][model-setting] Runs the evaluator at temperature zero.
- **SUMMARY/I-21.13** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L617| unit=Track 15): [E][role] Retains a hostile-critic evaluator role because this task is alignment-dependent.
- **SUMMARY/I-21.14** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L618| unit=Track 15): [E][integration] Reuses H-37 interactive plan-approval gate machinery.
- **SUMMARY/I-21.15** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L619| unit=Track 15): [E][security] Uses the I-1 supervisor as the isolation substrate.
- **SUMMARY/I-21.16** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L620| unit=Track 15 SURFACE): [E][UI] Adds a Sprint Contract card beside PlanApprovalCard.
- **SUMMARY/I-21.17** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L621| unit=Track 15): [E][failure] A vague goal is rejected before execution.
- **SUMMARY/I-21.18** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L622| unit=Track 15): [E][test] The evaluator's journaled context must contain zero generator turns.
- **SUMMARY/I-21.19** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L623| unit=Track 15): [E][observability] Emits RalphReinjected and, on the third failure, AdversaryEscalated.
- **SUMMARY/I-21.20** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L624| unit=Track 15): [E][determinism] Approval and execution rows must recompute under ELAI_SEED.
- **SUMMARY/I-22.01** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L631| unit=Track 15): [E][reward] Scores trajectories by progress delta phi(h_t) minus phi(h_t-1).
- **SUMMARY/I-22.02** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L632| unit=Track 15): [E][reward] Includes change in passing tests in phi.
- **SUMMARY/I-22.03** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L633| unit=Track 15): [E][reward] Includes falling compile-error count in phi.
- **SUMMARY/I-22.04** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L634| unit=Track 15): [E][reward] Includes covered requirements in phi.
- **SUMMARY/I-22.05** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L635| unit=Track 15): [E][invariant] Excludes LLM-opinion components from phi.
- **SUMMARY/I-22.06** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L636| unit=Track 15): [E][anti-gaming] Uses delta rather than absolute scores to prevent score inflation.
- **SUMMARY/I-22.07** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L637| unit=Track 15): [E][localization] Finds the first failing step.
- **SUMMARY/I-22.08** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L638| unit=Track 15): [E][recovery] Truncates the branch at that first error.
- **SUMMARY/I-22.09** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L639| unit=Track 15): [E][recovery] Preserves the valid prefix by forking H-49 at the truncation point.
- **SUMMARY/I-22.10** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L640| unit=Track 15): [E][anti-spiral] Decays reward during no-progress loops.
- **SUMMARY/I-22.11** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L641| unit=Track 15): [E][search] Backtracks after reward decay and expands a sibling.
- **SUMMARY/I-22.12** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L642| unit=Track 15): [E][integration] Shapes the D-2 PPM Layer-2 score and I-4 PUCT value.
- **SUMMARY/I-22.13** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L643| unit=Track 15 SURFACE): [E][UI] Shows a per-branch progress sparkline.
- **SUMMARY/I-22.14** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L644| unit=Track 15): [E][observability] Emits DecayApplied for isomorphic loops.
- **SUMMARY/I-22.15** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L645| unit=Track 15): [E][observability] Emits FirstErrorTruncated and an H-49 fork node.
- **SUMMARY/I-22.16** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L646| unit=Track 15): [E][traceability] Every phi component must be enumerable from deterministic journal data.
- **SUMMARY/I-23.01** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L652| unit=Track 15): [E][architecture] Exposes one conformal-calibration implementation.
- **SUMMARY/I-23.02** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L653| unit=Track 15): [E][consumer] Uses it for D-16 refusal.
- **SUMMARY/I-23.03** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L654| unit=Track 15): [E][consumer] Uses it for G-7 claim scores.
- **SUMMARY/I-23.04** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L655| unit=Track 15): [E][consumer] Uses it for I-4 search priors and uncertainty forks.
- **SUMMARY/I-23.05** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L656| unit=Track 15): [E][routing] High epistemic uncertainty forces exploration forks.
- **SUMMARY/I-23.06** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L657| unit=Track 15): [E][invariant] Prevents tau and coverage semantics from drifting between consumers.
- **SUMMARY/I-23.07** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L658| unit=Track 15): [E][integration] Re-exports the shipped calibration implementation through one facade and migrates call sites.
- **SUMMARY/I-23.08** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L660| unit=Track 15 SURFACE): [E][UI] Adds no new UI and preserves existing refusal surfaces.
- **SUMMARY/I-23.09** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L661| unit=Track 15): [E][observability] Refusal and search forks in one run must join to the same calibration ID and tau row.
- **SUMMARY/I-23.10** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L663| unit=Track 15): [E][test] A split-calibrations arm permits drift and must fail the new contract.
- **SUMMARY/I-23.11** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L664| unit=Track 15): [E][regression] All existing D-16 observable effects must remain unchanged.
- **SUMMARY/I-23.12** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L665| unit=Track 15): [E][scope] Adds no new mathematics; this is a unification feature.
- **SUMMARY/I-24.01** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L671| unit=Track 15): [E][canary] Injects AST-preserving obfuscated canaries into the Layer-2 scoring stream.
- **SUMMARY/I-24.02** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L672| unit=Track 15): [E][transform] Includes for-to-while or while-to-for transformations.
- **SUMMARY/I-24.03** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L673| unit=Track 15): [E][transform] Includes variable renames.
- **SUMMARY/I-24.04** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L674| unit=Track 15): [E][invariant] Preserves semantics while perturbing style.
- **SUMMARY/I-24.05** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L675| unit=Track 15): [E][detection] Detects style bias when the scorer's rating falls on an isomorphic variant.
- **SUMMARY/I-24.06** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L676| unit=Track 15): [E][observability] Logs the canary pair and both scores.
- **SUMMARY/I-24.07** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L677| unit=Track 15): [E][mitigation] Neutralizes a style-biased scorer's weight for the run.
- **SUMMARY/I-24.08** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L678| unit=Track 15): [E][lifecycle] Keeps the scorer neutralized until recalibration.
- **SUMMARY/I-24.09** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L679| unit=Track 15): [E][dependency] Extends the C-29 metamorphic engine and complements, rather than duplicates, H-44 candidate traps.
- **SUMMARY/I-24.10** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L681| unit=Track 15 SURFACE): [E][UI] Adds a scorer-health badge.
- **SUMMARY/I-24.11** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L682| unit=Track 15): [E][observability] Emits PrmCanaryEvent(StyleBiasDetected) and ScorerNeutralized.
- **SUMMARY/I-24.12** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L683| unit=Track 15): [E][behavior] An unbiased scorer emits CanaryInjected only.
- **SUMMARY/I-24.13** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L684| unit=Track 15): [E][security] Freezes the canary set out of band.
- **SUMMARY/I-24.14** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L685| unit=Track 15): [E][invariant] The scored model cannot author its own canaries.

#### SUMMARY / unit `Track 16` (77 records)

- **SUMMARY/I-25.01** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L693| unit=Track 16): [E][learning] Trains a small router on ELAI's own verified journal outcomes.
- **SUMMARY/I-25.02** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L694| unit=Track 16): [E][feature] Uses task class as a routing feature.
- **SUMMARY/I-25.03** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L695| unit=Track 16): [E][feature] Uses file count as a routing feature.
- **SUMMARY/I-25.04** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L696| unit=Track 16): [E][feature] Uses DKG centrality as a routing feature.
- **SUMMARY/I-25.05** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L697| unit=Track 16): [E][feature] Uses historical cost shape as a routing feature.
- **SUMMARY/I-25.06** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L698| unit=Track 16): [E][label] Learns outcomes per task features, execution lane, and cost.
- **SUMMARY/I-25.07** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L699| unit=Track 16): [E][routing] Pre-routes known-expensive AST shapes to the deep lane.
- **SUMMARY/I-25.08** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L700| unit=Track 16): [E][cost] Avoids burning cheap-lane tokens on shapes known to require deep execution.
- **SUMMARY/I-25.09** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L701| unit=Track 16): [E][promotion] Treats the learned router as an ID-6 ledger-gated artifact.
- **SUMMARY/I-25.10** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L702| unit=Track 16): [E][privacy] Composes with E-20 privacy routing masks.
- **SUMMARY/I-25.11** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L703| unit=Track 16): [E][integration] Hooks escalation_policy/cloud_mode and adds an offline trainer.
- **SUMMARY/I-25.12** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L704| unit=Track 16 SURFACE): [E][UI] Shows route explanations in the cost cockpit.
- **SUMMARY/I-25.13** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L705| unit=Track 16): [E][observability] RouteDecisionEvent includes the feature vector and predicted-versus-actual outcome.
- **SUMMARY/I-25.14** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L707| unit=Track 16): [E][measurement] Compares over-escalation and misroute rates with the H-7 heuristic over at least three runs.
- **SUMMARY/I-25.15** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L709| unit=Track 16): [E][honesty] Records an honest loss when the learned router does not beat the heuristic.
- **SUMMARY/I-25.16** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L710| unit=Track 16): [E][promotion] A losing router is not promoted and the block is ledgered.
- **SUMMARY/I-26.01** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L716| unit=Track 16): [E][budget] Scales search compute per node by serial depth.
- **SUMMARY/I-26.02** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L717| unit=Track 16): [E][budget] Gives deeper fixture tasks larger node budgets than shallow tasks for the same role.
- **SUMMARY/I-26.03** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L719| unit=Track 16): [E][collapse] At 85 percent budget, halts further exploration.
- **SUMMARY/I-26.04** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L720| unit=Track 16): [E][collapse] Scores the current frontier.
- **SUMMARY/I-26.05** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L721| unit=Track 16): [E][collapse] Synthesizes the best frontier result.
- **SUMMARY/I-26.06** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L722| unit=Track 16): [E][early-stop] Collapses early when Layer-2 scores remain flat across five supersteps.
- **SUMMARY/I-26.07** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L723| unit=Track 16): [E][provider] Maps cloud reasoning to the effort request parameter.
- **SUMMARY/I-26.08** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L724| unit=Track 16): [E][compatibility] Omits manual budget_tokens for Opus 4.7 and later.
- **SUMMARY/I-26.09** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L725| unit=Track 16): [E][constraint] Never forces JSON output for a reasoning role.
- **SUMMARY/I-26.10** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L726| unit=Track 16): [E][dependency] Reuses ReasoningBudgetScaled rows, extending payload only if needed.
- **SUMMARY/I-26.11** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L727| unit=Track 16): [E][priority] Keeps the E-39 hard cost ceiling supreme.
- **SUMMARY/I-26.12** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L728| unit=Track 16 SURFACE): [E][UI] Shows a branch-board budget bar.
- **SUMMARY/I-26.13** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L729| unit=Track 16): [E][observability] Collapse traces contain no post-collapse expansions.
- **SUMMARY/I-26.14** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L730| unit=Track 16): [E][measurement] A plateau fixture reports measured token savings.
- **SUMMARY/I-26.15** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L731| unit=Track 16): [E][test] A provider-builder contract test asserts budget_tokens is structurally absent on Opus 4.7-plus requests.
- **SUMMARY/I-27.01** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L738| unit=Track 16): [E][instrument] Measures verified-outcome rate against total test-time compute.
- **SUMMARY/I-27.02** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L739| unit=Track 16): [E][metric] Counts nodes expanded.
- **SUMMARY/I-27.03** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L740| unit=Track 16): [E][metric] Counts tokens.
- **SUMMARY/I-27.04** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L741| unit=Track 16): [E][metric] Counts monetary cost.
- **SUMMARY/I-27.05** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L742| unit=Track 16): [E][invariant] Uses W-1-grade verified outcomes rather than pass-at-k claims.
- **SUMMARY/I-27.06** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L743| unit=Track 16): [E][visualization] Produces a Pareto cost/accuracy view.
- **SUMMARY/I-27.07** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L744| unit=Track 16): [E][operator] Lets an operator choose an operating point.
- **SUMMARY/I-27.08** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L745| unit=Track 16): [E][honesty] Reports non-monotonic regions instead of smoothing them.
- **SUMMARY/I-27.09** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L746| unit=Track 16 SURFACE): [E][CLI] Adds elai bench convergence with a fixture suite argument.
- **SUMMARY/I-27.10** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L747| unit=Track 16 SURFACE): [E][UI] Adds a convergence-curve cockpit tile.
- **SUMMARY/I-27.11** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L748| unit=Track 16): [E][observability] Emits ConvergenceCurveMeasured with points recomputable from the journal.
- **SUMMARY/I-27.12** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L749| unit=Track 16): [E][analysis] Names the knee of the curve.
- **SUMMARY/I-27.13** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L750| unit=Track 16): [E][analysis] Reports cost per additional verified task.
- **SUMMARY/I-27.14** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L751| unit=Track 16): [E][test] A skip-verification arm must inflate the curve and fail.
- **SUMMARY/I-28.01** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L757| unit=Track 16): [E][credit] Walks backward from COMPLETE through the winning DAG trajectory.
- **SUMMARY/I-28.02** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L758| unit=Track 16): [E][credit] Assigns causal credit to retrieval chunks.
- **SUMMARY/I-28.03** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L759| unit=Track 16): [E][credit] Assigns causal credit to subplans.
- **SUMMARY/I-28.04** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L760| unit=Track 16): [E][credit] Assigns causal credit to template choices.
- **SUMMARY/I-28.05** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L761| unit=Track 16): [E][training-data] Extracts DPO pairs as a byproduct.
- **SUMMARY/I-28.06** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L762| unit=Track 16): [E][pair] Uses the winning branch as the positive.
- **SUMMARY/I-28.07** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L763| unit=Track 16): [E][pair] Uses the highest-Layer-2-scoring vetoed sibling as the fluent-but-wrong negative.
- **SUMMARY/I-28.08** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L764| unit=Track 16): [E][retrieval] Demotes chunks that consistently precede aborts.
- **SUMMARY/I-28.09** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L765| unit=Track 16): [E][decline] Declines Shapley/SHARP credit until RL infrastructure exists.
- **SUMMARY/I-28.10** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L766| unit=Track 16): [E][integration] Runs dag_credit.rs at PhaseComplete.
- **SUMMARY/I-28.11** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L767| unit=Track 16 SURFACE): [E][UI] Optionally displays credit heat on the plan view; absence may be W-6 justified.
- **SUMMARY/I-28.12** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L768| unit=Track 16): [E][observability] Emits journal-rederivable DagCreditAssigned rows.
- **SUMMARY/I-28.13** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L769| unit=Track 16): [E][artifact] Writes DPO pairs whose hashes match sealed RRA entries.
- **SUMMARY/I-28.14** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L770| unit=Track 16): [E][threshold] Demotes a chunk after three consecutive abort-preceding appearances in the stated observable.
- **SUMMARY/I-28.15** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L772| unit=Track 16): [E][test] The next seeded retrieval ranking must provably change after ChunkDemoted.
- **SUMMARY/I-28.16** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L773| unit=Track 16): [A][future] Shapley/SHARP is explicitly revisit-only and not part of the current capability.
- **SUMMARY/I-29.01** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L780| unit=Track 16): [E][generation] Adds a diffusion LLM as a fast, untrusted draft generator.
- **SUMMARY/I-29.02** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L781| unit=Track 16): [E][scope] Uses the lane for drafts and apply-edit streams.
- **SUMMARY/I-29.03** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L782| unit=Track 16): [E][trust] Subjects diffusion output to the identical I-17 and I-20 gates used for other generators.
- **SUMMARY/I-29.04** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L784| unit=Track 16): [E][concurrency] Uses CRDT merge semantics for parallel patches to a shared draft.
- **SUMMARY/I-29.05** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L785| unit=Track 16): [E][determinism] Requires parallel draft merges to be byte-identical under seed.
- **SUMMARY/I-29.06** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L786| unit=Track 16): [E][validation] Runs C-32 MiniCritic semantic checkpoints after each merge.
- **SUMMARY/I-29.07** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L787| unit=Track 16): [E][decline] Declines the full TTD-DR denoising-research loop.
- **SUMMARY/I-29.08** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L788| unit=Track 16): [E][adapter] Implements the dLLM client as another RuntimeAdapter variant.
- **SUMMARY/I-29.09** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L789| unit=Track 16): [E][dependency] Requires user approval and anti-slopsquat checks for the new dependency.
- **SUMMARY/I-29.10** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L790| unit=Track 16 SURFACE): [E][UI] Adds a draft-lane badge to generation cards.
- **SUMMARY/I-29.11** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L791| unit=Track 16): [E][test] The same Layer-1 gate that rejects main-generator output must reject a bad diffusion draft.
- **SUMMARY/I-29.12** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L793| unit=Track 16): [E][observability] Emits DiffusionDraftEvent DraftVetoed and CrdtMerged.
- **SUMMARY/I-29.13** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L794| unit=Track 16 SURFACE): [E][feature-flag] With the lane flag absent, existing paths are untouched.
- **SUMMARY/I-29.14** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L795| unit=Track 16): [E][rollout] Becomes default-on for drafts only after its corruption rate is bounded.
- **SUMMARY/I-29.15** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L796| unit=Track 16): [E][fallback] Otherwise remains an allowlisted experiment.
- **SUMMARY/I-29.16** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L797| unit=Track 16): [A][implementation] Automerge-class versus hand-rolled deterministic CRDT merge remains an explicit choice.

#### SUMMARY / unit `Track 17` (46 records)

- **SUMMARY/I-30.01** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L807| unit=Track 17): [E][dataset] Extracts root-to-verified-COMPLETE winning paths from sealed RRAs.
- **SUMMARY/I-30.02** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L808| unit=Track 17): [E][eligibility] Excludes unsealed runs from the dataset.
- **SUMMARY/I-30.03** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L809| unit=Track 17): [E][format] Exports supervised-fine-tuning JSONL.
- **SUMMARY/I-30.04** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L810| unit=Track 17): [E][negative-data] Records highest-Layer-2-scoring branches that failed deterministic checks.
- **SUMMARY/I-30.05** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L811| unit=Track 17): [E][semantics] Treats those negatives as canonical fluent-but-wrong examples.
- **SUMMARY/I-30.06** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L812| unit=Track 17): [E][training] Uses rejection-sampling SFT first.
- **SUMMARY/I-30.07** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L813| unit=Track 17): [E][training] Trains LoRA on the canonical local model.
- **SUMMARY/I-30.08** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L814| unit=Track 17): [E][strategy] Defers RL instability by taking the SFT gain first.
- **SUMMARY/I-30.09** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L815| unit=Track 17): [E][provenance] Every dataset row carries a provenance hash back to its RRA.
- **SUMMARY/I-30.10** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L816| unit=Track 17): [E][evaluation] Uses held-out evaluation and the H-17/H-18 promotion ledger.
- **SUMMARY/I-30.11** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L817| unit=Track 17 SURFACE): [E][CLI] Adds elai dataset export --format sft.
- **SUMMARY/I-30.12** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L818| unit=Track 17): [E][operations] Stores offline training scripts under workflow/training.
- **SUMMARY/I-30.13** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L819| unit=Track 17 SURFACE): [E][UI] Adds a dataset card with counts and provenance.
- **SUMMARY/I-30.14** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L820| unit=Track 17): [E][observability] Emits SftExhaustEvent rows.
- **SUMMARY/I-30.15** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L821| unit=Track 17): [E][integrity] A tampered dataset record must fail RRA provenance verification.
- **SUMMARY/I-30.16** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L822| unit=Track 17): [E][traceability] Negative pairs must reference real vetoed branches.
- **SUMMARY/I-30.17** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L823| unit=Track 17): [E][operations] Keeps the LoRA training run behind a RunPod checkbox.
- **SUMMARY/I-30.18** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L824| unit=Track 17): [E][promotion] Any resulting checkpoint can enter promotion only through ID-6 gates.
- **SUMMARY/I-30.19** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L825| unit=Track 17): [E][promotion] Promotion cites trial count, deflation, and held-out results.
- **SUMMARY/I-31.01** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L832| unit=Track 17): [E][training] Runs GRPO on execution-grounded rewards.
- **SUMMARY/I-31.02** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L833| unit=Track 17): [E][reward] Includes compile verdicts.
- **SUMMARY/I-31.03** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L834| unit=Track 17): [E][reward] Includes test verdicts.
- **SUMMARY/I-31.04** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L835| unit=Track 17): [E][reward] Includes mutation verdicts.
- **SUMMARY/I-31.05** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L836| unit=Track 17): [E][reward] Includes null-test verdicts.
- **SUMMARY/I-31.06** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L837| unit=Track 17): [E][reward] Applies XGrammar conformance as a format penalty.
- **SUMMARY/I-31.07** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L838| unit=Track 17): [E][service] Serves rewards through a typed local endpoint.
- **SUMMARY/I-31.08** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L839| unit=Track 17): [E][anti-collapse] Applies KL penalty against mode collapse.
- **SUMMARY/I-31.09** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L840| unit=Track 17): [E][training] Uses I-30 negative-trajectory masking.
- **SUMMARY/I-31.10** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L841| unit=Track 17): [E][blocking] Requires live H-43 execution-grounded fitness and prohibits byte-hash fitness.
- **SUMMARY/I-31.11** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L842| unit=Track 17): [E][blocking] Requires live H-44 behavioral traps.
- **SUMMARY/I-31.12** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L843| unit=Track 17): [E][blocking] Requires I-24 canaries in the scoring stream.
- **SUMMARY/I-31.13** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L844| unit=Track 17): [E][blocking] Requires the H-28 reward-tampering gate.
- **SUMMARY/I-31.14** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L845| unit=Track 17): [E][security] Freezes the evaluator, harness, and thresholds in the R8 set.
- **SUMMARY/I-31.15** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L846| unit=Track 17): [E][promotion] Requires H-17 trial-N evidence.
- **SUMMARY/I-31.16** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L847| unit=Track 17): [E][promotion] Requires H-18 deflation.
- **SUMMARY/I-31.17** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L848| unit=Track 17): [E][promotion] Requires D-6 held-out results.
- **SUMMARY/I-31.18** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L849| unit=Track 17): [E][promotion] Requires the D-7 transfer canary.
- **SUMMARY/I-31.19** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L850| unit=Track 17): [E][kill-switch] Keeps the whole lane behind the existing allowlisted darwinian-evolve kill switch.
- **SUMMARY/I-31.20** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L852| unit=Track 17): [E][constraint] Inherits the hard avoidance of DPO.
- **SUMMARY/I-31.21** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L853| unit=Track 17): [E][security] Places the reward endpoint on the I-1 supervisor side so the trained policy cannot reach reward code.
- **SUMMARY/I-31.22** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L855| unit=Track 17): [E][dependency] Requires user approval and anti-slopsquat review for new RL dependencies.
- **SUMMARY/I-31.23** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L856| unit=Track 17 SURFACE): [E][UI] Shows flywheel admits, blocks, and KL trend.
- **SUMMARY/I-31.24** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L857| unit=Track 17): [E][observability] Every GrpoRewardComputed/RewardServed row cites a gate-verdict row.
- **SUMMARY/I-31.25** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L858| unit=Track 17): [E][test] A harness-edit reward-hack fixture is hard-blocked by H-28.
- **SUMMARY/I-31.26** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L859| unit=Track 17): [E][promotion] A checkpoint either has a full PromotionLedgered chain or does not promote.
- **SUMMARY/I-31.27** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L860| unit=Track 17): [E][operations] Ships the deterministic fixture harness before the separately gated multi-night RunPod run.

#### SUMMARY / unit `Track 18` (176 records)

- **SUMMARY/I-32.01** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L869| unit=Track 18): [E][role] Removes narrative personas from executor prompts.
- **SUMMARY/I-32.02** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L870| unit=Track 18): [E][role] Removes narrative personas from context-compiler prompts.
- **SUMMARY/I-32.03** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L871| unit=Track 18): [E][role] Removes narrative personas from researcher prompts.
- **SUMMARY/I-32.04** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L872| unit=Track 18): [E][role-contract] Replaces those personas with schema-first role contracts.
- **SUMMARY/I-32.05** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L873| unit=Track 18): [E][role-contract] Specifies typed inputs and outputs.
- **SUMMARY/I-32.06** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L874| unit=Track 18): [E][role-contract] Specifies explicit constraints.
- **SUMMARY/I-32.07** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L875| unit=Track 18): [E][role] Retains hostile-critic framing for auditors.
- **SUMMARY/I-32.08** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L876| unit=Track 18): [E][role] Retains hostile-critic framing for critics.
- **SUMMARY/I-32.09** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L877| unit=Track 18): [E][role] Retains hostile-critic framing for adversaries.
- **SUMMARY/I-32.10** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L878| unit=Track 18): [E][role] Retains anti-spiral's five lateral personas because creative perturbation is an alignment-dependent use.
- **SUMMARY/I-32.11** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L880| unit=Track 18): [E][measurement] Runs a per-role A/B fixture evaluation before flipping prompts.
- **SUMMARY/I-32.12** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L881| unit=Track 18): [E][integration] Uses Phase 2C per-role GRC configurations, role templates, and dispatch configs.
- **SUMMARY/I-32.13** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L883| unit=Track 18 SURFACE): [E][UI] Adds no UI because the change is prompt-internal.
- **SUMMARY/I-32.14** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L884| unit=Track 18): [E][test] Executor system prompts must contain a schema contract and zero persona narrative.
- **SUMMARY/I-32.15** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L885| unit=Track 18): [E][observability] Emits RoleContractApplied(AbDelta) with seeded fixture deltas.
- **SUMMARY/I-32.16** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L886| unit=Track 18): [E][observability] Retained auditor framing emits PersonaRetained.
- **SUMMARY/I-32.17** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L887| unit=Track 18): [E][promotion] Any role fixture-pass regression blocks the refactor flip.
- **SUMMARY/I-33.01** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L893| unit=Track 18): [E][scope] Wraps decision-type LLM calls such as mode selection, gate advice, and classification.
- **SUMMARY/I-33.02** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L895| unit=Track 18): [E][CEFL] Has the orchestrator externalize the entire candidate set.
- **SUMMARY/I-33.03** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L896| unit=Track 18): [E][CEFL] Restricts the model to scoring or ranking supplied candidates.
- **SUMMARY/I-33.04** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L897| unit=Track 18): [E][invariant] Makes model-invented selection options structurally unparseable.
- **SUMMARY/I-33.05** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L898| unit=Track 18): [E][I6Q] Validates rationale quality deterministically.
- **SUMMARY/I-33.06** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L899| unit=Track 18): [E][I6Q] Enforces a minimum rationale length.
- **SUMMARY/I-33.07** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L900| unit=Track 18): [E][I6Q] Enforces lexical diversity.
- **SUMMARY/I-33.08** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L901| unit=Track 18): [E][I6Q] Rejects vacuous but superficially compliant rationales.
- **SUMMARY/I-33.09** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L902| unit=Track 18): [E][integrity] Uses commit-reveal for stochastic gate choices.
- **SUMMARY/I-33.10** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L903| unit=Track 18): [E][integrity] Commits blake3(seed, choice) before revealing the choice.
- **SUMMARY/I-33.11** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L904| unit=Track 18): [E][anti-gaming] Prevents the model from exploiting choice ordering.
- **SUMMARY/I-33.12** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L905| unit=Track 18): [E][metric] Ledgers cosmetic-deadlock rate.
- **SUMMARY/I-33.13** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L906| unit=Track 18): [E][metric] Ledgers deferral information utilization.
- **SUMMARY/I-33.14** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L907| unit=Track 18): [E][metric] Ledgers framing-success rate.
- **SUMMARY/I-33.15** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L908| unit=Track 18): [E][measurement] Measures governance decoupling under task stress locally rather than adopting the dossier result.
- **SUMMARY/I-33.16** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L910| unit=Track 18): [E][integration] Wraps prompt_decision and other LLM-decision seams in decision_governance.rs.
- **SUMMARY/I-33.17** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L912| unit=Track 18 SURFACE): [E][UI] Adds a governance-metrics tile.
- **SUMMARY/I-33.18** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L913| unit=Track 18): [E][failure] A vacuous rationale emits GovernancePrimitiveFired I6qRejected.
- **SUMMARY/I-33.19** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L914| unit=Track 18): [E][test] Reversing field order must produce the same decision on the framing-inversion fixture.
- **SUMMARY/I-33.20** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L916| unit=Track 18): [E][observability] CommitRevealSealed rows must put commit before reveal.
- **SUMMARY/I-33.21** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L917| unit=Track 18): [E][test] An out-of-order commit/reveal arm fails.
- **SUMMARY/I-34.01** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L923| unit=Track 18): [E][decoding] Activates CRANE-style grammar augmentation on the local lane.
- **SUMMARY/I-34.02** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L924| unit=Track 18): [E][decoding] Permits free-form thinking tokens between structural commitments.
- **SUMMARY/I-34.03** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L925| unit=Track 18): [E][measurement] Measures whether the hybrid recovers the constrained-reasoning tax.
- **SUMMARY/I-34.04** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L926| unit=Track 18): [E][decoding] Applies constraints only to final output in reflection/self-correction loops.
- **SUMMARY/I-34.05** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L927| unit=Track 18): [E][safety] Avoids structure snowballing during intermediate reflection.
- **SUMMARY/I-34.06** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L928| unit=Track 18): [E][control-plane] Stores grammars as pinned artifacts.
- **SUMMARY/I-34.07** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L929| unit=Track 18): [E][integrity] Pins grammar bytes by blake3, analogous to MCP schemas.
- **SUMMARY/I-34.08** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L930| unit=Track 18): [E][security] Never loads third-party grammars.
- **SUMMARY/I-34.09** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L931| unit=Track 18): [E][security] Rejects enum-bearing schemas from untrusted sources.
- **SUMMARY/I-34.10** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L932| unit=Track 18): [E][validation] Revalidates every constrained output after parsing.
- **SUMMARY/I-34.11** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L933| unit=Track 18): [E][fallback] Routes output that fails post-parse validation to refusal.
- **SUMMARY/I-34.12** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L934| unit=Track 18): [E][invariant] Never silently accepts a noncompliant decoded output.
- **SUMMARY/I-34.13** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L935| unit=Track 18): [E][integration] Adds the runtime leg in providers decoding and the sidecar mask layer.
- **SUMMARY/I-34.14** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L936| unit=Track 18): [E][security] Freezes the grammar pin manifest in R8.
- **SUMMARY/I-34.15** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L937| unit=Track 18 SURFACE): [E][UI] Shows grammar-pin status in the existing decode panel.
- **SUMMARY/I-34.16** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L938| unit=Track 18): [E][measurement] A reasoning fixture must produce thinking plus valid structure with pass rate at least the FreeText arm.
- **SUMMARY/I-34.17** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L940| unit=Track 18): [E][measurement] Records the tight-grammar arm's measured tax.
- **SUMMARY/I-34.18** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L941| unit=Track 18): [E][failure] A mutated grammar file emits GrammarGateEvent PinViolation and is refused.
- **SUMMARY/I-34.19** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L942| unit=Track 18): [E][failure] A silently noncompliant engine-stub output emits PostParseRejected.
- **SUMMARY/I-34.20** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L943| unit=Track 18): [E][test] A trust-mask broken arm must ship the bad output and fail.
- **SUMMARY/I-35.01** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L949| unit=Track 18): [E][policy] Evaluates rules over ordered tool-call history.
- **SUMMARY/I-35.02** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L950| unit=Track 18): [E][policy] Starts a rule when a configured tool-class trigger fires.
- **SUMMARY/I-35.03** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L951| unit=Track 18): [E][predicate] Can require tests since the most recent source write.
- **SUMMARY/I-35.04** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L952| unit=Track 18): [E][predicate] Can forbid a destructive verb after untrusted ingest until a taint check has run.
- **SUMMARY/I-35.05** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L954| unit=Track 18): [E][predicate] Can require an auditor-PASS row before git push.
- **SUMMARY/I-35.06** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L955| unit=Track 18): [E][enforcement] Can halt a call.
- **SUMMARY/I-35.07** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L956| unit=Track 18): [E][enforcement] Can require human confirmation.
- **SUMMARY/I-35.08** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L957| unit=Track 18): [E][enforcement] Can substitute another action.
- **SUMMARY/I-35.09** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L958| unit=Track 18): [E][security] Runs rules on the I-1 Supervisor plane.
- **SUMMARY/I-35.10** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L959| unit=Track 18): [E][security] Is immune to token-level attacks because it evaluates actions and history, not generated text.
- **SUMMARY/I-35.11** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L961| unit=Track 18): [E][scope] Adds temporal ordering on top of state masks, per-call argv shape, and per-task committed tool sets.
- **SUMMARY/I-35.12** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L963| unit=Track 18): [E][configuration] Stores rules in TOML.
- **SUMMARY/I-35.13** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L964| unit=Track 18): [E][security] Freezes the safety-core rule set with mutable equal to zero.
- **SUMMARY/I-35.14** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L965| unit=Track 18 SURFACE): [E][UI] Shows sequence-verdict rows in the gate panel.
- **SUMMARY/I-35.15** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L966| unit=Track 18): [E][failure] Git push without an in-session auditor PASS is halted.
- **SUMMARY/I-35.16** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L967| unit=Track 18): [E][diagnostics] ToolSequenceVerdict OrderViolation names the unsatisfied predicate.
- **SUMMARY/I-35.17** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L968| unit=Track 18): [E][behavior] A compliant sequence emits Allowed and proceeds.
- **SUMMARY/I-35.18** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L969| unit=Track 18): [E][integrity] Includes the sequence-rule hash in the frozen manifest.
- **SUMMARY/I-35.19** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L970| unit=Track 18): [E][security] Blocks a run that edits its own sequence rules.
- **SUMMARY/I-35.20** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L971| unit=Track 18): [E][determinism] Replay rederives identical sequence verdicts.
- **SUMMARY/I-35.21** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L972| unit=Track 18): [E][deferred] Defers Pro2Guard-style probabilistic reachability until a trace corpus exists.
- **SUMMARY/I-36.01** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L978| unit=Track 18): [E][artifact] Stores compiled prompt variants keyed by role and backend.
- **SUMMARY/I-36.02** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L979| unit=Track 18): [E][workflow] Produces variants offline.
- **SUMMARY/I-36.03** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L980| unit=Track 18): [E][optimization] Uses the cloud-lane optimizer to search instructions and exemplars.
- **SUMMARY/I-36.04** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L981| unit=Track 18): [E][compatibility] Emits plain-string prompts so call sites need not refactor.
- **SUMMARY/I-36.05** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L982| unit=Track 18 SURFACE): [E][validation] Validates variants on ELAI fixtures.
- **SUMMARY/I-36.06** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L983| unit=Track 18): [E][measurement] Measures and ledgers the source-to-target prompt transfer gap.
- **SUMMARY/I-36.07** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L984| unit=Track 18): [E][promotion] Promotes compiled prompts through the ID-6 ledger.
- **SUMMARY/I-36.08** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L985| unit=Track 18): [E][replay] Emits ID-3 receipts that identify the exact served variant.
- **SUMMARY/I-36.09** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L986| unit=Track 18): [E][provider-hygiene] Removes explicit step-by-step scaffolds under extended thinking.
- **SUMMARY/I-36.10** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L987| unit=Track 18): [E][provider-hygiene] Respects backend temperature defaults.
- **SUMMARY/I-36.11** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L988| unit=Track 18): [E][integration] Keys on B-37 PerRoleRuntime and shares the E-8 GEPA optimizer lane.
- **SUMMARY/I-36.12** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L989| unit=Track 18 SURFACE): [E][UI] Shows a per-role compiled-variant badge in SettingsWindow.
- **SUMMARY/I-36.13** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L990| unit=Track 18): [E][observability] PromptCompiled rows contain the variant hash and backend.
- **SUMMARY/I-36.14** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L991| unit=Track 18): [E][measurement] TransferGapMeasured compares source prompt on target with compiled prompt on target.
- **SUMMARY/I-36.15** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L993| unit=Track 18): [E][rollback] A variant below its fixture floor automatically rolls back.
- **SUMMARY/I-36.16** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L994| unit=Track 18): [E][observability] Rollback emits Rolledback.
- **SUMMARY/I-36.17** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L995| unit=Track 18): [E][test] A cloud extended-thinking request must contain no chain-of-thought scaffold.
- **SUMMARY/I-37.01** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1001| unit=Track 18): [E][requirements] Represents each atomic EARS-pattern requirement as a typed record.
- **SUMMARY/I-37.02** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1002| unit=Track 18): [E][data] Stores a content hash.
- **SUMMARY/I-37.03** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1003| unit=Track 18): [E][data] Binds tests to the requirement.
- **SUMMARY/I-37.04** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1004| unit=Track 18): [E][data] Stores I-20 mutation coverage for those tests.
- **SUMMARY/I-37.05** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1005| unit=Track 18): [E][data] Binds applicable metamorphic relations.
- **SUMMARY/I-37.06** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1006| unit=Track 18): [E][state] Computes a per-requirement VERIFIED or UNVERIFIED state.
- **SUMMARY/I-37.07** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1007| unit=Track 18 SURFACE): [E][UI] Surfaces requirement status per hunk in the reviewer cockpit.
- **SUMMARY/I-37.08** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1008| unit=Track 18): [E][interop] Imports Spec-Kit-style specification files.
- **SUMMARY/I-37.09** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1009| unit=Track 18): [E][interop] Imports Kiro-style specification files.
- **SUMMARY/I-37.10** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1010| unit=Track 18 SURFACE): [E][trust] Treats imported formats as intent sources while ELAI separately proves satisfaction.
- **SUMMARY/I-37.11** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1012| unit=Track 18): [E][status-example] Can define satisfaction through test coverage, sampled mutation score, and a named metamorphic relation.
- **SUMMARY/I-37.12** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1014| unit=Track 18): [E][honesty] Shows UNVERIFIED when those evidence conditions are absent.
- **SUMMARY/I-37.13** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1015| unit=Track 18): [E][dependency] Generalizes C-29 IntentPair, which becomes the simplest EARS record.
- **SUMMARY/I-37.14** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1016| unit=Track 18): [E][workflow] Authors or imports requirements at the PLANNING gate before TESTING_FIRST.
- **SUMMARY/I-37.15** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1017| unit=Track 18): [E][traceability] Each satisfied chip points to real test and mutation rows.
- **SUMMARY/I-37.16** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1018| unit=Track 18): [E][lifecycle] Deleting a bound test flips the state to Unverified on reaudit.
- **SUMMARY/I-37.17** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1019| unit=Track 18): [E][observability] Regression emits Regressed.
- **SUMMARY/I-37.18** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1020| unit=Track 18): [E][round-trip] Imported Spec-Kit data round-trips into typed records.
- **SUMMARY/I-37.19** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1021| unit=Track 18): [E][fail-closed] An unverifiable requirement remains visible and is never silently dropped.
- **SUMMARY/I-37.20** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1022| unit=Track 18): [A][threshold] The cited 88-percent sampled-mutation rule is presented as an example requirement expression, not clearly declared as a universal threshold.
- **SUMMARY/I-38.01** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1029| unit=Track 18): [E][benchmark] Defines QDELTA as defective diffs reaching a reviewer per 100 tasks.
- **SUMMARY/I-38.02** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1030| unit=Track 18): [E][ablation] Compares gates-on against gates-off.
- **SUMMARY/I-38.03** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1031| unit=Track 18): [E][defect] Counts held-out test failures.
- **SUMMARY/I-38.04** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1032| unit=Track 18): [E][defect] Counts surviving mutants found after merge.
- **SUMMARY/I-38.05** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1033| unit=Track 18): [E][defect] Counts regressions.
- **SUMMARY/I-38.06** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1034| unit=Track 18): [E][statistics] Uses bootstrap confidence intervals clustered by repository.
- **SUMMARY/I-38.07** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1035| unit=Track 18): [E][policy] Mechanically requires paired-task designs.
- **SUMMARY/I-38.08** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1036| unit=Track 18): [E][policy] Requires at least five seeds per condition.
- **SUMMARY/I-38.09** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1037| unit=Track 18): [E][reporting] Reports standard error of the mean.
- **SUMMARY/I-38.10** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1038| unit=Track 18): [E][policy] Requires pass-to-the-k reporting for reliability claims.
- **SUMMARY/I-38.11** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1039| unit=Track 18): [E][reporting] Reports dollars per task beside accuracy.
- **SUMMARY/I-38.12** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1040| unit=Track 18): [E][corpus] Uses date-stamped task cuts.
- **SUMMARY/I-38.13** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1041| unit=Track 18): [E][contamination] Surfaces the D-20 memorization probe.
- **SUMMARY/I-38.14** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1042| unit=Track 18): [E][framing] Reports only a gate ablation, never a leaderboard ranking.
- **SUMMARY/I-38.15** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1043| unit=Track 18): [E][integration] Mirrors the SAFEDELTA harness shape and composes the I-20 mutant leg.
- **SUMMARY/I-38.16** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1044| unit=Track 18): [E][artifact] Auto-generates methodology documentation from machine-readable configuration.
- **SUMMARY/I-38.17** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1045| unit=Track 18 SURFACE): [E][UI] Places a QDELTA tile beside SAFEDELTA.
- **SUMMARY/I-38.18** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1046| unit=Track 18 SURFACE): [E][CLI] elai bench qdelta emits QdeltaMeasured with on/off delta and confidence interval.
- **SUMMARY/I-38.19** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1047| unit=Track 18): [E][operations] Ships the fixture tier now and leaves the full corpus behind a RunPod checkbox.
- **SUMMARY/I-38.20** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1049| unit=Track 18): [E][fail-closed] Refuses runs with only three seeds or an unpaired design.
- **SUMMARY/I-38.21** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1050| unit=Track 18): [E][determinism] Regenerates the methodology document deterministically.
- **SUMMARY/I-38.22** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1051| unit=Track 18): [E][traceability] The methodology names the D-20 contamination result.
- **SUMMARY/I-39.01** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1057| unit=Track 18 SURFACE): [E][CLI] Adds elai plan status.
- **SUMMARY/I-39.02** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1058| unit=Track 18): [E][derivation] Derives per-plan state from journal data.
- **SUMMARY/I-39.03** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1059| unit=Track 18): [E][derivation] Derives per-plan state from plan-status fragments.
- **SUMMARY/I-39.04** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1060| unit=Track 18): [E][derivation] Derives per-plan state from wiring_contract_check results.
- **SUMMARY/I-39.05** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1061| unit=Track 18): [E][traceability] Adds a verified-against commit or map-date anchor to every output line.
- **SUMMARY/I-39.06** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1062| unit=Track 18): [E][lifecycle] Demotes hand-maintained tallies to generated build artifacts.
- **SUMMARY/I-39.07** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1063| unit=Track 18): [E][migration] Adds one journal-rot migration test.
- **SUMMARY/I-39.08** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1064| unit=Track 18): [E][migration] Replays an archived old-schema ssb.db through current code.
- **SUMMARY/I-39.09** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1065| unit=Track 18): [E][trust] Treats old RRAs as a trust product that must not silently rot.
- **SUMMARY/I-39.10** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1066| unit=Track 18): [E][integration] Reads SSB viewer paths, s-plan-audit fragments, wiring checks, F-13 archive, and E-44 migration machinery.
- **SUMMARY/I-39.11** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1068| unit=Track 18 SURFACE): [E][UI] Adds no separate UI because the CLI and generated document are the load-bearing artifacts.
- **SUMMARY/I-39.12** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1070| unit=Track 18 SURFACE): [E][CLI] Letter-filtered status output carries fragment path, checker verdict, and journal row for each session.
- **SUMMARY/I-39.13** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1072| unit=Track 18): [E][determinism] Identical inputs regenerate byte-identical status output.
- **SUMMARY/I-39.14** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1073| unit=Track 18): [E][test] The migration test opens a real F-13 archived database and decodes N rows.
- **SUMMARY/I-39.15** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1074| unit=Track 18): [E][observability] Successful replay emits PlanStateDerived.
- **SUMMARY/I-39.16** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1075| unit=Track 18): [E][fail-closed] A corrupt archive fails loudly and never skips.
- **SUMMARY/I-39.17** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1076| unit=Track 18): [A][status] The summary reports that five sources of truth drifted but does not enumerate all five in this section.
- **SUMMARY/I-40.01** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1083| unit=Track 18): [E][standard] Publishes the RRA format as an open specification.
- **SUMMARY/I-40.02** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1084| unit=Track 18): [E][standard] Versions the RRA specification.
- **SUMMARY/I-40.03** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1085| unit=Track 18): [E][verification] Ships an RRA conformance checker.
- **SUMMARY/I-40.04** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1086| unit=Track 18 SURFACE): [E][wrapper] Wraps an external agent through a minimal proxy/MCP surface.
- **SUMMARY/I-40.05** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1087| unit=Track 18 SURFACE): [E][wrapper] Routes external-agent tool traffic through ELAI taint tagging.
- **SUMMARY/I-40.06** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1088| unit=Track 18 SURFACE): [E][wrapper] Routes external-agent writes through ELAI write gates.
- **SUMMARY/I-40.07** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1089| unit=Track 18): [E][artifact] Emits an RRA for the wrapped external run.
- **SUMMARY/I-40.08** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1090| unit=Track 18): [E][strategy] Positions the wrapper and open standard as the adoption wedge.
- **SUMMARY/I-40.09** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1091| unit=Track 18): [E][strategy] Keeps the deep orchestrator as the premium destination.
- **SUMMARY/I-40.10** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1092| unit=Track 18): [E][privacy] Makes G-9 redaction a publication precondition.
- **SUMMARY/I-40.11** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1093| unit=Track 18): [E][privacy] Exported RRAs must not leak secrets.
- **SUMMARY/I-40.12** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1094| unit=Track 18): [E][integration] Composes existing taint, write/dependency/blast-radius gates, RRA, attestation, adapter, and MCP-hardening surfaces without forking them.
- **SUMMARY/I-40.13** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1096| unit=Track 18): [E][spec-path] Names docs/spec/rra-spec-v1.md as the in-repo versioned specification.
- **SUMMARY/I-40.14** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1097| unit=Track 18 SURFACE): [E][CLI] Adds elai wrap -- <agent command>.
- **SUMMARY/I-40.15** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1098| unit=Track 18): [E][proxy] Proxies wrapped tool input and output.
- **SUMMARY/I-40.16** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1099| unit=Track 18 SURFACE): [E][UI] Renders wrapped-run rows in the existing RRA viewer.
- **SUMMARY/I-40.17** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1100| unit=Track 18): [E][test] A wrapped external fixture must produce a spec-conformant RRA.
- **SUMMARY/I-40.18** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1101| unit=Track 18): [E][observability] Wrapped untrusted input creates taint rows.
- **SUMMARY/I-40.19** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1102| unit=Track 18): [E][security] A seeded malicious write from the wrapped agent must be blocked.
- **SUMMARY/I-40.20** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1103| unit=Track 18): [E][test] The conformance suite rejects a tampered artifact.
- **SUMMARY/I-40.21** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1104| unit=Track 18): [E][test] The exported bundle must contain no raw planted secret bytes.

#### SUMMARY / unit `Track 19` (94 records)

- **SUMMARY/I-41.01** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1113| unit=Track 19): [E][lens] Adds the Assumption Excavator lens.
- **SUMMARY/I-41.02** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1114| unit=Track 19): [E][lens] Adds the Implied Completeness lens.
- **SUMMARY/I-41.03** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1115| unit=Track 19): [E][lens] Adds the Ambiguity Mapper lens.
- **SUMMARY/I-41.04** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1116| unit=Track 19): [E][lens] Adds the Gap Analyzer lens.
- **SUMMARY/I-41.05** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1117| unit=Track 19): [E][lens] Adds the Pre-Implementation Architect lens.
- **SUMMARY/I-41.06** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1118| unit=Track 19): [E][lens] Adds the Public-Interface Validator lens.
- **SUMMARY/I-41.07** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1119| unit=Track 19): [E][lens] Adds the Documentation Validator lens.
- **SUMMARY/I-41.08** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1120| unit=Track 19): [E][lens] Adds the Anxiety Reader lens.
- **SUMMARY/I-41.09** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1121| unit=Track 19): [E][timing] Runs lenses on prose or specifications before deterministic artifacts exist.
- **SUMMARY/I-41.10** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1122| unit=Track 19): [E][trust] Treats every LLM-judged finding as advisory only.
- **SUMMARY/I-41.11** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1123| unit=Track 19): [E][grounding] Requires a concrete anchor for every finding.
- **SUMMARY/I-41.12** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1124| unit=Track 19): [E][anchor] Accepts a specification heading as an anchor.
- **SUMMARY/I-41.13** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1125| unit=Track 19): [E][anchor] Accepts a file and line as an anchor.
- **SUMMARY/I-41.14** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1126| unit=Track 19): [E][anchor] Accepts a symbol as an anchor.
- **SUMMARY/I-41.15** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1127| unit=Track 19): [E][anchor] Accepts a quoted phrase as an anchor.
- **SUMMARY/I-41.16** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1128| unit=Track 19): [E][fail-safe] Drops anchorless findings.
- **SUMMARY/I-41.17** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1129| unit=Track 19): [E][invariant] No lens output can accept or block.
- **SUMMARY/I-41.18** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1130| unit=Track 19): [E][budget] Caps each lens at eight findings.
- **SUMMARY/I-41.19** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1131| unit=Track 19): [E][quality] Deduplicates findings.
- **SUMMARY/I-41.20** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1132| unit=Track 19): [E][routing] Scales the number of lenses to artifact risk.
- **SUMMARY/I-41.21** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1133| unit=Track 19): [E][implementation] Reauthors lenses natively rather than using the E-43 MCP-scan path.
- **SUMMARY/I-41.22** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1134| unit=Track 19): [E][integration] Hooks B-40 idea framing and Phase 2B grounding sweep.
- **SUMMARY/I-41.23** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1135| unit=Track 19 SURFACE): [E][CLI] Adds elai critique <spec>.
- **SUMMARY/I-41.24** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1136| unit=Track 19 SURFACE): [E][UI] Shows nonblocking advisory findings in ReasoningDrawer or the framing panel.
- **SUMMARY/I-41.25** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1137| unit=Track 19): [E][human-loop] Routes BLOCKER-CANDIDATE to a human without automatic blocking.
- **SUMMARY/I-41.26** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1138| unit=Track 19): [E][observability] Emits CritiqueLensFinding(AdvisoryFinding) for planted anchored issues.
- **SUMMARY/I-41.27** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1139| unit=Track 19): [E][observability] Emits DroppedNoAnchor for an anchorless result.
- **SUMMARY/I-41.28** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1140| unit=Track 19): [E][test] A contract proves lens runs emit zero gate-verdict rows.
- **SUMMARY/I-42.01** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1147| unit=Track 19): [E][profile] Defines RunProfile Orchestrate.
- **SUMMARY/I-42.02** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1148| unit=Track 19): [E][profile] Defines RunProfile CritiqueOnly.
- **SUMMARY/I-42.03** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1149| unit=Track 19): [E][workflow] In CritiqueOnly the human operator writes the code.
- **SUMMARY/I-42.04** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1150| unit=Track 19): [E][workflow] Spawned agents perform design critique through I-41.
- **SUMMARY/I-42.05** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1151| unit=Track 19): [E][workflow] Spawned agents may run the shipped auditor/off-agent spine at ship time.
- **SUMMARY/I-42.06** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1152| unit=Track 19): [E][security] Structurally disables executor spawn in CritiqueOnly.
- **SUMMARY/I-42.07** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1153| unit=Track 19): [E][security] Structurally disables fixer spawn in CritiqueOnly.
- **SUMMARY/I-42.08** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1154| unit=Track 19): [E][constraint] Does not rely on a prompt instruction to suppress writers.
- **SUMMARY/I-42.09** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1155| unit=Track 19): [E][default] Keeps Orchestrate as the default profile.
- **SUMMARY/I-42.10** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1156| unit=Track 19): [E][invariant] CritiqueOnly must have zero executor- or fixer-attributed file_write rows.
- **SUMMARY/I-42.11** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1157| unit=Track 19): [E][architecture] Uses one engine and one verification spine for both profiles.
- **SUMMARY/I-42.12** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1158| unit=Track 19): [E][scope] Profiles differ only in who writes and whether executors spawn.
- **SUMMARY/I-42.13** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1159| unit=Track 19): [E][configuration] Keeps RunProfile orthogonal to B-22 GateMode.
- **SUMMARY/I-42.14** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1160| unit=Track 19): [E][integration] Uses I-1 to reject executor-side-effect Directives.
- **SUMMARY/I-42.15** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1161| unit=Track 19 SURFACE): [E][CLI] Adds elai run --profile critique-only.
- **SUMMARY/I-42.16** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1162| unit=Track 19 SURFACE): [E][UI] Shows the active profile in the existing StatusBar mode cluster.
- **SUMMARY/I-42.17** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1163| unit=Track 19): [E][test] The same fixture under Orchestrate must produce executor writes, proving the profile is not a no-op.
- **SUMMARY/I-42.18** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1165| unit=Track 19): [E][determinism] Off-agent verdicts are byte-identical across the two profiles.
- **SUMMARY/I-43.01** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1172| unit=Track 19): [E][pilot] Runs the shipped SecCodePLT/SAFEDELTA harness at reduced N.
- **SUMMARY/I-43.02** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1173| unit=Track 19): [E][pilot] Uses 50 to 100 tasks for the reduced pilot.
- **SUMMARY/I-43.03** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1174| unit=Track 19): [E][ablation] Compares gates on against gates off.
- **SUMMARY/I-43.04** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1175| unit=Track 19): [E][scoring] Scores dynamic exploit outcomes.
- **SUMMARY/I-43.05** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1176| unit=Track 19): [E][measurement] Reports residual-insecure-rate delta sign.
- **SUMMARY/I-43.06** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1177| unit=Track 19): [E][measurement] Reports residual-insecure-rate delta magnitude.
- **SUMMARY/I-43.07** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1178| unit=Track 19): [E][measurement] Reports the rate that reaches a reviewer.
- **SUMMARY/I-43.08** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1179| unit=Track 19): [E][anti-gaming] Detects the block-everything trap.
- **SUMMARY/I-43.09** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1180| unit=Track 19): [E][failure] A gate that wins only by blocking nearly everything fails the pilot.
- **SUMMARY/I-43.10** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1181| unit=Track 19): [E][scoper] Runs about 20 identical tasks on local Qwen and cloud Claude.
- **SUMMARY/I-43.11** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1182| unit=Track 19): [E][control] Holds the seed fixed across lanes.
- **SUMMARY/I-43.12** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1183| unit=Track 19): [E][measurement] Measures action-stream divergence on security paths.
- **SUMMARY/I-43.13** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1184| unit=Track 19): [E][claim] Uses divergence to choose honest reproducibility wording.
- **SUMMARY/I-43.14** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1185| unit=Track 19): [E][claim] Scopes cloud claims to trace authentication rather than deterministic replay.
- **SUMMARY/I-43.15** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1186| unit=Track 19): [E][scope] Distinguishes cross-lane divergence from H-23 same-environment determinism.
- **SUMMARY/I-43.16** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1187| unit=Track 19): [E][threshold-policy] Uses no paper constant as the threshold.
- **SUMMARY/I-43.17** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1188| unit=Track 19): [E][artifact] Produces a go/no-go memo and never directly emits an external marketing claim.
- **SUMMARY/I-43.18** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1189| unit=Track 19 SURFACE): [E][CLI] Adds elai bench claim-scope.
- **SUMMARY/I-43.19** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1190| unit=Track 19 SURFACE): [E][UI] Optionally shows a Phase-0 go/no-go cockpit panel; the memo may justify no UI.
- **SUMMARY/I-43.20** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1191| unit=Track 19): [E][observability] Emits ClaimScopeMeasured with pilot delta, reviewer rate, divergence, and scoped wording.
- **SUMMARY/I-43.21** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1193| unit=Track 19): [E][artifact] Names workflow/reasoning/I-43-phase0-memo.md as the memo output.
- **SUMMARY/I-43.22** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1194| unit=Track 19): [E][operations] Ships fixtures now and gates the live leg separately.
- **SUMMARY/I-43.23** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1195| unit=Track 19): [E][wording] Divergence above five percent in the stated observable forces trace-authentication-only wording.
- **SUMMARY/I-43.24** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1197| unit=Track 19): [E][determinism] All claim-scope numbers must recompute from the journal.
- **SUMMARY/I-43.25** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1198| unit=Track 19): [A][scope] The summary calls this a kill-gate and its pilot can fail, while Track 19 says none of its sessions sets the product DONE accept/block verdict. The safe reading is that it gates claim/build commitment, not the off-agent DONE decision.
- **SUMMARY/I-44.01** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1207| unit=Track 19): [E][policy] Defines RetentionPolicy Full.
- **SUMMARY/I-44.02** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1208| unit=Track 19): [E][policy] Defines RetentionPolicy ZeroAfterVerify.
- **SUMMARY/I-44.03** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1209| unit=Track 19): [E][timing] Purges raw inputs only after the verdict and evidence bundle are sealed.
- **SUMMARY/I-44.04** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1210| unit=Track 19): [E][purge] Removes raw prompts.
- **SUMMARY/I-44.05** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1211| unit=Track 19): [E][purge] Removes raw tool input and output.
- **SUMMARY/I-44.06** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1212| unit=Track 19): [E][purge] Removes raw model input and output.
- **SUMMARY/I-44.07** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1213| unit=Track 19): [E][granularity] Purges ordinary raw inputs at file granularity.
- **SUMMARY/I-44.08** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1214| unit=Track 19): [E][database] Purges the whole session database because SSB WAL is append-only.
- **SUMMARY/I-44.09** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1215| unit=Track 19): [E][retention] Keeps only redacted, hash-checkable evidence.
- **SUMMARY/I-44.10** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1216| unit=Track 19): [E][privacy] Requires G-9 to mask retained summaries before purge.
- **SUMMARY/I-44.11** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1217| unit=Track 19): [E][lifecycle] Reuses F-13 archive-then-purge machinery.
- **SUMMARY/I-44.12** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1218| unit=Track 19): [E][invariant] Export grep for raw-input markers must be empty after purge.
- **SUMMARY/I-44.13** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1219| unit=Track 19 SURFACE): [E][invariant] elai replay --verify must still pass for hash-checkable steps.
- **SUMMARY/I-44.14** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1220| unit=Track 19): [E][default] Makes zero-retention opt-in and leaves Full behavior unchanged.
- **SUMMARY/I-44.15** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1221| unit=Track 19): [E][safety] Treats purge as destructive and irreversible.
- **SUMMARY/I-44.16** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1222| unit=Track 19 SURFACE): [E][CLI] Adds elai run --zero-retention.
- **SUMMARY/I-44.17** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1223| unit=Track 19 SURFACE): [E][UI] Shows RetentionPurged rows in the RRA viewer or retention drawer.
- **SUMMARY/I-44.18** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1224| unit=Track 19): [E][observability] Emits RawInputsPurged.
- **SUMMARY/I-44.19** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1225| unit=Track 19): [E][observability] Emits EvidenceRetained with the retained-data blake3.
- **SUMMARY/I-44.20** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1226| unit=Track 19): [E][observability] Emits PurgeVerified.
- **SUMMARY/I-44.21** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1227| unit=Track 19): [E][test] Full and ZeroAfterVerify fixtures differ only in raw-byte presence.
- **SUMMARY/I-44.22** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1228| unit=Track 19): [E][determinism] Evidence and verdict bytes remain identical between retention modes.
- **SUMMARY/I-44.23** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1229| unit=Track 19): [E][scope] Treats self-hosting as covered by I-40 and SOC 2 as out-of-scope process work.

#### SUMMARY / unit `Track 20` (197 records)

- **SUMMARY/I-45.01** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1238| unit=Track 20): [E][integrity] Computes blake3 over canonical objective bytes.
- **SUMMARY/I-45.02** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1239| unit=Track 20): [E][receipt] Stamps the objective hash into every sealed PASS.
- **SUMMARY/I-45.03** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1240| unit=Track 20): [E][artifact] Adds the objective hash to RRA sealing.
- **SUMMARY/I-45.04** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1241| unit=Track 20): [E][artifact] Adds the objective hash to attestation.
- **SUMMARY/I-45.05** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1242| unit=Track 20): [E][staleness] Rechecks objective hash on every receipt read path.
- **SUMMARY/I-45.06** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1243| unit=Track 20): [E][failure] Changing goal text after PASS makes the receipt stale.
- **SUMMARY/I-45.07** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1244| unit=Track 20): [E][failure] Changing acceptance text after PASS makes the receipt stale.
- **SUMMARY/I-45.08** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1245| unit=Track 20): [E][compatibility] Verifies older receipts as explicit LEGACY.
- **SUMMARY/I-45.09** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1246| unit=Track 20): [E][invariant] Never labels a legacy receipt silently fresh.
- **SUMMARY/I-45.10** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1247| unit=Track 20): [E][scope] Complements F-4 artifact hashing by hashing the goal itself.
- **SUMMARY/I-45.11** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1248| unit=Track 20): [E][integration] Extends shipped rra.rs and attestation.rs in place.
- **SUMMARY/I-45.12** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1249| unit=Track 20 SURFACE): [E][UI] Shows a stale-receipt badge in the RRA viewer.
- **SUMMARY/I-45.13** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1250| unit=Track 20): [E][observability] Objective mutation emits ObjectiveSealed(MutationDetected).
- **SUMMARY/I-45.14** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1251| unit=Track 20): [E][failure] Verification hard-fails as ReceiptStale and names both hashes.
- **SUMMARY/I-45.15** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1252| unit=Track 20): [E][test] An unmodified objective verifies PASS.
- **SUMMARY/I-45.16** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1253| unit=Track 20): [E][test] A verifier that passes after objective mutation is the broken arm.
- **SUMMARY/I-46.01** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1260| unit=Track 20): [E][interceptor] Runs before existing bash validation.
- **SUMMARY/I-46.02** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1261| unit=Track 20): [E][redirect] Refuses and redirects bare grep commands.
- **SUMMARY/I-46.03** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1262| unit=Track 20): [E][redirect] Refuses and redirects bare cat commands.
- **SUMMARY/I-46.04** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1263| unit=Track 20): [E][redirect] Refuses and redirects bare find commands.
- **SUMMARY/I-46.05** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1264| unit=Track 20): [E][redirect] Refuses and redirects bare head commands.
- **SUMMARY/I-46.06** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1265| unit=Track 20): [E][redirect] Refuses and redirects bare tail commands.
- **SUMMARY/I-46.07** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1266| unit=Track 20): [E][response] Returns the exact registered-tool substitute call.
- **SUMMARY/I-46.08** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1267| unit=Track 20 SURFACE): [E][fixup] Applies a frozen deterministic table for known-broken command shapes.
- **SUMMARY/I-46.09** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1268| unit=Track 20): [E][behavior] Passes allowed-prefix commands unchanged.
- **SUMMARY/I-46.10** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1269| unit=Track 20): [E][constraint] Uses no LLM in interception or fixup.
- **SUMMARY/I-46.11** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1270| unit=Track 20): [E][safety] Never auto-executes the proposed substitute.
- **SUMMARY/I-46.12** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1271| unit=Track 20): [E][invariant] Interception may narrow but never widen downstream validation.
- **SUMMARY/I-46.13** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1272| unit=Track 20): [E][security] Freezes redirect and fixup tables in R8.
- **SUMMARY/I-46.14** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1273| unit=Track 20 SURFACE): [E][UI] Shows redirect rows in the tool-call drawer.
- **SUMMARY/I-46.15** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1274| unit=Track 20): [E][observability] A grep call emits BashRedirected(Redirected) naming the substitute.
- **SUMMARY/I-46.16** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1275| unit=Track 20 SURFACE): [E][test] A redirected command must spawn no process.
- **SUMMARY/I-46.17** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1276| unit=Track 20): [E][test] Allowed-prefix commands emit no redirect row.
- **SUMMARY/I-46.18** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1277| unit=Track 20): [E][test] A redirect row plus a spawned grep is the broken arm.
- **SUMMARY/I-47.01** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1284| unit=Track 20): [E][input-hardening] Wraps worker diffs entering the auditor as typed untrusted data.
- **SUMMARY/I-47.02** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1285| unit=Track 20): [E][input-hardening] Wraps worker claims as typed untrusted data.
- **SUMMARY/I-47.03** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1286| unit=Track 20): [E][input-hardening] Wraps tool output as typed untrusted data.
- **SUMMARY/I-47.04** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1287| unit=Track 20): [E][framing] Places a standing do-not-follow-embedded-instructions frame above untrusted inputs.
- **SUMMARY/I-47.05** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1289| unit=Track 20): [E][framing] Tells the auditor not to trust worker self-attestation.
- **SUMMARY/I-47.06** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1290| unit=Track 20): [E][dispatch] Mechanically injects a read-before-write rule.
- **SUMMARY/I-47.07** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1291| unit=Track 20): [E][dispatch] Mechanically injects scope discipline.
- **SUMMARY/I-47.08** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1292| unit=Track 20): [E][dispatch] Mechanically injects a no-premature-abstraction rule.
- **SUMMARY/I-47.09** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1293| unit=Track 20): [E][dispatch] Mechanically injects a no-while-I-am-here-refactoring rule.
- **SUMMARY/I-47.10** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1294| unit=Track 20): [E][artifact] Stores rules as a per-role frozen, byte-stable block.
- **SUMMARY/I-47.11** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1295| unit=Track 20): [E][cache] Preserves prefix stability across dispatch calls.
- **SUMMARY/I-47.12** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1296| unit=Track 20): [E][scope] Adds no classifier because E-37 owns data-plane classification.
- **SUMMARY/I-47.13** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1297| unit=Track 20): [E][integration] Coordinates with I-32 at the agent and auditor dispatch prompt builders.
- **SUMMARY/I-47.14** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1298| unit=Track 20): [E][security] Freezes the injection table in R8.
- **SUMMARY/I-47.15** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1299| unit=Track 20 SURFACE): [E][UI] Adds no UI because the behavior is prompt-internal.
- **SUMMARY/I-47.16** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1300| unit=Track 20): [E][observability] Emits PromptHardeningApplied(InputsTagged).
- **SUMMARY/I-47.17** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1301| unit=Track 20): [E][observability] Emits RulesInjected.
- **SUMMARY/I-47.18** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1302| unit=Track 20): [E][test] Two worker dispatches must contain byte-identical role blocks.
- **SUMMARY/I-47.19** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1303| unit=Track 20): [E][fail-closed] Hardening enabled without hardening rows is a failure.
- **SUMMARY/I-48.01** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1309| unit=Track 20): [E][artifact] Moves every dispatch prompt template into a static file.
- **SUMMARY/I-48.02** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1310| unit=Track 20): [E][layout] Stores prompt files under crates/*/prompts.
- **SUMMARY/I-48.03** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1311| unit=Track 20): [E][build] Imports prompt files through include_str.
- **SUMMARY/I-48.04** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1312| unit=Track 20): [E][review] Makes prompts diffable and reviewable.
- **SUMMARY/I-48.05** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1313| unit=Track 20): [E][constraint] Prohibits inline dispatch prompt templates.
- **SUMMARY/I-48.06** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1314| unit=Track 20): [E][migration] Preserves prompt bytes during extraction.
- **SUMMARY/I-48.07** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1315| unit=Track 20): [E][test] Compares assembled executor prompt hashes before and after extraction.
- **SUMMARY/I-48.08** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1316| unit=Track 20): [E][test] Compares assembled auditor prompt hashes before and after extraction.
- **SUMMARY/I-48.09** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1317| unit=Track 20): [E][contract] Bans new long inline prompt literals in dispatch code.
- **SUMMARY/I-48.10** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1318| unit=Track 20): [E][dependency] Makes I-36 compiled prompts and I-47 frozen blocks pinnable.
- **SUMMARY/I-48.11** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1319| unit=Track 20): [E][integration] Leaves D-1 template substitution unchanged.
- **SUMMARY/I-48.12** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1320| unit=Track 20 SURFACE): [E][UI] Adds no UI because it is a build-time refactor.
- **SUMMARY/I-48.13** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1321| unit=Track 20): [E][observability] Adds no new action type.
- **SUMMARY/I-49.01** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1328| unit=Track 20): [E][provider] Adds an AWS Bedrock direct runtime lane.
- **SUMMARY/I-49.02** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1329| unit=Track 20): [E][protocol] Implements SigV4 signing.
- **SUMMARY/I-49.03** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1330| unit=Track 20): [E][protocol] Decodes binary EventStream frames.
- **SUMMARY/I-49.04** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1331| unit=Track 20): [E][provider] Adds an Azure OpenAI Responses-shaped direct runtime lane.
- **SUMMARY/I-49.05** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1332| unit=Track 20): [E][shim] Adds an OpenAI-as-Anthropic cross-protocol normalization pattern.
- **SUMMARY/I-49.06** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1333| unit=Track 20): [E][integration-choice] Extends genai_adapter if its existing seam fits.
- **SUMMARY/I-49.07** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1334| unit=Track 20): [E][provenance] Requires reading the TypeScript competitor source before porting.
- **SUMMARY/I-49.08** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1335| unit=Track 20): [E][freshness] Requires current AWS protocol documentation research before work.
- **SUMMARY/I-49.09** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1336| unit=Track 20): [E][freshness] Requires current Azure protocol documentation research before work.
- **SUMMARY/I-49.10** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1337| unit=Track 20): [E][dependency] Requires anti-slopsquat review for candidate aws-prefixed crates.
- **SUMMARY/I-49.11** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1338| unit=Track 20): [E][dependency] Requires user approval for every new dependency.
- **SUMMARY/I-49.12** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1339| unit=Track 20): [E][test] Ships deterministic recorded-frame replay fixtures immediately.
- **SUMMARY/I-49.13** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1340| unit=Track 20): [E][test] Requires byte-exact frame decoding.
- **SUMMARY/I-49.14** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1341| unit=Track 20): [E][operations] Keeps real Bedrock and Azure calls behind separate live checkboxes.
- **SUMMARY/I-49.15** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1342| unit=Track 20 SURFACE): [E][constraint] Leaves oauth and existing CLI bridges on the keep list.
- **SUMMARY/I-49.16** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1343| unit=Track 20 SURFACE): [E][integration] Registers bedrock.rs, azure_openai.rs, and a shim under the runtime adapter surface.
- **SUMMARY/I-49.17** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1345| unit=Track 20 SURFACE): [E][UI] Lists both backends in the existing RuntimePanel.
- **SUMMARY/I-49.18** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1346| unit=Track 20 SURFACE): [E][CLI] elai runtime list reports both lanes.
- **SUMMARY/I-49.19** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1347| unit=Track 20): [E][failure] A corrupt recorded frame fails with a typed error.
- **SUMMARY/I-49.20** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1348| unit=Track 20): [E][test] SigV4 fixtures reproduce a documented known-good signature.
- **SUMMARY/I-49.21** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1349| unit=Track 20): [E][observability] Reuses runtime/route rows with backend IDs and adds no action type.
- **SUMMARY/I-49.22** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1350| unit=Track 20): [A][integration] Whether the shim extends genai_adapter is explicitly conditional.
- **SUMMARY/I-49.23** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1351| unit=Track 20): [A][precondition] The summary requires a full-form expansion before execution; this summary entry is not sufficient as an implementation spec.
- **SUMMARY/I-50.01** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1359| unit=Track 20): [E][API] Defines an AuthProvider trait.
- **SUMMARY/I-50.02** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1360| unit=Track 20): [E][API] AuthProvider supports acquire.
- **SUMMARY/I-50.03** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1361| unit=Track 20): [E][API] AuthProvider supports refresh.
- **SUMMARY/I-50.04** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1362| unit=Track 20): [E][API] AuthProvider supports revoke.
- **SUMMARY/I-50.05** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1363| unit=Track 20): [E][API] AuthProvider supports status.
- **SUMMARY/I-50.06** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1364| unit=Track 20): [E][lifecycle] A broker owns token lifecycle.
- **SUMMARY/I-50.07** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1365| unit=Track 20): [E][storage] A broker owns token storage.
- **SUMMARY/I-50.08** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1366| unit=Track 20): [E][migration] Moves the existing Anthropic/Claude subscription flow behind the trait.
- **SUMMARY/I-50.09** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1367| unit=Track 20): [E][compatibility] Preserves the existing token-file format byte-for-byte.
- **SUMMARY/I-50.10** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1368| unit=Track 20): [E][compatibility] Preserves refresh semantics.
- **SUMMARY/I-50.11** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1369| unit=Track 20): [E][scope] Adds no new provider login flows in this session.
- **SUMMARY/I-50.12** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1370| unit=Track 20): [E][scope] Adds no new storage backend.
- **SUMMARY/I-50.13** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1371| unit=Track 20): [E][privacy] Never journals token material.
- **SUMMARY/I-50.14** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1372| unit=Track 20): [E][integration] Adds auth_broker.rs and updates adapter call sites.
- **SUMMARY/I-50.15** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1373| unit=Track 20 SURFACE): [E][UI] Shows auth status in the existing RuntimePanel or Settings.
- **SUMMARY/I-50.16** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1374| unit=Track 20): [E][observability] Real requests emit AuthBrokerEvent(TokenAcquired or TokenRefreshed).
- **SUMMARY/I-50.17** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1375| unit=Track 20): [E][recovery] An expired-token fixture refreshes then succeeds.
- **SUMMARY/I-50.18** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1376| unit=Track 20): [E][failure] Refresh failure emits RefreshFailed plus a typed error.
- **SUMMARY/I-50.19** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1377| unit=Track 20): [E][test] Journal search for token material must be empty.
- **SUMMARY/I-50.20** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1378| unit=Track 20): [E][fail-closed] Authentication without a broker row is a failure.
- **SUMMARY/I-51.01** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1385| unit=Track 20): [E][catalog] Defines one models.toml catalog keyed by model ID.
- **SUMMARY/I-51.02** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1386| unit=Track 20 SURFACE): [E][field] Stores context window.
- **SUMMARY/I-51.03** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1387| unit=Track 20): [E][field] Stores input pricing.
- **SUMMARY/I-51.04** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1388| unit=Track 20): [E][field] Stores output pricing.
- **SUMMARY/I-51.05** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1389| unit=Track 20): [E][field] Stores cache pricing.
- **SUMMARY/I-51.06** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1390| unit=Track 20): [E][field] Stores thinking support.
- **SUMMARY/I-51.07** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1391| unit=Track 20): [E][field] Stores effort mapping.
- **SUMMARY/I-51.08** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1392| unit=Track 20): [E][field] Stores default max tokens.
- **SUMMARY/I-51.09** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1393| unit=Track 20): [E][field] Stores VRAM tier.
- **SUMMARY/I-51.10** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1394| unit=Track 20): [E][freshness] Stores a staleness date for every entry.
- **SUMMARY/I-51.11** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1395| unit=Track 20): [E][migration] Converts pricing.rs into a view over the catalog.
- **SUMMARY/I-51.12** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1396| unit=Track 20): [E][migration] Converts duplicated vram_tier.rs modules into catalog views.
- **SUMMARY/I-51.13** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1397| unit=Track 20): [E][compatibility] Preserves existing public APIs.
- **SUMMARY/I-51.14** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1398| unit=Track 20): [E][constraint] Wraps and preserves the measured VRAM-tier plan.
- **SUMMARY/I-51.15** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1399| unit=Track 20): [E][constraint] Performs no live price fetching.
- **SUMMARY/I-51.16** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1400| unit=Track 20): [E][invariant] Is behavior-neutral by construction.
- **SUMMARY/I-51.17** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1401| unit=Track 20): [E][integration] Supplies clean input to cost gates, routing, economics, and runtime defaults.
- **SUMMARY/I-51.18** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1402| unit=Track 20 SURFACE): [E][UI] Adds no UI behind preserved APIs.
- **SUMMARY/I-51.19** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1403| unit=Track 20): [E][test] Cost and tier outputs remain byte-identical before and after migration.
- **SUMMARY/I-51.20** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1404| unit=Track 20): [E][test] A seeded catalog/view divergence fails the drift contract.
- **SUMMARY/I-51.21** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1405| unit=Track 20): [E][test] Migrated modules contain zero price constants outside models.toml.
- **SUMMARY/I-51.22** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1406| unit=Track 20): [E][observability] Adds no action type.
- **SUMMARY/I-51.23** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1407| unit=Track 20): [A][deferred] B-37 default consumption is explicitly a future/deferred leg.
- **SUMMARY/I-52.01** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1413| unit=Track 20): [E][activation] Exposes already-shipped checkpoint machinery to agents.
- **SUMMARY/I-52.02** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1414| unit=Track 20): [E][tool] Adds a save arm.
- **SUMMARY/I-52.03** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1415| unit=Track 20): [E][tool] Adds a rewind arm.
- **SUMMARY/I-52.04** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1416| unit=Track 20): [E][tool] Adds a diff arm.
- **SUMMARY/I-52.05** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1417| unit=Track 20): [E][scope] Restricts checkpoints to the current session.
- **SUMMARY/I-52.06** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1418| unit=Track 20): [E][security] Refuses cross-session rewind.
- **SUMMARY/I-52.07** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1419| unit=Track 20): [E][safety] Refuses rewind when files outside the checkpointed set diverged.
- **SUMMARY/I-52.08** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1420| unit=Track 20): [E][diagnostics] Names each diverged path in the refusal.
- **SUMMARY/I-52.09** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1421| unit=Track 20): [E][coordination] If H-49 ships first, exposes the tool over H-49's store.
- **SUMMARY/I-52.10** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1422| unit=Track 20): [E][invariant] Never creates a second snapshot store.
- **SUMMARY/I-52.11** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1423| unit=Track 20): [E][scope] Supports in-branch, single-agent speculation.
- **SUMMARY/I-52.12** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1424| unit=Track 20): [E][scope] Leaves I-2 as the parallel-branch mechanism.
- **SUMMARY/I-52.13** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1425| unit=Track 20): [E][integration] Uses the three-arm tool catalog and existing checkpoint labels/scopes.
- **SUMMARY/I-52.14** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1426| unit=Track 20 SURFACE): [E][feature-flag] Inherits W-2 posture if the checkpoint substrate remains off.
- **SUMMARY/I-52.15** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1427| unit=Track 20 SURFACE): [E][UI] Shows checkpoint rows in the existing session drawer.
- **SUMMARY/I-52.16** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1428| unit=Track 20): [E][observability] Emits CheckpointToolEvent Saved and Rewound.
- **SUMMARY/I-52.17** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1429| unit=Track 20): [E][test] Rewound bytes must blake3-match checkpoint preimages.
- **SUMMARY/I-52.18** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1430| unit=Track 20): [E][test] Silent overwrite of a diverged file is the broken arm.
- **SUMMARY/I-53.01** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1437| unit=Track 20): [E][recipe] Discovers Justfile targets.
- **SUMMARY/I-53.02** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1438| unit=Track 20): [E][recipe] Discovers Makefile targets.
- **SUMMARY/I-53.03** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1439| unit=Track 20): [E][recipe] Discovers Cargo aliases.
- **SUMMARY/I-53.04** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1440| unit=Track 20): [E][recipe] Runs a selected named recipe.
- **SUMMARY/I-53.05** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1441| unit=Track 20): [E][security] Routes recipe execution through bash_validation.
- **SUMMARY/I-53.06** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1442| unit=Track 20): [E][security] Routes recipe execution through E-30 argv gates.
- **SUMMARY/I-53.07** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1443| unit=Track 20): [E][observability] Journals owner-sanctioned recipe invocations.
- **SUMMARY/I-53.08** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1444| unit=Track 20): [E][archive] Adds a read-only archive reader.
- **SUMMARY/I-53.09** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1445| unit=Track 20): [E][archive] Performs no extraction side effects.
- **SUMMARY/I-53.10** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1446| unit=Track 20): [E][JSON] Adds a capped JSON-tree explorer.
- **SUMMARY/I-53.11** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1447| unit=Track 20): [E][JSON] Marks truncation explicitly.
- **SUMMARY/I-53.12** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1448| unit=Track 20): [E][decline] Declines cron tools.
- **SUMMARY/I-53.13** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1449| unit=Track 20): [E][decline] Declines monitor tools.
- **SUMMARY/I-53.14** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1450| unit=Track 20): [E][decline] Declines background-job tools because orchestrator schedules own the concern.
- **SUMMARY/I-53.15** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1451| unit=Track 20): [E][dependency] Checks existing archive dependencies before proposing a new one.
- **SUMMARY/I-53.16** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1452| unit=Track 20): [E][dependency] Requires anti-slopsquat and approval for a new archive dependency.
- **SUMMARY/I-53.17** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1453| unit=Track 20 SURFACE): [E][UI] Uses the existing tool-call drawer.
- **SUMMARY/I-53.18** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1454| unit=Track 20): [E][test] Recipe listing names fixture Justfile targets.
- **SUMMARY/I-53.19** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1455| unit=Track 20): [E][fail-closed] A recipe run without a bash-validation verdict row fails.
- **SUMMARY/I-53.20** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1456| unit=Track 20): [E][test] Archive-read fixture directory diff remains empty.
- **SUMMARY/I-53.21** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1457| unit=Track 20): [E][observability] Adds no action type.
- **SUMMARY/I-54.01** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1464| unit=Track 20): [E][tool] Adds structural search.
- **SUMMARY/I-54.02** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1465| unit=Track 20): [E][tool] Adds structural-edit preview.
- **SUMMARY/I-54.03** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1466| unit=Track 20): [E][tool] Adds structural-edit apply.
- **SUMMARY/I-54.04** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1467| unit=Track 20): [E][parser] Uses in-tree tree-sitter 0.26.
- **SUMMARY/I-54.05** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1468| unit=Track 20): [E][parser] Reuses the existing nine grammars.
- **SUMMARY/I-54.06** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1469| unit=Track 20): [E][language] Starts with Rust support.
- **SUMMARY/I-54.07** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1470| unit=Track 20): [E][language] Starts with Python support.
- **SUMMARY/I-54.08** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1471| unit=Track 20): [E][language] Starts with TypeScript support.
- **SUMMARY/I-54.09** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1472| unit=Track 20): [E][pattern] Uses tree-sitter query syntax.
- **SUMMARY/I-54.10** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1473| unit=Track 20): [E][constraint] Introduces no new pattern DSL.
- **SUMMARY/I-54.11** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1474| unit=Track 20): [E][workflow] Requires preview before apply.
- **SUMMARY/I-54.12** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1475| unit=Track 20): [E][receipt] Journals the preview match set.
- **SUMMARY/I-54.13** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1476| unit=Track 20): [E][receipt] Journals the preview diff hash.
- **SUMMARY/I-54.14** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1477| unit=Track 20): [E][atomicity] Applies changes atomically per file.
- **SUMMARY/I-54.15** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1478| unit=Track 20): [E][staleness] Refuses apply when source changed after preview.
- **SUMMARY/I-54.16** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1479| unit=Track 20): [E][security] Routes apply through the same PreToolUse edit gates as ordinary Edit.
- **SUMMARY/I-54.17** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1480| unit=Track 20): [E][scope] Supports syntax-only edits.
- **SUMMARY/I-54.18** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1481| unit=Track 20): [E][scope] Leaves semantic rename to the E-52 LSP lane.
- **SUMMARY/I-54.19** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1482| unit=Track 20): [E][dependency] Blocks any ast-grep dependency.
- **SUMMARY/I-54.20** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1483| unit=Track 20): [E][architecture] Exposes a knowledge-crate public parser API rather than pinning a second parser version.
- **SUMMARY/I-54.21** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1485| unit=Track 20 SURFACE): [E][UI] Shows preview and apply rows in the existing tool-call drawer.
- **SUMMARY/I-54.22** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1486| unit=Track 20): [E][test] Post-apply bytes must equal the previewed diff.
- **SUMMARY/I-54.23** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1487| unit=Track 20): [E][test] The edited fixture must compile.
- **SUMMARY/I-54.24** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1488| unit=Track 20): [E][observability] Emits StructuralEditApplied Matched, Previewed, and Applied.
- **SUMMARY/I-54.25** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1489| unit=Track 20): [E][failure] A broken rewrite template fails at preview with ParseFailed and writes zero bytes.
- **SUMMARY/I-54.26** [SRC] (09-I-SUMMARY_INDEX.discovery.md:L1491| unit=Track 20): [E][fail-closed] Apply with no preceding preview row fails.

## Superseded notes (newer-plan / deep-over-summary precedence)

Losing target-state descriptions are retained here with anchors. Explicit CORRECT/conflict/ratified-correction markers and deep-ledger overrides of summary wording win the active target state; the summary atom remains under SUMMARY/*.

Superseded notes count: **30**.

- **SUPERSEDED** I-001/I28-C006 (DEEP-I-001.discovery.md:L346): C/DATA | I-GROUNDING-MAP's dated 221/532 baseline is superseded within this source by the refreshed 606-entry/derived-297 registry observation; both demand reread at implementation. | anchor=I-28:L30-L32, L69-L73; IG:L9-L18
- **SUPERSEDED** I-001/SD-I8-004 (DEEP-I-001.discovery.md:L541): CORRECT the summary's unconditional ReasoningDrawer surface: it is deferred/wire-dark behind GD-4 and W-6-justified for v1. | anchor=I8-E015, I8-C001
- **SUPERSEDED** I-001/SD-I28-001 (DEEP-I-001.discovery.md:L585): CORRECT summary DAG/node attribution to ratified default-off Slice-1 deterministic branch-grain sibling deltas. | anchor=I28-E001–I28-E011, I28-C001
- **SUPERSEDED** I-001/SD-I28-002 (DEEP-I-001.discovery.md:L586): ADD exact task-runner Option<SearchLadderOutcome> pass-through and placement before escalation; CORRECT PhaseComplete. | anchor=I28-E003–I28-E005, I28-C003
- **SUPERSEDED** I-001/SD-I28-004 (DEEP-I-001.discovery.md:L588): CORRECT L2-scored vetoed negative as forbidden; DPO selection is post-FORK-1 and must preserve frozen hard-veto semantics. | anchor=I28-E016–I28-E019, I28-A001, I28-C002
- **SUPERSEDED** I-001/SD-I28-005 (DEEP-I-001.discovery.md:L589): CORRECT chunk demotion/three-abort threshold to fully deferred Slice 3 pending real per-branch provenance/owner; node-grain waits for PUCT. | anchor=I28-E020–I28-E022, I28-A003, I28-C004
- **SUPERSEDED** I-001/SD-I28-006 (DEEP-I-001.discovery.md:L590): CORRECT optional immediate heat UI to no-dashboard unit/deferred heat; ADD I-30 reader-or-wire-dark artifact rule. | anchor=I28-E019, I28-E023, I28-A002, I28-C005
- **SUPERSEDED** I-001/SD-I37-003 (DEEP-I-001.discovery.md:L609): CORRECT deletion state from summary Unverified to source Regressed. | anchor=I37-E017, I37-C001
- **SUPERSEDED** I-001/SD-I48-003 (DEEP-I-001.discovery.md:L620): CORRECT global “every prompt now” scope to orchestrator/providers Tier 1 with Python/dashboard slices deferred. | anchor=I48-E010–I48-E011, I48-A002, I48-C001
- **SUPERSEDED** I-002/I002-I29-013 (DEEP-I-002.discovery.md:L570): [E][conflict-resolution] Character/line conflicts use last-write-wins on a content hash, never LLM judgment. Anchors: lines 22-24, 83.
- **SUPERSEDED** I-003/DI003-I01-031 (DEEP-I-003.discovery.md:L114): [C][tier conflict] The architectural premise says process-separated Supervisor and Supervisor-held credentials, but Revised Scope ships an in-process stub and defers OS separation and the credential vault. Tier-1 claims must follow the latter. Anchors: I-1_directive_supervisor_enforcement_plane.md:8-9,13,33-45,81.
- **SUPERSEDED** I-003/DI003-I32-007 (DEEP-I-003.discovery.md:L460): [E][config] Persona blocks are set max-tokens=0 and replaced by typed constraint and I/O schema. Anchor: I-32_structural_role_scoping.md:19-21.
- **SUPERSEDED** I-004/AC-001 (DEEP-I-004.discovery.md:L511): conflict | I-2’s opening platform description sounds immediate, but revised scope controls: worktrees are Tier 1 and Linux overlay/reflink is Tier 2. | anchor=I2-C001
- **SUPERSEDED** I-004/AC-014 (DEEP-I-004.discovery.md:L524): conflict/scope | I-34’s reasoning-heavy CRANE requirement must be reconciled with the amendment’s HD-3 ban on constrained decoding for reasoning/coder/planner roles. | anchor=I34-C001
- **SUPERSEDED** I-004/SD-I2-001 (DEEP-I-004.discovery.md:L544): **CORRECT** summary implementation scope: overlayfs/btrfs is not a Tier-1 implementation; the worktree pool is the default and Linux CoW is deferred. | 09-I:L123-L134 | anchor=I2-E005–I2-E006, I2-C001
- **SUPERSEDED** I-004/SD-I2-002 (DEEP-I-004.discovery.md:L545): **CORRECT** “adjacent-code conflict resolves deterministically”: an overlapping conflict emits `ConflictEscalated`, halts, and rolls back; only non-overlap follows the stated sequence policy. | 09-I:L140-L141 | anchor=I2-E009–I2-E012, I2-A001
- **SUPERSEDED** I-004/SD-I3-006 (DEEP-I-004.discovery.md:L560): **CORRECT/CLARIFY** concurrency framing: Tier 1 does not deliver concurrent multi-writer support despite the motivating language. | 09-I:L145-L164 | anchor=I3-E003, I3-E018, I3-C001
- **SUPERSEDED** I-004/SD-I21-001 (DEEP-I-004.discovery.md:L577): **CORRECT** summary “vague goal is rejected”: the explicit rejection target is a vague/open-ended **acceptance criterion**, with permitted reference categories. | 09-I:L605-L624 | anchor=I21-E003–I21-E006
- **SUPERSEDED** I-004/SD-I34-001 (DEEP-I-004.discovery.md:L599): **CORRECT** summary claim that implementation adds “the sidecar mask layer”: Python-sidecar CRANE integration is explicitly Tier 2; Tier 1 is provider/Rust runtime plus validation/pinning. | 09-I:L923-L943 | anchor=I34-E014, I34-E018, I34-C001
- **SUPERSEDED** I-004/SD-I35-001 (DEEP-I-004.discovery.md:L612): **CORRECT** summary’s “taint check has run”: source requires an intervening `taint_check PASS`, specifically before a write-class call. | 09-I:L951-L954 | anchor=I35-E005
- **SUPERSEDED** I-004/SD-I35-002 (DEEP-I-004.discovery.md:L613): **CORRECT** summary’s “mutable equal to zero”: the numbered source specifies an R8-frozen rule hash but no `mutable` field/value. | 09-I:L963-L970 | anchor=I35-E009, I35-E026
- **SUPERSEDED** I-006/I16-C001 (DEEP-I-006.discovery.md:L36): C/STATUS | Ratified correction: fp8/fp16 `--kv-cache-dtype`, Int4-class `--turboquant-bits`, `KVCompressor`/`_KVQuantCache`, and the `ELAI_STREAMING_LLM_SINKS` sink mask already ship; I-16 is an evaluation layer, not a new-lever implementation. | anchor=I-16:L4-L12
- **SUPERSEDED** I-006/AC-009 (DEEP-I-006.discovery.md:L505): conflict/threshold | I-26’s 85% boundary is variously `>`, `>=`, and “at.” | anchor=I26-C001
- **SUPERSEDED** I-006/AC-011 (DEEP-I-006.discovery.md:L507): conflict/dependency | I-30 both permits and forbids re-deriving negatives when I-28 is unavailable. | anchor=I30-C001
- **SUPERSEDED** I-006/AC-012 (DEEP-I-006.discovery.md:L508): conflict/schema | I-30 selects `TriLayerVerdict(Promoted)`, which I-17 does not define. | anchor=I30-C002, IX-009
- **SUPERSEDED** I-006/AC-013 (DEEP-I-006.discovery.md:L509): conflict/threshold | I-30 treats cited band numbers as non-target rationale and binding Tier-1 floors simultaneously. | anchor=I30-C003
- **SUPERSEDED** I-006/AC-016 (DEEP-I-006.discovery.md:L512): conflict/threshold | I-43’s no-paper-threshold rule coexists with a hard 5% wording cutoff. | anchor=I43-C001
- **SUPERSEDED** I-006/AC-019 (DEEP-I-006.discovery.md:L515): conflict/security | I-47 says label/preserve self-attestation while its event says `SelfAttestationStripped`; tag/table bytes and enablement semantics are also open. | anchor=I47-C001, I47-A001–I47-A003
- **SUPERSEDED** I-006/AC-022 (DEEP-I-006.discovery.md:L518): conflict/cross-cutting | Seven action families plus reused-row payload changes require serialized live-schema reconciliation; dated baseline arithmetic is not independently safe. | anchor=IX-020–IX-021
- **SUPERSEDED** I-006/AC-023 (DEEP-I-006.discovery.md:L519): conflict/cross-cutting | External numeric thresholds in I-30/I-43 conflict with the chunk-wide local-measurement doctrine. | anchor=IX-028

## Closing audit block

- **Ledgers merged:** 6/6 deep (`DEEP-I-001` .. `DEEP-I-006`) + summary (`09-I-SUMMARY_INDEX.discovery.md`).
- **Per-ledger atom counts (all distinct record kinds extracted):**
  - I-001: **435** (AC=43, CON=15, INF=2, IX=20, SD=68, SRC=287)
  - I-002: **654** (AC=26, CON=19, INF=4, IX=24, SD=58, SRC=523)
  - I-003: **415** (AC=32, CON=12, INF=7, IX=12, SD=36, SRC=316)
  - I-004: **469** (AC=69, CON=5, INF=6, IX=30, SD=64, SRC=295)
  - I-005: **380** (AC=20, CON=18, INF=9, SD=9, SRC=324)
  - I-006: **454** (AC=52, CON=16, IX=28, SD=66, SRC=292)
  - SUMMARY: **1022** (session inventory + CROSS)
- **Aggregate distinct record count:** **3829**
- **Superseded notes count:** **30**
- **Surface-tagged atoms:** **445**
- **Zero-omission statement:** Every distinct atomic ID extracted from the six deep ledgers (primary feature/behavior atoms, cross-source interactions IX-*, summary deltas SD-*, ambiguity/conflict registers AC/AMB/CON, and inferences INF where present) and every summary inventory atom (`I-N.xx` plus `I.CROSS.xx`) is present above under a namespaced ID with its source ledger file and line anchor. Relational records that only re-point at already-counted atoms keep separate IX/SD/AC IDs and are not collapsed into prose. No atomic record was compressed into prose-only form or dropped as minor. Deep-ledger CORRECT/conflict rows supersede summary target-state wording; losing summary descriptions remain under SUMMARY/* and are listed in Superseded notes where a corrective marker was detected.
- **Duplicate-ID check:** native IDs are unique within each ledger namespace; aggregate uses `I-00N/` or `SUMMARY/` prefixes so cross-ledger local ID reuse cannot collide.
- **Output path:** `docs/thefinalglance/feature-discovery/aggregates/I-AGGREGATE.md`

---

*Aggregator: I-family deep feature discovery. Inputs read-only. Single write: this file.*
