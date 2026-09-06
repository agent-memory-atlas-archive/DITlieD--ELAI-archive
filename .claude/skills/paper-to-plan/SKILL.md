---
name: paper-to-plan
description: End-to-end paper/repo steal pipeline. Triggers when the user drops one or more paper links (arXiv/DOI/PDF) or repo links (GitHub/GitLab) and asks whether/how they can improve ELAI, or says "paper-to-plan", "ingest this and plan", "steal from this", "can elai use this". One session replaces the old 4-session manual flow: ingest, then repo map, then ELAI context, then steal analysis, then new plan files + index updates.
license: MIT
compatibility: "Designed for Claude Code. ELAI workflow root only."
metadata:
  version: "1.0.0"
  author: "elai"
---

The user drops a paper or repo link. This skill turns it into authored ELAI plan files in ONE session, with the main model doing only the high-judgment work (steal reasoning, plan authoring) and agents doing all the bulk reading.

WHAT THIS REPLACES

The manual flow was 4 sessions: (1) opus session spawning sonnet paper-ingestors, (2) opus session mapping the repo with sonnet explorers, (3) fable/opus session reading ~165K tokens of plan summaries then reasoning about steals into a file, (4) fresh session reading that file + the plan template and authoring plan files. The expensive part - the 12-file context load - is replaced by ONE file: docs/thefinalglance/ELAI-FEATURE-MAP.md.

INPUTS

- one or more links: papers (arXiv ID/URL, DOI, PDF URL) and/or repos (git URL)
- optional: target plan family letter (default: the highest letter that has a <X>-0_INDEX.md under docs/thefinalglance/)
- optional: user budget/scope constraints in the same message

MODEL LADDER + COST DISCIPLINE (binding)

- haiku: bulk read/summarize work - repo partition explorers. Never the session model for bulk reads.
- sonnet: canonical agents (paper-ingestor) and repo synthesis/mapping. All canonical .claude/agents/*.md dispatches run model=sonnet per CLAUDE.md.
- main session model (fable/opus): steal analysis and plan authoring, in-band, never delegated - source fidelity matters more than token economy here.
- MAX 3 AGENTS IN PARALLEL at any moment (rate limit). Queue the rest.
- RULE INJECTION: every agent prompt begins with the pointer line from .claude/rules/agent-dispatch-preamble.md (binding worker rules: git read-only, no destructive ops, scope contract, BLOCKED=handoff-entry protocol) plus handoff_file=workflow/handoff/paper-to-plan-<slug>.md. Sub-agents do not reliably load CLAUDE.md; the pointer is the guarantee.
- If the whole run needs more than 2 agents total: announce agent count + model per agent + rough token cost ONCE at phase 0 and get the user's yes. That single yes covers all phases of this run. Re-announce only if the fleet grows beyond the announced plan (e.g. repo turned out 4x bigger).

PHASE 0 - INTAKE + ANNOUNCE

1. Parse links; classify each as paper or repo.
2. Resolve target plan family: user-named letter, else highest existing OPEN letter (Glob docs/thefinalglance/*-plan/*-0_INDEX.md). CLOSED families are never a target for new plan files: J is CLOSED as of 2026-07-10 (oversized grab-bag, already the largest plan) — never author a new J-N file, and an EXTENSION steal whose parent unit lives in J lands in the current open letter cross-linked to the J unit, not in J. Current open letter = P. If the resolved letter is itself oversized (its folder already dwarfs its siblings), open the NEXT letter rather than growing it.
3. For each repo: shallow-clone to docs/.source/<repo-name> (git clone --depth 1). Estimate tokens = total bytes of source files / 4, excluding .git, node_modules, vendored dirs, lockfiles, binary assets.
4. Compute the fleet: 1 sonnet paper-ingestor per paper + per repo (mode=source), plus repo mapping agents per phase 2 sizing, plus 1 sonnet synthesizer per large repo.
5. Announce fleet (count, models, rough token cost) and get yes if >2 agents. Then proceed without further questions.

PHASE 1 - INGEST (parallel, cap 3)

Per paper: dispatch 1 sonnet agent: subagent_type=paper-ingestor if available in the Agent tool, else general-purpose with prompt "Read /workspace/ELAI\.claude\agents\paper-ingestor.md and follow it strictly. mode=paper, target=<arXiv id or url>". The agent fetches paper + figures and writes workflow/papers/<slug>-interface.md.
Per repo: same agent, mode=source, pointed at the clone path. Writes workflow/papers/<repo-slug>-interface.md with reliability level.
Do not block phase 2 on phase 1 finishing for OTHER links; pipeline per link, respect the cap.

PHASE 2 - REPO MAP (repos only)

Sizing from phase 0 token estimate:
- under 200K tokens: 1 sonnet mapper agent reads the clone and writes docs/.source/<repo-name>-INDEX.md (structure, entry points, key modules, what each does, what is portable).
- 200K tokens or more: ceil(tokens/100K) haiku explorer agents (announced in phase 0), partitioned by directory subtree so partitions do not overlap; each writes workflow/papers/<repo-slug>-part-N.md. Then 1 sonnet synthesizer merges the parts into docs/.source/<repo-name>-INDEX.md.
- over 1M tokens: stop and ask the user to scope subdirectories. Do not launch a 10+ agent fleet on a monorepo.

PHASE 3 - ELAI CONTEXT (main model, one file)

Read docs/thefinalglance/ELAI-FEATURE-MAP.md directly. This is the single context-load; do NOT read the legacy 12-file list. For the DESIGN-topology of a subsystem a steal touches (what the capability is, as designed, no status), consult the matching section of docs/thefinalglance/ELAI-SPEC-OVERVIEW.md — read only the relevant letter/phase section, not the whole file.
Staleness check: if the map's header date is older than the newest *-SUMMARY_INDEX.md mtime under docs/thefinalglance/, warn the user and offer a refresh (dispatch 1 agent with .claude/skills/paper-to-plan/feature-map-regen.md; large-context model required - it reads ~165K tokens). Proceed with the stale map unless the user opts to wait.
Fallback ONLY if the map file is missing: the legacy list (phase-index-MASTER.txt; B-PLAN-SUMMARY.md; C/D/E/F/G/H/I/J-SUMMARY_INDEX.md; workflow/handoff/wire-remediation.md head; workflow/sessions/s-plan-audit-2026-06-04/FINISH/_MASTER-INDEX.md) - and in that case recommend regenerating the map first.

PHASE 4 - STEAL ANALYSIS (main model, in-band)

Read the phase 1 interface maps and phase 2 repo index. Write workflow/reasoning/steal-<slug>-<YYYY-MM-DD>.md. Per candidate steal:
- what it is, citing the paper section or repo file path
- which ELAI subsystem it touches, citing the feature-map entry
- verdict: NEW | EXTENSION(of plan-id) | ALREADY-SHIPPED(plan-id) | ALREADY-REJECTED(ledger entry) | REJECT(reason)
Check the feature map's REJECTED/ALREADY-TRIED ledger before any NEW/EXTENSION verdict - re-proposing a ledger entry is a P0-grade miss.
Verify every arXiv ID via WebSearch before citing it in a plan (house rule; 2 bad cites shipped in B-40/B-42 before this rule).
Wiring posture: ELAI is wired as plans are implemented. Do NOT generate steals or plan framing around "ELAI wiring is incomplete"; a feature's status flag is context, not a work item.

ENGINE SEAM (improvement-conjecture): candidate-steal generation runs THROUGH the improvement-conjecture engine (.claude/skills/improvement-conjecture/SKILL.md), not freestyle. Stage 1 diverges candidate improvement conjectures across (paper/repo mechanisms x ELAI target areas); stage 2 is the deterministic ledger + anchor + demarcation triage (`python .claude/tools/conjecture_triage.py --run <artifact> --json`), which SUBSUMES the manual ledger-check above and records each kill with its killer. The steal file and the engine run artifact are ONE document — use workflow/reasoning/conjecture-run-<slug>-<YYYY-MM-DD>.md as the steal file (the separate steal-<slug>.md is subsumed), whose PICK section carries the steal verdict per surviving candidate (NEW | EXTENSION(plan-id) | ALREADY-SHIPPED(plan-id) | ALREADY-REJECTED(ledger) | REJECT(reason)). Stage-4 survivors (top-k, each with an executed micro-experiment or a cited estimate; a judge-only candidate stays pending and cannot win) flow into PHASE 5 below UNCHANGED. The engine never authors plan files; PHASE 5 does.

PHASE 5 - PLAN AUTHORING (main model, in-band)

1. Read the target family's <X>-0_INDEX.md + <X>-SUMMARY_INDEX.md + docs/thefinalglance/PLAN-FOLDER-TEMPLATE.md.
2. Author the new plan files from the steal file's NEW/EXTENSION verdicts. FEWEST FILES POSSIBLE: group related steals into one plan file; only split when scopes genuinely do not share a subsystem.
3. Every plan unit is a vertical slice per .claude/rules/wiring-enforcement.md PRE_GATE: producer + in-plan consumer + smoke trigger in the same unit. New default-OFF flags need a wire-dark-allowlist row with promotion plan.
4. Update BOTH index files (<X>-0_INDEX.md and <X>-SUMMARY_INDEX.md) with the new entries.
5. FEATURE-MAP UPSERT (mandatory): the feature map is the source of truth for ELAI features + their status and how they compose - it grows with every plan, it is never allowed to go stale. Add each new plan unit to docs/thefinalglance/ELAI-FEATURE-MAP.md as a PLANNED(plan-id) entry in the right subsystem group with its composes/extends cross-links; add this run's REJECT/ALREADY-REJECTED verdicts to the REJECTED/ALREADY-TRIED ledger; update the plan-family ledger line; bump the header date.
6. SPEC-OVERVIEW UPSERT (mandatory): docs/thefinalglance/ELAI-SPEC-OVERVIEW.md is the DESIGN-topology map — the CAPABILITY each plan adds, judged as if built, with NO wiring/shipped/planned status, NO plan-id ledger lines, NO schema/SSB/effort counts. The two docs are complementary and BOTH must stay current: the feature map is the build ledger (status + plan-ids), the spec overview is the design argument (capabilities + how they compose). For each NEW capability authored this run: extend the matching letter/phase section's paragraph with a one-clause design description in the file's own voice (design language only — never a status flag or plan-id). Add a new `## X — <name>` section ONLY when this run opened a new letter. Do NOT record REJECT verdicts here (the spec is design-as-if-built; a rejected capability was never designed in — rejections live in the feature map ledger only). If a genuinely new capability THEME emerged, also refresh the "The shape of the whole" summary line for that letter. The spec has no header date to bump.
7. Append a one-line outcome (plan ids created) to the steal file.

PHASE 6 - WRAP

Report to the user: plan files created (paths), index lines added, steal verdicts summary (N new, N extensions, N rejected and why in one line each). No commit - docs/ and workflow/ are gitignored.
If context runs low mid-run: write a handoff per .claude/rules/handoff-protocol.md at workflow/handoff/paper-to-plan-<slug>.md and tell the user where to resume.

FAILURE MODES

- paper fetch fails: retry once with an alternate source (arXiv abs vs pdf, ar5iv, semanticscholar); if still failing, note it in the steal file and continue with the rest.
- repo clone fails: report and continue with papers.
- rate limit hit mid-fleet: drop parallelism to 1, keep going.
- agent returns garbage/empty: re-dispatch once with the same prompt; twice failed = note and continue.

WHEN NOT TO INVOKE

- the user asks a question ABOUT a paper without wanting plan files ("what does this paper claim") - that is oracle/paper-ingestor territory, not this pipeline.
- the link targets a different project root than ELAI.
- the user explicitly wants only ingestion ("just ingest this for RAG") - dispatch paper-ingestor alone.
