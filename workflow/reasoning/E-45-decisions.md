E-45 — Cross-language drift detector (session 2/4 of s-25120 cluster)

CONTEXT

E-45 plan as originally drafted prescribed ts-rs codegen (Tasks 1+2) + schemars on the
sidecar boundary (Task 4) + Pydantic mirror via datamodel-code-generator (Task 5).
The cross-check report (workflow/sessions/s-25120/cross-check-E/E-45_cross_language_drift-report.md)
flagged 13/17 claims as MISSING and surfaced the load-bearing conflict: D-10 DECISION-8
(2026-05-24) committed to a hand-authored TS mirror over ts-rs codegen, which directly
contradicts Tasks 1+2. The B-3 ts-rs SHIPPED claim in B-PLAN-SUMMARY (lines 139, 1028)
was itself drift — zero ts-rs / ts_rs / #[derive(TS)] hits anywhere in elai.map or
Cargo.lock at the time of execution.

DECISION-1 — Tasks 1+2 (ts-rs codegen) DEFERRED, supersede with host-side detector.

The plan rebase prompt enforces this: DECISION-8 is locked, B-3 claim is drift, and
re-litigating the ts-rs adoption is out of scope for E-45. Instead, this session
ships a Rust-host-side `cross_lang_drift` module that LINTS the hand-authored TS
mirror (and the new Pydantic boundary) by presence + serde-field-name shape probe,
not by codegen.

Substituted-for surface (alternative substrate that satisfies the same drift-gate
intent without violating DECISION-8):

  ts-rs codegen + snapshot test   ->  PairRegistry + DriftDetector::diff_pair
                                       (heuristic field-name presence on both
                                       sides; SchemaVersionMismatch arm for the
                                       version-constant comparison)
  schemars on sidecar boundary    ->  python/elai_inference/schemas/messages.py
                                       (Pydantic v2 typed boundary; SCHEMA_VERSION
                                       pinned to orchestrator constant via the
                                       __init__.py export)
  datamodel-code-generator mirror ->  hand-authored Pydantic mirror in messages.py
                                       (no Python codegen dep added; pure-Pydantic
                                       definitions match the dict-get defaults
                                       in server.handle_messages)

Costs: hand-authored Pydantic mirror has the same maintenance burden as the
hand-authored TS mirror — bumping the orchestrator side requires a same-commit
update on the sidecar side. The DriftDetector enforces this discipline at
audit / pre-commit time via field-name presence checks.

Benefits: zero new top-level deps (CLAUDE.md HARD RULE), one fewer build step
(no Python codegen target), and a single uniform discipline (hand-authored
mirrors on both Tauri and sidecar sides).

DECISION-2 — SCHEMA bump 136 -> 137 atomic.

Live state at session start: SCHEMA_VERSION=136 (per E-44 ship); TOTAL_VARIANTS=391;
all_test_payloads().len() = 325. Per the concurrent session warning ("Session 3
editing. SCHEMA=136. Read fresh; bump observed+1") and CLAUDE.md "SCHEMA bump
atomic: read live, current+1" rule, we re-read just before the edit and confirmed
no concurrent bump, then incremented to 137.

E-45 adds 3 SSB action types:
  - SsbActionType::CrossLangDriftDetected (one row per diff_pair returning >= 1 drift)
  - SsbActionType::CrossLangPairScanned (one row per detect_pairs repo walk)
  - SsbActionType::SchemaVersionMismatch (one row when rust_schema != python_schema)

TOTAL_VARIANTS bumps 391 -> 394.
all_test_payloads().len() bumps 325 -> 328.
Golden file regenerated (alphabetically sorted, 0-based ordinals matching
runtime_body_lines's enumerate() output).

DECISION-3 — 10th lockstep site (TOTAL_VARIANTS const) included per E-44 gotchas.

`workflow/memory-bank/gotchas.md` documents E44_TENTH_LOCKSTEP_SITE: action_type_lock.rs
TOTAL_VARIANTS + ALL_VARIANTS array IS the 10th lockstep site beyond the canonical
9-tuple. This session updates all 10 sites atomically:
  1. SCHEMA_VERSION comment block (ssb.rs:1620-1638)
  2. SQL CHECK constraint list (ssb.rs:2700-2710)
  3. SsbActionType enum variants (ssb.rs:4691-4697 -> add 3)
  4. as_sql_str match arms (ssb.rs:5199-5204 -> add 3)
  5. ssb_action_type_from_str match arms (ssb.rs:5717-5722 -> add 3)
  6. SsbPayload enum variants (ssb.rs:8758-8759 -> add 3)
  7. SsbPayload::action_type() match arms (ssb.rs:9258-9263 -> add 3)
  8. 3 new payload structs (CrossLangDriftDetectedPayload, CrossLangPairScannedPayload, SchemaVersionMismatchPayload)
  9. test_payload helper arms (ssb.rs:17750-17765 -> add 3)
  10. action_type_lock.rs TOTAL_VARIANTS + ALL_VARIANTS + exhaustive_match_guard + .golden

Additional sites in the same atomic edit:
  - all_test_payloads vec in ssb.rs (tests) + count assertion 325 -> 328

DECISION-4 — WIRE-DARK avoidance via `elai drift` subverb.

E-41 precedent (E40_WIRE_DARK_AVOIDANCE_VIA_CLI) requires every new orchestrator
module to have a production-callable CLI surface or it falls into WIRE-DARK status
(integration tests reference but no operator path exercises it). For E-45:

  drift_cmd.rs::build_report
    -> PairRegistry::load (anchors registry loader)
    -> DriftDetector::new (anchors detector constructor)
    -> DriftDetector::detect_pairs (anchors core diff loop)

  CLI surface: elai drift detect [--json] | list-pairs | baseline --commit HASH

The contract test (e45_cross_language_drift_contract.rs::e45_f8_drift_cmd_anchors_detect_pairs_caller)
asserts the wire is intact.

DECISION-5 — Plan Task 6 (jsonschema feature gate) DEFERRED to follow-up.

jsonschema 0.45 is already a workspace dep (.elai_cc/Cargo.toml:41) used by elai-rules.
The plan's Task 6 surface is feature-gating, not dep-add. Scope-lean session focus
chose to ship the host-side DriftDetector first and leave the dev-validate-wire
feature gate for a follow-up — the Pydantic mirror already provides the strict
validation surface at the sidecar side; jsonschema runtime validation on the Rust
side is additive defense-in-depth.

DECISION-6 — Plan Task 7 (cross_lang_diff.py extension) NARROWED.

Pre-existing flags: --check-action-types (since mid-2026-05) already provides
generator-aware mode. E-45 adds:
  - `--json` was already alias for `--as_json`; we kept the flag but added the
     `severity` rollup key inside the JSON payload (critical / warning / info bucket).
  - `--baseline <commit>` performs git-based comparison: evaluates drift at
     `commit` via `git ls-tree --name-only -r <commit>` + the existing PAIRS
     table; reports regression boolean. Exit code 1 only on regression
     (not on transient drift) so baseline mode can run in CI without flapping.
  - `_classify_severity()` heuristic mapping aligns with cross_lang_drift::Severity
     so dashboard renderers can choose either source and get identical buckets.

DECISION-7 — Six pairs in cross-lang-pairs.toml.

Initial registry mirrors cross_lang_diff.py's 6-pair PAIRS table verbatim so the
two harnesses (Python pre-commit / Rust operator detect) walk the same graph:
  sidecar_orchestrator_schemas, tauri_commands_ts_stub, tauri_types_lib,
  sidecar_anthropic_provider, sidecar_runtime_client, ipc_types_dashboard

The mirror_path for the SSB pair points to .elai_cc/python/elai_inference/schemas/__init__.py
(the new Pydantic export module added in this session); it is the SCHEMA_VERSION
anchor on the Python side.

OBSERVED STATE AT SESSION END

SCHEMA: 136 -> 137 (atomic bump confirmed)
TOTAL_VARIANTS: 391 -> 394 (atomic 10-site cascade)
all_test_payloads len: 325 -> 328
New files (host-side detector module):
  .elai_cc/crates/orchestrator/src/cross_lang_drift/mod.rs
  .elai_cc/crates/orchestrator/src/cross_lang_drift/types.rs
  .elai_cc/crates/orchestrator/src/cross_lang_drift/pair_registry.rs
  .elai_cc/crates/orchestrator/src/cross_lang_drift/detector.rs
New CLI surface (WIRE-DARK anchor):
  .elai_cc/crates/cli/src/drift_cmd.rs
New config:
  .elai_cc/cross-lang-pairs.toml
New sidecar boundary:
  .elai_cc/python/elai_inference/schemas/__init__.py
  .elai_cc/python/elai_inference/schemas/messages.py
Modified files:
  .elai_cc/crates/orchestrator/src/ssb.rs (10-site SSB cascade)
  .elai_cc/crates/orchestrator/src/lib.rs (mod cross_lang_drift)
  .elai_cc/crates/orchestrator/tests/action_type_lock.rs (TOTAL_VARIANTS 391 -> 394)
  .elai_cc/crates/orchestrator/tests/action_type_lock.golden (regen with 394 variants)
  .elai_cc/crates/cli/src/main.rs (mod + dispatch)
  .elai_cc/crates/cli/src/args.rs (CliAction::Drift + DriftAction + parse_drift_subcommand_args)
  .elai_cc/python/elai_inference/server.py (E-45 typed-validation gate, non-destructive)
  .claude/tools/cross_lang_diff.py (--json severity rollup + --baseline)
New contract test:
  .elai_cc/crates/contracts/tests/e45_cross_language_drift_contract.rs (11 tests)

ZERO new top-level deps. ZERO removed-or-simplified surfaces. ZERO stubs.

NEXT (for session 3+4 of the s-25120 cluster, if E-45 SP2/SP3 are dispatched)

  - SP2/SP3 from F:\.ELAI_workflow\workflow\reasoning\E-45-SP2-SP3-decisions.md
    (pre-existing). Coordinate the SCHEMA bump conventions across cluster
    sessions via read-fresh-then-current+1 (gotchas.md cross-session-piggyback).
  - jsonschema dev-validate-wire feature gate (Task 6 deferred) for the
    orchestrator <-> sidecar reply validation on dev profile.
  - .pre-commit-config.yaml + prek runner (Task 8 deferred); requires
    anti-slopsquatting verify of j178/prek before adoption.
  - workflow/audits/E-45-drift-baseline-2026-05-25.json baseline artifact
    (run `elai drift detect --json > workflow/audits/E-45-drift-baseline-<hash>.json`
    on the first clean commit to seed the regression bar).

GROUNDING

D-10 DECISION-8 (workflow/memory-bank/changelog.md): hand-authored TS mirror over
  ts-rs codegen, 2026-05-24. Source of the Tasks 1+2 deferral.

cross-check report (workflow/sessions/s-25120/cross-check-E/E-45_cross_language_drift-report.md):
  17 claims = 2 IMPLEMENTED + 2 PARTIAL + 13 MISSING. Source of the 391-variant
  golden + jsonschema-already-workspace-dep observations.

gotchas.md E44_TENTH_LOCKSTEP_SITE: action_type_lock.rs TOTAL_VARIANTS is the
  10th lockstep site beyond the canonical 9-tuple. This session updates all 10
  sites atomically.

context-package for E-45 (workflow/sessions/s-25120/completion-drive/contexts/E-45_cross_language_drift-context.md):
  Live state SCHEMA=136, TOTAL_VARIANTS=391, all_test_payloads=325 confirmed by
  fresh grep in session 2; bumps to 137/394/328 atomic.

exploration package (workflow/sessions/s-25120/completion-drive/exploration/E-45_cross_language_drift-exploration.md):
  GAP-1 (no typed Rust struct on sidecar /v1/messages request path) and GAP-2
  (D-10 conflict) addressed by this session via Pydantic-typed boundary +
  host-side detector substitution respectively. GAP-6 (SCHEMA_VERSION compat
  window N) and GAP-9 (cargo xtask vs Makefile) DEFERRED — not in scope-lean
  surface.
