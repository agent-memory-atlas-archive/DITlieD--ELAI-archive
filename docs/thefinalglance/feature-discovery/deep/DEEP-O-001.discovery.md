# O-plan exhaustive feature discovery — all original plans (`DEEP-O-001`)

## Scope, evidence rule, and inventory

This is the deep, original-source pass for the O family only. Every non-summary Markdown file found under `docs/thefinalglance/O-plan` was read from first through last line exactly once as a primary source. `15-O-SUMMARY_INDEX.discovery.md` was read only as an orientation and gap queue; no finding in the explicit ledger below is sourced from it. Assertions that an implementation is “shipped,” “live,” or “proven” remain **PLAN-asserted** unless the original itself says otherwise; this pass did not inspect code or other plan families.

Inventory command run before reading:

```text
rg --files --hidden --no-ignore docs/thefinalglance/O-plan | sort
```

It returned six Markdown files: the five originals below plus `O-SUMMARY_INDEX.md`. The summary was excluded by the task; all five remaining files were included.

| Source key | Exact primary-source path | Lines | SHA-256 |
|---|---|---:|---|
| `O0` | `docs/thefinalglance/O-plan/O-0_INDEX.md` | 125 | `a7988ad7c1f9c3dac2bd89d12996f3256d4aef0e1b5c4c11deb0e1f1fa41c217` |
| `O1` | `docs/thefinalglance/O-plan/O-1_research_daemon_generalized_cosmos.md` | 90 | `d1b23db489099d48d375c58b353da55eb9e40ffc2d7b9ad0b9c3a256bf1ece38` |
| `O2` | `docs/thefinalglance/O-plan/O-2_scheduled_taskspec_substrate.md` | 91 | `db5ba4492f42d55246c7a7c969c2337f86003fb55e2ef9e4bb11da8f2b915584` |
| `O3` | `docs/thefinalglance/O-plan/O-3_brain_vault_contract_mounting.md` | 91 | `a30462d74e297fa7935ec8c62484b27546ef6ce41c5d210e2f634902632754c0` |
| `O4` | `docs/thefinalglance/O-plan/O-4_canvas_workflow_mode.md` | 94 | `5deae2ce45ece1bba69c4bdaeead2222a5f727785603b599d7c1a10ac9c71899` |
| **Total** | **5 originals** | **491** | — |

### Heading and source-anchor map

- `O0`: title `L1`; provenance/predecessors `L2-L24`; **Why O-plan exists** `L26-L40`; **Status legend** `L42-L45`; wiring doctrine `L47-L60`; done definition `L62-L64`; four sessions `L66-L77`; constants/schema bumps `L79-L85`; sequencing `L87-L96`; inherited hard rules `L98-L103`; carried corrections `L105-L112`; session log `L114-L117`; next step `L119-L123`; terminator `L125`.
- `O1`: title/status/metadata/premise `L1-L7`; **Steal/build** `L9-L11`; **Existing ground truth** `L13-L17`; **REVISED scope** `L19-L21`; **Integration** `L23-L25`; **MOVES** `L27-L36`; **FORK** `L38-L39`; **RECON_NEEDED** `L41-L43`; **ABORT** `L45-L46`; **WIRING_CONTRACT** `L48-L69`; **VERIFY** `L71-L77`; **REDTEAM** `L79-L80`; **Grounding** `L82-L85`; **COMPOSES-WITH** `L87-L88`; terminator `L90`.
- `O2`: title/status/metadata/premise `L1-L7`; **Steal/build** `L9-L11`; **Existing ground truth** `L13-L17`; **REVISED scope** `L19-L21`; **Integration** `L23-L25`; **MOVES** `L27-L36`; **FORK** `L38-L39`; **RECON_NEEDED** `L41-L44`; **ABORT** `L46-L47`; **WIRING_CONTRACT** `L49-L70`; **VERIFY** `L72-L78`; **REDTEAM** `L80-L81`; **Grounding** `L83-L86`; **COMPOSES-WITH** `L88-L89`; terminator `L91`.
- `O3`: title/status/metadata/premise `L1-L7`; **Steal/build** `L9-L11`; **Existing ground truth** `L13-L16`; **REVISED scope** `L18-L20`; **Integration** `L22-L24`; **MOVES** `L26-L35`; **FORK** `L37-L38`; **RECON_NEEDED** `L40-L43`; **ABORT** `L45-L46`; **WIRING_CONTRACT** `L48-L70`; **VERIFY** `L72-L78`; **REDTEAM** `L80-L81`; **Grounding** `L83-L86`; **COMPOSES-WITH** `L88-L89`; terminator `L91`.
- `O4`: title/status/metadata/premise `L1-L7`; **Steal/build** `L9-L11`; **Existing ground truth** `L13-L17`; **REVISED scope** `L19-L21`; **Integration** `L23-L25`; **MOVES** `L27-L36`; **FORK** `L38-L39`; **RECON_NEEDED** `L41-L44`; **ABORT** `L46-L47`; **WIRING_CONTRACT** `L49-L73`; **VERIFY** `L75-L82`; **REDTEAM** `L84-L85`; **Grounding** `L87-L89`; **COMPOSES-WITH** `L91-L92`; terminator `L94`.

Source citations such as `O2:L31-L33` refer to those immutable, checksum-recorded inputs. Each `*-E###` entry is one counted explicit atomic finding. Inference, ambiguity, conflict, and summary comparison are isolated after the explicit ledger and are not included in the atomic count.

## Explicit atomic ledger

### O0 — O-Plan Index / track-wide contract

- **O0-E001** — The index names O as “the PERSONAL RESEARCH MACHINE side,” says it is wiring-enforced, comprises `4 sessions`, and has authored Track `RESEARCH-MACHINE`. (`O0:L1`)
- **O0-E002** — The index is dated `2026-07-06`. (`O0:L2`)
- **O0-E003** — Track-level `PROOF_SPINE` is `none`, owner-acknowledged. (`O0:L2`)
- **O0-E004** — The source digest’s layers `L1..L4` map `1:1` to `O-1..O-4`. (`O0:L4-L7`)
- **O0-E005** — Digest attacks `A1..A5` and its verbatim-steal list feed each unit’s `REDTEAM` and `stub_ban`. (`O0:L6-L7`)
- **O0-E006** — O-1/O-3 are instructed to steal the live-proven enum-gated classification, injection-canary, confidence-per-sink, `write_page`, `upsert_page`, and namespace patterns from the TEIKOKU handoff. (`O0:L8-L9`)
- **O0-E007** — O’s predecessor range is `B-plan .. N-plan`; O must compose with it and restate none of it under `GD-1`. (`O0:L11`)
- **O0-E008** — O is a product-surface letter that ships a new user-facing feature, unlike N’s invent-nothing interlacing role. (`O0:L11-L13`)
- **O0-E009** — The new surface is the combination of a research daemon, typed scheduler, mounted brain vault, and canvas workflow mode. (`O0:L12-L13`)
- **O0-E010** — The surface is estimated as roughly `80%` composition of shipped subsystems. (`O0:L13-L16`)
- **O0-E011** — Named composed subsystems are `B-42` Cosmos, `C-9` task_runner, `3F` rules-engine, `E-5` memory, `D-5` privacy, `G-1` taint, `F-3` tool-filter, `G-2` VCR, and `H-12/J-31/J-60` canvas. (`O0:L13-L15`)
- **O0-E012** — Genuinely new mechanisms are a closed `TaskSpec` vocabulary, vault namespace claims, canvas `WORKFLOW` mode, and Cosmos manifest generalization. (`O0:L15-L16`)
- **O0-E013** — Mandatory predecessor reading order is source design, `ELAI-FEATURE-MAP.md`, `F-0_INDEX.md`, `G-0_INDEX.md`, this index, `O-SUMMARY_INDEX.md`, then the target detail file. (`O0:L18-L24`)
- **O0-E014** — The feature map is designated the single context load and executors are told not to re-read the legacy 12-file list. (`O0:L20-L21`)
- **O0-E015** — The central finding is that the personal research machine is architecturally cheap because it mostly composes already-shipped subsystems. (`O0:L26-L30`)
- **O0-E016** — ELAI’s product differentiator for this surface is journaled, gated, replayable runs. (`O0:L29-L30`)
- **O0-E017** — Building a generic dataflow engine, described as “n8n-minus-integrations,” is explicitly the wrong architecture. (`O0:L30-L31`)
- **O0-E018** — The required architecture is a closed-vocabulary typed task layer. (`O0:L31-L32`)
- **O0-E019** — Every stage kind must exist as code. (`O0:L31-L32`)
- **O0-E020** — Every task fire must be an SSB row. (`O0:L31-L32`)
- **O0-E021** — O-1 swaps Cosmos’s topic source for a user-editable typed topic set. (`O0:L33-L34`)
- **O0-E022** — O-1 adds an enum-classify surface. (`O0:L33-L34`)
- **O0-E023** — O-1 must thread `G-1/F-3/G-2` trust handling through every fetched byte. (`O0:L33-L34`)
- **O0-E024** — O-2 adds a typed `TaskSpec` scheduler stored as rules-as-data in SQLite. (`O0:L35-L36`)
- **O0-E025** — O-2 supports `cron`, `ssb`, and `manual` trigger categories at full scope. (`O0:L35-L36`)
- **O0-E026** — O-2 journals fires and supplies a dashboard intervention-card inbox. (`O0:L35-L36`)
- **O0-E027** — O-2 reuses the task_runner interval and SSB temporal monitor. (`O0:L35-L36`)
- **O0-E028** — O-3 mounts, rather than converts, a portable Markdown brain vault. (`O0:L37-L38`)
- **O0-E029** — O-3 adds a namespace-claims contract so ELAI and TEIKOKU can share one vault. (`O0:L37-L38`)
- **O0-E030** — `memory_facts` remains the machine-grain store while the vault becomes the user-grain surface. (`O0:L37-L38`)
- **O0-E031** — O-4 adds canvas `WORKFLOW` as an editor and monitor over TaskSpecs. (`O0:L39-L40`)
- **O0-E032** — O-4 makes wire-dark conditions visible. (`O0:L39-L40`)
- **O0-E033** — O-4 dispatches through the `J-31` FSM and must never introduce a canvas runtime. (`O0:L39-L40`)
- **O0-E034** — Allowed status values are `PLANNED`, `IN PROGRESS`, `DONE`, and `DEFERRED`. (`O0:L42-L43`)
- **O0-E035** — `DONE` requires `wiring_contract_check.py --run-trigger` to pass. (`O0:L43-L44`)
- **O0-E036** — `DONE` requires `production_caller_check.py` and `python_wire_check.py` to be clean. (`O0:L43-L44`)
- **O0-E037** — `DONE` requires the `R10` integration smoke to be green. (`O0:L44`)
- **O0-E038** — Cargo-green is necessary but not sufficient for `DONE`. (`O0:L44-L45`)
- **O0-E039** — O inherits `W-1..W-10` verbatim from `F-0_INDEX.md`. (`O0:L50`)
- **O0-E040** — Inherited wiring doctrine requires a real-entrypoint trigger and an off-agent observable artifact. (`O0:L50-L51`)
- **O0-E041** — Missing evidence is a failure. (`O0:L51`)
- **O0-E042** — Reachability checks must use sound references rather than leaf grep. (`O0:L51`)
- **O0-E043** — A producer without a consumer is a build break. (`O0:L51`)
- **O0-E044** — Test callers never satisfy production reachability. (`O0:L51-L52`)
- **O0-E045** — The track inherits a frozen-gate requirement. (`O0:L52`)
- **O0-E046** — A default-off graveyard is forbidden. (`O0:L52`)
- **O0-E047** — Execution requires re-reading the live baseline and adding a dashboard done-item. (`O0:L52`)
- **O0-E048** — O inherits `GD-1..GD-5` from `G-0_INDEX.md`. (`O0:L53`)
- **O0-E049** — `GD-1` requires compose-never-restate: each O unit names consumed mechanisms and must not reimplement them. (`O0:L53-L54`)
- **O0-E050** — `GD-4` requires observe-only-first for any gate defaulting on. (`O0:L54`)
- **O0-E051** — `GD-5` requires honest residue to be named in-file. (`O0:L54`)
- **O0-E052** — O adds no new doctrine-delta family. (`O0:L55-L56`)
- **O0-E053** — `A1..A5` are enforced through per-unit `stub_ban` and `REDTEAM` blocks. (`O0:L55-L56`)
- **O0-E054** — All `O-1..O-4` have `PROOF_SPINE: none` because they are product-surface/distribution work. (`O0:L58`)
- **O0-E055** — The proof-spine absence was owner-acknowledged on `2026-07-06` under `PSR-2`. (`O0:L58`)
- **O0-E056** — `PS-3` is only a weak nearest candidate if briefings are counted as grounding inputs; the plan does not claim it. (`O0:L59-L60`)
- **O0-E057** — O feeds no `PS-1..PS-6` number directly. (`O0:L60`)
- **O0-E058** — The FSM may not reach `COMPLETE` without `cargo check` green. (`O0:L62-L63`)
- **O0-E059** — Completion requires the auditor to pass on affected crates only. (`O0:L62-L63`)
- **O0-E060** — Completion requires wiring-contract trigger evidence. (`O0:L63`)
- **O0-E061** — Completion requires a clean wire checker or a `WIRE-DARK[O-N]` row. (`O0:L63-L64`)
- **O0-E062** — O-4 completion requires `cross_lang_diff` for its frontend pair. (`O0:L64`)
- **O0-E063** — Completion requires the `R10` integration smoke and a memory-writer run. (`O0:L64`)
- **O0-E064** — O-1 is `[KEYSTONE]`, effort `M-L`, Wave `1`, with lever `manifest-generalization`. (`O0:L69-L71`)
- **O0-E065** — O-1 ships `steal-scan` and `monitor` kinds in its listed session scope. (`O0:L70-L71`)
- **O0-E066** — O-2 is effort `M-L`, Wave `1`, with lever `rules-as-data-scheduling`. (`O0:L72-L73`)
- **O0-E067** — O-3 is effort `M`, Wave `2`, with lever `mount-not-convert`. (`O0:L74-L75`)
- **O0-E068** — O-4 is effort `M`, Wave `3`, with lever `canvas-as-spec-editor`. (`O0:L76-L77`)
- **O0-E069** — Any O unit adding an SSB `action_type` must bump `SCHEMA_VERSION` at the live baseline. (`O0:L80-L85`)
- **O0-E070** — O-1 adds a classify-row action type. (`O0:L82`)
- **O0-E071** — O-2 adds `task_spec_fired`. (`O0:L82`)
- **O0-E072** — O-3 adds `vault_write` and `claim_violation`. (`O0:L82-L83`)
- **O0-E073** — O-4 reuses O-2’s fire row and therefore does not introduce a distinct O-4 event in the index. (`O0:L83`)
- **O0-E074** — Executors must never hard-code the schema baseline; they must re-read `ssb.rs` and `action_type_lock.rs` at edit time under `W-9`. (`O0:L83-L85`)
- **O0-E075** — The decision document must quote `baseline+N from LIVE X/Y as of <edit date> (file:line)`. (`O0:L84-L85`)
- **O0-E076** — Wave 1 runs O-1 and O-2 in parallel because their knowledge versus orchestrator/rules crate touch sets are declared disjoint. (`O0:L88-L92`)
- **O0-E077** — O-1’s trust-spine threading is the ELAI-over-TEIKOKU delta and is the keystone into which other work sinks. (`O0:L90-L92`)
- **O0-E078** — O-3 can land after Wave 1 supplies a `FileToBrain` caller. (`O0:L93-L94`)
- **O0-E079** — O-3 may instead ship with interim CLI caller `elai brain write`. (`O0:L93-L94`)
- **O0-E080** — O-4 depends on O-2’s `TaskSpec` store being queryable. (`O0:L95-L96`)
- **O0-E081** — If that store is not queryable, O-4 may ship against a read-only fixture projection under its fork. (`O0:L95-L96`)
- **O0-E082** — Hashing must use `blake3`, not `blake2` or `sha2`. (`O0:L99-L102`)
- **O0-E083** — Production paths may not use `.unwrap()` or `.expect()`. (`O0:L101`)
- **O0-E084** — Tests are scoped to affected crates and are run by the auditor. (`O0:L101-L102`)
- **O0-E085** — New dependencies require anti-slopsquatting review; none are expected. (`O0:L102`)
- **O0-E086** — Closed enums must deny unknown values. (`O0:L102`)
- **O0-E087** — New gates use a kill switch, not an enable flag, under `W-2`. (`O0:L102-L103`)
- **O0-E088** — Every arXiv/external claim must be WebSearch-verified at authoring; O cites no arXiv. (`O0:L103`)
- **O0-E089** — Cosmos generalization is a swap of the topic source, not a rewrite of `CosmosStage`. (`O0:L106-L109`)
- **O0-E090** — The wall-clock trigger remains supervisor-plane and outside the replay set under `K-14`. (`O0:L110`)
- **O0-E091** — The vault is mounted and never converted into `memory_facts`. (`O0:L111`)
- **O0-E092** — The canvas has no runtime; the `J-31` FSM performs dispatch. (`O0:L112`)
- **O0-E093** — The session log is empty at plan generation. (`O0:L114-L117`)
- **O0-E094** — The declared next step is O-1 recon of the Cosmos manifest type and G-1 taint-wrap API, then manifest generalization. (`O0:L119-L123`)
- **O0-E095** — O-1 and O-2 should be dispatched in parallel because they target different crates. (`O0:L122-L123`)

### O1 — O-1: research daemon — generalize Cosmos from designer-briefs to a user-editable typed topic set

- **O1-E001** — `O-1` is `PLANNED`, Wave `1`, product-surface class, and `PROOF_SPINE: none`, owner-acknowledged `2026-07-06`. (`O1:L1-L3`)
- **O1-E002** — Metadata is Track `RESEARCH-MACHINE`, lever `manifest-generalization`, source finding `L1`, grounding `MODERATE (index-derived, PLAN-asserted)`, nature `BUILD-on-shipped`, effort `M-L`, Wave `1`. (`O1:L5`)
- **O1-E003** — `CosmosWorker` is asserted to already be a scheduled research loop. (`O1:L7`)
- **O1-E004** — Its existing loop has an artifact manifest. (`O1:L7`)
- **O1-E005** — Its existing loop has an hourly tick. (`O1:L7`)
- **O1-E006** — Its existing loop refreshes through `FlashResearch`. (`O1:L7`)
- **O1-E007** — Its existing lifecycle time-decays artifacts to `DECAYED`. (`O1:L7`)
- **O1-E008** — Its existing briefing is capped at `<=1500` tokens. (`O1:L7`)
- **O1-E009** — `CosmosStage` already participates in the `QueryPipeline`. (`O1:L7`)
- **O1-E010** — The plan estimates this existing system as `90%` of a general research daemon. (`O1:L7`)
- **O1-E011** — The current limitation is a manifest hard-bound to designer briefs. (`O1:L7`)
- **O1-E012** — O-1 generalizes only the manifest surface. (`O1:L7`)
- **O1-E013** — O-1 must thread every fetched byte through the existing taint, tool-filter, and VCR trust spine. (`O1:L7`)
- **O1-E014** — O-1 creates neither a new worker loop nor a new scheduler; O-2 owns scheduling. (`O1:L7`)
- **O1-E015** — The generalized configuration root is `[[research.topics]]`. (`O1:L9-L10`)
- **O1-E016** — Each research-topic record has an `id` field. (`O1:L10`)
- **O1-E017** — Each research-topic record has a `kind` field. (`O1:L10`)
- **O1-E018** — Allowed topic kind `steal-scan` is part of a closed enum. (`O1:L10`)
- **O1-E019** — Allowed topic kind `leaderboard` is part of the closed enum. (`O1:L10`)
- **O1-E020** — Allowed topic kind `discovery` is part of the closed enum. (`O1:L10`)
- **O1-E021** — Allowed topic kind `monitor` is part of the closed enum. (`O1:L10`)
- **O1-E022** — Each research-topic record has `sources[]`. (`O1:L10`)
- **O1-E023** — Each research-topic record has `cadence`. (`O1:L10`)
- **O1-E024** — Each research-topic record has `confidence_threshold`. (`O1:L10`)
- **O1-E025** — Each research-topic record has `sink`. (`O1:L10`)
- **O1-E026** — A free-form `custom topic` kind is forbidden; novel kinds ship as code. (`O1:L10-L11`)
- **O1-E027** — The per-topic workflow begins with `fetch`. (`O1:L10`)
- **O1-E028** — Fetch output is classified through an enum-gated classifier. (`O1:L10`)
- **O1-E029** — Classification prepends an untrusted-content preamble. (`O1:L10`)
- **O1-E030** — Classification fences each item. (`O1:L10`)
- **O1-E031** — Classification uses a per-kind schema. (`O1:L10`)
- **O1-E032** — Injection canaries are frozen fixtures. (`O1:L10`)
- **O1-E033** — Classification is followed by a duplicate precheck against a rejected ledger. (`O1:L10`)
- **O1-E034** — The pipeline then files accepted material. (`O1:L10`)
- **O1-E035** — The pipeline builds a layered digest. (`O1:L10`)
- **O1-E036** — The full-scope pipeline maintains living leaderboard pages using guarded upsert. (`O1:L10`)
- **O1-E037** — G-1 taint provenance labels fetched bytes as untrusted content. (`O1:L10`)
- **O1-E038** — Existing G-1 policy gate-blocks untrusted-content flows to destructive sinks. (`O1:L10`)
- **O1-E039** — F-3’s tool filter is applied to the research role. (`O1:L10`)
- **O1-E040** — External bytes are recorded in a G-2 VCR cassette. (`O1:L10`)
- **O1-E041** — VCR capture makes a research run `RRA`-replayable. (`O1:L10`)
- **O1-E042** — The claimed ELAI-over-TEIKOKU delta is canaries plus taint plus replay. (`O1:L10`)
- **O1-E043** — Re-authoring `CosmosStage` is excluded; the unit must extend it. (`O1:L11`)
- **O1-E044** — Re-authoring the briefing compiler is excluded. (`O1:L11`)
- **O1-E045** — A classifier output reaching a prompt may not be free text. (`O1:L11`)
- **O1-E046** — Classifier output is a closed enum plus confidence. (`O1:L11`)
- **O1-E047** — Existing asserted implementation includes roughly `700 LOC` of `cosmos_worker`. (`O1:L13-L14`)
- **O1-E048** — Existing asserted symbols/surfaces include `VerificationMethod::CosmosCheck`, `CosmosStage`, `CosmosDrawer`, and the `live_cli` tick. (`O1:L14`)
- **O1-E049** — B-42/C-16 classify the existing Cosmos artifact tier as `[WIRED]`. (`O1:L14`)
- **O1-E050** — Existing hourly refresh runs from task_runner’s `tokio::interval` under `C-9`. (`O1:L15`)
- **O1-E051** — `FlashResearch` refresh is the B-40 path. (`O1:L15`)
- **O1-E052** — G-1 taint, F-3 tool filtering, and G-2 VCR/RRA are treated as shipped surfaces to consume rather than rebuild. (`O1:L16`)
- **O1-E053** — TEIKOKU’s classifier/canary/confidence-per-sink pattern is asserted live-proven with `3 injections neutralized` and `0 out-of-enum`. (`O1:L17`)
- **O1-E054** — Tier 1 ships the typed topic manifest. (`O1:L19-L20`)
- **O1-E055** — Tier 1 ships enum-gated classification and its confidence gate. (`O1:L20`)
- **O1-E056** — Tier 1 ships duplicate precheck and taint/tool-filter/VCR threading. (`O1:L20`)
- **O1-E057** — Tier 1 supports exactly `2 of the 4` topic kinds: `steal-scan` and `monitor`. (`O1:L20`)
- **O1-E058** — Tier 2 defers `leaderboard` and `discovery`. (`O1:L21`)
- **O1-E059** — Tier 2 defers living-leaderboard upsert pages until O-3’s vault supplies the page surface. (`O1:L21`)
- **O1-E060** — The intended backend owner is `crates/knowledge` or the live location of `cosmos_worker`. (`O1:L23-L24`)
- **O1-E061** — Backend integration generalizes the existing manifest loader. (`O1:L24`)
- **O1-E062** — Backend integration adds the typed-topic schema and enum classify stage. (`O1:L24`)
- **O1-E063** — G-1/F-3/G-2 are consumed specifically at fetch, classify, and sink seams. (`O1:L24`)
- **O1-E064** — `CosmosDrawer` is extended to show topic kind. (`O1:L25`)
- **O1-E065** — `CosmosDrawer` is extended to show last-run trust tier. (`O1:L25`)
- **O1-E066** — A new drawer must not be built. (`O1:L25`)
- **O1-E067** — Move M1 replaces the designer-brief manifest struct with the closed-kind typed-topic schema while preserving the loader signature so `CosmosStage` remains unchanged. (`O1:L27-L28`)
- **O1-E068** — M1 acceptance requires `[[research.topics]] kind = "steal-scan"` to load and be enumerated by the daemon. (`O1:L29`)
- **O1-E069** — An unknown topic `kind` must hard-refuse parsing rather than default. (`O1:L29`)
- **O1-E070** — If an unknown kind defaults to a catch-all, the fix is deny-unknown enum deserialization plus a negative parse test. (`O1:L30`)
- **O1-E071** — Move M2 requires fetched bytes to be taint-wrapped before reaching the classify prompt. (`O1:L31`)
- **O1-E072** — M2’s classifier emits exactly the shaped object `{class: enum, confidence: f32}`, never free text. (`O1:L31`)
- **O1-E073** — A canary-injection fixture must route to `unsorted` at low confidence. (`O1:L32`)
- **O1-E074** — That canary path must emit a taint-labelled SSB row. (`O1:L32`)
- **O1-E075** — Acceptance requires zero out-of-enum classifier outputs. (`O1:L32`)
- **O1-E076** — If a canary reaches the prompt unlabelled, classification must be re-anchored to consume only the taint-wrapped item. (`O1:L33`)
- **O1-E077** — The failed-canary remedy freezes the canary in a gate-manifest fixture under `R8`. (`O1:L33`)
- **O1-E078** — Move M3 records external bytes in the G-2 VCR cassette keyed by run ID. (`O1:L34`)
- **O1-E079** — Replay acceptance is byte-identical output from the cassette with the network detached. (`O1:L35`)
- **O1-E080** — Replay divergence means an unrecorded network read leaked into the run. (`O1:L36`)
- **O1-E081** — Replay-divergence remediation routes every remaining fetch through the VCR client and adds a detached-network replay assertion. (`O1:L36`)
- **O1-E082** — If the live manifest is hard-inlined rather than swappable, in-place generalization stops. (`O1:L38-L39`)
- **O1-E083** — That fork creates a new topic loader and adapter into the existing `CosmosStage`, records the seam in the decision document, and does not fork the worker loop. (`O1:L39`)
- **O1-E084** — Before editing, recon must establish the exact manifest type and whether classification already exists in ELAI or only TEIKOKU. (`O1:L41-L42`)
- **O1-E085** — That recon uses the stated Cosmos grep plus `codegraph_explore` on `cosmos_worker`. (`O1:L42`)
- **O1-E086** — Before editing, recon must find whether G-1 exposes an untrusted-byte wrapping API and read its one call site. (`O1:L43`)
- **O1-E087** — If generalization requires editing the frozen G-2 cassette format or G-1 taint gate, execution aborts and flags the issue. (`O1:L45-L46`)
- **O1-E088** — The daemon consumes trust gates and may never modify them. (`O1:L46`)
- **O1-E089** — Produced symbols include the closed-kind research-topic manifest schema and loader. (`O1:L48-L50`)
- **O1-E090** — Produced symbols include enum-gated classification with untrusted preamble, item fence, per-kind schema, and `{class,confidence}` output. (`O1:L50-L52`)
- **O1-E091** — Produced symbols include duplicate precheck against the rejected ledger. (`O1:L52`)
- **O1-E092** — Consumed symbols are `cosmos_worker`, `CosmosStage`, the briefing compiler, G-1 taint, F-3 tool filter, and G-2 VCR. (`O1:L53-L55`)
- **O1-E093** — Production reachability is the live `live_cli` Cosmos tick driving a configured topic end to end. (`O1:L56-L58`)
- **O1-E094** — No fetch path may bypass G-1 taint or G-2 cassette recording. (`O1:L59-L63`)
- **O1-E095** — The observable trigger is a live daemon tick for a `steal-scan` topic pointed at a canary-injection fixture. (`O1:L64-L66`)
- **O1-E096** — The observable classify SSB row carries `{class, confidence, taint_tier}`. (`O1:L66`)
- **O1-E097** — The canary must land in `unsorted` and never a steal sink. (`O1:L66`)
- **O1-E098** — The resulting briefing must remain `<=1500` tokens and include provenance tiers. (`O1:L67`)
- **O1-E099** — Out-of-enum output or a canary reaching a steal sink fails the run. (`O1:L67`)
- **O1-E100** — The production hook is the C-16 Cosmos tick in `live_cli`; the exact function is named at recon and wired in the same changeset. (`O1:L69`)
- **O1-E101** — Verification runs `wiring_contract_check.py` with this plan and `--run-trigger`. (`O1:L71-L74`)
- **O1-E102** — Verification runs `production_caller_check.py --files <changed .rs> --json`; a dark new public function blocks completion. (`O1:L74`)
- **O1-E103** — Verification passes only when the taint-tier classify row lands off-agent, the canary routes to `unsorted`, and the daemon tick—not a test—is the production caller. (`O1:L75`)
- **O1-E104** — The integration smoke is run by an independent agent through the real Cosmos tick. (`O1:L77`)
- **O1-E105** — Its hard input is a live, multi-item, long-enough source containing a paraphrased prompt injection so confidence-gate and duplicate-precheck paths fire. (`O1:L77`)
- **O1-E106** — A toy single-item happy path does not satisfy `R10`. (`O1:L77`)
- **O1-E107** — The smuggled-instruction red-team attack is intended to fail because classification is closed, confidence-bearing, taint-labelled to every sink, and protected by a frozen canary fixture. (`O1:L79-L80`)
- **O1-E108** — The successful generic-engine attack—adding a free-text topic kind to skip code—is patched by deny-unknown kind parsing. (`O1:L80`)
- **O1-E109** — O-1 composes with `CosmosStage`, `cosmos_worker`, briefing compiler, G-1, F-3, G-2/RRA, and B-40 `FlashResearch`, and reimplements none. (`O1:L82-L88`)
- **O1-E110** — O-2 supplies scheduling, O-3 supplies the vault sink, and O-4 supplies the canvas surface around O-1. (`O1:L87-L88`)
- **O1-E111** — A `#[cfg(test)]` caller never satisfies O-1 production reachability. (`O1:L56-L58`)

### O2 — O-2: scheduled TaskSpec substrate — typed, journaled, rules-as-data schedules over the closed stage vocabulary

- **O2-E001** — `O-2` is `PLANNED`, Wave `1`, product-surface class, `PROOF_SPINE: none`, and owner-acknowledged `2026-07-06`. (`O2:L1-L3`)
- **O2-E002** — Metadata is Track `RESEARCH-MACHINE`, lever `rules-as-data-scheduling`, source finding `L3`, grounding `MODERATE (index-derived, PLAN-asserted)`, nature `BUILD-on-shipped`, effort `M-L`, Wave `1`. (`O2:L5`)
- **O2-E003** — The borrowed Codex Automations shape supports time schedules. (`O2:L7`)
- **O2-E004** — The borrowed shape supports event schedules. (`O2:L7`)
- **O2-E005** — Scheduled results land in an inbox. (`O2:L7`)
- **O2-E006** — Runs with nothing to report auto-archive. (`O2:L7`)
- **O2-E007** — ELAI’s version is explicitly typed. (`O2:L7`)
- **O2-E008** — ELAI’s version is explicitly journaled. (`O2:L7`)
- **O2-E009** — A `TaskSpec` is stored as data in SQLite using the rules-engine precedent. (`O2:L7`)
- **O2-E010** — Every fire is an SSB row carrying its trigger cause. (`O2:L7`)
- **O2-E011** — Triggers are driven by the existing task_runner plus the SSB temporal monitor. (`O2:L7`)
- **O2-E012** — O-2 supplies the scheduler used by O-1’s daemon and edited by O-4’s canvas. (`O2:L7`)
- **O2-E013** — O-2 invents only the typed spec and trigger wiring. (`O2:L7`)
- **O2-E014** — `TaskSpec.trigger` is a closed enum. (`O2:L9-L10`)
- **O2-E015** — `Cron{expr}` is a trigger variant with an `expr` operand. (`O2:L10`)
- **O2-E016** — `SsbEvent{action_type filter}` is a trigger variant with an action-type filter. (`O2:L10`)
- **O2-E017** — `Manual` is a trigger variant. (`O2:L10`)
- **O2-E018** — `TaskSpec.pipeline` is an ordered list. (`O2:L10`)
- **O2-E019** — Pipeline stage kinds are a closed enum. (`O2:L10`)
- **O2-E020** — `Fetch` is a stage kind. (`O2:L10`)
- **O2-E021** — `Classify` is a stage kind. (`O2:L10`)
- **O2-E022** — `Gate` is a stage kind. (`O2:L10`)
- **O2-E023** — `FileToBrain` is a stage kind. (`O2:L10`)
- **O2-E024** — `Digest` is a stage kind. (`O2:L10`)
- **O2-E025** — `SpawnResearchTree` is a stage kind. (`O2:L10`)
- **O2-E026** — `RunPlanUnit` is a stage kind. (`O2:L10`)
- **O2-E027** — `Notify` is a stage kind. (`O2:L10`)
- **O2-E028** — `TaskSpec.sink` is a closed enum. (`O2:L10`)
- **O2-E029** — `VaultPage` is a sink variant. (`O2:L10`)
- **O2-E030** — `Digest` is a sink variant as well as a stage name. (`O2:L10`)
- **O2-E031** — `DashboardInbox` is a sink variant. (`O2:L10`)
- **O2-E032** — `Toast` is a sink variant. (`O2:L10`)
- **O2-E033** — TaskSpec rows mirror the rules-engine predicates-as-data persistence pattern. (`O2:L10`)
- **O2-E034** — The wall-clock trigger is supervisor-plane and outside the replay set under `K-14`. (`O2:L10`)
- **O2-E035** — Given recorded inputs, the actual TaskSpec run must remain deterministic. (`O2:L10`)
- **O2-E036** — Missed-run detection reuses the SSB temporal-monitor engine. (`O2:L10`)
- **O2-E037** — The existing dashboard intervention-card surface serves as the inbox sink. (`O2:L10`)
- **O2-E038** — Free-form or custom-node stage kinds are forbidden. (`O2:L11`)
- **O2-E039** — New stage kinds must ship as code through the normal plan process. (`O2:L11`)
- **O2-E040** — A wall-clock read inside the replay set is forbidden. (`O2:L11`)
- **O2-E041** — A second scheduler loop is forbidden; the task_runner interval must be reused. (`O2:L11`)
- **O2-E042** — A second inbox surface is forbidden; intervention cards must be reused. (`O2:L11`)
- **O2-E043** — Existing ground truth asserts rules predicates already persist as SQLite data under `3F`. (`O2:L13-L14`)
- **O2-E044** — Existing ground truth asserts task_runner periodic work is driven by `tokio::interval` under `C-9`. (`O2:L15`)
- **O2-E045** — Existing ground truth asserts SSB is the journal. (`O2:L16`)
- **O2-E046** — Existing ground truth asserts an SSB temporal-monitor already handles missed/late detection. (`O2:L16`)
- **O2-E047** — Existing ground truth asserts intervention cards already support human-in-the-loop inbox cards. (`O2:L17`)
- **O2-E048** — Tier 1 ships the `TaskSpec` schema and SQLite store. (`O2:L19-L20`)
- **O2-E049** — Tier 1 ships `Cron` and `Manual` triggers. (`O2:L20`)
- **O2-E050** — Tier 1 ships journaled fire rows. (`O2:L20`)
- **O2-E051** — Tier 1 ships the `DashboardInbox` sink. (`O2:L20`)
- **O2-E052** — Tier 1 stage support is `Fetch`, `Classify`, `Gate`, and `Digest`, described as the O-1-provided stages. (`O2:L20`)
- **O2-E053** — Tier 2 defers the `SsbEvent` trigger. (`O2:L21`)
- **O2-E054** — Tier 2 defers `SpawnResearchTree` and `RunPlanUnit` until an orchestrator-FSM wiring pass. (`O2:L21`)
- **O2-E055** — Tier 2 defers the missed-run detection hookup. (`O2:L21`)
- **O2-E056** — Backend ownership is `crates/orchestrator` or the rules-engine crate. (`O2:L23-L24`)
- **O2-E057** — Backend integration adds the TaskSpec table and loader. (`O2:L24`)
- **O2-E058** — Backend integration adds the trigger dispatcher to the task_runner interval. (`O2:L24`)
- **O2-E059** — Backend integration emits the SSB fire row. (`O2:L24`)
- **O2-E060** — Dashboard intervention cards render the `DashboardInbox` sink. (`O2:L25`)
- **O2-E061** — O-2 does not add a spec list/edit panel; O-4 owns that canvas surface. (`O2:L25`)
- **O2-E062** — Move M1 adds the TaskSpec table, strict closed-trigger/stage deserializer, and a boot-time loader. (`O2:L27-L28`)
- **O2-E063** — A Cron TaskSpec row must load and be enumerated by the dispatcher. (`O2:L29`)
- **O2-E064** — Unknown stage kinds hard-refuse parsing. (`O2:L29`)
- **O2-E065** — Unknown trigger variants hard-refuse parsing. (`O2:L29`)
- **O2-E066** — If either unknown value is accepted, the fix is deny-unknown strictness plus a negative parse test. (`O2:L30`)
- **O2-E067** — Move M2 wires Cron onto the existing task_runner interval. (`O2:L31`)
- **O2-E068** — A due spec fires its ordered pipeline. (`O2:L31`)
- **O2-E069** — That fire emits a journaled SSB row with its trigger cause. (`O2:L31`)
- **O2-E070** — At a due tick, a spec must fire exactly once. (`O2:L32`)
- **O2-E071** — The SSB action type is `task_spec_fired`. (`O2:L32`)
- **O2-E072** — `task_spec_fired` carries `spec_id`. (`O2:L32`)
- **O2-E073** — `task_spec_fired` carries `trigger_cause`. (`O2:L32`)
- **O2-E074** — `task_spec_fired` carries `stage_path`. (`O2:L32`)
- **O2-E075** — Double-fire or non-fire is remediated by gating on a last-fired watermark row. (`O2:L33`)
- **O2-E076** — The last-fired watermark is explicitly intended to make due-tick dispatch idempotent. (`O2:L33`)
- **O2-E077** — The idempotency path requires a double-fire assertion. (`O2:L33`)
- **O2-E078** — Move M3 routes terminal `Notify`/sink output to the intervention-card inbox. (`O2:L34`)
- **O2-E079** — A fire with results renders an inbox card. (`O2:L35`)
- **O2-E080** — A nothing-to-report fire auto-archives and leaves only its SSB row. (`O2:L34-L35`)
- **O2-E081** — Empty-inbox spam is remediated by a result-count-based `nothing-to-report -> archive` branch. (`O2:L36`)
- **O2-E082** — If the temporal monitor lacks a due/overdue query, missed-run detection moves to Tier 2. (`O2:L38-L39`)
- **O2-E083** — Under that fork, Tier 1 ships fire-on-tick only and records the seam; it must not build a second monitor. (`O2:L39`)
- **O2-E084** — Before editing, recon must determine the rules table/loader shape TaskSpec will mirror through the stated grep and codegraph exploration. (`O2:L41-L42`)
- **O2-E085** — Recon must determine whether task_runner accepts registered periodic jobs or needs extension, then read its registration site. (`O2:L43`)
- **O2-E086** — Recon must determine the intervention-card ingestion API. (`O2:L44`)
- **O2-E087** — If Cron wiring requires moving wall-clock reads into the replay/journal set, execution aborts and flags the violation. (`O2:L46-L47`)
- **O2-E088** — Trigger time is an input recorded once, never a replayed pipeline step. (`O2:L47`)
- **O2-E089** — Produced symbols include the closed trigger, stage-kind, and sink enums plus table and loader. (`O2:L49-L51`)
- **O2-E090** — Produced symbols include Cron/Manual dispatch on the task_runner interval. (`O2:L52`)
- **O2-E091** — Produced symbols include `task_spec_fired` and auto-archive-on-empty. (`O2:L53`)
- **O2-E092** — Consumed mechanisms are `3F`, `C-9`, SSB/K-14, temporal monitoring, intervention cards, and O-1 stages. (`O2:L54-L56`)
- **O2-E093** — Production reachability is a due Cron TaskSpec fired by a live task_runner interval. (`O2:L57-L59`)
- **O2-E094** — No fire is permitted without a journaled trigger-cause SSB row. (`O2:L60-L64`)
- **O2-E095** — The observable trigger registers a due-now Cron TaskSpec with a `Digest->DashboardInbox` pipeline and advances the task_runner tick. (`O2:L65-L67`)
- **O2-E096** — The observable evidence is exactly one `{spec_id, trigger_cause=Cron, stage_path}` fire row per due tick. (`O2:L67`)
- **O2-E097** — A result-bearing fire creates an intervention-card entry. (`O2:L68`)
- **O2-E098** — An empty fire that creates a card or any double-fire fails. (`O2:L68`)
- **O2-E099** — The production hook is the `C-9` interval-registration seam; the exact function is named at recon and wired in the same changeset. (`O2:L70`)
- **O2-E100** — Verification runs `wiring_contract_check.py` against this plan with `--run-trigger`. (`O2:L72-L75`)
- **O2-E101** — Verification runs `production_caller_check.py --files <changed .rs> --json`; a dark public function blocks. (`O2:L75`)
- **O2-E102** — Verification passes only after a real task_runner tick yields one off-agent fire row, a card for results, and auto-archive for empty output. (`O2:L76`)
- **O2-E103** — The independent integration smoke runs a full `Fetch->Classify->Gate->Digest` chain over a hard source. (`O2:L78`)
- **O2-E104** — The smoke advances several ticks. (`O2:L78`)
- **O2-E105** — The smoke asserts single-fire journaling and a result-bearing inbox card. (`O2:L78`)
- **O2-E106** — The smoke asserts auto-archive on a subsequent empty tick. (`O2:L78`)
- **O2-E107** — A one-tick smoke is insufficient; `R10` requires a multi-tick run exercising missed/idempotency paths. (`O2:L78`)
- **O2-E108** — The mid-pipeline clock-read red-team attack is intended to fail because supervisor-plane time is recorded and in-replay time reads are banned. (`O2:L80-L81`)
- **O2-E109** — Inbox flooding from a `1-minute` Cron that reports nothing is patched by auto-archive-on-empty while retaining the SSB audit row. (`O2:L81`)
- **O2-E110** — O-2 composes with rules-as-data `3F`, task_runner `C-9`, SSB/K-14 monitoring, intervention cards, and O-1 stages, and reimplements none. (`O2:L83-L89`)
- **O2-E111** — O-1 supplies stages, O-4 renders/edits specs, and O-3 supplies the `VaultPage` sink. (`O2:L88-L89`)
- **O2-E112** — A `#[cfg(test)]` caller never satisfies O-2 production reachability. (`O2:L57-L59`)

### O3 — O-3: brain vault contract + mounting — a portable markdown vault mounted (not converted), with namespace claims

- **O3-E001** — `O-3` is `PLANNED`, Wave `2`, product-surface class, `PROOF_SPINE: none`, and owner-acknowledged `2026-07-06`. (`O3:L1-L3`)
- **O3-E002** — Metadata is Track `RESEARCH-MACHINE`, lever `mount-not-convert`, source finding `L2`, grounding `MODERATE (index-derived, PLAN-asserted)`, nature `BUILD-on-shipped`, effort `M`, Wave `2`. (`O3:L5`)
- **O3-E003** — Brain migration is required to cost zero conversion. (`O3:L7`)
- **O3-E004** — The brain is split by storage/use grain. (`O3:L7`)
- **O3-E005** — E-5 `memory_facts` remains the machine-grain fact store. (`O3:L7`)
- **O3-E006** — `memory_facts` continues feeding prompts. (`O3:L7`)
- **O3-E007** — The existing auditor firewall around machine memory remains intact. (`O3:L7`)
- **O3-E008** — A deterministic Markdown vault is the user-grain surface. (`O3:L7`)
- **O3-E009** — The owner can read and edit the vault in Obsidian or GDrive. (`O3:L7`)
- **O3-E010** — The vault format contract comprises Markdown, YAML frontmatter, wikilinks, and machine-owned namespaces. (`O3:L7`)
- **O3-E011** — ELAI mounts the existing TEIKOKU brain vault rather than importing it. (`O3:L7`)
- **O3-E012** — Namespace claims are the only mechanism described as genuinely new for this unit. (`O3:L7`)
- **O3-E013** — Namespace claims prevent two harnesses writing one vault from colliding. (`O3:L7`)
- **O3-E014** — Topic pages use `topics/<t>/<slug>.md`. (`O3:L9-L10`)
- **O3-E015** — Pages use YAML frontmatter. (`O3:L10`)
- **O3-E016** — Pages support wikilinks. (`O3:L10`)
- **O3-E017** — Full contract includes living pages under `rankings/`. (`O3:L10`)
- **O3-E018** — Full contract includes living pages under `queue/`. (`O3:L10`)
- **O3-E019** — Full contract includes an `index` living page. (`O3:L10`)
- **O3-E020** — `write_page` is create/append-only. (`O3:L10`)
- **O3-E021** — `upsert_page` is guarded. (`O3:L10`)
- **O3-E022** — `upsert_page` is restricted to machine-owned namespaces. (`O3:L10`)
- **O3-E023** — `upsert_page` may never edit user notes. (`O3:L10`)
- **O3-E024** — A file named `namespace-claims` lives at the vault root. (`O3:L10`)
- **O3-E025** — The claims file declares which machine owns each namespace. (`O3:L10`)
- **O3-E026** — Every write outside a claimed namespace hard-refuses. (`O3:L10`)
- **O3-E027** — Namespace isolation is intended to let ELAI and TEIKOKU share one physical vault. (`O3:L10`)
- **O3-E028** — The existing `memory_facts` SQLite store remains unchanged. (`O3:L10`)
- **O3-E029** — Existing `Mem0` dispatch remains unchanged. (`O3:L10`)
- **O3-E030** — Existing machine-memory decay remains unchanged. (`O3:L10`)
- **O3-E031** — Existing `MINJA` defense remains unchanged. (`O3:L10`)
- **O3-E032** — A D-5-style life-layer denylist blocks personal topics from project prompts. (`O3:L10`)
- **O3-E033** — The same privacy boundary blocks personal topics from cloud calls. (`O3:L10`)
- **O3-E034** — Vault reads route through the existing privacy-redaction/air-gapped layer. (`O3:L10`)
- **O3-E035** — Converting the TEIKOKU vault into `memory_facts` is excluded. (`O3:L11`)
- **O3-E036** — Editing user-authored notes is excluded; allowed writer shapes are create/append and machine-namespace upsert. (`O3:L11`)
- **O3-E037** — Routing personal-brain content into project prompts is excluded. (`O3:L11`)
- **O3-E038** — A second fact store is excluded. (`O3:L11`)
- **O3-E039** — Existing ground truth asserts `memory_facts` is SQLite with Mem0-style dispatch, decay, and MINJA injection defense. (`O3:L13-L14`)
- **O3-E040** — Existing ground truth asserts a D-5 life-layer denylist and privacy-redaction/air-gapped read layer are shipped. (`O3:L15`)
- **O3-E041** — Existing ground truth asserts TEIKOKU’s writer discipline and rankings/queue/index pages are live-proven. (`O3:L16`)
- **O3-E042** — Tier 1 ships the vault contract reader and writer for frontmatter, wikilinks, and `topics/<t>/<slug>.md`. (`O3:L18-L19`)
- **O3-E043** — Tier 1 ships guarded create/append. (`O3:L19`)
- **O3-E044** — Tier 1 ships machine-namespace upsert. (`O3:L19`)
- **O3-E045** — Tier 1 ships the root namespace-claims file and hard refusal. (`O3:L19`)
- **O3-E046** — Tier 1 mounts a configured existing vault path. (`O3:L19`)
- **O3-E047** — Tier 2 defers D-5 life-layer enforcement on vault reads into prompts until a privacy-layer wiring pass. (`O3:L20`)
- **O3-E048** — Tier 2 defers bidirectional synchronization between `memory_facts` and vault views. (`O3:L20`)
- **O3-E049** — Backend ownership is the knowledge/memory crate. (`O3:L22-L23`)
- **O3-E050** — Backend integration adds the vault reader/writer module. (`O3:L23`)
- **O3-E051** — Backend integration adds the namespace-claims parser and guarded-upsert path. (`O3:L23`)
- **O3-E052** — The mount point resolves a configured vault root. (`O3:L23`)
- **O3-E053** — A dashboard vault-page view, if present, is read-only. (`O3:L24`)
- **O3-E054** — The dashboard view may be deferred entirely to Obsidian/GDrive. (`O3:L24`)
- **O3-E055** — No new ELAI panel is justified because the vault is intentionally an external-editor surface. (`O3:L24`)
- **O3-E056** — Move M1 parses frontmatter and wikilinks under the topic-path layout. (`O3:L26-L27`)
- **O3-E057** — M1 initially mounts a configured existing vault root read-only. (`O3:L27`)
- **O3-E058** — Mount acceptance lists existing TEIKOKU topics/pages without modifying them. (`O3:L28`)
- **O3-E059** — Frontmatter and wikilinks must round-trip byte-identically. (`O3:L28`)
- **O3-E060** — Any rewrite/reformat during read is failure. (`O3:L29`)
- **O3-E061** — The remedy for read mutation is a pure reader with no normalize-on-load plus a byte-identity test. (`O3:L29`)
- **O3-E062** — Move M2 gates every write through namespace claims. (`O3:L30`)
- **O3-E063** — An ELAI write into an unclaimed TEIKOKU-owned namespace yields a claim-violation error. (`O3:L31`)
- **O3-E064** — Writing into an ELAI-claimed namespace succeeds under create/append-only semantics. (`O3:L31`)
- **O3-E065** — If a cross-namespace write succeeds, the claim check must be moved into the single write chokepoint. (`O3:L32`)
- **O3-E066** — Cross-namespace protection requires a refusal test. (`O3:L32`)
- **O3-E067** — Move M3 restricts upsert to machine-owned `rankings`, `queue`, and `index` living pages. (`O3:L33`)
- **O3-E068** — Upsert may update a machine-owned rankings page in place. (`O3:L34`)
- **O3-E069** — Upsert targeting a user note must refuse. (`O3:L34`)
- **O3-E070** — If a user note is edited, the remedy is a namespace plus page-class predicate on upsert and a user-note refusal test. (`O3:L35`)
- **O3-E071** — If the TEIKOKU vault is inaccessible from the host, such as a WSL-only tree, Tier 1 uses a fresh ELAI-claimed vault. (`O3:L37-L38`)
- **O3-E072** — Under that fork, TEIKOKU mounting is a Tier 2 follow-up gated on a shared path and O-3 is not blocked. (`O3:L38`)
- **O3-E073** — Before authoring the reader, recon must read the TEIKOKU vault root and `write_page`/`upsert_page` source to establish layout and frontmatter. (`O3:L40-L41`)
- **O3-E074** — Recon must examine the `memory_facts` API to preserve a disjoint boundary and prevent accidental dual-write. (`O3:L42`)
- **O3-E075** — Recon must locate the D-5 denylist and privacy/air-gap read entrypoint. (`O3:L43`)
- **O3-E076** — If mount compatibility would require editing user-authored notes to fit ELAI, execution aborts and flags the issue. (`O3:L45-L46`)
- **O3-E077** — ELAI must adapt to the vault format; the vault may never be adapted to ELAI. (`O3:L46`)
- **O3-E078** — Produced symbols include the reader/writer for frontmatter, wikilinks, and the topic path. (`O3:L48-L50`)
- **O3-E079** — Produced symbols include the claims parser and hard refusal for unclaimed namespaces. (`O3:L51`)
- **O3-E080** — Produced symbols include guarded create/append and machine-namespace-only `upsert_page`. (`O3:L52`)
- **O3-E081** — O-3 consumes but does not dual-write or reimplement E-5 machine memory. (`O3:L53-L54`)
- **O3-E082** — O-3 consumes D-5 privacy and the TEIKOKU `brain_store` contract. (`O3:L55-L56`)
- **O3-E083** — Production reachability is an O-1/O-2 `FileToBrain` stage writing through the guarded writer during a live run. (`O3:L57-L59`)
- **O3-E084** — A production write into an unclaimed namespace must hard-refuse. (`O3:L60-L64`)
- **O3-E085** — A production path may not convert the vault into `memory_facts`. (`O3:L63`)
- **O3-E086** — A production path may not route personal-brain content into a project/cloud prompt. (`O3:L64`)
- **O3-E087** — The observable trigger mounts a vault, performs one ELAI-claimed `FileToBrain` write, and attempts one unclaimed write. (`O3:L65-L67`)
- **O3-E088** — A successful claimed write emits `vault_write`. (`O3:L67`)
- **O3-E089** — `vault_write` carries `namespace`. (`O3:L67`)
- **O3-E090** — `vault_write` carries `page`. (`O3:L67`)
- **O3-E091** — `vault_write.mode` is one of `create`, `append`, or `upsert`. (`O3:L67`)
- **O3-E092** — An unclaimed attempt emits `claim_violation`. (`O3:L67`)
- **O3-E093** — The claimed write creates a disk page with valid frontmatter and wikilinks. (`O3:L68`)
- **O3-E094** — Any unclaimed write or user-note mutation fails. (`O3:L68`)
- **O3-E095** — The production hook is the O-1/O-2 `FileToBrain` sink wired to the guarded writer in the same changeset. (`O3:L70`)
- **O3-E096** — If O-1/O-2 has not landed, interim production reachability is CLI subcommand `elai brain write`, named at recon. (`O3:L70`)
- **O3-E097** — Verification runs `wiring_contract_check.py` against this plan with `--run-trigger`. (`O3:L72-L75`)
- **O3-E098** — Verification runs `production_caller_check.py --files <changed .rs> --json`; a dark public function blocks. (`O3:L75`)
- **O3-E099** — Verification passes only when a real caller creates valid frontmatter and an unclaimed attempt produces an off-agent `claim_violation`, never silent success. (`O3:L76`)
- **O3-E100** — The independent integration smoke starts with a vault already containing a TEIKOKU-owned namespace. (`O3:L78`)
- **O3-E101** — The smoke performs both an ELAI-namespace write and an attempted TEIKOKU-namespace write. (`O3:L78`)
- **O3-E102** — The smoke asserts successful creation, hard claim refusal, and user-note-upsert refusal. (`O3:L78`)
- **O3-E103** — A single-owner happy path does not satisfy `R10`; the smoke must be a shared-vault two-owner run. (`O3:L78`)
- **O3-E104** — Cross-owner corruption is intended to fail because namespace claims scope writes and upsert is limited to machine-owned page classes. (`O3:L80-L81`)
- **O3-E105** — Personal-topic leakage is patched by applying D-5 on prompt-bound vault reads. (`O3:L81`)
- **O3-E106** — The D-5 ban is a Tier-1 `stub_ban`, but its enforcement wiring is Tier 2 and must land before personal topics may be mounted into project context. (`O3:L81`)
- **O3-E107** — O-3 composes with E-5 machine memory, D-5 privacy, TEIKOKU brain_store, and O-1/O-2 `FileToBrain`, and reimplements none. (`O3:L83-L89`)
- **O3-E108** — Mounting replaces migration while vault and `memory_facts` retain separate user and machine grains. (`O3:L88-L89`)
- **O3-E109** — A `#[cfg(test)]` caller never satisfies O-3 production reachability. (`O3:L57-L59`)

### O4 — O-4: canvas WORKFLOW mode — editor + monitor over TaskSpecs, wire-dark made visual, never a runtime

- **O4-E001** — `O-4` is `PLANNED`, Wave `3`, product-surface class, `PROOF_SPINE: none`, and owner-acknowledged `2026-07-06`. (`O4:L1-L3`)
- **O4-E002** — Metadata is Track `RESEARCH-MACHINE`, lever `canvas-as-spec-editor`, source finding `L4`, grounding `MODERATE (index-derived, PLAN-asserted)`, nature `BUILD-on-shipped`, effort `M`, Wave `3`. (`O4:L5`)
- **O4-E003** — H-12 is asserted to have shipped PLAN and RESEARCH canvas modes plus a mode bar. (`O4:L7`)
- **O4-E004** — J-31 defines canvas nodes as bound units. (`O4:L7`)
- **O4-E005** — J-31 defines canvas edges as verified claims. (`O4:L7`)
- **O4-E006** — J-31 requires canvas dispatch through the normal FSM. (`O4:L7`)
- **O4-E007** — J-31 forbids a canvas-side runtime. (`O4:L7`)
- **O4-E008** — J-60 overlays journal-derived run state on canvas nodes. (`O4:L7`)
- **O4-E009** — WORKFLOW applies that existing canvas shape to O-2 TaskSpecs. (`O4:L7`)
- **O4-E010** — A WORKFLOW node is a typed stage instance. (`O4:L7`)
- **O4-E011** — A WORKFLOW edge is dataflow carrying a provenance tier. (`O4:L7`)
- **O4-E012** — Node chrome is the stage’s last-run SSB state. (`O4:L7`)
- **O4-E013** — Editing a node edits the underlying TaskSpec row. (`O4:L7`)
- **O4-E014** — Node editing is constrained to the closed enum vocabulary. (`O4:L7`)
- **O4-E015** — Product positioning is “n8n with receipts”: gated, journaled, replayable, AI-research scoped, and not `400` ungated integrations. (`O4:L7`)
- **O4-E016** — A `WORKFLOW` mode is added to the existing canvas mode bar. (`O4:L9-L10`)
- **O4-E017** — The node palette is exactly O-2’s coded stage vocabulary; users cannot draw a stage kind that does not exist as code. (`O4:L10`)
- **O4-E018** — Edge provenance tiers come from O-1 taint labels. (`O4:L10`)
- **O4-E019** — Node run-state chrome reuses the J-60 overlay. (`O4:L10`)
- **O4-E020** — A node with a missing consumer renders visibly disconnected. (`O4:L10`)
- **O4-E021** — Visibly disconnected rendering makes wire-dark state visual. (`O4:L10`)
- **O4-E022** — Dispatch remains in the normal J-31 FSM and the canvas itself runs nothing. (`O4:L10`)
- **O4-E023** — Any canvas-side scheduler or stage runner is excluded. (`O4:L11`)
- **O4-E024** — A free-form stage kind typed on the canvas is excluded. (`O4:L11`)
- **O4-E025** — Re-authoring the mode bar is excluded. (`O4:L11`)
- **O4-E026** — Re-authoring J-60’s overlay is excluded. (`O4:L11`)
- **O4-E027** — An edge without a provenance tier is excluded. (`O4:L11`)
- **O4-E028** — Existing ground truth asserts H-12’s modes and bar, J-31’s FSM-only doctrine, and J-60’s status chrome are shipped. (`O4:L13-L16`)
- **O4-E029** — O-2 is expected to supply TaskSpec rows and closed stage, trigger, and sink enums. (`O4:L17`)
- **O4-E030** — O-1 is expected to supply taint provenance tiers. (`O4:L17`)
- **O4-E031** — Tier 1 ships the WORKFLOW mode flag in the mode bar. (`O4:L19-L20`)
- **O4-E032** — Tier 1 renders a TaskSpec as nodes and edges. (`O4:L20`)
- **O4-E033** — Tier 1 applies J-60 last-run chrome. (`O4:L20`)
- **O4-E034** — Tier 1 renders a missing consumer as visibly disconnected. (`O4:L20`)
- **O4-E035** — Tier 1’s interaction level is read-only inspection. (`O4:L20`)
- **O4-E036** — Tier 2 defers in-canvas node editing that writes back to the TaskSpec row until O-2 Tier 1 is wired. (`O4:L21`)
- **O4-E037** — Tier 2 defers edge-level provenance-tier drill-down. (`O4:L21`)
- **O4-E038** — UI ownership is `crates/dashboard-v2`, using Svelte 5 and Tauri v2. (`O4:L23-L24`)
- **O4-E039** — UI integration projects TaskSpecs to nodes and edges. (`O4:L24`)
- **O4-E040** — UI integration consumes J-60 for chrome. (`O4:L24`)
- **O4-E041** — The Tauri read surface includes `list_task_specs`. (`O4:L25`)
- **O4-E042** — The Tauri read surface includes `get_task_spec_runstate`. (`O4:L25`)
- **O4-E043** — The cross-language pair is the Rust command side plus Svelte view. (`O4:L25`)
- **O4-E044** — Move M1 extends H-12’s mode enum with WORKFLOW. (`O4:L27-L28`)
- **O4-E045** — M1 projects each stage instance to one graph node. (`O4:L28-L29`)
- **O4-E046** — M1 projects dataflow into graph edges. (`O4:L28`)
- **O4-E047** — Selecting WORKFLOW must render a configured TaskSpec as a node-per-stage graph. (`O4:L29`)
- **O4-E048** — The palette must offer exactly O-2’s closed stage kinds and no others. (`O4:L29`)
- **O4-E049** — If a free-form node appears, the palette must be bound to O-2’s enum at build time. (`O4:L30`)
- **O4-E050** — Move M2 overlays each node’s last-run SSB state as J-60 chrome. (`O4:L31`)
- **O4-E051** — A stage that ran displays its last-run status. (`O4:L32`)
- **O4-E052** — A stage whose downstream consumer is absent displays the disconnected treatment instead of a normal edge. (`O4:L32`)
- **O4-E053** — Missing-consumer state is derived from the TaskSpec edge set. (`O4:L33`)
- **O4-E054** — A wire-dark node rendered normally is fixed by wiring that derived state into rendering and adding a visual-disconnect assertion. (`O4:L33`)
- **O4-E055** — Move M3 adds a `run now` affordance that enqueues through the normal FSM. (`O4:L34`)
- **O4-E056** — The run-now affordance may never invoke a canvas-side executor. (`O4:L34`)
- **O4-E057** — A canvas-triggered run produces the same `task_spec_fired` SSB row as O-2’s scheduler. (`O4:L35`)
- **O4-E058** — The canvas introduces no canvas-owned execution event. (`O4:L35`)
- **O4-E059** — If the canvas runs a pipeline itself, the remedy is calling O-2’s dispatcher and asserting fire-row provenance shows the FSM path. (`O4:L36`)
- **O4-E060** — If O-2’s store is not queryable, O-4 Tier 1 may use a read-only fixture projection. (`O4:L38-L39`)
- **O4-E061** — Live store wiring is then gated on O-2. (`O4:L39`)
- **O4-E062** — O-4 must not create a canvas-side spec store even in fallback. (`O4:L39`)
- **O4-E063** — Before editing, recon must find and read H-12’s mode enum and registration mechanism. (`O4:L41-L42`)
- **O4-E064** — Recon must locate the J-60 API that binds last-run state to a node. (`O4:L43`)
- **O4-E065** — Recon must confirm O-2’s `list_task_specs` command or record it as a required O-2 input. (`O4:L44`)
- **O4-E066** — If WORKFLOW requires any canvas-side execution logic, execution aborts and flags the violation. (`O4:L46-L47`)
- **O4-E067** — The canvas is strictly an editor and monitor; all dispatch authority stays in the FSM. (`O4:L47`)
- **O4-E068** — Produced symbols include WORKFLOW and TaskSpec-to-nodes/edges projection. (`O4:L49-L51`)
- **O4-E069** — Produced symbols include the enum-constrained palette bound to O-2. (`O4:L52`)
- **O4-E070** — Produced symbols include missing-consumer visibly-disconnected rendering. (`O4:L53`)
- **O4-E071** — Produced symbols include both Tauri read commands. (`O4:L54`)
- **O4-E072** — O-4 consumes H-12, J-31, J-60, O-2 store/enums, and O-1 provenance rather than reauthoring them. (`O4:L55-L58`)
- **O4-E073** — Production reachability is a user selecting WORKFLOW in the live dashboard and inspecting a real TaskSpec. (`O4:L59-L61`)
- **O4-E074** — Neither cfg-test nor component-test callers satisfy production reachability. (`O4:L61`)
- **O4-E075** — `ts-rs` bindings for the Svelte/Rust frontend pair must not drift. (`O4:L67`)
- **O4-E076** — The observable trigger loads a TaskSpec with a deliberately missing downstream consumer. (`O4:L68-L70`)
- **O4-E077** — Selecting run-now must emit O-2’s `task_spec_fired` through the FSM and no canvas-owned execution row. (`O4:L70`)
- **O4-E078** — The observable artifact is nodes/edges with last-run chrome and a visibly disconnected missing-consumer node. (`O4:L71`)
- **O4-E079** — Any canvas execution path or normally-connected display of a wire-dark node fails. (`O4:L71`)
- **O4-E080** — The production hook registers WORKFLOW in H-12’s mode enum and exposes it from the dashboard route. (`O4:L73`)
- **O4-E081** — The run-now affordance calls O-2’s dispatcher in the same changeset. (`O4:L73`)
- **O4-E082** — Verification runs `wiring_contract_check.py` against this plan with `--run-trigger`. (`O4:L75-L78`)
- **O4-E083** — Verification runs `svelte_mount_reachability.py` to prove the view is reachable. (`O4:L78`)
- **O4-E084** — Verification runs `tauri_ipc_symmetry_check.py` to prove `list_task_specs` invoke/handler symmetry. (`O4:L78`)
- **O4-E085** — Verification runs `bun .claude/tools/visual/ui-wire-smoke.mjs ...` so a real WORKFLOW click lands a Rust-owned SSB row. (`O4:L79`)
- **O4-E086** — Completion requires a mount-reachable view, symmetric Tauri command, and live click through FSM dispatch rather than canvas execution. (`O4:L80`)
- **O4-E087** — The independent integration smoke uses the real dashboard and a TaskSpec spanning O-1 stages with one missing consumer. (`O4:L82`)
- **O4-E088** — The smoke selects WORKFLOW, triggers run-now, and asserts disconnected rendering, O-2’s FSM fire row, and J-60 chrome updating. (`O4:L82`)
- **O4-E089** — A component test does not satisfy `R10`; the smoke must use a live browser through the real entrypoint. (`O4:L82`)
- **O4-E090** — Client-side execution-engine smuggling is intended to fail because J-31 is a stub ban and run-now routes to O-2’s journaled dispatcher. (`O4:L84-L85`)
- **O4-E091** — Hiding wire-dark state with a normal edge is patched by deriving disconnection from the edge set and asserting it in the smoke. (`O4:L85`)
- **O4-E092** — O-4 composes with H-12, J-31, J-60, O-2, and O-1, and reimplements none. (`O4:L87-L92`)
- **O4-E093** — Final composition rule: canvas is editor/monitor, FSM is dispatcher, and wire-dark is rendered rather than hidden. (`O4:L91-L92`)

## Explicit cross-file composition

This section reorganizes already-counted explicit findings; it adds no atoms.

### Intended full-system path

1. **Define research work.** The user supplies `[[research.topics]]` records with `id`, closed `kind`, `sources[]`, `cadence`, `confidence_threshold`, and `sink` (`O1:L9-L10`). O-1 swaps only this source into Cosmos; the worker loop, stage, briefing compiler, and hourly task-runner seam remain the pre-existing execution substrate (`O0:L106-L109`; `O1:L7,L13-L16,L27-L30`).
2. **Fetch and contain untrusted bytes.** A live Cosmos tick fetches topic sources. Before prompt use, bytes receive G-1 untrusted provenance; the research role is narrowed by F-3; external bytes are recorded in a run-ID-keyed G-2 cassette (`O1:L10,L31-L36`). Classification uses an untrusted preamble, an item fence, a per-kind schema, frozen injection fixtures, closed class output, and `f32` confidence (`O1:L10,L31-L33,L48-L52`).
3. **Classify, deduplicate, and produce.** The TEIKOKU-derived workflow is fetch → enum classification → duplicate precheck against a rejected ledger → file → layered digest →, at full scope, guarded living-page upsert (`O1:L10`). Tier 1 covers `steal-scan` and `monitor`; `leaderboard`, `discovery`, and living leaderboard pages wait for Tier 2/O-3 (`O1:L19-L21`).
4. **Represent scheduled execution as data.** O-2 persists strict TaskSpec rows in SQLite. Each row combines a closed trigger, ordered closed stages, and a closed sink (`O2:L9-L10,L27-L30,L49-L53`). Tier 1 brings Cron/Manual, `Fetch/Classify/Gate/Digest`, fire rows, and DashboardInbox; event triggering, orchestrator stage kinds, and missed-run hookup are deferred (`O2:L19-L21`).
5. **Fire through the existing authority.** Supervisor-plane time makes a due decision outside replay; the existing task_runner interval dispatches the pipeline; a last-fired watermark makes a due tick idempotent; the run receives recorded time as an input (`O2:L10,L31-L33,L46-L47`). A fire writes `task_spec_fired {spec_id, trigger_cause, stage_path}` (`O2:L31-L33,L65-L70`).
6. **Surface only useful scheduled output.** Result-bearing output creates an existing intervention card. Nothing-to-report output auto-archives, leaving only the SSB receipt; the predicate is based on pipeline result count (`O2:L34-L36,L65-L68`).
7. **Write selected research to the owner vault.** O-3’s production seam is O-2’s `FileToBrain` stage calling a guarded writer (or interim `elai brain write`) (`O3:L57-L70`). The writer resolves a configured vault root, validates ownership at a single chokepoint, creates/appends only in ELAI-owned space, and upserts only machine-owned rankings/queue/index pages (`O3:L18-L35`). It records `vault_write {namespace,page,mode}` or `claim_violation` (`O3:L65-L68`).
8. **Keep the two memory grains separate.** The mounted deterministic Markdown vault is the human/owner grain; E-5 `memory_facts` remains the prompt-feeding machine grain. There is no conversion or dual-write, and ELAI adapts to existing vault bytes (`O3:L7,L10-L11,L37-L46,L53-L64`). Prompt-bound personal content is barred pending the Tier-2 D-5 read wiring (`O3:L18-L20,L80-L81`).
9. **Project, inspect, and eventually edit visually.** O-4 lists TaskSpecs through Tauri and renders a node-per-stage graph in Svelte. Provenance-labelled edges, J-60 last-run chrome, and missing-consumer disconnection make execution receipts and wire-dark state visible (`O4:L19-L33,L49-L71`). Tier 1 is read-only; Tier 2 writes node edits back to TaskSpec rows and adds provenance drill-down (`O4:L19-L21`).
10. **Run-now shares the same backend.** The canvas calls O-2’s dispatcher, which emits the same `task_spec_fired` receipt through the normal FSM. No client-side scheduler, stage runner, spec store, or separate execution event is allowed (`O4:L34-L39,L46-L47,L68-L73`).
11. **Replay and observe.** Network-dependent research replays byte-identically from G-2 with network detached; wall-clock observation itself remains outside replay (`O1:L34-L36`; `O2:L10,L46-L47`). O-1 classify rows, O-2 fire rows, O-3 write/refusal rows, and J-60’s fold of run state form the observable receipt chain (`O0:L80-L85`; `O1:L64-L75`; `O2:L65-L76`; `O3:L65-L76`; `O4:L31-L35,L68-L82`).

### Closed vocabularies and exact named values

| Vocabulary | Exact original-plan values | Admission behavior | Source |
|---|---|---|---|
| Research topic `kind` | `steal-scan`, `leaderboard`, `discovery`, `monitor` | Closed/deny-unknown; a new kind requires code; Tier 1 exposes only `steal-scan` and `monitor` | `O1:L9-L11,L19-L21,L27-L30` |
| TaskSpec `trigger` | `Cron{expr}`, `SsbEvent{action_type filter}`, `Manual` | Closed/deny-unknown; Tier 1 is Cron+Manual; SsbEvent is Tier 2 | `O2:L9-L11,L19-L21,L27-L30` |
| TaskSpec stage | `Fetch`, `Classify`, `Gate`, `FileToBrain`, `Digest`, `SpawnResearchTree`, `RunPlanUnit`, `Notify` | Ordered closed enum; novel stage code goes through plan process; Tier 1 explicitly names Fetch/Classify/Gate/Digest | `O2:L9-L11,L19-L21` |
| TaskSpec sink | `VaultPage`, `Digest`, `DashboardInbox`, `Toast` | Closed enum; Tier 1 explicitly names DashboardInbox | `O2:L10,L19-L20` |
| Classifier output | `{class: enum, confidence: f32}` | Never free text; zero out-of-enum; low-confidence canary goes to `unsorted` | `O1:L31-L33` |
| Vault write `mode` | `create`, `append`, `upsert` | Create/append inside a claimed namespace; upsert additionally requires machine-owned page class | `O3:L30-L35,L65-L68` |
| Track status | `PLANNED`, `IN PROGRESS`, `DONE`, `DEFERRED` | `DONE` has wiring, caller, smoke, and artifact gates beyond cargo-green | `O0:L42-L45` |

### Data, journal, and lifecycle surfaces

| Surface | Authoritative state in the plan | Lifecycle / observable result |
|---|---|---|
| Research topics | User-editable manifest `[[research.topics]]` | Existing hourly Cosmos loop enumerates topics; existing Cosmos artifacts can time-decay to `DECAYED`; drawer shows kind and last-run trust tier |
| Rejected ledger | Store consulted by O-1 duplicate precheck | Duplicate handling occurs before file/digest; exact record schema is not given |
| G-2 cassette | External bytes keyed by run ID | Detached-network, byte-identical replay; divergence detects unrecorded reads |
| TaskSpec | SQLite rows loaded at boot | Cron/manual dispatch at Tier 1; last-fired watermark protects each due tick; event trigger/missed-run hookup deferred |
| SSB classify row | O-1 schema-version bump site | Carries `{class, confidence, taint_tier}` and proves canary containment |
| `task_spec_fired` | O-2 schema-version bump site | Carries `{spec_id, trigger_cause, stage_path}` exactly once per due tick; reused by O-4 run-now |
| Inbox intervention card | Existing dashboard surface | Created only for result-bearing runs; empty runs auto-archive and retain SSB only |
| Markdown vault | Configured mounted root | Pure read/byte-identical round trip; claimed create/append; machine-owned upsert; external-editor ownership |
| `namespace-claims` | Root file mapping namespaces to machines | Hard refusal and `claim_violation` for unclaimed writes; permits ELAI/TEIKOKU cohabitation |
| `vault_write` | O-3 schema-version bump site | Records namespace, page, and create/append/upsert mode |
| Canvas projection | Derived from TaskSpec rows plus J-60 SSB run state | Read-only Tier 1; editing Tier 2; missing consumers are visually disconnected |

### Dependencies, rollout, and production owners

| Unit | Exact status / wave | Production owner and entrypoint | Hard dependency / fallback |
|---|---|---|---|
| O-1 | `PLANNED`, `[KEYSTONE]`, effort `M-L`, Wave 1 | Knowledge/Cosmos owner; live C-16 `live_cli` Cosmos tick | Consume frozen G-1/F-3/G-2; if manifest is hard-inlined, add a loader adapter without forking the loop |
| O-2 | `PLANNED`, effort `M-L`, Wave 1 | `crates/orchestrator` **or** rules crate; C-9 task_runner interval | O-1 stage implementations; if no temporal due-query exists, ship fire-on-tick and defer missed-run monitoring |
| O-3 | `PLANNED`, effort `M`, Wave 2 | Knowledge/memory crate; O-1/O-2 FileToBrain or interim `elai brain write` | Needs a writer caller; if TEIKOKU path is cross-host inaccessible, use a fresh ELAI namespace and defer shared mount |
| O-4 | `PLANNED`, effort `M`, Wave 3 | `crates/dashboard-v2`; live dashboard mode route + O-2 dispatcher | Queryable O-2 store; if absent, render a read-only fixture and never invent a client store |

O-1 and O-2 are explicitly parallel-safe in Wave 1 because their declared crate touch sets differ. O-3 follows when a FileToBrain/CLI caller exists. O-4 follows a queryable TaskSpec store, with a fixture-only Tier-1 fallback (`O0:L87-L96`). All four are product-surface units with no direct `PS-1..PS-6` contribution (`O0:L58-L60`).

### Failure, refusal, and recovery paths

| Failure / attack | Required response or fallback | Source |
|---|---|---|
| Unknown topic kind | Hard parse refusal; deny-unknown enum and negative test | `O1:L27-L30` |
| Canary reaches prompt unlabelled | Re-anchor classifier to taint-wrapped item; freeze fixture | `O1:L31-L33` |
| Replay diverges | Treat as leaked network read; route remaining fetch through VCR and add detached assertion | `O1:L34-L36` |
| Cosmos manifest hard-inlined | Stop in-place edit; use new loader+adapter; do not fork worker | `O1:L38-L39` |
| Trust gate/cassette format would need modification | Abort and flag; O-1 only consumes frozen gates | `O1:L45-L46` |
| Unknown TaskSpec trigger/stage | Hard parse refusal; strict enum and negative test | `O2:L27-L30` |
| Due spec double-fires or never fires | Gate with last-fired watermark and test duplicate firing | `O2:L31-L33` |
| Empty runs spam inbox | Result-count predicate sends them to auto-archive | `O2:L34-L36` |
| Temporal monitor lacks due query | Defer missed-run hookup, ship fire-on-tick, build no second monitor | `O2:L38-L39` |
| Clock would move inside replay | Abort and flag | `O2:L46-L47` |
| Reader reformats mounted page | Make reads pure; prohibit normalize-on-load; byte-identity test | `O3:L26-L29` |
| Cross-namespace write succeeds | Put claim check at sole write chokepoint; refusal test | `O3:L30-L32` |
| Upsert edits user note | Add namespace plus page-class predicate; refusal test | `O3:L33-L35` |
| Shared vault inaccessible | Fresh ELAI-claimed root; defer cross-host mounting | `O3:L37-L38` |
| ELAI schema would require user-note edits | Abort; ELAI adapts to vault | `O3:L45-L46` |
| Palette admits a free-form node | Bind palette to O-2 enum at build | `O4:L27-L30` |
| Missing consumer looks connected | Derive disconnection from edge set and assert visual treatment | `O4:L31-L33` |
| Canvas tries to execute | Call O-2 dispatcher/FSM and prove fire-row path | `O4:L34-L36,L46-L47` |
| O-2 store unavailable | Read-only fixture projection; never create canvas-side store | `O4:L38-L39` |

### Exact recon and verification commands carried by the originals

These commands restate counted executor/test obligations; they are preserved literally so small operational features are not lost.

| Unit | Purpose | Exact plan command |
|---|---|---|
| O-1 | Cosmos manifest/classifier recon | `grep -rn "cosmos_worker\|CosmosStage\|manifest" .elai_cc/crates` plus `codegraph_explore` on `cosmos_worker` |
| O-1 | Taint API recon | `grep -rn "taint\|provenance\|untrusted" .elai_cc/crates/*/src` |
| O-1 | Wiring proof | `python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/O-plan/O-1_research_daemon_generalized_cosmos.md --run-trigger` |
| O-1 | Production caller proof | `python .claude/tools/production_caller_check.py --files <changed .rs> --json` |
| O-2 | Rules-store recon | `grep -rn "rules\|predicate\|CREATE TABLE" .elai_cc/crates` plus `codegraph_explore` on the rules loader |
| O-2 | Interval-registration recon | `grep -rn "task_runner\|tokio::interval\|register" .elai_cc/crates/*/src` |
| O-2 | Intervention-card recon | `grep -rn "intervention\|InterventionCard\|inbox" .elai_cc/crates .elai_cc/crates/dashboard-v2/src` |
| O-2 | Wiring proof | `python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/O-plan/O-2_scheduled_taskspec_substrate.md --run-trigger` |
| O-2 | Production caller proof | `python .claude/tools/production_caller_check.py --files <changed .rs> --json` |
| O-3 | Machine-memory boundary recon | `grep -rn "memory_facts\|Mem0\|MINJA" .elai_cc/crates` |
| O-3 | Privacy boundary recon | `grep -rn "life.layer\|denylist\|air.gap\|redact" .elai_cc/crates` |
| O-3 | Wiring proof | `python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/O-plan/O-3_brain_vault_contract_mounting.md --run-trigger` |
| O-3 | Production caller proof | `python .claude/tools/production_caller_check.py --files <changed .rs> --json` |
| O-4 | Mode registration recon | `grep -rn "mode.bar\|CanvasMode\|PLAN\|RESEARCH" .elai_cc/crates/dashboard-v2/src` |
| O-4 | Run-state overlay recon | `grep -rn "J-60\|runstate\|overlay\|node.*status" .elai_cc/crates/dashboard-v2/src` |
| O-4 | Wiring proof | `python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/O-plan/O-4_canvas_workflow_mode.md --run-trigger` |
| O-4 | Svelte/Tauri reachability and symmetry | `python .claude/tools/svelte_mount_reachability.py` and `python .claude/tools/tauri_ipc_symmetry_check.py` |
| O-4 | Real UI wire smoke | `bun .claude/tools/visual/ui-wire-smoke.mjs ...` |

## Corrections and extensions versus `15-O-SUMMARY_INDEX.discovery.md`

The comparison target is orientation only. “Correction” below means the original is narrower or differently staged than the prior summary ledger; it does not mean the summary file was edited.

### Material corrections / narrowed readings

1. **Digest patches are per-unit, not literally identical across every unit.** O0 says `A1..A5` feed per-unit `stub_ban`/`REDTEAM` (`O0:L4-L7,L55-L56`). The detail plans specialize them: O-1 applies A1/A3, O-2 A2/A3, O-3 A4, and O-4 A3. The summary-ledger claim that every unit enforces the same A1, A3, and A4 set is broader than the originals.
2. **The engagement falsifier is not present in these five originals.** The summary ledger’s digest-open, `For-You` click, `N`-day zero-engagement, and honest topic-decay requirements are not stated in O0 or O1–O4. O1 does state an existing artifact lifecycle that decays to `DECAYED` (`O1:L7`), but it does not connect that state to opens/clicks or an `N`-day threshold. Those engagement requirements remain summary-only in this source scope.
3. **O-4 editing is not Tier 1.** The original makes Tier 1 read-only inspection and explicitly defers TaskSpec write-back plus provenance drill-down to Tier 2 (`O4:L19-L21`). Any summary reading of node editing as an immediate ship-now feature needs this tier qualifier.
4. **O-2 full vocabulary is larger than its Tier-1 runtime.** The schema names eight stages and four sinks, but Tier 1 explicitly names only Fetch/Classify/Gate/Digest and DashboardInbox; SsbEvent, SpawnResearchTree, RunPlanUnit, and missed-run wiring are Tier 2 (`O2:L9-L10,L19-L21`). Schema availability must not be confused with production wiring.
5. **O-1 full topic enum is larger than Tier 1.** Four kinds are schema-valid, but only `steal-scan` and `monitor` ship now; `leaderboard`, `discovery`, and living-page upserts are deferred (`O1:L19-L21`).
6. **Exactly-once is scoped to a due tick.** The original’s concrete mechanism is a last-fired watermark and the acceptance phrase “exactly once per due tick” (`O2:L31-L33,L65-L68`); it does not claim global exactly-once under crash, multi-process, or distributed scheduling.
7. **Privacy is a staged safety barrier.** Personal-to-project/cloud flow is a hard exclusion/stub ban, but the actual D-5 read-path wiring is Tier 2 and must land before personal topics become mountable into project context (`O3:L10-L11,L18-L20,L60-L64,L80-L81`).
8. **“Dark/absent” is not the exact status vocabulary of the originals.** All four details say `PLANNED`; O0 defines the formal statuses. The originals identify new versus reused mechanisms but do not independently inventory current code to prove each component dark (`O0:L42-L45`; `O1:L3`; `O2:L3`; `O3:L3`; `O4:L3`).
9. **The trust-spine boundary is more precise than a blanket byte claim.** O1 says G-1 wraps fetched bytes, F-3 applies to the research role, and G-2 records external bytes; integration names fetch/classify/sink seams (`O1:L10,L23-L24,L31-L36`). Metadata, redirect, attachment, and cache coverage remain unspecified.
10. **No new O-4 event is planned.** O0 explicitly says O-4 reuses O-2’s fire row, while O-1/O-2/O-3 add schema action types (`O0:L80-L85`).

### Original-only or materially expanded detail

- O0 supplies full wiring doctrine, release gates, `WIRE-DARK[O-N]`, `cross_lang_diff`, schema-baseline procedure, kill-switch convention, `blake3` requirement, affected-crate test scope, no `.unwrap()`/`.expect()` production rule, and anti-slopsquatting requirement (`O0:L42-L64,L79-L103`).
- O1 supplies the complete topic record shape, including `id`, `sources[]`, `cadence`, `confidence_threshold`, and `sink`; untrusted preamble/item fence/per-kind schema; rejected ledger; layered digest; `f32` confidence; run-ID cassette keying; detached-network byte identity; CosmosDrawer trust-tier chrome; adapter fork; and exact hard-input smoke (`O1:L9-L10,L23-L39,L64-L77`).
- O2 supplies trigger operands (`Cron{expr}`, `SsbEvent{action_type filter}`), the eight stage values, boot-time loading, the last-fired watermark, result-count auto-archive, trigger-time-as-recorded-input rule, intervention ingestion recon, fire-on-tick fork, and `1-minute` empty-Cron attack (`O2:L9-L11,L27-L47,L65-L81`).
- O3 supplies the exact topic path, root claims location, write modes, living-page classes, pure-read/no-normalize rule, byte-identical round trip, single write chokepoint, page-class upsert check, cross-host fallback, optional/no-new-panel UI stance, interim CLI, and two-owner smoke (`O3:L9-L10,L18-L46,L65-L81`).
- O4 supplies Svelte 5/Tauri v2 ownership, two read-command names, ts-rs drift ban, read-only Tier 1, edge-set-derived disconnection, real browser smoke, fixture projection fallback, and the prohibition on a client-side spec store (`O4:L19-L25,L31-L47,L49-L89`).

### Summary questions resolved by originals

- Topic-manifest fields are now enumerated, though their types/defaults remain open (`O1:L10`).
- TaskSpec’s stage enum is now fully enumerated (`O2:L10`).
- Cron and SSB-event operands are named (`O2:L10`).
- O-1 replay is keyed by run ID and tested byte-identically with networking detached (`O1:L34-L36`).
- O-2’s first idempotency mechanism is a last-fired watermark row (`O2:L31-L33`).
- Auto-archive is keyed off pipeline result count and leaves the SSB row (`O2:L34-L36`).
- O-3’s machine-owned upsert classes are rankings/queue/index (`O3:L33-L35`).
- O-3 reads must not normalize content (`O3:L26-L29`).
- O-4 missing-consumer rendering is derived from the TaskSpec edge set (`O4:L31-L33`).
- O-4’s frontend pair and validation commands are named (`O4:L67,L75-L80`).

### Summary questions not resolved by originals

- `VaultPage` versus `FileToBrain` naming remains unresolved: O2 calls the former a sink and the latter a stage; O3 calls FileToBrain both a stage and sink seam (`O2:L10,L88-L89`; `O3:L57-L70`).
- The `D-5` reference namespace remains unresolved; O3 repeats “D-5 life-layer denylist” but does not reconcile it to the differently titled D-plan file noted by the summary pass (`O3:L10,L15,L20,L43,L55,L64,L81`).
- `unsorted` is still not typed as a class, queue, path, or sink (`O1:L32,L66,L75`).
- Class-enum values and confidence range/calibration remain unstated (`O1:L10,L31-L33`).
- Shared-vault locking and atomic-write semantics remain unstated.
- Canvas write-side IPC, edit revisions, concurrent edits, authorization, and save errors remain unstated.

## Inferred system behavior — not explicit and not counted

- **O-INF01** — Full completion likely presents a single journey: configure a typed topic, schedule a closed pipeline, run trusted/replayable research, send useful output to inbox or vault, and inspect/edit/run the same spec on the canvas. This is a synthesis of declared composition edges, not a verbatim golden journey.
- **O-INF02** — O-4 likely folds O-2’s `task_spec_fired` plus stage-specific rows into J-60 chrome, but the fold key and error-state model are not stated.
- **O-INF03** — `VaultPage` likely denotes a terminal destination while `FileToBrain` denotes the executable stage that invokes it, but the originals never define that mapping.
- **O-INF04** — `unsorted` likely functions as an abstention/quarantine destination for low-confidence or adversarial items, but its schema and review lifecycle are unknown.
- **O-INF05** — O-1’s `sink` topic field likely chooses or parameterizes an O-2 sink, but the files do not state the adapter or validation rule.
- **O-INF06** — O-4’s typed stage node can satisfy J-31’s “bound unit” rule if each node binds a concrete TaskSpec stage implementation; that binding record is not specified.
- **O-INF07** — O-4’s provenance-bearing dataflow edge can satisfy J-31’s “verified claim” rule if its tier is journal-derived and validated; the proof record is not specified.
- **O-INF08** — A last-fired watermark likely uses `(spec_id, due-window)` or an equivalent identity, but no key or transaction scope is stated.
- **O-INF09** — The safest reading of O3’s privacy staging is that ordinary non-personal mounts can ship in Tier 1, while personal topics cannot enter project-context mounts until Tier 2.
- **O-INF10** — Because O-3 requires pure reads and permits external editors, robust implementation likely needs atomic writes and external-change detection, but neither mechanism is designed here.
- **O-INF11** — Because O-4 Tier 1 can use a fixture, that release may demonstrate projection UX without satisfying full live production reachability; the final done gate still appears to require the live O-2 caller.
- **O-INF12** — O0’s `memory-writer ran` completion item likely refers to plan/session memory bookkeeping, not O-3 vault writes, but this is not defined in O sources.

## Ambiguous, conflicting, or underspecified statements

### Internal staging tensions

1. **O-4 read-only Tier 1 versus run-now verification.** Tier 1 says read-only inspection, while M3, the production hook, observable effect, verifier, and integration smoke require a live run-now click (`O4:L19-L21,L34-L36,L68-L82`). Run-now is not assigned explicitly to Tier 1 or Tier 2.
2. **O-3 privacy ban versus deferred enforcement.** The safety rule is unconditional, yet D-5 read-path wiring is Tier 2 (`O3:L10-L11,L18-L20,L60-L64,L80-L81`). The original partially resolves this by forbidding personal topics in project context until Tier 2, but it does not state what Tier-1 mount validation enforces.
3. **O-3 bidirectional sync versus no dual-write.** Bidirectional synchronization of “views” is deferred while memory/vault dual-write is forbidden (`O3:L20,L42,L53-L64`). The permitted future sync direction and derived-view semantics need definition.
4. **O-2 schema versus Tier wiring.** `FileToBrain`, `Notify`, `VaultPage`, `Digest`, and `Toast` appear in the full schema, while the Tier blocks state only some inclusions and two specific deferrals (`O2:L9-L10,L19-L21`). The shipping status of FileToBrain, Notify, and non-Dashboard sinks is not fully assigned.
5. **Manual trigger “on interval.”** The produced-symbol block calls the Cron/Manual dispatcher “on the task_runner interval” (`O2:L52`), but a manual action normally has a direct entrypoint; the original does not settle whether Manual is polled, enqueued, or invoked synchronously.

### Naming and type ambiguities

1. **`VaultPage` / `FileToBrain`:** sink versus stage versus production seam is inconsistent across O2/O3.
2. **O-1 stages consumed by O-2:** O2 calls Fetch/Classify/Gate/Digest “the ones O-1 already provides,” but O1’s named workflow includes duplicate-precheck, file, layered digest, and confidence gate without defining corresponding reusable stage symbols (`O1:L10`; `O2:L20,L56`).
3. **Classifier `class`:** enum members are not listed. `unsorted` may be a member but is never typed as one.
4. **Confidence:** representation is `f32`, but range, threshold comparison, calibration, NaN handling, and per-kind/per-sink interaction are absent.
5. **Topic schema:** field names are given, but field types for `id`, `sources[]`, `cadence`, threshold, and sink; defaults; enabled state; uniqueness; and versioning are absent.
6. **TaskSpec identity:** no `spec_id` field is shown in the schema even though the event contains `spec_id`; revision, owner, timestamps, operands, and stage-output typing are absent.
7. **`namespace-claims`:** filename has no extension and its serialization, namespace grammar, claim precedence, acquisition/release, and conflict syntax are absent.
8. **Machine-owned page detection:** rankings/queue/index are named, but ownership metadata and treatment of a user-edited machine page are absent.
9. **Missing consumer:** derived from the edge set, but invalid/disabled/unavailable consumers and sink termination are not distinguished.
10. **Provenance tier:** O-1 produces and O-4 displays it, but the tier vocabulary and edge aggregation rule are absent.
11. **Classify action type:** O0 says O-1 adds a classify row but does not give its exact action-type string (`O0:L82`).

### Concurrency, lifecycle, and recovery gaps

- Last-fired watermark transaction boundaries, crash-before/after-row behavior, multi-process dispatch, retry idempotency, and catch-up/coalescing remain unspecified.
- Trigger timezone, Cron dialect, jitter, startup behavior, overlap, maximum run duration, cancellation, pause/resume, deletion, and retention remain unspecified.
- `SsbEvent` recursion/loop prevention, filter matching, event ordering, and replay interaction remain unspecified.
- Fetch retry/backoff, source rate limits, partial responses, cache behavior, provider degradation, secret handling, unsafe URLs, and partial pipeline failure remain unspecified.
- Rejected-ledger fingerprinting, false-positive handling, cross-topic scope, retention, and override remain unspecified.
- Vault atomicity, locking, simultaneous writers, symlink/path traversal rules, case sensitivity, partial-write recovery, backup/restore, corruption handling, and file-watcher behavior remain unspecified.
- Inbox/card read state, dismissal, retention, failure cards, deduplication, and the exact auto-archived object remain unspecified.
- Canvas branch/fan-in/fan-out/cycle representation, layout persistence, accessibility, save conflicts, undo/redo, cancellation, duplicate run-now clicks, and permission checks remain unspecified.

### Status and evidence caveat

All four unit files are `PLANNED`; all existing-ground-truth blocks are explicitly labelled `PLAN-asserted` and direct executors to re-grep under W-7. Therefore this ledger describes designed and asserted behavior, not implementation-confirmed shipment. `PROOF_SPINE: none` does not remove release evidence: the O0 done definition and each unit’s off-agent wiring/integration smoke remain blocking plan gates (`O0:L42-L64`; `O1:L71-L77`; `O2:L72-L78`; `O3:L72-L78`; `O4:L75-L82`).

## Category coverage and explicit non-goals

| Requested dimension | Original-source coverage |
|---|---|
| Commands/actions | Cosmos tick, TaskSpec boot/load/fire, run-now, FileToBrain, `elai brain write`, list/get Tauri reads, create/append/upsert |
| Inputs/outputs | Topic record fields; classifier object; TaskSpec trigger/stage/sink enums; three SSB event shapes; Markdown page output |
| Workflows/state | Fetch/classify/dedupe/file/digest; due-tick/watermark/card; mount/read/claim/write; project/chrome/disconnect/run |
| Schema/data/lifecycle | SQLite TaskSpecs, rejected ledger, run-ID cassette, namespace claims, vault paths, auto-archive, existing `DECAYED` state |
| Config/defaults | Named fields and strict closed enums; kill-switch convention; no concrete defaults for cadence, clock, enabled state, or paths |
| UI/API/CLI | CosmosDrawer extension, intervention cards, optional external-editor/read view, Svelte WORKFLOW, two Tauri reads, interim brain CLI |
| Integrations/events | B-42/C-9/3F/E-5/D-5/G-1/F-3/G-2/H-12/J-31/J-60; classify/fire/vault/refusal rows |
| Permissions/security/privacy | Taint, tool role filter, closed schemas, frozen canaries, namespace claims, user-note ban, D-5 boundary, no client runtime |
| Failure/recovery | Strict refusals, adapter/fallback paths, watermark, auto-archive, pure read, write chokepoint, fixture projection, explicit aborts |
| Determinism/idempotency/concurrency | Detached byte replay, supervisor-plane clock, recorded trigger input, due-tick watermark; distributed/vault concurrency unspecified |
| Observability | SSB receipt chain, CosmosDrawer trust tier, inbox cards, J-60 chrome, visible disconnection, off-agent artifacts |
| Tests/done gates | Wiring/caller checks, negative parses, canaries, detached replay, multi-tick smoke, two-owner vault smoke, real-browser smoke |
| Constraints/non-goals | No generic engine, custom kinds/nodes, second scheduler/inbox/fact store, vault conversion, user-note edits, canvas runtime, trust-gate edits |
| Dependencies/status/deferred | All four PLANNED; exact waves/effort; Tier 1/Tier 2 and fork conditions recorded above |

## Processed-path manifest and omission/duplicate check

| # | Exact processed primary path | Lines read | SHA-256 recheck | Ledger atoms | Inclusion count |
|---:|---|---:|---|---:|---:|
| 1 | `docs/thefinalglance/O-plan/O-0_INDEX.md` | 125/125 | `a7988ad7c1f9c3dac2bd89d12996f3256d4aef0e1b5c4c11deb0e1f1fa41c217` | 95 | 1 |
| 2 | `docs/thefinalglance/O-plan/O-1_research_daemon_generalized_cosmos.md` | 90/90 | `d1b23db489099d48d375c58b353da55eb9e40ffc2d7b9ad0b9c3a256bf1ece38` | 111 | 1 |
| 3 | `docs/thefinalglance/O-plan/O-2_scheduled_taskspec_substrate.md` | 91/91 | `db5ba4492f42d55246c7a7c969c2337f86003fb55e2ef9e4bb11da8f2b915584` | 112 | 1 |
| 4 | `docs/thefinalglance/O-plan/O-3_brain_vault_contract_mounting.md` | 91/91 | `a30462d74e297fa7935ec8c62484b27546ef6ce41c5d210e2f634902632754c0` | 109 | 1 |
| 5 | `docs/thefinalglance/O-plan/O-4_canvas_workflow_mode.md` | 94/94 | `5deae2ce45ece1bba69c4bdaeead2222a5f727785603b599d7c1a10ac9c71899` | 93 | 1 |
| **Total** | **5 original files** | **491/491** | **all matched initial inventory** | **520** | **5 exact inclusions** |

Excluded exactly once by rule: `docs/thefinalglance/O-plan/O-SUMMARY_INDEX.md` (summary, not an original). Orientation-only, non-primary file: `docs/thefinalglance/feature-discovery/15-O-SUMMARY_INDEX.discovery.md`.

Final checks: **0 omitted originals; 0 duplicate source inclusions; 0 duplicate atomic IDs; 0 out-of-scope plan-family sources; 520 explicit atomic findings.**
