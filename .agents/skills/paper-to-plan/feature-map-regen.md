FEATURE-MAP REGEN PROMPT (reusable agent task)

You are authoring or refreshing F:\.ELAI_workflow\docs\thefinalglance\ELAI-FEATURE-MAP.md.
This file is the SOURCE OF TRUTH for what ELAI's features are, their status, and how they compose with each other. It is the SINGLE context-load for "can this paper/repo improve ELAI" sessions, replacing the legacy 12-file read (~165K tokens). Optimize for FEATURE LOOKUP by a model evaluating external ideas, not for plan history.
It is a LIVING file: after bootstrap it is upserted in-place at plan authoring (paper-to-plan phase 5) and at phase completion (planning-workflow MEMORIZING / orchestrate step 14). Wholesale regeneration via this prompt is for bootstrap or irreparable drift only - it must never silently discard upserted entries; diff against the existing map before overwriting.

INPUTS (read in this order, sizes approximate)

1. F:\.ELAI_workflow\docs\thefinalglance\ALL-PLANS-MASTER-SUMMARY.md (47KB) - existing condensed skeleton. Use as base; verify against the sources below, do not re-derive from scratch.
2. F:\.ELAI_workflow\workflow\sessions\s-plan-audit-2026-06-04\FINISH\_MASTER-INDEX.md (21KB) plus the per-plan FINISH files in the same folder (B 26KB, C 40KB, D 17KB, E 36KB, F 13KB, G 9KB, base-phase-0..4) - the wired/dark/partial status ground truth per plan unit.
3. The plan summary indexes, mined ONLY for what the skeleton lacks (code anchors, key types, SSB action_types, rejections, doctrine):
   F:\.ELAI_workflow\docs\thefinalglance\B-plan\B-PLAN-SUMMARY.md (64KB)
   F:\.ELAI_workflow\docs\thefinalglance\C-plan\C-SUMMARY_INDEX.md (68KB)
   F:\.ELAI_workflow\docs\thefinalglance\D-plan\D-SUMMARY_INDEX.md (28KB)
   F:\.ELAI_workflow\docs\thefinalglance\E-plan\E-SUMMARY_INDEX.md (46KB)
   F:\.ELAI_workflow\docs\thefinalglance\F-plan\F-SUMMARY_INDEX.md (24KB)
   F:\.ELAI_workflow\docs\thefinalglance\G-plan\G-SUMMARY_INDEX.md (21KB)
   F:\.ELAI_workflow\docs\thefinalglance\H-plan\H-SUMMARY_INDEX.md (129KB)
   F:\.ELAI_workflow\docs\thefinalglance\I-plan\I-SUMMARY_INDEX.md (89KB)
   F:\.ELAI_workflow\docs\thefinalglance\J-plan\J-SUMMARY_INDEX.md (25KB)
4. F:\.ELAI_workflow\docs\specs\guide\steps\phase-index-MASTER.txt (81KB) - base-build (phase 0..4M) subsystem anchors.
5. F:\.ELAI_workflow\workflow\handoff\wire-remediation.md - read the HEAD ONLY (first ~40 lines: CURRENT STATUS + INDEX). Do NOT read the body. The map carries final status flags, never remediation narrative.
6. Live code ground truth for the schema stamp: grep SCHEMA_VERSION in F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\ssb.rs and TOTAL_VARIANTS in F:\.ELAI_workflow\.elai_cc\crates\orchestrator\tests\action_type_lock.rs. The live grep beats every doc number.
7. Spot-check at most 6 original plan detail files (docs/thefinalglance/<X>-plan/<X>-N*.md) ONLY where two sources conflict on what a feature is or where it lives.

OUTPUT SPEC - ELAI-FEATURE-MAP.md

Hard size target: 1200-1800 lines, roughly 15-25K tokens. Denser than the master summary. Plain text, ALL-CAPS section headers, no bold, no emoji, every line carries meaning.

HEADER: one purpose line (SOURCE OF TRUTH for ELAI features + their composition; single context-load replacing the legacy 12-file list); date stamp; LIVE SCHEMA_VERSION / TOTAL_VARIANTS stamp with the file:line it was grepped from; regen pointer (this prompt file, dispatch one agent with it); MAINTENANCE CONTRACT (verbatim): "LIVING FILE - upsert in-place, never let it go stale: plan authoring adds PLANNED entries + rejections (paper-to-plan phase 5); phase completion flips status flags + cross-links (planning-workflow MEMORIZING, orchestrate step 14). One flag per feature, never narrative. Bump this date on every upsert."

SECTION 1 - SUBSYSTEM CATALOG (the core, ~60% of the file). Group by CAPABILITY, never by plan letter:
  knowledge/retrieval; reasoning/GRC/grounding-modes; orchestration/FSM/agents/BSP; SSB/telemetry/observability; context-lifecycle/memory/handoff; verification/trust-gates/wiring-enforcement; self-improvement/DGM/rules-engine; security/sandbox/injection-defense; inference local-sidecar/cloud-routing/quantization; dashboard/canvas/UX; CLI surface; pipeline-factory/planning; benchmarks/eval.
Per feature ONE compact entry:
  NAME [STATUS] what it does + the key design decision; code anchor (crate/module path, or planned location); plan-id(s); SSB action_type / feature flag if any; cross-links - composes/extends/consumed-by (feature names or plan-ids) whenever a feature builds on, feeds, or gates another. The interaction edges are FIRST-CLASS content (this map is the source of truth for how features work together), mine them from the indexes ("extends C-21", "composes F verdicts", "replaces MSI sigmoid" and the like).
STATUS enum: WIRED | DARK(flag) | PARTIAL | PLANNED(plan-id) | DEFERRED | BENCH-PENDING. Status comes from the FINISH files and newest indexes; copy the flag, never the story.

SECTION 2 - PLAN-FAMILY LEDGER: one line per letter/track (theme, ID range, status counts).

SECTION 3 - DOCTRINE FOR NEW PLANS: the W/GD/HD/TD/VD/ID/IC/JD one-liners plus only the house rules that bind plan AUTHORING (vertical-slice PRE_GATE, Pattern-2 SSB variants, wire-dark-allowlist row requirement, citation verification). Do not restate CLAUDE.md content the session already loads.

SECTION 4 - REJECTED / ALREADY-TRIED LEDGER (high value, be exhaustive): every explicit rejection, decline, defer, hard-avoid across all plans with a one-line reason. Known members to capture (non-exhaustive, mine the indexes for the rest): swiftide; Gemma-4; Qwen3-32B routing; gajae provider layer wholesale; batch-racing vs Batch API; ciphers/abbreviation context compression; ast-grep dep; Lean; DPO; LoRA (E-47); speculative decoding phase-1; SPLADE early-drop conditions; the 13 I-plan Track-20 declines; FA3/FA4 on Ampere; SWE-bench Verified as headline metric. Purpose: a steal-analysis session must NEVER re-propose these.

SECTION 5 - EXTENSION POINTS: the canonical ways a new feature attaches to ELAI, each with where it lives and what a plan unit must include:
  new SSB action_type (Pattern-2 single-variant + golden lock bump); FsmSubstate; CLI subcommand; dashboard window/panel + ts-rs bindings pair; sidecar endpoint + Rust client pair; feature flag + wire-dark-allowlist.toml row + promotion plan; contracts test family; .claude harness hook/skill/tool (J-family, JD-1..4 evidence rules).

EXCLUSIONS (hard): session narratives, audit war stories, wiring-remediation history, token costs, per-session dates, CLAUDE.md restatement, motivational prose. A feature's wiring state is ONE status flag, never a paragraph. The phrase "wire-dark" may appear only inside a status flag or the allowlist extension point.

METHOD / CONTEXT DISCIPLINE

Check file sizes before reading. Read large files in chunks. After each source, append distilled notes to F:\.ELAI_workflow\workflow\sessions\featuremap-scratch\<source-slug>.md so the work survives context pressure; assemble the final map from the notes.
Conflict rule: newest wins. Live code grep > FINISH index (2026-06-04) > newer summary indexes > older ones. List anything unresolved at the bottom of the map under OPEN CONFLICTS.
After writing the map, add ONE pointer line at the very top of ALL-PLANS-MASTER-SUMMARY.md: "SUPERSEDED for steal-evaluation context-load by ELAI-FEATURE-MAP.md; this file remains a per-plan digest." Change nothing else in it.

RETURN (your final message, raw data): map path; line count; byte size; estimated tokens; section list; the top 5 content classes you dropped; conflicts found and how resolved.
