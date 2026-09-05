I-47: verifier-input hardening + dispatch rule injection — decisions

What: XML data-plane tagging of untrusted inputs at the auditor/critic
dispatch seam + frozen per-role rule-injection block at the worker
dispatch seam + PromptHardeningApplied SSB variant.

Seam: crates/tools/src/lib.rs execute_agent_with_spawn, the single
production caller of build_agent_prompt (W-7 confirmed: only one
production call site; task_runner.rs uses pool.spawn not build_agent_prompt).

Key decisions

D-1: OnceLock SSB slot in tools crate (mcp_rugpull_gate.rs pattern).
  Chosen over threading Arc<Ssb> through execute_agent_with_spawn signature
  because signature change would require updating 4+ test call sites and
  widens the public interface. The OnceLock approach is zero-signature-churn
  and matches the established pattern in mcp_rugpull_gate.rs + lock.rs.
  Bootstrap installs in: runtime_setup.rs (after mcp_rugpull_gate line) +
  live_cli.rs (alongside existing set_ssb_writer calls).

D-2: Role classifier at injection site, not inside normalize_subagent_type.
  normalize_subagent_type maps "verification"/"verify"/"verifier" to
  "Verification" but has no arm for "auditor"/"critic" (falls through to
  trimmed passthrough). Rather than widening normalize_subagent_type (which
  would affect tool filtering and other downstream logic), I-47 adds a local
  fn classify_dispatch_role(normalized: &str) -> DispatchRole that matches
  on both the normalized canonical names AND any alias strings. This is
  zero-blast-radius on the normalize path.

D-3: Mechanism A injects a hardening frame as a &'static str prefix prepended
  to the existing input.prompt before build_agent_prompt is called. The frame
  lives as HARDENING_FRAME const so it is compile-time frozen (R8).
  Self-attestation detection: simple str::contains check for a set of frozen
  keyword patterns (SELF_ATTESTATION_PATTERNS const slice). When detected,
  an additional <ELAI_SELF_ATTESTATION> sub-tag is appended after the
  main <ELAI_DATA> block with a note. This avoids any runtime allocation
  beyond what the prompt already requires.

D-4: Mechanism B rule block is a frozen per-role &'static str in a match arm
  in worker_rule_block(normalized: &str) -> &'static str. Text is the
  Karpathy-distillate (read-before-write, scope-to-request, surgical-changes,
  no-premature-abstraction, no-future-proofing, no-while-I-am-here-refactoring).
  Pushed to template_config.extra_constraints before build_agent_prompt call.
  HD-2: the bytes are identical on every call for a given role because the fn
  returns a &'static str keyed only by the role string (no task/context input).

D-5: SSB emit is done via the process-global slot (D-1). The emit is a
  tokio Handle::try_current().spawn(async move {...}) to avoid blocking the
  synchronous dispatch path. When no SSB writer is installed (contract tests,
  cold start), the row is silently dropped — consistent with mcp_rugpull_gate.

D-6: SCHEMA_VERSION bump from 244 to 245 (live re-read confirmed 244).
  TOTAL_VARIANTS bump from 553 to 554 (live re-read confirmed 553).
  golden regen via cargo test -p elai-orchestrator action_type_lock -- --ignored.

D-7: e45_cross_language_drift_contract mirrors TOTAL_VARIANTS; must be updated
  to 554 at the same changeset.

Expected test outcomes

  cargo check --workspace --tests: exit 0
  cargo test -p elai_cc_contracts --test i47: 5 arms pass
  cargo test -p elai-orchestrator action_type_lock: 2/2 pass (245/554)
  cargo test -p elai_cc_contracts --test e45: exact lock 554 pass
  i32 contract: 7/7 pass (role seam unmodified)
  e37 contract: pass (PromptArmor upstream not touched)
  i46 contract: 17/17 pass (BashRedirected unmodified)
  i41, i35, i1, i33, h39: pass (no shared files)

Unknowns / risks

  RISK-1 (WIRE-DARK-MACHINERY): SSB slot must be installed at runtime_setup.rs
  bootstrap, not only in the contract test. Verified by the contract test's
  install_prompt_hardening_ssb call + the NEGATIVE arm that fails when no row
  lands.
  RISK-2 (HD-2): worker_rule_block returns &'static str; if accidentally any
  format!() or String is introduced, the byte-stable assertion will catch it.
