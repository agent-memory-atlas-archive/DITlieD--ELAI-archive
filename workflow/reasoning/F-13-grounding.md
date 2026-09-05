GROUNDING SWEEP -- phase F-13
Source: F:\.ELAI_workflow\docs\thefinalglance\F-plan\F-13_session_archive_schema_bump.md
Date: 2026-06-03
FSM: PLANNING | Session: s-6312

SPEC REFERENCES
workflow/research/shard-prior-art/07-springdrift.md:49 | GROUNDED | file exists; line 49 = "Backup Actor -- git commits + remote push" matches claim
workflow/research/shard-prior-art/07-springdrift.md:110 | GROUNDED | line 110 = "Git-backed recovery: .springdrift/ is a git repo; periodic commits (default 5 min) + optional remote push; git checkout <commit> restores entire agent state" -- verbatim match
workflow/research/shard-prior-art/07-springdrift.md:18,:24 | GROUNDED | lines 18/24 = AGPL-3.0 license confirmed; line 18 = "License: AGPL-3.0"
workflow/research/shard-prior-art/07-springdrift.md:15 | GROUNDED | line 15 = "Paper: arXiv:2604.04660, cs.AI, submitted 2026-04-06" -- arXiv ID cite confirmed
workflow/research/shard-prior-art/07-springdrift.md:163,:174 | GROUNDED | lines 163/174 = "n=1, self-run, self-reported (no third-party reproduction)" and "reproducibility scaffolding exists even if not yet independently reproduced"
workflow/research/shard-prior-art/07-springdrift.md:187 | GROUNDED | line 187 = "Springdrift makes the whole .springdrift/ a git repo with periodic commits and git checkout of entire agent state" -- matches claim about git-backed whole-state vs SHARD per-change .bak
F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\ssb.rs:14334-14348 | GROUNDED | read range: run_migration DEF at :14334; DROP at :14342 "DROP TABLE IF EXISTS ssb_files; DROP TABLE IF EXISTS ssb;"; comment at :14339-14340 "Stale schema -> drop and recreate (data loss is acceptable for old sessions)"; CREATE_SSB_TABLE at :14344; CREATE_SSB_FILES_TABLE at :14345
F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\session_dag.rs:200-245 | GROUNDED | CREATE TABLE IF NOT EXISTS at :206/:211/:233; commit_hashes TEXT at :221; lazy ALTER TABLE confirmed in open() at :281 (ensure_raw_blocks_column)
F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\session_dag.rs:525-720 | GROUNDED | save_snapshot DEF at :532; load_snapshot DEF at :618; latest_snapshot DEF at :717
F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\checkpoint.rs:190-280 | GROUNDED | content_hash DEF at :194; blake3::hash CALL at :195; "SSB record contains metadata only (hashes + sizes + step_index + agent_id)" at :207; 5 files written :241-245
F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\architecture\approval_fatigue_guard.rs:1-60 | VERIFY_NEEDED | file exists; env-var const at :33-34 (not :30 as cited); plan says ":30" but const is at line 33. Off by 3 lines -- minor but cite is imprecise
F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\architecture\bash_complexity_guard.rs:1-60 | GROUNDED | env-var const at :34-35; plan cites ":31" (off by 3-4 lines) but the pattern is confirmed on-topic
F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\durability\checkpoint_migrator.rs:175 | GROUNDED | migrate_checkpoint DEF at :175 (exact line confirmed)
F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\ssb_viewer.rs | GROUNDED | SsbViewerError at :70; SsbViewer.open at :880; multiple viewer query fns exist -- pattern to mirror confirmed
docs/thefinalglance/F-plan/F-0_INDEX.md | GROUNDED | file exists (Glob confirmed)

CLAIM CLASSIFICATION

-- ssb::run_migration DROPs prior-version tables at one destructive site --
ssb.rs:14334-14348 single DROP, "data loss is acceptable for old sessions" | GROUNDED | read :14339-14342; DROP at :14342 is the sole destructive site; comment at :14339-14340 verbatim confirmed

-- session-DAG migration is non-destructive --
CREATE TABLE IF NOT EXISTS + lazy ALTER TABLE in open() | GROUNDED | session_dag.rs:206/:211/:233 = IF NOT EXISTS DDL; :281 = ALTER TABLE inside ensure_raw_blocks_column; open() at :404 calls ensure_raw_blocks_column for lazy migration -- non-destructive confirmed

-- checkpoints store hashes-not-bytes in SSB --
checkpoint.rs:207 "hashes + sizes + step_index + agent_id" | GROUNDED | read :207 verbatim: "SSB record contains metadata only (hashes + sizes + step_index + agent_id)"; 5 msgpack files written to disk separately :241-245

-- blake3 (not blake2/sha2) for archive integrity digest (CLAUDE.md HARD RULE) --
checkpoint.rs:5 "use blake3"; content_hash at :194-195 uses blake3::hash | GROUNDED | grep confirmed: checkpoint.rs:5 = "use blake3", :195 = "blake3::hash(data).to_hex().to_string()" -- in-tree precedent confirmed; CLAUDE.md anti-stale-training rule cited correctly

-- Springdrift git-backed whole-state recovery (arXiv:2604.04660) --
07-springdrift.md:110 + arXiv cite at :15 | GROUNDED | dossier line 110 = periodic commits + git checkout whole-state; arXiv:2604.04660 confirmed at :15; AGPL-3.0 at :18; author Seamus Brady single-author confirmed; eval unreproduced at :163/:174

-- F-13 scoped to minimal one-file-copy-before-DROP, NOT Backup-Actor --
stub_ban section, Task 2, killer_risk #1 | GROUNDED | plan explicitly: "Deliberately adds NO periodic commits, NO Backup Actor, NO unified store, NO git dependency"; stub_ban forbids "tokio::time::interval" and git shell-out; gap #3 explicitly deferred; this is plan scope definition not external behavior

-- SCHEMA_VERSION baseline 191 (cited from CLAUDE.md) --
ssb.rs:2898 actual const = 205 | VERIFY | CLAUDE.md says "SCHEMA_VERSION 191" but ssb.rs:2898 reads "const SCHEMA_VERSION: i64 = 205" -- STALE CLAUDE.md reference; plan correctly says "named exactly at edit time, never hard-coded final number in prose" and "baseline 191 + 1 (named at edit time)" -- the plan's guidance to name at edit time is correct defensive doctrine, but the plan's own WIRING_CONTRACT consumes_symbols says "baseline 191 per CLAUDE.md" which is stale. Implementer must verify live value (205 today) before editing.

-- approval_fatigue_guard.rs:30 as wire-dark env-gate const location --
actual const at :33-34 not :30 | VERIFY | read lines 30-35: comment at :31-33, const at :33 (pub const starts at line 33). Plan cites ":30" but const is at :33-34. 3-line offset; same file, same const, same pattern -- functionally grounded but line anchor imprecise

-- bash_complexity_guard.rs:31 as wire-dark env-gate const location --
actual const at :34-35 not :31 | VERIFY | read lines 30-35: :31 is comment start, :34-35 is the pub const. Plan cites ":31" -- off by 3 lines. Same pattern, same file, functionally grounded

-- DeterminismDriftPayload first_divergence_index at ssb.rs:16070-16079 --
not verified in this sweep | VERIFY | line range not read (outside mandatory reads list); claim is about CONTEXT framing, not a deliverable for F-13 implementation. Low-risk: verify with grep if needed before EXECUTING

-- DriftComparator at determinism.rs:317 (NOT ssb.rs:403) --
not verified in this sweep | VERIFY | plan itself flags this as W-7 correction from the source verdict; low-risk for F-13 scope (DriftComparator not a deliverable)

-- periodic ticks task_runner.rs:13190-13280 confirm NO whole-state committer --
REFUSAL_SLA_TICK at :13190 + SKILL_STALENESS_TICK at :13204 | GROUNDED | read :13185-13214; ELAI_REFUSAL_SLA_TICK_ENABLE_ENV const at :13190; REFUSAL_SLA_TICK_INTERVAL_SECS at :13196; SKILL_STALENESS_TICK_INTERVAL_SECS at :13204; neither commits state -- confirmed

-- rusqlite + std::fs + blake3 all in-tree (no new dep) --
blake3 use confirmed in checkpoint.rs:5 | GROUNDED | blake3 import at checkpoint.rs:5; rusqlite is the SSB store dep (confirmed from ssb.rs migration code using Connection); std::fs is stdlib -- no new dep claim confirmed

-- arXiv:2604.04660 verified resolves (2026-06-02 WebFetch) --
07-springdrift.md:15 = "arXiv:2604.04660...RESOLVES" | GROUNDED | dossier states "The arXiv ID RESOLVES -- abstract, HTML, PDF, and TeX source all present" [S1]; this is the dossier's own grounding claim; plan cites dossier as T1

-- migrate_checkpoint at checkpoint_migrator.rs:175 --
confirmed at exact line | GROUNDED | grep + read both confirm pub fn migrate_checkpoint at line 175

-- LEGACY_CHECKPOINT_SCHEMA_VERSION = 40 at checkpoint.rs:77 --
actual at :77 | GROUNDED | read :70-82; LEGACY_CHECKPOINT_SCHEMA_VERSION = 40 const at :77 exactly; serde(default) at :96 matches plan claim ":96-97"

ACTIONS REQUIRED
- verify SCHEMA_VERSION live value before EXECUTING: ssb.rs:2898 currently shows 205, not 191 as CLAUDE.md states; plan's "baseline 191 per CLAUDE.md" in WIRING_CONTRACT consumes_symbols is stale; implementer must grep ssb.rs for SCHEMA_VERSION at edit time (the plan's "named at edit time" guidance is correct -- just ensure the consumes_symbols prose note is not treated as authoritative)
- verify approval_fatigue_guard.rs env-gate const line before citing in new code: const is at :33-34 not :30; bash_complexity_guard.rs env-gate const is at :34-35 not :31; 3-line offsets throughout both files -- minor, but the pattern-mirror implementation should read actual file at implement time
- verify DeterminismDriftPayload location (ssb.rs:16070-16079) and DriftComparator (determinism.rs:317) before EXECUTING if these are referenced in implementation comments; context-only claims, not F-13 deliverables, low priority
- UNGROUNDED risk: Springdrift eval numbers (P@4=0.956, deployment figures) are vendor-reported n=1 per 07-springdrift.md:163-174; plan correctly flags killer_risk #2 and does NOT build on these numbers -- no action needed

VALIDATION WARNINGS
- SCHEMA_VERSION stale: CLAUDE.md says 191, ssb.rs:2898 says 205 -- the plan correctly defers to "named at edit time" but the consumes_symbols note citing "baseline 191 per CLAUDE.md" may mislead an implementer; flag for correction in the decision doc
- approval_fatigue_guard.rs and bash_complexity_guard.rs line anchors off by 3 lines each (plan cites :30/:31, actual consts at :33-34/:34-35) -- functionally grounded but imprecise; not blocking
- session_dag open() migration behavior: the plan says "lazy ALTER TABLE in open() :229-231" but the actual ALTER happens inside ensure_raw_blocks_column called at :446 (not a direct :229-231 ALTER statement); the comment at :229-231 DESCRIBES the migration; the implementation is at :281 via ensure_raw_blocks_column. Non-destructive claim is correct; anchor slightly off.
- all sources validated (all cited files exist and contain on-topic content)
