GROUNDING SWEEP -- phase H-21
Source: F:\.ELAI_workflow\docs\thefinalglance\H-plan\H-21_runtime_lookahead_differential.md
Date: 2026-06-05
FSM: PLANNING | Session: s-8404

SPEC REFERENCES
H-TRUST-GROUNDING-MAP.md TS-2,TS-5,NN-5,TD-5 | GROUNDED | file exists at docs/thefinalglance/H-plan/_grounding/H-TRUST-GROUNDING-MAP.md; TS-2 (lines 39-49) covers MetricGate/rule_engine.rs:98; TS-5 (lines 70-77) covers DriftComparator/determinism.rs:317; NN-5 (line 112-113) confirms perturb-and-diff runner absent; TD-5 (lines 149-153) confirms cross-hardware FP residue is physics, not asserted
determinism.rs:317-410 (DriftComparator + first-divergence-index + DeterminismDriftPayload) | GROUNDED | Read confirmed: DriftComparator struct at line 317; compare() method finds first mismatch index (lines 371-387); DeterminismDriftPayload at line 398-404 with first_divergence_index field at line 402; blake3 hash prefix used (line 400-401)
rule_engine.rs:98 (MetricGate) | GROUNDED | Read confirmed: CompiledPredicate::MetricGate enum variant at line 98-104; module doc at line 6 confirms MetricGate is first evaluation order
e45_cross_language_drift_contract.rs:294 | GROUNDED | Read confirmed: line 293-296 is the TOTAL_VARIANTS assertion ("TOTAL_VARIANTS: usize = 502"); plan instruction to bump this line is accurate
design T-5 + report 08 (Freqtrade look-ahead-analysis) | VERIFY_NEEDED | "design T-5" is a label in D:\.REGRET_workflow elai-trust-hardening-design.md (external workflow, not readable here); "report 08" refers to Freqtrade method section in that design doc; these are cross-workflow internal refs, not standalone files in F:\.ELAI_workflow -- the plan acknowledges this by saying "WebSearch the Freqtrade method before coding"
H-20_leakage_taint_composite.md | GROUNDED | file exists at docs/thefinalglance/H-plan/H-20_leakage_taint_composite.md (confirmed via Glob)

CLAIM CLASSIFICATION
DriftComparator first-divergence-index pattern at determinism.rs:317 | GROUNDED | Read lines 317-408; struct at 317, first-mismatch loop at 371-387, payload emitted at 398-407; blake3 hash prefix confirmed; pattern is reusable as cited
MetricGate at rule_engine.rs:98 orchestrates diff | GROUNDED | Read lines 98-114; CompiledPredicate::MetricGate variant exists; module doc confirms eval order MetricGate->GrepPattern->AstPattern; the plan's reuse instruction is sound
perturb-and-diff feature runner ABSENT (NN-5) | GROUNDED | H-TRUST-GROUNDING-MAP.md line 112-113 explicitly lists as absent; grounding map line 206 table row confirms "(absent), absent (NN-5)"; no file exists in codebase
Freqtrade look-ahead-analysis is perturb-and-diff | VERIFY | PARTIALLY MATCHES -- see note below; classified VERIFY not UNGROUNDED because the core idea (re-run on truncated data, compare signals) is real, but the exact mechanism differs from the plan's description
cross-hardware FP bitwise equality is physics not asserted | GROUNDED | H-TRUST-GROUNDING-MAP.md TD-5 line 149-153: "Cross-hardware FP bitwise equality (physics)...all stay human-gated"; consistent with plan's relative tolerance + stable verdict band claim
ELAI_SEED provides deterministic re-run (GD-2) | VERIFY | GD-2 referenced in grounding map line 220; ELAI_SEED referenced in CLAUDE.md but no direct grep of crates/core ELAI_SEED constant performed; plan says "recomputable from the journal (GD-2 same-env)" -- requires executor to verify ELAI_SEED is wired through the frozen runner
LookaheadDifferential SSB row Pattern-2 baseline+1 | VERIFY | TOTAL_VARIANTS confirmed at 502 in e45 contract (line 294) but SCHEMA_VERSION not re-read live; grounding map says 207/484 as of 2026-06-04, e45 contract now pins 502 variants -- baseline for H-21's +1 requires executor to re-read ssb.rs:2909 before coding
iloc[t+1] fixture must FAIL (produces different column under truncation) | GROUNDED | core invariant of the perturb-and-diff method; confirmed by Freqtrade source: analyze_indicators() at line 68-95 finds indicator differences between full_df and cut_df; this invariant holds for any feature reading future data

FREQTRADE LOOK-AHEAD-ANALYSIS -- DETAILED VERDICT

The plan claims Freqtrade look-ahead-analysis uses "perturb-and-diff" by "re-running a strategy on a truncated dataset and comparing signals."

What Freqtrade actually does (verified from github.com/freqtrade/freqtrade develop branch, freqtrade/optimize/analysis/lookahead.py, 283 lines, fetched 2026-06-05):

MATCHES the plan's high-level description:
- It runs the strategy TWICE per trade signal: once on full data (full_varHolder), once on time-truncated data (entry/exit varHolder with to_dt cut to the signal timestamp + 1 candle)
- It compares the resulting signals (report_signal()) and indicators (analyze_indicators()) between the two runs
- Any column that differs between full_df and cut_df is flagged as a look-ahead bias indicator (lookahead.py:88-95)
- It does identify the first differing indicator column by name (false_indicators list, line 89-90)

DOES NOT MATCH in one important respect:
- Freqtrade does NOT compare feature/signal column VALUES with a ulp/relative tolerance; it uses Python DataFrame.compare() which finds ANY difference (line 75: `cut_full_df.compare(cut_df)`) -- exact equality, not float tolerance
- Freqtrade's gate is per-TRADE-SIGNAL (truncate to the trade's open/close date, not a single future-truncation of the whole dataset); H-21 proposes a SINGLE future-truncation of the whole feature pipeline
- Freqtrade is signal-comparison-based (did the buy/sell signal appear?); H-21 proposes column-value-diff-based (did any feature column VALUE change?)

VERDICT: the Freqtrade look-ahead-analysis IS a real, verified open-source implementation of look-ahead bias detection using perturb-and-diff methodology. The plan's description is correct at the conceptual level (run twice, compare). The implementation differs in granularity (per-signal vs whole-pipeline) and tolerance (exact vs ulp). H-21's design is a sound extension of the same idea, not a direct port. The plan's instruction "WebSearch the Freqtrade method before coding" appropriately defers exact implementation decisions.

ACTIONS REQUIRED
- verify ELAI_SEED wiring through crates/core before coding lookahead_runner.py; grep "ELAI_SEED" in .elai_cc/crates/core/src/ to confirm the constant exists and is exported
- verify live SCHEMA_VERSION by reading crates/orchestrator/src/ssb.rs:2909 at EXECUTING time (grounding map value 207 is from 2026-06-04; Track 1/2 H-Ns may have bumped it); do NOT hard-code 207+1
- verify live TOTAL_VARIANTS by reading crates/orchestrator/tests/action_type_lock.rs:549 at EXECUTING time (e45 contract now pins 502, grounding map said 484 -- two bumps already happened since the map was written)
- note: design T-5 / report 08 are in D:\.REGRET_workflow (external workflow); the plan correctly says "WebSearch the Freqtrade method before coding" -- no blocking action, the core claim is verified above
- Freqtrade uses exact DataFrame.compare() not ulp tolerance -- executor must define the relative tolerance constant explicitly (not borrowed from Freqtrade); document the choice in H-21-decisions.md

VALIDATION WARNINGS
- "design T-5" and "report 08" in Reads (mandatory) section resolve to D:\.REGRET_workflow which is outside this workflow root; not MISSING (plan acknowledges WebSearch substitution) but executor cannot Read them directly
- grounding map SCHEMA_VERSION/TOTAL_VARIANTS constants (207/484) are already stale vs e45 contract (502 variants); always re-read live values at EXECUTING time
- no arXiv paper citations in H-21; no paper grounding needed
- all sources validated (no missing mandatory spec files within F:\.ELAI_workflow scope)
