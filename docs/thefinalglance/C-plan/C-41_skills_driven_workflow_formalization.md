# C-41: Skills-Driven Agentic Workflow Layer Formalization

Objective: formalize the existing implicit "skill"/"protocol" pattern in
.claude/skills/ and equivalent agent-side prompt scaffolding into a first-class
SkillRegistry consumed by the prompt-engineer agent. Direct application of
arXiv:2604.21910 "From Research Question to Scientific Workflow" findings:
explicit skill scaffolding raises measured intent accuracy from 44% to 83%.

Pre-requisites: Phase 4A Designer+FlashResearch SHIPPED; prompt-engineer agent
SHIPPED; C-39 (MCP bridge CLI surface) recommended for unified dispatch.
Parallel-safe with: all C-35..C-40, C-42..C-53. Sequence ahead of C-44
(Composer 2 backend) which benefits from explicit skill scaffolding when
routing role-specific calls.

Paper grounding (arXiv:2604.21910, 2026-04-25, verified abs page 2026-05-17):
"From Research Question to Scientific Workflow: Leveraging Agentic AI for
Science Automation." Measured claim: explicit skill-scaffolded workflows raise
agent intent-classification accuracy from 44% (baseline LLM) to 83%
(skill-augmented). Per verifier 2026-05-17: ELAI already has the
`.claude/skills/`-style layer at the apps_workflow side AND analogous prompt
scaffolding in agent definitions, so 1/3 of the architecture is free; the
missing piece is the explicit SkillRegistry + skill-routing dispatcher at the
prompt-engineer agent.

## Context

ELAI has skill-like artifacts in two places:
1. apps_workflow .claude/skills/ (idea-intake, planning-workflow, oracle, etc)
2. ELAI's own .claude/agents/ (designer, researcher, auditor, prompt-engineer)

Both define stable role-scoped behaviors with INPUT_FORMAT, OUTPUT_FORMAT,
TRIGGERS. But there's no FIRST-CLASS registry on the ELAI runtime side that
the prompt-engineer can query to ROUTE incoming task descriptions to the right
skill scaffold. Currently the prompt-engineer agent inlines the routing logic
in its own definition.

The paper's measurement: when an agentic workflow has explicit, registered,
addressable skill scaffolds vs implicit inline routing, intent-classification
accuracy doubles (44% → 83%) on the SciQuest benchmark. ELAI's analog is task
classification: SIMPLE / FOCUSED / HEAVY / IDEA + sub-domain.

C-41 makes the registry explicit so:
- New skills register by dropping a file in skills/ + an entry in
  skills/registry.toml (no code change required to add a skill)
- The prompt-engineer agent calls a registry.lookup(task_description) instead
  of inlining classification
- SSB emits skill_selected with the chosen skill + confidence
- Skill scaffolds are versioned so an upgrade can roll out gradually

## Reads

- F:\.ELAI_workflow\docs\specs\guide\steps\phase-4A.txt (Designer + FlashResearch)
- .elai_cc/crates/agent/src/prompt_engineer.rs
- .elai_cc/skills/ (current directory — may not yet exist; create if absent)
- workflow/papers/scientific-workflow-interface.md (paper-ingestor output, Task 1)

## Tasks

1. **Source-grounding** — paper-ingestor (mode=paper) against arXiv:2604.21910.
   Produces workflow/papers/scientific-workflow-interface.md capturing the
   skill-routing protocol and the SciQuest benchmark structure.

2. **skills/ directory + registry.toml** — establish the canonical location.
   Each skill: skills/{name}/SKILL.md (human-readable scaffold) +
   skills/{name}/meta.toml (machine-readable metadata: name, version,
   triggers, inputs, outputs).

3. **SkillRegistry (crates/agent/src/skills/registry.rs)** — load all
   skills at startup, build a routing index (trigram + sparse keyword
   match against trigger phrases). `lookup(task)` returns
   `Vec<(SkillCandidate, score)>` sorted by score descending.

4. **prompt-engineer integration** — refactor inline routing in the
   prompt-engineer agent definition to call SkillRegistry.lookup. Top-3
   candidates flow into the engineered prompt as scaffold context. If
   top-1 score < confidence threshold, fall through to default scaffold.

5. **Initial skill set migration** — port the existing implicit skills
   into the registry. Minimum: simple_query, focused_edit, heavy_plan,
   idea_intake, oracle, planning_workflow, paper_ingest, source_ingest,
   benchmark_run. Each gets a SKILL.md + meta.toml.

6. **SSB action types** — skill_lookup, skill_selected, skill_fell_through.
   Each carries top_candidates, chosen_skill, confidence, task_id.

7. **Tests first (TDD)**:
   - 4A.X.3: SkillRegistry loads N skills, indexes correctly
   - 4A.X.4: lookup() returns deterministic ordering on identical input
   - 4A.X.5: confidence threshold gates fall-through; below-threshold
     hits emit skill_fell_through SSB
   - 4A.X.6: prompt-engineer with registry produces structurally different
     scaffold from prompt-engineer without — A/B asserts non-identity
   - 4A.X.7: synthetic SciQuest-style fixture (100 task descriptions,
     ground-truth skill labels) — registry accuracy ≥ paper's reported
     gap minus 10pp (≥73% if paper baseline at 83%, conservative target
     since this isn't the paper's exact model)

8. **Versioning** — each skill carries a semver; SkillRegistry tracks
   compatibility. Major-version bump on breaking input/output change
   forces fallthrough until the consumer (prompt-engineer) updates.

9. **Dashboard panel (dashboard-v2/SkillsPanel.svelte)** — list registered
   skills, show recent skill_selected events, allow manual override for
   debugging.

10. **Contract tests** — tests/contracts/skill_registry_contract.rs verifies
    lookup signature, score monotonicity, fall-through behavior.

## WIRING_CONTRACT

produces_symbols:
  - SkillRegistry, SkillCandidate, SkillMeta (crates/agent/src/skills/)
  - skills/ directory + per-skill SKILL.md + meta.toml
  - skills/registry.toml index
  - SSB action_types: skill_lookup, skill_selected, skill_fell_through
  - SkillsPanel.svelte (dashboard-v2)
consumes_symbols:
  - prompt_engineer agent (crates/agent/src/prompt_engineer.rs)
  - SSB event bus
  - filesystem (skills/ tree)
reachable_from:
  - any /task or /explore invocation that triggers the prompt-engineer
  - dashboard SkillsPanel manual override
stub_ban:
  - SkillRegistry MUST emit skill_lookup SSB even on cache hit
  - lookup() MUST NOT return empty Vec on registry-empty state without
    emitting an explicit error row
  - fall-through path MUST be EXPLICIT (skill_fell_through SSB), not silent
frontend_pair: SkillsPanel.svelte
observable_effect:
  trigger: /task "implement X" (any non-trivial task)
  expect_ssb: skill_lookup + skill_selected with chosen_skill + confidence
  expect_artifact: prompt-engineer output includes skill scaffold (visible
                   in workflow/sessions/{id}/prompt-engineered-{n}.md)

## DETERMINISTIC_CHECK

```bash
# 1. skills/ directory + registry index
test -d .elai_cc/skills
test -f .elai_cc/skills/registry.toml

# 2. Minimum 9 skills migrated
grep -c "^\[skills\." .elai_cc/skills/registry.toml | awk '$1 >= 9 {exit 0} {exit 1}'

# 3. SkillRegistry symbols
rg "SkillRegistry|SkillCandidate|SkillMeta" .elai_cc/crates/agent/src/skills

# 4. SSB action types
rg "skill_lookup|skill_selected|skill_fell_through" .elai_cc/crates/orchestrator/src

# 5. TDD tests pass
cargo test -p elai_agent -- skills 2>&1 | grep "test result.*ok"

# 6. SciQuest synthetic accuracy fixture passes
cargo test -p elai_agent -- skills_synthetic_accuracy 2>&1 | grep "test result.*ok"

# 7. Contract test passes
cargo test -p elai_agent -- skill_registry_contract 2>&1 | grep "test result.*ok"
```

## ACCEPTANCE

- [ ] paper-ingestor produces workflow/papers/scientific-workflow-interface.md
- [ ] skills/ directory + registry.toml present
- [ ] Minimum 9 skills migrated (simple_query, focused_edit, heavy_plan,
      idea_intake, oracle, planning_workflow, paper_ingest, source_ingest,
      benchmark_run) with both SKILL.md and meta.toml
- [ ] SkillRegistry implements lookup with deterministic ordering
- [ ] prompt-engineer refactored to call lookup; inline routing removed
- [ ] Skill versioning honored (major bump forces fallthrough)
- [ ] Synthetic SciQuest-style accuracy ≥73% (paper's 83% minus 10pp
      conservative)
- [ ] All 3 SSB action types emitted with non-null fields
- [ ] TDD tests 4A.X.3..4A.X.7 written and passing BEFORE wiring
- [ ] Contract tests pass
- [ ] SkillsPanel.svelte renders registered skills + recent selections

## Files

- ADD: .elai_cc/skills/registry.toml
- ADD: .elai_cc/skills/simple_query/SKILL.md + meta.toml
- ADD: .elai_cc/skills/focused_edit/SKILL.md + meta.toml
- ADD: .elai_cc/skills/heavy_plan/SKILL.md + meta.toml
- ADD: .elai_cc/skills/idea_intake/SKILL.md + meta.toml
- ADD: .elai_cc/skills/oracle/SKILL.md + meta.toml
- ADD: .elai_cc/skills/planning_workflow/SKILL.md + meta.toml
- ADD: .elai_cc/skills/paper_ingest/SKILL.md + meta.toml
- ADD: .elai_cc/skills/source_ingest/SKILL.md + meta.toml
- ADD: .elai_cc/skills/benchmark_run/SKILL.md + meta.toml
- ADD: .elai_cc/crates/agent/src/skills/mod.rs
- ADD: .elai_cc/crates/agent/src/skills/registry.rs
- ADD: .elai_cc/crates/agent/src/skills/lookup.rs
- ADD: .elai_cc/crates/agent/tests/skills_tdd.rs
- ADD: .elai_cc/crates/agent/tests/contracts/skill_registry_contract.rs
- ADD: .elai_cc/crates/dashboard-v2/src/lib/components/SkillsPanel.svelte
- ADD: workflow/papers/scientific-workflow-interface.md
- MODIFY: .elai_cc/crates/agent/src/prompt_engineer.rs (call SkillRegistry)
- MODIFY: .elai_cc/crates/orchestrator/src/ssb/schema.rs (3 new action types)
- MODIFY: .elai_cc/Cargo.toml (no new workspace member; skills/ is data + integrated into agent crate)

## Affected Verification Reports

- F:\.ELAI_workflow\docs\thefinalglance\verification\reports\steps\phase-4A.report.md
  — annotate C-41 SkillRegistry arrival.

## Next

After C-41 lands: new skills land via filesystem drop, no code change.
Foundation for C-44 (Composer 2 backend tier) which uses skill_selected to
route role-specific calls to the right runtime model. Also unblocks
external skill contributions from non-Rust developers (drop a SKILL.md +
meta.toml, no PR-against-Rust-codebase needed).
