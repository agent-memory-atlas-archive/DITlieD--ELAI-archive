CENTRAL WORKFLOW HARNESS AUDIT
Purpose: inform the merge of the ELAI harness (F:\.ELAI_workflow\.claude) and the liteGAME harness (D:\.liteGAME_workflow\.claude) into ONE shared core + opt-in capability packs on a frozen local Claude Code CLI.
Method: read-only audit of both .claude trees + both .mcp.json. Decomposition pipeline agents diffed byte-for-byte; orchestration skills full-read; everything else frontmatter/first-body-line + size/diff.
Date: 2026-06-19.

LEGEND
CORE = belongs in the shared core. PACK:rust / PACK:unity / PACK:media / PACK:elai = opt-in capability pack. CUT = delete/retire candidate.
"Both-DIFFER" means same lineage, divergent body (usually project paths/gates). "Both-SAME" means byte-identical.


==========================================================================
1. SKILLS TABLE
==========================================================================
name | exists-in | one-line purpose | CLASS | better lineage / merge note
--------------------------------------------------------------------------
orchestrate (ELAI) / orchestrate-cross-project-plan (liteGAME) | both (renamed) | cross-project bootstrap: launch `claude -p` at target CWD, delegate to target native orchestration, phase-end sweep, 1 commit | CORE | NEAR-IDENTICAL mechanism. ELAI copy is richer (H-1..H-10 hard-rule block, D-030 production-caller injection, pod-cost tripwire). Take ELAI body, take liteGAME's clearer NAME (orchestrate-cross-project-plan) to disambiguate from auto/local. See section 5.
auto-orchestrate (ELAI) | ELAI only | general autonomous orchestration of ANY task (review/wiring/test/impl/polish), main-LLM-as-strategist, run-state + handoff mediation, off-agent verify | CORE | This is the target general orchestrator. Best foundation for the reworked one. Merge in liteGAME local-plan's hard COMMIT GATE + maintainer-voice executor brief. See section 5.
autonomous-orchestrate-plan (liteGAME) | liteGAME only | same-project in-session plan-file executor (B-plan + TEMPLATE shape), FSM cycle per task, hard commit gate | CORE | Best in-session plan walker. Its dual-shape parser + COMMIT GATE (surface-files-then-wait) are superior to anything ELAI has. Fold its commit gate into the merged orchestrator.
b-plan-runner (ELAI) | ELAI only | full FSM cycle for a numbered ELAI B-plan (context>scope>explore>synth>spec-ground>prompt>impact>exec>audit>fix>wiring-validate>memory>commit) | PACK:elai | Deeply ELAI-specific (B-plan paths, wiring-contract-validator, smoke-manifest, spec-grounder, SSB). The richest FSM sequence; lift its STRUCTURE into core orchestrator, leave the ELAI gate steps in the rust/elai pack.
paper-to-plan (ELAI) | ELAI only | drop paper/repo links > authored ELAI plan files in one session (ingest>map>steal-analysis>author) | PACK:elai | Plan-authoring is ELAI-corpus specific (ELAI-FEATURE-MAP, docs/thefinalglance). Generalize the ingest+steal pipeline to core; keep ELAI plan-authoring in the pack.
idea-intake | both-DIFFER (E=432 L=473) | recursive idea > Concepts > Phases > Steps > BabySteps decomposition cascade with per-level research+audit | CORE | Same lineage. Differences are project nouns. Merge to one; parameterize project paths.
planning-workflow | both-DIFFER (E=180 L=189) | plan creation, FSM states, plan-improvement protocol, phase completion, TDD | CORE | Same lineage. Reconcile to one; FSM + plan-improvement are shared.
prompt-transform | both-DIFFER (E=23 L=17) | reformat user prompt before implementation; sets GRC budget | CORE | Same lineage, trivial divergence. One copy.
rag-tools | both-DIFFER (E=29 L=24) | rules for local-rag MCP usage | CORE | One copy. Becomes more central as memory moves to indexed RAG.
oracle | both-DIFFER (E=136 L=131) | interactive grounded Q&A via researcher agent in oracle mode | CORE | One copy.
grill | both-DIFFER (E=107 L=90) | one-question-at-a-time interrogation to resolve decisions before planning | CORE | One copy. ELAI version references codegraph-elai/elai.map (parameterize).
pdf | both-SAME | PDF/doc > markdown/text via MinerU | CORE (utility) | Identical. One copy.
anti-spiral | ELAI only | stagnation detector + 5-persona switch | CORE | General; depends on persona-* agents (also core). Keep.
cook | ELAI only | design-discovery front-end (FRAME>DIVERGE>CONVERGE>EMIT brief) via gpt_image | PACK:media or CORE-design | UI/design discovery. Useful to any UI project; depends on gpt-bridge. Core-design or media pack.
spec-eval | ELAI only | judge a design AS-SPECIFIED (planned counts as built) for comparisons/improvements | CORE | General reasoning skill, not ELAI-bound despite examples. Keep, deproject the examples.
sidecar-protocol | ELAI only | remote sidecar boot/health/inference 3-step protocol | PACK:elai | ELAI sidecar infra only.
dashboard-handoff | ELAI only | post-edit dashboard-v2 (Svelte/Tauri) verification via dashboard-smoker | PACK:elai (or PACK:svelte) | ELAI dashboard-v2 specific.
impact-check | ELAI only | pre-edit blast-radius for .elai_cc edits via impact-mapper | PACK:rust/elai | Generalizable concept but wired to .elai_cc paths + elai.map.
unity-scaffold | liteGAME only | scaffold Unity feature module / asmdef quartet | PACK:unity | Pure Unity.
region-maps | liteGAME only | solid region maps for cel-shaded sprite art via color-key segmenter | PACK:media (game-art) | Pure game-art tooling.
banana-pro-director-2.0 | liteGAME only | Higgsfield image-prompt director (character/scene plates) | PACK:media | Pure media-gen.
cinema-worldbuilder-pro-2.0 | liteGAME only | Seedance video-prompt cinematography director | PACK:media | Pure media-gen.
strip-ai-footprint | liteGAME only | scrub AI-generator metadata/fingerprints from images | PACK:media (utility) | Standalone media utility.
summarize | liteGAME only | summarize url/video/pdf/audio via @steipete/summarize CLI | CORE (utility) or CUT | Overlaps pdf + general capability; small. Keep as core utility OR cut (low use).

SKILL COUNT: 28 distinct (15 ELAI-side incl shared, 15 liteGAME-side incl shared, 7 truly shared lineage). CORE ~13, PACK ~12 (media 5, unity 2, elai 5), CUT-candidate 1-2.


==========================================================================
2. AGENTS TABLE
==========================================================================
KEY FINDING: the entire DECOMPOSITION PIPELINE is byte-identical across both harnesses (researcher, idea-framer, idea-critic, concept-decomposer, phase-decomposer, step-decomposer, paper-ingestor, synthesizer all SAME). That cluster is the clean shared core. ELAI additionally carries -elai VARIANTS of three agents (it has BOTH generic and -elai) plus a set of ELAI-only specialist agents.

name | exists-in | model | role one-line | CLASS | merge note
--------------------------------------------------------------------------
DECOMPOSITION PIPELINE (treat as one unit) — all model=sonnet:
 idea-framer | both-SAME | top of cascade, frames the raw idea | CORE | identical, one copy
 idea-critic | both-SAME | adversarial review of framed idea (framer<>critic loop) | CORE | identical
 concept-decomposer | both-SAME | concept > phases | CORE | identical
 phase-decomposer | both-SAME | phase > steps | CORE | identical
 step-decomposer | both-SAME | step > baby steps | CORE | identical
 babystep-author | both-DIFFER (E=312 L=265) | writes 10-block baby-step detail file | CORE | reconcile; ELAI bigger (extra gates)
 researcher | both-SAME | tree-structured grounded research w/ floors; also oracle-mode | CORE | identical
 synthesizer | both-SAME | merges exploration into WIRING section | CORE | identical (ELAI also has synthesizer-elai variant)
 paper-ingestor | both-SAME | ingest paper/source > interface map | CORE | identical

PLANNING/EXEC CORE:
 context-compiler | both-DIFFER (E=189 L=203) | compiles per-task context package | CORE | reconcile; deproject paths
 scope-planner / scope-planner-elai | both-DIFFER + ELAI dup | decide EXPLORE vs SKIP, partition | CORE (+ elai variant) | collapse generic+elai into one parameterized agent
 integration-explorer / integration-explorer-elai | both-DIFFER + ELAI dup | explore a subsystem partition for wiring | CORE (+ elai variant) | collapse the duplicate; ELAI variant = ELAI nouns only
 prompt-engineer | both-DIFFER (E=153 L=156) | reshape task into structured prompt | CORE | reconcile
 memory-writer | both-DIFFER (E=69 L=73), model=haiku | write memory-bank via RAG mcp tools | CORE | reconcile; becomes RAG-writer when memory moves to indexed RAG
 auditor | both-DIFFER (E=316 L=207) | hostile senior auditor, sole test runner, BLOCK/WARN/DEAD-END | CORE | ELAI copy is 100 lines bigger = wiring/freeze/production-caller gates. Split: generic audit body = core; rust-wiring gate steps = PACK:rust
 designer | both-SAME (28 lines) | RETIRED stub | CUT | delete from both
 synthesizer-elai | ELAI only | ELAI-noun synthesizer | CUT (merge into synthesizer) | redundant with generic

ANTI-SPIRAL PERSONAS (one unit, ELAI only, all sonnet):
 persona-hacker / persona-researcher / persona-simplifier / persona-architect / persona-contrarian | ELAI only | lateral-thinking persona switch for anti-spiral | CORE | general; move with anti-spiral skill to core

ELAI SPECIALISTS:
 fixer | ELAI only, sonnet | single EXECUTING pass to clear auditor BLOCKs | CORE | general-purpose, keep in core
 impact-mapper | ELAI only, sonnet | pre-edit blast-radius (.rs/.py/.ts/.svelte) | CORE (rust-leaning) | concept is general; paths/elai.map are pack-bound
 adversary | ELAI only, sonnet | cross-family adversarial critic (D-13 grounding) | PACK:elai | tied to ELAI FalsifiabilityCrossFamily/SSB
 spec-grounder | ELAI only, sonnet | verify Spec references map to docs/specs | PACK:elai | ELAI spec corpus
 wiring-contract-validator | ELAI only, sonnet | thin runner over wiring_contract_check.py | PACK:rust | ELAI wiring gate
 test-attributer | ELAI only, sonnet | reclassify build/test failures | PACK:rust | cargo-output specific
 dashboard-smoker | ELAI only, sonnet | svelte-check + cargo check dashboard smoke | PACK:elai/svelte | dashboard-v2
 sidecar-health | ELAI only, sonnet | remote sidecar health check | PACK:elai | sidecar infra

AGENT COUNT: ELAI 31, liteGAME 16. Truly shared lineage: 16 (9 of them byte-identical). CORE ~20 (incl personas+pipeline), PACK ~9 (rust 3, elai 5, media 0), CUT 2 (designer, synthesizer-elai dup).


==========================================================================
3. RULES + RULEBOOK TABLE
==========================================================================
RULES (.claude/rules):
name | exists-in | purpose | CLASS | conflict
--------------------------------------------------------------------------
wiring-enforcement | both-DIFFER | 10 fail-closed rules; producer/consumer/runtime-evidence gates | CORE (doctrine) + PACK:rust (rust legs) | reconcile; rust-specific legs (cargo, SCIP, production_caller_check.py) belong in rust pack
maintainability-review | both-DIFFER (E=97 L=99) | 8-dim maintainability catalog + checker | CORE | reconcile to one
commit-cadence | both-DIFFER | one-commit-per-task + maintainer-voice messages | CORE | reconcile; liteGAME adds VCS abstraction (cm/git)
output-formatting | both-DIFFER (E=7 L=67) | output rules + maintainer voice + in-code-comment bans | CORE | CONFLICT: ELAI copy is a 7-line STUB; liteGAME copy is full (the maintainer-voice + comment-ban section the orchestrators cite). Use liteGAME copy. See section 7.
token-discipline | both-DIFFER (E=111 L=87) | token economy, anti-read-big, zero-lazy-code, anti-slopsquat | CORE | reconcile; ELAI bigger
main-llm-role | both-DIFFER (47=47) | orchestrate-not-implement boundary, anti-sycophancy | CORE | near-identical, reconcile
visual-ui-loop | both-DIFFER (E=46 L=36) | screenshot-diff-critique loop, schema-constrained | CORE-design or PACK | ELAI references CDP/dashboard-v2; generalize
bun-not-node | both-SAME | default JS runtime = bun, no new node/npx | CORE | identical
conditional-routing | both-DIFFER (53=53) | the lazy-load dispatch table for rules/rulebook | CORE | reconcile (it indexes the merged set)
tracing-test-assert | ELAI only | mandatory assert on logs_contain in #[traced_test] | PACK:rust | Rust tracing-test crate specific
production-caller-gate | ELAI only | D-030 changeset-grain production-caller checker | PACK:rust (+ py sibling) | wiring pack
agent-dispatch-preamble | ELAI only | binding worker rules injected into every dispatch (git read-only, scope, BLOCKED=handoff) | CORE | CRITICAL core safety rule; born of liteGAME git-reset-x8 incident. Must be core.
orchestrate-protocol | ELAI only (rules) | H-1..H-10 hard blocking orchestrate rules | CORE | both also have a rulebook/orchestrate-protocol.md (SAME). Reconcile rules vs rulebook duplication.
handoff-protocol | ELAI rules / liteGAME rulebook | one-file-per-problem topic-keyed handoff | CORE | CONFLICT: lives in different dirs across harnesses (both SAME content via rulebook). Pick rules/, dedupe.
skill-import-policy | ELAI only | scan 3rd-party skills (skillspector) before import | CORE | security; keep core
verify-before-write | liteGAME only | verify Unity C# symbols exist before writing | PACK:unity | Unity symbol blacklist
bug-fix-protocol | liteGAME only | bug-framed task protocol (reproduce-first) | CORE | general; adopt into core
doubt-driven-review | liteGAME only | force a doubt pass before letting an uncertain decision stand | CORE | general; adopt into core
shell-dialect | liteGAME only | Bash vs PowerShell tool dialect gotchas (Windows) | CORE | general Windows hygiene; adopt into core

RULEBOOK (.claude/rulebook):
SHARED-SAME: bubble-up, canvas-image-regen, multi-session, orchestrate-protocol, risk-levels, source-tiers, stub-tracking | CORE
SHARED-DIFFER: agent-spawning, coding-standards, design-taste, file-discipline, folder-ownership, gpt-image-prompting, gpt-second-opinion, integration-exploration, memorizing-and-audit, session-protocol | CORE (reconcile each; differences are project nouns / stack)
ELAI-ONLY: codegraph-vs-map (codegraph MCP vs elai.map) | PACK:rust/elai
liteGAME-ONLY packs:
 unity-* (anti-stale-unity-csharp, unity-architecture, unity-asset-hygiene, unity-map-usage, unity-mcp-usage, unity-mcp-and-builders, unity-ai-platform-decisions) | PACK:unity
 media (embervow-pipeline-quickref, remote-comfy, hyperframes-video-design, image-inspection, gpt-image already shared) | PACK:media
 android-cli | PACK:android (or CUT if unused)
 notion-ticket-format | PACK:notion (or CUT)
 performance-testing, runtime-debugging, ui-responsive-practices | PACK:unity (game UI/perf)
 extensible-systems, experimentation-hygiene | CORE (general engineering doctrine) — adopt
 project-config | CORE (per-project config template) — the pack-config seam
 handoff-protocol (rulebook copy) | dedupe vs rules

RULE COUNT: rules 15 ELAI / 13 liteGAME (9 shared). rulebook 18 ELAI / 37 liteGAME (17 shared). CORE rules ~16, PACK rules ~3. CORE rulebook ~22, PACK rulebook ~18 (unity ~10, media ~4, other ~4).


==========================================================================
4. MCP TABLE
==========================================================================
name | in | purpose | CLASS | install
--------------------------------------------------------------------------
local-rag | both | RAG search + FSM state machine + session registry + mcp_think + memorize | CORE | shared single install at C:\Users\user\.claude\rag (machine-global). The backbone; becomes more central as memory > indexed RAG.
canvas | both | localhost code-graph workbench off dependency-cruiser | CORE (opt) | per-root copy (port per root: ELAI 7323, liteGAME 7322). Server hard-anchors its own dir; cannot be shared.
gpt-bridge | both | cross-model second opinion + image + deep research | CORE | SHARED single install at D:\.apps_workflow (self-anchors). Register once.
pod-lifecycle | both | RunPod pod power/cost/migration | CORE (opt, infra) | shared infra, registered in all roots. liteGAME copy doc is fuller.
codegraph-elai | ELAI only | live C# is no — Rust/TS call-graph pinned to .elai_cc | PACK:rust/elai | per-root, -p pins scope
codegraph-zc / codegraph-eo | liteGAME only | C# call-graph pinned per game Assets/ | PACK:unity | per-game instances (Zombie_Citizen, Everlasting Odyssey)
unityMCP | liteGAME only | CoplayDev Unity Editor bridge | PACK:unity | per-root
unityMCP-runtime | liteGAME only | Murzak runtime AiTools bridge (INERT) | PACK:unity | disabled until dev build exists
ai-game-developer | liteGAME only | Murzak editor plugin bridge (caveated dup of unityMCP) | PACK:unity / CUT-candidate | overlaps unityMCP; the config itself says prefer unityMCP
comfy-pipeline | liteGAME only | ComfyUI rule predicates | PACK:media | per-root
comfyui-cwags | liteGAME only | artokun ComfyUI generation over SSH tunnel | PACK:media | npx (rewrite to bunx per bun-not-node)
blender-bridge | liteGAME only | Blender MCP | PACK:media | per-root
ae-mcp-dakkshin / ae-mcp-ishu86 / ae-mcp-rs | liteGAME only | three After Effects MCPs (all INERT, no AE installed) | PACK:media / CUT | pick ONE if AE ever lands; the other two are dead weight
unrealclaude / vibeue | liteGAME only | two Unreal Engine MCPs (both INERT, no UE installed) | PACK:unreal / CUT | no UE + no UE project here; strong CUT candidates

MCP COUNT: ELAI 5, liteGAME 16. CORE 4 (local-rag, gpt-bridge, pod-lifecycle, canvas). PACK media ~7, unity ~5, unreal 2. Several INERT (5 AE+UE) = cut unless the toolchain arrives.


==========================================================================
5. ORCHESTRATION DEEP-DIVE (most important)
==========================================================================
Five orchestration skills exist across the two harnesses. They cluster into THREE mechanisms:

MECH-A: CROSS-PROJECT BOOTSTRAP
  ELAI orchestrate  ==  liteGAME orchestrate-cross-project-plan (renamed, ~95% identical body)
  Mechanism: parse plan_path + depends_on; resolve target via walk-up for .claude/; load compat/<target>.md + stack-adapter; register session; dep-wait poll (mcp_session_list, 60s, never touch dep FSM); fill overrides-template.md; launch `claude -p --output-format stream-json --resume <sid>` at target CWD so target's NATIVE skills/agents load; tail stream-json (init/api_retry/tool_use/result); phase-end-sweep.ps1 (6 generic + stack gates); ONE commit at end.
  Gating: 4 tripwires (irreversible op, 50-event bubble cap, dep-wait>4h, sweep-fail x3). Otherwise fully autonomous.
  ELAI extras over liteGAME: the H-1..H-10 hard-rule block (dispatch is blocking, no Edit before parallel batch, no audit/memory skip); D-030 production-caller injection into dispatch; pod-cost tripwire; feature-map upsert step.
  Verdict: ONE skill. Keep ELAI body (richer), keep liteGAME's clearer name. The bootstrap-claude-p-at-target-CWD design is the right way to make orchestration general across projects — the target supplies its own pack.

MECH-B: SAME-PROJECT IN-SESSION PLAN WALK
  liteGAME autonomous-orchestrate-plan (no ELAI peer; ELAI's b-plan-runner is the ELAI-specific equivalent)
  Mechanism: main LLM IS the orchestrator (no sub-claude). Dual-shape parser (B-plan H1 shape vs TEMPLATE frontmatter shape) > unified internal model > per-task FSM cycle: context-compiler+integration-explorer parallel (cap 3) > skip prompt-engineer > executor > auditor (1 cycle, no re-audit) > memory-writer. Phase-end gates. THEN the standout feature: a HARD COMMIT GATE — list every changed file grouped by category, diff stats, sensitive-file scan, maintainer-voice draft, surface-and-WAIT for explicit commit verb (never infer from silence). Same 4 tripwires.
  ELAI b-plan-runner is the same family but ELAI-bound: 13 numbered steps with scope-planner-elai, parallel explorers+synthesizer, spec-grounder, impact-mapper, wiring-contract-validator + gate_freeze_check + smoke-manifest + wire-dark-allowlist, test-attributer, feature-map upsert. The most thorough FSM sequence in either harness.
  Verdict: the GENERIC in-session walker should be MECH-B (liteGAME). Lift its dual-shape parser + COMMIT GATE into core. b-plan-runner's extra gate STEPS (wiring/freeze/smoke/spec) become a PACK:rust overlay the core walker calls when the rust pack is active.

MECH-C: GENERAL TASK ORCHESTRATION (not plan-file)
  ELAI auto-orchestrate (no liteGAME peer)
  Mechanism: main LLM = pure strategist (never executor). run-state.md global state (objective, task graph w/ deps+status+agent+model, decisions log, budget, verification) updated after every return. ONE topic-keyed handoff file. Dispatch rules: rule-injection preamble on every prompt, scoped context per worker, model ladder (haiku bulk / sonnet impl+audit / main never spawned), parallel cap 3, fleet announcement+consent if >2 agents, execution/evaluation separation. Handoff mediation loop (worker BLOCKED > appends entry > orchestrator decides + appends DECISION > next worker pointed at it). Autonomy envelope w/ loop prevention (30-dispatch check-in cap, re-dispatch cap 2, monotonic-progress check). Verification-before-DONE off-worker. Integration gate R10 (vertical slices, threaded contracts, off-agent wire-check, independent integration smoke). One commit at end.
  Verdict: THIS is the seed for the reworked general orchestrator. It already targets "any task (review/wiring/test/impl/polish)" and is project-agnostic in design (the ELAI-specific bits are only the example paths + feature-map upsert).

RECOMMENDATION FOR THE SINGLE REWORKED GENERAL ORCHESTRATOR:
Base it on auto-orchestrate (MECH-C): strategist-never-executor, run-state task graph, handoff-mediation loop, off-agent verification, R10 integration gate, model ladder + parallel cap 3, agent-dispatch-preamble injection. From autonomous-orchestrate-plan (MECH-B) adopt the dual-shape plan parser and the HARD COMMIT GATE (surface-files-grouped + sensitive scan + wait-for-explicit-verb) — that is the best commit safety in either harness and belongs in every mode. From the cross-project skill (MECH-A) keep the `claude -p` bootstrap-at-target-CWD path as an OPTIONAL mode for when the work lives in another root with its own pack, plus dep-wait-without-touching-dep-FSM. Make all stack/gate-specific steps (b-plan-runner's wiring/freeze/smoke, Unity console-clean gate) PACK OVERLAYS the core orchestrator invokes by capability detection, not hardcoded steps. DROP: maintaining three separate orchestrate skills; collapse to one general orchestrator with three modes (general-task, in-session-plan, cross-project-bootstrap) selected by input shape. Universal invariants across all modes: agent-dispatch-preamble on every dispatch, parallel cap 3, off-agent verification, one-commit-at-end behind the hard commit gate, the 4 tripwires.


==========================================================================
6. CUT LIST
==========================================================================
- designer.md (both): RETIRED stub already. Delete from both.
- synthesizer-elai.md (ELAI): redundant duplicate of generic synthesizer (which is byte-identical across harnesses). Collapse to one parameterized synthesizer.
- ELAI generic-vs-elai agent duplication: scope-planner + scope-planner-elai, integration-explorer + integration-explorer-elai are kept as separate files where only project nouns differ. Collapse each pair to ONE agent that takes the project as a parameter. (3 files removed.)
- ae-mcp-ishu86 + ae-mcp-rs (liteGAME): three After Effects MCPs all INERT (no AE installed). Keep at most one if AE ever lands; cut the other two now.
- unrealclaude + vibeue (liteGAME): two Unreal MCPs, both INERT, no UE + no UE project. Strong cut unless an Unreal project is planned.
- ai-game-developer MCP (liteGAME): its own config says it overlaps CoplayDev unityMCP and to prefer unityMCP. Cut or leave disabled.
- summarize skill (liteGAME): overlaps pdf + general model capability; low-value. Cut-candidate (verify usage first).
- android-cli + notion-ticket-format rulebook (liteGAME): project-niche; pack-or-cut depending on whether Android/Notion are still in play.
- output-formatting.md (ELAI): 7-line stub superseded by liteGAME's full version — cut the stub, adopt liteGAME's.
- rules/handoff-protocol vs rulebook/handoff-protocol + rules/orchestrate-protocol vs rulebook/orchestrate-protocol: duplicated across rules/ and rulebook/. Dedupe to one location each.


==========================================================================
7. CONFLICTS (same-named, divergent) + RESOLUTION
==========================================================================
C1 output-formatting.md: ELAI=7-line stub, liteGAME=67-line full (maintainer voice + in-code-comment bans). The orchestrator executor briefs CITE the "In-code comments" section that exists ONLY in liteGAME's copy — ELAI dispatches reference a section that is absent in ELAI. RESOLVE: adopt liteGAME's full version as core; deproject the maintainer-voice naming if generalizing.
C2 handoff-protocol location: ELAI keeps it in rules/, liteGAME in rulebook/ (content same). RESOLVE: canonical location = rules/ (it is a HARD RULE). Remove the rulebook copy.
C3 orchestrate-protocol duplication: present in BOTH rules/ AND rulebook/ in each harness. RESOLVE: keep the rules/ copy (H-1..H-10 hard rule); rulebook copy is a pointer at most.
C4 auditor.md: ELAI=316 lines (wiring/freeze/production-caller/SSB gates), liteGAME=207 (Unity console-clean gate). RESOLVE: extract the shared hostile-audit core; push rust-wiring gate steps to PACK:rust and Unity gate steps to PACK:unity. The auditor should call pack-provided gate checklists, not hardcode them.
C5 wiring-enforcement.md: both-DIFFER. The 10-rule DOCTRINE is shared and portable (the file itself claims project-agnostic). The runnable LEGS differ by stack (cargo/SCIP/production_caller_check.py vs none). RESOLVE: doctrine in core; language checkers in packs.
C6 commit-cadence.md: liteGAME abstracts VCS (cm for Plastic + git), ELAI assumes git. RESOLVE: core keeps the VCS abstraction (liteGAME's) so packs can declare their VCS.
C7 conditional-routing.md: both index their own rule/rulebook set. RESOLVE: rewrite once over the merged core+active-pack set; pack activation extends the routing table.
C8 -elai naming convention: ELAI fork of three agents purely to swap project nouns. RESOLVE (see CUT): one agent + project parameter; the -elai suffix convention disappears.


==========================================================================
8. OPEN QUESTIONS (owner must decide)
==========================================================================
Q1 Pack granularity: is it PACK:media (one big media pack = comfy+blender+higgsfield+seedance+region-maps+strip-footprint) or split into media-gen vs game-art? The liteGAME media tooling is large and internally coupled (embervow pipeline).
Q2 Memory migration: memory-writer + output-formatting + memorizing-and-audit + the changelog/activeContext/progress/projectIndex file set all assume changelog-file memory. When memory moves to indexed RAG, do these become RAG-write agents/rules, or does the whole memory-bank-file doctrine get retired? This touches ~5 core artifacts.
Q3 FSM ownership: the FSM state machine (mcp_fsm_* in local-rag) is woven through every orchestration + planning skill + many agents. Is the FSM core, or does the reworked orchestration drop the heavyweight FSM for the lighter run-state.md model auto-orchestrate already uses? The two models currently coexist and partly overlap.
Q4 Which Unreal/AE/Android/Notion toolchains are actually staying? Five INERT MCPs (3 AE + 2 UE) and 2 niche rulebooks (android, notion) — confirm before packaging vs cutting.
Q5 spec-eval, cook, visual-ui-loop, design-taste: these are UI/design-reasoning, ELAI-authored but stack-general. Core, or a dedicated design pack? They depend on gpt-bridge (core) + canvas (core-opt).
Q6 codegraph: ELAI uses one pinned instance, liteGAME uses per-game instances + the codegraph-vs-map reconciliation doctrine. The core orchestrator's impact analysis wants codegraph — is codegraph core infra (per-project pinned) or a rust/unity pack tool? It is stack-agnostic (C#/Rust/TS) so leans core.
Q7 The b-plan-runner FSM sequence is the most complete in either harness but is ELAI-bound. Should its full 13-step sequence become the canonical in-session template (with pack-gated steps) or is the lighter liteGAME autonomous-orchestrate-plan cycle the canonical one with b-plan-runner kept only as the ELAI pack overlay?
