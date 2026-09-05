H-TRUST-GROUNDING-MAP — distilled grounding reference for TRACK 3 (TRUST-HARDENING, H-17..H-29)
Source: D:\.REGRET_workflow\memory-bank\plans\elai-trust-hardening-design.md (the T-1..T-13 design, 15-agent
  grounded research run, authored 2026-06-04) + live elai.map greps run 2026-06-04 (rules/safety/orchestrator
  substrate) + the F/G/D/C summary closure maps. Companion W-7 source for Track 1/2: _grounding/H-GROUNDING-MAP.md.
This is the W-7 grounding source for H-17..H-29. Authors and the wiring validator re-grep against it.
file:line:status triples are verbatim from elai.map (2026-06-04). Do not invent line numbers.

WHY A SEPARATE TRACK-3 MAP: Track 1/2 ground against caching/canvas substrate; Track 3 grounds against the
TRUST substrate (rules engine, SSB WAL, taint, determinism/RRA, mutation/metamorphic, DGM guards, gate-freeze).
The North Star (design PART 0): when ELAI runs an AUTONOMOUS system that searches for / builds trading
strategies, the operator must TRUST both the LLM-WRITTEN code AND the AUTONOMOUSLY-DISCOVERED strategy — and
that trust must be enforced by DETERMINISTIC ELAI mechanisms (rules-engine predicates, gates, runtime verifiers
reading off-agent artifacts), NOT by .md rules an LLM may ignore. The threat is the SEARCH ITSELF (selection
bias under cumulative trial count N), not only a lying LLM (design PART 0, Bailey & Lopez de Prado).

## CANONICAL CONSTANTS (LIVE 2026-06-04; NEVER hard-code; re-read at edit time — W-9)

SCHEMA_VERSION = 207   (crates/orchestrator/src/ssb.rs:2909, `const SCHEMA_VERSION: i64 = 207`). AUTHORITATIVE.
TOTAL_VARIANTS = 484   (crates/orchestrator/tests/action_type_lock.rs:549, `const TOTAL_VARIANTS: usize = 484`).
  AUTHORITATIVE. Same snapshot the Track-1/2 H-GROUNDING-MAP records. STALE (do NOT use as baseline): CLAUDE.md
  header 191/464; F-plan note 205/482. LATENT: e45_cross_language_drift_contract.rs:294 still pins 482 — the
  first Track-3 H-N that bumps SCHEMA also bumps that line (same instruction as Track 1/2).
BUMP GUIDANCE: every H-N adding an SSB action_type bumps 207->207+N and 484->484+N, updating BOTH ssb.rs:2909
  and action_type_lock.rs:549 (+ALL_VARIANTS array). Prefer Pattern-2 single-variant-with-Kind to minimise bumps;
  prefer REUSING an existing payload (GD-1) over a new variant. Track-3 work runs in WAVE 3 (after Track 1/2 +
  G-plan), so re-read the live integer at edit — it WILL have moved past 207.

## TRUST-SUBSTRATE-SHIPPED CORRECTIONS (load-bearing — these flip BUILD-from-nothing into BUILD-on-shipped)

The design (and the older G-SUMMARY) treat several substrate pieces as PLANNED/ABSENT. The 2026-06-04 elai.map
greps show they have SHIPPED since those docs were written. Track-3 H-Ns BUILD finance/search-specific gates ON
this shipped substrate (GD-1 compose-not-restate); re-implementing any of it is a P0.

TS-1 (H-17/H-18/H-23/H-25/H-29). SSB is the untamperable append-only WAL keystone — SHIPPED. Single-writer,
  no UPDATE/DELETE methods, debug update_hook panic (phase-3B; ssb.rs). Pattern-2 single-variant-with-Kind +
  9-tuple golden lock (action_type_lock.rs). The trial ledger T-1 is a NEW append-only table + a single
  register_trial() choke-point ON this substrate, NOT a new persistence engine.

TS-2 (H-18/H-21/H-26). The rules engine is SHIPPED and is the deterministic-gate substrate (phase-3F, crate
  elai-rules): Predicate {AstPattern predicate.rs:102, GrepPattern, SchemaRequired, MetricGate, Composite},
  CompiledPredicate (rule_engine.rs:98 SCIP_DEF compiled_regex; AstPattern arm :106; eval-order sort key :327),
  VerdictResult {Allow,Warn,Block,Escalate} (predicate.rs, referenced rule_engine.rs:582), evaluation order
  cheapest-first short-circuit on BLOCK (MetricGate->GrepPattern->AstPattern->Composite, asserted rule_engine.rs
  :1427), lifecycle PROPOSED->CANDIDATE->SHADOW->ACTIVE->RETIRED (3F.4), ALWAYS_INCLUDED_DOMAINS includes
  "security" (rule_engine.rs:187, per F-1), mutable column default 1 (crates/rules/src/schema.rs:46, per F-1) with
  a tested mutable=0 path (crates/rules/src/lifecycle.rs:871, per F-1). AstEvaluator trait (rules depends on
  knowledge/ for tree-sitter, no own grammar). TS dashboard mirror: PredicateType TS type
  (dashboard-v2/src/lib/api/types.ts:852), RuleLifecycleCounts. => T-2/T-4/T-6/T-10 = a NEW
  METRIC_GATE/COMPOSITE/AST_PATTERN rule (DATA in SQLite + a frozen sidecar where math is needed), NOT a new engine.

TS-3 (H-20). The TAINT/PROVENANCE primitive is SHIPPED but the live propagation is PARTIAL/wire-dark (carry C17
  from H-GROUNDING-MAP). Present: provenance.rs (from_web crates/safety/src/provenance.rs:145, Provenance/
  Provenanced), capability_gate.rs (allow_tool_with_origin :84, CapabilityVerdict/CapabilityGateError/
  is_destructive_tool re-exported safety/lib.rs:51), capability_flow.rs (check_against_commitment :187,
  taint_from_verdict :515, taint_text :534-area + CommittedToolSet/committed_slot — the F-3p1 leg is
  cross-entrypoint WIRED). production_agent_rpc.rs:60 imports allow_tool_with_origin. WIRE-DARK/PARTIAL (per C17):
  the E-37 spine (Provenance constructors, allow_tool_with_origin) has NO live prod caller/emitter; taint_text is
  only reachable via `elai safety` replay; G-1 taint_flow.rs (live harness-wide propagation) is ABSENT.
  => T-4 leakage = a NEW leakage-specific COMPOSITE consuming the taint PRIMITIVE; it has a real PREREQUISITE on
  G-1 (live taint propagation) OR builds a leakage-scoped dataflow pass on the DKG. Flag honestly (TD-5).

TS-4 (H-22/H-27). Mutation + metamorphic testing tiers are SHIPPED (C-28/C-29): metamorphic_testing.rs
  (MetamorphicRelation {Identity,Inverse,PermutationInvariance}; as_str :63; passed() :104; RelationLibrary +
  MetamorphicConfig + parse_relation; tests :729-747), mutation_testing.rs (MutantOutcome/MutationRunner/
  RustMutationRunner/mutation_score_pct + MutationSurvived SSB, per C-28), ValidationTier::Mutation. => T-6
  trading nulls = NEW named MetamorphicRelations (future-shuffle/time-reversal/zero-drift) added to the shipped
  RelationLibrary; T-11 verify-the-verifiers = turn the SHIPPED mutation+metamorphic tiers RECURSIVELY onto the
  gates' own logic. Restating the runner is a P0.

TS-5 (H-23/H-25/H-28). Determinism + reproducibility substrate is SHIPPED (C-27 + G-2 — G-SUMMARY said RRA
  ABSENT, that is STALE): determinism.rs (DriftComparator :317 incl audit_id :320/357; DeterminismConfig +
  system_fingerprint_pin from_env :175; DeterminismDriftPayload :403), ELAI_SEED + ELAI_SYSTEM_FINGERPRINT_ENV,
  wiring_contract.rs StubBanAuditor/ObservableEffectRunner (C-27). RRA: rra.rs:200 (struct RunArtifact),
  ssb.rs:20060 (RunArtifactEmittedPayload), ssb_viewer.rs:5248 (RunArtifactRow), `--emit-rra` flag (cli/args.rs:
  2615), replay verify reading artifact.system_fingerprint_pin (cli/run_subcommand.rs:584), replay_resume
  (ssb.rs:23502). => T-7 hash-equality re-run + T-9 result manifest COMPOSE-WITH the RRA/determinism substrate;
  re-implementing RRA or DriftComparator is a P0.

TS-6 (H-25/G-5). The F-4 test-integrity ("cant-lie") leg is SHIPPED: stdout_hash threaded into emit_test_outcome
  (task_runner.rs:16268), the anti-fabrication filter `.filter(|r| r.passed && r.stdout_hash.is_none())`
  (evidence.rs:845), the gate comment "a passing row with None hash is the fabrication signature" (gate.rs:441),
  TestIntegrityEvent SSB (dashboard commands.rs:9610/9636 has_stdout_hash), G-5 DSR recompute
  (g5_dsr_recompute_contract.rs:75, g5_silent_by_default_contract.rs). G-7 confidence (claim_confidence.rs:212
  passing_stdout_hashes). => T-9 EXTENDS F-4 from "test PASS is honest" to "an empirical RESULT (Sharpe/P&L) is
  honest" — a NEW result-manifest schema + re-derivation verifier; do NOT restate the stdout_hash leg.

TS-7 (H-27/H-28). DGM self-mod substrate + the frozen-key machinery are SHIPPED (B-23/D-9/F-11): config_archive.rs
  (select_parent :274, insert_variant), promotion_gate.rs (two_gate_promote :109, tested :182/193/207/221),
  darwin_admit_loop.rs:298 (the LIVE two_gate_promote driver caller — F-11 closed one generation),
  trap_tasks.rs (TrapTask :184, validate_for_traps :124, fake_test_log_trap :246, build registry :262),
  is_security_sensitive (self_mod_layer1a.rs:383), AllowedConfigKey closed enum (self_mod_layer1a.rs:203),
  LockedKeysAuditor + LOCKED_KEYS_TOUCH observable smoke (locked_keys_observable_smoke.rs:52/80/95,
  locked_keys_integration.rs, locked_keys_d10_integration.rs), blake3 archive verify ArchiveVerifyResult
  (ssb.rs:30840, D-8). => T-11/T-12 COMPOSE-WITH this substrate; T-12 reward-tampering ADDS the RF-INPUT
  tamper flag (score jumps with eval-harness hash intact) that the shipped blake3 tripwire does NOT catch.

TS-8 (H-18/H-29). Benchmark + drift substrate SHIPPED: seccodeplt SecCodePltRunner (bench_seccodeplt.rs:295,
  benchmarks/safedelta_runner.rs:454/487, bench_safedelta.rs:371) + InsecureRateMeasured SSB
  (f2c_seccodeplt_baseline_smoke.rs:117) — the frozen-evaluator-driven-by-Rust pattern T-2/T-18 mirror; G-6
  SAFEDELTA (safedelta_runner.rs); model_drift.rs (system_fingerprint :149/161, g10_model_drift_contract.rs,
  G-10) + canary.rs CanaryFamily (D-7). => T-13 drift FSM = a LOCKED interlock COMPOSING the shipped PSI/canary
  signals, not a new drift detector.

## NET-NEW / ABSENT CONFIRMATIONS (the genuine BUILD surface — zero elai.map hits 2026-06-04)

NN-1  trial ledger: `register_trial`, `strategy_trials` — ZERO hits. T-1 (H-17) is pure BUILD on the SSB WAL.
NN-2  finance deflation evaluators: `deflated_sharpe`/`DeflatedSharpe`, `probability_backtest`/PBO, `MinBTL` —
      ZERO hits. T-2 (H-18) is pure BUILD (frozen numpy/scipy sidecar orchestrated by a Rust METRIC_GATE).
NN-3  adaptive-overfitting budget: `reusable_holdout`, `Thresholdout` — ZERO hits. T-3 (H-19) is pure BUILD.
NN-4  leakage-specific predicate: no train/test-sink reachability predicate on the taint graph (the PRIMITIVE
      exists per TS-3; the leakage COMPOSITE does not). T-4 (H-20) = BUILD on the taint primitive (+ G-1 prereq).
NN-5  runtime look-ahead differential (perturb-and-diff feature pipeline): no future-truncation diff gate.
      T-5 (H-21) = BUILD (reuses a REGRET-style feature runner in a frozen sidecar).
NN-6  trading metamorphic relations (future-shuffle/time-reversal/zero-drift null): the RelationLibrary exists
      (TS-4) but carries only Identity/Inverse/PermutationInvariance. T-6 (H-22) = NEW named relations.
NN-7  same-seed metric-OUTPUT hash-equality re-run gate + seed-coverage AST gate: RRA exists (TS-5) but nothing
      RE-RUNS and compares DECLARED-METRIC digests. T-7 (H-23) = BUILD on RRA/replay.
NN-8  sim-to-real parity gates (mode-divergent-branch AST + replay==golden): no `if live:`/`if backtest:` AST
      ban, no event-log replay-equality gate. T-8 (H-24) = BUILD.
NN-9  result manifest + re-derivation: F-4 binds test-PASS (TS-6); nothing binds an empirical RESULT number to
      provenance + re-parses it. T-9 (H-25) = NEW schema + verifier.
NN-10 shadow execution + financial-invariant MR library: no seeded two-impl divergence gate, no put-call-parity/
      strike-monotonicity MR pack, no naked-`==`-on-float AST ban. T-10 (H-26) = BUILD.
NN-11 gate self-validation harness + null_band_sidedness MR + AST ban on symmetric comparator: the mutation +
      metamorphic tiers exist (TS-4) but are not turned onto the gates' OWN logic. T-11 (H-27) = BUILD.
NN-12 reward_tampering_counter + RF-INPUT-tamper flag: blake3 tripwire catches eval-harness hash mismatch (TS-7)
      but not an implausible post-self-mod score jump with the hash intact. T-12 (H-28) = NEW gate leg.
NN-13 LOCKED drift state machine gating the resume/keep-live FSM edge + 90-day model TTL + label-fed concept-
      drift: model_drift.rs is one signal (TS-8); the warning->drift->halt interlock is absent. T-13 (H-29) = BUILD.

## DOCTRINE DELTAS THIS TRACK ADDS (TD-1..TD-5; inherit W-1..10 + GD-1..5 + HD-1..5 verbatim)

TD-1 PERSISTED-N-KEYSTONE. Every significance/edge number deflates by a persisted, append-only, agent-
  unresettable cumulative trial count N (per (data_window, objective) scope). A reported Sharpe/edge whose config
  has ZERO matching trial rows is a HARD FAIL (trial_count_missing), never SKIP. (design D2, Bailey & LdP "Marcos'
  Third Law": N is the only gameable DSR input.) This is the keystone — without it an HONEST searcher still ships
  overfit edge.
TD-2 FROZEN-EVALUATOR. The backtest harness, metric code, gates, thresholds AND the trial ledger ALL live in the
  gate-manifest LOCKED(mutable=0) set the self-mod agent cannot read or mutate (reuse R8 gate_freeze_check.py +
  .claude/gate-manifest.txt + the rules-engine mutable=0 tier + AllowedConfigKey + LockedKeysAuditor). (design D3,
  Everitt current-RF.) Highest-leverage single safety move.
TD-3 LLM-IS-T3-ADVISORY-ONLY. LLM/auditor judgment is a SOFT T3 backstop that advises a human; it is NEVER the
  load-bearing promotion gate. (design D4; BadScientist accepts fabricated results 82%, detects at ~chance.)
  A T3 verdict can never PROMOTE; a deterministic T1 BLOCK can never be overridden by a probabilistic T3 green.
TD-4 VERIFY-THE-VERIFIERS. Each PROPOSED->ACTIVE gate revision must (a) kill 100% of a FROZEN out-of-band mutant
  corpus targeting the gate's OWN logic, (b) satisfy its declared metamorphic relations INCLUDING a mandatory
  null_band_sidedness MR, (c) reject every frozen trap input; any escape blocks promotion + increments a
  persisted escaped-mutant counter. (design D5; the literal REGRET GBM symmetric-vs-one-sided-band incident.)
TD-5 HONEST-RESIDUE. Name the irreducible undecidable residue (design PART 5) and route it to human Layer-3 / T3
  advisory; never advertise a guarantee the mechanism cannot deliver. Cross-hardware FP bitwise equality
  (physics), P(Y|X) concept drift before labels (undecidable), target-derived-feature leakage (needs judgment),
  GENUINE-vs-lucky-search-artifact (necessary not sufficient), self-mod SAFETY (Rice/Schmidhuber) all stay
  human-gated. Extends GD-5 (no fabricated guarantee).

## TRADING-LANE TIERING (W-10, BINDING for every Track-3 H-N)

ELAI is a coding-agent harness; the "trading code/strategy" is what ELAI's autonomous mode GENERATES or
DISCOVERS. The gates live IN ELAI (rules-engine predicates, a frozen Python METRIC_GATE sidecar for DSR/PBO/
leakage-differential math via numpy/scipy, SSB rows, the taint graph). Tiering:
  Tier-1 (the fail-closed done-gate, runs unconditionally): the in-process rules-engine predicate decision + the
    off-agent SSB row (trial row / null-band / replay hash / leakage-block row). Missing evidence = FAIL.
  Tier-2 (separately gated, MUST NOT advertise fail-closed if fail-open): the Python numpy/scipy sidecar that
    computes DSR/PBO/leakage-differential, and any LIVE/PAPER trading run. Ships a deterministic fixture harness
    exercising the REAL path now + a separately-gated live checkbox (mirrors seccodeplt/SAFEDELTA W-10 split).
  The live autonomous-trading lane is DEFERRED behind the fixture harness; no external trust claim before it.

## PER-SESSION SYMBOL MAP (H-17..H-29; file:line:status verbatim 2026-06-04)

### H-17 trial ledger (T-1)  [keystone — nothing else is trustworthy without it]
| symbol | file | line | status | note |
|---|---|---|---|---|
| SSB append-only WAL (single-writer, no UPDATE/DELETE) | crates/orchestrator/src/ssb.rs | (phase-3B) | shipped | The untamperable substrate. register_trial() is a new choke-point + a new strategy_trials table ON it. |
| SchemaRequired / MetricGate predicate (for the choke-point gate) | crates/rules/src/predicate.rs / rule_engine.rs | predicate.rs:102 area; rule_engine.rs:98 | shipped | The promotion gate (T-2) reads N from here; T-1 only WRITES rows + provides the read API. |
| register_trial / strategy_trials | (absent) | n/a | absent (NN-1) | BUILD: append-only table {objective, data_window, config_hash, seed, ts} + single choke-point; no agent delete/reset/update; ONC effective-N clustering for correlated trials. |
| gate-manifest LOCKED set (freeze the ledger + register fn) | .claude/gate-manifest.txt + .claude/tools/gate_freeze_check.py | (R8) | shipped | TD-2: the table + register_trial go in the frozen set. |

### H-18 Deflated-Sharpe / PBO / MinBTL promotion gate (T-2)  [#1 net-new gate]
| symbol | file | line | status | note |
|---|---|---|---|---|
| MetricGate / Composite predicate | crates/rules/src/rule_engine.rs | 98 (CompiledPredicate); 1427 (eval order) | shipped | The LOCKED COMPOSITE blocks promotion unless DSR+PBO+MinBTL clear AND N present. |
| SecCodePltRunner (frozen-evaluator-driven-by-Rust pattern) | crates/cli/src/bench_seccodeplt.rs / orchestrator/src/benchmarks/safedelta_runner.rs | 295 / 454 | shipped | The mirror pattern: Rust orchestrates a frozen scoring body, emits an SSB measured row. T-2's numpy/scipy DSR sidecar follows it. |
| register_trial (N source) | (H-17) | n/a | depends H-17 | Fail-closed if N absent (TD-1). |
| deflated_sharpe / PBO / MinBTL math | (absent) | n/a | absent (NN-2) | BUILD: frozen Python sidecar (numpy/scipy), Tier-2; the Rust METRIC_GATE enforces the threshold (Tier-1). |

### H-19 reusable-holdout query budget (T-3)
| symbol | file | line | status | note |
|---|---|---|---|---|
| MetricGate + LOCKED counter on SSB | crates/rules/src/rule_engine.rs ; crates/orchestrator/src/ssb.rs | 98 ; (3B) | shipped | A LOCKED query-budget counter (Thresholdout/Ladder) on the OOS set. |
| reusable_holdout / Thresholdout | (absent) | n/a | absent (NN-3) | BUILD: DP fresh-holdout budget; after budget spent -> refuse OOS queries (budget_exhausted). |

### H-20 leakage static-analysis COMPOSITE (T-4)  [trading-specific killer]
| symbol | file | line | status | note |
|---|---|---|---|---|
| Provenance / from_web | crates/safety/src/provenance.rs | 145 | shipped (wire-dark per C17) | The taint PRIMITIVE T-4 consumes. |
| allow_tool_with_origin / capability_gate | crates/safety/src/capability_gate.rs | 84 | shipped (wire-dark per C17) | gate-side of taint. |
| taint_text / taint_from_verdict / check_against_commitment | crates/orchestrator/src/capability_flow.rs | 187 (check); 515 (verdict); 534 (taint_text) | partial | taint_text reachable only via `elai safety` replay; F-3p1 CommittedToolSet leg cross-wired. |
| AstPattern predicate (.shift(1) detector) | crates/rules/src/predicate.rs | 102 | shipped | T2 AST_PATTERN for the missing signal->execution shift. |
| G-1 taint_flow (live propagation) | (absent) | n/a | absent | PREREQUISITE (TD-5/GD-1): T-4 needs G-1 live taint OR builds a leakage-scoped dataflow pass on the DKG. Flag honestly. |
| leakage train/test-sink reachability predicate | (absent) | n/a | absent (NN-4) | BUILD: COMPOSITE on the taint graph + SCHEMA_REQUIRED split-boundary declaration; FAIL-CLOSED with a REGISTERED suppression allowlist (never silent skip). |

### H-21 runtime look-ahead / causality differential gate (T-5)  [un-gameable backstop]
| symbol | file | line | status | note |
|---|---|---|---|---|
| MetricGate (orchestrates the diff) | crates/rules/src/rule_engine.rs | 98 | shipped | Rust runs the pipeline twice via a frozen sidecar; diffs columns in Rust. |
| DriftComparator (first-divergence-index pattern to mirror) | crates/orchestrator/src/determinism.rs | 317 | shipped | The "report the first mismatching column/index" pattern. |
| perturb-and-diff feature runner | (absent) | n/a | absent (NN-5) | BUILD: run feature pipeline on full vs future-truncated data; any column that changes uses the future. |

### H-22 trading metamorphic null-test COMPOSITE (T-6)  [generalize the REGRET future-shuffle gate]
| symbol | file | line | status | note |
|---|---|---|---|---|
| MetamorphicRelation enum + RelationLibrary + parse_relation | crates/orchestrator/src/metamorphic_testing.rs | 63 (as_str); 104 (passed); 729-747 (lib tests) | shipped | Carries Identity/Inverse/PermutationInvariance. T-6 ADDS named trading relations. |
| MetamorphicTestRunner + MrViolated SSB | crates/orchestrator/src/metamorphic_testing.rs ; ssb.rs | (C-29) | shipped | The runner + violation row to reuse. |
| MetricGate (DSR-vs-null band) | crates/rules/src/rule_engine.rs | 98 | shipped | The permutation null band gate. |
| future-shuffle / time-reversal / zero-drift-null relations | (absent) | n/a | absent (NN-6) | BUILD: NEW named relations in the shipped library; edge must collapse to noise under shuffled future. |

### H-23 same-seed re-run hash-equality METRIC_GATE (T-7)  [reproducibility centerpiece]
| symbol | file | line | status | note |
|---|---|---|---|---|
| RunArtifact (RRA) + emit/replay | crates/orchestrator/src/rra.rs ; cli/args.rs ; cli/run_subcommand.rs | rra.rs:200 ; args.rs:2615 (emit_rra) ; run_subcommand.rs:584 (replay) | shipped (G-2) | Re-invoke the REAL entrypoint under the recorded env+seed; T-7 adds DECLARED-METRIC-OUTPUT digest equality. |
| DriftComparator + DeterminismDriftPayload | crates/orchestrator/src/determinism.rs | 317 ; 403 | shipped | The blake3 prefix + first-divergence pattern; system_fingerprint_pin from_env :175. |
| seed-coverage AST gate (np/torch/random/DataLoader-shuffle) | crates/rules/src/predicate.rs (AstPattern) | 102 | shipped (predicate) | BUILD the seed-coverage rule; SCHEMA_REQUIRED env-pin manifest. |
| metric-OUTPUT digest re-run-equality | (absent) | n/a | absent (NN-7) | BUILD on RRA/replay. SCOPE (TD-5): same-env/same-seed ONLY; cross-hardware FP is physics — record, never assert. |

### H-24 sim-to-real parity gate (T-8)
| symbol | file | line | status | note |
|---|---|---|---|---|
| AstPattern predicate (`if live:`/`if backtest:` ban) | crates/rules/src/predicate.rs | 102 | shipped | T1 AST ban on mode-divergent branches + duplicate fill/router fns. |
| replay_resume / RRA event-log replay | crates/orchestrator/src/ssb.rs ; rra.rs | 23502 ; rra.rs:200 | shipped | The replay substrate for the golden-hash equality leg. |
| SCHEMA_REQUIRED single-entrypoint manifest | crates/rules/src/predicate.rs (SchemaRequired) | (predicate) | shipped | Declares the single shared entrypoint + swappable adapters. |
| replay==golden hash gate | (absent) | n/a | absent (NN-8) | BUILD: blake3(replay)==blake3(golden); behavioral parity B deferred (needs a live/paper run, Tier-2). |

### H-25 result manifest + re-derivation verifier (T-9)  [extends F-4 from "test passed" to "result honest"]
| symbol | file | line | status | note |
|---|---|---|---|---|
| stdout_hash leg (F-4) | crates/orchestrator/src/task_runner.rs ; evidence.rs ; gate.rs | 16268 (emit_test_outcome); 845 (anti-fab filter); 441 (gate) | shipped | The pattern: a passing claim with None hash is the fabrication signature. T-9 binds an empirical NUMBER the same way. |
| RunArtifact evidence manifest + blake3 | crates/orchestrator/src/rra.rs | 200 | shipped (G-2) | The evidence-hash bundle to extend with {value, captured_output_blake3, parser_expr, data_hash, n_trials, dsr_pvalue, pbo}. |
| AstPattern (forbid metric import/redefine/monkeypatch) | crates/rules/src/predicate.rs | 102 | shipped | Strategy code may not import/redefine/monkeypatch the metric harness; harness in gate-manifest. |
| result-manifest schema + re-parse verifier | (absent) | n/a | absent (NN-9) | BUILD: SCHEMA_REQUIRED manifest + a T1 re-derivation METRIC_GATE (DSR/PBO leg reuses H-17/H-18). |

### H-26 runtime differential / shadow exec + financial-invariant MR library (T-10)
| symbol | file | line | status | note |
|---|---|---|---|---|
| MetamorphicRelation library | crates/orchestrator/src/metamorphic_testing.rs | 104 | shipped | The versioned financial-invariant MR pack (put-call parity, strike-monotonicity, P&L additivity) is NEW relations. |
| mutation_testing.rs (kill-a-seeded-mutant gate for mined invariants) | crates/orchestrator/src/mutation_testing.rs | (C-28) | shipped | Daikon-mined invariants enter PROPOSED and must kill a seeded mutant first. |
| MetricGate (ULP/relative divergence bound) | crates/rules/src/rule_engine.rs | 98 | shipped | divergence<=frozen-ULP gate. |
| seeded shadow executor + naked-`==`-on-float AST ban | (absent) | n/a | absent (NN-10) | BUILD: two-impl divergence gate (new vs prior / vs QuantLib oracle) + AST ban. |

### H-27 gate self-validation (T-11)  [verify-the-verifiers; the GBM-band fix generalized]
| symbol | file | line | status | note |
|---|---|---|---|---|
| mutation_testing.rs (MutationRunner) | crates/orchestrator/src/mutation_testing.rs | (C-28) | shipped | Turn it RECURSIVELY onto the gates' OWN logic (frozen mutant corpus, kill 100%). |
| metamorphic_testing.rs (MR satisfaction) | crates/orchestrator/src/metamorphic_testing.rs | 104 | shipped | Each gate satisfies its declared MRs incl null_band_sidedness. |
| trap_tasks.rs (frozen trap inputs) | crates/orchestrator/src/trap_tasks.rs | 124 (validate); 184 (TrapTask); 246 (fake_test_log_trap) | shipped | Reject every frozen trap; escaped-mutant counter persisted. |
| rules lifecycle PROPOSED->ACTIVE gate hook | crates/rules/src/lifecycle.rs | 871 (mutable=0 path, per F-1) | shipped | T-11 hooks the SHADOW->ACTIVE promotion. |
| null_band_sidedness SCHEMA field + symmetric-comparator AST ban | (absent) | n/a | absent (NN-11) | BUILD: non-defaultable schema field + AST_PATTERN forbidding abs(x)<band on a one-sided-declared gate. |

### H-28 reward-tampering gate + harness-in-the-frozen-set (T-12)  [self-mod safety capstone]
| symbol | file | line | status | note |
|---|---|---|---|---|
| is_security_sensitive | crates/orchestrator/src/self_mod_layer1a.rs | 383 | shipped | The self-mod admit must call it (F-11 killer-risk #1). |
| two_gate_promote + live driver | crates/orchestrator/src/promotion_gate.rs ; darwin_admit_loop.rs | 109 ; 298 | shipped | The promotion path the reward-tamper gate sits in front of. |
| AllowedConfigKey closed enum + LockedKeysAuditor | crates/orchestrator/src/self_mod_layer1a.rs ; tests/locked_keys_observable_smoke.rs | 203 ; 52/80/95 | shipped | TD-2 frozen-key machinery; blake3 archive verify ArchiveVerifyResult ssb.rs:30840. |
| reward_tampering_counter + RF-INPUT-tamper flag | (absent) | n/a | absent (NN-12) | BUILD: HARD-BLOCK on eval-harness blake3 mismatch (shipped tripwire) + FLAG/BLOCK an implausible score delta with the hash INTACT (the new leg); 3a deterministic BLOCK never overridden by 3b green; structural self-mod stays human Layer-3. |

### H-29 LOCKED fail-closed drift state machine (T-13)
| symbol | file | line | status | note |
|---|---|---|---|---|
| model_drift.rs (one signal today) | crates/orchestrator/src/model_drift.rs | 149/161 (system_fingerprint) | shipped (G-10) | The interlock COMPOSES this + PSI + canary, it does not re-detect. |
| canary.rs CanaryFamily | crates/orchestrator/src/canary.rs | (D-7) | shipped | regression delta signal. |
| MetricGate (PSI) + SchemaRequired (schema fingerprint) + Composite (TTL) | crates/rules/src/rule_engine.rs / predicate.rs | 98 ; 102 | shipped | The four-axis interlock predicates. |
| LOCKED warning->drift->halt interlock gating the resume FSM edge | (absent) | n/a | absent (NN-13) | BUILD: PSI in+out, schema-fingerprint, rolling-14d live-vs-backtest deviation, 90-day TTL; missing evidence = HALT. HONEST GUARANTEE (TD-5): "halt within one realized-label cycle", NOT "never trade a regime break". |

## GROUNDING APPENDIX (load-bearing citations — from the design's 15 reports; verify arXiv IDs before any claim)

Selection bias / DSR: Bailey & Lopez de Prado (Deflated Sharpe Ratio; PBO/CSCV; MinBTL; "Marcos' Third Law" N);
  Harvey & Liu (deflated t>3.0); Dwork et al. reusable holdout / Thresholdout; Blum & Hardt "The Ladder".
Leakage: Subotic & Yang ASE'22 (Datalog-over-SSA, 92.9%); Drobnjakovic/Subotic/Urban abstract interpretation
  (arXiv:2211.16073); Kapoor & Narayanan "Leakage and the Reproducibility Crisis" (294 papers); Freqtrade
  look-ahead-analysis (perturb-and-diff).
Discovery validation: FunSearch (Nature 2023); AlphaEvolve; BadScientist (LLM-judge accepts fabrication 82%).
Reproducibility: Thinking Machines batch-invariant kernels (2025); ML reproducibility crisis.
Verify-the-verifiers: Papadakis 2019 (mutation); Segura 2016 (metamorphic); Barr 2015 (oracle problem);
  Inozemtseva & Holmes ICSE'14 (coverage != effectiveness).
Self-mod safety: Everitt et al. reward tampering (arXiv:1908.04734, current-RF); Denison et al.
  Sycophancy-to-Subterfuge; Rice / Schmidhuber undecidability.
Drift: NannyML / Gama et al. / Lu et al. concept-drift surveys; ADWIN / DDM / Page-Hinkley.
Runtime verification: RV-Monitor/MOP; Daikon; FPDiff; QuantLib.
Full per-topic detail + sources: D:\.REGRET_workflow\workflow\research\elai-trust\reports\01..15 (cross-project;
  the design doc PART 6 + GROUNDING APPENDIX is the in-repo digest). Per CLAUDE.md paper-grounding: spawn
  paper-ingestor (mode=paper) for any arXiv ID a Track-3 H-N depends on BEFORE EXECUTING; cite quote+page-anchor
  for effect-size numbers (feedback_paper_citation_verify — 2 bad cites already caught in B-40/B-42).

End of H-TRUST-GROUNDING-MAP.md.
