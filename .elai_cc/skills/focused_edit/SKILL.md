---
name: focused_edit
type: application
version: 1.0.0
description: Make a targeted edit to a known file or function with no planning phase.
triggers: edit, fix, patch, update, change, modify, rename, replace, tweak, adjust
inputs: file_path, change_description
outputs: edit_diff, audit_result
scope: execution
tags: [edit, single-file, fast-path, no-decomposition]
estimated_loc: 50
estimated_minutes: 5
---

Application skill — type=application. Used by the orchestrator pre-classify
hook to route targeted single-file edits past the full decomposition pipeline.

Use when:
  - the user names a specific file, function, or symbol to change
  - the change is bounded (renames, typos, single-statement fixes, one-line config)
  - blast radius is small (no cross-language pair, no public API drift)
  - tests for the touched area already exist

Do NOT use when the change ripples across crates, requires a new dependency,
introduces a new public type, or the user asks for a "feature" rather than a fix.

Output: an `Edit` tool call producing a unified diff, followed by a scoped
auditor run (`cargo check` for the touched crate, no full workspace tests).
