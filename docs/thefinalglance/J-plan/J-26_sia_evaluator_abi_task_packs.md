# J-26: sia evaluator ABI + task-pack format — the fitness-evaluator subprocess contract and the on-disk task convention for H-43/E-54

STATUS: PLANNED (WAVE RSW-1; .elai_cc runtime; EXTENSION — adopts two language-agnostic contracts from hexo-ai/sia into the H-43 execution-grounded fitness harness and the E-54 eval corpus; zero new subsystem)

- Track J (repo-steal extension) | lever PROVABLE | grounding STRONG (direct source read of
  hexo-ai/sia v0.5.1, MEDIUM reliability; paper arXiv:2605.27276 WebSearch-VERIFIED 2026-06-12)
  | nature EXTENSION | effort S-M | WAVE RSW-1

- Premise: sia (Hexo Labs' self-improving-AI framework, arXiv:2605.27276) runs the same loop
  shape ELAI's Darwinian/H-43 family plans — meta-agent writes agent, agent runs task, evaluator
  scores, feedback improves — with a notably clean OFF-AGENT evaluation contract that matches
  ELAI's wiring doctrine (R1/R4: artifact the agent did not author; missing evidence = FAIL).
  ELAI's loop architecture is stricter and already planned (H-43..H-48 frozen tests + traps +
  lineage; B1 verdict = confirmation, no port). The steal is the two CONTRACTS. Steal file:
  workflow/reasoning/steal-mimocode-sia-agentskills-2026-06-12.md (B2/B3; riders B1/B4).

## Steal/build
T1 EVALUATOR ABI (B2; sia EVALUATION_GUIDE.md + orchestrator.py:635-961). The fitness evaluator
  for H-43 (FitnessEvaluated, SCHEMA 221 family) is invoked as a SUBPROCESS with `--gen-dir
  <dir>`; it writes results.json INTO that dir; exit 0 = it ran; MISSING results.json = the
  evaluation never ran = hard FAIL, never a default score (R1/R2 verbatim). Top-level scalar
  fields of results.json are the only score channel the loop reads. The ABI is language-agnostic
  — evaluate.py today, a compiled gate binary tomorrow, zero loop change. ELAI delta over sia:
  the evaluator fileset is R8-FROZEN (gate-manifest), the candidate can never edit it — sia has
  no freeze; this is recorded as the DO-NOT-COPY residue of sia's design.
T2 TASK-PACK CONVENTION (B3; sia layout.py TaskLayout). On-disk format for fitness/eval tasks:
  data/public/task.md + data/public/<evaluator> + data/public/<fixtures> + data/private/<held-out>
  + reference/<sample solutions>. Adopted as the input format E-54's ingest_task (currently no
  CLI entrypoint — its known gap) and H-43's frozen-test corpus read; D-6 held-out rows map to
  data/private/. Free interop: sia-format packs run under ELAI unmodified, and ELAI fitness
  tasks are publishable as sia packs.
RIDERS (recorded, not tasks): B1 loop architecture = H-43-family confirmation; B4
  orchestrator-written append-only context.md from extracted metrics = SSB-journal confirmation
  (ELAI's record is stronger); B5 weights/RL mode = E-55/I-31 deferral stands (ledger); B6 log
  size guards = covered.

## Existing ground truth (PLAN-asserted from ELAI-FEATURE-MAP 2026-06-12; re-grep at edit, W-7)
- H-43 execution-grounded fitness [PARTIAL] FitnessEvaluated (221) — replaces the
  darwin_admit_loop.rs:186 blake3(config_bytes) placeholder (VD-1).
- E-54 EVAL CORPUS [DARK(ELAI_EVAL_CORPUS_EMBEDDINGS)] — ingest_task has no CLI entrypoint.
- D-6 HELD-OUT STORE [DARK, allowlisted]; H-47 eval firewall [PARTIAL]; G-6 SAFEDELTA
  [BENCH-PENDING]; F-11 one config-only generation (driver shipped).
- Source ground truth (W-3): docs/.source/sia/ clone + sia-INDEX.md section 8 +
  workflow/papers/sia-interface.md.

## REVISED scope (W-10 tiers)
SHIP NOW Tier-1 (in-process, fixture-driven): the ABI struct + subprocess runner + results.json
  reader with missing=FAIL semantics, wired where H-43's fitness evaluation dispatches; the
  task-pack reader + ONE golden fixture pack under the eval corpus root; the E-54 ingest CLI
  entrypoint reading packs (closes its named gap). DEFER Tier-2: any LIVE model-in-the-loop
  generation run (RunPod/remote-gated per W-10; F-11's checkbox discipline applies).

## Integration
- crates/orchestrator/src/ (H-43 fitness seam; new evaluator_abi module ~150 LOC: TaskPack +
  EvaluatorInvocation + ResultsJson types, subprocess runner, missing-artifact FAIL).
- crates/cli eval-corpus ingest arm (E-54 entrypoint, ~60 LOC).
- ONE golden task-pack fixture (small, deterministic evaluator that writes results.json).
- frontend_pair: N/A — no new dashboard surface; fitness rows surface through existing
  H-43/darwinian panels. Justified.

## WIRING_CONTRACT
produces_symbols:
- evaluator_abi module (TaskPack layout reader, subprocess invoke, ResultsJson reader,
  missing=FAIL verdict)
- E-54 ingest CLI entrypoint consuming task packs
- golden fixture pack + its contract test
consumes_symbols:
- H-43 FitnessEvaluated emission path (the ABI feeds it; do NOT re-implement the fitness scorer),
- E-54 ingest_task, D-6 held-out store rows, gate-manifest R8 freeze (the evaluator fileset
  registers there), F-11 generation driver (its evaluator call routes through the ABI).
reachable_from:
- the darwinian admit loop fitness call (PRODUCTION when ELAI_DARWINIAN_LOOP_ENABLE per its
  existing allowlist row) + the E-54 ingest CLI verb (PRODUCTION, unconditional). Contracts/-only
  callers never satisfy this (R7).
stub_ban:
- NO default/fallback score on missing results.json (missing = FAIL, R1); NO evaluator file
  writable by the candidate-producing agent (R8); NO LLM in the ABI path (TD-3); NO
  .unwrap()/.expect() on prod paths; NO blake2/sha2.
observable_effect:
- trigger: "`elai <eval-corpus ingest verb> --pack <golden-pack>` then the fitness harness run
  over the golden pack via the existing darwinian/F-11 driver (flag-gated)"
- expect_ssb: "FitnessEvaluated row whose score equals the golden results.json scalar; eval-corpus
  ingest rows for the pack"
- expect_artifact: "results.json present in the gen dir with the expected fields; NEGATIVE (R2):
  delete results.json (or make the evaluator exit nonzero) and the run must produce a hard FAIL
  verdict — a default-zero score or a SKIP is a gate bug."

## SHIP-ORDER MICRO-SLICES (vertical slices, executable on existing seams; S1/S2-verified)

Each slice is a MOVE with EXPECTED_OBSERVATION + ON_FAIL. Slices S2+S3 ship together (producer +
real production consumer in the same changeset, R6/R7). S5+S6 ship together (fixture + the test
that asserts against it). S7 is OUT-OF-BAND (R8 re-lock fork — surface to orchestrator).

PRODUCTION_HOOK seams (S1/S2-verified live):
- darwin_admit_loop.rs:1153-1157 (h43_fitness computation in run_one_generation) — the ABI
  channel branch goes here, feeds the SAME emit_fitness_row at :1173-1207.
- darwin_admit_loop.rs:1577-1589 (emit_fitness_row) — REUSED, not re-authored.
- darwin_loop_cmd.rs:101 (darwinian_admit, F-11 entry, flag-gated) — the production entrypoint.
- ingest.rs:250 (ingest_task, library fn, no CLI) — the gap J-26 closes with a CLI verb.
- args.rs:75 (CliAction enum) + args.rs:3344 (parse arm) — additive variant + parse arm.
- lib.rs:738 (pub mod declarations) — one additive `pub mod evaluator_abi;` line.

### Slice S1: workflow/reasoning/J-26-decisions.md (prose, pre-EXECUTING soft req)
MOVE: author the decisions doc recording the Tier-1 scope, the H-43 respect boundary (ABI is a
  sibling channel, not a scorer replacement), the R8 re-lock fork flag, and the GAP-1
  NewTaskInput converter mapping.
EXPECTED_OBSERVATION: `test -f workflow/reasoning/J-26-decisions.md` exits 0; file contains
  sections for scope, H-43 boundary, R8 fork, GAP-1 mapping.
ON_FAIL: file missing -> cause: executor skipped pre-EXECUTING soft req -> counter-move: author
  it before touching any .cs/.rs file; the executor may not proceed to S2 without it.

### Slice S2: crates/orchestrator/src/evaluator_abi.rs (NEW, ~150 LOC, the producer)
MOVE: create evaluator_abi.rs with TaskPack (layout reader for data/public/task.md + evaluator +
  fixtures + data/private/held-out + reference/), EvaluatorInvocation (--gen-dir subprocess
  runner via std::process::Command, no .unwrap() on prod paths, no blake2/sha2), ResultsJson
  reader (top-level scalar fields only), and missing-results.json = hard FAIL verdict (Err arm,
  no default-zero, no SKIP). Add `pub mod evaluator_abi;` to lib.rs:738 area. NO wiring into
  darwin_admit_loop yet — WIRE-DARK[J-26] marker on the pub fn until S3 wires it.
EXPECTED_OBSERVATION: `rg "results\.json" .elai_cc/crates/orchestrator/src/evaluator_abi.rs`
  returns the file; `rg -c "unwrap\(\)" .elai_cc/crates/orchestrator/src/evaluator_abi.rs` = 0;
  `rg "WIRE-DARK\[J-26\]" .elai_cc/crates/orchestrator/src/evaluator_abi.rs` returns the marker
  line above the pub fn; `cargo check -p elai-orchestrator` compiles clean.
ON_FAIL: cargo check fails -> cause: type mismatch or missing import -> counter-move: read the
  ExecutionFitness struct at execution_fitness.rs:156 and the FitnessError enum, align the
  ResultsJson-to-ExecutionFitness conversion; re-run cargo check. If the conversion is
  structurally impossible (ResultsJson fields don't map to ExecutionFitness), STOP + handoff
  (R-4) — do not force-fit.

### Slice S3: crates/orchestrator/src/darwin_admit_loop.rs (EDIT, ~10-20 LOC, the consumer)
MOVE: add a config-gated branch in run_one_generation at the h43_fitness computation site
  (darwin_admit_loop.rs:1153-1157) that, when a TaskPack evaluator is supplied for this
  generation, invokes evaluator_abi subprocess instead of the in-process scorer, converts
  ResultsJson -> ExecutionFitness, and feeds the SAME h43_fitness variable. The
  emit_fitness_row call at :1173-1207 is unchanged. The in-process path stays as fallback when
  no pack is supplied (RISK-3). Remove the WIRE-DARK[J-26] marker from S2.
EXPECTED_OBSERVATION: `rg "evaluator_abi" .elai_cc/crates/orchestrator/src/darwin_admit_loop.rs`
  returns the call-site; `cargo check -p elai-orchestrator` compiles clean;
  `python3 .claude/tools/production_caller_check.py --files .elai_cc/crates/orchestrator/src/evaluator_abi.rs .elai_cc/crates/orchestrator/src/darwin_admit_loop.rs`
  exits 0 (no DARK symbols); the WIRE-DARK marker is gone.
ON_FAIL: production_caller_check exits 1 (DARK) -> cause: the call-site branch is behind a
  cfg(test) or the symbol is not reachable from a production root -> counter-move: verify the
  branch is in the non-test run_one_generation fn and that darwinian_admit (darwin_loop_cmd.rs:101)
  calls run_one_generation; re-run the checker. If the branch is structurally unreachable (the
  config flag is never set in production), STOP + handoff (R-4).

### Slice S4: crates/cli eval-corpus ingest verb (NEW, ~60 LOC, second vertical slice)
MOVE: add CliAction::EvalCorpusIngest { pack_path, project_root, output_format } variant to
  args.rs:75 area (additive, mirrors DarwinLoop at args.rs:1411); add parse arm
  "eval-corpus-ingest" | "ingest-task-pack" => parse_eval_corpus_ingest_args(...) at args.rs:3344
  area; create crates/cli/src/eval_corpus_ingest_cmd.rs (~60 LOC, thin wrapper that builds a
  NewTaskInput from the TaskPack and calls elai_eval_corpus::ingest::ingest_task); add one
  match arm in main.rs dispatching to the new cmd.
EXPECTED_OBSERVATION: `rg "EvalCorpusIngest" .elai_cc/crates/cli/src/args.rs` returns the
  variant; `rg "ingest-task-pack" .elai_cc/crates/cli/src/args.rs` returns the parse arm;
  `test -f .elai_cc/crates/cli/src/eval_corpus_ingest_cmd.rs` exits 0; `cargo check -p elai-cli`
  compiles clean; `python3 .claude/tools/production_caller_check.py --files .elai_cc/crates/cli/src/eval_corpus_ingest_cmd.rs`
  exits 0 (the CLI entrypoint is the production caller for ingest_task).
ON_FAIL: cargo check fails on NewTaskInput shape -> cause: GAP-1, the TaskPack-to-NewTaskInput
  converter has a field mismatch -> counter-move: read task_schema.rs (eval-corpus) for the
  NewTaskInput.task schema, map task.md -> NewTask.problem, held-out rows -> prior_signatures;
  re-run cargo check. If the schema is incompatible (required field has no pack source), STOP +
  handoff (R-4) with the GAP-1 question.

### Slice S5: Golden task-pack fixture (NEW dir, deterministic, no LLM)
MOVE: create the golden fixture pack under .elai_cc/crates/eval-corpus/fixtures/golden-pack/
  with data/public/task.md + data/public/evaluator (a deterministic python or bash script that
  writes results.json with a known scalar, no LLM, no network) + data/public/fixtures/ +
  data/private/<held-out rows mapping to D-6> + reference/<sample solutions>.
EXPECTED_OBSERVATION: `find .elai_cc/crates/eval-corpus/fixtures/golden-pack -type f` returns
  task.md, evaluator, fixtures, data/private/*, reference/*; running the evaluator script
  locally writes results.json with the expected scalar.
ON_FAIL: evaluator script fails locally -> cause: script bug or missing dependency ->
  counter-move: fix the script (it is a deterministic fixture, not production code); re-run.
  If the script requires a dependency not in the repo, STOP + handoff (R-4) — no new deps
  without owner approval.

### Slice S6: Contract test (NEW, the R2 negative is load-bearing)
MOVE: create crates/contracts/tests/j26_evaluator_abi_contract.rs asserting (1) POSITIVE:
  golden pack ingests via the CLI verb, fitness run over golden pack produces FitnessEvaluated
  row with score == golden results.json scalar; (2) NEGATIVE (R2): delete results.json OR make
  evaluator exit nonzero -> hard FAIL verdict (no default-zero, no SKIP).
EXPECTED_OBSERVATION: `test -f .elai_cc/crates/contracts/tests/j26_evaluator_abi_contract.rs`
  exits 0; the test file contains both a positive and a negative assertion; `cargo test -p
  elai-contracts --test j26_evaluator_abi_contract` passes (auditor runs this, not the
  executor).
ON_FAIL: negative test passes against unfixed code (not a real reproducer) -> cause: the FAIL
  verdict path is not wired (R2 broken arm) -> counter-move: verify the Err arm in
  evaluator_abi.rs propagates to a hard FAIL, not .ok() suppression at darwin_admit_loop.rs:1154;
  fix the propagation; re-run. If the FAIL surface cannot be wired without re-authoring the
  emit path, STOP + handoff (R-4) — that is an architecture finding, not a bolt-on.

### Slice S7: gate-manifest R8 registration (OUT-OF-BAND, R8 re-lock fork)
MOVE: add evaluator_abi.rs + golden fixture paths to .claude/gate-manifest.txt; re-run
  gate_freeze_check.py --lock OUT-OF-BAND (human-reviewed, NOT by the executing plan run).
EXPECTED_OBSERVATION: gate-manifest.txt contains the new paths; gate_freeze_check.py --lock
  exits 0; the evaluator fileset is frozen.
ON_FAIL: gate_freeze_check.py --lock fails -> cause: the re-lock was not pre-authorized or the
  manifest format changed -> counter-move: surface to orchestrator as a material fork (R-4);
  do NOT edit gate-manifest.txt without the re-lock authorization (CLAUDE.md: "never edit a file
  in .claude/gate-manifest.txt without the re-lock").
ABORT: this slice is a material fork — if the re-lock is not pre-authorized by the orchestrator,
  STOP + handoff (R-4). The executor does not self-authorize gate edits.

### Slice S8: Close-out (production_caller_check + decisions update)
MOVE: re-run production_caller_check.py over the full changeset to confirm the three new symbols
  (TaskPack, EvaluatorInvocation, ResultsJson) are WIRED (not DARK); update
  workflow/reasoning/J-26-decisions.md with the close-out status.
EXPECTED_OBSERVATION: `python3 .claude/tools/production_caller_check.py --files <all changed
  .rs>` exits 0 (no DARK symbols); decisions doc has a close-out section.
ON_FAIL: checker exits 1 (DARK symbol) -> cause: a symbol's only caller is a test file (R7) ->
  counter-move: wire the symbol into a production entrypoint or add a WIRE-DARK[J-26] marker +
  allowlist row; re-run. If wiring is structurally impossible, STOP + handoff (R-4).

## FORK
FORK-1: trigger = GAP-1 (NewTaskInput.task schema mismatch when building from TaskPack). If the
  task.md -> NewTaskInput mapping has a required field with no pack source, take route B: STOP +
  handoff (R-4) with the GAP-1 question and the exact field mismatch; do not force-fit or invent
  a default. Route A (happy path): the mapping is clean, proceed with S4.
FORK-2: trigger = S7 re-lock not pre-authorized. If the orchestrator has not pre-authorized the
  gate-manifest re-lock, take route B: skip S7 in this changeset, surface as a material fork in
  the handoff, and let the orchestrator schedule the re-lock out-of-band. Route A: re-lock is
  pre-authorized, proceed with S7.
FORK-3: trigger = R2 negative test cannot be wired without re-authoring emit_fitness_row. If the
  FAIL verdict surface requires touching the emit path (H-43 respect boundary), take route B:
  STOP + handoff (R-4) with the architecture finding (the seam is wrong, the ABI needs its own
  FAIL verdict surface). Route A: the Err arm propagates through the existing .ok() -> None path
  and the contract test asserts the None row, proceed with S6.

## RECON_NEEDED
RECON-1: NewTaskInput.task schema (GAP-1). Settling check: `rg "struct NewTaskInput" -A 20
  .elai_cc/crates/eval-corpus/src/task_schema.rs` and `rg "struct NewTask " -A 20
  .elai_cc/crates/eval-corpus/src/task_schema.rs`. Map every required field to a TaskPack source.
  If any required field has no pack source, FORK-1 fires.
RECON-2: CliAction enum exhaustiveness. Settling check: `rg "enum CliAction" -A 5
  .elai_cc/crates/cli/src/args.rs` and confirm the new variant does not break match exhaustiveness
  (no #[non_exhaustive] on internal dispatch enums, per wiring-enforcement Layer 1).
RECON-3: results.json schema (sia top-level scalar fields). Settling check: read
  docs/.source/sia/EVALUATION_GUIDE.md for the results.json field contract; confirm the
  ResultsJson reader parses only top-level scalars (not nested objects) per the sia ABI spec.
RECON-4: blake3 vs blake2/sha2 (stub_ban). Settling check: `rg "blake2|sha2"
  .elai_cc/crates/orchestrator/src/evaluator_abi.rs` must return 0 matches; the corpus hash uses
  blake3 (darwin_admit_loop.rs:281 derived_alpha), not blake2/sha2.

## ABORT
ABORT-1: cargo check fails on S2 (evaluator_abi.rs) after 2 attempts to align the
  ResultsJson-to-ExecutionFitness conversion -> STOP + handoff (R-4). The conversion is
  structural, not a syntax fix.
ABORT-2: production_caller_check.py exits 1 (DARK) on S3 after the call-site branch is verified
  present -> STOP + handoff (R-4). The symbol is unreachable from a production root; the
  orchestrator re-scopes.
ABORT-3: the R2 negative test (S6) passes against unfixed code -> STOP + handoff (R-4). The
  test is not a reproducer; the FAIL verdict path is not wired.
ABORT-4: S7 re-lock not pre-authorized -> STOP + handoff (R-4). The executor does not
  self-authorize gate edits.
ABORT-5: the evaluator script (S5) requires a dependency not in the repo -> STOP + handoff
  (R-4). No new deps without owner approval.

## VERIFY
VERIFY-1 (after S2+S3): `cargo check -p elai-orchestrator` exits 0 (compile clean). Off-agent
  artifact: the cargo check exit code (not executor stdout).
VERIFY-2 (after S2+S3): `python3 .claude/tools/production_caller_check.py --files
  .elai_cc/crates/orchestrator/src/evaluator_abi.rs
  .elai_cc/crates/orchestrator/src/darwin_admit_loop.rs` exits 0 (no DARK symbols). Off-agent
  artifact: the checker's exit code + DARK symbol list (frozen tool, R8).
VERIFY-3 (after S4): `cargo check -p elai-cli` exits 0. Off-agent artifact: cargo check exit
  code.
VERIFY-4 (after S4): `python3 .claude/tools/production_caller_check.py --files
  .elai_cc/crates/cli/src/eval_corpus_ingest_cmd.rs` exits 0 (ingest_task has a production
  caller via the CLI entrypoint, not a test shim). Off-agent artifact: checker exit code.
VERIFY-5 (after S5+S6, auditor-run): `cargo test -p elai-contracts --test
  j26_evaluator_abi_contract` passes both positive and negative cases. Off-agent artifact: the
  NUnit/JUnit XML or cargo test TAP output the auditor reads (not executor self-report).
VERIFY-6 (after S7, out-of-band): `python3 .claude/tools/gate_freeze_check.py --lock` exits 0.
  Off-agent artifact: the frozen checker's exit code.
VERIFY-7 (after S8): `python3 .claude/tools/production_caller_check.py --files <all changed
  .rs>` exits 0 (all three new symbols WIRED). Off-agent artifact: checker exit code + symbol
  status table.
VERIFY-8 (R2 negative, load-bearing): the contract test's negative case (delete results.json OR
  evaluator nonzero exit) produces a hard FAIL verdict — no default-zero, no SKIP. Off-agent
  artifact: the test's assertion failure message when the FAIL path is broken (the test is red
  before the fix, green after — bug-fix-protocol step 1).

## REDTEAM
REDTEAM attack 1 (FAILED): "the candidate-producing agent edits the evaluator fileset to game
  the score." Why it failed against the plan: the evaluator fileset is R8-fROZEN
  (gate-manifest registration in S7); gate_freeze_check.py denies any edit by the executing run
  (R8: the gate is frozen and read-only to the executing run). The candidate cannot rewrite its
  own evaluator. PATCH: none needed — the R8 freeze is the defense; S7 registers the fileset.
REDTEAM attack 2 (FAILED): "missing results.json silently defaults to score 0, letting the
  candidate pass with a non-run." Why it failed: the plan's stub_ban explicitly forbids a
  default/fallback score (R1/R2); the Err arm in evaluator_abi.rs propagates to a hard FAIL
  verdict, and the S6 contract test asserts this (red-then-green). PATCH: none needed — the
  missing=FAIL semantics + the negative test are the defense.
REDTEAM attack 3 (SUCCEEDED, patched): "the ABI channel replaces the in-process scorer, breaking
  Tier-1 tests that supply an in-process corpus." Why it succeeded: the original plan wording
  ("wired where H-43's fitness evaluation dispatches") was ambiguous about replacement vs
  sibling. PATCH: the refined plan explicitly states the ABI is a SIBLING channel
  (config-gated branch at darwin_admit_loop.rs:1153), the in-process path stays as fallback when
  no pack is supplied (RISK-3), and the scorer (execution_grounded_fitness_for_candidate) is
  untouched (H-43 respect). The branch selects between channels; it does not replace.
REDTEAM attack 4 (SUCCEEDED, patched): "the executor edits gate-manifest.txt during the plan
  run, weakening the R8 freeze that judges it." Why it succeeded: S7 was listed as a plan slice
  without flagging the re-lock authorization requirement. PATCH: S7 is now marked OUT-OF-BAND
  with ABORT-5 + FORK-2; the executor must surface the re-lock as a material fork (R-4) if not
  pre-authorized, and never self-authorizes gate edits.

## INTEGRATION_SMOKE
INTEGRATION_SMOKE-1 (after S2+S3+S4, the first 3 code units): drive the REAL entrypoint
  `elai eval-corpus-ingest --pack <golden-pack>` (S4 CLI verb) THEN the fitness harness run over
  the golden pack via the existing darwinian/F-11 driver (flag-gated, ELAI_DARWINIAN_LOOP_ENABLE
  set). HARD input: the golden fixture pack (S5) with a deterministic evaluator that writes a
  known scalar. Assert: (a) ingest rows land in the SSB for the pack, (b) FitnessEvaluated row
  carries score == golden results.json scalar, (c) results.json present in the gen dir. This
  drives the REAL CLI entrypoint + the REAL fitness dispatch seam end-to-end, not a test shim
  calling evaluator_abi directly. Off-agent artifact: the SSB rows (FitnessEvaluated +
  EvalCorpusEvent) + the results.json file on disk.
INTEGRATION_SMOKE-2 (before DONE, after S5+S6): the R2 negative smoke — delete results.json
  from the gen dir, re-run the fitness harness, assert a hard FAIL verdict (no FitnessEvaluated
  row with a default-zero, no SKIP). This is the cross-seam negative that per-unit green cannot
  catch (the per-unit test asserts the FAIL in isolation; the integration smoke asserts it
  through the real entrypoint). Off-agent artifact: the absence of a FitnessEvaluated row with
  default-zero score in the SSB + the FAIL verdict log line.

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/J-plan/J-26_sia_evaluator_abi_task_packs.md --run-trigger
rg "results\.json" .elai_cc/crates/orchestrator/src/ -l            # ABI reader exists
rg -c "unwrap\(\)" .elai_cc/crates/orchestrator/src/evaluator_abi* # MUST be 0 (adjust to real path at edit)
test -f workflow/reasoning/J-26-decisions.md
```

## Grounding
- Source: sia clone v0.5.1 MIT (MEDIUM), sia-INDEX.md sections 8.1-8.4 + sia-interface.md
  behavioral contracts ("evaluate.py must accept --gen-dir; write results.json into gen_dir;
  exit 0"; "writing elsewhere causes silent status=warning" = the sia bug ELAI's missing=FAIL
  fixes). Paper: arXiv:2605.27276 (Hebbar et al. 2026) WebSearch-verified 2026-06-12.
- ELAI anchors PLAN-asserted from the feature map; re-grep at edit (W-7).
- Doctrine that bites: VD-1 (fitness execution-grounded, never bytes), R8 frozen evaluator
  fileset, W-10 live-run split, TD-3.
- SSB: NO new action_type preferred — reuse H-43 FitnessEvaluated (221) + E-54 ingest rows;
  net 0 (decide at edit; W-9 re-grep LIVE 231/542 first if a row is added).
- S1/S2 evidence: conductor/opus_walk/evidence/J-26/S1-context.md (fitness dispatch seam at
  darwin_admit_loop.rs:1153/1173/1582, ingest_task at ingest.rs:250, force_alpha precedent at
  :333-346, confirmed-absence grep hits) + conductor/opus_walk/evidence/J-26/S2-integrate.md
  (H-43 respect verified, blast radius 0 existing files except lib.rs + darwin_admit_loop.rs +
  args.rs + main.rs, R9 allowlist row at wire-dark-allowlist.toml:109-115, ship-order
  micro-slices S1-S6, GAP-1 NewTaskInput converter, RISK-1/2/3).

## COMPOSES-WITH
H-43 (the consumer), E-54, D-6, H-47, F-11, G-6 (packs become SAFEDELTA fixture inputs),
gate-manifest R8. Restates none; the sia LOOP is explicitly NOT ported (confirmation only).

End of J-26.
