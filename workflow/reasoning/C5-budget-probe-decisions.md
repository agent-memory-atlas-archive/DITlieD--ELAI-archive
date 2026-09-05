C.5 PRE_COMPACT live budget probe — decision doc (session s-14276, box)

WHAT
Close the C.5 non-RUNPOD residue from HANDOFF-s-22601 Section 2 item 2 + TODO[N-FOLLOWUP] at
task_runner.rs:136 and :8873. Replace the dead sentinel passed to the PreCompact ladder gate
with a LIVE per-session token occupancy, behind the existing wire-dark gate.

GROUNDED STATE (verified this session)
- with_pre_compact_token_state(u32, u32) builder + the 0.85 occupancy gate already ship
  (hook_dispatcher.rs:373 + :1028). Only the call-site passes None.
- task_runner.rs:8896 PreCompact branch currently passes None; sentinels PRE_COMPACT_PROBE_BUDGET=30_000
  (:141) + PRE_COMPACT_LADDER_TARGET_BUDGET=20_000 (:147) are policy literals.
- CumulativeTokenTracker (context_budget.rs:774; fields prompt_tokens_total + last_response_prompt,
  observe() at :805/:817) is owned privately by CloudGroundingClient.
- In cli, CloudGroundingClient is constructed ONLY in side commands (measure.rs, structured_output_cmd.rs),
  NOT co-located with the main TaskRunnerConfig in run_subcommand.rs. drive_task (task_runner.rs:5064)
  takes only writer/seq/task/resolved/cancel; ResolvedConfig (:2835) + TaskRunnerConfig (:2442) hold no tracker.
- current_session_budget() does not exist (only TODO comments). context_budget module exists (adaptive_reduce).

KEY DECISION — Choice A (accessor + Option-gated plumbing + sentinel fallback)
- Author context_budget::current_session_budget(tracker: &CumulativeTokenTracker) -> u64 returning
  last_response_prompt (IN-WINDOW occupancy, NOT cumulative prompt_tokens_total — cumulative would trip
  the 0.85 gate every time). Unit test pins this semantic.
- Add Option<Arc<Mutex<CumulativeTokenTracker>>> field on TaskRunnerConfig (default None; builder
  with_token_tracker; struct uses Default/struct-update so adding the field is zero E0063 where possible —
  executor must cargo check and fix any explicit-literal construction sites).
- Thread it to drive_task's PreCompact branch (via ResolvedConfig if drive_task only sees ResolvedConfig).
- At task_runner.rs:8896: when Some(tracker) -> used = current_session_budget(&tracker.lock()),
  ceiling = PRE_COMPACT_PROBE_BUDGET; pass .with_pre_compact_token_state(used.try_into().unwrap_or(u32::MAX),
  ceiling.try_into().unwrap_or(u32::MAX)). When None -> keep current sentinel behavior EXACTLY (no regression).
- In the cli runtime builder: populate the field from the main runtime client's tracker IFF one is
  reachable/co-located (upgrade to genuinely live); else leave None and document the residual.
- Keep both consts as fallback. Update the two TODO[N-FOLLOWUP] comments to reflect the accessor now exists.
- .try_into() not as-cast (anti-stale). No .unwrap() in prod except saturating try_into fallback is acceptable.

REJECTED — Choice B (new session-wide counter in the main conversation LLM-call loop): higher value but
touches the hot LLM-call path in cli live_cli/conversation, overlaps the risky SP3 extraction, and is a
larger design change. Deferred as the precise residual: "populate TaskRunnerConfig.token_tracker from the
main runtime client once the conversation loop exposes a shared session token counter."

EXPECTED TEST OUTCOME
- cargo check -p elai-orchestrator -p elai-cli --tests GREEN (fix any E0063 from the new field).
- New unit test in context_budget.rs (or contracts) asserting current_session_budget returns last_response_prompt.
- n_compaction_ladder_wire_contract.rs stays GREEN (asserts the literal consts; both kept as fallback).
- ZERO schema bump (rides shipped CompactionLadderTierApplied, SCHEMA 195).
- When token_tracker is None, PreCompact behavior is byte-identical to today (regression guard).

UNKNOWNS / RISK
- TaskRunnerConfig construction-site count (E0063 blast radius) — executor enumerates + fixes.
- Whether the main run path has ANY reachable tracker to populate (executor checks; default None if not).
- Mutex vs RwLock: use std::sync::Mutex only if the tracker is touched sync-only at the read site; the
  PreCompact branch is async — prefer tokio::sync::Mutex if the lock is held across an await, else std Mutex
  with a short non-await critical section (read value, drop guard before any await).
