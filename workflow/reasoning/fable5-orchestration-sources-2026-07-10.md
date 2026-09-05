Fable 5 orchestration — consolidated notes from 4 sources
Compiled 2026-07-10. Topic: Anthropic benchmarking Fable 5 as an orchestrator that delegates to cheaper models, plus a community "agentic OS" builder's guide built on that pattern.

SOURCE STATUS (grounding honesty)
- SVG file (C:\Users\user\Downloads\Untitled-2026-07-09-0432.svg): FULLY EXTRACTED verbatim below. This is the richest source. It is the "Fable 5 builder's guide, nine loops" diagram and matches the @Av1dlive post theme.
- x.com/Av1dlive/status/2074169173178212621: could NOT fetch directly (HTTP 402, X paywalls the API to logged-out fetchers). Live-web model pools (perplexity, grok) were exhausted this session. Mapped by theme to the SVG builder's guide above (same "nine loops / agentic OS" content). Tweet body itself not retrieved verbatim.
- x.com/ClaudeDevs/status/2074900291062034618: could NOT fetch directly (HTTP 402). Maps by theme to Anthropic's Fable-5-as-orchestrator cost-pattern announcement. Body not retrieved verbatim.
- reddit.com/r/ClaudeAI/comments/1ur2ml9 ("Anthropic just benchmarked Fable 5 orchestrates..."): could NOT fetch directly (Claude Code cannot fetch reddit.com or old.reddit.com). Its central claim (the benchmark) is grounded below via third-party tech press citing Anthropic.
- The Anthropic benchmark numbers below are TIER-3 (third-party press citing Anthropic), not read off the primary Anthropic post directly. Treat as high-confidence-but-secondhand until confirmed against anthropic.com. Sources listed at the end.

================================================================
PART 1 — THE ANTHROPIC BENCHMARK (the ClaudeDevs / Reddit theme)
================================================================

Core claim: Fable 5 is expensive to run solo, so Anthropic published cost-saving patterns that use Fable 5 as a MANAGER/ADVISOR delegating the bulk work to cheaper models (Sonnet 5, Haiku 4.5). Each pattern keeps near-Fable-5 quality at a fraction of the cost. Both run through Claude Managed Agents, each sub-agent using its own cache so context is not paid for twice.

Pattern A — ADVISOR (a.k.a. Executor–Advisor / "Oracle")
- Shape: Sonnet 5 is the executor and runs every turn; it calls Fable 5 only when it needs judgment (roughly once per task).
- Benchmark: on SWE-bench Pro, Sonnet 5 + Fable 5 advisor reaches ~92% of Fable 5's solo score at ~63% of the cost.

Pattern B — ORCHESTRATOR–WORKERS
- Shape: Fable 5 is the planner/orchestrator; it fans work out to Sonnet 5 worker agents that execute in their own context windows.
- Benchmark: on BrowseComp, reaches 96% of Fable 5's solo performance at 46% of the cost.
- Worked cost example (20-fact task): team (Fable orchestrator + Sonnet workers) ≈ $1.61 vs single Fable doing all 20 ≈ $4. About 2.5x cheaper AND ~3x faster (194s vs 608s), with >80% of tokens billed at the cheaper worker rate.

Pricing context (why the delegation matters)
- Fable 5: $10 per million input tokens / $50 per million output.
- Opus 4.8: $5 / $25 (half of Fable).
- Sonnet 5: intro $2 / $10 (five times cheaper than Fable), through Aug 31 2026.
- Anthropic extended free Fable 5 access (reported cutoff July 12 2026).

Anthropic's own framing (from the Fable 5 prompting guide, per press): delegation is a headline improvement — Fable 5 is described as "significantly more dependable at dispatching and sustaining parallel subagents." The one seat that consistently earns the premium price is the orchestrator.

================================================================
PART 2 — THE BUILDER'S GUIDE (SVG, verbatim) — @Av1dlive "nine loops"
================================================================

Title block:
  AGENTIC OS · THE WORKFLOWS
  Fable 5 builder's guide · nine loops · 3 × 3, numbered left to right

Nine loops (3x3 grid). Each is a self-contained autonomous-agent pattern built around a cheap-model loop with Fable 5 used sparingly for judgment.

01 · THE HEARTBEAT — one tick of loop.sh
  BUILD 3 · "nothing grades its own homework · four parties, last one deterministic"
  Flow: Signals (git log · issues · CI) -> Triage (cheap model, reads the tick) -> Conductor (Fable 5 · xhigh, read-only, picks ONE item) -> Worker (cheap model, isolated worktree) -> Verifier (fresh Fable 5 · high, sees spec + diff only) -> Gate (verify.sh, deterministic).
  Sub-labels: actionable / execute / diff / PASS (quiet -> exit 0) / Review queue (queued: <skill> · yours) / FAIL -> trust-log fail (FAILED: <skill> in STATE.md · worktree kept) / trust.tsv (pass/fail per skill).
  Routing: auto -> commit + open PR; queue/watch -> review: in worktree.
  Cadence: cron 07:00 weekdays · next tick.
  Economics: quiet tick ≈ $0.01 · actionable tick ≈ $0.86 · daily janitor ≈ $2.56/day.
  Exit codes: 0 quiet/done · 1 iteration cap · 2 reroute (never iterate on a swapped model) · 3 budget.

02 · ORCHESTRATOR–WORKERS — "fan out · Fable plans, cheap models execute"
  Orchestrator (Fable 5) -> Plan -> Main loop; fans out to Worker 1/2/3 (each Sonnet 5) in a worker loop. "Fan out."
  (This is Anthropic's Orchestrator–Workers pattern from Part 1.)

03 · EXECUTOR–ADVISOR (ORACLE) — "cheap loop every turn · expensive judgment on demand"
  Executor (Sonnet 5, runs every turn) -> Main loop; on demand calls Advisor (Fable 5) via a Tool call; advisor sends advice back.
  (This is Anthropic's Advisor pattern from Part 1.)

04 · THE TRUST LEDGER — "autonomy is per skill, not per loop"
  BUILD 4 · "autonomy is per skill, not per loop"
  Tiers: new skill -> watch (draft-only) -> queue (verified drafts wait; you review with coffee) -> auto (ships unattended).
  Promotion thresholds: watch->queue at 10+ runs · ≥90%; queue->auto at 20+ runs & ≥95% (also shown "20 runs · 95%").
  Demotion: any fail -> automatic demotion · ALERT -> stderr -> cron mails you.
  Ledger: trust.tsv: skill · runs · passes — tier recomputed on every write.

05 · STANDING GOALS — "finishing IS the enrollment"
  BUILD 5 · "finishing IS the enrollment"
  When the work is finished it writes goals/<name>.md (predicate = a shell command; on-violation: wake me).
  verify-goals.sh runs every predicate (timeout 60s each), cron 07:30 daily.
  exit 0 -> satisfied · last-pass stamped · row -> goal-ledger.tsv.
  exit 1 -> status: VIOLATED (flipped in the file) -> Sentinel (suspects: merges since last-pass; detects, fixes nothing) -> normal pipeline (the loop fixes it) -> goal · suspects · policy -> you -> fix -> re-verify.
  retire-when: human decision, logged.
  Wall: "a goal you only verify once is an assumption with a timestamp · flaky predicate -> quarantine, never delete."

06 · QUORUM — "three cheap models vote before the expensive one wakes"
  BUILD 7 · "three cheap models vote before the expensive one wakes"
  Signals (commits · issues · CI) -> three Voters: Voter 1 = deepseek-v4-flash, Voter 2 = qwen-3.6, Voter 3 = kimi-k2.6 -> "actionable?" -> Conductor (Fable 5 wakes on yes; on no -> quiet (V/3) · Fable sleeps).
  tick cost ≈ $0.03. "voters never see each other's answers."
  Install trigger: "when dispatch.tsv shows Fable wake-ups that ended in action: stop."

07 · SPARRING — "builder and breaker, opposed daily"
  BUILD 7 · "builder and breaker, opposed daily"
  Breaker (daily): reads yesterday's merged diffs, writes ONE failing test; solid code -> writes nothing.
  Builder (daily): if any @sparring test fails, fixes the CODE until it passes; never touches the test.
  Artifacts: failing test -> tests/sparring/ · @sparring.
  Escalation: You — disputes stop here; disagree twice -> you.
  Wall: "opposed by design · neither touches the other's output · the maker's confidence is not evidence."

08 · COMPOST — "weekly · failures become laws"
  BUILD 7 · "weekly · failures become laws"
  Inputs (the week's exhaust): FAILED: lines in STATE.md · fails in trust.tsv · FAILs in goal-ledger.tsv · PRs closed unmerged.
  Compost (weekly · /loop 7d) reads the exhaust and proposes: new CLAUDE.md law (quote incidents) · skill fix (same failure repeating) · missing standing goal. "propose only." Clean week -> say so, write nothing.
  Targets: CLAUDE.md · skills/ · goals/. "your signature — nothing lands unsigned."
  Wall: "three proposals max · the system proposes its own next improvement, you sign it."

09 · RATCHET — "one number, monotonic or self-reverting"
  BUILD 7 · "one number, monotonic or self-reverting"
  measure (lint 2>&1 | grep -c warning) -> one change (smallest fix · commit per fix) -> re-measure (after every change) -> down or flat -> next fix; up or a test breaks -> REVERT -> weld (the floor becomes a standing goal: goals/lint-zero.md · predicate = the measure) -> target reached.
  Wall: "the number NEVER goes up · never game the metric or edit lint config · stop after 3 attempts with no movement."

================================================================
CROSS-CUTTING DESIGN PRINCIPLES (recurring across the nine loops)
================================================================
- Cheap-by-default, expensive-on-demand: a cheap model runs every tick; Fable 5 is woken only for planning/judgment/verification, and always read-only or spec+diff-scoped so it cannot rubber-stamp its own work.
- Nothing grades its own homework: separate parties for execute / verify / gate, with the last gate deterministic (verify.sh, a shell predicate), not an LLM.
- Trust is earned per skill, not granted per loop: watch -> queue -> auto, promoted by measured pass-rate over N runs, auto-demoted on any failure.
- Everything is a ledger row: trust.tsv, goal-ledger.tsv, dispatch.tsv, STATE.md — tiers/status recomputed on every write.
- Metrics are monotonic and un-gameable: ratchet reverts on regression and forbids editing the lint config; standing goals re-verify continuously so a once-passing check can't silently rot.
- Human stays the signer: the system proposes (compost, three max) and escalates disputes; nothing lands unsigned.
- These map onto Anthropic's two published patterns: loop 02 = Orchestrator–Workers, loop 03 = Executor–Advisor (Oracle).

================================================================
SOURCES
================================================================
Primary (could not fetch verbatim this session): the two X posts (@Av1dlive, @ClaudeDevs) and the r/ClaudeAI thread. The SVG builder's guide WAS read verbatim from the local file.
Third-party press grounding the Anthropic benchmark numbers:
- the-decoder.com/anthropics-fix-for-fable-5s-high-cost-is-turning-it-into-a-manager-that-delegates-to-sonnet-5/
- developersdigest.tech/blog/fable-5-orchestrator-model-playbook
- datasciencedojo.com/blog/claude-code-fable-5-orchestrator-workflow/
- explainx.ai/blog/fable-5-advisor-orchestrator-patterns-july-2026
- best.xiaohu.ai/en/article/fable5-advisor-executor/
- anthropic.com/news/claude-fable-5-mythos-5 (model announcement + pricing)
- forbes.com/sites/ronschmelzer/2026/06/10/anthropic-fable-5-ai-model-cost/ (pricing/cost framing)
Confirm the 92%/63% (SWE-bench Pro) and 96%/46% (BrowseComp) figures against the primary Anthropic post before quoting them as first-party.
