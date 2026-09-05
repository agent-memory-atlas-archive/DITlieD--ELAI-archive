# DEEP-H-004 exhaustive feature discovery

## Scope, integrity, and method

- Processed manifest: `docs/thefinalglance/feature-discovery/DEEP-SOURCE-MANIFEST.md:L710-L724`, heading `### DEEP-H-004` (the following blank separator is L725; `DEEP-H-005` begins at L726).
- Declared chunk: family **H**, class **numbered/index original**, **9 files**, **1,452 logical lines**, **153,591 bytes**, target status **under 1,800**.
- Mechanical verification: fresh `sha256sum` and `wc -l -c` checks matched every manifest row and both aggregates exactly. Missing paths: **0**; unexpected paths: **0**; duplicate paths: **0**; hash mismatches: **0**; line mismatches: **0**; byte mismatches: **0**.
- Read boundary: all nine originals below were read completely, from first through last logical line. No other original deep-source chunk was read. `08-H-SUMMARY_INDEX.discovery.md` was consulted only for orientation and delta comparison; no discovery atom is sourced solely from it.
- Anchors use the immutable manifest-verified source image in this pass: `H-1:Lx-Ly`, etc. Repeated statements in task, wiring, acceptance, and test sections are merged into one atom with all load-bearing anchors rather than counted twice.
- Classification: `E` = explicit source fact/requirement; `I` = inference, never promoted to a requirement; `A` = unresolved ambiguity/choice; `C` = conflict or correction that cannot be silently flattened. Domain tags cover `STATUS`, `CAP`, `STATE`, `FLOW`, `DATA`, `CFG`, `INT`, `SEC`, `FAIL`, `DET`, `OBS`, `TEST`, `LIMIT`, `DEP`, and `DEFER`.

### Exact processed manifest and verification

| Manifest row | Lines | Bytes | SHA-256 | Exact absolute path | Verification |
|---|---:|---:|---|---|---|
| DEEP-H-004 | 199 | 17,231 | `ba8ec7494f463d6dd520bfbc27fe7c15ac20e5a4b7cc5ed79958bd0b7cdb2f0f` | `docs/thefinalglance/H-plan/H-1_prompt_caching_production_path.md` | exact |
| DEEP-H-004 | 183 | 14,603 | `8462ba3d59bb28f837e8ed6a547132a715b4cd30451a9d8fee0cd9e1ae0b6664` | `docs/thefinalglance/H-plan/H-2_cache_stable_layout_breakpoints.md` | exact |
| DEEP-H-004 | 190 | 17,358 | `50a7fc0ff067efcbabd6063912f5d83b4a4a97316362805f6afe63e8027cc9ec` | `docs/thefinalglance/H-plan/H-14_design_mode_web_first.md` | exact |
| DEEP-H-004 | 170 | 14,804 | `912007044c0325a81e065b37ebb8e98a43e4942a7fffc0be5fd11417ba7bdf06` | `docs/thefinalglance/H-plan/H-22_trading_metamorphic_null_composite.md` | exact |
| DEEP-H-004 | 159 | 13,040 | `0df2211e3277109d497a10529b46a71348d5a50fd88c0a1654d5f262be440df5` | `docs/thefinalglance/H-plan/H-24_sim_to_real_parity.md` | exact |
| DEEP-H-004 | 178 | 15,141 | `e2b48cf7acc1274d901c7b7ee814142ec8f64b84b0e56b22ab6b25d1be4bdb17` | `docs/thefinalglance/H-plan/H-27_gate_self_validation.md` | exact |
| DEEP-H-004 | 211 | 19,143 | `bd45662c99682012e88dbd4ea44c1e3ad784154dd371a4ed544c8e029035b158` | `docs/thefinalglance/H-plan/H-43_execution_grounded_dgm_fitness.md` | exact |
| DEEP-H-004 | 101 | 23,826 | `ee7d9e6f6bede4c6fc051dbc40b54c0e64932cfae6777f7d6b56167d7501a5e3` | `docs/thefinalglance/H-plan/H-49_undo_redo_branching_timetravel.md` | exact |
| DEEP-H-004 | 61 | 18,445 | `e9b67c78d692e8492cbc886503f730935b18bc337c6c349671654d257d2df85d` | `docs/thefinalglance/H-plan/H-53_citation_prebind_constrained_synthesis.md` | exact |
| **Aggregate** | **1,452** | **153,591** | — | **9 exact paths** | **exact** |

## Per-file atomic ledgers

### H-1 — prompt caching production path

Source: `docs/thefinalglance/H-plan/H-1_prompt_caching_production_path.md` (199/199 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| H1-E001 | E/STATUS | `H-1` is PLANNED, Wave 1, in-process ACTIVATION rather than a new caching build, and is the measured-number foundation with H-2/H-9. | H-1:L1-L3 |
| H1-E002 | E/STATE | The primary `handle_infer_request -> AnthropicClient::send_message` path already calls `apply_anthropic_cache_markers` unconditionally from send and stream; it marks the last system block and last tool definition with `CacheControl::ephemeral()`. | H-1:L5-L9, L37-L41 |
| H1-E003 | E/STATE | `AnthropicApiRuntime::build_request` / `complete` hardcodes `cache_control: None` while `supports_caching:true`; live secondary callers include `hook_prompt`, focus-tunnel orchestration, and `rlaif_judge`. This is the stated RuntimeAdapter split-brain. | H-1:L9-L15, L42-L44 |
| H1-E004 | E/STATE | `measure_cache_hit_rate` exists but has no production caller; it returns `verdict_code=0` at hit rate `<=0.05` and is behind default-off `ELAI_CRITIC_OF_DESIGN_ENABLE=1` / allowlist row 39. | H-1:L12-L15, L48-L49 |
| H1-E005 | E/DATA | `CumulativeTokenTracker.cache_read_tokens` is shipped and `grounding_client` records `usage.cache_read_input_tokens`; this is the live counter the monitor must consume. | H-1:L50-L51 |
| H1-E006 | E/LIMIT | Sixteen `build_tool_catalog` `cache_control:None` sites are inert by design because the downstream stamper re-marks the last tool; H-1 must leave them untouched. | H-1:L52-L53, L85-L88, L106-L107 |
| H1-E007 | E/LIMIT | Cloud cache reads are stated as `0.1x` repeated-input cost; output is billed separately at `5x` input. Caching affects only the repeated input prefix, not output. | H-1:L27-L35 |
| H1-E008 | E/OBS | The only honest H-1 target is ELAI's measured hit rate, journal-recomputable under `ELAI_SEED`; the pooled literature range must not be reported as ELAI performance. | H-1:L33-L35, L80-L88, L142-L143 |
| H1-E009 | E/DEP | Prerequisites are the shipped stamper and live inference chain. H-1 must precede H-2; H-4 may proceed only with coordination on `anthropic.rs`; H-5/H-8 are disjoint. | H-1:L17-L23 |
| H1-E010 | E/CAP | `AnthropicApiRuntime::build_request` must call or share-factor the existing marker stamper, remove unconditional `None` on the stable prefix, and make an advertised caching capability operative. | H-1:L67-L72, L91-L100 |
| H1-E011 | E/CFG | The minimum cacheable prefix is keyed to the exact live model id, not a family/tier: the contract pins Opus 4.8 to 1,024 and Opus 4.6 to 4,096. | H-1:L71-L72, L109-L109, L162-L166 |
| H1-E012 | E/FLOW | A non-test production caller must read cache-read/input token counts per inference and invoke `measure_cache_hit_rate`. | H-1:L74-L78, L98-L104 |
| H1-E013 | E/CFG | Marker stamping and monitoring are default-on with no enable override; only a kill switch may disable the monitor, and the old critic enable gate cannot remain terminal. | H-1:L74-L78, L108-L108, L138-L140 |
| H1-E014 | E/OBS | Prefer carrying hit rate on the existing H-9 `CloudSpend`/token path; only if reuse is impossible may one advisory `CacheEfficiencySampled` / `cache_efficiency_sampled` Pattern-2 row be added. | H-1:L80-L83, L91-L94 |
| H1-E015 | E/DATA | If `CacheEfficiencySampled` is necessary, it is a baseline+1 schema addition and must also reconcile/bump `e45_cross_language_drift_contract.rs:294`; otherwise H-1 adds no action type. | H-1:L157-L158, L175-L175 |
| H1-E016 | E/INT | Produced surfaces are RuntimeAdapter marker application, a live `measure_cache_hit_rate` caller, and the reused-or-conditional hit-rate emitter; consumed symbols are `apply_anthropic_cache_markers`, `build_request`, `measure_cache_hit_rate`, `CumulativeTokenTracker`, and `handle_infer_request`. | H-1:L90-L100 |
| H1-E017 | E/FLOW | Production reachability must cover both `ipc_accept -> handle_infer_request -> send_message` and `hook_prompt`/focus-tunnel `-> AnthropicApiRuntime::complete`, plus the monitor-to-journal artifact path. | H-1:L101-L104, L147-L156 |
| H1-E018 | E/FAIL | If a path cannot stamp a marker, it must log the failure and must not continue falsely advertising `supports_caching`; silent fail-open is banned. | H-1:L105-L111 |
| H1-E019 | E/DET | Hashing is blake3-only and production paths may not use `.unwrap()` or `.expect()`. | H-1:L105-L111, L124-L131 |
| H1-E020 | E/INT | H-1 has no independent frontend; H-9 owns the cost-cockpit hit-rate tile and dashboard E2E. | H-1:L112-L114, L152-L154 |
| H1-E021 | E/TEST | The live trigger is a real two-turn cloud task exercising both the main inference path and a RuntimeAdapter caller with no override. | H-1:L115-L117 |
| H1-E022 | E/OBS | On turn 2 the journal must show a non-zero cache-read fraction and `cache_read_input_tokens > 0`; that fraction must be recomputable. | H-1:L118-L119, L134-L143 |
| H1-E023 | E/TEST | The RuntimeAdapter request must carry ephemeral markers where its baseline did not, while the already-cached main path remains unchanged. | H-1:L120-L122, L134-L141 |
| H1-E024 | E/TEST | `wiring_contract_check.py --run-trigger` must pass with real reachability/artifacts; cargo-green alone is explicitly insufficient. | H-1:L124-L137, L147-L159 |
| H1-E025 | E/TEST | The contract test covers marker placement on last system/tool blocks, exact-model thresholds, non-zero monitor math, and execution with `ELAI_CRITIC_OF_DESIGN_ENABLE` unset. | H-1:L161-L167 |
| H1-E026 | E/TEST | A two-turn fixture must exercise both the main loop and a RuntimeAdapter caller. | H-1:L167-L167 |
| H1-E027 | E/TEST | Completion also requires workspace cargo check, auditor passes for providers/orchestrator, and no new dependency. | H-1:L144-L145 |
| H1-E028 | E/DATA | A decision record `workflow/reasoning/H-1-decisions.md` must preserve the split-brain, exact-model policy, deliberately untouched `None` sites, and measured-not-literature decision. | H-1:L85-L88, L131-L131 |
| H1-E029 | E/LIMIT | The RuntimeAdapter call frequency and dollar impact are unmeasured; H-9 must measure frequency before any cost-delta claim. | H-1:L187-L190 |
| H1-E030 | E/CFG | A model swap changes the prefix threshold, so code must read the live model id rather than a constant. | H-1:L191-L191 |
| H1-E031 | E/STATUS | ELAI's own hit rate has no baseline and remains unverified until H-9 measures it; no number may be claimed beforehand. | H-1:L192-L192 |
| H1-E032 | E/DEP | H-2 consumes H-1's unified path to optimize stable-prefix/tail/TTL placement; H-9 turns the measurement into the dashboard control. | H-1:L194-L197 |
| H1-E033 | E/STATUS | Estimated effort is 4–6 hours and the gate is wholly Tier-1/in-process with no network, subprocess, or RunPod dependency at gate time. | H-1:L17-L17, L158-L158 |
| H1-E034 | E/DATA | Expected implementation artifacts modify `anthropic_api.rs`, `cache_check.rs`, and a live emitter site; create the contract and decision files; touch SSB locks only conditionally. | H-1:L169-L175 |
| H1-I001 | I/INT | Because the source allows either CloudSpend reuse or a new row, the stable product contract is the measured journal field, not a particular event type. This is an inference, not a source-selected implementation. | H-1:L80-L83, L91-L94 |
| H1-A001 | A/CFG | The source says “only a kill-switch disables” but names no new or existing kill-switch identifier for H-1. | H-1:L77-L77, L108-L108, L138-L138 |
| H1-A002 | A/INT | The exact production insertion site for hit-rate emission remains an authored choice: `production_agent_rpc.rs` or `grounding_client.rs`, with CloudSpend reuse preferred. | H-1:L80-L83, L169-L175 |
| H1-C001 | C/STATE | The H-1 original explicitly corrects only the *primary* markerless-path overclaim and still asserts the RuntimeAdapter split-brain; the orientation summary records a temporally later H-50 claim that all three cloud sites cache and that H-1's split-brain premise is refuted. This chunk cannot resolve that live-code conflict. | H-1:L5-L15, L27-L35; orientation `08-H-SUMMARY_INDEX.discovery.md:L1091,L1484` |

Category coverage: capabilities/actions/surfaces ✓; workflows/states ✓; schema/data/lifecycle ✓; flags/defaults/config ✓; integrations/events ✓; security/compliance ✓ through fail-open and hard-rule constraints; failures/fallback ✓; determinism/idempotency ✓; observability/evidence ✓; tests/done gates ✓; constraints/non-goals ✓; dependencies/status ✓. No explicit retry or permission model is specified.

### H-2 — cache-stable layout, breakpoints, and TTL

Source: `docs/thefinalglance/H-plan/H-2_cache_stable_layout_breakpoints.md` (183/183 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| H2-E001 | E/STATUS | `H-2` is PLANNED, Wave 1, in-process BUILD because the existing breakpoint mechanism is computed-but-inert and Local-only. | H-2:L1-L3 |
| H2-E002 | E/STATE | `ContextBudget` already defines two breakpoints after `PhaseSpec` and `CodeContext`; `update_breakpoints` computes blake3 prefix hashes and `bp.valid`. | H-2:L5-L8, L34-L39 |
| H2-E003 | E/STATE | `bp.valid` is consumed only by `eprintln!`, slots 2–7 are empty at the caller (making BP1 degenerate), and `ApiDefault`/`Cloud` never call the allocator. | H-2:L8-L14, L35-L41 |
| H2-E004 | E/STATE | `CumulativeTokenTracker` is shipped and the H-1 unified marker path is the placement target H-2 drives. | H-2:L42-L43 |
| H2-E005 | E/LIMIT | Cache identity is a fully byte-identical prefix; a change before a breakpoint invalidates it and everything after it. Volatile tool results must remain beyond the cached region. | H-2:L23-L32 |
| H2-E006 | E/LIMIT | With a 20-block lookback, appending more than 20 blocks after the stable-prefix marker can miss silently unless a tail breakpoint is added. | H-2:L28-L32 |
| H2-E007 | E/DEP | H-1 must land first because it provides the unified marker-carrying path; H-3/H-5/H-8 are parallel-safe, while H-1 and H-2 are not. | H-2:L16-L21 |
| H2-E008 | E/FLOW | Replace empty slot 2–7 inputs with real bytes and invoke `update_breakpoints` from `ApiDefault`/`Cloud`, not only Local. | H-2:L53-L57 |
| H2-E009 | E/FAIL | Place a marker only where `bp.valid == true`; an unstable prefix receives no marker, explicitly failing closed. | H-2:L59-L61, L97-L103 |
| H2-E010 | E/CFG | Respect Anthropic's four-breakpoint cap; use two breakpoints (stable prefix and tail) and reserve two. Exceeding the cap produces HTTP 400. | H-2:L59-L65 |
| H2-E011 | E/CAP | Add the second tail breakpoint so long tool loops remain within the 20-block lookback. | H-2:L63-L65 |
| H2-E012 | E/CFG | Set `"ttl":"1h"` only for sessions expected to exceed five minutes; do not send the removed `extended-cache-ttl-2025-04-11` beta header. | H-2:L67-L70, L97-L100 |
| H2-E013 | E/LIMIT | A one-hour cache write costs 2x and breaks even on the second read; the five-minute default breaks even on the first. Free hits refresh the one-hour TTL. | H-2:L67-L70 |
| H2-E014 | E/DET | Add a debug assertion over the entire cached byte prefix so timestamps, random ids, or reordered maps become test failures rather than silent cache bills. | H-2:L72-L75, L172-L174 |
| H2-E015 | E/DATA | Tool results, SSB rows, and RAG chunks stay strictly after the last breakpoint; the stated invalidation order is tools -> system -> messages. | H-2:L72-L75, L97-L103 |
| H2-E016 | E/CFG | Exact minimum prefixes are: Opus 4.8 and Sonnet 4.6 = 1,024; Opus 4.7/4.6/4.5 and Haiku 4.5 = 4,096; Haiku 3.5 = 2,048. Selection reads the live model id. | H-2:L77-L78 |
| H2-E017 | E/DATA | `H-2-decisions.md` must record inert inputs/reachability, the `bp.valid` rule, GA one-hour TTL without beta, exact-model prefixes, tail rationale, and the debug assertion. | H-2:L80-L82 |
| H2-E018 | E/INT | Produced symbols span cloud breakpoint reach/decision in `context_budget.rs`, real slot feed/tail/assert in `runtime_client.rs`, and TTL/model policy in the Anthropic marker implementation. | H-2:L84-L88 |
| H2-E019 | E/INT | Consumed symbols are `update_breakpoints`, `CacheBreakpoint`, `bp.valid`, `CumulativeTokenTracker`, and the H-1 unified marker path. | H-2:L89-L94 |
| H2-E020 | E/FLOW | Required production chain: `BudgetPolicy::ApiDefault/Cloud -> update_breakpoints(real slots) -> bp.valid -> marker -> outgoing cloud request`. | H-2:L95-L96 |
| H2-E021 | E/DET | Prefix hashes use blake3 only; production code may not use `.unwrap()` or `.expect()`. | H-2:L97-L103 |
| H2-E022 | E/INT | H-2 adds no own frontend or action type; H-9 owns breakpoint/hit-rate cockpit display and dashboard E2E. | H-2:L104-L105, L141-L146 |
| H2-E023 | E/TEST | The live trigger is a real >=3-turn cloud task with stable `PhaseSpec`/`CodeContext` and no override. | H-2:L106-L108 |
| H2-E024 | E/OBS | The run must record `bp.valid` being consumed and a rising cache-read fraction across turns through H-9's journal path. | H-2:L109-L110 |
| H2-E025 | E/TEST | Artifact proof requires byte-identical prefix, stable-prefix and tail markers, a holding assertion, and a deliberate perturbation that fails the assertion. | H-2:L111-L113, L126-L133 |
| H2-E026 | E/CFG | Cache-stable layout is default-on with no enable override; only a kill switch may disable it. | H-2:L126-L130, L137-L140 |
| H2-E027 | E/TEST | `wiring_contract_check.py --run-trigger` must pass against the live cloud path; cargo-green alone is insufficient. | H-2:L115-L128 |
| H2-E028 | E/TEST | The contract pins: non-empty/nondegenerate slot hashing; true->marker and false->none; stable/perturbed assertion arms; >20-block tail marker; GA 1h TTL/no beta; and exact-model thresholds. | H-2:L150-L155 |
| H2-E029 | E/TEST | Completion also requires workspace cargo check, auditor passes for orchestrator/CLI/providers, no new dependency, <=4 breakpoints, and no beta header. | H-2:L133-L135 |
| H2-E030 | E/STATUS | The gate is wholly Tier-1/in-process and requires no network at gate time; estimated effort is 6–10 hours. | H-2:L16-L16, L147-L147 |
| H2-E031 | E/DATA | Expected artifacts modify `runtime_client.rs`, `context_budget.rs`, and `anthropic.rs`, and create the contract and decision files. | H-2:L157-L162 |
| H2-E032 | E/LIMIT | A single tail breakpoint is insufficient for very long tool loops; the two reserved breakpoints must be budgeted. | H-2:L175-L175 |
| H2-E033 | E/CFG | One-hour TTL must be conditional on expected session length, never applied blanketly. | H-2:L176-L176 |
| H2-E034 | E/DEP | H-9 consumes the hit-rate/expiry evidence; H-5 pairs with the longer TTL for batch contexts; H-6 must mask/drop trailing observations to preserve H-2's prefix. | H-2:L178-L181 |
| H2-I001 | I/FLOW | The source requires predicting “expected session length” but supplies no estimator; choosing a duration predictor or user hint is implementation inference. | H-2:L67-L70, L176-L176 |
| H2-A001 | A/CFG | The source says two breakpoints already exist and also asks for a “SECOND tail breakpoint”; it is clear the final policy uses stable-prefix + tail, but it does not map the pre-existing PhaseSpec/CodeContext indices unambiguously to the final two active marker positions. | H-2:L5-L14, L63-L65 |
| H2-A002 | A/CFG | A kill switch is permitted but no exact H-2 switch identifier is named. | H-2:L129-L129, L139-L139 |

Category coverage: capabilities/actions/surfaces ✓; workflows/states ✓; schema/data ✓; flags/defaults/config ✓; integrations/events ✓; security/compliance ✓ through fail-closed/hard rules; failures/fallback ✓; determinism ✓; observability/evidence ✓; tests/done gates ✓; constraints/non-goals ✓; dependencies/status ✓. No explicit permission, retry, recovery, concurrency, or idempotency API is specified.

### H-14 — web-first APP/WEBSITE DESIGN mode

Source: `docs/thefinalglance/H-plan/H-14_design_mode_web_first.md` (190/190 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| H14-E001 | E/STATUS | `H-14` is PLANNED Wave 2, net-new BUILD, the riskiest/largest mode, gated on a recast prove-it spike. | H-14:L1-L3, L13-L14 |
| H14-E002 | E/CAP | A `design.page` node renders a live real-web-app screen; navigation edges form the real page graph; clicking an element inside a page initiates editing. | H-14:L5-L11 |
| H14-E003 | E/DEFER | The slice is web-first; React Native/Expo is explicitly deferred and nothing in H-14 may imply it shipped. | H-14:L5-L7, L179-L184 |
| H14-E004 | E/CAP | B0 is an ELAI-owned dev-preview proxy on a dedicated controlled port; the preview loads the proxied origin, never the raw dev server. | H-14:L8-L11, L21-L25, L54-L58 |
| H14-E005 | E/SEC | The proxy strips/relaxes `X-Frame-Options`, rewrites `frame-ancestors` for the Tauri webview, and scopes Service Worker registration so an app SW cannot hijack the preview. | H-14:L21-L25, L54-L58 |
| H14-E006 | E/LIMIT | The proxy only fronts web dev servers ELAI controls; arbitrary deployed hard-CSP third-party sites are outside scope and cannot be advertised as supported. | H-14:L25-L27, L100-L108, L179-L182 |
| H14-E007 | E/STATE | React 19 removed Fiber `_debugSource`, making build-time source tagging mandatory. | H-14:L27-L29, L64-L68 |
| H14-E008 | E/DATA | `data-oid` is an opaque, collision-checked generated id; the server stores `id -> {file, range, scope}`. Line/column must never appear in the DOM attribute. | H-14:L27-L30, L64-L68 |
| H14-E009 | E/DEP | Mirror the in-repo `@onlook/parser` `ids.ts` / `createOid` pattern through a Babel/`@babel/standalone` JSX transform; do not depend on phantom/legacy `@onlook/babel-plugin-react`. | H-14:L37-L42, L64-L68 |
| H14-E010 | E/CORRECTION | Onlook is itself iframe-based (`HTMLIFrameElement`, CodeSandbox SDK, penpal); H-14's defensible novelty is sandbox/proxy/writeback control, not iframe rendering. | H-14:L29-L35, L37-L42 |
| H14-E011 | E/DEP | `recast` 0.23.11, MIT is the verified writeback printer and must pass anti-slopsquatting verification before addition. | H-14:L37-L40, L91-L93, L119-L129 |
| H14-E012 | E/STATE | Producing an applicable scoped diff against an existing large React codebase is unresolved prior art; whole-artifact regeneration/multipage limitations do not prove it. | H-14:L32-L35, L73-L75 |
| H14-E013 | E/DEP | H-13 must land first; H-14 consumes its `CanvasOpApplied` transport/`CanvasDispatcher` and the media validator, on top of H-10/H-11/H-12. It is not parallel-safe within the canvas track. | H-14:L15-L17, L43-L44 |
| H14-E014 | E/TEST | B0 passes only when a real dev-server page paints inside a Tauri webview in an off-agent screenshot; HTTP 200 is not proof. | H-14:L54-L58, L132-L134 |
| H14-E015 | E/CAP | `DesignPageNode.svelte` loads the controlled iframe and renders real app pages connected by their actual navigation graph. | H-14:L60-L62 |
| H14-E016 | E/FLOW | A click resolves `data-oid` through the server map to the AST range and selects that source node as deictic model context. | H-14:L70-L71 |
| H14-E017 | E/TEST | A recast spike against a representative large React codebase must prove an applicable scoped diff before structural writeback commits. | H-14:L73-L75, L132-L140 |
| H14-E018 | E/FALLBACK | If the recast spike fails, structural editing is descoped; deterministic style/text editing is still allowed to ship. | H-14:L73-L75, L179-L181 |
| H14-E019 | E/CAP | Style and text edits use deterministic recast patches with no model call; style values use token references rather than raw hex/px. | H-14:L77-L79, L100-L106 |
| H14-E020 | E/CAP | Only structural edits use the LLM, and their writeback is applied by recast with formatting preservation. | H-14:L77-L79 |
| H14-E021 | E/OBS | Writeback-corruption rate must be measured and bounded on at least three real runs before default-on. | H-14:L77-L79, L132-L138 |
| H14-E022 | E/DATA | `H-14-decisions.md` records proxy/gate, corrected novelty, opaque ids/server map, phantom dependency block, recast gate, RN deferral, and proxy limit. | H-14:L81-L83 |
| H14-E023 | E/INT | Produced symbols are `DevPreviewProxy`, `DesignPageNode.svelte`, `data_oid_tagger.rs` plus server map, and `recast_writeback.rs` plus spike record. | H-14:L85-L90 |
| H14-E024 | E/INT | Design operations ride H-13's `CanvasOpApplied` and dispatcher; H-14 must not create a new SSB transport. | H-14:L91-L96, L100-L108 |
| H14-E025 | E/FLOW | Production reachability is DESIGN frame -> page node -> proxy, plus real CLI/IPC recast command -> deterministic or structural patch on a real codebase. | H-14:L97-L99 |
| H14-E026 | E/FAIL | Raw dev-server iframes, structural-before-spike, model-mediated style/text edits, line/column DOM ids, phantom dependency use, and arbitrary-site claims are hard failures. | H-14:L100-L108 |
| H14-E027 | E/DET | Production Rust follows blake3 and no `.unwrap()`/`.expect()`; recast is pinned to verified 0.23.11. | H-14:L100-L108 |
| H14-E028 | E/INT | Frontend pairing is `DesignPageNode.svelte` with the Rust/Tauri proxy; `cross_lang_diff.py` and dashboard E2E must pass. | H-14:L109-L110, L143-L154 |
| H14-E029 | E/OBS | Live operation emits `design.*` `canvas_op_applied` rows for preview/nav changes and a recast patch result; the rendered/click/writeback artifacts must be off-agent verifiable. | H-14:L111-L117 |
| H14-E030 | E/TEST | End-to-end proof points DESIGN at a real dev server, clicks an element, performs one style and one structural edit, verifies correct AST resolution, deterministic no-model style patch, and formatting-preserving structural patch. | H-14:L111-L117, L132-L140 |
| H14-E031 | E/CFG | DESIGN is default-on behind its mode flag with no environment override. | H-14:L132-L136, L143-L146 |
| H14-E032 | E/TEST | `wiring_contract_check --run-trigger`, painted-preview proof, real production commands, anti-slopsquatting, cargo check, Svelte check, and auditor pass are required. | H-14:L119-L141 |
| H14-E033 | E/TEST | The contract tests proxy header rewriting, opaque tagging/server resolution, click-to-AST, deterministic style edit without model, structural formatting preservation, and absence of the phantom import. | H-14:L156-L162 |
| H14-E034 | E/DATA | Expected artifacts create three Tauri/Rust modules and `DesignPageNode.svelte`, add recast and Babel standalone to package metadata, and create contract/decision files. | H-14:L164-L169 |
| H14-E035 | E/LIMIT | Generated UI has a known accessibility risk (semantic structure, focus, keyboard navigation); preview/click-to-edit must not regress the user's app's a11y. | H-14:L183-L183 |
| H14-E036 | E/LIMIT | RN/Expo would require native/Fiber inspection (`getInspectorDataForViewAtPoint`), not the iframe mechanism, reinforcing that it is a separate slice. | H-14:L184-L184 |
| H14-E037 | E/DEP | H-15 generalizes the rendered interactive reply channel, and H-16 retires Orbit only after H-14 helps complete all five Canvas modes. | H-14:L186-L188 |
| H14-E038 | E/STATUS | Local proxy/writeback work is Tier-1; an LLM structural path may remain a separately gated checkbox when RunPod-dependent. | H-14:L143-L154 |
| H14-I001 | I/SEC | Because the server map exposes source location only through a controlled resolution path, authorization for that endpoint will be required in implementation, but the source defines no concrete auth/permission model. | H-14:L64-L71, L85-L99 |
| H14-A001 | A/TEST | The source both allows structural editing to be descoped on spike failure and lists successful structural editing in the overall W-1 acceptance. It does not define the alternate “H-14 complete without structural path” acceptance manifest. | H-14:L73-L79, L132-L140, L179-L181 |
| H14-A002 | A/CFG | “Default-on behind the mode flag” names no exact flag or rollout-state identifier. | H-14:L135-L135, L145-L145 |
| H14-A003 | A/LIMIT | The a11y statement says “must not regress” but supplies no metric, corpus, threshold, or done gate. | H-14:L183-L183 |

Category coverage: capabilities/actions/surfaces ✓; workflows/states ✓; schemas/data ✓; flags/defaults/config ✓; integrations/events ✓; security/privacy ✓; failures/fallback/recovery ✓; determinism ✓; observability/evidence ✓; tests/acceptance ✓; constraints/non-goals ✓; dependencies/status/deferred ✓. No explicit retry, idempotency, or concurrency policy is specified.

### H-22 — trading metamorphic null-test COMPOSITE

Source: `docs/thefinalglance/H-plan/H-22_trading_metamorphic_null_composite.md` (170/170 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| H22-E001 | E/STATUS | `H-22` is PLANNED Track-3/Wave-3 Cluster C, split into Tier-1 in-process relations and a Tier-2 permutation sidecar, and depends on H-17. | H-22:L1-L3 |
| H22-E002 | E/STATE | Existing compile/unit/DSR gates can admit overfit or leaking backtests; ELAI has a generic MR framework but no trading relation library or leakage gate. | H-22:L5-L13, L33-L40 |
| H22-E003 | E/FLOW | A frozen auditor-authored Layer-4 COMPOSITE runs on every discovered strategy and every backtest-touching commit. | H-22:L5-L13, L63-L66 |
| H22-E004 | E/CAP | `FutureShuffle`: apparent edge must collapse to noise when future data is shuffled; survival signals leakage and blocks. | H-22:L7-L9, L52-L57 |
| H22-E005 | E/CAP | `TimeReversal` is a named registered trading metamorphic relation. | H-22:L8-L9, L52-L57 |
| H22-E006 | E/CAP | `ZeroDriftPermutationNull` creates a zero-drift/permuted-return null distribution. | H-22:L8-L10, L52-L61 |
| H22-E007 | E/CAP | DSR is compared against the permutation null using persisted trial count `N` from H-17. | H-22:L8-L10, L59-L66 |
| H22-E008 | E/CAP | `ScaleInvariance` enforces multiplicative scale invariance. | H-22:L9-L10, L52-L57 |
| H22-E009 | E/CAP | `AdditiveInvariance` enforces additive invariance. | H-22:L9-L10, L52-L57 |
| H22-E010 | E/CAP | `CostMonotoneConcave` requires impact costs to increase monotonically and concavely with size under a square-root-law model. | H-22:L9-L10, L52-L57 |
| H22-E011 | E/DET | `SeedDeterminism` requires equality of blake3 equity-curve hashes under `ELAI_SEED`. | H-22:L9-L10, L52-L57 |
| H22-E012 | E/DEP | Extend the shipped `RelationLibrary` and `parse_relation`; reuse `MetamorphicTestRunner` and `MrViolated`, never reimplement the framework. | H-22:L10-L13, L33-L36, L52-L57 |
| H22-E013 | E/STATE | Shipped prerequisites include MR kinds Identity/Inverse/PermutationInvariance and rules-engine `MetricGate`/`COMPOSITE`; the seven trading relations are absent. | H-22:L18-L20, L33-L40 |
| H22-E014 | E/FLOW | Tier-2 `compute_null_band` shuffles future/returns, computes DSR-vs-null using H-17 N, is seeded by `ELAI_SEED`, and is frozen in the gate manifest. | H-22:L59-L61, L81-L90 |
| H22-E015 | E/CFG | `finance/trading_nulls.toml` is a domain-finance, `mutable=0`, LOCKED COMPOSITE running all relations. | H-22:L63-L66, L81-L86 |
| H22-E016 | E/CFG | Human-set alpha is the only tunable and is LOCKED; agents cannot change it. | H-22:L29-L31, L63-L66 |
| H22-E017 | E/FAIL | The composite blocks when the future-shuffled edge survives, DSR falls below the permutation-null alpha, or the null-band evidence is missing. | H-22:L63-L66, L94-L99 |
| H22-E018 | E/OBS | Add `TradingNullTest` with kinds `FutureShuffleSurvived`, `PermutationNullFail`, `CostLawViolated`, `SeedNondeterministic`, carrying null band and observed statistic; per-relation failures still use `MrViolated`. | H-22:L68-L71 |
| H22-E019 | E/DATA | `TradingNullTest` is Pattern-2 baseline+1 from the live schema, with action lock/e45 reconciliation; the null band is persisted as off-agent evidence. | H-22:L68-L71, L125-L132 |
| H22-E020 | E/TEST | Fixtures include a leaky strategy whose edge survives shuffle and must block, plus a genuine strategy whose edge collapses and must pass; the live run is separately gated. | H-22:L73-L74 |
| H22-E021 | E/OBS | Threshold false-positive rate must be measured/bounded on at least three real strategies before default-on. | H-22:L76-L79, L158-L160 |
| H22-E022 | E/DEP | H-27 mutation-tests these relations; H-17 supplies N; H-23/H-24 are parallel-safe; H-21 is complementary rather than redundant. | H-22:L18-L21, L76-L79, L161-L163 |
| H22-E023 | E/DATA | `H-22-decisions.md` records relation grounding, locked alpha, threshold risk/calibration, and H-27 self-validation. | H-22:L76-L79 |
| H22-E024 | E/INT | Produced symbols are seven named relation variants, Python `compute_null_band`, locked TOML, and `SsbActionType::TradingNullTest`. | H-22:L81-L86 |
| H22-E025 | E/INT | Consumed symbols are the shipped MR framework, H-17 `trial_count`, and rules `CompiledPredicate` COMPOSITE/MetricGate. | H-22:L87-L90 |
| H22-E026 | E/FLOW | Production reachability is pre-promotion plus backtest-touching commit hook; the COMPOSITE invokes the sidecar and emits `TradingNullTest`. | H-22:L91-L93 |
| H22-E027 | E/FAIL | Tier-2 must not advertise fail-closed behavior if its sidecar actually fails open. | H-22:L94-L99 |
| H22-E028 | E/DET | Equity hashes are blake3-only; production code forbids `.unwrap()`/`.expect()`. | H-22:L94-L99 |
| H22-E029 | E/INT | The trust cockpit shows null band and surviving-edge flag; cross-language pairing and a visible blocked leak are required. | H-22:L100-L105 |
| H22-E030 | E/OBS | For the leaky fixture, a journal-recomputable `TradingNullTest(FutureShuffleSurvived)` and the permutation band must land under `ELAI_SEED`. | H-22:L102-L105 |
| H22-E031 | E/CFG | The COMPOSITE is default-on on every promotion/backtest commit with no enable override; only a kill switch may disable it. | H-22:L116-L123 |
| H22-E032 | E/TEST | `wiring_contract_check --run-trigger` must block the leaky strategy and persist its null band; cargo-green alone is insufficient. | H-22:L107-L123 |
| H22-E033 | E/TEST | Acceptance requires N from H-17, locked alpha, H-27 validation, MR framework reuse, same-seed recomputation, Tier-1 fail-closed, and Tier-2 fixture/live checkbox. | H-22:L116-L123 |
| H22-E034 | E/TEST | Contract arms cover future-shuffle block/pass, permutation-null failure, cost-law failure, seed nondeterminism, immutable alpha, and relation registration/MrViolated reuse. | H-22:L134-L139 |
| H22-E035 | E/DEP | No new dependency is allowed; permutation code uses NumPy already present in the sidecar. | H-22:L116-L123 |
| H22-E036 | E/DATA | Expected artifacts modify the MR library and live gate hooks, create sidecar/TOML/fixtures/tests/decision doc, freeze sidecar/alpha, and bump SSB lock sites. | H-22:L141-L149 |
| H22-E037 | E/LIMIT | Cost-law relevance at retail size is unconfirmed and must be verified before locking that relation. | H-22:L164-L164 |
| H22-E038 | E/LIMIT | H-22 + H-18 + H-24 form a necessary, explicitly not sufficient, condition for a trustworthy edge. | H-22:L166-L168 |
| H22-E039 | E/STATUS | Estimated effort is 1.5–2 engineering weeks; threshold calibration is the stated main risk. | H-22:L15-L17, L158-L160 |
| H22-I001 | I/DET | Hash equality guarantees same-seed evidence only within the implemented environment; the source does not claim cross-hardware numerical identity. | H-22:L55-L61, L102-L105 |
| H22-A001 | A/CAP | The pass semantics/tolerances for `TimeReversal`, `ScaleInvariance`, and `AdditiveInvariance` are named but not numerically specified. | H-22:L52-L57 |
| H22-A002 | A/FLOW | Tier-1 says missing null-band fails closed, while producing the actual permutation/null band is assigned to the Tier-2 sidecar. The source does not fully specify how Tier-1 can become active before Tier-2 evidence exists. | H-22:L15-L17, L59-L66, L116-L123 |
| H22-A003 | A/CFG | Alpha and null-band calibration are mandatory, but no initial alpha/band value or calibration algorithm is fixed. | H-22:L63-L66, L76-L79, L158-L160 |

Category coverage: capabilities/actions/surfaces ✓; workflows/states ✓; schema/data/lifecycle ✓; flags/defaults/config ✓; integrations/events ✓; security/compliance ✓ through frozen/locked controls; failures/fallback ✓; determinism ✓; observability/evidence ✓; tests/done gates ✓; constraints/non-goals ✓; dependencies/status/tiering ✓. No explicit permissions, retry, recovery, concurrency, or idempotency API is specified.

### H-24 — sim-to-real structural/replay parity

Source: `docs/thefinalglance/H-plan/H-24_sim_to_real_parity.md` (159/159 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| H24-E001 | E/STATUS | `H-24` is PLANNED Track-3/Wave-3 Cluster D, building Tier-1 Parts A+C on shipped rules/replay substrate; Part B is deferred. | H-24:L1-L3 |
| H24-E002 | E/STATE | No existing gate proves research/live code-path identity or seeded replay/golden equality; those gates are absent. | H-24:L5-L13, L31-L38 |
| H24-E003 | E/CAP | Part A adds a T1 `AST_PATTERN` that blocks mode-divergent branches including `if live`, `if backtest`, and `if mode == "live"` in execution-path modules. | H-24:L5-L10, L47-L51 |
| H24-E004 | E/CAP | The same Part-A rule blocks duplicate fill, router, and order implementations on the execution path at `TESTING_FIRST`. | H-24:L7-L10, L49-L51 |
| H24-E005 | E/DATA | A `SCHEMA_REQUIRED` manifest declares exactly one shared execution entrypoint plus swappable data-source, fill-model, and clock adapters; missing declaration blocks. | H-24:L8-L10, L53-L55 |
| H24-E006 | E/FLOW | Live and backtest paths may differ only by adapter, never strategy/execution logic. | H-24:L53-L55 |
| H24-E007 | E/CAP | Part C replays the recorded event log under `ELAI_SEED` using shipped `replay_resume`/RRA and compares `blake3(replay)` to `blake3(golden)`. | H-24:L10-L13, L57-L60 |
| H24-E008 | E/FAIL | Replay divergence fails and reports the first mismatching event via the `DriftComparator` pattern; missing golden evidence also fails, never skips. | H-24:L57-L60, L86-L90 |
| H24-E009 | E/DEFER | Part B numeric live-vs-backtest reconciliation is Tier-2, requiring a live/paper run and LOCKED thresholds; only a deterministic fixture stub/documentation ships now. | H-24:L11-L15, L66-L68 |
| H24-E010 | E/LIMIT | Until Part B exists, H-24 proves structural same-path and reproducible-replay parity only; it must not advertise numeric behavioral parity. | H-24:L21-L29, L146-L152 |
| H24-E011 | E/DEP | Shipped prerequisites are rules `AstPattern`, `SchemaRequired`, `MetricGate`, `replay_resume`, RRA replay, and exchange-core determinism (integer money, single writer, `ClockProvider`, seeded RNG). | H-24:L15-L18, L31-L37 |
| H24-E012 | E/DEP | H-24 is parallel-safe with H-23/H-25/H-26; it composes, rather than reimplements, the common replay substrate. | H-24:L16-L19, L70-L72 |
| H24-E013 | E/OBS | Add `SimRealParity` kinds `ModeDivergentBranch`, `ReplayGoldenMismatch`, and `SingleEntrypointMissing`; payload/evidence includes AST findings and replay/golden hashes. | H-24:L62-L64 |
| H24-E014 | E/DATA | `SimRealParity` is a Pattern-2 baseline+1 live-schema bump with action lock and e45 reconciliation. | H-24:L62-L64, L115-L122 |
| H24-E015 | E/DATA | `H-24-decisions.md` records deterministic A+C, deferred B, shared-entrypoint/adapters, and reuse of replay/first-mismatch logic. | H-24:L70-L72 |
| H24-E016 | E/INT | Produced symbols are `verify_replay_golden`, locked `repro/sim_real_parity.toml`, and `SsbActionType::SimRealParity`. | H-24:L74-L78 |
| H24-E017 | E/INT | Consumed symbols are `replay_resume`, CLI RRA replay, `DriftComparator`, and rules `AstPattern`/`SchemaRequired`/`MetricGate`. | H-24:L79-L82 |
| H24-E018 | E/FLOW | Part A runs from `PreToolUse`; Part C runs pre-promotion through `verify_replay_golden -> replay_resume -> SimRealParity`. | H-24:L83-L85 |
| H24-E019 | E/DET | Replay/golden hashing is blake3-only; production code forbids `.unwrap()`/`.expect()` and wall-clock use follows `ClockProvider`. | H-24:L86-L90, L139-L144 |
| H24-E020 | E/INT | The trust cockpit shows mode findings and replay/golden hashes; cross-language checks and visible blocked fast paths are required. | H-24:L91-L96 |
| H24-E021 | E/TEST | A live fixture with an `if backtest` fill fast-path plus divergent log must emit both divergent-branch and replay-mismatch rows with the first mismatch; a shared clean path passes. | H-24:L93-L96 |
| H24-E022 | E/CFG | Parts A+C are default-on with no enable override; only a kill switch may disable them. | H-24:L107-L113 |
| H24-E023 | E/TEST | `wiring_contract_check --run-trigger` must block a fast path and fail a divergent replay; cargo-green alone is insufficient. | H-24:L98-L113 |
| H24-E024 | E/DET | The replay hash must recompute under the same `ELAI_SEED` and environment; the source keeps the scope honest rather than claiming broader parity. | H-24:L107-L113, L124-L128 |
| H24-E025 | E/TEST | Contract arms cover mode branch, duplicate fill, missing manifest, first-event replay mismatch, clean allow, explicit Part-B deferral, and same-seed hash identity. | H-24:L124-L129 |
| H24-E026 | E/TEST | Completion requires workspace cargo check, auditor pass, and no new dependency. | H-24:L107-L113 |
| H24-E027 | E/DATA | Expected artifacts create verifier/TOML/contracts/fixtures/decision doc, wire `PreToolUse` and promotion paths, and bump SSB lock sites. | H-24:L131-L137 |
| H24-E028 | E/SEC | The golden event log and hash must be trusted and frozen in the gate manifest so an agent cannot regenerate a favorable golden; H-28 guards regeneration. | H-24:L149-L150 |
| H24-E029 | E/LIMIT | AST scanning must be scoped to the shared entrypoint's transitive imports, not every `if` statement in the repository. | H-24:L146-L148 |
| H24-E030 | E/STATUS | Estimated A+C effort is about six hours; the current done gate is wholly in-process, while Part B needs external live/paper execution. | H-24:L15-L15 |
| H24-E031 | E/DEP | H-23/H-24/H-25 jointly provide deterministic reproducibility, parity, and anti-fabrication; H-25 follows to bind empirical numbers. | H-24:L154-L157 |
| H24-I001 | I/SEC | Because the manifest scopes transitive execution imports and pins a golden, manifest authorship/write authority must be protected, but the source names freezing rather than a concrete permissions system. | H-24:L49-L60, L147-L150 |
| H24-A001 | A/CAP | The duplicate-function detector's equivalence heuristic and the exact syntax/language coverage of `if live` variants are not specified. | H-24:L49-L55, L146-L148 |
| H24-A002 | A/CFG | The source permits a kill switch but gives no exact identifier for Parts A+C. | H-24:L109-L109, L117-L117 |

Category coverage: capabilities/actions/surfaces ✓; workflows/states/lifecycle ✓; schemas/data ✓; flags/defaults/config ✓; integrations/events ✓; security/compliance ✓; failures/fallback ✓; determinism ✓; observability/evidence ✓; tests/done gates ✓; constraints/non-goals ✓; dependencies/status/deferred ✓. No explicit retry, recovery, concurrency, idempotency, or end-user permission API is specified.

### H-27 — recursive gate self-validation

Source: `docs/thefinalglance/H-plan/H-27_gate_self_validation.md` (178/178 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| H27-E001 | E/STATUS | `H-27` is PLANNED Track-3/Wave-3 Cluster-F capstone, an in-process BUILD over shipped mutation/metamorphic/trap systems. | H-27:L1-L3 |
| H27-E002 | E/STATE | Existing mutation, MR, trap, freeze, and gate lifecycle surfaces are shipped but are not recursively applied to gate logic; the self-validation harness is absent. | H-27:L5-L14, L34-L43 |
| H27-E003 | E/FLOW | Every gate revision must pass mechanical self-validation before ACTIVE on the lifecycle promotion hook. | H-27:L5-L14, L61-L64 |
| H27-E004 | E/TEST | A gate must kill 100% (`mutation_score == 1.0`) of its frozen, human-authored, out-of-band mutant corpus targeting its own logic. | H-27:L8-L11, L56-L64 |
| H27-E005 | E/TEST | A gate must satisfy all declared metamorphic relations, including mandatory `null_band_sidedness`. | H-27:L8-L12, L61-L69 |
| H27-E006 | E/TEST | A gate must reject every frozen `TrapTask` through shipped `validate_for_traps`. | H-27:L9-L11, L61-L64 |
| H27-E007 | E/FAIL | Any escaped mutant, failed relation, or accepted trap blocks promotion and increments a persisted escaped-mutant counter. | H-27:L11-L14, L61-L73 |
| H27-E008 | E/DATA | Mutant corpora are per gate and include comparator flips, one-sided-to-symmetric changes, and N off-by-one cases; H-18/H-20/H-21/H-22/H-26 register corpora. | H-27:L56-L60 |
| H27-E009 | E/SEC | Corpora are frozen in `gate-manifest.txt`, authored out-of-band, and cannot be edited by the evaluated run; `gate_freeze_check.py` blocks self-edit. | H-27:L56-L60, L75-L76 |
| H27-E010 | E/DATA | Every band-using gate has a non-defaultable `SCHEMA_REQUIRED null_band_sidedness` field declaring one-sided versus symmetric. | H-27:L12-L14, L66-L69 |
| H27-E011 | E/FAIL | A T1 `AstPattern` forbids `abs(x) < band` when a gate declares a one-sided band; omission or symmetric implementation fails closed. | H-27:L12-L14, L66-L69 |
| H27-E012 | E/DET | Self-validation is entirely mechanical mutation/MR/trap scoring; no LLM verdict is allowed. | H-27:L16-L16, L23-L32 |
| H27-E013 | E/DEP | Reuse `MutationRunner`/`mutation_score_pct`, `RelationLibrary`/`parse_relation`, `validate_for_traps`/trap registry, and the lifecycle hook; do not reimplement runners. | H-27:L17-L20, L34-L42, L89-L93 |
| H27-E014 | E/OBS | Add `GateSelfValidation` kinds `Passed`, `MutantEscaped`, `MrFailed`, `TrapAccepted`, carrying the mutant-kill report. | H-27:L71-L73 |
| H27-E015 | E/DATA | The escaped-mutant counter is persisted, append-only, and agent-unresettable. | H-27:L71-L73 |
| H27-E016 | E/DATA | `H-27-decisions.md` records verify-the-verifiers doctrine, the literal GBM fix, mechanical-only judgment, and frozen out-of-band corpora. | H-27:L78-L80 |
| H27-E017 | E/INT | Produced symbols are `validate_gate_revision`, `NullBandSidedness`, locked meta TOML, `GateSelfValidation`, and per-gate corpora. | H-27:L82-L88 |
| H27-E018 | E/FLOW | Required live chain is rules lifecycle `SHADOW -> ACTIVE -> validate_gate_revision -> mutation/MR/trap checks -> SSB row`. | H-27:L89-L96 |
| H27-E019 | E/FAIL | A gate can never go ACTIVE on missing/escaped evidence, and the evaluated run cannot supply its own corpus. | H-27:L97-L102 |
| H27-E020 | E/DET | Production implementation uses blake3 and no `.unwrap()`/`.expect()`. | H-27:L97-L102 |
| H27-E021 | E/INT | The rules/meta cockpit shows mutant-kill report, escaped counter, and blocked promotion; cross-language pairing is required. | H-27:L103-L108 |
| H27-E022 | E/TEST | A proposed one-sided gate implemented with `abs(x)<band` must emit `MutantEscaped` or `MrFailed`, remain non-ACTIVE, and increment the counter; a correct gate kills all and promotes. | H-27:L105-L108 |
| H27-E023 | E/CFG | Self-validation runs on every promotion with no enable override; only a kill switch may disable it. | H-27:L120-L127 |
| H27-E024 | E/DET | Mutant-kill reports must recompute under `ELAI_SEED`. | H-27:L120-L127 |
| H27-E025 | E/TEST | `wiring_contract_check --run-trigger` and `gate_freeze_check.py` must pass on live promotion reachability; cargo-green alone is insufficient. | H-27:L110-L127 |
| H27-E026 | E/DATA | `GateSelfValidation` is a Pattern-2 baseline+1 schema bump with action lock and e45 reconciliation. | H-27:L129-L136 |
| H27-E027 | E/TEST | Contract arms cover sidedness MR failure, AST block, clean ACTIVE transition, escaped-counter increment, accepted-trap failure, frozen-corpus self-edit rejection, and absence of LLM judgment. | H-27:L138-L144 |
| H27-E028 | E/TEST | Completion requires workspace cargo check, auditor pass, and no new dependency. | H-27:L120-L127 |
| H27-E029 | E/DATA | Expected artifacts create validator/TOML/corpora/contracts/decision doc, extend MR library, hook lifecycle, and bump SSB lock sites. | H-27:L146-L154 |
| H27-E030 | E/LIMIT | Corpus adequacy is not proven by coverage; it is human-grown as real misspecifications appear, with escaped counts exposing gaps. | H-27:L163-L166 |
| H27-E031 | E/FAIL | The non-defaultable sidedness declaration must apply to every band gate, including H-18/H-22 and H-21/H-26 bounds; omission fails closed. | H-27:L167-L168 |
| H27-E032 | E/FLOW | The meta-gate itself must self-validate using a fixed bootstrap corpus frozen first; the bootstrap must be documented. | H-27:L169-L170 |
| H27-E033 | E/DEP | H-27 validates gates from H-18/H-20/H-21/H-22/H-26 and composes with H-28 as the meta-layer for optimizer trust. | H-27:L20-L21, L172-L176 |
| H27-E034 | E/STATUS | Estimated effort is 6–8 hours, wholly Tier-1/in-process, with no LLM verdict. | H-27:L16-L16 |
| H27-I001 | I/SEC | “Human-authored out-of-band” implies restricted write authority for corpus files, but the source's enforcement mechanism is only gate-manifest freezing, not a named ACL/role system. | H-27:L56-L60, L75-L76 |
| H27-A001 | A/LIFECYCLE | The objective says `PROPOSED -> ACTIVE`, while concrete integration hooks `SHADOW -> ACTIVE` in the shipped `PROPOSED -> CANDIDATE -> SHADOW -> ACTIVE` lifecycle. Treat the former as shorthand unless implementation proves otherwise. | H-27:L8-L8, L28-L29, L41-L42, L94-L96 |
| H27-A002 | A/CFG | A kill switch is permitted but its identifier is not named. | H-27:L122-L122, L131-L131 |
| H27-A003 | A/TEST | “100% of a frozen corpus” is exact for the corpus but corpus completeness has no closure criterion; the source explicitly leaves it evolutionary. | H-27:L56-L64, L163-L166 |

Category coverage: capabilities/actions/surfaces ✓; workflows/states/lifecycle ✓; schemas/data ✓; flags/defaults/config ✓; integrations/events ✓; security/compliance ✓; failure/fail-closed/recovery counter ✓; determinism ✓; observability/evidence ✓; tests/done gates ✓; constraints/non-goals ✓; dependencies/status ✓. No retry or concurrency model is specified; idempotency is only implicit in append-only evidence and is not promoted to a requirement.

### H-43 — execution-grounded DGM fitness

Source: `docs/thefinalglance/H-plan/H-43_execution_grounded_dgm_fitness.md` (211/211 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| H43-E001 | E/STATUS | `H-43` is PLANNED Track-7/Wave-7 keystone, tiered into an in-process scorer/SSB row and a separately gated live multi-generation bench. | H-43:L1-L3 |
| H43-E002 | E/STATE | Current `derived_alpha` maps the first eight bytes of `blake3(config_blob)` into `[0,1]`; it is deterministic but causally unrelated to candidate behavior. | H-43:L5-L14, L42-L48 |
| H43-E003 | E/CAP | Replace byte-derived fitness with execution-grounded fitness combining CodeRM-style generated-test pass rate and CodeT cross-solution dual agreement. | H-43:L11-L14, L28-L40 |
| H43-E004 | E/STATUS | Tier-1 compiles candidates, runs a frozen deterministic test set, emits measured fitness, replaces `derived_alpha`, and fails closed; Tier-2 runs the real local-Qwen/RunPod multigeneration bench later. | H-43:L16-L19 |
| H43-E005 | E/DEP | No hard prerequisite beyond the shipped loop/frozen-runner pattern; D-2/E-19/E-53 are composed upgrade or proof-ceiling paths rather than reauthored. | H-43:L20-L26 |
| H43-E006 | E/DEP | H-43 runs first in Track 7, is parallel-safe with H-44, blocks H-45/H-46/H-48, enables H-39's `correct` bit, and makes E-19 Task 11 runnable. | H-43:L23-L24, L193-L197 |
| H43-E007 | E/DEP | Mirror the shipped `safedelta_runner` frozen-runner/measured-row shape; do not create a second runner architecture. | H-43:L42-L48, L95-L99 |
| H43-E008 | E/DATA | `ExecutionFitness` returns `pass_rate: f64`, `tests_total`, `tests_passed`, and `dual_agreement: f64`. | H-43:L60-L64 |
| H43-E009 | E/FLOW | `execution_grounded_fitness` compiles candidate-produced code and evaluates it against a frozen, out-of-band generated-test set. | H-43:L60-L65 |
| H43-E010 | E/CFG | The declared formula is `fitness = w_pass*pass_rate + w_agree*dual_agreement`; default weights are 0.7/0.3 and are LOCKED against runtime/self-mod tuning. | H-43:L62-L65 |
| H43-E011 | E/DET | Scoring is deterministic under `ELAI_SEED` and may not use wall clock. | H-43:L64-L65 |
| H43-E012 | E/FLOW | `derived_alpha` keeps its signature and `force_alpha` fixture override but calls the real scorer; the production blake3-of-config-bytes path is deleted. | H-43:L67-L70 |
| H43-E013 | E/DET | blake3 may remain only for the frozen test-set/fixture hash, never as the production fitness signal. | H-43:L67-L70, L102-L107 |
| H43-E014 | E/SEC | Pure compile/run execution is family-free and default; any optional model-graded sub-signal must assert verifier family differs from generator family. | H-43:L72-L74 |
| H43-E015 | E/OBS | Emit one `FitnessEvaluated` row with `{pass_rate, dual_agreement, tests_total, fitness, seed, test_set_hash}`. | H-43:L76-L79 |
| H43-E016 | E/DATA | `FitnessEvaluated` kinds are `ExecutionPassRate`, `DualAgreement`, and `FitnessRegression`; it is Pattern-2 baseline+1 from the live schema and the first Track-7 bump repairs the stale e45 pin. | H-43:L76-L79 |
| H43-E017 | E/FLOW | If child execution fitness is below the parent's, emit `FitnessRegression`; the existing two-gate negative margin blocks it. | H-43:L81-L83 |
| H43-E018 | E/DATA | `H-43-decisions.md` records bytes-to-execution rationale, locked weights, D-2/E-19 upgrade path, and the frozen/out-of-band test-set honesty limit. | H-43:L85-L88 |
| H43-E019 | E/INT | Produced symbols are `execution_grounded_fitness`, `ExecutionFitness`, and `SsbActionType::FitnessEvaluated`. | H-43:L90-L94 |
| H43-E020 | E/INT | Consumed symbols are `derived_alpha`, `run_one_generation`, the safedelta pattern, and `two_gate_promote`; none may be rewritten wholesale. | H-43:L95-L99 |
| H43-E021 | E/FLOW | Live reachability is `run_one_generation -> derived_alpha -> execution_grounded_fitness -> FitnessEvaluated`. | H-43:L100-L101 |
| H43-E022 | E/CFG | The production path remains behind existing `cfg(feature="darwinian-evolve")` plus `ELAI_DARWINIAN_LOOP_ENABLE`; this registered/allowlisted kill switch is not a terminal new default-off gate. | H-43:L100-L101, L128-L137 |
| H43-E023 | E/FAIL | Production byte-hash fitness, same-family self-grading, and agent-tunable weights are explicit failures. | H-43:L102-L107 |
| H43-E024 | E/TEST | The `--fitness-from-bytes` broken arm must make the failing candidate score high/invert the ranking; a metric unchanged by the broken arm fails validation. | H-43:L102-L107, L111-L117 |
| H43-E025 | E/DET | `execution_fitness.rs` permits no blake2/sha2 and no production `.unwrap()`/`.expect()`. | H-43:L102-L107 |
| H43-E026 | E/INT | The DGM/archive drawer displays per-candidate pass rate and fitness; cross-language checks and visible failing-candidate loss are required. | H-43:L108-L110 |
| H43-E027 | E/TEST | The seeded live trigger evaluates frozen `C_good` and `C_bad`; two rows land, `C_good > C_bad`, and `C_bad` carries `FitnessRegression`. | H-43:L111-L117 |
| H43-E028 | E/DET | `{pass_rate, dual_agreement, fitness}` must recompute byte-identically from journal evidence under `ELAI_SEED`. | H-43:L111-L117, L128-L137 |
| H43-E029 | E/TEST | `wiring_contract_check --run-trigger` must show scorer-to-production reachability and rows; cargo-green alone is insufficient. | H-43:L119-L138 |
| H43-E030 | E/TEST | Contract arms cover good>bad, monotonic pass rate, equal score for equal behavior/different bytes, broken-arm inversion, immutable weights, same-seed equality, and child regression. | H-43:L153-L159 |
| H43-E031 | E/DATA | Frozen fixtures include `C_good`, `C_bad`, the generated-test set, and its hash. | H-43:L153-L159 |
| H43-E032 | E/DATA | Expected artifacts create scorer/contracts/fixtures/decision doc, modify `derived_alpha` and SSB locks, and add archive UI/type fields. | H-43:L161-L167 |
| H43-E033 | E/SEC | A candidate may not author its own evaluating tests; Tier-1 tests are frozen/out-of-band and Tier-2 test generation must use a separate model/family. | H-43:L181-L184 |
| H43-E034 | E/LIMIT | Tier-1 fixture pass rate proves causal wiring, not the production capability number; meaningful live measurement waits for Tier-2 and cannot be advertised early. | H-43:L185-L187 |
| H43-E035 | E/CFG | If 0.7/0.3 proves noisy, weights may change only through a frozen calibration set with a documented calibration date, never at runtime. | H-43:L188-L189 |
| H43-E036 | E/LIMIT | Execution grounding does not catch semantically correct but spec-violating code; H-44 and E-53 cover behavioral/proof residue. | H-43:L190-L191 |
| H43-E037 | E/STATUS | Estimated effort is 8–10 hours; the live multigeneration number is explicitly not a Tier-1 deliverable. | H-43:L16-L19 |
| H43-E038 | E/DEP | The J-11 amendment cites Arbor's merge gate as prior-art existence proof, not a component to copy or a second fitness owner. | H-43:L199-L209 |
| H43-E039 | E/FLOW | Arbor's cited pattern evaluates held-out `B_test` in a detached worktree/separate pool and uses `verified_score`, while the model-reported score is logged only for comparison. | H-43:L200-L205 |
| H43-E040 | E/FAIL | The cited Arbor gate rejects candidates with no improvement versus trunk/baseline. | H-43:L203-L205 |
| H43-E041 | E/LIMIT | H-43 owns the fitness function; Arbor is only support for “verified execution score beats LLM-reported score.” | H-43:L205-L209 |
| H43-I001 | I/DATA | The clean executable contract is the two-component 0.7/0.3 formula in Task 1; the amendment's extra term cannot be implemented without a further authored decision. | H-43:L60-L65, L205-L209 |
| H43-A001 | A/DATA | The J-11 amendment says scoring includes `dual_agreement + fixture_pass_rate + convergence_delta` while retaining only two locked weights (0.7/0.3); it neither defines `convergence_delta` nor supplies a third weight. | H-43:L60-L65, L205-L209 |
| H43-A002 | A/CFG | Initial weights are called LOCKED yet may later be tuned via a frozen calibration set; the lifecycle/versioning authority for such a change is not specified. | H-43:L62-L65, L188-L189 |
| H43-A003 | A/TEST | The exact generated-test count, timeout/sandbox behavior, and compile/run failure-to-score mapping are unspecified. | H-43:L60-L65, L153-L159 |
| H43-C001 | C/DATA | The primary task defines a two-term fitness; the source-cited amendment describes a three-signal fitness with the same two weights. These are internally conflicting formula descriptions and must not be silently merged. | H-43:L60-L65, L205-L209 |

Category coverage: capabilities/actions/surfaces ✓; workflows/states ✓; schema/data ✓; flags/defaults/config ✓; integrations/events ✓; security/compliance ✓; failure/fallback ✓; determinism ✓; observability/evidence ✓; tests/done gates ✓; constraints/non-goals ✓; dependencies/status/deferred ✓. No explicit retry, recovery, concurrency, idempotency, or end-user permission model is specified.

### H-49 — undo/redo/branching file time-travel and lossless history

Source: `docs/thefinalglance/H-plan/H-49_undo_redo_branching_timetravel.md` (101/101 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| H49-E001 | E/STATUS | `H-49` is Track 8/Wave 8, strong-grounding, L-effort BUILD-on-shipped: a CAS layer over E-25 checkpoints and undo-tree HEAD over `SessionDag`. | H-49:L1-L2 |
| H49-E002 | E/DEP | It depends on H-35, E-25, and H-6 and composes G-2 RRA, E-29 transcript, C-23 BranchTree, and E-44 saga compensation. | H-49:L2-L3 |
| H49-E003 | E/CAP | Every LLM file change is reversible and re-applicable at any point; undo must preserve all timelines. | H-49:L3-L3 |
| H49-E004 | E/CAP | Session history is lossless and paged so the first message remains reachable in arbitrarily long sessions. | H-49:L3-L3 |
| H49-E005 | E/LIMIT | Snapshot restore, not deterministic/model replay, is the load-bearing file mechanism because it is byte-based and cloud/model agnostic. | H-49:L3-L3 |
| H49-E006 | E/LIMIT | Cursor's agent-edit-only checkpoint model can clobber manual edits; H-49 adopts RV-1 pre-restore capture to prevent that. | H-49:L5-L9 |
| H49-E007 | E/LIMIT | Shadow-git is optional only: prior art can disable outside git and lose nested-git data, so ELAI CAS is the default. | H-49:L7-L10 |
| H49-E008 | E/DEP | The shipped byte-exact snapshot/restore engine must be wrapped, not replaced or reauthored. | H-49:L11-L11 |
| H49-E009 | E/STATE | `write_file` and `edit_file` already capture pre-image `original_file`, structured patch, and git diff before write, but discard the pre-image after transient tool-result JSON. | H-49:L13-L16 |
| H49-E010 | E/STATE | `edit_checkpoints::snapshot_pre_edit` and `rewind_to` already perform full-byte snapshots/restores and highest-sequence-first restore with a passing test. | H-49:L16-L16 |
| H49-E011 | E/STATE | Existing E-25 autosnapshot is default-off via `ELAI_E25_EDIT_CHECKPOINTS_ENABLE`, scrapes a lossy task summary lacking structured target path, stores full copies, and writes files non-atomically. | H-49:L16-L16 |
| H49-E012 | E/STATE | Existing `checkpoint list` / `checkpoint rewind [--dry-run][--steps N]` CLI is live but only restores whatever the gated heuristic captured. | H-49:L16-L16 |
| H49-E013 | E/STATE | `SessionDag` already has parent-linked snapshots, message context, save/load, branches, merge result, parent chain, and lazy ALTER migration; it stores only a code-change summary, not file bytes. | H-49:L17-L17 |
| H49-E014 | E/STATE | `FileWritePayload {path,size_bytes,hash}` is metadata-only, has zero production emitters, and a comment claiming a live diff is stale. SSB remains an index; CAS owns bytes. | H-49:L18-L18 |
| H49-E015 | E/STATE | H-35 supplies append-only ASC journal, compaction, replay/RRA and `SessionRewound`; `recover_stubs` is blake3-verified but production-dark and live readback returns `[stubbed]`. | H-49:L19-L19 |
| H49-E016 | E/STATE | Transcript UI/backend is capped to the latest 50 with no pagination; CLI resume prints only a banner; handoff drops older messages behind an elision sentinel; E-29 lacks cursor paging/virtual scroll. | H-49:L20-L20 |
| H49-E017 | E/DATA | CAS blobs live at `.elai_cc/.session-checkpoints/{sid}/objects/{blake3[..2]}/{blake3}`, are zstd-compressed, and use blake3 content keys. | H-49:L22-L24 |
| H49-E018 | E/DATA | `FileTree` maps project-relative paths to blob hashes and is itself stored at `manifests/{blake3}`. | H-49:L23-L24 |
| H49-E019 | E/DET | CAS insertion is hash-idempotent and deduplicates unchanged/reverted contents, making default-on capture affordable and retiring the immediate full-copy eviction question. | H-49:L24-L24, L27-L27 |
| H49-E020 | E/DATA | Ship `RestoreEngine { capture(paths)->Manifest, materialize(target) }` with config `[timetravel] engine = "cas" | "shadow_git"`; `CasRestoreEngine` is default and `ShadowGitRestoreEngine` is Tier-2 stubbed behind the selector. | H-49:L25-L25 |
| H49-E021 | E/SCHEMA | Add `file_manifest` and per-session `HEAD` reference columns to `snapshots` via lazy ALTER; each node pins conversation context plus its file-tree manifest. | H-49:L26-L26, L35-L37 |
| H49-E022 | E/FLOW | PostToolUse at the live file-write row site consumes `original_file` and post-image, persists new blobs, creates a child snapshot at grouping boundary, repoints the manifest, and advances HEAD. | H-49:L27-L27, L35-L38 |
| H49-E023 | E/CFG | Capture is default-on behind opt-out `ELAI_TIMETRAVEL_DISABLE` and replaces the default-off E-25 summary-scrape path. | H-49:L27-L27 |
| H49-E024 | E/FLOW | `undo` moves HEAD to parent and materializes that manifest. | H-49:L28-L28 |
| H49-E025 | E/FLOW | `redo` moves HEAD to a child using the last-followed edge; multiple children require a user prompt. | H-49:L28-L28 |
| H49-E026 | E/FLOW | Editing after undo creates a new child/sibling and preserves the old redo child. | H-49:L28-L28 |
| H49-E027 | E/RECOVERY | Before materialization, if on-disk blake3 differs from HEAD's expected hash, snapshot current bytes as an `auto:pre-restore` fork, then restore. | H-49:L28-L28 |
| H49-E028 | E/RECOVERY | Materialization uses temp file + fsync + rename, replacing non-atomic direct writes. | H-49:L28-L28 |
| H49-E029 | E/CFG | User-selectable grain is `edit|turn|babystep`, default `turn`; capture always remains per edit, and grain controls ancestor traversal as a view over one tree. | H-49:L29-L29 |
| H49-E030 | E/CAP | CLI adds `elai undo`/`redo` with `--grain`, `--scope code|conversation|both`, `--steps`, `--dry-run`, plus `history`/`timeline` and `jump <node|--turn N|"before baby step X">`. | H-49:L30-L30 |
| H49-E031 | E/INT | `elai rewind` aliases labeled-target undo, and `elai checkpoint rewind` becomes a thin wrapper over the same engine. | H-49:L30-L30 |
| H49-E032 | E/SEC | `--dry-run` changes no disk and reports path deltas plus RV-4 irreversible-side-effect warnings. | H-49:L30-L30 |
| H49-E033 | E/RECOVERY | Archival is a precondition of trimming/stubbing: a message may be dropped only if `recover_stubs` can recover it; H-6 wires live readback. | H-49:L31-L31 |
| H49-E034 | E/DATA | Extend `ssb_transcript`/`get_chat_history` with `before_seq|after_seq`, `limit`, `total_count`, and `first_seq`. | H-49:L31-L31 |
| H49-E035 | E/CAP | Replace the 50-entry transcript with virtual-window rendering, scroll-up loading, “Jump to start” via `first_seq`, and CLI `elai history --replay`. | H-49:L31-L31 |
| H49-E036 | E/CAP | Dashboard extends `BranchTree.svelte` into a timeline with per-node file-manifest diffs, Redo, multi-child indicators, and Files-Only versus Files+Conversation restore. | H-49:L32-L32 |
| H49-E037 | E/DEFER | Tier-2 defers full shadow-git plumbing, multi-checkpoint visual diff, Canvas/DESIGN op-stream time travel, conflict resolution beyond fail-closed refusal, and measured GC retention tuning. | H-49:L33-L33 |
| H49-E038 | E/FAIL | A configurable per-blob cap around 5 MB stores only a pointer and explicit “not snapshotted, too large” state; it must never pretend the content was captured. | H-49:L33-L33 |
| H49-E039 | E/INT | `architecture/timetravel.rs` owns CAS, `FileTree`, restore trait/implementations, HEAD/tree traversal, and RV-1 while wrapping `snapshot_pre_edit`/`rewind_to`. | H-49:L35-L37 |
| H49-E040 | E/INT | `session_dag.rs` adds `set_head`, `get_head`, and `children_of(node)` alongside lazy schema columns. | H-49:L37-L37 |
| H49-E041 | E/INT | CLI adds `timetravel_cmd.rs`, `CliAction::{Undo,Redo,History,Jump}`, parsers, and main dispatch; PostToolUse provides the real edit producer. | H-49:L38-L38 |
| H49-E042 | E/INT | Tauri adds `timetravel_undo/redo/jump(session_id,target,grain,scope)` plus paginated commands in `commands_h49.rs`; CLI/dashboard share one dispatcher and done gate. | H-49:L39-L39 |
| H49-E043 | E/OBS | Reuse `SessionRewound` for undo/restore; add one Pattern-2 `EditHistoryEvent` with kinds `SnapshotCaptured`, `Deduped`, `Redo`, `Branched`, `Jumped`, `TooLargeSkipped`. | H-49:L40-L40 |
| H49-E044 | E/DATA | `EditHistoryEvent` payload contains `{node_id,parent_id,paths_n,pre_hash?,post_hash?,kind,grain}`; bytes stay in CAS and are referenced by hash. | H-49:L42-L44 |
| H49-E045 | E/OBS | Capture emits SnapshotCaptured/Deduped; undo emits SessionRewound; redo/branch/jump emit EditHistoryEvent. CAS scope may emit `replay_resume` only if shadow/replay is selected. | H-49:L40-L43 |
| H49-E046 | E/DATA | The new event is baseline+1 and must follow nine-tuple enum/array/SQL/parser lockstep and reconcile stale e45 if still needed. | H-49:L40-L40 |
| H49-E047 | E/INT | The contract consumes actual `WriteFileOutput.original_file`/`EditFileOutput.original_file`, checkpoint primitives, SessionDag branch/recovery APIs, ASC SSB stream, compaction, H-35, BranchTree, and E-29 transcript. | H-49:L43-L45 |
| H49-E048 | E/FLOW | Real entrypoints are CLI actions, live PostToolUse capture, and Tauri commands; tests/contracts alone do not satisfy reachability. | H-49:L45-L45 |
| H49-E049 | E/FAIL | Placeholders, empty/hardcoded blobs, lying no-op redo, silent RV-1 clobber, `.unwrap()` in dispatcher, and stuffing content into `FileWritePayload` are banned. | H-49:L46-L46 |
| H49-E050 | E/TEST | Live edit must emit off-agent SnapshotCaptured and store a blake3-verifying pre-image blob. | H-49:L47-L47 |
| H49-E051 | E/TEST | `undo --scope code` must restore exact pre-hash and emit `SessionRewound` with matching `head_seq_before`. | H-49:L47-L47 |
| H49-E052 | E/TEST | `redo` must restore exact post-hash and emit `EditHistoryEvent kind=Redo`. | H-49:L47-L47 |
| H49-E053 | E/TEST | Edit-after-undo must yield two children and `Branched`, proving the prior timeline remains. | H-49:L47-L47 |
| H49-E054 | E/TEST | Manual drift followed by undo must first emit/capture `auto:pre-restore`; absence is failure. | H-49:L47-L47 |
| H49-E055 | E/OBS | A true no-op where HEAD equals target emits zero rows; smoke manifests distinguish no-op from a dead path. | H-49:L47-L47 |
| H49-E056 | E/OBS | Divergence-guard false-fork rate must be measured/bounded across at least three real runs before default-on. | H-49:L47-L47 |
| H49-E057 | E/LIMIT | File undo covers local and cloud models but cannot undo external side effects (network, external deletes, spend, push/deploy/sent messages). | H-49:L49-L52 |
| H49-E058 | E/OBS | E-44 `compensation_actions` supplies a warning count for irreversible actions after a point; H-49 never auto-reverts them or calls them undoable. | H-49:L52-L52, L54-L60 |
| H49-E059 | E/PRIVACY | Time travel stores work facts, not persona/PII; event rows carry hashes/node ids/paths, never transcript bytes. | H-49:L52-L52 |
| H49-E060 | E/DEP | H-49 delivers H-35's branch/fork, redo, and cloud code restore; wraps E-25 while replacing its flag/heuristic; shares H-6 recovery; extends E-29/C-23; aligns F-13 schema. | H-49:L54-L62 |
| H49-E061 | E/DET | J-15 requires fork points, payload node/parent ids, and HEAD to key on stable SSB `seq`/`event_id`, never positional row indices that change under replay ordering. | H-49:L64-L80 |
| H49-E062 | E/TEST | Add an LLM-free, auditor-only mutate-then-replay rider: replace exactly one recorded response in a J-14 content-hash fixture and deterministically replay downstream to exercise divergence/fork capture. | H-49:L82-L93 |
| H49-E063 | E/SEC | CAS, manifests, and events must never contain auth headers, credentials, or secrets; J-14 export scrubs auth and H-49 inherits that scrub. | H-49:L95-L101 |
| H49-I001 | I/DET | Since HEAD/node ids must be event-id-derived, snapshot creation needs a deterministic event-id derivation shared across CLI and Tauri; the specific derivation function is not named. | H-49:L70-L80 |
| H49-I002 | I/RECOVERY | A `ShadowGitRestoreEngine` selector can safely exist in Tier-1 only if selecting the unimplemented backend returns an explicit unavailable error rather than masquerading as restoration; the exact error is inferred. | H-49:L25-L25, L33-L33, L46-L46 |
| H49-A001 | A/CFG | The blob cap is only approximate (“~5MB”); its exact default, pointer schema, and user override bounds are unresolved. | H-49:L33-L33 |
| H49-A002 | A/FAIL | Tier-1 ships a `ShadowGitRestoreEngine` “stub,” while the contract bans placeholder restore. The intended distinction (explicit unsupported backend versus callable stub) must be resolved. | H-49:L25-L25, L33-L33, L46-L46 |
| H49-A003 | A/DATA | The plan alternates between “per-session HEAD ref column” and `file_manifest + head` columns on snapshot nodes; exact relational ownership/cardinality of HEAD requires schema design. | H-49:L26-L26, L37-L37, L70-L80 |
| H49-A004 | A/FLOW | Redo says “last-followed edge” and “prompt on branch ambiguity” but does not specify persistence of last-followed choice or headless-CLI behavior. | H-49:L28-L30 |

Category coverage: capabilities/actions/surfaces ✓; workflows/states/lifecycle ✓; schemas/data ✓; flags/defaults/config ✓; integrations/events ✓; permissions/security/privacy ✓; failures/fallback/recovery ✓; determinism/idempotency ✓; observability/evidence ✓; tests/done gates ✓; constraints/non-goals ✓; dependencies/status/deferred ✓. Concurrency is addressed only through atomic materialization; no lock/multiclient conflict protocol or retry schedule is specified.

### H-53 — citation pre-binding and constrained synthesis

Source: `docs/thefinalglance/H-plan/H-53_citation_prebind_constrained_synthesis.md` (61/61 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| H53-E001 | E/STATUS | `H-53` is Track 11/Wave 11, M-L BUILD, strong on ELAI's own components but only medium-grounded on Perplexity-specific mechanics. | H-53:L1-L2 |
| H53-E002 | E/STATE | Current `/research`/`/ask`/researcher output emits freeform `[N]`; E-48 and Phase 2B detect ghost/misattributed cites after generation rather than prebinding them. | H-53:L3-L3, L13-L20 |
| H53-E003 | E/FLOW | After retrieval/rerank, freeze the final ranked evidence before synthesis as numbered, closed-set anchors known before the first decoded token. | H-53:L5-L8 |
| H53-E004 | E/DATA | External-shape anchor data is `{id -> (verbatim_excerpt, excerpt_blake3, source_uri, retrieved_at, source_tier)}`. | H-53:L6-L8 |
| H53-E005 | E/DET | Anchor ids are assigned in rank order under `ELAI_SEED`; identical query/seed must yield byte-identical `[N]` numbering. | H-53:L23-L26 |
| H53-E006 | E/DATA | Tier-1 types are `EvidenceAnchorSet { anchors: Vec<EvidenceAnchor> }` and `EvidenceAnchor { id:u32, excerpt:String, excerpt_blake3:[u8;32], source_uri:String, retrieved_at_ms:u64, source_tier:u8 }`. | H-53:L23-L26 |
| H53-E007 | E/DET | Excerpts are pinned with blake3 only; blake2/sha2 are banned. | H-53:L23-L26, L41-L45 |
| H53-E008 | E/DEP | Retrieval/ranking is explicitly not in scope: the shipped hybrid/RRF/rerank/A-RAG/tier/freshness stack remains unchanged. | H-53:L13-L16 |
| H53-E009 | E/FLOW | Insert prebinding after A-RAG/rerank convergence and before synthesis on `/research`, `/ask`, researcher-oracle, and E-48 web-research paths. | H-53:L33-L36, L41-L44 |
| H53-E010 | E/DATA | Inject the volatile Evidence slot after the last cache breakpoint; coordinate per-backend budget with H-50 so it does not bust the stable prefix. | H-53:L17-L18, L23-L26, L33-L35 |
| H53-E011 | E/CAP | Tier-1 checks every model `[N]` for membership in the frozen set and support by its blake3-pinned excerpt, reusing E-48 verifier logic. | H-53:L9-L10, L23-L27 |
| H53-E012 | E/FAIL | An out-of-set id or unsupported span is a ghost: hold the answer and route to D-16 refusal/repair; never ship the ghost or invent an anchor. | H-53:L10-L11, L23-L27 |
| H53-E013 | E/DEP | Reuse H-52 `anchor_finding` normalize/consecutive-match/Ambiguous guard for excerpt binding; do not author a second resolver. | H-53:L9-L9, L19-L19, L33-L34 |
| H53-E014 | E/DEP | Reuse/extend E-48 CitationVerifier and `citation_verified` for closed-set membership+support; do not author a second verifier. | H-53:L10-L10, L23-L27, L33-L38 |
| H53-E015 | E/DEP | Read D-16's existing conformal abstain bit and D-17 refusal SLA; do not rederive refusal. | H-53:L11-L11, L20-L20 |
| H53-E016 | E/OBS | Prefer one reused/extended `citation_verified` row with `anchor_id:u32` and `pre_bound:bool`; add one Pattern-2 `CitationAnchorsBound` only if distinct prebinding evidence is truly required. | H-53:L23-L28, L33-L38 |
| H53-E017 | E/DATA | If `CitationAnchorsBound` is added it is baseline+1 from the freshly read live schema and follows the nine-tuple golden lock; otherwise no new variant is added. | H-53:L27-L27, L38-L38 |
| H53-E018 | E/CFG | Tier-1 prebinding is default-on behind opt-out `ELAI_CITATION_PREBIND_DISABLE`, never a default-off enable gate. | H-53:L28-L28 |
| H53-E019 | E/DEFER | Tier-2 defers E-35/C-26 citation-marker constrained decoding that mechanically makes out-of-set ids unemittable. | H-53:L29-L30 |
| H53-E020 | E/FAIL | Tier-2 grammar constrains only citation-marker positions, never prose/reasoning; unavailable grammar causes an explicit `grammar_unavailable` FreeText fallback and the grammar does not run on incapable backends. | H-53:L7-L8, L29-L30, L36-L37 |
| H53-E021 | E/DEFER | Final local-window cost tuning is Tier-2; the 25–32K local lane gets a smaller Evidence slot while cloud's 200K profile can absorb more. | H-53:L31-L31 |
| H53-E022 | E/INT | Create `citation_prebind.rs` with types and `freeze_anchors`, export it from `lib.rs`, and add the Evidence slot to ContextBudget. | H-53:L33-L35 |
| H53-E023 | E/INT | E-48 receives one additive verifier touch; the deferred decode layer extends `PER_ROLE_DECODING`/XGrammar rather than replacing it. | H-53:L35-L38 |
| H53-E024 | E/CAP | Existing research UI renders every `[N]` with its pre-bound anchor and source tier; unsupported spans render explicit refusal. | H-53:L39-L39 |
| H53-E025 | E/FLOW | Every cited answer produces a closed-set verification result; a no-citation answer produces zero binding rows. | H-53:L41-L46 |
| H53-E026 | E/FAIL | Hard bans: accepting an out-of-set cite, shipping ungroundable load-bearing claims, nondeterministic ids, grammar over prose, `.unwrap()`/`.expect()` in the pre-binder, or non-blake3 hashing. | H-53:L45-L45 |
| H53-E027 | E/TEST | Seeded `/research --mode cloud` fixture contains answerable and unanswerable subclaims; every emitted `[N]` must resolve to a supporting pinned anchor with `pre_bound=true`. | H-53:L46-L46 |
| H53-E028 | E/TEST | An out-of-set cite must be held and the unanswerable subclaim must refuse rather than invent a citation. | H-53:L46-L46 |
| H53-E029 | E/TEST | Repeating the query/seed must yield byte-identical citation numbering. | H-53:L46-L46 |
| H53-E030 | E/TEST | `--allow-freeform-citations` is the broken arm: skipping closed-set verification lets a ghost ship and must make the gate fail. | H-53:L46-L46 |
| H53-E031 | E/OBS | Refusal/over-hold rate must be measured/bounded on at least three real answer runs and checked against D-17's healthy band before default-on. | H-53:L46-L46 |
| H53-E032 | E/LIMIT | Perplexity's exact staged mechanism is analyst-reported, single-source, unofficial, and medium confidence; only its shape is borrowed and no Perplexity number may be presented as ELAI's. | H-53:L5-L8, L48-L50 |
| H53-E033 | E/LIMIT | Prebinding hardens citation-to-retrieved-evidence linkage, not truth of evidence; source quality remains retrieval/tier/freshness, and novel claims route to D-15 plus human review. | H-53:L48-L51 |
| H53-E034 | E/OBS | Publish only ELAI's measured ghost-citation rate and refusal band, journal-recomputable under `ELAI_SEED`; the honest claim is “can't cite what it didn't retrieve,” not general “can't lie.” | H-53:L51-L51 |
| H53-E035 | E/DEP | The full composition is E-48 closed-set consumer, H-52 resolver, E-35/C-26 deferred grammar, D-16/D-17 refusal, and Phase 3E/H-50 cache/window budgeting. | H-53:L53-L59 |
| H53-E036 | E/STATUS | The immediate vertical slice is deterministic prebind+closed-set verification; constrained decoding and final window tuning remain separately gated follow-ons. | H-53:L23-L31 |
| H53-I001 | I/SEC | Evidence anchors contain full source URIs and excerpts, so access/redaction policy may be needed for private sources, but H-53 states no such permission/privacy rule. | H-53:L6-L8, L23-L26 |
| H53-A001 | A/INT | Whether prebinding gets a distinct `CitationAnchorsBound` event or only extends `citation_verified` is explicitly deferred to the implementer at edit time. | H-53:L27-L27, L38-L38 |
| H53-A002 | A/FALLBACK | Tier-2 explicitly falls back to FreeText when grammar is unavailable, but Tier-1 closed-set verification still blocks ghosts; the source does not specify how the UI distinguishes degraded prevention from normal operation. | H-53:L17-L17, L29-L30, L36-L39 |
| H53-A003 | A/DATA | The exact Evidence-slot token budget, truncation policy, and number/length of anchors per backend are not specified. | H-53:L18-L18, L25-L25, L31-L31 |
| H53-C001 | C/CAP | The title/premise calls ghost citations “structurally impossible,” but SHIP-NOW still lets the model emit a ghost and catches/holds it after synthesis; only deferred Tier-2 grammar makes out-of-set markers mechanically unemittable. The current claim must be qualified. | H-53:L1-L3, L23-L30, L46-L46 |

Category coverage: capabilities/actions/surfaces ✓; workflows/states ✓; schemas/data ✓; flags/defaults/config ✓; integrations/events ✓; security/compliance ✓ but private-source authorization is unspecified; failures/fallback/refusal ✓; determinism ✓; observability/evidence ✓; tests/done gates ✓; constraints/non-goals ✓; dependencies/status/deferred ✓. No explicit retry, recovery, concurrency, or idempotency protocol is specified.

## Explicit interaction map

These are relational atoms stated by the assigned originals; they are not inferred architecture.

| Interaction ID | Explicit relationship | Source anchors / ledger atoms |
|---|---|---|
| IX-001 | H-1 unifies the marker path first; H-2 then makes placement byte-stable and adds tail/TTL policy. | H-1:L194-L197; H-2:L16-L19; H1-E032, H2-E007 |
| IX-002 | H-9 consumes H-1/H-2 measurement and owns their cockpit/E2E; H-1/H-2 do not independently own frontend rows. | H-1:L80-L83,L112-L114; H-2:L104-L105,L178-L181; H1-E014/H1-E020, H2-E022/H2-E034 |
| IX-003 | H-6 preserves H-2 caching by reducing only volatile trailing context. | H-2:L178-L181; H2-E034 |
| IX-004 | H-14 is downstream of H-13 Canvas operations/dispatcher and H-10/H-11/H-12 shell work; H-15 generalizes its reply channel and H-16 follows last. | H-14:L15-L17,L43-L44,L186-L188; H14-E013/H14-E037 |
| IX-005 | H-22 reads H-17's persistent N for DSR-vs-null and is recursively mutation-tested by H-27. | H-22:L18-L20,L29-L31,L76-L79; H22-E007/H22-E022 |
| IX-006 | H-22 shuffle and H-21 truncation are explicitly complementary leakage detectors. | H-22:L161-L163; H22-E022 |
| IX-007 | H-18's DSR-at-N, H-22's null band, and H-24's OOS/generalization spread are necessary but not sufficient for trusted edge. | H-22:L166-L168; H22-E038 |
| IX-008 | H-24 shares shipped replay with H-23, precedes H-25 result binding, and relies on H-28 to guard golden regeneration. | H-24:L16-L18,L149-L157; H24-E012/H24-E028/H24-E031 |
| IX-009 | H-27 validates the gate logic/corpora of H-18/H-20/H-21/H-22/H-26 and joins H-28 as the meta-trust layer. | H-27:L20-L21,L56-L60,L172-L176; H27-E008/H27-E033 |
| IX-010 | H-43 supplies H-39's correct bit, blocks H-45/H-46/H-48 until causal fitness exists, and enables E-19 Task 11. | H-43:L20-L24,L193-L197; H43-E005/H43-E006 |
| IX-011 | D-2 may later add cross-family step scoring and E-53 supplies a narrow proof ceiling; neither is reauthored by H-43. | H-43:L20-L22,L85-L88,L193-L197; H43-E005/H43-E018 |
| IX-012 | H-49 delivers H-35's deferred branch/redo/cloud-code restore while reusing `SessionRewound` and the rewind surface. | H-49:L54-L56; H49-E060 |
| IX-013 | H-49 wraps E-25's byte engine, replaces its default-off heuristic, and shares H-6 `recover_stubs` activation. | H-49:L56-L57; H49-E008/H49-E023/H49-E033/H49-E060 |
| IX-014 | H-49 extends E-29 transcript/C-23 BranchTree, uses E-44 only for irreversible warnings, retains G-2 verification, and aligns F-13 schema. | H-49:L58-L62; H49-E058/H49-E060 |
| IX-015 | H-53 inserts a precondition before E-48's verifier, reuses H-52's resolver, and delegates ungroundable spans to D-16/D-17. | H-53:L53-L57; H53-E013/H53-E014/H53-E015/H53-E035 |
| IX-016 | H-53 uses Phase-3E slots after the cache boundary and coordinates local/cloud budgeting with H-50; E-35/C-26 grammar is Tier-2. | H-53:L56-L59; H53-E010/H53-E019/H53-E021/H53-E035 |

## Summary-index additions and corrections

Comparison target: `docs/thefinalglance/feature-discovery/08-H-SUMMARY_INDEX.discovery.md`. “Add” means explicit detail absent or materially under-specified there; “Correct” means the summary statement must be qualified or removed. Atom references are authoritative and avoid restating/duplicating the source ledger.

### H-1 deltas

| Delta ID | Change | Atom references |
|---|---|---|
| SD-H1-001 | ADD the monitor's exact `<=0.05` verdict threshold, old critic flag/allowlist state, and live `CumulativeTokenTracker` source. | H1-E004–H1-E005 |
| SD-H1-002 | ADD the deliberate non-goal covering all 16 inert catalog `None` sites. | H1-E006 |
| SD-H1-003 | ADD pricing/scope honesty: caching discounts repeated input only and all ELAI claims must be measured/journal-recomputed. | H1-E007–H1-E008 |
| SD-H1-004 | ADD exact model-id threshold examples, live-model lookup, optional-event decision, and false-advertisement/logging failure rule. | H1-E011, H1-E014–H1-E018, H1-E030 |
| SD-H1-005 | ADD unmeasured RuntimeAdapter frequency/dollar impact, no hit-rate baseline, Tier-1 gate constraints, and conditional file/schema footprint. | H1-E029–H1-E034 |
| SD-H1-006 | CORRECT summary test `H1-012`: the assigned H-1 original contains no single-turn 1.25x-write-price acceptance arm. Keep that fact only if another cited source is named; it cannot be attributed to H-1. | H-1 full read; no matching line; manifest-verified 199 lines |
| SD-H1-007 | PRESERVE AS UNRESOLVED, not resolved fact: later-summary H-50 refutes the RuntimeAdapter split-brain, while the H-1 original asserts it. | H1-C001 |

### H-2 deltas

| Delta ID | Change | Atom references |
|---|---|---|
| SD-H2-001 | ADD four-breakpoint cap, final use-two/reserve-two policy, and long-loop exhaustion risk. | H2-E010–H2-E011, H2-E032 |
| SD-H2-002 | ADD conditional >5-minute TTL policy, 2x-write/read breakeven, refresh behavior, and prohibition on blanket 1h TTL. | H2-E012–H2-E013, H2-E033 |
| SD-H2-003 | ADD whole-prefix assertion scope, volatile-content placement, invalidation order, and exact per-model thresholds. | H2-E014–H2-E016 |
| SD-H2-004 | ADD no-new-action-type decision, Tier-1/no-network/no-dependency status, full contract arms, and H-5/H-6 composition. | H2-E022, H2-E028–H2-E034 |
| SD-H2-005 | FLAG unresolved final mapping of the two pre-existing breakpoints versus stable-prefix/tail positions and the unnamed kill switch. | H2-A001–H2-A002 |

### H-14 deltas

| Delta ID | Change | Atom references |
|---|---|---|
| SD-H14-001 | ADD the controlled-server-only CSP limit, React-19 `_debugSource` reason, collision-checked id semantics, exact verified dependencies, and anti-slopsquatting gate. | H14-E006–H14-E011 |
| SD-H14-002 | ADD spike-failure fallback: structural editing is descoped while deterministic style/text may ship; add token-reference-not-raw-values rule. | H14-E017–H14-E020 |
| SD-H14-003 | ADD explicit transport reuse/no-new-SSB rule, corrected off-agent event/artifact contract, and conditional RunPod structural checkbox. | H14-E024–H14-E030, H14-E038 |
| SD-H14-004 | ADD accessibility non-regression risk and native/Fiber reason RN is a separate slice. | H14-E035–H14-E036 |
| SD-H14-005 | CORRECT unqualified structural-edit acceptance: source permits a successful spike failure to descope structural work but supplies no alternate completion manifest. | H14-A001 |
| SD-H14-006 | FLAG unnamed mode flag and unmeasured a11y criterion. | H14-A002–H14-A003 |

### H-22 deltas

| Delta ID | Change | Atom references |
|---|---|---|
| SD-H22-001 | ADD Layer-4 “every strategy/every backtest commit” coverage, exact Tier-1/Tier-2 split, seeded/frozen sidecar, and `mutable=0 domain=finance`. | H22-E003, H22-E014–H22-E017 |
| SD-H22-002 | ADD exact `TradingNullTest` payload/e45 rules, live hooks, fail-open-advertising ban, and default-on kill-switch-only behavior. | H22-E018–H22-E019, H22-E024–H22-E031 |
| SD-H22-003 | ADD >=3-strategy FP calibration, existing-NumPy/no-new-dependency constraint, retail-size cost-law uncertainty, and necessary-not-sufficient limit. | H22-E021, H22-E035–H22-E039 |
| SD-H22-004 | FLAG unspecified relation tolerances/calibration and the Tier-1 fail-closed versus Tier-2-null-band sequencing gap. | H22-A001–H22-A003 |

### H-24 deltas

| Delta ID | Change | Atom references |
|---|---|---|
| SD-H24-001 | ADD exact adapter manifest fields, “logic differs only by adapter” invariant, missing-golden failure, and deterministic Part-B fixture stub. | H24-E005–H24-E010 |
| SD-H24-002 | ADD same-environment honesty, frozen golden/H-28 protection, execution-path transitive-import scope, and no-new-dependency gate. | H24-E024, H24-E026, H24-E028–H24-E030 |
| SD-H24-003 | ADD precise production hooks and schema/evidence payload plus joint H-23/H-25 lifecycle. | H24-E013–H24-E018, H24-E031 |
| SD-H24-004 | FLAG unspecified duplicate-function equivalence/language coverage and unnamed kill switch. | H24-A001–H24-A002 |

### H-27 deltas

| Delta ID | Change | Atom references |
|---|---|---|
| SD-H27-001 | ADD per-gate mutant examples/registration, gate-manifest edit blocking, append-only agent-unresettable counter, and exact event kinds. | H27-E008–H27-E015 |
| SD-H27-002 | ADD omission-fails-closed sidedness coverage, live `SHADOW->ACTIVE` chain, schema lockstep, and complete contract arms. | H27-E018–H27-E029, H27-E031 |
| SD-H27-003 | ADD corpus-completeness honesty and mandatory fixed bootstrap corpus for self-validating the meta-gate itself. | H27-E030, H27-E032 |
| SD-H27-004 | CORRECT/QUALIFY summary `PROPOSED -> ACTIVE`: concrete source lifecycle is `PROPOSED -> CANDIDATE -> SHADOW -> ACTIVE`, with H-27 hooked at `SHADOW -> ACTIVE`. | H27-A001 |
| SD-H27-005 | FLAG unnamed kill switch and lack of corpus-completeness closure. | H27-A002–H27-A003 |

### H-43 deltas

| Delta ID | Change | Atom references |
|---|---|---|
| SD-H43-001 | ADD exact `ExecutionFitness` fields, two-term 0.7/0.3 formula, no-wall-clock rule, and child-parent regression behavior. | H43-E008–H43-E017 |
| SD-H43-002 | ADD existing Darwin feature/env gate semantics, full event payload/schema repair, broken-arm definition, dashboard proof, and equal-behavior/different-bytes test. | H43-E015–H43-E031 |
| SD-H43-003 | ADD frozen/out-of-band and cross-family test-generation security, Tier-1-not-capability-number limit, frozen recalibration path, and spec-violation residue. | H43-E033–H43-E036 |
| SD-H43-004 | ADD J-11 Arbor prior-art amendment: detached held-out evaluation, verified-vs-reported score, no-improvement rejection, and ownership limit. | H43-E038–H43-E041 |
| SD-H43-005 | CORRECT/FLAG the fitness formula: primary task has two terms/weights, amendment names a third undefined `convergence_delta` while retaining two weights. | H43-I001, H43-A001, H43-C001 |
| SD-H43-006 | FLAG unclear locked-weight recalibration authority and unspecified compile/test sandbox/failure mapping. | H43-A002–H43-A003 |

### H-49 deltas

| Delta ID | Change | Atom references |
|---|---|---|
| SD-H49-001 | ADD exact shipped-state defects: discarded preimage, default-off lossy heuristic/full copies/non-atomic writes, metadata-only un-emitted FileWrite, production-dark recovery, and 50-row/lossy history. | H49-E009–H49-E016 |
| SD-H49-002 | ADD exact CAS paths/manifest, zstd/hash idempotency, RestoreEngine config, lazy schema, PostToolUse grouping, atomic materialization, and replacement of E-25 gating. | H49-E017–H49-E023, H49-E028 |
| SD-H49-003 | ADD branch-ambiguity prompt, exact grain semantics and CLI options/aliases/dry-run, archival-before-trim, cursor schema, virtual scroll, and dashboard restore scopes. | H49-E025, H49-E029–H49-E036 |
| SD-H49-004 | ADD deferred list, explicit too-large pointer state, event payload/conditional replay row, nine-tuple lock, real entrypoints, and hard stub bans. | H49-E037–H49-E049 |
| SD-H49-005 | ADD external-side-effect honesty, saga warning behavior, no-PII/transcript event policy, and full composition map. | H49-E057–H49-E060 |
| SD-H49-006 | ADD J-15 stable-event-id fork rule, deterministic mutate-then-replay auditor rider, and auth-header/credential scrub. | H49-E061–H49-E063 |
| SD-H49-007 | CORRECT summary's generic optional shadow-git statement: only trait+selector+stub ship Tier-1; functional shadow-git plumbing is deferred. | H49-E020, H49-E037, H49-A002 |
| SD-H49-008 | FLAG approximate blob cap, stub-vs-placeholder tension, HEAD schema ownership, and undefined last-followed/headless branch behavior. | H49-A001–H49-A004 |

### H-53 deltas

| Delta ID | Change | Atom references |
|---|---|---|
| SD-H53-001 | ADD exact anchor field types, rank/seed assignment rule, retrieval/ranking non-goal, and volatile Evidence-slot placement. | H53-E004–H53-E010 |
| SD-H53-002 | ADD exact Tier-1 failure/refusal behavior, no-reimplementation constraints, optional-event lock semantics, and no-citation zero-row behavior. | H53-E011–H53-E018, H53-E025–H53-E026 |
| SD-H53-003 | ADD marker-only grammar guard, explicit non-silent FreeText fallback/incapable-backend behavior, and local/cloud window figures. | H53-E019–H53-E021 |
| SD-H53-004 | ADD exact UI behavior, broken arm, >=3-run D-17 over-hold bound, unofficial-Perplexity limit, and honest “can't cite what it didn't retrieve” claim. | H53-E024, H53-E027–H53-E034 |
| SD-H53-005 | CORRECT headline/summary “structurally impossible” language for Tier-1: ship-now detects and holds ghosts after synthesis; only deferred grammar prevents emission. | H53-C001 |
| SD-H53-006 | FLAG optional SSB choice, degraded-mode UI ambiguity, unspecified anchor/window truncation, and absent private-source access/redaction policy. | H53-I001, H53-A001–H53-A003 |

## Exhaustiveness, zero-omission, and duplicate audit

- Source coverage: **9/9 files**, **1,452/1,452 logical lines**, **153,591/153,591 bytes** read and reconciled; source SHA mismatches **0**.
- Atomic source ledger: **388 unique atoms** = **350 explicit**, **10 inferred**, **25 ambiguous**, **3 conflicting/corrective**.
- Per-file atom counts: H-1 **38**; H-2 **37**; H-14 **42**; H-22 **43**; H-24 **34**; H-27 **38**; H-43 **46**; H-49 **69**; H-53 **41**. Sum **388**.
- Relational interaction records: **16** (`IX-001`–`IX-016`). Summary delta records: **51** (`SD-*`). These reference source atoms and are not double-counted as features.
- ID audit: extraction regex `^\| H(1|2|14|22|24|27|43|49|53)-[EIAC][0-9]{3} ` returns 388 rows and 388 unique values; duplicate atomic IDs **0**; missing/blank IDs **0**.
- Semantic duplicate audit: repeated objective/task/wiring/acceptance/test statements were collapsed into a single atom with multiple anchors. Interactions and summary deltas point back to atoms instead of cloning them. Duplicate feature atoms found **0**.
- Omission audit: every file was checked against all requested dimensions—capability/action/surface; workflow/state/lifecycle; schema/data; flag/default/config; integration/event; permission/security/privacy/compliance; failure/fallback/recovery; determinism/idempotency/concurrency; observability/evidence; test/acceptance/done gate; constraint/non-goal; dependency/status/deferred. A category absent from a source is called out below its ledger and was not invented.
- Interpretation audit: inferred/ambiguous/conflicting rows are separated from explicit rows; no inferred behavior is presented as a requirement. Summary-only statements are not imported as source facts; the unsupported H-1 1.25x test is explicitly corrected.
- Manifest boundary audit: the only original paths opened were the nine exact manifest rows in `DEEP-H-004`; orientation came only from `08-H-SUMMARY_INDEX.discovery.md`.
- Output metrics after finalization: `output_lines=0634`; `output_bytes=096505`.
- Canonical normalized output SHA-256: `72bee9bcc08ea614204d0bd5982bec42b39a4a9be075b6d8ef1e36dc83209de0`.
- Hash rule: compute SHA-256 over the exact final UTF-8 bytes (including final newline) after replacing only the 64 hexadecimal characters on the canonical normalized output SHA-256 line with 64 ASCII zeroes. Fixed-width output metrics and digest substitution make this self-verifiable without impossible raw self-reference.
