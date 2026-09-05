Track A — decisions (session s-155876, 2026-05-05)
=====================================================

What
----
Six small SSB+Tauri wiring gap fixes from
TESTED-AND-DASHBOARD-WIRING.md §9 + §11.1 + §11.2. Sequential per item;
single audit cycle at end. No behaviour changes outside the listed files.

Items in declared sequencing order
----------------------------------
A.1 — orchestrator/src/ssb.rs IndexingProgressPayload extend stage to
      "start"|"complete"|"failed", add error_kind/error_message Option<String>;
      emit "failed" row from indexing_service.rs Err arm + cli/src/indexing.rs
      mirror.
A.2 — orchestrator/src/ssb.rs new variant research_path_chosen
      {path, reason, ts_ms}; cli/src/research_pipeline.rs emit row + add
      ResearchAnswer.path: String field with #[serde(default)].
A.3 — cli/src/research_pipeline.rs replace substring `[` guard with
      LazyLock<Regex> matching \[\d+\] in synthesize_text selection branch.
A.4 — orchestrator/src/ssb.rs extend per-source row payload (variant TBD —
      web_research_step OR new web_source_fetched) with fetcher: String
      ("reqwest"|"obscura"); writer call site in web_research crate.
A.5 — pipeline-factory Phase/Step/BabyStep structs add per-struct
      #[serde(default)] OR deserialize_with helper. Mirror the
      deserialize_nodes_tolerant + ConvergenceStatus pattern from s-1928380.
A.6 — orchestrator/src/ssb.rs new variant grounding_cache_event {kind,
      key_fingerprint (8-char blake3 prefix), level, elapsed_us}; emit at
      research_tree.rs cache.peek site; expose Tauri command
      get_grounding_cache_stats in BOTH dashboard/src/commands.rs (v1)
      AND dashboard-v2/src-tauri/src/lib.rs (v2).

Key decisions
-------------
1. Pattern reuse over invention. Use existing in-tree patterns:
   - LazyLock<Regex> (NOT lazy_static crate) for A.3.
   - blake3 (NOT blake2/sha2) for A.6 fingerprint per anti-stale-training.
   - #[serde(default)] (NOT custom deserialize_with) where possible for A.5.
   - SSB action_type variant pattern matches existing ssb.rs enum surface.
2. New SSB variants MUST add pass-through arms in dashboard commands.rs
   pattern matchers OR the dashboard panics on unknown variant. Auditor
   verifies.
3. ResearchAnswer.path uses #[serde(default)] so existing struct literals
   (research_pipeline.rs:417, 489 per context-compiler) compile unchanged.
4. A.6 may require research_decompose signature change to thread Arc<Ssb>;
   default callers pass None. Confirm with grep before touching the 4
   cascade call sites flagged by context-compiler.

Expected test outcome
---------------------
- cargo check --workspace: 0 errors, 0 NEW warnings.
- cargo test -p elai-orchestrator -p elai-cli: PASS.
- New contract tests for A.1/A.2/A.3/A.4/A.6 land green; A.5 either PASSES
  on all 3 D5 queries OR documents a remaining Qwen-3B quirk that ONLY
  Track C-B constrained decoding can fix (acceptable per success criteria).

Unknowns
--------
- A.4: which existing SSB variant carries the per-source row; context-
  compiler enumerates web_research/src/* writers but final variant choice
  defers to execution agent's grep.
- A.5: extent of Phase/Step/BabyStep schema — if fields beyond `name` are
  also strict, scope expands. Capped: do NOT widen to deserialize_nodes_tolerant
  rewrite — only add #[serde(default)] to existing structs.
- elai.map staleness: per context-compiler note (NEXT-SESSION-PROMPT D7),
  raw ripgrep of .elai_cc/crates is required for ResearchAnswer struct
  literal sites BEFORE editing.

Risks (from context-compiler cross-crate report)
------------------------------------------------
- A.6 cascade: research_decompose signature change propagates to
  task_runner.rs, recursive_research.rs, plus 2 contract test files.
  Use None as default for all existing callers.
- A.2 cascade: ResearchAnswer struct literal sites in research_pipeline.rs:
  417 + 489 (and possibly cli/main.rs). #[serde(default)] keeps
  serde-roundtrip happy but Rust struct literals still need the field —
  add `path: Default::default()` at every literal OR use a constructor.

References
----------
- Context package: workflow/sessions/s-155876/context-package-track-A.md
- Source-of-truth: TESTED-AND-DASHBOARD-WIRING.md §9, §11.1, §11.2
- B1.2 prior fix detail: workflow/sessions/s-1928380/NEXT-SESSION-PROMPT.md
