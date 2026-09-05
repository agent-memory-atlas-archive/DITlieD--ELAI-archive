# H-14: APP/WEBSITE DESIGN mode (web-first; React Native / Expo deferred)

STATUS: PLANNED (WAVE 2; BUILD net-new; the riskiest mode; gated on a recast prove-it spike; GD-4/GD-5)

Objective: The richest mode and the user's original arc — a design.page node renders a live preview of a real web app
screen, nav edges connect pages into the real navigation graph, and you click an element INSIDE a page to edit it.
Web-first; RN/Expo explicitly deferred to a later separately-scoped slice (nothing here promises it). B0 (prerequisite):
ELAI's own dev-preview proxy serving a frameable origin. Click-to-edit: build-time data-oid OPAQUE-id tagging mirroring
@onlook/parser ids.ts in-repo (NOT the phantom @onlook/babel-plugin-react), with the id->{file,range,scope} map stored
SERVER-SIDE. Two-tier recast writeback (deterministic style/text patches; the LLM only for structural), gated on a
recast prove-it spike against a real large React codebase.

Estimated effort: ~3-4 sessions (~16-24 h), the largest in the plan. Net-new + a proxy + a build-time tagging step + a
writeback pipeline + a spike. Strongly gated.
Pre-requisites: H-13 (the canvas op transport — design.* ops ride canvas_op_applied) MUST land first. H-10/H-11/H-12 (the canvas shell/live layer/modes).
Parallel-safe with: nothing in the canvas track (it is the deepest dependency). 
Source: CANVAS-WORKBENCH-DESIGN.md §8 (the DESIGN mode + the iframe-embedding strategy + click-to-edit) + §14 (design-mode is the least-proven mode) + §13 C5; _grounding/H-GROUNDING-MAP.md the H-14 note + verify:design-deps (the load-bearing REFUTED/CORRECTED rows). Doctrine: H-0_INDEX.md.

## Context

You cannot point a Tauri <iframe> at localhost:3000 and expect it to render: the embedded app ships its own CSP and
X-Frame-Options (DENY/frame-ancestors blocks embedding), and a Service Worker can intercept requests (CANVAS §8). So
ELAI serves the design-target through its OWN dev-preview proxy on a dedicated port ELAI controls; the proxy strips/
relaxes X-Frame-Options + rewrites frame-ancestors so the Tauri webview is an allowed embedder, and scopes Service
Worker registration. The design.page iframe loads that ELAI-controlled proxied origin, never the raw dev server. Honest
limit (GD-5): this works for web dev servers ELAI can front; it does NOT embed an arbitrary deployed third-party site
with a hard CSP ELAI cannot rewrite. Click-to-edit (verify:design-deps, CORRECTED): React 19 removed Fiber _debugSource,
so build-time tagging is mandatory; Onlook's data-oid is an OPAQUE GENERATED id (createOid), NOT a sourcemap location —
the oid->location mapping is a SERVER-SIDE template-node map, not embedded in the attribute. The "Onlook runs its own
renderer" novelty premise is REFUTED — Onlook IS iframe-based (HTMLIFrameElement + CodeSandbox SDK + penpal); the
defensible H-14 difference is the SANDBOX/PROXY axis (ELAI proxies an arbitrary foreign app vs Onlook's CodeSandbox-SDK
container over penpal), not renderer-vs-iframe. Two-tier recast writeback: deterministic patches for style/text (a style
nudge never round-trips through the model), the LLM only for structural — and recast producing an applyable SCOPED diff
against a real large React codebase is UNSOLVED in the prior art (current tools regenerate whole artifacts, cannot
diff-against-existing or do multi-page) — so a prove-it spike precedes the structural path (GD-5 honest limit).

## Existing state (B/C/D/E/F/G coverage)
- No code-state reader was assigned to H-14 (it is a net-new web-first surface). The grounding is the design-deps web verification:
  - recast — EXISTS (0.23.11, MIT, github.com/benjamn/recast; maintained within ~15 months). The verified writeback printer.
  - @onlook/babel-plugin-react — PHANTOM/legacy: npm listing 403, ABSENT from the current main-branch repo (recursive grep empty) = BLOCK. The current impl is the in-repo @onlook/parser (@babel/standalone). Mirror packages/parser/src/ids.ts; do NOT depend on the phantom package.
  - data-oid — an OPAQUE GENERATED id (createOid + collision check), NOT line/column; the oid->location map is packages/parser/src/template-node/map.ts (SERVER-SIDE).
  - Onlook architecture — IFRAME-based (IFrameView = HTMLIFrameElement, useRef<HTMLIFrameElement>, @codesandbox/sdk + penpal). The renderer novelty is REFUTED.
- the H-13 canvas op transport (canvas_op_applied + the CanvasDispatcher) — design.* ops ride it.
- the media_validator (H-13 / shipped Tier2) — the off-agent paint verification for the preview.

## Reads (mandatory before coding)
- _grounding/H-GROUNDING-MAP.md the H-14 note + verify:design-deps (recast row + the REFUTED renderer-novelty + the CORRECTED data-oid-location + the CORRECTED babel-plugin rows + the DEPENDENCY BLOCKERS).
- CANVAS-WORKBENCH-DESIGN.md §8 (the DESIGN mode: the iframe-embedding strategy with its done-gate, the click-to-edit mechanism, the RN caveat, the token-references-not-hex rule) + §14 (the three concrete unknowns: the proxy, the novel proxied approach, the recast scoped-diff prove-it).
- the H-13 detail file (the canvas op transport design.* ops ride).
- CLAUDE.md HARD RULES + anti-slopsquatting (recast@0.23.11 verified; @onlook/babel-plugin-react BLOCK) + HD-4/GD-4/GD-5.

## Tasks (WAVE 2 — build, heavily gated)

1. **B0 prerequisite: the dev-preview proxy (~200 LOC) [its own done-gate].** ELAI serves the design-target through its
   own proxy on a dedicated port; strip/relax X-Frame-Options + rewrite frame-ancestors so the Tauri webview is an
   allowed embedder; scope Service Worker registration so the embedded app's SW cannot hijack the preview frame. Done-gate:
   the proxy serves a real dev server's page AND a Tauri webview embeds it, verified by an OFF-AGENT screenshot of the
   embedded page PAINTING (not a 200 on the fetch). NOVELTY re-grounded on the sandbox/proxy axis (Onlook is iframe-based).

2. **design.page live-preview node + nav edges (~220 LOC svelte).** DesignPageNode.svelte loads the ELAI-controlled
   proxied origin in an iframe; nav edges connect pages into the real navigation graph. Done-gate: a real app's pages
   render as preview nodes wired by their real navigation graph, verified by a screenshot of an actually-painted preview.

3. **Build-time data-oid OPAQUE-id tagging mirroring @onlook/parser ids.ts (~160 LOC) + the server-side map.** A Babel/
   @babel-standalone JSX transform that injects a data-oid OPAQUE generated id (createOid-style, collision-checked) at
   build time (React 19 removed _debugSource — build-time is mandatory); store the id->{file,range,scope} map SERVER-SIDE
   (do NOT encode line/col in the DOM attribute — that is the brittle React-Dev-Inspector approach Onlook avoids). Mirror
   @onlook/parser ids.ts in-repo; do NOT depend on @onlook/babel-plugin-react (BLOCK).

4. **Click-an-element -> AST-range selection (~90 LOC).** Clicking an element in a page node resolves its data-oid back
   to the AST range via the server-side map and selects that source node as model context (the deictic channel, §9).

5. **The recast prove-it spike (~80 LOC) BEFORE the structural path commits.** Prove recast can produce an applyable
   SCOPED diff against a real large React codebase (the prior art cannot diff-against-existing — GD-5). If the spike
   fails, the structural path is descoped and only the deterministic style/text path ships.

6. **Two-tier recast writeback (~180 LOC).** Deterministic patches for style/text edits (a style nudge never round-trips
   through the model — token references not raw hex/px, per CANVAS §8); the LLM ONLY for structural changes, applied via
   recast (format-preserving). Bound the writeback-corruption rate on >=3 runs (GD-4).

7. **Decision doc workflow/reasoning/H-14-decisions.md (~140 LOC).** Records: the proxy strategy (its done-gate); Onlook
   is iframe-based (REFUTED novelty — sandbox/proxy axis); data-oid is opaque (server-side map); @onlook/babel-plugin-react
   is a BLOCK (mirror @onlook/parser); the recast prove-it gate; RN deferred; the honest proxy limit (GD-5).

## WIRING_CONTRACT
produces_symbols:
  - crates/dashboard-v2/src-tauri/src/dev_preview_proxy.rs::DevPreviewProxy (frameable-origin proxy + SW scoping)
  - crates/dashboard-v2/src/lib/canvas/DesignPageNode.svelte (the proxied-iframe live preview + nav edges + click-to-edit)
  - crates/dashboard-v2/src-tauri/src/data_oid_tagger.rs (the build-time opaque-id JSX transform mirroring @onlook/parser ids.ts) + the server-side id->location map
  - crates/dashboard-v2/src-tauri/src/recast_writeback.rs (the two-tier deterministic+LLM writeback) + the recast prove-it spike record
consumes_symbols:
  - recast (the verified npm writeback printer — a new dep, anti-slopsquatting-verified @0.23.11)
  - the @onlook/parser ids.ts PATTERN (mirrored in-repo via @babel/standalone — NOT a dep)
  - crates/orchestrator/src/ssb.rs::SsbActionType::CanvasOpApplied (H-13 — design.* ops ride it; do NOT add a new transport)
  - the H-13 CanvasDispatcher (the design.* op-namespace gating)
  - the media_validator (H-13/shipped — the off-agent paint verification)
reachable_from:
  - the DESIGN mode frame (a canvas mode) -> DesignPageNode -> the dev-preview proxy  [the live preview]
  - the recast writeback command (CLI/IPC) -> a deterministic patch OR an LLM-structural patch on a real codebase  [the writeback]
stub_ban:
  - NO raw dev-server iframe (the ELAI-controlled proxied origin only — embedding by construction)
  - NO dependency on @onlook/babel-plugin-react (PHANTOM/BLOCK — mirror @onlook/parser ids.ts)
  - NO line/col encoded in the DOM attribute (opaque id + server-side map)
  - NO structural-writeback path before the recast prove-it spike passes (GD-5)
  - NO style/text edit round-tripping through the model (deterministic recast only)
  - NO new SSB transport (design.* ops ride canvas_op_applied — GD-1)
  - NO claim that the proxy embeds an arbitrary hard-CSP third-party site (honest limit, GD-5)
  - blake3; no .unwrap()/.expect() in prod; recast@0.23.11 anti-slopsquatting-verified before package.json add
frontend_pair:
  DesignPageNode.svelte (the proxied-iframe live preview + click-to-edit) + the dev-preview proxy (tauri/rust). cross_lang_diff PASS; W-6 dashboard E2E.
observable_effect:
  trigger: "(HD-4/GD-4/GD-5) point DESIGN mode at a real running web dev server (NO env override); click an element; make a style edit and a structural edit"
  expect_ssb: "design.* canvas_op_applied rows land for the preview node + nav edges; the recast writeback emits the patch result"
  expect_artifact: "a real app's pages render as preview nodes wired by their real navigation graph, verified by an
    OFF-AGENT screenshot of an actually-painted preview (NOT a 200); clicking an element resolves to its AST range via
    the opaque id; a style edit round-trips as a DETERMINISTIC recast patch with NO model call; a structural edit
    round-trips through the model with recast preserving formatting, both applied to a REAL codebase; the writeback-corruption rate is bounded"

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/H-plan/H-14_design_mode_web_first.md --run-trigger
test -f .elai_cc/crates/dashboard-v2/src-tauri/src/dev_preview_proxy.rs
test -f .elai_cc/crates/dashboard-v2/src/lib/canvas/DesignPageNode.svelte
rg "createOid|data-oid|template.node" .elai_cc/crates/dashboard-v2/src-tauri/src/data_oid_tagger.rs   # opaque-id tagging mirroring @onlook/parser
rg "onlook/babel-plugin-react" .elai_cc/crates/dashboard-v2/package.json                               # MUST be 0 (the phantom dep BLOCK)
npm view recast version time license   # anti-slopsquatting (expect 0.23.11, MIT)
rg "X-Frame-Options|frame-ancestors" .elai_cc/crates/dashboard-v2/src-tauri/src/dev_preview_proxy.rs  # the frameable-origin relaxation
python .claude/tools/cross_lang_diff.py
test -f workflow/reasoning/H-14-decisions.md
```

## ACCEPTANCE
- [ ] B0 done-gate: the dev-preview proxy serves a real dev server's page AND a Tauri webview embeds it (an off-agent screenshot of the PAINTED preview, not a 200).
- [ ] W-1 done-gate (HD-4): --run-trigger PASS (pages render as preview nodes; click-to-edit resolves; the two-tier writeback round-trips on a real codebase), verified off-agent.
- [ ] W-2 default-ON: DESIGN mode active behind the mode flag (no env override).
- [ ] W-3 production entrypoint: the preview + writeback fire from the real DESIGN mode + the real writeback command; not test-only.
- [ ] GD-4: the writeback-corruption rate is MEASURED + bounded on >=3 runs before default-on.
- [ ] GD-5: the recast prove-it spike passes before the structural path commits; the proxy honest limit is stated (no arbitrary-third-party claim).
- [ ] anti-slopsquatting: recast@0.23.11 verified; @onlook/babel-plugin-react NOT a dep (BLOCK honored).
- [ ] GD-1: design.* ops ride canvas_op_applied (no new transport); the @onlook/parser pattern is mirrored, not depended on.
- [ ] cargo check --workspace GREEN + dashboard svelte-check; auditor PASS.

## WIRING DONE-GATE (clause by clause)
- W-1: --run-trigger is ACCEPTANCE (HD-4 off-agent); the proxy + the page node (produced) are reached by the DESIGN mode (W-3); the screenshot is the off-agent paint proof.
- W-2: DESIGN mode default-on behind the mode flag.
- W-3: the DESIGN mode frame + the writeback command; not test.
- W-4: DesignPageNode + the proxy; cross_lang_diff PASS.
- W-5: a runnable point-at-a-real-dev-server -> a painted preview + a real writeback; the proxy is the prerequisite (its own gate).
- W-6: the preview + click-to-edit via dashboard-smoker.
- W-7: recast / @onlook/parser ids.ts / the Onlook iframe architecture web-verified 2026-06-03 (verify:design-deps).
- W-8: the page node (produced) consumes the proxy; the writeback (produced) consumes recast + the server-side map.
- W-9: NO new SSB transport (canvas_op_applied — H-13).
- W-10: the proxy + the writeback are local; the LLM-structural path is a separate gated checkbox if RunPod-pending.
- HD-4/GD-4/GD-5: off-agent paint gate; bounded corruption rate; the prove-it spike + the honest proxy limit.

## Tests to add
- crates/contracts/tests/h14_design_mode_contract.rs (~190 LOC): (a) the proxy relaxes X-Frame-Options + rewrites
  frame-ancestors so a fixture origin is embeddable; (b) the build-time tagger injects an OPAQUE data-oid (not line/col)
  and the server-side map resolves it to {file,range,scope}; (c) clicking a tagged element resolves to its AST range;
  (d) a style edit produces a DETERMINISTIC recast patch with NO model call; (e) a structural edit applies via recast
  preserving formatting on a real fixture codebase; (f) @onlook/babel-plugin-react is NOT imported anywhere.
- the recast prove-it spike fixture: an applyable scoped diff against a representative large React codebase.

## Files touched
- CREATE: crates/dashboard-v2/src-tauri/src/dev_preview_proxy.rs (~200 LOC) + data_oid_tagger.rs (~160 LOC) + recast_writeback.rs (~180 LOC)
- CREATE: crates/dashboard-v2/src/lib/canvas/DesignPageNode.svelte (~220 LOC)
- MODIFY: crates/dashboard-v2/package.json (~2 LOC: recast@0.23.11 + @babel/standalone, anti-slopsquatting-verified; NOT @onlook/babel-plugin-react)
- CREATE: crates/contracts/tests/h14_design_mode_contract.rs (~190 LOC)
- CREATE: workflow/reasoning/H-14-decisions.md (~140 LOC)

## Citations / Grounding
- CANVAS-WORKBENCH-DESIGN.md §8 (the iframe-embedding strategy + the dev-preview proxy + the build-time data-oid + the
  two-tier recast writeback + the RN caveat + token-references-not-hex) + §14 (the three concrete unknowns: the proxy, the novel proxied approach, the recast scoped-diff prove-it).
- verify:design-deps (2026-06-03): recast 0.23.11 MIT [CONFIRMED]; React 19 removed _debugSource -> build-time tagging
  [CONFIRMED]; Onlook data-oid is an opaque createOid id + server-side template-node map [CONFIRMED/CORRECTED]; Onlook
  is IFRAME-based not own-renderer [REFUTED]; @onlook/babel-plugin-react phantom/legacy [CORRECTED, BLOCK].
- CLAUDE.md HARD RULES + anti-slopsquatting + HD-4/GD-4/GD-5.

## Open questions / risks
- TOP risk: recast producing an applyable SCOPED diff against a real large React codebase is UNSOLVED prior art (current
  tools regenerate whole artifacts) — the prove-it spike is the gate; if it fails, the structural path is descoped and only the deterministic style/text path ships (still a real win).
- The proxy only fronts servers ELAI controls; an arbitrary deployed third-party hard-CSP site is out of scope (GD-5 honest limit, stated not pretended away).
- Generated UI is a measured a11y weakness (W4A 2025 — LLM UI code fails semantic structure/focus/keyboard nav); the click-to-edit + the preview must not regress a11y for the user's own app — flag as a known limit.
- RN/Expo (Fiber + getInspectorDataForViewAtPoint, native render not iframe) is a separate later slice; nothing in H-14 promises it (ELAI's locked stack for the current gig is RN — the web-first deferral must not read as a shipped RN feature).

## Next
H-15 (spawn_view) is the reply-channel generalization that the DESIGN preview (a rendered interactive view) is one
instance of. With H-14, the canvas hosts all five modes; H-16 retires the orbit last.

End of H-14.
