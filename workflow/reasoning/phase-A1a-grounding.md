GROUNDING SWEEP -- phase A1a
Source: F:\.ELAI_workflow\workflow\handoff\phase-A1a.md
Date: 2026-05-13
FSM: PLANNING | Session: s-34648

SPEC REFERENCES

docs/specs/guide/steps/phase-4F.txt (production_construction_paths_wired claim, FeasibilityProbe trait) | GROUNDED | phase-4F.txt:156-188 defines FeasibilityProbe trait; metrics/phase_4F.toml [task_runner].production_construction_paths_wired=false confirms the gap the phase-A1a plan targets

docs/specs/guide/steps/phase-4H.txt (Layer1A processor wiring, OSC/REG/EVO gates) | VERIFY_NEEDED | phase-4H.txt mentions DefaultLayer1aProcessor only in deterministic check greps (line 1146); no CLI entry wiring specified. Spec covers dashboard + self-mod pipeline only. See MISMATCH #2 below.

CLAIM CLASSIFICATION

"production constructor must exist; if missing, add it" | MISMATCH | self_mod_layer1a.rs:486 shows pub fn new(rules_db: Arc<AsyncMutex<Connection>>, rules_db_path: PathBuf, ssb: Arc<Ssb>) -- constructor EXISTS with 3 required args (not just a path). Plan implies constructor may be absent AND implies it takes only &Path. Both wrong.

"DefaultLayer1aProcessor constructed from rules.db path (canonical)" | MISMATCH | real new() at self_mod_layer1a.rs:486-504 requires Arc<AsyncMutex<Connection>> + PathBuf + Arc<Ssb>. CLI must open SQLite, wrap in Arc<AsyncMutex>, and provide an Arc<Ssb>. Missing args will cause compile error.

"DefaultFeasibilityProber constructed mirroring dashboard-v2 state.rs:382" | VERIFY | phase_4F.toml [dashboard].default_feasibility_prober_in_appstate=true confirms it is wired in dashboard; state.rs:382 is the correct template reference. VERIFIED: pattern exists.

"production_construction_paths_wired = false in phase_4F.toml" | GROUNDED | metrics/phase_4F.toml [task_runner].production_construction_paths_wired=false -- exact field confirmed at line read.

"phase-4H.txt requires Layer1A wiring at CLI entry" | UNGROUNDED | phase-4H.txt contains zero references to CLI entry point, clap, start_task_runner, or TaskRunnerConfig. Spec only wires Layer1A inside the self-mod pipeline (data-table growth triggers). CLI wiring is a project decision in phase-A1a, not a 4H requirement.

"FSM grep check (phase-4H.txt:1146-1170) covers CLI wiring case" | UNGROUNDED | line 1146 text: grep -r "Layer1A|layer_1a|data_table_validation" .elai_cc/crates/ | wc -l >= 3 -- checks symbol presence only, not CLI construction path or production constructor call. Does NOT validate CLI wiring.

"OSC/REG/EVO gates wired (phase-4H.txt)" | GROUNDED | phase-4H.txt:1123-1134 defines OSC/REG/EVO behavior. metrics/phase_4H.toml [convergence] confirms gate fields present. Not relevant to phase-A1a (self-mod subsystem, no CLI interaction).

"tokio::select! for Ctrl-C graceful shutdown in CLI" | VERIFY | tokio::select! + tokio::signal::ctrl_c() is standard tokio pattern. Anti-stale-training rule: tokio docs confirm signal::ctrl_c() in async context. VERIFIED: tokio 1.x signal module provides ctrl_c() future.

"SSB TaskSubmitted row drives drive_task end-to-end within 10s" | UNGROUNDED | No spec file cites a 10s latency bound for SSB->drive_task in test harness. No measured baseline in any metrics/*.toml. Origin unclear.

"Zero stubs: no .unwrap()/.expect() on production paths" | GROUNDED | CLAUDE.md hard rule (no .unwrap() in production Rust). Project-internal, not external claim. Grounded by CLAUDE.md.

ACTIONS REQUIRED

- MISMATCH: phase-A1a.md says "production constructor must exist; if missing, add it" but self_mod_layer1a.rs:486 shows it exists with signature pub fn new(rules_db: Arc<AsyncMutex<Connection>>, rules_db_path: PathBuf, ssb: Arc<Ssb>). Remove the conditional "if missing" clause. Executor must open rules.db via sqlx/rusqlite, wrap in Arc<AsyncMutex<Connection>>, and supply Arc<Ssb> -- not just pass a &Path. Update files_to_edit and acceptance_criteria in phase-A1a.md before TESTING_FIRST.

- MISMATCH: phase-4H.txt does not mandate Layer1A wiring at CLI entry. The citation "referenced via metrics/phase_4H.toml" is valid for OSC/REG/EVO bookkeeping but the CLI wiring is a project-level decision not traceable to 4H spec requirements. The spec_references entry for phase-4H.txt is misleading -- replace with an explicit project-decision citation or remove it.

- UNGROUNDED claim "SSB -> drive_task end-to-end within 10s" -- no spec or measured baseline. Either add a measured basis (prior benchmark) or annotate as "UNGROUNDED: timeout value chosen conservatively, no empirical basis yet" in the contract test spec.

- UNGROUNDED claim "FSM grep check at phase-4H.txt:1146 covers CLI wiring" -- it does not. Remove this from any grounding justification. The CLI wiring has no deterministic check in any existing spec; add one explicitly in phase-A1a acceptance_criteria (e.g., grep "start_task_runner" .elai_cc/crates/cli/src/ | wc -l >= 1).

VALIDATION WARNINGS

phase-4H.txt cited without line range -- only a toml metrics reference. Per-phase spec content for CLI wiring is absent (see MISMATCH #2 above).
DefaultLayer1aProcessor::new signature mismatch between plan description and actual code at self_mod_layer1a.rs:486 is a compile-time blocker if not corrected before implementation.
