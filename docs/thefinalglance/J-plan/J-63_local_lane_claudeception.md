# J-63: local-lane Claudeception — a keyless, sandboxed local-sidecar inference endpoint for ELAI-generated/run apps + artifacts, so "build me an AI-powered app" works out-of-the-box against the user's own model with no cloud key and no per-token cost

STATUS: PLANNED (WAVE FABLE-STEAL-1; nature BUILD with a runtime leg, the J-1/J-2 class; COMPOSES J-48 local serving router + RuntimeOrchestrator + the AnthropicClient/base_url wiring + trust_dialog.rs CVE guard + Tier-C process isolation, restates none)

- Track J (Fable-5 system-prompt steal) | lever LOCAL-FIRST / COST (capability-multiplier-not-weights) | grounding STRONG on the ELAI sidecar/security seams (code-grounded this session); ILLUSTRATIVE on the Fable source (leaked prompt, see Citations); the CAPABILITY-FIT is a design hypothesis (see Open questions) | nature BUILD | effort L | WAVE FABLE-STEAL-1 | parallel-safe with J-62 (disjoint files) | composes J-48 local serving tier router + RuntimeOrchestrator + sidecar AnthropicClient + trust_dialog.rs (F-class security) + Tier-C process isolation (4A) + J-2 control plane
- Premise: the Fable spec-eval found a second harness lever — "Claudeception" (anthropic_api_in_artifacts): claude.ai artifacts call the Anthropic /v1/messages endpoint at runtime with NO API key ("the assistant should never pass in an API key, as this is handled already"), so the artifact ITSELF is AI-powered. This is a large part of why Fable's generated apps feel magic: the output is not static code, it is a working AI app. ELAI is local-first and cost-conscious; the natural analog is NOT a cloud-keyed chat artifact but a KEYLESS LOCAL endpoint: when ELAI scaffolds/runs an app that wants LLM capability, wire it to the LOCAL sidecar (the ANTHROPIC_BASE_URL=localhost:8100 Anthropic-protocol server already running for ELAI's own inference) by default, so the generated app is AI-powered on the user's OWN model, no cloud key, no per-token cost. The grounded constraint that makes this non-trivial and net-new: ELAI today has NO generated-app / preview / dev-server / artifact-run concept (grep this session = none) — code ELAI produces is built/tested, never served with a wired LLM — AND base_url redirection is a known token-exfil CVE (trust_dialog.rs, CVE-2026-21852: "ANTHROPIC_BASE_URL / OPENAI_BASE_URL honored before trust dialog — API token exfil"). So a keyless local endpoint for generated code is a real capability AND security-load-bearing; the security model is not an afterthought, it is the spine of the feature.

## Steal/build (3 legs)

LEG 1 — KEYLESS LOCAL INFERENCE BROKER FOR GENERATED CODE [S1, BUILD]. Source: Fable Claudeception ("never pass in an API key, this is handled"). ELAI port: a broker endpoint, local-only (127.0.0.1, never bound to a routable interface), that an ELAI-generated/run app calls WITHOUT any key; the broker forwards to the sidecar through the SHIPPED RuntimeOrchestrator / J-48 local serving tier router (so the generated app inherits ELAI's tier policy, concurrency caps, and VRAM scheduling — never a second inference path). The generated code sees a stable, keyless base_url (e.g. http://127.0.0.1:<broker-port>/v1/messages); the broker holds NO real cloud key and CANNOT reach the cloud lane (a generated app that wants cloud must be an explicit, trust-dialog-gated user action — LEG 3). This is the "AI-powered app out of the box on your own model" capability ELAI lacks.

LEG 2 — GENERATED-APP RUN/PREVIEW SEAM [S2, BUILD]. Source: Fable's artifact run + present_files. ELAI port: a minimal "run this generated app against the local broker" seam — the missing preview concept (grep-confirmed absent). Scope it as a CLI/IPC verb (`elai artifact run <dir>` or an IPC handler) that launches the generated app with ELAI_GENERATED_APP_BASE_URL pointed at the LEG-1 broker, inside the Tier-C process-isolation sandbox (NO net beyond the broker, NO host FS beyond the app dir, the 4A seccomp profile), and surfaces its output. This is deliberately MINIMAL (run + observe), NOT a claude.ai-grade artifact renderer — that is a far larger product surface and out of scope (see Open questions / fit hypothesis).

LEG 3 — SECURITY MODEL (the spine, not an add-on) [S3, BUILD, load-bearing]. Source: trust_dialog.rs CVE-2026-21852 + Tier-C 4A + G-1 taint. The broker and the run seam MUST: (a) bind 127.0.0.1 only, reject any non-loopback bind (a routable broker is a BLOCK); (b) hold NO real cloud key — the broker forwards only to the local sidecar; a generated app requesting the cloud lane triggers an EXPLICIT trust_dialog gate (reusing trust_dialog.rs), never a silent base_url honor (the exact CVE pattern the guard exists for); (c) run the generated app under Tier-C isolation (no net except the broker socket, no host FS except the app dir, seccomp); (d) taint-screen (G-1) any data the generated app sends to the broker so a generated app cannot exfil host secrets through prompts; (e) rate/quota the broker through J-48's admission so a runaway generated app cannot starve ELAI's own inference. Missing any of (a)-(e) is a BLOCK — this leg is the feature's reason it can ship at all.

## HARD boundaries (negative space)
- NEVER a second inference path: the broker forwards through RuntimeOrchestrator / J-48, never a direct sidecar socket the generated app owns (GD-1 — reuse the shipped router).
- NEVER a routable bind / NEVER a real cloud key in the broker / NEVER a silent cloud base_url honor (the trust_dialog CVE class) — cloud from a generated app is an explicit user-gated act only.
- NOT a claude.ai-grade artifact renderer: LEG 2 is run+observe minimal, not a sandboxed React/HTML live-preview product (out of scope; rider).
- NO new ELAI_*_ENABLE for the capability proper; the broker is OFF by default and activates only when a generated app is explicitly run (no graveyard flag — the activation IS the user action, W-2). A kill-switch ELAI_GENERATED_APP_BROKER_DISABLE hard-disables it.
- NO bypass of Tier-C isolation for the generated app (4A seccomp/no-net/path-jail stands; a generated app is UNTRUSTED code by definition).
- NO .unwrap()/.expect() on prod paths; blake3 only; the broker is local-only in-process at gate time (W-10), no external network.

## Existing ground truth (grounded this session; PLAN-asserted anchors, executor re-greps per W-7)
- SIDECAR + AnthropicClient WIRED — the sidecar speaks the Anthropic /v1/messages protocol at ANTHROPIC_BASE_URL=localhost:8100 (CLAUDE.md remote-sidecar canon); runtime_builder + AnthropicClient honor ANTHROPIC_BASE_URL (run_subcommand.rs "export ANTHROPIC_BASE_URL so the runtime_builder + AnthropicClient ..."); C-5 BYOE sets base_url from cloud.custom_endpoint. The broker forwards to THIS, never a new client.
- J-48 LOCAL SERVING TIER ROUTER PLANNED + RuntimeOrchestrator WIRED — runtime_orchestrator.rs:185 (per-role backend + fallback), cloud_mode.rs:284 (CloudModeRouter local-vs-cloud + spend caps), vram_tier.rs:6 (VramTier fit). LEG 1 routes the broker through these (per-tier concurrency, VRAM scheduling, I-7 spill) — do NOT re-implement serving policy (GD-1).
- trust_dialog.rs SECURITY GUARD WIRED — crates/safety/src/trust_dialog.rs: CVE-2026-21852 ("ANTHROPIC_BASE_URL / OPENAI_BASE_URL honored before trust dialog — API token exfil"), TrustDialogError::HookLoadDeferred when base_url is set before trust. LEG 3 REUSES this for the cloud-from-generated-app gate; the broker's keyless local-only design is the safe default the CVE guard implies. Do NOT weaken the guard (W-8 frozen-gate).
- TIER-C PROCESS ISOLATION WIRED-core — crates/agent + crates/ipc-types (phase-4A: minimal OS processes, NO net/LLM/tools, seccomp 2 profiles, SO_PEERCRED; B-18 allowed_paths PARTIAL). LEG 2 runs the generated app under this; do NOT author a new sandbox (GD-1).
- G-1 TAINT PROPAGATION WIRED — Provenanced<String> + capability_gate before destructive sinks. LEG 3 screens generated-app -> broker data through this.
- NO generated-app/preview/dev-server/run-artifact concept exists (grep this session = none) — LEG 2 is genuinely net-new; this also means the capability-FIT is a hypothesis (Open questions), since ELAI is a build-and-verify coding agent, not a chat-artifacts product.
- J-2 ELAI-AS-MCP-SERVER CONTROL PLANE PLANNED — the broker's verb (artifact run) can later surface as a J-2 control-plane verb; sequence after J-2 if exposed externally (local-only first).

## REVISED scope (W-10 tiered)
Tier-1 (SHIP NOW): LEG 1 the keyless local broker forwarding through RuntimeOrchestrator/J-48 (127.0.0.1 only, no cloud key) + LEG 3 the full security model (loopback-only bind, no-key, Tier-C isolation, G-1 taint, J-48 quota, trust_dialog cloud gate). LEG 2 a MINIMAL run-and-observe verb (`elai artifact run <dir>` against the broker, sandboxed). All in-process / loopback at gate time (W-10), no external network.
Tier-2 (DEFER): a claude.ai-grade live artifact renderer (HTML/React preview surface — large product, out of scope); cloud-lane access from a generated app beyond the single trust-gated path; a dashboard "run generated app" panel; persistent generated-app sessions. All recorded as riders.

## Integration
- NEW (runtime): crates/orchestrator/src/generated_app_broker.rs (~160 LOC: a loopback-only Anthropic-protocol broker forwarding to RuntimeOrchestrator/J-48; no cloud key; per-app quota through J-48 admission; G-1 taint screen on inbound). NEW: a run seam — crates/cli/src/artifact_run_cmd.rs (~90 LOC: launch the generated app under Tier-C with ELAI_GENERATED_APP_BASE_URL -> broker, surface output) OR an IPC handler if dashboard-driven.
- MODIFY: crates/cli/src/args.rs (+ `elai artifact run <dir>` subcommand); crates/safety/src/trust_dialog.rs is NOT modified (the cloud-from-generated-app gate REUSES it — W-8); the Tier-C sandbox launch is reused from crates/agent (NOT re-authored).
- SSB: +GeneratedAppInference (Pattern-2 single-variant; Kind: BrokerForwarded / TaintBlocked / QuotaThrottled / CloudGateRequested / CloudGateDenied). baseline+1; SCHEMA_VERSION COMPUTED (ssb.rs:1508 = DDL_EPOCH(-309)+SSB_ACTION_TYPES.len()); full 9-tuple lockstep; re-grep at edit (W-9).
- frontend_pair: a GeneratedAppPanel tile (broker forwards / taint blocks / quota throttles / cloud-gate prompts per session) on the Monitor family; #[tauri::command] + ts-rs; cross_lang_diff.py PASS. Justified: a generated app calling a local model is a security-sensitive surface the operator must SEE (the believe-the-number + the safety surface).

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/generated_app_broker.rs::GeneratedAppBroker (loopback-only keyless Anthropic-protocol broker -> RuntimeOrchestrator/J-48; no cloud key; G-1 taint screen; J-48 quota) [LEG 1 + LEG 3]
  - the `elai artifact run <dir>` CLI verb (Tier-C-sandboxed launch with ELAI_GENERATED_APP_BASE_URL -> broker) [LEG 2]
  - the trust_dialog-gated cloud-from-generated-app path (REUSES trust_dialog.rs, authors no new key handling) [LEG 3]
  - GeneratedAppInference SSB action_type
consumes_symbols:
  - crates/orchestrator/src/runtime_orchestrator.rs (per-role backend + fallback — the broker forwards here, never a new client) — GD-1
  - J-48 local serving tier router (concurrency/VRAM/admission — the broker rides it) — GD-1
  - the sidecar AnthropicClient / ANTHROPIC_BASE_URL wiring (run_subcommand.rs) — GD-1
  - crates/safety/src/trust_dialog.rs (CVE-2026-21852 gate for any cloud-from-generated-app request — REUSE, do NOT weaken) — GD-1 + W-8
  - crates/agent Tier-C isolation (4A seccomp/no-net/path-jail — the generated app runs here) — GD-1
  - G-1 Provenanced<String> taint (inbound generated-app data screen) — GD-1
reachable_from:
  - `elai artifact run <dir>` on a real generated app -> the app calls the broker -> GeneratedAppInference::BrokerForwarded, the sidecar produces a real completion the app consumes  [PRODUCTION CLI seam]
  - a cloud-lane request from a generated app -> CloudGateRequested -> trust_dialog -> CloudGateDenied/Allowed  [PRODUCTION trust-dialog seam]
  - a #[cfg(test)] caller NEVER satisfies this (R7); the SSB row + the generated app's consumed completion is the off-agent artifact
stub_ban:
  - NO routable bind (127.0.0.1 only; a non-loopback bind is a BLOCK)
  - NO real cloud key in the broker; NO silent cloud base_url honor (the CVE class — cloud is trust-dialog-gated only)
  - NO second inference path (forward through RuntimeOrchestrator/J-48, never a direct sidecar socket the app owns) — GD-1
  - NO running the generated app outside Tier-C isolation (untrusted code; 4A seccomp/no-net/path-jail mandatory)
  - NO un-screened generated-app -> broker data (G-1 taint; a host secret reaching the prompt is a BLOCK)
  - NO new ELAI_*_ENABLE for the capability (the user-run IS the activation, W-2); kill-switch ELAI_GENERATED_APP_BROKER_DISABLE only
  - NO .unwrap()/.expect() on prod paths; NO blake2/sha2 (blake3); NO silent fallback (a disabled broker logs the disable)
frontend_pair:
  GeneratedAppPanel tile (broker forwards / taint blocks / quota throttles / cloud-gate prompts per session); #[tauri::command] + ts-rs + cross_lang_diff.py PASS. Justified: a generated app calling a local model is security-sensitive; the operator must see when it fires and when the security legs trip.
observable_effect:
  trigger: "`elai artifact run <dir>` on a generated app that issues an Anthropic-protocol request to the broker; separately, a generated app requesting the cloud lane"
  expect_ssb: "GeneratedAppInference::BrokerForwarded carrying the app id + the routed local tier (or TaintBlocked / QuotaThrottled / CloudGateRequested->CloudGateDenied) — byte-recomputable from the journal under ELAI_SEED (GD-2)"
  expect_artifact: "the generated app receives a real sidecar completion through the broker and uses it (the app is AI-powered on the local model); the GeneratedAppPanel count incremented. NEGATIVE that must FAIL: a broker bound to a non-loopback interface (must BLOCK at startup); a generated app reaching the cloud lane WITHOUT a trust_dialog prompt (must BLOCK — the CVE); a generated app reading host FS outside its dir (Tier-C must BLOCK); a host secret in a generated-app prompt reaching the sidecar (G-1 must BLOCK); the broker a silent no-op when ELAI_GENERATED_APP_BROKER_DISABLE is set (must log)"

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/J-plan/J-63_local_lane_claudeception.md --run-trigger
rg -n "127\.0\.0\.1|loopback|is_loopback" .elai_cc/crates/orchestrator/src/generated_app_broker.rs   # loopback-only bind
rg -n "RuntimeOrchestrator|runtime_orchestrator" .elai_cc/crates/orchestrator/src/generated_app_broker.rs  # forwards through the router, no 2nd path
rg -n "trust_dialog|CVE-2026-21852" .elai_cc/crates/orchestrator/src/generated_app_broker.rs .elai_cc/crates/cli/src/artifact_run_cmd.rs  # cloud gate reuses the guard
rg -cn "api_key|cloud.*key|secret" .elai_cc/crates/orchestrator/src/generated_app_broker.rs   # broker holds NO real cloud key (manual review on hits)
rg -n "GeneratedAppInference" .elai_cc/crates/orchestrator/src/ssb.rs                  # the action type
rg -c "\.unwrap\(\)|\.expect\(|blake2|sha2" .elai_cc/crates/orchestrator/src/generated_app_broker.rs  # MUST be 0
test -f workflow/reasoning/J-63-decisions.md
```

## ACCEPTANCE
- [ ] LEG 1: the keyless loopback-only broker forwards a generated app's Anthropic-protocol request through RuntimeOrchestrator/J-48 to the sidecar; the app receives a real completion; no cloud key in the broker; no second inference path.
- [ ] LEG 2: `elai artifact run <dir>` launches the generated app under Tier-C isolation with ELAI_GENERATED_APP_BASE_URL -> broker, surfaces output (run+observe minimal, not a renderer).
- [ ] LEG 3: non-loopback bind rejected; cloud-from-generated-app is trust_dialog-gated (reuses trust_dialog.rs, CVE pattern blocked); generated app runs sandboxed (no net except broker, no host FS except app dir); G-1 taint screens inbound; J-48 quota throttles a runaway app.
- [ ] broker OFF by default (the user-run is the activation); kill-switch ELAI_GENERATED_APP_BROKER_DISABLE; a disabled broker logs the disable.
- [ ] SSB +1 GeneratedAppInference; SCHEMA_VERSION + variant total re-grepped at edit, 9-tuple lockstep.
- [ ] frontend: GeneratedAppPanel + #[tauri::command] + ts-rs; cross_lang_diff.py PASS.
- [ ] cargo check --workspace GREEN; auditor PASS on -p elai-orchestrator + -p elai-cli + -p elai-safety only.
- [ ] No new dependency (reuses AnthropicClient, RuntimeOrchestrator, Tier-C, trust_dialog, G-1).
- [ ] depends J-48 (serving router) + Tier-C 4A; sequence after J-48.

## WIRING DONE-GATE (clause by clause)
- W-1 missing evidence = FAIL: the GeneratedAppInference row is required. W-2 kill-switch-not-enable: the capability activates on the user-run (no enable-flag graveyard), ELAI_GENERATED_APP_BROKER_DISABLE hard-off. W-3 no opt-in on the security legs (loopback/no-key/sandbox run unconditionally). W-4 measure-the-objective: the artifact is the generated app consuming a real sidecar completion, not the broker claiming it forwarded. W-5 sound-reference: reachability is the real CLI run + the real trust-dialog seam. W-6: GeneratedAppPanel. W-7: anchors re-grepped. W-8 gate frozen: J-63 does NOT weaken trust_dialog.rs (it reuses the guard). W-9: SCHEMA re-grep at edit. W-10 integration: an independent integration smoke runs a real generated app through `elai artifact run`, asserts a real completion landed AND that the security NEGATIVES all BLOCK (non-loopback, cloud-without-gate, FS-escape, taint, runaway). GD-1: composes RuntimeOrchestrator/J-48/AnthropicClient/Tier-C/trust_dialog/G-1, restates none. GD-2: row recomputable under ELAI_SEED. GD-4: the broker ships behind the explicit user-run (no always-on gate to FP-bound); the security BLOCKS are fail-closed by design. GD-5: honest residue + the fit hypothesis below.

## Tests to add
- crates/contracts/tests/j63_generated_app_broker_contract.rs (~110 LOC): (a) generated-app request -> BrokerForwarded -> real completion shape; (b) non-loopback bind attempt -> startup BLOCK; (c) cloud-lane request -> CloudGateRequested -> trust_dialog -> CloudGateDenied without a prompt-bypass; (d) host-secret in prompt -> TaintBlocked (G-1); (e) runaway app -> QuotaThrottled (J-48); (f) FS-escape attempt under Tier-C -> BLOCK; (g) ELAI_GENERATED_APP_BROKER_DISABLE -> logs disable, no-op (not silent).
- a live integration smoke (W-10): `elai artifact run` on a tiny real generated app that calls the broker, asserting a real sidecar completion + all five security negatives BLOCK.

## Files touched
- CREATE (runtime): crates/orchestrator/src/generated_app_broker.rs (~160); crates/cli/src/artifact_run_cmd.rs (~90); crates/contracts/tests/j63_generated_app_broker_contract.rs; workflow/reasoning/J-63-decisions.md
- MODIFY (runtime): crates/cli/src/args.rs (+ `elai artifact run <dir>`); crates/orchestrator/src/ssb.rs (+1 action type lockstep); crates/orchestrator/tests/action_type_lock.rs (+1); the Monitor panel family (GeneratedAppPanel) + #[tauri::command] + ts-rs binding
- REUSE-NOT-MODIFY: crates/safety/src/trust_dialog.rs (W-8); crates/agent Tier-C; RuntimeOrchestrator; J-48 router; G-1 taint

## Citations / Grounding
- FABLE SOURCE (ILLUSTRATIVE, UNVERIFIED PROVENANCE): C:/Users/user/Downloads/CLAUDE-FABLE-5.md anthropic_api_in_artifacts ("Claudeception": artifacts call /v1/messages with no API key, "this is handled already"; web_search + MCP tools usable from the artifact). Leaked / jailbreak-reconstructed prompt (jailbreak-fable-main.zip co-located = jailbreak-extraction signature); the PATTERN (AI-powered generated artifact backed by a keyless inference endpoint) is corroborated by the public claude.ai artifacts feature, so the design survives the unverified provenance. The ELAI design is grounded in ELAI's own sidecar + security seams, NOT the leak; the LOCAL-keyless + security-spine framing is ELAI-specific (the leak is cloud-keyed).
- ELAI ground truth (grounded this session 2026-06-21, PLAN-asserted, re-grep W-7): the sidecar Anthropic /v1/messages at ANTHROPIC_BASE_URL=localhost:8100 (CLAUDE.md remote-sidecar canon) + AnthropicClient honoring it (run_subcommand.rs); RuntimeOrchestrator (runtime_orchestrator.rs:185) + CloudModeRouter (cloud_mode.rs:284) + VramTier (vram_tier.rs:6); J-48 local serving tier router (PLANNED); trust_dialog.rs CVE-2026-21852 (the base_url-exfil guard LEG 3 reuses); Tier-C 4A (crates/agent, seccomp/no-net); G-1 taint (Provenanced<String>); SCHEMA_VERSION at ssb.rs:1503/1508 (COMPUTED). Grep-confirmed: NO generated-app/preview/dev-server concept exists (LEG 2 net-new).
- doctrine: GD-1 compose-not-restate (forward through the shipped router/sandbox/guard, P0 if duplicated); W-8 frozen-gate (do NOT weaken trust_dialog.rs); W-2 kill-switch-not-enable; CLAUDE.md (blake3, no .unwrap in prod, scoped -p tests, no new deps, ID-5 no fabricated stats).
- spec-eval source-of-record: the Fable-steal verdict this session (decision-ledger topic fable-orchestration-steal); reproduce in workflow/reasoning/J-63-decisions.md at execution.
- SSB: +1; SCHEMA_VERSION COMPUTED -> baseline ~268 / variant-total ~577 per J-0_INDEX live re-grep; NEVER hard-code, re-grep BOTH at edit (W-9/R-c).

## Open questions / risks
- CAPABILITY-FIT HYPOTHESIS (the load-bearing risk, surfaced not hidden): ELAI is a build-and-verify CODING agent, not a chat-artifacts product. "Generated apps that depend on a running local sidecar" is a real capability but its product fit is a HYPOTHESIS — does the user actually want ELAI to ship apps coupled to the sidecar, vs apps that are self-contained with a documented optional LLM hook? Decide BEFORE building (a material fork -> AskUserQuestion / R-4). The Tier-1 broker is useful even if LEG 2's run seam is scoped down to "wire the keyless base_url into scaffolded apps + document it" with no ELAI-hosted runner.
- The generated app depending on a running sidecar is a deployment coupling: a shipped app calling 127.0.0.1:<broker> only works while ELAI/sidecar runs. The card/docs the app ships with must state this (the app is AI-powered IN ELAI's local env; export requires the user's own endpoint). Honest residue, not a blocker.
- Security review is mandatory (this introduces an inference endpoint reachable by untrusted generated code): the adversarial-auditor / security-review must clear LEG 3 before LEG 1/2 ship; the five negatives are the gate.
- vs J-2 control plane: if the run verb is ever exposed beyond local, it routes through J-2's DENY-ALL authz, never a second external surface; local-only first.

## Riders (recorded, not in-scope)
- A claude.ai-grade live artifact renderer (HTML/React sandboxed preview) — large product surface, Tier-2.
- Cloud-lane access from a generated app beyond the single trust-gated path — Tier-2 (cost + key-handling implications).
- A dashboard "run generated app" panel + persistent generated-app sessions — Tier-2 (rider on the dashboard family).
- web_search / MCP-tool access from a generated app (the Fable artifact-tools analog) — Tier-2, each its own taint + capability gate.

End of J-63.
