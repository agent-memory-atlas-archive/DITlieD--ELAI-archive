GROUNDING SWEEP -- phase H-24
Source: F:\.ELAI_workflow\docs\thefinalglance\H-plan\H-24_sim_to_real_parity.md
Date: 2026-06-05
FSM: PLANNING | Session: s-8404

SPEC REFERENCES

_grounding/H-TRUST-GROUNDING-MAP.md TS-5, NN-8, TD-5 | GROUNDED | file exists; TS-5 at map line 70-77 covers replay_resume/RRA/DriftComparator; NN-8 at line 118-119 confirms mode-divergent ban + replay==golden ABSENT; TD-5 at line 149-153 honest-residue doctrine present
_grounding/H-TRUST-GROUNDING-MAP.md H-24 symbol table | GROUNDED | H-24 section at map line 224-230: AstPattern predicate.rs:102, replay_resume ssb.rs:23502, SchemaRequired, replay==golden absent (NN-8)
design T-8 + PART 4 | MISSING | cited as "design T-8 + PART 4"; H-TRUST-GROUNDING-MAP.md notes source is D:\.REGRET_workflow\memory-bank\plans\elai-trust-hardening-design.md (cross-project, not in F:\.ELAI_workflow); no per-phase file at F:\.ELAI_workflow\docs\specs\guide\steps\; this citation is VERIFY_NEEDED -- the grounding map functions as the in-repo proxy and is itself verified
H-23_same_seed_rerun_hash_equality.md (shared replay discipline) | GROUNDED | file exists at F:\.ELAI_workflow\docs\thefinalglance\H-plan\H-23_same_seed_rerun_hash_equality.md
CLAUDE.md HARD RULES (blake3, ClockProvider, GD-5, TD-5) | GROUNDED | blake3 HARD RULE present in CLAUDE.md Anti-Stale-Training block; ClockProvider confirmed in codebase (anti_spiral.rs:96, determinism.rs:322)

COMPOSE-WITH CODE VERIFICATION

crates/orchestrator/src/ssb.rs replay_resume | VERIFY_NEEDED | function exists at ssb.rs:23963 (not :23502 as cited). Line 23502 is SaasPriorArtEventPayload.ts_ms -- wrong line. Function is real, line number is stale by 461 lines.
crates/cli/src/run_subcommand.rs:584 RRA replay verify | MISMATCH | line 584 is NOT the RRA replay verify; it is artifact.system_fingerprint_pin dereference inside emit_model_drift_check (G-10 model drift subcommand). No RRA replay verify function found at this location via grep. The claim "RRA replay verify" at :584 is wrong -- grep found no "replay.*verify" match anywhere in run_subcommand.rs.
crates/orchestrator/src/determinism.rs DriftComparator:317 | GROUNDED | pub struct DriftComparator at line 317 confirmed; reports first mismatching event per doc comment line 310-316
crates/rules/src/predicate.rs:102 AstPattern + SchemaRequired + MetricGate | GROUNDED | predicate.rs:102 is AstPattern enum arm (read lines 95-126); SchemaRequired at :121; MetricGate at :126 -- all three present
crates/rules/src/rule_engine.rs:98 CompiledPredicate | GROUNDED | confirmed: pub enum CompiledPredicate at :98 (grep hit)
FpTolerance from lookahead_differential.rs (H-21) | GROUNDED | FpTolerance in lookahead_check_cmd.rs:30 + h21_lookahead_differential_contract.rs:28; lookahead_differential module confirmed shipped

CLAIM CLASSIFICATION

replay_resume function in ssb.rs -- SHIPPED | GROUNDED | confirmed at ssb.rs:23963; VERIFY: cited line :23502 is wrong (stale by 461 lines)
DriftComparator at determinism.rs:317 -- SHIPPED | GROUNDED | determinism.rs:317 confirmed pub struct DriftComparator
RRA replay verify at run_subcommand.rs:584 -- SHIPPED | MISMATCH | :584 is emit_model_drift_check body; no replay-verify function there; grep finds no replay.*verify in run_subcommand.rs
SCHEMA_VERSION = 207 / TOTAL_VARIANTS = 484 as baseline | UNGROUNDED | LIVE values: SCHEMA_VERSION=228 (ssb.rs:3057), TOTAL_VARIANTS=505 (action_type_lock.rs:684); e45:294 already pins 505; plan says "bump e45:294" but e45:294 already reads 505 -- the plan's baseline numbers are ~21 schema versions stale
verify_replay_golden ABSENT (NN-8) | GROUNDED | zero grep hits for verify_replay_golden or sim_real_parity in crates -- BUILD confirmed
mode-divergent-branch AST ban ABSENT (NN-8) | GROUNDED | zero grep hits for if live / if backtest rule in crates/rules/seed/repro -- BUILD confirmed
blake3 for replay/golden hash | GROUNDED | CLAUDE.md HARD RULE Anti-Stale-Training: blake3 mandatory; sha2/blake2 banned
LEAN-style live-vs-backtest reconciliation pattern | INGEST | H-24 line 141 says "WebSearch the LEAN reconciliation"; no paper ingested; LEAN is QuantConnect's open-source backtester -- the "single entrypoint with swappable adapters" design pattern is attributed to LEAN but no source-grounded interface map exists; claim is load-bearing for Task 2 design
exchange-core determinism (integer money math, single-writer loop) | VERIFY | integer math: no trading-specific money module found in ELAI crates (ELAI generates trading strategies, does not implement exchange fill models internally); claim is about the convention ELAI enforces in GENERATED code, not ELAI's own code -- grounded by C-15 entropy quarantine + ClockProvider (confirmed) but "integer money math" as a banned pattern in generated code needs the AST rule (not yet built, that is NN-8)
ClockProvider banned wall-clock | GROUNDED | ClockProvider confirmed in orchestrator (anti_spiral.rs:18, determinism.rs:322); ELAI_SEED confirmed in determinism.rs:47-48
Bailey & Lopez de Prado backtest overfitting claim (selection bias / N as gameable input) | GROUNDED | deflated-sharpe-ratio-2460551-interface.md ingested at HIGH reliability; "Marcos Third Law: N is the only gameable DSR input" grounded at interface line 85-88; DSR selection bias mechanism grounded
behavioral parity B DEFERRED (Tier-2) | GROUNDED | plan is explicit (lines 65-68, 89, 110, 122); no numeric threshold claimed; GD-5 honest-residue doctrine applied -- correct
golden event log provenance (frozen artifact, not agent-regeneratable) | GROUNDED | R8 (gate_freeze_check.py + gate-manifest.txt) cited in WIRING_CONTRACT; H-28 reward-tampering guards regeneration; freeze mechanism confirmed shipped (TS-7: gate_freeze_check.py at .claude/tools/)

ACTIONS REQUIRED

- MISMATCH: run_subcommand.rs:584 is NOT the RRA replay verify -- it is artifact.system_fingerprint_pin dereference in emit_model_drift_check. Before EXECUTING, grep run_subcommand.rs for the actual replay-verify entry point (pattern: replay|rra.*verify) and update the Reads block with the correct line. If no replay-verify sub-command exists yet, document as ABSENT.
- MISMATCH (stale line): ssb.rs replay_resume cited at :23502 but actually at :23963. Update Reads block in H-24 to ssb.rs:23960-23990 before EXECUTING.
- MISMATCH (stale baseline): SCHEMA_VERSION cited as 207, TOTAL_VARIANTS as 484 -- LIVE is 228/505. The plan says "baseline+1 from LIVE 207/484, re-read + bump e45:294" but e45:294 already pins 505. Executor must re-read ssb.rs const SCHEMA_VERSION + action_type_lock.rs TOTAL_VARIANTS at edit time and apply +1 from the live value; do not use 207/484.
- INGEST: LEAN-style live-vs-backtest reconciliation pattern (single shared entrypoint + swappable adapters) -- spawn paper-ingestor mode=source with path to LEAN engine source (github.com/QuantConnect/Lean) or WebSearch "LEAN backtester single entrypoint adapter pattern"; this claim is the architectural basis for Task 2 (SCHEMA_REQUIRED single-entrypoint manifest). Block Task 2 design until grounded.
- VERIFY: exchange-core "integer money math" as an enforced AST pattern in GENERATED trading code -- confirm whether ELAI already has a rule banning float arithmetic in fill/order modules or whether H-24 Task 1 AST_PATTERN is the first enforcement point. Grep crates/rules/seed/ for any existing integer-money or float-ban rule before authoring sim_real_parity.toml.

VALIDATION WARNINGS
- ssb.rs:23502 cited in Reads block and H-TRUST-GROUNDING-MAP.md TS-5 is stale (off by 461 lines); the grounding MAP was authored 2026-06-04 and ssb.rs has grown since
- SCHEMA_VERSION 207 / TOTAL_VARIANTS 484 in H-TRUST-GROUNDING-MAP.md canonical-constants header are stale (live: 228/505); map instructs "re-read at edit time" -- that instruction is valid but executor must heed it; plan text repeating 207/484 is misleading
- "design T-8 + PART 4" refers to D:\.REGRET_workflow cross-project doc not available in this repo; H-TRUST-GROUNDING-MAP.md is the in-repo proxy and is verified; treat as VERIFY_NEEDED resolved by the map
- all sources validated (grounding map file present and on-topic; code symbols confirmed except two stale line numbers and one missing replay-verify function)
