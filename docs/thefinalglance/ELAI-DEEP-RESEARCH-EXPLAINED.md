# ELAI — Deep Research, Explained (every detail)

Purpose: a single ground-truth explainer of how ELAI's "deep research" actually works, traced from the
user entrypoint down to the HTTP fetch and back up to the report. Everything here is read out of the
live code, not the plan prose — every claim carries a `file:line` anchor you can open. Where the code
itself says a leg is deferred / wire-dark / sidecar-pending, this doc says so in the same breath.

Scope note: "deep research" in ELAI is TWO cooperating systems, not one, plus a grounding transport and
a recursive-decomposition driver that reuses the same machinery. Read section 1 first or the rest will
look contradictory (the tree "has no LLM" yet research "calls the sidecar" — both true, different layers).

---

## 0. The four pieces (map before territory)

| Piece | File | Role |
|---|---|---|
| FlashResearch tree | `crates/orchestrator/src/research_tree.rs` (3108 lines) | The tree scaffold: complexity → breadth/depth, φ/ψ dual-threshold convergence, info-gain pruning, source-quality tiers, `ResearchReport` shape. Its `run()` loop is a **deterministic simulation**; real signals are injected by a grounding client. |
| Web research orchestrator | `crates/orchestrator/src/web_research_orchestrator.rs` (1052 lines) | The **real** per-node loop that actually hits the web: FORMULATE → SEARCH → QUALITY GATE → FETCH+EXTRACT → INDEX → RETRIEVE → REFLECT → CONVERGE. Lives in the orchestrator process, calls deps directly (no IPC). |
| Grounding transport | `crates/orchestrator/src/grounding_client.rs` (1674 lines) | The `GroundingClient` trait + three impls (Sidecar/local, Cloud/Anthropic, Synthetic/test) that turn a prompt + `ResearchLevel` into a parsed `ResearchReport`. This is where an LLM is actually called. |
| E-48 hardening substrate | `crates/orchestrator/src/research/web_orchestrator/` (10 modules) | Additive, wire-dark-by-default hardening: editable plan, tiered-search telemetry, speculative-branch pruner, conformal triple-threshold stop, contradiction classifier, 4-stage citation verifier, recency decay. |

Two more that reuse the above: `research_decompose` (recursive Idea→Concepts→Phases→Steps grounded
decomposition, `research_tree.rs:2191`) and the `ResearchCache` (LRU+TTL memo, `research_tree.rs:2041`).

---

## 1. What "deep research" means here, and the key subtlety

There are two distinct "research trees" in the codebase and they are NOT the same object:

- **`ResearchTree`** (`research_tree.rs:1145`) is the FlashResearch scaffold from arXiv 2510.05145.
  Its `run()` method (`research_tree.rs:1381`) walks nodes and advances φ/ψ, but the progression is a
  **deterministic sigmoid on iteration count**, not evidence — see `process_node_iteration`
  (`research_tree.rs:1440-1480`): `progress = 1 / (1 + e^(-(iterations-2)))`, then
  `phi = progress·(phi_min+0.15)`. The header says it plainly (`research_tree.rs:1378`): *"no LLM calls —
  real inference happens via IPC GroundingRequest proxy (out of 4D scope). This method processes the tree
  structure deterministically based on node properties and thresholds."* So the standalone `ResearchTree`
  is the **shape + convergence math + report format**; on its own it produces a structurally-valid but
  content-empty report. It is used as a testable, reproducible skeleton and by the multi-root / split-task
  paths.

- **`WebResearchOrchestrator`** (`web_research_orchestrator.rs:165`) is the thing that actually does
  research. It takes a `ResearchNode` (the same node type from `research_tree.rs:545`) and drives it
  through a real web loop, computing φ/ψ from **actual fetched citations** in `reflect()`
  (`web_research_orchestrator.rs:791`). This is the production path for `/research`.

So: the tree defines the data model and the convergence contract; the orchestrator fills a node with
real evidence against that contract; the grounding client is the LLM transport underneath both. Read
that way, nothing conflicts.

---

## 2. User entrypoint — `/research` and `/ask`

- The REPL slash command `research` is registered at `crates/cli/src/commands.rs:1088` and parses to
  `SlashCommand::Research { question }` (`commands.rs:1700`). `/ask` is an alias on the same answer path.
- The natural-language **classifier** routes bare questions here without a slash: rule 3 in
  `crates/cli/src/classifier.rs:79` — `HAS_QUESTION && !HAS_CODE_INTENT && !HAS_PROJECT_REF → /research`.
- Handling lives in `crates/cli/src/repl_command.rs`. It routes to task mode and constructs the runner:
  `repl_command.rs:573` ("routed to task mode (D2: DefaultWebResearchRunner)") →
  `repl_command.rs:744` `DefaultWebResearchRunner::new(...)`. Usage error on empty input:
  `repl_command.rs:534` `"Usage: /research <question>"`.
- On the answer path, H-53 **citation pre-binding + constrained synthesis** runs (`repl_command.rs:417`):
  citations are bound into a closed `EvidenceAnchorSet` before synthesis, and a detected **ghost
  citation** triggers the D-16 refusal path with the answer suppressed (`repl_command.rs:458`), emitting
  an SSB `research` row. There is also a disk-backed answer cache so idempotent `/research|/ask` repeats
  are cheap (`repl_command.rs:299`, the C-30 Oracle cache lazily created on first `/research`,
  `live_cli.rs:579`).

Distinction worth remembering (`bench_rag.rs:22`, `main.rs:343`): **`/research` is WEB research, not
retrieval.** RAG over the local index is a different command; `/research` goes to the internet.

---

## 3. The FlashResearch tree (`research_tree.rs`) — data model + convergence math

### 3.1 Complexity → budget (the only "tuning" knob that matters)

`assess_complexity(question)` (`research_tree.rs:1730`) is deterministic, no LLM: it looks at word count
and comparison markers (` vs `, ` versus `, ` compare `) to pick `QueryComplexity::{Simple,Moderate,Complex}`.
Complexity then fans out to every budget constant (`research_tree.rs:230-254`):

| Complexity | breadth_factor | depth_limit | phi_min |
|---|---|---|---|
| Simple   | 2 (`BREADTH_SIMPLE`)   | 3 (`DEPTH_SIMPLE`)   | 0.80 (`PHI_MIN_SIMPLE`) |
| Moderate | 3 (`BREADTH_MODERATE`) | 4 (`DEPTH_MODERATE`) | 0.70 (`PHI_MIN_MODERATE`) |
| Complex  | 4 (`BREADTH_COMPLEX`)  | 5 (`DEPTH_COMPLEX`)  | 0.60 (`PHI_MIN_COMPLEX`) |

`psi_min` is a flat 0.70 (`PSI_MIN_DEFAULT`, `research_tree.rs:157`) across all complexities. Note the
**harder the question, the LOWER the φ bar** — the design accepts weaker per-node certainty on complex
questions because they fan wider. Hard safety rails cap the tree regardless of complexity:
`MAX_TREE_DEPTH = 10`, `MAX_TOTAL_NODES = 200`, `MAX_ITERATIONS_DEFAULT = 50`
(`research_tree.rs:166-179`), enforced in `add_child` (`research_tree.rs:1649`, which errors
`MaxDepthExceeded`/`MaxNodesExceeded`).

### 3.2 The node and its two convergence scalars

A `ResearchNode` (`research_tree.rs:545`) carries `phi`, `psi`, `status`, `info_gain`, `iterations`,
`low_gain_streak`, `citations`, `answer`, `depth`. The two scalars are the whole convergence story:

- **φ (phi)** = *goal satisfaction / coverage*. In production it is average source-quality × a coverage
  factor (see 4.6).
- **ψ (psi)** = *quality / corroboration floor*. Average evidence-level × quality × coverage.

Convergence is an **AND, never an OR** (`ResearchNode::is_converged`, `research_tree.rs:604`):
`phi >= phi_min && psi >= psi_min`. A node with great coverage but single-sourced evidence does not
converge, and vice-versa. `PartialEq` on the node is hand-written with a NaN guard so a poisoned φ/ψ can
never compare equal (`research_tree.rs:562`).

### 3.3 Info-gain pruning (the "stop digging a dry hole" rule)

Each iteration computes `info_gain = |Δphi|`. If it is below `INFO_GAIN_THRESHOLD = 0.1`
(`research_tree.rs:160`) the node's `low_gain_streak` increments; otherwise it resets to 0. Once the
streak hits `LOW_GAIN_PRUNE_COUNT = 3` (`research_tree.rs:163`), `should_prune()` returns true
(`research_tree.rs:609`) and the node is marked `Pruned`. This is the info-gain pruning FlashResearch
prescribes: a branch that stops learning is abandoned, freeing budget for productive branches.

### 3.4 Source quality: 5 deterministic tiers, exact-domain match

`assign_source_quality(url)` (`research_tree.rs:323`) is deterministic and **anti-phishing**: it extracts
the host (`extract_host`, strips port, IPv6-safe) and matches against exact domain allow-lists
(`domain_matches`, `research_tree.rs:368` — host equals the domain OR ends with `.domain`, so
`arxiv.phishing.com` does NOT match Tier1). Tiers and scores (`research_tree.rs:169-173, 287-319`):

- Tier1 = 1.0 — arxiv.org, doi.org, ietf.org, w3.org, ieee.org, acm.org
- Tier2 = 0.8 — docs.rs, crates.io, pypi.org, doc.rust-lang.org, docs.python.org, learn.microsoft.com, developer.mozilla.org
- Tier3 = 0.6 — stackoverflow.com, github.com, gitlab.com
- Tier4 = 0.4 — medium.com, dev.to, hashnode.dev
- Tier5 = 0.2 — everything else (unverified)

Orthogonally, `EvidenceLevel` (`research_tree.rs:382`) ranks corroboration:
Anecdotal 0.25 / SingleSource 0.5 / MultiSource 0.75 / Verified 1.0. A single fetched page starts
`SingleSource`; once ≥2 corroborate, all citations on the node are upgraded to `MultiSource` (see 4.5).

### 3.5 The deterministic `run()` loop (skeleton path)

`ResearchTree::run()` (`research_tree.rs:1381`): start the clock, activate root, add `sub_questions` as
children, activate all children, then loop up to `max_iterations` — each iteration checks the time budget
(`is_time_exceeded`, `research_tree.rs:1705`), collects active nodes, and calls
`process_node_iteration` on each. It exits when no nodes are active or the budget/iteration cap is hit,
then `build_report()`. Again: on the bare tree, φ/ψ come from the sigmoid, not the web (3.1). This path
is what the 48+ unit tests exercise and what the multi-root/split-task features build on.

Multi-root (`new_multi_root`, `research_tree.rs:1221`, B-40): several research "streams" each become a
root node processed by the same loop; `stream_root_id`/`multi_root_ids` look them up. Used by
`dispatch_multi_root_research`. `split_step` (`research_tree.rs:1329`, C-9) realizes an autonomous-gate
`SplitTaskPlan` against the tree (depth-capped, emits `SplitTaskFailed` on empty).

### 3.6 The output: `ResearchReport`

`build_report` (`research_tree.rs:1483`) assembles the `ResearchReport` (`research_tree.rs:1085`):
`question`, `answer`, `alternatives`, `contradictions`, `sub_results` (one per child node with confidence
+ evidence level), a `snapshot` of every node, `quality_metrics` (a `convergence_ratio` metric threshold
0.5, and a `time_budget_ratio` metric threshold 1.1), and an overall `ConvergenceStatus`.

`ConvergenceStatus` (`research_tree.rs:862`) is `Converged` (all nodes) / `Partial{converged,total}` /
`Failed{reason}` / **`PreemptiveUnsolvable{...}`** (the E-48 triple-threshold arm, section 7.4).
`compute_convergence_status` (`research_tree.rs:1551`) picks between the first three by counting statuses.

Contradictions: `find_contradictions` (`research_tree.rs:1576`) is the LEGACY stub — it emits at most one
sample contradiction from the first two converged nodes for the report shape, is NOT prune-protected, and
is superseded by the E-48 `contradiction_classifier` (section 7.5). It only runs when
`plan.require_contradictions` is set.

### 3.7 Robustness: tolerant deserialization (because LLMs emit messy JSON)

Because the tree is reconstructed from LLM-generated JSON, three tolerant deserializers keep a single bad
element from nuking a whole report:
- `lenient_vec` (`research_tree.rs:488`) — for `alternatives`/`contradictions`/`sub_results`/`quality_metrics`;
  parses to `Vec<Value>` first, keeps only elements that round-trip, drops the rest with a `warn!`. Origin:
  a real cloud-Sonnet run (94dcc5b) where one bare-string entry aborted a 5KB `ResearchReport`.
- `deserialize_nodes_tolerant` (`research_tree.rs:909`) — per-node skip on missing `id`/`question`/`depth`
  (Qwen-3B often diverges from the schema), tolerates `nodes: null`.
- `deserialize_convergence_status` (`research_tree.rs:725`) — accepts unit-strings (`"Partial"`) or the
  struct form, lossy-fills counts with a `warn!`, aliases legacy `"Diverged"` → `Partial{0,0}`.

---

## 4. The real per-node web loop (`web_research_orchestrator.rs`)

This is where `/research` actually spends its time. The orchestrator is constructed from six infra crates
(`web_research_orchestrator.rs:1-8, 194`): `SearxClient` (search), `WebFetcher` (HTTP), `BasicExtractor`
(HTML→text), `SourceQualityTable` (tiering), `WebContentIndex` (A-RAG index), `ResearchBudgetManager`
(token budget), plus a `ResearchTierConfig`. Production wiring: `DefaultWebResearchRunner::build_orchestrator`
(`task_runner.rs:2193`) builds all six and `run()` (`task_runner.rs:2280`) creates a single root
`ResearchNode(1)` and calls `run_node_loop`.

### 4.1 The loop stages (`run_node_loop`, `web_research_orchestrator.rs:278`)

Max iterations = `tier_config.depth_limit × MAX_ITERATIONS_PER_DEPTH(10)` (`:284`). Per iteration:

1. **FORMULATE** (`:344` → `formulate_query`, `:639`) — first pass: the raw question. Later passes:
   append `-site:{domain}` exclusions for up to 3 already-visited domains so search **diversifies** away
   from sources already read. (Header notes production could call the sidecar to formulate; the shipped
   path is this deterministic domain-exclusion heuristic.)
2. **SEARCH** (`:354` → `search_and_rank`, `:670`) — `SearxClient.search(query)`, then results sorted by
   source-quality score DESC, ties broken by the engine's own relevance score. A per-tier histogram
   (t1..t4) rides the `search_completed` SSB stage row (`:372`).
3. **QUALITY GATE** (`:404`) — partition into high-quality (T1–T3) vs low-quality (T4); fetch high-quality
   always, append low-quality **only if budget remains** (`budget_manager.remaining() > 0`), then truncate
   to `tier_config.max_sources_per_node`.
4. **FETCH + EXTRACT** (`:429` → `fetch_and_extract`, `:757`) — batched by `max_concurrent_fetches`. Each
   URL passes the **egress gate first** (4.2), then reqwest or obscura (4.3), then `BasicExtractor`; an
   `Empty` page is an error and skipped. Every fetched body is **taint-registered** as untrusted web
   content (4.4).
5. **INDEX** (`:463`) — for each extract, estimate tokens (`EvidenceItem::estimate_tokens`); if the budget
   `can_admit`, ingest a `WebChunk` into the `WebContentIndex`, `admit` the tokens, push a `SourceCitation`
   onto the node (tier→quality via `source_tier_to_quality`, `:934`; evidence starts `SingleSource`,
   excerpt truncated to 200 chars). Budget-exceeding chunks are skipped (`:465`).
6. **corroboration upgrade** (`:493`) — once the node has ≥`MULTI_SOURCE_THRESHOLD(2)` citations, every
   `SingleSource` citation is promoted to `MultiSource`.
7. **RETRIEVE** (`:503`) — `content_index.keyword_search(question, breadth_factor)`, the A-RAG progressive
   disclosure step over what's been indexed so far.
8. **REFLECT** (`:518` → `reflect`, `:791`) — recompute φ/ψ from the citations (4.6). `info_gain = φ−φ_old`;
   `low_gain_streak` increments below `INFO_GAIN_THRESHOLD`, resets otherwise.
9. **CONVERGE** (`:530`) — converge iff `is_converged(phi_min, psi_min) && citations.len() >=
   MIN_SOURCES_FOR_CONVERGENCE(2)`. Else if `should_prune()` → prune. Else iterate again. On convergence
   the node is `Converged`; on max-iterations without either, it stays `Active` and emits a
   `max_iterations` step (`:606`).

Empty search results (`:377`) bump `low_gain_streak` and can prune the node even with zero fetches — a
question the web can't answer dies on the streak rule, not on a crash.

### 4.2 Egress blast-radius gate (G-3, fail-closed)

`enforce_egress` (`:712`) runs BEFORE any connect inside `fetch_and_extract`. It rejects a NUL/control-char
host pre-resolution, then confirms **every** resolved IP is inside the pinned allow-set (`pinned_egress_ips`,
`:695` — the union of every resolved `ResearcherNetConfig.allowed_hosts` IP plus trusted resolvers). A
`BlastRadiusVerdict::Block` refuses the fetch and emits a `blast_radius_blocked{EgressDenied}` SSB row (the
off-agent artifact). Kill-switch `ELAI_BLAST_RADIUS_DISABLE` disarms it; default-ON. A host that fails to
resolve contributes no IPs, so it can never match — fail-closed by construction.

### 4.3 Fetcher backends

Default is reqwest (`WebFetcher`, tagged `"reqwest"`). With the `obscura` cargo feature AND
`ELAI_FETCHER=obscura`, a headless-browser `ObscuraFetcher` is used instead (tagged `"obscura"`) for
JS-heavy pages (`task_runner.rs:2227`, `web_research_orchestrator.rs:764`). Feature-off + env-on logs a
warning and falls back to reqwest (`task_runner.rs:2244`). The fetcher tag rides every step event.

### 4.4 Taint threading (G-1)

Each fetched page body is `tainted_context().register(&text, Provenance::from_web(url))`
(`:443`) — untrusted web bytes go into a process-local taint side-table so a later destructive sink whose
argv derives from that body is blocked by the capability gate. Default-ON (no env gate on the register;
the kill-switch is at the sink). Trusted flows are a no-op (the side-table skips Trusted labels).

### 4.5 Tool-set commit (F-3)

Once per `run_node_loop`, before the first untrusted-byte boundary, `commit_minimal_set(task_id,
[Web, Read], now)` locks the researcher to a minimal tool set (`:301`) and (if an SSB writer is attached)
emits `ToolFilterEvent{Committed}` with the commit hash. The same `Arc<Mutex<Option<CommittedToolSet>>>`
is shared with the FSM enforcement layer so both read one source of truth.

### 4.6 The φ/ψ scoring, for real (`reflect`, `:791`)

- `coverage_factor = 1 − 1/(1+count)` — diminishing returns, → 1.0 as citations grow.
- `phi = avg(citation.quality.score) × coverage_factor`.
- `psi = avg(evidence_level.score) × avg(quality) × coverage_factor` (corroboration × quality × coverage;
  deliberately does NOT require tier diversity).

Separately, `compute_source_confidence(tiers)` (`:899`) is a public helper: if >80% of sources are
T4Rejected it returns a heavily-penalized sub-0.5 confidence; otherwise the tier-weighted mean
(T1 1.0 / T2 0.8 / T3 0.5 / T4 0.2).

### 4.7 Telemetry (how the dashboard sees it)

Two emitter granularities, both optional (no-op if unattached): `WebResearchStepEvent` (coarse per-iteration
`iterate`/`no_results`/`converged`/`pruned`/`max_iterations`, `:99`) and `WebResearchStageEvent` (fine
`formulate`/`search`/`extract`/`reflect`/`converge` × `started`/`completed`, `:150`). The CLI plugs in an
SSB-backed emitter (`repl_command.rs:17`) so `/research` surfaces `web_research_step` rows in the activity
feed. `NodeLoopStats` (`:884`) is the summary for metrics.

---

## 5. The grounding transport (`grounding_client.rs`) — where an LLM is actually called

`GroundingClient` (`:120`) is a two-method trait (native async fn, no `async_trait`): `ground(prompt,
level) -> ResearchReport` and `synthesize_text(prompt) -> String`. `ResearchLevel` (`:59`) selects one of
four prompt templates baked in at compile time via `include_str!` (`:49-52`): `l0_idea`, `l1_concept`,
`l2_phase`, `l3_step` (rendered by `{prompt}` substitution, `:85`). Named defaults: sidecar
`http://localhost:8100`, path `/v1/completions`, 30s timeout, 2048 max tokens, **temperature 0.0**
(`:43-47`) — determinism first.

Three implementations:

- **`SidecarGroundingClient`** (`:141`) — the local path. reqwest POST to the sidecar completions endpoint,
  parses the first choice, strips code fences, `parse_research_report_json` (`:1338`). Typed errors
  (`GroundingError`, `:95`): SidecarUnavailable / Timeout / HttpStatus / InvalidJson / EmptyResponse /
  RequestFailed.
- **`CloudGroundingClient`** (`:422`) — the Anthropic path. Adds real production concerns: a **daily USD
  spend cap** (`with_spend_cap_daily_usd`, `:520`) that blocks (not warns), a **cumulative token tracker**
  / ledger (`send_with_ledger`, `:843`), strict-schema application marking (`:556`), a **Batch API lane**
  (`ground_batch`, `:1099`, 50%-off non-interactive) gated by `batch_lane_enabled()` (`:1084`) emitting
  batch-lifecycle SSB rows, task-id tagging, and `from_env`/`from_env_async` constructors. E-35 blake3
  prefix hashing (`:27`) fingerprints provider/schema for cache keys.
- **`SyntheticGroundingClient`** / **`CannedSynthesisGroundingClient`** (`:1235`, `:1275`) — deterministic
  test doubles returning a canned `synthetic_report(topic)` (`:1303`).

`parse_research_report_json` (`:1338`) + `strip_code_fence` (`:1350`) are the shared JSON→`ResearchReport`
bridge; combined with the tolerant deserializers (3.7) this is what survives real LLM output. There is also
an `HttpCosmosResearchDispatcher` (`:1449`) implementing `ResearchDispatcher` for the Cosmos global-artifact
refresh worker, and an `SsbEmitterAdapter` (`:1583`) bridging knowledge-refresh events to the SSB.

---

## 6. Recursive decomposition — deep research at every tree level (`research_decompose`)

`research_decompose` (`research_tree.rs:2191`) is the driver that turns an idea into an executable tree by
running grounded research at each level: **Idea → Concepts → Phases → Steps → BabySteps**, each level using
the matching `ResearchLevel` template (section 5) against a `GroundingClient`. `extract_concepts`/`_phases`/
`_steps` (`:2363/2386/2433`) parse the grounded output; `decompose_to_baby_steps` (`:2455`) emits
`BabyStepSpec`s. A `BabyStepSpec` (`:1885`) carries the full 8-block contract the executor later walks:
GROUNDING, PRE_GATE, TASK, TDD, DETERMINISTIC_CHECK, ACCEPTANCE, INTEGRATION, EXIT_CRITERIA
(`:1945-2004`). `MAX_DECOMPOSE_DEPTH = 2`, `MAX_BABY_STEP_TOKENS = 2048`, `MAX_GROUNDING_TOKENS = 500`
(`:186-192`).

Two anti-hallucination legs ride here: `verify_packages` / `verify_packages_with_emitter` (`:2565/2576`)
run an API-registry existence check on every proposed dependency (`is_plausible_package_name`, `:2631`) and
emit `PackageRefVerified` SSB rows — the anti-slopsquat gate at plan time. `TechStackDecision`
(`:2006`) records chosen-vs-alternatives with a `DecisionSource` (research/constraint/default, `:2014`)
emitted as `TechStackDecided` rows. Per-node decomposition events are emitted through the injectable
`DecompositionTreeEmitter` (`:68`) / `SsbDecompositionEmitter` (`:2651`) so the dashboard's decomposition
tree view fills in.

`ResearchCache` (`:2041`) is an LRU (256 entries) + TTL (3600s) memo keyed by question, `get_or_research`
(`:2100`) computing on miss — so re-decomposing the same idea is free within the window.

Production dispatch of `research_decompose` is real and sidecar-backed (`task_runner.rs:6882`, the D2 path;
contract at `crates/contracts/tests/research_decompose_real_prompts_contract.rs`).

---

## 7. The E-48 hardening substrate (`research/web_orchestrator/`)

All of E-48 is **additive and wire-dark by default**. The gate env var is
`ELAI_WEB_RESEARCH_ORCHESTRATOR_ENABLE` (`web_orchestrator/mod.rs:120`); `is_enabled()` (`:130`) accepts
`1/true/yes/on`. Producers check it and silently `Ok(None)` when unset, so the base B-7 behavior is
unchanged for non-E-48 callers. A contract test (`:152`) strict-equals this const against the SSB-side
const to prevent silent drift. All telemetry rides a single `SsbActionType::WebResearchEvent` variant
carrying a Kind discriminator (Pattern-2, no per-task action-type cascade). SCHEMA_VERSION 184→185.

### 7.1 Editable plan (Task 1, `editable_plan.rs`)

A Gemini-deep-research-style editable chain-of-thought produced BEFORE search fires. `WebResearchPlan`
(`:83`) holds parent question + ranked sub-questions + a `planned_search_budget` (default 16). Sub-questions
are ranked deterministically by **expected information gain = 0.7·novelty + 0.3·specificity** (`:131`):
novelty = fraction of sub-question tokens NOT in the parent (`novelty_score`, `:187`), specificity =
capitalized-token / named-entity count saturating at 4 (`specificity_score`, `:202`). Capped at 8
sub-questions (`MAX_SUB_QUESTIONS`), each truncated to 280 chars. `interactive`/`approved_by_user` flags
gate a `--interactive-research-plan` human approve/edit step (flag wiring RUNPOD-PENDING).

### 7.2 Tiered-search telemetry (Task 2, `tier_pool_telemetry.rs`)

Thin adapter over `elai_web_search::tiered_search_pool` re-exporting the `BackendTier` rename and writing
`TieredSearchDispatched` rows when wired into `search_and_rank`. Grounds in arXiv 2509.23519
(ReliabilityRAG tier rationale).

### 7.3 Speculative-branch pruner (Task 5, `speculative_pruner.rs`)

The FlashResearch three-arm `BranchType` discriminator (`:41`): `Question` (default, legacy B-7 shape),
`Breadth` (fan-out), `Depth` (drill-down), `Speculative` (probabilistic side-bets). Only `Speculative`
branches are prune candidates. `evaluate` (`:123`) prunes iff ALL hold: branch is Speculative AND NOT
`prune_protected` (the Task-7 contradiction veto) AND `evidence_delta < threshold(0.10)` (NaN→0.0) AND
`low_gain_streak > floor(2)`. The verdict is computed here but the actual drop is delegated to the existing
`ResearchTree::prune_speculative_children` (single source of truth). `speculative_budget` (`:156`) allocates
`min(remaining_budget·0.20, parent_value·0.50)` search calls to side-arms.

### 7.4 Conformal triple-threshold stop (Task 6, `conformal_stop.rs`)

Extends the B-7 dual threshold (φ_min upper + ψ_min floor) with a THIRD lower bound: **preemptive
unsolvable**. `evaluate` returns `PreemptiveUnsolvable` when all four hold (`:103`): rounds ≥
`min_rounds_before_preemptive(3)` AND confidence < `preemptive_unsolvable_lower(0.20)` AND `low_delta_streak
≥ required(3)` AND the delta floor (0.05) was met each round. NaN confidence → 0.0. Grounds in arXiv
2602.03814 (Conformal Thinking). **Calibration caveat, stated in the code** (`:17-26`): ELAI has no
tagged-unsolvable corpus yet, so the threshold is conservative and `threshold_uncalibrated: true` rides the
SSB row and a startup warning; it rarely fires until 30+ tagged cases accrue.

### 7.5 Contradiction classifier (Task 7, `contradiction_classifier.rs`)

The never-silently-resolved contradiction surfacer that supersedes the legacy `find_contradictions` stub.
It sets `prune_protected: true` on the materialized `Contradiction` (the three E-48 `#[serde(default)]`
fields added to the struct: `prune_protected`, `surfaced_at_unix_ms`, `classification_reason`,
`research_tree.rs:449-471`), forcing the renderer to surface both sources verbatim and the speculative
pruner to refuse dropping the parent. Sidecar-backed LLM pair-judge; **live call is RUNPOD-PENDING** —
host-side scaffolding ships, the LLM leg lights up when the sidecar is available. Grounds in arXiv 2504.00180,
2511.06668 (healthcare RAG never-silently-resolve), 2510.03418 (LegalWiz).

### 7.6 Citation verifier + attribution lock (Tasks 8+9, `citation_verifier.rs`)

Four-stage cheapest-first verification (`:3`):
- (a) **URL-liveness** — HEAD, 5s timeout; 404/timeout/conn-error → `dead_url`, **dropped**.
- (b) **arXiv-ID** — regex `arxiv:\d{4}\.\d{4,5}` + HEAD `arxiv.org/abs/{id}`; 404 → `invalid_arxiv`, **dropped**.
- (c) **CrossRef DOI** — regex `10\.\d+/...` + HEAD `doi.org/{doi}`; 404 → `invalid_doi`, **dropped**.
- (d) **SemanticCite faithfulness** — sidecar pair-judge "does source Y SUPPORT claim X?"; `Support/Partial/
  Contradict/Orthogonal` (`:67`); a mismatch is **flagged, NOT dropped**.

Stages a/b/c are deterministic HTTP HEAD + regex; stage d is sidecar-backed and **RUNPOD-PENDING**
(a `FaithfulnessJudge` trait abstracts the backend for a test stub). Two compile-time invariants:
`VerifiedCitation` is a newtype whose constructor is `pub(super)` so **no final-report code can hold an
unverified citation** — the type system enforces "no citation in the report without verification"; and the
`AttributionSet` (Task 9, the Anthropic lead-never-cites rule) blocks the orchestrator from inventing a
citation — only the SEARCH-stage producer may attach one, and a violation emits
`citation_attribution_violation_blocked`. H-53 extends `VerificationOutcome` with `anchor_id`/`pre_bound`
for the closed-set citation-pre-binding path (section 2). Grounds in GhostCite (2602.06718),
CITEVERIFIER/SemanticCite (2604.03173), HalluCitation Matters (2601.18724).

### 7.7 Recency decay (Task 10, `recency_decay.rs`)

A temporal-marker regex + exponential decay multiplier layered on the existing
`SourceQualityTable::score`. Default half-life 365 days, configurable via `[web_search.recency]` in
`elai_cc.toml`. Grounds in arXiv 2509.19376 (Solving Freshness in RAG).

---

## 8. Config, env vars, and knobs (the operator surface)

- `ELAI_WEB_RESEARCH_ORCHESTRATOR_ENABLE=1` — arm the E-48 hardening substrate (default off / wire-dark).
- `ELAI_FETCHER=obscura` — headless-browser fetch (requires the `obscura` cargo feature; else warns+falls back).
- `ELAI_BLAST_RADIUS_DISABLE` — disarm the G-3 egress gate (default armed / fail-closed).
- Sidecar target: `ANTHROPIC_BASE_URL` (defaults `http://localhost:8100`), completions `/v1/completions`,
  30s timeout, 2048 max tokens, temp 0.0.
- Cloud: daily USD spend cap (blocks), Batch-lane enable, task-id tag — via `CloudGroundingClient` builders.
- `elai_cc.toml`: `[web_search.recency]` half-life; source-tier config (`parse_source_tier_config`,
  `task_runner.rs:2208`) drives the `SourceQualityTable`; `ResearchTierConfig` sets phi_min/psi_min/
  depth_limit/breadth_factor/max_sources_per_node/max_concurrent_fetches.
- SearXNG must be up before the sidecar (per project CLAUDE.md remote-sidecar block); search falls back to
  Brave per the search-architecture notes.

---

## 9. Determinism, wiring, and honest limits

- **Determinism**: temp 0.0 everywhere; complexity/quality/pruning are pure functions of text + counts;
  `ClockProvider` injected (no raw `std::time`); the bare tree loop is a reproducible sigmoid. This is what
  lets a `/research` run be replayed.
- **Wiring / SSB**: the whole loop is journaled — `web_research_step` stage/step rows,
  `ToolFilterEvent{Committed}`, `blast_radius_blocked`, `PackageRefVerified`, `TechStackDecided`,
  `WebResearchEvent` (E-48), and the `research` answer rows on the CLI path. The dashboard reads these; no
  metric is self-reported by the generating agent.
- **What is LIVE today**: the full web loop (search→fetch→extract→index→reflect→converge), source tiering,
  φ/ψ convergence, info-gain pruning, egress+taint+tool-commit gates, the editable-plan ranking, the
  speculative-pruner predicate, the deterministic citation stages (a/b/c), recency decay, `research_decompose`
  with package/tech-stack verification, and the H-53 citation pre-binding + ghost-citation refusal on the
  `/research` answer path.
- **What is RUNPOD-PENDING (sidecar-gated, host scaffolding shipped, LLM leg dark until the sidecar is up)**:
  the contradiction classifier's pair-judge (7.5) and the citation verifier's stage-(d) faithfulness
  judge (7.6). Also flag-pending: the `--interactive-research-plan` human approve/edit UI (7.1).
- **What is deliberately conservative / uncalibrated**: the conformal preemptive-unsolvable lower threshold
  (7.4) — it carries its own `threshold_uncalibrated` warning and rarely fires until a tagged-unsolvable
  corpus exists.
- **Honest failure modes named in the code**: LLM JSON drift (mitigated by three tolerant deserializers,
  3.7), phishing look-alike domains (defeated by exact-domain match, 3.4), citation hallucination (defeated
  by the 4-stage verifier + the `VerifiedCitation` type wall, 7.6), untrusted-content injection (taint +
  egress-pin, 4.2/4.4), and dependency hallucination (registry existence check in `verify_packages`, 6).

---

## 10. Paper grounding (verified arXiv IDs, from the module headers)

FlashResearch / ParallelResearch 2510.05145 (the tree + speculative branches + info-gain pruning);
Parallel Reasoning survey 2510.12164; Conformal Thinking 2602.03814 (triple-threshold stop);
Contradiction Detection in RAG 2504.00180, Healthcare-RAG never-silently-resolve 2511.06668, LegalWiz
2510.03418 (contradiction classifier); GhostCite 2602.06718, CITEVERIFIER/SemanticCite/CiteGuard
2604.03173, HalluCitation Matters 2601.18724 (citation verifier); Solving Freshness in RAG 2509.19376
(recency decay); ReliabilityRAG 2509.23519 (tiered search); Compound Deception 2602.05930; Cohere
reranking best-practices (the cross-encoder rerank in `elai_web_search`); simonwillison.net/2025/Jun/14
Anthropic multi-agent (editable plan + lead-never-cites attribution lock). Full list at
`web_orchestrator/mod.rs:62-77`; decision log at `workflow/reasoning/E-48-decisions.md`.

---

Ground-truth anchors are `file:line` into `.elai_cc/crates/orchestrator/src/`. If a leg here says
"LIVE" but a run shows it dark, check the E-48 env gate (section 8) and the sidecar health (the two
RUNPOD-PENDING legs need it up). This doc is a snapshot of the code as read on 2026-07-05.
