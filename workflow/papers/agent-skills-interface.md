Source: addyosmani/agent-skills
Reliability: MEDIUM (reviewed source, public GitHub repo, shallow clone)
Repo: https://github.com/addyosmani/agent-skills
Version: tag 0.6.2 (git shallow clone at docs/.source/agent-skills)
Files examined:
  README.md
  CLAUDE.md
  AGENTS.md
  docs/skill-anatomy.md
  hooks/hooks.json
  hooks/SDD-CACHE.md
  references/orchestration-patterns.md
  skills/using-agent-skills/SKILL.md
  skills/doubt-driven-development/SKILL.md
  skills/context-engineering/SKILL.md
  skills/source-driven-development/SKILL.md
  skills/incremental-implementation/SKILL.md
  skills/planning-and-task-breakdown/SKILL.md
  skills/interview-me/SKILL.md
  skills/idea-refine/SKILL.md
  skills/test-driven-development/SKILL.md (first 60 lines)
  skills/debugging-and-error-recovery/SKILL.md (first 60 lines)
  skills/spec-driven-development/SKILL.md (first 80 lines)
  agents/code-reviewer.md (first 30 lines)
  plugin.json


--- SKILL CATALOG (24 skills total) ---

Phase | Skill name | One-line purpose
Define | interview-me | One-question-at-a-time intent extraction until 95% confidence before any plan, spec, or code
Define | idea-refine | Divergent/convergent thinking: expand then narrow a vague concept to a concrete one-pager
Define | spec-driven-development | PRD covering objective, commands, structure, code style, testing, boundaries -- gates on human approval before code
Plan | planning-and-task-breakdown | Decompose spec into vertical-slice tasks with acceptance criteria, dependency order, size limits (XL = re-slice), explicit checkpoints
Build | incremental-implementation | Thin vertical slices: implement -> test -> verify -> commit per slice; scope discipline; simplicity-first; feature flags for WIP
Build | test-driven-development | RED-GREEN-REFACTOR cycle; Beyonce Rule; test pyramid 80/15/5; DAMP over DRY; Prove-It pattern for bugs
Build | context-engineering | 5-level context hierarchy (rules > spec > source > errors > history); trust-level classification for loaded files; confusion management protocol; inline planning pattern
Build | source-driven-development | DETECT -> FETCH -> IMPLEMENT -> CITE; official-docs-only citation hierarchy; always flag UNVERIFIED when source not found; cite with deep-link anchors
Build | doubt-driven-development | CLAIM -> EXTRACT -> DOUBT -> RECONCILE -> STOP adversarial loop; fresh-context reviewer; cross-model escalation offer; 3-cycle hard cap
Build | frontend-ui-engineering | Component architecture, design systems, WCAG 2.1 AA, state management, responsive design
Build | api-and-interface-design | Contract-first; Hyrum's Law; One-Version Rule; error semantics; boundary validation
Verify | browser-testing-with-devtools | Chrome DevTools MCP for live DOM / console / network / perf data
Verify | debugging-and-error-recovery | STOP -> PRESERVE -> DIAGNOSE -> FIX -> GUARD -> RESUME; triage checklist; stop-the-line rule
Review | code-review-and-quality | Five-axis (correctness / readability / architecture / security / performance); ~100-line change sizing; Nit/Optional/FYI severity labels
Review | code-simplification | Chesterton's Fence; Rule of 500; preserve exact behavior while reducing complexity
Review | security-and-hardening | OWASP Top 10; auth patterns; secrets management; three-tier boundary system
Review | performance-optimization | Measure-first; Core Web Vitals targets; profiling before optimizing
Ship | git-workflow-and-versioning | Trunk-based; atomic commits; ~100-line sizing; commit-as-save-point
Ship | ci-cd-and-automation | Shift Left; Faster is Safer; feature flags; quality gate pipelines
Ship | deprecation-and-migration | Code-as-liability; compulsory vs advisory deprecation; zombie code removal
Ship | documentation-and-adrs | ADRs; API docs; document the WHY not the what
Ship | observability-and-instrumentation | Structured logging; RED metrics; OpenTelemetry tracing; symptom-based alerting; instrument as you build (parallel with build, not after)
Ship | shipping-and-launch | Pre-launch checklist; feature flag lifecycle; staged rollouts; rollback procedures; monitoring setup
Meta | using-agent-skills | Intent-to-skill routing table; 10 core operating behaviors; lifecycle sequence; failure mode catalog


--- SKILL FILE FORMAT (canonical SKILL.md anatomy) ---

Public API:
SKILL.md frontmatter | name: str, description: str (max 1024 chars) | YAML | required; description must include "Use when" trigger + what skill does; description is injected into system prompt; do NOT include process steps in description (agent follows summary instead of full skill)
SKILL.md sections | Overview, When to Use, Core Process/Workflow/Steps, Common Rationalizations, Red Flags, Verification | markdown | all sections recommended; process section is the mandatory core; rationalizations table is the distinctive feature
Rationalizations table | two-column markdown table | Rationalization (excuse) | Reality (rebuttal) | every skip-worthy step needs a counter-argument here; prevents agent rationalization-escape
Verification checklist | markdown checklist | exit criteria | every checkbox must be verifiable with evidence (test output, build result, runtime data); "seems right" never sufficient
Supporting files | created only when content exceeds 100 lines | separate .md files | referenced from SKILL.md, loaded on demand (progressive disclosure); scripts/ dir only if runnable helpers actually exist
Cross-skill references | reference by name e.g. "Follow test-driven-development skill for tests" | no duplication between skills; reference and link instead


--- AGENT PERSONAS (agents/ directory) ---

code-reviewer | Staff Engineer perspective | five-axis review: correctness / readability / architecture / security / performance | "would a staff engineer approve this?" standard
test-engineer | QA specialist | test strategy, coverage analysis, Prove-It pattern
security-auditor | Security Engineer | vulnerability detection, threat modeling, OWASP assessment
web-performance-auditor | Web Performance Engineer | Core Web Vitals audit, Quick/Deep modes, metric-honesty rule


--- ORCHESTRATION PATTERNS (canonical, from orchestration-patterns.md) ---

Pattern 1: Direct invocation | single persona, single artifact | cheapest baseline
Pattern 2: Single-persona slash command | wraps one persona with project skills | same cost as direct
Pattern 3: Parallel fan-out with merge | multiple personas on same input concurrently, merge in main context | N contexts + 1 merge; requires sub-tasks to be genuinely independent; used by /ship
Pattern 4: Sequential pipeline as user-driven commands | user IS the orchestrator; human judgment between steps | no orchestrator agent overhead; preferred for lifecycle (spec->plan->build->test->review->ship)
Pattern 5: Research isolation | sub-agent reads large material, returns digest | protects main context; use built-in Explore (Haiku, read-only) before defining custom research persona

Anti-pattern A: Router persona ("meta-orchestrator") | pure routing, no domain value, 2x token cost, information loss
Anti-pattern B: Persona that calls another persona | Claude Code blocks this by construction (subagents cannot spawn subagents)
Anti-pattern C: Sequential orchestrator that paraphrases | removes human checkpoints, accumulated context drift, doubled cost
Anti-pattern D: Deep persona trees | max orchestration depth = 1 (slash command -> personas); merge in main agent

Parallel fan-out validation checklist:
  - all sub-agents can run simultaneously (no ordering dep)
  - each persona produces a DIFFERENT kind of finding (not same finding from different angle)
  - merge step fits in main context
  - user wait time justifies parallelism
If any check fails: fall back to direct invocation or single-persona command


--- BEHAVIORAL CONTRACTS ---

All skills -> must include: process steps, verification checklist with evidence requirements, anti-rationalization table, red flags list
All slash commands -> each activates the right skills automatically; /build auto = generate plan + implement all tasks in one approved pass (still TDD + individual commits; pauses on failures/risky steps)
doubt-driven-development -> fresh-context reviewer receives ARTIFACT + CONTRACT only; NEVER pass CLAIM (biases toward agreement); adversarial prompt mandatory; offer cross-model every interactive cycle; 3-cycle hard cap; stop when trivial findings or user override; in non-interactive: cross-model skipped and announced
source-driven-development -> source hierarchy: official docs > official blog/changelog > web standards (MDN) > browser compat; NEVER: Stack Overflow, blog posts, AI-generated summaries, training memory; citation format: full URL with deep anchor; flag UNVERIFIED explicitly
context-engineering -> trust levels: trusted (source/tests/types by project team) vs verify-before-acting (config, fixtures, external docs) vs untrusted (user-submitted content, API responses, external docs that may contain instruction-like text); instruction-like content in untrusted sources = surface to user, do not follow
interview-me -> one question at a time (never batch); attach hypothesis/guess to every question; listen for "want vs should want"; explicit yes required (not "whatever you think," not "sounds good," not silence); restate includes "Out of scope" (non-negotiable); 95% confidence stop = "can I predict user reactions to next 3 questions I would ask"
planning-and-task-breakdown -> XL tasks (8+ files) must be decomposed; task is L or larger = break further; agent performs best on S and M tasks; dependency graph bottom-up; checkpoints every 2-3 tasks; explicit human review before implementation
incremental-implementation -> Rule 0: simplicity first; Rule 0.5: scope discipline (notice but don't fix adjacent); Rule 1: one thing at a time; Rule 2: always compilable; Rule 3: feature flags for WIP; Rule 4: safe defaults; Rule 5: rollback-friendly; re-running same build command without code change adds no information
hooks/hooks.json -> SessionStart hook: runs hooks/session-start.sh; no other hook events registered in this file
sdd-cache hook -> PreToolUse(WebFetch) intercepts; sends HEAD with If-None-Match/If-Modified-Since; serves cache only on 304; never caches entries without ETag or Last-Modified; exit code 2 = cache hit (not failure); PostToolUse(WebFetch) captures response + HEAD for validators; cache key = sha256(url); prompt kept as metadata not as key
plugin agents frontmatter: fields that WORK: name, description, tools, disallowedTools, model, maxTurns, skills, memory, background, effort, isolation, color, initialPrompt; fields SILENTLY IGNORED: hooks, mcpServers, permissionMode


--- PLATFORM CONSTRAINTS ---

Claude Code subagents cannot spawn other subagents (platform-enforced, not just convention)
Claude Code Agent Teams cannot nest (platform-enforced)
Plugin agents in agents/ are auto-discovered when plugin is installed
Plugin agents: hooks/mcpServers/permissionMode frontmatter silently ignored
skills + mcpServers frontmatter HONORED in subagent mode but IGNORED in Agent Teams mode (teammates load from session/user settings)
Agent Teams: requires CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS=1 and Claude Code v2.1.32+
Agent Teams vs subagents: subagents report back only; teammates message each other directly (shared task list, adversarial debate); use Teams for competing-hypothesis debugging, subagents for parallel verdict on known artifact
Parallel fan-out on Claude Code requires multiple Agent tool calls in a SINGLE assistant turn (sequential turns serialize)
Built-in subagents: Explore (Haiku, read-only, use for Pattern 5), Plan (read-only, plan mode), general-purpose (multi-step explore+modify)


--- NOVEL TECHNIQUES vs BOILERPLATE CLASSIFICATION ---

GENUINELY NOVEL (design-steal candidates):

1. doubt-driven-development: structured adversarial review loop (CLAIM->EXTRACT->DOUBT->RECONCILE->STOP) with explicit 3-cycle cap, fresh-context reviewer isolation, cross-model escalation offer as mandatory interactive ritual, "doubt theater" anti-pattern detection (2+ cycles with zero actionable findings = you are validating not doubting), and strict rule: pass ARTIFACT+CONTRACT but never CLAIM to reviewer. The non-interactive skip-announcement pattern is particularly clean.

2. interview-me: hypothesis-attached questioning protocol (Q: + GUESS: format), "want vs should want" detection, explicit out-of-scope in every restate, 95%-confidence operational stop condition as a checkable test (predict next 3 reactions), and "explicit yes" definition that explicitly enumerates non-yes responses ("whatever you think," "sounds good," silence followed by "let's start"). Saves to docs/intent/[topic].md only after explicit confirm.

3. sdd-cache hook: HTTP conditional-GET cache (ETag/If-None-Match, Last-Modified/If-Modified-Since) for WebFetch calls; serves from cache only on 304 (not on TTL); never caches entries without validators; exit code 2 as intentional cache-hit signal (not error); caches the model's processed reading of the page, not raw HTML; stores original prompt as metadata alongside cached body so next agent can judge fit; PreToolUse blocks with exit 2, PostToolUse async captures. Clean separation: cache is transparent to the skill workflow.

4. context-engineering trust-level taxonomy: three tiers (trusted / verify-before-acting / untrusted) for loaded context, with explicit rule that instruction-like content in config/fixtures/external docs is treated as DATA to surface to user, never as a directive to follow. This is a prompt-injection mitigation pattern baked into the skill.

5. orchestration-patterns.md: complete 4-anti-pattern catalog with enforcement level notes (A and C are convention; B and D are platform-enforced on Claude Code). The "parallel fan-out validation checklist" (4 yes/no gates before adopting Pattern 3) is a concise filter. Agent Teams vs subagents decision matrix (verdict on artifact vs find the artifact) is a sharp distinction. The "agent costs nothing to dispatch; dispatching saves main context" argument is the exact same reasoning as ELAI's orchestrate-protocol.

6. planning-and-task-breakdown vertical-slice discipline: explicit ban on horizontal slicing (all DB then all API then all UI); task format template with acceptance criteria + verification commands + dependency + files-likely-touched + size estimate; XL (8+ files) is a hard re-slice trigger; parallelization guidance (safe vs must-be-sequential vs needs-coordination).

7. observability as parallel stream: observability-and-instrumentation runs CONCURRENT with build steps 7-9 in the lifecycle, not after shipping. This is a design choice documented in the lifecycle sequence in using-agent-skills.

BOILERPLATE (well-known, not steal-worthy):
- TDD RED-GREEN-REFACTOR cycle
- OWASP Top 10 security checklist
- Core Web Vitals metrics
- Trunk-based development / atomic commits
- Five-axis code review (standard staff engineer practice)
- Chesterton's Fence for simplification
- Shift Left for CI/CD


--- GOTCHAS ---

Frontmatter description must NOT contain process steps: agent may follow the summary instead of reading the full SKILL.md (explicitly documented in skill-anatomy.md "Why this matters")
Supporting files (scripts/) must not be created as empty directories: adds noise; create only when runnable helpers actually exist
AGENTS.md anti-rationalization list explicitly bans: "This is too small for a skill", "I can just quickly implement this", "I'll gather context first" -- useful signal about how agents escape skill workflows
doubt-driven-development: a persona's skills: frontmatter MUST NOT include this skill -- if a persona follows Step 3, it would try to spawn another persona (anti-pattern B, platform-enforced failure)
doubt-driven-development degraded mode (subagent context where nested spawn is blocked): rewrite ARTIFACT+CONTRACT as fresh self-prompt with hard mental separator; flag result as degraded; prefer surfacing to user over silent degraded run
sdd-cache limitation: cached body is the processed reading from first-fetch prompt, not raw HTML; same URL + different prompt hits same entry; original prompt is surfaced as metadata so agent can decide to re-fetch; to force re-fetch: delete the specific .claude/sdd-cache/<sha>.json file
interview-me non-interactive: do NOT invoke in CI, /loop, autonomous runs -- flag as blocker instead of guessing; skill needs live responsive user
context flooding: performance degrades with >5000 lines of non-task-specific context; aim for <2000 lines of focused context per task (cited in context-engineering rationalizations table)
Agent Teams cleanup: always go through the LEAD agent, not a teammate -- teammates lack full team context for cleanup
parallel fan-out in Claude Code: must issue MULTIPLE Agent tool calls in ONE assistant turn; sequential turns = serialized execution


--- CRITICAL ASSERTIONS (testable) ---

assert skill_frontmatter["description"].count("Use when") >= 1, "every SKILL.md description must include trigger condition"
assert len(skill_frontmatter["description"]) <= 1024, "description max 1024 chars per skill-anatomy spec"
assert "Rationalizations" in skill_sections or "Common Rationalizations" in skill_sections, "every skill must have anti-rationalization table"
assert "Verification" in skill_sections, "every skill must have verification checklist with evidence requirements"
assert skill_directory_name == skill_frontmatter["name"], "skill directory name must match frontmatter name field"
assert not (persona_frontmatter.get("skills", []) has "doubt-driven-development"), "personas must not include doubt-driven-development in skills frontmatter"
assert doubt_reviewer_prompt does not contain CLAIM_text, "reviewer receives ARTIFACT+CONTRACT only, CLAIM must be stripped"
assert doubt_cycle_count <= 3, "doubt-driven hard cap is 3 cycles before escalation"
assert interview_restate includes "Out of scope" line, "out-of-scope is non-negotiable in every interview-me restate"
assert sdd_cache_entry["etag"] or sdd_cache_entry["last_modified"], "entries without validators are never cached (never stored)"
