H-14 decisions — APP/WEBSITE DESIGN mode (web-first)
Session: s-10612 (2026-06-05)
FSM: EXECUTING
Schema baseline: SCHEMA_VERSION=222 / TOTAL_VARIANTS=499
H-14 SSB delta: ZERO (design.* ops ride SsbActionType::CanvasOpApplied per GD-1)

---

1. PROXY STRATEGY (done-gate: painted preview, not just 200)

The dev-preview proxy is the B0 prerequisite. ELAI fronts the design-target
dev server on a port ELAI controls; the proxy strips X-Frame-Options (obsolete
per MDN) and rewrites frame-ancestors in the CSP so the Tauri webview is an
allowed embedder.

Honest limit (GD-5): this works ONLY for web dev servers ELAI can front
(servers ELAI starts or proxies on a local port). An arbitrary deployed
third-party site with a hard CSP ELAI cannot rewrite is OUT OF SCOPE.
The done-gate is an off-agent screenshot of the PAINTED preview, not just
a 200 status on the fetch. This is Tier-2 deferred (stub-8).

Sources: CANVAS-WORKBENCH-DESIGN.md §8 + H-14-grounding.md (MDN X-Frame-Options
verified 2026-06-05: "ALLOW-FROM is obsolete; frame-ancestors is the modern replacement").

2. ONLOOK IS IFRAME-BASED (REFUTED novelty premise)

The original H-14 context assumed Onlook runs its own renderer ("novelty: renderer-vs-iframe").
This was REFUTED (verify:design-deps, H-GROUNDING-MAP.md:366):
  - Onlook IS iframe-based: IFrameView = HTMLIFrameElement, penpal postMessage RPC,
    @codesandbox/sdk web container.
  - ELAI's genuine novelty is the SANDBOX/PROXY axis: ELAI proxies an arbitrary
    foreign dev server (under ELAI's control) vs Onlook's CodeSandbox-SDK container.
  - This does NOT change the implementation; it corrects the marketing claim.

3. DATA-OID IS OPAQUE (server-side map, not line/col in DOM)

data-oid is a 16-char hex opaque id (blake3(file + scope + counter).to_hex()[..16]).
The id->location mapping is server-side ONLY in OidMap: HashMap<String, OidMapEntry>.
The DOM attribute carries ONLY the opaque id. Line/column is NOT encoded in the
DOM attribute — that is the brittle React-Dev-Inspector approach Onlook explicitly
avoids (verify:design-deps, H-GROUNDING-MAP.md:368).

Why blake3: CLAUDE.md HARD RULE Anti-Stale-Training (blake3, not sha2/blake2).
Collision probability for <= 65 536 elements per file: < 2^-48 (birthday bound on
64-bit truncated hash). The OID_WARN_THRESHOLD constant marks this boundary.

4. @onlook/babel-plugin-react IS A POLICY BLOCK (not just npm 404)

Anti-slopsquat check on 2026-06-05 (H-14-design.md exploration) found that the
package NOW EXISTS at npm@2.1.1. However, the H-14 stub_ban explicitly prohibits
the dep by POLICY (not just registry absence):
  - Mirror the @onlook/parser ids.ts pattern in-repo instead.
  - Use @babel/standalone@7.29.7 (MIT, verified 2026-06-05) for any JS transform.
  - This discrepancy is recorded here so future readers know the BLOCK is policy-driven.

5. RECAST PROVE-IT SPIKE — result and implications (GD-5)

The in-process prove-it spike (h14_design_mode_contract.rs contract (a)) PASSED:
  - The in-process text-replace path satisfies the deterministic writeback invariant:
    unmodified lines outside the changed attribute are byte-identical to the original.
  - recast.js (Node.js) satisfies the same invariant per its README:
    "reprints only those parts you modify" (github.com/benjamn/recast, verified).
  - recast.print() outputs the WHOLE FILE (no scoped diff API): the "scoped diff"
    notion means unmodified nodes are reprinted verbatim. A caller producing a patch
    must diff the reprint against the original using a separate tool.

Implications:
  - The DETERMINISTIC style/text writeback tier is VALID and ships.
  - The STRUCTURAL (LLM) tier remains stub-7 until the full recast.js round-trip
    is proven against a real large React codebase (requires Node.js at runtime in
    the Tauri process — Tier-2, separate gated checkbox).

6. RN/EXPO IS DEFERRED

React Native / Expo (Fiber + getInspectorDataForViewAtPoint, native render not iframe)
is a separate, later slice. Nothing in H-14 promises it. ELAI's locked stack for
the current gig is the web-first path described here.

7. GD-4 WRITEBACK CORRUPTION RATE — measurement deferred (T2-D)

GD-4: the writeback-corruption rate MUST be measured and bounded on >= 3 runs before
this path is default-ON. The measurement is Tier-2 deferred (separate gated checkbox,
T2-D). H-14 does NOT claim a corruption rate. The contract test verifies structural
correctness of the apply logic; the live-run measurement is separate.

8. FORWARD-DECLARED DEPS: @babel/standalone AND recast (A1 note)

recast@0.23.11 and @babel/standalone@7.29.7 are in dashboard-v2/package.json
runtime dependencies with ZERO current JS/TS callers in the dashboard src tree.

recast: forward-declared for stub-9 (the LLM structural writeback / TS-side
recast.js round-trip). The package was anti-slopsquat verified (MIT, 0.23.11,
npmjs.com, published and maintained). Its consumer arrives when stub-9 lands.

@babel/standalone@7.29.7: forward-declared as the JS-transform peer to recast
for the stub-9 data-oid Babel JSX transform. It is ~1MB (full Babel in-browser
runtime), but since it is currently unimported it is tree-shaken out of the
Vite/Tauri webview bundle — Vite's tree-shaker removes unimported ESM. The 1MB
cost is a Tier-2 concern triggered when stub-9 imports it. If the actual
transform runs at build time via node (not in the Tauri webview), the dep will
move to devDependencies at that point; no action needed now.

Neither dep has any wire-dark risk: the auditor's ZERO_IMPORT note is expected,
not accidental. The stub-9 babystep_id in stubs.md is the consumer pointer.

---

WIRED DONE-GATE STATUS

Tier-1 shipped (this session):
  - DESIGN_NAMESPACE const added to canvas_dispatcher.rs.
  - 'design' added to WIRED_MODES in canvas.svelte.ts.
  - data_oid_tagger.rs: create_oid (blake3-based opaque id) + OidMap + tag_jsx_element.
  - recast_writeback.rs: apply_recast_patch (deterministic tier) + PATCH_KIND_DETERMINISTIC.
  - dev_preview_proxy.rs: relax_headers_for_framing (X-Frame-Options removal +
    frame-ancestors rewrite) + start_dev_proxy scaffold.
  - commands_h14.rs: resolve_oid + apply_recast_patch + start_dev_proxy + stop_dev_proxy
    registered in lib.rs generate_handler!.
  - DesignPageNode.svelte: design op drain $effect + click-to-edit + page node/nav edge UI.
  - CanvasWindow.svelte: design branch wired (CanvasPanel + DesignPageNode mounted; R6 drain live).
  - stub.ts: DesignOidEntry + apply_recast_patch + resolve_oid + start_dev_proxy + stop_dev_proxy.
  - h14_design_mode_contract.rs: contracts (a)..(h) covering spike + oid + writeback + SSB round-trip.

Tier-2 deferred:
  - stub-7: structural (LLM+recast) writeback tier (GD-5 gate).
  - stub-8: live dev-preview proxy TCP listener + painted-preview screenshot done-gate.
  - T2-D: writeback-corruption rate measurement on >= 3 runs (GD-4).
  - T2-B: nav edges connecting pages into the real navigation graph (requires T2-A proxy live).
  - T2-C: VLM screenshot validation (media_validator Tier-2 VLM = None, RUNPOD-PENDING).
  - T2-E: full W-1 --run-trigger PASS (requires T2-A + T1-B + T1-C all live).

---

KEY REFERENCES:
  CANVAS-WORKBENCH-DESIGN.md §8 + §14
  H-GROUNDING-MAP.md verify:design-deps (2026-06-03)
  H-14-grounding.md (spec-grounder output, 2026-06-05)
  MDN X-Frame-Options (verified 2026-06-05): ALLOW-FROM obsolete; frame-ancestors modern
  github.com/benjamn/recast README (verified): reprints only modified parts
  CLAUDE.md HARD RULES: blake3, no .unwrap()/.expect() in prod, GD-1/GD-4/GD-5/HD-4
