# DEEP-J-013 exhaustive feature discovery

## Scope, integrity, and method

- Processed manifest: `docs/thefinalglance/feature-discovery/DEEP-SOURCE-MANIFEST.md:L1071-L1086`, heading `### DEEP-J-013`; the next chunk starts at L1088.
- Declared chunk: family J, **10 files**, **1,220 logical lines**, **133,280 bytes**, all ten classified `numbered/index original`; target status **under 1,800**.
- Mechanical verification: fresh `sha256sum`, `wc -l`, and byte-size checks matched every manifest row and both aggregates exactly. Missing paths: **0**; unexpected paths: **0**; duplicate paths: **0**; SHA mismatches: **0**; line mismatches: **0**; byte mismatches: **0**.
- Read boundary: all ten listed originals were read completely, first through last logical line. No original assigned to another deep chunk was opened. `10-J-SUMMARY_INDEX.discovery.md` was consulted only as an orientation and delta-comparison artifact; no source atom below is derived solely from it.
- Anchor notation refers to the manifest-verified source images: `J-14:Lx-Ly`, `J-29:Lx-Ly`, `J-31:Lx-Ly`, `J-34:Lx-Ly`, `J-38:Lx-Ly`, `J-46:Lx-Ly`, `J-50:Lx-Ly`, `J-60:Lx-Ly`, `J-69:Lx-Ly`, and `J-81:Lx-Ly`.
- Repeated premise, revised-scope, wiring, grounding, composition, and acceptance language is collapsed into one atom with every load-bearing anchor. Interaction, ambiguity/conflict, and summary-delta records reference atoms and are not counted again as features.
- Classification: `E` = explicit source fact/requirement; `I` = bounded inference; `A` = unresolved ambiguity/choice; `C` = internal conflict or summary correction that must not be silently flattened. Domains: `STATUS`, `CAP`, `SURFACE`, `FLOW`, `STATE`, `DATA`, `CFG`, `INT`, `EVENT`, `SEC`, `PRIV`, `COMP`, `FAIL`, `RECOVERY`, `RETRY`, `DET`, `IDEM`, `CONC`, `OBS`, `TEST`, `LIMIT`, `DEP`, `DEFER`, `EVIDENCE`.

### Exact processed manifest and verification

| Manifest class | Lines | Bytes | SHA-256 | Exact absolute path | Verification |
|---|---:|---:|---|---|---|
| numbered/index original | 72 | 9,367 | 5a5296af7a190c51b39d966fe05291768dc8f0d8712d383d76319b2e22b397ae | docs/thefinalglance/J-plan/J-14_repro_pack_rra_verify_upgrade.md | exact |
| numbered/index original | 186 | 12,890 | 169788cb27abb10d3e2b2134eeef0b6f636d23ccff44df6ea27f02f9392a5a19 | docs/thefinalglance/J-plan/J-29_usage_window_realtime_surfaces.md | exact |
| numbered/index original | 120 | 8,890 | 93a0c6a0cc59107cb1787986b4ed0fcb4df3215c2bfb04a665957c9e35a91d24 | docs/thefinalglance/J-plan/J-31_canvas_system_mode.md | exact |
| numbered/index original | 126 | 9,311 | 25b1eacdba09a9d68b388aa5341c402a2e0eab0a26170ad8774ba3f29b5c4685 | docs/thefinalglance/J-plan/J-34_empirical_context_calibration.md | exact |
| numbered/index original | 148 | 23,858 | c1795cb2d38b19db819e66709665eb6ecffd377511a9911c154f8b75cbe479da | docs/thefinalglance/J-plan/J-38_agentic_retrieval_loop.md | exact |
| numbered/index original | 55 | 10,818 | 9f9f34cae23380bb0e13233c69edb766905bfd727072f50e9e02f0272f1fa6c5 | docs/thefinalglance/J-plan/J-46_doctrine_debt_rate_measurement.md | exact |
| numbered/index original | 95 | 11,970 | 503e4ba063566a49689b4bc559af36633900f0efebc895b9cb476eeff615ddc7 | docs/thefinalglance/J-plan/J-50_recovery_trajectory_sft_exhaust.md | exact |
| numbered/index original | 239 | 22,580 | 0cb44a43e6a56a7c7a376e483112613d004ce4af1524f3d44e4ab64bdcb3efa8 | docs/thefinalglance/J-plan/J-60_layered_observability_canvas.md | exact |
| numbered/index original | 68 | 9,166 | 807d89feaea3cfbeab235142159e818cee7986af7788c2b55fe4fd4ac38f5483 | docs/thefinalglance/J-plan/J-69_ssb_log_integrity_hash_chain.md | exact |
| numbered/index original | 111 | 14,430 | d378502224735b714c18b57ee521a90478c8f09080c1b5c1985696b83ab1a5af | docs/thefinalglance/J-plan/J-81_autonomous_supervision_hardening.md | exact |
| **Aggregate** | **1,220** | **133,280** | — | **10 exact paths** | **exact** |

## Per-file atomic ledgers

### J-14 — repro-pack + RRA verify upgrade

Source: `docs/thefinalglance/J-plan/J-14_repro_pack_rra_verify_upgrade.md` (72/72 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| J14-E001 | E/STATUS | J-14 is Track J PROVABLE, STRONG grounding (detllm ingest + live grep), nature EXTENSION of G-2/G-10 plus small BUILD legs, effort M, WAVE 2-3, depends none; J-13 makes Tier-1/2 PASSABLE on local and without it tiers report honestly. | J-14:L1-L3 |
| J14-E002 | E/CAP | `elai replay --verify` gains a three-tier taxonomy: T0 = action-stream replay match (today's G-2 verdict); T1 = token-stream blake3 equality (passable once J-13 ships); T2 = score/logprob equality where the lane exposes scores. | J-14:L8-L8 |
| J14-E003 | E/CFG | Tier evaluation is capability-gated with recorded downgrades `{from, to, reason}`; invariant `tier_effective <= tier_requested` always; strict mode emits `UNSUPPORTED_REQUEST` and aborts, never silently degrades (R1 analog). | J-14:L8-L8, L44-L44 |
| J14-E004 | E/DATA | The RRA bundle carries a `determinism_applied` block stating controls actually active at run time: `ELAI_SEED`, `sampling_seed`, determinism-mode flags, and env controls actually read. | J-14:L9-L9, L31-L31 |
| J14-E005 | E/DET | detllm gotchas are mandatory: the fingerprint hash excludes its own field; artifact serialization is key-sorted for hash stability; score comparison is exact equality, never epsilon. | J-14:L9-L9, L43-L43 |
| J14-E006 | E/OBS | On T1/T2 mismatch, a first-divergence locator records the first divergent position plus both values into the bundle diff section; G-10's canary consumes it as "diverged at token N". | J-14:L10-L10, L37-L37 |
| J14-E007 | E/DATA | File-change snapshots record pre/post content hashes (optional zstd content) for files touched during a run into the bundle; the pre-image is already captured at `file_ops.rs:272/300` then discarded (H-49 finding, PLAN-asserted); full content store remains H-49 CAS scope (GD-1). | J-14:L11-L11, L20-L20, L47-L47 |
| J14-E008 | E/SEC | Auth/secret scrub before archive is unconditional and composes G-9 redaction functions without depending on `ELAI_RETENTION_PURGE_ENABLE`; culpa's verbatim auth-header store is an explicit DO-NOT-COPY exhibit. | J-14:L11-L11, L21-L21, L45-L45 |
| J14-E009 | E/SURFACE | CLI verb `elai rra export-fixture --adapter <name>` extracts the span subset (requests/responses for one adapter) into a small fixture file with a per-fixture blake3 fingerprint. | J-14:L12-L12, L32-L32, L39-L39 |
| J14-E010 | E/DATA | Fixture matching key is `(model + message-content-blake3 + tool_name)`, never positional (culpa/evalcraft brittleness). | J-14:L12-L12, L46-L46 |
| J14-E011 | E/DEP | The export-fixture format is I-49's replay-fixture input format, defined once in J-14. | J-14:L12-L12, L62-L62, L70-L70 |
| J14-E012 | E/CAP | Flow-fingerprint guard on `TemplateCrystallizer` (`compiled_pipeline.rs:293`): at crystallization hash `(tool sequence + arg-shape hashes)`; on each compiled-flow execution re-hash and compare (~40 LOC reimpl, blake3). | J-14:L13-L13, L19-L19, L33-L33, L36-L36 |
| J14-E013 | E/FLOW | Fingerprint mismatch de-crystallizes the template, falls back to the LLM path, and emits the mismatch event. | J-14:L13-L13, L51-L52 |
| J14-E014 | E/TEST | Entropy-scan CI is a deterministic lint over `crates/orchestrator` + `crates/cli` deterministic-path modules flagging unseeded randomness (`thread_rng` without `ELAI_SEED` derivation, uuid4-style); observe-only first (JD-3) with an allowlist for legitimate sites. | J-14:L14-L14, L67-L67 |
| J14-E015 | E/LIMIT | Excluded: detllm as a dependency; sha256 (blake3 everywhere); any LLM-judge equality; re-implementation of H-49 CAS or G-9 redaction internals. Schema is inspiration not dependency (18-star alpha). | J-14:L5-L5, L15-L15, L43-L47 |
| J14-E016 | E/INT | Edit set: `rra.rs` (~250 LOC: tier verdicts, determinism_applied, first-divergence, snapshots, scrub) + CLI export-fixture arm (~80 LOC) + crystallizer guard (~40 LOC) + entropy lint/xtask (~60 LOC). | J-14:L24-L25 |
| J14-E017 | E/EVENT | Adds `ReproPackEvent` SSB (Pattern-2 single-variant; Kind: `TierEvaluated` / `FirstDivergenceLocated` / `FlowFingerprintMismatch` / `FixtureExtracted`); baseline+1 from LIVE 229/540 (`ssb.rs:3066` / `action_type_lock.rs:662`, grep 2026-06-11); re-read at edit (W-9); full 6-part variant discipline. | J-14:L26-L27, L51-L51 |
| J14-E018 | E/SURFACE | Frontend pair: tier verdict + divergence position on the existing RRA/review surface (decide at edit; W-4) or W-6-justified absence. | J-14:L27-L27, L48-L48 |
| J14-E019 | E/INT | Production reachability: `elai replay --verify <bundle>` and `elai run --emit-rra` (tier verdict replaces binary verdict); compiled-flow execution through TemplateCrystallizer materialization; `#[cfg(test)]`/contracts callers never satisfy R7. | J-14:L38-L41 |
| J14-E020 | E/TEST | Trigger: `elai run --emit-rra` on a fixture task then `elai replay --verify`; mutate one recorded response byte and re-verify; also perturb a crystallized template tool sequence. | J-14:L50-L52 |
| J14-E021 | E/FAIL | Negative: bundle missing `determinism_applied` FAILS verify (missing evidence = FAIL); seeded auth-header fixture is provably scrubbed (grep on exported bytes = 0 hits); perturbed crystallized flow runs LLM fallback never the stale template. | J-14:L52-L52 |
| J14-E022 | E/DEP | Composes G-2 RRA extended in place, G-10 drift canary (consumer), J-13 (T1/T2 basis), H-49 (snapshot hashes; CAS stays H-49), I-49 (fixture format), I-45 objective-hash receipts (sibling seal, untouched), G-9 redaction, C-25 crystallizer. | J-14:L37-L37, L61-L62 |
| J14-E023 | E/EVIDENCE | Grounding: `workflow/papers/detllm-source-interface.md`; intentusnet fingerprint + entropy-scan via agent det-repos-S2; culpa/evalcraft via det-repos-S3; steal ledger `workflow/reasoning/steal-determinism-repos-2026-06-11.md`; live anchors `rra.rs:474/481`, `compiled_pipeline.rs:293/54`. | J-14:L17-L22, L54-L59 |
| J14-E024 | E/CFG | Bundle size policy: hash-pair-only default for snapshots; content opt-in; measured before any default flip (GD-4). | J-14:L66-L66 |
| J14-A001 | A/CFG | Tier names keep T0/T1/T2 semantics but ELAI vocabulary may be `ActionFaithful`/`TokenFaithful`/`ScoreFaithful`; mapping is decide-at-edit. | J-14:L65-L65 |
| J14-A002 | A/SURFACE | Exact RRA/review surface placement for tier + divergence UI is decide-at-edit under W-4. | J-14:L27-L27, L48-L48 |
| J14-A003 | A/TEST | Entropy-scan implementation vehicle (`.claude-side` lint OR clippy-style xtask) is decide-at-edit. | J-14:L25-L25 |
| J14-I001 | I/RECOVERY | Honest tier reporting without J-13 implies T1/T2 surface as unsupported/downgraded rather than false PASS when local token equality is not yet available. | J-14:L3-L3, L8-L8 |

Category coverage: capabilities/actions/surfaces ✓ (CLI verify/export-fixture, RRA review UI); workflows/states ✓; schemas/data/lifecycle ✓; defaults/config ✓; integrations/events ✓; security/privacy scrub ✓; failures/recovery ✓; determinism ✓; observability/evidence ✓; tests/done gates ✓; constraints/non-goals ✓; dependencies/status/deferred ✓. Formal retry/idempotency key APIs are not specified beyond fixture content keys.

### J-29 — usage window realtime surfaces

Source: `docs/thefinalglance/J-plan/J-29_usage_window_realtime_surfaces.md` (186/186 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| J29-E001 | E/STATUS | J-29 is PLANNED WAVE GSW-2, BUILD read-side, zero new SSB action types, effort M-L; depends J-28 for full `(model,task,session)` keying with a degraded mode over existing rows; gauge + existing-row tabs are independent of full keying. | J-29:L3-L8 |
| J29-E002 | E/SURFACE | New dashboard-v2 window `UsageWindow.svelte`: per-model / per-task / per-session / per-lane token + cost breakdown (J-28 rollup), cache effectiveness (read vs creation, CloudPanel hit-rate formula reused), tool-call + file-op counts (read-side over existing SSB action types), and the per-turn record view. | J-29:L20-L23, L84-L85, L99-L100 |
| J29-E003 | E/SURFACE | CloudPanel (5s poll `get_cloud_spend`), SpendDrawer (10s), MonitorWindow gauge row, and CostPanel are KEPT; UsageWindow is the deep view, not a replacement (no-removal rule). CostPanel is DARK behind `ELAI_COST_TOKEN_ECONOMICS_ENABLE` and is NOT this unit's surface. | J-29:L59-L63, L123-L123 |
| J29-E004 | E/CAP | Tauri read commands: `get_usage_rollup` (J-28 ledger), `get_backend_cost_rollup` (BackendCostAccrued rows joined hash→name by iterating `RuntimeBackend::all()`), pricing_rollup read (per-phase/agent/tool dollar breakdown — first consumer), and `get_live_usage`. | J-29:L26-L32, L86-L87, L101-L102 |
| J29-E005 | E/CFG | The pricing_rollup module's existing `ELAI_AGENTIC_OBSERVABILITY_ENABLE` gate is respected: gate off => tab renders explicit "observability gate off" state, never silent empty; no new gate is added. | J-29:L29-L32, L66-L67, L137-L138 |
| J29-E006 | E/CAP | LiveUsageGauge: in-process atomic counters updated by stream consumers (cloud SSE + local streaming path) holding running output-token count + turn input estimate; mid-stream values NEVER touch SSB (journal bloat ban / J-28 stub_ban). | J-29:L33-L38, L90-L91, L103-L103, L119-L119 |
| J29-E007 | E/SURFACE | Chat status line (CommandWindow) polls `get_live_usage` at 1s; CLI tick on the live streaming path rewrites a single status line with no scroll spam. | J-29:L35-L36, L91-L91, L114-L116 |
| J29-E008 | E/STATE | The per-turn `UsageTurnEvent` row remains the durable record; the gauge resets on turn end. | J-29:L37-L38 |
| J29-E009 | E/SURFACE | Local display rule: `lane != cloud` renders token counts ("in N / out M"), never a fake `$0.00` (matches `cost_model_for(Sidecar) = None`). Unknown-rate cloud models render `"no rate for <model>"` (J-28 T4 explicit None), never silent sonnet defaults. | J-29:L39-L41, L134-L135 |
| J29-E010 | E/OBS | Autonomy ratio: one derived read-side stat `auto-executed steps / total steps` from existing gate/approval/intervention action types; labeled DERIVED with formula in tooltip; LLM never computes it; pure journal arithmetic (GD-2). Exact source action types enumerated at edit from live `ssb.rs`. | J-29:L42-L45, L174-L176 |
| J29-E011 | E/SURFACE | Canvas usage badge op via existing E-40 3-arm `canvas_*` dispatcher pattern (CanvasDispatcher): session token total + live gauge value when streaming. Product canvas (CANVAS WORKBENCH Track 2) is in scope; harness canvas (`.claude/mcp/canvas`) is OUT of scope. | J-29:L46-L49, L92-L92, L104-L104, L112-L112 |
| J29-E012 | E/SURFACE | ContextStatePanel additionally renders absolute token count + limit beside the existing `token_budget_pct` badge; fields already in `context_budget_computed` payload read path (`commands.rs:8840-8845`); verify at edit or extend the read not the producer. Thresholds 70/85 from `ssb.rs` constants (`commands.rs:8799-8803`). | J-29:L50-L53, L68-L69, L93-L93, L105-L105 |
| J29-E013 | E/DEFER | Tier-1 ships T1/T2/T4/T5/T7 pure read-side + T3 gauge on cloud path + CLI tick. Tier-2 (remote-sidecar-gated): local-lane LIVE gauge confirmation with same boot discipline as J-28 Tier-2. | J-29:L76-L81 |
| J29-E014 | E/INT | Integration: UsageWindow + window host registration; Tauri commands (~200 LOC); `stub.ts` + `types.ts` lockstep; LiveUsageGauge (~80 LOC) at two stream-consumer seams; CLI tick (~30 LOC); CanvasDispatcher op + badge (~60 LOC); ContextStatePanel (~15 LOC). frontend_pair IS this unit (J-28's dashboard half). | J-29:L83-L95, L126-L128 |
| J29-E015 | E/EVENT | Net zero new action types; LIVE baseline SCHEMA_VERSION 234 / TOTAL_VARIANTS 543 (`ssb.rs:3096` / `action_type_lock.rs:676`); re-grep at edit (J-28 lands +1 first in GSW sequencing). | J-29:L70-L71, L132-L133, L156-L157 |
| J29-E016 | E/INT | Production reachability: dashboard window host (UsageWindow registered + openable); live chat status line; live CLI streaming tick; contracts-only never R7. | J-29:L113-L117 |
| J29-E017 | E/LIMIT | Stub bans: no SSB writes from gauge/mid-stream; no TS-side re-aggregation (Rust rollup is sole source, GD-2); empty journal => explicit empty state; unresolvable backend hash => `"unknown backend (<hash>)"` row never crash/blank; no deletion of CloudPanel/SpendDrawer/CostPanel; no new env gate; no LLM in displayed-number path; no `.unwrap()`/`.expect()` on prod paths. | J-29:L118-L125 |
| J29-E018 | E/TEST | Acceptance: dashboard-smoker drives one cloud fixture turn + one local fixture turn (J-28 triggers), opens UsageWindow; streaming fixture with chat status line visible; status line increments BEFORE turn end (C-5 closed). NEGATIVE: zero-row journal empty state; seeded unknown backend hash row; observability gate-off notice; svelte-check + cross_lang_diff clean. | J-29:L129-L138 |
| J29-E019 | E/TEST | DETERMINISTIC_CHECK: wiring_contract_check, rg UsageWindow, rg get_live_usage/get_usage_rollup/get_backend_cost_rollup, cross_lang_diff, rg write_payload count MUST be 0 on LiveUsageGauge file, `workflow/reasoning/J-29-decisions.md` exists. | J-29:L140-L148 |
| J29-E020 | E/DEP | Composes J-28 (feed), H-9 CloudPanel (kept shallow view), C-44 (BackendCostPanel absence closed), E-39 CostPanel (untouched), E-40 canvas dispatcher, phase-3E ContextStatePanel, H-13/Track-2 canvas host, dashboard-handoff skill. | J-29:L164-L168 |
| J29-E021 | E/LIMIT | No new dependency; charts reuse whatever existing panels use; a chart dep not already in dashboard-v2 package.json requires user confirmation; default = hand-rolled bars. | J-29:L158-L160, L173-L173 |
| J29-E022 | E/CFG | Poll (1s) vs Tauri event push for the gauge: start with poll (matches existing 5s panel idiom); upgrade on observed lag only (GD-4). Push-event upgrade is a later option if 1s poll proves laggy. | J-29:L54-L56, L171-L172 |
| J29-E023 | E/STATUS | Premise gaps closed: BackendCostPanel ABSENT; pricing_rollup zero consumers; ContextStatePanel pct-only; CloudPanel/SpendDrawer lack per-model/session/real-time. | J-29:L10-L17, L64-L67 |
| J29-A001 | A/OBS | If approval-shaped rows prove too sparse for autonomy-ratio meaning, ship behind "experimental" label or drop (honest residue, GD-5). | J-29:L174-L176 |
| J29-A002 | A/DATA | Per-task view depends on `task_id` propagation through J-28 rows on BOTH lanes; verify local seam carries task id at edit. | J-29:L177-L178 |
| J29-I001 | I/SURFACE | Because UsageWindow is declared the home surface for later H-9 remaining tier and E-39 economics panel, those future panels are expected to land into/beside this window rather than invent a second deep-usage home. | J-29:L180-L184 |

Category coverage: capabilities/actions/surfaces ✓ (UsageWindow, CommandWindow status line, CLI tick, canvas badge, ContextStatePanel, Tauri commands); workflows/states ✓; schemas/data ✓; flags/defaults ✓; integrations/events ✓ (net-zero SSB); security integrity of display rules ✓ (no privacy policy); failures/empty states ✓; determinism/recompute ✓; observability ✓; tests ✓; constraints ✓; deps/deferred ✓. Retry/idempotency not specified.

### J-31 — canvas system-composition mode

Source: `docs/thefinalglance/J-plan/J-31_canvas_system_mode.md` (120/120 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| J31-E001 | E/STATUS | J-31 is PLANNED WAVE CFW-2, BUILD-on-shipped, effort L, STRONG grounding (canvas substrate SHIPPED 2026-06-12; novelty unplanned); depends shipped H-13/H-15 canvas (live), J-16 (DRIFTED leg, Tier-2), J-32 (dictate-on-node soft — chip fallback exists). | J-31:L3-L5 |
| J31-E002 | E/CAP | Category distinction: n8n/Flowise/Langflow/Dify are runtime flow builders; J-31 is design-time system-composition for the user's codebase where the runtime is the generated system and edges are wiring claims (web-verified novelty 2026-06-12, GD-1 clear). | J-31:L9-L19 |
| J31-E003 | E/SURFACE | Add `'system'` to CanvasMode union (`'code'|'plan'|'research'|'arch'|'design'` at `canvas.svelte.ts:39`) + CanvasModeBar entry + a `'system'` op-namespace on shipped InProcessCanvasDispatcher (`canvas_dispatcher.rs:101`). | J-31:L23-L26, L59-L60, L87-L89 |
| J31-E004 | E/DATA | SystemNode `{ node_id, binding, display }` where binding = `PlanUnit(concept/phase/step id) | Contract(contract id) | McpService(server id)`. A node past DRAFT MUST bind; unbound renders DRAFT grey. | J-31:L27-L29 |
| J31-E005 | E/STATE | Bindings and verdicts persist ONLY as SSB rows; the journal is the store; the canvas re-derives state byte-identically (GD-2); no parallel state file. | J-31:L29-L31, L106-L107 |
| J31-E006 | E/DATA | SystemEdge `{ producer_node, consumer_node, contract }` carries the exact field set `wiring_contract_check.py` parses: produces / consumes / reachable_from / stub_ban / observable_effect (checker lines ~672-688; `--run-trigger` is a deprecated accepted no-op). | J-31:L32-L35 |
| J31-E007 | E/STATE | Edge verdict ladder, all DERIVED: declared (authored) → wired (static leg: producer symbol exists + consumer references it, DKG/elai.map reachability) → verified (contract's observable_effect SSB row exists; row id renders on the edge) → broken (previously-passing leg fails). LLM NEVER sets a verdict (TD-3). | J-31:L35-L39, L103-L106 |
| J31-E008 | E/STATE | Node lifecycle: DRAFT → FRAMED (bound; dictated intent passes shipped B-40 IdeaFramer + AmbiguityScorer; vague intent gets clarify loop) → BUILDING (FSM active; live halo via SSB canvas overlay) → WIRED → VERIFIED → DRIFTED. | J-31:L40-L44 |
| J31-E009 | E/DEFER | DRIFTED (Tier-2): a J-16 blast-radius cone touching the node's bound symbols flips status until re-audit; fallback until J-16 ships = DKG file-hash staleness. | J-31:L43-L44, L82-L83 |
| J31-E010 | E/FLOW | Dictate-on-node: select node + speak/type routes through J-32 front door with node's binding as scope (shipped CanvasUiAction::Select, `canvas_ui_action.rs:61`, G-1 taint-gated read-back). Until J-32: existing chip flow with node id prefilled. | J-31:L45-L48, L83-L83 |
| J31-E011 | E/SURFACE | External service node = MCP connector only: server id + blake3 schema pin (E-24) with G-4 rug-pull re-pin verdict rendered ON the node; ingress carries G-1 Provenance taint; edges from external node render taint badge. n8n's 400-integration treadmill is DECLINED. | J-31:L49-L52, L109-L110 |
| J31-E012 | E/FLOW | Build action dispatches bound plan unit through the NORMAL planning-workflow FSM (no parallel executor, no canvas-side runtime); progress renders from rows the run already emits. | J-31:L53-L55, L103-L103 |
| J31-E013 | E/CFG | Canvas is NOT the default window (`workspace.svelte.ts:99-100` primary = `'activity'`; `canvasAsDefault` false pending H-16); J-31 does not change that. | J-31:L60-L61, L118-L118 |
| J31-E014 | E/SURFACE | GraphWindow + `get_graph_view_data_impl` stay CODE mode (what exists); SYSTEM mode renders intent-side components. Same window, two directions of truth. | J-31:L68-L70 |
| J31-E015 | E/STATUS | Phase 3G PhaseEntry/ContractRef/extract_contracts are SPEC-level types, NOT confirmed runtime Rust types — re-grep at edit; if absent, J-31 ships own SystemBinding/EdgeContract types shaped on 3G spec. | J-31:L71-L73 |
| J31-E016 | E/CFG | Tier-1: mode + nodes + mandatory bindings + edges + declared/wired verdicts + MCP node pin-status + build action. Verdict derivation runs at audit/idle tick, auditor-owned, NEVER on render hot path. Tier-2: verified-via-observable_effect automation, DRIFTED via J-16, dictate-on-node via J-32, edge-level taint badges beyond node badge. | J-31:L77-L83 |
| J31-E017 | E/INT | Dispatcher `'system'` namespace ops: `create_system_node` / `bind_node` / `declare_edge` / `set_display` via existing 3-arm tool pattern (catalog + convert + dispatch, `production_agent_rpc.rs`). NEW `crates/orchestrator/src/system_canvas.rs` (~250 LOC): binding registry over SSB + verdict derivation. | J-31:L90-L94 |
| J31-E018 | E/EVENT | +SystemContractEvent (Pattern-2; Kind: `NodeBound` / `EdgeDeclared` / `EdgeVerdict` / `NodeDrifted`); baseline+1 from LIVE 241/550 (`ssb.rs:3152` / `action_type_lock.rs:705`, 2026-06-12); pure canvas ops PREFER-REUSE CanvasOpApplied (no second op-journal); full 6-part + golden-lock 9-tuple. | J-31:L95-L99 |
| J31-E019 | E/TEST | Done-gate: bind fixture node to real plan unit with real contract; run build; edge flips declared→wired only after static leg; →verified ONLY after observable_effect row lands (row id on edge); zero-evidence node renders wire-dark grey never green (R1); full state re-derives under ELAI_SEED. | J-31:L101-L107 |
| J31-E020 | E/FAIL | NEGATIVE: deriving verdicts from executor self-report FAILS; deleting bound consumer symbol flips edge to broken on next derivation tick; MCP schema hash change renders G-4 violation and edges drop to untrusted; node bound to nonexistent plan-unit id fails LOUD at bind time, never renders. | J-31:L107-L111 |
| J31-E021 | E/LIMIT | Runtime data-flow execution inside the canvas is DECLINED (runtime is user's codebase; execution is FSM's job). | J-31:L115-L116 |
| J31-E022 | E/DEFER | Per-node cost/usage badge rides J-29's canvas badge when it ships. Canvas-as-default stays H-12/H-16. System-node template library (trading/SaaS/ETL) is a later catalog unit like J-9. | J-31:L117-L120 |
| J31-E023 | E/DEP | Existing substrate: CanvasWindow.svelte production; CanvasModeBar.svelte:33; InProcessCanvasDispatcher; production_agent_rpc dispatch 3040-3062; CanvasOpApplied; spawn_view 1299-1332; CanvasViewEvent 3099; G-1 taint; G-2 RRA verify_rra:476. | J-31:L57-L67 |
| J31-A001 | A/DATA | Whether SystemBinding/EdgeContract reuse 3G runtime types or ship local shapes depends on edit-time re-grep of phase-3G types. | J-31:L71-L73 |
| J31-I001 | I/SURFACE | frontend_pair IS the SYSTEM mode surface itself under `src/lib/canvas/` with cross_lang_diff + svelte-check in-unit. | J-31:L87-L89 |

Category coverage: capabilities/surfaces ✓ (SYSTEM mode, ModeBar, MCP node badges, edge row-id display); workflows/states ✓; schemas ✓; config tiers ✓; events ✓; security/taint/MCP pin ✓; failures ✓; determinism GD-2 ✓; tests ✓; constraints/non-goals ✓; deps/deferred ✓. No formal privacy/retry APIs.

### J-34 — empirical context calibration

Source: `docs/thefinalglance/J-plan/J-34_empirical_context_calibration.md` (126/126 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| J34-E001 | E/STATUS | J-34 is PLANNED WAVE CC-1, BUILD-on-shipped calibration layer over H-50/3E/E-49, effort M-L, STRONG code-verified 2026-06-13; Tier-2 dep J-28 (journal-regression); invalidation composes G-10. | J-34:L3-L5 |
| J34-E002 | E/CAP | CLI `elai bench context-fill`: fixed fixture task set at stepped context-fill levels (10%..95%), padding with DETERMINISTIC real-repo content under ELAI_SEED (never lorem), scoring verified-OUTCOME rate (compile/test/gate verdicts, never LLM-judged, TD-3). | J-34:L28-L34, L114-L115 |
| J34-E003 | E/DATA | Output per (model, lane): fill→outcome curve + knee, written to calibration artifact + metrics TOML, byte-recomputable from the journal (GD-2). | J-34:L31-L34, L105-L106 |
| J34-E004 | E/CFG | `ModelCapabilityProfile` (`context_budget.rs:292`) gains `fill_degradation_curve` + `calibrated_trim_fraction` (Option fields; absent = today's behavior). | J-34:L35-L36, L66-L67 |
| J34-E005 | E/FLOW | `context_policy.rs::trim_threshold` gains per-model lookup that PREFERS measured value over `CLOUD_TRIM_FRACTION=0.85` / env constant when present; `recalibrate_budget` at `context_budget.rs:849` is the existing hook. H-50 regime split unchanged. | J-34:L36-L40, L64-L65 |
| J34-E006 | E/CAP | Positional placement: extend E-49 context_rot_probe (fixed 12K synthetic, 5 depths, pass/fail) into positional-value PROBE: constraint-recall fixtures at stepped depths d, deterministic honored/violated scoring → `ModelCapabilityProfile.positional_value_curve`. | J-34:L41-L44, L68-L69 |
| J34-E007 | E/CFG | 3E slot allocator gains PLACEMENT policy: slot ORDER by (slot recall-sensitivity × measured positional value) — high-sensitivity slots (active constraints, task spec, rules deltas) at high-recall edges; bulk recoverable content (CodeContext, SsbEntries) in measured trough. | J-34:L44-L47 |
| J34-E008 | E/DET | CACHE-SAFE BY CONSTRUCTION (HD-2): layout is fixed PER MODEL at calibration time, never per-call; byte-stable prefix ordering only changes when calibration artifact changes (logged/journaled event); `bp.valid` assert still guards. | J-34:L48-L50, L108-L113 |
| J34-E009 | E/DEP | I-6 AnchorPane stays last-position special case this generalizes; E-32 FITM remains orthogonal logit-level mitigation on local lane; placement works on BOTH lanes including cloud. | J-34:L51-L53, L70-L72 |
| J34-E010 | E/STATE | Calibration artifacts keyed by model identity (`ModelIdentity::identity_key`); G-10 model-drift/identity change marks them STALE (recalibrate, never silently reuse). | J-34:L54-L56, L74-L75 |
| J34-E011 | E/DEFER | Tier-2 (gated on J-28 UsageTurnEvent): journal-regression of verified-outcome vs fill% from REAL production turns per (model, task-class); until J-28 keys exist the padded-fixture bench stands alone. Also: per-task-class curves; G-10-triggered auto-recalibration (until G-10: manual `elai bench context-fill --recalibrate`). | J-34:L56-L60, L84-L86 |
| J34-E012 | E/CFG | Tier-1: legs 1+2 for LOCAL lane + cloud-fixture leg under small fixed budget on cheap workhorse under explicit E-39 cap (never opus-class); cost recorded in TOML. Leg 3 ships behind same artifact with allocator consuming only when present. | J-34:L80-L83 |
| J34-E013 | E/INT | NEW `context_calibration.rs` (~250 LOC) + CLI bench arm `context-fill` (mirrors seccodeplt/qdelta, `args_bench.rs`); edits to context_policy.rs, context_budget.rs, E-49 probe. | J-34:L90-L94 |
| J34-E014 | E/EVENT | +ContextCalibrationEvent (Pattern-2; Kind: `CurveMeasured` / `ThresholdApplied` / `PlacementApplied` / `CalibrationInvalidated`); baseline+1 from LIVE 241/550; ThresholdApplied/PlacementApplied make every consumption journal-visible (threshold used but never logged = R1 FAIL). | J-34:L95-L98 |
| J34-E015 | E/SURFACE | Frontend pair: measured curve + knee + active thresholds on ContextStatePanel/cost-cockpit family ("your model degrades HERE" tile). | J-34:L99-L101 |
| J34-E016 | E/TEST | Done-gate: local-lane CurveMeasured rows recompute under ELAI_SEED; subsequent session shows ThresholdApplied with MEASURED trim fraction (not 0.85); panel renders curve + knee; placement order follows policy AND HD-2 byte-stability passes. | J-34:L105-L110 |
| J34-E017 | E/FAIL | NEGATIVE: no calibration artifact uses today's constants AND logs that fallback; deleting artifact mid-session does not change layout until logged recalibration boundary; LLM-judge scoring FAILS contract; lorem/random padding FAILS fixture audit. | J-34:L110-L115 |
| J34-E018 | E/LIMIT | Needle-in-haystack retrieval probes are NOT the metric (retrieval ≠ task outcome). Live in-session rot meter (mid-session probe calls) is DECLINED for cost; ledgered fill% vs calibrated knee on the panel is the live instrument. | J-34:L121-L124 |
| J34-E019 | E/DEFER | Publishing measured curves as docs/strategy artifact = K-1 claims-policy material once numbers exist. Cross-hardware variance on local curves recorded honestly (same-box same-quant validity, GD-5). | J-34:L119-L126 |
| J34-E020 | E/STATUS | Folklore baselines named: `CLOUD_TRIM_FRACTION = 0.85` (`context_policy.rs:70`); local window from env; slot ORDER fixed by convention; Reddit 42%/67% is the field state of the art. GD-1: no unit measures per-model fill thresholds today. | J-34:L9-L24, L64-L65 |
| J34-A001 | A/CFG | Exact knee-detection algorithm and stepped fill grid internals beyond 10%..95% are not further specified. | J-34:L28-L34, L90-L91 |
| J34-A002 | A/TEST | Cloud fixture budget amount and workhorse model identity are constrained (E-39 cap, never opus-class) but not named as concrete constants. | J-34:L80-L82 |
| J34-I001 | I/OBS | The panel "fill% vs calibrated knee" instrument is the intended mid-session substitute for the declined live rot probe. | J-34:L123-L124 |

Category coverage: capabilities/CLI/surfaces ✓; workflows ✓; schemas/profile fields ✓; config/defaults ✓; events ✓; failures ✓; determinism/cache-safety ✓; observability ✓; tests ✓; constraints ✓; deps/deferred ✓. No privacy/concurrency APIs beyond journal visibility.

### J-38 — agentic retrieval loop

Source: `docs/thefinalglance/J-plan/J-38_agentic_retrieval_loop.md` (148/148 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| J38-E001 | E/STATUS | J-38 is Track J CANT-MISREAD, STRONG paper grounding (SWE-Explore/SpIDER/SweRank/LocAgent/GrepRAG abstracts VERIFIED 2026-06-16), nature BUILD, effort M, WAVE AR-1; soft-depends J-18 (hardened HYBRID is preferred; works on current cascade at GraphCodeBERT quality). | J-38:L3-L3, L17-L17 |
| J38-E002 | E/CAP | Mid-task tool `hybrid_search(query, top_k, mode)` registered as executor-callable alongside B-29 traversal tools; routes to FAST-tier HYBRID cascade; cross-encoder reranker OPTIONAL/off on mid-task path by default (~190ms/call compounds). | J-38:L8-L8, L14-L14, L22-L22, L30-L30 |
| J38-E003 | E/LIMIT | Loop is ADDITIVE to pre-task pipeline-fill; never replaces amortized prefill (SweRank: non-agentic retrieve+rerank beats pure agentic grep on localization and cost 44x). Removing prefill requires user confirmation. | J-38:L5-L5, L8-L8, L11-L11, L43-L43 |
| J38-E004 | E/CAP | SpIDER graph-neighbor fusion: after hybrid top-K, expand each candidate to call/import/type-resolution neighbors (depth <= 2) via SCIP/DKG (B-29 + J-18 LSP when present), then rerank expanded set. Applies to BOTH pre-task fill and mid-task tool. | J-38:L9-L9, L23-L23, L31-L31, L117-L123 |
| J38-E005 | E/CAP | Deterministic context-sufficiency gate: symbol-coverage over DKG-resolved IntentPair (C-29) + symbols surfaced during reasoning; below threshold signals executor to fire `hybrid_search`; above proceeds. LLM NEVER adjudicates sufficiency (TD-3). | J-38:L10-L10, L24-L24, L32-L32, L44-L44, L82-L83 |
| J38-E006 | E/CFG | Max-rounds cap bounds re-query (anti-spiral analog); unbounded mid-task re-query is structurally forbidden (stub_ban). | J-38:L10-L10, L45-L45, L109-L113 |
| J38-E007 | E/LIMIT | Excluded: LLM-judged sufficiency; MCTS/tree-search over retrieval (SWE-Search deferred); mandatory reranker on loop; second embedding model or index geometry; new default-OFF `ELAI_*_ENABLE` without wire-dark-allowlist + promotion plan. | J-38:L11-L11, L46-L48 |
| J38-E008 | E/LIMIT | Empty neighbor set contributes nothing / ZERO rank-filler to RRF (B6 RRF-noise critique), never top-k noise filler. | J-38:L49-L49, L54-L54 |
| J38-E009 | E/INT | Tool seam is the SAME IpcToolDispatcher B-29 tools register through (C-13); reuse, do not author new dispatch path. | J-38:L22-L22, L35-L35, L99-L101 |
| J38-E010 | E/EVENT | +RetrievalLoopEvent (Pattern-2; Kind: `MidTaskSearchInvoked` / `GraphNeighborExpanded` / `ContextSufficiencyChecked` / `RetrievalRoundCapped`); baseline+1 from LIVE ~268/577; PREFER-REUSE: if QueryTrace carries channel/round OR J-18 RetrievalAugmentEvent ships first, ADD kinds there (net 0); decide at edit. | J-38:L25-L25, L72-L72, L132-L134 |
| J38-E011 | E/SURFACE | Frontend pair: retrieval-round / sufficiency breakdown on existing KnowledgeDrawer (B-29) or W-6-justified absence. | J-38:L26-L26, L50-L50 |
| J38-E012 | E/INT | Production reachability: real `elai` EXECUTING task agent-calling hybrid_search mid-reasoning; pre-task fill path for leg-2 expansion; test/contracts never R7. | J-38:L38-L41 |
| J38-E013 | E/TEST | Trigger: pre-task fill MISSES right span; expect ContextSufficiencyChecked below-threshold → MidTaskSearchInvoked → GraphNeighborExpanded → span lands; RetrievalRoundCapped proves bound. A/B neighbor-of-top-K fixture shows fusion delta (ELAI-MEASURED, never literature +13%). | J-38:L51-L54, L136-L138 |
| J38-E014 | E/FAIL | NEGATIVE: sufficiency ABOVE threshold → NO mid-task search; max-rounds HIT on no-progress fixture stops with RetrievalRoundCapped; leg 1 disabled → pre-task-only MISSES span; empty neighbor set contributes zero filler. | J-38:L54-L54 |
| J38-E015 | E/CFG | First checkbox: gate threshold + max-rounds cap calibrated on >=3 real runs (GD-4, observe-only first) before any default-ON. | J-38:L69-L69 |
| J38-E016 | E/FLOW | Identifier vs intent routing (GrepRAG): identifier-shaped queries should route to BM25+symbol-graph (FAST, no embedding); embedding leg for intent/conceptual queries via deterministic query-shape classifier. | J-38:L70-L70, L104-L104 |
| J38-E017 | E/CFG | Reranker on loop OFF by default; low-K rerank on FINAL round before commit may be worth it — measure, do not assume (B7). | J-38:L71-L71 |
| J38-E018 | E/FLOW | Wargame UNIT 1: `compute_context_sufficiency(intent_symbols, context_symbols)` returns SufficiencyReport `{coverage_ratio, threshold, is_sufficient, missing symbols}`; gate called from `task_runner.rs` before executor starts. | J-38:L82-L88 |
| J38-E019 | E/FLOW | Wargame UNIT 2: hybrid_search registration via IpcToolDispatcher; max_rounds structural enforcement with RetrievalRoundCapped. | J-38:L97-L113 |
| J38-E020 | E/FLOW | Wargame UNIT 3: `expand_with_graph_neighbors(results, depth)` depth<=2; expanded set reranked by existing RRF; A/B neighbor fixture. | J-38:L115-L130 |
| J38-E021 | E/TEST | INTEGRATION_SMOKE after units 1+2 and full loop after unit 3; REDTEAM asserts over-retrieve/infinite-loop defenses and dual-path positive/negative. | J-38:L136-L145 |
| J38-E022 | E/DEP | Consumes HYBRID/RRF + FAST tier unchanged; B-29 DKG edges + IpcToolDispatcher; C-29 IntentPair; J-18 hardened cascade when shipped; QueryTrace prefer-reuse; composes J-16 as sibling DKG consumer. | J-38:L13-L19, L33-L37, L65-L66 |
| J38-E023 | E/EVIDENCE | Papers verified via arxiv WebFetch 2026-06-16; design-rationale numbers (15-19% line recall, r~0.92, +13%, 92.7%, 44x) never become targets (HD-1/ID-5). Synthesis: `workflow/reasoning/retrieval-agentic-loop-grounding-2026-06-16.md`. | J-38:L56-L63 |
| J38-A001 | A/EVENT | Whether to add RetrievalLoopEvent or extend QueryTrace/RetrievalAugmentEvent is decide-at-edit. | J-38:L25-L25, L72-L72, L132-L134 |
| J38-A002 | A/CFG | Identifier-query classifier is optional FORK; may defer and route all through HYBRID. | J-38:L104-L104, L145-L145 |
| J38-A003 | A/CFG | Concrete threshold and max_rounds values are unfixed pending >=3-run calibration. | J-38:L69-L69 |
| J38-I001 | I/FLOW | Sequence after J-18 is preferred so the loop exposes the hardened cascade; sufficiency gate is cheapest first checkbox. | J-38:L74-L75 |

Category coverage: capabilities/tools/surfaces ✓ (hybrid_search tool, KnowledgeDrawer); workflows/states ✓; schemas ✓; config/tiers ✓; events ✓; failures/caps ✓; determinism ✓; observability ✓; tests/wargame ✓; constraints ✓; deps/deferred ✓. No formal privacy policy.

### J-46 — doctrine-debt-rate measurement

Source: `docs/thefinalglance/J-plan/J-46_doctrine_debt_rate_measurement.md` (55/55 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| J46-E001 | E/STATUS | J-46 is Track J DOCTRINE-PROPAGATION (D4 on the propagation itself), BUILD derived metric + baseline sweep + feature-map stamp, effort M-L, WAVE DPW-4; depends J-43, J-44, J-45, ELAI-FEATURE-MAP.md. | J-46:L3-L3 |
| J46-E002 | E/CAP | Doctrine-debt rate = open doctrine-debt-ledger.toml rows / total reviewed decisions, per subsystem over time; computed by `.claude/tools/doctrine_debt_rate.py` from ledger (J-43) + reviewed-decision counts + DP-2 remediation-velocity (rows closed per window). | J-46:L4-L4, L8-L8 |
| J46-E003 | E/OBS | Success is the DELTA in this rate over time, not checker existence (D4 on itself). Every number recomputable from source files (GD-2 / G-5 DSR). | J-46:L4-L4, L8-L8, L29-L29 |
| J46-E004 | E/FLOW | EXP-DP4 OBSERVE-ONLY step 1 baseline: run J-43 floor + J-44 shape-check + random-sample substance audit across WIRED features first; establish baseline rate BEFORE any DD-* class gates. Full B..K sweep is background debt-burn-down, not blocking. | J-46:L7-L7, L9-L9 |
| J46-E005 | E/LIMIT | Nothing default-ON-blocking before this baseline + per-class FP rates (J-43 EXP-DP1) are in. Retroactive blocking sweep that stalls all work is excluded. | J-46:L7-L7, L12-L12 |
| J46-E006 | E/DATA | ELAI-FEATURE-MAP conformance stamp per feature: `clean | debt(row-id) | unreviewed`, upserted in-place on the SAME FEATURE-MAP UPSERT seam planning-workflow/orchestrate already run (one flag flip, never a narrative). `unreviewed` is treated as DEBT (D3). | J-46:L10-L10, L29-L30 |
| J46-E007 | E/FAIL | Implausible-zero self-test (D3): flags any subsystem reporting zero debt rate with non-trivial reviewed-decision count as SUSPECT; routes to DP-2 substance-audit panel. Clean rate must be earned by audited sample, not asserted. | J-46:L11-L11, L30-L30 |
| J46-E008 | E/LIMIT | Excluded: leaderboard-style "doctrine score" headline; counting exceptions without remediation velocity; trusting unaudited "all clean"; LLM computing the rate. | J-46:L12-L12, L29-L29 |
| J46-E009 | E/INT | Produces: doctrine_debt_rate.py, per-subsystem baseline-rate artifact, FEATURE-MAP stamp, implausible-zero flag. Consumes: doctrine-debt-ledger.toml, DP-2 remediation-velocity, FEATURE-MAP WIRED flags + UPSERT seam, J-45 calibration corpus (sink). | J-46:L26-L27 |
| J46-E010 | E/INT | Reachable from only-WIRED-first baseline sweep; doctrine_debt_rate.py over ledger; FEATURE-MAP UPSERT in MEMORIZING/orchestrate. | J-46:L28-L28 |
| J46-E011 | E/EVENT | Net 0 new action_type (harness/doc leg; rate file, ledger, FEATURE-MAP stamp are off-agent JD-1 artifacts). If J-43 ships OPTIONAL DoctrineDebtEvent, rate may derive from those rows; default net 0. | J-46:L31-L31 |
| J46-E012 | E/TEST | JD-1 observable: baseline sweep produces recomputable per-subsystem rate file; real feature flips stamp to debt(row-id) (grep-provable); seeded gamed ledger (zero rows, high reviewed count) trips implausible-zero; unreviewed feature reads as DEBT never clean. | J-46:L23-L23, L30-L30 |
| J46-E013 | E/DEP | Composes J-43 (ledger numerator), J-44 (remediation-velocity + panel for flags), J-45 (baseline findings seed calibration corpus), FEATURE-MAP UPSERT, SAFEDELTA G-6 (delta-not-rank), DP-5/J-47 (consumes rate as leak/quality measure), ELAI-DETERMINISM-DOCTRINE D4/D3. | J-46:L39-L46 |
| J46-E014 | E/LIMIT | Honest residue: pre-doctrine backlog will be large; rate honesty depends on substance-audit sampling; thin sampling is the surrogation trap. | J-46:L23-L23, L50-L50 |
| J46-A001 | A/DATA | "Total reviewed decisions" denominator definition must be pinned in the metric tool for cross-subsystem/time comparability. | J-46:L49-L49 |
| J46-A002 | A/CFG | Implausible-zero threshold (what reviewed-count makes zero rate suspect) is a named constant observed before trusting. | J-46:L52-L52 |
| J46-E015 | E/STATE | Stamp staleness: re-stamp tied to FEATURE-MAP UPSERT (fired on every shipped phase); a stamp older than feature's last touch = unreviewed = debt. | J-46:L51-L51 |
| J46-I001 | I/OBS | First baseline absolute rate should be framed as sequenced burn-down with promotion plans, not a damning headline; DELTA is the signal. | J-46:L50-L50 |

Category coverage: capabilities/tools/surfaces ✓ (CLI tool, FEATURE-MAP stamp, audit panel routing); workflows ✓; data/schemas ✓; config ✓; events net-zero ✓; failures/self-test ✓; determinism recompute ✓; observability/evidence JD-1 ✓; tests ✓; constraints ✓; deps ✓. No runtime UI window beyond stamp/panel routing.

### J-50 — recovery-trajectory SFT exhaust

Source: `docs/thefinalglance/J-plan/J-50_recovery_trajectory_sft_exhaust.md` (95/95 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| J50-E001 | E/STATUS | J-50 is PLANNED WAVE GUAVA-1, EXTENSION of I-30, effort M; depends I-30 (`sft_exhaust.rs`); binds IC-7, ID-6; Tier-1 offline pipeline now, Tier-2 LoRA RunPod-gated. | J-50:L3-L5 |
| J50-E002 | E/CAP | Extend I-30 walker: for each sealed RRA, extract chains that (a) contain >=1 turn with deterministic error OR L1 HARD-VETO OR failed gate, AND (b) continue through corrective turns to verified COMPLETE (`TriLayerVerdict(Promoted)` / PhaseComplete passing L1). | J-50:L10-L10 |
| J50-E003 | E/DATA | Full chain including error turn + correction turns becomes JSONL record with `label="recovery"`; distinct from I-30 label-less winning record and `label="negative"` vetoed pairs. | J-50:L10-L10, L12-L12 |
| J50-E004 | E/SEC | Same blake3(RRA_bytes) provenance hash; recovery record fails verification if hash does not match sealed RRA on disk (R2). | J-50:L10-L10, L59-L59, L71-L71 |
| J50-E005 | E/CFG | Named const `RECOVERY_TARGET_FRACTION` default 0.35, range [0.30, 0.40] per Guava 38.4%; exporter reports actual recovery fraction and emits WARN when corpus cannot supply target. | J-50:L10-L10, L28-L28 |
| J50-E006 | E/LIMIT | NEVER back-fill with synthetic or relabeled non-recovery chains (ID-5); under-target is reported, never fabricated. Vetoed-never-COMPLETE siblings stay `label="negative"`. | J-50:L10-L12, L59-L62 |
| J50-E007 | E/LIMIT | Orthogonal to I-5 PRISTINE CONTEXT INJECTOR: I-5 EXCLUDES failed-sibling tracebacks from INFERENCE context; J-50 INCLUDES failed-then-recovered chains in TRAINING data. Different stage (GD-5). | J-50:L12-L12, L88-L88 |
| J50-E008 | E/LIMIT | Dataset extraction only; SFT stays single I-30 run (IC-7: SFT before GRPO; no DPO E-55 HARD-AVOID; GRPO is I-31 LAST). No DPO/GRPO/PPO logic. | J-50:L12-L12, L63-L63, L87-L87 |
| J50-E009 | E/SURFACE | CLI `elai dataset export --format sft` (I-30 subcommand) also emits recovery records; `--recovery-fraction <f>` overrides named-const default. | J-50:L28-L28, L55-L55 |
| J50-E010 | E/SURFACE | I-30 dataset-card Svelte drawer gains `recovery_count` + `actual_recovery_fraction` + `under_target` badge; Tauri command + cross_lang_diff; operator-visible before LoRA run (W-6). | J-50:L29-L29, L39-L40, L66-L67 |
| J50-E011 | E/EVENT | Reuse SftExhaustEvent action_type; add Kind::`RecoveryExtracted` (+1 Kind arm, no new action_type). DatasetSealed carries recovery_count + actual_recovery_fraction. | J-50:L17-L17, L37-L38, L46-L46, L52-L52, L70-L71 |
| J50-E012 | E/INT | MODIFY sft_exhaust.rs (~90 LOC); ssb.rs + action_type_lock.rs (9-tuple lockstep); SCHEMA_VERSION computed `-309 + SSB_ACTION_TYPES.len()` at ssb.rs:1500 auto-follows; re-grep BOTH at edit W-9. | J-50:L37-L38, L79-L79 |
| J50-E013 | E/INT | Production reachability: `elai dataset export --format sft`; RecoveryExtracted rows + label=recovery JSONL on disk; test callers never R7. | J-50:L54-L56 |
| J50-E014 | E/TEST | Trigger export against sealed_rra_dir with known recovered run; expect RecoveryExtracted per chain with provenance + error-turn index + COMPLETE-turn index; JSONL provenance verifies; tampered recovery fails verify_rra → DatasetSealed(provenance_mismatch). NEGATIVE: zero recovered runs → zero recovery records, under_target WARN, fraction=0.0. | J-50:L69-L72 |
| J50-E015 | E/TEST | Contract test `crates/contracts/tests/j50_recovery_exhaust_contract.rs` + decision doc `workflow/reasoning/J-50-decisions.md`. | J-50:L30-L31 |
| J50-E016 | E/DEFER | Tier-2: LoRA run on recovery-enriched JSONL (I-30's existing checkbox); anti-slopsquat any LoRA lib + user confirmation before GPU run. | J-50:L33-L33 |
| J50-E017 | E/DEP | Consumes I-30 SftExhaust walker/writer/provenance; IS-10 verify_rra; I-4 search/ladder tree; ID-6 promotion chain unchanged. | J-50:L15-L22, L48-L52, L84-L88 |
| J50-E018 | E/EVIDENCE | Guava arXiv:2606.18363 §3.2 + App A.4 (743/1934=38.4%); Finding 3 success-only SFT collapses long-horizon. Scope caveat: embodied-manipulation 4B VLA sim — principle transfers; 38.4% is directional anchor not ELAI claim (GD-5/HD-1). | J-50:L5-L5, L76-L76 |
| J50-E019 | E/DEFER | Targeted-RL-on-bottleneck-tasks rider for I-31 (GRPO only on hard long-horizon tasks). Recovery-vs-success ablation = SAFEDELTA measurement once Tier-2 LoRA exists. | J-50:L92-L93 |
| J50-E020 | E/LIMIT | No unsealed RRA as source; no `.unwrap()`/`.expect()`; no blake2/sha2; no new `ELAI_*_ENABLE`. | J-50:L64-L64, L80-L80 |
| J50-A001 | A/CFG | Exact error/veto-turn detection predicates on tree node types are described in prose without an exhaustive enum of action types. | J-50:L10-L10 |
| J50-I001 | I/DATA | RecoveryExtracted payload fields include at least source RRA provenance hash, error-turn index, and COMPLETE-turn index (from expect_ssb). | J-50:L70-L71 |

Category coverage: capabilities/CLI/surfaces ✓ (export CLI, dataset-card drawer badges); workflows ✓; data/labels ✓; config fraction ✓; events ✓; security provenance ✓; failures ✓; determinism hash ✓; observability ✓; tests ✓; constraints ✓; deps/deferred ✓.

### J-60 — layered observability canvas (process altitude)

Source: `docs/thefinalglance/J-plan/J-60_layered_observability_canvas.md` (239/239 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| J60-E001 | E/STATUS | J-60 is PLANNED WAVE CFW-3, tier L, BUILD-on-shipped observability/read only; depends shipped H-13/H-15 canvas + J-31 (system-layer sibling); composes Tier-C + I-2 (PLANNED worktree) + phase_transition SSB (B-2 WIRED). Redirect deferred to J-61. Root decision observability-first in `workflow/reasoning/J-60-decisions.md`. | J-60:L3-L5, L17-L19 |
| J60-E002 | E/SURFACE | PROCESS layer is a third altitude over J-31: PROCESS (FsmState / AgentRole worker / phase node) → SYSTEM (J-31 bound units) → SCRIPT (capability/tool/MCP leaf). One graph, semantic zoom, NOT three graphs. | J-60:L23-L26 |
| J60-E003 | E/DATA | ProcessNode `{ node_id, binding: FsmSession(session id) | AgentWorker(agent_id) | PhaseStep(phase id), display }` derived from phase_transition rows (B-2) + agent_pool lifecycle/SSB rows. No parallel state file; re-derives under ELAI_SEED (GD-2). LLM never sets node status (TD-3). | J-60:L27-L31 |
| J60-E004 | E/SURFACE | TAB = DESCEND: opening an agent's tab IS zooming into its AgentWorker process node. Tab is a filtered VIEW of the one journal scoped to agent_id, never first-class state object. Descend reveals: agent task + FSM substate, tool-call stream (SSB rows), SYSTEM/SCRIPT subgraph touched, worktree diff. | J-60:L32-L36 |
| J60-E005 | E/SURFACE | Per-agent worktree view projects I-2 COW worktree when present; canvas does not HOLD it. Until I-2: per-agent diff derived from journaled taint-tagged op-stream / H-49 pre-image (`file_ops.rs:272/300` original; RECON says real capture is `timetravel.rs:310 capture_file`). When I-2 lands, live worktree replaces diff source; UI does not change. | J-60:L37-L45, L70-L70, L209-L211 |
| J60-E006 | E/LIMIT | EMIT-ONLY: engine projects, user navigates (zoom, open tab, H-15 deixis-select with G-1 taint-gated read-back) but issues NO graph mutation. Single-writer journal is read-only to canvas (SsbViewer WAL poll). Redirect verbs and user-preempt conflict model are J-61. | J-60:L46-L49, L119-L119 |
| J60-E007 | E/SURFACE | Deixis-only descend: process node collapses to ONE labeled node until user descends; ascending re-collapses; never auto-expand (anti-node-soup vs Langflow/Flowise plateau). | J-60:L50-L53, L144-L144 |
| J60-E008 | E/CFG | Tier-1: process altitude + nodes from phase_transition + agent-pool lifecycle + tab=descend + worktree diff from op-stream + deixis-only. Derivation auditor/idle-tick, NEVER render hot path. Tier-2: live I-2 worktree render; skill-grain process nodes when E-43 promoted+allowlisted; cross-altitude operates-on edges from run provenance. | J-60:L78-L85 |
| J60-E009 | E/LIMIT | skill_system_event is DARK (E-43, `ELAI_SKILL_SYSTEM_ENABLE`); v1 uses phase_transition + agent-lifecycle grain ONLY, no E-43 dependency. | J-60:L74-L76, L83-L84 |
| J60-E010 | E/INT | NEW `process_canvas.rs` (~220 LOC): process-node registry + journal derivation + worktree-diff projection. Frontend: canvas.svelte.ts altitude axis + ProcessLayer/AgentTab under `src/lib/canvas/`. Dispatcher READ-ONLY — reuse spawn_view / CanvasViewEvent for tab-open (`production_agent_rpc.rs:1299`). | J-60:L89-L96, L105-L112 |
| J60-E011 | E/EVENT | PREFER-REUSE — no new mutation action_type. Optional +ProcessViewProjected (Kind: `ProcessNodeDerived` / `AgentTabOpened` / `WorktreeDiffRendered`) ONLY if telemetry beacon wanted; else reuse CanvasViewEvent. baseline +0..+1 from LIVE 241/550 / I-drain ~268/577. Default PREFER-REUSE. | J-60:L97-L101, L176-L177, L206-L206 |
| J60-E012 | E/INT | Production: dashboard CanvasWindow PROCESS mode; tab-open via spawn_view production path; test/contracts never R7. | J-60:L114-L116 |
| J60-E013 | E/LIMIT | Stub bans: no parallel canvas state; no redirect/mutation verb; no LLM-set status; no worktree diff fabricated from model output; no auto-expand; no `.unwrap()`/`.expect()`; blake3 only. | J-60:L117-L122 |
| J60-E014 | E/TEST | Trigger: FSM task spawning >=2 agent_pool workers; open PROCESS canvas; descend agent. Expect phase_transition + agent-pool rows; optional ProcessViewProjected. Artifact: each live agent as process node; tab with scoped task/tool-stream + op-stream worktree diff; byte-identical re-derive. NEGATIVE: model-sourced status FAILS; zero journal rows renders NOTHING. | J-60:L125-L128 |
| J60-E015 | E/TEST | Contract `j60_process_canvas_contract.rs` (~120 LOC): (a) journal derivation, (b) ELAI_SEED byte-identical, (c) TD-3 rejection, (d) zero-rows→no node, (e) op-stream/H-49 diff source, (f) spawn_view reachability. | J-60:L158-L159, L194-L194 |
| J60-E016 | E/TEST | DETERMINISTIC_CHECK + ACCEPTANCE: GD-2, TD-3, R1/R2, W-2 ships ON (no kill-switch), deixis-only, cargo check GREEN, auditor PASS, no new dependency. | J-60:L130-L146 |
| J60-E017 | E/CFG | W-2: ships ON (read-only observability has no kill-switch graveyard risk); no default-OFF `ELAI_*_ENABLE` introduced. | J-60:L143-L143, L150-L150 |
| J60-E018 | E/DEP | Consumes FsmState/transition (fsm.rs), AgentPool/AgentRole, InProcessCanvasDispatcher + spawn_view, H-49 pre-image / EditHistoryEventPayload, J-31 system_canvas binding model (do not re-implement). Tier-C process isolation ships identity for tabs. | J-60:L57-L73, L108-L113 |
| J60-E019 | E/EVIDENCE | Grounding: elai.map FsmState/AgentPool anchors; visual-builder complexity plateau web-search 2026-06-20; AgentCoord arXiv:2404.11943; Dynamic Runtime Graphs arXiv:2603.22386. | J-60:L169-L171 |
| J60-E020 | E/FAIL | ABORT if dependency surface (FsmState, AgentPool, dispatcher, spawn_view, CanvasViewEvent, PhaseTransition SSB, EditHistoryEvent) is genuinely DARK at edit; if byte-identical re-derive fails after 2 attempts; if cargo check fails outside J-60 changeset. | J-60:L215-L218 |
| J60-A001 | A/SURFACE | Which AgentRole's worktree the tab renders pre-I-2 is unresolved (executor not auditor); if no isolatable tree, v1 worktree diff may be per-FSM-session not per-agent (FORK-3). | J-60:L174-L175, L207-L207, L212-L212 |
| J60-A002 | A/DATA | phase_transition SSB action_type name is PLAN-asserted; if grain is per-transition not per-phase, ProcessNode binding adjusts (FORK-1). | J-60:L175-L176, L205-L205, L209-L210 |
| J60-A003 | A/EVENT | Whether optional ProcessViewProjected earns golden-lock cost vs CanvasViewEvent reuse is decide-at-authoring; default PREFER-REUSE. | J-60:L176-L177, L206-L206 |
| J60-C001 | C/DEP | Plan cites `file_ops.rs:272/300` as pre-image capture; RECON_NEEDED records that anchor as STALE and names real site `timetravel.rs:310 capture_file`. Executor must re-grep. | J-60:L42-L42, L70-L70, L211-L211 |
| J60-I001 | I/SURFACE | CanvasMode union is expected to include `'process'` (wargame EXPECTED_OBSERVATION) as the altitude/mode entry for PROCESS; confirm against current union at edit. | J-60:L192-L192, L213-L213 |

Category coverage: capabilities/surfaces ✓ (PROCESS mode, ProcessLayer, AgentTab, descend/ascend, worktree view); workflows/states ✓; schemas ✓; config tiers ✓; events optional ✓; security/taint badges via G-1 ✓; failures ✓; determinism ✓; observability ✓; tests/wargame ✓; constraints ✓; deps/deferred ✓. Privacy not separately specified.

### J-69 — SSB log-integrity hash chain + watermark monotonicity

Source: `docs/thefinalglance/J-plan/J-69_ssb_log_integrity_hash_chain.md` (68/68 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| J69-E001 | E/STATUS | J-69 is Track J PROVABLE, STRONG raft-interface W12 confirmation of I-18/J-22, nature EXTENSION(I-18 + J-22), effort M, WAVE NASA-GATES-2, depends none (extends I-18 chain producer + J-22 writer isolation). W12 is independent confirmation; J-69 is consumer SCAN-GATE + watermark rider, NOT a competing chain (NG-2 / GROUP F). | J-69:L3-L5 |
| J69-E002 | E/CAP | Chain-scan gate: O(n) scan verifying `prev_hash = blake3(canonical_prev_row_bytes)` for every row; broken link = BLOCK at session-open and at `rra ... --verify`, BEFORE O(n) RRA replay. | J-69:L9-L9, L28-L28, L37-L37 |
| J69-E003 | E/CAP | Watermark-monotonicity gate: new session's first row watermark must be >= prior committed watermark (Leader-Completeness analog); regression = BLOCK (rolled-back/forked/replayed-stale journal). | J-69:L10-L10, L29-L29, L48-L48 |
| J69-E004 | E/DET | Canonical-bytes function is shared with J-68's serializer round-trip harness (J-68 proves determinism; J-69 depends on it); blake3 keyed per I-18 algorithm-prefix convention. | J-69:L11-L11, L34-L34, L54-L54, L62-L62 |
| J69-E005 | E/LIMIT | Excluded: new chain producer (I-18 owns chain_prev_hash; reimpl = GD-1 P0); new writer/commit point (J-22 sole-commit; J-69 read-only); sha256/blake2; auto-repair of broken chain (surface + BLOCK, never rewrite history, RV-4). | J-69:L12-L12, L40-L43 |
| J69-E006 | E/DATA | If chain_prev_hash COLUMN does not yet exist at edit (I-18 unshipped), J-69 first task is DDL field add — SCHEMA migration that bumps `SCHEMA_VERSION_DDL_EPOCH` (`ssb.rs:1503`), NOT a new action_type (`SSB_ACTION_TYPES.len()` unchanged); re-read live epoch (W-9). | J-69:L23-L23, L30-L30, L55-L55, L61-L61 |
| J69-E007 | E/EVENT | No new action_type; scan emits verdict through existing audit/RRA surface, or single ChainScanVerdict reuse if one exists. LIVE baseline 268/577; net 0 new action_type. | J-69:L23-L23, L55-L55 |
| J69-E008 | E/SURFACE | Frontend pair: chain-integrity status line on existing RRA/review surface (decide at edit, W-4) or W-6-justified absence. | J-69:L24-L24, L45-L45 |
| J69-E009 | E/INT | Produces `ssb::scan_chain` + `ssb::check_watermark_monotonicity` (+ conditional DDL). Consumes I-18 chain fields, J-22 sole-commit, J-68 canonical-bytes, G-2 RRA `--verify`. | J-69:L27-L35 |
| J69-E010 | E/INT | Production reachability: session-open on real `elai` run + `elai rra <bundle> --verify`; test/contracts never R7. | J-69:L36-L38 |
| J69-E011 | E/FAIL | "Chain absent" treated as pass is FORBIDDEN: missing chain field at session-open is FAIL (R2/ND-3 dead evidence path screams). | J-69:L44-L44, L49-L49 |
| J69-E012 | E/TEST | Trigger: hand-mutate ONE byte of one committed SSB row payload then session-open and `elai rra --verify`. Expect broken link at mutated position and BLOCK before RRA replay; watermark regression BLOCKS. NEGATIVE: untampered scans clean; no chain field FAILS; reordered pair fails scan (wrong predecessor hash); truncated journal FAILS never passes short chain as complete. | J-69:L46-L49 |
| J69-E013 | E/PERF | O(n) every session-open on long journal: bound by scanning from last verified watermark forward, persisting verified-watermark, never re-scanning proven history; LOG scanned range (no silent partial scan as full, R2). | J-69:L63-L63 |
| J69-E014 | E/DEP | Composes I-18 (producer), J-22 (sole-commit), J-68 (canonical-bytes proof), J-66 (monitors run only on J-69-verified journal), G-2 RRA, H-49 (separate CAS concern). | J-69:L57-L58 |
| J69-E015 | E/EVIDENCE | W12 raft-interface.md (Log-Matching → prev_hash; Leader-Completeness → watermark). Substrate I-18+J-22 PLAN-asserted. SSB single-writer grep-confirmed 2026-06-22. | J-69:L5-L5, L14-L20, L51-L55 |
| J69-A001 | A/DEP | Order of ship vs I-18: coordinate so there is ONE chain producer; if I-18 unshipped J-69 does minimal behavior-preserving DDL field add. | J-69:L61-L61 |
| J69-A002 | A/SURFACE | Exact RRA/review chain-integrity status placement is decide-at-edit. | J-69:L24-L24, L45-L45 |
| J69-I001 | I/DET | Scan and producer MUST hash identical serialization or every scan fails spuriously; pinning J-68 before either ships is load-bearing. | J-69:L62-L62 |

Category coverage: capabilities/gates/surfaces ✓ (session-open, rra verify, RRA status line); workflows ✓; data/DDL ✓; config ✓; events net-zero ✓; security integrity ✓; failures/BLOCK ✓; determinism blake3 ✓; performance bound ✓; tests ✓; constraints ✓; deps ✓.

### J-81 — autonomous supervision hardening

Source: `docs/thefinalglance/J-plan/J-81_autonomous_supervision_hardening.md` (111/111 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| J81-E001 | E/STATUS | J-81 is PLANNED WAVE KAT-1, effort M, EXTENSION(auto-orchestrate/orchestrate supervision + E-39), deterministic supervision legs, no new subsystem. | J-81:L3-L3 |
| J81-E002 | E/CAP | Process-wedge detector: output-signature (size:mtime + content-hash of worker's last-output region) compared across N supervision cycles; identical for N cycles with no busy-signature = WEDGE → escalate via existing T4 surface. Deterministic, no LLM. JD-1 evidence = hook-log row. | J-81:L5-L5, L30-L30, L69-L69 |
| J81-E003 | E/CAP | Durable wake-queue: persist supervision wake set so supervisor crash resumes loop; compose SESSION_HANDOFF + singleton lock + signal coalescing (firstmate FM_SIGNAL_GRACE=30s analog). Replaces naive dep-wait sleep-poll volatility; poll stays fallback transport. | J-81:L5-L5, L31-L31, L70-L70 |
| J81-E004 | E/CAP | Mid-loop deterministic cost/iteration/wallclock cap: wire E-39 ceiling into `task_runner` drive loop as mid-step check that aborts CURRENT step (AbortController-equivalent), distinct from dispatch-count envelope. Deterministic predicate ONLY. | J-81:L5-L5, L32-L32, L39-L39, L71-L71 |
| J81-E005 | E/LIMIT | NOT gnhf LLM-self-judged stopWhen/should_fully_stop (TD-3). NOT `git reset --hard` rollback (use H-49 CAS / RV-1). NOT firstmate `--dangerously-*` bypasses. NOT hierarchical secondmates (OFF-WEDGE vs central-FSM-gate trust spine). | J-81:L5-L5, L48-L50, L73-L73 |
| J81-E006 | E/STATUS | Existing gaps: auto-orchestrate envelope counts DISPATCHES not mid-run tokens; anti-spiral is REASONING-level blake3 3-hash stop; orchestrate T4 is sweep-fail-3x/fixer-3x; E-39 HARD CEILING is DARK with NO task_runner caller. | J-81:L5-L5, L16-L19 |
| J81-E007 | E/DEFER | Riders recorded in-place (no new files): gnhf commit-repair + backoff failure-taxonomy → bubble-up.md; turn-end completion signal (claude Stop-hook file touch) → B-37 / J-64 for external vendor CLIs. | J-81:L33-L33 |
| J81-E008 | E/INT | Produces: harness tools (supervision_wedge.py + wake_queue.py ~110 LOC stdlib); task_runner mid-loop cap (~50 LOC). Consumes auto-orchestrate/orchestrate envelope + T4; SESSION_HANDOFF/PreCompact; E-39 ceiling (wire, never re-implement); bubble-up retry budgets. | J-81:L37-L42, L90-L96 |
| J81-E009 | E/INT | Production: orchestrate/auto-orchestrate supervision loop → wedge + wake-queue (JD-1 hook-log); task_runner → mid-loop cost cap (E-39 caller that did not exist); test/contracts never satisfy runtime leg R7. | J-81:L43-L46 |
| J81-E010 | E/SURFACE | frontend_pair N/A justified: supervision is harness/runtime; wedge/abort surfaces through existing T4 escalation + cost panels (H-9/CloudPanel) the cost cap already feeds. | J-81:L52-L53, L80-L80 |
| J81-E011 | E/EVENT | Harness legs = JD-1 hook-log/state.json (no SSB); cost-cap leg = prefer-reuse E-39 ceiling rows (no new action_type unless E-39 needs one — decide at edit from LIVE ~267/577, W-9). | J-81:L56-L56, L84-L84 |
| J81-E012 | E/TEST | Trigger: worker emits no output for N cycles; crash supervisor mid-loop; step exceeds token/wallclock cap. Expect WEDGE flagged+escalated; wake set resumes; runaway step aborts current step (not only next dispatch-count check); LLM-self-judged stop MUST NOT be the gate. | J-81:L54-L57 |
| J81-E013 | E/TEST | Contract `j81_supervision_hardening_contract.rs`: (a) mid-loop cap aborts over-budget step, (b) wallclock cap deterministic, (c) stop predicate rejects should_fully_stop as gate, (d) harness wedge flags output-unchanged-N-cycles, (e) harness wake-queue resumes after simulated restart. | J-81:L87-L88 |
| J81-E014 | E/CFG | W-2: cost cap rides E-39's existing flag; new harness legs default-ON behind a `*_DISABLE` kill-switch. | J-81:L78-L78 |
| J81-E015 | E/TEST | DETERMINISTIC_CHECK: wiring_contract_check; rg wedge/output signature/wake.queue; rg mid.loop/abort/cost ceiling in task_runner; rg should_fully_stop count MUST be 0; J-81-decisions.md exists. | J-81:L59-L66 |
| J81-E016 | E/LIMIT | No re-implementation of E-39; no `.unwrap()`/`.expect()` on prod; wake-queue must not become second source of truth vs SESSION_HANDOFF (projection/extension, not parallel store). | J-81:L51-L51, L106-L106 |
| J81-E017 | E/EVIDENCE | steal-kunchen 2026-06-24 C1.2 (gnhf RunLimits) + C1.3 (firstmate fm-watch); firstmate/gnhf are macOS/Node — Rust+harness reimpl, no code copied; bun-not-node for any JS. | J-81:L5-L5, L98-L99 |
| J81-A001 | A/SCOPE | "output-unchanged" definition for ELAI Agent-tool dispatch (completion-notified) vs EXTERNAL vendor-CLI fleet (B-37/J-64) where output is the only liveness signal: scope detector to external-CLI / long-running-loop case; decide at authoring. | J-81:L104-L104 |
| J81-A002 | A/DEP | If E-39 ceiling types are wrong seam for task_runner caller, that is a material fork (STOP + handoff), not silent re-implement. | J-81:L105-L105 |
| J81-A003 | A/EVENT | Whether cost-cap needs a new action_type or pure E-39 row reuse is decide-at-edit. | J-81:L56-L56 |
| J81-I001 | I/FLOW | Escalation target for wedge is the existing T4 tripwire surface (sweep-fail-3x/fixer-3x family), not a new escalation subsystem. | J-81:L30-L30, L41-L41 |

Category coverage: capabilities/harness/runtime ✓; surfaces (T4/cost panels only, no new window) ✓; workflows ✓; config kill-switch ✓; events prefer-reuse ✓; security trust-boundary ✓; failures/abort ✓; determinism ✓; observability JD-1 ✓; tests ✓; constraints/DO-NOT-COPY ✓; deps/deferred ✓.

## Cross-file interaction records

| Interaction ID | Class | Interaction | Atom references |
|---|---|---|---|
| IX-001 | E | J-14 defines the content-keyed export-fixture format that I-49 and later J-91 re-executable RRA consume; J-14 is the fixture contract owner. | J14-E009–J14-E011 |
| IX-002 | E | J-14 first-divergence output is the G-10 canary consumer input; T1/T2 bases depend on J-13 sampling_seed. | J14-E002, J14-E006, J14-E022 |
| IX-003 | E | J-29 is the display half of J-28's usage ledger; mid-stream gauge is forbidden from writing SSB that J-28 owns as durable UsageTurnEvent. | J29-E001, J29-E006–J29-E008, J29-E020 |
| IX-004 | E | J-29 canvas usage badge and J-31 per-node cost/usage badge rider share the E-40 CanvasDispatcher surface; J-31 defers the badge until J-29 ships. | J29-E011; J31-E022 |
| IX-005 | E | J-29 absolute counts and J-34 curve/knee tile both extend ContextStatePanel / cost-cockpit family as complementary instruments (counts vs degradation). | J29-E012; J34-E015 |
| IX-006 | E | J-31 SYSTEM altitude is the sibling base for J-60 PROCESS altitude; J-60 reuses J-31 binding model and must not re-implement node binding. | J31-E003–J31-E008; J60-E001–J60-E002, J60-E018 |
| IX-007 | E | J-31 dictate-on-node soft-depends J-32 front door; chip fallback with prefilled node id bridges until J-32 ships. | J31-E010 |
| IX-008 | E | J-31 DRIFTED status Tier-2 consumes J-16 blast-radius cones; until then DKG file-hash staleness is the fallback. | J31-E009 |
| IX-009 | E | J-34 journal-regression Tier-2 hard-depends J-28 UsageTurnEvent keys that J-29 also displays; until then fixture bench stands alone. | J34-E011; J29-E001 |
| IX-010 | E | J-34 invalidation composes G-10 model-drift the same identity surface J-14's determinism/repro stack feeds. | J34-E010; J14-E006 |
| IX-011 | E | J-38 soft-depends J-18 hardened cascade; exposes hybrid_search via C-13 IpcToolDispatcher alongside B-29 tools; sufficiency targets C-29 IntentPair. | J38-E001–J38-E002, J38-E009, J38-E022 |
| IX-012 | E | J-38 PREFER-REUSE of QueryTrace / J-18 RetrievalAugmentEvent parallels J-60 PREFER-REUSE of CanvasViewEvent and J-50 Kind-arm reuse — net-zero pressure on the SSB registry. | J38-E010; J60-E011; J50-E011 |
| IX-013 | E | J-46 metric chain J-43 ledger → J-44 remediation velocity → J-45 calibration corpus seed → J-47 program falsifier consumer. | J46-E002, J46-E013 |
| IX-014 | E | J-50 extends I-30 SftExhaust surface and reuses SftExhaustEvent with +1 Kind; training-stage recovery inclusion is explicitly orthogonal to I-5 inference exclusion. | J50-E001–J50-E008, J50-E011 |
| IX-015 | E | J-60 full live worktree tabs hard-depend I-2 COW isolation; until then H-49 op-stream/timetravel pre-image is the diff source (same pre-image family J-14 persists as snapshot hashes). | J60-E005, J60-C001; J14-E007 |
| IX-016 | E | J-60 redirect/steer verbs are owned by J-61; J-60 is emit-only WATCH half of the layered canvas pair. | J60-E001, J60-E006 |
| IX-017 | E | J-69 is the integrity precondition J-66 temporal monitors and J-67 journaled Insert/Edit rely on; scan runs BEFORE G-2 RRA O(n) replay. | J69-E002, J69-E014 |
| IX-018 | E | J-69 verifies I-18's chain and trusts J-22 sole-commit; shares canonical-bytes with J-68; may perform minimal DDL if I-18 unshipped. | J69-E001, J69-E004–J69-E006 |
| IX-019 | E | J-81 wires dark E-39 into task_runner (missing caller); J-29 leaves CostPanel/E-39 UI untouched while J-81 feeds the same cost panels via abort outcomes. | J81-E004, J81-E006, J81-E010; J29-E003 |
| IX-020 | E | J-81 hardens auto-orchestrate/orchestrate loops that J-60 observes as process nodes; wedge escalation reuses T4 rather than inventing a parallel subsystem. | J81-E001–J81-E003, J81-I001; J60-E003 |
| IX-021 | E | J-14 ReproPackEvent, J-31 SystemContractEvent, J-34 ContextCalibrationEvent, and (conditional) J-38 RetrievalLoopEvent each add baseline+1 Pattern-2 families requiring W-9 live registry reread; J-29/J-46/J-69/J-81 prefer net-zero. | J14-E017; J31-E018; J34-E014; J38-E010; J29-E015; J46-E011; J69-E007; J81-E011 |
| IX-022 | E | Cross-chunk composition: J-48 reuses RouteDecisionEvent and J-29 read-side; J-49 multi-seat UI breakdown rides J-29; J-36 uses J-34 measured tail budget; J-39 admission uses J-34 ceilings. | J29-E002; J34-E003–J34-E005 (orientation from 10-J summary composition lines) |
| IX-023 | I | Canvas triad sequencing in family summary: J-31 system base, J-60 process watch, later J-61 steer and J-98 memory altitude — this chunk owns the system+process watch pair only. | J31-E001; J60-E001–J60-E002 |
| IX-024 | E | Blake3-only hashing is uniform: J-14/J-31/J-50/J-60/J-69 ban sha256/blake2; score equality in J-14 is exact never epsilon. | J14-E005, J14-E015; J31-E011; J50-E004, J50-E020; J60-E013; J69-E005 |
| IX-025 | E | R7 production-caller discipline is explicit in J-14, J-29, J-38, J-50, J-60, J-69, J-81; harness/doc legs (J-46, J-81 harness) use JD-1 off-agent evidence instead of SSB reachability. | J14-E019; J29-E016; J38-E012; J46-E012; J50-E013; J60-E012; J69-E010; J81-E009 |

## Explicit ambiguity and conflict register

| Register ID | Kind | Unresolved issue or required correction | Atom references |
|---|---|---|---|
| AC-001 | ambiguity/config | J-14 tier vocabulary mapping (T0/T1/T2 vs ActionFaithful/TokenFaithful/ScoreFaithful) and entropy-scan vehicle are decide-at-edit. | J14-A001, J14-A003 |
| AC-002 | ambiguity/surface | J-14 and J-69 RRA/review surface placements for tier/divergence and chain-integrity status are W-4 decide-at-edit. | J14-A002; J69-A002 |
| AC-003 | ambiguity/data | J-29 autonomy-ratio source action types, chart dependency confirmation, and local-lane task_id propagation remain edit-time. | J29-E010, J29-A001–J29-A002, J29-E021 |
| AC-004 | ambiguity/data | J-31 may ship local SystemBinding/EdgeContract types if Phase-3G runtime types are absent. | J31-A001, J31-E015 |
| AC-005 | ambiguity/measurement | J-34 knee algorithm, exact fill grid, and cloud workhorse/budget constants are underspecified beyond policy bounds. | J34-A001–J34-A002 |
| AC-006 | ambiguity/event | J-38 SSB reuse vs new RetrievalLoopEvent, identifier classifier FORK, and threshold/max_rounds values are open. | J38-A001–J38-A003 |
| AC-007 | ambiguity/data | J-46 denominator definition and implausible-zero reviewed-count threshold are named but unfixed. | J46-A001–J46-A002 |
| AC-008 | ambiguity/schema | J-50 error/veto-turn detection lacks an exhaustive action-type enum. | J50-A001 |
| AC-009 | conflict/anchor | J-60 cites file_ops.rs:272/300 for pre-image while RECON records that site as STALE and names timetravel.rs:310. | J60-C001 |
| AC-010 | ambiguity/surface | J-60 AgentRole/worktree grain, phase_transition binding grain, and ProcessViewProjected vs CanvasViewEvent remain forks. | J60-A001–J60-A003 |
| AC-011 | ambiguity/scope | J-81 wedge detector scope (external-CLI vs native Agent dispatch) and E-39 seam fitness are authoring forks. | J81-A001–J81-A002 |
| AC-012 | dependency | J-69 may perform chain_prev_hash DDL if I-18 unshipped; must coordinate to avoid dual producers. | J69-A001, J69-E006 |
| AC-013 | dependency | J-38 works without J-18 but at lower retrieval quality; J-31 DRIFTED and J-34 regression legs wait on J-16/J-28. | J38-E001; J31-E009; J34-E011 |

## Summary additions and corrections

Orientation comparison target: `docs/thefinalglance/feature-discovery/10-J-SUMMARY_INDEX.discovery.md`. “ADD” means an explicit source behavior omitted or materially compressed by the summary; “CORRECT/QUALIFY/REMOVE” means the summary’s wording changes scope or certainty. These are relational records, not additional feature atoms.

### J-14 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-J14-001 | **ADD** exact LIVE registry baseline 229/540, full ReproPackEvent kind list, production CLI verbs, and crystallizer compare-at-execute path. | 10-J:L172-L182 | J14-E012–J14-E013, J14-E017, J14-E019 |
| SD-J14-002 | **ADD** detllm gotchas (hash excludes own field, key-sorted serialization, exact score equality), hash-pair default + content opt-in, and G-9 flag-independence of scrub. | 10-J:L174-L179 | J14-E005, J14-E008, J14-E024 |
| SD-J14-003 | **ADD** negative evidence rules (missing determinism_applied fails; scrub grep; LLM fallback on fingerprint mismatch) and I-49 fixture-format ownership. | 10-J:L180-L182 | J14-E010–J14-E011, J14-E021 |
| SD-J14-004 | **ADD** open decisions: tier vocabulary mapping, frontend placement, entropy-scan vehicle, and observe-only JD-3 posture. | 10-J:L172-L182 | J14-A001–J14-A003, J14-E014 |

### J-29 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-J29-001 | **ADD** complete Tauri command set, 1s CommandWindow poll, CLI single-line tick, gauge reset-on-turn-end, and net-zero 234/543 baseline. | 10-J:L326-L335 | J29-E004–J29-E008, J29-E015 |
| SD-J29-002 | **ADD** explicit gate-off / empty / unknown-backend states, no-removal of CloudPanel/SpendDrawer/CostPanel, and harness-canvas out-of-scope. | 10-J:L328-L335 | J29-E003, J29-E005, J29-E011, J29-E017–J29-E018 |
| SD-J29-003 | **ADD** Tier-2 remote-sidecar local-gauge deferral, poll-vs-push GD-4 rule, chart-dep confirmation, and autonomy-ratio experimental escape. | 10-J:L326-L335 | J29-E013, J29-E021–J29-E022, J29-A001–J29-A002 |
| SD-J29-004 | **ADD** DETERMINISTIC_CHECK recipe, decisions doc, and BackendCostPanel-absence / pricing_rollup-first-consumer framing. | 10-J:L328-L335 | J29-E004, J29-E019, J29-E023 |

### J-31 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-J31-001 | **ADD** exact SystemNode/SystemEdge shapes, dispatcher op names, system_canvas.rs module, and SystemContractEvent kinds with 241/550 baseline. | 10-J:L359-L368 | J31-E004–J31-E007, J31-E017–J31-E018 |
| SD-J31-002 | **ADD** Tier-1 vs Tier-2 split (verified automation, J-16 DRIFTED, J-32 dictate, edge taint badges deferred) and auditor/idle-tick derivation rule. | 10-J:L361-L368 | J31-E009–J31-E010, J31-E016 |
| SD-J31-003 | **ADD** canvas-not-default invariant, CODE vs SYSTEM dual-truth window, Phase-3G type uncertainty, and declined runtime data-flow. | 10-J:L359-L368 | J31-E013–J31-E015, J31-E021, J31-A001 |
| SD-J31-004 | **ADD** full negative suite (self-report verdicts, deleted consumer, MCP rug-pull untrust, nonexistent plan-unit bind fail) and J-29 badge rider. | 10-J:L367-L368 | J31-E019–J31-E020, J31-E022 |

### J-34 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-J34-001 | **ADD** exact ContextCalibrationEvent kinds, R1 logging of ThresholdApplied/PlacementApplied, and E-39-capped cloud fixture budget rule. | 10-J:L381-L391 | J34-E012, J34-E014 |
| SD-J34-002 | **ADD** HD-2 per-model-static layout, bp.valid assert, mid-session artifact-delete non-churn, and declined live rot meter. | 10-J:L388-L391 | J34-E008, J34-E017–J34-E018 |
| SD-J34-003 | **ADD** I-6/E-32 composition, G-10 identity_key cache key, manual `--recalibrate` until G-10 auto, and K-1 publishing rider. | 10-J:L386-L391 | J34-E009–J34-E011, J34-E019 |
| SD-J34-004 | **ADD** folklore anchors (0.85, Reddit 42/67) and probe-primitive distinction (E-49 pass/fail canary ≠ calibration curve). | 10-J:L381-L391 | J34-E006, J34-E020 |

### J-38 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-J38-001 | **ADD** full event kind list, PREFER-REUSE vs new-event FORK, IpcToolDispatcher seam, and wargame unit order (sufficiency first). | 10-J:L428-L437 | J38-E010, J38-E018–J38-E021, J38-A001 |
| SD-J38-002 | **ADD** identifier-query BM25+graph routing OQ/FORK, final-round rerank measure-only, and empty-neighbor zero-filler B6 rule. | 10-J:L430-L437 | J38-E008, J38-E016–J38-E017, J38-A002 |
| SD-J38-003 | **ADD** observe-only >=3-run calibration before default-ON, literature numbers as design-rationale only, and soft J-18 quality caveat. | 10-J:L428-L437 | J38-E001, J38-E015, J38-E023 |
| SD-J38-004 | **ADD** REDTEAM dual-path patch (positive sufficiency fire required) and INTEGRATION_SMOKE hard inputs. | 10-J:L436-L437 | J38-E013–J38-E014, J38-E021 |

### J-46 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-J46-001 | **ADD** EXP-DP4 observe-only WIRED-first sequencing, background B..K burn-down, and nothing-blocking-before-baseline+FP-rates rule. | 10-J:L510-L518 | J46-E004–J46-E005 |
| SD-J46-002 | **ADD** FEATURE-MAP UPSERT seam mechanics, stamp staleness→unreviewed, and optional DoctrineDebtEvent alternate source. | 10-J:L515-L518 | J46-E006, J46-E011, J46-E015 |
| SD-J46-003 | **ADD** exact tool path doctrine_debt_rate.py, JD-1 gamed-ledger self-test, and Q4 surrogation/remediation-velocity doctrine. | 10-J:L512-L518 | J46-E002, J46-E007–J46-E008, J46-E012 |
| SD-J46-004 | **ADD** open denominator/threshold constants and backlog-as-sequencing-fact framing. | 10-J:L517-L518 | J46-A001–J46-A002, J46-I001 |

### J-50 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-J50-001 | **ADD** exact COMPLETE criteria (TriLayerVerdict Promoted / PhaseComplete L1), RecoveryExtracted payload indices, and DatasetSealed provenance_mismatch path. | 10-J:L551-L559 | J50-E002, J50-E011, J50-E014, J50-I001 |
| SD-J50-002 | **ADD** `--recovery-fraction` CLI override, computed SCHEMA_VERSION rule, and IC-7/E-55/I-31 training-order hard boundaries. | 10-J:L551-L559 | J50-E008–J50-E009, J50-E012 |
| SD-J50-003 | **ADD** Guava scope caveat (embodied sim, directional 38.4%), I-5 orthogonality statement, and I-31 bottleneck-RL rider. | 10-J:L553-L558 | J50-E007, J50-E018–J50-E019 |
| SD-J50-004 | **ADD** dataset-card under_target badge as W-6 operator gate before LoRA and Tier-2 RunPod/anti-slopsquat confirmation. | 10-J:L559-L559 | J50-E010, J50-E016 |

### J-60 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-J60-001 | **ADD** three-altitude PROCESS→SYSTEM→SCRIPT model, tab=descend unification, and emit-only vs J-61 steer split. | 10-J:L655-L662 | J60-E002, J60-E004, J60-E006 |
| SD-J60-002 | **ADD** optional ProcessViewProjected kinds, PREFER-REUSE default, ships-ON W-2 (no enable flag), and skill-grain E-43 Tier-2 gate. | 10-J:L655-L662 | J60-E008–J60-E009, J60-E011, J60-E017 |
| SD-J60-003 | **ADD** stale file_ops vs timetravel capture RECON, AgentRole/session-diff FORKs, and ABORT conditions. | 10-J:L657-L662 | J60-A001–J60-A003, J60-C001, J60-E020 |
| SD-J60-004 | **ADD** full contract test matrix (a–f), INTEGRATION_SMOKE R10 cross-seam, and deixis anti-node-soup category grounding. | 10-J:L660-L662 | J60-E007, J60-E014–J60-E015, J60-E019 |

### J-69 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-J69-001 | **ADD** verified-watermark partial-scan performance bound with logged range, and J-68 canonical-bytes pin dependency. | 10-J:L741-L748 | J69-E004, J69-E013, J69-I001 |
| SD-J69-002 | **ADD** DDL epoch-bump mechanics (not action_type), ChainScanVerdict prefer-reuse, and missing-chain FAIL (ND-3). | 10-J:L744-L748 | J69-E006–J69-E007, J69-E011 |
| SD-J69-003 | **ADD** reordered-row and truncated-journal negative cases, and J-66/J-67 precondition framing. | 10-J:L747-L748 | J69-E012, J69-E014 |
| SD-J69-004 | **ADD** I-18 coordination open risk for single chain producer. | 10-J:L744-L746 | J69-A001, J69-E006 |

### J-81 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-J81-001 | **ADD** exact harness file set (supervision_wedge.py, wake_queue.py), task_runner caller, and bubble-up/B-37/J-64 riders. | 10-J:L857-L865 | J81-E007–J81-E008 |
| SD-J81-002 | **ADD** DO-NOT-COPY set (reset-hard, dangerously-*, secondmates, LLM stopWhen), W-2 DISABLE kill-switch for harness legs, and wake-queue-not-parallel-store rule. | 10-J:L859-L865 | J81-E005, J81-E014, J81-E016 |
| SD-J81-003 | **ADD** external-CLI vs native Agent wedge-scope OQ and E-39 seam STOP+handoff fork. | 10-J:L857-L865 | J81-A001–J81-A002 |
| SD-J81-004 | **ADD** contract arms (a–e), JD-1 vs E-39 row evidence split, and frontend_pair N/A via T4+cost panels. | 10-J:L860-L865 | J81-E010–J81-E013 |

## Exhaustiveness, zero-omission, and duplicate audit

- Source coverage: **10/10 files**, **1,220/1,220 logical lines**, **133,280/133,280 bytes** read and reconciled; source SHA mismatches **0**.
- Atomic source ledger: **233 unique atoms** = **200 explicit**, **010 inferred**, **022 ambiguous**, **001 conflicting/corrective**.
- Per-file atom counts: J-14 **028**; J-29 **026**; J-31 **025**; J-34 **023**; J-38 **027**; J-46 **018**; J-50 **022**; J-60 **025**; J-69 **018**; J-81 **021**. Sum **233**.
- Relational interaction records: **25** (`IX-001`–`IX-025`). Summary-delta records: **040** (`SD-*`). Ambiguity/conflict register: **13** (`AC-001`–`AC-013`). These records reference source atoms and are not double-counted as features.
- ID audit: every atomic table row has exactly one unique ID matching `J14/J29/J31/J34/J38/J46/J50/J60/J69/J81-[EIAC]nnn`. Duplicate atomic IDs **0**; missing/blank atomic IDs **0**; nonconforming atomic IDs **0**.
- Semantic duplicate audit: repeated premise/scope/wiring/grounding/acceptance statements were merged into one atom with multiple anchors. Interactions, conflict register, and summary deltas reference atoms instead of cloning them. Duplicate feature atoms found **0**.
- Omission audit: every source was checked against capabilities/actions/surfaces; workflows/states; schemas/data/lifecycle; flags/defaults/config; integrations/events; permissions/security/privacy/compliance; failures/fallback/retry/recovery; determinism/idempotency/concurrency; observability/evidence; tests/acceptance/done gates; constraints/non-goals; dependencies/status/deferred. Each per-file ledger explicitly names absent dimensions rather than inventing them. User-facing windows/panels/CLI verbs/flags/status displays were captured as standalone SURFACE/CAP/CFG atoms (UsageWindow, CommandWindow meter, CLI ticks, canvas modes/tabs/badges, ContextStatePanel tiles, dataset-card badges, RRA review status lines, FEATURE-MAP stamps, KnowledgeDrawer breakdown, `elai replay --verify`, `elai rra export-fixture`, `elai bench context-fill`, `elai dataset export`, harness tools).
- Interpretation audit: inferred, ambiguous, and conflicting/corrective atoms are visibly separated by ID/class. No inference is promoted to a requirement. Orientation-only claims remain in delta records and are not source atoms.
- Manifest-boundary audit: the only original sources opened were the ten exact DEEP-J-013 manifest rows. The only orientation artifact consulted was `10-J-SUMMARY_INDEX.discovery.md`.
- Output metrics after finalization: `output_lines=0508`; `output_bytes=087475`.
- Canonical normalized output SHA-256: 9e1ef34f3029aa7c76e31eb0eb813afab907a6caceedd15cd45e22a9700f9c30.
- Hash rule: compute SHA-256 over the exact final UTF-8 bytes including the final newline after replacing only the 64 hexadecimal characters on the `Canonical normalized output SHA-256` line with 64 ASCII zeroes. Fixed-width output metrics and digest substitution avoid impossible raw self-reference.