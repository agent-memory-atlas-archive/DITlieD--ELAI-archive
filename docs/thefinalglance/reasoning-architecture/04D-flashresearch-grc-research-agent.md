# FlashResearch GRC — Research Agent + Designer (Phase 4D)

- **ELAI ids:** phase-4D / P4D-E001..P4D-E057 / P4D-I001 / P4D-A001..A004 / P4D-C001
- **Spec status (as designed):** WIRED (`ELAI-FEATURE-MAP.md:2728-2732`)
- **Crate / module (spec):** `crates/orchestrator` (`research_tree.rs`, Designer + Researcher roles in `agent_pool.rs`); Research Agent GRC **10/1200**; Designer GRC **8/1200**; web search via IPC (`web_search` crate, 4D.11+); isolation Phase 4A
- **Purpose of this file:** enough architecture to rebuild the GRC-grounded FlashResearch organ — per-node FORMULATE→SEARCH→EXTRACT→REFLECT→CONVERGE, dual φ/ψ thresholds, deterministic source quality, Designer PipelineSpec from ResearchReport, recursive Idea→…→BabyStep research. Not a SearXNG encyclopedia and not a status ledger.

## 1. What it is

**Pipeline Factory L1** is a **Research Agent** that designs pipelines from task descriptions via **tree-structured GRC-grounded research** (FlashResearch, arXiv:2510.05145), producing `PipelineSpec` + `QualitySpec` through a structured `ResearchReport` (`phase-4D.txt:1-10`, P4D-E002). A **Designer** role, distinct from Executor, consumes that report and emits plan-only TOML (`phase-4D.txt:28-35`, P4D-E005). GRC budgets: Research Agent **10/1200**, Designer **8/1200** (`phase-4D.txt:27-30`, `:119`, P4D-E004, `ELAI-FEATURE-MAP.md:2728-2730`). The LLM never controls traversal: the orchestrator / later `TreeNavigator` does (`phase-4D.txt:340-344`, P4D-E028). Prerequisite: Phase 4A Tier C isolation (`phase-4D.txt:2`, P4D-E001).

## 2. Why it exists

Without this organ an executor “just implements” from training: unverified packages, unmeasurable quality metrics, unbounded search trees, and ALL-TRAINING reports that look like research (`phase-4D.txt:217-218`, `:264-266`). If GRC budget is not 10/1200, the Research Agent exhausts mid-tree (`phase-4D.txt:116-119`). If Designer is secretly Executor, it implements instead of planning (`phase-4D.txt:28-36`, `technical-analysis-v3.txt Concern 5` as cited). If φ∨ψ is used instead of φ∧ψ, low-quality answers converge (`phase-4D.txt:279-280`). If source quality is LLM-scored, tiers are nondeterministic (`phase-4D.txt:115-116`, `:711`). B-40 later **reuses** this tree as multi-root presets; 4D is the engine (`B-40_idea_framer_research_presets.md:30-33`).

## 3. How it works (runtime)

### A. Roles and isolation

1. **Who.** Orchestrator spawns Research Agent (Tier C) and Designer. Research Agent: cloud preferred or Gemma4; GRC **10/1200** (`phase-4D.txt:27`). Designer: `AgentRole::Designer` (not Executor); **reuses executor model** (Gemma 4 31B); GRC **8/1200**; `required_steps: [Observe, Hypothesize, PlanAction, Verify, Converge]`; role_prompt plan-only TOML PipelineSpec — **do not implement** (`phase-4D.txt:28-35`).
2. **Tier C walls (fail-closed)** (`phase-4D.txt:86-90`, P4D-E014): network = web search **only** via orchestrator IPC proxy; GRC verify via IPC `GroundingRequest` (reasoning.txt §9); **no** FS to `.elai_cc/` or project source; model via IPC `LlmProvider`. Agent must not send `GroundingResult` (only `GroundingRequest`) (`phase-4D.txt:224`). Direct sockets: Unix `lsof -i` empty / Windows no TCP/UDP (`phase-4D.txt:288-291`).
3. **Designer Observe input.** `SystemConstraints { model_max_context, available_for_work, avg_file_tokens, avg_test_tokens }`. Index `< 10` files → `elai_cc.toml [planning.sizing]` defaults `default_avg_file_tokens=150`, `default_avg_test_tokens=80`; else `knowledge_engine.index_stats()` (`phase-4D.txt:37-45`, P4D-E006). Context budget constraint is an Observe input (`phase-4D.txt:44`).

The model generates sub-questions, queries, extracts, φ/ψ estimates, PipelineSpec text. It may **not** open sockets, score source tiers, pick the next node, or emit ungrounded package names.

### B. Per-node research loop (the GRC organ)

4. Adaptive complexity **before** the tree (pipeline-factory.txt L113-118; **no single hardcoded φ/ψ**) (`phase-4D.txt:91-95`, `:136-137`, P4D-E015):

| QueryComplexity | breadth | depth | phi_min | psi_min |
|---|---|---|---|---|
| Simple (single concept) | 2 | 3 | 0.8 | 0.7 |
| Moderate (comparison) | 3 | 4 | 0.7 | 0.6 |
| Complex (novel domain) | 4 | 5 | 0.6 | 0.5 |

5. **Loop** (`phase-4D.txt:239-247`, P4D-E021, later wired P4D-E052):
   1. **FORMULATE** — GRC generates search query from sub-question + prior findings (IPC inference).
   2. **SEARCH** — web search top **N default 10** via IPC (`:242`); quality gate prefers Tier1–3 (`:1172`).
   3. **EXTRACT** — fetch page; **sampling filter**: read first **500 tokens**, score, full read only if above threshold (`:243-244`). Adversarial: 100 results must not imply 100 full reads (`:279`).
   4. **REFLECT** — GRC evaluates **phi** (goal satisfaction), **psi** (quality), contradictions, new sub-questions, information gain (`:245-246`).
   5. **CONVERGE / EXPAND / ITERATE** — dual threshold and limits (`:247`).
6. **Convergence (deterministic).** Node converges iff `phi >= phi_min AND psi >= psi_min` (`phase-4D.txt:167`, `:258-259`, P4D-E023). Lifecycle: `Pending → Active → (Converged | Pruned)` (`:249`). φ=0.8, ψ=0.1 must **not** converge (`:280`).
7. **Prune.** `information_gain_threshold` in `[0.1, 0.9]`; below for **3 consecutive** iterations → `Pruned` (`phase-4D.txt:96-97`, `:273`, P4D-E016). Resources of pruned branches freed immediately (`:102`, `:256`).
8. **Bounds (fail-closed).** `depth_limit` (default 5 in tests), `breadth_factor` (default 3 in tests), `max_iterations` per node, acyclic parent-chain check on every child spawn (`phase-4D.txt:163-166`, `:250-251`). Self-parent rejected (`:115`). `time_budget` checked in the loop — **no node starts after expiry** (not root-only) (`:272`, `:281`). Target `time_budget_ratio <= 1.1` (`:312`).
9. **Speculative (cloud only).** Trigger `phi > 0.4`; run top `speculative_depth` children before parent converges; parent direction change **cancels** children; **local** `speculative_depth` forced **0**; cost bound `speculative_depth * breadth_factor` (`phase-4D.txt:252-257`, P4D-E022).
10. **Parallelism.** Root children (independent sub-questions) parallel; **local models sequential per level** (one GPU); cloud parallel, rate-limit Dynamic Fan-Out; deferred aggregation node after tree converges (`phase-4D.txt:98-102`, `:267-270`, P4D-E017).
11. **Journal.** `research_step` per iteration (`phase-4D.txt:85`, `:271`, P4D-E018). Wired web path also `web_research_step` (`:1183`, P4D-E054).
12. **L1 output gate.** `PipelineSpec` may **ONLY** reference alternatives from `ResearchReport`. `QualityMetric` must have an automated check method; unmeasurable → **REJECT** (`phase-4D.txt:264-266`, `:326-327`, P4D-E024). ALL-TRAINING report → HIGH_RISK (`:217-218`). Contradictions **surfaced**, never silently resolved (`:170`, `:219`).
13. **GRC exhaustion.** Budget 1/100 on complex research → **forced Converge**, not hang (`phase-4D.txt:211`). After 4A, orchestrator counts `ThinkStep` / `GroundingRequest` (`04A-orchestrator-authoritative-grc-budget.md` §3).

### C. Recursive extension (same organ, four levels)

14. `research_decompose(idea, mode)` is **code-driven** (`phase-4D.txt:515-562`, P4D-E033):
    - Level 0 IDEA: `flash_research(idea, Complex)` → `ConceptDecomposition`; drop concepts under feasibility threshold (`:346-350`).
    - Level 1 CONCEPT: cache `Moderate` research; INTERACTIVE `present_options`; AUTONOMOUS `auto_select` by `evidence_level * source_quality` (`:531-535`, `:645-647`).
    - Level 2 PHASE: `Simple` + `verify_packages` (`API_REGISTRY_CHECK`) (`:540-546`, `:568-570`). LLM cannot invent packages.
    - Level 3 STEP→BabySteps: `Simple`; each `BabyStepSpec` 8-block; `overall_context_summary` max **500 tokens**; baby step `<2K` context (`:364-368`, `:421-423`).
15. `ResearchCache`: `BTreeMap` + LRU `max_entries` + `ttl_seconds` + `ClockProvider cached_at` (`:497-508`). SSB `research_decompose_level` per level (`:650`, P4D-E037).

### D. Source quality (not GRC, but the grounding classifier)

16. `SourceQuality` 5-tier **domain lookup**, no LLM, no async, no network (`phase-4D.txt:80-83`, `:120`, P4D-E013). Same URL → same tier across 100/1000 calls (`:115-116`, `:287`). Exact domain or subdomain, not substring (`arxiv.evil.com` ≠ arxiv) (`:223`). Ranking: `evidence_level.score() * source_quality_avg` (`:63`). Confidence: `source_count * avg_quality * agreement_ratio` (`:78`, P4D-E012).

## 4. Types, enums, objects

Copy from `phase-4D.txt:46-83` and recursive block `:370-513`. Do not invent fields.

```
ResearchTree {
  plan: ResearchPlan,
  state: SharedResearchState,
  root: NodeId,
  breadth_factor: u32,
  depth_limit: u32,
  max_iterations: u32,
  phi_min: f64,
  psi_min: f64,
  speculative_depth: u32,
  time_budget: Duration,
  // information_gain_threshold: f64  in [0.1, 0.9]  (phase-4D.txt:96-97)
}

ResearchNode {
  question: String,
  parent: Option<NodeId>,
  children: Vec<NodeId>,
  phi: f64, psi: f64,            // construction rejects > 1.0 (phase-4D.txt:195)
  status: NodeStatus,            // Pending | Active | Converged | Pruned
  info_gain: f64,
  iterations: u32,
}
NodeId: newtype u64

ResearchPlan { query, complexity: QueryComplexity, sub_questions }
QueryComplexity { Simple, Moderate, Complex }
SharedResearchState { nodes: HashMap<NodeId, ResearchNode>, next_id: NodeId }

ResearchTreeSnapshot { nodes: Vec<ResearchNodeSnapshot>, edges: Vec<(NodeId,NodeId)> }
ConvergenceStatus { Converged{phi,psi}, Partial{phi,psi,reason}, Failed{reason} }
Alternative { description, sources: Vec<SourceCitation>, evidence_level: EvidenceLevel }
Contradiction { claim_a, claim_b, source_a, source_b }
EvidenceLevel { Benchmark(1.0), OfficialDocs(0.8), BlogPost(0.6), Training(0.4) }

QualityMetric { name, target, unit, method, phase, hard_fail }
MediaSpec { output_name, format, resolution, ssim_threshold, vlm_check, output_type: OutputType }
OutputType { Web, Model3D, Audio, UiScreenshot, Animation, Image, Custom(String) }
PerfSpec { name, target, unit, method, sample_count }

ResearchReport {
  query, tree, sub_questions: Vec<SubQuestionResult>,
  total_sources_fetched, total_sources_used, total_iterations,
  total_nodes_explored, total_nodes_pruned, speculative_hit_rate,
  convergence_status: Vec<(String, ConvergenceStatus)>,
}
SubQuestionResult {
  question, answer, sources, confidence: f64, phi, psi, depth,
  alternatives, contradictions,
}
SourceCitation { url, extracted_quote, quality_tier: SourceQuality }
SourceQuality { Tier1(1.0), Tier2(0.8), Tier3(0.6), Tier4(0.4), Tier5(0.2) }

# recursive
ConceptDecomposition { idea, concepts: Vec<ConceptNode>, research, feasibility_scores: Vec<(String,f64)> }
ConceptNode { concept_id, name, description, research, phases, feasibility, tech_stack }
PhaseNode { phase_id, concept_id, name, research, steps, framework, packages, depends_on }
StepNode { step_id, phase_id, name, research, baby_steps }
BabyStepSpec { baby_step_id, step_id, grounding, pre_gate, task, tdd,
               deterministic_check, acceptance, integration, exit_criteria }
BabyStepGrounding { research_findings, overall_context_summary /*max 500 tok*/, required_types, required_imports }
TddSpec { ..., write_before_code: bool }           // always true
AcceptanceFailAction { BubbleUp, ResearchAlternative, Replan }
TechStackDecision { language, framework, packages, decision_source: Research | UserChoice | AutoSelected }
PackageRef { name, version /*pinned*/, registry, verified: bool }  // API_REGISTRY_CHECK
ResearchCache { entries: BTreeMap<String, CachedResearch>, max_entries, ttl_seconds }
ConceptId(u64); BabyStepId(u64); // StepId/PhaseId reuse 4E
```

`PipelineSpec` / `QualitySpec` full field schemas live in pipeline-factory.txt §10 beyond the metric/media/perf structs (P4D-A004).

## 5. Control flow / state machine

**Owner of the loop:** orchestrator research runner + `ResearchTree` invariants. GRC `think` is inside FORMULATE and REFLECT (and Designer Observe→…→Converge). After 4A, IPC `ThinkStep`/`GroundingRequest` counts are authoritative (`phase-4D.txt:88`, `04A-orchestrator-authoritative-grc-budget.md`).

**Forced:** first Designer real step Observe (2A); Research Agent budget 10/1200; dual-AND convergence; local speculative_depth=0; acyclic spawn; time_budget in-loop; L1 references ⊆ ResearchReport alternatives.

Recursive owner: `research_decompose` / TreeNavigator — **LLM never chooses the next research target** (`phase-4D.txt:340-344`).

B-40 FRAMING, if enabled, runs **before** Level 0 (`B-40_idea_framer_research_presets.md:18-23`). 4D does not own that substate.

## 6. Grounding, receipts, gates

| Mechanism | Receipt |
|---|---|
| GRC citations | 2A `type:reference` + 2B verify via IPC GroundingRequest |
| SourceQuality | deterministic domain table `[research.trusted_domains]` |
| EvidenceLevel | Benchmark > OfficialDocs > BlogPost > Training |
| PackageRef.verified | `API_REGISTRY_CHECK` (2B method family, P4D-E015 / P4D-E035) |
| QualityMetric | automated `method` or REJECT |
| ALL-TRAINING | HIGH_RISK flag |
| Contradictions | `SubQuestionResult.contradictions` must be populated |

**Never allowed as a confidence signal:** LLM-assigned source tier; φ OR ψ; training-only alternatives in PipelineSpec; invented `PackageRef`; speculative_hit_rate as quality; window size as GRC budget (10/1200 is calls/chars, not tokens — token budgets are `ResearchBudgetManager` / VRAM tiers, neighbors).

IPC: `SearchRequest`, `FetchRequest`, `EmbedRequest`, `InferenceRequest` — agent never direct network (P4D-E053).

## 7. Budgets, limits, ceilings

| Knob | Value | Scope |
|---|---|---|
| Research Agent GRC | **10 / 1200** | per topic |
| Designer GRC | **8 / 1200** | per topic; required_steps 5 listed |
| search top N | default **10** | per SEARCH |
| EXTRACT sample | first **500** tokens | per fetch |
| info_gain prune | 3 consecutive below threshold | per node |
| info_gain range | [0.1, 0.9] | config |
| depth/breadth test defaults | 5 / 3 | 4D.3 |
| speculative local | 0 | single GPU |
| baby-step context | < **2K** tokens | Level 3 |
| BabyStepGrounding summary | max **500** tokens | compression |
| time_budget_ratio | ≤ **1.1** | 4D.6 |
| grounding_distribution | **zero UNCERTAIN** in final report | 4D.6 accept |
| SEARCH rate (wired) | SearXNG max 1 req/s; fetch max 2 req/s/domain; max_body 2MB; timeout 15s | 4D.11–4D.12 neighbors |

VRAM tiers (4D.15) adjust breadth/depth/φ/ψ/`speculative_depth` (only **48gb** has speculative_depth=2) (P4D-E048) — neighbor config, not a second GRC.

## 8. Roles and neighbors

**Roles:** Researcher (Research Agent) 10/1200; Designer 8/1200 distinct prompt/steps/output (`phase-4D.txt:28-36`, P3C-E023). Executor must not absorb Designer.

**Consumes:** 2A–2C GRC, 4A IPC, 2B `API_REGISTRY_CHECK`, knowledge `index_stats`, ClockProvider (cache timestamps).

**Produces:** `ResearchReport`, `PipelineSpec`/`QualitySpec`, recursive tree types, SSB `research_step` / `research_decompose_level` / `web_research_step`.

**Must not fork:** a second research GRC inside the agent that overrules orchestrator counts; browser-automation research (2–3× cost; SearXNG API instead) (`phase-4D.txt:703-705`); LLM-controlled next-node.

**Neighbors (do not rebuild as this organ):** 4D.11–4D.18 web_search/SearXNG/WebFetcher/A-RAG/VRAM/`ResearchBudgetManager` (P4D-E039..E056); 4E L2 builder/TreeNavigator/BubbleUp; B-40 multi-root presets; B-42 cosmos_get; 4I MediaSpec consumers.

**Exit wiring:** every routing struct must wire `ssb`, `fsm`, `clock` through `RuntimeClient`; no `None` after construction (`phase-4D.txt:329`, P4D-E027).

## 9. Config / flags / defaults

```
[planning.sizing]
default_avg_file_tokens = 150
default_avg_test_tokens = 80

[research.trusted_domains]
[research.trusted_subdomain_rules]

[research.vram_tiers]   # 8gb / 12gb / 12gb_7b / 16gb / 24gb / 48gb
```

`--vram-tier` manual override; detect rounds **DOWN**; no-GPU → 8gb (P4D-E049). Decision + grounding docs required before EXECUTING: `workflow/reasoning/phase-4D-decisions.md`, `phase-4D-grounding.md` (`phase-4D.txt:316-318`).

No B-8-style disable flag in phase-4D.txt. Isolation is the security default.

## 10. Implementation map

| Symbol | Spec |
|---|---|
| Research Agent / Designer role config | `phase-4D.txt:27-36` |
| `ResearchTree` / `ResearchNode` / report types | `:46-83` |
| per-node loop 4D.5 | `:239-273` |
| recursive types + `research_decompose` | `:370-562` |
| `ResearchCache` + `PackageRef` | `:490-570` |
| metrics `.elai_cc/metrics/phase_4D.toml` | `:293-313`, `:660-678` |
| IPC GroundingRequest | `:88`, reasoning.txt §9 |
| `web_search` crate (neighbor) | `:717+` |

## 11. Journal / SSB events

| action_type | when |
|---|---|
| `research_step` | per research iteration (`:85`, `:271`) |
| `research_decompose_level` | per recursive level (`:650`) |
| `web_research_step` | per node iteration on wired web loop (`:1183`) |

These are **not** in Phase 3B’s 17-type CHECK list — registration path unspecified (P4D-A001, P4D-C001). Rebuild: emit the strings; schema migration is a neighbor obligation. INTERACTIVE visibility is via these events (`:325`, P4D-E057). `present_options` UI (CLI vs dashboard) unspecified (P4D-I001, P4D-A003).

## 12. Tests and observables

**4D.3** ≥14 contract invariants (depth/breadth, acyclic, dual-AND, deterministic quality, contradictions, speculative bounds, fetched vs used) + ≥3 integration + ≥3 proptest (`phase-4D.txt:158-203`, P4D-E019).

**4D.4** ≥8 adversarial: FS escape, direct network, GRC exhaustion, depth/breadth bombs, speculative prune, quality gaming, empty task, ALL-TRAINING HIGH_RISK, contradictions (`:206-232`, P4D-E020).

**4D.7–4D.10** ≥13 recursive tests: levels, cache, TTL, package verify, grounding <500, Interactive/Autonomous, cross-deps, proptest (`:574-678`, P4D-E036).

Metrics keys (≥9): `grounding_distribution`, `spec_token_count`, depth/breadth utilization, `convergence_p50`/`p95`, `speculative_hit_rate`, `source_tier_distribution`, `time_budget_ratio`, `ssb_event_count`, `info_gain_trend` (`:307-308`). Recursive append: `cache_hit_rate`, `baby_step_count`, `grounding_p50` (`:674`).

## 13. Rebuild contract

Another harness has **this** organ iff:

- Research Agent GRC **10/1200**; Designer **8/1200** with steps Observe/Hypothesize/PlanAction/Verify/Converge; Designer ≠ Executor; plan-only prompt.
- Tier C: IPC-only search/grounding/LLM; no project/`.elai_cc` FS; cannot forge GroundingResult.
- Types in §4 (tree, node, report, source quality 5-tier, recursive decomposition + 8-block BabyStepSpec).
- Per-node FORMULATE → SEARCH (top 10) → EXTRACT (500-token sample) → REFLECT (φ/ψ/contradictions/info_gain) → CONVERGE/EXPAND/ITERATE.
- Dual-AND φ/ψ; adaptive complexity table; info_gain 3-strike prune; acyclic; time_budget in-loop; local speculative 0.
- SourceQuality pure function; PipelineSpec ⊆ ResearchReport alternatives; unmeasurable QualityMetric REJECT; packages API_REGISTRY_CHECK.
- Recursive four levels, cache, INTERACTIVE present vs AUTONOMOUS max evidence_level×source_quality.
- `research_step` (and decompose/web variants) journaled.
- Tests: ≥14 tree + ≥8 adversarial + recursive suite.

A “search then write a plan” mega-prompt, MCTS-as-tree, or LLM-scored domains is a cousin.

## 14. Non-goals / banned designs

- Grounding **verification algorithms** (2B) and CSQ (2D) — Research Agent **calls** them over IPC.
- L2 deterministic builder / DAG repair / BubbleUp (4E).
- Browser automation as the retrieval layer (`phase-4D.txt:703-705`).
- LLM-controlled flow (`:340-344`).
- Hardcoded single φ/ψ ignoring complexity (`:95`, `:136-137`).
- Substring domain matching (`:223`).
- Speculative execution on local GPU (`:196`, `:256`).
- Silent contradiction merge (`:170`).
- `Ok` / network inside 2B network verifiers used as if 4D implemented WEB_CHECK in-process.
- Replacing 10/1200 with “until context full”.

## 15. Source citations

Primary: `docs/specs/guide/steps/phase-4D.txt` (4D.1–4D.10 GRC/tree/recursive; 4D.11+ cited as neighbors). Exhaustive: `ELAI-EXHAUSTIVE-SUPER-FEATURE-LIST.md` P4D-E001..E057, P4D-A001..A004, P4D-C001. Feature map: `ELAI-FEATURE-MAP.md:2728-2732`. Overview: `ELAI-SPEC-OVERVIEW.md:110-114`. Systems: `ELAI-SYSTEMS-OVERVIEW.md:178-186`. Pre-reads named by phase file: `pipeline-factory.txt` §§1–9, `reasoning.txt` GRC+IPC, `flashresearch-interface.md` (`phase-4D.txt:12-16`). Isolation: `04A-orchestrator-authoritative-grc-budget.md`. Downstream: `B-40_idea_framer_research_presets.md`, `B-41_per_level_auditor_modes.md`.

## 16. Gaps

Label **UNCERTAIN**; do not fill:

- `ResearchTree` field list in 4D.1 omits `information_gain_threshold` then 4D.1 task requires it (`:46-49` vs `:96-97`).
- Full `PipelineSpec`/`QualitySpec` schemas (P4D-A004).
- `present_options` concrete UI (P4D-I001, P4D-A003).
- SSB CHECK migration for `research_*` types (P4D-A001, P4D-C001).
- Several VRAM `context_ceiling` values projected vs MEASURED (P4D-A002).
- EXTRACT “above threshold” numeric cutoff for full-page read (`:244`).
- Whether FORMULATE/REFLECT are full 2A chains or collapsed GRC topics per node.
- Researcher `required_steps` / `allowed_methods` (budget 10/1200 specified; 2C-style table not in phase-4D.txt).
- `ResearchAlternative` type named in 4E bubble-up (P4E-E020) vs 4D `Alternative`.
- How B-40 `new_multi_root` shares `ResearchCache` vs one cache per stream (B-40 says per-stream slot).
