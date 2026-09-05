# H-29 LOCKED fail-closed drift state machine (T-13) — seam-design proposal (doc-only)

session: opuswalk-H-29-PLAN-1786044193-cf771d  date: 2026-08-07
actor: synthesizer-elai on posthog-fable-5  role: PLAN  writes: this doc + the evidence notes only
base: worktree HEAD 6d85d72 (minimal checkout of /home/user/teikoku/elai; every seam verified with
`git grep` / `git show HEAD:` against the object store, not stale disk). Full findings:
conductor/opus_walk/evidence/H-29/seam-design-notes.md (worktree-relative; the absolute conductor
evidence dir is sandbox-denied to this worker).
plan: docs/thefinalglance/H-plan/H-29_drift_state_machine.md (176 ln, read in full)

QUEUE REASON (cited verbatim from TaskSpec): "refine-before-execute + unpinned-seam + paper-grounding
gated; plan_gate_check FAIL. LOCKED four-axis drift interlock absent, resume-FSM seam unnamed
(I-33/L-1 class), drift-detector math ungrounded. Not a clean codex burn."
GATE_JSON_REF: conductor/opus_walk/evidence/H-29/S3-disposition.md (SET_ASIDE: plan_gate FAIL +
unpinned resume-FSM seam + NannyML/ADWIN/DDM/Page-Hinkley grounding required first).

## 0. The load-bearing finding: the H-29 build ALREADY SHIPPED, and the owner already froze it

The S3 disposition and the queue premise say the LOCKED interlock is ABSENT (NN-13, "the BUILD").
At HEAD 6d85d72 that is FALSE. `crates/orchestrator/src/drift_interlock.rs` exists (1060 lines,
header "H-29 (SCHEMA_VERSION 238): LOCKED fail-closed drift state machine (T-13)"), landed in commit
`e38cd1a` "recover the git reset --hard loss and land the full h-plan tree at 226/537" — the SAME
recovery commit the H-27 S3 recon missed (see conductor/opus_walk/evidence/H-27/seam-design-proposal.md
§0; this is the second unit in the escalation queue with the identical stale-recon failure mode).
The unit is production-wired, contract-tested, SSB-registered, decision-documented (session s-8404,
2026-06-06), grounding-swept, and OWNER-FROZEN in .claude/gate-manifest.txt. All three queue blockers
were discharged by the shipped build itself.

## 1. Queue blockers vs verified HEAD

1. "LOCKED four-axis interlock absent" — SHIPPED. Four axes as LOCKED named consts in
   drift_interlock.rs: PSI_WARN_BAND=0.10 :99 / PSI_HALT_BAND=0.25 :103 (AXIS 1, inputs AND outputs,
   `compute_psi` :258 total + fail-closed, zero-bin floored toward HALT via PSI_BIN_EPSILON :124);
   blake3 schema-fingerprint (AXIS 2, DriftEvidence :197-201); DEVIATION_WINDOW_DAYS=14 :107 +
   DEVIATION_HALT_BAND=0.50 :111 (AXIS 3, ClockProvider); VALIDATION_TTL_DAYS=90 :115 (AXIS 4, the
   H-18 DSR re-check hook). warning->drift->halt = `InterlockVerdict` :294 (Running/Warning/DriftHalted)
   out of `classify_drift` :347. Missing evidence on ANY axis => HALT (contract test
   missing_evidence_halts_fail_closed). The state is a persistent journal-DERIVED latch
   (`DriftInterlockLedger::is_halted` :493 = COUNT('halt') > COUNT('unlock'); restart-proof,
   latch_persists_across_reopen :213); agent unlock is structurally impossible (free fn
   `unlock_drift_interlock` :712 requiring STRICTLY fresher validation evidence; negative compile-test
   :1050). Composes G-10 model_drift + D-7 canary + H-18 deflation via composed_breaches :212-215
   (GD-1, read-not-re-detect). mutable=0 rules echo: seed/finance/drift_interlock.toml FIN-DRIFT-001/002,
   seeded by project_rule_seeder.rs:697 from the production caller runtime_setup.rs:463.
2. "resume-FSM seam unnamed (I-33/L-1)" — RESOLVED BY DESIGN, not by discovery. The shipped
   workflow/reasoning/H-29-decisions.md carries a GROUNDED CORRECTION: three independent agents
   (grounding sweep, integration explorer, context-package) confirmed NO live-trading resume FSM edge
   exists anywhere in the codebase — fsm.rs is the agent-workflow FSM (FsmState :27:
   Idle/Planning/.../Researching), nothing trading-shaped (my re-grep at HEAD concurs: zero hits).
   The seam the plan told the executor to grep for was NEVER there; H-29 followed the H-18/H-19
   sibling pattern: Tier-1 W-3 production entrypoint is the always-compiled CLI `elai drift-check`
   (args.rs:1830, dispatch main.rs:1377, handler drift_check_cmd.rs driving evaluate_drift_interlock
   :618 over a real Ssb; --demo-drift exits 1 on DriftHalted). The live-trading resume edge is a
   DEFERRED Tier-2 forward dependency behind the plan's own W-10 owner checkbox + GD-4 false-halt
   calibration on >=3 real deployments.
3. "drift-detector math ungrounded" — DISCHARGED for what shipped. workflow/reasoning/H-29-grounding.md
   (s-8404): PSI is closed-form (no paper dependency); DDM ingested
   (workflow/papers/concept-drift-2004.05785-interface.md); ADWIN/Page-Hinkley NOT ingested AND NOT
   load-bearing because the label-fed half was deliberately deferred; NannyML informational only.
   No ungrounded detector math exists in the shipped code.
4. "plan_gate_check FAIL / refine-before-execute" — moot for the core: the build already executed with
   decisions + grounding docs. What actually FAILS the gate is doc rot: H-SUMMARY_INDEX.md §H-29 and the
   plan's "Existing state" still claim ABSENT/BUILD. That is a plan-maintenance correction, not a build.

## 2. Shipped vs residual

SHIPPED (evidence in §1 + notes §2): the four-axis LOCKED interlock, the persistent no-agent-unlock
latch, the `elai drift-check` W-1/W-3 entrypoint, the Pattern-2 DriftInterlock SSB lockstep
(payloads.rs:3603/:18746/:18767, action_type.rs:5699/:6572, schema.rs:5046 CHECK, golden line 182 of
612), the mutable=0 seed pair + runtime seeder caller, the 411-line h29 contract (12 tests covering plan
ACCEPTANCE a-f,h), gate_tax census row :213, and the owner freeze (gate-manifest.txt:181-182 — R8
already discharged out-of-band).

RESIDUAL (each named with its gate; none walk-buildable now):
- R-a live-trading resume FSM edge: greenfield I-33 (no live-trading FSM exists at all) behind the
  Tier-2 owner checkbox + GD-4 calibration evidence. OWNER-gated.
- R-b label-fed concept-drift half (ADWIN/Page-Hinkley): paper-grounding gated (the S3's own blocker
  class) + realized-label plumbing that does not exist. GROUNDING-gated.
- R-c conformal T3 advisory routing for the pre-label P(Y|X) residual: doctrine-routed (TD-5) but
  code-absent; wiring it means editing FROZEN drift_interlock.rs. OWNER/R8-gated, LOW weight (advisory).
- R-d deployment-cockpit frontend_pair (W-6): zero dashboard hits. OWNER-UI-hold-gated.
- R-e plan/SUMMARY/S3 doc rot says ABSENT at HEAD-that-says-SHIPPED: plan-maintenance pass (plan docs
  are outside this task's write set).

## 3. Why not the other verdicts

Not BUILD-WITH-SEAM: H-27 had one real un-shipped leg at an UN-frozen production seam; H-29 has none —
every residual sits behind an owner checkbox, an R8-frozen file, a UI hold, or a grounding gate, so any
thin slice would violate exactly the constraints this queue exists to protect. Not DEFER: DEFER asserts
the queued BUILD still waits on its gates; the BUILD is on HEAD, wired, tested, and frozen — deferring
"H-29" would keep a phantom unit in the queue and invite a third stale-recon dispatch. The residuals
should re-enter the queue as their OWN correctly-gated items (R-a Tier-2/owner; R-b grounding-first),
not as "H-29".

VERDICT: SUPERSEDED-BY — the shipped H-29 unit itself (session s-8404, landed in commit e38cd1a
"recover the git reset --hard loss and land the full h-plan tree at 226/537"): crates/orchestrator/src/
drift_interlock.rs (evaluate_drift_interlock :618, owner-frozen gate-manifest.txt:181) + seed/finance/
drift_interlock.toml (frozen :182) + crates/cli/src/drift_check_cmd.rs (`elai drift-check`, args.rs:1830)
+ crates/contracts/tests/h29_drift_interlock_contract.rs + workflow/reasoning/H-29-decisions.md +
H-29-grounding.md.
