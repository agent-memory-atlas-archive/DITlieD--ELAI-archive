# DEEP-P-002 exhaustive feature discovery

## Scope, integrity, and method

- Processed manifest: `docs/thefinalglance/feature-discovery/DEEP-SOURCE-MANIFEST.md:L1268-L1283`, heading `### DEEP-P-002`; the next chunk starts at L1286.
- Declared chunk: family P, **10 files**, **918 logical lines**, **122,607 bytes**, all ten classified `numbered/index original`; target status **under 1,800**.
- Mechanical verification: fresh `sha256sum`, `wc -l`, and byte-size checks matched every manifest row and both aggregates exactly. Missing paths: **0**; unexpected paths: **0**; duplicate paths: **0**; SHA mismatches: **0**; line mismatches: **0**; byte mismatches: **0**.
- Read boundary: all ten listed originals were read completely, first through last logical line. No original assigned to another deep chunk was opened. `16-P-SUMMARY_INDEX.discovery.md` was consulted only as an orientation and delta-comparison artifact; no source atom below is derived solely from it.
- Anchor notation refers to the manifest-verified source images: `P-1:Lx-Ly`, `P-2:Lx-Ly`, `P-6:Lx-Ly`, `P-7:Lx-Ly`, `P-9:Lx-Ly`, `P-10:Lx-Ly`, `P-11:Lx-Ly`, `P-14:Lx-Ly`, `P-19:Lx-Ly`, and `P-24:Lx-Ly`.
- Repeated premise, revised-scope, wiring, grounding, composition, and acceptance language is collapsed into one atom with every load-bearing anchor. Interaction, ambiguity/conflict, and summary-delta records reference atoms and are not counted again as features.
- Classification: `E` = explicit source fact/requirement; `I` = bounded inference; `A` = unresolved ambiguity/choice; `C` = internal conflict or summary correction that must not be silently flattened. Domains: `STATUS`, `CAP`, `SURFACE`, `FLOW`, `STATE`, `DATA`, `CFG`, `INT`, `EVENT`, `SEC`, `PRIV`, `COMP`, `FAIL`, `RECOVERY`, `RETRY`, `DET`, `IDEM`, `CONC`, `OBS`, `TEST`, `LIMIT`, `DEP`, `DEFER`, `EVIDENCE`.

### Exact processed manifest and verification

| Manifest class | Lines | Bytes | SHA-256 | Exact absolute path | Verification |
|---|---:|---:|---|---|---|
| numbered/index original | 95 | 11,822 | d37e4f9bb060c26c2da9ff6aeeaaee089c49127c7dc428026b89b6aaeb279c2f | docs/thefinalglance/P-plan/P-1_app_fleet_layer.md | exact |
| numbered/index original | 93 | 11,112 | a4b1e071309b819073fad0e62f42af5569a368b099cb59d7a1192dc863844ec1 | docs/thefinalglance/P-plan/P-2_inter_tool_ports_world_bus.md | exact |
| numbered/index original | 94 | 10,983 | bd5fe085984c05614ac2943214f12081317ec24613c84620124ba3e04d1ba99a | docs/thefinalglance/P-plan/P-6_life_data_privacy_lattice.md | exact |
| numbered/index original | 90 | 11,089 | 5506a5ca7700859d191ae3e25db952baeb5a3410ed6817f9dcf9e89c19057e02 | docs/thefinalglance/P-plan/P-7_per_user_operating_profile.md | exact |
| numbered/index original | 101 | 14,764 | 3d47ec75ebbaec3d718f69b6426a88579ded6b0ca3b098b4f4b1a0d5c9042530 | docs/thefinalglance/P-plan/P-9_connector_pack_admission_format.md | exact |
| numbered/index original | 107 | 14,909 | b924052721b5b738abe57794f68033662225b536d7f67edde7557c795d26bba1 | docs/thefinalglance/P-plan/P-10_ingress_adapters_credential_vault.md | exact |
| numbered/index original | 100 | 13,218 | d3818f462fb22db5733e2685f76fc5e6fba00357616408f942e881bf45c17824 | docs/thefinalglance/P-plan/P-11_workflow_from_goal.md | exact |
| numbered/index original | 77 | 9,717 | 810344081dedc569caba34343b1c7aa46c6044607cbc682b8e06edaa1dc139af | docs/thefinalglance/P-plan/P-14_explore_then_crystallize_lane.md | exact |
| numbered/index original | 78 | 9,147 | 06567a35e138a1ac7c86497887476bac72a0f88cc03d2cae6ea312caff70e4d1 | docs/thefinalglance/P-plan/P-19_config_ops_lifecycle_doctor.md | exact |
| numbered/index original | 83 | 15,846 | 7981ce57799a720f8a425dc0c1aa82332d840b9c7d4b019a9de6978045f85f3a | docs/thefinalglance/P-plan/P-24_mobile_design_test_lane.md | exact |
| **Aggregate** | **918** | **122,607** | — | **10 exact paths** | **exact** |

## Per-file atomic ledgers

### P-1 — app-fleet layer (health-tick + gated auto-repair) [KEYSTONE]

Source: `docs/thefinalglance/P-plan/P-1_app_fleet_layer.md` (95/95 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| P1-E001 | E/STATUS | P-1 is PLANNED WAVE 1 KEYSTONE, product-surface/distribution class with no PROOF_SPINE, owner-acked 2026-07-07, BUILD-on-shipped, Track PERSONAL-INCARNATION, lever `fleet-grain-over-taskspec`, GAP-1 / T2 keep-it-alive, MODERATE index-derived PLAN-asserted grounding, ~80% composition, effort M-L. | P-1:L1-L5 |
| P1-E002 | E/CAP | The app is the unit of care one grain above changeset/task: each of the user's N bespoke apps becomes a first-class object that is journaled, scheduled, and receipted. | P-1:L6-L7 |
| P1-E003 | E/DATA | `AppManifest` closed field set is `{ app_id, one_sentence_purpose, data_namespaces: [ns], ports (P-2 typed input/output schemas), smoke_trigger, rra_lineage, dep_manifest }`. | P-1:L9-L10, L31-L32 |
| P1-E004 | E/DATA | `FleetRegistry` is a SQLite rows-as-data store (like rules engine / O-2 TaskSpec) enumerating every ELAI-built app. | P-1:L10-L10, L22-L22, L31-L32 |
| P1-E005 | E/FLOW | A health tick is an O-2 TaskSpec with `trigger=Cron`, `pipeline=[Smoke, DriftCanary, DepRotCheck]`, `sink=DashboardInbox` — not a second scheduler. | P-1:L10-L11, L14-L14, L63-L64 |
| P1-E006 | E/DEP | Per-app drift canaries GENERALIZE G-10 from ELAI's own canary set to each app's declared smoke set (the model-update-breaks-use-case gap). | P-1:L10-L11, L15-L15 |
| P1-E007 | E/DEP | Dependency-rot watching points F-9's existence/age gate at each app's `dep_manifest` on a TTL; F-5 is staleness quarantine also pointed at a manifest. | P-1:L11-L11, L16-L16 |
| P1-E008 | E/SEC | When a tick goes red, ELAI proposes a fix that MUST pass the same wiring/claim gates as any changeset (never a silent patch), rendered human-first via N-7, one-click apply — gated AUTO-REPAIR. | P-1:L11-L11, L18-L18, L49-L50, L64-L65 |
| P1-E009 | E/LIMIT | EXCLUDED: a second scheduler; auto-repair that bypasses wiring/claim gates (A-repair-must-gate); a per-app code fork of the health machinery (manifest is DATA, A-5 two-masters); silent rollback (every repair/rollback is journaled + N-7-rendered). | P-1:L11-L11, L63-L67 |
| P1-E010 | E/DEP | L-14 `elai build "<idea>"` emits an RRA receipt; the manifest's `rra_lineage` is that receipt's id; the fleet registers what L-14 builds. | P-1:L17-L17 |
| P1-E011 | E/DEP | M-4's signed-updater manifest/rollback pattern is the shape; M-4 updates ELAI itself while P-1 applies the pattern per-user-app (GAP-1's distinction). | P-1:L19-L19 |
| P1-E012 | E/CFG | SHIP NOW (Tier-1): AppManifest schema + FleetRegistry SQLite + `elai fleet register <app>` CLI + health-tick TaskSpec template with Smoke + DepRotCheck stages (reusing O-2 + F-9) + red-tick DashboardInbox card via N-7. | P-1:L21-L22 |
| P1-E013 | E/DEFER | DEFER (Tier-2): per-app drift canaries (needs G-10 generalized — asserted-not-designed per A-6) + gated auto-repair apply-button (changeset/claim-gate seam). | P-1:L22-L23, L41-L42 |
| P1-E014 | E/INT | Integration: crates/orchestrator (or rules-engine) owns AppManifest table + FleetRegistry loader + health-tick TaskSpec factory + red-tick SSB emit. | P-1:L25-L26 |
| P1-E015 | E/SURFACE | CLI surface: `elai fleet register\|list\|tick <app>` subcommand (cli/main.rs) is the Tier-1 production caller. | P-1:L27-L27, L54-L55, L59-L60, L73-L73 |
| P1-E016 | E/SURFACE | Frontend: DashboardInbox (O-2 intervention-card sink) renders the red-tick card; a dedicated FleetPanel is deferred to P-5's operator surface. | P-1:L28-L28 |
| P1-E017 | E/SEC | M1: strict deserializer validates `data_namespaces` against O-3 vault namespace-claims; a manifest naming an unclaimed namespace is a hard refusal. | P-1:L31-L33, L58-L58 |
| P1-E018 | E/EVENT | M2: each tick journals a `fleet_health_tick` SSB row carrying `{app_id, verdict, red_reasons}`; broken app lands `verdict=red` with populated `red_reasons`. | P-1:L34-L36, L67-L67, L69-L71 |
| P1-E019 | E/IDEM | At the due tick each app's smoke fires exactly once; one `fleet_health_tick` row per due tick. | P-1:L34-L36, L69-L70 |
| P1-E020 | E/SURFACE | M3: red tick renders one N-7 DashboardInbox card (plain-language diagnosis: which app, what broke, one sentence); green tick auto-archives and leaves only the SSB row, no card. | P-1:L37-L39, L68-L71 |
| P1-E021 | E/FAIL | ON_FAIL arms: unclaimed-namespace register succeeds (privacy bypass); tick without SSB row (R1 missing evidence); green ticks spam inbox (missing auto-archive-on-green keyed off `verdict`). | P-1:L33-L33, L36-L36, L39-L39 |
| P1-E022 | E/CFG | FORK: if G-10 canary abstraction is NOT extractable per-app without restructuring, ship Tier-1 Smoke+DepRotCheck only, route drift canary to Tier-2, record G-10-generalization seam; do NOT re-implement a second canary engine. | P-1:L41-L42 |
| P1-E023 | E/LIMIT | ABORT: if gated auto-repair would apply without wiring_contract_check / claim gates, stop and flag; ship diagnosis + propose-fix render only; apply-button lands only once it routes through the real gate seam. | P-1:L49-L50 |
| P1-E024 | E/INT | Production reachability: `elai fleet register` on a live run AND task_runner interval fires a due health-tick TaskSpec; a `#[cfg(test)]` caller NEVER satisfies R7. | P-1:L59-L62, L73-L73 |
| P1-E025 | E/TEST | VERIFY: wiring_contract_check + production_caller_check; pass = `fleet_health_tick` row landed off-agent from real task_runner tick (or `elai fleet tick`), fired exactly once, red-tick inbox card for broken-app, auto-archive for green. | P-1:L75-L79 |
| P1-E026 | E/TEST | INTEGRATION_SMOKE R10: register TWO apps (healthy + broken after simulated dep-rot), advance several ticks, assert single-fire journaling per app + red N-7 card + green auto-archive + no cross-app leakage of red_reasons. | P-1:L81-L81 |
| P1-E027 | E/SEC | REDTEAM failed attack: silent red-tick fix is blocked by stub_ban + ABORT (repair is a changeset). Succeeded attack: register 200 apps storming task_runner — patched by ONE batched TaskSpec iterating the registry (one row per app, one dispatch per tick). | P-1:L83-L84 |
| P1-E028 | E/EVIDENCE | Grounding: design digest GAP-1/T2/A-2 (falsifier: fleet-health green %, median time-to-repair) + A-6 G-10 recon; reddit bespoke-tools thread; Ink & Switch malleable-software essay. | P-1:L86-L90 |
| P1-E029 | E/DEP | COMPOSES-WITH: O-2 TaskSpec + DashboardInbox (EXTEND), G-10 (generalize), F-9/F-5, L-14 RRA, N-7, O-3 namespace-claims. Re-implements NONE (GD-1). P-2 supplies typed ports; P-5 renders fleet operator grain; P-6 validates namespaces against privacy lattice. | P-1:L92-L93 |
| P1-A001 | A/DEP | Whether G-10's canary set is parameterizable by an app's smoke set or hard-bound to ELAI fixtures is RECON_NEEDED before Tier-1 drift-canary scope. | P-1:L44-L45 |
| P1-A002 | A/INT | O-2 TaskSpec factory API (programmatic create vs row-authored) is RECON_NEEDED. | P-1:L46-L46 |
| P1-A003 | A/INT | F-9 dep existence/age checker entrypoint (manifest path vs Cargo.lock) is RECON_NEEDED. | P-1:L47-L47 |
| P1-A004 | A/CFG | Exact TTL for F-9 dep_manifest watching and Cron schedule defaults for the health-tick template are not specified. | P-1:L10-L11, L21-L22 |
| P1-C001 | C/DEFER | Opening steal/build names DriftCanary in the pipeline, but REVISED Tier-1 ships only Smoke + DepRotCheck; drift canary is Tier-2 behind G-10 generalization. | P-1:L10-L11, L21-L23, L41-L42 |
| P1-C002 | C/DEFER | Opening prose includes one-click gated auto-repair, but Tier-1 ships diagnosis + propose-fix only; apply-button is Tier-2 + ABORT-gated. | P-1:L11-L11, L21-L23, L49-L50 |

Category coverage: capabilities/CLI/inbox surfaces ✓; workflows/states ✓; schemas/data ✓; flags/defaults ✓ (Cron template partial); integrations/events ✓; security/namespace/gates ✓; failures/recovery ✓; determinism/idempotency ✓; observability/evidence ✓; tests/done gates ✓; constraints/non-goals ✓; dependencies/status/deferred ✓. Formal retry API absent; single-fire-per-tick is the idempotency invariant.

### P-2 — inter-tool ports + world-grain event bus

Source: `docs/thefinalglance/P-plan/P-2_inter_tool_ports_world_bus.md` (93/93 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| P2-E001 | E/STATUS | P-2 is PLANNED WAVE 2, product-surface, no PROOF_SPINE, owner-acked 2026-07-07, BUILD-on-shipped mostly composition; port-schema registry is the genuine build; Track PERSONAL-INCARNATION; lever `typed-ports-over-canvas`; GAP-2 / T3 glue; MODERATE grounding; effort M; depends P-1 + O-4. | P-2:L1-L5 |
| P2-E002 | E/CAP | Every ELAI-built tool ships typed PORTS in its P-1 manifest; canvas wires tool->tool edges carrying N-6 provenance tier + G-1 taint; the bus is typed-edge projection + journaled cross-tool invocation SSB rows, not a runtime. | P-2:L6-L10 |
| P2-E003 | E/DATA | Extend P-1 AppManifest with `ports: { inputs: [PortSchema], outputs: [PortSchema] }` where `PortSchema = { name, json_schema, domain_label (life\|work\|shared, P-6) }`. | P-2:L10-L10, L30-L31 |
| P2-E004 | E/DATA | `PortRegistry` indexes every app's ports for wiring. | P-2:L10-L10, L30-L31, L52-L53 |
| P2-E005 | E/FLOW | On O-4 canvas, a tool->tool EDGE binds an output port to a compatible input port; edge carries N-6 provenance tier of the source + G-1 taint of flowing bytes. | P-2:L10-L10, L33-L35 |
| P2-E006 | E/SURFACE | An edge whose source port has NO consumer renders VISIBLY DISCONNECTED (O-4 wire-dark-made-visual at world grain). | P-2:L10-L10, L36-L38 |
| P2-E007 | E/SEC | elai-verify (M-5) conformance to PortSchema makes a third-party tool a mountable node (Tier-2, with L-12 admission). | P-2:L10-L10, L21-L22, L82-L83 |
| P2-E008 | E/LIMIT | EXCLUDED: live message-broker/runtime bus (A-3; dispatch is the FSM); second provenance scale (ride N-6 — A-1); untyped/free-text port (closed json_schema, `deny_unknown`); edge carrying bytes WITHOUT source taint tier. | P-2:L11-L11, L48-L49, L62-L66 |
| P2-E009 | E/CFG | SHIP NOW (Tier-1): PortSchema extension + PortRegistry + static port-compatibility check (`elai fleet wire <appA.out> <appB.in>`) refusing incompatible schemas + journaled `tool_port_edge` SSB row + G-1 taint tier on the edge record. | P-2:L20-L21 |
| P2-E010 | E/DEFER | DEFER (Tier-2): O-4 canvas WORKFLOW-mode rendering of tool->tool edges + third-party mount via M-5 elai-verify (L-12 admission seam shared with P-3). | P-2:L21-L22, L27-L27 |
| P2-E011 | E/SURFACE | CLI: `elai fleet wire\|ports <app>` is the Tier-1 production caller; Tier-1 exposes zero-consumer outputs via `elai fleet ports --unwired`. | P-2:L26-L26, L36-L38, L53-L54, L68-L70 |
| P2-E012 | E/SURFACE | frontend_pair Tier-2: dashboard-v2 O-4 WORKFLOW view renders tool->tool edges with N-6 tier chrome + disconnected-source affordance. | P-2:L27-L27, L59-L60 |
| P2-E013 | E/FAIL | Malformed port schema is hard parse-refusal; untyped port acceptance is ON_FAIL (interchange-format invariant). | P-2:L30-L32 |
| P2-E014 | E/EVENT | Successful bind emits `tool_port_edge` SSB row `{src_app.out, dst_app.in, n6_tier, taint_tier}`; schema-incompatible wire lands NO edge row and returns a refusal. | P-2:L33-L35, L67-L70 |
| P2-E015 | E/SEC | Taint-tier propagation: source port's G-1 taint tier is forced onto every edge record; dropped tier is a smoke failure. | P-2:L33-L35, L65-L66, L82-L83 |
| P2-E016 | E/CFG | FORK: if O-4 edge model is hard-bound to research-pipeline stage enum and cannot carry tool->tool edges without restructuring, ship Tier-1 CLI fully and route ALL canvas rendering to Tier-2; trust value is realized at CLI grain without canvas. | P-2:L40-L41 |
| P2-E017 | E/LIMIT | ABORT: if wiring the world-bus would require a LIVE message-broker executing edges outside the FSM, stop; bus is projection + journaled rows, never a runtime. | P-2:L48-L49 |
| P2-E018 | E/INT | Production reachability: `elai fleet wire` binds on a live run; Tier-2 O-4 WORKFLOW canvas renders/creates the edge; test callers never satisfy R7. | P-2:L58-L61, L72-L72 |
| P2-E019 | E/TEST | VERIFY: wiring_contract_check + production_caller_check; pass = `tool_port_edge` row from real `elai fleet wire` carrying N-6 + G-1 taint, incompatible wire refused with no edge row. | P-2:L74-L78 |
| P2-E020 | E/TEST | INTEGRATION_SMOKE R10: three apps (weather source, wardrobe consumer, budget unrelated); wire weather->wardrobe; refuse weather->budget; assert taint tier + `--unwired` unrelated output + G-2 replay of wardrobe reproduces weather bytes. | P-2:L80-L80 |
| P2-E021 | E/SEC | REDTEAM failed: launder untrusted weather bytes by dropping taint tier — stub_ban forces propagation. Succeeded attack: malicious third-party tool declaring conformant schema — patched by Tier-2 M-5 elai-verify + L-12 admission (schema-declaration alone never admits). | P-2:L82-L83 |
| P2-E022 | E/EVIDENCE | Grounding: design digest GAP-2/T3/A-1/A-2 (falsifier: wired-edge count, disconnected-output rate); reddit glue quotes; feature-map O-4, N-6, G-1/G-2, M-5/M-6. | P-2:L85-L88 |
| P2-E023 | E/DEP | COMPOSES-WITH: P-1 AppManifest (EXTEND ports), O-4 WORKFLOW (EXTEND edges), N-6, G-1/G-2, M-5/M-6. P-6 supplies `domain_label`; P-3 export bundles ports. | P-2:L90-L91 |
| P2-A001 | A/INT | O-4 edge data model (typed stage->stage vs generic node pair) is RECON_NEEDED. | P-2:L43-L44 |
| P2-A002 | A/INT | G-1 taint API for tagging/reading a byte-stream tier is RECON_NEEDED. | P-2:L45-L45 |
| P2-A003 | A/INT | Whether N-6 provenance tier attaches to arbitrary records or only claim objects is RECON_NEEDED. | P-2:L46-L46 |
| P2-A004 | A/DATA | Compatibility relation details and schema-version behavior for PortSchema are not fully specified beyond refuse-on-mismatch. | P-2:L20-L21, L33-L35 |
| P2-A005 | A/EVENT | Source does not require a separate refusal/audit SSB row for incompatible wire; only "NO edge row + refusal return". | P-2:L67-L70 |
| P2-C001 | C/DEFER | Steal/build names O-4 canvas edge rendering and third-party mount as core, but REVISED scope makes both Tier-2; Tier-1 is CLI registry/wire/SSB only. | P-2:L10-L10, L20-L22 |

Category coverage: capabilities/CLI/canvas surfaces ✓; workflows ✓; schemas/ports ✓; flags/config ✓; integrations/events ✓; security/taint/admission ✓; failures ✓; determinism/replay ✓; observability ✓; tests ✓; constraints ✓; deps/deferred ✓. No formal retry; no standalone privacy rule beyond domain_label.

### P-6 — life-data privacy lattice

Source: `docs/thefinalglance/P-plan/P-6_life_data_privacy_lattice.md` (94/94 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| P6-E001 | E/STATUS | P-6 is PLANNED WAVE 2, product-surface, no PROOF_SPINE, owner-acked 2026-07-07, BUILD-on-shipped extending N-6 with a privacy dimension (no second lattice); Track PERSONAL-INCARNATION; lever `out-flow-privacy-dimension`; GAP-6 / T6; MODERATE grounding; composition; effort M; depends N-6 + O-3. | P-6:L1-L5 |
| P6-E002 | E/PRIV | Third privacy axis: WHICH TOOL may read WHICH NAMESPACE; domain label life/work/shared + per-tool read grants; G-1 taint's MIRROR IMAGE (owned bytes flowing OUT toward models/tools vs untrusted bytes flowing IN). | P-6:L6-L10 |
| P6-E003 | E/DATA | Extend N-6 provenance-lattice mapping table with `domain: life\|work\|shared` per namespace and `read_grants: [{tool_id, namespace, granted_at, journal_ref}]`. | P-6:L9-L10, L30-L31 |
| P6-E004 | E/SEC | Enforcement at TWO boundaries: (a) O-3 vault read path — ungranted tool refused; (b) E-20 routing — owned byte's domain gates model/tool prompt entry. | P-6:L10-L10, L20-L22 |
| P6-E005 | E/PRIV | Default is deny (no grant = no read); a grant is an explicit, journaled, revertible act (H-49). | P-6:L10-L11, L62-L65 |
| P6-E006 | E/LIMIT | EXCLUDED: SECOND privacy lattice (A-1/N-2 authoring FAIL); implicit/default-allow read; grant that isn't journaled+revertible; mixing OUT-flow privacy dimension with IN-flow taint into one confused field. | P-6:L11-L11, L48-L49, L62-L66 |
| P6-E007 | E/CFG | SHIP NOW (Tier-1): domain dimension on N-6 mapping + read_grants table + O-3 vault-read enforcement + journaled `privacy_grant` / `privacy_read_refusal` SSB rows + N-7 render of each grant. | P-6:L20-L21 |
| P6-E008 | E/DEFER | DEFER (Tier-2): E-20 routing-boundary enforcement (lands with P-4 living-brain routing wall) + grant-management operator surface (frontend with P-5). | P-6:L21-L22, L27-L27 |
| P6-E009 | E/INT | Integration: crates/knowledge (or N-6 home) owns domain + read_grants + vault-read enforcement hook + SSB emit. | P-6:L24-L25 |
| P6-E010 | E/SURFACE | CLI: `elai privacy grant\|revoke\|show <tool> <namespace>` is the Tier-1 production caller. | P-6:L26-L26, L54-L55, L58-L59 |
| P6-E011 | E/EVENT | Granting emits `privacy_grant` row + N-7 human-rendered line; ungranted vault read lands `privacy_read_refusal` row. | P-6:L33-L38, L67-L70 |
| P6-E012 | E/RECOVERY | `elai privacy revoke` is the inverse journaled op; tool loses read access after revoke. | P-6:L36-L38 |
| P6-E013 | E/CFG | FORK: if N-6 mapping cannot take a new dimension without N-4 claim-object schema migration, ship Tier-1 as a JOIN table keyed on N-6 namespace id (still one lattice / N-2-clean), record the seam. | P-6:L40-L41 |
| P6-E014 | E/LIMIT | ABORT: if adding privacy requires minting a SECOND provenance/privacy lattice, stop and flag; corpus has exactly one lattice by doctrine. | P-6:L48-L49 |
| P6-E015 | E/INT | Production reachability: `elai privacy grant\|revoke` on live run AND O-3 vault-read path enforces grants on live read; test callers never satisfy R7. | P-6:L58-L61, L72-L72 |
| P6-E016 | E/TEST | VERIFY: wiring_contract_check + production_caller_check + corpus_composition_check (N-2 must pass — no second lattice); pass = ungranted vault read refused with `privacy_read_refusal`, granted read succeeded, grant journaled+revertible, N-2 confirms single lattice. | P-6:L74-L79 |
| P6-E017 | E/TEST | INTEGRATION_SMOKE R10: label three namespaces (budget=life, project=work, notes=shared), grant one tool on one namespace, cross-namespace multi-tool reads + revoke mid-run, assert every ungranted read refuses + grant/revoke journal + no owned byte to ungranted tool. | P-6:L81-L81 |
| P6-E018 | E/SEC | REDTEAM failed: relabel health namespace shared to read from budget tool — relabel is journaled N-6 change and still needs explicit grant. Succeeded attack: second privacy scale as "helper table" — patched by N-2 corpus_composition_check at authoring. | P-6:L83-L84 |
| P6-E019 | E/EVIDENCE | Grounding: design digest GAP-6/T6/A-1; reddit Azko87/Upbeat-Armadillo quotes; feature-map N-6/N-2/G-1/G-9/E-20/O-3/N-7/H-49. | P-6:L86-L89 |
| P6-E020 | E/DEP | COMPOSES-WITH: N-6 (EXTEND privacy dimension), N-2, G-1 (mirror), G-9, E-20, O-3, N-7, H-49. O-3 owns vault boundary; P-4 is same E-20 enforcement point (Tier-2 shared). | P-6:L91-L92 |
| P6-A001 | A/INT | N-6 mapping-table schema and whether a new dimension touches N-4 claim objects is RECON_NEEDED. | P-6:L43-L44 |
| P6-A002 | A/INT | O-3 vault-read entrypoint for grant-check insertion is RECON_NEEDED. | P-6:L45-L45 |
| P6-A003 | A/INT | E-20 routing boundary for domain-gated prompt entry (Tier-2) is RECON_NEEDED. | P-6:L46-L46 |
| P6-A004 | A/DATA | Grant scope beyond the example (expiry, nested namespaces, cross-domain precedence) is not specified. | P-6:L9-L10, L33-L35 |
| P6-C001 | C/DEFER | Steal/build names dual enforcement at O-3 AND E-20, but REVISED Tier-1 ships only O-3 vault-read; E-20 is Tier-2. | P-6:L10-L10, L20-L22 |

Category coverage: privacy/security ✓; CLI surfaces ✓; workflows/grants ✓; schemas ✓; config/defaults ✓; integrations/events ✓; failures/recovery ✓; observability ✓; tests ✓; constraints ✓; deps/deferred ✓. No concurrency/retry API.

### P-7 — per-user operating profile

Source: `docs/thefinalglance/P-plan/P-7_per_user_operating_profile.md` (90/90 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| P7-E001 | E/STATUS | P-7 is PLANNED WAVE 3, product-surface, no PROOF_SPINE, owner-acked 2026-07-07, BUILD-on-shipped riding anti-selection-bias promotion; Track PERSONAL-INCARNATION; lever `measured-asymmetric-process-adaptation`; GAP-7; MODERATE grounding; composition on N-3/H-17/H-18; effort M-L; depends N-3 landed. | P-7:L1-L5 |
| P7-E002 | E/CAP | Process adaptation (autonomy, gate strictness, explanation verbosity per DOMAIN per user) is disjoint from E-5/E-4 CONTENT adaptation (voice/terseness). | P-7:L6-L7, L11-L11, L17-L17, L63-L64 |
| P7-E003 | E/DATA | `OperatingProfile { per_domain: { autonomy, gate_strictness, explanation_verbosity } }` learned from journaled outcomes and promoted through H-17 trial ledger + H-18 deflated significance (what N-3 generalizes). | P-7:L9-L10, L20-L21 |
| P7-E004 | E/SEC | ASYMMETRY invariant (M-2 at process grain): profile may TIGHTEN gates freely; may LOOSEN ONLY against measured false-positive evidence from THAT user's own runs — defense against adaptation-as-sycophancy (A-4). | P-7:L10-L10, L15-L15, L29-L32, L49-L49, L60-L61 |
| P7-E005 | E/LIMIT | EXCLUDED: loosening driven by anything but measured own-run FP evidence; unmeasured/vibe promotion; profile change that isn't observe-first + journaled + revertible; adapting CONTENT (E-5's job). | P-7:L11-L11, L60-L64 |
| P7-E006 | E/CFG | SHIP NOW (Tier-1): OperatingProfile type + three default profiles (toy/connected/team) + ASYMMETRY enforcement + journaled `operating_profile_change` SSB row + observe-first + revert. | P-7:L20-L21 |
| P7-E007 | E/DEFER | DEFER (Tier-2): automatic per-domain learning loop from journaled outcomes (full N-3 pipeline once N-3 live and FP-rate falsifier measured) + per-domain granularity beyond the three defaults. | P-7:L21-L22, L40-L41 |
| P7-E008 | E/CFG | Three defaults on ONE engine: toy = P-5 ceremony floor; connected = P-2 world bus on; team = M org pack + P-8. | P-7:L6-L7, L36-L38 |
| P7-E009 | E/SURFACE | CLI: `elai profile show\|tighten\|loosen <domain>` is the Tier-1 production caller; loosen refuses without evidence. | P-7:L26-L26, L53-L54, L57-L58, L70-L70 |
| P7-E010 | E/EVENT | Profile change emits `operating_profile_change {domain, field, old, new, evidence_ref}`; tighten applies + journals; loosen without evidence is REFUSED (no change row); loosen with H-18-significant own-run FP evidence promotes + journals with evidence_ref. | P-7:L31-L35, L65-L68 |
| P7-E011 | E/FLOW | Every profile change is observe-first (GD-4), journaled, and revertible (H-49 applies to configuration); change runs observe-only first where it defaults a gate on. | P-7:L10-L10, L16-L16, L33-L35 |
| P7-E012 | E/CFG | FORK: if N-3 promotion pipeline is NOT yet live, ship Tier-1 with asymmetry-guarded MANUAL change function (tighten free, loosen refused without explicit own-run FP evidence) and route AUTOMATIC learning to Tier-2; asymmetry safety is fully realized manually. | P-7:L40-L41 |
| P7-E013 | E/LIMIT | ABORT: if loosen path can be satisfied WITHOUT measured own-run FP evidence (user assertion, global default, another user's data), stop and flag; that is the A-4 attack surface. | P-7:L48-L49 |
| P7-E014 | E/INT | Production reachability: `elai profile tighten\|loosen` CLI on a live run; test callers never satisfy R7. | P-7:L57-L59, L70-L70 |
| P7-E015 | E/TEST | VERIFY: wiring_contract_check + production_caller_check; pass = loosen REFUSED off-agent without measured own-run FP evidence and PROMOTED through H-18 with it, tighten applied freely, every change journaled + revertible. | P-7:L72-L76 |
| P7-E016 | E/TEST | INTEGRATION_SMOKE R10: independent agent tightens, attempts loosen with fabricated/insufficient evidence (must refuse), supplies measured own-run FP evidence and loosens (H-18 promote), reverts, asserts asymmetry + three defaults each mapped to sibling unit posture. | P-7:L78-L78 |
| P7-E017 | E/SEC | REDTEAM failed: talk ELAI into lowering a security gate by acting frustrated — loosening gated on measured FP evidence not affect. Succeeded attack: loosen using another user's or global FP rate — patched by scoping FP evidence to THAT user's own runs only. | P-7:L80-L81 |
| P7-E018 | E/EVIDENCE | Grounding: design digest GAP-7 + A-4 (adaptation-as-sycophancy) + A-2 (falsifier: profile-promotion FP rate); feature-map N-3/H-17/H-18/M-2/GD-4/H-49/E-5/E-4/D-14/N-15. | P-7:L83-L85 |
| P7-E019 | E/DEP | COMPOSES-WITH: N-3/H-17/H-18 (EXTEND to process metrics), M-2 (mirror asymmetry), GD-4/H-49, E-5 (disjoint), P-5/P-2/M/P-8 (defaults' targets). | P-7:L87-L88 |
| P7-A001 | A/CFG | FP metric definitions, promotion thresholds, observation duration, and default numeric values for autonomy/strictness/verbosity are absent. | P-7:L9-L10, L29-L32 |
| P7-A002 | A/DATA | Exact domain vocabulary for `per_domain` is not restated beyond sibling default targets; may rely on P-6/N-6. | P-7:L9-L10, L36-L38 |
| P7-A003 | A/INT | Whether H-17/H-18 can score a PROCESS metric (FP rate) as readily as a content metric is RECON_NEEDED. | P-7:L43-L44 |
| P7-A004 | A/INT | M-2 monotone-tighten enforcement point to mirror at process grain is RECON_NEEDED. | P-7:L45-L45 |
| P7-A005 | A/INT | Where E-5 content adaptation lives (to keep P-7 process-only disjoint) is RECON_NEEDED. | P-7:L46-L46 |
| P7-C001 | C/DEFER | Opening premise frames learned promotion from journaled outcomes as the core, but REVISED Tier-1 is manual asymmetry-guarded change; automatic learning loop is Tier-2 behind N-3 confirmation. | P-7:L6-L10, L20-L22, L40-L41 |

Category coverage: process-adaptation capability ✓; CLI surfaces ✓; states/events ✓; schemas ✓; config/defaults ✓; security/asymmetry ✓; failures/recovery ✓; observability ✓; tests ✓; constraints ✓; deps/deferred ✓. No UI panel in Tier-1.

### P-9 — connector-pack admission format

Source: `docs/thefinalglance/P-plan/P-9_connector_pack_admission_format.md` (101/101 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| P9-E001 | E/STATUS | P-9 is PLANNED product surface executing AFTER the proof spine (A-7 / GATE M-0 / PSR-1); authored 2026-07-08 from command-center eval; PROOF_SPINE none (distribution); PSR-2 ack 2026-07-09; breadth-moat for P's distribution phase. | P-9:L1-L5 |
| P9-E002 | E/CAP | Applies M-6 own-the-format to CAPABILITY: signed, versioned connector/stage-PACK format (declarative where possible + code where necessary) mounted through EXISTING admission family (L-12 + skillspector + J-102 + N-6 scanner registry + taint tiers). Vocabulary stays CLOSED; connector surface becomes UNBOUNDED. | P-9:L6-L7 |
| P9-E003 | E/DATA | ONE generic `http_call` stage kind parameterized by declarative ConnectorSpec (endpoint map, auth kind, pagination, response schema-map -> PortSchema) — DATA (admission-gated), never free-form code; ~80% of connectors are a DATA DIFF of the first pack. | P-9:L9-L10, L40-L46 |
| P9-E004 | E/CAP | Genuinely-new behavior takes LANE B code-pack path via B-37 RuntimeAdapter closed-trait precedent. | P-9:L10-L10, L31-L32, L50-L52 |
| P9-E005 | E/DEP | Substrate: P-2/M-5 mountable-node (tools with ELAI-verifiable evidence only — P-9 generalizes to arbitrary connectors); M-6 golden-lock; L-12/skillspector/J-102/N-6/G-1 admission family; B-37 RuntimeAdapter; O A3 closed stage vocabulary + A-5 data-not-code. | P-9:L12-L17 |
| P9-E006 | E/DATA | MOVE 1: ConnectorSpec is human-frozen SCHEMA_REQUIRED JSON-Schema for `{endpoints map, auth_kind enum, pagination enum, response schema-map -> PortSchema}`; versioned connector-spec.schema.json; malformed/code-in-data field REJECTED. | P-9:L40-L43 |
| P9-E007 | E/FLOW | MOVE 2: `http_call` executes ConnectorSpec (auth -> paginated fetch -> taint Untrusted at ingest (G-1) -> response schema-map -> PortSchema-validated emit). One kind, unbounded endpoints. | P-9:L44-L46 |
| P9-E008 | E/SEC | EGRESS IS PINNED: `http_call` may only connect to hosts in the mounted spec's declared endpoints map (composed with G-3 host-side allowlist/IP-pin); declared-host set is part of SIGNED pack; request to undeclared host REFUSED pre-connect (pure-data exfil patch). | P-9:L44-L46, L67-L67, L87-L88 |
| P9-E009 | E/SEC | MOVE 3: signed versioned pack (spec + PortSchema + N-6 tier + manifest) mounts through L-12/skillspector/J-102; M-6-style golden-lock; well-formed pack mounts (MountedConnector row/event); TAMPERED pack (bad signature / failed skillspector) REFUSES fail-closed (never mount-with-warning). | P-9:L47-L49, L84-L85 |
| P9-E010 | E/SURFACE | MOVE 4: `elai connector new <name>` scaffolds declarative pack template (LANE A); ELAI fills endpoints/auth/schema-map with anti-slopsquat + source-grounding recon; LANE B code pack implements closed trait + mounts through same gate. | P-9:L50-L52, L64-L66 |
| P9-E011 | E/DATA | MOVE 5: decisions doc `workflow/reasoning/P-9-decisions.md` records ConnectorSpec enum choices + API grounding, LANE-A/B boundary, golden-lock bump procedure. | P-9:L53-L55 |
| P9-E012 | E/CFG | FORK-1: if ConnectorSpec covers target API shape -> LANE A (data pack); if genuinely-new transport/behavior -> LANE B (B-37 code pack). Default LANE A. | P-9:L31-L32 |
| P9-E013 | E/CFG | FORK-2: pack signing reuses existing ELAI RRA/.rra signing key if applicable; else scoped key is RECON item. | P-9:L33-L33 |
| P9-E014 | E/LIMIT | ABORT-1: pack format that would OPEN stage vocabulary (generic/custom runtime node / n8n model) -> STOP. | P-9:L35-L36 |
| P9-E015 | E/LIMIT | ABORT-2: declarative ConnectorSpec carrying executable code (eval, script field) -> STOP; code goes through LANE B only. | P-9:L37-L37 |
| P9-E016 | E/LIMIT | ABORT-3: admission needing a NEW ungated path bypassing L-12/skillspector/N-6 -> STOP. | P-9:L38-L38 |
| P9-E017 | E/INT | PRODUCTION_HOOK: `http_call` consumed by O-1 fetch->classify / FSM stage dispatcher when mounted ConnectorSpec runs in a TaskSpec; pack-mount path produces MountedConnector SSB row via L-12/N-6 registry. Exact file+fn seams RECON at execution pre-flight. | P-9:L57-L58 |
| P9-E018 | E/TEST | INTEGRATION_SMOKE R10: mount fixture pack, run TaskSpec firing `http_call` against local mock behind auth+pagination, assert taint-tagged PortSchema-valid SSB row AND tampered pack REFUSES. | P-9:L60-L61 |
| P9-E019 | E/TEST | ACCEPTANCE binary: closed vocab + unbounded endpoints; declarative pack mounts through existing admission; tampered refuses; http_call taints Untrusted at ingest + PortSchema validates; versioned golden-locked enums source-grounded (KD-1); undeclared-host refused pre-connect; OFF-AGENT MountedConnector + fetch rows + INTEGRATION_SMOKE green. | P-9:L83-L89 |
| P9-E020 | E/LIMIT | stub_ban: NO open/custom runtime node; NO code field in declarative spec; NO new ungated admission; NO invented auth/pagination enums; NO mount-with-warning; NO http_call egress outside declared endpoints. | P-9:L67-L67 |
| P9-E021 | E/DEFER | REDTEAM deferred to execution pre-flight (P post-spine); known surfaces: code-in-data injection, admission-bypass, closed-vocab under custom-node pressure, pure-data exfil via undeclared-host URL templates. | P-9:L91-L92 |
| P9-E022 | E/EVIDENCE | ~1000-integration n8n figure is training-unsearched (KD-1) — do not cite externally without verification. Declarative-spec expressiveness ceiling is RECON-1 empirical. | P-9:L94-L96 |
| P9-E023 | E/DEP | Next: P-10 supplies auth substrate for http_call; P-11 is demo converting n8n user; all three execute after proof spine. | P-9:L98-L99 |
| P9-A001 | A/DATA | Minimal ConnectorSpec coverage for HTTP long tail (auth-kind + pagination enums) requires RECON-1 against real n8n node definitions + OpenAPI taxonomy — do NOT invent. | P-9:L26-L27 |
| P9-A002 | A/INT | Whether L-12/skillspector/J-102/N-6 accepts non-skill pack as-is or needs pack-type discriminator is RECON-2. | P-9:L28-L28 |
| P9-A003 | A/DATA | How response schema-map binds to existing PortSchema/threaded-contract system is RECON-3; response map must produce human-frozen PortSchema the generator satisfies. | P-9:L29-L29 |
| P9-A004 | A/SEC | Exact pack signing key reuse path is FORK-2 / RECON. | P-9:L33-L33 |

Category coverage: connector capability ✓; CLI `elai connector new` ✓; schemas/packs ✓; security/admission/egress ✓; failures/fail-closed ✓; tests/acceptance ✓; constraints/aborts ✓; deps/deferred ✓. No standalone UI panel; MountedConnector is journal surface. Summary has no P-9 section.

### P-10 — ingress adapters + credential vault

Source: `docs/thefinalglance/P-plan/P-10_ingress_adapters_credential_vault.md` (107/107 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| P10-E001 | E/STATUS | P-10 is PLANNED product surface post-spine (A-7/GATE M-0/PSR-1); authored 2026-07-08; PROOF_SPINE none; PSR-2 ack 2026-07-09; supplies ingress + credential substrate for P-9 http_call and command-center vision. | P-10:L1-L5 |
| P10-E002 | E/CAP | Do NOT grow O-2 trigger enum per protocol. Add INGRESS ADAPTER family converting any external shape into a taint-tagged (G-1) journaled SsbEvent row so downstream (O-2 SsbEvent trigger, G-2 replay, provenance, privacy lattice) works UNCHANGED. Plus first-class CredentialVault. | P-10:L6-L7 |
| P10-E003 | E/FLOW | Four external shapes normalize at boundary to `{external bytes -> taint Untrusted (G-1) -> SsbEvent row}`: (a) OUTBOUND PULL — O-2 Cron + fetch + vault only; (b) INBOUND PUSH — WebhookIngress on I-1 supervisor plane; (c) STREAMING/PERSISTENT — long-lived listener daemon (C-17 FileWatcherService shape); (d) GENUINELY OTHER — B-37/P-9 pack adapters. | P-10:L9-L14, L37-L38 |
| P10-E004 | E/DEP | Substrate: O-2 Cron\|SsbEvent\|Manual triggers (SsbEvent is join point); G-1/G-2/K-14/G3-G6 async replay; I-1 supervisor plane; C-17 FileWatcher; B-37/P-9 packs; G-3 credential broker + M-1 persona tokens are scatter P-10 consolidates; persona-excluded per P-3 export rules. | P-10:L16-L22 |
| P10-E005 | E/CAP | MOVE 1: IngressAdapter closed trait — one job `{external bytes -> taint Untrusted -> SCHEMA_REQUIRED SsbEvent row}`; same contract all four shapes; O-2 enum UNCHANGED; adapter event indistinguishable-downstream from native. | P-10:L47-L49, L89-L90 |
| P10-E006 | E/SEC | MOVE 2 WebhookIngress: local HTTP listener on I-1; INBOUND AUTH MANDATORY — every registered hook carries per-hook shared secret; inbound posts signature-verified (HMAC-over-body GitHub-webhook idiom; exact scheme RECON) BEFORE any byte is parsed; missing/failed signature REFUSED with journaled refusal row and NO SsbEvent. | P-10:L50-L52, L73-L73, L94-L94 |
| P10-E007 | E/DET | K-14 journals the external event as a replay input so G-2 replay reproduces from journal WITHOUT live re-fetch (faithful async replay). | P-10:L11-L12, L50-L52, L90-L91 |
| P10-E008 | E/SEC | MOVE 3 CredentialVault: first-class credential plane (OS-keychain preferred, encrypted-file fallback); secrets NEVER in SSB payload / RRA / P-3 export; http_call (P-9) and fetch stage draw auth from it via handle resolved at call time. | P-10:L53-L55, L91-L92 |
| P10-E009 | E/CFG | FORK-2 vault backend: prefer OS keychain; file-at-rest fallback with explicit at-rest-encryption RECON; never plaintext. | P-10:L39-L39, L100-L101 |
| P10-E010 | E/CAP | MOVE 4: persistent-listener daemon shape (c) reusing C-17 + shape-d pack path; registry of active adapters on I-1 plane; listeners NOT in pipeline hot path. | P-10:L56-L58, L92-L93 |
| P10-E011 | E/SURFACE | CLI credential acquisition v1: `elai cred add` manual paste; OAuth flows (device-code vs localhost-redirect, refresh rotation) are DECLARED follow-on; if a P-9 connector needs OAuth before follow-on lands, that connector is BLOCKED on it, not hand-rolled inline. | P-10:L35-L35, L73-L73 |
| P10-E012 | E/DATA | MOVE 5: decisions doc `workflow/reasoning/P-10-decisions.md` records four-shape routing, vault backend + at-rest choice, replay-as-input proof. | P-10:L59-L61 |
| P10-E013 | E/LIMIT | ABORT-1: growing O-2 trigger enum per protocol -> STOP. ABORT-2: inbound push not journalable as faithful replay input -> STOP. ABORT-3: credential in cleartext in any SSB/RRA/P-3 export -> STOP. | P-10:L41-L44 |
| P10-E014 | E/INT | PRODUCTION_HOOK: IngressAdapter/WebhookIngress/FileWatch -> O-2 SsbEvent trigger matcher; CredentialVault -> fetch / P-9 http_call auth path. Seams RECON at pre-flight. | P-10:L63-L64 |
| P10-E015 | E/TEST | INTEGRATION_SMOKE R10: POST to live WebhookIngress -> taint-tagged SsbEvent + O-2 TaskSpec fires; G-2-replay reproduces from journal (no re-fetch); http_call draws vault credential; grep journal/RRA/export for secret == 0 hits. | P-10:L66-L67 |
| P10-E016 | E/TEST | ACCEPTANCE: O-2 enum unchanged; all four shapes normalize to taint-tagged SsbEvent; faithful journal replay; CredentialVault usable + zero leak; listeners on I-1 gated by registry; unsigned post refused before parse; OFF-AGENT proof artifacts. | P-10:L89-L95 |
| P10-E017 | E/LIMIT | stub_ban: NO new O-2 trigger kind; NO live-re-fetch-on-replay; NO secret in journal/RRA/export; NO listener in hot path; NO plaintext at-rest; NO unauthenticated inbound ingress; NO inline hand-rolled OAuth. | P-10:L73-L73 |
| P10-E018 | E/DEFER | REDTEAM deferred to execution pre-flight; aim at credential-leak, webhook mid-run replay faithfulness, listener escaping supervisor plane, forged-ingress (including replaying captured signed payload / nonce-timestamp RECON). | P-10:L97-L98 |
| P10-E019 | E/LIMIT | Persistent-listener daemon lifecycle (crash/restart, backpressure on firehose) needs C-17-pattern reuse review. | P-10:L101-L102 |
| P10-E020 | E/DEP | Next: P-10 gives P-9 http_call its auth plane and command center its inbound-event surface; P-11 composes both into describe/confirm/run. | P-10:L104-L105 |
| P10-A001 | A/DATA | Exact SsbEvent row shape for external ingress so adapter event is indistinguishable-downstream is RECON-1 (SCHEMA_REQUIRED against O-2). | P-10:L31-L32 |
| P10-A002 | A/DET | Whether K-14 already journals external wall-clock as replay input or webhook needs a new "recorded external event" leg is RECON-2. | P-10:L33-L33 |
| P10-A003 | A/SEC | How secrets are stored at rest and whether credential bytes get a taint tier (never journaled cleartext, never exported P-3) is RECON-3. | P-10:L34-L34 |
| P10-A004 | A/SEC | Exact HMAC/signing scheme for webhook providers is RECON against real providers' signing docs. | P-10:L50-L50 |
| P10-A005 | A/CFG | OAuth acquisition is declared follow-on without schedule; provider-specific endpoints must be source-grounded at execution (anti-stale). | P-10:L35-L35 |

Category coverage: ingress/auth capabilities ✓; CLI `elai cred add` ✓; webhook listener surface ✓; schemas/events ✓; security/privacy/secrets ✓; failures/fail-closed ✓; determinism/replay ✓; tests ✓; constraints ✓; deps/deferred ✓. Summary has no P-10 section.

### P-11 — workflow from-goal

Source: `docs/thefinalglance/P-plan/P-11_workflow_from_goal.md` (100/100 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| P11-E001 | E/STATUS | P-11 is PLANNED product surface post-spine; authored 2026-07-08; PROOF_SPINE none; PSR-2 ack 2026-07-09; user-facing composition demo riding P-9 vocabulary + P-10 ingress; n8n-user-converting front door. | P-11:L1-L5 |
| P11-E002 | E/CAP | `elai workflow from-goal <intent>` has the model PROPOSE a TaskSpec graph drawn ONLY from closed stage/connector vocabulary (O A3 kinds + P-9 mounted connectors), rendered on O-4 canvas, human CONFIRMS before anything runs. | P-11:L6-L7, L41-L43 |
| P11-E003 | E/FLOW | Constrained synthesis: model may only propose graphs whose nodes are existing typed stage kinds + mounted connectors, wires satisfy PortSchema/threaded-contract, gaps render VISIBLY DISCONNECTED (never silently invented). Human is final wiring authority. | P-11:L9-L10 |
| P11-E004 | E/DEP | Substrate: H-13 model-drives-canvas-ops (constrained mutation seam); O-2 TaskSpec rows + triggers; O-4 canvas + wire-dark; L-14 elai build RRA receipt pattern; P-9 mounted connectors + O A3 closed enum; PortSchema contracts. | P-11:L12-L18 |
| P11-E005 | E/CFG | FORK-1: if constrained-decode/grammar path exists -> constrain at decode; else propose-then-validate-then-reject with fail-closed validator + bounded re-propose loop. Prefer constrained decode; validator is mandatory backstop either way. | P-11:L32-L33 |
| P11-E006 | E/CFG | FORK-2: intent needs unmounted connector -> render VISIBLY DISCONNECTED gap + offer `elai connector new` (P-9 LANE A); do NOT auto-author connector inside from-goal. | P-11:L34-L34, L67-L68 |
| P11-E007 | E/LIMIT | ABORT-1: synthesis emitting a node OUTSIDE closed vocabulary -> STOP. Unavailable capability renders as disconnected gap, never fabricated node. | P-11:L36-L37 |
| P11-E008 | E/LIMIT | ABORT-2: proposed graph could EXECUTE without explicit human confirmation -> STOP. Proposal is inert (rows exist, no trigger armed) until confirmed. | P-11:L37-L38, L48-L50 |
| P11-E009 | E/LIMIT | ABORT-3: proposal WITHOUT RRA receipt (intent -> graph, provable) -> STOP; L-14 receipt makes synthesis auditable. | P-11:L38-L39 |
| P11-E010 | E/SURFACE | MOVE 1 CLI: `elai workflow from-goal <intent>` proposes inert TaskSpec graph (no trigger armed). | P-11:L41-L43, L64-L67 |
| P11-E011 | E/SURFACE | MOVE 2: proposed graph renders on O-4 canvas; unavailable capability VISIBLY DISCONNECTED with "needs a P-9 connector pack" hint. | P-11:L44-L47 |
| P11-E012 | E/FLOW | MOVE 3 confirm-before-run: human confirms/edits on canvas; confirmation arms O-2 triggers; before confirm no stage dispatches (no execution SSB rows). | P-11:L48-L50 |
| P11-E013 | E/EVENT | MOVE 4: from-goal run journals `{intent, proposed graph, model, confirmation}` as RRA receipt (L-14 pattern); receipt verifies via G-2. | P-11:L51-L53 |
| P11-E014 | E/DATA | MOVE 5: decisions doc `workflow/reasoning/P-11-decisions.md` records decode-constraint choice, gap-rendering contract, confirm-gate placement. | P-11:L54-L56 |
| P11-E015 | E/INT | PRODUCTION_HOOK: CLI from-goal -> H-13 canvas-op + O-4 render + O-2 TaskSpec store (inert rows); confirm gate -> O-2 trigger-arm path; from-goal run -> L-14 RRA sink. Seams RECON at pre-flight. | P-11:L58-L59 |
| P11-E016 | E/TEST | INTEGRATION_SMOKE R10: multi-step intent mixing available + UNMOUNTED connector; assert available part connected, unmounted VISIBLY DISCONNECTED (no fabricated node), NOTHING dispatches pre-confirm, RRA intent->graph lands, post-confirm O-2 triggers arm. | P-11:L61-L62 |
| P11-E017 | E/TEST | ACCEPTANCE: every proposed node in closed vocab + wires PortSchema-valid; unavailable renders DISCONNECTED; zero pre-confirm execution; verifiable RRA; OFF-AGENT evidence + INTEGRATION_SMOKE green. | P-11:L84-L88 |
| P11-E018 | E/LIMIT | stub_ban: NO out-of-vocab/fabricated node; NO execution pre-confirm; NO proposal without RRA; NO auto-authoring connector inside from-goal. | P-11:L67-L68 |
| P11-E019 | E/DEFER | REDTEAM deferred to execution pre-flight; aim at smuggling out-of-vocab via prompt-injection in intent, proposal arming trigger without confirmation, fabricated-connected node for unavailable connector. | P-11:L90-L91 |
| P11-E020 | E/LIMIT | Intent ambiguity (one goal -> several graphs) is expected; from-goal proposes ONE + human edits on canvas — proposal front door, not autonomous builder. | P-11:L94-L95 |
| P11-E021 | E/DEP | Next: P-11 front door makes P-9 + P-10 legible as describe/confirm/run; all three execute after proof spine as command-center distribution layer. | P-11:L97-L98 |
| P11-A001 | A/CFG | Constrained-decode vs propose-then-validate (FORK-1) is RECON-1 empirical on ELAI's actual decode path. | P-11:L27-L28, L93-L94 |
| P11-A002 | A/SURFACE | Whether O-4 maps proposed-but-unavailable node to disconnected visual + connector-new hint is RECON-2. | P-11:L29-L29 |
| P11-A003 | A/FLOW | Exact O-2/FSM site for human confirmation so NOTHING executes until confirmed is RECON-3. | P-11:L30-L30 |
| P11-A004 | A/RETRY | Bounded re-propose loop bounds (count, stop condition) under propose-then-validate are not numeric. | P-11:L32-L33 |

Category coverage: CLI + canvas confirm UI surfaces ✓; constrained workflow synthesis ✓; states/inert-to-armed ✓; RRA evidence ✓; security/closed-vocab ✓; failures/aborts ✓; tests ✓; constraints ✓; deps/deferred ✓. Summary has no P-11 section (only reference candidates).

### P-14 — explore-then-crystallize record->replay lane

Source: `docs/thefinalglance/P-plan/P-14_explore_then_crystallize_lane.md` (77/77 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| P14-E001 | E/STATUS | P-14 is PLANNED WAVE CU-2 EXTENSION of E-14 + O-2; PROOF_SPINE feeds PS-5; execute after proof spine per A-7; Track COMPUTER-USE-STEAL; lever `agentic-run-mints-artifact`; STRONG grounding (cua replay_env HIGH; openclaw jsonl-replay is NOT record->re-score — C4); effort M-L; depends P-13. | P-14:L1-L5 |
| P14-E002 | E/CAP | Exploration run emits a deterministic script AS an O-2 TaskSpec stage + RRA receipt binding script->minting-run; replay = cheap zero-reasoning execution mode. | P-14:L5-L5, L18-L19 |
| P14-E003 | E/EVIDENCE | Prior-art correction C4: openclaw jsonl-replay re-drives INPUT through MOCK provider — cite cua replay_env, not openclaw, as the shape prior art. EXCLUDED as vaporware: openclaw-style mock replay framed as record->re-score. | P-14:L8-L9, L20-L21 |
| P14-E004 | E/CFG | SHIP NOW (Tier-1): crystallizer takes E-14 (browser) exploration trace and EMITS deterministic Playwright script as O-2 TaskSpec stage kind, with G-2 RRA binding emitted script hash to exploration run_id; replay drives real entrypoint with NO model in the loop. | P-14:L18-L19 |
| P14-E005 | E/DEFER | DEFER (Tier-2): pyautogui (desktop) crystallizer over P-12 traces; re-score-on-replay leg (cua evaluate_task analog); assertion synthesis from exploration AXTree deltas. | P-14:L19-L20 |
| P14-E006 | E/DATA | produces: `crates/orchestrator/src/crystallize/mod.rs` (Crystallizer) + `rra_bind.rs` (script_hash -> minting run_id RRA binder). | P-14:L29-L31 |
| P14-E007 | E/DEP | Consumes O-2 TaskSpecStage (EXTEND kinds), G-2 ReproducibleRunArtifact; composes C crystallizer general form (do NOT fork); J-96 case graph adjacency feeds PS-5. | P-14:L11-L16, L32-L34 |
| P14-E008 | E/SURFACE | CLI: `elai e2e crystallize` + replay via task_runner tick (`elai e2e replay --stage`); PRODUCTION CLI + task_runner. | P-14:L35-L36, L46-L48, L71-L72 |
| P14-E009 | E/SURFACE | frontend_pair: crystallized-stage node on O-4 canvas (Tier-2; decide at edit W-4). | P-14:L43-L44 |
| P14-E010 | E/LIMIT | stub_ban: NO crystallized stage without RRA binding to real minting run; NO model call in replay path; NO new dispatch runtime; NO .unwrap()/.expect(); NO blake2/sha2 (blake3 for script hash). | P-14:L38-L42 |
| P14-E011 | E/EVENT | Minting lands `crystallized_stage_minted` row `{script_hash, minting_run_id, rra_hash}`. | P-14:L46-L48, L50-L51 |
| P14-E012 | E/DET | Replay reproduces exploration's observable outcome with ZERO model calls in the trace; byte-reproducible under same seed (G-2). | P-14:L50-L55 |
| P14-E013 | E/CFG | FORK: if exploration has genuinely non-deterministic step (CAPTCHA, model-only inference), crystallization DOES NOT emit that step as deterministic — route to marked "agentic-step" stage re-entering exploration; record boundary in RRA; do NOT silently bake guessed value. | P-14:L53-L53 |
| P14-E014 | E/SEC | REDTEAM surviving-attack seed: crystallized script re-invoking model every step — caught by zero-model-calls assertion. Unpatched success: bake run-specific session token/timestamp into replay — PATCH: parameterize run-specific values as stage inputs and RRA records them; hard-coded volatile literal is stub_ban. Deep red-team deferred to execution pre-flight. | P-14:L61-L61 |
| P14-E015 | E/LIMIT | ABORT: if crystallization would produce replay NOT byte-reproducible under same seed, STOP — deterministic replay that drifts defeats the lever. | P-14:L55-L55 |
| P14-E016 | E/TEST | VERIFY: plan_gate_check + wiring_contract_check; load-bearing proof is off-agent crystallized_stage_minted + replay trace asserting zero model calls via real `elai e2e crystallize`/`replay`. | P-14:L56-L60 |
| P14-E017 | E/TEST | INTEGRATION_SMOKE: independent agent full E-14 multi-step SaaS journey (login + multi-page form), crystallize, replay N times byte-identical under seed with zero model calls, `elai replay --verify` re-binds script to minting run. Toy single-step does NOT satisfy R10. | P-14:L63-L64 |
| P14-E018 | E/INT | PRODUCTION_HOOK: task_runner TaskSpecStage gains crystallized_script kind; `elai e2e crystallize`/`replay` in cli/main.rs; Crystallizer + RRA binder wired both same-changeset. | P-14:L71-L72 |
| P14-E019 | E/DEP | Next: closes SYNTHESIS 2b; compounds PS-5; pairs with P-15 as reuse-artifact minters feeding K-18 case-reuse economics. | P-14:L74-L75 |
| P14-A001 | A/INT | Exact O-2 TaskSpecStage enum to extend and cua environment.py record->re-score shape confirmation are RECON_NEEDED at edit. | P-14:L54-L54 |
| P14-A002 | A/TEST | No quantitative replay-reliability threshold is set ("~100% reliability" is narrative industry split). | P-14:L8-L8 |
| P14-A003 | A/DATA | Full script portability matrix beyond parameterized stage inputs is not fully specified. | P-14:L61-L61 |
| P14-C001 | C/DEFER | Industry/summary framing often pairs Playwright + PyAutoGUI as peer emitters; numbered source puts browser Playwright in Tier-1 and desktop pyautogui in Tier-2. | P-14:L18-L20 |

Category coverage: crystallize/replay capability ✓; CLI surfaces ✓; O-4 Tier-2 node ✓; schemas/events ✓; determinism ✓; security/parameterization ✓; tests ✓; constraints ✓; deps/deferred ✓. P-13 perception during exploration is dependency context, not a P-14 implementation surface.

### P-19 — config/ops lifecycle + elai doctor + verified backups

Source: `docs/thefinalglance/P-plan/P-19_config_ops_lifecycle_doctor.md` (78/78 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| P19-E001 | E/STATUS | P-19 is PLANNED WAVE OC-2 NEW-area (composition-heavy; ELAI's thinnest area); EXTENSION of J config-ingress + M + 3H; PROOF_SPINE none (ops); owner-acked 2026-07-11; execute after proof spine; Track OPENCLAW-STEAL; lever `recoverable-config-and-state`; STRONG openclaw grounding. | P-19:L1-L5 |
| P19-E002 | E/CAP | Add recoverable config, `elai doctor`, verified journal backups, and a restart continuation queue so a trust product cannot cheaply corrupt its own config or fail to restore state. | P-19:L5-L5 |
| P19-E003 | E/EVIDENCE | Steal from openclaw: strict-schema fail-closed boot (unknown key = REFUSE), atomic writes, last-known-good with doctor-only restore, clobber guards (>50% shrink rejected, preserved as .rejected.<ts>), hot-reload settle + invalid-edit rejection; doctor detect/repair/lint with STABLE exit codes; VACUUM-INTO WAL-safe SQLite snapshots + manifest verify; restart continuation queue (one turn); compaction split-rule (tool-call/result pairs together) + successor transcripts (never rewrite in place). | P-19:L8-L9 |
| P19-E004 | E/LIMIT | CORRECTION C2: openclaw copies full process.env with NO allowlist — do the OPPOSITE (allowlist env); adopt openclaw cwd path-jail (case-fold-before-compare on win32) verbatim. | P-19:L9-L9, L20-L21, L40-L41 |
| P19-E005 | E/CFG | SHIP NOW (Tier-1): (a) fail-closed-default-object config posture (policy objects fail closed even when ABSENT) + last-known-good + rejected-write preservation (.rejected.<ts>) + shrink/clobber guards (>50% shrink rejected) + atomic writes on J config-ingress gate; (b) `elai doctor` detect/repair/lint with STABLE exit codes + structured findings; ENV ALLOWLISTED; cwd path-jail. | P-19:L18-L19 |
| P19-E006 | E/DEFER | DEFER (Tier-2): VACUUM-INTO WAL-safe verified SSB/session backups + manifest verify; restart continuation queue (one in-flight turn survives restart); post-upgrade compat probes; compaction split-rule + successor-transcript stated invariants (likely already in 3H — make explicit). | P-19:L19-L20 |
| P19-E007 | E/LIMIT | EXCLUDED: openclaw full-env-copy (C2); any config write that overwrites on >50% shrink. | P-19:L20-L21 |
| P19-E008 | E/DATA | produces: `crates/orchestrator/src/config/lifecycle.rs` (ConfigLifecycle: LKG + rejected-write + shrink/clobber + atomic write + env allowlist) + `crates/cli/src/doctor_cmd.rs` (elai doctor detect/repair/lint). | P-19:L29-L31 |
| P19-E009 | E/DEP | Consumes J ConfigIngressGate (EXTEND recoverable — do NOT re-implement admission) + 3H SessionStore (compaction split-rule + successor-transcript target). | P-19:L32-L34 |
| P19-E010 | E/SURFACE | CLI: `elai doctor` subcommand (detect/repair/lint); PRODUCTION CLI. No dashboard tile — justified W-6 N/A: structured text + stable exit codes for CI. | P-19:L35-L36, L44-L45, L56-L57 |
| P19-E011 | E/INT | Production reachability also includes config ingress path invoking lifecycle guards on every write. | P-19:L36-L37, L72-L73 |
| P19-E012 | E/LIMIT | stub_ban: NO config write overwriting on >50% shrink; NO full process.env copy; NO doctor exit code varying run-to-run for same state; NO compaction splitting tool-call from result; NO in-place transcript rewrite; NO .unwrap; NO blake2/sha2. | P-19:L39-L43 |
| P19-E013 | E/EVENT | >50%-shrink write REJECTED and preserved as .rejected.<ts> with `config_write_rejected` row; `elai doctor lint` returns STABLE nonzero exit + structured `doctor_finding` rows. | P-19:L46-L49, L51-L53 |
| P19-E014 | E/CFG | FORK: if config is corrupt beyond last-known-good restore, `elai doctor repair` refuses to auto-repair and surfaces structured finding for human action; do NOT silently write a default over user config. | P-19:L54-L54 |
| P19-E015 | E/LIMIT | ABORT: if config-ingress could admit an unknown key silently (drop fail-closed boot) or copy full env, STOP — both are openclaw failure modes this unit exists to beat. | P-19:L56-L56 |
| P19-E016 | E/SEC | REDTEAM: truncating write defeated by >50%-shrink clobber guard. Unpatched success: config edit smuggling secret via full-env copy (C2) — PATCH: env allowlisted; secrets live in P-10 vault, never config or journal. Deep red-team deferred. | P-19:L62-L62 |
| P19-E017 | E/TEST | VERIFY: plan_gate_check + wiring_contract_check; load-bearing proof is off-agent config_write_rejected + doctor_finding rows + stable doctor exit via real `elai doctor`. | P-19:L57-L61 |
| P19-E018 | E/TEST | INTEGRATION_SMOKE: hard input malformed config (unknown key) + >50% shrink + kill mid-turn; assert fail-closed boot rejects unknown key, shrink preserved not overwritten, doctor exit stable across 3 runs, in-flight turn resumes once after restart. Note: resume leg is Tier-2 deferred capability used in smoke description. | P-19:L64-L65 |
| P19-E019 | E/DEP | Next: closes O3 config/ops lifecycle; pairs with M packaging/support and Q ops hardening; journal-backup leg protects product's evidence store. | P-19:L75-L76 |
| P19-A001 | A/CFG | Stable exit-code map and structured doctor_finding schema are not enumerated. | P-19:L18-L19, L46-L49 |
| P19-A002 | A/DATA | Restore workflow, retention, backup encryption, storage destination, verification algorithm for Tier-2 backups are absent. | P-19:L19-L20 |
| P19-A003 | A/STATE | Exact J ConfigIngressGate + 3H SessionStore symbols and openclaw VACUUM-INTO + restart-continuation shape are RECON_NEEDED. | P-19:L55-L55 |
| P19-A004 | A/CONC | Multi-session / multi-queued continuation behavior beyond "one in-flight turn" is not defined (Tier-2). | P-19:L19-L20, L64-L65 |
| P19-C001 | C/DEFER | Summary inventory lists VACUUM backups, restart continuation, post-upgrade probes, and compaction invariants as peer requirements; numbered REVISED scope places those in Tier-2 while Tier-1 is config lifecycle guards + doctor. INTEGRATION_SMOKE still references mid-turn resume. | P-19:L18-L20, L64-L65 |

Category coverage: config lifecycle ✓; CLI doctor surfaces (detect/repair/lint) ✓; events/findings ✓; security/env-allowlist ✓; failures/recovery ✓; determinism/stable exits ✓; tests ✓; constraints ✓; deps/deferred ✓. Intentionally no UI dashboard tile.

### P-24 — mobile design/test lane

Source: `docs/thefinalglance/P-plan/P-24_mobile_design_test_lane.md` (83/83 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| P24-E001 | E/STATUS | P-24 is PLANNED WAVE CU-5; PROOF_SPINE none (product); owner-commissioned 2026-07-11; Tier-3 crystallize transitively feeds PS-5 via P-14 (PSR-1 tier 3); execute after proof spine; Track COMPUTER-USE-STEAL; lever `mobile-as-a-dimension-not-a-subsystem`; STRONG external + MODERATE ELAI-anchor grounding; EXTENSION; effort L; depends H-14(B0+tagging), P-12, P-13, P-14, P-16. | P-24:L1-L5 |
| P24-E002 | E/CAP | Mobile is a typed DIMENSION threaded through existing lanes, never a parallel mobile subsystem; closes H-14 smartphone deferral for RN/Expo. | P-24:L5-L5, L17-L17 |
| P24-E003 | E/FLOW | Three rungs cheapest-first; every rung labels render source; a lower rung NEVER discharges a higher rung's gate. | P-24:L16-L17, L41-L42 |
| P24-E004 | E/DATA | RUNG 1: typed DeviceProfile catalog as DATA rows `{name, width, height, dpr, ua, touch}` (named constants, no magic metrics in code). | P-24:L17-L18 |
| P24-E005 | E/SURFACE | RUNG 1: device_frame render variant on H-14 preview node and P-23 browse node (CDP Emulation.setDeviceMetricsOverride); E-14 journeys accept device MATRIX; SSIM goldens become per-profile. | P-24:L17-L18, L45-L46 |
| P24-E006 | E/FLOW | RUNG 2: Expo dev server (Metro web, production-grade SDK 54+) is frameable origin; H-14 B0 proxy + data-oid tagging + recast writeback extend to RNW; click-to-edit resolves RN component AST ranges. Render source labeled `rnw_web` — WEB pass on RN source, never claimed as native. | P-24:L18-L19 |
| P24-E007 | E/FLOW | RUNG 3: Android emulator (iOS simulator where macOS host exists) as P-12 sandbox backend; booted from PINNED AVD/sim snapshot whose image hash lands in RRA hardware pin. | P-24:L19-L20 |
| P24-E008 | E/DATA | Perception = platform a11y tree (uiautomator dump / XCUITest a11y) as NEW P-13 source tag `mobile_axtree`; pixel/OCR tiers fire only when empty (J-1 unchanged). | P-24:L19-L20 |
| P24-E009 | E/FLOW | Actions frame-bound per P-12; agentic exploration crystallizes via P-14 into Maestro YAML flow, RRA-bound; replay = zero-reasoning mode. | P-24:L19-L20 |
| P24-E010 | E/DEP | P-16 catalog provisions emulator images/SDKs/Maestro binary (admission-gated, never ad-hoc install). | P-24:L19-L20, L44-L44 |
| P24-E011 | E/DEFER | RN click-to-edit-on-native (Fiber getInspectorDataForViewAtPoint) is REGISTERED FOLLOW-UP inside Rung 3's FORK, not a promise. | P-24:L19-L20, L31-L31, L81-L81 |
| P24-E012 | E/LIMIT | EXCLUDED: parallel mobile subsystem; VLM-read screenshots as element source (P-13/J-1); real physical devices (neither disposable nor snapshot-pinnable — ABORT class). | P-24:L20-L21, L56-L56 |
| P24-E013 | E/DATA | produces: `crates/orchestrator/src/mobile/device_profile.rs` + `emulator_backend.rs` (P-12 SandboxBackend: pinned-snapshot boot + mobile_axtree feed + Maestro crystallize target). | P-24:L29-L31 |
| P24-E014 | E/SURFACE | CLI: `elai e2e run --device <profile>`; `elai operator run --backend emulator`. | P-24:L36-L37, L47-L48, L77-L78 |
| P24-E015 | E/CFG | task_runner gains `device_profile` field on browse/e2e/operator TaskSpec stages. | P-24:L37-L38 |
| P24-E016 | E/SURFACE | H-13 canvas dispatcher: device_frame render variant on preview/browse nodes; phone-bezel render + per-profile pass/fail chips; iOS-unavailable = HONEST-GAP disconnected affordance, never silent skip. | P-24:L38-L39, L45-L46 |
| P24-E017 | E/OBS | Every result row carries `render_source` enum `web_viewport \| rnw_web \| native_emulator`; rung-label laundering is a gate defect. | P-24:L41-L42, P-24:L49-L50 |
| P24-E018 | E/EVENT | expect_ssb `emulator_session`; journey axtree_snapshot rows carry device_profile + render_source; Rung-3 RRA hardware pin holds snapshot hash; crystallizes Maestro flow RRA-bound. | P-24:L47-L50, L52-L53 |
| P24-E019 | E/TEST | NEGATIVE fails: rnw_web pass presented against native_emulator gate; emulator boot from unpinned image; mobile count sourced from VLM. | P-24:L49-L50 |
| P24-E020 | E/CFG | FORK: no macOS host -> iOS Tier-3 unavailable as honest-gap (Android-only); NEVER substitute rnw_web for missing iOS. Empty uiautomator a11y on RN -> P-13 tier ladder (OCR/pixel) + `perception_degraded` (not silent axtree_exact). Maestro fails P-16 admission -> fall back to P-12 actuator with plain journaled step list as crystallize format; do NOT install around catalog. | P-24:L54-L55 |
| P24-E021 | E/LIMIT | ABORT: physical device drive; emulator that reaches host filesystem/network outside sandbox profile; missing rung-label on any result row; H-14 B0 proxy still unshipped at execution (STOP — do not re-implement proxy). | P-24:L56-L56 |
| P24-E022 | E/TEST | Maestro is actuation-only; replay verdicts come from SAME off-agent evidence as minting run (SSB rows + screenshot hash compare per 4I); negative arm: mutated fixture makes replay actuate green while evidence leg FAILS. | P-24:L53-L53, L63-L64 |
| P24-E023 | E/TEST | VERIFY: plan_gate_check + wiring_contract_check + production_caller_check on device_profile.rs + emulator_backend.rs. | P-24:L57-L62 |
| P24-E024 | E/TEST | INTEGRATION_SMOKE after P-24 lands with P-12/P-13/P-14: SAME journey across all three rungs on HARD Expo fixture (broken phone-only layout + control lacking a11y props); assert Rung1 layout catch, Rung2 AST click, Rung3 pinned boot + honest perception_degraded + Maestro crystallize + evidence-leg replay + correct render_source every row. | P-24:L66-L67 |
| P24-E025 | E/SEC | REDTEAM failed: claim mobile coverage from cheap rung — render_source typed field blocks. Failed: flaky emulator reboots different image until green — snapshot hash in RRA ABORTs mismatch. Succeeded unpatched: Maestro assertVisible as verdict authority — PATCH Maestro actuation-only + evidence-leg. Deep red-team deferred. | P-24:L63-L64 |
| P24-E026 | E/EVIDENCE | Grounding: H-14:1,7,184; Expo web SDK 54+ searched 2026-07-11; Maestro mobile-dev-inc searched 2026-07-11; CDP deviceScaleFactor in-repo capture harness; 4I/4D/4E/B-42. | P-24:L69-L75 |
| P24-E027 | E/INT | PRODUCTION_HOOK: cli device/emulator flags + task_runner device_profile + H-13 device_frame — DeviceProfile resolver + emulator backend wired into ALL THREE same-changeset (vertical slice; Rung 3 crystallize arm registers in P-14 target enum same-changeset). | P-24:L77-L78 |
| P24-A001 | A/DEP | Maestro license + latest release + binary checksum channel (P-16 license_tag) is RECON at edit (anti-slopsquat). | P-24:L55-L55 |
| P24-A002 | A/INT | Expo SDK data-oid tagging through Metro, uiautomator dump fidelity on RN fixture, and CDP device-metrics call sites need settling checks. | P-24:L55-L55 |
| P24-A003 | A/CFG | Device-profile field defaults beyond schema, SSIM thresholds, and full matrix coverage enumeration are not fixed. | P-24:L17-L18 |
| P24-A004 | A/DEFER | Native Fiber-inspector click-to-edit remains registered Rung-3 follow-up without acceptance criteria here. | P-24:L19-L20, L81-L81 |

Category coverage: multi-rung mobile capability ✓; CLI flags + canvas device_frame + pass/fail chips + honest-gap UI ✓; schemas/events ✓; security/sandbox/pins ✓; failures/aborts ✓; determinism/RRA ✓; tests ✓; constraints ✓; deps/deferred ✓.

## Cross-source interactions and lifecycle

| Interaction ID | Class | Interaction, ordering, or shared invariant | Evidence atoms |
|---|---|---|---|
| IX-001 | E | P-1 is the KEYSTONE: AppManifest is the bind point for P-2 ports, P-6 namespace validation, L-14 RRA lineage, and P-5 operator fleet render. | P1-E002–P1-E004, P1-E029; P2-E003; P6-E017 |
| IX-002 | E | P-2 extends P-1 manifests with PortSchema and defers O-4 canvas edge chrome to Tier-2; dispatch remains FSM (O-4 doctrine), never a bus runtime. | P2-E002–P2-E010, P2-E017 |
| IX-003 | E | P-6 domain_label on ports and AppManifest data_namespaces share the life\|work\|shared vocabulary; P-1 refuses unclaimed namespaces at register via O-3 claims. | P1-E017; P2-E003; P6-E003–P6-E005 |
| IX-004 | E | P-6 OUT-flow privacy is the mirror of G-1 IN-flow taint; both ride N-6's ONE lattice (N-2 forbids a second scale). | P6-E002–P6-E006, P6-E014 |
| IX-005 | E | P-7 connected default turns P-2 world bus on; toy default targets P-5 ceremony; team targets M org pack + P-8 — one engine, three data defaults. | P7-E008; P2-E001; P1-E016 |
| IX-006 | E | P-7 process adaptation is deliberately disjoint from E-5/E-4 content adaptation and extends D-14/N-15 anti-sycophancy to process grain via M-2 asymmetry. | P7-E002–P7-E005 |
| IX-007 | E | Command-center triad: P-9 owns connector-pack format + http_call; P-10 owns ingress adapters + CredentialVault auth plane; P-11 is the from-goal front door composing both for the user. All three execute after the proof spine. | P9-E001–P9-E003, P9-E023; P10-E001–P10-E002, P10-E020; P11-E001–P11-E002, P11-E021 |
| IX-008 | E | P-9 http_call taints Untrusted at ingest (G-1), validates to PortSchema (P-2), pins egress to signed declared hosts (G-3 compose), and mounts only through existing L-12/skillspector/J-102/N-6 admission. | P9-E007–P9-E009, P9-E020 |
| IX-009 | E | P-10 normalizes all external shapes to O-2 SsbEvent without growing the trigger enum; WebhookIngress lives on I-1 supervisor plane, not the pipeline hot path. | P10-E002–P10-E006, P10-E010 |
| IX-010 | E | P-10 CredentialVault must never leak into SSB/RRA/P-3 export and is the secret home P-19's env-allowlist patch points at (secrets not in config/journal). | P10-E008, P10-E013; P19-E016 |
| IX-011 | E | P-11 proposals are constrained to O A3 closed kinds + P-9 mounted connectors; unmounted needs render as O-4 wire-dark gaps offering P-9 `elai connector new`, never fabricated nodes or auto-authored packs. | P11-E002–P11-E007, P11-E011; P9-E010 |
| IX-012 | E | P-11 confirm-before-run keeps proposed TaskSpec rows inert until human arms O-2 triggers; P-18 later hardens this confirmation seam (referenced by summary). | P11-E008, P11-E012 |
| IX-013 | E | P-14 Tier-1 crystallizes E-14 browser traces into O-2 crystallized_script stages with G-2 RRA binding; P-24 Rung 3 reuses the same crystallizer arm for Maestro YAML. | P14-E004–P14-E011; P24-E009, P24-E027 |
| IX-014 | E | P-14 and P-15 both mint reuse artifacts feeding PS-5; openclaw mock-replay is explicitly excluded (C4). | P14-E001–P14-E003, P14-E019 |
| IX-015 | E | P-19 Tier-1 extends J config-ingress with recoverable guards and `elai doctor`; Tier-2 backups/continuation protect the same evidence store other units journal into. | P19-E005–P19-E006, P19-E019 |
| IX-016 | E | P-24 threads DeviceProfile/device_frame/mobile_axtree/emulator backend through H-14, E-14, P-12, P-13, P-14, P-16, P-23 without forking a mobile subsystem. | P24-E002–P24-E010, P24-E026 |
| IX-017 | E | P-24 render_source honesty (`web_viewport\|rnw_web\|native_emulator`) is the structural anti-laundering control analogous to P-2/P-11 wire-dark honest gaps. | P24-E017–P24-E019; P2-E006; P11-E011 |
| IX-018 | E | Shared production-caller doctrine: every unit forbids `#[cfg(test)]`/contracts callers from satisfying R7 and names real CLI and/or scheduler/FSM production hooks. | P1-E024; P2-E018; P6-E015; P7-E014; P9-E017; P10-E014; P11-E015; P14-E008; P19-E010–P19-E011; P24-E027 |
| IX-019 | E | Shared off-agent verification pattern: wiring_contract_check (+ often production_caller_check) and R10 multi-path INTEGRATION_SMOKE; P-6 additionally requires N-2 corpus_composition_check. | P1-E025–P1-E026; P6-E016–P6-E017; P9-E018–P9-E019 |
| IX-020 | E | Shared anti-second-substrate doctrine: no second scheduler (P-1), no second lattice (P-6), no second provenance scale (P-2), no second dispatch runtime (P-14), no parallel mobile subsystem (P-24), no open stage vocabulary (P-9/P-11). | P1-E009; P2-E008; P6-E006; P9-E014; P11-E007; P14-E010; P24-E012 |
| IX-021 | I | Dashboard DESIGN.md surfaces implied by this chunk: DashboardInbox red-tick cards (P-1), deferred FleetPanel (P-5), O-4 port edges + disconnected chrome (P-2 Tier-2), deferred privacy grant panel (P-5/P-6), O-4 proposed-graph + confirm UI (P-11), crystallized-stage node (P-14 Tier-2), device_frame + pass/fail chips + iOS honest-gap (P-24), intentionally no doctor dashboard tile (P-19). | P1-E016, P1-E020; P2-E012; P6-E008; P11-E011–P11-E012; P14-E009; P19-E010; P24-E016 |
| IX-022 | E | Post-spine execution gate is uniform for product-surface units in this chunk (especially P-9/P-10/P-11 and steal-track units); personal-wave units are owner-acked WAVE 1–3 but still product-surface class. | P1-E001; P9-E001; P10-E001; P11-E001; P14-E001; P19-E001; P24-E001 |

## Explicit ambiguity and conflict register

| Register ID | Kind | Unresolved issue or required correction | Atom references |
|---|---|---|---|
| AC-001 | conflict/scope | P-1 opening pipeline names DriftCanary and gated auto-repair apply; REVISED Tier-1 is Smoke+DepRotCheck diagnosis only. | P1-C001, P1-C002 |
| AC-002 | ambiguity/dep | P-1 G-10 per-app canary extractability, TaskSpec factory API, and F-9 entrypoint remain RECON. | P1-A001–P1-A003 |
| AC-003 | conflict/scope | P-2 steal/build centers O-4 canvas edges and third-party mount; REVISED Tier-1 is CLI-only. | P2-C001 |
| AC-004 | ambiguity | P-2 compatibility/version rules and separate refusal-row requirement are open. | P2-A004–P2-A005 |
| AC-005 | conflict/scope | P-6 dual O-3+E-20 enforcement is stated in steal/build but E-20 is Tier-2 only. | P6-C001 |
| AC-006 | ambiguity/data | P-6 grant expiry/nesting/precedence unspecified. | P6-A004 |
| AC-007 | conflict/scope | P-7 premise is measured auto-promotion; Tier-1 is manual asymmetry-guarded change. | P7-C001 |
| AC-008 | ambiguity/config | P-7 FP metrics/thresholds/observation duration/defaults and domain vocabulary unspecified. | P7-A001–P7-A002 |
| AC-009 | ambiguity | P-9 ConnectorSpec enums, pack-type admission discriminator, PortSchema binding, and signing key are RECON. | P9-A001–P9-A004 |
| AC-010 | ambiguity | P-10 SsbEvent ingress shape, K-14 replay leg, vault at-rest, webhook signing scheme, OAuth follow-on schedule are RECON. | P10-A001–P10-A005 |
| AC-011 | ambiguity | P-11 constrained-decode path, gap visual mapping, confirm-gate site, and re-propose bounds are RECON/open. | P11-A001–P11-A004 |
| AC-012 | conflict/scope | P-14 Playwright is Tier-1; PyAutoGUI is Tier-2 despite peer framing in coarser inventories. | P14-C001 |
| AC-013 | ambiguity | P-14 Stage enum extension site and quantitative reliability threshold open. | P14-A001–P14-A002 |
| AC-014 | conflict/scope | P-19 summary treats backups/continuation/compaction as peer features; numbered Tier-1 is config guards + doctor only. | P19-C001 |
| AC-015 | ambiguity | P-19 exit-code map, doctor_finding schema, backup restore details, multi-session continuation open. | P19-A001–P19-A004 |
| AC-016 | ambiguity | P-24 Maestro/Expo/uiautomator RECON items, SSIM thresholds, native Fiber follow-up criteria open. | P24-A001–P24-A004 |

## Summary additions and corrections

Orientation comparison target: `docs/thefinalglance/feature-discovery/16-P-SUMMARY_INDEX.discovery.md`. “ADD” means an explicit source behavior omitted or materially compressed by the summary; “CORRECT/QUALIFY/REMOVE” means the summary’s wording changes scope or certainty. These are relational records, not additional feature atoms.

### P-1 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-P1-001 | **CORRECT** drift canaries and one-click auto-repair are not Tier-1: Tier-1 is Smoke+DepRotCheck + N-7 diagnosis card; apply-button and G-10 generalization are Tier-2/ABORT-gated. | 16-P:L87-L123 | P1-E012–P1-E013, P1-C001–P1-C002 |
| SD-P1-002 | **ADD** exact CLI verbs `register\|list\|tick`, closed AppManifest field set including `app_id`/`one_sentence_purpose`/`smoke_trigger`, and O-3 namespace-claims hard refusal at register. | 16-P:L93-L118 | P1-E003, P1-E015, P1-E017 |
| SD-P1-003 | **ADD** auto-archive-on-green, exactly-once per due tick, batched multi-app TaskSpec storm patch, and dedicated FleetPanel deferral to P-5. | 16-P:L109-L123 | P1-E016, P1-E019–P1-E020, P1-E027 |
| SD-P1-004 | **ADD** production dual hooks (CLI + task_runner), R10 two-app multi-tick smoke, wiring/production_caller checks, and full consume list including O-3. | 16-P:L117-L118 | P1-E024–P1-E026, P1-E029 |
| SD-P1-005 | **EXPAND** P1-A01 recon with TaskSpec factory and F-9 entrypoint unknowns; add Cron/TTL default gap. | 16-P:L119-L122 | P1-A001–P1-A004 |

### P-2 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-P2-001 | **CORRECT/QUALIFY** O-4 canvas edge rendering and third-party M-5 mount are Tier-2; Tier-1 trust value is fully CLI (`wire`/`ports`/`--unwired`) + SSB. | 16-P:L124-L153 | P2-E009–P2-E012, P2-C001 |
| SD-P2-002 | **ADD** PortSchema fields `{name, json_schema, domain_label}`, `deny_unknown`, and P-6 domain coupling. | 16-P:L130-L131 | P2-E003, P2-E013 |
| SD-P2-003 | **ADD** FORK when O-4 edge model cannot carry tool->tool edges; ABORT on live broker; L-12 admission for malicious mount patch. | 16-P:L137-L153 | P2-E016–P2-E017, P2-E021 |
| SD-P2-004 | **EXPAND** compatibility/version and refusal-row ambiguities beyond summary P2-A01/A02. | 16-P:L150-L153 | P2-A001–P2-A005 |

### P-6 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-P6-001 | **CORRECT** E-20 routing enforcement is Tier-2 (with P-4), not Tier-1; Tier-1 is O-3 vault-read only. Summary lists E-20 as peer requirement. | 16-P:L257-L288 | P6-E007–P6-E008, P6-C001 |
| SD-P6-002 | **ADD** grant-management UI deferral to P-5, JOIN-table FORK if N-6 column migration hits N-4, and N-2 corpus_composition_check as hard VERIFY gate. | 16-P:L263-L287 | P6-E008, P6-E013, P6-E016 |
| SD-P6-003 | **ADD** full INTEGRATION_SMOKE multi-namespace multi-tool + mid-run revoke scenario and redteam second-scale patch. | 16-P:L283-L287 | P6-E017–P6-E018 |
| SD-P6-004 | **EXPAND** grant-scope ambiguity (expiry/nesting/precedence) already noted as P6-A01. | 16-P:L285-L287 | P6-A004 |

### P-7 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-P7-001 | **CORRECT/QUALIFY** automatic N-3 learning loop is Tier-2; Tier-1 is manual asymmetry-guarded change with FORK if N-3 not live. Summary reads as if measured promotion is immediate. | 16-P:L289-L323 | P7-E006–P7-E007, P7-E012, P7-C001 |
| SD-P7-002 | **ADD** exact default wiring toy->P-5, connected->P-2 on, team->M+P-8 and `operating_profile_change` field shape. | 16-P:L309-L316 | P7-E008, P7-E010 |
| SD-P7-003 | **ADD** ABORT on non-own-run FP evidence, observe-first GD-4 detail, and R10 multi-op refuse+promote+revert smoke. | 16-P:L306-L323 | P7-E011, P7-E013, P7-E016–P7-E017 |
| SD-P7-004 | **EXPAND** RECON for H-17/H-18 process-metric scoring and M-2 enforcement point beyond summary P7-A01/A02. | 16-P:L320-L323 | P7-A001–P7-A005 |

### P-9 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-P9-001 | **ADD entire unit** — summary has only reference-derived P9-I01/I02/A01 with no fields, CLI, events, or acceptance. Numbered source supplies full ConnectorSpec/http_call/pack/admission design. | 16-P:L360-L366 | P9-E001–P9-E023 |
| SD-P9-002 | **ADD** user-facing `elai connector new`, MountedConnector journal surface, LANE A/B fork, egress pin, fail-closed tampered-pack refusal. | 16-P:L360-L366 | P9-E008–P9-E010 |
| SD-P9-003 | **ADD** post-spine execution timing, KD-1 n8n-count caution, and deferred redteam surface list. | 16-P:L19-L24, L360-L366 | P9-E001, P9-E021–P9-E022 |

### P-10 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-P10-001 | **ADD entire unit** — summary has only P10-I01..I03/A01 references from later units; no vault schema, adapter taxonomy, or entrypoints. | 16-P:L367-L370 | P10-E001–P10-E020 |
| SD-P10-002 | **ADD** four-shape normalization, WebhookIngress HMAC-before-parse, `elai cred add` v1, CredentialVault non-leak, I-1 listener isolation, OAuth follow-on block. | 16-P:L367-L370 | P10-E003–P10-E011 |
| SD-P10-003 | **ADD** faithful async replay requirement (ABORT-2) and zero-leak negative arm as load-bearing acceptance. | 16-P:L367-L370 | P10-E007–P10-E008, P10-E015–P10-E016 |

### P-11 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-P11-001 | **ADD entire unit** — summary has only P11-I01/A01; no graph-synthesis rules, UI/API, events, or gates. | 16-P:L371-L372 | P11-E001–P11-E021 |
| SD-P11-002 | **ADD** `elai workflow from-goal`, O-4 proposed-graph + confirm UI, inert-until-confirm, RRA intent->graph, honest disconnected gap + connector-new hint. | 16-P:L371-L372 | P11-E010–P11-E013 |
| SD-P11-003 | **ADD** FORK constrained-decode vs validate, ABORT closed-vocab/pre-confirm/RRA, and deferred redteam injection surfaces. | 16-P:L371-L372 | P11-E005–P11-E009, P11-E019 |

### P-14 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-P14-001 | **CORRECT** PyAutoGUI desktop crystallizer is Tier-2 deferred; Tier-1 is E-14 browser Playwright only. Summary lists both as peer emitters. | 16-P:L464-L498 | P14-E004–P14-E005, P14-C001 |
| SD-P14-002 | **ADD** openclaw C4 correction (mock input replay is not prior art), agentic-step FORK for non-deterministic UI, blake3 script hash, and volatile-literal parameterization patch. | 16-P:L470-L498 | P14-E003, P14-E010, P14-E013–P14-E014 |
| SD-P14-003 | **ADD** exact module paths crystallize/mod.rs + rra_bind.rs, `elai e2e replay --stage`, and R10 multi-step SaaS smoke. | 16-P:L480-L493 | P14-E006, P14-E008, P14-E017 |
| SD-P14-004 | **QUALIFY** “~100% reliability” and Agent Zero citation: source emphasizes cua replay_env and excludes openclaw framing; no quantitative threshold. | 16-P:L491-L498 | P14-E003, P14-A002 |

### P-19 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-P19-001 | **CORRECT** VACUUM backups, restart continuation, post-upgrade probes, and compaction split/successor invariants are Tier-2; Tier-1 is fail-closed config lifecycle + doctor detect/repair/lint. Summary flattens all as peer E-rows. | 16-P:L770-L816 | P19-E005–P19-E006, P19-C001 |
| SD-P19-002 | **ADD** FORK refuse-to-auto-repair beyond LKG, ABORT on silent unknown-key admit / full-env copy, and P-10 vault as secret home vs config. | 16-P:L787-L816 | P19-E014–P19-E016 |
| SD-P19-003 | **ADD** exact symbols lifecycle.rs + doctor_cmd.rs, CI-facing no-frontend justification, and path-jail case-fold win32 detail. | 16-P:L800-L810 | P19-E004, P19-E008–P19-E010 |
| SD-P19-004 | **FLAG** INTEGRATION_SMOKE references mid-turn resume while resume is Tier-2 deferred — execution must either promote resume or narrow the smoke. | 16-P:L805-L816 | P19-E018, P19-C001 |

### P-24 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-P24-001 | **ADD** Maestro admission-fail fallback to P-12 journaled step list, perception_degraded on empty RN a11y, and never-substitute-rnw_web-for-iOS FORK rules. | 16-P:L620-L678 | P24-E020 |
| SD-P24-002 | **ADD** ABORT set (physical device, host escape, missing label, unshipped H-14 B0), production_caller_check on mobile modules, and vertical-slice same-changeset hook across CLI/task_runner/canvas. | 16-P:L656-L672 | P24-E021, P24-E023, P24-E027 |
| SD-P24-003 | **ADD** evidence-leg negative arm (Maestro green / evidence FAIL) and redteam Maestro-as-verdict patch already in source ON_FAIL. | 16-P:L648-L670 | P24-E022, P24-E025 |
| SD-P24-004 | **EXPAND** RECON list (Maestro license/checksum, Expo data-oid, uiautomator fidelity, CDP precedents) beyond summary A-rows. | 16-P:L674-L678 | P24-A001–P24-A003 |

## Exhaustiveness, zero-omission, and duplicate audit

- Source coverage: **10/10 files**, **918/918 logical lines**, **122,607/122,607 bytes** read and reconciled; source SHA mismatches **0**.
- Atomic source ledger: **269 unique atoms** = **220 explicit**, **000 inferred**, **042 ambiguous**, **007 conflicting/corrective**.
- Per-file atom counts: P-1 **035**; P-2 **029**; P-6 **025**; P-7 **025**; P-9 **027**; P-10 **025**; P-11 **025**; P-14 **023**; P-19 **024**; P-24 **031**. Sum **269**.
- Relational interaction records: **22** (`IX-001`–`IX-022`, including 1 inferred dashboard-surface interaction). Summary-delta records: **038** (`SD-*`). Ambiguity/conflict register: **16** (`AC-001`–`AC-016`). These records reference source atoms and are not double-counted as features.
- ID audit: every atomic table row has exactly one unique ID matching `P1/P2/P6/P7/P9/P10/P11/P14/P19/P24-[EIAC]nnn`. Duplicate atomic IDs **0**; missing/blank atomic IDs **0**; nonconforming atomic IDs **0**.
- Semantic duplicate audit: repeated premise/scope/wiring/grounding/acceptance statements were merged into one atom with multiple anchors. Interactions, conflict register, and summary deltas reference atoms instead of cloning them. Duplicate feature atoms found **0**.
- Omission audit: every source was checked against capabilities/actions/surfaces; workflows/states; schemas/data/lifecycle; flags/defaults/config; integrations/events; permissions/security/privacy/compliance; failures/fallback/retry/recovery; determinism/idempotency/concurrency; observability/evidence; tests/acceptance/done gates; constraints/non-goals; dependencies/status/deferred. Each per-file ledger explicitly names absent dimensions rather than inventing them. User-facing CLI verbs, flags, inbox/canvas/doctor/device-frame surfaces were recorded as separate SURFACE atoms rather than folded into backend bullets.
- Interpretation audit: inferred, ambiguous, and conflicting/corrective atoms are visibly separated by ID/class. No inference is promoted to a requirement. Summary-only reference candidates for P-9/P-10/P-11 were replaced by full numbered-source inventories and marked as entire-unit ADDs in the delta section.
- Manifest-boundary audit: the only original sources opened were the ten exact DEEP-P-002 manifest rows. The only orientation artifact consulted was `16-P-SUMMARY_INDEX.discovery.md`.
- Output metrics after finalization: `output_lines=0543`; `output_bytes=088382`.
- Canonical normalized output SHA-256: 2b5024e77577a61c8db81a97fbc7a1c306c8291b06822fb54166e38a2bb73cb0.
- Hash rule: compute SHA-256 over the exact final UTF-8 bytes including the final newline after replacing only the 64 hexadecimal characters on the `Canonical normalized output SHA-256` line with 64 ASCII zeroes. Fixed-width output metrics and digest substitution avoid impossible raw self-reference.
