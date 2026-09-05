# DEEP-I-004 exhaustive feature discovery

## Scope, integrity, and method

- Processed manifest: `docs/thefinalglance/feature-discovery/DEEP-SOURCE-MANIFEST.md:L824-L839`, heading `### DEEP-I-004`; the next chunk starts at L841.
- Declared chunk: family I, **10 files**, **1,155 logical lines**, **92,980 bytes**, all ten classified `numbered/index original`; target status **under 1,800**.
- Mechanical verification: fresh `sha256sum`, `wc -l`, and byte-size checks matched every manifest row and both aggregates exactly. Missing paths: **0**; unexpected paths: **0**; duplicate paths: **0**; SHA mismatches: **0**; line mismatches: **0**; byte mismatches: **0**.
- Read boundary: all ten listed originals were read completely, first through last logical line. No original assigned to another deep chunk was opened. `09-I-SUMMARY_INDEX.discovery.md` was consulted only as an orientation and delta-comparison artifact; no source atom below is derived solely from it.
- Anchor notation refers to the manifest-verified source images: `I-2:Lx-Ly`, `I-3:Lx-Ly`, `I-12:Lx-Ly`, `I-21:Lx-Ly`, `I-27:Lx-Ly`, `I-34:Lx-Ly`, `I-35:Lx-Ly`, `I-40:Lx-Ly`, `I-42:Lx-Ly`, and `I-45:Lx-Ly`.
- Repeated premise, revised-scope, wiring, grounding, composition, and acceptance language is collapsed into one atom with every load-bearing anchor. Interaction, ambiguity/conflict, and summary-delta records reference atoms and are not counted again as features.
- Classification: `E` = explicit source fact/requirement; `I` = bounded inference; `A` = unresolved ambiguity/choice; `C` = internal conflict or summary correction that must not be silently flattened. Domains: `STATUS`, `CAP`, `SURFACE`, `FLOW`, `STATE`, `DATA`, `CFG`, `INT`, `EVENT`, `SEC`, `PRIV`, `COMP`, `FAIL`, `RECOVERY`, `RETRY`, `DET`, `IDEM`, `CONC`, `OBS`, `TEST`, `LIMIT`, `DEP`, `DEFER`, `EVIDENCE`.

### Exact processed manifest and verification

| Manifest class | Lines | Bytes | SHA-256 | Exact absolute path | Verification |
|---|---:|---:|---|---|---|
| numbered/index original | 149 | 11,710 | fdaa9fe71d80559f57d01ef7d3ede36e00b0588d3fa301c129a51410db261474 | docs/thefinalglance/I-plan/I-2_cow_workspace_isolation.md | exact |
| numbered/index original | 129 | 9,829 | 8bad3f027824c9a46efcae6b28d188dc648fb929b682afdcb399b957611abc5f | docs/thefinalglance/I-plan/I-3_ssb_throughput_atomic_saves.md | exact |
| numbered/index original | 99 | 8,318 | 1e1cc7c674fe4856276377b4c963285d753a8e05d26f23c5b2b6474a3b59fc0e | docs/thefinalglance/I-plan/I-12_evidence_gated_playbook_curator.md | exact |
| numbered/index original | 113 | 11,881 | 9fd839ddafb7566288f9f399d4d73264d4e0fc9e43fccac23f67c6141e5bdfc9 | docs/thefinalglance/I-plan/I-21_sprint_contracts_ralph_loop.md | exact |
| numbered/index original | 93 | 7,288 | df9b3e9b1bf8c7c447a68b034b5e7aa9b516e59c02def240b4f9d9c175fa5f48 | docs/thefinalglance/I-plan/I-27_budgeted_convergence_instrumentation.md | exact |
| numbered/index original | 151 | 11,580 | 67b08387af37ab07ec0b5072c2a1fc84ddd8c381f4977b98cb91d5a14b6e9702 | docs/thefinalglance/I-plan/I-34_crane_hybrid_grammar_pinning.md | exact |
| numbered/index original | 108 | 8,542 | 9af1b88d39bba20f0ac5dfcc989d9e330284495e03ef9a83b2a6790876843a3a | docs/thefinalglance/I-plan/I-35_temporal_tool_sequence_enforcement.md | exact |
| numbered/index original | 123 | 9,379 | 8924710544fb9f2891a1677ed079ba9e09d93c647375ec536ec6d421a6a107d3 | docs/thefinalglance/I-plan/I-40_trust_layer_wrapper_rra_spec.md | exact |
| numbered/index original | 118 | 9,511 | 3474fa466b27fd10239e0336b81b3c123927c3ad4a32e38ca35d9cdef4f914f1 | docs/thefinalglance/I-plan/I-42_critique_only_profile.md | exact |
| numbered/index original | 72 | 4,942 | 8259c203eb17be3e60f4ceef392a3ed9d598ce4d996c47883a06e06d89afc295 | docs/thefinalglance/I-plan/I-45_objective_hash_sealed_receipts.md | exact |
| **Aggregate** | **1,155** | **92,980** | — | **10 exact paths** | **exact** |

## Per-file atomic ledgers

### I-2 — CoW workspace isolation

Source: `docs/thefinalglance/I-plan/I-2_cow_workspace_isolation.md` (149/149 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| I2-E001 | E/STATUS | I-2 is a strong-grounded Track-12 BUILD, concurrency lever, effort M-L, Wave 2, bound to ID-2, and dependent on I-1. | I-2:L1-L3 |
| I2-E002 | E/CONC | The motivating defect is that parallel tool-calling branches share a mutable working directory, can silently overwrite intermediate state, and therefore make wide search and merge results unsafe/non-deterministic. | I-2:L4-L9 |
| I2-E003 | E/CAP | Tier 1 creates `crates/orchestrator/src/sandbox/branch_workspace.rs::BranchWorkspace` with `spawn`, `escape_check`, and `barrier_merge`, plus `WorktreePoolImpl`. | I-2:L38-L45, L53-L58, L63-L69 |
| I2-E004 | E/FLOW | Every branch/agent receives a private workspace root; production branch dispatch is `task_runner.rs -> Supervisor -> BranchWorkspace::spawn`. | I-2:L6-L9, L56-L57, L77-L81 |
| I2-E005 | E/CFG | The cross-platform Tier-1 default is a pooled warm git-worktree implementation with shared Cargo target-dir: source is isolated while build artifacts are shared. | I-2:L12-L15, L40-L42 |
| I2-E006 | E/DEFER | Linux overlayfs/btrfs-reflink CoW is a separately gated Tier-2 implementation behind the same trait, not a cross-platform assumption. | I-2:L12-L14, L47-L48, L83-L85 |
| I2-E007 | E/SEC | The cloud-sandbox/E2B-class adapter is optional, never a hard dependency, Tier-2, and user-approval gated. | I-2:L14-L15, L47-L50, L84-L85 |
| I2-E008 | E/FLOW | Merges occur only at the existing explicit BSP barrier; `bsp/engine.rs` must compose `BranchWorkspace::barrier_merge` rather than reimplement barrier logic. | I-2:L8-L9, L15-L17, L52-L55, L72-L73, L79-L80, L89-L90 |
| I2-E009 | E/DET | Barrier inputs are ordered exclusively by SSB sequence; any other merge order is a block. | I-2:L15-L17, L42-L43, L90-L90 |
| I2-E010 | E/RECOVERY | Overlapping hunks produce `ConflictEscalated`, halt the merge, and restore the trunk through the existing checkpoint/H-49 restore semantics. | I-2:L16-L18, L31-L34, L43-L44, L71-L76, L97-L103, L116-L122 |
| I2-E011 | E/FLOW | The stated non-overlap policy is “last-seq wins on non-overlapping hunks”; overlap is never auto-resolved. | I-2:L15-L18 |
| I2-E012 | E/DET | An identical-seed rerun must produce a byte-identical merged file under GD-2. | I-2:L17-L19, L44-L45, L96-L103 |
| I2-E013 | E/SEC | An absolute-path write outside a branch root must be intercepted by the Supervisor escape gate before filesystem mutation and emit `EscapeBlocked`. | I-2:L8-L9, L18-L19, L40-L45, L56-L57, L96-L103 |
| I2-E014 | E/EVENT | I-2 adds one `BranchWorkspaceEvent` action type with discriminator arms `Spawned`, `Merged`, `EscapeBlocked`, and `ConflictEscalated`, expressed as baseline+1 from the dated 221/532 registry. | I-2:L42-L45, L63-L69, L112-L113 |
| I2-E015 | E/SURFACE | Existing C-23 `BranchTree` is extended with workspace badges from `Spawned`/`Merged`/`EscapeBlocked` rows through its existing Tauri read; no new branch UI is built, and `cross_lang_diff.py` must pass. | I-2:L29-L30, L60-L61, L74-L75, L91-L94, L119-L120 |
| I2-E016 | E/TEST | The contract test must prove disjoint roots, byte-identical deterministic reruns, and a pre-write-blocked escape fixture. | I-2:L44-L45, L58-L58, L96-L103 |
| I2-E017 | E/TEST | A seeded-conflict run must emit `ConflictEscalated` followed by trunk rollback; a clean two-branch run must emit two `Spawned` rows and an SSB-ordered `Merged` row. | I-2:L95-L103 |
| I2-E018 | E/OBS | Per-branch root prefixes are journaled so root disjointness is row-verifiable; a missing event row or non-deterministic merged output is an R1/R2 failure. | I-2:L95-L103 |
| I2-E019 | E/STATUS | `bsp/engine.rs::recover_from_checkpoint` and a test exist, but production reachability was not verified in this plan pass and must be retraced under R7. | I-2:L23-L26, L71-L72, L105-L108 |
| I2-E020 | E/STATUS | No branch-workspace module/layer existed in the cited workspace inventory, making this a genuine build. | I-2:L6-L7, L23-L26, L35-L36 |
| I2-E021 | E/DEP | E-15 Firecracker pool, C-11 sandbox seam, and C-23 BranchTree are PLAN-asserted shipped dependencies that must be re-grepped at execution and composed, not rebuilt. | I-2:L27-L30, L73-L75, L118-L120 |
| I2-E022 | E/DEP | H-49 divergence/restore semantics are PLAN-asserted planned; they own post-conflict rollback and must not be reimplemented by I-2. | I-2:L31-L32, L76-L76, L121-L121 |
| I2-E023 | E/DEP | The Phase-3D BSP aggregator/barrier is PLAN-asserted shipped; its exact production barrier site must be re-grepped at execution. | I-2:L33-L34, L54-L55, L72-L73, L106-L108 |
| I2-E024 | E/DATA | Tier 1 creates `crates/contracts/tests/i2_cow_workspace_contract.rs` and `workflow/reasoning/I-2-decisions.md` in addition to the runtime module. | I-2:L53-L59, L63-L69 |
| I2-E025 | E/INT | Qualifying production reachability is the parallel task launch into `spawn` and the BSP barrier into `barrier_merge`; a test-only/contracts caller never satisfies R7. | I-2:L78-L81 |
| I2-E026 | E/LIMIT | Shared mutable source workspaces, cross-platform overlayfs assumptions, and hard cloud-sandbox dependencies are prohibited. | I-2:L20-L21, L82-L85 |
| I2-E027 | E/LIMIT | Production paths prohibit `.unwrap()`/`.expect()`, blake2/sha2, fabricated measurements/anchors, duplicate BSP barrier logic, and non-SSB merge order. | I-2:L86-L90 |
| I2-E028 | E/DEFER | Cross-branch CRDT merge behavior beyond the sequence-ordered policy is deferred to I-29 diffusion drafts. | I-2:L47-L50 |
| I2-E029 | E/DEP | I-4 parallel evidence-ranked search is explicitly blocked on I-2 workspace isolation. | I-2:L4-L6, L122-L123 |
| I2-E030 | E/FLOW | The J-11 amendment adopts Arbor’s branch start point: each new worktree begins at current trunk HEAD so it includes prior merges, not original repository HEAD. | I-2:L125-L132 |
| I2-E031 | E/DATA | Arbor prior art filters generated `.coordinator/**`, `results/**`, `models/**`, `logs/**`, and similar artifacts from finalize commits via `filter_commit_paths`. | I-2:L125-L135 |
| I2-E032 | E/STATE | Worktree teardown removes the ephemeral worktree but preserves the branch as the durable later-merge artifact. | I-2:L133-L137 |
| I2-E033 | E/CONC | Merge evaluation runs in a detached worktree pool distinct from the executor worktree pool. | I-2:L138-L140 |
| I2-E034 | E/RECOVERY | Stale worktrees are force-removed before creation, and name collisions receive timestamp suffixes. | I-2:L141-L142 |
| I2-E035 | E/OBS | Arbor’s configured after-executor hook snapshots the submission artifact; the amendment identifies this as I-2’s barrier-merge analogue. | I-2:L143-L144 |
| I2-E036 | E/LIMIT | Arbor itself is Unix-only and does not pool worktrees; I-2 deliberately retains Windows-safe warm pooling as an improvement. | I-2:L145-L147 |
| I2-E037 | E/EVIDENCE | The J-11 amendment is source-cited to exact Arbor executor/git-artifact/git-ops anchors and says it grounds lifecycle semantics without changing I-2 scope. | I-2:L125-L149 |
| I2-I001 | I/STATE | The amendment’s preserved branch is the implied durable handoff from an ephemeral executor worktree to later barrier/merge evaluation; no separate handoff object is named. | I-2:L133-L144 |
| I2-A001 | A/DET | “Last-seq wins on non-overlapping hunks” does not define what sequence precedence changes when hunks do not overlap; exact patch application/canonical merge semantics remain unspecified. | I-2:L15-L18 |
| I2-A002 | A/SEC | The escape requirement names absolute-path writes but does not define symlink, `..`, mount, junction, or path-canonicalization behavior. | I-2:L8-L9, L18-L19, L40-L45, L101-L103 |
| I2-A003 | A/DEP | Tier-1 acceptance requires rollback after `ConflictEscalated`, while H-49 restore semantics are still PLAN-asserted planned; the temporary behavior if H-49 has not landed is not specified. | I-2:L31-L32, L43-L45, L76-L76, L97-L103 |
| I2-A004 | A/SURFACE | `ConflictEscalated` is an event arm but is omitted from the explicit BranchTree badge arm list, leaving its exact operator-visible rendering unresolved. | I-2:L43-L45, L60-L61, L67-L68, L91-L94 |
| I2-C001 | C/DEFER | The opening design says Linux “gets” overlayfs/btrfs reflink, but the controlling revised scope moves that implementation to Tier 2; Tier 1 is the worktree default. | I-2:L12-L15, L38-L50 |

Category coverage: capabilities/actions/surfaces ✓; workflows/states ✓; schemas/data/lifecycle ✓; defaults/config ✓; integrations/events ✓; security/permissions ✓ (no standalone privacy/compliance rule); failures/rollback ✓; determinism/concurrency ✓; observability/evidence ✓; tests/done gates ✓; constraints/non-goals ✓; dependencies/status/deferred ✓. No retry API or formal idempotency key is specified; rerun byte identity is the applicable invariant.

### I-3 — SSB throughput and atomic checkpoint saves

Source: `docs/thefinalglance/I-plan/I-3_ssb_throughput_atomic_saves.md` (129/129 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| I3-E001 | E/STATUS | I-3 is a strong-grounded Track-12 performance EXTENSION, effort S, Wave 1. | I-3:L1-L3 |
| I3-E002 | E/CONC | The SSB is the single append-only audit log for I-plan rows; search-era fanout motivates hardening it against write load and mid-write crashes. | I-3:L4-L9 |
| I3-E003 | E/FLOW | Tier 1 hardens the existing **single-writer SQLite** append path; concurrent multi-writer fanout is not part of the immediate implementation. | I-3:L11-L13, L49-L53 |
| I3-E004 | E/CFG | The Phase-3B pragma block must be audited/extended to include WAL mode, `busy_timeout`, `cache_size`, and `wal_autocheckpoint` exactly as the Phase-3B spec requires. | I-3:L11-L14, L38-L42, L55-L56, L66-L70 |
| I3-E005 | E/RECOVERY | Every non-atomic checkpoint write becomes `<path>.tmp -> fsync -> atomic rename`; readers must observe either the old complete file or the new complete file, never a torn file. | I-3:L13-L16, L42-L45, L57-L59, L68-L70 |
| I3-E006 | E/OBS | Actual insert throughput is measured from the real SSB write loop under seeded fixture load and stored in `metrics/phase_I3.toml`, rather than taken from a synthetic microbenchmark or paper. | I-3:L16-L20, L42-L44, L60-L60, L68-L70 |
| I3-E007 | E/DET | The throughput measurement must be recomputable under `ELAI_SEED` and becomes the local ID-5 baseline for future claims. | I-3:L16-L20, L42-L44, L102-L105, L111-L116 |
| I3-E008 | E/CFG | `io_uring` is a Linux-only Tier-2 experiment behind `ELAI_IO_URING_ENABLE`, with a `wire-dark-allowlist.toml` row/promotion plan and a separate enablement gate; it is never default. | I-3:L17-L20, L36-L36, L49-L51, L88-L88 |
| I3-E009 | E/LIMIT | No external/paper throughput number may become a target; the measured TOML value is the baseline. | I-3:L8-L9, L19-L20, L86-L86, L111-L116 |
| I3-E010 | E/LIMIT | Switching storage engines and asynchronous batched SSB writes are declined because SQLite+WAL and append-only golden locking are established invariants. | I-3:L21-L23 |
| I3-E011 | E/STATUS | The cited dated live registry was `SCHEMA_VERSION=221` and `TOTAL_VARIANTS=532` at `ssb.rs:2981` and `action_type_lock.rs:601`. | I-3:L25-L27, L107-L108 |
| I3-E012 | E/STATUS | Phase-3B pragmas and Phase-3C atomic writes are PLAN-asserted; some sites may already comply, so execution must re-grep/audit every actual checkpoint site. | I-3:L28-L31, L34-L35, L38-L43, L107-L110 |
| I3-E013 | E/DEP | H-44 durability is PLAN-asserted planned and composes with this atomic-write pattern; I-3 must not absorb H-44’s checkpoint scope. | I-3:L32-L33, L77-L77, L118-L122 |
| I3-E014 | E/STATUS | No `io_uring` integration existed in the cited tooling/crate inventory. | I-3:L36-L36 |
| I3-E015 | E/TEST | `cargo check` must confirm that the pragma/write-loop edit introduces no regression. | I-3:L38-L42 |
| I3-E016 | E/TEST | A kill-9 crash fixture must prove structural old-or-new completeness; the non-atomic broken arm must exhibit a torn file and fail. | I-3:L45-L45, L97-L105 |
| I3-E017 | E/EVENT | I-3 adds no action type and must reuse existing SSB append infrastructure, leaving the dated 221/532 baseline unchanged. | I-3:L45-L47, L67-L71, L85-L90, L100-L101, L115-L116 |
| I3-E018 | E/DEFER | Multi-writer fanout hardening waits until I-4 ships and real contention has been measured, with the decision deferred to full-form expansion. | I-3:L49-L53 |
| I3-E019 | E/INT | The planned edit set is the pragma/write loop in `crates/orchestrator/src/ssb.rs`, all remaining checkpoint sites in `session_dag.rs` and `orchestrator/`, plus `workflow/reasoning/I-3-decisions.md`. | I-3:L55-L61 |
| I3-E020 | E/SURFACE | I-3 intentionally adds no live UI: its observable artifacts are the metrics TOML and crash-fixture result, with future cockpit consumption assigned to I-27. | I-3:L62-L64, L93-L96 |
| I3-E021 | E/INT | Production reachability is the SSB write loop used by real row-emitting runs and the `session_dag.rs` checkpoint call; test-only/contracts callers do not satisfy R7. | I-3:L80-L84 |
| I3-E022 | E/LIMIT | Production edits prohibit `.unwrap()`/`.expect()`, blake2/sha2, fabricated locations, default-path `io_uring`, a new I-3 action type, and any weakening of append-only golden locking. | I-3:L85-L92 |
| I3-E023 | E/TEST | The acceptance trigger runs a seeded real SSB fixture, kills the process mid-checkpoint, and parses `insert_throughput_per_sec` from `metrics/phase_I3.toml`, which must be greater than zero. | I-3:L97-L100 |
| I3-E024 | E/OBS | The normal fixture must emit existing SSB rows without regression and must emit no I-3-specific variant. | I-3:L100-L101 |
| I3-E025 | E/TEST | Crash completeness is checked using file length plus blake3 against the pre-crash snapshot; the negative arm without temp/fsync/rename must fail. | I-3:L102-L105 |
| I3-E026 | E/DEP | I-3 Wave 1 precedes I-1 Wave 2 and I-4 Wave 4; durable Supervisor rows and high-fanout search appends consume its hardened path. | I-3:L123-L127 |
| I3-I001 | I/DATA | Because I-27 is named as the future cockpit consumer, `metrics/phase_I3.toml` implicitly needs a stable machine-readable schema even though I-3 specifies only one field. | I-3:L60-L64, L93-L96, L97-L105 |
| I3-A001 | A/CFG | The source names the required pragmas but supplies no concrete values for timeout, cache size, or autocheckpoint interval. | I-3:L11-L14, L38-L42 |
| I3-A002 | A/STATUS | Exact non-atomic checkpoint locations and which pragma clauses already ship are deliberately unresolved until execution-time audit. | I-3:L28-L35, L42-L43, L57-L59 |
| I3-A003 | A/TEST | Fixture workload, duration, hardware normalization, and any minimum acceptable throughput beyond `>0` are not defined. | I-3:L16-L20, L97-L105 |
| I3-A004 | A/RECOVERY | The text invokes POSIX rename but does not specify cross-platform replacement behavior, parent-directory fsync, temp-file cleanup, or retry after a failed rename. | I-3:L13-L16, L42-L45 |
| I3-C001 | C/CONC | The premise mentions concurrent writers, but the revised Tier-1 design remains single-writer and explicitly defers multi-writer hardening; this is durability/throughput hardening, not immediate multi-writer support. | I-3:L4-L9, L11-L13, L49-L53 |

Category coverage: capabilities/surfaces ✓ (internal CLI/artifact surface, intentionally no UI); workflows/states ✓; schema/data/lifecycle ✓; flags/defaults/config ✓; integrations/events ✓; security/integrity ✓ (no privacy/compliance policy); failures/recovery ✓; determinism/concurrency ✓; observability/evidence ✓; tests/done gates ✓; constraints/non-goals ✓; dependencies/status/deferred ✓. Retry behavior and formal idempotency are not specified.

### I-12 — evidence-gated playbook curator

Source: `docs/thefinalglance/I-plan/I-12_evidence_gated_playbook_curator.md` (99/99 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| I12-E001 | E/STATUS | I-12 is a strong-grounded Track-14 steering BUILD, effort M, Wave 3, governed by ID-6. | I-12:L1-L3 |
| I12-E002 | E/FLOW | `PlaybookCurator` may promote a rule/playbook entry only when it links to at least `k` SSB-verified successful outcomes. | I-12:L4-L8, L10-L13, L34-L37, L52-L56 |
| I12-E003 | E/OBS | Each candidate carries measured lift from a cheap on/off-injection job run on an offline tick; promoted content retains that measured lift. | I-12:L6-L8, L10-L14, L35-L40, L52-L55 |
| I12-E004 | E/TEST | A merged rule must pass the C-28 mutation suite with zero regressions before any ACTIVE promotion. | I-12:L12-L14, L35-L37, L60-L60, L69-L76, L79-L82 |
| I12-E005 | E/STATE | The curator feeds, and never replaces, the existing Phase-3F `PROPOSED -> SHADOW -> ACTIVE` lifecycle. | I-12:L15-L17, L26-L26, L38-L40, L47-L49, L58-L58, L91-L92 |
| I12-E006 | E/DEP | `TemplateCrystallizer`/`CompiledPipelineStore` provide the compiled-AI zero-token storage precedent; C-25 contributes the earlier success-count pattern. | I-12:L15-L17, L24-L27, L47-L48, L59-L59, L84-L86, L93-L93 |
| I12-E007 | E/SURFACE | `elai playbook export` writes verified entries in CLAUDE.md, AGENTS.md, and skills-compatible formats with embedded supporting evidence-row IDs. | I-12:L18-L20, L37-L39, L49-L49, L53-L56, L79-L82 |
| I12-E008 | E/STATE | The same ledger automatically demotes stale or regressing ACTIVE entries without requiring a human edit. | I-12:L19-L22, L35-L37, L52-L55, L79-L82 |
| I12-E009 | E/DEP | F-5 staleness demotion is the predecessor gate; I-12 extends it with evidence count rather than replacing it. | I-12:L21-L22, L30-L30, L62-L62, L91-L97 |
| I12-E010 | E/SEC | ID-6 treats learned weights/rules as configuration: promotion must traverse trial-ledger gates and the evaluator must remain outside the loop’s write boundary under R8 gate freeze. | I-12:L31-L32, L61-L61, L68-L76, L84-L89 |
| I12-E011 | E/EVENT | I-12 adds one `PlaybookEvent` action type with `Promoted`, `Demoted`, `ValidationRun`, and `Exported` kinds, baseline+1 from the dated 221/532 registry. | I-12:L35-L38, L52-L56, L79-L82, L84-L89 |
| I12-E012 | E/CAP | Tier 1 creates `crates/orchestrator/src/rule_curator.rs::PlaybookCurator` with evidence-count promotion, demotion, lift measurement, and validation behavior. | I-12:L34-L37, L46-L48, L52-L55 |
| I12-E013 | E/INT | The CLI subcommand is added under the PLAN-asserted `skills_cmd` family and must be re-grepped at execution. | I-12:L46-L50 |
| I12-E014 | E/CFG | Promotion evaluation runs through a Layer-1A offline scheduler entry whose interval is configurable. | I-12:L35-L40, L64-L67 |
| I12-E015 | E/DEP | The curator composes C-28 `MutationRunner`/`EquivalentMutantCache`, IS-16 `trial_ledger.rs`, E-8 GEPA’s metric shape, and the Phase-3F engine. | I-12:L28-L32, L47-L49, L57-L63, L84-L97 |
| I12-E016 | E/SURFACE | Evidence count, lift, and demotion events are user-actionable rows in the existing `RulesDrawer` playbook panel. | I-12:L49-L50, L77-L78 |
| I12-E017 | E/INT | Qualifying production paths are CLI `export_verified()` and scheduler `run_promotion_tick()`; test-only/contracts callers do not establish R7 reachability. | I-12:L64-L67 |
| I12-E018 | E/FAIL | A zero-evidence attempt is refused with `ValidationRun{outcome_count:0}` and no ACTIVE transition. | I-12:L68-L70, L79-L82 |
| I12-E019 | E/STATE | Successful promotion evidence includes `ValidationRun{regression_count:0}` before `PlaybookEvent{kind:Promoted,...}`. | I-12:L79-L82 |
| I12-E020 | E/FAIL | A candidate with at least one C-28 regression emits a positive `regression_count` and is refused. | I-12:L69-L70, L79-L82 |
| I12-E021 | E/LIMIT | I-12 must not fork/reimplement the Phase-3F promotion state machine. | I-12:L15-L17, L38-L40, L68-L72, L91-L92 |
| I12-E022 | E/EVIDENCE | ACE +10.6% and DSPy/MIPROv2 +13% are DESIGN-tagged prior-art figures requiring web confirmation and local measurement; neither may become a threshold. | I-12:L10-L14, L68-L73, L84-L89 |
| I12-E023 | E/LIMIT | Production promotion paths prohibit `.unwrap()`/`.expect()`, blake2/sha2, and self-evaluation inside the mutable loop boundary. | I-12:L68-L76 |
| I12-E024 | E/DEFER | Cross-corpus transfer validation waits for an I-38 QDELTA baseline. | I-12:L42-L44 |
| I12-E025 | E/DEFER | Automatic CLAUDE.md update is deferred behind user-approved PR-diff preview, `ELAI_PLAYBOOK_AUTOUPDATE_ENABLE`, a wire-dark allowlist row, and a promotion plan. | I-12:L42-L45 |
| I12-E026 | E/DATA | The required reasoning artifact is `workflow/reasoning/I-12-decisions.md`. | I-12:L46-L50 |
| I12-E027 | E/STATUS | IS-13 storage, IS-8 mutation, and IS-16 ledger seams are cited as existing/PLAN-asserted and require execution-time re-grep where noted. | I-12:L24-L32, L84-L89 |
| I12-I001 | I/STATE | Because export is restricted to verified/promoted entries and promotion is owned by Phase 3F, export is a read/serialization path rather than an alternate path to ACTIVE. | I-12:L15-L20, L37-L40, L53-L66 |
| I12-A001 | A/CFG | The required success threshold `k` has no value, derivation rule, or configuration-key name. | I-12:L6-L7, L12-L13, L35-L37 |
| I12-A002 | A/CFG | The offline scheduler interval is configurable but has no key name, unit, or default. | I-12:L35-L40 |
| I12-A003 | A/OBS | The on/off lift estimator does not define its corpus, pairing/randomization, minimum sample, confidence interval, or promotion threshold. | I-12:L10-L14, L35-L37 |
| I12-A004 | A/DATA | The exact serialized export block schemas, merge/collision rules, and behavior when target files already contain generated blocks are not specified. | I-12:L18-L20, L37-L39, L53-L56, L79-L82 |

Category coverage: capabilities/CLI/UI ✓; workflows/states/lifecycle ✓; data/schema ✓ at field/event level (serialized block schema unresolved); flags/config ✓; integrations/events ✓; security/permissions ✓ (user approval and frozen evaluator; no standalone privacy/compliance rule); failures/fail-closed ✓; determinism/gate isolation ✓; observability/evidence ✓; tests/done gates ✓; constraints/non-goals ✓; dependencies/status/deferred ✓. Retry and idempotency behavior are absent.

### I-21 — sprint contracts and bounded Ralph loop

Source: `docs/thefinalglance/I-plan/I-21_sprint_contracts_ralph_loop.md` (113/113 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| I21-E001 | E/STATUS | I-21 is a strong-grounded Track-15 trust/anti-bias BUILD, effort M, Wave 3, with IC-17 persona consistency; full-form expansion is mandatory before EXECUTING. | I-21:L1-L3, L111-L111 |
| I21-E002 | E/EVIDENCE | The motivating +0.12 self-review-bias figure is carried from research/drafts and must be web-confirmed before use as a headline gate number. | I-21:L4-L9, L96-L101 |
| I21-E003 | E/DATA | The initial Sprint Contract shape is `{goal, in_scope_files, acceptance_criteria, bounded_iteration_count, out_of_scope_marker}`. | I-21:L11-L16 |
| I21-E004 | E/DATA | A Sprint Contract is machine-readable structured data, not a free-text plan. | I-21:L11-L16 |
| I21-E005 | E/TEST | Each acceptance criterion must deterministically reference a test symbol, compile status, or wiring-contract field. | I-21:L13-L16, L49-L52, L83-L88 |
| I21-E006 | E/FAIL | Vague/open-ended criteria such as “implementation feels complete” are rejected before execution/at approval. | I-21:L13-L16, L49-L52, L83-L86, L91-L94 |
| I21-E007 | E/SEC | The isolated evaluator receives only the sealed Sprint Contract and task spec—no generator turns, intermediate outputs, or scratchpad. | I-21:L17-L21, L35-L36, L83-L85 |
| I21-E008 | E/FLOW | I-1 Supervisor enforces isolation by spawning an evaluator Directive whose payload is sealed and whose generator context is pruned. | I-21:L17-L20, L44-L45, L52-L52, L74-L79 |
| I21-E009 | E/CFG | The evaluator runs the D-13 adversarial posture at temperature zero with a retained hostile-critic persona because the work is alignment-dependent. | I-21:L20-L21, L30-L34, L96-L101 |
| I21-E010 | E/EVENT | Receipt of a contract emits `SprintContractEvent(Proposed)`. | I-21:L22-L24, L49-L56 |
| I21-E011 | E/STATE | Approval emits `Approved`; rejection emits `Rejected` with structured, concrete missing/ambiguous/scope-creep objections rather than a generic response. | I-21:L22-L24, L49-L56, L65-L67 |
| I21-E012 | E/FLOW | At generated-output exit, I-17 `TriLayerStack` checks the criteria; the reinjected trace must identify the exact failing gate and line. | I-21:L25-L29, L45-L45, L53-L55, L77-L79 |
| I21-E013 | E/RETRY | A failed check reinjects its trace for one retry and emits `RalphReinjected`. | I-21:L25-L29, L53-L56, L91-L94 |
| I21-E014 | E/RETRY | The Ralph loop is capped at **three total iterations**, meaning at most two reinjections. | I-21:L25-L29, L35-L36, L53-L55, L83-L86 |
| I21-E015 | E/RECOVERY | The third failure escalates to the existing D-13 cross-family adversary, emits `AdversaryEscalated`, and requires a concrete counterexample rather than only a score. | I-21:L27-L29, L41-L43, L53-L56, L74-L79, L91-L94 |
| I21-E016 | E/FAIL | The loop counter is SSB-tracked; overflow is a BLOCK, and H-38 provides stop-gate enforcement at the cap. | I-21:L27-L29, L41-L42, L53-L55, L74-L79, L83-L88 |
| I21-E017 | E/LIMIT | Shared generator/evaluator context, an unbounded loop, and non-deterministically checkable criteria are explicitly excluded. | I-21:L35-L36 |
| I21-E018 | E/STATUS | No sprint-contract module existed in the cited inventory; I-21 is a genuine build. | I-21:L4-L6, L38-L40, L96-L98 |
| I21-E019 | E/DEP | H-37 gate, H-38 stop gate, D-13 adversary, and C-32 MiniCritic posture are PLAN-asserted shipped composition seams, not code to duplicate. | I-21:L38-L46, L74-L79, L105-L109 |
| I21-E020 | E/RECOVERY | If I-1 has not landed, Tier 1 permits a temporary evaluator-isolation shim that is later replaced. | I-21:L44-L44, L52-L52 |
| I21-E021 | E/DEP | I-17 `TriLayerStack` is the sole acceptance-check trigger; I-21 must not add an ad-hoc duplicate checker. | I-21:L45-L45, L53-L55, L77-L79, L105-L109 |
| I21-E022 | E/CAP | Tier 1 creates `crates/orchestrator/src/sprint_contract.rs::SprintContract` and `::RalphLoop`, including validation, Directive construction, loop counting, and escalation. | I-21:L48-L56, L61-L73 |
| I21-E023 | E/EVENT | I-21 adds one `SprintContractEvent` action type with `Proposed`, `Approved`, `Rejected`, `RalphReinjected`, and `AdversaryEscalated`, baseline+1 from the dated 221/532 registry. | I-21:L49-L56, L65-L73, L103-L103 |
| I21-E024 | E/DEFER | Multi-contract sprints are deferred; Tier 1 only leaves a contract-chaining extension seam. | I-21:L57-L59 |
| I21-E025 | E/DEFER | Automatic generation of acceptance criteria is deferred; manually supplied deterministic criteria ship first. | I-21:L57-L59 |
| I21-E026 | E/INT | I-21 extends H-37 `run_interactive_gate` with a new payload and modifies `search/ladder.rs` or `task_runner.rs` at an execution-time-re-grepped outer retry seam. | I-21:L61-L65 |
| I21-E027 | E/SURFACE | A Sprint Contract card beside `PlanApprovalCard` shows contract fields, criterion-determinism badges, verdict/objection, and loop count via existing SSB subscription; `cross_lang_diff.py` must pass. | I-21:L65-L67, L89-L90 |
| I21-E028 | E/INT | Production reachability is task-runner/search-ladder sprint dispatch; contracts-only callers never satisfy R7. | I-21:L80-L82 |
| I21-E029 | E/LIMIT | Production code forbids generator context in the evaluator, counterless loops, free-text criteria, duplicated D-13 logic, `.unwrap()`/`.expect()`, and fabricated verdict/count data. | I-21:L83-L88 |
| I21-E030 | E/TEST | The acceptance trigger runs `elai run --sprint --task-fixture fixtures/vague_contract.toml --emit-ssb`. | I-21:L91-L92 |
| I21-E031 | E/OBS | The fixture must show `Rejected` with a concrete ambiguity objection, `RalphReinjected` after the first failed exit, and `AdversaryEscalated` on the third failure. | I-21:L92-L94 |
| I21-E032 | E/TEST | A journal join by session/role must prove the evaluator saw only the sealed payload and zero generator rows; a `--shared-context` arm must change verdict distribution and fail isolation. | I-21:L92-L94 |
| I21-E033 | E/DET | Approval and execution rows must byte-recompute under `ELAI_SEED`. | I-21:L93-L94 |
| I21-A001 | A/DATA | The proposal uses field name `bounded_iteration_count`, while revised Tier 1 uses `iteration_count`; the canonical serialized field name is unresolved. | I-21:L12-L13, L49-L51 |
| I21-A002 | A/TEST | “Reference to” an allowed symbol/status/field has no specified grammar, resolver, repository snapshot, or behavior for renamed/missing symbols. | I-21:L13-L16, L49-L52, L83-L86 |
| I21-A003 | A/DEP | I-1 is called the isolation substrate/hard sequencing need, yet a temporary shim is allowed; the shim’s interface, evidence, and removal gate are unspecified. | I-21:L17-L20, L44-L44, L52-L52, L77-L78 |
| I21-A004 | A/INT | The existing outer retry loop may be “replaced or supplemented”; the plan does not choose one or define nested-budget interaction. | I-21:L63-L64 |
| I21-A005 | A/STATE | The exact event/verdict sequence on the third failure—whether a final `Rejected`, stop event, or other row accompanies `AdversaryEscalated`—is not defined. | I-21:L22-L29, L53-L56, L91-L94 |

Category coverage: capabilities/UI ✓; workflows/states ✓; schemas/data ✓; model/default/config ✓; integrations/events ✓; evaluator isolation/security ✓ (no privacy/compliance policy); failure/retry/escalation/recovery ✓; determinism ✓ (no concurrency or idempotency contract); observability/evidence ✓; tests/acceptance/done gates ✓; constraints/non-goals ✓; dependencies/status/deferred ✓.

### I-27 — budgeted convergence instrumentation

Source: `docs/thefinalglance/I-plan/I-27_budgeted_convergence_instrumentation.md` (93/93 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| I27-E001 | E/STATUS | I-27 is a strong-grounded Track-16 provability BUILD, effort S-M, Wave 4. | I-27:L1-L3 |
| I27-E002 | E/CAP | It builds a public curve of W-1 verified-outcome rate versus total test-time compute for each configuration. | I-27:L4-L7, L9-L17 |
| I27-E003 | E/DATA | `bench_convergence.rs` sweeps a fixture suite through increasing compute envelopes expressed as nodes, tokens, and monetary spend. | I-27:L9-L12, L30-L36 |
| I27-E004 | E/DATA | Only journaled W-1 verified outcomes count; pass@k is explicitly excluded. | I-27:L10-L13, L17-L20, L64-L68 |
| I27-E005 | E/OBS | Each curve point is a mean with bootstrap confidence intervals over at least five paired seeds, inheriting I-38’s statistical floor. | I-27:L11-L13, L30-L36, L79-L84 |
| I27-E006 | E/SURFACE | A Pareto view overlays cost per additional verified task as a second axis so an operator can choose a cost/accuracy operating point. | I-27:L4-L7, L13-L15, L43-L47, L71-L72 |
| I27-E007 | E/OBS | The report identifies and names the knee where marginal gain collapses. | I-27:L13-L15, L74-L77 |
| I27-E008 | E/EVIDENCE | Any adjacent-level dip must be explicitly flagged as non-monotonic and never smoothed or generalized beyond the measured fixture sample. | I-27:L15-L20, L64-L68, L79-L82 |
| I27-E009 | E/EVENT | I-27 adds one flat `ConvergenceCurveMeasured` beacon/action type, baseline+1 from the dated 221/532 registry. | I-27:L15-L17, L74-L84 |
| I27-E010 | E/LIMIT | SWE-bench branding and external/paper compute-efficiency targets are prohibited; ELAI’s fixture suite is the corpus. | I-27:L18-L20, L64-L69, L79-L84 |
| I27-E011 | E/DEP | I-27 imports E-49 `$ / CTS` and `STABLE-Pass`, mirrors E-9’s fixture-runner shape, lands in H-9’s cockpit family, reads I-4 `SearchNodeEvent`, and shares I-38’s statistical method. | I-27:L22-L28, L43-L58, L79-L91 |
| I27-E012 | E/SEC | Under R8, the measured session may not modify the benchmark harness; the harness is measurement infrastructure, not a mutable gate. | I-27:L27-L27 |
| I27-E013 | E/CAP | Tier 1 creates `crates/cli/src/bench_convergence.rs::ConvergenceBench` for sweeping, aggregation, curve computation, and row emission. | I-27:L30-L36, L43-L52 |
| I27-E014 | E/SURFACE | Tier 1 creates/extends a dashboard-v2 convergence tile in the H-9 cost cockpit, rendering both curve and Pareto view through a Tauri-to-Svelte pair checked by `cross_lang_diff.py`. | I-27:L30-L35, L43-L47, L49-L53, L71-L72 |
| I27-E015 | E/TEST | Tier 1 includes `crates/contracts/tests/i27_convergence_instrumentation_contract.rs` and `workflow/reasoning/I-27-decisions.md`. | I-27:L35-L37 |
| I27-E016 | E/DEFER | Live/non-batch refresh waits for sustained I-4 trajectory volume. | I-27:L39-L41 |
| I27-E017 | E/DEFER | Cross-config overlays require at least three sufficiently sampled configurations and are RunPod-gated for GPU configurations. | I-27:L39-L42 |
| I27-E018 | E/SURFACE | Production CLI entry is `elai bench convergence --suite <fixture>`; the beacon is the off-agent artifact read by the cockpit. | I-27:L60-L63 |
| I27-E019 | E/DATA | The required report is `workflow/benchmarks/convergence_<fixture>_<date>.toml` with curve points, confidence intervals, Pareto data, knee label, and non-monotonic flags. | I-27:L74-L77 |
| I27-E020 | E/DET | `curve_points` must recompute byte-identically from the journal under `ELAI_SEED`; the row also carries CI bounds and `knee_compute_level`. | I-27:L74-L77 |
| I27-E021 | E/TEST | The acceptance run uses at least five seeds: `elai bench convergence --suite <fixture-set> --seeds 5`. | I-27:L74-L76 |
| I27-E022 | E/FAIL | A `--skip-verification` broken arm must inflate `verified_count`; the contract detects that inflation and fails. | I-27:L74-L77 |
| I27-E023 | E/LIMIT | Production code forbids pass@k, fabricated monotonicity, paper-derived targets, SWE-bench headlines, `.unwrap()`/`.expect()`, and blake2/sha2. | I-27:L64-L69 |
| I27-E024 | E/INT | E-49 definitions, E-9 runner skeleton, H-9 cockpit, I-4 telemetry, and I-38 bootstrap must be imported/composed rather than redefined. | I-27:L54-L58, L86-L91 |
| I27-I001 | I/DATA | `ConvergenceCurveMeasured` is implicitly an aggregate per suite/configuration rather than a per-node event because its payload must contain the complete recomputable curve and knee. | I-27:L9-L17, L74-L77 |
| I27-A001 | A/CFG | The fixture suite, exact compute-level sequence, repetitions per level, token/cost accounting boundaries, and configuration identity are not defined. | I-27:L9-L17, L30-L36 |
| I27-A002 | A/OBS | The bootstrap procedure, confidence level, knee-detection algorithm, and formal non-monotonic significance rule are unspecified. | I-27:L11-L17, L30-L36, L74-L84 |
| I27-A003 | A/DATA | Monetary denomination/provider normalization and the exact meaning of “cost per additional verified task” are left to imported E-49 definitions or later implementation. | I-27:L11-L15, L22-L23, L54-L56 |
| I27-A004 | A/DET | The row is seed-recomputable, but the report filename embeds `<date>` and no normalization rule is given for byte-stable artifact naming. | I-27:L74-L77 |

Category coverage: capability/CLI/UI ✓; workflow/state ✓; report/event data ✓; config/defaults △ (many parameters unresolved); integrations/events ✓; frozen-harness security ✓ (no privacy/compliance rule); failure behavior ✓; determinism ✓ (no idempotency/concurrency/retry contract); observability/evidence ✓; tests/done gates ✓; constraints/non-goals ✓; dependency/status/deferred ✓.

### I-34 — CRANE hybrid, grammar pinning, and post-parse validation

Source: `docs/thefinalglance/I-plan/I-34_crane_hybrid_grammar_pinning.md` (151/151 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| I34-E001 | E/STATUS | I-34 is a strong-grounded Track-18 trust/can’t-lie BUILD, effort M, Wave 1, bound to IC-18. | I-34:L1-L3 |
| I34-E002 | E/STATUS | `providers/src/decoding.rs::DecodingMode::Grammar{ebnf_path, crane_adaptive}` exists at the cited lines, but `crane_adaptive` has no runtime leg. | I-34:L5-L9, L33-L40, L94-L96 |
| I34-E003 | E/EVIDENCE | The premise carries three externally sourced risks—10–15pp constrained-reasoning loss, 38 silent JSONSchemaBench failures, and 96.2% EnumAttack ASR—which require web confirmation before headline use. | I-34:L5-L9, L94-L102 |
| I34-E004 | E/FLOW | On the local sidecar lane, CRANE permits free thinking tokens between structural commitment points and constrains the model when it commits to structured tokens. | I-34:L11-L18, L43-L49, L65-L68 |
| I34-E005 | E/SEC | Reflection/self-correction constrains only FINAL output; intermediate reasoning remains unconstrained to avoid structure snowballing. | I-34:L14-L18, L82-L86 |
| I34-E006 | E/DEP | C-37 re-evaluation is a consumer of the CRANE runtime; I-34 does not absorb C-37. | I-34:L17-L18, L37-L37, L61-L62, L104-L109 |
| I34-E007 | E/DATA | Every grammar is a blake3-pinned control-plane artifact recorded in a manifest modeled on E-24 MCP-schema pinning. | I-34:L20-L23, L43-L47, L57-L59, L65-L75 |
| I34-E008 | E/SEC | Third-party grammars are never loaded, and enum-bearing schemas from untrusted sources are rejected at load. | I-34:L20-L23, L30-L31, L82-L84 |
| I34-E009 | E/SEC | The grammar manifest is R8-frozen; tampering emits `GrammarGateEvent(PinViolation)`. | I-34:L20-L23, L43-L49, L77-L80 |
| I34-E010 | E/TEST | Every constrained generation is raw-reparsed by `re_validate_output()` against its schema after decoding. | I-34:L25-L28, L43-L49, L57-L58, L65-L70 |
| I34-E011 | E/FAIL | Silent noncompliance emits `PostParseRejected` and routes to the existing D-16 refusal path; skipping validation is the `--trust-mask` broken arm. | I-34:L25-L31, L47-L49, L60-L61, L76-L80, L89-L92 |
| I34-E012 | E/LIMIT | Loading a third-party grammar or trusting mask output without post-parse revalidation is explicitly excluded. | I-34:L30-L31 |
| I34-E013 | E/STATUS | E-35/E-18 XGrammar, C-26 `DecodingMode`, C-37, E-24 pins, and D-16 refusal are PLAN-asserted composition seams whose exact locations require execution-time re-grep. | I-34:L33-L41, L71-L80, L94-L109 |
| I34-E014 | E/CAP | Tier 1 implements the Rust/provider CRANE runtime, post-parse validator, grammar store/manifest/load verification, and `workflow/reasoning/I-34-decisions.md`. | I-34:L43-L51, L57-L70 |
| I34-E015 | E/EVENT | I-34 adds one `GrammarGateEvent` action type with `CranePassed`, `PinViolation`, and `PostParseRejected`, baseline+1 from the dated 221/532 registry. | I-34:L43-L49, L65-L70, L89-L102 |
| I34-E016 | E/TEST | Tier-1 fixtures cover a mutated grammar, seeded silent noncompliance, and reasoning-heavy CRANE whose measured pass rate must be at least its FreeText arm under local ID-5 measurement. | I-34:L47-L50, L89-L92 |
| I34-E017 | E/SURFACE | The existing decode panel gains a grammar-pin status chip through Svelte and Tauri; `cross_lang_diff.py` must pass. | I-34:L49-L50, L63-L63, L87-L92 |
| I34-E018 | E/DEFER | Python-sidecar mask-layer CRANE integration is Tier 2 and requires a separately gated, hardware-heavy sidecar boot. | I-34:L53-L55, L57-L60 |
| I34-E019 | E/DEFER | A cross-language grammar-compliance verifier pack is Tier 2 under I-17. | I-34:L53-L55 |
| I34-E020 | E/INT | Production reachability covers `DecodingMode::Grammar -> crane_adaptive`, grammar-manifest load verification, and post-parse validation into refusal; tests-only callers do not satisfy R7. | I-34:L77-L81 |
| I34-E021 | E/DATA | The new grammar-store module path is deliberately TBD until execution-time re-grep. | I-34:L65-L70 |
| I34-E022 | E/LIMIT | Production paths forbid third-party grammars, skipped validation, intermediate-reasoning constraints, `.unwrap()`/`.expect()`, and blake2/sha2. | I-34:L82-L86 |
| I34-E023 | E/OBS | The seeded fixture must emit `CranePassed`, `PinViolation`, and `PostParseRejected`; the status chip and broken-arm failure are required off-agent evidence. | I-34:L89-L92 |
| I34-E024 | E/EVIDENCE | CRANE, structure-snowballing, XGrammar, and JSONSchemaBench identifiers are carried from dossiers and explicitly require web confirmation before headline use. | I-34:L94-L102 |
| I34-E025 | E/DEP | The main design composes XGrammar’s mask layer, E-24’s pin protocol, D-16 refusal, and the existing decoding enum rather than reimplementing them. | I-34:L104-L109 |
| I34-E026 | E/EVIDENCE | The J-15 amendment cites a determinism-triage ledger, a dated crates.io anti-slopsquat snapshot, and a detLLM interface note as its grounding. | I-34:L111-L115 |
| I34-E027 | E/DEP | The amendment selects `llguidance` crates.io v1.7.6/MIT as a new orchestrator-side dependency, subject to edit-time reverification and user confirmation. | I-34:L117-L123, L141-L149 |
| I34-E028 | E/FLOW | Selected Option A compiles, validates, and blake3-pins a grammar with Rust `ParserFactory` at plan time; malformed grammars fail before any inference call. | I-34:L117-L123 |
| I34-E029 | E/DET | Grammar pins use the same blake3 stack as RRA so plan/execution drift can be detected by `replay --verify`. | I-34:L119-L123 |
| I34-E030 | E/INT | The Python sidecar’s XGrammar remains the inference-time masker; plan-time Rust parsing/pinning and runtime Python masking stay separate. | I-34:L124-L132 |
| I34-E031 | E/LIMIT | Option B—Rust-side token masking with llguidance—is rejected because it would redesign the Python logits loop, duplicate XGrammar, and violate the existing substrate contract. | I-34:L124-L132 |
| I34-E032 | E/LIMIT | The amendment cites HD-3 as banning constrained decoding on reasoning/coder/planner roles; the Rust addition is compile/validate only. | I-34:L128-L132 |
| I34-E033 | E/STATE | Compiled llguidance parser state is not serialized; the durable artifact is grammar spec plus blake3 pin, and the parser is recompiled on every boot. | I-34:L134-L139 |
| I34-E034 | E/EVIDENCE | Sub-second JSON-schema compilation is a README-derived performance claim, not a locally defined acceptance threshold. | I-34:L134-L138 |
| I34-E035 | E/SEC | Before adding the dependency, the executor must re-check crates.io identity/license/maintainer/download evidence and obtain explicit user confirmation. | I-34:L141-L147 |
| I34-E036 | E/EVIDENCE | The dated snapshot reports v1.7.6, MIT, roughly 525K downloads, and guidance-ai/Moskal/MS origin; it is expressly non-authoritative at edit time. | I-34:L141-L147 |
| I34-E037 | E/LIMIT | `xgrammar-rs` was considered and rejected as a C++ binding with far lower reported adoption that duplicates the existing XGrammar surface. | I-34:L147-L149 |
| I34-A001 | A/INT | The grammar-store module path and ownership are TBD, including whether it lives under providers or orchestrator. | I-34:L57-L60, L65-L70 |
| I34-A002 | A/SEC | “Untrusted source” and enum detection/trust provenance have no exact schema-loader classification algorithm. | I-34:L20-L23 |
| I34-A003 | A/FLOW | Commit-point identification, thinking-token grammar augmentation, and transition semantics are not specified as an algorithm or grammar transform. | I-34:L14-L18, L43-L49 |
| I34-A004 | A/DATA | Manifest serialization, pin entry schema, update authorization, and freeze-manifest nesting are not defined. | I-34:L20-L23, L43-L47, L57-L59 |
| I34-A005 | A/TEST | The reasoning fixture, pass-rate denominator, number of seeds, and treatment of ties with FreeText are unspecified. | I-34:L47-L50, L89-L92 |
| I34-A006 | A/DEP | Exact dependency version policy is unclear: the amendment names v1.7.6 but also commands edit-time reverification rather than trusting that snapshot. | I-34:L117-L123, L141-L147 |
| I34-C001 | C/SCOPE | The main body requires reasoning-heavy CRANE/final-only constraints, while the amendment says HD-3 bans constrained decoding on reasoning/coder/planner roles; role applicability must be reconciled before execution. | I-34:L14-L18, L47-L50, L128-L132 |

Category coverage: decoding capability/UI ✓; workflows/states ✓; grammar data/lifecycle ✓; flags/config ✓; integrations/events ✓; security/integrity/approval ✓ (no privacy/compliance policy); failure/refusal ✓; deterministic pinning/replay ✓ (no concurrency/idempotency/retry contract); observability/evidence ✓; tests/gates ✓; constraints/non-goals ✓; dependencies/status/deferred ✓.

### I-35 — temporal tool-sequence enforcement

Source: `docs/thefinalglance/I-plan/I-35_temporal_tool_sequence_enforcement.md` (108/108 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| I35-E001 | E/STATUS | I-35 is a strong-grounded Track-18 can’t-wreck BUILD, effort M, Wave 2, hard-dependent on I-1. | I-35:L1-L3 |
| I35-E002 | E/CAP | It adds the absent fourth enforcement dimension—ordering over tool-call history—on top of state masks, argv shape, and committed tool sets. | I-35:L5-L9, L32-L39, L101-L106 |
| I35-E003 | E/DATA | A `SequenceRule` consists of tool-class trigger, deterministic predicate over the current session’s ordered SSB/journal history, and an enforcement action. | I-35:L11-L20, L41-L45, L64-L70 |
| I35-E004 | E/TEST | One supported predicate requires a `test_run` PASS since the most recent `src_edit`. | I-35:L13-L17 |
| I35-E005 | E/SEC | One supported predicate forbids a write-class call after `untrusted_ingest` until an intervening `taint_check` PASS exists. | I-35:L13-L17, L41-L48 |
| I35-E006 | E/SEC | One supported predicate requires an in-session `auditor_PASS` before `git_push`. | I-35:L13-L17, L41-L48 |
| I35-E007 | E/FLOW | The rule model defines three enforcement actions: halt/refuse, require-confirmation advisory, or substitute with a safer tool class. | I-35:L18-L20 |
| I35-E008 | E/FAIL | Halt refuses the call and emits `ToolSequenceVerdict(OrderViolation)`; Tier-1 halt fixtures also emit `HaltEnforced`. | I-35:L18-L20, L45-L48, L86-L89 |
| I35-E009 | E/SEC | `SequenceRuleSet` is TOML-deserialized and R8-frozen; its blake3 hash is carried in the gate-freeze manifest and the running session may not edit it. | I-35:L21-L23, L41-L50, L56-L62, L80-L83 |
| I35-E010 | E/DET | Replay re-evaluates the same concrete ordered row list and must derive identical sequence verdicts under GD-2. | I-35:L21-L27, L86-L89 |
| I35-E011 | E/SEC | The source claims CDA/token-level attack immunity because predicates consume orchestrator-owned action-history rows rather than generated token content. | I-35:L5-L9, L25-L27 |
| I35-E012 | E/DEFER | Pro2Guard-style probabilistic DTMC reachability is deferred until a trace corpus exists; deterministic sequence rules ship first. | I-35:L29-L30, L52-L54, L80-L83, L91-L99 |
| I35-E013 | E/STATUS | Per-state Phase-3A masks, E-30 argv gating, and F-3 committed tool sets are PLAN-asserted existing; temporal ordering was absent. | I-35:L32-L39, L71-L76, L91-L106 |
| I35-E014 | E/DEP | IS-11 taint/provenance rows and IS-14’s production tool catalog are existing predicate/dispatch inputs to compose, not rebuild. | I-35:L37-L39, L56-L62, L71-L76, L101-L106 |
| I35-E015 | E/DEP | I-1 Supervisor must exist before the rule gate can wire into production dispatch. | I-35:L3-L3, L39-L39, L71-L79, L97-L102 |
| I35-E016 | E/CAP | Tier 1 creates `crates/orchestrator/src/tool_sequence.rs::{SequenceRule,SequenceRuleSet,evaluate_sequence_rule}`. | I-35:L41-L44, L56-L59, L64-L70 |
| I35-E017 | E/CFG | Tier 1 creates frozen `rules/tool_sequence_core.toml` covering at least `git_push`, `file_write`, and `untrusted_ingest` classes. | I-35:L41-L49, L56-L61, L64-L70 |
| I35-E018 | E/EVENT | I-35 adds one `ToolSequenceVerdict` action type with `Allowed`, `OrderViolation`, and `HaltEnforced`, baseline+1 from the dated 221/532 registry. | I-35:L41-L48, L64-L70, L86-L99 |
| I35-E019 | E/FLOW | The sequence check executes at I-1 Supervisor dispatch **after** existing per-call gates and **before** tool execution. | I-35:L41-L47, L56-L59, L77-L79 |
| I35-E020 | E/TEST | Required fixtures cover a compliant sequence, push without auditor PASS, and write after untrusted ingest without taint-check PASS. | I-35:L41-L48, L86-L89 |
| I35-E021 | E/SURFACE | Sequence-verdict rows render in the existing gate/MonitorWindow panel through Svelte plus Tauri, with `cross_lang_diff.py` required; a reasoning decision document is also required. | I-35:L47-L50, L56-L62, L84-L89 |
| I35-E022 | E/DEFER | Rules spanning multiple sessions are deferred; Tier 1 is session-scoped. | I-35:L52-L55 |
| I35-E023 | E/INT | Production chain is `supervisor::dispatch_tool -> tool_sequence::evaluate_sequence_rule -> ToolSequenceVerdict`; test-only callers do not satisfy R7. | I-35:L77-L79 |
| I35-E024 | E/LIMIT | Production code forbids mutable self-edited rule sets, Tier-1 probabilistic reachability, `.unwrap()`/`.expect()`, and blake2/sha2. | I-35:L80-L83 |
| I35-E025 | E/OBS | Violation payloads must name the unsatisfied predicate; compliant flow emits `Allowed`, while both negative sequences emit `OrderViolation` plus `HaltEnforced`. | I-35:L86-L89 |
| I35-E026 | E/TEST | Editing `tool_sequence_core.toml` during a run must make `gate_freeze_check.py` exit nonzero and block the run. | I-35:L80-L81, L86-L89 |
| I35-E027 | E/EVIDENCE | AgentSpec/Agent-C supply the trigger/predicate/enforcement shape and Pro2Guard supplies deferred prior art; their arXiv IDs require web confirmation before headline use. | I-35:L91-L99 |
| I35-A001 | A/DATA | The TOML schema, predicate DSL/AST, history-query language, validation errors, and rule precedence are not defined. | I-35:L11-L23, L41-L45 |
| I35-A002 | A/FLOW | Require-confirmation and substitute are part of the rule model but have no Tier-1 event kinds, fixtures, production acceptance evidence, or exact continuation semantics. | I-35:L18-L20, L41-L48, L64-L70, L86-L89 |
| I35-A003 | A/CFG | Mapping concrete tools to `git_push`, `file_write`, `untrusted_ingest`, and “safer” substitute classes is not specified. | I-35:L13-L20, L41-L48 |
| I35-A004 | A/STATE | Session boundary, replay-history truncation, and behavior after checkpoint restore are not defined for sequence predicates. | I-35:L14-L17, L21-L27, L52-L55 |
| I35-A005 | A/SEC | “CDA-immune by construction” is an explicit design claim, but this plan defines no adversarial acceptance test proving row provenance cannot be forged through another tool surface. | I-35:L5-L9, L25-L27, L86-L89 |

Category coverage: capability/UI ✓; workflow/state ✓; rule data/config ✓; integrations/events ✓; security/integrity ✓ (no privacy/compliance policy); failure/substitution/confirmation ✓ at model level (two arms lack acceptance detail); replay determinism ✓ (no concurrency/idempotency/retry contract); observability/evidence ✓; tests/gates ✓; constraints/non-goals ✓; dependencies/status/deferred ✓.

### I-40 — trust-layer wrapper and open RRA specification

Source: `docs/thefinalglance/I-plan/I-40_trust_layer_wrapper_rra_spec.md` (123/123 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| I40-E001 | E/STATUS | I-40 is a strong-grounded Track-18 wedge BUILD, effort L, Wave 5; full-form expansion is required before EXECUTING. | I-40:L1-L3, L11-L11 |
| I40-E002 | E/CAP | It exposes ELAI’s existing taint/write-gate/RRA/attestation trust layer around non-ELAI external agents as an adoption wedge. | I-40:L5-L9, L13-L35 |
| I40-E003 | E/SURFACE | Tier 1 publishes a versioned open format at `docs/spec/rra-spec-v1.md` and gives external implementers a conformance target. | I-40:L16-L20, L50-L54, L64-L65, L75-L80 |
| I40-E004 | E/DATA | The spec includes I-18 `chain_prev_hash`, taint-flow payload schema, write-gate record schema, and the RRA bundle envelope. | I-40:L16-L20 |
| I40-E005 | E/SURFACE | `elai rra verify <path>` checks a bundle against the published spec and must reject tampering with nonzero exit. | I-40:L16-L20, L50-L54, L66-L67, L89-L103 |
| I40-E006 | E/SURFACE | `elai wrap -- <agent cmd>` is a minimal MCP/stdio proxy that intercepts an external agent’s tool traffic. | I-40:L22-L23, L50-L57, L64-L67, L89-L91 |
| I40-E007 | E/SEC | Any untrusted ingest in wrapped traffic goes through existing G-1 taint tagging. | I-40:L22-L27, L68-L70, L81-L90, L100-L103 |
| I40-E008 | E/SEC | Destructive wrapped tool classes go through existing F-1/F-9/G-3 write gates. | I-40:L22-L27, L68-L71, L81-L90, L100-L103 |
| I40-E009 | E/OBS | Each wrapped tool call emits through the existing `rra.rs` surface, yielding a wrapped-run RRA. | I-40:L22-L27, L41-L47, L68-L72, L81-L90 |
| I40-E010 | E/PRIV | G-9 redaction is a mandatory pre-export step; an exported bundle containing raw secret bytes is a P0/stub-ban violation. | I-40:L5-L9, L22-L27, L37-L38, L43-L47, L68-L72, L93-L103 |
| I40-E011 | E/LIMIT | The wrapper adds no new trust primitives or forked gate logic; it composes the shipped substrate. | I-40:L29-L32, L93-L97, L115-L121 |
| I40-E012 | E/TEST | Tier 1 proves the wrapper with exactly one controllable external-agent fixture rather than a real-server matrix. | I-40:L29-L32, L50-L57, L93-L97 |
| I40-E013 | E/STATE | The published strategy records wrapper/open standard as the on-ramp and deep ELAI orchestration as the premium destination. | I-40:L34-L35, L50-L53 |
| I40-E014 | E/STATUS | `rra.rs::build_rra`/`verify_rra` and attestation are cited as shipped; the wrapper must extend/compose them, never fork them. | I-40:L40-L43, L68-L68, L81-L83, L105-L107, L115-L116 |
| I40-E015 | E/STATUS | `capability_flow.rs::taint_text` and `safety/src/provenance.rs` are cited as shipped taint substrates. | I-40:L41-L43, L68-L70, L81-L84, L105-L108 |
| I40-E016 | E/STATUS | F-1/F-9/G-3 gates, G-9 redaction, B-37 adapters, and E-24 MCP hardening are PLAN-asserted seams that require execution-time re-grep. | I-40:L43-L47, L68-L72, L81-L88, L105-L121 |
| I40-E017 | E/DEP | I-18’s per-row chain is required for the documented chained format; the source says I-18 must ship first or in Wave-5 parallel. | I-40:L16-L18, L48-L48, L81-L88, L105-L121 |
| I40-E018 | E/DATA | Planned implementation files are `crates/cli/src/wrap_cmd.rs`, `crates/cli/src/rra_verify_cmd.rs`, the spec document, and `workflow/reasoning/I-40-decisions.md`. | I-40:L50-L57, L64-L67, L75-L80 |
| I40-E019 | E/EVENT | I-40 adds no SSB action type and reuses existing `taint_flow` and `run_artifact_emitted` rows, leaving the dated 221/532 baseline unchanged. | I-40:L50-L56, L75-L80, L89-L90, L105-L113 |
| I40-E020 | E/SURFACE | Wrapped-run rows reuse the existing RRA viewer; no new Svelte component is planned, and `cross_lang_diff.py` applies only if execution-time re-grep confirms a live Svelte counterpart. | I-40:L54-L57, L73-L73, L98-L103 |
| I40-E021 | E/DEFER | A matrix of multiple real MCP server implementations is Tier 2 and user-approval/network-dependency gated. | I-40:L29-L32, L59-L61 |
| I40-E022 | E/DEFER | Commercial packaging/distribution is deferred as a post-validation business decision. | I-40:L59-L62 |
| I40-E023 | E/DEFER | Sigstore/Rekor publication for external-agent RRAs waits for I-18. | I-40:L59-L63 |
| I40-E024 | E/INT | Production chains are wrap CLI -> taint -> write gate -> RRA emit -> redaction and verify CLI -> conformance checker; test-only callers do not satisfy R7. | I-40:L89-L92 |
| I40-E025 | E/LIMIT | Production code forbids unredacted export, new wrapper-native trust primitives, expansion beyond one Tier-1 fixture, `.unwrap()`/`.expect()`, and blake2/sha2. | I-40:L93-L97 |
| I40-E026 | E/TEST | The fixture performs untrusted ingest plus a seeded malicious write; expected rows prove taint propagation and a write-gate BLOCK outside ELAI’s native generator loop. | I-40:L100-L103 |
| I40-E027 | E/TEST | A wrapper-produced bundle must pass `elai rra verify`, a tampered bundle must fail, and no planted raw secret bytes may appear in export. | I-40:L100-L103 |
| I40-E028 | E/EVIDENCE | Competitive/trust reviews are cited as converging on wrapper-as-wedge; IC-16 says the trust spine is substantially wired. | I-40:L105-L113 |
| I40-A001 | A/DEP | “I-18 must ship first or in Wave-5 parallel” does not define how I-40 can document/verify the chained format before I-18’s record layout is final. | I-40:L16-L18, L48-L48, L62-L63, L88-L88, L121-L121 |
| I40-A002 | A/DATA | The plan lists required spec sections but does not define normative field types, canonical serialization, version negotiation, compatibility policy, or conformance error taxonomy. | I-40:L16-L20, L50-L54 |
| I40-A003 | A/SEC | MCP/stdio interception completeness, process/platform handling, unsupported tool channels, and bypass behavior are not specified. | I-40:L22-L32, L50-L54 |
| I40-A004 | A/SURFACE | The RRA viewer’s live Svelte pairing is explicitly unknown until execution-time re-grep. | I-40:L73-L73, L98-L99 |
| I40-A005 | A/PRIV | G-9 owns redaction, but I-40 supplies no redaction failure policy, secret canary schema, or rule for redacting command output before versus after row sealing. | I-40:L22-L27, L37-L38, L68-L72, L100-L103 |

Category coverage: open spec/CLI/wrapper/UI ✓; workflows/states ✓; schemas/data/lifecycle ✓ at section level (normative schema details unresolved); config/defaults △; integrations/events ✓; security/privacy/approval ✓ (no explicit compliance certification); failures/fail-closed ✓; determinism/integrity via RRA ✓ (no idempotency/concurrency/retry contract); observability/evidence ✓; tests/gates ✓; constraints/non-goals ✓; dependencies/status/deferred ✓.

### I-42 — critique-only operating profile

Source: `docs/thefinalglance/I-plan/I-42_critique_only_profile.md` (118/118 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| I42-E001 | E/STATUS | I-42 is a moderate-grounded Track-19 human-as-verifier BUILD, effort M, Wave 3, dependent on I-41 and bound to the §6 ban-LLM-judge rule. | I-42:L1-L3 |
| I42-E002 | E/CFG | `RunProfile` has `Orchestrate` and `CritiqueOnly`; `Orchestrate` remains the default and the profile is selected at dispatch. | I-42:L12-L15, L41-L46, L64-L68 |
| I42-E003 | E/STATE | Both profiles use one orchestration engine and one identical off-agent verification spine; profile changes only who writes and whether executors/fixers spawn. | I-42:L4-L10, L20-L25, L82-L86, L100-L116 |
| I42-E004 | E/FLOW | In `CritiqueOnly`, the human operator is the sole code writer; spawned agents are for critique rather than implementation. | I-42:L4-L10, L12-L19 |
| I42-E005 | E/SEC | Executor/fixer spawn is structurally gated off so no spawned executor/fixer touches disk. | I-42:L4-L10, L12-L16, L41-L48, L64-L71 |
| I42-E006 | E/SEC | The generator may emit no disk-mutating Directive in `CritiqueOnly`; this is an execution/spawn gate, not a prompt request. | I-42:L12-L16, L20-L25, L36-L38, L41-L48, L82-L84 |
| I42-E007 | E/FLOW | I-41 `LensSet` runs at design time and surfaces findings for operator revision. | I-42:L12-L18, L30-L30, L41-L48, L72-L74 |
| I42-E008 | E/FLOW | The shipped auditor plus F-1/F-4/G-2/B-41 off-agent gate spine run at ship time on the operator’s diff. | I-42:L16-L19, L31-L32, L41-L48, L72-L75 |
| I42-E009 | E/STATE | The profile loop is Design critique -> operator revision/implementation -> ship-time auditor/spine -> human-judged iteration to convergence. | I-42:L16-L19 |
| I42-E010 | E/EVIDENCE | The source positions this profile for high-stakes, regulated, or low-trust-in-autonomy work, carrying the source-design claim that agent-spawned writes caused more harm than good. | I-42:L20-L23, L100-L106 |
| I42-E011 | E/OBS | Core invariant: the SSB set of `file_write` rows attributable to spawned executor/fixer agents is empty in `CritiqueOnly`. | I-42:L20-L23, L41-L49, L64-L71, L91-L98 |
| I42-E012 | E/LIMIT | Prompt-only “do not write” and a separate engine fork are explicitly declined. | I-42:L24-L25, L82-L87 |
| I42-E013 | E/STATUS | The orchestrate executor/fixer dispatch, auditor/spine, and B-22 mode-plumbing pattern are PLAN-asserted shipped and require execution-time re-grep. | I-42:L27-L38, L55-L62, L100-L108 |
| I42-E014 | E/CFG | `RunProfile` is orthogonal to B-22 `GateMode`; it must not overload the gate’s autonomous/interactive choice. | I-42:L33-L35, L72-L77, L82-L87, L110-L114 |
| I42-E015 | E/RECOVERY | If I-1 is shipped, Supervisor rejects executor-authored side-effect Directives; otherwise Tier 1 gates directly at the spawn site, with the exact seam chosen at edit time. | I-42:L36-L38, L43-L48, L72-L77 |
| I42-E016 | E/STATUS | No `RunProfile` concept existed in the cited inventory. | I-42:L39-L39 |
| I42-E017 | E/CAP | Tier 1 creates `crates/orchestrator/src/run_profile.rs::{RunProfile,from_cli_args}` and the executor/fixer spawn gate. | I-42:L41-L48, L55-L60, L64-L71 |
| I42-E018 | E/SURFACE | CLI selection is `elai run --profile critique-only`; ordinary orchestrate dispatch also checks `RunProfile` before every executor spawn. | I-42:L41-L46, L55-L58, L78-L81 |
| I42-E019 | E/TEST | Tier 1 includes `crates/contracts/tests/i42_critique_only_no_executor_writes_contract.rs` and `workflow/reasoning/I-42-decisions.md`. | I-42:L47-L49, L55-L60, L64-L71 |
| I42-E020 | E/DEFER | A Settings/dashboard profile switcher is deferred; CLI ships first. | I-42:L50-L52 |
| I42-E021 | E/DEFER | Per-profile review-time versus generation-time telemetry waits for the G-5 cockpit. | I-42:L50-L53 |
| I42-E022 | E/DEFER | Future review-only/pair profile variants are reserved but out of this session. | I-42:L50-L53 |
| I42-E023 | E/INT | CLI args/run dispatch thread the profile into the PLAN-asserted `task_runner.rs` executor/fixer spawn site. | I-42:L55-L59, L72-L80 |
| I42-E024 | E/SURFACE | The active profile reuses B-22’s existing StatusBar mode-badge cluster; no new component is created. | I-42:L61-L62, L88-L90 |
| I42-E025 | E/EVENT | I-42 adds no action type; profile is surfaced through existing `phase_transition` rows and the key proof is absence of executor-attributed writes. | I-42:L64-L71, L100-L108 |
| I42-E026 | E/INT | Production reachability is both critique-only CLI dispatch and normal orchestrate dispatch before spawn; tests-only callers do not satisfy R7. | I-42:L78-L81 |
| I42-E027 | E/LIMIT | Production code prohibits prompt-only gating, an engine fork, lens output on a verdict path, `GateMode` overloading, `.unwrap()`/`.expect()`, and blake2/sha2. | I-42:L82-L87 |
| I42-E028 | E/OBS | Expected critique-only evidence is I-41 design-time finding rows, ship-time auditor/spine verdicts, and zero executor/fixer-attributed file writes. | I-42:L91-L96 |
| I42-E029 | E/TEST | The same fixture under default `Orchestrate` must produce executor file writes, proving the profile gate is not a no-op. | I-42:L95-L98 |
| I42-E030 | E/DET | Off-agent spine verdict bytes must be identical across both profiles. | I-42:L95-L98 |
| I42-E031 | E/INT | I-40 is the external-agent trust-wrapper sibling; I-42 is the internal operator-writes profile and must not restate I-40. | I-42:L110-L116 |
| I42-I001 | I/DATA | Proving an empty executor/fixer write set implicitly requires existing `file_write` rows to carry stable actor-role attribution, although I-42 does not define that schema. | I-42:L20-L23, L47-L49, L64-L71, L91-L98 |
| I42-A001 | A/SEC | The prose says all spawned agents are critique-only, but the structural invariant/gate is specifically executor/fixer-attributed writes; permissions for other spawned critic/auditor agents are not separately specified. | I-42:L4-L10, L12-L23, L41-L49 |
| I42-A002 | A/DATA | The `phase_transition` field/discriminator that encodes `RunProfile` is not identified. | I-42:L64-L71, L100-L108 |
| I42-A003 | A/DET | The fixture must produce executor writes only in Orchestrate yet identical off-agent verdict bytes across profiles; the plan does not define how resulting diffs/artifacts are held constant for that comparison. | I-42:L91-L98 |
| I42-A004 | A/INT | Exact enforcement location remains edit-time conditional between I-1 Supervisor rejection and direct spawn-site refusal. | I-42:L36-L38, L43-L48, L72-L77 |
| I42-A005 | A/FLOW | The mechanics by which an operator’s external edits are detected/ingested between design and ship phases are not specified. | I-42:L16-L19, L91-L98 |

Category coverage: profile/CLI/UI ✓; workflows/states ✓; data/lifecycle △ (profile event encoding unresolved); defaults/config ✓; integrations/events ✓; write permission/security ✓ and regulated-use rationale ✓ (no formal privacy/compliance control); failures/refusal ✓; deterministic equivalence ✓ (no idempotency/concurrency/retry contract); observability/evidence ✓; tests/gates ✓; constraints/non-goals ✓; dependencies/status/deferred ✓.

### I-45 — objective-hash sealed receipts

Source: `docs/thefinalglance/I-plan/I-45_objective_hash_sealed_receipts.md` (72/72 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| I45-E001 | E/STATUS | I-45 is a Track-20 can’t-lie EXTENSION, effort S-M, Wave 1, strong-grounded on ELAI and moderate-grounded on the roach/SL-11 semantics, bound to ID-4. | I-45:L1-L4 |
| I45-E002 | E/CAP | It binds a sealed verifier receipt to the objective by hashing canonical objective bytes, preventing post-PASS goal/acceptance mutation from retaining a fresh-looking receipt. | I-45:L6-L11 |
| I45-E003 | E/DEP | Execution must read the cited original roach-pi extension source under `docs/.source/roach-pi/`; the technique is stolen, not its code. | I-45:L13-L16, L62-L64 |
| I45-E004 | E/DATA | `canonical_objective_bytes()` serializes orchestrator-owned TaskSpec, baby-step acceptance text, and future I-37 EARS records. | I-45:L17-L20, L25-L31, L43-L47 |
| I45-E005 | E/DET | Canonicalization requires stable field order, normalized whitespace, and blake3. | I-45:L17-L20 |
| I45-E006 | E/DATA | Verdict sealing stamps `objective_hash` into both the RRA seal and attestation. | I-45:L17-L21, L33-L40, L43-L47 |
| I45-E007 | E/FLOW | Every receipt read path rehashes the current objective, explicitly including `elai verify-attestation`, replay, and plan-status consumers. | I-45:L19-L21, L38-L49 |
| I45-E008 | E/FAIL | Current hash mismatch produces `ReceiptStale` and hard-fails; warn-and-continue is forbidden. | I-45:L19-L21, L43-L60 |
| I45-E009 | E/LIMIT | I-45 adds no verifier, gate, objective-versioning UI, or historical-receipt migration. | I-45:L22-L23 |
| I45-E010 | E/COMP | Historical receipts without the field are explicitly marked `LEGACY`; they may never be silently reported as fresh. | I-45:L22-L23, L50-L52 |
| I45-E011 | E/STATUS | `rra.rs`, `attestation.rs`, and `rra_cmd.rs` are cited as shipped, correcting a stale absence note; I-45 extends them in place. | I-45:L25-L29, L38-L49 |
| I45-E012 | E/DEP | F-4 `stdout_hash` is the sibling artifact-hash precedent; I-18 `chain_prev_hash` shares the record and requires field-layout coordination if it lands first. | I-45:L9-L11, L25-L31, L43-L47, L66-L70 |
| I45-E013 | E/DEP | B-40 TaskSpec and baby-step blocks are current objective sources; I-37 EARS is a forward composition source, not a prerequisite. | I-45:L30-L31, L43-L47, L66-L68 |
| I45-E014 | E/CAP | Tier 1 is serializer, seal-time stamp, read-time staleness check, and an in-process deterministic contract test. | I-45:L33-L35 |
| I45-E015 | E/DEFER | Per-requirement hashing waits for I-37, and cross-session objective lineage is deferred. | I-45:L33-L36 |
| I45-E016 | E/SURFACE | The existing RRA viewer gets a stale-receipt badge; no new component is created. | I-45:L38-L41 |
| I45-E017 | E/EVENT | I-45 adds one `ObjectiveSealed` action type with `Sealed`, `MutationDetected`, and `ReceiptStale`, baseline+1 from the dated 221/532 registry. | I-45:L43-L45, L53-L60, L66-L70 |
| I45-E018 | E/INT | Production reachability is `elai run --emit-rra` at the RRA seal site and `elai verify-attestation` at the read site; contracts-only callers do not satisfy R7. | I-45:L48-L49 |
| I45-E019 | E/SEC | The hash is never model-settable; serialization reads orchestrator-owned state only. | I-45:L50-L52 |
| I45-E020 | E/LIMIT | Production paths forbid blake2/sha2, model-supplied hashes, warn-and-continue, silent legacy freshness, and `.unwrap()`/`.expect()`. | I-45:L50-L52 |
| I45-E021 | E/TEST | Acceptance seals an RRA, mutates fixture objective text, and verifies the bundle. | I-45:L53-L55 |
| I45-E022 | E/OBS | Seal emits `ObjectiveSealed(Sealed)`; post-mutation verify emits `ObjectiveSealed(MutationDetected)` plus `ReceiptStale` and names both hashes. | I-45:L53-L60 |
| I45-E023 | E/TEST | The unmodified objective is the negative/control arm and must verify PASS; any verifier that still passes after mutation is the broken arm and fails R2. | I-45:L58-L60 |
| I45-E024 | E/DEP | I-39 plan status is an explicit downstream consumer that must recheck hashes. | I-45:L66-L68 |
| I45-E025 | E/SEC | The staleness check joins the frozen off-agent gate set under R1/R2/R7/R8. | I-45:L62-L64 |
| I45-I001 | I/DET | All seal/read consumers implicitly must share the same canonical serializer/version; otherwise unchanged objectives could disagree across RRA, attestation, replay, and plan status. | I-45:L17-L21, L38-L49, L66-L68 |
| I45-A001 | A/SCOPE | Title/premise say stamp every PASS, while the build paragraph says every sealed verdict; whether non-PASS seals also require `objective_hash` is unresolved. | I-45:L1-L1, L6-L11, L17-L21 |
| I45-A002 | A/DATA | Canonical field inclusion, TaskSpec/baby-step/EARS precedence, Unicode/newline normalization, and missing-source behavior are not defined. | I-45:L17-L20, L30-L31 |
| I45-A003 | A/COMP | “Verify as legacy” does not say whether LEGACY returns success, a distinct non-fresh verdict, or a policy-controlled failure. | I-45:L22-L23, L50-L52 |
| I45-A004 | A/EVENT | Acceptance says `ObjectiveSealed(MutationDetected) + ReceiptStale`, while the event list makes `ReceiptStale` an `ObjectiveSealed` kind; error versus event identity is ambiguous. | I-45:L43-L45, L53-L57, L66-L70 |
| I45-A005 | A/STATE | Detection/versioning for receipts lacking `objective_hash` is not specified beyond the LEGACY marker. | I-45:L22-L23 |

Category coverage: capability/CLI/UI ✓; workflows/states ✓; receipt schema/data/lifecycle ✓; config/defaults △; integrations/events ✓; integrity/security/compatibility ✓ (no privacy/compliance rule); failure/fail-closed ✓; determinism ✓ (no idempotency/concurrency/retry contract); observability/evidence ✓; tests/gates ✓; constraints/non-goals ✓; dependencies/status/deferred ✓.

## Cross-source interactions and lifecycle

| Interaction ID | Class | Interaction, ordering, or shared invariant | Evidence atoms |
|---|---|---|---|
| IX-001 | E | Track-12 safe parallelism is layered: I-2 isolates branch source state while I-3 makes the shared audit/checkpoint substrate durable under later search load. | I2-E002–I2-E012; I3-E002–I3-E007 |
| IX-002 | E | I-4 parallel search is directly blocked on I-2 workspace isolation and later consumes I-3’s high-rate append path. | I2-E029; I3-E018, I3-E026 |
| IX-003 | E | I-1 Supervisor is both I-2’s spawn/escape enforcement plane and I-35’s sequence-gate plane; I-3 precedes it so Supervisor rows land durably. | I2-E004, I2-E013; I3-E026; I35-E015, I35-E019, I35-E023 |
| IX-004 | E | I-2 conflict recovery composes BSP checkpoint recovery and H-49 trunk restore; it owns neither checkpoint nor rollback implementation. | I2-E008–I2-E010, I2-E019, I2-E022–I2-E023 |
| IX-005 | E | I-2 backend state is surfaced by extending C-23 BranchTree, while the underlying sandbox/pool implementation composes C-11/E-15. | I2-E015, I2-E021 |
| IX-006 | E | I-2’s deterministic sequence merge is the Tier-1 policy; richer cross-branch CRDT semantics are explicitly handed to I-29. | I2-E008–I2-E012, I2-E028 |
| IX-007 | E | I-3 intentionally produces a TOML rather than UI; I-27 is named as the later cockpit consumer while separately sourcing compute telemetry from I-4. | I3-E006–I3-E007, I3-E020, I3-I001; I27-E011, I27-E014 |
| IX-008 | E | I-12 is a gated extension chain: receipt count/lift -> C-28 mutation validation -> existing Phase-3F lifecycle -> RulesDrawer/export, with F-5 demotion and IS-16 freeze controls. | I12-E002–I12-E016, I12-I001 |
| IX-009 | E | I-12 cross-corpus transfer is blocked on I-38’s baseline; I-27 already consumes I-38’s paired-seed/bootstrap floor for a different measurement. | I12-E024; I27-E005, I27-E011 |
| IX-010 | E | I-21’s execution path composes H-37 approval, I-1 context isolation, I-17 deterministic checks, H-38 stop, and D-13 counterexample escalation in that order. | I21-E007–I21-E016, I21-E019–I21-E023 |
| IX-011 | E | I-21’s bounded retry is structurally distinct from I-3 durability recovery and I-2 rollback: it reinjects a failing verifier trace into generation rather than repeating I/O or restoring a branch. | I21-E012–I21-E016; I2-E010; I3-E005 |
| IX-012 | E | I-27 imports pass/cost definitions from E-49, runner shape from E-9, I-4 node telemetry, I-38 statistics, and H-9 display; none may be redefined locally. | I27-E011, I27-E024 |
| IX-013 | E | I-34’s runtime chain deliberately separates XGrammar token masking, Rust/provider CRANE control, mandatory post-parse validation, and D-16 refusal. | I34-E004–I34-E006, I34-E010–I34-E013, I34-E025, I34-E030–I34-E031 |
| IX-014 | E | I-34’s amendment aligns grammar spec/pin hashing with the RRA stack so replay can detect plan/execution grammar drift. | I34-E027–I34-E030, I34-E033 |
| IX-015 | E | C-37 consumes I-34’s CRANE runtime; the Python-sidecar CRANE mask work and cross-language pack remain separately deferred. | I34-E006, I34-E018–I34-E019 |
| IX-016 | E | I-35 is the temporal dimension layered after Phase-3A masks, E-30 argv checks, and F-3 committed sets, immediately before Supervisor execution. | I35-E002, I35-E013, I35-E019 |
| IX-017 | E | I-35 consumes IS-11 taint/provenance rows to make untrusted-ingest ordering enforceable at action history rather than token text. | I35-E005, I35-E011, I35-E014 |
| IX-018 | E | I-40’s spec and conformance checker depend on I-18’s chain format, while external Rekor publication is explicitly postponed until I-18 ships. | I40-E004–I40-E005, I40-E017, I40-E023, I40-A001 |
| IX-019 | E | I-40’s wrapped run must invoke G-9 redaction after trust processing and before bundle export; redaction is the publication/privacy precondition, not an optional display filter. | I40-E007–I40-E010, I40-E025–I40-E027 |
| IX-020 | E | I-42 consumes I-41 only for advisory design critique and preserves the shipped deterministic spine as the ship-time authority; lenses never become a DONE verdict. | I42-E001, I42-E003, I42-E007–I42-E009, I42-E027–I42-E030 |
| IX-021 | E | I-42 explicitly pairs with I-40 without overlap: I-42 governs internal operator-written runs; I-40 wraps external-agent tool traffic. | I42-E031; I40-E002, I40-E006 |
| IX-022 | E | I-45 and I-18 modify the same RRA/attestation record: objective hash and previous-row chain layout require coordination rather than parallel independent schemas. | I45-E006, I45-E012; I40-E004, I40-E017 |
| IX-023 | E | I-45’s objective source expands forward to I-37 EARS, and I-39 is a read-path consumer required to rerun the staleness check. | I45-E004, I45-E013, I45-E015, I45-E024 |
| IX-024 | I | Once I-45’s `objective_hash` lands, I-40’s versioned open RRA spec/conformance checker will need a coordinated version/schema decision even though neither source explicitly assigns that integration. | I40-E003–I40-E005, I40-A002; I45-E006–I45-E008 |
| IX-025 | E | Seven sources add baseline-relative action types—`BranchWorkspaceEvent`, `PlaybookEvent`, `SprintContractEvent`, `ConvergenceCurveMeasured`, `GrammarGateEvent`, `ToolSequenceVerdict`, and `ObjectiveSealed`; all require edit-time registry reread rather than independently assuming the dated 221/532 baseline. | I2-E014; I12-E011; I21-E023; I27-E009; I34-E015; I35-E018; I45-E017 |
| IX-026 | E | I-3, I-40, and I-42 explicitly add no action type and reuse existing row families; their acceptance evidence depends on existing rows and, for I-42, meaningful absence. | I3-E017, I3-E024; I40-E019; I42-E011, I42-E025, I42-I001 |
| IX-027 | E | Frozen/off-agent controls recur at different layers: I-12 freezes evaluator independence, I-27 freezes the harness, I-34 freezes grammar pins, I-35 freezes rule files, and I-45 joins the frozen receipt gate set. | I12-E010; I27-E012; I34-E009; I35-E009, I35-E026; I45-E025 |
| IX-028 | E | User/new-authority gates remain isolated: I-2 cloud adapter, I-12 auto-update, I-34 new Cargo dependency, and I-40 real MCP matrix each require explicit approval rather than silently entering Tier 1. | I2-E007; I12-E025; I34-E027, I34-E035; I40-E021 |
| IX-029 | E | All new hashing in this chunk uses blake3; I-2/I-3/I-12/I-34/I-35/I-40/I-42/I-45 explicitly ban blake2/sha2, and I-27 does so for its production bench path. | I2-E027; I3-E022; I12-E023; I27-E023; I34-E022; I35-E024; I40-E025; I42-E027; I45-E005, I45-E020 |
| IX-030 | I | The seven simultaneous action-family additions plus I-18/I-45 shared RRA fields make registry/schema edits a concurrency hotspot; serialization and golden-lock conflicts must be resolved from freshly reread live state. | IX-022, IX-025; I3-E011–I3-E012, I3-E017 |

## Explicit ambiguity and conflict register

| Register ID | Kind | Unresolved issue or required correction | Atom references |
|---|---|---|---|
| AC-001 | conflict | I-2’s opening platform description sounds immediate, but revised scope controls: worktrees are Tier 1 and Linux overlay/reflink is Tier 2. | I2-C001 |
| AC-002 | ambiguity | I-2 does not fully define “last-seq wins” for non-overlapping patches or canonical patch application. | I2-A001 |
| AC-003 | ambiguity/security | I-2’s absolute-path escape rule omits symlink/junction/canonicalization cases and its conflict badge is not mapped to UI. | I2-A002, I2-A004 |
| AC-004 | dependency | I-2 requires rollback while its H-49 restore owner is only PLAN-asserted planned; pre-H-49 fallback is missing. | I2-A003 |
| AC-005 | qualification | I-3 addresses a concurrency-motivated write path but remains single-writer in Tier 1; multi-writer behavior is deferred. | I3-C001 |
| AC-006 | ambiguity/config | I-3 leaves pragma values, remaining checkpoint sites, benchmark workload, and acceptable performance largely unspecified. | I3-A001–I3-A003 |
| AC-007 | ambiguity/recovery | I-3 cites POSIX rename without defining non-POSIX replacement, directory fsync, cleanup, or rename retry. | I3-A004 |
| AC-008 | ambiguity/measurement | I-12 does not fix `k`, schedule interval, or statistically valid lift method. | I12-A001–I12-A003 |
| AC-009 | ambiguity/data | I-12 names three export formats without specifying their block schemas or collision/update semantics. | I12-A004 |
| AC-010 | ambiguity/schema | I-21 uses both `bounded_iteration_count` and `iteration_count`, and its deterministic-criterion reference grammar is undefined. | I21-A001–I21-A002 |
| AC-011 | ambiguity/dependency | I-21 permits an unspecified temporary I-1 isolation shim and leaves replace-versus-supplement outer retry unresolved. | I21-A003–I21-A004 |
| AC-012 | ambiguity/state | I-21 does not fully define the final third-failure event/stop ordering. | I21-A005 |
| AC-013 | ambiguity/measurement | I-27 leaves compute stepping, bootstrap confidence, knee logic, cost normalization, and report-date determinism unresolved. | I27-A001–I27-A004 |
| AC-014 | conflict/scope | I-34’s reasoning-heavy CRANE requirement must be reconciled with the amendment’s HD-3 ban on constrained decoding for reasoning/coder/planner roles. | I34-C001 |
| AC-015 | ambiguity/implementation | I-34 leaves module ownership, commit-point transform, manifest schema, untrusted-source test, and fixture statistics unresolved. | I34-A001–I34-A005 |
| AC-016 | dependency | I-34’s named llguidance version is only a dated snapshot; user approval and current anti-slopsquat verification control. | I34-A006, I34-E035–I34-E036 |
| AC-017 | ambiguity/workflow | I-35 defines confirmation/substitution but supplies Tier-1 evidence only for allow/halt; its TOML DSL, class registry, and session-history semantics remain open. | I35-A001–I35-A004 |
| AC-018 | evidence | I-35’s absolute CDA-immunity claim has no corresponding adversarial provenance-forgery test in this plan. | I35-A005 |
| AC-019 | dependency/schema | I-40 documents I-18’s chain before its layout is final and omits normative serialization/version compatibility details. | I40-A001–I40-A002 |
| AC-020 | ambiguity/security/privacy | I-40 does not define proxy bypass coverage or redaction-stage failure semantics; its viewer pairing is also unverified. | I40-A003–I40-A005 |
| AC-021 | ambiguity/permissions | I-42 narrows its executable invariant to executor/fixer writes although prose says all spawned agents only critique; actor attribution and exact gate site are assumed/unresolved. | I42-I001, I42-A001–I42-A002, I42-A004 |
| AC-022 | ambiguity/determinism | I-42 requires Orchestrate-only writes and identical verdict bytes but does not specify an artifact-equivalent fixture; operator edit ingestion is also undefined. | I42-A003, I42-A005 |
| AC-023 | ambiguity/scope | I-45 alternates between every PASS and every sealed verdict, and its event/error identity plus LEGACY success semantics are unresolved. | I45-A001, I45-A003–I45-A005 |
| AC-024 | ambiguity/determinism | I-45 requires canonical bytes across multiple sources/consumers but omits canonical field/Unicode/precedence/version rules. | I45-I001, I45-A002 |

## Summary additions and corrections

Orientation comparison target: `docs/thefinalglance/feature-discovery/09-I-SUMMARY_INDEX.discovery.md`. “ADD” means an explicit source behavior omitted or materially compressed by the summary; “CORRECT/QUALIFY/REMOVE” means the summary’s wording changes scope or certainty. These are relational records, not additional feature atoms.

### I-2 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-I2-001 | **CORRECT** summary implementation scope: overlayfs/btrfs is not a Tier-1 implementation; the worktree pool is the default and Linux CoW is deferred. | 09-I:L123-L134 | I2-E005–I2-E006, I2-C001 |
| SD-I2-002 | **CORRECT** “adjacent-code conflict resolves deterministically”: an overlapping conflict emits `ConflictEscalated`, halts, and rolls back; only non-overlap follows the stated sequence policy. | 09-I:L140-L141 | I2-E009–I2-E012, I2-A001 |
| SD-I2-003 | **ADD** complete event family, pre-filesystem escape guarantee, rollback chain, missing-row hard failure, and unresolved conflict-event UI rendering. | 09-I:L136-L141 | I2-E010, I2-E013–I2-E018, I2-A004 |
| SD-I2-004 | **ADD** exact `BranchWorkspace` methods, production reachability, test-only exclusion, no-reimplementation/stub bans, and PLAN-asserted composition status. | 09-I:L123-L143 | I2-E003–I2-E004, I2-E019–I2-E027 |
| SD-I2-005 | **ADD/QUALIFY** cloud adapter is user-approval gated, CRDT merge is deferred to I-29, and H-49 readiness/path-escape semantics remain unresolved. | 09-I:L127-L143 | I2-E007, I2-E022, I2-E028–I2-E029, I2-A002–I2-A003 |
| SD-I2-006 | **ADD** the entire J-11 Arbor lifecycle amendment: current-trunk start, artifact filtering, preserved branch, separate eval pool, stale cleanup, snapshot hook, and Windows-safe pooling improvement. | 09-I:L119-L143 | I2-E030–I2-E037, I2-I001 |

### I-3 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-I3-001 | **ADD** WAL mode (omitted from the summary pragma list), explicit single-writer scope, `cargo check`, and no-action-type registry invariant. | 09-I:L149-L164 | I3-E003–I3-E004, I3-E015, I3-E017 |
| SD-I3-002 | **ADD** exact `ELAI_IO_URING_ENABLE` flag, allowlist/promotion gate, and separately deferred multi-writer fanout after I-4 contention measurement. | 09-I:L155-L158 | I3-E008, I3-E018 |
| SD-I3-003 | **ADD** seeded real-loop methodology, TOML-as-ID-5-baseline semantics, `insert_throughput_per_sec > 0`, and file-length+blake3 crash proof. | 09-I:L153-L162 | I3-E006–I3-E007, I3-E009, I3-E023–I3-E025 |
| SD-I3-004 | **ADD** declined storage-engine/async batching alternatives, H-44/I-1/I-4 composition boundaries, production reachability, and write-path bans. | 09-I:L145-L164 | I3-E010–I3-E014, I3-E019–I3-E022, I3-E026 |
| SD-I3-005 | **QUALIFY** summary status ambiguity with the source’s full unknowns: pragma values/sites, benchmark normalization, and cross-platform atomic-write details remain undefined. | 09-I:L163-L164 | I3-A001–I3-A004 |
| SD-I3-006 | **CORRECT/CLARIFY** concurrency framing: Tier 1 does not deliver concurrent multi-writer support despite the motivating language. | 09-I:L145-L164 | I3-E003, I3-E018, I3-C001 |

### I-12 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-I12-001 | **QUALIFY** summary “interleaved on/off injection”: the numbered source says cheap on/off injection on an offline tick but does not explicitly specify interleaving. | 09-I:L369-L371 | I12-E003, I12-A003 |
| SD-I12-002 | **ADD** full `PlaybookEvent` kinds, required validation-before-promotion order, automatic stale/regression demotion, and exporter-as-read lifecycle boundary. | 09-I:L366-L384 | I12-E005, I12-E008, I12-E011, I12-E018–I12-E020, I12-I001 |
| SD-I12-003 | **ADD** exact module/production entrypoints, configurable Layer-1A schedule, event-driven RulesDrawer fields, and decision artifact. | 09-I:L376-L383 | I12-E012–I12-E017, I12-E026 |
| SD-I12-004 | **ADD** ID-6/R8 evaluator-write-boundary rule, C-28/IS-16/E-8/F-5 composition seams, and DESIGN-number ban. | 09-I:L366-L386 | I12-E004, I12-E009–I12-E010, I12-E015, I12-E021–I12-E023, I12-E027 |
| SD-I12-005 | **ADD** cross-corpus/I-38 deferral and user-approved `ELAI_PLAYBOOK_AUTOUPDATE_ENABLE` PR-preview lane. | 09-I:L362-L386 | I12-E024–I12-E025 |
| SD-I12-006 | **EXPAND** the summary’s `k` ambiguity to include schedule, lift statistics, and export block/collision schema. | 09-I:L385-L386 | I12-A001–I12-A004 |

### I-21 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-I21-001 | **CORRECT** summary “vague goal is rejected”: the explicit rejection target is a vague/open-ended **acceptance criterion**, with permitted reference categories. | 09-I:L605-L624 | I21-E003–I21-E006 |
| SD-I21-002 | **ADD** exact contract fields including `out_of_scope_marker`, while flagging `bounded_iteration_count` versus `iteration_count` schema drift. | 09-I:L606-L610 | I21-E003–I21-E005, I21-A001 |
| SD-I21-003 | **QUALIFY** zero-shared-context wording: evaluator still receives task spec plus sealed contract, but no generator history/output/scratchpad; Supervisor pruning is the enforcement. | 09-I:L605-L619 | I21-E007–I21-E009 |
| SD-I21-004 | **ADD** `Proposed`/`Approved`/`Rejected` event arms, structured concrete objection fields, SSB loop counter, overflow BLOCK, and H-38 stop behavior. | 09-I:L611-L624 | I21-E010–I21-E011, I21-E016, I21-E023, I21-E027, I21-A005 |
| SD-I21-005 | **ADD** exact retry semantics: one trace per failed exit, gate+line payload, three total iterations/two reinjections, then D-13 counterexample. | 09-I:L612-L624 | I21-E012–I21-E016, I21-E031–I21-E033 |
| SD-I21-006 | **ADD** temporary I-1 isolation shim, unresolved replace/supplement outer retry, production reachability, and negative shared-context proof. | 09-I:L618-L624 | I21-E020–I21-E022, I21-E026, I21-E028–I21-E032, I21-A003–I21-A004 |
| SD-I21-007 | **ADD** multi-contract and auto-criteria deferrals, full-form requirement, external-figure caution, and unresolved criterion resolver. | 09-I:L601-L624 | I21-E001–I21-E002, I21-E024–I21-E025, I21-A002 |

### I-27 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-I27-001 | **ADD** mean curve points with bootstrap confidence intervals and at least five paired seeds under I-38’s statistical floor. | 09-I:L738-L751 | I27-E003–I27-E005, I27-E021 |
| SD-I27-002 | **ADD** exact beacon/report payload, artifact path, knee field, seed byte-recomputation, non-monotonic flags, and immutable-harness R8 rule. | 09-I:L745-L751 | I27-E007–I27-E009, I27-E012, I27-E019–I27-E020 |
| SD-I27-003 | **ADD** E-49/E-9/H-9/I-4/I-38 non-reimplementation dependencies, decision/contract artifacts, and production/off-agent reachability. | 09-I:L734-L751 | I27-E011, I27-E013–I27-E015, I27-E018, I27-E024 |
| SD-I27-004 | **ADD** live-refresh and >=3-config/RunPod overlay deferrals plus explicit paper/SWE-bench constraints. | 09-I:L734-L751 | I27-E010, I27-E016–I27-E017, I27-E023 |
| SD-I27-005 | **FLAG** compute-step, statistic, knee, cost, and date-normalization choices absent from the summary and source. | 09-I:L738-L751 | I27-A001–I27-A004 |

### I-34 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-I34-001 | **CORRECT** summary claim that implementation adds “the sidecar mask layer”: Python-sidecar CRANE integration is explicitly Tier 2; Tier 1 is provider/Rust runtime plus validation/pinning. | 09-I:L923-L943 | I34-E014, I34-E018, I34-C001 |
| SD-I34-002 | **REMOVE/FLAG** summary claim that Tier 1 records a tight-grammar arm’s measured tax; the numbered source only carries an external premise and requires CRANE pass rate >= FreeText in its fixture. | 09-I:L925-L940 | I34-E003, I34-E016, I34-E024, I34-A005 |
| SD-I34-003 | **ADD** complete event family, enum-source rejection, R8 freeze, exact production routes, D-16 refusal, and cross-language/UI evidence. | 09-I:L928-L943 | I34-E007–I34-E017, I34-E020–I34-E023 |
| SD-I34-004 | **ADD** cross-language verifier-pack deferral, module-path TBD, commit-point/manifest/trust ambiguities, and local fixture-statistics gap. | 09-I:L919-L943 | I34-E019, I34-E021, I34-A001–I34-A005 |
| SD-I34-005 | **ADD** J-15 Option-A amendment: plan-time llguidance `ParserFactory` compile/validate/pin with XGrammar unchanged at inference. | 09-I:L919-L943 | I34-E026–I34-E031 |
| SD-I34-006 | **ADD** nonserializable parser state, spec+pin persistence, boot recompilation, replay drift detection, and README-only compile-performance evidence. | 09-I:L919-L943 | I34-E029, I34-E033–I34-E034 |
| SD-I34-007 | **ADD** new-dependency user approval/current anti-slopsquat recheck, dated v1.7.6 snapshot caveat, and xgrammar-rs rejection. | 09-I:L919-L943 | I34-E027, I34-E035–I34-E037, I34-A006 |
| SD-I34-008 | **FLAG CONFLICT** reasoning-heavy CRANE/final-only constraints versus amendment HD-3 role ban must be resolved; the summary does not surface this. | 09-I:L923-L940 | I34-E004–I34-E005, I34-E032, I34-C001 |

### I-35 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-I35-001 | **CORRECT** summary’s “taint check has run”: source requires an intervening `taint_check PASS`, specifically before a write-class call. | 09-I:L951-L954 | I35-E005 |
| SD-I35-002 | **CORRECT** summary’s “mutable equal to zero”: the numbered source specifies an R8-frozen rule hash but no `mutable` field/value. | 09-I:L963-L970 | I35-E009, I35-E026 |
| SD-I35-003 | **ADD/FLAG** require-confirmation and substitute are explicit rule actions but lack Tier-1 events/fixtures/continuation semantics. | 09-I:L955-L957 | I35-E007–I35-E008, I35-A002 |
| SD-I35-004 | **ADD** exact post-existing-gates/pre-execution placement, current-session scope, full event kinds, class fixtures, and cross-session deferral. | 09-I:L949-L972 | I35-E003–I35-E006, I35-E017–I35-E023 |
| SD-I35-005 | **ADD** taint/catalog composition, production reachability, gate-freeze negative test, arXiv caution, and no-reimplementation constraints. | 09-I:L949-L972 | I35-E012–I35-E016, I35-E024–I35-E027 |
| SD-I35-006 | **FLAG** undefined TOML DSL/class mapping/history boundary and untested absolute CDA-immunity claim. | 09-I:L949-L972 | I35-A001, I35-A003–I35-A005 |

### I-40 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-I40-001 | **QUALIFY** summary’s “write/dependency/blast-radius gates” expansion: the numbered plan groups F-1/F-9/G-3 as destructive-class write gates and does not separately specify three wrapper stages. | 09-I:L1086-L1095 | I40-E007–I40-E011, I40-E016 |
| SD-I40-002 | **QUALIFY** summary “proxies wrapped tool input and output”: the numbered source explicitly says intercepts external tool traffic at MCP/stdio, without separately defining output-proxy semantics. | 09-I:L1097-L1099 | I40-E006, I40-A003 |
| SD-I40-003 | **ADD** exact open-spec contents (`chain_prev_hash`, taint schema, write-gate schema, envelope), I-18 ordering ambiguity, and normative schema gaps. | 09-I:L1083-L1096 | I40-E003–I40-E005, I40-E017, I40-A001–I40-A002 |
| SD-I40-004 | **ADD** no-new-action reuse of `taint_flow`/`run_artifact_emitted`, exact production chains, one-fixture limit, and new-trust-primitive ban. | 09-I:L1086-L1104 | I40-E012, I40-E018–I40-E019, I40-E024–I40-E027 |
| SD-I40-005 | **ADD** real MCP matrix/network approval, commercial packaging, and Rekor publication deferrals. | 09-I:L1079-L1104 | I40-E021–I40-E023 |
| SD-I40-006 | **QUALIFY** RRA viewer surface: source says Svelte pairing/cross-language gate is conditional on execution-time re-grep, not established. | 09-I:L1099-L1099 | I40-E020, I40-A004 |
| SD-I40-007 | **ADD ORIENTATION-ONLY STATUS** the discovery summary reports `docs/spec/rra-spec-v1.md` already exists and requires conformance inspection; that fact is not established by the numbered source and was not promoted to a source atom. | 09-I:L1702-L1705 | I40-E003, I40-A002 |
| SD-I40-008 | **ADD** redaction-stage/bypass uncertainties and explicit no-raw-secret P0 boundary. | 09-I:L1092-L1104 | I40-E010, I40-E025–I40-E027, I40-A003, I40-A005 |

### I-42 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-I42-001 | **QUALIFY** summary “spawned agents may run auditor/spine”: source says spawned agents are critique-only and the shipped auditor/spine runs at ship time, without explicitly assigning that spine to spawned agent authorship. | 09-I:L1149-L1152 | I42-E004, I42-E007–I42-E009, I42-A001 |
| SD-I42-002 | **ADD** generator-level no-side-effect-Directive rule, exact design/operator/ship loop, high-stakes rationale, and one-engine/no-prompt bans. | 09-I:L1147-L1165 | I42-E003–I42-E012, I42-E027 |
| SD-I42-003 | **ADD** no-new-action/phase-transition reuse, actor-attribution inference, production reachability, and edit-time Supervisor-versus-spawn-site choice. | 09-I:L1152-L1165 | I42-E015, I42-E023, I42-E025–I42-E026, I42-I001, I42-A002, I42-A004 |
| SD-I42-004 | **ADD** deferred Settings switch, per-profile telemetry, and future profile variants. | 09-I:L1142-L1165 | I42-E020–I42-E022 |
| SD-I42-005 | **ADD** exact contract/decision artifacts and complete acceptance evidence: critique rows, zero writes, Orchestrate control arm, byte-identical spine verdicts. | 09-I:L1147-L1165 | I42-E017–I42-E019, I42-E024, I42-E028–I42-E030 |
| SD-I42-006 | **FLAG** permissions for non-executor agents, operator-edit ingestion, and how differing write behavior yields identical verdict bytes. | 09-I:L1147-L1165 | I42-A001, I42-A003, I42-A005 |

### I-45 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-I45-001 | **ADD** exact canonical sources/order/whitespace/blake3 requirements, orchestrator-only ownership, and shared-serializer inference across every read path. | 09-I:L1238-L1253 | I45-E004–I45-E007, I45-E019–I45-E020, I45-I001 |
| SD-I45-002 | **ADD** the full read-path list (`verify-attestation`, replay, plan status), I-39 downstream role, and hard-fail/no-warning behavior. | 09-I:L1242-L1253 | I45-E007–I45-E008, I45-E018, I45-E024–I45-E025 |
| SD-I45-003 | **ADD/QUALIFY** complete `ObjectiveSealed` kind list and flag `ReceiptStale` event-versus-error ambiguity. | 09-I:L1250-L1253 | I45-E017, I45-E021–I45-E023, I45-A004 |
| SD-I45-004 | **ADD** original roach-pi read-before-execution rule, shipped-substrate correction, F-4/I-18 record coordination, and no-new-verifier/gate constraint. | 09-I:L1233-L1253 | I45-E003, I45-E009, I45-E011–I45-E014 |
| SD-I45-005 | **ADD** per-requirement/cross-session deferrals and unresolved LEGACY/missing-field semantics. | 09-I:L1238-L1253 | I45-E010, I45-E015, I45-A003, I45-A005 |
| SD-I45-006 | **FLAG** every-PASS versus every-sealed-verdict scope and undefined canonicalization/source precedence. | 09-I:L1238-L1244 | I45-A001–I45-A002 |

## Exhaustiveness, zero-omission, and duplicate audit

- Source coverage: **10/10 files**, **1,155/1,155 logical lines**, **92,980/92,980 bytes** read and reconciled; source SHA mismatches **0**.
- Atomic source ledger: **351 unique atoms** = **295 explicit**, **006 inferred**, **047 ambiguous**, **003 conflicting/corrective**.
- Per-file atom counts: I-2 **043**; I-3 **032**; I-12 **032**; I-21 **038**; I-27 **029**; I-34 **044**; I-35 **032**; I-40 **033**; I-42 **037**; I-45 **031**. Sum **351**.
- Relational interaction records: **30** (`IX-001`–`IX-030`). Summary-delta records: **064** (`SD-*`). Ambiguity/conflict register: **24** (`AC-001`–`AC-024`). These records reference source atoms and are not double-counted as features.
- ID audit: every atomic table row has exactly one unique ID matching `I2/I3/I12/I21/I27/I34/I35/I40/I42/I45-[EIAC]nnn`. Duplicate atomic IDs **0**; missing/blank atomic IDs **0**; nonconforming atomic IDs **0**.
- Semantic duplicate audit: repeated premise/scope/wiring/grounding/acceptance statements were merged into one atom with multiple anchors. Interactions, conflict register, and summary deltas reference atoms instead of cloning them. Duplicate feature atoms found **0**.
- Omission audit: every source was checked against capabilities/actions/surfaces; workflows/states; schemas/data/lifecycle; flags/defaults/config; integrations/events; permissions/security/privacy/compliance; failures/fallback/retry/recovery; determinism/idempotency/concurrency; observability/evidence; tests/acceptance/done gates; constraints/non-goals; dependencies/status/deferred. Each per-file ledger explicitly names absent dimensions rather than inventing them.
- Interpretation audit: inferred, ambiguous, and conflicting/corrective atoms are visibly separated by ID/class. No inference is promoted to a requirement. Orientation-only claims remain in delta records and are not source atoms.
- Manifest-boundary audit: the only original sources opened were the ten exact DEEP-I-004 manifest rows. The only orientation artifact consulted was `09-I-SUMMARY_INDEX.discovery.md`.
- Output metrics after finalization: `output_lines=0667`; `output_bytes=104607`.
- Canonical normalized output SHA-256: cf018a577b7f634769909807b6400de6a52e30dc7501655e3da3b246fc61b942.
- Hash rule: compute SHA-256 over the exact final UTF-8 bytes including the final newline after replacing only the 64 hexadecimal characters on the `Canonical normalized output SHA-256` line with 64 ASCII zeroes. Fixed-width output metrics and digest substitution avoid impossible raw self-reference.
