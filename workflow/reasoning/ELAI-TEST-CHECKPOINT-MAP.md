ELAI TEST-CHECKPOINT MAP (human index)
Generated 2026-06-27 from phase-index-MASTER + B..K SUMMARY indexes + ELAI-FEATURE-MAP + teach/elai-concept-map.
Machine-consumable twin: ELAI-TEST-CHECKPOINT-MAP.json (11 milestones, 86 features).

How to read this
- One section per milestone (a "this part is finished" checkpoint). Each bullet: feature_id - name - [surface] - trigger -> expected observable effect.
- surface = where the tester acts: dashboard (drive the UI like a human), cli (`elai <subcommand>`), both, internal (no user surface; verify the named SSB row), bench.
- Everything is treated AS SPECIFIED (spec-eval): a [planned]/[partial]/[wire-dark] status is an implementation note, never a test-skip reason. The tester verifies the design; status only tells you whether a green/red is expected today.

TESTER-RELEVANT CAPABILITIES
- (a) Multi-provider / free-model / BYOK runtime so the tester (or ELAI) can run on free API models: B-37 (multi-runtime backend abstraction, 11 backends + PerRoleRuntime - SHIPPED), B-27 + C-5 + E-39 (cloud mode + hard cost ceiling), E-20 + H-7 + I-25 (local/cloud + calibrated + ledger-trained routing), J-9 (hardware model catalog, per-VRAM free/local candidate tuples), J-28 (per-lane token/cost ledger), J-30 (BYOK on the B-37 backbone), K-24 (credit-based seats on an OpenRouter rail), E-51 + J-19 (MLX/llama.cpp + vLLM lanes). The agent-CLI bridge (claude_code/codex_cli/gemini_cli, C7-ROUTING) can also ride a vendor login through the Tier-C spawn seam.
- (b) Computer-use / CDP / dashboard-e2e / vision-testing inside ELAI itself: E-14 (Playwright AXTree-first browser E2E + multi-context journeys + deploy-preview worker - the computer-use leg), B-11 (dashboard Playwright/Vitest visual-regression harness, pinned CDP capture), the .claude/tools/visual/ ui-diff + ax-scaffold + vlm-prep tools (deterministic pixel diff + AXTree perception scaffold + schema-constrained critique - SHIPPED, see .claude/rules/visual-ui-loop.md), and J-1 (perception gateway + /see two-call perception scaffold). Capture-determinism rules (deviceScaleFactor=1, fromSurface=true, font-render-hinting=none) live in visual-ui-loop.md.

== BASE-SKELETON (phases 0-1: compiles + runs + indexes deterministically) ==
- phase-0 - FSM base loop + file_read + slash cmds - [cli] - `/task read src/main.rs` -> IDLE->EXECUTING->IDLE, content respecting visibility filter, phase_transition rows
- phase-0.10 - determinism proof - [cli] - ELAI_SEED=42 task x2 -> byte-identical SSB db dumps (diff=0)
- phase-1A - DKG tree-sitter graph - [cli] - `index .` -> Knowledge drawer shows non-zero typed nodes/edges
- phase-1B - BM25 full-text - [cli] - `/explore <kw>` -> ranked file:line chunks <10ms p99
- phase-1C - dual-model ONNX vectors - [internal] - startup -> canary within 5s; missing model -> DEGRADED not crash; circuit-breaker grid renders
- phase-1D - PageRank repo map - [both] - Knowledge drawer -> pagerank top symbols score>0
- phase-1E - CatRAG+A-RAG pipeline - [cli] - `/explore <q>` -> QueryTrace shows all stages latency_ms>0
- phase-1G - cross-encoder reranker - [internal] - `/explore` (>10 candidates) -> rerank StageTrace present
- phase-1I - retrieval resilience - [both] - kill a model -> SearchMode cascade + knowledge_degradation, no crash
- B-1 - path unification (.elai_cc) - [internal] - `index .` -> real symbol counts (no EmptyDkgQuerier)

== CORE-REASONING (phase 2: GRC + grounding) ==
- phase-2A - GRC state machine + per-topic budget - [internal] - `/task` -> grc_step_emitted rows w/ grounding; budget overflow forces converge
- phase-2B - grounding verification + plan sweep - [internal] - ungrounded claim -> PLANNING->TESTING_FIRST blocked; grounding_sweep_result
- phase-2C - per-agent GRC config (8 roles) - [both] - Reasoning drawer 8 gauges; auditor Converge w/o FINDING: -> auditor_rubber_stamp_rejected
- phase-2D - CSQ + inline feasibility - [internal] - `/task implement a fn that writes a file` -> CSQ Critical w/o guard BLOCKS; csq_evaluated

== CORE-ORCHESTRATION (phase 3: FSM + SSB + agents + BSP + budget + rules + plan + DAG) ==
- phase-3A - full 9-state FSM + 7 evidence gates - [both] - `/status` + StatusBar -> badge changes per transition, gates block on missing evidence
- phase-3B - SSB append-only WAL - [internal] - any task -> typed rows; UPDATE/DELETE panics in debug
- B-2 - phase_transition SSB wiring - [dashboard] - StatusBar badge + EXECUTING pulse + orbit halo on transition
- phase-3C - agent system + supervision + anti-spiral - [both] - AgentsPanel: 8 roles w/ heartbeat + restart count; red badge at anti_spiral_hit=3
- phase-3D - BSP supersteps + channels - [both] - parallel task -> deterministic write order + rising bsp_superstep counter
- phase-3E - context budget allocator (8 slots) - [both] - per-slot bar chart + context_reduction events under pressure
- B-6 - EvidenceGate panel + InteractiveGate - [dashboard] - 7 gate verdicts render; intervention_request on HIGH-risk gate
- phase-3F - rules engine (governance as data) - [both] - Rules drawer: seeded rule -> ALLOW/WARN/BLOCK; mutable=0 immune
- phase-3G - hierarchical plan mgmt - [both] - multi-phase task -> plan tree; contract mismatch BLOCKed (coherence_check)
- phase-3H - session DAG handoff + lossless trim - [both] - `/continue` -> resumes from handoff; stubs re-expand from SSB seq

== B-UI-TIER (dashboard-v2 + MVP completion + producer wiring) ==
- B-14 - live token streaming chat - [dashboard] - CommandWindow: type -> SSE token stream; Stop cancels mid-stream
- B-13 - activity orbit (PLAN/MAP/RESEARCH) - [dashboard] - ActivityWindow orbit switches mode on FSM change; bubble-up strip
- B-4 - knowledge producer wiring + QueryTrace - [dashboard] - `/explore <q>` -> QueryTrace + per-tier R@5/R@10/MRR badges with real numbers
- B-5 - reasoning producer wiring - [dashboard] - ReasoningDrawer: 8 live budget gauges + grounding-sweep pie + CSQ
- B-7 - decomposition tree + DAG viz - [dashboard] - multi-concept task -> ExecutablePipeline DAG + phase/step/baby_step_ready
- B-3 - stub command eradication - [dashboard] - open every drawer -> no mock/placeholder elements
- B-36 - dashboard-v2 src-tauri wiring - [dashboard] - Settings: sidecar_health, reindex_now, get_dkg_stats live
- B-26 - dashboard completion (Monitor/Explore/Review/Notify) - [dashboard] - Ctrl+1..7 windows; ReviewWindow hunk accept/reject; webhook
- C-6 - design token alignment + Stop/Ctrl+Enter - [dashboard] - computed tokens match palette; Ctrl+Enter sends, Stop during stream [planned]
- C-7 - dashboard UX completeness - [dashboard] - Ctrl+8 Explore; Ctrl+Shift+H recovery; <1000px hamburger [planned]
- C-4 - orchestration + cloud-spend surface - [dashboard] - cloud task -> MonitorWindow >=1 non-IDLE agent + non-zero spend [planned]

== RUNTIME-PROVIDERS (multi-provider / free-model / BYOK + cloud mode) ==
- B-37 - multi-runtime backend (11 backends, PerRoleRuntime) - [both] - `/runtime list` + RuntimePanel set/test per-role runtime
- B-27 - cloud mode (LOCAL/CLOUD/HYBRID) - [both] - `/task --mode cloud` -> CloudPanel mode; HYBRID escalates only on failure [planned]
- C-5 - cloud hard ceiling + BYOE - [both] - `/task --mode cloud` cap=0.05 -> WARN_LIMIT_REACHED then RouteBlocked [planned]
- E-39 - hard ceiling BLOCKS + difficulty estimator - [internal] - per-task cap -> hard_ceiling_blocked / retry_budget_exhausted [planned]
- E-20 - local/cloud routing + data-leakage guard - [internal] - prompt w/ API key -> route_decision local; data_leakage_blocked [planned]
- H-7 - architect/editor/verifier routing - [internal] - mixed task -> RouteDecisionEvent per step (editor cheap / architect cloud) [planned]
- I-25 - ledger-trained router - [internal] - mixed batch -> RouteDecisionEvent w/ feature vector + predicted-vs-actual [planned]
- K-24 - credit-based seats (OpenRouter rail) - [both] - metered run -> CreditLedgerEvent reserve/settle; ELAI_BILLING_ENABLE [planned]
- J-9 - hardware model catalog (per-VRAM free/local) - [internal] - `model-drift show` -> per-lane candidate tuples [planned]
- E-31 - per-VRAM tier matrix + EscalationPolicy - [internal] - tier resolution -> correct model+backend per 12/24/48GB [planned]
- E-27 - AWQ+Marlin + FP8 KV - [internal] - sidecar boots AWQ+Marlin; awq_loaded/fp8_kv_active [planned]
- E-28 - FA2 mandatory - [internal] - boot w/o FA2 refuses; fa2_pinned when active [planned]
- E-51 - MLX + llama.cpp adapters - [internal] - macOS->MLX, GGUF host->llama.cpp; mlx_loaded/llamacpp_loaded [planned]
- J-19 - vLLM serving lane - [internal] - `/runtime list` -> vLLM resolves + probe completes [planned]
- J-28 - token ledger (per-lane cost) - [both] - any turn -> UsageTurnEvent normalized input/output/cached + price [planned]
- J-30 - BYOK on the backbone - [both] - Settings BYOK key -> authenticates a runtime + runs a task [planned]
- C7-AGENTBRIDGE - agent-CLI bridge (ride vendor login) - [internal] - spawn seam launches vendor CLI under Tier-C [partial]

== AUTONOMY (autonomous mode + anti-spiral + feasibility) ==
- B-22 - autonomous mode (AutonomousGate + auto-advance) - [both] - `/task --mode autonomous` -> self-advance; MonitorWindow AutoAdvanceStats>0
- C-10 - anti-spiral (4 patterns + 5 personas + drift canary) - [both] - `--mode autonomous fix the impossible bug` -> stagnation_spinning_detected + lateral_thinking_activated + watchdog_timeout [planned]
- C-22 - feasibility prober (4F) - [both] - `/research --mode cloud implement using a novel library` -> system_probe + non-null ResourceDelta + FeasibilityPanel [planned]

== TRUST-VERIFICATION (the heart: wiring + cant-lie + RRA + blast-radius) ==
- C-27 - wiring contract audit + determinism - [cli] - `wiring audit` -> WiringEvidence (clean) / WiringViolation file:line (injected stub) [planned]
- C-28 - mutation testing gate (T-1) - [both] - tautological-test task -> BLOCK at AUDITING by low mutation_score [planned]
- F-4 - cant-lie test-integrity (stdout_hash) - [both] - test exits 1 while auditor claims PASS -> anti_fabrication_block [planned]
- G-2 - Reproducible-Run Artifact [KEYSTONE] - [cli] - `run --emit-rra` then `replay --verify` -> PASS or first DIVERGENCE; run_artifact_emitted
- G-1 - harness-wide taint propagation - [internal] - injected web/MCP -> destructive sink blocked; taint_flow names origin [planned]
- G-8 - signed attestation + offline verify - [cli] - `verify-attestation <commit>` -> verified/not-verified WITHOUT re-running [planned]
- I-18 - tamper-evident row chain + AI-BOM - [cli] - tamper a mid-run row -> verify breaks at that seq; CycloneDX-ML BOM [planned]
- J-16 - blast-radius anti-regression gate - [both] - rename a widely-called symbol -> a pass->fail caller is BLOCKed [planned]

== SECURITY-GATES (VibeSec write-gates + injection defense + sandbox) ==
- F-1 - VibeGate unified fail-closed write-gate - [both] - write SQLi/secret -> refused pre-disk; vibesec_code_vuln_blocked/vibesec_secret_blocked [planned]
- F-2c - low-FP CWE rules + seccodeplt baseline - [cli] - Python `password=...` -> blocked; `bench seccodeplt` -> insecure_rate_measured [planned]
- F-9 - hallucinated-dep write-time gate - [internal] - write `import nonexistent_pkg` -> refused; write_time_slopsquat_blocked [planned]
- F-3 - CaMeL / tool-filter data-plane - [internal] - AgentDojo injected fixture -> tool_filter_committed; out-of-set tool cannot fire [planned]
- E-30 - PreToolUse argv destructive-verb allowlist - [internal] - emit `rm -rf ~/` -> destructive_verb_denied regardless of LLM verdict [planned]
- G-3 - post-expansion FS gate + egress IP-pin - [internal] - `rm -rf $X` expanding to ~/ -> blast_radius_blocked w/ pre+post-expansion cmd [planned]
- G-4 - live MCP rug-pull re-pin - [internal] - MCP swaps a tool description post-approval -> mcp_rug_pull_blocked w/ diff [planned]
- E-37 - prompt-injection data-plane defense - [internal] - hostile-injected PDF -> promptarmor_verdict; egress_blocked [planned]
- C-11 - Tier-C sandbox isolation - [internal] - harmless /etc/passwd-read script -> NSpid!=1 + EACCES, no SANDBOX_VIOLATION [planned]

== MEDIA-PERCEPTION (media validator + perception + browser E2E) ==
- B-24 - media validator (Tier1 parsers + Tier2 VLM) - [both] - render task -> MediaValidationPanel; deliberate-fail -> Tier1 veto
- E-14 - browser E2E (AXTree + axe-core + deploy preview) - [both] - app build -> axtree_snapshot + deploy_preview_ready; a11y bug -> axe_violation [planned]
- E-36 - document vision RAG (8th GroundingMethod) - [internal] - claim from design.pdf -> visual_artifact_check verifies [planned]
- J-1 - perception gateway + /see - [both] - `/see <screenshot.png>` -> preprocess + perception-scaffold JSON before reasoning [planned]

== SELF-IMPROVEMENT (DGM self-modification + reasoning-template archive, safe) ==
- D-8 - cryptographic archive + verify - [cli] - `archive verify` -> tampered row exit 1 broken-link; clean exit 0 [planned]
- F-11 - close one config-only DGM generation under guards - [cli] - `ELAI_DARWINIAN_LOOP_ENABLE=1 darwin-loop --generations 1` + trap mutant -> >=2 darwinian_admit (RejectedByGuard + LoopClosed) [planned]
- D-4 - tripwire stack - [internal] - inject regression -> >=3 tripwires fire -> auto-revert; TRIPWIRE_FIRED [planned]
- D-5 - BAI Thompson promotion - [internal] - 50-round noisy-arm bench -> regret within Audibert bound [planned]
- C-21 - self-mod + locked keys - [internal] - commit touching LOCKED_KEYS file -> blocked; AllowedConfigKey closed enum [planned]
- I-31 - GRPO flywheel [LAST, gated] - [internal] - reward-hack candidate -> HARD-BLOCKed by H-28; rewards cite gate verdict rows [planned]

== TESTER-CAPABILITIES (what the fake-human tester reuses inside ELAI) ==
- E-14 - Playwright browser E2E + CDP capture - [internal] - browser e2e task -> AXTree-first snapshot + multi-context journey [planned]
- B-11 - dashboard visual-regression harness - [internal] - tests/visual.spec.ts -> pinned CDP screenshots vs golden frames [partial]
- VISUAL-UI-LOOP - ui-diff + ax-scaffold + vlm-prep tools - [internal] - deterministic pixel diff + crops + AXTree perception scaffold [shipped]
- J-1 - perception scaffold service - [internal] - `/see <dashboard screenshot>` -> structured element list before judging [planned]
