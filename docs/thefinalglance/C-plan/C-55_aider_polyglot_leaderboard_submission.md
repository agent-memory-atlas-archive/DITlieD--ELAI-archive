# C-55: Aider Polyglot Leaderboard Submission (RUNPOD-DEPENDENT)

Objective: run ELAI through the Aider Polyglot 225-problem benchmark in 2
configs (local 32B, hybrid Sonnet+Composer2+local) and submit the result to
the public leaderboard. Direct comparison surface with the existing
"ollama qwen2.5-coder:32b 72.9%" entry — ELAI's value-add over raw model
must show vs that baseline.

Pre-requisites: C-51 (Aider Polyglot harness adoption) SHIPPED — harness +
sandboxes + adapter all landed; this patch is the RUN. RunPod online.
Parallel-safe with: C-54, C-56, C-57, C-58 (sequential on same RunPod).

Reference grounding (verified via prior oracle 2026-05-17):
aider.chat/docs/leaderboards/ — current top: gpt-5 (high) 88.0% for $29.08;
gpt-5 (low) 81.3% for $10.37. Local entry: ollama qwen2.5-coder:32b at
72.9% on edit leaderboard (https://aider.chat/docs/leaderboards/edit.html).

## Context

The Aider Polyglot leaderboard publishes COST per benchmark run. This is
the single best public comparison surface for ELAI's "infrastructure
substitutes for parameters" thesis. ELAI's hypothesis: per-dollar
performance with the SAME local 32B model + ELAI's pipeline beats raw
ollama+qwen2.5-coder:32b's 72.9% on the same problems.

If it does: ELAI's infrastructure produces measurable per-dollar wins.
If it doesn't: the infrastructure is paying for itself in solve-rate only
under cloud-tier escalation, and the local-only thesis needs revision.

Two configs:
- **Config L (local 32B)**: backends = [local_qwen_32b]; direct comparable
  to the ollama entry.
- **Config H (hybrid)**: backends = [sonnet_46, composer2, local_qwen_32b];
  comparable to gpt-5 (high) at the cost-per-task level.

## Reads

- F:\.ELAI_workflow\docs\thefinalglance\C-plan\C-51_aider_polyglot_harness_adoption.md
- aider.chat/docs/benchmarks (current submission protocol — fresh fetch)

## Tasks

1. **Submission-protocol fetch** — fresh WebFetch aider.chat/docs/benchmarks
   for current submission format (JSON shape, required fields, public
   leaderboard PR template).

2. **Two config TOMLs** under .elai_cc/benchmarks/aider_polyglot/configs/:
   - config-L-local-32B.toml
   - config-H-hybrid.toml

3. **Per-task run-loop** — 225 problems × 2 configs = 450 task runs total.
   Per task collect: solved (bool), wall_time_s, llm_calls, tokens_in,
   tokens_out, cost_usd, repaired_after_fail (bool, signals C-31/C-32
   StructuralRetry kicked in).

4. **Cost methodology** — document the per-config cost calculation:
   - Config L: GPU-hour rate × wall-time fraction; document the rate
     (RunPod A100/H100/4090 hourly).
   - Config H: GPU-hour rate (local share) + cloud-API metered cost
     (Sonnet 4.6 + Composer 2).
   Both methodologies match Aider's published cost methodology to enable
   apples-to-apples comparison.

5. **Scorecard JSON** — match Aider leaderboard format exactly (per C-51
   sandbox shim). Save at workflow/sessions/{id}/aider_polyglot_run_{config}/scorecard.json.

6. **Public PR** — fork github.com/Aider-AI/aider, add ELAI entries to
   leaderboard data, open PR. PR body cites the methodology doc + the
   ELAI tag (commit SHA, elai_cc.toml versions). LeaderboardPRComposer
   wraps this with C-40 GitHub PR adapter.

7. **Methodology + results docs** — same structure as C-54:
   workflow/sessions/{id}/aider_polyglot_methodology.md + .elai_cc/docs/aider_polyglot_results.md.

8. **SSB action types** — polyglot_submission_started, polyglot_submission_completed,
   polyglot_pr_opened (consumes C-40 github_pr_created path).

## WIRING_CONTRACT

produces_symbols:
  - 2 config TOMLs (config-L, config-H)
  - 2 scorecard JSONs
  - methodology doc + results doc
  - public PR against Aider-AI/aider with leaderboard entry
  - SSB action_types: polyglot_submission_started, polyglot_submission_completed,
    polyglot_pr_opened
consumes_symbols:
  - C-51 PolyglotAdapter + SandboxedTestRunner per language
  - C-40 GitHub PR adapter (for public submission PR)
  - C-44 Composer 2 backend tier (config H requires)
  - SSB event bus
reachable_from:
  - elai benchmark aider-polyglot --config <name> --runpod-id <id>
  - elai benchmark aider-polyglot --submit (uses C-40 to open PR after
    both configs complete)
stub_ban:
  - cost calculation MUST be machine-derived from token + time inputs,
    NOT human-estimated
  - public PR MUST NOT auto-merge; opens as DRAFT for human review
  - results.md MUST cite both methodology + public-PR URL

## DETERMINISTIC_CHECK

```bash
test -f .elai_cc/benchmarks/aider_polyglot/configs/config-L-local-32B.toml
test -f .elai_cc/benchmarks/aider_polyglot/configs/config-H-hybrid.toml
test -f workflow/sessions/$(cat workflow/sessions/current.txt)/aider_polyglot_run_L/scorecard.json
test -f workflow/sessions/$(cat workflow/sessions/current.txt)/aider_polyglot_run_H/scorecard.json
test -f .elai_cc/docs/aider_polyglot_results.md
rg "polyglot_submission_started|polyglot_submission_completed|polyglot_pr_opened" .elai_cc/crates/orchestrator/src
```

## ACCEPTANCE

- [ ] Submission-protocol fetched fresh; format documented
- [ ] Both config TOMLs authored
- [ ] **Config L run completed** — 225 problems local 32B; scorecard.json present
- [ ] **Config H run completed** — 225 problems hybrid; scorecard.json present
- [ ] Cost methodology auto-derived
- [ ] Methodology doc + results doc generated
- [ ] Public DRAFT PR opened against Aider-AI/aider with leaderboard data
- [ ] All 3 SSB action types emit
- [ ] Comparison statement: ELAI local 32B vs ollama qwen2.5-coder:32b
      72.9% baseline documented honestly (better, worse, or equal)
- [ ] Comparison statement: ELAI hybrid vs gpt-5 (low/high) at matched cost
      documented honestly

## Files

- ADD: .elai_cc/benchmarks/aider_polyglot/configs/config-L-local-32B.toml
- ADD: .elai_cc/benchmarks/aider_polyglot/configs/config-H-hybrid.toml
- ADD: workflow/sessions/{id}/aider_polyglot_run_L/scorecard.json (post-run)
- ADD: workflow/sessions/{id}/aider_polyglot_run_H/scorecard.json (post-run)
- ADD: workflow/sessions/{id}/aider_polyglot_methodology.md (post-run)
- ADD: .elai_cc/docs/aider_polyglot_results.md (post-run)
- MODIFY: .elai_cc/crates/cli/src/main.rs (benchmark aider-polyglot subcommand
  + --submit flag)
- MODIFY: .elai_cc/crates/orchestrator/src/ssb/schema.rs (3 new action types)

## Next

C-55 closure produces ELAI's first PUBLIC entry on an external leaderboard.
Pairs with C-54: SWE-bench is the industry-standard issue-resolution
benchmark, Aider Polyglot is the cost-per-task benchmark. Together they
position ELAI on both axes.
