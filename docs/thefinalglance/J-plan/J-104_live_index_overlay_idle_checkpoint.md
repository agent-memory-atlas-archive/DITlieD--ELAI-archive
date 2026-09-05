# J-104: live-index overlay-before-base read + idle checkpoint — an in-session edit is visible to retrieval IMMEDIATELY, the overlay wins on conflict, the idle drain is atomic, an EXTENSION of B-31

STATUS: PLANNED (the G-6 gap: B-31 is incremental but BATCH, so an agent that just edited a file can retrieve a STALE symbol until the next pass; J-104 adds an overlay-first read + an order-triggered atomic drain; small, EXTENDS B-31, default-OFF first; overlay-wins-on-conflict + atomic-drain are the two load-bearing invariants)

- Track J | lever NEVER-RETRIEVE-A-SYMBOL-YOU-JUST-DELETED (in-session edit visibility without a full re-index) | grounding STRONG (B-31 IndexingService indexing_service.rs / DKG dkg/mod.rs / file_ops.rs:320/354/382 read_to_string edit signal / blake3 CASCADE code-grounded this session; re-grep at edit W-7) / EXTERNAL (Cursor Merkle 10-min, YoloFS snapshot, design-rationale only) | nature BUILD (EXTENSION of B-31) | effort S-M | WAVE 1
- Source: workflow/reasoning/jplan-sdl-context-memory-readaccess-2026-06-26.md §2 (J-104) + the gap file G-6 + Downloads/deep-research-report(12).md §4 (Cursor Merkle diff + YoloFS snapshot + the honest accuracy-vs-staleness caveat). Doctrine: J-0_INDEX.md + the reasoning file §0 universal block.
- NO sdl-mcp code copied (overlay-reader.ts/coordinator.ts/idle-monitor.ts are design rationale only; license). This EXTENDS the SHIPPED B-31, never re-architects it (house rule: never amend B-31 in place — J-104 is the overlay layer ON it).

## Context
Why this matters (the gap, from G-6): sdl-mcp queries an in-memory OVERLAY of recent edits BEFORE the base graph so an edit is visible to retrieval immediately without a full re-index, and an IdleMonitor drains the overlay to durable state during idle. ELAI's B-31 is incremental (blake3 CASCADE) but BATCH — it does not do overlay-first-read for instant in-session edit visibility, so an agent that JUST edited/renamed/deleted a symbol can retrieve the STALE pre-edit symbol until the next incremental pass runs. In an autonomous loop that edit-then-retrieve gap is exactly where a wrong-symbol retrieval poisons the next step.

THE TWO LOAD-BEARING INVARIANTS (the research's honest caveat, report(12) §4 — "no public accuracy-vs-staleness benchmark exists; rapid edits could confuse retrieval if the overlay is not authoritative"):
- OVERLAY-WINS-ON-CONFLICT: when a symbol exists in BOTH the overlay (just edited) and the base (stale), the overlay value is authoritative. A deleted/renamed symbol present in the base is SHADOWED by the overlay's tombstone — the agent never retrieves a function the just-completed edit removed.
- ATOMIC DRAIN: the idle-triggered drain into the durable DKG/blake3 store is atomic — a crash mid-drain leaves the durable store either OLD-consistent or NEW-consistent, never partial (reuse the B-31 lock lifecycle + the existing CASCADE update path; the drain is just an EARLIER trigger of work B-31 already does).

THE IDLE TRIGGER IS ORDER-BASED, NOT WALL-CLOCK (TZ-4 / RD-4 — order not duration): "idle" is detected from the supervisor/watchdog plane's step / no-material-progress signal (I-17 / J-100), NOT a wall-clock timer. This keeps the drain replay-stable (a seeded replay drains at the same logical point) and consistent with the time-determinism doctrine — a wall-clock idle timer would make the durable store non-deterministic across replays.

## Existing state (what J-104 extends — do NOT re-author, GD-1)
- RESUMABLE INDEXING + LOCK LIFECYCLE [WIRED] B-31, crates/orchestrator/src/indexing_service.rs — cross-process lock, resume-on-crash, blake3 CASCADE incremental update; kill-switch ELAI_DISABLE_INDEXING (still governs the BASE; J-104's overlay flag is separate). The drain REUSES this lock + CASCADE path.
- DKG [WIRED] crates/knowledge/src/dkg/mod.rs — the durable symbol store the overlay shadows + drains into. Retrieval reads it; the overlay sits in front.
- file_ops EDIT SIGNAL [WIRED] crates/tools/src/file_ops.rs read_to_string at :320/:354/:382 (+ the pre-image already captured at :272/:300 then discarded, per H-49) — the edit ALREADY flows through file_ops; the overlay reuses that post-edit symbol-delta signal (no new edit hook).
- CATRAG/RETRIEVAL READ ENTRYPOINT [WIRED] phase-1E pipeline.rs (crates/knowledge/src/pipeline.rs) — the read path that must consult overlay-first.
- AUTONOMOUS SUPERVISION / WATCHDOG [PLANNED(J-100)/WIRED(I-17)] the step / no-material-progress signal (J-100 T-5 watchdog promotion, I-17 Wave-3) — the ORDER-BASED idle trigger (TZ-4, not wall-clock).
- live SSB baseline ~268/577 (re-grep at edit, W-9); J-104 prefers-reuse B-31's indexing rows; +OverlayDrained ONLY if a journaled drain checkpoint is wanted (net 0..+1).

## Reads (mandatory before coding)
1. workflow/reasoning/jplan-sdl-context-memory-readaccess-2026-06-26.md §2 IN FULL (the overlay-first read, the atomic drain, the order-based idle trigger).
2. Downloads/deep-research-report(12).md §4 (Cursor Merkle 10-min / hot 8-10ms cold 500-600ms; YoloFS 100k files ~3.5s snapshot; the honest no-benchmark caveat) — DESIGN RATIONALE, NEVER a done-gate number.
3. crates/orchestrator/src/indexing_service.rs (B-31 lock lifecycle + CASCADE update path — the drain reuses both) — re-grep live at edit (W-7).
4. crates/knowledge/src/dkg/mod.rs (the durable store the overlay shadows) + crates/knowledge/src/pipeline.rs (the retrieval read entrypoint that must read overlay-first).
5. crates/tools/src/file_ops.rs (the post-edit signal the overlay captures — :272/:300 pre-image, :320/:354/:382 read_to_string) + the J-100 / I-17 step/no-progress idle signal.

## Tasks (WAVE 1 — EXTENSION of B-31; overlay-wins + atomic-drain are the load-bearing asserts)
1. [OVERLAY STORE] an in-memory overlay layer keyed by file/symbol that captures post-edit symbol deltas (added/changed/deleted/renamed) from the SHIPPED file_ops edit signal — reuse the signal, add NO new edit hook. A deletion/rename records a TOMBSTONE so the base symbol is shadowed. PRODUCER: crates/orchestrator/src/index_overlay.rs (new) or an overlay module under indexing_service.rs.
2. [OVERLAY-FIRST READ] the DKG/retrieval read entrypoint (pipeline.rs) consults the overlay BEFORE the base; on a conflict the overlay wins (the just-edited value; a tombstoned symbol is NOT returned). CONSUMER: the retrieval read path.
3. [IDLE ATOMIC DRAIN] an idle-triggered drain (order-based, from the J-100/I-17 step/no-progress signal, NOT wall-clock — TZ-4) flushes the overlay into the durable DKG via the EXISTING B-31 CASCADE + lock lifecycle, ATOMICALLY (crash mid-drain => old-consistent or new-consistent, never partial). After the drain the overlay is empty and the durable store is byte-identical to a from-scratch index of the same edits (GD-2).
4. [optional +OverlayDrained] if a journaled drain checkpoint is wanted (for replay/audit of WHEN the overlay drained), add ONE Pattern-2 SSB action_type OverlayDrained — full 9-tuple golden-lock lockstep, TOTAL_VARIANTS from LIVE at edit (W-9). Otherwise net 0 (ride B-31's indexing rows).
5. workflow/reasoning/J-104-decisions.md (the SSB decision; the overlay eviction-bound DECISION; the atomic-drain crash-recovery finding).

## WIRING_CONTRACT
produces_symbols:
  - the in-memory index overlay store (file/symbol deltas + tombstones, fed by the shipped file_ops edit signal) [index_overlay.rs]
  - the overlay-first read path on the DKG/retrieval entrypoint (overlay wins on conflict; tombstone shadows base) [pipeline.rs consumer]
  - the order-triggered (J-100/I-17 step/no-progress) atomic drain into the durable DKG via the B-31 CASCADE + lock lifecycle
  - OPTIONAL +OverlayDrained (Pattern-2) if a journaled drain checkpoint is chosen
consumes_symbols:
  - B-31 IndexingService lock lifecycle + blake3 CASCADE update path (indexing_service.rs) — the drain reuses both, cited not re-authored
  - the DKG durable store (dkg/mod.rs) + the retrieval read entrypoint (pipeline.rs) — overlay sits in front
  - the file_ops post-edit signal (file_ops.rs :272/:300/:320/:354/:382) — the overlay capture source, NO new edit hook
  - the J-100 T-5 / I-17 step-no-progress idle signal (TZ-4, order-based) — the drain trigger
  - ELAI_DISABLE_INDEXING (the base kill-switch, still governs the base; the overlay flag is separate)
reachable_from:
  - a live `elai run` turn edits a file -> IMMEDIATELY retrieves -> the new symbol is visible AND the deleted symbol is gone, BEFORE any batch re-index [PRODUCTION, the overlay-first read]
  - a live idle point (step/no-progress) -> the overlay drains atomically into the durable DKG; the durable store is byte-consistent [PRODUCTION, the drain]
  - a #[cfg(test)]/contracts/ caller NEVER satisfies the read or drain leg (R7)
stub_ban:
  - NO base-wins-on-conflict (the overlay MUST be authoritative; a tombstoned symbol is NEVER returned — the honest-caveat fix)
  - NO non-atomic drain (a crash mid-drain leaving a partial durable store is a FAIL; reuse the B-31 lock lifecycle)
  - NO wall-clock idle timer (the trigger is the order-based J-100/I-17 step/no-progress signal; a wall-clock drain breaks replay determinism — TZ-4)
  - NO new edit hook (reuse the shipped file_ops signal)
  - NO re-architecting B-31 (J-104 is the overlay LAYER on B-31, never an in-place edit of it)
  - NO default-ON before the edit->retrieve correctness + drain atomicity are proven on >=3 real runs (GD-4); ELAI_LIVE_OVERLAY_ENABLE needs a wire-dark-allowlist row + promotion_plan_id J-104-promotion (W-2/R9)
frontend_pair:
  none required — this is an indexing-layer change with no first-class UI signal. A KnowledgeDrawer "overlay N pending" badge is OPTIONAL; decide at edit (W-4) or W-6-justify N/A.
observable_effect:
  trigger: "edit a file (add a symbol + delete a symbol), then IMMEDIATELY retrieve, BEFORE any batch re-index; then drive an idle point and assert the atomic drain"
  expect_ssb: "B-31 indexing rows on the drain (net 0 new action_type); OR if chosen, OverlayDrained checkpoint rows recomputable from the journal (GD-2). bump TOTAL_VARIANTS from LIVE at edit (W-9)."
  expect_artifact: "the just-added symbol is retrievable + the just-deleted symbol is NOT (zero 'retrieved a symbol the edit removed' cases — the ELAI-measured done-gate); after the idle drain the durable DKG is byte-consistent with a from-scratch index of the same edits (GD-2). NEGATIVE: a crash injected mid-drain leaves the durable store old-consistent or new-consistent, never partial."

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/J-plan/J-104_live_index_overlay_idle_checkpoint.md --run-trigger
python .claude/tools/production_caller_check.py --files .elai_cc/crates/orchestrator/src/index_overlay.rs --json   # the overlay store has a non-test production caller (the read entrypoint + the drain)
rg -n "overlay|tombstone|overlay_first" .elai_cc/crates/knowledge/src/pipeline.rs | rg -v "test|contract"   # retrieval reads overlay-first in production
rg -n "drain|CASCADE|lock" .elai_cc/crates/orchestrator/src/indexing_service.rs   # the atomic drain reuses the B-31 lock + CASCADE
rg -n "step|no_progress|no_material_progress|idle" .elai_cc/crates/orchestrator/src/index_overlay.rs   # the drain trigger is order-based, not wall-clock
test -f workflow/reasoning/J-104-decisions.md
```

## ACCEPTANCE
- [ ] an edit-then-retrieve fixture: the just-added symbol is retrievable AND the just-deleted symbol is NOT, BEFORE any batch re-index runs (zero "retrieved a symbol the edit removed" cases — the ELAI-measured done-gate).
- [ ] the overlay wins on conflict; a tombstoned symbol is never returned (the honest-caveat fix).
- [ ] the idle drain is ATOMIC: a crash injected mid-drain leaves the durable store old-consistent or new-consistent, never partial (reuses the B-31 lock lifecycle).
- [ ] after the drain the durable DKG is byte-identical to a from-scratch index of the same edits (GD-2, recomputable).
- [ ] the idle trigger is the order-based J-100/I-17 step/no-progress signal, NOT a wall-clock timer (TZ-4, replay-stable).
- [ ] ELAI_LIVE_OVERLAY_ENABLE default-OFF with a wire-dark-allowlist row (promotion_plan_id=J-104-promotion); observe-only until edit->retrieve correctness + drain atomicity bounded on >=3 real runs (GD-4). NEVER cite Cursor's 10-min number.
- [ ] cargo check --workspace GREEN; auditor PASS on -p elai-orchestrator + -p elai-knowledge (+contracts); action_type_lock GREEN if +OverlayDrained. No new dependency.

## WIRING DONE-GATE (clause by clause)
- W-1: done = a live edit-then-immediate-retrieve returns the post-edit truth AND a live idle point drains atomically, observed end-to-end — not "the overlay struct exists". W-2: ELAI_LIVE_OVERLAY_ENABLE gets a wire-dark-allowlist row + promotion_plan_id J-104-promotion. W-3: EXTENSION of B-31 (reuses its lock + CASCADE), not a port.
- W-5: trigger = a live `elai run` edit->retrieve + an idle drain. W-7: indexing_service.rs + dkg/mod.rs + pipeline.rs + file_ops.rs + the J-100/I-17 idle signal re-grepped at edit. W-9: ~268/577 re-grep; bump only if +OverlayDrained.
- TZ-4 (K-14): the drain trigger is order-based, replay-stable, not wall-clock. GD-2: the post-drain durable store is recomputable. GD-4: observe-only first. ID-5: no Cursor/YoloFS number as a gate. R7/R9.

## Tests to add
- crates/contracts/tests/j104_index_overlay_contract.rs (~120 LOC): (a) an edit-then-retrieve fixture (add symbol A, delete symbol B) proves A is retrievable + B is NOT, BEFORE any batch re-index (the load-bearing assert); (b) the overlay-wins-on-conflict arm: a symbol present in both overlay (edited) and base (stale) returns the overlay value, a tombstone shadows the base; (c) the atomic-drain arm: a crash injected mid-drain leaves the durable store old-consistent or new-consistent, never partial (reuses the B-31 lock); (d) the post-drain durable DKG is byte-identical to a from-scratch index of the same edits (GD-2); (e) the drain trigger is the order-based step/no-progress signal, not wall-clock (TZ-4).

## Files touched
- CREATE: crates/orchestrator/src/index_overlay.rs (the in-memory overlay store + tombstones + the order-triggered atomic drain; exact seams re-grepped W-7)
- MODIFY: crates/knowledge/src/pipeline.rs (the retrieval read entrypoint consults the overlay first; overlay wins on conflict)
- MODIFY: crates/orchestrator/src/indexing_service.rs (expose the CASCADE + lock path the drain reuses; the overlay registers its drain hook)
- MODIFY (only if +OverlayDrained): crates/orchestrator/src/ssb.rs + ssb/payloads.rs + action_type.rs + tests/action_type_lock.rs + .golden (full 9-tuple lockstep, TOTAL_VARIANTS from LIVE)
- CREATE: workflow/reasoning/J-104-decisions.md
- CONFIG: .elai_cc/wire-dark-allowlist.toml row for ELAI_LIVE_OVERLAY_ENABLE (type=experiment, owner, expiry, promotion_plan_id=J-104-promotion, smoke)

## Citations / Grounding
- workflow/reasoning/jplan-sdl-context-memory-readaccess-2026-06-26.md §2 (the full J-104 spec) + elai-gaps-sdl-ai-setup-2026-06-26.md G-6.
- Downloads/deep-research-report(12).md §4 (Cursor Merkle-tree diff ~10-min sync, hot 8-10ms / cold 500-600ms; YoloFS 100k files ~3.5s snapshot; the HONEST caveat = no public accuracy-vs-staleness benchmark, rapid edits confuse retrieval unless the overlay is authoritative) — [EXTERNAL], design-rationale only, NEVER an ELAI done-gate number (ID-5/GD-1).
- ELAI ground truth (code-grounded this session; re-verify at edit W-7): B-31 IndexingService indexing_service.rs [WIRED]; DKG dkg/mod.rs [WIRED]; pipeline.rs retrieval [WIRED]; file_ops.rs edit signal :272/:300/:320/:354/:382 [WIRED]; J-100 T-5 / I-17 idle signal [PLANNED/WIRED]. live ~268/577 (W-9).
- Hard rules invoked: TZ-4 (K-14, order-based drain trigger, replay-stable); GD-2 (post-drain store recomputable); GD-4 (observe-only); ID-5 (no Cursor number as a gate); W-1/W-2/W-7/W-9; R7/R9. NO sdl-mcp code copied (license).

## Open questions / risks
- ATOMIC DRAIN (highest): a crash mid-drain must never leave a partial durable store. Mitigation = reuse the B-31 lock lifecycle; the crash-mid-drain test is the load-bearing arm.
- OVERLAY EVICTION BOUND: an unbounded overlay grows in a long autonomous run between idle points. DECISION = a size/age soft-cap that forces an early drain (still atomic) before the cap; recorded in J-104-decisions.md. // DECISION.
- the SSB decision (journal +OverlayDrained vs net-0): net 0 is the default; journal only if drain-timing audit is wanted; resolved in J-104-decisions.md. // DECISION.
- rapid-edit confusion: the honest caveat — repeated edits to the same symbol must coalesce newest-wins in the overlay (the J-25 newest-wins writer-coalescing pattern); recorded.

## Next
Closes the in-session edit-visibility gap: the agent never retrieves a symbol it just deleted, with no full re-index, and the durable store stays byte-consistent via an order-triggered atomic drain. EXTENDS B-31 (reuses its lock + CASCADE), composes the J-100/I-17 idle signal. Pairs with J-103 (what the rungs read is now never stale) and underpins J-105 (memory ROI attribution reads a fresh graph).

End of J-104.
