# J-10: insight backpropagation tree memory — losers teach; every branch's lesson re-enters the next idea

STATUS: PLANNED (WAVE 3; tree-node insight field + audit-time extraction first, constraints-block
injection after I-4 v1 lands; BUILD — the one genuinely-new capability in the Arbor steal-list, the
-27pp ablation mechanism)

Objective: ELAI's decomposition tree records WHAT each branch did (receipts, verdicts) but nothing
distills WHY a pruned branch failed and feeds that lesson forward into the next ideation/execution
turn. Arbor's measured-largest single lever is exactly this: insight backpropagation ablates at
-27pp, MORE than the tree structure itself (-18pp) (SRC-A interface map; raw figures EXTERNAL,
design-rationale only per HD-1/ID-5 — never a target or claim). J-10 ports the FOUR hard-coded
mechanisms (not the architecture — Arbor's "loop" is a prompt, M-1): (a) extract a 1-3 sentence
insight from each raw audit/experiment report via one small LLM call; (b) re-synthesize every
ancestor's insight from its children's insights on every verdict (root = global research memory);
(c) inject ancestor-lineage insights DOWN into executor prompts; (d) render a CONSTRAINTS BLOCK
(ROOT INSIGHT priors + PRUNED LESSONS + VALIDATED FINDINGS) into every ideation turn. Hard boundary:
the tree store is H-49/SessionDag, the receipts are I-4, the cross-handoff digest is I-11 — J-10
adds the insight FIELD + the up-synthesis + the down-injection; it re-implements none (GD-1).

Estimated effort: ~1-2 sessions (~10-14 h): the field + extraction + up-synthesis = M (two short
LLM calls + deterministic walk); the down-injection into ideation = S but gated on I-4 v1 (it edits
the ladder's prompt render).
Pre-requisites: tree-node field + audit-time extraction = none beyond H-49/SessionDag node store.
Constraints-block-into-ideation leg: I-4 v1 best-of-N ranker landed (it owns the ideation prompt
render the block injects into).
Parallel-safe with: J-3/J-4/J-6/J-7/J-9 (disjoint surfaces). Coordinate with I-4 (shared ranker
prompt render) and I-11 (shared digest concept — distinct scope, verify no double-bookkeeping).
Source: ARBOR-COMBENCH-DENOVOSWE-STEAL-LIST-2026-06-11.md S-1 (the index) + SRC-A (the Arbor source
files named below — W-3: this plan reads the source directly, never the summary). Doctrine: J-0_INDEX.md.

## Context (the mechanism, from the Arbor repo read — not the paper prose)
Arbor's insight loop is four pieces of hard-coded control plus prompt content (M-1: there is no
scheduler/UCT — the "coordinator loop" is prompt instructions; do NOT port the architecture):
1. EXTRACT: after an executor/experiment report returns, one small LLM call distils a 1-3 sentence
   insight against a verbatim JSON-fields contract; the report is head+tail truncated to ~12K chars
   before the call (tree_ops / executor_run, SRC-A).
2. UP-SYNTHESIS: every ancestor re-synthesizes its own insight from its children's insights ("under
   200 words"); the walk runs to the root on every experiment, so root is a live global memory
   rebuilt each cycle (propagate_insights, tree_ops.py:518).
3. DOWN-INJECTION (exec side): an executor dispatch gathers ancestor-lineage insights and injects
   them into the executor prompt (_gather_ancestor_insights, executor_run.py:226).
4. DOWN-INJECTION (ideation side): a CONSTRAINTS BLOCK renders into every ideation turn carrying
   ROOT INSIGHT (priors), PRUNED LESSONS ("do NOT re-propose any idea that shares the same hidden
   assumption or mechanism class without explicitly explaining how it counters the lesson"), and
   VALIDATED FINDINGS ("part of the trunk's working assumptions; build on them, don't re-derive")
   (get_constraints_block, idea_tree.py:358 / :397 / :415).

ANTI-GAMING (ELAI delta, NOT in Arbor): Arbor's insight field is LLM-written and ungated — a
fabricated lesson is undetectable. ELAI binds each insight to the SSB row(s) it was extracted from
via a provenance hash (the ID-3 receipt discipline applied to memory): an insight whose claimed
source rows do not hash-match is flagged, never silently trusted. This is the single reason J-10 is
not a verbatim port.

## Existing state (B..I coverage — why this is BUILD, not ACTIVATION)
- I-5 pristine context: its "compressed contrastive constraint" IS Arbor's pruned-lesson in
  miniature — J-10 generalizes it to a full constraints discipline with a measured ablation behind
  it. REUSE the contrastive-constraint render; widen its inputs.
- I-11 BeliefState: the cross-HANDOFF sibling (digest carried across session boundaries). The
  insight tree is cross-SIBLING / cross-ROUND memory inside one run — distinct scope, shares only
  the digest idea. Verify no double-bookkeeping at edit.
- I-4 per-branch receipts: today a receipt records the verdict; J-10 adds an insight field so a
  LOSING branch teaches the next ideation turn instead of just being pruned.
- I-12 evidence-gated playbook: root insight is the RUNTIME precursor of a playbook entry — cite as
  the downstream consumer; do not pre-build the playbook here.
- H-49/SessionDag: the tree node store the insight field lives on. REUSE; no new store.
- SSB: LIVE 226/537 (J-0 constants table, re-grep at edit W-9); no insight action_type exists.

## Reads (mandatory before coding — W-3: read the Arbor SOURCE, never the steal-list summary)
1. ARBOR-COMBENCH-DENOVOSWE-STEAL-LIST-2026-06-11.md S-1 + M-1 + M-4 + S-5 (the index + the
   architecture-is-a-prompt finding + the token-cost map).
2. The Arbor source files, read directly (all under
   F:\.ELAI_workflow\docs\.source\2606-11926-arbor-hypothesis-tree-refinement\):
   - src\coordinator\tools\tree_ops.py:518   propagate_insights (the upward walk + synthesis call)
   - src\coordinator\tools\executor_run.py:368  _parse_executor_report (extraction prompt + the
     verbatim JSON-fields contract + the 12K-char head+tail truncation)
   - src\coordinator\tools\executor_run.py:226  _gather_ancestor_insights (exec-side down-injection)
   - src\coordinator\idea_tree.py:358        get_constraints_block (ideation-side down-injection;
     :397 the pruned-lesson wording, :415 the validated-findings wording)
   - src\coordinator\idea_tree.py:28-93      the Node schema (insight + result + score + code_ref)
3. workflow/papers/2606-11926-arbor-hypothesis-tree-refinement-interface.md (the ingested interface
   map — orientation only; the source files above are ground truth).
4. crates/orchestrator/src/ (H-49/SessionDag node store + I-4 receipt struct + I-5 contrastive
   constraint render + I-11 BeliefState digest — re-grep exact paths at edit, W-7).
5. The ideation/ranker prompt render site I-4 owns (the CONSTRAINTS BLOCK injection target).
6. ssb.rs + action_type_lock.rs(.golden) for the live SCHEMA_VERSION/TOTAL_VARIANTS (re-read, W-9).

## Tasks (tree-node field + extraction + up-synthesis first; ideation injection after I-4 v1)
1. Node insight field (~40 LOC): add an `insight` field (text + provenance: the SSB row hashes it
   was extracted from) to the H-49/SessionDag decomposition-tree node. Re-use the existing node
   store; no new store. blake3 on the provenance bytes (Rust-side hash, never blake2/sha2).
2. EXTRACT leg (~80 LOC): on every audit verdict (and experiment report), one small LLM call distils
   a 1-3 sentence insight against a fixed structured-fields contract; the input report is head+tail
   truncated deterministically before the call (port Arbor's ~12K idiom, tune to the 32K local
   ceiling). DETERMINISTIC parse of the reply (TD-3: the LLM produces the insight text, not the
   gate; a malformed reply is a typed error, never a guessed insight).
3. UP-SYNTHESIS leg (~70 LOC): on every verdict, walk ancestor->root re-synthesizing each ancestor's
   insight from its children's insights (bounded ~200-word render). Root insight = the run's live
   global memory. Bounded by depth (ELAI depth<=4; cheap on the local sidecar).
4. DOWN-INJECTION exec side (~50 LOC): an executor/baby-step dispatch gathers ancestor-lineage
   insights and injects them into the dispatch prompt. Placed AFTER the last cache breakpoint
   (HD-2: never bust the byte-stable prefix).
5. DOWN-INJECTION ideation side (~60 LOC, AFTER I-4 v1): render the CONSTRAINTS BLOCK (ROOT INSIGHT
   + PRUNED LESSONS + VALIDATED FINDINGS) into the ideation/ranker prompt. Reuse I-5's contrastive
   constraint render; the pruned-lesson wording ported from idea_tree.py:397 (cite source line).
6. ANTI-GAMING bind (~40 LOC): each insight carries the provenance hash of its source SSB row(s);
   a verification path recomputes the hash and flags a mismatch (the insight is shown UNTRUSTED,
   never silently consumed). This is the gate that makes the field falsifiable (ID-3).
7. SSB: +InsightPropagated Pattern-2 single-variant (Kind: Extracted / AncestorResynthesized /
   ConstraintsInjected / PrunedLessonBound), baseline+1 from LIVE 226/537 — full 6-part variant
   discipline (SsbActionType + SsbPayload + payload struct + all_test_payloads + golden row +
   TOTAL_VARIANTS bump), re-read consts at edit (W-9).
8. Decision doc: workflow/reasoning/J-10-decisions.md (extraction prompt + JSON-fields contract,
   truncation budget chosen for the local ceiling, up-synthesis word bound, what stayed
   external-figure-only, the provenance-hash binding rationale).

## WIRING_CONTRACT
produces_symbols:
  - the `insight` field (+ provenance hash) on the H-49/SessionDag decomposition-tree node
  - the extract / up-synthesize / down-inject pipeline in crates/orchestrator/src/ (insight module)
  - the CONSTRAINTS BLOCK renderer into the I-4 ideation prompt (PRUNED LESSONS + VALIDATED FINDINGS
    + ROOT INSIGHT)
  - +InsightPropagated SSB (baseline+1)
consumes_symbols:
  - H-49/SessionDag node store (the tree the field lands on — reuse, never re-store)
  - I-4 per-branch receipt struct (the insight field hangs off the receipt; feed, never re-author)
  - I-5 contrastive-constraint render (reused for the pruned-lesson block)
  - the executor/baby-step dispatch prompt seam (exec-side injection; exact site re-grepped, W-7)
  - the I-4 ideation/ranker prompt render (ideation-side injection target)
  - SSB rows of the audit verdict (the provenance source the insight hashes against — ID-3)
reachable_from:
  - the AUDITING verdict path (extract + up-synthesis fire on every verdict) [PRODUCTION path]
  - the EXECUTING dispatch path (exec-side ancestor-insight injection) [PRODUCTION path]
  - the ideation/ranker render (constraints-block injection) [PRODUCTION path, post I-4 v1]
stub_ban:
  - NO LLM call on the gate: the provenance-hash check is deterministic; the LLM writes insight TEXT
    only, never the trust verdict (TD-3)
  - NO ungated insight: an insight with no provenance hash, or a hash mismatch, is shown UNTRUSTED,
    never silently injected (ID-3 — the reason this is not a verbatim Arbor port)
  - NO literature figure (-27pp/-18pp etc.) in any user-facing claim or metrics TOML (measured-only,
    HD-1/ID-5)
  - NO .unwrap()/.expect() on prod paths; blake3 for the provenance hash (never blake2/sha2)
frontend_pair:
  an insight column / lineage tooltip on the existing plan-or-branch view drawer tile (decide exact
  drawer at edit; W-4 satisfied or W-6-justified in the decisions doc).
observable_effect:
  trigger: "elai run over a fixture decomposition with >=2 sibling branches where one is pruned;
    the pruned branch's lesson must re-enter the next ideation turn"
  expect_ssb: "InsightPropagated rows (Extracted + AncestorResynthesized + at least one
    PrunedLessonBound + ConstraintsInjected) recomputable from the journal under ELAI_SEED (GD-2)"
  expect_artifact: "the next ideation turn's rendered prompt provably contains the pruned branch's
    lesson in the CONSTRAINTS BLOCK (grep the journaled request); the root insight reflects both
    siblings' outcomes. NEGATIVE: an insight with a forged/absent provenance hash is shown UNTRUSTED
    and is NOT injected (proves the gate discriminates, R2); the --no-insight control arm re-proposes
    the pruned idea (proves the mechanism is load-bearing)."

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/J-plan/J-10_insight_backprop_tree_memory.md --run-trigger
rg "InsightPropagated" .elai_cc/crates/orchestrator/src/ssb.rs                 # variant exists (all 6 parts)
rg "InsightPropagated" .elai_cc/crates/orchestrator/tests/action_type_lock.rs # golden + TOTAL_VARIANTS bump
rg -c "unwrap\(\)" .elai_cc/crates/orchestrator/src/perception/ 2>/dev/null; rg -c "unwrap\(\)" .elai_cc/crates/orchestrator/src/  # insight module MUST be 0 on prod paths
rg "blake2|sha2" .elai_cc/crates/orchestrator/src/ | rg -i insight            # MUST be empty (blake3 only)
test -f workflow/reasoning/J-10-decisions.md
```

## ACCEPTANCE
- [ ] the tree-node insight field + extract + up-synthesis ship and fire on every verdict; the
      ideation-side CONSTRAINTS BLOCK leg ships only after I-4 v1 (sequencing honored).
- [ ] every insight carries a provenance hash; a forged/absent hash yields UNTRUSTED (ID-3 gate
      bites); the LLM never produces the trust verdict (TD-3).
- [ ] scaffold/injection placed after the last cache breakpoint; HD-2 byte-stability assert holds.
- [ ] all shipped numbers ELAI-measured per config; the -27pp/-18pp figures appear only as cited
      external rationale (HD-1/ID-5).
- [ ] SSB 6-part discipline complete; baseline re-read at edit (W-9); cross_lang_diff PASS if the
      frontend pair lands.
- [ ] local-lane proven: both LLM calls (extract + up-synthesis) run on the remote 7B sidecar
      (short-input/short-output summarization, no frontier dependency — S-1 verdict).
- [ ] cargo check --workspace GREEN; auditor PASS on -p elai-orchestrator (+contracts).
- [ ] no new dependency without explicit approval.

## Tests to add
- crates/contracts/tests/j10_insight_backprop_contract.rs (~140 LOC): (a) extract produces a bounded
  insight with a provenance hash from a fixture report; (b) up-synthesis walks ancestor->root and
  root insight reflects two sibling outcomes; (c) a pruned branch's lesson appears in the next
  ideation turn's CONSTRAINTS BLOCK; (d) a forged provenance hash yields UNTRUSTED and is NOT
  injected (anti-gaming arm); (e) the InsightPropagated event lands with each correct Kind on the
  fixture; (f) broken arm: --no-insight control re-proposes the pruned idea (mechanism is
  load-bearing, R2). traced-test asserts on the journaled CONSTRAINTS BLOCK content (tracing-test
  logs_contain inside assert! per tracing-test-assert.md T-1).

## Files touched
- CREATE: crates/orchestrator/src/insight/{mod,extract,synthesize,inject,provenance}.rs (~340 LOC)
- MODIFY: the H-49/SessionDag node struct (+insight field + provenance hash)
- MODIFY: the I-4 receipt struct (insight field hangs off the receipt) + the ideation/ranker prompt
  render (CONSTRAINTS BLOCK injection; exact site at edit, W-7)
- MODIFY: the executor/baby-step dispatch prompt seam (exec-side ancestor-insight injection)
- MODIFY: ssb.rs + action_type_lock.rs(.golden) (+1 variant, 6 parts)
- CREATE: workflow/reasoning/J-10-decisions.md

## Citations / Grounding
- SRC-A: the Arbor source files named in Reads #2 (read directly, W-3) — the FOUR mechanisms +
  the Node schema + the constraints-block wording. The steal-list S-1 is the INDEX, not the source.
- ARBOR-COMBENCH-DENOVOSWE-STEAL-LIST-2026-06-11.md S-1 (landing + composes-with + local-lane
  verdict + anti-gaming note) / M-1 (architecture is a prompt: do NOT port the loop) / M-4 + S-5
  (token-cost map: the insight loop is two SHORT calls, not a cost driver).
- ELAI ground truth: J-0 constants (LIVE 226/537, re-read at edit), I-4/I-5/I-11/I-12/H-49 shipped
  status per the I/H summary indexes (STALE-by-construction; verify at edit, SRC-4 discipline).
- Hard rules invoked: blake3-not-blake2 (provenance hash), TD-3 (LLM never the gate), ID-3 (receipt
  provenance), HD-1/HD-2 (measured-only + cache byte-stability), W-7/W-9, GD-1/GD-2.

## Open questions / risks
- exec-side vs ideation-side injection order when I-4 v1 has not landed: ship legs 1-4 (field +
  extract + up-synthesis + exec injection) independently; gate ONLY the ideation CONSTRAINTS BLOCK
  on I-4 (it owns that render). Named so the wave does not stall on I-4.
- truncation budget vs the 32K local ceiling: Arbor's ~12K head+tail assumes a frontier window;
  re-tune for the local lane at edit and record the chosen budget in the decisions doc.
- double-bookkeeping with I-11 BeliefState: distinct scope (cross-round vs cross-handoff) but verify
  at edit that an insight is not both up-synthesized AND digested into the same handoff row.
- insight quality on the 7B lane: extraction is short summarization (7B-capable per S-1); if the
  ideation-side block quality is weak, route ONLY the ideation hypothesis turn to cloud (S-6 hybrid),
  never the extraction/synthesis calls. Measure before defaulting call-2-always ON (GD-4).

## Next
Lands the Arbor steal-list's one genuinely-new capability (the -27pp mechanism) on ELAI's existing
tree without porting Arbor's 500-turn architecture (M-1). Feeds I-12's playbook (root insight is its
runtime precursor) and gives I-4's loser branches a teaching role. J-11 carries the rest of the
steal-list (the gate/checker/SFT amendments) as pointer-line insertions into existing plan files.

End of J-10.
