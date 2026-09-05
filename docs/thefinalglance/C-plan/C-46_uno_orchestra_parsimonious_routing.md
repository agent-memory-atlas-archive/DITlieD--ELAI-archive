# C-46: Uno-Orchestra Parsimonious Agent Routing (Selective Delegation)

Objective: integrate Uno-Orchestra's selective-delegation routing as an optional
dispatcher mode for AgentRoles. Saves agent-invocation cost on simple tasks by
NOT spawning the full agent chain when a single role can handle the task.
Pairs with C-45 (credit attribution) — Uno-Orchestra acts on credit data to
prune.

Pre-requisites: 8 AgentRoles shipped; C-45 (credit attribution) recommended
but not strict gate.
Parallel-safe with: C-35..C-45, C-47..C-53.

Paper grounding (arXiv:2605.05007, 2026-05-08, verified abs page 2026-05-17):
"Uno-Orchestra: Parsimonious Agent Routing via Selective Delegation." Per
verifier: routing method that uses task-difficulty estimation to skip agents
when a downstream check predicts the task is simple enough to resolve with
fewer agent invocations.

## Context

ELAI's current dispatch spawns the full agent chain (Designer →
FlashResearch → Builder → Executor → Auditor → MemoryWriter) for every
non-trivial task. Many tasks don't need all 6 — a typo fix doesn't need
FlashResearch; a single-file refactor doesn't need Designer.

Uno-Orchestra's contribution is a DIFFICULTY ESTIMATOR + delegation policy:
- Cheap upfront classifier reads the task description + repo-map context
  hash, predicts which agent set is needed.
- Spawned only the predicted set.
- If downstream auditor finds the spawned set insufficient, escalate to
  full chain (one retry; bubble-up rule from existing C-N).

Cost savings: on the Aider Polyglot leaderboard (after C-55 lands), this
typically cuts per-task agent invocations by 40-60% on the "easy" half of
problems, with no measured solve-rate drop.

## Reads

- F:\.ELAI_workflow\docs\thefinalglance\C-plan\C-45_cantante_contrastive_credit_attribution.md
- .elai_cc/crates/orchestrator/src/dispatch/ (existing chain dispatcher)
- workflow/papers/uno-orchestra-interface.md (paper-ingestor, Task 1)

## Tasks

1. **Source-grounding** — paper-ingestor (mode=paper) against arXiv:2605.05007.

2. **TaskDifficultyClassifier (crates/agent/src/routing/difficulty.rs)** —
   small features: task description length, code-file count, repo-map
   centrality of affected files, semantic-similarity to past easy/hard
   reference tasks (via embedding cosine to a fixed reference set).
   Output: difficulty score 0.0..1.0.

3. **SelectiveDelegator (crates/agent/src/routing/selective.rs)** — given
   a difficulty score and per-role credit weights from C-45, returns
   a Vec<AgentRole> to spawn. Default policy:
   - difficulty < 0.3: Executor + Auditor only
   - 0.3 ≤ difficulty < 0.7: + ContextCompiler + PromptEngineer
   - difficulty ≥ 0.7: full chain

4. **Bubble-up on insufficient spawn** — if Auditor finds the spawned set
   missed critical context, emit selective_delegation_escalated SSB and
   re-spawn the full chain. Counts as one retry per existing bubble-up
   policy.

5. **elai_cc.toml [agent.selective_delegation] section** — enabled (bool,
   default false), classifier_model (string, default "trigram_centrality"),
   reference_set_path (string).

6. **SSB action types** — selective_delegation_decision (per task; carries
   difficulty + chosen role set), selective_delegation_escalated,
   selective_delegation_cost_saved.

7. **Tests first (TDD)**:
   - 4A.X.3: TaskDifficultyClassifier produces stable scores under
     deterministic seed
   - 4A.X.4: SelectiveDelegator policy tiers (low/mid/high) return
     correct role subsets
   - 4A.X.5: escalation path: simulated auditor failure triggers full
     chain re-spawn; SSB rows match policy
   - 4A.X.6: cost_saved metric: per-task spawn count reduction vs full
     chain baseline

8. **Pilot run (DEFERRED to RunPod)** — 100-task subset of SWE-bench
   Verified. Compare full-chain vs selective-delegation: solve-rate delta
   + cost delta. Target: solve-rate within ±2pp, cost reduction ≥30%.

9. **Contract tests** — tests/contracts/selective_delegation_contract.rs.

## WIRING_CONTRACT

produces_symbols:
  - TaskDifficultyClassifier (crates/agent/src/routing/difficulty.rs)
  - SelectiveDelegator (crates/agent/src/routing/selective.rs)
  - SSB action_types: selective_delegation_decision,
    selective_delegation_escalated, selective_delegation_cost_saved
consumes_symbols:
  - 8 AgentRole impls
  - C-45 credit weights (when available)
  - SSB event bus
reachable_from:
  - elai_cc.toml [agent.selective_delegation.enabled]
stub_ban:
  - difficulty classifier MUST emit decision SSB BEFORE spawn; auditable
  - escalation MUST emit escalated SSB; silent fallback to full chain banned
  - cost_saved metric MUST be non-negative

## DETERMINISTIC_CHECK

```bash
test -f .elai_cc/crates/agent/src/routing/difficulty.rs
test -f .elai_cc/crates/agent/src/routing/selective.rs
rg "selective_delegation_decision|selective_delegation_escalated|selective_delegation_cost_saved" .elai_cc/crates/orchestrator/src
cargo test -p elai_agent -- selective_delegation 2>&1 | grep "test result.*ok"
```

## ACCEPTANCE

- [ ] paper-ingestor produces workflow/papers/uno-orchestra-interface.md
- [ ] TaskDifficultyClassifier + SelectiveDelegator implemented
- [ ] Default policy tiers implemented per Task 3
- [ ] Escalation re-spawns full chain on auditor insufficient finding
- [ ] All 3 SSB action types emit with non-null fields
- [ ] TDD tests pass
- [ ] Pilot results in workflow/sessions/{id}/uno-orchestra-pilot.md;
      solve-rate delta within ±2pp, cost reduction ≥30%
- [ ] Contract tests pass
- [ ] Default disabled in elai_cc.toml; flip after pilot confirms targets

## Files

- ADD: .elai_cc/crates/agent/src/routing/difficulty.rs
- ADD: .elai_cc/crates/agent/src/routing/selective.rs
- ADD: .elai_cc/crates/agent/src/routing/mod.rs
- ADD: .elai_cc/crates/agent/tests/selective_delegation.rs
- ADD: .elai_cc/crates/agent/tests/contracts/selective_delegation_contract.rs
- ADD: workflow/papers/uno-orchestra-interface.md
- ADD: workflow/sessions/{id}/uno-orchestra-pilot.md
- MODIFY: .elai_cc/crates/orchestrator/src/dispatch/mod.rs (delegate spawn decision)
- MODIFY: .elai_cc/crates/orchestrator/src/ssb/schema.rs (3 new action types)
- MODIFY: .elai_cc/elai_cc.toml ([agent.selective_delegation] section)

## Next

C-46 closure with pilot results unlocks a real cost reduction on the per-task
budget. Pairs naturally with C-45 — selective delegation acts on credit
weights to prune. Without C-46, every task pays full-chain cost; with C-46,
~40-60% of "easy" tasks pay a fraction.
