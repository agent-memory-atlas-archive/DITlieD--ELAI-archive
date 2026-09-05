# O-plan exhaustive feature discovery — summary-index pass

## Scope and evidence discipline

- **Primary evidence read completely:** `O-plan/O-SUMMARY_INDEX.md`, 84/84 lines, including the premise, all four `RESEARCH-MACHINE` units, all `COMPOSES-WITH` / integration / `observable_effect` clauses, and the cross-cutting digest patches.
- **Orientation only:** `ELAI-SPEC-OVERVIEW.md`. Its O paragraph corroborates the four-unit shape, but no atomic item below is promoted from the overview.
- **Deliberately not used as feature evidence in this pass:** `O-0_INDEX.md`, the four original O unit plans, and the source-design document. They are queued in the deeper-source manifest so a later original-plan pass can confirm, refine, or contradict this summary.
- **Source-anchor notation:** `O-SUMMARY:Lx-Ly` means lines in `O-SUMMARY_INDEX.md` as numbered in the current file.
- **Discovery IDs are analyst-assigned**, solely to make atoms deduplicable. The canonical plan IDs and names remain exactly `O-1 — research daemon (generalize Cosmos)`, `O-2 — scheduled TaskSpec substrate`, `O-3 — brain vault contract + mounting`, and `O-4 — canvas WORKFLOW mode`.
- **Epistemic rule:** “explicit” means stated by the summary, including its assertions about already-shipped or externally proven pieces. It does **not** mean independently implementation-verified. Inferences and unresolved readings are isolated in later sections.

## Quantitative coverage

| Measure | Coverage |
|---|---:|
| Primary-summary lines read | 84 / 84 (100%) |
| O feature units inventoried | 4 / 4 (100%) |
| Explicit atomic statements indexed | 185 (global 14; O-1 42; O-2 43; O-3 35; O-4 33; cross-cutting 18) |
| Unit metadata blocks captured | 4 / 4 |
| `COMPOSES-WITH` blocks captured | 4 / 4 |
| Integration blocks captured | 4 / 4 |
| Positive `observable_effect` workflows captured | 4 / 4 |
| Explicit negative failure conditions captured | 8 (2 per unit) |
| Named values in fully enumerated enums | 11 (topic kind 4, trigger 3, sink 4) |
| Cross-cutting digest patches captured | A1, A2, A3, A4 (4 / 4) |
| Explicitly separated synthesis items | 10 inferences; 33 ambiguities; 12 summary-silent audit categories |
| Deeper O documents read for feature evidence | 0 (intentionally deferred) |

### Category coverage in the summary

| Requested category | Summary coverage | Where |
|---|---|---|
| Capabilities/actions | Strong | O-1 through O-4 atomic inventories |
| Workflows | Strong | O-1 pipeline, O-2 fire path, O-3 guarded write, O-4 render/edit/run path |
| Surfaces | Strong | Typed-topic surface, CosmosDrawer, inbox cards, markdown vault, WORKFLOW canvas |
| State/data/lifecycle | Strong | Manifest/topics, TaskSpec SQLite rows, SSB events, vault pages/claims, last-run state, engagement decay |
| Configuration | Partial | Closed enums and user-editable topics are named; schemas and editing mechanics are not |
| Integrations | Strong | Exact subsystem and crate/surface integration points are named |
| Errors/fallbacks | Partial | Eight fail conditions and hard refusals are named; most retry/degrade behavior is absent |
| Security/privacy | Strong | Taint, tool filtering, replay, canaries, namespace claims, life-layer denylist, closed vocabularies |
| Observability | Strong | SSB classifier/fire rows, vault/violation rows, last-run chrome, engagement signals |
| Tests/operations | Partial | Four live done-gates and eight negative legs; rollout, migration, backup, and runbook detail absent |
| Dependencies/status | Strong at summary granularity | Existing/reused/absent pieces, waves, effort, and dependencies are explicit |
| Future/deferred | Partial | Wave order and dark/absent mechanisms are named; no formal deferred backlog appears in the summary |

## Explicit atomic inventory

### Track-wide premise and product contract

| Discovery ID | Atomic statement | Kind | Source |
|---|---|---|---|
| O.G.E01 | The track is named `RESEARCH-MACHINE`. | Identity | O-SUMMARY:L1,L15 |
| O.G.E02 | The desired product is a “TEIKOKU-style personal research machine on the ELAI canvas.” | Product surface | O-SUMMARY:L9-L10 |
| O.G.E03 | The product decomposes into four vertical slices. | Architecture | O-SUMMARY:L10 |
| O.G.E04 | The four slices are asserted to be approximately 80% composition of shipped subsystems. | Composition/status assertion | O-SUMMARY:L10-L11 |
| O.G.E05 | The summary distinguishes TEIKOKU by noting that TEIKOKU already has classify canaries. | Comparative constraint | O-SUMMARY:L11 |
| O.G.E06 | The summary distinguishes n8n by characterizing its nodes as arbitrary and ungated. | Comparative constraint | O-SUMMARY:L11-L12 |
| O.G.E07 | Every O unit is required to be gated. | Global invariant | O-SUMMARY:L12-L13 |
| O.G.E08 | Every O unit is required to be journaled. | Global invariant | O-SUMMARY:L12-L13 |
| O.G.E09 | Every O unit is required to be replayable. | Global invariant | O-SUMMARY:L12-L13 |
| O.G.E10 | Every O unit is required to use a closed vocabulary. | Global invariant | O-SUMMARY:L12-L13 |
| O.G.E11 | O is classified as a product-surface letter. | Scope/status | O-SUMMARY:L13 |
| O.G.E12 | O declares `PROOF_SPINE: none`. | Proof status | O-SUMMARY:L13 |
| O.G.E13 | The absence of a proof-spine contribution is owner-acknowledged. | Proof status | O-SUMMARY:L13 |
| O.G.E14 | O feeds no `PS-1..PS-6` number. | Measurement status | O-SUMMARY:L13 |

### O-1 — research daemon (generalize Cosmos) [KEYSTONE]

Metadata preserved exactly: track `RESEARCH-MACHINE`; lever `manifest-generalization`; grounding `MODERATE (index-derived, PLAN-asserted)`; nature `BUILD-on-shipped`; effort `M-L`; `WAVE 1`; marked `[KEYSTONE]`. Source: O-SUMMARY:L17-L19.

| Discovery ID | Atomic statement | Kind | Source |
|---|---|---|---|
| O-1.E01 | `CosmosWorker` is asserted to already be a scheduled research loop. | Existing capability assertion | O-SUMMARY:L20 |
| O-1.E02 | The existing loop has a manifest. | Existing state/config | O-SUMMARY:L20 |
| O-1.E03 | The existing loop has an hourly tick. | Existing scheduling behavior | O-SUMMARY:L20 |
| O-1.E04 | The existing loop performs `FlashResearch` refresh. | Existing workflow behavior | O-SUMMARY:L20-L21 |
| O-1.E05 | The existing loop applies time decay. | Existing lifecycle behavior | O-SUMMARY:L21 |
| O-1.E06 | The existing loop produces a briefing capped at `<=1500` tokens. | Existing bound | O-SUMMARY:L21 |
| O-1.E07 | The existing loop uses `CosmosStage`. | Existing integration | O-SUMMARY:L21 |
| O-1.E08 | The current manifest is hard-bound to designer briefs. | Current limitation | O-SUMMARY:L21 |
| O-1.E09 | O-1 generalizes only the manifest, rather than replacing the research loop. | Scope constraint | O-SUMMARY:L21-L22 |
| O-1.E10 | The generalized manifest represents a user-editable topic set. | New configuration feature | O-SUMMARY:L22 |
| O-1.E11 | Each topic is typed. | Data contract | O-SUMMARY:L22 |
| O-1.E12 | Topic `kind` is a closed enum. | Safety/data invariant | O-SUMMARY:L22 |
| O-1.E13 | `steal-scan` is an allowed topic-kind value. | Enum value | O-SUMMARY:L22 |
| O-1.E14 | `leaderboard` is an allowed topic-kind value. | Enum value | O-SUMMARY:L22 |
| O-1.E15 | `discovery` is an allowed topic-kind value. | Enum value | O-SUMMARY:L22-L23 |
| O-1.E16 | `monitor` is an allowed topic-kind value. | Enum value | O-SUMMARY:L23 |
| O-1.E17 | G-1 taint is threaded through every fetched byte. | Security/provenance behavior | O-SUMMARY:L23 |
| O-1.E18 | The F-3 tool filter is threaded through every fetched byte. | Security behavior | O-SUMMARY:L23 |
| O-1.E19 | G-2 VCR is threaded through every fetched byte. | Replay behavior | O-SUMMARY:L23 |
| O-1.E20 | Each topic pipeline begins with `fetch`. | Workflow stage | O-SUMMARY:L23-L24 |
| O-1.E21 | The fetched item goes through `enum-classify`. | Workflow stage | O-SUMMARY:L24 |
| O-1.E22 | Classification is followed by a duplicate precheck (`dup-precheck`). | Workflow stage | O-SUMMARY:L24 |
| O-1.E23 | The per-topic pipeline includes `file`. | Workflow stage | O-SUMMARY:L24 |
| O-1.E24 | The per-topic pipeline ends in `digest`. | Workflow stage | O-SUMMARY:L24 |
| O-1.E25 | The `fetch->enum-classify->dup-precheck->file->digest` pipeline is asserted to be TEIKOKU-proven. | External grounding/status assertion | O-SUMMARY:L23-L25 |
| O-1.E26 | ELAI adds taint to the TEIKOKU-proven pipeline. | Added mechanism | O-SUMMARY:L24-L25 |
| O-1.E27 | ELAI adds replay on top of the classify canaries. | Added mechanism | O-SUMMARY:L24-L25 |
| O-1.E28 | The typed-topic surface is dark/absent at summary time. | Missing/status | O-SUMMARY:L25 |
| O-1.E29 | The enum-classify stage is absent as ELAI code and exists only in TEIKOKU at summary time. | Missing/status | O-SUMMARY:L25 |
| O-1.E30 | `cosmos_worker`, `CosmosStage`, and the briefing compiler must be extended and must not be re-authored. | Composition constraint | O-SUMMARY:L26 |
| O-1.E31 | O-1 composes with `FlashResearch`. | Dependency | O-SUMMARY:L26 |
| O-1.E32 | The production integration adds a cosmos manifest loader in `crates/knowledge`. | Integration point | O-SUMMARY:L27 |
| O-1.E33 | The production integration adds the classify stage in `crates/knowledge`. | Integration point | O-SUMMARY:L27 |
| O-1.E34 | `CosmosDrawer` is extended; no new drawer is created. | Surface/composition constraint | O-SUMMARY:L27 |
| O-1.E35 | The positive done-gate uses a live cosmos tick to drive a `steal-scan` topic over a canary-injection fixture. | Test/workflow | O-SUMMARY:L28 |
| O-1.E36 | That run must land an off-agent classify row in the SSB. | Observability/evidence | O-SUMMARY:L28-L29 |
| O-1.E37 | The classify row carries `class`. | Event field | O-SUMMARY:L29 |
| O-1.E38 | The classify row carries `confidence`. | Event field | O-SUMMARY:L29 |
| O-1.E39 | The classify row carries `taint_tier`. | Event field | O-SUMMARY:L29 |
| O-1.E40 | The injected canary must route to `unsorted`. | Safety/test behavior | O-SUMMARY:L29-L30 |
| O-1.E41 | An out-of-enum classifier output fails the done-gate. | Negative invariant | O-SUMMARY:L29-L30 |
| O-1.E42 | A canary reaching a steal sink fails the done-gate. | Negative invariant | O-SUMMARY:L30 |

### O-2 — scheduled TaskSpec substrate

Metadata preserved exactly: track `RESEARCH-MACHINE`; lever `rules-as-data-scheduling`; grounding `MODERATE (index-derived, PLAN-asserted)`; nature `BUILD-on-shipped`; effort `M-L`; `WAVE 1`. Source: O-SUMMARY:L32-L34.

| Discovery ID | Atomic statement | Kind | Source |
|---|---|---|---|
| O-2.E01 | O-2 adopts the Codex Automations pattern of time schedules. | Borrowed behavior | O-SUMMARY:L35 |
| O-2.E02 | O-2 adopts the Codex Automations pattern of event schedules. | Borrowed behavior | O-SUMMARY:L35 |
| O-2.E03 | Scheduled results flow to an inbox. | Borrowed workflow/surface | O-SUMMARY:L35 |
| O-2.E04 | The borrowed automation flow includes auto-archive. | Borrowed lifecycle behavior | O-SUMMARY:L35 |
| O-2.E05 | ELAI's scheduled substrate is typed. | Data invariant | O-SUMMARY:L35 |
| O-2.E06 | ELAI's scheduled substrate is journaled. | Evidence invariant | O-SUMMARY:L35 |
| O-2.E07 | A `TaskSpec` is stored as a SQLite row. | Persistence contract | O-SUMMARY:L35-L36 |
| O-2.E08 | The row follows the rules-engine rows-as-data pattern. | Composition/data pattern | O-SUMMARY:L36 |
| O-2.E09 | `TaskSpec.trigger` is a closed choice among named trigger kinds. | Data contract | O-SUMMARY:L36 |
| O-2.E10 | `Cron` is an allowed trigger value. | Enum value | O-SUMMARY:L36 |
| O-2.E11 | `SsbEvent` is an allowed trigger value. | Enum value | O-SUMMARY:L36 |
| O-2.E12 | `Manual` is an allowed trigger value. | Enum value | O-SUMMARY:L36 |
| O-2.E13 | `TaskSpec.pipeline` is ordered. | Data/workflow invariant | O-SUMMARY:L36-L37 |
| O-2.E14 | Pipeline stage kinds come from a closed enum. | Data/safety invariant | O-SUMMARY:L36-L37 |
| O-2.E15 | `TaskSpec.sink` is a closed choice among named sink kinds. | Data contract | O-SUMMARY:L37 |
| O-2.E16 | `VaultPage` is an allowed sink value. | Enum value | O-SUMMARY:L37 |
| O-2.E17 | `Digest` is an allowed sink value. | Enum value | O-SUMMARY:L37 |
| O-2.E18 | `DashboardInbox` is an allowed sink value. | Enum value | O-SUMMARY:L37 |
| O-2.E19 | `Toast` is an allowed sink value. | Enum value | O-SUMMARY:L37 |
| O-2.E20 | Every TaskSpec fire emits a journaled SSB row. | Observability/evidence | O-SUMMARY:L37-L38 |
| O-2.E21 | Every fire row carries the trigger cause. | Event field/invariant | O-SUMMARY:L37-L38 |
| O-2.E22 | K-14 keeps the wall-clock trigger in the supervisor plane. | Determinism boundary | O-SUMMARY:L38 |
| O-2.E23 | Keeping wall-clock triggering in the supervisor plane is intended to keep the run deterministic. | Determinism behavior | O-SUMMARY:L38-L39 |
| O-2.E24 | Missed-run detection reuses the SSB temporal monitor. | Lifecycle/operations reuse | O-SUMMARY:L38-L39 |
| O-2.E25 | Task results reuse the shipped intervention-card inbox surface. | Surface reuse/status assertion | O-SUMMARY:L39 |
| O-2.E26 | The typed TaskSpec itself is absent at summary time. | Missing/status | O-SUMMARY:L39-L40 |
| O-2.E27 | Trigger-to-`task_runner` wiring is absent at summary time. | Missing/status | O-SUMMARY:L39-L40 |
| O-2.E28 | O-2 composes with rules-engine rows-as-data (`3F`). | Dependency | O-SUMMARY:L41 |
| O-2.E29 | O-2 composes with the `task_runner` interval (`C-9`). | Dependency | O-SUMMARY:L41 |
| O-2.E30 | O-2 composes with SSB and the K-14 temporal monitor. | Dependency | O-SUMMARY:L41-L42 |
| O-2.E31 | O-2 composes with intervention cards. | Dependency | O-SUMMARY:L42 |
| O-2.E32 | O-2's pipeline vocabulary consumes O-1 stage kinds. | Cross-O dependency | O-SUMMARY:L42 |
| O-2.E33 | The integration must add a `TaskSpec` table in `crates/orchestrator` or the rules crate. | Integration point with unresolved owner | O-SUMMARY:L43 |
| O-2.E34 | The integration must add a TaskSpec dispatcher. | Integration point | O-SUMMARY:L43 |
| O-2.E35 | The integration must add fire-row emission. | Integration point | O-SUMMARY:L43 |
| O-2.E36 | A due-now `Cron` TaskSpec must fire exactly once per tick. | Scheduling invariant/done-gate | O-SUMMARY:L44 |
| O-2.E37 | The emitted event is named `task_spec_fired`. | Event type | O-SUMMARY:L44 |
| O-2.E38 | `task_spec_fired` carries `spec_id`. | Event field | O-SUMMARY:L44-L45 |
| O-2.E39 | `task_spec_fired` carries `trigger_cause`. | Event field | O-SUMMARY:L44-L45 |
| O-2.E40 | `task_spec_fired` carries `stage_path`. | Event field | O-SUMMARY:L44-L45 |
| O-2.E41 | A result-bearing fire renders an inbox card. | Surface/done-gate | O-SUMMARY:L45 |
| O-2.E42 | An empty fire that nevertheless renders a card fails the done-gate. | Negative invariant | O-SUMMARY:L45-L46 |
| O-2.E43 | A double-fire fails the done-gate. | Negative invariant | O-SUMMARY:L46 |

### O-3 — brain vault contract + mounting

Metadata preserved exactly: track `RESEARCH-MACHINE`; lever `mount-not-convert`; grounding `MODERATE (index-derived, PLAN-asserted)`; nature `BUILD-on-shipped`; effort `M`; `WAVE 2`. Source: O-SUMMARY:L48-L50.

| Discovery ID | Atomic statement | Kind | Source |
|---|---|---|---|
| O-3.E01 | O-3 splits “the brain” by storage/use grain. | Architecture invariant | O-SUMMARY:L51 |
| O-3.E02 | E-5 `memory_facts` remains the machine-grain store. | Existing role/status | O-SUMMARY:L51 |
| O-3.E03 | The machine-grain `memory_facts` store feeds prompts. | Existing role | O-SUMMARY:L51 |
| O-3.E04 | A markdown vault is the user-grain surface. | New user surface | O-SUMMARY:L51-L53 |
| O-3.E05 | The markdown vault format is deterministic. | Data invariant | O-SUMMARY:L51-L52 |
| O-3.E06 | Topic pages use the path contract `topics/<t>/<slug>.md`. | File-layout contract | O-SUMMARY:L52 |
| O-3.E07 | Vault pages use frontmatter. | File-format contract | O-SUMMARY:L52 |
| O-3.E08 | Vault pages support wikilinks. | File-format/navigation feature | O-SUMMARY:L52 |
| O-3.E09 | The vault contains living rankings. | User-data lifecycle feature | O-SUMMARY:L52 |
| O-3.E10 | The vault contains a living queue. | User-data lifecycle feature | O-SUMMARY:L52 |
| O-3.E11 | The vault contains a living index. | User-data lifecycle/navigation feature | O-SUMMARY:L52 |
| O-3.E12 | The vault adopts TEIKOKU `brain_store.write_page` discipline verbatim. | External contract reuse | O-SUMMARY:L52-L53 |
| O-3.E13 | The vault adopts TEIKOKU `brain_store.upsert_page` discipline verbatim. | External contract reuse | O-SUMMARY:L52-L53 |
| O-3.E14 | The markdown format itself is the migration contract. | Migration invariant | O-SUMMARY:L53-L54 |
| O-3.E15 | Migration requires zero conversion. | Mounting/migration behavior | O-SUMMARY:L53-L54 |
| O-3.E16 | ELAI mounts the existing TEIKOKU vault. | Integration behavior | O-SUMMARY:L54 |
| O-3.E17 | A namespace-claims file exists at the vault root. | New state/config mechanism | O-SUMMARY:L54-L55 |
| O-3.E18 | The claims file records which namespaces are claimed for writing. | Access-control contract | O-SUMMARY:L55 |
| O-3.E19 | A write outside a claimed namespace receives a hard refusal. | Security/error behavior | O-SUMMARY:L55-L56 |
| O-3.E20 | Namespace claims are what allow two harnesses to share one vault. | Multi-writer isolation behavior | O-SUMMARY:L55-L56 |
| O-3.E21 | A D-5 life-layer denylist provides the privacy boundary. | Privacy dependency | O-SUMMARY:L56 |
| O-3.E22 | Personal topics are kept out of project prompts. | Privacy invariant | O-SUMMARY:L56 |
| O-3.E23 | Personal topics are kept out of cloud prompts. | Privacy invariant | O-SUMMARY:L56 |
| O-3.E24 | The vault and E-5 `memory_facts` remain disjoint grains. | Storage invariant | O-SUMMARY:L57 |
| O-3.E25 | The two stores must not dual-write. | Storage/lifecycle invariant | O-SUMMARY:L57 |
| O-3.E26 | O-3 composes with the O-1/O-2 `FileToBrain` sink. | Cross-O dependency | O-SUMMARY:L58 |
| O-3.E27 | The integration adds a vault reader in `crates/knowledge`. | Integration point | O-SUMMARY:L59 |
| O-3.E28 | The integration adds a vault writer in `crates/knowledge`. | Integration point | O-SUMMARY:L59 |
| O-3.E29 | The integration adds a namespace-claims parser. | Integration point | O-SUMMARY:L59 |
| O-3.E30 | The integration adds guarded upsert. | Integration/security point | O-SUMMARY:L59 |
| O-3.E31 | A `FileToBrain` write inside an ELAI-claimed namespace creates a valid page. | Positive done-gate | O-SUMMARY:L60 |
| O-3.E32 | A successful claimed-namespace write emits a `vault_write` row. | Observability/evidence | O-SUMMARY:L60 |
| O-3.E33 | An attempted unclaimed-namespace write emits a `claim_violation` row. | Refusal observability | O-SUMMARY:L60-L61 |
| O-3.E34 | Any actual write into an unclaimed namespace fails the done-gate. | Negative invariant | O-SUMMARY:L61-L62 |
| O-3.E35 | Any mutation of a user note fails the done-gate. | Negative invariant | O-SUMMARY:L61-L62 |

### O-4 — canvas WORKFLOW mode

Metadata preserved exactly: track `RESEARCH-MACHINE`; lever `canvas-as-spec-editor`; grounding `MODERATE (index-derived, PLAN-asserted)`; nature `BUILD-on-shipped`; effort `M`; `WAVE 3`; depends on `O-2`. Source: O-SUMMARY:L64-L66.

| Discovery ID | Atomic statement | Kind | Source |
|---|---|---|---|
| O-4.E01 | A `WORKFLOW` mode is added to the shipped canvas mode bar. | New surface | O-SUMMARY:L67 |
| O-4.E02 | The mode bar is H-12 and must be extended rather than replaced. | Surface dependency/constraint | O-SUMMARY:L67,L73 |
| O-4.E03 | WORKFLOW follows the shape of J-31's canvas doctrine. | Architecture dependency | O-SUMMARY:L67-L68 |
| O-4.E04 | A workflow node represents a bound unit. | Graph semantic | O-SUMMARY:L67-L68 |
| O-4.E05 | A workflow edge represents a verified claim. | Graph semantic | O-SUMMARY:L68 |
| O-4.E06 | Workflow dispatch goes through the normal FSM. | Execution invariant | O-SUMMARY:L68 |
| O-4.E07 | The canvas has no independent runtime. | Execution/security invariant | O-SUMMARY:L68 |
| O-4.E08 | WORKFLOW projects O-2 TaskSpecs. | Cross-O surface dependency | O-SUMMARY:L68 |
| O-4.E09 | Each node is a typed stage instance. | Graph/data contract | O-SUMMARY:L68-L69 |
| O-4.E10 | Each edge represents dataflow. | Graph semantic | O-SUMMARY:L69 |
| O-4.E11 | Each edge carries a provenance tier. | Provenance/data contract | O-SUMMARY:L69 |
| O-4.E12 | Node chrome displays last-run SSB state. | Observability/surface | O-SUMMARY:L69 |
| O-4.E13 | The last-run chrome reuses J-60. | Surface dependency | O-SUMMARY:L69,L73 |
| O-4.E14 | Editing a node edits the underlying TaskSpec row. | Editing/persistence behavior | O-SUMMARY:L70 |
| O-4.E15 | Node edits are constrained to the closed enum. | Editing/safety invariant | O-SUMMARY:L70 |
| O-4.E16 | A node with a missing consumer renders visibly disconnected. | Error visualization | O-SUMMARY:L70-L71 |
| O-4.E17 | The surface makes a wire-dark condition visual instead of silently implying connectivity. | Error visualization invariant | O-SUMMARY:L71 |
| O-4.E18 | The intended product shape is “n8n with receipts.” | Product positioning | O-SUMMARY:L71 |
| O-4.E19 | The `WORKFLOW` mode is absent at summary time. | Missing/status | O-SUMMARY:L71-L72 |
| O-4.E20 | The `TaskSpec->nodes` projection is absent at summary time. | Missing/status | O-SUMMARY:L71-L72 |
| O-4.E21 | The enum-constrained palette is absent at summary time. | Missing/status | O-SUMMARY:L72 |
| O-4.E22 | WORKFLOW consumes the O-2 TaskSpec store and enums. | Cross-O dependency | O-SUMMARY:L73 |
| O-4.E23 | WORKFLOW consumes O-1 tiers. | Cross-O dependency | O-SUMMARY:L73 |
| O-4.E24 | The UI integration is a Svelte WORKFLOW view in `crates/dashboard-v2`. | Production entrypoint | O-SUMMARY:L74 |
| O-4.E25 | Rust exposes a `list_task_specs` Tauri command. | API/integration point | O-SUMMARY:L74 |
| O-4.E26 | Rust exposes a `get_task_spec_runstate` Tauri command. | API/integration point | O-SUMMARY:L74 |
| O-4.E27 | Selecting WORKFLOW in the live dashboard renders a TaskSpec as nodes and edges. | Positive done-gate | O-SUMMARY:L75 |
| O-4.E28 | The rendered graph includes J-60 last-run chrome. | Positive done-gate | O-SUMMARY:L75 |
| O-4.E29 | A missing-consumer node renders disconnected in the live surface. | Positive/error done-gate | O-SUMMARY:L75-L76 |
| O-4.E30 | `run-now` emits O-2's fire row. | Action/observability | O-SUMMARY:L76 |
| O-4.E31 | `run-now` reaches execution through the FSM. | Execution invariant/done-gate | O-SUMMARY:L76 |
| O-4.E32 | Any canvas-side execution path fails the done-gate. | Negative invariant | O-SUMMARY:L76-L77 |
| O-4.E33 | A normally connected node that renders wire-dark fails the done-gate. | Negative invariant | O-SUMMARY:L76-L77 |

### Cross-cutting digest patches and engagement falsifier

| Discovery ID | Atomic statement | Kind | Source |
|---|---|---|---|
| O.X.E01 | Every O unit enforces the same A1, A3, and A4 digest patches. | Cross-unit invariant | O-SUMMARY:L79-L82 |
| O.X.E02 | Those patches are enforced as `stub_ban + REDTEAM`. | Verification posture | O-SUMMARY:L79-L80 |
| O.X.E03 | A1 forbids sending a free-text classifier output to a prompt. | Classifier safety invariant | O-SUMMARY:L80-L81 |
| O.X.E04 | A1 requires classifier output to use a closed enum. | Classifier data invariant | O-SUMMARY:L80-L81 |
| O.X.E05 | A1 requires classifier output to carry confidence. | Classifier data invariant | O-SUMMARY:L81 |
| O.X.E06 | A1 requires frozen canaries. | Test/safety invariant | O-SUMMARY:L81 |
| O.X.E07 | A3 forbids generic nodes. | Workflow vocabulary invariant | O-SUMMARY:L81 |
| O.X.E08 | A3 forbids custom nodes. | Workflow vocabulary invariant | O-SUMMARY:L81 |
| O.X.E09 | A3 requires a closed stage vocabulary. | Workflow data invariant | O-SUMMARY:L81-L82 |
| O.X.E10 | A new stage kind must ship as code. | Extension invariant | O-SUMMARY:L81-L82 |
| O.X.E11 | A4 says ELAI ships the substrate. | Ownership boundary | O-SUMMARY:L82 |
| O.X.E12 | A4 says topic packs are data. | Ownership/config boundary | O-SUMMARY:L82 |
| O.X.E13 | A4 requires disjoint vault namespaces for the two masters. | Multi-writer invariant | O-SUMMARY:L82 |
| O.X.E14 | A2 is the whole-feature falsifier for the default-off graveyard. | Product validation invariant | O-SUMMARY:L83 |
| O.X.E15 | A successful digest is connected to a measured digest-open signal. | Engagement observability | O-SUMMARY:L83-L84 |
| O.X.E16 | A successful digest is connected to a measured `For-You` click signal. | Engagement observability | O-SUMMARY:L83-L84 |
| O.X.E17 | Zero engagement is evaluated after `N` days. | Lifecycle threshold | O-SUMMARY:L84 |
| O.X.E18 | A topic with zero engagement after `N` days decays honestly. | Lifecycle behavior/falsifier | O-SUMMARY:L84 |

## Explicit workflows and how the units compose

These are restatements of explicit links, not new mechanisms.

### 1. Per-topic research path (`O-1`)

`user-edited typed topic manifest` -> live/hourly Cosmos tick -> `fetch` (G-1 taint + F-3 tool filter + G-2 VCR on every fetched byte) -> closed-enum classification -> duplicate precheck -> file -> digest -> off-agent SSB classify evidence. A frozen canary must be diverted to `unsorted`; it must never reach a steal sink. Source: O-SUMMARY:L20-L30,L80-L81.

### 2. Scheduled execution path (`O-2`)

Supervisor-plane `Cron`, `SsbEvent`, or `Manual` trigger -> persisted typed `TaskSpec` -> ordered closed-enum stages -> one journaled `task_spec_fired` row with cause and stage path -> one of the typed sinks -> intervention-card inbox only when the fire bears a result. Temporal monitoring detects missed runs; a Cron due now fires exactly once per tick. Source: O-SUMMARY:L35-L46.

### 3. Guarded vault path (`O-3`)

O-1/O-2 `FileToBrain` request -> namespace-claims parser -> hard write-scope check -> guarded vault writer/upsert -> deterministic markdown page and `vault_write` row. A denied attempt emits `claim_violation` without producing the forbidden write. Prompt-bound reads remain separated from E-5 machine memory, and personal topics may not enter project/cloud prompts. Source: O-SUMMARY:L51-L62.

### 4. Visual editing and run path (`O-4`)

Select WORKFLOW -> list/project O-2 TaskSpecs to typed nodes and provenance-bearing edges -> overlay last-run SSB state -> edit a node by editing its closed-enum-constrained spec row -> display missing consumers as disconnected -> `run-now` dispatches through the normal FSM -> O-2 emits the fire row. The canvas may never execute directly. Source: O-SUMMARY:L67-L77.

### 5. Track validation loop

Digest success -> measure digest opens and `For-You` clicks -> after `N` days of zero engagement, decay the topic instead of preserving an unused default-off feature. Source: O-SUMMARY:L83-L84.

## Invariants spanning units

1. **Closed-vocabulary all the way down:** topic kind, classifier class, TaskSpec trigger/stage/sink, and canvas palette/editing cannot silently expand into free text or generic/custom nodes. New stage kinds ship as code. (O-SUMMARY:L22-L23,L35-L37,L70-L72,L80-L82)
2. **One execution authority:** WORKFLOW is a spec editor/projection, not a runtime; `run-now` enters the normal FSM and produces O-2 journal evidence. (O-SUMMARY:L67-L77)
3. **Off-agent evidence:** O-1 classification lands off-agent in SSB; O-2 fires, O-3 writes/refusals, and O-4 run state all surface journal-derived evidence. (O-SUMMARY:L28-L30,L37-L46,L60-L62,L69,L75-L77)
4. **Fail closed:** out-of-enum classification, canary-to-steal, duplicate fire, carding an empty result, unclaimed writes, user-note mutation, canvas-side execution, and false wire-dark rendering are explicit failures. (O-SUMMARY:L29-L30,L45-L46,L61-L62,L76-L77)
5. **Untrusted-input containment:** fetched bytes acquire taint, tool filtering, and replay coverage before downstream classification/storage; frozen canaries exercise the real path. (O-SUMMARY:L23-L30,L80-L81)
6. **Two-master separation:** ELAI owns a reusable substrate, topic packs remain data, stores remain grain-disjoint, and vault writers remain in disjoint claimed namespaces. (O-SUMMARY:L51-L58,L82)
7. **No migration-by-copy:** ELAI mounts the existing contract-compatible vault and does not convert it into `memory_facts` or dual-write both grains. (O-SUMMARY:L53-L58)
8. **Privacy boundary:** personal topics do not flow into project or cloud prompts. (O-SUMMARY:L56-L58)
9. **Deterministic time boundary:** wall-clock trigger observation stays in the supervisor plane while the run remains deterministic and journaled. (O-SUMMARY:L37-L39)
10. **Honest product use:** digest production alone is insufficient; engagement is measured and unused topics decay after an explicitly configurable-but-unspecified `N`-day interval. (O-SUMMARY:L83-L84)

## Dependencies, ownership, status, and sequencing

| Unit | Reused/extended | Genuinely new or absent | Production owner/entrypoint | Sequence/status |
|---|---|---|---|---|
| `O-1 — research daemon (generalize Cosmos)` | `cosmos_worker`, `CosmosStage`, briefing compiler, `FlashResearch`, G-1, F-3, G-2, `CosmosDrawer` | User-editable typed-topic manifest surface; ELAI enum-classify stage | `crates/knowledge` manifest loader + classify stage; extend `CosmosDrawer` | `[KEYSTONE]`, Wave 1, M-L; MODERATE, index-derived/PLAN-asserted; build on shipped; both new pieces explicitly dark/absent |
| `O-2 — scheduled TaskSpec substrate` | 3F rules-as-data, C-9 task-runner interval, SSB/K-14 temporal monitor, intervention cards, O-1 stage kinds | Typed SQLite `TaskSpec`; trigger-to-task-runner wiring; dispatcher/fire emitter | `crates/orchestrator` **or** rules crate (owner unresolved by summary) | Wave 1, M-L; MODERATE, index-derived/PLAN-asserted; build on shipped; typed spec and wiring absent |
| `O-3 — brain vault contract + mounting` | E-5 `memory_facts` as a disjoint machine grain; D-5-style denylist/privacy layer; TEIKOKU `brain_store`; O-1/O-2 FileToBrain | Root namespace-claims file, mounting/guarded access in ELAI | `crates/knowledge` vault reader/writer, claims parser, guarded upsert | Wave 2, M; MODERATE, index-derived/PLAN-asserted; build on shipped |
| `O-4 — canvas WORKFLOW mode` | H-12 mode bar, J-31 doctrine, J-60 overlay, O-2 store/enums, O-1 tiers | WORKFLOW mode, TaskSpec-to-node projection, enum-constrained palette | `crates/dashboard-v2` Svelte view + Rust Tauri commands | Wave 3, M; MODERATE, index-derived/PLAN-asserted; build on shipped; explicitly depends O-2; all three surface pieces absent |

### Summary-level deferred/future status

- No item is formally labeled `DEFERRED` in the summary.
- Wave ordering is explicit: O-1 and O-2 in Wave 1, O-3 in Wave 2, O-4 in Wave 3.
- O-4 explicitly depends on O-2; O-2 consumes O-1 stage kinds; O-3 consumes an O-1/O-2 sink; O-4 also consumes O-1 tiers.
- “Dark/absent” is explicit for O-1's typed-topic surface and ELAI enum-classify code, O-2's typed spec and trigger wiring, and O-4's mode/projection/palette.
- The summary does not state implementation completion for O-3's new claims mechanism, nor does it supply a shipped/dark ledger for its integration files.
- Track-level proof status remains `PROOF_SPINE: none`; none of these product-surface units contributes a `PS-1..PS-6` number.

## Inferred composition — not explicit requirements

The following readings are useful for whole-system synthesis but must remain marked **inferred** until an original plan confirms them.

| Inference ID | Inferred interaction | Basis | Confirmation needed |
|---|---|---|---|
| O.INF01 | The intended end-to-end product likely lets a user define research topics, schedule their closed stages, persist selected results into the mounted vault, inspect results/inbox state, and edit/run the same workflow visually. | O-2 consumes O-1 stage kinds; O-3 names an O-1/O-2 sink; O-4 projects/runs O-2. | Confirm exact golden journey and which sinks each topic kind uses. |
| O.INF02 | SSB rows likely provide the replay/last-run substrate used by O-4 chrome. | O-1/O-2 journal rows plus O-4 last-run SSB state. | Confirm event-fold keys, row types, and replay source of truth. |
| O.INF03 | `VaultPage` and `FileToBrain` may be two names or two layers for the same terminal action. | O-2 enumerates `VaultPage`; O-3 says O-1/O-2 `FileToBrain` sink. | Resolve terminology/schema in O-2 and O-3 originals. |
| O.INF04 | Topic decay likely changes manifest scheduling or priority rather than deleting vault content. | A2 says zero engagement “decays the topic.” | Confirm the mutated field, decay curve, reversibility, and whether scheduling is disabled. |
| O.INF05 | `unsorted` appears to be a quarantine/non-steal destination for ambiguous or canary classifications. | Canary routes to `unsorted`; reaching a steal sink fails. | Confirm whether `unsorted` is a class, queue, file path, or sink and what later review does. |
| O.INF06 | “No canvas runtime” likely means all graph edits are persisted as spec data and all execution authority stays engine-side. | Node edit updates the spec row; run-now goes via FSM. | Confirm validation, optimistic concurrency, approvals, and mutation commands. |
| O.INF07 | Two-harness vault sharing likely depends on non-overlapping claims plus atomic/locked writes. | Namespace claims prevent writes outside ownership. | Locking/concurrency is not stated; confirm in original. |
| O.INF08 | A “valid page” likely means conformance to the deterministic TEIKOKU markdown/frontmatter/wikilink contract. | Layout/contract and observable effect appear in the same unit. | Confirm validation rules and invalid-page refusal behavior. |
| O.INF09 | “Result-bearing” likely means a fire with a material user-visible output, not merely a successful empty run. | Empty fires must not card. | Confirm exact predicate and representation of empty/skipped/no-change outcomes. |
| O.INF10 | The user-editable topic set is likely data-only, while adding a novel stage implementation is code-only. | A4 makes topic packs data; A3 makes new stage kinds code. | Confirm who can edit packs and admission/versioning rules. |

## Ambiguities and contradictions to resolve in original sources

1. **`D-5` namespace collision:** the summary calls D-5 a “life-layer denylist,” while the canonical ELAI file named `D-plan/D-5_bai_thompson_promotion.md` has a different title. `D-5` may instead be a section/requirement ID inside the TEIKOKU/source-design document. Do not bind this privacy feature to ELAI D-plan/D-5 without reconciliation.
2. **Topic kinds versus stage kinds:** O-1 explicitly enumerates topic `kind`, but O-2 says it consumes “O-1 stage kinds.” The summary does not enumerate a stage-kind enum or say whether `fetch`, `enum-classify`, `dup-precheck`, `file`, and `digest` are its values.
3. **Sink naming mismatch:** O-2's sink enum contains `VaultPage`; O-3 calls the O-1/O-2 sink `FileToBrain`. Their relation is unspecified.
4. **Crate ownership unresolved:** the TaskSpec table is assigned to `crates/orchestrator (or rules crate)`.
5. **VCR undefined here:** G-2 VCR is named but its record/replay boundary, file format, and interaction with the G-2 reproducible-run artifact are not restated.
6. **“Every fetched byte” boundary unspecified:** the summary does not say whether metadata, redirects, attachments, tool outputs, and cached bytes all share the same taint/filter/VCR treatment.
7. **Classifier contract incomplete:** confidence type/range, class enum values, abstention, invalid-output repair/refusal, and ownership of the off-agent classifier are not specified.
8. **Canary contract incomplete:** frozen canary corpus, injection frequency, matching rules, and the exact definition of “steal sink” are absent.
9. **`unsorted` undefined:** it may be a classifier class, holding queue, vault namespace, or sink.
10. **Topic manifest schema incomplete:** only editability and the four `kind` values are named. IDs, query/source fields, cadence, enabled state, ownership, validation, versioning, and persistence are absent.
11. **Cosmos scheduling detail incomplete:** “hourly tick” is stated for the existing loop, but timezone, jitter, startup behavior, locking, and overlap rules are absent.
12. **TaskSpec schema incomplete:** IDs, revisions, stage operands, stage output typing, sink operands, enabled/paused state, ownership, and timestamps are unstated.
13. **Exactly-once scope incomplete:** “exactly once per tick” does not settle transaction boundaries, crash recovery, multiple processes, retried dispatch, or idempotency keys.
14. **Missed-run semantics incomplete:** detection is named, but catch-up, skip, coalesce, maximum backlog, and notification policies are not.
15. **Auto-archive semantics incomplete:** the borrowed flow names auto-archive without stating which artifact archives, when, or under what completion criteria.
16. **Result-bearing predicate undefined:** therefore the empty-fire/card negative cannot yet be implemented from the summary alone.
17. **SSB schemas incomplete:** row version, timestamps/clock domain, parent/run IDs, error fields, and replay ordering are absent for classify and fire events.
18. **Vault claims format incomplete:** filename spelling, schema, namespace syntax, precedence, conflict resolution, and claim acquisition/release are unstated.
19. **Shared-vault concurrency incomplete:** atomicity, locking, simultaneous writers, external-editor changes, and recovery from partial writes are unstated.
20. **User-note boundary incomplete:** the summary prohibits “any mutation of a user note” but does not define how a user note is distinguished from an ELAI-owned page or whether append counts as mutation.
21. **Guarded upsert semantics incomplete:** create/replace/merge behavior, conflict detection, and page ownership metadata are not stated.
22. **Vault page schema incomplete:** required frontmatter keys, wikilink validity, slug normalization, encoding, and ranking/queue/index update rules are absent.
23. **Mount validation incomplete:** the summary does not state what happens when an existing TEIKOKU vault violates the contract or contains unknown namespaces.
24. **Privacy enforcement point incomplete:** D-5 denylist is named, but config source, read boundaries, redaction behavior, local-versus-cloud distinction, and failure evidence are not.
25. **Provenance tiers incomplete:** O-4 edges carry a tier and consume O-1 tiers, but no tier vocabulary or admission rules are supplied.
26. **Graph projection incomplete:** fan-in/out, branching, cycles, ordering, graph layout, sink nodes, trigger nodes, and invalid TaskSpec rendering are not specified.
27. **Missing-consumer semantics incomplete:** the summary does not define how a consumer is deemed missing or distinguish missing from temporarily unavailable/disabled.
28. **Wire-dark semantics incomplete:** it is unclear whether “wire-dark” means no event producer, stale data, an unresolved type edge, or only a visual state.
29. **Editing API incomplete:** only read-side Tauri commands are named; the command that persists node edits, validation errors, revisions, and concurrent-edit handling is absent.
30. **Run-now policy incomplete:** authorization, confirmation, risk gates, cancellation, duplicate clicks, and error rendering are not stated.
31. **Engagement decay incomplete:** `N`, minimum sample, open/click identity, offline collection, privacy, decay function, reactivation, and pack-level versus topic-level scope are absent.
32. **Proof relationship deliberately absent:** with `PROOF_SPINE: none`, the summary gives observable effects but no PS metric; originals should clarify what evidence is release-blocking versus product telemetry.
33. **Status evidence is asserted, not verified:** “already,” “shipped,” “TEIKOKU-proven,” and “dark/absent” are plan assertions in a MODERATE-grounding summary, not code-ledger findings.

## Summary-silent feature candidates — do not count as designed features yet

These are required audit questions because the requested categories are otherwise uncovered. The summary neither specifies nor explicitly defers them.

- **Fetch/runtime failures:** retry/backoff, source rate limits, partial downloads, parse failures, offline operation, and provider degradation.
- **Duplicate-precheck behavior:** identity/fingerprint algorithm, false positives, cross-topic dedupe, replay determinism, and user override.
- **Pipeline typing:** stage input/output types, type checking, branching, fan-out, failure propagation, compensation, and partial-result behavior.
- **Task lifecycle:** queued/running/succeeded/failed/cancelled states, timeout, pause/resume, deletion, revisions, retention, and manual retry.
- **Notification lifecycle:** toast/card dismissal, auto-archive policy, read/unread state, dedupe, and failure notifications.
- **Vault operations:** backup/restore, reindex, portability checks, corrupted-frontmatter recovery, case sensitivity, path traversal defense, symlink policy, and storage quotas.
- **Access/security:** manifest editor authorization, TaskSpec ownership, multi-user boundaries, unsafe URL handling, secrets/credentials, and cloud egress approvals.
- **Privacy observability:** evidence that a denied personal-topic flow was blocked, redacted audit projection, and user-visible policy controls.
- **Canvas accessibility/UX:** keyboard editing, screen-reader semantics, zoom/layout persistence, undo/redo, diff/approval before save, and mobile behavior.
- **Operational health:** daemon enable/disable, health checks, metrics, logs, alerting, queue pressure, database migrations, backups, and repair commands.
- **Test matrix:** all four topic kinds, all three triggers, all four sinks, invalid schema/property tests, crash/restart, concurrency, and replay-equivalence tests.
- **Packaging/rollout:** default-off/on choice, upgrade path from designer-only Cosmos manifests, feature flags, compatibility versions, and rollback.

## Deeper-source manifest for later agents

### Tier 1 — explicitly named doctrine and source design (unread in this pass)

1. `docs/thefinalglance/O-plan/O-0_INDEX.md` — explicitly owns doctrine and sequencing (O-SUMMARY:L6).
2. `workflow/reasoning/elai-personal-research-machine-2026-07-06.md` — explicitly named source design (O-SUMMARY:L6-L7).

### Tier 2 — canonical original O unit plans discovered by filename (unread in this pass)

1. `docs/thefinalglance/O-plan/O-1_research_daemon_generalized_cosmos.md`
2. `docs/thefinalglance/O-plan/O-2_scheduled_taskspec_substrate.md`
3. `docs/thefinalglance/O-plan/O-3_brain_vault_contract_mounting.md`
4. `docs/thefinalglance/O-plan/O-4_canvas_workflow_mode.md`

### Tier 3 — explicitly referenced ELAI plans/subsystems to reconcile

| Reference in summary | Candidate source | Why inspect |
|---|---|---|
| G-1 taint | `docs/thefinalglance/G-plan/G-1_taint_propagation_dataplane.md` | Per-byte taint representation and sink admission |
| F-3 tool filter | `docs/thefinalglance/F-plan/F-3_camel_toolfilter_dataplane.md` | Untrusted-content filtering boundary |
| G-2 VCR/replay | `docs/thefinalglance/G-plan/G-2_reproducible_run_artifact.md` | Exact capture/replay contract |
| 3F rules-as-data | MASTER phase index and Phase 3F originals, exact file unresolved here | TaskSpec persistence/lifecycle reuse |
| C-9 task runner | `docs/thefinalglance/C-plan/C-9_autonomous_infrastructure.md` | Tick ownership and dispatch reuse |
| K-14 time doctrine/temporal monitor | `docs/thefinalglance/K-plan/K-14_time_awareness_determinism_doctrine.md` | Supervisor clock and missed-run fold |
| E-5 memory | `docs/thefinalglance/E-plan/E-5_user_modeling_and_memory.md` | Machine-grain boundary/no-dual-write |
| D-5 life-layer denylist | **Unresolved reference namespace**; canonical ELAI `D-plan/D-5_bai_thompson_promotion.md` title does not match | Determine whether D-5 is a source-design/TEIKOKU internal ID |
| H-12 mode bar | `docs/thefinalglance/H-plan/H-12_plan_research_modes_chrome.md` | Mode extension and canvas chrome contract |
| J-31 doctrine | `docs/thefinalglance/J-plan/J-31_canvas_system_mode.md` | Bound-unit/verified-edge/FSM-only semantics |
| J-60 overlay | `docs/thefinalglance/J-plan/J-60_layered_observability_canvas.md` | Last-run state projection/chrome |

### Tier 4 — named implementation and external-contract candidates

- Canonical working-tree candidate for shipped Cosmos behavior: `.elai_cc/crates/knowledge/src/cosmos_worker.rs`.
- Canonical working-tree candidate for its tick proof: `.elai_cc/crates/knowledge/tests/cosmos_worker_tick.rs`.
- Locate exact definitions/usages for `CosmosStage`, briefing compiler, `FlashResearch`, `CosmosDrawer`, `task_runner`, intervention cards, SSB temporal monitor, `list_task_specs`, and `get_task_spec_runstate` before claiming they are wired.
- Locate the TEIKOKU `brain_store` contract that defines `write_page` and `upsert_page`; the summary supplies no exact path.
- Locate frozen canary fixtures, `unsorted`, steal sinks, engagement signals, and the `For-You` surface; the summary supplies no exact paths or schemas.
- Treat duplicate conductor/worktree copies as non-canonical until repository ownership is established; do not infer shipment from filename presence.

## Handoff conclusion

The summary specifies a coherent four-part feature family: a generalized gated research daemon, a typed/journaled scheduler, a namespace-safe mounted user vault, and a receipt-bearing canvas editor that never becomes a runtime. Its smallest explicit behaviors are captured above, while schema gaps, naming collisions, error/lifecycle omissions, and asserted-versus-verified status are deliberately left visible for the original-plan agents.
