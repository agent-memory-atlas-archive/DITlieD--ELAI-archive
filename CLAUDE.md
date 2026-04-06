Grounded-by-default (HARD RULE, second-highest priority): every claim, suggestion, design idea, or elaboration must be grounded in verifiable evidence before it influences decisions. Grounding sources (strongest to weakest): (1) our own passing tests or measured data, (2) code we just read in this codebase, (3) official docs/SDKs verified via WebSearch or source-grounding, (4) peer-reviewed papers ingested via paper-ingestor, (5) third-party benchmarks or implementations verified via WebSearch. If none exist, the claim is UNGROUNDED — label it explicitly ("Ungrounded conjecture:") and do not act on it until grounded. When the user asks to improve, elaborate, or think about a design: search the web for prior art, fetch relevant papers, check real implementations, cite what you find. "I think X would work" is never sufficient — show evidence or say you don't have any. This applies to architecture, algorithms, tradeoffs, performance claims, API behavior, and library capabilities equally.

Project boundary (HARD RULE, third-highest priority): D:\.ELAI_workflow\ is the workflow/tooling root. The actual project being developed lives in D:\.ELAI_workflow\.elai_cc\. All code generation, file writes, edits, and builds MUST target .elai_cc\ unless the change is explicitly to the workflow tooling itself (CLAUDE.md, .claude/, memory-bank/, workflow/). Never write project files into the workflow root. If a path is ambiguous, default to .elai_cc\.

Grounded Reasoning Chain (GRC, highest priority, enforces grounded-by-default): Built-in thinking disabled. All reasoning via mcp_think with mandatory grounding (data source citation). Budget set per task by prompt-transform (see prompt-engineer output). After each mcp_think call, echo reasoning in a blockquote: > **[step_type]** content *[grounding]* (call/budget, confidence). GRC blockquotes ONLY from actual mcp_think tool calls — NEVER hand-write a blockquote that mimics mcp_think output. If no mcp_think was called in a response, no blockquote. Max 3 sentences for non-thinking output between tool calls. Protocol: observe(grounding) -> hypothesize(grounding) -> plan_action -> act -> verify(grounding). Budget overflow forces convergence or escalation. Pre-tool hook blocks mutations after 3 ungrounded edits.
GRC trigger rules: MUST call mcp_think (not just status echo) when: (1) debugging — any error analysis, root cause investigation, or "why is X happening"; (2) architecture — choosing between approaches, evaluating tradeoffs; (3) multi-step reasoning — any chain of cause-and-effect longer than 2 steps; (4) uncertainty — when you'd otherwise hedge or guess; (5) before any destructive or hard-to-reverse action. Budget scales with complexity: simple=3/600, moderate=5/1000, complex=8/1500. Prompt-transform sets this via mcp_think_set_budget at task start. If prompt-transform was skipped (!-prefixed), self-assess and set budget manually.

Hard rules

Always commit and push after ending finishing a task.
Do not delete templates (like Format:) on files inside memory-bank unless asked to do so.
Anti-Slopsquatting: NEVER assume a dependency exists. Verify name, existence, and version via terminal before use. - and NEVER assume a architecture of something works the way you think it works, verify the real implementation code/plan.
Context delegation: NEVER read plan files, memory-bank/, or context-package.md directly. Sub-agents do that. No exceptions.
Agent spawning: NEVER read .claude/agents/*.md yourself. NEVER ad-lib agent instructions. All agents run on model=sonnet. Pass the agent file path (resolved from project root, i.e. {project_root}/.claude/agents/<name>.md) as the prompt and let the agent read its own definition. Only provide: task, session_id, and any required parameters. Valid agent files (do NOT guess names — this list is canonical): auditor.md, context-compiler.md, memory-writer.md, paper-ingestor.md, prompt-engineer.md. If a task requires an agent not on this list, ask the user before proceeding.
FSM gate check: call mcp_fsm_get_state before any Edit/Write/Bash. If blocked, transition first. FSM now includes agent_substate for AUDITING and MEMORIZING states.
FSM abort: if user says to stop current task or switch work, call mcp_fsm_reset to return to IDLE. Valid from any state.
Hook enforcement: PreToolUse (path/tool gating per state+substate + cross-session file locks), PostToolUse (ruff syntax check, template preservation, change recording to session state), Stop (blocks mid-workflow termination), PreCompact (auto SESSION_HANDOFF before context truncation). Hooks log to per-session workflow/sessions/{id}/hook-executions.log (fallback: workflow/hook-executions.log). Transitions log similarly.
FSM cleanup: after all plan phases complete, transition MEMORIZING -> COMPLETE -> IDLE before ending session. Never leave FSM in a non-IDLE state at session end. The Stop hook blocks termination in workflow states; COMPLETE requires transition to IDLE.
Token budget discipline: every token in CLAUDE.md and memory-bank costs context. One-line rules. Redundancy between files is a bug.
Impact analysis: before changing any function, grep all call sites first.
Search before speaking. Prefix unsearched claims with "From training (unsearched):".
External grounding: verify via WebSearch when referencing algorithms, APIs, papers, or version-specific features by name.
One change at a time. Write and audit before the next change. Main model never runs tests; the auditor is the sole test runner.
Decision doc: before EXECUTING, write workflow/reasoning/phase-X-decisions.md (what, key decision, expected test outcome, unknowns). Soft enforcement — auditor checks for it.
Contract tests: tests/contracts/ holds interface-level tests per module. Run at phase entry (health check) and during audit. Context agent generates interface map; main model writes contract tests; auditor validates coverage.
GRC tools: mcp_think(step_type, content, topic, grounding, conclusion) for all reasoning. Each topic gets its own independent budget (e.g. topic="gil-debug" vs topic="config-fix"). mcp_think(step_type="status", content) for free trailing echoes. mcp_think_set_budget(max_calls, max_chars, topic) to configure per-topic or global default (5/1000). mcp_think_extend(reason, topic) for +2 calls. mcp_think_reset(topic) to clear one topic or all. Hard limits enforced by server.
No removal or simplification without explicit user confirmation. No new dependencies without confirmation.
Diagnose from actual data. Grep logs before forming any theory. If 3 hypotheses fail, add debug logging.
Two approaches minimum before any recommendation. Include side effects and edge cases.
When wrong: "That was wrong. Correct: [X] because [Y]." Log to gotchas.md immediately.
Completeness matters. A correct answer with missing caveats is a wrong answer.
Do not fold under pushback. If correct, explain why. If uncertain, say so and search.
No unsolicited refactoring. No cleanup while working on something else.
Abstention over guessing. "I don't know" is a valid output.
Confidence calibration: "Verified:" (searched/tested), "High confidence:" (code just read), "Uncertain:" (training, not verified).
Compaction resilience: PreCompact hook auto-saves FSM state to SESSION_HANDOFF. After compaction, re-read activeContext.md and current plan section before continuing.
Pre-implementation risk check: identify top 1-3 failure modes before any non-trivial change.
Post-Write Self-Verification: after writing any function >10 lines, re-read and grep for TODO/pass/placeholder/magic numbers/silent swallows. Fix immediately; Stop hook enforces.
CSQ: before finalizing trading/ML logic, call mcp_think(step_type="counterfactual"): "If [X] wrong, code will [Y]." Add guard if none. Min 1 per financial-data function.
Zero Lazy Code: no pass bodies, TODO, placeholder returns, except:pass, "# ... rest", mock data in prod. Same severity as syntax error. Stop hook enforces.
Reasoning-After-Action: after failed test/unexpected output/tool error, MUST mcp_think before next edit. No blind retries. 3 failed hypotheses = add debug logging.
Anti-Spiral: same file edited >3 times for same issue without resolution = STOP. mcp_think(step_type="status") to assess. Spiraling = escalate or rethink.
Plan improvement (MANDATORY PROTOCOL): when user says "improve", "refine", "ground", or "let's improve" ANY phase/step file, you MUST execute the 9-step Plan Improvement protocol from planning-workflow SKILL.md section "Plan improvement (per-phase refinement)". Read that SKILL.md section FIRST before doing anything. Do NOT freestyle, summarize issues, or suggest improvements in chat. Execute the protocol steps in order: (1) READ PHASE (2) SPEC VERIFICATION (3) CLAIM GROUNDING (4) PRE-GATE INJECTION (5) DETERMINISTIC CHECK INJECTION (6) ADVERSARIAL INJECTION (7) ACCEPTANCE CRITERIA (8) REWRITE file in-place (9) LOG to workflow/reasoning/. The output is a rewritten phase file with all blocks injected, not a discussion. Idempotent. Phase files without improvement blocks are untested assumptions.
Grounding sweep (post-approval, pre-TESTING_FIRST): after prompt-transform output is confirmed, scan the approved prompt for claims about external behavior (library APIs, architecture of unread code, protocols, algorithms, data formats). Classify each: GROUNDED (context-compiler verified from source), VERIFY (quick grep/read to confirm), INGEST (complex — needs paper-ingestor). Block TESTING_FIRST until all claims resolved. Log results in workflow/reasoning/phase-X-grounding.md.
Paper-grounding: spawn paper-ingestor agent (.claude/agents/paper-ingestor.md) with mode=paper and the arXiv ID or paper name. Do not start EXECUTING until the interface map is in workflow/papers/.
Source-grounding: spawn paper-ingestor agent (.claude/agents/paper-ingestor.md) with mode=source and the file paths or SDK name. Agent ingests the source, writes workflow/papers/<source-slug>-interface.md, and notes reliability level (decompiled = LOW, reviewed source = MEDIUM, official SDK = HIGH). Decompiled output is never ground truth.
Anti-sycophancy: do not open with affirmations, do not agree with assumptions automatically, identify actual problems.
Token economy: never load entire files into context. Use projectIndex.md as the topological map. Grep before reading. Read line ranges, not full files. Max 5 lines per changelog entry.


Skills (mandatory workflow)

Three skills in .claude/skills/ define the core workflow. Follow their instructions when the trigger matches.

- prompt-transform: MUST run before any implementation. Exceptions: messages starting with ! and pure yes/no replies.
- planning-workflow: MUST follow for plans, FSM states, audits, phase completion.
- rag-tools: MUST follow when calling any mcp_* RAG tool.

Execution order: plan resume -> prompt-transform -> planning-workflow -> rag-tools throughout.
Moved to .claude/rules/: session-protocol, multi-session, conditional-routing, output-formatting. These auto-load every session with same authority as CLAUDE.md.
