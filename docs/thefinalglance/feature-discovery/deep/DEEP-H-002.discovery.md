# DEEP-H-002 exhaustive feature discovery

## Scope, integrity, and method

This pass processed exactly the nine files assigned to manifest chunk `DEEP-H-002`. Numbered plans and auxiliary grounding maps are treated equally as evidence. `docs/thefinalglance/feature-discovery/08-H-SUMMARY_INDEX.discovery.md` was used only as the comparison/orientation baseline; it is not evidence for any source-ledger atom below. No source assigned to another deep chunk was opened.

Classification used below: `E-STATUS`, `E-STATE`, `E-REQ`, `E-SCHEMA`, `E-EVENT`, `E-TEST`, `E-EVIDENCE`, `E-LIMIT`, and `E-DEFER` are explicit source claims; `AMBIGUOUS` and `CONFLICT` preserve uncertainty or tension in the originals; `INFERRED` is synthesis that is not itself stated as a requirement. Every evidence atom has one unique, file-scoped ID and an exact source line anchor. Repeated statements within a file are folded into one atom with all reinforcing anchors rather than double-counted.

### Exact processed manifest and verification

| # | Class | Lines | Bytes | SHA-256 | Exact absolute path | Verification |
|---:|---|---:|---:|---|---|---|
| 1 | auxiliary/grounding/fixture | 153 | 14781 | `5383df80092bda52557751d59d5cc776cc4c80fa3fdd5d13099e102c119f826d` | `docs/thefinalglance/H-plan/_grounding/H-REASONING-GROUNDING-MAP.md` | exact |
| 2 | auxiliary/grounding/fixture | 293 | 29136 | `21efdffcf035fcb726f7fbf02e01acd52c5f8a1c00a5684ea31a568d77b00f55` | `docs/thefinalglance/H-plan/_grounding/H-TRUST-GROUNDING-MAP.md` | exact |
| 3 | numbered/index original | 196 | 16488 | `e774afc9d58af08ab3054b2d65856bfeeeca4e7f76d02fc1406c65ab2267d83e` | `docs/thefinalglance/H-plan/H-3_output_token_diff_fim_streaming.md` | exact |
| 4 | numbered/index original | 176 | 14255 | `4cec8e1da48bbeb639c1c6083e97e715ca5e036ac5ea9eccc2deae65d035a5c7` | `docs/thefinalglance/H-plan/H-5_batch_api_noninteractive_lanes.md` | exact |
| 5 | numbered/index original | 78 | 15246 | `51762414d3124c598f66b41f84fd00ad8557029442b36c5c11bb300f6bfd225c` | `docs/thefinalglance/H-plan/H-38_cost_confirm_taskstop_podstop.md` | exact |
| 6 | numbered/index original | 164 | 13197 | `1dfc508ba2d62734382c18e7d17ccd5afddbcea866a80704a6ec5d47dd7af21a` | `docs/thefinalglance/H-plan/H-40_decorrelation_keyed_verifier_retention.md` | exact |
| 7 | numbered/index original | 182 | 15299 | `d9efb3fd8ab447ca27ffac1f7cca46eddd9a9d34a20f84a2c3b7be9b9be92c56` | `docs/thefinalglance/H-plan/H-41_checker_coverage_gap_instrument.md` | exact |
| 8 | numbered/index original | 186 | 16397 | `45a66e3e2dc45b9ae92162fa5072733b265852edf52ea3ad6d1a6468e3f9f976` | `docs/thefinalglance/H-plan/H-45_worstcase_lineage_promotion_scoring.md` | exact |
| 9 | numbered/index original | 56 | 18430 | `1ac48ae75e5c5cea09742e9897c5dface12ce5b060507a332c11a2e0ac87ab93` | `docs/thefinalglance/H-plan/H-54_obligation_closure_escape_hatch_gate.md` | exact |
| **Total** |  | **1484** | **153229** |  | **9 files** | **all manifest values exact** |

Verification was performed with `wc -l -c` and `sha256sum` on the nine exact absolute paths. Manifest totals and all 27 per-file integrity values (line count, byte count, hash) match.

## Per-file evidence ledgers

### H-REASONING-GROUNDING-MAP.md — complete ledger

- **HRG-001** `[E-EVIDENCE]` The map is the 2026-06-04 W-7, grep-confirmed live-symbol source for H-39..H-42; executors must read their H-N slice, citations, and RD deltas before coding. (lines 1-7)
- **HRG-002** `[E-EVIDENCE]` The motivating Hallucination Stations claim is a per-forward-pass `O(N^2*d)` ceiling, peer-verifier pessimism, and “CoT buys nothing”; the map records the CoT conclusion as refuted. (lines 9-15)
- **HRG-003** `[E-EVIDENCE]` Polynomial CoT is cited as exactly PTIME, `T` CoT steps can simulate size-`T` circuits, and gradient descent can learn NC1 with CoT; the claimed bound applies to one pass (approximately TC0), not the autoregressive loop. (lines 13-16)
- **HRG-004** `[E-EVIDENCE]` Hallucination inevitability is retained but described as over-determined; common low-complexity factual hallucination is tied to evaluation incentives rewarding confident guessing over abstention. (lines 16-18)
- **HRG-005** `[E-EVIDENCE]` Verification pessimism is limited to generator-correlated verifiers; decorrelated weak ensembles and sound external checkers help, while Pass@K is the honest oracle ceiling. (lines 18-21)
- **HRG-006** `[E-STATUS]` D-10/D-11/D-16/D-17 are declared shipped and correctly aligned; Track 6 is four activation/extension refinements and instruments, not a redesign. (lines 21-24)
- **HRG-007** `[E-STATE]` `select_grounding_mode(claim, ctx)` is shipped and called from `agent_pool.rs:2030`; Rule 5 orders Constructive, then FalsifiabilityCrossFamily, then ConformalRefusal for Factual/Computational claims. (lines 29-34)
- **HRG-008** `[E-STATE]` Grounding Rule 1 routes Auditor/Meta to `ConservativeFallback`, providing an auditor-never-grades-itself firewall; `GroundingMode` also includes `OperationalDeferred`. (lines 30-34)
- **HRG-009** `[E-STATE]` D-11's `CheckerResult`, emitter, recent-result read, CLI/dashboard row, and constructive-checker lane are shipped. (lines 35-37)
- **HRG-010** `[E-STATE]` D-21's fixture benchmark already computes per-mode F1 and a 5x5 confusion matrix with a missing-class penalty; it is offline accuracy evidence, not live production coverage. (lines 38-40)
- **HRG-011** `[E-STATE]` D-16's persisted conformal-refusal substrate is shipped: `RefusalDecision {Answer, Abstain}`, `ConformalRefusalRow`, `TauUpdater`, persisted tau, decision/scoring, emitter, and CLI. (lines 41-43)
- **HRG-012** `[E-STATE]` D-17's refusal SLA is shipped with `refusal_rate`, `band_lo`, and `band_hi` for confabulating-versus-overcautious behavior. (lines 44-44)
- **HRG-013** `[E-STATE]` The self-mod promotion loop is shipped through `two_gate_promote`, `config_archive`, BAI sampling, `darwin_admit_loop`, and security sensitivity checks; its reward lacks an abstention term. (lines 45-48)
- **HRG-014** `[E-STATE]` Coding metrics (`pass_at_k`, `pass_at_1`, `record_pass_sample`) are shipped but represent only pass/total, with neither abstention credit nor confident-wrong penalty. (lines 49-50)
- **HRG-015** `[E-STATE]` SAFEDELTA/SecCodePlt provide the frozen-runner plus measured-SSB-row pattern that `AbstentionScored` should mirror. (lines 51-53)
- **HRG-016** `[E-STATE]` E-6 `IsoBudgetReport`, `insufficient_lift`, counts, its measured row, and dashboard are shipped; there is no unique-catch/error-decorrelation dimension. (lines 54-56)
- **HRG-017** `[E-STATE]` GRC `required_steps` is configured statically per role; no serial-depth scaling exists. (lines 57-58)
- **HRG-018** `[E-STATE]` E-19's L1 certainty, L2 micro-reflection, JSON parsing/emission, sidecar adapter, and budget truncation are shipped as the substrate H-42 would scale. (lines 59-61)
- **HRG-019** `[E-STATE]` D-7's cross-domain `CanaryFamily` and runner are shipped, and grounding Rule 3 already refuses when a canary blocks. (lines 62-63)
- **HRG-020** `[E-STATE]` D-3's `VerifierKind`, available-verifier context, and cross-family-verifier predicate are shipped as H-40's verifier set. (lines 64-65)
- **HRG-021** `[E-STATE]` H-39's genuine gap is optimized reward that treats correct abstention no better—or worse—than confident wrong across promotion, BAI, coding metrics, and bench scoring despite a shipped inference abstention signal. (lines 70-74)
- **HRG-022** `[E-STATE]` H-40's genuine gap is lift-only role pruning that can discard a low-accuracy verifier with conditionally decorrelated errors; the missing field is conditional error-decorrelation/unique catch on `IsoBudgetReport`. (lines 75-78)
- **HRG-023** `[E-STATE]` H-41's genuine gap is a live Constructive/Mode-A fraction plus a ranked checker-gap backlog identifying classes that fell to cross-family verification or refusal specifically for lack of a constructive checker. (lines 79-84)
- **HRG-024** `[E-STATE]` H-42's genuine gap is serial-depth-scaled GRC budgeting and a guard against treating context-window width as a remedy for serial-depth failure. (lines 85-89)
- **HRG-025** `[E-REQ]` RD-1 requires every reward optimized by promotion, BAI, GEPA, or bench scorers to credit correct abstention and penalize confident wrong strictly more; the abstention bit must be read from `RefusalDecision`, never re-derived. (lines 94-98)
- **HRG-026** `[E-REQ]` RD-2 retains/prunes a verifier by conditional error-decorrelation/unique catches rather than standalone accuracy; an ensemble may approach but must never claim to exceed the Pass@K oracle or “catch everything.” (lines 99-102)
- **HRG-027** `[E-REQ]` RD-3 forbids LLM grading where a sound constructive checker exists and makes journal-recomputable live Mode-A coverage plus the checker-gap backlog the investment signal. (lines 103-106)
- **HRG-028** `[E-REQ]` RD-4 requires reasoning budget to scale with serial depth, never context-window size; access and compute depth are distinct, and a context lift cannot be accepted as the fix for a depth-class failure. (lines 107-110)
- **HRG-029** `[E-EVIDENCE]` The map names and routes the supporting papers/interfaces for the single-pass/CoT result: arXiv 2507.07505, 2310.07923, 2402.12875, 2511.07378, 2503.01544, and 2502.02393. (lines 113-123)
- **HRG-030** `[E-EVIDENCE]` The abstention incentive anchor is arXiv 2509.04664; the decorrelation anchor is Weaver 2506.18203; the external-checker anchor is 2402.08115. (lines 124-130)
- **HRG-031** `[E-EVIDENCE]` The inevitability and statistically-negligible bounds are anchored to 2401.11817 and 2502.12187, including a GD-4 false-positive-bounding interpretation. (lines 131-134)
- **HRG-032** `[E-SCHEMA]` At the map snapshot, `SCHEMA_VERSION=220` and `TOTAL_VARIANTS=497`; 207/484, 191/464, and the approximate e45 pin 482 are stale and must not be used as an edit baseline. (lines 137-142)
- **HRG-033** `[E-SCHEMA]` H-39 adds one Pattern-2 `AbstentionScored` variant. (lines 143-144)
- **HRG-034** `[E-SCHEMA]` H-40 adds no action type; it extends `IsoBudgetReport` and reuses `iso_budget_measured`, with at most a payload/schema touch and zero variant delta. (lines 145-146)
- **HRG-035** `[E-SCHEMA]` H-41 adds one Pattern-2 `CheckerGapDetected`; its Mode-A ratio is a roll-up over existing rows and gets no variant. (lines 147-149)
- **HRG-036** `[E-SCHEMA]` H-42 adds one Pattern-2 `ReasoningBudgetScaled`; all four Track-6 plans together would add three variants, applied ship-order-cumulatively from the live constants. (lines 150-151)

### H-TRUST-GROUNDING-MAP.md — complete ledger

- **HTG-001** `[E-EVIDENCE]` This is the 2026-06-04 W-7 grounding source for H-17..H-29, distilled from a 15-agent trust design, live `elai.map` greps, and F/G/D/C closure maps; listed line/status triples are to be reused verbatim rather than invented. (lines 1-6)
- **HTG-002** `[E-REQ]` Track 3's North Star is deterministic trust in both LLM-written code and autonomously discovered trading strategies, enforced by rules predicates, gates, and off-agent runtime artifacts rather than markdown instructions. (lines 8-14)
- **HTG-003** `[E-REQ]` The threat model includes selection bias from the search process itself under cumulative trial count `N`, even with an honest LLM. (lines 10-14)
- **HTG-004** `[E-SCHEMA]` Snapshot constants are `SCHEMA_VERSION=207` and `TOTAL_VARIANTS=484`; 191/464 and 205/482 are stale, and the first Track-3 bump must also repair the latent e45 pin at 482. (lines 16-22)
- **HTG-005** `[E-SCHEMA]` Each new Track-3 action type increments both live constants and all lock sites; Pattern-2 single-variant-with-`Kind` and reuse of existing payloads are preferred, and Wave 3 implementers must re-read live values. (lines 23-26)
- **HTG-006** `[E-STATE]` The SSB substrate is a shipped single-writer append-only WAL with no update/delete methods, a debug update-hook panic, and Pattern-2/9-tuple golden locking; H-17 adds a table and choke point rather than a persistence engine. (lines 28-37)
- **HTG-007** `[E-STATE]` The shipped rules engine includes `AstPattern`, `GrepPattern`, `SchemaRequired`, `MetricGate`, `Composite`, compiled predicates, `Allow/Warn/Block/Escalate`, cheapest-first short-circuiting on Block, lifecycle states, dashboard mirrors, and a tested immutable path. (lines 39-49)
- **HTG-008** `[E-REQ]` Track-3 finance gates are data/predicates plus frozen sidecars where math requires them, not a second rule engine. (lines 47-49)
- **HTG-009** `[E-STATE]` Provenance, capability gating, and commitment-aware taint helpers are shipped, but live provenance emission is wire-dark, `taint_text` is replay-only, and G-1 live `taint_flow.rs` is absent. (lines 51-60)
- **HTG-010** `[E-REQ]` H-20 must either depend honestly on G-1 live taint propagation or build a finance-scoped DKG dataflow pass; it composes with the existing taint primitive. (lines 57-60)
- **HTG-011** `[E-STATE]` Mutation and metamorphic runners, their base relations, violation events, and mutation validation tier are shipped; H-22 adds named trading relations and H-27 recursively applies the runners to gate logic. (lines 62-68)
- **HTG-012** `[E-STATE]` Determinism/RRA/replay are shipped despite an older G-summary absence claim: `DriftComparator`, seed/fingerprint configuration, RRA emission/view/replay, `--emit-rra`, and `replay_resume` exist. (lines 70-77)
- **HTG-013** `[E-REQ]` H-23 and H-25 must compose with RRA/determinism rather than reimplement either. (lines 75-77)
- **HTG-014** `[E-STATE]` F-4's anti-fabrication test-integrity leg is shipped: stdout hashes reach `emit_test_outcome`, missing hashes invalidate passing evidence, and the dashboard/confidence consumers expose the signal. (lines 79-85)
- **HTG-015** `[E-REQ]` H-25 extends integrity from “test passed” to an empirical result number using a new result manifest and re-derivation, without restating stdout-hash logic. (lines 82-85)
- **HTG-016** `[E-STATE]` DGM promotion, trap tasks, frozen keys, security sensitivity, locked-key observability, and archive verification are shipped; H-28 adds the reward-function-input tamper leg that intact archive hashes do not detect. (lines 87-95)
- **HTG-017** `[E-STATE]` SecCodePlt/SAFEDELTA measured-runner patterns, model fingerprint drift, and canary families are shipped; H-29 composes these signals into an interlock rather than adding a detector. (lines 97-102)
- **HTG-018** `[E-STATE]` H-17 is net-new: `register_trial` and `strategy_trials` have zero hits. (lines 104-106)
- **HTG-019** `[E-STATE]` H-18 is net-new: DSR, PBO, and MinBTL have zero hits and require frozen NumPy/SciPy math orchestrated by a Rust metric gate. (lines 107-108)
- **HTG-020** `[E-STATE]` H-19's reusable-holdout/Thresholdout budget is absent. (line 109)
- **HTG-021** `[E-STATE]` H-20's train/test-sink reachability predicate is absent even though its taint primitive exists. (lines 110-111)
- **HTG-022** `[E-STATE]` H-21's future-truncation perturb-and-diff runtime gate is absent. (lines 112-113)
- **HTG-023** `[E-STATE]` H-22's future-shuffle, time-reversal, and zero-drift relations are absent from the shipped base relation library. (lines 114-115)
- **HTG-024** `[E-STATE]` H-23's declared-metric output re-run digest and seed-coverage AST gate are absent despite shipped RRA. (lines 116-117)
- **HTG-025** `[E-STATE]` H-24's mode-divergent-branch AST ban and replay-equals-golden gate are absent. (lines 118-119)
- **HTG-026** `[E-STATE]` H-25's empirical result manifest and re-parser are absent. (lines 120-121)
- **HTG-027** `[E-STATE]` H-26's seeded shadow executor, financial-invariant pack, and naked-float-equality ban are absent. (lines 122-123)
- **HTG-028** `[E-STATE]` H-27's recursive gate validator, mandatory `null_band_sidedness`, and symmetric-comparator ban are absent. (lines 124-125)
- **HTG-029** `[E-STATE]` H-28's persisted reward-tampering counter and intact-hash score-jump detector are absent. (lines 126-127)
- **HTG-030** `[E-STATE]` H-29's locked warning→drift→halt FSM, 90-day model TTL, resume/keep-live gate, and label-fed concept drift are absent. (lines 128-129)
- **HTG-031** `[E-REQ]` TD-1 requires significance/edge numbers to use an append-only, agent-unresettable `N` scoped by `(data_window, objective)`; zero matching rows hard-fail as `trial_count_missing`. (lines 131-137)
- **HTG-032** `[E-REQ]` TD-2 freezes the evaluator, backtest harness, metric code, gates, thresholds, and trial ledger as `LOCKED(mutable=0)` and unreadable/unmodifiable to self-mod agents through the named freeze mechanisms. (lines 138-141)
- **HTG-033** `[E-REQ]` TD-3 makes LLM/auditor judgment Tier-3 advisory only: it cannot promote and cannot override a deterministic Tier-1 block. (lines 142-144)
- **HTG-034** `[E-REQ]` TD-4 requires every proposed-to-active gate revision to kill 100% of a frozen out-of-band mutant corpus, satisfy all MRs including `null_band_sidedness`, reject all frozen traps, and persist an escaped-mutant count on any escape. (lines 145-148)
- **HTG-035** `[E-LIMIT]` TD-5 routes cross-hardware FP identity, pre-label concept drift, target-derived leakage, genuine-versus-lucky edge, and structural self-mod safety to human Layer 3/Tier 3; no mechanism may advertise guarantees over this residue. (lines 149-153)
- **HTG-036** `[E-REQ]` Tier 1 is the unconditional, fail-closed in-process rule decision plus off-agent row; missing trial/null/replay/leakage evidence fails. (lines 155-161)
- **HTG-037** `[E-REQ]` Tier 2 covers NumPy/SciPy math and live/paper trading runs, ships a deterministic fixture through the real path now, and uses a separately gated live checkbox without calling fail-open behavior fail-closed. (lines 162-164)
- **HTG-038** `[E-DEFER]` The live autonomous-trading lane is deferred behind the fixture harness, and no external trust claim is allowed before it. (lines 164-165)
- **HTG-039** `[E-SCHEMA]` H-17's new row schema is `{objective, data_window, config_hash, seed, ts}` with one write choke point, no agent reset/update/delete, ONC effective-N clustering, and a frozen table/function; H-18 only reads `N`. (lines 169-175)
- **HTG-040** `[E-REQ]` H-18 is a locked composite requiring DSR, PBO, MinBTL, and present `N`; frozen Python computes, Tier-1 Rust enforces, and no `N` fails closed. (lines 177-184)
- **HTG-041** `[E-REQ]` H-19 persists a locked Thresholdout/Ladder OOS query counter and refuses after exhaustion until a fresh holdout is provisioned. (lines 185-189)
- **HTG-042** `[E-REQ]` H-20 adds split-boundary schema, train/test reachability, and missing-`shift(1)` checks; it is fail-closed and permits only registered, never silent, suppression. (lines 191-199)
- **HTG-043** `[E-REQ]` H-21 runs a frozen feature pipeline on full and future-truncated data and performs a Rust first-mismatch column diff. (lines 201-206)
- **HTG-044** `[E-REQ]` H-22 extends the shipped MR library with future-shuffle, time-reversal, and zero-drift nulls and reuses the existing runner/violation row and metric gate. (lines 208-214)
- **HTG-045** `[E-REQ]` H-23 re-invokes the real RRA entrypoint under recorded seed/environment, checks declared-metric digests, adds seed-coverage/schema pins, and limits equality claims to same environment/seed. (lines 216-222)
- **HTG-046** `[E-REQ]` H-24 bans divergent live/backtest branches and duplicate routers, declares one shared entrypoint with adapters, and requires replay/golden BLAKE3 equality; behavioral parity remains Tier 2. (lines 224-230)
- **HTG-047** `[E-SCHEMA]` H-25's RRA result evidence extends with `{value, captured_output_blake3, parser_expr, data_hash, n_trials, dsr_pvalue, pbo}`, forbids metric harness import/redefinition/monkeypatching, and adds schema plus Tier-1 re-derivation. (lines 232-238)
- **HTG-048** `[E-REQ]` H-26 adds versioned put-call-parity, strike-monotonicity, and P&L-additivity relations; mined invariants start Proposed and must kill a seeded mutant; divergence uses frozen ULP/relative bounds and naked float `==` is banned. (lines 240-246)
- **HTG-049** `[E-REQ]` H-27 hooks gate self-validation into Shadow→Active, runs mutation/metamorphic/trap checks on the gate itself, and makes sidedness non-defaultable with an AST ban on symmetric code for one-sided declarations. (lines 248-255)
- **HTG-050** `[E-REQ]` H-28 sits before `two_gate_promote`, hard-blocks evaluator hash mismatch, flags/blocks implausible score jumps with intact hash, never lets a Tier-3 green override Tier-1, and leaves structural safety human-gated. (lines 257-263)
- **HTG-051** `[E-REQ]` H-29 composes fingerprint, PSI, canary, schema, live/backtest deviation, and 90-day TTL into a locked resume interlock; missing evidence halts, with the honest guarantee limited to halt within one realized-label cycle. (lines 265-271)
- **HTG-052** `[E-EVIDENCE]` The appendix routes selection-bias/DSR, reusable-holdout, leakage/static analysis, perturb-and-diff, discovery validation, and reproducibility references. (lines 273-281)
- **HTG-053** `[E-EVIDENCE]` The appendix routes mutation/metamorphic/oracle, reward-tampering/undecidability, drift, and runtime-verification references including QuantLib and Daikon. (lines 282-287)
- **HTG-054** `[E-REQ]` Before executing any arXiv-dependent Track-3 plan, the paper must be re-ingested/verified and effect-size claims must carry quote-plus-page anchors; two prior bad citations motivate the gate. (lines 288-291)

### H-3_output_token_diff_fim_streaming.md — complete ledger

- **H03-001** `[E-STATUS]` H-3 is planned Wave 1, in-process build/activation work, estimated at two sessions or 8–12 hours, with no network at gate time. (lines 1-3, 17)
- **H03-002** `[E-REQ]` Output tokens are accounted at 5× input and separately from caching; the plan targets diff/FIM output and early termination because caching cannot reduce output. (lines 5-6, 22-33, 138-139, 155)
- **H03-003** `[E-STATE]` Production DIFF is dark because `task_runner.rs:7871` always passes `editor_fs_override=None`, causing `UnsupportedMode` and `StructuredFallback`. (lines 6-8, 35-37)
- **H03-004** `[E-STATE]` FIM is wire-dark and requires all of `ELAI_FOCUS_TUNNEL_EXECUTE=1`, `mode==Fim`, `target_file`, and `edit_region`. (lines 38-40)
- **H03-005** `[E-STATE]` Streaming validation is dark on both cloud and sidecar because the shared loop's validator is hardcoded `None`; `on_chunk` has zero production callers. This explicitly corrects the older “sidecar-only” premise. (lines 8-14, 41-43)
- **H03-006** `[E-STATE]` `ModeSelector::select` is live each baby step, but Rule 1 forces Cloud→Structured; selection is live while diff execution is dark. (lines 44-45)
- **H03-007** `[E-STATE]` `try_edit_tiers` and `EditMatchTier {Exact, WhitespaceNormalized, Ellipsis, NotMatched}` are shipped through production `TOOL_EDIT`; `replay_hunks` is reachable only with an editor. (lines 40, 46-47)
- **H03-008** `[E-STATE]` `StreamingEarlyTerminated` has no production SSB producer; an early termination only logs `tracing::warn`. (line 48)
- **H03-009** `[E-EVIDENCE]` The plan records Aider unified-diff improvements and Diff-XYZ support for SEARCH/REPLACE on larger models, while rejecting the earlier “udiff for small models” claim. (lines 24-28, 174-180)
- **H03-010** `[E-LIMIT]` Aborted-stream dollar savings are unverified; only latency and downstream-token avoidance may be claimed until provider billing is confirmed. (lines 28-31, 109, 139, 188-189)
- **H03-011** `[E-REQ]` Constrained decoding/forced JSON is forbidden for coder and planner roles and reserved for operation-emission roles such as H-13 canvas ops. (lines 31-33, 107, 138, 193-194)
- **H03-012** `[E-REQ]` Add `Arc<dyn BufferEditor>` to `ResolvedConfig`/`BabyStepExecutionCtx`, pass `Some` at the production diff site, fire `replay_hunks`, and land changed-line output rather than a whole-file rewrite. (lines 62-65)
- **H03-013** `[E-REQ]` Relax Cloud→Structured Rule 1 so clear edit regions may select DIFF, while retaining Structured fallback for large refactors and new files. (lines 67-69)
- **H03-014** `[E-REQ]` Construct `Some(StreamingValidator)` at the single runtime-builder decision point so `on_token` runs on both cloud and sidecar. (lines 71-74)
- **H03-015** `[E-REQ]` Call `on_chunk(accum)` roughly every 50 tokens to activate tree-sitter error debounce and deprecated-import scanning. (lines 71-74)
- **H03-016** `[E-EVENT]` On terminal streaming condition, stop decoding and emit a production `StreamingEarlyTerminated` SSB row instead of only warning. (lines 76-79, 114-120)
- **H03-017** `[E-SCHEMA]` Reuse an existing `StreamingEarlyTerminated` action type if present; otherwise add one Pattern-2 variant, increment live schema/variant counts by one, and repair e45. (lines 76-79, 153, 172)
- **H03-018** `[E-REQ]` `ELAI_FOCUS_TUNNEL_EXECUTE` cannot remain an unregistered default-off terminal state: make it default-on or add an experiment allowlist row with promotion plan and ON-path smoke; the streaming validator is default-on. (lines 81-83, 135-136, 146)
- **H03-019** `[E-REQ]` Create `workflow/reasoning/H-3-decisions.md` recording all causal findings, reuse decisions, billing caveat, and the no-force-JSON boundary. (lines 85-87)
- **H03-020** `[E-REQ]` Produced symbols are the threaded editor, relaxed mode rule, default validator, ~50-token `on_chunk` call, SSB producer, and conditional action type. (lines 89-95)
- **H03-021** `[E-REQ]` Consumed symbols are `run_diff_step_with_provider`, `replay_hunks`, `try_edit_tiers`, `StreamingValidator::{on_token,on_chunk}`, and `ModeSelector::select`; their production paths are task-runner diff execution and the shared runtime stream. (lines 96-104)
- **H03-022** `[E-REQ]` The B-20 matcher must be reused, no grammar miss may silently become free form, BLAKE3 is required, and production code may not add `unwrap`/`expect`. (lines 105-110)
- **H03-023** `[E-REQ]` `GenerationModeSelected` and `StreamingEarlyTerminated` payloads must be mirrored in dashboard TypeScript/CloudPanel with locked wire form and `cross_lang_diff.py` passing. (lines 111-113)
- **H03-024** `[E-TEST]` A real cloud edit with no override must emit DIFF/SEARCH-REPLACE, show materially fewer journaled output tokens than the whole-file baseline, and land the changed lines on disk. (lines 114-120, 133-141)
- **H03-025** `[E-TEST]` A real forced error condition must stop decoding mid-stream and land a `StreamingEarlyTerminated` row; missing a production row is failure. (lines 114-120, 133-137)
- **H03-026** `[E-TEST]` Deterministic checks require the wiring trigger, editor/validator/on-chunk/emitter greps, cross-language diff, and the decision document. (lines 122-131)
- **H03-027** `[E-TEST]` Contract coverage must prove cloud DIFF instead of fallback, relaxed mode selection, validator default/on-token/on-chunk execution, row plus decoding stop, reuse rather than a second matcher, and no coder grammar. (lines 157-162)
- **H03-028** `[E-TEST]` Completion also requires workspace build green, auditor pass for orchestrator/CLI/tools, and no new dependency; cargo green alone is insufficient. (lines 133-142, 144-155)
- **H03-029** `[E-REQ]` Planned file changes cover task runner, mode selector, runtime builder/client, dashboard pair, one contract, one decision doc, and conditional SSB/lock/e45 sites. (lines 164-172)
- **H03-030** `[E-REQ]` B-28 focus tunnel and B-20 edit matching are shipped prerequisites; H-3 is parallel-safe with H-5 and must coordinate with H-4 only on a shared dispatch edit. (lines 18-20)
- **H03-031** `[E-LIMIT]` Flexible matching is required because model line miscounts make diff edits fragile; the cited 9× error increase is the risk motivating three-tier replay. (lines 183-185)
- **H03-032** `[E-REQ]` Structured fallback rate must be measured because relaxing Rule 1 can misroute large/unclear changes. (lines 186-187)
- **H03-033** `[E-REQ]` H-9 surfaces H-3's input:output reduction, H-2 composes cache input savings with diff output savings, and H-13 owns admissible constrained operation emission. (lines 191-194)
- **H03-034** `[CONFLICT]` The plan repeatedly says `try_edit_tiers` is reused “via the STRUCTURED path” (lines 14-15, 99, 106, 140), but its risk section says the three-tier matcher “must be on the diff path, not just the STRUCTURED path” (lines 183-185). Whether DIFF directly invokes `try_edit_tiers` or only `replay_hunks` is unresolved.
- **H03-035** `[AMBIGUOUS]` Although FIM darkness is a named objective gap, the task list contains no discrete FIM implementation beyond promoting the shared focus-tunnel flag; the required FIM terminal behavior is not specified. (lines 4-15, 38-40, 60-87)

### H-5_batch_api_noninteractive_lanes.md — complete ledger

- **H05-001** `[E-STATUS]` H-5 is planned Wave 1 pure build work, estimated at one to two sessions/6–10 hours, with a mocked batch round trip at Tier 1 and separately gated live endpoint at Tier 2. (lines 1-3, 14-15)
- **H05-002** `[E-STATE]` No Anthropic Message Batches transport exists; all current LLM calls use synchronous `/v1/messages` at full price. (lines 5-12, 32-34)
- **H05-003** `[E-EVIDENCE]` The plan records 50% pricing, typical under-one-hour and maximum 24-hour processing, no billing for expired work, 29-day result retention, and a 100,000-request-or-256MB cap. (lines 5-7, 23-27)
- **H05-004** `[E-REQ]` Add `AnthropicBatchClient` with create `POST /v1/messages/batches`, poll `GET .../{id}` until `ended_at`, retrieve `.../{id}/results`, and cancel `.../{id}/cancel`, reusing auth/header/base-URL plumbing from `send_raw_request`. (lines 9-10, 55-58, 81-90)
- **H05-005** `[E-REQ]` Batch requests use `anthropic-version: 2023-06-01`, may carry tools/system/multi-turn/extended thinking/most betas, and must exclude streaming, Fast mode, `max_tokens:0`, Threads, and `research_preview`. (lines 23-27, 94-99, 142-147)
- **H05-006** `[E-REQ]` Pair per-request cache control with H-2's one-hour TTL; the plan cites shared-context hit rates of 30–98% and warns the five-minute TTL can expire before completion. (lines 25-29, 55-58, 169)
- **H05-007** `[E-STATE]` `CosmosWorker::tick` is shipped as a sequential per-artifact loop with live CLI and dashboard triggers and is the clearest batch-eligible lane. (lines 35-37)
- **H05-008** `[E-STATE]` `WebResearchOrchestrator::run_node_loop` is shipped and sequential; its `chunks(batch_size)` is web-page fetching, not LLM batching. (lines 38-40)
- **H05-009** `[E-STATE]` GEPA reflection is wire-dark/RunPod-pending and current bench/eval commands do not call a model. (lines 41-42)
- **H05-010** `[E-REQ]` Replace the due-artifact loop with one batch submission, poll to completion, then apply returned results. (lines 60-62)
- **H05-011** `[E-REQ]` Only independent, non-human-waited research calls batch; inherently sequential dependency chains stay synchronous. (lines 64-65)
- **H05-012** `[E-REQ]` `/task`, chat REPL, interactive editors, and any user-waited operation remain on synchronous streaming. (lines 10-12, 27-30, 94-97)
- **H05-013** `[E-EVENT]` Add one Pattern-2 `batch_lifecycle` action type with `Kind {Submitted, Completed, Expired}`. (lines 60-71)
- **H05-014** `[E-SCHEMA]` `BatchLifecyclePayload` is `{kind, batch_id, request_count, model, spend_cents_half: i64, task_id}` and must move through variant, string, parser, SQL check, action mapping, test payloads, and golden lock. (lines 67-71)
- **H05-015** `[E-SCHEMA]` The action is baseline+1 from the live value, not hardcoded 207/484; the first applicable H plan also repairs e45's stale pin. (lines 67-71, 98, 125, 139)
- **H05-016** `[E-REQ]` Submitted→Completed/Expired transitions carry request count and half-priced spend, and the result must remain recomputable from the journal under `ELAI_SEED`. (lines 60-62, 102-107)
- **H05-017** `[E-REQ]` RunPod-pending bench/GEPA lanes ship a deterministic mock fixture against the real batch code now and a separately gated live checkbox; fail-open Tier 2 cannot advertise fail-closed. (lines 10-12, 73-75)
- **H05-018** `[E-REQ]` Create `workflow/reasoning/H-5-decisions.md` documenting absence, exclusions, TTL pairing, lane choice, tiering, and the prohibition on human-waited batching. (lines 77-78)
- **H05-019** `[E-REQ]` Production reachability is `cosmos_tick_emit_loop` or the dashboard Cosmos refresh trigger → `tick` → batch client → lifecycle row; H-9 may consume the same payload. (lines 80-93, 130-140)
- **H05-020** `[E-REQ]` An optional batch-status tile may read lifecycle rows; omitting it requires justification because H-9 already surfaces spend. (lines 100-101, 134-136)
- **H05-021** `[E-TEST]` A real Cosmos refresh with at least two due artifacts and no override must produce one batch ID, Submitted then Completed rows, `request_count>=2`, half synchronous spend, refreshed artifacts, and no change to interactive `/task`. (lines 102-107, 120-128)
- **H05-022** `[E-TEST]` Deterministic checks require transport/Cosmos/SSB/golden/e45 greps, the real wiring trigger, and the decision file. (lines 109-118)
- **H05-023** `[E-TEST]` Contract coverage must validate endpoint/version/exclusions, one-vs-N submission, lifecycle payload, all three Kind round trips, one-hour TTL, and a real-code mock fixture. (lines 142-147)
- **H05-024** `[E-TEST]` Workspace checks and auditor passes cover providers, knowledge, orchestrator, and contracts; no dependency is added because existing HTTP plumbing is reused. (lines 120-128)
- **H05-025** `[E-REQ]` Planned files add the provider module and export, alter Cosmos/research/SSB/locks/e45, and add one contract plus one decision doc. (lines 149-157)
- **H05-026** `[E-REQ]` Out-of-order batch results must be correlated to artifacts by `custom_id`, never submission position. (line 170)
- **H05-027** `[E-LIMIT]` Worst-case 24-hour latency is acceptable only for autonomous background refresh; an explicitly requested refresh that the user waits on must remain synchronous. (lines 166-168)
- **H05-028** `[E-REQ]` B-42 Cosmos and shipped Anthropic HTTP plumbing are prerequisites; shared `anthropic.rs` edits require coordination with H-1..H-4. (lines 16-19)
- **H05-029** `[E-REQ]` H-9 shows the discounted lane in cost breakdown, H-2 is the cache prerequisite, and GEPA/evals become batch eligible only after their RunPod scaffold is live. (lines 172-174)
- **H05-030** `[AMBIGUOUS]` The dashboard refresh is named as a production route to batching (lines 92-93), but the risk section says a dashboard/user-requested refresh being waited on must stay synchronous and routing should key on the autonomous tick loop, not the button (lines 166-168). Trigger intent must therefore be carried explicitly.

### H-38_cost_confirm_taskstop_podstop.md — complete ledger

- **H38-001** `[E-STATUS]` H-38 is Track 4, Wave 4, small build work with moderate grounding; it depends on H-9, composes H-37 and the orchestration Tier-1 tripwire, and targets safety/steering. (lines 1-3)
- **H38-002** `[E-REQ]` A billable or unsaved stop must surface cost-so-far and unsaved progress and require explicit confirmation; a non-billable, fully saved stop is frictionless. (line 4)
- **H38-003** `[E-REQ]` The design composes the existing spend ledger, live `dispatch_stop`, installed `InteractiveGate`, and harness irreversible-op tripwire; it must not rebuild spend math, prompt I/O, or the tripwire. (lines 6-7)
- **H38-004** `[E-EVIDENCE]` The nearest Odysseus analogy is validate-before-act; the gate predicate is reimplemented in Rust against SSB rather than porting Python. (lines 6-7, 71)
- **H38-005** `[E-STATE]` `SpendAccumulator`, model pricing, cache accumulation, `CloudSpendSnapshot`, routing caps, and `cloud_spend_summary(now_ms)` are production-wired sources for recomputed day/month spend. (lines 9-15)
- **H38-006** `[E-STATE]` Existing send-path daily-cap enforcement is post-hoc, emits reused `EscalateToCloud(cap_reached)`, and degrades to skip if the viewer cannot open; it is not pre-spend or pre-stop confirmation. (line 15)
- **H38-007** `[E-STATE]` `dispatch_stop` is production-wired but currently only blocks non-IDLE/non-CONFIGURING states and never reads cost or pod state; no task-runner confirmation exists. (lines 17-19)
- **H38-008** `[E-STATE]` `InteractiveGate`, `GateDecision {Approve, Reject, Modify, Redirect}`, `CliInteractiveGate::present_gate`, and its `RuntimeWiring` installation are production-wired; the broader H-37 decision loop is wire-dark. (lines 21-23)
- **H38-009** `[E-STATE]` `pod_stop` and `pod_cost_status` are harness MCP tools, not ELAI Rust; the only ELAI occurrence is a corpus string. (lines 25-26)
- **H38-010** `[E-REQ]` The harness skill's existing default-no Tier-1 confirmation list is extended with billable `pod_stop`/`pod_terminate` and current cost, rather than reauthored in Rust. (lines 28-29, 41)
- **H38-011** `[E-SCHEMA]` At authoring, schema/variant values were 220/497 and no stop-confirmation action type existed; implementation must re-read live values. (lines 31-34)
- **H38-012** `[E-REQ]` Add pure synchronous `StopCostGate::evaluate_stop(StopCostSnapshot) -> StopGateVerdict` in `stop_cost_gate.rs`. (lines 36-38, 49-52)
- **H38-013** `[E-SCHEMA]` `StopCostSnapshot` is `{day_usd: f64, unsaved_artifacts: usize, pod_billable: bool}`; `StopGateVerdict` is `Pass | ConfirmRequired {cost_usd, unsaved_count, reason}`. (line 38)
- **H38-014** `[E-REQ]` Confirmation is required iff cost exceeds named `CONFIRM_COST_FLOOR_USD`, unsaved count is positive, or pod is billable; only cheap/saved/non-pod stops pass silently. (line 38)
- **H38-015** `[E-REQ]` At stop time, read spend freshly via `cloud_spend_summary`, unsaved count from the pending-write set, and pod billing from config; then call the existing gate. (lines 39, 47, 53)
- **H38-016** `[E-REQ]` Approve proceeds, Reject returns `HookFinding::Block` with `rule_id="stop_cost_confirm_required"`, and a headless/no-human context holds by default rather than destroying work. (lines 39, 53, 65)
- **H38-017** `[E-EVENT]` Emit one `StopConfirmationEvent` carrying recomputable cost, unsaved count, and outcome `ConfirmedProceed`, `RejectedHeld`, or `PassedNoFriction`. (lines 40, 56-58)
- **H38-018** `[E-SCHEMA]` `StopConfirmationPayload` fields are `cost_usd: f64`, `unsaved_count: u32`, and `ts_ms: i64`; the Pattern-2 variant must include round-trip, fuzz, lock, parser, SQL, and all action-mapping sites. (lines 56-58)
- **H38-019** `[E-SCHEMA]` The snapshot expected 220→221 and 497→498, but explicitly requires checking the live constants before applying a one-variant delta. (line 58)
- **H38-020** `[E-REQ]` The safety confirmation is unconditionally active; `ELAI_STOP_COST_CONFIRM_ENABLE=1` gates only the probationary SSB beacon. (line 59)
- **H38-021** `[E-REQ]` The default-off beacon requires a wire-dark experiment row with owner H-38, expiry 2026-09-04, promotion `H-38-promote`, and an ON-path billable-stop smoke producing `ConfirmedProceed`. (line 59)
- **H38-022** `[E-REQ]` `dispatch_stop` integration occurs after the B-19 non-idle block and only for a real idle/configuring task-end stop; `InteractiveGate` and viewer/db state are threaded into the dispatcher. (line 53)
- **H38-023** `[E-REQ]` CloudPanel reuses the existing CloudSpend read; Tier 1 adds no new dashboard command or modal. (lines 43, 54)
- **H38-024** `[E-LIMIT]` The gate adds zero per-turn model token cost: it reads one float/count only on stop and sends a human-facing CLI string that never enters a model prompt. (line 45)
- **H38-025** `[E-REQ]` Cost is never cached or stored as memory; it is recomputed at stop time from SSB, so the source defines it as non-stale. (line 47)
- **H38-026** `[E-DEFER]` RunPod billing-API reconciliation, multi-pod batch-stop confirmation, a dashboard modal, and automatic checkpoint-before-stop are outside Tier 1. (line 43)
- **H38-027** `[E-TEST]` A real billable/unsaved stop must show both values, require a decision, and emit a recomputable row; a nonbillable/saved stop must proceed without friction and emit `PassedNoFriction`. (lines 61-66)
- **H38-028** `[E-TEST]` Missing the off-agent row fails the wiring gate; cargo green is not acceptance. (line 66)
- **H38-029** `[E-REQ]` Stub bans include empty payloads, swallowed `ConfirmRequired`, unconditional Pass, magic-number cost floor, silent headless proceed, and model-prompt injection. (lines 45, 65)
- **H38-030** `[E-EVIDENCE]` The motivating external incident is reporter-measured and must be reverified for exact ~$30/~1-hour values; ELAI's own gate value comes only from the journal. (lines 68-70)
- **H38-031** `[E-REQ]` H-38 complements, rather than replaces, F-9's post-hoc daily cap; both consume the same ledger. (lines 74-78)
- **H38-032** `[AMBIGUOUS]` The plan requires `PassedNoFriction`/other rows in the done-gate while also making the beacon default-off during probation. The trigger presumably enables the experiment, but the no-override/default behavior for telemetry acceptance is not stated. (lines 59, 66)
- **H38-033** `[CONFLICT]` A broad statement says a “$0 fully-saved” stop is frictionless (line 4), while the actual predicate also confirms whenever `pod_billable=true` even if current cost is zero (line 38). The precise rule is cheap/saved/**non-billable**.

### H-40_decorrelation_keyed_verifier_retention.md — complete ledger

- **H40-001** `[E-STATUS]` H-40 is planned Track 6/Wave 6, one-session (5–7 hour), in-process extension work with no new dependency or sidecar. (lines 1-3, 15-16)
- **H40-002** `[E-STATE]` E-6 currently prunes/recommends deletion using only marginal `insufficient_lift` below roughly one sigma; the report, row, commands, and dashboard are shipped. (lines 5-13, 34-37)
- **H40-003** `[E-EVIDENCE]` Weaver motivates preserving a lower-accuracy verifier when its conditional errors differ; the source records 87.7% versus 86.7% as a citation to verify and Pass@K as the ceiling. (lines 23-32, 145-150)
- **H40-004** `[E-STATE]` The shipped role universe includes `VerifierKind`, available verifiers, cross-family checks, and per-level auditor dispatch; conditional error-decorrelation is absent. (lines 38-41)
- **H40-005** `[E-REQ]` Compute per-role `unique_catch_rate(R)` as the fraction of consensus failures that only R got right. (lines 51-56)
- **H40-006** `[E-REQ]` Compute `decorrelation_score(R) = 1 - mean pairwise error-correlation` against other roles, conditional on ground truth, using deterministic seed-stable ratio math. (lines 53-56)
- **H40-007** `[E-SCHEMA]` Extend `IsoBudgetReport` and its serialized payload with per-role `unique_catch_rate` and `decorrelation_score`. (lines 58-64)
- **H40-008** `[E-REQ]` Preserve `insufficient_lift` for compatibility but add `insufficient_contribution = low_lift AND low_unique_catch`; only the both-low result recommends pruning. (lines 58-60)
- **H40-009** `[E-REQ]` Both floors are locked; a low-lift/high-unique-catch verifier is retained and a low/low control is prunable. (lines 58-71)
- **H40-010** `[E-EVENT]` Reuse and extend the existing `iso_budget_measured` row; add no action type, and make the numbers recomputable from journaled per-role outcome vectors under `ELAI_SEED`. (lines 62-64)
- **H40-011** `[E-REQ]` Add unique-catch/decorrelation columns beside lift in the iso-budget panel and explain why a low-lift role was retained. (lines 66-67)
- **H40-012** `[E-REQ]` Create `H-40-decisions.md` covering the Weaver rationale, locked both-low predicate, Pass@K ceiling, and non-soundness residue. (lines 69-71)
- **H40-013** `[E-REQ]` The live deletion gate must consume `insufficient_contribution`, never `insufficient_lift` alone. (lines 73-88)
- **H40-014** `[E-TEST]` A frozen R/C fixture must emit high/low unique-catch values, retain R, prune C, and show both decisions in the dashboard. (lines 92-98)
- **H40-015** `[E-TEST]` Broken arm `--prune-on-lift-only` must prune both R and C, proving the added axis is decision-bearing. (lines 84-98, 108-116)
- **H40-016** `[E-TEST]` The feature is default-on for every live iso-budget run with kill-switch only, byte-identical under seed, production reachable, cross-language mirrored, and dashboard-smoked. (lines 108-130)
- **H40-017** `[E-SCHEMA]` There is zero variant delta; if payload evolution touches schema, constants move in lockstep with baseline+0 variants. (line 128)
- **H40-018** `[E-TEST]` The contract tests formula correctness, R retained, C pruned, broken arm, and identical reruns against a frozen per-role matrix. (lines 132-136)
- **H40-019** `[E-REQ]` Planned changes are the existing harness/payload, commands/Svelte, one contract fixture, and one decision document; a second harness is a P0 error. (lines 138-143)
- **H40-020** `[E-LIMIT]` Without ground-truth labels, “decorrelation” degrades to the weaker proxy “disagreement diversity” and must be labeled as such. (lines 152-154)
- **H40-021** `[E-REQ]` Before automatic pruning trusts the both-low predicate, tune locked floors and bound false-positive/over-retain rates on at least three real runs. (lines 155-156)
- **H40-022** `[E-LIMIT]` Decorrelation raises an ensemble floor but does not make a verifier sound; final trust still needs H-41/D-11 sound checking or D-16 refusal. (lines 157-162)
- **H40-023** `[E-REQ]` H-40 has no hard prerequisite, composes D-3 and auditor roles, is parallel-safe with H-39/H-41/H-42, and feeds the verdict space whose reward H-39 fixes. (lines 17-21, 160-162)

### H-41_checker_coverage_gap_instrument.md — complete ledger

- **H41-001** `[E-STATUS]` H-41 is planned Track 6/Wave 6, one-session (6–8 hour), in-process instrumentation over shipped D-10/D-11, with no hard prerequisite. (lines 1-3, 18-22)
- **H41-002** `[E-STATE]` D-10 already orders constructive, cross-family, then refusal and firewalls Auditor/Meta; H-41 measures this preference rather than adding it. (lines 5-16, 38-41)
- **H41-003** `[E-STATE]` D-11's checker row/emitter/read/dashboard and D-21's offline per-mode F1 fixture are shipped; live coverage and a gap backlog are absent. (lines 42-46)
- **H41-004** `[E-EVIDENCE]` Kambhampati is used to distinguish failing LLM self/peer critique from sound external verification; Xu Table 3 supplies NP-complete, combinatorial, and undecidable risk taxa. (lines 26-36, 160-167)
- **H41-005** `[E-REQ]` Compute `mode_a_fraction = #Constructive / #(Factual|Computational claims)` per session plus Mode-B, refusal, and deferred fractions. (lines 57-62)
- **H41-006** `[E-EVENT]` Coverage is a read-only roll-up over shipped grounding-mode and `CheckerResultEmitted` rows, deterministic under `ELAI_SEED`, with no ratio action type. (lines 59-62)
- **H41-007** `[E-EVENT]` At `agent_pool.rs:2030`, emit `CheckerGapDetected` only when a Factual/Computational claim falls to cross-family or refusal **because** no constructive registry checker matched. (lines 64-68)
- **H41-008** `[E-SCHEMA]` `CheckerGapDetected` carries `{claim_class, complexity_taxon, fell_to_mode}` as one Pattern-2 variant, baseline+1 from live schema; the first Track-6 bump repairs e45. (lines 64-68)
- **H41-009** `[E-SCHEMA]` The deterministic taxonomy is `{DecidableHasChecker, DecidableNoChecker, NpCompleteOrCombinatorial, Undecidable, Unknown}`. (lines 70-73)
- **H41-010** `[E-REQ]` NP-complete/combinatorial and undecidable gaps are high priority and semantically call for a sound solver or refusal, never an LLM grade. (lines 70-73)
- **H41-011** `[E-REQ]` `elai checker-gaps` or the dashboard aggregates rows by `frequency × taxon priority` to rank which checker/solver would buy the most Mode-A coverage. (lines 75-76)
- **H41-012** `[E-REQ]` Create `H-41-decisions.md` explaining sound-checker rationale, taxonomy, instrument-not-gate scope, and that coverage measures reach rather than checker correctness. (lines 78-81)
- **H41-013** `[E-REQ]` Produced symbols are `mode_a_coverage`, `classify_complexity_taxon`, and `CheckerGapDetected`; they consume the existing router, registry, and live agent-pool site. (lines 83-95)
- **H41-014** `[E-REQ]` Router and registry must not be reimplemented; BLAKE3 and no production `unwrap`/`expect` remain binding. (lines 96-100)
- **H41-015** `[E-TEST]` A covered factual claim must route Constructive with no gap row; an uncovered one emits the exact payload and makes coverage recomputable. (lines 104-110)
- **H41-016** `[E-TEST]` An uncovered hard claim is high-ranked, backlog-visible, and deterministic; `--ignore-checker-gaps` must misreport coverage as the load-bearing broken arm. (lines 96-110, 121-130)
- **H41-017** `[E-TEST]` The feature runs default-on with kill-switch only from live `agent_pool`, cross-language dashboard pairing, dashboard smoke, one new variant, and no network/RunPod tier. (lines 121-143)
- **H41-018** `[E-TEST]` Contract fixtures span covered, uncovered, NP-complete, and undecidable cases; tests hand-check the ratio and require identical ratio/backlog order across seeded runs. (lines 145-150)
- **H41-019** `[E-REQ]` Planned changes add `checker_coverage.rs`, modify `agent_pool`, SSB/locks/e45, CLI/dashboard, and add one contract fixture plus decision doc. (lines 152-158)
- **H41-020** `[E-LIMIT]` The taxonomy classifier is heuristic; it must be calibrated on labeled claims and return `Unknown` rather than guess. (lines 169-171)
- **H41-021** `[E-LIMIT]` Near-zero coverage with an almost-empty registry is an honest signal that checker investment is needed, not an instrument defect. (lines 172-173)
- **H41-022** `[E-LIMIT]` Coverage measures access to a sound checker, not the checker's correctness; checker bugs remain H-26/H-27 territory. (lines 78-81, 127-129)
- **H41-023** `[E-STATUS]` H-41 explicitly says it is an instrument, not a blocking gate; it does not itself block Mode-B fallback, and a forced-refusal gate for uncovered hard claims is follow-on work after measured justification. (lines 174-175)
- **H41-024** `[CONFLICT]` RD-3/task language says hard claims “MUST route to a sound solver or refuse, NEVER an LLM grade” (lines 70-73, 96-99, 125), but the risk section says H-41 does not block a Mode-B fallback and enforcement is follow-on (lines 174-175). The current deliverable is detection/prioritization, not guaranteed enforcement.

### H-45_worstcase_lineage_promotion_scoring.md — complete ledger

- **H45-001** `[E-STATUS]` H-45 is planned Track 7/Wave 7, one-session (6–8 hour), W-10-tiered work: Tier 1 scorer/row/wiring and Tier 2 live PRMBench sweep. (lines 1-3, 16-18)
- **H45-002** `[E-STATE]` `two_gate_promote` currently admits on candidate-own alpha minus incumbent against a carried-forward `DEFAULT_TAU` labeled unconfirmed. (lines 5-14, 38-45)
- **H45-003** `[E-EVIDENCE]` The plan uses Gao for proxy overoptimization, Coste for worst-case/variance penalization, Eisenstein for lineage over seed diversity, Co-rewarding for disagreement, and PRMBench for calibration. (lines 26-36, 164-170)
- **H45-004** `[E-STATE]` Promotion gate/tau/k_m, archive selection, BAI, and SAFEDELTA pattern are shipped; H-43 execution fitness, D-13 cross-family views, and H-39 abstention scoring are planned dependencies/compositions. (lines 38-45)
- **H45-005** `[E-REQ]` H-43 is a hard prerequisite because worst-casing byte-hash fitness is meaningless; H-45 sequences after H-43 and composes, but does not merge, H-39's abstention dimension. (lines 19-24)
- **H45-006** `[E-REQ]` `aggregate_promotion_score` consumes at least two pretraining-lineage-distinct verdicts and computes `min(verdicts) - kappa * variance(verdicts)` with locked `kappa`. (lines 55-63)
- **H45-007** `[E-SCHEMA]` `PromotionScore` exposes `{worst_case, variance, mean, disagreement}`. (lines 57-60)
- **H45-008** `[E-REQ]` Family distinctness is asserted; random/finetune seed diversity alone is insufficient when cross-family views are available. (lines 61-63)
- **H45-009** `[E-REQ]` Disagreement above a locked band blocks as `LineageDiverseDisagree`; a single-view candidate is penalized rather than auto-passed. (lines 65-67)
- **H45-010** `[E-REQ]` Feed the aggregate into the existing `two_gate_promote` margin without rewriting tau/k_m gate logic. (lines 69-71)
- **H45-011** `[E-REQ]` Replace unconfirmed tau with a locked, dated value calibrated on PRMBench-style labeled accept/reject at a chosen precision/recall point; agents cannot tune tau toward easier admission. (lines 73-76)
- **H45-012** `[E-EVENT]` Emit `PromotionScoreAggregated` with `{worst_case, variance, mean, disagreement, tau, calibrated_at, seed}` and `Kind {WorstCaseScored, VariancePenalized, LineageDiverseDisagree}`. (lines 78-80)
- **H45-013** `[E-SCHEMA]` The row is Pattern-2 baseline+1 from the then-live 224/501, with e45 reconciliation if first; live values must be re-read. (lines 78-80, 145)
- **H45-014** `[E-REQ]` Create `H-45-decisions.md` covering the proxy law, conservative ensemble, lineage distinction, disagreement block, calibrated tau, and finite-ensemble oracle residue. (lines 82-84)
- **H45-015** `[E-REQ]` Production reachability is `darwin_admit_loop::run_one_generation → two_gate_promote → aggregate_promotion_score → row`, behind the existing darwinian kill-switch/allowlist. (lines 86-97)
- **H45-016** `[E-REQ]` Candidate-own mean, seed-only ensembles where family views exist, unconfirmed default-on tau, and agent-tunable tau/kappa are forbidden. (lines 98-104)
- **H45-017** `[E-REQ]` Dashboard/trust drawer shows worst-case, disagreement, and calibrated tau, including a visible high-mean candidate blocked by the aggregate. (lines 105-107)
- **H45-018** `[E-TEST]` Frozen candidates M (higher mean/high variance) and S (lower mean/low variance) must yield `worst_case(S)>worst_case(M)`, admit S, block M, and journal both deterministically. (lines 108-114)
- **H45-019** `[E-TEST]` Broken arm `--score-own-mean` must flip the verdict and promote M. (lines 98-114, 124-133)
- **H45-020** `[E-TEST]` The aggregate is the default promotion input under the existing kill-switch, runs from live promotion, uses family diversity, cross-language pairing, dashboard smoke, and seed-identical recomputation. (lines 124-147)
- **H45-021** `[E-TEST]` Tier 1 is the fail-closed in-process aggregate plus locked calibrated tau; the actual PRMBench sweep remains a separately gated checkbox. (lines 16-18, 133, 146)
- **H45-022** `[E-TEST]` Contract tests cover the formula, M/S ordering, disagreement block, gate input, locked/date-marked tau, broken arm, and deterministic rerun against a frozen multi-family/calibration fixture. (lines 149-154)
- **H45-023** `[E-REQ]` Planned changes add `promotion_score.rs`, extend `promotion_gate`, add the row/locks/e45 and dashboard types, plus one contract and one decision doc. (lines 156-162)
- **H45-024** `[E-LIMIT]` Two family views can still be correlated; report disagreement and add a third family (for example D-2 PPM) where cheap, treating the aggregate as necessary but insufficient. (lines 172-175)
- **H45-025** `[E-LIMIT]` PRMBench labels are imperfect; tau is a defensible default, not proof, and must be recalibrated when the lane's model changes. (lines 176-177)
- **H45-026** `[E-REQ]` `kappa` is tuned only on the frozen calibration slice, never at runtime; over-block rate must be bounded on at least three runs before default-on. (lines 178-179)
- **H45-027** `[E-REQ]` H-46 changes which parent to expand, H-48 wires the bandit, D-2 can supply a third view, E-53 is the CSQ-Critical proof ceiling, and H-39 modifies the same reward orthogonally. (lines 181-184)
- **H45-028** `[AMBIGUOUS]` Tier 1 is said to ship a “locked calibrated value,” while the PRMBench calibration sweep is Tier 2 and separately gated. The source does not state whether the Tier-1 value comes from the frozen PRMBench-style fixture, a prior calibration artifact, or an already-completed sweep. (lines 16-18, 73-80, 133, 146, 154)

### H-54_obligation_closure_escape_hatch_gate.md — complete ledger

- **H54-001** `[E-STATUS]` H-54 is Track 11/Wave 11, medium build work with strong external grounding; it depends on unfinished E-53, composes H-44 and R5/R7/R8, and targets a deterministic cant-lie proof gate. (lines 1-3)
- **H54-002** `[E-EVIDENCE]` The external failure modes are a relocated `sorry` in a renamed helper and a false assumed “known result,” both invisible to a top-statement keyword scan. (lines 3, 5-9)
- **H54-003** `[E-EVIDENCE]` The cited corpus snapshot is commit `0647711`, 71 Lean files/44,857 lines, with 128 sorries confined to two labeled QuantumOptics files and 69 fully closed files. (lines 3, 8, 44-45)
- **H54-004** `[E-REQ]` Traverse the full transitive obligation/lemma DAG by BFS from a discharged goal and block an unregistered escape hatch anywhere in the closure. (lines 8-10, 20-25)
- **H54-005** `[E-SCHEMA]` Public API is `scan_obligation_closure(goal: &DischargedGoal, verifier: VerifierKind) -> ClosureScanResult`. (line 22)
- **H54-006** `[E-SCHEMA]` `ClosureScanResult` is `{kind: ClosureScanKind, escape_nodes: Vec<EscapeNode>, closure_size: u32}` with `ClosureScanKind {Clean, EscapeHatchAnywhere, RegisteredException}`. (line 22)
- **H54-007** `[E-SCHEMA]` `EscapeNode` is `{obligation_id: String, hatch: HatchKind, registered: bool}` and `HatchKind` includes `Sorry`, `Admit`, `Axiom`, `Assume`, `VerifierExternal`, and `KaniAssumeVacuous`. (line 22)
- **H54-008** `[E-REQ]` The per-tool map recognizes Lean `sorry/admit/axiom`; Verus `assume`, `external_body`, `external`, and admit; Kani vacuous assume/unconstrained stubs; Dafny `assume/{:axiom}`; and a Z3 asserted assumption that restates the goal. (lines 8-9, 22)
- **H54-009** `[E-LIMIT]` If a verifier cannot expose its closure, return typed `ClosureUnavailable`; label the Layer-2 scan unsound there and rely on the verifier/runtime machine verdict as Layer-4 backstop. (lines 22, 41, 47)
- **H54-010** `[E-SCHEMA]` `obligation-closure-allowlist.toml` rows carry `{obligation_id, owner, reason, expiry_date, evidence_pointer}`. (lines 10, 23)
- **H54-011** `[E-REQ]` A matching unexpired row yields recorded `RegisteredException` without blocking; missing/expired registration blocks, and a run cannot self-grant an exception because the file is freeze-checked. (lines 10, 23, 33)
- **H54-012** `[E-EVENT]` Emit one off-agent row on every scan, preferring a `ClosureEscapeHatch` Kind extension to H-44's `TrapBehaviorDetected` if available, otherwise one Pattern-2 `ObligationClosureScanned`. (lines 24, 34, 37-39)
- **H54-013** `[E-SCHEMA]` Standalone row kinds are `ClosureClean`, `EscapeHatchAnywhere`, and `RegisteredException`; payload is `{goal_id, verifier_kind, closure_size, escape_obligation_ids: Vec<String>, registered: bool}`. (lines 24, 37-38)
- **H54-014** `[E-REQ]` Compute a BLAKE3 closure fingerprint as anti-stale evidence, byte-recomputable under `ELAI_SEED`. (lines 24, 38, 42)
- **H54-015** `[E-REQ]` Run the scan as the cheapest first Layer-2 leg of each E-53 formal lane, short-circuiting before an expensive proof check when a hole is found. (lines 24-25, 30-32)
- **H54-016** `[E-REQ]` The feature is default-on with kill-switch `ELAI_CLOSURE_SCAN_DISABLE`, under the existing E-53 cfg gate rather than a new default-off enable flag. (line 25)
- **H54-017** `[E-DEFER]` H-44's ImpossibleBench, isomorphic re-verification, and CodeHalu legs remain its own non-formal scope; H-54 does not replace them. (lines 11, 26-27, 49-50)
- **H54-018** `[E-DEFER]` A unified cross-verifier DAG abstraction is Tier 2; Tier 1 uses per-verifier scanners behind one trait. (line 28)
- **H54-019** `[E-REQ]` Add `verifier/closure_scan.rs`, register it under E-53, reuse each verifier's existing obligation exposure after re-grepping live symbols, and do not reauthor proof parsers. (lines 30-32)
- **H54-020** `[E-REQ]` Demote H-44's substring scan to `KeywordFallback` advisory; keep it, but make closure scan the deterministic formal-lane core. (lines 10-11, 31-32, 49-50)
- **H54-021** `[E-REQ]` The verification drawer shows blocked escape node plus obligation ID and shows registered exception owner/expiry; any added dashboard signal must pass cross-language diff. (line 35)
- **H54-022** `[E-REQ]` Production roots are E-53 dispatch during Auditing/CSQ-Critical promotion and, if present, H-44 trap dispatch behind `cfg(feature="darwinian-evolve")`; test-only roots do not satisfy reachability. (lines 37-40)
- **H54-023** `[E-REQ]` No “proved” verdict is allowed with an unregistered transitive hatch; top-level-only scans, silent exceptions/skips, production unwrap/expect, non-BLAKE3 hashes, or soundness claims without DAG exposure are forbidden. (line 41)
- **H54-024** `[E-TEST]` A clean closure emits exactly one `ClosureClean` and proceeds. (line 42)
- **H54-025** `[E-TEST]` A top-level-clean proof with a renamed admitted helper emits `EscapeHatchAnywhere` naming the obligation and blocks. (line 42)
- **H54-026** `[E-TEST]` A declared open obligation with a valid row emits `RegisteredException` and does not block; the same unregistered hole blocks. (line 42)
- **H54-027** `[E-TEST]` Broken arm `--top-statement-only` must miss the relocated hole and thereby fail the gate. (line 42)
- **H54-028** `[E-TEST]` Missing off-agent evidence fails; the scan kind and closure fingerprint must be byte-identical under seed. (line 42)
- **H54-029** `[E-REQ]` Registered-exception and false-block rates must be bounded on at least three real verifier runs before default-on. (line 42)
- **H54-030** `[E-STATE]` H-44 keyword replacement and E-53 verifier files are plan-asserted and unfinished; executors must re-grep live symbols rather than treat their paths as shipped fact. (lines 13-18, 46)
- **H54-031** `[E-SCHEMA]` Snapshot schema is plan-asserted 224/501; live values and ship order govern any +1 variant, and the first Track-11 bump repairs e45 if still stale. (line 18)
- **H54-032** `[E-LIMIT]` H-54 steals the inexpensive closure scan, not routine Lean proof search; cited AlphaProof success/cost/model/version figures are medium, self-reported, unverified for registry identity, and never ELAI measurements. (lines 15, 44-45)
- **H54-033** `[E-LIMIT]` The scan proves only absence of this hidden-hole class, not conformance of the theorem statement to intent; wrong-statement proofs route to C-29 `IntentPair`/F-8 SCF. (line 47)
- **H54-034** `[E-LIMIT]` Formal proof remains a narrow CSQ-Critical ceiling; boolean-only verifiers and statement conformance remain explicit residue. (lines 47, 54)
- **H54-035** `[E-REQ]` H-43/Track-7 provide the off-agent-verifier doctrine, H-44 supplies behavioral non-formal checks, E-53 supplies verifier closures, and R5/R7/R8 supply authority/reachability/freeze semantics. (lines 49-54)
- **H54-036** `[AMBIGUOUS]` The API accepts a `DischargedGoal` and scans its exposed dependency closure, yet integration says the scan runs before the verifier's own discharge/run. The lifecycle point at which a usable DAG exists is not resolved. (lines 22, 25, 31-32)
- **H54-037** `[AMBIGUOUS]` The source requires a row for every scan but the preferred composed event names only `TrapBehaviorDetected::ClosureEscapeHatch`; it does not specify how `Clean` and `RegisteredException` are represented without the standalone three-kind variant. (lines 24, 34, 37-38, 42)
- **H54-038** `[AMBIGUOUS]` The named E-53 per-verifier DAG-exposure symbols are not live-confirmed in this source; they are explicitly plan-asserted and must be discovered at execution. (lines 13-18, 31, 39, 46)

## Cross-source interactions and lifecycle composition

- **IX-001** `[INFERRED]` H-3 splits output savings from H-2 input-cache savings and delegates visibility to H-9; H-13 is the explicit boundary where constrained operation emission becomes acceptable. Evidence: H03-002/H03-011/H03-033.
- **IX-002** `[INFERRED]` H-5 depends on H-2's one-hour cache setting for long-running shared-context batches and on H-9 for discounted-spend display; interactive-route intent is the batching safety discriminator. Evidence: H05-006/H05-012/H05-019/H05-027/H05-029.
- **IX-003** `[INFERRED]` H-38 forms one two-boundary stop policy: ELAI Rust gates task-end cloud spend/unsaved writes, while the orchestration skill gates billable pod lifecycle; both reuse rather than duplicate existing human confirmation. Evidence: H38-003/H38-008/H38-009/H38-010/H38-015/H38-016.
- **IX-004** `[INFERRED]` H-40 and H-41 solve different verifier problems: H-40 retains useful probabilistic roles by unique catch, while H-41 measures migration away from probabilistic grading toward sound checkers. Evidence: HRG-022/HRG-023/H40-022/H41-021/H41-023.
- **IX-005** `[INFERRED]` Track-6 composition is abstention reward (H-39) → decorrelation-aware role retention (H-40) → live sound-checker reach (H-41) → serial-depth budget (H-42), all on shipped substrate and with only three net new variants. Evidence: HRG-021..HRG-028 and HRG-033..HRG-036.
- **IX-006** `[INFERRED]` H-45 cannot be meaningful before H-43 replaces byte-derived fitness; it then aggregates H-43/D-13 lineage views, composes H-39 abstention semantics, and feeds existing promotion without replacing the gate. Evidence: H45-004..H45-010/H45-027.
- **IX-007** `[INFERRED]` H-54 is meaningful only where E-53 exposes a proof DAG; where it does, it precedes expensive checking and strengthens H-44, while non-formal candidates stay with H-44 behavioral checks. Evidence: H54-009/H54-015/H54-017..H54-020/H54-035.
- **IX-008** `[INFERRED]` Across the chunk, all trustworthy metrics/events are either directly journaled or recomputable from SSB: H-3 output/termination, H-5 lifecycle/spend, H-38 stop cost/outcome, H-40 role outcomes, H-41 routing/gaps, H-45 promotion views, and H-54 closure fingerprint/result. Evidence: H03-024/H03-025, H05-016/H05-021, H38-017/H38-027, H40-010/H40-014, H41-006/H41-015, H45-012/H45-018, H54-012..H54-014/H54-028.
- **IX-009** `[INFERRED]` Default posture varies intentionally: H-40/H-41 run default-on with kill switches, H-45 is default inside an existing darwinian kill-switch, H-54 is default-on with a disable switch, H-3 must promote/allowlist its legacy flag, H-5 Cosmos batching is default-on, and H-38 gates only telemetry while safety is unconditional. Evidence: H03-018, H05-021, H38-020/H38-021, H40-016, H41-017, H45-015/H45-020, H54-016.
- **IX-010** `[INFERRED]` Schema baselines 207/484, 220/497, and 224/501 are time-local snapshots, not competing current truths; every plan explicitly requires live re-read and ship-order-cumulative deltas. Evidence: HTG-004/HTG-005, HRG-032..HRG-036, H38-011/H38-019, H45-013, H54-031.
- **IX-011** `[INFERRED]` Failure posture is consistently fail-closed only where the plan claims enforcement: H-38 holds headless stops, H-45 blocks disagreement/single-view weakness, H-54 blocks unregistered holes, and Track-3 misses halt; H-41 is deliberately measurement-only and must not be described as already enforcing refusal. Evidence: H38-016, H45-009, H54-011/H54-023, HTG-031/HTG-036, H41-023/H41-024.
- **IX-012** `[INFERRED]` Broken arms are first-class acceptance evidence: lift-only pruning, ignored checker gaps, own-mean promotion, and top-statement-only scanning must each visibly invert/corrupt the intended outcome. Evidence: H40-015, H41-016, H45-019, H54-027.

## Summary-index deltas, additions, and corrections

The orientation summary was accurate at headline level for the seven numbered plans, but the originals add or qualify the following items. Each delta is stated once; it does not replace the source-ledger atoms.

- **DELTA-001** `[ADDITION]` H-3's FIM activation predicate is four-way (`ELAI_FOCUS_TUNNEL_EXECUTE`, mode, target file, edit region), and the original never assigns a discrete FIM task beyond flag promotion. Summary H3-002 says only “double-gated and dark.” Evidence: H03-004/H03-035.
- **DELTA-002** `[ADDITION]` H-3 specifies `on_chunk`'s concrete tree-sitter/deprecated-import checks, conditional action-type reuse, dashboard pairing, fallback-rate measurement, no new dependency, and the exact contract matrix. Evidence: H03-015/H03-017/H03-023/H03-027..H03-032.
- **DELTA-003** `[CORRECTION/CONFLICT]` The H-3 summary says reuse `try_edit_tiers` on Structured; the original also insists the mitigation reach the diff path. This unresolved direct-via-diff versus indirect-via-Structured distinction must not be normalized away. Evidence: H03-034.
- **DELTA-004** `[ADDITION]` H-5 supports create/retrieve/cancel, defines 24-hour expiry/no billing, 29-day retention, 100K/256MB caps, exact exclusions, and out-of-order correlation by `custom_id`. Evidence: H05-003..H05-006/H05-026.
- **DELTA-005** `[ADDITION]` H-5's lifecycle includes `Expired`, has an exact payload and Pattern-2 sites, and batches only independent research nodes. Evidence: H05-011/H05-013..H05-016.
- **DELTA-006** `[CORRECTION/AMBIGUITY]` The summary says a human-waited call never batches; the original exposes a dashboard trigger as reachable but later excludes a user-waited dashboard refresh. Trigger intent needs an explicit field/branch. Evidence: H05-030.
- **DELTA-007** `[ADDITION]` The trust grounding map corrects multiple substrate statuses that the summary cannot convey in one session entry: RRA, rule engine, WAL, mutation/metamorphic tiers, frozen keys, and measured runners are shipped; finance-specific gates are the absent layer. Evidence: HTG-006..HTG-030.
- **DELTA-008** `[ADDITION]` Track-3 doctrine scopes persisted `N` by `(data_window, objective)`, freezes evaluator and ledger from even self-mod read/mutation, makes LLM green incapable of promotion, and requires 100% frozen-mutant kill plus `null_band_sidedness`. Evidence: HTG-031..HTG-035.
- **DELTA-009** `[ADDITION]` H-38's exact pure types, named cost floor, after-B19 insertion point, pending-write count, headless hold, skill-doc boundary, no-prompt privacy rule, fresh-at-stop recomputation, and four deferred items are summary-silent. Evidence: H38-012..H38-016/H38-022..H38-026.
- **DELTA-010** `[CORRECTION]` “Zero-cost, fully-saved” is frictionless only if the pod is also non-billable; `pod_billable` independently triggers confirmation. Evidence: H38-033.
- **DELTA-011** `[ADDITION/AMBIGUITY]` H-38 gates only the event beacon behind `ELAI_STOP_COST_CONFIRM_ENABLE`; confirmation remains unconditional, yet the done-gate requires rows without spelling out flag handling. Evidence: H38-020/H38-021/H38-032.
- **DELTA-012** `[ADDITION]` H-40 defines both formulas, retains `insufficient_lift` for compatibility, adds locked floors, defaults on with kill-switch only, and degrades to explicitly weaker disagreement diversity when labels are unavailable. Evidence: H40-005..H40-010/H40-016/H40-020.
- **DELTA-013** `[ADDITION]` H-40 requires at least three real runs to calibrate false-positive/over-retain behavior before auto-prune, and its dashboard must explain retention. Evidence: H40-011/H40-021.
- **DELTA-014** `[ADDITION]` H-41 defines the exact denominator, all fallback fractions, the five-value taxonomy, conditional “because no checker matched” emission, heuristic `Unknown` fallback, and correct near-empty-registry behavior. Evidence: H41-005..H41-011/H41-020/H41-021.
- **DELTA-015** `[CORRECTION/CONFLICT]` The H-41 summary reads like a routing prohibition for hard classes, but the original calls H-41 an instrument and defers forced refusal/blocking. Evidence: H41-023/H41-024.
- **DELTA-016** `[ADDITION]` H-45 requires at least two pretraining families, penalizes single-view candidates, defines the aggregate/event payload and kinds, dates tau, and freezes runtime tuning. Evidence: H45-006..H45-013/H45-016.
- **DELTA-017** `[ADDITION]` H-45's PRMBench sweep is Tier 2 even though Tier 1 needs a calibrated tau, its two-view result remains necessary-not-sufficient, a third view is encouraged, and model changes require recalibration. Evidence: H45-021/H45-024/H45-025/H45-028.
- **DELTA-018** `[ADDITION]` H-54 specifies concrete public structs/enums, six hatch kinds plus per-verifier token maps, typed `ClosureUnavailable`, exact allowlist fields, payload, BLAKE3 fingerprint, and a row for clean scans. Evidence: H54-005..H54-014/H54-024.
- **DELTA-019** `[ADDITION]` H-54 explicitly says E-53/H-44 symbols are unfinished plan assertions, Tier 1 uses per-verifier scanners, and it steals closure scanning rather than proof search. Evidence: H54-018/H54-019/H54-030/H54-032.
- **DELTA-020** `[ADDITION]` H-54's honesty boundary includes wrong-statement proofs, boolean-only verifiers, medium/self-reported corpus metrics, and a false-block/exception rate bound on three real runs. Evidence: H54-009/H54-029/H54-032..H54-034.
- **DELTA-021** `[AMBIGUITY]` H-54 leaves unresolved both when a `DischargedGoal` closure is available relative to the “pre-verifier” scan and how a composed H-44 event represents clean/registered results. Evidence: H54-036/H54-037.
- **DELTA-022** `[ADDITION]` The reasoning map corrects the “CoT buys nothing” premise, names the shipped refusal/checker/iso-budget/GRC substrate, and records exactly three net Track-6 variants. Evidence: HRG-002..HRG-006/HRG-007..HRG-020/HRG-032..HRG-036.

## Explicit ambiguity/conflict register

- **AC-001** `H-3 matcher reach`: `try_edit_tiers` is described as Structured reuse and as required on DIFF; resolve direct call graph before coding. See H03-034.
- **AC-002** `H-3 FIM terminal scope`: FIM is a named gap without a discrete deliverable. See H03-035.
- **AC-003** `H-5 trigger intent`: dashboard reachability and “human waited stays synchronous” need an explicit autonomous/user-waited discriminator. See H05-030.
- **AC-004** `H-38 zero-cost rule`: actual predicate includes `pod_billable`, qualifying the premise. See H38-033.
- **AC-005** `H-38 beacon acceptance`: probationary default-off telemetry and mandatory row evidence need a specified ON-path trigger. See H38-032.
- **AC-006** `H-41 enforcement`: taxonomy doctrine says never LLM-grade while this slice explicitly does not block. See H41-024.
- **AC-007** `H-45 calibration provenance`: Tier-1 calibrated tau versus Tier-2 calibration sweep lacks an artifact provenance rule. See H45-028.
- **AC-008** `H-54 scan timing`: discharged-goal/DAG input versus pre-discharge invocation is unresolved. See H54-036.
- **AC-009** `H-54 composed event cardinality`: the preferred H-44 event arm does not obviously encode clean and registered rows required on every scan. See H54-037.
- **AC-010** `H-54 live anchors`: E-53/H-44 integration is plan-asserted, not verified shipped state. See H54-038.

No source-backed requirement was recast as inferred. No direct cross-source contradiction was found beyond the listed internal plan tensions; dated schema numbers are treated as snapshots, not conflicts.

## Zero-omission / zero-duplicate audit

| Source | Lines read | Ledger atoms | Section/line coverage |
|---|---:|---:|---|
| H-REASONING-GROUNDING-MAP.md | 153/153 | 36 | purpose 1-7; rationale 9-24; RS 27-65; RG 67-89; RD 91-110; citations 112-134; schema 136-153 |
| H-TRUST-GROUNDING-MAP.md | 293/293 | 54 | metadata/North Star 1-14; constants 16-26; shipped corrections 28-102; absent map 104-129; doctrine 131-153; tiering 155-165; all H-17..H-29 tables 167-271; appendix 273-293 |
| H-3_output_token_diff_fim_streaming.md | 196/196 | 35 | status/objective/context/state/reads/tasks/wiring/checks/acceptance/tests/files/citations/risks/next, lines 1-196 |
| H-5_batch_api_noninteractive_lanes.md | 176/176 | 30 | status/objective/context/state/reads/tasks/wiring/checks/acceptance/tests/files/citations/risks/next, lines 1-176 |
| H-38_cost_confirm_taskstop_podstop.md | 78/78 | 33 | premise/build/state/scope/integration/wiring/grounding/composition, lines 1-78 |
| H-40_decorrelation_keyed_verifier_retention.md | 164/164 | 23 | status/objective/context/state/reads/tasks/wiring/checks/acceptance/tests/files/citations/risks/next, lines 1-164 |
| H-41_checker_coverage_gap_instrument.md | 182/182 | 24 | status/objective/context/state/reads/tasks/wiring/checks/acceptance/tests/files/citations/risks/next, lines 1-182 |
| H-45_worstcase_lineage_promotion_scoring.md | 186/186 | 28 | status/objective/context/state/reads/tasks/wiring/checks/acceptance/tests/files/citations/risks/next, lines 1-186 |
| H-54_obligation_closure_escape_hatch_gate.md | 56/56 | 38 | premise/build/state/scope/integration/wiring/grounding/composition, lines 1-56 |
| **Total** | **1484/1484** | **301 evidence atoms** | **all assigned source sections covered** |

Duplicate audit: the 301 evidence IDs are unique by file prefix. Repeated source assertions (especially task/wiring/acceptance restatements) were consolidated into a single atom with multiple anchors. The 12 `IX-*`, 22 `DELTA-*`, and 10 `AC-*` IDs are synthesis/index records and are not included in the 301 source-atom total; they cross-reference evidence IDs instead of duplicating source facts. Total unique atomic/index IDs expected: **345**. A machine audit must confirm 345 unique IDs and zero duplicate IDs before handoff.

Scope audit: exactly **9 files / 1,484 logical lines / 153,229 bytes** were treated as original evidence. The only non-manifest content read was the exact DEEP-H-002 manifest section and the designated H summary discovery file for orientation/comparison.

Canonical output SHA-256 (this line omitted from hash input): `27b7811c944982300254e90b9dd3d1d82efcf3b7a284854d40f649ae9291d63f`
