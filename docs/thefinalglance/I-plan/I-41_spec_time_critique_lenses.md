# I-41: spec-time critique lenses (L1-L8) — plural design-phase doubt, advisory, off the load-bearing path

- Track 19 — DESIGN-PHASE CRITIQUE + TRUST-CLAIM DISCIPLINE | lever PLURAL-DOUBT | grounding MODERATE | nature BUILD | effort M | WAVE 2 | binds ID-4 + the §6 ban-LLM-judge hard rule
- Premise: ELAI's strength is post-implementation/wiring doubt (off-agent deterministic gates); its DESIGN-phase
  critique is comparatively thin (B-40 idea-critic, B-41 per-level auditors, B-38 ambiguity, the 9-step
  plan-improvement protocol). The source design (workflow/research/grok-elai-trust/ELAI-CRITIQUE-LENSES.md,
  re-authored from alexself.dev/blog/automated-doubt) adds 8 named critique lenses that fire on PROSE/SPEC
  before any deterministic gate has an artifact to hash. They are LLM-judged and therefore NOISY/BYPASSABLE
  (MSR-2026: 60.2% of LLM-review PRs in the 0-30% signal band; NAACL Mar-2025: all 8 inline LLM defenses
  bypassed >50% ASR) — so they SURFACE to a human / feed plan-improvement and NEVER emit an accept/block
  verdict. This is the front-of-pipe complement to the off-agent spine, not a replacement for it.

## Steal/build
A `LensSet` of 8 re-authored critique prompts run at DESIGN/SPEC time over a plan/spec artifact:
L1 Assumption Excavator (the human-readable front end of the Phase-2B grounding-sweep — names interface
assumptions, classifies GROUNDED/VERIFY/INGEST, verifies nothing itself), L2 Implied Completeness Detector,
L3 Ambiguity Mapper (composes the SHIPPED B-38 AmbiguityScorer — lens produces the human map, scorer the
gate-time number), L4 Gap Analyzer, L5 Pre-Implementation Architect (enforces the wiring PRE_GATE: flags a
plan unit shipping a producer without its in-plan consumer + smoke trigger), L6 Public-Interface & API-Contract
Validator (the design-time mirror of cross_lang_diff.py + the E-45 action_type golden lock), L7 Documentation
Validator, L8 Anxiety Reader (catch-all, run last). Every finding carries the schema {lens, severity
(BLOCKER-CANDIDATE|CONCERN|NIT), location (REQUIRED concrete anchor), claim, why_it_matters,
suggested_resolution, confidence}. HARD anti-noise rule: a finding with no concrete anchor (spec heading /
file:line / symbol / quoted phrase) is DROPPED, not surfaced. Bounded: max 8 findings per lens per pass, deduped
across lenses by (location, claim); lens count scales to artifact risk (SIMPLE: 2-3 lenses; HEAVY: full set —
per the doc, the full 8 are "not cheap"). Declined: importing github.com/aself101/agents-and-pipelines directly
(skill-import-policy E-43 mcp-scan whole-directory gate would block first — the lenses are re-authored native to
avoid the import); running lenses on shipped code (that is the auditor + off-agent spine, not these).

## Existing ground truth
- B-40 idea_framing / FsmSubstate::Framing + idea-critic loop: SHIPPED (memory project_b40_shipped) — the
  primary wire-in point for the lenses (PLAN-asserted; executor re-greps per W-7).
- B-38 AmbiguityScorer: SHIPPED — L3 composes it, does NOT re-implement (GD-1; PLAN-asserted, re-grep W-7).
- E-45 action_type golden lock + .claude/tools/cross_lang_diff.py: SHIPPED harness — L6 is their design-time
  mirror (GD-1; PLAN-asserted, re-grep W-7).
- Phase 2B grounding-sweep (VerificationMethod FILE/SOURCE/...): SHIPPED — L1 feeds it, names assumptions only.
- planning-workflow plan-improvement protocol (the 9-step adversarial injection): the harness skill the lenses
  surface findings into (a harness doc, not a Rust symbol — the CLI entrypoint below gives the runtime hook).
- D-13 Adversary / E-6 / E-10 critic: SHIPPED/PLANNED critique surfaces — L8 is distinct (prose-time, not
  trajectory-time); composes, restates none.
- No `LensSet` / spec-time critique module exists today (IN — net-new; confirmed by absence of any L1-L8 session
  across B..I summaries; this session is the first).

## REVISED scope (W-10 tiered)
SHIP NOW (Tier-1):
- crates/orchestrator/src/critique/lenses.rs: the 8 lens prompts + LensSet + Finding schema + anchor-required
  drop rule + per-lens cap + (location,claim) dedup
- `elai critique <spec-path> [--lenses L1,L4,L8] [--risk simple|heavy]` CLI: runs the selected lenses, writes
  findings to workflow/reasoning/<artifact>-critique.md + advisory SSB rows
- wire the LensSet into the B-40 idea-critic loop as an ADVISORY surfacer (findings inform the human/critic;
  they never set the framing accept/reject verdict)
- CritiqueLensFinding SSB variant tagged advisory=true (AdvisoryFinding / LensRun / DroppedNoAnchor arms)
- contract test: NO lens output ever reaches a gate verdict path (the load-bearing-path-isolation invariant)

DEFER (Tier-2 / gated):
- automatic injection into the planning-workflow 9-step protocol (a harness-skill edit, sequenced after the
  runtime lens module is proven)
- per-lens eval/positive.jsonl + eval/negative.jsonl (the E-43 per-skill eval pattern) once findings accrue
- lens-prompt self-optimization (E-8 GEPA lane) — out of scope until a labelled finding corpus exists (ID-5)

## Integration
- CREATE: crates/orchestrator/src/critique/lenses.rs (~220 LOC: 8 prompts + LensSet + schema + drop/dedup)
- CREATE: crates/cli/src/critique_cmd.rs (~90 LOC: the `elai critique` subcommand)
- MODIFY: crates/orchestrator/src/idea_framing.rs — idea-critic loop calls LensSet as an advisory surfacer
  (PLAN-asserted seam; executor re-greps per W-7)
- CREATE: crates/contracts/tests/i41_critique_advisory_only_contract.rs (~60 LOC)
- CREATE: workflow/reasoning/I-41-decisions.md
- frontend_pair: advisory CritiqueLensFinding rows render in the ReasoningDrawer (or the B-40 framing panel) as
  a non-blocking findings list (PLAN-asserted; executor confirms which drawer at EXECUTING — W-6 satisfied by
  the existing SSB-backed drawer, no new gate UI; a BLOCKER-CANDIDATE routes to the human, never auto-blocks).

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/critique/lenses.rs::LensSet (the 8-lens runner)
  - crates/orchestrator/src/critique/lenses.rs::Finding (the anchor-required finding schema)
  - crates/cli/src/critique_cmd.rs::run (the `elai critique` entrypoint)
  - CritiqueLensFinding (new SSB action_type, advisory=true; arms AdvisoryFinding/LensRun/DroppedNoAnchor) —
    baseline+1 from LIVE 221/532 as of 2026-06-10 (ssb.rs:2981 / action_type_lock.rs:601); re-read at edit (W-9)
  - crates/contracts/tests/i41_critique_advisory_only_contract.rs
consumes_symbols:
  - B-38 AmbiguityScorer (why: L3 produces the human map from the same signal the scorer numbers; compose, do
    NOT re-implement the scorer)
  - E-45 action_type golden lock + cross_lang_diff.py (why: L6 mirrors them at design time; compose)
  - Phase 2B grounding-sweep VerificationMethod (why: L1 names assumptions that feed the sweep; do NOT verify)
  - B-40 idea_framing / idea-critic loop (why: the advisory wire-in point; the lenses inform, never gate)
  - the cloud/local model runtime (B-37 RuntimeAdapter) for the LLM-judged lens passes
reachable_from:
  - `elai critique <spec-path>` CLI -> critique_cmd::run -> LensSet::run -> CritiqueLensFinding rows  [PRODUCTION critique surface]
  - idea_framing.rs idea-critic loop -> LensSet::run (advisory) -> CritiqueLensFinding rows  [PRODUCTION framing surface]
  - a #[cfg(test)] / contracts/ caller NEVER satisfies this (R7)
stub_ban:
  - NO lens output on ANY accept/block verdict path (the §6 ban-LLM-judge hard rule; the isolation invariant)
  - NO surfacing a finding with no concrete anchor (the anchor-required DROP rule — DroppedNoAnchor row instead)
  - NO re-implementation of B-38 / E-45 / grounding-sweep (compose the shipped primitives — GD-1)
  - NO importing github.com/aself101/agents-and-pipelines (E-43 mcp-scan gate; lenses are native re-authored)
  - NO .unwrap()/.expect() on production paths; NO blake2/sha2 (blake3 only)
frontend_pair:
  CritiqueLensFinding rows render as a non-blocking advisory findings list in an existing SSB-backed drawer
  (ReasoningDrawer or the B-40 framing panel). Justified: the advisory row is the observable artifact; a
  BLOCKER-CANDIDATE is a human-routed chip, never an auto-block control (W-6 satisfied without a new gate UI).
observable_effect:
  trigger: "elai critique <fixture-spec-with-a-known-assumption-and-a-known-gap>"
  expect_ssb: "CritiqueLensFinding(AdvisoryFinding) rows whose location anchors point at the planted assumption
    (L1) and the planted unspecified-edge (L4); a CritiqueLensFinding(DroppedNoAnchor) row when a lens emits an
    anchorless finding; advisory=true on every row"
  expect_artifact: "workflow/reasoning/<artifact>-critique.md written with the findings; the contract test
    asserts the run produces ZERO gate-verdict rows (no accept/block emitted anywhere — the load-bearing path is
    provably untouched); a finding with no anchor never reaches the markdown (it is dropped, not surfaced) — R1"

## Grounding
- Source design: workflow/research/grok-elai-trust/ELAI-CRITIQUE-LENSES.md (the re-authored L1-L8 + doctrine);
  origin alexself.dev/blog/automated-doubt (WebSearch-confirm before any external headline — map citation caveat).
- Noise evidence (why advisory-only): MSR-2026 60.2% in 0-30% signal band; NAACL Mar-2025 all-8-defenses
  bypassed >50% ASR (both carried from ELAI-IMPROVEMENT-PLAN.md §6/R5; WebSearch-confirm before citing as fact).
- Composition substrate (PLAN-asserted, re-grep W-7): B-38 AmbiguityScorer, E-45 golden lock + cross_lang_diff,
  Phase 2B grounding-sweep, B-40 idea_framing.
- Doctrine: binds ID-4 (a neural/LLM signal may ORDER/SURFACE, never decide — deterministic veto is supreme) +
  the ELAI-IMPROVEMENT-PLAN.md §6 hard rule (ban any LLM-judge from the load-bearing verdict path).
- SSB: CritiqueLensFinding +1 variant (advisory) — baseline+1 from LIVE 221/532 as of 2026-06-10; re-read at edit (W-9).

## COMPOSES-WITH
- B-38 AmbiguityScorer — L3's gate-time number; the lens produces the human map. Do NOT re-implement.
- E-45 golden lock + cross_lang_diff.py — L6's gate-time enforcement; the lens is its design-time mirror.
- Phase 2B grounding-sweep — L1 names assumptions; the sweep verifies them. Do NOT verify inside the lens.
- B-40 idea-critic loop — the advisory wire-in; the lenses inform the critic, never the framing verdict.
- I-42 (critique-only profile) — consumes this LensSet as its design-phase critique engine.
- D-13 / E-6 / E-10 critic surfaces — trajectory/ship-time critique; L8 is the prose-time sibling, restates none.

End of I-41.
