# C-58: AgentForge 5-Role Head-to-Head + RRK Listwise Reranker Eval (RUNPOD-DEPENDENT)

Objective: two paired evaluations in one RunPod session — (a) ELAI's 8-role
agent suite vs AgentForge's 5-role suite on SWE-bench Lite; (b) ELAI's
current cross-encoder reranker vs RRK 8B listwise reranker on the
knowledge-retrieval pipeline.

Pre-requisites: C-45 (CANTANTE credit attribution) recommended so the
8-role baseline has per-role attribution to compare against; RunPod online.
Parallel-safe with: C-54, C-55, C-56, C-57.

Reference grounding (verified via prior oracle 2026-05-17):
- AgentForge: arXiv:2604.13120 (2026-04-13): "AGENTFORGE achieves 40.0%
  resolution on SWE-BENCH Lite, outperforming single-agent baselines by
  26-28 points." Open-source at github.com/raja21068/AutoCodeAI.
- RRK: arXiv:2604.26483 (2026-04-29): 8B listwise "runs 3x-18x faster than
  smaller rerankers (0.6-4B)." Counterintuitive but measured.

## Context

### AgentForge head-to-head

Both ELAI's 8-role and AgentForge's 5-role can be run against the same
SWE-bench Lite task set (smaller than Verified, faster turnaround). The
test:
- AgentForge published 40.0% on SWE-bench Lite.
- ELAI runs same task set with default 8 roles (Executor, Auditor,
  Researcher, Designer, PromptEngineer, ContextCompiler, MemoryWriter,
  MediaValidator).
- AgentForge runs in parallel on the same hardware with its 5 roles
  (Planner, Coder, Tester, Debugger, Critic).
- Compare solve rate + cost per task.

Pairs with C-45 — if AgentForge wins per-cost, the credit-attribution data
from C-45 shows which ELAI roles failed to earn their cost.

### RRK reranker eval

ELAI's knowledge retrieval pipeline currently has a cross-encoder reranker
≤200M params per the architecture. RRK's 8B listwise reranker claims
counterintuitive speedup vs smaller rerankers; if this transfers, ELAI's
reranker layer (knowledge crate) gets a measurable upgrade.

But: RRK is 8B parameters; ELAI's reranker is ≤200M. Even with the 3-18x
speedup, 8B may not fit ELAI's <50ms latency budget. Pilot measures both
solve-quality contribution AND latency on ELAI's typical query volume.

## Reads

- arXiv:2604.13120 + github.com/raja21068/AutoCodeAI
- arXiv:2604.26483 + reference implementation
- .elai_cc/crates/knowledge/src/reranker.rs (current cross-encoder)

## Tasks

### AgentForge head-to-head

1. **Source-grounding** — paper-ingestor (mode=paper + mode=source) against
   arXiv:2604.13120 + AutoCodeAI repo.

2. **AgentForge adapter** — vendor AutoCodeAI under .elai_cc/external/agentforge/
   with pinned SHA. Build an isolation harness that runs AgentForge against
   the same SWE-bench Lite task corpus.

3. **Head-to-head runner** — `elai benchmark head-to-head agentforge`
   subcommand runs both suites on the same task set sequentially, emits
   side-by-side scorecard.

### RRK reranker eval

4. **Source-grounding** — paper-ingestor against arXiv:2604.26483.

5. **RRK integration probe** — feature-gated `rrk_reranker` Cargo feature.
   Load RRK 8B per the paper's released checkpoint. Wire as alternative
   in knowledge/src/reranker.rs behind config flag.

6. **Latency benchmark** — measure per-query reranker latency on ELAI's
   typical query shape (top-K vectors → reranked). Compare ELAI current
   (≤200M) vs RRK (8B). Reject RRK if latency > 100ms (2x ELAI's 50ms
   target).

7. **Quality benchmark** — on a held-out retrieval set (e.g. the
   Codebase-Memory query corpus from C-56), measure top-K precision /
   recall for both rerankers.

8. **Decision matrix** — IF RRK latency acceptable AND quality improvement
   ≥5pp: ship as opt-in alternative reranker. ELSE: archive as
   measured-not-worth-it.

### Shared

9. **Methodology + results docs** — single combined doc covering both
   pairs of evaluation.

10. **SSB action types** — head_to_head_run_started, head_to_head_arm_completed,
    reranker_eval_run, reranker_eval_decision.

## WIRING_CONTRACT

produces_symbols:
  - .elai_cc/external/agentforge/ (vendored at pinned SHA)
  - AgentForgeAdapter (crates/task_runner/src/agentforge_adapter.rs)
  - rrk_reranker feature + RRK loader (crates/knowledge/src/rerankers/rrk.rs)
  - 2 scorecard JSONs (head-to-head + RRK)
  - methodology + results docs
  - SSB action_types: head_to_head_run_started, head_to_head_arm_completed,
    reranker_eval_run, reranker_eval_decision
consumes_symbols:
  - SWE-bench Lite task corpus
  - C-56 retrieval query corpus (for RRK quality benchmark)
  - C-45 credit-attribution data (for head-to-head context)
  - knowledge crate (reranker swap point)
reachable_from:
  - elai benchmark head-to-head agentforge --runpod-id <id>
  - elai benchmark reranker-eval --runpod-id <id>
stub_ban:
  - AgentForge vendoring MUST include LICENSE
  - latency benchmark MUST be reported at p99, not just mean
  - RRK adoption decision MUST be config-gated; don't auto-flip default
  - head-to-head MUST run on IDENTICAL task corpus + hardware (run them
    back-to-back, not interleaved)

## DETERMINISTIC_CHECK

```bash
test -d .elai_cc/external/agentforge
test -f .elai_cc/external/agentforge/LICENSE
test -f .elai_cc/crates/task_runner/src/agentforge_adapter.rs
test -f .elai_cc/crates/knowledge/src/rerankers/rrk.rs
test -f workflow/sessions/$(cat workflow/sessions/current.txt)/head_to_head/scorecard.json
test -f workflow/sessions/$(cat workflow/sessions/current.txt)/reranker_eval/scorecard.json
rg "head_to_head_run_started|head_to_head_arm_completed|reranker_eval_run|reranker_eval_decision" .elai_cc/crates/orchestrator/src
```

## ACCEPTANCE

(AgentForge head-to-head)
- [ ] AgentForge vendored at pinned SHA with LICENSE
- [ ] Adapter implemented
- [ ] **Head-to-head run completed** on SWE-bench Lite
- [ ] Side-by-side scorecard authored
- [ ] If AgentForge wins: candidate role-consolidation list documented,
      consults C-45 credit data
- [ ] If ELAI wins: confirms 8-role architecture earns its cost on this
      task class

(RRK reranker)
- [ ] RRK loader implemented behind feature flag
- [ ] **Latency benchmark completed** — p99 at ELAI's typical query volume
- [ ] **Quality benchmark completed** — top-K precision/recall delta
- [ ] Decision matrix applied; adoption / archival documented
- [ ] Config-gated if adopted; default unchanged

(Shared)
- [ ] All 4 SSB action types emit
- [ ] Combined methodology + results docs authored

## Files

- ADD: .elai_cc/external/agentforge/ (vendored)
- ADD: .elai_cc/crates/task_runner/src/agentforge_adapter.rs
- ADD: .elai_cc/crates/knowledge/src/rerankers/rrk.rs
- ADD: .elai_cc/crates/knowledge/src/rerankers/mod.rs (Reranker trait)
- ADD: workflow/sessions/{id}/head_to_head/scorecard.json (post-run)
- ADD: workflow/sessions/{id}/reranker_eval/scorecard.json (post-run)
- ADD: workflow/sessions/{id}/head_to_head_reranker_methodology.md (post-run)
- ADD: .elai_cc/docs/head_to_head_reranker_results.md (post-run)
- ADD: workflow/papers/agentforge-interface.md
- ADD: workflow/papers/rrk-reranker-interface.md
- MODIFY: .elai_cc/crates/cli/src/main.rs (2 new benchmark subcommands)
- MODIFY: .elai_cc/crates/orchestrator/src/ssb/schema.rs (4 new action types)
- MODIFY: .elai_cc/crates/knowledge/src/lib.rs (rerankers module)

## Next

C-58 closure produces (a) a measurement of ELAI's role architecture vs a
published 5-role baseline, (b) a measurement of whether listwise reranking
at 8B is worth ELAI's latency budget. Both feed C-59 master report.
