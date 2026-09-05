E-48 web-research orchestrator hardening — decisions

Session: cluster s-25120 completion-drive session 4/4 final
Plan: docs/thefinalglance/E-plan/E-48_web_research_orchestrator.md
Status: substrate shipped wire-dark, live wiring + dashboard panels deferred
SCHEMA bump: 184 -> 185 (single atomic). TOTAL_VARIANTS 457 -> 458.
ENV gate: ELAI_WEB_RESEARCH_ORCHESTRATOR_ENABLE.

DECISION-1 (Pattern-2 single variant, NOT 17-variant cascade)
The plan WIRING_CONTRACT named 17 SSB action_types. We collapsed to ONE
SsbActionType::WebResearchEvent + WebResearchEventKind 10-arm
discriminator per the multi-event Pattern-2 precedent (E-50 11-arm /
E-4 13-arm / E-54 14-arm / E-46 8-arm). Several plan-named fire-points
fold into a single Kind via outcome discriminators in details_json:
  * citation_dead_url / citation_invalid_arxiv / citation_invalid_doi /
    citation_unfaithful all ride CitationVerificationOutcome with the
    `outcome` field discriminator
  * tier_disabled_quota_exceeded / tiered_search_degraded_to_t2_only
    ride TieredSearchDispatched with the `status` field discriminator
  * citation_verification_pass_rate is computed read-side from
    CitationVerificationOutcome rows (no separate producer fire-point)
  * preemptive_stop_threshold_uncalibrated is a flag inside
    ResearchPreemptiveStop, not its own fire-point
  * contradiction_check_budget_clamped is a flag inside
    ContradictionNodeMaterialized, not its own fire-point
Saves 16 SCHEMA slots + 16 alphabetical inserts in action_type_lock.golden
per the existing single-variant cluster precedent.

DECISION-2 (naming collisions resolved)
Cross-check report flagged 3 high-severity collisions. Resolution:
  * SourceTier (domain quality, existing) STAYS unchanged. New backend
    identity enum = BackendTier (T1=Brave / T2=SearXNG / T3=Mojeek) in
    crates/web_search/src/tiered_search_pool.rs. Both enums coexist.
  * ResearchPlan (existing at research_tree.rs:627, tree-input shape)
    STAYS unchanged. New editable-CoT artifact = WebResearchPlan in
    crates/orchestrator/src/research/web_orchestrator/editable_plan.rs.
    Different concerns — both can coexist on the same research run.
  * Contradiction (existing at research_tree.rs:443) EXTENDED with 3
    #[serde(default)] fields (prune_protected, surfaced_at_unix_ms,
    classification_reason) rather than parallel ContradictionNode
    struct. Avoids the two-struct drift the cross-check flagged.
  * CrossEncoderReranker (knowledge crate) STAYS for RAG-retrieval
    reranking. New web-search reranker in
    crates/web_search/src/cross_encoder_rerank.rs covers web-search
    results reranking. Different surfaces, no shared code path needed.

DECISION-3 (landing in research/web_orchestrator/ subtree)
Per session-prompt anchor "orchestrator/src/research/web_orchestrator/
subtree (next to E-22, E-23, E-38 research/)". Created:
  crates/orchestrator/src/research/mod.rs           (subtree root)
  crates/orchestrator/src/research/web_orchestrator/mod.rs
    (env gate const + is_enabled helper + module map)
  + 7 sibling modules: editable_plan, speculative_pruner,
    conformal_stop, contradiction_classifier, citation_verifier,
    recency_decay, tier_pool_telemetry, producers.
Future E-22 (research index telemetry) + E-23 (developer discourse
research surfaces) will land as siblings under research/.

DECISION-4 (NO dashboard scope per session prompt)
Plan task 11 specifies 4 Svelte panels + 4 Tauri commands. Session
prompt anchor "NO dashboard wired=N". Deferred to follow-up patch.
Backend wire-dark substrate ships; readers can hit the SSB rows via
the existing dashboard SsbViewer once panels land.

DECISION-5 (Contradiction extension, not new struct)
Plan task 7 names a ContradictionNode struct. The existing
Contradiction struct already covers (claim_a, source_a, claim_b,
source_b, resolution). We extended it with 3 #[serde(default)] fields
rather than ship a parallel struct:
  * prune_protected: bool — speculative pruner refusal flag
  * surfaced_at_unix_ms: i64 — materialization timestamp
  * classification_reason: String — LLM rationale (capped at 200 chars)
Legacy rows deserialize cleanly (all fields default). The legacy
find_contradictions stub keeps prune_protected=false so existing
prune_speculative_children semantics are unchanged.

DECISION-6 (ConvergenceStatus PreemptiveUnsolvable variant added)
Task 6 triple-threshold convergence. Added new variant + permissive
deserializer arm. All known consumers use wildcard match patterns
(_ => ...) or matches!(_, Failed) — non-exhaustive call sites are safe.
Default threshold_uncalibrated=true; emits startup warning per
plan open question #5.

DECISION-7 (RUNPOD-PENDING live client wiring)
Following clients are substrate-only this commit; live HTTP wiring is
deferred:
  * BraveClient / MojeekClient — require API keys per their dashboards
  * Crawl4aiClient — requires self-hosted Playwright endpoint
  * JinaReaderClient — managed HTTP GET, free-tier rate limits need
    verification before shipping live
  * Sidecar /rerank endpoint — does not exist (plan open question #5);
    cascade falls through to Cohere if configured, then to no-rerank
  * Sidecar /contradiction endpoint — does not exist; SidecarClassifier
    returns Orthogonal until wiring lands
  * SemanticCite faithfulness judge — RUNPOD-PENDING via PendingJudge
    default (returns Support to avoid spurious unfaithful flags)
All substrate compiles + unit-tests pass; the orchestrator stays in B-7
behavior unless the env gate flips AND a live client gets wired.

DECISION-8 (compile-time VerifiedCitation invariant)
Task 8 + 9 requirements: (a) no citation in final report without
verification, (b) no lead-orchestrator citation outside SEARCH set.
Enforced via Rust's privacy system:
  * VerifiedCitation::new is pub(super) — only modules inside the
    web_orchestrator subtree can mint instances.
  * mint_verified is pub(super) in citation_verifier so the producers
    module can build them, but external callers cannot.
  * AttributionSet::lookup returns AttributionLookup (Allowed/Blocked);
    Blocked routes emit citation_attribution_violation_blocked SSB row.

DECISION-9 (anti-DDG guard in tiered_search_pool)
Plan stub_ban: NO DuckDuckGo as a tier (Bing-derivative per factually.co
2026). Substrate intentionally has no BackendTier::Ddg variant; the
no_duckduckgo_as_tier test pins this against regression.

DECISION-10 (blake3 ONLY)
URL canonicalization hash in tiered_search_pool + blake3_prefix_hex in
producers both use blake3::hash. CLAUDE.md anti-stale-training HARD
RULE: never blake2/sha2 for non-RFC uses.

Papers used (verified May 2026 via WebSearch on previous E-plan cycles):
  * arXiv:2510.05145  FlashResearch (Task 5)
  * arXiv:2602.06718  GhostCite (Task 8)
  * arXiv:2602.05930  Compound Deception (Task 8 + 9)
  * arXiv:2604.03173  CITEVERIFIER/SemanticCite/CiteGuard (Task 8 stage-d)
  * arXiv:2601.18724  HalluCitation Matters (Task 8 stage-b)
  * arXiv:2602.03814  Conformal Thinking (Task 6)
  * arXiv:2504.00180  Contradiction Detection in RAG (Task 7)
  * arXiv:2511.06668  Healthcare RAG never-silently-resolve (Task 7)
  * arXiv:2510.03418  LegalWiz (Task 7)
  * arXiv:2509.19376  Solving Freshness in RAG (Task 10)
  * arXiv:2509.23519  ReliabilityRAG (Task 2)
  * arXiv:2410.22954  Source-reliability heterogeneity (Task 2)
  * arXiv:2510.12164  Survey on Parallel Reasoning (Task 5)
  * simonwillison.net/2025/Jun/14  Anthropic multi-agent (Task 9)
  * docs.cohere.com/docs/reranking-best-practices (Task 4)
  * api-dashboard.search.brave.com/documentation/pricing (Task 2)
  * mojeek.com (Task 2)
  * spider.cloud/blog/firecrawl-vs-crawl4ai-vs-spider-honest-benchmark (Task 3)
  * factually.co private-search 2026 (Task 2 anti-DDG guard)

Files touched (CREATE unless noted MODIFY):
  CREATE crates/orchestrator/src/research/mod.rs
  CREATE crates/orchestrator/src/research/web_orchestrator/mod.rs
  CREATE crates/orchestrator/src/research/web_orchestrator/editable_plan.rs
  CREATE crates/orchestrator/src/research/web_orchestrator/speculative_pruner.rs
  CREATE crates/orchestrator/src/research/web_orchestrator/conformal_stop.rs
  CREATE crates/orchestrator/src/research/web_orchestrator/contradiction_classifier.rs
  CREATE crates/orchestrator/src/research/web_orchestrator/citation_verifier.rs
  CREATE crates/orchestrator/src/research/web_orchestrator/recency_decay.rs
  CREATE crates/orchestrator/src/research/web_orchestrator/tier_pool_telemetry.rs
  CREATE crates/orchestrator/src/research/web_orchestrator/producers.rs
  CREATE crates/web_search/src/tiered_search_pool.rs
  CREATE crates/web_search/src/extractor_pool.rs
  CREATE crates/web_search/src/cross_encoder_rerank.rs
  MODIFY crates/orchestrator/src/ssb.rs (Pattern-2 lockstep additions)
  MODIFY crates/orchestrator/src/lib.rs (pub mod research)
  MODIFY crates/orchestrator/src/research_tree.rs (Contradiction +3 fields,
                                                   ConvergenceStatus +variant)
  MODIFY crates/orchestrator/tests/action_type_lock.rs (TOTAL +1, ALL_VARIANTS +1,
                                                       exhaustive_match_guard +1)
  MODIFY crates/orchestrator/tests/action_type_lock.golden (ordinals + total)
  MODIFY crates/web_search/Cargo.toml (workspace blake3 dep)
  MODIFY crates/web_search/src/lib.rs (3 new pub mod)
  MODIFY workflow/reasoning/E-48-decisions.md (this file — overwrites prior draft)

Deferred (per session prompt + RUNPOD-PENDING):
  * NO dashboard panels (Task 11)
  * NO Tauri commands (Task 11)
  * NO CLI --interactive-research-plan flag (Task 1 plumbing)
  * NO live BraveClient / MojeekClient / Crawl4aiClient / JinaReaderClient
  * NO sidecar /rerank or /contradiction endpoint wiring
  * NO elai_cc.toml [web_search.*] config blocks
  * NO WebResearchOrchestrator integration of the new pieces into
    run_node_loop (substrate-only; integration is a follow-up to keep
    this commit scope-lean)
  * NO contract tests under crates/contracts/tests (unit tests inside
    each module cover the substrate; contract tests follow when the
    integration patch lands)

Open questions (deferred to follow-up cycles):
  * Conformal lower-threshold calibration set (open Q5) — ELAI does not
    yet have 30+ tagged-unsolvable cases. Default stays conservative;
    threshold_uncalibrated=true on every emitted row.
  * Brave free-credit dynamics — quota size shrunk Feb-2026 per
    costbench.com; SSB row records `disabled_tier` when quota
    exhausts.
  * Mojeek coverage on niche queries — third in cascade; downstream
    dedup handles empty T3 cleanly.
  * Speculative-pruner threshold 0.10 — calibration data accrues
    automatically via the SpeculativeBranchPruned row's evidence_delta
    field; future patch can tune.
