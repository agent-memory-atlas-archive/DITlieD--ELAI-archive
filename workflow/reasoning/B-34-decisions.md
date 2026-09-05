B-34 Entry-Point Detection — Phase Decisions
Session: s-32396 | Date: 2026-05-16

DECISION-B34-1: Framework HTTP verb set
Source: SocratiCode graph-entrypoints.ts:108-114 (verified verbatim).
Set: {get, post, put, delete, patch, head, options, all, use, route}.
Reason: parity with SocratiCode reference implementation. "route" -> "framework:route"
(distinct from HTTP verbs). Case-insensitive matching (callee_name.to_ascii_lowercase()).

DECISION-B34-2: Test-name regex anchors (char-by-char, no regex crate)
Patterns (from SocratiCode graph-entrypoints.ts:129-133):
  ^test[_A-Z]  — starts with "test", 5th char must be '_' or uppercase ASCII.
  ^Test[A-Z_]  — starts with "Test", 5th char must be uppercase ASCII or '_'.
  exact "it" | exact "describe"
Implementation: chars().collect::<Vec<_>>() index check. No regex crate dep.
Minimum length: 5 chars for test/Test patterns (4-char prefix + 1 discriminating char).

DECISION-B34-3: Payload selectivity (orphan files + filename-pattern matches)
Per B-29 lazy-loading invariant: must NOT load all payloads.
Strategy: load reverse_index once (ensure_reverse_index, all 256 shards, ~5ms).
Build all-files set from reverse_index keys UNION values (cheaper than name_index).
Candidate files = orphan_candidates (caller_count == 0) OR filename-pattern matches.
Filename stems checked: {main, index, __main__, Main} (case-sensitive on file path stem).
Expected payload load rate: < 5% of files per B-34 spec.

DECISION-B34-4: default include_tests = true
Rationale: the /research Oracle and the Designer both benefit from seeing test entry
points. The caller is responsible for filtering (researcher keeps all; designer filters
by setting include_tests=false in the EntryPointsRequest). Default=true avoids callers
having to opt in to see test harnesses.

DECISION-B34-5: Python __main__ as additive check
SocratiCode constants.ts:154 has Python = Set(["main"]) — __main__ NOT in the set.
ELAI adds sym.name == "__main__" as a separate check (additive, not contradictory).
Emits WellKnownName("__main__") regardless of language (handles any language where
the Python-style __main__ idiom appears).

DECISION-B34-6: DetectedEntryPoint naming (avoiding B-29 collision)
types::EntryPoint already exists at knowledge::symbol_graph::types:112 (B-29 type).
New type is named DetectedEntryPoint to avoid name collision.
Neither is re-exported at the symbol_graph module root (per mod.rs:9 warning).
Callers must qualify: knowledge::symbol_graph::entry_points::DetectedEntryPoint.

DECISION-B34-7: SSB SCHEMA_VERSION bump 41 -> 42
Existing version was 41 (B-40 FRAMING substate). B-34 adds 4 new SSB action types:
entry_points_detected, artifacts_indexed, artifact_searched, artifact_stale.
All 5 lockstep sites updated: enum / CHECK constraint / as_sql_str / from_str /
SsbPayload variants + action_type() match + test_payload_reconstructor.
Payload structs for Phase C indexer wiring (artifacts_indexed, artifact_stale)
will be emitted once Phase C2 wires ArtifactIndexer into IndexingService.

DECISION-B34-8: detect_entry_points signature (pure fn, not cache-taking)
Context-compiler suggested cache-taking signature. Task prompt specifies pure fn
(reverse_index + payloads). Chose pure fn because:
(a) easier to test with fixtures (no DB needed),
(b) dispatcher handles the selective payload loading,
(c) matches SocratiCode's detectEntryPoints(fileGraph, payloads) split.

SKIPPED: payload-selectivity tests (need real fixtures with a built symbol graph DB).
SKIPPED: performance tests (need real project; document for B-34 Phase 2 benchmarks).
