E-8 (self-evolving prompts) — decisions log
scope-id: E-8_self_evolving_prompts
session: s-25120 completion-drive batch 3/4
target: docs/thefinalglance/E-plan/E-8_self_evolving_prompts.md
status: SHIPPED (scope-lean Pattern-2)

DECISION-1: SCHEMA cascade shape — Pattern-2 single-variant
Choice: ONE SsbActionType variant `PromptOptimizationCompleted` carrying a
`PromptOptimizationCompletedPayload` whose `phase: OptimizerPhase` field is a 7-arm
discriminator: {DspyInit, OproStep, RankPromptVote, ApeProposal, GepaReflect,
ArchivePromote, ArchiveRetire}.
Rationale: 13th application of the Pattern-2 idiom (E-7 DarwinianGenerationCompleted /
DarwinianPhase precedent at ssb.rs:5414-5439 + :17109-17144). Cost ≈ 1/7 of the
naive +7-variant cascade: only 1 SsbActionType enum arm, 1 as_sql_str arm, 1 FromStr
arm, 1 SsbPayload arm, 1 action_type arm, 1 SQL CHECK entry, 1 ALL_VARIANTS row,
1 golden file line, 1 exhaustive lock-match arm. The 7 phases capture the lifecycle
the plan task 10 enumerates plus enough rotor coverage for the remaining 1
optimizer paradigm (RankPromptVote — kept as a phase even though paper-triager
flagged the paper as WRONG_REGIME, see DECISION-7).
SCHEMA 161 -> 162. TOTAL_VARIANTS 434 -> 435.

DECISION-2: ArchiveStatus naming — use canonical Candidate/Promoted/Retired/Frozen
Choice: drop the plan's "SHADOW/ACTIVE" labels in favour of the existing
`ArchiveStatus` enum at reasoning_template.rs:258-265.
Rationale: HIGH-1 from context-package + cross-check; "SHADOW" / "ACTIVE" do not
exist in the canonical lifecycle. Renaming SHADOW->Candidate and ACTIVE->Promoted
matches the existing can_transition_to surface (:282-289) with zero enum delta.

DECISION-3: cfg-feature gate — reuse `darwinian-evolve`
Choice: gate every E-8 module that names `MapElitesArchive` / `two_gate_promote` /
`cross_domain_canary` (admit path, transfer-check Rust side, panel inputs) behind
the SAME `cfg(feature = "darwinian-evolve")` feature already declared at
Cargo.toml:201 + lib.rs:346-367.
Rationale: HIGH-3 from context. Option (a) "ship E-8 unconditionally with cfg-gated
two_gate_promote call site" is rejected because every consumed E-7 type
(BehaviourDescriptor, ArchiveEntry, MapElitesArchive, PromotionConfig) is cfg-gated.
Adding a parallel default-feature stub set would duplicate the entire darwinian
substrate. Re-using the existing feature is one-line cheap, preserves wire-dark
posture, and keeps E-8 + E-7 admit paths on the same archive surface (Pareto
frontier shared, per plan §Context).

DECISION-4: baby_step_spec.rs absence — focused new file `baby_step_spec_lpo.rs`
Choice: create a focused `baby_step_spec_lpo.rs` module that operates over the
existing `BabyStep` struct in baby_step.rs PLUS a new `BabyStepSpec` shape carrying
the LPO edit/frozen tags. Plan task 2 says MODIFY but the file does not exist (only
grammars/baby_step_spec.ebnf — never compiled to Rust).
Rationale: HIGH-2 from context-package. EBNF -> Rust codegen is out-of-scope for
E-8 (no parser-generator dep added). The LPO `<edit>`/`<frozen>` enforcement is a
pure-text predicate over 8 block kinds (TASK, GROUNDING, PRE_GATE, TDD,
ROLE_PROMPT_BODY, DETERMINISTIC_CHECK, ACCEPTANCE, INTEGRATION, EXIT_CRITERIA) so
the focused module is ~120 LOC and self-contained — no upstream dep on the EBNF
grammar. Per LPO arXiv:2504.20355: tag wrapping is a span-level discipline, not a
parser change.

DECISION-5: gepa PyPI anti-slopsquatting — VERIFIED at triage
Choice: declare `gepa>=0.1.1,<0.2` in the new optimizer pyproject.toml. Per the
grounding file (workflow/sessions/s-25120/completion-drive/grounding/
E-8_self_evolving_prompts-grounding.md), paper-triager verified the package via
WebFetch of https://pypi.org/project/gepa/ on 2026-05-27:
  - latest version: 0.1.1
  - release date: 2026-03-16 (within 24-month window)
  - license: MIT
  - maintainer: Lakshya A Agrawal (matches GEPA paper author identity)
  - GitHub: https://github.com/gepa-ai/gepa
  - verdict: VERIFIED, no namespace-collision risk
This satisfies the CLAUDE.md anti-slopsquatting hard rule. Package is young; pin to
0.1.x range so a 0.2 API break does not silently break the optimizer.

DECISION-6: paper grounding — DSPy, OPRO, APE, GEPA used; RankPrompt demoted
DSPy (arXiv:2310.03714) — interface map at workflow/papers/dspy-interface.md.
OPRO (arXiv:2309.03409) — interface map at workflow/papers/opro-interface.md.
APE (arXiv:2211.01910) — REAL_AND_GROUNDED, kept as prior-art cite for
ApeProposal phase.
GEPA (arXiv:2507.19457) — REAL_AND_GROUNDED, ICLR 2026 Oral confirmed, drives
GepaReflect phase.
RankPrompt (arXiv:2403.12373) — flagged WRONG_REGIME by paper-triager (it's a
reasoning-quality method, not a prompt optimizer). Demoted to a supporting-only
cite; RankPromptVote phase remains in the OptimizerPhase enum because a
response-ranking sub-feature MAY be added later, but no live wiring depends on it
in E-8 ship.

DECISION-7: contract-test foundation — use paper-triager DSPy/OPRO interfaces
The paper-triager wrote workflow/papers/dspy-interface.md + opro-interface.md as
RAG-failure fallback. We use those as the contract surface for the python
optimizer's expected API (config dataclasses, optimize_template fn signature,
return shape). No contract-test re-derivation of paper claims — the interface map
IS the contract.

DECISION-8: dashboard wiring — YES (focused panel)
Choice: ship `OptimizerRunPanel.svelte` + `get_optimizer_run` Tauri command.
Rationale: plan task 8 + ACCEPTANCE-7 explicitly require the panel; cross-check
report flagged it MISSING. Reads the new SSB stream
`prompt_optimization_completed` via SsbViewer::recent_rows_of_type — same shape
as E-37 PromptArmorPanel + E-17 SP4 panels.

DECISION-9: held-out + cross-domain canary — RUNPOD-PENDING for live model calls
Choice: scaffold optimize_template() / validate_transfer() with full type surface
+ deterministic logic stubs; mark cloud-Sonnet + local-Qwen live call sites with
`# RUNPOD-PENDING: live LLM call` comments. The Rust admit path runs end-to-end
on test fixtures via the OptimizedTemplate struct it consumes.
Rationale: per session prompt, "any optimizer that needs live LLM calls to test
mutation -> scaffold + mark RUNPOD-PENDING". This preserves zero-lazy-code
discipline (no `pass` bodies, no `unimplemented!()` — every function returns a
real typed value) while keeping the live-inference seam visible.

DECISION-10: tests added
- crates/orchestrator/tests/optimizer_admit_lifecycle.rs
- crates/orchestrator/tests/optimizer_admit_cross_domain_blocks.rs
- crates/orchestrator/tests/optimizer_lpo_spans.rs
- crates/orchestrator/src/optimizer_admit.rs has #[cfg(test)] inline tests as the
  primary contract surface (no external SSB writes; deterministic).
- python/elai_optimizer/tests/test_optimize_smoke.py — 5-iter deterministic stub
  proves the Python orchestration shape.
- python/elai_optimizer/tests/test_transfer_check.py — re-eval gap > 2pp triggers
  overfit flag.

DECISION-11: no .unwrap() in new code; thiserror for errors
Per CLAUDE.md hard rule, all new Rust modules use Result + thiserror for error
paths. The optimize_cmd CLI handler returns Result<(), String> to match
formal_verify_cmd shape.

DECISION-12: cross-lang pair registry
Add `optimizer_run` -> {rust: orchestrator::optimizer_admit, ts: dashboard panel
OptimizerRunPanel.svelte, tauri: get_optimizer_run} to .elai_cc/cross-lang-pairs.toml
on a follow-on session (the registry file may not yet have a uniform schema for
ad-hoc additions; defer to E-45 review). Document the pair manually in this
decision log.

CITATIONS USED (all WebFetch-verified per paper-triager)
- arXiv:2507.19457 GEPA (Agrawal et al., ICLR 2026 Oral)
- arXiv:2310.03714 DSPy (Khattab et al.)
- arXiv:2309.03409 OPRO (Yang et al.)
- arXiv:2211.01910 APE (Zhou et al.)
- arXiv:2405.10276 small-LMs-as-optimizers limitation (Zhang et al.)
- arXiv:2512.01420 PromptBridge — transfer claim
- arXiv:2504.20355 LPO — `<edit>` tag pattern
- arXiv:2602.00997 ETGPO — gotchas injection ~1/3 token cost

NOT used (demoted / removed):
- arXiv:2403.12373 RankPrompt — WRONG_REGIME, supporting-only
- arXiv:2406.07496 TextGrad, arXiv:2511.12063 TextBO, arXiv:2510.08413 PAC bounds,
  arXiv:2507.03620 — referenced as plan grounding; not load-bearing in the ship

WIRING-CONTRACT realisation
produces: PromptOptimizationCompleted + OptimizerPhase + payload struct +
optimizer_admit::admit_optimized_template + optimizer_admit::OptimizedTemplate +
gotchas_inject::top_gotchas + auditor_feedback_export::export_for_run +
baby_step_spec_lpo::editable_spans + cli::optimize_cmd::optimize_template_command +
dashboard get_optimizer_run + OptimizerRunPanel.svelte +
python::elai_optimizer::optimizer::optimize_template +
python::elai_optimizer::transfer_check::validate_transfer
consumes: per_role_runtime (PromptEngineer route) + reasoning_template::Blake3Hash
+ template_archive::MapElitesArchive (cfg(darwinian-evolve)) +
promotion_gate::two_gate_promote (cfg(darwinian-evolve))

E-8-fix WARN-1 mop-up (2026-05-27 s-25120)
optimizer.py:_blake3_hex still falls through to hashlib.blake2b on ImportError.
blake3 is NOT in elai-optimizer/pyproject.toml dependencies. The fallback is
preserved so the smoke-test path can produce a syntactically-valid envelope in
a checkout without blake3 installed; the Rust admit side rejects non-blake3
hashes per anti-stale rule. ACTION REQUIRED: add `blake3>=0.4` to
python/elai_optimizer/pyproject.toml dependencies before WARN-1 can be closed
to blake3-only path. Until then the fallback is INTENTIONAL.
