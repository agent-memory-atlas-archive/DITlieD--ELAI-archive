# P-2: inter-tool contracts + the world-grain event bus — typed ports, provenance-carrying edges, wire-dark made visual at world grain

STATUS: PLANNED (WAVE 2; PROOF_SPINE: none (product-surface / distribution class), owner-acked 2026-07-07; BUILD-on-shipped — mostly composition; the port-schema registry is the genuine build)

- Track: PERSONAL-INCARNATION | lever: typed-ports-over-canvas | source finding: GAP-2 (answers T3 glue, and the user's "nodes/connected systems") | grounding: MODERATE (index-derived, PLAN-asserted) | nature: mostly composition | effort: M | WAVE 2 | depends P-1 (manifest) + O-4 (canvas)

- Premise: the thread's third demand is glue — "it requires a bit of glue" (Plane_Garbage); "agents will build modular tools and glue them together" (wtjones); "could in-house tools ever achieve some kind of standardization or modularity?" (nessthing). O-4's canvas WORKFLOW mode already runs edges over a CLOSED stage vocabulary, but at RESEARCH-PIPELINE grain. P-2 lifts the same discipline one level: every ELAI-built tool ships typed PORTS in its P-1 manifest, and the canvas wires tool->tool edges carrying an N-6 provenance tier + G-1 taint. The wardrobe app consuming the weather tool's output becomes an edge you can SEE, replay (G-2), and audit. The port/manifest schema is the interchange FORMAT of the personal-software world (the M-6 own-the-format instinct at world grain).

## Steal/build
Extend the P-1 `AppManifest` with `ports: { inputs: [PortSchema], outputs: [PortSchema] }` where `PortSchema = { name, json_schema, domain_label (life|work|shared, P-6) }`. A `PortRegistry` indexes every app's ports for wiring. On the O-4 canvas, a tool->tool EDGE binds an output port to a compatible input port; the edge carries the N-6 provenance tier of the source + the G-1 taint of the flowing bytes. An edge whose source port has NO consumer renders VISIBLY DISCONNECTED (O-4's wire-dark-made-visual, at world grain). elai-verify (M-5) conformance to the PortSchema makes a THIRD-PARTY tool a mountable node. The bus is not a runtime — dispatch stays through the FSM (J-31/O-4 doctrine); the "bus" is the typed-edge projection + the journaled cross-tool invocation SSB row.
EXCLUDED: a live message-broker/runtime bus (A-3 generic-engine trap — dispatch is the FSM, never a canvas/bus runtime); a second provenance scale (ride N-6's ONE lattice — A-1 binding); an untyped/free-text port (closed json_schema, `deny_unknown`); an edge that carries bytes WITHOUT the source's taint tier (G-1 must flow along the edge).

## Existing ground truth (PLAN-asserted; executor re-greps per W-7)
- O-4 canvas WORKFLOW mode: nodes = bound units, edges = verified claims, dispatch through the FSM, NO canvas runtime — P-2 reuses this exact shape one grain up.
- N-6 is the ONE provenance lattice + admission (N-2 forbids a second scale) — edges carry an N-6 tier.
- G-1 is taint tracking (untrusted bytes toward sinks); G-2 is the VCR/replay — a cross-tool edge is taint-tracked + replayable.
- M-6 is the evidence format as a published standard; M-5 is elai-verify — PortSchema conformance is the mount admission gate.
- P-1 owns the AppManifest that ports extend.

## REVISED scope (W-10 tiered)
- SHIP NOW (Tier-1): the PortSchema extension to AppManifest + the PortRegistry + a static port-compatibility check (`elai fleet wire <appA.out> <appB.in>`) that refuses incompatible schemas + the journaled `tool_port_edge` SSB row on a bound edge + G-1 taint tier carried on the edge record.
- DEFER (Tier-2): the O-4 canvas WORKFLOW-mode rendering of tool->tool edges (frontend_pair, lands after O-4 ships) + third-party mount via M-5 elai-verify conformance (crosses the L-12 admission seam — land with the sharing-surface wiring pass shared with P-3).

## Integration
- crates/orchestrator (or knowledge): the PortSchema type + PortRegistry + the compatibility checker + the `tool_port_edge` SSB emit + taint-tier propagation along the edge.
- CLI: `elai fleet wire|ports <app>` subcommand is the Tier-1 production caller.
- frontend_pair (Tier-2): crates/dashboard-v2 O-4 WORKFLOW view renders tool->tool edges with N-6 tier chrome + the disconnected-source affordance.

## MOVES (wargame)
M1 extend AppManifest with `ports` (closed PortSchema, `deny_unknown` json_schema) and build the PortRegistry that indexes inputs/outputs across the fleet.
EXPECTED_OBSERVATION: `elai fleet ports <app>` lists the app's typed inputs/outputs; a manifest with a malformed port schema is a hard parse-refusal.
ON_FAIL: an untyped port is accepted -> the interchange-format invariant breaks -> make PortSchema strict, add the negative parse test.
M2 add the static port-compatibility checker + bind an edge, emitting a `tool_port_edge` SSB row carrying `{src_app.out, dst_app.in, n6_tier, taint_tier}`.
EXPECTED_OBSERVATION: `elai fleet wire wardrobe.outfit weather.forecast` refuses (schema mismatch); `elai fleet wire wardrobe.in weather.forecast` binds and lands the SSB row with the weather tool's N-6 tier + G-1 taint carried.
ON_FAIL: an edge binds without the source's taint tier -> G-1 doesn't flow along the edge -> propagate the source port's taint tier onto the edge record, assert it in the row.
M3 render a source port with zero consumers as VISIBLY DISCONNECTED in the O-4 WORKFLOW view (Tier-2; Tier-1 exposes it via `elai fleet ports --unwired`).
EXPECTED_OBSERVATION: an output port no edge consumes shows up in `--unwired` (Tier-1) / renders disconnected on the canvas (Tier-2).
ON_FAIL: an unwired output is invisible -> world-grain wire-dark is hidden -> the PortRegistry flags zero-consumer outputs, surfaced in the CLI list and the canvas chrome.

## FORK
FORK: if O-4's canvas edge model is hard-bound to the research-pipeline stage enum and cannot carry a tool->tool edge without restructuring -> ship Tier-1 (CLI port registry + wire + SSB rows) fully and route ALL canvas rendering to Tier-2 behind O-4's completion; record the O-4-edge-generalization seam in the decision doc. The trust value (typed + taint-carrying + journaled edges) is fully realized at the CLI grain without the canvas.

## RECON_NEEDED
RECON_NEEDED: O-4's edge data model (is an edge a typed stage->stage binding or a generic node pair?) -> settle via reading O-4's shipped canvas WORKFLOW view + `grep -rn "edge\|WORKFLOW\|canvas.*mode" .elai_cc/crates/dashboard-v2/src .elai_cc/crates`.
RECON_NEEDED: the G-1 taint API for tagging a byte-stream with a tier + reading it back -> settle via `grep -rn "taint\|G-1\|tier\|sink" .elai_cc/crates` + codegraph_explore on the taint wrap/read API.
RECON_NEEDED: whether N-6's provenance tier is a value ELAI code can attach to an arbitrary record or is bound to claim objects only -> settle via reading N-6's lattice + N-4 claim object.

## ABORT
ABORT: if wiring the world-bus would require a LIVE message-broker/runtime executing edges outside the FSM (A-3 generic-engine trap; O-4 doctrine says dispatch is the FSM, no canvas runtime) -> stop and flag; the bus is a typed-edge PROJECTION + journaled invocation rows, never a runtime.

## WIRING_CONTRACT
produces_symbols:
  - the PortSchema type + the AppManifest `ports` extension + the PortRegistry
  - the `elai fleet wire|ports` CLI subcommand + the static port-compatibility checker
  - the journaled `tool_port_edge` SSB row + taint-tier propagation along the edge
consumes_symbols:
  - P-1 AppManifest, O-4 canvas WORKFLOW edge model, N-6 provenance lattice, G-1 taint + G-2 replay, M-5 elai-verify (Tier-2 mount), M-6 evidence-format doctrine
reachable_from:
  - the `elai fleet wire` CLI subcommand binds a tool->tool edge on a live run  [PRODUCTION CLI]
  - (Tier-2) the O-4 WORKFLOW canvas renders/creates the edge on a live dashboard  [PRODUCTION frontend]
  - a #[cfg(test)] caller NEVER satisfies this (R7)
stub_ban:
  - NO live message-broker / canvas runtime bus (A-3; dispatch is the FSM)
  - NO second provenance scale (ride N-6 — A-1)
  - NO untyped/free-text port
  - NO edge that carries bytes without the source's G-1 taint tier
observable_effect:
  trigger: "wire a weather-tool output port into a wardrobe-tool input port via `elai fleet wire`"
  expect_ssb: "one tool_port_edge row {src.out, dst.in, n6_tier, taint_tier} on a successful bind; a schema-incompatible wire lands NO edge row and returns a refusal"
  expect_artifact: "`elai fleet ports --unwired` lists a zero-consumer output; NEGATIVE: an edge bound across incompatible schemas, or an edge row missing the taint tier, must FAIL"

PRODUCTION_HOOK: cli/main.rs `elai fleet wire`/`ports` subcommand dispatch calls the PortRegistry compatibility checker + edge binder + the `tool_port_edge` SSB emit — the executor names the exact dispatch fn at RECON and wires it same-changeset.

## VERIFY
VERIFY: the auditor runs, off-agent:
`python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/P-plan/P-2_inter_tool_ports_world_bus.md --run-trigger`
`python .claude/tools/production_caller_check.py --files <changed .rs> --json`   # DARK new pub fn = BLOCK
pass = a `tool_port_edge` SSB row landed off-agent from a real `elai fleet wire` call, carrying the source's N-6 tier + G-1 taint, and an incompatible wire was refused with no edge row.

INTEGRATION_SMOKE: an independent agent registers three apps (weather=source, wardrobe=consumer, budget=unrelated), wires weather->wardrobe, attempts an incompatible weather->budget wire (must refuse), and asserts the edge row carries the correct taint tier + the unrelated output shows as `--unwired` + G-2 replay of the wardrobe run reproduces the same weather bytes — a multi-app wiring run firing the refusal + taint-flow + replay paths (R10).

## REDTEAM
REDTEAM: attack that FAILED — "launder untrusted weather bytes into a wardrobe sink by wiring an edge that drops the taint tier": FAILS because the stub_ban forces taint-tier propagation along every edge and the SSB row asserts it; a dropped tier is a smoke failure. Attack that SUCCEEDED and its patch — "mount a malicious third-party tool by declaring a conformant port schema it doesn't honor": patched by routing Tier-2 third-party mount through M-5 elai-verify + L-12 admission (the same skillspector-class gate skills pass), so schema-declaration alone never admits a node.

## Grounding
- design digest workflow/reasoning/elai-personal-software-incarnation-2026-07-07.md GAP-2 + T3 mapping + A-1 (no second lattice/scheduler) + A-2 (falsifier: wired-edge count, disconnected-output surfaced rate).
- reddit r/ClaudeAI thread archive — Plane_Garbage "requires a bit of glue"; wtjones "modular tools and glue them together"; nessthing "standardization or modularity".
- feature-map: O-4 canvas WORKFLOW, N-6 provenance lattice, G-1 taint / G-2 VCR, M-5 elai-verify, M-6 evidence format.

## COMPOSES-WITH
P-1 AppManifest (EXTEND with ports), O-4 canvas WORKFLOW (EXTEND edges), N-6 lattice, G-1/G-2, M-5/M-6. Re-implements NONE (GD-1). P-1 owns the manifest; P-6 supplies the `domain_label` on each port; P-3's export bundles ports as part of the mechanism.

End of P-2.
