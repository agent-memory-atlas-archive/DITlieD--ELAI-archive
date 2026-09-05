# E-53: Narrow formal-method gates — Z3 SMT in D-3 + Kani harnesses in AUDITING + AutoVerus on FSM/IPC invariants (CI-gating COMMITTING) + Dafny IR for safety-critical Python; Lean explicitly HARD-AVOID as audit gate
Objective: Wire formal verification ONLY where it earns its keep. Z3 SMT becomes the 4th VerifierKind in D-3's VerificationResult aggregator, firing only when CSQ classifies the path Critical (auth/money/crypto/parser/untrusted-input) — counts as 1 of the ≥2 verifier consensus votes. Kani harness checks run in AUDITING for Rust hot paths marked `#[cfg_attr(kani, kani::proof)]` — deterministic latency, bounded model-checking on memory/arithmetic invariants. AutoVerus-style Verus annotations encode ELAI's FSM transitions + IPC v2 protocol + KV-cache size invariants in a separate `verus-check` CI job that gates COMMITTING (not every audit — wall-clock budget). Dafny becomes a verification-aware IR for Python safety-critical paths via a MediaValidator-companion SpecValidator that confirms Dafny semantics match input. Lean 4 / theorem-prover loops are HARD-AVOIDED as routine gates per report (AlphaProof ~500 TPU-days/problem; Seed-Prover 1.5 16.5h GPU for 5 IMO problems — wrong unit of latency); Lean Copilot `select_premises` is allowed only as an optional READ-ONLY hint when verification BLOCKs fire.
Estimated effort: 3 sessions, ~12 h
Pre-requisites: D-3 PLANNED (VerificationResult aggregator + multi-verifier consensus rule — provides the slot Z3 plugs into; per D-SUMMARY_INDEX line 34: "Adds VerificationResult { deterministic, ppm, intrinsic, cross_context, consensus_rule }; promotion blocked unless ≥2 verifiers concur; PRMBench Deception Resistance defense"). C-52 PLANNED (Dafny Formal Verification Spike — RESEARCH-GRADE; per C-SUMMARY_INDEX line 719; cited paper 2604.22601 Dafny NL2VC-60). CSQ infrastructure SHIPPED (CLAUDE.md "CSQ: before finalizing trading/ML logic, call mcp_think(step_type='counterfactual')... Min 1 per financial-data function") — provides the Critical-path classification this E-N gates on.
Parallel-safe with: E-51 (edge inference — disjoint), E-52 (LSP integration — complementary precision oracle; both feed auditor evidence), E-54 (eval corpus — disjoint), E-55 (agentic RL — disjoint). EXTENDS D-3 directly (adds VerifierKind::Z3SmtCheck variant). EXTENDS C-52 (Dafny spike) by promoting Dafny from research-grade to wired-but-narrow Python safety-critical path.
Source report: workflow/improvement-sweep-2026-05-19/reports/53-formal-verification-codegen.md

## Context

The May 2026 formal-verification survey establishes that AI-as-prover SOTA is uneven: MiniF2F-test hit 99.6% with Seed-Prover (arXiv:2507.23726), DeepSeek-Prover-V2 88.9% (arXiv:2504.21801); PutnamBench reached only 7% on GPT-5 and 49/658 on DeepSeek-Prover-V2 (trishullab.github.io/PutnamBench); AutoVerus reaches >90% on 150-task Verus benchmark with half completing in <30s or ≤3 LLM calls (arXiv:2409.13082); Vericoding's 12,504-spec benchmark (arXiv:2509.22908, Tegmark et al., MIT+others) measures 82% Dafny / 44% Verus / 27% Lean — Dafny is the cheap entry point. The "Broken by Default" study (arXiv:2604.05292) — 3,500 LLM-generated artifacts via Z3 COBALT pipeline — establishes Z3-on-LLM-code as the right primitive: SMT catches CWE-131/190/916 classes that static analyzers miss systematically. Theorem (YC S25, $6M seed, venturebeat.com) reports catching a bug that passed Anthropic's own testing — existence proof of the formal-gate-catches-what-tests-don't claim. Emergent Formal Verification (arXiv:2603.21149) reports six independent autonomous AI ecosystems converged on Z3 — convergence evidence for the primitive. PRMBench (arXiv:2501.03124) "Deception Resistance" dimension shows state-of-the-art PRMs miss misleading-but-plausible steps; Z3/Kani/Verus compute, they don't fall for plausibility — exactly the layer D-3 needs. Shiplight 2025: AI-coded PRs 1.7x more bugs; modern review tools catch 42-48% vs <20% for traditional static analyzers — formal checks live at the upper band. The DAFNYCOMP cliff (DafnyPro, arXiv:2601.05385) is the central anti-pattern: if the LLM writes BOTH the spec AND the proof, the gate is hallucinated confidence (95.67% syntax → 3.69% verified). Spec must come from a different agent (CSQ counterfactual) or human. Lean theorem-prover loops are explicitly out of audit-latency-budget per AlphaProof (~500 TPU-days/problem, www2.eecs.berkeley.edu/Pubs/TechRpts/2025/EECS-2025-121) and Seed-Prover 1.5 (16.5h for 5 IMO 2025 problems per seed.bytedance.com).

## Existing state (B/C/D coverage)

- D-3 PLANNED (VerificationResult aggregator + multi-verifier consensus rule, per D-SUMMARY_INDEX line 34-35): provides the consensus-aggregator surface this E-N plugs into. EXTENSION: E-53 adds `VerifierKind::Z3SmtCheck` as a new VerifierKind variant; counts as 1 of the ≥2 verifier consensus votes; activated only when CSQ classifies the path Critical. Existing `{ deterministic, ppm, intrinsic, cross_context, consensus_rule }` surface remains unchanged additively.
- C-52 PLANNED (Dafny Formal Verification Spike, RESEARCH-GRADE; per C-SUMMARY_INDEX line 719-729; cited 2604.22601 Dafny NL2VC-60): C-52 frames Dafny as research; E-53 promotes Dafny to a narrow production gate for safety-critical Python paths only (auth/money/parser). PROMOTION: ships the Dafny SpecValidator wiring; C-52 spike outputs remain valid reference material.
- CSQ infrastructure (SHIPPED per CLAUDE.md "Critical / Counterfactual" hard rule + mcp_think counterfactual step_type): provides the Critical-path classifier that E-53 uses as a gate trigger. EXTENSION: Z3/Kani/Verus checks only fire when CSQ returns Critical — saves wall-clock budget on the vast majority of edits.
- MediaValidator agent (verify presence via Glob `.claude/agents/*media*`): if exists, E-53 ADDs a SpecValidator role alongside; if not, ships SpecValidator as a new agent under `.claude/agents/spec-validator.md` with model=sonnet per CLAUDE.md hard rule.
- D-13 Adversary cloud-call surface (existing per D-SUMMARY_INDEX line 100; D-10 Mode B routing pre-req): COORDINATES with E-53's "different-agent spec" rule — the spec for Z3/Verus encoding MUST come from CSQ counterfactual OR from D-13 Adversary, NEVER from the LLM that wrote the code (per DAFNYCOMP anti-pattern).
- PRMBench Deception Resistance (D-3 closes via consensus rule): E-53's Z3 gate is exactly the layer that doesn't fall for plausibility per PRMBench analysis — the consensus rule gains a strict guard.
- Kani / Verus / Dafny / Z3 / Lean: NONE currently in `.elai_cc/`. All NEW dependencies; each requires anti-slopsquatting verification per CLAUDE.md before adoption.

## Reads (mandatory before coding)

- `F:\.ELAI_workflow\workflow\improvement-sweep-2026-05-19\reports\53-formal-verification-codegen.md` (full)
- `F:\.ELAI_workflow\docs\thefinalglance\D-plan\D-SUMMARY_INDEX.md` lines 30-50 (D-3 surface — VerificationResult + consensus_rule)
- `F:\.ELAI_workflow\docs\thefinalglance\C-plan\C-SUMMARY_INDEX.md` lines 715-730 (C-52 Dafny spike — research baseline)
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\evidence.rs` (existing trait — verify via Read; per elai.map line 326787 DEF_TRAIT EvidenceGate at evidence.rs:160 — E-53 plugs into this)
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\system_probe.rs` (existing SystemProber trait per elai.map line 326788 — E-53's verifier processes follow same Send+Sync pattern)
- `F:\.ELAI_workflow\workflow\improvement-sweep-2026-05-19\E-plan\E-52_lsp_integration.md` (complementary precision oracle; coordinate on auditor evidence)
- `F:\.ELAI_workflow\CLAUDE.md` "CSQ: counterfactual" rule (full block) — grounds Critical-path classifier trigger
- https://arxiv.org/pdf/2604.05292 (WebFetch — Broken by Default Z3-on-LLM study, 3500 artifacts / 7 LLMs / 500 prompts)
- https://arxiv.org/abs/2603.21149 (WebFetch — Emergent Formal Verification convergent evidence)
- https://arxiv.org/pdf/2409.13082 (WebFetch — AutoVerus >90% on 150 Verus tasks)
- https://arxiv.org/html/2412.06176v1 (WebFetch — AlphaVerus Dafny→Verus translation + Treefinement tree search)
- https://arxiv.org/abs/2509.22908 (WebFetch — Vericoding benchmark: 82% Dafny / 44% Verus / 27% Lean)
- https://arxiv.org/abs/2507.16331 (WebFetch — Re:Form RL-on-Dafny; even 0.5B verifies Dafny after SFT)
- https://arxiv.org/pdf/2501.06283 (WebFetch — Dafny as Verification-Aware IR, POPL 2025)
- https://arxiv.org/abs/2501.03124 (WebFetch — PRMBench Deception Resistance dimension)
- https://model-checking.github.io/kani/ (WebFetch — Kani docs + cargo integration)
- https://aws.amazon.com/blogs/opensource/how-open-source-projects-are-using-kani-to-write-better-software-in-rust/ (WebFetch — Kani Autoharness 2025)
- https://www.microsoft.com/en-us/research/publication/verus-a-practical-foundation-for-systems-verification/ (WebFetch — Verus SMT-based deductive verifier for Rust)
- https://venturebeat.com/security/theorem-wants-to-stop-ai-written-bugs-before-they-ship-and-just-raised-usd6m (WebFetch — Theorem YC S25; caught bug past Anthropic testing)
- https://martin.kleppmann.com/2025/12/08/ai-formal-verification.html (WebFetch — Kleppmann 2025 framing)
- https://www2.eecs.berkeley.edu/Pubs/TechRpts/2025/EECS-2025-121.pdf (WebFetch — Berkeley TR on AlphaProof TPU-day cost, grounds Lean HARD AVOID)
- https://seed.bytedance.com/en/blog/seed-prover-1-5-advanced-mathematical-reasoning-through-a-novel-agentic-architecture (WebFetch — Seed-Prover 1.5 16.5h for 5 IMO 2025 problems, grounds Lean HARD AVOID)
- https://arxiv.org/abs/2404.12534 (WebFetch — Lean Copilot select_premises; allowed only as read-only hint)

## Tasks

1. Z3 SMT gate as 4th VerifierKind in D-3 aggregator. CREATE `.elai_cc/crates/orchestrator/src/z3_smt_check.rs` (~260 LOC) implementing a `Z3SmtCheck` verifier. The verifier wraps Z3 SMT via the `z3` crate (verify via `cargo search z3` + check Anti-slopsquatting: crates.io exists, last updated <12mo, valid License). Encoder takes a Rust function signature + CSQ counterfactual postcondition string and emits a Z3 SMT-LIB2 query; runs Z3 with a 5s wall-clock timeout. Verdict: `Sat → BLOCK with counterexample`; `Unsat → consensus vote`; `Unknown / Timeout → abstain (does NOT count toward consensus)`. Activation gate: CSQ classifier returns `Critical` (auth/money/crypto/parser/untrusted-input) — verify via Grep `CSQ::Critical|CounterfactualClass::Critical` for the existing variant; if absent, ADD `CounterfactualClass::{Critical, NonCritical}` to the CSQ module. MODIFY `crates/orchestrator/src/d3_verification.rs` (verify path via Grep `VerificationResult|VerifierKind` — likely `crates/orchestrator/src/` per D-3 surface) to add `VerifierKind::Z3SmtCheck` variant and `Vec<VerifierKind>` aggregator. The spec MUST come from CSQ counterfactual OR D-13 Adversary — NEVER from the LLM that wrote the code (anti-pattern guard per DAFNYCOMP cliff).

2. Kani harness check in AUDITING. CREATE `.elai_cc/crates/orchestrator/src/kani_harness.rs` (~220 LOC) implementing a wrapper around `cargo kani --harness <fn>` invocation. Triggered in AUDITING substate (not D-3 consensus — AUDITING gate, deterministic latency). LLM proposes property predicates from the spec (CSQ counterfactual OR D-13 Adversary OR human-authored Kani harness file under `crates/<crate>/proofs/`); the wrapper invokes Kani with a 30s wall-clock timeout per harness; outputs `KaniOutcome::{Verified, Falsified(Counterexample), Timeout, NotApplicable}`. Auditor emits `kani_harness_verified` / `kani_harness_falsified` / `kani_harness_timeout` SSB rows. Kani install verified via `cargo install --version <pinned> kani-verifier` AND anti-slopsquatting (the `kani-verifier` crate IS the canonical AWS-maintained crate per github.com/model-checking/kani — pin version SHA verified at install). Falsified verdict is BLOCK with the counterexample auto-routed into fixer prompt. Timeout is soft warning (logged), NOT BLOCK.

3. AutoVerus-style Verus annotation for FSM/IPC/KV-cache invariants — CI-gating, NOT per-audit. CREATE `.elai_cc/crates/orchestrator/src/verus_invariants.rs` (~280 LOC) — Rust module with Verus-annotated invariants for: (a) FSM state transitions (no transition skips an intermediate state; per CLAUDE.md "FSM gate check"); (b) IPC v2 frame size invariants (payload length ≤ MAX_FRAME, header length-prefix matches body length); (c) KV-cache size invariants (sum(per-slot KV) ≤ KV_CACHE_BUDGET_BYTES; per memory:project_qwen3b_baseline.md "25K tokens ~921 MB"). CREATE `.elai_cc/scripts/verus-check.sh` (~30 LOC shell) that invokes Verus on the annotated module with a 5min wall-clock cap. CREATE `.elai_cc/.github/workflows/verus-check.yml` (~40 LOC GH Actions job) running on every PR that gates COMMITTING — fails the CI job if Verus discovers a violation. Verus install verified per Anti-slopsquatting (github.com/verus-lang/verus is the canonical repo; pin to a recent release tag). LATENCY: CI job, NOT in-loop AUDITING — wall-clock budget is the right tier.

4. Dafny as verification-aware IR for safety-critical Python paths. CREATE `.elai_cc/python/elai_inference/dafny_spec_validator.py` (~240 LOC). When CSQ classifies a Python path Critical (auth/money/parser), route via Dafny IR: (i) CSQ counterfactual OR D-13 Adversary writes the spec in Dafny; (ii) the implementation Python is generated alongside; (iii) Dafny verifier runs on the IR with a 30s timeout; (iv) `SpecValidator` confirms the Dafny semantics match the input (semantic equivalence check, not just compile). Dafny install verified per Anti-slopsquatting (github.com/dafny-lang/dafny is canonical). Outcome variants: `DafnyOutcome::{Verified, NotVerified, Timeout, SpecMismatch}`. ONLY runs on Critical paths — not on every Python write. Emit `dafny_spec_validated` / `dafny_spec_failed` SSB rows.

5. SpecValidator agent (Dafny + Verus + Z3 — chooses one). CREATE `.claude/agents/spec-validator.md` (~80 LOC agent definition, model=sonnet per CLAUDE.md hard rule). Agent inputs: the function under audit + the Critical path classification from CSQ. Agent outputs: which formal-method tier to invoke (Z3 for Rust+SMT-shaped, Kani for Rust+memory-shaped, Verus for Rust+invariant-shaped, Dafny for Python+spec-shaped); the formal-spec to validate against (counterfactual postcondition from CSQ); and the verdict from running the chosen tool. The agent NEVER writes both the spec AND the proof — it consumes the spec from CSQ counterfactual / D-13 Adversary as input. Per CLAUDE.md "MEMORIZING delegation" pattern: agent runs in its own context, returns only verdict + grounding to main LLM.

6. Lean Copilot OPTIONAL premise-hint surface — READ-ONLY, NOT a gate. CREATE `.elai_cc/crates/orchestrator/src/lean_premise_hint.rs` (~120 LOC). When ANY of Z3/Kani/Verus/Dafny verifier returns `Not Verified` / `Falsified` / `Sat` (BLOCK), the auditor's BLOCK message may OPTIONALLY include a Lean Copilot `select_premises` hint generated by a separate background call. The hint is READ-ONLY — purely informational for the fixer. NEVER counts toward consensus, NEVER auto-applies. Wall-clock budget: 60s background, asynchronous to the BLOCK return. Lean Copilot via the lean-copilot package (github.com/lean-dojo/LeanCopilot) — anti-slopsquatting verified. ACCEPTANCE: lean_premise_hint MUST NOT block the BLOCK return — the auditor returns BLOCK first, hint arrives later as an SSB row.

7. PRMBench Deception Resistance canary test for the auditor itself. CREATE `.elai_cc/crates/contracts/tests/prmbench_deception_resistance_canary.rs` (~140 LOC). Synthetic dataset: 20 deceptive-but-plausible diffs (LLM-generated; verified by human curator that each is subtly wrong but looks correct). Test: each diff passes through the auditor with Z3+Kani gates enabled; auditor's anti-rubber-stamp rule MUST produce ≥1 FINDING on every one. Runs as a NIGHTLY canary (per CLAUDE.md `.elai_cc/.github/workflows/`), NOT per-audit (wall-clock budget). Cited grounding: PRMBench arXiv:2501.03124. Records pass-rate to `workflow/reasoning/E-53-prmbench-results.md` over time.

8. Anti-pattern guards: stub_ban + contract tests. CREATE `.elai_cc/crates/contracts/tests/no_llm_writes_both_spec_and_proof.rs` (~80 LOC). Verifies the DAFNYCOMP cliff guard: any Z3/Kani/Verus/Dafny verification call carries a `spec_origin: SpecOrigin::{CsqCounterfactual, Adversary, Human}` field; verifier REFUSES to consume a spec with `SpecOrigin::SameLlmAsImpl` — emits `spec_origin_violation_blocked` SSB row. CREATE `.elai_cc/crates/contracts/tests/no_lean_as_default_gate.rs` (~60 LOC). Verifies Lean is NEVER consulted as a default gate — only as the optional premise-hint surface (Task 6), invoked only after another verifier returns BLOCK, never primary, never on consensus path. CREATE `.elai_cc/crates/contracts/tests/critical_path_only_invokes_formal_verifier.rs` (~70 LOC). Verifies CSQ `Critical` is the only classification that triggers Z3/Kani/Verus/Dafny — non-Critical paths skip all formal-method overhead.

9. Bounded retry + escalation. The LDSW anti-pattern (DOI 10.1145/3788731.3788734 cited in report) requires bounded retries — LLM iterating on solver-error feedback can thrash. CREATE `.elai_cc/crates/orchestrator/src/formal_verifier_retry.rs` (~140 LOC). Cap retries at 3 per verifier; on 3rd failure, escalate to FINDING (auditor surfaces a structured finding to the user) and DO NOT loop. Emit `formal_verifier_retry_exhausted` SSB row with the cumulative failed-hypothesis trail. Each retry MUST present a DIFFERENT spec or DIFFERENT predicate — repeating the same Z3 query verbatim is a no-op and gets rejected with `formal_verifier_no_progress` row.

10. Dashboard panels + Tauri commands. CREATE `crates/dashboard-v2/src/lib/components/FormalVerificationPanel.svelte` (~160 LOC): shows the last N verifier outcomes (Z3 / Kani / Verus / Dafny) with verdict + counterexample preview. CREATE `crates/dashboard-v2/src/lib/components/CriticalPathClassificationPanel.svelte` (~100 LOC): shows which functions were classified Critical by CSQ in the current session + which were skipped. CREATE Tauri commands `get_formal_verification_outcomes` + `get_critical_path_classifications` in `crates/dashboard-v2/src-tauri/src/commands.rs` (~80 LOC). Lean explicitly NOT visible in the panel — keep the surface formally Z3/Kani/Verus/Dafny only.

## WIRING_CONTRACT

produces_symbols:
  - orchestrator::z3_smt_check::Z3SmtCheck (impl VerifierKind)
  - orchestrator::z3_smt_check::Z3Outcome::{Unsat, Sat(Counterexample), Unknown, Timeout}
  - orchestrator::d3_verification::VerifierKind::Z3SmtCheck (variant extension)
  - orchestrator::kani_harness::KaniHarness + KaniOutcome::{Verified, Falsified, Timeout, NotApplicable}
  - orchestrator::verus_invariants (Rust module with Verus-annotated FSM/IPC/KV-cache invariants)
  - orchestrator::lean_premise_hint::LeanPremiseHint (READ-ONLY hint surface)
  - orchestrator::formal_verifier_retry::FormalVerifierRetry (3-attempt cap; LDSW guard)
  - SpecOrigin::{CsqCounterfactual, Adversary, Human, SameLlmAsImpl}
  - SSB action_types: z3_smt_unsat, z3_smt_sat, z3_smt_unknown, kani_harness_verified, kani_harness_falsified, kani_harness_timeout, verus_invariant_violated, dafny_spec_validated, dafny_spec_failed, lean_premise_hint_generated, spec_origin_violation_blocked, formal_verifier_retry_exhausted, formal_verifier_no_progress
  - python/elai_inference/dafny_spec_validator.py (Python module + 30s-cap Dafny invocation)
  - .claude/agents/spec-validator.md (NEW agent definition — model=sonnet, lives in agent registry per CLAUDE.md hard rule)
  - .elai_cc/scripts/verus-check.sh (CI-gating Verus harness)
  - .elai_cc/.github/workflows/verus-check.yml (CI job)
  - Tauri commands: get_formal_verification_outcomes, get_critical_path_classifications
consumes_symbols:
  - crates/orchestrator/src/d3_verification.rs::VerificationResult (D-3; aggregator surface — extension point)
  - crates/orchestrator/src/d3_verification.rs::consensus_rule (D-3; Z3 vote enters consensus)
  - crates/orchestrator/src/evidence.rs::EvidenceGate trait (per elai.map line 326787)
  - CSQ classifier (CounterfactualClass::Critical — verify variant exists; ADD if absent)
  - D-13 Adversary verdict surface (per D-SUMMARY_INDEX line 100; spec_origin source)
  - z3 crate (Rust; anti-slopsquatting verified via `cargo search z3` + crates.io recent publish)
  - kani-verifier crate (anti-slopsquatting verified; github.com/model-checking/kani)
  - verus crate (anti-slopsquatting verified; github.com/verus-lang/verus)
  - dafny binary (anti-slopsquatting verified; github.com/dafny-lang/dafny pinned release)
  - lean-copilot package (anti-slopsquatting verified; github.com/lean-dojo/LeanCopilot)
  - blake3 (for counterexample-hash dedup; NOT blake2/sha2)
reachable_from:
  - crates/orchestrator/src/d3_verification.rs::run_verifiers (Z3SmtCheck dispatched here on CSQ Critical)
  - crates/orchestrator/src/auditor.rs (Kani harness invoked here in AUDITING substate; verify via Grep)
  - .github/workflows/verus-check.yml (CI gate for COMMITTING)
  - python/elai_inference/server.py (dafny_spec_validator.py loaded as module)
stub_ban:
  - no `.unwrap()` outside `#[cfg(test)]` in any new Rust module (CLAUDE.md HARD RULE)
  - no `unimplemented!()` / `todo!()` in any new module
  - LLM MUST NOT write BOTH the spec AND the proof (DAFNYCOMP cliff anti-pattern) — SpecOrigin::SameLlmAsImpl REFUSED by verifier; contract test `no_llm_writes_both_spec_and_proof` enforces
  - NO LEAN as default gate, NO LEAN in consensus path — Lean Copilot allowed ONLY as read-only premise hint (Task 6); contract test `no_lean_as_default_gate` enforces; per report's HARD AVOID rule
  - NO theorem-prover-loops as routine audit — wall-clock budget violation (AlphaProof 500 TPU-days/problem; Seed-Prover 16.5h)
  - Z3 Unknown / Timeout MUST abstain — does NOT count toward consensus (cannot interpret silence as Pass)
  - Kani Timeout is SOFT warning — does NOT auto-BLOCK
  - Dafny SpecValidator MUST run on Critical paths ONLY — NOT on every Python write (wall-clock budget)
  - bounded retry: 3 attempts per verifier max; on 3rd fail, FINDING + DO NOT LOOP (LDSW anti-pattern guard)
  - same-spec retry rejected: each retry MUST present a DIFFERENT spec or predicate; formal_verifier_no_progress emitted otherwise
  - Verus CI job is wall-clock gated at 5min; if exceeds, CI fails (NOT silently passes)
  - Dafny invocation is wall-clock gated at 30s per spec
  - all new dependencies (z3, kani-verifier, verus, dafny, lean-copilot) MUST pass Anti-slopsquatting verification (crates.io / cargo search / WebFetch release page / pinned SHA) per CLAUDE.md before adoption
  - blake3 only for counterexample-hash dedup; no blake2/sha2 (CLAUDE.md anti-stale-training)
  - dashboard FormalVerificationPanel.svelte MUST NOT display Lean rows — Lean is read-only hint, not gate
frontend_pair: crates/dashboard-v2/src/lib/components/FormalVerificationPanel.svelte + CriticalPathClassificationPanel.svelte (Task 10)
observable_effect:
  trigger: synthetic Rust function in `crates/auth/` with `fn verify_password(stored: &[u8], provided: &[u8]) -> bool { stored == provided }` (CWE-203 timing attack) + CSQ counterfactual classifies it Critical
  expect_ssb: ≥1 `z3_smt_sat` row with counterexample showing variable-time comparison violates constant-time predicate, ≥1 `kani_harness_falsified` row if a Kani harness asserts constant-time, ≥1 BLOCK verdict with the counterexample auto-routed into the fixer prompt
  expect_artifact: D-3 VerificationResult shows Z3SmtCheck = Falsified + 1 vote against consensus; final consensus = BLOCK; fixer receives counterexample as structured input; Lean Copilot OPTIONAL premise hint may attach asynchronously as a separate SSB row but is NOT in the consensus vote

## DETERMINISTIC_CHECK

```bash
# 1. Z3 SMT check module exists with 4 outcome variants
rg "Z3Outcome::Unsat|Z3Outcome::Sat|Z3Outcome::Unknown|Z3Outcome::Timeout" .elai_cc/crates/orchestrator/src/z3_smt_check.rs
# Expected: ≥4 hits

# 2. VerifierKind::Z3SmtCheck variant added to D-3 aggregator
rg "VerifierKind::Z3SmtCheck" .elai_cc/crates/orchestrator/src/d3_verification.rs
# Expected: ≥1 hit

# 3. Kani harness module exists with 4 outcomes
rg "KaniOutcome::Verified|KaniOutcome::Falsified|KaniOutcome::Timeout|KaniOutcome::NotApplicable" .elai_cc/crates/orchestrator/src/kani_harness.rs
# Expected: ≥4 hits

# 4. Verus invariants module exists with FSM/IPC/KV-cache annotations
rg "FSM|IPC|KV_CACHE|kv_cache" .elai_cc/crates/orchestrator/src/verus_invariants.rs
# Expected: ≥3 hits

# 5. CI gate for Verus exists
ls .elai_cc/.github/workflows/verus-check.yml .elai_cc/scripts/verus-check.sh
# Expected: 2 files exist

# 6. Dafny Python SpecValidator
rg "DafnyOutcome::Verified|DafnyOutcome::NotVerified|DafnyOutcome::Timeout|DafnyOutcome::SpecMismatch" .elai_cc/python/elai_inference/dafny_spec_validator.py
# Expected: ≥4 hits

# 7. SpecValidator agent registered
ls .claude/agents/spec-validator.md
# Expected: file exists
rg "model.*sonnet" .claude/agents/spec-validator.md
# Expected: ≥1 hit

# 8. Lean as READ-ONLY hint surface only
rg "READ-ONLY|read.only|optional.hint" .elai_cc/crates/orchestrator/src/lean_premise_hint.rs
# Expected: ≥2 hits

# 9. Lean NOT in consensus path
rg "VerifierKind::Lean|consensus.*Lean|Lean.*consensus" .elai_cc/crates/orchestrator/src/
# Expected: zero hits

# 10. SpecOrigin variants registered
rg "SpecOrigin::CsqCounterfactual|SpecOrigin::Adversary|SpecOrigin::Human|SpecOrigin::SameLlmAsImpl" .elai_cc/crates/orchestrator/src/
# Expected: ≥4 hits

# 11. DAFNYCOMP cliff guard test exists
ls .elai_cc/crates/contracts/tests/no_llm_writes_both_spec_and_proof.rs .elai_cc/crates/contracts/tests/no_lean_as_default_gate.rs .elai_cc/crates/contracts/tests/critical_path_only_invokes_formal_verifier.rs
# Expected: 3 files exist

# 12. Bounded retry (3 cap)
rg "max_retries.*3|MAX_RETRIES.*3|retry_count >= 3|formal_verifier_retry_exhausted" .elai_cc/crates/orchestrator/src/formal_verifier_retry.rs
# Expected: ≥2 hits

# 13. blake3 only
rg "blake3" .elai_cc/crates/orchestrator/src/z3_smt_check.rs
# Expected: ≥1 hit
rg "blake2|sha2" .elai_cc/crates/orchestrator/src/z3_smt_check.rs .elai_cc/crates/orchestrator/src/kani_harness.rs .elai_cc/crates/orchestrator/src/verus_invariants.rs
# Expected: zero hits

# 14. No .unwrap() outside #[cfg(test)] in new modules
rg "\\.unwrap\\(\\)" .elai_cc/crates/orchestrator/src/z3_smt_check.rs .elai_cc/crates/orchestrator/src/kani_harness.rs .elai_cc/crates/orchestrator/src/verus_invariants.rs .elai_cc/crates/orchestrator/src/lean_premise_hint.rs .elai_cc/crates/orchestrator/src/formal_verifier_retry.rs | rg -v "cfg\\(test\\)|#\\[test\\]|tests/"
# Expected: zero hits

# 15. Dashboard panels exist; FormalVerificationPanel does NOT show Lean
ls .elai_cc/crates/dashboard-v2/src/lib/components/FormalVerificationPanel.svelte .elai_cc/crates/dashboard-v2/src/lib/components/CriticalPathClassificationPanel.svelte
# Expected: 2 files exist
rg "Lean|lean" .elai_cc/crates/dashboard-v2/src/lib/components/FormalVerificationPanel.svelte
# Expected: zero hits (Lean is hint-only, separate surface — NOT in this panel)

# 16. Smoke tests
cd .elai_cc && cargo test -p contracts no_llm_writes_both_spec_and_proof no_lean_as_default_gate critical_path_only_invokes_formal_verifier prmbench_deception_resistance_canary
# Expected: 4 PASS
cargo test -p orchestrator z3_smt_check_rejects_unknown_as_consensus_vote kani_harness_blocks_on_falsified verus_invariant_ci_gates_commit
# Expected: 3 PASS
```

## ACCEPTANCE

- [ ] `Z3SmtCheck` verifier added as 4th VerifierKind variant in D-3 aggregator; fires only on CSQ Critical paths; Unknown/Timeout abstain from consensus
- [ ] Kani harness wrapper runs in AUDITING substate on functions with `#[cfg_attr(kani, kani::proof)]`; 30s timeout per harness; falsified = BLOCK with counterexample
- [ ] Verus annotations for FSM transitions + IPC v2 frame size + KV-cache size invariants; CI job `verus-check.yml` gates COMMITTING (NOT per-audit)
- [ ] Dafny SpecValidator for Critical Python paths only; 30s timeout; SpecMismatch = BLOCK
- [ ] SpecValidator agent registered at `.claude/agents/spec-validator.md` (model=sonnet)
- [ ] Lean Copilot premise-hint surface is READ-ONLY, asynchronous, does NOT count toward consensus, NOT in dashboard panel
- [ ] SpecOrigin tracking: every verifier call records spec_origin; SameLlmAsImpl is REFUSED with spec_origin_violation_blocked SSB
- [ ] Bounded retry: 3 attempts per verifier, then FINDING + no loop; same-spec retry rejected
- [ ] PRMBench Deception Resistance canary runs nightly; verifier MUST produce ≥1 FINDING per deceptive diff
- [ ] All 5 new dependencies (z3, kani-verifier, verus, dafny, lean-copilot) pass Anti-slopsquatting verification
- [ ] Dashboard panels: FormalVerificationPanel + CriticalPathClassificationPanel + Tauri commands wired
- [ ] No Lean as default gate (contract test `no_lean_as_default_gate`)
- [ ] No LLM-writes-both-spec-and-proof (contract test `no_llm_writes_both_spec_and_proof`)
- [ ] CSQ Critical-only triggers formal verifier (contract test `critical_path_only_invokes_formal_verifier`)
- [ ] blake3 only for counterexample-hash dedup; no blake2/sha2
- [ ] No `.unwrap()` outside `#[cfg(test)]` in new modules
- [ ] `cargo check --workspace` is green; existing D-3 PLANNED surface unchanged (additive only)

## Tests to add

- `crates/orchestrator/tests/z3_smt_check_rejects_unknown_as_consensus_vote.rs`: Z3 Unknown → abstain; consensus rule does NOT see it
- `crates/orchestrator/tests/z3_smt_check_only_fires_on_csq_critical.rs`: CSQ NonCritical → no Z3 invocation
- `crates/orchestrator/tests/kani_harness_blocks_on_falsified.rs`: Kani Falsified → AUDITING BLOCK with counterexample
- `crates/orchestrator/tests/kani_harness_timeout_does_not_auto_block.rs`: Kani Timeout → soft warning, not BLOCK
- `crates/orchestrator/tests/verus_invariant_ci_gates_commit.rs`: simulate Verus violation → CI job fails (verify via the harness script exit code)
- `crates/orchestrator/tests/dafny_spec_validator_blocks_on_mismatch.rs`: Dafny verifier returns NotVerified → BLOCK
- `crates/orchestrator/tests/lean_premise_hint_does_not_count_toward_consensus.rs`: Lean hint arrives async after BLOCK; consensus_rule MUST NOT see it
- `crates/orchestrator/tests/bounded_retry_stops_at_three.rs`: 3 failed Z3 retries → FINDING emitted, NO 4th retry
- `crates/orchestrator/tests/same_spec_retry_rejected.rs`: 2nd retry with same Z3 query → formal_verifier_no_progress
- `crates/contracts/tests/no_llm_writes_both_spec_and_proof.rs`: SpecOrigin::SameLlmAsImpl → REFUSED + spec_origin_violation_blocked
- `crates/contracts/tests/no_lean_as_default_gate.rs`: assert no `VerifierKind::Lean` variant exists
- `crates/contracts/tests/critical_path_only_invokes_formal_verifier.rs`: NonCritical path → zero invocations
- `crates/contracts/tests/prmbench_deception_resistance_canary.rs`: 20 deceptive diffs → ≥1 FINDING per
- `crates/dashboard-v2/tests/formal_verification_panel_has_no_lean.test.ts` (if Svelte harness exists; otherwise e2e): assert Lean rows absent from FormalVerificationPanel

## Files touched

- CREATE: `crates/orchestrator/src/z3_smt_check.rs` (~260 LOC)
- CREATE: `crates/orchestrator/src/kani_harness.rs` (~220 LOC)
- CREATE: `crates/orchestrator/src/verus_invariants.rs` (~280 LOC)
- CREATE: `crates/orchestrator/src/lean_premise_hint.rs` (~120 LOC)
- CREATE: `crates/orchestrator/src/formal_verifier_retry.rs` (~140 LOC)
- CREATE: `python/elai_inference/dafny_spec_validator.py` (~240 LOC)
- CREATE: `.claude/agents/spec-validator.md` (~80 LOC agent definition)
- CREATE: `.elai_cc/scripts/verus-check.sh` (~30 LOC)
- CREATE: `.elai_cc/.github/workflows/verus-check.yml` (~40 LOC CI job)
- CREATE: `crates/dashboard-v2/src/lib/components/FormalVerificationPanel.svelte` (~160 LOC)
- CREATE: `crates/dashboard-v2/src/lib/components/CriticalPathClassificationPanel.svelte` (~100 LOC)
- CREATE: 14 test files listed above (~40-140 LOC each)
- CREATE: `workflow/reasoning/E-53-decisions.md` (~120 LOC)
- CREATE: `workflow/reasoning/E-53-prmbench-results.md` (~30 LOC initial scaffold)
- MODIFY: `crates/orchestrator/src/d3_verification.rs` (~50 LOC delta — VerifierKind::Z3SmtCheck variant + consensus_rule glue; verify file exists via Glob first — if D-3 is PLANNED-not-shipped, ship a minimal D-3 surface stub alongside)
- MODIFY: `crates/orchestrator/src/csq.rs` (~30 LOC delta — verify CounterfactualClass::Critical variant exists; ADD if absent; verify path via Grep `CounterfactualClass|CSQ::Critical`)
- MODIFY: `crates/orchestrator/src/auditor.rs` (~40 LOC delta — invoke KaniHarness on functions tagged with kani::proof; verify via Grep first)
- MODIFY: `crates/dashboard-v2/src-tauri/src/commands.rs` (~80 LOC delta — get_formal_verification_outcomes + get_critical_path_classifications Tauri commands)
- MODIFY: `Cargo.toml` workspace (~20 LOC delta — add z3, kani-verifier dev-dep; pin versions verified via Anti-slopsquatting)
- MODIFY: `python/elai_inference/pyproject.toml` (~10 LOC delta — declare dafny CLI dep + path)

## Citations / Grounding

- https://arxiv.org/abs/2603.21149 — Emergent Formal Verification: 6 autonomous AI ecosystems converged on Z3; grounds Task 1 Z3-as-primitive choice
- https://arxiv.org/pdf/2604.05292 — Broken by Default: 3500 LLM artifacts via Z3 COBALT; SMT catches CWE-131/190/916 that static analyzers miss; grounds Task 1 Z3 gate
- https://arxiv.org/pdf/2409.13082 — AutoVerus: >90% on 150 Verus tasks, half complete in <30s or ≤3 LLM calls; grounds Task 3 production-feasible latency
- https://arxiv.org/html/2412.06176v1 — AlphaVerus: Dafny→Verus translation + Treefinement; grounds Task 4 Dafny IR pattern
- https://arxiv.org/abs/2509.22908 — Vericoding benchmark: 82% Dafny / 44% Verus / 27% Lean; grounds Dafny-as-cheap-entry framing
- https://arxiv.org/abs/2507.16331 — Re:Form RL-on-Dafny; even 0.5B verifies after SFT; grounds Task 4 LLM-can-emit-Dafny
- https://arxiv.org/pdf/2501.06283 — Dafny as Verification-Aware IR (POPL 2025); grounds Task 4 IR-pattern
- https://arxiv.org/abs/2501.03124 — PRMBench Deception Resistance; grounds Task 7 canary
- https://model-checking.github.io/kani/ + https://aws.amazon.com/blogs/opensource/how-open-source-projects-are-using-kani-to-write-better-software-in-rust/ — Kani CBMC-backed, AWS-maintained, Autoharness 2025; grounds Task 2
- https://www.microsoft.com/en-us/research/publication/verus-a-practical-foundation-for-systems-verification/ — Verus SMT-based deductive verifier; grounds Task 3
- https://venturebeat.com/security/theorem-wants-to-stop-ai-written-bugs-before-they-ship-and-just-raised-usd6m — Theorem YC S25, $6M seed, caught bug past Anthropic; grounds existence-of-commercial-demand
- https://www.shiplight.ai/blog/ai-generated-code-has-more-bugs — AI PRs 1.7x more bugs; modern review 42-48% catch rate vs <20% static analyzers; grounds gate-earns-keep
- https://martin.kleppmann.com/2025/12/08/ai-formal-verification.html — independent expert framing of same hypothesis
- https://www.nature.com/articles/s41586-025-09833-y — AlphaProof IMO 2024 silver, 80M auto-formalized statements; TPU-day cost is prohibitive; grounds LEAN HARD AVOID
- https://www2.eecs.berkeley.edu/Pubs/TechRpts/2025/EECS-2025-121.pdf — Berkeley TR: AlphaProof ~500 TPU-days/problem; grounds LEAN HARD AVOID
- https://seed.bytedance.com/en/blog/seed-prover-1-5-advanced-mathematical-reasoning-through-a-novel-agentic-architecture — Seed-Prover 1.5 16.5h GPU for 5 IMO 2025 problems; grounds LEAN HARD AVOID
- https://arxiv.org/abs/2404.12534 — Lean Copilot suggest_tactics/search_proofs/select_premises; grounds Task 6 read-only premise-hint
- https://www.arxiv.org/pdf/2601.05385 — DafnyPro POPL'26: 95.67% syntax → 3.69% verified cliff; grounds DAFNYCOMP anti-pattern in stub_ban
- https://dl.acm.org/doi/full/10.1145/3788731.3788734 — LDSW: iterating LLM with solver-error feedback unbounded thrashes; grounds Task 9 bounded retry
- CLAUDE.md "CSQ: counterfactual" rule — grounds Task 1 Critical-path classifier trigger
- CLAUDE.md anti-stale-training — blake3 only; no .unwrap() in lib code; grounds stub_ban
- CLAUDE.md "MEMORIZING delegation" — pattern for SpecValidator agent (Task 5)

## Open questions / risks

- Open Q1 (from report): does adding Z3 to D-3 anti-rubber-stamp aggregator reduce false-positive FINDINGs OR add them? Measure on PRMBench Deception Resistance subset via Task 7 canary; carry forward to E-53-decisions.md.
- Open Q2 (from report): MiniF2F-equivalent for ELAI's IPC v2 / SSB / FSM invariants — can autoformalize spec into Verus in <100 LOC per crate? Task 3 ships the FSM/IPC/KV-cache subset; remaining surfaces deferred to follow-up.
- Open Q3 (from report): wall-clock budget per audit — Z3 + Kani latency budget. Measured numbers carried into E-53-decisions.md after first sessions; default 5s Z3 + 30s Kani per harness.
- Open Q4 (from report): Dafny-IR-Python lower bug rate on financial/auth Python paths vs Python type-hints + property tests alone? AB on a real ticket; record in E-53-decisions.md.
- Open Q5 (from report): can local 3B Qwen (NF4) generate Verus proof annotations on simple invariants, or routes to cloud? Re:Form's 0.5B Dafny SFT is suggestive but Verus is harder than Dafny per vericoding. Pilot in Task 3 — falls back to cloud Sonnet via PerRoleRuntime if local fails.
- Risk 1: z3 / kani-verifier / verus / dafny / lean-copilot are 5 new dependencies — explicit user confirmation per CLAUDE.md "No new dependencies without confirmation" required before merge. Mitigation: E-53 ships behind a `formal-verification` feature flag in workspace Cargo.toml; flag is OFF by default; user must opt in.
- Risk 2: kani-verifier requires CBMC backend install (system-wide). Mitigation: Task 2 wraps `cargo kani` invocation with `which cargo-kani || warn` and downgrades to soft NotApplicable on missing install rather than BLOCK. Documented in E-53-decisions.md.
- Risk 3: Dafny binary install path varies per platform (Win/Mac/Linux). Mitigation: Task 4 wrapper queries `dafny --version` first; on miss, downgrades to NotApplicable + emits row. Verus and Lean Copilot similar.
- Risk 4: Lean Copilot is the heaviest install (mathlib pull, ~2-4GB disk). Mitigation: Task 6 makes Lean premise-hint OPT-IN via `ELAI_LEAN_PREMISE_HINT=1` env var (default OFF). If Lean install missing, no SSB row, no error — just absent hint.
- Risk 5: DAFNYCOMP cliff guard (SpecOrigin::SameLlmAsImpl REFUSED) requires the system to know which LLM wrote the impl AND which produced the spec. Mitigation: PerRoleRuntime already tracks per-role backend (per E-51 + B-37); the SpecValidator agent reads role-of-impl-author and role-of-spec-author and compares.
- Risk 6: Verus CI gate (5min cap) may flake on slow GH runners. Mitigation: CI job has retry-once-on-timeout flag; persistent failure escalates to issue auto-filed, not silent skip.
- Risk 7: PRMBench canary requires 20 hand-curated deceptive diffs. Mitigation: Task 7 ships the 20-diff fixture under `crates/contracts/fixtures/prmbench/`; each diff has human-curated label `deceptive: true` + ground-truth `expected_finding: <reason>`. Generation cost is 1 engineer-half-day, one-time.

## Next

E-53 closes the Z3+Kani+Verus+Dafny verifier-tier gap in D-3's consensus rule while keeping Lean theorem-prover loops explicitly OUT of routine audit (HARD AVOID per report's 500-TPU-day cost). After E-53, every CSQ-Critical Rust function gets a sub-second Z3 SMT check + 30s Kani harness; every Critical Python path gets a 30s Dafny SpecValidator pass; ELAI's FSM / IPC v2 / KV-cache invariants are CI-gated by Verus on every PR (NOT per-audit, wall-clock-safe). PRMBench Deception Resistance canary runs nightly. The DAFNYCOMP cliff (spec-and-proof-by-same-LLM) is hard-guarded by SpecOrigin tracking. Lean Copilot is available as a read-only premise hint when verification BLOCKs fire — informational only, never on consensus path. Downstream: E-52 (LSP diagnostic probe) and E-53 share the auditor-evidence surface; both feed `EvidenceVariant`; coordinate on row emission. Patch E-53.1 candidate: measure Open Q1-Q5 from the report on first sessions and tune timeouts / activation thresholds. Patch E-53.2: extend Verus invariants beyond FSM/IPC/KV-cache to additional safety-critical surfaces as they emerge (per Open Q2 vericoding scope). Patch E-53.3: if PRMBench canary surfaces a class of diffs the verifiers miss, add a new VerifierKind that targets that class.
