# E-11: Idea-to-SaaS prior-art integration (production-grade gates + scope-clarity ceiling)
Objective: translate the 2024-2026 idea-to-app prior art into concrete additions on top of the SHIPPED B-40 Idea-Framer + per-level critic loop. Bake the empirically-grounded production-readiness gates (Tenzai/Escape security-checklist, slopsquatting auto-verify, edit-format discipline, scope-ceiling ack) into ELAI's auditor and idea-critic so the system stops at "production" instead of "demo".
Estimated effort: 2-3 sessions (~10-12 h).
Pre-requisites: B-40 SHIPPED (idea-framing types + FsmSubstate::Framing + 5 SSB variants); B-41 SHIPPED (auditor_mode + per_level_audit_dispatch); C-21 PLANNED (MistakeBook automation); C-31 PLANNED (FilePlan); C-32 PLANNED (Skeleton-first + MiniCritic).
Parallel-safe with: E-12 (production fullstack codegen), E-13 (TDD), C-21..C-32 wave.
Source report: workflow/improvement-sweep-2026-05-19/reports/11-idea-to-saas-prior-art.md

## Context

The May-2026 survey of v0.app, Bolt, Lovable, Devin, Replit Agent 3, Cursor 2.0, Codex CLI, Aider and 15+ post-mortems converges on a single empirical claim: "idea > working demo" is solved in narrow stacks (Next.js/Supabase) but "idea > production SaaS" is solved by nobody. Veracode (2025, 100+ LLMs) measured 45% of AI-generated code failing basic security tests; Tenzai (CSA Dec 2025) found 100% of 15 vibe-coded apps missing CSRF and security headers; Escape.tech (Baytech 2025) found 65% of 1,400 apps with security issues, 400+ exposed secrets; the Help Net Security 2025 study put package-name hallucination at 19.7% across LLMs. The frontier SWE-bench Verified vs SWE-Bench Pro cliff (87.6% → 23%) and Devin's documented 3/20 success rate (Answer.AI, Jan 2025) both confirm that benchmark-tuned performance does not survive production ambiguity. ELAI's B-40 Idea-Framer + idea-critic loop + B-41 per-level auditor cover the framing and verdict structure, but the auditor today does not run the Tenzai-style security probes, does not auto-verify generated import lines against npm/PyPI/crates.io, does not emit a measurable scope-clarity score, and does not have a hard-abort lane for the "Devin pursues impossible solution for days" failure mode. This E-N closes those four gaps with grounded additions that map 1:1 to the report's top recommendations.

## Existing state (B/C/D coverage)

- EXTENSION of B-40 (SHIPPED, orchestrator::idea_framing at .elai_cc/crates/orchestrator/src/idea_framing.rs:795+, FsmSubstate::Framing, AgentRole::IdeaFramer + IdeaCritic, 5 SSB variants, 4 research presets in elai_cc.toml verified by idea_framing_unit.rs:240) — adds scope-clarity score + a measurable HITL gate the critic emits.
- EXTENSION of B-41 (SHIPPED, auditor_mode at .elai_cc/crates/orchestrator/src/auditor_mode.rs:307+, per_level_audit_dispatch, AgentRole +5, AuditTimelineWindow) — adds production-grade auditor mode beneath the existing per-level modes.
- AUGMENTS C-32 Code-Completeness Layer 2 (PLANNED) — the edit-format / unified-diff discipline (Aider grounding) goes into the executor path C-32 introduces.
- AUGMENTS C-21 Self-Modification / MistakeBook automation (PLANNED) — slopsquatting auto-verify becomes a MistakeBook category and a pre-commit hook here.
- NEW relative to B/C/D: scope-ceiling acknowledgement, the "Cognition junior-at-execution honesty" gate (`scope_clarity_score` < threshold → MANDATORY HITL questions from idea-critic).
- NET-NEW: ANTI_SPIRAL_HARD_ABORT lane on top of B-22b 4G.9 StagnationDetector — that detector currently rotates personas; this adds a wall-clock budget per phase that, once exceeded, ALWAYS escalates to HITL regardless of pattern.

## Reads (mandatory before coding)

- workflow/improvement-sweep-2026-05-19/reports/11-idea-to-saas-prior-art.md (this report)
- .elai_cc/crates/orchestrator/src/idea_framing.rs lines 795-1180 (existing IdeaFramer + critic surface; do NOT full-read)
- .elai_cc/crates/orchestrator/src/auditor_mode.rs lines 280-540 (per-level audit format_verdict, finding aggregation)
- .elai_cc/crates/contracts/tests/idea_framing_contract.rs (existing acceptance surface; new tests live alongside)
- .elai_cc/crates/orchestrator/tests/research_preset_dispatch.rs (preset dispatch the new security-preset extends)
- .elai_cc/crates/orchestrator/src/anti_spiral_extended.rs (C-10 module, when shipped — this E-N references its surface; if not yet in tree, file-stub the surface and reference C-10)
- workflow/papers/dgm-interface.md (closed-loop self-modification context for the scope-clarity score)
- arXiv:2502.12115 (SWE-Lancer end-to-end test rubric — read via paper-ingestor)
- arXiv:2510.10460 Planner-Coder Gap (rationale for per-level critic addition; already cited in B-40 papers)

## Tasks

1. Add `scope_clarity_score: f32 (0.0..=1.0)` to the IdeaCritic output struct in idea_framing.rs. Score is the IdeaCritic's calibrated estimate of how operationalizable the framing is on a 4-point rubric (problem stated / acceptance criteria measurable / user identifies success / scope bounded). Threshold default 0.6 from .elai_cc/elai_cc.toml [idea_framing] scope_clarity_min. Below threshold the critic MUST emit at least three concrete clarifying questions and the IdeaFramer FSM substate Framing transitions to a new substate FramingHitlBlocked (do not auto-advance). Add SSB action_type `scope_clarity_below_threshold` (4-byte u32 + payload {score, questions[]}). Schema bump +1.

2. Add a `production_grade` auditor mode under .elai_cc/crates/orchestrator/src/auditor_mode.rs alongside the existing per-level modes. New variant `AuditorMode::ProductionGrade` triggers a 15-item checklist runner against the produced artifact tree. The runner shells out (Tier C) to: gitleaks, semgrep with `--config=p/owasp-top-ten`, lighthouse-axe (only when frontend_pair is non-null), npm-audit/pip-audit/cargo-audit, and a SQLite probe of any Postgres migration emitted (`SELECT tablename FROM pg_tables WHERE rowsecurity=false` must be empty). The runner aggregates findings into the existing `LevelAuditVerdict.findings` Vec with new finding categories `CSRF_MISSING`, `SECRET_LEAK`, `RLS_OFF`, `OWASP_TOP10`, `PACKAGE_HALLUCINATED`. Each finding carries `Severity::Critical` and is FAIL-CLOSED at production-grade tier (advisory only at lower tiers, behind a config flag).

3. Add a `package_existence_verifier` Tier-1 check that runs PostToolUse on any executor write touching package.json / requirements.txt / Cargo.toml / pyproject.toml / pnpm-lock.yaml. For every added import line emit `PackageRef { name, registry, line }` and verify against the registry index with a HEAD request (npm view / pypi.org/pypi/{pkg}/json / crates.io/api/v1/crates/{pkg}); cache positive hits in `~/.cache/elai/pkg_verify.sqlite3` keyed by (registry, name, requested_version) with 7-day TTL. Any unresolvable name BLOCKS the write and emits a new MistakeBook category `Slopsquatting` per C-21's record_mistake_event path. New SSB action_type `package_hallucination_detected` with payload {pkg, registry, line, suggested_alternative}.

4. Promote idea-critic from the single Concept-level loop to a per-level critic at Concept / Phase / Step / BabyStep. The Step+BabyStep critic is a lighter prompt (model=local, max 200 tokens) and runs AFTER the executor proposes the FILE_PLAN block (C-31). Its verdict feeds the existing per_level_audit_dispatch; specifically the `RespawnOnce` outcome flows back to idea-critic with the FILE_PLAN as a counter-evidence input, mirroring the Planner-Coder Gap (arXiv:2510.10460) mitigation. New action_type `per_level_critic_verdict`.

5. Implement ANTI_SPIRAL_HARD_ABORT in .elai_cc/crates/orchestrator/src/anti_spiral_extended.rs (or adjacent — if C-10 not yet shipped, stage the addition as a delta plan). Add `phase_wall_clock_budget_secs` per FSM phase (default 7200 = 2 h; override per phase via elai_cc.toml [anti_spiral.budgets]). When phase elapsed > budget AND any of the 4 existing StagnationPatterns is active for ≥ 50% of last 10 minutes, the detector promotes from "switch persona" to `EscalationTrigger::HumanIntervention` and emits `anti_spiral_hard_abort` SSB. The hard-abort path freezes the current sub-tree, snapshots the in-flight files to `.elai_cc/sessions/{id}/abort/`, and posts a notification via the existing webhook channel.

6. Add Aider-style unified-diff edit format to the executor. New crate-internal type `EditFormat { Unified, SearchReplace, FullFile }` in .elai_cc/crates/orchestrator/src/edit_format.rs. Per-role default: executor=Unified, designer=FullFile (templates only). On unified-diff apply failure (line context drift), fall to SearchReplace; on second failure, FullFile and emit `edit_format_downgrade` SSB. Wire into C-32's StructuralRetrySequencer if both ship in same wave; if not, the new file is the carrier and C-32 picks it up. Grounding: Aider polyglot leaderboard data + Roo Code 30% token savings.

7. Add `[research_presets.production_saas]` entry to elai_cc.toml extending B-40's research_presets. Mandatory streams: tenzai_security_checklist (T1), owasp_top10_for_llm_apps (T1), package_registry_lookup (T2), rls_policy_template (T2). Optional: lovable_postmortems (T3), devin_failure_modes (T3). IdeaFramer auto-selects production_saas when classified ProjectClass is SaaS / API / web-app AND user explicitly typed "production" / "deploy" / "ship" in the brief. Acceptance: `[idea_framing.preset_dispatch] production_saas_triggers_at_keywords = ["production","deploy","ship","customers"]`.

8. Add a measurable `scope_ceiling` advisory message to the IdeaFramer when the brief implies a feature class known to fail across the surveyed products (Replit Agent 3's external API auth, Devin's ambiguous greenfield with mid-task scope change). Hardcode the 4 known-failure-class triggers in idea_framing.rs as a constant table `KNOWN_DEMO_VS_DEPLOY_CHASMS` and emit an advisory SSB row `scope_ceiling_advisory` with the triggered class + the cited post-mortem URL. This is advisory only — does not block — but the dashboard surfaces it as a yellow banner.

9. Wire the production-grade auditor mode into the existing per_level_audit_dispatch as a CONCEPT-LEVEL gate when ProjectClass==SaaS. The dispatch path stays inside auditor_mode.rs format_verdict surface; the 15-item checklist runs as a single Tier-C subprocess pool (max 4 parallel, 90s timeout, 5s grace). Failure produces a `LevelAuditVerdict.outcome = Cascade` so the existing B-41 cascade machinery propagates the failure up.

10. Author the dashboard surface delta. New panel `IdeaFramerSecurityPanel.svelte` under .elai_cc/crates/dashboard-v2/src/lib/components/ rendering: scope_clarity_score gauge, scope_ceiling advisory banner, production-grade checklist 15-row table with green/red/grey cells, slopsquatting recent-blocked list, anti-spiral hard-abort log. Mount in `lib/drawers/OrchestrationDrawer.svelte`. Two new Tauri commands `get_security_checklist_status`, `get_scope_clarity_history`. Do NOT introduce new SsbViewer state — read from existing per_level_audit rows + the new action_types.

## WIRING_CONTRACT
produces_symbols:
  - orchestrator::idea_framing::IdeaCriticVerdict::scope_clarity_score
  - orchestrator::auditor_mode::AuditorMode::ProductionGrade
  - orchestrator::package_verifier::PackageVerifier::verify_added_imports
  - orchestrator::anti_spiral_extended::EscalationTrigger::HumanIntervention (extended)
  - orchestrator::edit_format::EditFormat (new enum)
consumes_symbols:
  - orchestrator::idea_framing::IdeaFramer::run (extended; emits scope_clarity_below_threshold)
  - orchestrator::per_level_audit_dispatch (B-41) consumes per_level_critic_verdict rows
  - orchestrator::mistake_book (C-21) consumes Slopsquatting category
  - dashboard-v2::commands::get_security_checklist_status
  - dashboard-v2::commands::get_scope_clarity_history
reachable_from:
  - crates/cli/src/main.rs::main → task_runner::drive_task → idea_framing::run
  - crates/dashboard-v2/src-tauri/src/lib.rs::submit_task (when [idea_framing] enabled)
stub_ban:
  - todo!() / unimplemented!() in new files
  - Box<dyn Error> in package_verifier.rs (use thiserror PackageVerifyError enum)
  - .unwrap() outside #[cfg(test)] in any new module
  - hardcoded http://localhost for registry endpoints — use elai_cc.toml [package_verifier] base_urls table
frontend_pair: crates/dashboard-v2/src/lib/components/IdeaFramerSecurityPanel.svelte
observable_effect:
  trigger: "elai /task --mode cloud 'build me a production SaaS for project management with auth and Stripe billing'"
  expect_ssb: "scope_clarity_below_threshold AND scope_ceiling_advisory AND per_level_critic_verdict within 60s of FsmSubstate::Framing entry; if user supplies the 3 clarifying answers, framing converges and production_saas preset dispatches"
  expect_artifact: ".elai_cc/sessions/{id}/audit/production_grade_checklist.toml exists with 15 rows; any RLS_OFF / CSRF_MISSING / SECRET_LEAK row is Severity::Critical and Cascade outcome propagates to concept-level"

## DETERMINISTIC_CHECK
```bash
# 1. New SSB action_types present in registry
rg "scope_clarity_below_threshold|scope_ceiling_advisory|package_hallucination_detected|anti_spiral_hard_abort|per_level_critic_verdict|edit_format_downgrade" .elai_cc/crates/orchestrator/src/ssb.rs
# expect: >=6 hits (one per action_type)

# 2. ProductionGrade variant exists
rg "AuditorMode::ProductionGrade" .elai_cc/crates --type rust
# expect: >=2 hits (definition + match arm in format_verdict)

# 3. production_saas preset in elai_cc.toml
rg "\[research_presets.production_saas\]" .elai_cc/elai_cc.toml
# expect: 1 hit

# 4. Package verifier wired into PostToolUse
rg "PackageVerifier::verify_added_imports" .elai_cc/crates --type rust
# expect: >=2 hits (definition + invocation site in post_tool_use hook)

# 5. Hard-abort budget reads from config
rg "phase_wall_clock_budget_secs" .elai_cc/crates --type rust
# expect: >=2 hits

# 6. Smoke: scope-clarity < 0.6 blocks framing autoadvance
cargo test -p elai-orchestrator --test idea_framing_unit scope_clarity_below_threshold_blocks_autoadvance
# expect: pass

# 7. Smoke: slopsquatting detector blocks unknown package on PostToolUse
cargo test -p elai-orchestrator --test package_verifier_contract rejects_hallucinated_package
# expect: pass
```

## ACCEPTANCE
- [ ] IdeaCriticVerdict carries scope_clarity_score f32 ∈ [0,1] and the 5 new SSB action_types are registered in ssb.rs
- [ ] FsmSubstate::FramingHitlBlocked added and entered when scope_clarity_score < 0.6
- [ ] AuditorMode::ProductionGrade implemented, runs 15-item checklist as Tier-C subprocess pool, aggregates into LevelAuditVerdict.findings
- [ ] PackageVerifier wired into PostToolUse; cache at ~/.cache/elai/pkg_verify.sqlite3 with 7-day TTL; blocks write on unresolvable name
- [ ] Per-level critic emits per_level_critic_verdict at Concept/Phase/Step/BabyStep; Step+BabyStep run AFTER FILE_PLAN proposal
- [ ] ANTI_SPIRAL_HARD_ABORT engages on (phase elapsed > budget) AND (stagnation pattern active ≥ 50% of last 10 min) → SSB row + snapshot dir + webhook notification
- [ ] EditFormat enum present; executor default = Unified; downgrade chain Unified → SearchReplace → FullFile with edit_format_downgrade SSB on every step
- [ ] elai_cc.toml [research_presets.production_saas] dispatches when ProjectClass==SaaS AND brief has any of the 4 trigger keywords
- [ ] scope_ceiling advisory fires for any of the 4 KNOWN_DEMO_VS_DEPLOY_CHASMS triggers; dashboard yellow banner renders
- [ ] IdeaFramerSecurityPanel.svelte mounted; 2 Tauri commands wired; no NoopSsbNotifier in path

## Tests to add
- `.elai_cc/crates/orchestrator/tests/scope_clarity_threshold.rs`: 5 cases (clear/ambiguous/missing-success-criteria/scope-creep/multi-domain)
- `.elai_cc/crates/orchestrator/tests/package_verifier_contract.rs`: 8 cases (npm hit, npm miss, pypi hit, pypi miss, crates.io hit, crates.io miss, cache hit, cache miss)
- `.elai_cc/crates/contracts/tests/production_grade_auditor_contract.rs`: 15 cases (one per checklist item) + 3 aggregation tests
- `.elai_cc/crates/orchestrator/tests/per_level_critic_dispatch.rs`: 4 cases (one per level)
- `.elai_cc/crates/orchestrator/tests/anti_spiral_hard_abort.rs`: 3 cases (no-stagnation/stagnation-but-under-budget/stagnation-and-over-budget)
- `.elai_cc/crates/orchestrator/tests/edit_format_downgrade.rs`: 3 cases (unified-only success, fall to searchreplace, fall to fullfile)

## Files touched
- CREATE: `.elai_cc/crates/orchestrator/src/package_verifier.rs` (~280 LOC; PackageVerifier + PackageVerifyError + Sqlite cache)
- CREATE: `.elai_cc/crates/orchestrator/src/edit_format.rs` (~180 LOC; EditFormat enum + apply + downgrade chain)
- CREATE: `.elai_cc/crates/dashboard-v2/src/lib/components/IdeaFramerSecurityPanel.svelte` (~220 LOC)
- MODIFY: `.elai_cc/crates/orchestrator/src/idea_framing.rs` (+~120 LOC; scope_clarity_score + FramingHitlBlocked + scope_ceiling table + 3 new SSB emissions)
- MODIFY: `.elai_cc/crates/orchestrator/src/auditor_mode.rs` (+~180 LOC; ProductionGrade variant + 15-item checklist runner)
- MODIFY: `.elai_cc/crates/orchestrator/src/per_level_audit_dispatch.rs` (+~80 LOC; per_level_critic_verdict emission at Step+BabyStep)
- MODIFY: `.elai_cc/crates/orchestrator/src/anti_spiral_extended.rs` (+~70 LOC; HumanIntervention escalation + budget check)
- MODIFY: `.elai_cc/crates/orchestrator/src/ssb.rs` (+~60 LOC; 6 new SsbActionType variants + payloads)
- MODIFY: `.elai_cc/elai_cc.toml` (+~25 LOC; [research_presets.production_saas] + [package_verifier] + [anti_spiral.budgets])
- MODIFY: `.elai_cc/crates/dashboard-v2/src-tauri/src/commands.rs` (+~60 LOC; 2 new Tauri commands)
- MODIFY: `.elai_cc/crates/dashboard-v2/src/lib/drawers/OrchestrationDrawer.svelte` (+~30 LOC; mount IdeaFramerSecurityPanel)

## Citations / Grounding
- Veracode 2025 GenAI Code Security report — 45% AI code fails basic security; grounded in report TL;DR
- CSA / Tenzai Dec 2025 — 100% CSRF / headers missing across 15 vibe-coded apps; report failure-mode #1
- Help Net Security 2025 (slopsquatting paper) — 19.7% package-name hallucination rate; report failure-mode #3
- arXiv:2510.10460 Planner-Coder Gap — per-level critic rationale; cross-references B-40 citations
- arXiv:2502.12115 SWE-Lancer — E2E test rubric; cited by report best-technique #10
- Aider polyglot leaderboard (aider.chat/docs/leaderboards) — edit-format matters more than model strength
- Answer.AI Devin retrospective Jan 2025 — anti-spiral hard-abort grounding (Devin's days-long impossible-task chase)
- Escape.tech 1,400-app scan via Baytech 2025 — 400+ exposed secrets, secret-leak prevalence
- Cognition Devin 2025 review — "junior at execution, can't tackle ambiguity end-to-end" → scope_ceiling table

## Open questions / risks
- Ungrounded: 0.6 scope_clarity threshold is a starting heuristic; the report's open-question #2 calls for an empirical baseline run. Plan to measure on the first 20 idea-framing sessions and re-tune.
- The 15-item production-grade checklist depends on Tier-C subprocess availability (B-18 / C-11). If Tier C is not active, the checklist degrades to a warning-only mode and emits `production_grade_degraded_unavailable_sandbox`.
- Slopsquatting verifier introduces a network dependency in the executor path. Air-gapped mode (B-27 [execution] air_gapped=true) must fall to "verify against last-known cache only; block if not cached".
- The Aider edit-format change is a per-role default; cloud Sonnet 4.6 may not respect the unified-diff prompt without an explicit system prompt addition. Source-grounding via paper-ingestor mode=source on the Aider unified-diff prompt format is required before EXECUTING.
- The anti-spiral hard-abort wall-clock budget is a HEURISTIC; if the user explicitly requests long-running orchestrate (per .claude/skills/orchestrate), the budget must be overridable per session via /budget CLI command.

## Next
- After E-11 ships, schedule a measurement pass: run 20 idea-framing sessions on a controlled brief set, measure scope_clarity correlation with downstream auditor BLOCK rate (report open Q1/Q2).
- E-12 (production fullstack codegen) plugs into the new production_grade auditor mode and extends the 15-item checklist with framework-specific entries.
- E-13 (TDD) wires mutation-survival into per_level_critic_verdict so the Step-level critic has a deterministic signal in addition to LLM judgment.
