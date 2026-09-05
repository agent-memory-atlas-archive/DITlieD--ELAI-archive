# DEEP-PHASE-004 exhaustive feature discovery

## Scope, integrity, and method

- Processed manifest: `docs/thefinalglance/feature-discovery/DEEP-SOURCE-MANIFEST.md:L1370-L1381`, heading `### DEEP-PHASE-004`; the next chunk starts at L1383 (`### DEEP-PHASE-005`).
- Declared chunk: family PHASE, **6 files**, **2,773 logical lines**, **176,724 bytes**, all six classified `numbered/index original`; target status **within 1,800–2,800**.
- Mechanical verification: fresh `sha256sum`, `wc -l`, and byte-size checks matched every manifest row and both aggregates exactly. Missing paths: **0**; unexpected paths: **0**; duplicate paths: **0**; SHA mismatches: **0**; line mismatches: **0**; byte mismatches: **0**.
- Read boundary: all six listed originals were read completely, first through last logical line. No original assigned to another deep chunk was opened. `01-phase-index-MASTER.discovery.md` was consulted only as an orientation and delta-comparison artifact; no source atom below is derived solely from it.
- Anchor notation refers to the manifest-verified source images: `1E:Lx-Ly`, `2B:Lx-Ly`, `2C:Lx-Ly`, `3L:Lx-Ly`, `4C:Lx-Ly`, and `4I:Lx-Ly`.
- Repeated premise, revised-scope, wiring, grounding, composition, and acceptance language is collapsed into one atom with every load-bearing anchor. Interaction, ambiguity/conflict, and summary-delta records reference atoms and are not counted again as features.
- Classification: `E` = explicit source fact/requirement; `I` = bounded inference; `A` = unresolved ambiguity/choice; `C` = internal conflict or summary correction that must not be silently flattened. Domains: `STATUS`, `CAP`, `SURFACE`, `FLOW`, `STATE`, `DATA`, `CFG`, `INT`, `EVENT`, `SEC`, `PRIV`, `COMP`, `FAIL`, `RECOVERY`, `RETRY`, `DET`, `IDEM`, `CONC`, `OBS`, `TEST`, `LIMIT`, `DEP`, `DEFER`, `EVIDENCE`.

### Exact processed manifest and verification

| Manifest class | Lines | Bytes | SHA-256 | Exact absolute path | Verification |
|---|---:|---:|---|---|---|
| numbered/index original | 443 | 24,750 | 8ed050b69b6cf2b19a0915fdd7cea9bfa83844c0baa7a4d79f747933083d51cb | docs/specs/guide/steps/phase-1E.txt | exact |
| numbered/index original | 580 | 37,770 | 80df17b315e70cbd3ecaeea8fe22ccf0414bc2b85efc2d0a1807b58b5e9b3f51 | docs/specs/guide/steps/phase-2B.txt | exact |
| numbered/index original | 496 | 33,046 | ef2a2fc06ae8035a3d88a5aad8211be3b77e21507bc8a2982a7000ebe14435ee | docs/specs/guide/steps/phase-2C.txt | exact |
| numbered/index original | 251 | 13,522 | 23ba0f82895dea2027f715e9699759dfc1e9e36e3fe58e0973efcf615dfbc048 | docs/specs/guide/steps/phase-3L.txt | exact |
| numbered/index original | 380 | 24,461 | 9cfe4c5fb4337ac75555bfde25473912c921adf9ed0fda1f6e5300699b76a75e | docs/specs/guide/steps/phase-4C.txt | exact |
| numbered/index original | 623 | 43,175 | e220d899bc5f0b109e424509213c289a54cc06c31f11db810b300a3d0f6a379b | docs/specs/guide/steps/phase-4I.txt | exact |
| **Aggregate** | **2,773** | **176,724** | — | **6 exact paths** | **exact** |

## Per-file atomic ledgers

### 1E — CatRAG + A-RAG (query pipeline assembly)

Source: `docs/specs/guide/steps/phase-1E.txt` (443/443 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| P1E-E001 | E/STATUS | Phase 1E assembles full retrieval: query → BM25 → vector → PageRank → CatRAG → A-RAG. Prerequisites: 1D complete (PageRank + DKG); depends on 1B BM25 and 1C vectors. | 1E:L1-L9 |
| P1E-E002 | E/CAP | CatRAG is query-aware graph traversal over DKG (evidence-chain expansion). A-RAG is hierarchical retrieval with snippet progressive disclosure. Target 55–87% token savings vs full-chunk; sub-second latency goal; interactive budget ≤2s total. | 1E:L7-L11, L61-L62, L438-L442 |
| P1E-E003 | E/EVIDENCE | Grounding papers: CatRAG arXiv 2602.01965 (interface `workflow/papers/catrag-interface.md`); A-RAG arXiv 2602.03442 (interface `workflow/papers/arag-interface.md`); knowledge.txt Layers 4–5 + §3 query pipeline. | 1E:L13-L24 |
| P1E-E004 | E/DATA | CatRAG paper→ELAI map: entity→DkgNode; passage→file chunks; relation edges→calls/references/imports/implements; synonym edges→cosine>0.8; context edges→node-to-file; OpenIE/NER not used (tree-sitter + regex/fuzzy symbol match). | 1E:L26-L35 |
| P1E-E005 | E/CAP | A-RAG adaptation: keyword_search→BM25+snippets; semantic_search→dual-vector+snippets; chunk_read→disk file/line range; agent loop→ReAct with ollama/external LLM; one tool per step (no parallel tool calls). | 1E:L37-L43 |
| P1E-E006 | E/FLOW | Canonical stage order (knowledge.txt L104-105): BM25+SPLADE(RRF) → dual-vector rerank → PageRank filter → CatRAG expand → A-RAG filter → cross-encoder rerank → token-budget fit → context. Cross-encoder is Phase 1G; token-budget is Phase 3E; this phase stubs both. | 1E:L45-L49, L375-L377 |
| P1E-E007 | E/FAIL | R30: limit CatRAG LLM to N_seed=5, K_edge=15; cache scores. R31: if density <0.01 skip CatRAG and pass PageRank→A-RAG; add synonym edges >0.8. R32: A-RAG max_loops=15 + token budget + context tracker. R33: snippets ±2 code lines. R34: measure per-layer latency. | 1E:L51-L62 |
| P1E-E008 | E/CFG | CatRagConfig defaults: damping=0.5, epsilon=0.2, beta=2.5, n_seed=5, k_edge=15, min_graph_density=0.01. | 1E:L76-L83 |
| P1E-E009 | E/DATA | CatRagOverlay is sparse only: `reset_distribution: Vec<(usize,f64)>` and `edge_weight_deltas: Vec<(usize,usize,f64)>`; never full adjacency matrix copies. | 1E:L84-L87, L157-L159, L252 |
| P1E-E010 | E/CAP | CatRag API: `new(config)`, `compute_overlay(query, snapshot, pagerank_scores) -> Result<CatRagOverlay>`; modifies PPR inputs without mutating base graph. | 1E:L88-L97, L265-L269 |
| P1E-E011 | E/CFG | AragConfig defaults: max_loops=15, max_token_budget=128000, top_k=5. | 1E:L100-L104 |
| P1E-E012 | E/CAP | RetrievalTool enum: KeywordSearch{keywords,top_k}, SemanticSearch{query,top_k}, ChunkRead{chunk_ids}. RetrievalResult: chunk_id, snippet, score, tool_used. | 1E:L105-L115 |
| P1E-E013 | E/STATE | ContextTracker holds per-query HashSet of read_chunks; created fresh each query_pipeline call; not shared/concurrent. | 1E:L119-L121, L160-L161, L310-L313, L333-L334 |
| P1E-E014 | E/CAP | AragOrchestrator: `new`, `execute_tool`, `query_pipeline` (full BM25→vectors→PageRank→CatRAG→A-RAG wiring). | 1E:L116-L134 |
| P1E-E015 | E/CAP | QueryPipeline holds bm25 + catrag + arag (+ vector/pagerank from prior phases); `query` returns `Vec<PipelineStage{stage,results,latency_ms}>`. Modules: catrag, arag, pipeline wired in knowledge lib.rs. | 1E:L135-L150 |
| P1E-E016 | E/TEST | 1E.1 PRE_GATE: 1D tests ok; DkgSnapshot, Bm25Index, pagerank symbols present. DETERMINISTIC: cargo check + three pub mod greps. | 1E:L68-L72, L152-L156 |
| P1E-E017 | E/TEST | 1E.3 CatRAG tests: symbolic anchoring weight formula epsilon/|files|; dynamic edge weights only from seeds; key-fact boost *(1+beta); density guard empty overlay; phi tiers 0-3→0, 4→0.2, 5→0.25, 6→0.3, 7→2.0, 8→2.5, 9→3.0, 10→5.0. | 1E:L184-L194 |
| P1E-E018 | E/TEST | 1E.3 A-RAG tests: keyword score sum(count*\|keyword\|) snippets only; semantic oversample 3× aggregate max score; chunk_read second hit returns "(previously read)"; tracker monotonic; token budget stops with partial. | 1E:L196-L205 |
| P1E-E019 | E/TEST | 1E.3 pipeline tests: stages order bm25/vector/pagerank/catrag/arag; latency_ms>0; empty query no panic. Contracts: overlay apply/revert; RetrievalResult fields non-empty. All new tests fail on stubs. | 1E:L207-L223 |
| P1E-E020 | E/FLOW | Symbolic anchoring: split whitespace/camelCase/snake_case; case-insensitive substring fuzzy match; reset_weight=epsilon*\|files_containing\|^(-1); combine with PageRank seeds; normalize sum=1; no LLM. Cap matches at 2*N_seed; warn if >50% DKG matched. | 1E:L232-L238, L279-L280 |
| P1E-E021 | E/FLOW | Dynamic edge weighting: top N_seed by reset; prune neighbors by embedding similarity if >K_edge; heuristic scores by edge type calls=8 implements=7 references=5 imports=3 + target relevance + file proximity; map via phi; LLM scoring optional via circuit breaker only. | 1E:L240-L252 |
| P1E-E022 | E/FLOW | Key-fact enhancement: T_seed = query symbols + direct callees; boost symbol→file context edges by *(1+beta). Pure algorithmic. | 1E:L254-L258 |
| P1E-E023 | E/FAIL | Density = \|edges\|/(\|nodes\|*(\|nodes\|-1)); if < min_graph_density return empty overlay; log "CatRAG skipped: graph density {d} below threshold {t}". | 1E:L260-L263, L367 |
| P1E-E024 | E/CAP | Keyword tool: merge BM25 per keyword; snippet sentences with keyword ±2 lines; top_k snippets not full chunks. Semantic: vector query, 3× oversample, max-score per file. Chunk read: full content or previously-read placeholder. | 1E:L290-L308 |
| P1E-E025 | E/FLOW | Phase 1E A-RAG orchestration is deterministic fixed pipeline keyword→semantic→chunk_read; full ReAct LLM tool choice deferred to Phase 2+ reasoning crate. | 1E:L320-L324 |
| P1E-E026 | E/FLOW | Pipeline stages: (1) BM25 top_k=20 after stopword filter; (2) vector + RRF k=60 with BM25, skip if breaker; (3) PageRank importance filter; (4) CatRAG overlay re-PPR expand; (5) A-RAG snippet filter; (6–7) stubs stage "rerank" and "token_fit". | 1E:L343-L379 |
| P1E-E027 | E/FAIL | Stage failures wrap Result, log stage name, fall through. RRF ranks start at 1 (k=60) to avoid NaN. Optional stages degrade gracefully. | 1E:L385-L395 |
| P1E-E028 | E/OBS | measure_1e.rs indexes .elai_cc crates; 5 queries Q1–Q5; metrics phase_1E.toml: pipeline_p50_ms, pipeline_p99_ms, token_savings_mean_pct, catrag_overlay_size, arag_tools_per_query, stages_skipped. Targets p50<2000ms, savings>50%. | 1E:L397-L435 |
| P1E-E029 | E/TEST | Exit: full pipeline operational; savings>50%; latency<2s; graceful optional-layer skip; cargo test -p knowledge --lib 21+ green. | 1E:L438-L443 |
| P1E-E030 | E/DEP | Modules under `.elai_cc/crates/knowledge/src/{catrag,arag,pipeline}.rs`; PRE_GATE greps DkgSnapshot, Bm25Index, pagerank. | 1E:L69-L72, L74-L75, L98, L135 |
| P1E-I001 | I/DEFER | "Phase 2+" for ReAct A-RAG means reasoning-crate agent loop ownership; tools themselves still ship in 1E. | 1E:L320-L324 |
| P1E-A001 | A/CFG | SPLADE appears in knowledge.txt order but 1E pipeline stage list does not implement a dedicated splade stage (1H owns SPLADE). | 1E:L45-L49, L207-L209 |
| P1E-A002 | A/DET | "Heuristic scoring" formula for combining edge type + relevance + proximity into 0–10 before phi is not fully specified. | 1E:L245-L248 |
| P1E-C001 | C/OBS | Goal line says "sub-second" while R34 and exit criteria bind interactive budget to 2 seconds; treat 2s p50 as the hard acceptance, sub-second as aspirational. | 1E:L11, L61-L62, L411, L433, L441 |

Category coverage: capabilities/pipeline ✓; workflows/stages ✓; schemas/config ✓; flags/defaults ✓; integrations/BM25-vector-PageRank ✓; security/privacy none named; failures/degradation ✓; determinism (no hot-path LLM anchoring) ✓; observability/metrics ✓; tests/TDD ✓; constraints/deferred ReAct+1G+3E ✓. No formal CLI surface beyond measure binary.

### 2B — grounding verification

Source: `docs/specs/guide/steps/phase-2B.txt` (580/580 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| P2B-E001 | E/STATUS | Phase 2B implements verification methods that resolve claims against real sources. Prerequisite: Phase 2A (9+ GRC tests pass). | 2B:L1-L2 |
| P2B-E002 | E/CAP | Six methods: FILE_CHECK, SOURCE_CHECK, TEST_CHECK, WEB_CHECK, API_CHECK, API_REGISTRY_CHECK. Each returns binary pass/fail + evidence string (no confidence scores). | 2B:L6-L14, L44-L54 |
| P2B-E003 | E/FLOW | Plan grounding sweep before EXECUTING: classify claims GROUNDED / VERIFY_NEEDED / UNGROUNDED; verify VERIFY_NEEDED; block if any UNGROUNDED remain. Blocks PLANNING→EXECUTING. | 2B:L9-L18, L537-L543 |
| P2B-E004 | E/RECOVERY | Uncertain (2A) claims auto-promoted via FILE_CHECK/SOURCE_CHECK to Verified/High when resolvable; unresolved stay Uncertain and are flagged (rejection policy is 2C per-agent). | 2B:L20-L23, L463-L470 |
| P2B-E005 | E/DEFER | Phase 4A wraps verification behind IPC GroundingRequest; 2B is direct calls structured for later wrap without trait signature change (`&self`, stateless). | 2B:L25-L27, L141-L143 |
| P2B-E006 | E/DEP | Mandatory reads: reasoning.txt §§4–5,8–11; architecture-overview §4 stubs; phase-2A.1 stubs; tools.txt for file ops. | 2B:L30-L39 |
| P2B-E007 | E/DATA | VerificationResult {claim, passes, evidence, method_used: VerificationMethod}. GroundingResult {claim, method: VerificationMethod, level, source, verified_at:u64 TimeSource}. | 2B:L44-L45, L75-L80, L167-L175 |
| P2B-E008 | E/DATA | PlanGroundingResult {claims, all_grounded, ungrounded, verify_needed}. verify_needed = training: claims that could not auto-resolve; subset of ungrounded; all_grounded = ungrounded.is_empty(). | 2B:L96-L104, L176-L186 |
| P2B-E009 | E/CAP | VerificationMethod enum + requires_network() (Web/Api/ApiRegistry=true) + requires_tools() (File/Source/Test=true). | 2B:L67-L73, L157-L161, L339-L341 |
| P2B-E010 | E/CAP | Verifier trait object-safe Send+Sync: verify(method,claim,context)->Result; supported_methods(). TestRunner: run_test(name,crate)->TestRunResult{passed,output,duration_ms}. | 2B:L119-L129, L189-L202 |
| P2B-E011 | E/CAP | CargoTestRunner {sandbox BashSandbox, timeout_ms} production; MockTestRunner {results HashMap} for tests. LocalVerifier::new(Box<dyn TestRunner>). NetworkVerifier stubs. | 2B:L124-L128, L200-L212 |
| P2B-E012 | E/INT | Breaking trait changes from 2A: verify_claim gains context param; ground_plan returns PlanGroundingResult not Vec. Stubs were unimplemented!() so safe. | 2B:L96-L116, L203-L208 |
| P2B-E013 | E/DEP | reasoning/ gains tools/ dependency for FILE_CHECK/SOURCE_CHECK. Must not create tools→reasoning cycle; transitive knowledge via tools must not appear in cargo tree. Allowed deps: core, tools, thiserror, serde. | 2B:L82-L94, L132-L135, L209, L257-L276 |
| P2B-E014 | E/FAIL | GrcError::VerificationError {method, claim, reason}. Network methods return typed VerificationError not panic. | 2B:L187-L188, L423-L430, L483-L485 |
| P2B-E015 | E/TEST | ≥16 verification_tests: existing/missing file; source exist/missing; mock test; web/api/registry stubs; plan block/pass/empty; uncertain auto-resolve; requires_network; dyn Verifier; adversarial regex comment vs real claim; strip file: prefix. | 2B:L286-L388 |
| P2B-E016 | E/LIMIT | Unit tests ban network (no reqwest/hyper/TcpStream) and ban sleep/Duration/Instant timing. Network gated #[cfg(feature="network")] integration only. | 2B:L136-L138, L372-L375 |
| P2B-E017 | E/CAP | FILE_CHECK: tools::read_file after stripping "file:"/"grep:" prefix; evidence with path+lines or not found. SOURCE_CHECK: read_file + regex `(fn\|struct\|enum\|trait\|impl\|const\|static\|type\|mod)\s+{name}\b`; claim=symbol, context=path. | 2B:L398-L412, L436-L442 |
| P2B-E018 | E/CAP | TEST_CHECK: CargoTestRunner BashSandbox::execute `cargo test --exact {name} -p {crate}` timeout 30s; pass on exit 0; fail on nonzero/timeout. | 2B:L414-L421 |
| P2B-E019 | E/FLOW | verify_claim prefix map: file:/grep:→FileCheck; test:→TestCheck; search:→WebCheck; tool:→SourceCheck; training:→no verify passes=false Uncertain. Prefix always stripped before Verifier. | 2B:L432-L446 |
| P2B-E020 | E/FLOW | ground_plan: regex extract type:reference {file,grep,test,search,tool,training}; verify each; training tries FILE/SOURCE auto-resolve; empty plan trivially all_grounded=true. verified_at from TimeSource. | 2B:L448-L461, L139-L140 |
| P2B-E021 | E/OBS | Metrics `.elai_cc/metrics/phase_2B.toml`: file_check_latency_us <2000; source_check_latency_us <10000 (1000-line file); plan_sweep_latency_us <100000 (20 claims); methods_implemented=3/6. | 2B:L503-L534 |
| P2B-E022 | E/LIMIT | Zero todo!/unimplemented!/unwrap()/panic! in verification.rs; network stubs typed errors only. | 2B:L477-L500, L547 |
| P2B-E023 | E/EVIDENCE | Binary checks grounded MiniCheck EMNLP 2024 + Vertex AI; structural decomposition vs RARR/SAFE LLM; 6 methods reasoning.txt §4; tools dep crate map. | 2B:L50-L52, L61-L64, L551-L580 |
| P2B-I001 | I/SEC | Send+Sync on Verifier is inferred from 4A process isolation IPC, not explicit reasoning.txt bound. | 2B:L245-L249, L571-L574 |
| P2B-A001 | A/DATA | Exact regex for plan claim extraction (context vs free-text "file:") is required by T15 but pattern body not fully normative beyond type:reference set. | 2B:L349-L355, L452-L454 |
| P2B-A002 | A/FLOW | SOURCE_CHECK "looks like a path" vs "looks like a symbol" auto-resolution heuristic for Uncertain claims is not fully formalized. | 2B:L463-L469 |
| P2B-C001 | C/DEFER | Network stub message wording varies ("not implemented in phase 2B" vs summary "not impl until Phase 4A IPC"); behavioral requirement is typed error without network, not exact string. | 2B:L423-L430 |

Category coverage: capabilities/verification ✓; workflows/plan sweep ✓; schemas/types ✓; flags/feature network ✓; integrations/tools BashSandbox ✓; security integrity ✓ (no privacy policy); failures/typed errors ✓; determinism/TimeSource ✓; observability/metrics ✓; tests 16+ ✓; deferred network+IPC ✓.

### 2C — per-agent GRC configuration

Source: `docs/specs/guide/steps/phase-2C.txt` (496/496 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| P2C-E001 | E/STATUS | Phase 2C: per-role GRC budgets, step sequences, grounding constraints, allowed methods. Prerequisite: Phase 2B. | 2C:L1-L2 |
| P2C-E002 | E/CAP | Engine routes AgentGrcConfig at spawn. Layered config: [grc.defaults] + [agents.{role}.grc] field-by-field override (not deep merge). Underscore role keys (context_compiler) not hyphens. | 2C:L7-L12, L54-L66, L209-L210 |
| P2C-E003 | E/DATA | AgentGrcConfig 8 fields: role, budget, required_steps, grounding_level, csq_required, allowed_methods, transition_overrides, require_scope_enumeration. | 2C:L111-L125, L162-L164, L468-L469 |
| P2C-E004 | E/FLOW | D1 transition_overrides additive HashMap only (union with allowed_next, never remove). Auditor needs Verify→[Verify,Observe] for double-verify. | 2C:L28-L40, L321-L326, L448 |
| P2C-E005 | E/CFG | D2 validate at load: (a) required_steps non-empty; (b) valid chain via merged transitions; (c) max_calls ≥ len(required_steps); (d) csq_required ⇒ ≥1 Verify; (e) allowed_methods ⊆ known VerificationMethod. Err ConfigValidation. | 2C:L42-L52, L312-L319, L452 |
| P2C-E006 | E/SEC | allowed_methods is POLICY layer (what role may request) ≤ CAPABILITY layer (IPC proxy in 4A). Context-compiler policy FILE_CHECK+SOURCE_CHECK only; executor/auditor all methods. 2C enforces in-process; 4A adds second IPC layer. | 2C:L68-L84 |
| P2C-E007 | E/FAIL | Anti-rubber-stamp D5: auditor Converge requires ≥1 Verify step with FINDING: prefix (case-sensitive after trim); else InsufficientFindings {role, expected_min:1}. Const FINDING_PREFIX. | 2C:L86-L97, L350-L358, L450 |
| P2C-E008 | E/DATA | require_scope_enumeration: Observe ThinkStep.scope must list components; orchestrator checks plan_action coverage; unaddressed → WARN. Default true executor/designer, false memory_writer. | 2C:L120-L125, L279-L282, L455 |
| P2C-E009 | E/CAP | GrcError::InsufficientFindings, ConfigValidation. AgentGrcConfigLoader load_config/validate_config. GrcDefaults {budget, grounding_level, csq_required, allowed_methods}. Engine with_agent_config / get_agent_config. | 2C:L126-L136 |
| P2C-E010 | E/LIMIT | Role stored as String not enum; validate known roles at load. No toml crate in reasoning; receives typed structs from core/orchestrator. | 2C:L152-L156, L182-L188 |
| P2C-E011 | E/TEST | ≥15 agent_config tests T1–T15: auditor double-verify; context_compiler observe-only; executor 8/1200; memory_writer 2/400; empty steps; budget too small; rubber-stamp; Uncertain reject; allowed_methods filter; defaults+override; additive transitions; auditor 10/1200; prompt_engineer 5/800; designer distinct; scope enum per role. | 2C:L198-L302 |
| P2C-E012 | E/FLOW | effective_allowed_next = (base ∪ overrides) ∩ (required_steps ∪ {Status, Converge}). Whitelist makes observe-only actually block Hypothesize. | 2C:L321-L335 |
| P2C-E013 | E/STATE | GroundingLevel PartialOrd must be Uncertain < High < Verified (discriminant order or explicit impl). Reject think if level < config.grounding_level. Phase 2A order {Verified,High,Uncertain} is inverted and must be fixed. | 2C:L337-L348, L405-L406, L488-L491 |
| P2C-E014 | E/FLOW | with_agent_config stores config and resets topic state (no budget leak across roles). Missing role table → defaults only; malformed table → error not silent fallback. | 2C:L360-L372, L399-L404 |
| P2C-E015 | E/STATE | Budget authority: 2C in-process GrcEngine is AUTHORITATIVE through Phases 0–3; Phase 4A demotes agent-side to advisory and orchestrator enforces (orchestrator.txt L440-444). | 2C:L373-L380, L471 |
| P2C-E016 | E/OBS | Metrics phase_2C.toml: config_load_us <1000; routing_overhead_pct <10; memory per config <2KB including overrides. | 2C:L414-L442 |
| P2C-E017 | E/SURFACE | TOML surfaces: `[grc.defaults]`, `[agents.executor.grc]`, `[agents.auditor.grc]`, `[agents.context_compiler.grc]`, `[agents.memory_writer.grc]`, `[agents.prompt_engineer.grc]`, designer role table; pattern matches existing supervision/slo_tier/sandbox/role_prompt. | 2C:L54-L66, L23, L445-L447 |
| P2C-E018 | E/CAP | Six roles including Designer: same model as executor possible, distinct role_prompt/required_steps/output PipelineSpec. | 2C:L270-L277, L446-L447, L470 |
| P2C-E019 | E/LIMIT | Zero todo!/unimplemented!/unwrap() in engine.rs and config.rs after implement. | 2C:L387-L411, L457 |
| P2C-E020 | E/EVIDENCE | Spec gaps flagged for reconciliation: AgentGrcConfig needs 3 extra fields in reasoning.txt L186; GrcError missing variants; GroundingLevel order; invariant for rubber-stamp; allowed_methods vs grounding_level complementary docs. | 2C:L480-L496 |
| P2C-I001 | I/DEP | "Status" always-allowed alongside Converge implies Status is a GRC step type reachable even when not listed in required_steps (from 2A enum). | 2C:L327-L335 |
| P2C-A001 | A/DATA | Exact FINDING marker structured type deferred; prefix contract is interim internal protocol. | 2C:L294-L296, L394-L398 |
| P2C-A002 | A/CFG | Full authoritative TOML tables for all six roles are test-described but not listed as complete sample config block in this phase file. | 2C:L198-L277 |
| P2C-C001 | C/DATA | reasoning.txt still documents 4-field AgentGrcConfig while 2C requires 8; phase flags update, does not edit owned spec. | 2C:L82-L84, L468-L469, L481-L484 |

Category coverage: capabilities/config routing ✓; workflows/whitelist transitions ✓; schemas/TOML ✓; surfaces/role tables ✓; security policy methods ✓; failures/InsufficientFindings ✓; determinism PartialOrd ✓; observability/metrics ✓; tests 15+ ✓; deferred 4A dual-enforce ✓.

### 3L — Project Intelligence (conventions + Pi tier hooks + telescope)

Source: `docs/specs/guide/steps/phase-3L.txt` (251/251 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| P3L-E001 | E/STATUS | Phase 3L extracts project coding conventions; surfaces ConventionStats/NamingConventions; wires PiT1/PiT2/PiT3 hooks into prompt construction; integrates context telescope; latency p50≤50ms / p99≤100ms. | 3L:L1-L10 |
| P3L-E002 | E/DEP | Predecessor phase-3K is RAG production hardening (adaptive chunking, query expansion, HyDE, quality tiers), not a "Tier0 prompt scaffold". | 3L:L33-L36, L70-L71 |
| P3L-E003 | E/CAP | D1: hooks live in elai_knowledge::project_intelligence as PiT1Hook/PiT2Hook/PiT3Hook to avoid collision with elai_plugins::tiered_analysis T1/T2/T3. Reuse HookVerdict + TieredAnalysisResult (no fork). | 3L:L12-L17, L207-L208 |
| P3L-E004 | E/CFG | D2: regex via std::sync::LazyLock (Rust 1.80+), not once_cell::Lazy. Static-init .expect() allowed for literal patterns (CLAUDE.md S1). | 3L:L18-L20, L86-L89, L218-L219 |
| P3L-E005 | E/DATA | D3: conventions migration at knowledge/src/conventions_migration_v1.sql (flat sibling to metadata_migration_v2.sql, not migrations/ subdir). Separate conventions.db. | 3L:L21-L23, L94-L101 |
| P3L-E006 | E/DATA | D4: NamingConventions single surface from elai_core::project_profile; extract_conventions returns ConventionStats with naming field; no PiNamingConventions fork. | 3L:L24-L27, L79-L80 |
| P3L-E007 | E/OBS | D5: ProjectProfile snapshot at `.elai_cc/metrics/project_profile.toml` via persist_profile; reloaded at session start by `/profile` slash command (cli Profile variant). | 3L:L28-L30, L148-L151, L212 |
| P3L-E008 | E/CAP | D6: ProjectRuleSeeder API is seed_rules (not seed); PiT2Hook::run calls it. | 3L:L31-L32, L129-L131 |
| P3L-E009 | E/CAP | extract_conventions(&CorpusView)->ConventionStats; CorpusView holds Option<&[DkgNodeInfo]> + project_root + primary_language; DKG path if nodes non-empty else regex fallback. CONVENTIONS_SCHEMA_VERSION:u32=1. | 3L:L81-L91 |
| P3L-E010 | E/DET | extract_conventions byte-identical on repeat (sorted walker + determinism_matrix for Rust/Python/TypeScript/Svelte). Expected majority styles: snake_case, snake_case, camelCase, PascalCase. | 3L:L102-L113 |
| P3L-E011 | E/RECOVERY | PiT2 VERSION_GATE: if schema stale, execute_batch embedded migration (IF NOT EXISTS) and return SchemaTooOld for re-run. | 3L:L98-L101, L132-L135 |
| P3L-E012 | E/SURFACE | PiT1Hook::load + ProjectIntelligenceHook::render_block → CONVENTIONS block for SYSTEM via build_with_conventions. Format: per-category summaries + error_crate + top-3 avoid anti-patterns. Cap PI_T1_TOKEN_BUDGET=200 tokens (800 chars); truncate at line boundary. | 3L:L115-L126, L200-L201 |
| P3L-E013 | E/CAP | PiT2Hook::run seeds rules.db; evaluate Ok(None) on unrecognised language OR synthesised line count > PI_T2_CIRCUIT_BREAKER_LINES (mirrors plugins T2). | 3L:L128-L140, L221 |
| P3L-E014 | E/STATE | ProjectProfile aggregation after full reindex via populate_tier2_conventions + populate_tier3. PiT3Hook::evaluate ADVISORY ONLY — never HookVerdict::Block. | 3L:L142-L153, L220 |
| P3L-E015 | E/FLOW | assemble_telescope_with_hooks(profile, hooks)->(TelescopeContext, Vec<String>); priorities PI_T1=100 > PI_T2=80 > PI_T3=60; ties by hook id; TELESCOPE_MAX_CHARS=1200 drop whole blocks. Legacy assemble_telescope preserved. | 3L:L155-L167 |
| P3L-E016 | E/TEST | pi_telescope_contract: (a) top-K stable across 10 repeats; (b) T1>T2>T3; (c) per-hook budget; (d) PiT3 never Block; (e) PiT2 unrecognised language Ok(None). | 3L:L168-L174 |
| P3L-E017 | E/OBS | calibrate_phase_3L.py stdlib-only; 1000-file multi-lang corpus; emits phase_3L.toml; gates p50≤50ms p99≤100ms; floor 80% of gates for auditor. Fallback: reduce regex, shard by language, or in-process bench. | 3L:L176-L186 |
| P3L-E018 | E/INT | Reachable from prompt_builder build_with_conventions, /profile, telescope ordering; consumes rules.db seeder, conventions.db migration, HookVerdict from plugins. | 3L:L189-L214 |
| P3L-E019 | E/LIMIT | stub_ban: no .unwrap() outside tests or LazyLock init; regex only LazyLock; PiT3 never Block; PiT2 Ok(None) language/breaker. | 3L:L215-L221 |
| P3L-E020 | E/TEST | Observable: cargo test -p elai-knowledge conventions:: and elai_cc_contracts pi_telescope_contract; 4 language determinism + 5 telescope invariants; phase_3L.toml written. | 3L:L222-L227 |
| P3L-E021 | E/DET | DETERMINISTIC_CHECK rg list: extract_conventions, CorpusView, CONVENTIONS_SCHEMA_VERSION, PiT*Hook, ProjectIntelligenceHook, migration, persist_profile, assemble_telescope_with_hooks, pi_telescope_contract, build_with_conventions, calibration files. | 3L:L229-L251 |
| P3L-I001 | I/SURFACE | `/profile` is the primary operator CLI surface for reloading ProjectProfile; CONVENTIONS block is agent-visible prompt surface not a dashboard panel. | 3L:L148-L151, L119-L121 |
| P3L-A001 | A/CFG | Exact numeric value of PI_T2_CIRCUIT_BREAKER_LINES is not stated (only that it mirrors plugins T2). | 3L:L137-L140 |
| P3L-A002 | A/DATA | Full conventions.db SQL column schema beyond "migration_v1 + meta schema_version" is not expanded in this phase text. | 3L:L94-L97, L45-L46 |

Category coverage: capabilities/hooks ✓; surfaces /profile + CONVENTIONS prompt ✓; workflows/telescope ✓; schemas/migration ✓; failures SchemaTooOld ✓; determinism multi-lang ✓; observability calibration ✓; tests contract ✓; constraints Pi naming ✓. No privacy/compliance section.

### 4C — plugins hooks + tiered static analysis

Source: `docs/specs/guide/steps/phase-4C.txt` (380/380 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| P4C-E001 | E/STATUS | Phase 4C: invisible PreToolUse/PostToolUse/Stop/PreCompact hooks in orchestrator; tiered static analysis on SSB writes. Prerequisite: 4B tools via IPC. Agents never see hook existence. | 4C:L1-L7 |
| P4C-E002 | E/DEFER | Skills system and MCP bridge out of scope (skills=prompt templates no code path; MCP=1751 LOC claw-code later sub-phase). | 4C:L8-L10, L376-L380 |
| P4C-E003 | E/FLOW | Under Tier C: PreToolUse on ToolRequest/BashRequest before executor; PostToolUse after tool before ToolResult to agent; PostToolUse also ShortCoder code_simplify on AgentResponse before auditor view FS; Stop on session end/AgentKill; PreCompact when token_used/token_limit >0.85. | 4C:L22-L29, L258-L261 |
| P4C-E004 | E/SEC | Error sanitization: Block → generic IPC "Tool call rejected by policy" / "tool call rejected"; strip rule_id, pattern, severity, trading.* — agent must not see PreToolUse BLOCK from rule trading.14. Sanitize at HookVerdict enum mapping not string scrub only. | 4C:L34-L36, L66, L123-L129, L262-L263, L294-L296 |
| P4C-E005 | E/CAP | T1: predicate DSL from rules.db — AST_PATTERN, GREP_PATTERN, METRIC_GATE, COMPOSITE, SCHEMA_REQUIRED + phase_mask. T1 does not parse AST (T2 does). SCHEMA_REQUIRED → Escalate{target=ProviderTier} not ALLOW/BLOCK. Indexed SELECT by phase_mask & domain. Cache per phase; invalidate SHADOW→ACTIVE at next BSP superstep boundary. | 4C:L37-L41, L265-L272 |
| P4C-E006 | E/FLOW | T3 fires only when WARN exists AND no BLOCK from any tier. If T1=BLOCK and T2=WARN, T3 does not fire. Pattern-match only, no LLM. | 4C:L42-L43, L136-L137, L276-L278 |
| P4C-E007 | E/CFG | ruff PostToolUse is subprocess (~50–200ms); no public Rust API (astral-sh/ruff #12351). Circuit breaker skip ruff files >10000 lines. PostToolUse total target <500ms. | 4C:L44-L46, L251-L252, L337 |
| P4C-E008 | E/CAP | HookTrait methods pre_tool_use/post_tool_use/stop/pre_compact. HookVerdict Allow \| Block{reason} \| Warn{reason}. HookContext: fsm_state, agent_id, agent_role, session_id, active_rules. | 4C:L58-L61, L81 |
| P4C-E009 | E/CAP | TieredAnalysis T1/T2/T3 entry points. T2 tree-sitter syntax, import deny-list, dead/lazy code; circuit breaker >5000 lines → T1 only. T3 examples: fn>200 lines WARN:complexity; catch Exception:pass BLOCK:swallow. | 4C:L62-L69, L273-L280 |
| P4C-E010 | E/EVENT | BLOCK from any tier → SSB finding entry via single-writer channel. Orchestrator normalizes findings to blake2s pattern_hash (not auditor). PROPOSED rules source=AUDITOR never auto-promote; finding-source auditor cannot approve own findings. | 4C:L30-L33, L140-L148, L281-L285, L309-L310 |
| P4C-E011 | E/TEST | 30 contract tests plugins_hooks: generic block error; allow forward; ruff syntax block; stop IDLE gate; PreCompact checkpoint; sanitization; T1<1ms every SSB write; GREP pass-in-except BLOCK; T2 only file_write; T2<10ms <5000; T2 breaker >5000; T3 only WARN; T3<5ms; complexity/swallow patterns; SSB finding; IPC Tool/Bash PreToolUse; BLOCK prevents execution; normalization; AUDITOR no auto-promote; independent validation; cross-session lock write block/allow; SCHEMA_REQUIRED Escalate; T3 compound no-fire; ShortCoder placement + AST-equivalent. | 4C:L120-L175 |
| P4C-E012 | E/TEST | 10 adversarial: path traversal Unix+Windows; FSM-gated tool; independent Bash after Tool block; 1000-rule T1 still <1ms indexed; malformed predicate WARN; multi-WARN process all; T3 not on BLOCK; reason injection strip; same-auditor validate reject; AUDITOR flood stuck PROPOSED. | 4C:L197-L234 |
| P4C-E013 | E/SEC | PreToolUse loads get_active_rules for FSM state+domain; first BLOCK wins; cross-session file lock via SSB. Stop: non-IDLE Block, IDLE Allow. PreCompact: checkpoint before truncation. | 4C:L247-L257 |
| P4C-E014 | E/LIMIT | plugins depends on core/ + rules/ only; no plugins→orchestrator imports (orchestrator→plugins OK). Zero todo!/unimplemented!/pass bodies after implement. | 4C:L103-L110, L290, L304 |
| P4C-E015 | E/OBS | Metrics phase_4C.toml: T1/T2/T3 p50/p95/p99; hook overhead per IPC tool call; PostToolUse incl ruff; compile delta; test count; LOC; coverage. Targets T1 p99<1ms; T2 p99<10ms <5000 lines; T3 p99<5ms; hook overhead <2ms non-write; PostToolUse p99<500ms; coverage >80%; compile +<10s vs 4B. | 4C:L321-L347 |
| P4C-E016 | E/INT | RuntimeClient wiring: every routing call site/struct must wire ssb, fsm, clock; no None after construction. | 4C:L360 |
| P4C-E017 | E/DEP | 4A WARN mitigations: PostToolUse analysis is T1/T2/T3 not GRC WEB_CHECK; hooks synchronous before IPC response so GRC convergence cannot mid-cut; ShortCoder placement resolved tests 29-30. | 4C:L362-L374 |
| P4C-E018 | E/SURFACE | No agent-visible UI for hooks; operator-facing artifacts are SSB findings, metrics TOML, generic rejection errors on agent channel. | 4C:L4-L5, L355-L359 |
| P4C-I001 | I/CONC | T1 latency test note prefers Criterion benches over inline Instant in contracts; adversarial wall-clock Instant still listed as monotonic (not SystemTime). | 4C:L166-L167, L176 |
| P4C-A001 | A/CFG | Exact deny-list contents for T2 imports and lazy-code patterns beyond examples not fully enumerated. | 4C:L273-L275 |
| P4C-C001 | C/OBS | Adversarial note "ruff should use library binding" conflicts with decided subprocess + circuit breaker when binding unavailable; subprocess path is authoritative. | 4C:L44-L46, L297-L300 |

Category coverage: capabilities/hooks/tiers ✓; workflows/IPC intercept ✓; schemas/verdicts ✓; security sanitization+circular trust ✓; failures BLOCK/WARN ✓; concurrency BSP cache boundary ✓; observability metrics ✓; tests 40 ✓; deferred skills/MCP ✓. No formal privacy policy beyond agent isolation opacity.

### 4I — conditional media validation + Render-Capture-Evaluate

Source: `docs/specs/guide/steps/phase-4I.txt` (623/623 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| P4I-E001 | E/STATUS | Phase 4I conditional: implement only if pipeline tasks produce media. Prerequisites: 4A isolation. Validates images, 3D, audio, UI screenshots. Tier1 deterministic veto; Tier2 VLM advisory. | 4I:L1-L11 |
| P4I-E002 | E/FAIL | R15: VLM never auto-overrides Tier1; disagree → human review. R-NEW-1 tool availability degrade. R-NEW-2 GPU swap batch at phase boundary. R-NEW-3 Mode B requires SGLang. R-NEW-4 MoE VRAM stores all params. R-NEW-5 SSIM/LPIPS CPU EP + epsilon 1e-6. | 4I:L19-L39 |
| P4I-E003 | E/DATA | MediaValidationRequest: media_type Image\|Model3D\|Audio\|UiScreenshot; file_path; quality_spec; reference_path; mode Quick\|Full. Result: tier1, tier2 Option, final_verdict Pass\|Fail\|HumanReview\|Degraded, reason. | 4I:L72-L82 |
| P4I-E004 | E/CAP | Tier1Validator trait + Image/Model3D/Audio/UiScreenshot validators. ExternalToolWrapper check_available+execute timeout; ImageMagick/FFprobe/Tesseract/Blender wrappers. ToolCapabilityFlags built at startup; missing tool → DEGRADED check not panic. | 4I:L83-L91 |
| P4I-E005 | E/CFG | Mode A: Qwen3.5-VL-30B-A3B ollama Q4_K_M ~17GB; needs ollama≥0.12.7; Phase0 12GB cannot run (MockVlm). Mode B: InternVL3.5-38B SGLang only RadixAttention; ~38GB 8-bit dual 3090 or ~21GB Q4. Hardware guard nvml-wrapper. | 4I:L92-L103, L48-L49 |
| P4I-E006 | E/CFG | CLAP audio: Xenova/clap-htsat-unfused ONNX; ~200M; FP16 ~0.4GB always-loaded when audio active; raw PCM input. | 4I:L104-L107, L271-L276 |
| P4I-E007 | E/DATA | VlmValidationOutput constrained: overall_quality f32[0,1], issues[], confidence, reasoning; Issue severity Critical\|Major\|Minor + category/description/location. Mode B SGLang schema; Mode A format=json + post-validation; retry once temp=0 then Degraded. | 4I:L108-L118 |
| P4I-E008 | E/TEST | Contracts: T1 before T2; T1 fail→Fail ignore T2; T2 only if T1 pass + qualitative checks; schema conform; only media PipelineSpec; SSIM epsilon; T2 disagree→HumanReview. ≥20 tests base; ≥8 adversarial; ≥15 RCE; total ≥43. | 4I:L172-L200, L231-L240, L495-L496, L551 |
| P4I-E009 | E/SEC | Adversarial: EXIF/overlay prompt injection still schema-bound + Tier1-based verdict; 500MB size reject pre-VLM; malformed headers no panic; path must canonicalize inside output dir (symlink reject); VLM no /proc or parent enum; queue 64 overflow QueueFull; max 4 Tier1 / 1 Tier2 concurrent; ComfyUI timeout 600s cleanup; CLAP noise low score deterministic. | 4I:L211-L240 |
| P4I-E010 | E/CAP | Image: identify format/resolution/colorspace; size; SSIM image crate CPU EP; LPIPS ONNX CPU EP. 3D: Blender headless gated BLENDER flag. Audio: FFprobe + RMS silence + peak clip integer math. UI: resolution + SSIM mockup + Tesseract OCR. | 4I:L253-L263 |
| P4I-E011 | E/FLOW | Veto matrix: T1 fail→Fail; T1 pass no qualitative→Pass; T1+T2 pass→Pass; T1 pass T2 fail→HumanReview; T1 pass T2 disagree→HumanReview; schema-invalid after retry→Degraded; VLM unavailable→Degraded. | 4I:L277-L287 |
| P4I-E012 | E/CAP | Lazy load: ToolRegistry::register_lazy; factory builds flags + optional VLM on first MediaValidationRequest; non-media PipelineSpec never loads. VLM uses 15-message postcard IPC like agents; startup_timeout default 30s. | 4I:L288-L294 |
| P4I-E013 | E/CONC | Tier1 max 4 concurrent; Tier2 max 1; queue depth 64 → Error::QueueFull. | 4I:L295-L298, L223-L225 |
| P4I-E014 | E/OBS | Metrics phase_4I.toml: tier1_p95_ms (<100 image/audio/ui; 3D <500); tier2 mode A/B; gpu_swap mode A <15s / B <60s; vram; test counts; later RCE fields rce_* render/loop/convergence. | 4I:L331-L362, L578-L602 |
| P4I-E015 | E/CAP | MediaRenderAdapter trait: can_handle, async render→CapturedFrame[], cleanup. Types: Viewport, FrameMetadata, RenderConfig, CaptureMode Screenshot\|KeyFrames\|MultiAngle, Interaction Click/Scroll/Wait/Hover/Type, CameraAngle. | 4I:L380-L395 |
| P4I-E016 | E/CAP | Adapters: WebRenderAdapter Playwright; BlenderRenderAdapter; AnimationRenderAdapter FFmpeg; UiScreenshotAdapter pass-through; ComfyUIRenderAdapter REST. AdapterRegistry by OutputType from ToolCapabilityFlags. | 4I:L396-L406, L513-L547 |
| P4I-E017 | E/FLOW | VisualFeedbackLoop: render→T1→T2→issues; stop on T1 fail, max_iterations default 3, or issues unchanged (compare severity+category+location not raw string). Feedback structured VisualIssue with actionable_fix to executor prompt. | 4I:L407-L416, L451-L456, L535-L542 |
| P4I-E018 | E/EVENT | SSB action_type='visual_feedback' per iteration {iteration, issue_count, resolved_count, verdict} so orchestrator knows agent alive. | 4I:L492-L494, L543-L544, L566 |
| P4I-E019 | E/SURFACE | Functional viewports web 375/768/1440; Blender 4 angles; FFmpeg keyframes; ComfyUI mock never real in tests. visual_check=false → no render overhead. | 4I:L449-L469, L458-L459 |
| P4I-E020 | E/SEC | VLM process isolation: no read outside allowed_paths; no write; no network. Path traversal reject. Tier C seccomp/RLIMIT from 4A. | 4I:L185-L188, L221-L222 |
| P4I-E021 | E/LIMIT | architecture-overview L283 stale LLaVA models; media-validator.txt L60 "Active params ~3B" misleading for VRAM. RuntimeClient ssb/fsm/clock wiring required. | 4I:L41-L46, L613 |
| P4I-E022 | E/STATUS | Phase 4 overall exit: arbitrary task design pipeline INTERACTIVE; AUTONOMOUS on top (4G); mode switch preserves state; isolated agents; no enforcement visibility; isolation escape test; overhead <5%; self-mod cannot touch evaluation criteria. | 4I:L615-L623 |
| P4I-E023 | E/DEP | PRE_GATE tools: ImageMagick+FFmpeg required; Tesseract/Blender optional degrade; Playwright for RCE; PhaseEntry output_type/visual_check from 3G; L2 MissingAdapter/InvalidVisualCheck from 4E. | 4I:L61-L70, L376-L378 |
| P4I-I001 | I/SURFACE | HumanReview/Degraded verdicts are operator-facing outcomes (review queue semantics) even though no dashboard panel is specified in this phase file. | 4I:L81, L277-L287 |
| P4I-A001 | A/CFG | Exact QualitySpec threshold field names for SSIM/LPIPS/CLAP not restated here (owned by pipeline-factory QualitySpec). | 4I:L16, L75 |
| P4I-A002 | A/CONC | Whether max_iterations is globally fixed or QualitySpec-overridable beyond default 3 is only partially specified (field exists on VisualFeedbackLoop). | 4I:L408-L409, L451 |
| P4I-C001 | C/EVIDENCE | architecture-overview media models conflict with media-validator Mode A/B; phase binds media-validator as authority and flags overview stale. | 4I:L41-L45 |

Category coverage: capabilities/validators/adapters ✓; surfaces verdicts+viewports+SSB visual_feedback ✓; workflows veto+feedback loop ✓; schemas VLM JSON ✓; flags ToolCapabilityFlags+lazy ✓; security isolation/path ✓; failures degrade/queue ✓; determinism CPU EP ✓; concurrency bounds ✓; observability metrics ✓; tests ≥43 ✓; deferred non-media skip ✓.

## Cross-file interaction records

| Interaction ID | Class | Interaction | Atom references |
|---|---|---|---|
| IX-001 | E | 1E composes 1B BM25 + 1C vectors + 1D PageRank/DKG; leaves explicit stubs for 1G rerank and 3E token_fit. | P1E-E001, P1E-E006, P1E-E026 |
| IX-002 | E | 1E density guard and circuit-breaker skips feed 1I resilience/mode cascade and QueryTrace stage list. | P1E-E007, P1E-E023, P1E-E027 |
| IX-003 | E | 2B replaces 2A unimplemented verify_claim/ground_plan stubs with breaking signatures consumed by 2C method policy. | P2B-E012, P2B-E019, P2C-E006 |
| IX-004 | E | 2B plan sweep supplies the ungrounded-zero evidence gate for 3A PLANNING→TESTING_FIRST / EXECUTING path. | P2B-E003, P2B-E020 |
| IX-005 | E | 2C allowed_methods and grounding_level layer on 2B VerificationMethod enum; 4A later dual-enforces via IPC. | P2C-E006, P2C-E015, P2B-E005 |
| IX-006 | E | 2C auditor double-verify + FINDING anti-rubber-stamp is last defense before acceptance; composes with 3A AUDITING evidence. | P2C-E004, P2C-E007 |
| IX-007 | E | 3L Pi hooks reuse plugins HookVerdict/TieredAnalysisResult shapes without forking 4C SSB-write T1/T2/T3 family. | P3L-E003, P4C-E009 |
| IX-008 | E | 3L CONVENTIONS injection via reasoning prompt_builder and /profile CLI; telescope priorities sit upstream of agent SYSTEM context. | P3L-E012, P3L-E015, P3L-E007 |
| IX-009 | E | 3L predecessor is 3K RAG hardening; convention signal is additive prompt input, not a replacement for 1E pipeline. | P3L-E002, P1E-E001 |
| IX-010 | E | 4C PostToolUse ShortCoder placement resolves 4B tools.txt code_simplify + 4A WARN-3 under Tier C before auditor FS. | P4C-E003, P4C-E017 |
| IX-011 | E | 4C T1 SCHEMA_REQUIRED Escalate to ProviderTier couples rules DSL to provider constrained decoding (outside 4C body). | P4C-E005, P4C-E011 |
| IX-012 | E | 4C circular-trust normalization + AUDITOR PROPOSED freeze composes with rules.txt self-discovery and 3F-style promotion gates. | P4C-E010 |
| IX-013 | E | 4I lazy media path depends on 4D MediaSpec / 3G visual_check+output_type and 4E adapter structural check #8. | P4I-E001, P4I-E012, P4I-E023 |
| IX-014 | E | 4I VLM process uses 4A Tier C isolation + same postcard IPC as agents; ToolCapabilityFlags degrade without panicking. | P4I-E012, P4I-E020, P4I-E004 |
| IX-015 | E | 4I VisualFeedbackLoop writes SSB visual_feedback so 3B journal and 3C/4G liveness observe long RCE iterations. | P4I-E018, P4I-E017 |
| IX-016 | E | Hash split in chunk: 4C violation_patterns blake2s vs knowledge blake2 elsewhere; 4I does not introduce new content hash for media. | P4C-E010 |
| IX-017 | E | Budget authority handoff 2C→4A mirrors hook authority: enforcement is orchestrator-side under Tier C (4C hooks never in agent binary). | P2C-E015, P4C-E001 |
| IX-018 | I | 1E measure binary and 2B/2C/3L/4C/4I phase_*.toml metrics share the implementation-guide "every phase measures" pattern for later dashboards. | P1E-E028, P2B-E021, P2C-E016, P3L-E017, P4C-E015, P4I-E014 |
| IX-019 | E | 4C cross-session write locks and 2B FILE_CHECK both depend on tools/path reality but enforce different layers (policy hooks vs claim truth). | P4C-E013, P2B-E017 |
| IX-020 | E | Designer AgentGrcConfig (2C) produces PipelineSpec that later may include media steps triggering 4I lazy factory. | P2C-E018, P4I-E001 |

## Explicit ambiguity and conflict register

| Register ID | Kind | Unresolved issue or required correction | Atom references |
|---|---|---|---|
| AC-001 | conflict | 1E "sub-second" goal vs 2s hard interactive budget. | P1E-C001 |
| AC-002 | ambiguity | 1E heuristic edge-score aggregation into 0–10 before phi not fully formal. | P1E-A002 |
| AC-003 | ambiguity | 1E SPLADE in knowledge order vs no 1E splade stage. | P1E-A001 |
| AC-004 | ambiguity | 2B claim regex / Uncertain path-vs-symbol heuristics incomplete. | P2B-A001, P2B-A002 |
| AC-005 | conflict | reasoning.txt 4-field AgentGrcConfig vs 2C 8-field requirement + GroundingLevel order fix in 2A. | P2C-C001, P2C-E013, P2C-E020 |
| AC-006 | ambiguity | FINDING: prefix vs future structured FindingReport. | P2C-A001 |
| AC-007 | ambiguity | PI_T2_CIRCUIT_BREAKER_LINES exact value; full conventions SQL columns. | P3L-A001, P3L-A002 |
| AC-008 | conflict | 4C adversarial "prefer ruff library" vs decided subprocess+breaker. | P4C-C001 |
| AC-009 | ambiguity | T2 deny-list / lazy-code pattern catalog incomplete in phase text. | P4C-A001 |
| AC-010 | conflict | architecture-overview media models stale vs 4I Mode A/B authority. | P4I-C001 |
| AC-011 | ambiguity | QualitySpec threshold field names and max_iterations override policy. | P4I-A001, P4I-A002 |
| AC-012 | qualification | MASTER marks phase-3L as unindexed inventory item; deep extract is authoritative for 3L atoms. | P3L-E001 |

## Summary additions and corrections

Orientation comparison target: `docs/thefinalglance/feature-discovery/01-phase-index-MASTER.discovery.md`. “ADD” means an explicit source behavior omitted or materially compressed by the summary; “CORRECT/QUALIFY/REMOVE” means the summary’s wording changes scope or certainty. These are relational records, not additional feature atoms.

### 1E deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-1E-001 | **ADD** full risk set R30–R34 with mitigation numbers (N_seed, K_edge, density 0.01, max_loops, ±2 lines, 2s budget). | MASTER:L99-L112 | P1E-E007 |
| SD-1E-002 | **ADD** sparse CatRagOverlay type fields, QueryPipeline/PipelineStage schema, and measure_1e five-query catalog Q1–Q5 + phase_1E.toml keys. | MASTER:L103-L112 | P1E-E009, P1E-E015, P1E-E028 |
| SD-1E-003 | **ADD** deterministic 1E A-RAG fixed tool order vs deferred ReAct Phase 2+; stage failure fall-through; RRF rank-from-1. | MASTER:L111 | P1E-E025, P1E-E027 |
| SD-1E-004 | **ADD** match-cap 2*N_seed and >50% DKG match warning; optional LLM edge scoring via breaker only. | MASTER:L107 | P1E-E020, P1E-E021 |
| SD-1E-005 | **FLAG** sub-second vs 2s acceptance tension absent from summary compression. | MASTER:L100, L112 | P1E-C001 |

### 2B deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-2B-001 | **ADD** PlanGroundingResult verify_needed semantics (training-only subset of ungrounded) and empty-plan trivially grounded. | MASTER:L194 | P2B-E008, P2B-E020 |
| SD-2B-002 | **ADD** complete prefix dispatch table including grep→FileCheck and tool→SourceCheck; 30s cargo test timeout; BashSandbox API. | MASTER:L198-L201 | P2B-E017-P2B-E019 |
| SD-2B-003 | **ADD** full T1–T16 catalog, network/timing bans, tempfile cleanup, and dependency gate (no knowledge in cargo tree). | MASTER:L197, L196 | P2B-E015, P2B-E016, P2B-E013 |
| SD-2B-004 | **ADD** Uncertain auto-resolution during think() enhancement of 2A; 4A wrap-without-signature-change constraint. | MASTER:L191, L200 | P2B-E004, P2B-E005 |
| SD-2B-005 | **QUALIFY** summary network stub string vs source "not implemented in phase 2B" — behavioral typed error is the requirement. | MASTER:L200 | P2B-C001 |

### 2C deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-2C-001 | **ADD** complete T1–T15 test catalog and role budgets (executor 8/1200, auditor 10/1200, memory_writer 2/400, prompt_engineer 5/800, designer distinct). | MASTER:L209, L215 | P2C-E011, P2C-E018 |
| SD-2C-002 | **ADD** whitelist formula `(base ∪ overrides) ∩ (required_steps ∪ {Status,Converge})` and with_agent_config topic reset. | MASTER:L216 | P2C-E012, P2C-E014 |
| SD-2C-003 | **ADD** FINDING_PREFIX contract, scope enumeration WARN path, and five validation rules D2 including csq_required. | MASTER:L211-L212 | P2C-E005, P2C-E007, P2C-E008 |
| SD-2C-004 | **ADD** full SPEC GAPS list (8-field type, GrcError variants, PartialOrd order, rubber-stamp invariant, methods vs quality complementarity). | MASTER:L207-L218 | P2C-E020, P2C-C001 |
| SD-2C-005 | **ADD** metrics memory <2KB and malformed-table fail vs missing-table default distinction. | MASTER:L218 | P2C-E016, P2C-E014 |

### 3L deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-3L-001 | **ADD** entire Phase 3L feature set: MASTER only notes phase-3L.txt as unindexed file to inspect (Q-line), without atom extraction. | MASTER:L655, L713 | P3L-E001–P3L-E021 |
| SD-3L-002 | **ADD** Pi-prefixed hook naming, LazyLock D2, conventions_migration_v1 flat path, seed_rules API, profile.toml + /profile. | MASTER:L655 | P3L-E003–P3L-E008 |
| SD-3L-003 | **ADD** telescope priorities 100/80/60, TELESCOPE_MAX_CHARS 1200, pi_telescope_contract five invariants, calibration gates 50/100 ms. | MASTER:L655 | P3L-E015–P3L-E017 |
| SD-3L-004 | **ADD** four-language determinism matrix and CONVENTIONS prompt block budget 200 tokens. | MASTER:L655 | P3L-E010, P3L-E012 |
| SD-3L-005 | **ADD** 3K predecessor correction (not Tier0 scaffold) and reuse of plugins HookVerdict without forking 4C tiers. | MASTER:L655 | P3L-E002, P3L-E003 |

### 4C deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-4C-001 | **ADD** complete 30+10 test catalogs, cross-session locks, SCHEMA_REQUIRED Escalate, ShortCoder AST-equivalent tests. | MASTER:L421 | P4C-E011, P4C-E012 |
| SD-4C-002 | **ADD** PreCompact 0.85 handoff threshold, ruff >10000 breaker, T2 >5000 breaker, BSP cache invalidation timing. | MASTER:L413-L418 | P4C-E003, P4C-E007, P4C-E005 |
| SD-4C-003 | **ADD** exact sanitization string "Tool call rejected by policy", plugins dependency direction, RuntimeClient ssb/fsm/clock rule. | MASTER:L419 | P4C-E004, P4C-E014, P4C-E016 |
| SD-4C-004 | **ADD** 4A WARN-1/WARN-6 mitigations and metrics nine categories with coverage >80%. | MASTER:L422 | P4C-E017, P4C-E015 |
| SD-4C-005 | **ADD** deferred skills/MCP rationale (no 4D–4G dependency on 4C for those). | MASTER:L423 | P4C-E002 |

### 4I deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-4I-001 | **ADD** full veto matrix including schema-invalid Degraded and VLM-unavailable Degraded; Mode A post-validation field checks + retry. | MASTER:L506-L507 | P4I-E007, P4I-E011 |
| SD-4I-002 | **ADD** complete adversarial set (injection, 500MB, headers, path/symlink, sandbox, queue 65, ComfyUI 600s, CLAP noise). | MASTER:L496-L507 | P4I-E009 |
| SD-4I-003 | **ADD** full RCE type system (MediaRenderAdapter, five adapters, VisualFeedbackLoop, VisualIssue, convergence by severity/category/location). | MASTER:L505 | P4I-E015–P4I-E017 |
| SD-4I-004 | **ADD** SSB visual_feedback liveness, lazy register_lazy factory, ToolCapabilityFlags, MockVlm Phase0, ollama ≥0.12.7. | MASTER:L498-L502 | P4I-E012, P4I-E004, P4I-E005, P4I-E018 |
| SD-4I-005 | **ADD** RCE latency targets (web <3s, Blender <10s, loop web <10s/iter), convergence ≥70% measure, total ≥43 tests, Phase 4 overall exit block. | MASTER:L506-L507 | P4I-E014, P4I-E008, P4I-E022 |
| SD-4I-006 | **ADD** stale architecture-overview LLaVA note and MoE VRAM clarification conflict with media-validator L60. | MASTER:L501-L502 | P4I-E021, P4I-C001 |
| SD-4I-007 | **QUALIFY** summary "bounded N" iterations: source default max_iterations=3 with early stop on unchanged issues. | MASTER:L505 | P4I-E017 |

## Exhaustiveness, zero-omission, and duplicate audit

- Source coverage: **6/6 files**, **2,773/2,773 logical lines**, **176,724/176,724 bytes** read and reconciled; source SHA mismatches **0**.
- Atomic source ledger: **157 unique atoms** = **135 explicit**, **006 inferred**, **011 ambiguous**, **005 conflicting/corrective**.
- Per-file atom counts: 1E **034**; 2B **027**; 2C **024**; 3L **024**; 4C **021**; 4I **027**. Sum **157**.
- Relational interaction records: **20** (`IX-001`–`IX-020`). Summary-delta records: **032** (`SD-*`). Ambiguity/conflict register: **12** (`AC-001`–`AC-012`). These records reference source atoms and are not double-counted as features.
- ID audit: every atomic table row has exactly one unique ID matching `P1E/P2B/P2C/P3L/P4C/P4I-[EIAC]nnn`. Duplicate atomic IDs **0**; missing/blank atomic IDs **0**; nonconforming atomic IDs **0**.
- Semantic duplicate audit: repeated premise/scope/wiring/grounding/acceptance statements were merged into one atom with multiple anchors. Interactions, conflict register, and summary deltas reference atoms instead of cloning them. Duplicate feature atoms found **0**.
- Omission audit: every source was checked against capabilities/actions/surfaces; workflows/states; schemas/data/lifecycle; flags/defaults/config; integrations/events; permissions/security/privacy/compliance; failures/fallback/retry/recovery; determinism/idempotency/concurrency; observability/evidence; tests/acceptance/done gates; constraints/non-goals; dependencies/status/deferred. Each per-file ledger explicitly names absent dimensions rather than inventing them.
- User-facing surface audit: 1E measure binary + pipeline stages (no dashboard panel); 2B verification (internal + plan gate); 2C TOML role tables `[grc.defaults]`/`[agents.*.grc]`; 3L `/profile` + CONVENTIONS prompt block + project_profile.toml; 4C invisible hooks (agent sees only generic reject) + SSB findings + metrics; 4I verdicts Pass/Fail/HumanReview/Degraded, lazy media load, RCE viewports, SSB `visual_feedback` — each captured as explicit atoms where the source states them.
- Interpretation audit: inferred, ambiguous, and conflicting/corrective atoms are visibly separated by ID/class. No inference is promoted to a requirement. Orientation-only claims remain in delta records and are not source atoms.
- Manifest-boundary audit: the only original sources opened were the six exact DEEP-PHASE-004 manifest rows. The only orientation artifact consulted was `01-phase-index-MASTER.discovery.md`.
- Output metrics after finalization: `output_lines=0360`; `output_bytes=056549`.
- Canonical normalized output SHA-256: 1679edc472d305f8e64579c25160a629789ad6e18fc06a442505162db501a2f7.
- Hash rule: compute SHA-256 over the exact final UTF-8 bytes including the final newline after replacing only the 64 hexadecimal characters on the `Canonical normalized output SHA-256` line with 64 ASCII zeroes. Fixed-width output metrics and digest substitution avoid impossible raw self-reference.
