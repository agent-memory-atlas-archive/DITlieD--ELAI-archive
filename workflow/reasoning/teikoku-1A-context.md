CONTEXT PACKAGE — TEIKOKU P1-A (master-plan sub-phases 1.1 + 1.2 + 1.3, one integrated cycle)
task: build the merged TEIKOKU core+packs harness — core dir + packs/{rust,unity,media,elai} layout, projects.toml registry, execute the audit merge (skills/agents/rules/rulebook/MCP), prune the cut list, resolve §7 conflicts. READ-ONLY for this context agent; the EXECUTOR mutates the WSL distro.
task_id: TEIKOKU-P1-A-context
handoff_file: F:\.ELAI_workflow\workflow\handoff\teikoku-build.md
scope: /home/user/teikoku (WSL distro teikoku) — harness/, base-cc/, projects/, new core/ + packs/ + projects.toml. NEVER the live F: root except this one context file.
target-grain: the EXECUTOR runs in WSL at /home/user/teikoku; all target paths below are WSL paths.

================================================================================
SECTION 1 — THE CC-BUILD LOADER MECHANISM (load-bearing; how core+packs become loadable)
================================================================================
The frozen CC base is /home/user/teikoku/base-cc (package name teikoku-base-cc, @anthropic-ai/sdk 0.105.0). It is NOT modified in P1-A. How it discovers config decides the entire layout. All file:line below are VERIFIED reads of base-cc/src.

1A. SKILLS / AGENTS / COMMANDS / OUTPUT-STYLES / WORKFLOWS (the .claude subdirs CC enumerates)
- Canonical dir list: base-cc/src/utils/markdownConfigLoader.ts:29-36 CLAUDE_CONFIG_DIRECTORIES = ['commands','agents','output-styles','skills','workflows', (+ 'templates' under a feature flag)]. NOTE: there is NO 'rules' and NO 'rulebook' in this list — those are loaded by a DIFFERENT path (see 1C) / not at all (see 1D).
- Discovery walk: markdownConfigLoader.ts:234-289 getProjectDirsUpToHome(subdir, cwd) walks from cwd UPWARD, pushing every existing {dir}/.claude/{subdir}, and STOPS AT THE GIT ROOT (resolveStopBoundary, ts:191-220) or home. So skills/agents are discovered from every .claude between cwd and the git root inclusive.
- Three roots merged: loadMarkdownFilesForSubdir (ts:297-372) and getSkillDirCommands (loadSkillsDir.ts:638-714) load managed (getManagedFilePath()/.claude/<subdir>) + user (getClaudeConfigHomeDir()/<subdir> i.e. ~/.claude/skills, NO .claude segment for user) + projectDirs (the cwd-up-to-git-root walk) + --add-dir additionalDirs (loadSkillsDir.ts:699-708). Precedence managed>user>project; deduped by realpath/inode (loadSkillsDir.ts:118-124, markdownConfigLoader.ts:159-172).
- Symlinks ARE followed: loadMarkdownFiles uses ripGrep --files --hidden --follow --no-ignore --glob '*.md' (markdownConfigLoader.ts:564-568); native fallback follows symlinks too (ts:451-539). Inode dedup means the same physical file reached via two paths loads once.
- Skill format: ONLY directory form skill-name/SKILL.md (loadSkillsDir.ts:407-480, 424-428 — single .md in /skills/ is NOT loaded). Agents are single .md files under .claude/agents/.
- --bare mode (loadSkillsDir.ts:658-675) loads ONLY --add-dir paths, skipping auto-discovery. Not our default.

1B. MCP SERVERS
- Source files: base-cc/src/services/mcp/config.ts. Project MCP = .mcp.json files, walked from cwd to filesystem ROOT, closer-to-cwd wins (getMcpConfigsByScope('project'), config.ts:907-961, the dirs.reverse() merge). PLUS user mcpServers (global config) + local mcpServers (per-project config) + plugin servers + enterprise managed-mcp.json. getProjectMcpConfigsFromCwd (ts:843-881) reads ONLY cwd/.mcp.json.
- .mcp.json shape: { "mcpServers": { "<name>": { "command":..., "args":[...], "env":{...} } | { "type":"http|sse|ws", "url":... } } } (schema McpJsonConfigSchema, types.ts). Project servers must be APPROVED (getProjectMcpServerStatus==='approved', ts:1166-1170) or they don't connect — first-run prompts the user.
- Windows npx caveat (ts:1350-1369) is irrelevant in WSL (getPlatform()!=='windows' there); commands run native Linux.
- CRITICAL: .mcp.json is a SIBLING of .claude, NOT inside it. It lives at a project/dir root. So MCP packaging is independent of the core/.claude skill tree.

1C. CLAUDE.md + RULES (.claude/rules/*.md) — a SEPARATE loader
- base-cc/src/utils/claudemd.ts:790-1075 getMemoryFiles walks getOriginalCwd() UP to FILESYSTEM ROOT (ts:854-857 while currentDir !== parse(currentDir).root — note: to ROOT, NOT git-root, unlike skills/agents). At every dir it loads CLAUDE.md + .claude/CLAUDE.md + ALL .claude/rules/*.md recursively (processMdRules, ts:697-788; rules walk is recursive into subdirs).
- Also user ~/.claude/CLAUDE.md + ~/.claude/rules/*.md (ts:826-847), managed, and --add-dir CLAUDE.md/rules ONLY if env CLAUDE_CODE_ADDITIONAL_DIRECTORIES_CLAUDE_MD is truthy (ts:940-977).
- Conditional rules: a rule .md with frontmatter `paths:` is loaded ONLY when an operated-on file matches the glob (processConditionedMdRules, ts:1354-1397). Unconditional rules (no paths frontmatter) always load. This is the CC-native analog of ELAI's conditional-routing — but ELAI/liteGAME implement routing as a SINGLE conditional-routing.md that the model reads-then-Reads-the-target, NOT via per-file paths frontmatter. P1-A keeps the ELAI/liteGAME pattern (one routing doc); do not convert to native paths-frontmatter in P1.

1D. RULEBOOK IS NOT CC-NATIVE (verified: zero 'rulebook' references in base-cc/src)
- CC never enumerates .claude/rulebook/. Rulebook files are plain docs that the model Reads on a trigger named in conditional-routing.md. Therefore: rulebook files have NO loader requirement — they just need to EXIST at the path the (rewritten) conditional-routing.md points to. Place them under core/.claude/rulebook/ (and packs/<name>/.claude/rulebook/) purely as a filing convention; the routing table is what makes them reachable.

1E. THE DISTRO IS ONE GIT REPO (decisive)
- find: a single /home/user/teikoku/.git (top-level only). git root == distro root. Because skills/agents discovery STOPS at git root, ANY .claude between a project cwd and /home/user/teikoku is auto-discovered; nothing above the distro leaks in.

1F. RECOMMENDED CONCRETE LAYOUT (the executor implements this; it is loadable by the UNMODIFIED base-cc)
- core at the GIT ROOT: /home/user/teikoku/.claude/{skills,agents,rules,rulebook,tools,mcp} + /home/user/teikoku/CLAUDE.md. Because it sits at the git root, it auto-loads for EVERY project run from /home/user/teikoku/projects/<proj>/ FOR FREE (it is an ancestor on every walk). Core needs no composition step.
- packs as standalone trees: /home/user/teikoku/packs/{rust,unity,media,elai}/.claude/{skills,agents,rules,rulebook} (each a normal .claude subtree). Packs are NOT auto-discovered because packs/ is a SIBLING of projects/, not an ancestor of any project cwd.
- PER-PROJECT COMPOSITION (the one mechanism the executor MUST build — without it packs are invisible): assemble an EFFECTIVE .claude under each project that pulls in its active packs. Two grounded options:
  OPTION A (PRIMARY, recommend): for each active pack, SYMLINK the pack's skill dirs / agent files / rule files into projects/<proj>/.claude/{skills,agents,rules,rulebook}. The loader follows symlinks (ripgrep --follow) and inode-dedups, so a pack file symlinked into a project loads exactly once. Deterministic, no env reliance. A small compose script (driven by projects.toml `packs=[...]`) creates/refreshes the links. This is the seam the executor wires.
  OPTION B (FALLBACK, document only): run each project session with --add-dir pointing at each active pack root, and set CLAUDE_CODE_ADDITIONAL_DIRECTORIES_CLAUDE_MD=1 so pack rules/CLAUDE.md load too. Works for skills (loadSkillsDir.ts:699-708) and CLAUDE.md/rules (claudemd.ts:940-977) but is env-dependent and the orchestrator must inject the flags on every launch — more fragile.
- Why core-at-git-root not core-as-a-pack: putting core at the git root removes the most error-prone composition (core is the biggest set; auto-loading it eliminates a whole class of "core didn't load" bugs). Only the opt-in packs go through the compose step.
- FAILURE MODE TO AVOID (the prompt's #1): a layout where core or active packs sit OUTSIDE the cwd-to-git-root ancestor chain and no compose/--add-dir step bridges them = the CC build silently loads nothing. The audit criterion "core loads; ELAI resolves rust+elai packs" is exactly a test of 1F.

================================================================================
SECTION 2 — THE COMPLETE MERGE MAP (every file -> CLASS -> target -> action -> lineage)
================================================================================
Inventories VERIFIED by ls in WSL on 2026-06-20. ELAI tree = harness/elai/.claude; liteGAME tree = harness/litegame/.claude. Target paths assume Section 1F (core@/home/user/teikoku/.claude; packs@/home/user/teikoku/packs/<name>/.claude). "EL"=ELAI copy, "LG"=liteGAME copy.

2.1 SKILLS  (EL has 17 dirs; LG has 15 dirs)
name | in | CLASS | target | action | lineage-when-both
orchestrate | both (LG dir name is also 'orchestrate') | CORE | core/skills/orchestrate | merge-reconcile: take EL body (richer: H-1..H-10, D-030 inject, pod-cost tripwire), keep name 'orchestrate' | EL wins body
auto-orchestrate | EL only | CORE | core/skills/auto-orchestrate | move | —
autonomous-orchestrator | LG only (NOTE: audit called it 'autonomous-orchestrate-plan'; the actual dir is 'autonomous-orchestrator') | CORE | core/skills/autonomous-orchestrator | move; its dual-shape parser + hard COMMIT GATE are the keepers | —
b-plan-runner | EL only | PACK:elai | packs/elai/skills/b-plan-runner | move (ELAI B-plan/SSB/wiring-contract bound) | —
paper-to-plan | EL only | PACK:elai | packs/elai/skills/paper-to-plan | move | —
idea-intake | both-DIFFER | CORE | core/skills/idea-intake | merge-reconcile (diffs = project nouns) | reconcile
planning-workflow | both-DIFFER | CORE | core/skills/planning-workflow | merge-reconcile | reconcile
prompt-transform | both-DIFFER | CORE | core/skills/prompt-transform | merge-reconcile (trivial) | reconcile
rag-tools | both-DIFFER | CORE | core/skills/rag-tools | merge-reconcile | reconcile
oracle | both-DIFFER | CORE | core/skills/oracle | merge-reconcile | reconcile
grill | both-DIFFER | CORE | core/skills/grill | merge-reconcile (EL refs codegraph-elai/elai.map -> parameterize) | reconcile
pdf | both-SAME | CORE | core/skills/pdf | move one copy | identical
anti-spiral | EL only | CORE | core/skills/anti-spiral (depends on persona-* agents, also core) | move | —
cook | EL only | CORE (design-reasoning; binding Q5=CORE) | core/skills/cook | move | —
spec-eval | EL only | CORE (binding Q5=CORE) | core/skills/spec-eval | move (deproject examples) | —
sidecar-protocol | EL only | PACK:elai | packs/elai/skills/sidecar-protocol | move | —
dashboard-handoff | EL only | PACK:elai | packs/elai/skills/dashboard-handoff | move | —
impact-check | EL only | PACK:rust (binding: codegraph-elai->rust/elai) | packs/rust/skills/impact-check | move (wired to .elai_cc/elai.map) | —
unity-scaffold | LG only | PACK:unity | packs/unity/skills/unity-scaffold | move | —
region-maps | LG only | PACK:media (Q1: media is ONE pack) | packs/media/skills/region-maps | move | —
banana-pro-director-2.0 | LG only | PACK:media | packs/media/skills/banana-pro-director-2.0 | move | —
cinema-worldbuilder-pro-2.0 | LG only | PACK:media | packs/media/skills/cinema-worldbuilder-pro-2.0 | move | —
strip-ai-footprint | LG only | PACK:media | packs/media/skills/strip-ai-footprint | move | —
summarize | LG only | CORE-utility (audit cut-candidate) | core/skills/summarize | KEEP (binding did not list it for cut; overlaps pdf but low risk) — flag for owner if cut wanted | —

2.2 AGENTS  (EL has 32 .md; LG has 16 .md)
DECOMPOSITION PIPELINE — byte-identical across both per audit §2; move ONE copy to core/agents:
 idea-framer, idea-critic, concept-decomposer, phase-decomposer, step-decomposer, researcher, paper-ingestor, synthesizer  | both-SAME | CORE | core/agents/<name> | move one copy
 babystep-author | both-DIFFER (EL bigger, extra gates) | CORE | core/agents/babystep-author | merge-reconcile | EL superset
PLANNING/EXEC CORE:
 context-compiler | both-DIFFER | CORE | core/agents/context-compiler | merge-reconcile (deproject paths) | reconcile
 prompt-engineer | both-DIFFER | CORE | core/agents/prompt-engineer | merge-reconcile | reconcile
 memory-writer | both-DIFFER, model=haiku | CORE | core/agents/memory-writer | merge-reconcile (becomes RAG-writer in P2; P1 keeps it) | reconcile
 auditor | both-DIFFER (EL=316 lines wiring/freeze/production-caller/SSB; LG=207 Unity console-clean) | CORE body + pack gate overlays | core/agents/auditor (shared hostile-audit core) ; rust gate steps -> packs/rust ; unity gate steps -> packs/unity | C4 SPLIT (see §3)
COLLAPSE PAIRS (binding: one parameterized agent each; -elai suffix disappears, C8):
 scope-planner + scope-planner-elai | EL has BOTH (112 vs 132 lines) | CORE | core/agents/scope-planner | COLLAPSE to ONE param agent; delete -elai. NOTE diff is NOT noun-only — bodies diverge (see UNKNOWN U-1) | reconcile then delete -elai
 integration-explorer + integration-explorer-elai | EL has BOTH (120 vs 127 lines) | CORE | core/agents/integration-explorer | COLLAPSE to ONE param agent; delete -elai. WARNING: verified diff shows REAL divergence not just nouns — generic copy is iOS/.swift/current_app shaped (the LG-shared generic), -elai copy has its own Execution order (mcp_fsm_get_state, grep file-pivots, tags field). Reconcile both bodies, do not blind-pick. | reconcile then delete -elai
 synthesizer + synthesizer-elai | EL has BOTH (91 vs 115); generic synthesizer is byte-identical to LG | CUT -elai | core/agents/synthesizer | KEEP generic (=core), DELETE synthesizer-elai (§6 cut) | generic wins
ANTI-SPIRAL PERSONAS (EL only, all 5): persona-hacker, persona-researcher, persona-simplifier, persona-architect, persona-contrarian | CORE | core/agents/persona-* | move (travel with anti-spiral skill)
ELAI SPECIALISTS:
 fixer | EL only | CORE (general) | core/agents/fixer | move
 impact-mapper | EL only | CORE (rust-leaning; paths/elai.map pack-bound) | core/agents/impact-mapper | move; note path-coupling
 adversary | EL only | PACK:elai | packs/elai/agents/adversary | move (FalsifiabilityCrossFamily/SSB)
 spec-grounder | EL only | PACK:elai | packs/elai/agents/spec-grounder | move (ELAI spec corpus)
 wiring-contract-validator | EL only | PACK:rust | packs/rust/agents/wiring-contract-validator | move (runs wiring_contract_check.py)
 test-attributer | EL only | PACK:rust | packs/rust/agents/test-attributer | move (cargo-output)
 dashboard-smoker | EL only | PACK:elai | packs/elai/agents/dashboard-smoker | move (dashboard-v2)
 sidecar-health | EL only | PACK:elai | packs/elai/agents/sidecar-health | move (sidecar infra)
CUT:
 designer | both-SAME (28-line retired stub) | CUT | — | DELETE from both (§6)

2.3 RULES (.claude/rules) — EL has 15; LG has 13
wiring-enforcement | both-DIFFER | CORE doctrine + PACK:rust legs | core/rules/wiring-enforcement (10-rule doctrine) ; rust checker legs stay referenced from packs/rust | C5 reconcile
maintainability-review | both-DIFFER | CORE | core/rules/maintainability-review | merge-reconcile
commit-cadence | both-DIFFER | CORE | core/rules/commit-cadence | merge-reconcile; keep LG's VCS abstraction (cm/git) | C6: LG VCS-abstraction wins
output-formatting | both-DIFFER (EL=7-line STUB; LG=67-line FULL) VERIFIED wc -l 7 vs 67 | CORE | core/rules/output-formatting | ADOPT LG FULL version; DELETE EL stub | C1: LG wins (the orchestrators cite the in-code-comment section that exists ONLY in LG)
token-discipline | both-DIFFER (EL bigger) | CORE | core/rules/token-discipline | merge-reconcile | EL superset
main-llm-role | both-DIFFER (47=47 near-identical) | CORE | core/rules/main-llm-role | merge-reconcile
visual-ui-loop | both-DIFFER | CORE (design) | core/rules/visual-ui-loop | merge-reconcile (EL refs CDP/dashboard-v2 -> generalize) 
bun-not-node | both-SAME | CORE | core/rules/bun-not-node | move one copy
conditional-routing | both-DIFFER (53=53) | CORE | core/rules/conditional-routing | REWRITE over merged core+active-pack set (C7, see §4)
tracing-test-assert | EL only | PACK:rust | packs/rust/rules/tracing-test-assert | move (tracing-test crate)
production-caller-gate | EL only | PACK:rust (+py sibling) | packs/rust/rules/production-caller-gate | move
agent-dispatch-preamble | EL only | CORE (critical safety) | core/rules/agent-dispatch-preamble | move (born of git-reset-x8; MUST be core)
orchestrate-protocol | EL only in rules/ (also a rulebook/ dup) | CORE | core/rules/orchestrate-protocol | KEEP rules/ copy (H-1..H-10) ; rulebook dup -> pointer-or-delete | C3
handoff-protocol | EL in rules/ ; LG in rulebook/ (content same per audit; sizes 6337 vs 9987 bytes — VERIFY which is canonical) | CORE | core/rules/handoff-protocol | canonical location = rules/; remove the rulebook copy | C2: rules/ location wins (see UNKNOWN U-2 on byte mismatch)
skill-import-policy | EL only | CORE | core/rules/skill-import-policy | move (security)
verify-before-write | LG only | PACK:unity | packs/unity/rules/verify-before-write | move (Unity C# symbol blacklist)
bug-fix-protocol | LG only | CORE | core/rules/bug-fix-protocol | ADOPT to core (general reproduce-first)
doubt-driven-review | LG only | CORE | core/rules/doubt-driven-review | ADOPT to core (general)
shell-dialect | LG only | CORE | core/rules/shell-dialect | ADOPT to core (Bash/PowerShell hygiene; still relevant for the Windows-native bridge tools)

2.4 RULEBOOK (.claude/rulebook) — EL has 18; LG has 36. Recall §1D: rulebook is NOT CC-loaded; classing it just files it + drives the routing rewrite.
SHARED (move ONE copy to core/rulebook): bubble-up, canvas-image-regen, multi-session, risk-levels, source-tiers, stub-tracking (both-SAME per audit); agent-spawning, coding-standards, design-taste, file-discipline, folder-ownership, gpt-image-prompting, gpt-second-opinion, integration-exploration, memorizing-and-audit, session-protocol (both-DIFFER -> reconcile, diffs = project nouns/stack).
EL-ONLY: codegraph-vs-map | PACK:rust (binding codegraph-elai->rust/elai) | packs/rust/rulebook/codegraph-vs-map
orchestrate-protocol (rulebook dup) | dedupe vs rules/ copy (C3) -> delete or thin pointer
LG-ONLY -> packs:
 unity-*: anti-stale-unity-csharp, unity-architecture, unity-asset-hygiene, unity-map-usage, unity-mcp-usage, unity-mcp-and-builders, unity-ai-platform-decisions, performance-testing, runtime-debugging, ui-responsive-practices | PACK:unity | packs/unity/rulebook/<name>
 media: embervow-pipeline-quickref, remote-comfy, hyperframes-video-design, image-inspection | PACK:media | packs/media/rulebook/<name>
 android-cli | INACTIVE PACK (binding: android kept as inactive pack, NOT cut) | packs/android/rulebook/android-cli
 notion-ticket-format | INACTIVE PACK (binding: notion kept inactive, NOT cut) | packs/notion/rulebook/notion-ticket-format
 extensible-systems, experimentation-hygiene | CORE (general engineering doctrine) | core/rulebook/<name> | ADOPT
 project-config | CORE (the pack-config template seam) | core/rulebook/project-config | ADOPT — directly relevant to projects.toml §5
 handoff-protocol (LG rulebook copy) | dedupe vs core/rules/handoff-protocol (C2) -> delete

2.5 MCP  (audit §4; but see UNKNOWN U-3: no .mcp.json/mcpServers command blocks exist in the copied distro — the source .mcp.json was excluded by the COPY-MANIFEST. The merged .mcp.json must be AUTHORED FRESH from §4 + ELAI CLAUDE.md install paths, not copied.)
CORE servers (author into a merged .mcp.json at a level on the project walk — recommend per-project .mcp.json composed like §1F, or core .mcp.json at git root since MCP walks cwd->fs-root): local-rag (shared install C:\Users\user\.claude\rag — IN WSL must repoint to the WSL/Windows-bridge path; this is a P1.4 path-sweep item but author the entry now), gpt-bridge (shared install, register once), pod-lifecycle (shared infra), canvas (per-root, port-per-root; ELAI 7323).
PACK:rust/elai: codegraph-elai (per-root, -p pins .elai_cc scope) -> declared in the elai/rust project's composed .mcp.json.
PACK:unity: codegraph-zc, codegraph-eo (per-game C# graphs), unityMCP (CoplayDev bridge), unityMCP-runtime (INERT, disabled until dev build), ai-game-developer (CUT or leave-disabled — its own config says prefer unityMCP).
PACK:media: comfy-pipeline, comfyui-cwags (npx -> rewrite to bunx per bun-not-node), blender-bridge.
CUT (5 inert, binding Q4+§6 — REMOVE from any .mcp.json, the inert bundles live at harness/litegame/.claude/mcp/external/): ae-mcp-dakkshin, ae-mcp-ishu86, ae-mcp-rs (3 After Effects, no AE installed), unrealclaude, vibeue (2 Unreal, no UE).

================================================================================
SECTION 3 — §7 CONFLICT RESOLUTIONS AS CONCRETE FILE OPERATIONS (C1-C8)
================================================================================
C1 output-formatting: DELETE harness/elai/.claude/rules/output-formatting.md (7-line stub, VERIFIED). COPY harness/litegame/.claude/rules/output-formatting.md (67-line full) -> core/rules/output-formatting.md. (Deproject maintainer-voice naming only if the executor is generalizing voice; otherwise keep verbatim — the orchestrator executor briefs cite its "In-code comments" section.)
C2 handoff-protocol location: canonical = core/rules/handoff-protocol.md (HARD RULE belongs in rules/). Source the body from whichever copy is canonical (U-2: EL rules/ is 6337 B, LG rulebook/ is 9987 B — they are NOT same-size despite the audit's "content same"; the EXECUTOR must diff them and pick the fuller/correct one, likely LG's 9987 B given it is larger and the audit elsewhere finds LG fuller). DELETE the LG rulebook copy after.
C3 orchestrate-protocol duplication: KEEP core/rules/orchestrate-protocol.md (from EL rules/ copy, 8395 B, the H-1..H-10 hard rule). The EL rulebook/orchestrate-protocol.md (5702 B) and any LG rulebook copy: DELETE (or reduce to a one-line pointer to rules/). No two live copies.
C4 auditor split: keep the shared hostile-audit body (BLOCK/WARN/DEAD-END, sole-test-runner, evidence schema) in core/agents/auditor.md. Move the RUST-specific gate steps (production_caller_check.py, gate_freeze_check, wiring R-legs, tracing-test-assert, smoke-manifest, SSB action_type lock) into a packs/rust gate-checklist the auditor INVOKES by capability detection (active pack), not hardcodes. Move the Unity console-clean / playmode gate steps into packs/unity similarly. P1 mechanism = the core auditor reads a pack-provided gate-checklist file; do NOT attempt the deep single-orchestrator rework (that is P3).
C5 wiring-enforcement: doctrine (R1-R10) stays in core/rules/wiring-enforcement.md (the file itself claims project-agnostic). The runnable checker LEGS (cargo/SCIP/production_caller_check.py vs Unity) are referenced from packs/rust (already there via production-caller-gate.md + the .claude/tools scripts) and packs/unity. No code logic changes in P1 — just the doctrine/legs filing.
C6 commit-cadence: core/rules/commit-cadence.md = the LG version (it abstracts VCS: cm for Plastic + git) so packs can declare their VCS; fold in any EL-only specifics. EL assumes git-only; LG is the superset.
C7 conditional-routing: REWRITE once over the merged core+active-pack set (see §4). This is the single routing table; pack activation extends it. The rewrite is a core/rules/conditional-routing.md authored from both copies' union, with pack-specific dispatch lines marked as pack-gated.
C8 -elai naming: collapse scope-planner+scope-planner-elai -> ONE core/agents/scope-planner.md taking project as a param; same for integration-explorer (+ delete both -elai files); synthesizer-elai is a straight CUT (generic synthesizer is byte-identical to LG and already core). The -elai suffix convention disappears entirely. (See U-1 — the collapse is a real body reconcile, not a noun swap.)

================================================================================
SECTION 4 — conditional-routing.md REWRITE PLAN (C7)
================================================================================
Current EL conditional-routing.md (the live F: copy I read) is the model: a "load only when true" dispatch table mapping a trigger to a Read of a rule/rulebook file. The rewrite (authored as core/rules/conditional-routing.md):
1. UNION the EL + LG routing tables. Both are 53 lines, both-DIFFER. EL routes to ELAI rulebook (codegraph-vs-map, gpt-*, canvas, design-taste, wiring/production/maintainability). LG additionally routes Unity/media/android/notion/shell-dialect/doubt/bug-fix/extensible/experimentation triggers.
2. CORE rows (always live): the dispatch lines for the CORE-classed rules+rulebook from §2.3/§2.4 — wiring-enforcement, production-caller-gate (now rust-gated), maintainability-review, bun-not-node, agent-dispatch-preamble, handoff-protocol, orchestrate-protocol, auto-orchestrate, skill-import-policy, design-taste, visual-ui-loop, cook, gpt-image-prompting, gpt-second-opinion, codegraph-vs-map (rust-gated), bug-fix-protocol, doubt-driven-review, shell-dialect, extensible-systems, experimentation-hygiene, project-config, the rulebook lazy-load block, the capability dispatches (gpt-bridge, canvas).
3. PACK-GATED rows: a section per pack ("when rust-pack active: ... ; when unity-pack active: ... ; when media-pack active: ..."). Unity rows (verify-before-write, unity-* rulebook, performance-testing, runtime-debugging, ui-responsive-practices), media rows (embervow/remote-comfy/hyperframes/image-inspection + banana/cinema/region-maps/strip skills), rust rows (tracing-test-assert, production-caller-gate, codegraph-vs-map, impact-check), elai rows (sidecar-protocol, dashboard-handoff, b-plan-runner, paper-to-plan, spec-grounder, adversary).
4. PATH SWEEP: every path in the table becomes a WSL path under /home/user/teikoku/.claude (core) or packs/<name>/.claude. (Full path sweep is P1.4; but the routing rewrite must use the new core+pack paths from the start so it is not double-edited.)
5. Keep the EL/LG "Read on trigger" semantics (model Reads the doc) — do NOT convert to CC-native paths-frontmatter conditional rules in P1 (§1C). The routing doc is itself an unconditional CORE rule that always loads.
AUDIT this rewrite: every CORE rule/rulebook file in §2 has exactly one routing row; every pack file has a pack-gated row; no row points at a CUT file (§6); no row points at a stale F:/D: path (except intentional Windows-native bridge endpoints).

================================================================================
SECTION 5 — projects.toml SCHEMA + THE ELAI ROW (sub-phase 1.1)
================================================================================
Location: /home/user/teikoku/projects.toml (distro root; the registry the orchestrator + compose script read). Grounding for the field set: master-plan §1 "Project registry" line 67 + Phase 1.1 + the binding disposition. The rulebook/project-config.md (adopted to core, §2.4) is the per-project config template seam — align the schema with it.
Schema (one [[project]] table per registered project):
  [[project]]
  id            = "elai"                         # slug, RAG scope key, unique
  path          = "/home/user/teikoku/projects/elai_cc"   # WSL project root (cwd for sessions)
  platform      = "rust+ts"                       # platform tag (rust+ts | unity | media | ...)
  packs         = ["rust", "elai"]               # active capability packs (drives §1F composition + §4 pack-gated routing)
  done          = "wiring/tests+dashboard"       # done-definition handle -> resolves to the active-pack gate set (D-06); for elai = rust-pack gates (cargo+wiring+maintainability+production-caller+tracing) AND dashboard talks-to-dashboard + dashboard-polish pass
  lanes         = ["all"]                          # allowed worker lanes (all | claude-only | ...); elai = all lanes
  vcs           = "git"                            # VCS (git | cm) — feeds commit-cadence C6 abstraction
Day-1 (N-04): ONLY the elai row is registered. Game/other projects are added later phases. Reserve the shape so the game row would read: id="game", platform="unity", packs=["unity","media"], lanes=["claude-only"] (NDA), vcs per its repo. Do NOT author the game row in P1-A.
The ELAI row VERBATIM is the [[project]] block above with id="elai". Verify projects/elai_cc exists (it does: ls confirmed).

================================================================================
SECTION 6 — CUT LIST (sub-phase 1.3; audit §6 + binding). After execution NOTHING references these.
================================================================================
DELETE entirely:
 - harness/elai/.claude/agents/designer.md AND harness/litegame/.claude/agents/designer.md (retired stub, both)
 - harness/elai/.claude/agents/synthesizer-elai.md (redundant; generic synthesizer is core)
 - harness/elai/.claude/agents/scope-planner-elai.md (collapsed into core scope-planner, C8)
 - harness/elai/.claude/agents/integration-explorer-elai.md (collapsed into core integration-explorer, C8)
 - harness/elai/.claude/rules/output-formatting.md (7-line stub; LG full adopted, C1)
 - one of the orchestrate-protocol duplicates (the rulebook copy/copies; keep core/rules version, C3)
 - one of the handoff-protocol duplicates (after canonicalizing to core/rules, C2)
 - the 5 inert MCP bundles from any .mcp.json AND their external dirs: harness/litegame/.claude/mcp/external/{ae-mcp-dakkshin,ae-mcp-ishu86,ae-mcp-rs,unrealclaude,vibeue}. (ai-game-developer MCP: CUT or leave-disabled per binding/audit — prefer unityMCP.)
KEEP AS INACTIVE PACKS (binding overrides audit cut-candidate): android (packs/android, android-cli skill+rulebook), notion (packs/notion, notion-ticket-format). NOT cut.
KEEP (binding did not list for cut): summarize skill (core utility) — flag to owner only if a cut is desired.
POST-CUT GREP GATE: grep the merged core+active-pack set for any reference to designer / synthesizer-elai / scope-planner-elai / integration-explorer-elai / the 5 inert MCP names / the deleted dup paths -> must return ZERO (excluding the cut-list doc itself + handoff). This is an audit criterion (§7 below).

================================================================================
SECTION 7 — AUDIT CRITERIA (the executor must satisfy; the auditor re-checks OFF-AGENT)
================================================================================
A1 (1F loadability) core LOADS: from a session cwd=/home/user/teikoku/projects/elai_cc, base-cc enumerates the core skills/agents (core@git-root is an ancestor). Verify by listing discovered skills/agents (e.g. a base-cc dry-run or the loader's debug log) and confirming a CORE skill (e.g. orchestrate, idea-intake) + a CORE agent (e.g. context-compiler) appear.
A2 (1F pack resolution) ELAI RESOLVES rust+elai packs: from the same cwd, the composed effective .claude (symlink OR --add-dir per §1F) makes a rust-pack item (impact-check skill / wiring-contract-validator agent / tracing-test-assert rule) AND an elai-pack item (b-plan-runner skill / sidecar-health agent) discoverable; a UNITY-only item (unity-scaffold) is NOT discoverable for the elai project. This is the core+packs proof.
A3 ZERO duplicate agent lineages: no scope-planner-elai / integration-explorer-elai / synthesizer-elai anywhere in the merged set; exactly one of each collapsed agent.
A4 output-formatting FULL present + reachable: core/rules/output-formatting.md is the 67-line LG version; grep it contains the "In-code comments" / maintainer-voice section that the orchestrator executor briefs cite (the section absent from the EL stub). A bare 7-line stub anywhere = FAIL.
A5 all §6 CUT items gone AND unreferenced: the post-cut grep gate (§6) returns zero references; the 5 inert MCPs removed from every .mcp.json; designer/synthesizer-elai/-elai dups deleted.
A6 single canonical location for handoff-protocol (core/rules) and orchestrate-protocol (core/rules) — no second live copy.
A7 conditional-routing rewrite: one routing row per CORE rule/rulebook; pack-gated rows for pack files; no row -> a CUT file; no row -> a stale F:/D: code path (intentional Windows-native bridge endpoints excepted).
A8 projects.toml: parses; elai row present with path/platform/packs/done/lanes/vcs; projects/elai_cc exists.
A9 scope guard (master-plan §2 NON-GOALS): P1-A touches ONLY file reorg — no sidecar/ML/quant work (N-01), no gate REBUILD (N-02, improve/wrap only — and gate improvement is P3 not P1-A), no new deps (C-05). A new dependency introduced = FAIL (anti-slopsquat, verify if seen — expect zero).
A10 (process) one change set per sub-phase; ONE commit per phase after audit (master-plan §5, commit-cadence); memory via the handoff (teikoku-build.md), no memory-writer agent in this build (C-02).

================================================================================
SECTION 8 — CONSTRAINTS (for the executor)
================================================================================
Build: target = WSL distro /home/user/teikoku (C-03). base-cc is FROZEN — DO NOT edit base-cc/src in P1-A (the loader is used as-is; the composition mechanism in §1F is built AROUND it, not inside it). Use bun not node for any compose script (bun-not-node rule). Use rsync/cp/ln inside WSL.
Cycle (master-plan C-02 owner override): context-compiler -> executor -> auditor ONLY. NO integration-explorer, NO memory-writer agent. MAX 2 agents parallel. All agents Opus. (NOTE: this CONTRADICTS the ELAI orchestrate-protocol.md H-2/H-6 which mandate integration-explorer in the parallel batch — the master-plan C-02 + Appendix-A owner override explicitly supersedes it FOR THE TEIKOKU BUILD. The executor follows C-02, not generic orchestrate-protocol.)
Rules: master-plan §0 LOCKED DECISIONS are settled — do not re-litigate D-01..D-17. §2 NON-GOALS is the scope guard. Binding dispositions (Q1 media=one pack, Q4 cut 5 inert MCPs, Q5 design skills=core, Q6 codegraph=core-but-pinned, Q3 FSM stays core+unchanged for P1, P3-deferred single-orchestrator rework) are decided — fold in as constraints.
Locked: layout = core + packs/{rust,unity,media} + PACK:elai + inactive packs/{android,notion} (D-02). The single-reworked-orchestrator (§5 audit recommendation) is a P3 deliverable, NOT P1-A — for P1 just collapse dup lineages + resolve §7 conflicts; do NOT merge the three orchestrate skills into one now.
Git safety (agent-dispatch-preamble R-1): the executor's git is READ-ONLY except the ONE end-of-phase commit the orchestrator authorizes; no reset/checkout/clean/stash. The distro is one git repo — a stray git reset is the exact 2026-06-11 data-loss incident.
Tooling gotcha (this context agent hit it; pass it on): the F:-repo PreToolUse hook misgates this WSL build — the Bash tool is DENIED and Write to distro paths is denied in some FSM states. Use the PowerShell tool with `wsl -d teikoku -- bash -lc '...'` (UNGATED) for WSL commands, and Read/Grep/Glob on \\wsl.localhost\teikoku\... UNC paths. Do NOT mcp_fsm_reset to "fix" it (a prior P0 agent wasted a pass doing that — it is a wrong-tool error, not an FSM bug).

================================================================================
SECTION 9 — UNKNOWNS / UNVERIFIED (do NOT paper over; executor+auditor must resolve)
================================================================================
U-1 The -elai collapse is NOT a pure noun swap. VERIFIED diff of integration-explorer vs integration-explorer-elai shows divergent BODIES: the generic copy is iOS/.swift/current_app-shaped (it is the liteGAME-shared generic that landed in the ELAI tree), the -elai copy has its own Execution order (mcp_fsm_get_state call, grep file-pivots, a `tags` field CROSS_BOUNDARY/DASHBOARD/UI/LOW_CONFIDENCE). scope-planner pair is also 112 vs 132 lines. The executor must RECONCILE both bodies into one parameterized agent (project/stack as input), not pick one and rename. This is real merge work, flagged HIGH.
U-2 handoff-protocol byte mismatch: audit §7 C2 says "content same", but VERIFIED sizes differ (EL rules/ = 6337 B; LG rulebook/ = 9987 B). They are NOT byte-identical. The executor must diff them and choose the canonical body (likely the larger LG copy given the audit's general "LG fuller" finding) before placing at core/rules/handoff-protocol.md. Do not assume identity.
U-3 NO source .mcp.json in the distro. There is NO top-level .mcp.json and NO mcpServers command-block anywhere in harness/elai or harness/litegame (the only mcpServers blocks are inside the inert litegame mcp/external/ Unreal/AE bundles). The COPY-MANIFEST excluded the source roots' .mcp.json. CONSEQUENCE: the merged .mcp.json must be AUTHORED FRESH from audit §4 + the canonical install paths in ELAI's CLAUDE.md (local-rag@C:\Users\user\.claude\rag, gpt-bridge shared, pod-lifecycle, canvas per-root, codegraph-elai), NOT copied. The exact command/args/env per server are NOT in the distro — the executor needs them from ELAI's CLAUDE.md / the original F: .mcp.json (read-only) or must STOP+handoff if they cannot be recovered. The actual WSL launch commands (especially local-rag which is a Windows venv path) are a P1.4 path-sweep concern but the entries are authored in P1-A.
U-4 settings.json hooks carry STALE Windows paths: harness/elai/.claude/settings.json hooks point at F:/.ELAI_workflow/.claude/tools/*.ps1 + node + C:\...\python.exe (canvas-handoff, session-rag-sync, pre_tool_check). These are P1.4 path-sweep items, but note them now — the merged core settings.json hook paths are all stale and the canvas/session-rag hooks reference tools that may not be copied. Not a P1-A deliverable to FIX, but the executor must not assume hooks work.
U-5 liteGAME 'autonomous-orchestrator' vs audit's 'autonomous-orchestrate-plan': the actual skill DIR is named autonomous-orchestrator (VERIFIED ls); the audit §1/§5 calls it autonomous-orchestrate-plan. Same skill (MECH-B in-session plan walker), naming drift only. Use the real dir name.
U-6 The composition mechanism (§1F symlink vs --add-dir) is RECOMMENDED from loader code reading but NOT YET PROVEN end-to-end against this base-cc binary on a live elai session. The executor should run the A1/A2 loadability check EARLY (it is the #1 failure mode) before building the full merge on top of an unproven layout. If symlinks don't compose as predicted, fall back to --add-dir (U-6 fallback) and document.
U-7 base-cc dry-run method for A1/A2 not established: I did not run base-cc to enumerate loaded skills (no live session in this read-only pass). The auditor needs a concrete enumeration command (e.g. a `claude` debug/--print-skills invocation, or grepping the loader debug log with CLAUDE logging on). Establish it as part of the audit harness.

SOURCES CONSULTED
F:\.ELAI_workflow\.claude\rules\agent-dispatch-preamble.md (R-1..R-8 binding)
F:\.ELAI_workflow\.claude\agents\context-compiler.md (agent spec, redirected per prompt)
F:\.ELAI_workflow\workflow\reasoning\TEIKOKU-MASTER-PLAN.md (§0 D-01..D-17, §2 NON-GOALS, §3 C-01..C-07, Phase 1, §5 build method, Appendix A)
F:\.ELAI_workflow\workflow\reasoning\central-workflow-harness-audit.md (FULL — §1 skills, §2 agents, §3 rules+rulebook, §4 MCP, §5 orchestration, §6 cut list, §7 conflicts, §8 open questions)
F:\.ELAI_workflow\.claude\rules\conditional-routing.md (the live routing table model for the C7 rewrite)
WSL inventory (wsl -d teikoku): harness/elai/.claude/{skills(17),agents(32),rules(15),rulebook(18)} ; harness/litegame/.claude/{skills(15),agents(16),rules(13),rulebook(36)}
\\wsl.localhost\teikoku\...\base-cc\src\skills\loadSkillsDir.ts (skill loader; getSkillDirCommands, getProjectDirsUpToHome usage, --add-dir, --bare)
\\wsl.localhost\teikoku\...\base-cc\src\utils\markdownConfigLoader.ts (CLAUDE_CONFIG_DIRECTORIES, getProjectDirsUpToHome git-root stop, loadMarkdownFilesForSubdir three-root merge, ripgrep --follow)
\\wsl.localhost\teikoku\...\base-cc\src\utils\claudemd.ts (getMemoryFiles CLAUDE.md+rules walk to fs-root, processMdRules, conditional paths-frontmatter rules, --add-dir gated by env)
\\wsl.localhost\teikoku\...\base-cc\src\services\mcp\config.ts (.mcp.json scopes, getMcpConfigsByScope project walk, approval gate, npx-windows caveat)
WSL: confirmed single .git at /home/user/teikoku (one repo); rulebook has ZERO refs in base-cc/src (not CC-native); base-cc top-level + src/skills{loadSkillsDir,bundledSkills,mcpSkillBuilders}
WSL byte-verifications: output-formatting EL=7 LG=67 lines (C1); handoff EL-rules=6337B LG-rulebook=9987B (C2); orchestrate-protocol EL rules=8395B + EL rulebook=5702B dup (C3); projects/elai_cc exists; no projects.toml yet
WSL diff: integration-explorer vs -elai (U-1 real body divergence); agent line counts (scope-planner 112/132, integration-explorer 120/127, synthesizer 91/115)
WSL: harness/elai/.claude/settings.json (canvas hooks, stale F:/C: paths, U-4); settings.local.json (MCP permissions list, no command defs — U-3); COPY-MANIFEST.md (copy method, .git/.mcp excluded, base-cc 35M frozen)
mcp_think topic teikoku-1A-loader (loader/composition decision; rejected at 1688 chars but the reasoning is captured in §1F)
