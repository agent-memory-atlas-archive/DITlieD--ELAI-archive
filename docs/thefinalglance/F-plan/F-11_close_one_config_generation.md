# F-11: close one config-only DGM generation under existing guards (driver-only, source-rewrite deferred)

Objective: wire ELAI's already-shipped-but-inert DGM substrate into ONE closed config-only self-improvement generation — propose -> evaluate (one REAL benchmark, not the synthetic alternating reward) -> insert into the DGM archive -> select the next parent -> mutate — with EVERY admitted candidate routed in series through the four existing guards (blake3 tripwire pre-exec, trap-task cheat-pattern scoring on the LIVE candidate, the Layer-1A `is_security_sensitive` keyword filter, and the Two-Gate promotion gate). The closed loop's missing piece is exactly ONE driver: `config_archive`'s three core methods (`insert_variant`/`select_parent`/`sample_variant`) have zero production callers today (module + contract test + lib.rs re-export only), and everything the driver would call already exists and is tested. F-11 ships that driver wire-dark behind `ELAI_DARWINIAN_LOOP_ENABLE=1` + `cfg(feature = "darwinian-evolve")`, emits SSB, and scopes the mutant to config/template variants ONLY (never agent-source self-rewrite) — which keeps the loop inside the 2510.04399 bounded-capacity requirement and avoids the Layer-3 `human_approved` gate by construction. Acceptance is "loop closed AND a guard caught a trap-task mutant", NOT "score went up".

Estimated effort: 1 session (~6-9 h). EFFORT M — one driver function plus a thin orchestrator wiring call; the guards and archive already exist and are green. The live-evolution multi-generation run remains RUNPOD-PENDING and OUT OF SCOPE (W-10 Tier-2 split).

Pre-requisites: the DGM substrate SHIPPED-INERT (all confirmed in source AT AUTHORING TIME):
- `config_archive.rs` DGM archive — `insert_variant` (:198, score-range validation ONLY), `select_parent` (:274), `select_parent_with` (:283, EntropySource-seeded), `sample_variant` (:395), `ConfigArchive` struct, `DGM_LAMBDA`=10.0 (:35), `DGM_ALPHA_0`=0.5 (:37), `parent_selection_weights` w_i=s_i*h_i (:60). CONFIRMED.
- `darwin_loop_cmd.rs` CLI surface — `dispatch` (:35) wired at `main.rs:874` (`CliAction::DarwinLoop` -> `darwin_loop_cmd::dispatch`, :875); but the loop drives a SYNTHETIC alternating reward (:104) and prints `darwin-loop COMPLETE (RUNPOD-PENDING live evolution loop)` (:127) WITHOUT ever touching `config_archive`. CONFIRMED — this is the exact insertion point.
- Guard trio + Two-Gate, all green: `promotion_gate.rs::two_gate_promote` (:109, `DEFAULT_TAU`=0.05 :42, `K_m` UNCONFIRMED :34, cites arXiv:2510.04399); `trap_tasks.rs` patterns (MarkerRemoval :56, FakeTestLog :60, VerifierCapture :63; `TRAP_WEIGHT_MULTIPLIER`=10.0 :35, `REQUIRED_TRAP_COUNT`=50 :39); `tripwire.rs` blake3 (Blake3Hasher :34, `blake3_event_id` :254); `self_mod_safety.rs::` Layer-3 `human_approved` gate (:248).
- `self_mod_layer1a.rs::is_security_sensitive` (:383) — the auth/credential/password/secret/token/ssh/tls/ssl/private_key/api_key keyword filter, wired into the Layer-1A apply flow at :789/:846, but on a SEPARATE path from `config_archive` (it never fires on the archive admit loop). CONFIRMED — this is killer-risk #1.
- `external_benches.rs` — `get_suite` (:459), `list_suites` (:440), `BenchmarkResult` (:136), `BenchTarget` (:87), `BenchmarkSuite` trait. CONFIRMED — source of the ONE real benchmark eval.
- `runtime_setup.rs:354` — `DefaultLayer1aProcessor::new(rules_db, rules_db_path, ssb)`, the sole production self-mod wiring path. CONFIRMED.

Parallel-safe with: F-8 (SCF symbolic conflict, self-mod path) — F-8 adds a semantic pre-execution review at `apply_proposal`; F-11 adds an archive admit loop driver. Orthogonal IF F-11's `darwinian_admit` SSB lands under its own SCHEMA bump and does NOT edit `self_mod_layer1a.rs::apply_proposal` (F-11 only READS `is_security_sensitive`, it does not modify the apply path). NOT parallel-safe with any plan editing `crates/orchestrator/src/config_archive.rs`, `darwin_loop_cmd.rs`, `promotion_gate.rs`, or `trap_tasks.rs` in the same window.

Source verdict: the integration-explorer verdict of 2026-06-02 (INTEGRATE_NEW_F_PLAN, high confidence). Smallest-increment + killer-risks honored verbatim below; the verdict's two numeric corrections are recorded (the "17 async contract tests" claim is actually 14; the "14-file config_archive grep" is inflated by the UNRELATED Layer-1A `config_archive` SQLite table in `self_mod_layer1a.rs`, which the module docstring lines 10-12 explicitly disambiguates).

## Context

The Darwin Gödel Machine (arXiv:2505.22954, Zhang/Hu/Lu/Lange/Clune) grows an archive of self-modifying coding agents, samples a parent by a sigmoid-weighted score, mutates it, evaluates on a benchmark, and inserts the result back into the archive — an open-ended propose->evaluate->insert->select loop. It improved SWE-bench 20.0->50.0 and Polyglot 14.2->30.7 (VERIFIED arxiv.org/abs/2505.22954). ELAI faithfully ports the DGM parent-selection math: `sigmoid_perf_score` s_i = 1/(1+exp(-lambda*(alpha-alpha_0))) with lambda=10, alpha_0=0.5; `exploration_bonus` h_i = 1/(1+n_i); `parent_selection_weights` w_i = s_i*h_i (config_archive.rs:34-83) — with deceptive-dip ancestor retention in `enforce_cap` (ancestors of the ACTIVE variant survive pruning, docstring :5-8). The bandit math (`darwinian_bai.rs::SuccessiveRejects::log_bar`) exists. The full guard stack is present and tested.

The gap is VERIFIED REAL, not overstated: `config_archive`'s three core methods (`select_parent`/`insert_variant`/`sample_variant`) appear in exactly THREE files repo-wide (`grep` over `**/*.rs`): the module itself, `crates/contracts/tests/config_archive_contract.rs`, and the `lib.rs` re-export (:1578-1581). That is module + contract test + re-export = genuinely ZERO production callers. The closed loop propose->evaluate->insert_variant->select_parent->mutate is missing exactly ONE piece — the driver — and everything it would call already exists and is tested. The 14-file `config_archive` grep that might suggest otherwise is inflated by an UNRELATED Layer-1A SQLite table of the same name in `self_mod_layer1a.rs`; the module docstring (config_archive.rs:10-12) explicitly disambiguates: "Distinct from the Layer 1A `config_archive` table in `rules.db` which tracks applied-proposal history." So the callerless claim survives that confound.

`darwin_loop_cmd.rs` is a CLI surface that exercises the bandit substrate (DiscountedArmPosterior + SuccessiveRejects + trap fixtures) but drives a SYNTHETIC deterministic reward (line 104: `raw = if (arm_idx + gen_idx) % 2 == 0 { 1.0 } else { 0.0 }`) and terminates RUNPOD-PENDING (line 127) without ever calling `config_archive`. It is the exact insertion point: replace the synthetic reward with ONE real benchmark eval, feed the real alpha into `insert_variant`, then `select_parent` to seed the next generation, and route every admit through the guard stack in series.

The whole scope is ONE closed config-only generation under guards. NOT a multi-generation capability-gain run. Per the verdict, claiming a single closed generation shows a positive delta is UNGROUNDED — DGM needed many iterations and hit deceptive dips. v1 scopes acceptance to loop-closure + guard-fire, exactly as the verdict says.

## Existing state (B/C/D/E coverage)

- DGM archive (the admit boundary): `crates/orchestrator/src/config_archive.rs` ships `ConfigArchive` struct + `insert_variant` (:198, validates `score` in [0,1] ONLY — line 199-201, NO `is_security_sensitive` check, NO AutonomousGate routing), `get_variant` (:254), `select_parent` (:274), `select_parent_with` (:283, EntropySource-seeded for determinism), `enforce_cap` (:301, deceptive-dip ancestor retention), `revert` (:348), `sample_variant` (:395), and the DGM formula trio (`sigmoid_perf_score`/`exploration_bonus`/`parent_selection_weights`/`parent_selection_probabilities`, :49-83). DGM source: docstring cites `workflow/papers/dgm-interface.md` + spec phase-4H.txt 4H.3. CALLERLESS in production (3-file grep). F-11's driver is the FIRST production caller.
- DGM CLI surface (E-7): `crates/cli/src/darwin_loop_cmd.rs::dispatch` (:35) + `run` (:66), wired at `crates/cli/src/main.rs:874` (`CliAction::DarwinLoop`) + `args.rs:964` (`DarwinLoop` arg) + `args.rs:2194` (`"darwin-loop"` parser). Drives SYNTHETIC alternating reward (:104), prints `RUNPOD-PENDING` terminus (:127), emits NO SsbActionType (docstring :18-21, E-7 wire-dark default). F-11 replaces the synthetic reward + adds the archive wiring + adds ONE SSB variant.
- Two-Gate promotion (E-7): `crates/orchestrator/src/promotion_gate.rs::two_gate_promote` (:109) + `PromotionConfig` (`tau` default `DEFAULT_TAU`=0.05 :42; `K_m` cap UNCONFIRMED :34, carried-forward bai.rs default per docstring :39). Cites arXiv:2510.04399 §Theorem 2 / §Theorem 11. Gates by behaviour-descriptor margin + capacity cap. SELF-LABELS tau + K_m as UNCONFIRMED-for-ELAI (killer-risk #2). F-11 routes every archive admit through this AFTER the cheat-detection guards.
- Trap-task cheat patterns (E-7): `crates/orchestrator/src/trap_tasks.rs` ships `TrapPattern` enum (CoastRunnersGreenBlock, MarkerRemoval :56, FakeTestLog :60, VerifierCapture :63, MetricOverflow, LegoStackingFlip, UnitTestEdit), `as_str` (:88), `TRAP_WEIGHT_MULTIPLIER`=10.0 (:35), `REQUIRED_TRAP_COUNT`=50 (:39). Detects reward-hacking patterns. CURRENTLY scored on synthetic fixtures decoupled from the candidate (darwin_loop_cmd.rs:86-94 seeds a static pattern array) — killer-risk #3. F-11 threads the REAL candidate through trap scoring.
- Tripwire (D-4): `crates/orchestrator/src/tripwire.rs` ships blake3-derived `TripwireMonitor`/`TripwireFired`/`blake3_event_id` (:254, `Blake3Hasher` :34). Auto-revert on statistical drift. F-11 runs the tripwire pre-exec on each admit candidate.
- Layer-1A security filter (SEPARATE PATH — killer-risk #1): `crates/orchestrator/src/self_mod_layer1a.rs::is_security_sensitive` (:383) is a keyword filter (auth/credential/password/secret/token/ssh/tls/ssl/private_key/api_key) wired into the Layer-1A apply flow at :789 and :846. It lives on the `apply_proposal` path, NOT on the `config_archive` admit path. Wiring the archive loop creates a NEW autonomous mutation path that BYPASSES this filter unless the driver EXPLICITLY calls it. F-11 routes every admit through `is_security_sensitive` in series — this is the wiring gap the verdict flagged that the smallest-increment closes.
- Layer-3 human-approval gate (E-7): `crates/orchestrator/src/self_mod_safety.rs` (:248, `if !change.human_approved { ... }`) refuses a Layer-3 structural change without `human_approved=true`. F-11 scopes the mutant to config/template variants ONLY (NOT agent-source self-rewrite), so it never produces a Layer-3 `StructuralChange` and avoids this gate by construction (per 2510.04399 bounded-capacity requirement).
- One real benchmark eval: `crates/orchestrator/src/external_benches.rs::get_suite` (:459) / `list_suites` (:440) returns `Box<dyn BenchmarkSuite>`; `BenchmarkResult` (:136), `BenchTarget` (:87). F-11 calls ONE suite for the real alpha (replacing the synthetic reward).
- SSB golden lock: `crates/orchestrator/src/ssb.rs::SsbActionType` enum + sql_str/parse/payload/CHECK pattern; `crates/orchestrator/tests/action_type_lock.golden`. Baseline SCHEMA_VERSION 191 per CLAUDE.md. F-11 adds exactly one action_type.
- Dashboard self-mod surface (frontend pair anchor): `crates/dashboard-v2/src/lib/drawers/ReasoningDrawer.svelte` is the live REFLECT/self-mod drawer (already shows TripwireStatusStrip + ArchiveVerifyGauge). F-11's admit events surface through this same drawer + a sibling Tauri command.

## Reads (mandatory before coding)

- `F:\.ELAI_workflow\.elai_cc\crates\cli\src\darwin_loop_cmd.rs` (full — the synthetic-reward loop :96-128 is the insertion point; `dispatch` :35 is cfg-agnostic)
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\config_archive.rs` (full — `insert_variant` :198 validation surface, `select_parent`/`select_parent_with` :274/:283, `sample_variant` :395, the DGM formula trio :49-83, the docstring :10-12 disambiguating the Layer-1A table)
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\self_mod_layer1a.rs:323-400` (`Layer1aProcessor` trait :323, `apply_proposal` decl :336, `is_security_sensitive` :383 — the SEPARATE-path filter the driver must explicitly call); :780-860 (the existing `is_security_sensitive` call sites :789/:846 — mirror the call shape, do NOT modify them)
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\promotion_gate.rs` (full — `two_gate_promote` :109, `PromotionConfig` tau :42 / K_m UNCONFIRMED :34, the 2510.04399 theorem cites)
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\trap_tasks.rs` (full — `TrapPattern` :56-63, the scoring/weight surface :108, `score_*`/fixture-build path the driver threads the REAL candidate through)
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\tripwire.rs:1-270` (`TripwireMonitor`, `TripwireFired`, `blake3_event_id` :254 — the pre-exec check)
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\self_mod_safety.rs:240-260` (the Layer-3 `human_approved` gate :248 — confirm the config-only mutant never reaches it)
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\external_benches.rs:80-180` + :440-470 (`BenchTarget`, `BenchmarkResult`, `get_suite`/`list_suites` — the ONE real benchmark eval)
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\self_mod_types.rs` (`ConfigVariant` shape + `VariantStatus` + `VariantId` — the archive row the driver builds)
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\ssb.rs` (the SsbActionType enum + sql_str + parse arms + payload roundtrip + CHECK constraint — the 5-site pattern)
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\tests\action_type_lock.golden` (W-9 golden lock)
- `F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\src\lib\drawers\ReasoningDrawer.svelte` (the Svelte consumer to extend)
- `F:\.ELAI_workflow\workflow\papers\dgm-interface.md` (the ELAI DGM interface map; section 2.3 formulas the archive ports) — INGEST via paper-ingestor if not already present
- arXiv:2505.22954 (DGM) abstract/archive section — the propose->evaluate->insert->select loop + deceptive-dip caveat
- arXiv:2510.04399 (Two-Gate) §Theorem 2 / §Theorem 11 — the bounded-capacity-iff-PAC requirement the config-only scope respects

## Tasks

### TIER 1 (in-process, no network/subprocess/RunPod at gate time — this is the whole F-11 shippable scope; the live multi-generation evolution run is the DEFERRED Tier-2)

1. **Add `darwinian_admit` driver in `crates/cli/src/darwin_loop_cmd.rs` (~140 LOC delta) — the one closed generation.** Replace the synthetic alternating reward (line 104) with ONE real benchmark eval and wire the archive admit loop. The driver, for the single generation:
   - **(a) propose**: sample a parent from the archive (`ConfigArchive::select_parent_with` :283, EntropySource-seeded for determinism under `ELAI_SEED`) or, on empty archive, seed a root `ConfigVariant`; mutate it as a CONFIG/TEMPLATE variant ONLY (a `config_blob` delta — NOT an agent-source self-rewrite). The mutant produces a new `ConfigVariant`.
   - **(b) evaluate**: run ONE real benchmark via `external_benches::get_suite(name)` -> `BenchmarkResult` to compute the real `alpha` (score in [0,1]); this REPLACES the synthetic `raw` at line 104. No mock number.
   - **(c) guard chain IN SERIES (the safety spine — order matters)**: (i) `tripwire` pre-exec check on the candidate `config_blob` (blake3 fingerprint via `tripwire.rs::blake3_event_id`); (ii) `trap_tasks` cheat-pattern scoring on the LIVE candidate (NOT a static fixture array — thread the real `config_blob`/eval-log through the trap scorer, killer-risk #3); (iii) **`self_mod_layer1a::is_security_sensitive(&candidate_as_json)`** — the SEPARATE-path keyword filter the verdict flagged (killer-risk #1); a candidate whose config touches auth/credential/token/secret keywords is REJECTED before admit; (iv) `promotion_gate::two_gate_promote(&candidate, &descriptor, &archive_entries, &cfg)` with the documented-UNCONFIRMED tau/K_m (killer-risk #2 — the driver MUST NOT claim tau is calibrated).
   - **(d) admit**: ONLY if all four guards pass, `ConfigArchive::insert_variant(&candidate)` (the FIRST production caller); then `select_parent` to seed the NEXT generation's parent (loop-closure proof). If ANY guard rejects, emit the admit-rejected SSB row and do NOT insert.
   - Mutant = config/template variant ONLY, NEVER agent-source self-rewrite (respects 2510.04399 bounded-capacity; avoids the Layer-3 `human_approved` gate by construction — the driver builds a `ConfigVariant`, not a Layer-3 `StructuralChange`).
   - NO `.unwrap()`/`.expect()` in any non-test path; propagate via `Result`/`?`. NO magic numbers — thresholds are named consts. NO synthetic reward survives (the `raw = if % 2` line is removed, not kept as a fallback).

2. **Wire-dark gating + the orchestrator wiring call (~40 LOC across `darwin_loop_cmd.rs` + the call site).** The driver runs ONLY when BOTH `cfg(feature = "darwinian-evolve")` AND `ELAI_DARWINIAN_LOOP_ENABLE=1` are set (consistent with E-7 DECISION-13, darwin_loop_cmd.rs:9). With the feature off, the existing DISABLED message path (dispatch :53-61) is unchanged. This is a wire-dark feature by design (the verdict's smallest_increment point 5) — UNLIKE the F-2/F-8 default-ON doctrine, F-11 is an autonomous self-modification loop and ELAI policy (CLAUDE.md, memory feedback) gates autonomous self-mod behind explicit enablement. The W-2 done-gate is satisfied by running the observable_effect WITH the flag set (the flag is the feature's intended production posture, not a default-off escape hatch — see WIRING DONE-GATE W-2 below for the explicit doctrine deviation + justification).

3. **Add the `darwinian_admit` SSB action_type (~70 LOC across the 5 ssb.rs sites, W-9).** Single new action_type (Pattern-2 single-variant with a 3-arm `DarwinianAdmitKind` discriminator: `Admitted` / `RejectedByGuard` / `LoopClosed`). Touch all 5 sites consistently (E-30/E-31 pattern): (a) `SsbActionType` enum arm, (b) `sql_str`/`as_sql_str` value `"darwinian_admit"`, (c) parse arm, (d) `SsbPayload::DarwinianAdmit(DarwinianAdmitPayload { kind, variant_id, parent_id, alpha, guard_rejected_by: Option<String>, candidate_blake3, seed })`, (e) SQL CHECK constraint + round-trip golden fixture. Named SCHEMA bump: baseline 191 + 1 (the WIRING_CONTRACT names the exact post-bump number at edit time; per CLAUDE.md never hard-code a final number in plan prose — say "baseline + 1"). `candidate_blake3` uses blake3 (NEVER blake2/sha2). Flow through `action_type_lock.golden` (W-9) + `cross_lang_drift/detector.rs`.

4. **Add the ssb_viewer query (~40 LOC in `crates/orchestrator/src/ssb_viewer.rs`).** `pub fn darwinian_admits(&self) -> Result<Vec<DarwinianAdmitRow>, SsbViewerError>` mirroring the existing self-mod viewer query pattern. `DarwinianAdmitRow { seq, created_at, kind, variant_id, parent_id, alpha, guard_rejected_by, candidate_blake3 }`. This is the data the dashboard reads — what makes the signal NOT wire-dark (W-4).

5. **Frontend pair: Tauri command + ReasoningDrawer extension (~60 LOC tauri + ~50 LOC svelte, W-4).** (a) New `#[tauri::command] pub fn get_darwinian_admits(state: State<'_, AppState>) -> CmdResult<Value>` in `crates/dashboard-v2/src-tauri/src/commands.rs` (gated `#[cfg(feature = "self-modification")]` with the `#[cfg(not(...))]` empty-array twin, mirroring `get_applied_proposals`). Reads `viewer.darwinian_admits()`. (b) Extend `crates/dashboard-v2/src/lib/drawers/ReasoningDrawer.svelte`: add a "Darwinian Admits" section (Card) listing admit events with a tone badge (Admitted=info, RejectedByGuard=warn with the guard name, LoopClosed=success), bound via `reasoningData` state + the new command. Add the command to the TS stub (`src/lib/api/stub.ts` `logStub('get_darwinian_admits')`) so `cross_lang_diff.py` sees the Rust<->TS pair. Register the command in the Tauri `invoke_handler` list.

6. **Decision doc at `workflow/reasoning/F-11-decisions.md` (~120 LOC).** Records: (a) scope = ONE config-only generation under guards, mutant = config/template variant ONLY (never agent-source self-rewrite), and WHY (2510.04399 bounded-capacity-iff-PAC; avoids Layer-3 `human_approved` by construction); (b) the SECURITY WIRING GAP the verdict flagged — `is_security_sensitive` is on the Layer-1A `apply_proposal` path, NOT the `config_archive` admit path, so the driver MUST explicitly call it or the loop opens a new autonomous mutation path that bypasses the auth/credential/token filter (killer-risk #1); (c) the guard-chain ORDER (tripwire pre-exec -> trap scoring on LIVE candidate -> is_security_sensitive -> two_gate_promote) and why trap scoring must thread the REAL candidate (killer-risk #3 — synthetic-fixture trap scoring is theatre); (d) tau/K_m are documented-UNCONFIRMED carried-forward defaults, NOT held-out-calibrated; the doc records a follow-up to calibrate on real self-mod history before any tightening (killer-risk #2); (e) acceptance = loop-closure + guard-fire, NOT capability gain (a single closed generation showing a positive delta is UNGROUNDED — DGM needed many iterations + hit deceptive dips; killer-risk #4); (f) GROUNDING STATUS table (DGM math = PORTED + tested; Two-Gate = PORTED + tau/K_m UNCONFIRMED; "loop produces a measured uplift" = UNGROUNDED-until-many-iterations). This is the W-10 tier statement: F-11 is Tier-1 in-process (the ONE benchmark eval runs locally/offline under ELAI_SEED); the multi-generation live evolution run is the DEFERRED Tier-2 (RUNPOD-PENDING).

## WIRING_CONTRACT

produces_symbols:
  - crates/cli/src/darwin_loop_cmd.rs::darwinian_admit
  - crates/orchestrator/src/ssb_viewer.rs::darwinian_admits
  - crates/orchestrator/src/ssb_viewer.rs::DarwinianAdmitRow
  - SsbActionType::DarwinianAdmit
  - SsbPayload::DarwinianAdmit
  - DarwinianAdmitPayload
  - DarwinianAdmitKind
  - crates/dashboard-v2/src-tauri/src/commands.rs::get_darwinian_admits

consumes_symbols:
  - crates/orchestrator/src/config_archive.rs::ConfigArchive::insert_variant  (CONFIRMED config_archive.rs:198, pub async fn, score-range validation only)
  - crates/orchestrator/src/config_archive.rs::ConfigArchive::select_parent  (CONFIRMED config_archive.rs:274, pub async fn)
  - crates/orchestrator/src/config_archive.rs::ConfigArchive::select_parent_with  (CONFIRMED config_archive.rs:283, EntropySource-seeded for ELAI_SEED determinism)
  - crates/orchestrator/src/config_archive.rs::sample_variant  (CONFIRMED config_archive.rs:395, pub fn, EntropySource-generic)
  - crates/orchestrator/src/self_mod_layer1a.rs::is_security_sensitive  (CONFIRMED self_mod_layer1a.rs:383, pub fn — the SEPARATE-path filter the driver MUST call; killer-risk #1)
  - crates/orchestrator/src/promotion_gate.rs::two_gate_promote  (CONFIRMED promotion_gate.rs:109, pub fn; tau DEFAULT_TAU:42, K_m UNCONFIRMED:34)
  - crates/orchestrator/src/trap_tasks.rs::TrapPattern  (CONFIRMED trap_tasks.rs:56-63, MarkerRemoval/FakeTestLog/VerifierCapture)
  - crates/orchestrator/src/trap_tasks.rs::TRAP_WEIGHT_MULTIPLIER  (CONFIRMED trap_tasks.rs:35 = 10.0)
  - crates/orchestrator/src/tripwire.rs::blake3_event_id  (CONFIRMED tripwire.rs:254, blake3 not blake2/sha2)
  - crates/orchestrator/src/external_benches.rs::get_suite  (CONFIRMED external_benches.rs:459, returns Box<dyn BenchmarkSuite>)
  - crates/orchestrator/src/external_benches.rs::BenchmarkResult  (CONFIRMED external_benches.rs:136)
  - crates/orchestrator/src/self_mod_types.rs::ConfigVariant  (CONFIRMED — the archive row shape; re-exported lib.rs)
  - crates/cli/src/darwin_loop_cmd.rs::dispatch  (CONFIRMED darwin_loop_cmd.rs:35, wired at main.rs:874-875 -> the insertion point)
  - crates/dashboard-v2/src/lib/drawers/ReasoningDrawer.svelte  (CONFIRMED Glob/F-8 — the Svelte consumer extended)

reachable_from:
  - PRODUCTION (CLI): `elai darwin-loop --generations 1 ...` -> main.rs:874 `CliAction::DarwinLoop` -> darwin_loop_cmd::dispatch (:35) -> darwinian_admit driver (with ELAI_DARWINIAN_LOOP_ENABLE=1 + feature darwinian-evolve) -> one closed generation -> darwinian_admit SSB row(s)
  - PRODUCTION (guard chain): darwinian_admit -> tripwire::blake3_event_id (pre-exec) -> trap_tasks scoring (live candidate) -> self_mod_layer1a::is_security_sensitive (separate-path filter) -> promotion_gate::two_gate_promote -> config_archive::insert_variant (FIRST production caller) -> config_archive::select_parent (loop-closure)
  - DASHBOARD: ReasoningDrawer "Darwinian Admits" section -> get_darwinian_admits Tauri command -> ssb_viewer.darwinian_admits() -> live ssb.db rows rendered

stub_ban:
  - literal "TODO: wire archive" absent
  - literal "// real reward later" absent
  - the SYNTHETIC alternating reward (darwin_loop_cmd.rs:104 `raw = if (arm_idx + gen_idx) % 2 == 0`) is REMOVED, not retained as a fallback; the driver computes alpha from a REAL benchmark eval (external_benches::get_suite). A hard-coded / mock alpha is BLOCK.
  - the trap-task scoring runs on the LIVE candidate (the real config_blob/eval-log), NOT on a static TrapPattern fixture array decoupled from the candidate (killer-risk #3). A trap score that ignores the candidate is BLOCK.
  - the driver EXPLICITLY calls is_security_sensitive on the candidate before insert_variant (killer-risk #1). An admit path that never references is_security_sensitive is BLOCK (it opens the auth/credential/token bypass the verdict flagged).
  - NO agent-source self-rewrite mutant: the driver builds a ConfigVariant (config/template delta) ONLY, never a Layer-3 StructuralChange; the mutant must NOT reach self_mod_safety::human_approved (verify by construction).
  - NO `.unwrap()`/`.expect()` in darwin_loop_cmd.rs non-test paths.

frontend_pair:
  svelte_consumer: crates/dashboard-v2/src/lib/drawers/ReasoningDrawer.svelte  ("Darwinian Admits" section)
  tauri_command: crates/dashboard-v2/src-tauri/src/commands.rs::get_darwinian_admits
  ts_binding: crates/dashboard-v2/src/lib/api/stub.ts  (logStub('get_darwinian_admits'))

observable_effect:
  trigger: "cd /d F:\\.ELAI_workflow\\.elai_cc && set ELAI_DARWINIAN_LOOP_ENABLE=1&& set ELAI_DARWINIAN_LOOP_MUTANTS_JSON=F:\\.ELAI_workflow\\conductor\\worktrees\\F-11-INTEGRATE-1_fd0039a3\\crates\\cli\\tests\\fixtures\\darwin_loop_smoke_mutants.json&& cargo run --features darwinian-evolve -p elai-cli -- darwin-loop --generations 1 --mutants 3 --seed 42"
  expect_ssb: "darwinian_admit"
  expect_artifact: "the live ssb.db contains >=2 darwinian_admit rows: at least one kind=RejectedByGuard whose guard_rejected_by matches /trap|security|tripwire|two_gate/ (the injected trap-pattern mutant was caught), and exactly one kind=LoopClosed proving select_parent ran after insert_variant; each row's candidate_blake3 matches /^[0-9a-f]{16}/ (blake3 prefix). Verify: cd .elai_cc && cargo run -p elai-cli -- ssb tail --action darwinian_admit emits those rows; the same rows render in the ReasoningDrawer 'Darwinian Admits' section"

## DETERMINISTIC_CHECK

```bash
# W-1 DONE-GATE (the literal line — wiring is a hard, executed done-gate, not cargo-green):
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/F-plan/F-11_close_one_config_generation.md --run-trigger

# 1. The driver exists and replaces the synthetic reward (the synthetic line is GONE)
rg "pub fn darwinian_admit|fn darwinian_admit" .elai_cc/crates/cli/src/darwin_loop_cmd.rs
! rg "raw = if \(arm_idx \+ gen_idx" .elai_cc/crates/cli/src/darwin_loop_cmd.rs   # synthetic reward removed

# 2. The driver calls the FIRST production caller of the archive trio + select_parent loop-closure
rg "insert_variant|select_parent|select_parent_with|sample_variant" .elai_cc/crates/cli/src/darwin_loop_cmd.rs

# 3. The four guards are wired IN SERIES on the admit path (esp. the separate-path security filter)
rg "is_security_sensitive" .elai_cc/crates/cli/src/darwin_loop_cmd.rs    # killer-risk #1 — MUST be present
rg "two_gate_promote|TrapPattern|blake3_event_id|get_suite" .elai_cc/crates/cli/src/darwin_loop_cmd.rs

# 4. The SSB action_type exists at all 5 sites (W-9)
rg "darwinian_admit|DarwinianAdmit|DarwinianAdmitPayload|DarwinianAdmitKind" .elai_cc/crates/orchestrator/src/ssb.rs
rg "darwinian_admit" .elai_cc/crates/orchestrator/tests/action_type_lock.golden

# 5. The viewer query + row type (the data the dashboard reads — anti-wire-dark)
rg "pub fn darwinian_admits|DarwinianAdmitRow" .elai_cc/crates/orchestrator/src/ssb_viewer.rs

# 6. Frontend pair (W-4): Tauri command + Svelte consumer + TS binding
rg "get_darwinian_admits" .elai_cc/crates/dashboard-v2/src-tauri/src/commands.rs
rg "Darwinian Admits|get_darwinian_admits|darwinian" .elai_cc/crates/dashboard-v2/src/lib/drawers/ReasoningDrawer.svelte
rg "get_darwinian_admits" .elai_cc/crates/dashboard-v2/src/lib/api/stub.ts

# 7. Cross-language drift clean (Rust SSB variant <-> TS binding parity)
python .claude/tools/cross_lang_diff.py

# 8. blake3 not blake2/sha2 on the candidate fingerprint
rg "blake3" .elai_cc/crates/cli/src/darwin_loop_cmd.rs
! rg "blake2|sha2::Sha256" .elai_cc/crates/cli/src/darwin_loop_cmd.rs 2>/dev/null

# 9. Decision doc present
test -f workflow/reasoning/F-11-decisions.md

# 10. compile gate (necessary, NOT sufficient per W-1)
cd .elai_cc && cargo check --workspace --features darwinian-evolve
```

## ACCEPTANCE

- [ ] **W-1 wiring done-gate**: `python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/F-plan/F-11_close_one_config_generation.md --run-trigger` returns PASS (exit 0): every produces_symbol resolves as a DEF/SCIP_DEF, every consumes_symbol has a real CALL/REF, the reachable_from anchor is reached over CALL_RESOLVED within depth 8, stub_ban finds zero literals, and the observable_effect trigger RAN and emitted the rows. (cargo-green is necessary, NOT sufficient.)
- [ ] **W-2 enablement verification (DOCTRINE DEVIATION, justified)**: the observable_effect trigger runs with `ELAI_DARWINIAN_LOOP_ENABLE=1` + `--features darwinian-evolve` and emits `darwinian_admit`. UNLIKE F-2/F-8 (default-ON), F-11 is an autonomous self-modification loop; ELAI policy gates autonomous self-mod behind explicit enablement (CLAUDE.md autonomous-mode + E-7 DECISION-13). The flag is the feature's INTENDED PRODUCTION POSTURE (an opt-in autonomous capability), not a default-off escape hatch hiding an unfinished wire. The verdict's smallest_increment point 5 mandates exactly this gating. Recorded as a deliberate W-2 deviation in F-11-decisions.md; the wire is live AND exercised under the flag, which is the substance W-2 protects.
- [ ] **W-3 production entrypoint**: the driver is reached from the `elai darwin-loop` CLI subcommand (main.rs:874 `CliAction::DarwinLoop`) — a real production entrypoint, NOT a `#[cfg(test)]`-only caller.
- [ ] **LOOP CLOSED**: one generation runs propose -> evaluate (real benchmark alpha, NOT synthetic) -> guard-chain -> insert_variant -> select_parent; a `darwinian_admit` row with kind=LoopClosed proves select_parent ran after insert_variant. `config_archive::insert_variant` has its FIRST production caller.
- [ ] **GUARD FIRED (the acceptance, NOT "score went up")**: the injected trap-pattern mutant is caught by a guard; a `darwinian_admit` row with kind=RejectedByGuard + guard_rejected_by names the firing guard. Acceptance is loop-closure + guard-fire, NOT a capability delta (a single-generation positive delta is UNGROUNDED — killer-risk #4).
- [ ] **SECURITY FILTER EXPLICITLY WIRED (killer-risk #1)**: the driver calls `is_security_sensitive` on the candidate before `insert_variant`; a fixture candidate whose config touches an auth/token keyword is REJECTED with guard_rejected_by="security". `rg is_security_sensitive .elai_cc/crates/cli/src/darwin_loop_cmd.rs` returns a real call site. The new autonomous mutation path does NOT bypass the auth/credential/token filter.
- [ ] **TRAP SCORED ON LIVE CANDIDATE (killer-risk #3)**: the trap-task scoring consumes the real candidate (config_blob/eval-log), not a static TrapPattern fixture array; a contract test asserts a candidate carrying a MarkerRemoval/FakeTestLog/VerifierCapture pattern scores as a trap hit while a clean candidate does not.
- [ ] **CONFIG-ONLY MUTANT (no Layer-3 escalation)**: the mutant is a `ConfigVariant` (config/template delta) only; `rg "StructuralChange|human_approved" .elai_cc/crates/cli/src/darwin_loop_cmd.rs` returns nothing — the loop never produces a Layer-3 change and avoids `self_mod_safety::human_approved` by construction (respects 2510.04399 bounded-capacity).
- [ ] **tau/K_m UNCONFIRMED recorded (killer-risk #2)**: the decision doc states tau (0.05) and K_m are carried-forward UNCONFIRMED defaults, NOT held-out-calibrated; no plan prose claims a calibrated promotion threshold; a follow-up to calibrate on >=20 real self-mod proposals is recorded.
- [ ] **W-4 frontend pair + cross_lang_diff**: `get_darwinian_admits` Tauri command + ReasoningDrawer "Darwinian Admits" section + TS stub binding all present; `python .claude/tools/cross_lang_diff.py` passes (no Rust<->TS drift).
- [ ] **W-6 dashboard E2E**: with the dashboard running, after the `darwin-loop --generations 1` run the admit rows APPEAR in the Reasoning drawer "Darwinian Admits" section (verified THROUGH the dashboard by dashboard-smoker, not just a backend row).
- [ ] **W-9 SSB golden lock**: `darwinian_admit` flows through `action_type_lock.golden`; SCHEMA_VERSION bumped baseline 191 + 1 (named exactly in the executed plan); `action_type_lock` test + cross_lang_drift green; round-trip golden fixture for `DarwinianAdmitPayload` passes; `candidate_blake3` is blake3 (not blake2/sha2).
- [ ] **MULTI-GENERATION LIVE RUN DEFERRED (W-10 Tier-2)**: F-11 ships ONE generation only; the live multi-generation evolution loop (population mutation, cross-domain canary, promotion vote across many iterations) stays RUNPOD-PENDING and is recorded as the deferred Tier-2 in the decision doc. No separate live-RunPod checkbox is required for the in-process single generation.
- [ ] `cargo check --workspace --features darwinian-evolve` GREEN; auditor (sole test runner) PASS on `-p elai-cli` + `-p elai-orchestrator` + the new contract tests + the dashboard svelte-check.
- [ ] No new external dependency (blake3 + rusqlite + the external_benches + trap/promotion deps already in tree; anti-slopsquatting per CLAUDE.md — verify before any add).

## Tests to add

- `crates/contracts/tests/darwinian_admit_loop_contract.rs` (~180 LOC): drive `darwinian_admit` for one generation against a temp self_mod.db + ssb.db; assert `insert_variant` is called (archive row appears), `select_parent` is called after (LoopClosed row), and a clean candidate is admitted with no guard rejection. Assert determinism under `ELAI_SEED` (the blake3 candidate fingerprint + select_parent_with EntropySource path are stable across runs).
- `crates/contracts/tests/darwinian_admit_guard_chain_contract.rs` (~160 LOC): one fixture per guard rejection — (a) a trap-pattern mutant (MarkerRemoval) scored on the LIVE candidate is RejectedByGuard guard_rejected_by="trap" (killer-risk #3 regression guard); (b) a candidate whose config touches "token"/"credential" is RejectedByGuard guard_rejected_by="security" via `is_security_sensitive` (killer-risk #1 regression guard); (c) a low-margin candidate is rejected by two_gate_promote. Assert the guard chain runs IN ORDER (tripwire -> trap -> security -> two_gate).
- `crates/contracts/tests/darwinian_admit_ssb_roundtrip_contract.rs` (~70 LOC): `DarwinianAdmitPayload` round-trips through rmp; action_type sql_str == "darwinian_admit"; the 3-arm DarwinianAdmitKind discriminator round-trips; CHECK constraint accepts it.
- `crates/cli/tests/darwin_loop_observable_smoke.rs` (~120 LOC): the W-5 fixture-harness exercising the REAL darwin-loop --generations 1 path end-to-end against a temp ssb.db, with a trap-pattern mutant injected, asserting >=2 rows (a RejectedByGuard + a LoopClosed) + the candidate_blake3 regex. This is the deterministic harness; it is the SAME code path as the live trigger, not a mock.
- `crates/contracts/tests/darwinian_admit_no_self_rewrite_contract.rs` (~60 LOC): assert the driver produces a `ConfigVariant` (config/template delta) and NEVER a Layer-3 `StructuralChange` — the mutant never reaches `self_mod_safety::human_approved`.

## Files touched

- MODIFY: `crates/cli/src/darwin_loop_cmd.rs` (~180 LOC: darwinian_admit driver; remove synthetic reward :104; wire archive admit loop + 4-guard chain; ELAI_DARWINIAN_LOOP_ENABLE gating)
- CREATE: `crates/cli/tests/fixtures/clean_config_mutant.json` + `trap_marker_removal_mutant.json` + `security_token_mutant.json` + `low_margin_mutant.json` (~4 small fixtures)
- CREATE: `workflow/reasoning/F-11-decisions.md` (~120 LOC)
- CREATE: `crates/contracts/tests/darwinian_admit_loop_contract.rs` (~180 LOC)
- CREATE: `crates/contracts/tests/darwinian_admit_guard_chain_contract.rs` (~160 LOC)
- CREATE: `crates/contracts/tests/darwinian_admit_ssb_roundtrip_contract.rs` (~70 LOC)
- CREATE: `crates/contracts/tests/darwinian_admit_no_self_rewrite_contract.rs` (~60 LOC)
- CREATE: `crates/cli/tests/darwin_loop_observable_smoke.rs` (~120 LOC)
- MODIFY: `crates/orchestrator/src/ssb.rs` (~70 LOC: 1 new action_type across the 5 sites + DarwinianAdmitPayload + 3-arm Kind + golden + CHECK)
- MODIFY: `crates/orchestrator/src/ssb_viewer.rs` (~40 LOC: darwinian_admits query + DarwinianAdmitRow)
- MODIFY: `crates/orchestrator/tests/action_type_lock.golden` (~1 LOC: the locked variant)
- MODIFY: `crates/dashboard-v2/src-tauri/src/commands.rs` (~60 LOC: get_darwinian_admits + cfg twin)
- MODIFY: the Tauri builder generate-handler site (`crates/dashboard-v2/src-tauri/src/lib.rs` invoke_handler list) (~1 LOC: register get_darwinian_admits)
- MODIFY: `crates/dashboard-v2/src/lib/drawers/ReasoningDrawer.svelte` (~50 LOC: Darwinian Admits section)
- MODIFY: `crates/dashboard-v2/src/lib/api/stub.ts` (~6 LOC: get_darwinian_admits stub)

## Citations / Grounding

- arXiv:2505.22954 "Darwin Gödel Machine: Open-Ended Evolution of Self-Improving Agents" (Zhang/Hu/Lu/Lange/Clune) — the propose->evaluate->insert->select archive loop + sigmoid-weighted parent selection + deceptive-dip caveat. VERIFIED via WebSearch arxiv.org/abs/2505.22954: SWE-bench 20.0->50.0, Polyglot 14.2->30.7. NOTE: the 20.0->50.0 numbers are DGM's own multi-iteration coding-agent self-rewrite run — they are NOT an F-11 target (F-11 ships ONE config-only generation; claiming an uplift from it is UNGROUNDED — killer-risk #4).
- arXiv:2510.04399 "On The Statistical Limits of Self-Improving Agents" (Wang/Dorchen/Jin) — the Two-Gate guardrail (margin tau + capacity cap K_m), with distribution-free guarantees preserved IFF the policy-reachable model family is uniformly capacity-bounded. VERIFIED via WebSearch arxiv.org/abs/2510.04399 + confirmed in `promotion_gate.rs:3,12,14,34` source comments. This is WHY F-11 scopes the mutant to config/template variants (bounded capacity) and NOT agent-source self-rewrite (unbounded capacity would void the PAC guarantee). GROUNDING STATUS: the theorem is VERIFIED; ELAI's tau=0.05 + K_m are documented-UNCONFIRMED carried-forward defaults (promotion_gate.rs:34,39), NOT held-out-calibrated (killer-risk #2).
- elai.map / source (every consumes_symbol confirmed AT AUTHORING TIME, W-7): config_archive.rs (insert_variant:198 score-only validation, select_parent:274, select_parent_with:283, sample_variant:395, DGM_LAMBDA:35, DGM_ALPHA_0:37, parent_selection_weights:60, docstring:10-12 disambiguating the Layer-1A table), darwin_loop_cmd.rs (dispatch:35, synthetic reward:104, RUNPOD-PENDING terminus:127, wired main.rs:874-875 + args.rs:964/2194), promotion_gate.rs (two_gate_promote:109, DEFAULT_TAU:42, K_m UNCONFIRMED:34, cites 2510.04399), trap_tasks.rs (MarkerRemoval:56/FakeTestLog:60/VerifierCapture:63, TRAP_WEIGHT_MULTIPLIER:35, REQUIRED_TRAP_COUNT:39), tripwire.rs (Blake3Hasher:34, blake3_event_id:254), self_mod_safety.rs (human_approved gate:248), self_mod_layer1a.rs (is_security_sensitive:383 + call sites:789/846, Layer1aProcessor:323, apply_proposal:336/831), external_benches.rs (get_suite:459, list_suites:440, BenchmarkResult:136, BenchTarget:87), runtime_setup.rs (DefaultLayer1aProcessor::new:354), lib.rs (config_archive re-export:1578-1581).
- CALLERLESS PROOF (W-7): `grep "select_parent|insert_variant|sample_variant" **/*.rs` returns exactly 3 files — config_archive.rs (module) + config_archive_contract.rs (contract test) + lib.rs (re-export, :1578-1581). Genuinely zero production callers. F-11's driver is the FIRST.
- CLAUDE.md HARD RULES applied: blake3 (never blake2/sha2) for the candidate fingerprint; no `.unwrap()`/`.expect()` in non-test paths; Pattern-2 single-variant-with-Kind-discriminator (3-arm DarwinianAdmitKind); SCHEMA_VERSION baseline 191 + 1 (named at edit time, never hard-coded final number in prose); no new external dep (all deps in tree — verify before any add).
- VERDICT NUMERIC CORRECTIONS recorded (do NOT regress): (1) the integration verdict's "17 async contract tests" for config_archive is actually 14 (per the verdict's own self-correction) — does not change the gap; (2) the "14-file config_archive grep" is inflated by the UNRELATED Layer-1A SQLite table of the same name in self_mod_layer1a.rs (docstring config_archive.rs:10-12 disambiguates) — the callerless claim survives this confound.

## Open questions / risks

- **SECURITY-FILTER BYPASS is the headline risk (killer-risk #1, VERIFIED REAL)**: `config_archive::insert_variant` validates score-range ONLY (config_archive.rs:199-201) — NO `is_security_sensitive` check, NO AutonomousGate routing. That keyword filter lives in the SEPARATE `self_mod_layer1a.rs` module (:383), on the `apply_proposal` path, not the archive admit path. Wiring the loop creates a NEW autonomous mutation path that BYPASSES the auth/credential/token/secret rejection UNLESS the driver explicitly calls `is_security_sensitive`. MITIGATION: the driver routes every admit through `is_security_sensitive` in series (Task 1c.iii); stub_ban makes an admit path that never references it a BLOCK; a contract test (guard_chain_contract.rs case b) regression-guards it. The verdict's "route through existing guards in series" silently omitted this path — F-11 makes it explicit.
- **tau/K_m are UNCONFIRMED, not calibrated (killer-risk #2)**: `promotion_gate.rs` self-labels tau (0.05, :42) and K_m (:34) as "UNCONFIRMED for ELAI" carried-forward defaults, NOT held-out-calibrated. A miscalibrated tau admits regressions (too low) or starves the archive (too high). MITIGATION: F-11 makes NO calibration claim; the decision doc records a follow-up to calibrate on >=20 real self-mod proposals before any tightening. The "guard theory correctly grounded" framing must NOT overstate calibration.
- **Trap scoring must thread the REAL candidate (killer-risk #3)**: reward-hacking containment is structurally PRE-promotion (stronger than DGM's post-hoc lineage) BUT only if `trap_tasks` scoring runs on the LIVE candidate. `darwin_loop_cmd.rs` currently scores traps on a static `TrapPattern` array (:86-94) decoupled from the candidate — if the driver does not thread the real `config_blob`/eval-log through trap scoring, the cheat-detection is theatre. MITIGATION: Task 1c.ii + the guard_chain_contract.rs case a regression test; stub_ban makes a candidate-ignoring trap score a BLOCK.
- **A single closed generation showing a positive delta is UNGROUNDED (killer-risk #4)**: DGM needed MANY iterations and hit deceptive dips (arXiv:2505.22954). v1 MUST scope acceptance to loop-closure + guard-fire, NOT capability gain. MITIGATION: ACCEPTANCE phrases the success criterion as "LOOP CLOSED + GUARD FIRED", never "score went up"; the decision doc GROUNDING STATUS table marks "uplift" as UNGROUNDED-until-many-iterations.
- **W-2 doctrine deviation (wire-dark by design)**: UNLIKE the F-2/F-8 default-ON doctrine, F-11 is gated behind `ELAI_DARWINIAN_LOOP_ENABLE=1` + `cfg(feature=darwinian-evolve)`. This is a DELIBERATE deviation: an autonomous self-modification loop is exactly the class of feature ELAI policy gates behind explicit enablement (CLAUDE.md autonomous-mode, E-7 DECISION-13). The risk is that a wire-dark feature regresses to never-exercised. MITIGATION: the observable_effect + W-6 dashboard E2E exercise the wire UNDER the flag every run; the deviation is recorded in F-11-decisions.md with the justification, so it is an intentional opt-in posture, not a forgotten default-off escape hatch.
- **DEFERRED (W-10 Tier-2)**: the live multi-generation evolution loop (population mutation, cross-domain canary gate, promotion vote across iterations) stays RUNPOD-PENDING. F-11 ships ONE in-process generation. Do NOT scope-creep the multi-generation run in — it needs a RunPod 2x3090 window + a measured deceptive-dip resilience study, which is a future F-N.

## WIRING DONE-GATE

How F-11 satisfies each doctrine clause (the implementer cannot skip wiring):

- **W-1 (no-wire-dark done-gate)**: the DETERMINISTIC_CHECK opens with the literal `wiring_contract_check.py --run-trigger` line; ACCEPTANCE item 1 makes its PASS a hard gate; the FSM may not reach COMPLETE without it. cargo-green alone does not close F-11.
- **W-2 (no default-off terminal state — DELIBERATE DEVIATION)**: F-11 is an autonomous self-mod loop gated behind `ELAI_DARWINIAN_LOOP_ENABLE=1` + feature `darwinian-evolve`. This is NOT a default-off escape hatch hiding an unfinished wire — it is the INTENDED PRODUCTION POSTURE for an opt-in autonomous capability (CLAUDE.md autonomous-mode + E-7 DECISION-13 + the verdict's smallest_increment point 5 mandate the gating). The W-2 substance (the wire is live AND exercised) is satisfied: the observable_effect + W-6 E2E RUN under the flag every time. The deviation is recorded in F-11-decisions.md. (If a reviewer rejects the deviation, the fallback is default-ON-in-dry-run-mode: the loop runs but insert_variant is a no-op preview unless the flag is set — but the verdict's recommendation is the gated posture.)
- **W-3 (production entrypoint)**: reachable_from is the `elai darwin-loop` CLI subcommand, wired LIVE at main.rs:874 (`CliAction::DarwinLoop`) — a real production entrypoint, not a `#[cfg(test)]` caller.
- **W-4 (frontend pair mandatory)**: `darwinian_admit` is dashboard-surfaceable, so frontend_pair names ReasoningDrawer.svelte + get_darwinian_admits + the TS stub; cross_lang_diff.py must pass. A human SHOULD see admit/reject events, so the W-4 exception does not apply.
- **W-5 (observable_effect is a live trigger)**: the trigger is a real `cargo run --features darwinian-evolve -p elai-cli -- darwin-loop --generations 1` exercising the REAL driver + guard chain + archive; expect_ssb is a live ssb.db row; expect_artifact is an inspectable `ssb tail` output (a RejectedByGuard + a LoopClosed row). The deterministic-fixture smoke (`darwin_loop_observable_smoke.rs`) exercises the SAME production path (not a mock). The single in-process generation IS the live trigger; the multi-generation run is the DEFERRED Tier-2 with no separate checkbox in this F-N.
- **W-6 (dashboard checkpoint)**: ACCEPTANCE W-6 requires the admit rows to APPEAR in the Reasoning drawer "Darwinian Admits" section after the run — verified THROUGH the dashboard by dashboard-smoker.
- **W-7 (no hallucinated integration points)**: every consumes_symbol + reachable_from anchor + MODIFY target was confirmed in source at authoring time (line numbers in Citations). The callerless proof (3-file grep) and the Layer-1A-table confound (docstring :10-12) were both verified. The two arXiv IDs (2505.22954, 2510.04399) were WebSearch-verified to resolve to the claimed papers.
- **W-8 (produces/consumes symmetry)**: `darwinian_admit` driver is invoked in-plan by the `darwin-loop` CLI dispatch (Task 1-2); `darwinian_admits` viewer query is invoked in-plan by get_darwinian_admits (Task 5); `get_darwinian_admits` is invoked in-plan by ReasoningDrawer (Task 5). No produced symbol is left without an in-plan caller.
- **W-9 (SSB golden lock + named SCHEMA bump)**: one new action_type `darwinian_admit` flows through `action_type_lock.golden`; SCHEMA bump named as baseline 191 + 1 (exact post-bump number set at edit time); cross_lang_drift + action_type_lock must be green; Pattern-2 single-variant with a 3-arm DarwinianAdmitKind discriminator (Admitted/RejectedByGuard/LoopClosed).
- **W-10 (tiered-gate rule)**: F-11 is Tier-1 — the ONE benchmark eval + the four-guard chain + the archive admit run in-process under ELAI_SEED (the blake3 fingerprint + select_parent_with EntropySource are deterministic). The multi-generation live evolution loop (the only network/RunPod-adjacent axis) is explicitly DEFERRED out of F-11 (Tier-2, RUNPOD-PENDING), satisfying the rule that no fail-open network axis advertises a fail-closed promise inside this plan.

## Next

The natural composition is the DEFERRED Tier-2: the multi-generation live evolution loop (population mutation across N generations, cross-domain canary gate, promotion vote, deceptive-dip resilience study) on a RunPod 2x3090 window — a future F-N gated on a measured FP/calibration study of tau/K_m from F-11's own admit history. F-8 (SCF symbolic conflict) composes orthogonally: F-8 adds a semantic pre-execution review at `apply_proposal`; F-11 adds the archive admit loop — once both land, a candidate could be routed through F-8's semantic conflict detector AND F-11's guard chain. The tau/K_m calibration follow-up (killer-risk #2) feeds promotion_gate.rs and is a prerequisite before any tightening of the Two-Gate from WARN-margin to a hard capability-gain claim.

End of F-11.
