# J-14: repro-pack + RRA verify upgrade — tiered guarantees, first-divergence, self-describing artifacts, flow-fingerprint guard

- Track J (determinism-steal extensions) | lever PROVABLE | grounding STRONG (detllm source ingest + live grep) | nature EXTENSION (of G-2/G-10) + small BUILD legs | effort M | WAVE 2-3 | depends none (J-13 makes Tier-1/2 PASSABLE on local; without it the tiers report honestly)

- Premise: G-2's RRA verifies the action stream but its verdict is binary and its bundle is not self-describing. detllm (Apache-2.0, docs/.source/detllm, v0.1.1) contributes a clean verdict vocabulary and artifact schema: 3-tier guarantees (T0 artifact-only / T1 run-to-run token equality / T2 +score equality, capability-gated with explicit downgrades), a determinism_applied record (which seeds/env controls were ACTUALLY active — self-describing bundles), and a first-divergence locator (not "did it change" but "where did it first change" — the right unit for G-10's drift canary). Two triage riders land here because they ARE determinism verification: the intentusnet-style execution fingerprint as a TemplateCrystallizer drift guard, and an entropy-scan CI gate (unseeded random/uuid in deterministic paths). Schema is INSPIRATION not dependency (18-star alpha; we keep blake3 and our msgpack bundle).

## Steal/build
1. tier taxonomy on `elai replay --verify`: T0 = action-stream replay matches (today's G-2 verdict), T1 = token-stream blake3 equality (passable once J-13 ships), T2 = score/logprob equality where the lane exposes scores; capability-gated with recorded downgrades ({from, to, reason}) — tier_effective <= tier_requested always; strict mode -> UNSUPPORTED_REQUEST aborts, never silently degrades (R1 analog).
2. determinism_applied block IN the RRA bundle: ELAI_SEED, sampling_seed, determinism-mode flags, env controls actually read at run time — the bundle states its own guarantee basis. detllm gotchas honored: the fingerprint hash excludes its own field; artifact serialization is key-sorted for hash stability; score comparison is EXACT equality, never epsilon.
3. first-divergence locator: on T1/T2 mismatch, record the first divergent position + both values into the bundle diff section; G-10's canary consumes it (drift reports become "diverged at token N", exact).
4. file-change snapshots: record pre/post content hashes (+ optional zstd content) for files touched during a run INTO the bundle — the pre-image is ALREADY captured at file_ops.rs:272/300 then discarded (H-49 finding; PLAN-asserted, executor re-greps) — J-14 persists the hash pair; full content store stays H-49's CAS scope (GD-1: do not re-implement). Auth/secret scrub before archive (composes G-9 redaction; the culpa DO-NOT-COPY exhibit: it stores auth headers verbatim).
5. fixture-cassette extraction: `elai rra export-fixture --adapter <name>` pulls the span subset (requests/responses for one adapter) from a bundle into a small fixture file with a per-fixture blake3 fingerprint; matching key = (model + message-content-blake3 + tool_name), NEVER positional (the culpa/evalcraft brittleness); this is I-49's replay-fixture input format, defined once here.
6. flow-fingerprint guard on the crystallizer: at crystallization time hash (tool sequence + arg-shape hashes) per compiled template (compiled_pipeline.rs TemplateCrystallizer:293); on each compiled-flow execution re-hash and compare — mismatch = de-crystallize + fall back to the LLM path + emit (the intentusnet steal, reimplemented ~40 LOC, blake3).
7. entropy-scan CI: a deterministic lint over crates/orchestrator + crates/cli deterministic-path modules flagging unseeded randomness (thread_rng without ELAI_SEED derivation, uuid4-style) — observe-only first (JD-3), allowlist file for the legitimate sites.
EXCLUDED: detllm as a dependency; sha256 (blake3 everywhere, house rule); any LLM-judge equality; H-49's CAS object store (composed, not built here).

## Existing ground truth (grep-confirmed 2026-06-11 unless marked)
- crates/orchestrator/src/rra.rs verify_rra:474 / verify_artifact:481 — the verify leg this extends.
- crates/orchestrator/src/compiled_pipeline.rs TemplateCrystallizer:293 (check_crystallization gate :54; fires-after-N-successes test :874) — the fingerprint-guard target.
- file_ops.rs:272/300 pre-image capture-then-discard (PLAN-asserted from the H-49 design pass; re-grep per W-7).
- G-9 redaction (write-time secret+PII, blake3-keyed) DARK(ELAI_RETENTION_PURGE_ENABLE) — the scrub composes its functions, does not depend on its flag.
- interface map (W-3 ground truth): workflow/papers/detllm-source-interface.md (verbatim field tables, tier gates, divergence algorithm, 10 gotchas, contract stubs).

## Integration
- crates/orchestrator/src/rra.rs (~250 LOC: tier verdicts, determinism_applied, first-divergence, snapshots, scrub) + crates/cli rra/export-fixture arm (~80 LOC) + compiled_pipeline.rs guard (~40 LOC) + .claude-side entropy lint OR clippy-style xtask (~60 LOC, decide at edit).
- +ReproPackEvent SSB (Pattern-2 single-variant; Kind: TierEvaluated / FirstDivergenceLocated / FlowFingerprintMismatch / FixtureExtracted). baseline+1 from LIVE 229/540 (ssb.rs:3066 / action_type_lock.rs:662, grep 2026-06-11); re-read at edit (W-9). Full 6-part variant discipline.
- frontend_pair: tier verdict + divergence position on the existing RRA/review surface (decide at edit; W-4).

## WIRING_CONTRACT
produces_symbols:
- crates/orchestrator/src/rra.rs (TierVerdict enum + determinism_applied block + first_divergence diff + file snapshot hashes + scrub-before-archive)
- crates/cli (rra export-fixture subcommand arm)
- crates/orchestrator/src/compiled_pipeline.rs (flow fingerprint at crystallize + compare at execute)
consumes_symbols:
- crates/orchestrator/src/rra.rs::verify_rra (extended in place, not forked)
- crates/orchestrator/src/compiled_pipeline.rs::TemplateCrystallizer (the guard rides check/execute, never re-implements crystallization)
- G-9 redaction functions (scrub leg), J-13 sampling_seed field (T1/T2 basis), G-10 canary (first-divergence consumer)
reachable_from:
- `elai replay --verify <bundle>` and `elai run --emit-rra` (existing CLI verbs; tier verdict replaces the binary verdict)  [PRODUCTION RRA surface]
- compiled-flow execution path through TemplateCrystallizer materialization  [PRODUCTION pipeline]
- a #[cfg(test)] / contracts/ caller NEVER satisfies this (R7)
stub_ban:
- NO sha256 (blake3 only); NO epsilon score comparison (exact equality per the detllm finding)
- NO silent tier downgrade (every downgrade recorded {from,to,reason}; strict mode aborts)
- NO auth/secret material in exported bundles or fixtures (scrub is unconditional, negative-tested)
- NO positional fixture matching (content-keyed only)
- NO re-implementation of H-49 CAS or G-9 redaction internals
frontend_pair: tier + divergence on the existing RRA/review surface (decide at edit) or W-6-justified absence.
observable_effect:
- trigger: "`elai run --emit-rra` on a fixture task, then `elai replay --verify` against the bundle; then mutate one recorded response byte and re-verify"
- expect_ssb: "TierEvaluated{tier_effective, downgrades} on verify; FirstDivergenceLocated{position} on the mutated re-verify; FlowFingerprintMismatch when a crystallized template's tool sequence is perturbed in fixture"
- expect_artifact: "the bundle carries determinism_applied + (on divergence) the first-divergence diff; the exported fixture file's blake3 fingerprint validates. NEGATIVE (R2): a bundle missing determinism_applied FAILS verify (missing evidence = FAIL); a seeded auth-header fixture is provably scrubbed (grep on the exported bytes = 0 hits); the perturbed crystallized flow runs the LLM fallback, never the stale template."

## Grounding
- detllm source ingest: workflow/papers/detllm-source-interface.md (agent p2p-ingest-3; verbatim schemas + the 10 port gotchas).
- intentusnet fingerprint + entropy-scan: agent det-repos-S2 (README-fetched 2026-06-11; reimplemented, never a dep — 6-star solo project).
- culpa/evalcraft riders + DO-NOT-COPY exhibits: agent det-repos-S3 (source-fetched proxy.py/replay.py findings).
- steal ledger: workflow/reasoning/steal-determinism-repos-2026-06-11.md Tier-1 item 2 + Tier-2 items 5/6/8.
- live anchors: rra.rs:474/481, compiled_pipeline.rs:293/54 (grep 2026-06-11).

## COMPOSES-WITH
G-2 RRA [G KEYSTONE] (extended in place), G-10 drift canary (consumer), J-13 (T1/T2 basis), H-49 time-travel (snapshot hashes are its hash leg; CAS stays H-49's), I-49 replay-fixture harness (the fixture format defined here), I-45 objective-hash receipts (sibling seal, untouched), G-9 redaction (scrub), C-25 crystallizer (guard). Restates none.

## Open questions / risks
- tier names vs detllm's: keep T0/T1/T2 semantics but ELAI vocabulary (ActionFaithful/TokenFaithful/ScoreFaithful) — decide at edit, document the mapping.
- bundle size with snapshots: hash-pair-only default, content opt-in — measured before any default flip (GD-4).
- entropy-scan false positives on legitimate randomness (e.g. jitter/backoff): allowlist + observe-only first (JD-3).

## Next
Turns "reproducible" from a binary claim into a graded, self-describing verdict; gives G-6 SAFEDELTA runs shareable bundles whose guarantee basis is explicit; the fixture format unblocks I-49.

End of J-14.
