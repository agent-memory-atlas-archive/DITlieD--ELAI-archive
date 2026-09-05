# P-1: the app-fleet layer — the user's N bespoke apps as first-class objects, health-ticked and gated-auto-repaired [KEYSTONE]

STATUS: PLANNED (WAVE 1; PROOF_SPINE: none (product-surface / distribution class), owner-acked 2026-07-07; BUILD-on-shipped — the fleet grain)

- Track: PERSONAL-INCARNATION | lever: fleet-grain-over-taskspec | source finding: GAP-1 (answers T2 keep-it-alive) | grounding: MODERATE (index-derived, PLAN-asserted) | nature: ~80% composition | effort: M-L | WAVE 1

- Premise: the reddit bespoke-tools thread's single most load-bearing unsolved problem is maintenance — "to keep the little magic box alive, and up to date with the changing state of the world, that's 100% on you now" (florinandrei); "turning them into standalone applications is the real pain" (arjuna66671). No corpus letter owns "the user's N bespoke apps" as first-class objects. P-1 makes the APP the unit of care (one grain above the corpus's changeset/task) and turns florinandrei's "100% on you" into "journaled, scheduled, and receipted." It is the KEYSTONE of the P-plan: every other unit binds to the per-app manifest this unit defines.

## Steal/build
An `AppManifest` row: `{ app_id, one_sentence_purpose, data_namespaces: [ns], ports (P-2 typed input/output schemas), smoke_trigger, rra_lineage, dep_manifest }`. A `FleetRegistry` (SQLite, rows-as-data like the rules engine / O-2 TaskSpec) enumerates every ELAI-built app. A health tick is NOT a new scheduler — it is an O-2 TaskSpec with `trigger=Cron`, `pipeline=[Smoke, DriftCanary, DepRotCheck]`, `sink=DashboardInbox`. Per-app drift canaries GENERALIZE G-10 (ELAI's own model-drift regression canary) from ELAI's canary set to each app's declared smoke set — this is the literal "roll back when a model update breaks your use case" gap. Dependency-rot watching points F-9's existence/age gate at each app's `dep_manifest` on a TTL. When a tick goes red, ELAI proposes a fix that MUST pass the same wiring/claim gates as any changeset (never a silent patch), rendered human-first via N-7, one-click apply — gated AUTO-REPAIR.
EXCLUDED: a second scheduler (reuse O-2's TaskSpec + task_runner interval); auto-repair that bypasses the wiring/claim gates (A-repair-must-gate — a repair is a changeset, not an exception); a per-app code fork of the health machinery (the manifest is DATA, the tick engine is one substrate — A-5 two-masters); silent rollback (every repair/rollback is journaled + N-7-rendered).

## Existing ground truth (PLAN-asserted; executor re-greps per W-7)
- O-2 ships the typed TaskSpec scheduler (Cron/Manual triggers, journaled fires, DashboardInbox sink) — a health tick is a TaskSpec, not new machinery.
- G-10 is the model-drift regression canary (detects when a model swap regresses behavior) — generalize its canary-set abstraction per-app.
- F-9 is the dependency existence/age gate; F-5 is staleness quarantine — both point at a manifest.
- L-14 `elai build "<idea>"` emits an RRA receipt — the manifest's `rra_lineage` is that receipt's id; the fleet registers what L-14 builds.
- N-7 is the human-render verdict field — the red-tick diagnosis + the proposed repair render through it.
- M-4's signed-updater manifest/rollback pattern is the shape (but M-4 updates ELAI itself; P-1 applies the pattern per-user-app — this is GAP-1's distinction from M-4).

## REVISED scope (W-10 tiered)
- SHIP NOW (Tier-1): the AppManifest schema + FleetRegistry SQLite store + a `elai fleet register <app>` CLI caller + the health-tick TaskSpec template (Smoke + DepRotCheck stages, reusing O-2 + F-9) + the red-tick DashboardInbox card rendered via N-7.
- DEFER (Tier-2): per-app drift canaries (needs G-10 generalized — the generalization is asserted-not-designed per doc A-6, lands with a G-10 wiring pass) + gated auto-repair apply-button (crosses into the changeset/claim-gate seam — land with the wiring pass that threads it).

## Integration
- crates/orchestrator (or the rules-engine crate): the AppManifest table + FleetRegistry loader + the health-tick TaskSpec factory; the red-tick SSB emit.
- CLI: `elai fleet register|list|tick <app>` subcommand (cli/main.rs) is the Tier-1 production caller.
- frontend_pair: the DashboardInbox surface (O-2's intervention-card sink) renders the red-tick card; a dedicated FleetPanel is deferred to P-5's operator surface.

## MOVES (wargame)
M1 add the AppManifest table + strict deserializer (closed field set, `data_namespaces` validated against the O-3 vault namespace-claims) and the FleetRegistry loader that enumerates apps at boot.
EXPECTED_OBSERVATION: `elai fleet register <app>` with a valid manifest lands a registry row; `elai fleet list` enumerates it; a manifest naming an unclaimed namespace is a hard refusal.
ON_FAIL: a manifest with an unclaimed namespace registers anyway -> the P-6/O-3 privacy boundary is bypassed at registration -> validate `data_namespaces` against the namespace-claims file at register time, add the negative test.
M2 emit a health-tick TaskSpec per registered app (O-2 factory) whose pipeline runs the app's `smoke_trigger` + an F-9 DepRotCheck, and journal a `fleet_health_tick` SSB row per tick carrying `{app_id, verdict, red_reasons}`.
EXPECTED_OBSERVATION: at the due tick each app's smoke fires exactly once and a `fleet_health_tick` row lands; a broken app's tick lands `verdict=red` with populated `red_reasons`.
ON_FAIL: a tick fires but no row lands (the O-2 fire-row seam wasn't threaded) -> missing evidence is FAIL (R1) -> assert the row in the smoke, not the stdout.
M3 render a red tick as a DashboardInbox card through N-7 (plain-language diagnosis: which app, what broke, one sentence) and auto-archive a green tick.
EXPECTED_OBSERVATION: a red tick renders one N-7 card ("your wardrobe tool's weather dependency is 14 months stale"); a green tick leaves only the SSB row, no card.
ON_FAIL: green ticks spam the inbox -> the auto-archive-on-green predicate is missing -> reuse O-2's nothing-to-report->archive branch keyed off `verdict`.

## FORK
FORK: if G-10's canary abstraction is NOT extractable per-app without restructuring it (doc A-6 warns the generalization is asserted, not designed) -> ship Tier-1 with the Smoke + DepRotCheck stages only (no drift canary), route the drift canary to Tier-2, and record the G-10-generalization seam in the decision doc. Do NOT re-implement a second canary engine.

## RECON_NEEDED
RECON_NEEDED: whether G-10's canary set is parameterizable by an app's smoke set or is hard-bound to ELAI's own regression fixtures -> settle via `grep -rn "canary\|drift\|G-10\|regression" .elai_cc/crates` + codegraph_explore on the G-10 canary loader BEFORE committing to Tier-1 drift-canary scope.
RECON_NEEDED: the O-2 TaskSpec factory API (how a spec is created programmatically vs authored as a row) -> settle via reading O-2's shipped TaskSpec loader + `grep -rn "TaskSpec\|task_spec" .elai_cc/crates`.
RECON_NEEDED: the F-9 dep existence/age checker's callable entrypoint (does it take a manifest path or a Cargo.lock?) -> settle via `grep -rn "F-9\|dep.*age\|existence.*gate\|slopsquat" .elai_cc/crates`.

## ABORT
ABORT: if wiring gated auto-repair would let a repair apply WITHOUT passing the wiring_contract_check / claim gates that any changeset passes -> stop and flag; a repair is a changeset (it must clear the same gate stack), never a privileged silent patch. Ship the diagnosis + the propose-fix render; the apply-button lands only once it routes through the real gate seam.

## WIRING_CONTRACT
produces_symbols:
  - the AppManifest schema (closed field set) + its SQLite table and FleetRegistry loader
  - the `elai fleet register|list|tick` CLI subcommand
  - the health-tick TaskSpec factory + the journaled `fleet_health_tick` SSB row + the red-tick N-7 card + auto-archive-on-green
consumes_symbols:
  - O-2 TaskSpec scheduler + DashboardInbox sink, F-9 dep gate, F-5 staleness, G-10 canary abstraction, N-7 human-render, L-14 RRA lineage, O-3 namespace-claims (namespace validation)
reachable_from:
  - the `elai fleet register` CLI subcommand on a live run registers an app  [PRODUCTION CLI]
  - the task_runner interval fires a due health-tick TaskSpec on a live run  [PRODUCTION scheduler tick]
  - a #[cfg(test)] caller NEVER satisfies this (R7)
stub_ban:
  - NO second scheduler loop or second inbox surface (reuse O-2)
  - NO auto-repair that bypasses the changeset wiring/claim gates
  - NO per-app code fork of the tick engine (manifest is data; A-5)
  - NO health tick without a journaled `fleet_health_tick` SSB row (R1)
observable_effect:
  trigger: "register an app with a deliberately-broken smoke, advance the task_runner tick"
  expect_ssb: "one fleet_health_tick row {app_id, verdict=red, red_reasons} per due tick, exactly once"
  expect_artifact: "a DashboardInbox N-7 card naming the app + the plain-language break reason; NEGATIVE: a green tick that still cards, or a red tick with empty red_reasons, must FAIL"

PRODUCTION_HOOK: cli/main.rs `elai fleet` subcommand dispatch (Tier-1 register/list/tick) AND the task_runner interval registration seam (C-9) that fires the per-app health-tick TaskSpec — the executor names the exact registration fn at RECON and wires both same-changeset.

## VERIFY
VERIFY: the auditor runs, off-agent:
`python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/P-plan/P-1_app_fleet_layer.md --run-trigger`
`python .claude/tools/production_caller_check.py --files <changed .rs> --json`   # DARK new pub fn = BLOCK
pass = a `fleet_health_tick` SSB row landed off-agent from a real task_runner tick (or `elai fleet tick`), fired exactly once, with a red-tick inbox card for the broken-app case and auto-archive for the green case.

INTEGRATION_SMOKE: an independent agent registers TWO apps (one healthy, one whose smoke breaks after a simulated dep-rot), advances several ticks, and asserts single-fire journaling per app + the red N-7 card for the broken one + green auto-archive for the healthy one + no cross-app leakage of red_reasons — a multi-app multi-tick run firing the idempotency + auto-archive paths, not one tick (R10).

## REDTEAM
REDTEAM: attack that FAILED — "make a red tick silently fix an app and hide the change": FAILS because the stub_ban + ABORT force any repair through the changeset wiring/claim gates and journal it; there is no silent-patch path. Attack that SUCCEEDED and its patch — "register 200 trivial apps so the fleet tick storms the task_runner": patched by making the health-tick a single batched TaskSpec that iterates the registry inside one fire (one `fleet_health_tick` row per app, one dispatch per tick), not 200 competing Cron specs.

## Grounding
- design digest workflow/reasoning/elai-personal-software-incarnation-2026-07-07.md GAP-1 + T2 mapping + A-2 (falsifier: fleet-health green %, median time-to-repair a red tick) + A-6 (the surviving-attack RECON: G-10 per-app generalization).
- reddit r/ClaudeAI thread archive (bespoke tools, 1uopekl) — florinandrei "100% on you now"; arjuna66671 "turning them into standalone applications is the real pain".
- Ink & Switch malleable-software essay (inkandswitch.com/essay/malleable-software, June 2025) — adaptation at the point of use as the named field.
- feature-map: O-2 TaskSpec, G-10 drift canary, F-9 dep gate, F-5 staleness, L-14 elai-build RRA, N-7 human-render, M-4 updater pattern.

## COMPOSES-WITH
O-2 TaskSpec scheduler + DashboardInbox (EXTEND, do NOT re-author), G-10 canary (generalize), F-9/F-5 (point at manifest), L-14 RRA lineage, N-7 render, O-3 namespace-claims. Re-implements NONE (GD-1). P-2 supplies the manifest's typed ports; P-5 renders the fleet at the operator grain; P-6 validates the manifest's namespaces against the privacy lattice.

End of P-1.
