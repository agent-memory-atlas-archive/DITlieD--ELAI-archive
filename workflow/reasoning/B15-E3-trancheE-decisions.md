B-15 Tranche E Item E.3 -- Dashboard BTreeMap DTO migration (Rust side)
Session: s-22600 sub-phase trancheE | Date: 2026-05-31

Scope
- 4 IPC-wire DTO fields migrated from FxHashMap to BTreeMap (key type `String` everywhere; std::collections::BTreeMap is stdlib, ZERO new deps).
- Rust-side only; Svelte UI deliberately untouched per task brief (followup logged below).

Files modified (per ctx-compiler verified line ranges)
- .elai_cc/crates/dashboard-v2/src-tauri/src/commands_d6.rs
  - HeldOutStatsRow::by_source: FxHashMap<String, u32> -> BTreeMap<String, u32>
  - empty-default branch + local accumulator in get_held_out_stats migrated
  - `use rustc_hash::FxHashMap` REMOVED (no remaining uses); `use std::collections::BTreeMap` ADDED
- .elai_cc/crates/dashboard-v2/src-tauri/src/commands_d7.rs
  - CanaryBreakdown::by_report: FxHashMap<String, Vec<CanaryFamilyRow>> -> BTreeMap<String, Vec<CanaryFamilyRow>>
  - empty-default branch + local accumulator in get_canary_per_family_breakdown migrated
  - `use rustc_hash::FxHashMap` REMOVED; `use std::collections::BTreeMap` ADDED
- .elai_cc/crates/dashboard-v2/src-tauri/src/commands_d9.rs
  - LockedKeysRecentAudits::violations_by_audit: FxHashMap<String, Vec<LockedKeysViolationDto>> -> BTreeMap<...>
  - empty-default branch + local accumulator in get_locked_keys_recent_audits migrated
  - `use rustc_hash::FxHashMap` REMOVED; `use std::collections::BTreeMap` ADDED
- .elai_cc/crates/dashboard-v2/src-tauri/src/commands_e46.rs
  - GoalDriftStatusDto::firings_by_canary: FxHashMap<String, u32> -> BTreeMap<String, u32>
  - empty-default branch + local `by_canary` accumulator in get_goal_drift_status migrated
  - `use rustc_hash::FxHashMap` PRESERVED (still used by local `latest: FxHashMap<String, DualLedgerTaskDto>` @L110 -- per W3 a lookup-only intermediate, NOT a wire DTO field; out of scope per B15_DETERMINISM_MAP_TYPES gotcha rule)
  - `use std::collections::BTreeMap` ADDED

Why BTreeMap
- Per gotchas.md @L1159-1164 (B15_DETERMINISM_MAP_TYPES): serialization-observed maps need iteration determinism. The Tranche C-1 FxHashMap migration was the safe-no-regression mechanical step; BTreeMap is the canonical end-state for these 4 fields because each one is serialized via serde_json across a Tauri command boundary and consumed by Svelte panels.
- Per s-22300 stubs.md B15-dashboard-BTreeMap row: "longer-term right answer is BTreeMap so the Svelte side gets stable alphabetical list ordering."
- Per ctx-compiler verification: BTreeMap<String, T> serializes identically to FxHashMap<String, T> via serde_json (both emit a JSON object); wire compat PRESERVED. The only externally-observable change is iteration order becoming key-sorted alphabetical (the intended determinism improvement).

Why not Svelte side
- Out of mechanical scope per task brief (DE8).
- HeldOutPanel.svelte:106 already does `.sort((a, b) => b[1] - a[1])` (value-DESC sort); BTreeMap key-order is orthogonal -- ZERO observable change, NOT a regression.
- CanaryPanel.svelte:107 + LockedKeysPanel.svelte:114 use direct keyed lookup; ZERO iteration.
- PlanningLedgerPanel.svelte:69 init-only.

Open followup (deferred to dashboard owner sign-off)
- HeldOutPanel.svelte:106 client-side sort by value-DESC is independent of key order; the sort can stay if the panel intent is "show highest counts first" (it likely should). If the dashboard owner decides alphabetical-by-source is the right intent, the client-side sort can drop.
- Any panel that later adds `.entries()` / `for...of` iteration must know the iteration semantic is now BTreeMap-alphabetical (key-sorted ascending). Mark on the s-22300 stubs.md row when closing.

Locked decisions referenced
- DE7 / DE8 from F:\.ELAI_workflow\workflow\reasoning\B15-trancheE-decisions.md (overall Tranche E rationale doc).

Build verdict (auditor-deferred)
- `cargo check -p elai-dashboard-v2-tauri` run after the 4-file batch; auditor runs full `cargo test` in AUDITING substate.
