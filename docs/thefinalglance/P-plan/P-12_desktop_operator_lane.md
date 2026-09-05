# P-12: desktop/GUI operator lane — GUI actuation under ELAI's fail-closed gates

STATUS: PLANNED (WAVE CU-2; NEW capability — no desktop/GUI computer-use lane exists in B..Q; PROOF_SPINE: none (product), owner-acked 2026-07-11; execute after the proof spine per A-7)

- Track COMPUTER-USE-STEAL | lever gui-actuation-under-gates | grounding MODERATE (comparators source-verified in the eval doc; ELAI anchors PLAN-asserted, executor re-greps W-7) | nature NEW | effort L | WAVE CU-2 | depends P-13 | PROOF_SPINE none (product)
- Premise: ELAI specs browser E2E (E-14) + media validation (4I) but has NO general desktop/GUI computer-use stage. A workflow node "drive this GUI app / test this game build" has no home. This adds one, with the six comparators as vendored substrate + operator lanes, never rivals (the I-40 `elai wrap` inversion applied to computer use).

## Context — why this matters
Grounded in workflow/reasoning/canvas-computer-use-eval-2026-07-11.md ENTRY 7/8/10/11/12. agent-s (arXiv:2410.08164/2504.00906/2510.02250, OSWorld 66%->72.6% BoN, past the human baseline) proves the capability ceiling; cua (ENTRY 10) ships the sandbox substrate (Windows-Sandbox/lume/Docker, MIT) but its in-loop safety gating is a commented-out TODO (agent.py:808/839); OmniParser (ENTRY 6, arXiv:2408.00203) is the pixel-perception fallback for canvas/game surfaces. openclaw's computer-tool frame-binding (comparison V4, computer-tool.ts:488-551) is the best-engineered actuator-safety mechanism of the whole set. The division of labor: they bring VMs + capability, ELAI brings the fail-closed gate + receipts none of them have.

## Existing ground truth (B..Q coverage; PLAN-asserted; executor re-greps W-7)
- E-14 Browser E2E (E-SUMMARY_INDEX.md:134): AXTree-first Playwright + axe-core + VLM Tier-2 + SSIM diff. The browser sibling; the desktop lane mirrors its doctrine.
- J-1/J-8 perception gateway (J-SUMMARY_INDEX.md:105,260): deterministic image-ingestion, counts-from-AXTree-never-VLM. P-13 extends it; this lane consumes it.
- G-1 taint (harness-wide propagation), G-2 RRA (action stream + replay). An operator action is a new actuation source + a new RRA-recorded action.
- 4F feasibility probe (isolated sandbox with branch-and-explore) — the isolation substrate seam cua vendors into.
- 4I media validator (render-capture-evaluate, Tier-1 veto + Tier-2 advisory VLM) — the same evaluate-loop doctrine, reused for the game/GUI capture.

## Steal/build (REVISED scope, W-10 tiered)
- SHIP NOW (Tier-1): an OperatorLane crate wrapping (a) a vendored cua sandbox adapter (disposable VM/container, MIT), (b) the openclaw-style frame-binding — a coordinate action is authorized only while the exact sha256-hashed image the model saw is still in context; epoch-invalidated when compaction drops the screenshot; fail-closed on no-frame / node-switch / geometry-change; NEVER retargets to display 0; (c) a two-phase arm-commit (journal the capability grant BEFORE exposing it, so a crash mid-arm leaves no grant), (d) dual lease expiry (TTL checked on a periodic sweep AND inline at dispatch), (e) a per-action bounded screen-recording whose hash is written to an off-agent SSB row.
- DEFER (Tier-2): the OmniParser Tier-3 pixel adapter (lands with P-13); the full game-harness capture loop over 4I; multi-monitor.
- EXCLUDED as vaporware: cua's own in-loop gating (a stubbed TODO — ELAI supplies the gate, does not port their absent one); any ungated self-extension (agent-zero apt-get pole).

## Reads (mandatory before coding)
- workflow/reasoning/canvas-computer-use-eval-2026-07-11.md (ENTRY 7/8/10/11/12 + SYNTHESIS 2a/P4)
- docs/thefinalglance/E-plan/E-14_*.md (browser lane doctrine to mirror), docs/thefinalglance/J-plan/J-1_*.md (perception seam)
- .claude/rules/wiring-enforcement.md (R10 integration gate), .claude/rules/production-caller-gate.md (D-030)
- crates/orchestrator/src/ (the FSM dispatch seam an operator TaskSpec stage plugs into)

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/operator/mod.rs (OperatorLane: sandbox adapter + frame-binder + arm/lease + recorder)
  - crates/orchestrator/src/operator/frame_bind.rs (FrameBinding: sha256(image-bytes) -> authorized-action guard)
consumes_symbols:
  - crates/orchestrator/src/perception/mod.rs::PerceptionScaffold (P-13 element feed; do NOT re-implement)
  - crates/orchestrator/src/taint.rs::TaintSource (G-1; an operator action is a taint source)
  - crates/orchestrator/src/rra.rs::ActionStream (G-2; the action is RRA-recorded)
reachable_from:
  - crates/orchestrator/src/task_runner.rs (the O-2 tick dispatching an operator_action TaskSpec stage) [PRODUCTION task_runner]
  - crates/cli/src/main.rs (`elai operator run`) [PRODUCTION CLI]
  - a #[cfg(test)] or contracts/ caller NEVER satisfies this (R7)
stub_ban:
  - NO actuation on a stale/compacted frame_hash (fail-closed, never retarget to display 0)
  - NO capability exposed before its arm-grant row lands (two-phase commit)
  - NO .unwrap()/.expect() on the sandbox/actuation path; NO blake2/sha2 (blake3 for the frame hash)
  - NO operator action treated as trusted (it is a G-1 taint source and an operator lane, never load-bearing infra)
frontend_pair:
  operator-action rows + the screen-recording thumbnail in an existing dashboard drawer (decide exact drawer at edit, W-4).
observable_effect:
  trigger: "elai operator run --fixture tests/fixtures/operator/click_sandboxed_app.json"
  expect_ssb: "operator_action"
  expect_artifact: "a bounded screen-recording file whose hash == the operator_action row's frame/record hash; NEGATIVE: an action that actuates on a stale frame_hash, or an arm that exposes the capability before its grant row lands, FAILS"

## Wargame (plan-standard unit shape)
EXPECTED_OBSERVATION: an operator TaskSpec that clicks a sandboxed app lands exactly one operator_action row {frame_hash, action, lease_id} + one operator_frame_bind row off-agent, plus a screen-recording artifact whose hash matches the row; a coordinate action computed against a stale/compacted frame is rejected with no actuation.
ON_FAIL: an action actuates on a stale frame_hash (CAUSE: the frame-binder is not epoch-invalidating on compaction, or it retargets to display 0). COUNTER-MOVE: assert FrameBinding rejects when sha256(current-delivered-image) != the action's bound hash; add the compaction-epoch invalidation hook; the negative-arm test must FAIL closed.
FORK: if the vendored cua sandbox cannot be pinned to a deterministic image on the target host, route to the Docker/Windows-Sandbox backend (cua's multi-provider abstraction) and record the substrate in the RRA hardware pin; do NOT fall back to driving the host desktop directly.
RECON_NEEDED: confirm cua's current sandbox adapter API + license (MIT) and OmniParser icon_detect weights license (AGPL) via WebSearch/source at edit; confirm the exact orchestrator compaction hook that drops a screenshot from context (the epoch the frame-binder keys on). Settling check: `grep -n "sandbox" <cua clone>/libs/**/agent.py` + a source read of the compaction path.
ABORT: if a coordinate action would ever actuate without a live matching frame binding, or the sandbox is not disposable/isolated, STOP and flag (a computer-use lane that can touch the host outside its sandbox is a P0 — do not ship the producer alone).
VERIFY: the load-bearing proof is the off-agent operator_action SSB row + the recording-hash match, driven by the real `elai operator run` entrypoint (not a unit test calling the actuator). Run at execution:
```bash
python .claude/tools/plan_gate_check.py --file docs/thefinalglance/P-plan/P-12_desktop_operator_lane.md   # authoring gate (exit 0)
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/P-plan/P-12_desktop_operator_lane.md   # execution done-gate
python .claude/tools/production_caller_check.py --files crates/orchestrator/src/operator/mod.rs   # no DARK symbol
```
REDTEAM: surviving-attack seed — a model that CACHES a valid frame hash and replays it after the screen changed is defeated by the epoch invalidation (the hash is bound to the in-context image, dropped on compaction/node-switch). Attack that would succeed if unpatched: driving the host desktop when the sandbox is unavailable — PATCH: the FORK routes to another sandbox backend, never the host; sandbox-unavailable is an ABORT, not a fallback. Deep red-team DEFERRED to execution pre-flight (legacy-plan refine-before-execute path, as P-9..P-11).

## INTEGRATION_SMOKE
After P-12 + P-13 land: an independent agent drives `elai operator run` end-to-end on a HARD input (a real sandboxed GUI app, a multi-step click+type sequence long enough to trigger a compaction mid-run) and asserts (a) every actuation carries a live frame binding, (b) the compaction mid-run invalidates the stale frame and the next action re-binds, (c) operator_action rows + recording hashes match. Toy single-click happy-path does NOT satisfy R10.

## Grounding / citations
- Source: workflow/reasoning/canvas-computer-use-eval-2026-07-11.md (ENTRY 7/8/10/11/12, SYNTHESIS P4; comparators source-verified there). arXiv:2408.00203 (OmniParser), 2410.08164/2504.00906/2510.02250 (agent-s) — verified in the eval doc.
- ELAI ground truth (PLAN-asserted, executor re-greps W-7): E-14, J-1/J-8, G-1, G-2, 4F, 4I per the summary indexes cited above.
- CLAUDE.md hard rules invoked: blake3 not sha2; no .unwrap on prod; anti-slopsquatting on the cua vendor + OmniParser weights (license verified at edit); kill-switch not enable-flag (W-2).

## PRODUCTION_HOOK
crates/orchestrator/src/task_runner.rs — the O-2 tick that dispatches TaskSpec stages gains an operator_action stage kind that constructs + drives the OperatorLane; `elai operator run` in crates/cli/src/main.rs is the direct CLI entrypoint. The new OperatorLane fns are wired into BOTH in this changeset (vertical slice; not a later step).

## Next
Closes the desktop/GUI operator gap (SYNTHESIS 2a). Pairs with P-13 (the perception feed) and P-16 (the install catalog that provisions the apps this lane drives). Once landed, a workflow node "test this game build / drive this GUI app" is expressible, journaled, and replayable.

End of P-12.
