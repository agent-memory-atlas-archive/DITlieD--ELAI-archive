GPT second-opinion + research (capability rule, NOT a hard rule)

Loaded when: facing a hard architecture / design / algorithm fork, a bug that survived 3 failed hypotheses, wanting a cross-model diversity check on a conclusion about to be acted on, or wanting GPT's independent deep-research on an external-fact question. The point of this rule is that the main LLM reaches for this UNPROMPTED, the operator should not have to say "ask GPT".

WHAT THIS IS
The gpt-bridge MCP drives a pool of ChatGPT Pro accounts (browser, harvested-token bypass; built at .claude/mcp/gpt-bridge, handoff at workflow/handoffs/gpt-bridge-mcp.md). It gives the main LLM a DIFFERENT model family's reasoning and GPT's own deep-research, on demand. Its unique value is MODEL DIVERSITY (a second opinion that did not come from the same weights you are reasoning with) plus a cheap hands-off research run, NOT raw web facts (Oracle / researcher already does grounded, tier-classified web research and stays the primary path for that).

TOOLS
gpt_chat(prompt) - synchronous one-shot, highest tier (Pro > Thinking), auto-rotates on limit. Returns {text, account, tier, effort, ok}. For a quick cross-model second opinion (OpenAI GPT) on a SELF-CONTAINED question. It blocks the single-threaded server for the few seconds it runs, fine for a one-shot, never loop it.
grok_chat(prompt, effort?) - synchronous one-shot to xAI Grok, a SECOND model family distinct from OpenAI GPT (so an even stronger diversity vote when you want a non-GPT view). effort = auto (default) | fast | expert | heavy. heavy is the SuperGrok Heavy "Team of Experts" multi-agent mode (routed to the one Heavy account) for the hardest forks; expert is more reasoning than fast. Returns {text, account, effort, ok}. Grok has NO GPT-style deep-research mode and NO vision/image - for a hard grok question use effort=expert or heavy, not a research tool. Auto-rotates across grok accounts on limit; one-shot only, never loop it.
  MULTI-SESSION: grok is SERIALIZED across ALL workflow sessions by a cross-process mutex (the bridge is a shared install; running grok from several sessions at once used to pile up ~10 windows and trip grok's "session expired" gate). So if another session is mid grok-run, your grok_chat WAITS up to ~90s and then, if still busy, returns a clean "grok busy in another session" error - that is expected, just retry shortly; it is NOT a failure of the bridge. Heavy runs can hold the lock for minutes. (chatgpt is NOT serialized.)
gpt_deep_research(query) > gpt_deep_research_status(job_id) (poll to state=done) > gpt_deep_research_result(job_id). Async, ~10-15 min, GPT runs a full cited deep-research. For GPT's INDEPENDENT research take or a cheap hands-off run. (ChatGPT only; grok has no equivalent.)
(gpt_image / gpt_image_status / gpt_image_result / gpt_image_retry, gpt_harvest, gpt_pool_status also exist but are unrelated to reasoning offload.)

WHEN TO REACH FOR IT (unprompted)
- You would otherwise HEDGE on an architecture / design / algorithm fork (ties to GRC trigger 4 and main-llm-role two-approach mandate): take one cross-model vote before recommending.
- A bug SURVIVED 3 failed hypotheses (ties to token-discipline reasoning-after-action / anti-spiral): a different model may break a pattern-lock you cannot.
- You are about to ACT on a non-trivial conclusion and want an independent sanity check.
- You want GPT's independent deep-research on a current-external-fact question (API / framework / store-rule state past training).

WHEN NOT TO
- Routine steps, inner loops, or anything LOAD-BEARING. The bridge is browser-subscription + rate-limit-fragile (single-IP multi-account trips suspicious-activity logouts; accounts run "out of Pro messages"). Occasional use only.
- As a GROUNDING source. A model's opinion is WEAK grounding under grounded-by-default (below official docs / code-just-read / passing tests). It INFORMS, it does not GROUND. Reconcile every GPT claim against real evidence before acting; if it conflicts with code/docs/tests, the evidence wins.
- When the question needs repo / code context. The bridge has NO access to the codebase, plan files, or anything the main LLM holds. You must serialize the MINIMAL relevant snippet/spec into the prompt. If that is infeasible or would be lossy, do not use it.
- For pure current-fact web research, prefer Oracle / researcher (integrated, tier-classified). Use gpt_deep_research for GPT-diversity, not as an Oracle replacement.

HOW TO USE WELL
1. Frame a TIGHT, self-contained question. Paste the minimal snippet/spec plus the specific decision. Assume zero repo knowledge on GPT's side.
2. gpt_chat for fast; gpt_deep_research (poll status to done, then result) for a deep run.
3. Treat the answer as ONE vote. Surface it as "GPT second opinion (unsearched, one model's view):" and reconcile against grounded evidence per grounded-by-default. The main LLM owns the final call, never let GPT auto-decide.
4. Do not chain it into the agent pipeline or make it a standing step.

COST / RELIABILITY / ESCALATION
No API cost (subscription) but flaky: logouts, IP rate-limits, quick-vs-full DR variance. Fine for a one-shot, do not loop or fan out. If consulting GPT ever becomes ROUTINE or load-bearing, that is the signal to stop using the browser bridge and move to the vendor API (deterministic, version-pinnable, context-controlled), per the recovered analysis at .claude/mcp/gpt-bridge/state/research-reasoning-offload.md (route by task shape; Opus default for repo-scale/agentic; GPT-5.x for terminal/tool-heavy + long-context; vendor APIs not browser subs for any production delegation).
