# J-HARNESS-GROUNDING-MAP — single-track grounding for J-1..J-7
Grounded 2026-06-10, session s-jplan-author (main LLM direct, no agents per user instruction).
Method: 14-file pre-read (log: workflow/reasoning/J-plan-prereading-2026-06-10.md) + live glob/grep
sweep over the harness + targeted ssb.rs/action_type_lock.rs greps. Every J detail file cites these
IDs instead of re-asserting facts (R-a). PLAN-asserted anchors are labelled; executors re-grep (W-7).

================================================================================
HG — HARNESS GROUND TRUTH (glob/grep-confirmed 2026-06-10, this session)
================================================================================
HG-1  .claude/tools/ holds 26 stdlib-only python tools (impact_radius, wiring_contract_check,
      gate_freeze_check, bn_status_report, spec_link_check, agentskills_validator, cross_lang_diff,
      failure_taxonomy, codebase_mapper, ...). NO undo.py, NO handoff_ledger.py, NO backup tooling.
      J-3/J-4 are net-new builds, not activations.
HG-2  .claude/skills/ holds exactly 11 skills (anti-spiral, prompt-transform, dashboard-handoff,
      impact-check, rag-tools, sidecar-protocol, oracle, idea-intake, planning-workflow,
      b-plan-runner, orchestrate). NO incremental-eval, NO see. J-6 and the J-1 harness leg are
      net-new skills.
HG-3  the live hook seam is C:/Users/user/.claude/rag/hooks/pre_tool_check.py (a dated backup copy
      exists at rag/_backups/20260610-123838/). Registration via .claude/settings.local.json
      (PLAN-asserted from CLAUDE.md + handoff-protocol text; executor re-reads settings.local.json
      before wiring J-3's hook). Hook families per CLAUDE.md: PreToolUse / PostToolUse / Stop /
      PreCompact, logging to workflow/sessions/{id}/hook-executions.log (fallback
      workflow/hook-executions.log).
HG-4  PostToolUse change-recording to per-session state EXISTS: the wire-remediation handoff
      (2026-06-06 entry) edited workflow/sessions/state.json changes[] to clear a Stop-hook
      false-positive — proof the harness already records per-session file-change state J-4 can
      derive from. Exact schema PLAN-asserted; J-4's tool reads defensively (state.json AND
      hook-executions.log shapes, fail-visible).
HG-5  docs/thefinalglance/J-plan/ did not exist before this session (glob). Letters B..I taken;
      J is the next allocator slot. No numbering collision.
HG-6  the stale-F:-drive-hook bug is REAL and DELIBERATELY unfixed: pre_tool_check.py
      _SRC_PREFIXES lacks F: prefixes so the FSM gate is a no-op for F: paths; an agent auto-fix
      was REVERTED byte-faithful as an R8 violation (wire-remediation 2026-06-06). The fix is the
      user's deliberate call. J-3 must NOT slip it in (JD-4) and must name it as an open question.

================================================================================
RT — RUNTIME (.elai_cc) GROUND TRUTH
================================================================================
RT-1  LIVE SCHEMA_VERSION = 221 (crates/orchestrator/src/ssb.rs:2981, i64; SCHEMA_VERSION_PUBLIC
      re-export :2986) / TOTAL_VARIANTS = 532 (crates/orchestrator/tests/action_type_lock.rs:601).
      Grep-confirmed 2026-06-10. Matches the I-plan baseline. NEVER hard-code; re-read at edit (W-9).
      History (wire-remediation): 234/511 -> 236/513 -> 240/517 -> 209/520 -> 522 -> 221/532 —
      snapshots go stale within days.
RT-2  PLAN-asserted anchors carried from the redesign doc [SRC-1], executor re-greps per W-7:
      file_ops.rs:272/300 (pre-image captured then discarded), task_runner.rs:7871
      (editor_fs_override=None), runtime_builder.rs:306 (streaming validator None),
      conversation.rs:22 (flat 100K auto-compact), task_runner.rs:3898 (PreCompact no-op stub).
RT-3  crates/cli/src/mcp_client.rs + mcp_glue.rs exist in the live tree (git status 2026-06-10
      modified list) = C-39's CLI->MCP-client direction partially landed. J-2's server is the
      missing inverse direction. C-39's SSB types (cli_mcp_dispatch / _server_started /
      _server_stopped) are the prefer-reuse candidates for J-2 (verify shipped at edit).
RT-4  B-25 ElaiNeuron capability list (task_submit, task_status, pipeline_result, health_check)
      is authored but DEFERRED on Empire access (C-34 external blocker). J-2 reuses the list minus
      the Empire transport; B-25 stays deferred and untouched.
RT-5  security spine status for J-2's PRE_GATE (FINISH 06-07b static re-audit, MOVING — re-verify
      at execution): E-24 schema pin SHIPPED (mcp_schema_pin.rs per G-4 text); G static tally
      8 wired / 2 dark (G-4 re-pin gate status must be re-confirmed live); E-30 argv allowlist +
      G-3 blast-radius listed already-wired-needing-fixtures; C-42 authz attrs C-plan status
      unverified this pass.

================================================================================
SRC — SOURCE DOCUMENTS (the track's birth papers)
================================================================================
SRC-1 workflow/reasoning/harness-redesign-recommendations-2026-06-10.md (Sections A-D; the six
      N items) + companion workflow/reasoning/harness-redesign-reading-2026-06-10.md (the [eN]
      per-file reading log every claim is grounded in).
SRC-2 workflow/reasoning/ext-repo-steal-recommendations-2026-06-10.md (rev 2) + companion
      workflow/reasoning/ext-repo-eval-2026-06-10.md; skillspector interface map at
      workflow/papers/skillspector-interface.md; repo clones at
      D:\.liteGAME_workflow\workflow\external-skills\repos\<name> (summaries in ..\_summaries\).
SRC-3 vlm-vision-research-summary.md + Screenshot_42.png (J-1 and J-6 source material; figures in
      it are EXTERNAL, never ELAI-measured — ID-5 binds).
SRC-4 workflow/handoff/wire-remediation.md + workflow/sessions/s-plan-audit-2026-06-04/FINISH/
      _MASTER-INDEX.md — the status truth sources. Plan statuses are MOVING (H-32..H-52 driven
      live 2026-06-07); every J status citation carries a re-check-at-execution rule.

================================================================================
JC — BINDING CORRECTIONS FOR J AUTHORING (deltas vs the source docs)
================================================================================
JC-1  zstd is NOT stdlib python. The redesign doc's "zstd above a size threshold" for J-3 backups
      becomes gzip (stdlib) above threshold; zstd only if a compression dep is ever approved.
JC-2  blake3 is NOT stdlib python; py-blake3 is a queued NEW DEP (wire-remediation E-8 new-dep
      list, needs approval). J-3/J-4 hashing default = hashlib.blake2b keyed (stdlib), with an
      explicit deviation note: the CLAUDE.md blake3 rule is a Rust-crate rule; harness tools are
      stdlib-only by house convention (HG-1). Upgrade path to blake3 on dep approval; the
      content-address scheme stores the algorithm prefix so a later migration is additive.
JC-3  SSB deltas across J: harness items (J-3..J-7) add ZERO variants; J-1 adds +1
      (PerceptionScaffoldEvent); J-2 prefers reuse of C-39 rows (net 0..+1). Anything else is a
      scope violation.
JC-4  pillow (J-1 harness /see leg) is NOT stdlib — verify-before-use (anti-slopsquat protocol)
      and the skill degrades to pass-through (attach the raw image + a one-line warning) when
      pillow is absent. The skill never hard-fails an image read.
JC-5  new-skill convention per E-43 (SHIPPED): frontmatter must pass
      .claude/tools/agentskills_validator.py and the skill ships eval/positive.jsonl +
      eval/negative.jsonl, like all 11 existing skills. Binds J-6 and the J-1 /see skill.
JC-6  N-5 is a PORT CHECKLIST, not a build (the redesign doc's own words: "a half-day port, not a
      plan"). J-5 uses the compressed detail form; its first gate is verifying the 7 unverified
      liteGAME compat files (memory: project_litegame_orchestrate_compat).

End of J-HARNESS-GROUNDING-MAP.
