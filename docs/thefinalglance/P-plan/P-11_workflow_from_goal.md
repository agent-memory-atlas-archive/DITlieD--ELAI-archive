# P-11: elai workflow from-goal — intent to a closed-enum-constrained TaskSpec graph on the canvas, human-confirmed before it runs (GAP 3, the n8n-user-converting front door)

STATUS: PLANNED (product surface; executes AFTER the proof spine per A-7 / GATE M-0 / PSR-1 — authored now, not pulled forward). Authored 2026-07-08 from workflow/reasoning/elai-command-center-eval-2026-07-08.md §3-GAP3/§4.3/§7.

PROOF_SPINE: none (distribution) — spine-exempt product-surface (O/P class). User PSR-2 ack recorded (2026-07-09). The user-facing composition demo; rides P-9's connector vocabulary + P-10's ingress; does not feed a spine number.

Objective: GAP 3 — L-14 `elai build` synthesizes apps and H-13 lets the model drive canvas ops, but NOTHING does goal -> proposed TaskSpec graph (closed-enum-constrained, human confirms) on the O-4 canvas. P-11 is the cheap composition of H-13 + O-2: `elai workflow from-goal <intent>` has the model PROPOSE a TaskSpec graph drawn ONLY from the closed stage/connector vocabulary (O A3 kinds + P-9 mounted connectors), rendered on the O-4 canvas, and the human CONFIRMS before anything runs. It is the demo that converts an n8n user — same "describe it, see the graph" feel, but every proposed wire is provable (typed, gated, replayable) and nothing executes unconfirmed.

## Context
The market's pull is "describe a workflow, get a graph." ELAI's answer is NOT open-node freeform (that forfeits receipts, O A3 ban) — it is CONSTRAINED synthesis: the model may only propose graphs whose nodes are existing typed stage kinds + mounted connectors, whose wires satisfy the PortSchema/threaded-contract system, and whose gaps render VISIBLY DISCONNECTED on the O-4 canvas (O-4 wire-dark rendering) rather than being silently invented. The human is the final wiring authority (confirm/edit before run). The L-14 `elai build` front-door pattern is reused so the synthesis output carries an RRA receipt (what was proposed, from what intent, provable).

## Existing state (substrate P-11 composes, does NOT fork)
- H-13 model-drives-canvas-ops — the model->canvas mutation seam P-11 constrains to enum-only proposals.
- O-2 TaskSpec-as-data-rows + triggers — the graph P-11 proposes IS a set of TaskSpec rows + wires.
- O-4 canvas + VISIBLY-DISCONNECTED wire-dark rendering — where the proposed graph renders and where unavailable pieces show as disconnected (never silently synthesized).
- L-14 `elai build` one-shot front door + RRA receipt — the pattern so from-goal output is journaled/provable.
- P-9 mounted connectors + the closed stage vocabulary (O A3) — the ONLY vocabulary the model may draw from.
- PortSchema / SCHEMA_REQUIRED threaded contracts — the wire-validity constraint.

## Reads (mandatory before coding)
1. workflow/reasoning/elai-command-center-eval-2026-07-08.md §3-GAP3/§4.3/§7.
2. H-13 model-drives-canvas-ops (the constrained mutation seam) + O-4 canvas wire-dark rendering.
3. O-2 TaskSpec data-row + trigger contract (the graph target shape).
4. L-14 `elai build` front-door + its RRA receipt path (the journaling pattern to reuse).
5. The closed stage-kind enum (O A3) + P-9 mounted-connector registry (the allowed vocabulary) + PortSchema contract (wire validity).

## RECON_NEEDED
- RECON_NEEDED-1 (constrained-decode mechanism): how is the model's proposal CONSTRAINED to the closed vocabulary — grammar/enum-constrained decoding, or propose-then-validate-then-reject? SETTLING CHECK: read H-13's canvas-op interface + whether ELAI has a constrained-decode/grammar path; if propose-then-validate, the validator (reject any node not in the enum, any wire not PortSchema-valid) is the load-bearing gate and must be fail-closed.
- RECON_NEEDED-2 (unavailable-piece rendering): when the intent needs a stage kind/connector that is NOT mounted, does O-4 render it VISIBLY DISCONNECTED (the honest gap) rather than the model hallucinating a node? SETTLING CHECK: read O-4 wire-dark rendering; confirm a proposed-but-unavailable node maps to the disconnected visual + a "needs a P-9 connector pack" hint, never a fabricated node.
- RECON_NEEDED-3 (confirm-before-run gate): where in the O-2/FSM path does the human confirmation gate sit so NOTHING executes from a proposal until confirmed? SETTLING CHECK: read the O-2 Manual trigger + the FSM dispatch entry; the proposed graph is inert (rows exist, no trigger armed) until the human confirms.

## FORKS
- FORK-1 (decode constraint): TRIGGER = does a constrained-decode/grammar path exist (RECON-1)? Yes -> constrain at decode (the model literally cannot emit an out-of-vocab node). No -> propose-then-validate-then-reject with a fail-closed validator + a bounded re-propose loop. Prefer constrained decode; the validator is the mandatory backstop either way.
- FORK-2 (partial-graph on a gap): TRIGGER = the intent needs an unmounted connector. Render the graph with the gap VISIBLY DISCONNECTED + offer `elai connector new` (P-9 LANE A). Do NOT auto-author the connector inside from-goal (that is a separate gated mini-system, §7); from-goal proposes, the human decides to fill the gap.

## ABORT
- ABORT-1: if synthesis would emit a node OUTSIDE the closed vocabulary (a freeform/custom node to satisfy the intent) -> STOP. That is the open-node model O A3 bans; from-goal is constrained-only. An unavailable capability renders as a disconnected gap, never a fabricated node.
- ABORT-2: if a proposed graph could EXECUTE without explicit human confirmation -> STOP. The confirm-before-run gate is load-bearing; a proposal is inert until confirmed (ABORT-2 guards the "it ran something I did not approve" failure).
- ABORT-3: if the proposal is emitted WITHOUT an RRA receipt (intent -> graph, provable) -> STOP; the L-14 receipt is what makes the synthesis auditable.

## Tasks (MOVES — each EXPECTED_OBSERVATION + ON_FAIL)
MOVE 1 — the constrained proposer: `elai workflow from-goal <intent>` calls the model (via the H-13 canvas-op seam) to propose a TaskSpec graph, CONSTRAINED to the closed stage-kind enum + P-9 mounted connectors (FORK-1). Output is a set of proposed TaskSpec rows + wires, NOT executed.
  EXPECTED_OBSERVATION: from-goal on a fixture intent whose capability IS available emits a valid proposed graph (all nodes in-enum, all wires PortSchema-valid); the rows are inert (no trigger armed).
  ON_FAIL: an out-of-vocab node appears -> cause = the decode/validator let it through; counter-move = fail-closed reject at the validator (ABORT-1), re-propose bounded.
MOVE 2 — O-4 canvas render + the honest gap: the proposed graph renders on the O-4 canvas; an unavailable capability renders VISIBLY DISCONNECTED with a "needs a P-9 connector pack" hint (never a fabricated node).
  EXPECTED_OBSERVATION: a fixture intent needing an UNMOUNTED connector renders the graph with that node visibly disconnected + the connector-new hint; no hallucinated node is drawn.
  ON_FAIL: a needed-but-unavailable piece is drawn as connected -> cause = gap not mapped to wire-dark rendering; counter-move = unavailable -> disconnected (RECON-2 / ABORT-1).
MOVE 3 — the confirm-before-run gate: the proposed graph is inert (rows exist, no trigger armed) until the human confirms/edits on the canvas; confirmation arms the O-2 triggers.
  EXPECTED_OBSERVATION: a proposed graph does NOT run until an explicit confirm; before confirm, no stage dispatches (no execution SSB rows); after confirm, the O-2 triggers arm and it runs.
  ON_FAIL: a stage dispatches pre-confirm -> cause = proposal auto-armed; counter-move = proposal is data-only until the confirm gate (ABORT-2).
MOVE 4 — the L-14 RRA receipt: the from-goal run journals {intent, proposed graph, model, confirmation} as an RRA receipt (reuse the L-14 front-door pattern) so the synthesis is provable/replayable.
  EXPECTED_OBSERVATION: a from-goal run emits an RRA receipt binding the intent to the proposed graph; the receipt verifies (G-2).
  ON_FAIL: no receipt / unverifiable -> cause = front-door RRA path not reused; counter-move = route through the L-14 receipt pattern (ABORT-3).
MOVE 5 — decisions doc workflow/reasoning/P-11-decisions.md (the decode-constraint choice, the gap-rendering contract, the confirm-gate placement).
  EXPECTED_OBSERVATION: the doc exists and records the FORK-1 route + the confirm-gate seam.
  ON_FAIL: choices asserted not grounded -> counter-move = ground against H-13/O-4/O-2 (KD-1) before locking.

## PRODUCTION_HOOK (named now; confirmed at execution pre-flight)
- Producer `elai workflow from-goal` -> consumer the H-13 canvas-op seam + the O-4 canvas render (a proposed-graph render is the visible artifact) + the O-2 TaskSpec store (the inert rows). Producer the confirm gate -> consumer the O-2 trigger-arm path (confirmation is what makes it runnable). Producer the from-goal run -> consumer the L-14 RRA receipt sink. Exact file+fn seams resolved by the integration-explorer at execution pre-flight against live code.

## INTEGRATION_SMOKE (R10 — at execution)
Independent, real entrypoint, hard input: `elai workflow from-goal "<a multi-step intent mixing an available connector and an UNMOUNTED one>"` and assert (off-agent) — the available part renders as a valid connected graph, the unmounted part renders VISIBLY DISCONNECTED (no fabricated node), NOTHING dispatches before confirm (no execution rows), an RRA receipt binding intent->graph lands, and after an explicit confirm the O-2 triggers arm. Toy single-node happy-path does not satisfy it.

## WIRING_CONTRACT
produces: `elai workflow from-goal` (the constrained proposer) + the O-4 proposed-graph render + the confirm-before-run gate + the intent->graph RRA receipt.
consumes: H-13 canvas-op seam + O-2 TaskSpec/triggers + O-4 wire-dark rendering + L-14 front-door/RRA + P-9 mounted-connector vocabulary + O A3 closed enum + PortSchema contracts.
reachable_from: `elai workflow from-goal <intent>` [CLI] + the O-4 canvas confirm interaction [production UI] + the armed O-2 triggers after confirm [production run]. A test caller never satisfies R7.
stub_ban: NO out-of-vocab / fabricated node (ABORT-1); NO execution pre-confirm (ABORT-2); NO proposal without an RRA receipt (ABORT-3); NO auto-authoring a connector inside from-goal (FORK-2 — that is a separate gated mini-system).
observable_effect: trigger "elai workflow from-goal '<mixed available/unmounted intent>'"; expect a proposed graph on O-4 with the unavailable piece visibly disconnected, zero execution rows pre-confirm, an intent->graph RRA receipt, and triggers armed only post-confirm.

## DETERMINISTIC_CHECK (at execution)
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/P-plan/P-11_workflow_from_goal.md --run-trigger
# from-goal proposes only in-enum nodes; an unmounted capability renders disconnected; zero stage dispatches pre-confirm; an RRA receipt lands.
```

## VERIFY (at execution; off-agent pass artifacts)
```bash
# build + from-goal contract tests; then the real-entrypoint smoke (mixed-availability intent -> render -> pre-confirm-inert -> confirm -> arm)
# PASS = the proposed-graph render + the visibly-disconnected gap + zero pre-confirm execution rows + the intent->graph RRA receipt in the journal.
```
PASS artifacts are the O-4 render state + the absence of pre-confirm execution rows + the RRA receipt — never the executor's stdout (R4).

## ACCEPTANCE (binary; off-agent proof)
- [ ] every proposed node is in the closed vocabulary + every wire is PortSchema-valid (ABORT-1 held); an unavailable capability renders VISIBLY DISCONNECTED, never fabricated.
- [ ] NOTHING executes from a proposal until explicit human confirmation (ABORT-2 held; zero pre-confirm execution rows).
- [ ] the from-goal run emits a verifiable intent->graph RRA receipt (ABORT-3 held; L-14 pattern reused).
- [ ] OFF-AGENT: the proposed-graph render + the zero-pre-confirm-execution evidence + the RRA receipt from a real run; INTEGRATION_SMOKE green.

## REDTEAM (plan-standard point 7)
DEFERRED to the execution pre-flight (P executes post-spine, A-7/GATE M-0). The non-author adversarial-auditor attacks it when execution is first attempted; surviving attack + patches appended here. Aim then at: can the model smuggle an out-of-vocab capability past the validator (prompt-injection in the intent), can a proposal arm a trigger without confirmation, and does an unavailable-connector intent ever produce a fabricated-connected node instead of an honest gap.

## Open questions / risks
- Constrained-decode vs propose-then-validate (FORK-1) is a RECON-1 empirical on ELAI's actual decode path, not an assumption; the validator is the mandatory backstop regardless.
- Intent ambiguity (one goal -> several plausible graphs) is expected; from-goal proposes ONE + the human edits on the canvas — it is a proposal front door, not an autonomous builder.

## Next
P-11 is the front door that makes P-9 (connector vocabulary) + P-10 (ingress/auth) legible to a user as "describe it, confirm the graph, run it" — the n8n-converting demo, with receipts. All three (P-9/P-10/P-11) execute after the proof spine; together they are the command-center distribution layer.

End of P-11.
