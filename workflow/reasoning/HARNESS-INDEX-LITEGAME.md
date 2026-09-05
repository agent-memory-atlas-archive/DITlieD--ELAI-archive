# HARNESS INDEX — liteGAME (D:\.liteGAME_workflow)

Raw material for the TEIKOKU merge + gate-redesign. liteGAME is the Unity/game side; GAME-tagged rows feed TEIKOKU's "unity game" pack/toggle. CLASS legend: HOOKABLE (mechanically enforceable -> hook/checker), JUDGMENT (behavioral, not checkable), REFERENCE (lookup/depth, lazy). TAGS: STICK / CARROT / GAME / ALWAYS-ON / LAZY.
Format: path | what-it-is | TRIGGER | CLASS | TAGS  (HOOKABLE rows append: GATES <tool> + <check>)
Scope note: external-skills/repos/**, tree-sitter-rust/**, roslyn/** binaries, tools/tests/** and __pycache__ are vendored/test noise, not indexed as harness rules.

## ROOT CLAUDE.md (each HARD RULE block as its own row)

D:\.liteGAME_workflow\CLAUDE.md (whole) | router + always-on one-liners; loads rule files by trigger | always | REFERENCE | ALWAYS-ON CARROT
  - Grounded-by-default (HARD RULE) | every claim needs evidence; else "Ungrounded conjecture:" | always | JUDGMENT | STICK-intent (not mechanically checkable) ALWAYS-ON
  - Project boundary (HARD RULE) | writes target {CURRENT_GAME}/ only; never workflow root / another game; no mid-session game switch | always | HOOKABLE | GATES Write/Edit + deny path outside {CURRENT_GAME} or in another game folder (cf ELAI ".elai_cc boundary") GAME STICK
  - GRC v2 (highest priority) | native thinking ON; deterministic PreToolUse evidence-gate counts mutations-since-evidence, denies 3 edits w/o new Read/Grep/test, or 3 edits to one file w/o evidence (spiral gate) | always | HOOKABLE | GATES Edit/Write + already a gate (pre_tool_check.py, see ALREADY-A-GATE); fail-open STICK ALWAYS-ON
  - mcp_think now opt-in decision ledger | record durable decision at a real fork only | on fork | JUDGMENT | CARROT
  - Register session via mcp_session_register | session bookkeeping | session start | HOOKABLE | GATES session start + verify registration row exists STICK
  - Never commit before showing draft in maintainer voice | commit-draft gate | before commit | HOOKABLE | GATES Bash(cm ci) + require draft shown (partial: pre-checkin-gate covers hygiene not draft) STICK
  - Never AI attribution / no Co-Authored-By | no attribution trailers | always | HOOKABLE | GATES commit msg + grep banned trailer (cf ELAI same rule) STICK ALWAYS-ON
  - Code comments in maintainer voice (HARD RULE) | no 3rd-person/provenance comments in .cs | writing .cs | HOOKABLE | GATES Write/Edit(.cs) + grep "// per ", "// added by", agent/workflow refs GAME STICK
  - Workflow-tool / fan-out ban (HARD RULE) | never invoke Workflow tool off keyword; >2 subagents needs opt-in+count+model+cost | always | HOOKABLE | GATES Task/subagent spawn + block >2 without explicit user opt-in (cf ELAI identical) STICK ALWAYS-ON
  - FSM gate | mcp_fsm_get_state before any Edit/Write/Bash | before mutation | HOOKABLE | GATES Edit/Write/Bash + check FSM state allows STICK ALWAYS-ON
  - Anti-sycophancy | no opening affirmations / auto-agreement | always | JUDGMENT | CARROT ALWAYS-ON
  - Anti-Slopsquatting (HARD RULE) | verify dep name/existence/version before use | adding dep | HOOKABLE | GATES Edit/Write + on new import line require registry-verify receipt (cf ELAI identical) STICK
  - Anti-Stale-Training (HARD RULE) | prefer verified current practice; Unity/C# blacklist | always | JUDGMENT | partially HOOKABLE via verify-before-write gate; rest CARROT ALWAYS-ON
  - Token economy | grep before read; line ranges; max 5-line changelog | always | JUDGMENT | CARROT ALWAYS-ON (read-size cap is HOOKABLE in principle)
  - Confidence calibration | Verified/High-confidence/Uncertain labels | always | JUDGMENT | CARROT ALWAYS-ON
  - Memory directory governance | external memory read-only for rules/feedback | always | JUDGMENT | STICK-intent ALWAYS-ON
  - Remote GPU prefer (embervow) | use remote 3090 when reachable | GPU work | REFERENCE | GAME LAZY
  - Handoff one-file rule (HARD RULE) | append one topic-keyed handoff file, never date/session sibling | "write a handoff" | HOOKABLE | GATES Write under workflow/handoff/ + deny new dated sibling (cf ELAI handoff-protocol) STICK
  - GPT second-opinion (capability) | gpt-bridge optional vote, never grounds | hard fork | REFERENCE | CARROT LAZY
  - Canvas-workbench (capability) | localhost code-graph + click handoff hook | canvas use | REFERENCE | LAZY

## .claude/rules/

D:\.liteGAME_workflow\.claude\rules\conditional-routing.md | the trigger->rulefile dispatch table (game-path-aware) | always | REFERENCE | ALWAYS-ON CARROT
D:\.liteGAME_workflow\.claude\rules\wiring-enforcement.md | R1-R9 fail-closed done-gate doctrine (real entrypoint, off-agent artifact, missing=FAIL, frozen gate) | ship feature / done-gate / flag | HOOKABLE | GATES audit + run wiring/freeze checkers; per-rule deterministic (cf ELAI wiring-enforcement.md, near-identical) STICK
D:\.liteGAME_workflow\.claude\rules\verify-before-write.md | PreToolUse gate denying .cs writes that introduce removed/renamed/unverified Unity symbols | writing .cs | HOOKABLE | GATES Write/Edit(.cs) + unity-verify-gate.ps1 (LIVE hook) GAME STICK
D:\.liteGAME_workflow\.claude\rules\commit-cadence.md | one-changeset-per-task, end-of-plan only, maintainer voice, no AI attribution, pre-checkin hygiene | commit | HOOKABLE | GATES Bash(cm ci) + pre-checkin-gate.ps1 (LIVE) (cf ELAI commit-cadence.md) STICK
D:\.liteGAME_workflow\.claude\rules\output-formatting.md | file-authoring plain-text + maintainer voice + in-code comment ban list | authoring/output | HOOKABLE | GATES Write + grep banned comment patterns; voice is JUDGMENT (cf ELAI output-formatting.md) STICK+CARROT
D:\.liteGAME_workflow\.claude\rules\token-discipline.md | anti-read-big, post-write self-verify, zero-lazy-code, anti-spiral, grounding sweep | always | HOOKABLE | GATES Read(>200) + zero-lazy grep on Write (cf ELAI token-discipline.md) STICK+CARROT ALWAYS-ON
D:\.liteGAME_workflow\.claude\rules\main-llm-role.md | orchestrate-vs-implement boundary, delegation, two-approach mandate | always | JUDGMENT | CARROT ALWAYS-ON (cf ELAI main-llm-role.md)
D:\.liteGAME_workflow\.claude\rules\bug-fix-protocol.md | reproduce-first, evidence-before-hypothesis, one red->green test, smallest fix, anti-spiral | bug/regression/crash | JUDGMENT | partial HOOKABLE (no-edit-before-reproducer) CARROT LAZY
D:\.liteGAME_workflow\.claude\rules\visual-ui-loop.md | deterministic ui-diff.py before vision call; schema critique pass|fail|unknown, one fix/loop | screenshot-compare UI loop | HOOKABLE | GATES vision step + require ui-diff.py run first (cf ELAI visual-ui-loop.md, Node->Python port) GAME STICK LAZY
D:\.liteGAME_workflow\.claude\rules\doubt-driven-review.md | in-flight self-doubt loop before a risky decision hardens (CLAIM/EXTRACT/DOUBT/RECONCILE/STOP, 3-cycle cap) | non-trivial uncertain decision | JUDGMENT | CARROT LAZY
D:\.liteGAME_workflow\.claude\rules\maintainability-review.md | 8-dim maintainability catalog + MR-1..4 deterministic checker, off-agent | ship/audit code | HOOKABLE | GATES audit + maintainability_check.py (frozen) (cf ELAI maintainability-review.md) STICK LAZY
D:\.liteGAME_workflow\.claude\rules\shell-dialect.md | Bash tool must get bash syntax not PowerShell/CMD | running Bash | HOOKABLE | GATES Bash + bash-dialect-gate.ps1 (LIVE) STICK
D:\.liteGAME_workflow\.claude\rules\bun-not-node.md | default JS/TS runtime is bun; no new node/npm/npx; rewrite npx->bunx | JS/TS script/MCP | HOOKABLE | GATES Bash/Write + grep node/npm/npx in new scripts (cf ELAI bun-not-node.md identical) STICK LAZY

## .claude/rulebook/  (lazy-loaded depth; mostly REFERENCE)

D:\.liteGAME_workflow\.claude\rulebook\session-protocol.md | session start, tier classify, plan resume, map refresh | session start | REFERENCE | LAZY (cf ELAI rulebook/session-protocol.md)
D:\.liteGAME_workflow\.claude\rulebook\agent-spawning.md | canonical agent list + sequencing + prompt-engineer WHEN-clause | spawning agents | REFERENCE | STICK-intent LAZY (cf ELAI rulebook/agent-spawning.md)
D:\.liteGAME_workflow\.claude\rulebook\integration-exploration.md | scope-planner / N explorers / synthesizer HEAVY pattern | HEAVY wiring discovery | REFERENCE | LAZY (cf ELAI rulebook/integration-exploration.md)
D:\.liteGAME_workflow\.claude\rulebook\memorizing-and-audit.md | MEMORIZING delegation, 1 audit/phase, decision doc, scoped tests, FSM cleanup | MEMORIZING/audit | REFERENCE | STICK-intent LAZY (cf ELAI rulebook/memorizing-and-audit.md)
D:\.liteGAME_workflow\.claude\rulebook\handoff-protocol.md | one-file-per-problem cross-session handoff, glob-append, head-only | "write a handoff" / resume | REFERENCE | STICK-intent LAZY (cf ELAI handoff-protocol)
D:\.liteGAME_workflow\.claude\rulebook\multi-session.md | session claim/release cross-session file locks | multi-session | REFERENCE | STICK-intent LAZY (cf ELAI rulebook/multi-session.md)
D:\.liteGAME_workflow\.claude\rulebook\orchestrate-protocol.md | orchestrate FSM cycle, dep sleep-poll, phase-end gates, commit-once | "orchestrate" | REFERENCE | STICK-intent LAZY (cf ELAI orchestrate-protocol.md)
D:\.liteGAME_workflow\.claude\rulebook\bubble-up.md | failure bubble-up on baby-step/audit BLOCK/build fail | failure during step | REFERENCE | LAZY (cf ELAI rulebook/bubble-up.md)
D:\.liteGAME_workflow\.claude\rulebook\risk-levels.md | LOW/MED/HIGH node-risk classification | decomposition | REFERENCE | LAZY (cf ELAI rulebook/risk-levels.md)
D:\.liteGAME_workflow\.claude\rulebook\source-tiers.md | research source reliability tiers | researching/web | REFERENCE | LAZY (cf ELAI rulebook/source-tiers.md)
D:\.liteGAME_workflow\.claude\rulebook\stub-tracking.md | registered-stub markers + stubs.md row, will_resolve_in | stub created | HOOKABLE | GATES Write + zero-lazy exception only if registered (cf ELAI rulebook/stub-tracking.md) STICK LAZY
D:\.liteGAME_workflow\.claude\rulebook\file-discipline.md | line caps, fan-out caps, monolith prevention, while-coding self-check | authoring/>150 lines | HOOKABLE | GATES Write + MR-1 file-size in maintainability_check.py (cf ELAI rulebook/file-discipline.md) STICK LAZY
D:\.liteGAME_workflow\.claude\rulebook\folder-ownership.md | _Main vs per-dev folders; EO multi-dev maintainer/ subtree + own branch | EO writes / per-dev gig | HOOKABLE | GATES Write under EO + deny path outside maintainer/ GAME STICK LAZY
D:\.liteGAME_workflow\.claude\rulebook\coding-standards.md | Unity/C# conventions, IDD baseline, no-unsolicited-comments | writing code | REFERENCE | GAME CARROT LAZY (cf ELAI rulebook/coding-standards.md)
D:\.liteGAME_workflow\.claude\rulebook\anti-stale-unity-csharp.md | Unity API + C# staleness blacklist (top-10 hallucinations) | always (Unity) | REFERENCE | GAME STICK-source (feeds verify-before-write gate) LAZY
D:\.liteGAME_workflow\.claude\rulebook\performance-testing.md | perf budgets, profiler, regression detect | perf-sensitive code | REFERENCE | GAME CARROT LAZY
D:\.liteGAME_workflow\.claude\rulebook\extensible-systems.md | walking-skeleton seam-not-switch, SO-plugin, IL2CPP-safe discovery, rule-of-three | building extensible system | REFERENCE | GAME CARROT LAZY
D:\.liteGAME_workflow\.claude\rulebook\runtime-debugging.md | com.taporbit.agentdebug runtime state/log dump | runtime bug | REFERENCE | GAME LAZY
D:\.liteGAME_workflow\.claude\rulebook\unity-ai-platform-decisions.md | standing adopt/defer (keep unity-mcp, defer 1st-party MCP, skip Sentis/Muse) | Unity AI tool choice | REFERENCE | GAME CARROT LAZY
D:\.liteGAME_workflow\.claude\rulebook\unity-asset-hygiene.md | meta/GUID move-in-editor, FormerlySerializedAs, variant depth, asmdef, Addressables-by-lifetime | move/rename asset/asmdef | REFERENCE | GAME STICK-intent LAZY
D:\.liteGAME_workflow\.claude\rulebook\unity-architecture.md | greenfield layered Presentation/App/Domain/Infra/Bootstrap + DI hierarchy (opt-in) | new game/module scaffold | REFERENCE | GAME CARROT LAZY
D:\.liteGAME_workflow\.claude\rulebook\unity-mcp-usage.md | Unity MCP tool catalog, FSM gates, fallback ladder | editor-state ops | REFERENCE | GAME LAZY
D:\.liteGAME_workflow\.claude\rulebook\unity-mcp-and-builders.md | Unity MCP first + procedural builder + UXML text-first (anti fileID/GUID hallucination); hand-edit YAML = BLOCK | prefab/GameObject/SO create | HOOKABLE | GATES Write(.prefab/.unity/.asset YAML) + deny hand-edit GAME STICK LAZY
D:\.liteGAME_workflow\.claude\rulebook\unity-map-usage.md | map.txt schema + codegraph-vs-map split, regen at session start | locate symbol/trace callers | REFERENCE | GAME LAZY
D:\.liteGAME_workflow\.claude\rulebook\ui-responsive-practices.md | split-contract UI, safe-area, one CanvasScaler; FixedDesignFrame=BLOCK | UI layout/aspect-ratio | REFERENCE | GAME STICK-intent LAZY
D:\.liteGAME_workflow\.claude\rulebook\image-inspection.md | read-full-then-quadrant-crop habit for single-image inspection | inspecting an image | REFERENCE | LAZY
D:\.liteGAME_workflow\.claude\rulebook\design-taste.md | anti-slop taste layer: 3 dials, locked accent, authored FontAsset, focal hierarchy, AI-tell=BLOCK | UI judged on look | JUDGMENT | partial HOOKABLE via uss-taste-lint.py GAME CARROT LAZY (cf ELAI rulebook/design-taste.md)
D:\.liteGAME_workflow\.claude\rulebook\hyperframes-video-design.md | video/motion-graphic design model (digest only, MCP not installed) | video/trailer design | REFERENCE | GAME CARROT LAZY
D:\.liteGAME_workflow\.claude\rulebook\gpt-image-prompting.md | gpt_image prompt template + text-in-image rules | calling gpt_image | REFERENCE | LAZY (cf ELAI rulebook/gpt-image-prompting.md)
D:\.liteGAME_workflow\.claude\rulebook\gpt-second-opinion.md | gpt_chat/deep_research second-opinion capability, one weak vote | hard fork | REFERENCE | CARROT LAZY (cf ELAI rulebook/gpt-second-opinion.md)
D:\.liteGAME_workflow\.claude\rulebook\canvas-image-regen.md | regenerate a canvas node image | canvas image regen | REFERENCE | LAZY (cf ELAI rulebook/canvas-image-regen.md)
D:\.liteGAME_workflow\.claude\rulebook\experimentation-hygiene.md | sandbox throwaway in _Sandbox/, shelve to _Shelved/, list-confirm-delete-in-Editor (never "clean up" to agent) | prototyping/cleanup sweep | REFERENCE | GAME STICK-intent LAZY
D:\.liteGAME_workflow\.claude\rulebook\remote-comfy.md | remote 3090 box for embervow GPU work | embervow GPU | REFERENCE | GAME LAZY
D:\.liteGAME_workflow\.claude\rulebook\embervow-pipeline-quickref.md | embervow asset-gen pipeline discovery | embervow asset gen | REFERENCE | GAME LAZY
D:\.liteGAME_workflow\.claude\rulebook\notion-ticket-format.md | maintainer ticket schema (TITLE/DESCRIPTION/PROGRAMMING/EFFORT/PRIORITY) | ticket drafting | REFERENCE | CARROT LAZY
D:\.liteGAME_workflow\.claude\rulebook\project-config.md | GAMES list, stack, perf budgets, key paths (source of {CURRENT_GAME} resolution) | always (game resolve) | REFERENCE | GAME ALWAYS-ON-source LAZY
D:\.liteGAME_workflow\.claude\rulebook\android-cli.md | Google android CLI for emulator/device/SDK after Unity build | Android build/run | REFERENCE | GAME LAZY

## .claude/skills/*/SKILL.md  (trigger -> workflow)

D:\.liteGAME_workflow\.claude\skills\planning-workflow\SKILL.md | plans, FSM states, audits, phase completion + scope-plan/parallel-explore/synthesize | create plan / phase | REFERENCE | STICK-intent (mandatory workflow) ALWAYS-ON-ish (cf ELAI planning-workflow)
D:\.liteGAME_workflow\.claude\skills\prompt-transform\SKILL.md | DROPPED (auto-triggered too aggressively); points to prompt-engineer agent WHEN-clause | n/a | REFERENCE | LAZY (NOTE: liteGAME RETIRED this; ELAI keeps prompt-transform mandatory — divergence)
D:\.liteGAME_workflow\.claude\skills\rag-tools\SKILL.md | local-rag MCP tool usage rules | calling mcp_* RAG | REFERENCE | STICK-intent LAZY (cf ELAI rag-tools)
D:\.liteGAME_workflow\.claude\skills\oracle\SKILL.md | interactive grounded web Q&A via researcher | /oracle /ask | REFERENCE | CARROT LAZY (cf ELAI oracle)
D:\.liteGAME_workflow\.claude\skills\grill\SKILL.md | one-question-at-a-time decision interrogation before planning | /grill | REFERENCE | CARROT LAZY (cf ELAI grill)
D:\.liteGAME_workflow\.claude\skills\idea-intake\SKILL.md | recursive Concept->Phase->Step->BabyStep decomposition with per-level audit | idea/brief/"build me X" | REFERENCE | STICK-intent LAZY (cf ELAI idea-intake)
D:\.liteGAME_workflow\.claude\skills\orchestrate\SKILL.md | autonomous phase orchestrator, FSM cycle per sub-phase, commit-once | "orchestrate" | REFERENCE | STICK-intent LAZY (cf ELAI orchestrate)
D:\.liteGAME_workflow\.claude\skills\autonomous-orchestrator\SKILL.md | same-project plan-file FSM executor (B-plan + TEMPLATE shapes), main-LLM is orchestrator | autonomous-orchestrate-plan | REFERENCE | STICK-intent LAZY (cf ELAI auto-orchestrate)
D:\.liteGAME_workflow\.claude\skills\pdf\SKILL.md | PDF/doc -> markdown via MinerU | /pdf | REFERENCE | LAZY (cf ELAI pdf)
D:\.liteGAME_workflow\.claude\skills\summarize\SKILL.md | url/video/pdf/file summarize via @steipete/summarize CLI | "summarize this X" | REFERENCE | LAZY
D:\.liteGAME_workflow\.claude\skills\strip-ai-footprint\SKILL.md | wrapper over strip_ai_footprint.py; scrub EXIF/XMP/C2PA/ComfyUI chunks | "strip AI footprint"/"scrub image" | REFERENCE | GAME LAZY
D:\.liteGAME_workflow\.claude\skills\unity-scaffold\SKILL.md | materialize Unity feature/game skeleton (folders + asmdef quartet + CODEMAP stub) | "scaffold a feature"/"new game skeleton" | REFERENCE | GAME LAZY
D:\.liteGAME_workflow\.claude\skills\region-maps\SKILL.md | color-key segmenter: flat sprite -> region map + shading.yaml | "add maps to this asset" | REFERENCE | GAME LAZY
D:\.liteGAME_workflow\.claude\skills\banana-pro-director-2.0\SKILL.md | Higgsfield image-prompt director (Banana Pro/Soul Cinema/GPT-2), face lock + char sheets + scene plates | character/outfit/scene image prompts | REFERENCE | GAME LAZY
D:\.liteGAME_workflow\.claude\skills\cinema-worldbuilder-pro-2.0\SKILL.md | Seedance video-prompt director, 5-mode cinematography grammar, frame/subject lock | Seedance/cinematic scene prompt | REFERENCE | GAME LAZY

## .claude/agents/*.md  (name | role)

D:\.liteGAME_workflow\.claude\agents\auditor.md | sole test-runner + wiring/maintainability/done gate | AUDITING | REFERENCE | STICK-role (cf ELAI auditor)
D:\.liteGAME_workflow\.claude\agents\context-compiler.md | compiles context-package, reads plan/memory-bank for main LLM | HEAVY context | REFERENCE | (cf ELAI context-compiler)
D:\.liteGAME_workflow\.claude\agents\memory-writer.md | sole memory-bank writer (mcp_memorize/log_change) | MEMORIZING | REFERENCE | STICK-role (cf ELAI memory-writer)
D:\.liteGAME_workflow\.claude\agents\integration-explorer.md | parallel subsystem wiring exploration | HEAVY wiring | REFERENCE | (cf ELAI integration-explorer-elai)
D:\.liteGAME_workflow\.claude\agents\scope-planner.md | partitions HEAVY work into explorer scopes | HEAVY plan | REFERENCE | (cf ELAI scope-planner-elai)
D:\.liteGAME_workflow\.claude\agents\synthesizer.md | merges explorer outputs into a wiring plan | HEAVY synth | REFERENCE | (cf ELAI synthesizer-elai)
D:\.liteGAME_workflow\.claude\agents\prompt-engineer.md | prompt-transform for HEAVY-fuzzy tasks (revived w/ WHEN-clause) | HEAVY-fuzzy | REFERENCE | (cf ELAI prompt-engineer)
D:\.liteGAME_workflow\.claude\agents\paper-ingestor.md | ingest paper/source -> interface map | grounding | REFERENCE | (cf ELAI paper-ingestor)
D:\.liteGAME_workflow\.claude\agents\researcher.md | grounded web research / oracle mode | research/oracle | REFERENCE | (cf ELAI researcher)
D:\.liteGAME_workflow\.claude\agents\designer.md | design/UX agent (anti-slopsquat at plan time) | design | REFERENCE | (cf ELAI designer)
D:\.liteGAME_workflow\.claude\agents\idea-framer.md | frames raw idea before decomposition | idea-intake | REFERENCE | (cf ELAI idea-framing)
D:\.liteGAME_workflow\.claude\agents\idea-critic.md | adversarial critic loop on framed idea | idea-intake | REFERENCE |
D:\.liteGAME_workflow\.claude\agents\concept-decomposer.md | idea -> Concepts | idea-intake | REFERENCE |
D:\.liteGAME_workflow\.claude\agents\phase-decomposer.md | Concept -> Phases | idea-intake | REFERENCE |
D:\.liteGAME_workflow\.claude\agents\step-decomposer.md | Phase -> Steps | idea-intake | REFERENCE |
D:\.liteGAME_workflow\.claude\agents\babystep-author.md | Step -> BabySteps | idea-intake | REFERENCE |

## .claude/commands/*.md

D:\.liteGAME_workflow\.claude\commands\canvas-mode.md | sets canvas mode flag code|plan|research|design|arch (plain doc flag) | /canvas-mode | REFERENCE | LAZY (cf ELAI canvas-mode)
D:\.liteGAME_workflow\.claude\commands\skill-audit.md | runs skillspector gate on a skill package before promotion | /skill-audit | HOOKABLE | GATES skill import + skill-audit.py exit1=BLOCK STICK LAZY

## .claude/tools/  (gates + checkers + map/rag/visual utilities; tree-sitter/roslyn/tests excluded)

GATE SCRIPTS (live hooks + frozen checkers):
D:\.liteGAME_workflow\.claude\tools\unity-verify-gate.ps1 | PreToolUse(Edit|Write) deny .cs introducing removed/renamed Unity symbol or unverified version-sensitive symbol; fail-open | LIVE hook | HOOKABLE | GATES Write/Edit(.cs) GAME STICK
D:\.liteGAME_workflow\.claude\tools\pre-checkin-gate.ps1 | PreToolUse(Bash) on `cm ci`: BLOCK temp/debug markers, lazy stubs, banned AI/provenance comments, near-zero-FP secrets; WARN test/builder/scratch | LIVE hook | HOOKABLE | GATES Bash(cm ci) STICK
D:\.liteGAME_workflow\.claude\tools\bash-dialect-gate.ps1 | PreToolUse(Bash) deny PowerShell/CMD syntax fed to Bash tool; names bash equivalent; fail-open | LIVE hook | HOOKABLE | GATES Bash STICK
D:\.liteGAME_workflow\.claude\tools\session-gate.ps1 | Stop hook: cross-session wait poll + worklog gate (blocks mid-workflow Stop) | LIVE hook | HOOKABLE | GATES Stop STICK
D:\.liteGAME_workflow\.claude\tools\session-capture.ps1 | PostToolUse: append edit/evidence rows to worklog ndjson (receipt ledger for verify-gate) | LIVE hook | HOOKABLE | GATES PostToolUse STICK
D:\.liteGAME_workflow\.claude\tools\session-rag-sync.ps1 | PostToolUse(Edit|Write): one rolling rag entry/session every 5th edit; fail-open | LIVE hook | REFERENCE | STICK-support
D:\.liteGAME_workflow\.claude\tools\comfy-pull-output.ps1 | PostToolUse(comfyui enqueue): scp-pull ComfyUI PNGs from remote | LIVE hook | REFERENCE | GAME
D:\.liteGAME_workflow\.claude\tools\canvas-handoff-hook.mjs | UserPromptSubmit: inject clicked canvas element as additionalContext, consume once (node) | LIVE hook | REFERENCE | (NOTE: node not bun — cf bun-not-node.md)
D:\.liteGAME_workflow\.claude\tools\canvas-activity-hook.ps1 | PostToolUse/SubagentStop: append activity.jsonl for canvas live overlay | LIVE hook | REFERENCE |
D:\.liteGAME_workflow\.claude\tools\canvas-prompt-inject.ps1 | canvas prompt-injection helper | canvas | REFERENCE |
D:\.liteGAME_workflow\.claude\tools\gate_freeze_check.py | freeze wiring/gate fileset by content hash (blake3/sha3) vs gate-manifest.lock; modified gate=BLOCK exit3; git-independent | audit (R8) | HOOKABLE | GATES audit + detect self-modified gate STICK
D:\.liteGAME_workflow\.claude\tools\maintainability_check.py | MR-1 file-size BLOCK / MR-2 C# escape-hatch / MR-3 long-method / MR-4 branch-density WARN, whole-file (no git) | audit/done | HOOKABLE | GATES audit STICK
D:\.liteGAME_workflow\.claude\tools\skill-audit.py | wraps NVIDIA skillspector; score>50=DO_NOT_INSTALL exit1; static default, --llm opt-in | /skill-audit | HOOKABLE | GATES skill import STICK
D:\.liteGAME_workflow\.claude\tools\scan_unity_patterns.py | regex scanner for Unity C# mobile-perf/correctness anti-patterns; exit1 findings | code audit | HOOKABLE | GATES audit(.cs) GAME STICK
D:\.liteGAME_workflow\.claude\tools\validate_lifecycle.py | MonoBehaviour lifecycle imbalance scanner (Instantiate/subscribe/Addressables/Coroutine without cleanup); exit1 | code audit | HOOKABLE | GATES audit(.cs) GAME STICK
D:\.liteGAME_workflow\.claude\tools\uss-taste-lint.py | deterministic anti-slop/a11y USS checker (CONTRAST BLOCK/WARN, builtin-FONT tell, dead-gray NEUTRAL) | UI taste pre-flight | HOOKABLE | GATES audit(.uss) GAME STICK
D:\.liteGAME_workflow\.claude\tools\check_grounding_complete.py | phase-0 grounding gate: every CLAIM VERIFIED w/ evidence URL+date; exit2 incomplete | grounding sweep | HOOKABLE | GATES TESTING_FIRST entry STICK
D:\.liteGAME_workflow\.claude\tools\ui-diff.py | deterministic image-diff (MAE/changed-px/grid-hotspot/crops) before vision call | visual-ui-loop | HOOKABLE | GATES vision step (must run first) GAME STICK
D:\.liteGAME_workflow\.claude\tools\vlm-prep.py | downscale screenshot to 1568 longest side for VLM judge | visual-ui-loop | REFERENCE | GAME

MAP / RAG / RESEARCH UTILITIES:
D:\.liteGAME_workflow\.claude\tools\unity_map_extractor.py | generate grep-native map.txt for a game (resolves root from project-config GAMES) | session start / map regen | REFERENCE | GAME
D:\.liteGAME_workflow\.claude\tools\unity_map\ (schema/symbol_extractor/unity_guid_scanner/map_writer/unity_code_scanner/unity_scanner/tree_sitter_extractor) | Unity map extractor package internals | (lib) | REFERENCE | GAME
D:\.liteGAME_workflow\.claude\tools\codebase_mapper.py | Rust fn/call dependency mapper -> JSON (ported from ELAI; Rust-oriented, likely vestigial here) | map regen | REFERENCE | (cf ELAI codebase_mapper.py)
D:\.liteGAME_workflow\.claude\tools\query_graph.py | query dependency graphs from codebase_mapper | graph query | REFERENCE |
D:\.liteGAME_workflow\.claude\tools\query_bm25.py / find_bm25_new.py | BM25 graph integration probes (hardcoded F:\.ELAI_workflow path — ELAI residue) | ad-hoc | REFERENCE | (ELAI-residue, dedup candidate)
D:\.liteGAME_workflow\.claude\tools\fetch_arxiv.py | fetch arXiv paper (HTML then PDF) for mcp_ingest_file | paper grounding | REFERENCE | LAZY
D:\.liteGAME_workflow\.claude\tools\strip_ai_footprint.py | 3-layer image AI-footprint scrubber (metadata always-on, FFT/grain opt-in, ONNX restore) | strip-ai-footprint skill | REFERENCE | GAME
D:\.liteGAME_workflow\.claude\tools\extract_gdd_images.py | extract images from a GDD doc | GDD prep | REFERENCE | GAME
D:\.liteGAME_workflow\.claude\tools\extract... debug_def_icon.py / strip_buff_arrows.py / copy_minimap_icons.py | one-shot game-asset image utilities (icons/minimap/buff arrows) | ad-hoc asset prep | REFERENCE | GAME LAZY

PIPELINE SUBDIR (.claude/tools/pipeline/): run_pipeline.py / determinism_check.py / test_quality_scan.py / baseline_record.py / reconcile.py / _conformance_data.py | asset/data pipeline with determinism + quality-scan gates (has own conformance+stress test suite) | pipeline runs | HOOKABLE | GATES pipeline output + determinism_check + quality scan GAME STICK

## ====================================================================
## 1. DUPLICATES / OVERLAP  (dedup candidates for the merge)
## ====================================================================
- wiring-enforcement.md — liteGAME R1-R9 ~= ELAI R1-R10 (ELAI adds R10 integration-seam gate + Python wire-checker). Merge to one; take ELAI's R10. Both ship gate_freeze_check.py + maintainability_check.py (same lineage).
- maintainability-review.md + maintainability_check.py — present in BOTH roots, same thermo-nuclear catalog; liteGAME checker is whole-file (no git, Plastic SCM), ELAI is git-diff-grained. TEIKOKU needs the VCS-agnostic whole-file variant or a VCS adapter.
- commit-cadence.md — both: one-changeset/end-of-plan/maintainer-voice/no-AI-attribution. liteGAME adds Plastic `cm ci` pre-checkin-gate; ELAI is git. Merge with VCS toggle.
- handoff-protocol — liteGAME has it in rulebook/, ELAI in rules/. Identical one-file topic-keyed doctrine. Pick one location.
- token-discipline.md / output-formatting.md / main-llm-role.md / bun-not-node.md / visual-ui-loop.md / design-taste.md / gpt-second-opinion.md / gpt-image-prompting.md / canvas-image-regen.md / source-tiers / risk-levels / bubble-up / multi-session / session-protocol / agent-spawning / integration-exploration / memorizing-and-audit / stub-tracking / file-discipline — all have near-identical ELAI counterparts; dedup to one shared core, push stack-specifics to packs.
- visual-ui-loop + ui-diff/vlm-prep — ELAI is Node/pixelmatch (Svelte/CDP), liteGAME is Python/Pillow (Unity USS + UISnapshot). Same algorithm, different capture/stack. Keep one engine, two capture adapters.
- GRC DIVERGENCE (not a dup, a fork to reconcile): ELAI mandates mcp_think + GRC blockquote echo; liteGAME RETIRED that (GRC v2, native thinking + deterministic evidence-gate in pre_tool_check.py). TEIKOKU should adopt liteGAME's v2 (action-grounded gate) — it is the newer design.
- prompt-transform DIVERGENCE: ELAI keeps it mandatory; liteGAME DROPPED it (too aggressive) and revived prompt-engineer agent with a WHEN-clause. Reconcile.
- agent rosters overlap fully (auditor/context-compiler/memory-writer/integration-explorer/scope-planner/synthesizer/prompt-engineer/paper-ingestor/researcher/idea-* decomposers); names differ only by ELAI's "-elai" suffix.
- skill-audit.py + commands/skill-audit.md — same skillspector gate as ELAI (shared install lineage).
- codebase_mapper.py / query_bm25.py / find_bm25_new.py — ELAI-residue in liteGAME (Rust-oriented, hardcoded F:\.ELAI_workflow paths); the live Unity map path is unity_map_extractor.py + codegraph MCP. Drop the residue on merge.

## ====================================================================
## 2. ALREADY-A-GATE  (existing hooks / frozen checkers found)
## ====================================================================
PROJECT settings.json (D:\.liteGAME_workflow\.claude\settings.json) hooks:
  - PreToolUse(Edit|Write|MultiEdit) -> unity-verify-gate.ps1   [.cs removed/renamed/unverified Unity symbol deny, fail-open]
  - PreToolUse(Bash) -> bash-dialect-gate.ps1                   [PowerShell/CMD-in-Bash deny]
  - PreToolUse(Bash) -> pre-checkin-gate.ps1                    [cm ci hygiene: stubs/debug/provenance-comments/secrets BLOCK+WARN]
  - UserPromptSubmit -> canvas-handoff-hook.mjs                 [inject clicked canvas element]
  - PostToolUse(comfyui enqueue) -> comfy-pull-output.ps1       [pull remote ComfyUI PNGs]
  - PostToolUse(Edit|Write|MultiEdit|Read) -> canvas-activity-hook.ps1  [live overlay feed]
  - PostToolUse(Edit|Write|MultiEdit|Bash|unityMCP*) -> session-capture.ps1  [worklog receipt ledger]
  - PostToolUse(Edit|Write|MultiEdit) -> session-rag-sync.ps1   [rolling rag breadcrumb]
  - Stop -> session-gate.ps1                                    [cross-session wait + mid-workflow Stop block]
  - SubagentStop -> canvas-activity-hook.ps1
GLOBAL settings.local.json (C:\Users\user\.claude\rag\hooks\*, shared with ELAI):
  - PreToolUse(Write|Edit|Bash|NotebookEdit) -> pre_tool_check.py  [GRC-v2 evidence/spiral gate + destructive-bash denylist (raw+expanded) + secret WARN; fail-open, sentinel-gated]
  - PostToolUse(Write|Edit) -> post_tool_check.py                 [ruff/syntax + change recording]
  - PostToolUse(Read|Grep|Glob|WebSearch|WebFetch|search_context|unityMCP find/console/reflect/docs|Bash) -> record_evidence.py  [off-agent EVIDENCE ledger the model cannot fabricate]
  - Stop -> stop_check.py                                          [mid-workflow termination block]
  - PreCompact -> pre_compact_check.py                            [SESSION_HANDOFF autosave]
  - Typed event ledger: ledger.py -> .claude/state/worklog/{sid}.events.ndjson (gate_eval/spiral_hit/destructive_denied/secret_blocked rows)
FROZEN-GATE MACHINERY:
  - gate_freeze_check.py + gate-manifest.txt + gate-manifest.lock (content-hash freeze, git-independent, blake3/sha3, modified gate=BLOCK exit3)
OFF-AGENT CHECKERS (auditor-invoked, not hook-wired):
  - maintainability_check.py (MR-1..4), scan_unity_patterns.py, validate_lifecycle.py, uss-taste-lint.py, ui-diff.py, check_grounding_complete.py, skill-audit.py
  - pipeline/determinism_check.py + pipeline/test_quality_scan.py (asset pipeline gates, with conformance/stress test suite under tools/tests/)
ROSLYN: tools/roslyn/UnityBannedApiAnalyzer + AnalyzerHarness (compiled C# banned-API analyzer with banned/allowed fixtures) — a compile-time Unity-API gate.

## ====================================================================
## 3. UNITY / GAME-SPECIFIC  (TEIKOKU "unity game" pack — load only under the toggle)
## ====================================================================
Rules:
  rules/verify-before-write.md, rules/visual-ui-loop.md (Unity USS/UISnapshot variant)
Rulebook:
  rulebook/coding-standards.md, anti-stale-unity-csharp.md, performance-testing.md, extensible-systems.md,
  runtime-debugging.md, unity-ai-platform-decisions.md, unity-asset-hygiene.md, unity-architecture.md,
  unity-mcp-usage.md, unity-mcp-and-builders.md, unity-map-usage.md, ui-responsive-practices.md,
  design-taste.md (FontAsset/USS variant), hyperframes-video-design.md, experimentation-hygiene.md,
  remote-comfy.md, embervow-pipeline-quickref.md, project-config.md (GAMES/{CURRENT_GAME}), android-cli.md,
  folder-ownership.md (EO per-dev / _Main)
CLAUDE.md blocks: Project boundary ({CURRENT_GAME} routing), Code-comments-in-maintainer-voice (.cs), Remote-GPU-prefer
Skills:
  unity-scaffold, region-maps, banana-pro-director-2.0, cinema-worldbuilder-pro-2.0, strip-ai-footprint
Commands: (none game-only)
Tools (gates/checkers):
  unity-verify-gate.ps1, scan_unity_patterns.py, validate_lifecycle.py, uss-taste-lint.py, ui-diff.py, vlm-prep.py,
  roslyn/UnityBannedApiAnalyzer (+AnalyzerHarness), comfy-pull-output.ps1
Tools (map/asset):
  unity_map_extractor.py + unity_map/* package, extract_gdd_images.py, strip_ai_footprint.py,
  debug_def_icon.py, strip_buff_arrows.py, copy_minimap_icons.py, pipeline/* (asset-gen determinism/quality)
