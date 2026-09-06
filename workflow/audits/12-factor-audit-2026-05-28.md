12-factor agents -- ELAI surface audit
audit run: 2026-05-28
session: s-25120 E-22 completion-drive
reference: github.com/humanlayer/12-factor-agents (20.5k stars at sweep time; verified per grounding doc cite 10)
status: positioning artifact (one row per factor with grade + evidence; no fixer queue)

factor grading legend
  PASS    -- ELAI's surface satisfies the factor on grounded evidence in the repo
  PARTIAL -- partial coverage; documented gap with a follow-up pointer
  FAIL    -- no coverage; this would be a real reform target

audit table

01 natural language to tool calls
   grade: PASS
   evidence: plugins/mcp_bridge.rs dispatch_tool_call_with_gate + ToolGate (C-35) + AuthzPolicy (C-42) + ToolCalibrationRegistry (C-47) compose into a 3-layer policy chain before any tool round-trip. natural-language prompts go through this chain.
   weakness: depends on the upstream LLM emitting the right tool call; gate is reactive.

02 own your prompts
   grade: PASS
   evidence: prompt_optimization_completed SSB family (E-8); SkillSystemEvent (E-43); GepaTemplateEvolved + GepaTemplateEvicted (E-55 reward ladder). prompt fragments live in orchestrator::experience::PromptFragment with explicit owner attribution.

03 own your context window
   grade: PASS
   evidence: E-32 deep-mode + yarn-rope-scaling explicit context budget tracking; ELAI_LOCAL_MAX_CONTEXT env gate; ELAI_LAZY_MCP_SCHEMA_ENABLE (E-22) lazy tool schemas; LazySchemaCache (C-35) with hit/miss telemetry.
   weakness: cloud-side context window is bounded by provider, not measured locally; we trust Anthropic's reported usage.

04 tools are just structured outputs
   grade: PASS
   evidence: McpToolSpec + McpToolSummary; xgrammar_mask_applied + grammar_mode_activated SSB rows (E-35); InstructorModeActivated; CodeactActionExecuted. tools and structured-output engines are first-class.

05 unify execution state and business state
   grade: PARTIAL
   evidence: Ssb is the unified state layer (single SQLite + per-action_type indexes). FSM substates land via fsm_transition rows. agent_substate landed under B-41 per-level auditor modes.
   weakness: business-state-side write surface still goes through the dashboard's Tauri commands, not through SSB. cross-state queries require joins by hand; no single query interface.

06 launch / pause / resume with simple APIs
   grade: PARTIAL
   evidence: pod-lifecycle MCP (pod_create / pod_stop / pod_start / pod_restart / pod_terminate / pod_idle_check) covers process lifecycle. FSM has well-defined transitions. Checkpoint family (E-44) supports resume.
   weakness: no single "pause-the-orchestrator" call -- pause must come via PreCompact hook or external session reset; cross-session resume relies on SESSION_HANDOFF docs more than a typed API.

07 contact humans with tool calls
   grade: PASS
   evidence: gateway_hitl_pending SSB (C-42 authz layer) explicitly pages a human when policy demands; AutonomousGate triggers escalation; argv_approval_required (E-30) for destructive ops.

08 own your control flow
   grade: PASS
   evidence: FSM is owned by the orchestrator (.elai_cc/crates/orchestrator/src/fsm.rs); transitions logged to SSB; no hidden control flow inside a hosted LLM-as-controller -- the LLM proposes, the FSM disposes.

09 compact errors into the context window
   grade: PARTIAL
   evidence: anti-spiral skill handles 3-failed-hypotheses condition; failure_taxonomy.py classifies build/test failures into 13 categories before reading raw output; gotchas.md is the long-term memory of compacted errors.
   weakness: no automated "compact this stack trace into 50 tokens" inline within the FSM; the agent has to manually summarize.

10 small focused agents
   grade: PASS
   evidence: 11 ELAI-authored skills under .claude/skills/ each scoped to 1-3 verbs (orchestrate / planning-workflow / rag-tools / oracle / idea-intake / anti-spiral / b-plan-runner / dashboard-handoff / impact-check / prompt-transform / sidecar-protocol). per-level auditor modes (B-41) further specialize the auditor by tree level.

11 trigger from anywhere, meet users where they are
   grade: PARTIAL
   evidence: CLI surface (elai); Tauri dashboard; MCP bridge (local-rag + pod-lifecycle); remote sidecar; ScheduleWakeup / cron skills.
   weakness: no first-class Slack / Discord / email trigger surface (humanlayer's reference is heavier on chat-based triggers). Notion / Gmail / Google Calendar / Drive MCPs exist as deferred tools but are unauthenticated by default.

12 make your agent a stateless reducer
   grade: PASS
   evidence: FSM state lives in SSB (durable); each agent invocation is stateless -- the agent reads the FSM, acts, writes back. main LLM sub-agents (context-compiler / auditor / memory-writer / integration-explorer) are spawned fresh per task with GC'd context.

summary
  PASS:    01, 02, 03, 04, 07, 08, 10, 12  (8)
  PARTIAL: 05, 06, 09, 11                  (4)
  FAIL:    (none)
  net: 8 PASS / 4 PARTIAL / 0 FAIL

action items (positioning, NOT a fixer queue)
  1. PARTIAL-05 cross-state query interface: defer to a future C-plan; the SSB indexer view is good enough for now.
  2. PARTIAL-06 launch/pause API: the pod-lifecycle surface covers process-level; the agent-level pause is implicit. consider a future C-plan that surfaces "elai pause [session-id]".
  3. PARTIAL-09 auto-compaction of errors: arXiv:2603.18866 SkRAG style summary-on-failure could land alongside anti-spiral; defer.
  4. PARTIAL-11 chat triggers: depends on which surface the user picks first (the Notion / Gmail MCPs are already in the tool list but require user auth flow). out of E-22 scope.

grounding
  - github.com/humanlayer/12-factor-agents (verified live per grounding doc cite 10)
  - workflow/improvement-sweep-2026-05-19/reports/22-github-trending-may2026.md (source report)
  - .claude/skills/ directory (11 ELAI-authored skills)
  - .elai_cc/crates/orchestrator/src/ssb.rs (ssb action type catalogue, 443 variants post-E-22)
