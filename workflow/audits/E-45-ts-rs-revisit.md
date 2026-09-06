E-45 ts-rs revisit -- 8C deferral audit (2026-05-31, session s-21636)

STATUS: DEFERRED. The 8C sub-phase (ts-rs / specta adoption for application
IPC types) is NOT shipped in Tranche 8. This document records the decision,
the in-tree evidence, the conditions under which the decision should be
revisited, and what the user explicitly needs to OK before the deferral
flips.

DECISION POINTER
The canonical decision is D-10 DECISION-8 (hand-authored TS over ts-rs
codegen). Tranche 8 honored that decision and shipped only 8A
(schemars + JSON Schema fixtures on the 5 sidecar-boundary types) + 8B
(Pydantic mirror regen via datamodel-code-generator) per Option C HYBRID
from context-package-tranche8-E45.md lines 21-23.

IN-TREE EVIDENCE (10x DEFERRED locks)
The hand-authored TS policy is currently locked into 10 in-tree comments
+ the cross_lang_drift module header. Reverting D-10 means edits to all
of these:

  1. crates/dashboard-v2/src/lib/api/types.ts:587
  2. crates/dashboard-v2/src/lib/api/types.ts:2427
  3. crates/dashboard-v2/src/lib/api/types.ts:2472
  4. crates/dashboard-v2/src/lib/api/types.ts:2492
  5. crates/dashboard-v2/src/lib/api/types.ts:2549
  6. crates/dashboard-v2/src/lib/api/types.ts:2636
  7. crates/dashboard-v2/src/lib/api/types.ts:2638
  8. crates/dashboard-v2/src/lib/api/types.ts:2711
  9. crates/dashboard-v2/src/lib/api/types.ts:2857
 10. crates/dashboard-v2/src/lib/api/types.ts:2889

Plus:
  - crates/dashboard-v2/src/lib/drawers/ModeRoutingPanel.svelte:24 ("Hand-authored per D-5 DECISION-7 (ts-rs not adopted in this repo).")
  - crates/orchestrator/src/cross_lang_drift/mod.rs:22-25 ("E-45 plan's Tasks 1+2 (ts-rs codegen) are intentionally DEFERRED -- this module is the substitute that lints the hand-authored mirror by presence + serde-shape probes.")
  - crates/contracts/tests/e45_cross_language_drift_contract.rs:21-24 (same DEFERRED stance)
  - crates/orchestrator/src/empire/types.rs:11 ("this workspace does NOT use the ts-rs derive.")

USER-REVISIT GATE
Before adopting ts-rs / specta for application IPC types, the user MUST
explicitly answer YES to:

  Q1. Are we accepting the cost of rewriting the 10 in-tree DEFERRED comments + the cross_lang_drift module header to remove the lint-only substitute pattern ?
  Q2. Are we accepting that ts-rs adds a Rust workspace MSRV floor (ts-rs 12.0.1 requires Rust >= 1.88.0 per the crates.io spec page verified 2026-05-31) ?
  Q3. Are we accepting the new crates/ts_bindings/ crate registration in .elai_cc/Cargo.toml workspace.members + the snapshot-test infrastructure (~150 LOC of new test code, ~30 LOC of derive wrapper types) ?
  Q4. Is the cost of MAINTAINING auto-generated .d.ts files (i.e. regen workflows + reviewer overhead on auto-generated TS diffs) lower than the cost of the existing 6-pair DriftDetector lint approach ?

If any answer is NO or "not sure", the deferral stays.

CONSERVATIVE RECOMMENDATION
Keep hand-authored TS for application IPC types. Revisit when EITHER:

  (a) the boundary-type count grows past 30 (currently 6 pairs cover
      all sidecar + IPC + Tauri surfaces -- well under the threshold);
  (b) drift incidents exceed 3 in any 30-day window (DriftDetector
      surfaces these via `SsbActionType::CrossLangDriftDetected` rows
      -- run `elai drift list --since 30d` to count).

Today (2026-05-31), the boundary-type count is 6 and the DriftDetector
has surfaced zero drift incidents since Tranche 8 land. Both signals
indicate the substitute substrate is doing its job.

WHAT TRANCHE 8 SHIPPED INSTEAD
- 8A: schemars 1.2.1 derive on 5 sidecar-boundary types
  (MessageRequest / MessageResponse / InputMessage / SystemBlock /
  ToolDefinition in crates/providers/src/types.rs). The 5 JSON Schema
  fixtures live at crates/orchestrator/tests/schemas/*.json and are
  guarded by a snapshot test at
  crates/orchestrator/tests/schemas_snapshot.rs.
- 8B: Pydantic mirror regen pipeline via .claude/tools/regen_sidecar_schemas.py
  (subprocess + datamodel-code-generator 0.59.0, MIT, koxudaxi/
  datamodel-code-generator -- license verified 2026-05-31). Output
  lands at python/elai_inference/schemas/_generated.py + is re-exported
  via the schemas/__init__.py facade.
- SCHEMA_VERSION lockstep bumped: messages.py 138 -> 191 to match
  crates/orchestrator/src/ssb.rs::SCHEMA_VERSION_PUBLIC.

WHAT 8C WOULD ADD (deferred)
- crates/ts_bindings/ -- new workspace member, ts-rs 12.0.1 dep,
  ~150 LOC of mirror types + snapshot tests.
- crates/dashboard-v2/src/lib/types/generated/ -- auto-generated
  .d.ts files committed to source control.
- Updates to all 10 in-tree DEFERRED comments listed above.

NEXT REVIEW DATE
Trigger: explicit user request, OR drift-incident-count > 3 in 30 days,
OR boundary-type count > 30. Whichever comes first.
