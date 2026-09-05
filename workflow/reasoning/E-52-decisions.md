E-52 — LSP as verifier — decisions
session: s-orch-e52 (campaign continuation)
plan: F:\.ELAI_workflow\docs\thefinalglance\E-plan\E-52_lsp_integration.md

(a) SCOPE DETERMINATION — TIER-A ONLY (7-of-7 campaign drift confirmed)

E-52 returned PLAN_DRIFTED_REQUIRES_IMPROVEMENT from both dispatched agents. Drift findings:

  DRIFT-1 (HIGH): plan WIRING_CONTRACT cites `EvidenceVariant::DiagnosticProbeEvidence` — no `EvidenceVariant` enum exists. Real surface is `trait EvidenceGate` at evidence.rs:175 with per-gate structs registered in `all_gates()` + `gates_for_transition()`. The new gate would be `DiagnosticProbeGate: impl EvidenceGate`, not a variant of a non-existent enum.

  DRIFT-2 (HIGH): plan task line numbers for task_runner.rs (6680-6770, 6694, 6736) are stale by ~1800 lines. Real positions: 7210 (call site), 8357 (emit_tool_dispatch), 8385 (emit_validation_tier), 8444 (run_validate_dispatch).

  DRIFT-3 (HIGH): plan cites `auditor.rs` file — does NOT exist. Real files are `auditor_mode.rs` + `deterministic_auditor.rs` in orchestrator/src/.

  DRIFT-4 (HIGH): no `elai verify` or `elai lint` CliAction subcommand exists. Plan implies one as a Read.

  DRIFT-5 (HIGH): no LspDiagnosticsPanel or VerifierPanel.svelte exists; drawer path is src/lib/drawers/ not src/lib/components/ (same path-drift as E-29/E-33/E-36).

  DRIFT-6 (HIGH): zero existing LSP code anywhere (tower-lsp, lsp-types, multilspy, lsp_oracle.py). Adding ANY of these requires anti-slopsquatting WebSearch + version pin + explicit user confirmation per CLAUDE.md "No new dependencies without confirmation".

  DRIFT-7 (HIGH): plan adds 7 new SsbActionType variants (diagnostic_probe_started / _completed / _failed / _stalled / lsp_oracle_called / ts_bindings_drift / diagnostic_probe_evidence). 7 × 7-touchpoint lockstep + 288-variant action_type_lock.golden alphabetical regen + SCHEMA 97 → 98 atomic drop+recreate + count_is_locked canary update. ~50 atomic touchpoints. Grey-zone orchestrator/{root} edit.

  DRIFT-8 (HIGH): plan task 5 wants ts-morph dep in dashboard-v2 — not in package.json today. Needs explicit user confirmation per anti-slopsquatting.

  DRIFT-9 (MEDIUM): plan cites B-19 PostToolUse + B-29 symbol-graph as prerequisites — neither shows up as SHIPPED in projectIndex.md. B-29 Phase 1 (SCIP_DEF/SCIP_REF live in symbol_graph) IS shipped per the integration explorer; B-19 status unclear.

  DRIFT-10 (MEDIUM): codebase_mapper.py:1643 `ingest_scip` already parses .scip protobuf via scip_pb2; plan's "build SCIP reader" claim duplicates existing infra.

  DRIFT-11 (MEDIUM): 26-day-stale .claude/tools/scip_indices/ per elai_map_freshness.py — Tier-B should regen before EXECUTING the SCIP refresh probe.

  DRIFT-12 (LOW): plan's "tower-lsp 0.20.0" cite — tower-lsp last release is 2023-08, abandoned by the upstream maintainer. The current canonical async-LSP client crate in 2026 is lsp-server 0.7.9 (rust-analyzer's own choice, 2025-08). Plan needs the dep refresh.

7-of-7 plans now PLAN_DRIFTED_REQUIRES_IMPROVEMENT (E-44, E-16, E-31, E-29, E-21, E-36, E-52).

(b) WHAT TIER-A E-52 SHIPPED

  - crates/knowledge/src/diagnostic.rs (NEW, ~210 LOC)
    Defines DiagnosticSeverity 4-variant enum (Error / Warning / Information / Hint) + LSP 1..=4 scale + ProbeKind 5-variant enum (RustAnalyzer / Pyright / TscNoEmit / SvelteCheck / GenericLsp) + Diagnostic struct (path, 1-indexed line+column, severity, message, optional code) + DiagnosticProbeOutcome aggregate (kind, per-severity counters, duration_ms, diagnostics vec, is_clean(), total()). Includes 4 unit tests covering severity LSP scale + ordering, ProbeKind snake_case wire form, outcome counter consistency, diagnostic serde round-trip.

  - crates/knowledge/src/lib.rs (MODIFIED, +4 LOC)
    Adds `pub mod diagnostic;` declaration. Comment block points future implementers at the decisions doc.

  - crates/contracts/tests/diagnostic_envelope_contract.rs (NEW, ~130 LOC)
    Pins the Tier-A surface with 7 tests: severity 4-variant lock + LSP scale + wire form, ProbeKind 5-variant lock + wire form, outcome counter consistency invariant, is_clean only-errors-block rule (warnings advisory), 1-indexed line/column convention, diagnostic + outcome serde round-trip.

  - workflow/reasoning/E-52-decisions.md (NEW, this file)

(c) WHAT TIER-B DEFERRED + WHY

Per the 12 DRIFT findings:

  Task 1 (rust-analyzer SCIP refresh probe) — defer reasons: (a) .claude/tools/codebase_mapper.py already does this (DRIFT-10); (b) the SCIP-staleness-tracker belongs alongside the existing tool, not as new crate code. Tier-B optionally adds a thin wrapper if explicit user authorization arrives.

  Task 2 (PostToolUse diagnostic_probe hook) — defer reasons: (a) target line numbers in task_runner.rs drifted by 1800+ lines (DRIFT-2); (b) requires the EvidenceGate trait impl (DRIFT-1 — `EvidenceVariant` doesn't exist); (c) producer is grey-zone orchestrator/{root}. Tier-B regen elai.map + reconcile DRIFT-1/2 first.

  Task 3 (multilspy on-demand find-refs / goto-def sidecar) — defer reasons: (a) AVOIDED crate (python/elai_inference); (b) multilspy + pyright + LSP-server-binary anti-slopsquatting WebSearch verification not done; (c) HARDWARE-HEAVY per CLAUDE.md sidecar rule.

  Task 4 (ts-morph TS bindings drift gate) — defer reasons: (a) ts-morph dep needs explicit user confirmation (DRIFT-8); (b) dashboard-v2 already runs `svelte-check && tsc --noEmit` via `npm run check` so the drift signal is partly there — a thin wrapper at dashboard-v2/tools/check_ts_bindings.mjs could ship as a follow-up, but it's not strictly needed once Tier-B SvelteCheck probe lands.

  Task 5 (rust-analyzer ssr wrapper) — defer reasons: ssr is an unstable rust-analyzer feature; needs version pin + opt-in flag. Belongs in a session that lands the LSP client surface.

  Task 6 (7 new SsbActionType variants) — defer reasons: (a) 7 × 7-touchpoint surgery + 288-golden regen (DRIFT-7); (b) co-staging variants without their producers violates B-23 4I.1 dead-schema-row precedent; (c) grey-zone schema bump. Rides atomically with Tasks 2 + 4.

  Task 7 (DiagnosticProbePanel.svelte) — defer reasons: pattern matches E-21.A PagerankPanel + E-29.A drawers — graceful-empty over real producer is the right shape, but the producer must exist first (Tasks 2 + 6). Path is src/lib/drawers/ not components/ when it lands (DRIFT-5).

  Task 8 (auditor verifier-plugin integration) — defer reasons: (a) `auditor.rs` doesn't exist (DRIFT-3); real surface is auditor_mode.rs + deterministic_auditor.rs in grey-zone orchestrator/{root}; (b) needs EvidenceGate impl from Task 2.

  Task 9 (elai verify CliAction subcommand) — defer reasons: (a) DRIFT-4 — no such subcommand exists; (b) needs underlying probe traits + SSB producers from Tasks 2 + 6.

  Task 10 (visual_spatial / claim_class routing for LSP-as-grounding) — defer reasons: AVOIDED reasoning crate.

(d) WHY DiagnosticProbeOutcome + 4 + 5 VARIANT LOCK

Even though the probe pipeline is Tier-B, the envelope types lock the wire shape NOW so future producers (rust-analyzer / pyright / tsc / svelte-check) cannot diverge. The 4-variant severity tier mirrors LSP 1.x DiagnosticSeverity exactly (1=Error..4=Hint) — interop with any LSP vendor is guaranteed. The 5-variant ProbeKind taxonomy covers the four canonical verifiers ELAI's stack uses today plus a GenericLsp fall-through for future probes.

The `is_clean()` rule is the load-bearing audit gate: it checks ONLY error_count, treating warning / info / hint as advisory. The contract test pins this so a future "improve" pass that tightens to also-block-warnings cannot land silently — anyone changing the rule will see the contract assertion fail loudly.

The 1-indexed line/column convention is the dashboard / stdout readability standard. Tier-B producers MUST normalize from LSP 0-indexed to 1-indexed when emitting; the contract test pins this so a Tier-B producer that forgets the normalization fails the integration test before reaching production.

(e) DELIBERATE NON-ACTIONS

  - No new dependency added. tower-lsp / lsp-types / lsp-server / multilspy / ts-morph all deferred pending user authorization.
  - No SSB variant added. The 7 the plan would add ship atomically with their producers per the B-23 4I.1 precedent.
  - No orchestrator/{root} edit. EvidenceGate impl + task_runner emit sites defer until Tier-B.
  - No SCIP refresh script. codebase_mapper.py already has the surface.

(f) WHY DIAGNOSTIC ENVELOPE NEEDS TO LIVE IN knowledge AND NOT elai-core

The diagnostic types are knowledge-graph-adjacent (a verifier's output feeds the dkg + grounding pipelines) and not OS-primitive — putting them in knowledge keeps the elai-core surface tighter and makes the dependency direction (orchestrator depends on knowledge depends on core) natural. Tier-B may add a re-export from elai-core if the orchestrator's EvidenceGate impl needs to import without pulling all of knowledge — that's a deliberate compromise for that session to make.

END E-52 Tier-A decisions.

=====================================================================
E-52 Tier-B — LSP JSON-RPC handshake + 4 SSB variants — decisions
session: s-15320 (completion-drive cluster s-25120 session 2 plan 26 of 28)
plan: F:\.ELAI_workflow\docs\thefinalglance\E-plan\E-52_lsp_integration.md
prior: Tier-A above (s-orch-e52)
=====================================================================

(g) TIER-B WHAT — autonomous-mode rebased scope shipped

  - crates/orchestrator/src/symbol_graph/lsp_oracle.rs (MODIFIED, +~280 LOC)
    Adds LspMode enum (Probe | FullHandshake, default Probe) +
    oracle_lookup_with_mode entry point + full JSON-RPC initialize +
    initialized + shutdown + exit handshake per LSP base-protocol
    v3.17 (Microsoft-canonical spec). Hand-rolled Content-Length
    framing (write_frame + read_frame + parse_content_length) — zero
    new crate deps. tokio process + io-util features already in
    workspace Cargo.toml.
    Behaviour preservation: Probe mode is verbatim the E-40 --version
    smoke probe. Hosts without the binary stay on the RUNPOD-PENDING
    synthetic-answer path in either mode. ELAI_LSP_MODE env var
    selects mode; unset = Probe (back-compat default).
    Mocking: 7 new unit tests use tokio::io::duplex pipes to exercise
    framing without a real subprocess. Live rust-analyzer / pylsp
    handshake stays RUNPOD-PENDING per cluster handoff entry 15.

  - crates/knowledge/src/diagnostic.rs (MODIFIED, +~60 LOC)
    Adds DiagnosticProbeRunStatus sibling enum (Completed | TimedOut |
    ToolMissing, default Completed) + run_status field on
    DiagnosticProbeOutcome with #[serde(default)] for back-compat.
    Resolves DRIFT-14 from Tier-A: Timeout + ToolMissing now surface
    without breaking the locked diagnostic_envelope_contract.rs Tier-A
    shape (the existing struct stays; new field has serde default;
    contract tests still pass after explicit init).
    DRIFT-13 closure: shipped enum already uses Information (LSP 1.x
    canonical name) at diagnostic.rs:35 — new code matches; plan-side
    Severity::Info shorthand is dropped.

  - crates/orchestrator/src/ssb.rs (MODIFIED, ~150 LOC across 10 sites)
    SCHEMA_VERSION 138 -> 139 atomic bump + 4 new SsbActionType
    variants (DiagnosticProbeDispatched + DiagnosticProbeCompleted +
    DiagnosticProbeBlock + ScipRefreshCompleted) + 4 payload structs
    + 9-tuple cascade (enum + as_sql_str + from_str + SQL CHECK +
    SsbPayload variant + payload->action_type + test_payload arm +
    all_test_payloads + SCHEMA_VERSION const). Total variants
    399 -> 403. test_payloads count 333 -> 337.

  - crates/orchestrator/tests/action_type_lock.rs (MODIFIED)
    10th-site lockstep: TOTAL_VARIANTS 399 -> 403 + 4 new entries in
    ALL_VARIANTS array + 4 new arms in exhaustive_match_guard.

  - crates/orchestrator/tests/action_type_lock.golden (REGENERATED)
    4 new wire tags inserted alphabetically (diagnostic_probe_block at
    0111, diagnostic_probe_completed at 0112, diagnostic_probe_
    dispatched at 0113, scip_refresh_completed at 0311). Total
    variants header bumped 399 -> 403. Body re-numbered 0000..0402.

  - crates/cli/src/args.rs (MODIFIED, +~60 LOC)
    SymbolGraphAction::LspProbe { symbol, language, mode } variant +
    `scip lsp-probe --symbol NAME --language SLUG [--mode probe|
    full_handshake]` parser. Mirrors the safety_cmd flag-walking
    pattern for consistency.

  - crates/cli/src/scip_cmd.rs (MODIFIED, +~70 LOC)
    Dispatcher arm + run_lsp_probe handler that bridges the async
    oracle_lookup_with_mode into the synchronous CLI dispatcher via a
    per-call tokio::runtime::Builder::new_current_thread (matches the
    safety_cmd async-bridging pattern). Production caller anchor for
    the new LSP JSON-RPC handshake path — defeats
    E37_WIRE_DARK_GAP_PATTERN.

  - crates/orchestrator/src/symbol_graph/mod.rs (MODIFIED, +1 line)
    Re-exports LspMode + oracle_lookup_with_mode alongside the E-40
    re-exports so external callers (cli + contracts) have a clean
    surface.

  - crates/contracts/tests/diagnostic_envelope_contract.rs (MODIFIED,
    +~25 LOC)
    Adds DiagnosticProbeRunStatus import + explicit run_status field
    on the 3 existing struct literals + new
    outcome_run_status_back_compat_defaults_to_completed test that
    pins legacy-JSON deserialization without the field defaults to
    Completed.

  - crates/contracts/tests/e52_lsp_integration_contract.rs (NEW, ~17 KB)
    E-52 Tier-B contract test (6 sections, 16 tests):
      F1 SCHEMA_VERSION_PUBLIC floor at 139
      F2/a-e DiagnosticProbeRunStatus 3-variant + serde + back-compat
      F3/a-b LSP JSON-RPC framing parser (canonical + malformed)
      F4/a-e LspMode default + Probe/FullHandshake RUNPOD-PENDING
             back-compat + OracleKind taxonomy unchanged
      F5/a-e 4 new SSB variants discriminator-preserving round-trip
      F6     E-40 LspOracleConsulted reuse pin

  - workflow/reasoning/E-52-decisions.md (THIS APPEND)

(h) DELIBERATE NON-ACTIONS — Tier-B autonomous-mode scope discipline

  - NO new dependency added. lsp-server / lsp-types / multilspy / ts-
    morph all deferred per CLAUDE.md "no new deps without user
    confirmation". Hand-rolled Content-Length framing (~120 LOC) is the
    zero-new-dep path the integration explorer flagged as the natural
    default.
  - NO dashboard panel. Plan line 108 frontend_pair: null is honoured;
    handoff line 81 tension flagged in context-package WARN-2 but the
    plan's explicit value wins absent user override.
  - NO main.rs RunArgs / D-plan substrate edits. Session 3 territory
    per cluster handoff.
  - NO multilspy sidecar (Task 6 of plan). Deferred to patch-N pending
    explicit user multilspy authorization.
  - NO ts-morph TS bindings gate (Task 9 of plan). Deferred to patch-N.
  - NO PostToolUse hook registration in task_runner.rs (Task 2 of
    plan). The diagnostic-probe surface envelope + sibling enum +
    SSB variants ship NOW so a future patch-N can wire the producer
    without breaking the contract. Per B-23 4I.1 dead-schema-row
    precedent: the 4 variants ship because their producer (lsp_oracle
    handshake + future probe dispatcher) is in this commit; the
    PostToolUse hook is the consumer.
  - NO DiagnosticProbeGate registration in evidence.rs::all_gates()
    (DRIFT-1 closure). Per the dead-schema-row precedent, the gate
    co-stages with the task_runner hook in the same patch.
  - NO .claude/tools/scip_refresh.py wrapper. Variant ships dark
    pending a future commit that wraps `rust-analyzer scip` +
    codebase_mapper.py --merge-scip (codebase_mapper already has
    ingest_scip via scip_pb2 per DRIFT-10).

(i) KEY DECISIONS

  1. LspMode opt-in over feature-gate: a runtime enum + env var (vs
     `cfg(feature = "lsp_jsonrpc")`) keeps the handshake code in one
     binary and lets the auditor flip modes without recompile. The
     Probe-mode default keeps every existing call site behaving
     verbatim per the E-40 contract.

  2. DiagnosticProbeRunStatus as SIBLING enum (not breaking change):
     adding it as a 4th outcome variant would have invalidated the
     locked Tier-A diagnostic_envelope_contract.rs which constructs
     `DiagnosticProbeOutcome` as a struct literal. The sibling enum +
     `#[serde(default)]` field is the back-compat path that closes
     DRIFT-14 without forcing a contract-test rewrite that loses the
     Tier-A pin semantics.

  3. Severity::Information (LSP-canonical) wins DRIFT-13: the shipped
     enum at diagnostic.rs:35 already uses the LSP 1.x canonical name.
     Plan-side `Severity::Info` shorthand is dropped — code matches
     spec, not plan.

  4. Sub-verb naming: `scip lsp-probe` (kebab in CLI surface) maps to
     `SymbolGraphAction::LspProbe` (PascalCase enum) per existing
     scip-cmd convention.

  5. Async bridging in CLI: per-call current-thread runtime. Avoids a
     workspace-wide runtime redesign and matches the safety_cmd
     pattern. No new dep.

  6. SSB variant count 4 (not 7): per DRIFT-7 / context-package
     INFO-1, the 4 shippable variants are diagnostic_probe_dispatched
     + _completed + _block + scip_refresh_completed. The 3 lsp_server_*
     variants (spawned + idle_shutdown + ram_ceiling_exceeded) defer
     with the multilspy sidecar producer per the dead-schema-row rule.

  7. Cite-discipline: LSP base-protocol v3.17 is Microsoft-canonical
     (GROUNDED, no arXiv escalation). The plan-cited papers
     (arXiv:2510.22907 Lanser-CLI, arXiv:2508.03830 If-T, Preprints
     202510.0924 Code Retrieval, arXiv:2306.10763 MGD) are NOT inlined
     as code-level grounding for this Tier-B ship — only the LSP spec
     itself drives the handshake shape. Per session-2 handoff
     cite-discipline (E-13 onward).

(j) EXPECTED TEST OUTCOMES

  - cargo check --workspace: CLEAN (no new warnings beyond the
    pre-existing main.rs RunArgs.empire/export_ssb E0027/E0063 which
    session 3 owns).
  - cargo check -p elai-orchestrator: CLEAN.
  - cargo check -p elai-cli: CLEAN modulo session 3's pre-existing
    RunArgs surface.
  - cargo check -p elai_cc_contracts --test
    e52_lsp_integration_contract: CLEAN (16 tests defined).
  - Existing diagnostic_envelope_contract still passes (counters +
    is_clean + 1-indexed pins + new back-compat test).
  - action_type_lock golden regenerated atomically.

(k) UNKNOWNS (escalate to follow-on session)

  - Real rust-analyzer initialize-response schema variance: LSP 3.17
    permits servers to send window/logMessage notifications before the
    initialize response. The handshake loop skips those, but a real
    server may emit non-standard frames (e.g. progress notifications)
    that our skip-loop also tolerates by reading-until-id-match. Live
    handshake against rust-analyzer 0.4 is the canonical verification
    (RUNPOD-PENDING entry 15).
  - tokio::process::Child kill_on_drop semantics on Windows host
    differ from Linux (Windows uses TerminateProcess which leaves
    handles in an indeterminate state on rust-analyzer 0.3.x). The
    `start_kill()` after the handshake limits exposure but a Windows
    integration run may surface zombie processes.
  - The Content-Length max-cap of 16 MiB is well above any rust-
    analyzer initialize response observed in practice (~50 KB) but
    misbehaving servers could trip the cap; raise the cap if a real
    workload surfaces.

END E-52 Tier-B decisions.
