# I-29: diffusion draft lane — dLLM as untrusted generator behind the full veto stack, CRDT merge for parallel patches

- Track 16 — ROUTING + BUDGETS + MEASUREMENT | lever GENERATIVE | grounding MODERATE | nature BUILD | effort L | WAVE 5 | gated (new-dep + network, user approval required)
- Premise: Mercury-class dLLMs generate drafts in one forward pass at speeds unmatched by AR models. I-29
  consumes that speed with a trust-preserving wrapper: drafts pass the IDENTICAL I-17/I-20 veto stack as any
  other generator output; CRDT merge semantics let parallel sub-agents patch a shared draft without races.
  the dLLM is an UNTRUSTED generator — it has no special status. TTD-DR's full denoising-research loop is
  DECLINED (out of scope; the trust story requires the full veto stack to be in place first). new-dep +
  network access = user approval gated before any code lands.

FULL-FORM EXPANSION REQUIRED before EXECUTING (I-0 reading order rule).

## Steal/build
the dLLM client is a NEW RuntimeAdapter variant (B-37 adapter pattern — adds a new arm, does not fork the
trait). anti-slopsquat protocol is MANDATORY before naming any dLLM client crate: `npm view / pypi / cargo
search` + repo URL + last-published date + license, user-confirmed. draft flow: (1) the orchestrator assigns
a sub-task to the dLLM lane; (2) the dLLM generates a draft in one pass (or a few steps for parallel
filling); (3) the draft enters draft_merge.rs for CRDT reconciliation if multiple sub-agents patched the
same document; (4) the merged result passes the I-17 tri-layer veto (L1 HARD VETO is supreme; a dLLM draft
that fails compile is treated identically to an AR draft that fails compile — same SSB row, same pruning);
(5) surviving drafts enter I-20 budgeted mutation. CRDT semantics: the merge is deterministic and
commutative — parallel sub-agents each append patch operations to a shared CRDT log; merge resolves
conflicts at the character-/line-level by last-write-wins on a content hash, not by LLM judgment. C-32
MiniCritic fires at each semantic checkpoint (merge point). gate parity: the DiffusionDraftEvent rows prove
gate parity — DraftVetoed rows MUST reference a TriLayerVerdict(HardVeto) row; there is no shortcut for
diffusion output. W-2 ship posture: the lane ships default-ON ONLY after its corruption rate (fraction of
drafts reaching L1 HARD VETO that are genuinely bad, not just slow) is bounded on >=3 real runs (GD-4);
until then it is an allowlisted experiment in wire-dark-allowlist.toml (darwinian-evolve precedent). TTD-DR
full denoising-research loop DECLINED: the trust preserving claim requires the veto stack to be end-to-end;
a research loop that self-denoises its own outputs bypasses it. the narrow steal is speed, not autonomy.

## Existing ground truth
- B-37 RuntimeAdapter trait: the adapter pattern the dLLM client follows. PLAN-asserted; executor re-greps per W-7.
- C-32 MiniCritic: semantic checkpoint at merge points. PLAN-asserted; executor re-greps per W-7.
- E-48 research orchestrator: the long-form consumer where draft speed is most valuable. PLAN-asserted; executor re-greps per W-7.
- I-17 tri-layer veto stack (BLOCKING prerequisite for gate parity): the identical gate the dLLM draft must pass. depends I-17.
- IS-19: wiring tooling + R8 (draft_merge.rs is a measurement + merge artifact; neither the dLLM client nor the merge engine is reachable from the generator path).
- GD-4: default-ON gate requires >=3 real runs with FP/corruption rate bounded.

## REVISED scope (W-10 tiered)
SHIP NOW (Tier-1):
- draft_merge.rs deterministic CRDT merge skeleton (~100 LOC — patch log, content-hash conflict resolution,
  C-32 checkpoint call sites) — in-process, no external dep
- wire-dark-allowlist.toml row for the diffusion lane (type=experiment; owner; expiry_date; promotion_plan=I-29; smoke test path)
- contract test crates/contracts/tests/i29_diffusion_draft_lane_contract.rs (gate-parity fixture: a vetoed
  dLLM draft produces DiffusionDraftEvent(DraftVetoed) referencing TriLayerVerdict(HardVeto))
- decision doc workflow/reasoning/I-29-decisions.md

DEFER (Tier-2 / gated — user approval required before any code in this tier):
- dLLM client RuntimeAdapter variant (new-dep: anti-slopsquat verify + user-confirm the crate name/version/license first)
- network endpoint wiring for the dLLM service (requires the approved dep + a live dLLM endpoint)
- promotion to default-ON (requires GD-4 corruption-rate bound from >=3 real runs)
- parallel sub-agent CRDT volume test (requires the dLLM client live)

## Integration
- CREATE: crates/orchestrator/src/draft_merge.rs (~100 LOC — Tier-1 CRDT skeleton)
- CREATE (Tier-2, gated): crates/orchestrator/src/runtime/diffusion_adapter.rs (RuntimeAdapter variant — user-approved dep + anti-slopsquat first)
- MODIFY: wire-dark-allowlist.toml (new experiment row for diffusion lane)
- seams from the map: Track 16 anchor — C-32 MiniCritic + E-48 research orchestrator + I-17 gates + B-37 adapter pattern
- frontend_pair: draft-lane badge on generation cards (shows DraftAccepted / DraftVetoed status per generation). cross_lang_diff.py covers Tauri -> Svelte. Justified (not N/A): the badge is the user-visible gate-parity signal; W-6 is satisfied by the badge.

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/draft_merge.rs::DraftMerger (CRDT merge engine, Tier-1 in-process)
  - crates/orchestrator/src/runtime/diffusion_adapter.rs::DiffusionAdapter (Tier-2, gated — RuntimeAdapter variant for dLLM client)

consumes_symbols:
  - B-37 RuntimeAdapter trait (dLLM client implements the same trait, no special status) — GD-1
  - C-32 MiniCritic (semantic checkpoint at CRDT merge points) — GD-1
  - I-17 tri-layer veto stack (the IDENTICAL gate; DraftVetoed references HardVeto rows) — GD-1
  - I-20 budgeted mutation (surviving drafts enter mutation; no shortcut) — GD-1

reachable_from:
  - crates/orchestrator/src/search/ladder.rs or task_runner (sub-task dispatch to dLLM lane, Tier-2) -> DiffusionAdapter -> DraftMerger -> I-17 veto  [PRODUCTION search surface, Tier-2 gated]
  - Tier-1 contract test exercises DraftMerger directly (in-process) — test callers do not satisfy R7; the Tier-2 production path is the R7 evidence

stub_ban:
  - NO shortcut gate for dLLM output — every draft passes the IDENTICAL I-17 L1 HARD VETO (gate parity is the trust claim)
  - NO TTD-DR full denoising-research loop — DECLINED; not a stub, must not appear as a placeholder or deferred TODO
  - NO new-dep (dLLM client crate) without anti-slopsquat verification + user approval first (HARD RULE — CLAUDE.md)
  - NO default-ON until GD-4 corruption-rate bound is measured (<=3 runs are insufficient)
  - NO .unwrap()/.expect() on production paths; NO blake2/sha2
  - NO LLM-judgment conflict resolution in the CRDT merge — deterministic content-hash only

frontend_pair:
  draft-lane badge on generation cards. DiffusionDraftEvent rows (DraftAccepted / DraftVetoed / CrdtMerged) surface as badge states. cross_lang_diff.py covers Tauri -> Svelte. Justified: badge is the W-6 surface; the gate-parity claim is only user-visible via this badge.

observable_effect:
  trigger: "(Tier-1) python .claude/tools/wiring_contract_check.py --bn-file ... --run-trigger (contract test only); (Tier-2, gated) elai run --task <draft-fixture> --diffusion-lane"
  expect_ssb: "DiffusionDraftEvent(DraftVetoed) referencing a TriLayerVerdict(HardVeto) row for a seeded bad draft (proves gate parity); DiffusionDraftEvent(CrdtMerged) for a two-agent patch fixture; DiffusionDraftEvent(DraftAccepted) for a clean draft"
  expect_artifact: "two parallel patches to one draft merge byte-identically under seed (CrdtMerged row, same content hash on both runs — GD-2); the diffusion-lane-OFF arm leaves all existing generation paths untouched (regression guard); NEGATIVE: a dLLM draft bypassing I-17 is structurally impossible — the DraftVetoed row must always reference a HardVeto row (contract test asserts the join)"

## Grounding
- B-37 RuntimeAdapter (the adapter pattern; Track 16 anchor IS-13 for CompiledPipelineStore as storage precedent — analogous pattern).
- I-17 (the veto stack I-29 gates against): gate parity is the entire trust claim.
- GD-4 (FP/over-block bound before default-ON): the corruption rate must be measured; the allwolisted experiment row in wire-dark-allowlist.toml is the gating mechanism until then.
- W-2 (kill-switch posture): the lane ships behind the existing allowlisted kill-switched darwinian-evolve precedent (I-0 wiring doctrine).
- grounding: MODERATE (the speed claim for Mercury-class dLLMs is from the research corpus; not yet ELAI-measured; tagged [DESIGN]/[WEAK] until the Tier-2 dep lands and ELAI runs it). SSB row planning is STRONG (GD-1 reuse).
- SSB: +DiffusionDraftEvent (Kind: DraftAccepted / DraftVetoed / CrdtMerged). baseline+1 from LIVE 221/532 as of 2026-06-10 (ssb.rs:2981 / action_type_lock.rs:601); re-read at edit (W-9).
- arXiv IDs: none asserted for the CRDT merge (deterministic, hand-rolled). WebSearch-confirm any Mercury-class dLLM paper citation before headlining.

## COMPOSES-WITH
- B-37 RuntimeAdapter: do NOT fork the trait; add one variant.
- C-32 MiniCritic: do NOT re-implement the semantic critic; call it at merge checkpoints.
- I-17 tri-layer veto: do NOT re-implement L1 HARD VETO; route dLLM output through the identical stack.
- I-20 budgeted mutation: do NOT re-implement mutation; surviving drafts enter the existing scheduler.
- darwinian-evolve allowlist row precedent (W-2): the wire-dark-allowlist.toml row is the model; do NOT invent a new gate mechanism.

End of I-29.
