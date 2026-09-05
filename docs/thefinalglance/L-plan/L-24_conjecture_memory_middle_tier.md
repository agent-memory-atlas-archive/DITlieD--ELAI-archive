# L-24: the conjecture-memory middle tier — status-explicit unproven ideas between the journal and the verified case graph (explore lane ONLY) + the three collab riders

STATUS: PLANNED (WAVE 2; M-L; NEW tier + 3 EXTENSION legs — the DeepReason COLLAB wave: the author's one real hit on ELAI ("proven-only case minting is clean but sterile" — memory can only hold what current evaluators recognize) adopted for the EXPLORE lane; depends L-22/L-23 for the retrieval consumer, store ships loop-independent)

PROOF_SPINE (PSR-2): feeds PS-3 (hallucination / grounded-claims delta, L-10) — DECLARED, three ways: (leg CM) the conjecture tier is the explore-lane memory whose retrieval-ON vs retrieval-OFF survivor-map quality is an L-10 explore-arm ablation; (leg A) the novelty instrument IS a direct L-10 bench instrument under the CGG ON/OFF ablation; (leg B) the typed unblock condition makes L-10's abstention rows scoreable (a refusal that declares what evidence would change the answer is a gradeable grounded-claim, not a dead end). Leg D (flywheel diversity guard) is PROOF_SPINE: none (flywheel-integrity discipline on the D-026-gated I-30/I-31; user-acked 2026-07-11), riding inside this PS-3 file — the same pattern as L-23's S2 leg.

Objective: ELAI's memory plane has two tiers: the append-only journal (SSB, 3B — full genealogy, everything, no status semantics) and the verified case graph (J-96..J-98 — PROVEN fixes only, minted on an F-4 proof, the strict filter that makes code-fix reuse safe). Nothing in between: an unproven-but-live conjecture from an explore run is either a raw journal row (unretrievable as an idea) or nothing. The DeepReason collab surfaced this as the one real hit ("clean but sterile" — handoff deepreason-steal-and-collab.md CURRENT STATUS improvement (c)): a memory that only holds what its current evaluators can recognize cannot hold ideas AHEAD of the current receipt plane. L-24 adds the middle tier: unproven conjectures preserved as first-class retrievable objects with an explicit COMPUTED status (live / refuted / pending / suspended), provenance, attached criticisms, and declared missing tests (exactly what evidence would settle each). HARD BOUNDARY (locked user decision, do NOT re-litigate): this tier serves the EXPLORE/RESEARCH lane ONLY. The code-fix reuse path keeps its strict proven-only minting filter — J-96/J-97 are UNCHANGED and an unproven conjecture must NEVER be retrievable into the code-fix path (EX-5, structural: lane-tagged store + negative contract test). GD-1 callouts: statuses compose the N-16 evaluator-version edge machinery (never re-implement S4); quarantine reuses the F-5 cascade shape; the skeleton IS the L-22 falsifier-carrying claim class; the retrieval consumer IS the L-23 explore pack. The tier is a materialized VIEW over the journal (L-16 fold pattern) — statuses are COMPUTED off journal rows by a deterministic fold, never hand-curated.

Estimated effort: ~1 session (~M-L). All in-process (W-10 Tier-1); no network leg; the novelty-instrument leg runs inside the L-10 bench harness (fixture-driven now, live explore arm when L-23's lands).
Pre-requisites (PLAN-asserted; executor re-greps per W-7): L-22 falsifier skeleton class + L-23 explore.rs (NOT yet executed — see FORK-1); N-16 evaluator-id+version-hash verdict field (see RECON-3); D-17 refusal SLA seam (ELAI_REFUSAL_SLA_TICK_ENABLE, DARK); L-4/F-12 NLI meaning-cluster judge; SSB (ssb.rs) + RRA; F-5 staleness/quarantine; K-19 SF-1..SF-4 (binds this file: memory-reuse class).
Parallel-safe with: N-16 (compose via the shared verdict-field shape, disjoint files otherwise); Q-track (disjoint).
Source: workflow/handoff/deepreason-steal-and-collab.md CURRENT STATUS improvements (a)(b)(c)(d) — the live collab exchange with the DeepReason author (his 5:01PM reply, the "clean but sterile" hit + the judge-as-critic reframe); workflow/reasoning/steal-deepreason-2026-07-08.md §1/§4 (the status model + case-law closure prior art). Doctrine: L-0_INDEX.md (EX-1..EX-4; this file ADDS EX-5/EX-6).

## Context
Why this matters: DeepReason's status model (harness-spec-v1.3.md §4) is the prior art, and its load-bearing properties transfer whole: labels are COMPUTED by a deterministic pass over the artifact graph, never curated (accepted / refuted / suspended_unsupported); refuting a premise makes dependents `suspended_unsupported`, NOT refuted — "orphaned != false" (spec §4 line "Refuting a premise => dependents become suspended_unsupported"); N1 no-absorbing-status — every status admits an exit, refuted can reinstate when its refuter falls (spec §4 N1), which composes exactly with N-16's S4 evaluator-refutation verdict-collapse (a later-refuted evaluator collapses what it vouched for — the verdict rows already carry the evaluator-version edge N-16 lands); negative case law lives at the registration GATE and is never rendered into prompts (spec §11.5 atlas split = L-23 S3/EX-1, already ours). What L-24 adds over DeepReason: ELAI's version is journal-grounded (the fold is byte-recomputable under ELAI_SEED, GD-2), lane-partitioned (his engine has one lane; ELAI must protect the proven-only code path — the locked domain split), and bound by K-19's SF-1..SF-4 because a retrievable memory IS memory-reuse-class work (SF-3 observe-first + NEVER-into-the-auditor; SF-4 never-DELETE + poisoning quarantine). All DeepReason figures remain n=1 design-rationale (ID-5); L-10 measures ELAI's own deltas.

## Existing state (B..N coverage — PLAN-asserted where the unit is unexecuted; executor re-greps per W-7)
- SSB journal (ssb.rs) [WIRED] — the genealogy tier below; LIVE baseline 581 variants / SCHEMA_VERSION 272 derived (ssb.rs:1530-1544, read 2026-07-11); NEVER hard-code, re-read at edit (W-9).
- Verified case graph J-96..J-98 [PLANNED, UNCHANGED BY THIS FILE] — the proven tier above; mint-on-F-4-proof stays the filter; L-24 touches ZERO of its files (EX-5 negative check).
- L-22 falsifier-carrying skeleton {claim, mechanism, scope, falsifier[]} [PLANNED] — the conjecture object IS this class; a declared missing test = a falsifier not yet compiled/run. [L-22]
- L-23 explore.rs round loop + survivor map + relapse gate [PLANNED] — the production retrieval consumer + the gate the refuted set feeds. [L-23]
- N-16 evaluator-id + version-hash on verdict rows + S4 quarantine cascade + S7 criticism-debt metric [PLANNED] — the closure machinery statuses compose; a conjecture's criticism rows carry the same edge. [N-16]
- F-5 staleness cascade [WIRED-class] — the quarantine-not-delete shape. [N-16 ground truth]
- D-17 honest-refusal SLA [DARK(ELAI_REFUSAL_SLA_TICK_ENABLE)] + L-7 hallucination policy gate [PLANNED] — the refusal rows leg B enriches. [feature map]
- L-4 semantic-entropy NLI meaning-clustering + F-12 DeterministicNliJudge [PLANNED] — the novelty instrument's engine (never cosine-over-hashes; his own embedder failed, steal §3.3). [L-4/F-12]
- L-10 hallucination-delta bench + its explore arm [PLANNED, PS-3 owner] — where leg A's number lands. [L-10]
- I-30 verifier-ranked SFT exhaust + I-31 GRPO flywheel [PLANNED, D-026 GPU-gated, LAST + blocking guards] — leg D's target; code lands THERE, L-24 lands the binding contract. [feature map 1711-1727]
- K-19 SF-1..SF-4 self-improvement fitness doctrine [PLANNED] — BINDS this file (memory-reuse class): execution-grounded fitness only, observe-first, never-into-auditor, never-DELETE, poisoning quarantine. [feature map 1791-1809]

## Reads (mandatory before coding)
1. workflow/handoff/deepreason-steal-and-collab.md (head + CURRENT STATUS — the four improvements + the locked decisions); workflow/reasoning/steal-deepreason-2026-07-08.md §1 (status model), §4.2 S4 (closure), §4.3 REJECTS (no Dung adjudicator — statuses here are journal-fold labels, not argument-graph extensions).
2. DeepReason spec: C:/Users/user/Downloads/DeepReason-main/DeepReason-main/docs/harness-spec-v1.3.md §4 (two-pass labels, orphaned != false, N1 no-absorbing-status), §10 (case-law), §11.5 (atlas split: negative case law at the gate).
3. L-22 + L-23 files (the skeleton class + the explore loop seams); N-16 file (the evaluator-version edge shape — land ONE shape, see RECON-3); the J-96 minting filter (READ-ONLY, to confirm the boundary, never to edit).
4. F-0 W-1..W-10; G-0 GD-1/GD-2/GD-4/GD-5; L-0 EX-1..EX-4 + the new EX-5/EX-6; .claude/rules/proof-spine.md PSR-2; CLAUDE.md TD-3, ID-5, no-.unwrap-on-prod, blake3-only.

## Tasks (WAVE 2 — store, statuses, wires, riders; every move carries its observation + counter-move)

1. the conjecture store + node type (~120 LOC): crates/orchestrator/src/reason/conjecture_store.rs — ConjectureNode {skeleton (L-22 class), lane: Lane::Explore (the ONLY constructible lane for this type, EX-5), provenance (session/round/model-pin), criticisms: Vec<CriticismRef (SSB row id + evaluator-id + version-hash)>, missing_tests: Vec<DeclaredTest (the falsifier not yet run + what result would settle it)>}, plus a lane-TOKENED retrieval API (`retrieve(lane_token: ExploreLaneToken, ...)` — the code-fix path cannot even NAME the token type; not a runtime string check, a type-level wall).
EXPECTED_OBSERVATION: cargo check green; the negative contract test (a code-lane caller cannot obtain ExploreLaneToken outside reason/explore) compiles-fails when violated; unit test registers a node and reads it back only via the explore token.
ON_FAIL: if the type-level wall proves impossible without touching case-graph files (the cause: a shared retrieval trait forces a common surface) — counter-move: drop to a sealed module + private constructor (same guarantee, smaller blast); if THAT still requires editing J-96 files, ABORT-1.

2. the deterministic status fold (~90 LOC): status is a pure fold over journal rows, never a stored mutable field — pending (registered, no criticism executed yet), live (>= 1 executed criticism pass survived — a never-criticized node can NEVER be live, REDTEAM-3 patch), refuted (a compiled falsifier fired or an exogenous-anchored attack row landed), suspended (a premise's node is refuted -> dependents suspend, orphaned != false; OR a vouching evaluator was refuted -> N-16 S4 cascade suspends what it vouched for). No absorbing status (N1): a refuter row later refuted (its evaluator collapsed, its falsifier's checker bug-fixed) reinstates the target on the next fold. Quarantine (contradiction / SF-4 poisoning suspect) is a fold OUTPUT lane, excluded from default retrieval, NEVER a deletion.
EXPECTED_OBSERVATION: the fold, run twice over the same journal under ELAI_SEED, yields byte-identical status maps (GD-2); the fixture chain register -> criticize -> refute-premise produces dependents=suspended not refuted; refuting the refuter's evaluator reinstates.
ON_FAIL: if a status cannot be derived from journal rows alone (the cause: some criticism outcome was never journaled — a journal-completeness gap) — counter-move: add the missing SSB emission at the criticism site (that is the bug), never a hand-set status field; if hand-curation is the only path, ABORT-2.

3. the SSB lifecycle rows (~40 LOC): +ConjectureLifecycle flat action_type (Pattern-2; Kind: Registered/Criticized/Refuted/Suspended/Reinstated/Quarantined/Retrieved). baseline+1 from LIVE 581 variants / SCHEMA 272 (ssb.rs:1530-1544, 2026-07-11; re-read at edit, W-9). Retrieval itself is journaled (Kind: Retrieved carries the requesting lane + round id) so the EX-5 boundary is auditable off-agent, not just type-enforced.
EXPECTED_OBSERVATION: golden action-type lock regenerated; a fixture explore round leaves Registered + Retrieved rows; SCHEMA derives to 273.
ON_FAIL: if the action-type budget review prefers reuse (the cause: variant-growth discipline) — counter-move: reuse ExploreRoundCompleted with new Kind discriminators (net 0) and note it in the constants table; the Retrieved audit row is NON-NEGOTIABLE either way.

4. the explore-lane wires (~60 LOC): (a) L-23 pack assembly retrieves top-K live+pending conjectures for the current problem (deterministic match on skeleton scope/mechanism, never LLM-scored — TD-3), each rendered LABELED with its status + missing tests (an unproven idea enters the pack AS unproven); (b) the refuted set feeds the L-23 relapse gate AT THE GATE (I-8 hash + battery-equivalence), never rendered into the prompt (S3/EX-1); (c) suspended/quarantined are retrievable ONLY via an explicit debug read (--include-quarantined), never in the default pack.
EXPECTED_OBSERVATION: a fixture explore round's pack contains the live conjecture with its missing-tests line; a re-proposal of a refuted-equivalent is gate-blocked (no prompt render); the Retrieved SSB row names lane=explore.
ON_FAIL: if L-23 has not landed (FORK-1 route) — counter-move: ship the store + fold + a fixture-driven retrieval harness (contracts test drives the API); the pack wire is a named follow-on deliverable in L-23's execution, cross-linked here; the negative lane test runs REGARDLESS.

5. leg B — the typed unblock condition on refusals (~70 LOC): every abstention/refusal on the D-17/L-7 path (and GRC-v3 abstain steps — J-83, J CLOSED, cross-linked not amended) carries UnblockCondition {tried: [the checks/retrievals actually RUN, from the GRC receipt trail], missing: [the typed evidence that would change the answer — a checker id + input, a retrieval target, a human ruling], route: [which rung supplies it]}. COMPOSED DETERMINISTICALLY from the receipt trail (which verifications ran and failed / were unavailable) — never free-prose, never model-authored (the model may propose, the harness only records receipts it can cite; TD-3). This is the conjecture tier's missing_tests field applied to refusals: a refusal becomes a pending conjecture about its own answer. The D-17 SLA tick gains one clause: a refusal WITHOUT an unblock condition is an SLA violation (fail-closed on the emitter, not on the user).
EXPECTED_OBSERVATION: a fixture refusal lands an SSB refusal row carrying tried[]/missing[]/route; the SLA tick counts an unblock-less refusal as a violation; K-11 renders "what was tried + what would change the answer" for a non-author.
ON_FAIL: if the GRC receipt trail lacks the failed-check records to compose `tried` (the cause: receipts only journal successes) — counter-move: journal the failure/unavailable receipts at the check site (additive rows), then compose; if D-17's seam is unreachable (flag dead), wire leg B's emitter at the L-7 gate and register the D-17 tick clause as the follow-on, allowlist row per W-2.

6. leg A — the explore-bench novelty instrument (~50 LOC): the L-10 explore arm gains a novelty metric — NLI meaning-cluster novelty (L-4/F-12 deterministic-argmax entailment clustering; cluster survivor-map mechanisms by MEANING, count clusters + coverage; NEVER cosine-over-hash embeddings, his own instrument failed — steal §3.3) — reported under the CGG ON/OFF ablation (same runs L-10 already scores, one added column) plus retrieval-ON/OFF (the leg-CM ablation). Advisory number, never adjudicates (TD-3); measures "did the harness's structural moves recover latent diversity or generate genuinely-new clusters" — the collab's measure-not-settle question made a bench column.
EXPECTED_OBSERVATION: a fixture bench run emits the novelty column per arm in the L-10 report JSON; two runs under ELAI_SEED byte-agree (GD-2); the column is absent from no arm (missing = FAIL, R1).
ON_FAIL: if the F-12 NLI judge is not yet loadable in the bench harness (the cause: ONNX judge is its own unit) — counter-move: the instrument ships behind the L-10 bench's existing fixture-judge seam with the NLI swap as a named dependency; NEVER substitute an embedding metric (stub_ban).

7. leg D — the flywheel diversity guard, contract-first (~doctrine + 30 LOC schema): L-24 lands the BINDING admission contract on I-30/I-31 dataset rows (code lands inside I-30/I-31 when D-026 unlocks — no wire-dark machinery shipped ahead of its consumer): FD-1 train on MOVES not survivors — the exhaust builder must include criticism/refutation/abort moves (the conjecture tier preserves them as first-class rows), never winning trajectories only (survivor-bias collapse guard); FD-2 lambda-filtered labels — a training label is admitted only if its verdict window's exogenous-grounding ratio (N-16 S6) clears the floor (never distill rubric-consensus); FD-3 every dataset row carries the N-16 evaluator-id+version-hash edge, so an evaluator refutation quarantines the rows it vouched for (S4 collapse extends to training data). Ships as: the row-schema fields + FD-1..FD-3 recorded here + a feature-map clause on I-30/I-31 naming L-24 as their pre-gate; I-30's own done-gate consumes it.
EXPECTED_OBSERVATION: the I-30/I-31 feature-map entries carry the "guarded by L-24 FD-1..FD-3" clause; the row schema (fields only, no builder) compiles with the evaluator-edge field present.
ON_FAIL: if the schema fields cannot land without the builder (the cause: no crate owns the row type yet) — counter-move: FD-1..FD-3 land as contract text only (this file + feature map), zero code, and I-30 authors the type; the guard is then purely a pre-gate, which is the honest shape for a D-026-gated target.

8. workflow/reasoning/L-24-decisions.md — the decision doc (what shipped, the fold semantics chosen, the fork routes taken).
EXPECTED_OBSERVATION: file exists with the status-fold truth table + the lane-wall mechanism recorded.
ON_FAIL: missing doc — the auditor blocks COMPLETE (soft-enforced house rule); write it before AUDITING.

FORK-1: L-22/L-23 unexecuted at L-24's execution time (check: falsifier.rs / reason/explore.rs absent from the tree) -> take the Task-4 counter-move route (store + fold + fixture harness now; pack wire lands with L-23, cross-linked); do NOT block the whole file on the loop.
FORK-2: the SSB action-type review prefers variant reuse (check: the constants-table review at edit) -> Task-3 counter-move (ExploreRoundCompleted Kinds, net 0).
FORK-3: N-16's evaluator-id+version-hash field already landed when L-24 executes (check: rg "evaluator_version|evaluator_id" crates/orchestrator/src/ssb.rs) -> compose it verbatim; if NOT landed -> L-24 lands the SAME field shape N-16 names (one shape, cross-linked in both files, no fork of the schema — ND-1 unify-not-fork), and N-16's execution consumes it.

RECON_NEEDED-1: re-grep every PLAN-asserted anchor at edit (W-7): falsifier.rs, reason/explore.rs, anti_spiral_extended.rs, the D-17 ELAI_REFUSAL_SLA_TICK_ENABLE seam, the F-12 NLI judge symbol, ssb.rs SCHEMA baseline (272/581 goes stale — `rg "SSB_ACTION_TYPES.len" .elai_cc/crates/orchestrator/src/ssb.rs` + read the trailing comment).
RECON_NEEDED-2: confirm the J-96 boundary by grep before AND after: `rg -l "ConjectureNode|ExploreLaneToken" .elai_cc/crates/orchestrator/src/ | rg -v "reason/"` MUST return empty (the type never leaks outside the reason module; the case-graph files are untouched by `git diff --stat`).
RECON_NEEDED-3: whether GRC receipts journal FAILED/unavailable checks today (leg B's `tried` source): `rg "ReceiptOutcome|receipt.*fail" .elai_cc/crates/orchestrator/src/` — if absent, Task-5's counter-move (additive failure receipts) is in scope, size it before starting.
RECON_NEEDED-4: the L-10 bench report shape (where leg A's column lands): read the L-10 file's report schema section at edit; if L-10 is unauthored-in-code, the column spec lives in this file and L-10's execution consumes it.

ABORT-1: any change to this file's scope that requires EDITING J-96/J-97/J-98 files or the F-4 minting filter -> STOP and surface (the proven-only domain split is a locked user decision; no self-granted exception).
ABORT-2: statuses cannot be computed as a deterministic journal fold (hand-curation would be required) -> STOP and escalate; computed-not-curated is load-bearing (EX-6), a curated status store is a DIFFERENT design the user must approve.
ABORT-3: the same leg fails 3 attempts / the auditor BLOCKs twice -> anti-spiral protocol, bubble up; never widen scope to "fix" a blocked leg by touching sibling subsystems.

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/reason/conjecture_store.rs — ConjectureNode + ExploreLaneToken + the lane-tokened retrieval API + the deterministic status fold (pending/live/refuted/suspended + quarantine lane)
  - the +ConjectureLifecycle SSB action_type (or ExploreRoundCompleted Kinds per FORK-2) incl. the Retrieved audit row
  - the UnblockCondition typed field on D-17/L-7 refusal rows + the SLA violation clause (leg B)
  - the L-10 explore-arm NLI meaning-cluster novelty column under CGG ON/OFF x retrieval ON/OFF (leg A)
  - the FD-1..FD-3 dataset-row admission contract + row-schema fields for I-30/I-31 (leg D, contract-first)
consumes_symbols:
  - L-22 falsifier skeleton class (the conjecture object; missing_tests = unrun falsifiers) — never a second skeleton type
  - L-23 explore.rs pack assembly + relapse gate (the production retrieval consumer; refuted set feeds the GATE, never the prompt — S3/EX-1)
  - N-16 evaluator-id+version-hash edge + S4 quarantine cascade + S6 lambda (FD-2) — ONE field shape, compose never re-implement
  - F-5 staleness/quarantine shape; K-19 SF-1..SF-4 (binding: observe-first, never-into-auditor, never-DELETE, poisoning quarantine)
  - L-4/F-12 NLI meaning-cluster judge (leg A's engine); L-10 bench harness (the report sink)
  - D-17 refusal SLA tick + L-7 gate (leg B's carrier); the GRC receipt trail (the `tried` source)
  - SSB emit + RRA replay; L-16 pure-fold pattern (status = view, never state)
  - J-96..J-98 verified case graph: consumed as a BOUNDARY only (read to confirm untouched) — zero symbols shared, zero files edited
reachable_from:
  - `elai reason --profile explore "<question>"` > L-23 pack assembly > conjecture_store.retrieve(ExploreLaneToken, ..) [PRODUCTION reasoning surface]
  - a D-17/L-7 refusal on any lane > UnblockCondition composed from receipts > SSB refusal row + K-11 render [PRODUCTION refusal path]
  - `elai bench hallucination-delta` explore arm (L-10) > the novelty column [PRODUCTION bench entrypoint]
  - a #[cfg(test)] / contracts/ caller NEVER satisfies this (R7)
stub_ban:
  - NO retrieval of any ConjectureNode into a code-fix / auditor / case-graph pack (EX-5; SF-3 never-into-auditor) — the negative test is mandatory
  - NO stored mutable status field, NO hand-curated status, NO deletion of a refuted/quarantined node (EX-6; SF-4; append-only)
  - NO LLM-scored retrieval ranking, NO model-authored status or unblock condition (TD-3 — the model proposes text, the harness computes labels off receipts)
  - NO refuted/banned conjecture content rendered into a prompt (S3/EX-1 — the tabu lives at the gate); NO embedding-cosine novelty metric (steal §3.3 — NLI meaning-cluster only)
  - NO Dung argument-graph adjudicator (REJECTED steal §4.3 — the fold is over journal rows, not attack semantics); NO edits to J-96..J-98 / F-4 minting (ABORT-1)
  - NO .unwrap()/.expect() on prod paths; blake3 not sha2; no new dependency
frontend_pair:
  the conjecture map on the explore/canvas surface (nodes colored by computed status + missing-tests tooltip + quarantine count) rides L-23's survivor-map render (K-11/N-7) — one added status dimension, not a new surface; cross_lang_diff if a Tauri read is added.
observable_effect:
  trigger: "elai reason --profile explore on a fixture problem across TWO rounds: round 1 registers conjectures A (premise P) and B; between rounds a falsifier refutes P; round 2 retrieves"
  expect_ssb: "+ConjectureLifecycle rows: Registered(A,B,P), Criticized, Refuted(P), Suspended(A — orphaned != false), Retrieved(lane=explore, round 2 pack holds B live + A EXCLUDED from default retrieval); baseline+1 from LIVE 581/272 (ssb.rs:1530-1544; re-read at edit, W-9)"
  expect_artifact: "round-2 pack renders B labeled live WITH its missing tests; the status map recomputes byte-identically from the journal under ELAI_SEED (GD-2). NEGATIVE (must FAIL): a code-lane retrieval returning ANY ConjectureNode; a never-criticized node reporting live; a hand-set status surviving the fold; a refusal row missing tried[]/missing[] on the D-17 path"

INTEGRATION_SMOKE (R10 — after Tasks 1-5, before DONE): an INDEPENDENT agent drives `elai reason --profile explore` on a HARD fixture (multi-round, >=3 rival mechanisms, one premise-refutation mid-run, one forced abstention) and asserts off-agent: the lifecycle SSB chain lands in order; the suspended dependent is excluded from the default pack but present under --include-quarantined=false debug read; the refusal carries a typed unblock condition whose `missing` names a runnable check; re-running the fold over the journal reproduces the exact status map (GD-2); AND the cross-lane negative — a code-lane task run in the same session journals ZERO Retrieved rows with lane != explore. Toy single-round happy-path does not count.

VERIFY (the executor's runs, each pass criterion an OFF-AGENT artifact — an SSB row, a frozen checker's exit code, a contract-test result the auditor re-runs; never the executor's own stdout):
- after Tasks 1-3: `cargo nextest run -p elai-contracts l24_conjecture_store_contract` — PASS = the (a)-(i) assertions green, exit 0 (the auditor re-runs off-agent per R8; the executor's run is a courtesy).
- after Task 4: the two-round fixture trigger from observable_effect — PASS = the +ConjectureLifecycle SSB chain lands in the journal in order (queried off-agent from the db, not read from stdout).
- after Task 5: the fixture refusal — PASS = the refusal SSB row carries tried[]/missing[]/route and the SLA tick registers the unblock-less negative as a violation row.
- after Task 6: `elai bench hallucination-delta` fixture arm twice under ELAI_SEED — PASS = the novelty column present in both report JSONs and byte-identical (GD-2).
- before DONE: the INTEGRATION_SMOKE below, driven by an INDEPENDENT agent (R10) + the DETERMINISTIC_CHECK block, all exit 0; any missing artifact = FAIL (R1), never SKIP.

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/L-plan/L-24_conjecture_memory_middle_tier.md --run-trigger
rg "ExploreLaneToken" .elai_cc/crates/orchestrator/src/reason/conjecture_store.rs        # the type-level lane wall exists
rg -c "status\s*:\s*ConjectureStatus" .elai_cc/crates/orchestrator/src/reason/conjecture_store.rs   # MUST be 0 as a stored struct field (status is a fold output, EX-6)
rg -l "ConjectureNode|ExploreLaneToken" .elai_cc/crates/orchestrator/src/ | rg -v "reason/"          # MUST be empty (EX-5 — never leaks outside the reason module)
git diff --stat -- .elai_cc/crates/orchestrator/src/*case*                                # MUST be empty (J-96..J-98 untouched)
rg -ci "cosine|embedding" <the leg-A novelty instrument file>                             # MUST be 0 (NLI meaning-cluster only)
rg "UnblockCondition" .elai_cc/crates/orchestrator/src/                                   # leg B's field present on the refusal path
test -f workflow/reasoning/L-24-decisions.md
```

## ACCEPTANCE
- [ ] EX-5: an unproven conjecture is structurally unretrievable into the code-fix path (type-level lane token + the empty-leak grep + the cross-lane negative in the smoke).
- [ ] EX-6: every status is a deterministic journal-fold output (no stored status field; byte-identical recompute under ELAI_SEED); refute-premise -> dependents SUSPENDED not refuted; no absorbing status (refuter falls -> reinstate); quarantine never deletes.
- [ ] N-16 composition: criticism/verdict rows carry the ONE shared evaluator-id+version-hash shape; an evaluator refutation cascades into conjecture suspension (S4), and (leg D) into dataset-row quarantine (FD-3).
- [ ] leg B: UnblockCondition composed only from receipts (tried/missing/route typed); unblock-less refusal = SLA violation; rendered via K-11.
- [ ] leg A: the novelty column is NLI meaning-cluster (zero cosine/embedding), reported under CGG ON/OFF x retrieval ON/OFF, advisory-only (TD-3).
- [ ] leg D: FD-1..FD-3 recorded as the I-30/I-31 admission pre-gate (feature-map clause landed); no exhaust-builder code shipped ahead of D-026.
- [ ] SF-1..SF-4 (K-19) hold: observe-first on any new floor, never-into-auditor, never-DELETE, poisoning quarantine.
- [ ] cargo check --workspace GREEN; auditor PASS on -p orchestrator (+ cli if the bench verb is touched). No new dependency.

## WIRING DONE-GATE (clause by clause)
- W-1: done = the two-round explore trigger leaves the lifecycle SSB chain + the labeled pack + the recomputable status map (not cargo-green).
- W-2: no ELAI_*_ENABLE on the store (opt-in by invocation of the explore profile); leg B's D-17 tick clause rides the EXISTING dark flag's allowlist row — no new default-OFF flag.
- W-3: BUILD leg (the store/fold) + EXTENSION legs on L-10/D-17/I-30 contracts; composes, restates nothing.
- W-4: status dimension on the L-23 survivor-map render; cross_lang_diff if a Tauri read is added.
- W-5/W-10: all in-process; the smoke is the live end-to-end trigger; no network leg.
- W-6: the rendered conjecture map verified through the surface (dashboard-smoker) or justified N/A at edit.
- W-7: every PLAN-asserted anchor re-grepped at edit (RECON_NEEDED-1..4).
- W-9: SSB baseline re-read live at edit; golden action-type lock regenerated.
- GD-2: statuses, retrieval rows, and the novelty column are journal-recomputable under ELAI_SEED (RRA byte-replay).
- GD-4: any new floor (lambda floor FD-2, retrieval top-K) ships observe-first, measured before it acts.
- GD-5 honest residue: the tier preserves and labels conjectures; it does NOT make them good — a wrong-but-stable conjecture that never meets its missing test stays live-labeled-unproven forever; the only lever is running the declared tests (exogenous anchors). The lane wall is type+audit-enforced inside the orchestrator; an out-of-process reader of the raw journal is not stopped by it (the journal is append-only truth; the WALL is on the retrieval surfaces).
- ID-5: every DeepReason figure is design-rationale; L-10 measures ELAI's own retrieval-ON/OFF and CGG-ON/OFF deltas.

REDTEAM (attacks run against this plan before authoring-done; posture per plan-standard):
- ATTACK-1 (flood/poison the explore memory: register junk conjectures until retrieval drowns the pack) — FAILED against the plan: registration requires the L-22 skeleton with declared falsifiers (forbid-nothing = demarcation-refused), the anti-relapse gate blocks refuted-equivalents, retrieval is budgeted top-K deterministic-match, and SF-4 poisoning quarantine applies. Residue: a slow drip of well-formed junk is not detected by structure — it surfaces as criticism-debt (N-16 S7) and stays pending, never live (needs an executed criticism to promote).
- ATTACK-2 (smuggle an unproven conjecture into the code-fix path via the shared journal — a code-lane reader picks conjecture rows out of SSB) — SUCCEEDED against draft 1 -> PATCH: the type-level ExploreLaneToken wall + the Retrieved audit row journaling the requesting lane + the empty-leak grep in DETERMINISTIC_CHECK + the cross-lane negative in the INTEGRATION_SMOKE. The journal itself stays readable (append-only truth); the RETRIEVAL surfaces are the wall (GD-5 residue noted).
- ATTACK-3 (status laundering: a conjecture becomes "live" by never being criticized) — SUCCEEDED against draft 1 -> PATCH: live REQUIRES >= 1 executed criticism pass; registered-never-criticized stays pending forever, its missing tests render as evidence gaps in the survivor map, and the pending-forever population feeds N-16 S7 criticism-debt.
- ATTACK-4 (evaluator-refutation cascade as a memory-wipe weapon: refute one evaluator, watch the tier empty) — FAILED against the plan: collapse = suspend/quarantine, never delete (append-only, SF-4); reinstatement is automatic when the refuter itself falls (N1); the cascade only moves labels, rows are immutable.
- ATTACK-5 (leg B free-prose laundering: the model writes a plausible unblock condition that cites checks never run) — FAILED against the plan: UnblockCondition is composed by the harness from journaled receipts only; a `tried` entry without a matching receipt row cannot be constructed (the composer reads the journal, not the model's text).

## Tests to add
- crates/contracts/tests/l24_conjecture_store_contract.rs (~130 LOC): (a) register->criticize->refute-premise => dependents suspended (not refuted); (b) refuter's evaluator refuted => target reinstates on next fold (N1); (c) never-criticized node is pending, never live (ATTACK-3); (d) fold recomputes byte-identically under ELAI_SEED (GD-2); (e) NEGATIVE: no code path outside reason/ can construct ExploreLaneToken (compile-fail or visibility test) and a lane!=explore retrieval returns zero nodes + journals nothing (ATTACK-2/EX-5); (f) quarantined node absent from default retrieval, present under the debug read, never deleted; (g) refusal row carries tried/missing/route composed from receipts, and a fabricated tried-entry without a receipt row is unconstructible (ATTACK-5); (h) the novelty column asserts zero embedding/cosine symbols (leg A); (i) corrupt journal row => fold fails closed (exit/err, never a default status).

## Files touched
- CREATE: .elai_cc/crates/orchestrator/src/reason/conjecture_store.rs (~250 LOC: node + token + fold + retrieval)
- MODIFY: .elai_cc/crates/orchestrator/src/ssb.rs (~15 LOC: +ConjectureLifecycle variant; or FORK-2 net-0 Kinds)
- MODIFY: the L-23 pack assembly + relapse-gate seam (~30 LOC — or the FORK-1 fixture harness if L-23 unlanded)
- MODIFY: the D-17/L-7 refusal emission path (~70 LOC: UnblockCondition composer + SLA clause)
- MODIFY: the L-10 bench explore arm (~50 LOC: the novelty column; or the column spec lands here per RECON-4)
- CREATE: crates/contracts/tests/l24_conjecture_store_contract.rs (~130 LOC)
- CREATE: workflow/reasoning/L-24-decisions.md
- (feature map + indexes per the authoring session, not the executor)

## Citations / GROUNDING
- workflow/handoff/deepreason-steal-and-collab.md CURRENT STATUS (the four collab improvements (a)-(d); the locked decisions: middle tier adopted for the explore lane, proven-only code path unchanged) — the design source is the LIVE collab exchange with the DeepReason author, 2026-07-11.
- DeepReason harness-spec-v1.3.md (local: C:/Users/user/Downloads/DeepReason-main/DeepReason-main): §4 two-pass labels (accepted/refuted/suspended_unsupported; "Refuting a premise => dependents become suspended_unsupported, NOT refuted (orphaned != false)"; N1 no-absorbing-status), §10 case-law, §11.5 atlas split (negative case law at the gate, never in packs) [EXTERNAL / DESIGN-RATIONALE, ID-5 — prior art credited to AHepi/DeepReason].
- workflow/reasoning/steal-deepreason-2026-07-08.md §1 (status model), §4.2 S4/S6 (closure + lambda, landed as N-16 — composed here), §4.3 REJECTS (no Dung adjudicator), §3.3 (embedding novelty metric fails; NLI meaning-cluster replaces it).
- ELAI ground truth: ssb.rs:1530-1544 (LIVE 581 variants / SCHEMA 272 derived, read 2026-07-11 — re-read at edit); feature map 1732-1809 (J-96..J-98 VCG + K-19 SF-1..SF-4 binding memory-reuse work) + 469-470 (D-17 DARK seam) + 1711-1727 (I-30/I-31, D-026-gated); N-16 file (the evaluator-version edge + S4/S6/S7); L-22/L-23 files (PLAN-asserted seams, re-grep W-7).
- CLAUDE.md: TD-3, ID-5, GD-1/GD-2/GD-4/GD-5, no .unwrap on prod, blake3-only; .claude/rules/proof-spine.md PSR-2 (leg D none, user-acked 2026-07-11).

## Open questions / risks
- The `live` threshold (>= 1 executed criticism pass) is a seed choice: too strict and the pack starves early rounds (pending-heavy), too loose re-opens ATTACK-3. Seed at 1, observe-first (GD-4), let L-10's retrieval-ON/OFF arm judge; decide a promotion floor only on measured data.
- The retrieval top-K and the deterministic match keys (skeleton scope/mechanism) are unproven ranking heuristics — advisory retrieval, never adjudication (TD-3); if match quality is poor the fallback is BM25 over skeleton text (existing 1B machinery), decide at executing.
- Leg B's receipt coverage (RECON-3): if failure receipts are sparse today, `tried` starts thin — honest-thin beats fabricated-rich; the additive failure-receipt rows are the fix, sized at edit.
- The suspended-reinstatement fold cost over a long journal: the fold is a view — if recompute cost grows, materialize with the L-16 enumerated-recompute-causes pattern, never a mutable store (EX-6).

## Next
L-24 gives the explore lane a memory that can hold ideas ahead of the receipt plane — labeled, criticized, never laundered into the proven path — and pays the collab back with the three riders: the measure-not-settle novelty column (leg A feeds L-10/PS-3 directly), refusals that state their own unblock condition (leg B — every abstention becomes a pending conjecture about its own answer), and a flywheel that trains on moves, not survivors (leg D, contract-first for I-30/I-31). It pairs with L-22/L-23 (the skeleton + the loop), composes N-16 (the closure machinery), and leaves J-96..J-98 exactly as strict as the user locked them.

End of L-24.
