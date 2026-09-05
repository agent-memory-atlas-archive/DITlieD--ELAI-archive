# J-91: re-executable RRA — a third party re-runs against the recorded world, not just replays the actions (the wedge technical leg)

- Track J (lacuna-fill track) | lever EXTERNAL-FAITHFULNESS | grounding STRONG (live J-14/G-1/G-2/I-45/G-9 grep) | nature BUILD (= a J-14 EXTENSION) | effort L | WAVE 2 | depends J-14 (the ReproPack verify tiers), J-87 (a COMPLETE record), G-1 (the taint-ingested bytes)
- The TECHNICAL companion to K-20 (the K-native re-executable-RRA DIFFERENTIATOR, already authored). K-20 BANKS the wedge claim; J-91 BUILDS the re-execution. This is THE WEDGE — re-executable evidence, not just replayable (reasoning §L3).

- Premise: the RRA is REPLAYABLE (re-run recorded actions, check verdict-hashes) but not RE-EXECUTABLE. It verifies INTERNAL consistency, not EXTERNAL faithfulness; and if recording omitted an effect (the E1/J-87 gap) it faithfully reproduces an INCOMPLETE record. "Reproducible action-stream" today means replayable, not re-executable. A REPLAYABLE record proves "ELAI did these recorded steps"; a RE-EXECUTABLE record proves "a third party, given the recorded external world, gets the same sealed verdict" — the difference that makes the evidence a buyer can TRUST without trusting ELAI.

## Steal/build (report 8 §re-executable + ELAI's planned J-14)
1. CASSETTE — capture the taint-ingested untrusted BYTES (G-1's 4 ingest sites: WebFetch / MCP / file / sandbox) as a CONTENT-KEYED VCR CASSETTE (J-14's `rra export-fixture` shape), so a re-run replays against the RECORDED external responses, not replay-and-trust.
2. PINNED ENV — an OCI digest + a Nix-style lockfile + CAS fixtures (Bazel action-cache shape) so the re-run environment is byte-reproducible.
3. RE-RUN VERIFY — a third party re-EXECUTES the recorded actions against the cassette under the pinned env and checks the I-45-sealed verdict matches (not just the action-hash). This is the J-14 verify-tier EXTENSION.
4. TIERED FIDELITY with HONEST DOWNGRADE (already CL-4/GD-5 doctrine + J-14): T0 action-faithful / T1 token-faithful (only where J-13 batch-invariance + the C7-SIDECAR attention residue allow) / T2 score-faithful. The tier is SELF-DESCRIBING; a path that cannot deliver T1 honestly downgrades to T0 with a visible note.
5. WORLD-DRIFT as a FIDELITY DOWNGRADE, never hidden: a cassette proves the OLD world; surface staleness as a downgrade, not a portal back to the live world.
EXCLUDED: re-running against the LIVE world (re-execute against the CASSETTE; live re-run is not reproducible); claiming T1 where attention is non-batch-invariant (honest T0 downgrade; the C7-SIDECAR residue, J-94); snapshotting non-snapshottable external effects (payments/emails stay action-faithful-only with a visible note); re-authoring G-1/G-2/J-14 machinery (extend it, GD-1).

## Existing ground truth (grep-confirmed; re-grep at edit, W-7/W-9)
- REPRO-PACK VERIFY TIERS J-14 [PLANNED] T0/T1/T2 capability-gated tiers, determinism_applied self-describing bundles, first-divergence locator, `rra export-fixture` content-keyed cassette format for I-49; extends G-2, gated by J-13 for T1/T2. J-91 IS the re-execution extension of J-14.
- TAINT PROPAGATION G-1 [WIRED] Provenance constructed at 4 real ingest sites; the cassette captures THESE bytes.
- REPRODUCIBLE-RUN ARTIFACT G-2 [WIRED] msgpack+zstd bundle (SSB stream + seed + HardwareClass + model pin + F-gate verdict hashes + blake3 root); rra.rs; cloud lane = reproducible ACTION-STREAM.
- OBJECTIVE-HASH SEALED RECEIPTS I-45 [PLANNED] the verdict the re-run must match.
- JOURNAL REDACTION G-9 [DARK] the auth-scrub functions (J-14 reuses them; "verbatim-auth-header = DO-NOT-COPY" culpa).
- K-20 (the K-native differentiator) + J-87 (the completeness substrate the cassette's record depends on).
- live SSB baseline ~268/577 (grep 2026-06-26).

## Integration
- crates/orchestrator rra.rs: a re-execution verify path extending verify_rra (rra.rs:474, the J-14 seam) — load the cassette + pinned env, re-run the recorded actions against recorded external responses, compare the I-45 sealed verdict. `rra re-execute --cassette <path>` CLI verb.
- reuse ReproPackEvent +Kind (CassetteExported / ReExecuted / FidelityDowngraded) — net 0 (J-14's planned event). re-read at edit (W-9).
- FEATURE FLAG: ELAI_REEXECUTE_RRA_ENABLE (kill-switch; the re-execution is opt-in evidence, not a hot-path gate) + a wire-dark-allowlist row with promotion plan (W-2/R9).
- frontend_pair: a re-execution result + fidelity-tier badge on the RRA/reviewer surface (G-5/K-11 evidence render); cross_lang_diff if surfaced (W-4).

## WIRING_CONTRACT
produces_symbols:
- crates/orchestrator/src/rra.rs::re_execute_rra (re-run recorded actions vs the cassette under the pinned env; verdict match)
- crates/orchestrator/src/rra.rs::export_cassette (content-keyed VCR cassette of G-1-ingested bytes; extends export-fixture)
- crates/orchestrator/src/rra.rs::pin_environment (OCI digest + lockfile + CAS fixtures)
- crates/cli re-execute verb (`rra re-execute --cassette`)
consumes_symbols:
- J-14 ReproPack verify tiers + export-fixture (extended, not reauthored)
- G-1 taint-ingested bytes (the cassette content) + G-2 RRA bundle + I-45 sealed verdict (the match target)
- G-9 auth-scrub (the cassette is scrubbed; not reauthored)
- J-87 completeness (the record the cassette re-runs is COMPLETE, else it re-runs an incomplete record honestly)
reachable_from:
- a real `elai run --emit-rra` followed by `rra re-execute` on a DIFFERENT machine/process against the cassette  [PRODUCTION]
- a #[cfg(test)] / contracts/ caller NEVER satisfies this (R7)
stub_ban:
- NO re-run against the LIVE world (cassette only; live is not reproducible)
- NO T1 claim where attention is non-batch-invariant (honest T0 downgrade; CL-4/GD-5)
- NO hidden world-drift (surface as a fidelity downgrade, never silently)
- NO verbatim auth header in the cassette (G-9 scrub; DO-NOT-COPY culpa)
- NO re-authoring of G-1/G-2/J-14 (extend; re-implementing = GD-1 P0)
frontend_pair: re-execution result + fidelity-tier badge (decide at edit) or W-6-justified absence.
observable_effect:
- trigger: "an `elai run --emit-rra` that performs a WebFetch (a G-1 ingest), then `rra re-execute --cassette` on a fresh process against the recorded cassette; AND a re-run where the cassette is stale vs the live world"
- expect_ssb: "CassetteExported with the content-keyed WebFetch bytes; ReExecuted when the fresh-process verdict matches the I-45 seal; FidelityDowngraded (with the tier + reason) when attention non-batch-invariance forces T0 or world-drift is detected"
- expect_artifact: "the fresh process re-runs against the recorded bytes (NOT the live URL) and reproduces the sealed verdict at the declared tier. NEGATIVE (R2): a re-run that reaches the live network instead of the cassette FAILS the gate (it is not reproducible), never passes; a cassette with an unscrubbed auth header FAILS export; a T1 claim on a non-batch-invariant path is REFUSED and downgraded to T0 with a visible note, never silently asserted."

## Grounding
- report 8 §re-executable + §staleness (the cassette + pinned-env + world-drift-as-downgrade). DESIGN-RATIONALE only (LFG-3); the done-gate is the ELAI fresh-process verdict match.
- substrate: J-14 [PLANNED] + G-1/G-2 [WIRED] + I-45 [PLANNED] + G-9 [DARK] + J-13 (batch-invariance) + K-20 (the differentiator) — feature-map 1.4/1.6.
- the wedge framing is K-20's (the K-native dossier); J-91 is its proof-of-execution.
- live SSB baseline ~268/577 (grep 2026-06-26).

## COMPOSES-WITH
J-14 (the extended verify tiers + cassette format), G-1 (the bytes), G-2 (the bundle), I-45 (the sealed verdict), G-9 (the scrub), J-13 + J-94 C7-SIDECAR (the attention-determinism tier bound), J-87 (the complete record), K-20 (the K-native differentiator), K-11 (the human-viewable evidence render), G-6 SAFEDELTA (the measured wedge number this evidence backs). Restates none.

## Open questions / residue (reasoning §7)
- WORLD-DRIFT: a cassette proves the OLD world. Surface it as a fidelity downgrade; never let it read as a re-run against the current world.
- NON-SNAPSHOTTABLE EFFECTS: payments, emails, irreversible external side effects stay ACTION-FAITHFUL-ONLY with a visible note (they cannot be re-executed safely). The compensation registry (J-93 C3-SESSIONDAG) marks these.
- COMPLETENESS DEPENDENCY: re-executing an INCOMPLETE record faithfully reproduces an incomplete record (the E1 gap) — J-91's honesty depends on J-87. State the dependency in-file.

## Next
J-91 is the proof-of-execution behind K-20's wedge claim and feeds G-6 SAFEDELTA's "believe-the-number" envelope; K-11 renders the result human-viewable in the PR. The completeness it re-runs is proved by J-87.

End of J-91.
