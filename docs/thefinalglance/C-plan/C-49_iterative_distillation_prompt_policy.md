# C-49: Iterative Distillation of Experience for Prompt-Engineer

Objective: ship a prompt-policy distillation loop where successful task-execution
traces become structured prompt-templates that the prompt-engineer agent
consults for future similar tasks. Direct application of arXiv:2605.14443's
iterative-distillation pattern.

Pre-requisites: prompt-engineer agent SHIPPED; C-41 SkillRegistry recommended
(provides the skill-scoped binning for distillation).
Parallel-safe with: C-35..C-48, C-50..C-53.

Paper grounding (arXiv:2605.14443, 2026-05-17, verified abs page 2026-05-17,
most recent paper in the forum-dump batch): "Prompting Policies for Multi-step
Reasoning and Tool-Use in Black-box LLMs with Iterative Distillation of
Experience." Per verifier: trace-distillation method that takes
successful-execution traces and produces compressed prompt-policy artifacts
that bias future prompt construction toward the proven path.

## Context

ELAI's prompt-engineer currently constructs each prompt from scratch using
SkillRegistry scaffold (post C-41) + context-package.md. There is no
LEARNING from prior successful executions within the same project. Two
sessions on the same codebase repeat the same prompt-engineering cost.

Iterative Distillation produces, per skill + per project, a small
"experience corpus" file (.elai/experience/{skill_name}.md) containing:
- Distilled prompt fragments that consistently appeared in successful runs
- Common parameter shapes for tool calls
- Avoid-patterns from failure cases (mistakes.md material, already-tracked
  but currently siloed)

The prompt-engineer consults the experience corpus AFTER SkillRegistry
lookup. Successful runs append (with deduplication); failures append to
the avoid-pattern section.

## Reads

- F:\.ELAI_workflow\docs\thefinalglance\C-plan\C-41_skills_driven_workflow_formalization.md
- .elai_cc/crates/agent/src/prompt_engineer.rs
- workflow/sessions/{*}/mistakes.md (existing failure corpus to seed avoid-patterns)
- workflow/papers/iterative-distillation.md (paper-ingestor, Task 1)

## Tasks

1. **Source-grounding** — paper-ingestor (mode=paper) against arXiv:2605.14443.

2. **ExperienceCorpus store (crates/agent/src/experience.rs)** — per-skill
   per-project markdown file at .elai/experience/{skill_name}.md. Sections:
   ## Distilled prompt fragments, ## Common parameter shapes, ## Avoid patterns.

3. **Distiller (crates/agent/src/distiller.rs)** — runs after a successful
   task completes. Inputs: skill_name, full execution trace, SSB log.
   Output: appended fragments to ExperienceCorpus. Deduplication via
   shingling (5-gram exact match collapses).

4. **Avoid-pattern integration** — at distillation time, also consults
   mistakes.md and emits matching avoid-patterns. Two-way: avoid-patterns
   surface in prompt-engineer's scaffold; new mistakes update avoid-patterns.

5. **Prompt-engineer integration** — after SkillRegistry lookup, fetch
   ExperienceCorpus for the chosen skill. Inject distilled fragments into
   scaffold (within a budget — cap at ~500 tokens of experience context;
   prefer most-recent fragments).

6. **SSB action types** — experience_distilled, experience_consulted,
   experience_token_budget_exceeded.

7. **Tests first (TDD)**:
   - 4A.X.3: ExperienceCorpus round-trips markdown
   - 4A.X.4: Distiller deduplicates via shingling
   - 4A.X.5: avoid-pattern from mistakes.md surfaces in scaffold
   - 4A.X.6: token budget cap honored; oldest evicted when over budget
   - 4A.X.7: empty corpus (greenfield project) does not crash prompt-engineer

8. **Contract tests** — tests/contracts/experience_corpus_contract.rs.

## WIRING_CONTRACT

produces_symbols:
  - ExperienceCorpus, Distiller (crates/agent/src/experience.rs + distiller.rs)
  - .elai/experience/{skill_name}.md per-project artifacts
  - SSB action_types: experience_distilled, experience_consulted,
    experience_token_budget_exceeded
consumes_symbols:
  - prompt-engineer scaffold path
  - SkillRegistry (C-41)
  - workflow/sessions/{*}/mistakes.md
  - SSB event bus
reachable_from:
  - post-task successful completion (distillation trigger)
  - every prompt-engineer invocation (consultation trigger)
stub_ban:
  - dedup MUST be deterministic (shingling, not random sampling)
  - token budget MUST evict OLDEST when over budget, not newest
  - greenfield (empty corpus) MUST NOT panic; transparent skip

## DETERMINISTIC_CHECK

```bash
test -f .elai_cc/crates/agent/src/experience.rs
test -f .elai_cc/crates/agent/src/distiller.rs
rg "experience_distilled|experience_consulted|experience_token_budget_exceeded" .elai_cc/crates/orchestrator/src
cargo test -p elai_agent -- experience 2>&1 | grep "test result.*ok"
```

## ACCEPTANCE

- [ ] paper-ingestor produces workflow/papers/iterative-distillation.md
- [ ] ExperienceCorpus markdown schema implemented
- [ ] Distiller appends fragments with shingling dedup
- [ ] Avoid-patterns from mistakes.md integrate bidirectionally
- [ ] Prompt-engineer consults corpus after SkillRegistry
- [ ] Token budget cap (~500 tokens) honored
- [ ] All 3 SSB action types emit
- [ ] TDD tests pass; greenfield case does not panic
- [ ] Contract tests pass
- [ ] After 10 sessions on same codebase, experience corpus has non-empty
      Distilled fragments section (smoke test, not a formal acceptance gate)

## Files

- ADD: .elai_cc/crates/agent/src/experience.rs
- ADD: .elai_cc/crates/agent/src/distiller.rs
- ADD: .elai_cc/crates/agent/tests/experience_tdd.rs
- ADD: .elai_cc/crates/agent/tests/contracts/experience_corpus_contract.rs
- ADD: workflow/papers/iterative-distillation.md
- MODIFY: .elai_cc/crates/agent/src/prompt_engineer.rs (consult corpus)
- MODIFY: .elai_cc/crates/orchestrator/src/post_task_hook.rs (trigger distiller)
- MODIFY: .elai_cc/crates/orchestrator/src/ssb/schema.rs (3 new action types)

## Next

C-49 closure means per-project prompt-engineering cost amortizes over time.
First session on a codebase pays full prompt-construction cost; subsequent
sessions pay reduced cost as the experience corpus accumulates.
