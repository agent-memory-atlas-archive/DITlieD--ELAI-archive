# P-plan exhaustive feature discovery

Source under discovery: `docs/thefinalglance/P-plan/P-SUMMARY_INDEX.md`

Orientation only: `docs/thefinalglance/ELAI-SPEC-OVERVIEW.md`

This document inventories what the P summary explicitly specifies. It does not treat plan prose such as “ships” or “ADDS” as evidence that code is implemented. The overview itself calls the corpus a design map rather than a build ledger. Original P detail files were deliberately not read for this pass; they are enumerated in the deeper-source manifest for later agents.

## Evidence labels

- **Explicit (`E`)**: directly stated in the P summary, including named fields, enums, commands, rows, ordering, negative gates, dependencies, and status qualifiers.
- **Inference (`I`)**: a conservative implication needed to connect explicit statements; never promoted to a requirement without a deeper-source check.
- **Ambiguous (`A`)**: missing, internally inconsistent, threshold-free, recon-gated, or otherwise unresolved in this summary.

## Coverage

- The complete 530-line P summary was read: 530/530 lines, 100% byte-range coverage of the assigned source.
- The 592-line overview was read only to orient the P layer in the whole system.
- The summary claims “all 25 sessions” and its header names five tracks, but it contains only 22 session sections: P-1..P-8, P-12..P-25. Section-level coverage is therefore 22/25 declared sessions (88%).
- P-9, P-10, and P-11 have detail files on disk and are referenced by later P units, but their own sections and the `COMMAND-CENTER` track body are absent from this summary. They are missing-summary candidates, not silently reconstructed features.
- Present section count by track: PERSONAL-INCARNATION 8; COMPUTER-USE-STEAL 7 (P-12..P-16 plus addenda P-23/P-24); OPENCLAW-STEAL 6; CREATIVITY-FALSIFIER 1; COMMAND-CENTER 0.
- Header chronology: P-1..P-8 generated 2026-07-07; missing-body command-center P-9..P-11 dated 2026-07-08/09; steal-track P-12..P-22 dated 2026-07-11; P-23/P-24 are 2026-07-11 addenda; P-25 says authored 2026-07-14.
- Explicit atomic requirement/behavior count: 629 (24 plan-wide plus 605 unit-level), with 6 reference-derived inference candidates and 63 separately labeled ambiguities/recon gaps.
- No original/detail plan was used to fill a gap. All reference-derived candidates are isolated below.

### Source anchors

| Anchor | Lines | Scope |
|---|---:|---|
| P premise | 1-18 | Header, source-design pointer, grain lift, product/proof status |
| P-1 | 21-34 | App fleet |
| P-2 | 35-47 | Ports/event bus |
| P-3 | 48-61 | Persona split/export |
| P-4 | 62-76 | Living stores |
| P-5 | 77-91 | Ceremony/operator surface |
| P-6 | 92-105 | Privacy lattice |
| P-7 | 106-121 | Operating profile |
| P-8 | 122-138 | Small-team world |
| Personal cross-cutting | 139-150 | Anti-fork, metrics, execution timing |
| Computer-use premise | 151-158 | Comparator source and gap |
| P-12 | 159-185 | Desktop operator |
| P-13 | 186-207 | Perception provenance |
| P-14 | 208-230 | Crystallized replay |
| P-15 | 231-252 | Case reuse |
| P-16 | 253-273 | Install catalog |
| P-23 | 274-297 | Co-browse addendum |
| P-24 | 298-329 | Mobile addendum |
| OpenClaw premise | 330-338 | Comparator source and steal boundary |
| P-17 | 339-361 | Provider failover |
| P-18 | 362-386 | Durable TaskSpec/approval binding |
| P-19 | 387-412 | Config/ops/backup |
| P-20 | 413-436 | Context/memory lifecycle |
| P-21 | 437-462 | Gate credibility |
| P-22 | 463-487 | Control plane/channels |
| Steal-track cross-cutting | 488-498 | Proof feeds, containment, deferred red-team |
| P-25 | 499-530 | Creativity falsifier |

## Plan-wide explicit capability and doctrine

- **P-G-E01** — P lifts the existing trust spine one grain: changeset to app, task to fleet, seat to world, and org to household.
- **P-G-E02** — The product goal is one personal world of nodes and connected systems whose contents are protected, organized, and committed.
- **P-G-E03** — The intended scale ranges from one solo toy, through connected small projects, to a large multi-person project.
- **P-G-E04** — The eight initial personal-incarnation slices are described as approximately 80% composition of already-specified subsystems.
- **P-G-E05** — P’s differentiator at every grain is the combined use of receipts, taint, replay, cant-lie enforcement, and wiring gates.
- **P-G-E06** — P is classified as a product-surface letter with no direct PROOF_SPINE number; it is owner-acknowledged.
- **P-G-E07** — P is authored now but is to execute only after the proof spine under A-7 scope honesty.
- **P-G-E08** — The anti-fragmentation rule forbids a second lattice or scheduler; authoring must pass N-2 and reuse N-4/N-6/N-11.
- **P-G-E09** — The trust ceremony must be measured at the toy tier and be near-zero-cost or GAP-5 fails.
- **P-G-E10** — Process adaptation may loosen only from measured false-positive evidence belonging to that same user, never from user affect.
- **P-G-E11** — Substrates ship as code, while personas, packs, and profiles remain data.
- **P-G-E12** — Per-user, per-recipient, and per-seat code forks are forbidden.
- **P-G-E13** — Every P unit must expose its measured number from day one; default-off features without a number are treated as the whole-feature falsifier.
- **P-G-E14** — Named personal-track measures are fleet-health green percentage, wired-edge count, exported-tool re-verification success rate, toy-tier gate tax, profile-promotion false-positive rate, and merge determinism.
- **P-G-E15** — “No number, no claim” is the KD-1 rule for P.
- **P-G-E16** — User habits live in journaled data; G-10 monitors the model and B-37 swaps it.
- **P-G-E17** — Per-app generalization of the G-10 drift canary is only asserted and is explicitly left as P-1 recon work.
- **P-G-E18** — Computer-use comparators are treated as complementary operator backends, not rivals; a raw-capability winner can be vendored beneath `elai wrap` and ELAI’s taint, journal, and gates.
- **P-G-E19** — Computer-use exposed two declared gaps: no general desktop/GUI operator lane and no concrete agentic-run-to-replayable-artifact lane.
- **P-G-E20** — OpenClaw mechanics may be stolen as designs only, never as code.
- **P-G-E21** — P retains ELAI’s seeded determinism, replay verification, off-agent runtime gating, claim grounding, and prompt-injection boundary rather than adopting comparator trust semantics.
- **P-G-E22** — P does not need to beat comparator breadth; it may contain capability winners as vendored, gated backends.
- **P-G-E23** — P-14/P-15 feed PS-5, P-17 feeds PS-4, and P-21 feeds PS-6; all other computer-use/OpenClaw units declare no proof-spine contribution.
- **P-G-E24** — Deep red-team work for the steal tracks is deferred to each unit’s execution preflight; each detail file is expected to retain a surviving-attack seed.

## Exhaustive per-unit inventory

### P-1 — app-fleet layer (health-tick + gated auto-repair) [KEYSTONE]

Metadata: PERSONAL-INCARNATION; lever `fleet-grain-over-taskspec`; MODERATE/index-derived and PLAN-asserted grounding; approximately 80% composition; effort M-L; Wave 1.

Explicit atomic inventory:

- **P1-E01** — Treat each of the user’s bespoke apps as a first-class object rather than only a collection of tasks.
- **P1-E02** — Add an `AppManifest`.
- **P1-E03** — `AppManifest` records the app purpose.
- **P1-E04** — `AppManifest` records data namespaces.
- **P1-E05** — `AppManifest` records ports.
- **P1-E06** — `AppManifest` records the smoke check.
- **P1-E07** — `AppManifest` records RRA lineage.
- **P1-E08** — `AppManifest` records a dependency manifest.
- **P1-E09** — Add a `FleetRegistry` for the app objects.
- **P1-E10** — Implement a fleet health tick as an O-2 `TaskSpec`; do not create a separate scheduling mechanism.
- **P1-E11** — Generalize G-10 drift canaries to operate per app.
- **P1-E12** — Point F-9 dependency-rot checking at each app manifest.
- **P1-E13** — A red health tick proposes a repair.
- **P1-E14** — A proposed repair must pass the same wiring and claim gates used elsewhere.
- **P1-E15** — Auto-repair is gated; a red tick must never become a silent patch.
- **P1-E16** — Render the red state human-first through N-7.
- **P1-E17** — Add the orchestrator `AppManifest` table, `FleetRegistry`, and health-tick `TaskSpec` factory.
- **P1-E18** — Expose the fleet surface through the `elai fleet` CLI.
- **P1-E19** — Registering an app with a broken smoke and advancing the task-runner tick must land exactly one `fleet_health_tick` row off-agent.
- **P1-E20** — The health row contains `{app_id, verdict=red, red_reasons}`.
- **P1-E21** — A red tick creates an N-7 inbox card.
- **P1-E22** — A green tick that still creates a card fails acceptance.
- **P1-E23** — A red tick with empty `red_reasons` fails acceptance.

Interactions: extends O-2 `TaskSpec` and `DashboardInbox`; generalizes G-10; consumes F-9/F-5, L-14 RRA, N-7, and the M-4 pattern.

Ambiguity/recon:

- **P1-A01** — The per-app G-10 canary generalization is asserted but not designed; P cross-cutting explicitly leaves it for recon.
- **P1-A02** — The index names `elai fleet` but does not enumerate its subcommands or config schema.

### P-2 — inter-tool ports + world-grain event bus

Metadata: PERSONAL-INCARNATION; lever `typed-ports-over-canvas`; MODERATE/index-derived and PLAN-asserted grounding; mostly composition with a genuinely new port-schema registry; effort M; Wave 2; depends on P-1 and O-4.

Explicit atomic inventory:

- **P2-E01** — Extend `AppManifest` with a `PortSchema`.
- **P2-E02** — Add a `PortRegistry`.
- **P2-E03** — Let the O-4 canvas bind typed tool-output ports to typed tool-input ports.
- **P2-E04** — Every tool-to-tool edge carries an N-6 provenance tier.
- **P2-E05** — Every tool-to-tool edge carries a G-1 taint tier.
- **P2-E06** — Render an output with zero consumers as visibly disconnected.
- **P2-E07** — Use M-5 `elai-verify` conformance to admit a third-party tool as a mountable node.
- **P2-E08** — Do not introduce a live broker or separate runtime.
- **P2-E09** — Keep invocation dispatch in the FSM under O-4 doctrine.
- **P2-E10** — Define the “event bus” as a typed-edge projection plus journaled invocation rows.
- **P2-E11** — Add a compatibility checker alongside `PortSchema` and `PortRegistry` in the orchestrator.
- **P2-E12** — Expose `elai fleet wire` and `elai fleet ports`.
- **P2-E13** — Render the port/edge view through O-4 at Tier 2.
- **P2-E14** — A successful bind lands one `tool_port_edge` row.
- **P2-E15** — The edge row contains `{src.out, dst.in, n6_tier, taint_tier}`.
- **P2-E16** — An incompatible wire is refused and lands no edge row.
- **P2-E17** — An edge missing its taint tier fails acceptance.

Interactions: extends P-1 `AppManifest` and O-4 canvas edges; consumes N-6, G-1/G-2, M-5/M-6.

Ambiguity:

- **P2-A01** — “Incompatible wire lands NO row” is explicit for the edge row, but the summary does not say whether a separate refusal/audit row must exist.
- **P2-A02** — The compatibility relation and schema-version behavior are not defined in the summary.

### P-3 — persona/mechanism split + hand-this-box-to-a-friend export

Metadata: PERSONAL-INCARNATION; lever `mechanism-vs-persona-factoring`; MODERATE/index-derived and PLAN-asserted grounding; bundle is a build while factoring is doctrine; effort M; Wave 3; depends on P-1/P-2.

Explicit atomic inventory:

- **P3-E01** — Factor transferable app mechanism separately from recipient-specific persona data.
- **P3-E02** — Add an `AppBundle`.
- **P3-E03** — The bundle includes the mechanism.
- **P3-E04** — The bundle includes the manifest.
- **P3-E05** — The bundle includes an RRA receipt.
- **P3-E06** — The bundle includes G-9-redacted provenance.
- **P3-E07** — Persona rules rows are data and are excluded from the bundle.
- **P3-E08** — Persona memory namespaces are data and are excluded from the bundle.
- **P3-E09** — Persona thresholds are data and are excluded from the bundle.
- **P3-E10** — Persona tokens are data and are excluded from the bundle.
- **P3-E11** — Regenerate the persona layer on the recipient’s seat.
- **P3-E12** — `elai app import` re-verifies a bundle through M-5.
- **P3-E13** — Refuse to mount a tampered bundle.
- **P3-E14** — Apply the L-12/Skillspector admission family to app bundles.
- **P3-E15** — Friend-to-friend sharing and marketplace sharing use the same admission gate.
- **P3-E16** — Add orchestrator `AppBundle`, export/import, M-5 verification, and G-9 redaction wiring.
- **P3-E17** — Expose `elai app export` and `elai app import`.
- **P3-E18** — Export lands an `app_export` row.
- **P3-E19** — A tampered import lands an `app_import` refusal and does not mount.
- **P3-E20** — A clean import lands a mount row.
- **P3-E21** — Any tampered bundle that mounts fails acceptance.
- **P3-E22** — Any exported bundle that leaks a secret fails acceptance.

Interactions: consumes P-1/P-2 mechanism, M-5/M-6 verification/format, L-12 admission, G-8/G-9, FS-1/L-13, and E-5/P-7 persona regeneration.

Ambiguity:

- **P3-A01** — The exact bundle format/version/signature fields are absent from the summary.
- **P3-A02** — Persona regeneration inputs and whether every excluded category is regenerated or defaulted are not specified.

### P-4 — living external brains

Metadata: PERSONAL-INCARNATION; lever `mount-LIVE-not-static`; WEAK-until-RECON; composition plus a per-store contract; effort M-L; Wave 2; depends on O-3/P-6; explicitly GATED.

Explicit atomic inventory:

- **P4-E01** — Extend static O-3 vault mounts into live stores.
- **P4-E02** — Cover Obsidian under concurrent plugin edits as a live-store class.
- **P4-E03** — Cover Notion as a live-store class, subject to source recon.
- **P4-E04** — Cover SQLite as a live-store class.
- **P4-E05** — Add one `LivingStoreContract` per store class.
- **P4-E06** — A living-store contract defines watching.
- **P4-E07** — A living-store contract defines a change feed.
- **P4-E08** — Ingest through the contract is tagged through F-5/N-6.
- **P4-E09** — Writes are allowed only inside namespaces claimed through O-3.
- **P4-E10** — Lift B-35’s context pipeline to living stores.
- **P4-E11** — Make the D-5/O-3 life denylist a hard routing wall through P-6.
- **P4-E12** — Block implementation until the Obsidian/Notion API is source-grounded.
- **P4-E13** — Add `LivingStoreContract`, an Obsidian watch adapter, and an O-3-claim-gated writer in `crates/knowledge`.
- **P4-E14** — Expose `elai brain mount` and `elai brain sync`.
- **P4-E15** — Editing a claimed-namespace note lands `vault_ingest {note_path, n6_tier, staleness_ttl}`.
- **P4-E16** — Attempting an unclaimed write lands a `claim_violation` row.
- **P4-E17** — Mutating a user-owned note outside the claimed namespace fails acceptance.
- **P4-E18** — Ingest missing either provenance tier or staleness TTL fails acceptance.

Interactions: extends O-3; consumes B-35, F-5/N-6, D-5, P-6, and E-20.

Ambiguity/recon:

- **P4-A01** — The digest says Obsidian/Notion surfaces are training-only and ungrounded, yet says the Notion MCP is a live grounding source “this session”; the hard `WEAK-until-RECON`/`GATED` label still controls.
- **P4-A02** — Only an Obsidian adapter is named in integration despite three store classes being named.
- **P4-A03** — Watch ordering, conflict handling, deletions, and writeback semantics are not summarized.

### P-5 — non-coder operator surface + graduated ceremony

Metadata: PERSONAL-INCARNATION; lever `graduated-ceremony-profile`; MODERATE/index-derived and PLAN-asserted grounding; approximately 90% composition; effort M; Wave 1.

Explicit atomic inventory:

- **P5-E01** — Add a non-coder operator surface at the P-1 fleet grain.
- **P5-E02** — Add a `CeremonyProfile` over K-5/K-4.
- **P5-E03** — `CeremonyProfile` has `toy`, `personal`, `project`, and `critical` tiers.
- **P5-E04** — At `toy`, gates execute in observe-only mode.
- **P5-E05** — Toy gates still run.
- **P5-E06** — Toy gate verdicts are still journaled.
- **P5-E07** — Toy gates do not block on warnings.
- **P5-E08** — No tier may silently skip a gate.
- **P5-E09** — Collapse toy-facing receipts to one line.
- **P5-E10** — The one-line receipt must index/resolve to the full journal.
- **P5-E11** — Render a red fleet tick as one sentence plus one gated repair button through N-7.
- **P5-E12** — Measure the toy-tier gate tax and require it to be near zero.
- **P5-E13** — Add orchestrator `CeremonyProfile`, K-5 gate-mode wiring, and one-line receipt rendering.
- **P5-E14** — Expose `elai profile set` and `elai profile tax`.
- **P5-E15** — Setting a tier lands a `ceremony_profile_set` row.
- **P5-E16** — The underlying verdict is journaled at both toy and critical tiers.
- **P5-E17** — Critical blocks on WARN where toy does not.
- **P5-E18** — `elai profile tax --tier toy` is the named tax measurement entry point.
- **P5-E19** — Dropping the journal row at toy fails acceptance.

Interactions: extends K-5/K-4 and points N-7 at P-1; consumes GD-4/W-2.

Ambiguity:

- **P5-A01** — “Near-zero” has no numeric ceiling, workload, or confidence interval in the summary.
- **P5-A02** — Blocking behavior for `personal` and `project` is not enumerated.

### P-6 — life-data privacy lattice

Metadata: PERSONAL-INCARNATION; lever `out-flow-privacy-dimension`; MODERATE/index-derived and PLAN-asserted grounding; composition by extending N-6; effort M; Wave 2; depends on N-6/O-3.

Explicit atomic inventory:

- **P6-E01** — Extend the one N-6 lattice with a privacy domain dimension; do not create a second lattice.
- **P6-E02** — The domain enum is `life | work | shared`.
- **P6-E03** — Add per-tool `read_grants`.
- **P6-E04** — Read grants are default-deny.
- **P6-E05** — Enforce read grants at the O-3 vault-read boundary.
- **P6-E06** — Enforce the same grants at E-20 routing.
- **P6-E07** — Treat life-data egress as the mirror of G-1 taint: owned bytes flowing out versus untrusted bytes flowing in.
- **P6-E08** — Render every grant through N-7.
- **P6-E09** — Journal every grant.
- **P6-E10** — Make every grant revertible through H-49.
- **P6-E11** — Make a second privacy-lattice table an authoring failure under A-1/N-2.
- **P6-E12** — Add the domain dimension, grants, and O-3 read enforcement in `crates/knowledge`.
- **P6-E13** — Expose `elai privacy grant`, `revoke`, and `show`.
- **P6-E14** — Granting a tool access lands a `privacy_grant` row.
- **P6-E15** — A read inside the granted namespace succeeds.
- **P6-E16** — A read in another ungranted namespace lands `privacy_read_refusal`.
- **P6-E17** — N-2 must confirm that exactly one lattice exists.
- **P6-E18** — An ungranted read that succeeds fails acceptance.
- **P6-E19** — A second lattice table fails acceptance.

Interactions: extends N-6 and consumes N-2, G-1/G-9, E-20, O-3, N-7, and H-49.

Ambiguity:

- **P6-A01** — Grant scope beyond the example namespace, such as expiry, nested namespaces, and cross-domain precedence, is not specified.

### P-7 — per-user operating profile

Metadata: PERSONAL-INCARNATION; lever `measured-asymmetric-process-adaptation`; MODERATE/index-derived and PLAN-asserted grounding; composition on N-3/H-17/H-18; effort M-L; Wave 3; depends on N-3 being landed.

Explicit atomic inventory:

- **P7-E01** — Adapt process separately from E-5 content memory.
- **P7-E02** — Add an `OperatingProfile`.
- **P7-E03** — Store autonomy per domain.
- **P7-E04** — Store strictness per domain.
- **P7-E05** — Store verbosity per domain.
- **P7-E06** — Learn and promote profile changes through N-3/H-17/H-18 measurements rather than subjective affect.
- **P7-E07** — Tightening a profile is free to apply.
- **P7-E08** — Loosening requires measured false-positive evidence.
- **P7-E09** — Loosening evidence must come from the same user’s own runs.
- **P7-E10** — Raw counts alone cannot authorize loosening.
- **P7-E11** — Another user’s data cannot authorize loosening.
- **P7-E12** — Observe profile effects before promotion.
- **P7-E13** — Journal every profile change/refusal.
- **P7-E14** — Make profile changes revertible through H-49.
- **P7-E15** — Define `solo toy`, `connected`, and `team` as defaults over one engine, not separate products.
- **P7-E16** — Add the profile and an asymmetry-guarded change function to the orchestrator.
- **P7-E17** — Route loosen promotion through H-17/H-18.
- **P7-E18** — Expose `elai profile show`, `tighten`, and `loosen`.
- **P7-E19** — A loosen request without evidence is refused.
- **P7-E20** — A loosen request with measured own-run FP evidence is promoted through H-18.
- **P7-E21** — Tightening applies directly.
- **P7-E22** — Refusal, promotion, and revert rows carry an `evidence_ref` where applicable.

Interactions: extends N-3/H-17/H-18 to process metrics, applies M-2 asymmetry, and consumes GD-4/H-49, E-5, P-5/P-2/M/P-8.

Ambiguity:

- **P7-A01** — False-positive metric definitions, promotion thresholds, observation duration, and default values are absent.
- **P7-A02** — The exact domain vocabulary is not restated here and may rely on P-6/N-6.

### P-8 — small-team collaboration grain

Metadata: PERSONAL-INCARNATION; lever `shared-world-2-to-10-seats`; MODERATE/index-derived and PLAN-asserted grounding; the merge kernel is explicitly open; composition-heavy plus one hard design kernel; effort L; Wave 4; depends on O-3/K-23.

Explicit atomic inventory:

- **P8-E01** — Support 2-10 humans and their agents in one shared world.
- **P8-E02** — Extend O-3 namespace claims into shared vault namespaces with N claimants.
- **P8-E03** — Lift cross-session harness locks to product-level cross-seat locks.
- **P8-E04** — Add `seat_id` provenance to every SSB row.
- **P8-E05** — Add `agent_id` provenance to every SSB row.
- **P8-E06** — Make attribution answerable as a query.
- **P8-E07** — Apply the M-3 rollup at shared-project grain.
- **P8-E08** — Build the merge kernel as a spike first.
- **P8-E09** — Merge two seat journal streams losslessly.
- **P8-E10** — Make merge deterministic.
- **P8-E11** — Preserve causal ordering across the merged streams.
- **P8-E12** — Forbid last-writer-wins merge semantics.
- **P8-E13** — Do not introduce an org control plane.
- **P8-E14** — Keep collaboration local-first.
- **P8-E15** — Add SSB-stream merge, provenance columns, and cross-seat locks across orchestrator/knowledge.
- **P8-E16** — Expose `elai world join`, `merge`, and `whodid`.
- **P8-E17** — A merge lands `seat_journal_merge {seats, event_count, order_hash}`.
- **P8-E18** — Merging the same two streams in either direction produces an identical `order_hash`.
- **P8-E19** — A dropped event fails acceptance.
- **P8-E20** — A direction-dependent `order_hash` fails acceptance.
- **P8-E21** — A lost concurrent write fails acceptance.

Interactions: extends O-3 to N seats; consumes K-23, I-2, M-2/M-3, harness cross-session locks, and Phase 3H merge.

Ambiguity/open design:

- **P8-A01** — The deterministic causal merge algorithm is explicitly an open hard kernel.
- **P8-A02** — Identity enrollment, revocation, transport, offline reconciliation, and conflict presentation are not defined in this summary.

## Missing COMMAND-CENTER sections: reference-derived candidates only

These are not counted as discovered P-9/P-10/P-11 feature inventories because the summary contains no such sections.

- **P9-I01** — Later sections identify P-9 as a connector/stage-pack format and a gated pack lane.
- **P9-I02** — P-16 says an install-catalog target is a P-9 pack and distinguishes a headless code-pack connector from a GUI fallback.
- **P9-A01** — No P-9 fields, acceptance rows, CLI, status, or complete behavior are present in the summary.
- **P10-I01** — Later sections identify P-10 as a credential vault/auth plane used by connectors.
- **P10-I02** — P-23 assigns named browser-profile custody to P-10.
- **P10-I03** — P-22 associates P-10 ingress adapters with channel-binding precedence and `dmScope` session isolation.
- **P10-A01** — No P-10 vault schema, adapter taxonomy, credential lifecycle, acceptance gate, or direct entrypoint is summarized.
- **P11-I01** — Later sections identify P-11 as workflow-from-goal with a human confirm gate; P-18 hardens its “MOVE 3” through drift binding.
- **P11-A01** — No P-11 graph-synthesis rules, UI/API, events, negative gates, or grounding/status block is present.

### P-12 — desktop/GUI operator lane [NEW; the honest design gap]

Metadata: COMPUTER-USE-STEAL; lever `gui-actuation-under-gates`; MODERATE grounding (comparator source-verified, ELAI anchors PLAN-asserted); genuinely new; effort L; Wave CU-2; no proof-spine number; owner-acknowledged 2026-07-11.

Explicit atomic inventory:

- **P12-E01** — Add a general desktop/GUI `OperatorLane` for workflows such as driving a GUI app or testing a game build.
- **P12-E02** — Execute the lane inside a disposable sandbox.
- **P12-E03** — Vendor the isolation substrate from CUA’s Windows Sandbox, Lume, or Docker mechanisms rather than reimplementing it.
- **P12-E04** — Treat the vendored CUA substrate as MIT-licensed at summary time.
- **P12-E05** — Supply ELAI fail-closed gates because the comparator’s in-loop safety is only a commented-out TODO.
- **P12-E06** — Consume the P-13 perception feed.
- **P12-E07** — Fire the OmniParser Tier-3 pixel fallback only when the accessibility tree is empty or incomplete.
- **P12-E08** — Bind every coordinate action to the exact SHA-256 hash of the image delivered to the model.
- **P12-E09** — Authorize that coordinate action only while the exact image remains in context.
- **P12-E10** — Invalidate the frame epoch when compaction removes the image from context.
- **P12-E11** — Never retarget an action to display 0 if its original display/frame is unavailable.
- **P12-E12** — Use a two-phase arm/commit sequence.
- **P12-E13** — Journal the capability grant before exposing the actuation capability.
- **P12-E14** — Apply lease TTL expiry both during periodic sweep and inline at use.
- **P12-E15** — Capture a bounded screen recording for every operator action.
- **P12-E16** — Hash that screen recording into an off-agent evidence row.
- **P12-E17** — Treat every operator action as a G-1 taint source.
- **P12-E18** — Mark the action as an operator-lane action.
- **P12-E19** — Never make GUI actuation load-bearing infrastructure on which workflow correctness depends.
- **P12-E20** — Reuse 4F isolation, P-13 perception, and G-1 taint rather than reimplementing them.
- **P12-E21** — Compare CUA’s pure-function `OperatorNormalizer` repair against H-13 `sanitizeAction` rather than assuming equivalence.
- **P12-E22** — Add `crates/orchestrator/src/operator/` with lane, sandbox adapter, frame binder, and arm/lease components.
- **P12-E23** — Add `operator_action` as an O-2 `TaskSpec` stage kind.
- **P12-E24** — Expose `elai operator run`.
- **P12-E25** — Render an operator node on the O-4 canvas.
- **P12-E26** — Pair operator-action rows with a screen-recording thumbnail in an existing drawer.
- **P12-E27** — Re-verify at edit time that OmniParser `icon_detect` weights are AGPL and Florence captions are MIT.
- **P12-E28** — A successful sandbox click lands `operator_action {frame_hash, action, lease_id}`.
- **P12-E29** — The same action lands one `operator_frame_bind` row.
- **P12-E30** — The recording artifact hash must match the journal row.
- **P12-E31** — Reject a coordinate action against a stale or compacted frame with no actuation.
- **P12-E32** — Actuating on a stale `frame_hash` fails acceptance.
- **P12-E33** — Exposing capability before the grant row lands fails acceptance.

Interactions: sibling to E-14; extends J-1/J-8 through P-13; consumes G-1/G-2, 4F, H-13, O-2, and O-4.

Ambiguity/follow-up:

- **P12-A01** — “Incomplete” accessibility tree detection is not defined here; P-13 sharpens it to “provably incomplete.”
- **P12-A02** — The exact bounded-recording duration/size and lease duration are unspecified.
- **P12-A03** — The existing frontend drawer is deliberately left to an edit-time W-4 decision.
- **P12-A04** — The license assertion is explicitly provisional and must be rechecked.

### P-13 — perception provenance + set-of-marks id-indirection

Metadata: COMPUTER-USE-STEAL; lever `id-indirection-over-coords`; STRONG comparator grounding with cited OmniParser lift 70.5% to 93.8%; ELAI anchors PLAN-asserted; extension; effort M; Wave CU-1; no proof-spine number.

Explicit atomic inventory:

- **P13-E01** — Extend J-1 perception beyond conditioned bytes and AXTree/DOM text so the model can point safely.
- **P13-E02** — Apply set-of-marks ID indirection to the accessibility-tree node list.
- **P13-E03** — Have the model/VLM return an integer element ID rather than raw geometry.
- **P13-E04** — Keep element geometry owned by the deterministic harness.
- **P13-E05** — Attach a source tag to every element.
- **P13-E06** — The source-tag enum is `axtree_exact | ocr_text | vlm_caption | pixel_yolo`.
- **P13-E07** — Treat the source tag as the element’s trust boundary for G-1 taint/grounding.
- **P13-E08** — Add an interactability flag.
- **P13-E09** — Filter the VLM candidate set through the interactability flag.
- **P13-E10** — Use captions only as advisory data for unnamed canvas/game icons.
- **P13-E11** — Fuse OCR text into element labels asymmetrically.
- **P13-E12** — Fire Tier-3 pixel/YOLO perception only when the accessibility tree is empty or provably incomplete.
- **P13-E13** — Cover games, WebGL, and native dead-UIA surfaces through that guarded fallback.
- **P13-E14** — Never obtain counts from a VLM; preserve the J-1 counting law.
- **P13-E15** — Never let a per-crop caption stand in for element identity.
- **P13-E16** — Treat a mis-sourced element as a J-77 ghost citation.
- **P13-E17** — Extend J-1/J-8 scaffold rows with source and box-ID fields, preferring reuse and adding at most one net row.
- **P13-E18** — Put the guarded Tier-3 adapter in the existing `crates/orchestrator/src/perception/` seam.
- **P13-E19** — Render the source-tagged element breakdown in the existing perception tile.
- **P13-E20** — Add no CLI; expose perception as a service seam for E-14 and P-12.
- **P13-E21** — An empty-AXTree canvas fixture produces integer IDs and a source tag for every element.
- **P13-E22** — The Tier-3 pixel leg fires exactly once for that fixture.
- **P13-E23** — The trace contains zero VLM count calls.
- **P13-E24** — “Click element 7” resolves through the ID index rather than model-supplied coordinates.
- **P13-E25** — A VLM-sourced count fails acceptance.
- **P13-E26** — An element missing its source tag fails acceptance.

Interactions: extends J-1/J-8, preserves E-14 AXTree doctrine, feeds J-77 and G-1, and serves E-14/P-12.

Ambiguity:

- **P13-A01** — The cited 70.5→93.8% is comparator evidence, not an ELAI acceptance threshold.
- **P13-A02** — The summary does not define source-tag precedence when several sources contribute to one element.
- **P13-A03** — “Asymmetric” OCR fusion is named but its overwrite/merge rule is not specified.

### P-14 — explore-then-crystallize record-to-replay lane [FEEDS PS-5]

Metadata: COMPUTER-USE-STEAL; lever `agentic-run-mints-artifact`; STRONG comparator grounding; extension; effort M-L; Wave CU-2; depends on P-13; feeds PS-5.

Explicit atomic inventory:

- **P14-E01** — Let an E-14 browser or P-12 desktop exploration run mint a deterministic replay artifact.
- **P14-E02** — Emit Playwright for browser exploration.
- **P14-E03** — Emit PyAutoGUI for desktop exploration.
- **P14-E04** — Represent the emitted script as an O-2 `TaskSpec` stage.
- **P14-E05** — Bind the script to the exact exploration run that minted it through a G-2 RRA receipt.
- **P14-E06** — Use script replay as the cheap, zero-reasoning execution mode.
- **P14-E07** — Invoke no model during replay.
- **P14-E08** — Keep the crystallized flow inside O-2 dispatch rather than adding a second dispatcher.
- **P14-E09** — Extend the existing compiled-pipeline/template-crystallizer general form rather than fork it.
- **P14-E10** — Use P-13 perception during exploration.
- **P14-E11** — Add the O-2 stage kind `crystallized_script`.
- **P14-E12** — Add a script emitter over the E-14/P-12 trace.
- **P14-E13** — Add an RRA binder for minted scripts.
- **P14-E14** — Expose `elai e2e crystallize`.
- **P14-E15** — Replay through the normal task-runner tick.
- **P14-E16** — Render a crystallized-stage node on the O-4 canvas at Tier 2.
- **P14-E17** — Minting lands `crystallized_stage_minted {script_hash, minting_run_id, rra_hash}`.
- **P14-E18** — A replay drives the real entry point with zero model calls in its trace.
- **P14-E19** — Replay must reproduce the exploration’s observable outcome.
- **P14-E20** — A script whose RRA does not bind to a real minting run fails acceptance.
- **P14-E21** — A replay that silently invokes a model fails acceptance.
- **P14-E22** — Cite CUA `replay_env` and Agent Zero’s skills/scripts convention as prior art, not copied code.

Interactions: consumes E-14, P-12, P-13, O-2/O-4, G-2, and the C crystallizer; the resulting reuse compounds PS-5.

Ambiguity:

- **P14-A01** — “~100% reliable” is narrative motivation; no quantitative replay-reliability threshold is set.
- **P14-A02** — Script portability, nondeterministic UI handling, and secret/credential parameterization are not defined here.

### P-15 — verified case-graph hardening [FEEDS PS-5]

Metadata: COMPUTER-USE-STEAL; lever `gate-the-save-not-every-run`; STRONG source grounding; extension of J-96 despite J being closed; effort M; Wave CU-3; feeds PS-5.

Explicit atomic inventory:

- **P15-E01** — Preserve J-96’s rule that a typed case node is minted only after a proven fix.
- **P15-E02** — Add Agent-S-style retrieve behavior to the verified case graph.
- **P15-E03** — Add distillation behavior to the reuse loop.
- **P15-E04** — Add fusion of retrieved case information into the next plan.
- **P15-E05** — Support narrative and episodic case tiers in that loop.
- **P15-E06** — Use embedding retrieval as part of the loop.
- **P15-E07** — Gate every save behind J-96’s execution-grounded mint-on-proof filter.
- **P15-E08** — Do not copy Agent-S’s save-every-run/ungated-commit behavior.
- **P15-E09** — Add a bBoN behavior-narrative comparative judge.
- **P15-E10** — Run the bBoN judge over journal rows, not screenshots.
- **P15-E11** — Use ground-truth action/effect deltas available in journal rows.
- **P15-E12** — Insert bBoN only as an I evidence-ranked-search tie-breaker.
- **P15-E13** — Receipts choose first; narrative only breaks receipt-level ties.
- **P15-E14** — Keep reuse behind J-96’s strict execution-grounded filter and default-off.
- **P15-E15** — Reuse J-96 memory canvas node chrome.
- **P15-E16** — A proven fix followed by recurrence retrieves and fuses the case.
- **P15-E17** — An unproven run’s summary is not saved.
- **P15-E18** — Record a `reuse-hit` field on the J-96 case row.
- **P15-E19** — Feed that hit into K-18 recurrence-hit-rate economics.
- **P15-E20** — A case saved from an unproven run fails acceptance.
- **P15-E21** — A bBoN judgment sourced from screenshots fails acceptance.

Interactions: extends J-96; obeys K-19; feeds K-18/PS-5; slots into I evidence-ranked search.

Ambiguity:

- **P15-A01** — Retrieval embedding model, tier schemas, tie conditions, and fusion policy are not summarized.
- **P15-A02** — “Default-off” coexists with P’s default-off graveyard doctrine; promotion/on-switch criteria must come from the detail plan or K-18/K-19.

### P-16 — gated install catalog

Metadata: COMPUTER-USE-STEAL; lever `capability-on-demand-gated`; MODERATE grounding; extension; effort M; Wave CU-3; depends on missing-summary P-9/P-10; no proof-spine number; owner-acknowledged 2026-07-11.

Explicit atomic inventory:

- **P16-E01** — Add a named install catalog so capability-on-demand feels immediate.
- **P16-E02** — Each catalog row carries `tool`.
- **P16-E03** — Each catalog row carries `source`.
- **P16-E04** — Each catalog row carries `mirror`.
- **P16-E05** — Each catalog row carries `alias`.
- **P16-E06** — Each catalog row carries `license_tag`.
- **P16-E07** — Add an `installPolicy` admission decision behind the existing L-12/Skillspector/N-6/hidden-until-verified family.
- **P16-E08** — Resolve an install request to a P-9 headless code-pack connector when one exists.
- **P16-E09** — Prefer deterministic, journalable, replayable headless interfaces such as `blender --background --python`.
- **P16-E10** — Use P-12 GUI driving only as the last resort.
- **P16-E11** — Never use GUI driving as the install mechanism when a headless CLI exists.
- **P16-E12** — Reuse L-12 admission; do not add a second scanner.
- **P16-E13** — Do not copy OpenClaw’s advisory regex scanner.
- **P16-E14** — Add the install-catalog table and the policy check on the existing L-12 path.
- **P16-E15** — Expose `elai connector install <tool>`.
- **P16-E16** — Render catalog data and admission verdict in the connector surface at Tier 2.
- **P16-E17** — Every install attempt lands `install_catalog_admit {tool, source, license_tag, verdict}`.
- **P16-E18** — A clean catalog item mounts a P-9 headless pack.
- **P16-E19** — A flagged item is refused and no install occurs.
- **P16-E20** — Bypassing the L-12 admission family fails acceptance.
- **P16-E21** — GUI-driving an install despite a headless CLI fails acceptance.

Interactions: extends P-9, uses P-10 auth, L-12, N-6, and P-12.

Ambiguity:

- **P16-A01** — Its two direct prerequisites, P-9/P-10, have no sections in this summary.
- **P16-A02** — Source/mirror integrity, catalog signing/update, alias collisions, and license-policy vocabulary are not summarized.

### P-23 — journaled co-browse lane (CU-4 addendum)

Metadata: COMPUTER-USE-STEAL; lever `co-browse-with-receipts`; STRONG comparator grounding and MODERATE ELAI-anchor grounding; extension of H-15/E-14; effort M; Wave CU-4; depends on P-13; no proof-spine number; owner-commissioned 2026-07-11.

Explicit atomic inventory:

- **P23-E01** — Add a general co-browse surface as a canvas browse node over the existing E-14 engine.
- **P23-E02** — Tier 1 performs read-only extraction for requests such as pulling up documentation.
- **P23-E03** — Tag Tier-1 page extraction as tainted ingress.
- **P23-E04** — Route read-only pointing/deixis through H-15.
- **P23-E05** — Tier 2 performs co-browse through a CDP screencast.
- **P23-E06** — Keep the visible pane as dumb glass; engine-owned browser state remains authoritative.
- **P23-E07** — Interpose exactly one `InputProxy` with a private engine input handle.
- **P23-E08** — Make bypassing `InputProxy` structurally uncompileable.
- **P23-E09** — Journal navigation before dispatch.
- **P23-E10** — Journal clicks before dispatch.
- **P23-E11** — Journal keystrokes before dispatch.
- **P23-E12** — Tag every input with the actor, covering both user and model.
- **P23-E13** — Make session persistence a typed choice.
- **P23-E14** — Default sessions to ephemeral.
- **P23-E15** — Hold named persistent profiles in P-10 credential-vault custody.
- **P23-E16** — Gate outbound typed text through P-6 privacy.
- **P23-E17** — Show a domain-first-seen warning before outbound text reaches a newly seen domain.
- **P23-E18** — Have the model point through P-13 element IDs.
- **P23-E19** — Permit coordinate fallback only under P-12 frame binding.
- **P23-E20** — Exclude native Tauri multiwebview as the primary mechanism because its cargo flag is unstable.
- **P23-E21** — Permit native multiwebview only as a latency-triplined fork.
- **P23-E22** — Keep `InputProxy` interposed even in that fork.
- **P23-E23** — Never create a second browser substrate; thread the E-14 contract if E-14 is not executed at edit time.
- **P23-E24** — Allow a co-browse exploration to crystallize through P-14 into an RRA-bound replay script.
- **P23-E25** — Add `crates/orchestrator/src/cobrowse/{mod.rs,input_proxy.rs}`.
- **P23-E26** — Expose `elai browse <url>` and a front-door browse lane.
- **P23-E27** — Wire the CLI, front door, and H-13 canvas dispatcher in the same changeset.
- **P23-E28** — Render screencast/extraction plus an actor-colored input trail.
- **P23-E29** — The fixture run lands one `cobrowse_session` row.
- **P23-E30** — Every dispatched input has one prior actor-tagged `cobrowse_input` row.
- **P23-E31** — Check journal-before-dispatch ordering off-agent against the engine CDP log.
- **P23-E32** — An ephemeral session leaves zero profile bytes.
- **P23-E33** — Any page input without a prior row fails acceptance.
- **P23-E34** — A model coordinate action against a stale frame hash fails acceptance.
- **P23-E35** — A persistent profile outside vault custody fails acceptance.

Interactions: reuses E-14, H-13/H-15, P-13/P-14/P-12, P-10, P-6, G-1/G-2, and the J front-door router.

Ambiguity/follow-up:

- **P23-A01** — The latency tripline and conditions for the native-multiwebview fork are not quantified.
- **P23-A02** — Cookie/download/upload/pop-up/permission behavior is not summarized.
- **P23-A03** — P-10 profile-custody behavior cannot be fully traced because P-10 is absent from the summary.

### P-24 — mobile design/test lane (CU-5 addendum; deferred H-14 smartphone slice)

Metadata: COMPUTER-USE-STEAL; lever `mobile-as-a-dimension-not-a-subsystem`; STRONG external and MODERATE ELAI-anchor grounding; extension; effort L; Wave CU-5; depends on H-14/P-12/P-13/P-14/P-16; no direct proof-spine number, while Rung 3 transitively feeds PS-5 through P-14; owner-commissioned 2026-07-11.

Explicit atomic inventory:

- **P24-E01** — Model mobile as a typed dimension of existing design/test/operator lanes, not a parallel subsystem.
- **P24-E02** — Cover the React Native/Expo gap explicitly deferred by H-14.
- **P24-E03** — Make the existing `ConsumerMobileApp` product type previewable.
- **P24-E04** — Make that product type click-editable at the supported rung.
- **P24-E05** — Make that product type testable.
- **P24-E06** — Implement three rungs in cheapest-first order.
- **P24-E07** — Rung 1 adds a typed `DeviceProfile` catalog as data rows.
- **P24-E08** — Rung 1 adds a `device_frame` render variant to H-14 preview nodes.
- **P24-E09** — Rung 1 adds the same device-frame dimension to P-23 browse nodes.
- **P24-E10** — Rung 1 runs E-14 journeys across a device matrix.
- **P24-E11** — Rung 1 maintains per-profile SSIM goldens using CDP metrics emulation and in-repo capture precedent.
- **P24-E12** — Rung 2 treats an Expo development server as a frameable web origin.
- **P24-E13** — Rung 2 reuses H-14 proxying.
- **P24-E14** — Rung 2 reuses `data-oid` tagging.
- **P24-E15** — Rung 2 reuses Recast writeback over React-Native-Web output.
- **P24-E16** — A click in Rung 2 resolves to React Native component AST ranges.
- **P24-E17** — Rung 3 treats a native emulator as another P-12 sandbox backend.
- **P24-E18** — Boot the emulator from a pinned snapshot.
- **P24-E19** — Include the snapshot hash in the RRA hardware/environment pin.
- **P24-E20** — Add `mobile_axtree` as a P-13 source tag.
- **P24-E21** — Obtain mobile accessibility from UIAutomator/XCUITest accessibility data.
- **P24-E22** — Use pixel perception only when the mobile accessibility tree is empty, preserving J-1.
- **P24-E23** — Crystallize a Maestro YAML flow through P-14.
- **P24-E24** — Treat the Maestro flow as actuation-only.
- **P24-E25** — Obtain replay verdicts from the same off-agent SSB/screenshot evidence leg as the minting run.
- **P24-E26** — Provision emulator, SDK, and Maestro through P-16.
- **P24-E27** — Attach `render_source` to every mobile result row.
- **P24-E28** — The render-source enum is `web_viewport | rnw_web | native_emulator`.
- **P24-E29** — Never let a lower-rung pass discharge a higher-rung gate.
- **P24-E30** — Represent “mobile tested” as a typed claim.
- **P24-E31** — Class physical devices as ABORT because they are neither disposable nor snapshot-pinnable.
- **P24-E32** — Keep native React Native click-to-edit through a Fiber inspector as a registered Rung-3 follow-up/fork.
- **P24-E33** — Do not front-run H-14; extend its proxy, tagging, and writeback only after it exists.
- **P24-E34** — Add `crates/orchestrator/src/mobile/{device_profile.rs,emulator_backend.rs}`.
- **P24-E35** — Expose `elai e2e run --device` and `elai operator run --backend emulator`.
- **P24-E36** — Add `device_profile` to task-runner stages.
- **P24-E37** — Add the H-13 `device_frame` render variant.
- **P24-E38** — Render a phone-bezel node plus per-profile pass/fail chips.
- **P24-E39** — On systems without macOS, render iOS as an honest-gap affordance rather than silently skip it.
- **P24-E40** — The web fixture emits per-profile `axtree_snapshot` rows tagged with `device_profile` and `render_source`.
- **P24-E41** — A Rung-3 run lands `emulator_session {snapshot_hash, backend}`.
- **P24-E42** — That run crystallizes an RRA-bound Maestro flow.
- **P24-E43** — Presenting an `rnw_web` pass against a native gate fails acceptance.
- **P24-E44** — An unpinned emulator image fails acceptance.
- **P24-E45** — A VLM-sourced mobile count fails acceptance.

Interactions: extends H-14, E-14, P-12/P-13/P-14/P-16/P-23 and consumes 4I, 4D/4E, and B-42.

Ambiguity/future:

- **P24-A01** — Native Fiber-inspector click-to-edit is explicitly deferred inside a Rung-3 fork.
- **P24-A02** — Physical-device support is explicitly outside the executable lane unless its non-disposable/non-pinnable problem is redesigned.
- **P24-A03** — Device-profile fields, SSIM thresholds, matrix coverage, and Android/iOS backend support levels are not enumerated.

### P-17 — provider failover + honest routing economics [FEEDS PS-4]

Metadata: OPENCLAW-STEAL; lever `failover-semantics-package`; STRONG/source-verified comparator grounding with PLAN-asserted ELAI anchors; extension; effort M; Wave OC-1; feeds PS-4.

Explicit atomic inventory:

- **P17-E01** — Extend J’s free-API rotation pool and I-50 auth broker with a complete failure-semantics package.
- **P17-E02** — Classify rate-limit failures into their own cooldown lane.
- **P17-E03** — Escalate rate-limit cooldowns through 30 seconds, 1 minute, then 5 minutes.
- **P17-E04** — Classify billing failures into their own cooldown lane.
- **P17-E05** — Escalate billing cooldowns from 5 hours to 24 hours.
- **P17-E06** — Classify authentication failures into their own cooldown lane.
- **P17-E07** — Apply an authentication cooldown in the stated 10-to-60-minute range.
- **P17-E08** — Distinguish model-scoped cooldowns from profile-scoped cooldowns.
- **P17-E09** — Allow a sibling model on the same provider to try when only one model/lane is cooled.
- **P17-E10** — Preserve session stickiness to keep provider caches warm.
- **P17-E11** — Treat rotation as a cache-invalidation event.
- **P17-E12** — Journal that cache-invalidation event.
- **P17-E13** — Persist an override before retrying, closing the live-switch race.
- **P17-E14** — Honor a user’s explicit model choice strictly.
- **P17-E15** — Never silently substitute an explicitly selected model.
- **P17-E16** — Record every permitted substitution as an event, consistent with Q-4 swap/restart.
- **P17-E17** — Keep billed truth and estimated usage as different typed fields end to end.
- **P17-E18** — Require every usage surface to label which of the two it displays.
- **P17-E19** — Extend J-28 `UsageTurnEvent` with the typed billed/estimate distinction.
- **P17-E20** — Implement the cooldown taxonomy, scope, and stickiness in the provider failover state machine.
- **P17-E21** — Render failover and a billed-versus-estimate label in the RuntimePanel/CloudPanel usage surface.
- **P17-E22** — Every forced billing/rate-limit transition lands a `provider_failover_event` row with its correct lane and cooldown.
- **P17-E23** — Cooling the whole provider for a billing-lane failure fails acceptance.
- **P17-E24** — Silent substitution of an explicit model fails acceptance.
- **P17-E25** — Merging billed and estimated values into one number fails acceptance.

Interactions: extends J-28, J free-API rotation, I-50, and B-37 fallback chains; aligns with Q-4; feeds K-2/PS-4.

Ambiguity:

- **P17-A01** — The summary gives an auth cooldown range rather than a deterministic ladder.
- **P17-A02** — Exact mapping from failure status/error code to model/profile/billing lane and persistence lifetime is not enumerated.
- **P17-A03** — “Billing failure cools the billed lane” is clear as a negative invariant, but the precise scope object behind that lane is not named.

### P-18 — durable TaskSpec execution + approval drift-binding

Metadata: OPENCLAW-STEAL; lever `durable-runs-anti-toctou`; STRONG for revision concurrency and approval binding; WEAK-until-RECON for resume-token step skipping; extension; effort M-L; Wave OC-2; depends on missing-summary P-11; no proof-spine number; owner-acknowledged 2026-07-11.

Explicit atomic inventory:

- **P18-E01** — Extend the O-2/P-11 `TaskSpec` run store with durable-execution mechanics.
- **P18-E02** — Add a revision counter for optimistic concurrency.
- **P18-E03** — Require writers to provide an expected revision.
- **P18-E04** — Turn a stale expected revision into a conflict.
- **P18-E05** — Refuse a stale write without clobbering current state.
- **P18-E06** — Persist cancel intent.
- **P18-E07** — Make cancel intent sticky across restart.
- **P18-E08** — Add retry ladders per failure class.
- **P18-E09** — Add watchdog phases.
- **P18-E10** — Add preflight phases.
- **P18-E11** — Clamp commitment due time so an inferred follow-up cannot fire in the same tick in which it was inferred.
- **P18-E12** — Bind every human approval to a canonical hashed run plan.
- **P18-E13** — Bind every autonomous approval to the same canonical plan form.
- **P18-E14** — Include `cwd` in the approval-bound plan.
- **P18-E15** — Include `argv` in the approval-bound plan.
- **P18-E16** — Include the allowlisted `env` in the approval-bound plan.
- **P18-E17** — Include the pinned executable in the approval-bound plan.
- **P18-E18** — Bind concrete operand fingerprints alongside the plan hash.
- **P18-E19** — Revalidate the plan and operands immediately before execution.
- **P18-E20** — Deny execution on any drift between grant and use.
- **P18-E21** — Carry “approve+resume skips prior completed steps” only as `RECON_NEEDED`, not a verified mechanism.
- **P18-E22** — Re-derive resume as a real subprocess-checkpoint model.
- **P18-E23** — Test that model with SIGKILL.
- **P18-E24** — Test pipe backpressure.
- **P18-E25** — Add a step-checkpoint store rather than rely on cooperative wrapper enforcement.
- **P18-E26** — Add the run-store mechanics and approval plan hasher/validator to the 3A/4G/P-11 approval seams.
- **P18-E27** — Expose `elai workflow resume` and `elai workflow cancel`.
- **P18-E28** — Render run revision and drift-reject rows in the workflow view.
- **P18-E29** — Two concurrent writes with one stale revision land one `taskspec_run_revision` conflict and no clobber.
- **P18-E30** — Swapping an operand after grant lands `approval_drift_reject`.
- **P18-E31** — A simulated restart preserves cancellation.
- **P18-E32** — Last-writer-wins behavior fails acceptance.
- **P18-E33** — Executing a drifted plan fails acceptance.
- **P18-E34** — Firing a commitment in its own inference tick fails acceptance.

Interactions: extends O-2/O-4, hardens P-11 confirmation, applies at 3A/4G approvals, and composes with G blast-radius gates.

Ambiguity/recon:

- **P18-A01** — Resume-token step skipping is explicitly weak/unverified and must not be implemented from the external claim.
- **P18-A02** — Retry classes, backoff values, watchdog deadlines, and checkpoint granularity are absent.
- **P18-A03** — Canonical serialization and operand-fingerprint algorithms are not defined in the summary.
- **P18-A04** — P-11 is absent, so the exact confirmation seam called “MOVE 3” needs the detail source.

### P-19 — config/ops lifecycle + `elai doctor` + verified backups [NEW area]

Metadata: OPENCLAW-STEAL; lever `recoverable-config-and-state`; STRONG/source-verified comparator grounding; new but composition-heavy area; effort M-L; Wave OC-2; no proof-spine number; owner-acknowledged 2026-07-11.

Explicit atomic inventory:

- **P19-E01** — Make absent policy objects instantiate fail-closed defaults.
- **P19-E02** — Preserve a last-known-good configuration.
- **P19-E03** — Preserve rejected configuration writes as `.rejected.<ts>`.
- **P19-E04** — Reject a write that shrinks the configuration by more than 50%.
- **P19-E05** — Add clobber guards beyond the shrink check.
- **P19-E06** — Write configuration atomically.
- **P19-E07** — Route these mechanics through J’s existing config-ingress admission gate.
- **P19-E08** — Journal recoverable rejected writes.
- **P19-E09** — Add `elai doctor detect`.
- **P19-E10** — Add `elai doctor repair`.
- **P19-E11** — Add `elai doctor lint`.
- **P19-E12** — Give doctor modes stable exit codes suitable for CI.
- **P19-E13** — Run post-upgrade compatibility probes.
- **P19-E14** — Back up SSB/session SQLite state through WAL-safe `VACUUM INTO`.
- **P19-E15** — Produce a backup manifest.
- **P19-E16** — Verify the backup against its manifest.
- **P19-E17** — Add a restart continuation queue.
- **P19-E18** — Resume one in-flight turn once after restart.
- **P19-E19** — Never split a tool call from its result during compaction.
- **P19-E20** — Create a successor transcript during compaction.
- **P19-E21** — Never rewrite the prior transcript in place.
- **P19-E22** — Allowlist environment variables; never copy the full environment.
- **P19-E23** — Adopt a working-directory path jail.
- **P19-E24** — Case-fold paths before comparing them on Windows.
- **P19-E25** — Implement config LKG/rejected writes/guards plus the doctor command and backup/continuation machinery.
- **P19-E26** — Keep doctor as a structured-text CLI/ops surface with no new frontend.
- **P19-E27** — A >50%-shrink write is rejected, preserved, and lands `config_write_rejected`.
- **P19-E28** — `elai doctor lint` returns a stable nonzero exit for that bad state.
- **P19-E29** — Doctor emits structured `doctor_finding` rows.
- **P19-E30** — Killing mid-turn causes the in-flight turn to resume exactly once after restart.
- **P19-E31** — A shrink overwrite fails acceptance.
- **P19-E32** — A run-to-run-varying doctor exit code fails acceptance.
- **P19-E33** — Splitting a tool call from its result fails acceptance.

Interactions: extends J config ingress and 3H/H context lifecycle; packages through M support/release and Q ops hardening.

Ambiguity:

- **P19-A01** — The stable exit-code map and structured `doctor_finding` schema are not enumerated.
- **P19-A02** — Restore workflow, retention, backup encryption, storage destination, and verification algorithm are absent.
- **P19-A03** — The summary says “one in-flight turn” but does not define behavior with multiple sessions/queued continuations.

### P-20 — context/memory lifecycle hardening

Metadata: OPENCLAW-STEAL; lever `no-context-lost-uncached`; STRONG/source-verified comparator grounding with PLAN-asserted H anchors; extension; effort M; Wave OC-1; no proof-spine number.

Explicit atomic inventory:

- **P20-E01** — Insert a pre-compaction memory flush as an ordered, journaled step.
- **P20-E02** — Use a silent `NO_REPLY`-equivalent turn to offer notes to durable memory.
- **P20-E03** — Write durable notes before compaction.
- **P20-E04** — Flush once per compaction cycle.
- **P20-E05** — Forbid compaction from destroying context that was never offered to durable memory.
- **P20-E06** — Keep this in-session flush distinct from H-51 cross-session sleep-time handoff.
- **P20-E07** — Add a versioned, on-demand index shared by skills and rules.
- **P20-E08** — Store skill/rule name in each compact index entry.
- **P20-E09** — Store description in each compact index entry.
- **P20-E10** — Store path in each compact index entry.
- **P20-E11** — Store a SHA-256 version marker in each entry.
- **P20-E12** — Target roughly 24 prompt tokens per skill entry.
- **P20-E13** — Read the full skill/rule file only when needed.
- **P20-E14** — Read that file once per unchanged version.
- **P20-E15** — Re-read the file when its version changes.
- **P20-E16** — Compact/degrade index detail when the context budget demands it.
- **P20-E17** — Use one lazy-load mechanism for skills, rules, and MCP defer-loading.
- **P20-E18** — Use Dreaming’s score, recall, and diversity promotion gates as prior art for H memory governance.
- **P20-E19** — Add a per-provider cache-control mapping table.
- **P20-E20** — Map Anthropic to `cache_control` with 5-minute/1-hour behavior.
- **P20-E21** — Map OpenAI to `prompt_cache_key`.
- **P20-E22** — Map Gemini to `cachedContents`.
- **P20-E23** — Do not add a second memory store.
- **P20-E24** — Add the flush hook, versioned loader, and provider mapping table to the context lifecycle.
- **P20-E25** — Add no new frontend beyond existing cost/memory panels.
- **P20-E26** — At threshold, `memory_flush_event` lands before the compaction row and carries durable notes.
- **P20-E27** — The initial prompt contains only compact index entries.
- **P20-E28** — Invoking a skill reads its file once in the trace.
- **P20-E29** — Compaction before flush fails acceptance.
- **P20-E30** — Loading a full skill body before invocation fails acceptance.

Interactions: extends H-50/H-51, flushes to H two-tier memory, unifies L familiarity/H MCP lazy loading, and supplies H cache-stable provider details.

Ambiguity:

- **P20-A01** — Memory note selection, failure/retry behavior, and what counts as “offered” are unspecified.
- **P20-A02** — Compact-degrade levels and the actual token-measurement method are absent.
- **P20-A03** — Dreaming’s score/recall/diversity thresholds are only cited as prior art, not defined.
- **P20-A04** — Provider cache behavior is temporally unstable and must be reverified at implementation time.

### P-21 — evidence/gate-credibility hardening [FEEDS PS-6]

Metadata: OPENCLAW-STEAL; lever `prove-the-gate-still-bites`; STRONG/source-verified grounding with PLAN-asserted Q-6; extension; effort M; Wave OC-1; feeds PS-6.

Explicit atomic inventory:

- **P21-E01** — Add verify-the-verifier seeded negative-control canaries to Q-6.
- **P21-E02** — Construct known-bad artifacts for those canaries.
- **P21-E03** — Run canaries through the same production detectors used on real work.
- **P21-E04** — Hard-block the entire gate if a detector misses its canary.
- **P21-E05** — Return exit code 1 for that whole-gate failure.
- **P21-E06** — Treat a missing artifact as FAIL by default.
- **P21-E07** — Permit a missing-artifact escape only when explicitly represented in the manifest.
- **P21-E08** — Restrict `missingVerdict` escape reasons to `environment-blocked` or `optional-gap`.
- **P21-E09** — Apply `skipBackfill` as a cross-lane dependency.
- **P21-E10** — Clear a lane with skipped rows only if a named sibling lane passed.
- **P21-E11** — Make audit/export output a closed metadata struct.
- **P21-E12** — Exclude content-bearing fields structurally so secrets have nowhere to appear.
- **P21-E13** — Hash identifiers instead of storing their content.
- **P21-E14** — Restrict fields to a whitelist or the value `unknown`.
- **P21-E15** — Keep the shareable metadata projection and content-carrying debug bundle separate by type.
- **P21-E16** — Add smoke-evidence posture labels.
- **P21-E17** — Support posture `user-path`.
- **P21-E18** — Support posture `direct-gateway`.
- **P21-E19** — Support posture `native-approval`.
- **P21-E20** — Fail smoke evidence that lacks a posture label.
- **P21-E21** — Apply `unknown=FAIL`.
- **P21-E22** — Do not allow a classified product bug to produce a green report by default.
- **P21-E23** — Add the canary lane/missing escape to Q-6, posture to R10 evidence, and closed export type to G-9.
- **P21-E24** — Reuse the existing gate/evidence UI surface.
- **P21-E25** — A deliberately broken canary detector makes the self-test and whole gate fail with exit 1.
- **P21-E26** — The audit-export type has no content field.
- **P21-E27** — A missed canary that still permits PASS fails acceptance.
- **P21-E28** — A classified failure that defaults to green fails acceptance.
- **P21-E29** — Content in the metadata-only export fails acceptance.

Interactions: extends Q-6/PS-6 and wiring R10; uses G-9 structural redaction and M metadata-only fleet rollup.

Ambiguity:

- **P21-A01** — Canary corpus contents, seeding cadence, detector coverage, and required kill-rate are not enumerated here.
- **P21-A02** — `optional-gap` and `environment-blocked` admission criteria are named but not defined.
- **P21-A03** — Debug-bundle custody/access policy is outside this summary.

### P-22 — control-plane/channel hardening

Metadata: OPENCLAW-STEAL; lever `fenced-returns-signed-ingress`; STRONG/source-verified comparator grounding with PLAN-asserted ELAI anchors; extension; effort M-L; Wave OC-3; no proof-spine number.

Explicit atomic inventory:

- **P22-E01** — Treat child/subagent output as fenced evidence data.
- **P22-E02** — Make a subagent return structurally unable to carry executable instructions.
- **P22-E03** — Treat every subagent return as a G-1 taint source.
- **P22-E04** — Apply depth-scoped default tool stripping.
- **P22-E05** — Remove gateway authority from subagents by default.
- **P22-E06** — Remove cron authority from subagents by default.
- **P22-E07** — Remove messaging authority from subagents by default.
- **P22-E08** — Remove spawn authority from leaf agents.
- **P22-E09** — Add a deterministic nine-step channel-binding precedence.
- **P22-E10** — Resolve channel binding by most-specific-wins.
- **P22-E11** — Never let the model choose the channel.
- **P22-E12** — Add pairing codes that are short-lived.
- **P22-E13** — Cap pending pairing requests.
- **P22-E14** — Add `dmScope` session-isolation options for P-10 ingress/front-door/M enrollment.
- **P22-E15** — Pair devices through a challenge-signed flow.
- **P22-E16** — Bind the signature payload to the platform/device context.
- **P22-E17** — Return stable authentication error codes.
- **P22-E18** — Withhold an unknown event family from broadcast rather than leak it.
- **P22-E19** — Require idempotency keys on every side-effecting RPC.
- **P22-E20** — Deduplicate a replayed mutating RPC by that idempotency key.
- **P22-E21** — Order a trusted-policy hook tier before all third-party hooks.
- **P22-E22** — Prevent plugins from preempting ELAI’s frozen gates.
- **P22-E23** — Validate plugin manifests without executing them.
- **P22-E24** — Reuse the existing N-6 provenance scale; do not fork another.
- **P22-E25** — Wire fenced returns/tool stripping, signed pairing/unknown gating/idempotency, channel precedence, and hook ordering into their existing seams.
- **P22-E26** — Render enrollment and pairing UX on the control surface at Tier 2.
- **P22-E27** — Instruction-shaped subagent text lands as a fenced taint-source row and remains inert.
- **P22-E28** — An unknown broadcast event is withheld.
- **P22-E29** — A replayed mutating RPC is applied at most once.
- **P22-E30** — A subagent return that steers its parent fails acceptance.
- **P22-E31** — Leaking an unknown event family fails acceptance.
- **P22-E32** — Double-applying a mutating RPC fails acceptance.

Interactions: extends 3C/BSP, G-1, J control plane, P-10 ingress, M seats, and 4C plugin admission.

Ambiguity:

- **P22-A01** — The nine binding steps and their exact specificity keys are not listed.
- **P22-A02** — `dmScope` option values, code lifetimes, pending caps, signature algorithm, and platform payload schema are absent.
- **P22-A03** — “Instructions structurally unable to carry” requires a concrete fenced-data encoding not shown here.

### P-25 — exogenous-injection + evidence-coupled-revision creativity falsifier (B-EXO-0)

Metadata: CREATIVITY-FALSIFIER; levers `deterministic-exogenous-injection` and `grounded-loop-revision`; injection lever MEASURED in pilot P-A at 3.5× anchors, creative-yield effects still HYPOTHESES; new harness-only falsifier; effort M; authored 2026-07-14; successor to refuted Q-10 B-CREATIVITY-0 b0v3; Family B instantiates grounding-as-exit; evidence run not spine-gated.

Explicit atomic inventory:

- **P25-E01** — Test both surviving creativity-lever families on one frozen corpus.
- **P25-E02** — Define the corpus as K=30 open-world real ELAI improvement fixtures.
- **P25-E03** — Give fixtures hidden deterministic micro-executors.
- **P25-E04** — Treat B-CREATIVITY-1 corpus design as specified but never built at summary time.
- **P25-E05** — Run six arms total.
- **P25-E06** — Use two independently preregistered joint gates, one per family.
- **P25-E07** — Forbid cherry-picking between the two gates.
- **P25-E08** — Declare multiplicity explicitly.
- **P25-E09** — Family A calls one direct champion conditioned on exactly k=3 retrieved artifacts.
- **P25-E10** — Retrieve Family-A artifacts deterministically.
- **P25-E11** — Make Family-A retrieval model-free.
- **P25-E12** — Require retrieved artifacts to be target-relevant and verified.
- **P25-E13** — Draw the three artifact classes from an `elai.map` anchor slice, a measured bench/journal row, and an E-48 `VerifiedCitation`.
- **P25-E14** — Compare injection against a `DIRECT` arm.
- **P25-E15** — Compare against cross-subsystem-stratified `EXO_SHUFFLED`.
- **P25-E16** — Compare against `PAD_MATCHED`.
- **P25-E17** — Apply a hard 1.3× Family-A cost cap.
- **P25-E18** — Permit zero preprocessing model calls; that absence is part of the mechanism.
- **P25-E19** — Family B `TRIAGE_REVISE` first generates candidates.
- **P25-E20** — Run real deterministic public triage legs over the candidates.
- **P25-E21** — Include demarcation, anchor, and composition triage legs.
- **P25-E22** — Exclude the ledger triage leg.
- **P25-E23** — Make the exogenous-card grammar emit the fields being judged.
- **P25-E24** — Reconcile `KEY_ALIASES` before judging.
- **P25-E25** — Feed each candidate’s actual kill reasons into one revision call.
- **P25-E26** — Compare against `SHUFFLED_REVISE` with the identical template.
- **P25-E27** — Permute kill reasons away from their true candidates in the shuffled control.
- **P25-E28** — Apply a hard 2.5× Family-B cost cap.
- **P25-E29** — Require cost-normalized P1 non-inferiority.
- **P25-E30** — Fail a Family-B win bought by approximately double spend.
- **P25-E31** — Use execution-validated mechanism yield as a co-primary for each family.
- **P25-E32** — Use best target-value delta as the other co-primary.
- **P25-E33** — Evaluate co-primaries with paired bootstrap 95% confidence intervals.
- **P25-E34** — Do not add packetization, analogy, archive conditioning, operator rotation, or probes to this bench.
- **P25-E35** — Distinguish sequential evidence-coupled revision from previously tested archive cycles and parallel context-blind calls.
- **P25-E36** — Record rejection-ledger clearance.
- **P25-E37** — Preserve two red-team passes—authoring and delta—and patch all BLOCK findings before execution.
- **P25-E38** — Import Q-10 Tier-0 instrument modules rather than enlarge the runner beyond 1,000 lines.
- **P25-E39** — Use `conjecture_triage.py` kill legs as Family-B evidence.
- **P25-E40** — Use the research-creativity analysis/verifier.
- **P25-E41** — Use `elai.map` and E-48 `VerifiedCitation` rows as read-only substrate.
- **P25-E42** — Use the grounding-as-exit doctrine as the Family-B design source.
- **P25-E43** — Integrate planned `.claude/tools/exo_retrieval.py` and `.claude/tools/exo_injection_ab.py`.
- **P25-E44** — Implement a six-arm, two-call revision lifecycle.
- **P25-E45** — Freeze the triage-render template and fixture corpus.
- **P25-E46** — Put nothing in `.elai_cc` for this harness-only bench.
- **P25-E47** — Produce a sealed evidence directory.
- **P25-E48** — Require an independent checker to exit 0.
- **P25-E49** — Include a byte-level check of revision input.
- **P25-E50** — If Family A is supported, create a separate adoption unit for an improvement-conjecture stage-1 exogenous leg plus paper-to-plan seam.
- **P25-E51** — If Family B is supported, create a separate adoption unit for a triage-feedback revision leg.
- **P25-E52** — If both families are null/refuted, close the creativity line entirely.
- **P25-E53** — A retrieval-time model call fails the bench.
- **P25-E54** — A triage-time model call fails the bench.
- **P25-E55** — Hidden-side data in revision input fails the bench.
- **P25-E56** — A second revision round fails the bench.
- **P25-E57** — A shuffled arm matching treatment relevance fails the bench.
- **P25-E58** — A cost row beyond its family cap without G3 firing fails the bench.

Interactions: imports Q-10 Tier-0 instruments and consumes deterministic triage, research-creativity checking, map anchors, E-48 citations, and the grounding-as-exit doctrine; it creates adoption work only after evidence.

Ambiguity/status:

- **P25-A01** — The pilot’s 3.5× anchor gain supports injection mechanics only; both creative-yield effects remain hypotheses.
- **P25-A02** — The K=30 B-CREATIVITY-1 corpus is explicitly described as never built in this summary.
- **P25-A03** — The named integration files `exo_retrieval.py` and `exo_injection_ab.py` are not present at those exact paths in the current tree; newer P-25/EXO candidate tools exist and require detail-source reconciliation.
- **P25-A04** — Exact gate formulas, P1 definition, target-value metric, bootstrap resampling plan, and G3 behavior are not reproduced here.
- **P25-A05** — “Six-arm, two-call revision lifecycle” can be reconciled as four Family-A arms plus two Family-B arms, but the phrase “two-call” needs the detail plan to disambiguate per-candidate versus per-bench calls.

## End-to-end workflows and interactions

These sequences only order explicitly named behaviors; they add no new requirements.

1. **Fleet health and repair**: `AppManifest` registration → O-2 health-tick `TaskSpec` → smoke/drift/dependency checks → exactly one off-agent health row → N-7 human card only for red → proposed repair through the ordinary wiring/claim gates → no silent patch.
2. **World wiring**: declare ports on manifests → compatibility check → reject incompatibility without an edge → bind a compatible edge with provenance and taint tiers → retain FSM dispatch → show zero-consumer outputs as visibly disconnected.
3. **Transfer**: factor mechanism from persona data → redact provenance → export mechanism/manifest/RRA → journal export → M-5/L-12 reverify on recipient → refuse tamper or mount clean bundle → regenerate recipient persona locally.
4. **Living-brain access**: mount a live store after source recon → consume watch/change feed → tier/TTL-tag ingest → enforce namespace claim and P-6 privacy at read/routing boundaries → journal ingest/refusal → never write user-owned notes outside the claim.
5. **Graduated ceremony**: select a tier → journal profile set → run the same gate at every tier → toy observes and journals without WARN blocking → critical blocks → one-line receipt links to full evidence → measure toy tax.
6. **Profile adaptation**: observe process outcomes → permit tighten immediately → refuse loosen without own-run measured FP evidence → H-18 promotion with evidence → journal and support revert.
7. **Two-seat collaboration**: tag every seat/agent event → enforce namespace locks → merge both event streams losslessly and causally → derive direction-independent order hash → query attribution and shared-project rollup.
8. **Desktop actuation**: sandbox → perceive source-tagged elements → bind exact delivered frame hash → journal arm grant → expose leased action → journal action/frame bind before actuation → capture bounded recording → reject stale frames/no actuation.
9. **Perception-to-action**: condition input through J-1 → derive AXTree IDs → use guarded pixel fallback only if necessary → source-tag and filter interactable elements → model chooses ID → harness resolves geometry; never let VLM produce counts/identity truth.
10. **Explore/crystallize/reuse**: run model-assisted browser/desktop exploration → emit deterministic stage → bind to minting RRA → replay with zero model calls → save a reusable case only after proven execution → retrieve/distill/fuse on recurrence → receipts rank, journal narrative only breaks ties.
11. **Capability installation**: resolve named catalog row → existing admission/policy check → mount headless P-9 pack when possible → use GUI only as last resort → journal every admit/refusal.
12. **Co-browse**: open E-14-backed browse node → taint extraction → funnel both actors through one private `InputProxy` → journal prior to CDP dispatch → privacy-check outbound text → keep profiles ephemeral or vault-custodied → optionally crystallize the run.
13. **Mobile ladder**: choose device profile → exercise cheap web viewport/device frame first → optionally route Expo/RNW through H-14 click-to-edit → use pinned native emulator only for native gate → perceive through mobile accessibility → crystallize Maestro actuation → keep verdicts off-agent and rung-specific.
14. **Provider failover**: classify failure → choose model/profile lane and deterministic cooldown → preserve session stickiness if possible → persist override before retry → never override explicit selection silently → journal cache invalidation/failover → display billed and estimated cost separately.
15. **Durable approval**: revise run using expected revision → conflict stale writers → canonicalize/hash plan and operands at approval → revalidate at execution → deny drift → preserve cancel/restart state and never fire newly inferred commitments in the same tick.
16. **Recoverable ops**: admit config write → reject/preserve shrink/clobber → retain LKG → doctor detect/lint/repair with stable code → backup WAL safely and verify manifest → resume one in-flight turn once → compact only into a successor transcript without separating tool/result.
17. **Context lifecycle**: build compact versioned skill/rule/MCP index → lazy-read body on use → pre-compaction durable-memory flush → journal flush before compaction → re-read only on version change → apply provider-specific cache controls.
18. **Gate self-credibility**: plant known-bad artifact → run production detector → hard-fail entire gate on miss → require posture and named skip dependency → export only a structurally content-free projection.
19. **Control ingress**: bind channel deterministically → pair device with challenge signature → strip child authority by depth → fence child return as inert tainted data → withhold unknown event types → idempotently dedupe mutating RPC → run trusted policy hooks before third-party hooks.
20. **Creativity falsification**: freeze corpus/preregistration → run Family-A direct/injection controls under 1.3× → run Family-B true/shuffled kill-reason revision under 2.5× → evaluate both independent co-primary gates with paired CI → seal/check bytes and evidence → spawn only the supported family’s separate adoption unit, or close the line if both fail.

## Cross-cutting constraints and invariants

- Reuse one canonical scheduler, privacy/provenance lattice, memory store, browser engine, case graph, admission scanner, dispatcher, and control plane; any second copy is a design failure where stated.
- Keep code/mechanism common while persona, packs, profiles, catalogs, and device profiles remain data.
- Keep every actor/action attributable and journaled at the grain where it occurs.
- Never let a convenience surface remove the underlying journal row or full receipt.
- Never let GUI/operator execution become correctness evidence; correctness must come from off-agent rows/artifacts.
- Bind coordinate actions to the exact observed frame and invalidate that authority on staleness or compaction.
- Keep VLM output advisory for captions/point selection; counts, identity geometry, and mobile verdicts stay off-model.
- Journal grants and browser inputs before capability exposure or dispatch.
- Default privacy reads to deny and persistent browser state to ephemeral unless vault-custodied.
- Preserve explicit user model choice and distinguish billed truth from estimates by type.
- Bind approval to canonical plan bytes and concrete operands, then revalidate at use.
- Treat missing gate evidence and unknown values/events as failure or withholding unless a narrow typed escape is present.
- Keep shareable metadata structurally unable to contain secret-bearing content.
- Treat subagent returns as tainted inert evidence and decrease tool authority with depth.
- Prefer the cheapest mobile rung but never use it to satisfy a stronger rung’s claim.
- Preserve provenance and deterministic direction-independent ordering when merging seat journals.
- Fail a creativity result that buys improvement by violating the family’s cost cap or contaminating a control.

## Dependencies, status, proof feeds, and deferred work

| Unit | Direct dependency/status stated in summary | Proof/evidence status | Explicitly deferred, gated, or unresolved |
|---|---|---|---|
| P-1 | Wave 1, keystone | No PS number; MODERATE | Per-app G-10 canary generalization needs recon |
| P-2 | P-1, O-4; Wave 2 | No PS number; MODERATE | Compatibility/schema lifecycle absent |
| P-3 | P-1, P-2; Wave 3 | No PS number; MODERATE | Bundle/persona regeneration schema absent |
| P-4 | O-3, P-6; Wave 2; GATED | WEAK-until-RECON | Obsidian/Notion API grounding blocks code |
| P-5 | Wave 1 | No PS number; MODERATE | “Near-zero” tax lacks threshold |
| P-6 | N-6, O-3; Wave 2 | No PS number; MODERATE | Grant precedence/lifetime absent |
| P-7 | N-3 landed; Wave 3 | No PS number; MODERATE | Metric/threshold/default definitions absent |
| P-8 | O-3, K-23; Wave 4 | No PS number; MODERATE | Merge kernel is an open spike-first problem |
| P-9 | Detail file exists, summary body absent | Unknown here | Must be read by a deeper agent |
| P-10 | Detail file exists, summary body absent | Unknown here | Must be read by a deeper agent |
| P-11 | Detail file exists, summary body absent | Unknown here | Must be read by a deeper agent |
| P-12 | Wave CU-2; owner-acked | Product/no PS; MODERATE | License recheck; frontend placement at edit |
| P-13 | Wave CU-1 | Product/no PS; STRONG comparator | Source-fusion precedence absent |
| P-14 | P-13; Wave CU-2 | Feeds PS-5; STRONG | Reliability threshold absent |
| P-15 | J-96 extension; Wave CU-3 | Feeds PS-5; STRONG | Reuse default-off; promotion criteria external |
| P-16 | P-9/P-10; Wave CU-3; owner-acked | Product/no PS; MODERATE | Blocked in this summary by missing prerequisite bodies |
| P-23 | P-13; Wave CU-4; owner-commissioned | Product/no PS; strong comparator/moderate ELAI | Native multiwebview remains latency-triplined fork |
| P-24 | H-14/P-12/P-13/P-14/P-16; Wave CU-5 | No direct PS; Rung 3 transitively feeds PS-5 | Native RN Fiber editing follow-up; physical device ABORT |
| P-17 | Wave OC-1 | Feeds PS-4; STRONG comparator | Some cooldown mapping/scope details absent |
| P-18 | P-11; Wave OC-2; owner-acked | Product/no PS; mixed grounding | Resume/checkpoint mechanism is WEAK-until-RECON |
| P-19 | Wave OC-2; owner-acked; NEW area | Ops/no PS; STRONG comparator | Restore/retention/exit schema absent |
| P-20 | Wave OC-1 | No PS; STRONG comparator | Provider mapping must be current; promotion thresholds absent |
| P-21 | Wave OC-1 | Feeds PS-6; STRONG | Corpus/kill-rate and escape admission definitions absent |
| P-22 | Wave OC-3 | No PS; STRONG comparator | Nine precedence steps and auth parameters absent |
| P-25 | Harness-only evidence run; successor to refuted Q-10 mechanism | Pilot injection measured; creative yields hypotheses; not spine-gated | Corpus described as never built; adoption conditional on results |

Plan-wide execution status remains **as-designed**. The summary does not establish which Rust/CLI/frontend artifacts exist. Deep red-team execution is deferred for the steal tracks, and P as a whole is scheduled after the proof spine.

## Deeper-source manifest

### Primary P sources present on disk but not read in this pass

- `docs/thefinalglance/P-plan/P-0_INDEX.md` — doctrine and sequencing explicitly delegated here by the summary.
- `docs/thefinalglance/P-plan/P-1_app_fleet_layer.md`
- `docs/thefinalglance/P-plan/P-2_inter_tool_ports_world_bus.md`
- `docs/thefinalglance/P-plan/P-3_persona_split_export.md`
- `docs/thefinalglance/P-plan/P-4_living_external_brains.md`
- `docs/thefinalglance/P-plan/P-5_operator_surface_graduated_ceremony.md`
- `docs/thefinalglance/P-plan/P-6_life_data_privacy_lattice.md`
- `docs/thefinalglance/P-plan/P-7_per_user_operating_profile.md`
- `docs/thefinalglance/P-plan/P-8_small_team_collaboration_grain.md`
- `docs/thefinalglance/P-plan/P-9_connector_pack_admission_format.md` — **highest-priority missing-summary candidate**.
- `docs/thefinalglance/P-plan/P-10_ingress_adapters_credential_vault.md` — **highest-priority missing-summary candidate**.
- `docs/thefinalglance/P-plan/P-11_workflow_from_goal.md` — **highest-priority missing-summary candidate**.
- `docs/thefinalglance/P-plan/P-12_desktop_operator_lane.md`
- `docs/thefinalglance/P-plan/P-13_perception_provenance_setofmarks.md`
- `docs/thefinalglance/P-plan/P-14_explore_then_crystallize_lane.md`
- `docs/thefinalglance/P-plan/P-15_verified_case_graph_hardening.md`
- `docs/thefinalglance/P-plan/P-16_gated_install_catalog.md`
- `docs/thefinalglance/P-plan/P-17_provider_failover_economics.md`
- `docs/thefinalglance/P-plan/P-18_durable_taskspec_drift_binding.md`
- `docs/thefinalglance/P-plan/P-19_config_ops_lifecycle_doctor.md`
- `docs/thefinalglance/P-plan/P-20_context_memory_lifecycle.md`
- `docs/thefinalglance/P-plan/P-21_gate_credibility_hardening.md`
- `docs/thefinalglance/P-plan/P-22_control_plane_channel_hardening.md`
- `docs/thefinalglance/P-plan/P-23_journaled_cobrowse_lane.md`
- `docs/thefinalglance/P-plan/P-24_mobile_design_test_lane.md`
- `docs/thefinalglance/P-plan/P-25_exogenous_injection_creativity_falsifier.md`

### Design/evaluation sources explicitly named by the P summary

- `workflow/reasoning/elai-personal-software-incarnation-2026-07-07.md` — source design for P-1..P-8.
- `workflow/reasoning/canvas-computer-use-eval-2026-07-11.md` — computer-use comparison/steal source for P-12..P-16/P-23/P-24.
- `workflow/reasoning/openclaw-vs-elai-comparison-2026-07-11.md` — OpenClaw comparison/steal source for P-17..P-22.
- `workflow/reasoning/grounding-as-exit-grc-v3-2026-07-14.md` — likely exact grounding-as-exit doctrine file referenced by P-25; verify in its detail plan.
- `docs/thefinalglance/Q-plan/Q-10_research_conditioned_creativity.md` — refuted predecessor and Tier-0 instrument source for P-25.
- `docs/thefinalglance/E-plan/E-48_web_research_orchestrator.md` — `VerifiedCitation` substrate named by P-25.
- `docs/thefinalglance/H-plan/H-14_design_mode_web_first.md` — explicitly deferred smartphone/RN slice consumed by P-24.
- `docs/thefinalglance/J-plan/J-96_verified_case_graph_core.md` — verified case-graph core extended by P-15.

### P-25 implementation/evidence source candidates

- `.claude/tools/conjecture_triage.py` — explicitly named and present.
- `.claude/tools/research_creativity_analysis.py` — named analysis role, present.
- `.claude/tools/research_creativity_artifact_check.py` — likely named verifier role, present; detail plan must confirm.
- `.claude/tools/elai.map` and `elai.map` — both exist; the summary’s bare `elai.map` reference is ambiguous.
- `workflow/reasoning/exo-injection-fixture-pool-2026-07-14.json` — fixture-pool candidate present; reconcile against the summary’s “K=30 ... never built” statement and later chronology.
- `workflow/reasoning/exo-injection-candidate-core-binding-2026-07-15.md` — later source candidate present.
- `workflow/reasoning/exo-injection-evaluator-trust-architecture-2026-07-14.md` — evaluator source candidate present.
- `.claude/tools/p25_exo_lean.py`, `.claude/tools/p25_exo_lean_check.py`, and `.claude/tools/p25_exo_resume.py` — current-tree P-25-named candidates, not the exact integration names in the summary.
- `workflow/benchmarks/p25-exo-lean-2026-07-17/` — later benchmark directory candidate.
- **Referenced exact names not found:** `.claude/tools/exo_retrieval.py` and `.claude/tools/exo_injection_ab.py`.
- **Path unresolved from summary:** the frozen triage-render template, sealed evidence directory, independent-checker output, and the B-CREATIVITY-1 K=30 corpus have no exact path in the summary.

### External comparator/prior-art references named but not path-resolved here

- CUA: Windows Sandbox/Lume/Docker substrate, `OperatorNormalizer`, and `replay_env`.
- OmniParser arXiv:2408.00203, including `icon_detect` weights and Florence captions.
- Agent-S KnowledgeBase arXiv:2410.08164.
- Agent Zero skills/scripts convention and ungated install behavior.
- Open Computer Use `tools_config` catalog.
- OpenClaw ClawHub/install policy, frame/display binding, failover, gateway/channel, gate, redaction, and ops mechanisms; use the comparison document rather than code.
- Claude Code desktop browser announcement/post ID `2075635283211772279`, 9to5Mac coverage, and Tauri issues `#10131/#10420`.
- Expo SDK 54+/Metro/React-Native-Web and Maestro single-binary YAML flows.
- Task-Flow revision counter and approval drift binding.
- “Lobster” approve/resume claim, explicitly unverified.
- Dreaming memory-promotion gates.

## Discovery conclusion

The summary directly specifies 22 feature units spanning personal-world composition, desktop/mobile computer use, durable orchestration mechanics, and one benchmark-only creativity falsifier. Its most material coverage defect is the complete omission of the three COMMAND-CENTER units P-9/P-10/P-11 despite later dependencies on all three. No behavior from those detail files should enter the final super-list until a deeper agent reads them.
