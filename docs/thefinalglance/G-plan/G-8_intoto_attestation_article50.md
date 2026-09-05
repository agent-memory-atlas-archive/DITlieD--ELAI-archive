# G-8: Signed in-toto provenance attestation + Article-50 verifier CLI

STATUS: PLANNED (WAVE 2; depends G-2 for the shared evidence manifest; extends F-4 anti-fabrication off-machine)

Objective: emit a custom in-toto v1 predicate (subject bound by blake3 digest; carrying the model id,
ELAI_SEED, an SLSA-VSA-style PASSED/verifiedLevels rollup of every F-gate verdict, the evidence hashes, and the
SSB event id) to a git commit TRAILER (digest only) + a sidecar file + a new SSB action_type, signed via a LOCAL
blake3-Merkle proof (offline default, no new dependency) or an opt-in cosign keyless path (a NEW,
anti-slopsquatting-verified, feature-gated dependency). A new `elai verify-attestation <commit>` re-checks the
attestation OFFLINE and prints verified / not-verified WITHOUT re-running ELAI. POSITION AS "designed to support
Article-50 transparency marking," NOT "certified compliant" (the final GPAI Code of Practice lands ~June 2026;
source code is NOT an enumerated Art-50 example). Penalty exposure is the Art-50 transparency tier EUR 15M / 3%
(Art-99), NEVER the Art-5 prohibited-practices EUR 35M / 7%. Estimated effort: ~L (1-2 sessions). In-process /
offline default; the cosign path is opt-in and gated.

Pre-requisites: the in-code git-commit path crates/tools/src/git_commit.rs SHIPPED (grep-confirmed 2026-06-02:
run_git_commit:277, format_commit_message:184, create_commit:232, latest_commit_hash:237). The three F-gate
verdict payloads SHIPPED (VibeGateBlockedPayload.source_blake3 ssb.rs:25054/:25064 F-1; TestOutcomePayload.stdout_hash
ssb.rs:9877/:9887 F-4; WriteTimeSlopsquatBlockedPayload.dep_name_hash ssb.rs:25315/:25317 F-9). blake3 = "1.8"
workspace dep (Cargo.toml:65). The DepCheck CLI subcommand precedent SHIPPED (args.rs CliAction:75, DepCheck variant
:1035, parse :1997, main.rs dispatch :658 -> dep_check_cmd::run). G-8-corrected 2026-06-04 (see G-8-decisions.md): G-2
RRA evidence manifest is SHIPPED (rra.rs landed this run; G-8 REUSES RunArtifact/GateVerdictHash/extract_gate_manifest,
re-implements no F-gate). cosign/sigstore/in-toto/rekor remain ABSENT in-tree (cosign-attest is a marker feature [] with
no dep added; opt-in keyless path DEFERRED, anti-slopsquatting-gated before any future import).
Parallel-safe with: G-3, G-4, G-9 (disjoint files). NOT parallel-safe with G-2 (shares the evidence manifest) or any
plan editing git_commit.rs / ssb.rs write path in the same window.
Source: workflow/reasoning/ELAI-TRUST-MAXIMIZATION-2026-06-02.md R8. Doctrine: G-0_INDEX.md (W-1..W-10 + GD-1..GD-5).

## Context

Per the trust-max synthesis R8: F-4 binds a claimed test outcome to an orchestrator-captured artifact INSIDE a
run (the anti-fabrication gate). G-8 extends that trust OFF the machine: a buyer's security team should be able
to take a single committed artifact and verify, offline and without re-running ELAI, that the run passed every
F-gate. The vehicle is a custom in-toto Statement v1 predicate — the spec grants custom-predicate freedom, so the
predicate can digest ELAI's three F-gate blake3 hashes (F-1 source_blake3, F-4 stdout_hash, F-9 dep_name_hash) into
an SLSA-VSA-style verifiedLevels rollup ("trust without re-running"). The predicate's subject is bound by blake3
digest (over the staged tree/files, NOT the commit SHA — a digest-only trailer that references its own commit SHA
is a chicken-and-egg), emitted to a git commit trailer (digest only) + a sidecar file + a new SSB action_type.
Signing is LOCAL blake3-Merkle by default (offline, no new dep) with an opt-in cosign keyless path (Sigstore
Rekor-v2 is GA). `elai verify-attestation <commit>` re-checks offline.

The compliance framing is constrained (carried landmines, binding): position G-8 as "designed to support
Article-50 transparency marking," NOT "certified compliant" — the final GPAI Code of Practice / harmonised
standards land ~June 2026, and source code is not an enumerated Art-50 example. The penalty exposure cited MUST be
the Art-50 transparency-obligation tier EUR 15M / 3% (Art-99), NEVER the Art-5 prohibited-practices tier EUR 35M /
7%. These belong to different articles; pairing 35M/7% with a marking argument is the carried error.

ELAI ground truth (grep-confirmed 2026-06-02): the in-code git-commit path is crates/tools/src/git_commit.rs:
run_git_commit (:277) builds the message via format_commit_message (:184), executes `git commit -m` through
create_commit (:232-233), and reads the SHA via latest_commit_hash (:238); the ONLY existing trailer logic is the
NEGATIVE strip-filter at git_commit.rs:217-228 (removes Co-Authored-By/generated-with/noreply trailers per
.claude/rules/commit-cadence.md), so a provenance-digest trailer is GREENFIELD and must be appended post-filter or
allow-listed (there is no E-41 trailer-WRITER to extend). Tool dispatch routes through
crates/tools/src/adaptive_tool.rs::dispatch_git_commit (:270/:483). The F-4 beacon is TestIntegrityEventPayload
(ssb.rs:25170, SsbActionType arm :8266, 2-arm TestIntegrityEventKind Verified/FabricationBlocked at :9787-9788, the
50th single-variant Pattern-2). The three F-gate verdict payloads the verifiedLevels rollup reads are
VibeGateBlockedPayload.source_blake3 (ssb.rs:25054/:25064/:13168, F-1 v196), TestOutcomePayload.stdout_hash
(:9877/:9887, F-4 v197), WriteTimeSlopsquatBlockedPayload.dep_name_hash (ssb.rs:25315/:25317/:13183, F-9 v199). blake3
is a confirmed workspace dep (Cargo.toml:65 = "1.8"; orchestrator/Cargo.toml:82 = workspace=true). cosign / sigstore
/ rekor / in-toto are ABSENT from every Cargo.toml and .rs (only a README.md:99 roadmap bullet). The CLI surface is
CliAction (args.rs:75) parsed by the subcommand string-matcher (dep-check precedent args.rs:1997) and dispatched in
main.rs (88 arms; DepCheck dispatch main.rs:658 -> dep_check_cmd::run); a new verify-attestation mirrors
crates/cli/src/dep_check_cmd.rs. The SSB precedent is E-37 ProvenanceTagged (SsbActionType ssb.rs:6652, payload
ProvenanceTaggedPayload:20126, sql 'provenance_tagged' :8887/:9650, golden :4030); the multi-arm Pattern-2 template is
TestIntegrityEvent. The G-2 evidence manifest (rra.rs) is SHIPPED (G-2 landed this run) — REUSE it.
G-8-corrected 2026-06-04: live SCHEMA_VERSION=213 (ssb.rs:2956), TOTAL_VARIANTS=490 (tests/action_type_lock.rs:590). [was 205/482]

## Existing state (B/C/D/E/F coverage)
- git_commit.rs (crates/tools) — SHIPPED. run_git_commit:277, format_commit_message:184 (strip-filter:217-228,
  NEGATIVE only), create_commit:232, latest_commit_hash:237; dispatch adaptive_tool.rs:270/:483. The commit path G-8
  attaches a digest trailer to — there is NO positive trailer-writer today (G-8 ADDS one; GD-1 — it composes with the
  commit path, it does not re-implement git).
- F-4 anti-fabrication: TestOutcomePayload.stdout_hash (ssb.rs:9877/:9887) + TestIntegrityEventPayload (ssb.rs:25170,
  arm :8266, 2-arm Kind :9787) — SHIPPED. G-8 extends anti-fabrication from inside-the-run to off-machine (GD-1).
- The three F-gate verdict payloads for the verifiedLevels rollup: VibeGateBlockedPayload.source_blake3 (ssb.rs:25054/:25064,
  F-1), TestOutcomePayload.stdout_hash (:9887, F-4), WriteTimeSlopsquatBlockedPayload.dep_name_hash (ssb.rs:25315/:25317, F-9)
  — SHIPPED, all blake3-prefixed. The rollup READS them; re-implements none.
- blake3 = "1.8" (Cargo.toml:65, orchestrator:82) — SHIPPED. The local Merkle needs no new dep.
- cosign/sigstore/rekor/in-toto — ABSENT (README.md:99 roadmap bullet only). The opt-in keyless path is a NEW,
  feature-gated, anti-slopsquatting-verified dependency; the offline blake3-Merkle default needs none.
- CLI: CliAction (args.rs:75), DepCheck precedent (variant :1035, parse :1997, dispatch main.rs:658 -> dep_check_cmd::run),
  dep_check_cmd.rs (the subcommand-module template). run_subcommand declared main.rs:63.
- SSB precedent: ProvenanceTagged (ssb.rs:6652, payload:20126, sql:8887/:9650, golden:4030) + TestIntegrityEvent (multi-arm Pattern-2).
- G-2 RRA evidence manifest (rra.rs) — SHIPPED (G-2 landed this run). REUSE RunArtifact/GateVerdictHash/extract_gate_manifest; build_attestation reads RunArtifact.manifest. [G-8-corrected 2026-06-04]
- ssb.rs — SCHEMA 213 (:2956), TOTAL_VARIANTS 490 (tests/action_type_lock.rs:590). [G-8-corrected 2026-06-04; was 205/482]

## Reads (mandatory before coding)
- workflow/reasoning/ELAI-TRUST-MAXIMIZATION-2026-06-02.md R8 + the EU AI Act Art-50 / Art-99 landmines
- docs/strategy/ELAI-2026-POSTURE.md (the canonical posture spec for the EU AI Act timeline — E-38 HARD RULE; refresh if stale)
- F:\.ELAI_workflow\.elai_cc\crates\tools\src\git_commit.rs (full — run_git_commit + format_commit_message strip-filter + create_commit)
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\ssb.rs:6652,9877,20126,25054,25170,25315 (ProvenanceTagged + TestIntegrityEvent + the three F-gate payloads)
- F:\.ELAI_workflow\.elai_cc\crates\cli\src\args.rs:75,1035,1997 + main.rs:63,411,658 + crates/cli/src/dep_check_cmd.rs (the verify-attestation CLI template)
- F:\.ELAI_workflow\.elai_cc\Cargo.toml:65 (blake3 workspace dep) + README.md:99 (the cosign roadmap bullet — NOT a wired dep)
- docs/thefinalglance/G-plan/G-2_reproducible_run_artifact.md (the shared evidence manifest — forward dep)
- CLAUDE.md HARD RULES: blake3, no .unwrap()/.expect() in prod, anti-slopsquatting (cosign/sigstore opt-in), Pattern-2 SSB, SCHEMA baseline+N.

## Tasks (WAVE 2)

1. **Add the attestation builder (~200 LOC in a new crates/orchestrator/src/attestation.rs).** `pub struct
   InTotoPredicate { subject_blake3: String, model_id: String, elai_seed: u64, verified_levels: Vec<GateVerdict>,
   evidence_hashes: Vec<String>, ssb_event_id: i64 }` where `GateVerdict { gate: String /* F-1/F-4/F-9 */, passed: bool,
   verdict_blake3: String }`. `pub fn build_attestation(ssb: &Ssb, session_id, model_id, seed) -> InTotoPredicate`
   reads the session's three F-gate verdict payloads (VibeGate source_blake3 / TestOutcome stdout_hash / WriteTimeSlopsquat
   dep_name_hash) into the VSA-style rollup, subject bound by a blake3 digest over the STAGED tree/files (NOT the commit
   SHA — chicken-and-egg). `pub fn sign_local_merkle(pred: &InTotoPredicate) -> MerkleProof` (offline default, blake3,
   no new dep). NO `.unwrap()`/`.expect()`.

2. **Emit the attestation: trailer + sidecar + SSB (~90 LOC; ~40 LOC delta in git_commit.rs).** On a run-end commit,
   append a `provenance-digest: <blake3>` trailer in format_commit_message — AFTER the strip-filter (git_commit.rs:217-228)
   or by allow-listing it (the filter currently REMOVES attribution trailers, so the digest trailer must survive it);
   write the full predicate to a `workflow/sessions/{id}/attestation.intoto.json` sidecar; emit a
   `provenance_attestation_emitted` SSB row. Default-ON (W-2): kill-switch ELAI_ATTESTATION_DISABLE only. The trailer is
   DIGEST-ONLY (no AI-attribution text — respects .claude/rules/commit-cadence.md: no Co-Authored-By).

3. **Add `elai verify-attestation <commit>` (~140 LOC in a new crates/cli/src/verify_attestation_cmd.rs + CliAction wiring).**
   Mirror dep_check_cmd.rs: read the trailer digest + the sidecar predicate, recompute the local blake3-Merkle root,
   re-check every GateVerdict's verdict_blake3 against the recorded evidence, and print verified / not-verified — OFFLINE,
   WITHOUT re-running ELAI. Add a CliAction::VerifyAttestation variant (mirror DepCheck args.rs:1035), a parse arm
   (mirror :1997), and a main.rs dispatch arm (mirror :658). The opt-in cosign-keyless verify path is feature-gated.

4. **Add the `provenance_attestation_emitted` SSB action_type (~70 LOC in ssb.rs, W-9).** Mirror E-37 ProvenanceTagged
   (ssb.rs:6652/:20126/:8887): `ProvenanceAttestationEmittedPayload { subject_blake3: String, merkle_root: String,
   gate_count: u32, all_passed: bool, model_id: String, seed: u64, signing: String /* local-merkle | cosign-keyless */ }`.
   sql_str + parse + CHECK + round-trip golden + action_type_lock. G-8-corrected 2026-06-04: SHIPPED SCHEMA 213
   (ssb.rs:2956), TOTAL_VARIANTS 490 (tests/action_type_lock.rs:590) + ALL_VARIANTS row + golden regen + all_test_payloads 410. [plan-time estimate was 205->206 / 482->483]

5. **Anti-slopsquatting gate on the opt-in signing dep (~10 LOC config + decision).** The offline blake3-Merkle default
   adds ZERO deps. The cosign-keyless path is a NEW dependency — verify name+repo+version+publish-date+license per the
   anti-slopsquatting protocol BEFORE any import line; keep it feature-gated (`cfg(feature="cosign-keyless")`). NO
   sigstore crate added unconfirmed (stub_ban).

6. **Frontend pair: AttestationDrawer (~180 LOC svelte + ~110 LOC tauri) (W-4).** New
   crates/dashboard-v2/src/lib/drawers/AttestationDrawer.svelte rendering provenance_attestation_emitted rows (subject,
   merkle_root, the per-F-gate verifiedLevels rollup, all_passed, signing method); Tauri `get_attestations`.
   cross_lang_diff.py PASS. DrawerKey 'attestation'.

7. **Decision doc workflow/reasoning/G-8-decisions.md (~110 LOC).** Records: (a) custom in-toto v1 predicate freedom +
   the SLSA-VSA verifiedLevels rollup (trust-without-re-running); (b) the subject digests the STAGED tree, not the commit
   SHA (chicken-and-egg); (c) the trailer is digest-only + survives the strip-filter (no AI attribution); (d) LOCAL
   blake3-Merkle default (no dep) vs opt-in cosign-keyless (feature-gated, anti-slopsquatting); (e) COMPLIANCE FRAMING:
   "designed to support Article-50 marking" NOT "certified compliant"; EUR 15M/3% (Art-99), NOT 35M/7% (Art-5); final
   Code ~June 2026; source code not an enumerated Art-50 example; (f) G-8 extends F-4 off-machine + shares the G-2 manifest.

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/attestation.rs::build_attestation
  - crates/orchestrator/src/attestation.rs::sign_local_merkle
  - crates/orchestrator/src/attestation.rs::InTotoPredicate
  - crates/orchestrator/src/ssb.rs::SsbActionType::ProvenanceAttestationEmitted
  - crates/orchestrator/src/ssb.rs::ProvenanceAttestationEmittedPayload
  - crates/cli/src/verify_attestation_cmd.rs::run
  - crates/dashboard-v2/src-tauri/src/commands_g8.rs::get_attestations
  - crates/dashboard-v2/src/lib/drawers/AttestationDrawer.svelte
consumes_symbols:
  # G-8-corrected 2026-06-04 (see G-8-decisions.md): anchors reconciled to SHIPPED reality.
  - crates/tools/src/git_commit.rs::PROVENANCE_TRAILER_KEY "ELAI-Provenance-Digest" (:15) + format_commit_message (:198) [SHIPPED digest-only trailer, post strip-filter]
  - crates/orchestrator/src/rra.rs::RunArtifact + GateVerdictHash (ssb_event_seq:i64) + extract_gate_manifest (G-2 SHIPPED this run; build_attestation reads RunArtifact.manifest — REUSE, do NOT fork)
  - crates/orchestrator/src/ssb.rs::VibeGateBlockedPayload (F-1 source_blake3) via the RRA manifest rollup
  - crates/orchestrator/src/ssb.rs::TestOutcomePayload (F-4 stdout_hash) via the RRA manifest rollup
  - crates/orchestrator/src/ssb.rs::WriteTimeSlopsquatBlockedPayload (F-9 dep_name_hash) via the RRA manifest rollup
  - crates/orchestrator/src/ssb.rs::SsbActionType::ProvenanceTagged (E-37 Pattern-2 SSB precedent mirrored)
  - crates/cli/src/args.rs::CliAction::VerifyAttestation (verb SHIPPED) + main.rs dispatch
  - Cargo.toml::blake3 (workspace dep for the local Merkle, :65)
reachable_from:
  # G-8-corrected 2026-06-04 (see G-8-decisions.md): real production entrypoint is run_subcommand.rs run_async, NOT runtime_setup.rs.
  - crates/cli/src/run_subcommand.rs::run_async (:79, run-end) -> emit_attestation (:479) -> build_attestation (:598) ->
    write_sidecar attestation.intoto.json + write_payload(ProvenanceAttestationEmitted) (:617/:627)  [PRODUCTION run-exit path, live rra_ssb, project_id inherited]
  - crates/tools/src/git_commit.rs::format_commit_message (:198) -> ELAI-Provenance-Digest trailer (:15, digest-only, survives strip-filter)  [PRODUCTION commit-binding path]
  - crates/cli/src/args.rs::CliAction::VerifyAttestation -> `elai verify-attestation <commit>` -> verify_attestation_cmd::run (:47) -> verify_attestation_against_trailer (:61) -> verified/not-verified (OFFLINE, read-only, no re-run)
  - crates/dashboard-v2/src-tauri/src/commands_g8.rs::get_attestations -> AttestationDrawer.svelte  [dashboard]
stub_ban:
  - NO "certified compliant" claim — the predicate/CLI/doc say "designed to support Article-50 marking"
  - NO EUR 35M/7% paired with the marking argument — use EUR 15M/3% (Art-99); 35M/7% is the Art-5 prohibited tier
  - NO cosign/sigstore dependency added unconfirmed — offline blake3-Merkle is the default; cosign is opt-in + feature-gated + anti-slopsquatting-verified
  - NO subject bound to the commit SHA — the subject digests the STAGED tree/files (chicken-and-egg)
  - NO AI-attribution trailer (Co-Authored-By etc.) — the trailer is a digest-only provenance line
  - NO default-off ENABLE flag (only ELAI_ATTESTATION_DISABLE kill-switch; default ON, W-2)
  - blake3 only (the Merkle root + verdict hashes); no blake2/sha2
  - do NOT cite an existing E-41 trailer-WRITER. G-8-corrected 2026-06-04 (see G-8-decisions.md): crates/orchestrator/src/rra.rs is SHIPPED (G-2 landed this run) — REUSE its RunArtifact/GateVerdictHash/extract_gate_manifest, do NOT fork or re-implement the F-gate rollup
frontend_pair:
  svelte: crates/dashboard-v2/src/lib/drawers/AttestationDrawer.svelte
  tauri: crates/dashboard-v2/src-tauri/src/commands_g8.rs::get_attestations
observable_effect:
  trigger: "cd .elai_cc && cargo run -p elai-cli -- <run a fixture task that commits; then `elai verify-attestation <commit>`; NO env override>"
  expect_ssb: "a provenance_attestation_emitted row with subject_blake3 + merkle_root (blake3) + gate_count + all_passed + signing in the live ssb.db"
  expect_artifact: "the commit carries a digest-only provenance trailer + a workflow/sessions/<id>/attestation.intoto.json
    sidecar; `elai verify-attestation` prints verified WITHOUT re-running ELAI; tampering the sidecar makes it print not-verified;
    AttestationDrawer renders the verifiedLevels rollup"

## DETERMINISTIC_CHECK
```bash
# G-8-corrected 2026-06-04 (see G-8-decisions.md): run-end anchor is run_subcommand.rs::emit_attestation, NOT runtime_setup.rs.
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/G-plan/G-8_intoto_attestation_article50.md
rg "pub fn build_attestation|pub fn sign_local_merkle|pub struct InTotoStatement" .elai_cc/crates/orchestrator/src/attestation.rs
rg "emit_attestation|build_attestation" .elai_cc/crates/cli/src/run_subcommand.rs   # wired into the run-exit path (:479/:598)
rg "ELAI-Provenance-Digest|PROVENANCE_TRAILER_KEY" .elai_cc/crates/tools/src/git_commit.rs   # digest-only commit trailer (:15)
rg "ProvenanceAttestationEmitted|provenance_attestation_emitted" .elai_cc/crates/orchestrator/src/ssb.rs
rg "provenance_attestation_emitted" .elai_cc/crates/orchestrator/tests/action_type_lock.golden
rg -c "ELAI_ATTESTATION_ENABLE" .elai_cc/crates/orchestrator/src/attestation.rs   # MUST be 0 (kill-switch DISABLE only)
rg -ci "certified compliant|35M|7%%" .elai_cc/crates/cli/src/verify_attestation_cmd.rs   # MUST be 0 (designed-to-support + 15M/3% only)
rg -c "cosign|sigstore" .elai_cc/Cargo.toml   # 0 unless feature-gated + anti-slopsquatting-verified
rg -c "blake2|sha2" .elai_cc/crates/orchestrator/src/attestation.rs   # MUST be 0
test -f .elai_cc/crates/dashboard-v2/src/lib/drawers/AttestationDrawer.svelte
python .claude/tools/cross_lang_diff.py
test -f workflow/reasoning/G-8-decisions.md
```

## ACCEPTANCE
- [ ] W-1 done-gate: wiring_contract_check.py PASS (runtime leg runs unconditionally, no opt-in flag per R3); the observable_effect emitted the attestation +
  provenance_attestation_emitted row, and `elai verify-attestation` ran with a real verdict OFFLINE. cargo-green != DONE.
- [ ] W-2 default-ON: a run emits the attestation with NO env override; ELAI_ATTESTATION_DISABLE=1 suppresses; no ENABLE flag.
- [ ] W-3 production entrypoint: the attestation is emitted from the run-end / git_commit path AND verified via the
  `elai verify-attestation` CLI (not test-only).
- [ ] W-6 dashboard E2E: AttestationDrawer renders the verifiedLevels rollup via dashboard-smoker.
- [ ] OFFLINE verify: `elai verify-attestation` prints verified WITHOUT re-running ELAI; tampering the sidecar/trailer
  digest prints not-verified (a contract test asserts both).
- [ ] GD-1 COMPOSES-WITH: the verifiedLevels rollup reads F-1 source_blake3, F-4 stdout_hash, F-9 dep_name_hash for a run
  that exercised them — re-implements NONE.
- [ ] COMPLIANCE: a contract test asserts the predicate/CLI/doc text says "designed to support" (NOT "certified") and
  cites EUR 15M/3% (NOT 35M/7%).
- [ ] Anti-slopsquatting: the offline default adds no dep; the cosign path is feature-gated + a verified-dep row recorded
  before any import; cargo check passes WITHOUT the cosign feature.
- [ ] provenance_attestation_emitted round-trips via the SSB roundtrip contract + action_type_lock golden; SCHEMA bump
  SHIPPED baseline SCHEMA 213, TOTAL_VARIANTS 490. [G-8-corrected 2026-06-04; plan-time estimate was 205->206 / 482->483]
- [ ] cargo check --workspace GREEN; auditor PASS on -p elai-orchestrator -p elai-cli -p elai-tools -p contracts.

## WIRING DONE-GATE (clause by clause)
- W-1: wiring_contract_check (runtime leg unconditional, R3) is ACCEPTANCE 1; build_attestation (produced) is consumed by the run-end/commit
  path + the verify CLI (W-8), reached by the live commit path + `elai verify-attestation` (W-3).
- W-2: only ELAI_ATTESTATION_DISABLE; default ON; observable_effect runs with no override.
- W-3: the run-end/git_commit path + the verify_attestation_cmd CLI; not test-only.
- W-4: provenance_attestation_emitted is human-surfaceable -> AttestationDrawer + get_attestations; cross_lang_diff PASS.
- W-5: observable_effect is a runnable CLI -> a live ssb.db row + a committed trailer + an offline verify verdict; in-process.
- W-6: ACCEPTANCE verifies the rollup THROUGH AttestationDrawer via dashboard-smoker.
- W-7: git_commit.rs (run_git_commit:277, format_commit_message:184, strip-filter:217-228), the three F-gate payloads
  (ssb.rs:25054/:9887/:25315), ProvenanceTagged (ssb.rs:6652/:20126), CliAction/DepCheck (args.rs:75/:1035/:1997,
  main.rs:658), blake3 (Cargo.toml:65) grep-confirmed 2026-06-02; cosign/sigstore/in-toto + rra.rs confirmed ABSENT and
  NOT cited as present; the validator re-greps.
- W-8: build_attestation/sign_local_merkle (produced) called by the commit path + verify CLI (Task 2/3);
  ProvenanceAttestationEmittedPayload emitted (Task 2) + consumed by the Tauri command (Task 6). No produced symbol caller-less.
- W-9: ONE ProvenanceAttestationEmitted action_type (mirror ProvenanceTagged); action_type golden + cross_lang_diff; SHIPPED SCHEMA 213 / TOTAL_VARIANTS 490. [G-8-corrected 2026-06-04; was 205->206 / 482->483]
- W-10: TIERED — the offline blake3-Merkle attestation + offline verify is in-process Tier-1 (the done-gate); the opt-in
  cosign-keyless / Rekor path is network + a NEW dep, SEPARATELY gated (feature flag), never advertised as the default.
- GD-1: COMPOSES-WITH F-4 (extends off-machine) + F-1/F-9 (reads verdict hashes) + G-2 (shares the manifest, forward dep);
  re-implements none. GD-3: a verified attestation proves the run was REPRODUCED/PASSED-the-gates, not that the code is RIGHT.
  GD-5: "designed to support" not "certified"; EUR 15M/3% not 35M/7%; the cosign path carries its honest opt-in state.

## Tests to add
- crates/contracts/tests/g8_attestation_roundtrip_contract.rs (~160 LOC): (a) build_attestation over a seeded fixture
  session -> the verifiedLevels rollup contains the F-1/F-4/F-9 verdict hashes; (b) the local blake3-Merkle root re-computes
  identically; (c) the predicate round-trips (sidecar JSON); (d) provenance_attestation_emitted round-trips.
- crates/contracts/tests/g8_verify_offline_contract.rs (~140 LOC): (a) a clean attestation -> verify-attestation verified
  WITHOUT re-running; (b) a tampered sidecar digest -> not-verified; (c) the compliance text asserts "designed to support" +
  EUR 15M/3% and NOT "certified" / 35M/7%; (d) the digest-only trailer survives the format_commit_message strip-filter.
- crates/contracts/tests/fixtures/g8/ (a seeded run with F-1/F-4/F-9 verdicts + a tampered variant).

## Files touched
- CREATE: crates/orchestrator/src/attestation.rs (~200 LOC)
- CREATE: crates/cli/src/verify_attestation_cmd.rs (~140 LOC)
- CREATE: crates/dashboard-v2/src-tauri/src/commands_g8.rs (~110 LOC)
- CREATE: crates/dashboard-v2/src/lib/drawers/AttestationDrawer.svelte (~180 LOC)
- CREATE: crates/contracts/tests/g8_attestation_roundtrip_contract.rs (~160 LOC)
- CREATE: crates/contracts/tests/g8_verify_offline_contract.rs (~140 LOC)
- CREATE: crates/contracts/tests/fixtures/g8/*.json (fixtures)
- CREATE: workflow/reasoning/G-8-decisions.md (~110 LOC)
- MODIFY: crates/orchestrator/src/lib.rs (~2 LOC: pub mod attestation;)
- MODIFY: crates/tools/src/git_commit.rs (~40 LOC: digest-only provenance trailer, surviving the strip-filter)
- MODIFY: crates/orchestrator/src/ssb.rs (~70 LOC: ProvenanceAttestationEmitted variant + payload + sql_str + parse + CHECK + golden; mirror ProvenanceTagged)
- MODIFY: crates/orchestrator/tests/action_type_lock.rs (TOTAL_VARIANTS 490 SHIPPED + ALL_VARIANTS row) + tests/action_type_lock.golden (~1 row) [G-8-corrected 2026-06-04; was 482->483]
- MODIFY: crates/cli/src/args.rs (~12 LOC: CliAction::VerifyAttestation + parse arm) + main.rs (~4 LOC: dispatch arm + mod)
- MODIFY: crates/dashboard-v2/src/lib/api/types.ts + DrawerHost.svelte (~6 LOC: DrawerKey 'attestation')

## Citations / Grounding
- workflow/reasoning/ELAI-TRUST-MAXIMIZATION-2026-06-02.md R8. in-toto Statement v1 custom-predicate freedom CONFIRMED;
  SLSA VSA "trust without re-running" CONFIRMED; Sigstore Rekor-v2 GA CONFIRMED. EU AI Act Art-50(2) marking + the
  Code-of-Practice cryptographic/logging route CONFIRMED. POSITION AS "designed to support Article-50 marking," NOT
  "certified compliant" (final Code ~June 2026; source code not an enumerated example). Penalty tier EUR 15M/3% (Art-99),
  NOT EUR 35M/7% (Art-5 prohibited practices). Verify the EU AI Act timeline against docs/strategy/ELAI-2026-POSTURE.md
  per the E-38 HARD RULE; do NOT assert the 2026 compliance posture from training memory.
- ELAI ground truth (grep-confirmed 2026-06-02): git_commit.rs run_git_commit:277 / format_commit_message:184 (strip-filter
  :217-228, NEGATIVE only — no positive trailer-writer) / create_commit:232 / latest_commit_hash:237; adaptive_tool.rs
  dispatch_git_commit:270/:483; the three F-gate payloads VibeGateBlockedPayload.source_blake3 (ssb.rs:25054/:25064),
  TestOutcomePayload.stdout_hash (:9877/:9887), WriteTimeSlopsquatBlockedPayload.dep_name_hash (ssb.rs:25315/:25317);
  TestIntegrityEventPayload (ssb.rs:25170, 2-arm Kind :9787); ProvenanceTagged (ssb.rs:6652/:20126, sql:8887/:9650);
  blake3 Cargo.toml:65; cosign/sigstore/in-toto ABSENT (README.md:99 roadmap bullet only); CliAction args.rs:75 / DepCheck
  :1035/:1997 / main.rs:658 / dep_check_cmd.rs. G-8-corrected 2026-06-04: rra.rs SHIPPED (G-2 landed this run — REUSE
  RunArtifact/GateVerdictHash/extract_gate_manifest). Verify CLI SHIPPED at verify_attestation_cmd.rs (CliAction::VerifyAttestation).
  Live SCHEMA_VERSION 213 (ssb.rs:2956), TOTAL_VARIANTS 490 (tests/action_type_lock.rs:590). [was 205/482]
- CLAUDE.md HARD RULES: blake3, no .unwrap()/.expect() in prod, anti-slopsquatting (cosign/sigstore opt-in + verify-before-add),
  Pattern-2 SSB, SCHEMA baseline+N. E-38 posture reference (ELAI-2026-POSTURE.md) is binding for the EU AI Act claim.

## Open questions / risks
- The trailer-vs-strip-filter interaction: format_commit_message (:217-228) currently REMOVES trailers — confirm at
  authoring whether to append the digest trailer AFTER the filter or allow-list it; a digest trailer accidentally stripped
  makes the on-commit attestation silently absent (only the sidecar survives). Record the chosen approach in the decision doc.
- Subject binding: the predicate subject must digest the STAGED tree/files, not the commit SHA (a self-referential SHA is a
  chicken-and-egg). Confirm the staged-tree digest source at authoring.
- The cosign-keyless path needs a verified, feature-gated dependency (cosign/sigstore are absent today). Do NOT add it
  unconfirmed; the offline blake3-Merkle default carries the done-gate, and the keyless path is the gated follow-on (GD-5).
- COMPLIANCE is the highest-risk prose: the EU AI Act Art-50 framing MUST be "designed to support" + EUR 15M/3% (Art-99);
  pairing the EUR 35M/7% Art-5 figure with a marking argument is the carried landmine. Re-source from ELAI-2026-POSTURE.md
  or the primary Art-99 text (E-38 HARD RULE); do not assert from training memory.

## Next
G-8 signs G-2's RRA evidence manifest as the in-toto predicate (the artifact becomes off-machine-verifiable). G-5's cockpit
can show the attestation status per change (verified / not). G-8 + G-2 + G-6 together are the auditable-reproducible spine:
the run is replayable (G-2), the safety number is measured (G-6), and the whole thing is signed + offline-verifiable (G-8) —
the "trust without re-running" lever the synthesis names, positioned to SUPPORT (not claim) Article-50 marking.

End of G-8.
