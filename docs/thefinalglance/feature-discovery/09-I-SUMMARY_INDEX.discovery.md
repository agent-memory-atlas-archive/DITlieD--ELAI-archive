# I-plan summary feature discovery

## Scope, evidence standard, and coverage

This is a summary-level discovery artifact for the I plan. Its sole feature source is
docs/thefinalglance/I-plan/I-SUMMARY_INDEX.md. The I section of
ELAI-SPEC-OVERVIEW.md was used only as orientation. The 54 numbered plan files, I-0_INDEX.md, grounding
maps, research dossiers, and competitor repositories are inventoried at the end for separate deep
inspection; their contents were not silently folded into this extraction.

Evidence labels:

- **[E] Explicit**: directly stated by the summary.
- **[I] Inferred**: a composition consequence, not independently specified as a feature. Inferences are
  kept rare and are labeled.
- **[A] Ambiguous/open**: the summary explicitly leaves a threshold, choice, status, or implementation
  detail unresolved.

Quantitative coverage:

- Primary source read: **1,048/1,048 lines**.
- Numbered sessions covered: **54/54**, I-1 through I-54.
- Tracks covered: **9/9**, Tracks 12 through 20.
- Original numbered-plan presence check: **54/54 files present**.
- Additional I-plan control files present: I-0_INDEX.md and both grounding maps.
- Deep-source corpus not yet inspected: **6,678 lines across I-0 plus the 54 numbered plans**, plus
  **303 lines across the two grounding maps**.
- The summary says its live grounding baseline was **221/532 at 2026-06-10** and explicitly requires a
  re-read at edit time. That number is provenance, not current implementation status.

Track coverage map:

| Track | Sessions | Count | Summary theme |
|---|---:|---:|---|
| 12 | I-1..I-3 | 3 | Fault isolation and enforcement plane |
| 13 | I-4..I-9 | 6 | Evidence-ranked search |
| 14 | I-10..I-16 | 7 | Cache-invariant economics middleware |
| 15 | I-17..I-24 | 8 | Deterministic gates and anti-gaming verification |
| 16 | I-25..I-29 | 5 | Routing, budgets, and measurement |
| 17 | I-30..I-31 | 2 | Open-weight flywheel |
| 18 | I-32..I-40 | 9 | Deterministic-over-prompt hardening |
| 19 | I-41..I-44 | 4 | Design-phase critique and trust-claim discipline |
| 20 | I-45..I-54 | 10 | Competitor-steal ports |

## Corpus-level design rules and interactions

- **I.CROSS.01 [E]** I is described as the first letter authored after the wiring bedrock was reached;
  every session is intended as feature engineering over an already wiring-disciplined substrate.
- **I.CROSS.02 [E]** Every summary entry records track and lever, grounding strength, nature, the existing
  substrate or absent code, a COMPOSES-WITH chain, a production integration point, and an observable-effect
  wiring done-gate.
- **I.CROSS.03 [E]** Tracks 12-18 derive from the docs/prefableresearch corpus; Track 19 derives from two
  grok-elai-trust documents plus the token-efficiency plan; Track 20 derives from competitor code and
  STEAL-LIST.md.
- **I.CROSS.04 [E]** The generator proposes while the orchestrator owns state and receipt-ranked selection.
- **I.CROSS.05 [E]** Missing required evidence rows are failures, not silent skips, wherever R1 is named.
- **I.CROSS.06 [E]** Same-seed recomputation and byte-identical replay are recurring done-gates wherever
  GD-2 is named.
- **I.CROSS.07 [E]** Existing event/action types are reused where possible; several sessions explicitly
  prohibit introducing a new action type.
- **I.CROSS.08 [E]** Paper numbers are not adopted as product thresholds. ELAI measures its own baselines
  and records losing or non-monotonic results honestly.
- **I.CROSS.09 [E]** User approval, anti-slopsquat checks, and separate network/RunPod checkboxes gate
  optional dependencies or live external evaluations.
- **I.CROSS.10 [E]** The I-1 supervisor is the enforcement boundary used later by I-35 sequence rules and
  the I-31 reward endpoint.
- **I.CROSS.11 [E]** I-2 isolated workspaces make I-4 parallel search safe.
- **I.CROSS.12 [E]** I-17 is blocking for I-4; I-4 in turn blocks I-22, I-28, and I-30.
- **I.CROSS.13 [E]** I-30 plus the H safety guards and I-24 are blocking prerequisites for I-31, which is
  explicitly last.
- **I.CROSS.14 [E]** Track 14 inherits the local window-minimization versus cloud cache-stability split
  rather than reopening it.
- **I.CROSS.15 [E]** Track 15 exists to make I-4 selection receipts trustworthy: hard-veto supremacy,
  bounded mutation evidence, independent contracts, scorer canaries, and tamper-evident records.
- **I.CROSS.16 [E]** Track 17 may run only on a stronger-than-generator, execution-grounded filter; every
  learned weight is handled as a ledger-gated configuration artifact.
- **I.CROSS.17 [E]** Track 18 replaces prompt-shaped controls with deterministic mechanisms and measures
  each replacement.
- **I.CROSS.18 [E]** Track 19 LLM critique is advisory only and never determines accept/block; the
  off-agent F/G spine remains the sole DONE arbiter.
- **I.CROSS.19 [E]** Track 20 requires each executor to read the named competitor source file before
  execution. A summary-only port violates W-3.
- **I.CROSS.20 [E]** Track 20 corrections are binding: activation-not-build where substrate ships,
  wrap-never-delete for the measured VRAM tier plan, and idea-not-dependency for structural editing.
- **I.CROSS.21 [E]** The wholesale gajae provider-layer port is rejected; Tier-3 competitor items stay
  reference material and recorded declines must not be re-litigated.

## Exhaustive atomic inventory

### Track 12 — fault-isolation and enforcement-plane

#### I-1 — directive supervisor enforcement plane

Source anchor: I-SUMMARY_INDEX.md:35. Metadata: CANT-WRECK; STRONG; BUILD; L; Wave 2.

- **I-1.01 [E][capability]** Introduces pure Intent-to-Directive structs emitted by the generator.
- **I-1.02 [E][security]** Places Directive execution in a process-separated Supervisor.
- **I-1.03 [E][invariant]** Makes the Supervisor the only executor of side effects.
- **I-1.04 [E][workflow]** Runs every applicable gate before the Supervisor executes a Directive.
- **I-1.05 [E][security]** Makes the Supervisor, not the generator, hold credentials.
- **I-1.06 [E][security]** Prevents the generator from reaching gate code.
- **I-1.07 [E][security]** Prevents the generator from reaching the egress path or calling a sink directly.
- **I-1.08 [E][lifecycle]** Uses saga-style compensation after a crash to restore pre-tool state.
- **I-1.09 [E][dependency]** Reuses the existing Phase 4A IPC process seam rather than creating another.
- **I-1.10 [E][integration]** Moves F-1 VibeGate, E-30 argv, and G-3 blast-radius enforcement onto the
  supervisor plane.
- **I-1.11 [E][integration]** Integrates at task_runner dispatch and hook_dispatcher seams and proposes
  crates/orchestrator/src/directive.rs.
- **I-1.12 [E][test]** Adds a contract test that the generator cannot author or reach gate code.
- **I-1.13 [E][UI]** Shows supervisor verdict rows in an existing drawer.
- **I-1.14 [E][failure]** An injected OOM or panic during a tool call must restore exact pre-tool state.
- **I-1.15 [E][observability]** Successful compensation emits DirectiveExecuted(CompensationFired).
- **I-1.16 [E][failure]** A generator attempt to modify gate code or directly call a sink is refused.
- **I-1.17 [E][observability]** A refused Directive is logged in full.
- **I-1.18 [E][fail-closed]** A missing required supervisor row is a failure.
- **I-1.19 [E][dependency]** This boundary is the later execution home for I-35 temporal rules and I-31
  reward serving.

#### I-2 — cow workspace isolation

Source anchor: I-SUMMARY_INDEX.md:52. Metadata: CONCURRENCY; STRONG; BUILD; M-L; Wave 2; ID-2.

- **I-2.01 [E][capability]** Gives each parallel branch or agent an isolated workspace.
- **I-2.02 [E][platform]** Uses overlayfs or btrfs reflinks on Linux.
- **I-2.03 [E][platform]** Uses pooled warm git worktrees with a shared target directory on Windows and
  macOS.
- **I-2.04 [E][constraint]** Does not assume cross-platform copy-on-write support.
- **I-2.05 [E][workflow]** Permits merges only at an explicit deterministic BSP barrier.
- **I-2.06 [E][determinism]** Orders merges by SSB sequence.
- **I-2.07 [E][security]** Blocks a branch that escapes its workspace root.
- **I-2.08 [E][extension]** Allows an optional cloud-sandbox adapter behind the same trait.
- **I-2.09 [E][dependency]** Makes the cloud sandbox optional rather than a required dependency.
- **I-2.10 [E][integration]** Adds a branch-workspace trait plus overlayfs and worktree-pool implementations
  under crates/orchestrator/src/sandbox/branch_workspace.rs.
- **I-2.11 [E][integration]** Connects merging at the BSP aggregator merge site.
- **I-2.12 [E][UI]** Displays branch-workspace badges on BranchTree.
- **I-2.13 [E][observability]** Journals per-branch root prefixes so disjoint paths are verifiable.
- **I-2.14 [E][observability]** Emits BranchWorkspaceEvent states Spawned and Merged.
- **I-2.15 [E][failure]** Emits EscapeBlocked for an absolute-path write outside the branch root.
- **I-2.16 [E][test]** A seeded adjacent-code conflict must resolve deterministically at the barrier.
- **I-2.17 [E][test]** A same-seed rerun must produce a byte-identical merge.
- **I-2.18 [E][interaction]** Supplies safe parallelism to the I-4 search ladder while remaining distinct
  from I-52 single-agent in-branch rewind.

#### I-3 — ssb throughput + atomic checkpoint saves

Source anchor: I-SUMMARY_INDEX.md:68. Metadata: PERF; STRONG; EXTENSION; S; Wave 1.

- **I-3.01 [E][performance]** Verifies or extends busy_timeout, cache_size, and wal_autocheckpoint for
  search-era SSB write rates.
- **I-3.02 [E][durability]** Uses temporary-file, fsync, and rename atomic writes at every checkpoint site.
- **I-3.03 [E][scope]** Closes only checkpoint sites that are not already atomic.
- **I-3.04 [E][measurement]** Measures real SSB insert throughput rather than claiming a paper number.
- **I-3.05 [E][data]** Records throughput in metrics/phase_I3.toml.
- **I-3.06 [E][platform]** Exposes io_uring only as a Linux opt-in experiment.
- **I-3.07 [E][constraint]** Keeps io_uring allowlisted and never the default path.
- **I-3.08 [E][integration]** Touches the ssb.rs write-loop pragma block and checkpoint sites in the
  orchestrator and session DAG.
- **I-3.09 [E][UI]** Has no live UI; the metrics TOML is the justified observable surface.
- **I-3.10 [E][failure]** A kill -9 during a checkpoint must leave either the old or new complete file,
  never a torn file.
- **I-3.11 [E][test]** The no-atomic broken arm must produce a torn file and fail.
- **I-3.12 [A][status]** The summary requires re-verifying which pragmas and checkpoint sites already ship;
  it does not enumerate the remaining gaps.

### Track 13 — evidence-ranked search

#### I-4 — evidence-ranked search ladder [TRACK 13 KEYSTONE]

Source anchor: I-SUMMARY_INDEX.md:91. Metadata: SMARTER-ORCH; STRONG; BUILD; XL tiered; Wave 4; blocking
for I-22/I-28/I-30; depends on I-2 and I-17; ID-1/ID-2/ID-4.

- **I-4.01 [E][capability]** Selects search candidates by off-agent receipts rather than an LLM judge.
- **I-4.02 [E][workflow]** Version 1 runs best-of-N search.
- **I-4.03 [E][workflow]** Version 1 fans N candidates into isolated workspaces.
- **I-4.04 [E][workflow]** A tri-layer veto, budgeted mutation evidence, and progress score produce the
  survivor ranking.
- **I-4.05 [E][observability]** The SSB records why a branch won.
- **I-4.06 [E][workflow]** Version 2 upgrades selection to a tournament.
- **I-4.07 [E][workflow]** Version 3 builds a PUCT tree over SessionDag.
- **I-4.08 [E][concurrency]** PUCT uses virtual loss for parallel expansion.
- **I-4.09 [E][generation]** The generator proposes K divergent candidates using C-26 temperature
  schedules.
- **I-4.10 [E][invariant]** The orchestrator, not the generator, owns the search tree.
- **I-4.11 [E][dependency]** Reuses SessionDag/H-49 as the branch store.
- **I-4.12 [E][integration]** Adds crates/orchestrator/src/search/ladder.rs first and mcts.rs only at v3.
- **I-4.13 [E][rollout]** Replaces linear retry at baby-step dispatch only after the ladder demonstrates a
  measured win.
- **I-4.14 [E][UI]** Feeds SearchNodeEvent rows to the H15-1 branch-board comparison view.
- **I-4.15 [E][failure]** Compile failure creates a HardVeto and Pruned row.
- **I-4.16 [E][observability]** Promoted survivor rows include concrete receipts such as mutant-kill count
  and progress score.
- **I-4.17 [E][workflow]** The winning branch merges after ranking.
- **I-4.18 [E][test]** The LLM-judge-only broken arm must be able to pick a vetoed branch and fail.
- **I-4.19 [E][rollout]** Version 1 must ship before any tree-search code.
- **I-4.20 [A][configuration]** N and K are configurable or unspecified in the summary; the observable
  fixture uses three candidates but does not declare three as the product constant.

#### I-5 — pristine context injector

Source anchor: I-SUMMARY_INDEX.md:110. Metadata: TRUST-STEERING; STRONG; BUILD; M; Wave 2; ID-1 instrument.

- **I-5.01 [E][context]** Rebuilds branch context from the parent chain at expansion time.
- **I-5.02 [E][context]** Hard-excludes failed siblings.
- **I-5.03 [E][context]** Hard-excludes raw tracebacks.
- **I-5.04 [E][context]** Preserves failure information only as a compressed contrastive constraint.
- **I-5.05 [E][measurement]** Samples pristine-versus-accumulated context A/B decisions on the same choice.
- **I-5.06 [E][measurement]** Uses the E-19 reasoning-quality probe and records the measured vividness-tax
  delta.
- **I-5.07 [E][integration]** Extends H-6 masking/H-50 regime machinery without remasking.
- **I-5.08 [E][integration]** Hooks the context budget and prompt-builder assembly seam through a
  pristine_path_filter.
- **I-5.09 [E][UI]** Adds a vividness-tax tile to ReasoningDrawer.
- **I-5.10 [E][test]** A retry payload must contain zero bytes from a prior 5K-line traceback.
- **I-5.11 [E][constraint]** The retained contrastive constraint is at most 30 tokens in the stated
  observable fixture.
- **I-5.12 [E][observability]** Emits ContextPruned rows.
- **I-5.13 [E][determinism]** The A/B delta must be recomputable under ELAI_SEED.
- **I-5.14 [E][test]** A carry-tracebacks broken arm must reinflate context and fail.

#### I-6 — anchor pane slot

Source anchor: I-SUMMARY_INDEX.md:126. Metadata: STEERING; STRONG; BUILD; S; Wave 2.

- **I-6.01 [E][context]** Adds an immutable PrivilegedAnchor context slot.
- **I-6.02 [E][layout]** Appends the anchor last, after the final cache breakpoint.
- **I-6.03 [E][data]** Carries the TaskSpec in the anchor.
- **I-6.04 [E][data]** Carries unresolved claims in the anchor.
- **I-6.05 [E][data]** Carries active constraints in the anchor.
- **I-6.06 [E][generation]** Requires the model to emit a short acknowledgement tag reflecting the active
  constraint before generation.
- **I-6.07 [E][integration]** Extends ContextSlot with AnchorPane without rebalancing other slots.
- **I-6.08 [E][observability]** Reuses ContextReduction rows and introduces no new action type.
- **I-6.09 [E][UI]** Has no separate UI because it is prompt-internal.
- **I-6.10 [E][test]** On a 25K-plus-token task the final journaled prompt block must be the anchor.
- **I-6.11 [E][measurement]** Measures constraint-adherence improvement against a no-anchor arm rather than
  asserting it.
- **I-6.12 [E][invariant]** The cache-prefix stability assertion must continue to pass.

#### I-7 — cascading escalation + offline batch

Source anchor: I-SUMMARY_INDEX.md:141. Metadata: CHEAPER-SPEED; STRONG; BUILD; M; Wave 4; depends on I-11;
IC-1.

- **I-7.01 [E][workflow]** Runs wide cheap branches on the local lane first.
- **I-7.02 [E][workflow]** Applies deterministic checks locally.
- **I-7.03 [E][routing]** Escalates only after local failure.
- **I-7.04 [E][context]** Condenses the local failure set into I-11 BeliefState before escalation.
- **I-7.05 [E][routing]** Escalates synchronously to the cloud architect.
- **I-7.06 [E][constraint]** Keeps the Batch API in H-5 offline lanes.
- **I-7.07 [E][constraint]** Makes interactive-path batch routing structurally impossible.
- **I-7.08 [E][integration]** Hooks cloud_mode/escalation_policy and emits EscalationCascade rows.
- **I-7.09 [E][UI]** Shows the cascade trace in the cost cockpit.
- **I-7.10 [E][privacy/cost]** A locally solvable task produces LocalWin and zero journaled cloud spend.
- **I-7.11 [E][workflow]** A local-failure fixture escalates exactly once.
- **I-7.12 [E][constraint]** The observable escalated payload is at most 2K tokens.
- **I-7.13 [E][observability]** Records Condensed then Escalated rows and derives payload size from the
  journal.
- **I-7.14 [E][correction]** Explicitly rejects speculative racing against the Batch API as impossible.

#### I-8 — aletheia voluntary abort

Source anchor: I-SUMMARY_INDEX.md:156. Metadata: CANT-LIE; STRONG; BUILD; M; Wave 1.

- **I-8.01 [E][tool]** Adds an acknowledge_infeasible/abort_trajectory tool to the tool schema.
- **I-8.02 [E][reward]** Credits voluntary abort through the existing abstention-aware H-39/RD-1 score.
- **I-8.03 [E][lifecycle]** Rolls an aborted trajectory back to the last good H-49 trunk node.
- **I-8.04 [E][cache]** Hashes the aborted AST or approach shape.
- **I-8.05 [E][cache]** Stores aborted-path hashes in a dead-end cache.
- **I-8.06 [E][routing]** Prunes an isomorphic future branch before an LLM call.
- **I-8.07 [E][safety]** Aborts and penalizes deterministic checks that spin longer than 15 seconds.
- **I-8.08 [E][integration]** Adds the tool through the existing three-arm catalog pattern.
- **I-8.09 [E][integration]** Hooks the search/retry loop.
- **I-8.10 [E][data]** Keys the dead-end cache by blake3 of AST shape.
- **I-8.11 [E][UI]** Displays an abort/dead-end strip in ReasoningDrawer.
- **I-8.12 [E][observability]** Impossible tasks emit TrajectoryAborted(Voluntary) and a journaled rollback.
- **I-8.13 [E][observability]** Isomorphic reattempts emit DeadEndHash and must incur zero model calls.
- **I-8.14 [E][observability]** A spin fixture emits HaltingBound within budget.

#### I-9 — error amplification telemetry

Source anchor: I-SUMMARY_INDEX.md:172. Metadata: PROVABLE; STRONG; BUILD; M; Wave 4.

- **I-9.01 [E][measurement]** Defines alpha_t as whether a corrupted step amplifies or contracts downstream
  error.
- **I-9.02 [E][test]** Corrupts step t in a frozen fixture pipeline.
- **I-9.03 [E][dependency]** Reuses C-28 mutation operators as the corruption source.
- **I-9.04 [E][measurement]** Measures downstream recovery after corruption.
- **I-9.05 [E][control]** Actively anneals when live alpha exceeds 1.
- **I-9.06 [E][control]** Annealing lowers generation temperature.
- **I-9.07 [E][control]** Annealing tightens the grammar.
- **I-9.08 [E][measurement]** Measures ELAI's own alpha baseline before claiming contraction.
- **I-9.09 [E][CLI]** Adds elai bench error-amplification.
- **I-9.10 [E][integration]** Adds an error_amplification_runner using embedded fixtures.
- **I-9.11 [E][UI]** Adds an alpha-trend cockpit tile.
- **I-9.12 [E][observability]** Emits ErrorAmplificationMeasured with a seed-recomputable alpha.
- **I-9.13 [E][test]** A no-repair arm must yield alpha greater than 1 while the repaired arm contracts.
- **I-9.14 [E][observability]** A live alpha-greater-than-1 fixture must journal the resulting temperature
  drop.

### Track 14 — cache-invariant economics middleware

#### I-10 — prefix invariance + ttl tiering + cache-hit hard guard [TRACK 14 KEYSTONE]

Source anchor: I-SUMMARY_INDEX.md:197. Metadata: CHEAPER; STRONG; BUILD-on-shipped; M; Wave 1; IC-4/IC-5.

- **I-10.01 [E][cache]** Enforces a byte-frozen prompt prefix end to end.
- **I-10.02 [E][constraint]** Excludes timestamps from the frozen prefix.
- **I-10.03 [E][constraint]** Excludes UUIDs from the frozen prefix.
- **I-10.04 [E][constraint]** Excludes mutable endpoints from the frozen prefix.
- **I-10.05 [E][tools]** Restricts tools dynamically through tool_choice or allowed_tools payload
  attributes.
- **I-10.06 [E][constraint]** Never mutates prompt text to express dynamic tool restriction.
- **I-10.07 [E][measurement]** Chooses inline-frozen versus H-4-deferred tool arrays by measured cache-hit
  and token cost for each catalog size.
- **I-10.08 [E][cache]** Uses a one-hour TTL at the frozen-prefix breakpoint.
- **I-10.09 [E][cache]** Uses a five-minute TTL for volatile tails.
- **I-10.10 [E][planning]** A cache-economics planner selects tiers using inter-call gap statistics from
  the journal.
- **I-10.11 [E][guard]** Halts when unexplained misses exceed 10 percent for identical task classes.
- **I-10.12 [E][diagnostics]** Dumps a blake3 prefix diff on guard trip.
- **I-10.13 [E][diagnostics]** Names the leaking mutation and byte range.
- **I-10.14 [E][scheduling]** Uses burst-amortized scheduling for long local validation loops.
- **I-10.15 [E][fallback]** Demotes heartbeat traffic to a last resort.
- **I-10.16 [E][integration]** Extends shipped H-1/H-2/H-9 machinery only with the TTL, planner, and guard
  layer if those pieces already ship.
- **I-10.17 [E][UI]** Adds TTL and economics fields to the H-9 cost tile.
- **I-10.18 [E][observability]** Emits CacheEconomicsEvent TtlPlanned for real cloud calls.
- **I-10.19 [E][test]** A deliberately injected prefix timestamp trips the guard within one session.
- **I-10.20 [E][observability]** Emits HitRateGuardTripped and PrefixDiffDumped.
- **I-10.21 [A][decision]** Inline versus deferred tool-array layout is deliberately unresolved until
  measured per catalog size.

#### I-11 — pabu belief-state handoff

Source anchor: I-SUMMARY_INDEX.md:216. Metadata: CHEAPER-ORCH; STRONG; BUILD; M; Wave 2; ID-3; IC-8.

- **I-11.01 [E][data]** Defines a typed, versioned BeliefState.
- **I-11.02 [E][data]** Records what was tried.
- **I-11.03 [E][data]** Records why attempts failed.
- **I-11.04 [E][data]** Records current AST state.
- **I-11.05 [E][data]** Records active constraints.
- **I-11.06 [E][knowledge]** Links entities to the DKG.
- **I-11.07 [E][workflow]** Replaces raw history on local-to-cloud escalation.
- **I-11.08 [E][workflow]** Replaces raw history on architect-to-editor handoff.
- **I-11.09 [E][workflow]** Replaces raw history on session resume.
- **I-11.10 [E][observability]** Journals every handoff as an ID-3 receipt.
- **I-11.11 [E][measurement]** Periodically reruns tasks from full history versus BeliefState.
- **I-11.12 [E][measurement]** Records the measured success delta as compression cost.
- **I-11.13 [E][integration]** Shares the digest engine with H-51 while keeping cross-session scope
  distinct.
- **I-11.14 [E][recovery]** Uses H-6 recover_stubs to re-expand detail.
- **I-11.15 [E][UI]** Adds a BeliefState inspector row to SessionsDrawer.
- **I-11.16 [E][constraint]** Local-to-cloud escalation carries only prefix plus BeliefState.
- **I-11.17 [E][measurement]** The summary's observable compares an approximately 2K-token handoff with a
  20K-or-more-token history control.
- **I-11.18 [E][observability]** Emits BeliefStateEvent(Condensed/Handoff) with schema version.
- **I-11.19 [E][observability]** Emits seed-recomputable HonestyDelta results.
- **I-11.20 [E][fail-closed]** A schema-version mismatch yields a typed closed failure, never a silent
  parse.

#### I-12 — evidence-gated playbook curator

Source anchor: I-SUMMARY_INDEX.md:232. Metadata: STEERING; STRONG; BUILD; M; Wave 3; ID-6.

- **I-12.01 [E][learning]** Curates playbook/rule entries in an ACE-shaped learning loop.
- **I-12.02 [E][promotion]** Promotes an entry only when it links to at least k SSB-verified successful
  outcomes.
- **I-12.03 [E][measurement]** Stores each promoted entry's measured lift.
- **I-12.04 [E][measurement]** Measures lift through cheap interleaved on/off injection.
- **I-12.05 [E][lifecycle]** Demotes entries using the same evidence ledger.
- **I-12.06 [E][guard]** Refuses merged rules that regress the C-28 mutation suite.
- **I-12.07 [E][interop]** Exports verified content to CLAUDE.md.
- **I-12.08 [E][interop]** Exports verified content to AGENTS.md.
- **I-12.09 [E][interop]** Exports verified content to skill formats.
- **I-12.10 [E][dependency]** Feeds the existing Phase 3F proposed-to-shadow-to-active rule lifecycle.
- **I-12.11 [E][operations]** Runs the curator from an offline Layer-1A tick.
- **I-12.12 [E][UI]** Displays playbook rows in RulesDrawer.
- **I-12.13 [E][observability]** PlaybookEvent(Promoted) carries the supporting outcome row IDs.
- **I-12.14 [E][failure]** A mutation-regressing merge is refused by a failed ValidationRun.
- **I-12.15 [E][lifecycle]** Stale entries demote.
- **I-12.16 [E][CLI]** elai playbook export writes an AGENTS.md block.
- **I-12.17 [E][traceability]** Every exported entry cites its evidence rows.
- **I-12.18 [E][fail-closed]** Zero-evidence promotion fails.
- **I-12.19 [A][configuration]** The minimum success count k is named but not assigned a value in the
  summary.

#### I-13 — memory guard + dual serialization

Source anchor: I-SUMMARY_INDEX.md:250. Metadata: SECURITY-EFFICIENCY; STRONG; BUILD; M; Wave 2; IC-10.

- **I-13.01 [E][security]** Scans every Recall memory write at write time.
- **I-13.02 [E][security]** Scans every Archival memory write at write time.
- **I-13.03 [E][security]** Scans every playbook memory write at write time.
- **I-13.04 [E][security]** Detects injection patterns deterministically.
- **I-13.05 [E][security]** Detects invisible-Unicode and multibyte tricks.
- **I-13.06 [E][security]** Detects secrets.
- **I-13.07 [E][implementation]** Uses regex and Aho-Corasick first.
- **I-13.08 [E][dependency]** Allows a YARA engine only as a user-approved dependency.
- **I-13.09 [E][serialization]** Defines to_ui_dict as the full dashboard representation.
- **I-13.10 [E][serialization]** Defines to_llm_content as the condensed context representation.
- **I-13.11 [E][tool]** Adds TraceQuery for on-demand expansion of exact disk line ranges.
- **I-13.12 [E][integration]** Hooks memory write choke points and crates/tools tool_result.rs.
- **I-13.13 [E][UI]** Adds guard badges to MemoryFactsDrawer.
- **I-13.14 [E][failure]** Blocks a seeded invisible-Unicode poison attempt.
- **I-13.15 [E][observability]** Emits MemoryGuardEvent(PoisonBlocked) with the pattern class.
- **I-13.16 [E][behavior]** A 500-file listing remains complete in the dashboard but enters model context
  in condensed form.
- **I-13.17 [E][observability]** Journals the serialization token delta and DualSerialized.
- **I-13.18 [E][observability]** TraceQuery emits TraceQueryExpanded for the exact requested range.
- **I-13.19 [E][rollout]** Requires false-positive bounds on at least three real runs before default-on.

#### I-14 — deterministic context compression

Source anchor: I-SUMMARY_INDEX.md:267. Metadata: CHEAPER; STRONG; BUILD; M; Wave 2; ID-3; IC-2/IC-3.

- **I-14.01 [E][compression]** Makes Tier 1 compression deterministic.
- **I-14.02 [E][compression]** Uses headless LSP slicing to extract the exact AST node, declaration, or
  failing line referenced by a compiler error.
- **I-14.03 [E][compression]** Uses tree-sitter AST folding.
- **I-14.04 [E][compression]** Replaces irrelevant function bodies with an elision marker.
- **I-14.05 [E][invariant]** Preserves signatures and types to retain BPE priors.
- **I-14.06 [E][compression]** Allows Tier 2 token-classification compression only for prose payloads.
- **I-14.07 [E][resource]** Checks VRAM budget before Tier 2 use.
- **I-14.08 [E][constraint]** Bans cipher or abbreviation substitution.
- **I-14.09 [E][replay]** Emits a reversible ContextTransformReceipt for every transform.
- **I-14.10 [E][replay]** Enables replay to reconstruct the exact bytes the model saw.
- **I-14.11 [E][dependency]** Shares the E-52 headless rust-analyzer client and existing tree-sitter
  grammars.
- **I-14.12 [E][integration]** Hooks the tool-result-to-context seam through context_transform.rs.
- **I-14.13 [E][UI]** Displays transform receipts in the Transcript drawer.
- **I-14.14 [E][measurement]** Compresses the stated 10K-token panic fixture to at most 500 tokens.
- **I-14.15 [E][recovery]** A receipt's reverse pointer expands to bytes matching the original blake3.
- **I-14.16 [E][measurement]** Requires fixture pass rate to remain unchanged versus uncompressed input.
- **I-14.17 [E][fail-closed]** Refuses Tier 2 use without VRAM headroom.
- **I-14.18 [E][fail-closed]** Running transforms without receipts is a failure.

#### I-15 — semantic cache + blake3 fast path

Source anchor: I-SUMMARY_INDEX.md:285. Metadata: CHEAPER-SPEED; STRONG; EXTENSION; S; Wave 1.

- **I-15.01 [E][cache]** Caches intermediate DKG chunk summaries.
- **I-15.02 [E][cache]** Caches intermediate research extracts.
- **I-15.03 [E][cache]** Serves intermediate summaries for semantically similar queries.
- **I-15.04 [E][performance]** Runs an exact blake3 lookup before any vector math.
- **I-15.05 [E][performance]** Makes exact-repeat lookup O(1).
- **I-15.06 [E][security/scope]** Restricts durable caching to idempotent grounding content.
- **I-15.07 [E][integration]** Extends C-30/H-8 OracleAnswerCache and hooks A-RAG summarization.
- **I-15.08 [E][observability]** Reuses OracleCacheHit and OracleCacheMiss rows.
- **I-15.09 [E][UI]** Shows a cache-source badge on research answers.
- **I-15.10 [E][measurement]** Measures run-two RAG input-token reduction instead of claiming the paper's
  percentage.
- **I-15.11 [E][test]** Exact repeats must issue zero ONNX calls.
- **I-15.12 [E][observability]** Measures exact-path latency and call count from traces.
- **I-15.13 [E][guard]** Code-generation queries can never hit the durable cache.

#### I-16 — kv-cache pruning + quant eval

Source anchor: I-SUMMARY_INDEX.md:299. Metadata: LOCAL-VRAM; MODERATE; ACTIVATION+EVAL; M; Wave 5;
RunPod-gated; ID-5.

- **I-16.01 [E][evaluation]** Evaluates sink-protected KV pruning rather than assuming adoption.
- **I-16.02 [E][cache]** Pins the first four tokens during pruning.
- **I-16.03 [E][evaluation]** Evaluates Int4 KV quantization.
- **I-16.04 [E][measurement]** Measures fixture pass rate per configuration.
- **I-16.05 [E][measurement]** Measures VRAM per configuration.
- **I-16.06 [E][measurement]** Measures throughput per configuration.
- **I-16.07 [E][baseline]** Compares with the E-27 AWQ baseline.
- **I-16.08 [E][evaluation]** Keeps the MoE sidecar retarget as a separate checkbox.
- **I-16.09 [E][constraint]** Does not assume a model swap because prior swaps lost on measurement.
- **I-16.10 [E][invariant]** Preserves the FA2 mandate and LogitsProcessor chain.
- **I-16.11 [E][integration]** Adds sidecar flags and an E-27-shaped evaluation harness.
- **I-16.12 [E][CLI]** elai bench kv-tune writes KvCacheTuned with vram_gb, tok_s, and fixture_pass_rate.
- **I-16.13 [E][promotion]** Adopts a configuration only if pass rate remains within band and VRAM falls.
- **I-16.14 [E][observability]** Records the adopt/reject decision as a row.
- **I-16.15 [E][operations]** Ships a no-GPU deterministic fixture harness immediately while the real
  RunPod evaluation remains separately gated.
- **I-16.16 [A][threshold]** The acceptable pass-rate band is not specified in the summary.

### Track 15 — deterministic gates and anti-gaming verification

#### I-17 — tri-layer evaluation stack [BLOCKING for I-4]

Source anchor: I-SUMMARY_INDEX.md:324. Metadata: TRUST-PROVABLE; STRONG; BUILD by composition; M; Wave 3;
ID-4.

- **I-17.01 [E][evaluation]** Defines Layer 1 as a supreme hard veto.
- **I-17.02 [E][evaluation]** Layer 1 includes compile results.
- **I-17.03 [E][evaluation]** Layer 1 includes test results.
- **I-17.04 [E][evaluation]** Layer 1 includes F-1 VibeGate.
- **I-17.05 [E][evaluation]** Layer 1 includes F-9 dependency gate.
- **I-17.06 [E][evaluation]** Layer 1 includes D-11 constructive checkers.
- **I-17.07 [E][invariant]** Any Layer-1 failure forces score 0.0.
- **I-17.08 [E][invariant]** No later layer can override a hard veto.
- **I-17.09 [E][evaluation]** Layer 2 scores survivors only.
- **I-17.10 [E][evaluation]** Layer 2 uses the D-2 PPM seam when live and PRM-lite/E-19 until then.
- **I-17.11 [E][evaluation]** Layer 3 escalates disagreement to a cross-family D-13 adversary.
- **I-17.12 [E][safety]** Applies a 15-second empirical halting bound to deterministic Layer-1 checks.
- **I-17.13 [E][workflow]** Aborts the branch when a deterministic check exceeds that bound.
- **I-17.14 [E][reliability]** Reruns candidate oracles under seed variation to detect flakes.
- **I-17.15 [E][quarantine]** An oracle whose result flips is quarantined.
- **I-17.16 [E][invariant]** A quarantined oracle can neither veto nor vouch until rehabilitated.
- **I-17.17 [E][observability]** The flake decision and excluded vote are ledgered.
- **I-17.18 [E][extension]** Defines a Tier-2 cross-language verifier-pack seam for TypeScript and Python
  behind the D-3 adapter trait.
- **I-17.19 [E][integration]** Adds verifier_stack.rs for use by the I-4 ranker and baby-step audit seam.
- **I-17.20 [E][UI]** Shows per-branch verdict chips.
- **I-17.21 [E][observability]** A hallucinated import emits TriLayerVerdict HardVeto before any Layer-2
  call.
- **I-17.22 [E][test]** A hard-vetoed candidate must show Layer-2 call count zero.
- **I-17.23 [E][observability]** Survivor ordering emits ScoredSurvivor.
- **I-17.24 [E][observability]** A seeded flaky test emits FlakeQuarantined.
- **I-17.25 [E][test]** The Layer-2-overrides-Layer-1 broken arm must resurrect a vetoed branch and fail.
- **I-17.26 [A][deferred]** TypeScript/Python verifier packs are described only as Tier 2, with no delivery
  threshold in the summary.

#### I-18 — upip chain + sigstore + ai-bom

Source anchor: I-SUMMARY_INDEX.md:342. Metadata: AUDITABLE; STRONG; EXTENSION; M; Wave 3; IC-11.

- **I-18.01 [E][integrity]** Adds chain_prev_hash to each SSB row.
- **I-18.02 [E][integrity]** Uses the SSB single writer to make row order tamper-evident.
- **I-18.03 [E][integrity]** Computes a hierarchical STATE stack hash.
- **I-18.04 [E][integrity]** Computes a hierarchical DEPS stack hash.
- **I-18.05 [E][integrity]** Computes a hierarchical PROCESS stack hash.
- **I-18.06 [E][integrity]** Computes a hierarchical RESULT stack hash.
- **I-18.07 [E][integrity]** Computes a hierarchical VERIFY stack hash.
- **I-18.08 [E][artifact]** Seals the five-layer hash into the RRA.
- **I-18.09 [E][attestation]** Offers an opt-in Rekor transparency-log push.
- **I-18.10 [E][dependency]** Uses cosign keyless only as an opt-in dependency after anti-slopsquat and
  approval.
- **I-18.11 [E][artifact]** Adds an AI-BOM/CycloneDX-ML entry beside the attestation.
- **I-18.12 [E][interop]** Makes the artifact express the cited OWASP AISVS AC.8 expectation.
- **I-18.13 [E][offline]** Keeps local blake3 Merkle verification as the offline default.
- **I-18.14 [E][CLI]** Extends elai verify-attestation with chain and Rekor options.
- **I-18.15 [E][UI]** Adds a chain badge to the RRA viewer.
- **I-18.16 [E][failure]** Editing any mid-run row breaks chain verification at that exact row.
- **I-18.17 [E][diagnostics]** RowChainSealed(ChainBroken) names the failing sequence number.
- **I-18.18 [E][test]** The AI-BOM must validate against the CycloneDX schema.
- **I-18.19 [E][offline]** A Rekor inclusion proof must verify without rerunning ELAI.
- **I-18.20 [E][fail-closed]** Cargo.lock mutation during a run invalidates the DEPS layer and hard-halts.
- **I-18.21 [E][coordination]** The chain field shares a sealed-record layout with I-45 objective hash and
  therefore requires coordinated schema changes.

#### I-19 — tdd enforcer preflight

Source anchor: I-SUMMARY_INDEX.md:359. Metadata: CANT-WRECK; STRONG; EXTENSION; S-M; Wave 3; IC-12.

- **I-19.01 [E][gate]** Adds only the missing symbol-level PreToolUse leg rather than rebuilding the
  existing TESTING_FIRST FSM or anti-over-mock machinery.
- **I-19.02 [E][scope]** Applies to source-file edits during EXECUTING.
- **I-19.03 [E][gate]** Requires a corresponding test-symbol delta before the write lands.
- **I-19.04 [E][analysis]** Uses tree-sitter AST to verify a test references the modified function.
- **I-19.05 [E][gate]** Requires a syntax-only dry run before the write lands.
- **I-19.06 [E][language]** Uses cargo check --tests for Rust.
- **I-19.07 [E][language]** Uses pytest --collect-only for Python.
- **I-19.08 [E][exemption]** Exempts pure structural refactors.
- **I-19.09 [E][analysis]** Determines structural-refactor exemption through coverage delta.
- **I-19.10 [E][rollout]** Bounds false positives on at least three real runs before default-on.
- **I-19.11 [E][integration]** Adds the axis to F-1 VibeGate's compound PreToolUse evaluator rather than
  forking the hook.
- **I-19.12 [E][UI]** Uses the existing gate panel for block reasons.
- **I-19.13 [E][failure]** Editing in a new public function without a test-symbol delta is refused before
  disk.
- **I-19.14 [E][observability]** Emits TddPreflightBlocked MissingTestDelta.
- **I-19.15 [E][behavior]** The same edit with a matching failing test is allowed to land.
- **I-19.16 [E][observability]** Rename-only refactors emit RefactorExempted.
- **I-19.17 [A][threshold]** The acceptable false-positive bound is not numerically specified.

#### I-20 — budgeted mutation scheduler [TRACK 15 KEYSTONE]

Source anchor: I-SUMMARY_INDEX.md:374. Metadata: VERIFIED; STRONG; EXTENSION; M-L; Wave 3; ID-5; IC-14.

- **I-20.01 [E][testing]** Runs mutation testing inside the generation/audit loop.
- **I-20.02 [E][selection]** Selects mutants only from the changed hunk's data-flow cone.
- **I-20.03 [E][analysis]** Computes reachability through symbol-graph impact_radius.
- **I-20.04 [E][budget]** Time-boxes mutation work per hunk.
- **I-20.05 [E][sampling]** Samples mutants rather than requiring exhaustive enumeration.
- **I-20.06 [E][measurement]** Reports a confidence-interval mutation score.
- **I-20.07 [E][control]** Stops early when the confidence interval clears or fails the threshold.
- **I-20.08 [E][cache]** Caches results in the SSB.
- **I-20.09 [E][cache-key]** Keys cache entries by function content hash.
- **I-20.10 [E][cache-key]** Also binds cache entries to the test-set hash.
- **I-20.11 [E][performance]** Unchanged code does not repay mutation execution.
- **I-20.12 [E][feedback]** Converts surviving mutants into structured counterexample Directives.
- **I-20.13 [E][feedback]** Counterexamples identify the mutation distinction, operator, and source line.
- **I-20.14 [E][control]** Bounds generator retries after survivor feedback.
- **I-20.15 [E][extension]** Supports proptest and Hypothesis relation adapters as a parallel lane behind
  the D-3 trait.
- **I-20.16 [E][integration]** Wraps the existing mutation runner with mutation_budget.rs at audit and
  search seams.
- **I-20.17 [E][UI]** Shows a per-hunk mutation chip.
- **I-20.18 [E][observability]** Emits MutationBudgetedRun CiCleared with recomputable parameters and
  sample.
- **I-20.19 [E][observability]** Unchanged-function reaudit emits CacheHit and runs zero mutants.
- **I-20.20 [E][observability]** A survivor emits SurvivorDirective with operator and line.
- **I-20.21 [E][measurement]** Thresholds come from ELAI measurements, not paper constants.
- **I-20.22 [A][configuration]** Time-box length, sample policy, retry bound, and CI threshold are not
  numerically fixed in the summary.

#### I-21 — sprint contracts + ralph loop

Source anchor: I-SUMMARY_INDEX.md:393. Metadata: TRUST-ANTI-BIAS; STRONG; BUILD; M; Wave 3.

- **I-21.01 [E][separation]** Gives generator and evaluator zero shared context.
- **I-21.02 [E][planning]** The generator proposes a structured Sprint Contract.
- **I-21.03 [E][data]** A Sprint Contract contains the goal.
- **I-21.04 [E][data]** A Sprint Contract contains files in scope.
- **I-21.05 [E][data]** A Sprint Contract contains acceptance conditions.
- **I-21.06 [E][data]** A Sprint Contract contains bounded scope.
- **I-21.07 [E][workflow]** An isolated evaluator approves or rejects before execution.
- **I-21.08 [E][loop]** Failed exit checks reinject the trace into a bounded Ralph loop.
- **I-21.09 [E][budget]** The Ralph loop runs at most three times.
- **I-21.10 [E][escalation]** The third failure escalates to the D-13 cross-family adversary.
- **I-21.11 [E][output]** The adversary must produce a concrete counterexample.
- **I-21.12 [E][model-setting]** Runs the evaluator at temperature zero.
- **I-21.13 [E][role]** Retains a hostile-critic evaluator role because this task is alignment-dependent.
- **I-21.14 [E][integration]** Reuses H-37 interactive plan-approval gate machinery.
- **I-21.15 [E][security]** Uses the I-1 supervisor as the isolation substrate.
- **I-21.16 [E][UI]** Adds a Sprint Contract card beside PlanApprovalCard.
- **I-21.17 [E][failure]** A vague goal is rejected before execution.
- **I-21.18 [E][test]** The evaluator's journaled context must contain zero generator turns.
- **I-21.19 [E][observability]** Emits RalphReinjected and, on the third failure, AdversaryEscalated.
- **I-21.20 [E][determinism]** Approval and execution rows must recompute under ELAI_SEED.

#### I-22 — progress-shaped rewards

Source anchor: I-SUMMARY_INDEX.md:411. Metadata: ANTI-GAMING; STRONG; BUILD; M; Wave 4; depends on I-4;
ID-4.

- **I-22.01 [E][reward]** Scores trajectories by progress delta phi(h_t) minus phi(h_t-1).
- **I-22.02 [E][reward]** Includes change in passing tests in phi.
- **I-22.03 [E][reward]** Includes falling compile-error count in phi.
- **I-22.04 [E][reward]** Includes covered requirements in phi.
- **I-22.05 [E][invariant]** Excludes LLM-opinion components from phi.
- **I-22.06 [E][anti-gaming]** Uses delta rather than absolute scores to prevent score inflation.
- **I-22.07 [E][localization]** Finds the first failing step.
- **I-22.08 [E][recovery]** Truncates the branch at that first error.
- **I-22.09 [E][recovery]** Preserves the valid prefix by forking H-49 at the truncation point.
- **I-22.10 [E][anti-spiral]** Decays reward during no-progress loops.
- **I-22.11 [E][search]** Backtracks after reward decay and expands a sibling.
- **I-22.12 [E][integration]** Shapes the D-2 PPM Layer-2 score and I-4 PUCT value.
- **I-22.13 [E][UI]** Shows a per-branch progress sparkline.
- **I-22.14 [E][observability]** Emits DecayApplied for isomorphic loops.
- **I-22.15 [E][observability]** Emits FirstErrorTruncated and an H-49 fork node.
- **I-22.16 [E][traceability]** Every phi component must be enumerable from deterministic journal data.

#### I-23 — unified conformal calibration

Source anchor: I-SUMMARY_INDEX.md:426. Metadata: SMARTER-ORCH; STRONG; EXTENSION; S-M; Wave 1; IC-13.

- **I-23.01 [E][architecture]** Exposes one conformal-calibration implementation.
- **I-23.02 [E][consumer]** Uses it for D-16 refusal.
- **I-23.03 [E][consumer]** Uses it for G-7 claim scores.
- **I-23.04 [E][consumer]** Uses it for I-4 search priors and uncertainty forks.
- **I-23.05 [E][routing]** High epistemic uncertainty forces exploration forks.
- **I-23.06 [E][invariant]** Prevents tau and coverage semantics from drifting between consumers.
- **I-23.07 [E][integration]** Re-exports the shipped calibration implementation through one facade and
  migrates call sites.
- **I-23.08 [E][UI]** Adds no new UI and preserves existing refusal surfaces.
- **I-23.09 [E][observability]** Refusal and search forks in one run must join to the same calibration ID
  and tau row.
- **I-23.10 [E][test]** A split-calibrations arm permits drift and must fail the new contract.
- **I-23.11 [E][regression]** All existing D-16 observable effects must remain unchanged.
- **I-23.12 [E][scope]** Adds no new mathematics; this is a unification feature.

#### I-24 — isomorphic canary anti-gaming

Source anchor: I-SUMMARY_INDEX.md:442. Metadata: ANTI-GAMING; STRONG; EXTENSION; S-M; Wave 4.

- **I-24.01 [E][canary]** Injects AST-preserving obfuscated canaries into the Layer-2 scoring stream.
- **I-24.02 [E][transform]** Includes for-to-while or while-to-for transformations.
- **I-24.03 [E][transform]** Includes variable renames.
- **I-24.04 [E][invariant]** Preserves semantics while perturbing style.
- **I-24.05 [E][detection]** Detects style bias when the scorer's rating falls on an isomorphic variant.
- **I-24.06 [E][observability]** Logs the canary pair and both scores.
- **I-24.07 [E][mitigation]** Neutralizes a style-biased scorer's weight for the run.
- **I-24.08 [E][lifecycle]** Keeps the scorer neutralized until recalibration.
- **I-24.09 [E][dependency]** Extends the C-29 metamorphic engine and complements, rather than duplicates,
  H-44 candidate traps.
- **I-24.10 [E][UI]** Adds a scorer-health badge.
- **I-24.11 [E][observability]** Emits PrmCanaryEvent(StyleBiasDetected) and ScorerNeutralized.
- **I-24.12 [E][behavior]** An unbiased scorer emits CanaryInjected only.
- **I-24.13 [E][security]** Freezes the canary set out of band.
- **I-24.14 [E][invariant]** The scored model cannot author its own canaries.

### Track 16 — routing, budgets, and measurement

#### I-25 — ledger-trained router

Source anchor: I-SUMMARY_INDEX.md:467. Metadata: CHEAPER-SPEED; STRONG; EXTENSION; M; Wave 4; ID-5/ID-6.

- **I-25.01 [E][learning]** Trains a small router on ELAI's own verified journal outcomes.
- **I-25.02 [E][feature]** Uses task class as a routing feature.
- **I-25.03 [E][feature]** Uses file count as a routing feature.
- **I-25.04 [E][feature]** Uses DKG centrality as a routing feature.
- **I-25.05 [E][feature]** Uses historical cost shape as a routing feature.
- **I-25.06 [E][label]** Learns outcomes per task features, execution lane, and cost.
- **I-25.07 [E][routing]** Pre-routes known-expensive AST shapes to the deep lane.
- **I-25.08 [E][cost]** Avoids burning cheap-lane tokens on shapes known to require deep execution.
- **I-25.09 [E][promotion]** Treats the learned router as an ID-6 ledger-gated artifact.
- **I-25.10 [E][privacy]** Composes with E-20 privacy routing masks.
- **I-25.11 [E][integration]** Hooks escalation_policy/cloud_mode and adds an offline trainer.
- **I-25.12 [E][UI]** Shows route explanations in the cost cockpit.
- **I-25.13 [E][observability]** RouteDecisionEvent includes the feature vector and predicted-versus-actual
  outcome.
- **I-25.14 [E][measurement]** Compares over-escalation and misroute rates with the H-7 heuristic over at
  least three runs.
- **I-25.15 [E][honesty]** Records an honest loss when the learned router does not beat the heuristic.
- **I-25.16 [E][promotion]** A losing router is not promoted and the block is ledgered.

#### I-26 — dynamic thinking budgets

Source anchor: I-SUMMARY_INDEX.md:483. Metadata: STEERING; STRONG; EXTENSION; M; Wave 4.

- **I-26.01 [E][budget]** Scales search compute per node by serial depth.
- **I-26.02 [E][budget]** Gives deeper fixture tasks larger node budgets than shallow tasks for the same
  role.
- **I-26.03 [E][collapse]** At 85 percent budget, halts further exploration.
- **I-26.04 [E][collapse]** Scores the current frontier.
- **I-26.05 [E][collapse]** Synthesizes the best frontier result.
- **I-26.06 [E][early-stop]** Collapses early when Layer-2 scores remain flat across five supersteps.
- **I-26.07 [E][provider]** Maps cloud reasoning to the effort request parameter.
- **I-26.08 [E][compatibility]** Omits manual budget_tokens for Opus 4.7 and later.
- **I-26.09 [E][constraint]** Never forces JSON output for a reasoning role.
- **I-26.10 [E][dependency]** Reuses ReasoningBudgetScaled rows, extending payload only if needed.
- **I-26.11 [E][priority]** Keeps the E-39 hard cost ceiling supreme.
- **I-26.12 [E][UI]** Shows a branch-board budget bar.
- **I-26.13 [E][observability]** Collapse traces contain no post-collapse expansions.
- **I-26.14 [E][measurement]** A plateau fixture reports measured token savings.
- **I-26.15 [E][test]** A provider-builder contract test asserts budget_tokens is structurally absent on
  Opus 4.7-plus requests.

#### I-27 — budgeted convergence instrumentation

Source anchor: I-SUMMARY_INDEX.md:499. Metadata: PROVABLE; STRONG; BUILD; S-M; Wave 4.

- **I-27.01 [E][instrument]** Measures verified-outcome rate against total test-time compute.
- **I-27.02 [E][metric]** Counts nodes expanded.
- **I-27.03 [E][metric]** Counts tokens.
- **I-27.04 [E][metric]** Counts monetary cost.
- **I-27.05 [E][invariant]** Uses W-1-grade verified outcomes rather than pass-at-k claims.
- **I-27.06 [E][visualization]** Produces a Pareto cost/accuracy view.
- **I-27.07 [E][operator]** Lets an operator choose an operating point.
- **I-27.08 [E][honesty]** Reports non-monotonic regions instead of smoothing them.
- **I-27.09 [E][CLI]** Adds elai bench convergence with a fixture suite argument.
- **I-27.10 [E][UI]** Adds a convergence-curve cockpit tile.
- **I-27.11 [E][observability]** Emits ConvergenceCurveMeasured with points recomputable from the journal.
- **I-27.12 [E][analysis]** Names the knee of the curve.
- **I-27.13 [E][analysis]** Reports cost per additional verified task.
- **I-27.14 [E][test]** A skip-verification arm must inflate the curve and fail.

#### I-28 — dag credit assignment

Source anchor: I-SUMMARY_INDEX.md:512. Metadata: SELF-MOD; STRONG; BUILD; M-L; Wave 5; depends on I-4.

- **I-28.01 [E][credit]** Walks backward from COMPLETE through the winning DAG trajectory.
- **I-28.02 [E][credit]** Assigns causal credit to retrieval chunks.
- **I-28.03 [E][credit]** Assigns causal credit to subplans.
- **I-28.04 [E][credit]** Assigns causal credit to template choices.
- **I-28.05 [E][training-data]** Extracts DPO pairs as a byproduct.
- **I-28.06 [E][pair]** Uses the winning branch as the positive.
- **I-28.07 [E][pair]** Uses the highest-Layer-2-scoring vetoed sibling as the fluent-but-wrong negative.
- **I-28.08 [E][retrieval]** Demotes chunks that consistently precede aborts.
- **I-28.09 [E][decline]** Declines Shapley/SHARP credit until RL infrastructure exists.
- **I-28.10 [E][integration]** Runs dag_credit.rs at PhaseComplete.
- **I-28.11 [E][UI]** Optionally displays credit heat on the plan view; absence may be W-6 justified.
- **I-28.12 [E][observability]** Emits journal-rederivable DagCreditAssigned rows.
- **I-28.13 [E][artifact]** Writes DPO pairs whose hashes match sealed RRA entries.
- **I-28.14 [E][threshold]** Demotes a chunk after three consecutive abort-preceding appearances in the
  stated observable.
- **I-28.15 [E][test]** The next seeded retrieval ranking must provably change after ChunkDemoted.
- **I-28.16 [A][future]** Shapley/SHARP is explicitly revisit-only and not part of the current capability.

#### I-29 — diffusion draft lane

Source anchor: I-SUMMARY_INDEX.md:527. Metadata: GENERATIVE; MODERATE; BUILD; L; Wave 5; new-dependency and
network gated with user approval.

- **I-29.01 [E][generation]** Adds a diffusion LLM as a fast, untrusted draft generator.
- **I-29.02 [E][scope]** Uses the lane for drafts and apply-edit streams.
- **I-29.03 [E][trust]** Subjects diffusion output to the identical I-17 and I-20 gates used for other
  generators.
- **I-29.04 [E][concurrency]** Uses CRDT merge semantics for parallel patches to a shared draft.
- **I-29.05 [E][determinism]** Requires parallel draft merges to be byte-identical under seed.
- **I-29.06 [E][validation]** Runs C-32 MiniCritic semantic checkpoints after each merge.
- **I-29.07 [E][decline]** Declines the full TTD-DR denoising-research loop.
- **I-29.08 [E][adapter]** Implements the dLLM client as another RuntimeAdapter variant.
- **I-29.09 [E][dependency]** Requires user approval and anti-slopsquat checks for the new dependency.
- **I-29.10 [E][UI]** Adds a draft-lane badge to generation cards.
- **I-29.11 [E][test]** The same Layer-1 gate that rejects main-generator output must reject a bad diffusion
  draft.
- **I-29.12 [E][observability]** Emits DiffusionDraftEvent DraftVetoed and CrdtMerged.
- **I-29.13 [E][feature-flag]** With the lane flag absent, existing paths are untouched.
- **I-29.14 [E][rollout]** Becomes default-on for drafts only after its corruption rate is bounded.
- **I-29.15 [E][fallback]** Otherwise remains an allowlisted experiment.
- **I-29.16 [A][implementation]** Automerge-class versus hand-rolled deterministic CRDT merge remains an
  explicit choice.

### Track 17 — open-weight flywheel

#### I-30 — verifier-ranked sft exhaust

Source anchor: I-SUMMARY_INDEX.md:554. Metadata: ALIGNMENT; STRONG; BUILD; L; Wave 5; depends on I-4; ID-6;
IC-7.

- **I-30.01 [E][dataset]** Extracts root-to-verified-COMPLETE winning paths from sealed RRAs.
- **I-30.02 [E][eligibility]** Excludes unsealed runs from the dataset.
- **I-30.03 [E][format]** Exports supervised-fine-tuning JSONL.
- **I-30.04 [E][negative-data]** Records highest-Layer-2-scoring branches that failed deterministic checks.
- **I-30.05 [E][semantics]** Treats those negatives as canonical fluent-but-wrong examples.
- **I-30.06 [E][training]** Uses rejection-sampling SFT first.
- **I-30.07 [E][training]** Trains LoRA on the canonical local model.
- **I-30.08 [E][strategy]** Defers RL instability by taking the SFT gain first.
- **I-30.09 [E][provenance]** Every dataset row carries a provenance hash back to its RRA.
- **I-30.10 [E][evaluation]** Uses held-out evaluation and the H-17/H-18 promotion ledger.
- **I-30.11 [E][CLI]** Adds elai dataset export --format sft.
- **I-30.12 [E][operations]** Stores offline training scripts under workflow/training.
- **I-30.13 [E][UI]** Adds a dataset card with counts and provenance.
- **I-30.14 [E][observability]** Emits SftExhaustEvent rows.
- **I-30.15 [E][integrity]** A tampered dataset record must fail RRA provenance verification.
- **I-30.16 [E][traceability]** Negative pairs must reference real vetoed branches.
- **I-30.17 [E][operations]** Keeps the LoRA training run behind a RunPod checkbox.
- **I-30.18 [E][promotion]** Any resulting checkpoint can enter promotion only through ID-6 gates.
- **I-30.19 [E][promotion]** Promotion cites trial count, deflation, and held-out results.

#### I-31 — grpo flywheel [LAST; BLOCKING GUARDS]

Source anchor: I-SUMMARY_INDEX.md:573. Metadata: SELF-IMPROVEMENT; STRONG; BUILD; XL; Wave 5; LAST; ID-6;
IC-7; depends on I-30, H-43/H-44/H-28, and I-24.

- **I-31.01 [E][training]** Runs GRPO on execution-grounded rewards.
- **I-31.02 [E][reward]** Includes compile verdicts.
- **I-31.03 [E][reward]** Includes test verdicts.
- **I-31.04 [E][reward]** Includes mutation verdicts.
- **I-31.05 [E][reward]** Includes null-test verdicts.
- **I-31.06 [E][reward]** Applies XGrammar conformance as a format penalty.
- **I-31.07 [E][service]** Serves rewards through a typed local endpoint.
- **I-31.08 [E][anti-collapse]** Applies KL penalty against mode collapse.
- **I-31.09 [E][training]** Uses I-30 negative-trajectory masking.
- **I-31.10 [E][blocking]** Requires live H-43 execution-grounded fitness and prohibits byte-hash fitness.
- **I-31.11 [E][blocking]** Requires live H-44 behavioral traps.
- **I-31.12 [E][blocking]** Requires I-24 canaries in the scoring stream.
- **I-31.13 [E][blocking]** Requires the H-28 reward-tampering gate.
- **I-31.14 [E][security]** Freezes the evaluator, harness, and thresholds in the R8 set.
- **I-31.15 [E][promotion]** Requires H-17 trial-N evidence.
- **I-31.16 [E][promotion]** Requires H-18 deflation.
- **I-31.17 [E][promotion]** Requires D-6 held-out results.
- **I-31.18 [E][promotion]** Requires the D-7 transfer canary.
- **I-31.19 [E][kill-switch]** Keeps the whole lane behind the existing allowlisted
  darwinian-evolve kill switch.
- **I-31.20 [E][constraint]** Inherits the hard avoidance of DPO.
- **I-31.21 [E][security]** Places the reward endpoint on the I-1 supervisor side so the trained policy
  cannot reach reward code.
- **I-31.22 [E][dependency]** Requires user approval and anti-slopsquat review for new RL dependencies.
- **I-31.23 [E][UI]** Shows flywheel admits, blocks, and KL trend.
- **I-31.24 [E][observability]** Every GrpoRewardComputed/RewardServed row cites a gate-verdict row.
- **I-31.25 [E][test]** A harness-edit reward-hack fixture is hard-blocked by H-28.
- **I-31.26 [E][promotion]** A checkpoint either has a full PromotionLedgered chain or does not promote.
- **I-31.27 [E][operations]** Ships the deterministic fixture harness before the separately gated
  multi-night RunPod run.

### Track 18 — deterministic-over-prompt hardening

#### I-32 — structural role scoping

Source anchor: I-SUMMARY_INDEX.md:604. Metadata: STEERING-TRUST; STRONG; REFACTOR; M; Wave 2; IC-17.

- **I-32.01 [E][role]** Removes narrative personas from executor prompts.
- **I-32.02 [E][role]** Removes narrative personas from context-compiler prompts.
- **I-32.03 [E][role]** Removes narrative personas from researcher prompts.
- **I-32.04 [E][role-contract]** Replaces those personas with schema-first role contracts.
- **I-32.05 [E][role-contract]** Specifies typed inputs and outputs.
- **I-32.06 [E][role-contract]** Specifies explicit constraints.
- **I-32.07 [E][role]** Retains hostile-critic framing for auditors.
- **I-32.08 [E][role]** Retains hostile-critic framing for critics.
- **I-32.09 [E][role]** Retains hostile-critic framing for adversaries.
- **I-32.10 [E][role]** Retains anti-spiral's five lateral personas because creative perturbation is an
  alignment-dependent use.
- **I-32.11 [E][measurement]** Runs a per-role A/B fixture evaluation before flipping prompts.
- **I-32.12 [E][integration]** Uses Phase 2C per-role GRC configurations, role templates, and dispatch
  configs.
- **I-32.13 [E][UI]** Adds no UI because the change is prompt-internal.
- **I-32.14 [E][test]** Executor system prompts must contain a schema contract and zero persona narrative.
- **I-32.15 [E][observability]** Emits RoleContractApplied(AbDelta) with seeded fixture deltas.
- **I-32.16 [E][observability]** Retained auditor framing emits PersonaRetained.
- **I-32.17 [E][promotion]** Any role fixture-pass regression blocks the refactor flip.

#### I-33 — mechanical decision governance

Source anchor: I-SUMMARY_INDEX.md:621. Metadata: TRUST; STRONG; BUILD; M; Wave 2.

- **I-33.01 [E][scope]** Wraps decision-type LLM calls such as mode selection, gate advice, and
  classification.
- **I-33.02 [E][CEFL]** Has the orchestrator externalize the entire candidate set.
- **I-33.03 [E][CEFL]** Restricts the model to scoring or ranking supplied candidates.
- **I-33.04 [E][invariant]** Makes model-invented selection options structurally unparseable.
- **I-33.05 [E][I6Q]** Validates rationale quality deterministically.
- **I-33.06 [E][I6Q]** Enforces a minimum rationale length.
- **I-33.07 [E][I6Q]** Enforces lexical diversity.
- **I-33.08 [E][I6Q]** Rejects vacuous but superficially compliant rationales.
- **I-33.09 [E][integrity]** Uses commit-reveal for stochastic gate choices.
- **I-33.10 [E][integrity]** Commits blake3(seed, choice) before revealing the choice.
- **I-33.11 [E][anti-gaming]** Prevents the model from exploiting choice ordering.
- **I-33.12 [E][metric]** Ledgers cosmetic-deadlock rate.
- **I-33.13 [E][metric]** Ledgers deferral information utilization.
- **I-33.14 [E][metric]** Ledgers framing-success rate.
- **I-33.15 [E][measurement]** Measures governance decoupling under task stress locally rather than
  adopting the dossier result.
- **I-33.16 [E][integration]** Wraps prompt_decision and other LLM-decision seams in
  decision_governance.rs.
- **I-33.17 [E][UI]** Adds a governance-metrics tile.
- **I-33.18 [E][failure]** A vacuous rationale emits GovernancePrimitiveFired I6qRejected.
- **I-33.19 [E][test]** Reversing field order must produce the same decision on the framing-inversion
  fixture.
- **I-33.20 [E][observability]** CommitRevealSealed rows must put commit before reveal.
- **I-33.21 [E][test]** An out-of-order commit/reveal arm fails.

#### I-34 — crane hybrid + grammar pinning

Source anchor: I-SUMMARY_INDEX.md:641. Metadata: TRUST-CANT-LIE; STRONG; BUILD; M; Wave 1; IC-18.

- **I-34.01 [E][decoding]** Activates CRANE-style grammar augmentation on the local lane.
- **I-34.02 [E][decoding]** Permits free-form thinking tokens between structural commitments.
- **I-34.03 [E][measurement]** Measures whether the hybrid recovers the constrained-reasoning tax.
- **I-34.04 [E][decoding]** Applies constraints only to final output in reflection/self-correction loops.
- **I-34.05 [E][safety]** Avoids structure snowballing during intermediate reflection.
- **I-34.06 [E][control-plane]** Stores grammars as pinned artifacts.
- **I-34.07 [E][integrity]** Pins grammar bytes by blake3, analogous to MCP schemas.
- **I-34.08 [E][security]** Never loads third-party grammars.
- **I-34.09 [E][security]** Rejects enum-bearing schemas from untrusted sources.
- **I-34.10 [E][validation]** Revalidates every constrained output after parsing.
- **I-34.11 [E][fallback]** Routes output that fails post-parse validation to refusal.
- **I-34.12 [E][invariant]** Never silently accepts a noncompliant decoded output.
- **I-34.13 [E][integration]** Adds the runtime leg in providers decoding and the sidecar mask layer.
- **I-34.14 [E][security]** Freezes the grammar pin manifest in R8.
- **I-34.15 [E][UI]** Shows grammar-pin status in the existing decode panel.
- **I-34.16 [E][measurement]** A reasoning fixture must produce thinking plus valid structure with pass
  rate at least the FreeText arm.
- **I-34.17 [E][measurement]** Records the tight-grammar arm's measured tax.
- **I-34.18 [E][failure]** A mutated grammar file emits GrammarGateEvent PinViolation and is refused.
- **I-34.19 [E][failure]** A silently noncompliant engine-stub output emits PostParseRejected.
- **I-34.20 [E][test]** A trust-mask broken arm must ship the bad output and fail.

#### I-35 — temporal tool-sequence enforcement

Source anchor: I-SUMMARY_INDEX.md:663. Metadata: CANT-WRECK; STRONG; BUILD; M; Wave 2; depends on I-1.

- **I-35.01 [E][policy]** Evaluates rules over ordered tool-call history.
- **I-35.02 [E][policy]** Starts a rule when a configured tool-class trigger fires.
- **I-35.03 [E][predicate]** Can require tests since the most recent source write.
- **I-35.04 [E][predicate]** Can forbid a destructive verb after untrusted ingest until a taint check has
  run.
- **I-35.05 [E][predicate]** Can require an auditor-PASS row before git push.
- **I-35.06 [E][enforcement]** Can halt a call.
- **I-35.07 [E][enforcement]** Can require human confirmation.
- **I-35.08 [E][enforcement]** Can substitute another action.
- **I-35.09 [E][security]** Runs rules on the I-1 Supervisor plane.
- **I-35.10 [E][security]** Is immune to token-level attacks because it evaluates actions and history, not
  generated text.
- **I-35.11 [E][scope]** Adds temporal ordering on top of state masks, per-call argv shape, and per-task
  committed tool sets.
- **I-35.12 [E][configuration]** Stores rules in TOML.
- **I-35.13 [E][security]** Freezes the safety-core rule set with mutable equal to zero.
- **I-35.14 [E][UI]** Shows sequence-verdict rows in the gate panel.
- **I-35.15 [E][failure]** Git push without an in-session auditor PASS is halted.
- **I-35.16 [E][diagnostics]** ToolSequenceVerdict OrderViolation names the unsatisfied predicate.
- **I-35.17 [E][behavior]** A compliant sequence emits Allowed and proceeds.
- **I-35.18 [E][integrity]** Includes the sequence-rule hash in the frozen manifest.
- **I-35.19 [E][security]** Blocks a run that edits its own sequence rules.
- **I-35.20 [E][determinism]** Replay rederives identical sequence verdicts.
- **I-35.21 [E][deferred]** Defers Pro2Guard-style probabilistic reachability until a trace corpus exists.

#### I-36 — cross-model prompt compilation

Source anchor: I-SUMMARY_INDEX.md:682. Metadata: CHEAPER-PORTABLE; STRONG; BUILD; M; Wave 4; ID-3/ID-6.

- **I-36.01 [E][artifact]** Stores compiled prompt variants keyed by role and backend.
- **I-36.02 [E][workflow]** Produces variants offline.
- **I-36.03 [E][optimization]** Uses the cloud-lane optimizer to search instructions and exemplars.
- **I-36.04 [E][compatibility]** Emits plain-string prompts so call sites need not refactor.
- **I-36.05 [E][validation]** Validates variants on ELAI fixtures.
- **I-36.06 [E][measurement]** Measures and ledgers the source-to-target prompt transfer gap.
- **I-36.07 [E][promotion]** Promotes compiled prompts through the ID-6 ledger.
- **I-36.08 [E][replay]** Emits ID-3 receipts that identify the exact served variant.
- **I-36.09 [E][provider-hygiene]** Removes explicit step-by-step scaffolds under extended thinking.
- **I-36.10 [E][provider-hygiene]** Respects backend temperature defaults.
- **I-36.11 [E][integration]** Keys on B-37 PerRoleRuntime and shares the E-8 GEPA optimizer lane.
- **I-36.12 [E][UI]** Shows a per-role compiled-variant badge in SettingsWindow.
- **I-36.13 [E][observability]** PromptCompiled rows contain the variant hash and backend.
- **I-36.14 [E][measurement]** TransferGapMeasured compares source prompt on target with compiled prompt on
  target.
- **I-36.15 [E][rollback]** A variant below its fixture floor automatically rolls back.
- **I-36.16 [E][observability]** Rollback emits Rolledback.
- **I-36.17 [E][test]** A cloud extended-thinking request must contain no chain-of-thought scaffold.

#### I-37 — ears requirement objects

Source anchor: I-SUMMARY_INDEX.md:702. Metadata: VERIFIED-INTENT; STRONG; BUILD; M-L; Wave 3.

- **I-37.01 [E][requirements]** Represents each atomic EARS-pattern requirement as a typed record.
- **I-37.02 [E][data]** Stores a content hash.
- **I-37.03 [E][data]** Binds tests to the requirement.
- **I-37.04 [E][data]** Stores I-20 mutation coverage for those tests.
- **I-37.05 [E][data]** Binds applicable metamorphic relations.
- **I-37.06 [E][state]** Computes a per-requirement VERIFIED or UNVERIFIED state.
- **I-37.07 [E][UI]** Surfaces requirement status per hunk in the reviewer cockpit.
- **I-37.08 [E][interop]** Imports Spec-Kit-style specification files.
- **I-37.09 [E][interop]** Imports Kiro-style specification files.
- **I-37.10 [E][trust]** Treats imported formats as intent sources while ELAI separately proves
  satisfaction.
- **I-37.11 [E][status-example]** Can define satisfaction through test coverage, sampled mutation score,
  and a named metamorphic relation.
- **I-37.12 [E][honesty]** Shows UNVERIFIED when those evidence conditions are absent.
- **I-37.13 [E][dependency]** Generalizes C-29 IntentPair, which becomes the simplest EARS record.
- **I-37.14 [E][workflow]** Authors or imports requirements at the PLANNING gate before TESTING_FIRST.
- **I-37.15 [E][traceability]** Each satisfied chip points to real test and mutation rows.
- **I-37.16 [E][lifecycle]** Deleting a bound test flips the state to Unverified on reaudit.
- **I-37.17 [E][observability]** Regression emits Regressed.
- **I-37.18 [E][round-trip]** Imported Spec-Kit data round-trips into typed records.
- **I-37.19 [E][fail-closed]** An unverifiable requirement remains visible and is never silently dropped.
- **I-37.20 [A][threshold]** The cited 88-percent sampled-mutation rule is presented as an example
  requirement expression, not clearly declared as a universal threshold.

#### I-38 — qdelta + statistical floor

Source anchor: I-SUMMARY_INDEX.md:720. Metadata: PROVABLE; STRONG; BUILD; M; Wave 1; ID-5 instrument.

- **I-38.01 [E][benchmark]** Defines QDELTA as defective diffs reaching a reviewer per 100 tasks.
- **I-38.02 [E][ablation]** Compares gates-on against gates-off.
- **I-38.03 [E][defect]** Counts held-out test failures.
- **I-38.04 [E][defect]** Counts surviving mutants found after merge.
- **I-38.05 [E][defect]** Counts regressions.
- **I-38.06 [E][statistics]** Uses bootstrap confidence intervals clustered by repository.
- **I-38.07 [E][policy]** Mechanically requires paired-task designs.
- **I-38.08 [E][policy]** Requires at least five seeds per condition.
- **I-38.09 [E][reporting]** Reports standard error of the mean.
- **I-38.10 [E][policy]** Requires pass-to-the-k reporting for reliability claims.
- **I-38.11 [E][reporting]** Reports dollars per task beside accuracy.
- **I-38.12 [E][corpus]** Uses date-stamped task cuts.
- **I-38.13 [E][contamination]** Surfaces the D-20 memorization probe.
- **I-38.14 [E][framing]** Reports only a gate ablation, never a leaderboard ranking.
- **I-38.15 [E][integration]** Mirrors the SAFEDELTA harness shape and composes the I-20 mutant leg.
- **I-38.16 [E][artifact]** Auto-generates methodology documentation from machine-readable configuration.
- **I-38.17 [E][UI]** Places a QDELTA tile beside SAFEDELTA.
- **I-38.18 [E][CLI]** elai bench qdelta emits QdeltaMeasured with on/off delta and confidence interval.
- **I-38.19 [E][operations]** Ships the fixture tier now and leaves the full corpus behind a RunPod
  checkbox.
- **I-38.20 [E][fail-closed]** Refuses runs with only three seeds or an unpaired design.
- **I-38.21 [E][determinism]** Regenerates the methodology document deterministically.
- **I-38.22 [E][traceability]** The methodology names the D-20 contamination result.

#### I-39 — plan state from repo

Source anchor: I-SUMMARY_INDEX.md:738. Metadata: ANTI-DRIFT; STRONG; BUILD; S-M; Wave 1.

- **I-39.01 [E][CLI]** Adds elai plan status.
- **I-39.02 [E][derivation]** Derives per-plan state from journal data.
- **I-39.03 [E][derivation]** Derives per-plan state from plan-status fragments.
- **I-39.04 [E][derivation]** Derives per-plan state from wiring_contract_check results.
- **I-39.05 [E][traceability]** Adds a verified-against commit or map-date anchor to every output line.
- **I-39.06 [E][lifecycle]** Demotes hand-maintained tallies to generated build artifacts.
- **I-39.07 [E][migration]** Adds one journal-rot migration test.
- **I-39.08 [E][migration]** Replays an archived old-schema ssb.db through current code.
- **I-39.09 [E][trust]** Treats old RRAs as a trust product that must not silently rot.
- **I-39.10 [E][integration]** Reads SSB viewer paths, s-plan-audit fragments, wiring checks, F-13 archive,
  and E-44 migration machinery.
- **I-39.11 [E][UI]** Adds no separate UI because the CLI and generated document are the load-bearing
  artifacts.
- **I-39.12 [E][CLI]** Letter-filtered status output carries fragment path, checker verdict, and journal
  row for each session.
- **I-39.13 [E][determinism]** Identical inputs regenerate byte-identical status output.
- **I-39.14 [E][test]** The migration test opens a real F-13 archived database and decodes N rows.
- **I-39.15 [E][observability]** Successful replay emits PlanStateDerived.
- **I-39.16 [E][fail-closed]** A corrupt archive fails loudly and never skips.
- **I-39.17 [A][status]** The summary reports that five sources of truth drifted but does not enumerate
  all five in this section.

#### I-40 — trust-layer wrapper + rra spec

Source anchor: I-SUMMARY_INDEX.md:755. Metadata: WEDGE; STRONG strategy grounding; BUILD; L; Wave 5.

- **I-40.01 [E][standard]** Publishes the RRA format as an open specification.
- **I-40.02 [E][standard]** Versions the RRA specification.
- **I-40.03 [E][verification]** Ships an RRA conformance checker.
- **I-40.04 [E][wrapper]** Wraps an external agent through a minimal proxy/MCP surface.
- **I-40.05 [E][wrapper]** Routes external-agent tool traffic through ELAI taint tagging.
- **I-40.06 [E][wrapper]** Routes external-agent writes through ELAI write gates.
- **I-40.07 [E][artifact]** Emits an RRA for the wrapped external run.
- **I-40.08 [E][strategy]** Positions the wrapper and open standard as the adoption wedge.
- **I-40.09 [E][strategy]** Keeps the deep orchestrator as the premium destination.
- **I-40.10 [E][privacy]** Makes G-9 redaction a publication precondition.
- **I-40.11 [E][privacy]** Exported RRAs must not leak secrets.
- **I-40.12 [E][integration]** Composes existing taint, write/dependency/blast-radius gates, RRA,
  attestation, adapter, and MCP-hardening surfaces without forking them.
- **I-40.13 [E][spec-path]** Names docs/spec/rra-spec-v1.md as the in-repo versioned specification.
- **I-40.14 [E][CLI]** Adds elai wrap -- <agent command>.
- **I-40.15 [E][proxy]** Proxies wrapped tool input and output.
- **I-40.16 [E][UI]** Renders wrapped-run rows in the existing RRA viewer.
- **I-40.17 [E][test]** A wrapped external fixture must produce a spec-conformant RRA.
- **I-40.18 [E][observability]** Wrapped untrusted input creates taint rows.
- **I-40.19 [E][security]** A seeded malicious write from the wrapped agent must be blocked.
- **I-40.20 [E][test]** The conformance suite rejects a tampered artifact.
- **I-40.21 [E][test]** The exported bundle must contain no raw planted secret bytes.

### Track 19 — design-phase critique and trust-claim discipline

#### I-41 — spec-time critique lenses (L1-L8)

Source anchor: I-SUMMARY_INDEX.md:791. Metadata: PLURAL-DOUBT; MODERATE; BUILD; M; Wave 2; binds ID-4 and
the ban on load-bearing LLM judges.

- **I-41.01 [E][lens]** Adds the Assumption Excavator lens.
- **I-41.02 [E][lens]** Adds the Implied Completeness lens.
- **I-41.03 [E][lens]** Adds the Ambiguity Mapper lens.
- **I-41.04 [E][lens]** Adds the Gap Analyzer lens.
- **I-41.05 [E][lens]** Adds the Pre-Implementation Architect lens.
- **I-41.06 [E][lens]** Adds the Public-Interface Validator lens.
- **I-41.07 [E][lens]** Adds the Documentation Validator lens.
- **I-41.08 [E][lens]** Adds the Anxiety Reader lens.
- **I-41.09 [E][timing]** Runs lenses on prose or specifications before deterministic artifacts exist.
- **I-41.10 [E][trust]** Treats every LLM-judged finding as advisory only.
- **I-41.11 [E][grounding]** Requires a concrete anchor for every finding.
- **I-41.12 [E][anchor]** Accepts a specification heading as an anchor.
- **I-41.13 [E][anchor]** Accepts a file and line as an anchor.
- **I-41.14 [E][anchor]** Accepts a symbol as an anchor.
- **I-41.15 [E][anchor]** Accepts a quoted phrase as an anchor.
- **I-41.16 [E][fail-safe]** Drops anchorless findings.
- **I-41.17 [E][invariant]** No lens output can accept or block.
- **I-41.18 [E][budget]** Caps each lens at eight findings.
- **I-41.19 [E][quality]** Deduplicates findings.
- **I-41.20 [E][routing]** Scales the number of lenses to artifact risk.
- **I-41.21 [E][implementation]** Reauthors lenses natively rather than using the E-43 MCP-scan path.
- **I-41.22 [E][integration]** Hooks B-40 idea framing and Phase 2B grounding sweep.
- **I-41.23 [E][CLI]** Adds elai critique <spec>.
- **I-41.24 [E][UI]** Shows nonblocking advisory findings in ReasoningDrawer or the framing panel.
- **I-41.25 [E][human-loop]** Routes BLOCKER-CANDIDATE to a human without automatic blocking.
- **I-41.26 [E][observability]** Emits CritiqueLensFinding(AdvisoryFinding) for planted anchored issues.
- **I-41.27 [E][observability]** Emits DroppedNoAnchor for an anchorless result.
- **I-41.28 [E][test]** A contract proves lens runs emit zero gate-verdict rows.

#### I-42 — critique-only operating profile

Source anchor: I-SUMMARY_INDEX.md:810. Metadata: HUMAN-AS-VERIFIER; MODERATE; BUILD; M; Wave 3; depends on
I-41.

- **I-42.01 [E][profile]** Defines RunProfile Orchestrate.
- **I-42.02 [E][profile]** Defines RunProfile CritiqueOnly.
- **I-42.03 [E][workflow]** In CritiqueOnly the human operator writes the code.
- **I-42.04 [E][workflow]** Spawned agents perform design critique through I-41.
- **I-42.05 [E][workflow]** Spawned agents may run the shipped auditor/off-agent spine at ship time.
- **I-42.06 [E][security]** Structurally disables executor spawn in CritiqueOnly.
- **I-42.07 [E][security]** Structurally disables fixer spawn in CritiqueOnly.
- **I-42.08 [E][constraint]** Does not rely on a prompt instruction to suppress writers.
- **I-42.09 [E][default]** Keeps Orchestrate as the default profile.
- **I-42.10 [E][invariant]** CritiqueOnly must have zero executor- or fixer-attributed file_write rows.
- **I-42.11 [E][architecture]** Uses one engine and one verification spine for both profiles.
- **I-42.12 [E][scope]** Profiles differ only in who writes and whether executors spawn.
- **I-42.13 [E][configuration]** Keeps RunProfile orthogonal to B-22 GateMode.
- **I-42.14 [E][integration]** Uses I-1 to reject executor-side-effect Directives.
- **I-42.15 [E][CLI]** Adds elai run --profile critique-only.
- **I-42.16 [E][UI]** Shows the active profile in the existing StatusBar mode cluster.
- **I-42.17 [E][test]** The same fixture under Orchestrate must produce executor writes, proving the profile
  is not a no-op.
- **I-42.18 [E][determinism]** Off-agent verdicts are byte-identical across the two profiles.

#### I-43 — claim scoper + Phase-0 kill-gate

Source anchor: I-SUMMARY_INDEX.md:828. Metadata: FALSIFY-FIRST/PROVABLE; STRONG; BUILD; M; Wave 1; binds
ID-5.

- **I-43.01 [E][pilot]** Runs the shipped SecCodePLT/SAFEDELTA harness at reduced N.
- **I-43.02 [E][pilot]** Uses 50 to 100 tasks for the reduced pilot.
- **I-43.03 [E][ablation]** Compares gates on against gates off.
- **I-43.04 [E][scoring]** Scores dynamic exploit outcomes.
- **I-43.05 [E][measurement]** Reports residual-insecure-rate delta sign.
- **I-43.06 [E][measurement]** Reports residual-insecure-rate delta magnitude.
- **I-43.07 [E][measurement]** Reports the rate that reaches a reviewer.
- **I-43.08 [E][anti-gaming]** Detects the block-everything trap.
- **I-43.09 [E][failure]** A gate that wins only by blocking nearly everything fails the pilot.
- **I-43.10 [E][scoper]** Runs about 20 identical tasks on local Qwen and cloud Claude.
- **I-43.11 [E][control]** Holds the seed fixed across lanes.
- **I-43.12 [E][measurement]** Measures action-stream divergence on security paths.
- **I-43.13 [E][claim]** Uses divergence to choose honest reproducibility wording.
- **I-43.14 [E][claim]** Scopes cloud claims to trace authentication rather than deterministic replay.
- **I-43.15 [E][scope]** Distinguishes cross-lane divergence from H-23 same-environment determinism.
- **I-43.16 [E][threshold-policy]** Uses no paper constant as the threshold.
- **I-43.17 [E][artifact]** Produces a go/no-go memo and never directly emits an external marketing claim.
- **I-43.18 [E][CLI]** Adds elai bench claim-scope.
- **I-43.19 [E][UI]** Optionally shows a Phase-0 go/no-go cockpit panel; the memo may justify no UI.
- **I-43.20 [E][observability]** Emits ClaimScopeMeasured with pilot delta, reviewer rate, divergence, and
  scoped wording.
- **I-43.21 [E][artifact]** Names workflow/reasoning/I-43-phase0-memo.md as the memo output.
- **I-43.22 [E][operations]** Ships fixtures now and gates the live leg separately.
- **I-43.23 [E][wording]** Divergence above five percent in the stated observable forces
  trace-authentication-only wording.
- **I-43.24 [E][determinism]** All claim-scope numbers must recompute from the journal.
- **I-43.25 [A][scope]** The summary calls this a kill-gate and its pilot can fail, while Track 19 says none
  of its sessions sets the product DONE accept/block verdict. The safe reading is that it gates claim/build
  commitment, not the off-agent DONE decision.

#### I-44 — zero-retention-after-verification export mode

Source anchor: I-SUMMARY_INDEX.md:849. Metadata: AUDITABLE/TABLE-STAKES; STRONG; BUILD; S; Wave 3; depends
on G-9.

- **I-44.01 [E][policy]** Defines RetentionPolicy Full.
- **I-44.02 [E][policy]** Defines RetentionPolicy ZeroAfterVerify.
- **I-44.03 [E][timing]** Purges raw inputs only after the verdict and evidence bundle are sealed.
- **I-44.04 [E][purge]** Removes raw prompts.
- **I-44.05 [E][purge]** Removes raw tool input and output.
- **I-44.06 [E][purge]** Removes raw model input and output.
- **I-44.07 [E][granularity]** Purges ordinary raw inputs at file granularity.
- **I-44.08 [E][database]** Purges the whole session database because SSB WAL is append-only.
- **I-44.09 [E][retention]** Keeps only redacted, hash-checkable evidence.
- **I-44.10 [E][privacy]** Requires G-9 to mask retained summaries before purge.
- **I-44.11 [E][lifecycle]** Reuses F-13 archive-then-purge machinery.
- **I-44.12 [E][invariant]** Export grep for raw-input markers must be empty after purge.
- **I-44.13 [E][invariant]** elai replay --verify must still pass for hash-checkable steps.
- **I-44.14 [E][default]** Makes zero-retention opt-in and leaves Full behavior unchanged.
- **I-44.15 [E][safety]** Treats purge as destructive and irreversible.
- **I-44.16 [E][CLI]** Adds elai run --zero-retention.
- **I-44.17 [E][UI]** Shows RetentionPurged rows in the RRA viewer or retention drawer.
- **I-44.18 [E][observability]** Emits RawInputsPurged.
- **I-44.19 [E][observability]** Emits EvidenceRetained with the retained-data blake3.
- **I-44.20 [E][observability]** Emits PurgeVerified.
- **I-44.21 [E][test]** Full and ZeroAfterVerify fixtures differ only in raw-byte presence.
- **I-44.22 [E][determinism]** Evidence and verdict bytes remain identical between retention modes.
- **I-44.23 [E][scope]** Treats self-hosting as covered by I-40 and SOC 2 as out-of-scope process work.

### Track 20 — competitor-steal ports

#### I-45 — objective-hash sealed receipts

Source anchor: I-SUMMARY_INDEX.md:888. Metadata: CANT-LIE; STRONG SL-5/MODERATE roach SL-11; EXTENSION;
S-M; Wave 1.

- **I-45.01 [E][integrity]** Computes blake3 over canonical objective bytes.
- **I-45.02 [E][receipt]** Stamps the objective hash into every sealed PASS.
- **I-45.03 [E][artifact]** Adds the objective hash to RRA sealing.
- **I-45.04 [E][artifact]** Adds the objective hash to attestation.
- **I-45.05 [E][staleness]** Rechecks objective hash on every receipt read path.
- **I-45.06 [E][failure]** Changing goal text after PASS makes the receipt stale.
- **I-45.07 [E][failure]** Changing acceptance text after PASS makes the receipt stale.
- **I-45.08 [E][compatibility]** Verifies older receipts as explicit LEGACY.
- **I-45.09 [E][invariant]** Never labels a legacy receipt silently fresh.
- **I-45.10 [E][scope]** Complements F-4 artifact hashing by hashing the goal itself.
- **I-45.11 [E][integration]** Extends shipped rra.rs and attestation.rs in place.
- **I-45.12 [E][UI]** Shows a stale-receipt badge in the RRA viewer.
- **I-45.13 [E][observability]** Objective mutation emits ObjectiveSealed(MutationDetected).
- **I-45.14 [E][failure]** Verification hard-fails as ReceiptStale and names both hashes.
- **I-45.15 [E][test]** An unmodified objective verifies PASS.
- **I-45.16 [E][test]** A verifier that passes after objective mutation is the broken arm.

#### I-46 — bash interceptor + command fixup

Source anchor: I-SUMMARY_INDEX.md:903. Metadata: CHEAPER+CANT-WRECK; STRONG SL-2/MEDIUM gajae SL-10;
EXTENSION; S-M; Wave 2.

- **I-46.01 [E][interceptor]** Runs before existing bash validation.
- **I-46.02 [E][redirect]** Refuses and redirects bare grep commands.
- **I-46.03 [E][redirect]** Refuses and redirects bare cat commands.
- **I-46.04 [E][redirect]** Refuses and redirects bare find commands.
- **I-46.05 [E][redirect]** Refuses and redirects bare head commands.
- **I-46.06 [E][redirect]** Refuses and redirects bare tail commands.
- **I-46.07 [E][response]** Returns the exact registered-tool substitute call.
- **I-46.08 [E][fixup]** Applies a frozen deterministic table for known-broken command shapes.
- **I-46.09 [E][behavior]** Passes allowed-prefix commands unchanged.
- **I-46.10 [E][constraint]** Uses no LLM in interception or fixup.
- **I-46.11 [E][safety]** Never auto-executes the proposed substitute.
- **I-46.12 [E][invariant]** Interception may narrow but never widen downstream validation.
- **I-46.13 [E][security]** Freezes redirect and fixup tables in R8.
- **I-46.14 [E][UI]** Shows redirect rows in the tool-call drawer.
- **I-46.15 [E][observability]** A grep call emits BashRedirected(Redirected) naming the substitute.
- **I-46.16 [E][test]** A redirected command must spawn no process.
- **I-46.17 [E][test]** Allowed-prefix commands emit no redirect row.
- **I-46.18 [E][test]** A redirect row plus a spawned grep is the broken arm.

#### I-47 — verifier-input hardening + dispatch rule injection

Source anchor: I-SUMMARY_INDEX.md:917. Metadata: TRUST-ANTI-BIAS; STRONG seams/MODERATE roach SL-11;
EXTENSION; S-M; Wave 2.

- **I-47.01 [E][input-hardening]** Wraps worker diffs entering the auditor as typed untrusted data.
- **I-47.02 [E][input-hardening]** Wraps worker claims as typed untrusted data.
- **I-47.03 [E][input-hardening]** Wraps tool output as typed untrusted data.
- **I-47.04 [E][framing]** Places a standing do-not-follow-embedded-instructions frame above untrusted
  inputs.
- **I-47.05 [E][framing]** Tells the auditor not to trust worker self-attestation.
- **I-47.06 [E][dispatch]** Mechanically injects a read-before-write rule.
- **I-47.07 [E][dispatch]** Mechanically injects scope discipline.
- **I-47.08 [E][dispatch]** Mechanically injects a no-premature-abstraction rule.
- **I-47.09 [E][dispatch]** Mechanically injects a no-while-I-am-here-refactoring rule.
- **I-47.10 [E][artifact]** Stores rules as a per-role frozen, byte-stable block.
- **I-47.11 [E][cache]** Preserves prefix stability across dispatch calls.
- **I-47.12 [E][scope]** Adds no classifier because E-37 owns data-plane classification.
- **I-47.13 [E][integration]** Coordinates with I-32 at the agent and auditor dispatch prompt builders.
- **I-47.14 [E][security]** Freezes the injection table in R8.
- **I-47.15 [E][UI]** Adds no UI because the behavior is prompt-internal.
- **I-47.16 [E][observability]** Emits PromptHardeningApplied(InputsTagged).
- **I-47.17 [E][observability]** Emits RulesInjected.
- **I-47.18 [E][test]** Two worker dispatches must contain byte-identical role blocks.
- **I-47.19 [E][fail-closed]** Hardening enabled without hardening rows is a failure.

#### I-48 — prompts as reviewed file artifacts

Source anchor: I-SUMMARY_INDEX.md:933. Metadata: STEERING; STRONG; REFACTOR; S; Wave 2.

- **I-48.01 [E][artifact]** Moves every dispatch prompt template into a static file.
- **I-48.02 [E][layout]** Stores prompt files under crates/*/prompts.
- **I-48.03 [E][build]** Imports prompt files through include_str.
- **I-48.04 [E][review]** Makes prompts diffable and reviewable.
- **I-48.05 [E][constraint]** Prohibits inline dispatch prompt templates.
- **I-48.06 [E][migration]** Preserves prompt bytes during extraction.
- **I-48.07 [E][test]** Compares assembled executor prompt hashes before and after extraction.
- **I-48.08 [E][test]** Compares assembled auditor prompt hashes before and after extraction.
- **I-48.09 [E][contract]** Bans new long inline prompt literals in dispatch code.
- **I-48.10 [E][dependency]** Makes I-36 compiled prompts and I-47 frozen blocks pinnable.
- **I-48.11 [E][integration]** Leaves D-1 template substitution unchanged.
- **I-48.12 [E][UI]** Adds no UI because it is a build-time refactor.
- **I-48.13 [E][observability]** Adds no new action type.

#### I-49 — bedrock + azure direct lanes + cross-protocol shim

Source anchor: I-SUMMARY_INDEX.md:946. Metadata: REACH; STRONG gap SL-8/MEDIUM gajae SL-10; BUILD
reference port; L; Wave 4; live legs network-gated; full-form expansion required before execution.

- **I-49.01 [E][provider]** Adds an AWS Bedrock direct runtime lane.
- **I-49.02 [E][protocol]** Implements SigV4 signing.
- **I-49.03 [E][protocol]** Decodes binary EventStream frames.
- **I-49.04 [E][provider]** Adds an Azure OpenAI Responses-shaped direct runtime lane.
- **I-49.05 [E][shim]** Adds an OpenAI-as-Anthropic cross-protocol normalization pattern.
- **I-49.06 [E][integration-choice]** Extends genai_adapter if its existing seam fits.
- **I-49.07 [E][provenance]** Requires reading the TypeScript competitor source before porting.
- **I-49.08 [E][freshness]** Requires current AWS protocol documentation research before work.
- **I-49.09 [E][freshness]** Requires current Azure protocol documentation research before work.
- **I-49.10 [E][dependency]** Requires anti-slopsquat review for candidate aws-prefixed crates.
- **I-49.11 [E][dependency]** Requires user approval for every new dependency.
- **I-49.12 [E][test]** Ships deterministic recorded-frame replay fixtures immediately.
- **I-49.13 [E][test]** Requires byte-exact frame decoding.
- **I-49.14 [E][operations]** Keeps real Bedrock and Azure calls behind separate live checkboxes.
- **I-49.15 [E][constraint]** Leaves oauth and existing CLI bridges on the keep list.
- **I-49.16 [E][integration]** Registers bedrock.rs, azure_openai.rs, and a shim under the runtime adapter
  surface.
- **I-49.17 [E][UI]** Lists both backends in the existing RuntimePanel.
- **I-49.18 [E][CLI]** elai runtime list reports both lanes.
- **I-49.19 [E][failure]** A corrupt recorded frame fails with a typed error.
- **I-49.20 [E][test]** SigV4 fixtures reproduce a documented known-good signature.
- **I-49.21 [E][observability]** Reuses runtime/route rows with backend IDs and adds no action type.
- **I-49.22 [A][integration]** Whether the shim extends genai_adapter is explicitly conditional.
- **I-49.23 [A][precondition]** The summary requires a full-form expansion before execution; this summary
  entry is not sufficient as an implementation spec.

#### I-50 — subscription auth broker

Source anchor: I-SUMMARY_INDEX.md:963. Metadata: REACH; STRONG SL-3/MEDIUM gajae SL-10; EXTENSION; M;
Wave 4.

- **I-50.01 [E][API]** Defines an AuthProvider trait.
- **I-50.02 [E][API]** AuthProvider supports acquire.
- **I-50.03 [E][API]** AuthProvider supports refresh.
- **I-50.04 [E][API]** AuthProvider supports revoke.
- **I-50.05 [E][API]** AuthProvider supports status.
- **I-50.06 [E][lifecycle]** A broker owns token lifecycle.
- **I-50.07 [E][storage]** A broker owns token storage.
- **I-50.08 [E][migration]** Moves the existing Anthropic/Claude subscription flow behind the trait.
- **I-50.09 [E][compatibility]** Preserves the existing token-file format byte-for-byte.
- **I-50.10 [E][compatibility]** Preserves refresh semantics.
- **I-50.11 [E][scope]** Adds no new provider login flows in this session.
- **I-50.12 [E][scope]** Adds no new storage backend.
- **I-50.13 [E][privacy]** Never journals token material.
- **I-50.14 [E][integration]** Adds auth_broker.rs and updates adapter call sites.
- **I-50.15 [E][UI]** Shows auth status in the existing RuntimePanel or Settings.
- **I-50.16 [E][observability]** Real requests emit AuthBrokerEvent(TokenAcquired or TokenRefreshed).
- **I-50.17 [E][recovery]** An expired-token fixture refreshes then succeeds.
- **I-50.18 [E][failure]** Refresh failure emits RefreshFailed plus a typed error.
- **I-50.19 [E][test]** Journal search for token material must be empty.
- **I-50.20 [E][fail-closed]** Authentication without a broker row is a failure.

#### I-51 — unified model capability catalog

Source anchor: I-SUMMARY_INDEX.md:977. Metadata: SMARTER-ORCH; STRONG SL-4; EXTENSION consolidation; S-M;
Wave 1.

- **I-51.01 [E][catalog]** Defines one models.toml catalog keyed by model ID.
- **I-51.02 [E][field]** Stores context window.
- **I-51.03 [E][field]** Stores input pricing.
- **I-51.04 [E][field]** Stores output pricing.
- **I-51.05 [E][field]** Stores cache pricing.
- **I-51.06 [E][field]** Stores thinking support.
- **I-51.07 [E][field]** Stores effort mapping.
- **I-51.08 [E][field]** Stores default max tokens.
- **I-51.09 [E][field]** Stores VRAM tier.
- **I-51.10 [E][freshness]** Stores a staleness date for every entry.
- **I-51.11 [E][migration]** Converts pricing.rs into a view over the catalog.
- **I-51.12 [E][migration]** Converts duplicated vram_tier.rs modules into catalog views.
- **I-51.13 [E][compatibility]** Preserves existing public APIs.
- **I-51.14 [E][constraint]** Wraps and preserves the measured VRAM-tier plan.
- **I-51.15 [E][constraint]** Performs no live price fetching.
- **I-51.16 [E][invariant]** Is behavior-neutral by construction.
- **I-51.17 [E][integration]** Supplies clean input to cost gates, routing, economics, and runtime defaults.
- **I-51.18 [E][UI]** Adds no UI behind preserved APIs.
- **I-51.19 [E][test]** Cost and tier outputs remain byte-identical before and after migration.
- **I-51.20 [E][test]** A seeded catalog/view divergence fails the drift contract.
- **I-51.21 [E][test]** Migrated modules contain zero price constants outside models.toml.
- **I-51.22 [E][observability]** Adds no action type.
- **I-51.23 [A][deferred]** B-37 default consumption is explicitly a future/deferred leg.

#### I-52 — checkpoint/rewind agent tool

Source anchor: I-SUMMARY_INDEX.md:992. Metadata: CONCURRENCY-SAFETY; STRONG SL-6; ACTIVATION; S-M; Wave 3.

- **I-52.01 [E][activation]** Exposes already-shipped checkpoint machinery to agents.
- **I-52.02 [E][tool]** Adds a save arm.
- **I-52.03 [E][tool]** Adds a rewind arm.
- **I-52.04 [E][tool]** Adds a diff arm.
- **I-52.05 [E][scope]** Restricts checkpoints to the current session.
- **I-52.06 [E][security]** Refuses cross-session rewind.
- **I-52.07 [E][safety]** Refuses rewind when files outside the checkpointed set diverged.
- **I-52.08 [E][diagnostics]** Names each diverged path in the refusal.
- **I-52.09 [E][coordination]** If H-49 ships first, exposes the tool over H-49's store.
- **I-52.10 [E][invariant]** Never creates a second snapshot store.
- **I-52.11 [E][scope]** Supports in-branch, single-agent speculation.
- **I-52.12 [E][scope]** Leaves I-2 as the parallel-branch mechanism.
- **I-52.13 [E][integration]** Uses the three-arm tool catalog and existing checkpoint labels/scopes.
- **I-52.14 [E][feature-flag]** Inherits W-2 posture if the checkpoint substrate remains off.
- **I-52.15 [E][UI]** Shows checkpoint rows in the existing session drawer.
- **I-52.16 [E][observability]** Emits CheckpointToolEvent Saved and Rewound.
- **I-52.17 [E][test]** Rewound bytes must blake3-match checkpoint preimages.
- **I-52.18 [E][test]** Silent overwrite of a diverged file is the broken arm.

#### I-53 — recipe runners + utility tool pack

Source anchor: I-SUMMARY_INDEX.md:1008. Metadata: TOOLS; STRONG gap SL-9/MEDIUM gajae SL-10; BUILD; S-M;
Wave 2.

- **I-53.01 [E][recipe]** Discovers Justfile targets.
- **I-53.02 [E][recipe]** Discovers Makefile targets.
- **I-53.03 [E][recipe]** Discovers Cargo aliases.
- **I-53.04 [E][recipe]** Runs a selected named recipe.
- **I-53.05 [E][security]** Routes recipe execution through bash_validation.
- **I-53.06 [E][security]** Routes recipe execution through E-30 argv gates.
- **I-53.07 [E][observability]** Journals owner-sanctioned recipe invocations.
- **I-53.08 [E][archive]** Adds a read-only archive reader.
- **I-53.09 [E][archive]** Performs no extraction side effects.
- **I-53.10 [E][JSON]** Adds a capped JSON-tree explorer.
- **I-53.11 [E][JSON]** Marks truncation explicitly.
- **I-53.12 [E][decline]** Declines cron tools.
- **I-53.13 [E][decline]** Declines monitor tools.
- **I-53.14 [E][decline]** Declines background-job tools because orchestrator schedules own the concern.
- **I-53.15 [E][dependency]** Checks existing archive dependencies before proposing a new one.
- **I-53.16 [E][dependency]** Requires anti-slopsquat and approval for a new archive dependency.
- **I-53.17 [E][UI]** Uses the existing tool-call drawer.
- **I-53.18 [E][test]** Recipe listing names fixture Justfile targets.
- **I-53.19 [E][fail-closed]** A recipe run without a bash-validation verdict row fails.
- **I-53.20 [E][test]** Archive-read fixture directory diff remains empty.
- **I-53.21 [E][observability]** Adds no action type.

#### I-54 — structural AST edit tool

Source anchor: I-SUMMARY_INDEX.md:1024. Metadata: TOOLS-VERIFIED; STRONG SL-7/MEDIUM gajae SL-10; BUILD;
M; Wave 3.

- **I-54.01 [E][tool]** Adds structural search.
- **I-54.02 [E][tool]** Adds structural-edit preview.
- **I-54.03 [E][tool]** Adds structural-edit apply.
- **I-54.04 [E][parser]** Uses in-tree tree-sitter 0.26.
- **I-54.05 [E][parser]** Reuses the existing nine grammars.
- **I-54.06 [E][language]** Starts with Rust support.
- **I-54.07 [E][language]** Starts with Python support.
- **I-54.08 [E][language]** Starts with TypeScript support.
- **I-54.09 [E][pattern]** Uses tree-sitter query syntax.
- **I-54.10 [E][constraint]** Introduces no new pattern DSL.
- **I-54.11 [E][workflow]** Requires preview before apply.
- **I-54.12 [E][receipt]** Journals the preview match set.
- **I-54.13 [E][receipt]** Journals the preview diff hash.
- **I-54.14 [E][atomicity]** Applies changes atomically per file.
- **I-54.15 [E][staleness]** Refuses apply when source changed after preview.
- **I-54.16 [E][security]** Routes apply through the same PreToolUse edit gates as ordinary Edit.
- **I-54.17 [E][scope]** Supports syntax-only edits.
- **I-54.18 [E][scope]** Leaves semantic rename to the E-52 LSP lane.
- **I-54.19 [E][dependency]** Blocks any ast-grep dependency.
- **I-54.20 [E][architecture]** Exposes a knowledge-crate public parser API rather than pinning a second
  parser version.
- **I-54.21 [E][UI]** Shows preview and apply rows in the existing tool-call drawer.
- **I-54.22 [E][test]** Post-apply bytes must equal the previewed diff.
- **I-54.23 [E][test]** The edited fixture must compile.
- **I-54.24 [E][observability]** Emits StructuralEditApplied Matched, Previewed, and Applied.
- **I-54.25 [E][failure]** A broken rewrite template fails at preview with ParseFailed and writes zero
  bytes.
- **I-54.26 [E][fail-closed]** Apply with no preceding preview row fails.

## How the I capabilities work together

1. **Enforcement and safe parallelism:** I-1 moves side effects and credentials outside the generator,
   I-2 isolates candidate workspaces, and I-3 makes the high-rate journal/checkpoint path durable.
2. **Search proposal and context:** I-4 fans out candidates; I-5 prevents sibling-failure contamination;
   I-6 pins current intent at the prompt tail; I-7 escalates only condensed local failures; I-8 exits known
   dead ends; I-9 measures whether errors contract.
3. **Economics middleware:** I-10 keeps cloud prefixes cacheable; I-11 is the typed handoff unit; I-12
   learns only receipt-backed playbook content; I-13 guards and dual-serializes memory/tool results; I-14
   compresses reversibly; I-15 reuses safe intermediate grounding; I-16 experimentally reduces local
   KV-memory cost.
4. **Receipt-producing evaluation:** I-17 hard-vetoes and ranks; I-19 gates untested writes; I-20 supplies
   bounded mutation evidence; I-21 separates contract author and evaluator; I-22 converts deterministic
   progress into search value; I-23 supplies one uncertainty calibration; I-24 audits scorer invariance;
   I-18 seals the resulting record.
5. **Measured orchestration:** I-25 learns routing from verified outcomes; I-26 allocates node budgets;
   I-27 exposes the cost/verified-outcome curve; I-28 attributes winning and aborting DAG evidence; I-29
   adds an untrusted fast-draft lane without weakening gates.
6. **Learning flywheel:** sealed I-4/I-28 trajectories become I-30 SFT exhaust; only after all named safety
   prerequisites does I-31 serve execution-grounded rewards and consider weight promotion.
7. **Mechanical control and intent:** I-32 scopes roles structurally; I-33 governs decision calls; I-34
   pins and revalidates grammars; I-35 enforces action order; I-36 compiles prompts per backend; I-37 binds
   requirements to tests and mutation evidence.
8. **Product proof and distribution:** I-38 measures quality delta, I-39 derives plan status, and I-40
   exposes the trust layer around external agents.
9. **Front-of-pipe trust discipline:** I-41 adds anchored advisory design critique; I-42 lets humans write
   while retaining one audit spine; I-43 scopes claims before expensive trust builds; I-44 exports sealed
   evidence without raw-data retention.
10. **Competitor-derived finishing layer:** I-45 binds receipts to objectives; I-46 redirects unsafe or
    wasteful shell shapes; I-47/I-48 harden and externalize prompts; I-49/I-50/I-51 normalize provider
    protocol, auth, and model metadata; I-52 activates safe rewind; I-53 adds bounded utility tools; I-54
    adds preview-bound structural edits.

## Consolidated constraints and invariants

- Hard veto is supreme; neural scoring never resurrects a deterministically failed branch.
- The generator never owns the search tree, gate code, credentials, reward code, or side-effect egress.
- Branch merges occur only at deterministic barriers and journal sequence defines merge order.
- Transformations, handoffs, selections, promotions, and destructive retention actions require receipts.
- Same-seed claims are byte-recomputable where GD-2 is named; cloud cross-lane claims are scoped to trace
  authentication when deterministic token replay is not earned.
- Cache stability forbids dynamic data in frozen prefixes and forbids prompt-text tool restrictions.
- Durable caches are confined to idempotent grounding content; code generation cannot use them.
- Learned rules, routers, prompts, and weights must pass the same evidence-ledger promotion discipline.
- A model cannot author the canary, rule, evaluator, threshold, or harness used to judge itself.
- Untrusted draft/provider/external-agent paths receive the same gates as native generation.
- Missing evidence is failure wherever an R1 condition is declared.
- Destructive zero-retention remains opt-in; verification evidence must survive the purge.
- Track 19 critique never becomes a DONE gate.
- Track 20 ports must read original competitor code, not summaries alone.

## Dependency, rollout, and deferred-work ledger

| Session | Explicit dependency, status, or deferred condition |
|---|---|
| I-1 | Build; reuses shipped Phase 4A IPC and becomes the enforcement prerequisite for I-35/I-31. |
| I-2 | Build; platform-specific CoW/worktree implementations; cloud adapter optional. |
| I-3 | Extension; existing pragmas and atomic sites must be rechecked; io_uring opt-in only. |
| I-4 | Depends on I-2 and I-17; v1 before v2/v3; blocks I-22, I-28, I-30. |
| I-7 | Depends on I-11; interactive Batch API use is rejected. |
| I-10 | Build-on-shipped; scope must shrink if H-1/H-2 already ship. |
| I-13 | YARA optional and user-approved; default-on only after false-positive measurement. |
| I-16 | Evaluation, not adoption; RunPod-gated; MoE retarget is a separate checkbox. |
| I-17 | Blocking for I-4; cross-language packs are Tier-2. |
| I-19 | Default-on only after at least three real-run false-positive measurements. |
| I-28 | Depends on I-4; Shapley/SHARP declined until RL infrastructure exists. |
| I-29 | Moderate grounding; user-approved dependency/network lane; default-on only after corruption bound. |
| I-30 | Depends on I-4; training is a separate RunPod checkbox. |
| I-31 | Explicitly LAST; all H-43/H-44/H-28/I-24/I-30 prerequisites are blocking. |
| I-35 | Depends on I-1; probabilistic reachability deferred pending a trace corpus. |
| I-38 | Fixture tier now; full corpus RunPod-gated. |
| I-41 | Advisory-only moderate-grounding design critique. |
| I-42 | Depends on I-41; Orchestrate remains default. |
| I-43 | Fixture now; live leg gated; generated memo not currently present. |
| I-44 | Depends on G-9; destructive and irreversible; opt-in only. |
| I-49 | Full-form expansion and original TypeScript read required; live calls network-gated. |
| I-50 | No new auth providers or storage backend in this session. |
| I-51 | Behavior-neutral consolidation; B-37 defaults are deferred. |
| I-52 | Activation over shipped snapshots; rescope to H-49 store if H-49 lands first. |
| I-54 | ast-grep dependency is blocked; tree-sitter reuse is mandatory. |

## Deeper-source manifest

### Primary and I-plan control documents

- Read primary: docs/thefinalglance/I-plan/I-SUMMARY_INDEX.md
- Orientation only: docs/thefinalglance/ELAI-SPEC-OVERVIEW.md
- Required doctrine/index deep read:
  docs/thefinalglance/I-plan/I-0_INDEX.md
- Required grounding deep read:
  docs/thefinalglance/I-plan/_grounding/I-GROUNDING-MAP.md
- Required competitor-grounding deep read:
  docs/thefinalglance/I-plan/_grounding/I-STEAL-GROUNDING-MAP.md

### All 54 numbered original I plan files

All are present under docs/thefinalglance/I-plan/:

- I-1_directive_supervisor_enforcement_plane.md
- I-2_cow_workspace_isolation.md
- I-3_ssb_throughput_atomic_saves.md
- I-4_evidence_ranked_search_ladder.md
- I-5_pristine_context_injector.md
- I-6_anchor_pane_slot.md
- I-7_cascading_escalation_offline_batch.md
- I-8_aletheia_voluntary_abort.md
- I-9_error_amplification_telemetry.md
- I-10_prefix_invariance_ttl_tiering.md
- I-11_pabu_belief_state_handoff.md
- I-12_evidence_gated_playbook_curator.md
- I-13_memory_guard_dual_serialization.md
- I-14_deterministic_context_compression.md
- I-15_semantic_cache_blake3_fastpath.md
- I-16_kv_cache_pruning_quant.md
- I-17_tri_layer_evaluation_stack.md
- I-18_upip_chain_sigstore_aibom.md
- I-19_tdd_enforcer_preflight.md
- I-20_budgeted_mutation_scheduler.md
- I-21_sprint_contracts_ralph_loop.md
- I-22_progress_shaped_rewards.md
- I-23_unified_conformal_calibration.md
- I-24_isomorphic_canary_antigaming.md
- I-25_ledger_trained_router.md
- I-26_dynamic_thinking_budgets.md
- I-27_budgeted_convergence_instrumentation.md
- I-28_dag_credit_assignment.md
- I-29_diffusion_draft_lane.md
- I-30_verifier_ranked_sft_exhaust.md
- I-31_grpo_flywheel.md
- I-32_structural_role_scoping.md
- I-33_mechanical_decision_governance.md
- I-34_crane_hybrid_grammar_pinning.md
- I-35_temporal_tool_sequence_enforcement.md
- I-36_cross_model_prompt_compilation.md
- I-37_ears_requirement_objects.md
- I-38_qdelta_statistical_floor.md
- I-39_plan_state_from_repo.md
- I-40_trust_layer_wrapper_rra_spec.md
- I-41_spec_time_critique_lenses.md
- I-42_critique_only_profile.md
- I-43_claim_scoper_kill_gate.md
- I-44_zero_retention_export.md
- I-45_objective_hash_sealed_receipts.md
- I-46_bash_interceptor_redirect.md
- I-47_verifier_input_hardening.md
- I-48_prompts_as_artifacts.md
- I-49_bedrock_azure_direct_lanes.md
- I-50_subscription_auth_broker.md
- I-51_model_capability_catalog.md
- I-52_checkpoint_rewind_tool.md
- I-53_recipe_runner_tool_pack.md
- I-54_structural_ast_edit_tool.md

### Track 12-18 research corpus named by the summary

The summary names the whole docs/prefableresearch/ corpus. Present files requiring
source-level mapping are:

- ELAIplan_ADECO-Orchestrator-Blueprint.txt
- ELAIplan_Consolidated-I-Plan_Integrity-Inference-Search.txt
- QAfromfable.txt
- Research_AI-Assisted-Programming-Trust-Bias-TechDebt.txt
- Research_NaturalLanguage-vs-Deterministic-Constraints-Robustness.txt
- Research_Persona-Prompting-Empirical-Evaluation.txt
- Research_Prompt-Optimization-vs-Deliberative-Tokens-ROI.txt
- Research_PromptHeavy-vs-Infrastructure-MultiAgent-Systems.md
- Research_RateLimits-PromptCaching-TokenCompression.txt
- Research_Replacing-Personas-with-Constrained-Decoding.md
- Review_ADECO-Adversarial-Critique.txt
- Review_ELAI-Competitive_Verification-Core-Improvements.txt
- Review_ELAI-Trust-Harness_Competitive-Review-and-Plan.txt

### Track 19 source documents

Explicit:

- workflow/research/grok-elai-trust/ELAI-CRITIQUE-LENSES.md
- workflow/research/grok-elai-trust/ELAI-IMPROVEMENT-PLAN.md
- workflow/reasoning/ELAI-TOKEN-EFFICIENCY-PLAN-2026-06-08.md

Referenced supporting candidate:

- workflow/research/grok-elai-trust/R5-human-as-verifier.md

The same grok-elai-trust directory also contains 00-PROMPT.md, 06-research-distilled-bundle.md,
ELAI-F-plan-summary.md, ELAI-G-plan-summary.md, ELAI-H-plan-summary.md, R1-demand-gtm.md,
R2-technical-trust-spine.md, R3-measurement-safetydelta.md, and R4-competitive-landscape.md. The summary
does not assign all of these to an I session; inspect I-0 and I-41..I-44 before deciding which are binding.

### Track 20 competitor sources

Required list and summaries:

- docs/.source/STEAL-LIST.md
- docs/.source/lazycodex-SUMMARY.md
- docs/.source/gajae-code-SUMMARY.md
- docs/.source/roach-pi-SUMMARY.md

Original-code roots present:

- docs/.source/lazycodex/ — 619 files at discovery time
- docs/.source/lazycodex-main/ — 946 files at discovery time
- docs/.source/gajae-code/ — 3,080 files at discovery time
- docs/.source/roach-pi/ — 599 files at discovery time

The exact per-session source files required by SL-10/SL-11 are not enumerated in I-SUMMARY_INDEX.md.
Resolve them from I-STEAL-GROUNDING-MAP.md and I-45..I-54 before any port. The two lazycodex roots are a
specific ambiguity that the grounding map/original plans must resolve.

### Generated-output and status checks

- docs/spec/rra-spec-v1.md is already present even though I-40 is labeled BUILD;
  inspect its conformance/status rather than assuming either shipped or complete.
- workflow/reasoning/I-43-phase0-memo.md is absent at discovery time; it is a
  planned generated output, not a missing source.
- metrics/phase_I3.toml is absent at discovery time; it is a planned measurement
  output, not a missing source.
- IS/IN/IC families, GD/R/W rules, delta IDs, Track-20 declines, exact shipped/dark substrate claims, and
  current grounding counts require I-0 plus the grounding maps. The summary's 2026-06-10 status snapshot
  must not be treated as current implementation truth.
- I-49 explicitly requires current AWS/Azure protocol documentation and a full-form plan before execution;
  those current external documents are intentionally not inferred from this frozen summary.

## Extraction totals

- **1,001 atomic per-session feature/behavior records** above.
- **54 session sections**, each with an exact summary heading and line anchor.
- **21 corpus-level cross-cutting records**.
- Evidence labeling within the atomic inventory is overwhelmingly explicit; ambiguous items are isolated
  rather than silently resolved.
- No original-plan-only feature was invented or merged into this summary extraction.
