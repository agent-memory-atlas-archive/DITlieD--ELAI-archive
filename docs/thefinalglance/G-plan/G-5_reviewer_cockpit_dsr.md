# G-5: Reviewer Cockpit + Diff-Surface-to-Review (DSR) metric

STATUS: PLANNED (WAVE 2; depends G-2 for the RRA evidence manifest; consumes F-4/F-1/grounding-sweep rows)

Objective: a per-hunk reviewer cockpit that renders the proof tokens ELAI ALREADY emits — an F-4
stdout_hash-bound passing test, an F-1/F-2a write-gate verdict (with its CWE), a grounding-sweep citation —
so a reviewer SKIMS vouched hunks and spends attention only where there is no deterministic vouching. The
headline number is DSR = (lines needing full attention) / (total changed lines), recomputed deterministically
from the SSB journal (GD-2). Silent-by-default: surface a finding ONLY when backed by a deterministic gate
verdict or a failing test — never naive LLM-opinion noise (which is style-biased and misses SQLi/XSS,
arXiv:2509.13650; carries 0-30% signal in 60.2% of code-review-agent PRs, arXiv:2604.03196). HONEST SCOPE
(GD-2): the per-hunk VERDICT rows exist (ChangeDecisionPayload.hunk_id) but the changed-LINE COUNT (the DSR
denominator) is NOT persisted in any current SSB row — G-5 MUST add that field; DSR is not SSB-recomputable
today, and the plan says so. Estimated effort: ~1-2 sessions (~8-12 h). In-process; the within-subjects
review-time study is a SEPARATE measurement, not the done-gate.

Pre-requisites: F-4 TestOutcomePayload.stdout_hash SHIPPED (grep-confirmed 2026-06-02: ssb.rs:9877/:9887,
blake3 16-char prefix, SCHEMA 197). F-1 VibeGateBlockedPayload SHIPPED (ssb.rs:25054, source_blake3:25064,
cwe:25060). C-1 GroundingSweepResultPayload SHIPPED (ssb.rs:15287, one row per ground_plan). C-7
ChangeDecisionPayload SHIPPED (ssb.rs:15933, hunk_id:Option<String>, SCHEMA 49). The existing review surface
ReviewWindow.svelte + get_file_changes (commands.rs:10148 / lib.rs:775 -> SsbViewer::file_changes
ssb_viewer.rs:1387) SHIPPED. G-2 RRA is a FORWARD DEPENDENCY — rra.rs does NOT exist yet (confirmed absent).
Parallel-safe with: G-4, G-8, G-9 (disjoint files). NOT parallel-safe with G-2 (consumes its RRA manifest) or
any plan editing ssb.rs's FileWritePayload / write path in the same window.
Source: workflow/reasoning/ELAI-TRUST-MAXIMIZATION-2026-06-02.md R5. Doctrine: G-0_INDEX.md (W-1..W-10 + GD-1..GD-5).

## Context

Per the trust-max synthesis R5: reviewing/editing AI output is THE confirmed review-time sink (METR
arXiv:2507.09089 — a 19% slowdown when developers review AI-generated code). The market response — naive
LLM-generated review comments — carries 0-30% signal in 60.2% of cases (arXiv:2604.03196) and is style-biased,
missing SQLi/XSS (arXiv:2509.13650). ELAI's differentiator is that it ALREADY emits deterministic proof
tokens per change: an F-4 stdout_hash-bound passing test (ssb.rs:9887), an F-1 write-gate verdict with its CWE
(VibeGateBlockedPayload.cwe, ssb.rs:25060), a C-1 grounding-sweep result (ssb.rs:15287). The cockpit renders
those per hunk so a reviewer can SKIM the vouched hunks and concentrate attention on the un-vouched ones. The
headline is DSR = (lines needing full attention) / (total changed lines): the smaller it is, the more of the
diff carried deterministic vouching. DSR (and escaped-defect-rate) is also the GD-4 false-positive early
warning — a gate the reviewer learns to ignore shows up as a DSR that never shrinks.

THE HONEST GD-2 FINDING (do not overstate): per-hunk VERDICT rows ARE SSB-persisted — ChangeDecisionPayload
(ssb.rs:15933) carries hunk_id derived from the `@@ -N,M +P,Q @@` header hash (C-7, SCHEMA 49), so the DSR
NUMERATOR (hunks/lines needing attention) is reconstructable. BUT the changed-LINE COUNT — the DSR DENOMINATOR
— is NOT persisted anywhere: FileWritePayload (ssb.rs:9858) stores only {path, size_bytes, hash}, and the
SsbViewer FileChange struct (ssb_viewer.rs:159) stores only {path, action, summary}. So DSR is NOT recomputable
from the current SSB. G-5 MUST add a changed-line-count field (in a new review_surface_scored payload, sourced
from the FileWrite diff) so DSR becomes GD-2-recomputable. The plan does NOT claim DSR is already auditable —
it builds the missing denominator.

ELAI ground truth (grep-confirmed 2026-06-02): the F-4 vouching read-path is TestOutcomePayload.stdout_hash
(ssb.rs:9877/:9887), already read by the dashboard (commands.rs:837 raw + a has_stdout_hash:bool projection at
commands.rs:9501/:9556/:9582) — but there is NO dedicated SsbViewer TestOutcome reader (only file_changes at
ssb_viewer.rs:1387). The F-1 beacon VibeGateBlockedPayload (ssb.rs:25054, source_blake3:25064, cwe:25060,
wire-dark via ELAI_VIBE_GATE_ENABLE) and the F-2a SecurityRuleBlockedPayload.cwe (:25103) carry the CWE that is
the antidote to naive review. The grounding-sweep producer is GroundingSweepResultPayload (ssb.rs:15287) +
SsbActionType::GroundingSweepResult (:5238, "grounding_sweep_result":8547, one row per task_runner ground_plan()
at ssb.rs:3128). The existing review surface is ReviewWindow.svelte (calls get_file_changes at :59) backed by
the Tauri command get_file_changes (commands.rs:10148, registered lib.rs:775) -> SsbViewer::file_changes
(ssb_viewer.rs:1387; FileChange struct :159 = {path, action, summary} — NO line counts). The per-hunk verdict
precedent is C-7 ChangeDecisionPayload (ssb.rs:15933, SsbActionType::ChangeDecision:5325, "change_decision":8565,
hunk_id:Option<String>). A new `elai review --cockpit` subcommand attaches at the hand-rolled dispatcher
crates/cli/src/args.rs::parse_args (:1718), match rest[0].as_str() table (:1936), mirroring the "run" =>
parse_run_subcommand_args arm (:2159, fn:2345). The G-2 RRA evidence manifest (rra.rs) does NOT exist — G-5
DEPENDS_ON G-2. SCHEMA_VERSION=205 (ssb.rs:2898), TOTAL_VARIANTS=482 (tests/action_type_lock.rs:534).

## Existing state (B/C/D/E/F coverage)
- F-4 TestOutcomePayload (ssb.rs:9877, stdout_hash:9887) — SHIPPED. The per-hunk vouching signal; the dashboard
  already projects has_stdout_hash (commands.rs:9501/:9556). G-5 reads it per hunk; re-implements no part of F-4 (GD-1).
- F-1 VibeGateBlockedPayload (ssb.rs:25054, source_blake3:25064, cwe:25060) + F-2a SecurityRuleBlockedPayload.cwe
  (:25103) — SHIPPED, wire-dark. The CWE-carrying write-gate verdicts G-5 renders (the deterministic antidote to naive review).
- C-1 GroundingSweepResultPayload (ssb.rs:15287) + GroundingSweepResult (:5238) — SHIPPED. The citation/grounding token per hunk.
- C-7 ChangeDecisionPayload (ssb.rs:15933, hunk_id:Option<String>, SCHEMA 49) — SHIPPED. The per-hunk verdict
  precedent + the DSR numerator source; review_surface_scored mirrors its hunk-aware shape.
- FileWritePayload (ssb.rs:9858 = {path,size_bytes,hash}) + SsbViewer::file_changes / FileChange (ssb_viewer.rs:1387/:159
  = {path,action,summary}) — SHIPPED. The diff source rows. GD-2 GAP: NEITHER persists a changed-line count — G-5 adds it.
- ReviewWindow.svelte (get_file_changes at :59, stub.ts:3059) + get_file_changes Tauri (commands.rs:10148, lib.rs:775)
  — SHIPPED. The review surface G-5's cockpit extends.
- CLI dispatcher args.rs::parse_args:1718 (match rest[0].as_str():1936; "run" => parse_run_subcommand_args:2159) — SHIPPED.
- G-2 RRA evidence manifest (rra.rs) — ABSENT. FORWARD DEPENDENCY; G-5 reads the RRA manifest per-hunk once G-2 ships.
- ssb.rs — SCHEMA 205 (:2898), TOTAL_VARIANTS 482 (tests/action_type_lock.rs:534).

## Reads (mandatory before coding)
- workflow/reasoning/ELAI-TRUST-MAXIMIZATION-2026-06-02.md R5 + the MEASUREMENT design (within-subjects, METR)
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\ssb.rs:9858,9877,15287,15933,25054 (FileWrite/TestOutcome/GroundingSweep/ChangeDecision/VibeGate payloads)
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\ssb_viewer.rs:159,1387 (FileChange + file_changes — the reader G-5 extends)
- F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\src-tauri\src\commands.rs:837,9501,10148 (stdout_hash projection + get_file_changes)
- F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\src\lib\windows\ReviewWindow.svelte (the existing review surface)
- F:\.ELAI_workflow\.elai_cc\crates\cli\src\args.rs:1718,1936,2159 (the CLI dispatcher + the "run" subcommand precedent)
- docs/thefinalglance/G-plan/G-2_reproducible_run_artifact.md (the RRA evidence manifest G-5 consumes — forward dep)
- CLAUDE.md HARD RULES: blake3, no .unwrap()/.expect() in prod, Pattern-2 SSB, SCHEMA baseline+N.

## Tasks (WAVE 2)

1. **Add the DSR scorer + the changed-line-count field (~150 LOC in a new crates/orchestrator/src/review_surface.rs).**
   `pub struct HunkVouch { hunk_id: String, vouched_by: Vec<VouchKind>, lines_changed: u32, needs_attention: bool }`
   where `pub enum VouchKind { TestStdoutHash, WriteGateVerdict { cwe: Option<String> }, GroundingSweep }`. `pub fn
   score_review_surface(file_changes, test_rows, gate_rows, sweep_rows) -> ReviewSurfaceScore` joining the SSB rows per
   hunk and computing DSR = sum(lines of un-vouched hunks) / sum(all changed lines). THE GD-2 FIX: the changed-line
   count per hunk is sourced from the FileWrite diff and PERSISTED in the new review_surface_scored payload so DSR
   re-computes byte-identically from the journal. NO `.unwrap()`/`.expect()`; a hunk with no vouch is needs_attention=true (conservative).

2. **Silent-by-default finding rule (~40 LOC).** A finding is surfaced ONLY when backed by a deterministic gate
   verdict (VibeGate/SecurityRule with a CWE) or a failing test (stdout_hash present + passed=false) — NEVER a
   naive-LLM opinion. NO LLM-review-comment generation branch (stub_ban): the cockpit renders ELAI's own
   deterministic tokens, it does not call a model to invent review comments.

3. **CLI surface `elai review --cockpit` (~110 LOC across args.rs + a new review_cmd.rs).** Attach a "review" arm
   in the args.rs:1936 match table (mirror the "run" => parse_run_subcommand_args:2159 pattern); `elai review --cockpit
   [--session <id>] [--json]` reads the session's SSB (+ the RRA evidence manifest when G-2 has shipped), runs
   score_review_surface, prints the vouched/un-vouched bands + DSR. W-3 production entrypoint + W-5 trigger.

4. **Add the `review_surface_scored` SSB action_type (~80 LOC in ssb.rs, W-9).** ONE single-variant Pattern-2
   (mirror ChangeDecision ssb.rs:15933 hunk-aware shape + the F-1 wire-dark-by-env convention): `ReviewSurfaceScoredPayload
   { session_id, dsr: f64, total_changed_lines: u32, attention_lines: u32, vouched_hunks: u32, unvouched_hunks: u32 }`.
   sql_str "review_surface_scored" + parse + CHECK + round-trip golden + action_type_lock. SCHEMA bump baseline 205
   (ssb.rs:2898) -> 206; TOTAL_VARIANTS 482 (tests/action_type_lock.rs:534) -> 483 + ALL_VARIANTS row + golden regen.

5. **Default-ON + GD-4 FP early-warning (~30 LOC).** ELAI_REVIEW_COCKPIT_DISABLE kill-switch only; default ON; the
   observable_effect runs with NO override (W-2). DSR + escaped-defect-rate is the GD-4 FP early warning — record DSR
   across >=3 real orchestrate runs; a DSR that does not shrink (the reviewer is ignoring the bands) is the over-block signal.

6. **Frontend pair: the cockpit view (~280 LOC svelte + ~130 LOC tauri) (W-4).** Extend ReviewWindow.svelte (or a new
   CockpitWindow.svelte) to render each hunk with its vouch badges (test/gate-CWE/grounding) in vouched vs un-vouched
   bands + the DSR headline; Tauri `get_review_surface`. cross_lang_diff.py PASS.

7. **Decision doc workflow/reasoning/G-5-decisions.md (~110 LOC).** Records: (a) the GD-2 gap — changed-line counts
   were not persisted, G-5 adds them (DSR is built, not pre-existing); (b) silent-by-default (deterministic tokens only,
   the 0-30%-signal + style-bias papers); (c) DSR as the GD-4 FP early warning; (d) the within-subjects review-time
   study is a SEPARATE measurement (speed-AND-safety is the HYPOTHESIS it tests, NOT a pre-claimed result, GD-3); (e)
   G-5 DEPENDS_ON G-2 (RRA manifest), consumes F-4/F-1/grounding-sweep, re-implements none.

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/review_surface.rs::score_review_surface
  - crates/orchestrator/src/review_surface.rs::HunkVouch
  - crates/orchestrator/src/review_surface.rs::ReviewSurfaceScore
  - crates/orchestrator/src/ssb.rs::SsbActionType::ReviewSurfaceScored
  - crates/orchestrator/src/ssb.rs::ReviewSurfaceScoredPayload
  # G-5-corrected 2026-06-04 (see G-5-decisions.md): shipped CLI fn is run_review (main.rs:489 dispatch), not run.
  - crates/cli/src/review_cmd.rs::run_review
  # G-5-corrected 2026-06-04: build_file_signals is the shipped SSB-vouch reader (review_cmd.rs:331, the session_files_by_seq consumer).
  - crates/orchestrator/src/review_surface.rs::build_file_signals
  - crates/dashboard-v2/src-tauri/src/commands_g5.rs::get_review_surface
  - crates/dashboard-v2/src/lib/windows/CockpitWindow.svelte
consumes_symbols:
  - crates/orchestrator/src/ssb.rs::TestOutcomePayload (F-4 stdout_hash vouch, :9877/:9887)
  - crates/orchestrator/src/ssb.rs::VibeGateBlockedPayload (F-1 verdict + cwe, :25054/:25060) + SecurityRuleBlockedPayload (:25103)
  - crates/orchestrator/src/ssb.rs::GroundingSweepResultPayload (C-1 sweep, :15287)
  - crates/orchestrator/src/ssb.rs::ChangeDecisionPayload (C-7 hunk_id precedent + DSR numerator, :15933)
  - crates/orchestrator/src/ssb.rs::FileWritePayload (diff/line-count source, :9858)
  - crates/orchestrator/src/ssb_viewer.rs::file_changes (the existing reader G-5 extends, :1387)
  - crates/dashboard-v2/src-tauri/src/commands.rs::get_file_changes (:10148) + ReviewWindow.svelte (:59)
  - crates/cli/src/args.rs::parse_args (the "review" subcommand attach site)
  # G-5-corrected 2026-06-04 (see G-5-decisions.md DD5): the G-2 RRA manifest is NOT consumed by shipped G-5.
  # G-5 scores from the live SSB stream ALONE (build_file_signals over session_files_by_seq); reading a .rra is
  # OPTIONAL future enrichment only. Not a load-bearing consume; do NOT cite rra.rs as a present G-5 dependency (R2).
reachable_from:
  # G-5-corrected 2026-06-04 (see G-5-decisions.md): shipped emit chain from the CLI production entrypoint.
  - crates/cli/src/main.rs:489 (`elai review --cockpit` dispatch) -> review_cmd::run_review -> open_score_and_emit
    (review_cmd.rs:273) -> build_file_signals + score_review_surface (review_cmd.rs:331-332) ->
    ssb.write_payload(SsbActionType::ReviewSurfaceScored) (review_cmd.rs:351) = the review_surface_scored row + printed DSR
  - crates/dashboard-v2/src-tauri/src/commands_g5.rs::get_review_surface -> CockpitWindow.svelte (WindowType 'cockpit')  [dashboard VIEW]
stub_ban:
  - NO naive-LLM review-comment generation: findings come ONLY from deterministic gate verdicts or failing tests
  - NO fabricated DSR: the changed-line count is PERSISTED (new payload field) so DSR is GD-2-recomputable; a DSR
    computed from an un-persisted denominator is a BLOCK
  - NO claim that DSR is already SSB-recomputable pre-G-5 (it is NOT — FileWritePayload/FileChange carry no line count)
  - NO default-off ENABLE flag (only ELAI_REVIEW_COCKPIT_DISABLE kill-switch; default ON, W-2)
  - NO citation of crates/orchestrator/src/rra.rs as an existing symbol (G-2 forward dep; absent today)
  - blake3 only where hashing; no blake2/sha2
frontend_pair:
  svelte: crates/dashboard-v2/src/lib/windows/CockpitWindow.svelte
  tauri: crates/dashboard-v2/src-tauri/src/commands_g5.rs::get_review_surface
observable_effect:
  trigger: "cd .elai_cc && cargo run -p elai-cli -- <orchestrate a fixture change with some hunks backed by an F-4
    stdout_hash-passing test + an F-1 gate verdict and some un-vouched hunks; then `elai review --cockpit --session <id>`; NO env override>"
  expect_ssb: "a review_surface_scored row with dsr + total_changed_lines + attention_lines + vouched/unvouched hunk
    counts in the live ssb.db; the dsr re-computes byte-identically from the FileWrite/ChangeDecision/TestOutcome rows (GD-2)"
  expect_artifact: "the cockpit renders vouched hunks (test/gate-CWE/grounding badges) vs un-vouched bands + the DSR
    headline; CockpitWindow shows the bands via dashboard-smoker"

## DETERMINISTIC_CHECK
```bash
# G-5-corrected 2026-06-04 (see G-5-decisions.md): --run-trigger is a deprecated no-op (R3: the trigger always runs); dropped.
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/G-plan/G-5_reviewer_cockpit_dsr.md
rg "pub fn score_review_surface|pub struct HunkVouch|pub struct ReviewSurfaceScore" .elai_cc/crates/orchestrator/src/review_surface.rs
rg "score_review_surface" .elai_cc/crates/cli/src/review_cmd.rs   # wired into the CLI entrypoint (run_review -> open_score_and_emit:331-332)
rg "ReviewSurfaceScored|review_surface_scored" .elai_cc/crates/orchestrator/src/ssb.rs
rg "review_surface_scored" .elai_cc/crates/orchestrator/tests/action_type_lock.golden   # ordinal 0346
rg -c "ELAI_REVIEW_COCKPIT_ENABLE" .elai_cc/crates/cli/src/review_cmd.rs   # MUST be 0 (kill-switch DISABLE only, review_cmd.rs:70)
rg -c "total_changed_lines" .elai_cc/crates/orchestrator/src/ssb.rs                     # the GD-2 denominator field MUST exist
test -f .elai_cc/crates/dashboard-v2/src/lib/windows/CockpitWindow.svelte
python .claude/tools/cross_lang_diff.py
test -f workflow/reasoning/G-5-decisions.md
```

## ACCEPTANCE
- [ ] W-1 done-gate: wiring_contract_check.py --run-trigger PASS (produces exist, consumes resolve, reachable_from
  reached by BFS, stub_ban clean, observable_effect RAN + emitted the review_surface_scored row + printed DSR). cargo-green != DONE.
- [ ] W-2 default-ON: `elai review --cockpit` runs with NO env override; ELAI_REVIEW_COCKPIT_DISABLE=1 disables it.
- [ ] W-3 production entrypoint: the cockpit is reachable from `elai review --cockpit` (args.rs dispatch :2117 -> main.rs:489 -> run_review) + the dashboard CockpitWindow (not test-only).  # G-5-corrected 2026-06-04
- [ ] W-6 dashboard E2E: CockpitWindow renders vouched vs un-vouched bands + the DSR headline via dashboard-smoker.
- [ ] GD-2: the DSR re-computes byte-identically from the SSB rows (FileWrite line count + ChangeDecision hunk verdicts
  + TestOutcome stdout_hash) — a contract test asserts the recompute matches the persisted dsr. The changed-line-count
  field is PRESENT in the new payload (the gap G-5 closes).
- [ ] Silent-by-default: a contract test asserts a hunk with NO deterministic vouch and NO failing test surfaces NO
  finding (no naive-LLM opinion).
- [ ] GD-3: the decision doc frames speed-AND-safety as the HYPOTHESIS the within-subjects study tests, NOT a pre-claimed result.
- [ ] review_surface_scored round-trips via the SSB roundtrip contract + action_type_lock golden; SCHEMA bump named
  baseline 211 -> 212, TOTAL_VARIANTS 488 -> 489 (golden ordinal 0346).  # G-5-corrected 2026-06-04 (see G-5-decisions.md): shipped values 212/489, not the stale 206/483.
- [ ] cargo check --workspace GREEN; auditor PASS on -p elai-orchestrator -p elai-cli -p contracts.
- [ ] No new dependency (SSB readers + blake3 in-tree; anti-slopsquatting — verify before any add).

## WIRING DONE-GATE (clause by clause)
- W-1: wiring_contract_check (trigger runs unconditionally, R3) is ACCEPTANCE 1; score_review_surface (produced) is
  consumed by review_cmd::run_review + get_review_surface (W-8) reached by the CLI + dashboard (W-3).  # G-5-corrected 2026-06-04
- W-2: only ELAI_REVIEW_COCKPIT_DISABLE; default ON; observable_effect runs with no override.
- W-3: `elai review --cockpit` (args.rs: Review variant :449, ReviewAction::Cockpit :1329, dispatch :2117, parser :3551) +
  the dashboard CockpitWindow; not test-only.  # G-5-corrected 2026-06-04 (see G-5-decisions.md): real anchors, not the stale args.rs:1936
- W-4: review_surface_scored is human-surfaceable -> CockpitWindow + get_review_surface; cross_lang_diff PASS.
- W-5: observable_effect is a runnable CLI -> a live ssb.db row + a rendered cockpit; in-process, no RunPod.
- W-6: ACCEPTANCE verifies the bands + DSR THROUGH CockpitWindow via dashboard-smoker.
- W-7: TestOutcome/VibeGate/GroundingSweep/ChangeDecision/FileWrite payloads + file_changes + get_file_changes +
  args.rs dispatcher re-confirmed against the FRESH elai.map 2026-06-04 (G-5 files now indexed); rra.rs is NOT a shipped
  G-5 consume (DD5: SSB-only scoring) and is not cited as present; the validator re-greps.  # G-5-corrected 2026-06-04
- W-8: score_review_surface (produced) called by review_cmd (Task 3); ReviewSurfaceScoredPayload emitted (Task 4) +
  consumed by the Tauri command (Task 6). No produced symbol caller-less.
- W-9: ONE Pattern-2 ReviewSurfaceScored variant (mirror ChangeDecision + F-1 wire-dark-by-env); action_type golden +
  cross_lang_diff; bump 211->212 / 488->489 (golden ordinal 0346).  # G-5-corrected 2026-06-04: shipped 212/489, not the stale 206/483
- W-10: entirely Tier-1 in-process (read journal, join per hunk, compute DSR) — no network/subprocess/RunPod at gate time.
- GD-1: COMPOSES-WITH F-4 (stdout_hash), F-1 (verdict+cwe), C-1 (sweep); re-implements none. (G-2 RRA manifest = OPTIONAL
  future enrichment, NOT a shipped G-5 consume per DD5 — G-5 scores from live SSB alone.)  # G-5-corrected 2026-06-04
  GD-2: DSR recomputable from the journal (the denominator field is the fix). GD-3: speed-AND-safety is the study's
  hypothesis, not a claim. GD-4: DSR + escaped-defect-rate is the FP early warning.

## Tests to add
- crates/contracts/tests/g5_dsr_recompute_contract.rs (~160 LOC): (a) a fixture session with vouched + un-vouched hunks
  -> score_review_surface DSR matches the hand-computed ratio; (b) the persisted dsr re-computes byte-identically from
  the FileWrite line counts + ChangeDecision verdicts + TestOutcome stdout_hash rows (GD-2); (c) a hunk with no vouch is
  needs_attention=true; (d) the review_surface_scored payload round-trips; (e) total_changed_lines is persisted (the GD-2 field).
- crates/contracts/tests/g5_silent_by_default_contract.rs (~110 LOC): a hunk with no deterministic verdict + no failing
  test surfaces NO finding; a hunk with a VibeGate CWE verdict DOES surface; a hunk with a failing stdout_hash test DOES surface.
- crates/contracts/tests/fixtures/g5/ (sessions with mixed vouched/un-vouched hunks).

## Files touched
- CREATE: crates/orchestrator/src/review_surface.rs (~150 LOC)
- CREATE: crates/cli/src/review_cmd.rs (~110 LOC)
- CREATE: crates/dashboard-v2/src-tauri/src/commands_g5.rs (~130 LOC)
- CREATE: crates/dashboard-v2/src/lib/windows/CockpitWindow.svelte (~280 LOC)
- CREATE: crates/contracts/tests/g5_dsr_recompute_contract.rs (~160 LOC)
- CREATE: crates/contracts/tests/g5_silent_by_default_contract.rs (~110 LOC)
- CREATE: crates/contracts/tests/fixtures/g5/*.json (fixtures)
- CREATE: workflow/reasoning/G-5-decisions.md (~110 LOC)
- MODIFY: crates/orchestrator/src/lib.rs (~2 LOC: pub mod review_surface;)
- MODIFY: crates/orchestrator/src/ssb.rs (~80 LOC: ReviewSurfaceScored variant + payload (incl total_changed_lines) + sql_str + parse + CHECK + golden)
- MODIFY: crates/orchestrator/tests/action_type_lock.rs (~2 LOC: TOTAL_VARIANTS 482->483 + ALL_VARIANTS row) + tests/action_type_lock.golden (~1 row)
- MODIFY: crates/cli/src/args.rs (~20 LOC: "review" subcommand arm) + crates/cli/src/main.rs (~6 LOC: dispatch)
- MODIFY: crates/dashboard-v2/src/lib/api/types.ts + a window registration (~8 LOC)

## Citations / Grounding
- workflow/reasoning/ELAI-TRUST-MAXIMIZATION-2026-06-02.md R5. METR arXiv:2507.09089 (reviewing AI output is the time
  sink, 19% slowdown) CONFIRMED; arXiv:2604.03196 (code-review-agent PRs 0-30% signal in 60.2%) CONFIRMED; arXiv:2509.13650
  (naive LLM review style-biased, misses SQLi/XSS) CONFIRMED — the grounding for silent-by-default + rendering ELAI's own
  deterministic CWE-carrying verdicts (VibeGateBlockedPayload.cwe ssb.rs:25060) rather than naive LLM comments.
- ELAI ground truth (grep-confirmed 2026-06-02): TestOutcomePayload.stdout_hash ssb.rs:9877/:9887 (read at commands.rs:837/:9501);
  VibeGateBlockedPayload ssb.rs:25054/:25060/:25064 + SecurityRuleBlockedPayload.cwe:25103; GroundingSweepResultPayload
  ssb.rs:15287 / GroundingSweepResult:5238; ChangeDecisionPayload ssb.rs:15933 (hunk_id, C-7); FileWritePayload ssb.rs:9858
  ({path,size_bytes,hash} — NO line count); SsbViewer::file_changes ssb_viewer.rs:1387 / FileChange:159 ({path,action,summary});
  ReviewWindow.svelte get_file_changes:59 / commands.rs:10148 / lib.rs:775; args.rs parse_args:1718 / match:1936 / run-subcommand:2159.
  rra.rs ABSENT (G-2 forward dep). SCHEMA_VERSION 205 (ssb.rs:2898), TOTAL_VARIANTS 482 (tests/action_type_lock.rs:534).
- MEASUREMENT (the review-time number): a within-subjects screen-recorded clustered-SE study (METR design) of
  time-to-first-approve through the cockpit vs a plain diff, escaped-defect-rate as the guardrail. Speed-AND-safety
  simultaneity is the HYPOTHESIS the study TESTS (industry-wide unmeasured), NOT a pre-claimed result (GD-3).
- CLAUDE.md HARD RULES: blake3, no .unwrap()/.expect() in prod, Pattern-2 SSB, SCHEMA baseline+N, anti-slopsquatting.

## Open questions / risks
- GD-2 GAP (the load-bearing finding): changed-line counts are NOT in any current SSB row (FileWritePayload ssb.rs:9858
  is {path,size_bytes,hash}; FileChange ssb_viewer.rs:159 is {path,action,summary}). G-5 MUST add total_changed_lines to
  the new payload, sourced from the FileWrite diff, or DSR is not auditable. Do NOT claim DSR is already SSB-recomputable.
- The hunk_id join: ChangeDecision.hunk_id is derived from the `@@ -N,M +P,Q @@` header hash (C-7) — confirm at authoring
  that the FileWrite diff and the TestOutcome/gate rows can be joined to the SAME hunk_id, or vouching attaches at the
  wrong granularity (file-level fallback when hunk_id is None).
- The within-subjects study is a measurement, not the done-gate — do NOT block G-5 DONE on running the human study; the
  done-gate is the cockpit + the GD-2-recomputable DSR. The study is the follow-on that turns the hypothesis into a number.
- Do NOT cite rra.rs as present (G-2 forward dep); the cockpit reads the RRA manifest only once G-2 has shipped — until then
  it scores from the live SSB alone (the RRA enrichment is additive).

## Next
G-5 reads G-7's claim_confidence_scored rows to add a confidence-triage band (read low-confidence hunks first). G-2's RRA
evidence manifest enriches each hunk's vouch set off-machine (the cockpit over a replayed run). G-8's attestation signs the
DSR result. G-5 is the reviewer-facing surface that makes the whole F-gate + G-gate trust spine legible per change — the
"verified-before-seen" lever the synthesis names.

End of G-5.
