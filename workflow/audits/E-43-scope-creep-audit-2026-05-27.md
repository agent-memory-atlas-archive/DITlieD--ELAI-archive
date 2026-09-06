E-43 skill scope-creep audit (2026-05-27)

Per duet.so/guides/agent-skills-101-tools-vs-mcp-vs-skills: one skill, one verb. Skills covering >3 verbs are scope-creep candidates that misfire under auto-trigger (per Vercel eval, 53% vs 100%).

skill                  declared verbs                                                   #verbs  recommend
  anti-spiral          detect-stagnation, switch-persona                                  2     KEEP
  oracle               answer, ground, cite                                               3     KEEP
  prompt-transform     reformat-prompt                                                    1     KEEP
  b-plan-runner        run-bplan, continue-bplan, execute-bplan, ship-bplan               4     SPLIT-CANDIDATE (run vs continue could merge)
  dashboard-handoff    smoke-svelte, cargo-check-dashboard, tauri-dev-launch              3     KEEP
  idea-intake          decompose-idea, ground-research, audit-tree                        3     KEEP
  impact-check         pre-edit-blast-radius                                              1     KEEP
  orchestrate          orchestrate-plan, walk-fsm, wait-deps, gate-phase-end, commit-once 5     SPLIT-CANDIDATE (multi-verb but the verbs are pipeline stages; auditor flags but does not block)
  planning-workflow    create-plan, start-phase, improve-plan, audit-phase, complete-phase, handoff-context  6  SPLIT-CANDIDATE (the most overloaded skill; report's headline scope-creep flag)
  rag-tools            search-context, memorize, ingest, forget, list-topics              5     SPLIT-CANDIDATE (5 verbs but all are read/write ops on the same MCP server; less risky than planning-workflow)
  sidecar-protocol     boot-sidecar, health-check, run-inference                          3     KEEP

Recommendation summary:
  - planning-workflow is the headline scope-creep concern. 6 verbs cover plan creation, FSM execution, TDD cycles, audits, completion, and context handoff. A split into planning-create, planning-execute, planning-audit would dramatically improve per-trigger precision under the AGENTS.md-vs-Skills test bed. Track as a separate E-N follow-up; this audit does NOT mandate the split.
  - orchestrate is multi-verb but the verbs are pipeline stages of a single autonomous run. Lower-risk than planning-workflow.
  - rag-tools is multi-verb but the verbs are bounded to one MCP server. Lowest risk of the SPLIT-CANDIDATEs.
  - b-plan-runner covers 4 closely-related verbs (run/continue/execute/ship); a merge into one canonical verb with sub-flags would be cleaner. Low priority.

Grounding:
- duet.so/guides/agent-skills-101-tools-vs-mcp-vs-skills (one skill, one verb)
- vercel.com/blog/agents-md-outperforms-skills-in-our-agent-evals (53% misfire under auto-trigger when multi-verb)
