# ELAI — Research / Deep-Research Systems (Complete Reference)

A single consolidated reference for **every** research system, subsystem, plan, and code module in ELAI.
Compiled by reading three layers:

1. The top-level design spec (`ELAI-SPEC-OVERVIEW.md`).
2. The per-plan summary indexes (`docs/thefinalglance/{B..Q}-plan/*-SUMMARY_INDEX.md` + original plan
   files under each letter).
3. The live implementation under `.elai_cc/crates/` (every `file:line` anchor below is real code).

**The single most important framing**: "deep research" in ELAI is **not one system**. It is a layered
architecture of cooperating pieces, plus several sibling/derivative research systems (Personal Research
Machine, research-conditioned creativity, research-lane economics, diffusion research, hierarchical BAI).
The ground-truth explainer is `docs/thefinalglance/ELAI-DEEP-RESEARCH-EXPLAINED.md`; this file is a
broader map that also covers the sibling systems and the unshipped/REFUTED plans.

---

## 0. Quick status legend (used throughout)

| Tag | Meaning |
|---|---|
| **SHIPPED** | Live code on the production path, exercised by tests. |
| **WIRED/DARK** | Code present but gated off by an env var (e.g. `ELAI_WEB_RESEARCH_ORCHESTRATOR_ENABLE`). |
| **RUNPOD-PENDING** | Host-side scaffolding shipped; the LLM leg is dark until the sidecar is up. |
| **PLANNED** | Authored as a plan, not yet built. |
| **REFUTED** | Falsified on a benchmark; not built / not authorized. |
| **DEFERRED** | Research-grade; blocked on a missing piece. |

---

## PART I — The Core "Deep Research" Engine

This is what the `/research` and `/ask` slash commands run. Four cooperating pieces (from
`ELAI-DEEP-RESEARCH-EXPLAINED.md` §0):

| Piece | File | Role |
|---|---|---|
| **FlashResearch tree** | `.elai_cc/crates/orchestrator/src/research_tree.rs` (3108 lines) | Tree scaffold: complexity → breadth/depth, φ/ψ dual-threshold convergence, info-gain pruning, source-quality tiers, `ResearchReport` shape. Its `run()` loop is a **deterministic simulation**; real signals are injected by a grounding client. |
| **Web research orchestrator** | `.elai_cc/crates/orchestrator/src/web_research_orchestrator.rs` (1078 lines) | The **real** per-node loop that actually hits the web: FORMULATE → SEARCH → QUALITY GATE → FETCH+EXTRACT → INDEX → RETRIEVE → REFLECT → CONVERGE. |
| **Grounding transport** | `.elai_cc/crates/orchestrator/src/grounding_client.rs` (1756 lines) | The `GroundingClient` trait + impls (Sidecar/local, Cloud/Anthropic, Synthetic/test) that turn a prompt + `ResearchLevel` into a parsed `ResearchReport`. This is where an LLM is actually called. |
| **E-48 hardening substrate** | `.elai_cc/crates/orchestrator/src/research/web_orchestrator/` (10 modules) | Additive, wire-dark-by-default hardening: editable plan, tiered-search telemetry, speculative-branch pruner, conformal triple-threshold stop, contradiction classifier, 4-stage citation verifier, recency decay. |

Two more reuse the above: **`research_decompose`** (recursive Idea→Concepts→Phases→Steps grounded
decomposition, `research_tree.rs:2191`) and the **`ResearchCache`** (LRU+TTL memo,
`research_tree.rs:2041`).

### The key subtlety (read this or the rest looks contradictory)

There are two distinct "research trees" and they are NOT the same object:

- **`ResearchTree`** (`research_tree.rs:1145`) is the FlashResearch scaffold from arXiv 2510.05145.
  Its `run()` loop advances φ/ψ via a **deterministic sigmoid on iteration count**, not evidence
  (`process_node_iteration`, `research_tree.rs:1440`): `progress = 1/(1+e^(-(iterations-2)))`,
  `phi = progress·(phi_min+0.15)`. The header is explicit: *"no LLM calls — real inference happens
  via IPC GroundingRequest proxy (out of 4D scope)."* The standalone tree is the **shape +
  convergence math + report format**; it produces a structurally-valid but content-empty report.

- **`WebResearchOrchestrator`** (`web_research_orchestrator.rs:165`) is the thing that actually does
  research. It takes a `ResearchNode` and drives it through a real web loop, computing φ/ψ from
  **actual fetched citations** in `reflect()` (`:791`).

So: **the tree defines the data model and the convergence contract; the orchestrator fills a node with
real evidence against that contract; the grounding client is the LLM transport underneath both.**

---

## 1. The FlashResearch tree (`research_tree.rs`) — data model + convergence math

### 1.1 Complexity → budget (the only tuning knob that matters)

`assess_complexity(question)` (`research_tree.rs:1730`) is deterministic, no LLM: word count +
comparison markers (` vs `, ` versus `, ` compare `) → `QueryComplexity::{Simple,Moderate,Complex}`.
Complexity fans out to every budget constant (`research_tree.rs:230-254`):

| Complexity | breadth_factor | depth_limit | phi_min |
|---|---|---|---|
| Simple   | 2 (`BREADTH_SIMPLE`)   | 3 (`DEPTH_SIMPLE`)   | 0.80 (`PHI_MIN_SIMPLE`) |
| Moderate | 3 (`BREADTH_MODERATE`) | 4 (`DEPTH_MODERATE`) | 0.70 (`PHI_MIN_MODERATE`) |
| Complex  | 4 (`BREADTH_COMPLEX`)  | 5 (`DEPTH_COMPLEX`)  | 0.60 (`PHI_MIN_COMPLEX`) |

`psi_min` is flat 0.70 (`PSI_MIN_DEFAULT`, `:157`). Note: **harder question → LOWER φ bar** — the
design accepts weaker per-node certainty on complex questions because they fan wider. Hard caps
regardless of complexity: `MAX_TREE_DEPTH = 10`, `MAX_TOTAL_NODES = 200`, `MAX_ITERATIONS_DEFAULT = 50`
(`:166-179`), enforced in `add_child` (`:1649`, errors `MaxDepthExceeded`/`MaxNodesExceeded`).

### 1.2 The node and its two convergence scalars

A `ResearchNode` (`:545`) carries `phi`, `psi`, `status`, `info_gain`, `iterations`,
`low_gain_streak`, `citations`, `answer`, `depth`. Two scalars are the whole convergence story:

- **φ (phi)** = *goal satisfaction / coverage*. In production: average source-quality × a coverage factor.
- **ψ (psi)** = *quality / corroboration floor*. Average evidence-level × quality × coverage.

Convergence is an **AND, never an OR** (`ResearchNode::is_converged`, `:604`):
`phi >= phi_min && psi >= psi_min`. A node with great coverage but single-sourced evidence does not
converge, and vice-versa. `PartialEq` is hand-written with a NaN guard (`:562`).

### 1.3 Info-gain pruning ("stop digging a dry hole")

Each iteration computes `info_gain = |Δphi|`. Below `INFO_GAIN_THRESHOLD = 0.1` (`:160`) the node's
`low_gain_streak` increments; else resets to 0. At `LOW_GAIN_PRUNE_COUNT = 3` (`:163`), `should_prune()`
returns true (`:609`) → status `Pruned`. A branch that stops learning is abandoned, freeing budget for
productive branches.

### 1.4 Source quality: 5 deterministic tiers, exact-domain match

`assign_source_quality(url)` (`:323`) is deterministic and **anti-phishing**: extracts host
(`extract_host`, strips port, IPv6-safe), matches exact domain allow-lists (`domain_matches`, `:368` —
host equals the domain OR ends with `.domain`, so `arxiv.phishing.com` does NOT match Tier1). Tiers
(`:169-173, 287-319`):

| Tier | Score | Domains |
|---|---|---|
| Tier1 | 1.0 | arxiv.org, doi.org, ietf.org, w3.org, ieee.org, acm.org |
| Tier2 | 0.8 | docs.rs, crates.io, pypi.org, doc.rust-lang.org, docs.python.org, learn.microsoft.com, developer.mozilla.org |
| Tier3 | 0.6 | stackoverflow.com, github.com, gitlab.com |
| Tier4 | 0.4 | medium.com, dev.to, hashnode.dev |
| Tier5 | 0.2 | everything else (unverified) |

Orthogonally, `EvidenceLevel` (`:382`) ranks corroboration: Anecdotal 0.25 / SingleSource 0.5 /
MultiSource 0.75 / Verified 1.0. A single fetched page starts `SingleSource`; once ≥2 corroborate, all
citations on the node upgrade to `MultiSource`.

### 1.5 The deterministic `run()` loop (skeleton path)

`ResearchTree::run()` (`:1381`): start clock, activate root, add `sub_questions` as children, activate
all children, loop up to `max_iterations` — each iteration checks the time budget (`is_time_exceeded`,
`:1705`), collects active nodes, calls `process_node_iteration`. Exits when no nodes active or
budget/iteration cap hit, then `build_report()`. On the bare tree φ/ψ come from the sigmoid, not the web.

**Multi-root** (`new_multi_root`, `:1221`, B-40): several research "streams" each become a root node
processed by the same loop; `stream_root_id`/`multi_root_ids` look them up. **`split_step`** (`:1329`,
C-9) realizes an autonomous-gate `SplitTaskPlan` against the tree (depth-capped, emits `SplitTaskFailed`
on empty).

### 1.6 The output: `ResearchReport`

`build_report` (`:1483`) assembles the `ResearchReport` (`:1085`): `question`, `answer`, `alternatives`,
`contradictions`, `sub_results` (one per child node with confidence + evidence level), a `snapshot` of
every node, `quality_metrics` (a `convergence_ratio` metric threshold 0.5, and a `time_budget_ratio`
metric threshold 1.1), and an overall `ConvergenceStatus`.

`ConvergenceStatus` (`:862`): `Converged` (all nodes) / `Partial{converged,total}` / `Failed{reason}` /
**`PreemptiveUnsolvable{...}`** (E-48 triple-threshold arm, §I.7.4). `compute_convergence_status`
(`:1551`) picks between the first three by counting statuses.

Contradictions: `find_contradictions` (`:1576`) is the LEGACY stub — emits at most one sample
contradiction from the first two converged nodes, NOT prune-protected, superseded by the E-48
`contradiction_classifier` (§I.7.5). Only runs when `plan.require_contradictions` is set.

### 1.7 Robustness: tolerant deserialization (LLMs emit messy JSON)

Because the tree is reconstructed from LLM JSON, three tolerant deserializers keep one bad element from
nuking a whole report:
- `lenient_vec` (`:488`) — for `alternatives`/`contradictions`/`sub_results`/`quality_metrics`; parses
  to `Vec<Value>` first, keeps only elements that round-trip, drops the rest with `warn!`. Origin: a
  real cloud-Sonnet run (94dcc5b) where one bare-string entry aborted a 5KB `ResearchReport`.
- `deserialize_nodes_tolerant` (`:909`) — per-node skip on missing `id`/`question`/`depth` (Qwen-3B
  often diverges from the schema), tolerates `nodes: null`.
- `deserialize_convergence_status` (`:725`) — accepts unit-strings (`"Partial"`) or the struct form,
  lossy-fills counts with a `warn!`, aliases legacy `"Diverged"` → `Partial{0,0}`.

---

## 2. The 7-Phase Oracle Research Pipeline (System 1 — linear citation Q&A)

Linear, single-shot, citation-grounded Q&A engine. This is the **other** research engine in the
codebase — distinct from the FlashResearch tree.

**File:** `.elai_cc/crates/orchestrator/src/research_pipeline.rs` (1493 lines)

Header (verbatim):
```
// A-43: Oracle research pipeline — 7-phase research engine.
// Phases: A (query formulation) -> B (search execution) -> C (source tier classification)
//      -> D (fact extraction) -> E (cross-verification) -> F (synthesis) -> G (post-verification).
// D3: Phase F is now LLM-backed via GroundingClient with deterministic fallback.
```

Key symbols:
- `pub trait SearchBackend` (`:49`) — pluggable; `fn search(&self, query, max_results) -> Result<Vec<SearchResult>, ResearchPipelineError>`.
- `pub fn select_research_search_backend(sidecar_url)` (`:101`) — tries `SearxSearchBackend`, falls back to `StubSearchBackend`.
- **Phase A** `phase_a_formulate_queries(query)` (`:142`) — structural rephrasing into 2-4 queries.
- **Phase B** `phase_b_search(queries, backend, max_sources)` (`:184`) — executes searches, dedupes by URL.
- **Phase C** `phase_c_classify(collected, config, required_tier)` (`:227`) — classifies into T1/T2/T3/T4 via `elai_core::source_tier::classify_source`.
- **Phase D** `phase_d_extract_facts(classified)` (`:276`) — one fact per result from snippet.
- **Phase E** `phase_e_cross_verify(facts)` (`:319`) — groups by normalized claim, counts independent T1/T2 domains with exact quotes; assigns `VerificationLevel::{Verified,Supported,Weak,Unsupported}`.
- **Phase F** `phase_f_synthesize<C: GroundingClient>(query, claims, classified, grounding)` (`:407`) — LLM-backed synthesis via `grounding.synthesize_text()`, deterministic concatenation fallback when the LLM emits no `[N]` citation markers or returns `DETERMINISTIC_FALLBACK`.
- **Phase G** `phase_g_post_verify(answer, query)` (`:571`) — citation completeness, tier consistency, "I don't know" enforcement, hallucination guard.
- `pub fn run_research_pipeline<C: GroundingClient>(query, config, backend, grounding)` (`:654`) — top-level driver wiring A→G. Phase F is bridged sync→async via `phase_f_synthesize_blocking` (`:694`, current-thread Tokio runtime).

**Shared data model:** `.elai_cc/crates/core/src/research_types.rs` — `ResearchQuery`, `ExtractedFact`
(blake3 id via `generate_fact_id`), `VerificationLevel`, `VerifiedClaim`, `AnswerConfidence`,
`ResearchAnswer`. Constants: `MAX_SEARCH_QUERIES=4`, `MIN_SEARCH_QUERIES=2`, `VERIFIED_MIN_SOURCES=2`,
`SUPPORTED_MIN_SOURCES=1`, `MAX_UNCITED_SENTENCES=0`.

**Source tiering foundation:** `.elai_cc/crates/core/src/source_tier.rs` —
`SourceTier::{T1Authoritative, T2Strong, T3Weak, T4Rejected}`, `ClassificationMethod`,
`SourceClassification`, `SourceTierConfig` (compiled from `config/source-tiers.toml`).

---

## 3. The real per-node web loop (`web_research_orchestrator.rs`)

This is where `/research` actually spends its time. Constructed from six infra crates (`:1-8, 194`):
`SearxClient`, `WebFetcher`, `BasicExtractor`, `SourceQualityTable`, `WebContentIndex`,
`ResearchBudgetManager`, plus a `ResearchTierConfig`. Production wiring:
`DefaultWebResearchRunner::build_orchestrator` (`task_runner.rs:2193`) builds all six; `run()`
(`task_runner.rs:2280`) creates a single root `ResearchNode(1)` and calls `run_node_loop`.

### 3.1 The loop stages (`run_node_loop`, `web_research_orchestrator.rs:278`)

Max iterations = `tier_config.depth_limit × MAX_ITERATIONS_PER_DEPTH(10)` (`:284`). Per iteration:

1. **FORMULATE** (`:344` → `formulate_query`, `:639`) — first pass: raw question. Later passes: append
   `-site:{domain}` exclusions for up to 3 already-visited domains so search **diversifies**.
2. **SEARCH** (`:354` → `search_and_rank`, `:670`) — `SearxClient.search(query)`, results sorted by
   source-quality score DESC, ties broken by engine relevance score. Per-tier histogram rides the
   `search_completed` SSB stage row (`:372`).
3. **QUALITY GATE** (`:404`) — partition high-quality (T1–T3) vs low-quality (T4); fetch high-quality
   always, append low-quality **only if budget remains** (`budget_manager.remaining() > 0`), then
   truncate to `tier_config.max_sources_per_node`.
4. **FETCH + EXTRACT** (`:429` → `fetch_and_extract`, `:757`) — batched by `max_concurrent_fetches`.
   Each URL passes the **egress gate first** (3.2), then reqwest or obscura (3.3), then `BasicExtractor`;
   an `Empty` page is an error and skipped. Every fetched body is **taint-registered** (3.4).
5. **INDEX** (`:463`) — estimate tokens (`EvidenceItem::estimate_tokens`); if budget `can_admit`,
   ingest a `WebChunk` into the `WebContentIndex`, `admit` tokens, push a `SourceCitation`
   (tier→quality via `source_tier_to_quality`, `:934`; evidence starts `SingleSource`, excerpt
   truncated to 200 chars). Budget-exceeding chunks are skipped (`:465`).
6. **corroboration upgrade** (`:493`) — at ≥`MULTI_SOURCE_THRESHOLD(2)` citations, every `SingleSource`
   citation is promoted to `MultiSource`.
7. **RETRIEVE** (`:503`) — `content_index.keyword_search(question, breadth_factor)`, the A-RAG
   progressive disclosure step over what's been indexed so far.
8. **REFLECT** (`:518` → `reflect`, `:791`) — recompute φ/ψ from citations (3.6). `info_gain = φ−φ_old`;
   `low_gain_streak` increments below `INFO_GAIN_THRESHOLD`, resets otherwise.
9. **CONVERGE** (`:530`) — converge iff `is_converged(phi_min, psi_min) && citations.len() >=
   MIN_SOURCES_FOR_CONVERGENCE(2)`. Else if `should_prune()` → prune. Else iterate. On max-iterations
   without either, stays `Active` and emits `max_iterations` step (`:606`).

Empty search results (`:377`) bump `low_gain_streak` and can prune the node even with zero fetches —
a question the web can't answer dies on the streak rule, not a crash.

### 3.2 Egress blast-radius gate (G-3, fail-closed)

`enforce_egress` (`:712`) runs BEFORE any connect inside `fetch_and_extract`. Rejects NUL/control-char
hosts pre-resolution, then confirms **every** resolved IP is inside the pinned allow-set
(`pinned_egress_ips`, `:695` — union of every resolved `ResearcherNetConfig.allowed_hosts` IP plus
trusted resolvers). A `BlastRadiusVerdict::Block` refuses the fetch and emits a
`blast_radius_blocked{EgressDenied}` SSB row (the off-agent artifact). Kill-switch
`ELAI_BLAST_RADIUS_DISABLE` disarms; default-ON. A host that fails to resolve contributes no IPs, so it
can never match — **fail-closed by construction**.

### 3.3 Fetcher backends

Default reqwest (`WebFetcher`, tag `"reqwest"`). With the `obscura` cargo feature AND
`ELAI_FETCHER=obscura`, a headless-browser `ObscuraFetcher` is used (tag `"obscura"`) for JS-heavy
pages (`task_runner.rs:2227`, `web_research_orchestrator.rs:764`). Feature-off + env-on logs a warning
and falls back to reqwest (`task_runner.rs:2244`).

### 3.4 Taint threading (G-1)

Each fetched page body is `tainted_context().register(&text, Provenance::from_web(url))` (`:443`) —
untrusted web bytes go into a process-local taint side-table so a later destructive sink whose argv
derives from that body is blocked by the capability gate. Default-ON (no env gate on the register;
kill-switch is at the sink).

### 3.5 Tool-set commit (F-3)

Once per `run_node_loop`, before the first untrusted-byte boundary, `commit_minimal_set(task_id,
[Web, Read], now)` locks the researcher to a minimal tool set (`:301`) and (if an SSB writer is
attached) emits `ToolFilterEvent{Committed}` with the commit hash. The same
`Arc<Mutex<Option<CommittedToolSet>>>` is shared with the FSM enforcement layer so both read one source
of truth.

### 3.6 The φ/ψ scoring, for real (`reflect`, `:791`)

- `coverage_factor = 1 − 1/(1+count)` — diminishing returns, → 1.0 as citations grow.
- `phi = avg(citation.quality.score) × coverage_factor`.
- `psi = avg(evidence_level.score) × avg(quality) × coverage_factor` (corroboration × quality ×
  coverage; deliberately does NOT require tier diversity).

Separately, `compute_source_confidence(tiers)` (`:899`): if >80% of sources are T4Rejected it returns
a heavily-penalized sub-0.5 confidence; otherwise the tier-weighted mean (T1 1.0 / T2 0.8 / T3 0.5 /
T4 0.2).

### 3.7 Telemetry (how the dashboard sees it)

Two emitter granularities, both optional (no-op if unattached): `WebResearchStepEvent` (coarse
per-iteration `iterate`/`no_results`/`converged`/`pruned`/`max_iterations`, `:99`) and
`WebResearchStageEvent` (fine `formulate`/`search`/`extract`/`reflect`/`converge` ×
`started`/`completed`, `:150`). The CLI plugs in an SSB-backed emitter (`repl_command.rs:17`) so
`/research` surfaces `web_research_step` rows in the activity feed. `NodeLoopStats` (`:884`) is the
metrics summary.

---

## 4. The grounding transport (`grounding_client.rs`) — where an LLM is called

`GroundingClient` (`:120`) is a two-method trait (native async fn, no `async_trait`):
`ground(prompt, level) -> ResearchReport` and `synthesize_text(prompt) -> String`. `ResearchLevel`
(`:59`) selects one of four prompt templates baked in at compile time via `include_str!` (`:49-52`):
`l0_idea`, `l1_concept`, `l2_phase`, `l3_step` (rendered by `{prompt}` substitution, `:85`). Named
defaults: sidecar `http://localhost:8100`, path `/v1/completions`, 30s timeout, 2048 max tokens,
**temperature 0.0** (`:43-47`) — determinism first.

Three implementations:
- **`SidecarGroundingClient`** (`:141`) — local path. reqwest POST to the sidecar completions endpoint,
  parses first choice, strips code fences, `parse_research_report_json` (`:1338`). Typed errors
  (`GroundingError`, `:95`): SidecarUnavailable / Timeout / HttpStatus / InvalidJson / EmptyResponse /
  RequestFailed.
- **`CloudGroundingClient`** (`:422`) — Anthropic path. Production concerns: **daily USD spend cap**
  (`with_spend_cap_daily_usd`, `:520`, blocks not warns), **cumulative token tracker** / ledger
  (`send_with_ledger`, `:843`), strict-schema application marking (`:556`), **Batch API lane**
  (`ground_batch`, `:1099`, 50%-off non-interactive) gated by `batch_lane_enabled()` (`:1084`),
  task-id tagging, `from_env`/`from_env_async` constructors. E-35 blake3 prefix hashing (`:27`)
  fingerprints provider/schema for cache keys.
- **`SyntheticGroundingClient`** / **`CannedSynthesisGroundingClient`** (`:1235`, `:1275`) —
  deterministic test doubles returning a canned `synthetic_report(topic)` (`:1303`).

Also: `HttpCosmosResearchDispatcher` (`:1531`) implements `elai_knowledge::ResearchDispatcher` for the
Cosmos global-artifact refresh worker; `SsbEmitterAdapter` (`:1665`) bridges knowledge-refresh events
to the SSB.

`parse_research_report_json` (`:1338`) + `strip_code_fence` (`:1350`) are the shared
JSON→`ResearchReport` bridge; combined with the tolerant deserializers (§1.7) this is what survives
real LLM output.

---

## 5. Recursive decomposition — deep research at every tree level (`research_decompose`)

`research_decompose` (`research_tree.rs:2191`) is the driver that turns an idea into an executable tree
by running grounded research at each level: **Idea → Concepts → Phases → Steps → BabySteps**, each
level using the matching `ResearchLevel` template (§4) against a `GroundingClient`. `extract_concepts`/
`_phases`/`_steps` (`:2363/2386/2433`) parse the grounded output; `decompose_to_baby_steps` (`:2455`)
emits `BabyStepSpec`s. A `BabyStepSpec` (`:1885`) carries the full 8-block contract the executor later
walks: GROUNDING, PRE_GATE, TASK, TDD, DETERMINISTIC_CHECK, ACCEPTANCE, INTEGRATION, EXIT_CRITERIA
(`:1945-2004`). Caps: `MAX_DECOMPOSE_DEPTH = 2`, `MAX_BABY_STEP_TOKENS = 2048`,
`MAX_GROUNDING_TOKENS = 500` (`:186-192`).

Two anti-hallucination legs ride here:
- `verify_packages` / `verify_packages_with_emitter` (`:2565/2576`) — API-registry existence check on
  every proposed dependency (`is_plausible_package_name`, `:2631`), emits `PackageRefVerified` SSB rows
  — the anti-slopsquat gate at plan time.
- `TechStackDecision` (`:2006`) — chosen-vs-alternatives with a `DecisionSource`
  (research/constraint/default, `:2014`) emitted as `TechStackDecided` rows.

Per-node decomposition events are emitted through the injectable `DecompositionTreeEmitter` (`:68`) /
`SsbDecompositionEmitter` (`:2651`) so the dashboard's decomposition tree view fills in.

**`ResearchCache`** (`:2041`) is an LRU (256 entries) + TTL (3600s) memo keyed by question;
`get_or_research` (`:2100`) computes on miss — so re-decomposing the same idea is free within the window.

Production dispatch is real and sidecar-backed (`task_runner.rs:6882`, the D2 path; contract at
`crates/contracts/tests/research_decompose_real_prompts_contract.rs`).

---

## 6. The E-48 hardening substrate (`research/web_orchestrator/`)

All of E-48 is **additive and wire-dark by default**. The gate env var is
`ELAI_WEB_RESEARCH_ORCHESTRATOR_ENABLE` (`web_orchestrator/mod.rs:120`); `is_enabled()` (`:130`)
accepts `1/true/yes/on`. Producers check it and silently `Ok(None)` when unset, so the base B-7
behavior is unchanged for non-E-48 callers. All telemetry rides a single
`SsbActionType::WebResearchEvent` variant carrying a Kind discriminator (Pattern-2, no per-task
action-type cascade). SCHEMA_VERSION 184→185.

### 6.1 Editable plan (Task 1, `editable_plan.rs`)

Gemini-deep-research-style editable chain-of-thought produced BEFORE search fires. `WebResearchPlan`
(`:83`) holds parent question + ranked sub-questions + a `planned_search_budget` (default 16).
Sub-questions ranked deterministically by **expected information gain = 0.7·novelty + 0.3·specificity**
(`:131`): novelty = fraction of sub-question tokens NOT in the parent (`novelty_score`, `:187`);
specificity = capitalized-token / named-entity count saturating at 4 (`specificity_score`, `:202`).
Capped at 8 sub-questions (`MAX_SUB_QUESTIONS`), each truncated to 280 chars. `interactive`/
`approved_by_user` flags gate a `--interactive-research-plan` human approve/edit step
(flag wiring RUNPOD-PENDING).

### 6.2 Tiered-search telemetry (Task 2, `tier_pool_telemetry.rs`)

Thin adapter over `elai_web_search::tiered_search_pool` re-exporting the `BackendTier` rename and
writing `TieredSearchDispatched` rows when wired into `search_and_rank`. Grounds in arXiv 2509.23519
(ReliabilityRAG tier rationale).

### 6.3 Speculative-branch pruner (Task 5, `speculative_pruner.rs`)

FlashResearch three-arm `BranchType` discriminator (`:41`): `Question` (default, legacy B-7 shape),
`Breadth` (fan-out), `Depth` (drill-down), `Speculative` (probabilistic side-bets). Only `Speculative`
branches are prune candidates. `evaluate` (`:123`) prunes iff ALL hold: Speculative AND NOT
`prune_protected` (Task-7 contradiction veto) AND `evidence_delta < threshold(0.10)` (NaN→0.0) AND
`low_gain_streak > floor(2)`. The verdict is computed here but the actual drop is delegated to the
existing `ResearchTree::prune_speculative_children` (single source of truth). `speculative_budget`
(`:156`) allocates `min(remaining_budget·0.20, parent_value·0.50)` search calls to side-arms.

### 6.4 Conformal triple-threshold stop (Task 6, `conformal_stop.rs`)

Extends the B-7 dual threshold (φ_min upper + ψ_min floor) with a THIRD lower bound: **preemptive
unsolvable**. `evaluate` returns `PreemptiveUnsolvable` when all four hold (`:103`): rounds ≥
`min_rounds_before_preemptive(3)` AND confidence < `preemptive_unsolvable_lower(0.20)` AND
`low_delta_streak ≥ required(3)` AND the delta floor (0.05) was met each round. NaN confidence → 0.0.
Grounds in arXiv 2602.03814 (Conformal Thinking). **Calibration caveat, stated in the code** (`:17-26`):
ELAI has no tagged-unsolvable corpus yet, so the threshold is conservative and `threshold_uncalibrated:
true` rides the SSB row and a startup warning; rarely fires until 30+ tagged cases accrue.

### 6.5 Contradiction classifier (Task 7, `contradiction_classifier.rs`)

The never-silently-resolved contradiction surfacer that supersedes the legacy `find_contradictions`
stub. Sets `prune_protected: true` on the materialized `Contradiction` (the three E-48 `#[serde(default)]`
fields added to the struct: `prune_protected`, `surfaced_at_unix_ms`, `classification_reason`,
`research_tree.rs:449-471`), forcing the renderer to surface both sources verbatim and the speculative
pruner to refuse dropping the parent. Sidecar-backed LLM pair-judge; **live call is RUNPOD-PENDING** —
host-side scaffolding ships, the LLM leg lights up when the sidecar is available. Grounds in arXiv
2504.00180, 2511.06668 (healthcare RAG never-silently-resolve), 2510.03418 (LegalWiz).

### 6.6 Citation verifier + attribution lock (Tasks 8+9, `citation_verifier.rs`)

Four-stage cheapest-first verification (`:3`):
- (a) **URL-liveness** — HEAD, 5s timeout; 404/timeout/conn-error → `dead_url`, **dropped**.
- (b) **arXiv-ID** — regex `arxiv:\d{4}\.\d{4,5}` + HEAD `arxiv.org/abs/{id}`; 404 → `invalid_arxiv`, **dropped**.
- (c) **CrossRef DOI** — regex `10\.\d+/...` + HEAD `doi.org/{doi}`; 404 → `invalid_doi`, **dropped**.
- (d) **SemanticCite faithfulness** — sidecar pair-judge "does source Y SUPPORT claim X?";
  `Support/Partial/Contradict/Orthogonal` (`:67`); a mismatch is **flagged, NOT dropped**.

Stages a/b/c are deterministic HTTP HEAD + regex; stage d is sidecar-backed and **RUNPOD-PENDING** (a
`FaithfulnessJudge` trait abstracts the backend for a test stub). Two compile-time invariants:
`VerifiedCitation` is a newtype whose constructor is `pub(super)` so **no final-report code can hold an
unverified citation** — the type system enforces "no citation in the report without verification"; and
the `AttributionSet` (Task 9, the Anthropic lead-never-cites rule) blocks the orchestrator from
inventing a citation — only the SEARCH-stage producer may attach one, a violation emits
`citation_attribution_violation_blocked`. H-53 extends `VerificationOutcome` with `anchor_id`/
`pre_bound` for the closed-set citation-pre-binding path (§7).

### 6.7 Recency decay (Task 10, `recency_decay.rs`)

Temporal-marker regex + exponential decay multiplier layered on the existing `SourceQualityTable::score`.
Default half-life 365 days, configurable via `[web_search.recency]` in `elai_cc.toml`. Grounds in arXiv
2509.19376 (Solving Freshness in RAG).

### 6.8 Producers (`producers.rs`)

SSB `emit_*` free fns (one per fire-point): `emit_research_plan_emitted`, `emit_tiered_search_dispatched`,
`emit_extraction_failed_all_extractors`, `emit_cross_encoder_rerank_complete`,
`emit_speculative_branch_pruned`, `emit_research_preemptive_stop`,
`emit_contradiction_node_materialized`, `emit_citation_verification_outcome`,
`emit_citation_attribution_violation_blocked`, `emit_recency_decay_applied`. Each checks `is_enabled()`
and writes a `WebResearchEventPayload` row.

---

## 7. Citation integrity (cross-cutting)

Three modules enforce that no fabricated citation reaches the user:

- **`extractive_grounding.rs`** (157 lines) — J-41 `run_extractive_grounding(answer_text, evidence_spans)`;
  3-tier fingerprint (Tier1 exact 8-gram, Tier2 fuzzy 5-gram jaccard≥0.8, Tier3 keyword≥50%); flags
  `ZeroSpanClaim`s (sentences with no extractive support).
- **`citation_prebind.rs`** (551 lines) — H-53 closed-set citation pre-binding. `EvidenceAnchor`,
  `EvidenceAnchorSet`, `RankedEvidence`, `freeze_anchors(ranked, seed)`, `CitationCheckOutcome`,
  `check_citation_in_set(...)`, `extract_citation_ids(answer_text)`. A detected **ghost citation**
  triggers the D-16 refusal path with the answer suppressed (`repl_command.rs:458`).
- **`citation_verifier.rs`** — the `VerifiedCitation` newtype wall (§6.6).

---

## 8. Config, env vars, knobs (operator surface)

| Knob | Default | Effect |
|---|---|---|
| `ELAI_WEB_RESEARCH_ORCHESTRATOR_ENABLE=1` | off | Arm the E-48 hardening substrate (wire-dark). |
| `ELAI_RESEARCH_MODE` | Standard | `Fast`/`Standard`/`Deep` (J-40, §III.13). |
| `ELAI_FETCHER=obscura` | reqwest | Headless-browser fetch (requires `obscura` cargo feature; else warns+falls back). |
| `ELAI_BLAST_RADIUS_DISABLE` | armed | Disarm the G-3 egress gate (fail-closed when armed). |
| `ANTHROPIC_BASE_URL` | `http://localhost:8100` | Sidecar target; completions `/v1/completions`. |
| `elai_cc.toml` `[web_search.recency]` half-life | 365d | Recency decay. |
| `elai_cc.toml` `[research.vram_tiers.<tier>]` | per VRAM | `ResearchTierConfig`: phi_min/psi_min/depth_limit/breadth_factor/max_sources_per_node/max_concurrent_fetches. |
| CloudGroundingClient builders | — | Daily USD spend cap (blocks), Batch-lane enable, task-id tag. |

SearXNG must be up before the sidecar (per project CLAUDE.md remote-sidecar block); search falls back
to Brave per the search-architecture notes.

---

## 9. What is LIVE vs RUNPOD-PENDING

**LIVE today:** the full web loop (search→fetch→extract→index→reflect→converge), source tiering, φ/ψ
convergence, info-gain pruning, egress+taint+tool-commit gates, the editable-plan ranking, the
speculative-pruner predicate, the deterministic citation stages (a/b/c), recency decay,
`research_decompose` with package/tech-stack verification, and the H-53 citation pre-binding +
ghost-citation refusal on the `/research` answer path.

**RUNPOD-PENDING** (sidecar-gated; host scaffolding shipped, LLM leg dark until the sidecar is up): the
contradiction classifier's pair-judge (§6.5) and the citation verifier's stage-(d) faithfulness judge
(§6.6). Flag-pending: the `--interactive-research-plan` human approve/edit UI (§6.1).

**Deliberately conservative/uncalibrated:** the conformal preemptive-unsolvable lower threshold (§6.4) —
carries its own `threshold_uncalibrated` warning, rarely fires until a tagged-unsolvable corpus exists.

---

## PART II — Tier / Budget / Mode configuration

- **`research_tier.rs`** (227 lines) — `ResearchTierConfig` (model, quant, context_ceiling,
  research_context_budget, chunk_size, breadth_factor, depth_limit, phi_min, psi_min, embedding_device,
  max_sources_per_node, max_concurrent_fetches, speculative_depth); `research_config_for_tier(VramTier)`
  for 8/12/16/24/48GB + Cloud. Parsed from `[research.vram_tiers.<tier>]` in `elai_cc.toml`.
- **`research_budget.rs`** (368 lines) — `EvidenceItem`, `CompressedEvidence`, `ResearchBudgetManager`
  with `can_admit`, `admit`, `prune_to_budget` (drops lowest tier / lowest phi first; never drops T1),
  `compress_evidence` (first 3 sentences + last).
- **`research_mode.rs`** (52 lines) — `pub enum ResearchMode { Fast, Standard, Deep }` with `caps()`
  returning `ModeCaps { max_iterations, max_sources_per_node, breadth_factor }`. `Deep` = `depth × 15`
  iterations, `+5` sources, `+1` breadth. Selected via `ELAI_RESEARCH_MODE`. **This is the literal "deep
  research" knob.**

---

## PART III — Plan-letter research systems (the broader map)

The core engine (Part I) is the implementation of base-4D. The letter plans extend, harden, productize,
and occasionally FALSIFY research concepts. Each is summarized below with its plan file, status, and the
single most load-bearing excerpt.

### III.10. B-40 — Idea-Framer + Domain Research Presets + Critic Loop (SHIPPED)

**Plan:** `docs/thefinalglance/B-plan/B-40_idea_framer_research_presets.md`

Inserts a FRAMING substate before Phase 4D Recursive Decomposition that classifies any product idea,
dispatches a domain-specific research preset (multi-stream parallel FlashResearch), and runs a
10-category critic loop (1-3 rounds) that BLOCKs decomposition until the idea is framed, grounded, and
the user has answered the surfaced tradeoff card.

Three additions:
1. **Domain Research Presets** — TOML-declared named bundles of research streams per project class.
   Dispatched in parallel as a multi-root FlashResearch tree (`ResearchTree::new_multi_root`).
2. **Idea-Framer agent** — variant of Designer with restricted GRC budget (5/800), runs BEFORE Concept
   decomposition, produces `FramedIdea` + `TradeoffCard`.
3. **Critic Loop** — 10 critique categories (C-STALE, C-SLOP, C-POLICY, C-FEASIBILITY, C-MARKET,
   C-SCOPE, C-CONTRADICTION, C-DESIGN-HARD, C-VAGUE, C-MISSING) auditing the FramedIdea per round. Max
   3 rounds. BLOCK on any unresolved C-* category.

New types: `FramedIdea`, `TradeoffCard`, `CriticFinding`, `ProjectClass` (ConsumerMobileApp/WebApp/
SaaSDashboard/DesktopApp/GameMod/ResearchTool/LibraryOrSDK/CLITool/Other). New FSM substate `FRAMING`
under `PLANNING`.

### III.11. C-30 — Oracle Mode + `OracleAnswerCache` (the `/research` answer cache)

**Plan:** `docs/thefinalglance/C-plan/C-30_surgical_oracle_mode.md`

Oracle Mode answer-level cache with tier-based TTL (T1=30d, T2=7d, T3=1d) — distinct from
`ResearchCache` at `research_tree.rs:1805`, which covers decomposition reuse, not final `/research`
answer caching.

`OracleAnswerCache { inner: HashMap<CacheKey, CachedAnswer>, max_entries }`. `CacheKey =
(query_hash: [u8;32], project_hash: [u8;32])` where both hashes are blake3. `CachedAnswer { answer:
ResearchAnswer, cached_at: u64, source_tier: SourceTier }`. TTL by tier: T1Authoritative=2592000s (30d),
T2Strong=604800s (7d), T3Weak=86400s (1d), T4Rejected=0 (never cache).

Wired at the `/research` call site (`live_cli.rs:2203`): cache miss → run_research_pipeline → insert +
`oracle_cache_miss` SSB row; cache hit → return + `oracle_cache_hit` SSB row. Also home to the `oracle`
user skill (`/oracle`, `/ask`).

### III.12. E-48 — Web-research orchestrator hardening (see Part I §6)

**Plan:** `docs/thefinalglance/E-plan/E-48_web_research_orchestrator.md`

Pre-reqs: B-7, B-39, B-40 SHIPPED; C-48 PLANNED. The seven additive tasks (editable plan, tiered-search
telemetry, speculative-branch pruner, conformal triple-threshold stop, contradiction classifier,
citation verifier + attribution lock, recency decay) are all detailed in Part I §6.

### III.13. J-40 — Research modes (FAST/STANDARD/DEEP) + extractive compression + live SearXNG (PLANNED)

**Plan:** `docs/thefinalglance/J-plan/J-40_research_modes_extractive_compression.md`

Depends on J-38, E-48, base-1E. The thesis: *"the model is a cog, not a brain, in research: it should
only FORMULATE queries and RANK/SELECT pages; the harness fetches (in parallel), extractively compresses
each source to a small span set, and only distilled grounded spans + external per-step state ever enter
the reasoning context."*

Build/steal items:
1. **Query-aware EXTRACTIVE compressor for WEB sources** — verbatim spans preserve citation fidelity
   (never generative summarization). Per-source span budget so a 25-40K local model fits ~100-150 sources.
2. **Activate the E-48 live-search leg** — swap `StubSearchBackend` for a real `SearXngBackend`.
3. **Speed-tiered research modes** — FAST = one ranking pass; STANDARD = shallow tree (breadth 2-3,
   depth 3); DEEP = full tree (breadth 4-5, depth 5) + iterative refine.
4. **EXTERNAL per-step state between research iterations** — serialize hit lists + coverage map to the
   SESSION DAG / journal.

EXCLUDED: generative re-summarization; KV-cache reuse; full "Search-as-Code". Grounded in Perplexity
query-aware compression + arXiv:2501.14342 (CoRAG).

### III.14. O-Plan — The "Personal Research Machine" (PLANNED)

**Index:** `docs/thefinalglance/O-plan/O-0_INDEX.md` — Track `RESEARCH-MACHINE` (O-1..O-4). Source
synthesis: `workflow/reasoning/elai-personal-research-machine-2026-07-06.md`. Four layers:

- **O-1 research daemon** (KEYSTONE, `O-1_research_daemon_generalized_cosmos.md`) — generalize
  `CosmosWorker` (the already-shipped scheduled research loop) from a manifest hard-bound to designer
  briefs into a user-editable typed topic set: `[[research.topics]]: { id, kind = steal-scan |
  leaderboard | discovery | monitor, sources[], cadence, confidence_threshold, sink }` — a CLOSED enum of
  topic kinds, never free-form. Every fetched byte threads through ELAI's existing trust spine (taint +
  tool-filter + VCR).
- **O-2 scheduled TaskSpec substrate** (`O-2_scheduled_taskspec_substrate.md`) — `TaskSpec { trigger:
  Cron | SsbEvent | Manual; pipeline: ordered CLOSED ENUM of stage kinds (Fetch, Classify, Gate,
  FileToBrain, Digest, SpawnResearchTree, RunPlanUnit, Notify); sink }`. "n8n with receipts."
- **O-3 brain vault contract mounting** (`O-3_brain_vault_contract_mounting.md`) — mount (don't convert)
  a teikoku `brain_store` under a namespace claim so two harnesses share one vault.
- **O-4 canvas workflow mode** (`O-4_canvas_workflow_mode.md`) — typed stage nodes, dataflow edges
  carrying provenance tier, scoped to AI-research tasks. "n8n with receipts" on the canvas.

### III.15. Cosmos Artifact Tier (B-42 / C-16 — SHIPPED)

**Plans:** `docs/thefinalglance/B-plan/B-42_cosmos_artifact_tier.md`,
`docs/thefinalglance/C-plan/C-16_cosmos_pipeline_integration.md`. Feature-map: "COSMOS ARTIFACT TIER
[WIRED]".

`CosmosWorker` is the scheduled research loop that O-1 generalizes: global `~/.elai/cosmos/`, 8th
grounding method (`CosmosCheck`), ~12 default artifacts, refreshed on a cadence by a FlashResearch
worker (the `HttpCosmosResearchDispatcher` in `grounding_client.rs:1531`), time-decayed to `[DECAYED]`,
≤1500-token briefing, `CosmosStage` in the `QueryPipeline`.

### III.16. H-12 — Canvas PLAN/RESEARCH modes (PLANNED)

**Plan:** `docs/thefinalglance/H-plan/H-12_plan_research_modes_chrome.md`

Ports `OrbitMode1` (plan) and `OrbitMode3` (research) to flat clipping-frame modes on the same canvas.
The active mode is a PLAIN flag on `canvas.svelte.ts` (`mode in {code,plan,research,arch,design}`), NOT
an orthogonal FSM substate. **RESEARCH mode** (~140 LOC) renders `ResearchTreeNode` + children with
`TendrilEvent` sources as leaf nodes, `cites` edges, and red contradiction edges (the
`contradiction_with` field) between disagreeing sources. The single-consumer SSB handover rule: only the
active window holds the live SSB subscription; the inactive one is PAUSED.

### III.17. Q-10 — Research-conditioned creativity (REFUTED)

**Plan:** `docs/thefinalglance/Q-plan/Q-10_research_conditioned_creativity.md`. Track
`RESEARCH-CONDITIONED-CREATIVITY`. **STATUS: B-CREATIVITY-0 REFUTED (valid, 2026-07-14, run 3 / b0v3)**.

The premise: ELAI has grounded recursive research, verified citations, contradiction-bearing reports,
Cosmos artifacts. What it lacks is a proved interface that turns research into **new target mechanisms**
without allowing source citations to masquerade as target evidence. Q-10 added one materially new
intervention: verified source research compressed into a typed relational analogy packet
(`ResearchInspirationPacket`), used in one generation call, then discarded as authority; target grounding
and execution stay separate.

Key invariant: `evidence_scope=SOURCE_ONLY` — source citations prove only the source mechanism, never
the target side. Five-arm ladder: `DIRECT`, `MATCHED_SCAFFOLD`, `VERIFIED_EVIDENCE`, `RELATIONAL_PACKET`,
`SHUFFLED_PACKET`. **Result:** the relational-packet arm underperformed EVERY control on P1 recovery (all
deltas negative, e.g. −0.046 vs DIRECT/MATCHED_SCAFFOLD) and tied on P2 execution while costing ~8.8×
DIRECT. B-CREATIVITY-1 NOT authorized; no skill/Rust/runtime consumer; no paper-to-plan/idea-intake
adoption. The DESIGN (one-way packet) is not refuted, but this specific coupling does not pay on the
synthetic falsifier.

### III.18. Q-11 / Q-12 / Q-13 — AIDE² Research-Instrument Economics (PLANNED)

Track `AIDE2-RESEARCH-INSTRUMENT-ECONOMICS`. Source: the AIDE² deep-research steal (paper-to-plan
PHASE 5, 2026-07-15).

- **Q-11** (`Q-11_novelty_instrument_and_fixed_budget_bench.md`, ships FIRST) — instrument integrity
  BEFORE search improvement. *"before making ELAI's research search smarter, the instrument that MEASURES
  research novelty must be proven live and the bench must be spend-honest — the AIDE² dossier's own
  statistical anti-gaming layer was silently live-broken and had no impact, hidden by aggregate scores."*
  C6 live-path novelty canary (fail-closes verdict emission on discrimination-dead classifier); C10
  fixed-dollar-at-locked-cap reporting + two named do-not-win control arms (`unbounded_best_of_n`,
  `parallel_widening`) with a `spend_leak` flag; BASELINE frozen pre-change capture.
- **Q-12** (`Q-12_research_lane_context_and_search_policy.md`, gated on Q-11) — two units. UNIT 1: C3
  per-operator minimal-prompt matrix (each research operator FORMULATE/SEARCH/EXTRACT/REFLECT/CONVERGE
  gets a MINIMAL prompt: newest-12 one-line finding-summaries + exactly-ONE full-extract). UNIT 2: C1+C2
  lineage-bandit search policy — each draft-lineage is a bandit ARM; when the best lineage PLATEAUS,
  fork a restart in STRATEGY space, not code-quality space. Ships SELECTABLE behind default-OFF
  `ELAI_RESEARCH_BANDIT_ENABLE` with a mandatory DETERMINISTIC-SCHEDULER control arm and a KILL
  criterion.
- **Q-13** (`Q-13_receipt_bound_briefing.md`, sibling) — every compiled briefing line carries a receipt
  (a fetched-byte-hash of its source or a journal-row-id) resolved AT COMPILE TIME; a line whose receipt
  does not resolve is DROPPED and COUNTED, never silently emitted. *"A fabricated briefing line becomes
  structurally unrepresentable."*

### III.19. Q-15 — Research-bench & measurement-protocol hygiene (PLANNED)

**Plan:** `docs/thefinalglance/Q-plan/Q-15_research_bench_and_protocol_hygiene.md`. Track
`AUTORESEARCH-SYNERGY`. STATUS: PLANNED, S-1 REJECTED-by-measurement 2026-07-17.

Five sub-units:
- **S-1** pareto_per_task specialist retention (THIRD policy in Q-12's selector seam; ports Evo's GEPA
  `_pick_pareto_per_task`, arXiv:2507.19457) — CLOSED, must NOT be built (B-SYN-2 refuted it at the
  preregistered effect size).
- **S-2** refutation power floor — a REFUTED verdict must record its power; reclassify as
  NULL-UNDERPOWERED if the CI could not detect the claimed effect.
- **S-3** `verified: bool = False` provenance bit on the bench/fixture/canary catalog.
- **S-4** canonical pass@k/std_err variance helper (replaces hand-rolled bootstrap paths).
- **S-6** handoff mismatch-logging (doctrine rider).
- DEFERRED-CONDITIONAL: **S-5** campaign-grain typed stop conditions — blocked on a real multi-session
  RESEARCH-CAMPAIGN consumer.

### III.20. I-29 — Diffusion draft lane / TTD-DR (Test-Time Diffusion Deep Researcher) — narrowed, gated

**Plan (canonical, narrowed):** `docs/thefinalglance/I-plan/I-29_diffusion_draft_lane.md`.
**Pre-fab original concept:** `docs/prefableresearch/ELAIplan_Consolidated-I-Plan_Integrity-Inference-Search.txt`
(I-29, lines ~335–341). Track 16 ROUTING+BUDGETS+MEASUREMENT, WAVE 5. Gated (new-dep + network, user
approval required).

The narrowed canonical I-29 explicitly **DECLINES** the full TTD-DR research loop:
*"TTD-DR's full denoising-research loop is DECLINED (out of scope; the trust story requires the full
veto stack to be in place first) ... the dLLM is an UNTRUSTED generator — it has no special status. NO
TTD-DR full denoising-research loop — DECLINED; not a stub, must not appear as a placeholder or deferred
TODO."*

The steal is **speed only** (one-pass draft generation via Mercury-class dLLM + CRDT merge for parallel
patches), passing the identical I-17 tri-layer veto stack. The CRDT merge skeleton (`draft_merge.rs`) is
the only Tier-1 ship; the dLLM client is Tier-2 gated. The pre-fab original described the broader
concept: model long-form reporting as a continuous denoising diffusion process over a Markdown draft
stored in the `SessionDag`, bypassing autoregressive linear degradation; CRDTs enable dozens of
sub-agents to asynchronously patch the research document; Semantic Checkpoints require the draft to
compile/parse via C-32's MiniCritic at each denoising step.

### III.21. D-29 — Hierarchical Bayesian BAI for non-IID research rewards (DEFERRED, research-grade; partly shipped)

**Plan:** `docs/thefinalglance/D-plan/D-29_hierarchical_bai_research.md`.

*"Objective: published BAI (D-5; Audibert COLT 2010) assumes IID rewards. ELAI pipeline outcomes are
NOT IID across sessions (different tasks, different priors). Hierarchical Bayesian BAI may close this
gap. Closure includes 'infeasibility note' as valid outcome. Estimated effort: 4-8 sessions wall-clock +
RunPod for Gibbs sampler."*

Two closure paths: Path A (hierarchical regret ≤ flat-BAI → productize, gated by
`ELAI_BAI_MODE=hierarchical`); Path B (worse → infeasibility note). NOTE: the WIRING_CONTRACT shows
Path A actually shipped (Rust host-side dispatch at `task_runner.rs:11406` wired and live; the Python
Gibbs sampler benchmark is RUNPOD-PENDING). Despite "DEFERRED" in the title, this is partly shipped.

### III.22. PART 23 RESEARCH AGENT — the canonical original design

**File:** `docs/.research/elai_cc_design_document.txt` (Part 23, line ~1872). Mirrored in
`docs/specs/integration/pipeline-factory.txt` (section 4). Implemented as base-4D.

*"PART 23: RESEARCH AGENT — Purpose: structured SOTA discovery for novel task types. Equivalent to
'Gemini Deep Research' but deterministic — output is structured data, not prose. Fires during L1
Pipeline Designer BEFORE plan assembly. Makes the system domain-agnostic for novel task types. Role:
researcher. Model: cloud API preferred (broad knowledge + web access) or Gemma4 local with web search
tool. GRC budget: high (10/1200) — deep exploration needed. Trigger: L1 encounters a capability
requirement with no template match. Permissions: NETWORK (web search), FILESYSTEM (read-only)."*

Evidence levels: BENCHMARK > OFFICIAL_DOCS > BLOG_POST > TRAINING. Output: structured Research Report
(not prose).

---

## PART IV — Backing infrastructure

### IV.23. The `web_search` crate

**Dir:** `.elai_cc/crates/web_search/src/`

- `lib.rs` — `SearchResult`, `SearchError`, `WebSearchError`, `strip_html`, `parse_domain`,
  constants (`DEFAULT_SEARCH_TIMEOUT_SECS`, `DEFAULT_MAX_RESULTS=20`).
- `client.rs` — `SearxClient` (SearXNG client).
- `fetcher.rs` — `FetchedPage`, `WebFetcher`.
- `extractor.rs` — `PageType`, `ExtractedContent`, `ContentExtractor`, `QueryAwareExtractor` (J-40 T1),
  `BasicExtractor`, `QueryAwareBasicExtractor`.
- `source_quality.rs` — `SourceQualityTable` with `score_url` / `lookup_url`.
- `rate_limiter.rs`, `robots.rs` — politeness.
- `tiered_search_pool.rs` (E-48) — `BackendTier`, `TieredSearchPool`, `TierQuota`,
  `canonical_url_hash`, `dedupe_by_canonical_url`.
- `cross_encoder_rerank.rs` (E-48) — `RerankerBackend`, `RerankInput`, `RerankOutput`,
  `CrossEncoderRerankConfig`, `resolve_backend`, `passthrough_rerank` (Cohere/Jina RUNPOD-PENDING).
- `extractor_pool.rs` (E-48) — multi-extractor pool (crawl4ai/Jina RUNPOD-PENDING).
- `obscura_fetcher.rs` — headless-browser fetcher (feature = "obscura").

Host-side adapter: `.elai_cc/crates/orchestrator/src/searx_search_backend.rs` — `SearxSearchBackend`
implementing `research_pipeline::SearchBackend`.

### IV.24. Raw-evidence tier (`memory` crate)

**Dir:** `.elai_cc/crates/memory/src/raw_evidence/`

Header: *"J-21 Step 3 — provenance-linked raw-evidence tier + sufficiency-gated escalation. ... keeps
an immutable raw-evidence layer beneath the extracted-fact layer and escalates to it only when the fact
layer is insufficient."*

- `store` — `RawEvidenceStore`, `RawEvidenceRecord`, `MAX_RECORDS_PER_NAMESPACE` (append-only, FIFO
  compaction).
- `sufficiency` — `assess_sufficiency(authorized, excluded)`,
  `SufficiencyVerdict::{Sufficient, Insufficient(SufficiencyReason)}`,
  `SufficiencyReason::{AllExcluded, NoCandidates}`.
- `escalation` — `escalate_to_raw_evidence(store, ns, excluded_facts, sufficiency)`,
  `RawEvidenceEscalation`.
- `get_authorized_facts_with_escalation(...)` (mod.rs `:95`) — composed Stage1 authorized retrieval →
  Stage2 sufficiency gate → Stage3 raw-evidence escalation. Grounded in GEM arXiv:2605.26252 + MPBench
  arXiv:2606.04329.

### IV.25. Agentic retrieval loop (knowledge-backed)

- `.elai_cc/crates/orchestrator/src/retrieval_loop.rs` (74 lines) — J-38 Unit 3.
  `RetrievalLoopInput { intent_symbols, context_symbols, pipeline, dkg, current_round }`;
  `run_retrieval_loop(input) -> RetrievalLoopOutput` — calls `compute_context_sufficiency`; if
  insufficient and rounds remain, calls `pipeline.hybrid_search` + `expand_with_graph_neighbors`
  against the DKG. Backed by `elai_knowledge::{arag::RetrievalResult, degradation::SearchMode,
  pipeline::QueryPipeline, dkg::DkgEngine, hybrid_search_tool}`.
- `context_sufficiency.rs` — `compute_context_sufficiency`, `SufficiencyReport`.

---

## PART V — User-facing entry points & end-to-end flow

### V.26. Slash commands + skills

- **`/research`** and **`/ask`** slash commands — registered at `crates/cli/src/commands.rs:1088`,
  parsed to `SlashCommand::Research { question }` (`commands.rs:1700`). `/ask` is an alias on the same
  answer path.
- The natural-language **classifier** routes bare questions here: `HAS_QUESTION && !HAS_CODE_INTENT &&
  !HAS_PROJECT_REF → /research` (`crates/cli/src/classifier.rs:79`).
- **`oracle` skill** (`.agents/skills/oracle/SKILL.md`) — *"Interactive grounded Q&A. Triggers when
  user types /oracle [question], /ask [question], or asks any deep factual question requiring web
  grounding."*
- **`paper-to-plan` skill** (`.agents/skills/paper-to-plan/SKILL.md`) — *"End-to-end paper/repo steal
  pipeline. Triggers when the user drops one or more paper links (arXiv/DOI/PDF) or repo links
  (GitHub/GitLab) and asks whether/how they can improve ELAI."*

### V.27. End-to-end flow

1. User types `/research <question>` → parsed at `crates/cli/src/commands.rs:1700` → `SlashCommand::Research`
   (also via `FrontDoorLane::Research` in `classifier.rs`).
2. `crates/cli/src/repl_command.rs:291-381` builds a `ResearchQuery`, loads `SourceTierConfig`, calls
   `select_research_search_backend` (SearxNG → Stub fallback), constructs a `GroundingClient`
   (`SidecarGroundingClient`/`CloudGroundingClient`), and calls
   `elai_orchestrator::run_research_pipeline(...)`.
3. `run_research_pipeline` runs phases A→G (System 1). Phase F calls `grounding.synthesize_text()`.
   Phase G post-verifies. Result is a `ResearchAnswer` (possibly cached in `OracleAnswerCache`).
4. For deep/iterative research, `WebResearchOrchestrator::run_node_loop` (System 2c) drives
   `ResearchNode`s through FORMULATE→SEARCH→FETCH+EXTRACT→INDEX→RETRIEVE→REFLECT→CONVERGE, using the
   `web_search` crate and `ResearchBudgetManager`. `ResearchTree::run` walks the tree; `research_decompose`
   does the idea→concept→phase→step recursion via `GroundingClient`.
5. The E-48 substrate layers on: editable plan, tiered pool telemetry, speculative pruning, conformal
   stop, contradiction classification, 4-stage citation verification, recency decay — all gated by
   `ELAI_WEB_RESEARCH_ORCHESTRATOR_ENABLE` (wire-dark by default).
6. Citation integrity is enforced by `extractive_grounding`, `citation_prebind`, and the
   `citation_verifier::VerifiedCitation` newtype (compile-time: no unverified citation reaches the final
   report).
7. Memory-side raw-evidence and the agentic retrieval loop feed context into the orchestrator's
   retrieval path.

---

## PART VI — Paper grounding (verified arXiv IDs, from module headers)

| arXiv ID | Title concept | Used by |
|---|---|---|
| 2510.05145 | FlashResearch / ParallelResearch (the tree + speculative branches + info-gain pruning) | research_tree.rs |
| 2510.12164 | Parallel Reasoning survey | research_tree.rs |
| 2508.12752 / 2506.18096 / 2508.05668 | Deep Research surveys | pipeline-factory.txt |
| 2602.03814 | Conformal Thinking (triple-threshold stop) | conformal_stop.rs |
| 2504.00180 | Contradiction Detection in RAG | contradiction_classifier.rs |
| 2511.06668 | Healthcare-RAG never-silently-resolve | contradiction_classifier.rs |
| 2510.03418 | LegalWiz | contradiction_classifier.rs |
| 2602.06718 | GhostCite (citation hallucination) | citation_verifier.rs |
| 2604.03173 | CITEVERIFIER / SemanticCite / CiteGuard | citation_verifier.rs |
| 2601.18724 | HalluCitation Matters | citation_verifier.rs |
| 2509.19376 | Solving Freshness in RAG (recency decay) | recency_decay.rs |
| 2509.23519 | ReliabilityRAG (tiered search) | tier_pool_telemetry.rs |
| 2602.03442 | A-RAG (progressive disclosure) | web_research_orchestrator.rs |
| 2501.14342 | CoRAG (query-aware compression) | J-40 |
| 2602.05930 | Compound Deception | web_orchestrator/mod.rs |
| 2605.26252 | GEM (raw-evidence escalation) | memory/raw_evidence |
| 2606.04329 | MPBench | memory/raw_evidence |
| 2507.19457 | Evo GEPA (pareto_per_task) | Q-15 S-1 (REFUTED) |

Plus simonwillison.net/2025/Jun/14 (Anthropic multi-agent: editable plan + lead-never-cites attribution
lock) and the Gemini Deep Research editable-CoT overview. Full list at
`web_orchestrator/mod.rs:62-77`; decision log at `workflow/reasoning/E-48-decisions.md`.

---

## PART VII — Cross-cutting notes & gotchas

- **Keyword search negative results.** The compound terms "research-mode", "research loop", "research
  agent", "research phase", "inquiry", "investigation", "literature", and "evidence gathering" return
  **zero** hits as literal phrases in the docs tree. The actual terms are "research mode",
  "RESEARCH AGENT", "research daemon", "research preset", `ResearchTree`, `research_decompose`.
  "DeepResearch" as one PascalCase word also returns zero hits — the codebase writes it as two words
  ("deep research") or as `FlashResearch` / `WebResearchOrchestrator`.
- **"Grounding" means two distinct things.** (a) The `GroundingClient` LLM transport (Part I §4); and
  (b) the general "evidence grounding" doctrine across plans. The `_grounding/` subfolders in K/L/H/Q/
  J/I-plan hold per-plan source/reliability ledgers, not a separate research system.
- **Two distinct "research trees".** `ResearchTree` (the deterministic FlashResearch skeleton) vs
  `WebResearchOrchestrator` (the real web loop). Read Part I §0 first or the architecture looks
  contradictory.
- **Two distinct caches.** `ResearchCache` (`research_tree.rs:1805`) covers decomposition reuse;
  `OracleAnswerCache` (`oracle_cache.rs`) covers final `/research` answer caching. Don't conflate.
- **The `python/` directory** (`elai_inference`, `elai_optimizer`, `elai_finetune`) contains no
  research-related symbols — it's the model-serving side, not the research pipeline.
- **Contract tests pinning research behavior** live under `.elai_cc/crates/contracts/tests/`:
  `a22_researching_fsm_contract.rs`, `a47_research_cli_contract.rs`, `e48_research_contracts.rs`,
  `e48_web_orchestrator_on_path.rs`, `research_citation_regex_contract.rs`,
  `research_decompose_live_sidecar_contract.rs`, `research_decompose_cache_hit.rs`,
  `research_decompose_constrained.rs`, `research_path_chosen_contract.rs`,
  `web_research_orchestrator_contract.rs`, `j40_research_mode_contract.rs`,
  `j41_extractive_grounding_contract.rs`, `j41_binary_grounding_judge_contract.rs`,
  `h53_citation_prebind_contract.rs`, `grounding_cache_event_contract.rs`,
  `searx_search_backend_contract.rs`.

---

## PART VIII — Determinism, wiring, and honest limits (from the code itself)

- **Determinism.** Temp 0.0 everywhere; complexity/quality/pruning are pure functions of text + counts;
  `ClockProvider` injected (no raw `std::time`); the bare tree loop is a reproducible sigmoid. This is
  what lets a `/research` run be replayed.
- **Wiring / SSB.** The whole loop is journaled — `web_research_step` stage/step rows,
  `ToolFilterEvent{Committed}`, `blast_radius_blocked`, `PackageRefVerified`, `TechStackDecided`,
  `WebResearchEvent` (E-48), and the `research` answer rows on the CLI path. The dashboard reads these;
  no metric is self-reported by the generating agent.
- **Honest failure modes named in the code.** LLM JSON drift (mitigated by three tolerant
  deserializers, §1.7); phishing look-alike domains (defeated by exact-domain match, §1.4); citation
  hallucination (defeated by the 4-stage verifier + the `VerifiedCitation` type wall, §6.6);
  untrusted-content injection (taint + egress-pin, §3.2/3.4); dependency hallucination (registry
  existence check in `verify_packages`, §5).

---

## Source-of-truth files (where to dig deeper)

| File | What it is |
|---|---|
| `docs/thefinalglance/ELAI-DEEP-RESEARCH-EXPLAINED.md` | Ground-truth explainer of the core engine (read from live code). |
| `docs/thefinalglance/ELAI-SPEC-OVERVIEW.md` | Top-level design spec, all phases + letter plans. |
| `docs/specs/integration/pipeline-factory.txt` | Canonical spec of the tree-structured research architecture (section 5) + RESEARCH AGENT (section 4). |
| `docs/specs/guide/steps/phase-4D.txt` | Phase 4D original plan incl. "WEB RESEARCH INFRASTRUCTURE EXTENSION (Deep Research)". |
| `docs/.research/elai_cc_design_document.txt` | Original design doc, PART 23 RESEARCH AGENT. |
| `docs/thefinalglance/E-plan/E-48_web_research_orchestrator.md` | E-48 plan (the hardening substrate). |
| `docs/thefinalglance/B-plan/B-40_idea_framer_research_presets.md` | B-40 idea-framer. |
| `docs/thefinalglance/C-plan/C-30_surgical_oracle_mode.md` | C-30 Oracle cache. |
| `docs/thefinalglance/J-plan/J-40_research_modes_extractive_compression.md` | J-40 modes. |
| `docs/thefinalglance/O-plan/O-0_INDEX.md` | Personal Research Machine index. |
| `docs/thefinalglance/Q-plan/Q-10..Q-15` | Q research-conditioned creativity (REFUTED) + AIDE² economics. |
| `.elai_cc/crates/orchestrator/src/research_tree.rs` | FlashResearch tree + research_decompose. |
| `.elai_cc/crates/orchestrator/src/research_pipeline.rs` | 7-phase Oracle pipeline. |
| `.elai_cc/crates/orchestrator/src/web_research_orchestrator.rs` | Per-node web loop. |
| `.elai_cc/crates/orchestrator/src/grounding_client.rs` | GroundingClient transport. |
| `.elai_cc/crates/orchestrator/src/research/web_orchestrator/` | E-48 substrate (10 modules). |
| `workflow/reasoning/E-48-decisions.md` | E-48 decision log. |

*Compiled 2026-08-02 from spec + plan summaries + live code. Ground-truth anchors are `file:line` into
`.elai_cc/crates/orchestrator/src/`. If a leg here says "LIVE" but a run shows it dark, check the E-48
env gate (§8) and the sidecar health (the two RUNPOD-PENDING legs need it up).*
