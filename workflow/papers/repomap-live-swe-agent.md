repomap: live-swe-agent
source: C:\Users\user\Downloads\new\live-swe-agent-main
arxiv: 2511.13646
date: 2026-06-15


1. WHAT IT IS

Live-SWE-agent is a methodology + config layer on top of mini-swe-agent (SWE-agent/mini-swe-agent).
It is NOT a new agent framework with source code -- the entire contribution is in 3 YAML config files and a paper.
The "live" in the name means runtime self-evolution: the agent can create, write, and invoke its own Python tools
on the fly during a single trajectory. The agent literally writes scripts to disk within a bash session and runs
them as sub-processes, extending its own capability set without any external tool-registration step. There is no
continual/online learning, no streaming inference change, no model fine-tuning at runtime -- the self-evolution is
purely behavioral: the LLM generates Python utilities that it then calls.

The paper's key claim: software agents are software systems, and LLM-based agents already possess intrinsic
capability to extend/modify their own behavior at runtime. A minimal open scaffold that instructs the agent to
create task-specific tools can outperform manually engineered proprietary scaffolds.


2. STRUCTURE MAP

config/livesweagent.yaml
  Base general-use config. No Docker environment. step_limit=0 (unlimited), cost_limit=$3.
  mode: confirm (pauses for human approval at each step -- interactive/demo mode).
  Has a full Jinja2 instance_template with tool-creation examples embedded in the prompt.
  Includes the action_observation_template with the reflection trigger.

config/livesweagent_swebench.yaml
  SWE-bench Verified evaluation config. Docker environment, cwd=/testbed, timeout=60s.
  step_limit=250, cost_limit=$3. model_name="" (caller sets it). temperature=0.
  Instance template includes explicit PR description framing (wraps task in <pr_description> tags).
  Submission command: echo COMPLETE_TASK_AND_SUBMIT_FINAL_OUTPUT && git add -A && git diff --cached

config/livesweagent_swebench_pro.yaml
  SWE-bench Pro evaluation config. Structurally identical to swebench.yaml.
  Differences: cwd=/app (Pro benchmark uses /app, Verified uses /testbed), start_with_bash=true,
  pull_timeout=600 (longer Docker pull timeout, Pro uses more diverse/heavier images).
  All limits identical to swebench.yaml.

config/README.md
  Notes that swebench.yaml and swebench_pro.yaml reproduce paper results exactly.
  livesweagent.yaml is for general/interactive use.


3. ARCHITECTURE + METHOD

The agent loop is the mini-swe-agent ReAct loop:
  - Inject system_template + instance_template into model context.
  - Each turn: model outputs THOUGHT section + one bash code block.
  - Framework executes the bash command in a subshell (stateless, new subshell per step).
  - Output fed back via action_observation_template.
  - Repeat until submission command or step_limit/cost_limit hit.

Key architectural fields:

system_template:
  Minimal. Instructs one-bash-block-per-response. Requires THOUGHT section. Rejects multi-block responses.
  Notably does NOT enumerate tools -- there are no pre-registered tools, only bash. The model's tool set IS bash.

instance_template (the main innovation surface):
  - Wraps task as {{task}} for general use, <pr_description>{{task}}</pr_description> for SWE-bench.
  - Embeds a 6-step recommended workflow: analyze -> reproduce -> edit -> verify -> edge-cases -> submit.
  - Contains explicit "Creating your own tools" section with concrete Python tool creation examples.
    The instruction is: "You should at least create a simple edit tool that can help you effectively
    edit arbitrary files instead of using bash commands". This is the live self-evolution prompt.
  - Working-directory discipline: general config = unrestricted; SWE-bench = /testbed; Pro = /app.
  - Stateless subshell warning: "Directory or environment variable changes are not persistent."
    Workaround shown: prefix commands with env-var assignment, or write/load from files.
  - Provides nl + sed snippet for viewing specific line ranges (explicit token-efficient reading pattern).
  - Submission is a terminal action -- the model cannot continue after submitting.

action_observation_template (post-step reflection trigger):
  After each command, if output < 10000 chars, appends this literal text to the returned observation:
    "Reflect on the previous trajectories and decide if there are any tools you can create
    to help you with the current task. Note that just because you can use basic bash commands
    doesn't mean you should not create any tools that can still be helpful."
  This is the ONLY mechanism driving the self-evolution behavior. It is a one-line natural-language nudge
  injected at every successful step. No structured tool-registry, no meta-learning, no feedback loop beyond this.
  Long outputs (>= 10000 chars) suppress the reflection nudge and show only head + tail (5000 chars each).

format_error_template:
  Fires when model outputs != 1 bash block. Restates the one-action rule. Tells model how to submit.

step_limit:
  0 in base config (unlimited, relies on cost_limit).
  250 in evaluation configs (matches SWE-bench standard).

cost_limit:
  $3 per trajectory across all three configs.

mode: confirm
  Only in base config. Human-confirmation mode for interactive use. Evaluation configs omit this (run unattended).

environment_class: docker
  Evaluation configs only. Isolated Docker environment per task. Base config has no environment block
  (runs in caller's shell). Docker timeout=60s per command. Pro adds pull_timeout=600s.

Environment variables injected (all configs):
  PAGER=cat, MANPAGER=cat, LESS=-R   -- prevents pagers from blocking bash output
  PIP_PROGRESS_BAR=off, TQDM_DISABLE=1  -- suppresses progress bars from polluting output

model:
  temperature=0 (deterministic greedy). All configs. drop_params=true in base config (strips unsupported params).
  model_name="" in evaluation configs -- caller specifies the model externally (paper tested Opus 4.5, Gemini 3 Pro).

Jinja2 templating: system and instance templates are Jinja2. MacOS conditional in base config:
  {%- if system == "Darwin" -%} ... use `sed -i ''` instead of `sed -i` {%- endif -%}
  This means the framework passes platform info into the template.


4. KEY TECHNIQUES PORTABLE TO ELAI

A. STEP-LEVEL REFLECTION NUDGE injected in the observation template.
   Location: action_observation_template in all three YAMLs.
   Quote: "Reflect on the previous trajectories and decide if there are any tools you can create..."
   This is appended AFTER the command output, before the model generates its next step.
   ELAI mapping: ELAI's executor receives (plan-step, current-output) and generates the next action.
   The observation template equivalent is the message ELAI sends back to the model after each tool call.
   Add a trailing reflection line to ELAI's step-return message: "Reflect on prior steps and consider
   whether a reusable sub-script or helper would accelerate the remaining steps in this task."
   This is a zero-cost prompt injection on the sidecar response path (runtime_client.rs / the message
   assembly before returning to the executor's next iteration). No model change required.
   PORTABLE: yes, directly applicable.

B. IN-TRAJECTORY TOOL SYNTHESIS (the live self-evolution core).
   Location: instance_template "Creating your own tools" section.
   The agent is INSTRUCTED to write Python helper scripts to disk within the task session.
   These scripts are task-specific -- not general utilities. The model sizes the tool to the current problem.
   Quote: "The tools you create can be for any purpose, it does not need to be general, instead think
   about how it can help you specifically with the current task at hand."
   ELAI mapping: ELAI's executor currently generates edits to the target repo. It does not generate
   auxiliary scripts for its own workflow. Adding this instruction to the executor's instance prompt
   would let the executor write task-local Python search/diff/analysis scripts before applying the main edit.
   This is especially useful for ELAI's explorer phase (generating a task-specific analysis script to map
   a codebase area before the main edit, then discarding it). The scripts live in a scratch dir and are
   not committed (they're git-untracked throwaway tools).
   PORTABLE: yes, with one constraint -- ELAI's executor runs in a Windows/PowerShell shell, not bash.
   The Python tool creation pattern still applies; only the shell examples need adjusting.

C. STATELESS SUBSHELL DISCIPLINE + ENV-FILE WORKAROUND.
   Location: instance_template "Command Execution Rules" section.
   Quote: "Directory or environment variable changes are not persistent. Every action is executed in a
   new subshell. However, you can prefix any action with MY_ENV_VAR=MY_VALUE cd /path/to/working/dir &&
   ... or write/load environment variables from files."
   This is an explicit prompt-layer solution to the stateless-shell problem. The agent learns to persist
   state via files rather than relying on shell session state.
   ELAI mapping: ELAI's executor already runs commands step by step, but the prompt may not explicitly
   teach the model that env changes don't persist. Adding this explicit statement to the executor's prompt
   would reduce failures where the model assumes a prior cd or export is still in effect. Pair with
   "write state to .elai_task_state.json and load it at each step" as the explicit workaround.
   PORTABLE: yes, prompt-only change.

D. OUTPUT TRUNCATION WITH HEAD+TAIL (output budget management).
   Location: action_observation_template, the Jinja2 branch for output >= 10000 chars.
   Quote: head 5000 chars + tail 5000 chars, middle elided, length reported.
   ELAI mapping: ELAI's runtime_client.rs passes sidecar output back to the executor. If the output is
   a large file read or a long test run, it can blow the local context window (32K ceiling). Adding
   a token-budget truncation layer in runtime_client.rs that applies head+tail slicing when output
   exceeds a configurable char threshold would match this pattern. The tail is important (test results
   often at the end); head-only truncation loses the verdict.
   PORTABLE: yes, implementation in runtime_client.rs output post-processing.

E. WORKFLOW STEPS AS PROMPT (reproducibility-first ordering).
   Location: instance_template "Recommended Workflow" section.
   The 6-step order: analyze -> reproduce-issue-with-a-script -> edit -> verify-fix -> edge-cases -> submit.
   The reproduce step is explicit and comes BEFORE the edit. This enforces a test-first discipline at prompt level.
   ELAI mapping: ELAI's executor prompt currently drives the model toward editing. Adding an explicit
   "step 2: write a script that reproduces the issue, run it, see it fail" instruction before "step 3: edit"
   would improve correctness on bug-fix tasks. This is the ELAI analog of the TESTING_FIRST FSM state --
   but pushed into the model's own reasoning within a single executor run rather than a separate FSM state.
   For SWE-bench-style bug-fix tasks in ELAI's executor, this ordering is directly applicable.
   PORTABLE: yes, prompt-only addition to executor's instance template.

F. $3 COST CEILING PER TRAJECTORY.
   Location: cost_limit field in all configs.
   This is enforced by mini-swe-agent's cost tracking (not revealed in the config structure itself, but
   the field exists). $3 at Sonnet 4.6 pricing (~$3/$15 per M tokens in/out) allows roughly 200K output
   tokens or ~1M input tokens. At Opus 4.5 pricing it's tighter (~100K output tokens).
   ELAI mapping: ELAI has a step_limit but not a dollar cost_limit per task. Adding a per-task budget
   cap in the FSM (checked before each executor step via the runtime cost tracker) would prevent runaway
   trajectories. This is especially important for cloud-routed tasks hitting Opus 4.5.
   PORTABLE: requires cost-tracking integration with the FSM, not a prompt change. Medium complexity.

NON-PORTABLE ITEMS:
   - Docker environment isolation (environment_class: docker, cwd=/testbed|/app): ELAI operates on a local
     Windows development tree, not sandboxed containers. The isolation is by ELAI's FSM scope rules, not Docker.
   - git add -A && git diff --cached as submission command: ELAI's commit discipline is manual and human-gated;
     auto-commit on task completion is blocked by ELAI's rules (no-auto-commit memory feedback).
   - start_with_bash: true (Pro config): mini-swe-agent specific; ELAI uses PowerShell as the primary shell.
   - MacOS sed -i '' platform conditional: irrelevant to Windows.


5. BENCHMARK NUMBERS

From README.md (stated claims, source = arxiv:2511.13646 + leaderboard at live-swe-agent.github.io):

  Claude Opus 4.5 + Live-SWE-agent: 79.2% on SWE-bench Verified (Nov 24th 2025 date)
    Stated as "leading all current open-source scaffolds, very close to Anthropic's internal
    manually engineered scaffold for Opus 4.5"

  Gemini 3 Pro + Live-SWE-agent: 77.4% on SWE-bench Verified (Nov 20th 2025 date)
    Stated as "outperforming all available models including Claude 4.5" at that date

  Live-SWE-agent: 45.8% on SWE-bench Pro (Nov 17th 2025, model not stated in README for this number)

Context on these numbers:
  These are Nov 2025 claims. As of 2026-06-15 the leaderboard may have been surpassed.
  The paper itself is arxiv:2511.13646 (preprint, not peer-reviewed as of mapping date).
  The comparison claim is "open-source scaffolds" + "proprietary commercial scaffolds" -- verify
  the comparison set before citing these as current SOTA.

NOTE: These numbers are NOT verified against the live leaderboard in this mapping run.
The live-swe-agent.github.io leaderboard was not fetched; the numbers come from README.md only.


6. DEPENDENCIES / STACK

Runtime framework: mini-swe-agent (github.com/SWE-agent/mini-swe-agent)
  Installed via the mini-swe-agent guide (not replicated in this repo).
  Invoked as: mini --config config/livesweagent.yaml

Template engine: Jinja2 (used for system_template, instance_template, action_observation_template rendering)
  Variables injected: {{task}}, {{system}}, {{release}}, {{version}}, {{machine}}, {{output.returncode}},
  {{output.output}}, {{actions|length}}

Model interface: LiteLLM-style (model_name="" means the caller supplies the model string externally)
  Confirmed models used in paper: Claude Opus 4.5, Gemini 3 Pro.
  temperature=0 in evaluation configs (greedy/deterministic decoding).

Environment: Linux bash (evaluation), Docker (evaluation configs), native shell (base config)
  The framework runs on Linux in evaluation mode. The agent itself executes bash commands.
  No Python version specified in configs. Tool scripts the agent creates use Python 3 by convention
  (#!/usr/bin/env python3 in examples).

Evaluation benchmark integration:
  SWE-bench Verified: Docker image with testbed at /testbed, standard git-diff submission
  SWE-bench Pro: Docker image with /app working dir, pull_timeout=600 for heavier images

No additional Python dependencies visible in this repo beyond mini-swe-agent itself.
The "tools you create" are synthesized at runtime by the model and have no fixed dependency list.
