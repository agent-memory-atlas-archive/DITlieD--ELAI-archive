B-THROUGH-I PLAN COMPLETION + WIRING AUDIT - MASTER SYNTHESIS
session 2026-07-02. six sonnet auditors, direct grep/Read of live .elai_cc source with file:line
anchors (elai.map used as locator only, it was 8h stale). H and I audited for the FIRST time;
B..G audited as a delta against the verified 2026-06-12 baseline in
workflow/sessions/s-plan-audit-2026-06-04/FINISH/_MASTER-INDEX.md.

detail reports (this dir): H-1-29.md, H-30-54.md, I-1-27.md, I-28-54.md, BCD-delta.md, EFG-delta.md
handoff for the finish session: workflow/handoff/plan-finish-b-to-i.md

================================================================================
PER-LETTER TALLY, 2026-07-02
================================================================================
  letter  plans  WIRED  PARTIAL  DARK  NOT_STARTED  BENCH/GPU  OTHER
  B          45     30       11     3            0          2      0   (unchanged since 06-12; B-28 DARK is likely a baseline error, re-scope to PARTIAL)
  C          59     26       21     3            0          1      8   (C-47 DARK>WIRED grounded flip; 8 = USER-DEFERRED external harnesses, confirmed no drift)
  D          30     16        3    11            0          0      0   (zero commits touched D since 06-12; NOT re-verified this pass; runtime legs still UNVERIFIED-no-journal)
  E          55      4       38     6            0          2      0   (tally stands pending full re-sweep; R9 punch list 16 > 6 open)
  F          14      8        3     3            0          0      0   (zero commits touched F since 06-12)
  G          10      6        3     0            0          1      0   (G-6 SAFEDELTA still PENDING-GPU)
  H          54     43        9     1            1          0      0   (first audit; DARK=H-5 registered/allowlisted; NOT_STARTED=H-16, correctly blocked on H-12)
  I          54     41        4     0            9*         0      0   (first audit; * includes registered/intentional gates, see below)

  note on H/I bookkeeping: the two I reports carry small internal tally inconsistencies
  (per-plan itemized lists vs summary counts differ by 1-4). the ITEMIZED per-plan lists in the
  report files are authoritative; the numbers above follow them. I NOT_STARTED itemized:
  I-6, I-16, I-25 (pure gaps), I-28/I-29/I-30 (WAVE-5, deps unmet), I-31 (correctly self-gated
  by a shipped 3-condition unlock checker), plus I-26 counted PARTIAL/registered-deferred by
  one report and NOT_STARTED by its return note. I PARTIAL itemized: I-23, I-26, I-33, I-36.

headline: of the ~296 numbered plans B..I audited, roughly 58% are WIRED with production
reachability proven at symbol level, ~31% PARTIAL, and the rest split across registered-dark,
bench/GPU-gated, user-deferred, and 4-7 genuine unbuilt gaps concentrated in I Track 15/16
and H-4/H-16. this is NOT "everything done": the punch list below is what a finish session
must close before that claim holds.

================================================================================
PUNCH LIST (ordered; anchors in the per-letter reports)
================================================================================
P0 GENUINE UNREGISTERED GAPS / DEFECTS
 1. I-33 governed_prompt_decision has ZERO callers anywhere (its own docs call it "the
    production entry point"); no WIRE-DARK marker, no allowlist row. wire it into a real
    decision seam or register it. (I-28-54.md finding 1)
 2. I-23 live facade-bypass: production_agent_rpc.rs:86 imports conformal_calibration
    directly while line 3143 uses the facade; the plan's no-bypass contract test only scans
    task_runner.rs so it cannot catch this. fix import + widen the contract test. (I-1-27.md)
 3. C-15 entropy gate REGRESSED: check_entropy_quarantine.sh exits 1 today, 1 leak at
    task_runner.rs:15303 introduced by 07-02 B-5 work, despite the 06-28 closure commit
    claiming 0 leaks. one-line entropy-allow or ClockProvider thread. (BCD-delta.md)
 4. glm-orchestrator dirt-baseline bug (orchestrator.py): failed attempts silently leak
    their fixes into the next attempt's baseline; caused ~10h loss across B-2..B-7, swept
    by eacd440 but the BUG IS UNFIXED. fix before any next multi-attempt run. (BCD-delta.md)
 5. H Track-3 capstone gates built + CLI-reachable but NOT intercepting the loops they
    guard: H-27 (rule SHADOW>ACTIVE), H-28 (darwin_admit_loop before insert_variant),
    H-29 (live-trading resume edge, which does not exist yet). autonomous loops run
    today with none of the three firing automatically. (H-1-29.md finding 1)
 6. H-10 PagerankPanel.svelte is an orphan (imported nowhere) while its IPC command and
    materialiser are live. one-line mount fix. (H-1-29.md finding 4)
 7. B-4 stale doc-comment at commands.rs:2427 claims failed_files "never plumbed" but the
    chain is closed end-to-end (indexing_service.rs:754, task_runner.rs:5987). one-line
    comment fix. (BCD-delta.md finding 5)

UNBUILT, DECIDE BUILD-OR-DEFER
 8. H-4 Tool Search Tool catalog deferral (the ~77K>8.7K token claim): zero implementation;
    only the OAuth beta header + dkg_* 3-arm fix landed. H-9 cost cockpit will never show
    this saving as things stand. (H-1-29.md finding 3)
 9. I-6 anchor pane slot: zero code, plan's own effort=S, cheapest gap to close.
10. I-16 kv-cache pruning: even the no-GPU fixture harness the plan says "ships now" is
    absent; not hardware-blocked, just unbuilt.
11. I-25 ledger-trained router: CostProfiler / offline trainer / escalation_policy hook all
    absent.
12. I-28/I-29/I-30: WAVE-5 self-improvement track, unbuilt, deps (I-4 PUCT v2 etc) unmet.
    I-31 stays closed by design until its unlock checker passes.
13. H-16 retire-orbit: blocked on H-12 canvasAsDefault still false; three.js/troika still
    in package.json. flip the flag + soak before deleting orbit.
14. E-53 formal verifiers: all 5 toolchain impls (Z3/Kani/Verus/Dafny/Lean) still stubs,
    zero commits touched formal_verify_cmd.rs since 06-12.
15. H-26: the 5 financial metamorphic relations the plan demanded are absent from
    metamorphic_testing.rs; confirm whether the invariants moved into shadow_exec.rs or
    were dropped. (H-1-29.md finding 5)

OPEN R9 FLAG ROWS (6 remain of the baseline 16; df18539 closed 6, 4 were baseline errors)
16. E-32 ELAI_FITM_ENABLED + ELAI_DEEP_MODE (RunPod-pending), E-4/E-8/E-11 (blocked on
    WAVE-B), E-25 two flags (missing smoke files). (EFG-delta.md)

VERIFICATION DEBT (not code gaps, audit gaps)
17. D-plan: 11 DARK from 06-12 never re-verified; zero commits touched D so unchanged, but
    the runtime SSB-row legs remain UNVERIFIED-no-journal for the whole D set.
18. E DARK-6 and F DARK-3 itemized plan lists were lost between audit generations (FINISH
    files are stale pre-correction snapshots); a scoped re-sweep is needed to NAME those
    9 plans before anyone can fix them.
19. H-30..54 returned an unusually clean 25/25 WIRED; the auditor itself recommends an
    independently-scoped spot-check of 3-5 random plans. also ~6-10 of those verdicts are
    symbol-level WIRED only, deeper behavior not re-derived (list in H-30-54.md finding 2).
20. E-51 "5 unresolved auditor BLOCKs" sub-claim: no BLOCK-log artifact found, unverified.
21. the 06-12 baseline punch lists contain at least 3 confirmed pre-baseline false claims
    (E-42/E-48/E-50/E-52 rows existed; E-51 adapters had prod callers; G-10 panel existed).
    treat all 06-12 itemized claims as noisy, spot-verify before acting.

DOC/CITATION DRIFT (cheap, batch in one pass)
22. H-36 plan cites hook_dispatcher.rs:1146; real wiring is task_runner.rs:1183/:11346.
23. B-28 06-12 DARK verdict is a likely false negative (2 of 3 hooks unconditionally
    instantiated since e38cd1a, pre-baseline); re-scope to PARTIAL.

INTENTIONAL / NO ACTION (so the next session does not re-litigate)
 - registered-dark done right: H-5 batch API (no lane tolerates 24h window), I-34/I-36/
   I-37/I-47/I-50 sub-features, C-31/C-45 routing legs (promotion plans named), I-26
   budget controller (pending I-4 BranchInput depth field), I-38/I-43/I-49 fixture-vs-live
   splits per D-026/W-10.
 - none of I Track 15's search-quality stack (I-4/I-17/I-19/I-21/I-22/I-24) is live on a
   stock install: all behind default-OFF flags per the GD-4 measured-first-win posture.
   by design, but it means "wired" != "running" for that whole track.
 - C's 8 external-harness plans stay USER-DEFERRED, no drift.
 - G-6 SAFEDELTA stays PENDING-GPU (RunPod 2x3090).
