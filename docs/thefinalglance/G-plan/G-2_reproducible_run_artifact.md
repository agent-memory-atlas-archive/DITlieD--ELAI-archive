# G-2: Reproducible-Run Artifact (RRA) — the trust-spine keystone

STATUS: PLANNED (WAVE 2 keystone; anchors G-5/G-6/G-8/G-9)

Objective: turn ELAI's stated headline "a security control you can audit and reproduce" from PROSE into a
shippable FILE. Emit a portable, blake3-rooted, ELAI_SEED-pinned artifact bundling the SSB action stream +
seed + HardwareClass + system_fingerprint pin + model pin + an evidence manifest of every F-gate's blake3
verdict hash, and a verifier `elai replay --verify` that re-runs the recorded action stream and reports PASS
(action-stream reproduced AND every F-gate verdict hash re-checked) or the first DIVERGENCE. This is the one
trust artifact no competitor ships: claude-replay is viewer-only (no re-exec, no seed, no divergence). HONEST
SCOPE (GD-5): the SSB stores HASHED tool I/O and the cloud lane is token-opaque, so the guarantee is
REPRODUCIBLE-ACTION-STREAM + VERIFY-DIVERGENCE, NOT bit-identical token replay — state this, do not over-claim.
Estimated effort: ~1-2 sessions (~8-12 h). In-process; the optional resume-from-journal half is gated.

Pre-requisites: SSB SHIPPED (crates/orchestrator/src/ssb.rs — WAL append-only, no UPDATE/DELETE per ssb.rs:5;
write_payload + SsbActionType + action_type golden). Determinism SHIPPED (ELAI_SEED + HardwareClass +
system_fingerprint — grep-confirmed present across crates incl. seccodeplt_runner.rs / l3_lock.rs;
canonical module path confirm-at-authoring, W-7). F-gates whose verdict hashes the manifest consumes are
SHIPPED per memory F-plan Tier-1 (F-1 source_blake3, F-4 stdout_hash, F-9 dep_name_hash; F-10 delta_blake3
when it lands). The run entrypoint build_production_hook_dispatcher is SHIPPED (triple-bootstrap:
run_subcommand.rs:358 / task_cmd.rs:428 / dashboard lib.rs:1434, per F-9 WIRING_CONTRACT).
Parallel-safe with: G-1 (consumes G-1's taint_flow rows into the manifest), G-4. NOT parallel-safe with any
plan editing ssb.rs's write path in the same window (coordinate the edit window).
Source: workflow/reasoning/ELAI-TRUST-MAXIMIZATION-2026-06-02.md R2 + §6 (the replay-completeness + cloud-lane
caveats). Doctrine: G-0_INDEX.md (W-1..W-10 + GD-1..GD-5).

## Context

Per the trust-max synthesis R2 and §0 headline: the SSB append-only journal + ELAI_SEED determinism is the
recurring credibility anchor across all 8 axes — it is what turns "trust us" into "replay us." But today it is
INTERNAL plumbing: there is no single portable file a buyer's security team can take off-machine and re-run,
and the RESUME path (recover_from_checkpoint) is a stub. The whitespace is first-in-category: claude-replay is
a VIEWER (renders a transcript) with no re-execution, no seed pin, no divergence detection; event-log-as-
source-of-truth is established design (ESAA arXiv:2602.23193, ActiveGraph arXiv:2605.21997, per the synthesis
verifier — cite as precedent). ELAI already records the stream + hashed tool I/O under WAL append-only
(ssb.rs:1-5) and has the seed/HardwareClass/system_fingerprint primitives; G-2 bundles them + adds the verifier.

THE HONEST GUARANTEE (GD-5, do not overstate): (1) the ACTION STREAM (which tools fired, in what order, with
what input/output HASHES) is reproducible under a pinned ELAI_SEED for the local lane; (2) every F-gate verdict
hash in the manifest is re-checkable offline; (3) for the cloud lane (Anthropic system_fingerprint is opaque)
and for body-reading downstream steps (SSB stores HASHED tool I/O, response bodies may be GC'd), the guarantee
degrades to VERIFY-DIVERGENCE (detect that a re-run diverged at event N) rather than full RE-EXECUTE. A green
replay proves the run was REPRODUCED, NOT that it was CORRECT (DFAH arXiv:2601.15322, GD-3). The RRA states its
own guarantee level per-run.

## Existing state (B/C/D/E/F coverage)
- ssb.rs (crates/orchestrator) — SHIPPED. SQLite WAL, append-only, no UPDATE/DELETE (ssb.rs:5); write_payload;
  SsbActionType + action_type golden lock; hashed tool I/O recorded. The stream G-2 serializes.
- Determinism (ELAI_SEED + HardwareClass + system_fingerprint) — SHIPPED; grep-confirmed present (the canonical
  module is referenced from seccodeplt_runner.rs / l3_lock.rs / project_rule_seeder.rs — confirm the exact path
  at authoring, W-7). The pins G-2 records.
- F-1 (vibegate source_blake3), F-4 (TestOutcomePayload.stdout_hash), F-9 (WriteTimeSlopsquatBlockedPayload
  dep_name_hash), F-10 (delta_blake3) — the verdict hashes the evidence manifest consumes (GD-1 COMPOSES-WITH).
- build_production_hook_dispatcher (runtime_setup.rs) — the sole production wiring path; the run entrypoint G-2
  taps to emit the RRA at run end.
- recover_from_checkpoint — STUB (the RESUME gap; NOT "BSP has no caller" which was REFUTED, task_runner.rs:5018
  is a non-test BspEngine caller). The optional resume-from-journal half builds on closing this stub.
- rmp-serde — in-tree (msgpack, since Phase 0). Compression: use the already-vendored codec; do NOT add zstd
  without confirmation (anti-slopsquatting) — verify in-tree first.

## Reads (mandatory before coding)
- workflow/reasoning/ELAI-TRUST-MAXIMIZATION-2026-06-02.md R2 + §6 (capture-completeness + cloud-lane caveats)
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\ssb.rs:1-60 (the WAL append-only invariant + write_payload)
- the determinism module (locate via grep HardwareClass/system_fingerprint/ELAI_SEED; read the pin API)
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\runtime_setup.rs (build_production_hook_dispatcher — the
  run entrypoint to tap)
- the F-1/F-4/F-9 payload structs in ssb.rs (the verdict-hash fields the manifest reads)
- F:\.ELAI_workflow\.elai_cc\crates\cli\src\main.rs (the CLI dispatch — where `run --emit-rra`, `rra show`,
  `replay --verify` attach)
- CLAUDE.md HARD RULES: blake3 (the RRA root + per-event hashes), no .unwrap()/.expect() in prod, anti-slopsquatting.

## Tasks (WAVE 2)

1. **Add crates/orchestrator/src/rra.rs (~220 LOC) — the artifact type + serializer.** `pub struct RunArtifact {
   schema_version: u32, elai_seed: u64, hardware_class: String, system_fingerprint: String, model_pin: String,
   action_stream: Vec<SsbEventDigest>, evidence_manifest: Vec<GateVerdictHash>, root_blake3: String,
   guarantee_level: GuaranteeLevel }` where `pub enum GuaranteeLevel { ReproducibleTokens, ReproducibleActionStream,
   VerifyDivergence }` (GD-5 honest state — local-seeded vs cloud-opaque). `GateVerdictHash { gate: String /* F-1/
   F-4/F-9/F-10 */, action_type: String, verdict_blake3: String, ssb_event_id: i64 }`. `pub fn build_rra(ssb:
   &Ssb, seed, pins) -> RunArtifact` reads the session's SSB rows, extracts each F-gate verdict hash, computes a
   blake3 Merkle root over (stream || manifest || pins). `pub fn write_rra(art: &RunArtifact, path) -> Result<...>`
   serializes via rmp-serde + the in-tree compression codec. blake3 ONLY. NO .unwrap()/.expect() on the path.

2. **Add the verifier `pub fn verify_rra(path) -> ReplayVerdict` (~140 LOC in rra.rs).** Re-reads the artifact,
   recomputes the blake3 root (tamper check), re-checks every GateVerdictHash against a fresh recompute where the
   inputs are present, and — for the action stream — replays the recorded actions under the pinned ELAI_SEED,
   reporting `ReplayVerdict::{ Verified, Divergence { first_event_id, expected_blake3, got_blake3 }, Tampered }`.
   Per GD-5 the verdict carries the guarantee_level (a VerifyDivergence-level artifact reports divergence, never a
   false "tokens reproduced"). NO fabricated Verified: a cloud-opaque step that cannot be re-executed reports
   VerifyDivergence-checked, not Verified-by-re-exec.

3. **Emit the RRA at run end from the production entrypoint (~50 LOC delta in runtime_setup.rs / the run path).**
   When a run completes (the build_production_hook_dispatcher-wired run), call build_rra + write_rra to
   `workflow/sessions/{id}/run.rra` and emit ONE `run_artifact_emitted` SSB row carrying the root_blake3 +
   guarantee_level + the gate count. Default-ON behind no env override (W-2); kill-switch ELAI_RRA_DISABLE only.

4. **CLI surface (~110 LOC across crates/cli/src/main.rs + a new rra_cmd.rs).** `elai run --emit-rra` (force-emit),
   `elai rra show <path>` (pretty-print the manifest: seed, pins, gate verdicts, guarantee_level, root), `elai
   replay --verify <path>` (run verify_rra, print Verified / Divergence@event / Tampered). These are the W-3
   production entrypoints + the W-5 observable_effect triggers. `--json` per harness convention.

5. **Add the `run_artifact_emitted` SSB action_type (~60 LOC in ssb.rs, W-9).** `SsbActionType::RunArtifactEmitted`
   + `RunArtifactEmittedPayload { root_blake3, guarantee_level: String, gate_count: u32, seed: u64, session_id }`.
   sql_str + parse + CHECK + round-trip golden + action_type_lock. SCHEMA bump named baseline+1.

6. **Frontend pair: RRA viewer panel (~240 LOC svelte + ~120 LOC tauri) (W-4).** New RraViewerPanel.svelte +
   Tauri `get_run_artifact(session_id)` rendering the manifest (seed, pins, per-gate verdict hash, guarantee_level,
   the replay verdict if `replay --verify` was run). cross_lang_diff.py PASS. This is what a reviewer/auditor opens.

7. **(OPTIONAL, gated) Resume-from-journal half.** Close the recover_from_checkpoint stub so a run can RESUME from
   the RRA action stream (not just verify it). This is a SEPARATE gated sub-phase (effort, and it touches the
   live FSM resume path) — NOT required for G-2 DONE. The verify path is the done-gate; resume is the follow-on.

8. **Decision doc workflow/reasoning/G-2-decisions.md (~110 LOC).** Records: (a) the THREE guarantee levels +
   why the cloud lane is VerifyDivergence not ReproducibleTokens (GD-5; system_fingerprint opacity); (b)
   capture-completeness — SSB stores HASHED tool I/O, so body-reading replay is verify-divergence (synthesis §6);
   (c) blake3 Merkle root design; (d) green-replay != correct (DFAH, GD-3); (e) resume-from-journal is the gated
   follow-on (recover_from_checkpoint stub), NOT a BSP-caller gap (REFUTED).

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/rra.rs::RunArtifact
  - crates/orchestrator/src/rra.rs::GuaranteeLevel
  - crates/orchestrator/src/rra.rs::build_rra
  - crates/orchestrator/src/rra.rs::write_rra
  - crates/orchestrator/src/rra.rs::verify_rra
  - crates/orchestrator/src/rra.rs::ReplayVerdict
  - crates/orchestrator/src/ssb.rs::SsbActionType::RunArtifactEmitted
  - crates/orchestrator/src/ssb.rs::RunArtifactEmittedPayload
  - crates/cli/src/rra_cmd.rs::run
  - crates/dashboard-v2/src-tauri/src/commands_g2.rs::get_run_artifact
  - crates/dashboard-v2/src/lib/windows/RraViewerPanel.svelte
consumes_symbols:
  - crates/orchestrator/src/ssb.rs::Ssb (write_payload; the session row reader)
  - crates/orchestrator/src/ssb.rs::TestOutcomePayload (F-4 stdout_hash — verdict hash source)
  - crates/orchestrator/src/ssb.rs::VibeGateBlockedPayload (F-1 source_blake3 — verdict hash source)
  - crates/orchestrator/src/ssb.rs::WriteTimeSlopsquatBlockedPayload (F-9 dep_name_hash — verdict hash source)
  - crates/cli/src/run_subcommand.rs::run_async (the run entrypoint; emit taps run_async EXIT, mirroring --export-ssb)
  - the determinism module (ELAI_SEED + HardwareClass + system_fingerprint pins — confirm path at authoring, W-7)
  - crates/cli/src/main.rs (CLI dispatch)
  # G-2-corrected 2026-06-04 (see G-2-decisions.md DD2): emit at run_async EXIT (run_subcommand.rs:457 ->
  # emit_run_artifact:485 -> build_rra/write_rra -> ssb.write_payload(RunArtifactEmitted):518, using rra_ssb =
  # clone of the live wiring.ssb Arc), NOT build_production_hook_dispatcher (runtime_setup.rs builds the
  # PostToolUse/VibeGate tool-use dispatcher, not the artifact emit). Mirrors --export-ssb flush-at-exit.
reachable_from:
  - crates/cli/src/main.rs -> `elai run --emit-rra` -> build_rra/write_rra -> run.rra + run_artifact_emitted row
  - crates/cli/src/main.rs -> `elai replay --verify <path>` -> verify_rra -> ReplayVerdict printed
  - crates/cli/src/run_subcommand.rs::run_async EXIT -> emit_run_artifact -> build_rra/write_rra ->
    ssb.write_payload(RunArtifactEmitted) row [PRODUCTION] (default-ON; ELAI_RRA_DISABLE kill-switch only)
  - crates/dashboard-v2/src-tauri/src/commands_g2.rs::get_run_artifact -> RraViewerPanel.svelte [dashboard]
stub_ban:
  - "TODO: rra later" / "// replay stub" literals absent
  - NO fabricated Verified: verify_rra MUST return VerifyDivergence (not Verified) for a cloud-opaque /
    body-GC'd step it cannot re-execute (GD-5)
  - NO default-off ENABLE flag (only ELAI_RRA_DISABLE kill-switch; default ON, W-2)
  - blake3 only for the root + per-event hashes (no blake2/sha2)
  - NO new dependency assumed (rmp-serde in-tree; compression codec must be the in-tree one; zstd only with
    explicit user confirmation per anti-slopsquatting)
  - NO .unwrap()/.expect() in rra.rs production paths
frontend_pair:
  svelte: crates/dashboard-v2/src/lib/windows/RraViewerPanel.svelte
  tauri: crates/dashboard-v2/src-tauri/src/commands_g2.rs::get_run_artifact
observable_effect:
  trigger: "cd .elai_cc && cargo run -p elai-cli -- run --emit-rra <fixture-plan> ; then cargo run -p elai-cli -- replay --verify workflow/sessions/<id>/run.rra"
  expect_ssb: "a run_artifact_emitted row with root_blake3 (blake3) + guarantee_level + gate_count in the live ssb.db"
  expect_artifact: "workflow/sessions/<id>/run.rra exists; `replay --verify` prints Verified (action stream
    reproduced + every F-gate verdict hash re-checked) OR a Divergence@event; `rra show` lists seed+pins+gate
    verdicts; tampering one byte of run.rra makes verify_rra return Tampered (blake3 root mismatch)"

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/G-plan/G-2_reproducible_run_artifact.md --run-trigger
rg "pub struct RunArtifact|pub fn build_rra|pub fn verify_rra|pub enum GuaranteeLevel|pub enum ReplayVerdict" .elai_cc/crates/orchestrator/src/rra.rs
rg "RunArtifactEmitted|run_artifact_emitted" .elai_cc/crates/orchestrator/src/ssb.rs
rg "run_artifact_emitted" .elai_cc/crates/orchestrator/tests/action_type_lock.golden
rg "emit_run_artifact|build_rra|write_rra" .elai_cc/crates/cli/src/run_subcommand.rs   # G-2-corrected: emit at run_async exit, NOT runtime_setup.rs
rg "build_rra|verify_rra" .elai_cc/crates/cli/src/rra_cmd.rs   # CLI rra show / replay --verify
rg -c "blake2|sha2" .elai_cc/crates/orchestrator/src/rra.rs   # MUST be 0
rg -c "ELAI_RRA_ENABLE" .elai_cc/crates/orchestrator/src/rra.rs   # MUST be 0 (kill-switch DISABLE only)
test -f .elai_cc/crates/dashboard-v2/src/lib/windows/RraViewerPanel.svelte
python .claude/tools/cross_lang_diff.py
test -f workflow/reasoning/G-2-decisions.md
```

## ACCEPTANCE
- [ ] W-1 done-gate: wiring_contract_check.py --run-trigger PASS; the observable_effect emitted run.rra +
  run_artifact_emitted, and `replay --verify` ran with a real verdict. cargo-green != DONE.
- [ ] W-2 default-ON: a run emits the RRA with NO env override; ELAI_RRA_DISABLE=1 suppresses; no ENABLE flag.
- [ ] W-3 production entrypoint: the RRA is emitted from the run_async exit path (run_subcommand.rs:457
  emit_run_artifact, mirroring --export-ssb; G-2-corrected 2026-06-04, see DD2 — NOT build_production_hook_dispatcher)
  AND from the `elai run --emit-rra` / `replay --verify` CLI (not test-only).
- [ ] W-5/GD-5 HONEST GUARANTEE: `replay --verify` returns Verified for a local-seeded fixture; for a cloud-lane
  fixture it returns VerifyDivergence-checked (NOT a fabricated tokens-Verified); tampering a byte returns Tampered.
- [ ] W-6 dashboard E2E: RraViewerPanel renders the manifest + replay verdict via dashboard-smoker.
- [ ] GD-1 COMPOSES-WITH: the evidence manifest contains the F-1 source_blake3, F-4 stdout_hash, F-9 dep_name_hash
  for a run that exercised those gates — re-implements NONE of them (it reads their payload fields).
- [ ] GD-2 REPRODUCIBLE-METRIC: the blake3 root re-computes byte-identically from the SSB stream under the pinned seed.
- [ ] run_artifact_emitted round-trips + action_type_lock golden; SCHEMA bump named baseline+1.
- [ ] cargo check --workspace GREEN; auditor PASS on -p elai-orchestrator -p elai-cli -p contracts.
- [ ] No new dependency (rmp-serde + in-tree compression + blake3; zstd only with confirmation — anti-slopsquatting).

## WIRING DONE-GATE (clause by clause)
- W-1: wiring_contract_check --run-trigger is ACCEPTANCE 1; build_rra (produced) is consumed by the run-end hook
  + the CLI (W-8), reached by the live run path (W-3).
- W-2: only ELAI_RRA_DISABLE; default ON; observable_effect runs with no override.
- W-3: run_async exit (run_subcommand.rs:457 emit_run_artifact, mirroring --export-ssb; G-2-corrected 2026-06-04
  per DD2 — NOT build_production_hook_dispatcher) + the rra_cmd CLI; not test-only.
- W-4: run_artifact_emitted is human-surfaceable -> RraViewerPanel + get_run_artifact; cross_lang_diff PASS.
- W-5: observable_effect is runnable CLI -> a real .rra file + a live verify verdict; pure in-process, no RunPod.
- W-6: ACCEPTANCE verifies the manifest + verdict THROUGH RraViewerPanel via dashboard-smoker.
- W-7: ssb.rs (WAL append-only :5), run_subcommand.rs::run_async exit emit (G-2-corrected 2026-06-04 per DD2 —
  superseding the stale build_production_hook_dispatcher anchor), F-1/F-4/F-9 payloads
  grep-confirmed; the determinism module path is confirm-at-authoring + the validator re-greps.
- W-8: build_rra/verify_rra (produced) called by the run_async exit hook + CLI (Task 3/4); RunArtifactEmittedPayload emitted
  (Task 3) + consumed by the Tauri command (Task 6). No produced symbol caller-less.
- W-9: ONE RunArtifactEmitted variant through the golden lock; SCHEMA bump named baseline+1.
- W-10: entirely Tier-1 in-process (read journal, hash, serialize, replay-verify) — no network/subprocess/RunPod
  at gate time. The resume-from-journal half (Task 7) is a SEPARATE gated sub-phase, not part of the done-gate.
- GD-1: COMPOSES-WITH F-1/F-4/F-9/F-10 (consumes verdict hashes). GD-3: green-replay != correct, stated in the
  manifest + decision doc. GD-5: the three honest guarantee levels; no fabricated tokens-Verified for the cloud lane.

## Tests to add
- crates/contracts/tests/g2_rra_roundtrip_contract.rs (~150 LOC): build_rra over a seeded fixture SSB -> the
  blake3 root re-computes identically; write_rra/read round-trips; the manifest contains the F-1/F-4/F-9 verdict
  hashes for a run that exercised them.
- crates/contracts/tests/g2_replay_verify_contract.rs (~140 LOC): (a) a clean local-seeded fixture -> Verified;
  (b) a tampered byte -> Tampered (root mismatch); (c) a cloud-opaque fixture step -> VerifyDivergence-checked,
  NOT a fabricated Verified (GD-5); (d) an injected divergence at event N -> Divergence{ first_event_id=N }.
- crates/orchestrator/tests/g2_rra_emit_smoke.rs (~90 LOC): a run through the real run-end hook writes run.rra +
  the run_artifact_emitted row (the W-5 deterministic harness over the REAL path).

## Files touched
- CREATE: crates/orchestrator/src/rra.rs (~360 LOC: RunArtifact + GuaranteeLevel + build_rra/write_rra/verify_rra + ReplayVerdict)
- CREATE: crates/cli/src/rra_cmd.rs (~110 LOC: run --emit-rra / rra show / replay --verify)
- CREATE: crates/dashboard-v2/src-tauri/src/commands_g2.rs (~120 LOC: get_run_artifact)
- CREATE: crates/dashboard-v2/src/lib/windows/RraViewerPanel.svelte (~240 LOC)
- CREATE: crates/contracts/tests/g2_rra_roundtrip_contract.rs (~150 LOC)
- CREATE: crates/contracts/tests/g2_replay_verify_contract.rs (~140 LOC)
- CREATE: crates/orchestrator/tests/g2_rra_emit_smoke.rs (~90 LOC)
- CREATE: workflow/reasoning/G-2-decisions.md (~110 LOC)
- MODIFY: crates/orchestrator/src/ssb.rs (~60 LOC: RunArtifactEmitted variant + payload + sql_str + parse + CHECK + golden)
- MODIFY: crates/orchestrator/src/lib.rs (~2 LOC: pub mod rra;)
- MODIFY: crates/orchestrator/src/runtime_setup.rs (~50 LOC: run-end build_rra hook)
- MODIFY: crates/cli/src/main.rs (~40 LOC: run --emit-rra / rra / replay subcommand dispatch)
- MODIFY: crates/orchestrator/tests/action_type_lock.golden (~1 row)
- MODIFY: crates/dashboard-v2/src/lib/api/types.ts + a window registration (~10 LOC)

## Citations / Grounding
- workflow/reasoning/ELAI-TRUST-MAXIMIZATION-2026-06-02.md R2 + §6 (the keystone; capture-completeness + cloud-lane
  caveats — VERIFIED via the workflow verifier). ESAA arXiv:2602.23193 + ActiveGraph arXiv:2605.21997
  (event-log-as-source-of-truth precedent, CONFIRMED per the verifier); DFAH arXiv:2601.15322 (green-replay !=
  correct, CONFIRMED). claude-replay is viewer-only (no re-exec/seed/divergence) — first-in-category whitespace.
- ELAI ground truth (grep-confirmed 2026-06-02): ssb.rs WAL append-only / no UPDATE/DELETE (ssb.rs:5); ELAI_SEED +
  HardwareClass + system_fingerprint present across crates (canonical determinism module path confirm-at-authoring);
  build_production_hook_dispatcher (the run entrypoint, triple-bootstrap per F-9); F-1/F-4/F-9 verdict-hash payload
  fields. recover_from_checkpoint is a STUB (the resume gap); "BSP has no production caller" is REFUTED
  (task_runner.rs:5018 is a non-test caller) — do not regress this in the decision doc.
- CLAUDE.md HARD RULES: blake3 (root + per-event), no .unwrap()/.expect() in prod, rmp-serde in-tree, anti-slopsquatting.

## Open questions / risks
- CAPTURE-COMPLETENESS (synthesis §6): the SSB stores HASHED tool I/O — confirm at authoring whether enough of the
  tool RESPONSE BODY is retained (vs GC'd spill files) to RE-EXECUTE body-reading downstream steps, vs only
  VERIFY-DIVERGENCE. This BOUNDS the replay guarantee; the GuaranteeLevel enum carries the honest answer per-run.
- CLOUD-LANE (synthesis §6): Anthropic's system_fingerprint is opaque; cloud-lane runs are
  ReproducibleActionStream / VerifyDivergence, never ReproducibleTokens. The RRA must label this, not hide it (GD-5).
- COMPRESSION dep: do NOT add zstd on assumption — verify the in-tree codec first; if zstd is genuinely needed,
  anti-slopsquatting verify + user confirmation (CLAUDE.md).
- The blake3 Merkle root must cover the pins + manifest + stream so a tamper anywhere is detected — the
  roundtrip contract asserts a single-byte tamper flips the root.
- Resume-from-journal (Task 7) is the gated follow-on; do NOT let it block the verify-path done-gate.

## Next
G-2 is the anchor: G-5 (cockpit) reads the RRA evidence manifest per-hunk; G-6 (SAFEDELTA) wraps each benchmark
run in an RRA so the safety-delta is independently replayable; G-8 (attestation) signs the RRA's evidence
manifest as an in-toto predicate; G-9 (redaction) is the RRA's precondition (the artifact must not export
unredacted secrets). The optional resume-from-journal half (recover_from_checkpoint) turns "verify a run" into
"resume a run" — the natural follow-on once the verify path is green.

End of G-2.
