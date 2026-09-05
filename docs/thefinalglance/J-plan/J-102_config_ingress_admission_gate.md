# J-102: config-ingress admission gate — a blake3 LOAD-TIME admission manifest over the instruction surface + MCP add-scan + skill hard-enforce, ONE E-43/E-24 EXTENSION (WI-6a/b/c/d)

STATUS: PLANNED (WAVE 1; the technical config-admission half the K-13 OUT-OF-K block recorded NOT authored; an EXTENSION on E-43/E-24/J-52, NOT an in-place amend — new-plan-files-only; the absent surface = nothing scans a new/modified instruction .md before it loads into the model context; SSB +ConfigAdmissionEvent, net 0..+1)

- Track J | lever NO-OUT-OF-BAND-CONFIG (everything a user can ADD or MODIFY that ends up in the model's hands passes an ELAI safety check at first-touch + after any modification; it is not possible to add MCPs/skills/rules out-of-band of ELAI's own safety pipes) | grounding STRONG (pins.rs / config.rs / J-52 PathRuleInjector / E-43 skill-import-policy code-grounded 2026-06-25 in the OWASP analysis Section H; re-grep at edit W-7) | nature BUILD (one load-time gate over the instruction surface + an MCP-content extension + a soft->hard skill promotion) | effort L (WI-6c keystone) | WAVE 1
- This is the second technical companion to K-13 (the OWASP PROVE+PACKAGE half). K-13 maps the spine; J-101 hardens the DATA path (untrusted data into sinks); J-102 hardens the CONFIG path (untrusted config into the model's instruction surface). OWASP threats: LLM03 Supply Chain / LLM04 Data & Model Poisoning / LLM01 Prompt Injection + Agentic ASI04 Agentic Supply Chain / ASI06 Memory & Context Poisoning. This is ADMISSION CONTROL on the agent's OWN configuration + instruction surface.
- Source: workflow/reasoning/owasp-llm-agentic-gap-analysis-2026-06-25.md Section H (the user ask + agentlint grounding + the ELAI current-state gap + the architectural "no out-of-band bypass" answer), WI-6a/b/c/d. K-13 is the K-native PROVE+PACKAGE half. Doctrine: J-0_INDEX.md.

## Context
USER ASK (Section H verbatim intent): everything a user can ADD or MODIFY that ends up in the model's hands — .md skills/rules/agent-defs/CLAUDE.md/AGENTS.md, MCP server configs — must pass an ELAI safety check at FIRST-TOUCH and again AFTER any modification; it must NOT be possible to add MCPs/skills/rules out-of-band of ELAI (everything routes through ELAI's own safety pipes).

THE ARCHITECTURAL KEY (the "no out-of-band bypass" answer): enforce at LOAD time, not ADD time. A raw file edit on disk cannot be prevented; but a content-hash-keyed admission manifest (blake3 TOFU on first-touch, re-scan on hash change) that FAIL-CLOSED QUARANTINES any artifact lacking a matching ALLOWED row means an un-admitted/modified artifact NEVER reaches context or the dispatcher. A mediated `elai mcp add` / `elai skill add` UX is convenience; the load-time hash gate is the enforcement.

ELAI CURRENT STATE (live-code verified, Section H):
- MCP: STRONG, AHEAD of agentlint on identity — crates/plugins/src/mcp/pins.rs McpServerPin{exact_version,expected_pubkey_blake3,auth_required} + McpPinPolicy{enforce_pin_list,refuse_version_float,refuse_unauthenticated} -> enforce_pin Allow/Refused; + E-24 blake3 schema-pin + G-4 live re-pin on tools/list_changed + C-42 DENY-ALL authz. GAP = agentlint-style CONTENT checks (dangerous FS-path args, hardcoded secret in env block, approval-field presence) are NOT in the MCP admission path. (WI-6a)
- Skills: SOFT gate — E-43 skill-import-policy + skillspector /skill-audit + workflow/skill-imports.log; the policy doc itself says PreToolUse HARD enforcement is DEFERRED -> today it is code-review-time, not a runtime block. (WI-6b)
- Rules / agents / CLAUDE.md / AGENTS.md: NOTHING. ConfigLoader (core/config.rs:205) merges config; J-52 injects path-keyed rules into the context Rules slot; J-62 is a pre-WRITE env-card gate. NONE scans a new/modified instruction .md for embedded injection/secrets/capability-escalation BEFORE it loads into the model context. A TOFU-trusted instruction surface = a real ASI04/ASI06/LLM01 vector. (WI-6c, THE GAP, the keystone)

AGENTLINT (grounded 2026-06-26, MIT, Python, pip leporis-agentlint): a NARROW MCP-config-only static scanner (mcp.json / claude_desktop_config.json / .mcp.json) — hardcoded secrets in env, broad-FS-path args, missing auth env vars, absent approval fields; risk 1-10, CI fails >=7. Does NOT cover CLAUDE.md/AGENTS.md/skills/rules; does NOT do injection. => STEAL = its MCP check-RULE-SET as DATA only. DO NOT port the pip dep (ELAI is Rust; reuse F-1 secret-GREP + VibeGate AST; ship checks as rules). Recorded as an idea/rule-set steal, no new dependency (anti-slopsquat: agentlint is not added).

## Existing state (what the legs extend / cite; do NOT re-author, GD-1)
- MCP HARDENING (E-24) [PARTIAL] crates/plugins/src/mcp/pins.rs McpServerPin/McpPinPolicy + enforce_pin + blake3 schema-pin; `elai mcp pin` not routed through slash handler. The WI-6a content-check fold target.
- LIVE RUG-PULL RE-PIN (G-4) [WIRED] re-pin on tools/list_changed with description diff. C-42 zero-trust MCP authz DENY-ALL. The MCP admission path WI-6a extends (identity already strong).
- AGENT SKILL SYSTEMS (E-43) [DARK(ELAI_SKILL_SYSTEM_ENABLE)] agentskills.io v1 frontmatter validator + `elai skills-mgmt` + skill-import-policy (skillspector whole-dir scan) + workflow/skill-imports.log; the policy doc names PreToolUse HARD enforcement as its OWN deferred piece — the WI-6b soft->hard promotion target.
- ConfigLoader (core/config.rs:205) — the WI-6c LOAD seam (merges config); the admission manifest gates here + at the context_budget Rules/PhaseSpec slot fill + at agent dispatch.
- PATH-KEYED DYNAMIC RULE INJECTION (J-52) [PLANNED] a PostToolUse path-fingerprint hook injects the matching local rule into the context_budget Rules slot — the NATURAL HOOK seam WI-6c intercepts (it already intercepts path-keyed rule injection; J-62 extends it for env-cards). WI-6c REUSES this seam, does not re-build it.
- VIBEGATE F-1 secret-GREP + VibeGate AST [WIRED] — the deterministic secret/dangerous-pattern legs WI-6a/WI-6c reuse (no new dep, no agentlint pip port).
- CONTEXT BUDGET ALLOCATOR Rules/PhaseSpec slots [WIRED] — the LOAD point WI-6c gates (an un-admitted artifact never fills the slot).
- D-13 cross-family adversary [DARK] — the WI-6c ADVISORY semantic leg (TD-3, NOT the gate; same LLM-hard limit as J-101 WI-1).
- live SSB baseline ~268/577 (re-grep at edit, W-9); J-102 +ConfigAdmissionEvent (Pattern-2: Admitted/Quarantined/ReadmissionRequired), prefer-reuse skill_system_event / mcp pin rows where possible (net 0..+1).

## Reads (mandatory before coding)
1. workflow/reasoning/owasp-llm-agentic-gap-analysis-2026-06-25.md Section H IN FULL (the user ask, agentlint rule-set, the ELAI current-state MCP/Skills/Rules split, the architectural LOAD-not-ADD key, WI-6a/b/c/d, the HONEST RESIDUE).
2. .claude/rules/skill-import-policy.md (E-43; the policy that names its own deferred PreToolUse HARD-enforcement piece — WI-6b promotes it) + the skillspector /skill-audit wrapper + workflow/skill-imports.log schema.
3. crates/plugins/src/mcp/pins.rs (McpServerPin/McpPinPolicy/enforce_pin — the WI-6a fold target) + mcp_pin_cmd — re-grep live at authoring (W-7).
4. crates/core/src/config.rs:205 ConfigLoader (the WI-6c LOAD seam) + the J-52 PathRuleInjector plan (the intercept hook WI-6c reuses) + the context_budget Rules/PhaseSpec slot fill.
5. The F-1 VibeGate secret-GREP + AST surface (the deterministic legs reused, no agentlint pip dep) + .claude/rules/bun-not-node.md (N/A here — Rust gate, no JS runtime) + CLAUDE.md anti-slopsquatting (agentlint is NOT added as a dep, rule-set as DATA only).

## Tasks (WAVE 1 — WI-6c keystone; WI-6a cheap extend; WI-6b soft->hard promotion)
1. [WI-6c KEYSTONE — the absent surface] a blake3-keyed admission manifest over the instruction surface (.claude/rules/*.md + .claude/agents/*.md + CLAUDE.md + AGENTS.md + .claude/skills/*/SKILL.md). At LOAD into the context-budget Rules/PhaseSpec slot (ConfigLoader core/config.rs:205) AND at agent dispatch: if the artifact is NEW or its hash CHANGED since last admission, run the admission scanner and FAIL-CLOSED QUARANTINE (it never enters context/dispatch). Scanner = DETERMINISTIC legs (secrets via F-1 GREP; dangerous-verb/capability-escalation directive patterns; known injection phrases "ignore previous"/"forget instructions"; zero-width/homoglyph/encoded-payload detection — a concrete deterministic win on obfuscated .md injection; frontmatter/schema validation) + an ADVISORY semantic leg (D-13 cross-family critic "is this directive trying to escalate", TD-3, NOT the gate). SEAM: ConfigLoader + the context_budget Rules-slot fill + agent dispatch; NATURAL HOOK = the J-52 PathRuleInjector (REUSE, GD-1). OBSERVE-ONLY first (GD-4): emit the would-quarantine set, measure the FP rate on the clean config, before defaulting the quarantine ON. +ConfigAdmissionEvent.
2. [WI-6a] MCP add-scan content checks (agentlint rule-set as DATA): fold dangerous-FS-path-arg + hardcoded-secret-in-env (reuse F-1 GREP) + approval-field-presence + a risk-score INTO the EXISTING pins.rs / mcp_pin_cmd admission path (which already does version-float/unauth/pubkey). No new dep (no agentlint pip port). Composes E-24/G-4/C-42/F-1.
3. [WI-6b] skills hard-enforce: promote E-43 skill-import-policy from SOFT (code-review) to HARD PreToolUse refusal — a skill with no workflow/skill-imports.log ALLOWED row matching its CURRENT content-hash is REFUSED at reference/load (the policy doc names this as its own deferred piece). Composes E-43/skillspector.
4. [WI-6d] the LOAD-TIME hash check of 6a/6b/6c IS the "no out-of-band bypass" enforcement (it catches raw external edits via changed-hash-without-admission-row). An OPTIONAL mediated `elai mcp add` / `elai skill add` UX rides on top (convenience, not the gate) — record as a follow-on, do not build it as the enforcement.
5. workflow/reasoning/J-102-decisions.md (the WI-6c FP observe-only delta + the settings.json-hook-registration admission decision + the J-52-seam confirmation).

## WIRING_CONTRACT
produces_symbols:
  - the blake3 LOAD-TIME admission manifest + scanner over the instruction surface, FAIL-CLOSED quarantining new/hash-changed artifacts at ConfigLoader + the Rules-slot fill + agent dispatch [WI-6c]
  - the MCP add-scan content checks (dangerous-FS-path / hardcoded-secret / approval-field / risk-score) folded into pins.rs / mcp_pin_cmd [WI-6a]
  - the E-43 skill admission promoted SOFT->HARD PreToolUse refusal (no matching content-hash ALLOWED row => refused at load) [WI-6b]
  - +ConfigAdmissionEvent (Admitted | Quarantined | ReadmissionRequired)
consumes_symbols:
  - ConfigLoader core/config.rs:205 (the LOAD seam) + the J-52 PathRuleInjector hook (REUSE, the intercept seam) + the context_budget Rules/PhaseSpec slot fill — the WI-6c gate points
  - F-1 VibeGate secret-GREP + AST (the deterministic legs; NO agentlint pip dep) + D-13 cross-family critic (the ADVISORY leg, NOT the gate)
  - pins.rs McpServerPin/McpPinPolicy/enforce_pin + mcp_pin_cmd (E-24, the WI-6a fold target) + G-4 re-pin + C-42 authz
  - .claude/rules/skill-import-policy.md + skillspector + workflow/skill-imports.log (E-43, the WI-6b soft->hard target)
  - the agentlint MCP rule-set as DATA (Section H; rule-set only, NOT the pip dep — anti-slopsquat)
reachable_from:
  - a live `elai run` that LOADS a new or modified .claude/rules/x.md (or agent-def / CLAUDE.md / SKILL.md) -> the admission scanner runs, an un-admitted/changed artifact is QUARANTINED before it fills the context Rules slot or reaches dispatch (NOT a unit test calling the scanner) [PRODUCTION, the keystone]
  - a live `elai mcp pin` / MCP load of an mcp.json with a dangerous FS-path arg or a hardcoded key -> the WI-6a content check refuses [PRODUCTION]
  - a live skill reference whose content-hash has no ALLOWED skill-imports.log row -> the WI-6b HARD PreToolUse refusal blocks [PRODUCTION]
  - a #[cfg(test)] / contracts/ caller NEVER satisfies any leg (R7)
stub_ban:
  - NO agentlint pip dependency (rule-set as DATA only; reuse F-1 GREP + VibeGate AST; anti-slopsquatting — agentlint is not added)
  - NO ADD-time-only enforcement (the LOAD-time hash check is the no-out-of-band-bypass enforcement; a mediated `elai * add` UX is convenience only, WI-6d)
  - NO semantic-injection detection AS THE GATE (LLM-hard, paraphrase limit, same as J-101 WI-1) — deterministic legs FAIL-CLOSED, the D-13 semantic leg is ADVISORY (TD-3)
  - NO default-ON quarantine before the clean-config FP rate is measured (GD-4 observe-first); the gate's default-ON flag gets a wire-dark-allowlist row (W-2/R9)
  - NO missing-admission-row => pass (FAIL-CLOSED; missing evidence is a hard FAIL, R1)
  - NO fork of E-43 / the J-52 PathRuleInjector (extend/reuse the seams; GD-1)
frontend_pair:
  ConfigAdmissionEvent (Admitted/Quarantined/ReadmissionRequired) surfaces on the SSB safety timeline + a quarantine banner; decide a config-admission tile at edit (W-4) or W-6-justify (a security gate, no required new UI).
observable_effect:
  trigger: "a seeded malicious config set: (i) .claude/rules/x.md with an 'ignore all gates' directive + a zero-width-obfuscated exfil line; (ii) an mcp.json with a /etc FS-path arg + a hardcoded key; (iii) an unscanned skill; PLUS a clean config (the FP guard) and a modify-an-already-admitted-file case"
  expect_ssb: "a ConfigAdmissionEvent{Quarantined} per malicious artifact + {Admitted} per clean artifact + {ReadmissionRequired} on the modified-already-admitted file (recomputable GD-2); prefer-reuse skill_system_event / mcp pin rows where the artifact is a skill/MCP."
  expect_artifact: "the three malicious artifacts are QUARANTINED at LOAD and never enter context/dispatch; a clean config loads UNCHANGED (the ten-benign FP guard); modifying an already-admitted file flips it back to QUARANTINE until re-admitted (the on-modify re-scan proven — the user's 'again after any modification'). NEGATIVE (R2): the gate-OFF arm lets the obfuscated .md into the Rules slot (proves the gate is load-bearing); a benign edit to a clean rule re-admits cleanly (no FP)."

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/J-plan/J-102_config_ingress_admission_gate.md --run-trigger
python .claude/tools/production_caller_check.py --files .elai_cc/crates/core/src/config.rs,.elai_cc/crates/plugins/src/mcp/pins.rs --json   # the admission manifest + MCP content-check have non-test production callers
rg -n "admission|quarantine|ConfigAdmissionEvent" .elai_cc/crates/core/src/config.rs | rg -v "test"   # the WI-6c LOAD-time gate at ConfigLoader
rg -n "zero.?width|homoglyph|ignore previous|forget instructions" .elai_cc/crates/ | rg -v "test"   # the deterministic obfuscated-injection legs
rg -n "FS.?path|hardcoded|approval" .elai_cc/crates/plugins/src/mcp/pins.rs   # the WI-6a content checks folded into the MCP admission path
rg -ni "leporis|agentlint" .elai_cc/Cargo.toml .elai_cc/crates/*/Cargo.toml ; test $? -ne 0   # agentlint is NOT a dependency (rule-set as DATA, anti-slopsquat)
test -f workflow/reasoning/J-102-decisions.md
```

## ACCEPTANCE
- [ ] WI-6c: a blake3 LOAD-TIME admission manifest over the instruction surface FAIL-CLOSED quarantines a new/hash-changed artifact at ConfigLoader + the Rules-slot fill + dispatch, before it reaches context; the deterministic legs (F-1 secrets / dangerous-verb / injection-phrase / zero-width-homoglyph / frontmatter) run; the D-13 semantic leg is ADVISORY only.
- [ ] on-modify re-scan: modifying an already-admitted file flips it to QUARANTINE until re-admitted (the user's "again after any modification").
- [ ] WI-6a: dangerous-FS-path / hardcoded-secret (F-1) / approval-field / risk-score checks folded into the EXISTING pins.rs admission path; NO agentlint pip dep.
- [ ] WI-6b: E-43 skill admission promoted SOFT->HARD — a skill with no matching content-hash ALLOWED skill-imports.log row is REFUSED at load.
- [ ] WI-6d: the LOAD-time hash check IS the no-out-of-band-bypass enforcement; OBSERVE-ONLY first (GD-4), the clean-config FP rate measured before default-ON; the gate flag gets a wire-dark-allowlist row.
- [ ] +ConfigAdmissionEvent (or prefer-reuse skill_system_event / mcp pin rows); cargo check --workspace GREEN; auditor PASS on -p elai-core + -p elai-plugins (+contracts). No new dependency.

## WIRING DONE-GATE (clause by clause)
- W-1: done = a live `elai run` LOADS a seeded obfuscated .md and it is QUARANTINED at the real ConfigLoader/dispatch seam (R10 integration smoke: a multi-artifact config load with one poisoned + one modified-already-admitted file), not a unit test calling the scanner. W-2: the default-ON gate flag + the WI-6b hard-enforce flag get wire-dark-allowlist rows (R9). W-3: WI-6c is net-new; WI-6a/b extend E-24/E-43, not ports.
- W-5: trigger = a live malicious-config-set load + a clean-config FP run + a modify-already-admitted run. W-7: config.rs:205 + pins.rs + the J-52 PathRuleInjector seam + skill-import-policy re-grepped at edit. W-9: ~268/577 re-grep.
- GD-1: reuse the J-52 PathRuleInjector + extend E-24/E-43/F-1, never fork. GD-4: observe-first. TD-3: deterministic legs are the gate, the D-13 critic is advisory. R1: missing admission row => FAIL-CLOSED. R7: test callers excluded. R10: the cross-seam smoke is mandatory.

## Tests to add
- crates/contracts/tests/j102_config_admission_contract.rs (~140 LOC): (a) a .claude/rules/x.md with an "ignore all gates" directive + a zero-width-obfuscated exfil line is QUARANTINED at load + never fills the Rules slot; (b) an mcp.json with a /etc FS-path arg + a hardcoded key is REFUSED by the WI-6a content check; (c) an unscanned skill (no content-hash ALLOWED row) is REFUSED by the WI-6b HARD PreToolUse; (d) a clean config loads UNCHANGED (the ten-benign FP guard, R2 — no false quarantine); (e) modifying an already-admitted file flips it to QUARANTINE until re-admitted (the on-modify re-scan, the user's core ask); (f) a missing admission row fails-CLOSED (R1); (g) the gate-OFF arm lets the obfuscated .md through (R2 negative — proves load-bearing); a ConfigAdmissionEvent lands per outcome (traced-test: logs_contain inside assert!, tracing-test-assert.md T-1).

## Files touched
- MODIFY: crates/core/src/config.rs (the WI-6c blake3 admission manifest + scanner + FAIL-CLOSED quarantine at ConfigLoader:205, gating the Rules/PhaseSpec slot fill; reuse the J-52 PathRuleInjector intercept)
- MODIFY: crates/plugins/src/mcp/pins.rs + mcp_pin_cmd (the WI-6a content checks folded into the existing admission path)
- MODIFY: .claude/rules/skill-import-policy.md (WI-6b: name the now-HARD PreToolUse enforcement — the policy doc's own deferred piece) + the skill load/reference path (the HARD refusal)
- CREATE: the admission scanner module (deterministic legs reusing F-1 GREP/AST; the obfuscated-injection detectors) + crates/contracts/tests/j102_config_admission_contract.rs
- CREATE: workflow/reasoning/J-102-decisions.md (the WI-6c FP observe-only delta + the settings.json-hook-registration decision + the J-52-seam confirmation)

## Citations / Grounding
- workflow/reasoning/owasp-llm-agentic-gap-analysis-2026-06-25.md Section H (the user ask, agentlint rule-set, the ELAI MCP/Skills/Rules current-state split, the LOAD-not-ADD architectural key, WI-6a/b/c/d, the HONEST RESIDUE).
- ELAI ground truth (live-code verified 2026-06-25; re-verify at edit, W-7): pins.rs McpServerPin/McpPinPolicy/enforce_pin + E-24 schema-pin + G-4 re-pin + C-42 authz [PARTIAL/WIRED] (WI-6a fold target); E-43 skill-import-policy + skillspector + skill-imports.log [DARK/SOFT] (WI-6b target); ConfigLoader core/config.rs:205 [WIRED] (WI-6c seam); J-52 PathRuleInjector [PLANNED] (the intercept hook reused); F-1 VibeGate secret-GREP/AST [WIRED] (the deterministic legs); D-13 cross-family critic [DARK] (the advisory leg). live ~268/577 (W-9).
- agentlint (MIT, Python, pip leporis-agentlint; grounded 2026-06-26 — a NARROW MCP-config-only static scanner): STEAL the MCP rule-set as DATA only, DO NOT port the pip dep (ELAI is Rust; anti-slopsquatting — agentlint is not added). OWASP LLM03/LLM04/LLM01 + ASI04/ASI06 (genai.owasp.org 2025; the threat rubric). [EXTERNAL, design-rationale.]
- Hard rules invoked: GD-1 (reuse J-52/extend E-24/E-43/F-1, never fork); GD-4 (observe-first); TD-3 (deterministic gate, advisory semantic leg); R1 (missing admission row => FAIL-CLOSED); R7/R9/R10; W-7/W-9; anti-slopsquatting (no agentlint dep).
- K-13 is the K-native PROVE+PACKAGE half (the OWASP control-mapping that records this gap); J-102 is the technical config-admission gate it recorded OUT-OF-K. J-101 is the sibling DATA-path hardening (untrusted data into sinks); J-102 is the CONFIG-path hardening (untrusted config into the instruction surface).

## Open questions / risks
- HONEST RESIDUE (Section H): semantic injection-in-instruction detection is LLM-hard (paraphrase limit, same as J-101 WI-1) -> the deterministic legs FAIL-CLOSED, the semantic leg is ADVISORY only. The deterministic win is the obfuscated-payload class (zero-width/homoglyph/encoded) + secrets + dangerous-verb directives, which IS catchable.
- settings.json hook-registration admission: a poisoned hook COMMAND is a sink too — confirm at authoring whether settings.json needs admission (recommend YES, it is part of the instruction/capability surface). // DECISION at authoring.
- J-52 PathRuleInjector is the named intercept seam (WI-6c NATURAL HOOK) — confirm it is the right seam at authoring (W-7 re-grep); J-52 is PLANNED, so the seam may need J-52 to land first OR the gate wires directly at ConfigLoader. Note the soft-dependency.
- FP risk: an over-strict admission scanner quarantines legitimate rule edits, blocking the operator's own workflow. Mitigation = OBSERVE-ONLY first + the clean-config FP guard as a hard gate + a clear re-admission path (the user re-admits a legitimate change).

## Next
Closes the absent surface (nothing scans a new/modified instruction .md before it loads into the model context) with a LOAD-time blake3 admission gate — the "no out-of-band bypass" the user asked for, enforced at LOAD not ADD. Extends the already-strong MCP identity path with content checks (WI-6a) and promotes the E-43 skill gate soft->hard (WI-6b). Pairs with J-101 (the DATA-path injection hardening) — both are the "ingress governance" theme K-13 maps. K-13 PROVES + PACKAGES the spine; J-101/J-102 harden it.

End of J-102.
