# Cross-family shared invariants — ELAI deep feature discovery

## Meta

- **Inputs (read-only):** 17 family aggregates in this directory: `PHASE`, `B`..`Q`.
- **Output:** this file only.
- **Method:** keyword and doctrine sweeps across aggregates (fail-closed, append-only, ELAI_SEED, blake3, off-agent, W-1..W-10, GD-*, R1/R8, wire-dark, idempotency, claims, budgets, CANT-LIE, UI runes, memory grain). Shared means asserted as a system rule in **two or more** families with atom-level anchors.
- **NEWER-PLAN PRECEDENCE (disputes):** explicit SUPERSEDED/replacement marker > later letter family > higher plan number for **target state**. Losing wording kept as a superseded note with its aggregate anchor.
- **Anchor form:** `Family atom-id @ AGGREGATE.md:line` (line is the aggregate line, not the original plan).
- **Classification note:** family atoms carry their own E/I/A/C tags; this document does not re-grade them.

---

# Theme A — Integrity / evidence

## INV-A01 — Off-agent evidence is required

**Statement:** A gate measures an artifact the generator did not write (event row, coverage hit, hash-bound test outcome, span, wiring run-trigger). The model’s own stdout or self-report is not sufficient evidence.

**Families:** B, D, E, F, H, I, J, K, L, N, O, P, Q (13)

**Per-family evidence:**

| Family | Atom ID | Aggregate anchor | Note |
|---|---|---|---|
| B | SUMMARY/SUM-0389 | B-AGGREGATE.md:2197 | Names global trio: off-agent evidence, fail-closed missing evidence, injected time/RNG |
| D | SUM-D-30-061 | D-AGGREGATE.md:2078 | D composes with off-agent / fail-closed / deterministic spine |
| H | H-003/DH003-H0-037 | H-AGGREGATE.md:70 | HD-4 auditor-driven acceptance needs off-agent SSB row |
| I | I-002/I002-IDX-023 | I-AGGREGATE.md:65 | Track-19 LLM critique is advisory; only off-agent spine decides |
| J | J-001/DJ001-G-026 | J-AGGREGATE.md:1515 | JD-1 real harness trigger + off-agent artifact |
| O | O-001/O0-E040 | O-AGGREGATE.md:91 | Real-entrypoint trigger + off-agent observable artifact |
| P | P-001/P4-E020 | P-AGGREGATE.md:134 | VERIFY off-agent: wiring_contract_check + production_caller_check |
| Q | Q-002/Q6-E019 | Q-AGGREGATE.md:140 | Auditor gate-run invokes off-agent; test-only never substitutes |

**Variations:** B states the overview triad as context for B concrete liveness/entropy checks. I scopes the rule to critique surfaces (advisory LLM). F/G wire the same idea as W-1/W-5 run-trigger + live observable effect without always using the words “off-agent.”

---

## INV-A02 — Missing evidence is FAIL (never SKIP)

**Statement:** Absence of required evidence fails the gate. Missing evidence must not be treated as pass, skip, or silent clean.

**Families:** B, G, H, J, K, M, N, O, P, Q (10+; fail-closed wording also in C,D,E,F,I,L,PHASE)

**Per-family evidence:**

| Family | Atom ID | Aggregate anchor | Note |
|---|---|---|---|
| B | SUMMARY/SUM-0389 | B-AGGREGATE.md:2197 | “fail-closed missing evidence” |
| H | H-006/H006-H29-008 | H-AGGREGATE.md:3467 | Missing evidence on any of four axes → HALT, never SKIP |
| K | K-001/K0-E027 | K-AGGREGATE.md:127 | No-evidence hunk renders UNVOUCHED, never silently clean |
| M | M-001/M6-E033 | M-AGGREGATE.md:148 | LOCK-4 / R1 normative: missing evidence = FAIL; UNVOUCHED never renders clean |
| O | O-001/O0-E041 | O-AGGREGATE.md:92 | “Missing evidence is a failure.” |
| N | N-002/IX-018 | N-AGGREGATE.md:795 | Fail-closed authoring R1 composes with R10 seam gate |
| Q | Q-002/IX-006 | Q-AGGREGATE.md:819 | Q-1/Q-6 harden R1/R2 missing/dead evidence as tested properties |
| D | D12-006 | D-AGGREGATE.md:213 | Claim without procedure cannot reach MEMORIZING; auditor enforces |

**Variations:** M elevates R1 to conformance for third-party verifiers (LOCK-4). H uses HALT vocabulary. K uses UNVOUCHED presentation. O folds many negative cases into SUMMARY/INV-04 (out-of-enum, canary-to-steal, empty card, unclaimed write).

---

## INV-A03 — Author emits; auditor enforces (no self-certify)

**Statement:** The authoring/generating path may emit claims or proposals; an independent auditor/checker enforces gates. The same agent does not certify its own work as done.

**Families:** D, F, H, I, Q (+ B/G/J via wiring auditor + W-1)

**Per-family evidence:**

| Family | Atom ID | Aggregate anchor | Note |
|---|---|---|---|
| D | D12-006 | D-AGGREGATE.md:213 | Author emits L3 claim; auditor enforces fail-closed MEMORIZING bar |
| F | SUMMARY/F6-19 | F-AGGREGATE.md:1305 | Auditor enforces type barrier between skill trust and memory trust |
| H | H-003/DH003-H0-037 | H-AGGREGATE.md:70 | Canvas/model-driving acceptance is auditor-driven |
| I | I-002/I002-IDX-023 | I-AGGREGATE.md:65 | LLM critique never load-bearing |
| Q | Q-001/Q13-E008 | Q-AGGREGATE.md:434 | Excludes LLM judge for validity among other silent-emission bans |

**Variations:** Strength is hard in D/M (procedural claim gates) and softer as “advisory only” in I Track-19. Q bans LLM-as-judge for validity rather than naming “auditor.”

---

## INV-A04 — Claims discipline (measured or tagged)

**Statement:** External-facing or superlative claims ship only with a journal-recomputable measured pointer or an explicit `[EXTERNAL]` / `[DESIGN]` tag. Claims linters fail closed on violation.

**Families:** E, G, H, I, J, K, L, M, N, Q (10)

**Per-family evidence:**

| Family | Atom ID | Aggregate anchor | Note |
|---|---|---|---|
| K | K-001/K1-E013 | K-AGGREGATE.md:160 | claims_policy_lint: number/superlative needs measured-pointer or tag; exit nonzero |
| K | SUMMARY/K-1.09 | K-AGGREGATE.md:719 | Journal-recomputable ELAI number or `[EXTERNAL]`/`[DESIGN]` |
| G | G-001/DG001-G0-E020 | G-AGGREGATE.md (~GD-3) | Safety/perf results are gate-ON vs gate-OFF marginal effects, not SWE-bench rank claims |
| M | M-001/IX-003 | M-AGGREGATE.md:652 | MD-1..MD-5 letter-wide: includes claims discipline on money |
| N | N-002/N8-E020 | N-AGGREGATE.md:566 | No headline number not recomputable from journal (GD-2) |

**Variations:** K is the operational lint. G GD-3 is evaluation-claim boundary. M scopes claims discipline to commerce/money surfaces.

---

## INV-A05 — CANT-LIE / anti-fabrication

**Statement:** Production paths must not fabricate measurements, anchors, or clean verdicts. Learned/admission surfaces reject re-learning or inventing instruction-surface content (CANT-LIE).

**Families:** B, C, E, F, G, H, I, J, K, L, N, P, PHASE, Q (14 with related wording)

**Per-family evidence:**

| Family | Atom ID | Aggregate anchor | Note |
|---|---|---|---|
| I | I-001/I8-E019 | I-AGGREGATE.md:832 | Ban fabricated measurements; blake3 only; no unwrap |
| J | J-007/J0-U109 | J-AGGREGATE.md:128 | J-109 CANT-LIE admission pipeline at memory/playbook/skill seams |
| J | J-002/DJ002-93-019 | J-AGGREGATE.md:2132 | False-positive block only after cant-lie/wiring/mutation gates pass |
| B | SUMMARY/SUM-0391 | B-AGGREGATE.md:2199 | Do not label fake in-process tests as isolation/IPC evidence |
| K | K-001/K0-E027 | K-AGGREGATE.md:127 | UNVOUCHED never silently clean |

**Variations:** I states “fabricated measurements” as a hard production ban on many units. J names CANT-LIE as an admission pipeline. B focuses on fake test evidence.

---

## INV-A06 — Journal completeness (observed effect needs a row)

**Statement:** An append-only log cannot prove completeness by itself. Every covered side-effect must be mediated or independently observed. Observed sensitive effect with no matching journal row is DETERMINISTIC FAIL (RECONCILE-MISSING=FAIL). Dropped-event counters fail closed.

**Families:** K (primary), plus G/H/N/O event-emission doctrine

**Per-family evidence:**

| Family | Atom ID | Aggregate anchor | Note |
|---|---|---|---|
| K | K-003/K21-E002 | K-AGGREGATE.md:1730 | Journal COMPLETENESS invariant; missing row = hard FAIL |
| K | K-003/K21-E006 | K-AGGREGATE.md:500 | RECONCILE-MISSING=FAIL; MONITOR-THE-MONITOR; dropped counters fail closed |
| O | O-001/O0-E020 | O-AGGREGATE.md:71 | Every task fire must be an SSB row |
| G | G-001/DG001-G0-E007 | G-AGGREGATE.md:47 | W-1 live effect must emit SSB row and artifact |

**Variations:** K is the strongest statement (completeness cannot be assumed). O/G state mandatory emission; K states detection of missing mediation.

---

# Theme B — Determinism / replay

## INV-B01 — Append-only SSB journal (no UPDATE / no DELETE)

**Statement:** The credibility substrate is an SSB SQLite-WAL journal that is append-only: no UPDATE and no DELETE of sealed event history. Schema is golden-locked; evolution is append-only field/variant growth.

**Families:** B, C, D, E, G, H, I, J, K, L, M, N, O, PHASE, Q (15; F uses immutable gate rows more than SSB wording)

**Per-family evidence:**

| Family | Atom ID | Aggregate anchor | Note |
|---|---|---|---|
| G | G-001/DG001-G0-E003 | G-AGGREGATE.md:44 | SSB WAL append-only/no-UPDATE/no-DELETE + ELAI_SEED substrate |
| I | I-001/IG-E007 | I-AGGREGATE.md:732 | SSB schema golden-locked and append-only |
| H | H-001/H35-003 | H-AGGREGATE.md:606 | UI/restore is logical layer over seeded append-only SSB replay |
| K | K-003/K12-E002 | K-AGGREGATE.md:1621 | Lossless RECORD = append-only SSB; trim→stub→recover_stubs + blake3 |
| M | M-001/M6-E021 | M-AGGREGATE.md:612 | Append-only field evolution locked by golden file |
| C | SUMMARY/S-I0886 | C-AGGREGATE.md:561 | Producer → append-only event row → SsbViewer/Tauri |
| PHASE | P3D-E003 | PHASE-AGGREGATE.md:593 | Total-ordered LastValue or append-only Topic via SSB seq |
| L | L-002/L8-E005 | L-AGGREGATE.md:608 | Pruning QUARANTINES never DELETEs |

**Variations / conflicts:**

- **D held-out store:** D0-041 says append-only layout; D6-015 rewrites TOML status on retirement (D-001/AC-008 @ D-AGGREGATE.md:406). Target: no-delete of evidence history holds; status rewrite is a separate TOML store, not SSB mutation.
- **B entry_points:** B34-042 notes project `[entry_points]` append-only customization is only a decision-doc recommendation (not firm). Does not weaken SSB append-only.

---

## INV-B02 — Controlled time and randomness (ELAI_SEED / providers)

**Statement:** Wall-clock and RNG used for harness decisions come from injected providers. `ELAI_SEED` pins RNG (and related seeded paths). Seeded clocks / `ClockProvider` make replay possible. Identical inputs + seed yield byte-stable decisions where claimed.

**Families:** B, C, D, E, F, G, H, I, J, K, L, M, N, P, PHASE, Q (16; O has byte-identical replay without always naming ELAI_SEED)

**Per-family evidence:**

| Family | Atom ID | Aggregate anchor | Note |
|---|---|---|---|
| PHASE | P0-E060 | PHASE-AGGREGATE.md:107 | ClockProvider: now, elapsed_since, system_now |
| C | C-006/C15-E009 | C-AGGREGATE.md:1333 | ELAI_SEED seeds RNG; SeededClock tick-0 contract; byte-identical non-clock fields |
| C | C-006/C15-E004 | C-AGGREGATE.md:1328 | SeededClock AtomicU64 tick starts at 0 regardless of ELAI_SEED |
| D | D5-028 | D-AGGREGATE.md:132 | SeededRng; two seed-42 runs → byte-identical promotion decisions |
| G | G-001/DG001-G0-E003 | G-AGGREGATE.md:44 | ELAI_SEED part of credibility/replay substrate |
| B | B39-065 | B-AGGREGATE.md:1857 | Five-run fixed-seed/mock triage under ELAI_SEED=42 |
| N | N-001/N0-E024 | N-AGGREGATE.md:128 | GD-2: numbers recomputable under pinned ELAI_SEED |

**Variations:** C documents that ELAI_SEED does **not** seed SeededClock’s tick counter (tick always starts at 0). D/B use seed for sampling/triage. PHASE introduces ClockProvider in phase-0.

---

## INV-B03 — GD-2: metrics recomputable from journal under seed

**Statement:** Every numeric metric / safety-delta / confidence number emitted to a human-facing or gate-facing surface must be recomputable from the SSB stream under a pinned `ELAI_SEED`. Otherwise it is wire-dark.

**Families:** G, H, N (doctrine owners); echoed in J, K, L, Q measurement paths

**Per-family evidence:**

| Family | Atom ID | Aggregate anchor | Note |
|---|---|---|---|
| G | G-001/DG001-G0-E019 | G-AGGREGATE.md:59 | GD-2 recomputable from SSB under ELAI_SEED or wire-dark |
| H | H-003/DH003-H0-031 | H-AGGREGATE.md:69 | Same GD-2 text on H index |
| N | N-001/N0-E024 | N-AGGREGATE.md:128 | GD-2 REPRODUCIBLE-METRIC for all N-N emits |
| N | N-001/IX-012 | N-AGGREGATE.md:763 | N-1/N-9/N-11 recompute from SSB/journal under seed |
| J | J-004/J1-E021 | J-AGGREGATE.md:138 | Perception observables expect rows under ELAI_SEED (GD-2) |

**Variations:** G/H/N treat GD-2 as letter-wide doctrine. Later letters (N,Q) restate it as “inherit G-0 GD-*.”

---

## INV-B04 — Byte-identical / hash-checkable replay where claimed

**Statement:** Replay and verification paths that claim determinism must produce byte-identical or hash-checkable outcomes for non-clock fields (cassette replay with network detached; purge must not break hash-checkable steps).

**Families:** All 17 mention determinism/replay; strong explicit atoms in B, C, E, F, G, H, I, J, K, O, PHASE

**Per-family evidence:**

| Family | Atom ID | Aggregate anchor | Note |
|---|---|---|---|
| O | O-001/O1-E079 | O-AGGREGATE.md:228 | Replay acceptance: byte-identical cassette output, network detached |
| C | C-006/C15-E026 | C-AGGREGATE.md:1350 | Two-run ELAI_SEED=42 byte-identical SSB action_type sequences |
| I | I-005/DII005-I44-016 | I-AGGREGATE.md:393 | `elai replay --verify` still passes after purge for hash-checkable steps |
| E | E-004/E004.SD29.CF003 | E-AGGREGATE.md:1237 | Conflict: best-effort replay vs fail-closed frozen predicates (see superseded) |
| B | B-001/X-012 | B-AGGREGATE.md:2255 | Determinism spans injected seeds, fixed order, seeded UI timing |

**Superseded / conflict note (E):** E-29 acknowledges clock/RNG/KV gaps make some replay best-effort; **target state** remains fail-closed frozen predicates + controlled providers (INV-B02/B03), not “best-effort is good enough.” Losing wording: E-004/E004.SD29.CF003 @ E-AGGREGATE.md:1237.

---

## INV-B05 — BLAKE3-only content hashing (non-RFC)

**Statement:** Non-RFC content hashing uses **blake3** only. blake2 and sha2 are forbidden for those uses (identity, staleness, fingerprints, manifests, gate digests).

**Families:** All 17 reference blake3; explicit ban of blake2/sha2 in B, D, E, F, G, H, I, J, L, N, O, P, Q, PHASE (later units)

**Per-family evidence:**

| Family | Atom ID | Aggregate anchor | Note |
|---|---|---|---|
| E | E-001/DEEP-E001-E0-062 | E-AGGREGATE.md:117 | Global content-hash rule: blake3; never blake2 or sha2 |
| B | B16-014 | B-AGGREGATE.md:730 | phase-3C supersedes blake2s with blake3 for non-RFC role |
| B | SUMMARY/SUM-0392 | B-AGGREGATE.md:2156 | Family-wide blake3 never blake2/sha2 |
| G | G-001/DG001-G0-E042 | G-AGGREGATE.md:83 | BLAKE3 only; BLAKE2/SHA2 forbidden |
| F | F-001/DF001-F0-E041 | F-AGGREGATE.md:460 | All F work inherits BLAKE3-only hashing |
| O | O-001/O0-E082 | O-AGGREGATE.md:133 | Hashing must use blake3, not blake2 or sha2 |
| N | N-001/N0-E080 | N-AGGREGATE.md:184 | Hard rules: blake3 only |
| P | P-001/P0-E043 | P-AGGREGATE.md:90 | Hard rules: blake3 not blake2/sha2 |
| Q | Q-001/Q0-E024 | Q-AGGREGATE.md:80 | Same hard-rule block |
| L | L-002/L0-E055 | L-AGGREGATE.md:486 | Hard rules: blake3 only |
| PHASE | P4DOR-E014 | PHASE-AGGREGATE.md:675 | Hashes MUST use blake3 (no blake2/sha2) |

**Superseded notes (blake2 → blake3):**

1. **PHASE early units used blake2** for ONNX setup and vector-cache keys: P0-E015 (blake2 verify) @ PHASE-AGGREGATE.md:62; P1C-E009 content_blake2s @ :139; P1A-E035 blake2s staleness @ :533. **Target state:** B16-014 / E-0-062 / G0-E042 supersede non-RFC blake2 with blake3. Losing wording retained at those PHASE anchors.
2. **B19-015** records phase-4C saying blake2s while ipc_hooks uses blake3; source treats phase-4C as stale and keeps blake3 (@ B-AGGREGATE.md:830).
3. **B35-006** corrects SocratiCode SHA-256 description: ELAI must use blake3 under anti-stale rule (@ B-AGGREGATE.md:1600).
4. Explicit SUPERSEDED trail: B SUP-074 from B16-014 @ B-AGGREGATE.md:2909.

**Variation:** RFC/protocol contexts may still mention other hashes when the external protocol requires them; the ban is for ELAI non-RFC content identity.

---

## INV-B06 — Idempotent durable operations

**Statement:** Indexing, refresh, seal, store load/append, and due-tick dispatch are idempotent: re-running without content change does not rewrite or double-fire.

**Families:** B, C, D, E, F, G, H, I, J, O, P, PHASE (12)

**Per-family evidence:**

| Family | Atom ID | Aggregate anchor | Note |
|---|---|---|---|
| B | B42-E09 | B-AGGREGATE.md:1982 | Refresh compares blake3; unchanged → no rewrite |
| B | B35-055 | B-AGGREGATE.md:1649 | Idempotent staleness tests (skip unchanged) |
| D | D6-042 | D-AGGREGATE.md:194 | Idempotent store load/append/retire |
| E | E-001/DEEP-E001-E0-048 | E-AGGREGATE.md:103 | Durable checkpointing: idempotent replay |
| F | F-002/DF002-F2C-035 | F-AGGREGATE.md:857 | Gate rows immutable/idempotent; seed tampering fail-closed |
| O | O-001/O2-E076 | O-AGGREGATE.md:339 | Last-fired watermark makes due-tick dispatch idempotent |
| O | SUMMARY/O-2.E42–E43 | O-AGGREGATE.md:696-697 | Empty fire card / double-fire fail done-gate |
| P | P-001/P0-E023 | P-AGGREGATE.md:70 | Mandatory idempotency keys on control-plane/channel |
| I | I-001/I18-E017 | I-AGGREGATE.md:935 | Sealed ChainAdvanced row bytes immutable |

**Variations:** O uses watermark + negative done-gates for double-fire. P mandates idempotency keys on channel binding. F treats security seed rows as immutable/idempotent.

---

# Theme C — Safety / gating

## INV-C01 — Fail-closed is the default gate posture

**Statement:** Gates fail closed: blocked write, failed verdict, or refused action when a required check does not pass. Fail-open paths must not advertise fail-closed guarantees.

**Families:** All 17

**Per-family evidence (sample):**

| Family | Atom ID | Aggregate anchor | Note |
|---|---|---|---|
| B | B24-011 | B-AGGREGATE.md:1034 | Tier-1 failure → Fail; Tier-1 pass + Tier-2 disagree → HumanReview |
| F | SUMMARY/F1-06 | F-AGGREGATE.md:241 | Write blocked fail-closed if any axis fires |
| G | G-001/DG001-G0-E016 | G-AGGREGATE.md:57 | Fail-open Tier 2 may not advertise fail-closed |
| L | L-001/LOQF-E010 | L-AGGREGATE.md:134 | VibeGate F-1 fail-closed multi-axis PreToolUse |
| M | M-001/M0-E019 | M-AGGREGATE.md:285 | MD-1 keeps safety gates fail-closed |
| P | P-001/P0-E019 | P-AGGREGATE.md:67 | Config/ops: fail-closed defaults, LKG, shrink-guard |
| Q | Q-001/Q8-E003 | Q-AGGREGATE.md:387 | Crossed SET ceiling is fail-closed BLOCK |
| PHASE | PHASE-001/SD-P4M-005 | PHASE-AGGREGATE.md:481 | exit_code fail-closed among runtime wiring |

**Variations:** Scope objects differ (write gate, media verdict, novelty canary, spend guard, claims lint). Posture is shared.

---

## INV-C02 — W-10: Tier-1 in-process fail-closed vs Tier-2 gated

**Statement:** Axes needing network, subprocess, or RunPod at gate time are split. In-process Tier-1 is the fail-closed done-gate. Tier-2 is separately gated and may not inherit Tier-1 fail-closed advertising.

**Families:** F, G, H, I, J, K, L, M, N, O, P, Q, PHASE, B (14+)

**Per-family evidence:**

| Family | Atom ID | Aggregate anchor | Note |
|---|---|---|---|
| G | G-001/DG001-G0-E016 | G-AGGREGATE.md:57 | W-10 definition |
| N | N-001/N0-E022 | N-AGGREGATE.md:126 | W-10 restated on N index |
| F | F-002/DF002-F1-003 | F-AGGREGATE.md:192 | Dependency-existence / MCP scan excluded from Tier-1 advertising |
| J | J-007/J0-E011 | J-AGGREGATE.md:70 | W-1..W-10 inherited VERBATIM from F-0 |
| O | O-001/O0-E039 | O-AGGREGATE.md:90 | O inherits W-1..W-10 verbatim |
| M | M-001/M0-E005 | M-AGGREGATE.md:271 | Inherits F-0 W-1..W-10 + G-0 GD-* + K-0 KD-* |

**Variations:** Letter families after F treat W-10 as mandatory predecessor doctrine rather than redefining it.

---

## INV-C03 — W-2: kill-switch DISABLE only (no terminal default-off ENABLE)

**Statement:** Features that claim default-on protection must not ship with a terminal default-off ENABLE flag. Only a DISABLE kill-switch is allowed; default is ON; observable evidence runs without an env override. Explicit registered exceptions are named, not silent.

**Families:** F (owner), G, J, N, O, P, Q, B (wire-dark allowlist), H

**Per-family evidence:**

| Family | Atom ID | Aggregate anchor | Note |
|---|---|---|---|
| F | F-002/DF002-WD-011 | F-AGGREGATE.md:1075 | W-2: forbid terminal default-off; permit default-on disable-only kill switch |
| G | G-001/DG001-G0-E008 | G-AGGREGATE.md:49 | Same W-2 text |
| N | N-001/N0-E013 | N-AGGREGATE.md:117 | Kill-switch DISABLE only; default ON |
| F | F-002/DF002-F1-022 | F-AGGREGATE.md:211 | Default-off ELAI_VIBEGATE_ENABLE forbidden |
| B | B31-002 | B-AGGREGATE.md:1373 | ELAI_DISABLE_INDEXING on wire-dark-allowlist under R9 |

**Superseded / conflict notes:**

1. **F-11 / F-13 dual opt-in for self-modification:** F-001/DF001-F11-E019 @ F-AGGREGATE.md:631 calls dual opt-in a **deliberate W-2 deviation** for autonomous self-modification, not unfinished wire-dark. Target: named exception, not a silent default-off pattern.
2. **F-13 permanent ENABLE unresolved options:** F-001/DF001-F13-C005 @ F-AGGREGATE.md:168/1783 — conflict among keep-as-staging / flip-on / internal-forensics exception; silent default-off shipment forbidden. Target remains W-2 unless an explicit exception is approved.
3. **B E-47 LoRA ship gate** defaults wire-dark OFF (B0-074 @ B-AGGREGATE.md:125). Treated as feature not yet default-on, not as a shipped protection gate advertising ON.

---

## INV-C04 — Frozen gate / run cannot rewrite its judge (R8)

**Statement:** The gate judging a run is frozen and read-only to that run. Gate-manifest membership, golden locks, and frozen checkers are not mutated by the run under judgment. Intensity/policy cannot reduce frozen checkers.

**Families:** E, G, H, I, J, K, L, M, N, O, P, Q (12+)

**Per-family evidence:**

| Family | Atom ID | Aggregate anchor | Note |
|---|---|---|---|
| M | M-001/M6-E024 | M-AGGREGATE.md:615 | Golden lock in frozen gate-manifest (R8); run cannot mutate format it is judged by |
| O | O-001/O0-E045 | O-AGGREGATE.md:96 | Track inherits frozen-gate requirement |
| K | SUMMARY/Frozen judgment | K-AGGREGATE.md:3026 | Run cannot mutate its own gate-intensity table |
| L | L-002/L21-E007 | L-AGGREGATE.md:690 | Intensity can only ADD verification; no row reduces frozen checker (R8) |
| J | J-001/DJ001-G-005 | J-AGGREGATE.md:1494 | DG-3 gate_freeze_check doctrine |
| I | I-001/I28-E033 | I-AGGREGATE.md:1019 | Plan-only unit must not edit frozen gates/evaluators |
| P | P-001/P21-E019 | P-AGGREGATE.md:332 | INTEGRATION_SMOKE: whole frozen gate manifest |

**Variations:** L states intensity-add-only as the operational form of R8. M states golden-lock file membership. J names gate_freeze_check tooling.

---

## INV-C05 — Production caller / wiring reachability (W-3 / W-8 / R7)

**Statement:** Every produced symbol needs a production (non-test-only) caller. Reachability starts at a real CLI/FSM/hook/Tauri entrypoint. `#[cfg(test)]`-only callers are wire-dark. DONE requires wiring/run-trigger pass, not cargo-green alone.

**Families:** B, C, E, F, G, H, I, J, K, L, N, O, P, PHASE, Q (15)

**Per-family evidence:**

| Family | Atom ID | Aggregate anchor | Note |
|---|---|---|---|
| G | G-001/DG001-G0-E009 | G-AGGREGATE.md:51 | W-3 real entrypoint; test-only = wire-dark |
| G | G-001/DG001-G0-E014 | G-AGGREGATE.md:55 | W-8 every produced symbol needs in-plan caller |
| G | G-001/DG001-G0-E006 | G-AGGREGATE.md:47 | DONE requires full wiring gate; cargo-green insufficient |
| N | N-001/IX-009 | N-AGGREGATE.md:760 | Production reachability R7 repeated per unit |
| P | P-001/P4-E020 | P-AGGREGATE.md:134 | production_caller_check in VERIFY |
| J | J-007/J0-E011 | J-AGGREGATE.md:70 | Done = run-trigger PASS not build-green |

**Variations:** Letter indexes restate F-0/G-0 wiring doctrine. B uses WIRING_CONTRACT / DETERMINISTIC_CHECK blocks per B-N.

---

## INV-C06 — No production unwrap/expect; no production NoopSsbNotifier

**Statement:** Production paths ban `.unwrap()` / `.expect()` (and panic) outside tests and the main entry exception. Production producers may not use `NoopSsbNotifier` after the C-1/C-8 class of fixes.

**Families:** unwrap: B, C, E, F, G, H, I, J, K, N, PHASE (+ P/Q hard-rule blocks). NoopSsb: C (primary), B, E paths

**Per-family evidence:**

| Family | Atom ID | Aggregate anchor | Note |
|---|---|---|---|
| C | SUMMARY/S-E0939 | C-AGGREGATE.md:4313 | Production may not use unwrap/expect except tests and main |
| C | SUMMARY/S-E0941 | C-AGGREGATE.md:4315 | Production producers may not use NoopSsbNotifier after C-1/C-8 |
| C | C-002/C1-E19 | C-AGGREGATE.md:648 | Replace production NoopSsbNotifier with live Arc\<Ssb\> |
| B | B0-058 | B-AGGREGATE.md:109 | Part III: ort/thiserror/blake3/no-unwrap |
| G | G-002/DG002-G2-040 | G-AGGREGATE.md:457 | BLAKE3 only; no unwrap/expect on RRA paths |
| F | F-001/DF001-F0-E041 | F-AGGREGATE.md:460 | No production unwrap/expect inherited by F |
| I | (many units) | e.g. I-AGGREGATE.md:832 | Production ban unwrap + blake2 + fabricated measurements |

**Variations:** C is the schema/SSB-wiring home of NoopSsb ban. Other families restate unwrap ban in hard-rule footers.

---

## INV-C07 — Wire-dark honesty

**Statement:** Incomplete, default-off, fixture-only, or test-only wiring is labeled wire-dark (or equivalent honest state). UI and claims must not imply live connectivity or guarantees the path does not have.

**Families:** All 17 use wire-dark / W-3 / honest-state language

**Per-family evidence:**

| Family | Atom ID | Aggregate anchor | Note |
|---|---|---|---|
| G | G-001/DG001-G0-E004 | G-AGGREGATE.md:45 | Substrate shipped but mostly WIRE-DARK until activated |
| G | G-001/DG001-G0-E022 | G-AGGREGATE.md:61 | Fail-open / RunPod-pending carry honest states |
| O | O-001/O0-E032 | O-AGGREGATE.md:83 | O-4 makes wire-dark conditions visible |
| O | SUMMARY/O-4.E17 | O-AGGREGATE.md:755 | Surface visualizes wire-dark instead of silent connectivity |
| O | SUMMARY/O-4.E33 | O-AGGREGATE.md:771 | Connected node rendering wire-dark fails done-gate |
| B | B0-025 | B-AGGREGATE.md:76 | ELAI_DISABLE_INDEXING wire-dark registration |

**Variations:** O is strongest on **visual** wire-dark honesty. G is strongest on **claim** honesty for fail-open paths.

---

## INV-C08 — Authority split (harness owns gates; models generate)

**Statement:** Models generate and select. The harness sizes work, owns state, runs gates, journals, and decides completion. Controllers and tainted external agents do not sit in the host control seat.

**Families:** B, D, E, F, G, H, I, J, K, P, PHASE (11+)

**Per-family evidence:**

| Family | Atom ID | Aggregate anchor | Note |
|---|---|---|---|
| B | SUMMARY/SUM-0389 | B-AGGREGATE.md:2197 | Global harness/evidence boundaries |
| I | I-002/I002-IDX-023 | I-AGGREGATE.md:65 | Off-agent spine decides; LLM critique advisory |
| K | K-003/K27-E006 | K-AGGREGATE.md:556 | Codex output is TAINTED DATA; never controller seat; never executes on host |
| I | I-004/SD-I45-001 | I-AGGREGATE.md:405 | Orchestrator-only ownership of canonical serializer paths |
| G | G-001/DG001-G0-E001 | G-AGGREGATE.md:42 | G owns READ/EVIDENCE/RUNTIME/MEASUREMENT; composes with F WRITE gates |

**Variations:** K’s codex taint rule is a sharp instance. I’s orchestrator-only ownership is about shared serializers. Product thesis lives in overview language that B/D cite.

---

## INV-C09 — Policy / intensity asymmetry (tighten free; loosen only on measured own-run evidence)

**Statement:** Profiles and intensity tables may tighten gates freely. They may loosen only against measured false-positive evidence from that user’s own runs (defense against adaptation-as-sycophancy). Weak-lane intensity can only ADD verification relative to fixed policy.

**Families:** G, H, J, L, M, P (6)

**Per-family evidence:**

| Family | Atom ID | Aggregate anchor | Note |
|---|---|---|---|
| P | P-002/P7-E004 | P-AGGREGATE.md:447 | ASYMMETRY invariant (M-2 at process grain) |
| P | P-001/P0-E034 | P-AGGREGATE.md:81 | P-7 OperatingProfile asymmetric loosen only on measured own-run |
| L | L-002/L21-E007 | L-AGGREGATE.md:690 | Intensity can only ADD verification (R8) |
| M | SUMMARY/WF-008 | M-AGGREGATE.md:1084 | Seat may tighten; cannot weaken; raise/weaken attempt blocked |
| J | J-008/SD-J57-002 | J-AGGREGATE.md:725 | Model-vs-prompt asymmetry (auto-demote prompts; surface-only models) |

**Variations:** P/M speak profile/seat grain. L speaks verification intensity. J speaks model/prompt demotion asymmetry (related, not identical).

---

# Theme D — Budget / economics

## INV-D01 — Hard spend / cost ceilings fail closed

**Statement:** Token/spend economics include hard ceilings and retry budgets. Crossing a declared ceiling blocks or reroutes; free-lane revocation fails closed rather than open spend.

**Families:** B, C, E, H, I, J, M, N, PHASE, Q (10)

**Per-family evidence:**

| Family | Atom ID | Aggregate anchor | Note |
|---|---|---|---|
| E | E-001/DEEP-E001-E0-043 | E-AGGREGATE.md:98 | E-39 hard spend ceiling and retry budget |
| E | E-002/E39-037 | E-AGGREGATE.md:532 | Cerebras free revoke → fail closed, reroute |
| C | C-002/C5-E20 | C-AGGREGATE.md:124 | CloudPanel daily spend/cap; warn-at-80; blocked badges |
| Q | Q-001/Q8-E003 | Q-AGGREGATE.md:387 | Verifier size ceiling fail-closed BLOCK |
| N | N-002/N5-E003 area | N-AGGREGATE.md (target state) | Safety-gates > cost-ceiling > abstention > escalation |
| M | M-001/M2-E004 | M-AGGREGATE.md:398 | Pack contents include spend caps |
| B | B0-020 / StatusBar | B-AGGREGATE.md:20+ | Context-budget gauge / ContextBudgetComputed |

**Variations:** E owns token economics. C owns cloud spend UI. Q owns verifier binary/crate ceilings. N places cost-ceiling in arbitration order.

---

## INV-D02 — Context / reasoning budgets are first-class and observable

**Statement:** Context and reasoning budgets are computed, capped, and surfaced (SSB / StatusBar / ReasoningBudgetScaled). Budget exhaustion drives compaction ladders and reductions, not silent overflow.

**Families:** B, H, I, J, L, PHASE, E

**Per-family evidence:**

| Family | Atom ID | Aggregate anchor | Note |
|---|---|---|---|
| B | B0-020 | B-AGGREGATE.md:20 | context_budget_computed_snapshot + ContextBudgetComputedPayload |
| I | I-001/IG-E012 | I-AGGREGATE.md:737 | context_budget.rs: blake3 prefix stability, adaptive_reduce |
| H | H-007/H007-42-013 | H-AGGREGATE.md:702 | Pattern-2 ReasoningBudgetScaled |
| PHASE | P3A-E010 | PHASE-AGGREGATE.md:194 | token_budget_remaining for Phase 3E |

---

# Theme E — Memory / state

## INV-E01 — Machine memory and personal vault are disjoint grains

**Statement:** `memory_facts` (machine-grain SQLite / Mem0-style) and the personal markdown vault (user-grain) remain disjoint. No dual-write conversion of vault into memory_facts. Personal topics stay out of project and cloud prompts.

**Families:** O (primary), E (memory_facts), D (life-layer denylist composition)

**Per-family evidence:**

| Family | Atom ID | Aggregate anchor | Note |
|---|---|---|---|
| O | O-001/O0-E030 | O-AGGREGATE.md:81 | memory_facts machine-grain; vault user-grain |
| O | O-001/O3-E032–E033 | O-AGGREGATE.md:410-411 | Personal topics blocked from project + cloud |
| O | SUMMARY/O-3.E24–E25 | O-AGGREGATE.md:724-725 | Disjoint grains; must not dual-write |
| O | O-001/O3-E035 | O-AGGREGATE.md:413 | Converting vault into memory_facts excluded |
| O | SUMMARY/O-3.E34–E35 | O-AGGREGATE.md:734-735 | Unclaimed namespace write / user-note mutation fail done-gate |

**Variations:** O owns the vault grain. E owns memory_facts machinery. Boundary is shared.

---

## INV-E02 — Compaction and recovery must not destroy unaudited durable context

**Statement:** Pre-compaction memory flush is an ordered journaled step. Compaction must not destroy context never offered to durable memory. Recovery uses stubs + blake3 recover, not silent drop. Quarantine prefers quarantine over delete.

**Families:** P, K, L, H, PHASE (3H compaction lineage)

**Per-family evidence:**

| Family | Atom ID | Aggregate anchor | Note |
|---|---|---|---|
| P | P-003/P20-E002 | P-AGGREGATE.md:740 | Pre-compaction memory-flush ordered; “no compaction destroys context never offered to durable memory” |
| K | K-003/K12-E002 | K-AGGREGATE.md:1621 | trim→stub→recover_stubs with blake3 under lossless RECORD |
| L | L-002/L8-E005 | L-AGGREGATE.md:608 | Prune quarantines, never deletes |
| H | H-001/H35-003 | H-AGGREGATE.md:606 | Restore over seeded append-only SSB, not separate checkpoint fiction |

**Superseded note:** P-002/SD-P19-001 @ P-AGGREGATE.md:1857 **CORRECT**s summary flattening: VACUUM backups / restart continuation / compaction split-rule invariants are **Tier-2**; Tier-1 is fail-closed config lifecycle + doctor. Losing “all peer E-rows” summary wording: P19-C001 / SUP-020 / SUP-027.

---

## INV-E03 — Sealed rows and immutable instruction surface

**Statement:** Once a sequence/row is sealed, its bytes are immutable. The instruction surface (rules/skills/manifest members) is immutable for admission purposes: candidates that re-learn it are rejected (CANT-LIE / J-102).

**Families:** I, J, L, M, F

**Per-family evidence:**

| Family | Atom ID | Aggregate anchor | Note |
|---|---|---|---|
| I | I-001/I18-E017 | I-AGGREGATE.md:935 | Sealed ChainAdvanced row immutable |
| J | J-007/J0-U109 | J-AGGREGATE.md:128 | Reject candidates duplicating IMMUTABLE instruction surface |
| L | L-001/LOQF-E025 | L-AGGREGATE.md:149 | J-102 blake3 load-time admission over .md instruction surface, FAIL-CLOSED quarantine |
| F | F-002/DF002-F1-013 | F-AGGREGATE.md:202 | Immutable security rows compared by BLAKE3 digest at startup |

---

# Theme F — UI / UX conventions

## INV-F01 — Dashboard UI: Svelte 5 runes only (V2)

**Statement:** Active dashboard is V2 (`dashboard-v2`). Only Svelte 5 `$state` / `$derived` / `$effect` / `$bindable` runes. Legacy `$:` labels and `createEventDispatcher` are banned. Mount via `mount(App, { target })`. Tauri 2 event API.

**Families:** B (owner); C/J when they touch dashboard pairs

**Per-family evidence:**

| Family | Atom ID | Aggregate anchor | Note |
|---|---|---|---|
| B | SUMMARY/SUM-0004 | B-AGGREGATE.md:44 | V2 baseline invariants including runes-only |
| B | SUMMARY/SUM-0402 | B-AGGREGATE.md:2236 | Svelte 5 runes only; Tauri event API v2 |
| B | B14-E06 | B-AGGREGATE.md:647 | Token buffer $state; $effect cleanup; legacy banned |
| B | B39-052 | B-AGGREGATE.md:1844 | UI must use Svelte 5 runes |

**Variations:** This is B-family-owned. Other families inherit via frontend_pair (W-4) without restating runes.

---

## INV-F02 — Activity is an orbit; workspace is the shell

**Statement:** Activity is presented as an orbit, not a text feed. Workspace is the canonical shell. Solid CTAs excluded in favor of outlined buttons (B-12/B-13/B-14 era).

**Families:** B (primary)

**Per-family evidence:**

| Family | Atom ID | Aggregate anchor | Note |
|---|---|---|---|
| B | SUMMARY/SUM-0403 | B-AGGREGATE.md:2237 | Orbit not text feed; Workspace shell; outlined buttons |

---

## INV-F03 — Window vs cosmos data locality

**Statement:** Window state is localStorage only (no cross-device sync). Cosmos data is user-global but stored outside project repositories.

**Families:** B (primary); E/C for Cosmos stages

**Per-family evidence:**

| Family | Atom ID | Aggregate anchor | Note |
|---|---|---|---|
| B | SUMMARY/SUM-0404 | B-AGGREGATE.md:2238 | localStorage window state; Cosmos user-global outside repos |
| B | B42-E04 | B-AGGREGATE.md:1977 | Cosmos at `~/.elai/cosmos/` outside every project repo |

---

## INV-F04 — Closed enums / closed stage vocabularies on user-facing automation

**Statement:** Classifiers and canvas/workflow stages use closed enums. Free-text classifier output must not flow into prompts. Out-of-enum classification fails the done-gate. Canvas has no independent runtime; execution goes through the FSM.

**Families:** O (primary), J (closed verb sets), P (port schema), N (output boundary)

**Per-family evidence:**

| Family | Atom ID | Aggregate anchor | Note |
|---|---|---|---|
| O | SUMMARY/O.X.E03–E06 | O-AGGREGATE.md:777-780 | A1: no free-text classifier to prompt; closed enum + confidence; frozen canaries |
| O | SUMMARY/O-1.E41–E42 | O-AGGREGATE.md:650-651 | Out-of-enum / canary-to-steal fail done-gate |
| O | SUMMARY/O-4.E06–E07 | O-AGGREGATE.md:744-745 | Workflow via normal FSM; canvas no independent runtime |
| O | SUMMARY/O-4.E32 | O-AGGREGATE.md:770 | Canvas-side execution path fails done-gate |
| J | J-005/J61-E003 | J-AGGREGATE.md:773 | Closed verb set ONLY for canvas ops |
| P | P-002/P2-E013 | P-AGGREGATE.md:396 | Malformed port schema hard parse-refusal |

---

## INV-F05 — W-4 frontend pair for dashboard-surfaceable SSB

**Statement:** Any human-visible / dashboard-surfaceable SSB action requires a Svelte/Tauri frontend pair and cross_lang_diff pass (or N/A with one-line justification via W-6).

**Families:** F, G, J, O, B, C, I

**Per-family evidence:**

| Family | Atom ID | Aggregate anchor | Note |
|---|---|---|---|
| G | G-001/DG001-G0-E010 | G-AGGREGATE.md:52 | W-4 frontend pair + cross_lang_diff |
| F | F-001/DF001-F0-E013 | F-AGGREGATE.md:37 | W-4 every human-visible SSB names frontend pair |
| J | J-007/J0-E011 | J-AGGREGATE.md:70 | frontend_pair where dashboard-surfaceable |
| K | K-003/K12-E013 | K-AGGREGATE.md:437 | frontend_pair N/A justified for pure runtime gates |

---

## INV-F06 — Schema golden-lock and named bumps (W-9)

**Statement:** New SSB action types pass the golden lock, prefer Pattern-2 Kind variants, and name rather than hard-code the schema bump. Schema baseline is re-read live at edit time.

**Families:** All 17 touch schema; doctrine atoms in F, G, C, I, J, N, O, Q, M

**Per-family evidence:**

| Family | Atom ID | Aggregate anchor | Note |
|---|---|---|---|
| G | G-001/DG001-G0-E015 | G-AGGREGATE.md:56 | W-9 golden lock + Pattern-2 + named bump |
| I | I-001/IG-E007 | I-AGGREGATE.md:732 | SSB schema golden-locked |
| C | SUMMARY/S-E0930–S-E0936 | C-AGGREGATE.md:4306-4312 | Wave schema bump coordination (historical baselines) |
| Q | Q-001/Q0-E020 | Q-AGGREGATE.md:79 | Schema bump sites: baseline re-read live; never hard-code |
| M | M-001/M6-E021 | M-AGGREGATE.md:612 | Append-only field evolution locked by golden file |

**Variations / conflicts:** C records chronological ambiguity of schema 46→47 across C-1/C-27 (SUMMARY/S-A0937 @ C-AGGREGATE.md:4085). Target: live re-read of schema baseline at edit (Q0-E020 / W-9), not hard-coded wave numbers from summary snapshots.

---

# Theme G — Wiring doctrine pack (cross-cutting inheritance)

## INV-G01 — Letter families inherit F-0 W-1..W-10 (and often G-0 GD-1..GD-5)

**Statement:** Post-F letter plans treat F-0 wiring rules W-1..W-10 as mandatory predecessor doctrine. G-0 GD-1..GD-5 (compose-not-restate, recomputable metrics, claim boundary, FP-bounded default-ON, honest fail-open) are similarly inherited by H/N/M/O/P/Q.

**Families:** F (defines W-*), G (defines GD-*), H, I, J, K, L, M, N, O, P, Q

**Per-family evidence:**

| Family | Atom ID | Aggregate anchor | Note |
|---|---|---|---|
| F | F-001/DF001-F0-E011.. | F-AGGREGATE.md:430+ | W-2 staged default-off must flip on |
| G | G-001/DG001-G0-E007..E021 | G-AGGREGATE.md:47-60 | W-1..W-10 + GD-1..GD-5 |
| J | J-007/J0-E011 | J-AGGREGATE.md:70 | W-1..W-10 VERBATIM from F-0 |
| O | O-001/O0-E039 | O-AGGREGATE.md:90 | Inherits W-1..W-10 verbatim |
| M | M-001/M0-E005 | M-AGGREGATE.md:271 | F-0 W + G-0 GD + K-0 KD + MD-1..MD-5 |
| N | N-001/N0-E005 | N-AGGREGATE.md:109 | Mandatory predecessor reading before N-N |

**Variations:** M/K add letter-local MD/KD packs. Inheritance is additive, not a rewrite of W/GD.

---

## INV-G02 — Visibility filter hides harness internals from agents

**Statement:** Default `hidden_paths` / visibility filter hide model weights, rules DBs, harness internals, checkpoints, and SSB binary internals from agent-visible filesystem views.

**Families:** PHASE, B, E

**Per-family evidence:**

| Family | Atom ID | Aggregate anchor | Note |
|---|---|---|---|
| PHASE | P0-E017 | PHASE-AGGREGATE.md:64 | Hides `.elai_cc/models/**` via default hidden_paths |
| PHASE | P0-E056 | PHASE-AGGREGATE.md:103 | allowed_paths + hidden_paths; hide models, rules.db, harness, checkpoints, SSB |
| E | SUMMARY/E-37.F08 | E-AGGREGATE.md:6387 | Visibility filter + MCP deny-list are control-plane defenses, not sufficient data-plane defenses |

**Variations:** E warns visibility is control-plane only (not data-plane complete). Still a shared default-hide invariant.

---

# Superseded notes register (cross-family disputes)

| ID | Losing wording | Winner / target | Precedence basis | Anchors |
|---|---|---|---|---|
| SUP-X01 | Non-RFC hashing with blake2s / blake2 (PHASE-0/1A/1C, some phase-4C text) | blake3 only for non-RFC | Explicit supersession B16-014; later E/G hard rules | PHASE P0-E015:62, P1C-E009:139, P1A-E035:533; B B16-014:730, B19-015:830; E E0-062:117 |
| SUP-X02 | SocratiCode-style SHA-256 for artifact identity | blake3 under anti-stale rule | Later B-35 correction | B B35-006:1600 |
| SUP-X03 | “Replay is best-effort / not deterministic” as acceptable target (E-29 clock/RNG gaps) | Controlled providers + fail-closed frozen predicates | Later determinism doctrine + fail-closed | E E004.SD29.CF003:1237 vs INV-B02/B03/C01 |
| SUP-X04 | P-19 summary flattens VACUUM backups / restart continuation / compaction invariants as peer Tier-1 | Tier-2 deferred; Tier-1 = fail-closed config lifecycle + doctor | Explicit CORRECT delta SD-P19-001 | P SD-P19-001:1857; SUP-020/027 |
| SUP-X05 | Held-out “append-only” vs TOML status rewrite (D-6) | SSB/history no-delete remains; retirement status rewrite is separate store semantics | Conflict AC noted; no-delete of evidence retained | D D-001/AC-008:406; D0-041:74 |
| SUP-X06 | Hard-coded schema wave numbers (C summary 46→47 chronology) | Live re-read of schema baseline; named bumps (W-9) | Later Q/G doctrine | C S-A0937:4085; Q Q0-E020:79; G G0-E015:56 |
| SUP-X07 | Silent terminal default-off ENABLE as shipping pattern | W-2 kill-switch DISABLE only; named exceptions only | F-0 W-2; explicit F-11/F-13 exception handling | F WD-011:1075; F11-E019:631; F13-C005:168 |
| SUP-X08 | B34 project `[entry_points]` append-only as firm config | Decision-doc recommendation only | Ambiguity atom overstates summary | B B34-042:1578 |

**Superseded notes count:** 8

---

# Closing audit

| Metric | Value |
|---|---|
| Families covered | **17/17** (PHASE, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q) |
| Shared invariant records | **32** (INV-A01..A06, B01..B06, C01..C09, D01..D02, E01..E03, F01..F06, G01..G02) |
| Themes | **7** — Integrity/evidence; Determinism/replay; Safety/gating; Budget/economics; Memory/state; UI/UX conventions; Wiring doctrine pack |
| Superseded notes | **8** |
| Inputs mutated | **none** (read-only) |
| Writes | this file only |

## Theme list (short)

1. Integrity / evidence  
2. Determinism / replay  
3. Safety / gating  
4. Budget / economics  
5. Memory / state  
6. UI / UX conventions  
7. Wiring doctrine pack  

## Coverage caveats (honest)

- **Universal (17/17 keyword presence):** fail-closed, determinism/replay, blake3, SSB schema language, wire-dark/kill-switch family.
- **Near-universal (14–16):** append-only journal, ELAI_SEED/providers, production caller, W-10 tier split.
- **Doctrine-cluster (letter F+):** W-1..W-10 and GD-1..GD-5 are defined once and **inherited**; earlier PHASE/B/C/D often embody the same rules under different names (DETERMINISTIC_CHECK, ClockProvider, media fail-closed) without the W-/GD- labels.
- **Narrow but shared:** personal vault grain (O-primary), Svelte-5 runes (B-primary), journal COMPLETENESS RECONCILE-MISSING (K-primary) — shared when other families compose with them, not re-authored everywhere.

---

*Generated from family aggregates only. Does not assert shipping/wiring truth beyond what the aggregates record.*
