# J-99: time-as-context plane — per-message timestamps + agent-time + cross-day resume awareness, the PLANE-1 technical half of the time-awareness design, BOUND by K-14's TZ-1..TZ-4

STATUS: PLANNED (WAVE 1; the time-awareness J track the K-14 OUT-OF-K block recorded NOT authored; BUILD = genuine net-new T-1/T-2 + a T-3 H-51 header extension; net 0 new SSB — the stamp is model-INPUT context recorded into the existing action-stream, not a journaled event; the T-1 cache-placement smoke is the gating risk and ships first)

- Track J | lever TIME-AS-CONTEXT (the model can REASON about rhythm + elapsed work) | grounding STRONG (clock.rs/conversation.rs/context_budget.rs code-grounded 2026-06-25 in the design doc; re-grep at edit W-7) / EXTERNAL (the temporally-blind failure, design-rationale only) | nature BUILD | effort M (T-1 + T-2 low-risk; T-3 a header extension) | WAVE 1
- This is PLANE 1 of the two-plane time design (the user's idea conflated TIME-AS-CONTEXT with TIME-AS-CONTROL; keeping them separate IS the design — TIME-AS-CONTROL is J-100). PLANE 1 places real wall-clock FACTS in the prompt so the model can judge rhythm + elapsed work; it informs judgment, it never enforces a guarantee (that is J-100). Every leg is BOUND by K-14's four-clause time-determinism invariant TZ-1..TZ-4 (CITED, not re-authored — K-14 is the pre-gate that makes this file correct from the start, so R-1 cannot recur at audit).
- Source: workflow/reasoning/time-awareness-design-2026-06-25.md §2 (PLANE 1), §2.1 (placement rule), §2.2 (block contents), §2.3 (cross-day), §2.4 (agent-time), T-1/T-2/T-3, IMP-1/IMP-3/IMP-4/IMP-10, R-1/R-2. K-14 is the K-native DISCIPLINE+PACKAGE half. Doctrine: J-0_INDEX.md + .claude/rules/time-determinism.md (K-14's TZ rule).

## Context
Why this matters: the temporally-blind failure mode is real and MEASURED (arXiv:2510.23853 / 2601.13206 — a once-set system-prompt timestamp never updates, so a 6-hour or next-day continuation still reads as the original second). The user's instinct (per-message timestamp + "last message X ago" + a session day that updates across days) is the established, measured fix — but on its own it is NOT sufficient: it covers HUMAN time (gaps between user messages) and ignores AGENT time (how long the model's own tools have run within a turn), and it only fires when the NEXT user message arrives. J-99 ships the full PLANE-1 set: human-time (T-1), agent-time (T-2, the part that actually fixes the documented tool-timing failure), and cross-day-at-resume (T-3) so a 10-day gap is known BEFORE the model acts, not after.

THE SINGLE HIGHEST RISK (the design's R-1, why K-14 exists): a volatile timestamp placed naively near the TOP of the prompt (system/rules area) and updated every turn BUSTS ELAI's byte-stable cache prefix every turn (cloud cost) AND breaks RRA byte-identity (the replay re-computes a different clock). The fix is mechanical but must be designed in from the start, not discovered at audit — that is exactly TZ-1/TZ-2 (K-14). J-99 obeys them:
- TZ-1 PLACEMENT: the time block is VOLATILE content and lives in a dedicated 3E slot STRICTLY AFTER the last cache breakpoint (the 10-slot allocator already structures this; OutputReserve is last, the time slot sits just before it, after breakpoint 2). A changing timestamp then costs ZERO cache invalidation.
- TZ-2 SOURCE+JOURNAL: the human-facing stamp is sourced from the REAL OsClock (clock.rs:85), NOT SeededClock, AND recorded INTO the SSB action-stream as part of "what the model saw" — so an RRA replays the EXACT bytes the model was given (the stamp becomes recorded data, not a re-computed value). Under ELAI_SEED the SeededClock continues to drive gate/journal timestamps for byte-identity.
- TZ-3 NEVER-A-GATE-INPUT: no rule predicate, FSM transition, or promotion gate may branch on the time block (ND-1, J-66). It is judgment context only.
HONEST CAVEAT BUILT IN (IMP-10, R-2 temporal-hallucination): a "time passed" stamp can make the model over-anchor and fabricate updates. Pair the stamp with the SHIPPED staleness instruments (F-5 quarantine cascade / H-30 stale_fact_reached_prompt / H-32 validate-citation-before-use) so "time passed" routes to RE-VERIFY-the-data, never to "assume it changed". The same elapsed-since figure the model reads is what the staleness family already computes (G13/G14 of the design).

DECISIONS taken here (reversible, mark // DECISION; the K-14 D-2 open question lands in this pass):
- TIMESTAMP GRANULARITY: minute-resolution LOCAL time when the local tz is known, else second-resolution UTC stated explicitly ("2026-06-25 14:32 local" / "2026-06-25 14:32:07 UTC (tz unknown)"). Never a fabricated tz (HD-1/ID-5 honesty, design §2.2). // DECISION reversible.

## Existing state (what the legs build on / cite; do NOT re-author, GD-1)
- ClockProvider / OsClock / SeededClock [WIRED] core/src/clock.rs:43/85/124 — now()/system_now()/now_ms(); the TZ-2 source split. Re-exported core/lib.rs:123.
- CONTEXT BUDGET ALLOCATOR [WIRED, 10 slots] context_budget.rs, 2 cache breakpoints (xxhash prefix), OutputReserve last — the TZ-1 placement target (the time slot after breakpoint 2). Already structures volatile-after-breakpoint (H-6/HD-2 masking lands trailing observations after the prefix).
- conversation.rs [WIRED] assembles ApiRequest{system_prompt, messages:Vec<ConversationMessage>}; NO per-message timestamp injected today (design §G2, grep clean) — T-1/T-2 are the genuine net-new.
- CROSS-SESSION SLEEP-TIME HANDOFF (H-51) [PARTIAL] handoff_digest.rs auto-authors a journal topic-digest at resume — the T-3 header-extension target.
- STALENESS family [WIRED/PARTIAL] F-5 quarantine cascade, H-30 stale_fact_reached_prompt, H-32 validate-citation-before-use — the IMP-10 re-verify route (already compute the elapsed-since figure; T-1/T-3 only surface it to the MODEL).
- ContextSlot enum (context_budget.rs) — a new volatile TimeContext slot lands after breakpoint 2, before OutputReserve (per-turn re-fill is cache-safe by TZ-1).
- live SSB baseline ~268/577 (re-grep at edit, W-9); J-99 adds NO new action_type.

## Reads (mandatory before coding)
1. workflow/reasoning/time-awareness-design-2026-06-25.md IN FULL (§2 PLANE 1, §2.1-2.4, T-1/T-2/T-3, IMP-2/3/4/10, R-1/R-2). The grounding ledger G1/G2/G12/G13/G14.
2. .claude/rules/time-determinism.md (K-14's TZ-1..TZ-4 — the binding pre-gate; this file OBEYS it). + conditional-routing loads it on a timestamp task.
3. core/src/clock.rs (OsClock vs SeededClock, TZ-2) + crates/orchestrator/src/context_budget.rs (the cache-breakpoint structure + ContextSlot enum, TZ-1) — re-grep both live at authoring (W-7).
4. crates/orchestrator/src/conversation.rs (the message-assembly path; where ConversationMessage is built and where the volatile slot fills) + handoff_digest.rs (the H-51 resume header, T-3).
5. The staleness instruments F-5/H-30/H-32 (the IMP-10 re-verify route — the stamp feeds them, never a "assume it changed" path).

## Tasks (WAVE 1 — context plane; T-1 first because its cache-placement smoke is the gating risk)
1. [T-1] ConversationMessage.created_at (ClockProvider-sourced, OsClock) + a dedicated VOLATILE TimeContext slot in the 3E allocator placed STRICTLY AFTER the last cache breakpoint; populate each turn with: absolute now (granularity per the DECISION above) + a DAY-ROLLOVER marker when now's date != the previous turn's ("[day changed: 3 days since your last reply]") + the delta since the user's previous message ("last user message: 4m ago" / "11 days ago"), computed now - previous-user-message wall-clock (both system_now()). Record the rendered block into the SSB action-stream as model-input bytes (TZ-2). CONSUMER: the conversation.rs build path. EXCLUDE folklore (design §2.2): no predicted-next-message, no fabricated tz.
2. [T-2] AGENT-TIME: on each model re-entry WITHIN a multi-step turn (tool loop / autonomous run), inject elapsed-since-turn-start + elapsed-since-last-tool into the SAME volatile slot ("turn elapsed: 3m12s; last tool (cargo build) started 2m ago, still running"). This is the fact the temporally-blind paper shows agents lack; it is the context-side mirror of J-100's control. CONSUMER: the turn loop. Reads the same OsClock; TZ-1 placement holds.
3. [T-3] SESSION-RESUME elapsed header: extend H-51's handoff_digest header with "elapsed since last session: N days" computed from the last journaled row's ClockProvider timestamp vs system_now(). This lands the "10 days passed" awareness at RESUME (earlier + more reliable than waiting for the next user message). CONSUMER: the H-51 resume path. Reuses the elapsed-since figure the staleness family already computes (IMP-4).
4. [IMP-10 guard] wire the surfaced elapsed-time so a large delta ROUTES to the staleness re-verify instruments (F-5/H-30/H-32), never to a model-assumed update — a contract test asserts a backdated fixture surfaces the delta AND triggers the re-verify path, not a fabricated "the data changed".
5. workflow/reasoning/J-99-decisions.md (the granularity DECISION + the T-1 cache-placement smoke result + the T-3 header finding).

## WIRING_CONTRACT
produces_symbols:
  - ConversationMessage.created_at (ClockProvider/OsClock-sourced) + the volatile TimeContext 3E slot (after the last cache breakpoint) [T-1]
  - the per-turn time block (absolute now + day-rollover marker + since-last-user delta), recorded into the SSB action-stream as model-input bytes (TZ-2) [T-1]
  - the agent-time block (elapsed-since-turn / elapsed-since-last-tool) on multi-step re-entry [T-2]
  - the H-51 resume "N days since last session" header extension [T-3]
consumes_symbols:
  - ClockProvider::system_now / OsClock (core/src/clock.rs:85; the REAL clock, NOT SeededClock; TZ-2) — cited, not modified
  - the 3E context_budget allocator's cache-breakpoint structure + ContextSlot enum (TZ-1) — extended with one volatile slot, the prefix is unchanged
  - conversation.rs message-assembly (T-1/T-2 consumer) + handoff_digest.rs H-51 resume (T-3 consumer)
  - the staleness instruments F-5/H-30/H-32 (the IMP-10 re-verify route — the elapsed figure feeds them)
  - .claude/rules/time-determinism.md TZ-1..TZ-4 (K-14, the binding pre-gate — OBEYED, not re-authored)
reachable_from:
  - a live `elai run` turn assembles the prompt -> the volatile time slot renders AFTER the last cache breakpoint with the real OsClock stamp [PRODUCTION]
  - a live multi-tool turn re-enters the model -> the agent-time block advances monotonically [PRODUCTION, T-2]
  - a live session RESUME after a backdated last-journaled-row -> the H-51 digest header carries "N days since last session" [PRODUCTION, T-3]
  - a #[cfg(test)] / contracts/ caller NEVER satisfies any leg (R7)
stub_ban:
  - NO time block in the SystemPrompt / Rules / PhaseSpec slot, and NO time block BEFORE the last cache breakpoint (TZ-1; the R-1 cache/replay regression — the smoke MUST prove byte-stable prefix)
  - NO SeededClock for the human-facing stamp (TZ-2: OsClock-sourced + journaled-as-input; SeededClock keeps gate/journal byte-identity)
  - NO wall-clock value read by ANY gate / FSM transition / promotion (TZ-3 / ND-1, J-66)
  - NO fabricated timezone, NO predicted-next-message time, NO relative phrasing the model can't verify (design §2.2; HD-1/ID-5)
  - NO "time passed -> assume the data changed" path; a large delta routes to the F-5/H-30/H-32 re-verify instruments (IMP-10, R-2)
frontend_pair:
  the rendered time block is visible on the conversation/turn surface (it is model-input context the operator can also read). A dedicated dashboard tile is NOT required (the stamp is in the transcript); decide at edit (W-4) or W-6-justify.
observable_effect:
  trigger: "a 2-turn fixture across a simulated day boundary (T-1); a 3-tool fixture within one turn (T-2); a resume whose last journaled row is backdated N days (T-3)"
  expect_ssb: "none NEW — the rendered time block is recorded into the existing SSB action-stream as model-input bytes (TZ-2, recomputable GD-2); T-3 reuses HandoffLifecycleEvent (227). net 0 new action_type."
  expect_artifact: "T-1: the time block appears AFTER the last cache breakpoint (the cache prefix is byte-identical across the two turns — the gating smoke), the day-rollover marker + since-last-user delta strings are correct. T-2: the elapsed strings advance monotonically and are OsClock-sourced. T-3: the resume header reads 'N days since last session'. NEGATIVE (R2): a fixture that places the block BEFORE the breakpoint FAILS the HD-2 byte-stability assert (proves the placement is load-bearing); a large-delta fixture triggers the staleness re-verify path, never a fabricated update."

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/J-plan/J-99_time_as_context_plane.md --run-trigger
python .claude/tools/production_caller_check.py --files .elai_cc/crates/orchestrator/src/conversation.rs,.elai_cc/crates/orchestrator/src/context_budget.rs --json   # the time-slot fill + created_at have a non-test production caller
rg -n "created_at|TimeContext|system_now" .elai_cc/crates/orchestrator/src/conversation.rs | rg -v "test|contract"   # >=1 production call-site
rg -n "TimeContext|after.*breakpoint|volatile" .elai_cc/crates/orchestrator/src/context_budget.rs   # the volatile slot sits after the last cache breakpoint
rg -n "days since last session|elapsed since last session" .elai_cc/crates/orchestrator/src/handoff_digest.rs   # T-3 header present
rg -ni "SeededClock" .elai_cc/crates/orchestrator/src/conversation.rs ; test $? -ne 0   # the human-facing stamp is NOT SeededClock-sourced (TZ-2)
test -f workflow/reasoning/J-99-decisions.md
```

## ACCEPTANCE
- [ ] T-1: ConversationMessage.created_at (OsClock) + the volatile TimeContext slot render the per-turn block AFTER the last cache breakpoint; the cache prefix is byte-identical across turns (the R-1 gating smoke passes; HD-2/TZ-1).
- [ ] the block carries absolute now + a day-rollover marker on a date change + the since-last-user delta; no fabricated tz / predicted time (design §2.2).
- [ ] T-2: agent-time (elapsed-since-turn / elapsed-since-last-tool) advances monotonically on multi-step re-entry, OsClock-sourced.
- [ ] T-3: the H-51 resume digest carries "N days since last session" from the last journaled row vs system_now().
- [ ] TZ-3/ND-1 holds: no wall-clock value is a gate / transition / promotion input; the block is journaled-as-input (TZ-2), RRA byte-identity preserved.
- [ ] IMP-10: a large elapsed delta routes to the F-5/H-30/H-32 re-verify path, never a model-assumed update; net 0 new SSB action_types.
- [ ] cargo check --workspace GREEN; auditor PASS on -p elai-orchestrator (+contracts). No new dependency.

## WIRING DONE-GATE (clause by clause)
- W-1: done = a live turn renders the time block after the breakpoint with a byte-stable prefix (the R-1 smoke), not "the field exists". W-2: N/A (no new feature flag; if a kill-switch is added it gets a wire-dark-allowlist row). W-3: N/A (net-new, not a port).
- W-5: trigger = a live `elai run` 2-turn day-boundary fixture + a multi-tool turn + a backdated resume. W-7: clock.rs + context_budget breakpoints + conversation.rs + handoff_digest.rs re-grepped at edit. W-9: ~268/577 re-grep.
- TZ-1..TZ-4 (K-14): OBEYED, the rule is the pre-gate; GD-2 the journaled stamp is recomputable (the recorded bytes ARE the replay). ND-1/J-66: no wall-clock gate input. HD-2: the cache prefix is byte-stable (the load-bearing smoke). R7: test callers excluded.

## Tests to add
- crates/contracts/tests/j99_time_context_contract.rs (~120 LOC): (a) a 2-turn day-boundary fixture proves the time block renders AFTER the last cache breakpoint AND the cache prefix hash is identical across both turns (HD-2/TZ-1 — the gating smoke), the day-rollover + delta strings correct; (b) the placement-NEGATIVE arm: a block placed before the breakpoint FAILS the byte-stability assert (R2, proves load-bearing); (c) a 3-tool fixture proves the agent-time strings advance monotonically + are OsClock-sourced (T-2); (d) a backdated-resume fixture proves the T-3 header; (e) the human-facing stamp is OsClock not SeededClock (TZ-2); (f) an IMP-10 large-delta fixture routes to the staleness re-verify instruments, never a fabricated update.

## Files touched
- MODIFY: crates/orchestrator/src/conversation.rs (ConversationMessage.created_at + the volatile time-slot fill per turn + agent-time on re-entry; exact sites re-grepped W-7)
- MODIFY: crates/orchestrator/src/context_budget.rs (one VOLATILE TimeContext ContextSlot after the last cache breakpoint; the cache prefix unchanged)
- MODIFY: crates/orchestrator/src/handoff_digest.rs (the H-51 resume "N days since last session" header; T-3)
- CREATE: workflow/reasoning/J-99-decisions.md (the granularity DECISION + the T-1 cache-placement smoke result + the T-3 finding)

## Citations / Grounding
- workflow/reasoning/time-awareness-design-2026-06-25.md §2/§2.1-2.4 (PLANE 1, placement, contents, cross-day, agent-time), T-1/T-2/T-3, IMP-2/3/4/10, R-1/R-2; the grounding ledger G1/G2/G12/G13/G14.
- ELAI ground truth (code-grounded 2026-06-25 in the design doc; re-verify at edit, W-7): ClockProvider/OsClock/SeededClock clock.rs:43/85/124 [WIRED]; CONTEXT BUDGET ALLOCATOR 10 slots + 2 cache breakpoints [WIRED]; conversation.rs (no per-message timestamp today); handoff_digest.rs H-51 [PARTIAL]; F-5/H-30/H-32 staleness [WIRED/PARTIAL]. live ~268/577 (W-9).
- arXiv:2510.23853 "Your LLM Agents are Temporally Blind" + arXiv:2601.13206 (the MEASURED failure the feature answers — [EXTERNAL], design-rationale only, NEVER claimed as an ELAI number; HD-1/ID-5).
- Hard rules invoked: TZ-1..TZ-4 (K-14 time-determinism, .claude/rules/time-determinism.md — the binding pre-gate); ND-1 (J-66 no wall-clock gate input); HD-2 (cache byte-stability — the R-1 gating smoke); GD-2 (the journaled stamp is the replay); HD-1/ID-5 (no fabricated/external number); R7 (test callers excluded); W-7/W-9.
- K-14 is the K-native DISCIPLINE+PACKAGE half (the TZ rule + the replayable-async-tools differentiator); J-99 is PLANE-1 of the technical track it recorded OUT-OF-K. J-100 is PLANE-2 (TIME-AS-CONTROL).

## Open questions / risks
- R-1 CACHE/DETERMINISM REGRESSION (highest): a volatile stamp before a cache breakpoint invalidates the prefix every turn (cloud cost) + breaks RRA byte-identity. Mitigation = the T-1 placement smoke; the whole file is gated on it (do T-1 first).
- R-2 TEMPORAL HALLUCINATION: the model over-anchors on "time passed" and fabricates updates. Mitigation = IMP-10, route elapsed-time to the staleness re-verify path (F-5/H-30/H-32), never to assume-it-changed.
- The granularity DECISION (minute-local-if-known-else-UTC) is reversible; if a tz-privacy concern surfaces, default to UTC-only. // DECISION.
- T-3 may find the H-51 digest already surfaces an elapsed figure (the staleness family computes it) -> the leg degrades to a header-render + a contract test, recorded honestly, not padded into code.

## Next
Ships the model's sense of HUMAN time (T-1), AGENT time (T-2, the part that fixes the documented tool-timing failure), and CROSS-DAY-at-resume (T-3), all cache/replay-safe by K-14's TZ rule. Pairs with J-100 (TIME-AS-CONTROL: the decoupled async-tool loop + the can't-get-stuck watchdog) — together they are the full time-awareness design; J-99 informs the model's judgment, J-100 enforces the guarantee. K-14 banks the doctrine + the differentiator.

End of J-99.
