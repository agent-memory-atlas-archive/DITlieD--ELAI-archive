D-9 — L3 Frozen Surfaces decisions

Session: s-25120 (completion-drive close-out of D-9 gaps)
Prior session that shipped the substrate: s-62376
Schema at substrate ship: 75
Schema at this completion drive: 117 (head)

DECISION-1 — NO SCHEMA bump
Rationale: D-9 substrate (LockedKey + LockedKeysRegistry +
LockedKeysAuditor + commands_d9.rs Tauri commands + LockedKeysPanel.svelte
+ 2 SSB variants LockedKeysAuditRun + LockedKeysViolationDetected) was
shipped at SCHEMA 75 by prior session s-62376. The current head is
already at SCHEMA 117. This completion drive closes tooling gaps only
(pre-commit script + CI workflow + integration test + docs +
decisions); no public SSB / IPC type was added or modified, so no
SCHEMA bump is required.

DECISION-2 — Preserve shipped SSB variant names
The D-9 plan body proposed `LockedKeysTouch` / `L3OverrideRequested` /
`LockedKeysTouchEvent` as candidate SSB variant names. The shipped
substrate uses the richer "audit-run beacon" semantic:
`LockedKeysAuditRun` (fired once per audit invocation regardless of
outcome) + `LockedKeysViolationDetected` (fired once per actual
overlap). The shipped pair captures both the "did the gate run at all"
question and the "what did it find" question, which the plan's single-
event names did not. Rename would be a breaking change for the already-
mounted dashboard consumer and the existing observable_effect smoke
test. Preserve the shipped semantic; document the plan / ship name
divergence here so future reviewers don't think the plan was skipped.

DECISION-3 — Cohen "Boxed Myopic" cite disambiguation
The D-9 plan body line 37 cited Cohen et al. "Boxed Myopic AI" as
arxiv:1908.04734. That arXiv ID is in fact Everitt et al. ("Reward
Tampering Problems and Solutions"). Cohen's BoMAI / "Asymptotically
Unambitious Artificial General Intelligence" is arxiv:1905.12186.
Both papers belong in the grounding set for D-9 (different angles on
the same alignment property), but the IDs were swapped in the plan.

Verified resolution (s-25120):
  - 1908.04734 = Everitt — reward tampering / current-RF principle.
  - 1906.01820 = Hubinger — mesa-optimization risks.
  - 1905.12186 = Cohen — BoMAI / asymptotically unambitious AGI.

All three are now documented in `.elai_cc/docs/L3-FROZEN-SURFACES.md`
with the correct attributions. The paper-ingestor outputs in
`workflow/papers/` also use the correct IDs.

DECISION-4 — Concrete struct, not a FrozenSurface trait
The shipped substrate uses a `LockedKey` struct + a `LockedKeysRegistry`
container struct, not a `trait FrozenSurface { ... }` abstraction.
Reasoning:

  (a) D-5 free-fn+struct precedent: the wider crate prefers concrete
      types over traits when the surface has a single canonical
      implementation. The locked-keys auditor has exactly one
      production implementation; a trait would add cost (dyn dispatch
      or generic monomorphisation) for zero abstraction value.
  (b) The serde round trip is simpler with a concrete struct: derive
      Serialize/Deserialize directly, no trait-object serialisation
      dance.
  (c) Test substitution is handled by the `LockedKeysSink` enum
      (Channel | Capture), which IS the right abstraction boundary —
      tests inject a Capture sink and inspect the buffer; production
      uses the Channel sink against the mpsc bus.

DECISION-5 — Bootstrap registry stays EMPTY by design
The shipped registry at `.elai_cc/scripts/locked_keys_registry.toml`
contains zero `[[locked_keys]]` entries. Populating it during the D-9
orchestrate run itself would block orchestrate's own commits via the
pre-commit hook (the hook fires on every commit; the first commit-
with-locks would lock orchestrate out of committing the following
plan's edits to the locked surfaces). The first live entries were
added in a follow-up patch under D-10 once the substrate had been
verified.

Operators wanting to seed their own locks should do so AFTER the
substrate is live, as a dedicated patch (not mixed into a feature
plan that may itself need to edit the soon-to-be-locked surfaces).

DECISION-6 — Three-type model collapsed to one
The plan proposed three locked-key types: `LockedPath`, `LockedSymbol`,
`LockedTomlKey`. The shipped substrate collapses these into a single
`LockedKey` struct with `path` + optional `line_range`. Rationale:

  (a) The MVP CLI surface stages whole-file diffs only (git diff
      --cached --name-only); a per-symbol freeze requires a symbol-
      graph-backed diff source that does not yet exist.
  (b) `LockedTomlKey` semantics are subsumed by a path-lock on the
      TOML file plus a `line_range` covering the locked key's lines
      (when sub-file ranges become useful).
  (c) Premature trifurcation forces every downstream consumer
      (dashboard, CLI, hook, CI) to handle three enum variants where
      one would do. The shipped single-struct shape can grow into the
      three-variant shape later if a real need emerges (open / closed
      principle).

DECISION-7 — Operator responsibility for pre-freeze verification
Per arxiv:1906.01820 (Hubinger mesa-optimization), hard-coded
constants ARE robust alignment by construction (no learned mesa-
objective can drift them at runtime). The corollary is that freezing
the wrong value is permanently wrong, because the chain hash means the
mistake is reflected in every subsequent audit-run row's
registry_blake3.

This shifts the cost of correctness from runtime to commit time, which
is the right tradeoff for safety-critical surfaces, but the
implication for the operator is non-trivial: pre-freeze verification
is mandatory. The three checks documented in
`.elai_cc/docs/L3-FROZEN-SURFACES.md` (canonical home, current value
correctness, supervisor reachability) are the minimum bar.

References

Plan: docs/thefinalglance/D-plan/D-9_l3_frozen_surfaces.md
Substrate: .elai_cc/crates/orchestrator/src/locked_keys_auditor.rs
Tauri commands: .elai_cc/crates/dashboard-v2/src-tauri/src/commands_d9.rs
Dashboard panel: .elai_cc/crates/dashboard-v2/ui/src/lib/drawers/LockedKeysPanel.svelte
Pre-commit script (this drive): .elai_cc/scripts/locked_keys_check.sh
CI workflow (this drive): .github/workflows/locked-keys-check.yml
Integration test (this drive): .elai_cc/crates/orchestrator/tests/locked_keys_integration.rs
Docs (this drive): .elai_cc/docs/L3-FROZEN-SURFACES.md

Papers ingested (verified):
  workflow/papers/everitt-reward-tampering-1908.04734-interface.md
  workflow/papers/hubinger-mesa-1906.01820-interface.md
  workflow/papers/cohen-boxed-myopic-interface.md
