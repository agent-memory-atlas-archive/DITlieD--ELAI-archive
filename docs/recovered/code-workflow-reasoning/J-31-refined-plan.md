# J-31 (refined) — CANVAS SYSTEM-COMPOSITION MODE BACKEND

Source: cursor pickup (walk-state) + `conductor/state/wiring-reports/J-31.md`
(2026-06-29 STATE-CHECK @ 9760574) + prior EXECUTE-1 patch
(`conductor/artifacts/J-31-EXECUTE-1/patch.diff`, 1454 lines, authored Jun 28,
never landed) + live reconciliation against HEAD `dec8850` (2026-07-22).
The original spec file `docs/thefinalglance/J-plan/J-31_canvas_system_mode.md`
is DARK (absent tree + all git refs); this file IS the current refined plan.
The frontend UI shell is SHIPPED+WIRED (Half A); this plan ships Half B
(orchestrator backend). The designed contract lives in
`crates/dashboard-v2/src/lib/canvas/system_mode_fixtures.ts` (the seam spec).

## GROUNDING (verified live on HEAD dec8850, 2026-07-22)

- Frontend seam SHIPPED: `SystemMode.svelte` mounted `CanvasWindow.svelte:413`;
  `CanvasMode 'system'` in `canvas.svelte.ts:44`, in `WIRED_MODES` :59;
  `CANVAS_MODE_SYSTEM` const :52; `CanvasModeBar.svelte` surfaces SYSTEM button.
- Backend ALL DARK (grep-confirmed this turn): no `system_canvas.rs`,
  no `SystemContractEvent` in ssb.rs (0 matches), not in `action_type_lock.golden`
  (0 matches), no `SYSTEM_NAMESPACE` in `canvas_dispatcher.rs`, no Tauri cmds.
- SSB layout is now SPLIT MODULES (Tier-B Move 1, df18539):
  `ssb/action_type.rs` (enum + as_sql_str + from_str), `ssb/payloads.rs`
  (SsbPayload + payload structs), `ssb/schema.rs` (SQL CHECK), `ssb/tests.rs`
  (test_payload). `SSB_ACTION_TYPES` array lives in `ssb.rs:62-1628`.
- COUNTERS ARE DERIVED (no hand-edit): `SCHEMA_VERSION = -309 + len(SSB_ACTION_TYPES)`
  (`ssb.rs:1635/1640`); `TOTAL_VARIANTS = -309 + len(REGISTRY)`. Append a
  variant to `SSB_ACTION_TYPES` and the counters auto-bump. Last array entry:
  `NodeAdmissionEvent(NodeAdmissionEventKind::Admitted)` (J-39 T2, ssb.rs:1627).
- Pattern-2 multi-arm TEMPLATE to mirror (the most recent): `NodeAdmissionEvent`
  — footprint across 8 sites: `node_admission.rs` (emitter+payload+kind),
  `lib.rs:2563` (pub mod + pub use), `ssb.rs:1627` (SSB_ACTION_TYPES entry),
  `ssb/action_type.rs` (enum arm), `ssb/schema.rs:5558` (CHECK entry),
  `ssb/payloads.rs` (SsbPayload arm), `ssb/tests.rs:3885` (test_payload arm),
  `task_runner.rs:18105` (production caller).
- VERDICT POLICY (TD-3): the LLM NEVER sets a verdict; verdicts derive from
  checker output + journal rows. STORE INVARIANT (GD-2): the journal (SSB) is
  the SOLE store; the canvas re-derives state byte-identically from rows.
- Frontend contract types (mirror EXACTLY): `SystemBindingKind` =
  plan_unit|contract|mcp_service|draft; `SystemNodeLifecycle` =
  draft|framed|building|wired|verified|drifted; `SystemEdgeVerdict` =
  declared|wired|verified|broken; `SystemEdgeContract` =
  {produces, consumes, reachable_from, observable_effect, evidenceRow?}.

## EXECUTOR LANES (this window)

codex PAID lane is DEAD (both accounts 403 INSUFFICIENT_BALANCE; CODEX_FIRST
absent; credits expired 2026-07-15). EXECUTOR routes to the FREE swe lane
(glm-5-2 main worker) for EVERY unit — owner GO 2026-07-14 grants free-lane
fallback for hard/greenfield units. Free swe has a ~105-turn structural cap
(D-55/D-56): CANNOT land multi-file coupled slices (3× zero-write on K-6's
9-file slice). THEREFORE each unit is a SINGLE-FILE micro-slice with a
pre-digested digest (CONTEXT DISCIPLINE). opus_land gates every free diff
off-agent (REFUSES any path not in --files, runs the frozen checkers over the
real diff), so a free-lane botch cannot dirty main.

## UNIT 1 — SystemContractEvent SSB variant (the keystone primitive)

Single coupled slice (Pattern-2 multi-arm lockstep mirroring NodeAdmissionEvent).
This is the load-bearing primitive every other unit consumes; it CANNOT be
split further (the 8 lockstep sites must move together or the crate breaks).

PRODUCTION_HOOK: emit in `node_admission.rs`-style emitter
`emit_system_contract_event` (new file `crates/orchestrator/src/system_contract_event.rs`)
called from the S6 auditor idle tick (task_runner.rs); SSB write is the seam.
WIRING_CONTRACT: SystemContractEvent SSB row —
`SsbActionType::SystemContractEvent(SystemContractEventKind::NodeBound)` added
to `SSB_ACTION_TYPES` (ssb.rs, appends after NodeAdmissionEvent); enum arm in
`ssb/action_type.rs`; `as_sql_str`="system_contract_event"; `from_str` arm;
SQL CHECK entry in `ssb/schema.rs`; `SsbPayload::SystemContractEvent` arm +
`SystemContractEventPayload` + 4-arm `SystemContractEventKind`
(NodeBound/EdgeDeclared/EdgeVerdict/NodeDrifted) in `ssb/payloads.rs`;
test_payload arm in `ssb/tests.rs`; golden regenerated via
`action_type_lock.rs`'s blessed `regen_golden_file --ignored`.
SCHEMA_VERSION/TOTAL_VARIANTS auto-derive from the array append (no hand-edit).

EXPECTED_OBSERVATION: after landing, `cargo check -p elai-orchestrator
--all-targets` (CARGO_TARGET_DIR=$HOME/.cache/cargo-target-elai per D-12
DISK-SAFETY) exits 0; `cargo nextest run -p elai-orchestrator action_type_lock`
PASSES (golden matches the new variant set); `python3 .claude/tools/
production_caller_check.py --symbols SystemContractEvent` reports the emitter
WIRED (>=1 distinct non-test production caller in task_runner.rs); `grep
system_contract crates/orchestrator/tests/action_type_lock.golden` shows the
new row; `python3 .claude/tools/wiring_contract_check.py` PASSES.

ON_FAIL: cargo E0004 non-exhaustive match -> CAUSE a match arm in ssb.rs or
payloads.rs was not extended for the new variant -> COUNTER-MOVE add the arm
(switch on SsbActionType/SsbPayload must list every variant). golden mismatch
(LOCK_FAILED) -> CAUSE golden regenerated against a stale array -> COUNTER-MOVE
rerun `cargo test action_type_lock -- --ignored regen_golden_file` then re-run
the lock test. production_caller_check DARK -> CAUSE emitter fn has no
non-test caller -> COUNTER-MOVE wire emit_system_contract_event into the
task_runner auditor idle tick (the production caller).

FORK: if `wiring_contract_check.py` flags the CHECK-constraint allowlist as
missing the new action_type (the D-47 / R9 class: a separate non-auto-derived
DDL allowlist in ssb/schema.rs) -> take route B: ALSO add the
`system_contract_event` string to the `CHECK(action_type IN (...))` literal in
ssb/schema.rs (the array append does NOT touch this hand-maintained list).

RECON_NEEDED: confirm `SystemContractEventKind` 4 arms match the frontend
`SystemEdgeVerdict` + lifecycle set (declared/wired/verified/broken map to
NodeBound/EdgeDeclared/EdgeVerdict/NodeDrifted) — settles via reading
system_mode_fixtures.ts (done, cited in GROUNDING).

ABORT: if `ssb/payloads.rs` SsbPayload enum is not at the line cited (drift),
STOP and report the new line; if `all_test_payloads`/test_payload shape does
not match NodeAdmissionEvent's (ssb/tests.rs:3885), STOP and report — do not
improvise the test harness shape.

VERIFY: `$ cargo nextest run -p elai-orchestrator action_type_lock`
(off-agent exit 0, the golden file is the artifact the builder did not author);
`$ python3 .claude/tools/production_caller_check.py --symbols SystemContractEvent`
(WIRED verdict, off-agent); `$ python3 .claude/tools/wiring_contract_check.py`
(PASS, off-agent).

REDTEAM: attack — a worker adds the SsbActionType enum arm but forgets the
`SSB_ACTION_TYPES` array append, so SCHEMA_VERSION does NOT bump while the
enum claims a new variant (silent registry/enum skew). WHY IT FAILED against
this plan: VERIFY runs `action_type_lock` which compares the golden
(byte-derived from `SSB_ACTION_TYPES`) against the live enum set; a skew fails
the lock test RED. Patch born: none needed — the lock test is the guard.

## UNIT 2 — system_canvas.rs binding registry + verdict derivation

Single-file slice (new file `crates/orchestrator/src/system_canvas.rs`).
Depends on Unit 1 (consumes SystemContractEvent SSB rows). Ships the
read-side re-derivation (GD-2: journal is sole store) + the DTOs mirroring
the frontend shapes exactly + the verdict-derivation fn that maps
wiring_contract_check output to declared/wired/verified/broken (TD-3: LLM
never sets verdict).

PRODUCTION_HOOK: `get_system_canvas()` re-derives {nodes, edges} from SSB
rows; called by the Unit 4 Tauri read command.
EXPECTED_OBSERVATION: `cargo check -p elai-orchestrator` exits 0 with the new
module; `pub mod system_canvas;` + `pub use` in lib.rs; a unit test
`system_canvas_re_derives_from_journal` constructs NodeBound+EdgeDeclared rows,
calls get_system_canvas, asserts the nodes/edges match.
ON_FAIL: borrow/lifetime error in the SSB read path -> CAUSE Ssb read API
returns owned vs borrowed -> COUNTER-MOVE clone the row into the DTO (DTOs are
Serialize, owned is correct).
FORK: if wiring_contract_check.py cannot be shelled from the Rust verdict
derivation (no std::process seam, or the checker lives under .claude/tools
unreachable at runtime) -> take route B: verdict derivation consumes
PRE-COMPUTED checker results passed in by the auditor tick (the auditor shells
the checker; system_canvas only maps result->verdict), not a runtime shell.
RECON_NEEDED: confirm the Ssb read API method name for streaming rows by a
symbol (get_system_canvas mirrors how other read-side DTOs query the board —
grep `ssb_viewer` or `Ssb::read` for the precedent).
ABORT: if no read API exists to stream SystemContractEvent rows by action_type,
STOP (the unit needs a Unit-1.5 read API addition first).
VERIFY: `$ cargo nextest run -p elai-orchestrator system_canvas` (off-agent
exit 0); the unit test is the artifact.
REDTEAM: attack — the verdict fn accepts an LLM-supplied verdict string and
echoes it (TD-3 violation). WHY IT FAILED: the fn signature takes only checker
result + journal rows, never a verdict param; the unit test asserts a broken
checker result yields verdict=broken regardless of any node input.

## UNIT 3 — SYSTEM_NAMESPACE + op handlers in canvas_dispatcher.rs

Single-file slice (modify `crates/orchestrator/src/canvas_dispatcher.rs`).
Adds `pub const SYSTEM_NAMESPACE: &str = "system";` + bind_node/declare_edge/
set_display op handlers gated to SYSTEM_NAMESPACE in check_mode_allowed.
Depends on Units 1-2 (the ops emit SystemContractEvent rows via system_canvas).
PRODUCTION_HOOK: check_mode_allowed dispatches a "system" mode op to the
system_canvas emitter.
EXPECTED_OBSERVATION: `cargo check -p elai-orchestrator` exits 0; a test
asserts check_mode_allowed(SYSTEM_NAMESPACE, "bind_node") returns Allowed while
an unknown op returns Denied.
ON_FAIL: ARCH_NAMESPACE/DESIGN_NAMESPACE pattern not followed -> CAUSE the
existing consts sit at a different line -> COUNTER-MOVE mirror the exact
const + match-arm shape of ARCH_NAMESPACE (canvas_dispatcher.rs:44).
FORK: none (deterministic mirror of an existing namespace).
RECON_NEEDED: confirm ARCH_NAMESPACE op-handler shape by reading
canvas_dispatcher.rs:44-60.
ABORT: if canvas_dispatcher.rs has no check_mode_allowed fn (renamed), STOP.
VERIFY: `$ cargo nextest run -p elai-orchestrator canvas_dispatcher` exit 0.
REDTEAM: attack — an op handler accepts a verdict from the canvas payload
(TD-3). WHY IT FAILED: handlers only accept bind/declare/display params, never
a verdict; the verdict is always derived in Unit 2.

## UNIT 4 — Tauri commands + frontend swap (dashboard-v2)

Two-file slice: new `crates/dashboard-v2/src-tauri/src/system_canvas_cmds.rs`
(get_system_canvas + refresh_system_verdicts #[tauri::command] handlers +
generate_handler! registration) + modify `SystemMode.svelte` to swap
SYSTEM_CANVAS_FIXTURE for a tryInvoke('get_system_canvas') call. These two
files are tightly coupled (the command name must match the invoke) and compile
apart from the orchestrator, so one slice.
PRODUCTION_HOOK: SystemMode.svelte onMount invokes get_system_canvas; the
Tauri command calls system_canvas::get_system_canvas (Unit 2).
EXPECTED_OBSERVATION: `cargo check -p dashboard-v2` exits 0; `cross_lang_diff
.py` PASSES (ts-rs bindings generated); SystemMode.svelte renders real (empty
when no SSB) instead of the fixture.
ON_FAIL: ts-rs binding generation fails -> CAUSE the DTO lacks #[derive(TS)]
-> COUNTER-MOVE add the TS derive on SystemCanvasDto in Unit 2 (threaded
contract) OR a src-tauri local binding type.
FORK: if the SSB DB is unreachable from the Tauri command context -> route B:
get_system_canvas returns the EMPTY_CANVAS DTO (Unit 2 already defines this).
RECON_NEEDED: confirm the generate_handler! registration file path in
src-tauri/src by grepping an existing command registration (e.g. the canvas
arch/design commands).
ABORT: if no existing canvas command to mirror in src-tauri, STOP.
VERIFY: `$ cargo check -p dashboard-v2` exit 0; `$ python3 .claude/tools/
cross_lang_diff.py` PASS; manual: the SYSTEM canvas mode renders (empty).
REDTEAM: attack — the frontend reads the fixture when the backend returns empty
(masking a real backend gap). WHY IT FAILED: SystemMode.svelte's fixture
import is REMOVED in this unit; only tryInvoke drives the render.

## UNIT 5 — wire-dark-allowlist + INTEGRATION_SMOKE

If any backend piece ships behind a kill-switch flag (ELAI_*_ENABLE), land its
wire-dark-allowlist.toml row (W-2 / R9). Then the integration smoke.

INTEGRATION_SMOKE: drive the REAL `elai` entrypoint in canvas system mode on a
HARD multi-node input (bind 3 nodes + 2 edges, one edge deliberately
undeclared), assert off-agent: the get_system_canvas read returns the 3 nodes
+ 2 edges, the undeclared edge has verdict=declared (NOT verified — TD-3), and
a SystemContractEvent SSB row exists per bind (off-agent golden/SSB read, not
stdout). Strict-peer provider per the smoke cadence.

## DISPATCH ORDER (this session, budget permitting)

Unit 1 first (keystone; everything depends on it). Then Unit 2. Each is a
separate free-lane dispatch with a pre-digested digest (the NodeAdmissionEvent
lockstep footprint for Unit 1; the Ssb read precedent for Unit 2). Units 3-5
are subsequent sessions (plan boundary). If Unit 1 free-lane returns empty
diff twice (D-9), SET_ASIDE — do NOT spin.
