CONDUCTOR QUALITY GAP: fork-DONE vs main-landable
date 2026-06-29  controller Opus 4.8

PROBLEM
The conductor marks ~600+ tasks DONE but the merge lane lands 0 (TICK 9: processed:4
landed:0 escalated:1 quarantined:3). The merge gate is working as designed; the overnight
free-glm output is genuinely not main-quality. Root cause is a gate asymmetry: the per-task
DONE gate (gate_runner.run_gates) is a strict SUBSET of the merge gate (merge_lane). So
"DONE" never meant "landable". The 600 were certified against the weaker bar.

THE ASYMMETRY (measured from code, not narrative)
per-task gate (gate_runner.build_gate_commands + the 3 synthetic stages):
  gate_freeze_check; production_caller_check (--symbols, only if NEW pub rs syms);
  python_wire_check (only if NEW py syms); maintainability_check; traced_test_assert_check
  (only if test .rs touched); wiring_contract_check + spec_link_check (only if plan file
  resolves); cargo check (--workspace OR -p when scoped_cargo); cargo test -p <crate> --lib.
  Optional, ALL DEFAULT-OFF: slop_cleaner, cli_runtime_backstop, surface_contract.
merge gate adds, on top of the same static checks:
  1. broader cargo test catching cross-crate new_failures (per-task only runs -p crate --lib);
  2. classifiers (LITTER / GOODHART-DEP / DEP-SLOPSQUAT) -> caught contracts litter, knowledge
     ort-disable goodhart;
  3. air-gapped downgrade-only GLM review (merge_reviewer.review_slice) -> caught providers
     tautological tests (assert source CONTAINS strings) + python writing fabricated True
     "acceptance evidence" into the Rust-owned SSB DB via PRAGMA ignore_check_constraints=ON.

KEY FACTS
- The trust-hardening apparatus is already BUILT and TESTED but dormant: conductor_config
  LimitConfig has surface_contract, cli_runtime_backstop, slop_cleaner, scoped_cargo,
  failure_classifier, wire_dark_sweep, suspicious_empty_guard -- all default False. Prior
  sessions built the bar and never armed it.
- merge gate is per CRATE SLICE (aggregates all tasks touching that crate). So ONE
  fabricated-evidence task poisons the entire crate slice and blocks every good task with it.
  This is why it is all-or-nothing per crate.
- The 600 are NOT lost: fork commits persist in conductor/elai-repo, regenerable. But DB
  "DONE" status is meaningless for landing.
- merge_reviewer.review_slice(slice_id, slice_diff, gate_summary) -> ReviewVerdict, and
  reconcile_verdict(gate_ok, review) is DOWNGRADE-ONLY (a model can never upgrade a gate
  FAIL to LAND). Directly reusable per-task.
- LIVE: the daemon drives workers through gate_runner now; a change to gate_runner or to
  config.limits is picked up on the next pass. Every change must be fail-closed + reversible
  via a kill-switch flag.

DECISION (user: do all 3)
Order: build the bar (1 + 2) BEFORE re-gating (3). Re-gating against the old bar would
re-certify slop.

PART 1  harden the per-task gate so DONE == landable going forward
  1a. NET-NEW deterministic detectors (the genuinely-missing checks the merge GLM catches):
      - tautological / source-grep test detector: flag test bodies that assert on SOURCE
        TEXT not behavior (include_str! of a .rs then .contains; read_to_string of a source
        path then assert contains; assert!(...contains("fn "/"pub "/"struct ")). Layer-2
        static, necessary-not-sufficient, fail-closed, exit 1 on finding.
      - trust-violation ban: PRAGMA ...ignore_check_constraints; foreign_keys=OFF on the SSB
        DB; python opening/writing the Rust-owned ssb.db; fabricated literal evidence. HARD
        BLOCK (trust-model violation, not a style nit).
      Wired as added findings inside the existing slop_cleaner stage (one place, one flag,
      already frozen) AND shipped as a standalone .claude/tools checker the merge lane can
      also call. Added to gate-manifest freeze.
  1b. broaden per-task cargo test from `-p crate --lib` toward the crate's full test set
      (drop --lib) so cross-crate / integration new_failures surface at task time; rely on
      mask_baseline_failures for pre-existing reds. Behind a flag for safe rollout.
  1c. arm the dormant stages in config, carefully:
      - slop_cleaner=true (now carries 1a) -- LOW blast radius, changed-files-only.
      - cli_runtime_backstop=true -- demands real off-agent SSB row for cli plans (defeats
        the PRAGMA-fabrication class at runtime).
      - scoped_cargo: keep current; revisit with 1b.
      - surface_contract: DO NOT arm globally yet. gate_runner only positively asserts cli
        evidence; ui/api/algo surfaces fail-closed -> arming it now mass-NEEDS_REVIEWs every
        non-cli task. Stage it after the ui/api/algo evidence sources are wired (separate).

PART 2  per-task GLM review before DONE
  Add a default-OFF gate stage that, after the static gate PASSES, runs
  merge_reviewer.review_slice over the task diff + gate summary and applies reconcile_verdict
  (downgrade-only). A QUARANTINE/ESCALATE downgrades the task to NEEDS_REVIEW instead of
  VERIFIED. Free (glm-5.2 worker), air-gapped, fail-soft (unparseable -> hold, never pass).
  This stops slop at the source so crate slices never get poisoned. Cost: one review/task
  throughput. Flag: per_task_review.

PART 3  re-gate the 600 backlog against the hardened bar
  A new conductor subcommand that, for each DONE task, re-runs the hardened gate over its
  integrated changeset on conductor/elai-repo and, on FAIL, reverts DONE -> NEEDS_REVIEW with
  a typed reason so the fleet re-does it properly. Read-only sample first (report real slop
  %), then --apply. Builds on the existing wire_dark_sweep read-path. Backs up control.db
  first.

RISKS / GUARDRAILS
- Live daemon: every gate change is fail-closed + kill-switch flagged; verify conductor suite
  green before arming any flag.
- surface_contract blast radius (above) -- explicitly staged, not armed now.
- Part 2 throughput cost -- per_task_review default-OFF, arm after measuring.
- Part 3 mass status revert -- control.db backup + read-only sample + typed reasons + reversible.
- All new gate tools frozen in gate-manifest (R8) so a worker run cannot weaken them.

VERIFICATION PLAN
conductor suite (python -m unittest discover -s tests) green after each part; new tests per
detector (tautological true/false, trust-violation true/false), per-task-review wiring test,
re-gate dry-run test. Do NOT run the full .elai_cc workspace test suite here.
