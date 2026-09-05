Source: ypi — a recursive coding agent
URL: https://raw.works/ypi-a-recursive-coding-agent/ (blog post 2026-02-12)
Repo: https://github.com/rawwerks/ypi (reviewed source)
Version: v0.5.1 (2026-03-23)
License: MIT
Reliability: MEDIUM (reviewed source, maintained, MIT, not peer-reviewed)
Grounding: ypi launcher (rawwerks/ypi/ypi), install.sh, SYSTEM_PROMPT.md, repo README all fetched directly.

WHAT YPI IS

ypi = Pi coding agent + rlm_query recursive sub-call primitive.
Named after the Y combinator: "ypi is Pi that can call itself."
Pi = earendil-works/pi-coding-agent (bash REPL coding agent; separate package).
ypi = Pi extension that adds the RLM recursion layer on top of Pi.
Parent package pi (@earendil-works/pi-coding-agent) is a prerequisite; ypi installs it.

SIBLING COMPARISON (vs earendil-works/pi, which ELAI evaluated as REJECT)

pi: generic single-agent coding runtime with bash REPL; no self-delegation; flat agent.
ypi: adds rlm_query (spawns child Pi instances); adds workspace isolation (jj); adds budget controls.
Key differentiator: self-delegation with isolation. ypi is an RLM; pi is not (fails G5/G6).

RECURSIVE LOOP ARCHITECTURE

Three integrated pieces:
1. SYSTEM_PROMPT.md — externalized system prompt teaching recursive decomposition strategy
2. Context/REPL — $CONTEXT file + bash environment as the persistent executable environment
3. rlm_query "prompt" — native Pi tool that spawns a full child ypi process

Loop behavior per SYSTEM_PROMPT.md:
1. Size up the problem (wc -l, assess complexity, search before reading)
2. Search and explore (grep, find) before full reads
3. Delegate via rlm_query for sub-problems (synchronous default, --async for parallel)
4. Validate sub-agent output before proceeding
5. Do small tasks directly without delegation (no unnecessary recursion)
Context window management: "finite and non-renewable" — token budget awareness is a first-class design constraint; decompose before context fills.

RECURSION MODEL (from ypi launcher, verified source)

RLM_DEPTH (env): current depth, starts 0
RLM_MAX_DEPTH (env): max recursion limit, default 3
At max_depth: rlm_query is removed from available tools (prevents infinite recursion)
Each child gets its own jj workspace (file isolation); read-only fallback if jj unavailable
Parent reviews child changes via: jj diff; absorbs via: jj squash --from <child>
Children share the same Pi extension + tools, minus rlm_query at max depth

ENVIRONMENT VARIABLES (from ypi launcher, verified)

RLM_DEPTH        | int | 0       | incremented per recursive call
RLM_MAX_DEPTH    | int | 3       | hard depth ceiling; rlm_query removed at this depth
RLM_PROVIDER     | str | —       | LLM provider for sub-calls
RLM_MODEL        | str | —       | LLM model identifier for sub-calls
RLM_CHILD_MODEL  | str | —       | cheaper model override for depth > 0
RLM_CHILD_PROVIDER | str | —     | provider override for depth > 0
RLM_TIMEOUT      | int | —       | wall-clock seconds for entire recursive tree
RLM_MAX_CALLS    | int | —       | max total rlm_query invocations across tree
RLM_BUDGET       | float | —     | max dollar spend across tree
RLM_JJ           | int | 1       | workspace isolation via jj (default on)
RLM_EXTENSIONS   | int | 1       | Pi extensions enabled
RLM_CHILD_EXTENSIONS | int | —   | extension override for child processes
RLM_TRACE_ID     | str | —       | generated ID linking recursive sessions for tracing
RLM_SHARED_SESSIONS | int | 1    | session log sharing across recursive tree
RLM_JSON         | int | 1       | JSON mode + cost tracking
PI_TRACE_FILE    | path | —      | trace log path with timing data
YPI_SHELL_HELPER | int | —       | enables shell-compatible rlm_query helper

PRIMITIVES (public commands, from README and launcher)

rlm_query "prompt"    | spawns child agent synchronously; returns string output
rlm_query --async "prompt" | spawns child agent asynchronously; enables parallelism
rlm_cost              | reports spend; supports --json; reads RLM_JSON tracking
rlm_cleanup           | cleanup utility (sessions, temp state)
rlm_sessions          | list active/completed recursive sessions
rlm_parse_json        | helper for parsing JSON output from sub-agents
ypi "query"           | main launcher; bunx ypi "query" works without install

INSTALL / RUNTIME

npm install -g ypi   | global install via npm
bun install -g ypi   | global install via bun (preferred per CLAUDE.md bun-not-node rule)
bunx ypi "query"     | run without install
curl -fsSL https://raw.githubusercontent.com/rawwerks/ypi/master/install.sh | bash | direct install
.pi-version file: version pinning; make doctor checks compatibility

WORKSPACE ISOLATION MODEL

jj (Jujutsu VCS) required for full workspace isolation.
Each child rlm_query spawns in its own jj workspace: changes are isolated.
Parent absorbs child changes explicitly: jj squash --from <child-workspace>
Without jj: children default to read-only access unless explicitly enabled.
Implication for ELAI integration: jj is a hard dependency for write-capable recursive coding.

BENCHMARK (from blog post, partial)

77.6% on LongMemEval mentioned in context of an earlier tool-use iteration. [MEASURED, single data point, not the final architecture]
Full benchmark breakdown not published for ypi specifically. [UNVERIFIED]

RLM GATE ASSESSMENT (G1-G7, from verified source)

G1 model-call outer shape: PASS — ypi "prompt" in, answer out, standard interface
G2 prompt externalization: PASS — SYSTEM_PROMPT.md is external; $CONTEXT is env file
G3 symbolic handle: PASS — file paths, jj workspace handles, $CONTEXT variable
G4 persistent executable environment: PASS — bash REPL persists across steps
G5 programmatic subcalls: PASS — rlm_query is a native tool callable from bash
G6 model-chosen decomposition: PASS — SYSTEM_PROMPT teaches model to decide when to recurse
G7 symbolic intermediate state: PASS — intermediates stay in files/env vars, not re-verbalized
Classification: FULL RLM (passes all G1-G7)

FAILURE MODES

Recursion at max_depth silently loses rlm_query access — root should not assume unlimited depth.
Context window fills at deep recursion if children return verbose output — SYSTEM_PROMPT explicitly warns.
jj unavailable = children read-only, write tasks silently degrade.
Budget/timeout exhaustion terminates tree mid-task without guaranteed rollback.
GEPA-style optimization of SYSTEM_PROMPT likely overfits (per sibling rlm-as-memory finding).

IMPLEMENTATION PITFALLS (do NOT do this)

Do not call rlm_query in a tight loop without checking RLM_MAX_CALLS — tree budget exhaustion.
Do not return large artifacts as strings from child to parent — verbalize handles (file paths), not content.
Do not skip jj setup if children need write access — read-only fallback is silent.
Do not use RLM_CHILD_MODEL without verifying the cheaper model can handle the sub-task complexity.
Do not use foreground rlm_query for independent parallel sub-tasks — use --async.
Do not assume output from a child is correct without validation (SYSTEM_PROMPT rule).

RELATIONSHIP TO OTHER MAPS

Base RLM algorithm: rlm-interface.md (arXiv:2512.24601)
RLM as memory: rlm-as-memory-rawworks-interface.md
RLM rubric (is ypi an RLM?): rlm-rubric-recursive-coding-agents-interface.md
pi agent (rejected predecessor): earendil-works/pi evaluation in ELAI session logs

UNVERIFIED

Full LongMemEval result for ypi's current architecture (v0.5.1).
Behavior when RLM_BUDGET is hit mid-deep-recursion (rollback behavior not documented).
Whether ypi.sh at https://ypi.sh is a distinct hosted deployment or just the install script.
jj version compatibility matrix.
