I-40 decisions — trust-layer wrapper + RRA spec
2026-06-15 (initial), 2026-06-29 (final-state update, Step 6/6)

WHAT
Six deliverables shipped across 6 steps:
  1. docs/spec/rra-spec-v1.md (84 LOC) — the open .rra format spec: C1-C6 required
     conformance checks, I1-I4 informational items, I-18 StackHash, I-45 objective_hash.
  2. crates/cli/src/wrap_cmd.rs (486 LOC) — `elai wrap -- <agent cmd>` trust-layer proxy.
     Fixture mode (deterministic, GPU-free) + subprocess mode (line-oriented stdio
     interception). Composes G-1 taint_text + F-2/G-3 evaluate_vibegate + G-2 build_rra
     + G-9 redact_payload (GD-1 — no new trust primitives).
  3. crates/cli/src/rra_verify_cmd.rs (208 LOC) — `elai rra verify <path>` conformance
     checker. C1-C6 required checks (exit nonzero on failure) + I1-I4 informational.
  4. Supporting wiring: args.rs (CliAction::Wrap variant + RraAction::Verify variant),
     rra_cmd.rs (RraAction::Verify dispatch arm + render_event_stream for Step 5),
     main.rs (mod wrap_cmd + mod rra_verify_cmd + CliAction::Wrap dispatch at :710-712).
  5. crates/cli/src/rra_cmd.rs render_event_stream (Step 5) — text viewer enhancement
     that renders wrapped-run event rows (taint_flow + run_artifact_emitted) in
     `elai rra show`. Dashboard RraViewerPanel already surfaces the beacon via
     get_run_artifact (no session filter) — frontend_pair satisfied without a new
     Svelte component.
  6. Contract tests (authored by auditor per R7/PCG-3):
     - crates/contracts/tests/i40_trust_wrapper_contract.rs (573 LOC, 5 arms):
       (a) real-db TaintFlow + RunArtifactEmitted row emission,
       (b) kill-switch suppresses RRA not taint,
       (c) G-9 redaction: no secret bytes in emitted payloads,
       (d) rra verify conformant fixture,
       (e) rra verify rejects malformed bundle (exit nonzero).
     - crates/contracts/tests/i40_viewer_render_contract.rs (206 LOC, 2 arms):
       (a) event stream contains taint_flow rows,
       (b) event stream contains run_artifact_emitted beacon.

KEY DECISIONS

FORK-1 resolution: `elai wrap` = NEW top-level CliAction::Wrap (NOT extending run_subcommand).
  Rationale: run_subcommand is the ELAI-internal generator loop seal; the trust-layer wrapper
  wraps EXTERNAL agent traffic. Same physical machine but different SSB session and different
  trust posture. The wrapper constructs its own Arc<Ssb> with WRAP_SESSION_ID. Grounding:
  context-package-I-40.md PRODUCTION_HOOK: main.rs CliAction dispatch at :671-699.

FORK-2 resolution: `elai rra verify` = DISTINCT from `elai replay --verify`.
  - `elai rra verify` (rra_verify_cmd.rs): checks C1-C6 structural requirements from
    docs/spec/rra-spec-v1.md. Format check only; does not re-derive content hashes.
  - `elai replay --verify` (rra_cmd.rs): re-computes blake3 root + every event content hash.
    Content tamper check only; does not validate schema version.
  A bundle can be schema-conformant but content-tampered (passes verify, fails replay --verify)
  or content-intact but schema-old (passes replay --verify, fails verify). Orthogonal.

NO new SSB action_type (GD-1 compliance).
  Reuses TaintFlow (kind=Web for untrusted ingest, kind=File for destructive-write-block)
  and RunArtifactEmitted for the wrap RRA beacon. action_type_lock test confirms 2/2 pass.

G-9 redaction precondition.
  Every write_payload call in wrap_cmd.rs has redact_payload called on the payload BEFORE
  the ssb.write_payload await. This is the mandatory G-9 precondition for all exported RRA bundles.
  The build_rra path in emit_wrap_rra reads already-redacted stream bytes from SSB, so no
  second redaction pass is needed (which would corrupt the tokens).

Kill-switch ELAI_WRAP_RRA_DISABLE (R9 compliance).
  Default-ON RRA emit. Type=kill_switch (not experiment) because the ON path is the correct
  default and the kill-switch is for ops-safety (envs that cannot write to disk).
  Registered in wire-dark-allowlist.toml with expiry 2027-12-01.

blake3 only.
  All hash computations use blake3::hash (imported via the blake3 crate). No sha2/blake2.
  The 16-hex prefix pattern (h.chars().take(16)) mirrors the established rra.rs convention.

EXPECTED TEST OUTCOME
  - cargo check --workspace: PASS (0 errors, pre-existing warnings only)
  - cargo test -p elai-orchestrator --test action_type_lock: 2 passed, 0 failed
  - cargo test -p elai_cc_contracts --test e45_cross_language_drift_contract: 14 passed, 0 failed
  - production_caller_check.py: PASS (0 DARK fns; UNSOUND-NAME on `run` is correct
    -- "run" is too common for name-grep but the caller is CliAction::Wrap => wrap_cmd::run
    in main.rs:711; rra_verify_cmd::run_verify_conformance is WIRED from rra_cmd.rs:86)
  - gate_freeze_check.py: OK (51 files locked, sha3_256 hash stable)

FINAL SHIPPED STATE (Step 6/6, 2026-06-29)
  All 6 steps of I-40 are shipped:
    Step 1: docs/spec/rra-spec-v1.md (84 LOC) — .rra format spec with C1-C6 + I1-I4.
    Step 2: crates/cli/src/wrap_cmd.rs (486 LOC) — fixture + subprocess proxy modes.
    Step 3: crates/cli/src/rra_verify_cmd.rs (208 LOC) — conformance checker.
    Step 4: Supporting wiring in args.rs, rra_cmd.rs, main.rs.
    Step 5: rra_cmd.rs render_event_stream + i40_viewer_render_contract.rs (206 LOC).
    Step 6: This decision doc updated to final state.
  Contract tests: BOTH authored (previously DEFERRED, now shipped):
    - i40_trust_wrapper_contract.rs (573 LOC, 5 arms) — the smoke_test named in
      wire-dark-allowlist.toml:wrap_fixture_emits_taint_flow_and_rra_bundle.
    - i40_viewer_render_contract.rs (206 LOC, 2 arms) — frontend_pair data substrate.
  GD-1 compliance: NO new SSB action_type. Reuses TaintFlow + RunArtifactEmitted.
  G-9 compliance: redact_payload called before every write_payload in wrap_cmd.rs.
  R9 compliance: ELAI_WRAP_RRA_DISABLE registered in wire-dark-allowlist.toml (kill_switch).
  stub_ban compliance: no unredacted RRA export, no new trust primitives, no external-agent
    matrix beyond one fixture agent (Tier-2 DEFERRED per spec).

UNKNOWNS
  - Full contracts suite: OOM during parallel linker (pre-existing Windows memory pressure;
    `STATUS_STACK_BUFFER_OVERRUN` on f9_ssb_roundtrip_contract and h47_eval_firewall_contract).
    NOT caused by this changeset. Affected tests are the same as in prior sessions.
  - Tier-2 DEFERRED: full external-agent matrix (multiple real MCP server implementations
    beyond one fixture agent) — needs real MCP servers; user-approval gated as a new-network
    integration. Sigstore/Rekor push for external-agent RRAs — compose after I-18 ships.
