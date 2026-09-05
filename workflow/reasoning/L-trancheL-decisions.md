L-trancheL-decisions -- s-22600 Tranche L (F.1.c production wire)

Date: 2026-06-01
Session: s-22600
Tranche: L (singletons-batch follow-up; resolves Tranche F DECISION-F-EXEC-3 deferral of F.1.c)
Status: SHIPPED. Workspace cargo check --tests GREEN (warnings only, no errors, no new approval-fatigue-related warnings).

================================================================================
SCOPE
================================================================================
Wire the ApprovalFatigueGuard substrate (shipped Tranche C C.1.b) into the
production ConversationRuntime turn-loop so that per-session auto-approval
fatigue at the PermissionPolicy boundary forces explicit user reconfirmation
when the rolling-window allow-rate crosses the threshold.

================================================================================
DECISION-L-1 -- plumbing topology: Option D (ConversationRuntime owns guard)
================================================================================
PICKED: Option D from context-package-trancheL.md L74-104.
REJECTED: Option A (field on PermissionPolicy), Option B (thread through
  drive_task), Option C (thread_local / per-session-id singleton map).

WHY:
1. Tranche I "simplicity wins over flexibility" + Tranche F "minimum blast
   radius": Option D adds ONE optional field to ONE struct on ONE crate; zero
   changes to PermissionPolicy / Permissions / skill_templating / any 14
   PermissionPolicy::new test fixtures.
2. Preserves Tranche F precedent (DECISION-T-F-4): guard is stateless or
   caller-owned; permissions.rs stays SSB-pure; the observation site is the
   SSB-emitter-bearing caller. Option A would invert this precedent.
3. Matches substrate doc-comment intent (approval_fatigue_guard.rs:14-17)
   + cost/free_tier_guard.rs:103 FreeTierGuard precedent: "one instance per
   orchestrator session". ConversationRuntime IS the orchestrator-session-owner
   (holds Session by value).

================================================================================
DECISION-L-2 -- observation site is conversation.rs:449-462 (post-authorize)
================================================================================
PICKED: insert observation block IMMEDIATELY AFTER the
  `self.permission_policy.authorize_with_context(...)` chain returns into
  `permission_outcome` (now a `let mut`), and BEFORE the
  `let result_message = match permission_outcome { ... }`.
REJECTED: permissions.rs::authorize_with_context site (inverts F.4 precedent;
  permissions.rs has no SsbEmitter handle + no per-session state).
REJECTED: skill_templating.rs::expand_skill_template site (substrate
  doc-comment is aspirational here -- skill_templating has zero permission
  decisions in it).

WIRE-IN LOGIC:
  if let Some(guard) = self.approval_fatigue.as_mut() {
      if approval_fatigue_guard_enabled() {
          let kind = match &permission_outcome {
              PermissionOutcome::Allow      => ApprovalKind::Allow,
              PermissionOutcome::Deny { .. } => ApprovalKind::Deny,
          };
          let verdict = guard.record_and_check(kind);
          if let ApprovalFatigueVerdict::Refused { reason, .. } = &verdict {
              permission_outcome = PermissionOutcome::Deny { reason: reason.clone() };
          }
      }
  }

================================================================================
DECISION-L-3 -- double-gate (field-Some AND env-truthy)
================================================================================
The wire-in fires only when BOTH gates pass:
  (1) `self.approval_fatigue` is `Some(_)` -- operator has called
      `with_approval_fatigue(...)` on the builder (CLI does this
      unconditionally; tests + library users default to None);
  (2) `approval_fatigue_guard_enabled()` returns true -- the
      `ELAI_E25_APPROVAL_FATIGUE_GUARD_ENABLE` env var is set to a truthy
      value (1/true/yes/on, case-insensitive).
RATIONALE: matches F.2 precedent for `llm_guard_config` (presence-of-config
gates the feature) AND adds the env gate as a fine-grained operator kill
switch so the guard can be disabled without rebuilding/redeploying the CLI.
Default OFF (env-gate dormant), so the CLI's unconditional builder call is
wire-dark.

================================================================================
DECISION-L-4 -- override semantics: refuse the CURRENT prompt, not the NEXT
================================================================================
The substrate doc-comment (approval_fatigue_guard.rs:5-8) says "force
explicit user reconfirmation on the next prompt". The wire-in overrides on
the CURRENT prompt instead. JUSTIFICATION (per context-package L121-129):
a 90%-auto-approve pattern would otherwise let this prompt through too. The
refusal-on-current-prompt semantic matches what the substrate's Refused
verdict carries (a Deny reason that conveys the rate + threshold + window),
and is what a user actually wants when fatigue is detected: stop right now,
not next time.

================================================================================
DECISION-L-5 -- SSB emit: caller-side-deferred (E1 pattern from F.1.a)
================================================================================
The wire-in does NOT emit a ClaudeCodeArchitectureEvent row from within
ConversationRuntime. Per Tranche F DECISION-T-F-4 + the F.1.a precedent
for BashComplexityGuard (which also has SSB-emit deferred per Tranche F
audit -- the Deny.reason carries the structured payload, and a downstream
SSB-aware caller emits when it picks up the Deny):
  - the Refused verdict's reason string (e.g.
    "approval_fatigue: observed_rate_pct 100 >= threshold_pct 80 over window of 50")
    is sufficient signal for a downstream emitter;
  - adding `ssb_emitter: Option<Arc<dyn SsbEmitter>>` to ConversationRuntime
    is a separate concern (higher blast radius; multi-call-site builder
    update) and pairs naturally with the BashComplexityBlocked emit gap.
Tracked in stubs.md as `F1c-approval-fatigue-emit` -- DOCUMENTED-DEFERRAL,
will_resolve_in "future guards-SSB-emit patch alongside F1a-bash-complexity-emit".

================================================================================
DECISION-L-6 -- contract test surface: 5 + 2 backfill
================================================================================
NEW FILE: .elai_cc/crates/contracts/tests/l_approval_fatigue_prod_contract.rs (~250 LOC)
  T1: ConversationRuntime default has approval_fatigue: None (back-compat
      guarantee for every existing fixture).
  T2: with_approval_fatigue installs the guard (builder semantics).
  T3: field Some + env=1 + window full at 100% allow-rate -> N+1th verdict
      is Refused; reason starts with "approval_fatigue:" prefix (the
      caller-side contract for SSB-emit deferral).
  T4: env=0 -> approval_fatigue_guard_enabled() returns false (the wire-in
      short-circuits; guard stays dormant even when field is Some).
  T5: rolling-window eviction is COUNT-based, NOT timestamp / cool-down
      (5-Allow window followed by 4-Deny pushes rate to 20%, then one more
      Deny to 0% -- the "reset" mechanism the wire-in relies on).

BACKFILL: .elai_cc/crates/contracts/tests/e25_guards_contract.rs +30 LOC
  approval_fatigue_guard_canonical_constructor_is_with_defaults: locks
    ApprovalFatigueGuard::with_defaults() as the CLI install path + asserts
    first observation does not refuse.
  approval_fatigue_env_gate_is_off_by_default: best-effort assertion that
    the default-OFF wire-dark posture is preserved (only fires when the
    test runner inherits an unset env var, so other tests in the same
    binary that mutate the env don't poison this assertion).

ENV-ISOLATION PATTERN (per WARN-L-3 in context-package):
  Per-test EnvScope (RAII set/clear) wrapped around a static
  `ENV_LOCK: Mutex<()>` so parallel-test execution does not race on the
  process-global env. Pattern mirrors c47b_case_base_persist_contract.rs.
  Lock-poison falls through via `unwrap_or_else(|p| p.into_inner())` so a
  sibling panic does not cascade.

================================================================================
SCHEMA + WIRE-DARK
================================================================================
SCHEMA_VERSION_PUBLIC: UNCHANGED at 195 (ssb.rs:2731). Reuses the SHIPPED
  Pattern-2 single-variant ClaudeCodeArchitectureEvent + the SHIPPED
  ClaudeCodeArchitectureEventKind::ApprovalFatigueDetected arm + the SHIPPED
  ClaudeCodeArchPrimitive::ApprovalFatigueGuard mapping (claude_code_arch.rs
  per Tranche C C.1.b). ZERO new SsbActionType variants.
TOTAL_VARIANTS: UNCHANGED (no new Kind arms).
WIRE-DARK: default OFF via double-gate (field-Some + env-truthy). CLI
  always installs the field; env defaults to unset so production traffic
  stays dormant.

================================================================================
FILES TOUCHED
================================================================================
1. .elai_cc/crates/orchestrator/src/conversation.rs
     +1 use import (approval_fatigue_guard::{enabled fn + ApprovalFatigueGuard +
        ApprovalFatigueVerdict + ApprovalKind}).
     +1 struct field (`approval_fatigue: Option<ApprovalFatigueGuard>` with
        7-line doc-comment).
     +1 struct-init line in new_with_features.
     +1 builder method `with_approval_fatigue` (10-line doc + 4-line body).
     +1 test-only accessor `approval_fatigue_is_installed` (gated under
        `#[cfg(any(test, feature = "test-support"))]`).
     +28 lines observation block at the authorize_with_context post-return
        site (~L449-490). `permission_outcome` binding flipped from `let`
        to `let mut`.

2. .elai_cc/crates/cli/src/runtime_builder.rs
     +6 lines opt-in `.with_approval_fatigue(ApprovalFatigueGuard::with_defaults())`
        on the live-CLI ConversationRuntime construction chain (after
        with_output_config + with_hook_progress_reporter). Wire-dark since
        env-gated.

3. .elai_cc/crates/contracts/tests/l_approval_fatigue_prod_contract.rs (NEW)
     5 tests (T1-T5) per DECISION-L-6 above.

4. .elai_cc/crates/contracts/tests/e25_guards_contract.rs
     +30 lines: 2 backfill tests per DECISION-L-6 above.

5. workflow/sessions/s-22600/stubs.md
     +1 row `F1c-approval-fatigue` (DOCUMENTED-DEFERRAL -> WIRED-OPTION-D).
     +1 row `F1c-approval-fatigue-emit` (NEW DOCUMENTED-DEFERRAL; will_resolve_in
        future "guards-SSB-emit" patch alongside F1a-bash-complexity-emit).

================================================================================
VERIFICATION
================================================================================
cargo check -p elai-orchestrator      -> PASS (14 pre-existing warnings; ZERO new).
cargo check -p elai-cli               -> PASS (1 pre-existing warning; ZERO new).
cargo check --workspace --tests       -> PASS (warnings only, no errors,
                                            no new approval-fatigue warnings).

No cargo test runs (per user constraint: cargo check --workspace --tests only).

================================================================================
OPEN FOLLOW-UPS
================================================================================
- guards-SSB-emit patch: lift the caller-side SSB emit into a single
  patch covering BOTH F.1.a (BashComplexityBlocked) and F.1.c
  (ApprovalFatigueDetected). The wire-in already surfaces the structured
  reason prefix; a downstream caller (likely in conversation.rs or the
  tool_result construction site) can pick this up and emit via the
  existing ssb_emitter pathway when one is plumbed onto ConversationRuntime.
- elai.map regen: pending Tranche H regen task (per
  activeContext.md:48). Tranche L touches conversation.rs + runtime_builder.rs
  + 2 contract test files; map drift is bounded.
