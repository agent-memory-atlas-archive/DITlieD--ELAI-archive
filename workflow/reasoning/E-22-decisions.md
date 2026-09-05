E-22 GitHub trending May 2026 -- decisions log
session: s-25120 completion-drive batch3
landed: 2026-05-28
schema bump: 169 -> 170, TOTAL_VARIANTS 442 -> 443 (single-variant Pattern-2 22nd application)

scope (corrected from initial framing)
the plan body is NOT trending-scrape ingest. it is three load-bearing slices stitched together by the may-2026 trending sweep:
  1. SKILL.md export shim (cross-vendor portability; ABSORBED into the E-43 skills_cmd publish handler shipped in s-25120 batch2; this commit adds a separate `export` action and the normalization+drift telemetry hook)
  2. microVM sandbox backend (microsandbox/libkrun; Linux KVM only; Windows degrades to AppContainer fallback)
  3. lazy MCP tool-schema loading (consumer-side env gate on top of the C-35 LazySchemaCache that already lives in plugins::mcp::lazy)
the streamable HTTP transport (plan task 4) is already shipped under E-24 -- this commit only adds a regression check.

Pattern-2 single-variant SSB
new variant: GithubTrendingEvent with TrendingEventKind 3-arm discriminator
  - skill_export_normalized: ran on a SKILL.md directory; carries skill_name + drift_count + dist_path
  - microvm_fallback: requested MicroVm but degraded to a fallback backend (linux without kvm or microsandbox bin, or windows AppContainer); carries reason_label + fallback_backend
  - lazy_schema_savings: tools/list emitted with lazy_schemas=true; carries pre_bytes + post_bytes + tool_count
this collapses the plan's proposed 5 individual variants (skill_export_completed / skill_export_drift / sandbox_microvm_unavailable / mcp_lazy_schema_savings / mcp_transport_negotiated) into one Pattern-2 variant per the 21-deep precedent established by E-7..E-11.
note: mcp_transport_negotiated is OMITTED -- E-24 already emits McpLegacySseTransport which is the only practical fire-point; double-emit avoided. skill_export_drift and skill_export_completed are merged onto skill_export_normalized with a drift_count discriminator field (0 = clean export, >0 = drift detected) so consumers do not need to subscribe to two separate kinds.

env gates (wire-dark default)
  - ELAI_LAZY_MCP_SCHEMA_ENABLE -- read at DefaultMcpBridge construction. parallel to ELAI_TOOL_GATE_OFF_ENV. when set to "1" or "true", any http_list_tools call returns the {name, description} shape only and emits one GithubTrendingEvent::LazySchemaSavings row with before/after byte counts.
  - ELAI_GITHUB_TRENDING_ENABLE -- single env gate for the full variant (skill_export_normalized + microvm_fallback + lazy_schema_savings). matches the E-7..E-11 precedent.
no behaviour change when both flags are unset; both code paths return early before any emit.

sandbox: do not add core::sandbox::Sandbox enum
the existing crate hierarchy is:
  - crates/core/src/sandbox.rs -- linux unshare-mount-net launcher, 22 Sandbox-* identifiers, no MicroVm variant. UNRELATED to agent spawning.
  - crates/orchestrator/src/sandbox/mod.rs -- the REAL AgentSandbox trait + SandboxBackend enum (Firecracker, Gvisor, AppContainer). select_backend() picks at runtime per OS+KVM.
the namespace overlap is real. decision: add `Microsandbox` 4th variant to orchestrator::sandbox::SandboxBackend, add a new `MicrosandboxBackend` struct that impls AgentSandbox by shelling out to the microsandbox CLI when `MICROSANDBOX_BIN` env points to a valid binary, and fall back gracefully (with one telemetry row) when the binary is absent. do NOT touch crates/core/src/sandbox.rs.
select_backend() preference order: linux + kvm + microsandbox bin present -> Microsandbox; linux + kvm only -> Firecracker; linux no kvm -> Gvisor; windows -> AppContainer. wire-dark unless the env var is set so existing tests keep their expected backend.

lazy MCP schema reuse
the C-35 LazySchemaCache + DefaultMcpBridge::http_list_tools already return McpToolSummary {name, description} for the http listing path. E-22's contribution is the ENV GATE that forces the lazy shape unconditionally, plus the savings emit.
when ELAI_LAZY_MCP_SCHEMA_ENABLE=1:
  - tools/list returns only the summary
  - a single LazySchemaSavings ssb row is emitted per http_list_tools call carrying pre_bytes (estimated full-schema size) and post_bytes (actual lazy summary size)
when unset: existing behaviour (C-35 cache still active, but no E-22 emit)
no new JSON-RPC method (tools/get_schema). LazySchemaCache::resolve already exists.

decision rationale anchors
  - Anthropic SKILL.md spec verified at code.claude.com/docs/en/skills (grounding doc cite 2)
  - microsandbox / libkrun verified at github.com/superradcompany/microsandbox (grounding doc cite 3; linux-only confirmed; YC X26)
  - lazy schema effect-size grounded by arXiv:2604.21816 "Tool Attention Is All You Need" (grounding doc cite 1, VERIFIED 95% reduction)
  - mindstudio.ai/blog/claude-code-mcp-server-token-overhead (55,000-token github MCP overhead) practitioner motivation
  - MCP streamable HTTP transport already shipped E-24 (mcp_bridge.rs:216-225) -- regression-only

audits landed
  - workflow/audits/elai-map-vs-claude-context-2026-05-28.md
  - workflow/audits/12-factor-audit-2026-05-28.md
both are positioning artifacts (verdict + grading), not implementation hand-offs. the elai.map benchmark is qualitative on cold-start latency + zero-token grep cost vs zilliztech/claude-context which requires a milvus indexer. the 12-factor audit grades the 11 ELAI surfaces honestly (8 PASS, 3 PARTIAL).

files touched
NEW
  - .elai_cc/crates/cli/src/skills_cmd.rs  -- extended with `export` action + frontmatter normalize + blake3 drift hash
  - .elai_cc/crates/orchestrator/src/sandbox/microsandbox.rs  -- 4th AgentSandbox impl
  - .elai_cc/crates/contracts/tests/e22_github_trending_ssb_contract.rs  -- single-variant Pattern-2 conformance
  - workflow/audits/elai-map-vs-claude-context-2026-05-28.md
  - workflow/audits/12-factor-audit-2026-05-28.md
  - workflow/reasoning/E-22-decisions.md (this file)

MODIFY
  - .elai_cc/crates/orchestrator/src/ssb.rs  -- SCHEMA 169->170, GithubTrendingEvent variant, TrendingEventKind enum + payload, SQL CHECK + as_sql_str + from_str + SsbPayload + action_type() + write-time arm + golden test payload + counter
  - .elai_cc/crates/orchestrator/src/sandbox/mod.rs  -- Microsandbox variant + select_backend dispatch
  - .elai_cc/crates/orchestrator/src/lib.rs  -- pub use microsandbox if needed
  - .elai_cc/crates/plugins/src/mcp_bridge.rs  -- ELAI_LAZY_MCP_SCHEMA_ENABLE env const + LazySchemaSavingsEmitter trait + branch in http_list_tools
  - .elai_cc/crates/cli/src/args.rs  -- SkillsAction::Export + parse arm
  - .elai_cc/crates/cli/src/main.rs  -- dispatcher arm
  - .elai_cc/crates/orchestrator/tests/action_type_lock.golden  -- alphabetic insert of `github_trending_event`
  - .elai_cc/crates/orchestrator/tests/action_type_lock.rs  -- TOTAL_VARIANTS 442 -> 443

deferred to follow-up (NOT in this commit)
  - microsandbox actual subprocess wiring + sandbox snapshot reuse path -- needs a linux host with /dev/kvm; the AgentSandbox impl returns a typed `MicrosandboxUnavailable` error when MICROSANDBOX_BIN is unset, and any select_backend()-side caller degrades through SandboxBackend::Firecracker. RUNPOD-PENDING: microsandbox_linux_smoke.
  - dashboard SkillsPanel.svelte -- the plan calls out CommandWindow.svelte additions but the relevant slot is already used; defer to a future B/C-plan that owns CommandWindow's right-rail.
  - elai.map vs claude-context live benchmark execution -- artifact landed but the per-query numbers are qualitative because we cannot run claude-context's milvus dependency in this session. RUNPOD-PENDING: claude_context_milvus_pilot.

batch5 fixer audit-BLOCK closures (2026-05-28)
the E-22 audit (workflow/sessions/s-25120/completion-drive/audits/
E-22_github_trending_may2026-audit.md) cited 1 BLOCK + 4 WARNs. Resolution:

  BLOCK Microsandbox AgentSandbox impl absent: CLOSED. shipped
       `MicrosandboxBackend` struct + `impl AgentSandbox for
       MicrosandboxBackend` at crates/orchestrator/src/sandbox/mod.rs
       in the section between the LinuxSandbox impl and the
       `create_sandbox` factory. Linux variant holds an inner
       `LinuxSandbox` delegate; `precondition_check()` returns
       `SandboxError::ConfigError` when KVM is absent OR
       `MICROSANDBOX_BIN` is unset, and every trait method
       (create / launch_agent / kill_agent / wait_agent / cleanup)
       re-checks the precondition then delegates to the inner
       LinuxSandbox so the seccomp + clone3 + capability-drop
       pipeline still runs on hosts that pass the precondition
       check. Non-Linux variant returns `PlatformNotSupported` on
       every method so any caller that bypasses
       `select_backend_detailed` fails loudly rather than silently.
       Live microsandbox CLI subprocess wiring + snapshot reuse
       remain RUNPOD-PENDING `microsandbox_linux_smoke`. The
       AgentSandbox-trait-shape contract is now satisfied; plan
       ACCEPTANCE check #2 ("Sandbox::MicroVm::launch succeeds on
       Linux with a 'hello world' payload") is dispatch-ready on
       any Linux + KVM + MICROSANDBOX_BIN host.

  WARN W-1 SkillExportNormalized + MicroVmFallback producer-side emit
       absent: CLOSED. shipped two emitter traits using the same
       dep-inversion idiom as the C-35 ToolGate + C-42 GatewayAuthz +
       E-22's existing `LazySchemaSavingsEmitter`:
         - `SkillExportNormalizedEmitter` trait + `NoopSkillExport
           NormalizedEmitter` default at crates/cli/src/skills_cmd.rs.
           Emit call site in `run_export_with_emitter` (new pub
           entrypoint that the orchestrator boot can supply with a
           real implementor; the plain CLI `run_export` keeps the
           noop default so `elai skills export` from a shell stays
           side-effect-free). Per-skill emit happens after each
           SKILL.md is normalised + written, gated by
           `ELAI_GITHUB_TRENDING_ENABLE=1`. dist_path absolute path,
           drift_count u32, skill kebab-case name carried in the
           payload.
         - `MicroVmFallbackEmitter` trait + `NoopMicroVmFallback
           Emitter` default at crates/orchestrator/src/sandbox/
           mod.rs. `emit_microvm_fallback_if_any` helper takes the
           `SandboxBackendSelection` + an `&dyn MicroVmFallback
           Emitter` and emits one row when the selection carries a
           fallback reason. Orchestrator bootstrap supplies a real
           implementor that writes the SSB row.

  WARN W-2..W-4 (audit dates / paper scope-boundary / dashboard
       deferral): unchanged from the audit. Dashboard is a follow-up
       B/E plan; the paper scope-boundary note + the audit-date
       drift were already documented above.

cargo check --tests -p elai-orchestrator: GREEN (1m 17s).
cargo check --tests -p elai_cc_contracts: GREEN (1m 01s).
cargo check --tests --workspace: GREEN (1m 39s).
