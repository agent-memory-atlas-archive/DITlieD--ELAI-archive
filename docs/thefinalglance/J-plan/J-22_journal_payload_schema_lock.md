# J-22: journal payload-schema lock + writer-isolation contract — close the field-shape drift the golden lock misses

- Track J (pillar-SOTA recheck extensions) | lever PROVABLE | grounding STRONG (RFC 6962 + serde verified; PunkGo real prototype) | nature BUILD (small; the one genuinely-new P0-journal item) | effort S | WAVE 2 | depends none

- Premise: the trusted run journal (single-writer SQLite WAL, append-only, blake3 hash-chained, blake3 sealed RRA bundles, hand-rolled golden-lock schema freeze) is HARDEN-not-replace (verify-p0; SQLite-WAL is the correct foundation, no embedded store justifies migration). Of the report's named additions, FIVE are RIDERS on already-planned work (I-18 Merkle/chain, G-2 RRA, G-8 attestation, I-1 writer isolation) — recorded in the ledger, not re-authored. Exactly ONE is genuinely new and uncovered: the golden lock validates action_type NAMES but NOT payload FIELD shapes, so a payload struct field rename/add/type-change slips through the action_type_lock test undetected. J-22 closes that hole at near-zero cost, plus a writer-isolation contract test that asserts the invariant we already have structurally.

## Steal/build
1. PAYLOAD-SCHEMA LOCK (the genuinely-new item): add #[serde(deny_unknown_fields)] to the SSB payload structs + a snapshot test (insta or the existing golden-row mechanism extended to payload field shape) that locks each payload struct's FIELD SET, not just its action_type name. A field rename/add/type-change then BREAKS the test the same way a renamed action_type breaks the golden lock today. This closes the cross-language drift hole (E-45's ts-rs snapshot catches the TS side; this catches the Rust payload-struct side at the source).
2. WRITER-ISOLATION CONTRACT TEST (PunkGo's insight, cheaply realized): the SSB is already single-writer (the structural property PunkGo's "kernel is sole commit point" argues for). Add a contract test asserting the invariant — that no path other than the single SSB writer commits a row — so a future change that introduces a second writer FAILS loudly. This is the entire delta vs PunkGo (we have the property; we lacked the test).
3. BATCHED-MERKLE RIDER NOTE (NOT built here): record that I-18's per-row chain_prev_hash is O(n)-replay for inclusion proofs; a batched RFC-6962 Merkle checkpoint (root every ~1000 events, Ed25519-signed, stored as a MerkleCheckpoint row) gives O(log n) inclusion + consistency proofs. This is a RIDER on I-18, authored there, not here — J-22 names it so it is not lost.
EXCLUDED: migrating to Protobuf/prost for buf-breaking schema-compat (the report's "strictly superior" path is gated on a Protobuf migration we are NOT doing; the serde deny_unknown_fields + snapshot is the realistic same-benefit-no-migration alternative); Google Trillian (not embeddable, verify-p0 rejected); a separate writer PROCESS (that is I-1's directive-supervisor — for the JOURNAL the delta is the contract test, not a new process); per-event Merkle (~850x slower, batch is the pattern); re-authoring I-18/G-2/G-8.

## Existing ground truth (grep-confirmed; re-grep at edit, W-7/W-9)
- SSB SHARED STATE BOARD [WIRED] single-writer SQLite WAL, append-only, broadcast-after-commit (feature map 1.4; crates/orchestrator/src/ssb.rs) — the single-writer property J-22's contract test locks.
- ACTION_TYPE GOLDEN LOCK [WIRED] 9-tuple lockstep, SsbActionType + SsbPayload + payload struct + all_test_payloads + golden row + TOTAL_VARIANTS (crates/orchestrator/tests/action_type_lock.rs + .golden; C-27/E-45) — validates NAMES; J-22 adds the FIELD-SHAPE leg.
- CROSS-LANGUAGE DRIFT [PARTIAL] ts-rs snapshot + action_type golden lock (E-45) — the TS-side drift catch; J-22 is the Rust payload-struct-side complement.
- I-18 uPIP CHAIN + AI-BOM [PLANNED] per-row chain_prev_hash + 5-layer stack hash + sigstore/Rekor opt-in — the chain work J-22 rides (the Merkle rider lives there).
- G-2 RRA [WIRED] + G-8 ATTESTATION [PARTIAL] — the sealed bundle + attestation J-22 does NOT re-author (the in-toto/SLSA + Rekor-anchor are riders on G-8/I-18 per the ledger).
- live constants: SCHEMA_VERSION 231 (ssb.rs:3084) / TOTAL_VARIANTS 542 (action_type_lock.rs:671), grep 2026-06-12.

## Integration
- crates/orchestrator SSB payload structs: add #[serde(deny_unknown_fields)] + the field-shape snapshot test (~zero runtime LOC; a derive attribute + a test file). Extends the action_type_lock test family.
- crates/contracts/tests/ a writer-isolation contract test asserting single-writer commit (~40 LOC test).
- NO new action_type — J-22 is a test-layer + derive-attribute hardening (zero SSB surface; JC-3 analog). The lock GUARDS the existing 542 variants' payloads; it adds none.
- frontend_pair: none (test-layer); W-6-justified absence (the journal integrity surface is the existing wiring-audit, not a new panel).

## WIRING_CONTRACT
produces_symbols:
- the #[serde(deny_unknown_fields)] attribute on the SSB payload structs (field-set lock)
- the payload field-shape snapshot test (extends action_type_lock; catches field rename/add/type-change)
- crates/contracts/tests/ writer-isolation contract test (asserts single-writer commit)
consumes_symbols:
- the action_type_lock golden-row mechanism (extended to field shape; NOT reauthored)
- the SSB single-writer commit path (asserted; NOT changed)
reachable_from:
- the cargo test / action_type_lock suite the auditor runs on every change (the existing schema-lock gate)
- NOTE: this is a TEST-LAYER gate (Layer-2 static, the wiring-doctrine's necessary-not-sufficient leg); its "entrypoint" is the auditor's lock-test run, not a runtime path — and that is correct for a schema-drift guard (R5: a static leg, the runtime journal write is the backstop it protects)
stub_ban:
- NO name-only schema lock (the field-shape leg is the point; a payload struct change that keeps the action_type name must BREAK the test)
- NO deny_unknown_fields omitted on a payload struct (every SSB payload struct carries it; a new variant without it FAILS the lock-test's own coverage check)
- NO second SSB writer (the contract test FAILS if a second commit path appears)
- NO Protobuf migration smuggled in (serde + snapshot only; the buf-breaking path is explicitly out)
- NO re-authoring of I-18 chain / G-2 RRA / G-8 attestation (riders recorded, built in their own files)
frontend_pair: none (test-layer); W-6-justified absence.
observable_effect:
- trigger: "a deliberate change to an SSB payload struct that RENAMES a field but keeps the action_type name unchanged, run through the auditor's action_type_lock suite"
- expect_ssb: "none (test-layer); the existing 542 variants are unchanged — J-22 adds no row"
- expect_artifact: "the field-shape snapshot test FAILS on the renamed field (today's name-only golden lock PASSES it — that is the exact hole J-22 closes); the writer-isolation contract test PASSES on the current single-writer code. NEGATIVE (R2): a payload struct WITHOUT deny_unknown_fields is caught by the lock-test's coverage self-check, never silently uncovered; introducing a second SSB commit path FAILS the writer-isolation test, not silence; a benign field-order-only change does NOT false-fail (snapshot on the field SET, not source order)."

## Grounding
- RFC 6962 / RFC 9162 (VERIFIED via verify-p0: O(log n) inclusion + consistency proofs; batched Signed Tree Head is the standard pattern, per-event ~850x slower) — grounds the Merkle RIDER note on I-18, not built here.
- PunkGo / punkgo-kernel arXiv:2602.20214 (VERIFIED: real paper "Right to History", real repo github.com/PunkGo/punkgo-kernel, Rust, v0.5.1, 3 stars — EXPERIMENTAL prototype, not a dependency) — its writer-isolation insight is the contract-test delta; ELAI already has the structural property.
- serde deny_unknown_fields + snapshot (VERIFIED via verify-p0: real, known edge cases; the realistic non-Protobuf alternative to buf-breaking; verify-p0 verdict = the SINGLE genuinely-new highest-value addition).
- in-toto/SLSA v1.0 + Rekor offline-staple + RFC-3161 TSA = RIDERS on G-8/I-18 (verify-p0 verdicts; recorded in the ledger Tier-2, not authored here).
- live SSB 231/542 (grep 2026-06-12).

## COMPOSES-WITH
C-27/E-45 golden lock (extends it to field shape), I-18 chain/Merkle (the rider lives there; J-22 is the field-shape complement), G-2 RRA + G-8 attestation (untouched; the standard-predicate riders are theirs), I-1 directive supervisor (the process-level writer isolation; J-22 is the journal-specific contract-test analog). Restates none.

## Open questions / risks
- snapshot tooling: insta is a new dev-dep (anti-slopsquat at edit) OR extend the EXISTING golden-row mechanism to payload field shape with no new dep — prefer the latter (the action_type_lock already snapshots golden rows; widen it to field sets). Decide at edit.
- deny_unknown_fields edge cases: it interacts badly with #[serde(flatten)] and some enum representations (verify-p0 noted known issues) — audit the payload structs for flatten/untagged-enum use before applying blanket.
- the Merkle rider: must actually land in I-18 (this file only names it). If I-18 ships without the batched checkpoint, the O(n)-inclusion-proof gap persists — track it so the rider is not orphaned.

## Next
The field-shape lock is the cheapest integrity win of the whole recheck (a derive attribute + a test). The Merkle/Rekor/in-toto riders are recorded for I-18/G-8; the writer-isolation contract test makes PunkGo's "sole commit point" invariant machine-checked rather than merely true.

End of J-22.
