repomap: meta-harness (stanford-iris-lab)

Source: C:\Users\user\Downloads\new\meta-harness-main  (57 files, ~70K tok code)
Paper: Meta-Harness: End-to-End Optimization of Model Harnesses, Lee/Nair/Zhang/Lee/Khattab/Finn, arXiv:2603.28052 (2026). Authors include Omar Khattab (DSPy) and Chelsea Finn.
License: see LICENSE in repo. Release note: "cleaned up version of the paper code; not tested beyond verifying it runs."
Verified: all claims below read from source in this repo. Benchmark numbers are quoted from the shipped SKILL.md / README (NOT re-run here) -> labelled.

================================================================
1. WHAT IT IS
================================================================

Meta-Harness is an outer search loop that OPTIMIZES THE HARNESS CODE around a FIXED base model. "Harness" = the code that decides what a model stores, retrieves, and shows itself while it works (memory, retrieval, context construction, prompt architecture, tool scaffolding, the agent control loop). The base model never changes; only the surrounding Python is searched.

The central idea ("meta" in meta-harness): the PROPOSER of new harnesses is itself a coding agent (Claude Code via `claude -p`). So it is an agent that writes the harness code for another agent (or for a fixed predict/learn loop). It is an evolutionary / hill-climbing search where:
  - the genome = a single self-contained Python file implementing a fixed interface,
  - the mutation operator = a Claude Code session that reads the run history + failure traces and writes 1-3 new candidate files,
  - the fitness function = an off-agent benchmark the proposer is FORBIDDEN to run itself,
  - selection = a Pareto frontier (accuracy vs injected-context cost) persisted across iterations,
  - the held-out test set is NEVER shown during search (leakage discipline is a first-class concern).

It is deliberately a FRAMEWORK + two reference experiments, not one app:
  - text_classification: searches over MEMORY SYSTEMS (a predict/learn-from-batch interface) for few-shot text classification.
  - terminal_bench_2: searches over AGENT SCAFFOLDS (subclasses of Terminus2) for Terminal-Bench 2.0 coding/terminal tasks.
ONBOARDING.md is a conversation prompt that produces a `domain_spec.md` to port the loop to a new domain.

================================================================
2. STRUCTURE MAP
================================================================

Top level:
  README.md            - framework overview, quick-start, citation.
  ONBOARDING.md        - LLM-facing onboarding prompt; screens domain fit, drives a Q&A to emit domain_spec.md (template inside). Has explicit "screening for poor fit" + "pushing on leakage" example dialogues.
  CITATION.cff, LICENSE, .env.example

reference_examples/text_classification/   (memory-system search)
  meta_harness.py      - OUTER LOOP. Phase0 baselines -> N iterations {propose -> validate -> benchmark -> update frontier} -> final test eval. val-only during search.
  inner_loop.py        - the fixed train/eval engine: online (predict->feedback->learn) and offline (train-with-labels->early-stop-on-val->eval) modes; threaded eval; JSONL trajectory logger; micro-F1.
  memory_system.py     - MemorySystem ABC (the harness interface): predict / learn_from_batch / get_state / set_state, + call_llm wrapper that records per-thread last-prompt len/hash/text, + robust extract_json_field.
  benchmark.py         - runs inner_loop per (dataset,memory,model), computes Pareto frontier (accuracy vs ctx tokens), writes frontier_val.json, prints tables.
  claude_wrapper.py    - the PROPOSER driver (see below). Shared idiom across both examples.
  llm.py               - thin LLM client w/ usage accounting (calls, in/out tokens).
  config.yaml          - datasets, split sizes (train200/val50/test100), models, baselines.
  .claude/skills/meta-harness/SKILL.md - the PROPOSER's instruction set (the mutation policy).
  agents/              - search space: no_memory.py, fewshot_memory.py, fewshot_all.py (baselines) + proposer-written candidates.
  data/                - loaders, evaluators, 5 datasets as jsonl (skipped).

reference_examples/terminal_bench_2/   (agent-scaffold search)
  meta_harness.py      - OUTER LOOP for scaffolds. Same shape, but fitness comes from Harbor/Runloop eval (run_eval.sh) over 89 TB2 tasks x N trials; parses per-trial reward + rollout cost/turn/token metrics; adds a SMOKE TEST gate (1 task x 1 trial) before full eval.
  agents/baseline_kira.py, baseline_terminus2.py - the two baseline scaffolds (Terminus2 subclasses).
  claude_wrapper.py    - same proposer driver.
  anthropic_caching.py - helper that stamps cache_control:ephemeral on the last 3 messages for Anthropic models (litellm).
  prompt-templates/terminus-kira.txt - system prompt the scaffold points at.
  scripts/run_eval.sh  - shells out to Harbor eval on Runloop.
  .claude/skills/meta-harness-terminal-bench-2/SKILL.md - proposer instructions for scaffold mutation.

================================================================
3. ARCHITECTURE + METHOD (the bulk)
================================================================

THE TWO-LEVEL LOOP
  OUTER (meta_harness.py, deterministic Python): orchestrates iterations, owns the benchmark and the frontier, persists history. It NEVER proposes; it only evaluates and bookkeeps.
  INNER MUTATION (a Claude Code session): the proposer. Given run history + failure traces, it writes new candidate harness files. It NEVER benchmarks.
This separation is the load-bearing design choice: the agent that writes the code cannot run the gate that judges it.

OUTER LOOP STEPS (text_classification meta_harness.py::run_evolve, terminal_bench_2 mirrors it):
  Phase 0 - benchmark fixed baselines, seed the frontier.
  Iteration i:
    1. render_task_prompt(i) -> a prompt naming the run dirs + where to write pending_eval.json.
    2. propose_claude(): spawn `claude -p` with the meta-harness SKILL injected, allowed tools [Read,Glob,Grep,Agent,Write,Edit,Bash], model=opus, effort=max, timeout ~2400s. Returns True iff pending_eval.json was written. (Notably: it POPS ANTHROPIC_API_KEY so the proposer uses subscription auth to dodge API rate limits, then restores it for the eval phase.)
    3. validate_candidates(): import-check each candidate file in a subprocess (`python -c "from ... import *"`). terminal_bench_2 ADDS a smoke_test: 1 task x 1 trial through the real eval harness, fail if any runtime error.
    4. benchmark: run the fixed inner loop / Harbor eval per valid candidate.
    5. update frontier (Pareto) + append one JSONL row per candidate to evolution_summary.jsonl (with hypothesis, axis, delta vs best, components, timing, and for TB2 rollout cost/turns/tokens).
  Phase Final - evaluate the frontier set (baselines + Pareto winners) ONCE on the held-out TEST split (written to a separate results/ dir never read during search). TB2 re-runs winners at higher trial count (5) for a denoised final number.
  Resumable: start_iteration = max(iteration in evolution_summary.jsonl)+1. --fresh clears candidates+logs but keeps baseline files. SIGINT/SIGTERM set a flag and finish the current step cleanly.

THE HARNESS INTERFACE (memory_system.py) - the "genome contract":
  MemorySystem ABC with exactly four required methods:
    predict(input) -> (answer, metadata)         # must work cold-start, no prior learning
    learn_from_batch(batch_results) -> None       # batch_results carry input/prediction/ground_truth/was_correct/metadata
    get_state() -> str / set_state(str)           # JSON-serializable checkpoint (enables early-stopping + restore)
  + call_llm(prompt): wraps the LLM and records (per-thread, threading.local) the last prompt's length, md5[:8] hash, and full text. This is how the benchmark measures INJECTED CONTEXT COST without the harness reporting it -- an off-agent measurement (context_len = prompt_len - len(test_input)).
  + extract_json_field(): a hardened JSON-field extractor (direct parse -> fenced code block -> balanced-brace scan w/ trailing-comma repair -> regex fallback). Harnesses are told to use it instead of bespoke regex.

THE INNER LOOP (inner_loop.py):
  online mode: for each batch -> parallel predict (ThreadPoolExecutor up to 32) -> score with check_answer -> learn_from_batch. Single pass. Models the deployment-time "learn as you go".
  offline mode: train with ground truth visible as the "prediction", multiple epochs, eval on val after each epoch, keep best checkpoint (get_state) and restore it (set_state) = early stopping. skip_train_eval shortcut for val-only search runs (saves ~15 min/system).
  Result dicts carry accuracy + correct/total + optional avg_f1/micro_f1 (computed by summing tp/fp/fn globally). JSONLLogger writes step/train_batch/val_epoch/eval_step rows incl prompt_len + prompt_hash per step -> the trajectory the proposer later reads.

THE FRONTIER (benchmark.py::compute_pareto_frontier):
  Not "highest accuracy wins" but a 2-objective Pareto set: maximize accuracy, minimize injected-context tokens. A candidate survives if no other has BOTH higher accuracy AND lower ctx cost. Persisted in frontier_val.json (_pareto list). This is what makes the search cost-aware, not just score-chasing, and it directly bounds harness bloat.

THE PROPOSER / MUTATION POLICY (SKILL.md) - this is where the "how to evolve well" knowledge lives:
  text_classification SKILL:
    - "Implement 3 new memory systems every iteration; mix exploitation + exploration." Banned from declaring the frontier optimal or stopping early (prevents the agent from rationalizing laziness).
    - ANTI-PARAMETER-TUNING: the named #1 failure mode is candidates that only tweak constants (pool sizes, retrieval k, char budgets). Rule: a candidate must change a FUNDAMENTAL MECHANISM (new retrieval algorithm, new prompt architecture, new learning strategy, new memory structure). Explicit self-critique step: "if predict()/learn_from_batch() logic is identical to base except numbers, REWRITE."
    - EXPLOITATION AXES enumerated (A=prompt template, B=memory content, C=selection algo, D=sizing, E=learning trigger, F=LLM usage in learning); "if last 3 iters explored the same axis, pick different ones" = forced diversification.
    - ANTI-OVERFITTING: no dataset-specific hints, never mention dataset names; general heuristics OK ("prioritize recent errors"). Guards the held-out generalization.
    - MANDATORY PROTOTYPE step: write a throwaway /tmp script exercising the mechanism on real logged examples, try 2-3 variants, before writing the final file. "copy a top performer then targeted-edit" for correct imports.
    - "Combining systems is valid" + name-drops prior art (DSPy, OPRO, Reflexion, CEIL) as a mutation source.
    - Self-authored POST-EVAL REPORTS (<=30 lines/iteration): what changed, which datasets moved + why, takeaway. Becomes proposer memory for the next iteration.
    - "Do ALL work in the main session, do NOT delegate to subagents -- constraints get lost when you delegate, leading to parameter-only changes." (text_classification only.)
  terminal_bench_2 SKILL (scaffold variant):
    - 1 candidate/iteration, ONE mechanism per candidate ("if tempted to add 'and also', that's a second candidate"). Mechanism-first: identify a specific failure mode from trajectories, then target it; never speculative.
    - Search space = ARBITRARY PYTHON: override any method of Terminus2 (_call_llm_with_tools, _parse_tool_calls, _execute_commands, _run_agent_loop, _summarize_context, prompt template path, etc), add tools, make raw API calls. Only hard constraint: subclass Terminus2 + class named AgentHarness (eval-harness compat). Cannot edit existing agents / meta_harness / wrapper.
    - HERE it DOES delegate: a 2-subagent workflow (analyze-trajectories subagent -> implement subagent), because the scaffold search space is large. "Deep-read failed AND successful trajectories" is called the most important step.

THE PROPOSER DRIVER (claude_wrapper.py) - reusable infra worth noting independently:
  - Wraps `claude -p --output-format stream-json --verbose`, parses the NDJSON event stream into a SessionResult (text, tool_calls w/ inputs+outputs, files_read/written w/ line counts, token usage incl cache_creation/cache_read, cost_usd, session_id, exit_code).
  - HERMETIC by default: --setting-sources "" + --disable-slash-commands + --strict-mcp-config + an empty --plugin-dir, so the proposer does NOT inherit the operator's local skills/MCP/plugins. The skill is injected explicitly via --append-system-prompt. This makes proposer runs reproducible regardless of who runs them.
  - log_session() writes a per-run dir: meta.json (scannable summary), response.md, events.jsonl (raw), artifacts/ (JSON code blocks auto-extracted from the response, e.g. pending_eval.json), tools/NNN_Tool.txt (one human-readable file per tool call, Read outputs de-numbered). This is a clean "agent run -> auditable artifact tree" pattern.
  - Streaming progress callback (one line per tool call), background pipe-drain threads, hard timeout -> kill + exit 124.

THE CONTRACT THAT MAKES IT WORK (anti-gaming, very relevant to ELAI's wiring doctrine):
  1. Proposer cannot run the benchmark (separation of mutate vs judge).
  2. Fitness is measured off-agent from artifacts the proposer didn't author (reward.json from Harbor; prompt_len/hash recorded inside call_llm, not self-reported).
  3. Test set is physically partitioned (separate results/ dir, only touched in Phase Final).
  4. Validation gate before scoring: import-check + (TB2) runtime smoke test; failed candidates logged as "failed" with score 0, iteration continues.
  5. Frontier is multi-objective so a candidate cannot win by spending unlimited context.

================================================================
4. KEY TECHNIQUES PORTABLE TO ELAI
================================================================

ELAI already has: FSM plan execution, the multi-agent loop (context-compiler/explorer/executor/auditor/memory-writer), wiring-enforcement (off-agent artifact gates), and a self-improvement/DGM track (H-43..H-48: execution-grounded fitness, eval firewall, lineage promotion). Meta-Harness is the SAME GENUS as the DGM track and validates several ELAI decisions; the steals are concrete mechanisms.

T1. MUTATOR / JUDGE SEPARATION ENFORCED BY TOOL DENY-LIST, not just by prompt.
  Where: text_classification/meta_harness.py::PROPOSER_ALLOWED_TOOLS + propose_claude; SKILL.md "You do NOT run benchmarks."
  Maps to ELAI: ELAI's DGM self-improvement loop + the wiring-enforcement "freeze-the-gate" rule (R8) and production-caller-gate. ELAI enforces this socially (auditor is the sole test runner) but the proposer/executor could still in principle invoke the gate. Meta-Harness's claude_wrapper gives the runnable enforcement: spawn the executor/proposer with the benchmark/gate commands OUTSIDE its allowed_tools / behind a deny-list, so "executor runs its own gate" becomes structurally impossible, not just rule-banned. Directly hardens the in-boundary failure both wiring-enforcement.md and production-caller-gate.md exist to prevent.

T2. MULTI-OBJECTIVE PARETO FRONTIER (quality vs context-cost), persisted across runs.
  Where: benchmark.py::compute_pareto_frontier (maximize accuracy, minimize injected ctx tokens); frontier_val.json _pareto.
  Maps to ELAI: ELAI's DGM promotion (H-45 worst-case+lineage scoring) and the efficiency/trust direction (cheaper via wire-dark/caching, $/resolved-task as the config-level metric). ELAI's self-improvement promotion is largely single-objective (fitness). Steal: make ELAI's promotion gate a Pareto set over (task-success, token/$ cost), so an evolved harness/agent-config that wins only by burning context is dominated and rejected. The off-agent context measurement trick (T6) supplies the cost axis cheaply.

T3. HERMETIC PROPOSER VIA `claude -p` WITH INHERITANCE DISABLED + FULL ARTIFACT CAPTURE.
  Where: claude_wrapper.py (build_command flags: --setting-sources "", --disable-slash-commands, --strict-mcp-config, empty --plugin-dir; skill injected via --append-system-prompt; log_session artifact tree).
  Maps to ELAI: ELAI dispatches agents via the Agent tool / `claude -p` (orchestrate-launcher.ps1 already shells `claude -p`). Steal two things: (a) the hermetic flag set so a dispatched ELAI sub-agent's run is reproducible and does NOT silently inherit the operator's local MCP/skills (relevant to the skill-import-policy + agent-dispatch-preamble guarantees -- right now "sub-agent context loading is NOT guaranteed" is a stated risk; explicit injection + empty inheritance closes it); (b) the stream-json->SessionResult->artifact-tree parser as a reusable telemetry layer for orchestrate runs (token/cache/cost accounting + per-tool-call audit files = exactly what the auditor and the usage-ledger J-28/J-29 want). Portable as-is (Python).

T4. THE MUTATION-POLICY-AS-SKILL with explicit anti-degeneracy rules.
  Where: both SKILL.md files: anti-parameter-tuning, enumerated exploration AXES with forced rotation, mandatory-prototype-before-final, mandatory self-critique, "never declare done/optimal", mechanism-first-from-trajectories, anti-overfitting (no task-specific hints).
  Maps to ELAI: ELAI's anti-spiral skill (4 stagnation patterns + 5 personas) and the DGM trap/behavioral-recall (H-44). The Meta-Harness axes + "if last 3 iters used the same axis pick another" is a cleaner, cheaper diversification operator than persona-switching and could feed anti-spiral. The "candidate must change a fundamental mechanism, not tune constants; self-critique by diffing predict()/learn() against base" is a direct anti-degenerate-edit rule ELAI's self-improvement proposer should carry. The mandatory-prototype step (throwaway /tmp script on real logged examples before the real edit) is a grounded-by-default practice ELAI can require of its executor for risky mechanisms.

T5. SELF-WRITTEN POST-EVAL REPORTS as proposer memory + append-only JSONL history.
  Where: SKILL.md Step 0 (write a <=30-line report per past iteration lacking one); evolution_summary.jsonl (one row/candidate: hypothesis, axis, delta, components, timing, rollout metrics).
  Maps to ELAI: ELAI's memory-bank (activeContext/changelog/progress) + the ELAI-FEATURE-MAP living source-of-truth + handoff-protocol. The structured row schema (hypothesis + falsifiable + outcome delta + component tags) is a tighter "what was tried and did it move the number" ledger than ELAI's prose changelog, and is exactly what a self-improvement track needs to avoid re-trying dead branches. Steal the schema for a DGM lineage/attempt log.

T6. OFF-AGENT CONTEXT-COST MEASUREMENT inside the LLM call wrapper.
  Where: memory_system.py::call_llm records per-thread last_prompt_len/hash/text; inner_loop computes context_len = prompt_len - len(input) (the harness never self-reports its cost).
  Maps to ELAI: the J-28/J-29 token ledger + usage window. ELAI measures cloud tokens from the API response; the prompt-hash trick additionally gives a cheap dedupe/cache-hit signal and an injected-context metric per turn that the agent cannot inflate or hide. Cheap, portable.

T7 (validation gate ladder): import-check -> runtime smoke (1 task x 1 trial) -> full eval -> high-trial final eval.
  Where: validate_candidate + smoke_test + Phase Final 5-trial.
  Maps to ELAI: mirrors ELAI's layered wiring gate (Layer1 compile -> Layer3 entrypoint smoke -> Layer4 runtime). Confirmatory, not novel for ELAI, but the "denoise winners with extra trials at the end only" is a cost-saving pattern for ELAI's noisy auditor/eval runs.

NON-PORTABLE / FLAG:
  - Harbor + Runloop dependency (terminal_bench_2 fitness) is TB2-specific cloud-eval infra; ELAI would supply its own fitness (its benchmark/auditor), not this.
  - The whole framing assumes REPEATED tasks with a stable measurable metric and a clean held-out split (ONBOARDING.md screens this hard). ELAI's plan-execution work is often one-off; Meta-Harness applies to ELAI's RECURRING loops (the agent loop itself, routing policy, context policy, the self-improvement track), NOT to one-shot plan phases.
  - text_classification MemorySystem (predict/learn_from_batch) is a narrow supervised interface; the GENERAL transferable artifact is the OUTER LOOP + proposer + frontier + contract, not that interface.
  - Proposer uses Opus at effort=max with ~40-min timeouts per iteration: expensive. Subscription-auth dodge (popping ANTHROPIC_API_KEY) is a rate-limit hack, not a design.

================================================================
5. BENCHMARK NUMBERS (quoted, NOT re-run here)
================================================================
  Terminal-Bench 2.0 hard split (terminal_bench_2 SKILL.md, line 30): the released reference run starts from a 28.1% KIRA baseline and reaches 46.5% on this split. Model: Claude Opus 4.6. Eval = 89 official TB2 tasks; search at 2 trials/task, winners re-eval at 5 trials.
  text_classification: no headline number stated in the shipped repo text; datasets USPTO / Symptom2Disease / LawBench (config.yaml lists USPTO/Symptom2Disease/LawBench; data/ also ships aegis2, crime_prediction, finer), split 200train/50val/100test, default model openrouter/openai/gpt-oss-120b. Per-dataset val accuracy is the search metric; absolute scores are produced at runtime, not committed.
  (Paper arXiv:2603.28052 likely has the full table; not fetched.)

================================================================
6. DEPENDENCIES / STACK
================================================================
  Python, managed by uv (pyproject.toml per example). PyYAML (config). litellm (LLM calls in TB2 scaffolds + caching helper; OpenAI-compatible/openrouter in text_classification llm.py). python-dotenv (TB2). stdlib: subprocess, threading, concurrent.futures (ThreadPoolExecutor), hashlib(md5), argparse, signal, json/jsonl.
  External: the Claude Code CLI (`claude -p`) is REQUIRED -- it is the proposer; uses Pro/Max SUBSCRIPTION auth by default (wrapper pops the API key). terminal_bench_2 additionally needs Harbor + Runloop (RUNLOOP_API_KEY) for the TB2 eval (scripts/run_eval.sh) and harbor.agents.terminus_2.Terminus2 as the scaffold base class.
  Models seen: anthropic/claude-opus-4-6 (TB2 solver + proposer "opus"), openrouter/openai/gpt-oss-120b (text_classification solver default).
  No Rust, no DB; state is flat files (JSON/JSONL) under logs/<run_name>/.
