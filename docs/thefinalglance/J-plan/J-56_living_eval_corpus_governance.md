# J-56: living eval corpus — failure-ingest entrypoint + problem-type taxonomy + CI subset-vs-full-on-merge governance (activate the dark E-54 corpus, make "the test set is a living system" real)

STATUS: PLANNED (WAVE PV-1; effort M; ACTIVATION — turns the dark E-54 corpus + the never-invoked G-6 SAFEDELTA into a growing, categorized, CI-governed eval system; no new engine)

Objective: the Five-Pillars "evaluation is the spec, and the test-case library is a LIVING governed system that grows from caught failures, categorized by problem-type, run as a SUBSET in CI on change and FULL on merge" maps onto ELAI substrate that already exists but is DARK: EVAL CORPUS DESIGN (E-54 — 13-gram contamination filter + per-family shards, but ingest_task has NO CLI entrypoint, feature-map ~1331) and SAFEDELTA (G-6 — CLI + dashboard wired, bench NEVER invoked, 0 safety_delta_measured rows, ~1324). J-56 ships the activation: an `elai eval ingest` production entrypoint that captures a caught failure as a categorized corpus row, a problem-type taxonomy (mirroring the gotchas/audit failure families), and a CI governance hook (subset-on-change, full-on-merge — the Five-Pillars lesson that Layer-3 evals get expensive so you gate them). HARD BOUNDARY: the corpus scores against DETERMINISTIC gate verdicts, NEVER an LLM-as-judge (TD-3) — the talk's Layer-2 LLM-judge stays rejected; J-56 is the Layer-1/Layer-3 (deterministic + behavioural) corpus. GD-1 do-NOT-re-implement: E-54 owns the 13-gram filter + shards; G-6 owns the gate-ON-vs-OFF ablation; D-6 owns the held-out store; this is the ingest + taxonomy + CI seam only.

Estimated effort: ~1-2 sessions (~8 h). In-process (the ingest, the taxonomy classifier, the CI subset selector are pure logic over recorded rows + on-disk fixtures); a FULL corpus run is bench-tier (the SAFEDELTA/held-out harness is RunPod/sidecar-gated, W-10) but the ingest + governance legs run at gate time.
Pre-requisites (re-grep at edit, W-7): E-54 corpus design + ingest_task SHIPPED-dark (no CLI); G-6 SAFEDELTA CLI + safety_delta_measured SSB SHIPPED; D-6 held-out store SHIPPED; the gotchas/audit failure taxonomy (.claude/tools/failure_taxonomy.py 13 categories — the category seed).
Parallel-safe with: J-58, J-59 (disjoint files). J-57 DEPENDS on J-56 (the eval gate it runs).
Source: workflow/reasoning/steal-production-ai-talks-2026-06-20.md (Five-Pillars Pillar-1 + lessons 1/3 + Anthropic "grow the corpus from failures"). Doctrine: J-0_INDEX.md.

## Context
Why this matters (external evidence, scope-tagged per HD-1): the Five-Pillars case study's load-bearing claim is that the eval dataset is a LIVING system — "start at 200 cases, it grows, the bigger it grows the better the system gets", with three governance lessons: (1) the corpus needs an OWNER and rows CATEGORIZED by problem type so you can trace what changed against the right class; (3) Layer-3 (behavioural) evals get expensive at scale, so CI runs a SUBSET on each change and the FULL suite only on merge to main. [DESIGN, vendor-framework — directional, the numbers are illustrative not measured]. the Anthropic talk reinforces the mechanism from the other side: "embed prompting that writes learnings to a JSON file, leave breadcrumbs — tried this -> evaluated -> found this bug -> fixed -> fix worked". ELAI already has the contamination filter and the gate-ablation harness; what is missing is the LOOP that feeds a caught failure back IN as a permanent, categorized regression row, and the CI discipline that keeps the full suite affordable. correctness boundary: scoring is DETERMINISTIC (gate verdicts / behavioural tool-sequence checks), the LLM never judges a corpus row (TD-3) — that is the one place ELAI deliberately diverges from the talk's Layer-2.

## Existing state (B/C/D/E/F/G/H/I/J coverage — W-7 anchors, nothing from memory)
- EVAL CORPUS DESIGN [DARK(ELAI_EVAL_CORPUS_EMBEDDINGS), E-54]: 13-gram contamination filter + per-family shards; `ingest_task` has NO CLI entrypoint (feature-map ~1331-1333). J-56 gives ingest_task its production CLI.
- SAFEDELTA [BENCH-PENDING, G-6]: gate-ON-vs-OFF ablation over SusVibes/BaxBench/SeCodePLT/AgentDojo; CLI + dashboard wired, bench never invoked, 0 safety_delta_measured rows (feature-map ~1324-1328). J-56 makes the corpus the bench reads a GROWING one.
- HELD-OUT STORE [DARK, D-6]: time-windowed held-out + 9/13-gram filter; sample_held_out_task test-only (feature-map ~637). The corpus respects the held-out split (no leakage from regression rows into held-out).
- failure_taxonomy.py (13 deterministic regex categories, CLAUDE.md harness-tools): the seed for the problem-type taxonomy (security / grounding / tool-sequence / regression / wiring-dark / ...).
- gotchas.md (the human failure ledger): the existing, ungoverned, uncategorized-for-eval version of what J-56 formalizes.

## Reads (mandatory before coding)
1. workflow/reasoning/steal-production-ai-talks-2026-06-20.md (the Pillar-1 + lessons 1/3 verdict + the TD-3 boundary).
2. the E-54 detail file + the ingest_task signature (the dark function to wire); crates/knowledge or crates/orchestrator (re-grep where ingest_task lives).
3. the G-6 SAFEDELTA CLI + safety_delta_measured SSB emit site; D-6 held-out split API.
4. .claude/tools/failure_taxonomy.py (the 13-category seed) + workflow/memory-bank/gotchas.md format.
5. CLAUDE.md: TD-3 (deterministic scoring, no LLM-judge on a gate), W-2 kill-switch, named constants, blake3 (the row content-hash + 13-gram keys are blake3, never sha2).

## Tasks (WAVE PV-1 — the ingest entrypoint + taxonomy + CI governance)
1. `elai eval ingest` CLI (~90 LOC): capture a caught failure as a corpus row {task_input, expected (gate verdict / behavioural assertion), observed, category, source_session, provenance_hash}. Source modes: `--from-session <sid>` (pull the failing trace from SSB) or `--from-fixture <path>`. Runs the E-54 13-gram contamination filter BEFORE insert (a row overlapping the held-out set or an existing row is REJECTED). NOT: an LLM scoring the row; NOT: ingesting without a category.
2. Problem-type taxonomy (~70 LOC): a closed enum of eval categories seeded from failure_taxonomy.py (security / grounding / tool-sequence / regression / wiring-dark / format / ...). Every corpus row carries exactly one category; an uncategorized row is REJECTED at ingest (lesson 1 — categorized so you trace against the right class). NOT: a free-text tag; NOT: a model-assigned category (deterministic classifier or explicit `--category`).
3. CI governance selector (~80 LOC): `elai eval run --scope subset|full`. subset = the categories touched by the current diff (cheap, on-change); full = the whole corpus (on merge to main). The subset selector reads the diff's touched subsystems (reuse the J-16 blast-radius cone / B-29 impact API if shipped, else a path->category map). NOT: running the full suite on every change (the expensive failure mode lesson-3 names).
4. SSB EvalCorpusEvent (~50 LOC): Pattern-2, 3-arm EvalCorpusEventKind { RowIngested{category, provenance_hash}, RowRejected{reason: contamination|uncategorized}, SuiteRun{scope, pass, fail} }; golden-lock lockstep.
5. Decision doc: workflow/reasoning/J-56-decisions.md.

## WIRING_CONTRACT
produces_symbols:
  - the `elai eval ingest` + `elai eval run --scope` subcommands (the corpus entrypoints)
  - the EvalCategory taxonomy enum + the deterministic categorizer
  - the CI subset selector (diff-touched-categories -> the subset to run)
  - crates/.../ssb.rs::SsbActionType::EvalCorpusEvent (3-arm)
consumes_symbols:
  - E-54 ingest_task + the 13-gram contamination filter (the dark fn this wires; GD-1 do NOT re-implement the filter)
  - G-6 SAFEDELTA harness (the corpus is the dataset it ablates over; GD-1)
  - D-6 held-out split (the contamination guard against held-out leakage; GD-1)
  - .claude/tools/failure_taxonomy.py categories (the taxonomy seed)
  - J-16 blast-radius cone / B-29 impact API for the subset selector (if shipped; else a path->category map)
reachable_from:
  - the `elai eval ingest` CLI subcommand (PRODUCTION — an operator/auditor ingests a caught failure)
  - the `elai eval run --scope` CLI invoked by CI (PRODUCTION — the governance entrypoint)
  - a #[cfg(test)] caller NEVER satisfies this (R7)
stub_ban:
  - NO LLM-as-judge scoring a corpus row or assigning a category (TD-3 — deterministic gate verdict only)
  - NO uncategorized row admitted (lesson-1 — every row has exactly one EvalCategory)
  - NO full-suite-on-every-change (the subset selector is the lesson-3 cost guard)
  - NO re-implementation of the E-54 13-gram filter / G-6 ablation / D-6 held-out split (compose)
  - NO sha2 for the row content-hash / n-gram keys (blake3, CLAUDE.md anti-stale)
  - NO .unwrap()/.expect() on the ingest / run prod path
frontend_pair:
  N/A with justification — the corpus surfaces through the EXISTING SAFEDELTA dashboard tile (G-6); J-56 adds rows + run-scope to it, no new Tauri command. Justified.
observable_effect:
  trigger: "elai eval ingest --from-session <a real failing session id> --category grounding  ;  then  elai eval run --scope subset"
  expect_ssb: "an EvalCorpusEvent::RowIngested{category=grounding, provenance_hash} then an EvalCorpusEvent::SuiteRun{scope=subset, pass, fail} with the ingested row exercised"
  expect_artifact: "a new categorized row in the corpus store + the subset run report. NEGATIVE that must FAIL: ingesting a row that 13-gram-overlaps the held-out set (must RowRejected{contamination}); ingesting with no category (must RowRejected{uncategorized}); a subset run that silently expands to the full corpus."

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/J-plan/J-56_living_eval_corpus_governance.md --run-trigger
rg -n "eval ingest|EvalCategory|EvalCorpusEvent|scope.*subset" crates   # the entrypoints + taxonomy exist
rg -c "llm_judge|judge_model|gpt|claude_chat" crates/.../eval_corpus*    # MUST be 0 (TD-3, no LLM scoring)
rg -c "sha256|blake2" crates/.../eval_corpus*                            # MUST be 0 (blake3 only)
rg -c "unwrap\(\)|expect\(" crates/.../eval_corpus*                      # MUST be 0 (prod path)
test -f workflow/reasoning/J-56-decisions.md
```

## ACCEPTANCE
- [ ] TD-3: corpus rows scored by deterministic gate verdicts / behavioural assertions; NO LLM-judge (rg count 0).
- [ ] LESSON-1: every row carries exactly one EvalCategory; uncategorized ingest -> RowRejected (contract-tested).
- [ ] LESSON-3: `--scope subset` runs only diff-touched categories; full runs only on the merge path (contract-tested).
- [ ] R7/D-030: ingest + run reachable from the CLI (production), not a test.
- [ ] GD-1: E-54 filter / G-6 ablation / D-6 held-out / failure_taxonomy categories consumed, not restated.
- [ ] W-2: corpus governance ships ON (it is a CLI + CI hook, no enable-flag graveyard); E-54's embeddings flag is unchanged.
- [ ] cargo check --workspace GREEN; auditor PASS on -p the touched crate only.
- [ ] No new dependency.

## WIRING DONE-GATE (clause by clause)
- W-1: entrypoint = `elai eval ingest` / `elai eval run`; artifact = EvalCorpusEvent rows. Met.
- W-2: CLI + CI hook (no new ELAI_*_ENABLE). Met.
- W-3: the talk verdict reproduced in-repo (the steal doc, W-3). Met.
- W-4: measures the objective (a real categorized row landed + the subset actually ran the right categories), not stdout. Met.
- W-5: production_caller_check over the new fns; called from the CLI not a test. Met.
- W-6: rides the G-6 SAFEDELTA tile; no new surface. Justified.
- W-7: E-54/G-6/D-6/failure_taxonomy anchors re-grepped at edit.
- W-9: SCHEMA baseline re-read at edit (EvalCorpusEvent = baseline+1).
- W-10: ingest + governance in-process; the FULL corpus ablation is RunPod/sidecar-gated (G-6). Tiered.
- GD-4: the subset selector defaults permissive (runs MORE categories than the strict diff cone) until its FP/miss rate is observed across >=3 real diffs; GD-5: the corpus quality is unmeasured until rows accumulate — ship the loop, the numbers come from use.

## Tests to add
- crates/contracts/tests/j56_eval_corpus_contract.rs (~100 LOC): (a) ingest a well-formed failure -> RowIngested with the right category; (b) ingest overlapping the held-out set -> RowRejected{contamination}; (c) ingest with no category -> RowRejected{uncategorized}; (d) `--scope subset` over a diff touching only "grounding" runs only grounding rows; (e) `--scope full` runs all categories; (f) corpus row content-hash is blake3; (g) NO LLM-judge call on any scoring path.

## Files touched
- CREATE: crates/.../eval_corpus.rs (~200 LOC: ingest, taxonomy, subset selector) — re-grep the owning crate at edit
- MODIFY: crates/cli/src/main.rs (~60 LOC: `elai eval ingest` / `elai eval run` subcommands)
- MODIFY: crates/.../ssb.rs (EvalCorpusEvent 3-arm + golden lock) (~50 LOC)
- CREATE: crates/contracts/tests/j56_eval_corpus_contract.rs (~100 LOC)
- CREATE: workflow/reasoning/J-56-decisions.md (~30 LOC)

## Citations / Grounding
- Source: workflow/reasoning/steal-production-ai-talks-2026-06-20.md (Five-Pillars Pillar-1 + lessons 1/3; the Anthropic breadcrumb-corpus mechanism). [DESIGN, vendor-framework directional — the "200 cases" numbers are illustrative, not a measured ELAI target; the done-gate is ELAI's own corpus growth, not the talk's figure.]
- ELAI ground truth (PLAN-asserted, executor re-greps per W-7): E-54 ingest_task + 13-gram filter; G-6 SAFEDELTA harness; D-6 held-out; failure_taxonomy.py categories.
- CLAUDE.md: TD-3 (no LLM-judge on a gate), W-2, blake3-only, named-constants, no-.unwrap-in-prod.
- LIVE SCHEMA baseline ~268/577 (Tier-B derived, commit 5b0a66d, 2026-06-15); EvalCorpusEvent = baseline+1; re-read at edit (R-c/W-9).

## Open questions / risks
- Where ingest_task lives (knowledge vs orchestrator crate) — PLAN-asserted; executor re-greps at EXECUTING (W-7).
- The subset selector's diff->category mapping: prefer the J-16 blast-radius cone if shipped, else a static path->category map (decide at edit; additive — start with the static map, swap to the cone when J-16 lands).
- Honest residue (GD-5): the corpus is only as good as the failures fed in; J-56 ships the LOOP and the discipline, not a pre-populated suite — the value compounds with use, it does not arrive full.

## Next
J-56 makes "the eval set is a living system" real: caught failures become permanent categorized regression rows, CI stays affordable via subset-on-change, and the SAFEDELTA number finally has a growing dataset to ablate over. It is the eval gate that J-57 (eval-gated prompt promotion) runs against. Pairs with G-6 (the ablation), D-6 (held-out), J-16 (the diff cone).

End of J-56.
