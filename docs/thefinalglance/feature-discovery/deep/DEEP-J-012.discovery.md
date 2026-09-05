# DEEP-J-012 exhaustive feature discovery

## Scope, integrity, and method

- Processed manifest: `docs/thefinalglance/feature-discovery/DEEP-SOURCE-MANIFEST.md:L1054-L1069`, heading `### DEEP-J-012`; the next chunk starts at L1071.
- Declared chunk: family J, **10 files**, **1,226 logical lines**, **151,391 bytes**, all ten classified `numbered/index original`; target status **under 1,800**.
- Mechanical verification: fresh `sha256sum`, `wc -l`, and byte-size checks matched every manifest row and both aggregates exactly. Missing paths: **0**; unexpected paths: **0**; duplicate paths: **0**; SHA mismatches: **0**; line mismatches: **0**; byte mismatches: **0**.
- Read boundary: all ten listed originals were read completely, first through last logical line. No original assigned to another deep chunk was opened. `10-J-SUMMARY_INDEX.discovery.md` was consulted only as an orientation and delta-comparison artifact; no source atom below is derived solely from it.
- Anchor notation refers to the manifest-verified source images: `J-9:Lx-Ly`, `J-18:Lx-Ly`, `J-49:Lx-Ly`, `J-53:Lx-Ly`, `J-59:Lx-Ly`, `J-68:Lx-Ly`, `J-71:Lx-Ly`, `J-84:Lx-Ly`, `J-85:Lx-Ly`, and `J-103:Lx-Ly`.
- Repeated premise, revised-scope, wiring, grounding, composition, and acceptance language is collapsed into one atom with every load-bearing anchor. Interaction, ambiguity/conflict, and summary-delta records reference atoms and are not counted again as features.
- Classification: `E` = explicit source fact/requirement; `I` = bounded inference; `A` = unresolved ambiguity/choice; `C` = internal conflict or summary correction that must not be silently flattened. Domains: `STATUS`, `CAP`, `SURFACE`, `FLOW`, `STATE`, `DATA`, `CFG`, `INT`, `EVENT`, `SEC`, `PRIV`, `COMP`, `FAIL`, `RECOVERY`, `RETRY`, `DET`, `IDEM`, `CONC`, `OBS`, `TEST`, `LIMIT`, `DEP`, `DEFER`, `EVIDENCE`.

### Exact processed manifest and verification

| Manifest class | Lines | Bytes | SHA-256 | Exact absolute path | Verification |
|---|---:|---:|---|---|---|
| numbered/index original | 240 | 16,822 | a7eb946cef30e208f2c3a039d9b5126910c87d75de1fb7e710d8d6fea465bf8c | docs/thefinalglance/J-plan/J-9_hardware_detection_model_catalog.md | exact |
| numbered/index original | 70 | 12,583 | edc4539355c4d97c9e2b2ff56b707976c063ad06c3c3467c405cd9479a1b149e | docs/thefinalglance/J-plan/J-18_p1_retrieval_hardening.md | exact |
| numbered/index original | 186 | 13,600 | a124503bf301609fe3a97f57e3f41c39677453a554a7f4885273fc798940c780 | docs/thefinalglance/J-plan/J-49_multi_seat_admission_quota.md | exact |
| numbered/index original | 87 | 18,043 | c3b036aee24a606d6d7debf427fa1596e6a54b38569d338493945a3c6df1b8a2 | docs/thefinalglance/J-plan/J-53_openai_compat_tool_input_repair.md | exact |
| numbered/index original | 129 | 17,832 | da150da29827859d0eb72871ce79575060e22cf882f4fb0b54ee08e09dc3696d | docs/thefinalglance/J-plan/J-59_per_seat_memory_trust_isolation.md | exact |
| numbered/index original | 74 | 10,068 | 325e2e553ed4388a27f0d913c4f4b8fc695fb71e705c6602f11c46858ff9f962 | docs/thefinalglance/J-plan/J-68_self_code_formal_verification.md | exact |
| numbered/index original | 62 | 8,416 | b27555b4a4902d7f9d804fca60f941add6a7e0d5cf01dcf6aeb5470d55615443 | docs/thefinalglance/J-plan/J-71_cedar_policy_as_code_spike.md | exact |
| numbered/index original | 116 | 14,700 | bdaa479f33571f8c1ca52a4bc469ff61652565b6404a8886c9b847a7c0b59e5b | docs/thefinalglance/J-plan/J-84_grcv3_reasoning_effort_classifier.md | exact |
| numbered/index original | 124 | 17,211 | 72afcf551b143da4bc4fbad8dc69360863db92a5332d7b257199983d5a894b07 | docs/thefinalglance/J-plan/J-85_grcv3_overthinking_guard.md | exact |
| numbered/index original | 138 | 22,116 | 35c515d2fdde9be20d8191529f373618ac784359de6c3c6a7a74fe17c3fe364a | docs/thefinalglance/J-plan/J-103_read_access_ladder_interceptor.md | exact |
| **Aggregate** | **1,226** | **151,391** | — | **10 exact paths** | **exact** |

## Per-file atomic ledgers

### J-9 — hardware detection + per-lane model eligibility catalog

Source: `docs/thefinalglance/J-plan/J-9_hardware_detection_model_catalog.md` (240/240 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| J9-E001 | E/STATUS | J-9 is PLANNED Wave 2 BUILD wrapping shipped probes; Tier-1 SHIP NOW is probe→catalog builder + SSB emit (model-free); Tier-2 is deferred CLI/panel. | J-9:L1-L4, L21-L21 |
| J9-E002 | E/CAP | Builds `HardwareModelCatalog`: probe GPU/RAM/disk/platform once, resolve data-driven per-lane eligibility (coding LLM / vision VLM / OCR / embedding+reranker) with per-quantization VRAM-fit estimates, emit journaled SSB rows. | J-9:L6-L14, L70-L90 |
| J9-E003 | E/STATUS | Existing substrate HW-1..HW-9 (GpuInfo, SystemProbeResult, classify_vram_tier, VramTier, ModelCapabilityProfile, can_fit_llm, research_config_for_tier, BackendMode) is SHIPPED internal routing input; J-9 adds zero probing code. | J-9:L6-L10, L41-L52, L109-L113 |
| J9-E004 | E/LIMIT | CATALOG DATA ONLY: no routing override, no auto-install, no model-pin flip, no CLAUDE.md/allowlist change (JC-10, DC-3). | J-9:L16-L18, L123-L124, L165-L166 |
| J9-E005 | E/CFG | HW-14 policy shift: Gemma 4 / Qwen3-32B rejections were user-hardware-specific; catalog lists them TIER-CONDITIONALLY with prior-rejection residue note verbatim. | J-9:L14-L16, L82-L84, L163-L164 |
| J9-E006 | E/EVIDENCE | Odysseus hwfit is structural prior art only (lane/fit-score structure); ELAI derives own weights from measured fixtures; never copy odysseus weights (GD-5). | J-9:L17-L19, L55-L56, L88-L90, L171-L171 |
| J9-E007 | E/DEP | No pre-reqs beyond shipped substrate (re-grep W-7). Vision/OCR lane seed from J-8 vision_ladder.toml when J-8 ships; J-9 ships first with coding/embedding live and vision seeded from VR-13 marked `[DESIGN; J-8 eval pending]`. | J-9:L22-L26, L78-L81, L231-L232 |
| J9-E008 | E/CONC | Parallel-safe with J-2..J-7 (disjoint). Coordinate with J-8 on vision_ladder.toml ownership. | J-9:L26-L26 |
| J9-E009 | E/DATA | Catalog data model in `crates/orchestrator/src/hardware_catalog.rs`: `ModelCatalogEntry {lane (Coding\|Vision\|Ocr\|Embedding), model_id, backend: BackendMode, footprint: Vec<{quant, min_vram_mb}>, platform_tags, source_tag ([MEASURED]\|[DESIGN]), residue_note: Option<String>}`, `CatalogLane`, `HardwareModelCatalog {probe, lanes}`. Entries DESERIALIZED from toml — zero model constants in code. | J-9:L70-L75, L120-L122 |
| J9-E010 | E/DATA | Seed file `data/hardware_catalog/model_catalog.toml`: coding from E-31 matrix (Qwen2.5-Coder 3B/7B/32B per tier + cloud); vision from VR-13 (Phi-4-MM, Qwen2.5-VL, Gemma-3-27B, InternVL3, FastVLM apple-silicon per JC-9) all DESIGN; OCR (PaddleOCR-VL canonical per HW-13; dots.ocr gated JC-7); embedding (GraphCodeBERT/Granite shipped; Qwen3-VL-Embedding gated JC-8). | J-9:L76-L81, L218-L219 |
| J9-E011 | E/DATA | Residue wording for Gemma-4/Qwen3-32B: "rejected 2026-05 on a 12GB box for context-extension limits; tier-conditional since 2026-06-11; unmeasured on this hardware". | J-9:L82-L84 |
| J9-E012 | E/FLOW | `HardwareModelCatalog::build(probe: &SystemProbeResult)`: classify_vram_tier for tier, can_fit_llm per footprint row, platform_tags vs detected platform, backend availability vs HW-9. Absent backend listed with `eligible=false`, reason=`backend_missing`. | J-9:L85-L90, L167-L167 |
| J9-E013 | E/OBS | Fit-score uses odysseus STRUCTURE (fit × footprint headroom) with ELAI-derived weights documented in decisions doc. | J-9:L88-L90, L98-L99 |
| J9-E014 | E/EVENT | +HardwareModelCatalogEvent (Pattern-2 single-variant; Kind: CatalogBuilt / TierAssigned / EligibilityChecked / LaneSeeded), baseline+1 from LIVE 226/537; re-read at edit (W-9). | J-9:L91-L94, L58-L58, L169-L169 |
| J9-E015 | E/INT | Emit CatalogBuilt at the existing SystemProbe startup call path (exact site re-grepped at edit, W-7). Duplicate CatalogBuilt rows per boot are a FAIL. | J-9:L93-L94, L115-L117, L229-L230 |
| J9-E016 | E/SURFACE | Tier-2 DEFER: CLI subcommand `elai hardware-catalog [--lane X] [--json]` + `HardwareModelCatalogPanel` dashboard tile (frontend_pair); deferred per DC-2, registered not dropped. | J-9:L95-L97, L107-L107, L128-L131, L172-L172 |
| J9-E017 | E/DATA | Decision doc `workflow/reasoning/J-9-decisions.md` (lane seeds + source tags, fit-score weights, HW-14 residue wording, Tier-2 defer record). | J-9:L98-L99, L209-L209 |
| J9-E018 | E/OBS | Trigger: `elai run` (startup probe) + fixture probes (8/12/16/24/48GB); expect CatalogBuilt with detected tier + per-lane eligible counts + fit-scored ordering, recomputable under ELAI_SEED (GD-2). | J-9:L132-L137, L185-L186 |
| J9-E019 | E/FAIL | Negative: zero-VRAM probe → cloud-lane-only eligibility + honest residue (never empty/silent); entry claiming eligibility above fixture VRAM FAILS; grep proves zero routing-path symbols consume catalog. | J-9:L138-L144, L161-L162 |
| J9-E020 | E/DET | On author's 12GB box coding lane resolves micro_7b-class eligible and large_32b ineligible with reason=vram; Gemma-4 appears ONLY on ≥24GB fixtures WITH residue note. | J-9:L138-L140 |
| J9-E021 | E/LIMIT | Stub bans: no re-impl of probe/fit primitives; no model names/footprints as code constants; no hidden ineligibility; no external perf figure as measured; no .unwrap/.expect; blake3 only; no new deps. | J-9:L119-L127, L150-L153, L170-L171 |
| J9-E022 | E/TEST | Contract `j9_hardware_catalog_contract.rs` (~160 LOC): five fixture eligibility; zero-VRAM cloud-only; over-VRAM eligible=true FAILS; Gemma-4 absent@12GB present@48GB+residue; backend_missing listed; corrupt/missing toml degrades to probe-only CatalogBuilt with lanes_unavailable; routing-isolation compile/grep assert. | J-9:L191-L201 |
| J9-E023 | E/CFG | Tier-1 ships default-ON on startup path (no enable flag); Tier-2 is registered deferred checkbox, not a dark flag (W-2). | J-9:L177-L178 |
| J9-E024 | E/LIMIT | Multi-GPU and CPU-offload untested residue: catalog reads primary GpuInfo; named in emitted residue note (GD-5). | J-9:L224-L225 |
| J9-E025 | E/LIMIT | VRAM footprints are quantization-dependent ESTIMATES; real headroom depends on KV cache + context; labelled as estimates (GD-5). | J-9:L226-L228, L187-L188 |
| J9-E026 | E/DEFER | Later user-gated session may promote catalog verdicts into routing defaults; explicitly out of scope (JC-10/DC-3). J-8 eval upgrades vision/OCR from DESIGN to MEASURED. | J-9:L234-L238 |
| J9-E027 | E/INT | Consumes: system_probe SystemProbeResult/GpuInfo/classify_vram_tier; vram_tier VramTier; context_budget ModelCapabilityProfile; agent_pool can_fit_llm; runtime_builder BackendMode; vision_ladder.toml when J-8 ships. | J-9:L108-L114 |
| J9-E028 | E/TEST | Deterministic checks: wiring_contract_check; model names must be 0 in .rs (live in toml); classify_vram_tier\|can_fit_llm consumed; routing paths must not reference hardware_catalog. | J-9:L146-L156 |
| J9-A001 | A/INT | Startup probe call locations may be plural (CLI vs dashboard boot); single canonical emit site is decided at edit. | J-9:L229-L230 |
| J9-A002 | A/DATA | Exact fit-score weight values and derivation live only in the decisions doc (not fixed in the plan body). | J-9:L88-L90, L98-L99 |
| J9-A003 | A/CFG | Exact platform_tag vocabulary and multi-GPU selection rule beyond primary GpuInfo are unspecified. | J-9:L87-L88, L224-L225 |

Category coverage: capabilities/CLI/panel ✓; workflows/states ✓; schemas/data/lifecycle ✓; flags/defaults ✓ (default-ON Tier-1, Tier-2 deferred); integrations/events ✓; security (informational isolation) ✓; failures/degradation ✓; determinism ✓; observability/evidence ✓; tests/done gates ✓; constraints/non-goals ✓; dependencies/status/deferred ✓. No formal privacy policy or retry API beyond recompute-from-seed.

### J-18 — P1 retrieval hardening

Source: `docs/thefinalglance/J-plan/J-18_p1_retrieval_hardening.md` (70/70 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| J18-E001 | E/STATUS | J-18 is Track J BUILD+SWAP, lever CANT-MISREAD, grounding STRONG, effort M, Wave 3, depends none (extends E-17, rides knowledge crate). | J-18:L1-L3 |
| J18-E002 | E/CAP | Three independently shippable legs: (1) CodeRankEmbed 768d embedding swap; (2) LSP fourth retrieval channel; (3) agentic-grep repair pass. | J-18:L5-L10 |
| J18-E003 | E/DATA | Embedding swap: replace GraphCodeBERT-era 768d with CodeRankEmbed INT8 ONNX; VERIFIED hidden_size=768 exact match → rebuild-only (no usearch geometry change); index_version bump + re-embed required. | J-18:L8-L8, L14-L14, L21-L21, L42-L42 |
| J18-E004 | E/SEC | Export ONNX ourselves from nomic-ai/CodeRankEmbed via optimum; do NOT depend on unofficial community quant mrsladoje/CodeRankEmbed-onnx-int8 (anti-slopsquat). | J-18:L8-L8, L40-L40, L52-L52 |
| J18-E005 | E/CFG | CodeRankEmbed query-prefix quirk is required and part of byte-stable input; must assert prefix in contract test. max_trained_positions=2048 vs claimed 8192 — verify >2048 chunking before trusting long-function embeddings. | J-18:L8-L8, L63-L65 |
| J18-E006 | E/EVIDENCE | CodeRankEmbed: MIT, 137M, CoIR NDCG@10=60.1 is literature; A/B must be ELAI-MEASURED, never the literature 60.1 as acceptance. | J-18:L8-L8, L49-L49, L52-L52 |
| J18-E007 | E/CAP | LSP fourth channel: go-to-definition / find-references / cross-file type resolution alongside tree-sitter/BM25/vector. Prefer driving rust-analyzer / pyright / typescript-language-server over LSP stdio from Rust; Serena MCP is alternative. | J-18:L9-L9, L22-L22 |
| J18-E008 | E/FAIL | Tree-sitter STAYS always-available incremental fallback; LSP init can fail → DEGRADED not crash; do NOT replace tree-sitter. | J-18:L9-L9, L11-L11, L41-L41, L49-L49 |
| J18-E009 | E/DEP | LSP channel narrows J-16 blast-radius false-negatives (macros, trait impls, dynamic dispatch) and reuses E-52 rust-analyzer subprocess harness (compose, not re-author). | J-18:L9-L9, L17-L18, L34-L34, L60-L60 |
| J18-E010 | E/CAP | Agentic-grep repair: post-retrieval verify/extend when static-index confidence is low; "index-first, grep-to-repair" (CORTEXA/DCI), NOT index-OR-grep. Additive, behind a feature flag. | J-18:L10-L10, L23-L23, L31-L31 |
| J18-E011 | E/LIMIT | Scope boundary: narrow post-hoc CONFIRM of a single low-confidence hit. Broad mid-task hybrid cascade tool + SpIDER graph-neighbor fusion + context-sufficiency gate is J-38. J-18 ships first; J-38 exposes J-18's hardened cascade. | J-18:L10-L10, L60-L60, L67-L68 |
| J18-E012 | E/LIMIT | EXCLUDED: replace tree-sitter; CodeXEmbed-7B / nomic-embed-code / voyage-code-3 (>500M or cloud); jina-code-embeddings (896d mismatch + CC-BY-NC); MCTS graph-traversal RANGER replacement. | J-18:L11-L11 |
| J18-E013 | E/EVENT | +RetrievalAugmentEvent (Pattern-2; Kind: EmbeddingModelSwapped / LspChannelQueried / AgenticGrepRepaired / CommunityPriorApplied); baseline+1 from LIVE 231/542 (grep 2026-06-12); re-read at edit. PREFER-REUSE QueryTrace/knowledge channel-source field if present (net 0). | J-18:L24-L24, L56-L56 |
| J18-E014 | E/SURFACE | frontend_pair: retrieval-channel breakdown on existing KnowledgeDrawer (B-29) or W-6-justified absence; cross_lang pair none (Rust-internal). | J-18:L25-L25, L45-L45 |
| J18-E015 | E/INT | Integration sites: crates/knowledge/src/vector (~80 LOC + onnx asset); new lsp_channel (~200 LOC); agentic-grep (~100 LOC behind flag). | J-18:L21-L23 |
| J18-E016 | E/INT | Production reachability: real `elai` retrieval query (search/task → HYBRID cascade → RRF). Test/contracts caller never satisfies R7. | J-18:L36-L38 |
| J18-E017 | E/LIMIT | No new default-OFF ELAI_*_ENABLE without wire-dark-allowlist + promotion plan (R9/W-2). Channel with no hit contributes nothing to RRF, not rank-k filler (B6 noise critique). | J-18:L43-L44 |
| J18-E018 | E/OBS | Trigger: symbol reachable only via macro-expanded/trait-object reference with LSP on. Expect EmbeddingModelSwapped on rebuild; LspChannelQueried resolving edge; symbol in RRF top-k. | J-18:L46-L48 |
| J18-E019 | E/FAIL | Negative: LSP init failure DEGRADES cascade intact; dim-mismatch FAILS migration (never silent corrupt); agentic-grep off returns unrepaired index result (flag observable). | J-18:L49-L49 |
| J18-E020 | E/CFG | LSP channel must be SELECTIVE (only when tree-sitter confidence low / symbol-resolution queries), not every query (latency critique B7). | J-18:L64-L64 |
| J18-E021 | E/DEP | Composes E-17 (next retrieval-quality increment), E-52 LSP harness, B-29/J-16, RRF k=60 / SearchMode cascade, J-38 mid-task loop substrate. Restates none. | J-18:L15-L18, L59-L60 |
| J18-E022 | E/EVIDENCE | HARDEN-not-replace reinforced by SweRank + SpIDER (hybrid beats pure agentic); SWE-Explore agentic tier is J-38's gap, not J-18's. | J-18:L57-L57 |
| J18-E023 | E/DEFER | Louvain community priors for PageRank harden (ocg crate) only if pursued; anti-slopsquat at edit. | J-18:L11-L11, L55-L55 |
| J18-A001 | A/CFG | Exact agentic-grep feature-flag name, allowlist row, and promotion plan are not named in the plan body. | J-18:L10-L10, L23-L23, L43-L43 |
| J18-A002 | A/CFG | Low-confidence threshold that triggers agentic-grep is unspecified. | J-18:L10-L10, L23-L23 |
| J18-A003 | A/SURFACE | KnowledgeDrawer channel breakdown vs W-6 absence is decide-at-edit. | J-18:L25-L25, L45-L45 |
| J18-A004 | A/EVENT | Prefer-reuse vs new RetrievalAugmentEvent decided at edit. | J-18:L24-L24 |
| J18-A005 | A/CFG | Chunking A/B outcome for >2048-token functions (window size vs model) is an open first checkbox. | J-18:L63-L63 |

Category coverage: capabilities ✓; workflows ✓; data/index migration ✓; flags (partial — names open) ✓; integrations/events ✓; failures/degrade ✓; determinism (query-prefix, index_version) ✓; observability ✓; tests/acceptance ✓; constraints/exclusions ✓; deps/deferred ✓. No privacy policy; no dedicated CLI verb beyond real retrieval path.

### J-49 — multi-seat admission + per-seat quota / fair-share / spend isolation

Source: `docs/thefinalglance/J-plan/J-49_multi_seat_admission_quota.md` (186/186 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| J49-E001 | E/STATUS | J-49 is PLANNED WAVE LSW-2, effort M-L, EXTENSION+BUILD; activates dormant UsageTurnEvent.identity; depends J-48. | J-49:L1-L3, L23-L26 |
| J49-E002 | E/CAP | Multi-seat leg: SeatRegistry (data-driven), per-seat admission pre-filter in FRONT of J-48 LocalServingRouter::decide (fair-share + concurrency + spend ceiling), per-seat aggregation keyed off identity. | J-49:L8-L13, L67-L77 |
| J49-E003 | E/CFG | Present seats.toml = multi-seat active; absent = single implicit "local" seat with today's behavior. NO new ELAI_*_ENABLE graveyard (W-2). | J-49:L9-L10, L68-L70, L100-L100, L129-L129 |
| J49-E004 | E/LIMIT | HARD BOUNDARY: NO synthetic credit/billing display unit (stays REJECTED — raw tokens + USD only); NO OpenRouter-style dynamic registry (I-51 static models.toml); fair-share is weighted token-bucket, NOT a new scheduler. | J-49:L13-L16, L52-L55, L96-L99, L127-L128 |
| J49-E005 | E/DEP | GD-1: serving decision is J-48; local/cloud spend caps are CloudModeRouter (C-5); usage capture + identity + repricing is J-28; read-side window is J-29. | J-49:L16-L18, L88-L91, L132-L132 |
| J49-E006 | E/EVIDENCE | Goose ledger REJECTED per-user multi-tenant accounting with revisit trigger "revisit only if multi-tenant"; user multi-seat goal 2026-06-18 IS that trigger. Credit-unit rejection STANDS. | J-49:L5-L8, L37-L41, L52-L55 |
| J49-E007 | E/DATA | SeatRegistry in `seat_registry.rs`: seat-id → {weight, per-seat concurrency cap, per-seat daily/monthly USD cap}. Source seats.toml (mirrors I-51 models.toml shape), inventory-validated. | J-49:L67-L70 |
| J49-E008 | E/FLOW | Weighted token-bucket fair-share admission in RuntimeOrchestrator::try_complete BEFORE J-48 decide(); identity over cap or bucket-empty → AdmitQueued/Deny for THAT seat only (never starves others). | J-49:L71-L74, L92-L94, L106-L111 |
| J49-E009 | E/SEC | Per-seat spend ceiling reuses CloudModeRouter cap check keyed by identity (NOT a new cap engine). | J-49:L73-L74, L98-L98 |
| J49-E010 | E/OBS | Per-seat aggregation rolls up UsageTurnEvent by identity (pure read, DSR/G-5 derived not recalled). Extend J-29 UsageWindow with per-seat breakdown; no new dashboard command. | J-49:L75-L77, L103-L105 |
| J49-E011 | E/EVENT | Emit SeatAdmissionEvent OR prefer-reuse J-48 decision row with identity discriminator (decide at edit; net 0..+1 SSB). Row must carry seat-id + admit/deny + reason. | J-49:L78-L79, L110-L111, L171-L172 |
| J49-E012 | E/SEC | NO key/PII in SSB payload; seat-id is opaque handle; never carries seat secrets. | J-49:L101-L101 |
| J49-E013 | E/SURFACE | frontend_pair N/A with justification: per-seat view is breakdown of already-shipped J-29 UsageWindow; no new Tauri command. | J-49:L103-L105, L142-L142 |
| J49-E014 | E/INT | Production reachability: try_complete admission before every completion. Test caller never satisfies R7. | J-49:L92-L94, L131-L131 |
| J49-E015 | E/TEST | Contract j49_multi_seat_contract.rs: (a) absent seats.toml → single local; (b) equal-weight fair-share under one-seat flood; (c) over USD cap denied while others serve; (d) rollup correct identity; (e) NO synthetic credit; (f) corrupt seats.toml degrades to single-seat, never panic. | J-49:L148-L153 |
| J49-E016 | E/FAIL | Negative contract: one seat's flood causing another seat's deny (fair-share violated) OR any synthetic credit unit in rollup is contract failure. | J-49:L113-L114, L130-L130 |
| J49-E017 | E/CFG | GD-4: per-seat caps default permissive; deny path activates only after FP/friction observed across ≥3 real multi-seat sessions. GD-5: fair-share weights unmeasured until real two-seat workload; ship equal weights. | J-49:L144-L146, L173-L174 |
| J49-E018 | E/DEFER | Seat identity provenance (local/CLI vs multi-client) defers to J-2 elai-as-MCP-server; J-49 takes identity as input and does not invent auth. | J-49:L175-L178, L180-L183 |
| J49-E019 | E/CONC | Live multi-seat smoke (two concurrent identities) is RunPod/remote-sidecar-gated (W-10); admitter + rollup run in-process at gate time. Parallel-safe with none in this wave (depends J-48; same try_complete seam). | J-49:L20-L22, L27-L27, L140-L141 |
| J49-E020 | E/DATA | Decision doc workflow/reasoning/J-49-decisions.md. Files: CREATE seat_registry.rs; MODIFY runtime_orchestrator try_complete; MODIFY J-29 UsageWindow; CREATE contract + decisions. | J-49:L80-L80, L155-L161 |
| J49-E021 | E/LIMIT | No .unwrap/.expect on admission prod path; no new dependency (existing toml dep). | J-49:L102-L102, L134-L134 |
| J49-A001 | A/EVENT | SeatAdmissionEvent vs J-48 decision-row reuse is open at edit. | J-49:L78-L79, L171-L172 |
| J49-A002 | A/CFG | Concrete default concurrency/USD cap values and token-bucket rate/burst constants are "named consts" but not numeric in the plan. | J-49:L63-L64, L67-L70 |
| J49-A003 | A/SEC | Exact seat-id format and how CLI/local path populates identity before J-2 are unspecified (deferred provenance). | J-49:L175-L178 |

Category coverage: capabilities ✓; workflows/admission ✓; data/config seats.toml ✓; surfaces (J-29 breakdown, no new UI) ✓; events ✓; security/PII ✓; failures/fair-share ✓; concurrency ✓; tests ✓; deps/deferred ✓. Privacy as isolation of spend identity; no memory isolation here (that is J-59).

### J-53 — OpenAI-compat tool-input repair

Source: `docs/thefinalglance/J-plan/J-53_openai_compat_tool_input_repair.md` (87/87 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| J53-E001 | E/STATUS | J-53 is PLANNED WAVE TIR-1 BUILD-on-shipped; effort M; parallel-safe with J-50/J-51/J-52; composes B-37 / E-18 / E-35 / C-26 / J-51. | J-53:L1-L5 |
| J53-E002 | E/CAP | Deterministic validate-then-repair shim at unmasked OpenAI-compat `parse_tool_arguments` seam ELAI cannot grammar-mask (cloud today; any local OpenAI-compat backend later). | J-53:L1-L1, L6-L6, L14-L15 |
| J53-E003 | E/LIMIT | Masked local SIDECAR path is OUT: Anthropic `/v1/messages` tool_use.input never reaches parse_tool_arguments AND XGrammar prevents malformed shapes. J-53 fires on parse_tool_arguments ONLY. | J-53:L6-L6, L14-L15 |
| J53-E004 | E/FAIL | Today's anti-pattern: `serde_json::from_str(arguments).unwrap_or_else(|_| json!({ "raw": arguments }))` at openai_compat.rs:844-845; test :1096 LOCKS the anti-pattern. J-53 REMOVES silent raw blob. | J-53:L6-L6, L12-L12, L20-L20, L24-L24, L53-L53 |
| J53-E005 | E/CAP | NEW `crates/providers/src/tool_input_repair.rs` ~180 LOC: `repair_tool_input(tool_name, raw_args, input_schema) -> RepairOutcome` = ShipUnchanged \| Repaired{kinds} \| Unrepairable{model_readable_msg}. | J-53:L10-L10, L38-L39 |
| J53-E006 | E/FLOW | Validate-then-repair NEVER preprocess-then-validate (greedy preprocess corrupts valid-but-unusual content, e.g. writeFile body that is JSON-shaped). | J-53:L10-L10, L21-L21, L54-L54 |
| J53-E007 | E/FLOW | Order: (a) parse as-is; if parses AND matches input_schema types → ShipUnchanged (valid never rewritten). (b) On failure, ordered repairs: json-array-parse BEFORE bare-string-wrap; single-arg-{}-unwrap; null-of-optional DROP (omit, never send null). ORDER load-bearing. | J-53:L10-L10, L58-L58 |
| J53-E008 | E/FLOW | (c) typed-path unwrap: for path-typed schema fields, strip degenerate markdown auto-link `[text](url)` ONLY where link-text == url-without-protocol; real links pass through. (d) re-validate. | J-53:L10-L10 |
| J53-E009 | E/LIMIT | NO new dependency: minimal structural type-matcher against declared input_schema "type". Full jsonschema crate DEFERRED, user-confirmation-flagged. No LLM-judge; readable error templated. | J-53:L10-L10, L18-L19, L55-L56, L81-L81 |
| J53-E010 | E/INT | Wire shim: parse_tool_arguments gains tool input_schema via name→schema map from request tools; response tool_call today carries only name+arguments (:813). ShipUnchanged/Repaired → corrected Value; Unrepairable → model-readable tool_input_invalid riding E-18 one-shot retry AFTER deterministic repair. | J-53:L11-L11, L44-L46 |
| J53-E011 | E/TEST | UPDATE test :1096 to assert four shapes repair and unrepairable surfaces readable error (intentional contract change, not gate-gaming). | J-53:L11-L11 |
| J53-E012 | E/CFG | Relational-invariant defaults at TOOL DISPATCHER not parser: `limit` alone → offset=0; `offset` alone → limit=<tool default>; SURFACE chosen default in ToolResult with NO `Error:` prefix so model self-corrects. | J-53:L11-L11, L41-L41 |
| J53-E013 | E/EVENT | +ToolInputRepaired SSB (Pattern-2; Kind: ShapeRepaired / RelationalDefaulted / UnrepairableSurfaced); full 9-tuple lockstep; baseline+1 from LIVE COMPUTED count (~268/577 as of 2026-06-19; re-grep both at edit). | J-53:L12-L12, L32-L33, L42-L42, L71-L71 |
| J53-E014 | E/OBS | Every non-ShipUnchanged emits journal row provider+model+tool+repair-kind. Repair-RATE per (provider, model, tool) is FORMAT signal sibling to J-51 SEMANTIC adherence; lands on same ModelCapabilityProfile/failure_taxonomy without merging metrics. | J-53:L12-L12, L47-L47, L78-L79 |
| J53-E015 | E/CFG | Default-ON kill-switch `ELAI_TOOLCALL_REPAIR_DISABLE` (W-2, NOT a new ENABLE); disabled → legacy passthrough, logged, never silent flip. | J-53:L12-L12, L57-L57 |
| J53-E016 | E/SURFACE | frontend_pair: per-(provider,model) tool-input-repair-rate row on model/cost-cockpit panel family; #[tauri::command] + ts-rs + cross_lang_diff.py PASS. | J-53:L34-L34, L59-L59 |
| J53-E017 | E/FAIL | Missing schema for a tool = surface+log, never silent pass. Unrepairable = model-readable error + SSB, never silent blob (R1). | J-53:L20-L20 |
| J53-E018 | E/LIMIT | NOT constrained decoding; NOT re-prompt first resort (E-18 owns one-shot after repair fails); NOT local Anthropic-shaped sidecar; NOT second cloud strict-schema path (E-35 task 7). | J-53:L16-L17, L77-L77 |
| J53-E019 | E/OBS | Trigger: elai /task --mode cloud with stringified-array fixture then unrepairable fixture. Expect ShapeRepaired{kind=json_array_parse} + tool executes; UnrepairableSurfaced + E-18 retry; limit-only read_file → RelationalDefaulted{offset=0} + ToolResult note. | J-53:L60-L62 |
| J53-E020 | E/FAIL | Negative: with DISABLE set, same malformed arg reproduces {"raw"} passthrough AND disable logged; valid arg byte-identical ShipUnchanged zero repair row; preprocess rewriting JSON writeFile body FAILS contract. | J-53:L63-L63 |
| J53-E021 | E/EVIDENCE | arXiv:2509.18847: small-model re-prompt recovery ~20% vs Claude >50% grounds deterministic-repair-before-reprompt. CommandCodeAI post is WEAK design impetus only, never grounding source. | J-53:L6-L6, L66-L68 |
| J53-E022 | E/DEFER | Riders: jsonschema crate upgrade; per-cloud repair-rate table as K-1 claims material; port to Gemini wire (I-49); CodeAct thin-verb surface is orthogonal (not this plan). | J-53:L81-L85 |
| J53-E023 | E/INT | Production: any elai /task through OpenAI-compat provider whose model emits malformed tool arg (openai_compat.rs:813). | J-53:L49-L51 |
| J53-A001 | A/CFG | Exact list of "offset/limit-class" tools and their default limit values are re-grepped at edit, not enumerated. | J-53:L11-L11, L48-L48 |
| J53-A002 | A/DATA | Structural matcher residual miss rate and when to promote jsonschema are measurement-gated after ship. | J-53:L81-L81 |

Category coverage: capabilities/repair ✓; CLI path (cloud task) ✓; surfaces (cockpit repair-rate, ToolResult note, kill-switch) ✓; flows/order ✓; events/telemetry ✓; failures/retry handoff ✓; determinism ✓; tests ✓; constraints ✓; deps/deferred ✓. No privacy section beyond no secrets in repair logs (provider/model/tool only).

### J-59 — per-seat memory + trust isolation

Source: `docs/thefinalglance/J-plan/J-59_per_seat_memory_trust_isolation.md` (129/129 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| J59-E001 | E/STATUS | J-59 is PLANNED WAVE PV-2 effort M; EXTENSION of J-49 + E-5/H-31/H-32; depends J-49; data-driven activation, no new engine. | J-59:L1-L3 |
| J59-E002 | E/CAP | Ships: per-seat memory namespacing by J-49 seat-id; cross-seat read/write isolation; cross-seat poisoning block; auditor-objectivity firewall extended per-seat-AND-role. | J-59:L3-L5, L29-L35 |
| J59-E003 | E/LIMIT | HARD BOUNDARY: NO new memory engine; NO cross-seat sharing even opt-in in v1; single-seat default = local, byte-identical to pre-J-59; per-seat audit-VIEW UI DEFERRED. | J-59:L5-L5, L59-L59, L63-L64, L124-L124 |
| J59-E004 | E/DEP | GD-1: E-5 Mem0 dispatcher + capture/retrieval; H-31 preference memory + auditor firewall (arXiv:2510.09905 carried, not re-verified); H-32 fact memory; J-49 SeatRegistry + identity; C-48 TrustTier; H-33 MemoryGovernanceEvent prefer-reuse. | J-59:L5-L5, L43-L49, L115-L116 |
| J59-E005 | E/PRIV | Multi-tenant leakage risk: seat A's preferences/facts retrieved into seat B's prompt; poisoned fact contaminates all. Data-foundation + governance pillars frame this as trust/safety gap. | J-59:L3-L4, L12-L13 |
| J59-E006 | E/DET | Isolation is DETERMINISTIC filter on retrieval + write keyed by seat-id (LLM never decides seat; TD-3). Seat-id opaque handle from J-49; auth provenance still J-2. | J-59:L13-L13, L57-L57, L84-L84, L100-L100 |
| J59-E007 | E/DATA | Every memory row (H-31 preference, H-32 fact) carries seat-id; Mem0 ADD/UPDATE write path stamps current seat-id; absent SeatRegistry → local → pre-J-59 identical. | J-59:L30-L30, L39-L39 |
| J59-E008 | E/SEC | Cross-seat retrieval isolation: H-31 hard-filter gains seat predicate; only current seat's rows eligible; another seat's row structurally unreachable (poisoning block). Hard exclusion, not soft de-prioritization. | J-59:L31-L31, L55-L55, L80-L81 |
| J59-E009 | E/SEC | Auditor-objectivity firewall per-seat-AND-role: auditor/critic/adversary get ZERO memory injection under ANY seat (role-gated first, seat-gated second). Never re-implement firewall; compose H-31. | J-59:L32-L32, L56-L56, L82-L82 |
| J59-E010 | E/SEC | Per-seat TrustTier (C-48): a seat's writes carry that seat's tier from seats.toml; low-trust seat cannot poison high-trust seat even within own namespace. | J-59:L33-L33, L20-L20 |
| J59-E011 | E/EVENT | Prefer-reuse H-33 MemoryGovernanceEvent with per-seat discriminator (or J-49 SeatAdmissionEvent); net 0..+1. Row carries seat-id + isolation action (write-stamped / retrieval-filtered / firewall-blocked), NEVER memory content or seat secret. | J-59:L34-L34, L61-L61, L66-L67 |
| J59-E012 | E/INT | Production reachability: memory retrieval-into-prompt (ContextSlot::Memory) and Mem0 ADD/UPDATE write path. Test never R7. | J-59:L50-L53, L85-L85 |
| J59-E013 | E/SURFACE | frontend_pair N/A: per-seat audit-VIEW explicitly DEFERRED; isolation proven by contract + SSB, not rendered view. | J-59:L63-L64, L96-L96 |
| J59-E014 | E/CFG | NO new ELAI_*_ENABLE; multi-seat memory activates by J-49 seats.toml presence (W-2). | J-59:L60-L60, L83-L83 |
| J59-E015 | E/OBS | Trigger: two seats write contradictory prefs; build prompt for seat B; auditor turn under seat A. Expect stamped writes, retrieval-filtered row, firewall-block; seat B prompt has ZERO of seat A; auditor context ZERO memory. | J-59:L65-L68 |
| J59-E016 | E/FAIL | Negative FAIL: cross-seat fact leak; auditor/critic/adversary receiving any memory under any seat; absent seats.toml changing single-seat behavior. | J-59:L68-L68 |
| J59-E017 | E/TEST | Contract j59_per_seat_memory_contract.rs: absent→local identical; two-seat exclusion; cross-seat read empty; cross-seat write rejected; auditor zero memory both seats; low-trust cannot escalate another seat's tier; SSB never carries content/secret. | J-59:L102-L103 |
| J59-E018 | E/RECOVERY | GD-4: firewall + isolation default fail-closed; row whose seat-id is unresolved is EXCLUDED, never injected. | J-59:L100-L100 |
| J59-E019 | E/DEFER | Cross-seat sharing is separate future plan, never a relaxation of hard exclusion. Per-seat audit-VIEW deferred to keep single-user-trust-spine identity. | J-59:L124-L127 |
| J59-E020 | E/DATA | Decision doc J-59-decisions.md; LIVE SCHEMA ~268/577 prefer-reuse H-33. | J-59:L35-L35, L118-L118 |
| J59-A001 | A/EVENT | Reuse H-33 vs dedicated arm decided at edit. | J-59:L34-L34, L122-L122 |
| J59-A002 | A/INT | Exact crates/memory capture+retrieval file layout is PLAN-asserted dark; re-grep at EXECUTING (W-7). | J-59:L16-L16, L121-L121 |
| J59-A003 | A/SEC | Seat-identity authenticity (multi-client) remains J-2's job; J-59 isolates given a correct id. | J-59:L100-L100, L123-L123 |

Category coverage: capabilities ✓; privacy/isolation ✓; security/firewall/trust tiers ✓; workflows write/retrieve ✓; surfaces deferred ✓; events ✓; failures/fail-closed ✓; determinism TD-3 ✓; tests ✓; deps/deferred ✓. No CLI verb; no new enable flag.

### J-68 — formal verification of ELAI's own critical code

Source: `docs/thefinalglance/J-plan/J-68_self_code_formal_verification.md` (74/74 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| J68-E001 | E/STATUS | J-68 is Track J NASA-gates WAVE NASA-GATES-3 BUILD, lever PROVABLE, effort L, depends none (verifies SHIPPED core; sequenced after cheaper gates). | J-68:L1-L3 |
| J68-E002 | E/CAP | Three legs: D1 Kani bounded-model-check; D2 SPARK-style contracts on privileged adapters; D3 TLA+ FSM spec + TLC gate. Distinct from D-11 (z3/sympy on GENERATED math). | J-68:L3-L4, L8-L17, L61-L61 |
| J68-E003 | E/LIMIT | Scoped to SMALL critical core ONLY (cbmc-interface hard boundary). All three UNSOUND beyond bound (ND-2 Layer-2; never replace runtime gates J-66/J-67). | J-68:L6-L6, L18-L18, L50-L50 |
| J68-E004 | E/TEST | D1 harnesses: ssb/payloads serialize/deserialize round-trip; FSM transition never panics on bounded next-state; gate predicates FAIL-CLOSED (evidence-absent → Block); wiring_contract parser never panics; budget counters never underflow. | J-68:L9-L14, L34-L34 |
| J68-E005 | E/FAIL | Auditor runs `cargo kani` in CI; unwinding-assertion failure OR counterexample = BLOCK; HARNESS_NOT_FOUND = BLOCK (fail-closed R1/ND-3). | J-68:L15-L15, L48-L48, L55-L55 |
| J68-E006 | E/CAP | D2: every side-effecting Supervisor-plane fn (file ops, egress, SSB write, credential handling) carries pre/post (assert! entry, debug_assert! before return) + AoRTE discharged by D1 harness BEFORE production side-effect sink. Pairs with no-.unwrap() rule. | J-68:L16-L16, L36-L36 |
| J68-E007 | E/DEP | Crates: kani primary; creusot 0.12.0 optional gold; Prusti REJECTED (unmaintained). Toolchain user-confirmation-flagged at edit (no new dep without confirmation). | J-68:L16-L16, L28-L28, L49-L49, L60-L60 |
| J68-E008 | E/DATA | D3: commit ElaiFSM.tla with Inv2 (AUDITING only from EXECUTING), Inv3 (no EXECUTING without evidence_gate=TRUE), Inv4 (COMPLETE only after MEMORIZING). Privileged FSM change ships spec delta + passing TLC; invariant counterexample = BLOCK. | J-68:L17-L17, L35-L35, L49-L49 |
| J68-E009 | E/LIMIT | EXCLUDED: prove whole workspace; treat Kani PASS as runtime guarantee; TLA+ that models implementation instead of design contract. | J-68:L18-L18, L50-L50 |
| J68-E010 | E/INT | NO SSB action_type (verification artifacts are off-agent CI evidence, JD-1). frontend_pair N/A — verdict surfaces through audit gate. | J-68:L29-L30, L51-L51 |
| J68-E011 | E/INT | Production reachability: AUDITING/CI path auditor runs cargo kani + TLC over changeset. #[cfg(test)] proptest is NOT substitute for bounded proof (R7). | J-68:L42-L45 |
| J68-E012 | E/DATA | + gate-manifest.txt row for cargo kani + TLC CI gates (frozen R8). | J-68:L37-L37, L64-L64 |
| J68-E013 | E/OBS | Trigger: auditor kani on SSB+FSM harnesses + tlc ElaiFSM.tla with deliberately broken transition and bad serializer. Expect counterexample BLOCK both; surface to audit verdict. | J-68:L52-L54 |
| J68-E014 | E/FAIL | Negative: clean core passes; MISSING harness is FAIL not SKIP; vacuous zero-state TLC FAILS; executor local kani necessary-not-sufficient — auditor off-agent re-run is authority; gate_freeze_check proves harnesses/spec not edited by run. | J-68:L55-L55 |
| J68-E015 | E/CFG | CI scope: small core only; run on AUDITING for changesets that TOUCH verified modules (manifest-driven), not every commit; LOG when skipped (no silent skip; ND-3). | J-68:L67-L67 |
| J68-E016 | E/DET | Unwind bound is named constant, documented; residual beyond bound is runtime gates' job (ND-2, GD-5). | J-68:L68-L68 |
| J68-E017 | E/LIMIT | Missing spec-delta on privileged FSM change = BLOCK (R8 frozen rule against TLA drift). | J-68:L69-L69 |
| J68-E018 | E/DEP | Composes phase-3I DST (complements design-time BFS); D-11 sibling disjoint; CLAUDE.md no-.unwrap; J-65 P10 rules same core; gate_freeze_check + gate-manifest; J-70 DO-333 traceability technique=model-check/proof. | J-68:L63-L64, L72-L72 |
| J68-E019 | E/STATUS | FSM is 9-state: PLANNING/TESTING_FIRST/EXECUTING/AUDITING/MEMORIZING/COMPLETE/IDLE + substates. | J-68:L22-L22 |
| J68-A001 | A/DEP | Exact kani/creusot/TLC install/CI wiring and confirmed versions re-verified at edit (dated 0.67.0/0.12.0 snapshot). | J-68:L28-L28, L60-L60 |
| J68-A002 | A/CFG | Numeric unwind bounds per harness are not fixed in the plan body (must be named constants at authoring). | J-68:L68-L68 |

Category coverage: capabilities (proof gates) ✓; CI/audit surface ✓; failures/BLOCK ✓; determinism/soundness bounds ✓; tests/harnesses ✓; constraints ✓; deps/confirmation ✓; deferred whole-workspace ✓. No product UI; no enable flag (CI path).

### J-71 — policy-as-code spike (Cedar)

Source: `docs/thefinalglance/J-plan/J-71_cedar_policy_as_code_spike.md` (62/62 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| J71-E001 | E/STATUS | J-71 is SPIKE (NOT a commitment), effort S, WAVE NASA-GATES-5, depends none. SPIKE-FIRST. No production wiring until spike reports. May be declined. | J-71:L1-L6 |
| J71-E002 | E/CAP | Time-boxed evaluation: re-express ONE existing capability gate as Cedar policy + schema; run SMT policy-equivalence across a refactor; report keep-Python vs adopt-Cedar with MEASURED numbers. | J-71:L6-L12, L8-L12 |
| J71-E003 | E/DATA | Cedar model: principal=agent-role, action=tool/transition, resource=file/SSB-scope, context=FSM-state. Payoff = Theorem 4.2 policy-equivalence (prove refactor changed NO permissions). | J-71:L6-L6, L9-L10, L46-L46 |
| J71-E004 | E/OBS | Measure four numbers: authoring cost, SMT analysis runtime, expressiveness fit, second-source-of-truth cost. | J-71:L11-L11, L38-L38, L42-L42 |
| J71-E005 | E/DATA | ONLY persistent artifact: workflow/reasoning/J-71-cedar-spike.md + decision keep-Python \| author-a-Cedar-adoption-plan \| decline. Throwaway policy/schema in scratch. | J-71:L12-L12, L22-L22, L27-L28 |
| J71-E006 | E/LIMIT | EXCLUDED: any production wiring of Cedar; adding cedar-policy as workspace dep (isolated scratch, user-confirmed; ADOPT decision only); treating one-gate re-expression as migration. | J-71:L13-L13, L35-L37 |
| J71-E007 | E/INT | reachable_from N/A by design — SPIKE, not production feature. Entry is human-read report. | J-71:L32-L33 |
| J71-E008 | E/DEP | cedar-policy 4.11.1 Apache-2.0, anti-slopsquat-verified 2026-06-21; re-verify at edit; isolated scratch NOT workspace dep. | J-71:L18-L18, L47-L47 |
| J71-E009 | E/DEP | Consumes ONE existing gate read-only from rules engine 3F / transition gate / C-42 authz — NOT modified. | J-71:L16-L17, L29-L30 |
| J71-E010 | E/DATA | Premature `.elai_cc/tests/contracts/test_cedar_contract.py` is scaffold-or-removal at user's call; spike does not silently adopt as committed. | J-71:L19-L19, L57-L57 |
| J71-E011 | E/FAIL | Negative: adopt recommendation WITHOUT measured second-source cost is incomplete; wiring Cedar into real gate FAILS read-only constraint; unverified cedar-policy version FAILS anti-slopsquat. | J-71:L43-L43, L38-L38 |
| J71-E012 | E/LIMIT | If report cannot quantify second-source drift cost, recommendation defaults to keep-Python (conservative). | J-71:L55-L55 |
| J71-E013 | E/SURFACE | frontend_pair N/A — spike/report. Justified. | J-71:L23-L23, L39-L39 |
| J71-E014 | E/DEP | Relates to I-1 / I-35 capability gates Cedar could express; J-70 would receive DO-333 row IF adopted. Production plan is SEPARATE future file. | J-71:L48-L48, L51-L52, L59-L60 |
| J71-A001 | A/CAP | Concrete gate chosen for re-expression is pick-at-spike-time. | J-71:L9-L9, L16-L16 |
| J71-A002 | A/DATA | Whether premature test_cedar_contract.py is removed or kept as scaffold is user decision. | J-71:L19-L19, L57-L57 |
| J71-A003 | A/LIMIT | Expressiveness fit for evidence-absent→Block and step-count temporal predicates is unknown until measured. | J-71:L56-L56 |

Category coverage: spike capability ✓; report surface ✓; measurements ✓; constraints/non-wiring ✓; deps ✓; decision outcomes ✓. No product CLI/UI; no SSB; no production flags.

### J-84 — GRC-v3 2-axis reasoning-effort classifier

Source: `docs/thefinalglance/J-plan/J-84_grcv3_reasoning_effort_classifier.md` (116/116 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| J84-E001 | E/STATUS | J-84 is PLANNED WAVE GRC-2 effort M; EXTENSION(H-42 + I-26); depends J-83; the deterministic classifier that picks reasoning budget. | J-84:L1-L3 |
| J84-E002 | E/CAP | 2-axis classifier: TYPE × COST/difficulty → tier {Zero, Depth, Width, EscalateOrAbstain} over H-42 + I-26 budget substrate. | J-84:L3-L5, L14-L18, L41-L43 |
| J84-E003 | E/CAP | CC-1 TYPE axis reuses D-10 select_grounding_mode: reason vs RETRIEVE (factual/recall → DKG/A-RAG, NEVER a chain) vs validate. Does not rebuild D-10. | J-84:L3-L4, L15-L15, L45-L45, L77-L77 |
| J84-E004 | E/CAP | CC-2 COST axis: difficulty ← f(H-39 confidence, C-45/C-47 case-base hit, J-38 coverage, C-29 IntentPair count). Pure deterministic function; NO LLM input (TD-3). | J-84:L3-L4, L16-L16, L35-L35, L78-L78 |
| J84-E005 | E/FLOW | CC-3 routing (DC-5): trivial→ZERO; easy/medium→serial DEPTH (revise J-83 carrier, H-42); hard→parallel WIDTH (J-86 best-of-N/BSP); very-hard (proxy<~0.05)→I-7 escalate or I-8 ABSTAIN. WINDOW is never a tier (RD-4). | J-84:L4-L4, L17-L17, L79-L79 |
| J84-E006 | E/SURFACE | CC-4 effort toggle: manual lambda override (U_lambda) on top of auto-tier via I-27 ConvergenceCurveMeasured + H-9 cost cockpit; user dials light↔heavy; floor/ceiling stay deterministic. | J-84:L4-L4, L18-L18, L37-L37, L52-L52, L80-L80 |
| J84-E007 | E/LIMIT | LLM NEVER picks its own tier (TD-3). Done-gate is ELAI-MEASURED per-(model,task-type) on gold set via K-15 `elai bench reasoning-budget`, NEVER paper quintile bins (ID-5). | J-84:L5-L5, L55-L57, L81-L81 |
| J84-E008 | E/CFG | OBSERVE-ONLY first (GD-4): emit chosen tier vs realized verified-outcome; do NOT gate until K-15 calibrates knee. Then gate. | J-84:L5-L5, L38-L38, L81-L81, L111-L111 |
| J84-E009 | E/CAP | Module `crates/orchestrator/src/reasoning_classifier.rs` (~160 LOC): 2-axis classifier + tier→budget router + lambda toggle. | J-84:L42-L43, L96-L96 |
| J84-E010 | E/EVENT | Prefer-reuse ReasoningBudgetScaled (+Kind: TierRouted / EffortOverridden) — net 0..+1 from LIVE 268/577, decide at edit (W-9). | J-84:L63-L63, L89-L89 |
| J84-E011 | E/SURFACE | frontend_pair: H-9 CloudPanel / cost-effort tile shows chosen tier + lambda dial (decide exact surface at edit); tier-vs-outcome telemetry rides I-27 ConvergencePanel. Justified if no new tile. | J-84:L59-L60 |
| J84-E012 | E/INT | Production: GRC reasoning loop / task_runner sets budget per step; effort toggle via CLI/dashboard lambda dial. Test never R7. | J-84:L50-L53 |
| J84-E013 | E/OBS | Trigger: trivial / factual-recall / medium multi-step / hard branching. Expect trivial→ZERO (no chain budget); recall→RETRIEVE not chain; medium→DEPTH; hard→WIDTH. | J-84:L61-L64 |
| J84-E014 | E/FAIL | Negative: recall routed to reasoning chain MUST NOT happen; LLM-proposed tier is ignored (TD-3). | J-84:L64-L64 |
| J84-E015 | E/TEST | Contract j84: trivial→Zero; recall→Retrieve; medium→Depth; hard→Width; very-hard→EscalateOrAbstain; lambda override within floor/ceiling; LLM tier ignored; empty proxy fails-closed to safe lower-cost/abstain (R1). | J-84:L92-L93 |
| J84-E016 | E/CFG | W-2: I-26 budget rides its existing flag; classifier OBSERVE-ONLY first then default-ON behind a *_DISABLE with allowlist row. | J-84:L86-L86 |
| J84-E017 | E/DEP | Consumes: select_grounding_mode; abstention_score + C-45/C-47 + J-38 + C-29; H-42 ReasoningBudgetScaled + I-26 BudgetController; I-27 + H-9; J-83 BeliefState carrier. | J-84:L44-L49 |
| J84-E018 | E/DATA | Decision doc J-84-decisions.md must state DC-5 explicitly (window→never, easy/medium→depth, hard→width). | J-84:L39-L39, L110-L110 |
| J84-E019 | E/EVIDENCE | Grounded by test-time-compute 2408.03314; zero-shot-CoT CommonsenseQA -4.2pp (recall hurts); RULER window-never; ROUND-3 ADD-1 task-type×gain table is prior not threshold. | J-84:L3-L3, L105-L105 |
| J84-A001 | A/CFG | COST proxy weighting (linear/threshold combination of four signals) starts transparent; calibrate against K-15; not fixed in plan. | J-84:L109-L109 |
| J84-A002 | A/EVENT | Whether to add TierRouted/EffortOverridden Kind arms vs pure reuse is decide-at-edit. | J-84:L63-L63 |
| J84-A003 | A/SURFACE | Exact H-9 tile vs ConvergencePanel-only is decide-at-edit. | J-84:L59-L60 |
| J84-A004 | A/CFG | Exact disable-flag name after observe-only is not fixed (pattern *_DISABLE). | J-84:L86-L86 |

Category coverage: capabilities/router ✓; surfaces (lambda dial, panels, K-15 bench) ✓; workflows/tiers ✓; config/observe-then-gate ✓; events ✓; failures/negatives ✓; determinism TD-3/RD-4 ✓; tests ✓; deps ✓. No privacy; retry not applicable.

### J-85 — GRC-v3 deterministic overthinking guard

Source: `docs/thefinalglance/J-plan/J-85_grcv3_overthinking_guard.md` (124/124 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| J85-E001 | E/STATUS | J-85 is PLANNED WAVE GRC-2 effort M BUILD — the ONE genuinely-new component of GRC-v3 wave; depends J-83; measured by K-15. | J-85:L1-L3 |
| J85-E002 | E/CAP | Deterministic overthinking guard: per-call reasoning CEILING + early-stop MONITOR on three signals that catch chain drifting OFF a correct answer. Distinct from anti-spiral. | J-85:L3-L5, L14-L19 |
| J85-E003 | E/EVIDENCE | Overthinking paper 2604.10739: accuracy non-monotonic in reasoning-token budget; negative flip; oscillation r=0.78 (lead), hesitation r=0.71, declining confidence r=0.63; flip_ratio>1.0 regime; 76.3% precision — design rationale, not ELAI thresholds. | J-85:L3-L4, L112-L112 |
| J85-E004 | E/CAP | OG-1 per-call ceiling: hard reasoning-token ceiling from J-84 tier, bounded by K-15-measured per-(model,task-type) knee, NEVER paper 6K/7K (ID-5). Hitting ceiling = stop+commit-last-confirmed. | J-85:L15-L15, L39-L39, L83-L83 |
| J85-E005 | E/CAP | OG-2 3-signal early-stop: oscillation (intermediate-answer flips, lead), hesitation markers (FROZEN word-list scan, gate-manifest R8), declining confidence (H-39 bit trajectory); flip_ratio>1.0 = STOP. Deterministic TD-3, no LLM. | J-85:L16-L16, L38-L38, L84-L84 |
| J85-E006 | E/CAP | OG-3: SEPARATE detector beside anti_spiral_extended.rs — anti-spiral=repetition, overthinking=drift-off-correct; both run, neither subsumes (GA-3). | J-85:L17-L17, L40-L40, L85-L85 |
| J85-E007 | E/CAP | OG-4 freeze composition: on J-83 receipt-confirmed commit, FREEZE entry so generator cannot oscillate it away (IMPLICATION-31). | J-85:L5-L5, L18-L18, L41-L41, L86-L86 |
| J85-E008 | E/CAP | OG-5 modality-change: after ~3 failed receipted retries, signal MODALITY_CHANGE (retrieve / I-7 escalate / I-8 abstain), never "reason more" (IMPLICATION 23). Guard signals; J-84/J-86 act. | J-85:L5-L5, L19-L19, L42-L42, L87-L87 |
| J85-E009 | E/LIMIT | Guard acts on LEAD signal (oscillation), accepting some FP early stops; tuned on ELAI data (K-15). Never LLM stop (TD-3). Never duplicate anti-spiral. | J-85:L5-L5, L61-L63, L117-L117 |
| J85-E010 | E/CAP | Module `overthinking_guard.rs` (~160 LOC) + frozen hesitation-marker fixture under gate-manifest R8. | J-85:L46-L48, L103-L104 |
| J85-E011 | E/INT | Monitor sits at GRC Verify step / task_runner reasoning loop; freeze on J-83 commit path. Test never R7. | J-85:L38-L38, L57-L59 |
| J85-E012 | E/EVENT | +OverthinkingGuardFired (Pattern-2; Kind: CeilingHit / OscillationStopped / HesitationStopped / ConfidenceDeclined) OR reuse ReasoningBudgetScaled (+Kind) — net 0..+1 from LIVE 268/577. | J-85:L68-L68, L96-L96 |
| J85-E013 | E/SURFACE | chosen-stop-reason surfaces on existing GRC/reasoning telemetry panel + I-27 ConvergencePanel; decide tile at edit or justified N/A. | J-85:L64-L65 |
| J85-E014 | E/CFG | Default-ON behind a *_DISABLE kill-switch with allowlist row (safety guard, not experiment) (W-2). | J-85:L92-L92 |
| J85-E015 | E/OBS | Trigger: task tuned to overthink past peak; guard STOPS at/near measured knee and commits last receipt-confirmed state, NOT drifted one. | J-85:L66-L69 |
| J85-E016 | E/FAIL | Negative: anti-spiral and overthinking DISTINCT (repetition-only fires anti-spiral only; drift fires guard only); LLM-proposed stop NEVER the gate. | J-85:L69-L69 |
| J85-E017 | E/TEST | Contract j85: flip_ratio>1 stops; ceiling commits last confirmed; distinct from anti-spiral; J-83 freeze; 3 retries→MODALITY_CHANGE; LLM keep-going ignored; hesitation list is frozen fixture. | J-85:L99-L100 |
| J85-E018 | E/LIMIT | Local Qwen-7B overthinks earliest/lowest — lowest ceiling, self-eval is noise; do NOT share one knee across lanes (per-(model,task-type)). | J-85:L118-L118 |
| J85-E019 | E/DEP | Consumes anti_spiral_extended; I-26 BudgetController + J-84 tier; abstention_score H-39; J-83 BeliefState; K-15 knee artifact; I-7/I-8. | J-85:L49-L55 |
| J85-A001 | A/DATA | "Intermediate answer" extraction for oscillation counting must be deterministic/cheap (structured field or regex); if it needs the model it is a material TD-3 fork — STOP + handoff. Extraction design open at authoring. | J-85:L119-L119 |
| J85-A002 | A/EVENT | New OverthinkingGuardFired vs ReasoningBudgetScaled reuse is decide-at-edit. | J-85:L68-L68 |
| J85-A003 | A/CFG | Exact DISABLE flag name and initial knee values before K-15 measurement are open. | J-85:L92-L92, L39-L39 |
| J85-A004 | A/CFG | Exact ~3 failed-retry threshold is approximate in source ("~3"), not a named const yet. | J-85:L19-L19, L42-L42 |

Category coverage: capabilities/guard ✓; surfaces (stop-reason panel, disable flag) ✓; workflows ✓; events ✓; security/freeze ✓; failures ✓; determinism ✓; tests ✓; deps ✓. No privacy; no formal idempotency.

### J-103 — graduated proof-of-need read-access ladder + Read-tool interceptor + savings meter

Source: `docs/thefinalglance/J-plan/J-103_read_access_ladder_interceptor.md` (138/138 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| J103-E001 | E/STATUS | J-103 is PLANNED WAVE 1 BUILD, effort L; NEW policy/decision layer composing shipped rungs + rules engine + I-46 interceptor + J-38 sufficiency; default-OFF observe-only first. | J-103:L1-L5 |
| J103-E002 | E/CAP | Graduated read-access ladder: agent NAMEs identifiers expected; enforcer verifies they exist in served rung before return; serve CHEAPEST rung that satisfies; climb-on-miss; never hard-deny on economy. | J-103:L1-L1, L11-L12, L42-L46 |
| J103-E003 | E/STATE | Rung enum mapping to SHIPPED retrieval: SymbolCard (DKG/B-29 symbol_context) → Skeleton (B-29 traversal/signatures) → HotPathExcerpt (E-17 FAST/STANDARD + J-25) → CodeWindow (full file). NO new retrieval — each rung dispatches existing path. | J-103:L43-L43, J-103:L52-L53 |
| J103-E004 | E/CAP | READ-ARTIFACT predicate family on phase-3F rules engine: given Read request + NAMED identifiers + J-78 access scope → approve(rung) / DOWNGRADE-to-rung-N / deny. | J-103:L24-L24, L44-L44, L54-L54 |
| J103-E005 | E/SEC | Deny ONLY on security/capability_gate overlap (F-1), NEVER on economy. Interceptor BIASES to DOWNGRADE + ESCALATE-ON-MISS (over-block guard / YoloFS failure mode). | J-103:L14-L16, L74-L74, L100-L100 |
| J103-E006 | E/CAP | Widen I-46 bash_validation refuse-with-redirect to first-class Read tool at PreToolUse / IpcToolDispatcher: naive full-file Read intercepted → require named identifiers → serve cheapest satisfying rung. HARD gate, never prompt nudge. | J-103:L18-L18, L45-L45, L55-L55, L76-L76 |
| J103-E007 | E/FLOW | Presence-check + escalate-on-miss: after serve, J-38 sufficiency (symbol-coverage of NAMED identifiers over served text vs C-29 IntentPair). Coverage < threshold OR identifier absent → CLIMB one rung, re-serve (max-rounds, TD-3). Hallucinated id escalates to CodeWindow, never silent thin rung. | J-103:L15-L15, L46-L46, L56-L56, L70-L70, L99-L99 |
| J103-E008 | E/OBS | Meter: would-have-read-tokens (CodeWindow size) vs actually-served-tokens; surface "saved X vs naive read" via J-29 LiveUsageGauge (read-side prefer-reuse, NO new action_type required); consume CatRAG phase-1E savings accounting. G-5 instrument + done-gate source. | J-103:L47-L47, L57-L57, L81-L81, L98-L98 |
| J103-E009 | E/EVENT | OPTIONAL +ReadAccessDecision Pattern-2 Kind {Approve, Downgrade, Deny, EscalateOnMiss} if journaled decision wanted (recommended GD-2); else net 0 ride J-29 gauge. TOTAL_VARIANTS from LIVE at edit. | J-103:L48-L48, L58-L58, L84-L84, L132-L132 |
| J103-E010 | E/CFG | Flag `ELAI_READ_LADDER_ENABLE` default-OFF with wire-dark-allowlist row (type=experiment, promotion_plan_id=J-103-promotion); observe-only until over-block/FP bounded on ≥3 real runs (GD-4). | J-103:L79-L79, L103-L103, L108-L108, L121-L121 |
| J103-E011 | E/DET | Rung decision + climb are deterministic; LLM only NAMES identifiers (TD-3). Task-outcome parity measured by GATE VERDICTS (F-4/wiring/intent-pair), never LLM-judge. | J-103:L77-L77, L101-L102 |
| J103-E012 | E/LIMIT | NO soft prompt-nudge interception; NO new retrieval path; NO paper MutaGReP/YoloFS/CodeRay number as done-gate (ID-5). sdl-mcp design ideas only; NO code copied (license). | J-103:L7-L7, L76-L78, L101-L101, L127-L127 |
| J103-E013 | E/DEP | Soft-dep J-78: if unshipped, degrades to "name identifiers" without declared-scope cross-check (still correct, weaker provenance). Composes J-38, J-29, I-46, B-29, E-17, DKG, rules engine, capability_gate/F-1. | J-103:L26-L27, L59-L67, L133-L133 |
| J103-E014 | E/SURFACE | "saved X vs naive read" on J-29 usage meter / ContextStatePanel; dedicated tile NOT required (decide at edit or W-6). | J-103:L80-L81 |
| J103-E015 | E/INT | Production: live `elai run` Read naming real ids (cheapest rung + meter); hallucinated id climbs to CodeWindow; legitimate whole-file DOWNGRADE at most, never economy hard-deny. Test never R7. | J-103:L68-L72 |
| J103-E016 | E/TEST | Contract j103 (~150 LOC): real-id cheapest rung + meter vs CatRAG; BRITTLENESS-NEGATIVE hallucinated climbs never thin-miss; OVER-BLOCK-NEGATIVE whole-file never economy-deny; security-only deny path; deterministic under ELAI_SEED; optional decision rows replay. | J-103:L111-L112 |
| J103-E017 | E/DATA | Module `read_ladder.rs`; MODIFY rules predicate family; MODIFY bash_validation + Read dispatch seam; MODIFY J-29 gauge; optional SSB 9-tuple; CREATE J-103-decisions.md; CONFIG wire-dark-allowlist row. | J-103:L114-L121 |
| J103-E018 | E/FAIL | Done gate: zero "served a rung missing a named identifier" cases. Over-block rate is gating metric before default-ON. | J-103:L99-L99, L130-L130 |
| J103-A001 | A/EVENT | Journal ReadAccessDecision vs net-0 gauge is open DECISION resolved in J-103-decisions.md. | J-103:L48-L49, L132-L132 |
| J103-A002 | A/CFG | Exact symbol-coverage threshold and max-rounds for climb are not numeric in the plan. | J-103:L46-L46 |
| J103-A003 | A/SURFACE | Dedicated savings tile vs gauge-only is decide-at-edit. | J-103:L80-L81 |

Category coverage: capabilities/rungs/interceptor/meter ✓; surfaces (Read intercept, savings gauge, enable flag) ✓; workflows climb ✓; security deny path ✓; failures/brittleness/over-block ✓; determinism ✓; events optional ✓; tests ✓; deps/soft-dep ✓; observe-first ✓. Privacy not separate beyond scope enforcement.

## Cross-source interactions and lifecycle

| Interaction ID | Class | Interaction, ordering, or shared invariant | Evidence atoms |
|---|---|---|---|
| IX-001 | E | J-9 catalog is informational-only substrate; later promotion into routing is out of scope and must not be consumed by cloud_mode/escalation_policy (JC-10). | J9-E004, J9-E019, J9-E021 |
| IX-002 | E | J-9 vision/OCR DESIGN seeds coordinate with J-8 vision_ladder.toml ownership; J-9 ships first, J-8 eval refreshes MEASURED. | J9-E007, J9-E010, J9-E026 |
| IX-003 | E | J-18 embedding swap + LSP channel feed J-16 blast-radius edges; agentic-grep is narrow confirm while J-38 is the broad mid-task loop that exposes J-18's cascade. | J18-E009–J18-E011, J18-E021 |
| IX-004 | E | J-18 reuses E-52 rust-analyzer harness and E-17/RRF cascade; HARDEN-not-replace leaves tree-sitter fallback and SearchMode DEGRADED path intact. | J18-E008, J18-E021, J18-E019 |
| IX-005 | E | J-49 depends on J-48 decide() and fronts it; multi-seat activates by seats.toml; identity provenance is J-2's future job. | J49-E001–J49-E003, J49-E008, J49-E018 |
| IX-006 | E | J-49 reuses J-28 identity field, C-5/CloudModeRouter caps, J-29 UsageWindow breakdown; credit-unit rejection stands. | J49-E004–J49-E005, J49-E009–J49-E010 |
| IX-007 | E | J-59 depends on J-49 seat-id and seats.toml activation; economics (J-49) then memory/trust isolation (J-59). | J59-E001–J59-E002, J59-E014; J49-E003 |
| IX-008 | E | J-59 extends H-31 auditor firewall (zero memory for auditor/critic/adversary under ANY seat) and C-48 TrustTier per-seat; composes E-5/H-32 without new engine. | J59-E009–J59-E010, J59-E004 |
| IX-009 | E | J-53 repair-rate is the FORMAT sibling of J-51 SEMANTIC adherence on the same ModelCapabilityProfile/failure_taxonomy family without merging metrics. | J53-E014, J53-E001 |
| IX-010 | E | J-53 repairs first on OpenAI-compat unmasked seam; Unrepairable rides E-18 one-shot retry; local XGrammar/E-35/C-26 prevent-side is untouched. | J53-E002–J53-E003, J53-E010, J53-E018 |
| IX-011 | E | J-84 sizes the J-83 carrier's budget; J-85 caps and early-stops during the call; J-86 owns WIDTH branches; sequence GRC: carrier → classifier/guard → candidates. | J84-E001, J84-E005; J85-E001, J85-E004; IX summary spine |
| IX-012 | E | J-84 and J-85 are parallel-safe (disjoint seams): classifier sets budget UP FRONT; guard stops DURING. Both calibrate on K-15 `elai bench reasoning-budget`. | J84-E007–J84-E008; J85-E001, J85-E004 |
| IX-013 | E | J-85 freezes J-83 receipt-confirmed commits and signals modality change to J-84/J-86 (retrieve/escalate/abstain), never more reasoning. | J85-E007–J85-E008 |
| IX-014 | E | J-85 runs BESIDE anti_spiral_extended (repetition) without merge; distinct stop reasons recorded. | J85-E006, J85-E016 |
| IX-015 | E | J-103 applies J-38 sufficiency as presence-check climb; enforces J-78 declared scope when available; meters through J-29/CatRAG; widens I-46 hard intercept to Read. | J103-E006–J103-E008, J103-E013 |
| IX-016 | E | J-103 deny path is only capability_gate/F-1 security overlap; economy path is downgrade/climb — pairs with J-18 retrieval hardening as "what to fetch" vs "how much code to serve". | J103-E005; J18-E002 |
| IX-017 | E | J-68 is proof leg of NASA-gates (complements phase-3I DST); J-71 is last non-committal Cedar spike; both require user confirmation before new toolchain deps. | J68-E002–J68-E003, J68-E007; J71-E001, J71-E006 |
| IX-018 | E | J-68 TLA Inv2/3/4 over FSM and Kani fail-closed predicates strengthen the same gate machinery J-71 might later re-express as Cedar. | J68-E004, J68-E008; J71-E003, J71-E014 |
| IX-019 | E | SSB families in this chunk: HardwareModelCatalogEvent; RetrievalAugmentEvent (or QueryTrace reuse); SeatAdmissionEvent (or J-48 reuse); ToolInputRepaired; MemoryGovernanceEvent reuse; no J-68/J-71 action type; ReasoningBudgetScaled reuse (±Kind); OverthinkingGuardFired optional; ReadAccessDecision optional. All require live baseline re-read at edit. | J9-E014; J18-E013; J49-E011; J53-E013; J59-E011; J68-E010; J84-E010; J85-E012; J103-E009 |
| IX-020 | E | Flag postures differ: J-9 Tier-1 default-ON no flag / Tier-2 deferred; J-49/J-59 data-driven seats.toml; J-53 ELAI_TOOLCALL_REPAIR_DISABLE default-ON; J-84 observe-then-DISABLE; J-85 default-ON *_DISABLE; J-103 ELAI_READ_LADDER_ENABLE default-OFF allowlist; J-18 agentic-grep flag TBD; J-68/J-71 no product flags. | J9-E023; J49-E003; J59-E014; J53-E015; J84-E016; J85-E014; J103-E010; J18-A001 |
| IX-021 | E | User-facing surfaces cluster: deferred `elai hardware-catalog` + HardwareModelCatalogPanel (J-9); KnowledgeDrawer channel breakdown (J-18); J-29 per-seat UsageWindow (J-49); repair-rate cockpit tile + ToolResult default note + DISABLE (J-53); deferred per-seat audit view (J-59); CI/audit kani/TLC (J-68); spike report (J-71); lambda dial + ConvergencePanel/H-9 (J-84); stop-reason panel + DISABLE (J-85); Read intercept + savings gauge + ELAI_READ_LADDER_ENABLE (J-103); K-15 bench shared by J-84/J-85. | J9-E016; J18-E014; J49-E013; J53-E012, J53-E015–J53-E016; J59-E013; J68-E010; J71-E005; J84-E006, J84-E011; J85-E013–J85-E014; J103-E006, J103-E008, J103-E010 |
| IX-022 | I | Multi-seat + repair + catalog + read ladder all journal decisions that a future J-2 multi-client control plane and J-29/H-9 cockpits will display; identity/auth authenticity remains outside this chunk. | J49-E018; J59-A003; J9-E016; J103-E008 |
| IX-023 | E | Determinism doctrine cluster: TD-3 (no LLM seat/tier/stop/rung adjudication) binds J-59, J-84, J-85, J-103; ID-5 measure-your-own knee binds J-84/J-85/K-15 and forbids paper numbers on J-103; GD-4 observe-only-first binds J-84 and J-103. | J59-E006; J84-E007–J84-E008; J85-E004, J85-E009; J103-E010–J103-E011 |

## Explicit ambiguity and conflict register

| Register ID | Kind | Unresolved issue or required correction | Atom references |
|---|---|---|---|
| AC-001 | ambiguity | J-9 single canonical startup emit site among plural probe call sites is edit-time. | J9-A001 |
| AC-002 | ambiguity | J-9 fit-score weights and platform_tag multi-GPU rules live in decisions/residue, not fixed numbers. | J9-A002–J9-A003 |
| AC-003 | ambiguity | J-18 agentic-grep flag name, low-confidence threshold, SSB reuse, and KnowledgeDrawer decide-at-edit. | J18-A001–J18-A004 |
| AC-004 | ambiguity | J-18 long-function chunking A/B for 2048-trained positions is open first checkbox. | J18-A005 |
| AC-005 | ambiguity | J-49 SeatAdmissionEvent vs J-48 row reuse; numeric bucket/cap constants; seat-id provenance pre-J-2. | J49-A001–J49-A003 |
| AC-006 | conflict/summary | J-53 source load-bearing repair ORDER is json-array-parse BEFORE bare-string-wrap (then {}-unwrap, null DROP); summary compresses order as "omit null, parse stringified array, unwrap object, wrap bare string" and must not silently replace the source order. | J53-E007; SD-J53-001 |
| AC-007 | ambiguity | J-53 offset/limit-class tool set and default limit values re-grepped at edit. | J53-A001 |
| AC-008 | ambiguity | J-59 H-33 reuse vs new arm; exact memory capture file layout; seat authenticity deferred to J-2. | J59-A001–J59-A003 |
| AC-009 | ambiguity | J-68 numeric unwind bounds and confirmed toolchain versions at edit. | J68-A001–J68-A002 |
| AC-010 | ambiguity | J-71 concrete gate choice; premature test_cedar_contract.py keep/remove; expressiveness fit unknown. | J71-A001–J71-A003 |
| AC-011 | ambiguity | J-84 proxy weights, Kind arms, exact panel surface, and post-observe disable flag name. | J84-A001–J84-A004 |
| AC-012 | ambiguity | J-85 intermediate-answer extraction design (TD-3 fork risk); event family; DISABLE name; exact ~3 retry const. | J85-A001–J85-A004 |
| AC-013 | ambiguity | J-103 journal-vs-gauge decision; coverage threshold/max-rounds; dedicated savings tile. | J103-A001–J103-A003 |

## Summary additions and corrections

Orientation comparison target: `docs/thefinalglance/feature-discovery/10-J-SUMMARY_INDEX.discovery.md`. “ADD” means an explicit source behavior omitted or materially compressed by the summary; “CORRECT/QUALIFY/REMOVE” means the summary’s wording changes scope or certainty. These are relational records, not additional feature atoms.

### J-9 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-J9-001 | **ADD** Wave-2 Tier-1 default-ON / Tier-2 deferred status, effort estimate, parallel-safe set, and HW-1..HW-9 consume-only boundary (zero probing code). | 10-J:L118-L127 | J9-E001, J9-E003, J9-E008, J9-E023 |
| SD-J9-002 | **ADD** exact ModelCatalogEntry schema, model_catalog.toml path, lane seed sources (E-31, VR-13, PaddleOCR, GraphCodeBERT), and DESIGN/MEASURED source tags. | 10-J:L120-L121 | J9-E009–J9-E011 |
| SD-J9-003 | **ADD** full HardwareModelCatalogEvent kinds, LIVE 226/537 baseline, duplicate-CatalogBuilt-per-boot FAIL, and routing-isolation grep contracts. | 10-J:L125 | J9-E014–J9-E015, J9-E019, J9-E022, J9-E028 |
| SD-J9-004 | **ADD** exact CLI `elai hardware-catalog [--lane X] [--json]`, HardwareModelCatalogPanel, backend_missing reason, and corrupt-toml lanes_unavailable degrade. | 10-J:L126 | J9-E012, J9-E016, J9-E022 |
| SD-J9-005 | **ADD** multi-GPU/estimate residue, vision_ladder ownership with J-8, and explicit out-of-scope routing promotion. | 10-J:L1280 | J9-E024–J9-E026, J9-A001 |

### J-18 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-J18-001 | **ADD** query-prefix contract, 2048-vs-8192 trained-context risk, self-export ONNX (anti-slopsquat), and literature-60.1 ban as acceptance. | 10-J:L215-L223 | J18-E003–J18-E006 |
| SD-J18-002 | **ADD** prefer rust-analyzer stdio over Serena, RRF no-hit-no-filler rule, selective LSP invocation, and agentic-grep scope vs J-38. | 10-J:L219-L221 | J18-E007, J18-E010–J18-E011, J18-E017, J18-E020 |
| SD-J18-003 | **ADD** full RetrievalAugmentEvent kinds, LIVE 231/542, prefer-reuse QueryTrace, and HARDEN-not-replace paper reinforcement. | 10-J:L222 | J18-E013, J18-E022 |
| SD-J18-004 | **ADD** full exclusion list (CodeXEmbed/jina/RANGER) and Louvain deferral. | 10-J:L215-L223 | J18-E012, J18-E023 |
| SD-J18-005 | **FLAG** unnamed agentic-grep flag and confidence threshold compressed away by the summary. | 10-J:L220 | J18-A001–J18-A002 |

### J-49 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-J49-001 | **ADD** WAVE LSW-2, effort, try_complete production seam, RunPod contention gate, and equal-weight GD-5 residue. | 10-J:L541-L549 | J49-E001, J49-E014, J49-E017, J49-E019 |
| SD-J49-002 | **ADD** exact SeatRegistry field shape, seats.toml inventory validation, concurrency cap field, and corrupt-toml degrade. | 10-J:L543 | J49-E007, J49-E015 |
| SD-J49-003 | **ADD** SSB seat-id opaque / no-PII rule, SeatAdmissionEvent prefer-reuse ambiguity, and GD-4 permissive-until-≥3-sessions. | 10-J:L541-L549 | J49-E011–J49-E012, J49-E017, J49-A001 |
| SD-J49-004 | **ADD** full contract arms including synthetic-credit ban as test and identity-as-input / J-2 auth deferral. | 10-J:L548 | J49-E015–J49-E016, J49-E018 |

### J-53 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-J53-001 | **CORRECT** repair order: source requires json-array-parse BEFORE bare-string-wrap (load-bearing double-wrap hazard); summary reorders omit-null first. | 10-J:L588 | J53-E007 |
| SD-J53-002 | **ADD** exact seam anchors (:813/:844/:1096), intentional test rewrite, tool_input_invalid readable error, and schema-map threading from request tools. | 10-J:L584-L593 | J53-E004, J53-E010–J53-E011 |
| SD-J53-003 | **ADD** typed-path only when link-text == url-without-protocol; ToolResult default note without `Error:` prefix; ShipUnchanged byte-identity. | 10-J:L589-L590 | J53-E008, J53-E012, J53-E006 |
| SD-J53-004 | **ADD** ToolInputRepaired kinds, LIVE ~268/577, cockpit cross_lang_diff requirement, and jsonschema/I-49/K-1 riders. | 10-J:L593, L1312 | J53-E013, J53-E016, J53-E022 |
| SD-J53-005 | **ADD** explicit local OpenAI-compat future backends (Ollama/llama.cpp/vLLM-OpenAI-mode) as in-scope while Anthropic sidecar stays out. | 10-J:L586 | J53-E002–J53-E003 |

### J-59 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-J59-001 | **ADD** WAVE PV-2, ContextSlot::Memory injection site, Mem0 ADD/UPDATE stamp, and fail-closed unresolved seat-id. | 10-J:L645-L653 | J59-E007–J59-E008, J59-E012, J59-E018 |
| SD-J59-002 | **ADD** per-seat C-48 TrustTier, role-then-seat firewall order, and isolation action kinds on MemoryGovernanceEvent. | 10-J:L647-L651 | J59-E009–J59-E011 |
| SD-J59-003 | **ADD** full contract arms (cross-seat write reject, tier non-escalation, SSB no content) and dark capture/retrieval layout residue. | 10-J:L645-L653 | J59-E017, J59-A002 |
| SD-J59-004 | **ADD** explicit no-cross-seat-sharing future-plan boundary and single-user-trust-spine rationale for deferred audit UI. | 10-J:L650, L1290 | J59-E003, J59-E019 |

### J-68 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-J68-001 | **ADD** exact Inv2/Inv3/Inv4 statements, 9-state FSM list, and D1 module list (ssb/payloads, FSM, gates, wiring parser, budget counters). | 10-J:L731-L739 | J68-E004, J68-E008, J68-E019 |
| SD-J68-002 | **ADD** D2 AoRTE/pre-post on Supervisor side-effect sinks, creusot optional, Prusti rejected, and no-SSB/off-agent CI evidence posture. | 10-J:L735-L738 | J68-E006–J68-E007, J68-E010 |
| SD-J68-003 | **ADD** manifest-driven CI skip logging (ND-3), named unwind-bound residue, missing-spec-delta BLOCK, and gate_freeze authority. | 10-J:L738-L739 | J68-E014–J68-E017 |
| SD-J68-004 | **ADD** distinct-from-D-11 framing and J-70 technique=model-check composition. | 10-J:L731-L739, L1292 | J68-E002, J68-E018 |

### J-71 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-J71-001 | **ADD** WAVE NASA-GATES-5, cedar-policy 4.11.1 scratch isolation, Thm 4.2 payoff wording, and default keep-Python if second-source unquantified. | 10-J:L759-L767 | J71-E001, J71-E003, J71-E008, J71-E012 |
| SD-J71-002 | **ADD** premature test_cedar_contract.py scaffold-or-remove question and separate future adoption-plan requirement. | 10-J:L766 | J71-E010, J71-E014, J71-A002 |
| SD-J71-003 | **ADD** I-1/I-35 related-gate context and expressiveness-fit honesty requirement for temporal/evidence-absent conditions. | 10-J:L759-L767 | J71-E014, J71-A003 |

### J-84 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-J84-001 | **ADD** exact proxy input bundle (H-39, C-45/C-47, J-38, C-29), very-hard ~0.05, and OBSERVE-ONLY-before-gate/GD-4. | 10-J:L886-L894 | J84-E004–J84-E005, J84-E008 |
| SD-J84-002 | **ADD** module path reasoning_classifier.rs, ReasoningBudgetScaled Kind options, I-26 existing flag ride, and fail-closed empty proxy. | 10-J:L886-L894 | J84-E009–J84-E010, J84-E015–J84-E016 |
| SD-J84-003 | **ADD** DC-5 must appear in decisions doc; proxy weight calibration against K-15; H-9/ConvergencePanel surface options. | 10-J:L891-L894 | J84-E018, J84-A001, J84-E011 |
| SD-J84-004 | **ADD** `elai bench reasoning-budget` (K-15) as done-gate and paper-bin ban (ID-5). | 10-J:L894, L1172 | J84-E007 |

### J-85 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-J85-001 | **ADD** paper signal correlations as design rationale only; act-on-lead-oscillation FP acceptance; Qwen-7B lowest-knee residue. | 10-J:L896-L904 | J85-E003, J85-E009, J85-E018 |
| SD-J85-002 | **ADD** OverthinkingGuardFired kinds, freeze composition IMPLICATION-31, modality-change signal ownership by J-84/J-86. | 10-J:L901-L902 | J85-E007–J85-E008, J85-E012 |
| SD-J85-003 | **ADD** default-ON *_DISABLE safety posture, intermediate-answer extraction TD-3 fork risk, and frozen hesitation R8 fixture path. | 10-J:L904 | J85-E005, J85-E010, J85-E014, J85-A001 |
| SD-J85-004 | **ADD** full contract arms including distinct-from-anti-spiral dual fixtures. | 10-J:L904 | J85-E016–J85-E017 |

### J-103 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-J103-001 | **ADD** exact rung→shipped-path mapping, READ-ARTIFACT verdict family, and PreToolUse/IpcToolDispatcher call-site. | 10-J:L1085-L1095 | J103-E003–J103-E004, J103-E006 |
| SD-J103-002 | **ADD** optional ReadAccessDecision kinds, CatRAG accounting consumption, promotion_plan_id=J-103-promotion, and soft-dep J-78 degrade path. | 10-J:L1093-L1095, L1299 | J103-E008–J103-E010, J103-E013 |
| SD-J103-003 | **ADD** brittleness (CrossCodeEval) and over-block (YoloFS) guards as dual load-bearing constraints with named negatives. | 10-J:L1089-L1091 | J103-E005, J103-E007, J103-E016, J103-E018 |
| SD-J103-004 | **ADD** sdl-mcp license ban on code copy, decision-doc open questions, and gauge-vs-journal SSB decision. | 10-J:L1085-L1095 | J103-E012, J103-A001, J103-E017 |

## Exhaustiveness, zero-omission, and duplicate audit

- Source coverage: **10/10 files**, **1,226/1,226 logical lines**, **151,391/151,391 bytes** read and reconciled; source SHA mismatches **0**.
- Atomic source ledger: **236 unique atoms** = **204 explicit**, **000 inferred**, **032 ambiguous**, **000 conflicting/corrective** (source-vs-summary repair-order conflict is register AC-006 / summary delta SD-J53-001, not a per-file C atom).
- Per-file atom counts: J-9 **031**; J-18 **028**; J-49 **024**; J-53 **025**; J-59 **023**; J-68 **021**; J-71 **017**; J-84 **023**; J-85 **023**; J-103 **021**. Sum **236**.
- Relational interaction records: **23** (`IX-001`–`IX-023`). Summary-delta records: **042** (`SD-*`). Ambiguity/conflict register: **13** (`AC-001`–`AC-013`). These records reference source atoms and are not double-counted as features.
- ID audit: every atomic table row has exactly one unique ID matching `J9/J18/J49/J53/J59/J68/J71/J84/J85/J103-[EIAC]nnn`. Duplicate atomic IDs **0**; missing/blank atomic IDs **0**; nonconforming atomic IDs **0**.
- Semantic duplicate audit: repeated premise/scope/wiring/grounding/acceptance statements were merged into one atom with multiple anchors. Interactions, conflict register, and summary deltas reference atoms instead of cloning them. Duplicate feature atoms found **0**.
- Omission audit: every source was checked against capabilities/actions/surfaces; workflows/states; schemas/data/lifecycle; flags/defaults/config; integrations/events; permissions/security/privacy/compliance; failures/fallback/retry/recovery; determinism/idempotency/concurrency; observability/evidence; tests/acceptance/done gates; constraints/non-goals; dependencies/status/deferred. Each per-file ledger explicitly names absent dimensions rather than inventing them. User-facing surfaces (CLI verbs, panels, drawers, gauges, kill/enable flags, ToolResult notes, benches, CI audit gates, spike reports) were recorded as standalone atoms where present and never folded only into backend bullets.
- Interpretation audit: inferred, ambiguous, and conflicting/corrective atoms are visibly separated by ID/class. No inference is promoted to a requirement. Orientation-only claims remain in delta records and are not source atoms.
- Manifest-boundary audit: the only original sources opened were the ten exact DEEP-J-012 manifest rows. The only orientation artifact consulted was `10-J-SUMMARY_INDEX.discovery.md`.
- Output metrics after finalization: `output_lines=0512`; `output_bytes=079561`.
- Canonical normalized output SHA-256: b4709d4fc48f0a5e60c699f9b027b3e4ea7906e15f7d659952ad248b81650fbe.
- Hash rule: compute SHA-256 over the exact final UTF-8 bytes including the final newline after replacing only the 64 hexadecimal characters on the `Canonical normalized output SHA-256` line with 64 ASCII zeroes. Fixed-width output metrics and digest substitution avoid impossible raw self-reference.
