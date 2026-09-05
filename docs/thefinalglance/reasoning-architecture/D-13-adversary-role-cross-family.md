# Adversary role + cross-family verifier (D-13)

- **ELAI ids:** D-13 / RPS.IV / D0-017 / D0-048 / D003-D13-E001..E032
- **Spec status (as designed):** DARK(`ELAI_ADVERSARY_ENABLE`) (`ELAI-FEATURE-MAP.md:592-595`). D-SUMMARY labels the plan PLANNED (`D-plan/D-SUMMARY_INDEX.md:98`). FEATURE-MAP also notes `WIRED_STATIC` per 06-07b and `adversary_runtime=None` in boot. Live hook: `task_runner.rs:3766-3782`.
- **Crate / module (spec):** `crates/orchestrator/src/adversary.rs`; `AgentRole::Adversary` in `agent_role.rs`; Mode B branch in `grounding_mode.rs`; D-3 `verification_result.rs`; `.claude/agents/adversary.md`
- **Purpose of this file:** enough architecture to rebuild Mode B with hard cross-family constraints. Not a status ledger.

## 1. What it is

**Mode B** is last-resort grounding: spawn `AgentRole::Adversary` on a **different model family** from the executor, collect **structured** `CounterEvidence`, and let the **auditor** (never the executor) decide (`D-plan/D-13_adversary_role_cross_family.md:1-11,20-31`). If no alternate family is available, Mode B is **skipped** and the system falls to Mode E. Same-model adversarial probing is forbidden.

## 2. Why it exists

Lightweight Latent Verifiers: verifier transfer fails across base LLMs — heterogeneity is the defense (`arxiv:2504.16760`). Self-Correction Bench: **64.5%** blind-spot rate on same-model critique (`arxiv:2507.02778`). Constitutional AI in small LMs: same-family recursive critique can collapse the model (`arxiv:2504.04918`) (`D-plan/D-13_adversary_role_cross_family.md:4-11,23-31`). If this organ is missing, Mode B is self-talk. If it is faked (same Qwen “adversary”, or freeform narrative), it is the failure mode the papers measured.

## 3. How it works (runtime)

1. **Who calls it.** D-10 `dispatch_by_mode` `FalsifiabilityCrossFamily` branch (`D-plan/D-13_adversary_role_cross_family.md:125-135`, `D-plan/D-10_mode_routing_predicate.md:107-115`). Live later fill: `dispatch_adversarial_pass` when `ELAI_ADVERSARY_ENABLE=1` (`task_runner.rs:3766-3773`).
2. **Input.** `claim: &Claim`, `ctx: &OrchContext`, `AdversaryConfig`.
3. **`probe` (ordered, fail-closed):**
   1. `ctx.has_alternate_family_model` false → `CrossFamilyUnavailable` → `ClaimResult::Refused { reason: "no cross-family adversary" }` + `CrossFamilyUnavailable` SSB.
   2. `cross_family_check(executor, adversary)` false → `SameFamily` error (caller must fall to Mode E). **Qwen25Coder vs Qwen25Math is same family** (`:83-85,156-159`).
   3. Load `.claude/agents/adversary.md`.
   4. Invoke `RuntimeAdapter`; parse JSON to `CounterEvidence`; **schema-validate**. Malformed / freeform → `AdversaryError` / `ParseError`, **not** text fallback (`:120-123,210-213`).
   5. Emit `AdversaryVerdict`.
4. **Map verdict → ClaimResult:**
   - `ClaimRobust` → `Verified`
   - `ClaimFalse` → `Falsified`
   - `ClaimWeak` → `Inconclusive`
   (`:132-135`)
5. **D-3 consensus.** `SignalKind::Adversary`; `VerificationResult.adversary: Option<CounterEvidence>`. Robust = Pass, False = Reject, Weak = neither / Inconclusive contribution (`:137-142`). **Auditor inspects (counter-evidence, original claim). NEVER the executor** (`:28-29`).
6. **The model (adversary) may generate** structured JSON matching the schema. It may **not** pick Mode B, run as the executor family, or hedge instead of `ClaimRobust`.
7. **Default deployment:** local Qwen2.5-Coder-7B executor; cloud Sonnet as Adversary. D-2 PPM sidecar is **same-family Qwen — not an Adversary**. Cloud down and no second local family → Mode E (`:33-37`).

## 4. Types, enums, objects

Copy from `D-plan/D-13_adversary_role_cross_family.md:69-97,111-115`. Do not invent fields.

```
AgentRole::Adversary
  metadata: required_runtime_family is_cross_family_from_executor  // spawn-time

struct AdversaryConfig {
  runtime_role: AgentRole,
  allowed_families: Vec<ModelFamily>,
  max_response_tokens: u32,
  structured_output_schema: CounterEvidenceSchema,
}

enum ModelFamily {  // closed + extensible Other
  Qwen25Coder, Qwen25Math, AnthropicClaude, OpenAiGpt, DeepSeek, Other(String)
}

fn cross_family_check(executor: ModelFamily, adversary: ModelFamily) -> bool
  // different variant; same-family pairs false → Mode E

struct CounterEvidence {
  verdict: CounterVerdict,
  evidence_points: Vec<EvidencePoint>,
  source_tier_citations: Vec<SourceTierCitation>,
  confidence: f32,
}
enum CounterVerdict { ClaimRobust, ClaimWeak, ClaimFalse }

struct EvidencePoint {
  kind: EvidenceKind,
  content: BoundedString<2048>,
  citation_ref: Option<SourceTierCitation>,
}
enum EvidenceKind { ContradictoryExample, MissingPrecondition, LogicalGap, AlternateInterpretation }

enum SourceTierCitation {
  T1Arxiv { id: String },
  T1Nature { doi: String },
  T2OrgBlog { url: String },
  T3Other { url: String, note: String },
}

struct AdversaryProbe {
  runtime_adapter: Arc<dyn RuntimeAdapter>,
  config: AdversaryConfig,
  ssb_tx: SsbSender,
}
async fn probe(&self, claim, ctx) -> Result<CounterEvidence, AdversaryError>
```

`AdversaryError` is named `thiserror`; variants at least `CrossFamilyUnavailable` and `SameFamily` (`:114-123,192`). Full variant list otherwise **UNCERTAIN**.

## 5. Control flow / state machine

**Owner:** orchestrator Mode B dispatch + spawn-time family check. Auditor owns the accept/reject decision over (claim, counter-evidence).

```
D-10 selects B
  → spawn AdversaryProbe (PerRoleRuntime B-37)
  → cross-family preflight
       fail → Refused + CrossFamilyUnavailable → Mode E
  → structured probe
       ClaimRobust → Verified (D-3 Pass signal)
       ClaimFalse  → Falsified (D-3 Reject)
       ClaimWeak   → Inconclusive
  → auditor inspects; executor does not self-grade
```

**Forced:** Mode B is last-resort (A and D first — that is D-10). No alternate family ⇒ skip B. Same-family pair ⇒ skip B.

**v1:** single Adversary, fresh probe per claim. Ensemble and caching deferred (`:300-305`).

## 6. Grounding, receipts, gates

Receipt is schema-valid `CounterEvidence` JSON plus `AdversaryVerdict` SSB. Narrative critique is banned (CTIM-Rover REALM 2025: narrative degrades small-model attention) (`:26-27,178-180`).

**Never allowed as a confidence signal:** executor self-critique, PPM-as-adversary, unparsed prose, confidence float **without** structured evidence points. `confidence: f32` is a field on CounterEvidence; it is **not** a substitute for the verdict enum.

Source-tier citations are the evidence classes (T1 arxiv/nature, T2 org blog, T3 other).

## 7. Budgets, limits, ceilings

| Knob | Value | Scope |
|---|---|---|
| `max_response_tokens` | 4096 default | per probe (`:151-154`) |
| evidence content | `BoundedString<2048>` | per EvidencePoint |
| observable evidence_count | ≥2 on the Rust-GC false claim | smoke |
| SCHEMA_VERSION (plan) | 58 → 59 | SSB |
| unit tests | ≥10 | acceptance |
| allowed_families default | anthropic_claude, openai_gpt, deepseek | `[adversary]` |

Cost: cloud Sonnet is session-throttled (decision a) (`:174-177`). Numeric throttle: **UNCERTAIN**.

## 8. Roles and neighbors

**Adversary** (new role) vs **executor** vs **auditor**. Extends C-12 per-level auditor REVIEW with structured cross-family probing (`D-plan/D-SUMMARY_INDEX.md:303`, `ELAI-FEATURE-MAP.md:592-595`). Consumes D-10 Mode B, D-3 `VerificationResult`, B-37 `PerRoleRuntime`. D-14 reuses the cross-family runtime for inverted-framing.

**Must not fork:** same-family “red team”; executor deciding the verdict; multi-adversary majority (deferred).

## 9. Config / flags / defaults

- **`ELAI_ADVERSARY_ENABLE`** — opt-in (`runtime_config` + `task_runner.rs:3766-3771`, `wire-dark-allowlist.toml:851-856`). Copy this flag.
- **`elai_cc.toml [adversary]` L3-locked:**

```toml
[adversary]
allowed_families = ["anthropic_claude", "openai_gpt", "deepseek"]
max_response_tokens = 4096
```

(`D-plan/D-13_adversary_role_cross_family.md:151-154`). `allowed_families` **must load from TOML**, not hardcoded (`:212`).

- Agent file `.claude/agents/adversary.md`: “Strongest possible critic… If you cannot find counter-evidence, mark ClaimRobust… Do NOT hedge.” Cite latent-verifiers + Self-Correction Bench in the system message. Default runtime: Sonnet (`:99-109`).

## 10. Implementation map

| Symbol | Spec path |
|---|---|
| `adversary.rs` | config, probe, types |
| `agent_role.rs` | `Adversary` variant |
| `grounding_mode.rs` | Mode B branch |
| `verification_result.rs` | `SignalKind::Adversary` |
| `elai_cc.toml` | `[adversary]` |
| `.claude/agents/adversary.md` | prompt |
| `ssb.rs` 58→59 | AdversaryVerdict, CrossFamilyUnavailable |
| `orchestrator/tests/adversary_integration.rs` | A/B/C scenarios |
| live `agent_pool::run_adversarial_pass` | `task_runner.rs:3770` |
| live `AdversaryRuntimeState` | `task_runner.rs:3782` |

## 11. Journal / SSB events

Plan SCHEMA_VERSION **58 → 59**:

```
AdversaryVerdict { claim_id: Uuid, verdict: String, confidence: f64,
                   evidence_count: u32, runtime_family: String, audit_id: Uuid }
CrossFamilyUnavailable { claim_id: Uuid, executor_family: String, reason: String }
```

(`:144-149`). D-SUMMARY names `ADVERSARY_VERDICT` + `CROSS_FAMILY_UNAVAILABLE` (`D-plan/D-SUMMARY_INDEX.md:100`). Live later fill also mentions `adversary_generated` + `consensus_reevaluated_with_counter` (`task_runner.rs:3771-3772`) — mapping **UNCERTAIN**.

## 12. Tests and observables

| Case | Assert |
|---|---|
| Qwen25Coder × AnthropicClaude | `cross_family_check` true |
| Qwen25Coder × Qwen25Math | false |
| CounterEvidence JSON | serde round-trip |
| T1Arxiv id `2504.16760` | ok; malformed → ParseError |
| Scenario A tokio-is-async-runtime | ClaimRobust → Verified |
| Scenario B “Rust requires GC” | ClaimFalse → Falsified |
| Scenario C no cloud | CrossFamilyUnavailable + Refused |
| observable | ModeSelected B + AdversaryVerdict ClaimFalse evidence_count≥2 + Falsified |
| cloud A | confident-wrong → ClaimFalse → consensus rejects |
| cloud B | robust claim → ClaimRobust → consensus accepts with other signals |
| cloud C | cloud down → Mode E |

Papers: `self-correction-bench-2507.02778`, `constitution-collapse-2504.04918`, `constitutional-small-2503.17365`. ≥10 tests.

## 13. Rebuild contract

Another harness has **this** organ iff:

- `AgentRole::Adversary` with spawn-time cross-family enforcement.
- `cross_family_check` rejects same-family pairs including Qwen Coder vs Qwen Math.
- Structured `CounterEvidence` JSON only; freeform → error.
- No alternate family → `CrossFamilyUnavailable` + Mode E, never same-model probe.
- Auditor (D-3 signal), not executor, consumes the verdict.
- `[adversary]` loaded from config; `ELAI_ADVERSARY_ENABLE` default-OFF isolation allowed, but enabling without a real cross-family runtime must refuse, not silently self-critique.
- Mapping Robust/False/Weak → Verified/Falsified/Inconclusive.

A same-model “be critical” prompt is a cousin.

## 14. Non-goals / banned designs

- D-14 sycophancy (orthogonal; reuses infra).
- Multi-Adversary ensemble (deferred).
- Critique caching (deferred; fresh per claim) (`:300-305`).
- PPM / Qwen-Math as Adversary.
- Narrative critique; text-mode fallback on parse failure.
- Executor self-grading the counter-evidence.

## 15. Source citations

Primary: `D-plan/D-13_adversary_role_cross_family.md` (full file). Index: `D-plan/D-SUMMARY_INDEX.md:98-102`. Feature map: `ELAI-FEATURE-MAP.md:592-595`. Systems: `ELAI-SYSTEMS-OVERVIEW.md:236-237`. Exhaustive: D0-017, D0-048, D003-D13-E001..E032. Live: `task_runner.rs:3766-3782`, `wire-dark-allowlist.toml:851-856`.

## 16. Gaps

Label **UNCERTAIN**; do not fill:

- Full `AdversaryError` / `CounterEvidenceSchema` definitions.
- How `ModelFamily` is detected from a live runtime (string compare? model id table?).
- Numeric session cost throttle for Sonnet.
- Live `adversary_generated` / `consensus_reevaluated_with_counter` vs plan SSB names.
- `min_family_distance` named in `task_runner.rs:3774` — not in the D-13 plan.
- `confidence: f32` vs SSB `confidence: f64`.
- Whether `Other(String)` can pair-match as same family.
- Timeout / retry / cancellation of the adapter call.
