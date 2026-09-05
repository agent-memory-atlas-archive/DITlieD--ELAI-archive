C-33 — Documentation + Phase Map Updates Decisions
====================================================

Session  : s-25672
Plan     : F:\.ELAI_workflow\docs\thefinalglance\C-plan\C-33_docs_phase_map_updates.md
Context  : F:\.ELAI_workflow\workflow\sessions\s-25672\context-package-C33.md
Baseline : SCHEMA_VERSION = 63 (ssb.rs:493); slash_command_specs().len() = 152
           (commands.rs:4899); 2 lockstep FailureCategory enums in tools and
           core; 1 independent FailureCategory in mistake_book (untouched).

DRIFT decisions (orchestrator-supplied; adopted verbatim)
---------------------------------------------------------

D1 (SCHEMA reference). The plan said "26 confirmed" / "46 current". Real
   value at ssb.rs:493 is 63. B-0_INDEX update uses the full B-N -> C-N
   progression text supplied by the orchestrator (26 -> 63 with every
   intermediate bump cited). No SCHEMA bump in C-33.

D2 (slash command catalog). slash_command_specs().len() = 152, not 28+.
   migration_manifest.toml already had 78 [[command]] entries (C-14
   convention: PascalCase name, status = "KEPT"/"STRIPPED"/"DEFERRED").
   C-33 EXTENDS that table by 79 entries (5 missing PascalCase enum
   variants + 74 lowercase spec-only entries that route through
   SlashCommand::Unknown). Final count: 157 [[command]] entries.
   Plan's invented `[[file.commands]]` / `action = "KEEP"` format
   REJECTED — mirrors the existing convention instead.

D3 (FailureCategory enum lockstep). 3 enums share the name; only 2 are
   lockstep partners. Targets of C-33:
     - tools::failure_taxonomy::FailureCategory (9 -> 12 variants).
     - core::tool_dispatcher::ToolFailureCategory (9 -> 12 variants).
   Untouched:
     - core::mistake_book::FailureCategory (16-variant independent enum
       that already carries LazyCode / EmptyBody / FileSizeExceeded
       under different semantics; consumers in plugins/tiered_analysis
       and reasoning/traversal would break if modified).

D4 (B-21 wording). B-21 reconciled by C-22: alternative_explored and
   feasibility_escalated were SHIPPED at SCHEMA 53 -> 54. Plan-side
   "deferred to C-22" wording REMOVED; replaced with the canonical
   6-name SSB action_types list + WIRING_DIVERGENCE_RESOLVED note
   crediting C-22-decisions.md D4.

D5 (B-39 REFINE_CAP). chat_reflect.rs:48 reads
   `pub const REFINE_CAP: u32 = 3`. B-39 plan L543 single-line edit
   replaces "Why REFINE cap = 2" with the cap=3 explanation that
   cites RefineBench Table 3 t=2->t=3 +12-20pp and the contract test
   at chat_reflect_contract.rs:503-511. The unrelated "Total pipeline
   budget cap = 20K" line stayed.

Implementation notes
--------------------

NOTE-A (4 new step files). All four files written under
   docs/specs/guide/steps/. docs/specs/ is gitignored — these files are
   local-only and do not appear in `git status`. Targets met:
     - phase-3L.txt: 142 LOC (target 80-120; came in slightly long with
       the WIRING_CONTRACT + DETERMINISTIC_CHECK blocks).
     - phase-3J-BASELINE.txt: 112 LOC (target 60-90; long due to
       acceptance-gate detail).
     - phase-4D-SURGICAL.txt: 130 LOC (target 80-120).
     - phase-4D-ORACLE.txt: 124 LOC (target 70-100).

NOTE-B (3J / 4G addenda). Appended only — no existing content removed
   or restructured. Each addendum opens with a status banner and lists
   shipped extensions with file:line landmarks.

NOTE-C (enum edit pattern). Single lockstep batch: enum body +
   detect_category arms + suggest_fix arms in failure_taxonomy.rs;
   enum body + as_str arm in tool_dispatcher.rs. Compile checked at
   each batch. No non-exhaustive match callers existed for either enum
   (verified via grep before editing).

NOTE-D (mistake_book FailureCategory). LEFT ALONE. Confirmed via grep
   that LazyCode/EmptyBody/FileSizeExceeded already exist there with
   different semantics; touching them would require coordinated edits
   in plugins/tiered_analysis and reasoning/traversal whose match arms
   already cover those variants.

NOTE-E (contract test layout). c33_phase_map_contract.rs uses the same
   parent-path resolution pattern as c30_surgical_oracle_contract.rs
   (CARGO_MANIFEST_DIR.parent().parent() for elai_cc root, then once more
   for workflow root) so docs/thefinalglance/B-plan files can be read
   from a contracts test without env-var plumbing. Tests soft-skip with
   `eprintln!` + return when a plan file is missing (gitignored repos).

Test results
------------

elai-tools test failure_taxonomy_9cat: 10 passed / 0 failed (9 existing
   round-trip tests + 1 9-variant structural test — the structural test
   still validates the original 9 names; new 3 are covered by the C-33
   contract test).

elai-tools (full): 217 passed / 4 failed. The 4 failures
   (bash_tool_reports_success..., glob_and_grep_tools_cover_..., 2 ×
   worker_tools_*) are PRE-EXISTING and unrelated to FailureCategory —
   they fail on tool-routing assertions about heredoc redirection that
   were broken before this session. Verified by checking `git diff --stat`
   — the only C-33 change in crates/tools/src is failure_taxonomy.rs.

elai_cc_contracts c33_phase_map_contract: 10 passed / 0 failed (new).

elai_cc_contracts tool_dispatcher_contract: 9 passed / 0 failed (unchanged).

elai-core (full): 148 passed / 0 failed (across 4 test suites).

cargo check --workspace: CLEAN (only pre-existing dead_code warnings).

ACCEPTANCE
----------

[X] 4 new step files exist under docs/specs/guide/steps/.
[X] phase-3J.txt 3J.ADDENDUM appended (ETCLogitsProcessor + mean_logprob
    landmarks present).
[X] phase-4G.txt 4G.ADDENDUM appended (AutoAdvanceEvaluator +
    cloud_spend_warning landmarks present).
[X] FailureCategory (tools) has 12 variants including the C-33 three.
[X] ToolFailureCategory (core) has 12 variants in lockstep.
[X] detect_category + suggest_fix have arms for all 12 variants.
[X] B-39 plan: REFINE cap=3 wording present; stale cap=2 line removed.
[X] B-21 plan: WIRING_CONTRACT lists system_probe / feasibility_report /
    novel_step_detected / stub_run / alternative_explored /
    feasibility_escalated; "deferred to C-22" wording removed.
[X] B-0_INDEX.md: SHIPPED blocks for B-37/B-38/B-40/B-41/B-42 present;
    SCHEMA_VERSION=63 progression line present.
[X] migration_manifest.toml: 157 [[command]] entries (>= 80; existing 78
    extended). Uses [[command]] / status="KEPT" format (NOT the plan's
    invented [[file.commands]] / action="KEEP").
[X] c33_phase_map_contract.rs: 10 tests pass.
[X] cargo check --workspace: clean.
[X] No new dependencies introduced.
[X] No SCHEMA_VERSION bump (FailureCategory is internal to tools/core,
    not an SsbActionType boundary).

Files touched
-------------

CREATE (gitignored under docs/specs/):
  - docs/specs/guide/steps/phase-3L.txt
  - docs/specs/guide/steps/phase-3J-BASELINE.txt
  - docs/specs/guide/steps/phase-4D-SURGICAL.txt
  - docs/specs/guide/steps/phase-4D-ORACLE.txt

MODIFY (gitignored under docs/specs/):
  - docs/specs/guide/steps/phase-3J.txt (3J.ADDENDUM appended)
  - docs/specs/guide/steps/phase-4G.txt (4G.ADDENDUM appended)

MODIFY (tracked):
  - .elai_cc/crates/tools/src/failure_taxonomy.rs
  - .elai_cc/crates/core/src/tool_dispatcher.rs
  - .elai_cc/migration_manifest.toml
  - docs/thefinalglance/B-plan/B-39_chat_reflect_pipeline.md (L543)
  - docs/thefinalglance/B-plan/B-21_phase_4F_feasibility_probe.md (WIRING_CONTRACT)
  - docs/thefinalglance/B-plan/B-0_INDEX.md (L128 + after L148)

CREATE (tracked):
  - .elai_cc/crates/contracts/tests/c33_phase_map_contract.rs
  - workflow/reasoning/C-33-decisions.md (this file)

Open follow-ups
---------------

None mandatory. The 4 pre-existing elai-tools test failures
(bash_tool_reports..., glob_and_grep_tools..., 2 × worker_tools_*) are
NOT C-33 regressions and remain for whichever C-N or follow-up patch
revisits the tool-routing safety layer.

WARN closure (session s-25672, post-audit)
------------------------------------------

WARN-1 closed. failure_taxonomy.rs detect_category() arm order rewritten:
the three C-33 patterns (LazyCode / EmptyBody / FileSizeExceeded) now
precede the RuntimePanic catch-all (lines ~157-187). Inline comment
brought into agreement with the code. Mixed-input fixtures
(panic banner + TODO body) will now classify as LazyCode, restoring the
behaviour the contract test fixtures assumed. cargo test
-p elai_cc_contracts --test c33_phase_map_contract still 10/10 PASS;
elai-tools failure_taxonomy lib test PASS. cargo check --workspace clean.

WARN-2 closed (paths only — tool resolver bug remains, separate issue).
Three Spec references: paths in phase-3L.txt rewritten to the canonical
crate roots verified against the working tree:
  - project_rule_seeder.rs : crates/knowledge/src/  ->  crates/rules/src/
  - indexing_service.rs    : crates/knowledge/src/  ->  crates/orchestrator/src/
  - telescope.rs           : crates/reasoning/src/  ->  crates/core/src/
Also updated the consumes_symbols line for CorpusView from
knowledge/src/indexing_service.rs to orchestrator/src/indexing_service.rs
for internal consistency. The spec_link_check.py resolver still
double-counts the parenthetical token after each path; that tool fix is
deferred to a separate follow-up patch (the four step files share the
behaviour, not C-33 specific).

WARN-3 closed. migration_manifest.toml deduped. The second "Ask"
[[command]] entry (former L912-915) removed; the canonical first entry
(former L866-869) gained a single-line rationale that absorbs the
alias-of-/research note ("Ad-hoc question pipeline. Alias of /research;
wired in live_cli.rs. Live."). Manifest now carries 156 unique
[[command]] entries (down from 157, all distinct primary keys).
Downstream tools building HashMap<name, status> will no longer collide.
migration_manifest_has_at_least_eighty_command_entries still PASS.

Acceptance re-verification (post-fix)
- detect_category arm order: C-33 patterns BEFORE RuntimePanic. PASS.
- phase-3L.txt: three Spec references rewritten + CorpusView line
  updated. Direct ls verifies each target file exists. PASS.
- migration_manifest.toml: 156 unique [[command]] entries, single "Ask"
  row. PASS.
- cargo check --workspace: clean. PASS.
- cargo test -p elai_cc_contracts --test c33_phase_map_contract: 10/10
  PASS.
- elai-tools lib tests: 217/4 (4 pre-existing failures unrelated to
  C-33 still present; failure_taxonomy test PASS).
- elai-core lib tests: 129/0.

End of C-33-decisions.md
