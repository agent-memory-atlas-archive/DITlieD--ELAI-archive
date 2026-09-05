# Idea-Framer GRC + Domain Research Presets + Critic Loop (B-40)

- **ELAI ids:** B-40 / E-B40-001..E-B40-039 / SUM-0323..SUM-0334
- **Spec status (as designed):** DARK(`ELAI_IDEA_FRAMING_ENABLE`) in feature map (`ELAI-FEATURE-MAP.md:771-773`); organ fully specified in B-40
- **Crate / module (spec):** `crates/orchestrator/src/idea_framing.rs`; `research_tree.rs` `ResearchTree::new_multi_root`; `fsm.rs` `FsmSubstate::Framing`; `reasoning/src/config.rs` `idea_framer` + `idea_critic`; dashboard `IdeaFramingWindow.svelte`
- **Purpose of this file:** enough architecture to rebuild the FRAMING organ — classify, multi-root FlashResearch presets, idea-framer GRC **5/800**, 10-category critic max 3 rounds, tradeoff card — so decomposition does not start on a blank page. Not a status ledger.

## 1. What it is

**Idea-framing** inserts a `FRAMING` substate **before** Phase 4D Recursive Decomposition (Idea → Concepts). It classifies a product idea (`ProjectClass`), dispatches a domain-specific research preset as a **multi-root** FlashResearch tree, runs a 10-category critic loop (1–3 rounds), and **BLOCKs** decomposition until the idea is framed, grounded, and the user has answered the tradeoff card (`B-40_idea_framer_research_presets.md:1-9`, `:18-23`, `ELAI-SYSTEMS-OVERVIEW.md:192-194`). Three additions: TOML `[research_presets.<class>]`, `idea_framer` (Designer variant, GRC **5/800**) producing `FramedIdea` + `TradeoffCard`, and `idea_critic` with max 3 rounds (`:63-73`, E-B40-005). FlashResearch tree + dual-threshold convergence already exist (`research_tree.rs`, arXiv:2510.05145); B-40 dispatches a **named multi-stream root** instead of a single-question root (`:30-33`).

## 2. Why it exists

Today Designer fires one generic FlashResearch tree from the raw prompt and immediately decomposes. A “consumer mobile app” never mandatorily pulls competitor UX teardown, negative reviews, store guidelines, IP/licensing, monetization, design-trends (`B-40_idea_framer_research_presets.md:41-48`). Failure: plausible-wrong Concepts → 5×5×3×3 = **225** baby steps on the wrong problem (`:51-54`, E-B40-004). B-41 catches that **late**; framing catches it **early** (`:55`). If FRAMING is missing, 4D Level 0 still runs — ungrounded. If it is faked (LLM “I framed it” without presets/critic/user card), Concepts inherit fiction.

## 3. How it works (runtime)

1. **Who.** Non-trivial `/task` at CLI `run_turn` **or** dashboard `IdeaFramingWindow` submit (`:198-199`). Auto: PLANNING → FRAMING (`:147-149`).
2. **Classify (deterministic first).** `classify_project_class` regex (A-47 pattern); LLM **only** if `Other`. Target <10ms for the 90% common case (`:93-97`, `:323-324`, E-B40-006, E-B40-036). Enum: `ConsumerMobileApp | WebApp | SaaSDashboard | DesktopApp | GameMod | ResearchTool | LibraryOrSDK | CLITool | Other` (`:91-95`).
3. **Load preset.** `[research_presets.<class>]` ordered `ResearchStream`s (`:99-107`):
   - `name`, Tera `query_template` with `{idea}` / `{class}`
   - `min_source_tier` T1 / T2 (`source_quality.rs`)
   - `parallelism_budget` (max sibling sub-questions)
   - `cosmos_grounding` (B-42 artifact name; **no-op** if B-42 absent)
   - `mandatory: bool` — non-mandatory skipped if budget tight; **mandatory must not be skipped** (emit `degraded_streams` instead) (`:107`, `:205`, E-B40-038)
4. **Multi-root research.** `ResearchTree::new_multi_root(streams)` replaces `new(query)` on the FRAMING path. Each stream has its own `ResearchCache` slot. Convergence: **all mandatory** streams converge **OR** explicit `degraded_streams` in output (`:113-116`, E-B40-009). Neighbor 4D loop FORMULATE→SEARCH→EXTRACT→REFLECT→CONVERGE still applies per stream (`phase-4D.txt:239-247`).
5. **Idea-framer agent.** Role `idea_framer`: GRC **5/800**, required_steps `[Observe, Hypothesize, Verify, Converge]` (extends Designer config, **not** Designer’s 8/1200) (`:120-122`, `:204`, E-B40-010). Produces `FramedIdea`. Budget must **not** exceed 5/800 (`stub_ban :204`).
6. **Critic loop.** `run_critic_loop(framed: &mut FramedIdea, max_rounds=3)` (`:136`):
   - Spawn `idea_critic` (same 5/800 + step list) (`:120-122`)
   - Collect findings; group by `CriticCategory`
   - Zero BLOCKs this round → **converge, exit**
   - Else if round < max: feed `rescope_from_critic` to idea_framer; rewrite
   - If round == max AND BLOCKs remain: emit `idea_framing_block`; FRAMING **refuses to advance** (`:140-145`, E-B40-014)
7. **Tradeoff card.** Ordered `Tradeoff { name, options[], default, user_facing_question }` surfaced dashboard + CommandWindow **before** decomposition (`:127-129`). User radio answers; Submit advances FRAMING → PLANNING (`:163-165`).
8. **FSM / tools.** Substate of PLANNING (not a 10th top-level state) (`:147`, `:320-322`). PreToolUse in FRAMING: **only research/read**; no Edit/Write/Bash beyond research scratch (`:150-151`, E-B40-015). Return path: FRAMING → PLANNING → FEASIBILITY_PROBE → … (`:147-149`).
9. **Auto-advance 7th check.** `derive_framing_evidence(...)` (B-22 `derive_autonomous_evidence` shape): `idea_framed` + `tradeoff_card_emitted` + **zero** unresolved `idea_framing_block`. Gated on `state == FRAMING`. Convergence evidence also stated as `rounds_used ≥ 1 AND zero unresolved C-*` (`:24-26`, `:153-159`, E-B40-003, E-B40-016).
10. **Artifacts.** `workflow/sessions/{sid}/idea-framing/{framed-idea.toml, critic-rounds/round-N.toml, tradeoff-responses.toml}`. Designer Concepts consumes via B-35 `[knowledge.artifacts]` auto-injection (`:171-175`).
11. **Commit / fail-closed.** Advance only after framed + card answered + no unresolved BLOCK. **Abstain:** `idea_framing_block` at round 3 is an acceptable 5% fixture failure mode (`:266-268`). Classifier `Other` is not a pass into “unclassified product”.

The model generates framed prose, critic findings, LLM fallback class. It may **not** skip mandatory streams, raise GRC above 5/800, or auto-advance past unresolved C-* BLOCKs.

## 4. Types, enums, objects

Copy from `B-40_idea_framer_research_presets.md:91-134`, `:177-189`.

```
enum ProjectClass {
  ConsumerMobileApp, WebApp, SaaSDashboard, DesktopApp,
  GameMod, ResearchTool, LibraryOrSDK, CLITool, Other
}
fn classify_project_class(...) -> ProjectClass

ResearchStream {
  name,
  query_template,          // Tera {idea} {class}
  min_source_tier,         // T1 | T2
  parallelism_budget,
  cosmos_grounding,
  mandatory: bool,
}

FramedIdea {
  project_class,
  research_summary,
  key_decisions,
  known_unknowns,
  tradeoff_card,
}

TradeoffCard { tradeoffs: ordered Vec<Tradeoff> }
Tradeoff { name, options[], default, user_facing_question }

CriticFinding {
  category: CriticCategory,
  severity: BLOCK | WARN | INFO,
  evidence_ref,            // research stream pointer
  proposed_fix,
}

enum CriticCategory {
  STALE, SLOP, POLICY, FEASIBILITY, MARKET,
  SCOPE, CONTRADICTION, DESIGN_HARD, VAGUE, MISSING
}  // exactly 10

run_critic_loop(framed: &mut FramedIdea, max_rounds=3)
ResearchTree::new_multi_root(streams)
FsmSubstate::Framing

# GRC (both roles)
idea_framer / idea_critic:
  GrcBudget 5/800
  required_steps [Observe, Hypothesize, Verify, Converge]
```

Observable artifact predicates: `project_class != Other`, `tradeoff_card.tradeoffs.len() ≥ 2`, `research_summary.streams.len() == preset.mandatory_streams.len()` (`:217-220`, E-B40-024). PictureThis example: ConsumerMobileApp → **6** mandatory streams (`:210-213`).

## 5. Control flow / state machine

**Owner:** orchestrator FSM + `run_critic_loop`. LLM does not choose the next stream (4D principle: TreeNavigator / code drives flow) (`phase-4D.txt:340-344`).

```
/task non-trivial
  PLANNING → FRAMING
    classify (regex → maybe LLM Other)
    new_multi_root(mandatory+optional)
    idea_framer Observe→Hypothesize→Verify→Converge  (≤5 calls / 800 chars)
    critic round 1..3
      zero BLOCK → idea_framed + tradeoff_card_emitted
      BLOCK && round<3 → rescope_from_critic
      BLOCK && round==3 → idea_framing_block  (stay in FRAMING)
    user answers card
  FRAMING → PLANNING → FEASIBILITY_PROBE → 4D Concepts
```

**Forced:** PreToolUse research/read only while FRAMING. Mandatory streams → converge or `degraded_streams`, never silent omit.

**stub_ban conflict:** “`run_critic_loop` must NOT short-circuit (max_rounds=3 enforced even when 0 findings)” (`:203`) vs per-round “zero BLOCKs → converge, exit” (`:140`) and “1–3 rounds” / `rounds_used ≥ 1` (`:5-6`, `:24-26`). Exhaustive SUP-049. Rebuild the **body loop** (early exit on zero BLOCKs, max 3); put the no-short-circuit ban in §16.

## 6. Grounding, receipts, gates

Research streams enforce `min_source_tier` via 4D `SourceQuality` domain lookup (no LLM scoring) (`:104`, `phase-4D.txt:80-83`). Unmet tier → `degraded_streams` (`:280-291` research_preset_dispatch test).

Critic `evidence_ref` points at a research stream — findings are not uncited vibes (`:132`).

**Never allowed as a confidence signal:** “preset ran” without mandatory converge/degraded list; critic WARN treated as BLOCK; skipping the user card because AutoAdvance is on; idea_framer self-eval of framing quality; using Designer 8/1200 as if it were the framer.

7th AutoAdvance check is **FRAMING-gated**; existing 6 B-22 checks stay (`:157-159`, E-B40-031).

## 7. Budgets, limits, ceilings

| Knob | Value | Scope |
|---|---|---|
| idea_framer / idea_critic GRC | **5 / 800** | per topic; must not exceed |
| critic max rounds | **3** | per framing |
| classifier latency | < 10 ms regex path | 90% case |
| seed presets | ≥ 4 classes | ConsumerMobileApp, WebApp, SaaSDashboard, GameMod |
| mandatory streams / class | ≥ 4 | acceptance |
| classifier fixture | 50 prompts, ≥ 90% accuracy | `project_class_50.toml` |
| critic converge | ≤ 3 rounds on 95% fixtures; 5% `idea_framing_block` | acceptance |
| ablation | ≥ 30% drop in Concept-tier BLOCK vs no-framing | 50 ideas |
| tests | ≥ 28 (15 unit + 8 convergence + 5 classifier) | |
| PictureThis streams | 6 mandatory | observable example |
| SCHEMA designed | 28 → 29 (5 action types) | later 41 — §16 |

Designer neighbor remains **8/1200** (`phase-4D.txt:28-30`). Do not reuse that budget for the framer.

## 8. Roles and neighbors

**New roles:** `idea_framer`, `idea_critic` in `agent_pool` + `agent_grc_config_for_role` (`:122`, `:310-311`).

**Consumes:** `ResearchTree`, `AgentPool::spawn`, `BasicGrcEngine`, `SourceQualityTable`, B-22 evaluator, B-35 artifacts, 4D FlashResearch (`:191-195`).

**Produces:** types in §4; `FsmSubstate::Framing`; 5 SSB types; window + Monitor “Critic rounds: N/3” (`:168-169`).

**Must not fork:** a second framing LLM outside GRC 5/800; skipping B-40 and hoping B-41 Concepts audit is enough (`:55-56`); top-level FSM state explosion (`:320-322`).

**Downstream:** B-41 mode=concepts uses `FramedIdea` as ground truth; missing frame → WARN + coarse rules (`B-41_per_level_auditor_modes.md:12-16`, `:286-287`).

## 9. Config / flags / defaults

- Feature map: **`ELAI_IDEA_FRAMING_ENABLE`** DARK (`ELAI-FEATURE-MAP.md:771-773`). Original B-40 does not name this env; rebuild the organ as specified and treat the flag as a later default-OFF wrap (§16).
- `[research_presets.*]` in `elai_cc.toml`; ≥4 classes (`:312`).
- Seed rules domains `design_hard`, `app_store` mentioned as rules.db seeds (`:86`) — shared with B-41.
- `get_idea_framing_state` must **not** return mock-data; live SSB only (`:202`, E-B40-022).
- Decision doc `workflow/reasoning/B-40-decisions.md` (`:317-333`).

## 10. Implementation map

| Symbol | Spec |
|---|---|
| `orchestrator/src/idea_framing.rs` (~700 LOC) | `:298` |
| `ResearchTree::new_multi_root` | `:177-185`, `:306` |
| `FsmSubstate::Framing` | `:186`, `:307` |
| `derive_framing_evidence` in `task_runner.rs` | `:153-159`, `:309` |
| `agent_pool.rs` two roles | `:310` |
| `reasoning/src/config.rs` two arms | `:311` |
| `IdeaFramingWindow.svelte` | `:303`, `:161-164` |
| `get_idea_framing_state` Tauri | `:313-314` |
| MonitorWindow critic-rounds badge | `:315` |
| fixture `tests/fixtures/project_class_50.toml` | `:304` |

## 11. Journal / SSB events

Designed five types (`:188-189`, E-B40-020). Sequence (`:214-216`):

```
idea_framing_started
  → project_class_classified
  → (research_tree progress)
  → critic_round_complete  (×N, 1 ≤ N ≤ 3)
  → idea_framed
  → tradeoff_card_emitted
```

Plus fail path `idea_framing_block` (`:143-144`) — **sixth** string, not in the “5 SSB action_types” list; still required for the gate (`:157`). SCHEMA designed 28→29; shipped ledger 41 with pin-42 vs B-34 (`B-PLAN-SUMMARY.md:992`, SUM-0334).

Timing observable: 5–15 min on local 7B for PictureThis (`:216`).

## 12. Tests and observables

Deterministic checks (`:222-260`): SCHEMA 29 grep (stale vs later — assert **types exist**, not the integer); 5 action variants; `enum ProjectClass` + `classify_project_class`; CriticCategory count **10**; ≥4 TOML presets; ≥28 tests; `FsmSubstate::Framing`; `derive_framing_evidence`; IdeaFramingWindow file; cargo + svelte-check.

Suites (`:280-294`): `idea_framing_unit.rs` (50-fixture classifier, preset loader, multi-root, exhaustive CriticCategory); `idea_framing_contract.rs` (convergence, artifact, card, SSB); `idea_framing_block_contract.rs` (round-3 BLOCK, no auto-advance); `research_preset_dispatch.rs` (degraded_streams on T3–T4 only); `autonomous_gate_with_framing_contract.rs` (+≥4 cases on existing 14).

Paper-citation-verify: arXiv:2407.21787 replaced by **arXiv:2409.04109** (Si et al.) (`B-PLAN-SUMMARY.md:877-881`). Also FlashResearch 2510.05145, RefineBench 2511.22173.

## 13. Rebuild contract

Another harness has **this** organ iff:

- `FsmSubstate::Framing` under PLANNING; tool filter research/read only; return to PLANNING then FEASIBILITY_PROBE.
- `ProjectClass` 9 variants; regex-first classifier; LLM only on Other.
- Preset schema + `new_multi_root`; mandatory converge or `degraded_streams`.
- `idea_framer` / `idea_critic` GRC **5/800**, steps Observe/Hypothesize/Verify/Converge.
- `FramedIdea`, `TradeoffCard`, 10-variant `CriticCategory`, `run_critic_loop` max 3, final unresolved BLOCK → `idea_framing_block` + no advance.
- User must answer tradeoffs before Concepts.
- 7th AutoAdvance check FRAMING-gated; artifacts on disk; live `get_idea_framing_state` (no mock).
- ≥4 presets with ≥4 mandatory streams; 50-prompt ≥90% class accuracy.

A single “write a product brief” LLM call is a cousin.

## 14. Non-goals / banned designs

- Skipping FRAMING and decomposing from the raw idea (`:7-8`).
- idea_framer budget > 5/800 (`:204`).
- Silently dropping mandatory streams (`:205`).
- FRAMING as a new top-level FSM state (`:320-322`).
- LLM-only classification for the common 90% (`:323-324`).
- 5 or 20 critic categories (`:325-328`).
- Mock `get_idea_framing_state` (`:202`).
- Treating B-41 as a substitute for framing (`:55-56`).

## 15. Source citations

Primary: `docs/thefinalglance/B-plan/B-40_idea_framer_research_presets.md` (full file). Index: `B-PLAN-SUMMARY.md:850-882`, `:992`. Feature map: `ELAI-FEATURE-MAP.md:771-773`. Systems: `ELAI-SYSTEMS-OVERVIEW.md:192-194`. Overview: `ELAI-SPEC-OVERVIEW.md:154-156`. Exhaustive: E-B40-001..E-B40-039. 4D neighbor: `phase-4D.txt` 4D.7–4D.10, `:27-36`. B-22/B-35/B-41/B-42 as cited in B-40 reads.

## 16. Gaps

Label **UNCERTAIN**; do not fill:

- `ELAI_IDEA_FRAMING_ENABLE` named only in feature map, not B-40 body.
- SCHEMA 28→29 vs shipped 41 / pin 42 (`E-B40-002`, SUM-0334, B41-029).
- `idea_framing_block` not in the “5 action_types” list but required by the gate.
- stub_ban “no short-circuit even at 0 findings” vs zero-BLOCK early exit (SUP-049).
- Transition overrides for idea_framer skipping PlanAction/Act (2C whitelist vs default `allowed_next`). Spec lists required_steps only.
- Exact regex set for `classify_project_class`; LLM prompt for Other.
- Mandatory stream **names** per class (deferred to decision doc `:111`).
- `research_summary.streams` type (not a named struct).
- What “non-trivial idea” means for the PLANNING→FRAMING auto trigger (`:148`).
- Whether critic INFO/WARN can remain at exit (body says BLOCK veto; WARN unspecified).
