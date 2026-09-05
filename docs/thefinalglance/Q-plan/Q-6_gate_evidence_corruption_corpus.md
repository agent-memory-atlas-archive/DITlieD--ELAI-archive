# Q-6: gate-manifest-wide evidence-corruption corpus + measured per-gate kill-rate — the Hipp move (test the thing that IS the product: the gates)

STATUS: PLANNED (WAVE 1; EXTENSION(H trading-gate self-validation -> the WHOLE gate manifest); BUILD — one shared corpus generator + a measured kill-rate per gate)

- Track: ENGINEERING-TASTE | lever: concentrate-testing-on-the-product-which-is-the-gates | source: bellard-taste V5 (SQLite/Hipp 590:1 test:code, verified 2026-07-10) | grounding: STRONG (sqlite.org/testing.html fetched 2026-07-10; feature-map:1495 H gate self-validation) | nature: EXTENSION | effort: M | WAVE 1
- PROOF_SPINE: feeds the K compliance/appsec brief (a measured per-gate KILL-RATE number, regression-tracked); indirectly hardens PS-1..PS-6 by converting wiring-enforcement R1/R2 (missing evidence = FAIL; a dead evidence path must scream) from doctrine into a TESTED, numbered property of EVERY gate. Declares a K-brief number relationship, so no product/distribution `none` ack is required.

- Premise (bellard-taste V5): SQLite spends 590:1 test:code because the LIBRARY is the product (155.8 KSLOC lib vs 92053.1 KSLOC test, 100% branch + MC/DC, dbsqlfuzz ~1e9 mutations/day, OOM + I/O-error + crash injection — sqlite.org/testing.html, fetched 2026-07-10). ELAI's product is the GATES. The mutant/anomaly discipline currently scoped to the H trading-gate family ("each gate kills its own frozen mutant corpus", feature-map:1495) belongs on the ENTIRE gate manifest. Every frozen checker gets a standard evidence-corruption corpus (truncated journal, corrupt msgpack, missing SSB db, flipped verdict hash, zero-row evidence, OOM/I-O-error injection in the checker process) and MUST BLOCK on every corpus item; the per-gate KILL-RATE is a measured, regression-tracked number. This makes R1/R2 a tested property, not a promise.

## Steal/build
ONE deterministic corpus GENERATOR keyed by EVIDENCE TYPE (journal / SSB-db / hash / manifest / coverage-artifact), not hand-written corpora per gate (A4 anti-swamp): each frozen gate in .claude/gate-manifest.txt DECLARES the evidence types it reads; the generator emits the corruption corpus for those types (truncated / corrupt-msgpack / missing / flipped-hash / zero-row / OOM-I/O-inject); the gate is run against each item and must exit nonzero (BLOCK) on ALL of them; a per-gate KILL-RATE (items-blocked / items) is emitted as a measured row (`GateKillRateMeasured{gate_id, kill_rate, corpus_hash}`, the GateTaxMeasured/RecurrenceHitRateMeasured idiom). A kill-rate below 100% is a gate that fail-OPENS on some corruption — a finding, regression-tracked.
EXCLUDED: hand-authored per-gate corpora (A4 maintenance swamp — one generator keyed by evidence type, gates consume by declared type); running the corpus INSIDE the executing plan run (R8 — the corpus + generator are auditor-owned frozen artifacts, not run by the run they judge); a kill-rate that is asserted/stored rather than recomputed from the corpus (GD-2).

## Existing ground truth (PLAN-asserted; executor re-greps per W-7)
- H trading-gate self-validation: "each gate kills its own frozen mutant corpus" (feature-map:1495) — the discipline Q-6 generalizes from one family to the manifest.
- 3I DST (madsim, proptest 10000 cases, IPC fault injection, 100-run identical checkpoints) — the fault-injection idiom for the OOM/I-O corpus items.
- C-plan mutation testing (code mutants) + N evaluator-refutation verdict-collapse — adjacent mutant discipline (Q-6 is about EVIDENCE corruption, not code mutants).
- .claude/gate-manifest.txt (the frozen gate fileset, R8) — the manifest Q-6 iterates; each gate declares its evidence types.
- wiring-enforcement R1 (missing evidence = FAIL) + R2 (dead evidence path must scream) — the doctrine Q-6 turns into a tested number.
- SSB measured-number row idiom (GateTaxMeasured schema_version 271 / RecurrenceHitRateMeasured 270) — the `GateKillRateMeasured` pattern.

## REVISED scope (W-10 tiered)
- SHIP NOW (Tier-1): the evidence-type-keyed corpus generator + the per-evidence-type corruption set (truncated / corrupt-msgpack / missing / flipped-hash / zero-row) + the per-gate kill-rate runner over the manifest + `GateKillRateMeasured` + the regression-track (a kill-rate drop is a BLOCK).
- DEFER (Tier-2): the in-process OOM/I-O-error INJECTION items (need a fault-injection harness around the checker process; land after the file-corruption corpus proves the runner) + the K compliance/appsec-brief roll-up of the manifest-wide kill-rate.

## Integration
- .claude/tools (the auditor's frozen toolset): the corpus generator + the kill-rate runner; each gate-manifest entry gains an evidence-type declaration.
- crates/cli (the measured-number emit): `elai bench gate-kill-rate` recomputes + emits `GateKillRateMeasured`.
- frontend_pair: N/A with justification — the kill-rate is a bench/CLI + SSB artifact; a dashboard tile is a later K-brief job. Justified.

## MOVES (wargame)
M1 the evidence-type-keyed corpus generator: given an evidence type (journal/SSB-db/hash/manifest), emit the deterministic corruption set (truncated, corrupt-msgpack, missing, flipped-hash, zero-row); each gate-manifest entry declares the types it reads.
EXPECTED_OBSERVATION: the generator emits a fixed, hash-stable corpus per evidence type; a gate declaring type `journal` gets exactly the journal-corruption items; the corpus_hash is reproducible.
ON_FAIL: the corpus is nondeterministic / hand-listed per gate -> A4 swamp + irreproducible kill-rate -> generate deterministically from the evidence-type key; assert corpus_hash stability.
M2 the kill-rate runner: run each frozen gate against its corpus; require exit nonzero (BLOCK) on EVERY item; emit `GateKillRateMeasured{gate_id, kill_rate, corpus_hash}`.
EXPECTED_OBSERVATION: a correctly fail-closed gate scores kill_rate=1.0 (blocks all corruption); a gate that returns PASS on a zero-row or missing-db item scores <1.0 and is named.
ON_FAIL: a gate fail-OPENS on a corruption item (returns PASS/SKIP) and the runner reports 1.0 anyway -> the runner mis-scores -> a non-BLOCK exit on any item lowers the kill-rate; assert a deliberately-fail-open stub gate scores <1.0.
M3 regression-track the kill-rate: a drop in any gate's kill-rate between runs is a BLOCK; the manifest-wide kill-rate is the K-brief number.
EXPECTED_OBSERVATION: re-running on an unchanged manifest reproduces identical kill-rates; introducing a fail-open regression in one gate drops its kill-rate and BLOCKS.
ON_FAIL: a kill-rate drop passes silently -> the regression guard is absent -> compare against the last recorded kill-rate; a drop is a nonzero exit.

## FORK
FORK: if a gate's evidence type is not cleanly one of {journal, SSB-db, hash, manifest, coverage} (a composite/bespoke evidence path) -> declare its types as the UNION it reads and generate the corpus for each; if a gate reads NO off-agent evidence at all (a pure static lexical checker), it is exempt from the corpus and RECORDED as `evidence_type: static` in the manifest (a static-only gate has nothing to corrupt) — do NOT force a synthetic corpus onto it.

## RECON_NEEDED
RECON_NEEDED-1: the .claude/gate-manifest.txt entry shape + how to add an evidence-type declaration per gate -> settle via reading .claude/gate-manifest.txt + `grep -rn "gate.manifest\|gate_freeze" .claude/tools`.
RECON_NEEDED-2: each frozen gate's actual evidence-read path (which file/db/artifact it opens) -> settle via reading the checker scripts referenced in the manifest (the evidence type is what the gate opens, not what it claims).
RECON_NEEDED-3: the 3I DST fault-injection harness API for the Tier-2 OOM/I-O items -> settle via `grep -rn "madsim\|fault\|inject\|DST" .elai_cc/crates`.

## ABORT
ABORT: if running the corpus would require the EXECUTING plan run to author or run the generator over its OWN gates (R8: the gate + its corpus are frozen, auditor-owned) -> STOP. The corpus generator and the kill-rate runner are frozen auditor artifacts in .claude/tools; a run that generates the corpus that judges it has graded its own homework.

## WIRING_CONTRACT
produces_symbols:
  - .claude/tools evidence-type-keyed corpus generator + the per-gate kill-rate runner (frozen, auditor-owned)
  - the per-gate evidence-type declaration on each gate-manifest entry
  - `GateKillRateMeasured` SSB row + `elai bench gate-kill-rate` (baseline+1; re-read at edit W-9)
consumes_symbols:
  - .claude/gate-manifest.txt (R8 frozen fileset); H trading-gate self-validation idiom (feature-map:1495); 3I DST fault-injection (Tier-2); wiring-enforcement R1/R2; SSB measured-number idiom; gate_freeze_check.py
reachable_from:
  - `elai bench gate-kill-rate` runs the corpus over the manifest [PRODUCTION auditor bench]; the auditor's gate-run invokes it off-agent
  - a #[cfg(test)] caller NEVER satisfies this (R7)
stub_ban:
  - NO hand-authored per-gate corpus (one evidence-type-keyed generator; A4)
  - NO corpus run by the plan run it judges (R8 — auditor-owned frozen artifact)
  - NO asserted/stored kill-rate (recompute from the corpus; GD-2)
  - NO forcing a synthetic corpus on a genuinely static-only gate (record evidence_type: static)
observable_effect:
  trigger: "elai bench gate-kill-rate over the frozen gate manifest, then re-run after stubbing one gate to fail-open on a zero-row item"
  expect_ssb: "one GateKillRateMeasured per gate {gate_id, kill_rate, corpus_hash}; a correctly fail-closed gate = 1.0; the fail-open-stubbed gate <1.0 and named"
  expect_artifact: "a manifest-wide kill-rate number recomputable from the journal; NEGATIVE: a gate that returns PASS on a corrupted/missing evidence item but still scores 1.0, or a hand-listed nondeterministic corpus, must FAIL"

PRODUCTION_HOOK: producer the kill-rate runner + `GateKillRateMeasured` emit -> consumer `elai bench gate-kill-rate` (the auditor bench CLI), wired same-changeset; the runner is invoked in the auditor's gate pass over a changeset; the measured row is the off-agent artifact.

## VERIFY
VERIFY: the auditor runs, off-agent:
`python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/Q-plan/Q-6_gate_evidence_corruption_corpus.md --run-trigger`
`python .claude/tools/gate_freeze_check.py`   # the generator + runner are frozen (R8); a run editing them is BLOCK
pass = `GateKillRateMeasured` rows landed off-agent for every non-static manifest gate, a deliberately fail-open stub gate scored <1.0 and was named, and the kill-rate reproduced identically on a re-run (deterministic corpus).

INTEGRATION_SMOKE: an independent agent runs the full corpus over the REAL frozen gate manifest, asserts every fail-closed gate scores 1.0, then injects a fail-open regression into one real gate and confirms the runner drops that gate's kill-rate and BLOCKS — the real manifest + real gates, not a toy checker (R10).

## REDTEAM
REDTEAM: attack that FAILED — "let a gate quietly return PASS when its evidence db is missing and still look tested": FAILS because M2 runs the gate against the missing-db corpus item and a non-BLOCK exit lowers its kill-rate below 1.0, naming the fail-open — the exact R2 dead-evidence-path scream, now measured. Attack that SUCCEEDED and its patch — "drown the effort in per-gate corpora that rot": patched by the SINGLE evidence-type-keyed generator (A4) so gates consume a shared deterministic corpus by declared type; there are no hand-maintained per-gate corpora to rot.

## Grounding
- bellard-taste-elai-2026-07-10.md V5 + A4 (the one-generator anti-swamp counter) + the SQLite testing facts (sqlite.org/testing.html, fetched 2026-07-10).
- H trading-gate self-validation (feature-map:1495), 3I DST, wiring-enforcement R1/R2, .claude/gate-manifest.txt (R8), SSB measured-number rows — confirmed at authoring, executor re-greps (W-7).

## COMPOSES-WITH
.claude/gate-manifest.txt (R8 frozen set), H trading-gate self-validation idiom, 3I DST fault-injection, wiring-enforcement R1/R2, SSB measured-number rows, gate_freeze_check.py. Re-implements NONE (GD-1). Pairs with Q-5 (both trust-spine durability); its kill-rate number feeds the K compliance/appsec brief.

End of Q-6.
