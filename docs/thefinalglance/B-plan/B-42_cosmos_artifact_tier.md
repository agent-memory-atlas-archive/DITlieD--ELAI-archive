# B-42: Cosmos Tier on B-35 (cross-project knowledge cache + anti-staleness scheduler)
Objective: extend B-35's per-project Context Artifacts pipeline with a
global "cosmos" tier that stores cross-project knowledge — design trends
2026, iOS/Android App Store guidelines, web-app UX patterns, copyright /
trade-dress boundaries, color-palette references, common SaaS billing
patterns, etc — refreshed on a schedule via Layer 1A (B-23) so the
Designer (and idea-framer from B-40 + per-level auditor from B-41)
ground every product-class decomposition in current research without
paying for redundant research per task. One PictureThis competitive
teardown becomes reusable next session for "plant ID app", "tree ID
app", "garden plant care app", etc.
Estimated effort: 2 sessions.
Pre-requisites: B-35 (Context Artifacts Pipeline — cosmos extends the
  same indexing primitive), B-23 (Phase 4H Layer 1A — refresh cadence
  uses the same growth/decay machinery as deprecated_patterns), B-31
  (lock primitives — cosmos refresh must coexist with code indexing).
  Recommended sibling: B-40 (project-class detection determines which
  cosmos artifacts inject into Designer briefing) and B-41 (mode=concepts
  auditor uses cosmos as ground-truth comparison surface). SCHEMA_VERSION
  bumped by +1 (current is 28 baseline; could be higher if B-40/B-41
  shipped first).
Parallel-safe with: B-24, B-25, B-27, B-28, B-37, B-38.

## Spec references

- docs/specs/guide/steps/phase-4H.txt 4H Layer 1A — automatic data-table
  growth (no LLM, no staged eval) with FPR validation. Cosmos refresh
  reuses this exact mechanism: scheduled web-research → blake3-hashed
  artifact write → SHADOW status → ACTIVE on FPR threshold.
- docs/specs/guide/steps/phase-4D.txt 4D research-tree convergence.
  Cosmos refresh queries reuse FlashResearch but converge on a
  STORAGE artifact rather than a per-task answer.
- docs/specs/guide/steps/phase-1E.txt — A-RAG hierarchical retrieval +
  CatRAG graph-aware traversal already operate at per-project scope;
  cosmos extends with a SECOND collection at user-global scope.
- B-35 Context Artifacts Pipeline (DETAILED prereq).
- arXiv:2305.18290 (LLM staleness in code generation, 2023) — confirms
  training-data cutoff effects on API/library knowledge. Cosmos
  refresh schedule (nightly for fast-moving categories, weekly for
  slow) calibrated against expected knowledge half-life per domain.

## Context

Today: B-35 ships per-project [knowledge.artifacts] — DB schemas,
OpenAPI specs, ADRs declared in elai_cc.toml. Designer's Recursive
Decomposition consults them during ExistingProject decomposition.
Scope is bounded to the current project directory.

Gap: when the user says "make a plant-ID app like PictureThis", the
useful knowledge isn't *in* the (empty) project — it's the **research
ELAI conducted last week for a similar idea** plus **the iOS App Store
guidelines as of last refresh**. Today that research evaporates after
the task ends. The next time the user asks anything app-shaped, ELAI
re-researches from scratch ($35-55, 30-90 min per the user's image
estimate).

B-42 introduces a **cosmos** tier:
- Storage location: `~/.elai/cosmos/` (cross-project, user-scoped — NOT
  in any project repo, NOT gitignored at project level because it lives
  outside).
- Format identical to B-35: blake3-hashed artifact files (Markdown +
  TOML metadata) with separate vector collection (`cosmos_global` per
  ELAI instance).
- Indexing: same B-32 chunker, same hybrid (vector + BM25 RRF) search
  contract as B-35.
- Population: scheduled via Layer 1A worker. Each cosmos artifact has
  a `refresh_cadence` (daily / weekly / monthly / quarterly) + a
  `research_query` template. Layer 1A ticks fire FlashResearch jobs
  whose convergence writes to the cosmos collection.
- Consumption: Designer + idea-framer (B-40) + per-level auditor (B-41)
  pull AVAILABLE_COSMOS into their briefing prefix. Filtering by
  detected ProjectClass (B-40) + tier-relevance.
- Grounding: `GroundingMethod::CosmosCheck` (8th method, after B-35's
  ArtifactCheck) — verifies claims against cosmos artifacts. PipelineGrc
  dispatches to cosmos for project-class-relevant claim categories
  (design, IP, monetization, app-store, marketing).

Anti-staleness:
- blake3 content-hash detection on every refresh (no rewrite if
  unchanged — saves disk + dashboard noise).
- Time-decay confidence: artifacts older than 2× their refresh_cadence
  drop to SHADOW status; consumers warned + research stream re-prioritized.
- Refresh worker honors B-31 lock primitives — concurrent code
  indexing + cosmos refresh coexist cleanly.

## Reads (mandatory before coding)

- F:\.ELAI_workflow\docs\thefinalglance\B-plan\B-35_context_artifacts_pipeline.md (full file — cosmos extends this)
- F:\.ELAI_workflow\docs\thefinalglance\B-plan\B-23_phase_4H_self_modification.md (Layer 1A pattern)
- F:\.ELAI_workflow\docs\thefinalglance\B-plan\B-31_resumable_indexing_lock_lifecycle.md (lock primitives — cosmos worker reuses)
- .elai_cc/crates/knowledge/src/pipeline.rs (QueryPipeline — extend with CosmosStage between ArtifactStage and ARagStage)
- .elai_cc/crates/reasoning/src/grounding/verify.rs (PipelineGrc::verify_claim — add CosmosCheck dispatch)
- .elai_cc/crates/orchestrator/src/research_tree.rs (FlashResearch — cosmos worker dispatches via this)
- .elai_cc/crates/web_search/src/source_quality.rs (T1-T4 — cosmos artifacts must source from T1/T2 only by default)

## Tasks

### Phase 1 — Cosmos storage + manifest + worker (Session 1)

1. Cosmos directory layout:
   ```
   ~/.elai/cosmos/
     manifest.toml                        # cosmos catalog
     artifacts/
       design_trends_2026.md
       design_trends_2026.toml            # metadata: hash, refreshed_at, research_query
       ios_app_store_guidelines.md
       ios_app_store_guidelines.toml
       web_app_ux_patterns_2026.md
       web_app_ux_patterns_2026.toml
       copyright_tradedress_guide.md
       ...
     index/
       cosmos_global.bm25                 # tantivy index
       cosmos_global.vec                  # vector store collection
   ```

2. Manifest format (`~/.elai/cosmos/manifest.toml`):
   ```toml
   schema_version = 1

   [[cosmos.artifacts]]
   name = "design_trends_2026"
   project_classes = ["ConsumerMobileApp", "WebApp", "SaaSDashboard"]
   refresh_cadence = "weekly"
   min_source_tier = "T1"
   research_query = "current mobile/web app design trends {year} typography color motion"
   max_size_bytes = 32_768
   tier_relevance = { concepts = 0.9, phases = 0.5, steps = 0.2, babysteps = 0.0 }

   [[cosmos.artifacts]]
   name = "ios_app_store_guidelines"
   project_classes = ["ConsumerMobileApp"]
   refresh_cadence = "monthly"
   min_source_tier = "T1"  # apple.com only
   research_query = "Apple App Store Review Guidelines latest changes {year}"
   max_size_bytes = 65_536
   tier_relevance = { concepts = 0.7, phases = 0.4, babysteps = 0.6, plan = 1.0 }

   # ... ≥ 12 default artifacts shipped
   ```

3. Cosmos worker (`crates/knowledge/src/cosmos_worker.rs` CREATE):
   - `CosmosWorker::tick()` runs on Layer 1A schedule (default: hourly,
     skips artifacts not yet due for refresh).
   - Per due artifact: build FlashResearch query from `research_query`
     + current year + min_source_tier filter; await convergence;
     write Markdown body + blake3 hash; update `.toml` metadata.
   - blake3 unchanged → skip write (logged, no SSB churn).
   - Lock acquisition via B-31 IndexLock(LockScope::Index) — cosmos
     index lives in same lock namespace as code (single writer guarantee).

4. Lazy refresh on demand: `cosmos_get(name) -> Result<Artifact>` API.
   If artifact stale (age > 2× refresh_cadence), trigger blocking
   refresh up to 30s timeout; fall back to stale artifact + WARN
   metadata if timeout.

### Phase 2 — Pipeline integration + grounding (Session 2)

5. New `CosmosStage` in QueryPipeline (knowledge/src/pipeline.rs):
   inserted between ArtifactStage (B-35) and ARagStage. Same hybrid
   (vector + BM25 RRF) retrieval contract as B-35.

6. New `GroundingMethod::CosmosCheck` (8th method):
   - Dispatched by PipelineGrc when claim category ∈ {design,
     ip_boundary, monetization, app_store, marketing_pattern,
     ux_convention}.
   - Returns same `VerificationResult` shape as ArtifactCheck (binary
     pass/fail with quote + source_ref).
   - Failure path: claim ungrounded → flagged in plan grounding
     sweep → blocks AUTO-advance per B-22 evidence check.

7. Designer + idea-framer + per-level auditor briefing injection:
   - `compile_cosmos_briefing(project_class, tier_relevance_threshold)`
     selects artifacts with `project_classes contains class` AND
     `tier_relevance.{tier} >= threshold`.
   - Returns AVAILABLE_COSMOS block (≤ 1500 tokens default budget;
     auto-summarized via ShortCoder if oversize).
   - B-40 idea-framer: full cosmos briefing, threshold=0.5.
   - B-41 mode=concepts: full briefing, threshold=0.4.
   - B-41 mode=phases: filtered briefing, threshold=0.6.
   - B-41 mode=babysteps: skipped (artifacts irrelevant at line scope).
   - Designer recursive decomposition: full briefing, threshold=0.5.

### Phase 3 — Anti-staleness + dashboard (carried into Session 2)

8. Time-decay confidence: artifact age computed from `refreshed_at`.
   Status enum: FRESH (age ≤ refresh_cadence), STALE (age ≤ 2×), DECAYED
   (age > 2× — drops to SHADOW, callers WARN'd, refresh re-prioritized).
   Worker bumps DECAYED artifacts to top of refresh queue.

9. Cosmos refresh emits SSB: `cosmos_refresh_started`,
   `cosmos_artifact_updated`, `cosmos_artifact_decayed`,
   `cosmos_grounding_used`. 4 new action_types, schema +1 bump.

10. Dashboard surface (V2):
    - `drawers/CosmosDrawer.svelte` (NEW): list of cosmos artifacts
      with status (FRESH/STALE/DECAYED), refreshed_at, manual "refresh
      now" button per artifact.
    - `MonitorWindow.svelte` adds "cosmos: X fresh / Y stale / Z decayed"
      mini-badge during PLANNING / FRAMING.
    - New stub `get_cosmos_status` + `trigger_cosmos_refresh` →
      live Tauri commands.

11. Default cosmos artifact set (≥ 12 shipped at first install):
    - design_trends_2026 (mobile + web)
    - ios_app_store_guidelines
    - android_play_store_guidelines
    - web_app_ux_patterns_2026
    - saas_dashboard_patterns
    - copyright_tradedress_guide
    - asset_licensing_quickref (CC0/Unsplash/Material/Lucide/etc)
    - monetization_patterns_consumer_apps
    - monetization_patterns_saas
    - color_palette_trends_2026
    - typography_trends_2026
    - accessibility_wcag_aa_quickref

## WIRING_CONTRACT

produces_symbols:
  - elai_knowledge::cosmos_worker::CosmosWorker
  - elai_knowledge::cosmos_worker::cosmos_get
  - elai_knowledge::cosmos_worker::CosmosStatus (FRESH/STALE/DECAYED)
  - elai_knowledge::pipeline::CosmosStage
  - elai_reasoning::grounding::verify::GroundingMethod::CosmosCheck (8th variant)
  - elai_orchestrator::briefing::compile_cosmos_briefing
  - 4 SSB action_types: cosmos_refresh_started, cosmos_artifact_updated, cosmos_artifact_decayed, cosmos_grounding_used
  - 12 default cosmos artifacts populated at ~/.elai/cosmos/artifacts/

consumes_symbols:
  - B-35 artifact pipeline primitives (Bm25Index, vector store, ArtifactStage)
  - elai_orchestrator::research_tree::ResearchTree (existing)
  - elai_web_search::source_quality::SourceQualityTable (existing)
  - B-31 IndexLock (LockScope::Index)
  - B-23 Layer 1A scheduler (existing tick mechanism)

reachable_from:
  - Layer 1A scheduler tick (cosmos_worker.tick() at hourly cadence)
  - Designer / idea-framer (B-40) / per-level auditor (B-41) briefing builders
  - PipelineGrc::verify_claim (CosmosCheck dispatch on relevant categories)
  - dashboard-v2 CosmosDrawer manual refresh button

stub_ban:
  - cosmos artifacts must NOT be hand-written (research-driven only —
    blake3 mismatch from manual edit triggers warning + re-research)
  - cosmos refresh must NOT bypass min_source_tier filter
  - DECAYED artifacts must NOT be silently used (caller WARN required)
  - briefing injection must NOT exceed budget (1500 tokens default;
    ShortCoder fallback or skip — never overflow context)
  - cosmos worker must NOT run during AUDITING/EXECUTING states (perf)

frontend_pair: dashboard-v2 CosmosDrawer + MonitorWindow status badge

observable_effect:
  trigger: |
    fresh ELAI install → run /task `make a plant-ID app like PictureThis`.
    On first invocation, FRAMING substate (B-40) requests AVAILABLE_COSMOS
    briefing. Initial cosmos is empty → worker dispatches mandatory
    artifacts (design_trends_2026, ios_app_store_guidelines,
    monetization_patterns_consumer_apps) blocking up to 30s each.
  expect_ssb: |
    cosmos_refresh_started (×3) →
    cosmos_artifact_updated (×3 with blake3 hashes) →
    Designer briefing built with non-empty AVAILABLE_COSMOS block →
    cosmos_grounding_used (×N — at least once per relevant claim
    category in plan-grounding sweep).
  expect_artifact: |
    ~/.elai/cosmos/artifacts/design_trends_2026.md exists,
    blake3-hashed in .toml metadata,
    refreshed_at within last 30 minutes.

## DETERMINISTIC_CHECK

```bash
# 1. Schema bumped (+1 from current baseline)
rg "SCHEMA_VERSION: u32 =" .elai_cc/crates/orchestrator/src/ssb.rs

# 2. Four new SSB action_types
for t in cosmos_refresh_started cosmos_artifact_updated cosmos_artifact_decayed cosmos_grounding_used; do
  rg -q "$t" .elai_cc/crates/orchestrator/src/ssb.rs || echo "MISSING: $t"
done

# 3. CosmosWorker module exists
test -f .elai_cc/crates/knowledge/src/cosmos_worker.rs

# 4. CosmosStage in pipeline
rg "CosmosStage" .elai_cc/crates/knowledge/src/pipeline.rs

# 5. GroundingMethod::CosmosCheck (8th method)
rg -A 10 "enum GroundingMethod" .elai_cc/crates/reasoning/src/grounding/verify.rs | grep -c "^\s*[A-Z][a-zA-Z]*"
# expected: 8

# 6. Default manifest ships ≥ 12 artifacts
python3 -c "import tomllib; d=tomllib.load(open('.elai_cc/crates/knowledge/seed/cosmos_default_manifest.toml','rb')); assert len(d['cosmos']['artifacts']) >= 12, len(d['cosmos']['artifacts'])"

# 7. blake3 used (anti-stale rule — NOT sha256)
rg "blake3" .elai_cc/crates/knowledge/src/cosmos_worker.rs

# 8. CosmosDrawer.svelte exists in V2 dashboard
test -f .elai_cc/crates/dashboard-v2/src/lib/drawers/CosmosDrawer.svelte

# 9. Briefing budget enforcement
rg "compile_cosmos_briefing" .elai_cc/crates/orchestrator/src/briefing.rs
rg "1500" .elai_cc/crates/orchestrator/src/briefing.rs  # default budget

# 10. ≥ 25 tests
cargo test -p elai-knowledge cosmos 2>&1 | grep "test result"
# expected: ≥ 25 (worker tick, blake3 idempotency, lock contention,
#                 stage retrieval, status decay, briefing budget,
#                 grounding dispatch)

# 11. cargo check + svelte-check clean
cargo check --workspace
cd .elai_cc/crates/dashboard-v2 && npx svelte-check
```

## ACCEPTANCE

- [ ] ~/.elai/cosmos/ layout created with manifest + artifacts/ + index/
- [ ] CosmosWorker.tick() runs on Layer 1A schedule; skips up-to-date
      artifacts; respects B-31 lock
- [ ] blake3 idempotency: re-running tick() with no source change
      produces no artifact_updated SSB rows
- [ ] CosmosStage wired into QueryPipeline between ArtifactStage and ARagStage
- [ ] GroundingMethod::CosmosCheck dispatched by PipelineGrc for
      design/ip_boundary/monetization/app_store/marketing_pattern/ux_convention
      claim categories
- [ ] Designer + idea-framer + per-level auditor (when shipped) consume
      AVAILABLE_COSMOS briefing block; budget enforced (≤ 1500 tokens
      default)
- [ ] Time-decay: artifact older than 2× refresh_cadence flagged DECAYED;
      caller emits WARN; worker re-prioritizes refresh
- [ ] Default ≥ 12 cosmos artifacts populated within 30 minutes of fresh install
- [ ] dashboard CosmosDrawer renders status + manual refresh; no mock-data
- [ ] Ablation: 30-task fixture (consumer-app shaped), measure plan-
      grounding sweep BLOCK rate with vs without cosmos active —
      expect ≥ 50% reduction in ungrounded-claim BLOCKs (cosmos provides
      the grounding source that was previously missing)
- [ ] ≥ 25 tests pass; svelte-check 0 errors; cargo check workspace clean

## Tests to add

- crates/knowledge/tests/cosmos_worker_tick.rs: tick() schedule logic,
  per-cadence due-detection, blake3 idempotency
- crates/knowledge/tests/cosmos_lock_contention.rs: cosmos worker
  + code indexer concurrent access via B-31 IndexLock
- crates/knowledge/tests/cosmos_pipeline_stage.rs: CosmosStage retrieval
  contract, hybrid scoring fairness vs ArtifactStage
- crates/contracts/tests/cosmos_grounding_contract.rs: PipelineGrc
  dispatches CosmosCheck for relevant claim categories; failure path
  produces ungrounded_claims entry
- crates/orchestrator/tests/cosmos_briefing_budget.rs: compile_cosmos_briefing
  enforces 1500-token cap; ShortCoder fallback on overflow
- crates/contracts/tests/cosmos_decay_contract.rs: DECAYED status
  emits WARN to caller, doesn't silently serve stale data

## Files touched

- CREATE: .elai_cc/crates/knowledge/src/cosmos_worker.rs (~500 LOC)
- CREATE: .elai_cc/crates/knowledge/seed/cosmos_default_manifest.toml (≥ 12 artifacts)
- CREATE: .elai_cc/crates/knowledge/tests/cosmos_worker_tick.rs (~200 LOC)
- CREATE: .elai_cc/crates/knowledge/tests/cosmos_lock_contention.rs (~150 LOC)
- CREATE: .elai_cc/crates/knowledge/tests/cosmos_pipeline_stage.rs (~180 LOC)
- CREATE: .elai_cc/crates/contracts/tests/cosmos_grounding_contract.rs (~140 LOC)
- CREATE: .elai_cc/crates/orchestrator/tests/cosmos_briefing_budget.rs (~120 LOC)
- CREATE: .elai_cc/crates/contracts/tests/cosmos_decay_contract.rs (~100 LOC)
- CREATE: .elai_cc/crates/orchestrator/src/briefing.rs (~250 LOC; cosmos + future briefing helpers)
- CREATE: .elai_cc/crates/dashboard-v2/src/lib/drawers/CosmosDrawer.svelte (~280 LOC)
- CREATE: workflow/reasoning/B-42-decisions.md
- MODIFY: .elai_cc/crates/knowledge/src/pipeline.rs (+ CosmosStage)
- MODIFY: .elai_cc/crates/knowledge/src/lib.rs (+ cosmos_worker module export)
- MODIFY: .elai_cc/crates/reasoning/src/grounding/verify.rs (+ CosmosCheck variant)
- MODIFY: .elai_cc/crates/orchestrator/src/ssb.rs (+4 action_types, schema bump)
- MODIFY: .elai_cc/crates/orchestrator/src/task_runner.rs (+ Layer 1A cosmos tick wiring)
- MODIFY: .elai_cc/crates/dashboard-v2/src-tauri/src/lib.rs (+ get_cosmos_status, trigger_cosmos_refresh)
- MODIFY: .elai_cc/crates/dashboard-v2/src/lib/api/stub.ts (+ live invokes)
- MODIFY: .elai_cc/crates/dashboard-v2/src/lib/windows/MonitorWindow.svelte (+ cosmos badge)

## Decision doc

Write workflow/reasoning/B-42-decisions.md before EXECUTING:
- Why ~/.elai/cosmos/ user-global location vs per-project: cosmos
  knowledge is genuinely cross-project (design trends apply to every
  app); per-project would force redundant research per project. The
  alternative — a system-wide /etc/elai/cosmos — fails on Windows and
  forces sudo on Linux.
- Why blake3 (not sha256): CLAUDE.md anti-stale rule.
- Why min_source_tier defaults to T1: cosmos artifacts are AUTHORITATIVE
  reference; downstream consumers trust them blindly. T2-T3 sources are
  for active research where contradictions can be surfaced; cosmos is
  for ground-truth.
- Why 1500-token briefing budget default: empirical fit for 7B local
  context — leaves ≥ 18K headroom out of 25K hard ceiling for actual
  task content.
- Why DECAYED status doesn't auto-delete: stale data is better than no
  data when refresh fails (e.g., offline). Caller decides via WARN.
- Why we DON'T mirror the user's image's "design hard rules" into cosmos:
  rules are deterministic enforcement (rules.db, B-41), cosmos is
  reference material (knowledge to ground against). Different concerns;
  putting rules in cosmos blurs the deterministic vs probabilistic
  boundary that Phase 3F design protects.

## Next

If B-43 (Visual Auditor) is approved: B-43 consumes cosmos artifacts
(design_trends_2026, color_palette_trends_2026, typography_trends_2026)
as the ground-truth comparison surface for VLM critique.
