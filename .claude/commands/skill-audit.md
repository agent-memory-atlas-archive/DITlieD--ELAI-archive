Run the skill-security gate (NVIDIA skillspector) on an agent skill package before promoting it into .claude/skills/.

USAGE
  /skill-audit <path-to-skill-package> [--llm]

WHAT IT DOES
Scans a SKILL package (a directory containing SKILL.md plus any scripts/references, or a single SKILL.md) for the 64 skillspector patterns across 16 categories: prompt injection, data exfiltration, privilege escalation, supply chain, excessive agency, output handling, system-prompt leakage, memory poisoning, tool misuse, rogue agent, trigger abuse, dangerous AST calls, taint flow, YARA malware signatures, MCP least-privilege, MCP tool poisoning. Prints score (0-100), severity, recommendation, and every finding with rule id + file:line. Static-only by default (no key, deterministic); --llm adds the semantic stage to filter false positives (needs SKILLSPECTOR_PROVIDER + creds).

HOW TO RUN (the wrapper must run under skillspector's venv interpreter)
On this machine the skillspector venv is the liteGAME clone (same machine, reused, no reinstall):
  & "D:/.liteGAME_workflow/workflow/external-skills/repos/skillspector/.venv/Scripts/python.exe" "/workspace/ELAI/.claude/tools/skill-audit.py" "<path-to-skill-package>"
Add --llm as a final arg for the semantic stage. Exit 0 = PASS (score<=50), exit 1 = BLOCK (score>50).

RELATIONSHIP TO skill-import-policy.md (E-43)
This is the CONCRETE implementation of the scanner that .claude/rules/skill-import-policy.md mandates. That rule named mcp-scan, which is NOT installed on this machine (verified 2026-06-10), so the policy was a text-only soft-gate with no runnable scanner. skillspector fills that gap. Per the policy you MUST record each import in workflow/skill-imports.log, one TSV row:
  ts_unix_ms<TAB>skill_name<TAB>skill_dir<TAB>scan_tool<TAB>findings_count<TAB>scan_hash<TAB>action
scan_tool = skillspector-2.1.2 ; scan_hash = blake3 prefix of the wrapper output ; action = ALLOWED / BLOCKED / NEEDS_REVIEW. A HIGH/CRITICAL finding is action=BLOCKED and the skill may not be referenced. skillspector replaces mcp-scan everywhere the policy says "mcp-scan --recursive".

GRANULARITY (the load-bearing caveat, do not get this wrong)
Point it at the SKILL PACKAGE, never a whole upstream tool repo. A full application repo always scores 100/CRITICAL because real software legitimately uses subprocess/network/fs/eval, so the score is meaningless at repo granularity. At skill-package granularity it is real signal: a clean skill scores 0/SAFE, and a skill that shells out to external binaries or carries credential-scraping reference docs lights up HIGH/CRITICAL. The skillspector terminal renderer (rich) crashes on this Windows console, which is exactly why this wrapper uses the Python API (graph.invoke) instead of the bare `skillspector scan` CLI.

WHEN TO RUN
- Before copying ANY third-party skill into .claude/skills/ (gate every external skill on a clean-enough static pass first).
- Before promoting any skill cloned from a marketplace or upstream repo.
- Periodically on .claude/skills/ to catch drift (a skill edited to add a dangerous instruction).
- NOT required for the ELAI-authored skills already shipped under .claude/skills/ (their provenance is the git log).

READING THE RESULT
A BLOCK is not an automatic reject, it is a triage trigger. Re-run with --llm to drop false positives, then for each surviving HIGH/CRITICAL decide: expected-for-purpose (e.g. a research skill that legitimately runs yt-dlp), prunable (e.g. unused mixed-script triggers), or genuinely malicious (strip it or do not promote). Only promote once the residue is understood. Treat YARA malware matches and credential/exfil taint chains as serious until proven benign.

CONTEXT
skillspector is a standalone CLI that never runs inside the agent (zero integration risk). Reused from the liteGAME clone at D:/.liteGAME_workflow/workflow/external-skills/repos/skillspector/.venv (python 3.13). Upstream: NVIDIA/skillspector, Apache-2.0, v2.1.2. Wrapper: .claude/tools/skill-audit.py.
