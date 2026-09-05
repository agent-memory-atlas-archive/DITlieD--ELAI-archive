# J-64: ACP transport for the agent bridges — adopt the Agent Client Protocol as an ADDITIVE transport behind a flag on the existing CLI-bridge spawn seam, so one ACP client reaches every current and future ACP agent without surrendering ELAI's broader, more-isolated direct-CLI coverage

STATUS: PLANNED (WAVE GOOSE-ACP-1; .elai_cc runtime + small panel extension; EXTENSION of B-37 — transport ONLY: zero OAuth change (I-50's), zero new wire formats (I-49's), zero removal of the direct-CLI adapters (no-removal-without-confirmation))

- Track J (repo-steal extension) | lever SMARTER-ORCHESTRATION (churn-resistance, capability-multiplier-not-weights) | grounding STRONG on the ELAI bridge/spawn seams (code-verified this session) + STRONG on ACP maturity (web-verified + crate anti-slopsquat-verified 2026-06-21); ILLUSTRATIVE on goose source (Apache-2.0, the deprecation decision) | nature EXTENSION | effort M | WAVE GOOSE-ACP-1 | depends B-37 (live) | parallel-safe with J-63 (disjoint files: J-63 builds generated_app_broker / artifact_run; J-64 builds acp_transport)

- Premise: ELAI ALREADY ships goose's "ride the vendor's existing login" mechanism, and BROADER than goose — crates/providers/src/runtime/{claude_code,codex_cli,copilot_cli,gemini_cli,kiro_cli,opencode}.rs, each spawning the vendor CLI through cli_common::spawn_isolated_with_event under Tier-C isolation, all on the B-37 RuntimeBackend enum, all contract-tested (claude_code_runtime_contract.rs / codex_cli_runtime_contract.rs / copilot_cli_runtime_contract.rs). goose does not even have kiro/opencode. So there is NO "login with any provider" gap (that was a mis-framing this session, corrected against live code). The ONLY substantive difference vs goose is TRANSPORT: ELAI hand-parses each vendor's `--output-format stream-json` dialect (N argv-builders + N parsers — claude_code.rs:68 build_argv + handle_event:90 per vendor), so every vendor flag/format change is an independent breakage and a new agent is a new hand-written adapter; goose ADDITIONALLY speaks ACP (the Agent Client Protocol — Apache-2.0 JSON-RPC over stdio, "LSP for coding agents", agent-client-protocol crate, 25+ agents by Mar 2026 including ELAI's exact bridge set + a central registry) and DEPRECATED its own direct claude_code.rs in favor of claude-agent-acp. ACP is the right DIRECTION for this subsystem — one client reaches every current-and-future ACP agent and vendor format churn is absorbed by the vendor's adapter, not by ELAI (the same unbundling LSP did for language tooling). But ACP is an editor<->agent protocol with a thick bidirectional callback surface (fs reads, permission prompts, tool approvals) versus ELAI's thin (prompt,config)->(text,usage) RuntimeAdapter, so adopting it means implementing the CLIENT callback side, not just swapping a parser. Therefore the move is ADDITIVE behind a flag — a second transport on the SAME cli_common Tier-C spawn seam, preserving the working direct-CLI adapters + the CliSubprocessSpawned telemetry + the isolation, mirroring goose keeping both families. NOT a migration. Spec-eval verdict + grounding: workflow/reasoning/steal-goose-acp-2026-06-21.md.

## Steal/build (3 legs)

LEG 1 — ACP CLIENT TRANSPORT [S1, BUILD]. Source: goose crates/goose/src/acp/provider.rs::AcpProvider (the "drive a third-party authenticated agent CLI as a backend" abstraction five goose providers ride). ELAI port: a new crates/providers/src/runtime/acp_transport.rs that, given a RuntimeFactoryConfig + an ACP-adapter argv (e.g. claude-agent-acp), spawns the adapter through the EXISTING cli_common::spawn_isolated_with_event (the SAME Tier-C isolation + the SAME RuntimeEvent::CliSubprocessSpawned row, never a second spawn seam) and speaks ACP JSON-RPC over the child's stdio via the agent-client-protocol crate (v0.15.0, Apache-2.0, anti-slopsquat-verified 2026-06-21). It implements the SHIPPED RuntimeAdapter trait (runtime/types.rs) — (prompt,config)->(text,usage) streaming — by mapping ELAI's thin completion request onto one ACP session/prompt turn and collecting the assistant text + token usage off the ACP stream. ACP sits BEHIND the existing trait; NO new RuntimeAdapter surface, NO change to how callers obtain a completion.

LEG 2 — ACP CLIENT CALLBACK POLICY [S2, BUILD, load-bearing — the security spine]. ACP is bidirectional: the wrapped agent can call BACK to request filesystem reads, permission decisions, and tool approvals. ELAI's use is a dumb completion backend, so the ACP client callback handler is RESTRICTIVE BY DEFAULT — fs-access and permission callbacks are DENIED unless the request is inside the Tier-C 4A path-jail AND explicitly allowed; any escalation beyond the jail is trust_dialog-gated (REUSE trust_dialog.rs, the CVE-2026-21852 base_url/exfil guard class, never weaken it — W-8). The callback channel is exactly the liability the additive design must contain: an ACP agent backend must NOT be able to use fs/permission callbacks to reach host FS or secrets. Missing any of deny-by-default / jail-confinement / trust-gated-escalation is a BLOCK.

LEG 3 — PER-BACKEND TRANSPORT SELECTION [S3, BUILD]. Each agent that has an ACP adapter (claude/codex/copilot/gemini/opencode per the ACP registry) gains an OPTIONAL acp transport path, selectable via config: per-backend `transport = direct-cli | acp`, DEFAULT direct-cli. W-2 (kill-switch-not-enable): the direct-CLI path stays the default-ON behavior, ACP is opt-in per backend, and a global ELAI_ACP_TRANSPORT_DISABLE hard-disables it. RuntimeFactoryConfig carries the transport choice; the B-37 factory / resolve_backend routes to acp_transport.rs vs the direct adapter. NO removal or deprecation of any direct-CLI adapter (no-removal-without-confirmation; ELAI's broader coverage incl kiro/opencode stays; goose itself keeps both families). The transport choice is surfaced beside J-30's per-backend descriptor in the RuntimePanel.

## HARD boundaries (negative space)
- NEVER rip out / deprecate the direct-CLI adapters (no-removal-without-confirmation, GD-1): ACP is purely additive; the kiro/opencode coverage goose lacks stays.
- NEVER a second spawn seam: acp_transport spawns through cli_common::spawn_isolated_with_event, reusing Tier-C 4A + the CliSubprocessSpawned row (GD-1 — reuse the shipped seam).
- NEVER an unrestricted ACP callback handler: fs/permission/tool callbacks are denied-by-default inside the Tier-C jail; escalation is trust_dialog-gated (the base_url-exfil CVE class). A callback that reaches host FS/secrets is a confinement BLOCK.
- NEVER a silent transport fallback: a backend configured transport=acp whose adapter binary is absent FAILS LOUD, never silently to direct-CLI (a silent fallback would hide a misconfiguration and is itself a finding).
- NO new SSB action_type: extend RuntimeEvent::CliSubprocessSpawned (event_sink.rs:58) with a `transport: BridgeTransport` field (DirectCli | Acp, #[serde(default)] = DirectCli for back-compat with existing rows). SCHEMA_VERSION / TOTAL_VARIANTS are DERIVED from the action_type count (ssb.rs:1508, epoch -309 + SSB_ACTION_TYPES.len()); adding a payload field, not an action_type, leaves the counter UNCHANGED.
- NO default-OFF ELAI_*_ENABLE graveyard flag: ACP is per-backend opt-in config + the kill-switch ELAI_ACP_TRANSPORT_DISABLE (W-2).
- NO change to the OAuth lane (I-50) or the wire-format lanes (I-49): ACP rides authenticated vendor CLIs, exactly like the direct bridges; it touches transport only.
- NO .unwrap()/.expect() on prod paths; blake3 only; the ONLY new dep is agent-client-protocol (user-confirmed + re-verified at edit, W-9).

## Existing ground truth (grep/read 2026-06-21; re-grep at edit, W-7)
- B-37 MULTI-RUNTIME BACKEND [WIRED]: RuntimeAdapter trait + RuntimeFactoryConfig (crates/providers/src/runtime/types.rs), resolve_backend / the factory, PerRoleRuntime, RuntimePanel (feature map; phase-4M). The factory/resolve seam is THE integration point — J-64 adds a transport branch, never a parallel registry.
- The direct-CLI bridges: claude_code.rs (build_argv:68 -> `claude -p ... --output-format stream-json --verbose --include-partial-messages --permission-mode <mode>`; handle_event:90 parses the NDJSON), codex_cli.rs, copilot_cli.rs, gemini_cli.rs, kiro_cli.rs, opencode.rs — all spawn via cli_common::spawn_isolated_with_event (cli_common.rs:330) under isolation.rs Tier-C; contract-tested (contracts/tests/{claude_code,codex_cli,copilot_cli}_runtime_contract.rs).
- RuntimeEvent::CliSubprocessSpawned { backend, pid, argv_redacted } (event_sink.rs:58) — the off-agent row J-64 extends with `transport`. Recorded through RuntimeEventSink (event_sink.rs:74), which the orchestrator implements against the SSB writer.
- SCHEMA_VERSION = SCHEMA_VERSION_DDL_EPOCH(-309) + SSB_ACTION_TYPES.len() (ssb.rs:1508); TOTAL_VARIANTS derived the same way (ssb.rs:1445). LIVE baseline ~268/577 as of 2026-06-21 (J-0_INDEX); J-64 adds NO action_type so the counter is UNCHANGED — re-read at edit (W-9).
- trust_dialog.rs CVE-2026-21852 (ANTHROPIC_BASE_URL/OPENAI_BASE_URL honored before the trust dialog = token exfil) — REUSED as the ACP callback-escalation gate (W-8, the J-63 precedent).
- I-50 SUBSCRIPTION AUTH BROKER [PLANNED] (OAuth lane) and J-30 PROVIDER BYOK [PLANNED] (key/base-url config + RuntimePanel descriptor) — J-64 is orthogonal to the first and a sibling of the second (the transport selector rides J-30's per-backend descriptor row).
- goose: crates/goose/src/acp/provider.rs::AcpProvider + claude_acp.rs (claude-agent-acp; setup text "use goose with your Claude Code subscription via the claude-agent-acp adapter"; the direct claude_code.rs is marked deprecated in its own metadata). Interface map: workflow/papers/goose-interface.md.

## REVISED scope (W-10 tiers)
SHIP NOW Tier-1 (in-process / local, no cloud key spend): the acp_transport module + its RuntimeAdapter impl + the LEG-2 deny-by-default callback policy + the LEG-3 per-backend transport discriminator + the CliSubprocessSpawned `transport` field; exercised against ONE ACP adapter (claude-agent-acp) on the user's OWN already-authenticated `claude` CLI (no new auth, no cloud key beyond the user's existing subscription — identical auth surface to the direct claude_code path today) plus a mock/fixture ACP server for the protocol + callback-confinement unit tests.
DEFER Tier-2 (per-agent, on demand): registering the remaining ACP adapters (codex-acp / copilot / gemini / opencode) — each is a one-line transport entry once the seam proves out; live verification against the user's real per-vendor logins, run on demand, never in CI.

## Integration
- NEW crates/providers/src/runtime/acp_transport.rs (~260 LOC: the ACP client over a cli_common-spawned adapter, RuntimeAdapter impl, the deny-by-default callback handler).
- MODIFY crates/providers/src/runtime/event_sink.rs (~6 LOC: BridgeTransport enum + the `transport` field on CliSubprocessSpawned, serde default).
- MODIFY crates/providers/src/runtime/types.rs + the factory/resolve seam (~40 LOC: the per-backend transport choice on RuntimeFactoryConfig + the routing branch).
- MODIFY crates/providers/Cargo.toml (+1 dep: agent-client-protocol = "=0.15.0", user-confirmed at edit).
- RuntimePanel extension + stub.ts/types.ts (+ the transport-selector Tauri read, ~80 LOC) — sits beside J-30's per-backend descriptor row.
- frontend_pair: the RuntimePanel transport selector + its Tauri command (cross_lang_diff applies).

## WIRING_CONTRACT
produces_symbols:
- crates/providers/src/runtime/acp_transport.rs::AcpTransport (implements RuntimeAdapter; speaks ACP over a cli_common-spawned adapter via the agent-client-protocol crate; deny-by-default callback handler)
- BridgeTransport enum (DirectCli | Acp) + the `transport` field on RuntimeEvent::CliSubprocessSpawned (event_sink.rs)
- the per-backend `transport` config on RuntimeFactoryConfig + the factory/resolve routing branch
consumes_symbols:
- cli_common::spawn_isolated_with_event + runtime/isolation.rs Tier-C (the spawn + isolation seam — REUSED, never re-implemented)
- runtime/types.rs RuntimeAdapter + RuntimeFactoryConfig (the trait ACP sits behind) + resolve_backend / the B-37 factory (the routing seam)
- trust_dialog.rs (the callback-escalation gate — reused, W-8)
- runtime/event_sink.rs RuntimeEvent (extended with a field, not duplicated)
reachable_from:
- the B-37 factory / resolve_backend production path -> a task routed to a backend whose config sets transport=acp -> AcpTransport::complete drives the real adapter subprocess  [PRODUCTION runtime]
- the `elai runtime` / providers configure surface that sets the per-backend transport  [PRODUCTION CLI]
- the RuntimePanel transport selector + its Tauri command  [PRODUCTION dashboard]
- a #[cfg(test)] / contracts/ caller NEVER satisfies this (R7)
stub_ban:
- NO removal / deprecation of any direct-CLI adapter (no-removal-without-confirmation)
- NO second spawn seam (reuse cli_common::spawn_isolated_with_event + Tier-C)
- NO unrestricted ACP callback handler (fs/permission denied-by-default in the Tier-C jail; escalation trust_dialog-gated)
- NO silent transport fallback (a configured acp backend with a missing adapter binary fails LOUD)
- NO new SSB action_type (extend CliSubprocessSpawned; the derived counter does not move)
- NO default-OFF ELAI_*_ENABLE (kill-switch ELAI_ACP_TRANSPORT_DISABLE only, W-2)
- NO .unwrap()/.expect() on prod paths; NO blake2/sha2; NO new dep beyond agent-client-protocol
frontend_pair:
  RuntimePanel per-backend transport selector + its inventory/read Tauri command; stub.ts/types.ts in-unit; cross_lang_diff clean. Justified.
observable_effect:
  trigger: "a backend configured transport=acp (claude-agent-acp adapter) against the user's authenticated `claude` CLI, one real completion routed through resolve_backend (e.g. `elai runtime complete --backend claude-code --transport acp`)"
  expect_ssb: "RuntimeEvent::CliSubprocessSpawned with transport=Acp naming backend=ClaudeCode (the adapter spawned through cli_common, same isolation seam); the completion's existing RuntimeBackendSelected / usage rows name the backend"
  expect_artifact: "the ACP session returns assistant text + token usage that AcpTransport surfaces through RuntimeAdapter IDENTICALLY to the direct-CLI path (a parity fixture diffs the direct-cli vs acp completion for the same prompt). NEGATIVE (R1/R2): a transport=acp backend whose adapter binary is absent FAILS loud (a journal/stderr error, never a silent direct-CLI fallback); an ACP fs-access callback outside the Tier-C jail is DENIED + logged, never honored; a callback-escape reaching host FS FAILS the confinement contract test."

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/J-plan/J-64_acp_transport_for_agent_bridges.md --run-trigger
rg "agent-client-protocol" .elai_cc/crates/providers/Cargo.toml                         # the ONE new dep (user-confirmed, anti-slopsquat-verified)
rg "AcpTransport|acp_transport" .elai_cc/crates/providers/src/runtime -l                # transport module exists + is referenced
rg "BridgeTransport|transport:" .elai_cc/crates/providers/src/runtime/event_sink.rs     # discriminator on the EXISTING event (NO new action_type)
rg -ci "silent.*fallback|fallback_to_direct" .elai_cc/crates/providers/src/runtime/acp_transport.rs   # MUST be 0 (no silent transport fallback)
python .claude/tools/cross_lang_diff.py                                                 # panel pair clean
test -f workflow/reasoning/J-64-decisions.md
```

## WARGAME BLOCKS (refine-before-execute, plan-standard)

EXPECTED_OBSERVATION LEG 1 (ACP client transport): agent-client-protocol = "=0.15.0" appears in crates/providers/Cargo.toml under [dependencies]; crates/providers/src/runtime/acp_transport.rs exists and declares pub struct AcpTransport implementing the RuntimeAdapter trait (runtime/types.rs); cargo check -p providers --all-targets exits 0; AcpTransport spawns the adapter through cli_common::spawn_isolated_with_event (same Tier-C isolation seam, never a second spawn), confirmed by a grep of spawn_isolated_with_event in acp_transport.rs.

ON_FAIL LEG 1: failure = cargo check fails on agent-client-protocol import or API mismatch; cause = crate version drift since 2026-06-19 publication or a feature-gate the SDK requires; counter-move = re-verify the crate API surface via curl -s https://crates.io/api/v1/crates/agent-client-protocol + read the crate lib.rs on github at the pinned tag, adjust the import path or feature flags, re-check. A second fail = ABORT (the dep is unsafe to pin at this version).

EXPECTED_OBSERVATION LEG 2 (callback policy): acp_transport.rs contains a callback handler that DENIES fs-access and permission callbacks by default; any callback request outside the Tier-C 4A path-jail is DENIED + logged (never honored); trust_dialog.rs is reused for any escalation (imported, not re-implemented); a confinement contract test asserts an out-of-jail fs-read callback returns Deny.

ON_FAIL LEG 2: failure = a confinement test shows a callback reaching host FS; cause = the deny-by-default guard is missing, the jail boundary check is wrong, or trust_dialog was bypassed; counter-move = harden the callback handler (add the path-jail check before any fs access), re-verify the Tier-C 4A boundary, re-run the confinement test. A callback that cannot be confined = ABORT (the security spine is violated, the plan cannot ship).

EXPECTED_OBSERVATION LEG 3 (per-backend transport selection): RuntimeFactoryConfig in types.rs carries a transport field (DirectCli | Acp, default DirectCli); resolve_backend in mod.rs has a routing branch that selects AcpTransport when transport=Acp; event_sink.rs RuntimeEvent::CliSubprocessSpawned has a transport: BridgeTransport field with #[serde(default)] = DirectCli (back-compat with existing SSB rows); all six direct-CLI adapters (claude_code/codex_cli/copilot_cli/gemini_cli/kiro_cli/opencode) are untouched; cargo check -p providers exits 0.

ON_FAIL LEG 3: failure = resolve_backend does not route to AcpTransport when transport=acp, or existing direct-CLI path breaks; cause = routing branch missing, wrong enum variant, or the serde default broke existing row deserialization; counter-move = add/fix the routing branch, verify #[serde(default)] on the transport field, run cargo nextest run -p providers to confirm existing runtime contract tests still pass.

FORK: if agent-client-protocol 0.15.0 has a different API surface than the plan assumes (the plan was grounded 2026-06-21, the crate publishes rapidly) -> re-verify the crate API at edit time (RECON_NEEDED: curl -s https://crates.io/api/v1/crates/agent-client-protocol + read the pinned version docs.rs page), adapt the AcpTransport impl to the real API, do NOT guess the API from training memory.

FORK: if an ACP adapter requires a jailed read-only FS callback to function (some agents may need it) -> default hard-deny (the plan LEG 2 default), defer the per-agent read-only-fs decision to Tier-2 (post-Tier-1). Do NOT weaken the deny-by-default policy to make one agent work.

RECON_NEEDED: re-verify agent-client-protocol crate name + version + maintenance status + license at edit time via curl -s https://crates.io/api/v1/crates/agent-client-protocol (anti-slopsquatting HARD RULE, W-9). The plan grounded this 2026-06-21; the crate may have published a new version since.

RECON_NEEDED: re-grep cli_common::spawn_isolated_with_event + RuntimeEvent::CliSubprocessSpawned + resolve_backend at edit time to confirm the API signatures match the plan 2026-06-21 anchors (W-7). Re-grep ssb.rs:1508 + ssb.rs:1445 to confirm SCHEMA_VERSION / TOTAL_VARIANTS derivation is unchanged (the plan adds NO action_type).

ABORT: agent-client-protocol crate is yanked or has a published security advisory at edit time -> STOP, do not add the dep, surface to owner.

ABORT: the ACP callback surface cannot be confined to the Tier-C jail (a callback escapes to host FS or secrets) -> STOP, the security spine is violated, do not ship the slice.

ABORT: cargo check -p providers fails after adding the dep and the failure is not fixable in one S6b fixer cycle -> STOP + handoff (do not iterate past D-9 second-fail fork).

ABORT: a direct-CLI adapter (claude_code/codex_cli/copilot_cli/gemini_cli/kiro_cli/opencode) is broken by the transport-field addition -> STOP, the no-removal boundary is violated, revert the transport field, re-scope.

VERIFY: python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/J-plan/J-64_acp_transport_for_agent_bridges.md --run-trigger (off-agent contract check, exit 0 = PASS).
VERIFY: python .claude/tools/production_caller_check.py --files <changed .rs> (off-agent wire check; AcpTransport must have a production caller via resolve_backend, exit 0 = WIRED).
VERIFY: cargo check -p providers --all-targets (compile gate, exit 0).
VERIFY: cargo nextest run -p providers --filter acp (test gate, exit 0; or cargo test -p providers if nextest absent).
VERIFY: rg -ci "silent.*fallback|fallback_to_direct" .elai_cc/crates/providers/src/runtime/acp_transport.rs MUST be 0 (no silent transport fallback, exit 1 = BLOCK).
VERIFY: a confinement contract test asserts an ACP fs-access callback outside the Tier-C jail is DENIED (off-agent assert, the test must fail if the callback is honored).

REDTEAM: attack 1 (callback escape to host FS): an ACP agent backend sends an fs-read callback for /etc/passwd or ~/.ssh/id_rsa. The deny-by-default handler must DENY + log, never honor. RESULT: attack FAILED against the plan (LEG 2 specifies deny-by-default inside the Tier-C jail; anything outside is denied). PATCH: the callback handler checks the requested path against the Tier-C 4A jail before any fs access; a confinement contract test asserts the denial.
REDTEAM: attack 2 (silent transport fallback): a backend configured transport=acp whose adapter binary is absent silently falls back to direct-CLI, hiding the misconfiguration. RESULT: attack FAILED against the plan (HARD boundaries: NEVER a silent transport fallback). PATCH: the transport selection FAILS LOUD on missing adapter binary (a journal/stderr error, never a silent direct-CLI fallback). The rg check for silent.*fallback must return 0.
REDTEAM: attack 3 (SSB back-compat break): existing CliSubprocessSpawned rows in the SSB have no transport field, so adding the field breaks deserialization of historical rows. RESULT: attack SUCCEEDED against the naive impl (a bare enum field with no default). PATCH: the transport field uses #[serde(default)] = DirectCli, so existing rows deserialize as DirectCli (the only transport that existed when they were written). Verified by re-reading ssb.rs:1508 derivation (NO new action_type, the derived counter is unchanged).
REDTEAM: attack 4 (direct-CLI removal disguised as additive): the executor, while wiring the transport branch, removes or deprecates a direct-CLI adapter (the no-removal boundary). RESULT: attack FAILED against the plan (HARD boundaries + no-removal-without-confirmation). PATCH: the cross_lang_diff + a grep for the six direct-CLI adapter files confirms all are present post-diff.

INTEGRATION_SMOKE: after LEG 1 + LEG 2 + LEG 3, drive the REAL entrypoint end-to-end on a HARD input. Spec: (a) a mock/fixture ACP server (in-process, no real adapter binary needed for the smoke) accepts one ACP session, ELAI routes a completion through resolve_backend with transport=acp, assert the CliSubprocessSpawned SSB row lands with transport=Acp (off-agent artifact, R4); (b) the completion returns assistant text + token usage identically to the direct-CLI path (a parity fixture diffs direct-cli vs acp output for the same prompt); (c) the mock ACP server sends an out-of-jail fs-read callback, assert it is DENIED + logged (the confinement contract). HARD input: a multi-turn ACP session where the wrapped agent sends both a completion response AND an out-of-jail callback in the same turn, exercising the callback handler on the live path. Per-unit green (each leg own test) is necessary, NOT sufficient; this cross-seam smoke is the R10 gate.

## Grounding
- ELAI ground truth (this session's reads): B-37 RuntimeAdapter/cli_common/resolve_backend (feature map [WIRED] + greps); the six CLI bridges + cli_common.rs:330 spawn_isolated_with_event + isolation.rs Tier-C; claude_code.rs:68 build_argv / :90 handle_event (the per-vendor parser this avoids); RuntimeEvent::CliSubprocessSpawned (event_sink.rs:58); the derived SCHEMA/TOTAL_VARIANTS (ssb.rs:1508/1445); trust_dialog.rs CVE-2026-21852; I-50 / J-30 (orthogonal / sibling).
- goose (Apache-2.0, docs/.source/goose-main, MEDIUM, ideas only — clean-room names AcpTransport/BridgeTransport are ELAI-native): crates/goose/src/acp/provider.rs::AcpProvider + claude_acp.rs (the deprecation of direct claude_code in favor of claude-agent-acp). Interface map workflow/papers/goose-interface.md.
- External (web-verified 2026-06-21): the Agent Client Protocol (agentclientprotocol.com / zed.dev/acp) — open JSON-RPC over stdio, "LSP for coding agents", v1 protocol stable, 25+ agents by Mar 2026 incl Claude Code/Codex CLI/Copilot CLI/OpenCode/Gemini CLI + a central registry. Dep agent-client-protocol = "=0.15.0" (Apache-2.0, github.com/agentclientprotocol/rust-sdk, published 2026-06-19; anti-slopsquat-verified via crates.io API 2026-06-21; the crate goose itself uses). Re-verify name/version/maintenance at edit (anti-slopsquat HARD RULE, W-9).
- Ledger honored: does NOT re-open the device-code-OAuth NOT-WORTH-PORTING ruling (OAuth stays I-50). The feature map's "ACP proc-macro covered/out-of-focus" line (ELAI-FEATURE-MAP L2024-28) referred to goose's goose-acp-MACROS code-gen crate, NOT the AcpProvider TRANSPORT pattern; J-64 is the transport, a distinct and previously un-ledgered item (this session's correction, recorded in the steal file).
- New dependency: agent-client-protocol ONLY (user-confirmed + re-verified at edit). Tier-1 needs no other dep.
- Doctrine: W-2 (kill-switch not enable), W-7/W-9 (re-grep anchors + re-verify the dep at edit), W-8 (reuse trust_dialog, never weaken), W-10 (tiered), GD-1 (compose B-37/I-50/J-30 + reuse the spawn seam, never restate), GD-5 (Windows: ACP adapters are npm-distributed binaries the user must install — fail-loud if absent, document the install step), R7, no-removal-without-confirmation.

## COMPOSES-WITH
B-37 (the RuntimeAdapter trait / factory / spawn seam / RuntimePanel this extends — transport branch, never a parallel registry), I-50 (OAuth lane — orthogonal, untouched), J-30 (the BYOK config/panel sibling; the transport selector sits beside J-30's per-backend descriptor row), Tier-C 4A (the spawn isolation reused for the ACP adapter subprocess), trust_dialog.rs (the callback-escalation gate, reused per W-8), J-63 (a sibling runtime-leg build on the spawn/sidecar seams — disjoint files, parallel-safe). Restates none.

## Open questions / risks
- The ACP CLIENT callback surface is the real effort (not the JSON): the deny-by-default policy is the safe baseline, but some agents may NEED a jailed read-only FS callback to function — decide per-agent at edit whether a Tier-C-path-jailed read-only fs callback is allowed vs a hard deny. Default hard-deny until an agent provably requires it.
- Protocol maturity: ACP is pre-1.0 (crate 0.15.0, 2026-06); pin the exact dep version and re-verify at edit (W-9); churn risk is the cost of riding an emerging standard early.
- Windows (primary host): ACP adapters (claude-agent-acp etc.) are npm-distributed and must be installed; the transport fails LOUD if the adapter binary is absent (never silent fallback), and the direct-CLI path stays the zero-extra-install default (GD-5 honest residue).
- Usage/token mapping: confirm the ACP session exposes per-turn token counts; if an adapter does not, usage is adapter-dependent on the acp path — note it as a parity caveat rather than fabricating a count.

## Next
J-64 gives ELAI goose's one genuine transport edge — one ACP client reaching every current-and-future ACP agent, with vendor format churn absorbed by the vendor adapter — without surrendering ELAI's broader, more-isolated direct-CLI coverage (kiro/opencode included). It pairs with J-30 (the config/panel surface the transport selector rides) and leaves I-50 (OAuth) and I-49 (wire formats) untouched. Once Tier-1 proves out on claude-agent-acp, each remaining ACP agent is a one-line transport entry.

End of J-64.
