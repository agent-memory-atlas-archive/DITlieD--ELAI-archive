# DEEP-J-008 exhaustive feature discovery

## Scope, integrity, and method

- Processed manifest: `docs/thefinalglance/feature-discovery/DEEP-SOURCE-MANIFEST.md:L986-L1001`, heading `### DEEP-J-008`; the next chunk starts at L1003.
- Declared chunk: family J, **10 files**, **1,219 logical lines**, **151,045 bytes**, all ten classified `numbered/index original`; target status **under 1,800**.
- Mechanical verification: fresh `sha256sum`, `wc -l`, and byte-size checks matched every manifest row and both aggregates exactly. Missing paths: **0**; unexpected paths: **0**; duplicate paths: **0**; SHA mismatches: **0**; line mismatches: **0**; byte mismatches: **0**.
- Read boundary: all ten listed originals were read completely, first through last logical line. No original assigned to another deep chunk was opened. `10-J-SUMMARY_INDEX.discovery.md` was consulted only as an orientation and delta-comparison artifact; no source atom below is derived solely from it.
- Anchor notation refers to the manifest-verified source images: `J-2:Lx-Ly`, `J-16:Lx-Ly`, `J-41:Lx-Ly`, `J-54:Lx-Ly`, `J-55:Lx-Ly`, `J-57:Lx-Ly`, `J-58:Lx-Ly`, `J-67:Lx-Ly`, `J-82:Lx-Ly`, and `J-91:Lx-Ly`.
- Repeated premise, revised-scope, wiring, grounding, composition, and acceptance language is collapsed into one atom with every load-bearing anchor. Interaction, ambiguity/conflict, and summary-delta records reference atoms and are not counted again as features.
- Classification: `E` = explicit source fact/requirement; `I` = bounded inference; `A` = unresolved ambiguity/choice; `C` = internal conflict or summary correction that must not be silently flattened. Domains: `STATUS`, `CAP`, `SURFACE`, `FLOW`, `STATE`, `DATA`, `CFG`, `INT`, `EVENT`, `SEC`, `PRIV`, `COMP`, `FAIL`, `RECOVERY`, `RETRY`, `DET`, `IDEM`, `CONC`, `OBS`, `TEST`, `LIMIT`, `DEP`, `DEFER`, `EVIDENCE`.

### Exact processed manifest and verification

| Manifest class | Lines | Bytes | SHA-256 | Exact absolute path | Verification |
|---|---:|---:|---|---|---|
| numbered/index original | 215 | 14,684 | 7d166b95f215267fe093e58a6d117668abcce96b4028b020bc0bb0383991265d | docs/thefinalglance/J-plan/J-2_elai_mcp_control_plane.md | exact |
| numbered/index original | 78 | 11,213 | 4a849f9126a91b0b7f0cc3e9b6ebb18cc50710efcb7eab117e9b596b6c09f846 | docs/thefinalglance/J-plan/J-16_blast_radius_regression_gate.md | exact |
| numbered/index original | 54 | 9,714 | 1824daf87a060c94d5723e68348ea0d5d01a61115672aea38c371091c696e035 | docs/thefinalglance/J-plan/J-41_cheap_grounding_stack.md | exact |
| numbered/index original | 237 | 31,996 | 9857f71108b82ecbcf85bb81d2d5d66b60c7ecd0965d6e946ef82424d68ea9f8 | docs/thefinalglance/J-plan/J-54_free_api_rotation_pool.md | exact |
| numbered/index original | 135 | 18,023 | 90b33af25158f2a1f3b57f909b1f3328b0229a71d1c4f63e3bd7d4d26823cb2c | docs/thefinalglance/J-plan/J-55_model_strength_modality_router.md | exact |
| numbered/index original | 126 | 15,539 | bdb1041b29320632cddac7f989d5ef71c3cf9924cf55cd8c74ed332bafb0f54a | docs/thefinalglance/J-plan/J-57_eval_gated_prompt_promotion.md | exact |
| numbered/index original | 120 | 16,920 | 00421fa3c1cc90b6657ef5f0eaa29599e62af3f10b19c5e136c6d90deef06327 | docs/thefinalglance/J-plan/J-58_sprint_contract_negotiation_loop.md | exact |
| numbered/index original | 70 | 9,167 | fbb21f6a4b0665843097a4a30fbd6807f19084163850094c5eb4c11e5a9fb0ba | docs/thefinalglance/J-plan/J-67_multimode_gate_actions.md | exact |
| numbered/index original | 110 | 15,029 | 41edb223794a6645d503c2966c8fb4977d9184acd105f6fae4ed9ab6e3fc8b21 | docs/thefinalglance/J-plan/J-82_deterministic_token_efficient_emission.md | exact |
| numbered/index original | 74 | 8,760 | c093406bfe92affa457cf9472a0eecb1def7ec8c76b488f4a7dbd25d3009f8d0 | docs/thefinalglance/J-plan/J-91_reexecutable_rra_track.md | exact |
| **Aggregate** | **1,219** | **151,045** | — | **10 exact paths** | **exact** |

## Per-file atomic ledgers

### J-2 — elai-as-MCP-server control plane

Source: `docs/thefinalglance/J-plan/J-2_elai_mcp_control_plane.md` (215/215 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| J2-E001 | E/STATUS | J-2 is PLANNED Wave 4 BUILD, deliberately last big build, stdio local-only transport, blocked until the security-precondition live re-check passes. | J-2:L1-L4 |
| J2-E002 | E/CAP | Objective is a long-lived MCP SERVER so external clients (Claude Code, dashboard, scripts) can submit tasks, poll status, read FSM state, fork/rewind sessions, read plan status and RRAs, and answer interventions on a RUNNING instance. | J-2:L6-L11 |
| J2-E003 | E/LIMIT | Novelty claim: B-25 ElaiNeuron sketched 4 capabilities but is DEFERRED on Empire; C-39 is the inverse (CLI as MCP client); I-40 wraps other agents; nobody authored elai-as-control-plane. | J-2:L11-L14 |
| J2-E004 | E/SEC | Hard boundary: every verb rides an existing primitive and its gates; a verb that bypasses FSM, destructive-verb gates, or the journal is a P0 (GD-1/GD-2). Without the security spine this is an RCE hole. | J-2:L14-L17 |
| J2-E005 | E/CFG | Effort ~2 sessions (12-16 h). Pure in-process + stdio; TCP explicitly out of scope this plan. | J-2:L19-L20 |
| J2-E006 | E/DEP | Blocking live pre-reqs: C-42 authz DENY-ALL; E-24 blake3 schema pin + G-4 rug-pull re-pin (confirm G-4 specifically); E-30 argv destructive deny + G-3 blast-radius gate behind every disk/process verb. | J-2:L21-L27 |
| J2-E007 | E/CONC | Parallel-safe with J-3/J-4/J-5/J-6/J-7; NOT J-1 (shared CLI dispatch seam). | J-2:L28-L28 |
| J2-E008 | E/DATA | Journal IS the state: external client gets the same evidence surface as the interactive operator; no parallel bookkeeping or second source of truth (GD-2). | J-2:L32-L35 |
| J2-E009 | E/CAP | Verb set: task_submit / task_status (task_runner), fsm_get_state (FSM), session_fork / session_rewind (A-1 chain H-35+H-49+I-52 if live), plan_status (I-39), rra_show (G-2 rra.rs SHIPPED), intervention_respond (B-6 intervention SSB pair). | J-2:L35-L40 |
| J2-E010 | E/FAIL | session_fork/rewind degrades to typed NotAvailable when the A-1 chain is unshipped; never a reimplementation. | J-2:L37-L39, L67-L68, L142-L144, L164-L165 |
| J2-E011 | E/SEC | DENY-ALL: a verb without an authz label does not exist to the client. | J-2:L40-L41, L65-L65, L100-L100 |
| J2-E012 | E/DEP | B-25 capability list is reusable; FactoryLine/Empire transport is not wanted; B-25 stays deferred. | J-2:L44-L45 |
| J2-E013 | E/EVENT | Prefer-reuse C-39 SSB rows cli_mcp_dispatch / _server_started / _server_stopped; else ONE +McpControlPlaneEvent (baseline+1 from LIVE 221/532, 6-part discipline). Verbs otherwise emit existing primitive rows. | J-2:L46-L47, L71-L73 |
| J2-E014 | E/INT | I-40 `elai wrap` is the inverse concern (gates around OTHER agents); no overlap. H-34/C-35 external MCP schema consumption is untouched. | J-2:L48-L49 |
| J2-E015 | E/FLOW | PRE_GATE: grep+run live status of C-42/E-24/G-4/E-30/G-3; record verdicts in J-2-decisions.md; ANY miss stalls plan at PLANNED with named blocker. | J-2:L61-L63, L127-L128 |
| J2-E016 | E/CAP | Server skeleton: crates/cli/src/mcp_server.rs (sibling of mcp_client.rs) — stdio transport, tool registry carrying ONLY authz-labelled verbs, DENY-ALL otherwise, NO TCP. | J-2:L64-L65, L87-L88 |
| J2-E017 | E/FLOW | Each verb is a thin adapter: task_submit->task_runner; fsm_get_state->FSM read; session_fork/rewind->A-1 if live else NotAvailable; plan_status->I-39; rra_show->rra.rs; intervention_respond->B-6 SSB pair write. Same FSM/destructive gates as interactive. | J-2:L66-L70 |
| J2-E018 | E/CONC | Mid-FSM control-plane call respects the state machine: task_submit during EXECUTING queues or refuses per FSM rules; never a second concurrent driver of the same session; session-scoped locks reused. | J-2:L74-L77 |
| J2-E019 | E/TEST | Client smoke: external MCP fixture submit->poll->read rra; adversarial arm: unlabelled verb, destructive verb without gate verdict, malformed frame. | J-2:L78-L80, L140-L144 |
| J2-E020 | E/SURFACE | frontend_pair: control-plane connection + verb rows in an existing sessions/monitor drawer, or W-6 justification. | J-2:L81-L82, L105-L106 |
| J2-E021 | E/DATA | Decision doc workflow/reasoning/J-2-decisions.md: verb list final, authz labels, transport posture, SSB reuse-vs-new, security re-check evidence. | J-2:L83-L84, L150-L150 |
| J2-E022 | E/SURFACE | CLI entrypoint: `elai mcp-serve` new arm in existing CLI dispatch [PRODUCTION]; external stdio MCP client is JD-1-style live consumer. | J-2:L96-L98 |
| J2-E023 | E/LIMIT | Stub bans: no unlabelled verb; no privileged bypass ("server is trusted" banned); no parallel state store / cache that answers status without the journal; no TCP; no .unwrap/.expect on prod paths. | J-2:L99-L104 |
| J2-E024 | E/OBS | Observable: same SSB rows as interactive fixture under ELAI_SEED (GD-2) + server lifecycle rows; client receives status/rra payloads byte-derivable from journal. | J-2:L107-L114 |
| J2-E025 | E/FAIL | Negative: unlabelled verb returns DENY-ALL; destructive without gate BLOCKED; --skip-authz broken arm exposes verb and FAILS contract (R2). | J-2:L112-L115, L140-L144 |
| J2-E026 | E/TEST | Contract j2_control_plane_contract.rs: (a) unlabelled DENY-ALL; (b) task_submit journal identical to interactive under seed; (c) destructive without gate BLOCK; (d) rewind NotAvailable when A-1 absent; (e) malformed frame typed error, server lives; (f) --skip-authz FAILS. | J-2:L140-L144 |
| J2-E027 | E/DATA | Files: CREATE mcp_server.rs ~450 LOC; MODIFY cli args/dispatch mcp-serve arm; conditional ssb.rs/action_type_lock; CREATE J-2-decisions.md. | J-2:L146-L150 |
| J2-E028 | E/LIMIT | Acceptance: no new dependency beyond what mcp_client.rs already uses without approval; cargo check workspace GREEN; auditor PASS on elai-cli (+orchestrator if touched); stdio-only grep-proven. | J-2:L134-L137 |
| J2-E029 | E/DEFER | Multi-client (two external clients on one elai) out of scope v1: single client documented; concurrent drivers refused. Auth beyond same-user-stdio deferred; any TCP/remote is a NEW plan. | J-2:L160-L163 |
| J2-E030 | E/DEFER | J-15 amendment (a): if workflow verbs grow beyond v1, steal conductor Script-step (exit-code-only routing under Tier-C sandbox, no LLM) and Set-step (LLM-free derived values from prior SSB rows) as BSP node SHAPES, not FSM states; not v1 scope. | J-2:L172-L192 |
| J2-E031 | E/CFG | J-15 amendment (b): MCP tool-config DELEGATED to host runtime; elai server MUST NOT embed tool definitions/resource URIs/transport settings in workflow or plan files; advertises supported verbs only via tools/list handshake. | J-2:L194-L203 |
| J2-E032 | E/INT | Transparent-proxy shape note: if recording/replay leg is added, client->gateway->server with zero changes on either side; intercept, journal to SSB, forward unmodified; same shape as I-40 elai wrap; do NOT translate MCP messages. | J-2:L205-L213 |
| J2-A001 | A/EVENT | C-39 exact shipped surface mid-edit 2026-06-10: reuse-vs-new SSB verdict and transport plumbing decided at edit after fresh read (W-7). | J-2:L158-L159 |
| J2-A002 | A/FLOW | Whether task_submit during EXECUTING queues or refuses is left to existing FSM rules without a fixed choice in this plan. | J-2:L74-L76 |
| J2-I001 | I/SURFACE | Dashboard is named as a future automation consumer of the control plane, so drawer surfacing is the natural product path for operator visibility of external drives. | J-2:L168-L170, L81-L82 |

Category coverage: capabilities/actions/surfaces ✓; workflows/states ✓; schemas/data/lifecycle ✓; defaults/config ✓; integrations/events ✓; security/permissions ✓ (auth beyond stdio deferred); failures/recovery ✓; determinism/concurrency ✓; observability/evidence ✓; tests/done gates ✓; constraints/non-goals ✓; dependencies/status/deferred ✓. No formal idempotency key; privacy is same-user-stdio only.

### J-16 — blast-radius / anti-regression gate

Source: `docs/thefinalglance/J-plan/J-16_blast_radius_regression_gate.md` (78/78 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| J16-E001 | E/STATUS | J-16 is Track J pillar-SOTA recheck, lever CANT-WRECK, STRONG grounding, BUILD, effort L, WAVE 1, depends none (rides shipped B-29), KEYSTONE highest-priority unit of the pillar recheck. | J-16:L1-L4 |
| J16-E002 | E/FLOW | Motivating defect: narrow-focus LLM edits; B-29 impact API is PRE-EDIT ADVISORY the model can ignore, not a gate. Novel whitespace: no published regression-test-selection/over-editing benchmark for agent edits. | J-16:L6-L6 |
| J16-E003 | E/CAP | On every accepted edit batch the gate (a) computes reverse-dep cone of each changed symbol, (b) enforces diff-scope, (c) selects regression tests covering the cone, (d) flags silent deletion. LLM never adjudicates (TD-3). | J-16:L6-L6, L16-L20 |
| J16-E004 | E/EVIDENCE | Closes top50 bad-behaviors #14 regression, #15 over-editing, #16 silent deletion, #29 unrequested improvements, #31 symptom patches (cone advisory for root, not hard block). | J-16:L8-L13 |
| J16-E005 | E/CAP | blast_radius gate module: post-EXECUTING / pre-AUDITING; depth-bounded reverse-dep cone via shipped B-29 impact API; no reimplementation of traversal (GD-1). | J-16:L16-L16, L31-L31 |
| J16-E006 | E/CFG | DIFF-SCOPE leg: compare changed-file set to plan unit WIRING_CONTRACT produces_symbols / babystep scope or ad-hoc user-stated target; out-of-scope+forced-dependents = scope violation with exact hunks. Default OBSERVE-ONLY (GD-4) until FP rate bounded over >=3 real runs; then BLOCK behind wire-dark-allowlist row. | J-16:L17-L17, L52-L52 |
| J16-E007 | E/TEST | REGRESSION-SELECTION: from cone select existing tests whose coverage intersects dependents; NO new test authoring; auditor runs them; pass->fail transition = regression BLOCK (R4 un-gameable). | J-16:L18-L18 |
| J16-E008 | E/FAIL | SILENT-DELETION: net-negative LOC on symbols not named in edit intent; flags removed pub items / dangling call sites; NEVER auto-reverts (RV-4); surfaces for review. | J-16:L19-L19, L50-L50 |
| J16-E009 | E/SEC | Gate is AUDITOR-owned and FROZEN to the executing run (R8): executor cannot edit the gate; regression set + scope contract are read-only inputs. | J-16:L20-L20 |
| J16-E010 | E/LIMIT | EXCLUDED: LLM judgment of edit size; auto-revert of any kind; NEW impact-traversal engine (re-implement = GD-1 P0). | J-16:L21-L21 |
| J16-E011 | E/DEP | Substrate: B-29 impact_radius/call_flow/symbol_context; DKG; codegraph MCP; impact_radius.py; AntiFabricationGate.check at task_runner (F-4 sibling); orchestrator-captured git2 diff; impact-mapper agent becomes enforced. | J-16:L23-L28 |
| J16-E012 | E/EVENT | +BlastRadiusEvent Pattern-2 single-variant; Kind: ConeComputed / ScopeViolationFlagged / RegressionSetSelected / SilentDeletionFlagged. baseline+1 from LIVE 231/542 (ssb.rs:3084 / action_type_lock.rs:671, 2026-06-12); re-read at edit. | J-16:L32-L32, L65-L65 |
| J16-E013 | E/SURFACE | frontend_pair: blast-radius panel/drawer on existing audit-timeline surface (AuditTimelineWindow / B-41) showing cone + selected regression set + scope/deletion flags; cross_lang_diff or W-6 absence for first runtime-only slice. | J-16:L33-L33, L54-L54 |
| J16-E014 | E/CAP | produces_symbols: compute_cone, check_diff_scope, select_regression_set, flag_silent_deletion in crates/orchestrator/src/blast_radius.rs. | J-16:L36-L40 |
| J16-E015 | E/INT | Production reachability: AUDITING FSM transition on real elai task/fix run (post-EXECUTING, auditor-driven); test/contracts callers never satisfy R7. | J-16:L46-L47 |
| J16-E016 | E/LIMIT | Stub bans: no LLM in gate; no auto-revert; no new traversal; no BLOCK default until FP bound + allowlist; no regression set of only test-file coverage with no production-reachable dependent (R7). | J-16:L48-L53 |
| J16-E017 | E/OBS | Trigger: fix run editing a symbol with known dependents through real AUDITING. Expect ConeComputed listing dependents; RegressionSetSelected; ScopeViolationFlagged if out-of-scope. | J-16:L55-L57 |
| J16-E018 | E/FAIL | Negative: empty cone on symbol with dependents (DKG miss) FAILS self-test never SKIPs; undeclared pub deletion raises SilentDeletionFlagged; unmappable/parse-failure diff FAILS never passes. | J-16:L58-L58 |
| J16-E019 | E/FAIL | DKG incompleteness (protobuf/sqlx/proc-macro symbols): gate must DECLARE coverage (langs/edge-kinds) and FAIL-LOUD on unmappable diff; J-18 LSP channel narrows gap. | J-16:L70-L71 |
| J16-E020 | E/CFG | Ad-hoc fix scope falls back to user-stated target file(s); out-of-scope = flag-not-block until friction known; document fallback explicitly. | J-16:L72-L72 |
| J16-E021 | E/LIMIT | High-fan-in symbols: bound cone depth + cap selected suite; LOG the cap (no silent truncation, R-no-silent-caps). | J-16:L73-L73 |
| J16-E022 | E/DEP | Composes B-29, F-4, C-28/C-29, G-5/H-52, I-20; restates none. Cone is substrate for H-52/G-5 per-hunk review. | J-16:L67-L68, L75-L76 |
| J16-A001 | A/CFG | Exact cone depth bound and selected-suite cap values are not fixed in the plan. | J-16:L73-L73 |
| J16-A002 | A/SURFACE | Exact audit-timeline panel placement and first-slice W-6 vs full UI is decide-at-edit. | J-16:L33-L33, L54-L54 |
| J16-I001 | I/STATE | Observe-only default implies a later promotion decision artifact (allowlist row + measured FP rate) before BLOCK is load-bearing. | J-16:L17-L17, L52-L52 |

Category coverage: capabilities/actions/surfaces ✓; workflows/states ✓; events ✓; security/freeze ✓; failures ✓; determinism ✓; tests/done gates ✓; constraints ✓; dependencies/deferred ✓. No privacy/compliance surface; no retry API; no CLI verb (auditor-owned gate).

### J-41 — cheap grounding stack

Source: `docs/thefinalglance/J-plan/J-41_cheap_grounding_stack.md` (54/54 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| J41-E001 | E/STATUS | J-41 Track J, lever grounding-determinism + local-economy, nature build (activate F-12 OnnxNliJudge + extend H-53/E-35), effort M, WAVE DD-3, depends F-12, H-53, E-35, J-40. | J-41:L1-L3 |
| J41-E002 | E/CAP | Three-layer stack: cheap deterministic EXTRACTIVE first pass (span fingerprinting) -> SMALL claim-level entailment only on flags -> CLOSED-SET citation binding so ghost citations are structurally impossible. | J-41:L4-L4, L21-L22 |
| J41-E003 | E/TEST | EXP-2 OBSERVE-ONLY step 1: 100 (source-snippet, claim) pairs from ELAI codebase (50/50), run MiniCheck-FT5 770M or Paladin-mini 3.8B, measure balanced accuracy; <75% = no transfer without fine-tune; decides model slot usability. Not default-ON until passed. | J-41:L7-L7, L48-L48 |
| J41-E004 | E/CAP | Three-tier extractive fingerprint first pass: substring (exact Rabin-Karp/KMP full confidence) -> fuzzy (>=0.8 token-overlap/edit-distance) -> semantic (>=0.85 cosine). Sentence/n-gram>=5 meaningful; plagiarism floor 8-10 consecutive words. Flags zero-support claims for tier 2. | J-41:L8-L8 |
| J41-E005 | E/CAP | Claim-level entailment via F-12 OnnxNliJudge: drop small model ONNX into `<root>/.elai_cc/models/onnx/`. ADAPTER: OnnxNliJudge expects 3-class NLI but MiniCheck/Paladin are BINARY; add BinaryGroundingJudge sibling or map binary->NLI. Claim decomposition via small T5/BART. | J-41:L9-L9, L49-L49 |
| J41-E006 | E/CAP | Closed-set citation: DecodingMode::ClosedSet { allowed_ids } feeds H-53 EvidenceAnchorSet id list as grammar/token allowlist at decode (invalid -> logit -inf). Out-of-set -> D-16 conformal refusal. | J-41:L10-L10 |
| J41-E007 | E/LIMIT | EXCLUDED: token-level hash membership; generative claim rewriting; NLI as SOLE gate on synthesized output; RAG-HAT forced citation without abstention (D-16 mandatory). | J-41:L11-L11 |
| J41-E008 | E/DEP | NliJudge trait + DeterministicNliJudge WIRED; OnnxNliJudge compiles, model file ABSENT (fail-safe SKIP when missing). EvidenceAnchorSet + freeze_anchors + check_citation_in_set WIRED; CitationAnchorsBound (229) exists. ConstrainedDecodingConfig Grammar mode WIRED. | J-41:L13-L18 |
| J41-E009 | E/EVENT | Reuse CitationAnchorsBound + GROUNDING VERIFICATION verdicts; net 0 new action types. Optional extractive telemetry prefer-reuse; Pattern-2 only if genuinely new from LIVE 268/577. | J-41:L22-L22, L30-L30 |
| J41-E010 | E/CAP | produces: ExtractiveGroundingPass, BinaryGroundingJudge (or adapter), DecodingMode::ClosedSet, bind_closed_set_decode. | J-41:L25-L25 |
| J41-E011 | E/INT | reachable_from: research_pipeline.rs (J-40 hands spans); prebind_verify_citations; decode role-config call site. | J-41:L27-L27 |
| J41-E012 | E/LIMIT | stub_ban: no token-level membership; no SKIP-as-pass on missing model (explicit SKIP verdict surfaced, not silent grounded); no forced citation without abstention branch. | J-41:L28-L28 |
| J41-E013 | E/OBS | Out-of-set citation REFUSED at decode or post-hoc rejected -> D-16 refusal / CitationAnchorsBound; unsupported claim flagged by tier 1 without tier-2 entailment logged as ungrounded. | J-41:L29-L29 |
| J41-E014 | E/DEP | Composes F-12, H-53, E-35, D-16, J-40 (ranked compressed spans as evidence set), ELAI-DETERMINISM-DOCTRINE. | J-41:L39-L45 |
| J41-E015 | E/CFG | Promotion to default-ON follows GD >=3-run FP-bound rule after EXP-1+EXP-2 and gate-verified observable_effect. | J-41:L53-L54 |
| J41-E016 | E/FAIL | Missing ONNX model must not silently pass as grounded; ClosedSet requires provider/sidecar token allowlist (E-35 substrate PARTIAL — confirm at edit). | J-41:L15-L15, L51-L51 |
| J41-A001 | A/CAP | Whether BinaryGroundingJudge sibling vs label mapping is chosen at implementation for 3-class vs binary mismatch. | J-41:L9-L9, L49-L49 |
| J41-A002 | A/DEP | ClosedSet decode support on local sidecar (E-35 PARTIAL) is unconfirmed until edit. | J-41:L51-L51 |
| J41-A003 | A/TEST | Exact EXP-2 model selection between MiniCheck-FT5 and Paladin-mini is alternative, not fixed. | J-41:L7-L7 |
| J41-I001 | I/FAIL | J-40 span-budget compression can drop a true claim's support and cause false ungrounded; source names recall tuning + tier-2 paraphrase catch as mitigation. | J-41:L50-L50 |

Category coverage: capabilities ✓; grounding/security of claims ✓; config/flags via EXP/default-ON ✓; events (reuse) ✓; failures/abstention ✓; tests/EXP ✓; constraints ✓; dependencies ✓. No dedicated UI surface in source; no CLI verb.

### J-54 — free-API rotation pool

Source: `docs/thefinalglance/J-plan/J-54_free_api_rotation_pool.md` (237/237 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| J54-E001 | E/STATUS | J-54 PLANNED WAVE 1 BUILD-on-shipped: FreePool RuntimeBackend kind; ports Liberty rotation natively onto B-37 trait; no litellm. | J-54:L1-L3 |
| J54-E002 | E/CAP | FreePool is user-assignable PER ROLE exactly like local/cloud/hybrid (B-37 PerRoleRuntime); supplies free-tier economics: per-(key,model) daily quota, 429 header-priority cooldown, whole-pool exhaustion backoff, keyless overflow, multi-key rotation. | J-54:L4-L5, L23-L24 |
| J54-E003 | E/LIMIT | Dynamic catalog fetch (catalog_sync.py) ALREADY-REJECTED; free catalog is STATIC ROWS in models.toml; never auto-fetched (S7). | J-54:L5-L5, L123-L123, L152-L152 |
| J54-E004 | E/DEP | Compose not reimplement: B-37 RuntimeAdapter/PerRoleRuntime/resolve_backend; I-7 EscalationCascade; C-5 CloudModeRouter caps; G-2 rra seal + G-10 drift canary; I-51 models.toml. | J-54:L5-L5, L26-L33 |
| J54-E005 | E/CFG | Effort ~2 sessions (~10 h). In-process quota/routing/heal; network only on free-provider completion (reqwest). No GPU. | J-54:L7-L7 |
| J54-E006 | E/DEP | FreePool is SEPARATE from E-39 CerebrasFree/GroqFree RuntimeBackend arms (those return NotImplemented); free_pool/ directory does NOT exist yet. | J-54:L13-L13, L16-L16 |
| J54-E007 | E/SEC | Raw API key NEVER persisted; only blake3 fingerprint KeyId{provider, fp=blake3(raw_key).to_hex()[:8]} (CB-1 overrides Liberty sha256); from_raw_key consumes the key. | J-54:L24-L24, L52-L52, L121-L122 |
| J54-E008 | E/DATA | Day counters persist across restart to ~/.elai/free_pool/quota_state.json (atomic tmp+rename); per-minute counters + cooldowns do NOT persist (I-3). UTC-day roll clears day+minute+cooldowns; UTC-minute roll clears minute only. | J-54:L24-L24, L52-L52, L127-L127 |
| J54-E009 | E/SEC | Uncensored group has NO cross-group fallback and NO overflow suffix (I-6); silently falling back to safety-tuned model defeats the lane. | J-54:L24-L24, L59-L59, L124-L124 |
| J54-E010 | E/RETRY | soonest_reset is MINIMUM across reset hints, not max (I-9). note_429 header priority: x-ratelimit-reset-requests > Retry-After > x-ratelimit-reset-tokens > 60s default. | J-54:L24-L24, L52-L52 |
| J54-E011 | E/FLOW | Dispatch: quota-filter deployments, simple-shuffle within live group, on group exhaustion walk fallback chain, on whole-pool exhaustion spill to I-7 cloud (if role allows) or sleep to soonest_reset and retry. | J-54:L24-L24, L58-L59 |
| J54-E012 | E/DATA | FreeModel catalog fields: id/provider/group/context_window/max_output/rpm/tpm/rpd/tpd under models.toml [free_pool] + DENYLISTED_MODEL_IDS typed loader; not a hardcoded Rust list. | J-54:L44-L45 |
| J54-E013 | E/CAP | QuotaTracker: Mutex struct with day_reqs/day_tokens + minute counters + cooldowns; has_headroom (rpd/tpd + cooldown); soonest_reset. | J-54:L51-L52 |
| J54-E014 | E/CAP | FreePool dispatcher builds Vec<Deployment> per group from catalog x keys from env GROQ_API_KEY/_1/_2..; EXACT fallback chain big>fast-small>gemini-flash>mistral>openrouter-free, uncensored EMPTY; overflow-pollinations on NON-uncensored; empty-after-filter => PoolExhausted. | J-54:L58-L59 |
| J54-E015 | E/CAP | Keyless overflow: Pollinations always-present deployment (openai-compat, api_key literal "none", https://text.pollinations.ai/openai). Drop g4f (CB-2 Python-only). Guarantee >=1 overflow always reachable (I-7). | J-54:L65-L66 |
| J54-E016 | E/RECOVERY | Heal wrapper: on PoolExhausted, if role permits cloud -> I-7 EscalationCascade; else sleep to soonest_reset (bounded max_total_wait/max_retries), retry; ALL other errors propagate immediately. | J-54:L71-L72 |
| J54-E017 | E/INT | FreePoolAdapter implements RuntimeAdapter; registered in B-37 factory; resolve_backend returns it when role kind=FreePool; uses existing PerRoleRuntime surface (no parallel config). | J-54:L77-L78 |
| J54-E018 | E/SEC | Trust-marker: when FreePool for AgentRole::Executor or ::Auditor, stamp RRA non_reproducible=true + emit FreePoolTrustDowngrade + suppress-with-notice G-10 drift canary. Other roles: no downgrade. | J-54:L84-L85 |
| J54-E019 | E/EVENT | FreePoolEvent Pattern-2 3-arm FreePoolEventKind: ModelRotated{group,model_fp,key_fp}, PoolExhaustedBackoff{soonest_reset_ms}, FreePoolTrustDowngrade{role}; 9-tuple golden-lock; payload NO key material. baseline+1 from LIVE 268/577. | J-54:L90-L91, L226-L226 |
| J54-E020 | E/SURFACE | RuntimePanel gains free-pool assignment control + per-run "non-reproducible (free-pool)" badge when verification role on pool; Tauri read of role->backend assignment + FreePoolEvent stream. | J-54:L129-L130, L182-L182 |
| J54-E021 | E/SURFACE | CLI/config: `elai run --role-backend draft=free-pool "<task>"` (or config-file equivalent) with GROQ_API_KEY. | J-54:L131-L132, L217-L218 |
| J54-E022 | E/OBS | Expect ModelRotated row with fingerprints; forced-exhaustion -> PoolExhaustedBackoff; quota_state.json day-counter row. Negative: executor on pool without non_reproducible + FreePoolTrustDowngrade is FAIL. | J-54:L132-L134 |
| J54-E023 | E/CFG | W-2: ships ON when a role is assigned; kill-switch IS the role assignment (no default-OFF ELAI_*_ENABLE). | J-54:L149-L149, L161-L161 |
| J54-E024 | E/TEST | j54_free_pool_contract.rs: (a) raw key absent from fp+json; (b) DENYLISTED absent; (c) uncensored empty fallback+no overflow; (d) note_429 soonest header; (e) Pollinations >=1; (f) day persists, minute/cooldown do not; (g) executor non_reproducible + FreePoolTrustDowngrade. | J-54:L173-L174 |
| J54-E025 | E/DATA | Files: CREATE free_pool/{mod,quota,dispatch,heal}.rs ~620 LOC; MODIFY runtime factory/enum/resolve; models.toml; ssb FreePoolEvent; rra non_reproducible; RuntimePanel; contracts; J-54-decisions.md. | J-54:L176-L184 |
| J54-E026 | E/FAIL | ABORT if: B-37 trait/resolve missing; E-39 already implements rotation pool; blake3 not available and user refuses add; I-51 models.toml rejected; RuntimeBackend #[non_exhaustive] unextendable. | J-54:L189-L195 |
| J54-E027 | E/LIMIT | No sha256/sha2/blake2; no dynamic catalog/OpenRouter live registry/cheahjs; no silent model swap on executor/auditor without stamps; no .unwrap on prod dispatch/quota/heal; no new crate without confirmation. | J-54:L120-L128 |
| J54-E028 | E/OBS | Integration smoke: multi-turn forced rotation; executor trust-marker; forced-exhaustion heal/spill. | J-54:L216-L220 |
| J54-A001 | A/CFG | Whether to surface a per-pool request-budget in config (lean yes, reuse rpd accounting) decided at authoring. | J-54:L228-L229 |
| J54-A002 | A/CFG | Uncensored rows lean: include group structure + empty-fallback invariant, leave uncensored rows commented so user opts in by uncommenting. | J-54:L230-L231 |
| J54-A003 | A/CFG | Env multi-key GROQ_API_KEY_1/_2 port vs J-30 custom_providers.toml generalization: lean port env for known free providers + J-30 for rest. | J-54:L232-L232 |
| J54-I001 | I/SEC | 8-hex blake3 collision risk (32 bits) is acknowledged as fingerprint not key id; collision over-counts one key, never leaks. | J-54:L171-L171, L233-L233 |

Category coverage: capabilities/surfaces ✓ (RuntimePanel, CLI role-backend, badge); workflows/quota lifecycle ✓; config/flags ✓; events ✓; security/privacy of keys ✓; failures/heal/retry ✓; determinism of fingerprints ✓; tests/done gates ✓; constraints ✓; deps/abort ✓.

### J-55 — model-strength modality router

Source: `docs/thefinalglance/J-plan/J-55_model_strength_modality_router.md` (135/135 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| J55-E001 | E/STATUS | J-55 PLANNED WAVE 1 EXTENSION KEYSTONE of win-both design; composes J-51/E-31/E-35/I-36/H-4/J-34; reimplements none. | J-55:L1-L3 |
| J55-E002 | E/CAP | ModalityRouter resolves ACTION MODALITY per (model, role): CodeAct for capable lanes; constrained single-tool-call under XGrammar + prompt-compiled minimal schemas for weak local; universal SEARCH/REPLACE-diff edit path regardless of modality; progressive tool disclosure on schema'd path. | J-55:L4-L5, L16-L17 |
| J55-E003 | E/LIMIT | Does NOT author CodeAct, SEARCH/REPLACE, adherence bench, prompt compilation, or tier router; deterministic dispatcher + J-34-style bench only. | J-55:L5-L5 |
| J55-E004 | E/EVIDENCE | Keystone: naive CodeAct everywhere loses on Qwen2.5-Coder-7B (~12% multi-tool code-mode); viability ~14B; router is load-bearing unit. Papers arXiv:2402.01030, 2509.04508, 2605.02363 verified 2026-06-20 DESIGN-scoped. | J-55:L5-L5, L13-L16, L121-L122 |
| J55-E005 | E/DEP | Pre-reqs: E-35 CodeAct+SEARCH/REPLACE; J-51 tool_call_adherence (sequence AFTER if unshipped); E-31 tier matrix; I-36 CompiledPromptStore; H-4/H-34/J-24 progressive disclosure; J-34 bench pattern; B-10 ModelCapabilityProfile. | J-55:L8-L8, L19-L26 |
| J55-E006 | E/DET | Modality decision is DETERMINISTIC (TD-3): reads ModelCapabilityProfile + tier; LLM never picks its own modality. | J-55:L17-L17, L62-L62 |
| J55-E007 | E/CAP | ActionModality enum { CodeAct, ConstrainedToolCall, TextFallback }; ModalityRouter::resolve(profile, role, tier). Rules: tier=cloud OR (local AND params>=14B AND tool_call_adherence>=threshold) -> CodeAct; local weak -> ConstrainedToolCall; no-tool -> TextFallback. Thresholds NAMED CONSTANTS seeded by bench. | J-55:L36-L36 |
| J55-E008 | E/FLOW | Executor dispatch reads resolve and branches emission/parse: CodeAct->E-35 code surface; ConstrainedToolCall->single tool under XGrammar + I-36 compiled minimal schema NOT full registry; progressive disclosure on schema'd path. | J-55:L37-L37 |
| J55-E009 | E/FLOW | EDIT always routes through E-35 SEARCH/REPLACE matcher regardless of ActionModality; assert whole-file-write not taken when diff applies. | J-55:L38-L38 |
| J55-E010 | E/TEST | `elai bench modality` (J-34 pattern): measure per-(model,lane) {CodeAct, ConstrainedToolCall} on tool-success + tokens + latency; emit crossover into ModelCapabilityProfile.modality_crossover. Gate verdicts deterministic; NEVER LLM-judge. Local legs sidecar-gated. | J-55:L39-L39 |
| J55-E011 | E/EVENT | ModalityRouted Pattern-2 3-arm: CodeActChosen{model_fp,role}, ConstrainedCallChosen{model_fp,role,schema_tokens}, EditFormatChosen{diff:bool}; golden-lock; baseline+1 from 268/577. | J-55:L40-L40, L125-L125 |
| J55-E012 | E/SURFACE | ContextStatePanel or RuntimePanel shows chosen modality per role + per-turn schema-token count; Tauri read of ModalityRouted stream. | J-55:L69-L70, L117-L117 |
| J55-E013 | E/SURFACE | CLI: `elai run --backend local` vs `--backend cloud` for dual-lane smoke; `elai bench modality` production bench entrypoint. | J-55:L71-L73, L59-L59 |
| J55-E014 | E/OBS | Local-7B expects ConstrainedCallChosen with schema_tokens; cloud expects CodeActChosen. Negative: Qwen-7B CodeActChosen is FAIL (keystone guard). | J-55:L72-L74 |
| J55-E015 | E/CFG | W-2: ships ON behind ELAI_MODALITY_ROUTER_DISABLE (kill-switch, default-ON). | J-55:L87-L87, L97-L97 |
| J55-E016 | E/TEST | j55_modality_router_contract: (a) deterministic resolve; (b) cloud->CodeAct; (c) local sub-14B/low-adherence->ConstrainedToolCall; (d) no-tool->TextFallback; (e) constrained uses I-36 minimal not full registry; (f) edit via SEARCH/REPLACE; (g) Qwen-7B never CodeActChosen. | J-55:L109-L110 |
| J55-E017 | E/DATA | CREATE modality_router.rs ~290 LOC; MODIFY executor dispatch, ModelCapabilityProfile+=modality_crossover, elai bench modality, ssb, dashboard panels, contracts, J-55-decisions.md. | J-55:L112-L119 |
| J55-E018 | E/LIMIT | Stub bans: no LLM modality choice; no hardcoded always-CodeAct/JSON; no magic-number thresholds; no full tool registry on constrained lane; no whole-file when diff applies; no reimplementation of composed substrates; no .unwrap on resolve/dispatch. | J-55:L61-L68 |
| J55-E019 | E/INT | Production: executor turn dispatch calls resolve before each agentic turn; bench CLI seeds crossover. Test callers never satisfy R7. | J-55:L57-L60 |
| J55-A001 | A/DEP | If J-51 unshipped, WAVE-1 can resolve on params+tier alone and fold adherence later (additive). | J-55:L128-L128 |
| J55-A002 | A/CFG | Until local sidecar runs bench, thresholds are paper-derived defaults LABELLED directional; bench replaces them. | J-55:L129-L129 |
| J55-I001 | I/DET | 14B threshold may stale as newer 7B coders improve; source requires measured modality_crossover as source of truth via re-bench. | J-55:L130-L130 |

Category coverage: capabilities/surfaces ✓ (badge, CLI, bench); workflows ✓; config/flags ✓; events ✓; tests ✓; constraints ✓; deps ✓. No privacy surface.

### J-57 — eval-gated prompt-template promotion

Source: `docs/thefinalglance/J-plan/J-57_eval_gated_prompt_promotion.md` (126/126 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| J57-E001 | E/STATUS | J-57 PLANNED WAVE PV-1 effort M EXTENSION; composes D-1 hash-chained templates + I-36 CompiledPromptStore + G-10 drift + J-56 eval gate; depends J-56. | J-57:L1-L3 |
| J57-E002 | E/CAP | Ships missing workflow: PromptStage state machine (Test/Staging/Prod), promotion gate running J-56 eval subset, promote ONLY on no-regression vs current Prod, automatic rollback to last-good Prod hash on G-10 behavioural regression. | J-57:L4-L4 |
| J57-E003 | E/LIMIT | Does NOT author hash-chain (D-1), compiled store (I-36), drift canary (G-10), or eval corpus (J-56); promotion SM + eval-gate wiring + rollback trigger only. | J-57:L4-L4 |
| J57-E004 | E/DET | Promotion decision is DETERMINISTIC (TD-3): J-56 subset-run verdict; LLM never adjudicates "better" prompt. | J-57:L13-L13, L52-L53 |
| J57-E005 | E/RECOVERY | Prompt-layer DOES auto-demote-to-last-good (reversible/local); G-10 model-layer remains surface-never-auto-swap. | J-57:L13-L13, L31-L31, L57-L57 |
| J57-E006 | E/STATE | PromptStage { Test, Staging, Prod }; each ReasoningTemplate version (blake3 hash in D-1 chain) carries a stage. Load path (select_template / build_system_prompt_for_role) serves Prod-stage only; un-promoted invisible to prod. | J-57:L29-L29 |
| J57-E007 | E/SURFACE | CLI: `elai prompt promote <template-id> --to staging|prod`. Must pass Staging; Test->Prod skip rejected. | J-57:L30-L30, L55-L55 |
| J57-E008 | E/FLOW | Promotion runs `elai eval run --scope subset` over categories the template affects; compare candidate pass/fail vs current Prod; regressing candidate -> Blocked, Prod pointer UNCHANGED. | J-57:L30-L30 |
| J57-E009 | E/RECOVERY | When G-10 drift fires on a turn whose prompt is a recently-promoted Prod version: auto-demote to last-good Prod hash; emit regression provenance; NOT silent. | J-57:L31-L31 |
| J57-E010 | E/DATA | Governed promotion provenance every promote/rollback: {from_hash, to_hash, eval_verdict \| drift_signal, reason} in D-1 chain (lesson-2). | J-57:L32-L32 |
| J57-E011 | E/EVENT | PromptPromotionEvent Pattern-2 4-arm: Staged{template_id,to_hash}, Promoted{from,to,eval_pass}, BlockedByEval{candidate,regressed_categories}, RolledBack{from,to_lastgood,drift_signal}. baseline+1. | J-57:L33-L33 |
| J57-E012 | E/INT | Production paths: elai prompt promote CLI; prod load serving Prod hash; G-10 canary-fire handler calling rollback. Test never satisfies R7. | J-57:L47-L51 |
| J57-E013 | E/SURFACE | frontend_pair: rides EXISTING D-26 ReasoningTemplate dashboard; add stage badge + promotion/rollback provenance rows; no new Tauri command. If missing stage column, ~20 LOC + cross_lang_diff. | J-57:L60-L61, L108-L108 |
| J57-E014 | E/OBS | Clean path: Staged then Promoted{eval_pass=true}; regressing: BlockedByEval with Prod unchanged. Negative FAIL: regressing reaches Prod; Test->Prod skip; demotion without last-good hash. | J-57:L62-L65 |
| J57-E015 | E/CFG | W-2: promotion is CLI + gate; no new ELAI_*_ENABLE; I-36 ELAI_PROMPT_COMPILE_DISABLE unchanged. | J-57:L83-L83 |
| J57-E016 | E/TEST | j57_prompt_promotion_contract: clean promote path; BlockedByEval; Test->Prod reject; drift RolledBack with provenance; blake3 hashes; no LLM-judge. | J-57:L99-L100 |
| J57-E017 | E/DATA | CREATE prompt_promotion.rs ~240 LOC (owning crate re-grep); MODIFY cli main, D-1/I-36 load path, G-10 handler, ssb, D-26 tile; contracts; J-57-decisions.md. | J-57:L102-L110 |
| J57-E018 | E/LIMIT | Stub bans: no LLM better-prompt; no new versioning scheme; no Test->Prod skip; no silent rollback; no auto-MODEL-swap; no sha2 on template hashes; no .unwrap on promote/load/rollback. | J-57:L52-L59 |
| J57-E019 | E/CFG | Eval-gate defaults BLOCK-on-regression (fail-closed); rollback reuses G-10 tuned signal (no new FP surface). | J-57:L97-L97 |
| J57-A001 | A/INT | Exact crate ownership of select_template / build_system_prompt_for_role and how stage is threaded is PLAN-asserted; re-grep at EXECUTING. | J-57:L119-L119 |
| J57-A002 | A/RECOVERY | Auto-demote vs surface-only only revisited if a prompt rollback is non-reversible in a given store (should not be). | J-57:L120-L120 |
| J57-I001 | I/LIMIT | Gate strength equals J-56 corpus coverage for the category; thin categories gate weakly until corpus grows (GD-5). | J-57:L97-L97, L121-L121 |

Category coverage: capabilities/surfaces ✓ (CLI promote, stage badge, provenance); state machine ✓; events ✓; security of promotion ✓; recovery/rollback ✓; tests ✓; constraints ✓; deps ✓.

### J-58 — pre-execution sprint-contract negotiation loop

Source: `docs/thefinalglance/J-plan/J-58_sprint_contract_negotiation_loop.md` (120/120 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| J58-E001 | E/STATUS | J-58 PLANNED WAVE PV-1 effort S-M; EXTENSION of I-21; adds bounded multi-round pre-execution re-propose loop; reuses SprintContractEvent +1 Kind arm; composes I-21/D-13. | J-58:L1-L3 |
| J58-E002 | E/CAP | On Reject, structured objection feeds generator which re-proposes refined contract (typically more/more-specific criteria); isolated evaluator re-judges; bounded N rounds; Approved -> execute; non-convergence at cap -> D-13 adversary or surface. | J-58:L4-L4 |
| J58-E003 | E/SEC | R8 freeze: evaluator stays ISOLATED — zero generator scratchpad; negotiation ONLY via sealed contract payload + structured objection field (I-1 sealed-payload path); NEVER shared context window. | J-58:L4-L4, L50-L50 |
| J58-E004 | E/DEP | I-21 already ships SprintContract struct, isolated evaluator, single-shot Approve/Reject, post-exec Ralph loop; J-58 fills the missing pre-exec re-propose-until-agree. | J-58:L4-L4, L15-L16 |
| J58-E005 | E/DET | Every contract version still must pass I-21 rule: acceptance_criteria deterministically checkable (symbol refs / compile status / wiring_contract fields, never "feels complete"); negotiation may only make criteria MORE specific. | J-58:L13-L13 |
| J58-E006 | E/CAP | NegotiationLoop in sprint_contract.rs: NAMED const cap default e.g. 3, SEPARATE from post-exec Ralph counter. On Rejected: feed objection, re-propose, re-judge until Approved or cap. | J-58:L29-L29 |
| J58-E007 | E/LIMIT | Granularity-monotonicity: re-proposed contract must strictly increase acceptance-criteria specificity (criteria count must not decrease; each still passes I-21 checkability); loosening re-proposal is itself Rejected. | J-58:L30-L30 |
| J58-E008 | E/FAIL | Cap-escalation: non-convergence escalates to D-13 (reuse AdversaryEscalated) OR surfaces to user; adversary produces concrete counter-contract not a score; NEVER silently proceed to execution on never-Approved. | J-58:L31-L31 |
| J58-E009 | E/EVENT | Reuse SprintContractEvent; ADD Renegotiated{round, criteria_count_before, criteria_count_after}; TOTAL_VARIANTS +1; no new action_type. | J-58:L32-L32, L110-L110 |
| J58-E010 | E/INT | Production: I-21 sprint-contract dispatch in task_runner / search ladder pre-exec approval phase before generator writes code. Test never R7. | J-58:L46-L48 |
| J58-E011 | E/SURFACE | I-21 contract card (PlanApprovalCard sibling) gains negotiation-round counter + per-round criteria-count delta; Tauri read of Renegotiated rows; cross_lang_diff PASS. | J-58:L57-L58, L102-L102 |
| J58-E012 | E/OBS | Fixture vague_contract.toml: Rejected -> Renegotiated{criteria_count rises} -> Approved; evaluator payload every round has ZERO generator turns. Negative: unbounded; loosening Approved; scratchpad in evaluator; execution without Approved. | J-58:L59-L62 |
| J58-E013 | E/TEST | j58_negotiation_loop_contract: vague->tighten within cap; R8 zero generator rows; loosening Rejected; cap->AdversaryEscalated; N+1 impossible; Renegotiated carries before/after counts. | J-58:L96-L97 |
| J58-E014 | E/CFG | No new flag (tightens existing WIRED loop). Cap NAMED const, tunable, default 3, never unbounded. | J-58:L86-L86, L114-L114 |
| J58-E015 | E/LIMIT | Stub bans: no shared context; no unbounded negotiation; no loosening admitted; no execute without Approved; no new action_type; no reimplementation of I-21/D-13/I-1; no .unwrap on negotiation path. | J-58:L49-L56 |
| J58-E016 | E/DATA | MODIFY sprint_contract.rs ~150 LOC, ssb Renegotiated arm, dashboard contract card; CREATE contracts + J-58-decisions.md. | J-58:L99-L104 |
| J58-A001 | A/FLOW | Whether generator re-proposes autonomously or orchestrator re-dispatches is decide-at-edit; I-1 supervisor seam is natural driver. | J-58:L112-L113 |
| J58-I001 | I/LIMIT | Criteria-count rise is a PROXY for specificity; redundant criteria can game the count; deterministic-checkability + cap bound but do not prove genuine quality (named residue; quality shows in post-exec Ralph). | J-58:L94-L94, L115-L115 |

Category coverage: capabilities/surfaces ✓ (contract card, round counter); workflows/negotiation ✓; events ✓; security/isolation ✓; failures/escalation ✓; tests ✓; constraints ✓; deps ✓.

### J-67 — multi-mode gate actions

Source: `docs/thefinalglance/J-plan/J-67_multimode_gate_actions.md` (70/70 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| J67-E001 | E/STATUS | J-67 Track J NASA-gates wave, lever CANT-DERAIL, EXTENSION(J-12), effort M, WAVE NASA-GATES-4, depends J-66 (verdict) + J-69 (journaled inserts/suppresses). | J-67:L1-L3 |
| J67-E002 | E/CAP | Upgrades J-12 boolean Block(msg) to Edit-Automata typed GateAction {Accept, Suppress(reason), Insert(Vec<RemediationAction>), Block(msg)}; Edit = suppress + insert (e.g. redact secret from tool result then proceed). | J-67:L3-L5, L9-L12 |
| J67-E003 | E/FLOW | Existing gate chain (rules-engine 3F / hook chain 4C / J-12) returns GateAction instead of bool; orchestrator applies it via dispatcher in PreToolUse / J-12 H2 chain. | J-67:L9-L9, L18-L18 |
| J67-E004 | E/FLOW | SUPPRESS: drop dangerous tool-call, emit journaled suppressed row, continue run (agent self-corrects from absence + row). | J-67:L10-L10 |
| J67-E005 | E/FLOW | INSERT: inject mandatory step (approval, declassify, taint-screen) BEFORE side effect; inserted actions journaled in SSB total order BEFORE gated action. | J-67:L11-L11 |
| J67-E006 | E/FLOW | EDIT (suppress+insert): redact secret from tool RESULT (H4 annotator surface) before model, journaling the redaction. | J-67:L12-L12 |
| J67-E007 | E/SEC | ENFORCEABILITY CLASSIFIER (ND-1/W2): before mode assignment, deterministic classifier tags suppressible / insertion-enforceable / not-runtime-enforceable; non-safety property REFUSED edit semantics at authoring. | J-67:L13-L13 |
| J67-E008 | E/LIMIT | EXCLUDED: Insert/Edit without journaling (RRA break, ties J-69); edit-semantics for liveness/non-safety (ND-1); LLM choosing GateAction (verdict from J-66 or rules, TD-3 — LLM only RECEIVES Block msg); second gate chain (extend J-12). | J-67:L14-L14 |
| J67-E009 | E/DEP | Extends J-12 RuntimeInterventionHarness (H2 pre-tool at runtime_setup.rs:120; H4 annotators); rules 3F + hooks 4C; J-66 SsbMonitorVerdict; J-69 chain-scan + SSB single writer; G-1 taint + trust_dialog.rs for Insert remediation. | J-67:L16-L21 |
| J67-E010 | E/DATA | NEW crates/orchestrator/src/intervention/gate_action.rs (~200 LOC): enum + apply dispatcher + ND-1 classifier; MODIFY J-12 dispatcher + PreToolUse return type. | J-67:L24-L24 |
| J67-E011 | E/EVENT | NO new action_type preferred: EXTEND J-12 HarnessInterventionEvent Kind with Suppressed / Inserted / Edited. If J-12 unshipped: +GateActionApplied baseline+1. LIVE baseline 268/577. | J-67:L25-L25, L57-L57 |
| J67-E012 | E/SURFACE | frontend_pair: GateAction rows on J-12 intervention drawer (decide at edit) or W-6 absence. | J-67:L26-L26, L47-L47 |
| J67-E013 | E/INT | Production: real elai run whose tool-call trips Suppress/Insert/Edit through live PreToolUse. Test never R7. | J-67:L38-L40 |
| J67-E014 | E/LIMIT | Stub bans: no unjournaled Insert/Edit; no edit on non-safety policy; no LLM mode selection; no second chain; no silent Suppress (always journaled row, R2). | J-67:L41-L46 |
| J67-E015 | E/CFG | Under ELAI_GATE_ACTION_DISABLE chain falls back to J-12 Block-only and SAYS so (no silent behavior change). | J-67:L51-L51 |
| J67-E016 | E/OBS | Expect HarnessInterventionEvent Suppressed (run continues); Inserted with approval/declassify rows before gated side-effect. Negatives: unjournaled Insert fails RRA-replay; nonsafety Edit fails classifier; silent Suppress fails. | J-67:L48-L51 |
| J67-E017 | E/SEC | Mode/policy mapping security-sensitive: enforceability classifier is frozen (gate-manifest, R8) so executing run cannot quietly upgrade Block to Edit on itself. | J-67:L65-L65 |
| J67-E018 | E/CFG | SUPPRESS OBSERVE-ONLY first (GD-4); measure self-correction vs silent stall; suppressed-then-stalled is a finding not success. | J-67:L64-L64 |
| J67-A001 | A/DATA | Exact journaled row schema for re-enacting inserts decided with J-69 at edit (load-bearing for RRA-replay). | J-67:L62-L63 |
| J67-A002 | A/SURFACE | Exact intervention-drawer presentation decide-at-edit. | J-67:L26-L26 |
| J67-I001 | I/STATE | NASA runtime spine order implied: J-69 integrity -> J-66 verdict -> J-67 typed action. | J-67:L67-L68 |

Category coverage: capabilities/surfaces ✓ (intervention drawer, disable flag); workflows/modes ✓; events ✓; security/classifier ✓; failures ✓; constraints ✓; deps ✓. No standalone CLI verb beyond gate disable flag.

### J-82 — deterministic token-efficient emission

Source: `docs/thefinalglance/J-plan/J-82_deterministic_token_efficient_emission.md` (110/110 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| J82-E001 | E/STATUS | J-82 PLANNED WAVE KAT-2 effort M NEW (measure-first); composes J-24/H-4/mcp_budget; deterministic non-generative token reduction on NON-critical text only. | J-82:L1-L3 |
| J82-E002 | E/CAP | Three legs: (a) token-efficient OUTPUT format (TOON or ELAI-measured alt) for heaviest tool returns (CLI + IPC + harness tools); (b) dictionary-encoding for REPETITIVE tool-output only (logs/test spew/telemetry) NEVER code, only if dropped volatiles also journal-archived; (c) protected-span tokenizer priority fence>inline>URL>datetime>version>path>identifier>prose; structured pass BYTE-FOR-BYTE. | J-82:L4-L5 |
| J82-E003 | E/LIMIT | Hard boundary: deterministic-reversible ONLY (IC-3 ciphers banned); NO LLM summarization (HD-2); NEVER on code facts/identifiers/must-preserve (K-12); done-gate is ELAI-MEASURED token delta never literature number; OBSERVE/measure before default-ON (GD-4). | J-82:L5-L5 |
| J82-E004 | E/DEP | mcp_budget.rs TokenBudget/enforce DEFAULT_BUDGET_TOKENS=5000 governs tool-LIST size not OUTPUT (gap leg a fills); J-24/H-4/H-34; context_budget.format(); I-5 pristine_path_filter + H-6 masking; K-12 faithfulness. | J-82:L7-L8, L15-L21 |
| J82-E005 | E/CFG | Strategic fork: axi prefers CLI+TOON over MCP; ELAI keeps MCP and applies AXI principles to own CLI/harness tools instead. | J-82:L12-L13 |
| J82-E006 | E/TEST | Measurement harness FIRST: `elai bench tool-output` measures token delta per leg on REAL heavy returns under deterministic corpus/ELAI tokenizer; no leg default-on without measured positive delta; marginal delta legitimate KEEP-OFF. | J-82:L29-L29, L68-L68 |
| J82-E007 | E/CAP | Leg (a): adopt TOON (toon-go pin v0.0.0-20251202084852; axi-sdk-js ^0.1.7 UNVERIFIED — npm view first; fetch grammar at toonformat.dev, do not invent) OR ELAI-measured alternative; deterministic reversible to JSON. | J-82:L30-L30 |
| J82-E008 | E/CAP | Leg (b): collapse repetitive logs only where volatiles archived to journal; REJECT stream otherwise; NEVER on code. | J-82:L31-L31 |
| J82-E009 | E/CAP | Leg (c): priority-ordered protected spans; drop prose fillers only; harden I-5/H-6; composes K-12 must-preserve never dropped. | J-82:L32-L32 |
| J82-E010 | E/DATA | produces: token_emit/ (format, log_dict, protected_span); `elai bench tool-output`. Files under .elai_cc/crates/orchestrator/src/token_emit/. | J-82:L36-L38, L89-L94 |
| J82-E011 | E/INT | Production: Tier-C IPC tool-result path + elai CLI heavy-return path; bench CLI; log lane on build/test-output emit. Test never R7. | J-82:L42-L45 |
| J82-E012 | E/LIMIT | Stub bans: no LLM/ciphers; never on code/must-preserve; no log collapse without journal archive; no literature done-gate; no unverified dep; no .unwrap. | J-82:L46-L50 |
| J82-E013 | E/SURFACE | frontend_pair N/A justified: no dashboard surface; measured deltas via bench + J-29 usage panels. | J-82:L51-L52, L79-L79 |
| J82-E014 | E/EVENT | Bench rows prefer-reuse bench family / I-27 ConvergenceCurve-style; net 0..+1 from LIVE ~267/577 decide at edit; no new env gate (W-2). | J-82:L55-L55 |
| J82-E015 | E/CFG | Legs default-OFF until measured; then default-ON behind *_DISABLE kill-switch; no new ELAI_*_ENABLE enable-flag. | J-82:L77-L77 |
| J82-E016 | E/TEST | j82_token_emit_contract: (a) leg c preserves fence/path/identifier byte-for-byte; (b) format round-trips JSON; (c) leg b refuses unarchived volatiles; (d) leg b never on code; (e) bench deterministic under ELAI_SEED; (f) K-12 must-preserve never dropped. | J-82:L86-L87 |
| J82-E017 | E/OBS | Expect measured delta per leg in J-82-decisions.md; structured spans survive; negatives: altered code span / unarchived log collapse rejected. | J-82:L56-L56 |
| J82-A001 | A/CAP | TOON-adopt vs ELAI-roll compact format decided at authoring from measured delta + anti-slopsquat result. | J-82:L102-L103 |
| J82-A002 | A/EVENT | Exact SSB bench family reuse vs +1 decided at edit (W-9). | J-82:L55-L55 |
| J82-I001 | I/LIMIT | honey-for-devs median 0.0% on real terse prompts implies scope HEAVIEST returns first; KEEP-OFF on light returns is correct. | J-82:L13-L13, L104-L104 |

Category coverage: capabilities/CLI bench ✓; intentionally no UI; config/default-OFF ✓; events prefer-reuse ✓; security of code preservation ✓; tests ✓; constraints ✓; deps ✓.

### J-91 — re-executable RRA

Source: `docs/thefinalglance/J-plan/J-91_reexecutable_rra_track.md` (74/74 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| J91-E001 | E/STATUS | J-91 Track J lacuna-fill, lever EXTERNAL-FAITHFULNESS, BUILD = J-14 EXTENSION, effort L, WAVE 2, depends J-14, J-87, G-1. Technical companion to K-20 differentiator. | J-91:L1-L3 |
| J91-E002 | E/CAP | Premise: RRA is REPLAYABLE (action hashes) but not RE-EXECUTABLE (third party re-runs against recorded external world for same sealed verdict). | J-91:L5-L5 |
| J91-E003 | E/CAP | CASSETTE: capture G-1 taint-ingested untrusted BYTES (WebFetch/MCP/file/sandbox) as content-keyed VCR cassette (J-14 rra export-fixture shape). | J-91:L8-L8 |
| J91-E004 | E/CAP | PINNED ENV: OCI digest + Nix-style lockfile + CAS fixtures (Bazel action-cache shape) for byte-reproducible re-run environment. | J-91:L10-L10 |
| J91-E005 | E/CAP | RE-RUN VERIFY: third party re-EXECUTES recorded actions against cassette under pinned env; checks I-45-sealed verdict matches (not just action-hash). J-14 verify-tier extension. | J-91:L11-L11 |
| J91-E006 | E/STATE | TIERED FIDELITY with honest downgrade: T0 action-faithful / T1 token-faithful (only where J-13 batch-invariance + C7-SIDECAR allow) / T2 score-faithful. Self-describing; cannot deliver T1 -> visible T0 downgrade. | J-91:L12-L12 |
| J91-E007 | E/FAIL | WORLD-DRIFT as fidelity downgrade, never hidden: cassette proves OLD world; surface staleness as downgrade not portal to live world. | J-91:L13-L13, L66-L67 |
| J91-E008 | E/LIMIT | EXCLUDED: re-run against LIVE world; claiming T1 where attention non-batch-invariant; snapshotting non-snapshottable effects (payments/emails stay action-faithful-only with note); re-authoring G-1/G-2/J-14. | J-91:L14-L14 |
| J91-E009 | E/DEP | Substrate: J-14 PLANNED T0/T1/T2; G-1 WIRED 4 ingest sites; G-2 WIRED RRA bundle; I-45 PLANNED sealed receipts; G-9 DARK auth-scrub; K-20 + J-87 completeness. LIVE ~268/577. | J-91:L16-L23 |
| J91-E010 | E/CAP | produces: re_execute_rra, export_cassette, pin_environment in rra.rs; CLI `rra re-execute --cassette`. | J-91:L26-L26, L32-L36 |
| J91-E011 | E/EVENT | Reuse ReproPackEvent +Kind CassetteExported / ReExecuted / FidelityDowngraded; net 0 (J-14 planned event); re-read at edit. | J-91:L27-L27 |
| J91-E012 | E/CFG | FEATURE FLAG: ELAI_REEXECUTE_RRA_ENABLE (kill-switch; re-execution opt-in evidence not hot-path gate) + wire-dark-allowlist row with promotion plan (W-2/R9). | J-91:L28-L28 |
| J91-E013 | E/SURFACE | frontend_pair: re-execution result + fidelity-tier badge on RRA/reviewer surface (G-5/K-11); cross_lang_diff or W-6 absence. | J-91:L29-L29, L51-L51 |
| J91-E014 | E/INT | Production: `elai run --emit-rra` then `rra re-execute` on DIFFERENT machine/process against cassette. Test never R7. | J-91:L42-L44 |
| J91-E015 | E/LIMIT | Stub bans: no live-world re-run; no dishonest T1; no hidden world-drift; no verbatim auth header in cassette (G-9 scrub); no re-author G-1/G-2/J-14. | J-91:L45-L50 |
| J91-E016 | E/OBS | Trigger WebFetch then re-execute on fresh process; also stale-cassette case. Expect CassetteExported, ReExecuted on verdict match, FidelityDowngraded on attention/world-drift. | J-91:L52-L54 |
| J91-E017 | E/FAIL | Negative: re-run reaching live network FAILS; unscrubbed auth header FAILS export; T1 on non-batch-invariant REFUSED and downgraded to T0 with visible note. | J-91:L55-L55 |
| J91-E018 | E/DEP | Completeness depends on J-87: re-executing incomplete record faithfully reproduces incomplete record. Non-snapshottable effects stay action-faithful-only; J-93 C3-SESSIONDAG compensation registry marks them. | J-91:L68-L69 |
| J91-E019 | E/DEP | Composes J-14, G-1, G-2, I-45, G-9, J-13+J-94, J-87, K-20, K-11, G-6 SAFEDELTA. Restates none. | J-91:L63-L64 |
| J91-A001 | A/SURFACE | Exact badge placement on RRA/reviewer surface decide-at-edit. | J-91:L29-L29, L51-L51 |
| J91-A002 | A/DATA | Exact OCI/Nix/CAS pin format details left to implementation under the named shape. | J-91:L10-L10 |
| J91-I001 | I/STATUS | K-20 banks wedge claim; J-91 builds proof-of-execution that feeds G-6 believe-the-number envelope and K-11 human render. | J-91:L3-L3, L71-L72 |

Category coverage: capabilities/CLI surfaces ✓ (rra re-execute, emit-rra, badge); fidelity states ✓; events ✓; security/scrub ✓; failures/downgrade ✓; config flag ✓; tests via negatives ✓; constraints ✓; deps ✓.

## Cross-source interactions and lifecycle

| IX ID | Class | Interaction | Atom references |
|---|---|---|---|
| IX-001 | E | J-2 pairs with I-40 as inverse trust-layer halves: wrap others vs be driven by others through gate-respecting MCP. | J2-E014, J2-E032 |
| IX-002 | E | J-2 PRE_GATE security spine (C-42/E-24/G-4/E-30/G-3) must be live before any control-plane exposure; same destructive/blast gates J-16 hardens at audit time. | J2-E006, J2-E015; J16-E005 |
| IX-003 | E | J-54 FreePool and J-55 modality router are explicitly parallel-safe (disjoint files): J-54 chooses WHICH free/local model; J-55 chooses HOW it acts. | J54-E002; J55-E001, J55-E002 |
| IX-004 | E | J-54 verification-lane trust-marker (non_reproducible + FreePoolTrustDowngrade + G-10 notice) is the honesty wedge for free throughput. | J54-E018, J54-E022 |
| IX-005 | E | J-55 consumes J-51 tool_call_adherence, E-31 tier, I-36 compiled schemas, E-35 CodeAct/SEARCH-REPLACE, H-4 progressive disclosure; sequences after J-51 when unshipped. | J55-E005, J55-E007–J55-E009, J55-A001 |
| IX-006 | E | J-57 depends on J-56 eval subset as promotion gate and reuses G-10 as prompt-layer rollback signal while preserving model surface-only posture. | J57-E001, J57-E008–J57-E009 |
| IX-007 | E | J-57 makes D-1 Prod-stage hashes load-bearing for select_template / I-36 build_system_prompt_for_role. | J57-E006, J57-E003 |
| IX-008 | E | J-58 wraps I-21 pre-exec approval without replacing post-exec Ralph loop; adds Renegotiated arm only. | J58-E001, J58-E004, J58-E009 |
| IX-009 | E | J-58 and J-57 both harden the generate/verify loop: contracts before build; prompts promoted only on no-regression. | J58-E002; J57-E002 |
| IX-010 | E | J-67 extends J-12 gate chain; verdict source is J-66; Insert/Edit journaling integrity depends on J-69. | J67-E001, J67-E009, J67-E011 |
| IX-011 | E | J-67 Insert remediation reuses G-1 taint + trust_dialog; Edit redaction rides J-12 H4 annotators. | J67-E006, J67-E009 |
| IX-012 | E | J-41 pipeline consumes J-40 ranked spans as evidence set and activates F-12/H-53/E-35/D-16 closed-set grounding. | J41-E002, J41-E014 |
| IX-013 | E | J-82 extends mcp_budget from list to OUTPUT and composes J-24/H-4/H-34 make-MCP-cheap thread without abandoning MCP. | J82-E002, J82-E004, J82-E005 |
| IX-014 | E | J-82 leg (c) is guarded by K-12 must-preserve; leg (b) journal-archive precondition is the losslessness gate. | J82-E002, J82-E008–J82-E009 |
| IX-015 | E | J-91 extends J-14 verify tiers + cassette; consumes G-1 bytes, G-2 bundle, I-45 seal, G-9 scrub; completeness honesty depends on J-87. | J91-E001, J91-E009–J91-E011, J91-E018 |
| IX-016 | E | J-91 fidelity T1 bound by J-13 batch-invariance + J-94 C7-SIDECAR; non-snapshottable effects deferred to action-faithful with J-93 marks. | J91-E006, J91-E008, J91-E018 |
| IX-017 | E | J-16 cone is substrate for H-52/G-5 review and is widened later by J-18 LSP / J-90 cross-lang edges (named as consumers, not in-scope builds). | J16-E019, J16-E022 |
| IX-018 | E | Multiple chunk units add or extend SSB families requiring live baseline re-read: McpControlPlaneEvent or C-39 reuse; BlastRadiusEvent; FreePoolEvent; ModalityRouted; PromptPromotionEvent; SprintContractEvent+Renegotiated; HarnessInterventionEvent kinds or GateActionApplied; ReproPackEvent kinds; J-82 bench prefer-reuse. | J2-E013; J16-E012; J54-E019; J55-E011; J57-E011; J58-E009; J67-E011; J82-E014; J91-E011 |
| IX-019 | E | User-facing control surfaces cluster: `elai mcp-serve` + session drawer (J-2); RuntimePanel free-pool + non-reproducible badge (J-54); modality badge/schema tokens (J-55); `elai prompt promote` + D-26 stage badge (J-57); contract-card round counter (J-58); blast-radius audit panel (J-16); intervention drawer GateAction rows (J-67); `elai bench modality` / `elai bench tool-output` (J-55/J-82); `rra re-execute` + fidelity badge (J-91). | J2-E020–J2-E022; J54-E020–J54-E021; J55-E012–J55-E013; J57-E007, J57-E013; J58-E011; J16-E013; J67-E012; J82-E006; J91-E010, J91-E013 |
| IX-020 | E | Kill-switch / gate postures differ by design: J-54 assignment-gated (no ENABLE flag); J-55 ELAI_MODALITY_ROUTER_DISABLE default-ON; J-67 ELAI_GATE_ACTION_DISABLE falls back Block-only loudly; J-82 legs default-OFF until measured then *_DISABLE; J-91 ELAI_REEXECUTE_RRA_ENABLE opt-in evidence. | J54-E023; J55-E015; J67-E015; J82-E015; J91-E012 |
| IX-021 | I | Free-pool non-reproducible runs interacting with J-91 re-execution create an honesty chain: free verification lanes must not claim sealed reproducibility that re-exec would falsify. | J54-E018; J91-E002, J91-E005 |
| IX-022 | E | J-2 session_rewind NotAvailable degradation and J-91 incomplete-record honesty both refuse to fake missing substrate (A-1 chain / J-87). | J2-E010; J91-E018 |

## Explicit ambiguity and conflict register

| Register ID | Kind | Unresolved issue or required correction | Atom references |
|---|---|---|---|
| AC-001 | ambiguity | J-2 mid-FSM task_submit queue-vs-refuse and C-39 reuse-vs-new SSB are edit-time decisions. | J2-A001, J2-A002 |
| AC-002 | ambiguity | J-16 cone depth/suite-cap values and first-slice UI placement are unspecified. | J16-A001, J16-A002 |
| AC-003 | ambiguity | J-41 BinaryGroundingJudge vs label map; ClosedSet sidecar support; EXP-2 model choice. | J41-A001–J41-A003 |
| AC-004 | ambiguity | J-54 per-pool budget surface, uncensored default rows, multi-key env vs J-30 generalization. | J54-A001–J54-A003 |
| AC-005 | ambiguity/dependency | J-55 adherence signal if J-51 unshipped; directional paper thresholds until bench runs. | J55-A001–J55-A002 |
| AC-006 | ambiguity | J-57 crate ownership of load path and optional surface-only rollback edge case. | J57-A001–J57-A002 |
| AC-007 | ambiguity | J-58 autonomous generator re-propose vs orchestrator re-dispatch. | J58-A001 |
| AC-008 | ambiguity | J-67 insert row schema with J-69; drawer presentation. | J67-A001–J67-A002 |
| AC-009 | ambiguity | J-82 TOON vs roll-own; exact bench SSB family. | J82-A001–J82-A002 |
| AC-010 | ambiguity | J-91 pin format details and badge placement. | J91-A001–J91-A002 |
| AC-011 | residue | J-58 criteria-count is a granularity PROXY (named); J-57 gate strength = corpus coverage; J-54 8-hex fp collision residue; J-82 marginal-delta KEEP-OFF; J-41 EXP-2 may force fine-tune. | J58-I001; J57-I001; J54-I001; J82-I001; J41-E003 |

## Summary additions and corrections

Orientation comparison target: `docs/thefinalglance/feature-discovery/10-J-SUMMARY_INDEX.discovery.md`. “ADD” means an explicit source behavior omitted or materially compressed by the summary; “CORRECT/QUALIFY/REMOVE” means the summary’s wording changes scope or certainty. These are relational records, not additional feature atoms.

### J-2 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-J2-001 | **ADD** Wave-4 deliberate-last status, effort estimate, parallel-safe set, and NOT-J-1 shared CLI seam. | 10-J:L46-L56 | J2-E001, J2-E005, J2-E007 |
| SD-J2-002 | **ADD** exact module path mcp_server.rs, `elai mcp-serve` production entrypoint, and stub bans (no trusted-server bypass, no parallel status cache, no TCP). | 10-J:L46-L56, L1318 | J2-E016, J2-E022–J2-E023 |
| SD-J2-003 | **ADD** concurrency rule (no second concurrent session driver) and session-scoped locks reuse. | 10-J:L46-L56 | J2-E018 |
| SD-J2-004 | **ADD** full contract arms including NotAvailable rewind, malformed-frame survival, and --skip-authz broken arm. | 10-J:L56 | J2-E025–J2-E026 |
| SD-J2-005 | **ADD** entire J-15 amendment: BSP Script-step/Set-step extensions, MCP tool-config host delegation, transparent-proxy recording shape. | 10-J:L188 | J2-E030–J2-E032 |
| SD-J2-006 | **ADD** multi-client and remote-auth deferrals; A-1 conditional NotAvailable residue. | 10-J:L46-L56 | J2-E010, J2-E029 |

### J-16 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-J16-001 | **ADD** exact post-EXECUTING/pre-AUDITING boundary, auditor ownership R8 freeze, and F-4 sibling framing. | 10-J:L195-L204 | J16-E005, J16-E009, J16-E011 |
| SD-J16-002 | **ADD** OBSERVE-ONLY default until >=3-run FP bound then wire-dark-allowlist BLOCK promotion; ad-hoc scope fallback. | 10-J:L202 | J16-E006, J16-E020 |
| SD-J16-003 | **ADD** full BlastRadiusEvent kinds, LIVE 231/542 baseline stamp, and production R7 AUDITING reachability. | 10-J:L203, L1332 | J16-E012, J16-E015 |
| SD-J16-004 | **ADD** declared coverage / FAIL-LOUD unmappable-diff rule, cap logging (no silent truncation), and J-18 LSP composition. | 10-J:L204 | J16-E018–J16-E021 |
| SD-J16-005 | **ADD** top50 behavior mapping #14/#15/#16/#29/#31 and explicit non-goals (LLM size rubric, auto-revert, new traversal). | 10-J:L195-L204 | J16-E004, J16-E010 |

### J-41 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-J41-001 | **ADD** exact fingerprint thresholds (0.8 fuzzy / 0.85 cosine), plagiarism floor 8-10 words, and EXP-2 <75% transfer gate. | 10-J:L460-L468 | J41-E003–J41-E004 |
| SD-J41-002 | **ADD** ONNX model path/absence fail-safe SKIP semantics and BinaryGroundingJudge adapter requirement. | 10-J:L463 | J41-E005, J41-E008, J41-E016 |
| SD-J41-003 | **ADD** net-0 action-type preference, LIVE 268/577 stamp, and DD-3 default-ON after EXP+FP bound. | 10-J:L468 | J41-E009, J41-E015 |
| SD-J41-004 | **ADD** J-40 span-budget false-ungrounded risk and E-35 ClosedSet sidecar PARTIAL caveat. | 10-J:L460-L468 | J41-A002, J41-I001 |

### J-54 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-J54-001 | **ADD** exact fallback chain order, Pollinations keyless overflow URL/key-literal, and uncensored empty-fallback invariant I-6. | 10-J:L595-L604 | J54-E009, J54-E014–J54-E015 |
| SD-J54-002 | **ADD** 429 header priority order, soonest_reset = min, day-vs-minute persistence split, quota_state path. | 10-J:L598 | J54-E008, J54-E010 |
| SD-J54-003 | **ADD** FreePoolEvent three arms, assignment-as-kill-switch W-2 posture, and ABORT conditions. | 10-J:L595-L604, L1334 | J54-E019, J54-E023, J54-E026 |
| SD-J54-004 | **ADD** full contract suite, integration smokes (rotation/trust/exhaustion), and separation from E-39 free arms. | 10-J:L595-L604 | J54-E006, J54-E024, J54-E028 |
| SD-J54-005 | **ADD** open authoring choices: per-pool budget, commented uncensored rows, env multi-key vs J-30. | 10-J:L595-L604 | J54-A001–J54-A003 |
| SD-J54-006 | **ADD** redteam attack catalog (wire-dark, raw-key leak, silent swap, uncensored fallback, sha256) as binding negative design. | 10-J:L595-L604 | J54-E024, J54-E027 |

### J-55 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-J55-001 | **ADD** TextFallback arm, named-constant threshold seeding from bench, and three independent token sinks framing. | 10-J:L606-L615 | J55-E002, J55-E007, J55-E010 |
| SD-J55-002 | **ADD** ELAI_MODALITY_ROUTER_DISABLE default-ON kill-switch and exact ModalityRouted arms including schema_tokens. | 10-J:L606-L615, L1334 | J55-E011, J55-E015 |
| SD-J55-003 | **ADD** production resolve-before-each-turn reachability and full contract arms. | 10-J:L606-L615 | J55-E016, J55-E019 |
| SD-J55-004 | **ADD** paper IDs as DESIGN-scoped motivation only; measured crossover is load-bearing proof (GD-5). | 10-J:L606-L615 | J55-E004, J55-A002, J55-I001 |
| SD-J55-005 | **ADD** J-51 sequencing contingency (params+tier first). | 10-J:L606-L615 | J55-A001 |

### J-57 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-J57-001 | **ADD** exact PromptPromotionEvent four arms and governed provenance fields (from/to/eval|drift/reason). | 10-J:L627-L634, L1335 | J57-E010–J57-E011 |
| SD-J57-002 | **ADD** model-vs-prompt asymmetry: auto-demote prompts; surface-only models (G-10). | 10-J:L631 | J57-E005, J57-E009 |
| SD-J57-003 | **ADD** D-26 stage badge + no-new-Tauri justification; no new ENABLE flag; fail-closed BLOCK-on-regression. | 10-J:L633 | J57-E013, J57-E015, J57-E019 |
| SD-J57-004 | **ADD** production triple reachability (CLI/load/canary handler) and GD-5 corpus-coverage residue. | 10-J:L627-L634 | J57-E012, J57-I001 |

### J-58 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-J58-001 | **ADD** cap default example 3 separate from Ralph counter; adversary produces counter-contract not score. | 10-J:L636-L643 | J58-E006, J58-E008 |
| SD-J58-002 | **ADD** exact Renegotiated payload fields and TOTAL_VARIANTS +1 / no new action_type discipline. | 10-J:L642, L1335 | J58-E009 |
| SD-J58-003 | **ADD** I-21 deterministic-checkability re-run on every re-proposal and criteria-count PROXY residue. | 10-J:L639 | J58-E005, J58-E007, J58-I001 |
| SD-J58-004 | **ADD** production sprint dispatch reachability and decide-at-edit re-dispatch ownership. | 10-J:L636-L643 | J58-E010, J58-A001 |

### J-67 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-J67-001 | **ADD** module path gate_action.rs, PreToolUse bool->GateAction type change, and prefer-extend HarnessInterventionEvent kinds. | 10-J:L721-L729, L1336 | J67-E010–J67-E011 |
| SD-J67-002 | **ADD** ND-1 enforceability classifier frozen under R8 and OBSERVE-FIRST suppress measurement. | 10-J:L728-L729 | J67-E007, J67-E017–J67-E018 |
| SD-J67-003 | **ADD** ELAI_GATE_ACTION_DISABLE loud fallback; insert-before-side-effect total-order rule; J-69 schema co-decision. | 10-J:L729 | J67-E005, J67-E015, J67-A001 |
| SD-J67-004 | **ADD** Ligatti/W3 grounding and NASA spine completion claim with J-66/J-69. | 10-J:L721-L729, L1273 | J67-E001, J67-I001 |

### J-82 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-J82-001 | **ADD** measure-first `elai bench tool-output` as done-gate, default-OFF-until-measured then *_DISABLE, and KEEP-OFF legitimacy. | 10-J:L866-L874, L1320 | J82-E006, J82-E015 |
| SD-J82-002 | **ADD** anti-slopsquat pins (toon-go, axi-sdk-js UNVERIFIED), grammar fetch-not-invent, and TOON-vs-roll decision. | 10-J:L874 | J82-E007, J82-A001 |
| SD-J82-003 | **ADD** mcp_budget DEFAULT_BUDGET_TOKENS=5000 list-vs-output gap, axi/MCP strategic fork, and N/A frontend justification via J-29. | 10-J:L872 | J82-E004–J82-E005, J82-E013 |
| SD-J82-004 | **ADD** full contract arms and honey-for-devs marginal-delta residue. | 10-J:L866-L874, L1296 | J82-E016, J82-I001 |

### J-91 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-J91-001 | **ADD** exact symbols re_execute_rra/export_cassette/pin_environment and CLI `rra re-execute --cassette`. | 10-J:L958-L966 | J91-E010 |
| SD-J91-002 | **ADD** ELAI_REEXECUTE_RRA_ENABLE opt-in + wire-dark-allowlist; different-machine production reachability. | 10-J:L958-L966 | J91-E012, J91-E014 |
| SD-J91-003 | **ADD** full ReproPackEvent kind list CassetteExported/ReExecuted/FidelityDowngraded and negative arms (live network, unscrubbed auth, dishonest T1). | 10-J:L965, L1330 | J91-E011, J91-E016–J91-E017 |
| SD-J91-004 | **ADD** G-9 scrub / DO-NOT-COPY auth, J-93 non-snapshottable mark, and K-20/G-6/K-11 wedge consumer chain. | 10-J:L964-L966 | J91-E015, J91-E018–J91-E019, J91-I001 |

## Exhaustiveness, zero-omission, and duplicate audit

- Source coverage: **10/10 files**, **1,219/1,219 logical lines**, **151,045/151,045 bytes** read and reconciled; source SHA mismatches **0**.
- Atomic source ledger: **237 unique atoms** = **206 explicit**, **010 inferred**, **021 ambiguous**, **000 conflicting/corrective**.
- Per-file atom counts: J-2 **035**; J-16 **025**; J-41 **020**; J-54 **032**; J-55 **022**; J-57 **022**; J-58 **018**; J-67 **021**; J-82 **020**; J-91 **022**. Sum **237**.
- Relational interaction records: **22** (`IX-001`–`IX-022`). Summary-delta records: **046** (`SD-*`). Ambiguity/conflict register: **11** (`AC-001`–`AC-011`). These records reference source atoms and are not double-counted as features.
- ID audit: every atomic table row has exactly one unique ID matching `J2/J16/J41/J54/J55/J57/J58/J67/J82/J91-[EIAC]nnn`. Duplicate atomic IDs **0**; missing/blank atomic IDs **0**; nonconforming atomic IDs **0**.
- Semantic duplicate audit: repeated premise/scope/wiring/grounding/acceptance statements were merged into one atom with multiple anchors. Interactions, conflict register, and summary deltas reference atoms instead of cloning them. Duplicate feature atoms found **0**.
- Omission audit: every source was checked against capabilities/actions/surfaces; workflows/states; schemas/data/lifecycle; flags/defaults/config; integrations/events; permissions/security/privacy/compliance; failures/fallback/retry/recovery; determinism/idempotency/concurrency; observability/evidence; tests/acceptance/done gates; constraints/non-goals; dependencies/status/deferred. Each per-file ledger explicitly names absent dimensions rather than inventing them. User-facing surfaces (CLI verbs, panels, badges, drawers, disable flags, benches) were recorded as standalone atoms where present.
- Interpretation audit: inferred, ambiguous, and conflicting/corrective atoms are visibly separated by ID/class. No inference is promoted to a requirement. Orientation-only claims remain in delta records and are not source atoms.
- Manifest-boundary audit: the only original sources opened were the ten exact DEEP-J-008 manifest rows. The only orientation artifact consulted was `10-J-SUMMARY_INDEX.discovery.md`.
- Output metrics after finalization: `output_lines=0514`; `output_bytes=76743`.
- Canonical normalized output SHA-256: 5e9f330564ec874163df4ac2903da800bd19699f1c1d76b07bb0973a463aaa70.
- Hash rule: compute SHA-256 over the exact final UTF-8 bytes including the final newline after replacing only the 64 hexadecimal characters on the `Canonical normalized output SHA-256` line with 64 ASCII zeroes. Fixed-width output metrics and digest substitution avoid impossible raw self-reference.
