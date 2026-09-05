# N-8 seam proposal — governance-coverage substrate

TASK: opuswalk-N-8-PLAN-1787805792-20ca2e | ROLE: PLAN (K-27 seam design)
DATE: 2026-08-27 | BASE: HEAD (main @ b9efaf5e; prior evidence base 975d1b76)
QUEUE REASON: SET_ASIDE after S3 wired=no and plan_gate FAIL (P1 EXPECTED_OBSERVATION / P3 FORK / P5 ABORT / P6 VERIFY / P7 REDTEAM / P10 INTEGRATION_SMOKE missing — see conductor/opus_walk/evidence/N-8/plan-gate.json).
RE-ENTRY CONDITIONS MET BY THIS DOC: explicit production seams per view, EXPECTED_OBSERVATION, VERIFY artifact, REDTEAM, ABORT conditions. This is a seam-design doc, not the plan rewrite itself; the opus controller owns the re-entry rewrite decision.

## Prior evidence cited (all under conductor/opus_walk/evidence/N-8/)

recon.md · S1-context.md · S2-integration.md · S3-backend-precheck.md · S3-frontend-precheck.md · plan-gate.json

## Ground state (from evidence, executor must re-grep current main before EXECUTE)

- N-8 produced symbols are ALL DARK: no governance-coverage checker, no `governance_coverage_measured` SSB action_type variant, no mutant-corpus/kill-rate lifecycle field in `.claude/gate-manifest.txt`, no governance row in the N-1 spine (S3-backend-precheck §1–3).
- Shipped and wired: B-17 gate-liveness (test binary + CI runner), H-27 mutant-kill (`validate_gates*` + `elai gate-selfcheck` CLI), N-1 measurement spine (2 bench rows + `elai bench index` CLI caller).
- Partially shipped: J-70 DO-333 trace schema IS present in manifest rows + validated by `gate_freeze_check.py --trace` (5 tab-separated fields; exit 0 observed, checked=71). J-70's own full instrument (auditor_budget_pool, H-10 carve-out) remains unshipped — do not conflate schema presence with the J-70 unit landing (S3-backend §"Consumed instrument state").
- Genuinely DARK: K-22 doctrine-enforcer coverage (no doctrine_coverage.py anywhere), J-46 debt-rate (no doctrine_debt_rate.py, no ledger). DoctrineEnforcer and DebtRate views MUST fail-closed as UNMEASURED. Never fabricate a number (plan NEGATIVE observable_effect: a decorative rule reported covered must FAIL).
- Owner UI hold active (S3-frontend): no tile, no Tauri command, no .svelte in any slice below. Planned G-5/M-3 tile is downstream.

## The five views and their real seams on CURRENT main

Evidence-pack citations were taken at base 975d1b76 (2026-08-12/13); main has moved (HEAD b9efaf5e). Every citation below marked OK was re-grepped against CURRENT main (b9efaf5e) by this worker on 2026-08-27 (verification log at the end of this doc); remaining caveats are named inline. Slice 0 keeps a cheap re-grep duty because main moves between PLAN and EXECUTE.

| # | View (kind) | Production hook / source seam | State |
|---|---|---|---|
| V1 | B-17 gate-liveness | Runner `.elai_cc/scripts/check_gate_liveness.sh:47` (invokes `cargo test … gate_liveness`, parses result :54-93; file is 94 lines) → test binary `.elai_cc/crates/orchestrator/tests/gate_liveness.rs` — 8 `#[test]`, active wired tests start `:156 "Wired gate 1/4"`; the 2 `#[ignore]` wiring stubs are at :348 (B-4-blocked) and :442 (B-2-blocked) and must not count as live coverage. CI workflow exists at `.elai_cc/.github/workflows/gate-liveness.yml` (inside the `.elai_cc` workspace — NOT repo-root `.github/workflows/`; recon's root-relative path was stale) | OK @b9efaf5e |
| V2 | K-22 doctrine-enforcer coverage | NONE — DARK upstream confirmed on b9efaf5e (no `doctrine_coverage.py` anywhere). Seam this unit creates: fail-closed arm of the checker that reports UNMEASURED when `doctrine_coverage.py` output/ledger is absent | REFERENCE ONLY |
| V3 | J-46 debt-rate | NONE — DARK upstream confirmed on b9efaf5e (only match for doctrine-debt naming is the plan doc `docs/thefinalglance/J-plan/J-46_doctrine_debt_rate_measurement.md`). Same fail-closed treatment as V2 (UNMEASURED), nameable as DEFERRED residue | REFERENCE ONLY |
| V4 | J-70 traceability | `.claude/tools/gate_freeze_check.py` — `_parse_manifest_rows` :78-110, `_validate_trace_rows` :116, `run()` :172, `main()` :274 with argparse `--manifest/--lock-file/--root/--lock/--trace/--json` (:277-284); rows in `.claude/gate-manifest.txt` (296-line file, 71 tab-separated frozen rows, 5-field DO-333: path/objective_covered/technique/evidence_artifact/satisfaction) | OK @b9efaf5e |
| V5 | H-27 mutant-kill generalized | Producer `.elai_cc/crates/orchestrator/src/gate_self_validation.rs` — frozen `REGISTRY: &[GateEntry]` const :70, `GateSelfValidationOutcome` :179 + `is_pass()` :199, `validate_gates` :227 (pub async), `validate_gates_with` core driver (doc header :242-243), `emit_outcome` :442 (payload constructed through SSB `GateSelfValidation`). CLI dispatch: `.elai_cc/crates/cli/src/main.rs:1368` `CliAction::GateSelfcheck` → `gate_selfcheck_cmd::run` :1374 → `elai gate-selfcheck`. Frozen registration row `.claude/gate-manifest.txt:133` (META comment block starts :124) | OK @b9efaf5e |

### V1 — B-17 gate-liveness (live leg)
Consume the *observed liveness* B-17 already produces: the wired production gate-fire sites named by recon (`GroundingSweepGate`, `ground_plan`, `generate_csq`, `check_inline_feasibility`, `emit_auditor_rubber_stamp_rejected`, `get_evidence_gates`; callers in task_runner.rs / engine.rs / dashboard). Do NOT recreate the runner, tests, or gate functions (RESPECT: shipped B-17 surfaces). The checker joins the count of live B-17 liveness probes into GateLiveness; counting rule: active tests only (ignored stubs excluded), taken from the same inventory B-17's runner reports — never hand-maintained.

### V4 — J-70 traceability (join-only, zero rebuild)
The 5-field DO-333 schema is present TODAY and validated by the existing `--trace` mode (S3-backend §4 ran it green, checked=71). Join rule: the checker reads the same parsed rows `_parse_manifest_rows` yields and reports one Traceability entry per row whose required fields validate. If the live parser's field set differs on current main, follow the live code and label deviations; never fork a second parser or touch gate files (frozen to executors; forbidden edits).

### V5 — H-27 generalized to every frozen gate (Tier-1 admission rule, not a second validator)
H-27 today validates its fixed Track-3/finance 8-gate registry, not arbitrary manifest entries. N-8 adds the ADMISSION requirement, implemented at the freeze seam: a NEW gate entering `.claude/gate-manifest.txt` must declare a frozen mutant corpus reference AND a measured kill rate (100% per H-27 contract) or `run()` refuses the addition before hash-locking/acceptance. Kill figures come from running the existing `validate_gates_with` scoring over the declared corpus, reusing `GateSelfValidationOutcome.is_pass` semantics (fail-closed: survivors ⇒ refuse). Tier-2 (back-filling corpora for the ~40 legacy gates) stays DEFERRED and must NOT block entry rules for new gates. No changes to gate_self_validation.rs internals (RESPECT: shipped H-27 producer).

### V2/V3 — K-22 + J-46 fail-closed arms
Emit `DoctrineEnforcer=UNMEASURED` / `DebtRate=UNMEASURED` with the absence itself cited (path greps recorded in the run journal). A clause/policy string found without a machine enforcer counts as 0 enforcement contribution for the headline numerator — never rounded up, never estimated. When K-22/J-46 land later, they slot into these two arms without touching the emit contract.

## Production hook: where governance_coverage_measured emits

PRIMARY EMIT SEAM: the registered SSB action enum in `.elai_cc/crates/orchestrator/src/ssb/action_type.rs` — add `GovernanceCoverageMeasured` immediately adjacent to `SsbActionType::GateSelfValidation` (:3885 enum variant site), then the same-lockstep arm of the registry: `Self::GateSelfValidation => "gate_self_validation"` inside `as_sql_str` (:5747) and `"gate_self_validation" => Ok(...)` inside `from_str` (:6639). LOCKSTEP REQUIREMENT (S3-backend notes + E-44's discovery of a wider lockstep): ALL registration sites in the SAME changeset — enum variant + as_sql_str + from_str + payload/schema sites + the `TOTAL_VARIANTS` bump (currently 598, comment ladder near :4882) + regeneration of `tests/action_type_lock.golden` / `action_type_lock.rs`. A half-registered variant fails the audit, and since commit 23a3e9ad added 35 variants atomically across plans, treat those two test files as part of the emit-seam surface.

CALL SITE — PRODUCTION GOVERNANCE SEAM: the real caller is the new CLI subcommand in `.elai_cc/crates/cli/src/main.rs`, dispatched in the `CliAction` match beside the two existing production precedents (`CliAction::GateSelfcheck` arm :1368 calling `gate_selfcheck_cmd::run` :1374; `CliAction::BenchIndex` :1581 which consumes `measurement_spine::{dark_families, spine_entries}` and prints via `render_spine_table` :1600). Proposed command shape: `elai governance-coverage [--json]` → runs the checker → emits one `governance_coverage_measured(GateLiveness|Traceability|MutantKill|…)` event per measured view + the joined headline. FREEZE SEAM CALLER (second reachable_from): `.claude/tools/gate_freeze_check.py::run()` :172 gains the new-gate corpus/kill admission check executed on every freeze invocation — same checker logic invoked from BOTH a Rust-free Python CI surface and, where used authoring-side, the plain script call documented in the plan (CI/authoring invocation leg).

NO-GO: emitting from any background loop, FSM transition, or Svelte invoke. Those are decorative paths; the only legal emitters are the two production seams above, both named here with the entrypoint dispatching them.

N-1 headline sink: headline routes through the EXISTING spine discipline (`measurement_spine.rs` two-row registry + `BenchIndex` dark-route semantics preserved). Add a governance spine row ONLY once the checker has its real CLI caller wired (slice 4, gated). Numerator limited to genuinely live legs (liveness count + mutant-killed gates + trace-valid rows); full "N% machine-enforced" claim stays deferred until K-22 lands.

## RESPECT sets honored by every slice

(Terminology note: no canonical RESPECT-set registry exists in `.claude/` or the conductor docs as of 2026-08-27 — sets below are defined operationally from the queue-reason constraints — preserve shipped B-17/H-27/N-1 surfaces, fail closed for dark K-22/J-46, join J-70's existing trace fields without rebuilding them; the opus controller may substitute authoritative rows.)

- B-17 set: `tests/gate_liveness.rs`, `scripts/check_gate_liveness.sh` (+ sibling `.bat`, per recon — not re-verified this session), `.elai_cc/.github/workflows/gate-liveness.yml` (current-main location; recon cited it root-relative), gate-fn definitions/callers — consume outcomes, never edit or reroute.
- H-27 set: `orchestrator/src/gate_self_validation.rs` internals, fixed registry, `GateSelfValidation` SSB action, `gate-selfcheck` CLI, frozen row at manifest ~:133 — reuse `validate_gates_with` scoring verbatim; admission checks live in the freeze checker, not inside H-27.
- N-1 set: `measurement_spine.rs` registry shape + `BenchIndex` dispatch semantics; only additive row(s), dark-route behavior intact.
- Frozen-tool set: `.claude/gate-manifest.txt` + `gate_freeze_check.py` hash/lock/trace validation may be EXTENDED additively (new optional-to-required fields for NEW rows) but never weakened; malformed/missing evidence exits nonzero. NOTE (R-3): the executing agent must confirm whether manifest/checker files count as gate-edits for its own TaskSpec BEFORE touching them; this doc treats them as legitimate edit targets only for the future EXECUTE changeset, planned additive-first.
- UI set: entire dashboard surface (stub.ts incl. unrelated I-33 `get_governance_metrics` ~:5252, `tryInvoke` ~:281, gate APIs ~:1196) — zero touches.

## Slice plan (each slice ships producer+consumer+smoke through the real entrypoint; backend-only, UI hold respected)

- SLICE 0 (ground): re-grep current main; pin exact line numbers for every seam above (drop stale ones); record grep outputs in the verify artifact. Output: seam-truth section of the handoff; no product code.
- SLICE 1 (substrate type + fail-closed arms): define view-kind data (GateLiveness/Traceability/MutantKill enums + UNMEASURED marker), build checker core joining V1/V4/V5 live legs, V2/V3 hard-coded fail-closed. Smoke: checker on current tree returns UNMEASURED-only for DoctrineEnforcer/DebtRate and never invents numbers.
- SLICE 2 (SSB emission + CLI production caller): register `GovernanceCoverageMeasured` across ALL lockstep sites; wire `elai governance-coverage` dispatch in cli/main.rs beside BenchIndex; negative test: unregistered/decorative rule reported covered MUST FAIL; smoke drives the real binary, expecting emission visible through the standard SSB journal read path.
- SLICE 3 (freeze-lifecycle admission): extend `gate_freeze_check.py` — new-gate rows require corpus ref + measured kill rate else refusal pre-lock; keep `--trace` DO-333 validation and hash integrity untouched; smoke: refusal case (new row w/o evidence ⇒ nonzero exit) + acceptance case (row with valid evidence passes) through the actual script invocation, both from CLI.
- SLICE 4 (N-1 sink row, gated): add governance spine row + wire headline recomputation through spine conventions; preserve dark-route semantics. Deferred-with-the-unit: K-22/J-46 enrichment, ~40 legacy-gate corpus backfill (Tier-2), G-5/M-3 tile (owner hold).

VERIFICATION artifact per slice = the smoke command + observed stdout captured in the wiring report (off-agent verifier re-runs them). WIRE-DARK is NOT applicable: every symbol listed has the hook above wired in-slice, or is DESIGNED-DARK-VIEW (V2/V3) reported UNMEASURED — which the plan's NEGATIVE observable_effect explicitly mandates.

EXPECTED_OBSERVATION (for the eventual rewrite): running the wired CLI on current main prints GateLiveness>0, MutantKill derived from H-27 outcomes of the fixed registry, Traceability=71-row join, DoctrineEnforcer=UNMEASURED, DebtRate=UNMEASURED; and `python3 .claude/tools/gate_freeze_check.py` refuses a fabricated new-gate row lacking corpus/kill evidence while accepting one carrying it.

VERIFY REDTEAM ABORT (contract carriers — expanded in the rewrite):
- VERIFY: slices 2–3 smokes off-agent rerun (`elai governance-coverage --json`, freeze-refusal case), python_wire_check on changed .py, production_caller_check on changed .rs — GREEN is necessary, off-agent pass artifact sufficient.
- REDTEAM attacks to record: (a) fake-coverage attack — a JSON patch marks an unenforced rule covered ⇒ checker must fail closed; (b) bypass attack — a new manifest row added directly to gate-manifest.txt without freeze invocation ⇒ next `--json` run flags added_unbaselined (already enforced by hash diff) AND --trace still passes only if fields carry evidence (admission extension must not let pre-existing rows escape the new requirement retroactively — grandfather rule needed in rewrite).
- ABORT: checksum/hash instability of frozen files during execution; UI-hold breach (any .svelte/Tauri change appears in diff); K-22/J-46 land mid-flight (fork to enrich, not abort); depot cargo shim failure ⇒ STOP/BLOCKED, never local-toolchain retry (build law).

## Seam verification log (worker-run, 2026-08-27, tree main @ b9efaf5e)

All commands `cd /home/user/teikoku/elai` (clean status; reads == HEAD): (1) `grep -rn "governance_coverage_measured|GovernanceCoverageMeasured"` over `.elai_cc/crates .claude/tools workflow/reasoning/seam-proposals` → hits ONLY in this proposal doc ⇒ N-8 symbols still DARK on current main, verdict preconditions unchanged since recon. (2) `find` for doctrine_coverage/doctrine_debt → only plan docs ⇒ V2/V3 upstreams still absent. (3) `grep -n` anchors: cli/main.rs GateSelfcheck :1368 / run :1374 / BenchIndex :1581 / render_spine_table :1600; action_type.rs enum :3885 / as_sql_str :5747 / from_str :6639 / TOTAL_VARIANTS ladder ~:4882 (now 597→598); gate_freeze_check.py :78/:116/:172/:274 (argparse :277-284); gate-manifest.txt row :133 (296 lines, 71 rows); gate_self_validation.rs REGISTRY :70 / outcome :179 / is_pass :199 / validate_gates :227 / emit_outcome :442; measurement_spine.rs SpineEntry :7 / EXPECTED_SPINE_COUNT=2 :14 / spine_entries :30 / dark_families :35 / render_spine_table :43; gate_liveness.rs 8 #[test], wired region from :156, ignored stubs :348+:442; check_gate_liveness.sh :47 cargo invocation (94-line file); stub.ts get_governance_metrics :5252-5256. (4) Bulk commit 23a3e9ad inspected via --stat: E-plan cluster merge +35 SSB variants (+action_type_lock golden); contains NO N-8/governance-coverage code ⇒ not a supersession signal. (5) `.github/workflows/gate-liveness.yml` absent at repo root → actual path is `.elai_cc/.github/workflows/gate-liveness.yml` (verified present). Unre-verified items are flagged inline where they appear (the `.bat` sibling script).

## Consequence map (why BUILD-WITH-SEAM beats DEFER/SUPERSEDED)

DEFER rejected: DEFER would strand the only currently-realizable legs (V1/V4/V5) behind a permanently-dark dependency; the plan tiers exactly this way (SHIP NOW vs DEFERRED residue) and its NEGATIVE observable_effect makes fail-closed joining safe today.
SUPERSEDED-BY rejected: N-16 extends N-8 (three more views) and composes AFTER the substrate exists; nothing else claims this ground. K-22/J-46/J-70 are consumers/producers of specific arms, not replacements for the substrate.

## Verdict

Each live leg has a named, current-main production hook (CLI dispatch in `.elai_cc/crates/cli/src/main.rs` — GateSelfcheck arm :1368, BenchIndex :1581; SSB action_type.rs registration sites :3885/:5747/:6639; `.claude/tools/gate_freeze_check.py::run()` :172 / `main()` argparse :274-284), the dark legs have mandated fail-closed arms, the UI hold is structurally satisfied (slices 0–4 touch no frontend file), and RESPECT sets are all additive-safe.

VERDICT: BUILD-WITH-SEAM
