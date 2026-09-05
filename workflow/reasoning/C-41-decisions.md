C-41 Application Skills System — Decisions Log (sub-phase 5A)

Session: s-21636 / Tranche 5 / C-41 sub-phase 5A
Date: 2026-05-31
Plan: F:\.ELAI_workflow\docs\thefinalglance\C-plan\C-41_skills_driven_workflow_formalization.md
Status: SHIPPED — 9 SKILL.md files on disk, pre-classify hook live behind ELAI_APPLICATION_SKILLS_ENABLE, contract test green.

GROUND TRUTH AT SUB-PHASE START
The C-41 Rust backend has been shipped for weeks under crates/orchestrator/src/skill_registry/{mod,manifest,discovery,eval,ranker,scoring}.rs (~1502 LOC, 11-test contract suite). What was missing was the DATA + INTEGRATION layer: an empty .elai_cc/skills/ directory, no production caller of top_k_skills, no decision doc reconciling spec drift. Sub-phase 5A closes that gap WITHOUT renegotiating the shipped backend.

DECISION-1 — SPEC-DRIFT acknowledgement (HIGH severity, do not rewrite plan)
The C-41 plan file (line 80-81, line 182-190) says the skills registry lives under crates/agent/src/skills/registry.rs and ships SSB action types "skill_lookup / skill_selected / skill_fell_through". The shipped code lives under crates/orchestrator/src/skill_registry/ and ships SsbActionType::SkillTriggered + SkillExecuted (2-variant pair, SCHEMA_VERSION >= 111). We are NOT rewriting the plan file. Future cross-check passes must treat the plan as superseded by implementation for these specific fields. Rationale: the plan was authored before the orchestrator-side decision was made; rewriting it now would invalidate the ~80% of the plan text that IS still accurate (manifest schema motivation, paper-grounded effect size, 9-skill name list, frontmatter convention).

DECISION-2 — SSB shape (2-action pair stays, no Pattern-2 collapse)
Considered: collapsing SkillTriggered+SkillExecuted into a single Pattern-2 SkillRoutingEvent with a Triggered/Executed discriminator. REJECTED. The 2-action pair mirrors C-40 GithubPr* + tool_dispatch/tool_result precedents (request/effect pair, not 3-arm discriminator). E-43 SkillSystemEvent (Pattern-2) is a SEPARATE event family for HARNESS skills under .claude/skills/ and stays distinct. Collapsing here would buy nothing and lose the join-on-skill_name_hash dashboard query that already pins the pair.

DECISION-3 — Namespace discriminator (type: application in frontmatter)
3 of the 9 application skill names (idea_intake, oracle, planning_workflow) collide on directory leaf name with harness skills under .claude/skills/. The existing discovery walker dedupes case-insensitively with FIRST-WINS root precedence, which would let one skill type shadow the other depending on root order. To document the namespace intent and prepare for future discrimination, every application SKILL.md ships a frontmatter key `type: application`. The current parse_skill_frontmatter_extended silently ignores unknown keys so this is a forward-compat marker, not an active discriminator today. Active discrimination today is by ROOT PATH: the task_runner pre-classify hook supplies ONLY the `.elai_cc/skills/` root, not `.claude/skills/`, so harness skills never enter the application registry. A future schema bump can add `type:` to FrontmatterFields and gate registry membership at parse time.

DECISION-4 — Pre-classify hook gating (wire-dark via env var)
The hook is gated by ELAI_APPLICATION_SKILLS_ENABLE=1 and is OFF by default. Rationale: (a) the dispatcher half (execute_skill for application skills) is not yet implemented, so emitting SkillTriggered without a paired SkillExecuted would create an asymmetric dashboard surface; (b) the wire-dark pattern mirrors the B-40 idea-framing hook (ELAI_IDEA_FRAMING_ENABLE) and E-43 skill-system telemetry (ELAI_SKILL_SYSTEM_ENABLE) — same idiom keeps operator mental model consistent.

DECISION-5 — SkillExecuted emission deferred to sub-phase 5C+
Only SkillTriggered emits in 5A. SkillExecuted requires a real dispatcher binding (call site -> execute_skill -> outcome -> SSB row). Until that dispatcher exists, emitting SkillExecuted would either fabricate the exit_ok signal or always emit exit_ok=true, both of which corrupt the metric. Deferred.

DECISION-6 — Root resolution (.elai_cc/skills/ + ELAI_APPLICATION_SKILLS_DIR override)
The hook resolves the application skills root in this order:
  1. If `ELAI_APPLICATION_SKILLS_DIR` env var is set and non-empty, use that path verbatim.
  2. Else, use `<project_dir>/.elai_cc/skills/` (project_dir resolved by the orchestrator's project resolver).
  3. If project_dir is None, fall back to `<cwd>/.elai_cc/skills/`.
The override exists for test fixtures and per-pod customization. NO `.claude/skills/` root is supplied to keep the namespace clean.

DECISION-7 — Confidence threshold reused at 0.20 (no per-call override)
`SKILL_CONFIDENCE_THRESHOLD = 0.20` lives in scoring.rs and is pinned by the existing contract test. The pre-classify hook reuses this constant verbatim. Per-call threshold overrides are NOT supported in 5A — adding them would invite calibration drift.

DECISION-8 — Top-k = 1 in the pre-classify hook
The hook calls `top_k_skills(&manifests, &ctx, 1)` and emits at most ONE SkillTriggered row per task. Rationale: emitting the full top-K (e.g. top-3) would inflate SSB row counts without changing the routing decision, since the consumer always picks rank=1 when above threshold and falls through otherwise. The ranker remains capable of returning K candidates for the dashboard's "what else could have matched" view, but the pre-classify emission is single-row.

DECISION-9 — 9 skill names + one-line role
  - simple_query: short factual question, no plan, no edit, no audit.
  - focused_edit: targeted single-file edit, scoped auditor run only.
  - heavy_plan: multi-week SaaS/app feature decomposition, full FSM cycle.
  - idea_intake: raw idea framing via idea_framing module (Layer A).
  - oracle: web-grounded research with citations, read-only.
  - planning_workflow: resume / continue / execute an existing phased plan.
  - paper_ingest: arXiv / DOI paper -> RAG + interface map.
  - source_ingest: SDK / repo -> RAG + interface map + reliability tag.
  - benchmark_run: SWE-bench / polyglot / HumanEval / etc, batched.

DECISION-10 — Discovery.rs unchanged
The shipped find_skills + walk_root logic already handles arbitrary roots, sorts entries deterministically, and dedupes case-insensitively. NO changes were made to discovery.rs in 5A. The hook simply supplies a single DiscoveryRoot::skills_dir(...) and consumes the existing return type.

OUT-OF-SCOPE (sub-phase 5A) — TRACKED FOR 5B / 5C
  - SkillsPanel.svelte (dashboard inspector card).
  - commands_c41.rs (Tauri bridge with 2-3 read-only commands).
  - execute_skill dispatcher + paired SkillExecuted SSB row.
  - SkillExecutedPayload emit site.
  - SciQuest-style 100-row eval fixture.
  - prompt-engineer agent prompt scaffold updates referencing top_k_skills context.

VERIFICATION
  - cargo check --workspace: see executor report.
  - c41_skill_routing_contract.rs: 4 tests (nine_canonical_skill_manifests_load, heavy_plan_routes_for_saas_dashboard_prompt, off_topic_prompt_falls_through_or_resolves_simple_query, type_application_field_present_in_canonical_manifests).
  - Existing c41_skills_driven_workflow_contract.rs (357 LOC, 11 tests) remains pinned at SCHEMA_VERSION_PUBLIC >= 111.

ENV VARS ADDED IN 5A
  - ELAI_APPLICATION_SKILLS_ENABLE  (gate: pre-classify hook OFF by default)
  - ELAI_APPLICATION_SKILLS_DIR     (override: skills root path)

NO new SSB action types. NO new dependencies. NO discovery.rs / manifest.rs / scoring.rs / ranker.rs / eval.rs changes. Only task_runner.rs + new test + new data files.
