# J-25 decisions — refine-before-execute (S0)

DATE: 2026-07-20
ACTOR: opuswalk-J-25-PLAN worker (glm-5-2, FREE lane)
SCOPE: plan-standard refine only; no product code touched.

## S2 verdict
EXECUTE_TIER1_T3 — proceed with the Tier-1 slice (T3 boundary guard, T4 pressure ladder, T6
completion gate, T2 budgeted read). Tier-2 (T1/T5/T7) stays DEFERRED with H-50/H-51 / E-5 / J-21
as already written.

## FORK-1 default
ROUTE A (tool-pair only). session::ContentBlock has NO Thinking variant at S1 recon time; the
thinking-sibling leg is deferred until ContentBlock gains Thinking via a separate owner-approved
change. The executor re-greps session::ContentBlock at edit time; if a Thinking variant has
landed, take ROUTE B (include same-message thinking-block siblings in the backward walk).

## API-only attribution; ZERO LOCAL GPU
This refine ran on the FREE glm-5-2 lane. No local GPU was probed; :8181 untouched. All
attribution is API-only.

## plan line 1551 stale; real trim seam 917-1046
The plan body cites "session_dag.rs:1551 region" for the SessionDag 3-pass trim seam. S1 recon
found the real trim seam at session_dag.rs:917-1046 (the 1551 line is stale). The executor
re-greps session_dag.rs at edit time (W-7) and uses the 917-1046 region as the T3 production
hook; if the seam has moved again, ABORT-4 fires (SET_ASIDE, surface as RECON_NEEDED).

## Gate failures fixed (plan_gate_check.py)
- P1 EXPECTED_OBSERVATION: injected per MOVE (T3/T4/T6/T2) + Tier-2 DEFERRED stubs.
- P3 FORK trigger: FORK-1 + FORK-2 with explicit TRIGGER lines.
- P5 ABORT conditions: ABORT-1..ABORT-4.
- P6 VERIFY: off-agent runs spelled out with off-agent pass artifacts.
- P7 REDTEAM: ATTACK-1..ATTACK-5.
- P10 INTEGRATION_SMOKE: per-unit smoke + cross-seam smoke cadence; Tier-2 n/a.
P2/P4/P9 already PASS; no changes to those sections.

## Open questions / risks
- The trim seam line range (917-1046) is S1-recon; the executor re-greps at edit (W-7). If the
  seam has moved, ABORT-4 -> RECON_NEEDED.
- T7 relative-floor scoring lands with E-5 retrieval un-dark or J-21, whichever executes first;
  not in this wave.
- SSB net 0..+1 (ContextMechanicsEvent) is decide-at-edit; prefer-reuse of
  HandoffLifecycleEvent (227) + existing trim/compaction rows.

End of J-25-decisions.md.
