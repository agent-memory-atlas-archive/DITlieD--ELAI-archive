F1c-approval-fatigue-deferral.md

Session: s-22600 Tranche F (orchestrate followup)
Date: 2026-05-31
Owner: Tranche F executor (fixer)

Decision
ApprovalFatigueGuard wiring DEFERRED to a future tranche (no code edits this pass).

Reason
The audit-followup-named insertion site (skill_templating.rs::expand_skill_template, microsandbox-absent branch) lacks per-session state plumbing — ApprovalFatigueGuard holds a VecDeque<ApprovalKind> at architecture/approval_fatigue_guard.rs:102 that must persist across approval decisions. Constructing ApprovalFatigueGuard::with_defaults() inside the fn body would create a fresh zero-window per call and the guard would NEVER fire. Threading &mut ApprovalFatigueGuard through expand_skill_template breaks every caller signature.

The substrate-doc-named site (permissions.rs:312 in prompt_or_deny) lacks a Permissions-owned ApprovalFatigueGuard field. Threading it requires adding a field to PermissionPolicy + every constructor in cli/runtime_setup.rs + conversation.rs + dashboard-v2/src-tauri (estimated ~5 call sites). That is multi-file impact-radius and out of Tranche F scope.

Will-resolve-in
Future tranche after E-24 lands per-session-state plumbing OR a separate patch-N that adds the Permissions-owned ApprovalFatigueGuard field with its 5 call-site constructor updates. The substrate (approval_fatigue_guard.rs) ships fully tested in 4 contract tests + in-module tests; the deferral is purely about plumbing, not correctness of the guard surface itself.

Stub
Tracked in workflow/sessions/s-22300/stubs.md row F1c-approval-fatigue (DOCUMENTED-DEFERRAL).

Cross-reference
F:\.ELAI_workflow\workflow\sessions\s-22600\context-package-trancheF.md DECISION-T-F-3 (explorer-resolved deferral rationale).
F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\architecture\approval_fatigue_guard.rs L11-13 (substrate doc-comment naming permissions.rs:312 + conversation.rs:1052 as canonical triggers).
