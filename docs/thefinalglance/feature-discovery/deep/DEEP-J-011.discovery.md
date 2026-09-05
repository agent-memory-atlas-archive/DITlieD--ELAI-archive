# DEEP-J-011 exhaustive feature discovery

## Scope, integrity, and method

- Processed manifest: `docs/thefinalglance/feature-discovery/DEEP-SOURCE-MANIFEST.md:L1037-L1052`, heading `### DEEP-J-011`; the next chunk starts at L1054.
- Declared chunk: family J, **10 files**, **1,225 logical lines**, **147,166 bytes**, all ten classified `numbered/index original`; target status **under 1,800**.
- Mechanical verification: fresh `sha256sum`, `wc -l`, and byte-size checks matched every manifest row and both aggregates exactly. Missing paths: **0**; unexpected paths: **0**; duplicate paths: **0**; SHA mismatches: **0**; line mismatches: **0**; byte mismatches: **0**.
- Read boundary: all ten listed originals were read completely, first through last logical line. No original assigned to another deep chunk was opened. `10-J-SUMMARY_INDEX.discovery.md` was consulted only as an orientation and delta-comparison artifact; no source atom below is derived solely from it.
- Anchor notation refers to the manifest-verified source images: `J-7:Lx-Ly`, `J-30:Lx-Ly`, `J-42:Lx-Ly`, `J-48:Lx-Ly`, `J-72:Lx-Ly`, `J-78:Lx-Ly`, `J-89:Lx-Ly`, `J-92:Lx-Ly`, `J-95:Lx-Ly`, and `J-108:Lx-Ly`.
- Repeated premise, revised-scope, wiring, grounding, composition, and acceptance language is collapsed into one atom with every load-bearing anchor. Interaction, ambiguity/conflict, and summary-delta records reference atoms and are not counted again as features.
- Classification: `E` = explicit source fact/requirement; `I` = bounded inference; `A` = unresolved ambiguity/choice; `C` = internal conflict or summary correction that must not be silently flattened. Domains: `STATUS`, `CAP`, `SURFACE`, `FLOW`, `STATE`, `DATA`, `CFG`, `INT`, `EVENT`, `SEC`, `PRIV`, `COMP`, `FAIL`, `RECOVERY`, `RETRY`, `DET`, `IDEM`, `CONC`, `OBS`, `TEST`, `LIMIT`, `DEP`, `DEFER`, `EVIDENCE`.

### Exact processed manifest and verification

| Manifest class | Lines | Bytes | SHA-256 | Exact absolute path | Verification |
|---|---:|---:|---|---|---|
| numbered/index original | 135 | 8,540 | 798be092b4525c7b4fc4cc36569350085f9890302c8c8820023fe48239b97919 | docs/thefinalglance/J-plan/J-7_ext_steal_intake_closeout.md | exact |
| numbered/index original | 222 | 16,207 | 6e8da5646f39b204daa54f1571b073cb36f0bc80aea8835ef22ae2039aeb8004 | docs/thefinalglance/J-plan/J-30_provider_byok_custom_endpoints.md | exact |
| numbered/index original | 61 | 13,618 | 5f313dcee6cdbc1a5e2bb347ebb8483f434a59ba7a2601e15b48a9e28869e33c | docs/thefinalglance/J-plan/J-42_within_step_distill_reset_reasoning.md | exact |
| numbered/index original | 232 | 19,923 | e4c748baeb0f130f289125f6ab5a0876eb30a1675156354fa17bbf227de0c3e9 | docs/thefinalglance/J-plan/J-48_local_serving_tier_router.md | exact |
| numbered/index original | 75 | 11,342 | a416badabaf6e5b2695c69a96acf17d8f5896c517c623837bc96011ad4f91780 | docs/thefinalglance/J-plan/J-72_indomain_retriever_adaptation.md | exact |
| numbered/index original | 72 | 11,663 | 47d53f8c1dc6895fc598476c7ea89fec082d3483d3264c6f0ff7c14d3997af77 | docs/thefinalglance/J-plan/J-78_context_access_spec.md | exact |
| numbered/index original | 79 | 11,197 | 56abf6becb4268ad2430304a900e38b5877d3ebdecf514cc245b948e4f8f2dfc | docs/thefinalglance/J-plan/J-89_objective_and_plan_dag_prevention_gate.md | exact |
| numbered/index original | 104 | 12,817 | 64401c209e23a7236590486fbc70cb91733f23d6df20109cd0e0110d6ddea4be | docs/thefinalglance/J-plan/J-92_orchestrator_fsm_dispatch_ordering.md | exact |
| numbered/index original | 120 | 18,203 | 7490b518892c6855a34e6315e02f7eddc398365751caac58a5b5f1aade2a7db3 | docs/thefinalglance/J-plan/J-95_context_activation_wires.md | exact |
| numbered/index original | 125 | 23,656 | 133d235652d0b88f393956ac3f5b264c16715970bec5de5c50a7321a26cdf084 | docs/thefinalglance/J-plan/J-108_retrieval_navigator_depth_operator.md | exact |
| **Aggregate** | **1,225** | **147,166** | — | **10 exact paths** | **exact** |

## Per-file atomic ledgers

### J-7 — ext-steal intake closeout

Source: `docs/thefinalglance/J-plan/J-7_ext_steal_intake_closeout.md` (135/135 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| J7-E001 | E/STATUS | J-7 is PLANNED WAVE 2 EXTENSION, harness-side, effort S (<2h), Track J, lever AUDITABLE, grounding STRONG; source is the 2026-06-10 ext-repo steal doc (SRC-2), not a Section B item; the unit is the approval surface pending user yes. | J-7:L1-L8 |
| J7-E002 | E/FLOW | Left ownerless, three concrete pending-yes actions and ~9 pointer-line insertions rot; J-7 bundles them as one session with a done-gate discharging both 2026-06-10 reasoning docs. | J-7:L9-L11 |
| J7-E003 | E/CAP | Action A: skillspector is the resolved scanner; smoke from clone `skillspector scan <skill> --no-llm -f json` (STATIC ONLY: deterministic, no API key, no egress except optional OSV.dev with silent offline fallback). | J-7:L14-L17 |
| J7-E004 | E/DATA | Parse findings into ONE TSV row in `workflow/skill-imports.log` per schema: ts, skill_name, skill_dir, scan_tool, findings_count, scan_hash, action. | J-7:L18-L19 |
| J7-E005 | E/DET | `scan_hash` = hash of scanner output; JC-2 governs (blake2b stdlib now, algorithm-prefixed; blake3 on dep approval). | J-7:L19-L21 |
| J7-E006 | E/CFG | Amend `.claude/rules/skill-import-policy.md` item 1: scanner RESOLVED = skillspector --no-llm (NVIDIA-org GitHub verifiable channel); mcp-scan demoted to optional second leg if dist-channel ever verifies. | J-7:L22-L24 |
| J7-E007 | E/LIMIT | Binding caveats: static precision MODERATE so findings are the reviewable artifact and 0-100 score is advisory only; score SATURATES (log findings_count + per-finding severities, never just score); executable detection is extension-only. | J-7:L24-L27 |
| J7-E008 | E/SEC | The mcp_rug_pull analyzer is a STUB (RP1-RP3 unimplemented); G-4 re-pin gate REMAINS the only MCP rug-pull coverage (named non-coverage, GD-5). | J-7:L27-L29 |
| J7-E009 | E/CAP | Action B: impeccable deterministic design lint intake; anti-slopsquat first via `npm view impeccable version dist-tags time` (recent publish date required). | J-7:L30-L32 |
| J7-E010 | E/TEST | Trial `npx impeccable detect --fast --json` over dashboard-v2 src; record findings count + runtime in the decisions doc. | J-7:L33-L34 |
| J7-E011 | E/SURFACE | Add ONE line to the dashboard-smoker checklist (dashboard-handoff skill check set): run impeccable --fast on changed `.svelte`/`.css`. | J-7:L35-L36 |
| J7-E012 | E/LIMIT | CLI-only use means no SKILL.md import and no skill-import scan triggers; HOLD stands on importing the 23-command impeccable skill itself (>3-verb scope-creep flag; if ever wanted: whole-dir scan + log row + approval first). | J-7:L36-L38 |
| J7-E013 | E/DATA | Action C: ~9 one-line GD-1 insertions of form `external prior-art: <repo path> -> <files>` into F-1, B-29/B-30/E-22, I-12/I-14/I-40, H-12/H-14; one line each, no scope changes to those plans. | J-7:L39-L43 |
| J7-E014 | E/LIMIT | Binding for A-C: STEAL METHOD requires later reimplementation to read named source files from the clones; a summary-only port is a W-3 violation; licenses Apache-2.0/MIT with attribution in any adapted-file header. | J-7:L44-L46 |
| J7-E015 | E/STATUS | skill-import-policy item 1 scanner slot is the documented unresolved gap (mcp-scan dist-channel UNVERIFIED). | J-7:L48-L50 |
| J7-E016 | E/DEP | `workflow/papers/skillspector-interface.md` exists and is the binding scoping source for action A. | J-7:L51-L52 |
| J7-E017 | E/DEP | Clones on disk at `D:\.liteGAME_workflow\workflow\external-skills\repos\` (SRC-2, verified 2026-06-10). | J-7:L53-L54 |
| J7-E018 | E/LIMIT | Declines already recorded (agentmemory, Agent-Reach, gajae wholesale) are NOT reopened (J-0 DECLINES). | J-7:L55-L56 |
| J7-E019 | E/DEFER | SHIP NOW: A+B+C; DEFER F-1 Tier-2 Rust reimplementation of skillspector pattern inventory and any impeccable rule-engine native port. | J-7:L58-L61 |
| J7-E020 | E/INT | Integration touch points: skill-imports.log (+1..2 rows); skill-import-policy.md item-1; dashboard-handoff checklist one line; 9 plan files one pointer each; frontend_pair N/A. | J-7:L63-L65 |
| J7-E021 | E/INT | Production reachability is the skill-import gate itself: every FUTURE external-skill import runs the resolved scanner (HARNESS PRODUCTION, JD-1). | J-7:L76-L78 |
| J7-E022 | E/LIMIT | stub_ban: no auto-trust of 0-100 score; no claim of MCP rug-pull coverage; no impeccable skill import; no new dep into this root (run from clone/npx); no summary-only future port. | J-7:L79-L83 |
| J7-E023 | E/OBS | Observable trigger: run A scan on an own skill + B npm-view+trial + insert C lines. Expect skill-imports.log row with findings_count+severities+action (ALLOWED or NEEDS_REVIEW); policy names skillspector resolved WITH rug-pull caveat; npm view recorded; 9 pointer lines grep-present. | J-7:L85-L91 |
| J7-E024 | E/FAIL | NEGATIVE: scanning a seeded malicious-pattern fixture yields HIGH/CRITICAL findings -> action=BLOCKED row and import refused (gate bites, R2). | J-7:L91-L93 |
| J7-E025 | E/TEST | DETERMINISTIC_CHECK greps skillspector + rug-pull in policy, tails skill-imports.log, greps `external prior-art` across plan docs (>=9), requires `workflow/reasoning/J-7-decisions.md`. | J-7:L95-L102 |
| J7-E026 | E/TEST | ACCEPTANCE checklist: scan --no-llm logged advisory score; policy resolution + stub non-coverage; impeccable npm-verified and skill HOLD; 9 pointer lines only; negative BLOCKED arm; no new dependency installed into root. | J-7:L104-L110 |
| J7-E027 | E/DATA | Files: MODIFY skill-import-policy item 1, skill-imports.log, dashboard-handoff checklist, 9 plan files (F-1, B-29, B-30, E-22, I-12, I-14, I-40, H-12, H-14); CREATE J-7-decisions.md. | J-7:L112-L117 |
| J7-E028 | E/EVIDENCE | Grounding: SRC-2 rev 2 + skillspector-interface.md; JC-2 hashing; J-0 DECLINES; licenses skillspector Apache-2.0, impeccable Apache-2.0. | J-7:L119-L122 |
| J7-E029 | E/FAIL | Platform needs (Python 3.12+, yara-python prebuilt on Windows, git on PATH, typer<0.24 pin): run from clone env; if env fight exceeds budget, log blocker honestly and stop (S-effort). | J-7:L124-L127 |
| J7-E030 | E/CONC | Target plan files are living documents under concurrent sessions — insert via small anchored edits, re-grep before each (W-7). | J-7:L128-L129 |
| J7-E031 | E/DEP | After discharge, J-5's port inherits the resolved scanner; six new lanes planned as J-1..J-6 plus J-7 intake. | J-7:L131-L134 |
| J7-A001 | A/STATUS | Header marks pending user yes as the approval surface; dependency/action authorization must be rechecked before execution. | J-7:L3-L5 |
| J7-A002 | A/CFG | Exact clone path is Windows-style `D:\...`; Linux workspace path resolution at edit is unspecified. | J-7:L53-L54 |

Category coverage: capabilities/CLI/checklist ✓; workflows/states ✓; schemas/log rows ✓; flags/config ✓; integrations/events (log, no SSB) ✓; security/permissions ✓; failures/block ✓; determinism ✓; observability/evidence ✓; tests/done gates ✓; constraints/non-goals ✓; dependencies/status/deferred ✓. No formal privacy/compliance policy, retry API, or idempotency key.

### J-30 — provider BYOK custom endpoints

Source: `docs/thefinalglance/J-plan/J-30_provider_byok_custom_endpoints.md` (222/222 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| J30-E001 | E/STATUS | J-30 is PLANNED WAVE GSW-2 EXTENSION of B-37/I-50: keys and endpoints ONLY; zero OAuth, zero new wire formats, zero dynamic catalogs; Track J, lever REACH+STEERING, grounding STRONG, effort M. | J-30:L1-L7 |
| J30-E002 | E/CAP | Premise: provider abstraction SHIPPED (B-37 RuntimeAdapter, 11 backends, PerRoleRuntime, factory) but BYOK ergonomics absent: env-only keys, no per-backend base-url beyond ANTHROPIC_BASE_URL, no declarative anthropic/openai-compat endpoints without Rust, no live-test configure, no turn-scoped backend lock. | J-30:L9-L20 |
| J30-E003 | E/CFG | T1 key store: two-tier per-provider lookup env var (exact name, untouched precedence) then key store file; default 0600 file under ELAI config dir (Unix); Windows restrictive ACL best-effort + documented caveat. | J-30:L23-L26 |
| J30-E004 | E/DET | Atomic write: tmp + rename + fsync (goose config/base.rs hygiene). | J-30:L26-L27 |
| J30-E005 | E/DEFER | OS-keyring backend OPTIONAL behind explicit user dep confirmation (keyring crate; anti-slopsquat at edit); file tier ships first with zero new deps. | J-30:L27-L29 |
| J30-E006 | E/SEC | Key material NEVER lands in SSB/journal/logs (I-50 doctrine, G-9). | J-30:L29-L30 |
| J30-E007 | E/DATA | T2 `custom_providers.toml`: {name, engine=anthropic-compat\|openai-compat, base_url, api_key_ref (env name or store key — never the key itself), models[], headers?, timeout_s?}. | J-30:L31-L33 |
| J30-E008 | E/INT | Loaded at startup, registered as dynamic backends through EXISTING B-37 factory/resolve path; covers vLLM/LiteLLM-proxy/self-hosted with zero new Rust per endpoint. | J-30:L33-L35 |
| J30-E009 | E/FAIL | openai-compat engine carries field-probe note (S-33): unsupported request fields detected/dropped per endpoint, never silently timing out. | J-30:L35-L37 |
| J30-E010 | E/CFG | TOML not YAML (house style; beside models.toml). | J-30:L37-L37 |
| J30-E011 | E/CFG | T3: per-backend base_url via config + env override for BUILT-IN backends; ANTHROPIC_BASE_URL keeps working unchanged. | J-30:L38-L40 |
| J30-E012 | E/DATA | T4: declarative descriptor per backend (required keys, optional keys, base-url support, secret flags) for CLI and panel; masked entry for secret-flagged values; NO oauth_flow/device_code_flow arms. | J-30:L41-L44 |
| J30-E013 | E/SURFACE | T5 CLI verb: extend existing configure surface if present else `elai providers configure`: select backend -> masked key entry -> optional base-url -> LIVE minimal test call -> atomic save ONLY on success. | J-30:L45-L48 |
| J30-E014 | E/SURFACE | Dashboard panel mirrors same descriptor-driven flow (thin: form + test button + status). | J-30:L48-L49 |
| J30-E015 | E/FLOW | On 4xx model-availability errors, suggest models from I-51 catalog (S-29 rider, optional). | J-30:L49-L50 |
| J30-E016 | E/SURFACE | UI/UX inspiration: OpenCode Providers list, per-provider Connect, env-detected Environment badge, live-verify status pill; LAYOUT + INTERACTION ONLY, clean-room ELAI-native code; NO OAuth/device-code/credential-replay copied. | J-30:L51-L59 |
| J30-E017 | E/OBS | T6 inventory status per backend: configured? (descriptor-satisfied), verified_at, models (from models.toml/custom_providers.toml), stale flag (verified_at older than threshold). | J-30:L60-L62 |
| J30-E018 | E/SURFACE | Inventory surfaced as RuntimePanel extension (B-37 panel), not a new window. | J-30:L62-L63 |
| J30-E019 | E/CONC | T7 turn-scoped backend lock: once RuntimeOrchestrator resolves a backend for a turn/sequence it is LOCKED until sequence completes; mid-sequence swap attempt refused + logged. | J-30:L64-L67 |
| J30-E020 | E/FLOW | Existing role routing (PerRoleRuntime, H-7/E-31) decides BETWEEN turns; EscalateToCloud remains sanctioned mid-task escalation and is NOT blocked by the lock (sequence boundary by definition). | J-30:L67-L69 |
| J30-E021 | E/DEFER | Riders recorded not tasks: S-25 Gemini wire optional I-49; S-26 retry/auth-refresh = I-10/I-50; S-27 dynamic catalog ALREADY-REJECTED; S-28 mTLS REJECT. | J-30:L70-L72 |
| J30-E022 | E/DEP | B-37 MULTI-RUNTIME BACKEND [WIRED]; I-50 OAuth stays I-50; I-51 models.toml; I-49 wire formats not duplicated. | J-30:L74-L85 |
| J30-E023 | E/EVENT | SSB SCHEMA_VERSION 234 / TOTAL_VARIANTS 543 at authoring; J-30 = +1 AFTER J-28; ProviderByokEvent +1 with full 6-part discipline. | J-30:L86-L87, L110-L110, L120-L121 |
| J30-E024 | E/EVENT | ProviderByokEvent Kind: KeyStored / EndpointVerified / CustomBackendRegistered; payload backend name + ok flags + ts — NEVER key material. | J-30:L120-L121 |
| J30-E025 | E/DEFER | Tier-1 ships T1-T4, T6 (fixture clock), T7, T5 against LOCAL sidecar anthropic-compat + fixture HTTP openai-compat; Tier-2 live third-party cloud verification deferred (user keys, on demand, never CI). | J-30:L93-L99 |
| J30-E026 | E/CAP | NEW modules: key_store.rs (~180 LOC), custom_backends.rs (~200 LOC), descriptor+inventory (~120), runtime_orchestrator lock (~40), CLI configure (~150), RuntimePanel+Tauri (~120). | J-30:L101-L111 |
| J30-E027 | E/INT | Production reachability: `elai providers configure` [CLI]; startup custom_providers.toml load -> factory registration [runtime]; RuntimePanel inventory/test commands [dashboard]; contracts-only never R7. | J-30:L128-L133 |
| J30-E028 | E/LIMIT | stub_ban: no key material in SSB/logs/errors/toml; no OAuth/device-code; no new wire adapters; no dynamic remote catalog; no save-without-successful-test; no silent vendor-proxy fallback; no unwrap/expect; no blake2/sha2; no new env gate. | J-30:L134-L141 |
| J30-E029 | E/FAIL | NEGATIVE: dead endpoint FAILS configure and nothing saved; contract greps journal/store for fixture key bytes and FAILS on hit; mid-sequence swap refused+logged; missing api_key_ref fails loudly at load, never silent skip. | J-30:L153-L157 |
| J30-E030 | E/TEST | DETERMINISTIC_CHECK: wiring_contract_check, ProviderByokEvent in ssb.rs, custom_providers loader exists, oauth/device_code count MUST be 0 in key_store, cross_lang_diff, J-30-decisions.md. | J-30:L159-L167 |
| J30-E031 | E/DEP | COMPOSES-WITH B-37, I-50, I-51, I-49, J-19 (vLLM first custom toml interop), H-7/E-31, J-28 usage ledger, E-33 config surfaces. | J-30:L196-L202 |
| J30-E032 | E/LIMIT | Lock vs streaming retries: provider retry is SAME backend; only cross-backend swaps refused (document in lock comment). | J-30:L212-L213 |
| J30-A001 | A/SURFACE | Whether a configure verb already exists on CLI (E-33): check at edit; extend never duplicate. | J-30:L207-L208 |
| J30-A002 | A/INT | Dynamic registration vs compile-time-only B-37 factory: if compile-time-only, T2 lands as startup registry layer ON resolve path (no parallel registry). | J-30:L209-L211 |
| J30-A003 | A/CFG | Inventory stale threshold value and Windows ACL exact strength remain unspecified beyond best-effort caveat. | J-30:L60-L62, L205-L206 |
| J30-I001 | I/SURFACE | First-run onboarding is named as a panel consumer of OpenCode layout inspiration but no separate onboarding route/flag is specified beyond the RuntimePanel extension. | J-30:L51-L59, L109-L109 |

Category coverage: capabilities/CLI/panel ✓; workflows/states ✓; schemas/data ✓; flags/config ✓; integrations/events ✓; security/key material ✓; failures/fail-closed ✓; concurrency/lock ✓; observability ✓; tests ✓; constraints ✓; dependencies/deferred ✓. Formal privacy policy beyond key-in-log ban is I-50 doctrine; no retry API owned here.

### J-42 — within-step distill-reset reasoning

Source: `docs/thefinalglance/J-plan/J-42_within_step_distill_reset_reasoning.md` (61/61 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| J42-E001 | E/STATUS | J-42 Track J, lever REASON-DETERMINISM + local-economy, nature build extending GRC reset_topic + B-39 refine, effort M, WAVE DD-4; depends GRC-2A, B-39, E-25/H-50, D-16, H-42. | J-42:L3-L3 |
| J42-E002 | E/CAP | Cog invariant: model generates reasoning segment AND proposes compact progress-distillate; DETERMINISTIC trigger decides WHEN to checkpoint; FROZEN admission decides distillate acceptability; harness RESETS working context and re-seeds with {pinned original goal + admitted distillate}. | J-42:L4-L4 |
| J42-E003 | E/STATUS | GROUNDED ABSENT: ELAI does reason-on-distillate at agent/orchestration grain and refines by APPENDING (B-39), but no within-step distill->reset->re-reason-fresh operator; GRC reset_topic clears budget/step counters only. | J-42:L4-L4 |
| J42-E004 | E/TEST | EXP-3 OBSERVE-ONLY step 1: on local-model reasoning benchmarks (MATH500/AIME24/GPQA_diamond + coding slice) with 3B/7B, compare straight-line vs distill-reset at EQUAL total token budget. | J-42:L7-L7 |
| J42-E005 | E/CFG | Win = distill-reset >= straight-line accuracy at LOWER peak context AND no abstention/uncertainty degradation; accuracy <= straight-line OR degraded abstention stays observe-only / context-economy-only. No default-ON until FP/over-reset and abstention-quality bounded on >=3 runs. | J-42:L7-L7 |
| J42-E006 | E/CAP | `checkpoint_and_reset(topic)` extends GRC reset_topic: capture model-proposed distillate, frozen admission, clear topic budget/steps, re-seed with pinned goal + admitted distillate. | J-42:L8-L8 |
| J42-E007 | E/DET | Deterministic trigger: fire when topic working-context fill crosses fraction of lane ceiling (ELAI_LOCAL_MAX_CONTEXT * trim_fraction, context_policy.rs:70 / H-50) OR GRC topic budget (max_chars/max_calls) nears exhaustion; reuses H-42 ReasoningBudgetScaled. NEVER model saying "I'll summarize now." | J-42:L9-L9 |
| J42-E008 | E/SEC | Frozen distillate admission (mandatory cog gate): model proposes FREE-FORM text; HD-3 BANS constrained-decode/force-JSON of reasoning-role output; harness parses best-effort (chat_reflect verdict-parser idiom). | J-42:L10-L10 |
| J42-E009 | E/FAIL | Admit iff: bounded to sawtooth floor size cap; non-empty; AND uncertainty-preserved (open questions/hedges/abstention markers from pre-reset trace survive, cross-checked D-16 RefusalDecision/AbstentionScored). Failure -> re-distill once, else escalate (I-7) / fall back to straight-line truncation. | J-42:L10-L10 |
| J42-E010 | E/DET | Pin ORIGINAL goal in every re-seed (not only rolling distillate) to resist iterative-summarization directional-attractor drift (2601.08846). | J-42:L10-L10 |
| J42-E011 | E/FLOW | Host loop: B-39 chat_reflect AUDITOR refine currently APPENDS prior verdict; J-42 adds distill-reset MODE as deep/local path: checkpoint_and_reset between passes instead of append; also wire into PipelineGrc hot paths and planning/research step loops. | J-42:L11-L11 |
| J42-E012 | E/LIMIT | EXCLUDED: SFT on InftyThink-reconstructed datasets (RUNPOD-PENDING); population-aggregation (RSA/B-39 N=4 parallel sibling); constrained-decode of reasoning or distillate; any model-decided reset trigger. | J-42:L12-L12 |
| J42-E013 | E/DEP | Seam: GrcEngine::reset_topic engine.rs:556 (trait :35); orchestrator wrapper grc.rs:194/:199. ChatReflectPipeline refine chat_reflect.rs:409-465. CLOUD_TRIM_FRACTION=0.85 context_policy.rs:70. | J-42:L16-L19 |
| J42-E014 | E/OBS | ReasoningTraceEntry written to SSB per GRC step; recovery builds context from checkpoint — the trace the distillate compresses already exists. | J-42:L18-L18 |
| J42-E015 | E/EVENT | +ReasoningCheckpointEvent (Pattern-2 single-variant); Kind 3-arm: ResetAndContinued \| DistillateRejected \| EscalatedToCloud; payload {trigger: context_fill\|grc_budget, distillate_tokens, uncertainty_preserved, pass_index, outcome}. | J-42:L33-L34 |
| J42-E016 | E/CFG | baseline+1 from LIVE SCHEMA 268 / SSB_ACTION_TYPES 577 at authoring, COMPUTED at implementation NEVER hard-coded, SEQUENTIAL after any DD-wave sibling that ships first (W-9); default-ON observe-only emit preferred. | J-42:L34-L34 |
| J42-E017 | E/INT | reachable_from: GRC reasoning-step path in PipelineGrc; chat_reflect AUDITOR refine; /reason command path; planning/research step loops. | J-42:L31-L31 |
| J42-E018 | E/LIMIT | stub_ban: no unwrap; no constrained-decode of distillate (HD-3); no silent SKIP on failed admission (re-distill-once-then-escalate); no model-decided reset trigger. | J-42:L32-L32 |
| J42-E019 | E/LIMIT | Honest residue: distill-reset makes small local model FIT more depth into bounded context, does NOT make it CAPABLE beyond reach (4B ~0% agentic); hard reasoning still escalates via I-7. | J-42:L26-L26 |
| J42-E020 | E/LIMIT | Second residue 2603.24472: self-distillation can suppress epistemic uncertainty and degrade OOD up to 40%; do NOT ship default-ON if abstention degrades. | J-42:L26-L26 |
| J42-E021 | E/EVIDENCE | Keystone InftyThink arXiv:2503.06692 VERIFIED; RSA 2509.26626; self-distill 2603.24472; attractors 2601.08846. | J-42:L36-L40 |
| J42-E022 | E/DEP | COMPOSES-WITH GRC 2A, B-39, E-25/H-50, H-42, D-16, I-7, J-39/J-40 DD triad, ELAI-DETERMINISM-DOCTRINE. | J-42:L43-L51 |
| J42-E023 | E/CFG | Cap reset-pass count (depth cap like J-39 MAX_SPLIT_DEPTH) and require measurable per-pass progress or escalate. | J-42:L56-L56 |
| J42-E024 | E/CFG | J-42 reset is LOCAL-lane-first; on cloud must respect keep-cache-prefix (likely append-mode on cloud, reset-mode only local) — mirrors H-50 opposite-policies. | J-42:L57-L57 |
| J42-A001 | A/CFG | Exact sawtooth floor size cap and pass-count depth default are not numeric. | J-42:L10-L10, L56-L56 |
| J42-A002 | A/CFG | Cloud vs local append/reset split "Decide the split at edit". | J-42:L57-L57 |
| J42-A003 | A/TEST | Uncertainty-preservation check is heuristic; real guard is EXP-3 abstention-quality measurement. | J-42:L55-L55 |
| J42-C001 | C/CFG | Summary and source both note cloud may keep cache/append while local resets; exact split remains open, not a hard ship default for cloud reset. | J-42:L57-L57 |

Category coverage: capabilities/reasoning ops ✓; workflows/states ✓; events/payload ✓; config/triggers ✓; integrations ✓; security (HD-3) ✓; failures/escalation ✓; determinism ✓; observability ✓; tests/EXP-3 ✓; constraints/deferred ✓. No standalone UI surface beyond `/reason` path; no formal privacy rule.

### J-48 — local serving tier router

Source: `docs/thefinalglance/J-plan/J-48_local_serving_tier_router.md` (232/232 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| J48-E001 | E/STATUS | J-48 PLANNED WAVE LSW-1 effort M BUILD — outer policy layer ONLY; engine owns fine scheduling. | J-48:L1-L3 |
| J48-E002 | E/CAP | LocalServingRouter owns: (1) route-by-tier with per-tier max-concurrency cap; (2) warm/cold whole-model lifecycle (pin small tiers; big tier EXCLUSIVE); (3) busy/cold cloud-spill via I-7; (4) coarse FIFO admission with at most 3 priority classes. | J-48:L9-L12 |
| J48-E003 | E/LIMIT | HARD BOUNDARY: NO continuous batching, chunked prefill, KV/paged-cache, token-level preemption, MLFQ re-order by estimated tokens (engine = J-19 vLLM). | J-48:L13-L16 |
| J48-E004 | E/DEP | GD-1 do-NOT-re-implement: J-19 engine; J-9 VRAM catalog; CloudModeRouter local/cloud + spend caps; I-7 waterfall. | J-48:L16-L18 |
| J48-E005 | E/STATUS | In-process for policy+admission; live multi-model smoke remote-sidecar/RunPod-gated (W-10) needing >=2 resident models. | J-48:L19-L21 |
| J48-E006 | E/DEP | Prerequisites SHIPPED: CloudModeRouter, RuntimeOrchestrator, VramTier, I-7; J-19/J-9 PLANNED — degrades gracefully over whatever backends exist. | J-48:L22-L26 |
| J48-E007 | E/DEP | Parallel-safe with J-49 (per-seat pre-filter in front of decide); author J-48 contract first. | J-48:L27-L28 |
| J48-E008 | E/FLOW | Router is stateless decision over (requested tier, live residency, per-tier inflight, spend snapshot) returning ServeLocal(tier) \| Wait \| Hibernate(small)+ServeLocal(big) \| SpillToCloud. | J-48:L42-L44 |
| J48-E009 | E/CAP | Task 1: LocalServingRouter in local_serving.rs; decide() inputs (AgentRole/tier, TierResidency snapshot, SpendSnapshotInput); returns LocalServingDecision { ServeLocal{tier}, AdmitQueued{tier, class}, HibernateThenServe{evict, serve}, SpillToCloud{reason} }. | J-48:L81-L85 |
| J48-E010 | E/CFG | Per-tier max-concurrency caps + warm-set max are NAMED constants (never magic numbers); decide() never sees prompt body, only counts and tier ids. | J-48:L86-L88 |
| J48-E011 | E/STATE | Task 2: TierResidency tracker RwLock<BTreeMap<VramTier, TierState>> on RuntimeOrchestrator (warm/cold + inflight, bump on dispatch, decrement on completion). | J-48:L89-L91 |
| J48-E012 | E/CONC | EXCLUSIVE-mode invariant: big tier warm => small tiers forced cold (encode in type, not ad-hoc if). | J-48:L91-L92 |
| J48-E013 | E/INT | Task 3: wire decide() as PRE-STEP inside RuntimeOrchestrator::try_complete BEFORE fallback chain; SpillToCloud hands to I-7/CloudModeRouter; AdmitQueued coarse FIFO <=3 NamedClass. Production caller (D-030). | J-48:L93-L97 |
| J48-E014 | E/EVENT | Task 4: prefer-reuse RouteDecisionEvent (E-31, 3-arm) via existing wire-dark gate; only add LocalServingDecisionEvent if 3 arms cannot carry {tier, decision, reason} (net 0..+1 SSB). | J-48:L98-L100 |
| J48-E015 | E/CFG | No new ELAI_*_ENABLE; ships ON; reuses E-31 gate only. | J-48:L114-L114, L176-L176 |
| J48-E016 | E/SURFACE | frontend_pair N/A: live serving gauge + tier warm/cold + spilled is J-29 UsageWindow territory; J-48 emits audit row J-29 consumes; no new Tauri command. | J-48:L150-L153 |
| J48-E017 | E/OBS | expect_ssb: RouteDecisionEvent or LocalServingDecisionEvent with decision=SpillToCloud and non-empty reason when big tier cold/busy and cloud cap available. | J-48:L158-L159 |
| J48-E018 | E/FAIL | NEGATIVE: SpillToCloud with empty reason OR local serve exceeding per-tier concurrency cap is contract failure. | J-48:L160-L162 |
| J48-E019 | E/TEST | j48_local_serving_contract.rs: ServeLocal under cap; AdmitQueued at cap; HibernateThenServe EXCLUSIVE; SpillToCloud non-empty reason when big cold/busy AND cloud cap; NEVER spill when air_gapped; negative inflight degrades to safe Wait never panic. | J-48:L197-L202 |
| J48-E020 | E/TEST | ACCEPTANCE: W-1 real try_complete row not build-green; W-2 no new enable flag; R7 decide from try_complete; hard boundary rg count 0 on batch/KV/preempt; GD-1 consume not restate; cargo check green; no new dep. | J-48:L174-L181 |
| J48-E021 | E/DATA | Files: CREATE local_serving.rs; MODIFY runtime_orchestrator.rs TierResidency+pre-step; MODIFY lib.rs mod; CREATE contract; CREATE J-48-decisions.md. | J-48:L204-L209 |
| J48-E022 | E/EVIDENCE | Warm/cold numbers directional/HD-1 (32B-4bit@32K ~24GB; model-swap ~30-120s); NOT measured on box; ship permissive defaults until measured (GD-5). | J-48:L37-L41, L123-L123, L224-L225 |
| J48-E023 | E/LIMIT | ABORT: continuous-batching/KV/preempt/MLFQ in local_serving; re-implement CloudModeRouter/I-7; magic-number caps; empty SpillToCloud reason; decide() test-only reachable. | J-48:L124-L124 |
| J48-E024 | E/CFG | GD-4: spill decision defaults ADVISORY-then-active only after FP/friction observed; caps/warm-set start permissive. | J-48:L191-L194 |
| J48-A001 | A/EVENT | Whether RouteDecisionEvent 3-arm can carry local serving decision or needs new variant — decide at edit. | J-48:L220-L221 |
| J48-A002 | A/CONC | EXCLUSIVE eviction when two big-tier requests queue: start strict FIFO + single big-tier slot; revisit only if measured contention warrants. | J-48:L222-L223 |
| J48-I001 | I/CFG | Summary "Treat 32B as exclusive" is topology-motivated EXCLUSIVE big-tier policy; exact tier enum names for "big"/"small" are left to VramTier composition at edit. | J-48:L37-L41, L91-L92 |

Category coverage: capabilities/router ✓; workflows/states ✓; config/named consts ✓; integrations/events ✓; concurrency/admission ✓; failures ✓; observability ✓; tests ✓; surfaces (explicit N/A to J-29) ✓; deferred/smoke tiers ✓. No privacy policy; no keystore.

### J-72 — in-domain retriever adaptation

Source: `docs/thefinalglance/J-plan/J-72_indomain_retriever_adaptation.md` (75/75 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| J72-E001 | E/STATUS | J-72 KEYSTONE of context-retrieval wave CR-1; BUILD offline pipeline + SWAP inference artifact; effort M-L; depends none; EXTENDS J-18; grounding STRONG (MCompassRAG / EvoEmbedding / CodeRAG-system). | J-72:L1-L5 |
| J72-E002 | E/CAP | Variant a encoder path: length-weighted InfoNCE (tau=0.1) fine-tune of code embedder on ELAI issue-PR-edit mapped to EvoTrain-180K schema -> INT8 ONNX export exactly as J-18 CodeRankEmbed swap (same usearch HNSW slot, index_version bump + re-embed, no geometry change). FOLD INTO J-18, never duplicate embedder slot. | J-72:L8-L8 |
| J72-E003 | E/CAP | Variant b scoring head: distill LLM teacher offline (~$10-20 for ~20K judgments) into ~2M-param 3-layer head on FROZEN ONNX embedder; REPLACES ms-marco cross-encoder reranker at embedding latency, ZERO LLM calls in hot path; tiny ONNX module, CPU, deterministic. | J-72:L9-L9 |
| J72-E004 | E/DATA | GOLD SUPERVISION SET from ELAI issue/PR/edit pairs + sealed-RRA agent traces; done-gate is ELAI-MEASURED MRR/Recall@k delta on gold set, NEVER paper number (ID-5). | J-72:L10-L10 |
| J72-E005 | E/LIMIT | EXCLUDED: KaLM online GPU decoder (R1); EvoRAG memory-queue/LoRA/flash-attn runtime (R2 — only contrastive loss+schema survive); training infra in-product (ELAI ships NO in-product training). | J-72:L11-L11 |
| J72-E006 | E/DEP | VECTOR SEARCH [WIRED] dual-model ONNX, usearch HNSW, blake3 cache, RRF k=60, index_version; CROSS-ENCODER RERANKER [WIRED] ms-marco MiniLM ONNX CPU; J-18 CodeRankEmbed [PLANNED]; G-2 RRA [WIRED]. | J-72:L13-L17 |
| J72-E007 | E/DEFER | Tier-1 in-product model-free: ONNX load path + `elai retrieval gold-build` + `elai bench retrieval-adapt` (deterministic, never LLM-judge). Tier-2 out-of-band GPU fine-tune/distill RunPod/remote-gated; only ONNX artifact re-enters product. | J-72:L19-L21 |
| J72-E008 | E/SURFACE | CLI verbs: `elai retrieval gold-build` (assemble triples from sealed RRAs + issue/PR/edit) and `elai bench retrieval-adapt` (MRR/Recall@k delta). | J-72:L26-L26, L35-L36 |
| J72-E009 | E/SURFACE | frontend_pair: MRR/Recall-delta breakdown on existing KnowledgeDrawer (B-29) or W-6-justified absence (decide at edit). | J-72:L29-L29, L53-L53 |
| J72-E010 | E/EVENT | PREFER-REUSE J-18 RetrievalAugmentEvent; Kind EmbeddingModelSwapped covers adapted-model swap; add Kind ScoringHeadDistilled if head ships; net 0 if J-18 unshipped else baseline+1 from LIVE ~269/578. | J-72:L28-L28 |
| J72-E011 | E/INT | Production: real elai retrieval query through HYBRID cascade adapted encoder+head; CLI gold-build and bench; contracts never R7. | J-72:L42-L45 |
| J72-E012 | E/LIMIT | stub_ban: no in-product training; no paper number done-gate (CORE-Bench numbers UNVERIFIED/not-found — must NOT cite 20.3/32.8/60.1); no unofficial community ONNX quant; no embedder swap without index_version bump+re-embed; no KaLM/EvoRAG runtime; no new default-OFF flag without allowlist+promotion. | J-72:L46-L52 |
| J72-E013 | E/FAIL | NEGATIVE: dim/geometry mismatch FAILS index_version migration never silent corrupt; missing adapted .onnx DEGRADES to current slot (cascade intact); scoring head shape mismatch FAILS at load never rank-k noise. | J-72:L57-L57 |
| J72-E014 | E/EVIDENCE | W1 WARNING: report CORE-Bench headline NOT FOUND as paper; numbers UNVERIFIED/possibly synthesized; lever stands on 3 primary sources. | J-72:L63-L63 |
| J72-E015 | E/DEP | COMPOSES-WITH J-18 (ships AFTER or WITH it), G-2 RRA, usearch+RRF unchanged, ms-marco seam replaced by head, J-75 ablation harness. | J-72:L67-L68 |
| J72-E016 | E/FAIL | Gold set bottleneck: need large/clean enough corpus; held-out split + J-75 ablation showing adapted beats stock else don't ship swap. | J-72:L70-L71 |
| J72-E017 | E/CFG | Variants independently shippable; head cheaper/reversible (rerank swap not index rebuild) — decide order at edit. | J-72:L72-L72 |
| J72-E018 | E/INT | G-10 model-drift canary must invalidate adapted artifact when corpus identity changes (compose, not re-author). | J-72:L73-L73 |
| J72-A001 | A/CFG | Which variant lands first and exact gold-set size/quality thresholds unspecified. | J-72:L70-L72 |
| J72-A002 | A/SURFACE | KnowledgeDrawer MRR/Recall UI vs W-6 absence decide at edit. | J-72:L53-L53 |

Category coverage: capabilities/CLI/bench ✓; workflows offline/online ✓; data/gold set ✓; config/export ✓; events ✓; failures/degrade ✓; determinism ✓; observability ✓; surfaces ✓; deferred GPU ✓. Privacy: RRA seals as supervision source only.

### J-78 — context-access spec

Source: `docs/thefinalglance/J-plan/J-78_context_access_spec.md` (72/72 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| J78-E001 | E/STATUS | J-78 EXTENDS I-5; WAVE VO-2 effort M; lever CANT-MISREAD; grounding STRONG (Conductor arXiv:2512.04388 access_list). | J-78:L1-L5 |
| J78-E002 | E/CAP | Per-dispatch access_spec names which prior SSB rows / sibling outputs / memory namespaces the dispatch MAY read; orchestrator builds context from access_spec ONLY; read outside spec is violation not silent inclusion. | J-78:L8-L8 |
| J78-E003 | E/OBS | JOURNALED + RECOMPUTABLE (GD-2): access_spec journaled so worker context reproducible under ELAI_SEED; RRA can show allowed rows X,Y,Z and no others. | J-78:L9-L9 |
| J78-E004 | E/DET | DETERMINISTIC (TD-3): access_spec computed by orchestrator pure function of dispatch role + sub-phase + I-5/firewall rules, NOT proposed by LLM. Conductor learns access_list; ELAI declares it. | J-78:L10-L10 |
| J78-E005 | E/LIMIT | EXCLUDED: TRINITY latent hidden-state routing (O5 REJECTED); LEARNED router proposing access_list; new orchestration topology engine (I-4 answers novel topology). | J-78:L11-L11 |
| J78-E006 | E/DEP | Composes I-5 pristine hard-exclude, H-31 auditor ZERO memory, 3E context_budget 8 slots (SsbEntries/GrcHistory/CodeContext), orchestrate/auto-orchestrate habit, SSB journal. | J-78:L13-L18 |
| J78-E007 | E/CFG | Tier-1: access_spec struct + context-build-from-spec-only + journaled row + contract that dispatch cannot read outside spec. OBSERVE-ONLY first (GD-4): journal + flag would-be out-of-spec reads before hard-enforce until friction/FP bounded on >=3 real orchestrate runs. No Tier-2 GPU. | J-78:L20-L22 |
| J78-E008 | E/EVENT | +ContextAccessSpecEvent Kind: SpecDeclared / OutOfSpecReadFlagged / SpecEnforced; baseline+1 from LIVE ~269/578 OR prefer-reuse agent-dispatch row field (net 0) — decide at edit. | J-78:L28-L28 |
| J78-E009 | E/SURFACE | frontend_pair: per-dispatch access-spec on agent-pool / orchestrate surface or W-6-justified absence (decide at edit). | J-78:L29-L29, L50-L50 |
| J78-E010 | E/INT | Production: real elai orchestrate/auto-orchestrate/task dispatch building worker context from access_spec; contracts never R7. | J-78:L40-L42 |
| J78-E011 | E/LIMIT | stub_ban: no LLM-proposed access_spec; no latent routing; no silent inclusion outside spec; no re-author I-5/H-31; no enforced block before FP bounded >=3 runs; no new default-OFF without allowlist. | J-78:L43-L49 |
| J78-E012 | E/OBS | Trigger: orchestrate dispatching auditor (zero memory) and executor after failed sibling (exclude traceback). expect_ssb SpecDeclared per dispatch; OutOfSpecReadFlagged on violation. | J-78:L51-L53 |
| J78-E013 | E/FAIL | NEGATIVE: missing/empty access_spec gets MINIMAL safe context not full dump; out-of-spec first flagged (OBSERVE) then blocked (ENFORCE), never silently allowed. | J-78:L54-L54 |
| J78-E014 | E/EVIDENCE | Conductor access_list is the steal (O1); TRINITY O5 rejected; O2 Thinker/Worker/Verifier confirms I-21; O3 novel topology -> I-4; O4 GTM not code. | J-78:L56-L61 |
| J78-E015 | E/EVIDENCE | Premise "5x slower AND 5x pricier" REFUTED: Conductor 735 vs 1412 tok/sample more efficient; paddo.dev build cheaper+faster than Opus 4.8 on cost/speed. | J-78:L62-L62 |
| J78-E016 | E/CFG | Spec should be ROLE+SUB-PHASE-derived by default with explicit overrides only where I-5/firewall already differ; measure friction before enforce. | J-78:L68-L69 |
| J78-A001 | A/EVENT | prefer-reuse vs new SSB ContextAccessSpecEvent decide at edit. | J-78:L70-L70 |
| J78-A002 | A/SURFACE | Exact agent-pool/orchestrate UI chrome decide at edit vs W-6 absence. | J-78:L50-L50 |
| J78-A003 | A/CFG | Granularity of access_spec (row-level vs role-derived) measured not fixed. | J-78:L68-L69 |

Category coverage: capabilities/access contract ✓; workflows/dispatch ✓; events ✓; security (auditor zero-memory) ✓; failures ✓; determinism ✓; observability/RRA ✓; surfaces ✓; deferred enforce ✓. No CLI verb beyond orchestrate production path.

### J-89 — objective + plan-DAG prevention gate

Source: `docs/thefinalglance/J-plan/J-89_objective_and_plan_dag_prevention_gate.md` (79/79 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| J89-E001 | E/STATUS | J-89 RUTHLESS-3 prevention pair L1+L8; BUILD effort L WAVE 1; depends J-34 for L1 escalation ENFORCEMENT half (MECHANISM ships now). | J-89:L1-L3 |
| J89-E002 | E/CAP | LEG A L1: force every objective into TYPED SPEC BUNDLE (purpose, scope, domain assumptions, non-functional constraints, competency questions, obstacle log); promote advisory spec-grounder/grounding-sweep to first-class TAINT-TRACKED evidence-bound artifact with DETERMINISTIC coverage checks. | J-89:L8-L8 |
| J89-E003 | E/CAP | CROSS-FAMILY ADVERSARIAL SPEC JURY from DISJOINT families attacking OBJECTIVE (omission, ambiguity, harmful readings, proxy-gaming), DISTINCT from auditor; substrate D-13 + gpt-bridge. | J-89:L10-L10 |
| J89-E004 | E/LIMIT | Jury is the ONE place an LLM enters the whole program — legal only at PLANNING (off hot path); output ADVISORY-by-construction ("survived N challenges", escalate-on-disagreement), NEVER deterministic verdict (TD-3). | J-89:L10-L10 |
| J89-E005 | E/CAP | SPAN-LINKED AUTOFORMALIZATION of highest-risk clauses ONLY (nl2spec/SYNTHTL); not whole-spec. | J-89:L11-L11 |
| J89-E006 | E/STATE | ATTESTATION = I-45 + G-8 as FREEZE step; mutation after PASS -> detectably stale receipt. | J-89:L12-L12 |
| J89-E007 | E/LIMIT | EXCLUDED: jury deterministic gate verdict; whole-spec autoformalization; claim of "correct objective" (only survived N challenges + frozen attestation). | J-89:L13-L13 |
| J89-E008 | E/CAP | LEG B L8: every plan node TYPED signature (required inputs, produced outputs, consumed assumptions, emitted obligations); three DETERMINISTIC checks BEFORE code: GOAL COVERAGE, ARTIFACT REACHABILITY, PROTOCOL REALIZABILITY. | J-89:L17-L17 |
| J89-E009 | E/INT | Lift wiring_contract_check.py producer/consumer symmetry to PLAN-DAG time via plan_dag_check.py; substrate BabyStepSpec 8-block, C-29, Kahn cycle detection. | J-89:L17-L17, L30-L30 |
| J89-E010 | E/LIMIT | EXCLUDED: claim of SEMANTIC completeness (L8 is STRUCTURAL only); new symmetry engine (lift existing). | J-89:L18-L18 |
| J89-E011 | E/CAP | produces: build_spec_bundle, check_bundle_coverage, run_spec_jury; plan_dag_check {check_goal_coverage, check_artifact_reachability, check_protocol_realizability}. | J-89:L35-L39 |
| J89-E012 | E/EVENT | +SpecBundleEvent Kind: BundleSealed / JuryChallengeSurvived / JuryEscalated; or reuse I-45 ObjectiveSealed (prefer-reuse, decide at edit); net 0..1 from LIVE ~268/577. | J-89:L29-L29 |
| J89-E013 | E/CFG | FEATURE FLAGS: ELAI_SPEC_BUNDLE_GATE_ENABLE (kill-switch; mechanism ships; jury-escalation ENFORCEMENT is J-34-measured) + L8 checker observe-only first; wire-dark-allowlist rows + promotion plans. | J-89:L31-L31 |
| J89-E014 | E/SURFACE | frontend_pair: spec-bundle/jury panel on planning surface + L8 plan-DAG report in existing plan view; W-6-justified where absent. | J-89:L32-L32, L55-L55 |
| J89-E015 | E/INT | L1 reachable at PLANNING->TESTING_FIRST on real elai plan/idea-intake; L8 at planning-workflow PRE_GATE / b-plan-runner AUTHORING; contracts never R7. | J-89:L45-L48 |
| J89-E016 | E/LIMIT | stub_ban: no jury deterministic verdict; no "correct objective" claim; no semantic plan completeness claim; no new symmetry/attestation engine; no ENFORCEMENT of jury escalation until J-34-measured. | J-89:L49-L54 |
| J89-E017 | E/OBS | Trigger: idea-intake with unstated domain assumption + plan-author with orphan-producer leaf. expect BundleSealed; JuryChallengeSurvived listing ambiguity; JuryEscalated on cross-family disagreement. | J-89:L56-L58 |
| J89-E018 | E/FAIL | L1 check_bundle_coverage FAILS unstated assumption at PLANNING before code; L8 FAILS orphan-producer at AUTHORING before code. NEGATIVE: empty competency-question set FAILS (never SKIP); unparseable plan-DAG FAILS; single-family jury REFUSED. | J-89:L59-L59 |
| J89-E019 | E/LIMIT | RESIDUE Rice/halting: no algorithm decides whether objective captures intent; TD-5 abstain-and-escalate-to-human is first-class EXIT. | J-89:L65-L65 |
| J89-E020 | E/CFG | LOCAL-LANE RESIDUE: ~1.8GB free VRAM cannot co-resident second family; LOCAL spec-jury DEGRADES to deterministic-checker-as-cross-family substitute (J-94 C5-SELFMOD THIN). | J-89:L73-L73 |
| J89-E021 | E/DEP | COMPOSES-WITH I-45, spec-grounder, D-13, G-8, wiring_contract_check/BabyStepSpec/C-29/Kahn, J-87 taint, wiring-enforcement PRE_GATE. | J-89:L68-L69 |
| J89-E022 | E/INT | L8 is FROZEN gate (R8, gate-manifest); NO new SSB for L8 (harness, JD-1 log evidence). | J-89:L30-L30 |
| J89-A001 | A/EVENT | SpecBundleEvent vs I-45 ObjectiveSealed reuse decide at edit. | J-89:L29-L29 |
| J89-A002 | A/CFG | Value of N challenges and exact escalate-on-disagreement threshold unspecified. | J-89:L10-L10 |
| J89-A003 | A/SURFACE | Planning panel chrome details decide at edit. | J-89:L32-L32 |

Category coverage: capabilities/gates ✓; planning surfaces ✓; CLI/workflow PRE_GATE ✓; events ✓; security/taint ✓; failures ✓; determinism ✓; tests/done ✓; flags ✓; deferred enforcement ✓. Formal privacy not owned; jury advisory only.

### J-92 — orchestrator FSM dispatch ordering

Source: `docs/thefinalglance/J-plan/J-92_orchestrator_fsm_dispatch_ordering.md` (104/104 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| J92-E001 | E/STATUS | J-92 BUILD FSM substates + capability tokens; effort M WAVE 2; depends I-1, J-66, J-68; lever GATE-THE-GUARD; closes logged P0 of main LLM rationalizing past dispatch order. | J-92:L1-L6 |
| J92-E002 | E/CAP | Promote orchestrator dispatch to first-class FSM SUBSTATES with EvidenceCheck; "edit before context-compiler returned" becomes STRUCTURALLY ILLEGAL transition. Substate sequence PARALLEL_BATCH -> EXECUTING -> AUDITING -> MEMORIZING is the FSM. | J-92:L9-L9 |
| J92-E003 | E/SEC | UNFORGEABLE per-phase CAPABILITY TOKENS: agent cannot invoke step without token minted only on PRIOR phase evidence. Edit on plan-scope requires token from context-compiler + integration-explorer evidence; test-run needs AUDITING entry token; memory-write needs MEMORIZING token. | J-92:L10-L10 |
| J92-E004 | E/DEP | I-1 directive-supervisor plane IS this architecture (generator Intent->Directive; process-separated supervisor executes + holds tokens). Move orchestrate-protocol H-1..H-10 from prose into gates. | J-92:L11-L11 |
| J92-E005 | E/DEP | J-68 ElaiFSM.tla + TLC proves ordering invariant; J-66 temporal monitor catches live violation per committed row. | J-92:L12-L12 |
| J92-E006 | E/LIMIT | EXCLUDED: LLM deciding transition legality; forgeable/advisory token; re-authoring FSM or I-1. | J-92:L13-L13 |
| J92-E007 | E/EVENT | +OrchestratorDispatchEvent Kind: TokenMinted / TokenConsumed / IllegalTransitionRefused; net 0..+1 prefer-reuse phase_transition Kind. | J-92:L25-L25 |
| J92-E008 | E/CFG | FEATURE FLAG ELAI_ORCHESTRATOR_FSM_TOKENS_ENABLE (kill-switch; OBSERVE-ONLY first per GD-4 — illegal transition FLAGGED before BLOCKS; USER_OVERRIDE path preserved) + wire-dark-allowlist row + promotion plan. | J-92:L27-L27 |
| J92-E009 | E/SURFACE | frontend_pair: dispatch-substate + token state on orchestrate-run / FSM surface; W-6-justified where absent. | J-92:L28-L28, L48-L48 |
| J92-E010 | E/INT | Production: orchestrate/auto-orchestrate attempting Edit on plan-scope before context-compiler evidence landed. | J-92:L39-L41 |
| J92-E011 | E/LIMIT | stub_ban: no LLM transition judge; no forgeable token; no prose-only enforcement left as gate; no BLOCK default before FP/USER_OVERRIDE friction bounded >=3 runs; no re-author FSM/I-1. | J-92:L42-L47 |
| J92-E012 | E/OBS | Legal path: TokenMinted at PARALLEL_BATCH completion then TokenConsumed at Edit; illegal path IllegalTransitionRefused naming missing evidence + attempted action. | J-92:L50-L51 |
| J92-E013 | E/FAIL | NEGATIVE: token claimed without backing evidence FAILS verification never mints; monitor on run with no dispatch rows FAILS self-test; regress bottoms at proven code (J-68) or human USER_OVERRIDE. | J-92:L52-L52 |
| J92-E014 | E/RECOVERY | Legitimate USER_OVERRIDE mints token via override path and is journaled as such (protocol E-1 legal skip); gate cannot brick deliberate human override. | J-92:L52-L52, L65-L65 |
| J92-E015 | E/FAIL | If kill-switch OFF, illegal transition observed and flagged (observe-only) rather than refused; SSB still records attempt so J-66 can fire. If EvidenceCheck bypassed, abort to L1-HardVeto. | J-92:L76-L76 |
| J92-E016 | E/DEP | FORK: if I-1 process-separated supervisor missing at ship, thin InProcess path ships alone (token mint/consume in-process); resolves when I-1 lands. | J-92:L78-L79 |
| J92-E017 | E/TEST | VERIFY: cargo test j92_ filter; production_caller_check on mint_on_evidence/consume_for_action/refuse_illegal_transition; falsify by deleting call sites. | J-92:L86-L89 |
| J92-E018 | E/TEST | REDTEAM: mint without evidence fails; Edit with kill-switch ON no token fails; forge TokenMinted bytes fails; rationalize past gate fails (EvidenceCheck ignores LLM). | J-92:L91-L95 |
| J92-E019 | E/LIMIT | Residual: J-66/J-68 stay compose-with, not this plan's ship gate for structural token code. | J-92:L101-L102 |
| J92-A001 | A/EVENT | Prefer-reuse phase_transition Kind vs new OrchestratorDispatchEvent decide at edit. | J-92:L25-L25 |
| J92-A002 | A/SURFACE | Orchestrate/FSM UI pair is W-6 / owner-ui-hold; smoke does not cover UI. | J-92:L99-L99 |

Category coverage: capabilities/FSM tokens ✓; surfaces (orchestrate/FSM) ✓; workflows/states ✓; security/unforgeable ✓; events ✓; flags ✓; failures/override ✓; tests/redteam ✓; dependencies/forks ✓. No privacy policy; no CLI verb beyond orchestrate.

### J-95 — context-activation wires

Source: `docs/thefinalglance/J-plan/J-95_context_activation_wires.md` (120/120 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| J95-E001 | E/STATUS | J-95 PLANNED WAVE 1 BUILD = activation not invention; ZERO new capability; net 0 new SSB; lever IMPORTANT-CONTEXT-NO-BLOAT; effort M (3 wires) + S (pointers). | J-95:L1-L6 |
| J95-E002 | E/STATUS | Deep-dive found design more wired than 06-23 map: W1 ContextBudget 10 slots WIRED; W2 H-50 regime WIRED (stale uncommitted note); W3 I-10 TTL WIRED; W4 dual tool serialization WIRED; W5 FSM phase tools WIRED; W6 BeliefState BUILT. Genuine remaining wires G2, G5, G4-verify. | J-95:L9-L17 |
| J95-E003 | E/CAP | LEG A IMP-2 G2: wire shipped adaptive_reduce relevance ladder into live conversation/turn-prep overflow BEFORE lossy conversation trim/compaction; keep high-overlap SSB + current-file+deps; drop irrelevant. | J-95:L21-L24 |
| J95-E004 | E/DET | TD-3: reduction order deterministic via file_overlap_score; no LLM picks what to drop. Reuse existing context_reduction SSB; net 0 action_type. | J-95:L23-L24 |
| J95-E005 | E/CAP | LEG B IMP-4 G5: at 3E slot-assembly (context_budget compute), route SsbEntries/CodeContext candidates through pristine_path_filter so failed-sibling traceback/wrong-path burst becomes contrastive CONSTRAINT not raw error text. | J-95:L26-L28 |
| J95-E006 | E/DEP | LEG B COMPOSE J-78 declared journaled per-dispatch access_spec so what enters each model call is auditable. | J-95:L28-L28 |
| J95-E007 | E/CFG | LEG B OBSERVE-ONLY first (GD-4): emit would-exclude set, measure byte/relevance delta, before defaulting exclusion ON. Reuse I-5 pristine-path SSB + J-78 event; net 0 here. | J-95:L28-L29 |
| J95-E008 | E/CAP | LEG C IMP-3 G4: VERIFY BeliefState condense on live I-7 EscalationCascade + cross-tier handoff; feature map cites escalation_waterfall.rs:239 Condensed; if path dark wire it; if fully wired degrade to contract test + honest note. | J-95:L31-L34 |
| J95-E009 | E/DATA | Condense {tried / failed-because / current-AST-state / constraints, DKG-keyed} into typed BeliefState instead of shipping >=20K raw transcript; faithfulness is K-12 Critical-Atom-Recall not re-authored. | J-95:L33-L33 |
| J95-E010 | E/DEP | SEQUENCING pointers (not re-authored): IMP-1=J-34 bench context-fill; IMP-5=J-16+J-38; IMP-6=J-54+H-7; IMP-7=I-1; IMP-8=J-19 vLLM prefix cache. | J-95:L36-L42 |
| J95-E011 | E/FLOW | Wave A: LEG A, LEG C, IMP-5a J-16. Wave B: IMP-1 J-34. Wave C: LEG B + J-78, J-38, J-54. Wave D: I-1, J-19. | J-95:L44-L48 |
| J95-E012 | E/INT | Production: A live elai run overflowing trim -> adaptive_reduce; B live dispatch with failed-path burst -> constraint not raw; C live local->cloud escalation -> typed BeliefState. | J-95:L64-L67 |
| J95-E013 | E/LIMIT | stub_ban: no LLM drop/exclude/condense; no cache-prefix rewrite (reductions AFTER last cache breakpoint, HD-2); no default-ON before would-reduce/would-exclude measured >=3 runs; no paper knee (RR-3/ID-5). | J-95:L69-L73 |
| J95-E014 | E/SURFACE | frontend_pair: ContextStatePanel reduction/exclusion deltas (decide at edit W-4) or W-6-justified. | J-95:L54-L54, L74-L74 |
| J95-E015 | E/OBS | expect_ssb: A context_reduction with FilteredByRelevance/file_overlap_score before conversation trim; B I-5 pristine-path + J-78 access-spec; C BeliefStateEvent(Condensed) ~2K belief. | J-95:L76-L77 |
| J95-E016 | E/FAIL | NEGATIVE: --no-adaptive-reduce drops high-overlap row; --no-pristine re-admits failed burst raw; LEG C if no dark path ships asserting contract never silent SKIP. | J-95:L78-L78 |
| J95-E017 | E/TEST | j95_context_activation_contract.rs (~120 LOC) arms a-d including HD-2 byte-stability across A/B. | J-95:L99-L100 |
| J95-E018 | E/TEST | DETERMINISTIC_CHECK: wiring_contract_check, production_caller_check adaptive_reduce+pristine, rg production call-sites, BeliefState condense in escalation_waterfall, J-95-decisions.md. | J-95:L80-L88 |
| J95-E019 | E/EVENT | net 0 new SSB action_types (prefer-reuse shipped rows; baseline re-read W-9). | J-95:L94-L94 |
| J95-E020 | E/LIMIT | Bound by CL-1..CL-4 context-regime doctrine (cited not re-authored) + HD-2 + RR-3/ID-5. | J-95:L6-L6, L96-L96 |
| J95-A001 | A/INT | LEG A exact overflow branch (conversation.rs trim vs 3E adaptive) grepped at edit — wire at branch that fires FIRST. | J-95:L114-L114 |
| J95-A002 | A/STATUS | LEG C may find condense fully wired -> contract-test-only; honest residual. | J-95:L113-L113 |
| J95-A003 | A/CFG | LEG B near-miss risk: defaulting exclusion ON may drop relevant failed-path; measure precision; constrained-not-deleted render keeps info recoverable. | J-95:L115-L115 |

Category coverage: capabilities/wires ✓; surfaces ContextStatePanel ✓; workflows overflow/escalation ✓; events reuse ✓; determinism/HD-2 ✓; failures negative arms ✓; tests ✓; sequencing deps ✓. No new CLI verbs; activation of existing paths only.

### J-108 — retrieval navigator depth operator

Source: `docs/thefinalglance/J-plan/J-108_retrieval_navigator_depth_operator.md` (125/125 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| J108-E001 | E/STATUS | J-108 PLANNED WAVE RLM-2 effort M; EXTENSION(J-42 + J-86 + J-38); DEPTH sibling to J-86 WIDTH ladder; local-lane default GATED on J-107; depends J-107 ships AFTER it. | J-108:L1-L3 |
| J108-E002 | E/CAP | RN-1 PRIMARY: route multi-hop through hippo_rag.rs::retrieve_notes (PPR one deterministic pass) + ATOMIC model extraction (read entity/regex number; optional 3-sample vote for atomic read only). Model NEVER navigates. | J-108:L16-L16, L38-L38 |
| J108-E003 | E/CAP | RN-2 FALLBACK: constrained RetrievalNavigator FIXED NavOp::{Grep(pat), Read(ids), Expand(id), SubQuery{ids,q} depth<=2, Answer(s)} over query_with_tier candidate set; fires ONLY when RN-1 has no clean entity linkage. | J-108:L17-L17, L39-L39 |
| J108-E004 | E/DET | Model PROPOSES one named op; deterministic executor RUNS it; every op is SSB row (RS-2). Caps ~25-37% on hard multi-hop — safety net never primary. | J-108:L17-L17 |
| J108-E005 | E/FAIL | RN-3 ABSTAIN-ON-BUDGET == SearchLadder BudgetCollapsed -> winner:None: on max_steps exhaustion return "no answer", NEVER a guess. | J-108:L18-L18, L40-L40 |
| J108-E006 | E/CONC | RN-4 SubQuery capped child call depth<=2 over shipped orchestrate/BSP sub-agent machinery, not arbitrary REPL; below J-55 code-mode collapse threshold. | J-108:L19-L19 |
| J108-E007 | E/CFG | RN-5 max_steps = J-107-MEASURED knee per-(model,lane) (ID-5); fixture default now, live knee when J-107 lands (W-10). NEVER paper constant. | J-108:L20-L20, L41-L41 |
| J108-E008 | E/LIMIT | Hard boundaries: NO free-Python RLM; NO model-CHOSEN orchestration (fixed op set); NO local general CodeAct (J-55); local default-ON contingent on J-107. | J-108:L5-L5, J-108:L59-L63 |
| J108-E009 | E/DEP | GD-1 consume not reimplement: hippo_rag PPR, query_with_tier, adaptive_tool ToolDispatcher, SearchLadder idiom, J-86, J-38, J-42, SessionDag/H-49. | J-108:L5-L5, L23-L28 |
| J108-E010 | E/CFG | Env gate default-OFF ELAI_RETRIEVAL_NAVIGATOR_ENABLE + wire-dark-allowlist.toml row (type=experiment, owner, expiry_date, promotion_plan_id=J-108, smoke_test) per R9. | J-108:L42-L42 |
| J108-E011 | E/INT | PRODUCTION_HOOK: navigate()/route MUST be reached from task_runner retrieval path SAME changeset (D-030); seam PLAN-asserted ~task_runner.rs:12869+. | J-108:L42-L42, L56-L57 |
| J108-E012 | E/CFG | Local-lane default-ON ONLY if J-107 arm cleared (lift + no J-55 collapse); else local stays single-shot/HippoRAG-only and navigator cloud/capable-lane only. | J-108:L43-L43 |
| J108-E013 | E/SURFACE | frontend_pair: existing search/ConvergencePanel surfaces hop chain + selected/abstained + op stream — tile vs reuse at edit (W-4); N/A if rides I-4/I-27. | J-108:L64-L65 |
| J108-E014 | E/EVENT | Reuse SearchNodeEvent (Promoted/BudgetCollapsed) + adaptive_tool op markers + query_with_tier rows; net 0..+1 Kind at edit. | J-108:L68-L68 |
| J108-E015 | E/OBS | Integration smoke 3 cases: entity-linked multi-hop -> HippoRAG; no linkage -> navigator; unreachable -> abstain; hard long-context input. | J-108:L66-L67 |
| J108-E016 | E/FAIL | NEGATIVE that must FAIL: model writing/executing code over context; fabricated answer on budget exhaustion; op not in fixed NavOp set. | J-108:L69-L69 |
| J108-E017 | E/TEST | j108_retrieval_navigator_contract.rs (~110 LOC) arms a-i including GD-2 replay, out-of-set reject, no code exec path, SubQuery depth<=2, max_steps from knee not literal, empty candidate abstains. | J-108:L98-L99 |
| J108-E018 | E/TEST | DETERMINISTIC_CHECK: wiring_contract_check; NavOp/abstain/BudgetCollapsed/hippo/max_steps present; exec/eval/python count MUST 0; model-authors-op count MUST 0; production_caller 0-DARK; allowlist row; J-108-decisions.md. | J-108:L71-L80 |
| J108-E019 | E/DATA | CREATE navigator.rs (~210 LOC); MODIFY task_runner.rs (~40); MODIFY wire-dark-allowlist; MODIFY ssb/.golden only if +Kind; CREATE contract; CREATE J-108-decisions.md. | J-108:L101-L107 |
| J108-E020 | E/EVIDENCE | Measured merge from rlm-recursive-agents-grc + RLM-MERGE implementation sketch + RLM-VS-ELAI results (B fabricated, C abstained, D multi-hop arm; ORACLE 8/8 vs model nav ~25%). | J-108:L109-L111 |
| J108-E021 | E/LIMIT | Atomic-read 3-sample vote is for FUZZY entity/number read ONLY — never multi-hop selector (IC-6; hop traversal is deterministic HippoRAG). | J-108:L120-L120 |
| J108-A001 | A/DET | "has clean entity linkage" routing must be deterministic (reuse J-84 TYPE axis / DKG edge presence); verify it forecloses model-chosen routing structurally. | J-108:L117-L117 |
| J108-A002 | A/INT | task_runner seam PLAN-asserted ~12869; wrong seam is material fork STOP+handoff; do not ship navigator.rs dark. | J-108:L118-L118 |
| J108-A003 | A/CFG | If J-107 slips, ship capable-lane only + local HippoRAG+single-shot and SAY so (GD-5). | J-108:L119-L119 |
| J108-A004 | A/SURFACE | ConvergencePanel tile vs reuse decide at edit. | J-108:L64-L65 |

Category coverage: capabilities/depth ops ✓; surfaces ConvergencePanel ✓; workflows multi-hop ✓; config flags ✓; events reuse ✓; failures/abstain ✓; determinism/fixed NavOp ✓; tests ✓; security (no free code) ✓; dependencies/J-107 gate ✓.

## Cross-source interactions and lifecycle

| Interaction ID | Class | Interaction, ordering, or shared invariant | Evidence atoms |
|---|---|---|---|
| IX-001 | E | J-7 resolves the skill-import scanner so later ports (including J-5) inherit skillspector --no-llm; pointer lines feed F-1/B-29/I-12/H-12 families without scope edits. | J7-E003–J7-E008, J7-E013, J7-E031 |
| IX-002 | E | J-30 extends B-37 factory/RuntimePanel without parallel registry; OAuth stays I-50; wire formats stay I-49; models/inventory suggestions come from I-51; custom vLLM endpoints are natural J-19 interop. | J30-E001, J30-E008, J30-E021–J30-E022, J30-E031 |
| IX-003 | E | J-30 turn lock binds within a sequence while H-7/E-31 role routing decides between turns; EscalateToCloud remains the sanctioned boundary. | J30-E019–J30-E020 |
| IX-004 | E | J-42 is the REASONING axis of the DD triad with J-39 decomposition and J-40 research; all three share bound-and-reset / distill discipline. | J42-E001–J42-E002, J42-E022 |
| IX-005 | E | J-42 extends GRC reset_topic and B-39 refine; consumes H-50 context fraction, H-42 budget, D-16 abstention, I-7 escalation; is within-STEP sibling of E-25/H-50 session compaction. | J42-E006–J42-E011, J42-E013, J42-E022 |
| IX-006 | E | J-48 outer serving policy sits above J-19 engine and J-9 fit catalog; consumes CloudModeRouter spend + I-7 spill; emits audit for J-29 UsageWindow; J-49 adds per-seat pre-filter in front of decide(). | J48-E002–J48-E007, J48-E013, J48-E016 |
| IX-007 | E | J-72 folds into J-18 embedder swap path; replaces ms-marco rerank with optional scoring head; gold set shared with J-75 ablation; G-2 RRA supplies agent-trace supervision; G-10 canary invalidates on corpus drift. | J72-E002–J72-E006, J72-E015, J72-E018 |
| IX-008 | E | J-78 formalizes I-5 pristine exclude and H-31 auditor zero-memory as declared access_spec cases; gates 3E slot content; pairs with J-95 LEG B compose point. | J78-E002, J78-E006–J78-E007; J95-E005–J95-E007 |
| IX-009 | E | J-89 L1 enriches I-45 sealed objective into typed bundle and freezes via G-8; L8 lifts wiring_contract_check to plan time; enforcement of jury escalation waits on J-34; J-87 taint-tracks the bundle. | J89-E002, J89-E006, J89-E009, J89-E013, J89-E021 |
| IX-010 | E | J-92 installs orchestrator ordering structure that J-66 monitors live and J-68 proves; tokens live on I-1 supervisor plane; lifts orchestrate-protocol H-1..H-10 prose into gates. | J92-E002–J92-E005, J92-E008 |
| IX-011 | E | J-95 activates shipped adaptive_reduce (H-6), pristine_path_filter (I-5), and BeliefState condense (I-7/I-11) without new SSB; sequences J-34/J-16/J-38/J-54/H-7/I-1/J-19. | J95-E003–J95-E011, J95-E019 |
| IX-012 | E | J-108 DEPTH composes HippoRAG + query_with_tier + adaptive_tool + SearchLadder idiom; orthogonal to J-86 WIDTH; extends J-42 within-step and J-38 mid-task loops; step budget and local default gated by J-107; classifier J-84 routes single-shot vs depth. | J108-E001–J108-E009, J108-E012 |
| IX-013 | E | Free-Python RLM and model-chosen orchestration are rejected across J-108; J-55 forbids local CodeAct default; abstain-on-budget mirrors SearchLadder BudgetCollapsed trust contract. | J108-E005, J108-E008, J108-E016 |
| IX-014 | E | Context visibility chain for orchestrated work: J-89 validates objective/plan before code; J-92 orders dispatch; J-78 declares what each worker may read; J-95 reduces/filters what actually fills slots. | J89-E015; J92-E002; J78-E002; J95-E003–J95-E007 |
| IX-015 | E | Provider/runtime freedom cluster: J-30 BYOK+lock; J-48 local tier admission/spill; both ride B-37/RuntimeOrchestrator seams without rebuilding engines. | J30-E026–J30-E027; J48-E009–J48-E013 |
| IX-016 | E | Multiple sources prefer-reuse SSB families rather than always +1: J-48 RouteDecisionEvent; J-72 RetrievalAugmentEvent; J-78 optional agent-dispatch field; J-89 I-45 arm; J-92 phase_transition; J-95 net 0; J-108 SearchNodeEvent. | J48-E014; J72-E010; J78-E008; J89-E012; J92-E007; J95-E019; J108-E014 |
| IX-017 | E | Observe-only / GD-4 gates recur: J-42 EXP-3; J-78 enforce after FP; J-89 mechanism before J-34 enforce; J-92 flag then block; J-95 would-reduce/would-exclude; J-108 default-OFF navigator. | J42-E005; J78-E007; J89-E013; J92-E008; J95-E007; J108-E010 |
| IX-018 | E | User-facing CLI verbs introduced or extended in this chunk: skill-import gate (J-7 harness); `elai providers configure` (J-30); `/reason` path (J-42); `elai retrieval gold-build` + `elai bench retrieval-adapt` (J-72); orchestrate/auto-orchestrate (J-78/J-92); idea-intake/plan PRE_GATE (J-89); elai run overflow/escalation (J-95); task_runner retrieval (J-108). | J7-E021; J30-E013; J42-E017; J72-E008; J78-E010; J89-E015; J95-E012; J108-E011 |
| IX-019 | E | Dashboard/panel surfaces: dashboard-smoker impeccable line (J-7); RuntimePanel inventory/configure (J-30); KnowledgeDrawer MRR/delta optional (J-72); agent-pool access-spec optional (J-78); planning jury+DAG report (J-89); orchestrate/FSM tokens (J-92); ContextStatePanel deltas (J-95); ConvergencePanel hop stream (J-108); J-48 explicitly defers gauge to J-29. | J7-E011; J30-E014–J30-E018; J72-E009; J78-E009; J89-E014; J92-E009; J95-E014; J108-E013; J48-E016 |
| IX-020 | I | J-30 + J-48 both edit RuntimeOrchestrator; turn lock (J-30) and serving pre-step (J-48) must compose without mutually refusing legitimate EscalateToCloud. | J30-E019–J30-E020; J48-E013 |
| IX-021 | E | Hashing/key policy: J-7 JC-2 scan_hash (blake2b now / blake3 later); J-30 bans key material in journals; J-72/J-95 use blake3 content paths already in cascade; J-48 requires blake3-only posture in hard rules. | J7-E005; J30-E006; J72-E006; J48-E003 |
| IX-022 | E | J-108 and J-42 both implement distill/reset-style bounded depth but at different grains: J-42 within one GRC reasoning step; J-108 multi-hop retrieval depth with fixed NavOps. | J42-E002, J42-E006; J108-E001–J108-E003 |

## Explicit ambiguity and conflict register

| Register ID | Kind | Unresolved issue or required correction | Atom references |
|---|---|---|---|
| AC-001 | ambiguity/status | J-7 pending-user-yes and Windows clone path resolution at edit. | J7-A001–J7-A002 |
| AC-002 | ambiguity/surface | J-30 configure-verb existence, factory dynamic-registration shape, stale threshold/ACL strength. | J30-A001–J30-A003 |
| AC-003 | ambiguity/config | J-42 numeric size/depth caps, cloud vs local reset split, heuristic uncertainty check vs EXP-3. | J42-A001–J42-A003, J42-C001 |
| AC-004 | ambiguity/event | J-48 RouteDecisionEvent payload fit vs new LocalServingDecisionEvent; multi-big-tier FIFO policy. | J48-A001–J48-A002 |
| AC-005 | ambiguity/config | J-72 first-shipping variant and gold-set quality thresholds; KnowledgeDrawer UI choice. | J72-A001–J72-A002 |
| AC-006 | ambiguity/event | J-78 new ContextAccessSpecEvent vs agent-dispatch field; UI chrome; role-derived granularity. | J78-A001–J78-A003 |
| AC-007 | ambiguity/event | J-89 SpecBundleEvent vs I-45 reuse; N-challenge threshold; planning panel chrome. | J89-A001–J89-A003 |
| AC-008 | ambiguity/event | J-92 phase_transition reuse vs new event; UI hold. | J92-A001–J92-A002 |
| AC-009 | ambiguity/int | J-95 exact overflow branch; LEG C may be test-only; pristine exclusion precision risk. | J95-A001–J95-A003 |
| AC-010 | ambiguity/det | J-108 entity-linkage routing determinism; task_runner seam line; J-107 slip fallback; panel tile. | J108-A001–J108-A004 |

## Summary additions and corrections

Orientation comparison target: `docs/thefinalglance/feature-discovery/10-J-SUMMARY_INDEX.discovery.md`. “ADD” means an explicit source behavior omitted or materially compressed by the summary; “CORRECT/QUALIFY/REMOVE” means the summary’s wording changes scope or certainty. These are relational records, not additional feature atoms.

### J-7 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-J7-001 | **ADD** exact TSV schema fields, JC-2 scan_hash policy (blake2b now/blake3 later), and score-saturation logging rules. | 10-J:L96-L104 | J7-E004–J7-E007 |
| SD-J7-002 | **ADD** mcp_rug_pull stub non-coverage and G-4 remaining sole rug-pull defense (named GD-5). | 10-J:L99-L99 | J7-E008 |
| SD-J7-003 | **ADD** deferred F-1 Tier-2 Rust pattern reimplementation and impeccable rule-engine native port. | compressed | J7-E019 |
| SD-J7-004 | **ADD** exact nine target plan files, STEAL METHOD W-3 ban on summary-only ports, and concurrent-edit re-grep discipline. | 10-J:L101-L101 | J7-E013–J7-E014, J7-E030 |
| SD-J7-005 | **ADD** platform env budget stop condition and production path = future skill-import gate (JD-1). | partial | J7-E021, J7-E029 |

### J-30 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-J30-001 | **ADD** full custom_providers.toml field schema, field-probe drop behavior, and TOML-not-YAML house style. | 10-J:L341-L342 | J30-E007–J30-E010 |
| SD-J30-002 | **ADD** OpenCode UI inspiration boundaries (layout/interaction only; no OAuth copy) and optional I-51 4xx model suggestions. | compressed | J30-E015–J30-E016 |
| SD-J30-003 | **ADD** ProviderByokEvent kinds, SSB baseline 234/543 sequencing after J-28, and Tier-1 local-sidecar-only live test vs Tier-2 cloud keys. | partial | J30-E023–J30-E025 |
| SD-J30-004 | **ADD** exact module LOC plan, riders S-25..S-28, and lock-vs-retry same-backend clarification. | compressed | J30-E021, J30-E026, J30-E032 |
| SD-J30-005 | **ADD** negative arms: key-byte journal grep, missing api_key_ref loud fail, mid-sequence swap log. | 10-J:L347-L347 | J30-E029 |

### J-42 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-J42-001 | **ADD** EXP-3 win criteria (accuracy + abstention quality), >=3-run gate before default-ON, and equal-budget methodology. | 10-J:L478-L478 | J42-E004–J42-E005 |
| SD-J42-002 | **ADD** HD-3 free-form distillate ban, re-distill-once-then-escalate admission failure path, and attractor-pin of original goal. | 10-J:L474-L476 | J42-E008–J42-E010 |
| SD-J42-003 | **ADD** ReasoningCheckpointEvent 3-arm payload fields and SCHEMA 268/577 baseline sequencing. | 10-J:L480-L480 | J42-E015–J42-E016 |
| SD-J42-004 | **ADD** honest residues (fit≠capability; 40% OOD self-distill risk; SFT deferred) and pass-count depth cap requirement. | partial | J42-E012, J42-E019–J42-E020, J42-E023 |
| SD-J42-005 | **FLAG** open cloud append vs local reset split (summary says cloud keeps append initially; source still "decide at edit"). | 10-J:L477-L477 | J42-E024, J42-C001 |

### J-48 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-J48-001 | **ADD** full LocalServingDecision enum including AdmitQueued class and Wait/safe-degrade on corrupt inflight. | 10-J:L532-L532 | J48-E008–J48-E009, J48-E019 |
| SD-J48-002 | **ADD** production pre-step at try_complete, D-030 caller requirement, and ships-ON no new enable flag. | partial | J48-E013, J48-E015 |
| SD-J48-003 | **ADD** EXCLUSIVE type-encoded invariant, NamedClass FIFO <=3, and air_gapped never-spill contract arm. | 10-J:L533-L536 | J48-E012, J48-E019 |
| SD-J48-004 | **ADD** frontend N/A justification to J-29, wargame moves, and directional HD-1 warm/cold residue. | compressed | J48-E016, J48-E022 |
| SD-J48-005 | **QUALIFY** summary "32B exclusive on 24GB": source uses topology-motivated EXCLUSIVE big-tier policy with directional numbers, not a fixed model id hardcode. | 10-J:L534-L534 | J48-I001, J48-E022 |

### J-72 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-J72-001 | **ADD** InfoNCE tau=0.1, EvoTrain-180K schema mapping, and fold-into-J-18 exact-dim migration discipline. | 10-J:L772-L772 | J72-E002 |
| SD-J72-002 | **ADD** teacher distill cost band (~$10-20/20K), ~2M 3-layer head, and explicit KaLM/EvoRAG exclusions R1/R2. | 10-J:L773-L773 | J72-E003, J72-E005 |
| SD-J72-003 | **ADD** CLI verbs gold-build/bench retrieval-adapt, W1 CORE-Bench unverified-number ban, and G-10 drift canary compose. | partial | J72-E008, J72-E012, J72-E014, J72-E018 |
| SD-J72-004 | **ADD** Tier-1 product vs Tier-2 GPU split and optional KnowledgeDrawer UI. | compressed | J72-E007, J72-E009 |

### J-78 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-J78-001 | **ADD** ContextAccessSpecEvent kinds SpecDeclared/OutOfSpecReadFlagged/SpecEnforced and prefer-reuse option. | 10-J:L832-L832 | J78-E008 |
| SD-J78-002 | **ADD** role+sub-phase pure-function computation, Conductor learn-vs-ELAI-declare contrast, and TRINITY/O5 rejection. | 10-J:L831-L835 | J78-E004–J78-E005 |
| SD-J78-003 | **ADD** refute of 5x slower/pricier premise and O2/O3/O4 recorded-not-built riders. | none | J78-E014–J78-E015 |
| SD-J78-004 | **ADD** OBSERVE-ONLY enforce after >=3 runs and missing/empty -> minimal safe context negative arm. | 10-J:L833-L834 | J78-E007, J78-E013 |

### J-89 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-J89-001 | **ADD** full SPEC BUNDLE field list and three L8 deterministic checks (goal coverage / artifact reachability / protocol realizability). | 10-J:L939-L944 | J89-E002, J89-E008 |
| SD-J89-002 | **ADD** jury as sole LLM entry (planning-only advisory), highest-risk-only autoformalization, and single-family jury REFUSED negative. | 10-J:L941-L941 | J89-E003–J89-E005, J89-E018 |
| SD-J89-003 | **ADD** ELAI_SPEC_BUNDLE_GATE_ENABLE, L8 frozen harness gate no new SSB, and local-lane jury degrades to checker substitute. | 10-J:L945-L946 | J89-E013, J89-E020, J89-E022 |
| SD-J89-004 | **ADD** exact production symbols and PLANNING->TESTING_FIRST / b-plan-runner AUTHORING reachability. | compressed | J89-E011, J89-E015 |

### J-92 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-J92-001 | **ADD** exact substate sequence PARALLEL_BATCH/EXECUTING/AUDITING/MEMORIZING and per-action token mint sites (edit/test/memory-write). | 10-J:L1014-L1016 | J92-E002–J92-E003 |
| SD-J92-002 | **ADD** ELAI_ORCHESTRATOR_FSM_TOKENS_ENABLE kill-switch, observe-then-block, USER_OVERRIDE mint path, and L1-HardVeto if EvidenceCheck bypassed. | 10-J:L1018-L1019 | J92-E008, J92-E014–J92-E015 |
| SD-J92-003 | **ADD** OrchestratorDispatchEvent kinds, I-1 InProcess fork, and four redteam closed-fail arms. | compressed | J92-E007, J92-E016, J92-E018 |
| SD-J92-004 | **ADD** residual that J-66/J-68 are compose-with not ship gate for token code. | partial | J92-E019 |

### J-95 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-J95-001 | **ADD** honest deep-dive W1-W6 already-wired inventory and 10-slot ContextBudget correction. | compressed | J95-E002 |
| SD-J95-002 | **ADD** exact LEG A/B/C production reachability, negative --no-adaptive-reduce/--no-pristine arms, and LEG C verify-or-contract-test degrade. | 10-J:L1045-L1048 | J95-E012, J95-E016, J95-E008 |
| SD-J95-003 | **ADD** Wave A-D sequencing with cheapest-first order and CL-1..CL-4 binding. | 10-J:L1049-L1051 | J95-E010–J95-E011, J95-E020 |
| SD-J95-004 | **ADD** HD-2 cache-prefix placement rule and ContextStatePanel surface option. | 10-J:L1051-L1051 | J95-E013–J95-E014 |

### J-108 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-J108-001 | **ADD** RN-1..RN-5 named operator stack, SubQuery depth<=2, and max_steps from J-107 knee not paper. | 10-J:L1142-L1149 | J108-E002–J108-E007 |
| SD-J108-002 | **ADD** default-OFF ELAI_RETRIEVAL_NAVIGATOR_ENABLE allowlist row, D-030 same-changeset task_runner wire, and local-lane gated by J-107. | 10-J:L1150-L1150 | J108-E010–J108-E012 |
| SD-J108-003 | **ADD** full contract arms a-i, exec/eval rg MUST 0, and atomic 3-sample vote scoped only to fuzzy extraction. | compressed | J108-E017–J108-E018, J108-E021 |
| SD-J108-004 | **ADD** measured RG-4/RG-5 grounding (free RLM 0%+hallucinate; model nav ~25%; oracle 8/8) justifying HippoRAG-primary. | partial | J108-E020 |
| SD-J108-005 | **FLAG** PLAN-asserted task_runner:~12869 seam and entity-linkage routing must be re-grepped/structurally deterministic. | none | J108-A001–J108-A002 |

## Exhaustiveness, zero-omission, and duplicate audit

- Source coverage: **10/10 files**, **1,225/1,225 logical lines**, **147,166/147,166 bytes** read and reconciled; source SHA mismatches **0**.
- Atomic source ledger: **257 unique atoms** = **227 explicit**, **002 inferred**, **027 ambiguous**, **001 conflicting/corrective**.
- Per-file atom counts: J-7 **033**; J-30 **036**; J-42 **028**; J-48 **027**; J-72 **020**; J-78 **019**; J-89 **025**; J-92 **021**; J-95 **023**; J-108 **025**. Sum **257**.
- Relational interaction records: **22** (`IX-001`–`IX-022`). Summary-delta records: **045** (`SD-*`). Ambiguity/conflict register: **10** (`AC-001`–`AC-010`). These records reference source atoms and are not double-counted as features.
- ID audit: every atomic table row has exactly one unique ID matching `J7/J30/J42/J48/J72/J78/J89/J92/J95/J108-[EIAC]nnn`. Duplicate atomic IDs **0**; missing/blank atomic IDs **0**; nonconforming atomic IDs **0**.
- Semantic duplicate audit: repeated premise/scope/wiring/grounding/acceptance statements were merged into one atom with multiple anchors. Interactions, conflict register, and summary deltas reference atoms instead of cloning them. Duplicate feature atoms found **0**.
- Omission audit: every source was checked against capabilities/actions/surfaces; workflows/states; schemas/data/lifecycle; flags/defaults/config; integrations/events; permissions/security/privacy/compliance; failures/fallback/retry/recovery; determinism/idempotency/concurrency; observability/evidence; tests/acceptance/done gates; constraints/non-goals; dependencies/status/deferred. Each per-file ledger explicitly names absent dimensions rather than inventing them. User-facing surfaces captured as own atoms: dashboard-smoker impeccable checklist line (J-7); `elai providers configure` + RuntimePanel form/test/status/inventory (J-30); `/reason` path (J-42); J-29 UsageWindow read-side for serving audit (J-48, N/A new UI); `elai retrieval gold-build` / `elai bench retrieval-adapt` + optional KnowledgeDrawer (J-72); agent-pool/orchestrate access-spec display (J-78); planning jury panel + plan-DAG report (J-89); orchestrate/FSM substate+token display (J-92); ContextStatePanel reduction/exclusion deltas (J-95); ConvergencePanel hop/op stream (J-108).
- Interpretation audit: inferred, ambiguous, and conflicting/corrective atoms are visibly separated by ID/class. No inference is promoted to a requirement. Orientation-only claims remain in delta records and are not source atoms.
- Manifest-boundary audit: the only original sources opened were the ten exact DEEP-J-011 manifest rows. The only orientation artifact consulted was `10-J-SUMMARY_INDEX.discovery.md`. Format reference consulted: `DEEP-I-004.discovery.md` for shape only.
- Output metrics after finalization: `output_lines=0532`; `output_bytes=080956`.
- Canonical normalized output SHA-256: 68ee9fd88684632401d9c2245782e1a809e6938107a65146c384be3d0cddebe4
- Hash rule: compute SHA-256 over the exact final UTF-8 bytes including the final newline after replacing only the 64 hexadecimal characters on the `Canonical normalized output SHA-256` line with 64 ASCII zeroes. Fixed-width output metrics and digest substitution avoid impossible raw self-reference.
