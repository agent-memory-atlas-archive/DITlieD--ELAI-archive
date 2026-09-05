# P-21: evidence / gate-credibility hardening (verify-the-verifier + structural redaction)

STATUS: PLANNED (WAVE OC-1; EXTENSION of Q-6 + wiring R10 + G-9 + M; PROOF_SPINE: feeds PS-6; execute after the proof spine per A-7)

- Track OPENCLAW-STEAL | lever prove-the-gate-still-bites | grounding STRONG (verify-the-verifier + structural redaction VERIFIED-IN-SOURCE V1/V2; Q-6 PLAN-asserted, executor re-greps W-7) | nature EXTENSION | effort M | WAVE OC-1 | PROOF_SPINE feeds PS-6
- Premise: openclaw's fail-closed INSTINCTS are real, test-covered, and independently converge on ELAI doctrine — steal them as reference implementations that harden the gate corpus and feed the PS-6 wire-dark/gate-credibility number.

## Context — why this matters
Grounded in workflow/reasoning/openclaw-vs-elai-comparison-2026-07-11.md 2.2 + steals S8/S9/S15 + deep-read V1/V2/V3/N1/N2/C3. VERIFIED-IN-SOURCE: (V1) confidence-self-test builds synthetic KNOWN-BAD artifacts, runs each through the SAME production detectors, and the self-test summary is itself a REQUIRED fail-closed lane — a missed canary drives the WHOLE gate to fail (exitCode=1, confidence-report.ts:1139-1270); (V2) the durable audit record type has NO content field at all (audit-event-types.ts:34-56) + hash-don't-store ids + whitelist-or-"unknown" fields — STRUCTURAL redaction, stronger than a denylist scrubber; (N1) missing-artifact=FAIL with an in-manifest explicit escape (env-blocked or optional-gap only) ~= the WIRE-DARK allowlist; (N2) skipBackfill: a lane with skipped rows clears ONLY if a NAMED sibling passed ~= R10 threaded-required-input. CORRECTIONS: (V3) the trajectory bundle in the SAME repo is CONTENT-carrying — keep the metadata export STRICTLY separate BY TYPE from the debug bundle; (C3) openclaw's default green report is fail-closed on UNKNOWNS ONLY, not on classified failures — copy unknown=FAIL, do NOT copy the default that lets a classified bug pass.

## Existing ground truth (B..Q coverage; PLAN-asserted; executor re-greps W-7)
- Q-6 gate-manifest evidence-corruption corpus + kill-rate (Q-SUMMARY_INDEX.md:51): every frozen checker proven to fail-closed against truncated/corrupt/missing/flipped evidence, a measured per-gate kill-rate — the PS-6-adjacent number this feeds. EXTEND with the verifier-canary lane.
- wiring-enforcement R10 (integration-seam gate): posture labels + skipBackfill compose here.
- G-9 redaction gate (write-time secret/PII redaction): STRUCTURAL redaction hardens it.
- M fleet evidence rollup (org-grain from seat journals, zero telemetry): the metadata-only projection consumer.

## Steal/build (REVISED scope, W-10 tiered)
- SHIP NOW (Tier-1): (a) verify-the-verifier seeded negative-control canaries into Q-6 (known-BAD artifacts through the SAME production detectors; a missed canary hard-BLOCKS the whole gate, exit 1); (b) missing-artifact=FAIL with an in-manifest explicit escape (env-blocked | optional-gap only) — the N1 convergence with the WIRE-DARK allowlist.
- DEFER (Tier-2): (c) skipBackfill cross-lane dependency (a skip clears only if a NAMED sibling passed) into R10; (d) STRUCTURAL redaction (a closed metadata-only audit-export struct) into G-9, kept separate BY TYPE from the content-carrying debug bundle; (e) posture labels (user-path | direct-gateway | native-approval) on R10 smoke evidence.
- EXCLUDED as vaporware: openclaw's default that lets a classified failure pass a green report (C3 — copy unknown=FAIL only); conflating the metadata audit ledger with the content-carrying trajectory bundle (V3).

## Reads (mandatory before coding)
- workflow/reasoning/openclaw-vs-elai-comparison-2026-07-11.md (2.2; steals S8/S9/S15; ENTRY 13 V1/V2/V3/N1/N2/C3)
- docs/thefinalglance/Q-plan/Q-6_*.md (the corpus to extend), .claude/rules/wiring-enforcement.md (R10), G-9 redaction, M fleet rollup
- crates/orchestrator/src/ (the Q-6 corpus runner + the R10 smoke evidence + G-9 redaction)

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/gate_corpus/verifier_canary.rs (seeded known-BAD canaries through production detectors; missed canary -> whole-gate FAIL exit 1)
  - crates/orchestrator/src/audit/metadata_export.rs (a CLOSED metadata-only export struct: no content field, hash-don't-store ids)
consumes_symbols:
  - crates/orchestrator/src/gate_corpus/mod.rs::EvidenceCorruptionCorpus (Q-6; EXTEND with the canary lane, prefer-reuse — net 0..+1)
  - crates/orchestrator/src/redaction.rs::RedactionGate (G-9; STRUCTURAL redaction hardens it)
reachable_from:
  - crates/orchestrator/src/gate_corpus/mod.rs (the corpus runner invoking the canary lane) [PRODUCTION gate corpus]
  - a #[cfg(test)] or contracts/ caller NEVER satisfies this (R7)
stub_ban:
  - NO missed canary that still passes the gate (a missed canary hard-BLOCKS the whole gate, exit 1)
  - NO classified failure passing a green report by default (unknown=FAIL is copied; the C3 default is NOT)
  - NO content field on the metadata-only export struct (STRUCTURAL redaction: content has nowhere to leak; keep the debug bundle separate BY TYPE)
  - NO missing artifact treated as pass (missing=FAIL unless an in-manifest env-blocked/optional-gap escape); NO .unwrap on prod; NO blake2/sha2
frontend_pair:
  rides the existing gate/evidence surface (the corpus kill-rate + posture label; W-4).
observable_effect:
  trigger: "elai bench gate-corpus --with-canaries --break-detector redaction"
  expect_ssb: "gate_credibility_measured"
  expect_artifact: "with a canary detector deliberately broken, the self-test lane detects the miss and the WHOLE gate goes to fail (exit 1); a smoke with no posture label FAILS; the metadata-export struct has no content field to leak into; NEGATIVE: a missed canary that still passes, a classified failure passing a green report by default, or content in the metadata-only export, FAILS"

## Wargame (plan-standard unit shape)
EXPECTED_OBSERVATION: running the gate corpus with a canary detector deliberately broken, the self-test lane detects the miss and the whole gate fails (exit 1); a smoke evidence row with no posture label FAILS; the metadata-export struct type-checks with no content field.
ON_FAIL: a missed canary still passes the gate, or content leaks into the metadata export (CAUSE: the self-test summary is not a required fail-closed lane, or the export struct has a content field). COUNTER-MOVE: make the self-test summary a required lane whose miss forces exit 1; define the export as a closed struct with no content field; the negative tests must FAIL.
FORK: if a required artifact is genuinely absent for an ENV reason (a sandbox not provisioned), the manifest's explicit escape (env-blocked) applies — but ONLY via a declared manifest row (N1); an undeclared absence is still FAIL. Route an undeclared miss to FAIL, a declared env-block to WARN.
RECON_NEEDED: confirm the exact Q-6 EvidenceCorruptionCorpus + G-9 RedactionGate symbols to extend + the openclaw confidence-report self-test shape (exitCode=1 semantics). Settling check: `grep -n "EvidenceCorruptionCorpus\|RedactionGate" crates/orchestrator/src/` + source-read openclaw confidence-report.ts.
ABORT: if the metadata export and the content-carrying debug bundle would share a type or a serializer (the V3 conflation trap), STOP and flag — they MUST be separate types so content cannot leak through a shared shape.
VERIFY: the load-bearing proof is the off-agent gate_credibility_measured row + the whole-gate exit 1 on a broken canary, driven by the real `elai bench gate-corpus` entrypoint. Run:
```bash
python .claude/tools/plan_gate_check.py --file docs/thefinalglance/P-plan/P-21_gate_credibility_hardening.md   # authoring gate (exit 0)
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/P-plan/P-21_gate_credibility_hardening.md   # execution done-gate
```
REDTEAM: surviving-attack seed — a detector that silently stops detecting (drift/rot) is caught by the seeded canary (a known-BAD artifact it must flag; a miss fails the whole gate). Attack that would succeed if unpatched: exfiltrating content through the "audit export" — PATCH: the export is a closed metadata-only struct with no content field, separate BY TYPE from the debug bundle (V2/V3). Deep red-team DEFERRED to execution pre-flight.

## INTEGRATION_SMOKE
An independent agent runs `elai bench gate-corpus` across the WHOLE frozen gate manifest on a HARD input: each frozen checker gets a seeded canary + a broken-detector arm, asserting every broken detector forces exit 1, every smoke carries a posture label, and no export path carries content. Toy single-gate canary does NOT satisfy R10.

## Grounding / citations
- Source: workflow/reasoning/openclaw-vs-elai-comparison-2026-07-11.md 2.2, steals S8/S9/S15, ENTRY 13 V1/V2/V3/N1/N2/C3 (VERIFIED-IN-SOURCE grades).
- ELAI ground truth (PLAN-asserted, executor re-greps W-7): Q-6, wiring R10, G-9, M per the indexes cited.
- CLAUDE.md hard rules: missing evidence = FAIL (wiring R1); verify-the-verifiers; blake3; no .unwrap on prod.

## PRODUCTION_HOOK
crates/orchestrator/src/gate_corpus/mod.rs — the Q-6 corpus runner invokes the verifier-canary lane on every gate; the R10 smoke evidence path gains a posture label; G-9 gains the closed metadata-export struct. All wired in this changeset.

## Next
Closes W2-adjacent credibility (their qa gates live inside their own boundary; ELAI's are frozen + off-agent) and feeds the PS-6 gate-credibility number. Pairs with Q-6 (the corpus) and M (the metadata-only rollup projection).

End of P-21.
