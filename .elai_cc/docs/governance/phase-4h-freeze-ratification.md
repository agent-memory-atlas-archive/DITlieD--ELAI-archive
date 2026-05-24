Phase 4H scope-freeze ratification

C-43 governance document. Records the in-scope, frozen, and out-of-scope
items at the Phase 4H boundary, the ratification cadence, and the
mechanism by which violations are surfaced.

This file is intentionally prose-only. The machine-readable manifest is
.elai_cc/scope-freeze.toml; the Rust enforcer is
crates/orchestrator/src/scope_freeze.rs; the CLI surface is
`elai freeze status|verify|ratify`. The Tauri dashboard widget is
crates/dashboard-v2/src/lib/widgets/FreezeStatusBar.svelte.

Grounding posture

Scope-freeze enforcement is an ELAI design decision. No prior-art
grounding was found at the C-43 session-2 close. The source plan cited
arXiv:2604.21725 as ground truth for this mechanism; two independent
WebFetches verified that paper is actually "Agent Evolving Learning for
Open-Ended Environments" (Thompson Sampling agent learning, not
scope-freeze). The citation has been removed across all C-43
deliverables — see workflow/sessions/s-25120/mistakes.md for the
bad-cite analysis. Treat every "AEL"-flavored claim that survives in
earlier drafts as ungrounded conjecture pending re-ingest.

In-scope items (covered by the Phase 4H ratification)

The 3-layer self-modification surface from Phase 4H ships these
modules. They are eligible to be frozen by a ratification event:

  crates/orchestrator/src/self_mod_orchestrator.rs (Layer 3 driver)
  crates/orchestrator/src/self_mod_safety.rs (MSI auto-revert constants)
  crates/orchestrator/src/self_mod_layer1a.rs (Layer 1A allowlist)
  crates/orchestrator/src/self_mod_types.rs (shared types)
  crates/orchestrator/src/self_mod_bridge.rs (Class A bridge)
  crates/orchestrator/src/self_mod_proposal.rs (Class B proposals)
  crates/orchestrator/src/msi.rs (MSI tracker)
  crates/orchestrator/src/evolution_gate.rs (similarity gate)
  crates/orchestrator/src/config_archive.rs (revert archive)

Frozen items (post-ratification immutable)

Every path in scope-freeze.toml `frozen_items`. Modifying a frozen item
without first running `elai freeze ratify` is a contract breach. The
verify pass walks the manifest; any item whose on-disk mtime is later
than `ratification_ts_ms` produces a ScopeFreezeViolated SSB row.

CI gates the breach by enabling the orchestrator's
`phase_4h_freeze_violated` Cargo feature; the
`#[cfg(feature = "phase_4h_freeze_violated")] compile_error!` in
scope_freeze.rs causes the build to fail with a pointer back to this
document. No nightly toolchain is required.

Out-of-scope items (deferred to Phase 5)

Phase 5 self-modification is out of scope for the C-43 ratification.
The following are explicitly NOT covered by the current freeze and may
be mutated freely:

  crates/orchestrator/src/scope_freeze.rs (the enforcer itself)
  crates/cli/src/freeze_cmd.rs (the CLI handler)
  crates/dashboard-v2/src/lib/widgets/FreezeStatusBar.svelte
  workflow/sessions/<sid>/mistakes.md (audit ledger)

Ratification cadence

One ratification event per phase boundary. A ratification writes
scope-freeze.toml atomically (write-temp-then-rename) and emits exactly
one ScopeFreezeRatified SSB row carrying the manifest blake3 hash, the
ratifier id hash, the item count, and the wall-clock timestamp. The
dashboard FreezeStatusBar reads the most recent ratification to render
the green/red badge.

A re-ratification (e.g. expanding the frozen set at the Phase 5 cutover)
emits a fresh ScopeFreezeRatified row; readers MUST sort by ts_ms and
take the latest. There is no in-place "amendment" — every ratification
event overwrites the manifest in full.

Failure modes

  Frozen item missing on disk:
    verify_freeze emits a ScopeFreezeViolated row with
    modified_at_ms = 0 to flag the absence. The CI feature flag fires
    in the next build.

  Frozen item mtime newer than ratification_ts_ms:
    verify_freeze emits a ScopeFreezeViolated row carrying the
    item_path_hash, frozen_since_ms, and modified_at_ms. The
    dashboard surfaces the violation in the FreezeStatusBar badge.

  scope-freeze.toml missing or malformed:
    verify_freeze surfaces a typed FreezeError; the CLI prints the
    error and exits non-zero. No SSB row is emitted (the manifest is
    the source of truth — a missing manifest is a configuration
    failure, not a freeze violation).

Footnote
  Scope-freeze enforcement is an ELAI design decision; mistakes.md
  logs the bad-cite analysis for arXiv:2604.21725.
