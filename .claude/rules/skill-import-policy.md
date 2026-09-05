Skill import policy (E-43, HARD RULE)

Loaded by: any task that imports a third-party SKILL.md into .claude/skills/, or that references an external skill identifier the main LLM has not authored.

Background

Snyk's ToxicSkills study (Feb 2026, snyk.io/blog/toxicskills-malicious-ai-agent-skills-clawhub) scanned 3,984 publicly available agent skills and found:
- 13.4% had critical-severity issues
- 36.82% had at least one security issue
- 76 confirmed malicious skills (91% used prompt injection)
- The ClawHavoc campaign alone published 1,184 malicious skills from 12 accounts

Anthropic's official scanner was demonstrated bypassable (venturebeat 2026) when payloads were placed in `tests/` directories alongside SKILL.md. Whole-directory scanning is mandatory; SKILL.md-only inspection is insufficient.

Policy

Any external skill (any SKILL.md NOT authored locally inside this repo's .claude/skills/) MUST satisfy ALL of the following before any reference to it is permitted:

1. The entire skill directory MUST be scanned recursively (the whole directory, NOT just SKILL.md). The CONCRETE scanner on this machine is skillspector (NVIDIA/skillspector, Apache-2.0, v2.1.2), run via /skill-audit (wrapper .claude/tools/skill-audit.py, under the skillspector venv python). mcp-scan was NEVER installed here (verified 2026-06-10: `which mcp-scan` -> not found), so skillspector is the runnable replacement everywhere this policy previously said `mcp-scan --recursive <skill-dir>`. skillspector scans the whole package (its 16 categories include the tests/-directory payload class that bypassed Anthropic's scanner). RUN AT SKILL-PACKAGE GRANULARITY, never a whole upstream repo (a full app always scores 100/CRITICAL = noise). Either scanner's dist-channel must be verified per the anti-slopsquatting protocol in CLAUDE.md before first use.

2. The scan result MUST be recorded in workflow/skill-imports.log as one row per import. Schema (TSV):
   ts_unix_ms<TAB>skill_name<TAB>skill_dir<TAB>scan_tool<TAB>findings_count<TAB>scan_hash<TAB>action

   scan_hash is the blake3 prefix of the scanner output (blake3 -- NOT blake2 or sha2, per CLAUDE.md HARD RULE Anti-Stale-Training). action is one of: ALLOWED / BLOCKED / NEEDS_REVIEW.

3. The main LLM is BLOCKED from referencing any external skill whose import has not been recorded in workflow/skill-imports.log with action=ALLOWED. PreToolUse hook enforcement is deferred to a follow-up plan (see open-question below); until then this is a soft-gate documented here and enforced by code review at audit time.

4. If the scanner reports any HIGH/CRITICAL findings, the import is BLOCKED. The blocked row stays in workflow/skill-imports.log with action=BLOCKED and the main LLM may not reference the skill, period.

5. Scope-creep refusal: an external skill that covers >3 verbs is treated as suspicious. The reviewer notes the verb count in workflow/audits/E-43-scope-creep-audit-2026-05-27.md and prefers a split before allowing.

What counts as an "external skill"

- Any directory copied or symlinked into .claude/skills/ from outside this repo's git history.
- Any reference to a SKILL.md path that resolves outside this repo.
- Any registry / marketplace install of a skill (e.g. `mcp-scan install <name>` against agentskills.io, awesome-agent-skills, etc.).

What is NOT external

- The 11 ELAI-authored skills shipped under .claude/skills/ in this repo. Their provenance is the git log; no scan required for self-authored content.
- Modifications to ELAI-authored skills by the main LLM during normal plan execution (not an import).

Open questions / deferred

- RESOLVED 2026-06-10: the scanner is no longer text-only. skillspector is installed and reused from the liteGAME clone venv (D:/.liteGAME_workflow/workflow/external-skills/repos/skillspector/.venv, python 3.13, same machine no reinstall); /skill-audit + .claude/tools/skill-audit.py are the runnable gate. mcp-scan dist-channel verification is moot (mcp-scan not used).
- PreToolUse hook enforcement that BLOCKS Edit/Write/Bash when an unscanned external skill is referenced. Still deferred; until then this is a soft-gate enforced by /skill-audit + the workflow/skill-imports.log row at audit time.

Grounding

- snyk.io/blog/toxicskills-malicious-ai-agent-skills-clawhub (ToxicSkills landscape)
- venturebeat.com (Anthropic scanner bypass via tests/-directory payloads, 2026)
- github.com/snyk-labs/mcp-scan (90-100% recall, 0% FP claim; NOT installed on this machine)
- github.com/NVIDIA/skillspector (the concrete scanner in use; 64 patterns / 16 categories; static + optional LLM stage)
- agentskills.io/specification (the SKILL.md v1 spec the policy gates against)
