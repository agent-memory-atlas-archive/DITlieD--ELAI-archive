# Surface digest: SYNERGY + PHASE + D + B (lines 51955-53390)

## SYNERGY (UI-relevant only)

### Hub systems that surface in UI
- **SSB** (phase-3B journal): session event bus; feeds almost every drawer/panel that reads live state (StatusBar, orbit, Monitor, Plan, Reasoning, D-26/D-27 archive/routing panels). Schema bumps per family; single-writer SQLite-WAL.
- **B-37 RuntimeAdapter**: multi-backend routing; feeds RuntimePanel/Settings, `/runtime` CLI, per-role health g/y/r, fallback chains (consumers C/D/E/I/J/K/L/N/P).
- **G-2 RRA / replay-verify**: proof spine; UI when verification/attestation surfaces appear (not a primary window name here).
- **F-4 cant-lie / PROVEN**: case gate; verdicts roll into attestation and reuse UIs; do not rebuild as later monitor.
- **F-1 VibeGate**: PreToolUse external review; shared secret patterns with G-9 redaction.
- **F-5 quarantine**: flaky isolation; feeds quarantine/sycophancy flags and L prune paths.
- **C-28 / C-29**: mutation + intent gates; honesty front door before curator/do-test.
- **J-16 blast-radius**: adjacent to B-29/C-24 blast-radius UI; named substrate for K/L.
- **B-40 framing / ResearchReport**: IdeaFramingWindow, research types (consumers C/E/I/J/O/Q).
- **B-29 blast-radius / decomposition UI**: KnowledgeDrawer hop bands, impact tools (closed by C-24).
- **D-16 refusal/calibration**: RefusalRateGauge, conformal surfaces.
- **H-9 spend ledger**: Monitor spend gauge / CloudPanel cost displays.
- **I-25 post-wake router / intervention-card inbox**: intervention cards (Monitor/Command).
- **I-11 BeliefState**: PARTIAL; advisor/condense paths (CF-039).
- **O-2 TaskSpec + Cron/SsbEvent**: dashboard inbox subset Tier-1 (CF-031).
- **N-6 ONE lattice**: privacy/admission; P ports privacy UI constraints.
- **D-13 adversary principle**: independence; rejects same-family PPM as adversary (CF-042).
- **Dashboard chain (synergy §5)**: B-* gaps → C-* closes → D-26/D-27 reuse C-3/C-6/C-7 + SSB → H/I/J drawers ride C IPC + B-37.

### Theme F "UI / UX conventions" invariants (every INV, dense)
- **INV-F01 — Dashboard UI: Svelte 5 runes only (V2).** `$state`/`$derived`/`$effect`/`$bindable` only; no legacy `$:`; Tauri 2 events. Families: B owner; C/J on pairs. Anchors: `SUMMARY/SUM-0004`, `SUM-0402`.
- **INV-F02 — Activity is an orbit; workspace is the shell.** Orbit not text feed; outlined buttons over solid CTAs. Families: B. Anchor: `SUMMARY/SUM-0403`.
- **INV-F03 — Window vs cosmos data locality.** Window state = localStorage only; Cosmos user-global outside project repos (`~/.elai/cosmos/`). Families: B; E/C stages. Anchors: `SUMMARY/SUM-0404`; `B42-E04`.
- **INV-F04 — Closed enums / closed stage vocabularies.** No free-text classifier into prompts; out-of-enum fails done-gate; canvas has no independent runtime. Families: O, J, P, N. Anchors: SUMMARY O.X.E03–E06; `J-005/J61-E003`.
- **INV-F05 — W-4 frontend pair for dashboard-surfaceable SSB.** Human-visible SSB action needs Svelte/Tauri pair + cross_lang_diff (or N/A + W-6 justification). Families: F, G, J, O, B, C, I. Anchors: `G-001/DG001-G0-E010`; `F-001/DF001-F0-E013`.
- **INV-F06 — Schema golden-lock and named bumps (W-9).** New action types pass golden lock; Pattern-2 Kind preferred; named bump; re-read baseline live (never hard-code wave numbers). Families: all 17; doctrine F/G/C/I/J/N/O/Q/M. Anchors: `G-001/DG001-G0-E015`; `Q-001/Q0-E020`; `M-001/M6-E021`.

### Other invariants that constrain UI
- **INV-A01..A06**: off-agent evidence required; missing evidence FAIL not SKIP; no self-certify; claims measured/tagged; CANT-LIE; observed effect needs journal row. UI must not show green on empty evidence.
- **INV-B01**: append-only SSB (no UPDATE/DELETE) — drawers show history, never mutate sealed rows.
- **INV-B02**: `ELAI_SEED` / SeededClock for deterministic displays (SeededClock tick starts at 0 regardless of seed).
- **INV-B03 GD-2**: metrics recomputable from journal under seed — budget/spend numbers must be journal-backed.
- **INV-B05**: BLAKE3-only content hashing (supersedes blake2).
- **INV-C01**: fail-closed default; **INV-C07 wire-dark honesty**: UI must not imply live connectivity when wire-dark.
- **INV-C02 W-10**: Tier-1 in-process fail-closed vs Tier-2 gated (only Tier-1 is done-gate).
- **INV-C03 W-2**: kill-switch DISABLE only (no terminal default-off ENABLE) except named dual opt-in.
- **INV-D01**: hard spend ceilings fail closed — Monitor/CloudPanel warn/block.
- **INV-D02**: context/reasoning budgets first-class and observable (SSB/StatusBar); exhaustion → compaction ladder, not silent overflow.
- **INV-E01**: machine memory vs personal vault disjoint; personal topics out of project/cloud prompts.
- **INV-G02**: visibility filter hides harness internals from agents (`hidden_paths`: models, rules DB, harness, checkpoints, SSB internals).

### CF-nnn / RA-nnn that touch user-facing surfaces
- **CF-010 B-31 indexing**: DONE (resumable) — progress panel, lock banners live. Winner: DONE.
- **CF-011 B-38 ambiguity**: SHIPPED schema 42→43 — GatesPanel/OrchestrationDrawer events.
- **CF-013 B-39 REFINE_CAP**: canonical 3 (not 2).
- **CF-014 B-12 shell IA**: single Workspace + IconRail; Tier-0 five-icon retired.
- **CF-015 D-28 adversarial farm**: shipping-pass host live; training deferred.
- **CF-016 D-29 hierarchical BAI**: Path A host Rust shipped; Gibbs/RunPod DEFERRED — mixed.
- **CF-017 D-30 calibration bootstrap**: Rust dispatch live; corpus/data DEFERRED — partial.
- **CF-018 F-10 self-mod**: wire-dark until F-11 — UI must not claim full loop live.
- **CF-025 M-1 license**: CLI + ExpiredGrace banner + B-43 activation; **no SettingsWindow** (deep wins over summary Settings pairing).
- **CF-030 O-4 canvas**: Tier-1 read-only inspection (not node editing).
- **CF-031 O-2 stages**: schema eight stages vs Tier-1 Fetch/Classify/Gate/Digest + DashboardInbox only.
- **CF-033 TokenUsage fields**: design 4 vs shim 2 → **RA-003 UNRESOLVED**.
- **CF-040 D-1/D-8 archive hash**: two-part vs three-part → **RA-004 UNRESOLVED** (ArchiveVerify surfaces).
- **CF-041 D-7 canary block rule**: non-target-only vs ANY-family → **RA-005 UNRESOLVED** (Canary report Allow/Block).
- **CF-042 D-2 PPM vs D-13**: same-family Qwen Math≠valid adversary; D-13 reject wins.
- **CF-043 D-13 ModelFamily predicate**: behavior clear, impl shape open → **RA-006**.
- **RA-001 P-21**: Tier-2 defer vs wiring still requires skipBackfill/redaction/posture — recon before implement.
- **RA-002 J-52 vs E-52**: default-on daemon vs dark enable — unresolved enable semantics.
- **RA-007 P-12 frame hash**: sha256 vs blake3 conflict same unit.
- **RA-008 N-8**: five views vs Kind enum four (Traceability omit).
- **RA-009 Q-11 vs Q-14**: novelty canary ownership split.
- **RA-010 G unit DONE**: seam anchors shipped, initiative DONE word open.

---

## PHASE

### Surface inventory
- **Slash CLI core** [type: CLI] [atoms: P0-E055, MASTER-E021] [status: planned/BUILD phase-0]: `/task`, `/config`, `/continue`, `/explore`, `/status`; bare messages fallback to `/task`; unknown command error. KEEP candidates (P0-E026): config, add-dir, api-key, stop, retry, paste, screenshot, search, system-prompt, temperature, max-tokens. STRIP claw-code set (P0-E027): rewind, summary, bughunter, ultraplan, desktop, ide, tag, brief, advisor, stickers, insights, thinkback, release-notes. Conflict: 28+ claw-code commands vs elai_cc (P0-E012).
- **CONFIGURING mode** [type: CLI] [atoms: P3A-E008] [status: planned]: via `/config` or `/workflow`; config_paths_only tools; no evidence gate; →IDLE on completion.
- **INTERACTIVE gate (CLI)** [type: CLI] [atoms: P3A-E024, P3J-E045] [status: planned]: structured EvidenceSummary present_gate (not prose); CliInteractiveGate prints EvidenceSummary+RiskLevel; prompt Approve/Reject/Modify/Redirect; LOW risk auto-advances.
- **Native dashboard (4K)** [type: window] [atoms: P4K-E002, P4K-E026, MASTER-E403/404] [status: planned BUILD]: structured control surface not chatbot; `elai --dashboard` focuses existing window if running. Surfaces: project selector, task input, phase ribbon, intervention panel, activity feed, reasoning chain, commit history.
- **Visual system 4K** [type: page] [atoms: P4K-E006, P4K-E027] [status: planned]: teal `#1B7A6E` on charcoal `#0f1318`; JetBrains Mono; density over whitespace. CSS tokens: bg-void/base/surface/hover/input; teal/purple scales; semantic pass/warn/fail/info; mono xs–xl.
- **ProjectSelector** [type: page/window] [atoms: P4K-E029–E034] [status: planned]: logo + subtitle "a deterministic development engine"; + New Project (name+parent picker, create_dir_all, template+task without leaving app); Open Existing (native folder dialog); Recent max 10 (name/path/last opened) from elai_cc.toml [projects] or ~/.elai_cc/projects.json; name alnum-_; inline red errors; templates Svelte/React/Astro/Vanilla from templates/web/manifest.toml.
- **Task input** [type: panel] [atoms: P4K-E035] [status: planned]: header, placeholder, Start purple, Enter submit; `submit_task`.
- **Workspace shell** [type: window] [atoms: P4K-E037–E045] [status: planned]: plan tree left 320px + activity feed right; draggable divider min 280px; double-click 50/50. Status bar 24px: project + FSM state. Plan tree phases/steps/baby-steps; `-->` active; done/active/pending styles; expand active; elapsed times. Feed types: [reasoning] teal-ghost collapsed 2-3 lines; [edit] path+lines +N -M; [validate] checkmark/x; [test] pass/fail counts; [preview] teal link → system browser. Step headers; auto-scroll; Jump to latest pill when scrolled up.
- **Diff viewer (ApproveChange)** [type: panel] [atoms: P4K-E049–E051, P4K-A003] [status: planned; open: replace feed vs modal]: unified teal add / fail remove / sticky header; Accept / Decline / Retry+note; per-hunk Accept/Decline on hover; multi-file vertical list.
- **Tauri command set 4K** [type: CLI/command] [atoms: P4K-E019–E024, P4K-E053]: `get_fsm_state`, `get_phase_progress`, `get_recent_actions(limit,offset)`, `get_pending_interventions`, `get_file_changes(session_id)`, `submit_intervention_response(request_id,chosen,note)`, `get_action_detail(action_id)`.
- **RuntimePanel / Settings** [type: panel|window] [atoms: P4M-E023–E028, P4M-A002] [status: planned; open placement]: CLI `/runtime show|set <role> <backend>|fallback <role> <b1>,<b2>`; per-role backend, chain, live health g/y/r, Test backend button. Placement RuntimePanel vs SettingsWindow open.
- **SurgicalDrawer** [type: drawer] [atoms: P4DS-E002, P4DS-E011, P4DS-A002] [status: planned; layout open]: `/task --surgical "<prompt>"` → surgical_plan_created + ComplexityClass + AffectedSet; full field layout not specified.
- **Mode / model CLI** [type: CLI] [atoms: P4G-E012, P3J-E038/E042, PIDX-E013/E024] [status: mixed]: `/mode autonomous|interactive` (takes effect next gate, per-session); `/model`; `--local|--api|--hybrid`; `elai.bat` simplified --local + HOME fallback. Runtime flags asserted by MASTER; body may omit (P4G-I001).
- **InterventionCard (persona exhaustion)** [type: card] [atoms: P4G-E028] [status: planned]: all 5 personas + failure reasons on INTERACTIVE persona exhaustion.
- **INTERACTIVE spiral escalation panel** [type: panel] [atoms: P3C-E056, P3C-I001] [status: planned]: text "Agent {role} spiraling on {topic}: repeated {step_type} 3x. Evidence: [3 steps]. Options: [retry with hint] [substitute model] [skip step] [abort task]".
- **ContextBudget escalate** [type: CLI/toast] [atoms: P3E-E035, P3E-A002] [status: planned; no dedicated panel]: Display breakdown; INTERACTIVE surfaces Escalate(breakdown); no cockpit named.
- **Plan skeleton CLI** [type: CLI] [atoms: P3J-E052] [status: planned]: `/plan` shows skeleton from `.elai_cc/plans/{pipeline_id}.toml`; `/plan phase <id>` PhaseDetail; Slot 2 phase context; contract satisfied vs pending.
- **Research CLI** [type: CLI] [atoms: P4DOR-E013/E015, P3J-E067] [status: planned]: `/research`, `/ask`; oracle cache hit/miss; JSON-mode constrained decoding.
- **Profile / conventions** [type: CLI|file] [atoms: P3L-E007/E012, P3L-I001] [status: planned]: `/profile` reloads ProjectProfile; CONVENTIONS block agent-visible (not dashboard panel). Cap PI_T1_TOKEN_BUDGET=200 tokens.
- **present_options Interactive** [type: card/CLI] [atoms: P4D-E034, P4D-I001, P4D-A003] [status: planned; UI form open]: INTERACTIVE presents alternatives with evidence; AUTONOMOUS picks highest evidence_level×source_quality; concrete panel/command wiring not named.
- **SSB research progress** [type: status] [atoms: P4D-E057]: research_step / research_decompose_level / web_research_step for INTERACTIVE visibility.
- **HumanReview / Degraded** [type: toast|queue] [atoms: P4I-I001] [status: planned; no panel named]: operator-facing review queue semantics.
- **Isolation surfaces** [type: CLI/error] [atoms: P4A-E042, P4A-I001]: generic policy rejection; human approval MAJOR replan + auditor PROPOSED rules; no new dashboard panel.
- **DIVERGING MSI alert** [type: notification] [atoms: P4H-E038]: alerts user and pauses self-mod for that PipelineMode only.
- **dep-audit CLI** [type: CLI] [atoms: P4H-E069/E044]: `elai dep-audit` nightly anti-staleness.
- **ElaiNeuron / Empire host** [type: panel|CLI] [atoms: P4J-E017/E024/E033–E035, P4J-I001] [status: EMPIRE-PENDING scaffolding]: only task_submit/task_status/pipeline_result/health_check; Svelte panel + Tauri commands required but exact names not listed; brainctl → FactoryLine lifecycle.
- **index build CLI** [type: CLI] [atoms: P1B-E019, P1A-E039]: `elai-cli index build --project-root`; expects SSB symbol_graph_updated / index_checkpoint.
- **Rebuild notify** [type: notification] [atoms: P1I-E034]: user notified before rebuild (not silent).
- **Operator artifacts (no panels)** [type: file|CLI]: decision markdown 1F (P1F-E044); metrics TOML; probe SUPPORTED/NOT_SUPPORTED; rules evaluate/PostToolUse WARN/INFO strings (P3F-E045); self_mod_history.db + phase_4H.toml for later cockpit (P4H-I001).

### User flows
- **FLOW first-run project → task** [P4K-E029–E035] [planned]: ProjectSelector → New Project or Open Existing or Recent → template pick → task input Start/Enter → `submit_task` → Workspace plan tree + feed.
- **FLOW interactive gate** [P3A-E024, P3J-E045, P4K-E022/E024] [planned]: evidence present_gate → RiskLevel → Approve|Reject|Modify|Redirect (LOW auto) → submit_intervention_response → continue/IDLE.
- **FLOW diff approve** [P4K-E049–E051] [planned]: ApproveChange → unified diff → Accept|Decline|Retry+note / per-hunk → multi-file nav. Open: feed-inline vs modal (P4K-A003).
- **FLOW mode switch** [P4G-E012] [planned]: `/mode autonomous|interactive` → takes effect at next gate evaluation (not mid-phase).
- **FLOW surgical task** [P4DS-E002/E011] [planned]: `/task --surgical` → surgical_plan_created ≤2s → SurgicalDrawer ComplexityClass+AffectedSet.
- **FLOW research cached** [P4DOR-E015] [planned]: `/research "…"` twice in 60s → miss then hit cached_age_secs>0.
- **FLOW spiral escalate** [P3C-E056] [planned]: 3× repeated step_type → InterventionCard options retry-hint|substitute|skip|abort.
- **FLOW present_options** [P4D-E034] [planned]: INTERACTIVE pick among alternatives; AUTONOMOUS auto highest evidence. Error/empty: [unclear form].
- **FLOW full composition path** [MASTER-I444] [composed]: FSM classify → ground/research → typed tree → isolated agents → SSB journal → baby/step/phase/concept + FSM gates → optional media-validate → memorize/handoff → complete.
- **FLOW user surface chain** [PHASE-003/IX-017]: `index build` → knowledge canary → `/task --surgical` + SurgicalDrawer → `/mode`/`/model`/`/plan`.

### Onboarding-relevant surfaces
- ProjectSelector first-run (logo, subtitle, New/Open/Recent) [P4K-E029–E032].
- Template picker from manifest.toml [P4K-E034].
- `elai.bat` / release elai.exe primary sidecar path [P3J-E057].
- ResearchTierConfig auto-detect VRAM or `--vram-tier`; no-GPU → 8gb [P4D-E049].
- Visibility policy allowed_paths/hidden_paths [MASTER-E021, INV-G02].

### Notifications / background surfaces
- [preview] opens system browser [P4K-E044].
- DIVERGING MSI pause self-mod [P4H-E038].
- Rebuild pre-notify [P1I-E034].
- Stage3 promotion demote previous ARCHIVED [P4H-E035] (operator measurement surface).
- PostToolUse WARN/INFO completeness strings [P3F-E037].
- Nightly `elai dep-audit` [P4H-E069].

### Decide-at-edit / open UI decisions
- Diff viewer: replace feed vs modal [P4K-A003].
- RuntimePanel vs SettingsWindow placement [P4M-A002].
- SurgicalDrawer fields beyond ComplexityClass+AffectedSet [P4DS-A002, PHASE-003/AC-010].
- present_options concrete UI (CLI vs modal vs gate API) [P4D-A003].
- No dedicated budget cockpit [P3E-A002].
- ElaiNeuron exact command/panel names [P4J-I001].
- TokenUsage 4 vs 2 fields [RA-003 / CF-033].
- 4G runtime flags may live only in MASTER orientation [P4G-I001].

---

## D

### Surface inventory
- **CLI `elai template migrate`** [type: CLI] [atoms: D-002/D1-E25, D1-E38]: migrate TOML/MD prompts to Frozen archive.
- **CLI `elai archive verify`** [type: CLI] [atoms: D-002/D8-E01, D8-E10, D-001/D0-043]: path, --json, --fail-fast; exit 0/1. Slash **ArchiveVerify** [D8-E10].
- **CLI `elai ppm health`** [type: CLI] [atoms: D-003/D003-D2-E023]: three-stage health; exit 0/1. Flag `ELAI_PPM_ENABLE=1` [D2-E018, D10-E16].
- **CLI `elai calibration status|update`** [type: CLI] [atoms: D16-E015/E016]: corpus size, tau, rates (status always 0); update manual tau recompute; CI needs `L3_OVERRIDE:calibration_update`.
- **Bench CLIs** [type: CLI]: `elai bench sycophancy` [D23-009]; `mode-routing` [D21-E09]; `canary-regression` [D24-E10]; `prmbench` [D19-E011]; `tamper` [D25-E015]; `arc-agi2` [D20-013] + Slash BenchArcAgi2 [D20-026]; `conformal-validate` [D22-009] + Slash BenchConformalValidate [D22-017].
- **CLI `elai farm-adv-rm run|verdict`** [type: CLI] [atoms: D28-E24] [status: shipping-pass claimed live / CF-015 host live].
- **CLI `elai discovery review`** [type: CLI] [atoms: D15-017]: v1 print pending only; TUI deferred.
- **Hook script** [type: CLI]: `bn_status_report.py --plan=D` [D0-068, D26-034] progress overlay.
- **MutationArchivePanel.svelte** [type: panel] [atoms: D26-011, D0-061]: archive list/filter/colors.
- **LineageGraphView.svelte** [type: panel] [atoms: D26-013, D0-061]: Cytoscape parent-hash DAG.
- **TripwireStatusStrip.svelte** [type: panel] [atoms: D26-014]: last 5 events; 3s red SLA; yellow/red strip.
- **BaiArmsTable.svelte** [type: panel] [atoms: D26-015]: posteriors + CI. Flags `ELAI_BAI_ENABLE=1`, `ELAI_BAI_MODE=hierarchical` [D29-011/019]; `ELAI_SEED` deterministic BAI [D5-028].
- **HeldOutPanel.svelte** [type: panel] [atoms: D26-016]: active/retired per family.
- **ArchiveVerifyGauge.svelte** [type: panel] [atoms: D26-017]: Clean/Tampered status.
- **ReasoningDrawer D-26 tabs + dedicated archive sub-window** [type: window|drawer] [atoms: D26-018, D26-023]: archive reachability.
- **Tauri archive commands** [type: command] [atoms: D26-005–010]: `get_archive_recent(limit)`, `get_archive_verify_status` (async), `get_tripwire_status(window_minutes)`, `get_bai_arms`, `get_canary_report(optional template_id)`, `get_held_out_status`.
- **ModeRoutingPanel.svelte** [type: panel] [atoms: D27-011, D0-062]: pie/stacked modes; ModeSelected distribution.
- **RefusalRateGauge.svelte** [type: panel] [atoms: D27-012, D0-052]: healthy band [5%,50%].
- **NoveltyBreakdown.svelte** [type: panel] [atoms: D27-013].
- **DiscoveryQueueDrawer.svelte** [type: drawer] [atoms: D27-014]: ratify/reject; commands `get_discovery_queue`, `ratify_discovery` (reviewer), `reject_discovery` (reviewer+reason) [D27-008–010].
- **SycophancyFlagStrip.svelte** [type: panel] [atoms: D27-015, D0-062]: quarantine links; blocks MEMORIZING.
- **CheckerStatusPanel.svelte** [type: panel] [atoms: D27-016]: registry health.
- **Host windows D-27** [type: window] [atoms: D27-017]: MonitorWindow (routing/refusal); ChatWindow (sycophancy); Reasoning tab (novelty/queue/checkers).
- **Commands D-27** [type: command]: `get_mode_breakdown`, `get_refusal_rate`, `get_novelty_breakdown`, `get_sycophancy_flags` [D27-004–007].
- **L3-locked configs** [type: file]: `[bai]`, `[held_out]`, `[tripwire]`, `[canary]`, `[verification]`, `[runtime.ppm]`, `[adversary]`, `[conformal]`, `[refusal_sla]`, `[sycophancy_probe]`, `[observable_effect]`; `[novelty]` underspecified defaults; `reasoning.templates` commented archive_path/bounded_string_max.
- **Flags**: `ELAI_CLOUD_MODE` alternate-family [D10-E16]; `ELAI_CALIBRATION_BOOTSTRAP_ENABLE` [D30-E013]; `L3_OVERRIDE:*` commit tag; `[ci skip]` skips archive-verify CI [D8-E13].
- **Status displays**: Archive Clean/Tampered; Refusal gauge healthy band; Tripwire yellow/red; Canary Allow/Block/InsufficientData [D7-017]; PRMBench GatingVerdict [D19-E009]; ARCVerdict HonestlyEvaluated/RefusalOutOfBand/Contamination [D20-011].
- **CI hooks**: `scripts/locked_keys_check.sh` pre-commit; CI archive-verify.yml, locked-keys-audit.yml, bench-prmbench.yml.

### User flows
- **FLOW archive verify** [D8-E01/E10]: `elai archive verify` or ArchiveVerify slash → Clean|Tampered gauge; exit 0/1; CI fail-fast path; `[ci skip]` opt-out.
- **FLOW discovery review** [D15-014, D27-009/010]: DiscoveryCandidate queue → DiscoveryQueueDrawer → ratify (reviewer) | reject (reviewer+reason); CLI `discovery review` prints pending only (TUI deferred).
- **FLOW sycophancy quarantine** [D14-E01/E17/E29]: SYCOPHANCY_FLAG → strip + quarantine links → blocks MEMORIZING.
- **FLOW tripwire** [D4-022/024]: TripwireFiredConcurrent → strip red ≤3s SLA → AutoRevert path.
- **FLOW canary regression** [D7-023/025]: CrossDomainRegression → Canary report Block; [unclear] block rule non-target vs ANY-family (RA-005).
- **FLOW refusal SLA** [D17-E008]: RefusalRateOutOfBand bubble-up B13 pattern → gauge out of [5%,50%].
- **FLOW locked keys** [D9-E25, D0-044]: LockedKeysTouch / L3OverrideRequested notification; pre-commit locked_keys_check.sh.
- **FLOW calibration update** [D16-E016]: `elai calibration update` → needs L3_OVERRIDE tag in CI.
- **FLOW BAI hierarchical** [D29 mixed CF-016]: enable flags → BaiArmsTable posteriors; Gibbs path DEFERRED.
- **FLOW ClaimWithoutProcedure** [D12-014/024]: auditor BLOCK notification.
- **FLOW GoodhartRiskFlag** [D18-022, D0-053]: risk flag surface.

### Onboarding-relevant surfaces
- Template migrate into Frozen archive [D1-E25].
- L3-locked config tables at install/ops time.
- `ELAI_PPM_ENABLE` / `ELAI_CLOUD_MODE` opt-in gates.

### Notifications / background surfaces
- Bubble-Up RefusalRateOutOfBand [D17-E008].
- LockedKeysTouch / L3OverrideRequested [D9-E25].
- SYCOPHANCY_FLAG / quarantine [D14-*].
- TripwireFiredConcurrent + AutoRevert [D4-*].
- CrossDomainRegression block [D7-*].
- ClaimWithoutProcedure BLOCK [D12-*].
- DiscoveryCandidate queue [D15/D27].
- GoodhartRiskFlag [D18-*].

### Decide-at-edit / open UI decisions
- Archive hash two-part vs three-part [RA-004 / CF-040].
- Canary block rule objective vs implementation [RA-005 / CF-041].
- D-13 ModelFamily grouping shape [RA-006].
- `[novelty]` defaults underspecified [D15-023].
- discovery review TUI deferred [D15-017].
- D-29/D-30 partial/DEFERRED legs [CF-016/017].

---

## B

### Surface inventory

#### Shell / identity / stack
- **Dashboard V2** [type: window] [atoms: B0-002–004, SUM-0002–0004] [status: active architecture]: `crates/dashboard-v2/`; Tauri 2.10, Svelte 5 runes, Vite 5, three.js orbit. V1 parked `crates/.bkp_old/dashboard/`. Remap: ChatWindow→CommandWindow; ActivityFeed→ActivityWindow (orbit PLAN/MAP/RESEARCH); Settings/Plan/Monitor/Explore/Review same-named *Window; Shell→AppShell; Workspace→screens/Workspace; StatusBar/Sidebar→layout.
- **Visual tokens B-11** [type: page] [atoms: B11-002–017, SUM-0077/0079] [status: largely done audit; acceptance gate not proven B11-020]: JetBrains Mono (+ Inter display titles only); Lucide 16px / 20px sidebar monochrome teal when active; 1px bg-hover borders teal only active/focused; 6px panel/3px control corners; no shadows; 8px base spacing; scale 10/12/13/15/18 only; panel switch 80ms out / 120ms in; progress dot 3s breathe; no bounce/shake; orbit constellation-breathe 11s, dashflow active, shooting star ~14s.
- **Workspace shell B-12** [type: window] [atoms: B12-001–028, CF-014] [status: single Workspace SHIPPED; Tier-0 retired]: main.ts mount → ProjectSelector first run → Workspace owns AppShell/Sidebar/Panel/PanelDivider/StatusBar/WindowHost/Drawer/CommandPalette/Rail/orbit. ProjectSelector asks project only. Five-icon sidebar superseded by three-button 48px IconRail + four-question-tab IA (remap deferred B-26). `Ctrl+.` palette remains. Drawer consolidation deferred (~60 drawers). Superseded: Ctrl+Shift+W Tier-0 toggle, primary_shell TOML, first-run shell question [B12-011].
- **StatusBar** [type: panel] [atoms: B2-001–025, B9-015, SUM-0018] [status: producer+consumers planned/shipped mixed]: FSM badge colors; EXECUTING pulse; context-budget gauge amber 70% red 85%; handoff threshold; BSP superstep text; gate mode badge always visible (patch-4G.1). Manual smoke: transition ≤2s.

#### Primary windows
- **CommandWindow + CommandPalette** [type: window] [atoms: B14-E02–E15, B0-022, SUM-0087–0091] [status: B-14 shipped streaming]: vertical card list; outlined input-shell; slash dialog on `/`; groups Workflow (`/task /fix /research /continue`), Project (`/explore /status /config`), Help (`/ask`); outlined Approve teal / Reject-Modify-Redirect neutral; no solid CTAs. `stream_chat` Tauri 2 emit `turn|streaming_started|response|token|stop`; Stop→`cancel_stream`. Placeholders by FSM state. Intent: TASK|FIX|EXPLORE|CONFIG|INTERVENE|STEER|ABORT. Open: InterventionCard standalone vs variant [B14-A01]; stream return vs emitter [B14-A02].
- **ActivityWindow (orbit)** [type: window] [atoms: E-B13-002–013, B0-021, SUM-0082–0084] [status: V2 orbit authoritative; text feed retired]: PLAN constellation Concept→Phase→Step→BabyStep active centered; MAP PageRank rings + DKG edges; RESEARCH FORMULATE→SEARCH→EXTRACT→REFLECT→CONVERGE phi/psi bars. Captions from (fsm, grc.step_type, bubble_action?) no LLM. Layers: OrbitGrcDots, OrbitBudgetRing, OrbitEvidenceGates, BubbleUpStrip, OrbitShootingStarOverlay. Hover-reveal chrome DESIGN §7. Superseded: V1 [reasoning]/edit]/validate]/test] feed, Jump pill, FilterStrip [E-B13-028].
- **PlanWindow** [type: window] [atoms: B7-001–025, SUM-0046–0055] [status: largely shipped overlay; acceptance unchecked]: WindowHost direct mount; Concept/Phase/Step/BabyStep + eight BabyStep blocks (GROUNDING…EXIT_CRITERIA); dual with orbit PLAN mode; `get_decomposition_tree`, `get_phase_detail`, `get_baby_step_detail`, `get_dag_repair_log`, `get_executable_pipeline`, tree_position_changed highlight. Deferred: phase_sized_split badge [B7-017]; decomposition_alternative intervention card [B7-018]; real get_research_budget / get_source_tier_distribution [B7-020]. Cold-start stub fallbacks allowed [B7-023].
- **MonitorWindow** [type: window] [atoms: B0-024, B005-F04-006, SUM-0188] [status: B-26 polish; polling live]: 5s poll FSM/system/supervision/auto-advance/cloud-spend; inline intervention Approve/Reject/Take over; progress/phase dots; supervision; MSI/Proposals; thinking pulse; 100ms step flash; AutoAdvanceStats; Critic rounds N/3 in FRAMING; cosmos badge PLANNING/FRAMING; per-mode audit X/Y badges [B41-020].
- **ExploreWindow** [type: window] [atoms: B005-F04-007, E-B3-028] [status: graceful empty / Oracle pending]: `run_explore_query` Q&A stream with file refs/code/citations; 2.2s pending breath. No research chip (Oracle destination) [B0-051]. Full Oracle path B-26-owned / BM25-only limited until then.
- **ReviewWindow** [type: window] [atoms: B0-024, B005-F04-017, SUM-0192]: retry-note bar; Ctrl+Shift+A accept-all / Ctrl+Shift+R reject-all only when primary=review. Remove misleading Ctrl+Shift+H [SUM-0193].
- **SettingsWindow** [type: window] [atoms: B4-023/025, B27-E22, B37 RuntimePanel]: index schema version/migration; canary drift cosine/thr 0.85 + recompute; RuntimePanel per-role backend/chain/health Test; cloud provider/model/air-gap; webhook URL one-line blur save http(s); EmpireBridgePanel between runtime and cloud [B25-E18 connection shipped, events/repair pending].
- **IdeaFramingWindow** [type: window] [atoms: E-B40-017, SUM-0332] [status: planned; summary SHIPPED claim vs unchecked acceptance B-003/SDU-028]: pre-Workspace first-task; framed idea + TradeoffCard radio answers → FRAMING→PLANNING. `get_idea_framing_state` live not mock.
- **AuditTimelineWindow** [type: window] [atoms: B41-020/025]: mode colors + cascade arrows; live `get_per_level_verdicts(task_id)` no mock.
- **GraphWindow (optional A)** [type: window] [atoms: B33-E03–E38] [status: planned; decide A/B/C]: header stats/mode, cytoscape canvas, sidebar, footer search/layout/Mermaid/PNG; Ctrl+G; modes file|symbol|decomposition. Options B orbit MAP or C KnowledgeDrawer-only recommended unless right-click flyout needs A. User confirmation required for 8th window + cytoscape.

#### Drawers / panels
- **KnowledgeDrawer** [type: drawer] [atoms: B4-014–034, SUM-0020/0026]: query trace timeline (BM25/vector/PageRank/CatRAG/A-RAG/rerank/token-fit); breakers 5-row grid; recall metrics; pagerank_top; freshness; canary; vector cache hit rate; indexed/failed file counts drill-in; languages; knowledge_degradation; hop impact table + `/symbol_impact` [B005-F06-028]; grammar loaded/failed [E-B30-006].
- **ReasoningDrawer** [type: drawer] [atoms: B5-E12–E15, SUM-0035]: eight per-agent budget gauges (red <20%); grounding-sweep pie GROUNDED/VERIFY_NEEDED/UNGROUNDED; CSQ list Critical red badge; feasibility Pass/Fail/Blocked; collapsible per-role AgentGrcConfig (divergent red); REFLECT panel [B39-047]; seventh panel reflect state [SUM-0317 summary; canonical may be stub B39-054].
- **OrchestrationDrawer** [type: drawer] [atoms: B005-F01-003–015, B38-015–017]: AgentsPanel/EvidenceGates/Budget detail; seven evidence-gate pips also on ActivityWindow; `get_recent_evidence_gates` / `get_active_agents`; anti-spiral red badge at count 3; context reduction log; ambiguity events; GatesPanel section INITIAL/PROGRESS/REFINED/READY + dimension bars 1/2|2/2. B-17: inline VERDICT COUNTS not separate GatesPanel.svelte (reserved B-38) [B005-F02-017].
- **AntiStalenessDrawer** [type: drawer] [atoms: SUM-0020 Class C]: dep_audit, baseline_harness, deterministic_audit (mock until producers).
- **ResearchDrawer** [type: drawer] [atoms: B7-022]: research tree stages.
- **CosmosDrawer** [type: drawer] [atoms: B42-E24, SUM-0362] [status: ledger shipped worker+drawer; spawn_loop deferred]: status/refreshed; manual refresh; `get_cosmos_status`/`trigger_cosmos_refresh`. Path `~/.elai/cosmos/` user-global intentionally not `.elai_cc` [SUM-0453].
- **SurgicalDrawer** [type: drawer]: see PHASE (also B surface chain).
- **MediaValidationPanel** [type: panel] [atoms: B24-028, SUM-0179] [status: summary DONE claims not in canonical B-24 unchecked]: Settings or Monitor mount; thumbnails/results; Tier-1 veto > Tier-2 advisory; missing VLM degrades.
- **FeasibilityPanel / OrchestrationDrawer feasibility** [type: panel] [atoms: B21-012] [status: partial]: per-concept results, novel-step counts, alternatives ≤3, probe progress.
- **ProposalsPanel + MsiPanel** [type: panel] [atoms: B23-018] [status: planned flag-gated]: archive, per-mode MSI, pending/staged/last revert; mode verdict convergence/regression/evolution badges. Stub names: get_pending_proposals, get_applied_proposals, apply_proposal_from_user, reject_proposal, get_msi_status [B23-035]. Feature `self-modification` [E-B3-026].
- **ArtifactsPanel** [type: panel] [atoms: B35-036–040] [status: wholly planned]: name/path/description/status/chunk/last-indexed; Re-index + filtered Search; first 200 desc chars; mount Workspace or KnowledgeDrawer. Commands list/search/reindex/remove (name drift project-prefixed vs short) [B35-038 / DELTA-B35-08].
- **BranchTree.svelte** [type: panel] [atoms: B0-019, E-B9-010] [status: summary DONE; canonical acceptance unchecked / DAG writes deferred]: fork/merge/close stubs→CLI; real branch-status reads; palette `/session fork|merge|close|list`.
- **FocusTunnelPanel** [type: panel] [atoms: B005-F05-025]: seven focus_tunnel modules.
- **CloudPanel** [type: panel] [atoms: B27-E22, SUM-0203]: provider/model/air-gap; per-task/day gauges; warn 80%/100%; ESCALATE_TO_CLOUD.
- **BubbleUpStrip** [type: panel] [atoms: B0-021, E-B13]: orbit caption/status rows including FEASIBILITY_PROBE.
- **GatesPanel (B-38)** [type: panel] [atoms: B38-015]: ambiguity milestones + bars; not B-17 file.
- **Index progress panel** [type: panel] [atoms: B31-043] [status: B-31 DONE CF-010]: live checkpoints; resume + lock-contention/read-only banners; batch/file savings; holder PID.
- **RuntimePanel** [type: panel] [atoms: E-B37-005/017, SUM-0298] [status: summary SHIPPED; acceptance unchecked SDU-027]: backends, chain, g/y/r health, Test backend; `/runtime show|set|fallback`.
- **RemoteSidecarBootOverlay** [type: panel] [atoms: B0-033]: progress; start/get/cancel_remote_sidecar; Workspace `?remote=1`.
- **NotificationManager.svelte** [type: toast] [atoms: B005-F04-009, B0-024]: Browser Notification API; localStorage permission; FSM state-change; consent banner; Workspace mount; webhook parallel fire, errors swallowed [B005-F04-015].
- **Oracle research chip** [type: card] [atoms: B0-047–052]: one-key confirm over auto-detected research intent; never silent; Enter accept Esc reject Tab edit; dispatches `/research`; autofocus; disables textarea while pending. Counterfactuals: slash /research|/ask, lone `?`, streaming, `what calls foo()?` no chip; pytorch version yes.

#### Commands / CLI (dense roster)
- Core FSM: `get_fsm_state`, `get_context_state`, `get_phase_progress`/`get_plan_progress` [open either/or B005-F04-A01], `submit_task`, `submit_intervention_response`, `set_gate_mode`.
- Knowledge B-4: `get_query_trace`/`get_query_trace_recent` [name conflict B-002/SD-014], `get_embedding_breakers`/`get_circuit_breaker_states`, `get_recall_metrics`, `get_pagerank_top`, `get_freshness_state`, `get_canary_drift`, `get_vector_cache_hit_rate`, `get_search_mode`, `get_dkg_stats`.
- Reasoning B-5: `get_grc_budgets`, `get_grounding_sweep`, `get_csq_log`, `get_feasibility_log`, `get_agent_grc_config`, (+ contract sixth `get_auditor_rubber_stamp_rejections` [SD-B5-02]).
- Orchestration B-6: `get_evidence_gates`/`get_recent_evidence_gates`, `get_agents`/`get_active_agents`, `get_last_context_budget`.
- B-36 Track D (13 rows + 15 V2-only) [B36-E01–E26, SUM-0285–0290]: sidecar health/settings/restart (remote SSH only); `run_planning_task`, `run_research`, `one_shot_prompt`, fetch settings, `obscura_test_fetch`, `run_d5_live_contract`, `reindex_now`, `get_tool_visibility`, local max-context/min-prompt, `sidecar_search`, live grounding_cache_stats; plus recompute_steering_vector, get_steering_status, get_mcp_status, get_gg_signals, get_sidecar_status, get_bubble_up_events, get_hook_executions, get_shortcoder_stats, run_explore_query, continue_task, …
- Branch B-9: `fork_session_branch`, `merge_session_branch`, `close_session_branch`, `get_session_branches`, load_handoff `/continue` picker.
- Reflect B-39: `/reason on|off`; `get_reflect_state` [canonical stub vs summary live]; soft warn 85% `ELAI_B39_SOFTWARN_ENABLE` [B0-081].
- Framing B-40: `get_idea_framing_state`.
- Cosmos B-42: `get_cosmos_status`, `trigger_cosmos_refresh`.
- Graph B-33: `get_graph_view_data`, `get_impact_overlay`, `export_graph_mermaid`.
- Symbol graph B-29 tools: compute_impact_radius, compute_call_flow, compute_symbol_context, list_symbols; dashboard `get_impact_analysis`.
- Artifacts B-35: list/index/search/remove_artifacts (+ project-prefixed variants).
- Indexing B-31: `cancel_indexing`, auto_resume; flag `ELAI_DISABLE_INDEXING` wire-dark allowlist.
- Entry points B-34: `list_entry_points`, CLI `/entry_points`.
- Cloud B-27: `/task --mode cloud|hybrid|local`; spend gauge commands.
- Feature flags: `get_feature_flags`; Cargo `autonomous-mode`, `self-modification`; offline `ELAI_DASHBOARD_OFFLINE` / `__ELAI_DASHBOARD_OFFLINE__` / VITE_*; `ELAI_DASHBOARD_DEV=1` Class B hide; `ELAI_CODE_VEC=1` wire-dark rollout.
- CLI index: `elai-cli index build`; `elai dep-audit`; `elai.exe bench canary-regression`.

#### Status / notifications (B)
- Phase badge + EXECUTING pulse + orbit halo/bubble [B2].
- Anti-spiral red badge count 3 [SUM-0041].
- EvidenceGate verdict panel seven gates [SUM-0037].
- ContextBudget bar chart + AdaptiveLevel events [SUM-0043].
- Canary cosine X / thr 0.85 red fail [B4-025].
- ReflectBudgetSoftWarn 85% [B0-081].
- Cosmos N fresh / Y stale / Z decayed mini-badge [SUM-0362].
- Index lock/resume banners [B31-043].
- Remote activation runbook overlay [B0-092].
- Webhook + Browser Notification on intervention [B005-F04].
- Responsive: <1000px hamburger Sidebar; <800px secondaryCollapsed [SUM-0196].
- Shortcuts: Ctrl+1–7, Ctrl+N/W/D, Ctrl+., Ctrl+,, Ctrl+Shift+P, Esc; Review Ctrl+Shift+A/R guarded [B005-F04-012/017].
- Window state localStorage key `elai.dashboard-v2.workspace.v1` only; no SSB WINDOW_STATE; no cross-device [SUM-0189, INV-F03].

### User flows
- **FLOW first-run shell** [B12-003]: launch → ProjectSelector → select/create project → Workspace (no shell choice).
- **FLOW chat / task submit** [B14]: type or `/` palette → classify intent → stream_chat tokens → cards Status|Intervention|Error|Disambiguation|Steering; Stop cancel_stream. During EXECUTING: constraints or wait for intervention.
- **FLOW oracle chip** [B0-047–050]: natural research question → chip → Enter=/research | Esc reject | Tab edit → else normal send.
- **FLOW interactive intervention** [B005-F01-008, F04-006]: producer needs approval → intervention_request SSB → Monitor/orbit card Approve|Reject|Modify|Redirect|Take over → submit_intervention_response → optional webhook+browser notify.
- **FLOW autonomous gate** [B0-038–040, B005-F03]: `/mode` / CommandPalette set_gate_mode → AutoAdvanceStats (5s poll not push) → HIGH-risk still human; material watchdog events only. Open: CLI `/mode` still informational at recorded point [B0-040].
- **FLOW framing** [E-B40-017]: IdeaFramingWindow tradeoffs → FRAMING evidence seventh check → PLANNING. Critic rounds N/3 Monitor. Zero BLOCK exit vs max_rounds=3 conflict [C-B40-001].
- **FLOW plan drill** [B7]: task → decomposition SSB → PlanWindow tree + orbit PLAN → cursor highlight → DAG repair log / executable pipeline. Error: cold-start stubs until producers fire.
- **FLOW research orbit** [E-B13-005, B7-014]: FORMULATE…CONVERGE stages via WebResearchStep; captions update.
- **FLOW diff/review** [B14/B26]: REVIEW primary → bulk Ctrl+Shift+A/R; retry-note bar.
- **FLOW branch session** [E-B9]: palette `/session fork|merge|close|list` → BranchTree; `/continue` → handoff picker → load_handoff. Merge/close PARTIAL deep producer B-9 [E-B3-024]. Budget 85% auto-handoff imminent.
- **FLOW indexing resume** [B31]: startup auto_resume → progress panel; contention → read-only banner + holder PID; cancel_indexing at boundary; SIGTERM graceful ≤60s tier D.
- **FLOW remote sidecar** [B0-033/092, B36]: `?remote=1` or elai.bat --remote → overlay boot → health/settings/search live; restart SSH remote only never local heavy.
- **FLOW cloud spend** [B27-E08/E22]: resolve mode → probe <100ms → fallback or LOCAL notify → Monitor spend $; 80% COMMAND warn; 100% HYBRID degrade / CLOUD fail step.
- **FLOW reflect chat** [B39/B39C]: `/reason on` → TRIAGE→REASONER→AUDITOR→SYNTHESIZE; skipped=false text-only markdown bypass tools; softwarn 85%.
- **FLOW media validate** [B24, SUM-0388]: renderable → Tier-1 deterministic first (veto absolute) → Tier-2 advisory → human review if disagree → RCE loop.
- **FLOW self-mod proposals** [B23]: feature flag → ProposalsPanel → apply|reject; L3 structural changes HUMAN-ONLY proposal only; MSI badges.
- **FLOW graph impact** [B33]: Ctrl+G or Plan “Show in Graph” / Activity “Show blast radius” → mode + impact overlay hop colors → Mermaid/PNG export.
- **FLOW cosmos refresh** [B42]: Layer-1A tick or manual drawer refresh → Monitor badge; briefing inject.
- **FLOW artifact index** [B35 planned]: declare manifest → auto-index with code → ArtifactsPanel reindex/search.
- **FLOW ambiguity gate** [B38]: score → GatesPanel labels → floor fail events → OrchestrationDrawer.
- **FLOW auditor modes** [B41]: mode dispatch → AuditTimelineWindow cascade; one-respawn policy ≠ BubbleUp limits.
- **FLOW offline dashboard** [E-B3-002/041]: ELAI_DASHBOARD_OFFLINE → tryInvoke mock fallbacks; get_feature_flags exempt; live commands must not Ok(default()).

### Onboarding-relevant surfaces
- ProjectSelector first-run only project [B12-006]; layout wizard deferred/meaningless [B12-024].
- B-43 CLI-only install/doctor path explicitly excludes dashboard; B-45 owns packaged Tauri [B-005/X09, B005-F08-007]. Installer: HTTPS, blake3 binary verify, credentials.json 0600, elai_cc.toml.template VRAM+B-37 defaults [B005-F08].
- Model ONNX stage under `.elai_cc/models/onnx/` blake3 verify [B4-008/009, SUM-0023].
- Remote sidecar runbook SSH key/port/scripts [B0-092].
- Path authority `.elai_cc` not `.elai`; warn coexistence never auto-delete [B1-014]; CLI bootstrap clear uninitialized error not EmptyDkgQuerier [B1-013].

### Notifications / background surfaces
- NotificationManager + webhook [B005-F04].
- BubbleUpStrip + shooting star [B13/B0-021].
- Index lock/resume/cancel banners [B31].
- Remote boot overlay [B0-033].
- Cloud 80/100% spend warnings [B27].
- Reflect softwarn [B0-081].
- LanguageGrammarStatus startup SSB [E-B30-006].
- CanaryDrift startup + every 30s [B4-025].
- CircuitBreakerStateChange [B0-014].
- DIVERGING / MSI via Proposals path (B-23).
- Empire connection state (partial panel) [B25-E18].

### Decide-at-edit / open UI decisions
- Graph placement A GraphWindow / B orbit MAP / C KnowledgeDrawer [B33-E04, SUM-0260]; B conflicts B-36 “do not touch orbit” [B-004/X-C01].
- InterventionCard extract vs inline Monitor [B005-F04-005]; CommandWindow takeover reconsider later.
- Phase progress: extend FsmSnapshot vs `get_plan_progress` [B005-F04-A01].
- B-14 InterventionCard component vs card variant [B14-A01]; stream mechanism [B14-A02].
- B-39 live Tauri get_reflect_state + drawer pill vs Settings [B39-073]; summary vs canonical stub [B-002/SD-035].
- B-7 deferred: phase_sized_split, decomposition_alternative card, research metric readers [B7-017/018/020].
- B-35 command-name project-prefixed vs short [B35-038]; ActivityFeed vs orbit terminology unmapped [DELTA-B35-09].
- B-40 critic zero-findings short-circuit vs max_rounds=3 ban [C-B40-001].
- B-12 sidebar three vs historical five icons deferred [B12-023].
- B-9 dashboard DAG writes deferred; handoff schema ts-rs vs manual types (B-3 no ts-rs) [E-B3-017 vs E-B9-013].
- B-4 command name 1:1 conflicts query_trace(_recent), breakers names [B4-003, B-002/SD-014].
- B-5 five vs six commands / rubber-stamp reader inventory [B5-C02, B-004/X-C03].
- AutoAdvance feature flag name not named [B005-F03-A03].
- B-24/B-37/B-40 summary DONE vs canonical unchecked acceptance — treat summary as later evidence not canonical DONE [B-002/SD-019, B-003/SDU-027/028].
- W-6 N/A: any human-visible SSB without Svelte pair needs W-6 justification [INV-F05].
- Fixture path double `.elai_cc/.elai_cc` and `.elai_cc/.elai/sessions` unreconciled [B29S-040, B315-C03, B005-F06-A02] — path display risk only.

Superseded notes (B): V1 text ActivityFeed/Jump/Filter [E-B13]; Tier-0 Shell and five-mode sidebar [B12/CF-014]; ChatWindow name [B0-004]; primary_shell TOML [B12-008]; Ctrl+Shift+H recovery claim [SUM-0193]; V1 ssb_viewer path (AppState commands) [B0-003]; historical six-window defaults INTERACTIVE ACTIVITY/COMMAND vs AUTONOMOUS MONITOR/ACTIVITY [B12-014] as historical only under single shell.
