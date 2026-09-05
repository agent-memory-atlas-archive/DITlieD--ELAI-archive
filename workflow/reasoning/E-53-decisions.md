E-53 formal-verification codegen — decisions log

Session: s-15320 (executor of completion-drive cluster s-25120 plan 27 of 28)
Plan: F:\.ELAI_workflow\docs\thefinalglance\E-plan\E-53_formal_verification_codegen.md
Context package: workflow/sessions/s-25120/completion-drive/context/E-53-context-package.md
Exploration: workflow/sessions/s-25120/completion-drive/exploration/E-53-integration.md

WHAT SHIPPED (lean rebase per autonomous-mode handoff guidance)

1 SSB variant: SsbActionType::FormalVerifierCompleted at SCHEMA_VERSION 140 (TOTAL_VARIANTS 403 -> 404). Single atomic +1 bump per the C-52 DafnyVerifyCompleted single-variant precedent (the plan originally proposed 13 variants — consolidation per exploration GAP-4).

VerifierKind enum extension: added Z3SmtCheck as the 5th variant (4 -> 5) via the L3-lock escape clause documented in verification_result.rs:33-35 ("New kinds land via plan-level schema bumps"). Plan-level SCHEMA bump justifies the unlock.

csq.rs (~210 LOC): runtime Rust impl of the CounterfactualClass classifier per DRIFT-2 BLOCK closure. CLAUDE.md prose CSQ rule remains AUTHORITATIVE for LLM behavior; this module is the runtime check the orchestrator consults. Heuristic: path-keyword match on auth / login / credential / token / password / secret / crypto / cipher / sign / verify_sig / money / payment / billing / balance / parser / deserialize / untrusted / sandbox_escape returns CounterfactualRisky. NEVER promotes to CounterfactualSafe — only the formal verifier can attest Safe.

formal_verify/ module (3 files, ~420 LOC total):
  - mod.rs: facade + 3 GROUNDED inline arXiv cites (2604.22601 NL2VC re-used from C-52, 2501.03124 PRMBench, AlphaProof Nature 2025); 11 INGEST-required papers DEFERRED to mistakes.md cite-incident #11.
  - types.rs: VerifierKind (Z3Smt / Dafny / Kani / Verus toolchain enum — distinct from verification_result::VerifierKind which is the consensus-source family per L3-lock comment), VerifierOutcome (Verified / Refuted / Inapplicable / RunpodPending), VerifyRequest, ALL_VERIFIER_KINDS const.
  - orchestrator.rs: run() entry composing C-50 contract_coding::StructuralContract emission + verifier dispatch. Dafny route reuses C-52 invoke_dafny_cli with explicit DafnyVerifyResult -> VerifierOutcome mapping (vacuous-guard failure -> Inapplicable). Z3 / Kani / Verus routes return Inapplicable {reason: "<kind>_stub"} in lean ship.

CLI subverb: elai formal-verify run --target PATH --verifier KIND + elai formal-verify status. Wired via crates/cli/src/formal_verify_cmd.rs + args.rs FormalVerifyAction enum + main.rs dispatch arm. WIRE-DARK avoidance per E40_WIRE_DARK_AVOIDANCE_VIA_CLI satisfied — every new function (formal_verify::run, all 4 VerifierKind variants, all 4 VerifierOutcome variants) has a production call site through this subverb.

Contract test: crates/contracts/tests/e53_formal_verification_contract.rs (~10.5 KB) covers E1..E7: FormalVerifierCompleted round-trip, VerifierKind 5-variant exhaustiveness, CSQ classifier risky/unknown behavior, Dafny RunpodPending path, Z3/Kani/Verus stub paths, C-50 contract_emitted composition, GEMMA4_BLOCKED inheritance.

KEY DECISIONS

Lean-scope: 13 plan-proposed SSB variants consolidated to 1 (FormalVerifierCompleted carrying verifier_kind_hash + outcome_hash + csq_class_hash discriminators). Rationale: C-52 precedent + exploration GAP-4 + SCHEMA bump magnitude reduction (140 vs 152). The variant payload carries the discriminators that the 13-variant proposal would have spread across the enum.

VerifierKind L3 unlock: the verification_result.rs:33-35 lock comment itself documents the escape clause "New kinds land via plan-level schema bumps". E-53 IS such a plan-level bump, so adding Z3SmtCheck is sanctioned. Cite-trail: lock comment + this decisions doc + the inline E-53 SCHEMA 139 -> 140 reference at the new variant.

CSQ as Rust module (DRIFT-2 closure): plan claimed CSQ classifier existed; exploration confirmed zero hits for CounterfactualClass enum and BlockSource::CsqCritical was the only related surface. New csq.rs module ships the classifier; CLAUDE.md prose rule remains authoritative for LLM behavior. The two surfaces are complementary, not redundant: CLAUDE.md drives the mcp_think reasoning loop; csq.rs is the runtime check the orchestrator consults before dispatch.

GEMMA-4 BLOCKED per C-52 substrate rule: the formal_verify pipeline NEVER routes to Gemma-4 as a model backend. The block constant GEMMA4_BLOCKED is inherited from dafny_verify::types::GEMMA4_BLOCKED (paper's top performer at 90.91% verify@5 RQ3a per arXiv:2604.22601 but blocked per memory feedback_no_gemma4). Substitutes from the paper's evaluation set: Codestral-22B-v0.1 (63.64% RQ2 verify@5), GPT-OSS-120B (63.64% RQ2 + 81.82% RQ3b), Qwen 3 Coder 30B (45.45% RQ2). Current ELAI sidecar default (Qwen 2.5 Coder 7B) is NOT in the paper's evaluation set; expected verify@5 below 45% on the signature tier.

NO new dependencies: z3 / kani-verifier / verus / dafny / lean-copilot all RUNPOD-PENDING. Anti-slopsquatting per CLAUDE.md: a Cargo.toml line is the only mechanism to introduce a dep; this ship adds none. Runtime detection at the boundary (which-style probe + tokio::process::Command shellout) returns VerifierOutcome::RunpodPending when the binary is absent. The dep additions land in a follow-up patch after user OK per CLAUDE.md "No new dependencies without confirmation" hard rule.

NO dashboard panel: deferred to E-29 territory (per session-3 boundary in handoff). The plan's Tasks 10 FormalVerificationPanel.svelte + CriticalPathClassificationPanel.svelte are not shipped in this lean E-53 ship — when added they go to crates/dashboard-v2/src/lib/components/ per E-52 confirmed path. The CLI surface (elai formal-verify status) provides the operator-facing summary in the meantime.

Cite-discipline: 3 GROUNDED inline cites (arXiv:2604.22601 Dafny NL2VC re-used from C-52, arXiv:2501.03124 PRMBench, AlphaProof Nature 2025). 11 INGEST-required papers DEFERRED to workflow/sessions/s-25120/completion-drive/mistakes.md cite-incident #11 (E-53-INGEST-pending): arXiv:2604.05292 Z3 COBALT, arXiv:2603.21149 Emergent Formal Verification, arXiv:2409.13082 AutoVerus, arXiv:2412.06176 AlphaVerus, arXiv:2509.22908 Vericoding, arXiv:2507.16331 Re:Form, arXiv:2501.06283 Dafny POPL'25, arXiv:2507.23726 Seed-Prover, arXiv:2504.21801 DeepSeek-Prover-V2, arXiv:2404.12534 Lean Copilot, arXiv:2601.05385 / 2604.05292 DafnyPro variance. Cited by name only in docstrings; no inline numbered claims like "90% verify@5" or "$6M seed".

arXiv ID variance: the plan body cites DafnyPro as both 2604.05292 (line 10) AND 2601.05385 (line 281). Neither is paper-ingestor-verified in this session. DEFERRED — the formal_verify code refers to DafnyPro by name only ("DafnyPro paper") with no inline ID, awaiting paper-ingestor verification.

NO main.rs RunArgs.empire/export_ssb touched: session 3 territory per session-2-handoff line 173. The pre-existing E0027/E0063 on main.rs persists; the formal-verify dispatch arm is added without modifying RunArgs.

NO held_out.rs / D-plan substrate touched: session 3 in-flight per handoff line 174-176.

EXPECTED TEST OUTCOME

cargo check -p elai-orchestrator: PASS (new modules csq + formal_verify compile; VerifierKind extension preserves all existing exhaustive matches since no exhaustive match exists on the enum — only the as_str method which I updated).

cargo check -p elai-cli: PASS modulo pre-existing main.rs E0027/E0063 (session 3 territory; documented in session-2-handoff).

cargo check -p elai_cc_contracts --test e53_formal_verification_contract: PASS — E1..E7 covered.

action_type_lock.golden: regenerated with 404 entries + new "formal_verifier_completed" ordinal 0139 (between fitm_calibration_applied 0138 and framework_profile_selected 0140).

ssb.rs all_test_payloads count: 337 -> 338.

UNKNOWNS

Live SMT / Dafny / Kani runtime cost: untested locally (toolchains absent). The pipeline's per-toolchain default time budget (Z3 5s, Dafny inherits C-52 MAX_HEAL_ITER * per-iter, Kani 30s, Verus 5min CI) is design-time-only.

CSQ classifier false-positive rate on real codebases: UNKNOWN. The 18-pattern heuristic list is curated by hand from auth / money / crypto / parser / untrusted-input territory; real-world miss + false-positive rates require corpus measurement in a follow-up.

RUNPOD pod size for full verification corpus: not estimated. The 4 toolchain binaries (Z3 ~50 MB, Dafny ~200 MB, Kani ~150 MB, Verus + Lean ~500 MB combined) plus model weights for the spec-author / impl-author roles (Codestral-22B-v0.1 NF4 ~15 GB, GPT-OSS-120B in 4-bit ~60 GB) need profiling against the remote 3090 box.

WIRE-DARK posture validation: the new FormalVerifierCompleted SSB row has its production call anchor in elai formal-verify run but is not yet emitted from a non-test caller (the CLI subverb prints the report but does not write SSB). A patch-N follow-up should add the SSB writer wiring once the host has an Ssb instance handle during the CLI dispatch.

DRIFT NOTES (plan-text vs code; no code changes from these — documented for future-session traceability)

DRIFT-1 path: plan cites d3_verification.rs; real file is verification_result.rs. Code references the real file throughout.

DRIFT-3 auditor.rs: plan cites auditor.rs (single file); real surface is auditor_mode.rs + deterministic_auditor.rs + deterministic_audit_gate.rs + locked_keys_auditor.rs + per_level_audit_dispatch.rs. No Kani harness dispatch wiring touches the auditor surface in this lean ship — deferred.

DRIFT-7 "impl VerifierKind" on struct: plan WIRING_CONTRACT misframes VerifierKind as a trait. Real wiring: VerifierKind is an enum + the formal_verify::VerifierKind toolchain enum is a parallel surface + the orchestrator dispatches via match-on-variant. No "impl VerifierKind" syntax appears in the code.

DRIFT-9 SpecValidator agent (model=sonnet): NOT shipped in this lean E-53 ship. The agent definition is a workflow-tooling-side artifact (.claude/agents/spec-validator.md) that the planning agents would dispatch; deferred per autonomous-mode lean-scope.
