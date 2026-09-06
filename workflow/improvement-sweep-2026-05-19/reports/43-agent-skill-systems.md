# Agent skill systems (May 2026)

## TL;DR
- PUBLISH 4 ELAI skills as portable SKILL.md to public marketplaces (agentskills.io / awesome-agent-skills): `anti-spiral`, `oracle`, `prompt-transform`, `sidecar-protocol`. They fill gaps no public skill covers (stagnation detection, grounded Q&A, prompt rewrite, remote-GPU health) and earn distribution-side trust by being open.
- KEEP INTERNAL: `b-plan-runner`, `orchestrate`, `planning-workflow`, `idea-intake`, `impact-check`, `dashboard-handoff`. These are project-shaped (FSM, B-N plans, Tauri/Svelte dashboard, .elai_cc layout) and don't generalize.
- ADD to ELAI: (1) per-skill `eval/` directory with negative-trigger tests (Vercel data: skills 53% pass when not explicitly indexed in AGENTS.md vs 100% for passive context) [agent-skills-eval, vercel-blog]; (2) skill telemetry hook logging `skill_name + trigger_method + outcome` per invocation [gh-issue-35319]; (3) `mcp-scan` or equivalent on any third-party skill imported (36% prompt-injection rate, 13.4% critical flaws per Snyk) [snyk-toxicskills].
- ADOPT spec conformance: rename ELAI skill frontmatter to comply with agentskills.io v1 (`name` kebab-case <=64 chars, `description` <=1024 chars, optional `license` / `compatibility` / `metadata.version` / experimental `allowed-tools`). ELAI already complies on `name`+`description`; add `compatibility: "Designed for Claude Code with .elai_cc/ layout"` to keep portability honest [agentskills-spec].
- DROP the assumption "skills are portable end-to-end": format ports, install paths don't. Codex reads `.agents/skills/`, Claude reads `.claude/skills/`, Gemini reads `~/.gemini/antigravity/skills/` [paperclipped, codex-docs]. Provide an `elai install-skills --target codex|claude|gemini` shim, don't pretend it's free.

## Key sources

| URL | what | takeaway |
|---|---|---|
| https://agentskills.io/specification | Canonical Agent Skills v1 spec | 6 frontmatter fields total: `name` (req, <=64), `description` (req, <=1024), `license`, `compatibility` (<=500), `metadata` (free-form), `allowed-tools` (experimental). Body recommended <=500 lines / 5K tokens. |
| https://github.com/anthropics/skills | Anthropic first-party skills repo | Spec file is just a redirect to agentskills.io. First-party skills bundle: docx, pdf, pptx, xlsx, skill-creator. |
| https://developers.openai.com/codex/skills | OpenAI Codex SKILL.md adoption | Same `name`/`description` required. Different install paths (`.agents/skills`, `$HOME/.agents/skills`, `/etc/codex/skills`). Dual invocation: explicit `$skill-name` and implicit description match. Initial skill list capped ~8K chars. |
| https://www.paperclipped.de/en/blog/agent-skills-open-standard-interoperability/ | 32-tool adoption summary | Format ports; install dirs do not. Claude `.claude/skills/`, Codex `.agents/skills/`, Gemini `~/.gemini/antigravity/skills/`. No standardized versioning or dependency declaration. |
| https://snyk.io/blog/toxicskills-malicious-ai-agent-skills-clawhub/ | ToxicSkills supply chain study | 3,984 skills scanned, 13.4% critical issues, 36.82% any security issue, 76 confirmed malicious. 91% of malicious skills use prompt injection. ClawHavoc campaign = 1,184 malicious skills, 12 publisher accts, shared C2 IP. `mcp-scan` tool: 90-100% recall, 0% FP. |
| https://venturebeat.com/security/anthropic-skill-scanners-passed-every-check-malicious-code-test-file | Anthropic scanner bypass | Malicious code hidden in `tests/` files evaded all official scanners. Defense: scan whole skill dir, not just `SKILL.md`. (Could not refetch full text, rate-limited at 429.) |
| https://developers.openai.com/blog/eval-skills | Skill eval methodology | Test 3 pillars: functional viability, directive compliance, operational efficiency. Min 10-20 prompts, 3-5 trials each. Include negative-trigger tests. |
| https://medium.com/google-cloud/agent-skills-evals-stop-vibe-testing-your-skills-edd9eaaa6a1a | Eval framework guide | Stop vibe-testing. Build automated harness measuring trigger correctness over model rev. |
| https://github.com/darkrishabh/agent-skills-eval | Open-source skill eval runner | Test runner for agentskills.io-style skills. |
| https://vercel.com/blog/agents-md-outperforms-skills-in-our-agent-evals | AGENTS.md vs Skills eval | Passive AGENTS.md context: 100% pass. Skills with explicit invocation prompt: 79%. Skills alone (auto-trigger only): 53% = baseline. Skills must be referenced in AGENTS.md to be reliably found. |
| https://github.com/anthropics/claude-code/issues/35319 | Skill telemetry feature request | Proposed: emit `Skill:<name>` events in session-meta with timestamp, trigger method, project hash. Not yet shipped. |
| https://github.com/VoltAgent/awesome-agent-skills | Largest curated registry | 22.2K stars, 1,100+ skills, hand-picked policy. Major contributors: Anthropic, Microsoft (133 skills), Google, Vercel, Cloudflare, HF, Trail of Bits. |
| https://github.com/snyk-labs/toxicskills-goof | ToxicSkills goof repo | Reference malicious skill samples for scanner testing. |
| https://duet.so/guides/agent-skills-101-tools-vs-mcp-vs-skills | Skills vs MCP positioning | Skills = knowledge (procedure, decision rules), MCP = connectivity (API calls). Both Anthropic open standards, complementary. Plugin can contain Skills, Skill can call MCP tools. |
| https://simonwillison.net/2025/Dec/19/agent-skills/ | Independent critique | Spec is "deliciously tiny" but under-specified. `allowed-tools` experimental, `metadata` undefined-by-design. OpenAI absent at launch, quietly adopted later. |

## Skills.md spec comparison

**Anthropic v1 (agentskills.io, source of truth):**
- Required: `name` (1-64 chars, `[a-z0-9-]`, no leading/trailing/consecutive hyphens, must match parent dir name); `description` (1-1024 chars, no `<>`).
- Optional: `license` (free string); `compatibility` (1-500 chars); `metadata` (arbitrary k/v map for client extensions including `version` and `author` by convention, NOT spec-mandated); `allowed-tools` (space-separated, e.g. `Bash(git:*) Bash(jq:*) Read`, marked experimental).
- Body: no format restrictions; recommended <=500 lines / <=5000 tokens; rest in `scripts/`, `references/`, `assets/` subdirs.
- Progressive disclosure: ~100 tokens metadata at startup -> full body on activation -> bundled resources on demand.
- Source: https://agentskills.io/specification (full quoted table above in Key sources).

**OpenAI Codex variant:**
- Same required fields: `name`, `description`.
- Optional metadata is split into a separate `agents/openai.yaml` instead of `metadata` block, breaking single-file portability for Codex-specific config [codex-docs].
- Initial skill list capped at ~8K chars (very large catalogs drop tail skills, with warning).
- Install path is `.agents/skills/` (not `.claude/skills/`).
- Explicit invocation via `$skill-name` syntax in user input is first-class; Anthropic only documents implicit description-match.
- Duplicate `name` resolution: Codex shows both, does not merge. Anthropic spec is silent.

**Diff that matters in practice:**
- A skill written for Claude with `metadata.version` works on Codex (Codex tolerates `metadata` per spec).
- A Codex-specific `agents/openai.yaml` is invisible to Claude.
- A skill that uses `allowed-tools` may or may not be honored — implementation-dependent and currently experimental.

## Cross-vendor portability reality

Format ports, install does not [paperclipped, codex-docs]:

| Vendor | Install path | Notes |
|---|---|---|
| Claude Code | `.claude/skills/` (project) + `~/.claude/skills/` (user) | Reference impl. |
| OpenAI Codex CLI | `.agents/skills/` + `$HOME/.agents/skills/` + `/etc/codex/skills/` + bundled | Hierarchical scope (REPO/USER/ADMIN/SYSTEM). |
| Google Gemini CLI | `~/.gemini/antigravity/skills/` | Single user dir documented. |
| Cursor / VS Code / Junie / Kiro / Goose | Vendor-specific paths | Same SKILL.md format, different discovery. |

What runs unchanged:
- The SKILL.md frontmatter + Markdown body.
- `scripts/` referenced via relative paths (assuming target has the interpreter).
- `references/` Markdown loaded on demand.

What does NOT port:
- Install paths (above).
- Tool grants — `allowed-tools` is experimental; some clients ignore, some enforce, some prompt.
- Vendor-specific extension files (`agents/openai.yaml`, `.claude/agents/*.md` ELAI uses for sub-agents) are invisible to other vendors.
- Telemetry/usage hooks — each vendor surfaces invocations differently (Anthropic proposed in gh-issue-35319, not standardized).

## Skill ecosystem snapshot

| Skill / Bundle | Source | Stars | What it does | Trust signal |
|---|---|---|---|---|
| Anthropic skill-creator | github.com/anthropics/skills/skills/skill-creator | first-party | Authors new skills with eval scaffolding | Vendor-authored |
| Anthropic docx/pdf/pptx/xlsx | github.com/anthropics/skills | first-party | Office-doc round-trip | Vendor-authored |
| Obra Superpowers | (community, see awesome-skills.com) | 40.9K | Largest community skill collection, opinionated bundle | Highest community trust per skywork.ai roundup |
| awesome-agent-skills (VoltAgent) | github.com/VoltAgent/awesome-agent-skills | 22.2K | Curated 1,100+ skills, "not AI-slop" policy | Maintained, hand-curated, multi-vendor compatibility tags |
| code-reviewer | agensi.io/code-reviewer | 116 installs (Agensi #1) | Diff review with style rules | Top install on Agensi |
| git-commit-writer | agensi.io | 65 installs | Generates commit messages | Agensi top-10 |
| readme-generator | agensi.io | 49 installs | Boilerplate README from repo | Agensi top-10 |
| pr-description-writer | agensi.io | 36 installs | PR body from diff + commits | Agensi top-10 |
| temporal-reasoning-sleuth | agensi.io | 32 installs | Date/time/timezone debugging | Niche but useful |
| env-doctor | agensi.io | 30 installs | Diagnose env mismatch | Niche |
| Vercel React Performance | github.com/vercel-labs/agent-skills | first-party | React perf review checklist | Vercel-authored, ported to AGENTS.md after eval failure |
| Microsoft .NET/Python/Java/TS/Rust skills | via awesome-agent-skills | first-party (133) | Per-stack idioms and gotchas | Vendor-authored |
| Trail of Bits security skills | via awesome-agent-skills | first-party | Static analysis, smart contract auditing | Security-firm authored |

ELAI gap analysis vs the catalog:
- No public skill does anti-spiral pattern detection with persona switching — ELAI's `anti-spiral` is novel.
- No public skill bundles remote-GPU sidecar boot + health-check protocol — ELAI's `sidecar-protocol` is novel.
- No public skill does pre-implementation prompt rewriting with grounding sweep — ELAI's `prompt-transform` is novel (`skill-creator` is the closest, but it's authoring, not transforming user prompts pre-implementation).
- Many public skills do "code review", "commit messages", "PR writers" — ELAI doesn't need to ship these.

## Trust + supply-chain pattern

Empirical landscape as of Feb 2026 [snyk-toxicskills]:
- 3,984 skills scanned on ClawHub + skills.sh.
- 1,467 (36.82%) contain at least one security issue.
- 534 (13.4%) contain at least one critical-severity issue.
- 76 confirmed malicious through human review.
- 91% of confirmed malicious skills use prompt injection (vs 2.6% across all skills — strong differentiator).
- 100% of confirmed malicious skills also have malicious code patterns (curl|bash, base64 exfil, password-protected ZIPs).
- Named campaigns: ClawHavoc (1,184 skills, 12 accts, shared C2 IP), `zaycv` (40+ skills), `Aslaep123` (crypto credential theft), `aztr0nutzs` (ready-to-deploy hostile repo).

Defenses emerging:
- `mcp-scan` (Snyk-labs): deterministic-rules + multi-model behavioral analysis. Claimed 90-100% recall, 0% FP on labeled set [snyk-toxicskills].
- OWASP Agentic Skills Top-10 draft, AST01 = Malicious Skills.
- Skill scanner bypass [venturebeat]: scanners check `SKILL.md`, attackers hide payload in `tests/` files. Defense: scan entire skill directory recursively, not just root SKILL.md.

Anthropic skill scanner gaps per Feb 2026 reporting: code-pattern scanners only catch ~9% of malicious skills (the binary-exploit ones); prompt-injection-only payloads slip through because they look like ordinary instructions [snyk-toxicskills].

Publishing barrier reality: "a SKILL.md file and a GitHub account that is one week old" [paperclipped] = no code signing, no review board, no sandboxing by default. Linux Foundation's Agentic AI Foundation (AAIF, founded Dec 9 2025) provides governance home but no enforcement yet.

## Best additions for ELAI

Ranked by leverage (publish/add):

1. **Publish `anti-spiral` to agentskills.io + awesome-agent-skills**.
   - Grounding: no public skill does stagnation detection + persona switching; ELAI's 4-pattern detector (Spinning, Oscillation, NoDrift, DiminishingReturns) and 5-persona swap (Hacker, Researcher, Simplifier, Architect, Contrarian) is unique per the ecosystem snapshot.
   - Hook: existing skill file `/workspace/ELAI\.claude\skills\anti-spiral\SKILL.md`. Strip ELAI-specific FSM refs into an optional `references/elai-fsm.md`; keep the persona switch protocol generic.
   - Add `compatibility: "Designed for Claude Code. Works on Codex with explicit $anti-spiral invocation."`

2. **Publish `oracle` (grounded Q&A)**.
   - Grounding: public skills do code review, commits, READMEs — none do "deep grounded factual Q&A with citation chain" except first-party skill-creator. Differentiated.
   - Hook: depends on ELAI `researcher` agent; for portability, ship the protocol (web-search + citation enforcement + paper-ingest fallback) without requiring the agent file; readers implement.
   - Trust signal: an ELAI public skill with mature eval suite gives ELAI distribution credibility before publishing more.

3. **Publish `prompt-transform` (mandatory prompt rewrite pre-implementation)**.
   - Grounding: Vercel's eval [vercel-blog] shows passive context (AGENTS.md) gets 100% adherence vs 53% for skills-alone; `prompt-transform` operationalizes the AGENTS.md-style rewrite as a skill that runs before any implementation. Differentiates from "code-reviewer" by acting pre-implementation.
   - Hook: existing `/workspace/ELAI\.claude\skills\prompt-transform\SKILL.md`. Generalize: strip B-plan / FSM mentions; keep the grounded-by-default + risk-check + 2-approach rule.

4. **Add per-skill eval/ directory inside ELAI**.
   - Grounding: OpenAI [eval-skills blog] + skill-creator V2 [codex.danielvaughan.com] + agent-skills-eval runner [darkrishabh repo] all converged on 3-pillar eval (function/directive/efficiency) with 10-20 prompts + 3-5 trials.
   - Hook: add `/workspace/ELAI\.claude\skills\<skill>\evals\positive.jsonl` (should trigger) and `negative.jsonl` (should NOT trigger). Wire into auditor via new `skill-eval-runner` tool. Block skill changes that drop trigger accuracy.

5. **Add skill telemetry log**.
   - Grounding: gh-issue-35319 (Anthropic) confirms there is no built-in usage telemetry yet; orgs are rolling their own. ELAI is an org-of-one with multiple skills competing for context budget.
   - Hook: extend existing per-session hook log (`workflow/sessions/{id}/hook-executions.log`) with `skill_invoked` events `{skill, trigger_method=explicit|implicit, prompt_hash, outcome}`. Reuse the FSM hook infrastructure already in CLAUDE.md.
   - Win: know which of ELAI's 10+ skills are dead weight in the context budget vs actually firing.

6. **Add `mcp-scan` (or equivalent) gate before importing third-party skills**.
   - Grounding: Snyk study — 13.4% of public skills have critical flaws, 91% of malicious use prompt injection [snyk-toxicskills]. Anthropic's official scanner missed `tests/`-hidden payloads [venturebeat].
   - Hook: new pre-import rule in `/workspace/ELAI\.claude\rules\` — any external skill must pass `mcp-scan` against the WHOLE directory (not just `SKILL.md`); record the scan hash + result in a skill-imports log; main LLM blocked from referencing unscanned external skills.

7. **Add `compatibility` and `metadata.version` to every ELAI SKILL.md frontmatter**.
   - Grounding: agentskills.io spec section "metadata field" recommends version under metadata as convention; `compatibility` field exists specifically for env-bound skills.
   - Hook: one-line edit per skill. Example: `compatibility: "Requires .elai_cc/ layout, local-rag MCP server, FSM state machine"` makes scope-creep refusal honest and prevents users on Codex from expecting full functionality.

8. **Borrow Manus' three-file working-memory pattern as an ELAI sub-skill, NOT publish**.
   - Grounding: Vellum / Manus pattern [dev.to imaginex] — `task_plan.md`, `notes.md`, `deliverable.md` outperforms vector DBs for single-task scope.
   - Hook: ELAI already does this through `workflow/sessions/{id}/` substructure; document the convention as a sub-skill `session-scratchpad` that other ELAI skills reference.

## Anti-patterns

Cited:
- **Skill scope creep / dumping ground**: "a skill that tries to cover 10 different task types... activates unpredictably, instructions conflict, maintaining it is impossible" [articsledge, termdock]. Rule: one skill, one verb [duet.so]. ELAI's `planning-workflow` is borderline — it covers plan creation, FSM execution, TDD, audit, completion. Consider splitting once size becomes a problem.
- **Vibe-testing skills**: "stop relying on informal vibe checks" [google-cloud medium]; deploy eval harness with negative-trigger tests.
- **Pretending Skills are portable**: install path divergence breaks cross-vendor distribution; don't ship a "Claude skill" without honest `compatibility:` field and per-vendor install instructions [paperclipped].
- **Trusting SKILL.md scanners alone**: Anthropic scanners missed payloads hidden in `tests/` [venturebeat]; scan whole directories, including non-Markdown files.
- **Skill-as-passive-context**: Vercel eval [vercel-blog] — Skills auto-trigger correctly only ~53% of the time without explicit prompt-side references. Skills need either AGENTS.md-style index OR explicit `$skill-name` invocation to be reliable.
- **Loose `metadata`**: agentskills.io leaves `metadata` undefined-by-design [simonwillison.net]; using ad-hoc keys breaks consumer tooling. Stick to conventions like `metadata.author`, `metadata.version` quoted as string.

## Open questions

Measurable:
1. **Skill-invocation precision/recall on ELAI**: with 10+ skills, what's the false-trigger rate? Measurable via prompt-transform-style test corpus, target >=95% precision per Vercel-style eval methodology.
2. **Context-budget cost of dormant skills**: ~100 tokens metadata x 10 skills = ~1K tokens permanent overhead. Worth measuring against actual invocation frequency once telemetry hook lands.
3. **Cross-vendor parity check**: do ELAI's published skills (if shipped) actually fire on Codex CLI under explicit `$skill-name` invocation? Run agent-skills-eval [darkrishabh repo] against both Claude Code and Codex CLI.
4. **Supply-chain scan coverage**: of the public skills ELAI might import, what % pass `mcp-scan`? Tracks ecosystem hygiene over time, especially after Anthropic tightens the scanner gap [venturebeat].
5. **Effectiveness of AGENTS.md-style index vs SKILL.md alone for ELAI**: Vercel saw 100% vs 53% [vercel-blog]; does the same gap exist for procedural ELAI skills, or only for framework-knowledge skills? If true for ELAI, the CLAUDE.md project rules are doing more work than skills.
