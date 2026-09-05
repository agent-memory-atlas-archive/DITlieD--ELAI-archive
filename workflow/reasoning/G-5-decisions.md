G-5 decisions — Reviewer Cockpit + Diff-Surface-to-Review (DSR) metric
session s-56068, 2026-06-04

WHAT
A per-hunk reviewer cockpit that renders ELAI's already-emitted deterministic proof tokens (F-4 stdout_hash-bound passing contract test, F-1/F-2a write-gate CWE verdict, the grounding-sweep citation) so a reviewer SKIMS vouched hunks. Headline DSR = sum(un-vouched changed lines) / sum(all changed lines), recomputed byte-identically from git-diff + SSB under ELAI_SEED (GD-2). Silent-by-default: a finding surfaces ONLY when backed by a deterministic gate verdict or a failing test (never an LLM opinion). The within-subjects time-to-first-approve study is a DEFERRED research protocol, NOT the done-gate (GD-3/GD-5 — no measured review-time number is claimed).

GROUND TRUTH (context-compiler + 2 explorers, s-56068, fresh elai.map): VERIFIED the E0412 "concurrent G-block" claim is FALSE — `cargo check -p elai-orchestrator` and `-p elai_cc_contracts --tests` both Finished clean (only unused-import `Path` warnings from G-1/G-3 leftovers, cosmetic). Tree is GREEN. Baseline SCHEMA_VERSION 211 (ssb.rs:2938), TOTAL_VARIANTS 488 (action_type_lock.rs:575) -> 212/489.

KEY DECISIONS

DD1 DSR DATA SOURCES (two pure inputs -> GD-2 determinism).
- DENOMINATOR + per-hunk bodies = `git diff` against a PINNED base. SSB stores NO raw line bodies (FileWritePayload = {path,size_bytes,hash}; FileChange = {path,action,summary}). Use the in-tree git-diff pattern: wiring_coverage.rs:398 (`git diff --unified=0 <base>`) / focus_tunnel::diff::DiffParser::parse -> Vec<DiffHunk>. PIN the base deterministically (merge-base or a recorded base commit) so the diff bytes are stable under ELAI_SEED. Do NOT use the C-7 ReviewWindow's FAKE-one-hunk-per-file synthesis — use the real git-diff hunks.
- VOUCH SET = SSB via query_session_stream_asc (seq-ASC, byte-identical per session). A hunk/file is VOUCHED when its file_path has a clean/passing deterministic signal: a passing TestIntegrityEvent, an F-1 VibeGateBlocked that did NOT fire (or a clean write-gate verdict), an F-2a SecurityRuleBlocked that did NOT fire. UN-VOUCHED (needs full attention) = no vouch OR a failing-test/fired-gate signal.

DD2 GRANULARITY = HUNK/FILE-LEVEL, not per-line (honest limit, GD-5). The F-4 proof-token has NO per-hunk data edge: TestIntegrityEventPayload carries test_name only, NOT a covered file/hunk path; GroundingSweepResultPayload is aggregate (no per-file key). So vouching maps at the FILE level (the file_path-bearing signals: VibeGateBlocked/SecurityRuleBlocked) and the contract-test->file link is a documented heuristic, NOT a precise per-hunk coverage edge. DSR is computed at hunk granularity (a hunk inherits its file's vouch state). State this honestly: G-5 does NOT claim per-line test-coverage attribution. A precise test->hunk coverage edge is a follow-up (would need a new payload field).

DD3 review_surface_scored SSB variant. NEW Pattern-2 single-variant, mirror the C-7 ChangeDecisionPayload{decision,path,hunk_id,note} precedent (verdict-only rows reference by path+hunk_id, NEVER inline the diff). Payload = DSR ratio + vouched line count + un-vouched line count + total changed lines + the pinned base commit. Bump SCHEMA 211->212 (ssb.rs:2938) + TOTAL_VARIANTS 488->489 (action_type_lock.rs:575) in lockstep; regen golden (manual/codegen path), mirror the just-shipped BlastRadiusBlocked cascade. GD-2: DSR re-computes byte-identically from the pinned git-diff + the ASC SSB stream.

DD4 EMIT FROM A LIVE-Ssb PRODUCER (the recurring SSB-ctor wire-dark check). `elai review --cockpit` is the HEADLESS PRODUCER: it parses the git diff, reads the SSB vouch set, computes DSR, PRINTS DSR, and EMITS review_surface_scored. It must hold a live Arc<Ssb> for the session (mirror G-2 rra_cmd.rs which opens/holds the session Ssb at run_subcommand.rs:374/457/518). The Tauri dashboard command is the read-only CONSUMER of the row. Trace the emit to a real Ssb (NOT a read-only dashboard command, NOT a reconstructed Ssb).

DD5 GD-1 COMPOSE-NOT-RESTATE: consume F-4 stdout_hash (TestOutcomePayload) + G-2's RRA manifest (rra::read_rra -> RunArtifact.manifest: Vec<GateVerdictHash>; additive — but G-5 MUST also score from live SSB alone when no .rra exists) + the grounding-sweep. Re-implement NONE. Reading the RRA is optional enrichment; the SSB stream is the primary deterministic source.

DD6 COCKPIT IS A FULL VIEW (WindowType), not a drawer. Route through WindowHost.svelte (WindowType if/else, mirror an existing window arm). Adding a 'cockpit' WindowType requires updating ALL THREE label maps in sync: types.ts:67 (the WindowType union), WindowHost SCOPE_LABEL:56, workspace VALID_WINDOW_TYPES:9 — missing ANY one silently falls back to FallbackWindow (the G-2/G-3 'rra-viewer' label-miss pattern). Verify all three. frontend_pair: the cockpit window (render vouched/un-vouched bands over the git-diff hunks) + commands_g5.rs Tauri command (returns per-hunk vouch data + DSR).

DD7 SILENT-BY-DEFAULT: surface a finding ONLY when backed by a deterministic gate verdict (VibeGateBlocked/SecurityRuleBlocked fired) or a failing test (TestIntegrityEvent failing/FabricationBlocked). NO LLM-opinion findings. All surfaceable signals live in SSB with file_path; filter to them.

DD8 NO MEASURED NUMBER (GD-3/GD-5): the cockpit + DSR is the CODE done-gate. The within-subjects screen-recorded clustered-SE METR-design study (time-to-first-approve cockpit vs plain diff, escaped-defect-rate guardrail) is a DEFERRED RESEARCH PROTOCOL — document it as the measurement plan; do NOT state any measured review-time number anywhere in code/CLI/docs. Speed-AND-safety simultaneity is the HYPOTHESIS the study tests.

EXPECTED TEST OUTCOME
Deterministic fixture: a fixture branch/diff with N changed lines across files, some files carrying a passing-test/clean-gate SSB signal (vouched) and some not (un-vouched) -> `elai review --cockpit` computes DSR = un-vouched/total, prints it, emits a review_surface_scored row; the DSR re-computes byte-identically from the pinned git-diff + the ASC SSB stream (GD-2). A file with a FIRED gate / failing test -> surfaced as a finding (silent-by-default holds: a clean file with no signal is NOT surfaced). No LLM opinion appears.

GD-4 / W-5
GD-4: the DSR + escaped-defect-rate is itself the FP early-warning (a DSR that marks everything un-vouched = the gate adds no value; one that vouches a buggy hunk = a miss). In-process: the fixture proves the ratio computes correctly. W-5: the live cockpit render (dashboard-smoker) + the human time-study are separately tracked (Tier-2/research); the deterministic DSR fixture is the Tier-1 gate.

UNKNOWNS / RISKS
- Pinning the diff base deterministically (merge-base vs recorded base) — the executor picks the most deterministic available base and records it in the row.
- The contract-test->file vouch heuristic (no precise per-hunk edge) — honest file-level granularity; do not over-claim per-line coverage.
- `elai review --cockpit` Ssb handle: confirm it opens/holds the session Ssb (mirror rra_cmd); if headless review runs outside a journaled session, it must open the session's ssb.db like the rra/replay verbs do.
- Cosmetic: unused-import `Path` warnings in production_agent_rpc.rs / others (G-1/G-3 leftovers) — note for the final cleanup sweep, not G-5's job.
