Workflow Order of Operations

Format: step / agent / action

Session start / context-compiler / compiles context package from memory-bank and codebase
Prompt transform / prompt-engineer / reformats raw user message into structured prompt with rules
Plan creation / main model / drafts phase-by-phase plan with deliverables and test outcomes
Plan audit / adversarial-auditor / validates feasibility, completeness, and domain rule compliance
Execution / main model / TDD implementation strictly following the approved plan
Testing and verification / adversarial-auditor / runs tiered pipeline and code review
Memory update / memory-writer / records changes, gotchas, and updates activeContext
Session handoff / main model / records progress for context exhaustion continuity


FSM States and Transitions

Format: state -> state / condition

IDLE -> CONFIGURING / user requests workflow or agent modifications
IDLE -> PLANNING / user initiates a new task requiring a plan
CONFIGURING -> IDLE / workflow changes complete
PLANNING -> TESTING_FIRST / plan approved, begin TDD
TESTING_FIRST -> EXECUTING / tests written, begin implementation
EXECUTING -> AUDITING / implementation complete, request review
AUDITING -> EXECUTING / auditor returns BLOCK findings (increments audit_cycles)
AUDITING -> PLANNING / auditor declares DEAD END due to unsalvageable approach
AUDITING -> MEMORIZING / auditor returns AUDIT PASSED
MEMORIZING -> PLANNING / phase complete, move to next phase
MEMORIZING -> COMPLETE / all phases complete
COMPLETE -> IDLE / workflow cleanup


Subsystem Map (Topological Map)

[FSM Orchestrator]
Owns: State transitions, tool gating, and cross-session file locks
Depends on: .fsm-state.json
Provides to: Claude Code hooks (PreToolUse, PostToolUse, Stop, PreCompact)
Events/Hooks: Blocks disallowed tools per state, auto-saves to SESSION_HANDOFF before context truncation

[Context Compilation]
Owns: Blast radius analysis, reference context gathering, interface map generation
Depends on: memory-bank, project files, RAG
Provides to: workflow/sessions/{session_id}/context-package.md
Events/Hooks: Flags stale references, detects interface conflicts before planning

[Prompt Engineering]
Owns: Injection of domain rules, complexity assessment (GRC budget), output formatting
Depends on: workflow/sessions/{session_id}/context-package.md, memory-bank rules
Provides to: Main model instructions block
Events/Hooks: Triggers on coding, refactoring, architecture, and planning requests

[Tiered Validation Pipeline]
Owns: Sequential testing from determinism to stress
Depends on: scripts/run_pipeline.py, tests directory
Provides to: adversarial-auditor agent
Events/Hooks: Fails fast on first tier failure

[Adversarial Auditing]
Owns: Code review, pipeline execution, dead-end declarations, verification scripts
Depends on: Tiered Validation Pipeline, domain rules, audit-scratch scripts
Provides to: Main model via findings (BLOCK, WARN, INFO)
Events/Hooks: Validates temporal integrity, data pipeline leaks, and statistical rigor

[Memory Management]
Owns: Documentation updates and RAG memorization
Depends on: Main model change manifest
Provides to: memory-bank (changelog, progress, activeContext)
Events/Hooks: Runs post-audit to persist state before next phase or completion