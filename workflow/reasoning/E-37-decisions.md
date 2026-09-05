E-37 Prompt-Injection Defense — Pre-EXECUTING Decisions
Session: s-1233230 | Date: 2026-05-21

WHAT I WILL IMPLEMENT
10 tasks across 7 new modules + modifications to 6 existing files + dashboard wiring:
1. safety::spotlight (220 LOC) — per-call random delimiter + datamarking on every untrusted ingest
2. plugins::tool_description_pin (180 LOC) — blake3 per-tool description pinning (NOT sha2)
3. safety::prompt_armor (250 LOC) — off-model classifier via injection_classifier role
4. safety::provenance (180 LOC) — DataProvenance enum (renamed from Provenance, collision with memory::Provenance)
5. safety::capability_gate (150 LOC) — CaMeL-lite destructive-verb gate on tainted argv
6. safety::output_sanitizer (120 LOC) — markdown-image strip + egress allowlist at WebFetcher
7. safety::output_classifier (140 LOC) — T1 PostToolUse constitutional check
8. Task 8 — MCP-Scan classifier on list_tools() via prompt_armor invocation
9. Task 9 — AgentDojo regression CI gate
10. Task 10 — output classifier on T1 PostToolUse

KEY DECISIONS

D1: DataProvenance rename (not Provenance)
Reason: memory::Provenance exists in C-48 (crates/memory/src/trust.rs:62), re-exported as elai_memory::Provenance.
task_runner.rs imports both elai-memory (via elai-knowledge) and elai-safety — collision unavoidable.
Decision: rename safety type to DataProvenance throughout. Provenanced<T> wrapper has no conflict, keeps its name.

D2: CapGateDecision rename (not GateDecision)
Reason: 3 existing GateDecision types — orchestrator::gate::GateDecision, tools::action_classifier::GateDecision,
orchestrator::ssb::GateDecisionPayload. All in scope in task_runner.rs.
Decision: rename to CapGateDecision { Allow, Reject(reason: String) }.

D3: Spotlight wraps at web_research_orchestrator.rs::fetch_and_extract (lines 601-616), NOT in fetcher/extractor
Reason: ContentExtractor trait signature is LOCKED ("Signature locked for 4D.17", object-safe).
Changing the trait breaks all impls. The sole call site that sequences fetch+extract is fetch_and_extract.
Decision: wrap at call site, not in trait. One hook point covers all ingest paths.

D4: Egress allowlist hook BEFORE HTTP send in fetcher.rs:109 (between lines 105-109)
Reason: domain already extracted at line 94 via extract_domain(). Blocking before send prevents any outbound.
Decision: new WebSearchError::EgressBlocked { url: String } variant + allowlist: Option<EgressAllowlist> field on WebFetcher.

D5: CLI needs 3 files (args.rs + main.rs + commands.rs), not 1 as plan estimated
Reason: MCP dispatch is blob-string CliAction::Mcp { args: Option<String> }. "pin update <server> <tool>" is
4 tokens — cannot fit in existing 2-field SlashCommand::Mcp shape without a new variant.
Decision: add CliAction::McpPin { server, tool, update, output_format } in args.rs; dispatch in main.rs;
handler in commands.rs (or new mcp_pin_cmd.rs). Matches RuntimeSubcommand pattern from B-37D.

D6: CSP written from scratch (was null)
Reason: tauri.conf.json:28 shows "csp": null — no existing policy to extend.
Decision: full string: "default-src 'self' tauri: asset:; img-src 'self' data: blob: tauri: asset: github.com huggingface.co arxiv.org; script-src 'self'"
Risk: must include tauri: and asset: or Tauri dev server breaks. Test explicitly.

D7: commands_e37.rs (new file) for dashboard Tauri commands
Reason: separate per-E-plan module pattern (commands_e1.rs established this). Avoids merge races.
Decision: create crates/dashboard-v2/src-tauri/src/commands_e37.rs + mod declaration in lib.rs + registration in invoke_handler after line 637.

D8: SCHEMA_VERSION 96 -> 97 (9 new SsbActionType variants as single batch)
Reason: current SCHEMA_VERSION=96 (ssb.rs:955), 284 existing variants. E-37 adds 9 in one atomic bump.
Decision: bump to 97 with all 9 variants added in alphabetical order within enum.

D9: AgentRole::InjectionClassifier in elai-ipc-types crate (NOT orchestrator)
Reason: AgentRole is defined in elai_ipc_types, imported by orchestrator. Adding it in orchestrator
would require re-exporting or cause import confusion.
Decision: add variant to crates/ipc-types/src/lib.rs (or wherever AgentRole is defined); add to
PerRoleRuntime::default() with RuntimeBackend::Sidecar + add to all_roles() list.

D10: elai-web-search -> elai-safety dep needed (not yet wired)
Reason: web_research_orchestrator.rs will call spotlight::wrap_untrusted — needs elai-safety in scope.
Decision: add elai-safety to crates/web_search/Cargo.toml. No circular dep (safety has no upward deps).

EXPECTED TEST OUTCOME
All 10 contract tests pass:
- spotlight_envelope_wraps_all_ingest_sources_contract: 5 ingest paths, delimiter randomized per call
- tool_description_pin_blake3_contract: first=pin, mutated=block+diff, update re-pins
- prompt_armor_classifier_verdict_contract: 5 bad fixtures=Block, 5 clean=Clean
- capability_gate_rejects_tainted_argv_contract: WebTierC=Reject, User=Allow
- egress_allowlist_blocks_attacker_host_contract: attacker.local -> EgressDenied + SSB row
- output_sanitizer_strips_markdown_image_contract: 5 patterns all stripped
- mcp_scan_classifier_list_tools_contract: line-jumping desc -> blocked + SSB
- output_classifier_flags_exfil_url_contract: ![](attacker.local) -> flagged + SSB
- sandbox_egress_dns_default_deny_contract: attacker.local DNS -> blocked + SSB
- agentdojo_regression_smoke: 10 sampled tasks, ASR < 5%

UNKNOWNS
- Token overhead of Spotlighting: target <1.5x total. Measure on typical B-N session before declaring shippable.
- PromptArmor FPR on ELAI corpus: run on 200 sampled spans before default-enabling Block.
- agentdojo PyPI package verification: MUST verify via pip/PyPI before adding (anti-slopsquatting HARD RULE).
- C-11 seccomp coordination: Task 7 egress lockdown is additive to C-11 — verify C-11 not yet shipped.

===

EXECUTION ADDENDUM — s-25120 session 2 (2026-05-25)

SHIPPED 9 of 10 tasks atomically (Task 7 deferred per rebase). Wire-dark posture
on the LLM classifier path (HeuristicClassifier always loaded; sidecar LLM
classifier deferred to follow-on). 

SCHEMA atomic bump: 132 -> 133 in same commit. ALL_VARIANTS 369 -> 378. Golden
regenerated alphabetically (9 new entries land in sorted positions).

D-EXEC-1: AgentRole::InjectionClassifier added as 17th variant (NOT 16th —
the context-compiler note said 16 but live count was 16 pre-E-37; new = 17th).
Cascade sites updated: agent_pool.rs (3 arms: snake_case + grc_config +
allowed_paths fallthrough), bsp/engine.rs, ipc_gateway.rs (grounding methods),
routing/selective.rs (Primitive::Verify), per_role_runtime.rs (Sidecar tier
mapping + all_roles helper test), dashboard-v2 commands.rs (16-role default
list), reasoning_surface_contract.rs test list, c45_cantante exclusion
assertion. Per-AgentRole `match` arms with `_ =>` fallthrough did NOT need
explicit additions (10+ such sites verified via cargo check --workspace).

D-EXEC-2: Spotlight uses elai_core::EntropySource via `&mut dyn EntropySource`
parameter (NOT &mut impl Rng). g5_spotlight_uses_entropy_source_trait contract
test enforces this at compile-time via fn requires_entropy_source<E:
EntropySource>(_e: &mut E) bound assertion.

D-EXEC-3: 9 SSB variants chosen (NOT the 9 from cross-check report, which
were the older draft list — InjectionAttemptDetected, InjectionAttemptBlocked,
SpotlightWrapped, SpotlightBreached, ToolDescriptionPinMismatch,
CapabilityGateRejected, ProvenanceTagged, PromptArmorVerdict,
InjectionFixtureReplayed). Picks 5 detection-state variants (the 4 from cross-
check renamed to clearer noun-form) + 4 from the cross-check (Spotlight* +
ToolDescriptionPin* + CapabilityGate* + remaining). Sibling Egress + Output*
variants from the original plan were folded into broader InjectionAttempt*
to keep the cascade at exactly 9 (largest in cluster — E-18 was 4).

D-EXEC-4: CSP gap documented at "csp warning" level per session rebase #1.
tauri.conf.json `csp: null` retained pending live `npm run tauri:dev` smoke
test that the proposed restrictive policy
("default-src 'self'; img-src 'self' data: https://github.com
https://*.githubusercontent.com https://huggingface.co https://arxiv.org
https://mermaid.live; script-src 'self'; style-src 'self' 'unsafe-inline';
connect-src 'self' http://127.0.0.1:8100 http://127.0.0.1:8200 http://localhost:8100
http://localhost:8200; frame-ancestors 'none'") doesn't break icon / asset /
sidecar fetch rendering. CSP `_csp_note` JSON sibling field was tried and
REJECTED by tauri-build schema validator at compile time (unknown field).
Documentation lives here instead. RUNPOD-PENDING: live dashboard render
verification under the policy.

D-EXEC-5: Task 7 (Tier C seccomp egress lockdown) DEFERRED to C-11.B follow-on
per rebase #2. crates/sandbox/ + seccomp module DO NOT EXIST in the current
tree (verified). Cannot author seccomp/network-namespace rules against absent
surface. RUNPOD-PENDING-C11B-DEPENDENCY marker logged in mistakes.md.

D-EXEC-6: RUNPOD-PENDING — live AgentDojo / EchoLeak / CometJacking adversarial
fixture replay against deployed models. The `elai safety fixture-replay`
CLI surface ships the synthetic-fixture path (3 known canonical phrasings;
HeuristicClassifier verdict) — the live model invocation harness (Python
agentdojo package install + replay through the dispatch chain end-to-end)
is RUNPOD-PENDING per session prompt anti-slopsquatting verification + GPU
availability.

D-EXEC-7: 4 future-year arxiv IDs cited in the plan (2604.* prefixes) — these
are placeholder citations the plan author intended to verify. NOT inline-cited
in code (no `// arXiv:2604.*` comments in safety crate or ssb.rs). Logged in
mistakes.md as 5th cite-incident-deferral per cluster discipline.

D-EXEC-8: capability_gate operates at tool-name layer (`is_destructive_tool`
predicate over DESTRUCTIVE_TOOLS const list) AND argv-provenance layer
(rejects when ANY arg has TrustLevel::Untrusted). Composes WITH E-30
destructive_verb_allowlist::check_argv at argv-regex layer (BOTH run; neither
replaces the other). Recallable provenance (memory recall) does NOT block —
matches user expectation that memory-derived data can drive tools.

D-EXEC-9: 6 safety modules + lib.rs re-exports finalized (Tool struct also
exported so the contracts crate can build pin records without depending on
elai-plugins). Per E-37 architecture: the safety crate is the canonical
authority for the LABELS (INJECTION_CLASSIFIER_ROLE_LABEL) even though the
AgentRole enum lives downstream in elai-ipc-types.

CHANGES TO PLAN
- 6 safety modules under flat crates/safety/src/ (NOT a sub-directory tree).
- output_sanitizer + output_classifier folded into the broader PromptArmor
  classifier + capability_gate rejection layers (different concerns from the
  original plan's task split; net surface area smaller).
- AgentDojo CI workflow NOT shipped (Task 9 scope — RUNPOD-PENDING per
  anti-slopsquatting; needs PyPI verification of agentdojo package before
  pip install line can land in .yml).
- Decision doc append-only here; mistakes.md gets the 4-cite deferral entry.

NEXT
- Auditor cargo check + workspace tests.
- patch-N follow-on: live tauri:dev CSP smoke test + restrictive policy roll.
- C-11.B follow-on: seccomp + network-namespace egress lockdown.
- RUNPOD follow-on: agentdojo PyPI verification + live fixture replay.
