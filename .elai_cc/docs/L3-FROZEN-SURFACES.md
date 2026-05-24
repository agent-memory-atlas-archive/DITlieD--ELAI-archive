L3 Frozen Surfaces — Operator Reference (D-9)

Overview

L3-locked surfaces are source positions the agent MUST NOT modify without
an explicit human override. They form the third tier of ELAI's access-
control layering:

  L1: convention (CLAUDE.md rule the agent reads at session start)
  L2: lint / hook (PreToolUse hook refuses the edit at tool time)
  L3: cryptographic gate (pre-commit + CI both verify against a signed
       registry; the agent cannot land the commit at all)

D-9 ships the L3 substrate. The auditor is invocable from the
`elai audit-locked-keys` CLI subcommand, surfaces violations via two
SSB rows (LockedKeysAuditRun + LockedKeysViolationDetected), and is
enforced at commit time by both a local pre-commit hook and a GitHub
Actions workflow.

Grounding

  arxiv:1908.04734 (Everitt et al., "Reward Tampering Problems and
    Solutions"). T1 source for the current-RF principle: the agent
    must not edit the function that grades its own work. L3 locks the
    scoring functions, gate definitions, and tripwire detectors so a
    misaligned policy cannot self-rewrite the supervisor.

  arxiv:1906.01820 (Hubinger et al., "Risks from Learned Optimization
    in Advanced Machine Learning Systems"). T1 source for
    mesa-optimization. Hard-coded constants ARE robust alignment by
    construction (no learned mesa-objective can drift them at runtime),
    but the corollary is that "freezing the wrong value is permanently
    wrong" — pre-freeze verification of every new locked surface is the
    operator's responsibility (see "Adding a new locked surface" below).

  arxiv:1905.12186 (Cohen, Hutter, "Boxed Myopic AI"; BoMAI).
    Supporting source for the principle that a sufficiently sandboxed
    agent with a frozen scoring function and no path to modify its own
    supervisor can be made asymptotically aligned. L3 is the practical
    enforcement of the "no path to modify the supervisor" half of that
    setup inside a code-editing agent.

  NOTE — citation disambiguation: an early draft of the D-9 plan body
  cited Cohen et al. as arxiv:1908.04734. That arXiv ID is in fact
  Everitt et al. Cohen's BoMAI is arxiv:1905.12186. See
  workflow/reasoning/D-9-decisions.md DECISION-3 for the full audit
  trail.

How LockedKey + LockedKeysRegistry work

The registry lives at `.elai_cc/scripts/locked_keys_registry.toml`. Each
entry is a TOML `[[locked_keys]]` table:

  path             Workspace-relative POSIX path. The auditor normalises
                   Windows backslashes before matching.
  line_range       Inclusive [start, end] line range. Omit or use
                   [0, 4294967295] for whole-file locks. (Whole-file is
                   the MVP CLI surface; sub-file ranges parse but are
                   only meaningful once a future B-N introduces a
                   symbol-graph-backed diff source.)
  reason           Human-readable explanation surfaced inline in the
                   pre-commit hook error so the operator knows what
                   they hit.
  owner_session    Session id that registered the lock. Empty string for
                   bootstrap-seed entries.
  freeze_ts_ms     UNIX ms of registration. Zero for bootstrap entries.
  signature_blake3 Blake3 hex of the canonical pre-image
                   `path|line_range|reason|owner_session|freeze_ts_ms`.
                   Stored as a 64-char hex string. Empty / absent round-
                   trips to all zeros for legacy registries.

Loaded by `elai_orchestrator::LockedKeysRegistry::load` (fail-closed:
any IO or parse error returns `LockedKeysAuditError::RegistryUnreadable`).
The auditor emits exactly one `LockedKeysAuditRun` SSB row per call
(regardless of outcome — the run row is the durable signal that the
audit happened) and one `LockedKeysViolationDetected` SSB row per
overlap.

The audit-run row carries a `registry_blake3` derived from the loaded
content. Mid-session mutation of the registry shows up as a hash drift
between two consecutive audit-run rows; dashboard consumers can alarm
on the drift.

Pre-commit + CI enforcement

Two entry points exercise the same `elai audit-locked-keys` CLI driver:

  Local pre-commit hook
    `.elai_cc/scripts/locked_keys_check.sh` is invoked by the workspace
    pre-commit hook (template in `install_pre_commit_hook.sh`). It cd's
    into `.elai_cc/`, locates the registry, and shells out
    `elai audit-locked-keys --staged --json`. Exit 0 = clean; exit 1 =
    violation OR registry-unreadable (fail-closed).

    Bypass: `git commit --no-verify` skips the hook. Use this only with
    a documented L3_OVERRIDE; the override convention itself is a
    separate plan (D-9 plan task 7).

  GitHub Actions
    `.github/workflows/locked-keys-check.yml` runs on every push and
    pull request. It computes the diff between the PR head and the
    base ref (or HEAD~1 for direct pushes), then runs
    `elai audit-locked-keys --files <diff>` with the comma-separated
    file list. Exit 1 fails the step.

Both invocations emit the same JSON body (under `--json`) so log-
parsing tools and the dashboard's SSB consumer see identical schemas
from either entry point.

How to add a new locked surface

PRE-FREEZE VERIFICATION (mandatory per arxiv:1906.01820 mesa-
optimization corollary): before adding any new entry, confirm that the
surface is actually the right thing to freeze. Mistakes are
permanent — the registry chain hashes them in. Specifically:

  1. Confirm the path is the canonical home for the value/function.
     A locked symbol that lives in two places defeats the freeze.

  2. Confirm the value is correct AT THIS MOMENT. L3 says "this stays
     as-is forever"; if you have any uncertainty, hold off.

  3. Confirm the auditor itself, the freeze infrastructure, and the
     override convention are reachable. A locked surface inside a
     broken supervisor is worse than no lock.

When the three checks pass, append a new `[[locked_keys]]` entry to
`.elai_cc/scripts/locked_keys_registry.toml`. Populate `path`, `reason`,
`owner_session` (your session id), and `freeze_ts_ms` (current UNIX
ms). Leave `signature_blake3 = ""` for the load to default-zero; the
auditor recomputes the signature on read.

Then commit. The pre-commit hook will run against the registry change
itself — if the registry is in the locked-paths list (it is not, by
default), the commit fails and you have a bootstrap problem to resolve
separately.

Why the bootstrap registry ships EMPTY

The registry at HEAD is intentionally empty. The reason is operational:
populating it during the D-9 orchestrate run itself would block
orchestrate's own commits via the pre-commit hook (the hook fires on
every commit including those that introduce locked surfaces, so the
first commit-with-locks would lock orchestrate out of committing the
following plan's edits). The first live entries were added in a
follow-up patch under D-10 once the surface was stable and the
override convention was documented.

Operators wanting to seed their own locks should do so AFTER the
substrate exists, as part of a dedicated patch (not mixed into a
feature plan). See `workflow/reasoning/D-9-decisions.md` DECISION-5.

Recovery: a legitimate edit needs to touch a locked surface

Scenario: a real bug exists inside a locked path, or an authorised
refactor needs to move the locked symbol. The workflow:

  1. Open a TICKET (in `workflow/reasoning/` or the issue tracker, your
     team's convention) explaining what change is needed and why the
     L3 lock is the wrong scope or wrong value.

  2. Get a second-pair-of-eyes review. The lock exists because an
     earlier reviewer judged the surface to be safety-critical;
     overriding it needs another reviewer's signature.

  3. With the override approved, the operator may either:

       (a) `git commit --no-verify` the corrective edit, reference the
           L3_OVERRIDE ticket id in the commit message body. CI still
           runs the audit and will fail the PR; the reviewer merges
           with a documented override note explaining the bypass.

       (b) Edit the registry entry first: change the locked path,
           remove the entry, or tighten the `line_range` so the new
           edit no longer overlaps. Commit the registry change first
           (it is not itself locked), then commit the corrective edit
           normally.

  Option (b) is preferred for permanent changes (the lock should
  reflect the new reality). Option (a) is for one-off authorised
  edits where the locked surface should still be locked after.

CLI quick reference

  elai audit-locked-keys --staged --json
      Audit the currently staged files using the default registry.

  elai audit-locked-keys --files crates/orchestrator/src/tripwire.rs --json
      Audit an explicit file list.

  elai audit-locked-keys --registry /alt/registry.toml --staged
      Use a non-default registry path (useful for testing).

JSON body shape (always emitted with `--json`):

  {
    "ok": true | false,
    "registry_path": "<absolute path>",
    "n_audited": <u32>,
    "n_violations": <u32>,
    "violations": [
      {
        "attempted_edit": "<path>",
        "attempted_lines": [<u32>, <u32>],
        "locked_key_path": "<path>",
        "reason": "<string>"
      }
    ]
  }

References

  Source:   .elai_cc/crates/orchestrator/src/locked_keys_auditor.rs
  CLI:      .elai_cc/crates/cli/src/audit_locked_keys.rs
  Hook:     .elai_cc/scripts/locked_keys_check.sh
  CI:       .github/workflows/locked-keys-check.yml
  Registry: .elai_cc/scripts/locked_keys_registry.toml
  Tests:    .elai_cc/crates/orchestrator/tests/locked_keys_integration.rs
            .elai_cc/crates/orchestrator/tests/locked_keys_observable_smoke.rs
  Plan:     docs/thefinalglance/D-plan/D-9_l3_frozen_surfaces.md
  Decisions: workflow/reasoning/D-9-decisions.md
