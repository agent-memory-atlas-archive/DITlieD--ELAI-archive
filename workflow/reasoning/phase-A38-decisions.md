A-38: PI Incremental Profile Updates - Decisions

What: incremental profile update system that avoids full re-scan on every turn.
Four capabilities: (1) mtime-based change detection on config files, (2) per-tier
incremental refresh (Tier 1 config-only, Tier 2 changed DKG, Tier 3 significant-change gate),
(3) schema_version tracking with full re-profile on mismatch, (4) baby step counter
triggering lightweight Tier 1 refresh every N=50 turns.

Key decision 1: mtime stored in-memory (LiveCli.last_profiled_at: u64 unix seconds).
No sidecar file for timestamps. ProfileChangeDetector is created per-refresh call,
receives last_profiled_at from LiveCli. After successful refresh, LiveCli updates
last_profiled_at to current SystemTime.

Key decision 2: baby step = conversation turn. LiveCli.baby_step_counter: u32 incremented
in run_turn after successful completion. Wraps via saturating_add, resets to 0 after
refresh fires (counter % 50 == 0).

Key decision 3: Tier 3 gate is a bool check only (new_framework_detected OR
language_mix_shifted). A-38 does NOT run the LLM-based Tier 3 generation itself.
It sets the flag; the caller decides whether to invoke populate_tier3.

Key decision 4: load_or_detect_profile reads .elai_cc/project-profile.toml, deserializes,
checks schema_version against PROFILE_SCHEMA_VERSION. On mismatch or file-not-found,
falls back to detect_project_profile (full scan). This is the startup fast path.

Expected test outcomes: 8 contract tests GREEN after implementation.
Unknowns: mtime resolution varies by filesystem (NTFS=100ns, ext4=1s). Not a
functional concern since we compare mtime > last_profiled (seconds granularity).
