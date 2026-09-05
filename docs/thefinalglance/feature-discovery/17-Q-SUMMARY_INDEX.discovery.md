# ELAI Q-plan feature discovery — Q-SUMMARY_INDEX

## Scope and evidence discipline

- Primary source read in full: `docs/thefinalglance/Q-plan/Q-SUMMARY_INDEX.md` (148/148 lines).
- Orientation only: `docs/thefinalglance/ELAI-SPEC-OVERVIEW.md`, especially the Q overview. It describes intended design, not shipped/wired state.
- Per assignment, no Q original plan was opened. Their paths and existence were inventoried so later deep-reading agents can cover them.
- Every item in the atomic inventory is **explicit** in the primary source unless it is placed in the later **Inferred interactions** or **Ambiguities / unresolved details** sections.
- “Adds/builds” below means “the Q plan specifies,” not “confirmed shipped.” The summary itself contains planned, conditional, pending, rejected, refuted, and reconnaissance-dependent work.
- Source anchors use line numbers in `Q-SUMMARY_INDEX.md`.

## Quantitative coverage

| Coverage axis | Result |
|---|---:|
| Primary-summary lines read | 148 / 148 (100%) |
| Indexed Q units covered | Q-1..Q-15: 15 / 15 (100%) |
| Tracks covered | 6 / 6 (100%) |
| Named Q-9 sub-units covered | 2 / 2 |
| Named Q-11 sub-units covered | C6, C10, baseline: 3 / 3 |
| Named Q-12 sub-units covered | C3, C1+C2: 2 / 2 |
| Named Q-13 sub-units covered | C11: 1 / 1 |
| Named Q-14 sub-units covered | E-1, E-2, E-3, E-4: 4 / 4 |
| Named Q-15 sub-units covered | S-1, S-2, S-3, S-4, S-6: 5 / 5; S-5 recorded as deferred |
| Explicit atomic items below | 454 |
| Inferred interactions, separately labeled | 10 |
| Ambiguities / unresolved details, separately labeled | 24 |
| Unindexed Q original candidates found | 1 (`Q-16_elai_on_elai_e2e_functional_suite.md`) |

## Q-wide intent and boundaries

Source: lines 1-7, 39-40, 72-73, 84-85, 98-99, 124-125, 145-146.

- **QW-01 [purpose]** Q is a non-product-surface letter: it hardens or measures the trust spine instead of introducing a new primary user surface.
- **QW-02 [thesis]** Q treats the deterministic harness as the product and the model as a replaceable component.
- **QW-03 [proof]** Q is intended to move PS-3, PS-4, PS-6, or K measurement evidence, depending on the unit.
- **QW-04 [authority boundary]** Research citations do not substitute for target grounding or execution.
- **QW-05 [authority boundary]** Novelty labels do not substitute for target grounding or execution.
- **QW-06 [authority boundary]** Human preference does not substitute for target grounding or execution.
- **QW-07 [authority boundary]** Generic extra deliberation does not substitute for target grounding or execution.
- **QW-08 [reuse]** Every Q unit must compose with named existing artifacts and re-implement none of them.
- **QW-09 [done-gate]** Every unit is framed by a live trigger, an off-agent artifact, and a negative case that must fail.
- **QW-10 [advisory invariant]** TD-3, “advisory-never-the-gate,” binds Q-2 and Q-3.
- **QW-11 [sequence]** Distribution hygiene is sequenced after the proof spine, specifically after PS-2/PS-4.
- **QW-12 [research sequence]** Instrumentation and spend-honest measurement must precede research-search changes.
- **QW-13 [external-claim discipline]** Every AIDE² vendor number remains `[EXTERNAL]`, never an ELAI result.
- **QW-14 [external-claim discipline]** Every Evo/NeMo mechanism remains `[EXTERNAL]` when discussed as a vendor mechanism.
- **QW-15 [no training expansion]** The autoresearch work is harness Python/Rust on the free-lane apparatus; it does not add rented-GPU or training-layer work.

## Exhaustive atomic feature inventory

### Q-1 — standing goals [KEYSTONE]

Source: lines 11-16. Track `ORCHESTRATION-ECONOMICS`; lever `finishing-is-the-enrollment`; grounding `MODERATE`; nature `NEW`; effort `M-L`; `WAVE 1`; feeds `PS-6`.

- **Q1-01 [lifecycle]** Completing a unit enrolls a persistent `STANDING GOAL`.
- **Q1-02 [predicate]** A standing goal’s predicate is the completed unit’s own frozen-checker command.
- **Q1-03 [state/data]** `crates/orchestrator` gains a goal-ledger table.
- **Q1-04 [design choice]** The goal ledger sits over O-2 `TaskSpec`s.
- **Q1-05 [immutability]** The rejected alternative is adding cadence directly to the frozen gate, because that would violate R8.
- **Q1-06 [enrollment seam]** Enrollment occurs at the FSM `COMPLETE` / auditor-`PASS` seam.
- **Q1-07 [scheduler]** Each standing goal gets an O-2 scheduled `TaskSpec` using `task_runner` intervals.
- **Q1-08 [execution]** A predicate runner re-runs the frozen checker on every scheduled cadence.
- **Q1-09 [event]** Enrollment emits `GoalEnrolled` at `DONE`.
- **Q1-10 [event]** Each scheduled successful check emits `GoalVerified`.
- **Q1-11 [failure]** A predicate exit code other than zero produces `GoalViolated`.
- **Q1-12 [event payload]** `GoalViolated` names suspected causes in `suspects`.
- **Q1-13 [safety]** The sentinel names suspects but never attempts a repair.
- **Q1-14 [safety]** A sentinel that changes files is a failing implementation.
- **Q1-15 [workflow]** A violation re-enters ELAI through the existing front door/router.
- **Q1-16 [UI]** The existing intervention-card inbox renders the `GoalViolated` item.
- **Q1-17 [flakiness]** A flaky predicate is passed to F-5 quarantine.
- **Q1-18 [durability]** A flaky standing goal is never deleted merely for being flaky.
- **Q1-19 [tiering]** Ratchet-weld and compost-proposer behavior can ride on the ledger at Tier 2.
- **Q1-20 [test]** The live proof ships a unit to `COMPLETE`, reverts its wiring, advances ticks, and expects the first post-revert tick to violate.
- **Q1-21 [negative test]** Deleting a flaky goal must fail.
- **Q1-22 [dependencies]** Q-1 consumes O-2 scheduling, the SSB journal, F-5 quarantine, I-25 routing/intervention cards, the 3F rules engine, and each unit’s frozen verifier.

### Q-2 — quorum wake-gate

Source: lines 18-23. Track `ORCHESTRATION-ECONOMICS`; lever `cross-family-blind-quorum`; grounding `MODERATE`; nature `EXTENSION`; effort `M`; `WAVE 2`; feeds `PS-4`; depends on O-2 ticks.

- **Q2-01 [trigger]** The wake-gate runs on an O-1/O-2 scheduled tick.
- **Q2-02 [voters]** It fans out to `N` cheap voters from the J free-API pool.
- **Q2-03 [diversity]** The voters must be from distinct model families.
- **Q2-04 [independence]** Voters independently decide whether the tick is actionable.
- **Q2-05 [blindness]** Each voter votes blind rather than seeing or coordinating with the other votes.
- **Q2-06 [decision]** The expensive conductor wakes only on a majority-yes result.
- **Q2-07 [placement]** Wake triage precedes the 4G `AutonomousGate` evaluator call.
- **Q2-08 [cost boundary]** The quorum gates cost/attention, never correctness.
- **Q2-09 [delay semantics]** A false-no can delay work only until a later tick.
- **Q2-10 [in-flight safety]** A voter must not suppress an in-flight result.
- **Q2-11 [quiet behavior]** A non-actionable tick has approximately voter-only cost and must not wake the expensive conductor.
- **Q2-12 [actionable behavior]** An actionable tick wakes the conductor exactly once.
- **Q2-13 [event]** Every voter emits `WakeQuorumVoted`; there are `N` distinct-family rows per tick.
- **Q2-14 [event]** Every tick emits one `WakeGated` row.
- **Q2-15 [event data]** `WakeGated.woke` is false on the quiet case and true on the actionable case.
- **Q2-16 [metric]** Wake precision must be recomputable from the journal/SSB evidence.
- **Q2-17 [self-removal]** At Tier 2, the filter self-uninstalls if measured wake precision proves it redundant.
- **Q2-18 [UI/ops]** There is no new frontend; evidence is exposed through SSB rows and the existing dispatch-ledger view.
- **Q2-19 [negative test]** A quiet tick that still wakes must fail.
- **Q2-20 [dependencies]** Q-2 consumes O-1 daemon/O-2 ticks, 4G gate evaluation, J free-API voters, I-25 routing, D-13’s cross-family principle, and the SSB measured-number idiom.

### Q-3 — advisor lane + K-2 advisor arm

Source: lines 25-30. Track `ORCHESTRATION-ECONOMICS`; lever `cheap-executor-flagship-advice`; grounding `MODERATE` plus an `[EXTERNAL]` baseline; nature `EXTENSION`; effort `M-L`; `WAVE 2`; feeds `PS-4`.

- **Q3-01 [execution lane]** The cheap model remains the executor on every task turn.
- **Q3-02 [tool]** The flagship model is exposed to the weak-lane loop as a tool.
- **Q3-03 [invocation]** The executor initiates the advisor call mid-loop when judgment is needed, rather than only after failure or before dispatch.
- **Q3-04 [frequency intent]** The design budgets approximately one flagship consultation per task.
- **Q3-05 [continuity]** Advice flows back to the same cheap executor.
- **Q3-06 [identity invariant]** An advisor call must not swap the executor identity.
- **Q3-07 [payload]** The advisor receives the I-11 `BeliefState` under I-7 payload discipline.
- **Q3-08 [payload cap]** The advisor payload is capped at `<=2K`.
- **Q3-09 [budget]** Advisor spend is governed by the H-9 spend ledger.
- **Q3-10 [backend]** The flagship backend uses the 4M `RuntimeAdapter`.
- **Q3-11 [authority]** Advice is advisory and can never be presented as a `PASS` verdict.
- **Q3-12 [event]** A consultation emits `AdvisorConsulted`.
- **Q3-13 [event invariant]** `AdvisorConsulted` carries/verifies the unchanged executor identity.
- **Q3-14 [over-budget behavior]** A second call over the budget is flagged `over_budget`.
- **Q3-15 [CLI]** `elai bench routing-economics --arm advisor` recomputes advisor economics.
- **Q3-16 [bench arm]** K-2 gains an `ADVISOR` arm beside local, cloud, and hybrid.
- **Q3-17 [metric]** The advisor arm reports dollars per verified task.
- **Q3-18 [external baseline]** The Anthropic `92%-quality-at-63%-cost` SWE-bench-Pro figure is retained only as a tagged `[EXTERNAL]` comparison baseline.
- **Q3-19 [claim safety]** The Anthropic figure must never be represented as an ELAI measurement.
- **Q3-20 [UI]** No frontend is added.
- **Q3-21 [negative test]** Swapping executors during consultation or printing advice as a pass verdict must fail.

### Q-4 — swap-restart attribution

Source: lines 32-37. Track `ORCHESTRATION-ECONOMICS`; lever `iteration-state-is-model-conditioned`; grounding `STRONG` from an `autonomous_gate.rs` read; nature `EXTENSION`; effort `S-M`; `WAVE 2`; guards `PS-4/PS-5`.

- **Q4-01 [trigger]** A mid-unit model swap caused by fallback, rotation, or substitution activates the rule.
- **Q4-02 [fresh restart]** The replacement model restarts the unit fresh.
- **Q4-03 [discard]** Prior-model partial diffs are discarded.
- **Q4-04 [discard]** Prior-model self-consistency state is discarded.
- **Q4-05 [discard]** Prior-model style commitments are discarded.
- **Q4-06 [retained state]** Only the I-11 `BeliefState` crosses into the replacement model’s fresh start.
- **Q4-07 [context invariant]** The new model’s turn-0 context excludes prior turns.
- **Q4-08 [existing event]** `SubstituteModel{from,to}` continues to emit `ModelSwap` when triggered after `max_restarts_exhausted`.
- **Q4-09 [new event]** The restart branch additionally emits `ModelSwapRestart{discarded_turns}`.
- **Q4-10 [ordering]** `ModelSwap` precedes `ModelSwapRestart`.
- **Q4-11 [accounting]** `discarded_turns` records restart tax so PS-4 economics include discarded work.
- **Q4-12 [test]** The specified live case expects `discarded_turns>0` after exhausting restarts.
- **Q4-13 [attribution]** Fresh restart preserves clean per-model credit in F-6/I-25 and related Q/K measurements.
- **Q4-14 [failure]** Continuing on old partial state must fail.
- **Q4-15 [failure]** Restarting without a `ModelSwapRestart` row must fail.
- **Q4-16 [recon disposition]** Consumer behavior is `RECON-1`; if restart-fresh already exists, Q-4 collapses to a doctrine line plus the attribution row rather than duplicating behavior.
- **Q4-17 [dependencies]** Q-4 consumes 4G substitution events/functions, 4M fallback chains, J rotation, I-11 state, F-6 trust accounting, and I-25 routing.

### Q-5 — local per-turn token-hash journal

Source: lines 44-49. Track `ENGINEERING-TASTE`; lever `hw-independent-determinism`; grounding `STRONG` (`bellard.org/ts_zip`); nature `EXTENSION`; effort `S-M`; `WAVE 1`; hardens the K re-executable-RRA differentiator.

- **Q5-01 [hashing]** At every local-lane turn boundary, compute `blake3(emitted_token_id_seq)`.
- **Q5-02 [storage]** Store the token hash on the J-28 `usage_turn_event` row.
- **Q5-03 [size]** The local row carries a 32-byte BLAKE3 hash.
- **Q5-04 [lane boundary]** Cloud turns carry no token hash.
- **Q5-05 [CLI]** `replay --verify` compares local token hashes turn by turn.
- **Q5-06 [localization]** Replay reports the first divergent turn, not merely an end-state mismatch.
- **Q5-07 [same-class claim]** Same-hardware-class local replay becomes token-verifiable.
- **Q5-08 [cross-class behavior]** Cross-class replay identifies the first point where the exogenous model diverges.
- **Q5-09 [drift integration]** Cross-class divergence composes with the G drift canary.
- **Q5-10 [claim scope]** The RRA claim is explicitly scoped as: harness replay anywhere.
- **Q5-11 [claim scope]** The RRA claim is explicitly scoped as: local tokens hash-verified on the same hardware class.
- **Q5-12 [claim scope]** The RRA claim is explicitly scoped as: cloud replay verifies the action stream, not tokens.
- **Q5-13 [test]** A one-turn seed change must localize to exactly that turn.
- **Q5-14 [negative test]** Reporting only an end-state mismatch must fail.
- **Q5-15 [negative test]** Emitting a token hash for a cloud turn must fail.
- **Q5-16 [exclusion]** Heavy deterministic-eval `llama.cpp` mode (`V2b`) is out and recon-only.
- **Q5-17 [dependencies]** Q-5 consumes G-2 RRA/replay, J-28 usage rows, the GD-5 cloud boundary, the G drift canary, and BLAKE3.

### Q-6 — gate-manifest evidence-corruption corpus + kill-rate

Source: lines 51-56. Track `ENGINEERING-TASTE`; lever `test-the-gates-are-the-product`; grounding `STRONG` (`sqlite.org/testing` and `feature-map:1495`); nature `EXTENSION`; effort `M`; `WAVE 1`; feeds the K compliance/appsec brief.

- **Q6-01 [generator]** Build one deterministic corruption-corpus generator keyed by evidence type.
- **Q6-02 [manifest schema]** Every gate-manifest entry declares the evidence types that gate reads.
- **Q6-03 [evidence type]** The generator supports journal evidence.
- **Q6-04 [evidence type]** The generator supports SSB-database evidence.
- **Q6-05 [evidence type]** The generator supports hash evidence.
- **Q6-06 [evidence type]** The generator supports manifest evidence.
- **Q6-07 [mutation]** Generate truncated evidence.
- **Q6-08 [mutation]** Generate corrupt-msgpack evidence.
- **Q6-09 [mutation]** Generate missing evidence.
- **Q6-10 [mutation]** Generate flipped-hash evidence.
- **Q6-11 [mutation]** Generate zero-row evidence.
- **Q6-12 [fault injection]** Generate OOM-injection cases.
- **Q6-13 [gate property]** Every gate must block every corruption item applicable to its declared evidence types.
- **Q6-14 [metric]** Measure a per-gate kill rate.
- **Q6-15 [event]** Emit `GateKillRateMeasured` for every gate.
- **Q6-16 [regression]** Kill rate is regression tracked.
- **Q6-17 [expected result]** A fully fail-closed gate scores `1.0`.
- **Q6-18 [diagnostic]** A gate stubbed to fail open on a zero-row item scores below `1.0` and is named.
- **Q6-19 [frozen evidence]** The corpus and runner are frozen auditor artifacts under R8.
- **Q6-20 [separation]** The run being judged never runs or authors its own corruption corpus/runner.
- **Q6-21 [CLI]** `elai bench gate-kill-rate` runs the corpus over the manifest.
- **Q6-22 [tooling]** The generator and runner live in `.claude/tools`, and evidence declarations extend `.claude/gate-manifest.txt`.
- **Q6-23 [negative test]** A gate that passes corrupt/missing evidence while scoring `1.0` must fail the system.
- **Q6-24 [negative test]** A hand-listed nondeterministic corpus must fail.
- **Q6-25 [invariant]** The unit operationalizes wiring R1/R2: missing evidence is failure and a dead evidence path must scream.
- **Q6-26 [dependencies]** Q-6 consumes the frozen gate manifest, H’s trading-gate mutant idiom, 3I DST fault injection, SSB measured rows, and `gate_freeze_check.py`.

### Q-7 — weak-lane code-style bench

Source: lines 58-63. Track `ENGINEERING-TASTE`; lever `weak-model-navigable-code`; grounding `MODERATE` with hypothesized ELAI effect; nature `NEW (bench-first)`; effort `M`; `WAVE 2`; feeds `PS-4`.

- **Q7-01 [objective]** The bench measures weak-local-lane task success as the objective of code-style policy.
- **Q7-02 [corpus]** It runs the D-6/E-54 evaluation shard.
- **Q7-03 [lane]** It uses the shipped weak local lane.
- **Q7-04 [variant]** Compare file-size cap on versus off.
- **Q7-05 [variant]** Compare explicit versus inferred types.
- **Q7-06 [variant]** Compare colocated versus bare documentation.
- **Q7-07 [variant]** Compare flat versus deep nesting.
- **Q7-08 [metric]** Measure pass rate for each controlled variant.
- **Q7-09 [event]** Emit `WeakLaneStyleBench{variant, shard_id, pass_rate, n}` for each variant.
- **Q7-10 [comparability]** Compared variants use the same `shard_id`.
- **Q7-11 [statistics]** Classify the result as `WIN`, `NULL`, or `LOSS` against an `n`-aware noise band.
- **Q7-12 [promotion gate]** Promote a style rule into the L output-quality gate only after a beyond-noise `WIN`.
- **Q7-13 [tiering]** The L-gate rule is a conditional Tier-2 orchestrator change, not part of the initial bench.
- **Q7-14 [cheap falsifier]** Ship the cheapest falsifier first: two variants on one shard.
- **Q7-15 [null handling]** A `NULL` drops the idea and records the null result.
- **Q7-16 [CLI]** `elai bench weak-lane-style` reuses the K-2 harness.
- **Q7-17 [negative test]** Reporting a within-noise delta as a win must fail.
- **Q7-18 [negative test]** Promoting an L-gate rule without a `WIN` row must fail.
- **Q7-19 [dependencies]** Q-7 consumes D-6/E-54, the local weak lane, K-2 shard harness, C code-completeness knobs, L quality gate, and SSB measured rows.

### Q-8 — verifier size/dep ceiling

Source: lines 65-70. Track `ENGINEERING-TASTE`; lever `single-binary-made-a-number`; grounding `STRONG` (`bellard.org/ts_server` plus M design); nature `NEW-small`; effort `S`; `WAVE 3`; proof-spine relationship `none (discipline)` with delegated acknowledgement.

- **Q8-01 [measurement]** Measure the standalone verifier’s crate count.
- **Q8-02 [measurement]** Measure the standalone verifier’s binary size in bytes.
- **Q8-03 [ceiling]** Define a crate-count ceiling after a real verifier baseline exists.
- **Q8-04 [ceiling]** Define a binary-size ceiling after a real verifier baseline exists.
- **Q8-05 [build invariant]** Assert that the verifier build uses no network dependency.
- **Q8-06 [build invariant]** Assert that the verifier build uses no Python dependency.
- **Q8-07 [CI]** A CI lint blocks a build that crosses a `SET` ceiling.
- **Q8-08 [pending behavior]** A `PENDING` ceiling reports measurement but does not block.
- **Q8-09 [baseline integrity]** The ceiling remains pending until an actual M-verifier build establishes the baseline.
- **Q8-10 [claim integrity]** Fabricating a ceiling before a real build must fail.
- **Q8-11 [acceptance]** The size/dependency discipline is cross-linked into M’s verifier acceptance criterion.
- **Q8-12 [CLI]** `elai bench verifier-size` performs the measurement.
- **Q8-13 [event]** Emit `VerifierSizeMeasured{crate_count, binary_bytes, ceiling_*}`.
- **Q8-14 [test]** Add a bloat dependency past a set ceiling and require a block.
- **Q8-15 [rejection]** TextSynth is rejected as a verifier dependency because it is binary-only and non-commercial.
- **Q8-16 [integration flexibility]** Measurement/lint may live in `.claude/tools` or the M-verifier build.
- **Q8-17 [dependencies]** Q-8 consumes M’s standalone verifier/open evidence standard, K-4’s pending/set ceiling idiom, claims-policy lint, and the E-51/4M single-binary posture.

### Q-9 — distribution hygiene (2 units: fork-hygiene + one-registry)

Source: lines 75-85. Track `DISTRIBUTION`; lever `maintenance-economics-without-new-build`; grounding `STRONG` from Kilo/Hermes overviews; nature `EXTENSION x2`; effort `S-M`; `WAVE 3`; proof-spine relationship `none (distribution)` with delegated acknowledgement.

- **Q9-A01 [scope]** Fork hygiene applies to the 4M vendored RuntimeAdapter CLIs that still track live upstreams.
- **Q9-A02 [exclusion]** Fork hygiene explicitly does not apply to the already-diverged Phase-0 fork.
- **Q9-A03 [marker]** ELAI-side changes in a vendored CLI carry `elai_change` markers.
- **Q9-A04 [CI]** CI quarantines/inspects those vendored areas.
- **Q9-A05 [failure]** Unmarked ELAI-side drift inside a vendored CLI blocks.
- **Q9-A06 [maintenance outcome]** Marking and quarantine keep the local diff small and re-vendorable.
- **Q9-B01 [registry]** Define one canonical surface/command registry.
- **Q9-B02 [derivation]** The dashboard derives its commands from that registry.
- **Q9-B03 [derivation]** The CLI derives its commands from that registry.
- **Q9-B04 [derivation]** The canvas derives its commands from that registry.
- **Q9-B05 [derivation]** The MCP control plane derives its commands from that registry.
- **Q9-B06 [checker]** Extend `cross_lang_diff` to check command-surface symmetry.
- **Q9-B07 [drift failure]** A surface command absent from the registry blocks as drift.
- **Q9-B08 [orphan failure]** A registry command exposed by no surface blocks as drift.
- **Q9-B09 [frontend pair]** Registry-to-surface symmetry itself is the required cross-language frontend pair.
- **Q9-01 [artifact]** Both units produce frozen-checker exit codes, not new SSB rows.
- **Q9-02 [auditor integration]** Both checks run in the auditor’s frozen cross-language/freeze pass.
- **Q9-03 [test]** The live proof injects an unmarked vendored edit and an off-registry CLI subcommand and expects both checks to block.
- **Q9-04 [runtime boundary]** Neither unit requires a new runtime build.
- **Q9-05 [dependencies]** Q-9 consumes 4M vendored CLIs, `cross_lang_diff.py`, `ts-rs`, J MCP, N-2 corpus-composition checking, and A-1 one-source discipline.

### Q-10 — source-scoped relational research packets + executable permission ladder

Source: lines 87-99. Track `RESEARCH-CONDITIONED-CREATIVITY`; lever `exogenous-relational-inspiration`; grounding `STRONG` for mechanism / `HYPOTHESIS` for ELAI effect; nature `NEW then conditional EXTENSION`; effort `L`; `WAVE 2 benchmark-first`; conditionally feeds `PS-3`.

#### Q-10 packet and evidence boundary

- **Q10-01 [schema]** Define a versioned `ResearchInspirationPacket`.
- **Q10-02 [schema]** A packet carries the source mechanism.
- **Q10-03 [schema]** A packet carries per-claim support edges.
- **Q10-04 [schema]** A packet carries relation/source/predicate support edges.
- **Q10-05 [schema]** A packet carries object mappings.
- **Q10-06 [schema]** A packet carries shared relations.
- **Q10-07 [schema]** A packet carries invariants.
- **Q10-08 [schema]** A packet carries disanalogies.
- **Q10-09 [schema]** A packet carries contradiction references.
- **Q10-10 [schema]** A packet carries source references.
- **Q10-11 [schema]** A packet carries a transfer hypothesis.
- **Q10-12 [schema]** A packet carries a predicted observation.
- **Q10-13 [schema]** A packet carries a target falsifier.
- **Q10-14 [authority type]** Every packet declares `evidence_scope=SOURCE_ONLY`.
- **Q10-15 [authority boundary]** Source edges cannot prove that a relation is shared with the target.
- **Q10-16 [authority boundary]** Source edges cannot support the target side of the claim.
- **Q10-17 [traceability]** Every conditioned candidate emits a `TRANSFER-EDGE`.
- **Q10-18 [target proof]** Target anchors remain independent of the source packet.
- **Q10-19 [target proof]** Target grounding remains independent of the source packet.
- **Q10-20 [target proof]** Target execution remains independent of the source packet.
- **Q10-21 [architecture]** Research supplies bounded mechanisms and relations, not authority.
- **Q10-22 [architecture]** The design is one-way and typed, with no feedback cycle, archive-conditioned controller, or shared memory.

#### Q-10 Tier 0 and B-CREATIVITY-0

- **Q10-23 [Tier 0]** Freeze packet validation.
- **Q10-24 [Tier 0]** Freeze canonical BLAKE3 packet identity.
- **Q10-25 [Tier 0]** Freeze `K=30` fixtures.
- **Q10-26 [security]** Isolate hidden outcomes from the candidate-generation path.
- **Q10-27 [Tier 0]** Freeze stage manifests.
- **Q10-28 [fairness]** Match preprocessing across treatment and controls.
- **Q10-29 [fairness]** Match token envelopes across treatment and controls.
- **Q10-30 [preflight]** Run a locked preflight before the experiment.
- **Q10-31 [arm]** Include the `DIRECT` arm.
- **Q10-32 [arm]** Include the `MATCHED_SCAFFOLD` arm.
- **Q10-33 [arm]** Include the `VERIFIED_EVIDENCE` arm.
- **Q10-34 [arm]** Include the `RELATIONAL_PACKET` arm.
- **Q10-35 [arm]** Include a schema-valid relation-`SHUFFLED_PACKET` control.
- **Q10-36 [enumerability]** The strategy space is exactly enumerable and executable.
- **Q10-37 [support criterion]** Support requires paired-CI95 wins on Recovery against every control.
- **Q10-38 [support criterion]** Support requires a best-value win against every control.
- **Q10-39 [guard]** Support also requires validity guards to pass.
- **Q10-40 [guard]** Support also requires grounding guards to pass.
- **Q10-41 [guard]** Support also requires cost guards to pass.
- **Q10-42 [control meaning]** `MATCHED_SCAFFOLD` tests whether generic extra target deliberation explains any lift.
- **Q10-43 [control meaning]** Relation shuffling tests whether the relational mapping itself matters.
- **Q10-44 [control meaning]** `VERIFIED_EVIDENCE` tests whether relational structure adds value beyond showing the research.
- **Q10-45 [control meaning]** `DIRECT` tests whether the complete conditioning pipeline helps at all.
- **Q10-46 [outcome authority]** Target execution decides whether transfer succeeded.

#### Q-10 conditional permission ladder

- **Q10-47 [B1 gate]** Only B-CREATIVITY-0 support permits B-CREATIVITY-1.
- **Q10-48 [B1 bench]** B-CREATIVITY-1 would use `K=30` real ELAI improvement fixtures.
- **Q10-49 [B1 bench]** B-CREATIVITY-1 fixtures use hidden deterministic micro-executors.
- **Q10-50 [adoption gate]** Only B1 support permits canonical `paper-to-plan` adoption.
- **Q10-51 [adoption gate]** Only B1 support permits canonical `idea-intake` adoption.
- **Q10-52 [B2]** A later B-CREATIVITY-2 is observe-only.
- **Q10-53 [B2 metric]** B2 requires a win on receipt-valid meaning-cluster coverage.
- **Q10-54 [B2 metric]** B2 requires a win on best verified outcome.
- **Q10-55 [B2 guard]** B2 must not harm PS-3.
- **Q10-56 [B2 guard]** B2 must not harm N-16.
- **Q10-57 [B2 guard]** B2 must not harm the cost guard.
- **Q10-58 [B2 guard]** B2 must not harm the proven/unproven lane wall.
- **Q10-59 [runtime gate]** Only B2 support permits `/reason --profile explore` consumption.
- **Q10-60 [integration]** Tier 0/1 lives only in `.claude/tools`.
- **Q10-61 [integration]** Tier 2 canonical behavior lives in `.claude/skills` only after B1 support.
- **Q10-62 [parity]** `.agents` parity is a separate implementation slice.
- **Q10-63 [integration]** Tier 3 Rust/CLI is an explicit explore consumer only after B2 support and live-L seam reconnaissance.
- **Q10-64 [SSB boundary]** Tiers 0 through 2 add no SSB action type.
- **Q10-65 [SSB reuse]** Tier 3 should prefer an existing L/N measured row.
- **Q10-66 [schema restraint]** Tier 3 may add at most one discriminated row, and only after live schema reconnaissance.

#### Q-10 artifacts, failures, and actual disposition

- **Q10-67 [artifact]** Preserve frozen hashes.
- **Q10-68 [artifact]** Preserve packets and support edges.
- **Q10-69 [artifact]** Preserve raw outputs from all five arms.
- **Q10-70 [artifact]** Preserve matched preprocessing and token counts.
- **Q10-71 [artifact]** Preserve deterministic execution receipts.
- **Q10-72 [artifact]** Preserve unfused metrics and costs.
- **Q10-73 [artifact]** Preserve the bootstrap-vector hash.
- **Q10-74 [artifact]** Preserve the joint disposition.
- **Q10-75 [artifact]** Preserve packet ID and transfer edge.
- **Q10-76 [artifact]** Preserve an independent target receipt.
- **Q10-77 [negative test]** Hidden-answer leakage must fail.
- **Q10-78 [negative test]** Post-freeze mutation must fail.
- **Q10-79 [negative test]** A shuffled mapping that matches treatment must fail.
- **Q10-80 [negative test]** Accepting a source citation as target proof must fail.
- **Q10-81 [lane restriction]** Packet retrieval from code-fix, auditor, critic, or verified-case lanes must fail.
- **Q10-82 [run history]** The first two B0 runs were invalid: one used a padding proxy and one had lane-meter nondeterminism.
- **Q10-83 [valid result]** Version 3 produced a valid run (`run 3 / b0v3`).
- **Q10-84 [result]** On P1 strategy Recovery, `RELATIONAL_PACKET` lost by `-0.046` versus both `DIRECT` and `MATCHED_SCAFFOLD`.
- **Q10-85 [result]** On P1 it lost by `-0.029` versus `VERIFIED_EVIDENCE`.
- **Q10-86 [result]** On P1 it lost by `-0.034` versus `SHUFFLED`.
- **Q10-87 [result]** On P2 execution it tied the controls.
- **Q10-88 [cost result]** Total cost was approximately `8.8x DIRECT`.
- **Q10-89 [guard result]** The G3 cost guard failed.
- **Q10-90 [verdict]** `B-CREATIVITY-0` is `VALID REFUTED` as of 2026-07-14.
- **Q10-91 [halt]** The permission ladder halts: no B-CREATIVITY-1.
- **Q10-92 [halt]** No Q-10 skill, Rust, or runtime consumer is permitted.
- **Q10-93 [halt]** No `paper-to-plan` or `idea-intake` adoption is permitted.
- **Q10-94 [interpretation]** The one-way typed-packet design itself is not declared refuted; the tested research-to-creativity coupling did not pay on the synthetic falsifier.
- **Q10-95 [next-search boundary]** Any next creativity attempt must use a different mechanism.
- **Q10-96 [binding closeout]** The B-0/B-1/B-3 closeout remains binding: no `ProbeUtility`, cycles, operator allocation/fade, B-2/B-4, or planning-archive-to-L bridge.

### Q-11 — novelty-instrument canary + fixed-budget explore reporting + pre-change baselines

Source: lines 101-108. Track `AIDE2-RESEARCH-INSTRUMENT-ECONOMICS`; lever `measuring-instrument-before-search-improvement`; grounding `STRONG`; nature `BUILD-on-shipped`; effort `M`; wave `AIDE2-1`; feeds PS-4 via C10, PS-3 via C6/baseline, and PS-6 via baseline t0.

- **Q11-01 [C6 canary]** Exercise the live novelty-classification path with a known-`SAME` item.
- **Q11-02 [C6 canary]** Exercise the live novelty-classification path with a known-`DISTINCT` item.
- **Q11-03 [fail closed]** A miss sets `instrument_strength=FAILED`.
- **Q11-04 [suppression]** A failed canary suppresses all novelty-verdict rows rather than emitting dead labels.
- **Q11-05 [runtime]** Harden `novelty.rs` `NoveltyClass` / `NoveltyClassifiedEmitter` rather than creating a second runtime classifier.
- **Q11-06 [tool self-test]** Add a `conjecture_triage.py` novelty-canary leg.
- **Q11-07 [tool self-test]** `--self-test-canary` fails if no live canary is present.
- **Q11-08 [enablement]** The live test uses `ELAI_NOVELTY_LABEL_ENABLE=1`.
- **Q11-09 [C10 metric]** Extend `frontier_strategy_bench.py` with value-per-dollar columns.
- **Q11-10 [budget]** Compute the columns at a locked spend cap.
- **Q11-11 [cost source]** Read cost from J-28 `usage_turn_event`; never re-derive it.
- **Q11-12 [controls]** Include two named control arms in the fixed-budget comparison.
- **Q11-13 [anti-gaming]** A raw win by spending more is marked `spend_leak`.
- **Q11-14 [promotion safety]** A `spend_leak` control-arm win is not rendered as a promotion.
- **Q11-15 [statistics]** Preserve the existing paired-CI95 within-noise gate.
- **Q11-16 [baseline]** `--baseline` captures a frozen pre-change baseline.
- **Q11-17 [baseline identity]** Freeze the baseline with BLAKE3.
- **Q11-18 [reproducibility]** Repeated baseline capture must reproduce byte for byte.
- **Q11-19 [ordering]** Q-11 ships before Q-12.
- **Q11-20 [dependency gate]** Q-12 cannot proceed until Q-11’s bench is green.
- **Q11-21 [negative test]** Emitting verdicts from a discrimination-dead classifier must fail.
- **Q11-22 [negative test]** Rendering a spend-leaking control win as promotion must fail.
- **Q11-23 [negative test]** A non-reproducible baseline must fail.
- **Q11-24 [downstream protection]** The canary protects D-15, N-10, and L-10 novelty surfaces.
- **Q11-25 [claim discipline]** The AIDE² “had no impact” and fixed-budget vendor claims remain `[EXTERNAL]`.
- **Q11-26 [status]** C6’s proof-spine sub-unit is `none` for instrument discipline and was user-acknowledged 2026-07-15.

### Q-12 — research-lane operator context compression + lineage-bandit search policy

Source: lines 110-115. Track `AIDE2-RESEARCH-INSTRUMENT-ECONOMICS`; lever `minimal-context-then-bandit-allocation`; grounding `STRONG`; nature `BUILD-on-shipped`; effort `M-L`; wave `AIDE2-2`; C3 feeds PS-4; C1+C2 proof-spine relationship `none`; depends on Q-11 bench green.

#### C3 — minimal operator context

- **Q12-C3-01 [prompt contract]** Define a minimal-prompt matrix per research operator.
- **Q12-C3-02 [history cap]** Include the newest 12 one-line node summaries.
- **Q12-C3-03 [full context cap]** Include exactly one full extract.
- **Q12-C3-04 [field discipline]** Each operator receives only its declared fields.
- **Q12-C3-05 [exclusion]** An operator prompt must not carry the full unbounded node history.
- **Q12-C3-06 [metric]** Journal a prompt-size ratio.
- **Q12-C3-07 [budget reuse]** Convert freed prompt budget into additional research iterations.
- **Q12-C3-08 [accounting]** Prompt-size ratio and reinvested iterations recompute from `usage_turn_event` under GD-2.
- **Q12-C3-09 [default]** Minimal-prompt behavior is default-on.
- **Q12-C3-10 [kill switch]** `ELAI_RESEARCH_MINIMAL_PROMPT_DISABLE` disables it.

#### C1+C2 — lineage bandit and fork-on-stall

- **Q12-C12-01 [unit boundary]** C1 and C2 are one combined build unit.
- **Q12-C12-02 [policy]** Add lineage-bandit allocation as a selectable policy.
- **Q12-C12-03 [arms]** Treat draft lineages as bandit arms.
- **Q12-C12-04 [within arm]** Allocate greedily within a selected lineage.
- **Q12-C12-05 [stall detector]** Detect a plateau when the last four `info_gain` values are flat.
- **Q12-C12-06 [fork]** On plateau, fork the best-so-far result into a fresh strategy-space lineage.
- **Q12-C12-07 [selector seam]** Implement the policy in the existing `research_tree.rs` selector seam.
- **Q12-C12-08 [state]** Track lineage/arm statistics.
- **Q12-C12-09 [experiment flag]** `ELAI_RESEARCH_BANDIT_ENABLE` selects the experiment.
- **Q12-C12-10 [default]** The bandit flag is default-off.
- **Q12-C12-11 [wire discipline]** Register the experimental flag in `.elai_cc/wire-dark-allowlist.toml` under R9.
- **Q12-C12-12 [control]** Always include a deterministic-scheduler control arm.
- **Q12-C12-13 [kill]** The experiment has a mandatory kill criterion.
- **Q12-C12-14 [bench]** Compare bandit and deterministic arms with Q-11’s fixed-budget apparatus.
- **Q12-C12-15 [spend equality]** The two arms must run at equal budget.
- **Q12-C12-16 [promotion]** Promote only after a beyond-noise win at equal budget.
- **Q12-C12-17 [replication]** Promotion additionally requires a second benchmark.
- **Q12-C12-18 [routing safety]** The bandit must not become default routing before earning promotion.
- **Q12-C12-19 [null policy]** A null result retains the deterministic scheduler.
- **Q12-C12-20 [negative test]** A bandit win with `spend_leak` must fail.
- **Q12-C12-21 [negative test]** Calling a within-noise delta a win must fail.
- **Q12-C12-22 [negative test]** Shipping the bandit as default routing must fail.
- **Q12-C12-23 [external claims]** Vendor `16x` and `+0.042..+0.053` figures remain `[EXTERNAL]`; the formula is undisclosed, realized ELAI effect is expected smaller, and the numbers are never attributed to Weco.
- **Q12-01 [trigger]** Proof uses a real multi-node run with `ELAI_WEB_RESEARCH_ORCHESTRATOR_ENABLE=1`, followed by the equal-budget bench.
- **Q12-02 [dependencies]** Q-12 extends `ResearchTree`/`ResearchNode`/`info_gain`/`ConvergenceStatus`, hardens web-research prompt builders, reuses Q-11 controls, B-40 report types, J-28 costs, and J declared-access discipline.
- **Q12-03 [UI]** No frontend is added; the research-lane experiment flag is the justification.

### Q-13 — receipt-bound Cosmos briefing lines

Source: lines 117-125. Track `AIDE2-RESEARCH-INSTRUMENT-ECONOMICS`; lever `fabricated-briefing-line-unrepresentable`; grounding `STRONG`; nature `EXTENSION`; effort `S`; wave `AIDE2-2 sibling`; proof-spine relationship `none (trust-infrastructure)`; user-acknowledged 2026-07-15.

- **Q13-01 [C11 binding]** Bind every compiled Cosmos briefing line to a receipt.
- **Q13-02 [receipt form]** A receipt may be a fetched-byte hash.
- **Q13-03 [receipt form]** A receipt may be a journal-row ID.
- **Q13-04 [resolution]** Resolve the receipt at briefing compile time.
- **Q13-05 [failure behavior]** Drop an unresolvable line.
- **Q13-06 [failure behavior]** Never silently emit an unresolvable line.
- **Q13-07 [accounting]** Count dropped lines in `dropped_receipt_lines`.
- **Q13-08 [exactness]** The count increases by exactly the number of unresolvable source lines.
- **Q13-09 [recomputability]** Recompute the count from the source-resolution log under GD-2.
- **Q13-10 [type reuse]** Reuse E-48 `VerifiedCitation` / `VerificationOutcome`.
- **Q13-11 [resolver reuse]** Reuse the H-52 anchor resolver.
- **Q13-12 [schema restraint]** Do not create a second citation type.
- **Q13-13 [default]** Receipt binding is default-on.
- **Q13-14 [kill switch]** `ELAI_COSMOS_RECEIPT_BIND_DISABLE` disables it.
- **Q13-15 [integration recon]** Implement at `compile_cosmos_briefing` in `briefing.rs` or `cosmos_worker.rs`, decided by `RECON-1`.
- **Q13-16 [premise-false behavior]** If equivalent binding already exists, collapse Q-13 to the drop-count addition only.
- **Q13-17 [test]** Compile a real briefing with resolvable lines and one dangling line and require the dangling line to be dropped/count to increment.
- **Q13-18 [negative test]** Emitting a line with no receipt must fail.
- **Q13-19 [negative test]** Allowing an unresolvable line to survive must fail.
- **Q13-20 [negative test]** Creating a second citation type must fail.
- **Q13-21 [negative wording]** The source also says “a receipt bound to the output” must fail; this is preserved as written and flagged as semantically unresolved below.
- **Q13-22 [UI]** No frontend is added; a Tier-2 UI is deferred.
- **Q13-23 [claim discipline]** This is pure ELAI-pattern reuse and makes no external effect claim.

### Q-14 — conjecture-engine instrument hardening (harness novelty canary + quantified kills + cost-honest tie-break + P-25 citation)

Source: lines 127-135. Track `AUTORESEARCH-SYNERGY`; lever `instrument-the-engine-that-picks-improvements`; grounding `STRONG`; nature `BUILD-on-shipped`; effort `M`; wave `AUTORESEARCH-1`; E-1 feeds PS-3; E-2/E-3/E-4 proof-spine relationship `none`; user-directed 2026-07-17 and acknowledgement-review flagged.

#### E-1 — harness novelty instrument

- **Q14-E1-01 [observed gap]** The archived 2026-07-15 AIDE² run has ten `UNCERTAIN` equivalence verdicts with `instrument_strength=WEAK`.
- **Q14-E1-02 [cause]** With no supplied verdicts file, `conjecture_triage.py --novelty` falls back to deterministic bin coordinates that cannot discriminate the candidates.
- **Q14-E1-03 [minting]** Add a minted-NLI-verdicts path to `conjecture_triage.py --novelty`.
- **Q14-E1-04 [judge]** Mint verdicts with a spawned Sonnet judge.
- **Q14-E1-05 [audit]** Subject minted verdicts to I-24 and N-15 audits.
- **Q14-E1-06 [canary]** Inject a known-`SAME` canary into every novelty invocation.
- **Q14-E1-07 [canary]** Inject a known-`DISTINCT` canary into every novelty invocation.
- **Q14-E1-08 [fail closed]** A canary miss sets `instrument_strength=FAILED`.
- **Q14-E1-09 [suppression]** Suppress all verdict rows while the instrument is failed.
- **Q14-E1-10 [self-test]** Add `--self-test-novelty-canary`.
- **Q14-E1-11 [dead-path detection]** The self-test fails if the canary path is absent, implementing R2.
- **Q14-E1-12 [fixture]** Use the archived AIDE² run as the live fixture.
- **Q14-E1-13 [positive test]** A healthy minted-verdict run emits at least one non-`UNCERTAIN` verdict.
- **Q14-E1-14 [scramble test]** A scrambled verdicts file emits zero verdict rows and reports `instrument_strength=FAILED`.

#### E-2 — quantified kills

- **Q14-E2-01 [schema extension]** Add an optional quantitative tail to experiment `KILL-EDGE` records.
- **Q14-E2-02 [field]** The tail may record `delta`.
- **Q14-E2-03 [field]** The tail may record the noise `band`.
- **Q14-E2-04 [field]** The tail may record `verdict`.
- **Q14-E2-05 [compatibility]** The extension is append-only.
- **Q14-E2-06 [compatibility]** Grandfathered artifacts without the tail continue to validate and fold.
- **Q14-E2-07 [recheck ordering]** `RECHECK` reinstates within-noise kills first.
- **Q14-E2-08 [power interaction]** A within-noise kill becomes a Q-15 `NULL-UNDERPOWERED` candidate.
- **Q14-E2-09 [negative test]** Making the tail mandatory and breaking grandfathered validation must fail.

#### E-4 — cost-honest SAFE tie-break

- **Q14-E4-01 [cost input]** Use measured stage-3 token cost in the `SAFE` selector.
- **Q14-E4-02 [ordering]** Cost breaks a tie before candidate ID order.
- **Q14-E4-03 [evidence discipline]** Consume measured cost; do not re-derive it.
- **Q14-E4-04 [test]** `--novelty --profile safe` must order tied candidates by measured cost.
- **Q14-E4-05 [negative test]** Re-derived cost in the tie-break must fail.

#### E-3 — citation/doctrine rider and common constraints

- **Q14-E3-01 [citation]** Cite P-25 Family A beside the exogenous-injection survival claim.
- **Q14-E3-02 [ledger]** Add a corresponding `ELAI-FEATURE-MAP.md` ledger cross-reference.
- **Q14-E3-03 [negative test]** Leaving that survival claim uncited must fail.
- **Q14-01 [authority]** `conjecture_triage.py` remains advisory and is never added to the gate manifest.
- **Q14-02 [CLI wiring]** Every new function is consumed by its `argparse` CLI arm in the same changeset.
- **Q14-03 [validation]** The live flow runs `--validate` and `--fold` on a quantified-tail artifact.
- **Q14-04 [dependencies]** Q-14 mirrors Q-11’s runtime canary concept on the harness side, extends the improvement-conjecture skill/tool, reuses the archived AIDE² receipt, and feeds Q-15’s power floor.
- **Q14-05 [UI]** No frontend is added because this is harness-engine tooling.

### Q-15 — research-bench + measurement-protocol hygiene (pareto retention + power floor + verified flag + variance helper + handoff mismatch)

Source: lines 137-146. Track `AUTORESEARCH-SYNERGY`; lever `harden-the-research-bench-and-its-measurement-protocols`; grounding `STRONG`; nature `BUILD-on-shipped` with S-1 extending Q-12; effort `M`; wave `AUTORESEARCH-2`; proof-spine relationship `none`; user-directed 2026-07-17 and acknowledgement-review flagged.

#### S-1 — Pareto specialist retention

- **Q15-S1-01 [policy arm]** Add `pareto_per_task` as a third selectable policy in Q-12’s existing selector seam.
- **Q15-S1-02 [no duplicate selector]** It is one policy arm, not a second selector implementation.
- **Q15-S1-03 [retention step]** Select the best candidate per task.
- **Q15-S1-04 [prune step]** Apply set-cover dominance pruning.
- **Q15-S1-05 [weight step]** Weight retained candidates by tasks won.
- **Q15-S1-06 [specialists]** Preserve a specialist that wins some tasks even if aggregate score would discard it.
- **Q15-S1-07 [initial consumer]** Use the bench lane as the safe first consumer.
- **Q15-S1-08 [research-lane constraint]** Do not assume research-lane per-task scores already exist.
- **Q15-S1-09 [bench]** Run a per-task-scored, equal-budget comparison under Q-11’s apparatus.
- **Q15-S1-10 [promotion]** Require beyond-noise benefit without `spend_leak`.
- **Q15-S1-11 [negative test]** A Pareto win with `spend_leak` must fail.

#### S-2 — power floor

- **Q15-S2-01 [preregistration]** Every preregistration includes a power block.
- **Q15-S2-02 [closeout]** Every `REFUTED` closeout includes a power block.
- **Q15-S2-03 [power field]** Record `K`.
- **Q15-S2-04 [power field]** Record paired sample count.
- **Q15-S2-05 [power field]** Record minimal detectable effect at CI95.
- **Q15-S2-06 [verdict correction]** If the CI could not detect the claimed effect, classify the outcome `NULL-UNDERPOWERED`, not `REFUTED`.
- **Q15-S2-07 [kill interaction]** Consume Q-14 E-2 quantitative kill tails to distinguish within-noise losses.
- **Q15-S2-08 [test]** A synthetic half-K reanalysis becomes `NULL-UNDERPOWERED`.
- **Q15-S2-09 [regression]** The real P-25 analysis remains `REFUTED` after the power check.
- **Q15-S2-10 [negative test]** Reclassifying a genuinely refuted family as underpowered must fail.

#### S-3 — reviewed provenance

- **Q15-S3-01 [catalog schema]** Add a per-artifact `verified` provenance bit to the bench/fixture/canary catalog.
- **Q15-S3-02 [safe default]** A newly registered artifact defaults to `verified:false`.
- **Q15-S3-03 [human review]** A human-review stamp changes the provenance state under explicit discipline.
- **Q15-S3-04 [cadence]** Compose with Q-1’s verified-invariant re-check for ongoing review state.
- **Q15-S3-05 [negative test]** A new fixture defaulting `verified:true` must fail.

#### S-4 — canonical variance helper

- **Q15-S4-01 [configuration]** Thread repeat count through the bench as configuration.
- **Q15-S4-02 [single implementation]** Use one canonical variance/majority helper across benches.
- **Q15-S4-03 [metric]** The helper computes `pass@k`.
- **Q15-S4-04 [metric]** The helper computes `pass@1[avg-of-k]`.
- **Q15-S4-05 [metric]** The helper computes `std_err`.
- **Q15-S4-06 [comparability]** Q-11 per-dollar columns use the canonical outputs so comparisons line up across benches.
- **Q15-S4-07 [negative test]** Two coexisting variance implementations must fail.

#### S-6 — handoff mismatch and track-level boundaries

- **Q15-S6-01 [resume protocol]** On resume, re-verify handoff notes against the live tree.
- **Q15-S6-02 [mismatch record]** If notes and reality disagree, write a mismatch entry.
- **Q15-S6-03 [state]** The mismatch may be represented by an optional `handoff_state` field.
- **Q15-S6-04 [negative test]** Silent re-verification without recording the mismatch must fail.
- **Q15-01 [external source]** GEPA is identified as arXiv:2507.19457 and must not be attributed to Weco.
- **Q15-02 [external discipline]** GEPA, NeMo-RL, and NeMo-Gym mechanisms remain `[EXTERNAL]`.
- **Q15-03 [design-only]** Federation and upstream-PR opportunities are recommendations, not build units.
- **Q15-04 [deferred]** S-5 campaign-grain stop conditions wait for a real multi-session research-campaign consumer.
- **Q15-05 [dependencies]** Q-15 composes with Q-12’s selector, Q-11’s fixed-budget/CI95 apparatus, Q-1 review cadence, Q-14 quantified kills, B-40 report types, and the handoff protocol/state.
- **Q15-06 [UI]** No frontend is added because this is research-bench and protocol tooling.

## Explicit workflows and how Q’s features work together

### 1. Completion-to-durability loop

1. A unit reaches FSM `COMPLETE` and its auditor passes.
2. Q-1 enrolls that exact frozen checker in the goal ledger and schedules it through O-2.
3. Each tick emits `GoalVerified` or, on nonzero exit, `GoalViolated{suspects}`.
4. A violation becomes an intervention card and re-enters I-25/front-door work routing.
5. Flaky predicates are quarantined, not erased; the sentinel cannot self-heal.

### 2. Cheap scheduled orchestration with attribution integrity

1. O-1/O-2 produces a scheduled wake site.
2. Q-2 asks blind, cross-family cheap voters whether it is actionable.
3. Majority-no avoids the flagship wake; majority-yes wakes it once. This controls cost only and cannot suppress in-flight evidence.
4. During a weak-lane task, Q-3 lets the cheap executor request budgeted flagship advice and then continue under the same identity.
5. If fallback/rotation/substitution changes the executor model, Q-4 discards model-conditioned iteration state, retains only `BeliefState`, and journals restart tax.
6. Q-4 therefore keeps Q-2/Q-3/K-2/K-18 per-model and per-dollar attribution honest (explicit in the orchestration cross-cutting note).

### 3. Harness-hardening chain

1. Q-5 anchors every local turn to a token-sequence hash and lets replay identify the first divergent turn.
2. Q-6 mutates each declared evidence type and measures whether every frozen checker fails closed.
3. Q-7 experimentally tests code-style policies on the weak lane and allows an L-gate rule only after a measured win.
4. Q-8 constrains the independent verifier’s crate/binary footprint and dependencies after a real baseline exists.
5. Together these make determinism scope, gate credibility, weak-lane navigability, and verifier simplicity measured rather than asserted.

### 4. Distribution-maintenance chain

1. Q-9A marks local drift in still-vendored RuntimeAdapter CLIs and blocks unmarked changes.
2. Q-9B centralizes the command set and derives dashboard/CLI/canvas/MCP exposures from it.
3. The frozen cross-language/freeze pass blocks both vendored drift and command-surface drift without adding runtime events.

### 5. Research instrument-before-improvement chain

1. Q-11 first proves the novelty classifier can distinguish known same/distinct cases, suppresses failed-instrument verdicts, locks budget, reads ledger cost, and freezes a pre-change baseline.
2. Only with a green Q-11 bench does Q-12 compress per-operator prompts and compare the default-off lineage bandit with the deterministic scheduler at equal budget.
3. Q-15 S-1 adds Pareto specialist retention as another arm in that same selector seam, not a new selector.
4. Q-15 S-2 prevents an underpowered null from being called a refutation; Q-15 S-4 makes repeated/cost-normalized results comparable.
5. Q-14 hardens the separate improvement-conjecture harness canary, quantifies kills, and makes safe-profile ties cost-honest; its kill tail feeds Q-15’s power analysis.
6. Q-13 independently closes the research-output provenance boundary by dropping unreceipted briefing lines.

### 6. Q-10 permission ladder — executed and halted

1. Tier 0 freezes a packet schema, identities, fixtures, outcome isolation, and five matched arms.
2. Two invalid pilots are repaired; version 3 becomes a valid falsifier.
3. The relational arm loses every P1 comparison, ties P2, and costs about 8.8 times direct, failing G3.
4. The resulting `VALID REFUTED` disposition halts B1, B2, skill adoption, Rust/CLI runtime integration, and `paper-to-plan`/`idea-intake` adoption.
5. Therefore Q-10’s proposed packet consumer is not part of the post-Q runtime under the summary’s current status.

## Constraints and invariants

| Invariant | Applies to | Exact consequence |
|---|---|---|
| Frozen checker remains independent | Q-1, Q-6, Q-9 | Do not mutate the checker/corpus from the judged run; output is an off-agent event or exit code. |
| Advisory never gates correctness | Q-2, Q-3, Q-14 | Quorum controls wake cost, advice cannot be a pass verdict, conjecture triage stays out of the gate manifest. |
| Missing/dead evidence fails closed | Q-1, Q-6, Q-11, Q-13, Q-14 | Nonzero standing predicate violates; corrupt/missing gate evidence blocks; failed canaries suppress; unresolved briefing lines drop. |
| Measured cost is consumed, not re-derived | Q-3, Q-11, Q-12, Q-14, Q-15 | Use spend/usage rows for economics, equal-budget checks, and cost tie-breaks. |
| Spend cannot manufacture a win | Q-11, Q-12, Q-15 | A spend-leaking result is flagged and cannot promote. |
| Within-noise is not a win | Q-7, Q-12, Q-14, Q-15 | No style/bandit promotion; quantified kills are reinstatable; underpowered nulls are not refutations. |
| Source evidence has source-only authority | Q-10 | Relation/target claims require independent target anchors and execution. |
| Model-conditioned work cannot cross a swap | Q-4 | Drop prior turns/diffs/commitments; retain only `BeliefState`; journal discarded work. |
| Lane-specific replay claims | Q-5 | Same-class local tokens are hashed; cloud is action-stream only; cloud rows have no token hash. |
| Pending numbers cannot block | Q-8 | Measure/report until a real baseline sets a ceiling; only a set ceiling blocks. |
| One implementation/source | Q-6, Q-9, Q-13, Q-15 | One corruption generator, one command registry, reuse citation types/resolver, one variance helper. |
| Experiments do not silently become defaults | Q-7, Q-10, Q-12, Q-15 | Promotion requires the specified gates; refuted/underpowered work remains non-runtime. |
| External numbers remain external | Q-3, Q-11, Q-12, Q-15 | Never relabel vendor figures or mechanisms as ELAI results. |

## Dependencies, status, and deferred/rejected work

| Unit | Principal dependencies | Status / gate | Deferred, rejected, or collapse rule |
|---|---|---|---|
| Q-1 | O-2, SSB, F-5, I-25, 3F, frozen checker | New, Wave 1, keystone | Ratchet-weld and compost-proposer are Tier 2. |
| Q-2 | O-1/O-2, 4G, J voters, I-25, D-13, SSB | Extension, Wave 2 | Self-uninstall is Tier 2; threshold unspecified. |
| Q-3 | I-11/I-7, 4M, H-9, K-2, H-7 | Extension, Wave 2 | No frontend. |
| Q-4 | 4G/4M/J, I-11, F-6/I-25 | Extension, Wave 2, RECON-1 | Collapse to doctrine + event row if restart-fresh already exists. |
| Q-5 | G-2/G drift, J-28, GD-5, BLAKE3 | Extension, Wave 1 | Heavy deterministic-eval llama.cpp mode V2b is out/recon-only. |
| Q-6 | Gate manifest, H mutant idiom, 3I, R1/R2/R8, SSB | Extension, Wave 1 | None stated. |
| Q-7 | D-6/E-54, weak lane, K-2, C knobs, L gate | Bench-first hypothesis, Wave 2 | L rule exists only after a win; null drops the idea. |
| Q-8 | M verifier, K-4 idiom, claims lint, E-51/4M | New-small, Wave 3; ceiling initially pending | TextSynth rejected; ceiling cannot be invented. |
| Q-9 | 4M vendored CLIs, cross-lang gate, ts-rs, J/N/A | Extension x2, Wave 3; after proof spine | Phase-0 diverged fork excluded. |
| Q-10 | B-40/4D, E-48, B-42/O, D/L/N/I, conjecture harness | B0v3 `VALID REFUTED`; permission ladder halted | No B1/B2/runtime/adoption; multiple FDL-era mechanisms remain prohibited. |
| Q-11 | novelty runtime, SSB/J-28, frontier bench, triage tool | Ships first; user-acked | Q-12 blocked until green. |
| Q-12 | Q-11 bench, research tree/orchestrator, B-40, J-28 | Minimal prompt default-on; bandit default-off experiment | Null retains deterministic scheduler; second benchmark required for promotion. |
| Q-13 | E-48/H-52, B-42/O-1, briefing compiler | Extension; RECON-1; user-acked | Collapse to drop-count if equivalent binding exists; Tier-2 UI deferred. |
| Q-14 | Q-11 concept, conjecture harness/skill, archived AIDE² run, Q-15 | Build-on-shipped; ack-review flagged | Advisory only; grandfathered no-tail artifacts remain valid. |
| Q-15 | Q-12 seam, Q-11 bench, Q-1 cadence, Q-14 tail, B-40, handoff | Build-on-shipped; ack-review flagged | S-5 waits for real campaign consumer; federation/upstream PR are recommendations only. |

Explicit deferred/rejected ledger:

- Q-5: V2b heavy deterministic-eval llama.cpp mode is out/recon-only.
- Q-7: any L output-quality style rule is deferred until a beyond-noise win.
- Q-8: numeric verifier ceilings are pending until a real build; TextSynth is rejected.
- Engineering-taste cross-cutting: neural lossless compression in the evidence path is rejected because it conflicts with the small, boring verifier.
- Q-10: B-CREATIVITY-1, B-CREATIVITY-2, skill/Rust/runtime consumers, and paper-to-plan/idea-intake adoption are halted by the valid refutation.
- Q-10: ProbeUtility, cycles, operator allocation/fade, B-2/B-4, and a planning-archive-to-L bridge remain prohibited by the binding closeout.
- Q-11/Q-12: research search changes wait on a green instrument/bench; the bandit remains default-off unless it wins twice at equal budget.
- Q-13: Tier-2 UI is deferred.
- AIDE² track: C5 novelty-survival re-judge and C9 Cosmos hidden-probe split are unscheduled until scoring semantics are formalized; otherwise C5 could punish success and C9 could reward keyword coverage.
- Q-15: S-5 campaign-grain stopping rules wait for a real multi-session campaign consumer.
- Q-15: federation and upstream-PR opportunities are recommendations, not build units.

## Inferred interactions — not explicit features

These are bounded synthesis from explicit Q statements, not claims made verbatim by the summary.

- **INF-01** Because Q-1 accepts a completed unit’s frozen verifier generically, Q-5/Q-6/Q-8/Q-9 checkers could become standing goals after those units complete; the summary does not enumerate which Q units enroll.
- **INF-02** Q-2 could reduce the wake cost of scheduled Q-1 violation handling because both use O-2 ticks, but the summary never explicitly wires Q-2 in front of Q-1 predicates.
- **INF-03** Q-3’s unchanged executor ID plus Q-4’s restart-tax row together provide enough evidence to separate “advice helped” from “a stronger model took over,” but the exact analysis query is not specified.
- **INF-04** Q-5’s first-divergent-turn hash can narrow which event/evidence inputs Q-6 should corrupt during replay debugging, but no direct Q-5-to-Q-6 API is specified.
- **INF-05** Q-8’s small verifier may eventually consume Q-6 gate-kill evidence under M’s open evidence standard, but that consumer relationship is not stated.
- **INF-06** Q-9’s one registry likely becomes the source used by future Q CLI bench additions, but the summary does not explicitly migrate Q-3/Q-6/Q-7/Q-8 commands into it.
- **INF-07** Q-11’s frozen baseline provides the t0 against which Q-12 minimal-context savings and bandit effects are attributed; exact baseline schema linkage is not given.
- **INF-08** Q-15 S-3 review stamps could themselves become Q-1 standing invariants; Q-15 says they compose with Q-1 cadence but does not define enrollment commands.
- **INF-09** Q-13 receipt failures could become research-lane quality signals, but the summary only specifies drop-and-count, not feedback into Q-12 allocation.
- **INF-10** Since Q-10 is refuted and halted, its packet schema may remain only as frozen experimental evidence rather than a canonical runtime type; the exact artifact-retention location is not stated.

## Ambiguities / unresolved details — do not invent during compilation

- **AMB-01 [Q-wide]** The heading says “All 15 sessions,” while the directory contains a present `Q-16_elai_on_elai_e2e_functional_suite.md`; Q-16 is absent from this summary and requires a separate original-plan read.
- **AMB-02 [Q-wide]** The premise narrates “four tracks,” then adds a fifth and the file later adds a sixth. The actual summary has six track sections.
- **AMB-03 [Q-1]** Goal-ledger schema, cadence defaults, retention, concurrency, and suspect-naming algorithm are not specified.
- **AMB-04 [Q-1]** “Ratchet-weld” and “compost-proposer” are named but their behavior is not defined here.
- **AMB-05 [Q-2]** `N`, majority/tie semantics, voter timeouts, family-selection rules, wake-precision formula, redundancy threshold, and self-uninstall mechanics are unspecified.
- **AMB-06 [Q-3]** “~once/task,” the actual budget, response schema, timeout/fallback behavior, and `over_budget` enforcement semantics are unspecified.
- **AMB-07 [Q-4]** The exact `BeliefState` snapshot/serialization and whether the task-runner already restarts fresh require RECON-1.
- **AMB-08 [Q-5]** “Hardware class” identity and behavior when a local provider cannot expose emitted token IDs are not specified.
- **AMB-09 [Q-6]** Corpus-generation seeds/enumeration, evidence declaration grammar, OOM injection method, denominator definition, and kill-rate regression threshold are unspecified.
- **AMB-10 [Q-7]** The n-aware band, sample size beyond the first 2x1 falsifier, exact style transformations, and promotion scope are unspecified.
- **AMB-11 [Q-8]** Both numeric ceilings and the component that owns them are intentionally unresolved until a real M build.
- **AMB-12 [Q-9]** `elai_change` marker syntax, exact vendored-CLI inventory, registry schema, derivation codegen, and command alias/version handling are unspecified.
- **AMB-13 [Q-10]** Exact Recovery/best-value formulas, CI/bootstrap parameters, all guard thresholds, packet version number, and support-edge serialization are absent from the summary.
- **AMB-14 [Q-10]** The source says the packet design itself is not refuted while the specific coupling is refuted; later compilers must not promote the design without a new mechanism and new evidence.
- **AMB-15 [Q-11/Q-14]** Q-11’s integration line assigns a novelty canary leg and `--self-test-canary` to `conjecture_triage.py`, but Q-14 later says Q-11 hardens only runtime `novelty.rs` and the harness has no sibling, then adds `--self-test-novelty-canary`. This is an internal overlap/supersession ambiguity requiring original-plan chronology or implementation recon.
- **AMB-16 [Q-11]** The two named controls are not named in the summary; locked-cap amount, per-dollar numerator, and canary fixture values are unspecified.
- **AMB-17 [Q-12]** The bandit formula is explicitly undisclosed; reward, exploration term, initialization, flatness tolerance, kill criterion, and second-benchmark identity are unspecified.
- **AMB-18 [Q-12]** `ELAI_RESEARCH_MINIMAL_PROMPT_DISABLE` is an inverted default-on kill switch; behavior when set/unset and its dark-wire status should be confirmed in the original plan.
- **AMB-19 [Q-13]** Integration ownership is unresolved between `briefing.rs` and `cosmos_worker.rs`; equivalent binding may collapse the unit.
- **AMB-20 [Q-13]** The negative phrase “a receipt bound to the output ... must FAIL” is semantically unclear. It may distinguish source-receipt binding from self-referential output binding, but the summary alone does not authorize that interpretation.
- **AMB-21 [Q-14]** Minted-NLI schema, Sonnet invocation/budget, I-24/N-15 audit thresholds, scrambled-file construction, and stage-3 cost row are unspecified.
- **AMB-22 [Q-15]** Research-lane per-task scoring may not exist; only the bench lane is safe initially. Promotion from bench to research lane is not specified.
- **AMB-23 [Q-15]** Canonical helper module/catalog path, repeat defaults, human-review authority/stamp schema, and handoff mismatch record schema are unspecified.
- **AMB-24 [sources]** Exact local digests for Bellard, SQLite, Carmack, Anthropic, Hipp, and TextSynth are not named by path in the summary; they must be recovered from originals rather than guessed.

## Deeper-source manifest

### Primary/index sources

| Source | Presence | Role / coverage |
|---|---|---|
| `docs/thefinalglance/Q-plan/Q-SUMMARY_INDEX.md` | Present; read fully | Primary evidence for this discovery. |
| `docs/thefinalglance/ELAI-SPEC-OVERVIEW.md` | Present; orientation only | System thesis and condensed Q topology; not build status. |
| `docs/thefinalglance/Q-plan/Q-0_INDEX.md` | Present; intentionally unread | Doctrine and per-unit `PROOF_SPINE` table explicitly delegated by the summary. |
| `docs/specs/guide/steps/phase-index-MASTER.txt` | Present; intentionally unread | Core phase source of truth referenced by the overview, not a Q original. |
| `docs/thefinalglance/ELAI-FEATURE-MAP.md` | Present; intentionally unread | Explicit anchors `feature-map:1495`, rejected ledger, and Q-14 E-3 cross-reference. |

### Original Q plan files — present, reserved for deep-reading agents

| Unit | Original source path | Summary anchor |
|---|---|---|
| Q-1 | `docs/thefinalglance/Q-plan/Q-1_standing_goals.md` | lines 11-16 |
| Q-2 | `docs/thefinalglance/Q-plan/Q-2_quorum_wake_gate.md` | lines 18-23 |
| Q-3 | `docs/thefinalglance/Q-plan/Q-3_advisor_lane.md` | lines 25-30 |
| Q-4 | `docs/thefinalglance/Q-plan/Q-4_swap_restart_attribution.md` | lines 32-37 |
| Q-5 | `docs/thefinalglance/Q-plan/Q-5_local_token_hash_journal.md` | lines 44-49 |
| Q-6 | `docs/thefinalglance/Q-plan/Q-6_gate_evidence_corruption_corpus.md` | lines 51-56 |
| Q-7 | `docs/thefinalglance/Q-plan/Q-7_weak_lane_codestyle_bench.md` | lines 58-63 |
| Q-8 | `docs/thefinalglance/Q-plan/Q-8_verifier_size_ceiling.md` | lines 65-70 |
| Q-9 | `docs/thefinalglance/Q-plan/Q-9_distribution_hygiene.md` | lines 77-82 |
| Q-10 | `docs/thefinalglance/Q-plan/Q-10_research_conditioned_creativity.md` | lines 89-99 |
| Q-11 | `docs/thefinalglance/Q-plan/Q-11_novelty_instrument_and_fixed_budget_bench.md` | lines 103-108 |
| Q-12 | `docs/thefinalglance/Q-plan/Q-12_research_lane_context_and_search_policy.md` | lines 110-115 |
| Q-13 | `docs/thefinalglance/Q-plan/Q-13_receipt_bound_briefing.md` | lines 117-125 |
| Q-14 | `docs/thefinalglance/Q-plan/Q-14_conjecture_engine_instrument_hardening.md` | lines 129-135 |
| Q-15 | `docs/thefinalglance/Q-plan/Q-15_research_bench_and_protocol_hygiene.md` | lines 137-146 |

### Grounding and result sources explicitly named or strongly localized

| Q unit(s) | Source / anchor | Presence and certainty |
|---|---|---|
| Q-10 | `docs/thefinalglance/Q-plan/_grounding/Q-RESEARCH-CREATIVITY-GROUNDING-MAP.md` | Present Q grounding document; not opened. |
| Q-10 | `workflow/benchmarks/research-creativity-b0v3-2026-07-14/report.md` | Present; explicitly cited as the valid-refutation receipt; not opened. |
| Q-9 | `docs/.source/KILOCODE-SPEC-OVERVIEW.md` | Present candidate local copy matching the explicitly named Kilo overview. |
| Q-9 | `docs/.source/HERMES-SPEC-OVERVIEW.md` | Present candidate local copy matching the explicitly named Hermes overview. |
| Q-11/Q-12 | `docs/.source/aide2-announcement.md` | Present candidate local AIDE² source. |
| Q-11/Q-12/Q-14 | `docs/.source/aideml-SUMMARY.md` | Present candidate digest; exact original-plan use must be confirmed. |
| Q-14/Q-15 | `docs/.source/evo-SUMMARY.md` | Present local Evo digest. |
| Q-15 | `docs/.source/evo/plugins/evo/src/evo/frontier_strategies.py` (`_pick_pareto_per_task`, summary cites line 251) | Present source implementation explicitly anchored by symbol/line. |
| Q-15 | `.claude/papers/2507.19457-GEPA-reflective-prompt-evolution.pdf` | Present local paper matching arXiv:2507.19457. |
| Q-15 | `docs/.source/nemo-rl-SUMMARY.md` | Present local NeMo-RL digest. |
| Q-15 | `docs/.source/nemo-gym-SUMMARY.md` | Present local NeMo-Gym digest. |
| Q-15 | `docs/.source/nemo-gym/nemo_gym/server_metadata.py` (summary cites line 25) | Present source for `verified: bool=False`. |
| Q-15 | `docs/.source/nemo-gym/nemo_gym/reward_profile.py` (summary cites line 310) | Present source for `compute_pass_majority_metrics`. |
| Q-3 | Anthropic SWE-bench-Pro `92% quality / 63% cost`, WebSearch-confirmed 2026-07-10 | Explicit external baseline; no exact local path supplied in the summary. |
| Q-5 | `bellard.org/ts_zip` | Explicit external grounding URL label; no exact local path supplied. |
| Q-6 | `sqlite.org/testing` | Explicit external grounding URL label; no exact local path supplied. |
| Q-7 | Carmack, June 2026 weak-model-navigable-code mechanism | Explicit external source description; no exact URL/path supplied. |
| Q-8 | `bellard.org/ts_server` | Explicit external grounding URL label; no exact local path supplied. |
| Q-8 | TextSynth dependency/license assessment | Explicit rejection basis; no exact source path supplied. |
| Engineering-taste cross-cutting | Bellard/Carmack/Hipp convergence claim | Explicit named grounding cluster; Hipp source is not otherwise localized in the summary. |

### Internal implementation/evidence anchors named by the summary

These are destinations or existing composed artifacts, not substitutes for the original Q plans.

- Present: `.elai_cc/crates/orchestrator/src/autonomous_gate.rs` (Q-4 code-read anchor).
- Present: `.elai_cc/crates/orchestrator/src/novelty.rs` (Q-11 runtime novelty path).
- Present: `.elai_cc/crates/orchestrator/src/ssb.rs` (Q-11 cost/event anchor).
- Present: `.elai_cc/crates/orchestrator/src/web_research_orchestrator.rs` (Q-12 prompt path).
- Present: `.elai_cc/crates/orchestrator/src/research_tree.rs` (Q-12/Q-15 selector path).
- Present: `.elai_cc/crates/orchestrator/src/briefing.rs` and `.elai_cc/crates/knowledge/src/cosmos_worker.rs` (Q-13 RECON alternatives).
- Present: `.elai_cc/crates/orchestrator/src/research/web_orchestrator/citation_verifier.rs` (E-48 reuse anchor).
- Present: `.elai_cc/crates/cli/src/measure.rs` (K-4 pending/set ceiling idiom candidate).
- Present: `.claude/gate-manifest.txt`, `.claude/tools/gate_freeze_check.py`, and `.claude/tools/cross_lang_diff.py` (Q-6/Q-9 frozen checks).
- Present: `.claude/tools/frontier_strategy_bench.py` and `.claude/tools/conjecture_triage.py` (Q-10/Q-11/Q-12/Q-14/Q-15 bench/harness).
- Present: `.claude/skills/improvement-conjecture/SKILL.md` (Q-14 E-3).
- Present: `.claude/rules/handoff-protocol.md` and `.claude/tools/handoff_state` (Q-15 S-6).
- Present: `.elai_cc/wire-dark-allowlist.toml` (Q-12 R9 experiment registration).
- Present: `.claude/skills/paper-to-plan` and `.claude/skills/idea-intake`; Q-10 explicitly forbids adopting the refuted packet into them.

### Missing, extra, or unresolved deeper-source candidates

- **Present but absent from the summary:** `docs/thefinalglance/Q-plan/Q-16_elai_on_elai_e2e_functional_suite.md`. It is the only numbered Q original found beyond the claimed Q-1..Q-15 index. This discovery deliberately does not infer its features.
- **Referenced symbolically, exact file not found at the expected `.claude/tools` name:** `N-2 corpus_composition_check` / `corpus_composition_check.py`. The dependency may live under another name or be plan-only; deep recon is required.
- **Not localized by the summary:** exact Bellard, SQLite, Carmack, Anthropic, Hipp, and TextSynth receipts/digests.
- **Not named by the summary:** the exact two Q-11 control-arm names, Q-12 second benchmark, Q-12 bandit formula/kill threshold, and Q-15 bench catalog file.
- **Chronology conflict requiring deeper read:** Q-11 and Q-14 both describe a `conjecture_triage.py` novelty canary, while Q-14 says the harness sibling did not yet exist.

## Coverage conclusion

The indexed Q design is fully represented here at summary granularity: every Q-1..Q-15 entry, every named sub-unit, observable event/artifact, CLI/env surface, fail-closed negative, dependency, status gate, rejection, and deferral in the 148-line index has an explicit atom or status entry. The one known coverage hole is intentional and visible: `Q-16_elai_on_elai_e2e_functional_suite.md` exists but is not indexed by `Q-SUMMARY_INDEX.md`, so its features require a separate original-plan discovery pass.
