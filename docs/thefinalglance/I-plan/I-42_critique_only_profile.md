# I-42: critique-only operating profile — operator writes, agents critique only, no spawned executor touches disk

- Track 19 — DESIGN-PHASE CRITIQUE + TRUST-CLAIM DISCIPLINE | lever HUMAN-AS-VERIFIER | grounding MODERATE | nature BUILD | effort M | WAVE 3 | depends I-41 | binds the §6 ban-LLM-judge rule
- Premise: ELAI's default is autonomous orchestrate (spawned executor/fixer agents write code). The source
  design (ELAI-CRITIQUE-LENSES.md "CRITIQUE-ONLY MODE" + ELAI-IMPROVEMENT-PLAN.md §11.2) adds a SECOND selectable
  profile alongside orchestrate, mirroring the "human writes / AI critiques only" stance the R5 human-verifier
  research calls durable: the OPERATOR writes the code, spawned agents are dispatched ONLY for critique (I-41
  lenses at design time; the SHIPPED auditor + off-agent spine at ship time), and NO executor/fixer agent ever
  touches disk. Both profiles share ONE engine and ONE off-agent spine — the only difference is WHO writes and
  whether executor agents are spawned. The trust guarantee (off-agent deterministic evidence) is identical in both.

## Steal/build
A `RunProfile` enum { Orchestrate (default), CritiqueOnly } selected at dispatch time. In CritiqueOnly:
- the executor/fixer agent spawn path is GATED OFF (a structural refusal, not a prompt instruction) — the
  generator emits no disk-mutating Directive; the operator's own edits are the only writes
- the I-41 LensSet runs at DESIGN time (operator revises the spec from the surfaced findings)
- the SHIPPED auditor + the off-agent gate spine (F-1/F-4/G-2/L6/L8) run at SHIP time on the operator's diff
- the loop is: Design (lenses -> operator revises) -> Operator implements -> Ship (auditor + spine) -> iterate
  to convergence; the operator is the judge.
Rationale carried verbatim from the source: agent-spawned writes "caused more harm than good" in the
human-verifier engagement — CritiqueOnly is the conservative counterpart for high-stakes / regulated / low-
trust-in-autonomy work; orchestrate stays the default for autonomous throughput. INVARIANT: in CritiqueOnly the
set of file_write SSB rows attributable to a spawned executor/fixer agent is EMPTY (the off-agent observable).
Declined: a prompt-only "please don't write files" instruction (bypassable — must be a structural dispatch gate,
mirroring the I-1 supervisor posture); a separate engine fork (the whole point is one engine, one spine).

## Existing ground truth
- orchestrate skill + the executor/fixer agent dispatch path (task_runner spawn): SHIPPED — the path this
  session gates off in CritiqueOnly (PLAN-asserted; executor re-greps per W-7).
- I-41 LensSet: the design-phase critique engine CritiqueOnly runs (this session's hard dependency).
- the SHIPPED auditor + off-agent gate spine (F-1/F-4 stdout_hash, G-2 RRA, B-41 per-level auditor):
  the ship-time critique CritiqueOnly keeps unchanged (GD-1; PLAN-asserted, re-grep W-7).
- B-22 GateMode / AutonomousGate (autonomous vs interactive): SHIPPED precedent for a dispatch-time mode flag —
  RunProfile is orthogonal (autonomy of the GATE vs whether executors are spawned at all); compose the flag
  plumbing pattern, do NOT overload GateMode.
- I-1 directive supervisor (Track 12): the structural seam where "no disk-mutating Directive from the generator"
  is enforceable — CritiqueOnly is the supervisor configured to reject all executor-authored side-effect
  Directives (compose if I-1 has shipped; else gate at the spawn site directly — decide at edit per W-7).
- No RunProfile concept exists today (IN — net-new).

## REVISED scope (W-10 tiered)
SHIP NOW (Tier-1):
- crates/orchestrator/src/run_profile.rs: RunProfile enum + from_cli_args + the executor/fixer spawn gate
- `elai run --profile critique-only` (and orchestrate default) CLI flag
- the structural spawn-gate: in CritiqueOnly, executor/fixer dispatch is refused at the spawn site (or routed
  through the I-1 supervisor as a rejected side-effect Directive)
- the CritiqueOnly loop wiring: I-41 lenses at design, auditor + spine at ship (reuse shipped dispatch order)
- contract test: in CritiqueOnly, ZERO executor/fixer-attributed file_write SSB rows (the invariant)

DEFER (Tier-2 / gated):
- a dashboard profile switcher (a Settings toggle) — Tier-1 ships the CLI flag; the toggle is a small follow-on
- per-profile telemetry rollup (review-time vs generation-time deltas) — needs the G-5 cockpit (compose later)
- reserved RunProfile variants (e.g. a future review-only or pair profile) — not in this session's scope

## Integration
- CREATE: crates/orchestrator/src/run_profile.rs (~90 LOC: RunProfile + from_cli_args + spawn gate)
- MODIFY: crates/cli/src/args.rs + the run/orchestrate dispatch — thread --profile (PLAN-asserted seam; re-grep W-7)
- MODIFY: the executor/fixer spawn site in task_runner.rs — consult RunProfile before spawn (PLAN-asserted; re-grep W-7)
- CREATE: crates/contracts/tests/i42_critique_only_no_executor_writes_contract.rs (~60 LOC)
- CREATE: workflow/reasoning/I-42-decisions.md
- frontend_pair: the active RunProfile shown in the StatusBar (reuses the existing mode-badge pattern from
  B-22 patch-4G.1; PLAN-asserted; re-grep W-7) — W-6 satisfied by the existing badge cluster, no new component.

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/run_profile.rs::RunProfile (Orchestrate | CritiqueOnly)
  - crates/orchestrator/src/run_profile.rs::from_cli_args (the selector)
  - the executor/fixer spawn gate (refuses spawn in CritiqueOnly)
  - NO new SSB action_type — the profile is a run fact surfaced via existing phase_transition rows; the
    invariant is proven from the ABSENCE of executor-attributed file_write rows (GD-1; I-0 delta-table reuse note)
  - crates/contracts/tests/i42_critique_only_no_executor_writes_contract.rs
consumes_symbols:
  - I-41 LensSet (why: the design-phase critique engine CritiqueOnly runs)
  - the SHIPPED auditor + off-agent spine (F-1/F-4/G-2/B-41) (why: the ship-time critique, unchanged — GD-1)
  - the executor/fixer spawn path in task_runner (why: the path gated off in CritiqueOnly)
  - I-1 directive supervisor (why: the structural seam to reject executor side-effect Directives; compose if shipped)
  - B-22 GateMode flag plumbing (why: the dispatch-time mode-flag pattern; compose, do NOT overload GateMode)
reachable_from:
  - `elai run --profile critique-only` CLI -> dispatch -> spawn gate refuses executor/fixer  [PRODUCTION run surface]
  - the orchestrate dispatch path -> RunProfile check before any executor spawn  [PRODUCTION orchestrate surface]
  - a #[cfg(test)] / contracts/ caller NEVER satisfies this (R7)
stub_ban:
  - NO prompt-only "do not write files" instruction (the gate is structural at the spawn site — bypassable prompts banned)
  - NO second engine fork (one engine, one spine — the profile only changes WHO writes + whether executors spawn)
  - NO lens output on a verdict path (inherits the I-41 §6 rule)
  - NO overloading GateMode with the profile (RunProfile is orthogonal to gate autonomy)
  - NO .unwrap()/.expect() on production paths; NO blake2/sha2 (blake3 only)
frontend_pair:
  the active RunProfile rendered in the StatusBar mode cluster (reuses the B-22 patch-4G.1 badge pattern; no new
  component). Justified: the profile is a run fact; the badge is the operator-visible surface (W-6 satisfied).
observable_effect:
  trigger: "elai run --profile critique-only <fixture-task> (the operator-edit path simulated by a fixture diff)"
  expect_ssb: "I-41 CritiqueLensFinding rows at design time + auditor/spine verdict rows at ship time; ZERO
    file_write rows attributable to a spawned executor/fixer agent (the CritiqueOnly invariant)"
  expect_artifact: "the contract test asserts the executor/fixer spawn was refused (the spawn-gate fired) and
    the file_write SSB set from executor/fixer agents is EMPTY; the SAME fixture under --profile orchestrate
    (default) DOES produce executor file_write rows (proving the gate is the differentiator, not a no-op) — R2;
    the off-agent spine verdicts are byte-identical across both profiles (one spine, GD-2)"

## Grounding
- Source design: ELAI-CRITIQUE-LENSES.md "CRITIQUE-ONLY MODE" + ELAI-IMPROVEMENT-PLAN.md §11.2 / §1.1 (the
  human-as-verifier R5 conclusion; the "agent-spawned writes caused more harm than good" rationale).
- Composition substrate (PLAN-asserted, re-grep W-7): orchestrate dispatch, the auditor + off-agent spine,
  B-22 GateMode plumbing, I-1 supervisor seam.
- Doctrine: binds the §6 ban-LLM-judge rule (the spine, not the lenses, decides DONE in BOTH profiles) and the
  GD-1 compose-not-restate posture (one engine, one spine).
- SSB: NO new action_type — reuse phase_transition + prove the invariant from absent executor file_write rows
  (I-0 delta table; baseline 221/532 as of 2026-06-10; re-read at edit W-9).

## COMPOSES-WITH
- I-41 critique lenses — the design-phase critique engine; CritiqueOnly is its operating-mode consumer.
- the SHIPPED auditor + off-agent spine — the ship-time critique, identical across profiles. Do NOT fork.
- I-1 directive supervisor — the structural seam to reject executor side-effect Directives in CritiqueOnly.
- B-22 GateMode — the dispatch-flag pattern; RunProfile is orthogonal (do NOT overload GateMode).
- I-40 trust-layer wrapper — the external-agent wedge; CritiqueOnly is the internal-operator-writes sibling
  (both keep the off-agent spine; restates none).

End of I-42.
