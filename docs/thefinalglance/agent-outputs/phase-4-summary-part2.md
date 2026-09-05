# Phase 4 Lossless Summary — Part 2 (4D + 4E + 4F)

# PHASE 4D — PIPELINE FACTORY L1 (Research Agent + FlashResearch) — IMPROVED 2026-04-06

Prereq: 4A done. Status: design + recursive extension + web research infra complete (user says pre0..4E done).

GOAL: Research Agent designs pipelines from task descriptions. Tree-structured research with GRC grounding. Produces PipelineSpec + QualitySpec via structured ResearchReport. Paper: arxiv 2510.05145 FlashResearch (up to 5x speedup over sequential).

READ: pipeline-factory.txt S1-9 (L1 designer, Research Agent, tree arch, source quality, output types, progress), reasoning.txt S2-5 + S9 (GRC + IPC-proxied verification), external-analysis.txt (FlashResearch), workflow/papers/flashresearch-interface.md.

## Types (4D.1 SKELETON)

**Research Agent + Designer role configs**:
- Research Agent: model cloud preferred or Gemma4, GRC 10/1200.
- Designer (DISTINCT from Executor) role=AgentRole::Designer, reuses executor model (Gemma 4 31B), GRC 8/1200, required_steps=[Observe, Hypothesize, PlanAction, Verify, Converge]. role_prompt: "Design phased execution plan from research findings. Output: TOML PipelineSpec with phases, dependencies, tool requirements, resource estimates, quality metrics. Do not implement. Plan only."

**SystemConstraints (extended)**: `{ model_max_context, available_for_work, avg_file_tokens, avg_test_tokens }`. When knowledge index <10 files: use elai_cc.toml [planning.sizing] defaults (default_avg_file_tokens=150, default_avg_test_tokens=80). Else: knowledge_engine.index_stats(). L1 Observe includes context budget constraint.

**ResearchTree** `{ plan:ResearchPlan, state:SharedResearchState, root:NodeId, breadth_factor:u32, depth_limit:u32, max_iterations:u32, phi_min:f64, psi_min:f64, speculative_depth:u32, time_budget:Duration }`

**ResearchNode** `{ question:String, parent:Option<NodeId>, children:Vec<NodeId>, phi:f64, psi:f64, status:NodeStatus, info_gain:f64, iterations:u32 }`
- NodeId = newtype u64 (typed arena index)
- NodeStatus: Pending | Active | Converged | Pruned

**ResearchPlan** `{ query, complexity: QueryComplexity, sub_questions }`
- QueryComplexity: Simple | Moderate | Complex
- SharedResearchState `{ nodes:HashMap<NodeId,ResearchNode>, next_id:NodeId }`

**Output types**:
- ResearchTreeSnapshot `{ nodes, edges }`
- ConvergenceStatus: Converged{phi,psi} | Partial{phi,psi,reason} | Failed{reason}
- Alternative `{ description, sources:Vec<SourceCitation>, evidence_level }`
- Contradiction `{ claim_a, claim_b, source_a, source_b }`
- EvidenceLevel: Benchmark(1.0) | OfficialDocs(0.8) | BlogPost(0.6) | Training(0.4). Ranking formula: `evidence_level.score() * source_quality_avg` (pure arithmetic).

**PipelineSpec + QualitySpec**:
- QualityMetric `{ name, target, unit, method, phase, hard_fail }`
- MediaSpec `{ output_name, format, resolution, ssim_threshold, vlm_check, output_type: OutputType }`
- OutputType: Web | Model3D | Audio | UiScreenshot | Animation | Image | Custom(String)
- PerfSpec `{ name, target, unit, method, sample_count }`

**ResearchReport** `{ query, tree:ResearchTreeSnapshot, sub_questions:Vec<SubQuestionResult>, total_sources_fetched/used, total_iterations, total_nodes_explored/pruned, speculative_hit_rate, convergence_status }`

**SubQuestionResult** `{ question, answer, sources, confidence, phi, psi, depth, alternatives, contradictions }`. Confidence formula: `source_count * avg_quality * agreement_ratio`.

**SourceCitation** `{ url, extracted_quote, quality_tier: SourceQuality }`

**SourceQuality** (5-tier deterministic, NO LLM, domain lookup): Tier1(1.0) | Tier2(0.8) | Tier3(0.6) | Tier4(0.4) | Tier5(0.2). Configurable: elai_cc.toml [research.trusted_domains].

**Adaptive complexity** (pipeline-factory.txt L113-118):
- Simple: breadth=2, depth=3, phi_min=0.8, psi_min=0.7
- Moderate: breadth=3, depth=4, phi_min=0.7, psi_min=0.6
- Complex: breadth=4, depth=5, phi_min=0.6, psi_min=0.5

information_gain_threshold f64 [0.1, 0.9]. Below threshold for 3 consecutive iterations → prune.

Research Agent as Tier C: web search only via IPC proxy, GRC verification via IPC GroundingRequest, no FS access, model via IPC to LlmProvider.

Deferred aggregation for parallel sub-question collection: root children parallel, local models sequential within level (one GPU), cloud parallel across breadth (rate-limit aware via orchestrator S8 Dynamic Fan-Out), pruned branches free resources immediately.

## Steps (base 4D.1–4D.6)

### 4D.2 VERIFY — cargo build + existing tests clean, no new warnings.

### 4D.3 TESTS (≥14 contract + ≥3 integration + ≥3 proptest, all FAIL)
Contract: tree depth bounded by depth_limit (default 5), breadth by breadth_factor (default 3), acyclic (child never references ancestor), bounded iterations per node, dual-threshold convergence (phi >= phi_min AND psi >= psi_min), source quality deterministic (no LLM), confidence = source_count * avg_quality * agreement_ratio, contradictions surfaced (never silently resolved), parallel bounded by resource limits (local sequential), speculative disabled when local (single GPU), speculative bounded by speculative_depth * breadth_factor, pruned speculative frees resources, speculative_hit_rate tracked, total sources tracked (fetched vs used).

Integration: task → ResearchReport with grounded alternatives. PipelineSpec from ResearchReport only grounded refs. QualitySpec has measurable metrics (unmeasurable REJECT via L2). Research Agent Tier C (no .elai_cc / project source). SSB receives research_step events. time_budget enforcement. Adaptive complexity works.

Proptest: random tree no cycles, random phi/psi in [0,1], source quality idempotent.

### 4D.4 ADVERSARIAL (≥8, all FAIL)
FS escape blocked. Direct network blocked (raw HTTP). GRC budget 1/100 on complex → forced convergence (not hang). Tree depth bomb. Tree breadth bomb. Speculative branch pruning on parent direction change. Source quality gaming (arxiv.phishing.com → Tier5 not Tier1, exact/subdomain match only). Empty task graceful error. ALL-TRAINING report flagged HIGH_RISK (pipeline-factory.txt S3 L36-37). Contradictions surfaced in SubQuestionResult.contradictions.

### 4D.5 IMPLEMENT
Per-node research loop (pipeline-factory.txt S5 L87-101):
1. FORMULATE: GRC generates search query from sub-question + prior findings
2. SEARCH: web search returns top N (default 10) via IPC
3. EXTRACT: fetch page, sampling-based relevance filter (read first 500 tokens, score, read full if above threshold)
4. REFLECT: GRC evaluates phi (goal satisfaction), psi (quality), contradictions, new sub-questions, information gain
5. CONVERGE/EXPAND/ITERATE

Tree lifecycle: Pending → Active → (Converged | Pruned). Invariant acyclic (parent chain check). Bounded by depth_limit, breadth_factor, max_iterations.

Speculative (cloud only): trigger phi>0.4 suggests likely child questions. Execute top speculative_depth children before parent converges. Parent direction change → cancel. Local models: speculative_depth forced to 0. Max waste: speculative_depth * breadth_factor.

Dual-threshold convergence, adaptive. Deterministic domain lookup. Parallel sub-questions via Dynamic Fan-Out. Deferred aggregation node collects all sub-results after tree converges. Rate-limit aware for cloud APIs. SSB research_step per iteration. time_budget enforcement. information_gain_threshold prune.

Research Agent process has no direct network socket: Linux `lsof -i -p $PID` empty; Windows `netstat -ano | findstr $PID` shows only named pipe.

### 4D.6 MEASURE (9 metric categories → .elai_cc/metrics/phase_4D.toml)
grounding_distribution (% VERIFIED vs HIGH vs UNCERTAIN), spec_token_count, depth_utilization, convergence_p50, speculative_hit_rate, source_tier_distribution, time_budget_ratio (<=1.1 = within 10%), ssb_event_count, info_gain_trend (should increase then plateau).

## Recursive Decomposition Extension (Fractal Research Pipeline, 4D.7-4D.10)

DESIGN PRINCIPLE: System is a PIPELINE not a session. Each agent is a tool. Orchestrator (Rust) drives everything. LLM never controls flow. TreeNavigator decides research, not LLM. LLM receives grounded options and picks (autonomous) or presents (interactive).

4 DECOMPOSITION LEVELS:
- Level 0 IDEA: user input → ConceptDecomposition (N concepts with feasibility scores). Research scope: profitability, usefulness, tech stack feasibility, language. Gate: feasibility > threshold.
- Level 1 CONCEPT (e.g. "Database Schema & RLS"): per-concept FlashResearch → PhaseDecomposition (frameworks, packages, approach). Research: which framework, packages, API compatibility, version constraints. Gate: grounded package/framework (verified exists + compatible).
- Level 2 PHASE (e.g. "concept1_phase1: JWT Auth with Axum"): per-phase research → StepDecomposition (ordered impl steps). Research: exact API usage, version-specific behavior, edge cases. Gate: grounded implementation path (not training-derived).
- Level 3 STEP → BABY STEPS (e.g. "Argon2 hashing"): atomic baby steps (<2K token context each). Output: Vec<BabyStepSpec> with full 8-block structure. Research: function signatures, error handling, test patterns. Gate: independently executable and testable.

### New Types

```
struct ConceptDecomposition { idea, concepts:Vec<ConceptNode>, research:ResearchReport,
    feasibility_scores:Vec<(String, f64)> }
struct ConceptNode { concept_id:ConceptId, name, description, research:Option<ResearchReport>,
    phases:Vec<PhaseNode>, feasibility:f64, tech_stack:Option<TechStackDecision> }
struct PhaseNode { phase_id:PhaseId, concept_id, name, research, steps:Vec<StepNode>,
    framework:Option<String>, packages:Vec<PackageRef>, depends_on:Vec<PhaseId> }
struct StepNode { step_id:StepId, phase_id, name, research, baby_steps:Vec<BabyStepSpec> }
struct BabyStepSpec { baby_step_id:BabyStepId, step_id, grounding:BabyStepGrounding,
    pre_gate:Vec<PreGateCondition>, task:TaskSpec, tdd:TddSpec,
    deterministic_check:Vec<DeterministicCheck>, acceptance:AcceptanceCriteria,
    integration:IntegrationSpec, exit_criteria:Vec<ExitCriterion> }
struct BabyStepGrounding { research_findings:Vec<SourceCitation>,
    overall_context_summary:String (max 500 tokens), required_types, required_imports }
struct PreGateCondition { description, check_command, expected_exit_code:i32 }
struct TaskSpec { description, max_lines:u32, target_file, entry_point }
struct TddSpec { test_file, test_count, test_framework, write_before_code:bool (always true) }
struct DeterministicCheck { description, command, expected_output }
struct AcceptanceCriteria { min_tests_passing, max_retry_alternatives (default 3),
    on_all_fail:AcceptanceFailAction }
enum AcceptanceFailAction { BubbleUp | ResearchAlternative | Replan }
struct IntegrationSpec { connects_to:Vec<String>, integration_test_command,
    deferred:bool }
struct ExitCriterion { description, check:DeterministicCheck }
struct TechStackDecision { language, framework, packages, decision_source:DecisionSource }
enum DecisionSource { Research(Vec<SourceCitation>) | UserChoice | AutoSelected{reason} }
struct PackageRef { name, version (pinned verified), registry, verified:bool }
struct ResearchCache { entries:BTreeMap<String,CachedResearch>, max_entries, ttl_seconds }
struct CachedResearch { query, report, cached_at (ClockProvider), hit_count }
Newtype IDs: struct ConceptId(u64); struct BabyStepId(u64); // StepId/PhaseId from 4E
```

### Recursive Research Protocol

`fn research_decompose(idea, mode) -> DecompositionTree`:
- Level 0: top-level research, extract_concepts (deterministic)
- Per concept: cache_key=normalize_query(name), get_or_research flash_research(concept.name, Moderate). INTERACTIVE: present_options(alternatives). AUTONOMOUS: auto_select by evidence_level.
- Per phase: flash_research(phase.name, Simple). verify_packages (API_REGISTRY_CHECK each). extract_steps.
- Per step: flash_research(step.name, Simple). decompose_to_baby_steps. Each baby step gets compressed grounding from parent chain.

ResearchCache prevents redundant calls. Siblings researching similar topics (e.g. two phases both need "Axum routing") hit cache. TTL prevents stale across sessions.

Package verification: every PackageRef gets API_REGISTRY_CHECK (reasoning.txt S4). Nonexistent/wrong version → retry with corrected info. Anti-hallucination gate. LLM cannot invent a package.

### 4D.7 RECURSIVE TYPES SKELETON — add all types + stubs returning Err.
### 4D.8 RECURSIVE RESEARCH TESTS (≥13, all FAIL): Level 0/1/2/3, cache, cache TTL, package verification pos/neg, grounding compression <500 tokens, INTERACTIVE vs AUTONOMOUS branching, cross-concept dependency, proptest never panics.
### 4D.9 RECURSIVE RESEARCH IMPLEMENT
- research_decompose: orchestrator-driven 4-level loop
- extract_concepts/phases/steps: deterministic structured parsing (not free-form LLM text)
- decompose_to_baby_steps: <2K token per baby step. Max split depth 2.
- BabyStepGrounding compression: walk parent chain, extract only relevant citations, compress to max 500 tokens.
- ResearchCache with LRU eviction at max_entries
- Package verification via API_REGISTRY_CHECK
- INTERACTIVE: present alternatives with evidence_level, wait for user choice. AUTONOMOUS: highest evidence_level * source_quality.
- Cross-concept dep detection: if phase A uses types from phase B, add depends_on edge. Uses DKG if available, else structural analysis.
- SSB research_decompose_level event per level transition.

### 4D.10 RECURSIVE RESEARCH MEASURE
Decomposition depth util, research calls per level (decrease at deeper levels due to cache), cache hit rate, baby step count per idea (avg + max), baby step grounding size p50/p95 (<500 tokens), package verification pass rate, time per level (L0/L1 seconds, L2/L3 ms).

## Web Research Infrastructure Extension (4D.11-4D.18, "Deep Research")

Paper grounding: arxiv 2508.12752 Deep Research Survey, arxiv 2506.18096 Deep Research Agents Roadmap, arxiv 2508.05668 Deep Search Agents Survey, arxiv 2510.05145 FlashResearch, arxiv 2602.03442 A-RAG. Trusted source list: docs/Seeking Verified AIFree Information.txt (200+ domains). Baseline: .elai_cc/metrics/baseline_findings_and_actions.md.

Design principles:
1. API-based retrieval, not browser automation (2506.18096: browser 2-3x resource cost, prohibitive below 8GB VRAM). SearXNG self-hosted = zero API cost.
2. ManuSearch-style deterministic orchestration (2508.12752: no RL needed; rule-based planner + search + extraction). LLM doesn't control flow.
3. Progressive disclosure from A-RAG (2602.03442): snippets first, selective full read. 90% token savings.
4. Evidence grows monotonically (2508.12752 invariant). Prune at synthesis, not retrieval.
5. Hybrid tree-based preferred tuning-free (2508.05668). Our FlashResearch tree + per-node loop matches.
6. Source quality deterministic domain lookup, never LLM-scored.

### 4D.11 WEB SEARCH CLIENT (SearXNG)
New crate `.elai_cc/crates/web_search/`. Docker setup: `docker run -d --name elai-searxng -p 8888:8080 -v ./searxng-settings:/etc/searxng searxng/searxng:latest`. Custom settings.yml (JSON output, engine weights, disable telemetry, result_count=20).

```
struct SearxClient { base_url:String (default http://localhost:8888), timeout (10s), max_results (20) }
fn search(&self, query) -> Result<Vec<SearchResult>>  // GET /search?q=...&format=json
struct SearchResult { title, url, snippet, engine, score, parsed_domain }
fn health_check() -> Result<bool>  // GET / with 2s timeout
```
Rate limit 1 req/s (configurable). 1 retry exponential backoff on 5xx. IPC proxy: SearxClient in orchestrator. Research Agent (Tier C) sends via IPC. No LLM dep — pure HTTP + JSON.

Adversarial: 503 mocked → Result::Err, not panic. Special chars URL-encoded. HTML in snippets stripped.

### 4D.12 CONTENT EXTRACTION (HTML → clean text)
```
struct WebFetcher { client:reqwest::Client (timeout 15s, max_redirects 5,
    user_agent "ELAI-Research/0.1 (research-agent)", max_body_size 2MB),
    rate_limiter per-domain (2 req/s/domain) }
fn fetch(url) -> Result<FetchedPage { url, status, content_type, html, fetch_time_ms }>
trait ContentExtractor { fn extract(html, url) -> Result<ExtractedContent
    { title, text, word_count, page_type, extracted_at }> }
```
ReadabilityExtractor initial. Fallback to raw strip HTML. Verify readability-rust on crates.io. If not viable: remove <script>/<style>/<nav>/<header>/<footer>, extract from <p>/<article>/<main>/<section>/<div>, collapse whitespace.

robots.txt respect: `fn check_robots(url) -> Result<bool>`, parse Disallow, cache per-domain 1h, Disallow → Err.

IPC proxy: WebFetcher in orchestrator.

Adversarial: non-UTF8 → lossy convert not crash. >2MB rejected before full read (no OOM). 404 robots.txt → fully allowed. JS-only page → page_type=Empty (not silent empty string).

rs-trafilatura evaluated at 4D.18 if quality insufficient (F1=0.966 per github.com/Murrough-Foley/rs-trafilatura).

### 4D.13 SOURCE QUALITY TABLE (trusted domain lookup)
Parse docs/Seeking Verified AIFree Information.txt into structured table.
```
struct SourceQualityTable { domains:HashMap<String,SourceTier>,
    subdomain_rules:Vec<(String, SourceTier)> }
enum SourceTier { Tier1(1.0), Tier2(0.8), Tier3(0.6), Tier4(0.4), Tier5(0.2) }
```
Tier1: .gov, .int, peer-reviewed (arxiv.org, pubmed, ieee.org, acm.org, data.gov, cdc.gov, nasa.gov, who.int).
Tier2: institutional repos (zenodo, osf.io, cochranelibrary, semanticscholar, biorxiv, medrxiv, core.ac.uk, doaj.org).
Tier3: university/professional (plato.stanford.edu, quantamagazine, examine.com, nutritionsource.hsph.harvard.edu, nber.org).
Tier4: known commercial/code registries (docs.rs, crates.io, pypi.org, npmjs.com, stackoverflow.com, github.com w/ star threshold).
Tier5: unknown.

`lookup(domain) -> SourceTier`: exact first, then subdomain rules (*.gov → Tier1, *.edu → Tier3). Default Tier5. `score(domain) -> f64`. TOML config in elai_cc.toml `[research.trusted_domains]` + `[research.trusted_subdomain_rules]`.

Adversarial: "arxiv.evil.com" → Tier5 (exact or registered-domain match, not substring). URL with port: strip. URL with path: only domain.

### 4D.14 WEB A-RAG (hierarchical retrieval for fetched web content)
Extend knowledge crate.
```
struct WebContentIndex { chunks:HashMap<String,WebChunk>, sentences:Vec<String>,
    embeddings:Option<Vec<Vec<f32>>>, sentence_to_chunk:Vec<String> }
struct WebChunk { chunk_id, source_url, source_tier, text, sentences, word_count }
```
Sentence splitting for PROSE (distinct from code): code uses AST; prose uses regex `[.!?\n]+` with min length 10 chars (A-RAG paper L36-37).

Chunk size ~500 tokens for prose (halved from A-RAG's 1000 for smaller contexts). Configurable per VRAM tier.

Three retrieval tools (reusing RetrievalTool enum from arag.rs):
- KeywordSearch: `count(keyword, text) * len(keyword)` scoring (A-RAG L72-77)
- SemanticSearch: embed query, dot product vs sentence embeddings, 3x oversampling then chunk aggregation (arag-interface.md L109-113)
- ChunkRead: full text with ContextTracker dedup

Embedding strategy: GPU available + model not loaded → sentence-transformers on CPU. Sidecar running → proxy via sidecar /v1/embed. Fallback: keyword-only.

Progressive disclosure (A-RAG key insight): search tools return SNIPPETS (matched sentences), not full chunks. Agent evaluates, decides which chunks to read. chunk_read returns full, ContextTracker prevents re-reading. Saves 90% tokens (arag-interface.md L273).

Adversarial: URL in sentence not split mid-URL. Keyword scoring normalized by length (short chunk with 5 hits > long chunk with 5 hits). Embeddings L2-normalized (dot product == cosine).

### 4D.15 VRAM-TIERED RESEARCH CONFIG
elai_cc.toml `[research.vram_tiers]` with 6 tiers:

| Tier | Model | Quant | Ctx | Budget | Chunk | Br | Dep | phi | psi | Embed | Src | Concur |
|------|-------|-------|-----|--------|-------|----|----|-----|-----|-------|-----|--------|
| 8gb | 3B | NF4 | 16384 | 12000 | 300 | 2 | 3 | 0.8 | 0.7 | cpu | 5 | 2 |
| 12gb | 3B | NF4+FA2 | 40000 (measured) | 30000 | 500 | 3 | 4 | 0.7 | 0.6 | cpu | 10 | 4 |
| 12gb_7b | 7B | NF4+FA2+TQ8 | 35000 | 26000 | 400 | 3 | 4 | 0.7 | 0.6 | cpu | 8 | 3 |
| 16gb | 7B | NF4+FA2 | 40000 | 30000 | 500 | 3 | 5 | 0.7 | 0.6 | gpu | 10 | 4 |
| 24gb | 14B | NF4+FA2 | 25000 | 18000 | 500 | 3 | 5 | 0.6 | 0.5 | gpu | 10 | 4 |
| 48gb | 32B | NF4+FA2 | 24000 | 18000 | 600 | 4 | 5 | 0.6 | 0.5 | gpu | 15 | 6 (speculative_depth=2) |

Only 48gb tier has speculative_depth > 0.

```
struct ResearchTierConfig  // parsed from TOML at startup
fn detect_vram_tier() -> VramTier  // query GPU VRAM, map to nearest, round DOWN
fn research_config_for_tier(tier) -> ResearchTierConfig
```
Adversarial: nvidia-smi failure → fallback to 8gb. 10GB VRAM → 8gb tier (round DOWN).

### 4D.16 TOKEN BUDGET MANAGER (evidence pruning for small models)
```
struct ResearchBudgetManager { tier_config, current_evidence_tokens, current_sources }
fn can_admit(new_evidence_tokens) -> bool  // current + new <= research_context_budget
fn prune_to_budget(evidence: &mut Vec<EvidenceItem>) -> usize
    // drop lowest-tier first (Tier5 before Tier4), within tier drop lowest phi first,
    // NEVER drop Tier1. Returns pruned count.
fn compress_evidence(item) -> CompressedEvidence
    // keep first 3 sentences + last sentence (intro + conclusion).
    // Source: 2508.12752 (we compress representation, not discard record)
```
Integration with A-RAG progressive disclosure: after snippet search, budget manager decides which to read in full. High-tier read full. Low-tier snippets unless critical.

Token counting: byte-level (~4 bytes/token English). Overcount 10% safety.

Adversarial: budget=100, 5 items 50 tokens each → assert ≥1 item retained (highest tier + phi). Compressed evidence retains source_url + source_tier for citation. Estimate within 20% of actual.

### 4D.17 RESEARCH ORCHESTRATOR (wire everything)
```
struct WebResearchOrchestrator {
    searx_client, web_fetcher, quality_table, content_index,
    budget_manager, tier_config,
}
```
Per-node loop (FORMULATE-SEARCH-QUALITY-FETCH+EXTRACT-INDEX-RETRIEVE-REFLECT-CONVERGE):
1. LLM via sidecar (FORMULATE)
2. SearxClient.search
3. Quality gate (Tier1-3 first, Tier4-5 only if budget allows)
4. WebFetcher.fetch → ContentExtractor.extract (parallel up to max_concurrent_fetches)
5. Index → WebContentIndex (chunk + embed)
6. A-RAG progressive disclosure
7. REFLECT via sidecar
8. CONVERGE or EXPAND

IPC: Research Agent (Tier C) sends SearchRequest / FetchRequest / EmbedRequest / InferenceRequest to orchestrator. Orchestrator proxies to SearXNG / target URL / sidecar /v1/embed / sidecar /v1/completions. Agent never has direct network.

SSB: web_research_step per node iteration.

Adversarial: SearXNG down → partial result + error note, not hang. >80% Tier5 sources → flag confidence=LOW. Sidecar unavailable → clear error. 10 URLs same domain → max 2 concurrent per rate limiter.

### 4D.18 END-TO-END MEASUREMENT
10 queries (3 Simple, 4 Moderate, 3 Complex). Per-query metrics: research_time_seconds, total_sources_fetched/used, source_tier_distribution, tree_depth_used vs limit, convergence_iterations (mean/p50/p95), token_budget_utilization, evidence_items_pruned, report_grounding_level (% Tier1-3).

Run on 12GB RTX 3060 with both 3B and 7B. Store `.elai_cc/metrics/phase_4D_web_research.toml`.

Extraction quality: sample 20 pages, manually verify. If >20% issues, evaluate rs-trafilatura.

Acceptance: all 10 queries produce report (no crash/hang), token_budget_utilization < 1.0 all queries/tiers, ≥30% Tier1-3 for complex queries, research_time <5min simple / <15min complex (12GB), extraction quality ≥80% clean.

## Exit Criteria (4D)
Research Agent produces grounded pipeline designs. All 14 contract invariants tested + passing. All 8+ adversarial passing. Source tiering deterministic. SSB progress tracking. PipelineSpec only grounded refs. QualitySpec rejects unmeasurable. time_budget within 10%.

---

# PHASE 4E — PIPELINE FACTORY L2 (builder + AutoB2G-derived DAG repair) — IMPROVED 2026-04-06

Prereq: 4D done (PipelineSpec, QualitySpec, ResearchReport operational). Status: user says pre0..4E done.

GOAL: Deterministic pipeline assembly from PipelineSpec. No LLM, no agent process. L2 runs entirely in orchestrator crate. Pure Rust. DAG repair loop (AutoB2G-derived: bounded DAG validation + iterative repair) uses DETERMINISTIC strategies only. Paper: arxiv 2603.26005 AutoB2G (adopted: DAG structure, bounded repair T_DAG, structured feedback; NOT adopted: SOCIA LLM-driven repair).

Two repair strategies:
1. REORDER: topological sort detects back-edges → break by removing weakest dependency
2. SUBSTITUTE: missing tool → find equivalent in ToolRegistry (same category, compatible IO)

Non-repairable (escalate immediately, no repair attempt): resource overflow (validation check #4, not structural), missing transitive deps with ambiguous resolution.

## Types

Newtypes: PipelineId(u64), StepId(u64), NodeIndex(usize), StateId(u64), DataRef(String).

```
struct ExecutablePipeline {
    pipeline_id, spec:PipelineSpec (preserved for traceability),
    fsm_states:Vec<PipelineFsmState>, dependency_graph:DependencyGraph,
    novel_step_ids:Vec<StepId>  // consumed by 4F FEASIBILITY_PROBE
    agent_configs:Vec<AgentPhaseConfig>, rule_configs:Vec<RuleBinding>,
    resource_estimate:ResourceEstimate, test_harness:TestHarnessSkeleton,
    repair_log:Vec<RepairAction>, output_dir: PathBuf (.elai_cc/pipelines/{id}/)
}

struct AgentPhaseConfig { phase_id:StepId, agent_role:String, grc_budget:(u32,u32),
    model_preference:Option<String> }
struct RuleBinding { phase_id:StepId, rule_ids:Vec<RuleId>, install_order:u32 }
struct DependencyGraph { nodes:Vec<DagNode>, edges:Vec<(NodeIndex,NodeIndex)> }
    // Invariant: topological_sort always succeeds after validation/repair
struct DagNode { step_id, tool_id:ToolId (must exist in ToolRegistry),
    inputs:Vec<DataRef>, outputs:Vec<DataRef>, resource_req:ResourceReq, optional:bool }
struct PipelineFsmState { state_id, phase_name, allowed_tools, allowed_rules,
    entry_gate:Option<Predicate>, exit_gate:Predicate (compiled), is_terminal:bool }
struct ResourceEstimate { total_vram_mb, peak_vram_mb, estimated_disk_mb,
    estimated_duration_secs, fits_hardware:bool }
struct TestHarnessSkeleton { test_cases:Vec<TestCase>, fixture_stubs:Vec<FixtureStub> }
    // Invariant: test_cases.len() >= metrics.len() + perf_specs.len()
enum RepairAction { Reorder{from,to,reason}, Substitute{old_tool,new_tool,reason} }
enum RepairOutcome { Repaired{rounds,actions}, Escalated{rounds,remaining_issues} }
struct ValidationError { kind:ValidationErrorKind, node:Option<NodeIndex>, message:String }
enum ValidationErrorKind { CycleDetected, MissingTool, MissingDependency,
    ResourceOverflow, UnmeasurableMetric, InvalidPredicate, MissingAdapter, InvalidVisualCheck }
```

## Structural Validation Checklist (8 checks)
1. Tools in PipelineSpec exist in ToolRegistry
2. Models exist in ModelPool or installable
3. No dependency cycles (topological sort)
4. Resources fit hardware (ModelPool.available_vram)
5. Every QualityMetric has automated check method (unmeasurable → REJECT)
6. FSM state predicates syntactically valid (rules::compile_predicate)
7. Novel step detection: tool_id not in ToolRegistry → marked novel (4F dependency)
8. Visual check adapter validation: visual_check=true phases have valid MediaRenderAdapter for output_type (else MissingAdapter / InvalidVisualCheck)

## Steps

### 4E.1 SKELETON — all types + 9 stubs
PipelineBuilder, StructuralValidator, DagRepairLoop, FsmStateGenerator, TestHarnessGenerator, ResourceEstimator, NoveltyDetector, AgentConfigurator, RuleInstaller.

L2 lives in orchestrator crate. No separate crate — no LLM, no agent process.

### 4E.2 VERIFY — cargo build/test/clippy clean.

### 4E.3 TESTS (≥30, all FAIL)
7 validation + 7 repair + 3 novel + 4 FSM + 2 harness + 3 resource + 1 visual_check + 3 end-to-end.
Plus proptest: random acyclic graph → topo sort succeeds; random PipelineSpec → no panic; repair rounds produce structurally different graphs.

Adversarial catches:
- Repair loop without graph equality check can loop T_DAG times on same graph → inject unrepairable cycle, assert escalation (not T_DAG identical attempts)
- SUBSTITUTE with incompatible IO → silent wrong results. Test: substitute must have compatible input/output DataRef types.
- Resource estimation ignoring concurrent phases → peak underestimates. Test: 2 concurrent GPU phases, peak > single phase VRAM.
- QualitySpec silently dropping unmeasurable metrics → harness incomplete. Test: inject unmeasurable → error returned.
- FSM predicates referencing undefined variables → runtime failure. Test: generate, parse every predicate, variables resolve.

### 4E.4 ADVERSARIAL TESTS (≥10)
Empty PipelineSpec → graceful error. 100+ phases completes <5s. Nonexistent tools → MissingTool per step. All-optional → valid (possibly empty). Circular A→B→C→A detect + repair or escalate. Resources >10x VRAM → immediate escalation. T_DAG rounds with new error every round → escalation at round 10. Repair introducing new cycle detected in next round. REORDER + SUBSTITUTE same round: REORDER first, SUBSTITUTE re-evaluated on modified graph. Single-phase → is_terminal=true. Phase with no tools → empty allowed_tools (valid).

### 4E.5 IMPLEMENT — 5-STAGE elaboration (was 4): Research → Plan Assembly → Structural Validation → **Phase Sizing (Stage 3.5)** → Feasibility Probe.

**Phase Sizing (Stage 3.5, after structural validation, before repair)**:
```
phase_complexity = files_touched * avg_file_tokens
                 + test_count * avg_test_tokens
                 + contract_count * CONTRACT_OVERHEAD (default 200)
available_for_work = from context budget allocator
Split trigger: phase_complexity > available_for_work * SPLIT_RATIO (0.6)
Split by: contract boundaries, then directory proximity
Max split depth: 2 (matches 4G SPLIT_TASK)
Merge: adjacent phases only, no external dependents between them
Uses PlanSkeleton::split_phase with preserve_contracts=true (Phase 3G)
```

Kahn's cycle detection: L2 runs Kahn's on phase dep graph BEFORE producing ExecutablePipeline. Cycle → ValidationError::CyclicDependency with cycle path. No runtime deadlock.

Structural validation impl (8 checks per checklist).

FSM state generation: each PipelineSpec phase → PipelineFsmState. allowed_tools=union of phase steps' tools. allowed_rules=rules engine query for phase context. entry/exit conditions from PipelineSpec phase gates.

DAG repair bounded loop:
- validate → if errors, apply first applicable strategy → re-validate
- REORDER for cycles, SUBSTITUTE for missing tools
- Non-repairable (resource overflow, unmeasurable) → immediate escalation
- Hash graph structure per round, compare prev
- If hash unchanged → escalate (fixpoint, no progress)
- If round > T_DAG=10 → escalate

Resource estimation via ModelPool: per-step model VRAM + tool disk + runtime. Peak = max concurrent. fits_hardware = peak <= ModelPool.available_vram_mb.

Test harness from QualitySpec: one TestCase per QualityMetric + PerfSpec. Fixture stubs for common setup.

Output: ExecutablePipeline → `.elai_cc/pipelines/{id}/` as msgpack.

Zero LLM calls anywhere in L2 (grep confirms).

### 4E.6 MEASURE
Assembly time (1/5/10/50/100 phase) — target <1s for 10-phase. Repair success rate (fraction of repairable auto-repaired). Avg repair rounds. Repair strategy distribution. Test harness coverage (test_cases/metrics target 1.0). Resource estimation accuracy deferred to 4F+.

Target: Repair success rate ≥50% on test corpus of intentionally broken PipelineSpecs (N≥10). Store `.elai_cc/metrics/phase_4E.toml`.

## Decomposition Tree Extension (Fractal Pipeline Builder, 4E.7–4E.12)

Base 4E builds flat ExecutablePipeline. Extension builds DecompositionTree from ConceptDecomposition (4D recursive). Tree is primary execution artifact. Orchestrator traverses deterministically. Each leaf (baby step) cycles FSM: TESTING_FIRST → EXECUTING → AUDITING. No LLM controls traversal. TreeNavigator is pure Rust.

DESIGN: DecompositionTree is DATA, not a prompt. Typed Rust struct. Orchestrator reads to determine next. LLM fills in content (code, tests) but never decides flow. Flow: TreeNavigator picks next baby step → orchestrator assembles context from BabyStepGrounding → FSM cycles → result recorded → Navigator picks next. If baby step fails acceptance, BubbleUpHandler decides what to re-research. All decisions deterministic given tree state.

### New Types

```
struct DecompositionTree { tree_id:PipelineId, root:ConceptDecomposition,
    navigator:TreeNavigatorState, execution_log:Vec<BabyStepResult>,
    bubble_up_log:Vec<BubbleUpEvent>, integration_checkpoints:Vec<IntegrationCheckpoint>,
    created_at:u64 }
  // Invariant: always valid DAG, verified at construction

struct TreeNavigatorState { current_concept, current_phase, current_step, current_baby_step,
    completed:BTreeSet<BabyStepId> (deterministic iteration),
    failed:BTreeMap<BabyStepId,FailureRecord>, blocked:BTreeSet<BabyStepId>,
    traversal_order:Vec<BabyStepId> (pre-computed, deterministic) }
  // Serialized msgpack at every baby step completion for crash recovery

struct FailureRecord { baby_step_id, attempts:u32, alternatives_tried:Vec<String>,
    last_error, bubbled_up:bool }
struct BabyStepResult { baby_step_id, outcome:BabyStepOutcome, tests_written/passing,
    files_modified, commit_hash:Option<String>, elapsed_ms, auditor_findings }
enum BabyStepOutcome { Pass | FailRetry{attempt} | FailBubbleUp | Skipped{reason} | Deferred }
struct BubbleUpEvent { source:BabyStepId, target_level:DecompositionLevel, target_id,
    action:BubbleUpAction, research_report:Option<ResearchReport> }
enum DecompositionLevel { Concept, Phase, Step, BabyStep }
enum BubbleUpAction { ResearchAlternative | RedecomposeStep | ReplanPhase | ReplanConcept | Escalate }
struct BubbleUpPolicy { max_retries_per_baby_step (3), max_retries_per_step (2),
    max_retries_per_phase (1), max_retries_per_concept (1), on_concept_fail (Escalate) }
struct IntegrationCheckpoint { level, id, test_command, test_passed:bool,
    connects_to:Vec<String>, timestamp:u64 }

trait TreeNavigator {
    fn next_baby_step(&self, state, tree) -> Option<BabyStepId>
    fn record_result(&mut self, result)
    fn handle_failure(&mut self, baby_step_id, policy) -> BubbleUpAction
    fn is_complete(&self) -> bool
    fn integration_ready(&self, baby_step_id) -> bool
}
// Implementation: DepthFirstNavigator (default).
// Traversal: depth-first within concept. Concepts in dep order. Baby steps within step sequential.
// Steps within phase sequential. Phases within concept topological. Concepts topological.
```

### Tree Navigator Algorithm
```
fn next_baby_step(state, tree) -> Option<BabyStepId> {
    for id in &state.traversal_order {
        if state.completed.contains(id) { continue; }
        if state.blocked.contains(id) { continue; }
        let spec = tree.get_baby_step(id);
        if !all_pre_gates_pass(&spec.pre_gate) { continue; }
        if !dependencies_met(id, &state.completed, tree) { continue; }
        return Some(*id);
    }
    None
}
// Traversal order pre-computed at tree construction: topological sort at each level.
// Deterministic: same tree → same order → same execution sequence.
```

### Baby Step Execution Cycle (orchestrator-driven)

1. **Context assembly** (orchestrator, pure Rust, <5ms): Load BabyStepGrounding into Slot 3 (~500 tokens grounding + ~200 tokens task). Cheap local models produce quality code from narrow grounded context.
2. **PRE_GATE check**: each PreGateCondition.check_command via bash. All return expected_exit_code else BLOCKED.
3. **TESTING_FIRST**: executor writes tests per TddSpec. Reference grounded types/APIs. Evidence gate: tests compile but fail (right reasons, not syntax). Auditor validates test quality.
4. **EXECUTING**: executor writes code per TaskSpec. Context: ONLY BabyStepGrounding + task + test file. Every edit triggers T1/T2/T3 (4C). Incremental compilation (4B). Max lines TaskSpec.max_lines enforced.
5. **AUDITING**: auditor runs DeterministicCheck. Adversarial tests. BLOCK → AUDITING→EXECUTING. Max 3 audit retries per baby step.
6. **ACCEPTANCE check**: min_tests_passing met? If FAIL and attempts<max: go back to research for this step (ResearchAlternative) with failure context. New baby steps generated. Retry. If attempts≥max: BubbleUpHandler.
7. **INTEGRATION**: if !deferred AND all siblings complete: run integration_test_command. Pass → IntegrationCheckpoint. Fail → BubbleUp to step. Deferred: skip, run at step completion.
8. **EXIT CRITERIA**: each ExitCriterion.check. All pass → COMPLETE, navigator advances. Any fail → acceptance failure.

### Bubble-Up Failure Handling
```
fn handle_failure(baby_step_id, policy) -> BubbleUpAction {
    if attempts < max_retries_per_baby_step: ResearchAlternative  // try different approach
    let step = parent_step; if step_failures < max_retries_per_step: RedecomposeStep
    let phase = parent_phase; if phase_failures < max_retries_per_phase: ReplanPhase
    let concept = parent_concept; if concept_failures < max_retries_per_concept: ReplanConcept
    return policy.on_concept_fail  // default Escalate
}
```
Actions trigger re-research:
- ResearchAlternative: FlashResearch on baby step with failure context
- RedecomposeStep: FlashResearch on step, regenerate baby steps
- ReplanPhase: FlashResearch on phase, regenerate steps + baby steps
- ReplanConcept: FlashResearch on concept, regenerate phases/steps/baby steps
- Escalate: INTERACTIVE → human. AUTONOMOUS → ReplanRequest(MAJOR).

All bubble-up events → SSB action_type='replan_request' with tree context.

### Integration Checkpoints (4 granularity levels)
- BABY STEP: cargo check on affected files (<5s)
- STEP: cargo test on module (<30s)
- PHASE: cargo test -p crate (<60s)
- CONCEPT: cargo test --workspace -- concept_integration (<120s)

Test stubs generated by L2 builder from IntegrationSpec. Executor fills in test bodies during TESTING_FIRST of first baby step with integration tests. Initially fail (TDD), pass as baby steps complete.

### 4E.7 DECOMPOSITION TREE SKELETON — add all recursive types + stubs.
### 4E.8 TREE NAVIGATION + BUBBLE-UP TESTS (≥18, all FAIL)
Linear tree visits in order. Branching tree respects topological order. Dep skip. Pre-gate block. Completion returns None. Crash recovery via serialize/deserialize. Bubble-up: 1→ResearchAlternative, 3→RedecomposeStep, 2 step failures→ReplanPhase, 1 phase failure→ReplanConcept, concept failure→Escalate (default). Custom policy. Integration checkpoints. Deferred integration skipped. Step completion integration. Phase completion integration. Integration failure → BubbleUp to step. Proptest: random tree → valid topo; random failures → terminate.
### 4E.9 BABY STEP EXECUTION CYCLE TESTS (≥15, all FAIL)
Context assembly (<1K tokens in Slot 3). Pre-gate pass/fail. TDD tests-before-code. Tests fail correctly. EXECUTING T1/T2/T3 fire. AUDITING deterministic checks. ACCEPTANCE pass/fail → ResearchAlternative / BubbleUp. Integration deferred/not. Exit criteria pass/fail. Max lines via T2.
### 4E.10 IMPLEMENT NAVIGATOR + BUBBLE-UP
DepthFirstNavigator: topological sort concepts→phases→steps→baby steps. Pre-gate via BashSandbox (tools/). Dep resolution via completed set.
BubbleUpHandler: count failures per level. Compare to policy. Trigger re-research at appropriate level. Emit ReplanRequest(MAJOR) on Escalate to FSM.
Baby step orchestration: assemble grounding → Slot 3 → drive FSM TESTING_FIRST→EXECUTING→AUDITING → record result → integration.
IntegrationCheckpoint recording. TreeNavigatorState → msgpack for crash recovery (after every baby step).

Adversarial: bubble-up re-research produces DIFFERENT query each time (failure context appended). Corrupted checkpoint → graceful recovery (re-traverse from SSB).

### 4E.11 IMPLEMENT INTEGRATION CHECKPOINTS
Baby step: cargo check affected files (or lang equivalent). Step: step-level test command. Fail → BubbleUp step. Phase: phase-level integration. Fail → BubbleUp phase. Concept: concept-level e2e. Fail → BubbleUp concept. All results to SSB.

### 4E.12 MEASURE
Baby steps per idea (avg/median/max). FSM cycles per idea. Bubble-up rate (% escalated). Integration pass rate first time. Traversal overhead vs agent time. Context per baby step p50/p95 (<2K tokens). Total execution time per idea. Cache effectiveness.

Target: context p95 <2K tokens. Traversal overhead <5%.

## Exit Criteria (4E)
PipelineSpec → ExecutablePipeline deterministically. 8 validations enforced. DAG repair within T_DAG=10 rounds. Novel steps detected for 4F. Test harness from QualitySpec. No LLM in L2. ≥30 tests + integration + adversarial + property pass.

---

# PHASE 4F — FEASIBILITY_PROBE — IMPROVED 2026-04-06

Prereq: 4E done (L2 assembles ExecutablePipeline deterministically). Status: NOT done.

GOAL: System probe detects hardware/resource constraints before execution. Novel step stubs (<50 lines) test feasibility in sandbox with branch-and-explore. Deterministic skip when zero novel steps. Escalation after max 3 alternatives per novel step. Output: stub_results.msgpack (FeasibilityReport, architecture-overview.txt S5).

READ: orchestrator.txt S2 (FSM phase-aware loading FEASIBILITY tools), S10 (branch-and-explore), S12 (Tier C IPC), architecture-overview.txt S5 (data flow) + S3 (FSM skip PLANNING→TESTING_FIRST), pipeline-factory.txt S3 (Stage 4 = feasibility probe), providers.txt S4 (ModelPool VRAM, can_fit_onnx), reasoning.txt S10 (FeasibilityOutcome, InlineFeasibilityResult), risk-register.txt R4/R5/R22/R50 (VRAM contention).

## Types (new, not in spec, propagate to orchestrator.txt after approval)

```
struct SystemProbeResult { gpu_count:u32, gpu_info:Vec<GpuInfo>, ram_total_bytes:u64,
    ram_free_bytes:u64, disk_free_bytes:u64, os:String, is_supported:bool }
struct GpuInfo { index, name, total_bytes, free_bytes, driver_version,
    uuid:String (stable cross-reboot, nvml device.uuid()) }
struct SystemConstraints { min_vram_bytes, min_ram_bytes, min_disk_bytes, required_gpu_count }
struct StubResult { step_id, alternative_index:u32 (0-2), outcome:FeasibilityOutcome,
    elapsed_ms, stdout_hash:[u8;32] (blake3), stderr_summary:String (first 512 bytes),
    resource_delta:ResourceDelta }
struct ResourceDelta { vram_before/after, ram_before/after }
struct FeasibilityReport { probe, constraints, constraint_results:Vec<ConstraintResult>,
    stubs:Vec<StubResult> (one per passing novel step; escalated in escalations),
    skipped_steps:Vec<String>, escalations:Vec<FeasibilityEscalation> }
struct ConstraintResult { resource:String ("vram_gpu0","vram_gpu1","ram","disk"),
    required, available, margin:f64 ((available-required)/required), passed:bool }
struct FeasibilityEscalation { step_id, reason, stubs_tried:Vec<StubResult> (all 3 failed) }
enum FeasibilityDecision { Proceed | ProceedWithConstraints(Vec<String>) |
    Escalate(Vec<FeasibilityEscalation>) | Replan(Vec<FeasibilityEscalation>) }
```
Serialization: msgpack (disk), postcard (IPC wire). FeasibilityOutcome reused from reasoning crate (Phase 2D).

## Skip Criteria (deterministic)
FEASIBILITY_PROBE skipped PLANNING→TESTING_FIRST when ALL:
1. `PipelineSpec.novel_steps.is_empty() == true` (L2 marks novel when tool/approach not in capability registry)
2. ALL resource constraints met with ≥20% margin: `resource_free >= resource_required * 1.20`. Applied to VRAM (per-GPU), RAM, disk. All three must pass.

Skip logged to SSB action_type='phase_transition' with skip_reason. Grounding: architecture-overview.txt L139.

## Risks
- R-4F-1: nvml-wrapper requires NVIDIA driver. Nvml::init() Err on no GPU → graceful handling (gpu_count=0, empty vec).
- R-4F-2: sysinfo refresh overhead. Keep single System instance, call refresh_specifics with needed categories only.
- R-4F-3: VRAM readings point-in-time. Another process may allocate between probe and stub. Re-check via ModelPool.available_vram() (nvml memory_info().free) before stub launch.
- R-4F-4: Windows named pipe IPC latency may differ. Same IPC as 4A Tier C.
- R-4F-5: Stub sandbox escape. Tier C isolation + FEASIBILITY tool restrictions (stub write + bash + web search ONLY, NO edit/git).

## Steps

### 4F.1 SKELETON
PRE_GATE: 4E exit criteria met; `grep -c "pub enum FeasibilityOutcome" .elai_cc/crates/reasoning/src/types.rs == 1`; elai_cc_agent binary compiles.

```
trait FeasibilityProbe {
    fn probe_system(&self) -> Result<SystemProbeResult>
    fn check_constraints(&self, probe, constraints) -> bool
    fn run_stub(&self, step, alternative:u32, timeout:Duration) -> Result<StubResult>
    fn decide(&self, report, mode:ExecutionMode) -> FeasibilityDecision
}
enum ExecutionMode { Interactive, Autonomous }  // defined in orchestrator::types
```

Stub harness uses EXPLICIT AgentDispatchConfig overrides (not default Executor):
- GRC budget: 3/600 (not 8/1200)
- ToolFilter: allowed_tools = [stub_write, bash] only (NO edit/git)
- Sandbox: 25s graceful / 30s hard kill
- Strict <50 lines per stub (enforced before execution)

DispatchStage.agent is AUTHORITATIVE. Code checking role==Executor must also check dispatch config (technical-analysis-v3 Concern 5, BLOCK 7).

IPC: reuse wire protocol (1-byte version + 4-byte length + postcard).

SSB branch per stub alternative (tag="branch:stub_{step_id}_{n}").

Adversarial: nvml unavailable → gpu_count=0 gracefully. Report serialized size <10MB (MAX_FRAME_SIZE).

### 4F.2 VERIFY — cargo build/test workspace clean, no shadowing.

### 4F.3 TEST (16 contract/integration, all FAIL)
- T1: SystemProbe detects GPU count/VRAM via nvml-wrapper. Mock: 2 GPUs, 12GB each.
- T2: detects RAM + disk via sysinfo. Mock: 32GB RAM, 100GB disk. Real sysinfo in 4F.5 (live smoke test).
- T3: probe meets constraints → true. 12GB VRAM vs 8GB min → true.
- T4: probe fails → false. 4GB vs 8GB → false.
- T5: Stub executes in sandbox, returns Pass. Trivial println → Pass, <30000ms.
- T6: Stub at exactly 50 lines REJECTED (strict <50).
- T7: Stub timeout 30s hard kill. Infinite loop → Fail, ~30000ms.
- T8: Max 3 alternatives → Escalate with 3 StubResults.
- T9: SSB branch per alternative. 2 stubs same step → branch:stub_{id}_0 and _1.
- T10a: Skip both conditions met → no stubs run.
- T10b: No novel steps BUT margin<20% → probe runs (resource probe only).
- T10c: margin≥20% BUT novel steps exist → probe runs (stubs execute).
- T11: FeasibilityReport msgpack round-trips.
- T12: Stub sandbox restrictions — requests EditFile → Denied.
- T13: VRAM re-check before launch — below threshold → NOT launched, Fail with reason="insufficient_vram".
- T14: Dual timeout 25s graceful / 30s hard. Stub ACKs graceful and exits clean. Assert stdout contains "graceful_ack", elapsed 25000-30000ms.
- T15: ProceedWithConstraints — 1 GPU 8GB, constraints prefer 2 GPUs min 1. Stubs pass on single. → ProceedWithConstraints(["single_gpu_fallback"]).
- T16: Malformed IPC frame → synthesized StubResult with outcome=Fail, stderr_summary contains "malformed_ipc", step_id + elapsed_ms populated.

### 4F.4 ADVERSARIAL (8)
A1: FS escape /tmp or .elai_cc/ → blocked Tier C. A2: network not-researcher → seccomp/AppContainer blocks. A3: malformed postcard → orchestrator rejects frame, stub Fail (not crash). A4: Concurrent stubs same GPU VRAM contention → second waits or Fail, not OOM. A5: no GPU → gpu_count=0, constraints_met=false if GPU required. A6: output >MAX_FRAME_SIZE → truncated/rejected, no OOM. A7: VRAM dropped between probe and launch → re-check catches, stub NOT launched, Fail reason="vram_dropped". A8: 4th stub after 3 branches → escalation, no 4th branch created.

All timing tests use injected time, not real clocks.

### 4F.5 IMPLEMENT
PRE_GATE: 24 tests exist + failing. nvml-wrapper + sysinfo in Cargo.toml (`grep "nvml-wrapper" .elai_cc/Cargo.toml && grep "sysinfo" .elai_cc/Cargo.toml`).

I1 SystemProbe:
- GPU: nvml-wrapper `Nvml::init()` → `device_by_index(n)` → `device.memory_info()` (MemoryInfo { total, used, free }). `device.name()`, `device.uuid()`. Handle Nvml::init() failure (no GPU).
- RAM: sysinfo System with `refresh_specifics(RefreshKind::nothing().with_memory(MemoryRefreshKind::everything()))`. `total_memory()`, `used_memory()`. Disks separate: `Disks::new_with_refreshed_list()` → `disk.available_space()`. Check `sysinfo::IS_SUPPORTED_SYSTEM`. NOTE: RefreshKind does NOT support BitOr — use builder.
- Single System instance (recreation expensive).
- Pin sysinfo 0.37+ (MSRV 1.88.0).

I2 Constraint checker: pure function. 20% margin.

I3 Stub harness: reuse 4A agent with role=feasibility, FEASIBILITY set. Line count: count newlines, reject if >=50 (strict <50, matches pipeline-factory.txt S4).
Dual timeout: `tokio::time::timeout(Duration::from_secs(30))`. At 25s: graceful shutdown IPC. At 30s on Elapsed: `child.kill().await` EXPLICITLY (tokio's kill_on_drop false by default, verified tokio docs + issue #2685, dropping Child does NOT kill). Windows: TerminateProcess via child.kill(). Linux: SIGKILL. After kill: verify dead (waitpid/GetExitCodeProcess).
Resource delta probed before/after. Stdout captured, blake3 hashed.

I4 SSB branch: pre-stub snapshot tag="branch:stub_{step_id}_{n}". After winner merge; discard losers. Max 3 per step.

I5 Decision: all pass → Proceed. Pass with constraints → ProceedWithConstraints. 3 fails → Escalate (INTERACTIVE) or Replan (AUTONOMOUS).

I6 Skip: check novel_steps.is_empty() AND margin≥20%. Log SSB, transition PLANNING→TESTING_FIRST.

I7 Report: msgpack (rmp-serde) → stub_results.msgpack. Round-trip verified.

### 4F.6 MEASURE (9 metrics → .elai_cc/metrics/phase_4F.toml)
system_probe_latency_ms (target <500), stub_execution_p50_ms, stub_execution_p99_ms (<30000), constraint_check_latency_us (<100), report_serialization_bytes, skip_criteria_evaluation_us, test_count (24), test_pass_count (24), compile_time_delta_s (<10 from 4E).

## Testable Invariants
1. SystemProbe never panics on missing GPU (gpu_count=0)
2. Constraint check pure (no I/O)
3. Stub line count enforced before exec (>=50 = rejected, strict <50)
4. Stub hard-killed at 30s (no runaway)
5. Max 3 SSB branches per novel step
6. Escalation after 3 fails (never 4th)
7. Skip criteria deterministic
8. FeasibilityReport msgpack round-trip lossless
9. Stub sandbox enforces FEASIBILITY tool restrictions
10. VRAM re-checked immediately before launch
11. FeasibilityDecision matches mode (Escalate=INTERACTIVE, Replan=AUTONOMOUS)
12. stdout hash (blake3) deterministic for identical bytes

## Dependencies
Crates added: nvml-wrapper, sysinfo, blake3 (if not present). rmp-serde already workspace-wide. FeasibilityOutcome reused from reasoning crate.

## Baby Step Feasibility Extension (4F.7-4F.8)

Base probes top-level novel steps. Extension probes at baby step level within DecompositionTree.

DESIGN: After DecompositionTree built (4E), before execution, system identifies baby steps using novel packages/frameworks/APIs. Each novel baby step gets <50 line stub.

Novel baby step detection:
- PackageRef with verified=false → novel
- Framework not previously used in this project → novel
- System capability not yet tested (GPU, FFI) → novel

Feasibility runs PER CONCEPT (not all at once): before executing concept N, probe all novel baby steps in concept N. More efficient: concept 2's approach may change based on concept 1 results.

Skip criteria per concept: no novel baby steps in concept → skip.

### New Types
```
struct BabyStepFeasibilityReport { concept_id, probed:Vec<BabyStepStubResult>,
    skipped:Vec<BabyStepId>, escalations:Vec<BabyStepEscalation> }
struct BabyStepStubResult { baby_step_id, package_refs:Vec<PackageRef>, stub_result:StubResult }
struct BabyStepEscalation { baby_step_id, reason, alternatives_tried:Vec<BabyStepStubResult> }
```

### 4F.7 BABY STEP FEASIBILITY TESTS (≥6)
Verified package → not probed. Unverified → probed. Pass → proceed. 3 fails → BabyStepEscalation. Per-concept isolation: concept 2 not probed until concept 1 execution starts. All verified → entire concept feasibility skipped.

### 4F.8 IMPLEMENT + MEASURE
Novel baby step detection from BabyStepSpec. Per-concept feasibility loop. Stub gen from BabyStepSpec (extract import + usage). Reuse base FeasibilityProbe for stub exec. Measure: novel detection rate, probe pass rate, time per concept.

## Exit Criteria (4F)
24 tests pass (16 contract + 8 adversarial). System probe detects constraints. Stubs validate in Tier C with branch-and-explore. Escalation works INTERACTIVE + AUTONOMOUS. Skip criteria prevents unnecessary probing. FeasibilityReport persisted. Per-concept feasibility operational.
