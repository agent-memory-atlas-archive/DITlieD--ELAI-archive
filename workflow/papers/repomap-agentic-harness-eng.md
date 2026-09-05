repomap: agentic-harness-engineering (AHE)
source: C:\Users\user\Downloads\new\agentic-harness-engineering-main
paper: arXiv:2604.25850 (Fudan & PKU, 2026), MIT license, Python >=3.13, uv-managed
mapped: 2026-06-15, method-focused (read README + evolve.py + trace_converter.py + agent/skill/config trees + the evolved-harness product; ignored trace/result JSON corpora and the bundled PDF)

================================================================
1. WHAT IT IS
================================================================

AHE (Agentic Harness Engineering) is an automated system that EVOLVES THE HARNESS around a coding agent while holding the base LLM FIXED. The thesis: in 2026 what decides whether a coding agent reliably ships work is not the model, it is the "harness" wrapped around it (system prompt, tool descriptions, tool implementations, middleware, skills, sub-agents, long-term memory). AHE makes that harness an optimizable, observable, git-tracked artifact and runs an automated `evaluate -> analyze -> improve -> verify` outer loop that mutates harness components based on FAILURE EVIDENCE distilled from execution traces, with each edit framed as a FALSIFIABLE hypothesis (predicted task flips) that the next iteration's evaluation confirms or rolls back.

Central thesis in one line: a coding agent's harness is a dataset, and the competitive advantage is the traces it captures (quoted in HARNESS.md). Three pillars, all "observability":
  - Component observability: harness decomposed into 7 orthogonal file-level components (NexAU framework), each git-tracked so every edit is auditable + revertible.
  - Experience observability: an "Agent Debugger" (adb) distills ~10M-token raw traces per iteration into layered sourced reports (overview.md + per-task detail/{task}.md); the optimizer reads digests but can drill back to any raw rollout.
  - Decision observability: an "Evolve Agent" proposes evidence-backed edits, predicts impact, and is automatically falsified by the next iteration's flipped tasks.

The repo is the runnable harness + the meta-agent + the orchestration loop. The evolved WINNER harness (what the loop discovered) ships at experiments/evolved_harness/.

================================================================
2. STRUCTURE MAP (dirs + one-line roles)
================================================================

evolve.py                     The whole outer loop orchestrator (~4717 lines). evaluate (harbor) -> compute pass@k stats -> trace pre-extraction -> adb analysis -> build evolution query -> run evolve agent (optionally Best-of-N) -> attribute change flips -> auto-rollback to best-ever. Stdlib + yaml + dotenv only; no framework dep at top level.
trace_converter.py            Vendored trace normalizer: turns NexAU InMemoryTracer / Langfuse / OpenAI-Responses dumps into a cleaned {system_prompt, messages, tool_definitions, subagents, tokens, cost} dict. Span classification (is_llm_span / is_tool_span), sub-agent extraction from parent/child + metadata heuristics.

agents/code_agent_simple/     The MINIMAL starting coding agent that gets evolved. Baseline = 1 tool (run_shell_command), no middleware/skills/sub-agents. systemprompt.md (~0.5KB), code_agent.yaml, LongTermMEMORY.md, ShortTermMEMORY.md, start.py.
agents/evolve_agent/          The meta-agent (built on NexAU). evolve_prompt.md (15KB, the meta-instructions), evolve_agent.yaml, a full tool suite (file/shell/web/session + mcp_client), and rich middleware.
  middleware/                 ralph_loop.py (block complete_task until a verification command ran exit-0), context_compaction/ (sliding-window + adaptive trace compaction + token-threshold trigger), llm_failover.py, long_tool_output.py, round_and_token_reminder.py, environment_info.py, agent_events_middleware.py.
  skills/agent-debugger-cli/  Bundled `adb` CLI source (_source/, partially open-sourced); `adb ask` (QA over traces) + `adb check` (QC issue detection). SKILL.md documents config + trace-type routing.
  skills/nexau-evolution-guide/ The how-to-evolve reference the meta-agent reads (component schemas, YAML config, hook points, creation guides). reference/{hooks,tools,agents,llms,sandbox,skills}.md + validate_agent.py.
  tools/                      file_tools (read/write/replace/glob/search/list/apply_patch/multiedit/read_visual_file), shell_tools, web_tools, session_tools (ask_user/complete_task/save_memory/write_todos), recall_sub_agent, run_code, skill_tool, mcp_client (59KB).
agents/explore_agent/         Pre-evolution exploration agent. source_agent/ (clones+reads NexAU source) + web_agent/ (reads E2B docs + a curated list of top coding-agent design blogs). Runs in parallel with iteration-1 eval (zero added wall-clock) to produce a knowledge skill for the evolve agent.

configs/base.yaml             Shared defaults (dataset, target_pass_rate 0.95, max_iterations 100, harbor k=2 n_concurrent=64, explore_agent web_sources list, agent_debugger settings, best_of_n strategy_hints, post_evolve transfer datasets).
configs/experiments/*.yaml    Per-experiment overlays inheriting base via `_base:` deep-merge (gpt54 with/without adb, dry-run).
scripts/evolve.sh             tmux launcher wrapper around `uv run python evolve.py`. evolve-resume.sh, build_templates.py (builds one E2B template per task, 16-way parallel).
experiments/evolved_harness/  THE PRODUCT: the frozen winner harness. systemprompt.md (8.9KB, 8 numbered engineering rules), middleware/execution_risk_hints.py (24KB regex risk-pattern injector), tools/shell_tools/run_shell_command.py (27KB, evolved from 10KB), LongTermMEMORY.md (5 distilled lessons), code_agent.yaml.
skills/agentic-harness-engineering/  A portable, framework-agnostic SKILL that re-implements AHE's method as a workflow (audit / generate-manifest / verify / init). references/HARNESS.md = the cross-agent "Harness Specification v1.0" (the 7-component spec + Change Manifest schema). profiles/ for codex/hermes/openclaw.

================================================================
3. ARCHITECTURE + METHOD (detailed)
================================================================

THE 7 ORTHOGONAL HARNESS COMPONENTS (the unit of evolution; HARNESS.md + evolve_prompt.md):
  1 System Rules (systemprompt.md) - behavioral rules, advisory, applies to all tasks
  2 Tool Descriptions (tool_descriptions/*.tool.yaml) - schema + usage caveats the LLM reads
  3 Tool Implementations (tools/*.py) - actual capability code
  4 Middleware (middleware/*.py + code_agent.yaml) - hooks into the agent loop pipeline
  5 Skills (skills/*/SKILL.md) - on-demand reusable workflow packages
  6 Sub-Agents (sub_agents/*/) - delegated isolated-context child agents
  7 Long-Term Memory (LongTermMEMORY.md) - persistent cross-session knowledge, agent reads at startup
"Orthogonal" = editing one does not touch the others; each maps to exactly one file/dir so an edit is visible + git-revertible. Registration is explicit in code_agent.yaml (creating a file is not enough). The config dir is on sys.path; binding strings like `tools.file_tools:read_file` resolve to module:fn. This is a near-1:1 structural twin of ELAI's own harness (.claude/agents, .claude/skills, .claude/rules, tools, hooks).

THE OUTER LOOP (evolve.py, per iteration NNN):
  Phase 0  create_experiment_dir + init_workspace: copy code_agent_simple -> workspace/, `git init`, commit "v0 baseline". The workspace IS a git repo; every evolve edit is a commit; rollback = git.
  Phase 1  run_harbor: launch `harbor run` (the laude-institute harbor runner) over the dataset inside E2B sandboxes, k=2 rollouts/task, 64-concurrent. Per trial it writes agent/nexau_in_memory_tracer.cleaned.json (full step trace), agent/nexau.txt (runtime log), verifier/reward.txt (pass/fail). THE TRACE, NOT THE SCORE, IS THE UNIT every later step operates on.
  Phase 2  compute_stats: pass/fail/exception per task; pass@k via the Chen et al. unbiased estimator (pass_at_k_est); a task with k>1 passes only if ALL rollouts pass; exception-type clustering from exception.txt; timeout detection.
  Phase 2.3.5  trace pre-extraction (extract_verifier_failures from ctrf.json clustered by error type; extract_agent_behavior_stats from the tracer: n_llm_calls, n_tool_calls, tool_usage, tool_errors, duration, possibly_hit_max_turns) - cheap deterministic digest so the evolve agent does not re-read raw traces.
  Phase 2.4  cross-iteration task history + stability classification (stable_pass / stable_fail / unstable / possibly_unstable / infra_only) and a FULL 9-STATE TRANSITION MATRIX between iterations (flipped fail->pass, regressed pass->fail, infra_recovered, infra_lost, exception_to_fail, etc.; with k>1 also rollout_improved/regressed/unchanged). This matrix is the falsification substrate.
  Phase 2.5a  Agent Debugger: run_parallel_adb_ask spawns `adb ask` (16-concurrent), one process per task carrying that task's rollouts. For failing tasks it asks for ROOT CAUSE / PASS-vs-FAIL divergence / CRITICAL MISTAKE / GENERAL MECHANISM (explicitly "general patterns, NOT task-specific knowledge"). For all-pass tasks it asks for KEY STRATEGY / REUSABLE PATTERN / FRAGILITY RISK. Verifier output is cross-referenced because the agent never sees it (catches "agent thought it passed but verifier disagrees"). Results distill to analysis/overview.md + detail/{task}.md.
  Phase 3  build_evolution_query: assembles a structured report (stats + transition matrix + stability + adb overview + previous change-attribution + previous Best-of-N variant comparison + optional MANDATORY strategy constraint) and run_evolve_agent runs the meta-agent. The evolve agent may write ONLY inside workspace/, must read analysis/ FIRST, groups failures into PATTERN CLASSES, and for each picks the right component LEVEL to fix. For every change it must commit four fields: failure_evidence, root_cause, targeted_fix, predicted_impact (predicted task flips + at-risk regressions). Output: per-change git commit + change_manifest.json.
  Phase 4  staggered-generation falsification: each runs/iteration_NNN/ mixes two generations - input/ holds loop NNN-1's just-evaluated workspace, evolve/ holds loop NNN's new edits (evaluated next loop). evaluate_changes attributes the next eval's flips back to a change_manifest's predictions and writes change_evaluation.json. Predictions that do not hold are surfaced to the next evolve agent for ROLLBACK + PIVOT.
  Safety net  update_best_ever + perform_auto_rollback: tracks the best-pass-rate iteration ever; if an iteration regresses, the loop can git-reset the workspace back to the best-ever state so evolution never strands on a bad branch. Loop terminates on target_pass_rate or max_iterations.

KEY EVOLUTION DOCTRINE (evolve_prompt.md + nexau-evolution-guide):
  - Controllability: only workspace/ is writable; runs/ read-only; llm_config is HANDS-OFF (model/temp/max_tokens never touched - "LLM config changes cause broad hard-to-diagnose regressions").
  - Evidence-driven: no change on intuition/"best practices"; every change traces to a failing task + trace excerpt.
  - Start MINIMAL: the agent begins with ONE tool, no middleware/skills/sub-agents. The paper's claim: starting minimal and evolving works best because every component is "earned, not preset."
  - Component extrapolation anti-pattern: if a failure class survives 2+ iterations of fixes at one component level, that level is wrong - rollback and re-approach at a different level (e.g. a prompt rule that keeps failing -> replace with middleware or a new tool).
  - pass@1 is the SOLE target; timeouts count as failures; partial-pass (k>1) tasks are the most valuable - compare the passing vs failing rollout of the SAME task, find the divergence, make the winning strategy the reliable default ("turn pass@k into pass@1").

BEST-OF-N (configs best_of_n, evolve.py setup_variant_workspace/select_variant_winner/adopt_variant_winner): each iteration optionally spawns N evolve agents IN PARALLEL on separate git WORKTREES, each given a MANDATORY orthogonal strategy constraint (e.g. variant A = structural changes only: middleware/tools/sub-agents; variant B = guidance only: prompt/skills/descriptions/memory). All variants evaluated in parallel; best pass-rate wins, adopted via git merge, losers git-tagged. The next iteration sees a CROSS-VARIANT comparison so it learns from the loser too (the loser may have fixed tasks the winner did not).

THE EXPLORE AGENT (pre-flight knowledge harvest): before evolution, source_agent clones NexAU and reads its internals; web_agent reads a CURATED list of top-tier coding-agent design sources (LangChain deepagents #1-on-terminal-bench writeup, HumanLayer harness design, Anthropic Claude Code best-practices + SWE-bench-sonnet report, OpenHands architecture, Aider leaderboard, Devin terminal-bench). Output becomes a knowledge skill the evolve agent consults. Runs in parallel with iteration-1 eval for zero added wall-clock.

THE EVOLVED WINNER HARNESS (experiments/evolved_harness/ - the empirical RESULT, most valuable artifact):
  STRIKING: the winner is still MINIMAL - one tool (run_shell_command), ONE middleware (ExecutionRiskHintsMiddleware), no sub-agents, no skills. The evolution went almost entirely into (a) the SYSTEM PROMPT, (b) the SHELL TOOL implementation, (c) one regex middleware, (d) 5 long-term-memory lessons.
  - systemprompt.md grew 0.5KB -> 8.9KB, encoding 8 numbered "engineering experience" rules, all of which are anti-reward-hacking / verification-discipline rules: (1) Contract first - read the verifier/tests as source of truth, copy literal public paths/filenames/signatures, do not invent a proxy metric. (2) Mirror the evaluator before finishing - run an independent final check that hits the EXACT interface the evaluator hits, from the evaluator's cwd; a pass via a copied helper/hidden build dir/LD_LIBRARY_PATH hack is only a debug signal. (3) Preserve semantics, minimal changes. (4) Control candidate selection - scorecard, submit only best-verified. (5) Generalize not overfit - test perturbations/hidden instances. (6) Manage time - timeout_ms probes, background long jobs, pivot on stall. (7) Finish only when end-state is ready NOW. (8) Semantic checks not existence/build-success; fail-fast custom validators (a later "passed" never overrides an earlier mismatch).
  - execution_risk_hints.py: a before_model/after_tool middleware holding ~30 compiled regexes that detect risky shell sequences (validating against `/` instead of the named socket, existence-only checks, near-threshold benchmark wins, non-fail-fast diff/cmp asserts, building into the live tree, raw model API bypassing the official wrapper, destructive post-success cleanup) and injects a one-shot corrective hint into the next model call. It is the runtime enforcement of the prompt rules.
  - LongTermMEMORY.md: 5 distilled cross-session lessons, all verification-discipline (exact contract surfaces; freeze published state after a passing check; close the loop from the on-disk artifact; fail-fast validators; performance needs margin not a single near-threshold pass).

NEXAU MIDDLEWARE HOOK MODEL (nexau-evolution-guide hooks): before_agent, after_agent, before_model, after_model (can force_continue), before_tool, after_tool, wrap_model_call, wrap_tool_call. before_* run top-down, after_* bottom-up. Middleware can make its own LLM side-calls via LLMCaller (with middleware_manager=None to avoid recursion). This is a clean, generic agent-loop interception model.

================================================================
4. KEY TECHNIQUES PORTABLE TO ELAI
================================================================

T1. STAGGERED-GENERATION FALSIFICATION OF HARNESS EDITS (the headline steal).
    Where: evolve.py build_evolution_query + evaluate_changes + change_evaluation.json; HARNESS.md Change Manifest schema.
    What: every harness change is a hypothesis carrying {failure_evidence, root_cause, targeted_fix, predicted_fixes[], at_risk_regressions[]}. The NEXT evaluation attributes actual pass<->fail flips back to the prediction; wrong predictions are rolled back. A 9-state transition matrix (flip/regress/infra/exception) is the substrate.
    Maps onto ELAI: this is exactly the empirical backbone the DGM self-improvement track (H-43..H-48) needs and the wiring-enforcement doctrine demands - "measure the real objective off-agent, missing/wrong evidence is a verdict." ELAI already logs SSB rows + has change_evaluation-style attribution interest. A "harness change manifest" with falsifiable predicted_fixes, verified against the next benchmark run's pass-rate delta, IS the un-gameable stronger-than-generator filter the Track-7 META-FINDING says a self-improvement loop provably collapses without. Directly grounds H-39 abstention-aware promotion (a change with a false prediction is "abstained"/rolled back) and H-45 worst-case+lineage promotion scoring. HIGH portability; pure method, no framework lock-in.

T2. TRACE-AS-UNIT + LLM TRACE DISTILLATION INTO SOURCED ROOT-CAUSE REPORTS (Agent Debugger).
    Where: agents/evolve_agent/skills/agent-debugger-cli (`adb ask`/`adb check`); evolve.py run_parallel_adb_ask + DEFAULT_DEBUG_QUERY; trace_converter.py.
    What: raw ~10M-token traces -> per-task LLM QA (root cause / pass-vs-fail divergence / general mechanism, explicitly NOT task-specific) -> layered overview.md + detail/{task}.md the optimizer reads, with drill-back. Verifier output cross-referenced because the agent never saw it (catches false self-success). adb check is a deterministic QC pass that emits typed issues {issue_type, summary, evidence, message_index}.
    Maps onto ELAI: ELAI's auditor + failure_taxonomy.py already classify failure output; this adds an LLM "why did the agent fail" layer over the agent's OWN trace, distilled into a digest the next planning cycle consumes. The pass-vs-fail-rollout divergence analysis is novel for ELAI and directly serves the multi-agent loop's memory-writer (write the GENERAL MECHANISM, not the task fix). trace_converter.py's span classification + sub-agent extraction is reusable for ELAI's own multi-agent trace inspection. MEDIUM-HIGH portability (adb core partially closed; the METHOD and the query templates port fully).

T3. THE 7-COMPONENT ORTHOGONAL HARNESS SPEC + CHANGE MANIFEST (HARNESS.md v1.0).
    Where: skills/agentic-harness-engineering/references/HARNESS.md; SKILL.md (audit/generate-manifest/verify/init workflows).
    What: a cross-agent vocabulary (System Rules / Tool Descriptions / Tool Implementations / Middleware / Skills / Sub-Agents / Long-Term Memory), an anti-pattern list (no impl in rules, no memory in rules, no two components doing one job, no impl-without-description-drift), and a falsifiable Change Manifest format with keep/revert/partial verdicts.
    Maps onto ELAI: ELAI's harness already IS roughly these 7 components but they are not named/audited as orthogonal. A "harness audit" (is .claude/rules vs .claude/tools vs .claude/agents orthogonal? does every tool-description match its impl?) plus a Change Manifest on .claude edits would formalize ELAI's own meta-evolution. The anti-pattern "if a failure class survives 2+ fixes at one component level, switch levels" is a concrete addition to ELAI's anti-spiral skill (which detects stagnation but does not prescribe a LEVEL switch). HIGH portability; it is literally a spec.

T4. RALPH-LOOP "VERIFY-BEFORE-DONE" MIDDLEWARE.
    Where: agents/evolve_agent/middleware/ralph_loop.py.
    What: intercept the complete_task/stop tool; scan recent history for a verification command that ran exit-0 (pytest/make test/curl/diff/...); if none, BLOCK completion and inject a "write+run tests first" message; force-allow after max_blocks to avoid infinite loop.
    Maps onto ELAI: a runtime-loop analog of ELAI's wiring-enforcement "missing evidence is a hard FAIL" but applied at the AGENT's own stop boundary instead of the auditor's. ELAI's agents could carry a stop-guard that refuses DONE until a real verification command is observed in the transcript (mirrors R-5 "reporting passed=true on anything you did not run is a syntax-error-severity violation"). MEDIUM portability (ELAI Rust runtime, not NexAU - reimplement the pattern, not the code).

T5. THE EVOLVED ANTI-REWARD-HACKING SYSTEM PROMPT + EXECUTION-RISK REGEX MIDDLEWARE (the discovered content itself).
    Where: experiments/evolved_harness/systemprompt.md + middleware/execution_risk_hints.py + LongTermMEMORY.md.
    What: 8 verification-discipline rules + ~30 regex risk detectors that an automated loop DISCOVERED were the highest-leverage harness content for a shell-only coding agent. Every rule is about hitting the exact evaluator contract and not faking success.
    Maps onto ELAI: this is empirically-grounded prior art for ELAI's own rule files. "Mirror the evaluator before finishing / hit the exact named interface / a pass via a helper script or LD_LIBRARY_PATH hack is only a debug signal / freeze published state after a passing check / fail-fast validators" can be lifted near-verbatim into ELAI's codeRules / wiring-enforcement / production-caller-gate doctrine - and it converges independently on ELAI's existing "measure the real objective not a proxy" and "verify semantics not just 'test passed'" (auto-memory feedback_verify_semantics). The regex middleware is a concrete template for an ELAI pre-tool hook that screams on existence-only checks or near-threshold benchmark wins. HIGH portability for the PROMPT TEXT (directly readable); MEDIUM for the regex middleware (reimplement as a hook).

T6. BEST-OF-N PARALLEL EVOLUTION WITH ORTHOGONAL STRATEGY CONSTRAINTS (via git worktrees).
    Where: evolve.py setup_variant_workspace / run_evolve_agent_on_variant / adopt_variant_winner; base.yaml best_of_n.strategy_hints.
    What: N evolve agents in parallel on separate git worktrees, each forced into a DIFFERENT component-class (structural vs guidance), best wins by pass-rate, merged in, loser kept as a learning signal.
    Maps onto ELAI: a diversity-vote pattern for ELAI's plan-improvement / DGM branching - run two scoped improvement attempts under orthogonal constraints, keep the measured winner, feed the loser's wins forward. Converges with ELAI's "two approaches minimum before any recommendation" rule. The git-worktree isolation is clean. MEDIUM portability (ELAI's parallel-agent cap is 3; this fits).

NON-PORTABLE / FLAG:
  - harbor + E2B SaaS sandboxes + NexAU framework + Terminal-Bench dataset are the substrate; ELAI does not run on NexAU and has no E2B. Port the METHODS, not the runner. The adb CLI core is only partially open-sourced (company strategy) - the SKILL.md query templates + trace_converter.py are the open, portable parts.
  - The whole loop assumes a large external benchmark with an automated pass/fail verifier per task (Terminal-Bench-2). ELAI has no equivalent standing benchmark with per-task verifiers (the recurring SAFEDELTA "need a real measured number" gap in ELAI's market memory). Falsification-driven evolution (T1) is only as good as the benchmark behind it - this is the SAME prerequisite ELAI's GTM/trust roadmap already identified. The technique is portable; the missing piece is ELAI's benchmark, not the method.
  - llm_config is deliberately frozen; AHE evolves ONLY the harness. ELAI's interest in model/runtime routing (E-31 tier matrix) is orthogonal and out of AHE's scope - do not read AHE as evidence about model selection.

================================================================
5. BENCHMARK NUMBERS (as stated in README/paper, grounding only)
================================================================

- Terminal-Bench 2 pass@1 lifted 69.7% -> 77.0% on GPT-5.4 across ten evaluate->analyze->improve iterations (README Overview + paper abstract).
- Surpasses hand-written Codex harness (71.9%) and the self-evolving ACE and TF-GRPO baselines (README; exact baseline numbers for ACE/TF-GRPO not stated in README).
- AHE on GPT-5.5 ranked #3 on the Terminal-Bench 2.0 public leaderboard at 84.7% (News 2026-05-14, ranking as of 2026-05-15).
- The frozen evolved harness transfers WITHOUT re-evolution to SWE-bench-verified and to four alternate base models (README; per-model transfer deltas shown only in assets/figures/transfer_model.png, not as text - UNVERIFIED numerically here).
- Source caveat: all numbers are author-reported (README + arXiv:2604.25850); not independently reproduced in this mapping. Date 2026 places this past my training cutoff - treated as repo-stated, not training-grounded.

================================================================
6. DEPENDENCIES / STACK
================================================================

- Python >=3.13, managed with uv (pyproject.toml). Top-level evolve.py: stdlib + pyyaml + python-dotenv only.
- NexAU framework v0.3.9 (github.com/nex-agi/NexAU) - the agent runtime (agents/middleware/tools/hooks/tracer abstraction). The code agents and evolve agent run on it.
- harbor (laude-institute) - the benchmark runner CLI (`harbor run --agent nexau --env e2b ...`); datasets from laude-institute/harbor-datasets.
- E2B (e2b.dev) - sandbox per rollout; SaaS (concurrency-capped) or self-hosted cluster; templates prebuilt via scripts/build_templates.py.
- adb (agent-debugger-cli) - bundled partially-open-source trace QA/QC CLI; local-runner auto-creates a venv under ~/.adb.
- LLM access via OpenAI-compatible endpoint (LLM_API_KEY/BASE_URL/MODEL env); experiments use gpt-5.4/gpt-5.5; api_type openai_responses with reasoning.effort=high for the code agent.
- SERPER_API_KEY (web search for explore/evolve agents), optional Langfuse (trace store), optional Feishu webhook (notifications), tmux (launcher).
- Datasets: Terminal-Bench 2.0 (primary), SWE-bench-verified (transfer).
