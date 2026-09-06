# Surface digest: F + G + H (lines 55905-57357)

Source-only extract of dashboard-visible windows, drawers, panels, CLI verbs, toasts, and review gates from ELAI-EXHAUSTIVE-SUPER-FEATURE-LIST family surfaces F/G/H. Statuses are plan-recorded (shipped/DONE/WIRED vs planned/BUILD vs DEFERRED vs wire-dark/DARK vs mixed/partial vs superseded). Atom IDs preserved on every claim.

---

## F

### Surface inventory

- **VibeGateDrawer** [type: drawer] [atoms: F-002/DF002-F1-031, F-002/DF002-F1-032, SUMMARY/F1-31, SUMMARY/F1-37] [status: planned/BUILD, Tier-1 axes wired in plan]
  Pre-write security block list under `DrawerKey 'vibegate'`. Frontend pair: `VibeGateDrawer.svelte` + `commands_vibegate.rs::list_vibegate_blocks`. Shows block kind, rule ID, file path, citation. Entry: dashboard drawer after PreToolUse block; evidence also via hook-executions.log. Exit: inspect-only (block already refused write).

- **VibeGate PreToolUse compound gate** [type: gate/background] [atoms: F-002/DF002-F1-001..030, SUMMARY/F1-01..41] [status: mixed — Tier-1 code+secret planned fail-closed; dep/skill axes reserved]
  Synchronous in-process PreToolUse over candidate write content. Tier-1 active axes: code vulnerability (`RuleEngine` over immutable `vibesec` security domain) + secret regex (`vibegate_secrets.rs`: AWS AKIA, PEM headers, password/secret/token/api_key assignments). Any axis Block → compound block; no prompt bypass. Kill switch only: `ELAI_VIBEGATE_DISABLED=1` (default ON; `ELAI_VIBEGATE_ENABLE` forbidden). SSB: `VibeGateBlocked` kinds CodeVuln/Secret live; DepHallucinated/Skill schema-reserved. Payload: kind, rule_id, file_path, source_blake3, citation, fsm_state. Tools covered: Edit, Write, NotebookEdit, Bash heredoc/redirect. Block before disk; baby-step `passed=false`, `block_source=T1Hook`.

- **SecurityRulesPanel** [type: panel/drawer] [atoms: F-002/DF002-F2-019, F-002/DF002-F2C-032, SUMMARY/F2C-28] [status: planned/BUILD]
  Recent security-rule block rows under `DrawerKey 'security_rules'`. Pair: `SecurityRulesPanel.svelte` + `commands_f2.rs::get_security_blocks_recent`. Product must not brand “VibeSec”; internal names VibeGate/security-rules (F-002/DF002-F2-042). F-2c reuses this panel for seccodeplt insecure-rate results; no new frontend (SUMMARY/F2C-29).

- **Insecure-rate / SecCodePlt bench surface** [type: CLI + panel tile] [atoms: F-002/DF002-F2-026, F2C-004..037, SUMMARY/F2-18, F2-27, F2C-09] [status: mixed — F-2b CLI/schema asserted shipped in deep; panel tile deferred]
  CLI: `elai bench seccodeplt` (corrected route via `bench_seccodeplt.rs`/`bench_cmd.rs`, not `benchmark.rs`). Emits `insecure_rate_measured` JSON under fixed `ELAI_SEED`. Superseded/deferred: `get_insecure_rate_latest` panel tile belongs to F-2b deferred contract (F-002/DF002-F2-020). Superseded spelling: `elai benchmark --suite seccodeplt` vs real `elai bench seccodeplt` (F-002/DF002-SD-015).

- **ToolGateInspector (Tool Filter / data-plane)** [type: panel section] [atoms: F-002/DF002-F3-029, F3-030, F3-031, F-002/DF002-SD-030] [status: planned/BUILD]
  Existing `ToolGateInspector.svelte` gains Tool Filter section; `commands_c35.rs` maps tool-filter events into extended `ToolGateRowDto`. Displays allow-set, commitment hash, quarantines, derived before/after ASR. CLI: `elai safety tool-filter --fixture …` → JSON (commitment hash, denied tool, fixture-suite ASR). Default-on; `ELAI_CAMEL_TOOLFILTER_ENABLE` is disable-only despite name [unclear parsing]. Pure set membership; no LLM override of denial.

- **AuditTimelineWindow (test integrity)** [type: window] [atoms: F-001/DF001-F4-E017, E023, SUMMARY/F4-25] [status: planned/BUILD]
  Dashboard: `get_test_integrity_events` + `TestIntegrityEventJson`; AuditTimelineWindow shows `fabricationBlocks` and red badge. Gate on drive_task post-loop before PhaseComplete (fsm.rs transition rejected — no integration rows). Empty stdout hashes empty bytes (None ≠ empty).

- **Staleness quarantine drawer** [type: drawer] [atoms: F-002/DF002-F5-025, F5-026, F5-027, SUMMARY/F5-25] [status: planned/BUILD]
  Knowledge/staleness drawer via `get_staleness_quarantine` (broadcast rows, DKG/Cosmos aggregate counts). CLI: `elai dkg staleness-tick --workspace .` forces decay tick. Cascade default-on; `ELAI_STALENESS_CASCADE_KILL=1` disable-only. Thresholds >1.0 flag / >2.0 quarantine: SHARD-origin, decide confirm (F-002/DF002-F5-036).

- **Skill library panel** [type: panel|drawer] [atoms: F-002/DF002-F6-021, F6-022, F6-039, F6-041, F6-049, SUMMARY/F6-34, F-002/DF002-C-009] [status: planned/BUILD; name conflict]
  Shows skill tiers S1–S4. CLI under `elai skills-mgmt`: `library list|show|promote|tick`; S4 promotion requires `--human-grant`. Dashboard S4-approval button DEFERRED; CLI human-grant is in-scope gate. Mount strategy: exactly one of MonitorWindow panel or `DrawerKey` after checking live surface. Conflict: contract names `SkillsPanel.svelte`/`commands_f6.rs` vs tasks name `SkillLibraryPanel.svelte`/`commands.rs`. Autonomy barrier default-on; `ELAI_SKILL_AUTONOMY_BARRIER_DISABLE=1` only; `ELAI_SKILL_LIBRARY_ENABLE` forbidden.

- **TemplateRankingDrawer** [type: drawer] [atoms: F-002/DF002-F7-012..014, F7-037..038, F7-040, F-002/DF002-C-012] [status: Half-B shipped asserted; lattice excluded]
  Pair: `TemplateRankingDrawer.svelte` + `commands_f7.rs::get_template_ranking`. Renders utility and first-try ratio side by side. CLI: `elai template-ranking list|show --concept-class X [--json]`. Default-on; `ELAI_TEMPLATE_CBR_ENABLE` disable-only. Superseded: lattice UI/`SafetyLatticeVerified`/`get_lattice_status` excluded from as-built (F7-037). Conflict: older MonitorWindow `TemplateRankingPanel` vs reconciled drawer. Superseded CLI: `elai formal-verify lattice` vs summary `elai verify safety-lattice`.

- **ReasoningDrawer (symbolic conflicts + Darwinian admits)** [type: drawer] [atoms: F-001/DF001-F8-E016..023, F11-E015..016, SUMMARY/F8-18, F11-35] [status: planned/BUILD; F-11 dual-gated]
  Symbolic conflicts: `get_symbolic_conflicts` + tone badges per kind; feature-off → empty array. Live trigger: contradictory_locked_variant.json → pending-not-applied + BLAKE3-prefix row + drawer. Darwinian Admits section: `get_darwinian_admits` (seq/time/kind/variant/parent/alpha/rejecting-guard/fingerprint). Production needs both `darwinian-evolve` feature AND `ELAI_DARWINIAN_LOOP_ENABLE=1` (DARK unless both). CLI: `elai darwin-loop`. Fallback proposed: default-on dry-run preview without flag (W-2 exception rejected path).

- **DepsSecurityPanel** [type: drawer] [atoms: F-001/DF001-F9-E016, E019, E020, SUMMARY/F9-24] [status: planned/BUILD; D-11 hard blocker]
  Daily block/advisory counts; DrawerKey + Tauri DTO + TS stub. Write-time slopsquat gate default-on; `ELAI_DEP_WRITETIME_GATE_DISABLE=1` only. CLI: `elai dep-check <path> [--offline]` (offline semantics [unclear]). Extends existence-and-age to Edit/Write PreToolUse + crates.io.

- **SecurityMonotonicityDrawer** [type: drawer] [atoms: F-001/DF001-F10-E017, E023, E025, SUMMARY/F10-24] [status: planned/BUILD]
  Pair: `SecurityMonotonicityDrawer` + `get_security_monotonicity_blocks`; also live REFINE via `ReflectEvent::SecurityRegressed`. CLI: `elai safety monotonicity --fixture <PATH>`. Default-on; `ELAI_SECURITY_MONOTONICITY_DISABLE=1`. Blocks REFINE commit when class set regresses.

- **MemoryFactsDrawer (semantic contradictions)** [type: drawer] [atoms: F-001/DF001-F12-E022, E028, SUMMARY/F12-31] [status: planned/BUILD; payload gap]
  `get_semantic_contradictions` + MemoryFactsDrawer: older/newer IDs, label, confidence, action, disposition tone. Route: live_cli post-reflect memory tick; SurfaceToUser is safe default (never auto-close on low confidence/judge error). Threshold 0.85. Kill: `ELAI_SEMANTIC_CONTRADICTION_KILL` disable-only. Conflict: SSB payload fields insufficient for dashboard-promised label/confidence/action/fingerprint (F-001/DF001-F12-C003).

- **Session archive / recovery drawer** [type: drawer] [atoms: F-001/DF001-F13-E019, SUMMARY/F13-26] [status: planned/BUILD; Svelte filename unresolved]
  `get_session_archives` + implement-time-resolved recovery drawer (source refuses to guess filename). Archives gated by `ELAI_SSB_ARCHIVE_ON_BUMP_ENABLE` (default-off ENABLE conflicts W-2 — unresolved F13-C005). Dashboard must not imply guaranteed recovery (best-effort). No restore command in scope (F13-A004).

- **Wiring / W-gate doctrine surfaces** [type: CLI/gate] [atoms: F-001/DF001-F0-E006..021, SUMMARY/FG-02..12, F-002/DF002-WD-*] [status: process doctrine]
  Session results intended dashboard-visible. DONE needs `wiring_contract_check.py --run-trigger`, frontend_pair, cross_lang_diff, dashboard E2E (W-6; N/A only for truly headless with one-line justification).

### User flows

- FLOW **VibeGate block write** [atoms: F-002/DF002-F1-015..035, SUMMARY/F1-32..37] [status: planned]: Agent Edit/Write/NotebookEdit/Bash-write → `dispatch_pre_tool_use` → `evaluate_vibegate` (code+secret) → if Block: HookFinding T1 vibegate + SSB `vibesec_*_blocked` + short-circuit (no disk) → drawer lists kind/rule/path/citation; else proceed. Kill-switch path: `ELAI_VIBEGATE_DISABLED=1` → gate off. Seed integrity fail at startup → refuse dispatcher.

- FLOW **SecCodePlt insecure-rate baseline** [atoms: F2C-023, F2C-025, SUMMARY/F2C-09] [status: mixed]: `elai bench seccodeplt --json` ≥40 fixtures under fixed seed → real evaluate_vibegate → write JSON artifact + `insecure_rate_measured` → SecurityRulesPanel surfaces results.

- FLOW **Tool-filter commitment** [atoms: F3-022..030] [status: planned]: `elai safety tool-filter --fixture` OR live researcher path → commitment + allow-set enforcement → deny non-member tools → ToolGateInspector shows commitment/quarantine/ASR. Full CaMeL DEFERRED after PDF/IPC ceiling decision.

- FLOW **Test-integrity red badge** [atoms: F4-E021..E023] [status: planned]: drive_task post-loop gate → failure/clean row same superstep → block FSM PhaseComplete → audit artifact → AuditTimelineWindow red badge. No env override on fixture path.

- FLOW **Staleness cascade** [atoms: F5-019..027] [status: planned]: post-reflect `decay_tick` (or CLI tick) → broadcast → demote dependents / flag Cosmos Stale → drawer shows quarantine. Kill: `ELAI_STALENESS_CASCADE_KILL=1`.

- FLOW **Skill promote S4** [atoms: F6-021..022, F6-049] [status: planned]: CLI `library promote --human-grant` → TrustPromoted event → panel shows S4. Dashboard S4 button DEFERRED.

- FLOW **Template ranking view** [atoms: F7-012..014] [status: Half-B]: CLI list/show or drawer `get_template_ranking` → utility + first-try side by side. Lattice path EXCLUDED.

- FLOW **Symbolic conflict screen** [atoms: F8-E022..023] [status: planned]: apply contradictory variant → screen before admission → pending candidate + BLAKE3 row → ReasoningDrawer badges. Operator-confirm semantics in acceptance.

- FLOW **Write-time dep gate** [atoms: F9-E010..016] [status: planned]: Edit/Write dep path → cache hit allow / miss RegistryCheck → NotFound|TooNew block + `write_time_slopsquat_blocked` → DepsSecurityPanel counts. CLI dep-check same event.

- FLOW **Security monotonicity REFINE** [atoms: F10-E011..026] [status: planned]: REFINE leg → class set-diff → Regressed → event + session log + drawer; Monotone → continue. Kill disable only.

- FLOW **Darwinian admit loop** [atoms: F11-E018..022] [status: wire-dark without dual flags]: flags+feature on → `elai darwin-loop` → guard rejections + LoopClosed → ReasoningDrawer Darwinian Admits.

- FLOW **Semantic contradiction adjudicate** [atoms: F12-E012..022] [status: planned]: memory tick → audit_conflicts then cross-key NLI → Contradiction≥0.85 closes older OR SurfaceToUser → drawer. Kill disable only. Cloud judge forbidden.

- FLOW **SSB archive on bump** [atoms: F13-E011..024] [status: planned, flag default-off]: open stale ssb with ENABLE → archive copy → drop/recreate → digest row → recovery drawer. Failure best-effort.

### Onboarding-relevant surfaces

- None named as first-run onboarding in F range. Closest: dashboard-visible session results (SUMMARY/FG-02); VibeGate/security drawers appear only after first blocked write or orchestrate baby-step with SQLi+AWS fixture.

### Notifications / background surfaces

- SSB rows as durable signals: VibeGateBlocked, skill_library_event, MemoryConflictDetected, darwinian_admits, blast-class security events, staleness broadcasts.
- Session log pipeline roles recorded on ship (F0-E044): context-compiler … memory-writer … committing — process not end-user UI.
- hook-executions.log T1 VibeGate match (F1-035).

### Decide-at-edit / open UI decisions

- F-6: MonitorWindow vs DrawerKey mount — decide after checking actual surface (F6-039). SkillsPanel vs SkillLibraryPanel name conflict (C-009).
- F-5: adopt >2.0 quarantine threshold for ELAI? (F5-036).
- F-7: lattice UI fully excluded; do not re-open as Half-B (F7-037).
- F-12: missing persistence for label/confidence/action/fingerprint vs drawer (C003/D-D023).
- F-13: recovery drawer Svelte filename unresolved; ENABLE default-off vs W-2 three dispositions unresolved (C005); no restore UX.
- F-3: `ELAI_CAMEL_TOOLFILTER_ENABLE` accepted values undefined (A-024).
- F-2: WARN vs refuse disk for missing-authz (A-010).
- Privacy: no redaction/retention rules for new SSB/dashboard artifacts (A-045, I-011).
- W-6 N/A only for truly headless with one-line justification (F0-E016, WD-016).

---

## G

### Surface inventory

- **TaintFlowDrawer** [type: drawer] [atoms: SUMMARY/G-1-024, G-001/DG001-G1-E030, G1-E034, G1-E041..043] [status: planned/BUILD]
  Drawer key `taint-flow`; `TaintFlowDrawer.svelte` + `commands_g1.rs::get_taint_flows`. Lists Kind / trust / block state. Entry: PreToolUse/taint path after Web/MCP untrusted text. Default-on; `ELAI_TAINT_PROPAGATE_DISABLE=1` only. Fixture: Web/MCP text requesting `git push --force` → no destructive effect; Trusted benign flow unblocked; drawer visible.

- **RraViewerPanel** [type: panel] [atoms: SUMMARY/G-2-015..033, G-002/DG002-G2-031..049] [status: planned/BUILD]
  RRA viewer for run artifact: seed, pins, gate hashes, guarantee level, root, replay verdict. Pair: `RraViewerPanel.svelte` + `commands_g2.rs::get_run_artifact`. CLI: `elai run --emit-rra` (force), `elai rra show <path>`, `elai replay --verify <path>` (`Verified` | `Divergence@event` | `Tampered`). Emission default-on at `run_async` exit; kill `ELAI_RRA_DISABLE` only; ENABLE forbidden. Artifact fields: schema_version, elai_seed, hardware_class, system_fingerprint, model_pin, action_stream, evidence_manifest, root_blake3, guarantee_level. Honest headline: reproducible action stream + divergence verification, not bit-identical token replay.

- **BlastRadiusDrawer** [type: drawer] [atoms: SUMMARY/G-3-028, G-001/DG001-G3-E027..034, G3-E039] [status: planned/BUILD]
  Key `blast-radius`; shows Kind + pre/post expansion command diff. Payload: kind, pre/post_expansion_command, target_or_host, worktree_root_blake3, trust_label, ts_ms. Default-on; `ELAI_BLAST_RADIUS_DISABLE`. Reachability: `dispatch_pre_tool_use` → check_fs_target/check_egress_host → Block + SSB. Windows AppContainer parity out of scope / unresearched.

- **McpRugPullDrawer** [type: drawer] [atoms: SUMMARY/G-4-003, G4-015, G-001/DG001-G4-E018..025] [status: planned/BUILD; ambiguities]
  Before/after description diff on `notifications/tools/list_changed`. Pair: `McpRugPullDrawer.svelte` (or existing MCP drawer) + `get_mcp_rug_pulls`. Default-on; `ELAI_MCP_REPIN_DISABLE`. No automatic `update_pin`. Ambiguity: payload stores hashes/names/diff count not ToolDescDiff.before/after yet UI requires real text diff (G4-A001). Operator-confirm path for pin update unspecified (G4-A002). Path conflict: tasks still target plugins MCP push vs corrected stdio funnel (G4-C003).

- **CockpitWindow / Review cockpit (G-5)** [type: window] [atoms: SUMMARY/G-5-006..025, G-001/DG001-G5-E002..049] [status: planned/BUILD Wave-2; human study non-blocking]
  **Heavy.** Reviewer cockpit reads RRA + SSB evidence; marks hunks vouched/un-vouched so reviewers skim vouched and focus unvouched. Pair: `CockpitWindow.svelte` / extended `ReviewWindow`; `get_review_surface`; window type `cockpit`. Renders per-hunk badges, vouched/unvouched bands, headline DSR. CLI: `elai review --cockpit [--session <id>] [--json]` → scores session SSB, prints bands + DSR; optionally reads RRA after G-2. Default-on; `ELAI_REVIEW_COCKPIT_DISABLE`. No LLM review-comment call; finding needs VibeGate/SecurityRule CWE evidence or stdout_hash with passed=false. Existing baseline: `ReviewWindow.svelte → get_file_changes → SsbViewer::file_changes`. Compare cockpit vs plain diff (study hypothesis, not DONE gate). DONE = cockpit + journal-auditable DSR; human study optional downstream.

- **ConfidenceDrawer** [type: drawer] [atoms: G-001/DG001-G7-E032, G7-E036, G7-E042] [status: planned/BUILD]
  Key `confidence`; score/components/abstention + origin/sink/quarantine rows via `get_claim_confidence` + `get_trifecta_quarantines`. Default-on; `ELAI_CLAIM_CONFIDENCE_DISABLE`. Evidence: low-score abstained + trifecta_quarantined; held change, no tool reach.

- **SafeDeltaPanel** [type: panel] [atoms: SUMMARY/G-6-029..033, G-002/DG002-G6-042..057] [status: planned/BUILD]
  `SafeDeltaPanel.svelte` + `get_safety_delta_latest`: per-suite/per-CWE deltas + 95% CI; “REPRODUCED not RIGHT” framing. CLI: `elai bench safedelta` (own sub-verb, not seccodeplt option). Matrix under ELAI_SEED; wrapped by G-2 RRA when present. Headline: fewer dynamically exploitable diffs per 100 tasks reaching reviewer.

- **AttestationDrawer** [type: drawer] [atoms: SUMMARY/G-8-015..026, G-002/DG002-G8-033..051] [status: planned/BUILD]
  Key `attestation`; subject, Merkle root, gate rollup, all-passed, signing method. CLI: `elai verify-attestation <commit>` offline → `verified`|`not-verified` without re-running ELAI. Emission default-on; `ELAI_ATTESTATION_DISABLE`. Commit trailer + sidecar required.

- **RedactionPanel** [type: panel] [atoms: G-002/DG002-G9-036..053] [status: planned/BUILD]
  Per-session counts/advisories only — never secret values. Core redaction default-on (`ELAI_REDACT_DISABLE` only); optional advisory may be wire-dark behind `ELAI_REDACT_ADVISORY_ENABLE`.

- **ModelDriftPanel** [type: panel] [atoms: SUMMARY/G-10-033..053, G-002/DG002-G10-039..066] [status: planned/BUILD]
  Identity, baseline/current bars, active signals, review queue, fingerprint-confirmed vs behavioral badge. CLI: `elai model-drift baseline|check|show` (+ `--json`). Default-on detect; auto-pin remains off; kill `ELAI_MODEL_DRIFT_DISABLE`. Cloud never fabricates FingerprintConfirmed.

- **G product spine (non-UI but user-visible outcomes)** [atoms: G-001/DG001-G0-E002, E041]: reviewer cockpit + re-executable run artifact + anti-gameable safety-delta measurement; composition G-2 RRA + G-6 SAFEDELTA.

### User flows

- FLOW **Taint block untrusted tool** [G1-E022, E041..043] [planned]: read Web/MCP → Untrusted provenance → before git push/Write/Bash `allow_tool_with_origin` → block force-push class → TaintFlowDrawer rows; Trusted file in `.elai_cc/` unblocked.

- FLOW **Emit and verify RRA** [G2-028..049] [planned]: `elai run` exit → emit_run_artifact (.rra) → `elai rra show` OR dashboard RraViewerPanel → `elai replay --verify` → Verified/Divergence/Tampered. Suppress with `ELAI_RRA_DISABLE=1`.

- FLOW **Blast-radius PreToolUse** [G3-E036..039] [planned]: tool with FS escape/egress → pre+post expansion recorded → block side effect → BlastRadiusDrawer pre/post diff.

- FLOW **MCP schema rug-pull** [G4-E018..032] [planned]: tools/list_changed → re-verify pins → McpRugPullBlocked + drawer before/after → no auto update_pin → [unclear] operator confirm to re-pin.

- FLOW **Cockpit review** [G5-E023..037] [planned]: orchestrate mixed vouched/unvouched fixture → `elai review --cockpit --session` OR open CockpitWindow → badges/bands/DSR from SSB+optional RRA → reviewer skims vouched, attends unvouched. DSR must recompute under ELAI_SEED.

- FLOW **Claim confidence abstain** [G7-E042] [planned]: low-evidence claim path → ClaimConfidenceScored abstained + trifecta quarantine → ConfidenceDrawer; change held.

- FLOW **SafeDelta bench** [G6-052..054] [planned]: `elai bench safedelta --suites seccodeplt --seed 42` → SafetyDeltaMeasured + signed RRA manifest → panel suite/CWE/CI.

- FLOW **Attest commit** [G8-045..047] [planned]: commit path emits attestation → trailer+sidecar → `elai verify-attestation` offline; tampered sidecar → not-verified; drawer rollup.

- FLOW **Redact secrets** [G9-049..053] [planned]: hook log tokens only; purge expired; RedactionPanel counts; kill suppress.

- FLOW **Model drift canary** [G10-053..061] [planned]: baseline under seed → provider change → check → ModelDriftSuspected + panel bars/badge; clean recheck NoDrift.

### Onboarding-relevant surfaces

- RRA first-run: first completed `elai run` should produce viewable .rra (G-2 done fixture).
- Cockpit: reviewer-facing trust spine after G-2/G-5 ship; not first-agent-onboarding.

### Notifications / background surfaces

- G-4: MCP `notifications/tools/list_changed` handler (protocol notification, not OS toast).
- SSB evidence stream under ELAI_SEED for all G metrics (GD-2); wire-dark if not recomputable.
- Attestation emission at run-end/commit; redaction purge of expired hook logs.

### Decide-at-edit / open UI decisions

- G-4: where before/after description text is stored for drawer; operator-confirm command/UI for pin update (A001/A002).
- G-5: CockpitWindow vs extended ReviewWindow naming; `--run-trigger` deprecated no-op vs acceptance still requiring flag (C003/C004).
- G-7: acceptance text still mentions untrusted-span quarantine vs corrected block-only semantics (C007).
- W-6 dashboard-smoker required for all G UI pairs or N/A with justification (G0-E012).

---

## H

### Surface inventory

#### Efficiency / cost (H-1..H-9)

- **CloudPanel + TokenTrackerPanel (H-9 cost cockpit)** [type: panel] [atoms: H-001/H9-002..029, SUMMARY/H9-003..014, H-001/HG-014] [status: planned BUILD; currently DARK zeros]
  **Heavy.** Mounted CloudPanel polls ~5s but shows all zeros: SpendAccumulator/CloudSpend exist but cloud constructors never attach SSB writer. H-9 activates writer attachment + non-test cost feeder + `TokenTrackerPanel` (~280-line Svelte / 60-line Tauri): input/output/cached tokens, cache-expiry timer, $/resolved-task, H-1 hit rate; reads live CloudSpend; real redaction count (not hardcoded zero). Until G-9: say redaction not wired, not “zero secrets.” Live trigger: real cloud-mode task → open CloudPanel → nonzero token/cost/hit-rate. Values recompute under ELAI_SEED. No new SSB action type. H-1/H-2 have no independent frontend; H-9 owns hit-rate/breakpoint tiles and E2E.

- **H-1/H-2 cache path** [type: background + H-9 tile] [atoms: H-004/H1-E004..032, H2-E022..026] [status: planned activation; monitor caller-dark]
  Marker stamping/monitor default-on (kill unnamed [A]); old `ELAI_CRITIC_OF_DESIGN_ENABLE` cannot stay terminal. Feed H-9 only.

- **H-3 generation mode / streaming** [type: CloudPanel pair] [atoms: H-002/H03-004..035] [status: FIM wire-dark]
  `GenerationModeSelected` + `StreamingEarlyTerminated` mirrored in TS/CloudPanel. FIM needs `ELAI_FOCUS_TUNNEL_EXECUTE=1` + mode Fim + target_file + edit_region. Flag must not stay unregistered default-off terminal.

- **H-5 batch lifecycle** [type: optional tile DEFERRED] [atoms: H-002/H05-016..030, SUMMARY/H5-012] [status: planned; batch-status tile optional]
  Cosmos tick → batch client → lifecycle rows; dashboard Cosmos refresh is a production route but user-waited refresh must stay synchronous — need autonomous/user-waited discriminator [unclear].

- **H-6 compaction ladder panel** [type: panel reuse] [atoms: H-006/H006-H6-004..020] [status: ladder default-off today; promote planned]
  Frontend: existing E-25 ladder-state via `get_compaction_ladder_state`. Promote off `ELAI_E25_COMPACTION_LADDER_ENABLE` default-off → default-on/kill-only. Wire `get_relevant_facts` non-test caller (memory path dark today).

- **H-7 route-decision drawer** [type: drawer] [atoms: H-006/H006-H7-012..019] [status: planned; routing observational/default-off today]
  Three Kind arms; promote `ELAI_ROUTE_DECISION_SSB_ENABLE` to default-on kill-switch semantics.

- **H-8 oracle cache** [type: CLI signal] [atoms: H-006/H006-H8-003..013] [status: planned persist]
  No dashboard pair; cache-stats line in `/research` is human signal. Hit/Miss events.

#### Canvas workbench migration (H-10..H-16) — full detail

- **Canvas.svelte / WindowHost canvas shell (H-10)** [type: window] [atoms: H-005/H005-10-002..033, SUMMARY/H10-001..016] [status: planned BUILD Wave-2 B0]
  New 2D generative canvas WindowHost type for code topology + dead-code islands. Correction: symbol call-graph already reaches GraphWindow via `build_symbol_view → get_graph_view_data(mode=symbol)`; PageRank producer-dark (`get_pagerank_top` empty until materialize on `elai symbol_graph build`). Canvas mounts renderer (Cytoscape WebGL provisional → canvas fallback), pan/zoom, viewport culling. Evidence: showFps HUD ≥60, non-empty PageRank panel, orphan/SCC/dead fields honest. No new SSB type. Downstream: H-11/H-12 extend; H-16 after full default-canvas release.

- **Live overlay + FileExplorer (H-11)** [type: overlay + column] [atoms: H-001/H11-002..031, SUMMARY/H11-001..009] [status: planned BUILD]
  Overlay on active Canvas mode: active-node halo, traversed edges dash-flow, follow-agent attention-centering (default on). Ghost → real node on index catch-up or fade 30s to dropped marker. Bidirectional VS Code-style explorer over path-keyed focus bus: select Canvas file ↔ tree highlight/center. AppShell: new persisted resizable column outside 48px icon rail (split clamp 0.2–0.8, 250ms persist). FS: project-root scoped only (never whole-disk); tauri-plugin-fs conditional if Svelte reads direct. Activate UI-dark `list_projects`/`create_project`. Events: `CanvasActivityEvent` kinds/payload; store `canvasActivity.svelte.ts`. Reachability: Canvas→overlay; AppShell→FileExplorer→FS commands; live agent→event rows→marker.

- **PlanMode / ResearchMode / CanvasModeBar (H-12)** [type: frames + chrome] [atoms: H-003/DH003-H12-001..053, SUMMARY/H12-010..013] [status: planned BUILD]
  Port Orbit PLAN + RESEARCH to flat clipping frames on one Canvas. `canvas.svelte.ts` plain mode flag `{code, plan, research, arch, design}` — not FSM substate. Shared chrome: CanvasModeBar + zoom/search/inspector. Canvas default via workspace flag (`workspace.svelte.ts` primary currently `activity`); Orbit remains tab until H-16. **Atomic single SSB consumer**: only active window owns subscription; inactive suspended not merely hidden. Surfaces: `PlanMode.svelte`, `ResearchMode.svelte`, `CanvasModeBar.svelte`. No new Tauri command; consume existing stores. Live: real plan/research run, flag on, switch Canvas/Orbit tabs → one subscription owner, PLAN active-node light from plan store.

- **Canvas tools / CanvasPanel (H-13)** [type: tools + panel] [atoms: H-007/H007-13-002..038, SUMMARY/H13-001..009] [status: planned BUILD]
  Model-driven ops: place/connect/update/delete/fork/render_diagram/screenshot tools; `CanvasDispatcher` validates namespace vs mode, sanitizes, mutates store, journals `CanvasOpApplied` (ten kinds), live stream. Native screenshot capture must be built. Default-on from live model-call path. Mermaid import bridged into op model. Forbidden: second media validator, second canvas transport, FSM mode substate.

- **DesignPageNode (H-14)** [type: canvas mode node] [atoms: H-004/H14-E013..037] [status: planned after H-13]
  DESIGN frame: controlled iframe + real app pages on navigation graph; DevPreviewProxy; recast_writeback CLI/IPC. Default-on behind mode flag (flag ID [unclear]). a11y risk on generated UI.

- **Spawn-view reply channel (H-15)** [type: board/view] [atoms: H-007/H007-15-009..038, SUMMARY/H15-008..016] [status: planned]
  Model may spawn interactive board views; Select serializes Focused/Blurry/clustered context + **taint flag at handoff** (never silent trusted instruction). `spawn_view` only above routing-confidence threshold; default medium text; suppress-canvas hatch. Events: ViewSpawned, UiAction, StateCheckpoint. Kill/reopen dashboard reconstructs tab from SSB. ToolCall consent separate from plan approval (H-I-007). Publish/share-canvas DEFERRED behind G-8 attestation.

- **Orbit retirement (H-16)** [type: DELETE migration] [atoms: H-007/H007-16-001..023, SUMMARY/H16-001..010] [status: PLANNED Wave-2 last; not parallel-safe]
  Delete only after Canvas-as-default full release without regressions. Remove Three/Troika/StarCanvas/OrbitCanvas/Starfield/NodeMandala/scene modules; keep `orbit/lcg.ts` as canvas seed source; flip workspace primary panel activity→canvas; remove three deps. Evidence is absence (green build, zero grep). No new SSB.

#### Trust cockpit extensions (H-17..H-29) — surfaces only

- **Trust-cockpit drawers (shared)** [type: drawer/cockpit tiles] [status: planned BUILD default-on kill-only unless noted]
  - H-17: trial ledger N surface in H-18/H-29 cockpit; optional count strip; no isolated FE required (`H006-H17-019`).
  - H-18: DSR, PBO, N drawer; fail DSR-at-N evidence (`H007-18-023`, SUMMARY/H18-012).
  - H-19: remaining/total holdout gauge optional; W-6 min shows `budget_exhausted` refusal (`DH003-H19-030`); DONE UI optional-vs-required tension (A-014).
  - H-20: leakage drawer shows fit site + reachability path (`H20-020`, SUMMARY/H20-012).
  - H-21: leakage drawer first changing column/index (`H005-21-015`).
  - H-22: trust cockpit null band + surviving-edge flag (`H22-E029`).
  - H-24: parity drawer mode findings + replay/golden hashes (`H24-E020`, SUMMARY/H24-010).
  - H-25: reviewer cockpit reported vs re-derived value + provenance; compose G-5 (`H005-25-018`).
  - H-26: divergence/falsification drawer (`H007-26-020`, SUMMARY/H26-013).
  - H-27: rules/meta cockpit mutant-kill report, escaped counter, blocked promotion (`H27-E021`).
  - H-28: self-mod cockpit harness hash, score-delta history, counter (`H006-H28-015`); H-28W activation flag `ELAI_H28_REWARD_GATE_ENABLE` default OFF wire-dark allowlist.
  - H-29: deployment cockpit per-axis Warning/Drift/Halt; vendor-schema halt visible (`H006-H29-020`).
  - H-39: cost/trust cockpit abstention rate + adjusted score (`H007-39-019`).
  - H-40: iso-budget panel unique-catch/decorrelation columns + retention explanation (`H40-011`).
  - H-41: `elai checker-gaps` or dashboard gap backlog rank (`H41-011`).
  - H-42: Reasoning drawer subtask depth, scaled budget, failure class (`H007-42-018`).
  - H-43..H-48: DGM/archive/trust drawers (fitness, recall, worst-case, CMP, firewall counts, bandit posteriors) behind darwinian feature/env.
  - H-52: G-5 cockpit Anchored inline vs un-vouched fallback band; `ELAI_REVIEW_ANCHOR_DISABLE` (`H005-52-019..023`).

#### Memory / UX (H-30..H-37, H-33 heavy)

- **MemoryFactsDrawer governance (H-31/H-32/H-33)** [type: drawer] [atoms: H-003/DH003-H33-*, H005-31-*, DH003-H32-*, SUMMARY/H33-*] [status: mixed planned; CLI dispatch stub today]
  **Heavy H-33.** Extends F-12 drawer — no standalone MemoryHub (DEFERRED). Today: `get_memory_facts` → list_current limit 200; active rows only; no supersession history. Add: “include superseded” toggle via `get_memory_facts_with_history`; per-row forget/pin → `memory_forget` / `memory_pin`; “superseded by” affordance (H-31). CLI: `elai memory list|inspect|forget|pin|unpin|provenance` (de-stub main dispatch); `--json`; `/continue` resume lists influencing decisions + flags superseded before action (no model prompt inject). Governance reads default-on; pin enforcement default-off `ELAI_MEMORY_PIN_ENABLE`. H-32 badges: Valid / Stale-excluded / TTL-expiring. H-31 Memory slot default-off `ELAI_USER_MEMORY_ENABLE` + allowlist; compile-time empty Memory for auditor/critic/adversary. Conflicts: CLI list active+superseded vs list_current (C-008); history API for CLI [A-025].

- **H-35 Rewind** [type: CLI + CommandWindow control] [atoms: H-001/H35-*, SUMMARY/H35-*] [status: planned BUILD default-on]
  `elai rewind <target>` with `--turn` / `--checkpoint` / scope / summarize; CommandWindow per-turn “Rewind to here” → `rewind_session` Tauri; shares dispatcher with CLI. Default-on opt-out `ELAI_REWIND_DISABLE`. No prompt inject; summarize shrinks window only. Adjacent UI: RraViewerPanel. Post-rewind build_rra/verify_rra must Verified under seed. CLI grammar mutual exclusion [unclear AC-016].

- **H-36 completion / needs-input notifications** [type: toast + OS + terminal] [atoms: H-006/H006-H36-*, SUMMARY/H36-*] [status: planned; intervention no-op today]
  **Heavy.** Completion/auditing toasts live; intervention at NotificationManager.svelte:114-118 is explicit no-op → replace by reading needs_input rows via new Tauri command → needs-input toast. Also: terminal OSC9; native OS via notify-send/osascript/PowerShell best-effort. `ELAI_NOTIFY_ENABLE` default-off for OS spawn (allowlist+smoke); terminal escape + SSB row unconditional. Fixed text never model-generated. Conflict: “opt-out” prose vs default-off enable (C-012). Depends H-35 for inspect/rewind recovery path (I-008).

- **H-37 PlanApprovalCard** [type: card] [atoms: H-007/H007-37-*, SUMMARY/H37-*] [status: planned; gate presence-only today]
  **Heavy.** Dashboard approval card: pending plan + three decision buttons (interactive) or recorded autonomous decision. Orchestrator after compliance, before FSM transition. Pattern-2 `plan_decision_event`. Flag default-off `ELAI_PLAN_APPROVAL_GATE_ENABLE` + allowlist (naming vs kill_switch type conflict C-CFG). Reject → no TESTING_FIRST transition. Ambiguity: async dashboard input correlation to present_gate vs CLI stdin gate (A-FLOW). GateMode default Interactive; hybrid maps Interactive; `ELAI_DISABLE_AUTONOMOUS_GATE` existing autonomous kill.

- **H-38 stop cost confirm** [type: CLI confirm; CloudPanel reuse] [atoms: H-002/H38-*, SUMMARY/H38-*] [status: planned]
  Safety confirm unconditional; `ELAI_STOP_COST_CONFIRM_ENABLE` gates beacon only. Tier-1: no new dashboard modal; reuses CloudSpend read. Modal/batch-stop DEFERRED.

- **H-34 MCP schema budget** [type: CLI bench; FE DEFERRED] [atoms: H-007/H007-34-*, SUMMARY/H34-*] [status: planned default-off experiment]
  `elai bench mcp-schema-budget`; schema-budget tile DEFERRED. Flag `ELAI_MCP_DEFER_SCHEMA_ENABLE` allowlist.

- **H-49 time travel / BranchTree** [type: CLI + dashboard timeline] [atoms: H-004/H49-*, SUMMARY/H49-022..026] [status: planned default-on]
  CLI: `elai undo|redo|history|timeline|jump` with grain/scope/steps/dry-run. Dashboard: extend `BranchTree.svelte` timeline, file-manifest diffs, Redo, multi-child, Files-Only vs Files+Conversation. Virtual-window transcript (replace 50-cap), Jump to start. Capture default-on `ELAI_TIMETRAVEL_DISABLE` opt-out; replaces E-25 default-off scrape. Tauri `timetravel_undo/redo/jump` + paginated history. Branch ambiguity prompt / headless redo [unclear A004].

- **H-50 context gauge** [type: gauge] [atoms: H-007/H007-50-*, SUMMARY/H50-*] [status: planned default-on]
  Per-regime context-budget gauge + “unloaded N items (reloadable)”. `effective_context_window(backend)` single truth; respect `ELAI_LOCAL_MAX_CONTEXT`; kill `ELAI_CONTEXT_POLICY_DISABLE`.

- **H-51 handoff digest** [type: auto-detect /continue] [atoms: H-006/H006-H51-*] [status: planned]
  Detection default-on; auto-continuation default-off; kill `ELAI_HANDOFF_AUTORESUME_DISABLE` [scope unclear A-015].

- **H-53 research citation UI** [type: research UI] [atoms: H-004/H53-*, SUMMARY/H53-*] [status: planned default-on]
  Every `[N]` pre-bound anchor + source tier; unsupported spans explicit refusal. Kill `ELAI_CITATION_PREBIND_DISABLE`. Degraded FreeText vs normal UI distinction [unclear].

- **H-54 verification drawer** [type: drawer] [atoms: H-002/H54-021, SUMMARY/H54-020] [status: planned default-on]
  Blocked escape node + obligation ID; registered exception owner/expiry. Kill `ELAI_CLOSURE_SCAN_DISABLE`.

### User flows

- FLOW **Canvas Wave-2 migration** [H-10→16] [planned]: H-10 shell+PageRank → H-11 overlay+explorer → H-12 PLAN/RESEARCH frames + Canvas default flag + single-consumer handoff → H-13 model ops live stream → H-15 spawn_view/Select+taint → H-14 DESIGN pages → full-release proof → H-16 delete orbit/three.

- FLOW **Model drives canvas** [H-13] [planned]: model tool call → CanvasDispatcher namespace check → reject wrong mode OR sanitize/apply → CanvasOpApplied row → CanvasPanel live draw. Screenshot via native capture.

- FLOW **Select on tainted node** [H-15] [planned]: user Select → serialize state + provenance flag → next model turn; never trust as instruction. Kill/reopen → reconstruct from journal.

- FLOW **Cost cockpit light-up** [H-9] [planned]: cloud task with writer attached → CloudSpend rows → CloudPanel+TokenTracker nonzero; pre-state zeros.

- FLOW **Memory governance** [H-33] [planned]: CLI/dashboard list/history → pin/forget → events actor cli|dashboard → pin flag on: decay skips pinned. `/continue` lists decisions pre-action.

- FLOW **Rewind session** [H-35] [planned]: CommandWindow “Rewind to here” or CLI target → SessionRewound → optional code restore → RRA verify Verified.

- FLOW **Needs-input notify** [H-36] [planned]: RequestHuman → SSB needs_input → terminal OSC9 always; OS if flag on; dashboard toast from rows (no longer no-op).

- FLOW **Plan approval** [H-37] [planned]: flag on → orchestrator edge → PlanApprovalCard or CLI stdin → Approve/Redirect/Reject → decision row; Reject blocks TESTING_FIRST.

- FLOW **Review anchor in cockpit** [H-52] [planned]: `elai review --cockpit` → anchor before G-5 score → Anchored badge or un-vouched band.

- FLOW **Time travel undo** [H-49] [planned]: PostToolUse capture CAS → undo/redo/jump CLI or BranchTree → restore files and/or conversation; branch prompt on ambiguity.

### Onboarding-relevant surfaces

- Canvas-as-default workspace flag (H-12): first open after flag shows Canvas not Activity/Orbit.
- FileExplorer column (H-11): project list/create activation is project onboarding.
- H-51 auto-detected handoff digest for session continuation (human-confirmed).
- H-33 `/continue` decision list before action.
- CloudPanel zeros today = false empty state; H-9 must show honest empty when no cloud rows.

### Notifications / background surfaces

- H-36: OS notify, OSC9, dashboard completion toast (live), needs-input toast (to activate).
- CanvasActivityEvent / CanvasOpApplied / CanvasViewEvent / SessionRewound / plan_decision_event / MemoryGovernanceEvent / EditHistoryEvent / TrialRegistered / trust-gate rows.
- Compaction ladder tiers currently estimate-only without content mutation until H-6 promotion (HG-029).

### Decide-at-edit / open UI decisions

- H-12: workspace flag name, persistence, initial default on/off (DH003-A-003); research store path placeholder.
- H-11: tauri-plugin-fs only if Svelte direct reads (conditional) vs files-touched unconditional (AC-013).
- H-16: unresolved galaxy-toggle choice (DELTA-007-006).
- H-19: optional gauge vs required dashboard visibility for DONE (A-014); kill switch unnamed.
- H-31: how superseded history reaches drawer if list_current only (H005-31-050).
- H-33: pin ordering, forget atomicity, flag-disabled pin UI semantics (A-026..029); MemoryHub deferred no owner/date.
- H-35: CLI target form mutual exclusion (AC-016); recover_stubs PROD-WIRED vs zero callers conflict (AC-002).
- H-36: ENABLE vs opt-out terminology (C-012).
- H-37: dashboard async decision transport; enable flag typed as kill_switch (C-CFG/A-FLOW).
- H-49: last-followed redo persistence and headless branch behavior (A004).
- H-50: dead-trailing eligibility, recent-window N, summary algorithm unspecified (A-DET).
- H-51: what HANDOFF_AUTORESUME_DISABLE actually disables (A-015).
- H-53: degraded grammar FreeText UI labeling (A002).
- Multiple trust gates: kill-switch IDs unnamed (H-1, H-2, H-24, H-27, H-39, H-42).
- W-6 N/A: H-17 optional strip; H-30/H-34 no isolated FE; H-8 no dashboard pair — document N/A.

---

## Cross-family composition (source-stated)

- F security blocks feed G-5 vouch evidence (VibeGate/SecurityRule CWE or failed stdout_hash).
- G-2 RRA + G-5 cockpit + G-8 attestation + G-6 measurement = signed offline trust spine; H-25/H-52 compose into same reviewer cockpit.
- H-33 extends F-12 MemoryFactsDrawer; F-12 semantic contradictions and H-32 validity badges share drawer.
- H-15 taint at Select as strong as live G-1; replay-only taint_text = reduced tier (H007-I-002).
- Canvas events split three ways: H-11 overlay, H-13 ops, H-15 view lifecycle — none subsumes another (H0-118).
- Dual human controls: canvas ToolCall consent (H-15) and plan approval (H-37) independent (H007-I-007).

---

*End digest F+G+H. Family I begins source line 57358 — out of range.*
