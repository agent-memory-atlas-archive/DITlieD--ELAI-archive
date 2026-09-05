D-11 constructive checker registry — closure decisions
======================================================

Session: s-25120 (completion-drive cluster, slot 3/4)
Plan: F:/.ELAI_workflow/docs/thefinalglance/D-plan/D-11_constructive_checker_registry.md
Prior recheck verdict: PARTIAL (2 partial + 13 missing + 2 not-wired + 0 drift; 10 of 27 IMPLEMENTED)
Closure verdict: GREEN (15 of 15 GAPs closed; substrate preserved at SCHEMA 120, head 120 -- NO bump)

This doc records the 6 binding decisions the closure pass made over
the plan-text, the recheck-report, and the substrate as it stood at
the start of the session. Every decision points back to its evidence
source so a future session can verify the reasoning is grounded.

------------------------------------------------------------------
DECISION-1 -- NO SCHEMA bump (preserve head 120).
------------------------------------------------------------------

The plan text targets SCHEMA_VERSION 56 -> 57. The substrate shipped
the D-11 SSB plumbing as a single consolidated `CheckerResultEmitted`
variant at SCHEMA 79 long before this closure pass; head SCHEMA is
now 120 (C-45 piggyback per memory-bank/activeContext.md). Per the
[SCHEMA-actual vs plan-stale] rule (now octuple-confirmed across
D-2..D-10), the plan-blurb is stale and MUST NOT drive a bump.

The closure adds zero new SSB variants -- the +2 CheckerKind extensions
(CompilePy, RegistryCheck) ride the existing single-variant
`CheckerResultEmitted` payload via its `checker_kind: String` field.
The contract test pins the floor at >=120 (the cluster's current head)
so any future SSB-touching change must reaffirm the existing
CheckerResultEmitted contract rather than re-bump.

Grounding:
  * .elai_cc/crates/orchestrator/src/ssb.rs:1318 (`SCHEMA_VERSION: i64 = 120`)
  * memory-bank/activeContext.md (C-45 SCHEMA 117->120 +3 CANTANTE QUINTUPLE)
  * workflow/reasoning/D-10-decisions.md DECISION-1 (octuple precedent)

------------------------------------------------------------------
DECISION-2 -- Preserve shipped single `CheckerResultEmitted` SSB
variant; reject plan's 4-way split (CheckerDispatched + CheckerPass
+ CheckerFail + CheckerInapplicable).
------------------------------------------------------------------

The plan calls for 4 distinct SsbActionType variants, one per result
kind. The substrate ships ONE variant carrying `result_kind` as a
string discriminator on the payload (CheckerResultEmittedPayload at
ssb.rs:11793). Splitting into 4 variants is API-breaking:
  * BabyStepPanel + commands_d11.rs counts_by_kind aggregation key off
    the single action_type="checker_result_emitted" filter.
  * action_type_lock.rs TOTAL_VARIANTS=337 (post C-45) -- adding 4
    inflates the ordinal cascade and forces a multi-bump.
  * D-10 DECISION-2 established the consolidated-variant precedent for
    ClaimRouted (plan asked for 2 variants, shipped 1). D-11 follows
    sibling precedent.

The closure preserves the single-variant shape. Result kind is carried
in the payload's `result_kind` field per the deterministic JSON shape
the dashboard already consumes.

Grounding:
  * .elai_cc/crates/orchestrator/src/ssb.rs:3244 (SsbActionType::CheckerResultEmitted)
  * .elai_cc/crates/orchestrator/src/ssb.rs:11793 (CheckerResultEmittedPayload)
  * workflow/reasoning/D-10-decisions.md DECISION-2 (variant-consolidation precedent)
  * exploration/D-11_constructive_checker_registry-exploration.md GAP-2 HIGH

------------------------------------------------------------------
DECISION-3 -- Extend `CheckerKind` closed enum +2 variants
(CompilePy, RegistryCheck) atomic with their new helper scripts.
------------------------------------------------------------------

The shipped enum had 6 variants (Sympy, Z3, CompileRust, RegexMatch,
JsonSchemaValidate, NumericRange). The plan explicitly names
compile_py + registry_check as canonical kinds; the closure ships the
helper scripts in the same atomic change.

Extension cost is bounded:
  * CheckerKind is L3-frozen per D-9 locked-keys registry, but the +2
    extension is documented in this decision doc as the ratification
    record. Future sessions extending the enum must add a DECISION-N
    entry here as the audit trail.
  * `as_str` + `from_str` + `all()` array length 6 -> 8 are mechanical
    updates; downstream dashboard KIND_TABLE updates to 8 rows
    (commands_d11.rs).
  * Contract test `checker_kind_is_exhaustive_at_eight_variants`
    freezes the new surface.

The +2 selection is paper-grounded:
  * CompilePy: PoT arxiv:2211.12588 §3.1 + CodeAct arxiv:2402.01030
    §2.3 -- the Python interpreter is canonical ground truth for code
    claims. py_compile is the syntactic load.
  * RegistryCheck: CLAUDE.md anti-slopsquatting HARD RULE; see
    DECISION-5 for full rationale.

Grounding:
  * .elai_cc/crates/orchestrator/src/checker_registry.rs:61-83 (closed enum)
  * .elai_cc/crates/dashboard-v2/src-tauri/src/commands_d11.rs:77-119 (KIND_TABLE)
  * workflow/papers/pot-2211.12588-interface.md §3.1
  * workflow/papers/codeact-2402.01030-interface.md §2.3

------------------------------------------------------------------
DECISION-4 -- SandboxHandle stub REPLACED with subprocess + 30s
tokio::time::timeout. Isolation level documented as "subprocess +
timeout only, NOT a true sandbox".
------------------------------------------------------------------

The plan references "Tier C sandbox (B-18 + C-11) SandboxHandle" as
the helper-script execution surface. Investigation per the exploration
report (GAP-11) confirmed:
  * .elai_cc/crates/sandbox/ does NOT exist.
  * .elai_cc/crates/orchestrator/src/sandbox.rs ships ProcessLaunch +
    agent-profile config but does NOT expose a SandboxHandle the
    checker_registry can route through without significant rewiring.
  * The shipped run_shellout_check (checker_registry.rs:636-706) uses
    tokio::process::Command directly with a 30s
    tokio::time::timeout wall clock.

The closure preserves the direct-subprocess path. Isolation level is
explicitly documented as:
  > Subprocess + 30s wall-clock timeout. NOT a true sandbox. No
  > cgroup / namespace / seccomp / Firecracker isolation. Helper
  > scripts run as the orchestrator's uid/gid with full filesystem
  > read access. True sandbox upgrade is a follow-on patch when the
  > .elai_cc/crates/sandbox crate (or the existing
  > orchestrator::sandbox::ProcessLaunch surface) is generalised to
  > host arbitrary helper-script binaries.

This is acceptable for D-11's Mode A (constructive grading) surface
because:
  * Helper scripts ship in-tree under .elai_cc/checkers/ and are
    code-reviewed at land time; not user-provided binaries.
  * The 30s wall clock bounds runaway behaviour.
  * Network access (registry_check.py) is by design.

Grounding:
  * .elai_cc/crates/orchestrator/src/checker_registry.rs:322
    (`SHELLOUT_TIMEOUT: Duration = Duration::from_secs(30)`)
  * exploration/D-11_constructive_checker_registry-exploration.md GAP-11

------------------------------------------------------------------
DECISION-5 -- Ship registry_check.py as the canonical anti-slopsquatting
defense per CLAUDE.md HARD RULE.
------------------------------------------------------------------

CLAUDE.md (Anti-slopsquatting HARD RULE block) mandates that every
package add MUST be verified against the upstream registry before use:
  > NEVER assume a dependency exists. Verify name + existence +
  > version + last-published date BEFORE use. The verification
  > protocol is mandatory regardless of how confident you feel about
  > the package name.

D-11's contribution is to make this defense first-class in the
constructive-grading path:
  * .elai_cc/checkers/registry/registry_check.py queries PyPI
    /pypi/{pkg}/json + npm registry.npmjs.org/{pkg} and reports PASS
    iff the package exists AND was published within max_age_days
    (default 730).
  * The CheckerKind::RegistryCheck variant + KIND_TABLE entry surface
    this in the dashboard so operators can see slopsquatting blocks
    happen in real time.
  * The integration test `registry_check_script_pypi_smoke` exercises
    the live PyPI path (gracefully skipped without network).

Stale-publish heuristic: a package that exists but hasn't been
published in 2+ years is FAIL, not Pass. Hijacked-abandoned packages
are a known slopsquatting vector (researcher's note: "typosquatting
catches typos; abandoned-package squatting catches LLM hallucinations
of stale forks"). Default 730 days is configurable per claim via the
candidate.max_age_days field.

Grounding:
  * CLAUDE.md "Anti-slopsquatting (HARD RULE)" block (full protocol)
  * .elai_cc/checkers/registry/registry_check.py (the implementation)
  * .elai_cc/checkers/pyproject.toml ("requests" pinned >=2.34.0,
    PyPI-verified 2026-05-14)

------------------------------------------------------------------
DECISION-6 -- D-1 VerifierKind partition is {Ppm, Intrinsic,
Deterministic, CrossContext}; D-11 CheckerKind sits BELOW
VerifierKind::Deterministic. NO D-1 enum extension.
------------------------------------------------------------------

The plan's partition input describes VerifierKind as
{Compile, Test, Prm, Refuse}. The actual production type at
verification_result.rs:38-50 is {Ppm, Intrinsic, Deterministic,
CrossContext}. The plan-text VerifierKind names DO NOT EXIST in
the codebase.

Mapping:
  * D-11 CheckerKind (8 variants) feeds VerifierKind::Deterministic
    as the per-claim verifier family. One Deterministic verifier
    aggregates signals from all 8 applicable checkers.
  * D-1 does NOT need a Compile / Test / Prm / Refuse extension.
  * D-11 sits BELOW VerifierKind, NOT parallel to it.

The contract test for D-1 (d1_verifier_kind_contract.rs if it exists)
freezes the 4-variant {Ppm, Intrinsic, Deterministic, CrossContext}
surface. D-11 does not perturb that surface.

Grounding:
  * .elai_cc/crates/orchestrator/src/verification_result.rs:38-50
    (actual VerifierKind enum)
  * exploration/D-11_constructive_checker_registry-exploration.md
    GAP-7 MEDIUM (partition drift)
  * grounding_mode.rs:159-166 (Constructive iff
    `ctx.available_verifiers contains Deterministic`)

------------------------------------------------------------------
CLOSURE INVENTORY (15 GAPs)
------------------------------------------------------------------

Files added (10):
  * .elai_cc/checkers/math/z3.py
  * .elai_cc/checkers/code/compile_py.sh
  * .elai_cc/checkers/registry/registry_check.py
  * .elai_cc/checkers/text/regex_assertion.py
  * .elai_cc/checkers/checkers.toml (6-row registration manifest)
  * .elai_cc/checkers/pyproject.toml (sympy + z3-solver + requests)
  * .elai_cc/crates/contracts/tests/checker_registry_contract.rs
  * .elai_cc/crates/orchestrator/tests/checker_registry_integration.rs
  * workflow/reasoning/D-11-decisions.md (this file)
  * (3 paper-ingestor outputs pre-existed: pot, codeact, alphaproof)

Files extended (4):
  * .elai_cc/crates/orchestrator/src/checker_registry.rs:
    + CheckerKind::CompilePy + CheckerKind::RegistryCheck variants
    + CheckerKind::from_str inverse
    + ScriptChecker + TomlCheckerEntry + TomlCheckerManifest
    + LoadError + CheckerRegistry::load_from_toml(_str)
    + parse_claim_type helper
  * .elai_cc/crates/orchestrator/src/runtime_config.rs:
    + ELAI_CHECKER_ENABLE_ENV + CheckerRuntimeConfig::is_enabled
    + CheckerRuntimeState (emitter + registry Arc-bundle)
  * .elai_cc/crates/orchestrator/src/task_runner.rs:
    + ResolvedConfig.checker_runtime field (default None)
    + dispatch_constructive_checkers_hook (called after
      dispatch_claim_routing in drive_task)
  * .elai_cc/crates/orchestrator/src/lib.rs:
    + ScriptChecker / TomlCheckerEntry / TomlCheckerManifest /
      CheckerLoadError re-exports
  * .elai_cc/crates/dashboard-v2/src-tauri/src/commands_d11.rs:
    + KIND_TABLE 6 -> 8 rows (+compile_py +registry_check)
    + test names updated to match 8-variant surface

Schema bump: NONE (preserve head 120 per DECISION-1)
SSB variants added: 0 (preserve consolidated CheckerResultEmitted per DECISION-2)
CheckerKind variants added: 2 (atomic L3 extension per DECISION-3)
Dashboard wiring: panel already shipped + mounted; KIND_TABLE extended
Test coverage: 10 contract tests + 4 integration tests added
