# Cross-family interaction map — ELAI deep feature discovery

## Document contract

- **Inputs (read-only):** 17 family aggregates in this directory (`PHASE`, `B`…`Q`).
- **Scope:** Inter-family dependencies and compositions only. Intra-family edges stay in each aggregate.
- **Edge definition:** A directed edge is `consumer family feature/unit → provider feature` when an aggregate atom (prefer `IX-*`, else `E/DEP` / `E/INT` / WIRING / closes / extends / gates / supersedes) names a feature owned by another family.
- **Evidence rule:** Every listed edge carries at least one atom/record ID and an aggregate line anchor.
- **NEWER-PLAN PRECEDENCE:** Explicit SUPERSEDED marker wins; else later letter family wins for the same interaction (B oldest, Q newest). PHASE is foundation substrate, not a letter-order peer. Losers retained as superseded notes.
- **Counts:**
  - Raw cross-family mention edges (multi-ledger): **3010**
  - Unique (consumer family, provider feature, relation): **1670**
  - Unique structural pairs (consumer family → provider feature): **881** ← primary edge count for this map
  - Hub systems (provider referenced by ≥3 families): **115**
- **Generated:** 2026-08-12 from aggregate IX/DEP/INT signal extraction.

## How to read

- Relation verbs: `consumes`, `depends`, `extends`, `composes`, `gates`, `closes/extends`, `reuses`, `feeds`, `mirrors`, `supersedes`, `interacts`.
- Provider IDs keep native form (`F-4`, `phase-3B`→`phase-3B`, `SSB`).
- SSB is treated as a shared backbone (authored primarily under PHASE 3B + schema bumps across families).

---

## Hub systems (referenced by 3+ families)

These are load-bearing cross-family substrates. Changing a hub without updating dependents is a system break.

| Hub | Families | Raw edges | Role |
|---|---:|---:|---|
| `SSB` | 10 (G, H, I, J, K, L, M, N, P, Q) | 43 | session event bus / append-only journal (phase-3B backbone) |
| `B-37` | 9 (C, D, E, I, J, K, L, N, P) | 67 | RuntimeAdapter multi-backend substrate |
| `G-2` | 9 (H, I, J, K, L, M, O, P, Q) | 63 | RRA / replay-verify evidence plane |
| `D-13` | 7 (E, H, I, J, K, L, Q) | 45 | cross-family adversary / independence principle |
| `G-1` | 7 (H, J, K, L, N, O, P) | 43 | taint lattice (untrusted IN-flow) |
| `F-4` | 7 (G, H, I, J, K, L, P) | 41 | cant-lie / PROVEN case gate (stdout_hash, temporal gate) |
| `F-5` | 6 (I, J, L, N, P, Q) | 29 | quarantine / flaky isolation |
| `F-1` | 6 (G, H, I, J, L, N) | 26 | VibeGate / Layer-1 external review gate family |
| `B-40` | 6 (C, E, I, J, O, Q) | 19 | research report / framing + ResearchReport types |
| `H-49` | 5 (I, J, L, N, P) | 40 | retirement / RV snapshot + durable memory seam |
| `B-29` | 5 (C, E, J, K, L) | 39 | blast-radius / decomposition UI substrate (closed by C-24) |
| `G-9` | 5 (I, J, K, M, P) | 36 | redaction / privacy export plane |
| `D-16` | 5 (H, I, J, L, N) | 30 | refusal / calibration (RefusalDecision) |
| `H-9` | 5 (I, J, K, M, Q) | 29 | spend ledger / cost accounting |
| `E-54` | 5 (H, J, L, N, Q) | 21 | corpus / weak-lane related measurement substrate |
| `I-27` | 5 (J, K, L, N, Q) | 21 | convergence / bootstrap stats tooling |
| `G-6` | 5 (I, J, K, L, N) | 20 | seccomp / SAFEDELTA security strip |
| `D-11` | 5 (E, F, I, J, L) | 15 | checker registry |
| `D-6` | 5 (E, H, J, K, N) | 14 | held-out / corpus related |
| `B-22` | 5 (C, E, H, J, L) | 13 |  |
| `H-7` | 5 (I, J, K, N, Q) | 10 | split / over-escalation idiom |
| `G-10` | 4 (J, K, N, P) | 36 | model-drift canary / invalidation |
| `D-3` | 4 (E, H, I, L) | 24 | verification consensus / PPM path |
| `H-39` | 4 (I, J, K, L) | 24 | related H-plane substrate |
| `G-5` | 4 (H, J, K, N) | 22 |  |
| `H-31` | 4 (I, J, K, N) | 22 | related harness control |
| `B-41` | 4 (C, E, L, N) | 19 | per-level audit types |
| `D-17` | 4 (H, I, L, N) | 18 | refusal rate monitor |
| `E-39` | 4 (I, J, M, N) | 16 | caps / router list |
| `H-50` | 4 (I, J, L, N) | 16 | related H-plane substrate |
| `I-11` | 4 (J, K, L, Q) | 16 | BeliefState |
| `D-1` | 4 (E, G, I, J) | 15 |  |
| `H-32` | 4 (I, J, K, L) | 15 |  |
| `I-12` | 4 (J, L, N, PHASE) | 15 | curator (must not reimplement 3F lifecycle) |
| `I-4` | 4 (J, K, L, N) | 15 | related I substrate |
| `I-51` | 4 (J, K, L, N) | 15 | related I substrate |
| `K-11` | 4 (J, L, M, N) | 15 | PR evidence emitter |
| `C-29` | 4 (E, I, J, L) | 14 | intent / planning-mutation related gate |
| `E-25` | 4 (H, I, J, L) | 14 |  |
| `I-25` | 4 (J, K, N, Q) | 14 | post-wake router / intervention-card inbox |
| `D-10` | 4 (E, J, L, N) | 13 | mode routing predicate |
| `E-43` | 4 (F, G, J, L) | 13 |  |
| `E-48` | 4 (H, J, P, Q) | 13 | VerifiedCitation |
| `B-39` | 4 (C, E, J, L) | 12 |  |
| `E-35` | 4 (I, J, L, N) | 11 |  |

_Full hub list count: 115. Top 45 by family fan-in shown; remaining hubs appear in the reverse index._

### Hub deep dives (evidence)

#### `SSB` — session event bus / append-only journal (phase-3B backbone) (10 families)

- **G** `interacts` — atom `G-002/IX-015` — anchor `G-AGGREGATE.md:L1236`
  - **G-002/IX-015** [E] [interaction] label=`Shared SSB edit window` refs=`DG002-G2-013, DG002-G6-012, DG002-G8-019, DG002-G9-018, DG002-G10-021`: G-2/G-8/G-9 explicitly require coordination around SSB or commit/write-pat
- **H** `mirrors` — atom `H-001/IX-013` — anchor `H-AGGREGATE.md:L1391`
  - **H-001/IX-013** [IX] (DEEP-H-001.discovery.md:L384): H-GROUNDING L269-280, H-UX L31, H-VERIFICATION L57-59: SSB is the shared off-agent observability plane; new rows mirror established Pattern-2/single-writer/frozen-r
- **I** `interacts` — atom `I-001/IX-007` — anchor `I-AGGREGATE.md:L1163`
  - **I-001/IX-007** [IX] (DEEP-I-001.discovery.md:L481| unit=RELATIONAL): I-18 chains the SSB evidence on which I-28's sealed DPO provenance relies; an unsealed record cannot enter the I-30 training-data path. | anchor=I1
- **J** `reuses` — atom `J-004/IX-019` — anchor `J-AGGREGATE.md:L2946`
  - **J-004/IX-019** [IX] (DEEP-J-004.discovery.md:L420 | unit=Cross-source interactions and lifecycle): E | Moving SSB baselines across sources (J-1 221/532, J-12 229/540, LFG/SCM/J-64/J-65/J-74/J-97/J-104 ~268-269/577-57
- **K** `gates` — atom `K-001/IX-008` — anchor `K-AGGREGATE.md:L1194`
  - **K-001/IX-008** [IX] (DEEP-K-001.discovery.md:L326): E | K-16 and K-8/K-14 share the same pre-gate shape: `.claude/rules/*` + conditional-routing + JD-1 live load evidence + zero SSB. | K16-E001, K16-E010; K8-E009–K8-
- **L** `reuses` — atom `L-001/IX-017` — anchor `L-AGGREGATE.md:L1707`
  - **L-001/IX-017** [E] [interaction] (ledger-line: DEEP-L-001.discovery.md:L454): Prefer-reuse SSB policy: L-4 may +1 SemanticEntropyMeasured; L-6/L-9 net 0; L-12 +0..1 SkillPromoted; L-18 net 0; L-22 +FalsifierAuthored;
- **M** `interacts` — atom `M-001/IX-016` — anchor `M-AGGREGATE.md:L665`
  - **M-001/IX-016** [E/IX] (DEEP-M-001.discovery.md:L470): SSB budget: mandatory +EntitlementEvent and +OrgPolicyEvent; optional OrgReportEvent/ReleaseChannelEvent prefer-reuse; M-5 and M-6 add zero. | evidence: M0-E055–M
- **N** `interacts` — atom `N-001/IX-012` — anchor `N-AGGREGATE.md:L763`
  - **N-001/IX-012** [E] (DEEP-N-001.discovery.md:L396): GD-2 determinism: N-1 fixture, N-9 journal-label deltas, N-11 pinned-seed byte-identical folds all recompute from SSB/journal under seed. | evidence=N0-E024, N1-E017
- **P** `interacts` — atom `P-001/IX-013` — anchor `P-AGGREGATE.md:L1684`
  - **P-001/IX-013** [IX] class=`E` (ledger-line: DEEP-P-001.discovery.md:L394): SSB action_type map in P-0 assigns row families used by P-4/P-5/P-8/P-12/P-13/P-16/P-18/P-21 events. _(evidence: P0-E040; P4-E023; P5-E011; P
- **Q** `interacts` — atom `Q-002/IX-015` — anchor `Q-AGGREGATE.md:L828`
  - **Q-002/IX-015** [IX] class=`E` evidence=`Q1-E026; Q2-E020; Q3-E020; Q4-E019; Q6-E006; Q10-E032; Q11-E018` (ledger-line: DEEP-Q-002.discovery.md:L378): Multiple units add measured SSB rows (Goal*, Wake*, AdvisorConsult

#### `G-2` — RRA / replay-verify evidence plane (9 families)

- **H** `extends` — atom `H-004/IX-014` — anchor `H-AGGREGATE.md:L2889`
  - **H-004/IX-014** [IX] (DEEP-H-004.discovery.md:L517): H-49 extends E-29 transcript/C-23 BranchTree, uses E-44 only for irreversible warnings, retains G-2 verification, and aligns F-13 schema. | H-49:L58-L62; H49-E058/H
- **I** `composes` — atom `I-006/I43-E013` — anchor `I-AGGREGATE.md:L3551`
  - **I-006/I43-E013** [SRC] (DEEP-I-006.discovery.md:L317| unit=I-43): E/DEP | The implementation composes I-38 QDELTA, shipped G-6/SecCodePLT/SAFEDELTA dynamic-exploit/git-strip path, G-2 RRA capture, H-23 comparator sha
- **J** `consumes` — atom `J-008/IX-015` — anchor `J-AGGREGATE.md:L4450`
  - **J-008/IX-015** [IX] (DEEP-J-008.discovery.md:L376 | unit=Cross-source interactions and lifecycle): E | J-91 extends J-14 verify tiers + cassette; consumes G-1 bytes, G-2 bundle, I-45 seal, G-9 scrub; completeness hon
- **K** `interacts` — atom `K-001/IX-009` — anchor `K-AGGREGATE.md:L1195`
  - **K-001/IX-009** [IX] (DEEP-K-001.discovery.md:L327): E | K-20 strict-strengthens G-2 replayable RRA using J-14 cassette packaging; measured proof stays OUT-OF-K on J-14-extension. | K20-E002–K20-E004, K20-E021; K0-E04
- **L** `gates` — atom `L-002/L7-E008` — anchor `L-AGGREGATE.md:L578`
  - **L-002/L7-E008** [E/DEP] anchor=`L-7:L8-L8` unit=`L-7 — Hallucination policy gate + abstention` (ledger-line: DEEP-L-002.discovery.md:L200): Pre-reqs: D-16 RefusalDecision tau; I-8 acknowledge_infeasible; D-17 SLA; B-
- **M** `composes` — atom `M-001/M5-E029` — anchor `M-AGGREGATE.md:L131`
  - **M-001/M5-E029** [SURFACE-RELATED] class=`E/DEP` anchor=`M-5:L45-L48` (DEEP-M-001.discovery.md:L367): COMPOSES-WITH: G-8 attestation verify, G-2 rra verify, K-6 conformance, M-3 org verify (the four legs the OSS binar
- **O** `supersedes` — atom `SUP/SD-MC009` — anchor `O-AGGREGATE.md:L1114`
  - **SUP/SD-MC009** [superseded] winner=`O-001 originals (deep)` loser=`SUMMARY orientation` sd_ref=`O-001/SD-MC009` title=`The trust-spine boundary is more precise than a blanket byte claim.` (ledger-line: DEEP-O-001.dis
- **P** `consumes` — atom `P-003/IX-011` — anchor `P-AGGREGATE.md:L1730`
  - **P-003/IX-011** [IX] class=`E` (ledger-line: DEEP-P-003.discovery.md:L330): P-23 reuses E-14 engine (or vendors then E-14 consumes); H-15 view_type + H-13 dispatcher; P-13 ids + P-12 frame bind; P-14 crystallize; P-6 
- **Q** `consumes` — atom `Q-001/Q5-E006` — anchor `Q-AGGREGATE.md:L343`
  - **Q-001/Q5-E006** [SRC] class=`E/DEP` anchor=`Q-5:L14-L20, L57-L58` (ledger-line: DEEP-Q-001.discovery.md:L140): Consumes G-2 RRA, GD-5, J-28 usage_turn_event (commit fe069d0), sidecar sampling_seed, G model-drift cana

#### `G-1` — taint lattice (untrusted IN-flow) (7 families)

- **H** `composes` — atom `H-002/HTG-010` — anchor `H-AGGREGATE.md:L1531`
  - **H-002/HTG-010** [SRC] (DEEP-H-002.discovery.md:L78): H-20 must either depend honestly on G-1 live taint propagation or build a finance-scoped DKG dataflow pass; it composes with the existing taint primitive. (lines 5
- **J** `consumes` — atom `J-008/IX-015` — anchor `J-AGGREGATE.md:L4450`
  - **J-008/IX-015** [IX] (DEEP-J-008.discovery.md:L376 | unit=Cross-source interactions and lifecycle): E | J-91 extends J-14 verify tiers + cassette; consumes G-1 bytes, G-2 bundle, I-45 seal, G-9 scrub; completeness hon
- **K** `interacts` — atom `K-001/K20-E004` — anchor `K-AGGREGATE.md:L180`
  - **K-001/K20-E004** [SRC] (DEEP-K-001.discovery.md:L287 | SURFACE): E/DEP | Substrate cited: G-2 RRA WIRED (`elai run --emit-rra` / `rra show` / `replay --verify`; msgpack+zstd; action-stream on cloud); J-14 PLANNED ver
- **L** `consumes` — atom `L-002/L1-E016` — anchor `L-AGGREGATE.md:L519`
  - **L-002/L1-E016** [E/DEP] anchor=`L-1:L36-L38` unit=`L-1 — Typed claim extraction substrate` (ledger-line: DEEP-L-002.discovery.md:L129): Consumes grounding_mode.rs::select_grounding_mode (COMPOSE never re-implement) a
- **N** `composes` — atom `N-002/N6-E002` — anchor `N-AGGREGATE.md:L492`
  - **N-002/N6-E002** [E/SEC] anchor=`N-6:L5-L5` unit=`N-6 — provenance / trust lattice + admission-gate family` (ledger-line: DEEP-N-002.discovery.md:L120): Premise: six independent "how much do we trust this byte's origi
- **O** `supersedes` — atom `SUP/SD-MC009` — anchor `O-AGGREGATE.md:L1114`
  - **SUP/SD-MC009** [superseded] winner=`O-001 originals (deep)` loser=`SUMMARY orientation` sd_ref=`O-001/SD-MC009` title=`The trust-spine boundary is more precise than a blanket byte claim.` (ledger-line: DEEP-O-001.dis
- **P** `mirrors` — atom `P-002/IX-004` — anchor `P-AGGREGATE.md:L1698`
  - **P-002/IX-004** [IX] class=`E` (ledger-line: DEEP-P-002.discovery.md:L397): P-6 OUT-flow privacy is the mirror of G-1 IN-flow taint; both ride N-6's ONE lattice (N-2 forbids a second scale). _(evidence: P6-E002–P6-E00

#### `F-4` — cant-lie / PROVEN case gate (stdout_hash, temporal gate) (7 families)

- **G** `consumes` — atom `G-002/IX-009` — anchor `G-AGGREGATE.md:L1230`
  - **G-002/IX-009** [E] [interaction] label=`F-1/F-4/F-9 → G-8` refs=`DG002-G8-016, DG002-G8-020, DG002-G8-027, DG002-G8-053`: The attestation rollup consumes their recorded BLAKE3 verdicts; F-4’s anti-fabrication evidenc
- **H** `depends` — atom `H-005/H005-52-001` — anchor `H-AGGREGATE.md:L826`
  - **H-005/H005-52-001** [SRC] (DEEP-H-005.discovery.md:L372 | SURFACE): E-STATUS/DEP | L1-L3 | Build depends on planned G-5 cockpit/DSR and C-7 hunk space; composes F-4 and defers E-41/C-40 GitHub delivery.
- **I** `interacts` — atom `I-004/I45-E012` — anchor `I-AGGREGATE.md:L2757`
  - **I-004/I45-E012** [SRC] (DEEP-I-004.discovery.md:L449| unit=I-45): E/DEP | F-4 `stdout_hash` is the sibling artifact-hash precedent; I-18 `chain_prev_hash` shares the record and requires field-layout coordination if i
- **J** `gates` — atom `J-005/IX-001` — anchor `J-AGGREGATE.md:L3340`
  - **J-005/IX-001** [IX] (DEEP-J-005.discovery.md:L398 | unit=Cross-file interaction records): E | NASA map SG-2/NG-1 forbids rebuilding F-4 temporal gate as J-66 monitor; J-65/J-69 are siblings of F-4, not replacements. 
- **K** `consumes` — atom `K-002/K3-E014` — anchor `K-AGGREGATE.md:L325`
  - **K-002/K3-E014** [SRC] (DEEP-K-002.discovery.md:L88 | SURFACE): E/DEP | Consumes github_pr adapter (KG-2/KC-2), review_surface G-5, H-52 resolver, F-4, G-8, J-16 when present, RRA manifest G-2. | anchor=K-3:L18-L21, L
- **L** `composes` — atom `L-002/L0-E018` — anchor `L-AGGREGATE.md:L449`
  - **L-002/L0-E018** [E/DEP] anchor=`L-0:L46-L46` unit=`L-0 — L-Plan Index (doctrine, tracks, sequencing, constants)` (ledger-line: DEEP-L-002.discovery.md:L53): Inherits GD-1/GD-2/GD-4/GD-5: compose-not-restate (re-imple
- **P** `extends` — atom `P-003/P15-E006` — anchor `P-AGGREGATE.md:L688`
  - **P-003/P15-E006** [E] class=`E/DEP` anchor=`P-15:L11-L16, L32-L34, L68-L68` unit=`P-15` (ledger-line: DEEP-P-003.discovery.md:L79): Extends J-96 CaseNode/MintOnProof; composes K-18 recurrence-hit-rate, K-19 gate-the-s

#### `B-37` — RuntimeAdapter multi-backend substrate (9 families)

- **C** `depends` — atom `C-005/C44-E02` — anchor `C-AGGREGATE.md:L2890`
  - **C-005/C44-E02** [feature] [E] (unit=C-44, ledger=C-005 | anchor: L8-L13, L68-L73): B-37 and C-39 are shipped prerequisites; API access is a hard gate, sequencing is after C-39 and C-42, and routing integration separa
- **D** `consumes` — atom `D:L746` — anchor `D-AGGREGATE.md:L746`
  - D14-E27 | [E] | DEP,INT | The module consumes B-37 `RuntimeAdapter`, D-10 Claim, SSB writing, and the user-message field on `OrchContext`, and is reachable from the task-runner post-dispatch hook plus config. | D-14_sy
- **E** `consumes` — atom `E-006/E006-E24-002` — anchor `E-AGGREGATE.md:L1656`
  - **E-006/E006-E24-002** [SRC] SURFACE (unit=E-24_agent_protocols_mcp | anchor: DEEP-E-006.discovery.md:L104): [DEP/INT] It depends on B-37 plus planned C-35/C-39/C-42/B-19/B-20, extends B-3’s existing bridge contract, c
- **I** `interacts` — atom `I-001/IX-014` — anchor `I-AGGREGATE.md:L1170`
  - **I-001/IX-014** [IX] (DEEP-I-001.discovery.md:L488| unit=RELATIONAL): I-36 serves a compiled variant only after I-32 role scoping and uses B-37 for the role/backend key; its optimizer and promotion evaluator stay outs
- **J** `consumes` — atom `J-009/IX-005` — anchor `J-AGGREGATE.md:L4820`
  - **J-009/IX-005** [IX] (DEEP-J-009.discovery.md:L362 | unit=Cross-source interactions and lifecycle): E | J-19 is additive backend #12 on B-37 RuntimeAdapter; J-51 role routing also consumes B-37 PerRoleRuntime/try_comp
- **K** `reuses` — atom `K-003/K24-E026` — anchor `K-AGGREGATE.md:L526`
  - **K-003/K24-E026** [SRC] (DEEP-K-003.discovery.md:L273 | SURFACE): E/DEP | GD-1 reuse list: J-28 meter, J-49 seat/quota, C-5/J-81 caps + runaway kill, H-9 $/resolved-task, J-54 free pool, J-55 modality router, B-37 bac
- **L** `composes` — atom `L-002/L0-E007` — anchor `L-AGGREGATE.md:L438`
  - **L-002/L0-E007** [E/STATUS] anchor=`L-0:L20-L20` unit=`L-0 — L-Plan Index (doctrine, tracks, sequencing, constants)` (ledger-line: DEEP-L-002.discovery.md:L42): Predecessors B..K; L COMPOSES-WITH them (GD-1) and resta
- **N** `interacts` — atom `N-002/N13-E006` — anchor `N-AGGREGATE.md:L611`
  - **N-002/N13-E006** [E/DEP] anchor=`N-13:L8-L9` unit=`N-13 — OutputBoundary + structured-output capability matrix` (ledger-line: DEEP-N-002.discovery.md:L263): Pre-requisites re-greped at edit (W-7): the >=8 machines ab
- **P** `consumes` — atom `P-003/IX-005` — anchor `P-AGGREGATE.md:L1724`
  - **P-003/IX-005** [IX] class=`E` (ledger-line: DEEP-P-003.discovery.md:L324): P-17 hardens J free-API rotation + I-50 auth + J-28 ledger; feeds K-2/PS-4; substitutions align with Q-4; B-37 consumes fallback chains. _(ev

#### `D-13` — cross-family adversary / independence principle (7 families)

- **E** `consumes` — atom `E-002/E53-027` — anchor `E-AGGREGATE.md:L600`
  - **E-002/E53-027** [SRC] SURFACE (unit=E-53_formal_verification_codegen | anchor: DEEP-E-002.discovery.md:L393): [INT/DEP] The system consumes D-3 aggregation, EvidenceGate, CSQ, D-13 origin, five formal dependencies, a
- **H** `composes` — atom `H-002/IX-006` — anchor `H-AGGREGATE.md:L1841`
  - **H-002/IX-006** [IX] (DEEP-H-002.discovery.md:L363): H-45 cannot be meaningful before H-43 replaces byte-derived fitness; it then aggregates H-43/D-13 lineage views, composes H-39 abstention semantics, and feeds exist
- **I** `composes` — atom `I-004/IX-010` — anchor `I-AGGREGATE.md:L2795`
  - **I-004/IX-010** [IX] (DEEP-I-004.discovery.md:L485| unit=RELATIONAL): E | I-21’s execution path composes H-37 approval, I-1 context isolation, I-17 deterministic checks, H-38 stop, and D-13 counterexample escalation i
- **J** `depends` — atom `J-004/IX-016` — anchor `J-AGGREGATE.md:L2943`
  - **J-004/IX-016** [IX] (DEEP-J-004.discovery.md:L417 | unit=Cross-source interactions and lifecycle): E | J-97 depends on J-96 store/recurrence first; fitness reuses H-43/H-45/H-48, D-6/H-47 held-out, D-13 cross-family,
- **K** `gates` — atom `K-001/K16-E008` — anchor `K-AGGREGATE.md:L1131`
  - **K-001/K16-E008** [SRC/E] (DEEP-K-001.discovery.md:L253): `E/DEP` — Runtime archetypes cited (BIND not restate): F-4 AntiFabricationGate (WIRED blake3 stdout_hash); D-13 adversary cross-family (DARK); I-4 select-by-re
- **L** `gates` — atom `L-002/IX-012` — anchor `L-AGGREGATE.md:L1725`
  - **L-002/IX-012** [E] [interaction] (ledger-line: DEEP-L-002.discovery.md:L418): L-15 binding revision in L-0 reorders W2 to L-3+L-5 before L-4 and requires D-13 alongside L-4; this chunk's L-2 still lists L-4/L-5 as ex
- **Q** `consumes` — atom `Q-002/Q2-E011` — anchor `Q-AGGREGATE.md:L557`
  - **Q-002/Q2-E011** [SRC] class=`E/DEP` anchor=`Q-2:L14-L19, L56-L57, L88-L89` (ledger-line: DEEP-Q-002.discovery.md:L88): Consumes O-1/O-2 wake sites, 4G AutonomousGate, I-25 post-wake router, J free-API rotation, SSB m

#### `F-5` — quarantine / flaky isolation (6 families)

- **I** `gates` — atom `I-004/IX-008` — anchor `I-AGGREGATE.md:L550`
  - **I-004/IX-008** [IX] (DEEP-I-004.discovery.md:L483| unit=RELATIONAL SURFACE): E | I-12 is a gated extension chain: receipt count/lift -> C-28 mutation validation -> existing Phase-3F lifecycle -> RulesDrawer/export, w
- **J** `composes` — atom `J-005/J21-E021` — anchor `J-AGGREGATE.md:L3117`
  - **J-005/J21-E021** [SRC] (DEEP-J-005.discovery.md:L129 | unit=J-21 — memory semantic-validation + poisoning hardening): E/DEP E/DEP | COMPOSES-WITH H-32 (existence→semantic), H-31 firewall (typed packet), H-30 instrume
- **L** `reuses` — atom `L-002/IX-004` — anchor `L-AGGREGATE.md:L1717`
  - **L-002/IX-004** [E] [interaction] (ledger-line: DEEP-L-002.discovery.md:L410): L-8 reuses F-5 kill-switch rather than inventing a new flag; L-7 invents ELAI_HALLUCINATION_GATE_DISABLE; both are W-2 kill-switch style. 
- **N** `composes` — atom `N-001/IX-014` — anchor `N-AGGREGATE.md:L765`
  - **N-001/IX-014** [E] (DEEP-N-001.discovery.md:L398): N-4 memo composes C-30 cache shape, J-96 case nodes, J-21 identity, and F-5 invalidation; I-8 is the negative-space twin already caching failures. | evidence=N4-E007
- **P** `consumes` — atom `P-001/P4-E008` — anchor `P-AGGREGATE.md:L122`
  - **P-001/P4-E008** [E] class=`E/DEP` anchor=`P-4:L13-L17, L56-L57, L91-L92` unit=`P-4` (ledger-line: DEEP-P-001.discovery.md:L116): Consumes O-3 vault + namespace-claims, B-35, F-5, N-6, D-5 denylist, P-6 + E-20 routing
- **Q** `consumes` — atom `Q-002/Q1-E012` — anchor `Q-AGGREGATE.md:L126`
  - **Q-002/Q1-E012** anchor=`Q-1:L14-L20, L58-L59, L92-L93` (ledger-line: DEEP-Q-002.discovery.md:L47): Consumes O-2 TaskSpec+task_runner+`task_spec_fired`; SSB measured-number row pattern (schema_version 270/271); F-5 qu

#### `C-28` — mutation validation gate (3 families)

- **F** `depends` — atom `SUMMARY/F4-19` — anchor `F-AGGREGATE.md:L1249`
  - **SUMMARY/F4-19** `[E]` (unit=`F-4`, ledger-line=185) F-4 depends on C-28. _(anchor: L88-L88; source: 06-F-SUMMARY_INDEX.discovery.md)_
- **I** `gates` — atom `I-004/IX-008` — anchor `I-AGGREGATE.md:L550`
  - **I-004/IX-008** [IX] (DEEP-I-004.discovery.md:L483| unit=RELATIONAL SURFACE): E | I-12 is a gated extension chain: receipt count/lift -> C-28 mutation validation -> existing Phase-3F lifecycle -> RulesDrawer/export, w
- **J** `extends` — atom `J-009/IX-006` — anchor `J-AGGREGATE.md:L4821`
  - **J-009/IX-006** [IX] (DEEP-J-009.discovery.md:L363 | unit=Cross-source interactions and lifecycle): E | J-23 extends F-1/F-2 VibeGate as external Layer-1 review backends; J-88 E3 re-routes INPUT of the same binary-gat

#### `C-29` — intent / planning-mutation related gate (4 families)

- **E** `extends` — atom `E-005/E005-E13-002` — anchor `E-AGGREGATE.md:L1284`
  - **E-005/E005-E13-002** [SRC] SURFACE (unit=E-13_tdd_verification_agents | anchor: DEEP-E-005.discovery.md:L88): [DEP/STATUS] It augments C-28/C-29, depends also on C-27/C-31/C-32, extends D-3's verifier aggregate, reus
- **I** `composes` — atom `I-001/IX-017` — anchor `I-AGGREGATE.md:L1173`
  - **I-001/IX-017** [IX] (DEEP-I-001.discovery.md:L491| unit=RELATIONAL): I-37 composes C-29 intent, C-50 planning gates, I-20 mutation evidence, RelationLibrary, and G-5/H-52 presentation without owning those subsystems.
- **J** `interacts` — atom `J-013/IX-011` — anchor `J-AGGREGATE.md:L6450`
  - **J-013/IX-011** [IX] (DEEP-J-013.discovery.md:L368 | unit=Cross-file interaction records): E | J-38 soft-depends J-18 hardened cascade; exposes hybrid_search via C-13 IpcToolDispatcher alongside B-29 tools; sufficienc
- **L** `gates` — atom `L-003/L5-E006` — anchor `L-AGGREGATE.md:L796`
  - **L-003/L5-E006** [E/DEP] anchor=`L-5:L8-L8, L15-L26` unit=`L-5 — counterfactual do-test on model claims [KEYSTONE]` (ledger-line: DEEP-L-003.discovery.md:L90): Pre-reqs include F-4 AntiFabricationGate (task_runner.rs:

#### `J-16` — blast-radius analysis (when present) (2 families)

- **K** `consumes` — atom `K-002/K3-E014` — anchor `K-AGGREGATE.md:L325`
  - **K-002/K3-E014** [SRC] (DEEP-K-002.discovery.md:L88 | SURFACE): E/DEP | Consumes github_pr adapter (KG-2/KC-2), review_surface G-5, H-52 resolver, F-4, G-8, J-16 when present, RRA manifest G-2. | anchor=K-3:L18-L21, L
- **L** `gates` — atom `L-003/L5-E006` — anchor `L-AGGREGATE.md:L796`
  - **L-003/L5-E006** [E/DEP] anchor=`L-5:L8-L8, L15-L26` unit=`L-5 — counterfactual do-test on model claims [KEYSTONE]` (ledger-line: DEEP-L-003.discovery.md:L90): Pre-reqs include F-4 AntiFabricationGate (task_runner.rs:

#### `G-8` — attestation signing / verify core (3 families)

- **J** `interacts` — atom `J-011/IX-009` — anchor `J-AGGREGATE.md:L5688`
  - **J-011/IX-009** [IX] (DEEP-J-011.discovery.md:L390 | unit=Cross-source interactions and lifecycle): E | J-89 L1 enriches I-45 sealed objective into typed bundle and freezes via G-8; L8 lifts wiring_contract_check to p
- **K** `consumes` — atom `K-002/K3-E014` — anchor `K-AGGREGATE.md:L325`
  - **K-002/K3-E014** [SRC] (DEEP-K-002.discovery.md:L88 | SURFACE): E/DEP | Consumes github_pr adapter (KG-2/KC-2), review_surface G-5, H-52 resolver, F-4, G-8, J-16 when present, RRA manifest G-2. | anchor=K-3:L18-L21, L
- **M** `extends` — atom `M-001/IX-008` — anchor `M-AGGREGATE.md:L657`
  - **M-001/IX-008** [E/IX] (DEEP-M-001.discovery.md:L462): M-3 extends K-6 rather than forking it, requires G-9 redaction precondition, and reuses G-2/G-8 for chain verify. | evidence: M3-E010–M3-E013, M3-E028

#### `G-9` — redaction / privacy export plane (5 families)

- **I** `extends` — atom `I-001/IX-005` — anchor `I-AGGREGATE.md:L1161`
  - **I-001/IX-005** [IX] (DEEP-I-001.discovery.md:L479| unit=RELATIONAL): I-13 extends the shipped safety/memory stack at write time and must not duplicate H-31/H-32, E-37, MINJA, G-9, or H-6. | anchor=IG-E017, IG-E051, I
- **J** `consumes` — atom `J-008/IX-015` — anchor `J-AGGREGATE.md:L4450`
  - **J-008/IX-015** [IX] (DEEP-J-008.discovery.md:L376 | unit=Cross-source interactions and lifecycle): E | J-91 extends J-14 verify tiers + cassette; consumes G-1 bytes, G-2 bundle, I-45 seal, G-9 scrub; completeness hon
- **K** `interacts` — atom `K-002/IX-005` — anchor `K-AGGREGATE.md:L1502`
  - **K-002/IX-005** [IX] (DEEP-K-002.discovery.md:L309): E | K-6 G-9 secret-scrub and redacted RRA composition is the publication privacy leg complementary to K-3’s public PR render. | K6-E003, E009; K3-E003
- **M** `extends` — atom `M-001/IX-008` — anchor `M-AGGREGATE.md:L657`
  - **M-001/IX-008** [E/IX] (DEEP-M-001.discovery.md:L462): M-3 extends K-6 rather than forking it, requires G-9 redaction precondition, and reuses G-2/G-8 for chain verify. | evidence: M3-E010–M3-E013, M3-E028
- **P** `interacts` — atom `P-003/IX-002` — anchor `P-AGGREGATE.md:L1721`
  - **P-003/IX-002** [IX] class=`E` (ledger-line: DEEP-P-003.discovery.md:L321): P-3 import re-verification rides M-5/M-6; admission rides L-12/skillspector; provenance redaction rides G-9; RRA/lineage rides L-14/G-2. _(ev

#### `G-10` — model-drift canary / invalidation (4 families)

- **J** `depends` — atom `J-008/IX-006` — anchor `J-AGGREGATE.md:L1386`
  - **J-008/IX-006** [IX] (DEEP-J-008.discovery.md:L367 | unit=Cross-source interactions and lifecycle | SURFACE): E | J-57 depends on J-56 eval subset as promotion gate and reuses G-10 as prompt-layer rollback signal whil
- **K** `extends` — atom `K-004/K25-E015` — anchor `K-AGGREGATE.md:L666`
  - **K-004/K25-E015** [SRC] (DEEP-K-004.discovery.md:L326 | SURFACE): E/DEP | Extends K-8 tags + provider-absorption-pregate.md; composes elai_status_query.py advisory-cache pattern; sibling to K-9 model-refresh axis (dis
- **N** `composes` — atom `N-001/N1-E030` — anchor `N-AGGREGATE.md:L242`
  - **N-001/N1-E030** [E/DEP] anchor=`N-1:L56-L57` unit=`N-1 — measurement spine as ONE subsystem` (ledger-line: DEEP-N-001.discovery.md:L177): COMPOSES-WITH: J-56/E-54/L-20 corpus; K-15/I-27 runner; ModelCapabilityProfile
- **P** `extends` — atom `P-002/P1-E029` — anchor `P-AGGREGATE.md:L374`
  - **P-002/P1-E029** [E] class=`E/DEP` anchor=`P-1:L92-L93` unit=`P-1` (ledger-line: DEEP-P-002.discovery.md:L65): COMPOSES-WITH: O-2 TaskSpec + DashboardInbox (EXTEND), G-10 (generalize), F-9/F-5, L-14 RRA, N-7, O-3 name

#### `O-2` — TaskSpec + Cron/SsbEvent triggers + task_runner (2 families)

- **P** `gates` — atom `P-001/IX-008` — anchor `P-AGGREGATE.md:L1679`
  - **P-001/IX-008** [IX] class=`E` (ledger-line: DEEP-P-001.discovery.md:L389): P-18 hardens P-11 confirm gate and O-2 run store; smoke requires P-11 multi-step confirm under 4G. _(evidence: P18-E001, P18-E004, P18-E021)_
- **Q** `consumes` — atom `Q-002/Q1-E012` — anchor `Q-AGGREGATE.md:L126`
  - **Q-002/Q1-E012** anchor=`Q-1:L14-L20, L58-L59, L92-L93` (ledger-line: DEEP-Q-002.discovery.md:L47): Consumes O-2 TaskSpec+task_runner+`task_spec_fired`; SSB measured-number row pattern (schema_version 270/271); F-5 qu

#### `I-25` — post-wake router / intervention-card inbox (4 families)

- **J** `interacts` — atom `J-006/J11-E004` — anchor `J-AGGREGATE.md:L3506`
  - **J-006/J11-E004** [SRC] (DEEP-J-006.discovery.md:L95 | unit=J-11 — arbor / combench / denovoswe steal intake closeout): E/DEP E/DEP | Targets: I-2, I-4, I-17, I-25, I-30, D-6, D-11, H-43, H-47, H-50; none currently ci
- **K** `interacts` — atom `K-003/K2-E007` — anchor `K-AGGREGATE.md:L485`
  - **K-003/K2-E007** [SRC] (DEEP-K-003.discovery.md:L42 | SURFACE): E/DEP | Pre-requisites: H-7 calibrated routing + I-25 router (A-6 — re-grep cloud_mode.rs CloudModeRouter + providers escalation_policy.rs); bench dispat
- **N** `consumes` — atom `SUMMARY/DEP-003` — anchor `N-AGGREGATE.md:L1592`
  - **SUMMARY/DEP-003** [E] anchor=`14-N:L787` (14-N-SUMMARY_INDEX.discovery.md:L787): N-3 consumes H-17/H-18/H-19, H-47, I-30/I-31, ID-6, N-1, J-56/L-20/D-6, 3F, J-57, J-97, B-23/H-45, I-4/I-25, and L-11.
- **Q** `composes` — atom `Q-002/IX-004` — anchor `Q-AGGREGATE.md:L817`
  - **Q-002/IX-004** [IX] class=`E` evidence=`Q2-E011; Q3-E005, Q3-E010; Q4-E009, Q4-E023` (ledger-line: DEEP-Q-002.discovery.md:L367): Q-2/Q-3/Q-4 all consume I-11 BeliefState and/or I-25 routing and compose O-2/4G/4M/J s

#### `J-28` — usage_turn_event cost ledger rows (3 families)

- **K** `consumes` — atom `K-003/IX-010` — anchor `K-AGGREGATE.md:L1865`
  - **K-003/IX-010** [IX] (DEEP-K-003.discovery.md:L374): K24-E006/E008, K2-E009 | K-24 L2/L3 book Settled amounts against J-28 UsageTurnEvent/SpendAccumulator that K-2 also consumes for live economics. | K-24:L35-L37; K-2
- **P** `consumes` — atom `P-003/IX-005` — anchor `P-AGGREGATE.md:L1724`
  - **P-003/IX-005** [IX] class=`E` (ledger-line: DEEP-P-003.discovery.md:L324): P-17 hardens J free-API rotation + I-50 auth + J-28 ledger; feeds K-2/PS-4; substitutions align with Q-4; B-37 consumes fallback chains. _(ev
- **Q** `interacts` — atom `Q-002/IX-009` — anchor `Q-AGGREGATE.md:L822`
  - **Q-002/IX-009** [IX] class=`E` evidence=`Q11-E006, Q11-E007; Q12-E004, Q12-E016` (ledger-line: DEEP-Q-002.discovery.md:L372): Q-12 C3 reinvested budget accounting and C1+C2 equal-budget compare both read J-28 `usage_t

#### `N-6` — ONE provenance/trust lattice + admission (1 families)

- **P** `mirrors` — atom `P-002/IX-004` — anchor `P-AGGREGATE.md:L1698`
  - **P-002/IX-004** [IX] class=`E` (ledger-line: DEEP-P-002.discovery.md:L397): P-6 OUT-flow privacy is the mirror of G-1 IN-flow taint; both ride N-6's ONE lattice (N-2 forbids a second scale). _(evidence: P6-E002–P6-E00

#### `H-49` — retirement / RV snapshot + durable memory seam (5 families)

- **I** `composes` — atom `I-004/IX-004` — anchor `I-AGGREGATE.md:L2789`
  - **I-004/IX-004** [IX] (DEEP-I-004.discovery.md:L479| unit=RELATIONAL): E | I-2 conflict recovery composes BSP checkpoint recovery and H-49 trunk restore; it owns neither checkpoint nor rollback implementation. | anchor
- **J** `reuses` — atom `J-005/IX-011` — anchor `J-AGGREGATE.md:L3350`
  - **J-005/IX-011** [IX] (DEEP-J-005.discovery.md:L408 | unit=Cross-file interaction records): E | J-61 preempt reuses H-49 EditHistoryEvent; J-25 rider A5 also confirms H-49 RV-2 microcompaction without owning it. | J61-
- **L** `reuses` — atom `L-001/IX-008` — anchor `L-AGGREGATE.md:L1698`
  - **L-001/IX-008** [E] [interaction] (ledger-line: DEEP-L-001.discovery.md:L445): L-18 is a pure corollary of L-16 immutability: losslessness and time-travel collapse to projection parameters; it re-points K-12 and reuse
- **N** `extends` — atom `N-001/IX-005` — anchor `N-AGGREGATE.md:L756`
  - **N-001/IX-005** [E] (DEEP-N-001.discovery.md:L389): N-11 Wave 7 extends L-16 rather than inventing a fold; N-0 landmine forbids re-authoring the fold and narrows residual to recover_stubs + H-49 store. | evidence=N0-E
- **P** `extends` — atom `P-002/P6-E020` — anchor `P-AGGREGATE.md:L435`
  - **P-002/P6-E020** [E] class=`E/DEP` anchor=`P-6:L91-L92` unit=`P-6` (ledger-line: DEEP-P-002.discovery.md:L138): COMPOSES-WITH: N-6 (EXTEND privacy dimension), N-2, G-1 (mirror), G-9, E-20, O-3, N-7, H-49. O-3 owns vau

#### `B-40` — research report / framing + ResearchReport types (6 families)

- **C** `depends` — atom `SUMMARY/S-E0544` — anchor `C-AGGREGATE.md:L2449`
  - **SUMMARY/S-E0544** [feature] [E] (unit=C-33, ledger=SUMMARY | anchor: L437-L440): Closes update-phase-map, B-0_INDEX, B-39, B-40, and B-21; depends on C-30 and C-22; is otherwise parallel-safe with all plans because o
- **E** `consumes` — atom `E-005/E005-E50-002` — anchor `E-AGGREGATE.md:L1477`
  - **E-005/E005-E50-002** [SRC] SURFACE (unit=E-50_saas_architecture_patterns | anchor: DEEP-E-005.discovery.md:L326): [DEP/STATUS] It consumes shipped E-11/E-12 and B-7/B-40, is parallel-safe with E-14/E-15/E-17, and fee
- **I** `consumes` — atom `I-003/DI003-I41-035` — anchor `I-AGGREGATE.md:L2202`
  - **I-003/DI003-I41-035** [SRC] (DEEP-I-003.discovery.md:L735| unit=I-41): [E][dependency] LensSet consumes B-38, E-45/cross_lang_diff, Phase 2B VerificationMethod, B-40, and B-37 runtime. Anchor: I-41_spec_time_critique
- **J** `interacts` — atom `J-010/J110-E006` — anchor `J-AGGREGATE.md:L5247`
  - **J-010/J110-E006** [SRC] (DEEP-J-010.discovery.md:L418 | unit=J-110 — abstention timeliness + eval supply): E/DEP E/DEP | Pre-reqs: H-39 AbstentionScored (SCHEMA 218) + abstention_score.rs; D-16 RefusalDecision; J-56 
- **O** `composes` — atom `O-001/O1-E109` — anchor `O-AGGREGATE.md:L258`
  - **O-001/O1-E109** [E] anchor=`O1:L82-L88` (ledger-line: DEEP-O-001.discovery.md:L244): O-1 composes with `CosmosStage`, `cosmos_worker`, briefing compiler, G-1, F-3, G-2/RRA, and B-40 `FlashResearch`, and reimplements 
- **Q** `interacts` — atom `Q-001/QG-E005` — anchor `Q-AGGREGATE.md:L255`
  - **Q-001/QG-E005** [SRC] class=`E/DEP` anchor=`QG:L22-L22` (ledger-line: DEEP-Q-001.discovery.md:L40): RC-1: recursive research already returns typed reports with sources, alternatives, contradictions, and sub-question 

#### `K-24` — BCC billing/credits rail (2 families)

- **M** `gates` — atom `M-001/IX-004` — anchor `M-AGGREGATE.md:L653`
  - **M-001/IX-004** [E/IX] (DEEP-M-001.discovery.md:L458): M-1 is the keystone: its `tier()` read gates K-24 Team credit grants, M-2 pack apply, and M-3 org report. | evidence: M1-E025–M1-E028; M2-E002, M2-E026; M3-E002
- **N** `consumes` — atom `SUMMARY/DEP-011` — anchor `N-AGGREGATE.md:L1600`
  - **SUMMARY/DEP-011** [E] anchor=`14-N:L795` (14-N-SUMMARY_INDEX.discovery.md:L795): N-11 extends L-16 and consumes J-31/J-60/J-98/K-23, I-39, K-24, M-3, K-22, J-68, H-49, ND-2, and GD-2.

#### `I-11` — BeliefState (4 families)

- **J** `consumes` — atom `J-006/IX-010` — anchor `J-AGGREGATE.md:L3761`
  - **J-006/IX-010** [IX] (DEEP-J-006.discovery.md:L402 | unit=Cross-file interaction records): E | J-83 completes I-11 BeliefState; J-85 consumes commit-freeze; J-86 layers receipt-gated candidates; J-84 sizes effort into
- **K** `extends` — atom `K-003/IX-009` — anchor `K-AGGREGATE.md:L391`
  - **K-003/IX-009** [IX] (DEEP-K-003.discovery.md:L373 | SURFACE): IX | K12-E004, K15-E017 | K-12 extends I-11 BeliefState with critical_atoms; J-83 (OUT-OF-K via K-15) is the typed BeliefState reasoning carrier — composi
- **L** `interacts` — atom `L-003/IX-011` — anchor `L-AGGREGATE.md:L1744`
  - **L-003/IX-011** [E] [interaction] (ledger-line: DEEP-L-003.discovery.md:L430): L-17 PressureCeiling hands off to L-16 BeliefState projection variant (I-11). _(evidence: L17-E009; L16-E013)_
- **Q** `composes` — atom `Q-002/IX-004` — anchor `Q-AGGREGATE.md:L817`
  - **Q-002/IX-004** [IX] class=`E` evidence=`Q2-E011; Q3-E005, Q3-E010; Q4-E009, Q4-E023` (ledger-line: DEEP-Q-002.discovery.md:L367): Q-2/Q-3/Q-4 all consume I-11 BeliefState and/or I-25 routing and compose O-2/4G/4M/J s

#### `I-12` — curator (must not reimplement 3F lifecycle) (4 families)

- **J** `depends` — atom `J-004/IX-016` — anchor `J-AGGREGATE.md:L2943`
  - **J-004/IX-016** [IX] (DEEP-J-004.discovery.md:L417 | unit=Cross-source interactions and lifecycle): E | J-97 depends on J-96 store/recurrence first; fitness reuses H-43/H-45/H-48, D-6/H-47 held-out, D-13 cross-family,
- **L** `composes` — atom `L-001/IX-007` — anchor `L-AGGREGATE.md:L1697`
  - **L-001/IX-007** [E] [interaction] (ledger-line: DEEP-L-001.discovery.md:L444): L-12 curator promote-on-k composes I-12 and feeds J-96 episodic→skill→rule ladder also named as L-6 case-mint eligibility after targeted g
- **N** `feeds` — atom `N-001/IX-017` — anchor `N-AGGREGATE.md:L768`
  - **N-001/IX-017** [E] (DEEP-N-001.discovery.md:L401): N-12(a) FailureClass feeds auditor, anti-spiral, MAST, corpus rows, and case edges as one classification event — training-data free for J-25/I-12. | evidence=N12-E00
- **PHASE** `interacts` — atom `PHASE-005/IX-006` — anchor `PHASE-AGGREGATE.md:L1758`
  - PHASE-005/IX-006 | E | 3F lifecycle PROPOSED→…→ACTIVE is the machine I-12 curator must not reimplement (cross-family note; source owns SM). | P3F-E018, P3F-E019 |

#### `K-2` — routing-economics / shard harness (3 families)

- **N** `interacts` — atom `N-002/N14-E030` — anchor `N-AGGREGATE.md:L675`
  - **N-002/N14-E030** [E/DEP] anchor=`N-14:L113-L113` unit=`N-14 — self-improvement edit-loop instruments` (ledger-line: DEEP-N-002.discovery.md:L333): Sequences behind EvoPolicyGym bench candidate (S2-4, recorded as K-2 
- **P** `consumes` — atom `P-003/P17-E012` — anchor `P-AGGREGATE.md:L720`
  - **P-003/P17-E012** [E] class=`E/DEP` anchor=`P-17:L11-L17, L33-L35, L67-L69` unit=`P-17` (ledger-line: DEEP-P-003.discovery.md:L117): Consumes UsageTurnEvent (J-28) and AuthBroker (I-50); B-37 fallback chains consume; 
- **Q** `extends` — atom `Q-002/IX-014` — anchor `Q-AGGREGATE.md:L182`
  - **Q-002/IX-014** anchor=`Q2-E016; Q3-E016; Q4-E005` (ledger-line: DEEP-Q-002.discovery.md:L377): Q-3 K-2 `--arm advisor` and Q-2 `elai bench wake-gate` extend the same routing-economics measurement family that Q-4 rest

#### `E-48` — VerifiedCitation (4 families)

- **H** `gates` — atom `H-004/IX-015` — anchor `H-AGGREGATE.md:L2890`
  - **H-004/IX-015** [IX] (DEEP-H-004.discovery.md:L518): H-53 inserts a precondition before E-48's verifier, reuses H-52's resolver, and delegates ungroundable spans to D-16/D-17. | H-53:L53-L57; H53-E013/H53-E014/H53-E01
- **J** `depends` — atom `J-001/DJ001-40-001` — anchor `J-AGGREGATE.md:L1580`
  - **J-001/DJ001-40-001** [SRC] (DEEP-J-001.discovery.md:L123 | unit=3. J-40_research_modes_extractive_compression.md — 52/52 lines): [E|status/dependency] J-40 is a medium DD-2 build activating E-48 live search and exten
- **P** `interacts` — atom `P-003/IX-015` — anchor `P-AGGREGATE.md:L1734`
  - **P-003/IX-015** [IX] class=`E` (ledger-line: DEEP-P-003.discovery.md:L334): P-25 retrieval substrate: elai.map anchors, sealed bench rows, E-48 VerifiedCitation, feature-map ledger — read-only. _(evidence: P25-E008, P
- **Q** `consumes` — atom `Q-002/Q10-E015` — anchor `Q-AGGREGATE.md:L142`
  - **Q-002/Q10-E015** anchor=`Q-10:L19-L28, L103-L110, L163-L165` (ledger-line: DEEP-Q-002.discovery.md:L229): Consumes B-40/Phase 4D ResearchReport/SourceCitation/SubQuestionResult; E-48 VerifiedCitation; B-42/O-1 Cosmos

#### `J-96` — CaseNode / MintOnProof (3 families)

- **L** `composes` — atom `L-001/IX-007` — anchor `L-AGGREGATE.md:L1697`
  - **L-001/IX-007** [E] [interaction] (ledger-line: DEEP-L-001.discovery.md:L444): L-12 curator promote-on-k composes I-12 and feeds J-96 episodic→skill→rule ladder also named as L-6 case-mint eligibility after targeted g
- **N** `composes` — atom `N-001/IX-014` — anchor `N-AGGREGATE.md:L765`
  - **N-001/IX-014** [E] (DEEP-N-001.discovery.md:L398): N-4 memo composes C-30 cache shape, J-96 case nodes, J-21 identity, and F-5 invalidation; I-8 is the negative-space twin already caching failures. | evidence=N4-E007
- **P** `extends` — atom `P-003/IX-003` — anchor `P-AGGREGATE.md:L1722`
  - **P-003/IX-003** [IX] class=`E` (ledger-line: DEEP-P-003.discovery.md:L322): P-15 extends closed J-96 CaseNode/MintOnProof; reuse-hit feeds K-18/PS-5; bBoN (Tier-2) slots behind I evidence-ranked search after receipts.

#### `L-12` — skill admission / skillspector family (2 families)

- **J** `interacts` — atom `J-002/DJ002-CON-002` — anchor `J-AGGREGATE.md:L2299`
  - **J-002/DJ002-CON-002** [AC] (DEEP-J-002.discovery.md:L442 | unit=Conflicts and material mismatches): [C|scope/integration] J-109's objective includes skill L-12 as a fronted write seam, but its task, consumer, observa
- **P** `interacts` — atom `P-003/IX-018` — anchor `P-AGGREGATE.md:L1737`
  - **P-003/IX-018** [IX] class=`I` (ledger-line: DEEP-P-003.discovery.md:L337): P-3 marketplace/friend sharing (Tier-2) and P-22 control-surface pairing UX (Tier-2) are distinct distribution vs device-enrollment surfaces 

#### `D-16` — refusal / calibration (RefusalDecision) (5 families)

- **H** `interacts` — atom `H-004/H53-E015` — anchor `H-AGGREGATE.md:L2840`
  - **H-004/H53-E015** [SRC] (DEEP-H-004.discovery.md:L468): E/DEP | Read D-16's existing conformal abstain bit and D-17 refusal SLA; do not rederive refusal. | H-53:L11-L11, L20-L20
- **I** `interacts` — atom `I-004/IX-013` — anchor `I-AGGREGATE.md:L2798`
  - **I-004/IX-013** [IX] (DEEP-I-004.discovery.md:L488| unit=RELATIONAL): E | I-34’s runtime chain deliberately separates XGrammar token masking, Rust/provider CRANE control, mandatory post-parse validation, and D-16 refu
- **J** `consumes` — atom `J-011/IX-005` — anchor `J-AGGREGATE.md:L5684`
  - **J-011/IX-005** [IX] (DEEP-J-011.discovery.md:L386 | unit=Cross-source interactions and lifecycle): E | J-42 extends GRC reset_topic and B-39 refine; consumes H-50 context fraction, H-42 budget, D-16 abstention, I-7 e
- **L** `composes` — atom `L-002/L0-E018` — anchor `L-AGGREGATE.md:L449`
  - **L-002/L0-E018** [E/DEP] anchor=`L-0:L46-L46` unit=`L-0 — L-Plan Index (doctrine, tracks, sequencing, constants)` (ledger-line: DEEP-L-002.discovery.md:L53): Inherits GD-1/GD-2/GD-4/GD-5: compose-not-restate (re-imple
- **N** `extends` — atom `N-001/N12-E008` — anchor `N-AGGREGATE.md:L387`
  - **N-001/N12-E008** [E/DEP] anchor=`N-12:L9-L9` unit=`N-12 — consistency + dedupe bundle` (ledger-line: DEEP-N-001.discovery.md:L352): L-7 already composes D-16+I-8+D-17; RD-1 mandates read-never-re-derive — extend, don

#### `H-9` — spend ledger / cost accounting (5 families)

- **I** `interacts` — atom `I-004/IX-012` — anchor `I-AGGREGATE.md:L2797`
  - **I-004/IX-012** [IX] (DEEP-I-004.discovery.md:L487| unit=RELATIONAL): E | I-27 imports pass/cost definitions from E-49, runner shape from E-9, I-4 node telemetry, I-38 statistics, and H-9 display; none may be redefine
- **J** `interacts` — atom `J-005/IX-021` — anchor `J-AGGREGATE.md:L1354`
  - **J-005/IX-021** [IX] (DEEP-J-005.discovery.md:L418 | unit=Cross-file interaction records | SURFACE): E | User-facing surface cluster for dashboard DESIGN.md: memory-hub provenance (J-21); runtime/context steering+budg
- **K** `composes` — atom `K-003/K15-E007` — anchor `K-AGGREGATE.md:L474`
  - **K-003/K15-E007** [SRC] (DEEP-K-003.discovery.md:L189 | SURFACE): E/DEP | Composes I-27 ConvergenceCurveMeasured + bench_convergence (DARK); J-34 `elai bench context-fill` sibling shape; safedelta_runner + args_bench.
- **M** `extends` — atom `M-001/M3-E028` — anchor `M-AGGREGATE.md:L469`
  - **M-001/M3-E028** [E/DEP] anchor=`M-3:L48-L51` (ledger-line: DEEP-M-001.discovery.md:L254): COMPOSES-WITH: K-6 export (input shape — extended, not forked); G-2 RRA + G-8 attestation verify core (the --verify leg); G-9 
- **Q** `consumes` — atom `Q-002/Q3-E010` — anchor `Q-AGGREGATE.md:L585`
  - **Q-002/Q3-E010** [SRC] class=`E/DEP` anchor=`Q-3:L14-L20, L59-L60, L93-L94` (ledger-line: DEEP-Q-002.discovery.md:L122): Consumes H-7 split, I-7/I-11 BeliefState, I-25 router (pre-dispatch only), 4M RuntimeAdapter, H-

#### `F-1` — VibeGate / Layer-1 external review gate family (6 families)

- **G** `interacts` — atom `G-002/IX-011` — anchor `G-AGGREGATE.md:L1232`
  - **G-002/IX-011** [E] [interaction] label=`F-1 → G-9` refs=`DG002-G9-007, DG002-G9-065`: One `SECRET_PATTERNS` detector is reused for both pre-write blocking and persistence redaction.
- **H** `interacts` — atom `H-005/H005-52-033` — anchor `H-AGGREGATE.md:L3236`
  - **H-005/H005-52-033** [SRC] (DEEP-H-005.discovery.md:L404): E-STATE/INT | L15-L18 | Planned consumers/producers are G-5 `HunkVouch`/scorer, C-7 header-hash ID, F-4 stdout hash, F-1 CWE, C-1 grounding sweep, and E-41 re
- **I** `consumes` — atom `I-006/IX-003` — anchor `I-AGGREGATE.md:L3701`
  - **I-006/IX-003** [IX] (DEEP-I-006.discovery.md:L464| unit=RELATIONAL): I | I-19’s pre-write predicate composes F-1, which I-17 later consumes as an L1 axis; therefore a source edit rejected by I-19 should never become 
- **J** `reuses` — atom `J-006/IX-015` — anchor `J-AGGREGATE.md:L3766`
  - **J-006/IX-015** [IX] (DEEP-J-006.discovery.md:L407 | unit=Cross-file interaction records): E | J-102 reuses J-52 PathRuleInjector, E-24 pins, E-43 skill policy, F-1 VibeGate, D-13 advisory critic. | J102-E005–E006, J1
- **L** `gates` — atom `L-001/L12-E013` — anchor `L-AGGREGATE.md:L288`
  - **L-001/L12-E013** [E/DEP] [SURFACE] anchor=`L-12:L8-L8` unit=`L-12 — skill front-door + self-improving skills` (ledger-line: DEEP-L-001.discovery.md:L277): Prerequisites: E-43 skill_system_event + agentskills_validato
- **N** `gates` — atom `N-002/N6-E007` — anchor `N-AGGREGATE.md:L497`
  - **N-002/N6-E007** [E/DEP] anchor=`N-6:L9-L9, L15-L15, L32-L32` unit=`N-6 — provenance / trust lattice + admission-gate family` (ledger-line: DEEP-N-002.discovery.md:L125): They already share legs (J-102 reuses F-1's se

#### `B-29` — blast-radius / decomposition UI substrate (closed by C-24) (5 families)

- **C** `depends` — atom `SUMMARY/S-E0240` — anchor `C-AGGREGATE.md:L1506`
  - **SUMMARY/S-E0240** [feature] [E] (unit=C-18, ledger=SUMMARY | anchor: L226-L229): Closes B-2, B-29, B-32, B-33, B-35, B-36, phase-3B, phase-3D, and phase-3H; depends on C-1, C-2, and C-13; is parallel-safe with C-19 a
- **E** `depends` — atom `E-005/E005-E52-002` — anchor `E-AGGREGATE.md:L1521`
  - **E-005/E005-E52-002** [SRC] SURFACE (unit=E-52_lsp_integration | anchor: DEEP-E-005.discovery.md:L375): [DEP/STATUS] It depends on planned B-19 PostToolUse and B-29 symbol graph, coordinates with E-40's elai.map consu
- **J** `consumes` — atom `J-010/IX-007` — anchor `J-AGGREGATE.md:L5282`
  - **J-010/IX-007** [IX] (DEEP-J-010.discovery.md:L457 | unit=Cross-source interactions and lifecycle): E | J-39 wraps base-4E DefaultPhaseSizingSplitter, consumes B-29 cone and J-34 ceilings (with baseline fallback), esc
- **K** `composes` — atom `K-004/K5-E020` — anchor `K-AGGREGATE.md:L2005`
  - **K-004/K5-E020** [SRC/E] (DEEP-K-004.discovery.md:L116): `E/DEP` — Composes B-22/C-30/J-16/F-2/D-15/B-29 signals and F-1/F-9/E-30/G-1/I-20/H-26/E-53 legs; re-grep live symbols W-7; KG-8 inventory context. — plan-ancho
- **L** `composes` — atom `L-001/L6-E006` — anchor `L-AGGREGATE.md:L217`
  - **L-001/L6-E006** [E/DEP] anchor=`L-6:L5-L5, L39-L39, L48-L48` unit=`L-6 — risk-as-curiosity targeted-effort router` (ledger-line: DEEP-L-001.discovery.md:L194): GD-1: do not re-implement J-38/J-103/I-7 — dispatch them

#### `J-16` note (blast radius)

- Named as optional/present substrate from **K** (`K-002/K3-E014` @ `K-AGGREGATE.md:L325`: consumes F-4, G-8, **J-16 when present**).
- Named as L-5 prerequisite from **L** (`L-003/L5-E006` @ `L-AGGREGATE.md:L796`; live plan notes **J-16 WIRED** in `L-003/L5-C001`).
- Fan-in is lower than F-4/G-1 in the automated signal set because many units say "blast radius" without the `J-16` id; treat as hub-adjacent to B-29/C-24 blast-radius UI and G-3 BLAST-RADIUS mediation.

---

## Edges by consuming family

Grouped by the family that **depends on / consumes / extends / gates with** other families. Each edge: `→ provider` · relation · atom · anchor.

For large families, the list is the **highest-signal** structural pairs (IX/DEP preferred), not every raw mention.

### PHASE — Core engine phases (0–4*): FSM, SSB backbone, agents, research, autonomy

_Structural pairs from this family: **3** (showing top 3 by evidence score)._

**→ family B**
- → `B-16A` · **supersedes** · `PHASE-008/IX-030` · `PHASE-AGGREGATE.md:L2950`

**→ family C**
- → `C-30` · **interacts** · `PHASE-002/IX-014` · `PHASE-AGGREGATE.md:L753`

**→ family I**
- → `I-12` · **interacts** · `PHASE-005/IX-006` · `PHASE-AGGREGATE.md:L1758`


### B — Dashboard V2 harness, runtime adapters, research/Cosmos UI wiring

_Structural pairs from this family: **8** (showing top 8 by evidence score)._

**→ family C**
- → `C-33` · **supersedes** · `B20-001` · `B-AGGREGATE.md:L853`
- → `C-47` · **extends** · `B0-080` · `B-AGGREGATE.md:L131`
- → `C-47b` · **extends** · `B0-080` · `B-AGGREGATE.md:L131`

**→ family E**
- → `E-1` · **interacts** · `B005-F05-001` · `B-AGGREGATE.md:L1172`
- → `E-7` · **interacts** · `B005-F05-001` · `B-AGGREGATE.md:L1172`

**→ family PHASE**
- → `phase-3C` · **supersedes** · `B16-014` · `B-AGGREGATE.md:L730`
- → `phase-4F` · **interacts** · `B21-001` · `B-AGGREGATE.md:L883`
- → `phase-4H` · **interacts** · `B23-001` · `B-AGGREGATE.md:L972`


### C — Closes B/phase gaps: drawers, IPC, gates, blast-radius UI, surgical modes

_Structural pairs from this family: **58** (showing top 40 by evidence score)._

**→ family B**
- → `B-0` · **depends** · `SUMMARY/S-E0544` · `C-AGGREGATE.md:L2449`
- → `B-11` · **depends** · `SUMMARY/S-E0089` · `C-AGGREGATE.md:L919`
- → `B-12` · **depends** · `SUMMARY/S-E0089` · `C-AGGREGATE.md:L919`
- → `B-13` · **depends** · `SUMMARY/S-E0036` · `C-AGGREGATE.md:L702`
- → `B-14` · **depends** · `SUMMARY/S-E0089` · `C-AGGREGATE.md:L919`
- → `B-15` · **depends** · `SUMMARY/S-E0193` · `C-AGGREGATE.md:L1357`
- → `B-16` · **depends** · `SUMMARY/S-E0193` · `C-AGGREGATE.md:L1357`
- → `B-18` · **depends** · `SUMMARY/S-E0145` · `C-AGGREGATE.md:L1166`
- → `B-19` · **depends** · `SUMMARY/S-E0170` · `C-AGGREGATE.md:L1262`
- → `B-2` · **depends** · `SUMMARY/S-E0240` · `C-AGGREGATE.md:L1506`
- → `B-20` · **depends** · `SUMMARY/S-E0170` · `C-AGGREGATE.md:L1262`
- → `B-21` · **depends** · `SUMMARY/S-E0300` · `C-AGGREGATE.md:L1745`
- → `B-22` · **depends** · `SUMMARY/S-E0115` · `C-AGGREGATE.md:L1051`
- → `B-23` · **depends** · `SUMMARY/S-E0284` · `C-AGGREGATE.md:L1701`
- → `B-26` · **depends** · `SUMMARY/S-E0056` · `C-AGGREGATE.md:L793`
- → `B-27` · **depends** · `SUMMARY/S-E0062` · `C-AGGREGATE.md:L133`
- → `B-29` · **depends** · `SUMMARY/S-E0240` · `C-AGGREGATE.md:L1506`
- → `B-29.5` · **depends** · `SUMMARY/S-E0269` · `C-AGGREGATE.md:L1644`
- → `B-3` · **depends** · `SUMMARY/S-E0049` · `C-AGGREGATE.md:L760`
- → `B-30` · **depends** · `SUMMARY/S-E0222` · `C-AGGREGATE.md:L1464`
- → `B-31` · **depends** · `SUMMARY/S-E0222` · `C-AGGREGATE.md:L1464`
- → `B-31.5` · **depends** · `SUMMARY/S-E0222` · `C-AGGREGATE.md:L1464`
- → `B-32` · **depends** · `SUMMARY/S-E0240` · `C-AGGREGATE.md:L1506`
- → `B-33` · **depends** · `SUMMARY/S-E0240` · `C-AGGREGATE.md:L1506`
- → `B-34` · **depends** · `SUMMARY/S-E0222` · `C-AGGREGATE.md:L1464`
- → `B-35` · **depends** · `SUMMARY/S-E0206` · `C-AGGREGATE.md:L1405`
- → `B-36` · **depends** · `SUMMARY/S-E0056` · `C-AGGREGATE.md:L793`
- → `B-38` · **depends** · `SUMMARY/S-E0036` · `C-AGGREGATE.md:L702`
- → `B-39` · **depends** · `SUMMARY/S-E0544` · `C-AGGREGATE.md:L2449`
- → `B-4` · **depends** · `SUMMARY/S-E0049` · `C-AGGREGATE.md:L760`
- → `B-40` · **depends** · `SUMMARY/S-E0544` · `C-AGGREGATE.md:L2449`
- → `B-41` · **depends** · `SUMMARY/S-E0158` · `C-AGGREGATE.md:L1205`
- → `B-42` · **depends** · `SUMMARY/S-E0036` · `C-AGGREGATE.md:L702`
- → `B-5` · **depends** · `SUMMARY/S-E0049` · `C-AGGREGATE.md:L760`
- → `B-6` · **depends** · `SUMMARY/S-E0036` · `C-AGGREGATE.md:L702`
- → `B-7` · **depends** · `SUMMARY/S-E0036` · `C-AGGREGATE.md:L702`
- → `B-9` · **depends** · `SUMMARY/S-E0254` · `C-AGGREGATE.md:L1576`

**→ family PHASE**
- → `phase-0` · **depends** · `SUMMARY/S-E0193` · `C-AGGREGATE.md:L1357`
- → `phase-3B` · **depends** · `SUMMARY/S-E0240` · `C-AGGREGATE.md:L1506`
- → `phase-3D` · **depends** · `SUMMARY/S-E0240` · `C-AGGREGATE.md:L1506`


### D — Reasoning templates, mode routing, verification, novelty, calibration

_Structural pairs from this family: **12** (showing top 12 by evidence score)._

**→ family B**
- → `B-37` · **consumes** · `D:L746` · `D-AGGREGATE.md:L746`

**→ family C**
- → `C-15` · **interacts** · `D-001/IX-003` · `D-AGGREGATE.md:L388`
- → `C-17` · **consumes** · `DEP/INT` · `D-AGGREGATE.md:L188`
- → `C-24` · **consumes** · `SUM-D-30-073` · `D-AGGREGATE.md:L2090`
- → `C-30` · **consumes** · `SUM-D-30-073` · `D-AGGREGATE.md:L2090`
- → `C-12` · **extends** · `SUM-D-30-075` · `D-AGGREGATE.md:L2092`
- → `C-21` · **extends** · `SUM-D-30-070` · `D-AGGREGATE.md:L2087`
- → `C-22` · **extends** · `SUM-D-30-076` · `D-AGGREGATE.md:L2093`
- → `C-27` · **gates** · `DET/TEST/DEP` · `D-AGGREGATE.md:L62`
- → `C-31` · **extends** · `SUM-D-30-074` · `D-AGGREGATE.md:L2091`

**→ family H**
- → `H-47` · **composes** · `D-001/IX-009` · `D-AGGREGATE.md:L394`

**→ family PHASE**
- → `phase-4F` · **extends** · `SUM-D-18-001` · `D-AGGREGATE.md:L1885`


### E — TDD agents, browser, citations, completeness, memory adaptation

_Structural pairs from this family: **63** (showing top 45 by evidence score)._

**→ family B**
- → `B-20` · **consumes** · `E-006/E006-E24-002` · `E-AGGREGATE.md:L1656`
- → `B-3` · **consumes** · `E-006/E006-E24-002` · `E-AGGREGATE.md:L1656`
- → `B-37` · **consumes** · `E-006/E006-E24-002` · `E-AGGREGATE.md:L1656`
- → `B-18` · **consumes** · `004/E004` · `E-AGGREGATE.md:L3799`
- → `B-19` · **depends** · `E-005/E005-E52-002` · `E-AGGREGATE.md:L1521`
- → `B-22` · **consumes** · `004/E004` · `E-AGGREGATE.md:L1059`
- → `B-23` · **depends** · `E-006/E006-E46-002` · `E-AGGREGATE.md:L1787`
- → `B-26` · **consumes** · `004/E004` · `E-AGGREGATE.md:L1059`
- → `B-29` · **depends** · `E-005/E005-E52-002` · `E-AGGREGATE.md:L1521`
- → `B-31` · **consumes** · `E-003/E003-E32-035` · `E-AGGREGATE.md:L3369`
- → `B-35` · **consumes** · `E-001/DEEP-E001-INT-008` · `E-AGGREGATE.md:L2408`
- → `B-40` · **consumes** · `E-005/E005-E50-002` · `E-AGGREGATE.md:L1477`
- → `B-41` · **depends** · `E-006/E006-E23-002` · `E-AGGREGATE.md:L1638`
- → `B-1` · **supersedes** · `E-002/E17-008` · `E-AGGREGATE.md:L2675`
- → `B-16` · **extends** · `E-002/E34-002` · `E-AGGREGATE.md:L473`
- → `B-22b` · **extends** · `E-005/E005-E13-002` · `E-AGGREGATE.md:L1284`
- → `B-24` · **extends** · `E-002/C-09` · `E-AGGREGATE.md:L664`
- → `B-27` · **extends** · `E-002/E39-002` · `E-AGGREGATE.md:L500`
- → `B-38` · **supersedes** · `003/E003-E4-002` · `E-AGGREGATE.md:L6800`
- → `B-4` · **supersedes** · `E-002/E17-008` · `E-AGGREGATE.md:L2675`
- → `B-7` · **extends** · `004/E004` · `E-AGGREGATE.md:L1216`
- → `B-9` · **extends** · `E-002/E3-006` · `E-AGGREGATE.md:L351`

**→ family C**
- → `C-13` · **consumes** · `E-003/E003-E32-035` · `E-AGGREGATE.md:L3369`
- → `C-21` · **consumes** · `E-002/E1-043` · `E-AGGREGATE.md:L347`
- → `C-27` · **consumes** · `E-001/DEEP-E001-INT-010` · `E-AGGREGATE.md:L2410`
- → `C-35` · **consumes** · `004/E004` · `E-AGGREGATE.md:L1207`
- → `C-39` · **consumes** · `E-005/E005-X-008` · `E-AGGREGATE.md:L4614`
- → `C-40` · **depends** · `004/E004` · `E-AGGREGATE.md:L4106`
- → `C-48` · **depends** · `E-006/E006-E46-002` · `E-AGGREGATE.md:L1787`
- → `C-52` · **consumes** · `E-002/X-09` · `E-AGGREGATE.md:L625`
- → `C-7` · **consumes** · `E-001/DEEP-E001-INT-008` · `E-AGGREGATE.md:L2408`
- → `C-03` · **supersedes** · `002/SD-E1-09` · `E-AGGREGATE.md:L6783`
- → `C-06` · **supersedes** · `E-002/C-06` · `E-AGGREGATE.md:L2955`
- → `C-09` · **extends** · `E-002/C-09` · `E-AGGREGATE.md:L664`
- → `C-10` · **supersedes** · `002/C-10` · `E-AGGREGATE.md:L6779`
- → `C-11` · **extends** · `004/E004` · `E-AGGREGATE.md:L1213`

**→ family D**
- → `D-1` · **consumes** · `E-005/E005-E52-021` · `E-AGGREGATE.md:L1540`
- → `D-13` · **consumes** · `E-002/E53-027` · `E-AGGREGATE.md:L600`
- → `D-14` · **consumes** · `E-002/E1-043` · `E-AGGREGATE.md:L347`
- → `D-24` · **consumes** · `E-003/E003-E54-055` · `E-AGGREGATE.md:L890`
- → `D-28` · **depends** · `E-006/E006-E47-002` · `E-AGGREGATE.md:L1812`
- → `D-3` · **consumes** · `E-002/E53-027` · `E-AGGREGATE.md:L600`
- → `D-30` · **depends** · `E-006/E006-E47-002` · `E-AGGREGATE.md:L1812`
- → `D-7` · **consumes** · `E-001/DEEP-E001-INT-010` · `E-AGGREGATE.md:L2410`

**→ family PHASE**
- → `phase-3H` · **consumes** · `004/E004` · `E-AGGREGATE.md:L1059`


### F — Cant-lie gates (F-4), VibeGate (F-1), quarantine (F-5), trust (F-6)

_Structural pairs from this family: **7** (showing top 7 by evidence score)._

**→ family C**
- → `C-27` · **depends** · `SUMMARY/F4-18` · `F-AGGREGATE.md:L1248`
- → `C-28` · **depends** · `SUMMARY/F4-19` · `F-AGGREGATE.md:L1249`

**→ family D**
- → `D-11` · **extends** · `SUMMARY/F9-09` · `F-AGGREGATE.md:L1381`

**→ family E**
- → `E-43` · **consumes** · `F-002/DF002-F1-046` · `F-AGGREGATE.md:L235`
- → `E-41` · **extends** · `SUMMARY/F9-10` · `F-AGGREGATE.md:L1382`
- → `E-29` · **composes** · `F-001/DF001-F4-E032` · `F-AGGREGATE.md:L505`

**→ family PHASE**
- → `phase-2` · **supersedes** · `SUMMARY/F3` · `F-AGGREGATE.md:L1889`


### G — Trust spine: taint (G-1), RRA (G-2), attest (G-8), redact (G-9), drift (G-10)

_Structural pairs from this family: **13** (showing top 13 by evidence score)._

**→ family PHASE/core**
- → `SSB` · **interacts** · `G-002/IX-015` · `G-AGGREGATE.md:L1236`

**→ family C**
- → `C-1` · **interacts** · `G-001/DG001-G5-E043` · `G-AGGREGATE.md:L326`

**→ family D**
- → `D-1` · **interacts** · `G-002/IX-014` · `G-AGGREGATE.md:L1235`
- → `D-7` · **interacts** · `G-002/IX-012` · `G-AGGREGATE.md:L1233`

**→ family E**
- → `E-30` · **consumes** · `G-001/DG001-G3-E001` · `G-AGGREGATE.md:L169`
- → `E-43` · **extends** · `SUMMARY/DEP-06` · `G-AGGREGATE.md:L1195`

**→ family F**
- → `F-4` · **consumes** · `G-002/IX-009` · `G-AGGREGATE.md:L1230`
- → `F-9` · **consumes** · `G-002/IX-009` · `G-AGGREGATE.md:L1230`
- → `F-1` · **interacts** · `G-002/IX-011` · `G-AGGREGATE.md:L1232`
- → `F-2b` · **interacts** · `G-002/IX-006` · `G-AGGREGATE.md:L1227`
- → `F-10` · **consumes** · `G-001/DG001-G0-E017` · `G-AGGREGATE.md:L58`
- → `F-2` · **consumes** · `G-002/DG002-G6-010` · `G-AGGREGATE.md:L491`
- → `F-2a` · **gates** · `G-001/DG001-G5-E003` · `G-AGGREGATE.md:L281`


### H — UX, spend, retirement, fitness, compaction, finance-scoped compositions

_Structural pairs from this family: **39** (showing top 39 by evidence score)._

**→ family PHASE/core**
- → `SSB` · **mirrors** · `H-001/IX-013` · `H-AGGREGATE.md:L1391`

**→ family B**
- → `B-22` · **composes** · `H-007/H007-37-031` · `H-AGGREGATE.md:L3910`
- → `B-6` · **composes** · `H-007/H007-37-031` · `H-AGGREGATE.md:L3910`

**→ family C**
- → `C-23` · **extends** · `H-004/IX-014` · `H-AGGREGATE.md:L2889`
- → `C-26` · **interacts** · `H-004/IX-016` · `H-AGGREGATE.md:L2891`
- → `C-1` · **consumes** · `H-005/H005-52-038` · `H-AGGREGATE.md:L3241`
- → `C-40` · **depends** · `H-005/H005-52-001` · `H-AGGREGATE.md:L826`
- → `C-7` · **depends** · `H-005/H005-52-001` · `H-AGGREGATE.md:L826`
- → `C-35` · **composes** · `H-007/H007-34-001` · `H-AGGREGATE.md:L3842`

**→ family D**
- → `D-13` · **composes** · `H-002/IX-006` · `H-AGGREGATE.md:L1841`
- → `D-17` · **gates** · `H-004/IX-015` · `H-AGGREGATE.md:L2890`
- → `D-2` · **interacts** · `H-004/IX-011` · `H-AGGREGATE.md:L2886`
- → `D-9` · **interacts** · `H-001/IX-009` · `H-AGGREGATE.md:L1387`
- → `D-16` · **interacts** · `H-004/H53-E015` · `H-AGGREGATE.md:L2840`
- → `D-24` · **composes** · `H-005/H005-44-012` · `H-AGGREGATE.md:L3107`
- → `D-3` · **composes** · `H-002/H40-023` · `H-AGGREGATE.md:L1708`
- → `D-5` · **composes** · `H-001/H46-005` · `H-AGGREGATE.md:L1350`
- → `D-6` · **composes** · `H-005/H005-47-002` · `H-AGGREGATE.md:L3130`

**→ family E**
- → `E-29` · **extends** · `H-004/IX-014` · `H-AGGREGATE.md:L2889`
- → `E-44` · **extends** · `H-004/IX-014` · `H-AGGREGATE.md:L2889`
- → `E-48` · **gates** · `H-004/IX-015` · `H-AGGREGATE.md:L2890`
- → `E-19` · **interacts** · `H-004/IX-010` · `H-AGGREGATE.md:L2885`
- → `E-25` · **interacts** · `H-001/IX-009` · `H-AGGREGATE.md:L1387`
- → `E-53` · **interacts** · `H-002/IX-007` · `H-AGGREGATE.md:L1842`
- → `E-54` · **consumes** · `H-001/VG-008` · `H-AGGREGATE.md:L1148`
- → `E-18` · **interacts** · `H-005/H005-04-027` · `H-AGGREGATE.md:L2927`
- → `E-40` · **interacts** · `H-005/H005-04-027` · `H-AGGREGATE.md:L2927`
- → `E-38` · **composes** · `H-007/H007-37-031` · `H-AGGREGATE.md:L3910`
- → `E-41` · **interacts** · `H-005/H005-52-033` · `H-AGGREGATE.md:L3236`

**→ family F**
- → `F-12` · **extends** · `H-003/DH003-IX-011` · `H-AGGREGATE.md:L982`
- → `F-13` · **extends** · `H-004/IX-014` · `H-AGGREGATE.md:L2889`
- → `F-4` · **depends** · `H-005/H005-52-001` · `H-AGGREGATE.md:L826`
- → `F-3p1` · **composes** · `H-007/H007-13-014` · `H-AGGREGATE.md:L3667`
- → `F-1` · **interacts** · `H-005/H005-52-033` · `H-AGGREGATE.md:L3236`

**→ family G**
- → `G-2` · **extends** · `H-004/IX-014` · `H-AGGREGATE.md:L2889`
- → `G-5` · **depends** · `H-005/H005-52-001` · `H-AGGREGATE.md:L826`
- → `G-1` · **composes** · `H-002/HTG-010` · `H-AGGREGATE.md:L1531`

**→ family J**
- → `J-11` · **gates** · `H-004/H43-E038` · `H-AGGREGATE.md:L2728`

**→ family PHASE**
- → `phase-3E` · **interacts** · `H-004/IX-016` · `H-AGGREGATE.md:L2891`


### I — Belief, routing, curator, intervention cards, evidence search

_Structural pairs from this family: **79** (showing top 45 by evidence score)._

**→ family PHASE/core**
- → `SSB` · **interacts** · `I-001/IX-007` · `I-AGGREGATE.md:L1163`

**→ family B**
- → `B-37` · **interacts** · `I-001/IX-014` · `I-AGGREGATE.md:L1170`
- → `B-38` · **consumes** · `I-003/DI003-I41-035` · `I-AGGREGATE.md:L2202`
- → `B-40` · **consumes** · `I-003/DI003-I41-035` · `I-AGGREGATE.md:L2202`

**→ family C**
- → `C-37` · **consumes** · `I-004/IX-015` · `I-AGGREGATE.md:L2800`
- → `C-29` · **composes** · `I-001/IX-017` · `I-AGGREGATE.md:L1173`
- → `C-50` · **composes** · `I-001/IX-017` · `I-AGGREGATE.md:L1173`
- → `C-23` · **extends** · `I-004/IX-005` · `I-AGGREGATE.md:L548`
- → `C-28` · **gates** · `I-004/IX-008` · `I-AGGREGATE.md:L550`
- → `C-11` · **composes** · `I-004/I2-E021` · `I-AGGREGATE.md:L167`

**→ family D**
- → `D-13` · **composes** · `I-004/IX-010` · `I-AGGREGATE.md:L2795`
- → `D-16` · **interacts** · `I-004/IX-013` · `I-AGGREGATE.md:L2798`
- → `D-1` · **feeds** · `I-001/I48-E008` · `I-AGGREGATE.md:L1131`

**→ family E**
- → `E-37` · **extends** · `I-001/IX-005` · `I-AGGREGATE.md:L1161`
- → `E-15` · **extends** · `I-004/IX-005` · `I-AGGREGATE.md:L548`
- → `E-30` · **interacts** · `I-004/IX-016` · `I-AGGREGATE.md:L2801`
- → `E-49` · **interacts** · `I-004/IX-012` · `I-AGGREGATE.md:L2797`
- → `E-9` · **interacts** · `I-004/IX-012` · `I-AGGREGATE.md:L2797`
- → `E-44` · **consumes** · `I-003/DI003-I01-020` · `I-AGGREGATE.md:L1904`
- → `E-13` · **consumes** · `I-006/I19-E019` · `I-AGGREGATE.md:L3368`
- → `E-19` · **consumes** · `I-006/I26-E018` · `I-AGGREGATE.md:L3447`
- → `E-24` · **composes** · `I-004/I34-E025` · `I-AGGREGATE.md:L2584`
- → `E-26` · **consumes** · `I-006/I16-E018` · `I-AGGREGATE.md:L3284`
- → `E-28` · **composes** · `I-006/I16-E009` · `I-AGGREGATE.md:L129`
- → `E-32` · **composes** · `I-006/I16-E009` · `I-AGGREGATE.md:L129`
- → `E-39` · **consumes** · `I-002/I002-I10-038` · `I-AGGREGATE.md:L1633`
- → `E-8` · **composes** · `I-004/I12-E015` · `I-AGGREGATE.md:L2448`

**→ family F**
- → `F-1` · **consumes** · `I-006/IX-003` · `I-AGGREGATE.md:L3701`
- → `F-5` · **gates** · `I-004/IX-008` · `I-AGGREGATE.md:L550`
- → `F-3` · **interacts** · `I-004/IX-016` · `I-AGGREGATE.md:L2801`

**→ family G**
- → `G-9` · **extends** · `I-001/IX-005` · `I-AGGREGATE.md:L1161`
- → `G-6` · **interacts** · `I-006/IX-013` · `I-AGGREGATE.md:L3711`
- → `G-7` · **interacts** · `I-001/IX-009` · `I-AGGREGATE.md:L1165`

**→ family H**
- → `H-32` · **extends** · `I-001/IX-005` · `I-AGGREGATE.md:L1161`
- → `H-51` · **gates** · `I-001/IX-006` · `I-AGGREGATE.md:L1162`
- → `H-52` · **composes** · `I-001/IX-017` · `I-AGGREGATE.md:L1173`
- → `H-6` · **extends** · `I-001/IX-005` · `I-AGGREGATE.md:L1161`
- → `H-8` · **gates** · `I-001/IX-006` · `I-AGGREGATE.md:L1162`
- → `H-37` · **composes** · `I-004/IX-010` · `I-AGGREGATE.md:L2795`
- → `H-38` · **composes** · `I-004/IX-010` · `I-AGGREGATE.md:L2795`
- → `H-49` · **composes** · `I-004/IX-004` · `I-AGGREGATE.md:L2789`
- → `H-39` · **interacts** · `I-001/IX-002` · `I-AGGREGATE.md:L1158`
- → `H-9` · **interacts** · `I-004/IX-012` · `I-AGGREGATE.md:L2797`

**→ family PHASE**
- → `phase-3F` · **gates** · `I-004/IX-008` · `I-AGGREGATE.md:L550`
- → `phase-3A` · **interacts** · `I-004/IX-016` · `I-AGGREGATE.md:L2801`


### J — Harness redesign mega-family: perception, cases, free-API, blast radius, MCP

_Structural pairs from this family: **178** (showing top 55 by evidence score)._

**→ family B**
- → `B-39` · **consumes** · `J-011/IX-005` · `J-AGGREGATE.md:L5684`
- → `B-22` · **consumes** · `J-010/IX-002` · `J-AGGREGATE.md:L1395`
- → `B-29` · **consumes** · `J-010/IX-007` · `J-AGGREGATE.md:L5282`
- → `B-37` · **consumes** · `J-009/IX-005` · `J-AGGREGATE.md:L4820`
- → `B-24` · **feeds** · `J-004/IX-006` · `J-AGGREGATE.md:L1376`

**→ family C**
- → `C-28` · **extends** · `J-009/IX-006` · `J-AGGREGATE.md:L4821`
- → `C-48` · **extends** · `J-012/IX-008` · `J-AGGREGATE.md:L6068`
- → `C-10` · **composes** · `J-004/IX-008` · `J-AGGREGATE.md:L2935`
- → `C-39` · **extends** · `J-008/IX-018` · `J-AGGREGATE.md:L4453`

**→ family D**
- → `D-16` · **consumes** · `J-011/IX-005` · `J-AGGREGATE.md:L5684`
- → `D-13` · **depends** · `J-004/IX-016` · `J-AGGREGATE.md:L2943`
- → `D-6` · **depends** · `J-010/IX-008` · `J-AGGREGATE.md:L7999`
- → `D-11` · **gates** · `J-006/IX-004` · `J-AGGREGATE.md:L3755`

**→ family E**
- → `E-31` · **consumes** · `J-008/IX-005` · `J-AGGREGATE.md:L4440`
- → `E-35` · **consumes** · `J-008/IX-005` · `J-AGGREGATE.md:L4440`
- → `E-36` · **consumes** · `J-006/IX-001` · `J-AGGREGATE.md:L3752`
- → `E-54` · **depends** · `J-010/IX-008` · `J-AGGREGATE.md:L7999`
- → `E-24` · **reuses** · `J-006/IX-015` · `J-AGGREGATE.md:L3766`
- → `E-30` · **composes** · `J-004/IX-008` · `J-AGGREGATE.md:L2935`
- → `E-39` · **feeds** · `J-013/IX-019` · `J-AGGREGATE.md:L1368`
- → `E-40` · **reuses** · `J-004/IX-015` · `J-AGGREGATE.md:L2942`
- → `E-43` · **reuses** · `J-006/IX-015` · `J-AGGREGATE.md:L3766`
- → `E-52` · **reuses** · `J-012/IX-004` · `J-AGGREGATE.md:L6064`

**→ family F**
- → `F-2` · **extends** · `J-009/IX-006` · `J-AGGREGATE.md:L4821`
- → `F-1` · **reuses** · `J-006/IX-015` · `J-AGGREGATE.md:L3766`
- → `F-4` · **gates** · `J-005/IX-001` · `J-AGGREGATE.md:L3340`

**→ family G**
- → `G-1` · **consumes** · `J-008/IX-015` · `J-AGGREGATE.md:L4450`
- → `G-2` · **consumes** · `J-008/IX-015` · `J-AGGREGATE.md:L4450`
- → `G-9` · **consumes** · `J-008/IX-015` · `J-AGGREGATE.md:L4450`
- → `G-10` · **depends** · `J-008/IX-006` · `J-AGGREGATE.md:L1386`
- → `G-6` · **depends** · `J-010/IX-008` · `J-AGGREGATE.md:L7999`
- → `G-3` · **gates** · `J-008/IX-002` · `J-AGGREGATE.md:L7951`

**→ family H**
- → `H-42` · **consumes** · `J-011/IX-005` · `J-AGGREGATE.md:L5684`
- → `H-50` · **consumes** · `J-011/IX-005` · `J-AGGREGATE.md:L5684`
- → `H-31` · **depends** · `J-004/IX-016` · `J-AGGREGATE.md:L2943`
- → `H-37` · **consumes** · `J-010/IX-002` · `J-AGGREGATE.md:L1395`
- → `H-39` · **depends** · `J-004/IX-016` · `J-AGGREGATE.md:L2943`
- → `H-4` · **consumes** · `J-008/IX-005` · `J-AGGREGATE.md:L4440`
- → `H-47` · **depends** · `J-004/IX-016` · `J-AGGREGATE.md:L2943`
- → `H-48` · **depends** · `J-004/IX-016` · `J-AGGREGATE.md:L2943`
- → `H-32` · **extends** · `J-012/IX-008` · `J-AGGREGATE.md:L6068`
- → `H-34` · **extends** · `J-008/IX-013` · `J-AGGREGATE.md:L4448`
- → `H-43` · **supersedes** · `J-010/IX-006` · `J-AGGREGATE.md:L5281`

**→ family I**
- → `I-45` · **consumes** · `J-008/IX-015` · `J-AGGREGATE.md:L4450`
- → `I-7` · **consumes** · `J-011/IX-005` · `J-AGGREGATE.md:L5684`
- → `I-11` · **consumes** · `J-006/IX-010` · `J-AGGREGATE.md:L3761`
- → `I-12` · **depends** · `J-004/IX-016` · `J-AGGREGATE.md:L2943`
- → `I-36` · **consumes** · `J-008/IX-005` · `J-AGGREGATE.md:L4440`
- → `I-2` · **extends** · `J-010/IX-011` · `J-AGGREGATE.md:L1397`
- → `I-30` · **extends** · `J-013/IX-014` · `J-AGGREGATE.md:L1366`
- → `I-49` · **extends** · `J-011/IX-002` · `J-AGGREGATE.md:L1400`
- → `I-5` · **extends** · `J-013/IX-014` · `J-AGGREGATE.md:L1366`
- → `I-50` · **extends** · `J-011/IX-002` · `J-AGGREGATE.md:L1400`
- → `I-51` · **extends** · `J-011/IX-002` · `J-AGGREGATE.md:L1400`

**→ family PHASE**
- → `phase-3I` · **gates** · `J-012/IX-017` · `J-AGGREGATE.md:L6077`


### K — Economics, PR evidence, gate tax, billing credits, measurement benches

_Structural pairs from this family: **75** (showing top 40 by evidence score)._

**→ family PHASE/core**
- → `SSB` · **gates** · `K-001/IX-008` · `K-AGGREGATE.md:L1194`

**→ family B**
- → `B-29` · **composes** · `K-004/K5-E020` · `K-AGGREGATE.md:L2005`
- → `B-37` · **reuses** · `K-003/K24-E026` · `K-AGGREGATE.md:L526`

**→ family D**
- → `D-13` · **gates** · `K-001/K16-E008` · `K-AGGREGATE.md:L1131`
- → `D-8` · **gates** · `K-003/K27-E011` · `K-AGGREGATE.md:L1839`

**→ family E**
- → `E-23` · **interacts** · `K-001/IX-015` · `K-AGGREGATE.md:L1201`
- → `E-38` · **consumes** · `K-002/K6-E015` · `K-AGGREGATE.md:L1348`
- → `E-53` · **composes** · `K-004/K5-E020` · `K-AGGREGATE.md:L2005`

**→ family F**
- → `F-2b` · **gates** · `K-003/K13-E010` · `K-AGGREGATE.md:L1658`
- → `F-4` · **consumes** · `K-002/K3-E014` · `K-AGGREGATE.md:L325`

**→ family G**
- → `G-2` · **interacts** · `K-001/IX-009` · `K-AGGREGATE.md:L1195`
- → `G-6` · **interacts** · `K-003/IX-005` · `K-AGGREGATE.md:L1860`
- → `G-9` · **interacts** · `K-002/IX-005` · `K-AGGREGATE.md:L1502`
- → `G-10` · **extends** · `K-004/K25-E015` · `K-AGGREGATE.md:L666`
- → `G-3` · **extends** · `K-003/K21-E012` · `K-AGGREGATE.md:L1740`
- → `G-5` · **consumes** · `K-002/K3-E014` · `K-AGGREGATE.md:L325`
- → `G-8` · **consumes** · `K-002/K3-E014` · `K-AGGREGATE.md:L325`

**→ family H**
- → `H-52` · **interacts** · `K-002/IX-002` · `K-AGGREGATE.md:L1499`
- → `H-36` · **consumes** · `K-003/K14-E012` · `K-AGGREGATE.md:L464`
- → `H-39` · **gates** · `K-001/K16-E008` · `K-AGGREGATE.md:L1131`
- → `H-9` · **composes** · `K-003/K15-E007` · `K-AGGREGATE.md:L474`

**→ family I**
- → `I-11` · **extends** · `K-003/IX-009` · `K-AGGREGATE.md:L391`
- → `I-27` · **composes** · `K-002/IX-008` · `K-AGGREGATE.md:L1505`
- → `I-43` · **interacts** · `K-001/IX-019` · `K-AGGREGATE.md:L1205`
- → `I-18` · **consumes** · `K-002/K6-E015` · `K-AGGREGATE.md:L1348`
- → `I-4` · **gates** · `K-001/K16-E008` · `K-AGGREGATE.md:L1131`
- → `I-44` · **consumes** · `K-002/K6-E015` · `K-AGGREGATE.md:L1348`
- → `I-51` · **reuses** · `K-003/K24-E026` · `K-AGGREGATE.md:L526`

**→ family J**
- → `J-28` · **consumes** · `K-003/IX-010` · `K-AGGREGATE.md:L1865`
- → `J-83` · **extends** · `K-003/IX-009` · `K-AGGREGATE.md:L391`
- → `J-86` · **gates** · `K-001/IX-007` · `K-AGGREGATE.md:L1193`
- → `J-70` · **composes** · `K-002/IX-012` · `K-AGGREGATE.md:L1509`
- → `J-80` · **reuses** · `K-004/IX-010` · `K-AGGREGATE.md:L2201`
- → `J-14` · **interacts** · `K-001/IX-009` · `K-AGGREGATE.md:L1195`
- → `J-34` · **interacts** · `K-002/IX-007` · `K-AGGREGATE.md:L225`
- → `J-85` · **interacts** · `K-003/IX-005` · `K-AGGREGATE.md:L1860`
- → `J-9` · **interacts** · `K-001/IX-012` · `K-AGGREGATE.md:L1198`
- → `J-69` · **extends** · `K-003/K21-E012` · `K-AGGREGATE.md:L1740`
- → `J-16` · **consumes** · `K-002/K3-E014` · `K-AGGREGATE.md:L325`
- → `J-49` · **reuses** · `K-003/K24-E026` · `K-AGGREGATE.md:L526`


### L — Claims, counterfactuals, hallucination policy, skill admission

_Structural pairs from this family: **90** (showing top 45 by evidence score)._

**→ family PHASE/core**
- → `SSB` · **reuses** · `L-001/IX-017` · `L-AGGREGATE.md:L1707`

**→ family B**
- → `B-39` · **interacts** · `L-001/IX-005` · `L-AGGREGATE.md:L1695`
- → `B-22` · **composes** · `L-002/L7-E004` · `L-AGGREGATE.md:L574`
- → `B-29` · **composes** · `L-001/L6-E006` · `L-AGGREGATE.md:L217`
- → `B-37` · **composes** · `L-002/L0-E007` · `L-AGGREGATE.md:L438`
- → `B-41` · **composes** · `L-002/L21-E021` · `L-AGGREGATE.md:L704`

**→ family C**
- → `C-32` · **interacts** · `L-002/IX-007` · `L-AGGREGATE.md:L1720`
- → `C-29` · **gates** · `L-003/L5-E006` · `L-AGGREGATE.md:L796`
- → `C-31` · **gates** · `L-003/L11-E006` · `L-AGGREGATE.md:L883`

**→ family D**
- → `D-13` · **gates** · `L-002/IX-012` · `L-AGGREGATE.md:L1725`
- → `D-11` · **interacts** · `L-001/IX-003` · `L-AGGREGATE.md:L1693`
- → `D-16` · **composes** · `L-002/L0-E018` · `L-AGGREGATE.md:L449`
- → `D-17` · **composes** · `L-002/L7-E004` · `L-AGGREGATE.md:L574`
- → `D-3` · **composes** · `L-002/L0-E018` · `L-AGGREGATE.md:L449`

**→ family E**
- → `E-12` · **composes** · `L-002/IX-005` · `L-AGGREGATE.md:L1718`
- → `E-43` · **interacts** · `L-001/IX-006` · `L-AGGREGATE.md:L1696`
- → `E-53` · **interacts** · `L-001/IX-005` · `L-AGGREGATE.md:L1695`
- → `E-25` · **composes** · `L-003/L16-E013` · `L-AGGREGATE.md:L1003`
- → `E-31` · **composes** · `L-002/L21-E021` · `L-AGGREGATE.md:L704`

**→ family F**
- → `F-12` · **gates** · `L-001/IX-013` · `L-AGGREGATE.md:L1703`
- → `F-5` · **reuses** · `L-002/IX-004` · `L-AGGREGATE.md:L1717`
- → `F-0` · **reuses** · `L-002/L0-E017` · `L-AGGREGATE.md:L448`
- → `F-1` · **gates** · `L-001/L12-E013` · `L-AGGREGATE.md:L288`
- → `F-4` · **composes** · `L-002/L0-E018` · `L-AGGREGATE.md:L449`
- → `F-6` · **gates** · `L-001/L12-E013` · `L-AGGREGATE.md:L288`

**→ family G**
- → `G-1` · **consumes** · `L-002/L1-E016` · `L-AGGREGATE.md:L519`

**→ family H**
- → `H-49` · **reuses** · `L-001/IX-008` · `L-AGGREGATE.md:L1698`

**→ family I**
- → `I-2` · **depends** · `L-003/IX-019` · `L-AGGREGATE.md:L1752`
- → `I-10` · **composes** · `L-002/IX-009` · `L-AGGREGATE.md:L1722`
- → `I-12` · **composes** · `L-001/IX-007` · `L-AGGREGATE.md:L1697`
- → `I-27` · **reuses** · `L-003/IX-014` · `L-AGGREGATE.md:L1747`
- → `I-4` · **composes** · `L-001/IX-011` · `L-AGGREGATE.md:L1701`
- → `I-40` · **composes** · `L-002/IX-005` · `L-AGGREGATE.md:L1718`
- → `I-8` · **gates** · `L-002/IX-008` · `L-AGGREGATE.md:L1721`
- → `I-11` · **interacts** · `L-003/IX-011` · `L-AGGREGATE.md:L1744`

**→ family J**
- → `J-55` · **gates** · `L-003/IX-009` · `L-AGGREGATE.md:L1742`
- → `J-103` · **composes** · `L-003/IX-006` · `L-AGGREGATE.md:L1739`
- → `J-36` · **composes** · `L-002/IX-009` · `L-AGGREGATE.md:L1722`
- → `J-83` · **composes** · `L-003/IX-006` · `L-AGGREGATE.md:L1739`
- → `J-96` · **composes** · `L-001/IX-007` · `L-AGGREGATE.md:L1697`
- → `J-102` · **interacts** · `L-001/IX-006` · `L-AGGREGATE.md:L1696`
- → `J-2` · **interacts** · `L-002/IX-017` · `L-AGGREGATE.md:L1730`

**→ family K**
- → `K-12` · **reuses** · `L-001/IX-008` · `L-AGGREGATE.md:L1698`
- → `K-11` · **interacts** · `L-001/IX-018` · `L-AGGREGATE.md:L1708`

**→ family N**
- → `N-13` · **composes** · `L-002/IX-009` · `L-AGGREGATE.md:L1722`


### M — Org commerce: entitlement tiers, policy packs, org report, verifier extract

_Structural pairs from this family: **21** (showing top 21 by evidence score)._

**→ family PHASE/core**
- → `SSB` · **interacts** · `M-001/IX-016` · `M-AGGREGATE.md:L665`

**→ family B**
- → `B-43` · **reuses** · `M-001/IX-009` · `M-AGGREGATE.md:L658`
- → `B-45` · **depends** · `M-001/M4-E002` · `M-AGGREGATE.md:L493`

**→ family E**
- → `E-39` · **composes** · `M-001/M2-E026` · `M-AGGREGATE.md:L420`
- → `E-45` · **composes** · `M-001/M6-E034` · `M-AGGREGATE.md:L149`

**→ family G**
- → `G-8` · **extends** · `M-001/IX-008` · `M-AGGREGATE.md:L657`
- → `G-9` · **extends** · `M-001/IX-008` · `M-AGGREGATE.md:L657`
- → `G-2` · **composes** · `M-001/M5-E029` · `M-AGGREGATE.md:L131`

**→ family H**
- → `H-9` · **extends** · `M-001/M3-E028` · `M-AGGREGATE.md:L469`

**→ family J**
- → `J-59` · **gates** · `M-001/IX-013` · `M-AGGREGATE.md:L662`
- → `J-49` · **feeds** · `M-001/M0-E033` · `M-AGGREGATE.md:L299`

**→ family K**
- → `K-6` · **depends** · `M-001/IX-011` · `M-AGGREGATE.md:L660`
- → `K-24` · **gates** · `M-001/IX-004` · `M-AGGREGATE.md:L653`
- → `K-3` · **gates** · `M-001/IX-001` · `M-AGGREGATE.md:L650`
- → `K-1` · **interacts** · `M-001/IX-012` · `M-AGGREGATE.md:L661`
- → `K-11` · **gates** · `M-001/M0-E036` · `M-AGGREGATE.md:L302`
- → `K-4` · **extends** · `M-001/M3-E028` · `M-AGGREGATE.md:L469`
- → `K-5` · **composes** · `M-001/M2-E026` · `M-AGGREGATE.md:L420`
- → `K-7` · **composes** · `M-001/M5-E029` · `M-AGGREGATE.md:L131`
- → `K-13` · **interacts** · `M-001/M0-E037` · `M-AGGREGATE.md:L303`
- → `K-23` · **interacts** · `M-001/M0-E038` · `M-AGGREGATE.md:L304`


### N — Measurement spine, provenance lattice (N-6), dashboard inbox (N-7)

_Structural pairs from this family: **131** (showing top 45 by evidence score)._

**→ family PHASE/core**
- → `SSB` · **interacts** · `N-001/IX-012` · `N-AGGREGATE.md:L763`

**→ family B**
- → `B-17` · **composes** · `N-002/N8-E025` · `N-AGGREGATE.md:L571`
- → `B-21` · **feeds** · `N-002/N10-E008` · `N-AGGREGATE.md:L586`
- → `B-22b` · **consumes** · `N-001/N12-E022` · `N-AGGREGATE.md:L401`
- → `B-24` · **consumes** · `SUMMARY/DEP-013` · `N-AGGREGATE.md:L1602`
- → `B-28` · **composes** · `N-002/N5-E026` · `N-AGGREGATE.md:L483`
- → `B-41` · **consumes** · `SUMMARY/DEP-015` · `N-AGGREGATE.md:L1604`
- → `B-42` · **composes** · `N-001/N1-E030` · `N-AGGREGATE.md:L242`

**→ family C**
- → `C-49` · **closes/extends** · `N-001/IX-019` · `N-AGGREGATE.md:L770`
- → `C-30` · **composes** · `N-001/IX-014` · `N-AGGREGATE.md:L765`
- → `C-10` · **consumes** · `N-001/N12-E022` · `N-AGGREGATE.md:L401`
- → `C-26` · **consumes** · `SUMMARY/DEP-013` · `N-AGGREGATE.md:L1602`
- → `C-45` · **consumes** · `SUMMARY/DEP-009` · `N-AGGREGATE.md:L1598`

**→ family D**
- → `D-17` · **consumes** · `SUMMARY/DEP-016` · `N-AGGREGATE.md:L1605`
- → `D-10` · **consumes** · `SUMMARY/DEP-005` · `N-AGGREGATE.md:L1594`
- → `D-15` · **feeds** · `N-002/N10-E008` · `N-AGGREGATE.md:L586`
- → `D-16` · **extends** · `N-001/N12-E008` · `N-AGGREGATE.md:L387`

**→ family F**
- → `F-5` · **composes** · `N-001/IX-014` · `N-AGGREGATE.md:L765`

**→ family G**
- → `G-5` · **reuses** · `N-001/IX-010` · `N-AGGREGATE.md:L761`

**→ family H**
- → `H-49` · **extends** · `N-001/IX-005` · `N-AGGREGATE.md:L756`
- → `H-19` · **interacts** · `N-001/IX-015` · `N-AGGREGATE.md:L766`
- → `H-27` · **consumes** · `SUMMARY/DEP-016` · `N-AGGREGATE.md:L1605`

**→ family I**
- → `I-12` · **feeds** · `N-001/IX-017` · `N-AGGREGATE.md:L768`
- → `I-8` · **composes** · `N-001/IX-014` · `N-AGGREGATE.md:L765`
- → `I-31` · **interacts** · `N-001/IX-015` · `N-AGGREGATE.md:L766`
- → `I-24` · **consumes** · `SUMMARY/DEP-016` · `N-AGGREGATE.md:L1605`
- → `I-39` · **consumes** · `SUMMARY/DEP-002` · `N-AGGREGATE.md:L1591`

**→ family J**
- → `J-109` · **closes/extends** · `N-001/IX-019` · `N-AGGREGATE.md:L770`
- → `J-97` · **closes/extends** · `N-001/IX-019` · `N-AGGREGATE.md:L770`
- → `J-21` · **composes** · `N-001/IX-014` · `N-AGGREGATE.md:L765`
- → `J-83` · **gates** · `N-001/IX-009` · `N-AGGREGATE.md:L760`
- → `J-96` · **composes** · `N-001/IX-014` · `N-AGGREGATE.md:L765`
- → `J-68` · **consumes** · `SUMMARY/DEP-011` · `N-AGGREGATE.md:L1600`
- → `J-89` · **consumes** · `SUMMARY/DEP-002` · `N-AGGREGATE.md:L1591`
- → `J-93` · **consumes** · `SUMMARY/DEP-002` · `N-AGGREGATE.md:L1591`

**→ family K**
- → `K-22` · **consumes** · `SUMMARY/DEP-011` · `N-AGGREGATE.md:L1600`
- → `K-23` · **consumes** · `SUMMARY/DEP-011` · `N-AGGREGATE.md:L1600`
- → `K-24` · **consumes** · `SUMMARY/DEP-011` · `N-AGGREGATE.md:L1600`

**→ family L**
- → `L-16` · **extends** · `N-001/IX-005` · `N-AGGREGATE.md:L756`
- → `L-2` · **interacts** · `N-001/IX-018` · `N-AGGREGATE.md:L769`
- → `L-7` · **interacts** · `N-002/IX-002` · `N-AGGREGATE.md:L779`

**→ family M**
- → `M-2` · **interacts** · `N-002/IX-016` · `N-AGGREGATE.md:L793`
- → `M-3` · **interacts** · `N-002/IX-012` · `N-AGGREGATE.md:L789`
- → `M-1` · **consumes** · `SUMMARY/DEP-002` · `N-AGGREGATE.md:L1591`

**→ family PHASE**
- → `phase-5` · **consumes** · `SUMMARY/DEP-002` · `N-AGGREGATE.md:L1591`


### O — Daemon/workflow: O-1 Cosmos/daemon, O-2 TaskSpec, O-3 claims, O-4 canvas

_Structural pairs from this family: **11** (showing top 11 by evidence score)._

**→ family B**
- → `B-40` · **composes** · `O-001/O1-E109` · `O-AGGREGATE.md:L258`

**→ family C**
- → `C-9` · **composes** · `O-001/O2-E110` · `O-AGGREGATE.md:L373`

**→ family D**
- → `D-5` · **consumes** · `O-001/O3-E082` · `O-AGGREGATE.md:L460`

**→ family E**
- → `E-5` · **consumes** · `O-001/O3-E081` · `O-AGGREGATE.md:L459`

**→ family F**
- → `F-3` · **supersedes** · `SUP/SD-MC009` · `O-AGGREGATE.md:L1114`

**→ family G**
- → `G-1` · **supersedes** · `SUP/SD-MC009` · `O-AGGREGATE.md:L1114`
- → `G-2` · **supersedes** · `SUP/SD-MC009` · `O-AGGREGATE.md:L1114`

**→ family H**
- → `H-12` · **consumes** · `O-001/O4-E072` · `O-AGGREGATE.md:L562`

**→ family J**
- → `J-31` · **consumes** · `O-001/O4-E072` · `O-AGGREGATE.md:L562`
- → `J-60` · **consumes** · `O-001/O4-E040` · `O-AGGREGATE.md:L530`

**→ family K**
- → `K-14` · **composes** · `O-001/O2-E110` · `O-AGGREGATE.md:L373`


### P — Fleet apps, ports bus, privacy lattice OUT-flow, connectors, co-browse

_Structural pairs from this family: **63** (showing top 45 by evidence score)._

**→ family PHASE/core**
- → `SSB` · **interacts** · `P-001/IX-013` · `P-AGGREGATE.md:L1684`

**→ family B**
- → `B-37` · **consumes** · `P-003/IX-005` · `P-AGGREGATE.md:L1724`
- → `B-35` · **consumes** · `P-001/P4-E008` · `P-AGGREGATE.md:L122`

**→ family D**
- → `D-5` · **consumes** · `P-001/P4-E008` · `P-AGGREGATE.md:L122`

**→ family E**
- → `E-14` · **consumes** · `P-003/IX-011` · `P-AGGREGATE.md:L1730`
- → `E-20` · **consumes** · `P-001/IX-003` · `P-AGGREGATE.md:L1674`
- → `E-4` · **extends** · `P-002/IX-006` · `P-AGGREGATE.md:L1700`
- → `E-5` · **composes** · `P-003/IX-001` · `P-AGGREGATE.md:L1720`
- → `E-48` · **interacts** · `P-003/IX-015` · `P-AGGREGATE.md:L1734`

**→ family F**
- → `F-4` · **extends** · `P-003/P15-E006` · `P-AGGREGATE.md:L688`
- → `F-5` · **consumes** · `P-001/P4-E008` · `P-AGGREGATE.md:L122`
- → `F-9` · **gates** · `P-002/P1-E007` · `P-AGGREGATE.md:L352`

**→ family G**
- → `G-2` · **consumes** · `P-003/IX-011` · `P-AGGREGATE.md:L1730`
- → `G-1` · **mirrors** · `P-002/IX-004` · `P-AGGREGATE.md:L1698`
- → `G-3` · **composes** · `P-002/IX-008` · `P-AGGREGATE.md:L1702`
- → `G-9` · **interacts** · `P-003/IX-002` · `P-AGGREGATE.md:L1721`
- → `G-10` · **extends** · `P-002/P1-E029` · `P-AGGREGATE.md:L374`

**→ family H**
- → `H-13` · **consumes** · `P-003/IX-011` · `P-AGGREGATE.md:L1730`
- → `H-15` · **consumes** · `P-003/IX-011` · `P-AGGREGATE.md:L1730`
- → `H-14` · **interacts** · `P-002/IX-016` · `P-AGGREGATE.md:L1710`
- → `H-51` · **interacts** · `P-003/IX-007` · `P-AGGREGATE.md:L1726`
- → `H-18` · **extends** · `P-002/P7-E019` · `P-AGGREGATE.md:L462`
- → `H-49` · **extends** · `P-002/P6-E020` · `P-AGGREGATE.md:L435`

**→ family I**
- → `I-50` · **consumes** · `P-003/IX-005` · `P-AGGREGATE.md:L1724`
- → `I-1` · **interacts** · `P-002/IX-009` · `P-AGGREGATE.md:L1703`

**→ family J**
- → `J-28` · **consumes** · `P-003/IX-005` · `P-AGGREGATE.md:L1724`
- → `J-96` · **extends** · `P-003/IX-003` · `P-AGGREGATE.md:L1722`
- → `J-1` · **extends** · `P-001/P13-E004` · `P-AGGREGATE.md:L232`
- → `J-77` · **extends** · `P-001/P13-E004` · `P-AGGREGATE.md:L232`
- → `J-8` · **extends** · `P-001/P13-E004` · `P-AGGREGATE.md:L232`

**→ family K**
- → `K-18` · **reuses** · `P-003/IX-004` · `P-AGGREGATE.md:L1723`
- → `K-19` · **extends** · `P-003/P15-E006` · `P-AGGREGATE.md:L688`

**→ family L**
- → `L-12` · **interacts** · `P-003/IX-018` · `P-AGGREGATE.md:L1737`
- → `L-14` · **interacts** · `P-002/IX-001` · `P-AGGREGATE.md:L1695`

**→ family M**
- → `M-2` · **extends** · `P-002/IX-006` · `P-AGGREGATE.md:L1700`
- → `M-6` · **interacts** · `P-003/IX-002` · `P-AGGREGATE.md:L1721`

**→ family N**
- → `N-15` · **extends** · `P-002/IX-006` · `P-AGGREGATE.md:L1700`
- → `N-2` · **mirrors** · `P-002/IX-004` · `P-AGGREGATE.md:L1698`
- → `N-6` · **mirrors** · `P-002/IX-004` · `P-AGGREGATE.md:L1698`

**→ family O**
- → `O-4` · **extends** · `P-002/IX-002` · `P-AGGREGATE.md:L1696`
- → `O-2` · **gates** · `P-001/IX-008` · `P-AGGREGATE.md:L1679`
- → `O-3` · **interacts** · `P-002/IX-003` · `P-AGGREGATE.md:L1697`

**→ family Q**
- → `Q-4` · **consumes** · `P-003/IX-005` · `P-AGGREGATE.md:L1724`
- → `Q-10` · **supersedes** · `P-003/IX-014` · `P-AGGREGATE.md:L1733`
- → `Q-6` · **extends** · `P-001/IX-009` · `P-AGGREGATE.md:L1680`


### Q — Orch economics, taste benches, distribution hygiene, creativity instruments

_Structural pairs from this family: **30** (showing top 30 by evidence score)._

**→ family PHASE/core**
- → `SSB` · **interacts** · `Q-002/IX-015` · `Q-AGGREGATE.md:L828`

**→ family B**
- → `B-40` · **interacts** · `Q-001/QG-E005` · `Q-AGGREGATE.md:L255`

**→ family D**
- → `D-13` · **consumes** · `Q-002/Q2-E011` · `Q-AGGREGATE.md:L557`
- → `D-15` · **consumes** · `Q-002/Q10-E015` · `Q-AGGREGATE.md:L142`

**→ family E**
- → `E-2` · **consumes** · `Q-001/IX-011` · `Q-AGGREGATE.md:L799`
- → `E-1` · **mirrors** · `Q-001/IX-010` · `Q-AGGREGATE.md:L798`
- → `E-48` · **consumes** · `Q-002/Q10-E015` · `Q-AGGREGATE.md:L142`
- → `E-54` · **consumes** · `Q-001/Q7-E007` · `Q-AGGREGATE.md:L369`

**→ family F**
- → `F-0` · **composes** · `Q-001/Q0-E005` · `Q-AGGREGATE.md:L307`
- → `F-5` · **consumes** · `Q-002/Q1-E012` · `Q-AGGREGATE.md:L126`
- → `F-6` · **composes** · `Q-002/Q4-E023` · `Q-AGGREGATE.md:L626`

**→ family G**
- → `G-0` · **composes** · `Q-001/Q0-E005` · `Q-AGGREGATE.md:L307`
- → `G-2` · **consumes** · `Q-001/Q5-E006` · `Q-AGGREGATE.md:L343`

**→ family H**
- → `H-52` · **reuses** · `Q-001/IX-008` · `Q-AGGREGATE.md:L796`
- → `H-7` · **consumes** · `Q-002/Q3-E010` · `Q-AGGREGATE.md:L585`
- → `H-9` · **consumes** · `Q-002/Q3-E010` · `Q-AGGREGATE.md:L585`

**→ family I**
- → `I-11` · **composes** · `Q-002/IX-004` · `Q-AGGREGATE.md:L817`
- → `I-25` · **composes** · `Q-002/IX-004` · `Q-AGGREGATE.md:L817`
- → `I-27` · **reuses** · `Q-002/IX-012` · `Q-AGGREGATE.md:L181`

**→ family J**
- → `J-28` · **interacts** · `Q-002/IX-009` · `Q-AGGREGATE.md:L822`

**→ family K**
- → `K-2` · **extends** · `Q-002/IX-014` · `Q-AGGREGATE.md:L182`
- → `K-4` · **interacts** · `Q-001/IX-006` · `Q-AGGREGATE.md:L794`

**→ family L**
- → `L-24` · **consumes** · `Q-002/Q10-E015` · `Q-AGGREGATE.md:L142`
- → `L-10` · **interacts** · `Q-002/Q11-E010` · `Q-AGGREGATE.md:L709`

**→ family N**
- → `N-16` · **consumes** · `Q-002/Q10-E015` · `Q-AGGREGATE.md:L142`
- → `N-2` · **consumes** · `SUMMARY/Q9-05` · `Q-AGGREGATE.md:L204`

**→ family O**
- → `O-0` · **composes** · `Q-001/Q0-E005` · `Q-AGGREGATE.md:L307`
- → `O-1` · **consumes** · `Q-002/Q10-E015` · `Q-AGGREGATE.md:L142`
- → `O-2` · **consumes** · `Q-002/Q1-E012` · `Q-AGGREGATE.md:L126`

**→ family PHASE**
- → `phase-4D` · **consumes** · `Q-002/Q10-E015` · `Q-AGGREGATE.md:L142`


---

## Reverse index — most-depended-on features

Providers ranked by distinct consumer **families**, then raw edge count. Includes hubs and high-fan-in features.

| Rank | Provider | # families | # raw edges | Consumer families |
|---:|---|---:|---:|---|
| 1 | `SSB` | 10 | 43 | G, H, I, J, K, L, M, N, P, Q |
| 2 | `B-37` | 9 | 67 | C, D, E, I, J, K, L, N, P |
| 3 | `G-2` | 9 | 63 | H, I, J, K, L, M, O, P, Q |
| 4 | `D-13` | 7 | 45 | E, H, I, J, K, L, Q |
| 5 | `G-1` | 7 | 43 | H, J, K, L, N, O, P |
| 6 | `F-4` | 7 | 41 | G, H, I, J, K, L, P |
| 7 | `F-5` | 6 | 29 | I, J, L, N, P, Q |
| 8 | `F-1` | 6 | 26 | G, H, I, J, L, N |
| 9 | `B-40` | 6 | 19 | C, E, I, J, O, Q |
| 10 | `H-49` | 5 | 40 | I, J, L, N, P |
| 11 | `B-29` | 5 | 39 | C, E, J, K, L |
| 12 | `G-9` | 5 | 36 | I, J, K, M, P |
| 13 | `D-16` | 5 | 30 | H, I, J, L, N |
| 14 | `H-9` | 5 | 29 | I, J, K, M, Q |
| 15 | `E-54` | 5 | 21 | H, J, L, N, Q |
| 16 | `I-27` | 5 | 21 | J, K, L, N, Q |
| 17 | `G-6` | 5 | 20 | I, J, K, L, N |
| 18 | `D-11` | 5 | 15 | E, F, I, J, L |
| 19 | `D-6` | 5 | 14 | E, H, J, K, N |
| 20 | `B-22` | 5 | 13 | C, E, H, J, L |
| 21 | `H-7` | 5 | 10 | I, J, K, N, Q |
| 22 | `G-10` | 4 | 36 | J, K, N, P |
| 23 | `D-3` | 4 | 24 | E, H, I, L |
| 24 | `H-39` | 4 | 24 | I, J, K, L |
| 25 | `G-5` | 4 | 22 | H, J, K, N |
| 26 | `H-31` | 4 | 22 | I, J, K, N |
| 27 | `B-41` | 4 | 19 | C, E, L, N |
| 28 | `D-17` | 4 | 18 | H, I, L, N |
| 29 | `E-39` | 4 | 16 | I, J, M, N |
| 30 | `H-50` | 4 | 16 | I, J, L, N |
| 31 | `I-11` | 4 | 16 | J, K, L, Q |
| 32 | `D-1` | 4 | 15 | E, G, I, J |
| 33 | `H-32` | 4 | 15 | I, J, K, L |
| 34 | `I-12` | 4 | 15 | J, L, N, PHASE |
| 35 | `I-4` | 4 | 15 | J, K, L, N |
| 36 | `I-51` | 4 | 15 | J, K, L, N |
| 37 | `K-11` | 4 | 15 | J, L, M, N |
| 38 | `C-29` | 4 | 14 | E, I, J, L |
| 39 | `E-25` | 4 | 14 | H, I, J, L |
| 40 | `I-25` | 4 | 14 | J, K, N, Q |
| 41 | `D-10` | 4 | 13 | E, J, L, N |
| 42 | `E-43` | 4 | 13 | F, G, J, L |
| 43 | `E-48` | 4 | 13 | H, J, P, Q |
| 44 | `B-39` | 4 | 12 | C, E, J, L |
| 45 | `E-35` | 4 | 11 | I, J, L, N |
| 46 | `E-5` | 4 | 11 | J, K, O, P |
| 47 | `F-6` | 4 | 11 | J, L, N, Q |
| 48 | `G-3` | 4 | 11 | I, J, K, P |
| 49 | `E-31` | 4 | 10 | I, J, K, L |
| 50 | `D-2` | 4 | 8 | E, H, I, J |
| 51 | `K-4` | 4 | 8 | M, N, P, Q |
| 52 | `D-5` | 4 | 7 | H, J, O, P |
| 53 | `H-52` | 4 | 7 | I, J, K, Q |
| 54 | `B-38` | 4 | 6 | C, E, I, J |
| 55 | `F-0` | 4 | 6 | J, L, P, Q |
| 56 | `G-0` | 4 | 4 | J, L, P, Q |
| 57 | `G-8` | 3 | 28 | J, K, M |
| 58 | `J-83` | 3 | 22 | K, L, N |
| 59 | `H-43` | 3 | 17 | I, J, K |
| 60 | `J-28` | 3 | 17 | K, P, Q |
| 61 | `F-12` | 3 | 16 | H, J, L |
| 62 | `H-6` | 3 | 16 | I, J, L |
| 63 | `B-24` | 3 | 14 | E, J, N |
| 64 | `H-13` | 3 | 14 | J, N, P |
| 65 | `I-1` | 3 | 14 | J, N, P |
| 66 | `B-31` | 3 | 13 | C, E, J |
| 67 | `I-2` | 3 | 13 | J, L, P |
| 68 | `J-96` | 3 | 13 | L, N, P |
| 69 | `K-12` | 3 | 13 | J, L, N |
| 70 | `I-8` | 3 | 12 | J, L, N |
| 71 | `N-7` | 3 | 12 | J, L, P |
| 72 | `B-42` | 3 | 11 | C, L, N |
| 73 | `C-32` | 3 | 11 | E, I, L |
| 74 | `E-30` | 3 | 11 | G, I, J |
| 75 | `E-49` | 3 | 11 | I, J, L |
| 76 | `J-55` | 3 | 11 | K, L, N |
| 77 | `K-18` | 3 | 11 | J, N, P |
| 78 | `C-30` | 3 | 10 | D, N, PHASE |
| 79 | `J-21` | 3 | 10 | K, L, N |
| 80 | `C-26` | 3 | 9 | H, J, N |

_Total distinct external provider IDs cited: **426**._

### Named load-bearing gates (explicit product vocabulary)

| Feature | Canonical role | Primary consumers (from map) |
|---|---|---|
| **F-4** | Cant-lie / PROVEN case / temporal+stdout hash gate | G, H, I, J, K, L, P |
| **C-28** | Mutation validation gate | F (F-4 depends), I (I-12 chain), J |
| **C-29** | Intent / planning-mutation companion gate | E, I, J, L |
| **G-1** | Taint Untrusted on IN-flow bytes | H, J, K, L, N, O, P |
| **G-2** | RRA seal + replay `--verify` | H, I, J, K, L, M, O, P, Q |
| **SSB** | Append-only session journal / event bus | G–N, P, Q (+ PHASE backbone) |
| **J-16** | Blast-radius analysis | K, L (hub-adjacent to B-29/C-24/G-3) |
| **N-6** | ONE provenance/trust lattice + admission | P (ports/privacy), N internal, J/L admission family |
| **O-2** | TaskSpec + triggers (Cron/SsbEvent/Manual) | P, Q |
| **I-25** | Router + intervention-card inbox | J, K, N, Q |
| **B-37** | RuntimeAdapter | C, D, E, I, J, K, L, N, P |
| **D-13** | Cross-family independence (adversary principle) | E, H, I, J, K, L, Q |
| **K-24** | Billing/credits rail (gated by M-1 tier) | M, N |
| **G-8** | Attestation sign/verify | J, K, M |
| **F-5** | Quarantine-on-flaky | I, J, L, N, P, Q |

---

## Composition chains (multi-hop, evidence-backed)

Selected end-to-end compositions that span 3+ families.

### 1. Trust spine on every external byte

```
external ingest (P-9/P-10/P-12/P-22)
  → G-1 taint Untrusted
  → N-6 lattice tier (N-2 forbids second scale)
  → G-2 RRA / SSB journal
  → G-9 redaction on export (M-3/M-4/P-3)
  → F-4 / F-5 case+quarantine when claims enter reuse
```

- Evidence: `P-002/IX-004` (P↔G-1↔N-6) `P-AGGREGATE.md` IX; `P-002/IX-008` (P-9←G-1,G-3,L-12,J-102,N-6); `M-001/IX-013` (J-59/G-9 privacy chain); `Q-002/Q1-E012` (Q-1←F-5).

### 2. Mutation / claim honesty stack

```
C-28 mutation gate + C-29 intent
  → F-4 cant-lie / PROVEN
  → I-12 curator (must not reimplement 3F SM)  [PHASE-005/IX-006]
  → L-5 counterfactual do-test (prereqs C-29, J-16, F-4)
  → J-96 CaseNode / MintOnProof → P-15 reuse → K-18 hit-rate
```

- Evidence: `SUMMARY/F4-19` F-4 depends C-28; `I-004/IX-008` I-12←C-28; `L-003/L5-E006`; `P-003/IX-003` P-15←J-96/K-18.

### 3. Org commerce over seat trust

```
G-8 attestation + G-2 verify + K-6 export shape
  → M-1 tier()  [gates K-24 credits, M-2 packs, M-3 report]
  → M-3 org rollup (extends K-6; J-59 isolation gate)
  → M-5/M-6 open verifier + golden-lock (Q-8 ceiling consumes M)
```

- Evidence: `M-001/IX-004`, `M-001/IX-008`, `M-001/IX-011`; `Q-001/IX-006` Q-8↔M/K-4.

### 4. Orchestration economics (Q over O/I/J/H/K)

```
O-1 daemon / O-2 TaskSpec wake sites
  → Q-2 cross-family wake quorum (D-13 principle, J free-API voters)
  → I-25 router / intervention cards
  → Q-3 advisor (H-7 split, I-11 BeliefState, H-9 spend, K-2 arm)
  → Q-4 model swap (F-6 trust, 4G/4M, J rotation)
  → J-28 usage rows + SSB measured-number idiom (GD-2)
```

- Evidence: `Q-002/IX-004`, `SUMMARY/Q2-20`, `SUMMARY/Q4-17`, `Q-002/IX-009`, `Q-002/IX-014`.

### 5. Dashboard closes B/phase; later families extend C surfaces

```
B-* dashboard/runtime gaps
  → C-* closes (many SUMMARY S-E* "Closes B-N / phase-*")
  → D-26/D-27 dashboards reuse C-3/C-6/C-7 + SSB
  → H/I/J drawers and harness panels ride C IPC + B-37 RuntimeAdapter
```

- Evidence: C-AGGREGATE SUMMARY closes rows; `D26-003` depends C-3/C-4/C-6/C-7; B-37 hub fan-in.

### 6. PHASE 3B SSB as universal journal

```
phase-3B typed action_type + single-writer SSB
  → almost every later family journals measured rows
  → schema bumps per family (D 47→…, J baselines, M entitlement events, Q Goal*/Wake*)
  → conflict: phase-4D research_* types vs 3B 17-type CHECK (PHASE-007/IX-017)
```

- Evidence: `PHASE-007/IX-009`, `PHASE-007/IX-017`; per-family IX SSB budget atoms (G/H/I/J/K/L/M/N/P/Q).

---

## Supersession notes (cross-family, NEWER-PLAN PRECEDENCE)

When two families describe the same interaction differently, the winner is recorded first.

| Topic | Winner | Loser (retained) | Rule | Anchors |
|---|---|---|---|---|
| blake2s vs blake3 project hash | PHASE-008 blake3 for 3C/4F | B-16A blake2s | Explicit supersedes | `PHASE-008/IX-030` @ PHASE-AGGREGATE; B-16A |
| Plain Thompson / BAI promotion | **J-20** supersedes plain-TS of D-5/D-29/H-48 | D-5, D-29, H-48 plain-TS slices | Explicit SUPERSEDING new file; never amend losers | `J-010/IX-006`, J20-E002 @ J-AGGREGATE |
| Q-10 research-conditioned creativity mechanism | **VALID REFUTED** (Q deep+summary) | Pre-verdict Tier-2/3 adoption text | Measurement banner | Q SUP-003/006; `P-003/IX-014` P-25 supersedes refuted Q-10 runner without reuse |
| Q-14 E-1 novelty canary | SHIPPED 2026-07-18 (deep) | Summary "planned" inventory | Deep SHIPPED beats summary | Q SUP-001 |
| Q-15 S-1 policy arm | REJECTED-by-measurement | Summary live arm inventory | Measurement rejection | Q SUP-002 |
| K-2 live-arm gate | **W-10 infra** (K-26 MOVE 6) | Summary "block live dollar on J-28" | Deep correction supersedes summary | `K-003/K2-C001` @ K-AGGREGATE |
| L-5 F-4/J-16 line anchors | Live L-5 plan | Older summary line numbers | Live plan supersedes summary | `L-003/L5-C001` @ L-AGGREGATE |
| P Tier-1 vs steal/build chrome | Deep REVISED Tier-1 (CLI/registry) | Opening text with O-4 canvas / M-5 mount as core | Deep conflict target state | P SUP-015/023 etc. @ P-AGGREGATE |
| O trust-spine wording | O-001 deep boundary | SUMMARY blanket byte claim | Deep > summary | `SUP/SD-MC009` @ O-AGGREGATE |
| 3F lifecycle ownership | PHASE 3F owns SM | I-12 must not reimplement | Cross-family note in PHASE | `PHASE-005/IX-006` |
| Letter-order default | Later letter (e.g. Q > P > … > B) | Older letter's conflicting target state | NEWER-PLAN when no SUPERSEDED marker | Document contract |

---

## Family-to-family adjacency (coarse)

Edge weight = unique provider features of column family consumed by row family.

Consumer ↓ \ Provider → | PHASE | B | C | D | E | F | G | H | I | J | K | L | M | N | O | P | Q |
|---| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| **PHASE** | · | 1 | 1 | · | · | · | · | · | 1 | · | · | · | · | · | · | · | · |
| **B** | 3 | · | 3 | · | 2 | · | · | · | · | · | · | · | · | · | · | · | · |
| **C** | 17 | 41 | · | · | · | · | · | · | · | · | · | · | · | · | · | · | · |
| **D** | 1 | 1 | 9 | · | · | · | · | 1 | · | · | · | · | · | · | · | · | · |
| **E** | 2 | 23 | 23 | 15 | · | · | · | · | · | · | · | · | · | · | · | · | · |
| **F** | 1 | · | 2 | 1 | 3 | · | · | · | · | · | · | · | · | · | · | · | · |
| **G** | 1 | · | 1 | 2 | 2 | 7 | · | · | · | · | · | · | · | · | · | · | · |
| **H** | 2 | 2 | 6 | 9 | 11 | 5 | 3 | · | · | 1 | · | · | · | · | · | · | · |
| **I** | 7 | 3 | 9 | 7 | 19 | 5 | 5 | 22 | · | 2 | · | · | · | · | · | · | · |
| **J** | 8 | 14 | 15 | 11 | 27 | 10 | 14 | 31 | 33 | · | 11 | 3 | · | 1 | · | · | · |
| **K** | 1 | 2 | · | 3 | 7 | 3 | 8 | 9 | 11 | 20 | · | 10 | · | 1 | · | · | · |
| **L** | 1 | 6 | 3 | 8 | 8 | 6 | 5 | 7 | 14 | 24 | 4 | · | · | 4 | · | · | · |
| **M** | 1 | 2 | · | · | 2 | · | 3 | 1 | · | 2 | 10 | · | · | · | · | · | · |
| **N** | 2 | 8 | 6 | 7 | 8 | 3 | 4 | 12 | 18 | 34 | 13 | 12 | 4 | · | · | · | · |
| **O** | · | 1 | 1 | 1 | 1 | 1 | 2 | 1 | · | 2 | 1 | · | · | · | · | · | · |
| **P** | 2 | 2 | 1 | 1 | 5 | 4 | 6 | 6 | 3 | 5 | 6 | 3 | 6 | 5 | 5 | · | 3 |
| **Q** | 2 | 1 | · | 2 | 4 | 3 | 2 | 3 | 3 | 1 | 2 | 2 | · | 2 | 3 | · | · |

_Cells count unique provider **features** (not raw mentions). Dot = zero in structural pair set._

---

## Closing audit block

| Check | Result |
|---|---|
| Families covered | **17/17** (PHASE, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q) |
| Structural edges (consumer family → provider feature) | **881** |
| Relation-qualified edges | **1670** |
| Raw mention edges (multi-ledger) | **3010** |
| Hub systems (≥3 consumer families) | **115** |
| Distinct provider feature IDs | **426** |
| Structural edges listed in consumer sections (capped) | **504** |
| Inputs mutated | **none** (read-only) |
| Output path | this file only |
| Precedence policy applied | SUPERSEDED marker > later letter family; PHASE foundation; losers retained |
| Confidence | **High confidence** for hub fan-in and IX/DEP edges; raw mentions may include summary duplicates |

### Hub systems list (complete ids with ≥3 families)

`SSB`, `B-37`, `G-2`, `D-13`, `G-1`, `F-4`, `F-5`, `F-1`, `B-40`, `H-49`, `B-29`, `G-9`, `D-16`, `H-9`, `E-54`, `I-27`, `G-6`, `D-11`, `D-6`, `B-22`, `H-7`, `G-10`, `D-3`, `H-39`, `G-5`, `H-31`, `B-41`, `D-17`, `E-39`, `H-50`, `I-11`, `D-1`, `H-32`, `I-12`, `I-4`, `I-51`, `K-11`, `C-29`, `E-25`, `I-25`, `D-10`, `E-43`, `E-48`, `B-39`, `E-35`, `E-5`, `F-6`, `G-3`, `E-31`, `D-2`, `K-4`, `D-5`, `H-52`, `B-38`, `F-0`, `G-0`, `G-8`, `J-83`, `H-43`, `J-28`, `F-12`, `H-6`, `B-24`, `H-13`, `I-1`, `B-31`, `I-2`, `J-96`, `K-12`, `I-8`, `N-7`, `B-42`, `C-32`, `E-30`, `E-49`, `J-55`, `K-18`, `C-30`, `J-21`, `C-26`, `E-20`, `E-37`, `E-53`, `I-40`, `L-7`, `C-10`, `C-28`, `F-2`, `G-7`, `H-47`, `K-2`, `phase-4F`, `B-35`, `C-27`, `C-35`, `B-23`, `E-29`, `H-15`, `H-51`, `J-85`, `C-46`, `D-7`, `K-19`, `phase-3H`, `phase-3K`, `F-3`, `F-9`, `I-38`, `K-23`, `K-5`, `phase-4H`, `B-18`, `E-27`, `E-38`, `L-10`

### Per-family structural pair counts

- **PHASE:** 3
- **B:** 8
- **C:** 58
- **D:** 12
- **E:** 63
- **F:** 7
- **G:** 13
- **H:** 39
- **I:** 79
- **J:** 178
- **K:** 75
- **L:** 90
- **M:** 21
- **N:** 131
- **O:** 11
- **P:** 63
- **Q:** 30

---

## Method notes

1. Scanned each `*-AGGREGATE.md` for lines matching IX / E/DEP / E/INT / consumes / depends / COMPOSES / WIRING / closes / extends / gates / supersedes.
2. Extracted provider feature IDs `[B-Q]-\d+…` excluding same-family and ledger namespaces (`B-001/`).
3. PHASE providers also matched as `phase-*`; SSB kept as shared backbone id.
4. Best evidence line kept per (consumer family, provider) by score (IX > DEP > other).
5. Not a substitute for reading a unit plan before implementation; it is a navigation index over the discovery aggregates.

