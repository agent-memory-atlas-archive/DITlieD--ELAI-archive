B-15 Tranche I Item I.2 -- orchestrator crate .unwrap() burndown NULL-RESULT
Session: s-22600 sub-phase trancheI | Date: 2026-06-01

Verdict: NULL-RESULT (no source edits). Orchestrator crate segment of B15-unwrap-burndown is VERIFIED-COMPLIANT.

Scope examined
- .elai_cc/crates/orchestrator/src/ (~140 files containing .unwrap(); raw count 784 sites).

Method (DI6 canonical methodology)
- Per file with .unwrap() count > 0: locate the canonical `#[cfg(test)]` boundary via Grep, compare to first .unwrap() line, then classify pre-boundary sites manually if any.
- Automated awk per-file scan across all ~140 orchestrator src files: ONLY deterministic_auditor.rs reports prod_count > 0 (value = 3, all already-exempt).
- Doc-comment false positives recognised across multiple files where `//` or `//!` lines contain ".unwrap()" ban-text without a real call site.

Top-15 raw site counts + cfg(test) boundary (file | raw | cfg(test) | first unwrap | verdict)
- session_dag.rs                                 | 171 | L1706 | L1714 | ALL TEST
- plan_skeleton.rs                               | 156 | L1180 | L1343 | ALL TEST
- mistake_book.rs                                | 40  | L304  | L340  | ALL TEST
- deterministic_auditor.rs                       | 33  | L1483 | L1219 | 3 PROD + 30 TEST (3 prod LazyLock-static-regex case-3 exempt)
- supervision.rs                                 | 27  | L248  | L277  | ALL TEST
- research_tree.rs                               | 16  | L2766 | L2795 | ALL TEST
- skeleton_generator.rs                          | 9   | L398  | L462  | ALL TEST
- media_validator/feedback_loop.rs               | 11  | L498  | L532  | ALL TEST (L32 doc-comment ban-text)
- tool_filter.rs                                 | 6   | L389  | L766  | ALL TEST
- ipc_accept.rs                                  | 6   | L826  | L864  | ALL TEST (L18/L709 doc-comment ban-text)
- ipc_gateway.rs                                 | 4   | L771  | L962  | ALL TEST
- osc.rs                                         | 6   | L211  | L223  | ALL TEST
- observable_effect.rs                           | 6   | L1179 | L1278 | ALL TEST
- planning_decomposition/reflection_buffer.rs    | 6   | L293  | L347  | ALL TEST
- intent_pair.rs                                 | 7   | L317  | L327  | ALL TEST

All other files (1-5 raw counts): verified by automated scan -- 0 production unwraps. Doc-comment ban-text inflates the raw count for many files (auditor_feedback_export.rs L24 module-doc, baby_step_spec_lpo.rs L32, background_task.rs L32, codeact_sandbox.rs L50, compiled_pipeline.rs L23, conformal_calibration.rs L328, anti_spiral_extended.rs L31, production_agent_rpc.rs L15, runtime_setup.rs L17/L201/L614).

The 3 prod sites in deterministic_auditor.rs (CLAUDE.md exemption case 3 -- const/static init with provably valid literals)
- L1218-1220:
    #[allow(clippy::unwrap_used)]
    static MOD_DECL: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
        regex::Regex::new(r"(?m)^\s*(?:pub\s+)?mod\s+(\w+)\s*;").unwrap()
    });
- L1267-1270:
    #[allow(clippy::unwrap_used)]
    static PY_IMPORT: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
        regex::Regex::new(r"from\s+\.(\w+)\s+import").unwrap()
    });
- L1316-1319:
    #[allow(clippy::unwrap_used)]
    static TS_EXPORT: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
        regex::Regex::new(r#"export\s+.*\s+from\s+['"]\./(\w+)['"]"#).unwrap()
    });
- Inline SAFETY rationale @L1215-1217: "SAFETY: LazyLock infallible regex (B-16B D5 exempt) -- literal pattern is statically validated; regex::Regex::new on a const-checked literal cannot panic at runtime once compiled."
- Each site carries `#[allow(clippy::unwrap_used)]` + inline B-16B D5 SAFETY comment. Already-exempt; preserve untouched.

Critical correction to stubs.md framing
- stubs.md row B15-unwrap-burndown headline cited orchestrator/session_dag.rs 171 as a prod-migration target. The 171 raw count is correct but every site is inside the #[cfg(test)] block at L1706+ (first unwrap L1714 is tempfile::tempdir().unwrap() in test helper). Zero production work for session_dag.rs.

Canonical error types (already in place; no migration target needed)
- 103 files in orchestrator/src/ import thiserror -- canonical error handling pervasive.
- Per-domain thiserror enums (sampled): AdversaryError, AgentPoolError, OperationalError, RoleDispatchError, EditCheckpointError, PromptHandlerError, ArchiveVerifyError, CryptoError, ArtifactsError, FeedbackExportError, AuditorModeError, BabyStepError, BackgroundTaskError, plus 50+ more.
- NO crate-wide unified OrchestratorError enum -- domain-scoped enums per module is the established convention. Do NOT propose unifying.

CLAUDE.md hard rule satisfied
- 781 of 784 .unwrap() occurrences sit inside cfg(test) blocks (test-exempt path).
- 3 prod sites in deterministic_auditor.rs are LazyLock-static-regex with #[allow(clippy::unwrap_used)] + B-16B D5 SAFETY rationale (const/static init exemption, case 3).
- Zero production-side .unwrap() violations remain.

Outcome
- B15-unwrap-burndown orchestrator segment closed VERIFIED-COMPLIANT-NULL-RESULT.
- No production-side work needed.
- Remaining B-15 unwrap crates per DI7: plugins + providers (recommend a future LOW-tier tranche applying DI6 methodology FIRST).
