# L-OQF Grounding Map — Output-Quality + Familiarity-Surface track (L-T2, L-11..L-14)

Authored 2026-06-27, main LLM in-band (spec-eval AS-SPECIFIED + paper-to-plan PHASE 5). LIVE baseline ~SCHEMA 268 / TV 577 (Stage-5 stamp 5b0a66d 2026-06-15; re-grep at every edit per W-7). This map is the W-7 source: TS = substrate shipped/planned (COMPOSE-not-restate, GD-1); NN = net-new builds; DC = feature-map corrections; WV = external verification.
Anchors carried from ELAI-FEATURE-MAP.md (full read this session) + C-SUMMARY C-31/C-32 + J-SUMMARY J-23/J-43/J-65 + .claude/rules/maintainability-review.md (in CLAUDE.md). File-path/line anchors are AS-OF the map; every executor re-greps at edit (W-7) and records drift here.

## TS — substrate to COMPOSE (re-implementing any of these is a P0, GD-1)

TS-1  C-31 Code-Completeness L1 [PARTIAL]: crates/core/src/file_plan.rs — FunctionSpec/FileSpec(path,language,max_lines,functions)/FileModSpec/FilePlan + validate() against LanguageLimitsMap, MAX_FUNCTIONS_PER_FILE=20; FILE_PLAN is the 9th BabyStepSpec field. MistakeRecorder unwired (the PARTIAL). [C-SUMMARY C-31] — SEAM 1 host for L-11.
TS-2  C-32 Code-Completeness L2 [WIRED]: skeleton_generator.rs (BlankId->BLANK_{N}, SkeletonGenerator::generate, fill_blanks runs MiniCritic per fill, flush_to_disk one fs::write) + mini_critic.rs (MiniCriticVerdict PASS/WARN/BLOCK; order: T1 regex subset reusing tiered_analysis.rs LazyLock statics, line<3 BLOCK, tree-sitter fragment ERROR-node BLOCK; emits mini_critic_verdict SSB) + structural_retry.rs (Decompose/Template/FewShot/BestOfN, AcceptAndComplete at ratio>=0.5) + bridge_template.rs; wired into task_runner baby-step dispatch when FilePlan present. 5 SSB: skeleton_generated/skeleton_flushed/mini_critic_verdict/structural_retry_stage_attempt/bridge_file_generated. [C-SUMMARY C-32] — SEAM 2 host for L-11.
TS-3  maintainability_check.py + .claude/rules/maintainability-review.md [WIRED]: MR-1 file<1000 (cross=BLOCK), MR-2 as-any/unsafe (BLOCK) + : any/allow/cast (WARN), MR-3 fn>80 (WARN), MR-4 net-branching>10 (WARN); 8 dimensions (0 structural-simplification, 5 type/boundary, 6 canonical-layering = SOLID-adjacent); ROLE BOUNDARY = AUDITOR-owned, never executor-self; frozen in gate-manifest (R8). [CLAUDE.md] — SEAM 3 floor + SEAM 4 sibling for L-11.
TS-4  J-65 power-of-ten gate [PLANNED, observe-only]: .claude/tools/power_of_ten_check.py over GENERATED code (acyclic call graph, bounded loops, assertion density>=2, every-return-checked, zero-warnings-BLOCK, cfg<=2, fn<=60 on flagged modules); auditor-owned, sibling of maintainability_check.py. [feature-map 1.6] — L-11 composes; solid_proxy_check.py is its SOLID sibling.
TS-5  J-23 multi-language review-gate catalog [PLANNED]: sarif_ingest (Semgrep CE/PMD-CPD/Psalm/detekt) + arch_gate TOML (dependency-cruiser/Knip/Deptrac/Periphery) + a11y/token-drift gate; gates GENERATED multi-lang code first. +ReviewGateCatalogEvent. [feature-map 1.8] — L-11 SEAM 4 generated-code lane.
TS-6  B-29 symbol-level impact API [WIRED]: impact_radius/call_flow/symbol_context/list, fan-in/fan-out. [feature-map 1.1] — the SRP fan-out + DIP data source for solid_proxy_check.py (R5 sound-reference-not-grep).
TS-7  elai.map + codebase_mapper.py [WIRED, .claude]: TAG/SYMBOL/FILE:LINE/LANG/EXTRA/SNIPPET incl DEF/CALL/USE/IMPL_TRAIT/TYPE_REF/SCIP_REF. [CLAUDE.md] — the DIP (cross-layer concrete TYPE_REF vs IMPL_TRAIT) + ISP (trait method count) data source.
TS-8  RULES ENGINE phase-3F [WIRED]: rules-as-DATA in SQLite, predicates AST/GREP/SCHEMA/METRIC/COMPOSITE, verdicts ALLOW/WARN/BLOCK/ESCALATE, lifecycle PROPOSED->CANDIDATE->SHADOW->ACTIVE->RETIRED (fpr<5%+tpr>80%+causal>50%), mutable=0 immune tier. [feature-map 1.7] — the SHADOW->ACTIVE promotion path for L-11 quality rules AND the user-authorable rule substrate for L-13.
TS-9  VibeGate F-1 [WIRED]: fail-closed multi-axis PreToolUse (code-vuln AST/GREP + secret GREP Tier-1; dep-existence + skill-scan Tier-2), evaluate_vibegate in production PreToolUse chain, mutable=0 "vibesec". [feature-map 1.8] — SEAM 3 write-time host for the MR floor.
TS-10 I-38 QDELTA + statistical floor [PLANNED]: quality sibling of G-6 SAFEDELTA, >=5 seeds paired pass^k. [feature-map 1.13] — the L-11 measured quality number (compose, do NOT fork the runner).
TS-11 J-55 modality router [PLANNED]: ModalityRouter::resolve route weak sub-14B local lane to constrained-call + minimal schemas, NEVER CodeAct (Qwen-7B guard). [feature-map 1.9] — L-11 SEAM 2 pairs with it so the weak lane emits gate-passable code by construction.
TS-12 J-89 L1 spec-jury [PLANNED]: cross-family advisory the-ONE-LLM-entry at PLANNING-time. [feature-map 1.6] — the OCP/LSP advisory home (deterministic checker cannot cover them).
TS-13 E-43 AGENT SKILL SYSTEM [DARK(ELAI_SKILL_SYSTEM_ENABLE)]: agentskills.io-v1 validator + `elai skills-mgmt` + per-skill eval JSONLs + skill-import-policy (skillspector whole-dir scan); log_skill_invocation NO production caller (the unlit door). skill_system_event SSB. [feature-map 1.8] — L-12 ACTIVATES this.
TS-14 F-6 LEARNED-SKILL LIBRARY [WIRED]: SkillTrustTier S1-S4 gates autonomy, Voyager critic-gated acquisition, skill_permits_autonomous in production. [feature-map 1.8] — L-12 composes (the acquisition half).
TS-15 I-12 EVIDENCE-GATED PLAYBOOK CURATOR [PLANNED]: promote on >=k SSB-verified outcomes, export to CLAUDE.md/AGENTS.md/skills. [feature-map 1.5] — L-12 wires the episodic->skill->rule ladder.
TS-16 HOOK SYSTEM T1/T2/T3 [WIRED]: PreToolUse/PostToolUse/Stop/PreCompact intercept IPC in orchestrator (agents never see hooks), T1<1ms rules/T2<10ms tree-sitter/T3<5ms pattern NO LLM. NO SubagentStop axis. [feature-map 1.8] — L-13 host.
TS-17 J-52 SubagentStop evidence gate + per-edit LSP block-feed + PATH-KEYED DYNAMIC RULE INJECTION [PLANNED]: PostToolUse path-fingerprint hook injects the matching local rule into the context_budget Rules slot. [feature-map 1.8] — L-13 composes (the path-keyed injection is the user-rule delivery mechanism); J-62 reuses the same seam.
TS-18 J-27 harness protected-blocks + webcache + dispatch [PLANNED]: stdlib hooks shape. [feature-map 1.14] — L-13 hook-shape precedent.
TS-19 PIPELINE FACTORY L1/L2 [WIRED]: Idea->Concepts->Phases->Steps->BabySteps; deterministic L2 assembly NO LLM, AutoB2G DAG repair. [feature-map 1.12] — L-14 backbone.
TS-20 PRODUCTION FULLSTACK CODEGEN E-12 [WIRED 06-07] + web templates (vanilla/astro/vite-react/vite-svelte): 7 framework manifests + 15-item checklist. [feature-map 1.12 + .elai_cc/templates/web] — L-14 output.
TS-21 J-63 LOCAL-LANE CLAUDECEPTION `elai artifact run` [PLANNED]: keyless loopback sidecar broker for ELAI-generated apps, Tier-C isolation, trust_dialog-gated. [feature-map 1.9] — L-14 run-the-app seam (compose, do NOT re-author).
TS-22 I-40 TRUST-LAYER WRAPPER + RRA [WIRED]: `elai wrap` + `elai rra verify`, RunArtifactEmitted. [feature-map 1.11] — L-14 attaches the receipt to the one-shot output (FS-3).
TS-23 J-2 ELAI-AS-MCP control plane [PLANNED]: task_submit/status/fsm_get_state/... over stdio, same FSM gates. [feature-map 1.3] — L-14 composes (ELAI as an MCP server the user wires into their CC/Codex), do NOT re-author.
TS-24 J-102 config-ingress admission gate [PLANNED]: blake3 load-time manifest over the .md instruction surface (.claude/rules+agents+SKILL.md), FAIL-CLOSED quarantine. [feature-map 1.8] — the FS-2 guard for user-added skills/rules/hooks (compose).

## NN — net-new builds (the ~15% that is not activation)

NN-1 solid_proxy_check.py — a NEW frozen auditor checker (.claude/tools/), stdlib-only, consuming B-29 (TS-6) + elai.map (TS-7): SRP god-object (fan-out AND size) = the one deterministic BLOCK; DIP cross-layer concrete-vs-trait = WARN; ISP fat-trait = WARN; OCP/LSP NOT covered (advisory, TS-12). Sibling of maintainability_check.py/power_of_ten_check.py; auditor-owned; frozen R8; tags C-MAINTAINABILITY-SRP/-DIP/-ISP. L-11.
NN-2 the SOLID/god-object MiniCritic METRIC predicate — extend C-32 mini_critic.rs (TS-2) with a METRIC check (fan-out+method-count) so weak-model output retries in-loop via StructuralRetry; the any-model self-correction leg. L-11.
NN-3 the lit skill FRONT DOOR — give E-43 log_skill_invocation a production caller + a user-facing `elai skill add <dir>` / drawer flow that runs skillspector (TS-13) + J-102 admission (TS-24) before a SKILL.md is usable; the episodic->skill ladder consumer wired off I-12 (TS-15). L-12.
NN-4 user-authorable rule/hook registration — a `elai rule add` / hook-register surface that writes a rules-engine DATA row (TS-8) or registers a PostToolUse path-keyed rule (TS-17), evaluated by the existing T1/T2/T3 engine (TS-16), FS-2-gated (no arbitrary code; passes J-102). L-13.
NN-5 the one-shot verified builder ENTRYPOINT — `elai build "<idea>"` (or a dashboard one-field flow) over the pipeline factory (TS-19) + E-12 (TS-20) that ALWAYS attaches the I-40 RRA receipt (TS-22) and can `elai artifact run` it (TS-21); FS-3. L-14.

## DC — feature-map corrections (none material this run)
DC-1  C-32 is WIRED (gen-time loop live); C-31 PARTIAL (MistakeRecorder unwired) — both confirmed from C-SUMMARY, no correction needed; noted so L-11 SEAM 1 accounts for the C-31 PARTIAL residue.
DC-2  the prior-turn grep "234 SOLID hits" were ALL incidental (CSS/cohesion); the by-name-SOLID-absent finding stands (FINDING-2). Recorded so a future steal session does not mis-read the 234 as existing enforcement.

## OQ — output-quality delta family (binds L-11)
OQ-1 verdict-is-auditor-owned-never-self (in-boundary rubber-stamp ~100%, maintainability-review.md ROLE BOUNDARY).
OQ-2 deterministic-FLOOR (write-time, MR-1/MR-2 + god-object) vs advisory-CEILING (audit-time SOLID/OCP/LSP) split.
OQ-3 observe-first + measured-FP bound on >=3 runs before any BLOCK-by-default (GD-4 + J-43 rate).
OQ-4 mechanizable-proxy-only: SRP/DIP deterministic; ISP weak-WARN; OCP/LSP cross-family ADVISORY, never a deterministic block (honest residue, GD-5).
OQ-5 fix-at-generation-not-just-block-at-audit: the gen-time MiniCritic self-corrects (retries); the any-model quality lever (pairs J-55).

## FS — familiarity-surface delta family (binds L-12/L-13/L-14)
FS-1 user-extends-via-DATA-not-recompile (rules/skills as DATA on the WIRED 3F/E-43 substrate).
FS-2 every-user-surface-passes-the-SAME-trust-gates: a user skill/hook/rule cannot bypass VibeGate F-1 / skillspector / J-102 admission / wiring; it is DATA evaluated by the existing engine, never arbitrary code.
FS-3 trust-spine-attached-to-the-lazy-path: the one-shot output SHIPS an RRA receipt (I-40); the lazy product IS the trust product with a simple front end.
FS-4 format-compatibility: agentskills.io / CC SKILL.md + MCP (consume + serve) so users keep their existing CC/Codex workflow.

## WV — external verification (EXTERNAL/DESIGN-RATIONALE only, ID-5, no number is a done-gate)
WV-1 SOLID = R. C. Martin, "Design Principles and Design Patterns" (2000) [DS-1]. Reference-tier, no numeric claim; used as the principle taxonomy only. SRP/DIP mechanizable as fan-out/cross-layer proxies; OCP/LSP not deterministically checkable.
WV-2 Holzmann "Power of Ten" (NASA/JPL 2006) [DS-2]. Already carried + WebSearch-verified by J-65; reused as the complexity-floor rationale, no number is a done-gate.
(No arXiv ID introduced this run; both references are textbook/standards-tier design-rationale, not measured claims — no WebSearch floor triggered.)

End of L-OQF-GROUNDING-MAP.
