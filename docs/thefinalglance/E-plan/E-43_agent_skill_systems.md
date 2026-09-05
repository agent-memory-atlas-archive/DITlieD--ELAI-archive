# E-43: SKILL.md export to agentskills.io v1 + per-skill eval/ + skill telemetry + mcp-scan gate for third-party skills
Objective: bring ELAI's `.claude/skills/` into formal agentskills.io v1 spec compliance, publish 3 novel ELAI skills (anti-spiral, oracle, prompt-transform) to public marketplaces as cross-vendor SKILL.md, ship per-skill `eval/` directories with positive/negative trigger corpora wired into a `skill-eval-runner`, add skill-invocation telemetry to the hook executions log, and gate every third-party skill import behind an `mcp-scan` whole-directory scan before any reference is allowed.
Estimated effort: 2-3 sessions, ~8-10 h
Pre-requisites: E-22 SHIPPED-or-IN-FLIGHT (SKILL.md export shim — overlaps; coordinate which lands first), B-19 PLANNED (Hook System Unification — for telemetry hook), C-39 PLANNED (MCP Bridge — if `mcp-scan` ships as MCP server)
Parallel-safe with: E-41 (CI), E-42 (grammars), E-44 (durability), E-45 (drift)
Source report: workflow/improvement-sweep-2026-05-19/reports/43-agent-skill-systems.md

## Context

The agent skills ecosystem matured fast in Dec 2025 — Feb 2026: Anthropic published the SKILL.md spec (agentskills.io/specification, 6 frontmatter fields, ≤500 lines body, progressive disclosure), OpenAI Codex adopted SKILL.md (developers.openai.com/codex/skills) at different install paths (`.agents/skills/` vs `.claude/skills/`), Google Gemini added it at `~/.gemini/antigravity/skills/`. The format is cross-vendor; install paths are NOT. ELAI's `.claude/skills/` already follows the structure but does not export portable manifests, has no eval/ directory, no telemetry hook, and no third-party-import scan.

Supply-chain risk is documented and live: Snyk's ToxicSkills study (Feb 2026, snyk.io/blog/toxicskills-malicious-ai-agent-skills-clawhub) scanned 3,984 skills and found 13.4% with critical-severity issues, 36.82% with at least one security issue, 76 confirmed malicious; 91% of confirmed malicious skills use prompt injection (vs 2.6% baseline). The ClawHavoc campaign (1,184 malicious skills, 12 accounts, shared C2 IP) and named `zaycv`/`Aslaep123`/`aztr0nutzs` campaigns are active. Anthropic's official scanner was bypassed (venturebeat 2026) when payloads hid in `tests/` directories — scanners that only check `SKILL.md` miss `tests/`/`scripts/` payloads. `mcp-scan` (snyk-labs) claims 90-100% recall and 0% FP on the labeled set; it must scan the whole directory recursively.

Vercel's AGENTS.md vs Skills eval (vercel.com/blog/agents-md-outperforms-skills-in-our-agent-evals) is the most cited datapoint: passive AGENTS.md context hits 100% pass; Skills with explicit invocation prompt hit 79%; Skills alone (auto-trigger only) hit 53%. ELAI's CLAUDE.md already operates as an AGENTS.md-style passive index for the project — that's doing more work than the skills themselves. The implication for E-43: skills must either be referenced explicitly in CLAUDE.md OR support explicit `$skill-name` invocation OR they will misfire ~half the time.

ELAI's three novel public-publishable skills (per the report's gap analysis): `anti-spiral` (4-pattern stagnation detector + 5-persona switcher — no public peer), `oracle` (deep grounded Q&A — closest peer is Anthropic's `skill-creator` which is authoring not querying), and `prompt-transform` (mandatory pre-implementation rewrite — operationalizes the AGENTS.md-style passive context as a skill). The remaining 6+ ELAI skills (`b-plan-runner`, `orchestrate`, `planning-workflow`, `idea-intake`, `impact-check`, `dashboard-handoff`) are project-shaped (FSM, B-N plans, `.elai_cc/` layout) and stay internal.

## Existing state (B/C/D coverage)

- E-22 SHIPPED-or-IN-FLIGHT (per Wave 5 of this improvement sweep): SKILL.md export shim, walks `.claude/skills/`, normalizes YAML frontmatter to Anthropic spec, writes to `dist/skills/`, computes blake3 drift hash. E-43 EXTENDS: adds the `compatibility:` and `metadata.version` fields to source SKILL.md frontmatter, adds the per-skill `eval/` directory, ships the publish path for non-Claude targets (Codex, Gemini), and adds the `mcp-scan` import gate. If E-22 is not yet in flight, E-43 absorbs the export shim work — coordinate with the user.
- B-19 PLANNED (Hook System Unification): telemetry hook landing point. EXTENSION: E-43's `skill_invoked` event goes through the unified hook surface that B-19 stabilises. If B-19 is not yet started, E-43 lands the per-skill hook into the existing `workflow/sessions/{id}/hook-executions.log` and B-19 unifies later.
- B-20 PLANNED (Tool Registry Unification): the `skill-eval-runner` tool registers here. EXTENSION: E-43 adds the tool definition; B-20 absorbs it during registry stabilisation.
- C-39 PLANNED (MCP Bridge CLI Surface): if `mcp-scan` is published as an MCP server, the bridge already in C-39 handles transport. EXTENSION: E-43 adds `mcp-scan` as a known MCP server in the registry; C-39 ships the transport.
- The `.claude/skills/` directory already exists with 10+ skills (b-plan-runner, anti-spiral, oracle, prompt-transform, planning-workflow, rag-tools, sidecar-protocol, dashboard-handoff, idea-intake, impact-check, orchestrate, and per-rule files in `.claude/rules/`). E-43 modifies SKILL.md frontmatter in-place for the 3 publishable skills AND adds `eval/` subdirs across all 10+.
- `.claude/rules/conditional-routing.md` already lists the lazy-loaded rulebook dispatches; E-43 adds an entry pointing to a new `.claude/rules/skill-import-policy.md` (CREATE) that gates third-party skill imports behind mcp-scan.

## Reads (mandatory before coding)

- `F:\.ELAI_workflow\workflow\improvement-sweep-2026-05-19\reports\43-agent-skill-systems.md` (full)
- `F:\.ELAI_workflow\workflow\improvement-sweep-2026-05-19\E-plan\E-22_github_trending_may2026.md` (existing SKILL.md export shim work — coordinate)
- `F:\.ELAI_workflow\.claude\skills\anti-spiral\SKILL.md` (publishable skill #1)
- `F:\.ELAI_workflow\.claude\skills\oracle\SKILL.md` (publishable skill #2)
- `F:\.ELAI_workflow\.claude\skills\prompt-transform\SKILL.md` (publishable skill #3)
- `F:\.ELAI_workflow\.claude\rules\conditional-routing.md` (add skill-import-policy entry)
- `F:\.ELAI_workflow\CLAUDE.md` lines ~"Skills (mandatory workflow)" (passive AGENTS.md-style index for skills — Vercel grounding)
- WebFetch: agentskills.io/specification (canonical v1 spec)
- WebFetch: github.com/snyk-labs/mcp-scan (scanner tool)
- WebFetch: developers.openai.com/codex/skills (Codex install paths + `$skill-name` invocation)
- WebFetch: github.com/darkrishabh/agent-skills-eval (open-source eval runner)
- WebFetch: snyk.io/blog/toxicskills-malicious-ai-agent-skills-clawhub (ToxicSkills numbers)

## Tasks

1. Frontmatter compliance + `compatibility` + `metadata.version` fields. For each `.claude/skills/*/SKILL.md`, audit YAML frontmatter against agentskills.io v1: `name` (1-64 chars, kebab-case, matches dir name); `description` (1-1024 chars, no `<>`); optional `license`, `compatibility` (≤500 chars), `metadata` (free-form, conventional keys `version`/`author`), `allowed-tools` (experimental, space-separated). Add `compatibility:` lines per skill that honestly describe scope-creep refusal:
   - Internal skills (b-plan-runner, orchestrate, planning-workflow, idea-intake, impact-check, dashboard-handoff): `compatibility: "Requires .elai_cc/ layout, local-rag MCP server, FSM state machine"`
   - Publishable skills (anti-spiral, oracle, prompt-transform): `compatibility: "Designed for Claude Code. Works on Codex with explicit $<skill-name> invocation."`
   - Add `metadata: { version: "1.0.0", author: "elai" }` to every skill.
   This is a SKILL.md frontmatter edit only — no body changes. Cited grounding: agentskills.io/specification.

2. Per-skill `eval/` directory with positive + negative trigger corpora. For each `.claude/skills/<skill>/`, add `eval/positive.jsonl` (10-20 prompts that SHOULD trigger the skill) and `eval/negative.jsonl` (10-20 prompts that SHOULD NOT trigger). JSONL format: `{prompt: "...", expected_trigger: bool, rationale: "..."}`. Cited grounding: developers.openai.com/blog/eval-skills (3-pillar methodology: function/directive/efficiency, min 10-20 prompts × 3-5 trials), darkrishabh/agent-skills-eval (open-source runner).

3. `skill-eval-runner` tool. New CLI subcommand `elai skills eval [--skill <name>] [--target claude|codex]` that iterates each skill's `eval/*.jsonl`, runs the prompt through a configurable harness (default: dry-run through the local sidecar to test trigger detection), and reports precision/recall per skill. Output `workflow/audits/E-43-skill-eval-2026-05-19.md` with one table row per skill. Soft-gate: any skill with precision <90% OR recall <80% emits `skill_eval_below_threshold` SSB row but does NOT block. Auditor consumes the artifact during plan-end audit. Cited grounding: vercel.com/blog/agents-md-outperforms-skills-in-our-agent-evals (53% vs 79% vs 100% — quantify ELAI's actual rate).

4. Skill invocation telemetry hook. Add a PostToolUse hook (or extend existing hook) at `workflow/sessions/{id}/hook-executions.log` that appends `skill_invoked` events with `{skill_name, trigger_method: explicit|implicit, prompt_hash: blake3(prompt)[:16], outcome: passed|skipped|errored, ts}`. Use blake3 not blake2/sha2 per HARD RULE Anti-Stale-Training. SSB row `skill_invocation_logged` for dashboard visibility. Cited grounding: github.com/anthropics/claude-code/issues/35319 (Anthropic proposed; not yet shipped — org-of-one rolls own).

5. Publish path: `elai skills publish --target <claude|codex|gemini>`. Takes the E-22 export output at `dist/skills/` and packages for the target's intake format. Targets:
   - `claude`: identity-map (SKILL.md is the same; install to `.claude/skills/` user-dir or zip for distribution).
   - `codex`: write to `dist/codex/skills/<name>/SKILL.md` with the same body but generate an `agents/openai.yaml` sibling for any Codex-specific metadata (per paperclipped.de/en/blog/agent-skills-open-standard-interoperability). Initial-skill-catalog cap at ~8K chars — warn if exceeded.
   - `gemini`: write to `dist/gemini/skills/<name>/SKILL.md` matching `~/.gemini/antigravity/skills/` install convention.
   Verify each target's spec via WebFetch at startup before any publish (Anti-Slopsquatting — the formats may have drifted; trust the verification not the training). Cited grounding: paperclipped.de + developers.openai.com/codex/skills.

6. `mcp-scan` import gate for third-party skills. New rule file `.claude/rules/skill-import-policy.md` (CREATE) that defines: any external skill imported into `.claude/skills/` MUST pass `mcp-scan --recursive <skill-dir>` before being referenced; the scan hash + result are recorded in `workflow/skill-imports.log` (one row per import); the main LLM is blocked from referencing unscanned external skills (enforced at the PreToolUse hook level). Scan the WHOLE directory not just SKILL.md — Anthropic scanner missed `tests/`-hidden payloads (venturebeat 2026). Cited grounding: snyk.io/blog/toxicskills + github.com/snyk-labs/mcp-scan.

7. Conditional-routing update. Add to `.claude/rules/conditional-routing.md`:
   `Importing any external skill / referencing a third-party SKILL.md: load .claude/rules/skill-import-policy.md.`
   One-line edit; same pattern as the existing rulebook dispatches.

8. CLAUDE.md AGENTS.md-style reference update. Per the Vercel eval, passive context gets 100% adherence vs 53% for skills alone. ELAI's CLAUDE.md already references skills in the "Skills (mandatory workflow)" block. Audit that block to ensure every publishable skill is mentioned with a one-line trigger description; this is how skills go from 53% → 100% per Vercel. Surgical edit, ≤5 lines per skill.

9. Skill scope-creep audit. The report flags `planning-workflow` as borderline (covers plan creation, FSM execution, TDD, audit, completion) per articsledge + termdock anti-pattern guidance. Audit pass: list each skill and its declared verb(s); if any skill covers >3 verbs, write a one-line note in `workflow/audits/E-43-skill-eval-2026-05-19.md` recommending split. No code changes here — this is a positioning artifact. Cited grounding: duet.so/guides/agent-skills-101-tools-vs-mcp-vs-skills (one skill, one verb).

10. Publication readiness checklist. Before publishing any skill externally:
    - SKILL.md frontmatter passes `agentskills-validator` (build a tiny stdlib-only Python validator at `.claude/tools/agentskills_validator.py` that asserts the 6-field shape and length limits per spec).
    - Body ≤500 lines and ≤5K tokens (recommended ceiling per spec).
    - `eval/positive.jsonl` precision ≥95% on the local sidecar harness.
    - mcp-scan on the skill directory returns 0 findings.
    Output `workflow/audits/E-43-publish-readiness-2026-05-19.md` with a per-skill PASS/FAIL grade.

## WIRING_CONTRACT

produces_symbols:
  - cli::skills::eval_cmd(skill: Option<&str>, target: Target) -> Result<EvalReport>
  - cli::skills::publish_cmd(target: PublishTarget) -> Result<PublishReport>
  - tools::skill_eval_runner::run(skill_dir: &Path) -> Result<EvalReport>
  - tools::skill_eval_runner::EvalReport { precision: f32, recall: f32, n_positive: usize, n_negative: usize }
  - orchestrator::ssb::SsbActionType::SkillInvocationLogged
  - orchestrator::ssb::SsbActionType::SkillEvalBelowThreshold
  - orchestrator::ssb::SsbActionType::SkillExternalImportScanned
  - .claude/rules/skill-import-policy.md (NEW rule)
  - .claude/tools/agentskills_validator.py (NEW stdlib validator)
consumes_symbols:
  - .claude/skills/*/SKILL.md (frontmatter edits)
  - .claude/skills/*/eval/positive.jsonl, eval/negative.jsonl (NEW)
  - E-22 export shim at `crates/<skills_crate>::export::export_all` (if E-22 has shipped)
  - workflow/sessions/{id}/hook-executions.log (telemetry append target)
reachable_from:
  - crates/cli/src/main.rs::main (new `skills eval`, `skills publish` subcommands extending E-22's `skills export`)
  - PreToolUse hook (skill-import-policy enforcement)
  - PostToolUse hook (skill_invoked telemetry append)
stub_ban:
  - no blake2 or sha2 for prompt_hash (HARD RULE Anti-Stale-Training — use blake3)
  - no .unwrap() in skill_eval_runner (production code)
  - no claiming a skill is portable without `compatibility:` field set honestly
  - no publishing a skill that hasn't passed mcp-scan
  - no auto-importing third-party skills without the rule-gated scan
frontend_pair: crates/dashboard-v2/src/lib/windows/SkillsPanel.svelte (NEW or extend existing; shows per-skill eval scores + last invocation timestamp + scan status)
observable_effect:
  trigger: "elai skills eval --skill anti-spiral" on a clean tree
  expect_ssb: "skill_invocation_logged" rows for each test prompt (multi); "skill_eval_below_threshold" only if precision<90 or recall<80
  expect_artifact: workflow/audits/E-43-skill-eval-2026-05-19.md exists with at least one row per `.claude/skills/<skill>` directory containing precision, recall, sample sizes

## DETERMINISTIC_CHECK

```bash
# 1. agentskills_validator script exists and validates SKILL.md frontmatter
test -f .claude/tools/agentskills_validator.py
python .claude/tools/agentskills_validator.py .claude/skills/anti-spiral/SKILL.md
# Expected: exit 0

# 2. Every skill has `compatibility:` and `metadata.version`
for d in .claude/skills/*/; do
  rg "^compatibility:" "$d/SKILL.md" >/dev/null || { echo "missing compatibility in $d"; exit 1; }
  rg "version:" "$d/SKILL.md" >/dev/null || { echo "missing metadata.version in $d"; exit 1; }
done
# Expected: no missing-field errors

# 3. Every skill has eval/ directory
for d in .claude/skills/*/; do
  test -d "$d/eval" || { echo "missing eval/ in $d"; exit 1; }
  test -f "$d/eval/positive.jsonl" || { echo "missing eval/positive.jsonl in $d"; exit 1; }
  test -f "$d/eval/negative.jsonl" || { echo "missing eval/negative.jsonl in $d"; exit 1; }
done

# 4. skill-import-policy rule exists and is referenced from conditional-routing
test -f .claude/rules/skill-import-policy.md
rg "skill-import-policy" .claude/rules/conditional-routing.md
# Expected: 1 hit

# 5. CLI subcommands present
rg "skills eval|skills publish|EvalReport|PublishReport" .elai_cc/crates/cli/src --type rust
# Expected: ≥4 hits

# 6. SSB rows registered
rg "SkillInvocationLogged|SkillEvalBelowThreshold|SkillExternalImportScanned" .elai_cc/crates/orchestrator/src/ssb.rs
# Expected: ≥3 hits

# 7. blake3 used for prompt_hash (no blake2/sha2)
rg "blake3::hash|Blake3" .elai_cc/crates/cli/src/skills_cmd.rs .elai_cc/crates/orchestrator/src/skill_telemetry.rs 2>/dev/null
! rg "blake2|Sha256|sha2::" .elai_cc/crates/cli/src/skills_cmd.rs .elai_cc/crates/orchestrator/src/skill_telemetry.rs 2>/dev/null
# Expected: blake3 hits present; blake2/sha2 hits absent

# 8. Audit artifacts present
test -f workflow/audits/E-43-skill-eval-2026-05-19.md
test -f workflow/audits/E-43-publish-readiness-2026-05-19.md
```

## ACCEPTANCE

- [ ] All `.claude/skills/*/SKILL.md` frontmatter passes `agentskills_validator.py`
- [ ] Every skill has an `eval/positive.jsonl` and `eval/negative.jsonl` with ≥10 entries each
- [ ] `elai skills eval` runs end-to-end and emits per-skill precision + recall
- [ ] `elai skills publish --target codex` produces `dist/codex/skills/<name>/SKILL.md` + optional `agents/openai.yaml` per Codex spec
- [ ] `elai skills publish --target gemini` produces `dist/gemini/skills/<name>/SKILL.md` matching Gemini install convention
- [ ] `.claude/rules/skill-import-policy.md` exists and is referenced from `conditional-routing.md`
- [ ] PostToolUse hook appends `skill_invoked` events to `workflow/sessions/{id}/hook-executions.log` with blake3 prompt_hash
- [ ] At least the 3 publishable skills (anti-spiral, oracle, prompt-transform) have `compatibility:` lines that explicitly call out Claude-default + Codex `$<name>` invocation
- [ ] At least the 3 publishable skills pass the publication readiness checklist (mcp-scan PASS, eval precision ≥95%, body ≤500 lines)
- [ ] Skill scope-creep audit artifact lists `planning-workflow` (and any other multi-verb skill) with a recommendation note
- [ ] No `blake2` or `sha2::` import in any new file for E-43 (HARD RULE)

## Tests to add

- `crates/cli/tests/skills_eval_dry_run.rs`: `elai skills eval --skill anti-spiral --dry-run` outputs valid JSON precision/recall
- `crates/cli/tests/skills_publish_codex_layout.rs`: `--target codex --dry-run` writes the right file tree
- `crates/cli/tests/skills_publish_gemini_layout.rs`: `--target gemini --dry-run` writes the right file tree
- `crates/orchestrator/tests/skill_telemetry_blake3.rs`: prompt_hash uses blake3 not blake2/sha2
- `.claude/tools/tests/test_agentskills_validator.py` (stdlib unittest): valid SKILL.md frontmatter passes; missing `name` fails; description >1024 chars fails
- `tests/contracts/skill_import_policy_contract.rs`: skill-import-policy.md exists, referenced from conditional-routing.md, names mcp-scan
- `tests/contracts/skill_eval_dirs_contract.rs`: every `.claude/skills/<skill>` has `eval/positive.jsonl` + `eval/negative.jsonl`

## Files touched

- MODIFY: every `.claude/skills/*/SKILL.md` (frontmatter only — add `compatibility:`, `metadata.version`, `metadata.author`); ~5 LOC per file × ~12 skills
- CREATE: every `.claude/skills/*/eval/positive.jsonl` (~20 LOC per file × ~12 skills)
- CREATE: every `.claude/skills/*/eval/negative.jsonl` (~20 LOC per file × ~12 skills)
- CREATE: `.claude/rules/skill-import-policy.md` (~80 LOC)
- MODIFY: `.claude/rules/conditional-routing.md` (+1 line dispatch entry)
- MODIFY: `F:\.ELAI_workflow\CLAUDE.md` (Skills section — add one-line trigger description per publishable skill if missing; ~5 LOC delta)
- CREATE: `.claude/tools/agentskills_validator.py` (~120 LOC stdlib-only Python)
- CREATE: `.elai_cc/crates/cli/src/skills_cmd.rs` (or extend E-22's existing module; +~200 LOC for eval + publish-target codex/gemini)
- CREATE: `.elai_cc/crates/orchestrator/src/skill_telemetry.rs` (~80 LOC — append events to hook log with blake3 prompt_hash)
- MODIFY: `.elai_cc/crates/orchestrator/src/ssb.rs` (+~30 LOC for 3 new action types — bump SCHEMA_VERSION trajectory)
- CREATE: `workflow/audits/E-43-skill-eval-2026-05-19.md`
- CREATE: `workflow/audits/E-43-publish-readiness-2026-05-19.md`
- CREATE: `workflow/audits/E-43-scope-creep-audit-2026-05-19.md`
- CREATE: `workflow/skill-imports.log` (empty, gitignored — populated at runtime)
- CREATE: `workflow/reasoning/E-43-decisions.md`
- OPTIONAL: extend `.elai_cc/crates/dashboard-v2/src/lib/windows/SkillsPanel.svelte` (or create — ~150 LOC)

## Citations / Grounding

- agentskills.io/specification (Anthropic SKILL.md v1 canonical) — grounds Task 1 frontmatter shape
- developers.openai.com/codex/skills (Codex adoption, `.agents/skills/` path, `$skill-name` invocation, ~8K char catalog cap) — grounds Task 5
- paperclipped.de/en/blog/agent-skills-open-standard-interoperability (format ports, install paths don't) — grounds Task 5
- vercel.com/blog/agents-md-outperforms-skills-in-our-agent-evals (100% passive context vs 53% skills-alone) — grounds Task 3 + Task 8
- developers.openai.com/blog/eval-skills (3-pillar eval methodology, 10-20 prompts × 3-5 trials, negative-trigger tests) — grounds Task 2 + Task 3
- github.com/darkrishabh/agent-skills-eval (open-source eval runner reference) — grounds Task 3
- snyk.io/blog/toxicskills-malicious-ai-agent-skills-clawhub (13.4% critical / 36.82% any issue / 76 confirmed malicious / 91% prompt-injection in malicious) — grounds Task 6 + Task 10
- venturebeat.com Anthropic scanner bypass (payload hidden in `tests/`) — grounds Task 6 whole-directory scan
- github.com/snyk-labs/mcp-scan (90-100% recall, 0% FP claimed) — grounds Task 6
- github.com/anthropics/claude-code/issues/35319 (skill telemetry feature request, not yet shipped) — grounds Task 4
- duet.so/guides/agent-skills-101-tools-vs-mcp-vs-skills (one skill, one verb) — grounds Task 9
- E-22 in this same sweep (SKILL.md export shim) — coordinate overlap

## Open questions / risks

- `mcp-scan` tool availability: it's a snyk-labs tool; verify it's published as `npm`/`pypi`/`cargo` (and which) before relying on it. If it's only an MCP server (no standalone CLI), wire through C-39's MCPClient. Anti-slopsquatting: never assume the package layout without verification.
- Vercel eval methodology (53/79/100) was measured against a specific corpus; ELAI's procedural skills may behave differently (the report's Open Question 5). Measure ELAI's actual rate via Task 3 before claiming the same gap holds.
- Codex `agents/openai.yaml` format: report cites this as "split into a separate file"; the exact key set is implementation-dependent. Verify via WebFetch before emitting; if uncertain, ship `--target codex` behind a feature flag with explicit "format unverified" warning.
- `planning-workflow` scope-creep: the report flags it but does not mandate a split. E-43 only writes the audit recommendation; splitting is a follow-up E-N if the user agrees.
- Skill context-budget cost: ~100 tokens × 12+ skills = ~1.2K tokens permanent overhead per session. Worth measuring against actual invocation frequency via Task 4 telemetry; if any skill never fires across N sessions, candidates for removal.
- Anthropic publishing in Dec 2025; the spec may have evolved by May 2026. Re-verify agentskills.io/specification at task start before Task 1 frontmatter edits.

## Next

E-43 unblocks publishing 3 skills to public marketplaces (agentskills.io, awesome-agent-skills) as a flag-plant move; the SKILL.md format is identity-mapped for Claude, dist-formatted for Codex/Gemini via Task 5. Task 4's telemetry feeds back into the eval harness — after 10+ sessions of data, the audit can surface dead-weight skills for trimming. The `planning-workflow` split (if user agrees per Task 9 audit) should become a separate E-N. Any future third-party skill import MUST go through `.claude/rules/skill-import-policy.md` — this is the supply-chain hardening that the ToxicSkills landscape demands.
