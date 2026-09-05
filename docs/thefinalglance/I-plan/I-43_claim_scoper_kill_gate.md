# I-43: reproducibility-claim scoper + cheap pre-falsification kill-gate — falsify the trust thesis before the expensive build

- Track 19 — DESIGN-PHASE CRITIQUE + TRUST-CLAIM DISCIPLINE | lever PROVABLE / FALSIFY-FIRST | grounding STRONG | nature BUILD | effort M | WAVE 1 | binds ID-5 | composes I-38/G-6/H-23
- Premise: grounded-by-default forbids pouring effort into the heavy SAFEDELTA build (G-6, XL, RunPod) while the
  core number is unproven. The source plan (ELAI-IMPROVEMENT-PLAN.md §2 Phase-0) prescribes CHEAP falsifiers run
  FIRST: F0.1 a reduced 50-100-task gate-ON-vs-OFF pilot looking only for the SIGN + rough magnitude of the
  residual-insecure-rate delta (kill if flat/negative or if the gate "wins" only by collapsing the
  reviewer-reaching rate), and F0.2 a 20-task local-Qwen-vs-cloud-Claude action-stream divergence test to decide
  the reproducibility CLAIM WORDING before marketing it (cloud = "trace authentication + signed evidence", NOT
  "deterministic replay"; full recompute only in an open-weight mode). ELAI has the FULL instruments (I-38 QDELTA
  harness, G-6 SAFEDELTA, H-23 same-seed re-run) but NOT the cheap reduced-N go/no-go nor the cross-lane
  divergence scoper. This session builds the falsifier harness so the keystone build is gated on a green, not faith.

## Steal/build
Two instruments behind one `elai bench claim-scope` subcommand:
(a) PILOT (F0.1): run the SHIPPED seccodeplt/safedelta runner at reduced N (default 50-100) gate-ON vs gate-OFF
on git-stripped tasks with dynamic-exploit scoring, reporting (i) the residual-insecure-rate delta sign +
rough magnitude AND (ii) the reviewer-reaching rate under gate-ON (the block-everything trap detector — a gate
that wins only by blocking nearly everything FAILS the pilot). This is QDELTA (I-38) at small N framed as a
GO/NO-GO, not a headline — the output is a go/no-go memo artifact, never an external claim.
(b) DIVERGENCE SCOPER (F0.2): run ~20 identical tasks on the local lane (Qwen2.5-Coder) vs the cloud lane
(Claude) under fixed seed and measure ACTION-STREAM divergence on security-critical paths. >5% action-stream
divergence on those paths means the word "reproducible" is unusable on the cloud path -> the harness EMITS the
claim wording it can honestly support (cloud = trace-authentication; reserve recompute for open-weight mode).
This is distinct from H-23 (same-env/same-seed re-run hash equality) — I-43 measures CROSS-LANE divergence to
scope the CLAIM, not to assert determinism. Every number recomputes from the SSB journal under ELAI_SEED (GD-2);
no paper constant is a threshold (ID-5 — the delta sign and the divergence % are ELAI-measured). Declined: a
single big-N run (the whole point is cheap-first); claiming "deterministic replay" on the cloud path (GD-5 — the
divergence scoper exists precisely to forbid that overclaim).

## Existing ground truth
- I-38 QDELTA harness (QdeltaMeasured, bootstrap CI, gates-ON-vs-OFF, the statistical floor): the full
  instrument I-43 runs at reduced N for the pilot (compose, do NOT re-implement — GD-1; PLAN-asserted, re-grep W-7).
- G-6 SAFEDELTA + the SHIPPED seccodeplt_runner / safedelta_runner (F-2b): the gate-ON/OFF execution path +
  dynamic-exploit scoring + git-strip + the Endor cheat-detector (compose; PLAN-asserted, re-grep W-7).
- H-23 same-seed re-run hash-equality gate: the determinism centerpiece — I-43's divergence scoper is the
  CROSS-LANE sibling (different question: claim-scoping, not assertion); composes the RRA/replay surface.
- G-2 RRA (rra.rs build_rra:384, --emit-rra, replay): SHIPPED per H-23 — the action-stream capture the
  divergence scoper diffs across lanes (GD-1; PLAN-asserted, re-grep W-7).
- B-37 PerRoleRuntime + the local/cloud lanes: SHIPPED — the two lanes the divergence test drives.
- No reduced-N go/no-go pilot and no cross-lane divergence scoper exist today (IN — net-new).

## REVISED scope (W-10 tiered)
SHIP NOW (Tier-1, deterministic fixture harness):
- crates/cli/src/bench_claim_scope.rs: `elai bench claim-scope --suite <fixture> --n 50` (pilot) and
  `--mode divergence` (the cross-lane scoper)
- the pilot reduced-N driver over the SHIPPED seccodeplt/safedelta runner (gate-ON vs OFF + reviewer-reaching
  rate + block-everything-trap detector)
- the divergence scoper: action-stream diff of local-vs-cloud RRA traces on security-critical paths -> emits the
  honest claim wording
- a go/no-go memo writer: workflow/reasoning/I-43-phase0-memo.md (three greens or a re-scope)
- ClaimScopeMeasured SSB variant (PilotDelta / CrossLaneDivergence / ClaimScoped arms)
- a deterministic fixture harness (mock runner + canned RRA traces) exercising the REAL code path with no GPU/cloud

DEFER (Tier-2 / gated, W-10):
- the LIVE pilot run (real SeCodePLT/BaxBench, dynamic exploit, RunPod) — a separately-gated checkbox
- the LIVE cross-lane run (real cloud Claude + real local Qwen) — gated; the fixture harness proves the path now
- channel willingness-to-pay (F0.3 discovery calls) — BUSINESS validation, NOT an engineering session (out of
  I-plan scope; recorded in the memo template as the third Phase-0 green, run off-tool)

## Integration
- CREATE: crates/cli/src/bench_claim_scope.rs (~200 LOC: pilot driver + divergence scoper + memo writer)
- seam to I-38 QDELTA harness (the reduced-N reuse; GD-1) + the seccodeplt/safedelta runner (PLAN-asserted, re-grep W-7)
- seam to G-2 rra.rs (the action-stream capture diffed across lanes; GD-1)
- seam to B-37 PerRoleRuntime (drive the local vs cloud lane)
- CREATE: crates/contracts/tests/i43_claim_scope_fixture_contract.rs (~70 LOC: mock runner, the trap detector,
  the divergence threshold)
- CREATE: workflow/reasoning/I-43-decisions.md + the memo template workflow/reasoning/I-43-phase0-memo.md
- frontend_pair: ClaimScopeMeasured rows render in the cost/metrics cockpit (H-9 tile family) as a Phase-0
  go/no-go panel (PLAN-asserted; re-grep W-7) — or W-6 justified absence (bench TOML + memo are the primary
  artifacts; a cockpit tile is the optional surface).

## WIRING_CONTRACT
produces_symbols:
  - crates/cli/src/bench_claim_scope.rs::run (the `elai bench claim-scope` entrypoint)
  - the reduced-N pilot driver + the block-everything-trap detector (reviewer-reaching rate guard)
  - the cross-lane action-stream divergence scoper + the honest-claim-wording emitter
  - ClaimScopeMeasured (new SSB action_type; arms PilotDelta/CrossLaneDivergence/ClaimScoped) — baseline+1 from
    LIVE 221/532 as of 2026-06-10 (ssb.rs:2981 / action_type_lock.rs:601); re-read at edit (W-9)
  - workflow/reasoning/I-43-phase0-memo.md (the go/no-go memo artifact)
consumes_symbols:
  - I-38 QDELTA harness (why: the full ablation instrument run at reduced N; compose, do NOT re-implement)
  - the SHIPPED seccodeplt/safedelta runner (why: the gate-ON/OFF + dynamic-exploit + git-strip execution path)
  - G-2 rra.rs build_rra (why: the action-stream traces the divergence scoper diffs across lanes)
  - H-23 determinism/DriftComparator shape (why: the first-divergence-index reporting pattern; mirror, restate none)
  - B-37 PerRoleRuntime (why: drive the local vs cloud lane for the divergence test)
reachable_from:
  - `elai bench claim-scope --suite <fixture> --n 50` CLI -> bench_claim_scope::run -> ClaimScopeMeasured rows  [PRODUCTION bench surface]
  - `elai bench claim-scope --mode divergence` CLI -> the cross-lane scoper -> ClaimScopeMeasured(CrossLaneDivergence)  [PRODUCTION bench surface]
  - a #[cfg(test)] / contracts/ caller NEVER satisfies this (R7)
stub_ban:
  - NO single big-N run masquerading as the pilot (the pilot is cheap-first reduced-N by construction)
  - NO "deterministic replay" claim on the cloud path emitted by the scoper (GD-5 — the scoper forbids the overclaim)
  - NO paper constant as the pilot threshold (ID-5 — the delta sign + divergence % are ELAI-measured)
  - NO re-implementation of I-38/G-6/H-23 (compose the shipped instruments — GD-1)
  - NO fabricated pilot numbers; the LIVE leg is a separately-gated checkbox, the fixture harness exercises the real path (W-10)
  - NO .unwrap()/.expect() on production paths; NO blake2/sha2 (blake3 only)
frontend_pair:
  ClaimScopeMeasured rows render a Phase-0 go/no-go panel in the H-9 cost cockpit family (PLAN-asserted; re-grep
  W-7). Justified: the bench TOML + the go/no-go memo are the load-bearing artifacts; the cockpit tile is the
  optional human surface (W-6 satisfied by the memo if the tile is deferred).
observable_effect:
  trigger: "elai bench claim-scope --suite <fixture> --n 50  (Tier-1 deterministic fixture; LIVE run = gated checkbox)"
  expect_ssb: "ClaimScopeMeasured(PilotDelta) carrying delta-sign + magnitude + the reviewer-reaching rate;
    ClaimScopeMeasured(CrossLaneDivergence) carrying the action-stream divergence % on security paths;
    ClaimScopeMeasured(ClaimScoped) carrying the honest claim wording the divergence supports"
  expect_artifact: "workflow/reasoning/I-43-phase0-memo.md written with the go/no-go verdict; every number
    recomputes byte-identically from the journal under ELAI_SEED (GD-2); a fixture where gate-ON blocks nearly
    everything trips the block-everything-trap detector (the pilot FAILS — the trap is caught, not hidden) — R2;
    a fixture with >5% cross-lane divergence emits the trace-authentication-only wording, never replay (GD-5)"

## Grounding
- Source plan: ELAI-IMPROVEMENT-PLAN.md §2 (F0.1 pilot + the block-everything trap; F0.2 divergence + claim
  wording), §7 (R-1/R-2 risks + the cheapest falsifier each), §8 (claim discipline — the exact words), §10
  (first concrete move: F0.1 on the shipped seccodeplt path).
- R3 measurement report grounding (SUSVIBES 61% correct / 10.5% secure -> ~50pt gap; the only-credible-metric +
  block-everything trap; power: <2-3pt on 200 tasks is dead). WebSearch-confirm any external benchmark number before headline.
- Composition substrate (PLAN-asserted, re-grep W-7): I-38 QDELTA, G-6 SAFEDELTA + seccodeplt/safedelta runner,
  G-2 rra.rs, H-23 DriftComparator, B-37 lanes.
- Doctrine: binds ID-5 (no paper constant is a threshold — the pilot measures ELAI's own delta) + GD-5 (the
  divergence scoper forbids the cloud-path replay overclaim) + GD-2 (every number journal-recomputable).
- SSB: ClaimScopeMeasured +1 variant — baseline+1 from LIVE 221/532 as of 2026-06-10; re-read at edit (W-9).

## COMPOSES-WITH
- I-38 QDELTA + the statistical floor — the full instrument; I-43 runs it at reduced N as a go/no-go. Do NOT re-implement.
- G-6 SAFEDELTA + the seccodeplt/safedelta runner — the gate-ON/OFF execution path. Compose.
- H-23 same-seed re-run — the determinism sibling; I-43 measures CROSS-LANE divergence (different question).
- G-2 RRA — the action-stream capture diffed across lanes. Do NOT re-implement the RRA format.
- I-40 trust-layer wrapper — consumes the honest claim wording I-43 scopes (the wrapper must not advertise replay
  on the cloud lane; I-43 produces the words it may use). Restates none.

End of I-43.
