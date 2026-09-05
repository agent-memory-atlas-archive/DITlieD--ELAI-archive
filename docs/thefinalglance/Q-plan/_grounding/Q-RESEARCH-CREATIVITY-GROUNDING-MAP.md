# Q-10 grounding map — research-conditioned creativity

Date: 2026-07-12  
Scope: planning/tool and conditional ELAI explore-lane coupling only.  
Design source: `workflow/reasoning/research-coupled-creativity-design-2026-07-12.md`.  
Locked evidence contract: `workflow/reasoning/research-coupled-creativity-preregistration-2026-07-12.md`.

This map separates four things that are easy to conflate:

1. what ELAI already specifies;
2. what the live checkout already exposes;
3. what external work supports as a mechanism;
4. what Q-10 still has to prove about ELAI.

Nothing in this file is evidence that research conditioning improves creativity. The effect is a
preregistered hypothesis until the paired executable benchmarks support it.

## A. Local facts and composition anchors

| ID | Grounded fact | Canon / live anchor | Q-10 consequence |
|---|---|---|---|
| RC-1 | Recursive research already returns typed reports with sources, alternatives, contradictions, and sub-question results. | Phase 4D / B-40; `.elai_cc/crates/orchestrator/src/research_tree.rs` (`SourceCitation` near 414, alternatives near 427, contradictions near 443, `SubQuestionResult` near 525, `ResearchReport` near 1085; re-read at edit). | Consume the report; do not build a second research system. |
| RC-2 | Web research already verifies citations and attributes producer/outcome. | E-48; `.elai_cc/crates/orchestrator/src/research/web_orchestrator/citation_verifier.rs` (`VerificationOutcome` near 99, `VerifiedCitation` near 169; re-read at edit). | Packet source refs must resolve to verified artifacts. A generator cannot mint them. |
| RC-3 | Cosmos/O already compiles bounded research briefings from typed artifacts. | B-42, O-1/O-2; `.elai_cc/crates/orchestrator/src/briefing.rs::compile_cosmos_briefing` near 94; re-read at edit. | Reuse the artifact/briefing boundary, but do not inject full raw reports as the treatment. |
| RC-4 | D-15 classifies archive-relative novelty/honesty; it does not measure creative discovery or target value. | D-15 plan and `.elai_cc/crates/orchestrator/src/novelty.rs::NoveltyClass` near 74. | Report novelty, grounding, validity, execution value, and cost as separate axes. |
| RC-5 | N supplies the single measurement spine, counterfactual/ablation engine, novelty-to-budget wires, and exogenous-grounding floor. | N-1, N-9, N-10, N-16. | The runtime arm must be an ablation on the existing measurement spine and must preserve N-16. |
| RC-6 | L specifies ClaimSet grounding, risk-as-curiosity, semantic meaning clusters, and an explore-only conjecture tier separated from proven code-fix paths. | L-10, L-23, L-24. | Any runtime consumer is observe-only and explore-lane-only; the packet cannot enter code-fix, auditor, critic, or verified-case retrieval. |
| RC-7 | The harness already supports one-call candidate generation, an append-only archive/status fold, unfused axes, declared selectors, and independent receipts. | `.claude/skills/improvement-conjecture/SKILL.md`; novelty-engine v1.1 closeout. | Extend the one-call candidate input boundary; do not add a controller or shared archive bridge. |
| RC-8 | The repository already has an exactly enumerable 45-strategy executable benchmark and convergence tooling. | `.claude/tools/frontier_strategy_bench.py`; I-27 convergence bench. | Use an execution-complete space for B-CREATIVITY-0, where Recovery and value can be measured without an LLM judge. |
| RC-9 | `paper-to-plan` and `idea-intake` are the two planning-tool seams that consume research before plan authoring. | `.claude/skills/paper-to-plan/SKILL.md`; `.claude/skills/idea-intake/SKILL.md`. | Adopt packet conditioning there only after B-CREATIVITY-1 is supported. |
| RC-10 | `.agents/skills/improvement-conjecture` is absent and the `.agents` copies of paper-to-plan/idea-intake are not guaranteed byte-parity with `.claude`. | Live file inventory, 2026-07-12. | Canonical `.claude` adoption comes first; parity is a separate, explicit post-support slice. Never silently update a missing mirror. |

## B. Binding negative evidence

These are constraints, not historical footnotes.

| Evidence | Result | Binding design correction |
|---|---|---|
| B-0 ProbeUtility, K=20 | REFUTED: success/normalized-cost delta `0.0000`, CI95 `[-0.0375,+0.0375]`. | No ProbeUtility, `PROBE`, `--discriminate`, B-2, L2, or L3. |
| B-1 NOVEL-CREDIBLE selector | Narrowly SUPPORTED for archive-relative novelty, while losing judged value. | A selector may surface a different pick; it cannot be described as a generation/value improvement. |
| B-3 multi-cycle FDL, K=30 | REFUTED on discovery, execution value, and frontier yield. | No cycles, operator allocation/fade, frontier controller, B-4, or planning-archive-to-L bridge. |

Q-10 is eligible for a new test only because its intervention is materially different: exogenous verified
source mechanisms are translated into a fixed relational packet in one direction. There is no archive-conditioned
operator rotation, controller state, feedback cycle, or shared runtime memory.

## C. External mechanism grounding

| ID | Primary source | What it supports | What it does **not** establish |
|---|---|---|---|
| W-1 | [Analogical Reasoning for Biomedical Research Idea Generation (2026)](https://arxiv.org/abs/2605.11258) | A two-step source-problem extraction and target transfer based on object mappings and shared relations; reports implemented cross-domain transfers. | It does not establish an ELAI effect size or authorize raw-paper injection. |
| W-2 | [ResearchBench (2025)](https://arxiv.org/abs/2503.21248) | Separating inspiration retrieval, hypothesis composition, and ranking is a useful research-idea evaluation decomposition. | It does not make a ranking judge a sufficient primary gate. |
| W-3 | [HypoSpace (2025)](https://arxiv.org/abs/2510.15614) | Validity, Uniqueness, and Recovery can be measured in exactly enumerable deterministic hypothesis spaces. | It does not replace target execution value or grounding guards. |
| W-4 | [Execution-Grounded Evaluation for AI Research (2026)](https://arxiv.org/abs/2601.14525) | Plausible research ideas can fail under execution; best and average execution behavior should be measured separately. | It does not prove analogy packets help. |
| W-5 | [Can LLMs Generate Novel Research Ideas? (2024)](https://arxiv.org/abs/2409.04109) | Novelty and feasibility can trade off, diversity/self-evaluation are weak points, and downstream execution matters. | Human preference is not an execution receipt. |
| W-6 | [Verbalized Sampling (2025)](https://arxiv.org/abs/2510.01171) | Prompt-level sampling can increase diversity and is a relevant secondary comparator. | Diversity alone is not validated creativity or value. |
| W-7 | [Design Fixation in Large Language Models (2024)](https://arxiv.org/abs/2403.11164) | Examples can constrain subsequent design variation, motivating bounded packets and an evidence-only control. | It does not imply that all retrieval harms creativity. |

External claims are mechanism-level grounding only. Q-10's ELAI benefit remains `HYPOTHESIS` until the
locked benchmark ladder passes.

## D. Architecture decision and corrections

| Candidate | Decision | Reason |
|---|---|---|
| Full `ResearchReport` / Cosmos briefing injected into generation | REJECT RAW PROMOTION ARM | Easy composition, but raw prose raises fixation, context, leakage, and citation-laundering risk. `VERIFIED_EVIDENCE` is a bounded source-fact/support-edge projection with raw excerpts removed, not the full report. |
| Generate first, research only to verify/repair | STRONG BASELINE | Preserves grounding but may not improve candidate discovery. |
| Typed relational `ResearchInspirationPacket` | SELECTED TREATMENT | Carries mechanism, mappings, relations, invariants, disanalogies, contradictions, and source provenance without raw excerpts. |
| Shared research/conjecture memory with iterative controller | REJECTED | Recreates the failed FDL shape and risks crossing L's proven/unproven memory wall. |

### Binding packet semantics

- `evidence_scope` is always `SOURCE_ONLY`.
- Source citations support the source mechanism and the source-side predicate of a relation only; every
  source claim/relation-source-predicate carries an explicit support edge to a positively verified source
  ref. The target predicate and assertion that the relation is shared remain unvalidated.
- `transfer_hypothesis`, `predicted_observation`, and `target_falsifier` begin unvalidated.
- The packetizer owns canonical blake3 identity; the generator cannot edit identity, status, or evidence refs.
- Raw paper/report/archive prose does not enter the candidate prompt.
- Contradictions and disanalogies are retained.
- Every use emits a `TRANSFER-EDGE` from candidate to packet relation ids.
- Target acceptance still requires normal target anchors, an executable falsifier, grounding receipts, and execution evidence.

## E. Readiness ledger

The design is evaluated as specified; readiness is reported separately.

| Layer | Spec/readiness on 2026-07-12 | Q-10 rule |
|---|---|---|
| Research producers (4D/B-40, E-48, Cosmos/O) | Typed live seams found. | May be consumed now by an offline packetizer/benchmark. |
| Harness v1.1 | Live one-call candidate/archive utilities found. | May host B-CREATIVITY-0 and -1 after implementation. |
| L-23/L-24 runtime seams | Specified; corresponding live Rust symbols were not found in the inspected paths. | No B-CREATIVITY-2 or runtime type until re-grep confirms the seams exist. |
| Planning skills | Canonical `.claude` files found. | No adoption until B-CREATIVITY-1 support. |
| Runtime production consumer | Not implemented. | No production consumer until all B-CREATIVITY-2 gates support it. |

Live schema note: recon on 2026-07-12 found derived `SCHEMA_VERSION 272` with `UsageTurnEvent` the latest
named row. The authoritative value is derived in `.elai_cc/crates/orchestrator/src/ssb.rs` near 1539-1549.
Re-read `ssb.rs` and the action-type
lock at edit time. Tier-1 and Tier-2 need no SSB action type; the conditional Tier-3 should reuse the L/N
measurement row if compatible and add at most one discriminated event variant only if recon proves reuse impossible.

## F. Benchmark-to-permission map

| Gate | What it measures | What a pass permits | What failure forbids |
|---|---|---|---|
| Instrument qualification | Fixture validity, hidden-outcome isolation, manifest immutability, packet validation. | The first evidence call. | Any interpretation of benchmark outcomes. |
| B-CREATIVITY-0 | Recovery and best execution delta in a closed executable strategy space under direct, matched-scaffold, evidence-only, relational, and relation-shuffled controls. | B-CREATIVITY-1 only. | Skill adoption, Rust/runtime work, or benefit claims. |
| B-CREATIVITY-1 | Execution-validated mechanism yield and best target value on real ELAI planning fixtures. | Canonical planning-skill adoption and authoring B-CREATIVITY-2. | Changes to paper-to-plan, idea-intake, or runtime. |
| B-CREATIVITY-2 | Receipt-valid meaning-cluster coverage and best verified target outcome under PS-3/N-16/lane-wall guards. | A production `/reason --profile explore` packet consumer. | Any runtime routing/default change. |

The exact arms, statistics, non-inferiority margins, and joint gates are locked in the preregistration.

End of Q-10 research-creativity grounding map.
