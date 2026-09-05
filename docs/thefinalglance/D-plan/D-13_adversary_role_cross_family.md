# D-13: Adversary role + cross-family verifier wiring (Mode B with hard constraints)

Objective: add AgentRole::Adversary that spawns on a DIFFERENT model family
from executor; structured counter-evidence output; required for any GroundingMode::
FalsifiabilityCrossFamily dispatch. Per Lightweight Latent Verifiers (arxiv:
2504.16760, T1): verifier transfer fails between base LLMs — heterogeneous-
verifier is the literature's defense. Per Self-Correction Bench (arxiv:
2507.02778, T1): 64.5% blind-spot rate confirms same-model self-critique
fails; cross-family critique is auxiliary-useful. Per Constitutional AI in
Small LLMs (arxiv:2504.04918, T1): same-model recursive self-critique can
cause MODEL COLLAPSE on small models — never same-family.
Estimated effort: 2-3 sessions.
Pre-requisites: D-10 (Mode B routing), D-3 (consensus rule consumes
  Adversary verdict). spec-grounder verifies.
Parallel-safe with: D-14 (sycophancy probe; orthogonal mechanism; same cross-
  family infrastructure though), D-15 (NoveltyClass).

## Context

Real-problem §4.3 (lines 240-251). Mode B falsifiability inversion is the
LAST-RESORT grounding mode — most claims should route to A or D first.
When invoked, hard constraints (per literature):
1. Adversary spawned on a DIFFERENT model family from executor. Heterogeneous-
   verifier per arxiv:2504.16760 (T1).
2. Adversary output STRUCTURED (counter-evidence with source-tier citation),
   not narrative. Per CTIM-Rover REALM 2025 (T1; narrative degrades small-
   model attention).
3. Auditor inspects (counter-evidence, original claim) and decides; NEVER
   the executor itself.
4. If no alternate-family model available, Mode B is SKIPPED — system falls
   to Mode E. NO same-model adversarial probing.

For ELAI's current deployment (Qwen2.5-Coder-7B local + cloud Sonnet
fallback B-37): cloud Sonnet is the cross-family Adversary (different family
from Qwen). When cloud is unavailable AND no second local sidecar (D-2 PPM
is same-family Qwen, NOT suitable Adversary)→ Adversary unavailable →
Mode E.

Partial foundation present and usable:
- AgentRole enum (existing) — extend with Adversary variant.
- B-37 PerRoleRuntime — Adversary role gets its own runtime slot
  pointing to cloud Sonnet or distinct local family.
- D-3 VerificationResult consumes Adversary as one of consensus signals.
- D-10 GroundingMode::FalsifiabilityCrossFamily branch dispatches Adversary.

## Reads

- F:\.ELAI_workflow\docs\specs\guide\real-problem-solving-grounded-2026-05-19.md
  (§4.3 Adversary; §1 Mode B limits — lines 52-66 + 240-251)
- F:\.ELAI_workflow\workflow\papers\latent-verifiers-2504.16760-interface.md (D-2)
- F:\.ELAI_workflow\workflow\papers\rstar-math-2501.04519-interface.md (D-2)
- .elai_cc/crates/orchestrator/src/grounding_mode.rs (D-10 dispatch)
- .elai_cc/crates/orchestrator/src/verification_result.rs (D-3 consumer)
- .elai_cc/crates/orchestrator/src/runtime/per_role_runtime.rs (B-37
  PerRoleRuntime)

## Paper-ingestor pre-step

Spawn paper-ingestor with mode=paper for:
- arxiv:2507.02778 Self-Correction Bench (64.5% blind-spot rate)
- arxiv:2504.04918 Constitution or Collapse (small-LM constitutional risk)
- arxiv:2503.17365 Constitutional AI in Small LLMs

Verify workflow/papers/{self-correction-bench-2507.02778,constitution-collapse-
2504.04918,constitutional-small-2503.17365}-interface.md.

## Tasks

1. **AgentRole::Adversary** extension in
   .elai_cc/crates/orchestrator/src/agent_role.rs (existing AgentRole enum):
   - Add `Adversary` variant.
   - Add metadata: required_runtime_family is_cross_family_from_executor;
     enforced at spawn time.

2. **AdversaryConfig** in new file
   .elai_cc/crates/orchestrator/src/adversary.rs:
   - `struct AdversaryConfig { runtime_role: AgentRole, allowed_families:
     Vec<ModelFamily>, max_response_tokens: u32, structured_output_schema:
     CounterEvidenceSchema }`
   - `enum ModelFamily { Qwen25Coder, Qwen25Math, AnthropicClaude, OpenAiGpt,
     DeepSeek, Other(String) }` (closed enum for current ELAI deployment +
     extensible string variant for future)
   - `fn cross_family_check(executor: ModelFamily, adversary: ModelFamily) ->
     bool`: ensures different ModelFamily variant; same-family pairs return
     false → caller must fall to Mode E.

3. **CounterEvidence schema** in adversary.rs:
   - `struct CounterEvidence { verdict: CounterVerdict, evidence_points:
     Vec<EvidencePoint>, source_tier_citations: Vec<SourceTierCitation>,
     confidence: f32 }`
   - `enum CounterVerdict { ClaimRobust, ClaimWeak, ClaimFalse }`
   - `struct EvidencePoint { kind: EvidenceKind, content: BoundedString<2048>,
     citation_ref: Option<SourceTierCitation> }`
   - `enum EvidenceKind { ContradictoryExample, MissingPrecondition,
     LogicalGap, AlternateInterpretation }`
   - `enum SourceTierCitation { T1Arxiv { id: String }, T1Nature { doi: String },
     T2OrgBlog { url: String }, T3Other { url: String, note: String } }`

4. **AdversaryAgent prompt** in
   .claude/agents/adversary.md (new agent definition file):
   - Role: "Strongest possible critic. Produce structured counter-evidence
     that the claim is FALSE. If you cannot find counter-evidence, mark
     ClaimRobust with explicit reasoning. Do NOT hedge."
   - Output schema: CounterEvidence JSON (matches Rust struct).
   - Prompt MUST cite Lightweight Latent Verifiers (arxiv:2504.16760, T1)
     and Self-Correction Bench (arxiv:2507.02778, T1) in system message to
     anchor the cross-family motivation.
   - Per CLAUDE.md HARD RULE: agent runs on sonnet (default cross-family
     for Qwen-local deployment).

5. **AdversaryProbe** in adversary.rs:
   - `struct AdversaryProbe { runtime_adapter: Arc<dyn RuntimeAdapter>,
     config: AdversaryConfig, ssb_tx: SsbSender }`
   - `async fn probe(&self, claim: &Claim, ctx: &OrchContext) ->
     Result<CounterEvidence, AdversaryError>`:
     1. Check cross_family availability via ctx.has_alternate_family_model;
        if false → return CrossFamilyUnavailable error.
     2. Cross-family check on config.runtime_role vs executor family →
        if same → return SameFamily error.
     3. Construct adversary prompt (load .claude/agents/adversary.md).
     4. Send via runtime_adapter; parse JSON response to CounterEvidence;
        validate against schema.
     5. Emit AdversaryVerdict SSB row.

6. **GroundingMode::FalsifiabilityCrossFamily dispatch** in
   .elai_cc/crates/orchestrator/src/grounding_mode.rs (D-10):
   - Inside dispatch_by_mode FalsifiabilityCrossFamily branch:
     - Spawn AdversaryProbe with ctx.adversary_role config.
     - Call probe(claim, ctx).
     - If CrossFamilyUnavailable → return ClaimResult::Refused with reason
       "no cross-family adversary"; emit CrossFamilyUnavailable SSB row.
     - If CounterEvidence::ClaimRobust → ClaimResult::Verified (with
       evidence_snippet from CounterEvidence).
     - If CounterEvidence::ClaimFalse → ClaimResult::Falsified.
     - If CounterEvidence::ClaimWeak → ClaimResult::Inconclusive.

7. **VerificationResult Adversary integration** in
   .elai_cc/crates/orchestrator/src/verification_result.rs (D-3):
   - Extend SignalKind enum: + Adversary.
   - Extend VerificationResult struct: + adversary: Option<CounterEvidence>.
   - Update compute_verdict to consider Adversary as one signal; ClaimRobust
     = Pass; ClaimFalse = Reject; ClaimWeak = neither (Inconclusive contribution).

8. **SSB action types** in ssb.rs (SCHEMA_VERSION 58→59):
   - `AdversaryVerdict { claim_id: Uuid, verdict: String, confidence: f64,
     evidence_count: u32, runtime_family: String, audit_id: Uuid }`
   - `CrossFamilyUnavailable { claim_id: Uuid, executor_family: String,
     reason: String }`
   - Round-trip tests.

9. **elai_cc.toml [adversary] section**:
   - `[adversary]\nallowed_families = ["anthropic_claude", "openai_gpt",
     "deepseek"]\nmax_response_tokens = 4096`
   - L3-LOCKED.

10. **Unit tests** in adversary.rs:
    - cross_family_check between Qwen25Coder + AnthropicClaude → true.
    - cross_family_check between Qwen25Coder + Qwen25Math → false (same
      Qwen family, even though different specialization).
    - CounterEvidence schema round-trip (serde JSON).
    - SourceTierCitation parse: T1Arxiv with id "2504.16760" → ok; malformed
      id → ParseError.

11. **Integration test** in
    .elai_cc/crates/orchestrator/tests/adversary_integration.rs:
    - Mock cloud Sonnet adapter; mock executor as Qwen.
    - Scenario A: claim "tokio is a Rust async runtime" → Adversary returns
      ClaimRobust → ClaimResult::Verified.
    - Scenario B: synthetic confident-wrong claim "Rust requires garbage
      collection" → Adversary returns ClaimFalse → ClaimResult::Falsified.
    - Scenario C: no cloud config → CrossFamilyUnavailable SSB row +
      ClaimResult::Refused.

12. **Decision doc** workflow/reasoning/D-13-decisions.md (3 decisions):
    - (a) Why cloud Sonnet as default Adversary (not local 2nd-family
      sidecar): no production 2nd-family sidecar yet; cloud is operational
      cross-family; cost-throttled per session.
    - (b) Why structured CounterEvidence (not freeform critique): CTIM-Rover
      REALM 2025 (T1) — narrative degrades small-model attention; structured
      output more actionable for auditor inspection.
    - (c) Why fall to Mode E (Refuse) when no Adversary: Self-Correction Bench
      explicitly disqualifies same-model self-critique; refusing is a stronger
      epistemic position than hallucinated agreement.

## WIRING_CONTRACT

produces_symbols:
  - AgentRole::Adversary variant
  - AdversaryConfig + CounterEvidence + EvidencePoint + EvidenceKind +
    SourceTierCitation + ModelFamily (closed enums except Other)
  - AdversaryProbe + cross_family_check + probe
  - AdversaryError thiserror
  - SsbActionType::AdversaryVerdict + ::CrossFamilyUnavailable + payloads
  - SignalKind::Adversary variant (D-3 extension)
  - .claude/agents/adversary.md agent definition
  - [adversary] elai_cc.toml section
  - SCHEMA_VERSION bump 58→59
consumes_symbols:
  - GroundingMode::FalsifiabilityCrossFamily dispatch (D-10)
  - VerificationResult (D-3; gains Adversary signal)
  - PerRoleRuntime (B-37; Adversary role-runtime config)
  - SsbWriter
reachable_from:
  - D-10 grounding_mode dispatch path
  - D-3 verification_result consensus signal
  - .claude/agents/adversary.md (spawned per Mode B claim)
stub_ban:
  - AdversaryProbe MUST cross_family_check + emit CrossFamilyUnavailable
    on same-family pair (defends Self-Correction Bench risk)
  - probe MUST parse JSON to CounterEvidence; malformed → AdversaryError, NOT
    fallback to text mode
  - allowed_families MUST load from elai_cc.toml; not hardcoded
  - structured_output_schema MUST validate; freeform response → ParseError
observable_effect:
  trigger: synthetic claim "Rust requires garbage collection" via cloud /task
           with cloud Sonnet as Adversary
  expect_ssb: 1 ModeSelected row (mode=FalsifiabilityCrossFamily); 1
              AdversaryVerdict row (verdict=ClaimFalse, evidence_count>=2);
              claim ClaimResult::Falsified
  expect_artifact: .claude/agents/adversary.md present; elai_cc.toml
                   [adversary] section present

## DETERMINISTIC_CHECK

```bash
# 1. SCHEMA_VERSION 59
rg "SCHEMA_VERSION: i64 = 59" .elai_cc/crates/orchestrator/src/ssb.rs

# 2. SSB variants
rg "AdversaryVerdict|CrossFamilyUnavailable" .elai_cc/crates/orchestrator/src/ssb.rs

# 3. Adversary types
rg "struct AdversaryProbe|struct CounterEvidence|enum ModelFamily|enum CounterVerdict" \
  .elai_cc/crates/orchestrator/src/adversary.rs

# 4. AgentRole::Adversary
rg "Adversary" .elai_cc/crates/orchestrator/src/agent_role.rs

# 5. Agent file
test -f .claude/agents/adversary.md && echo OK

# 6. SignalKind::Adversary
rg "SignalKind::Adversary" .elai_cc/crates/orchestrator/src/verification_result.rs

# 7. elai_cc.toml [adversary]
rg "\[adversary\]" .elai_cc/elai_cc.toml

# 8. Build + tests
cargo check --workspace 2>&1 | tail -3
cargo test -p elai_orchestrator -- adversary 2>&1 | grep "test result"

# 9. Paper-ingestor outputs
test -f workflow/papers/self-correction-bench-2507.02778-interface.md && \
  test -f workflow/papers/constitution-collapse-2504.04918-interface.md && \
  test -f workflow/papers/constitutional-small-2503.17365-interface.md && echo OK
```

## ACCEPTANCE

- [ ] SCHEMA_VERSION_PUBLIC == 59
- [ ] 2 SSB variants round-trip
- [ ] AdversaryProbe compiles; unit tests pass
- [ ] cross_family_check correctly rejects same-family pairs (Qwen25Coder vs
      Qwen25Math returns false)
- [ ] CounterEvidence JSON schema round-trip via serde
- [ ] .claude/agents/adversary.md present + valid markdown
- [ ] AgentRole::Adversary variant + cross-family enforcement
- [ ] SignalKind::Adversary in VerificationResult; compute_verdict integration
- [ ] elai_cc.toml [adversary] loaded
- [ ] cargo check --workspace: zero new errors
- [ ] cargo test -p elai_orchestrator -- adversary: all pass, ≥10 tests
- [ ] Cloud smoke test A: synthetic confident-wrong claim via cloud /task with
      Mode B dispatch; Adversary returns ClaimFalse; AdversaryVerdict SSB row;
      consensus rejects
- [ ] Cloud smoke test B: synthetic robust claim; Adversary returns
      ClaimRobust; consensus accepts (alongside other passing signals)
- [ ] Cloud smoke test C: cloud unavailable + no alt-family local → Mode B
      → CrossFamilyUnavailable SSB row → falls to Mode E (Refuse)
- [ ] All 3 paper-ingestor outputs present
- [ ] Decision doc workflow/reasoning/D-13-decisions.md

## Files touched

- ADD: .elai_cc/crates/orchestrator/src/adversary.rs
- ADD: .claude/agents/adversary.md
- MODIFY: .elai_cc/crates/orchestrator/src/agent_role.rs (Adversary variant)
- MODIFY: .elai_cc/crates/orchestrator/src/ssb.rs (SCHEMA_VERSION 58→59;
    2 new variants)
- MODIFY: .elai_cc/crates/orchestrator/src/grounding_mode.rs (D-10
    FalsifiabilityCrossFamily branch wiring)
- MODIFY: .elai_cc/crates/orchestrator/src/verification_result.rs (D-3
    SignalKind::Adversary)
- MODIFY: .elai_cc/elai_cc.toml ([adversary] section)
- MODIFY: .elai_cc/crates/orchestrator/src/lib.rs (pub mod adversary)
- ADD: .elai_cc/crates/orchestrator/tests/adversary_integration.rs
- ADD: workflow/reasoning/D-13-decisions.md
- ADD: workflow/papers/{self-correction-bench-2507.02778,constitution-collapse-
       2504.04918,constitutional-small-2503.17365}-interface.md

## Out-of-scope

- D-14 sycophancy probe (orthogonal AUDITING mechanism, reuses cross-family
  infrastructure)
- Multi-Adversary ensemble (DEFERRED; single Adversary first)
- Adversary critique caching (DEFERRED; per-claim fresh probe)

## Next

D-14 (sycophancy probe) reuses cross-family infrastructure for inverted-framing
re-run. D-23 (sycophancy benchmark) gates D-14.
