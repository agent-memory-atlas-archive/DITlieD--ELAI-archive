# Family O aggregate — ELAI deep feature discovery

## Document contract

- **Family:** O (RESEARCH-MACHINE / personal research machine: O-0 index + O-1 research daemon + O-2 TaskSpec substrate + O-3 brain vault + O-4 canvas WORKFLOW).
- **Inputs:** `DEEP-O-001.discovery.md` (1 deep ledger) and `15-O-SUMMARY_INDEX.discovery.md`.
- **Policy:** Every distinct atomic record is preserved with its ID and source anchor (ledger file + line). No prose compression. No silent drop of minor atoms.
- **Namespacing:** Deep atoms keep native IDs under ledger prefix `O-001/`. Summary atoms use `SUMMARY/`. Cross-file IX/SD/AC stay namespaced by ledger (e.g. `O-001/IX-PATH01`, `O-001/SD-MC001`, `SUMMARY/AC-001`).
- **NEWER-PLAN PRECEDENCE (within family):** An explicit SUPERSEDED/replacement/CORRECT marker in a source beats everything; otherwise a higher plan number is newer and wins the **target state** description. Losing descriptions are retained as superseded notes with anchors. **Deep originals supersede the summary orientation pass** where they conflict (see Material corrections / Superseded notes).
- **Status language preserved:** PLANNED / IN PROGRESS / DONE / DEFERRED / dark/absent / Tier 1 / Tier 2 / KEYSTONE are not collapsed.
- **Generated:** 2026-08-12 (O-family aggregator pass)

### Ledgers merged

| Ledger | Path | Explicit E | IX | SD | AC | INF | Other | Subtotal |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| O-001 | `docs/thefinalglance/feature-discovery/deep/DEEP-O-001.discovery.md` | 520 | 85 | 31 | 25 | 12 | 0 | **673** |
| SUMMARY | `docs/thefinalglance/feature-discovery/15-O-SUMMARY_INDEX.discovery.md` | 185 | 0 | 0 | 33 | 10 | 27 (WF+INV+SILENT) | **255** |
| **Aggregate distinct records** | — | — | — | — | — | — | — | **928** |

### Per-ledger declared vs extracted

- **O-001 explicit E:** source footer declares **520**; extracted **520** — **exact**
- **O-001 unit breakdown:** O0=95, O1=111, O2=112, O3=109, O4=93 (declared 95+111+112+109+93=520)
- **SUMMARY explicit E:** source declares **185**; extracted **185** — **exact** (global 14; O-1 42; O-2 43; O-3 35; O-4 33; cross-cutting 18)

### Family map (session inventory)

| Unit | Role | Wave / effort | Deep ledger |
|---|---|---|---|
| O-0 | Track index / doctrine / sequencing | track-wide | O-001 |
| O-1 research daemon (generalize Cosmos) | KEYSTONE; typed-topic manifest + enum-classify | Wave 1, M-L | O-001 |
| O-2 scheduled TaskSpec substrate | Typed/journaled scheduler | Wave 1, M-L | O-001 |
| O-3 brain vault contract + mounting | Mount-not-convert; namespace claims | Wave 2, M | O-001 |
| O-4 canvas WORKFLOW mode | Spec editor/monitor; never a runtime | Wave 3, M; depends O-2 | O-001 |
| Summary orientation | 185 inventory atoms | all | SUMMARY |

## Notation

- `[E]` explicit · `[I]` inference · `[A]` ambiguity · `[C]` conflict/correction · `[Δ]` summary delta · `[SURFACE]` user-facing · `[superseded]` losing target-state retained · `[IX]` cross-file interaction/composition · `[WF]` workflow · `[INV]` invariant · `[SILENT]` summary-silent audit candidate.
- Anchors: `O0:Ln` / `O1:Ln` / … = O-plan originals; `O-SUMMARY:Ln` = summary index lines; ledger-line = discovery file line.

---

## Per-unit atomic records (deep ledger O-001)

Source file: `docs/thefinalglance/feature-discovery/deep/DEEP-O-001.discovery.md`
Extracted: E=520, IX=85, SD=31, AC=25, INF=12, subtotal=673

### Unit `O0` — O-Plan Index / track-wide contract — from O-001 (95 records)

- **O-001/O0-E001** [E] anchor=`O0:L1` (ledger-line: DEEP-O-001.discovery.md:L38): The index names O as “the PERSONAL RESEARCH MACHINE side,” says it is wiring-enforced, comprises `4 sessions`, and has authored Track `RESEARCH-MACHINE`.
- **O-001/O0-E002** [E] anchor=`O0:L2` (ledger-line: DEEP-O-001.discovery.md:L39): The index is dated `2026-07-06`.
- **O-001/O0-E003** [E] anchor=`O0:L2` (ledger-line: DEEP-O-001.discovery.md:L40): Track-level `PROOF_SPINE` is `none`, owner-acknowledged.
- **O-001/O0-E004** [E] anchor=`O0:L4-L7` (ledger-line: DEEP-O-001.discovery.md:L41): The source digest’s layers `L1..L4` map `1:1` to `O-1..O-4`.
- **O-001/O0-E005** [E] anchor=`O0:L6-L7` (ledger-line: DEEP-O-001.discovery.md:L42): Digest attacks `A1..A5` and its verbatim-steal list feed each unit’s `REDTEAM` and `stub_ban`.
- **O-001/O0-E006** [E] anchor=`O0:L8-L9` (ledger-line: DEEP-O-001.discovery.md:L43): O-1/O-3 are instructed to steal the live-proven enum-gated classification, injection-canary, confidence-per-sink, `write_page`, `upsert_page`, and namespace patterns from the TEIKOKU handoff.
- **O-001/O0-E007** [E] anchor=`O0:L11` (ledger-line: DEEP-O-001.discovery.md:L44): O’s predecessor range is `B-plan .. N-plan`; O must compose with it and restate none of it under `GD-1`.
- **O-001/O0-E008** [E] [SURFACE] anchor=`O0:L11-L13` (ledger-line: DEEP-O-001.discovery.md:L45): O is a product-surface letter that ships a new user-facing feature, unlike N’s invent-nothing interlacing role.
- **O-001/O0-E009** [E] [SURFACE] anchor=`O0:L12-L13` (ledger-line: DEEP-O-001.discovery.md:L46): The new surface is the combination of a research daemon, typed scheduler, mounted brain vault, and canvas workflow mode.
- **O-001/O0-E010** [E] anchor=`O0:L13-L16` (ledger-line: DEEP-O-001.discovery.md:L47): The surface is estimated as roughly `80%` composition of shipped subsystems.
- **O-001/O0-E011** [E] [SURFACE] anchor=`O0:L13-L15` (ledger-line: DEEP-O-001.discovery.md:L48): Named composed subsystems are `B-42` Cosmos, `C-9` task_runner, `3F` rules-engine, `E-5` memory, `D-5` privacy, `G-1` taint, `F-3` tool-filter, `G-2` VCR, and `H-12/J-31/J-60` canvas.
- **O-001/O0-E012** [E] [SURFACE] anchor=`O0:L15-L16` (ledger-line: DEEP-O-001.discovery.md:L49): Genuinely new mechanisms are a closed `TaskSpec` vocabulary, vault namespace claims, canvas `WORKFLOW` mode, and Cosmos manifest generalization.
- **O-001/O0-E013** [E] anchor=`O0:L18-L24` (ledger-line: DEEP-O-001.discovery.md:L50): Mandatory predecessor reading order is source design, `ELAI-FEATURE-MAP.md`, `F-0_INDEX.md`, `G-0_INDEX.md`, this index, `O-SUMMARY_INDEX.md`, then the target detail file.
- **O-001/O0-E014** [E] anchor=`O0:L20-L21` (ledger-line: DEEP-O-001.discovery.md:L51): The feature map is designated the single context load and executors are told not to re-read the legacy 12-file list.
- **O-001/O0-E015** [E] anchor=`O0:L26-L30` (ledger-line: DEEP-O-001.discovery.md:L52): The central finding is that the personal research machine is architecturally cheap because it mostly composes already-shipped subsystems.
- **O-001/O0-E016** [E] anchor=`O0:L29-L30` (ledger-line: DEEP-O-001.discovery.md:L53): ELAI’s product differentiator for this surface is journaled, gated, replayable runs.
- **O-001/O0-E017** [E] anchor=`O0:L30-L31` (ledger-line: DEEP-O-001.discovery.md:L54): Building a generic dataflow engine, described as “n8n-minus-integrations,” is explicitly the wrong architecture.
- **O-001/O0-E018** [E] anchor=`O0:L31-L32` (ledger-line: DEEP-O-001.discovery.md:L55): The required architecture is a closed-vocabulary typed task layer.
- **O-001/O0-E019** [E] anchor=`O0:L31-L32` (ledger-line: DEEP-O-001.discovery.md:L56): Every stage kind must exist as code.
- **O-001/O0-E020** [E] anchor=`O0:L31-L32` (ledger-line: DEEP-O-001.discovery.md:L57): Every task fire must be an SSB row.
- **O-001/O0-E021** [E] [SURFACE] anchor=`O0:L33-L34` (ledger-line: DEEP-O-001.discovery.md:L58): O-1 swaps Cosmos’s topic source for a user-editable typed topic set.
- **O-001/O0-E022** [E] anchor=`O0:L33-L34` (ledger-line: DEEP-O-001.discovery.md:L59): O-1 adds an enum-classify surface.
- **O-001/O0-E023** [E] anchor=`O0:L33-L34` (ledger-line: DEEP-O-001.discovery.md:L60): O-1 must thread `G-1/F-3/G-2` trust handling through every fetched byte.
- **O-001/O0-E024** [E] anchor=`O0:L35-L36` (ledger-line: DEEP-O-001.discovery.md:L61): O-2 adds a typed `TaskSpec` scheduler stored as rules-as-data in SQLite.
- **O-001/O0-E025** [E] anchor=`O0:L35-L36` (ledger-line: DEEP-O-001.discovery.md:L62): O-2 supports `cron`, `ssb`, and `manual` trigger categories at full scope.
- **O-001/O0-E026** [E] [SURFACE] anchor=`O0:L35-L36` (ledger-line: DEEP-O-001.discovery.md:L63): O-2 journals fires and supplies a dashboard intervention-card inbox.
- **O-001/O0-E027** [E] anchor=`O0:L35-L36` (ledger-line: DEEP-O-001.discovery.md:L64): O-2 reuses the task_runner interval and SSB temporal monitor.
- **O-001/O0-E028** [E] [SURFACE] anchor=`O0:L37-L38` (ledger-line: DEEP-O-001.discovery.md:L65): O-3 mounts, rather than converts, a portable Markdown brain vault.
- **O-001/O0-E029** [E] [SURFACE] anchor=`O0:L37-L38` (ledger-line: DEEP-O-001.discovery.md:L66): O-3 adds a namespace-claims contract so ELAI and TEIKOKU can share one vault.
- **O-001/O0-E030** [E] [SURFACE] anchor=`O0:L37-L38` (ledger-line: DEEP-O-001.discovery.md:L67): `memory_facts` remains the machine-grain store while the vault becomes the user-grain surface.
- **O-001/O0-E031** [E] [SURFACE] anchor=`O0:L39-L40` (ledger-line: DEEP-O-001.discovery.md:L68): O-4 adds canvas `WORKFLOW` as an editor and monitor over TaskSpecs.
- **O-001/O0-E032** [E] anchor=`O0:L39-L40` (ledger-line: DEEP-O-001.discovery.md:L69): O-4 makes wire-dark conditions visible.
- **O-001/O0-E033** [E] [SURFACE] anchor=`O0:L39-L40` (ledger-line: DEEP-O-001.discovery.md:L70): O-4 dispatches through the `J-31` FSM and must never introduce a canvas runtime.
- **O-001/O0-E034** [E] anchor=`O0:L42-L43` (ledger-line: DEEP-O-001.discovery.md:L71): Allowed status values are `PLANNED`, `IN PROGRESS`, `DONE`, and `DEFERRED`.
- **O-001/O0-E035** [E] anchor=`O0:L43-L44` (ledger-line: DEEP-O-001.discovery.md:L72): `DONE` requires `wiring_contract_check.py --run-trigger` to pass.
- **O-001/O0-E036** [E] anchor=`O0:L43-L44` (ledger-line: DEEP-O-001.discovery.md:L73): `DONE` requires `production_caller_check.py` and `python_wire_check.py` to be clean.
- **O-001/O0-E037** [E] anchor=`O0:L44` (ledger-line: DEEP-O-001.discovery.md:L74): `DONE` requires the `R10` integration smoke to be green.
- **O-001/O0-E038** [E] anchor=`O0:L44-L45` (ledger-line: DEEP-O-001.discovery.md:L75): Cargo-green is necessary but not sufficient for `DONE`.
- **O-001/O0-E039** [E] anchor=`O0:L50` (ledger-line: DEEP-O-001.discovery.md:L76): O inherits `W-1..W-10` verbatim from `F-0_INDEX.md`.
- **O-001/O0-E040** [E] anchor=`O0:L50-L51` (ledger-line: DEEP-O-001.discovery.md:L77): Inherited wiring doctrine requires a real-entrypoint trigger and an off-agent observable artifact.
- **O-001/O0-E041** [E] anchor=`O0:L51` (ledger-line: DEEP-O-001.discovery.md:L78): Missing evidence is a failure.
- **O-001/O0-E042** [E] anchor=`O0:L51` (ledger-line: DEEP-O-001.discovery.md:L79): Reachability checks must use sound references rather than leaf grep.
- **O-001/O0-E043** [E] anchor=`O0:L51` (ledger-line: DEEP-O-001.discovery.md:L80): A producer without a consumer is a build break.
- **O-001/O0-E044** [E] anchor=`O0:L51-L52` (ledger-line: DEEP-O-001.discovery.md:L81): Test callers never satisfy production reachability.
- **O-001/O0-E045** [E] anchor=`O0:L52` (ledger-line: DEEP-O-001.discovery.md:L82): The track inherits a frozen-gate requirement.
- **O-001/O0-E046** [E] anchor=`O0:L52` (ledger-line: DEEP-O-001.discovery.md:L83): A default-off graveyard is forbidden.
- **O-001/O0-E047** [E] [SURFACE] anchor=`O0:L52` (ledger-line: DEEP-O-001.discovery.md:L84): Execution requires re-reading the live baseline and adding a dashboard done-item.
- **O-001/O0-E048** [E] anchor=`O0:L53` (ledger-line: DEEP-O-001.discovery.md:L85): O inherits `GD-1..GD-5` from `G-0_INDEX.md`.
- **O-001/O0-E049** [E] anchor=`O0:L53-L54` (ledger-line: DEEP-O-001.discovery.md:L86): `GD-1` requires compose-never-restate: each O unit names consumed mechanisms and must not reimplement them.
- **O-001/O0-E050** [E] anchor=`O0:L54` (ledger-line: DEEP-O-001.discovery.md:L87): `GD-4` requires observe-only-first for any gate defaulting on.
- **O-001/O0-E051** [E] anchor=`O0:L54` (ledger-line: DEEP-O-001.discovery.md:L88): `GD-5` requires honest residue to be named in-file.
- **O-001/O0-E052** [E] anchor=`O0:L55-L56` (ledger-line: DEEP-O-001.discovery.md:L89): O adds no new doctrine-delta family.
- **O-001/O0-E053** [E] anchor=`O0:L55-L56` (ledger-line: DEEP-O-001.discovery.md:L90): `A1..A5` are enforced through per-unit `stub_ban` and `REDTEAM` blocks.
- **O-001/O0-E054** [E] anchor=`O0:L58` (ledger-line: DEEP-O-001.discovery.md:L91): All `O-1..O-4` have `PROOF_SPINE: none` because they are product-surface/distribution work.
- **O-001/O0-E055** [E] anchor=`O0:L58` (ledger-line: DEEP-O-001.discovery.md:L92): The proof-spine absence was owner-acknowledged on `2026-07-06` under `PSR-2`.
- **O-001/O0-E056** [E] anchor=`O0:L59-L60` (ledger-line: DEEP-O-001.discovery.md:L93): `PS-3` is only a weak nearest candidate if briefings are counted as grounding inputs; the plan does not claim it.
- **O-001/O0-E057** [E] anchor=`O0:L60` (ledger-line: DEEP-O-001.discovery.md:L94): O feeds no `PS-1..PS-6` number directly.
- **O-001/O0-E058** [E] anchor=`O0:L62-L63` (ledger-line: DEEP-O-001.discovery.md:L95): The FSM may not reach `COMPLETE` without `cargo check` green.
- **O-001/O0-E059** [E] anchor=`O0:L62-L63` (ledger-line: DEEP-O-001.discovery.md:L96): Completion requires the auditor to pass on affected crates only.
- **O-001/O0-E060** [E] anchor=`O0:L63` (ledger-line: DEEP-O-001.discovery.md:L97): Completion requires wiring-contract trigger evidence.
- **O-001/O0-E061** [E] anchor=`O0:L63-L64` (ledger-line: DEEP-O-001.discovery.md:L98): Completion requires a clean wire checker or a `WIRE-DARK[O-N]` row.
- **O-001/O0-E062** [E] anchor=`O0:L64` (ledger-line: DEEP-O-001.discovery.md:L99): O-4 completion requires `cross_lang_diff` for its frontend pair.
- **O-001/O0-E063** [E] anchor=`O0:L64` (ledger-line: DEEP-O-001.discovery.md:L100): Completion requires the `R10` integration smoke and a memory-writer run.
- **O-001/O0-E064** [E] anchor=`O0:L69-L71` (ledger-line: DEEP-O-001.discovery.md:L101): O-1 is `[KEYSTONE]`, effort `M-L`, Wave `1`, with lever `manifest-generalization`.
- **O-001/O0-E065** [E] anchor=`O0:L70-L71` (ledger-line: DEEP-O-001.discovery.md:L102): O-1 ships `steal-scan` and `monitor` kinds in its listed session scope.
- **O-001/O0-E066** [E] anchor=`O0:L72-L73` (ledger-line: DEEP-O-001.discovery.md:L103): O-2 is effort `M-L`, Wave `1`, with lever `rules-as-data-scheduling`.
- **O-001/O0-E067** [E] anchor=`O0:L74-L75` (ledger-line: DEEP-O-001.discovery.md:L104): O-3 is effort `M`, Wave `2`, with lever `mount-not-convert`.
- **O-001/O0-E068** [E] [SURFACE] anchor=`O0:L76-L77` (ledger-line: DEEP-O-001.discovery.md:L105): O-4 is effort `M`, Wave `3`, with lever `canvas-as-spec-editor`.
- **O-001/O0-E069** [E] anchor=`O0:L80-L85` (ledger-line: DEEP-O-001.discovery.md:L106): Any O unit adding an SSB `action_type` must bump `SCHEMA_VERSION` at the live baseline.
- **O-001/O0-E070** [E] anchor=`O0:L82` (ledger-line: DEEP-O-001.discovery.md:L107): O-1 adds a classify-row action type.
- **O-001/O0-E071** [E] anchor=`O0:L82` (ledger-line: DEEP-O-001.discovery.md:L108): O-2 adds `task_spec_fired`.
- **O-001/O0-E072** [E] [SURFACE] anchor=`O0:L82-L83` (ledger-line: DEEP-O-001.discovery.md:L109): O-3 adds `vault_write` and `claim_violation`.
- **O-001/O0-E073** [E] anchor=`O0:L83` (ledger-line: DEEP-O-001.discovery.md:L110): O-4 reuses O-2’s fire row and therefore does not introduce a distinct O-4 event in the index.
- **O-001/O0-E074** [E] anchor=`O0:L83-L85` (ledger-line: DEEP-O-001.discovery.md:L111): Executors must never hard-code the schema baseline; they must re-read `ssb.rs` and `action_type_lock.rs` at edit time under `W-9`.
- **O-001/O0-E075** [E] anchor=`O0:L84-L85` (ledger-line: DEEP-O-001.discovery.md:L112): The decision document must quote `baseline+N from LIVE X/Y as of <edit date> (file:line)`.
- **O-001/O0-E076** [E] anchor=`O0:L88-L92` (ledger-line: DEEP-O-001.discovery.md:L113): Wave 1 runs O-1 and O-2 in parallel because their knowledge versus orchestrator/rules crate touch sets are declared disjoint.
- **O-001/O0-E077** [E] anchor=`O0:L90-L92` (ledger-line: DEEP-O-001.discovery.md:L114): O-1’s trust-spine threading is the ELAI-over-TEIKOKU delta and is the keystone into which other work sinks.
- **O-001/O0-E078** [E] anchor=`O0:L93-L94` (ledger-line: DEEP-O-001.discovery.md:L115): O-3 can land after Wave 1 supplies a `FileToBrain` caller.
- **O-001/O0-E079** [E] [SURFACE] anchor=`O0:L93-L94` (ledger-line: DEEP-O-001.discovery.md:L116): O-3 may instead ship with interim CLI caller `elai brain write`.
- **O-001/O0-E080** [E] anchor=`O0:L95-L96` (ledger-line: DEEP-O-001.discovery.md:L117): O-4 depends on O-2’s `TaskSpec` store being queryable.
- **O-001/O0-E081** [E] anchor=`O0:L95-L96` (ledger-line: DEEP-O-001.discovery.md:L118): If that store is not queryable, O-4 may ship against a read-only fixture projection under its fork.
- **O-001/O0-E082** [E] anchor=`O0:L99-L102` (ledger-line: DEEP-O-001.discovery.md:L119): Hashing must use `blake3`, not `blake2` or `sha2`.
- **O-001/O0-E083** [E] anchor=`O0:L101` (ledger-line: DEEP-O-001.discovery.md:L120): Production paths may not use `.unwrap()` or `.expect()`.
- **O-001/O0-E084** [E] anchor=`O0:L101-L102` (ledger-line: DEEP-O-001.discovery.md:L121): Tests are scoped to affected crates and are run by the auditor.
- **O-001/O0-E085** [E] [SURFACE] anchor=`O0:L102` (ledger-line: DEEP-O-001.discovery.md:L122): New dependencies require anti-slopsquatting review; none are expected.
- **O-001/O0-E086** [E] anchor=`O0:L102` (ledger-line: DEEP-O-001.discovery.md:L123): Closed enums must deny unknown values.
- **O-001/O0-E087** [E] anchor=`O0:L102-L103` (ledger-line: DEEP-O-001.discovery.md:L124): New gates use a kill switch, not an enable flag, under `W-2`.
- **O-001/O0-E088** [E] anchor=`O0:L103` (ledger-line: DEEP-O-001.discovery.md:L125): Every arXiv/external claim must be WebSearch-verified at authoring; O cites no arXiv.
- **O-001/O0-E089** [E] anchor=`O0:L106-L109` (ledger-line: DEEP-O-001.discovery.md:L126): Cosmos generalization is a swap of the topic source, not a rewrite of `CosmosStage`.
- **O-001/O0-E090** [E] anchor=`O0:L110` (ledger-line: DEEP-O-001.discovery.md:L127): The wall-clock trigger remains supervisor-plane and outside the replay set under `K-14`.
- **O-001/O0-E091** [E] [SURFACE] anchor=`O0:L111` (ledger-line: DEEP-O-001.discovery.md:L128): The vault is mounted and never converted into `memory_facts`.
- **O-001/O0-E092** [E] [SURFACE] anchor=`O0:L112` (ledger-line: DEEP-O-001.discovery.md:L129): The canvas has no runtime; the `J-31` FSM performs dispatch.
- **O-001/O0-E093** [E] anchor=`O0:L114-L117` (ledger-line: DEEP-O-001.discovery.md:L130): The session log is empty at plan generation.
- **O-001/O0-E094** [E] anchor=`O0:L119-L123` (ledger-line: DEEP-O-001.discovery.md:L131): The declared next step is O-1 recon of the Cosmos manifest type and G-1 taint-wrap API, then manifest generalization.
- **O-001/O0-E095** [E] anchor=`O0:L122-L123` (ledger-line: DEEP-O-001.discovery.md:L132): O-1 and O-2 should be dispatched in parallel because they target different crates.

### Unit `O1` — O-1: research daemon — generalize Cosmos from designer-briefs to a user-editable typed topic set — from O-001 (111 records)

- **O-001/O1-E001** [E] anchor=`O1:L1-L3` (ledger-line: DEEP-O-001.discovery.md:L136): `O-1` is `PLANNED`, Wave `1`, product-surface class, and `PROOF_SPINE: none`, owner-acknowledged `2026-07-06`.
- **O-001/O1-E002** [E] anchor=`O1:L5` (ledger-line: DEEP-O-001.discovery.md:L137): Metadata is Track `RESEARCH-MACHINE`, lever `manifest-generalization`, source finding `L1`, grounding `MODERATE (index-derived, PLAN-asserted)`, nature `BUILD-on-shipped`, effort `M-L`, Wave `1`.
- **O-001/O1-E003** [E] anchor=`O1:L7` (ledger-line: DEEP-O-001.discovery.md:L138): `CosmosWorker` is asserted to already be a scheduled research loop.
- **O-001/O1-E004** [E] anchor=`O1:L7` (ledger-line: DEEP-O-001.discovery.md:L139): Its existing loop has an artifact manifest.
- **O-001/O1-E005** [E] anchor=`O1:L7` (ledger-line: DEEP-O-001.discovery.md:L140): Its existing loop has an hourly tick.
- **O-001/O1-E006** [E] anchor=`O1:L7` (ledger-line: DEEP-O-001.discovery.md:L141): Its existing loop refreshes through `FlashResearch`.
- **O-001/O1-E007** [E] anchor=`O1:L7` (ledger-line: DEEP-O-001.discovery.md:L142): Its existing lifecycle time-decays artifacts to `DECAYED`.
- **O-001/O1-E008** [E] anchor=`O1:L7` (ledger-line: DEEP-O-001.discovery.md:L143): Its existing briefing is capped at `<=1500` tokens.
- **O-001/O1-E009** [E] anchor=`O1:L7` (ledger-line: DEEP-O-001.discovery.md:L144): `CosmosStage` already participates in the `QueryPipeline`.
- **O-001/O1-E010** [E] anchor=`O1:L7` (ledger-line: DEEP-O-001.discovery.md:L145): The plan estimates this existing system as `90%` of a general research daemon.
- **O-001/O1-E011** [E] anchor=`O1:L7` (ledger-line: DEEP-O-001.discovery.md:L146): The current limitation is a manifest hard-bound to designer briefs.
- **O-001/O1-E012** [E] anchor=`O1:L7` (ledger-line: DEEP-O-001.discovery.md:L147): O-1 generalizes only the manifest surface.
- **O-001/O1-E013** [E] anchor=`O1:L7` (ledger-line: DEEP-O-001.discovery.md:L148): O-1 must thread every fetched byte through the existing taint, tool-filter, and VCR trust spine.
- **O-001/O1-E014** [E] anchor=`O1:L7` (ledger-line: DEEP-O-001.discovery.md:L149): O-1 creates neither a new worker loop nor a new scheduler; O-2 owns scheduling.
- **O-001/O1-E015** [E] anchor=`O1:L9-L10` (ledger-line: DEEP-O-001.discovery.md:L150): The generalized configuration root is `[[research.topics]]`.
- **O-001/O1-E016** [E] anchor=`O1:L10` (ledger-line: DEEP-O-001.discovery.md:L151): Each research-topic record has an `id` field.
- **O-001/O1-E017** [E] anchor=`O1:L10` (ledger-line: DEEP-O-001.discovery.md:L152): Each research-topic record has a `kind` field.
- **O-001/O1-E018** [E] anchor=`O1:L10` (ledger-line: DEEP-O-001.discovery.md:L153): Allowed topic kind `steal-scan` is part of a closed enum.
- **O-001/O1-E019** [E] anchor=`O1:L10` (ledger-line: DEEP-O-001.discovery.md:L154): Allowed topic kind `leaderboard` is part of the closed enum.
- **O-001/O1-E020** [E] anchor=`O1:L10` (ledger-line: DEEP-O-001.discovery.md:L155): Allowed topic kind `discovery` is part of the closed enum.
- **O-001/O1-E021** [E] anchor=`O1:L10` (ledger-line: DEEP-O-001.discovery.md:L156): Allowed topic kind `monitor` is part of the closed enum.
- **O-001/O1-E022** [E] anchor=`O1:L10` (ledger-line: DEEP-O-001.discovery.md:L157): Each research-topic record has `sources[]`.
- **O-001/O1-E023** [E] anchor=`O1:L10` (ledger-line: DEEP-O-001.discovery.md:L158): Each research-topic record has `cadence`.
- **O-001/O1-E024** [E] anchor=`O1:L10` (ledger-line: DEEP-O-001.discovery.md:L159): Each research-topic record has `confidence_threshold`.
- **O-001/O1-E025** [E] anchor=`O1:L10` (ledger-line: DEEP-O-001.discovery.md:L160): Each research-topic record has `sink`.
- **O-001/O1-E026** [E] anchor=`O1:L10-L11` (ledger-line: DEEP-O-001.discovery.md:L161): A free-form `custom topic` kind is forbidden; novel kinds ship as code.
- **O-001/O1-E027** [E] [SURFACE] anchor=`O1:L10` (ledger-line: DEEP-O-001.discovery.md:L162): The per-topic workflow begins with `fetch`.
- **O-001/O1-E028** [E] anchor=`O1:L10` (ledger-line: DEEP-O-001.discovery.md:L163): Fetch output is classified through an enum-gated classifier.
- **O-001/O1-E029** [E] anchor=`O1:L10` (ledger-line: DEEP-O-001.discovery.md:L164): Classification prepends an untrusted-content preamble.
- **O-001/O1-E030** [E] anchor=`O1:L10` (ledger-line: DEEP-O-001.discovery.md:L165): Classification fences each item.
- **O-001/O1-E031** [E] anchor=`O1:L10` (ledger-line: DEEP-O-001.discovery.md:L166): Classification uses a per-kind schema.
- **O-001/O1-E032** [E] anchor=`O1:L10` (ledger-line: DEEP-O-001.discovery.md:L167): Injection canaries are frozen fixtures.
- **O-001/O1-E033** [E] anchor=`O1:L10` (ledger-line: DEEP-O-001.discovery.md:L168): Classification is followed by a duplicate precheck against a rejected ledger.
- **O-001/O1-E034** [E] anchor=`O1:L10` (ledger-line: DEEP-O-001.discovery.md:L169): The pipeline then files accepted material.
- **O-001/O1-E035** [E] anchor=`O1:L10` (ledger-line: DEEP-O-001.discovery.md:L170): The pipeline builds a layered digest.
- **O-001/O1-E036** [E] anchor=`O1:L10` (ledger-line: DEEP-O-001.discovery.md:L171): The full-scope pipeline maintains living leaderboard pages using guarded upsert.
- **O-001/O1-E037** [E] anchor=`O1:L10` (ledger-line: DEEP-O-001.discovery.md:L172): G-1 taint provenance labels fetched bytes as untrusted content.
- **O-001/O1-E038** [E] anchor=`O1:L10` (ledger-line: DEEP-O-001.discovery.md:L173): Existing G-1 policy gate-blocks untrusted-content flows to destructive sinks.
- **O-001/O1-E039** [E] anchor=`O1:L10` (ledger-line: DEEP-O-001.discovery.md:L174): F-3’s tool filter is applied to the research role.
- **O-001/O1-E040** [E] anchor=`O1:L10` (ledger-line: DEEP-O-001.discovery.md:L175): External bytes are recorded in a G-2 VCR cassette.
- **O-001/O1-E041** [E] anchor=`O1:L10` (ledger-line: DEEP-O-001.discovery.md:L176): VCR capture makes a research run `RRA`-replayable.
- **O-001/O1-E042** [E] anchor=`O1:L10` (ledger-line: DEEP-O-001.discovery.md:L177): The claimed ELAI-over-TEIKOKU delta is canaries plus taint plus replay.
- **O-001/O1-E043** [E] anchor=`O1:L11` (ledger-line: DEEP-O-001.discovery.md:L178): Re-authoring `CosmosStage` is excluded; the unit must extend it.
- **O-001/O1-E044** [E] anchor=`O1:L11` (ledger-line: DEEP-O-001.discovery.md:L179): Re-authoring the briefing compiler is excluded.
- **O-001/O1-E045** [E] anchor=`O1:L11` (ledger-line: DEEP-O-001.discovery.md:L180): A classifier output reaching a prompt may not be free text.
- **O-001/O1-E046** [E] anchor=`O1:L11` (ledger-line: DEEP-O-001.discovery.md:L181): Classifier output is a closed enum plus confidence.
- **O-001/O1-E047** [E] anchor=`O1:L13-L14` (ledger-line: DEEP-O-001.discovery.md:L182): Existing asserted implementation includes roughly `700 LOC` of `cosmos_worker`.
- **O-001/O1-E048** [E] [SURFACE] anchor=`O1:L14` (ledger-line: DEEP-O-001.discovery.md:L183): Existing asserted symbols/surfaces include `VerificationMethod::CosmosCheck`, `CosmosStage`, `CosmosDrawer`, and the `live_cli` tick.
- **O-001/O1-E049** [E] anchor=`O1:L14` (ledger-line: DEEP-O-001.discovery.md:L184): B-42/C-16 classify the existing Cosmos artifact tier as `[WIRED]`.
- **O-001/O1-E050** [E] anchor=`O1:L15` (ledger-line: DEEP-O-001.discovery.md:L185): Existing hourly refresh runs from task_runner’s `tokio::interval` under `C-9`.
- **O-001/O1-E051** [E] anchor=`O1:L15` (ledger-line: DEEP-O-001.discovery.md:L186): `FlashResearch` refresh is the B-40 path.
- **O-001/O1-E052** [E] anchor=`O1:L16` (ledger-line: DEEP-O-001.discovery.md:L187): G-1 taint, F-3 tool filtering, and G-2 VCR/RRA are treated as shipped surfaces to consume rather than rebuild.
- **O-001/O1-E053** [E] anchor=`O1:L17` (ledger-line: DEEP-O-001.discovery.md:L188): TEIKOKU’s classifier/canary/confidence-per-sink pattern is asserted live-proven with `3 injections neutralized` and `0 out-of-enum`.
- **O-001/O1-E054** [E] anchor=`O1:L19-L20` (ledger-line: DEEP-O-001.discovery.md:L189): Tier 1 ships the typed topic manifest.
- **O-001/O1-E055** [E] anchor=`O1:L20` (ledger-line: DEEP-O-001.discovery.md:L190): Tier 1 ships enum-gated classification and its confidence gate.
- **O-001/O1-E056** [E] anchor=`O1:L20` (ledger-line: DEEP-O-001.discovery.md:L191): Tier 1 ships duplicate precheck and taint/tool-filter/VCR threading.
- **O-001/O1-E057** [E] anchor=`O1:L20` (ledger-line: DEEP-O-001.discovery.md:L192): Tier 1 supports exactly `2 of the 4` topic kinds: `steal-scan` and `monitor`.
- **O-001/O1-E058** [E] anchor=`O1:L21` (ledger-line: DEEP-O-001.discovery.md:L193): Tier 2 defers `leaderboard` and `discovery`.
- **O-001/O1-E059** [E] [SURFACE] anchor=`O1:L21` (ledger-line: DEEP-O-001.discovery.md:L194): Tier 2 defers living-leaderboard upsert pages until O-3’s vault supplies the page surface.
- **O-001/O1-E060** [E] anchor=`O1:L23-L24` (ledger-line: DEEP-O-001.discovery.md:L195): The intended backend owner is `crates/knowledge` or the live location of `cosmos_worker`.
- **O-001/O1-E061** [E] anchor=`O1:L24` (ledger-line: DEEP-O-001.discovery.md:L196): Backend integration generalizes the existing manifest loader.
- **O-001/O1-E062** [E] anchor=`O1:L24` (ledger-line: DEEP-O-001.discovery.md:L197): Backend integration adds the typed-topic schema and enum classify stage.
- **O-001/O1-E063** [E] anchor=`O1:L24` (ledger-line: DEEP-O-001.discovery.md:L198): G-1/F-3/G-2 are consumed specifically at fetch, classify, and sink seams.
- **O-001/O1-E064** [E] [SURFACE] anchor=`O1:L25` (ledger-line: DEEP-O-001.discovery.md:L199): `CosmosDrawer` is extended to show topic kind.
- **O-001/O1-E065** [E] [SURFACE] anchor=`O1:L25` (ledger-line: DEEP-O-001.discovery.md:L200): `CosmosDrawer` is extended to show last-run trust tier.
- **O-001/O1-E066** [E] [SURFACE] anchor=`O1:L25` (ledger-line: DEEP-O-001.discovery.md:L201): A new drawer must not be built.
- **O-001/O1-E067** [E] anchor=`O1:L27-L28` (ledger-line: DEEP-O-001.discovery.md:L202): Move M1 replaces the designer-brief manifest struct with the closed-kind typed-topic schema while preserving the loader signature so `CosmosStage` remains unchanged.
- **O-001/O1-E068** [E] anchor=`O1:L29` (ledger-line: DEEP-O-001.discovery.md:L203): M1 acceptance requires `[[research.topics]] kind = "steal-scan"` to load and be enumerated by the daemon.
- **O-001/O1-E069** [E] anchor=`O1:L29` (ledger-line: DEEP-O-001.discovery.md:L204): An unknown topic `kind` must hard-refuse parsing rather than default.
- **O-001/O1-E070** [E] anchor=`O1:L30` (ledger-line: DEEP-O-001.discovery.md:L205): If an unknown kind defaults to a catch-all, the fix is deny-unknown enum deserialization plus a negative parse test.
- **O-001/O1-E071** [E] anchor=`O1:L31` (ledger-line: DEEP-O-001.discovery.md:L206): Move M2 requires fetched bytes to be taint-wrapped before reaching the classify prompt.
- **O-001/O1-E072** [E] anchor=`O1:L31` (ledger-line: DEEP-O-001.discovery.md:L207): M2’s classifier emits exactly the shaped object `{class: enum, confidence: f32}`, never free text.
- **O-001/O1-E073** [E] anchor=`O1:L32` (ledger-line: DEEP-O-001.discovery.md:L208): A canary-injection fixture must route to `unsorted` at low confidence.
- **O-001/O1-E074** [E] anchor=`O1:L32` (ledger-line: DEEP-O-001.discovery.md:L209): That canary path must emit a taint-labelled SSB row.
- **O-001/O1-E075** [E] anchor=`O1:L32` (ledger-line: DEEP-O-001.discovery.md:L210): Acceptance requires zero out-of-enum classifier outputs.
- **O-001/O1-E076** [E] anchor=`O1:L33` (ledger-line: DEEP-O-001.discovery.md:L211): If a canary reaches the prompt unlabelled, classification must be re-anchored to consume only the taint-wrapped item.
- **O-001/O1-E077** [E] anchor=`O1:L33` (ledger-line: DEEP-O-001.discovery.md:L212): The failed-canary remedy freezes the canary in a gate-manifest fixture under `R8`.
- **O-001/O1-E078** [E] anchor=`O1:L34` (ledger-line: DEEP-O-001.discovery.md:L213): Move M3 records external bytes in the G-2 VCR cassette keyed by run ID.
- **O-001/O1-E079** [E] anchor=`O1:L35` (ledger-line: DEEP-O-001.discovery.md:L214): Replay acceptance is byte-identical output from the cassette with the network detached.
- **O-001/O1-E080** [E] anchor=`O1:L36` (ledger-line: DEEP-O-001.discovery.md:L215): Replay divergence means an unrecorded network read leaked into the run.
- **O-001/O1-E081** [E] [SURFACE] anchor=`O1:L36` (ledger-line: DEEP-O-001.discovery.md:L216): Replay-divergence remediation routes every remaining fetch through the VCR client and adds a detached-network replay assertion.
- **O-001/O1-E082** [E] anchor=`O1:L38-L39` (ledger-line: DEEP-O-001.discovery.md:L217): If the live manifest is hard-inlined rather than swappable, in-place generalization stops.
- **O-001/O1-E083** [E] anchor=`O1:L39` (ledger-line: DEEP-O-001.discovery.md:L218): That fork creates a new topic loader and adapter into the existing `CosmosStage`, records the seam in the decision document, and does not fork the worker loop.
- **O-001/O1-E084** [E] anchor=`O1:L41-L42` (ledger-line: DEEP-O-001.discovery.md:L219): Before editing, recon must establish the exact manifest type and whether classification already exists in ELAI or only TEIKOKU.
- **O-001/O1-E085** [E] anchor=`O1:L42` (ledger-line: DEEP-O-001.discovery.md:L220): That recon uses the stated Cosmos grep plus `codegraph_explore` on `cosmos_worker`.
- **O-001/O1-E086** [E] anchor=`O1:L43` (ledger-line: DEEP-O-001.discovery.md:L221): Before editing, recon must find whether G-1 exposes an untrusted-byte wrapping API and read its one call site.
- **O-001/O1-E087** [E] anchor=`O1:L45-L46` (ledger-line: DEEP-O-001.discovery.md:L222): If generalization requires editing the frozen G-2 cassette format or G-1 taint gate, execution aborts and flags the issue.
- **O-001/O1-E088** [E] anchor=`O1:L46` (ledger-line: DEEP-O-001.discovery.md:L223): The daemon consumes trust gates and may never modify them.
- **O-001/O1-E089** [E] anchor=`O1:L48-L50` (ledger-line: DEEP-O-001.discovery.md:L224): Produced symbols include the closed-kind research-topic manifest schema and loader.
- **O-001/O1-E090** [E] anchor=`O1:L50-L52` (ledger-line: DEEP-O-001.discovery.md:L225): Produced symbols include enum-gated classification with untrusted preamble, item fence, per-kind schema, and `{class,confidence}` output.
- **O-001/O1-E091** [E] anchor=`O1:L52` (ledger-line: DEEP-O-001.discovery.md:L226): Produced symbols include duplicate precheck against the rejected ledger.
- **O-001/O1-E092** [E] anchor=`O1:L53-L55` (ledger-line: DEEP-O-001.discovery.md:L227): Consumed symbols are `cosmos_worker`, `CosmosStage`, the briefing compiler, G-1 taint, F-3 tool filter, and G-2 VCR.
- **O-001/O1-E093** [E] [SURFACE] anchor=`O1:L56-L58` (ledger-line: DEEP-O-001.discovery.md:L228): Production reachability is the live `live_cli` Cosmos tick driving a configured topic end to end.
- **O-001/O1-E094** [E] anchor=`O1:L59-L63` (ledger-line: DEEP-O-001.discovery.md:L229): No fetch path may bypass G-1 taint or G-2 cassette recording.
- **O-001/O1-E095** [E] anchor=`O1:L64-L66` (ledger-line: DEEP-O-001.discovery.md:L230): The observable trigger is a live daemon tick for a `steal-scan` topic pointed at a canary-injection fixture.
- **O-001/O1-E096** [E] anchor=`O1:L66` (ledger-line: DEEP-O-001.discovery.md:L231): The observable classify SSB row carries `{class, confidence, taint_tier}`.
- **O-001/O1-E097** [E] anchor=`O1:L66` (ledger-line: DEEP-O-001.discovery.md:L232): The canary must land in `unsorted` and never a steal sink.
- **O-001/O1-E098** [E] anchor=`O1:L67` (ledger-line: DEEP-O-001.discovery.md:L233): The resulting briefing must remain `<=1500` tokens and include provenance tiers.
- **O-001/O1-E099** [E] anchor=`O1:L67` (ledger-line: DEEP-O-001.discovery.md:L234): Out-of-enum output or a canary reaching a steal sink fails the run.
- **O-001/O1-E100** [E] [SURFACE] anchor=`O1:L69` (ledger-line: DEEP-O-001.discovery.md:L235): The production hook is the C-16 Cosmos tick in `live_cli`; the exact function is named at recon and wired in the same changeset.
- **O-001/O1-E101** [E] anchor=`O1:L71-L74` (ledger-line: DEEP-O-001.discovery.md:L236): Verification runs `wiring_contract_check.py` with this plan and `--run-trigger`.
- **O-001/O1-E102** [E] anchor=`O1:L74` (ledger-line: DEEP-O-001.discovery.md:L237): Verification runs `production_caller_check.py --files <changed .rs> --json`; a dark new public function blocks completion.
- **O-001/O1-E103** [E] anchor=`O1:L75` (ledger-line: DEEP-O-001.discovery.md:L238): Verification passes only when the taint-tier classify row lands off-agent, the canary routes to `unsorted`, and the daemon tick—not a test—is the production caller.
- **O-001/O1-E104** [E] anchor=`O1:L77` (ledger-line: DEEP-O-001.discovery.md:L239): The integration smoke is run by an independent agent through the real Cosmos tick.
- **O-001/O1-E105** [E] anchor=`O1:L77` (ledger-line: DEEP-O-001.discovery.md:L240): Its hard input is a live, multi-item, long-enough source containing a paraphrased prompt injection so confidence-gate and duplicate-precheck paths fire.
- **O-001/O1-E106** [E] anchor=`O1:L77` (ledger-line: DEEP-O-001.discovery.md:L241): A toy single-item happy path does not satisfy `R10`.
- **O-001/O1-E107** [E] anchor=`O1:L79-L80` (ledger-line: DEEP-O-001.discovery.md:L242): The smuggled-instruction red-team attack is intended to fail because classification is closed, confidence-bearing, taint-labelled to every sink, and protected by a frozen canary fixture.
- **O-001/O1-E108** [E] anchor=`O1:L80` (ledger-line: DEEP-O-001.discovery.md:L243): The successful generic-engine attack—adding a free-text topic kind to skip code—is patched by deny-unknown kind parsing.
- **O-001/O1-E109** [E] anchor=`O1:L82-L88` (ledger-line: DEEP-O-001.discovery.md:L244): O-1 composes with `CosmosStage`, `cosmos_worker`, briefing compiler, G-1, F-3, G-2/RRA, and B-40 `FlashResearch`, and reimplements none.
- **O-001/O1-E110** [E] [SURFACE] anchor=`O1:L87-L88` (ledger-line: DEEP-O-001.discovery.md:L245): O-2 supplies scheduling, O-3 supplies the vault sink, and O-4 supplies the canvas surface around O-1.
- **O-001/O1-E111** [E] anchor=`O1:L56-L58` (ledger-line: DEEP-O-001.discovery.md:L246): A `#[cfg(test)]` caller never satisfies O-1 production reachability.

### Unit `O2` — O-2: scheduled TaskSpec substrate — typed, journaled, rules-as-data schedules over the closed stage vocabulary — from O-001 (112 records)

- **O-001/O2-E001** [E] anchor=`O2:L1-L3` (ledger-line: DEEP-O-001.discovery.md:L250): `O-2` is `PLANNED`, Wave `1`, product-surface class, `PROOF_SPINE: none`, and owner-acknowledged `2026-07-06`.
- **O-001/O2-E002** [E] anchor=`O2:L5` (ledger-line: DEEP-O-001.discovery.md:L251): Metadata is Track `RESEARCH-MACHINE`, lever `rules-as-data-scheduling`, source finding `L3`, grounding `MODERATE (index-derived, PLAN-asserted)`, nature `BUILD-on-shipped`, effort `M-L`, Wave `1`.
- **O-001/O2-E003** [E] anchor=`O2:L7` (ledger-line: DEEP-O-001.discovery.md:L252): The borrowed Codex Automations shape supports time schedules.
- **O-001/O2-E004** [E] anchor=`O2:L7` (ledger-line: DEEP-O-001.discovery.md:L253): The borrowed shape supports event schedules.
- **O-001/O2-E005** [E] [SURFACE] anchor=`O2:L7` (ledger-line: DEEP-O-001.discovery.md:L254): Scheduled results land in an inbox.
- **O-001/O2-E006** [E] anchor=`O2:L7` (ledger-line: DEEP-O-001.discovery.md:L255): Runs with nothing to report auto-archive.
- **O-001/O2-E007** [E] anchor=`O2:L7` (ledger-line: DEEP-O-001.discovery.md:L256): ELAI’s version is explicitly typed.
- **O-001/O2-E008** [E] anchor=`O2:L7` (ledger-line: DEEP-O-001.discovery.md:L257): ELAI’s version is explicitly journaled.
- **O-001/O2-E009** [E] anchor=`O2:L7` (ledger-line: DEEP-O-001.discovery.md:L258): A `TaskSpec` is stored as data in SQLite using the rules-engine precedent.
- **O-001/O2-E010** [E] anchor=`O2:L7` (ledger-line: DEEP-O-001.discovery.md:L259): Every fire is an SSB row carrying its trigger cause.
- **O-001/O2-E011** [E] anchor=`O2:L7` (ledger-line: DEEP-O-001.discovery.md:L260): Triggers are driven by the existing task_runner plus the SSB temporal monitor.
- **O-001/O2-E012** [E] [SURFACE] anchor=`O2:L7` (ledger-line: DEEP-O-001.discovery.md:L261): O-2 supplies the scheduler used by O-1’s daemon and edited by O-4’s canvas.
- **O-001/O2-E013** [E] anchor=`O2:L7` (ledger-line: DEEP-O-001.discovery.md:L262): O-2 invents only the typed spec and trigger wiring.
- **O-001/O2-E014** [E] anchor=`O2:L9-L10` (ledger-line: DEEP-O-001.discovery.md:L263): `TaskSpec.trigger` is a closed enum.
- **O-001/O2-E015** [E] anchor=`O2:L10` (ledger-line: DEEP-O-001.discovery.md:L264): `Cron{expr}` is a trigger variant with an `expr` operand.
- **O-001/O2-E016** [E] anchor=`O2:L10` (ledger-line: DEEP-O-001.discovery.md:L265): `SsbEvent{action_type filter}` is a trigger variant with an action-type filter.
- **O-001/O2-E017** [E] anchor=`O2:L10` (ledger-line: DEEP-O-001.discovery.md:L266): `Manual` is a trigger variant.
- **O-001/O2-E018** [E] anchor=`O2:L10` (ledger-line: DEEP-O-001.discovery.md:L267): `TaskSpec.pipeline` is an ordered list.
- **O-001/O2-E019** [E] anchor=`O2:L10` (ledger-line: DEEP-O-001.discovery.md:L268): Pipeline stage kinds are a closed enum.
- **O-001/O2-E020** [E] anchor=`O2:L10` (ledger-line: DEEP-O-001.discovery.md:L269): `Fetch` is a stage kind.
- **O-001/O2-E021** [E] anchor=`O2:L10` (ledger-line: DEEP-O-001.discovery.md:L270): `Classify` is a stage kind.
- **O-001/O2-E022** [E] anchor=`O2:L10` (ledger-line: DEEP-O-001.discovery.md:L271): `Gate` is a stage kind.
- **O-001/O2-E023** [E] anchor=`O2:L10` (ledger-line: DEEP-O-001.discovery.md:L272): `FileToBrain` is a stage kind.
- **O-001/O2-E024** [E] anchor=`O2:L10` (ledger-line: DEEP-O-001.discovery.md:L273): `Digest` is a stage kind.
- **O-001/O2-E025** [E] anchor=`O2:L10` (ledger-line: DEEP-O-001.discovery.md:L274): `SpawnResearchTree` is a stage kind.
- **O-001/O2-E026** [E] anchor=`O2:L10` (ledger-line: DEEP-O-001.discovery.md:L275): `RunPlanUnit` is a stage kind.
- **O-001/O2-E027** [E] anchor=`O2:L10` (ledger-line: DEEP-O-001.discovery.md:L276): `Notify` is a stage kind.
- **O-001/O2-E028** [E] anchor=`O2:L10` (ledger-line: DEEP-O-001.discovery.md:L277): `TaskSpec.sink` is a closed enum.
- **O-001/O2-E029** [E] [SURFACE] anchor=`O2:L10` (ledger-line: DEEP-O-001.discovery.md:L278): `VaultPage` is a sink variant.
- **O-001/O2-E030** [E] anchor=`O2:L10` (ledger-line: DEEP-O-001.discovery.md:L279): `Digest` is a sink variant as well as a stage name.
- **O-001/O2-E031** [E] [SURFACE] anchor=`O2:L10` (ledger-line: DEEP-O-001.discovery.md:L280): `DashboardInbox` is a sink variant.
- **O-001/O2-E032** [E] [SURFACE] anchor=`O2:L10` (ledger-line: DEEP-O-001.discovery.md:L281): `Toast` is a sink variant.
- **O-001/O2-E033** [E] anchor=`O2:L10` (ledger-line: DEEP-O-001.discovery.md:L282): TaskSpec rows mirror the rules-engine predicates-as-data persistence pattern.
- **O-001/O2-E034** [E] anchor=`O2:L10` (ledger-line: DEEP-O-001.discovery.md:L283): The wall-clock trigger is supervisor-plane and outside the replay set under `K-14`.
- **O-001/O2-E035** [E] anchor=`O2:L10` (ledger-line: DEEP-O-001.discovery.md:L284): Given recorded inputs, the actual TaskSpec run must remain deterministic.
- **O-001/O2-E036** [E] anchor=`O2:L10` (ledger-line: DEEP-O-001.discovery.md:L285): Missed-run detection reuses the SSB temporal-monitor engine.
- **O-001/O2-E037** [E] [SURFACE] anchor=`O2:L10` (ledger-line: DEEP-O-001.discovery.md:L286): The existing dashboard intervention-card surface serves as the inbox sink.
- **O-001/O2-E038** [E] anchor=`O2:L11` (ledger-line: DEEP-O-001.discovery.md:L287): Free-form or custom-node stage kinds are forbidden.
- **O-001/O2-E039** [E] anchor=`O2:L11` (ledger-line: DEEP-O-001.discovery.md:L288): New stage kinds must ship as code through the normal plan process.
- **O-001/O2-E040** [E] anchor=`O2:L11` (ledger-line: DEEP-O-001.discovery.md:L289): A wall-clock read inside the replay set is forbidden.
- **O-001/O2-E041** [E] anchor=`O2:L11` (ledger-line: DEEP-O-001.discovery.md:L290): A second scheduler loop is forbidden; the task_runner interval must be reused.
- **O-001/O2-E042** [E] [SURFACE] anchor=`O2:L11` (ledger-line: DEEP-O-001.discovery.md:L291): A second inbox surface is forbidden; intervention cards must be reused.
- **O-001/O2-E043** [E] anchor=`O2:L13-L14` (ledger-line: DEEP-O-001.discovery.md:L292): Existing ground truth asserts rules predicates already persist as SQLite data under `3F`.
- **O-001/O2-E044** [E] anchor=`O2:L15` (ledger-line: DEEP-O-001.discovery.md:L293): Existing ground truth asserts task_runner periodic work is driven by `tokio::interval` under `C-9`.
- **O-001/O2-E045** [E] anchor=`O2:L16` (ledger-line: DEEP-O-001.discovery.md:L294): Existing ground truth asserts SSB is the journal.
- **O-001/O2-E046** [E] anchor=`O2:L16` (ledger-line: DEEP-O-001.discovery.md:L295): Existing ground truth asserts an SSB temporal-monitor already handles missed/late detection.
- **O-001/O2-E047** [E] [SURFACE] anchor=`O2:L17` (ledger-line: DEEP-O-001.discovery.md:L296): Existing ground truth asserts intervention cards already support human-in-the-loop inbox cards.
- **O-001/O2-E048** [E] anchor=`O2:L19-L20` (ledger-line: DEEP-O-001.discovery.md:L297): Tier 1 ships the `TaskSpec` schema and SQLite store.
- **O-001/O2-E049** [E] anchor=`O2:L20` (ledger-line: DEEP-O-001.discovery.md:L298): Tier 1 ships `Cron` and `Manual` triggers.
- **O-001/O2-E050** [E] anchor=`O2:L20` (ledger-line: DEEP-O-001.discovery.md:L299): Tier 1 ships journaled fire rows.
- **O-001/O2-E051** [E] [SURFACE] anchor=`O2:L20` (ledger-line: DEEP-O-001.discovery.md:L300): Tier 1 ships the `DashboardInbox` sink.
- **O-001/O2-E052** [E] anchor=`O2:L20` (ledger-line: DEEP-O-001.discovery.md:L301): Tier 1 stage support is `Fetch`, `Classify`, `Gate`, and `Digest`, described as the O-1-provided stages.
- **O-001/O2-E053** [E] anchor=`O2:L21` (ledger-line: DEEP-O-001.discovery.md:L302): Tier 2 defers the `SsbEvent` trigger.
- **O-001/O2-E054** [E] anchor=`O2:L21` (ledger-line: DEEP-O-001.discovery.md:L303): Tier 2 defers `SpawnResearchTree` and `RunPlanUnit` until an orchestrator-FSM wiring pass.
- **O-001/O2-E055** [E] anchor=`O2:L21` (ledger-line: DEEP-O-001.discovery.md:L304): Tier 2 defers the missed-run detection hookup.
- **O-001/O2-E056** [E] anchor=`O2:L23-L24` (ledger-line: DEEP-O-001.discovery.md:L305): Backend ownership is `crates/orchestrator` or the rules-engine crate.
- **O-001/O2-E057** [E] anchor=`O2:L24` (ledger-line: DEEP-O-001.discovery.md:L306): Backend integration adds the TaskSpec table and loader.
- **O-001/O2-E058** [E] anchor=`O2:L24` (ledger-line: DEEP-O-001.discovery.md:L307): Backend integration adds the trigger dispatcher to the task_runner interval.
- **O-001/O2-E059** [E] anchor=`O2:L24` (ledger-line: DEEP-O-001.discovery.md:L308): Backend integration emits the SSB fire row.
- **O-001/O2-E060** [E] [SURFACE] anchor=`O2:L25` (ledger-line: DEEP-O-001.discovery.md:L309): Dashboard intervention cards render the `DashboardInbox` sink.
- **O-001/O2-E061** [E] [SURFACE] anchor=`O2:L25` (ledger-line: DEEP-O-001.discovery.md:L310): O-2 does not add a spec list/edit panel; O-4 owns that canvas surface.
- **O-001/O2-E062** [E] anchor=`O2:L27-L28` (ledger-line: DEEP-O-001.discovery.md:L311): Move M1 adds the TaskSpec table, strict closed-trigger/stage deserializer, and a boot-time loader.
- **O-001/O2-E063** [E] anchor=`O2:L29` (ledger-line: DEEP-O-001.discovery.md:L312): A Cron TaskSpec row must load and be enumerated by the dispatcher.
- **O-001/O2-E064** [E] anchor=`O2:L29` (ledger-line: DEEP-O-001.discovery.md:L313): Unknown stage kinds hard-refuse parsing.
- **O-001/O2-E065** [E] anchor=`O2:L29` (ledger-line: DEEP-O-001.discovery.md:L314): Unknown trigger variants hard-refuse parsing.
- **O-001/O2-E066** [E] anchor=`O2:L30` (ledger-line: DEEP-O-001.discovery.md:L315): If either unknown value is accepted, the fix is deny-unknown strictness plus a negative parse test.
- **O-001/O2-E067** [E] anchor=`O2:L31` (ledger-line: DEEP-O-001.discovery.md:L316): Move M2 wires Cron onto the existing task_runner interval.
- **O-001/O2-E068** [E] anchor=`O2:L31` (ledger-line: DEEP-O-001.discovery.md:L317): A due spec fires its ordered pipeline.
- **O-001/O2-E069** [E] anchor=`O2:L31` (ledger-line: DEEP-O-001.discovery.md:L318): That fire emits a journaled SSB row with its trigger cause.
- **O-001/O2-E070** [E] anchor=`O2:L32` (ledger-line: DEEP-O-001.discovery.md:L319): At a due tick, a spec must fire exactly once.
- **O-001/O2-E071** [E] anchor=`O2:L32` (ledger-line: DEEP-O-001.discovery.md:L320): The SSB action type is `task_spec_fired`.
- **O-001/O2-E072** [E] anchor=`O2:L32` (ledger-line: DEEP-O-001.discovery.md:L321): `task_spec_fired` carries `spec_id`.
- **O-001/O2-E073** [E] anchor=`O2:L32` (ledger-line: DEEP-O-001.discovery.md:L322): `task_spec_fired` carries `trigger_cause`.
- **O-001/O2-E074** [E] anchor=`O2:L32` (ledger-line: DEEP-O-001.discovery.md:L323): `task_spec_fired` carries `stage_path`.
- **O-001/O2-E075** [E] anchor=`O2:L33` (ledger-line: DEEP-O-001.discovery.md:L324): Double-fire or non-fire is remediated by gating on a last-fired watermark row.
- **O-001/O2-E076** [E] anchor=`O2:L33` (ledger-line: DEEP-O-001.discovery.md:L325): The last-fired watermark is explicitly intended to make due-tick dispatch idempotent.
- **O-001/O2-E077** [E] anchor=`O2:L33` (ledger-line: DEEP-O-001.discovery.md:L326): The idempotency path requires a double-fire assertion.
- **O-001/O2-E078** [E] [SURFACE] anchor=`O2:L34` (ledger-line: DEEP-O-001.discovery.md:L327): Move M3 routes terminal `Notify`/sink output to the intervention-card inbox.
- **O-001/O2-E079** [E] [SURFACE] anchor=`O2:L35` (ledger-line: DEEP-O-001.discovery.md:L328): A fire with results renders an inbox card.
- **O-001/O2-E080** [E] anchor=`O2:L34-L35` (ledger-line: DEEP-O-001.discovery.md:L329): A nothing-to-report fire auto-archives and leaves only its SSB row.
- **O-001/O2-E081** [E] [SURFACE] anchor=`O2:L36` (ledger-line: DEEP-O-001.discovery.md:L330): Empty-inbox spam is remediated by a result-count-based `nothing-to-report -> archive` branch.
- **O-001/O2-E082** [E] anchor=`O2:L38-L39` (ledger-line: DEEP-O-001.discovery.md:L331): If the temporal monitor lacks a due/overdue query, missed-run detection moves to Tier 2.
- **O-001/O2-E083** [E] anchor=`O2:L39` (ledger-line: DEEP-O-001.discovery.md:L332): Under that fork, Tier 1 ships fire-on-tick only and records the seam; it must not build a second monitor.
- **O-001/O2-E084** [E] anchor=`O2:L41-L42` (ledger-line: DEEP-O-001.discovery.md:L333): Before editing, recon must determine the rules table/loader shape TaskSpec will mirror through the stated grep and codegraph exploration.
- **O-001/O2-E085** [E] anchor=`O2:L43` (ledger-line: DEEP-O-001.discovery.md:L334): Recon must determine whether task_runner accepts registered periodic jobs or needs extension, then read its registration site.
- **O-001/O2-E086** [E] [SURFACE] anchor=`O2:L44` (ledger-line: DEEP-O-001.discovery.md:L335): Recon must determine the intervention-card ingestion API.
- **O-001/O2-E087** [E] anchor=`O2:L46-L47` (ledger-line: DEEP-O-001.discovery.md:L336): If Cron wiring requires moving wall-clock reads into the replay/journal set, execution aborts and flags the violation.
- **O-001/O2-E088** [E] anchor=`O2:L47` (ledger-line: DEEP-O-001.discovery.md:L337): Trigger time is an input recorded once, never a replayed pipeline step.
- **O-001/O2-E089** [E] anchor=`O2:L49-L51` (ledger-line: DEEP-O-001.discovery.md:L338): Produced symbols include the closed trigger, stage-kind, and sink enums plus table and loader.
- **O-001/O2-E090** [E] anchor=`O2:L52` (ledger-line: DEEP-O-001.discovery.md:L339): Produced symbols include Cron/Manual dispatch on the task_runner interval.
- **O-001/O2-E091** [E] anchor=`O2:L53` (ledger-line: DEEP-O-001.discovery.md:L340): Produced symbols include `task_spec_fired` and auto-archive-on-empty.
- **O-001/O2-E092** [E] [SURFACE] anchor=`O2:L54-L56` (ledger-line: DEEP-O-001.discovery.md:L341): Consumed mechanisms are `3F`, `C-9`, SSB/K-14, temporal monitoring, intervention cards, and O-1 stages.
- **O-001/O2-E093** [E] anchor=`O2:L57-L59` (ledger-line: DEEP-O-001.discovery.md:L342): Production reachability is a due Cron TaskSpec fired by a live task_runner interval.
- **O-001/O2-E094** [E] anchor=`O2:L60-L64` (ledger-line: DEEP-O-001.discovery.md:L343): No fire is permitted without a journaled trigger-cause SSB row.
- **O-001/O2-E095** [E] [SURFACE] anchor=`O2:L65-L67` (ledger-line: DEEP-O-001.discovery.md:L344): The observable trigger registers a due-now Cron TaskSpec with a `Digest->DashboardInbox` pipeline and advances the task_runner tick.
- **O-001/O2-E096** [E] anchor=`O2:L67` (ledger-line: DEEP-O-001.discovery.md:L345): The observable evidence is exactly one `{spec_id, trigger_cause=Cron, stage_path}` fire row per due tick.
- **O-001/O2-E097** [E] [SURFACE] anchor=`O2:L68` (ledger-line: DEEP-O-001.discovery.md:L346): A result-bearing fire creates an intervention-card entry.
- **O-001/O2-E098** [E] anchor=`O2:L68` (ledger-line: DEEP-O-001.discovery.md:L347): An empty fire that creates a card or any double-fire fails.
- **O-001/O2-E099** [E] anchor=`O2:L70` (ledger-line: DEEP-O-001.discovery.md:L348): The production hook is the `C-9` interval-registration seam; the exact function is named at recon and wired in the same changeset.
- **O-001/O2-E100** [E] anchor=`O2:L72-L75` (ledger-line: DEEP-O-001.discovery.md:L349): Verification runs `wiring_contract_check.py` against this plan with `--run-trigger`.
- **O-001/O2-E101** [E] anchor=`O2:L75` (ledger-line: DEEP-O-001.discovery.md:L350): Verification runs `production_caller_check.py --files <changed .rs> --json`; a dark public function blocks.
- **O-001/O2-E102** [E] anchor=`O2:L76` (ledger-line: DEEP-O-001.discovery.md:L351): Verification passes only after a real task_runner tick yields one off-agent fire row, a card for results, and auto-archive for empty output.
- **O-001/O2-E103** [E] anchor=`O2:L78` (ledger-line: DEEP-O-001.discovery.md:L352): The independent integration smoke runs a full `Fetch->Classify->Gate->Digest` chain over a hard source.
- **O-001/O2-E104** [E] anchor=`O2:L78` (ledger-line: DEEP-O-001.discovery.md:L353): The smoke advances several ticks.
- **O-001/O2-E105** [E] [SURFACE] anchor=`O2:L78` (ledger-line: DEEP-O-001.discovery.md:L354): The smoke asserts single-fire journaling and a result-bearing inbox card.
- **O-001/O2-E106** [E] anchor=`O2:L78` (ledger-line: DEEP-O-001.discovery.md:L355): The smoke asserts auto-archive on a subsequent empty tick.
- **O-001/O2-E107** [E] anchor=`O2:L78` (ledger-line: DEEP-O-001.discovery.md:L356): A one-tick smoke is insufficient; `R10` requires a multi-tick run exercising missed/idempotency paths.
- **O-001/O2-E108** [E] anchor=`O2:L80-L81` (ledger-line: DEEP-O-001.discovery.md:L357): The mid-pipeline clock-read red-team attack is intended to fail because supervisor-plane time is recorded and in-replay time reads are banned.
- **O-001/O2-E109** [E] [SURFACE] anchor=`O2:L81` (ledger-line: DEEP-O-001.discovery.md:L358): Inbox flooding from a `1-minute` Cron that reports nothing is patched by auto-archive-on-empty while retaining the SSB audit row.
- **O-001/O2-E110** [E] [SURFACE] anchor=`O2:L83-L89` (ledger-line: DEEP-O-001.discovery.md:L359): O-2 composes with rules-as-data `3F`, task_runner `C-9`, SSB/K-14 monitoring, intervention cards, and O-1 stages, and reimplements none.
- **O-001/O2-E111** [E] [SURFACE] anchor=`O2:L88-L89` (ledger-line: DEEP-O-001.discovery.md:L360): O-1 supplies stages, O-4 renders/edits specs, and O-3 supplies the `VaultPage` sink.
- **O-001/O2-E112** [E] anchor=`O2:L57-L59` (ledger-line: DEEP-O-001.discovery.md:L361): A `#[cfg(test)]` caller never satisfies O-2 production reachability.

### Unit `O3` — O-3: brain vault contract + mounting — portable markdown vault mounted (not converted), with namespace claims — from O-001 (109 records)

- **O-001/O3-E001** [E] anchor=`O3:L1-L3` (ledger-line: DEEP-O-001.discovery.md:L365): `O-3` is `PLANNED`, Wave `2`, product-surface class, `PROOF_SPINE: none`, and owner-acknowledged `2026-07-06`.
- **O-001/O3-E002** [E] anchor=`O3:L5` (ledger-line: DEEP-O-001.discovery.md:L366): Metadata is Track `RESEARCH-MACHINE`, lever `mount-not-convert`, source finding `L2`, grounding `MODERATE (index-derived, PLAN-asserted)`, nature `BUILD-on-shipped`, effort `M`, Wave `2`.
- **O-001/O3-E003** [E] anchor=`O3:L7` (ledger-line: DEEP-O-001.discovery.md:L367): Brain migration is required to cost zero conversion.
- **O-001/O3-E004** [E] anchor=`O3:L7` (ledger-line: DEEP-O-001.discovery.md:L368): The brain is split by storage/use grain.
- **O-001/O3-E005** [E] anchor=`O3:L7` (ledger-line: DEEP-O-001.discovery.md:L369): E-5 `memory_facts` remains the machine-grain fact store.
- **O-001/O3-E006** [E] anchor=`O3:L7` (ledger-line: DEEP-O-001.discovery.md:L370): `memory_facts` continues feeding prompts.
- **O-001/O3-E007** [E] anchor=`O3:L7` (ledger-line: DEEP-O-001.discovery.md:L371): The existing auditor firewall around machine memory remains intact.
- **O-001/O3-E008** [E] [SURFACE] anchor=`O3:L7` (ledger-line: DEEP-O-001.discovery.md:L372): A deterministic Markdown vault is the user-grain surface.
- **O-001/O3-E009** [E] [SURFACE] anchor=`O3:L7` (ledger-line: DEEP-O-001.discovery.md:L373): The owner can read and edit the vault in Obsidian or GDrive.
- **O-001/O3-E010** [E] [SURFACE] anchor=`O3:L7` (ledger-line: DEEP-O-001.discovery.md:L374): The vault format contract comprises Markdown, YAML frontmatter, wikilinks, and machine-owned namespaces.
- **O-001/O3-E011** [E] [SURFACE] anchor=`O3:L7` (ledger-line: DEEP-O-001.discovery.md:L375): ELAI mounts the existing TEIKOKU brain vault rather than importing it.
- **O-001/O3-E012** [E] anchor=`O3:L7` (ledger-line: DEEP-O-001.discovery.md:L376): Namespace claims are the only mechanism described as genuinely new for this unit.
- **O-001/O3-E013** [E] [SURFACE] anchor=`O3:L7` (ledger-line: DEEP-O-001.discovery.md:L377): Namespace claims prevent two harnesses writing one vault from colliding.
- **O-001/O3-E014** [E] anchor=`O3:L9-L10` (ledger-line: DEEP-O-001.discovery.md:L378): Topic pages use `topics/<t>/<slug>.md`.
- **O-001/O3-E015** [E] anchor=`O3:L10` (ledger-line: DEEP-O-001.discovery.md:L379): Pages use YAML frontmatter.
- **O-001/O3-E016** [E] anchor=`O3:L10` (ledger-line: DEEP-O-001.discovery.md:L380): Pages support wikilinks.
- **O-001/O3-E017** [E] anchor=`O3:L10` (ledger-line: DEEP-O-001.discovery.md:L381): Full contract includes living pages under `rankings/`.
- **O-001/O3-E018** [E] anchor=`O3:L10` (ledger-line: DEEP-O-001.discovery.md:L382): Full contract includes living pages under `queue/`.
- **O-001/O3-E019** [E] anchor=`O3:L10` (ledger-line: DEEP-O-001.discovery.md:L383): Full contract includes an `index` living page.
- **O-001/O3-E020** [E] anchor=`O3:L10` (ledger-line: DEEP-O-001.discovery.md:L384): `write_page` is create/append-only.
- **O-001/O3-E021** [E] anchor=`O3:L10` (ledger-line: DEEP-O-001.discovery.md:L385): `upsert_page` is guarded.
- **O-001/O3-E022** [E] anchor=`O3:L10` (ledger-line: DEEP-O-001.discovery.md:L386): `upsert_page` is restricted to machine-owned namespaces.
- **O-001/O3-E023** [E] anchor=`O3:L10` (ledger-line: DEEP-O-001.discovery.md:L387): `upsert_page` may never edit user notes.
- **O-001/O3-E024** [E] [SURFACE] anchor=`O3:L10` (ledger-line: DEEP-O-001.discovery.md:L388): A file named `namespace-claims` lives at the vault root.
- **O-001/O3-E025** [E] anchor=`O3:L10` (ledger-line: DEEP-O-001.discovery.md:L389): The claims file declares which machine owns each namespace.
- **O-001/O3-E026** [E] anchor=`O3:L10` (ledger-line: DEEP-O-001.discovery.md:L390): Every write outside a claimed namespace hard-refuses.
- **O-001/O3-E027** [E] [SURFACE] anchor=`O3:L10` (ledger-line: DEEP-O-001.discovery.md:L391): Namespace isolation is intended to let ELAI and TEIKOKU share one physical vault.
- **O-001/O3-E028** [E] anchor=`O3:L10` (ledger-line: DEEP-O-001.discovery.md:L392): The existing `memory_facts` SQLite store remains unchanged.
- **O-001/O3-E029** [E] anchor=`O3:L10` (ledger-line: DEEP-O-001.discovery.md:L393): Existing `Mem0` dispatch remains unchanged.
- **O-001/O3-E030** [E] anchor=`O3:L10` (ledger-line: DEEP-O-001.discovery.md:L394): Existing machine-memory decay remains unchanged.
- **O-001/O3-E031** [E] anchor=`O3:L10` (ledger-line: DEEP-O-001.discovery.md:L395): Existing `MINJA` defense remains unchanged.
- **O-001/O3-E032** [E] anchor=`O3:L10` (ledger-line: DEEP-O-001.discovery.md:L396): A D-5-style life-layer denylist blocks personal topics from project prompts.
- **O-001/O3-E033** [E] anchor=`O3:L10` (ledger-line: DEEP-O-001.discovery.md:L397): The same privacy boundary blocks personal topics from cloud calls.
- **O-001/O3-E034** [E] [SURFACE] anchor=`O3:L10` (ledger-line: DEEP-O-001.discovery.md:L398): Vault reads route through the existing privacy-redaction/air-gapped layer.
- **O-001/O3-E035** [E] [SURFACE] anchor=`O3:L11` (ledger-line: DEEP-O-001.discovery.md:L399): Converting the TEIKOKU vault into `memory_facts` is excluded.
- **O-001/O3-E036** [E] anchor=`O3:L11` (ledger-line: DEEP-O-001.discovery.md:L400): Editing user-authored notes is excluded; allowed writer shapes are create/append and machine-namespace upsert.
- **O-001/O3-E037** [E] anchor=`O3:L11` (ledger-line: DEEP-O-001.discovery.md:L401): Routing personal-brain content into project prompts is excluded.
- **O-001/O3-E038** [E] anchor=`O3:L11` (ledger-line: DEEP-O-001.discovery.md:L402): A second fact store is excluded.
- **O-001/O3-E039** [E] anchor=`O3:L13-L14` (ledger-line: DEEP-O-001.discovery.md:L403): Existing ground truth asserts `memory_facts` is SQLite with Mem0-style dispatch, decay, and MINJA injection defense.
- **O-001/O3-E040** [E] anchor=`O3:L15` (ledger-line: DEEP-O-001.discovery.md:L404): Existing ground truth asserts a D-5 life-layer denylist and privacy-redaction/air-gapped read layer are shipped.
- **O-001/O3-E041** [E] anchor=`O3:L16` (ledger-line: DEEP-O-001.discovery.md:L405): Existing ground truth asserts TEIKOKU’s writer discipline and rankings/queue/index pages are live-proven.
- **O-001/O3-E042** [E] [SURFACE] anchor=`O3:L18-L19` (ledger-line: DEEP-O-001.discovery.md:L406): Tier 1 ships the vault contract reader and writer for frontmatter, wikilinks, and `topics/<t>/<slug>.md`.
- **O-001/O3-E043** [E] anchor=`O3:L19` (ledger-line: DEEP-O-001.discovery.md:L407): Tier 1 ships guarded create/append.
- **O-001/O3-E044** [E] anchor=`O3:L19` (ledger-line: DEEP-O-001.discovery.md:L408): Tier 1 ships machine-namespace upsert.
- **O-001/O3-E045** [E] anchor=`O3:L19` (ledger-line: DEEP-O-001.discovery.md:L409): Tier 1 ships the root namespace-claims file and hard refusal.
- **O-001/O3-E046** [E] [SURFACE] anchor=`O3:L19` (ledger-line: DEEP-O-001.discovery.md:L410): Tier 1 mounts a configured existing vault path.
- **O-001/O3-E047** [E] [SURFACE] anchor=`O3:L20` (ledger-line: DEEP-O-001.discovery.md:L411): Tier 2 defers D-5 life-layer enforcement on vault reads into prompts until a privacy-layer wiring pass.
- **O-001/O3-E048** [E] [SURFACE] anchor=`O3:L20` (ledger-line: DEEP-O-001.discovery.md:L412): Tier 2 defers bidirectional synchronization between `memory_facts` and vault views.
- **O-001/O3-E049** [E] anchor=`O3:L22-L23` (ledger-line: DEEP-O-001.discovery.md:L413): Backend ownership is the knowledge/memory crate.
- **O-001/O3-E050** [E] [SURFACE] anchor=`O3:L23` (ledger-line: DEEP-O-001.discovery.md:L414): Backend integration adds the vault reader/writer module.
- **O-001/O3-E051** [E] anchor=`O3:L23` (ledger-line: DEEP-O-001.discovery.md:L415): Backend integration adds the namespace-claims parser and guarded-upsert path.
- **O-001/O3-E052** [E] [SURFACE] anchor=`O3:L23` (ledger-line: DEEP-O-001.discovery.md:L416): The mount point resolves a configured vault root.
- **O-001/O3-E053** [E] [SURFACE] anchor=`O3:L24` (ledger-line: DEEP-O-001.discovery.md:L417): A dashboard vault-page view, if present, is read-only.
- **O-001/O3-E054** [E] [SURFACE] anchor=`O3:L24` (ledger-line: DEEP-O-001.discovery.md:L418): The dashboard view may be deferred entirely to Obsidian/GDrive.
- **O-001/O3-E055** [E] [SURFACE] anchor=`O3:L24` (ledger-line: DEEP-O-001.discovery.md:L419): No new ELAI panel is justified because the vault is intentionally an external-editor surface.
- **O-001/O3-E056** [E] anchor=`O3:L26-L27` (ledger-line: DEEP-O-001.discovery.md:L420): Move M1 parses frontmatter and wikilinks under the topic-path layout.
- **O-001/O3-E057** [E] [SURFACE] anchor=`O3:L27` (ledger-line: DEEP-O-001.discovery.md:L421): M1 initially mounts a configured existing vault root read-only.
- **O-001/O3-E058** [E] anchor=`O3:L28` (ledger-line: DEEP-O-001.discovery.md:L422): Mount acceptance lists existing TEIKOKU topics/pages without modifying them.
- **O-001/O3-E059** [E] anchor=`O3:L28` (ledger-line: DEEP-O-001.discovery.md:L423): Frontmatter and wikilinks must round-trip byte-identically.
- **O-001/O3-E060** [E] anchor=`O3:L29` (ledger-line: DEEP-O-001.discovery.md:L424): Any rewrite/reformat during read is failure.
- **O-001/O3-E061** [E] anchor=`O3:L29` (ledger-line: DEEP-O-001.discovery.md:L425): The remedy for read mutation is a pure reader with no normalize-on-load plus a byte-identity test.
- **O-001/O3-E062** [E] anchor=`O3:L30` (ledger-line: DEEP-O-001.discovery.md:L426): Move M2 gates every write through namespace claims.
- **O-001/O3-E063** [E] anchor=`O3:L31` (ledger-line: DEEP-O-001.discovery.md:L427): An ELAI write into an unclaimed TEIKOKU-owned namespace yields a claim-violation error.
- **O-001/O3-E064** [E] anchor=`O3:L31` (ledger-line: DEEP-O-001.discovery.md:L428): Writing into an ELAI-claimed namespace succeeds under create/append-only semantics.
- **O-001/O3-E065** [E] anchor=`O3:L32` (ledger-line: DEEP-O-001.discovery.md:L429): If a cross-namespace write succeeds, the claim check must be moved into the single write chokepoint.
- **O-001/O3-E066** [E] anchor=`O3:L32` (ledger-line: DEEP-O-001.discovery.md:L430): Cross-namespace protection requires a refusal test.
- **O-001/O3-E067** [E] anchor=`O3:L33` (ledger-line: DEEP-O-001.discovery.md:L431): Move M3 restricts upsert to machine-owned `rankings`, `queue`, and `index` living pages.
- **O-001/O3-E068** [E] anchor=`O3:L34` (ledger-line: DEEP-O-001.discovery.md:L432): Upsert may update a machine-owned rankings page in place.
- **O-001/O3-E069** [E] anchor=`O3:L34` (ledger-line: DEEP-O-001.discovery.md:L433): Upsert targeting a user note must refuse.
- **O-001/O3-E070** [E] anchor=`O3:L35` (ledger-line: DEEP-O-001.discovery.md:L434): If a user note is edited, the remedy is a namespace plus page-class predicate on upsert and a user-note refusal test.
- **O-001/O3-E071** [E] [SURFACE] anchor=`O3:L37-L38` (ledger-line: DEEP-O-001.discovery.md:L435): If the TEIKOKU vault is inaccessible from the host, such as a WSL-only tree, Tier 1 uses a fresh ELAI-claimed vault.
- **O-001/O3-E072** [E] anchor=`O3:L38` (ledger-line: DEEP-O-001.discovery.md:L436): Under that fork, TEIKOKU mounting is a Tier 2 follow-up gated on a shared path and O-3 is not blocked.
- **O-001/O3-E073** [E] [SURFACE] anchor=`O3:L40-L41` (ledger-line: DEEP-O-001.discovery.md:L437): Before authoring the reader, recon must read the TEIKOKU vault root and `write_page`/`upsert_page` source to establish layout and frontmatter.
- **O-001/O3-E074** [E] anchor=`O3:L42` (ledger-line: DEEP-O-001.discovery.md:L438): Recon must examine the `memory_facts` API to preserve a disjoint boundary and prevent accidental dual-write.
- **O-001/O3-E075** [E] anchor=`O3:L43` (ledger-line: DEEP-O-001.discovery.md:L439): Recon must locate the D-5 denylist and privacy/air-gap read entrypoint.
- **O-001/O3-E076** [E] anchor=`O3:L45-L46` (ledger-line: DEEP-O-001.discovery.md:L440): If mount compatibility would require editing user-authored notes to fit ELAI, execution aborts and flags the issue.
- **O-001/O3-E077** [E] [SURFACE] anchor=`O3:L46` (ledger-line: DEEP-O-001.discovery.md:L441): ELAI must adapt to the vault format; the vault may never be adapted to ELAI.
- **O-001/O3-E078** [E] anchor=`O3:L48-L50` (ledger-line: DEEP-O-001.discovery.md:L442): Produced symbols include the reader/writer for frontmatter, wikilinks, and the topic path.
- **O-001/O3-E079** [E] anchor=`O3:L51` (ledger-line: DEEP-O-001.discovery.md:L443): Produced symbols include the claims parser and hard refusal for unclaimed namespaces.
- **O-001/O3-E080** [E] anchor=`O3:L52` (ledger-line: DEEP-O-001.discovery.md:L444): Produced symbols include guarded create/append and machine-namespace-only `upsert_page`.
- **O-001/O3-E081** [E] anchor=`O3:L53-L54` (ledger-line: DEEP-O-001.discovery.md:L445): O-3 consumes but does not dual-write or reimplement E-5 machine memory.
- **O-001/O3-E082** [E] anchor=`O3:L55-L56` (ledger-line: DEEP-O-001.discovery.md:L446): O-3 consumes D-5 privacy and the TEIKOKU `brain_store` contract.
- **O-001/O3-E083** [E] anchor=`O3:L57-L59` (ledger-line: DEEP-O-001.discovery.md:L447): Production reachability is an O-1/O-2 `FileToBrain` stage writing through the guarded writer during a live run.
- **O-001/O3-E084** [E] anchor=`O3:L60-L64` (ledger-line: DEEP-O-001.discovery.md:L448): A production write into an unclaimed namespace must hard-refuse.
- **O-001/O3-E085** [E] [SURFACE] anchor=`O3:L63` (ledger-line: DEEP-O-001.discovery.md:L449): A production path may not convert the vault into `memory_facts`.
- **O-001/O3-E086** [E] anchor=`O3:L64` (ledger-line: DEEP-O-001.discovery.md:L450): A production path may not route personal-brain content into a project/cloud prompt.
- **O-001/O3-E087** [E] [SURFACE] anchor=`O3:L65-L67` (ledger-line: DEEP-O-001.discovery.md:L451): The observable trigger mounts a vault, performs one ELAI-claimed `FileToBrain` write, and attempts one unclaimed write.
- **O-001/O3-E088** [E] [SURFACE] anchor=`O3:L67` (ledger-line: DEEP-O-001.discovery.md:L452): A successful claimed write emits `vault_write`.
- **O-001/O3-E089** [E] [SURFACE] anchor=`O3:L67` (ledger-line: DEEP-O-001.discovery.md:L453): `vault_write` carries `namespace`.
- **O-001/O3-E090** [E] [SURFACE] anchor=`O3:L67` (ledger-line: DEEP-O-001.discovery.md:L454): `vault_write` carries `page`.
- **O-001/O3-E091** [E] [SURFACE] anchor=`O3:L67` (ledger-line: DEEP-O-001.discovery.md:L455): `vault_write.mode` is one of `create`, `append`, or `upsert`.
- **O-001/O3-E092** [E] anchor=`O3:L67` (ledger-line: DEEP-O-001.discovery.md:L456): An unclaimed attempt emits `claim_violation`.
- **O-001/O3-E093** [E] anchor=`O3:L68` (ledger-line: DEEP-O-001.discovery.md:L457): The claimed write creates a disk page with valid frontmatter and wikilinks.
- **O-001/O3-E094** [E] anchor=`O3:L68` (ledger-line: DEEP-O-001.discovery.md:L458): Any unclaimed write or user-note mutation fails.
- **O-001/O3-E095** [E] anchor=`O3:L70` (ledger-line: DEEP-O-001.discovery.md:L459): The production hook is the O-1/O-2 `FileToBrain` sink wired to the guarded writer in the same changeset.
- **O-001/O3-E096** [E] [SURFACE] anchor=`O3:L70` (ledger-line: DEEP-O-001.discovery.md:L460): If O-1/O-2 has not landed, interim production reachability is CLI subcommand `elai brain write`, named at recon.
- **O-001/O3-E097** [E] anchor=`O3:L72-L75` (ledger-line: DEEP-O-001.discovery.md:L461): Verification runs `wiring_contract_check.py` against this plan with `--run-trigger`.
- **O-001/O3-E098** [E] anchor=`O3:L75` (ledger-line: DEEP-O-001.discovery.md:L462): Verification runs `production_caller_check.py --files <changed .rs> --json`; a dark public function blocks.
- **O-001/O3-E099** [E] anchor=`O3:L76` (ledger-line: DEEP-O-001.discovery.md:L463): Verification passes only when a real caller creates valid frontmatter and an unclaimed attempt produces an off-agent `claim_violation`, never silent success.
- **O-001/O3-E100** [E] [SURFACE] anchor=`O3:L78` (ledger-line: DEEP-O-001.discovery.md:L464): The independent integration smoke starts with a vault already containing a TEIKOKU-owned namespace.
- **O-001/O3-E101** [E] anchor=`O3:L78` (ledger-line: DEEP-O-001.discovery.md:L465): The smoke performs both an ELAI-namespace write and an attempted TEIKOKU-namespace write.
- **O-001/O3-E102** [E] anchor=`O3:L78` (ledger-line: DEEP-O-001.discovery.md:L466): The smoke asserts successful creation, hard claim refusal, and user-note-upsert refusal.
- **O-001/O3-E103** [E] [SURFACE] anchor=`O3:L78` (ledger-line: DEEP-O-001.discovery.md:L467): A single-owner happy path does not satisfy `R10`; the smoke must be a shared-vault two-owner run.
- **O-001/O3-E104** [E] anchor=`O3:L80-L81` (ledger-line: DEEP-O-001.discovery.md:L468): Cross-owner corruption is intended to fail because namespace claims scope writes and upsert is limited to machine-owned page classes.
- **O-001/O3-E105** [E] [SURFACE] anchor=`O3:L81` (ledger-line: DEEP-O-001.discovery.md:L469): Personal-topic leakage is patched by applying D-5 on prompt-bound vault reads.
- **O-001/O3-E106** [E] anchor=`O3:L81` (ledger-line: DEEP-O-001.discovery.md:L470): The D-5 ban is a Tier-1 `stub_ban`, but its enforcement wiring is Tier 2 and must land before personal topics may be mounted into project context.
- **O-001/O3-E107** [E] anchor=`O3:L83-L89` (ledger-line: DEEP-O-001.discovery.md:L471): O-3 composes with E-5 machine memory, D-5 privacy, TEIKOKU brain_store, and O-1/O-2 `FileToBrain`, and reimplements none.
- **O-001/O3-E108** [E] [SURFACE] anchor=`O3:L88-L89` (ledger-line: DEEP-O-001.discovery.md:L472): Mounting replaces migration while vault and `memory_facts` retain separate user and machine grains.
- **O-001/O3-E109** [E] anchor=`O3:L57-L59` (ledger-line: DEEP-O-001.discovery.md:L473): A `#[cfg(test)]` caller never satisfies O-3 production reachability.

### Unit `O4` — O-4: canvas WORKFLOW mode — editor + monitor over TaskSpecs, wire-dark made visual, never a runtime — from O-001 (93 records)

- **O-001/O4-E001** [E] anchor=`O4:L1-L3` (ledger-line: DEEP-O-001.discovery.md:L477): `O-4` is `PLANNED`, Wave `3`, product-surface class, `PROOF_SPINE: none`, and owner-acknowledged `2026-07-06`.
- **O-001/O4-E002** [E] [SURFACE] anchor=`O4:L5` (ledger-line: DEEP-O-001.discovery.md:L478): Metadata is Track `RESEARCH-MACHINE`, lever `canvas-as-spec-editor`, source finding `L4`, grounding `MODERATE (index-derived, PLAN-asserted)`, nature `BUILD-on-shipped`, effort `M`, Wave `3`.
- **O-001/O4-E003** [E] [SURFACE] anchor=`O4:L7` (ledger-line: DEEP-O-001.discovery.md:L479): H-12 is asserted to have shipped PLAN and RESEARCH canvas modes plus a mode bar.
- **O-001/O4-E004** [E] [SURFACE] anchor=`O4:L7` (ledger-line: DEEP-O-001.discovery.md:L480): J-31 defines canvas nodes as bound units.
- **O-001/O4-E005** [E] [SURFACE] anchor=`O4:L7` (ledger-line: DEEP-O-001.discovery.md:L481): J-31 defines canvas edges as verified claims.
- **O-001/O4-E006** [E] [SURFACE] anchor=`O4:L7` (ledger-line: DEEP-O-001.discovery.md:L482): J-31 requires canvas dispatch through the normal FSM.
- **O-001/O4-E007** [E] [SURFACE] anchor=`O4:L7` (ledger-line: DEEP-O-001.discovery.md:L483): J-31 forbids a canvas-side runtime.
- **O-001/O4-E008** [E] [SURFACE] anchor=`O4:L7` (ledger-line: DEEP-O-001.discovery.md:L484): J-60 overlays journal-derived run state on canvas nodes.
- **O-001/O4-E009** [E] [SURFACE] anchor=`O4:L7` (ledger-line: DEEP-O-001.discovery.md:L485): WORKFLOW applies that existing canvas shape to O-2 TaskSpecs.
- **O-001/O4-E010** [E] [SURFACE] anchor=`O4:L7` (ledger-line: DEEP-O-001.discovery.md:L486): A WORKFLOW node is a typed stage instance.
- **O-001/O4-E011** [E] [SURFACE] anchor=`O4:L7` (ledger-line: DEEP-O-001.discovery.md:L487): A WORKFLOW edge is dataflow carrying a provenance tier.
- **O-001/O4-E012** [E] [SURFACE] anchor=`O4:L7` (ledger-line: DEEP-O-001.discovery.md:L488): Node chrome is the stage’s last-run SSB state.
- **O-001/O4-E013** [E] anchor=`O4:L7` (ledger-line: DEEP-O-001.discovery.md:L489): Editing a node edits the underlying TaskSpec row.
- **O-001/O4-E014** [E] anchor=`O4:L7` (ledger-line: DEEP-O-001.discovery.md:L490): Node editing is constrained to the closed enum vocabulary.
- **O-001/O4-E015** [E] anchor=`O4:L7` (ledger-line: DEEP-O-001.discovery.md:L491): Product positioning is “n8n with receipts”: gated, journaled, replayable, AI-research scoped, and not `400` ungated integrations.
- **O-001/O4-E016** [E] [SURFACE] anchor=`O4:L9-L10` (ledger-line: DEEP-O-001.discovery.md:L492): A `WORKFLOW` mode is added to the existing canvas mode bar.
- **O-001/O4-E017** [E] [SURFACE] anchor=`O4:L10` (ledger-line: DEEP-O-001.discovery.md:L493): The node palette is exactly O-2’s coded stage vocabulary; users cannot draw a stage kind that does not exist as code.
- **O-001/O4-E018** [E] anchor=`O4:L10` (ledger-line: DEEP-O-001.discovery.md:L494): Edge provenance tiers come from O-1 taint labels.
- **O-001/O4-E019** [E] anchor=`O4:L10` (ledger-line: DEEP-O-001.discovery.md:L495): Node run-state chrome reuses the J-60 overlay.
- **O-001/O4-E020** [E] anchor=`O4:L10` (ledger-line: DEEP-O-001.discovery.md:L496): A node with a missing consumer renders visibly disconnected.
- **O-001/O4-E021** [E] anchor=`O4:L10` (ledger-line: DEEP-O-001.discovery.md:L497): Visibly disconnected rendering makes wire-dark state visual.
- **O-001/O4-E022** [E] [SURFACE] anchor=`O4:L10` (ledger-line: DEEP-O-001.discovery.md:L498): Dispatch remains in the normal J-31 FSM and the canvas itself runs nothing.
- **O-001/O4-E023** [E] [SURFACE] anchor=`O4:L11` (ledger-line: DEEP-O-001.discovery.md:L499): Any canvas-side scheduler or stage runner is excluded.
- **O-001/O4-E024** [E] [SURFACE] anchor=`O4:L11` (ledger-line: DEEP-O-001.discovery.md:L500): A free-form stage kind typed on the canvas is excluded.
- **O-001/O4-E025** [E] [SURFACE] anchor=`O4:L11` (ledger-line: DEEP-O-001.discovery.md:L501): Re-authoring the mode bar is excluded.
- **O-001/O4-E026** [E] anchor=`O4:L11` (ledger-line: DEEP-O-001.discovery.md:L502): Re-authoring J-60’s overlay is excluded.
- **O-001/O4-E027** [E] anchor=`O4:L11` (ledger-line: DEEP-O-001.discovery.md:L503): An edge without a provenance tier is excluded.
- **O-001/O4-E028** [E] anchor=`O4:L13-L16` (ledger-line: DEEP-O-001.discovery.md:L504): Existing ground truth asserts H-12’s modes and bar, J-31’s FSM-only doctrine, and J-60’s status chrome are shipped.
- **O-001/O4-E029** [E] anchor=`O4:L17` (ledger-line: DEEP-O-001.discovery.md:L505): O-2 is expected to supply TaskSpec rows and closed stage, trigger, and sink enums.
- **O-001/O4-E030** [E] anchor=`O4:L17` (ledger-line: DEEP-O-001.discovery.md:L506): O-1 is expected to supply taint provenance tiers.
- **O-001/O4-E031** [E] [SURFACE] anchor=`O4:L19-L20` (ledger-line: DEEP-O-001.discovery.md:L507): Tier 1 ships the WORKFLOW mode flag in the mode bar.
- **O-001/O4-E032** [E] anchor=`O4:L20` (ledger-line: DEEP-O-001.discovery.md:L508): Tier 1 renders a TaskSpec as nodes and edges.
- **O-001/O4-E033** [E] anchor=`O4:L20` (ledger-line: DEEP-O-001.discovery.md:L509): Tier 1 applies J-60 last-run chrome.
- **O-001/O4-E034** [E] anchor=`O4:L20` (ledger-line: DEEP-O-001.discovery.md:L510): Tier 1 renders a missing consumer as visibly disconnected.
- **O-001/O4-E035** [E] anchor=`O4:L20` (ledger-line: DEEP-O-001.discovery.md:L511): Tier 1’s interaction level is read-only inspection.
- **O-001/O4-E036** [E] [SURFACE] anchor=`O4:L21` (ledger-line: DEEP-O-001.discovery.md:L512): Tier 2 defers in-canvas node editing that writes back to the TaskSpec row until O-2 Tier 1 is wired.
- **O-001/O4-E037** [E] anchor=`O4:L21` (ledger-line: DEEP-O-001.discovery.md:L513): Tier 2 defers edge-level provenance-tier drill-down.
- **O-001/O4-E038** [E] [SURFACE] anchor=`O4:L23-L24` (ledger-line: DEEP-O-001.discovery.md:L514): UI ownership is `crates/dashboard-v2`, using Svelte 5 and Tauri v2.
- **O-001/O4-E039** [E] anchor=`O4:L24` (ledger-line: DEEP-O-001.discovery.md:L515): UI integration projects TaskSpecs to nodes and edges.
- **O-001/O4-E040** [E] anchor=`O4:L24` (ledger-line: DEEP-O-001.discovery.md:L516): UI integration consumes J-60 for chrome.
- **O-001/O4-E041** [E] [SURFACE] anchor=`O4:L25` (ledger-line: DEEP-O-001.discovery.md:L517): The Tauri read surface includes `list_task_specs`.
- **O-001/O4-E042** [E] [SURFACE] anchor=`O4:L25` (ledger-line: DEEP-O-001.discovery.md:L518): The Tauri read surface includes `get_task_spec_runstate`.
- **O-001/O4-E043** [E] [SURFACE] anchor=`O4:L25` (ledger-line: DEEP-O-001.discovery.md:L519): The cross-language pair is the Rust command side plus Svelte view.
- **O-001/O4-E044** [E] [SURFACE] anchor=`O4:L27-L28` (ledger-line: DEEP-O-001.discovery.md:L520): Move M1 extends H-12’s mode enum with WORKFLOW.
- **O-001/O4-E045** [E] anchor=`O4:L28-L29` (ledger-line: DEEP-O-001.discovery.md:L521): M1 projects each stage instance to one graph node.
- **O-001/O4-E046** [E] anchor=`O4:L28` (ledger-line: DEEP-O-001.discovery.md:L522): M1 projects dataflow into graph edges.
- **O-001/O4-E047** [E] [SURFACE] anchor=`O4:L29` (ledger-line: DEEP-O-001.discovery.md:L523): Selecting WORKFLOW must render a configured TaskSpec as a node-per-stage graph.
- **O-001/O4-E048** [E] [SURFACE] anchor=`O4:L29` (ledger-line: DEEP-O-001.discovery.md:L524): The palette must offer exactly O-2’s closed stage kinds and no others.
- **O-001/O4-E049** [E] [SURFACE] anchor=`O4:L30` (ledger-line: DEEP-O-001.discovery.md:L525): If a free-form node appears, the palette must be bound to O-2’s enum at build time.
- **O-001/O4-E050** [E] anchor=`O4:L31` (ledger-line: DEEP-O-001.discovery.md:L526): Move M2 overlays each node’s last-run SSB state as J-60 chrome.
- **O-001/O4-E051** [E] anchor=`O4:L32` (ledger-line: DEEP-O-001.discovery.md:L527): A stage that ran displays its last-run status.
- **O-001/O4-E052** [E] anchor=`O4:L32` (ledger-line: DEEP-O-001.discovery.md:L528): A stage whose downstream consumer is absent displays the disconnected treatment instead of a normal edge.
- **O-001/O4-E053** [E] anchor=`O4:L33` (ledger-line: DEEP-O-001.discovery.md:L529): Missing-consumer state is derived from the TaskSpec edge set.
- **O-001/O4-E054** [E] anchor=`O4:L33` (ledger-line: DEEP-O-001.discovery.md:L530): A wire-dark node rendered normally is fixed by wiring that derived state into rendering and adding a visual-disconnect assertion.
- **O-001/O4-E055** [E] [SURFACE] anchor=`O4:L34` (ledger-line: DEEP-O-001.discovery.md:L531): Move M3 adds a `run now` affordance that enqueues through the normal FSM.
- **O-001/O4-E056** [E] [SURFACE] anchor=`O4:L34` (ledger-line: DEEP-O-001.discovery.md:L532): The run-now affordance may never invoke a canvas-side executor.
- **O-001/O4-E057** [E] [SURFACE] anchor=`O4:L35` (ledger-line: DEEP-O-001.discovery.md:L533): A canvas-triggered run produces the same `task_spec_fired` SSB row as O-2’s scheduler.
- **O-001/O4-E058** [E] [SURFACE] anchor=`O4:L35` (ledger-line: DEEP-O-001.discovery.md:L534): The canvas introduces no canvas-owned execution event.
- **O-001/O4-E059** [E] [SURFACE] anchor=`O4:L36` (ledger-line: DEEP-O-001.discovery.md:L535): If the canvas runs a pipeline itself, the remedy is calling O-2’s dispatcher and asserting fire-row provenance shows the FSM path.
- **O-001/O4-E060** [E] anchor=`O4:L38-L39` (ledger-line: DEEP-O-001.discovery.md:L536): If O-2’s store is not queryable, O-4 Tier 1 may use a read-only fixture projection.
- **O-001/O4-E061** [E] anchor=`O4:L39` (ledger-line: DEEP-O-001.discovery.md:L537): Live store wiring is then gated on O-2.
- **O-001/O4-E062** [E] [SURFACE] anchor=`O4:L39` (ledger-line: DEEP-O-001.discovery.md:L538): O-4 must not create a canvas-side spec store even in fallback.
- **O-001/O4-E063** [E] [SURFACE] anchor=`O4:L41-L42` (ledger-line: DEEP-O-001.discovery.md:L539): Before editing, recon must find and read H-12’s mode enum and registration mechanism.
- **O-001/O4-E064** [E] anchor=`O4:L43` (ledger-line: DEEP-O-001.discovery.md:L540): Recon must locate the J-60 API that binds last-run state to a node.
- **O-001/O4-E065** [E] [SURFACE] anchor=`O4:L44` (ledger-line: DEEP-O-001.discovery.md:L541): Recon must confirm O-2’s `list_task_specs` command or record it as a required O-2 input.
- **O-001/O4-E066** [E] [SURFACE] anchor=`O4:L46-L47` (ledger-line: DEEP-O-001.discovery.md:L542): If WORKFLOW requires any canvas-side execution logic, execution aborts and flags the violation.
- **O-001/O4-E067** [E] [SURFACE] anchor=`O4:L47` (ledger-line: DEEP-O-001.discovery.md:L543): The canvas is strictly an editor and monitor; all dispatch authority stays in the FSM.
- **O-001/O4-E068** [E] [SURFACE] anchor=`O4:L49-L51` (ledger-line: DEEP-O-001.discovery.md:L544): Produced symbols include WORKFLOW and TaskSpec-to-nodes/edges projection.
- **O-001/O4-E069** [E] [SURFACE] anchor=`O4:L52` (ledger-line: DEEP-O-001.discovery.md:L545): Produced symbols include the enum-constrained palette bound to O-2.
- **O-001/O4-E070** [E] anchor=`O4:L53` (ledger-line: DEEP-O-001.discovery.md:L546): Produced symbols include missing-consumer visibly-disconnected rendering.
- **O-001/O4-E071** [E] [SURFACE] anchor=`O4:L54` (ledger-line: DEEP-O-001.discovery.md:L547): Produced symbols include both Tauri read commands.
- **O-001/O4-E072** [E] anchor=`O4:L55-L58` (ledger-line: DEEP-O-001.discovery.md:L548): O-4 consumes H-12, J-31, J-60, O-2 store/enums, and O-1 provenance rather than reauthoring them.
- **O-001/O4-E073** [E] [SURFACE] anchor=`O4:L59-L61` (ledger-line: DEEP-O-001.discovery.md:L549): Production reachability is a user selecting WORKFLOW in the live dashboard and inspecting a real TaskSpec.
- **O-001/O4-E074** [E] anchor=`O4:L61` (ledger-line: DEEP-O-001.discovery.md:L550): Neither cfg-test nor component-test callers satisfy production reachability.
- **O-001/O4-E075** [E] [SURFACE] anchor=`O4:L67` (ledger-line: DEEP-O-001.discovery.md:L551): `ts-rs` bindings for the Svelte/Rust frontend pair must not drift.
- **O-001/O4-E076** [E] anchor=`O4:L68-L70` (ledger-line: DEEP-O-001.discovery.md:L552): The observable trigger loads a TaskSpec with a deliberately missing downstream consumer.
- **O-001/O4-E077** [E] [SURFACE] anchor=`O4:L70` (ledger-line: DEEP-O-001.discovery.md:L553): Selecting run-now must emit O-2’s `task_spec_fired` through the FSM and no canvas-owned execution row.
- **O-001/O4-E078** [E] anchor=`O4:L71` (ledger-line: DEEP-O-001.discovery.md:L554): The observable artifact is nodes/edges with last-run chrome and a visibly disconnected missing-consumer node.
- **O-001/O4-E079** [E] [SURFACE] anchor=`O4:L71` (ledger-line: DEEP-O-001.discovery.md:L555): Any canvas execution path or normally-connected display of a wire-dark node fails.
- **O-001/O4-E080** [E] [SURFACE] anchor=`O4:L73` (ledger-line: DEEP-O-001.discovery.md:L556): The production hook registers WORKFLOW in H-12’s mode enum and exposes it from the dashboard route.
- **O-001/O4-E081** [E] [SURFACE] anchor=`O4:L73` (ledger-line: DEEP-O-001.discovery.md:L557): The run-now affordance calls O-2’s dispatcher in the same changeset.
- **O-001/O4-E082** [E] anchor=`O4:L75-L78` (ledger-line: DEEP-O-001.discovery.md:L558): Verification runs `wiring_contract_check.py` against this plan with `--run-trigger`.
- **O-001/O4-E083** [E] [SURFACE] anchor=`O4:L78` (ledger-line: DEEP-O-001.discovery.md:L559): Verification runs `svelte_mount_reachability.py` to prove the view is reachable.
- **O-001/O4-E084** [E] [SURFACE] anchor=`O4:L78` (ledger-line: DEEP-O-001.discovery.md:L560): Verification runs `tauri_ipc_symmetry_check.py` to prove `list_task_specs` invoke/handler symmetry.
- **O-001/O4-E085** [E] [SURFACE] anchor=`O4:L79` (ledger-line: DEEP-O-001.discovery.md:L561): Verification runs `bun .claude/tools/visual/ui-wire-smoke.mjs ...` so a real WORKFLOW click lands a Rust-owned SSB row.
- **O-001/O4-E086** [E] [SURFACE] anchor=`O4:L80` (ledger-line: DEEP-O-001.discovery.md:L562): Completion requires a mount-reachable view, symmetric Tauri command, and live click through FSM dispatch rather than canvas execution.
- **O-001/O4-E087** [E] [SURFACE] anchor=`O4:L82` (ledger-line: DEEP-O-001.discovery.md:L563): The independent integration smoke uses the real dashboard and a TaskSpec spanning O-1 stages with one missing consumer.
- **O-001/O4-E088** [E] [SURFACE] anchor=`O4:L82` (ledger-line: DEEP-O-001.discovery.md:L564): The smoke selects WORKFLOW, triggers run-now, and asserts disconnected rendering, O-2’s FSM fire row, and J-60 chrome updating.
- **O-001/O4-E089** [E] anchor=`O4:L82` (ledger-line: DEEP-O-001.discovery.md:L565): A component test does not satisfy `R10`; the smoke must use a live browser through the real entrypoint.
- **O-001/O4-E090** [E] [SURFACE] anchor=`O4:L84-L85` (ledger-line: DEEP-O-001.discovery.md:L566): Client-side execution-engine smuggling is intended to fail because J-31 is a stub ban and run-now routes to O-2’s journaled dispatcher.
- **O-001/O4-E091** [E] anchor=`O4:L85` (ledger-line: DEEP-O-001.discovery.md:L567): Hiding wire-dark state with a normal edge is patched by deriving disconnection from the edge set and asserting it in the smoke.
- **O-001/O4-E092** [E] anchor=`O4:L87-L92` (ledger-line: DEEP-O-001.discovery.md:L568): O-4 composes with H-12, J-31, J-60, O-2, and O-1, and reimplements none.
- **O-001/O4-E093** [E] [SURFACE] anchor=`O4:L91-L92` (ledger-line: DEEP-O-001.discovery.md:L569): Final composition rule: canvas is editor/monitor, FSM is dispatcher, and wire-dark is rendered rather than hidden.

---

## Summary inventory atoms — from 15-O-SUMMARY_INDEX

Ledger: `docs/thefinalglance/feature-discovery/15-O-SUMMARY_INDEX.discovery.md`. Orientation-level explicit atoms. Where a deep CORRECT/material-correction delta supersedes them, the summary atom is retained and a superseded note records the loser.

### Summary section `G` — Track-wide premise and product contract (14 records)

- **SUMMARY/O.G.E01** [E/Identity] anchor=`O-SUMMARY:L1,L15` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L52): The track is named `RESEARCH-MACHINE`.
- **SUMMARY/O.G.E02** [E/Product surface] [SURFACE] anchor=`O-SUMMARY:L9-L10` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L53): The desired product is a “TEIKOKU-style personal research machine on the ELAI canvas.”
- **SUMMARY/O.G.E03** [E/Architecture] anchor=`O-SUMMARY:L10` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L54): The product decomposes into four vertical slices.
- **SUMMARY/O.G.E04** [E/Composition/status assertion] anchor=`O-SUMMARY:L10-L11` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L55): The four slices are asserted to be approximately 80% composition of shipped subsystems.
- **SUMMARY/O.G.E05** [E/Comparative constraint] anchor=`O-SUMMARY:L11` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L56): The summary distinguishes TEIKOKU by noting that TEIKOKU already has classify canaries.
- **SUMMARY/O.G.E06** [E/Comparative constraint] anchor=`O-SUMMARY:L11-L12` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L57): The summary distinguishes n8n by characterizing its nodes as arbitrary and ungated.
- **SUMMARY/O.G.E07** [E/Global invariant] anchor=`O-SUMMARY:L12-L13` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L58): Every O unit is required to be gated.
- **SUMMARY/O.G.E08** [E/Global invariant] anchor=`O-SUMMARY:L12-L13` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L59): Every O unit is required to be journaled.
- **SUMMARY/O.G.E09** [E/Global invariant] anchor=`O-SUMMARY:L12-L13` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L60): Every O unit is required to be replayable.
- **SUMMARY/O.G.E10** [E/Global invariant] anchor=`O-SUMMARY:L12-L13` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L61): Every O unit is required to use a closed vocabulary.
- **SUMMARY/O.G.E11** [E/Scope/status] anchor=`O-SUMMARY:L13` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L62): O is classified as a product-surface letter.
- **SUMMARY/O.G.E12** [E/Proof status] anchor=`O-SUMMARY:L13` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L63): O declares `PROOF_SPINE: none`.
- **SUMMARY/O.G.E13** [E/Proof status] anchor=`O-SUMMARY:L13` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L64): The absence of a proof-spine contribution is owner-acknowledged.
- **SUMMARY/O.G.E14** [E/Measurement status] anchor=`O-SUMMARY:L13` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L65): O feeds no `PS-1..PS-6` number.

### Summary section `O-1` — O-1 — research daemon (generalize Cosmos) [KEYSTONE] (42 records)

- **SUMMARY/O-1.E01** [E/Existing capability assertion] anchor=`O-SUMMARY:L20` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L73): `CosmosWorker` is asserted to already be a scheduled research loop.
- **SUMMARY/O-1.E02** [E/Existing state/config] anchor=`O-SUMMARY:L20` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L74): The existing loop has a manifest.
- **SUMMARY/O-1.E03** [E/Existing scheduling behavior] anchor=`O-SUMMARY:L20` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L75): The existing loop has an hourly tick.
- **SUMMARY/O-1.E04** [E/Existing workflow behavior] anchor=`O-SUMMARY:L20-L21` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L76): The existing loop performs `FlashResearch` refresh.
- **SUMMARY/O-1.E05** [E/Existing lifecycle behavior] anchor=`O-SUMMARY:L21` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L77): The existing loop applies time decay.
- **SUMMARY/O-1.E06** [E/Existing bound] anchor=`O-SUMMARY:L21` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L78): The existing loop produces a briefing capped at `<=1500` tokens.
- **SUMMARY/O-1.E07** [E/Existing integration] anchor=`O-SUMMARY:L21` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L79): The existing loop uses `CosmosStage`.
- **SUMMARY/O-1.E08** [E/Current limitation] anchor=`O-SUMMARY:L21` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L80): The current manifest is hard-bound to designer briefs.
- **SUMMARY/O-1.E09** [E/Scope constraint] anchor=`O-SUMMARY:L21-L22` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L81): O-1 generalizes only the manifest, rather than replacing the research loop.
- **SUMMARY/O-1.E10** [E/New configuration feature] [SURFACE] anchor=`O-SUMMARY:L22` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L82): The generalized manifest represents a user-editable topic set.
- **SUMMARY/O-1.E11** [E/Data contract] anchor=`O-SUMMARY:L22` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L83): Each topic is typed.
- **SUMMARY/O-1.E12** [E/Safety/data invariant] anchor=`O-SUMMARY:L22` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L84): Topic `kind` is a closed enum.
- **SUMMARY/O-1.E13** [E/Enum value] anchor=`O-SUMMARY:L22` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L85): `steal-scan` is an allowed topic-kind value.
- **SUMMARY/O-1.E14** [E/Enum value] anchor=`O-SUMMARY:L22` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L86): `leaderboard` is an allowed topic-kind value.
- **SUMMARY/O-1.E15** [E/Enum value] anchor=`O-SUMMARY:L22-L23` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L87): `discovery` is an allowed topic-kind value.
- **SUMMARY/O-1.E16** [E/Enum value] anchor=`O-SUMMARY:L23` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L88): `monitor` is an allowed topic-kind value.
- **SUMMARY/O-1.E17** [E/Security/provenance behavior] anchor=`O-SUMMARY:L23` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L89): G-1 taint is threaded through every fetched byte.
- **SUMMARY/O-1.E18** [E/Security behavior] anchor=`O-SUMMARY:L23` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L90): The F-3 tool filter is threaded through every fetched byte.
- **SUMMARY/O-1.E19** [E/Replay behavior] anchor=`O-SUMMARY:L23` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L91): G-2 VCR is threaded through every fetched byte.
- **SUMMARY/O-1.E20** [E/Workflow stage] anchor=`O-SUMMARY:L23-L24` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L92): Each topic pipeline begins with `fetch`.
- **SUMMARY/O-1.E21** [E/Workflow stage] anchor=`O-SUMMARY:L24` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L93): The fetched item goes through `enum-classify`.
- **SUMMARY/O-1.E22** [E/Workflow stage] anchor=`O-SUMMARY:L24` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L94): Classification is followed by a duplicate precheck (`dup-precheck`).
- **SUMMARY/O-1.E23** [E/Workflow stage] anchor=`O-SUMMARY:L24` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L95): The per-topic pipeline includes `file`.
- **SUMMARY/O-1.E24** [E/Workflow stage] anchor=`O-SUMMARY:L24` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L96): The per-topic pipeline ends in `digest`.
- **SUMMARY/O-1.E25** [E/External grounding/status assertion] anchor=`O-SUMMARY:L23-L25` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L97): The `fetch->enum-classify->dup-precheck->file->digest` pipeline is asserted to be TEIKOKU-proven.
- **SUMMARY/O-1.E26** [E/Added mechanism] anchor=`O-SUMMARY:L24-L25` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L98): ELAI adds taint to the TEIKOKU-proven pipeline.
- **SUMMARY/O-1.E27** [E/Added mechanism] anchor=`O-SUMMARY:L24-L25` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L99): ELAI adds replay on top of the classify canaries.
- **SUMMARY/O-1.E28** [E/Missing/status] anchor=`O-SUMMARY:L25` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L100): The typed-topic surface is dark/absent at summary time.
- **SUMMARY/O-1.E29** [E/Missing/status] anchor=`O-SUMMARY:L25` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L101): The enum-classify stage is absent as ELAI code and exists only in TEIKOKU at summary time.
- **SUMMARY/O-1.E30** [E/Composition constraint] anchor=`O-SUMMARY:L26` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L102): `cosmos_worker`, `CosmosStage`, and the briefing compiler must be extended and must not be re-authored.
- **SUMMARY/O-1.E31** [E/Dependency] anchor=`O-SUMMARY:L26` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L103): O-1 composes with `FlashResearch`.
- **SUMMARY/O-1.E32** [E/Integration point] anchor=`O-SUMMARY:L27` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L104): The production integration adds a cosmos manifest loader in `crates/knowledge`.
- **SUMMARY/O-1.E33** [E/Integration point] anchor=`O-SUMMARY:L27` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L105): The production integration adds the classify stage in `crates/knowledge`.
- **SUMMARY/O-1.E34** [E/Surface/composition constraint] [SURFACE] anchor=`O-SUMMARY:L27` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L106): `CosmosDrawer` is extended; no new drawer is created.
- **SUMMARY/O-1.E35** [E/Test/workflow] anchor=`O-SUMMARY:L28` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L107): The positive done-gate uses a live cosmos tick to drive a `steal-scan` topic over a canary-injection fixture.
- **SUMMARY/O-1.E36** [E/Observability/evidence] anchor=`O-SUMMARY:L28-L29` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L108): That run must land an off-agent classify row in the SSB.
- **SUMMARY/O-1.E37** [E/Event field] anchor=`O-SUMMARY:L29` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L109): The classify row carries `class`.
- **SUMMARY/O-1.E38** [E/Event field] anchor=`O-SUMMARY:L29` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L110): The classify row carries `confidence`.
- **SUMMARY/O-1.E39** [E/Event field] anchor=`O-SUMMARY:L29` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L111): The classify row carries `taint_tier`.
- **SUMMARY/O-1.E40** [E/Safety/test behavior] anchor=`O-SUMMARY:L29-L30` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L112): The injected canary must route to `unsorted`.
- **SUMMARY/O-1.E41** [E/Negative invariant] anchor=`O-SUMMARY:L29-L30` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L113): An out-of-enum classifier output fails the done-gate.
- **SUMMARY/O-1.E42** [E/Negative invariant] anchor=`O-SUMMARY:L30` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L114): A canary reaching a steal sink fails the done-gate.

### Summary section `O-2` — O-2 — scheduled TaskSpec substrate (43 records)

- **SUMMARY/O-2.E01** [E/Borrowed behavior] anchor=`O-SUMMARY:L35` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L122): O-2 adopts the Codex Automations pattern of time schedules.
- **SUMMARY/O-2.E02** [E/Borrowed behavior] anchor=`O-SUMMARY:L35` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L123): O-2 adopts the Codex Automations pattern of event schedules.
- **SUMMARY/O-2.E03** [E/Borrowed workflow/surface] [SURFACE] anchor=`O-SUMMARY:L35` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L124): Scheduled results flow to an inbox.
- **SUMMARY/O-2.E04** [E/Borrowed lifecycle behavior] anchor=`O-SUMMARY:L35` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L125): The borrowed automation flow includes auto-archive.
- **SUMMARY/O-2.E05** [E/Data invariant] anchor=`O-SUMMARY:L35` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L126): ELAI's scheduled substrate is typed.
- **SUMMARY/O-2.E06** [E/Evidence invariant] anchor=`O-SUMMARY:L35` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L127): ELAI's scheduled substrate is journaled.
- **SUMMARY/O-2.E07** [E/Persistence contract] anchor=`O-SUMMARY:L35-L36` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L128): A `TaskSpec` is stored as a SQLite row.
- **SUMMARY/O-2.E08** [E/Composition/data pattern] anchor=`O-SUMMARY:L36` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L129): The row follows the rules-engine rows-as-data pattern.
- **SUMMARY/O-2.E09** [E/Data contract] anchor=`O-SUMMARY:L36` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L130): `TaskSpec.trigger` is a closed choice among named trigger kinds.
- **SUMMARY/O-2.E10** [E/Enum value] anchor=`O-SUMMARY:L36` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L131): `Cron` is an allowed trigger value.
- **SUMMARY/O-2.E11** [E/Enum value] anchor=`O-SUMMARY:L36` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L132): `SsbEvent` is an allowed trigger value.
- **SUMMARY/O-2.E12** [E/Enum value] anchor=`O-SUMMARY:L36` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L133): `Manual` is an allowed trigger value.
- **SUMMARY/O-2.E13** [E/Data/workflow invariant] anchor=`O-SUMMARY:L36-L37` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L134): `TaskSpec.pipeline` is ordered.
- **SUMMARY/O-2.E14** [E/Data/safety invariant] anchor=`O-SUMMARY:L36-L37` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L135): Pipeline stage kinds come from a closed enum.
- **SUMMARY/O-2.E15** [E/Data contract] anchor=`O-SUMMARY:L37` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L136): `TaskSpec.sink` is a closed choice among named sink kinds.
- **SUMMARY/O-2.E16** [E/Enum value] [SURFACE] anchor=`O-SUMMARY:L37` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L137): `VaultPage` is an allowed sink value.
- **SUMMARY/O-2.E17** [E/Enum value] anchor=`O-SUMMARY:L37` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L138): `Digest` is an allowed sink value.
- **SUMMARY/O-2.E18** [E/Enum value] [SURFACE] anchor=`O-SUMMARY:L37` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L139): `DashboardInbox` is an allowed sink value.
- **SUMMARY/O-2.E19** [E/Enum value] [SURFACE] anchor=`O-SUMMARY:L37` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L140): `Toast` is an allowed sink value.
- **SUMMARY/O-2.E20** [E/Observability/evidence] anchor=`O-SUMMARY:L37-L38` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L141): Every TaskSpec fire emits a journaled SSB row.
- **SUMMARY/O-2.E21** [E/Event field/invariant] anchor=`O-SUMMARY:L37-L38` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L142): Every fire row carries the trigger cause.
- **SUMMARY/O-2.E22** [E/Determinism boundary] anchor=`O-SUMMARY:L38` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L143): K-14 keeps the wall-clock trigger in the supervisor plane.
- **SUMMARY/O-2.E23** [E/Determinism behavior] anchor=`O-SUMMARY:L38-L39` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L144): Keeping wall-clock triggering in the supervisor plane is intended to keep the run deterministic.
- **SUMMARY/O-2.E24** [E/Lifecycle/operations reuse] anchor=`O-SUMMARY:L38-L39` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L145): Missed-run detection reuses the SSB temporal monitor.
- **SUMMARY/O-2.E25** [E/Surface reuse/status assertion] [SURFACE] anchor=`O-SUMMARY:L39` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L146): Task results reuse the shipped intervention-card inbox surface.
- **SUMMARY/O-2.E26** [E/Missing/status] anchor=`O-SUMMARY:L39-L40` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L147): The typed TaskSpec itself is absent at summary time.
- **SUMMARY/O-2.E27** [E/Missing/status] anchor=`O-SUMMARY:L39-L40` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L148): Trigger-to-`task_runner` wiring is absent at summary time.
- **SUMMARY/O-2.E28** [E/Dependency] anchor=`O-SUMMARY:L41` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L149): O-2 composes with rules-engine rows-as-data (`3F`).
- **SUMMARY/O-2.E29** [E/Dependency] anchor=`O-SUMMARY:L41` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L150): O-2 composes with the `task_runner` interval (`C-9`).
- **SUMMARY/O-2.E30** [E/Dependency] anchor=`O-SUMMARY:L41-L42` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L151): O-2 composes with SSB and the K-14 temporal monitor.
- **SUMMARY/O-2.E31** [E/Dependency] [SURFACE] anchor=`O-SUMMARY:L42` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L152): O-2 composes with intervention cards.
- **SUMMARY/O-2.E32** [E/Cross-O dependency] anchor=`O-SUMMARY:L42` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L153): O-2's pipeline vocabulary consumes O-1 stage kinds.
- **SUMMARY/O-2.E33** [E/Integration point with unresolved owner] anchor=`O-SUMMARY:L43` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L154): The integration must add a `TaskSpec` table in `crates/orchestrator` or the rules crate.
- **SUMMARY/O-2.E34** [E/Integration point] anchor=`O-SUMMARY:L43` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L155): The integration must add a TaskSpec dispatcher.
- **SUMMARY/O-2.E35** [E/Integration point] anchor=`O-SUMMARY:L43` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L156): The integration must add fire-row emission.
- **SUMMARY/O-2.E36** [E/Scheduling invariant/done-gate] anchor=`O-SUMMARY:L44` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L157): A due-now `Cron` TaskSpec must fire exactly once per tick.
- **SUMMARY/O-2.E37** [E/Event type] anchor=`O-SUMMARY:L44` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L158): The emitted event is named `task_spec_fired`.
- **SUMMARY/O-2.E38** [E/Event field] anchor=`O-SUMMARY:L44-L45` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L159): `task_spec_fired` carries `spec_id`.
- **SUMMARY/O-2.E39** [E/Event field] anchor=`O-SUMMARY:L44-L45` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L160): `task_spec_fired` carries `trigger_cause`.
- **SUMMARY/O-2.E40** [E/Event field] anchor=`O-SUMMARY:L44-L45` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L161): `task_spec_fired` carries `stage_path`.
- **SUMMARY/O-2.E41** [E/Surface/done-gate] [SURFACE] anchor=`O-SUMMARY:L45` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L162): A result-bearing fire renders an inbox card.
- **SUMMARY/O-2.E42** [E/Negative invariant] anchor=`O-SUMMARY:L45-L46` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L163): An empty fire that nevertheless renders a card fails the done-gate.
- **SUMMARY/O-2.E43** [E/Negative invariant] anchor=`O-SUMMARY:L46` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L164): A double-fire fails the done-gate.

### Summary section `O-3` — O-3 — brain vault contract + mounting (35 records)

- **SUMMARY/O-3.E01** [E/Architecture invariant] anchor=`O-SUMMARY:L51` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L172): O-3 splits “the brain” by storage/use grain.
- **SUMMARY/O-3.E02** [E/Existing role/status] anchor=`O-SUMMARY:L51` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L173): E-5 `memory_facts` remains the machine-grain store.
- **SUMMARY/O-3.E03** [E/Existing role] anchor=`O-SUMMARY:L51` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L174): The machine-grain `memory_facts` store feeds prompts.
- **SUMMARY/O-3.E04** [E/New user surface] [SURFACE] anchor=`O-SUMMARY:L51-L53` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L175): A markdown vault is the user-grain surface.
- **SUMMARY/O-3.E05** [E/Data invariant] [SURFACE] anchor=`O-SUMMARY:L51-L52` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L176): The markdown vault format is deterministic.
- **SUMMARY/O-3.E06** [E/File-layout contract] anchor=`O-SUMMARY:L52` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L177): Topic pages use the path contract `topics/<t>/<slug>.md`.
- **SUMMARY/O-3.E07** [E/File-format contract] [SURFACE] anchor=`O-SUMMARY:L52` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L178): Vault pages use frontmatter.
- **SUMMARY/O-3.E08** [E/File-format/navigation feature] [SURFACE] anchor=`O-SUMMARY:L52` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L179): Vault pages support wikilinks.
- **SUMMARY/O-3.E09** [E/User-data lifecycle feature] [SURFACE] anchor=`O-SUMMARY:L52` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L180): The vault contains living rankings.
- **SUMMARY/O-3.E10** [E/User-data lifecycle feature] [SURFACE] anchor=`O-SUMMARY:L52` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L181): The vault contains a living queue.
- **SUMMARY/O-3.E11** [E/User-data lifecycle/navigation feature] [SURFACE] anchor=`O-SUMMARY:L52` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L182): The vault contains a living index.
- **SUMMARY/O-3.E12** [E/External contract reuse] [SURFACE] anchor=`O-SUMMARY:L52-L53` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L183): The vault adopts TEIKOKU `brain_store.write_page` discipline verbatim.
- **SUMMARY/O-3.E13** [E/External contract reuse] [SURFACE] anchor=`O-SUMMARY:L52-L53` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L184): The vault adopts TEIKOKU `brain_store.upsert_page` discipline verbatim.
- **SUMMARY/O-3.E14** [E/Migration invariant] [SURFACE] anchor=`O-SUMMARY:L53-L54` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L185): The markdown format itself is the migration contract.
- **SUMMARY/O-3.E15** [E/Mounting/migration behavior] anchor=`O-SUMMARY:L53-L54` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L186): Migration requires zero conversion.
- **SUMMARY/O-3.E16** [E/Integration behavior] [SURFACE] anchor=`O-SUMMARY:L54` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L187): ELAI mounts the existing TEIKOKU vault.
- **SUMMARY/O-3.E17** [E/New state/config mechanism] [SURFACE] anchor=`O-SUMMARY:L54-L55` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L188): A namespace-claims file exists at the vault root.
- **SUMMARY/O-3.E18** [E/Access-control contract] anchor=`O-SUMMARY:L55` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L189): The claims file records which namespaces are claimed for writing.
- **SUMMARY/O-3.E19** [E/Security/error behavior] anchor=`O-SUMMARY:L55-L56` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L190): A write outside a claimed namespace receives a hard refusal.
- **SUMMARY/O-3.E20** [E/Multi-writer isolation behavior] [SURFACE] anchor=`O-SUMMARY:L55-L56` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L191): Namespace claims are what allow two harnesses to share one vault.
- **SUMMARY/O-3.E21** [E/Privacy dependency] anchor=`O-SUMMARY:L56` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L192): A D-5 life-layer denylist provides the privacy boundary.
- **SUMMARY/O-3.E22** [E/Privacy invariant] anchor=`O-SUMMARY:L56` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L193): Personal topics are kept out of project prompts.
- **SUMMARY/O-3.E23** [E/Privacy invariant] anchor=`O-SUMMARY:L56` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L194): Personal topics are kept out of cloud prompts.
- **SUMMARY/O-3.E24** [E/Storage invariant] [SURFACE] anchor=`O-SUMMARY:L57` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L195): The vault and E-5 `memory_facts` remain disjoint grains.
- **SUMMARY/O-3.E25** [E/Storage/lifecycle invariant] anchor=`O-SUMMARY:L57` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L196): The two stores must not dual-write.
- **SUMMARY/O-3.E26** [E/Cross-O dependency] anchor=`O-SUMMARY:L58` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L197): O-3 composes with the O-1/O-2 `FileToBrain` sink.
- **SUMMARY/O-3.E27** [E/Integration point] [SURFACE] anchor=`O-SUMMARY:L59` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L198): The integration adds a vault reader in `crates/knowledge`.
- **SUMMARY/O-3.E28** [E/Integration point] [SURFACE] anchor=`O-SUMMARY:L59` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L199): The integration adds a vault writer in `crates/knowledge`.
- **SUMMARY/O-3.E29** [E/Integration point] anchor=`O-SUMMARY:L59` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L200): The integration adds a namespace-claims parser.
- **SUMMARY/O-3.E30** [E/Integration/security point] anchor=`O-SUMMARY:L59` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L201): The integration adds guarded upsert.
- **SUMMARY/O-3.E31** [E/Positive done-gate] anchor=`O-SUMMARY:L60` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L202): A `FileToBrain` write inside an ELAI-claimed namespace creates a valid page.
- **SUMMARY/O-3.E32** [E/Observability/evidence] [SURFACE] anchor=`O-SUMMARY:L60` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L203): A successful claimed-namespace write emits a `vault_write` row.
- **SUMMARY/O-3.E33** [E/Refusal observability] anchor=`O-SUMMARY:L60-L61` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L204): An attempted unclaimed-namespace write emits a `claim_violation` row.
- **SUMMARY/O-3.E34** [E/Negative invariant] anchor=`O-SUMMARY:L61-L62` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L205): Any actual write into an unclaimed namespace fails the done-gate.
- **SUMMARY/O-3.E35** [E/Negative invariant] anchor=`O-SUMMARY:L61-L62` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L206): Any mutation of a user note fails the done-gate.

### Summary section `O-4` — O-4 — canvas WORKFLOW mode (33 records)

- **SUMMARY/O-4.E01** [E/New surface] [SURFACE] anchor=`O-SUMMARY:L67` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L214): A `WORKFLOW` mode is added to the shipped canvas mode bar.
- **SUMMARY/O-4.E02** [E/Surface dependency/constraint] [SURFACE] anchor=`O-SUMMARY:L67,L73` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L215): The mode bar is H-12 and must be extended rather than replaced.
- **SUMMARY/O-4.E03** [E/Architecture dependency] [SURFACE] anchor=`O-SUMMARY:L67-L68` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L216): WORKFLOW follows the shape of J-31's canvas doctrine.
- **SUMMARY/O-4.E04** [E/Graph semantic] [SURFACE] anchor=`O-SUMMARY:L67-L68` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L217): A workflow node represents a bound unit.
- **SUMMARY/O-4.E05** [E/Graph semantic] [SURFACE] anchor=`O-SUMMARY:L68` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L218): A workflow edge represents a verified claim.
- **SUMMARY/O-4.E06** [E/Execution invariant] [SURFACE] anchor=`O-SUMMARY:L68` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L219): Workflow dispatch goes through the normal FSM.
- **SUMMARY/O-4.E07** [E/Execution/security invariant] [SURFACE] anchor=`O-SUMMARY:L68` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L220): The canvas has no independent runtime.
- **SUMMARY/O-4.E08** [E/Cross-O surface dependency] [SURFACE] anchor=`O-SUMMARY:L68` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L221): WORKFLOW projects O-2 TaskSpecs.
- **SUMMARY/O-4.E09** [E/Graph/data contract] anchor=`O-SUMMARY:L68-L69` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L222): Each node is a typed stage instance.
- **SUMMARY/O-4.E10** [E/Graph semantic] anchor=`O-SUMMARY:L69` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L223): Each edge represents dataflow.
- **SUMMARY/O-4.E11** [E/Provenance/data contract] anchor=`O-SUMMARY:L69` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L224): Each edge carries a provenance tier.
- **SUMMARY/O-4.E12** [E/Observability/surface] [SURFACE] anchor=`O-SUMMARY:L69` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L225): Node chrome displays last-run SSB state.
- **SUMMARY/O-4.E13** [E/Surface dependency] [SURFACE] anchor=`O-SUMMARY:L69,L73` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L226): The last-run chrome reuses J-60.
- **SUMMARY/O-4.E14** [E/Editing/persistence behavior] anchor=`O-SUMMARY:L70` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L227): Editing a node edits the underlying TaskSpec row.
- **SUMMARY/O-4.E15** [E/Editing/safety invariant] anchor=`O-SUMMARY:L70` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L228): Node edits are constrained to the closed enum.
- **SUMMARY/O-4.E16** [E/Error visualization] anchor=`O-SUMMARY:L70-L71` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L229): A node with a missing consumer renders visibly disconnected.
- **SUMMARY/O-4.E17** [E/Error visualization invariant] anchor=`O-SUMMARY:L71` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L230): The surface makes a wire-dark condition visual instead of silently implying connectivity.
- **SUMMARY/O-4.E18** [E/Product positioning] anchor=`O-SUMMARY:L71` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L231): The intended product shape is “n8n with receipts.”
- **SUMMARY/O-4.E19** [E/Missing/status] [SURFACE] anchor=`O-SUMMARY:L71-L72` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L232): The `WORKFLOW` mode is absent at summary time.
- **SUMMARY/O-4.E20** [E/Missing/status] anchor=`O-SUMMARY:L71-L72` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L233): The `TaskSpec->nodes` projection is absent at summary time.
- **SUMMARY/O-4.E21** [E/Missing/status] [SURFACE] anchor=`O-SUMMARY:L72` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L234): The enum-constrained palette is absent at summary time.
- **SUMMARY/O-4.E22** [E/Cross-O dependency] [SURFACE] anchor=`O-SUMMARY:L73` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L235): WORKFLOW consumes the O-2 TaskSpec store and enums.
- **SUMMARY/O-4.E23** [E/Cross-O dependency] [SURFACE] anchor=`O-SUMMARY:L73` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L236): WORKFLOW consumes O-1 tiers.
- **SUMMARY/O-4.E24** [E/Production entrypoint] [SURFACE] anchor=`O-SUMMARY:L74` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L237): The UI integration is a Svelte WORKFLOW view in `crates/dashboard-v2`.
- **SUMMARY/O-4.E25** [E/API/integration point] [SURFACE] anchor=`O-SUMMARY:L74` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L238): Rust exposes a `list_task_specs` Tauri command.
- **SUMMARY/O-4.E26** [E/API/integration point] [SURFACE] anchor=`O-SUMMARY:L74` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L239): Rust exposes a `get_task_spec_runstate` Tauri command.
- **SUMMARY/O-4.E27** [E/Positive done-gate] [SURFACE] anchor=`O-SUMMARY:L75` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L240): Selecting WORKFLOW in the live dashboard renders a TaskSpec as nodes and edges.
- **SUMMARY/O-4.E28** [E/Positive done-gate] anchor=`O-SUMMARY:L75` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L241): The rendered graph includes J-60 last-run chrome.
- **SUMMARY/O-4.E29** [E/Positive/error done-gate] anchor=`O-SUMMARY:L75-L76` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L242): A missing-consumer node renders disconnected in the live surface.
- **SUMMARY/O-4.E30** [E/Action/observability] [SURFACE] anchor=`O-SUMMARY:L76` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L243): `run-now` emits O-2's fire row.
- **SUMMARY/O-4.E31** [E/Execution invariant/done-gate] [SURFACE] anchor=`O-SUMMARY:L76` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L244): `run-now` reaches execution through the FSM.
- **SUMMARY/O-4.E32** [E/Negative invariant] [SURFACE] anchor=`O-SUMMARY:L76-L77` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L245): Any canvas-side execution path fails the done-gate.
- **SUMMARY/O-4.E33** [E/Negative invariant] anchor=`O-SUMMARY:L76-L77` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L246): A normally connected node that renders wire-dark fails the done-gate.

### Summary section `X` — Cross-cutting digest patches and engagement falsifier (18 records)

- **SUMMARY/O.X.E01** [E/Cross-unit invariant] anchor=`O-SUMMARY:L79-L82` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L252): Every O unit enforces the same A1, A3, and A4 digest patches.
- **SUMMARY/O.X.E02** [E/Verification posture] anchor=`O-SUMMARY:L79-L80` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L253): Those patches are enforced as `stub_ban + REDTEAM`.
- **SUMMARY/O.X.E03** [E/Classifier safety invariant] anchor=`O-SUMMARY:L80-L81` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L254): A1 forbids sending a free-text classifier output to a prompt.
- **SUMMARY/O.X.E04** [E/Classifier data invariant] anchor=`O-SUMMARY:L80-L81` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L255): A1 requires classifier output to use a closed enum.
- **SUMMARY/O.X.E05** [E/Classifier data invariant] anchor=`O-SUMMARY:L81` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L256): A1 requires classifier output to carry confidence.
- **SUMMARY/O.X.E06** [E/Test/safety invariant] anchor=`O-SUMMARY:L81` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L257): A1 requires frozen canaries.
- **SUMMARY/O.X.E07** [E/Workflow vocabulary invariant] anchor=`O-SUMMARY:L81` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L258): A3 forbids generic nodes.
- **SUMMARY/O.X.E08** [E/Workflow vocabulary invariant] anchor=`O-SUMMARY:L81` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L259): A3 forbids custom nodes.
- **SUMMARY/O.X.E09** [E/Workflow data invariant] anchor=`O-SUMMARY:L81-L82` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L260): A3 requires a closed stage vocabulary.
- **SUMMARY/O.X.E10** [E/Extension invariant] anchor=`O-SUMMARY:L81-L82` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L261): A new stage kind must ship as code.
- **SUMMARY/O.X.E11** [E/Ownership boundary] anchor=`O-SUMMARY:L82` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L262): A4 says ELAI ships the substrate.
- **SUMMARY/O.X.E12** [E/Ownership/config boundary] anchor=`O-SUMMARY:L82` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L263): A4 says topic packs are data.
- **SUMMARY/O.X.E13** [E/Multi-writer invariant] [SURFACE] anchor=`O-SUMMARY:L82` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L264): A4 requires disjoint vault namespaces for the two masters.
- **SUMMARY/O.X.E14** [E/Product validation invariant] anchor=`O-SUMMARY:L83` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L265): A2 is the whole-feature falsifier for the default-off graveyard.
- **SUMMARY/O.X.E15** [E/Engagement observability] [SURFACE] anchor=`O-SUMMARY:L83-L84` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L266): A successful digest is connected to a measured digest-open signal.
- **SUMMARY/O.X.E16** [E/Engagement observability] [SURFACE] anchor=`O-SUMMARY:L83-L84` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L267): A successful digest is connected to a measured `For-You` click signal.
- **SUMMARY/O.X.E17** [E/Lifecycle threshold] anchor=`O-SUMMARY:L84` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L268): Zero engagement is evaluated after `N` days.
- **SUMMARY/O.X.E18** [E/Lifecycle behavior/falsifier] anchor=`O-SUMMARY:L84` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L269): A topic with zero engagement after `N` days decays honestly.

### Summary workflows (WF)

- **SUMMARY/WF-01** [WF] title=`Per-topic research path (`O-1`)` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L275): `user-edited typed topic manifest` -> live/hourly Cosmos tick -> `fetch` (G-1 taint + F-3 tool filter + G-2 VCR on every fetched byte) -> closed-enum classification -> duplicate precheck -> file -> digest -> off-agent SSB classify evidence. A frozen canary must be diverted to `unsorted`; it must never reach a steal sink. Source: O-SUMMARY:L20-L30,L80-L81.
- **SUMMARY/WF-02** [WF] [SURFACE] title=`Scheduled execution path (`O-2`)` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L279): Supervisor-plane `Cron`, `SsbEvent`, or `Manual` trigger -> persisted typed `TaskSpec` -> ordered closed-enum stages -> one journaled `task_spec_fired` row with cause and stage path -> one of the typed sinks -> intervention-card inbox only when the fire bears a result. Temporal monitoring detects missed runs; a Cron due now fires exactly once per tick. Source: O-SUMMARY:L35-L46.
- **SUMMARY/WF-03** [WF] [SURFACE] title=`Guarded vault path (`O-3`)` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L283): O-1/O-2 `FileToBrain` request -> namespace-claims parser -> hard write-scope check -> guarded vault writer/upsert -> deterministic markdown page and `vault_write` row. A denied attempt emits `claim_violation` without producing the forbidden write. Prompt-bound reads remain separated from E-5 machine memory, and personal topics may not enter project/cloud prompts. Source: O-SUMMARY:L51-L62.
- **SUMMARY/WF-04** [WF] [SURFACE] title=`Visual editing and run path (`O-4`)` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L287): Select WORKFLOW -> list/project O-2 TaskSpecs to typed nodes and provenance-bearing edges -> overlay last-run SSB state -> edit a node by editing its closed-enum-constrained spec row -> display missing consumers as disconnected -> `run-now` dispatches through the normal FSM -> O-2 emits the fire row. The canvas may never execute directly. Source: O-SUMMARY:L67-L77.
- **SUMMARY/WF-05** [WF] [SURFACE] title=`Track validation loop` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L291): Digest success -> measure digest opens and `For-You` clicks -> after `N` days of zero engagement, decay the topic instead of preserving an unused default-off feature. Source: O-SUMMARY:L83-L84.

### Summary invariants (INV)

- **SUMMARY/INV-01** [INV] title=`Closed-vocabulary all the way down:` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L297): topic kind, classifier class, TaskSpec trigger/stage/sink, and canvas palette/editing cannot silently expand into free text or generic/custom nodes. New stage kinds ship as code. (O-SUMMARY:L22-L23,L35-L37,L70-L72,L80-L82)
- **SUMMARY/INV-02** [INV] title=`One execution authority:` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L298): WORKFLOW is a spec editor/projection, not a runtime; `run-now` enters the normal FSM and produces O-2 journal evidence. (O-SUMMARY:L67-L77)
- **SUMMARY/INV-03** [INV] title=`Off-agent evidence:` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L299): O-1 classification lands off-agent in SSB; O-2 fires, O-3 writes/refusals, and O-4 run state all surface journal-derived evidence. (O-SUMMARY:L28-L30,L37-L46,L60-L62,L69,L75-L77)
- **SUMMARY/INV-04** [INV] title=`Fail closed:` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L300): out-of-enum classification, canary-to-steal, duplicate fire, carding an empty result, unclaimed writes, user-note mutation, canvas-side execution, and false wire-dark rendering are explicit failures. (O-SUMMARY:L29-L30,L45-L46,L61-L62,L76-L77)
- **SUMMARY/INV-05** [INV] title=`Untrusted-input containment:` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L301): fetched bytes acquire taint, tool filtering, and replay coverage before downstream classification/storage; frozen canaries exercise the real path. (O-SUMMARY:L23-L30,L80-L81)
- **SUMMARY/INV-06** [INV] title=`Two-master separation:` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L302): ELAI owns a reusable substrate, topic packs remain data, stores remain grain-disjoint, and vault writers remain in disjoint claimed namespaces. (O-SUMMARY:L51-L58,L82)
- **SUMMARY/INV-07** [INV] title=`No migration-by-copy:` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L303): ELAI mounts the existing contract-compatible vault and does not convert it into `memory_facts` or dual-write both grains. (O-SUMMARY:L53-L58)
- **SUMMARY/INV-08** [INV] title=`Privacy boundary:` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L304): personal topics do not flow into project or cloud prompts. (O-SUMMARY:L56-L58)
- **SUMMARY/INV-09** [INV] title=`Deterministic time boundary:` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L305): wall-clock trigger observation stays in the supervisor plane while the run remains deterministic and journaled. (O-SUMMARY:L37-L39)
- **SUMMARY/INV-10** [INV] title=`Honest product use:` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L306): digest production alone is insufficient; engagement is measured and unused topics decay after an explicitly configurable-but-unspecified `N`-day interval. (O-SUMMARY:L83-L84)

### Summary-silent feature candidates (SILENT) — not designed features yet

- **SUMMARY/SILENT-001** [SILENT] title=`Fetch/runtime failures:` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L383): retry/backoff, source rate limits, partial downloads, parse failures, offline operation, and provider degradation.
- **SUMMARY/SILENT-002** [SILENT] title=`Duplicate-precheck behavior:` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L384): identity/fingerprint algorithm, false positives, cross-topic dedupe, replay determinism, and user override.
- **SUMMARY/SILENT-003** [SILENT] title=`Pipeline typing:` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L385): stage input/output types, type checking, branching, fan-out, failure propagation, compensation, and partial-result behavior.
- **SUMMARY/SILENT-004** [SILENT] title=`Task lifecycle:` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L386): queued/running/succeeded/failed/cancelled states, timeout, pause/resume, deletion, revisions, retention, and manual retry.
- **SUMMARY/SILENT-005** [SILENT] title=`Notification lifecycle:` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L387): toast/card dismissal, auto-archive policy, read/unread state, dedupe, and failure notifications.
- **SUMMARY/SILENT-006** [SILENT] title=`Vault operations:` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L388): backup/restore, reindex, portability checks, corrupted-frontmatter recovery, case sensitivity, path traversal defense, symlink policy, and storage quotas.
- **SUMMARY/SILENT-007** [SILENT] title=`Access/security:` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L389): manifest editor authorization, TaskSpec ownership, multi-user boundaries, unsafe URL handling, secrets/credentials, and cloud egress approvals.
- **SUMMARY/SILENT-008** [SILENT] title=`Privacy observability:` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L390): evidence that a denied personal-topic flow was blocked, redacted audit projection, and user-visible policy controls.
- **SUMMARY/SILENT-009** [SILENT] title=`Canvas accessibility/UX:` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L391): keyboard editing, screen-reader semantics, zoom/layout persistence, undo/redo, diff/approval before save, and mobile behavior.
- **SUMMARY/SILENT-010** [SILENT] title=`Operational health:` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L392): daemon enable/disable, health checks, metrics, logs, alerting, queue pressure, database migrations, backups, and repair commands.
- **SUMMARY/SILENT-011** [SILENT] title=`Test matrix:` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L393): all four topic kinds, all three triggers, all four sinks, invalid schema/property tests, crash/restart, concurrency, and replay-equivalence tests.
- **SUMMARY/SILENT-012** [SILENT] title=`Packaging/rollout:` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L394): default-off/on choice, upgrade path from designer-only Cosmos manifests, feature flags, compatibility versions, and rollback.

---

## Cross-file interaction records (IX-*)

Namespaced by ledger. Deep IX rows reorganize already-counted explicit findings into composition views; they are distinct interaction records for aggregate navigation.

### O-001/IX — Intended full-system path

- **O-001/IX-PATH01** [IX] title=`Define research work.` (ledger-line: DEEP-O-001.discovery.md:L577): The user supplies `[[research.topics]]` records with `id`, closed `kind`, `sources[]`, `cadence`, `confidence_threshold`, and `sink` (`O1:L9-L10`). O-1 swaps only this source into Cosmos; the worker loop, stage, briefing compiler, and hourly task-runner seam remain the pre-existing execution substrate (`O0:L106-L109`; `O1:L7,L13-L16,L27-L30`).
- **O-001/IX-PATH02** [IX] title=`Fetch and contain untrusted bytes.` (ledger-line: DEEP-O-001.discovery.md:L578): A live Cosmos tick fetches topic sources. Before prompt use, bytes receive G-1 untrusted provenance; the research role is narrowed by F-3; external bytes are recorded in a run-ID-keyed G-2 cassette (`O1:L10,L31-L36`). Classification uses an untrusted preamble, an item fence, a per-kind schema, frozen injection fixtures, closed class output, and `f32` confidence (`O1:L10,L31-L33,L48-L52`).
- **O-001/IX-PATH03** [IX] [SURFACE] title=`Classify, deduplicate, and produce.` (ledger-line: DEEP-O-001.discovery.md:L579): The TEIKOKU-derived workflow is fetch → enum classification → duplicate precheck against a rejected ledger → file → layered digest →, at full scope, guarded living-page upsert (`O1:L10`). Tier 1 covers `steal-scan` and `monitor`; `leaderboard`, `discovery`, and living leaderboard pages wait for Tier 2/O-3 (`O1:L19-L21`).
- **O-001/IX-PATH04** [IX] [SURFACE] title=`Represent scheduled execution as data.` (ledger-line: DEEP-O-001.discovery.md:L580): O-2 persists strict TaskSpec rows in SQLite. Each row combines a closed trigger, ordered closed stages, and a closed sink (`O2:L9-L10,L27-L30,L49-L53`). Tier 1 brings Cron/Manual, `Fetch/Classify/Gate/Digest`, fire rows, and DashboardInbox; event triggering, orchestrator stage kinds, and missed-run hookup are deferred (`O2:L19-L21`).
- **O-001/IX-PATH05** [IX] title=`Fire through the existing authority.` (ledger-line: DEEP-O-001.discovery.md:L581): Supervisor-plane time makes a due decision outside replay; the existing task_runner interval dispatches the pipeline; a last-fired watermark makes a due tick idempotent; the run receives recorded time as an input (`O2:L10,L31-L33,L46-L47`). A fire writes `task_spec_fired {spec_id, trigger_cause, stage_path}` (`O2:L31-L33,L65-L70`).
- **O-001/IX-PATH06** [IX] [SURFACE] title=`Surface only useful scheduled output.` (ledger-line: DEEP-O-001.discovery.md:L582): Result-bearing output creates an existing intervention card. Nothing-to-report output auto-archives, leaving only the SSB receipt; the predicate is based on pipeline result count (`O2:L34-L36,L65-L68`).
- **O-001/IX-PATH07** [IX] [SURFACE] title=`Write selected research to the owner vault.` (ledger-line: DEEP-O-001.discovery.md:L583): O-3’s production seam is O-2’s `FileToBrain` stage calling a guarded writer (or interim `elai brain write`) (`O3:L57-L70`). The writer resolves a configured vault root, validates ownership at a single chokepoint, creates/appends only in ELAI-owned space, and upserts only machine-owned rankings/queue/index pages (`O3:L18-L35`). It records `vault_write {namespace,page,mode}` or `claim_violation` (`O3:L65-L68`).
- **O-001/IX-PATH08** [IX] [SURFACE] title=`Keep the two memory grains separate.` (ledger-line: DEEP-O-001.discovery.md:L584): The mounted deterministic Markdown vault is the human/owner grain; E-5 `memory_facts` remains the prompt-feeding machine grain. There is no conversion or dual-write, and ELAI adapts to existing vault bytes (`O3:L7,L10-L11,L37-L46,L53-L64`). Prompt-bound personal content is barred pending the Tier-2 D-5 read wiring (`O3:L18-L20,L80-L81`).
- **O-001/IX-PATH09** [IX] [SURFACE] title=`Project, inspect, and eventually edit visually.` (ledger-line: DEEP-O-001.discovery.md:L585): O-4 lists TaskSpecs through Tauri and renders a node-per-stage graph in Svelte. Provenance-labelled edges, J-60 last-run chrome, and missing-consumer disconnection make execution receipts and wire-dark state visible (`O4:L19-L33,L49-L71`). Tier 1 is read-only; Tier 2 writes node edits back to TaskSpec rows and adds provenance drill-down (`O4:L19-L21`).
- **O-001/IX-PATH10** [IX] [SURFACE] title=`Run-now shares the same backend.` (ledger-line: DEEP-O-001.discovery.md:L586): The canvas calls O-2’s dispatcher, which emits the same `task_spec_fired` receipt through the normal FSM. No client-side scheduler, stage runner, spec store, or separate execution event is allowed (`O4:L34-L39,L46-L47,L68-L73`).
- **O-001/IX-PATH11** [IX] title=`Replay and observe.` (ledger-line: DEEP-O-001.discovery.md:L587): Network-dependent research replays byte-identically from G-2 with network detached; wall-clock observation itself remains outside replay (`O1:L34-L36`; `O2:L10,L46-L47`). O-1 classify rows, O-2 fire rows, O-3 write/refusal rows, and J-60’s fold of run state form the observable receipt chain (`O0:L80-L85`; `O1:L64-L75`; `O2:L65-L76`; `O3:L65-L76`; `O4:L31-L35,L68-L82`).

### O-001/IX — Closed vocabularies

- **O-001/IX-VOCAB001** [IX] vocab=`Research topic `kind`` values=``steal-scan`, `leaderboard`, `discovery`, `monitor`` admission=`Closed/deny-unknown; a new kind requires code; Tier 1 exposes only `steal-scan` and `monitor`` source=``O1:L9-L11,L19-L21,L27-L30`` (ledger-line: DEEP-O-001.discovery.md:L593)
- **O-001/IX-VOCAB002** [IX] vocab=`TaskSpec `trigger`` values=``Cron{expr}`, `SsbEvent{action_type filter}`, `Manual`` admission=`Closed/deny-unknown; Tier 1 is Cron+Manual; SsbEvent is Tier 2` source=``O2:L9-L11,L19-L21,L27-L30`` (ledger-line: DEEP-O-001.discovery.md:L594)
- **O-001/IX-VOCAB003** [IX] vocab=`TaskSpec stage` values=``Fetch`, `Classify`, `Gate`, `FileToBrain`, `Digest`, `SpawnResearchTree`, `RunPlanUnit`, `Notify`` admission=`Ordered closed enum; novel stage code goes through plan process; Tier 1 explicitly names Fetch/Classify/Gate/Digest` source=``O2:L9-L11,L19-L21`` (ledger-line: DEEP-O-001.discovery.md:L595)
- **O-001/IX-VOCAB004** [IX] vocab=`TaskSpec sink` values=``VaultPage`, `Digest`, `DashboardInbox`, `Toast`` admission=`Closed enum; Tier 1 explicitly names DashboardInbox` source=``O2:L10,L19-L20`` (ledger-line: DEEP-O-001.discovery.md:L596)
- **O-001/IX-VOCAB005** [IX] vocab=`Classifier output` values=``{class: enum, confidence: f32}`` admission=`Never free text; zero out-of-enum; low-confidence canary goes to `unsorted`` source=``O1:L31-L33`` (ledger-line: DEEP-O-001.discovery.md:L597)
- **O-001/IX-VOCAB006** [IX] vocab=`Vault write `mode`` values=``create`, `append`, `upsert`` admission=`Create/append inside a claimed namespace; upsert additionally requires machine-owned page class` source=``O3:L30-L35,L65-L68`` (ledger-line: DEEP-O-001.discovery.md:L598)
- **O-001/IX-VOCAB007** [IX] vocab=`Track status` values=``PLANNED`, `IN PROGRESS`, `DONE`, `DEFERRED`` admission=``DONE` has wiring, caller, smoke, and artifact gates beyond cargo-green` source=``O0:L42-L45`` (ledger-line: DEEP-O-001.discovery.md:L599)

### O-001/IX — Data, journal, and lifecycle surfaces

- **O-001/IX-DATA001** [IX] [SURFACE] surface=`Research topics` state=`User-editable manifest `[[research.topics]]`` lifecycle=`Existing hourly Cosmos loop enumerates topics; existing Cosmos artifacts can time-decay to `DECAYED`; drawer shows kind and last-run trust tier` (ledger-line: DEEP-O-001.discovery.md:L605)
- **O-001/IX-DATA002** [IX] surface=`Rejected ledger` state=`Store consulted by O-1 duplicate precheck` lifecycle=`Duplicate handling occurs before file/digest; exact record schema is not given` (ledger-line: DEEP-O-001.discovery.md:L606)
- **O-001/IX-DATA003** [IX] surface=`G-2 cassette` state=`External bytes keyed by run ID` lifecycle=`Detached-network, byte-identical replay; divergence detects unrecorded reads` (ledger-line: DEEP-O-001.discovery.md:L607)
- **O-001/IX-DATA004** [IX] surface=`TaskSpec` state=`SQLite rows loaded at boot` lifecycle=`Cron/manual dispatch at Tier 1; last-fired watermark protects each due tick; event trigger/missed-run hookup deferred` (ledger-line: DEEP-O-001.discovery.md:L608)
- **O-001/IX-DATA005** [IX] surface=`SSB classify row` state=`O-1 schema-version bump site` lifecycle=`Carries `{class, confidence, taint_tier}` and proves canary containment` (ledger-line: DEEP-O-001.discovery.md:L609)
- **O-001/IX-DATA006** [IX] [SURFACE] surface=``task_spec_fired`` state=`O-2 schema-version bump site` lifecycle=`Carries `{spec_id, trigger_cause, stage_path}` exactly once per due tick; reused by O-4 run-now` (ledger-line: DEEP-O-001.discovery.md:L610)
- **O-001/IX-DATA007** [IX] [SURFACE] surface=`Inbox intervention card` state=`Existing dashboard surface` lifecycle=`Created only for result-bearing runs; empty runs auto-archive and retain SSB only` (ledger-line: DEEP-O-001.discovery.md:L611)
- **O-001/IX-DATA008** [IX] [SURFACE] surface=`Markdown vault` state=`Configured mounted root` lifecycle=`Pure read/byte-identical round trip; claimed create/append; machine-owned upsert; external-editor ownership` (ledger-line: DEEP-O-001.discovery.md:L612)
- **O-001/IX-DATA009** [IX] surface=``namespace-claims`` state=`Root file mapping namespaces to machines` lifecycle=`Hard refusal and `claim_violation` for unclaimed writes; permits ELAI/TEIKOKU cohabitation` (ledger-line: DEEP-O-001.discovery.md:L613)
- **O-001/IX-DATA010** [IX] [SURFACE] surface=``vault_write`` state=`O-3 schema-version bump site` lifecycle=`Records namespace, page, and create/append/upsert mode` (ledger-line: DEEP-O-001.discovery.md:L614)
- **O-001/IX-DATA011** [IX] [SURFACE] surface=`Canvas projection` state=`Derived from TaskSpec rows plus J-60 SSB run state` lifecycle=`Read-only Tier 1; editing Tier 2; missing consumers are visually disconnected` (ledger-line: DEEP-O-001.discovery.md:L615)

### O-001/IX — Dependencies, rollout, production owners

- **O-001/IX-DEP001** [IX] unit=`O-1` status=``PLANNED`, `[KEYSTONE]`, effort `M-L`, Wave 1` owner=`Knowledge/Cosmos owner; live C-16 `live_cli` Cosmos tick` dep=`Consume frozen G-1/F-3/G-2; if manifest is hard-inlined, add a loader adapter without forking the loop` (ledger-line: DEEP-O-001.discovery.md:L621)
- **O-001/IX-DEP002** [IX] unit=`O-2` status=``PLANNED`, effort `M-L`, Wave 1` owner=``crates/orchestrator` **or** rules crate; C-9 task_runner interval` dep=`O-1 stage implementations; if no temporal due-query exists, ship fire-on-tick and defer missed-run monitoring` (ledger-line: DEEP-O-001.discovery.md:L622)
- **O-001/IX-DEP003** [IX] unit=`O-3` status=``PLANNED`, effort `M`, Wave 2` owner=`Knowledge/memory crate; O-1/O-2 FileToBrain or interim `elai brain write`` dep=`Needs a writer caller; if TEIKOKU path is cross-host inaccessible, use a fresh ELAI namespace and defer shared mount` (ledger-line: DEEP-O-001.discovery.md:L623)
- **O-001/IX-DEP004** [IX] unit=`O-4` status=``PLANNED`, effort `M`, Wave 3` owner=``crates/dashboard-v2`; live dashboard mode route + O-2 dispatcher` dep=`Queryable O-2 store; if absent, render a read-only fixture and never invent a client store` (ledger-line: DEEP-O-001.discovery.md:L624)
- **O-001/IX-DEPNOTE001** [IX] unit=`(sequencing note)` status=`` owner=`` dep=`O-1 and O-2 are explicitly parallel-safe in Wave 1 because their declared crate touch sets differ. O-3 follows when a FileToBrain/CLI caller exists. O-4 follows a queryable TaskSpec store, with a fixture-only Tier-1 fallback (`O0:L87-L96`). All four are product-surface units with no direct `PS-1..PS-6` contribution (`O0:L58-L60`).` (ledger-line: DEEP-O-001.discovery.md:L626)

### O-001/IX — Failure, refusal, and recovery paths

- **O-001/IX-FAIL001** [IX] failure=`Unknown topic kind` response=`Hard parse refusal; deny-unknown enum and negative test` source=``O1:L27-L30`` (ledger-line: DEEP-O-001.discovery.md:L632)
- **O-001/IX-FAIL002** [IX] failure=`Canary reaches prompt unlabelled` response=`Re-anchor classifier to taint-wrapped item; freeze fixture` source=``O1:L31-L33`` (ledger-line: DEEP-O-001.discovery.md:L633)
- **O-001/IX-FAIL003** [IX] failure=`Replay diverges` response=`Treat as leaked network read; route remaining fetch through VCR and add detached assertion` source=``O1:L34-L36`` (ledger-line: DEEP-O-001.discovery.md:L634)
- **O-001/IX-FAIL004** [IX] failure=`Cosmos manifest hard-inlined` response=`Stop in-place edit; use new loader+adapter; do not fork worker` source=``O1:L38-L39`` (ledger-line: DEEP-O-001.discovery.md:L635)
- **O-001/IX-FAIL005** [IX] failure=`Trust gate/cassette format would need modification` response=`Abort and flag; O-1 only consumes frozen gates` source=``O1:L45-L46`` (ledger-line: DEEP-O-001.discovery.md:L636)
- **O-001/IX-FAIL006** [IX] failure=`Unknown TaskSpec trigger/stage` response=`Hard parse refusal; strict enum and negative test` source=``O2:L27-L30`` (ledger-line: DEEP-O-001.discovery.md:L637)
- **O-001/IX-FAIL007** [IX] failure=`Due spec double-fires or never fires` response=`Gate with last-fired watermark and test duplicate firing` source=``O2:L31-L33`` (ledger-line: DEEP-O-001.discovery.md:L638)
- **O-001/IX-FAIL008** [IX] failure=`Empty runs spam inbox` response=`Result-count predicate sends them to auto-archive` source=``O2:L34-L36`` (ledger-line: DEEP-O-001.discovery.md:L639)
- **O-001/IX-FAIL009** [IX] failure=`Temporal monitor lacks due query` response=`Defer missed-run hookup, ship fire-on-tick, build no second monitor` source=``O2:L38-L39`` (ledger-line: DEEP-O-001.discovery.md:L640)
- **O-001/IX-FAIL010** [IX] failure=`Clock would move inside replay` response=`Abort and flag` source=``O2:L46-L47`` (ledger-line: DEEP-O-001.discovery.md:L641)
- **O-001/IX-FAIL011** [IX] failure=`Reader reformats mounted page` response=`Make reads pure; prohibit normalize-on-load; byte-identity test` source=``O3:L26-L29`` (ledger-line: DEEP-O-001.discovery.md:L642)
- **O-001/IX-FAIL012** [IX] failure=`Cross-namespace write succeeds` response=`Put claim check at sole write chokepoint; refusal test` source=``O3:L30-L32`` (ledger-line: DEEP-O-001.discovery.md:L643)
- **O-001/IX-FAIL013** [IX] failure=`Upsert edits user note` response=`Add namespace plus page-class predicate; refusal test` source=``O3:L33-L35`` (ledger-line: DEEP-O-001.discovery.md:L644)
- **O-001/IX-FAIL014** [IX] failure=`Shared vault inaccessible` response=`Fresh ELAI-claimed root; defer cross-host mounting` source=``O3:L37-L38`` (ledger-line: DEEP-O-001.discovery.md:L645)
- **O-001/IX-FAIL015** [IX] failure=`ELAI schema would require user-note edits` response=`Abort; ELAI adapts to vault` source=``O3:L45-L46`` (ledger-line: DEEP-O-001.discovery.md:L646)
- **O-001/IX-FAIL016** [IX] failure=`Palette admits a free-form node` response=`Bind palette to O-2 enum at build` source=``O4:L27-L30`` (ledger-line: DEEP-O-001.discovery.md:L647)
- **O-001/IX-FAIL017** [IX] failure=`Missing consumer looks connected` response=`Derive disconnection from edge set and assert visual treatment` source=``O4:L31-L33`` (ledger-line: DEEP-O-001.discovery.md:L648)
- **O-001/IX-FAIL018** [IX] failure=`Canvas tries to execute` response=`Call O-2 dispatcher/FSM and prove fire-row path` source=``O4:L34-L36,L46-L47`` (ledger-line: DEEP-O-001.discovery.md:L649)
- **O-001/IX-FAIL019** [IX] failure=`O-2 store unavailable` response=`Read-only fixture projection; never create canvas-side store` source=``O4:L38-L39`` (ledger-line: DEEP-O-001.discovery.md:L650)

### O-001/IX — Exact recon and verification commands

- **O-001/IX-RECON001** [IX] unit=`O-1` purpose=`Cosmos manifest/classifier recon` command=``grep -rn "cosmos_worker\` (ledger-line: DEEP-O-001.discovery.md:L658)
- **O-001/IX-RECON002** [IX] unit=`O-1` purpose=`Taint API recon` command=``grep -rn "taint\` (ledger-line: DEEP-O-001.discovery.md:L659)
- **O-001/IX-RECON003** [IX] unit=`O-1` purpose=`Wiring proof` command=``python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/O-plan/O-1_research_daemon_generalized_cosmos.md --run-trigger`` (ledger-line: DEEP-O-001.discovery.md:L660)
- **O-001/IX-RECON004** [IX] unit=`O-1` purpose=`Production caller proof` command=``python .claude/tools/production_caller_check.py --files <changed .rs> --json`` (ledger-line: DEEP-O-001.discovery.md:L661)
- **O-001/IX-RECON005** [IX] unit=`O-2` purpose=`Rules-store recon` command=``grep -rn "rules\` (ledger-line: DEEP-O-001.discovery.md:L662)
- **O-001/IX-RECON006** [IX] unit=`O-2` purpose=`Interval-registration recon` command=``grep -rn "task_runner\` (ledger-line: DEEP-O-001.discovery.md:L663)
- **O-001/IX-RECON007** [IX] [SURFACE] unit=`O-2` purpose=`Intervention-card recon` command=``grep -rn "intervention\` (ledger-line: DEEP-O-001.discovery.md:L664)
- **O-001/IX-RECON008** [IX] unit=`O-2` purpose=`Wiring proof` command=``python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/O-plan/O-2_scheduled_taskspec_substrate.md --run-trigger`` (ledger-line: DEEP-O-001.discovery.md:L665)
- **O-001/IX-RECON009** [IX] unit=`O-2` purpose=`Production caller proof` command=``python .claude/tools/production_caller_check.py --files <changed .rs> --json`` (ledger-line: DEEP-O-001.discovery.md:L666)
- **O-001/IX-RECON010** [IX] unit=`O-3` purpose=`Machine-memory boundary recon` command=``grep -rn "memory_facts\` (ledger-line: DEEP-O-001.discovery.md:L667)
- **O-001/IX-RECON011** [IX] unit=`O-3` purpose=`Privacy boundary recon` command=``grep -rn "life.layer\` (ledger-line: DEEP-O-001.discovery.md:L668)
- **O-001/IX-RECON012** [IX] [SURFACE] unit=`O-3` purpose=`Wiring proof` command=``python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/O-plan/O-3_brain_vault_contract_mounting.md --run-trigger`` (ledger-line: DEEP-O-001.discovery.md:L669)
- **O-001/IX-RECON013** [IX] unit=`O-3` purpose=`Production caller proof` command=``python .claude/tools/production_caller_check.py --files <changed .rs> --json`` (ledger-line: DEEP-O-001.discovery.md:L670)
- **O-001/IX-RECON014** [IX] unit=`O-4` purpose=`Mode registration recon` command=``grep -rn "mode.bar\` (ledger-line: DEEP-O-001.discovery.md:L671)
- **O-001/IX-RECON015** [IX] unit=`O-4` purpose=`Run-state overlay recon` command=``grep -rn "J-60\` (ledger-line: DEEP-O-001.discovery.md:L672)
- **O-001/IX-RECON016** [IX] [SURFACE] unit=`O-4` purpose=`Wiring proof` command=``python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/O-plan/O-4_canvas_workflow_mode.md --run-trigger`` (ledger-line: DEEP-O-001.discovery.md:L673)
- **O-001/IX-RECON017** [IX] [SURFACE] unit=`O-4` purpose=`Svelte/Tauri reachability and symmetry` command=``python .claude/tools/svelte_mount_reachability.py` and `python .claude/tools/tauri_ipc_symmetry_check.py`` (ledger-line: DEEP-O-001.discovery.md:L674)
- **O-001/IX-RECON018** [IX] unit=`O-4` purpose=`Real UI wire smoke` command=``bun .claude/tools/visual/ui-wire-smoke.mjs ...`` (ledger-line: DEEP-O-001.discovery.md:L675)

### O-001/IX — Category coverage matrix

- **O-001/IX-CAT001** [IX] dimension=`Commands/actions` coverage=`Cosmos tick, TaskSpec boot/load/fire, run-now, FileToBrain, `elai brain write`, list/get Tauri reads, create/append/upsert` (ledger-line: DEEP-O-001.discovery.md:L782)
- **O-001/IX-CAT002** [IX] dimension=`Inputs/outputs` coverage=`Topic record fields; classifier object; TaskSpec trigger/stage/sink enums; three SSB event shapes; Markdown page output` (ledger-line: DEEP-O-001.discovery.md:L783)
- **O-001/IX-CAT003** [IX] dimension=`Workflows/state` coverage=`Fetch/classify/dedupe/file/digest; due-tick/watermark/card; mount/read/claim/write; project/chrome/disconnect/run` (ledger-line: DEEP-O-001.discovery.md:L784)
- **O-001/IX-CAT004** [IX] dimension=`Schema/data/lifecycle` coverage=`SQLite TaskSpecs, rejected ledger, run-ID cassette, namespace claims, vault paths, auto-archive, existing `DECAYED` state` (ledger-line: DEEP-O-001.discovery.md:L785)
- **O-001/IX-CAT005** [IX] dimension=`Config/defaults` coverage=`Named fields and strict closed enums; kill-switch convention; no concrete defaults for cadence, clock, enabled state, or paths` (ledger-line: DEEP-O-001.discovery.md:L786)
- **O-001/IX-CAT006** [IX] dimension=`UI/API/CLI` coverage=`CosmosDrawer extension, intervention cards, optional external-editor/read view, Svelte WORKFLOW, two Tauri reads, interim brain CLI` (ledger-line: DEEP-O-001.discovery.md:L787)
- **O-001/IX-CAT007** [IX] dimension=`Integrations/events` coverage=`B-42/C-9/3F/E-5/D-5/G-1/F-3/G-2/H-12/J-31/J-60; classify/fire/vault/refusal rows` (ledger-line: DEEP-O-001.discovery.md:L788)
- **O-001/IX-CAT008** [IX] dimension=`Permissions/security/privacy` coverage=`Taint, tool role filter, closed schemas, frozen canaries, namespace claims, user-note ban, D-5 boundary, no client runtime` (ledger-line: DEEP-O-001.discovery.md:L789)
- **O-001/IX-CAT009** [IX] dimension=`Failure/recovery` coverage=`Strict refusals, adapter/fallback paths, watermark, auto-archive, pure read, write chokepoint, fixture projection, explicit aborts` (ledger-line: DEEP-O-001.discovery.md:L790)
- **O-001/IX-CAT010** [IX] dimension=`Determinism/idempotency/concurrency` coverage=`Detached byte replay, supervisor-plane clock, recorded trigger input, due-tick watermark; distributed/vault concurrency unspecified` (ledger-line: DEEP-O-001.discovery.md:L791)
- **O-001/IX-CAT011** [IX] dimension=`Observability` coverage=`SSB receipt chain, CosmosDrawer trust tier, inbox cards, J-60 chrome, visible disconnection, off-agent artifacts` (ledger-line: DEEP-O-001.discovery.md:L792)
- **O-001/IX-CAT012** [IX] dimension=`Tests/done gates` coverage=`Wiring/caller checks, negative parses, canaries, detached replay, multi-tick smoke, two-owner vault smoke, real-browser smoke` (ledger-line: DEEP-O-001.discovery.md:L793)
- **O-001/IX-CAT013** [IX] dimension=`Constraints/non-goals` coverage=`No generic engine, custom kinds/nodes, second scheduler/inbox/fact store, vault conversion, user-note edits, canvas runtime, trust-gate edits` (ledger-line: DEEP-O-001.discovery.md:L794)
- **O-001/IX-CAT014** [IX] dimension=`Dependencies/status/deferred` coverage=`All four PLANNED; exact waves/effort; Tier 1/Tier 2 and fork conditions recorded above` (ledger-line: DEEP-O-001.discovery.md:L795)

---

## Summary-delta records (SD-*)

Deep-vs-summary corrections and expansions. Deep originals win target state; summary wording retained as loser where noted.

### O-001/SD — Material corrections / narrowed readings

- **O-001/SD-MC001** [Δ] title=`Digest patches are per-unit, not literally identical across every unit.` (ledger-line: DEEP-O-001.discovery.md:L683): O0 says `A1..A5` feed per-unit `stub_ban`/`REDTEAM` (`O0:L4-L7,L55-L56`). The detail plans specialize them: O-1 applies A1/A3, O-2 A2/A3, O-3 A4, and O-4 A3. The summary-ledger claim that every unit enforces the same A1, A3, and A4 set is broader than the originals.
- **O-001/SD-MC002** [Δ] title=`The engagement falsifier is not present in these five originals.` (ledger-line: DEEP-O-001.discovery.md:L684): The summary ledger’s digest-open, `For-You` click, `N`-day zero-engagement, and honest topic-decay requirements are not stated in O0 or O1–O4. O1 does state an existing artifact lifecycle that decays to `DECAYED` (`O1:L7`), but it does not connect that state to opens/clicks or an `N`-day threshold. Those engagement requirements remain summary-only in this source scope.
- **O-001/SD-MC003** [Δ] title=`O-4 editing is not Tier 1.` (ledger-line: DEEP-O-001.discovery.md:L685): The original makes Tier 1 read-only inspection and explicitly defers TaskSpec write-back plus provenance drill-down to Tier 2 (`O4:L19-L21`). Any summary reading of node editing as an immediate ship-now feature needs this tier qualifier.
- **O-001/SD-MC004** [Δ] title=`O-2 full vocabulary is larger than its Tier-1 runtime.` (ledger-line: DEEP-O-001.discovery.md:L686): The schema names eight stages and four sinks, but Tier 1 explicitly names only Fetch/Classify/Gate/Digest and DashboardInbox; SsbEvent, SpawnResearchTree, RunPlanUnit, and missed-run wiring are Tier 2 (`O2:L9-L10,L19-L21`). Schema availability must not be confused with production wiring.
- **O-001/SD-MC005** [Δ] title=`O-1 full topic enum is larger than Tier 1.` (ledger-line: DEEP-O-001.discovery.md:L687): Four kinds are schema-valid, but only `steal-scan` and `monitor` ship now; `leaderboard`, `discovery`, and living-page upserts are deferred (`O1:L19-L21`).
- **O-001/SD-MC006** [Δ] title=`Exactly-once is scoped to a due tick.` (ledger-line: DEEP-O-001.discovery.md:L688): The original’s concrete mechanism is a last-fired watermark and the acceptance phrase “exactly once per due tick” (`O2:L31-L33,L65-L68`); it does not claim global exactly-once under crash, multi-process, or distributed scheduling.
- **O-001/SD-MC007** [Δ] title=`Privacy is a staged safety barrier.` (ledger-line: DEEP-O-001.discovery.md:L689): Personal-to-project/cloud flow is a hard exclusion/stub ban, but the actual D-5 read-path wiring is Tier 2 and must land before personal topics become mountable into project context (`O3:L10-L11,L18-L20,L60-L64,L80-L81`).
- **O-001/SD-MC008** [Δ] title=`“Dark/absent” is not the exact status vocabulary of the originals.` (ledger-line: DEEP-O-001.discovery.md:L690): All four details say `PLANNED`; O0 defines the formal statuses. The originals identify new versus reused mechanisms but do not independently inventory current code to prove each component dark (`O0:L42-L45`; `O1:L3`; `O2:L3`; `O3:L3`; `O4:L3`).
- **O-001/SD-MC009** [Δ] title=`The trust-spine boundary is more precise than a blanket byte claim.` (ledger-line: DEEP-O-001.discovery.md:L691): O1 says G-1 wraps fetched bytes, F-3 applies to the research role, and G-2 records external bytes; integration names fetch/classify/sink seams (`O1:L10,L23-L24,L31-L36`). Metadata, redirect, attachment, and cache coverage remain unspecified.
- **O-001/SD-MC010** [Δ] title=`No new O-4 event is planned.` (ledger-line: DEEP-O-001.discovery.md:L692): O0 explicitly says O-4 reuses O-2’s fire row, while O-1/O-2/O-3 add schema action types (`O0:L80-L85`).

### O-001/SD — Original-only or materially expanded detail

- **O-001/SD-OE001** [Δ] (ledger-line: DEEP-O-001.discovery.md:L696): O0 supplies full wiring doctrine, release gates, `WIRE-DARK[O-N]`, `cross_lang_diff`, schema-baseline procedure, kill-switch convention, `blake3` requirement, affected-crate test scope, no `.unwrap()`/`.expect()` production rule, and anti-slopsquatting requirement (`O0:L42-L64,L79-L103`).
- **O-001/SD-OE002** [Δ] (ledger-line: DEEP-O-001.discovery.md:L697): O1 supplies the complete topic record shape, including `id`, `sources[]`, `cadence`, `confidence_threshold`, and `sink`; untrusted preamble/item fence/per-kind schema; rejected ledger; layered digest; `f32` confidence; run-ID cassette keying; detached-network byte identity; CosmosDrawer trust-tier chrome; adapter fork; and exact hard-input smoke (`O1:L9-L10,L23-L39,L64-L77`).
- **O-001/SD-OE003** [Δ] (ledger-line: DEEP-O-001.discovery.md:L698): O2 supplies trigger operands (`Cron{expr}`, `SsbEvent{action_type filter}`), the eight stage values, boot-time loading, the last-fired watermark, result-count auto-archive, trigger-time-as-recorded-input rule, intervention ingestion recon, fire-on-tick fork, and `1-minute` empty-Cron attack (`O2:L9-L11,L27-L47,L65-L81`).
- **O-001/SD-OE004** [Δ] (ledger-line: DEEP-O-001.discovery.md:L699): O3 supplies the exact topic path, root claims location, write modes, living-page classes, pure-read/no-normalize rule, byte-identical round trip, single write chokepoint, page-class upsert check, cross-host fallback, optional/no-new-panel UI stance, interim CLI, and two-owner smoke (`O3:L9-L10,L18-L46,L65-L81`).
- **O-001/SD-OE005** [Δ] (ledger-line: DEEP-O-001.discovery.md:L700): O4 supplies Svelte 5/Tauri v2 ownership, two read-command names, ts-rs drift ban, read-only Tier 1, edge-set-derived disconnection, real browser smoke, fixture projection fallback, and the prohibition on a client-side spec store (`O4:L19-L25,L31-L47,L49-L89`).

### O-001/SD — Summary questions resolved by originals

- **O-001/SD-RQ001** [Δ] (ledger-line: DEEP-O-001.discovery.md:L704): Topic-manifest fields are now enumerated, though their types/defaults remain open (`O1:L10`).
- **O-001/SD-RQ002** [Δ] (ledger-line: DEEP-O-001.discovery.md:L705): TaskSpec’s stage enum is now fully enumerated (`O2:L10`).
- **O-001/SD-RQ003** [Δ] (ledger-line: DEEP-O-001.discovery.md:L706): Cron and SSB-event operands are named (`O2:L10`).
- **O-001/SD-RQ004** [Δ] (ledger-line: DEEP-O-001.discovery.md:L707): O-1 replay is keyed by run ID and tested byte-identically with networking detached (`O1:L34-L36`).
- **O-001/SD-RQ005** [Δ] (ledger-line: DEEP-O-001.discovery.md:L708): O-2’s first idempotency mechanism is a last-fired watermark row (`O2:L31-L33`).
- **O-001/SD-RQ006** [Δ] (ledger-line: DEEP-O-001.discovery.md:L709): Auto-archive is keyed off pipeline result count and leaves the SSB row (`O2:L34-L36`).
- **O-001/SD-RQ007** [Δ] (ledger-line: DEEP-O-001.discovery.md:L710): O-3’s machine-owned upsert classes are rankings/queue/index (`O3:L33-L35`).
- **O-001/SD-RQ008** [Δ] (ledger-line: DEEP-O-001.discovery.md:L711): O-3 reads must not normalize content (`O3:L26-L29`).
- **O-001/SD-RQ009** [Δ] (ledger-line: DEEP-O-001.discovery.md:L712): O-4 missing-consumer rendering is derived from the TaskSpec edge set (`O4:L31-L33`).
- **O-001/SD-RQ010** [Δ] (ledger-line: DEEP-O-001.discovery.md:L713): O-4’s frontend pair and validation commands are named (`O4:L67,L75-L80`).

### O-001/SD — Summary questions not resolved by originals

- **O-001/SD-UQ001** [Δ/A] (ledger-line: DEEP-O-001.discovery.md:L717): `VaultPage` versus `FileToBrain` naming remains unresolved: O2 calls the former a sink and the latter a stage; O3 calls FileToBrain both a stage and sink seam (`O2:L10,L88-L89`; `O3:L57-L70`).
- **O-001/SD-UQ002** [Δ/A] (ledger-line: DEEP-O-001.discovery.md:L718): The `D-5` reference namespace remains unresolved; O3 repeats “D-5 life-layer denylist” but does not reconcile it to the differently titled D-plan file noted by the summary pass (`O3:L10,L15,L20,L43,L55,L64,L81`).
- **O-001/SD-UQ003** [Δ/A] (ledger-line: DEEP-O-001.discovery.md:L719): `unsorted` is still not typed as a class, queue, path, or sink (`O1:L32,L66,L75`).
- **O-001/SD-UQ004** [Δ/A] (ledger-line: DEEP-O-001.discovery.md:L720): Class-enum values and confidence range/calibration remain unstated (`O1:L10,L31-L33`).
- **O-001/SD-UQ005** [Δ/A] (ledger-line: DEEP-O-001.discovery.md:L721): Shared-vault locking and atomic-write semantics remain unstated.
- **O-001/SD-UQ006** [Δ/A] (ledger-line: DEEP-O-001.discovery.md:L722): Canvas write-side IPC, edit revisions, concurrent edits, authorization, and save errors remain unstated.

---

## Ambiguity and conflict registers (AC-*)

### O-001/AC — Internal staging tensions

- **O-001/AC-ST001** [A] title=`O-4 read-only Tier 1 versus run-now verification.` (ledger-line: DEEP-O-001.discovery.md:L743): Tier 1 says read-only inspection, while M3, the production hook, observable effect, verifier, and integration smoke require a live run-now click (`O4:L19-L21,L34-L36,L68-L82`). Run-now is not assigned explicitly to Tier 1 or Tier 2.
- **O-001/AC-ST002** [A] title=`O-3 privacy ban versus deferred enforcement.` (ledger-line: DEEP-O-001.discovery.md:L744): The safety rule is unconditional, yet D-5 read-path wiring is Tier 2 (`O3:L10-L11,L18-L20,L60-L64,L80-L81`). The original partially resolves this by forbidding personal topics in project context until Tier 2, but it does not state what Tier-1 mount validation enforces.
- **O-001/AC-ST003** [A] title=`O-3 bidirectional sync versus no dual-write.` (ledger-line: DEEP-O-001.discovery.md:L745): Bidirectional synchronization of “views” is deferred while memory/vault dual-write is forbidden (`O3:L20,L42,L53-L64`). The permitted future sync direction and derived-view semantics need definition.
- **O-001/AC-ST004** [A] title=`O-2 schema versus Tier wiring.` (ledger-line: DEEP-O-001.discovery.md:L746): `FileToBrain`, `Notify`, `VaultPage`, `Digest`, and `Toast` appear in the full schema, while the Tier blocks state only some inclusions and two specific deferrals (`O2:L9-L10,L19-L21`). The shipping status of FileToBrain, Notify, and non-Dashboard sinks is not fully assigned.
- **O-001/AC-ST005** [A] title=`Manual trigger “on interval.”` (ledger-line: DEEP-O-001.discovery.md:L747): The produced-symbol block calls the Cron/Manual dispatcher “on the task_runner interval” (`O2:L52`), but a manual action normally has a direct entrypoint; the original does not settle whether Manual is polled, enqueued, or invoked synchronously.

### O-001/AC — Naming and type ambiguities

- **O-001/AC-NA001** [A] title=``VaultPage` / `FileToBrain`:` (ledger-line: DEEP-O-001.discovery.md:L751): sink versus stage versus production seam is inconsistent across O2/O3.
- **O-001/AC-NA002** [A] title=`O-1 stages consumed by O-2:` (ledger-line: DEEP-O-001.discovery.md:L752): O2 calls Fetch/Classify/Gate/Digest “the ones O-1 already provides,” but O1’s named workflow includes duplicate-precheck, file, layered digest, and confidence gate without defining corresponding reusable stage symbols (`O1:L10`; `O2:L20,L56`).
- **O-001/AC-NA003** [A] title=`Classifier `class`:` (ledger-line: DEEP-O-001.discovery.md:L753): enum members are not listed. `unsorted` may be a member but is never typed as one.
- **O-001/AC-NA004** [A] title=`Confidence:` (ledger-line: DEEP-O-001.discovery.md:L754): representation is `f32`, but range, threshold comparison, calibration, NaN handling, and per-kind/per-sink interaction are absent.
- **O-001/AC-NA005** [A] title=`Topic schema:` (ledger-line: DEEP-O-001.discovery.md:L755): field names are given, but field types for `id`, `sources[]`, `cadence`, threshold, and sink; defaults; enabled state; uniqueness; and versioning are absent.
- **O-001/AC-NA006** [A] title=`TaskSpec identity:` (ledger-line: DEEP-O-001.discovery.md:L756): no `spec_id` field is shown in the schema even though the event contains `spec_id`; revision, owner, timestamps, operands, and stage-output typing are absent.
- **O-001/AC-NA007** [A] title=``namespace-claims`:` (ledger-line: DEEP-O-001.discovery.md:L757): filename has no extension and its serialization, namespace grammar, claim precedence, acquisition/release, and conflict syntax are absent.
- **O-001/AC-NA008** [A] title=`Machine-owned page detection:` (ledger-line: DEEP-O-001.discovery.md:L758): rankings/queue/index are named, but ownership metadata and treatment of a user-edited machine page are absent.
- **O-001/AC-NA009** [A] title=`Missing consumer:` (ledger-line: DEEP-O-001.discovery.md:L759): derived from the edge set, but invalid/disabled/unavailable consumers and sink termination are not distinguished.
- **O-001/AC-NA010** [A] title=`Provenance tier:` (ledger-line: DEEP-O-001.discovery.md:L760): O-1 produces and O-4 displays it, but the tier vocabulary and edge aggregation rule are absent.
- **O-001/AC-NA011** [A] title=`Classify action type:` (ledger-line: DEEP-O-001.discovery.md:L761): O0 says O-1 adds a classify row but does not give its exact action-type string (`O0:L82`).

### O-001/AC — Concurrency, lifecycle, and recovery gaps

- **O-001/AC-CL001** [A] (ledger-line: DEEP-O-001.discovery.md:L765): Last-fired watermark transaction boundaries, crash-before/after-row behavior, multi-process dispatch, retry idempotency, and catch-up/coalescing remain unspecified.
- **O-001/AC-CL002** [A] (ledger-line: DEEP-O-001.discovery.md:L766): Trigger timezone, Cron dialect, jitter, startup behavior, overlap, maximum run duration, cancellation, pause/resume, deletion, and retention remain unspecified.
- **O-001/AC-CL003** [A] (ledger-line: DEEP-O-001.discovery.md:L767): `SsbEvent` recursion/loop prevention, filter matching, event ordering, and replay interaction remain unspecified.
- **O-001/AC-CL004** [A] (ledger-line: DEEP-O-001.discovery.md:L768): Fetch retry/backoff, source rate limits, partial responses, cache behavior, provider degradation, secret handling, unsafe URLs, and partial pipeline failure remain unspecified.
- **O-001/AC-CL005** [A] (ledger-line: DEEP-O-001.discovery.md:L769): Rejected-ledger fingerprinting, false-positive handling, cross-topic scope, retention, and override remain unspecified.
- **O-001/AC-CL006** [A] (ledger-line: DEEP-O-001.discovery.md:L770): Vault atomicity, locking, simultaneous writers, symlink/path traversal rules, case sensitivity, partial-write recovery, backup/restore, corruption handling, and file-watcher behavior remain unspecified.
- **O-001/AC-CL007** [A] (ledger-line: DEEP-O-001.discovery.md:L771): Inbox/card read state, dismissal, retention, failure cards, deduplication, and the exact auto-archived object remain unspecified.
- **O-001/AC-CL008** [A] (ledger-line: DEEP-O-001.discovery.md:L772): Canvas branch/fan-in/fan-out/cycle representation, layout persistence, accessibility, save conflicts, undo/redo, cancellation, duplicate run-now clicks, and permission checks remain unspecified.

### O-001/AC — Status and evidence caveat

- **O-001/AC-STEV001** [A] title=`Status and evidence caveat` (ledger-line: DEEP-O-001.discovery.md:L776): All four unit files are `PLANNED`; all existing-ground-truth blocks are explicitly labelled `PLAN-asserted` and direct executors to re-grep under W-7. Therefore this ledger describes designed and asserted behavior, not implementation-confirmed shipment. `PROOF_SPINE: none` does not remove release evidence: the O0 done definition and each unit’s off-agent wiring/integration smoke remain blocking plan gates (`O0:L42-L64`; `O1:L71-L77`; `O2:L72-L78`; `O3:L72-L78`; `O4:L75-L82`).

### SUMMARY/AC — Ambiguities and contradictions from summary pass

- **SUMMARY/AC-001** [A] title=``D-5` namespace collision:` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L345): the summary calls D-5 a “life-layer denylist,” while the canonical ELAI file named `D-plan/D-5_bai_thompson_promotion.md` has a different title. `D-5` may instead be a section/requirement ID inside the TEIKOKU/source-design document. Do not bind this privacy feature to ELAI D-plan/D-5 without reconciliation.
- **SUMMARY/AC-002** [A] title=`Topic kinds versus stage kinds:` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L346): O-1 explicitly enumerates topic `kind`, but O-2 says it consumes “O-1 stage kinds.” The summary does not enumerate a stage-kind enum or say whether `fetch`, `enum-classify`, `dup-precheck`, `file`, and `digest` are its values.
- **SUMMARY/AC-003** [A] title=`Sink naming mismatch:` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L347): O-2's sink enum contains `VaultPage`; O-3 calls the O-1/O-2 sink `FileToBrain`. Their relation is unspecified.
- **SUMMARY/AC-004** [A] title=`Crate ownership unresolved:` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L348): the TaskSpec table is assigned to `crates/orchestrator (or rules crate)`.
- **SUMMARY/AC-005** [A] title=`VCR undefined here:` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L349): G-2 VCR is named but its record/replay boundary, file format, and interaction with the G-2 reproducible-run artifact are not restated.
- **SUMMARY/AC-006** [A] title=`“Every fetched byte” boundary unspecified:` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L350): the summary does not say whether metadata, redirects, attachments, tool outputs, and cached bytes all share the same taint/filter/VCR treatment.
- **SUMMARY/AC-007** [A] title=`Classifier contract incomplete:` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L351): confidence type/range, class enum values, abstention, invalid-output repair/refusal, and ownership of the off-agent classifier are not specified.
- **SUMMARY/AC-008** [A] title=`Canary contract incomplete:` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L352): frozen canary corpus, injection frequency, matching rules, and the exact definition of “steal sink” are absent.
- **SUMMARY/AC-009** [A] title=``unsorted` undefined:` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L353): it may be a classifier class, holding queue, vault namespace, or sink.
- **SUMMARY/AC-010** [A] title=`Topic manifest schema incomplete:` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L354): only editability and the four `kind` values are named. IDs, query/source fields, cadence, enabled state, ownership, validation, versioning, and persistence are absent.
- **SUMMARY/AC-011** [A] title=`Cosmos scheduling detail incomplete:` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L355): “hourly tick” is stated for the existing loop, but timezone, jitter, startup behavior, locking, and overlap rules are absent.
- **SUMMARY/AC-012** [A] title=`TaskSpec schema incomplete:` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L356): IDs, revisions, stage operands, stage output typing, sink operands, enabled/paused state, ownership, and timestamps are unstated.
- **SUMMARY/AC-013** [A] title=`Exactly-once scope incomplete:` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L357): “exactly once per tick” does not settle transaction boundaries, crash recovery, multiple processes, retried dispatch, or idempotency keys.
- **SUMMARY/AC-014** [A] title=`Missed-run semantics incomplete:` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L358): detection is named, but catch-up, skip, coalesce, maximum backlog, and notification policies are not.
- **SUMMARY/AC-015** [A] title=`Auto-archive semantics incomplete:` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L359): the borrowed flow names auto-archive without stating which artifact archives, when, or under what completion criteria.
- **SUMMARY/AC-016** [A] title=`Result-bearing predicate undefined:` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L360): therefore the empty-fire/card negative cannot yet be implemented from the summary alone.
- **SUMMARY/AC-017** [A] title=`SSB schemas incomplete:` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L361): row version, timestamps/clock domain, parent/run IDs, error fields, and replay ordering are absent for classify and fire events.
- **SUMMARY/AC-018** [A] title=`Vault claims format incomplete:` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L362): filename spelling, schema, namespace syntax, precedence, conflict resolution, and claim acquisition/release are unstated.
- **SUMMARY/AC-019** [A] title=`Shared-vault concurrency incomplete:` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L363): atomicity, locking, simultaneous writers, external-editor changes, and recovery from partial writes are unstated.
- **SUMMARY/AC-020** [A] title=`User-note boundary incomplete:` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L364): the summary prohibits “any mutation of a user note” but does not define how a user note is distinguished from an ELAI-owned page or whether append counts as mutation.
- **SUMMARY/AC-021** [A] title=`Guarded upsert semantics incomplete:` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L365): create/replace/merge behavior, conflict detection, and page ownership metadata are not stated.
- **SUMMARY/AC-022** [A] title=`Vault page schema incomplete:` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L366): required frontmatter keys, wikilink validity, slug normalization, encoding, and ranking/queue/index update rules are absent.
- **SUMMARY/AC-023** [A] title=`Mount validation incomplete:` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L367): the summary does not state what happens when an existing TEIKOKU vault violates the contract or contains unknown namespaces.
- **SUMMARY/AC-024** [A] title=`Privacy enforcement point incomplete:` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L368): D-5 denylist is named, but config source, read boundaries, redaction behavior, local-versus-cloud distinction, and failure evidence are not.
- **SUMMARY/AC-025** [A] title=`Provenance tiers incomplete:` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L369): O-4 edges carry a tier and consume O-1 tiers, but no tier vocabulary or admission rules are supplied.
- **SUMMARY/AC-026** [A] title=`Graph projection incomplete:` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L370): fan-in/out, branching, cycles, ordering, graph layout, sink nodes, trigger nodes, and invalid TaskSpec rendering are not specified.
- **SUMMARY/AC-027** [A] title=`Missing-consumer semantics incomplete:` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L371): the summary does not define how a consumer is deemed missing or distinguish missing from temporarily unavailable/disabled.
- **SUMMARY/AC-028** [A] title=`Wire-dark semantics incomplete:` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L372): it is unclear whether “wire-dark” means no event producer, stale data, an unresolved type edge, or only a visual state.
- **SUMMARY/AC-029** [A] title=`Editing API incomplete:` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L373): only read-side Tauri commands are named; the command that persists node edits, validation errors, revisions, and concurrent-edit handling is absent.
- **SUMMARY/AC-030** [A] title=`Run-now policy incomplete:` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L374): authorization, confirmation, risk gates, cancellation, duplicate clicks, and error rendering are not stated.
- **SUMMARY/AC-031** [A] title=`Engagement decay incomplete:` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L375): `N`, minimum sample, open/click identity, offline collection, privacy, decay function, reactivation, and pack-level versus topic-level scope are absent.
- **SUMMARY/AC-032** [A] title=`Proof relationship deliberately absent:` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L376): with `PROOF_SPINE: none`, the summary gives observable effects but no PS metric; originals should clarify what evidence is release-blocking versus product telemetry.
- **SUMMARY/AC-033** [A] title=`Status evidence is asserted, not verified:` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L377): “already,” “shipped,” “TEIKOKU-proven,” and “dark/absent” are plan assertions in a MODERATE-grounding summary, not code-ledger findings.

---

## Inference records (I-*)

### O-001/INF — Inferred system behavior (not explicit, not counted in deep E footer)

- **O-001/O-INF01** [I] (ledger-line: DEEP-O-001.discovery.md:L726): Full completion likely presents a single journey: configure a typed topic, schedule a closed pipeline, run trusted/replayable research, send useful output to inbox or vault, and inspect/edit/run the same spec on the canvas. This is a synthesis of declared composition edges, not a verbatim golden journey.
- **O-001/O-INF02** [I] (ledger-line: DEEP-O-001.discovery.md:L727): O-4 likely folds O-2’s `task_spec_fired` plus stage-specific rows into J-60 chrome, but the fold key and error-state model are not stated.
- **O-001/O-INF03** [I] (ledger-line: DEEP-O-001.discovery.md:L728): `VaultPage` likely denotes a terminal destination while `FileToBrain` denotes the executable stage that invokes it, but the originals never define that mapping.
- **O-001/O-INF04** [I] (ledger-line: DEEP-O-001.discovery.md:L729): `unsorted` likely functions as an abstention/quarantine destination for low-confidence or adversarial items, but its schema and review lifecycle are unknown.
- **O-001/O-INF05** [I] (ledger-line: DEEP-O-001.discovery.md:L730): O-1’s `sink` topic field likely chooses or parameterizes an O-2 sink, but the files do not state the adapter or validation rule.
- **O-001/O-INF06** [I] (ledger-line: DEEP-O-001.discovery.md:L731): O-4’s typed stage node can satisfy J-31’s “bound unit” rule if each node binds a concrete TaskSpec stage implementation; that binding record is not specified.
- **O-001/O-INF07** [I] (ledger-line: DEEP-O-001.discovery.md:L732): O-4’s provenance-bearing dataflow edge can satisfy J-31’s “verified claim” rule if its tier is journal-derived and validated; the proof record is not specified.
- **O-001/O-INF08** [I] (ledger-line: DEEP-O-001.discovery.md:L733): A last-fired watermark likely uses `(spec_id, due-window)` or an equivalent identity, but no key or transaction scope is stated.
- **O-001/O-INF09** [I] (ledger-line: DEEP-O-001.discovery.md:L734): The safest reading of O3’s privacy staging is that ordinary non-personal mounts can ship in Tier 1, while personal topics cannot enter project-context mounts until Tier 2.
- **O-001/O-INF10** [I] (ledger-line: DEEP-O-001.discovery.md:L735): Because O-3 requires pure reads and permits external editors, robust implementation likely needs atomic writes and external-change detection, but neither mechanism is designed here.
- **O-001/O-INF11** [I] (ledger-line: DEEP-O-001.discovery.md:L736): Because O-4 Tier 1 can use a fixture, that release may demonstrate projection UX without satisfying full live production reachability; the final done gate still appears to require the live O-2 caller.
- **O-001/O-INF12** [I] (ledger-line: DEEP-O-001.discovery.md:L737): O0’s `memory-writer ran` completion item likely refers to plan/session memory bookkeeping, not O-3 vault writes, but this is not defined in O sources.

### SUMMARY/INF — Inferred composition (not explicit requirements)

- **SUMMARY/O.INF01** [I] basis=`O-2 consumes O-1 stage kinds; O-3 names an O-1/O-2 sink; O-4 projects/runs O-2.` confirm=`Confirm exact golden journey and which sinks each topic kind uses.` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L332): The intended end-to-end product likely lets a user define research topics, schedule their closed stages, persist selected results into the mounted vault, inspect results/inbox state, and edit/run the same workflow visually.
- **SUMMARY/O.INF02** [I] basis=`O-1/O-2 journal rows plus O-4 last-run SSB state.` confirm=`Confirm event-fold keys, row types, and replay source of truth.` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L333): SSB rows likely provide the replay/last-run substrate used by O-4 chrome.
- **SUMMARY/O.INF03** [I] basis=`O-2 enumerates `VaultPage`; O-3 says O-1/O-2 `FileToBrain` sink.` confirm=`Resolve terminology/schema in O-2 and O-3 originals.` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L334): `VaultPage` and `FileToBrain` may be two names or two layers for the same terminal action.
- **SUMMARY/O.INF04** [I] basis=`A2 says zero engagement “decays the topic.”` confirm=`Confirm the mutated field, decay curve, reversibility, and whether scheduling is disabled.` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L335): Topic decay likely changes manifest scheduling or priority rather than deleting vault content.
- **SUMMARY/O.INF05** [I] basis=`Canary routes to `unsorted`; reaching a steal sink fails.` confirm=`Confirm whether `unsorted` is a class, queue, file path, or sink and what later review does.` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L336): `unsorted` appears to be a quarantine/non-steal destination for ambiguous or canary classifications.
- **SUMMARY/O.INF06** [I] basis=`Node edit updates the spec row; run-now goes via FSM.` confirm=`Confirm validation, optimistic concurrency, approvals, and mutation commands.` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L337): “No canvas runtime” likely means all graph edits are persisted as spec data and all execution authority stays engine-side.
- **SUMMARY/O.INF07** [I] basis=`Namespace claims prevent writes outside ownership.` confirm=`Locking/concurrency is not stated; confirm in original.` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L338): Two-harness vault sharing likely depends on non-overlapping claims plus atomic/locked writes.
- **SUMMARY/O.INF08** [I] basis=`Layout/contract and observable effect appear in the same unit.` confirm=`Confirm validation rules and invalid-page refusal behavior.` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L339): A “valid page” likely means conformance to the deterministic TEIKOKU markdown/frontmatter/wikilink contract.
- **SUMMARY/O.INF09** [I] basis=`Empty fires must not card.` confirm=`Confirm exact predicate and representation of empty/skipped/no-change outcomes.` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L340): “Result-bearing” likely means a fire with a material user-visible output, not merely a successful empty run.
- **SUMMARY/O.INF10** [I] basis=`A4 makes topic packs data; A3 makes new stage kinds code.` confirm=`Confirm who can edit packs and admission/versioning rules.` (ledger-line: 15-O-SUMMARY_INDEX.discovery.md:L341): The user-editable topic set is likely data-only, while adding a novel stage implementation is code-only.

---

## Superseded notes (newer-plan precedence)

Deep original plans win the target-state description over summary orientation where the deep pass records a material correction. Summary atoms remain in the inventory above; these notes record the losing reading.

- **SUP/SD-MC001** [superseded] winner=`O-001 originals (deep)` loser=`SUMMARY orientation` sd_ref=`O-001/SD-MC001` title=`Digest patches are per-unit, not literally identical across every unit.` (ledger-line: DEEP-O-001.discovery.md:L683): O0 says `A1..A5` feed per-unit `stub_ban`/`REDTEAM` (`O0:L4-L7,L55-L56`). The detail plans specialize them: O-1 applies A1/A3, O-2 A2/A3, O-3 A4, and O-4 A3. The summary-ledger claim that every unit enforces the same A1, A3, and A4 set is broader than the originals.
- **SUP/SD-MC002** [superseded] winner=`O-001 originals (deep)` loser=`SUMMARY orientation` sd_ref=`O-001/SD-MC002` title=`The engagement falsifier is not present in these five originals.` (ledger-line: DEEP-O-001.discovery.md:L684): The summary ledger’s digest-open, `For-You` click, `N`-day zero-engagement, and honest topic-decay requirements are not stated in O0 or O1–O4. O1 does state an existing artifact lifecycle that decays to `DECAYED` (`O1:L7`), but it does not connect that state to opens/clicks or an `N`-day threshold. Those engagement requirements remain summary-only in this source scope.
- **SUP/SD-MC003** [superseded] winner=`O-001 originals (deep)` loser=`SUMMARY orientation` sd_ref=`O-001/SD-MC003` title=`O-4 editing is not Tier 1.` (ledger-line: DEEP-O-001.discovery.md:L685): The original makes Tier 1 read-only inspection and explicitly defers TaskSpec write-back plus provenance drill-down to Tier 2 (`O4:L19-L21`). Any summary reading of node editing as an immediate ship-now feature needs this tier qualifier.
- **SUP/SD-MC004** [superseded] winner=`O-001 originals (deep)` loser=`SUMMARY orientation` sd_ref=`O-001/SD-MC004` title=`O-2 full vocabulary is larger than its Tier-1 runtime.` (ledger-line: DEEP-O-001.discovery.md:L686): The schema names eight stages and four sinks, but Tier 1 explicitly names only Fetch/Classify/Gate/Digest and DashboardInbox; SsbEvent, SpawnResearchTree, RunPlanUnit, and missed-run wiring are Tier 2 (`O2:L9-L10,L19-L21`). Schema availability must not be confused with production wiring.
- **SUP/SD-MC005** [superseded] winner=`O-001 originals (deep)` loser=`SUMMARY orientation` sd_ref=`O-001/SD-MC005` title=`O-1 full topic enum is larger than Tier 1.` (ledger-line: DEEP-O-001.discovery.md:L687): Four kinds are schema-valid, but only `steal-scan` and `monitor` ship now; `leaderboard`, `discovery`, and living-page upserts are deferred (`O1:L19-L21`).
- **SUP/SD-MC006** [superseded] winner=`O-001 originals (deep)` loser=`SUMMARY orientation` sd_ref=`O-001/SD-MC006` title=`Exactly-once is scoped to a due tick.` (ledger-line: DEEP-O-001.discovery.md:L688): The original’s concrete mechanism is a last-fired watermark and the acceptance phrase “exactly once per due tick” (`O2:L31-L33,L65-L68`); it does not claim global exactly-once under crash, multi-process, or distributed scheduling.
- **SUP/SD-MC007** [superseded] winner=`O-001 originals (deep)` loser=`SUMMARY orientation` sd_ref=`O-001/SD-MC007` title=`Privacy is a staged safety barrier.` (ledger-line: DEEP-O-001.discovery.md:L689): Personal-to-project/cloud flow is a hard exclusion/stub ban, but the actual D-5 read-path wiring is Tier 2 and must land before personal topics become mountable into project context (`O3:L10-L11,L18-L20,L60-L64,L80-L81`).
- **SUP/SD-MC008** [superseded] winner=`O-001 originals (deep)` loser=`SUMMARY orientation` sd_ref=`O-001/SD-MC008` title=`“Dark/absent” is not the exact status vocabulary of the originals.` (ledger-line: DEEP-O-001.discovery.md:L690): All four details say `PLANNED`; O0 defines the formal statuses. The originals identify new versus reused mechanisms but do not independently inventory current code to prove each component dark (`O0:L42-L45`; `O1:L3`; `O2:L3`; `O3:L3`; `O4:L3`).
- **SUP/SD-MC009** [superseded] winner=`O-001 originals (deep)` loser=`SUMMARY orientation` sd_ref=`O-001/SD-MC009` title=`The trust-spine boundary is more precise than a blanket byte claim.` (ledger-line: DEEP-O-001.discovery.md:L691): O1 says G-1 wraps fetched bytes, F-3 applies to the research role, and G-2 records external bytes; integration names fetch/classify/sink seams (`O1:L10,L23-L24,L31-L36`). Metadata, redirect, attachment, and cache coverage remain unspecified.
- **SUP/SD-MC010** [superseded] winner=`O-001 originals (deep)` loser=`SUMMARY orientation` sd_ref=`O-001/SD-MC010` title=`No new O-4 event is planned.` (ledger-line: DEEP-O-001.discovery.md:L692): O0 explicitly says O-4 reuses O-2’s fire row, while O-1/O-2/O-3 add schema action types (`O0:L80-L85`).

---

## User-facing surfaces

Collects every window, panel, command, CLI verb, flag, status display, and notification atom of the family. Reference atoms by ID. Feeds dashboard DESIGN.md.

### Curated surface checklist (family O, by product surface)

Reference atoms only; do not invent surfaces absent from the inventories.

- **Config: `[[research.topics]]` typed topic manifest** (user-editable; closed `kind`) — refs: O-001/O1-E015..E025, O-001/O0-E021, SUMMARY/O-1.E10..E16
- **Topic kinds (schema):** `steal-scan`, `leaderboard`, `discovery`, `monitor` — refs: O-001/O1-E018..E021; Tier 1 ships steal-scan+monitor: O-001/O1-E057..E058, O-001/O0-E065
- **UI: CosmosDrawer** (extend, not replace; show topic kind + last-run trust tier) — refs: O-001/O1-E064..E066, SUMMARY/O-1.E34
- **UI: intervention-card inbox / DashboardInbox sink** — refs: O-001/O2-E037, O-001/O2-E060, O-001/O2-E078..E080, SUMMARY/O-2.E25, SUMMARY/O-2.E41
- **UI: Toast sink** — refs: O-001/O2-E032, SUMMARY/O-2.E19
- **UI: canvas mode bar WORKFLOW mode** (H-12 extend) — refs: O-001/O4-E016, O-001/O4-E031, O-001/O4-E044, SUMMARY/O-4.E01..E02
- **UI: WORKFLOW Svelte view** (`crates/dashboard-v2`, Svelte 5 / Tauri v2) — refs: O-001/O4-E038..E043, SUMMARY/O-4.E24
- **UI: enum-constrained node palette** (O-2 stage vocabulary only) — refs: O-001/O4-E017, O-001/O4-E048..E049, SUMMARY/O-4.E21
- **UI: node graph projection** (node = stage instance; edge = dataflow + provenance tier) — refs: O-001/O4-E010..E011, O-001/O4-E045..E047, SUMMARY/O-4.E04..E11
- **UI: J-60 last-run chrome on nodes** — refs: O-001/O4-E012, O-001/O4-E019, O-001/O4-E050..E051, SUMMARY/O-4.E12..E13
- **UI: missing-consumer / wire-dark visual disconnect** — refs: O-001/O4-E020..E021, O-001/O4-E052..E054, SUMMARY/O-4.E16..E17
- **UI action: `run now` / `run-now`** (FSM dispatch only; no canvas runtime) — refs: O-001/O4-E055..E059, SUMMARY/O-4.E30..E32
- **UI: optional read-only dashboard vault-page view** (or defer to Obsidian/GDrive; no new panel justified) — refs: O-001/O3-E053..E055, O-001/O3-E009
- **External surface: Markdown vault** (Obsidian/GDrive; mount TEIKOKU vault) — refs: O-001/O3-E008..E011, SUMMARY/O-3.E04..E16
- **External surface: living pages** `topics/<t>/<slug>.md`, `rankings/`, `queue/`, `index` — refs: O-001/O3-E014..E019, SUMMARY/O-3.E06..E11
- **Config: root `namespace-claims` file** — refs: O-001/O3-E024..E026, SUMMARY/O-3.E17..E20
- **CLI: interim `elai brain write`** (when FileToBrain caller absent) — refs: O-001/O3-E096, O-001/O0-E079
- **Tauri command: `list_task_specs`** — refs: O-001/O4-E041, SUMMARY/O-4.E25
- **Tauri command: `get_task_spec_runstate`** — refs: O-001/O4-E042, SUMMARY/O-4.E26
- **Production tick: live_cli Cosmos tick / C-16** — refs: O-001/O1-E093, O-001/O1-E100
- **Production tick: C-9 task_runner interval** — refs: O-001/O2-E093, O-001/O2-E099
- **Notification/status: SSB classify row** `{class, confidence, taint_tier}` — refs: O-001/O1-E096, SUMMARY/O-1.E36..E39
- **Notification/status: `task_spec_fired`** `{spec_id, trigger_cause, stage_path}` — refs: O-001/O2-E071..E074, SUMMARY/O-2.E37..E40
- **Notification/status: `vault_write`** `{namespace, page, mode}` — refs: O-001/O3-E088..E091, SUMMARY/O-3.E32
- **Notification/status: `claim_violation`** — refs: O-001/O3-E092, SUMMARY/O-3.E33
- **Status: canary route `unsorted`** — refs: O-001/O1-E073, O-001/O1-E097, SUMMARY/O-1.E40
- **Status: artifact lifecycle `DECAYED`** — refs: O-001/O1-E007
- **Engagement signals (summary-only; not in originals):** digest-open, `For-You` click, N-day zero-engagement decay — refs: SUMMARY/O.X.E14..E18; superseded/narrowed by O-001/SD-MC002
- **Product positioning:** “n8n with receipts” / not generic ungated engine — refs: O-001/O4-E015, O-001/O0-E017, SUMMARY/O-4.E18, SUMMARY/O.G.E06
- **Track product name:** TEIKOKU-style personal research machine on the ELAI canvas / RESEARCH-MACHINE — refs: O-001/O0-E001, SUMMARY/O.G.E01..E02

### Surface-tagged atom index (auto)

Count of atoms tagged `[SURFACE]` during extraction: **209** unique IDs.

- O-001/IX-DATA001, O-001/IX-DATA006, O-001/IX-DATA007, O-001/IX-DATA008, O-001/IX-DATA010, O-001/IX-DATA011, O-001/IX-PATH03, O-001/IX-PATH04, O-001/IX-PATH06, O-001/IX-PATH07, O-001/IX-PATH08, O-001/IX-PATH09, O-001/IX-PATH10, O-001/IX-RECON007, O-001/IX-RECON012, O-001/IX-RECON016, O-001/IX-RECON017, O-001/O0-E008, O-001/O0-E009, O-001/O0-E011
- O-001/O0-E012, O-001/O0-E021, O-001/O0-E026, O-001/O0-E028, O-001/O0-E029, O-001/O0-E030, O-001/O0-E031, O-001/O0-E033, O-001/O0-E047, O-001/O0-E068, O-001/O0-E072, O-001/O0-E079, O-001/O0-E085, O-001/O0-E091, O-001/O0-E092, O-001/O1-E027, O-001/O1-E048, O-001/O1-E059, O-001/O1-E064, O-001/O1-E065
- O-001/O1-E066, O-001/O1-E081, O-001/O1-E093, O-001/O1-E100, O-001/O1-E110, O-001/O2-E005, O-001/O2-E012, O-001/O2-E029, O-001/O2-E031, O-001/O2-E032, O-001/O2-E037, O-001/O2-E042, O-001/O2-E047, O-001/O2-E051, O-001/O2-E060, O-001/O2-E061, O-001/O2-E078, O-001/O2-E079, O-001/O2-E081, O-001/O2-E086
- O-001/O2-E092, O-001/O2-E095, O-001/O2-E097, O-001/O2-E105, O-001/O2-E109, O-001/O2-E110, O-001/O2-E111, O-001/O3-E008, O-001/O3-E009, O-001/O3-E010, O-001/O3-E011, O-001/O3-E013, O-001/O3-E024, O-001/O3-E027, O-001/O3-E034, O-001/O3-E035, O-001/O3-E042, O-001/O3-E046, O-001/O3-E047, O-001/O3-E048
- O-001/O3-E050, O-001/O3-E052, O-001/O3-E053, O-001/O3-E054, O-001/O3-E055, O-001/O3-E057, O-001/O3-E071, O-001/O3-E073, O-001/O3-E077, O-001/O3-E085, O-001/O3-E087, O-001/O3-E088, O-001/O3-E089, O-001/O3-E090, O-001/O3-E091, O-001/O3-E096, O-001/O3-E100, O-001/O3-E103, O-001/O3-E105, O-001/O3-E108
- O-001/O4-E002, O-001/O4-E003, O-001/O4-E004, O-001/O4-E005, O-001/O4-E006, O-001/O4-E007, O-001/O4-E008, O-001/O4-E009, O-001/O4-E010, O-001/O4-E011, O-001/O4-E012, O-001/O4-E016, O-001/O4-E017, O-001/O4-E022, O-001/O4-E023, O-001/O4-E024, O-001/O4-E025, O-001/O4-E031, O-001/O4-E036, O-001/O4-E038
- O-001/O4-E041, O-001/O4-E042, O-001/O4-E043, O-001/O4-E044, O-001/O4-E047, O-001/O4-E048, O-001/O4-E049, O-001/O4-E055, O-001/O4-E056, O-001/O4-E057, O-001/O4-E058, O-001/O4-E059, O-001/O4-E062, O-001/O4-E063, O-001/O4-E065, O-001/O4-E066, O-001/O4-E067, O-001/O4-E068, O-001/O4-E069, O-001/O4-E071
- O-001/O4-E073, O-001/O4-E075, O-001/O4-E077, O-001/O4-E079, O-001/O4-E080, O-001/O4-E081, O-001/O4-E083, O-001/O4-E084, O-001/O4-E085, O-001/O4-E086, O-001/O4-E087, O-001/O4-E088, O-001/O4-E090, O-001/O4-E093, SUMMARY/O-1.E10, SUMMARY/O-1.E34, SUMMARY/O-2.E03, SUMMARY/O-2.E16, SUMMARY/O-2.E18, SUMMARY/O-2.E19
- SUMMARY/O-2.E25, SUMMARY/O-2.E31, SUMMARY/O-2.E41, SUMMARY/O-3.E04, SUMMARY/O-3.E05, SUMMARY/O-3.E07, SUMMARY/O-3.E08, SUMMARY/O-3.E09, SUMMARY/O-3.E10, SUMMARY/O-3.E11, SUMMARY/O-3.E12, SUMMARY/O-3.E13, SUMMARY/O-3.E14, SUMMARY/O-3.E16, SUMMARY/O-3.E17, SUMMARY/O-3.E20, SUMMARY/O-3.E24, SUMMARY/O-3.E27, SUMMARY/O-3.E28, SUMMARY/O-3.E32
- SUMMARY/O-4.E01, SUMMARY/O-4.E02, SUMMARY/O-4.E03, SUMMARY/O-4.E04, SUMMARY/O-4.E05, SUMMARY/O-4.E06, SUMMARY/O-4.E07, SUMMARY/O-4.E08, SUMMARY/O-4.E12, SUMMARY/O-4.E13, SUMMARY/O-4.E19, SUMMARY/O-4.E21, SUMMARY/O-4.E22, SUMMARY/O-4.E23, SUMMARY/O-4.E24, SUMMARY/O-4.E25, SUMMARY/O-4.E26, SUMMARY/O-4.E27, SUMMARY/O-4.E30, SUMMARY/O-4.E31
- SUMMARY/O-4.E32, SUMMARY/O.G.E02, SUMMARY/O.X.E13, SUMMARY/O.X.E15, SUMMARY/O.X.E16, SUMMARY/WF-02, SUMMARY/WF-03, SUMMARY/WF-04, SUMMARY/WF-05

---

## Closing audit block

- **Ledgers merged:** 1/1 deep (`DEEP-O-001`) + summary (`15-O-SUMMARY_INDEX.discovery.md`).
- **Per-ledger atom counts (all distinct record kinds):**
  - O-001: **673** = explicit E 520 (O0=95, O1=111, O2=112, O3=109, O4=93) + IX 85 (path 11 + vocab 7 + data 11 + dep 5 + fail 19 + recon 18 + cat 14) + SD 31 + AC 25 + INF 12
  - SUMMARY: **255** = inventory E 185 (G=14, O-1=42, O-2=43, O-3=35, O-4=33, X=18) + WF 5 + INV 10 + AC 33 + INF 10 + SILENT 12
- **Aggregate distinct record count:** **928**
- **Superseded notes count:** **10**
- **Surface-tagged atoms (unique):** **209**
- **Zero-omission statement:** Every explicit atomic bullet in DEEP-O-001 (520 E atoms, unit footers 95+111+112+109+93) and every summary inventory table row (185 E atoms) is present above under a namespaced ID with its source anchor. Cross-file composition tables, material corrections, original expansions, resolved/unresolved questions, staging tensions, naming ambiguities, concurrency gaps, status caveat, deep+summary inferences, summary workflows, invariants, and summary-silent candidates are preserved as distinct records. Composition/IX tables reorganize explicit findings for navigation and are counted as IX records, not as replacement for E atoms. No atomic record was compressed into prose or dropped as minor.
- **Duplicate-ID check (native E IDs within deep):** unique=520/520.
- **Duplicate-ID check (native E IDs within summary):** unique=185/185.
- **Cross-check vs deep ledger footer:** DEEP-O-001 declares **520** explicit atomic findings — match.
- **Cross-check vs summary quantitative coverage:** summary declares **185** explicit atomic statements — match.
- **Output path:** `docs/thefinalglance/feature-discovery/aggregates/O-AGGREGATE.md`

---

*Aggregator: O-family deep feature discovery. Inputs read-only. Single write: this file.*
