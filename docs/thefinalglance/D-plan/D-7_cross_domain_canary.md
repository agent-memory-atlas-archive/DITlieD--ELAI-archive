# D-7: Cross-domain transfer canary — 5 orthogonal task families

Objective: every L2 mutation must be evaluated against ALL 5 task families
(code-fix, math-proof, prose-reason, planning, knowledge-QA) even if proposed
for a single family; promotion BLOCKED if any non-target family regresses by
>5%. NanoFlux pattern (arxiv:2509.23252, T1) — improvement on math must not
regress medical/scientific reasoning by >k% (companion §3.5, lines 268-276).
Estimated effort: 1-2 sessions.
Pre-requisites: D-6 (held-out store has per-family tagging; canary draws from
  it).
Parallel-safe with: D-5 (BAI consumes canary verdict as one gate),
  D-8 (archive verify). NOT parallel-safe with D-4 (tripwire reads canary
  signals as Δρ source).

## Context

Companion §3.5 (lines 268-276): NanoFlux (arxiv:2509.23252, T1) demonstrated
that improvement on a target task family can cause regression on adjacent
families if the verifier is single-domain. Defense: track per-family pass-rate;
treat improvement on target + non-regression on others as the gate.

The 5 families chosen (companion UC-3 acknowledges this is a guess; instrument
carefully):
- **code-fix**: rust/python compile + test pass; standardized fixtures from
  exercism-public + leetcode-public.
- **math-proof**: SymPy-checkable algebraic identities + step-by-step proofs;
  fixtures from PRMBench math subset (D-19 ingests).
- **prose-reason**: natural-language inference + chain-of-reasoning; fixtures
  from HotpotQA + AdversarialQA public splits.
- **planning**: multi-step decomposition + dependency order; fixtures from
  PlanBench + AlfWorld public.
- **knowledge-QA**: factual recall + grounded citation; fixtures from
  PopQA + TriviaQA + FActScore.

Per-family fixture corpus is bootstrapped from public benchmarks
(HuggingFace datasets), tagged into D-6 held-out store on first ingest.

Partial foundation present and usable:
- D-6 HeldOutStore.query(domain=Some) filters per-family.
- D-1 Domain enum has Code, Math, Prose, Mixed — extend with Planning + KnowledgeQa
  (atomic with D-7).
- D-5 BAI gate consumes single gate signal; D-7 produces per-family
  CrossDomainRegression rows BAI aggregates.

## Reads

- F:\.ELAI_workflow\docs\specs\guide\reasoning-improvement-grounded-2026-05-19.md
  (§3.5 cross-domain canary; UC-3 task-family count — lines 268-276 + 433)
- F:\.ELAI_workflow\workflow\papers\livecodebench-2403.07974-interface.md (D-6)
- .elai_cc/crates/orchestrator/src/held_out.rs (D-6)
- .elai_cc/crates/orchestrator/src/reasoning_template.rs (D-1 Domain enum;
  extension here)

## Paper-ingestor pre-step

Spawn paper-ingestor with mode=paper for:
- arxiv:2509.23252 NanoFlux Cross-Domain

Verify workflow/papers/nanoflux-2509.23252-interface.md.

## Tasks

1. **Domain enum extension** in reasoning_template.rs (D-1):
   - Atomic with D-7 land: extend Domain enum from
     `{ Code, Math, Prose, Mixed }` to
     `{ Code, Math, Prose, Mixed, Planning, KnowledgeQa }`.
   - Update all existing matchers to be exhaustive (compile-time check).
   - Migration: existing ReasoningTemplate entries with Domain::Mixed remain
     valid; no on-disk migration needed.

2. **CanaryFamily types** in new file
   .elai_cc/crates/orchestrator/src/canary.rs:
   - `enum CanaryFamily { CodeFix, MathProof, ProseReason, Planning, KnowledgeQa }`
     (closed enum; L3-locked; mapping to Domain enum 1:1 except `Code →
     CodeFix` and `Math → MathProof`).
   - `struct FamilyEvalResult { family: CanaryFamily, pass_count: u32,
     total_count: u32, pass_rate: f32 }`
   - `struct CanaryReport { template_id: TemplateId, baseline: Vec<FamilyEvalResult>,
     candidate: Vec<FamilyEvalResult>, deltas: Vec<(CanaryFamily, f32)>,
     target_family: CanaryFamily, decision: CanaryDecision }`
   - `enum CanaryDecision { Allow, BlockRegression { family: CanaryFamily, delta: f32 }, InsufficientData { family: CanaryFamily } }`

3. **CanaryRunner** in canary.rs:
   - `struct CanaryRunner { held_out: Arc<HeldOutStore>, evaluator:
     Arc<dyn Evaluator>, regression_threshold: f32, min_samples_per_family: u32 }`
   - `fn run(&self, template_id: TemplateId, baseline_template: TemplateId,
     candidate_template: TemplateId, target_family: CanaryFamily) -> CanaryReport`:
     for each family, query held-out tasks (D-6); run both baseline + candidate;
     compute pass-rates; compute deltas.
   - Decision logic: BlockRegression if ANY family delta < -regression_threshold;
     InsufficientData if any family has total_count < min_samples_per_family.

4. **Family bootstrap fixtures** (manual data-load step):
   - Author/import ≥10 tasks per family into D-6 HeldOutStore.
   - code-fix: 10 from exercism rust + 5 from leetcode-public.
   - math-proof: 10 from PRMBench math subset.
   - prose-reason: 10 from HotpotQA + 5 AdversarialQA.
   - planning: 10 from PlanBench.
   - knowledge-QA: 10 from PopQA + 5 TriviaQA.
   - Each task: ExpectedOutcome populated; domain field set correctly;
     source field cites benchmark + version.

5. **SSB action types** in ssb.rs (SCHEMA_VERSION 52→53):
   - `CrossDomainRegression { template_id: TemplateId, family: String,
     baseline_pass_rate: f64, candidate_pass_rate: f64, delta: f64,
     threshold: f64, audit_id: Uuid }`
   - `CanaryFamilyPass { template_id: TemplateId, family: String, pass_rate:
     f64, sample_count: u32 }`
   - `CanaryInsufficientData { template_id: TemplateId, family: String,
     sample_count: u32, required: u32 }`
   - Round-trip tests.

6. **elai_cc.toml [canary] section**:
   - `[canary]\nregression_threshold = 0.05  # 5%\nmin_samples_per_family = 5\n
     enabled_families = ["code_fix", "math_proof", "prose_reason", "planning",
     "knowledge_qa"]`
   - L3-LOCKED.

7. **Integration with BAI gate** in
   .elai_cc/crates/orchestrator/src/bai.rs:
   - BaiPromotionEngine.promote_round invokes CanaryRunner.run before any
     promotion; if CanaryDecision::BlockRegression, skip promotion + emit
     SSB row.
   - Promotion path: baseline = current Promoted template for target_family;
     candidate = Candidate awaiting promotion.

8. **Unit tests**:
   - FamilyEvalResult.pass_rate = passes / total.
   - CanaryReport.decision logic table (≥10 scenarios across Allow,
     BlockRegression, InsufficientData).
   - CanaryFamily ↔ Domain mapping bijection.
   - regression_threshold 0.05 triggers BlockRegression at delta=-0.06; not at
     delta=-0.04.

9. **Integration test** in
   .elai_cc/crates/orchestrator/tests/canary_integration.rs:
   - Mock HeldOutStore with 50 tasks across 5 families.
   - Run CanaryRunner against synthetic baseline + candidate templates.
   - Scenario A: candidate improves code-fix by +10%, no regression elsewhere
     → Allow.
   - Scenario B: candidate improves code-fix by +10%, regresses prose-reason
     by 8% → BlockRegression.
   - Scenario C: only 3 tasks in planning family → InsufficientData.

10. **Decision doc** workflow/reasoning/D-7-decisions.md (3 decisions):
    - (a) Why 5 families specifically: companion UC-3 — 3 too few, 12 too
      many; 5 is sweet spot per literature absent specific guidance.
    - (b) Why 5% regression threshold: companion §3.5 suggested k=5; balances
      noise floor (≥10 samples gives ≥3% std) against meaningful change.
    - (c) Why min_samples_per_family=5: minimum for statistical sanity; if
      held-out is sparse, BAI auto-defers promotion until samples accumulate.

## WIRING_CONTRACT

produces_symbols:
  - CanaryFamily (closed enum) + CanaryFamily ↔ Domain mapping
  - FamilyEvalResult, CanaryReport, CanaryDecision (closed enum)
  - CanaryRunner
  - SsbActionType::CrossDomainRegression + ::CanaryFamilyPass +
    ::CanaryInsufficientData + payloads
  - SCHEMA_VERSION bump 52→53
  - [canary] elai_cc.toml section
  - Domain enum extension: Planning + KnowledgeQa
consumes_symbols:
  - HeldOutStore.query (D-6)
  - BaiPromotionEngine (D-5; integrates gate)
  - Domain enum (D-1; extended here)
  - Evaluator trait (existing; mock in tests)
reachable_from:
  - BAI promote_round (D-5) calls CanaryRunner.run before each promotion
  - BAI promotion verdict consumes CanaryDecision
stub_ban:
  - CanaryRunner MUST require min_samples_per_family per family; InsufficientData
    branch must emit SSB row
  - Domain enum extension MUST keep existing variants stable (no rename)
  - regression_threshold MUST load from elai_cc.toml; not hardcoded
observable_effect:
  trigger: BAI promotion of synthetic candidate with code-fix +10% / prose
           -8% deltas (test harness simulates)
  expect_ssb: 1 CrossDomainRegression row (family=prose_reason, delta=-0.08);
              4 CanaryFamilyPass rows for non-regressing families; 0
              TemplatePromoted row
  expect_artifact: ArchiveChain reflects candidate as still Candidate (not
                   Promoted); CanaryReport persisted to workflow/sessions/
                   {id}/canary-reports/

## DETERMINISTIC_CHECK

```bash
# 1. SCHEMA_VERSION 53
rg "SCHEMA_VERSION: i64 = 53" .elai_cc/crates/orchestrator/src/ssb.rs

# 2. SSB variants
rg "CrossDomainRegression|CanaryFamilyPass|CanaryInsufficientData" \
  .elai_cc/crates/orchestrator/src/ssb.rs | wc -l
# Expected: >= 12

# 3. Canary types
rg "enum CanaryFamily|struct CanaryRunner|enum CanaryDecision" \
  .elai_cc/crates/orchestrator/src/canary.rs

# 4. Domain enum extension
rg "Domain::Planning\|Domain::KnowledgeQa" \
  .elai_cc/crates/orchestrator/src/reasoning_template.rs

# 5. elai_cc.toml [canary]
rg "\[canary\]" .elai_cc/elai_cc.toml

# 6. Seed fixtures across 5 families
for family in code math prose plan knowledge; do
  rg "domain.*=\s*\"$family" .elai_cc/held-out/2026-05-19/*.toml | wc -l
done
# Expected: each >= 10

# 7. BAI integration
rg "CanaryRunner" .elai_cc/crates/orchestrator/src/bai.rs

# 8. Build + tests
cargo check --workspace 2>&1 | tail -3
cargo test -p elai_orchestrator -- canary 2>&1 | grep "test result"

# 9. Paper-ingestor
test -f workflow/papers/nanoflux-2509.23252-interface.md && echo OK
```

## ACCEPTANCE

- [ ] SCHEMA_VERSION_PUBLIC == 53
- [ ] 3 SSB variants round-trip
- [ ] CanaryFamily ↔ Domain bijection; Planning + KnowledgeQa added to Domain
- [ ] CanaryRunner.run decision logic table passes (≥10 scenarios)
- [ ] 50 fixture tasks across 5 families seeded in D-6 store
- [ ] BAI integration: candidate with cross-family regression → BlockRegression
- [ ] elai_cc.toml [canary] loaded; defaults applied
- [ ] cargo check --workspace: zero new errors
- [ ] cargo test -p elai_orchestrator -- canary: all pass, ≥10 new tests
- [ ] Cloud smoke test A: synthetic candidate +10% code-fix / -8% prose →
      CrossDomainRegression SSB row + BAI skip promotion
- [ ] Cloud smoke test B: synthetic candidate +10% code-fix / +5% math /
      neutral elsewhere → 5 CanaryFamilyPass + TemplatePromoted rows
- [ ] Cloud smoke test C: family with only 3 samples → CanaryInsufficientData
- [ ] Paper-ingestor (nanoflux) present
- [ ] Decision doc workflow/reasoning/D-7-decisions.md

## Files touched

- ADD: .elai_cc/crates/orchestrator/src/canary.rs
- MODIFY: .elai_cc/crates/orchestrator/src/reasoning_template.rs
    (Domain enum: + Planning + KnowledgeQa)
- MODIFY: .elai_cc/crates/orchestrator/src/ssb.rs
    (SCHEMA_VERSION 52→53; 3 new variants + payloads)
- MODIFY: .elai_cc/crates/orchestrator/src/bai.rs (CanaryRunner integration)
- MODIFY: .elai_cc/elai_cc.toml ([canary] section)
- MODIFY: .elai_cc/crates/orchestrator/src/lib.rs (pub mod canary)
- ADD: .elai_cc/held-out/2026-05-19/canary-{family}-*.toml (≥50 seed tasks)
- ADD: .elai_cc/crates/orchestrator/tests/canary_integration.rs
- ADD: workflow/reasoning/D-7-decisions.md
- ADD: workflow/papers/nanoflux-2509.23252-interface.md

## Out-of-scope

- D-24 (cross-domain canary regression validation suite; benchmarks D-7)
- D-15 NoveltyClass (uses DKG, not canary)
- D-18 ObservableEffect (probe substrate, not canary)

## Next

D-8 (archive verify CLI) surfaces all BAI + canary lineage to user via `elai
archive verify`. D-24 benchmarks D-7 with 100-mutation regression validation.
