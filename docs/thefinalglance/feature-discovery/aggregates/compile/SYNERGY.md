# Synergy, shared invariants, and status conflicts

Companion to `HEAD.md` for the ELAI exhaustive super feature list. Distilled from:

- `docs/thefinalglance/feature-discovery/aggregates/CROSS-FAMILY-INTERACTION-MAP.md`
- `docs/thefinalglance/feature-discovery/aggregates/CROSS-FAMILY-SHARED-INVARIANTS.md`
- `docs/thefinalglance/feature-discovery/aggregates/CROSS-FAMILY-STATUS-CONFLICTS.md`

Generated 2026-08-12. Inputs read-only.

---

## How the systems compose (synergy)

Cross-family composition is directed: **consumer family feature → provider feature** when an aggregate atom names a feature owned by another family. Map totals: **3010** raw mention edges, **1670** unique (consumer, provider, relation) triples, **881** structural pairs (consumer family → provider feature), **115** hub systems (provider referenced by ≥3 families), **426** distinct external provider feature IDs.

### Hub systems (load-bearing substrates)

Changing a hub without updating dependents is a system break. Below: primary hubs named in product vocabulary, each with composition narrative, consumer families, and sample evidence anchors. Full hub id list (115) is in the interaction map closing audit.

#### `SSB` — session event bus / append-only journal (phase-3B backbone)

**10 families · 43 raw edges.** PHASE 3B defines typed `action_type` + single-writer SQLite-WAL journal. Later families journal measured rows; schema bumps per letter. Coordination required around shared edit windows and prefer-reuse row budgets.

- **Consumers:** G, H, I, J, K, L, M, N, P, Q (plus PHASE backbone).
- **Evidence:** `G-002/IX-015` @ G-AGG:L1236; `H-001/IX-013` @ H-AGG:L1391; `I-001/IX-007` @ I-AGG:L1163; `J-004/IX-019` @ J-AGG:L2946; `K-001/IX-008` @ K-AGG:L1194; `L-001/IX-017` @ L-AGG:L1707; `M-001/IX-016` @ M-AGG:L665; `N-001/IX-012` @ N-AGG:L763; `P-001/IX-013` @ P-AGG:L1684; `Q-002/IX-015` @ Q-AGG:L828.
- **Known gap:** phase-4D `research_*` types vs 3B 17-type CHECK (`PHASE-007/IX-017`; CF-006c).

#### `B-37` — RuntimeAdapter multi-backend substrate

**9 families · 67 raw edges.** One runtime abstraction over multi-backend routing (cloud/local/CLI/process isolation). Per-role hybrid routing and fallback chains feed almost every later agent surface.

- **Consumers:** C, D, E, I, J, K, L, N, P.
- **Evidence:** `C-005/C44-E02` @ C-AGG:L2890; D14-E27 @ D-AGG:L746; `E-006/E006-E24-002` @ E-AGG:L1656; `I-001/IX-014` @ I-AGG:L1170; `J-009/IX-005` @ J-AGG:L4820; `K-003/K24-E026` @ K-AGG:L526; `L-002/L0-E007` @ L-AGG:L438; `N-002/N13-E006` @ N-AGG:L611; `P-003/IX-005` @ P-AGG:L1724.

#### `G-2` — RRA / replay-verify evidence plane

**9 families · 63 raw edges.** Replayable run artifact (RRA) seal + `replay --verify`. Downstream units extend cassettes, retain verification, or consume the bundle for proof spines.

- **Consumers:** H, I, J, K, L, M, O, P, Q.
- **Evidence:** `H-004/IX-014` @ H-AGG:L2889; `I-006/I43-E013` @ I-AGG:L3551; `J-008/IX-015` @ J-AGG:L4450; `K-001/IX-009` @ K-AGG:L1195; `L-002/L7-E008` @ L-AGG:L578; `M-001/M5-E029` @ M-AGG:L131; `P-003/IX-011` @ P-AGG:L1730; `Q-001/Q5-E006` @ Q-AGG:L343.
- **Note:** G-5 may DONE on SSB-only path with RRA optional (CF-020).

#### `G-1` — taint lattice (untrusted IN-flow)

**7 families · 43 raw edges.** Untrusted tag on external bytes at ingest. Mirror on OUT-flow privacy (P-6) shares N-6’s ONE lattice (N-2 forbids a second scale).

- **Consumers:** H, J, K, L, N, O, P.
- **Evidence:** `H-002/HTG-010` @ H-AGG:L1531; `J-008/IX-015` @ J-AGG:L4450; `K-001/K20-E004` @ K-AGG:L180; `L-002/L1-E016` @ L-AGG:L519; `N-002/N6-E002` @ N-AGG:L492; `P-002/IX-004` @ P-AGG:L1698.

#### `F-4` — cant-lie / PROVEN case gate (stdout_hash, temporal gate)

**7 families · 41 raw edges.** Anti-fabrication: orchestrator-captured BLAKE3 stdout hash; temporal gate. Attestation (G-8) rolls up recorded verdicts. Do not rebuild as a later monitor (J-66 NASA map).

- **Consumers:** G, H, I, J, K, L, P.
- **Evidence:** `G-002/IX-009` @ G-AGG:L1230; `H-005/H005-52-001` @ H-AGG:L826; `I-004/I45-E012` @ I-AGG:L2757; `J-005/IX-001` @ J-AGG:L3340; `K-002/K3-E014` @ K-AGG:L325; `L-002/L0-E018` @ L-AGG:L449; `P-003/P15-E006` @ P-AGG:L688.
- **Depends on:** C-28 (`SUMMARY/F4-19` @ F-AGG:L1249).

#### `F-1` — VibeGate / Layer-1 external review gate family

**6 families · 26 raw edges.** Multi-axis PreToolUse / write-time external review. Shared secret-pattern detector with G-9 redaction. J-23 extends as external Layer-1 backends; N-6 admission reuses F-1 legs.

- **Consumers:** G, H, I, J, L, N.
- **Evidence:** `G-002/IX-011` @ G-AGG:L1232; `H-005/H005-52-033` @ H-AGG; `I-006/IX-003` @ I-AGG; `J-006/IX-015` @ J-AGG; `N-002/N6-E007` @ N-AGG:L125.

#### `F-5` — quarantine / flaky isolation

**6 families · 29 raw edges.** Quarantine-on-flaky and staleness isolation. Feeds I-12 extension chain, L prune substrate, P/Q measured paths.

- **Consumers:** I, J, L, N, P, Q.
- **Evidence:** `I-004/IX-008` @ I-AGG:L550; L CGG TS-14 F-5 WIRED; `Q-002/Q1-E012` (Q-1←F-5).

#### `C-28` / `C-29` — mutation validation + intent / planning-mutation gates

**C-28:** mutation validation (F-4 depends; I-12 chain; J). **C-29:** intent / metamorphic companion (E, I, J, L). Together they are the mutation honesty front door for F-4, I-12 curator, and L-5 counterfactual do-test.

- **Evidence:** `SUMMARY/F4-19`; `I-004/IX-008`; `I-001/IX-017`; `E-005/E005-E13-002`; `J-009/IX-006`.

#### `J-16` — blast-radius analysis (hub-adjacent)

Fan-in lower in automated ID extraction because many units say “blast radius” without the id. Named substrate from **K** (`K-002/K3-E014`: F-4, G-8, **J-16 when present**, G-2) and **L-5** prereq (`L-003/L5-E006`; live plan notes J-16 WIRED in `L-003/L5-C001`). Adjacent to B-29 / C-24 blast-radius UI and G-3 BLAST-RADIUS mediation.

#### Additional high-fan-in hubs (short)

| Hub | Families | Role | Consumers (map) |
|---|---:|---|---|
| `D-13` | 7 | Cross-family adversary / independence principle | E, H, I, J, K, L, Q |
| `B-40` | 6 | Research report / framing + ResearchReport types | C, E, I, J, O, Q |
| `H-49` | 5 | Retirement / RV snapshot + durable memory | I, J, L, N, P (CAS/RV-1 SHIPPED for consumers; CF-038) |
| `B-29` | 5 | Blast-radius / decomposition UI (closed by C-24) | C, E, J, K, L |
| `G-9` | 5 | Redaction / privacy export | I, J, K, M, P |
| `D-16` | 5 | Refusal / calibration (RefusalDecision) | H, I, J, L, N |
| `H-9` | 5 | Spend ledger / cost accounting | I, J, K, M, Q |
| `G-8` | 3+ | Attestation sign/verify | J, K, M |
| `G-10` | 4 | Model-drift canary / invalidation | J, K, N, P |
| `N-6` | (lattice) | ONE provenance/trust lattice + admission | P ports/privacy; J/L admission family |
| `O-2` | 2+ | TaskSpec + Cron/SsbEvent triggers | P, Q |
| `I-25` | 4 | Post-wake router / intervention-card inbox | J, K, N, Q |
| `I-11` | 4 | BeliefState (PARTIAL live condense; CF-039) | J, K, L, Q |
| `I-12` | 4 | Curator (must not reimplement PHASE 3F lifecycle) | J, L, N, PHASE |
| `J-28` | 3 | usage_turn_event cost ledger rows | K, P, Q |
| `J-96` | 3 | CaseNode / MintOnProof | L, N, P |
| `E-48` | 4 | VerifiedCitation | H, J, P, Q |
| `K-24` | 2+ | BCC billing/credits (gated by M-1 tier) | M, N |
| `K-2` | 3 | Routing-economics / shard harness | K, P, Q paths |

### Six multi-hop composition chains

#### 1. Trust spine on every external byte

```
external ingest (P-9/P-10/P-12/P-22)
  → G-1 taint Untrusted
  → N-6 lattice tier (N-2 forbids second scale)
  → G-2 RRA / SSB journal
  → G-9 redaction on export (M-3/M-4/P-3)
  → F-4 / F-5 case+quarantine when claims enter reuse
```

Evidence: `P-002/IX-004`, `P-002/IX-008`, `M-001/IX-013`, `Q-002/Q1-E012`.

#### 2. Mutation / claim honesty stack

```
C-28 mutation gate + C-29 intent
  → F-4 cant-lie / PROVEN
  → I-12 curator (must not reimplement 3F SM)  [PHASE-005/IX-006]
  → L-5 counterfactual do-test (prereqs C-29, J-16, F-4)
  → J-96 CaseNode / MintOnProof → P-15 reuse → K-18 hit-rate
```

Evidence: `SUMMARY/F4-19`; `I-004/IX-008`; `L-003/L5-E006`; `P-003/IX-003`.

#### 3. Org commerce over seat trust

```
G-8 attestation + G-2 verify + K-6 export shape
  → M-1 tier()  [gates K-24 credits, M-2 packs, M-3 report]
  → M-3 org rollup (extends K-6; J-59 isolation)
  → M-5/M-6 open verifier + golden-lock (Q-8 ceiling consumes M)
```

Evidence: `M-001/IX-004`, `M-001/IX-008`, `M-001/IX-011`; `Q-001/IX-006`.

#### 4. Orchestration economics (Q over O/I/J/H/K)

```
O-1 daemon / O-2 TaskSpec wake sites
  → Q-2 cross-family wake quorum (D-13 principle, J free-API voters)
  → I-25 router / intervention cards
  → Q-3 advisor (H-7 split, I-11 BeliefState, H-9 spend, K-2 arm)
  → Q-4 model swap (F-6 trust, 4G/4M, J rotation)
  → J-28 usage rows + SSB measured-number idiom (GD-2)
```

Evidence: `Q-002/IX-004`, `SUMMARY/Q2-20`, `SUMMARY/Q4-17`, `Q-002/IX-009`, `Q-002/IX-014`.

#### 5. Dashboard closes B/phase; later families extend C surfaces

```
B-* dashboard/runtime gaps
  → C-* closes (many SUMMARY S-E* "Closes B-N / phase-*")
  → D-26/D-27 dashboards reuse C-3/C-6/C-7 + SSB
  → H/I/J drawers and harness panels ride C IPC + B-37 RuntimeAdapter
```

Evidence: C-AGGREGATE SUMMARY closes rows; D26-003 depends C-3/C-4/C-6/C-7; B-37 hub fan-in.

#### 6. PHASE 3B SSB as universal journal

```
phase-3B typed action_type + single-writer SSB
  → almost every later family journals measured rows
  → schema bumps per family (D, J, M entitlement, Q Goal*/Wake*)
  → conflict: phase-4D research_* types vs 3B 17-type CHECK
```

Evidence: `PHASE-007/IX-009`, `PHASE-007/IX-017`; per-family SSB budget IX atoms.

### Reverse index summary

Providers ranked by distinct consumer families (top of map reverse index):

1. **SSB** (10) · 2. **B-37** (9) · 3. **G-2** (9) · 4. **D-13** (7) · 5. **G-1** (7) · 6. **F-4** (7) · 7. **F-5** (6) · 8. **F-1** (6) · 9. **B-40** (6) · 10. **H-49** (5) · then B-29, G-9, D-16, H-9, E-54, I-27, G-6, D-11, D-6, B-22, H-7, G-10, …

Heaviest **consumer** structural pair counts: J (178), N (131), L (90), I (79), K (75), E (63), P (63), C (58). Coarse family-to-family adjacency matrix lives in the interaction map § Family-to-family adjacency.

Named load-bearing gates (product vocabulary): F-4, C-28, C-29, G-1, G-2, SSB, J-16, N-6, O-2, I-25, B-37, D-13, K-24, G-8, F-5.

---

## Shared invariants

**34** invariant records across **7** themes (INV-A01..A06, B01..B06, C01..C09, D01..D02, E01..E03, F01..F06, G01..G02; source footer says 32 by arithmetic error on the same id ranges). Shared means asserted as a system rule in **two or more** families with atom-level anchors. Wording compressed; every INV id retained. Full per-family evidence tables: `CROSS-FAMILY-SHARED-INVARIANTS.md`.

### Theme A — Integrity / evidence

- **INV-A01 — Off-agent evidence is required.** A gate measures an artifact the generator did not write. Model self-report is not enough. Families: B, D, E, F, H, I, J, K, L, N, O, P, Q (13). Anchors e.g. `SUMMARY/SUM-0389` @ B:2197; `P-001/P4-E020` @ P:134; `Q-002/Q6-E019` @ Q:140.
- **INV-A02 — Missing evidence is FAIL (never SKIP).** Absence fails the gate (HALT / UNVOUCHED / never silent clean). Families: B, G, H, J, K, M, N, O, P, Q (+ fail-closed wording in C,D,E,F,I,L,PHASE). Anchors: `H-006/H006-H29-008`; `K-001/K0-E027`; `M-001/M6-E033`; `O-001/O0-E041`.
- **INV-A03 — Author emits; auditor enforces (no self-certify).** Generating path proposes; independent auditor/checker enforces. Families: D, F, H, I, Q (+ B/G/J wiring auditor + W-1). Anchors: `D12-006`; `SUMMARY/F6-19`; `I-002/I002-IDX-023`.
- **INV-A04 — Claims discipline (measured or tagged).** Superlative/external claims need journal-recomputable pointer or `[EXTERNAL]`/`[DESIGN]`. Families: E, G, H, I, J, K, L, M, N, Q. Anchors: `K-001/K1-E013`; G GD-3; `N-002/N8-E020`.
- **INV-A05 — CANT-LIE / anti-fabrication.** No fabricated measurements/anchors/clean verdicts; admission rejects inventing instruction-surface content. Families: B, C, E, F, G, H, I, J, K, L, N, P, PHASE, Q. Anchors: `I-001/I8-E019`; `J-007/J0-U109`; `SUMMARY/SUM-0391`.
- **INV-A06 — Journal completeness (observed effect needs a row).** Covered side-effects mediated or observed; missing row = DETERMINISTIC FAIL; dropped-event counters fail closed. Families: K primary; G/H/N/O emission doctrine. Anchors: `K-003/K21-E002`, `K21-E006`; `O-001/O0-E020`; `G-001/DG001-G0-E007`.

### Theme B — Determinism / replay

- **INV-B01 — Append-only SSB journal (no UPDATE / no DELETE).** Sealed history immutable; schema golden-locked; evolution is append-only growth. Families: B, C, D, E, G, H, I, J, K, L, M, N, O, PHASE, Q (15). Anchors: `G-001/DG001-G0-E003`; `I-001/IG-E007`; `L-002/L8-E005` (quarantine never DELETE). D held-out TOML status rewrite is a separate store, not SSB mutation.
- **INV-B02 — Controlled time and randomness (ELAI_SEED / providers).** Injected ClockProvider / SeededRng; `ELAI_SEED` pins RNG. Families: B–N, P, PHASE, Q (16). Anchors: `P0-E060`; `C-006/C15-E009`; `D5-028`; `B39-065`. Note: SeededClock tick starts at 0 regardless of ELAI_SEED (`C15-E004`).
- **INV-B03 — GD-2: metrics recomputable from journal under seed.** Human/gate-facing numbers recomputable from SSB under pinned seed or wire-dark. Families: G, H, N owners; J, K, L, Q echo. Anchors: `G-001/DG001-G0-E019`; `N-001/N0-E024`.
- **INV-B04 — Byte-identical / hash-checkable replay where claimed.** Non-clock fields byte-identical or hash-checkable; network detached on cassette replay. Families: all 17 mention; strong in B, C, E, F, G, H, I, J, K, O, PHASE. Anchors: `O-001/O1-E079`; `C-006/C15-E026`; `I-005/DII005-I44-016`. E-29 “best-effort” is losing wording vs fail-closed frozen predicates.
- **INV-B05 — BLAKE3-only content hashing (non-RFC).** blake2/sha2 forbidden for identity/staleness/fingerprints/gate digests. Families: all 17; explicit ban in B, D, E, F, G, H, I, J, L, N, O, P, Q, PHASE. Anchors: `E-001/DEEP-E001-E0-062`; `B16-014`; `G-001/DG001-G0-E042`. Supersedes early PHASE blake2 and B-16A blake2s.
- **INV-B06 — Idempotent durable operations.** Re-run without content change does not rewrite or double-fire. Families: B, C, D, E, F, G, H, I, J, O, P, PHASE. Anchors: `B42-E09`; `D6-042`; `O-001/O2-E076`; `P-001/P0-E023`.

### Theme C — Safety / gating

- **INV-C01 — Fail-closed is the default gate posture.** Required check fails → block/refuse. Fail-open paths must not advertise fail-closed. Families: all 17.
- **INV-C02 — W-10: Tier-1 in-process fail-closed vs Tier-2 gated.** Network/subprocess/RunPod axes split; only Tier-1 is fail-closed done-gate. Families: F, G, H, I, J, K, L, M, N, O, P, Q, PHASE, B (14+). Anchor: `G-001/DG001-G0-E016`.
- **INV-C03 — W-2: kill-switch DISABLE only (no terminal default-off ENABLE).** Default-on protection uses DISABLE kill-switch only; named exceptions only (e.g. F-11 dual opt-in). Families: F, G, J, N, O, P, Q, B, H. Anchors: `F-002/DF002-WD-011`; `N-001/N0-E013`.
- **INV-C04 — Frozen gate / run cannot rewrite its judge (R8).** Gate-manifest, golden locks, frozen checkers read-only to the judged run; intensity may only ADD verification. Families: E, G, H, I, J, K, L, M, N, O, P, Q. Anchors: `M-001/M6-E024`; `L-002/L21-E007`; `J-001/DJ001-G-005`.
- **INV-C05 — Production caller / wiring reachability (W-3 / W-8 / R7).** Every produced symbol needs a non-test production caller from real entrypoint; DONE ≠ cargo-green alone. Families: B, C, E, F, G, H, I, J, K, L, N, O, P, PHASE, Q (15). Anchors: `G-001/DG001-G0-E009`, `G0-E014`, `G0-E006`; `P-001/P4-E020`.
- **INV-C06 — No production unwrap/expect; no production NoopSsbNotifier.** Unwrap/expect banned outside tests/main; NoopSsbNotifier banned after C-1/C-8 class. Families: unwrap B,C,E,F,G,H,I,J,K,N,PHASE (+P/Q); NoopSsb C primary. Anchors: `SUMMARY/S-E0939`, `S-E0941`; `C-002/C1-E19`.
- **INV-C07 — Wire-dark honesty.** Incomplete/default-off/test-only wiring labeled wire-dark; UI must not imply live connectivity. Families: all 17. Anchors: `G-001/DG001-G0-E004`; `O-001/O0-E032`; SUMMARY O-4.E17/E33.
- **INV-C08 — Authority split (harness owns gates; models generate).** Models generate; harness sizes work, owns state, runs gates, journals, decides completion. Families: B, D, E, F, G, H, I, J, K, P, PHASE. Anchors: `I-002/I002-IDX-023`; `K-003/K27-E006` (Codex output TAINTED DATA).
- **INV-C09 — Policy / intensity asymmetry.** Tighten freely; loosen only on measured own-run false-positive evidence; weak-lane intensity only ADDs verification. Families: G, H, J, L, M, P. Anchors: `P-002/P7-E004`; `L-002/L21-E007`; `SUMMARY/WF-008` @ M.

### Theme D — Budget / economics

- **INV-D01 — Hard spend / cost ceilings fail closed.** Crossing ceiling blocks/reroutes; free-lane revoke fails closed. Families: B, C, E, H, I, J, M, N, PHASE, Q. Anchors: `E-001/DEEP-E001-E0-043`; `C-002/C5-E20`; `Q-001/Q8-E003`.
- **INV-D02 — Context / reasoning budgets are first-class and observable.** Computed, capped, surfaced (SSB/StatusBar); exhaustion drives compaction ladders, not silent overflow. Families: B, H, I, J, L, PHASE, E. Anchors: `B0-020`; `I-001/IG-E012`; `H-007/H007-42-013`.

### Theme E — Memory / state

- **INV-E01 — Machine memory and personal vault are disjoint grains.** `memory_facts` vs personal markdown vault; no dual-write; personal topics out of project/cloud prompts. Families: O primary; E; D denylist. Anchors: `O-001/O0-E030`; `O3-E032–E035`.
- **INV-E02 — Compaction and recovery must not destroy unaudited durable context.** Pre-compaction flush ordered/journaled; recover via stubs+blake3; quarantine > delete. Families: P, K, L, H, PHASE. Anchors: `P-003/P20-E002`; `K-003/K12-E002`; `L-002/L8-E005`.
- **INV-E03 — Sealed rows and immutable instruction surface.** Sealed bytes immutable; instruction surface admission rejects re-learning (CANT-LIE / J-102). Families: I, J, L, M, F. Anchors: `I-001/I18-E017`; `J-007/J0-U109`; `L-001/LOQF-E025`.

### Theme F — UI / UX conventions

- **INV-F01 — Dashboard UI: Svelte 5 runes only (V2).** `$state`/`$derived`/`$effect`/`$bindable` only; no legacy `$:`; Tauri 2 events. Families: B owner; C/J on pairs. Anchors: `SUMMARY/SUM-0004`, `SUM-0402`.
- **INV-F02 — Activity is an orbit; workspace is the shell.** Orbit not text feed; outlined buttons over solid CTAs. Families: B. Anchor: `SUMMARY/SUM-0403`.
- **INV-F03 — Window vs cosmos data locality.** Window state = localStorage only; Cosmos user-global outside project repos (`~/.elai/cosmos/`). Families: B; E/C stages. Anchors: `SUMMARY/SUM-0404`; `B42-E04`.
- **INV-F04 — Closed enums / closed stage vocabularies.** No free-text classifier into prompts; out-of-enum fails done-gate; canvas has no independent runtime. Families: O, J, P, N. Anchors: SUMMARY O.X.E03–E06; `J-005/J61-E003`.
- **INV-F05 — W-4 frontend pair for dashboard-surfaceable SSB.** Human-visible SSB action needs Svelte/Tauri pair + cross_lang_diff (or N/A + W-6 justification). Families: F, G, J, O, B, C, I. Anchors: `G-001/DG001-G0-E010`; `F-001/DF001-F0-E013`.
- **INV-F06 — Schema golden-lock and named bumps (W-9).** New action types pass golden lock; Pattern-2 Kind preferred; named bump; re-read baseline live (never hard-code wave numbers). Families: all 17 touch schema; doctrine in F, G, C, I, J, N, O, Q, M. Anchors: `G-001/DG001-G0-E015`; `Q-001/Q0-E020`; `M-001/M6-E021`.

### Theme G — Wiring doctrine pack

- **INV-G01 — Letter families inherit F-0 W-1..W-10 (and often G-0 GD-1..GD-5).** Post-F letters treat W-* and GD-* as mandatory predecessor doctrine. Families: F, G, H, I, J, K, L, M, N, O, P, Q. Anchors: `J-007/J0-E011`; `O-001/O0-E039`; `M-001/M0-E005`.
- **INV-G02 — Visibility filter hides harness internals from agents.** Default `hidden_paths` hide models, rules DB, harness, checkpoints, SSB internals. Families: PHASE, B, E. Anchors: `P0-E017`, `P0-E056`; E-37.F08 (control-plane only, not data-plane complete).

Cross-family superseded notes on invariant disputes (blake2→blake3, best-effort replay, P-19 tier flatten, D-6 TOML, hard-coded schema waves, silent default-off ENABLE, B34 entry_points firmness): 8 SUP-X* rows in the invariants source.

---

## Status conflicts and resolutions

**43** top-level conflict records (CF-001..CF-043). CF-004 expands to 16 tier sub-rows (004a..004p); CF-006 to 4 phase sub-rows (006a..006d). **10** residual ambiguities (RA-001..RA-010) marked **UNRESOLVED**. Precedence: explicit SUPERSEDED/measurement/VETO > later letter > higher plan number; deep > summary; losers retained.

### CF-001..CF-043 (compact)

**CF-001 — Q-14 E-1 novelty canary.** SHIPPED 2026-07-18 (deep: canary + fail-closed instrument + `--self-test-novelty-canary`) vs summary planned inventory + coarse file header PLANNED. **RESOLVED — deep SHIPPED wins** (per-sub-unit banners). Atoms: `Q-001/Q14-E002/E003`; SUP-001/005. Loser: summary planned; header STATUS word.

**CF-002 — Q-15 S-1 GEPA policy arm.** REJECTED-by-measurement 2026-07-17 vs summary live selectable arm. **RESOLVED — REJECTED is target.** Live remaining: S-2,S-3,S-4,S-6. Atoms: `Q-001/Q15-E001`.

**CF-003 — K-4 gate-tax design.** Summary ON/OFF SAFEDELTA ablation + wire-dark inventory as scope + single over-block number vs refined **VETO** of product gates ON/OFF + FIX-1 ban wire-dark as scope source + two labeled numbers. **RESOLVED — refined VETO/FIX is target.** Atoms: `K-004/K4-E012..E018`, SD-K4-001..004.

**CF-004 — P-family Tier-1 vs Tier-2 corrections** (deep REVISED over summary peer-flattening; P newer than B..O):

| Sub | Feature | Winner Tier-1 | Loser → Tier-2 / note |
|---|---|---|---|
| 004a | P-15 case-graph | gated retrieve/distill/fuse + reuse-hit | bBoN, narrative/episodic, embedding |
| 004b | P-20 memory lifecycle | pre-compaction flush + on-demand skill/rules index | Dreaming, cache mapping |
| 004c | P-22 channel hardening | fenced child returns + depth-scoped tool-strip | pairing codes, dmScope, device pairing, etc. |
| 004d | P-12 desktop/GUI | cua sandbox + frame-bind + arm/lease + recording | OmniParser Tier-3 (with P-13) |
| 004e | P-13 perception SoM | `elai perception scaffold` + PRODUCTION_HOOK | Tier-3 pixel/YOLO (with P-12) |
| 004f | P-18 TaskSpec exec | revision-counter + sticky-cancel + drift-binding | retry/watchdog/due-time → Tier-2/RECON |
| 004g | P-21 gate-credibility | REVISED intent: skipBackfill/redaction/posture → Tier-2 | **same-file wiring still requires them → RA-001** |
| 004h | P-1 fleet health | Smoke + DepRotCheck + N-7 diagnosis | DriftCanary/auto-repair core claims |
| 004i | P-2 inter-tool ports | CLI registry/wire/SSB only | O-4 canvas edge, M-5 mount as core |
| 004j | P-6 privacy lattice | O-3 vault-read only | dual O-3 AND E-20; E-20 → Tier-2 |
| 004k | P-7 operating profile | Manual asymmetry-guarded change | Automatic N-3 learning loop as core |
| 004l | P-14 crystallize | E-14 browser Playwright only | PyAutoGUI peer emitters |
| 004m | P-19 config/ops | Fail-closed config + doctor | VACUUM backups, restart continuation, etc. as peer |
| 004n | P-3 persona export | Tier-1 without persona regen | Full import+regenerate as Tier-1 |
| 004o | P-8 multi-seat | Two-seat kernel Tier-1 | “2-10 scale” as immediate |
| 004p | P-16 install catalog | “hidden-until-verified” gate → Tier-2 | Summary admission family phrasing |

**004a–f, h–p RESOLVED — deep REVISED Tier-1 wins.** **004g partial → RA-001.**

**CF-005 — J-80 worktree pool doc-drift.** Callers actually in `task_runner.rs` (not bsp/engine prose); dep = `fd-lock` (not sysinfo); H-49 CAS/RV-1 SHIPPED for consumers; I-2 → PARTIAL on map. **RESOLVED — recon/wargame corrections win.** Atoms: `J-010/J80-C001..C003`.

**CF-006 — PHASE-007 dep-graph and payload types.**  
- **006a** 1I missing 1G→1I edge → **RESOLVED: ACTION-REQUIRED dep-graph fix.**  
- **006b** SsbPayload “15 variants” vs 17 action_types → **RESOLVED: 17 CHECK schema controls.**  
- **006c** 4D research_* not in 3B 17-type CHECK → **RESOLVED as known schema gap** (extend before journal).  
- **006d** 2D methods vs SourceMatch-only until 4F → **RESOLVED: SourceMatch-only interim.**

**CF-007 — Phase 4J empire_* events.** Five variants (not singular `empire_event`); test target orchestrator `--features empire`. **RESOLVED — PRE-RESOLVED DECISION wins.** Atoms: `P4J-E004`, `P4J-C001/C002`.

**CF-008 — Q-10 B-CREATIVITY-0 ladder.** VALID REFUTED 2026-07-14 vs open Tier-0 authorization language. **RESOLVED — VALID REFUTED halt binding.** Atoms: `Q-002/Q10-E001`; SUP-003/006.

**CF-009 — Q-6 OOM/I-O injection tier.** Tier-1 = file corruption only; OOM/I-O Tier-2 vs summary first-class. **RESOLVED — Tier-1 file corruption.** Atoms: `Q-002/Q6-E005`.

**CF-010 — B-31 resumable indexing.** B-0 reconciliation DONE vs older PLANNED. **RESOLVED — DONE.** Atoms: `B0-025`; SUP-002/004/009.

**CF-011 — B-38 ambiguity-gate.** SHIPPED (schema 42→43) vs earlier PLANNED. **RESOLVED — SHIPPED.** Atom: `B0-028`.

**CF-012 — B-20 failure categories.** Twelve categories (2026-05-23 rebaseline + C-33) vs original nine. **RESOLVED — twelve canonical.** Atoms: `B20-001`.

**CF-013 — B-39 REFINE_CAP.** Shipped/contract = 3 vs most text cap 2. **RESOLVED — 3 current canonical.** Atom: `B39-037`.

**CF-014 — B-12 dashboard shell IA.** Single Workspace + IconRail vs Tier-0 five-icon model. **RESOLVED — single Workspace / Tier-0 retired.** Atoms: `B12-002` etc.

**CF-015 — D-28 adversarial farm.** Shipping-pass host live (`AdversarialFarmRunCompleted`) vs header DEFERRED. **RESOLVED for host legs — shipping-pass wins;** training/deploy remain deferred.

**CF-016 — D-29 hierarchical BAI.** Path A host Rust shipped; Gibbs/RunPod DEFERRED vs clean DEFERRED only. **RESOLVED as mixed** — not full DONE.

**CF-017 — D-30 calibration bootstrap.** Rust dispatch live; corpus/data/D22 DEFERRED vs blanket DEFERRED. **RESOLVED as partial.**

**CF-018 — F-10 self-mod leg.** Intentional wire-dark until F-11 vs summary all-three-loop-legs done. **RESOLVED — wire-dark until F-11.** F-AGG SUP-019.

**CF-019 — G-1/G-4/G-5/G-7 header PLANNED vs shipped seam anchors.** **RESOLVED for reading discipline:** dated shipped anchors are ground truth for **named seams**; do not promote whole unit to DONE without explicit DONE. Atoms: `G1-C004`, `G4-C005`, `G5-C007`, `G7-C010`. Initiative DONE word → **RA-010**.

**CF-020 — G-5 dependency on G-2.** Hard G-2 prereq prose vs SSB-only scoring shipped (RRA optional). **RESOLVED — SSB-only sufficient for G-5 DONE framing.** Atom: `G-001/DG001-G5-C006`.

**CF-021 — G-2 recover_from_checkpoint.** “No production caller” myth vs `task_runner.rs:5018` production caller; resume-from-RRA still optional stub. **RESOLVED — do not revive no-caller claim.** Atom: `G-002/DG002-G2-011`.

**CF-022 — I-2 / belief / RRA map statuses.** Stale PLANNED/absent memories vs PARTIAL/SHIPPED facts (I-2 via J-80 MAP FIX; I-11 PARTIAL). **RESOLVED — later PARTIAL/SHIPPED wins.** Atoms: `I-002/I002-GM-032`; `J-010/J80-E003`.

**CF-023 — J-20 non-stationary TS.** J-20 SUPERSEDES plain-TS slices of D-5 / D-29 / H-48. **RESOLVED — J-20 is target TS mechanism.** Atoms: `J-010/J20-E002`, `IX-006`.

**CF-024 — J-52 default-on daemon vs E-52 dark enable.** Plan does not state supersede/remove/bypass. **UNRESOLVED → RA-002.** Atom: `J-002/DJ002-CON-008`.

**CF-025 — M-1 license surface.** Deep: CLI + ExpiredGrace banner + B-43 activation (no SettingsWindow) vs summary SettingsWindow pairing. **RESOLVED — deep CLI+banner.** Atoms: `M-001/M1-C001`.

**CF-026 — M enterprise rider numbering.** M-7+ vs residual summary M-6+. **RESOLVED — M-7+.** M-AGG SUP-002.

**CF-027 — N program inventory.** 16 units (N-1..N-16) vs “all 15 sessions.” **RESOLVED — 16.** N-AGG SUP-001.

**CF-028 — N-6 provenance scales.** Six vs summary five. **RESOLVED — six.** N-AGG SUP-002.

**CF-029 — N-5 router set.** Deep: E-39 not in ten routers vs summary first-class E-39 conflict. **RESOLVED — E-39 not in ten-router list.** N-AGG SUP-006.

**CF-030 — O-4 canvas editing tier.** Tier-1 read-only inspection vs summary node editing as ship-now. **RESOLVED — Tier-1 read-only.** O SUP/SD-MC003.

**CF-031 — O-2 stage/sink vocabulary vs Tier-1 runtime.** Schema eight stages/four sinks vs Tier-1 Fetch/Classify/Gate/Digest + DashboardInbox only. **RESOLVED — schema ≠ production wiring; Tier-1 subset is ship scope.** O SUP/SD-MC004. Same pattern O-1 topics SUP/SD-MC005.

**CF-032 — L-map substrate status labels.** L grounding map DARK/WIRED/PLANNED inventory of other families (G-7, D-16, D-13, I-4, E-43, D-11 DARK; B-39/B-29/I-40 WIRED; J-65/J-63/J-102 PLANNED). **RESOLVED as L-map orientation at discovery time** — later explicit SHIP/REJECT overrides L’s snapshot for that unit. Atoms: `L-001/LCGG-E007..E021`.

**CF-033 — PHASE TokenUsage field set.** Design 4 fields vs 0.2c shim 2 fields. **UNRESOLVED → RA-003.** Atoms: `P0-C001`, `PHASE-001/AC-001`.

**CF-034 — PHASE SPLADE deferral home.** 1G (source) vs 1H (MASTER summary). **RESOLVED — prefer phase source 1G** over MASTER orientation.

**CF-035 — PHASE 4I media models.** media-validator Mode A/B vs stale architecture-overview. **RESOLVED — media-validator is authority.**

**CF-036 — PHASE 4C ruff path.** Subprocess+breaker (decided) vs library-binding adversarial note. **RESOLVED — subprocess+breaker.**

**CF-037 — PHASE 1F prerequisite.** “Phase-0 independent” vs corrected 1C branch dependency. **RESOLVED — 1C dependency is target.**

**CF-038 — H-49 CAS / RV-1.** Map PARTIAL vs SHIPPED production caller `timetravel_cmd.rs` (J-80 recon). **RESOLVED — treat CAS/RV-1 as SHIPPED for consumers;** broader unit may stay PARTIAL on other legs.

**CF-039 — I-11 belief_state.** Older PLANNED vs GS-1/J map PARTIAL (`belief_state.rs`, condensed receipt path). **RESOLVED — PARTIAL.** Atom: `J-002/DJ002-G-004`.

**CF-040 — D-1 / D-8 archive hash formula.** Two-part (body+parent) vs three-part (+mutation_proposal_text). **UNRESOLVED → RA-004.** D-AGG SUP-002.

**CF-041 — D-7 canary block rule.** Objective non-target-only vs implementation ANY-family below threshold. **UNRESOLVED → RA-005.** D-AGG SUP-008.

**CF-042 — D-2 “different family” PPM vs D-13 cross-family.** Qwen Math PPM is same family as Qwen Coder; not valid D-13 adversary. **RESOLVED — D-13 same-family rejection wins.** Atoms: `D-003/D003-ISSUE-C001`, `D003-DELTA-002`.

**CF-043 — D-13 ModelFamily predicate wording.** Prose “different enum variants suffice” vs test requiring Qwen25Coder+Qwen25Math same-family/false. **UNRESOLVED on implementation shape → RA-006** (behavior target clear: same-family reject).

### Residual ambiguities (UNRESOLVED) — RA-001..RA-010

| ID | Feature | Why precedence fails | Key anchors |
|---|---|---|---|
| **RA-001** | **P-21** gate-credibility Tier boundary | Same plan: REVISED scope defers skipBackfill/redaction/posture to Tier-2 while WIRING_CONTRACT/stub_ban/smoke/PRODUCTION_HOOK require them. No SUPERSEDED chooses a half. | `P-001/SD-P21-001` vs `P21-C001`; P-AGG SUP-008/013; `P-21:L18-L71` |
| **RA-002** | **J-52 vs E-52** enable semantics | J-52 default-on daemon vs E-52 dark enable; no supersede/remove/bypass stated. | `J-002/DJ002-CON-008`; J-52:L16,L22,L30,L69,L96 |
| **RA-003** | **TokenUsage** field set | Design 4 fields vs shim 2; no later final schema marker. | `P0-C001`, `PHASE-001/AC-001` |
| **RA-004** | **D-1 vs D-8** archive hash | Two-part vs three-part; SUP-002 keeps conflict open. | `D-002/D1-E16`, `D8-E03`, `X-C01` |
| **RA-005** | **D-7** canary block rule | Non-target-only objective vs ANY-family implementation; neither deleted. | D-AGG SUP-008 |
| **RA-006** | **D-13** family-grouping predicate | Behavior target clear; enum/grouping implementation unspecified. | `D-003/D003-ISSUE-C004` |
| **RA-007** | **P-12** frame hash algorithm | Frame-binding text says sha256 of image bytes; hard rules require blake3. Same unit, no SUPERSEDED. | `P-001/P12-A003`; `P-12:L19,L32,L44,L73` |
| **RA-008** | **N-8** five views vs Kind enum | Target five views; Kind list names four and omits explicit J-70 Traceability. | `N-002/N8-E003`, `N8-A002`, `N0-A001`; N-AGG SUP-008 |
| **RA-009** | **Q-11 vs Q-14** novelty canary ownership | Summary assigns canary/`--self-test-canary` to Q-11 path; Q-14 ships harness `--self-test-novelty-canary` and says Q-11 hardens only runtime `novelty.rs`. | `SUMMARY/AMB-15`; Q-14 E-1 SHIPPED atoms |
| **RA-010** | **G unit DONE** promotion | Multiple G units have shipped seam anchors while header stays PLANNED; no file declares overall DONE. Seam ground truth via CF-019; initiative DONE word open. | `G1-C004`, `G4-C005`, `G5-C007`, `G7-C010` |

### How to use this register

1. When plan and summary disagree on status, look up the unit here first.
2. Apply the RESOLUTION line; do not re-derive from summary orientation alone.
3. If the unit is in the residual table, stop and recon before implement.
4. Wording-only deltas stay in per-family SUP inventories (518 family-local SUP/PREC total); only ship/wire/tier/disposition conflicts are lifted into CF-*.

---

## Closing counts (this companion)

| Metric | Value |
|---|---:|
| Hub systems in map (≥3 families) | **115** |
| Featured hubs in synergy narratives (primary + short table) | **~30** named product hubs |
| Structural cross-family edges | **881** |
| Shared invariant records | **34** (all INV-* section headers; source meta said 32) |
| Status conflict records (top-level CF) | **43** (CF-001..043) |
| CF-004 tier sub-rows | **16** (a–p) |
| CF-006 phase sub-rows | **4** (a–d) |
| Residual ambiguities UNRESOLVED | **10** (RA-001..010) |
| Multi-hop composition chains | **6** |
