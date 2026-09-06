# TDD + verification agents for ELAI

## TL;DR
- Replace single-test-flip EXECUTING gate with **dual oracle**: ≥1 example test flip AND ≥1 property/invariant pass (PGS shows +13.4% pass@1 vs example-only TDD; arXiv:2506.18315). Properties resist sycophancy better than examples.
- Auditor gains a **mutation-survival gate**: run `cargo-mutants` on the changed crate at AUDITING; any surviving mutant in the touched function/file is a BLOCK FINDING. Meta ACH proves the targeted-mutant approach: privacy engineers accepted 73% of LLM-tests generated against synthetic mutants (arXiv:2501.12862).
- Add **anti-over-mock lint** to PostToolUse hook on test files: Hora & Robbes 2026 find coding-agent commits add mocks at 36% vs 26% for humans (arXiv:2602.00409). MistakeBook should ban auto-generated `mock.patch`/`vi.mock`/`mockall` without corresponding integration-level reference test.
- Promote IntegrationTestRunner CONCEPT level to run **proptest + cargo-mutants in parallel** with a cached fixed seed corpus, gated by changed-files diff (avoids combinatorial blow-up). Use property-coverage (% invariants exercised) not line coverage as the level metric.
- Acknowledge SWE-bench-Verified rot: UTBoost flags 345 erroneously-passed patches affecting 24.4% of Verified leaderboard entries (arXiv:2506.09289). ELAI's PRMBench-style ≥2-verifier consensus (D-3) should mirror Nexus 4-agent deliberative critique with execution-grounded validation, not just LLM-pair vote.

## Key papers / repos
arxiv-id / URL | title | takeaway
- arXiv:2510.09907 | Agentic Property-Based Testing (Maaz et al., 2025) | LLM agent infers properties + runs Hypothesis: 56% of bug reports across 100 Python packages were valid bugs; 86% precision at top-21 ranked. Local PDF saved. | AI test quality + PBT
- arXiv:2506.18315 | Property-Generated Solver (PGS) | Generator+Tester agents w/ PBT counterexamples; +13.4% pass@1 vs example-TDD, 1.4–1.6x bug-fix rate vs debug-focused methods, 64% fix rate on initial fails. Local PDF saved. | PBT, self-debug
- arXiv:2506.09289 | UTBoost: Rigorous Evaluation of Coding Agents on SWE-Bench | 345 erroneously-labeled-passed patches; 40.9% Lite + 24.4% Verified leaderboard entries affected; 36 instances with insufficient test coverage. Local PDF saved. | AI test quality
- arXiv:2501.12862 | Mutation-Guided LLM-based Test Generation at Meta (ACH) | LLM-generated mutants + tests that catch them; 73% engineer acceptance on privacy domain, 36% privacy-relevant; first large-scale industrial deployment of mutant+test co-generation. Local PDF saved. | mutation, contracts
- arXiv:2404.09952 | LLMorpheus | LLM-generated mutants produce realistic real-bug-mimicking mutations StrykerJS cannot (e.g. wrong-method calls). | mutation
- arXiv:2502.14212 | Less Is More: Data Quality for Unit Test Generation (FSE 2025) | 43.52% Methods2Test / 29.65% Atlas datasets are noise; CleanTest filter raises branch coverage +67% (Methods2Test) and bug detection +21.42%. | AI test quality, when MORE hurts
- arXiv:2303.11366 | Reflexion (Shinn et al.) | Verbal-RL self-reflection over execution traces: 91% pass@1 on HumanEval vs 80% GPT-4 baseline. | self-debug
- arXiv:2312.13010 | AgentCoder | 3-agent (programmer / test-designer / test-executor) split; foundational pattern for ELAI's separation of executor from auditor. | self-debug
- arXiv:2402.01030 | CodeAct | Replaces JSON-action with executable Python; tighter feedback loop than text-only critique. | self-debug
- arXiv:2308.04748 | Fuzz4All (ICSE 2024) | LLM as fuzzer-input generator across 6 langs; found 98 bugs (64 confirmed novel) in GCC/Clang/Z3/CVC5/OpenJDK/Qiskit. | fuzzing
- arXiv:2406.08467 | DafnyBench | Best model+prompt reaches 68% success generating Dafny verification hints; baseline for ProofGPT/AlphaProof-style verification probes. | formal verification
- arXiv:2505.13938 | CLEVER: Curated Benchmark for Formally Verified Code Generation | Lean-based verified codegen benchmark for measuring spec-conformance not just test-passing. | formal verification
- arXiv:2509.22908 | VeriCoding | Formally-verified program-synthesis benchmark (Lean / Verus / Dafny). | formal verification
- arXiv:2503.05061 | No Free Labels: Limits of LLM-as-Judge Without Human Grounding | LLM judges show TPR>96% but TNR<25% under class imbalance — sycophancy/agreeableness bias. | oracles
- arXiv:2601.05542 | Understanding LLM-Driven Test Oracle Generation (ASE 2025) | Full class context +12.9% oracle quality; simpler prompts +23% over complex ones; assertion-compilability is dominant failure mode. | oracles
- arXiv:2504.17287 | RBCTest: LLM mines API-response-body oracles | Observation-Confirmation cuts hallucination; 85.1–93.6% precision on constraint mining from OpenAPI specs. | contracts, oracles
- arXiv:2602.00409 | Are Coding Agents Generating Over-Mocked Tests? (Hora & Robbes, MSR 2026) | Across 1.2M commits / 2168 repos: agents add mocks in 36% of commits vs 26% non-agents; 23% test-modification rate vs 13%. | AI test quality
- https://github.com/sourcefrog/cargo-mutants | cargo-mutants | Production Rust mutation-tester; works on unmodified trees; supports return-value-replace, binary-op-replace, unary-op-delete, match-arm-delete, struct-field-delete (incl. `..Default::default()`-base mutations). | mutation
- https://docs.pact.io/ | Pact (Consumer-Driven Contract testing) | Broker + can-i-deploy gates; bi-directional w/ OpenAPI; 2025-best-practices doc on Pact site. | contracts
- https://github.com/SWE-agent/SWE-ReX | SWE-ReX | Sandboxed massively-parallel exec runtime backing SWE-agent; reference for how to wire IntegrationTestRunner's CONCEPT-level isolation. | TDD-for-agents in prod
- https://engineering.fb.com/2025/09/30/security/llms-are-the-key-to-mutation-testing-and-better-compliance/ | Meta ACH blog | "shift from coverage-as-goal to fault-as-goal" — direct policy ELAI MistakeBook should adopt. | mutation
- https://arxiv.org/abs/2509.02360 | PRMs for course-correcting SWE agents | Process-Reward Models intervene mid-trajectory; relevant to ELAI's D-3 ≥2-verifier consensus. | oracles, self-debug

## Test-generation quality benchmark
| technique | tool | benchmark (cite) | catch rate vs vacuous rate | local-7B feasible? |
| --- | --- | --- | --- | --- |
| LLM example-test gen (raw) | Aider/SWE-agent | UTBoost on SWE-bench Verified (arXiv:2506.09289) | 19.78% of "solved" are semantically wrong; 24.4% of Verified leaderboard affected | yes (local Qwen-2.5-Coder-7B already used by ELAI) |
| LLM example-test gen (cleaned data) | CleanTest | Defects4J (arXiv:2502.14212) | +21.42% bug-detection; +67% branch-cov | yes — filtering is offline |
| LLM property-based testing | PGS / agentic-PBT | HumanEval/MBPP (arXiv:2506.18315); 100 Python pkgs (arXiv:2510.09907) | +13.4% pass@1 over example-TDD; 56% of bug reports valid, 86% at top-21 | yes — Hypothesis/proptest run locally; LLM only generates strategies |
| LLM-generated mutation tests | ACH (Meta) | privacy mutants on FB/IG/WA Kotlin code (arXiv:2501.12862) | 73% engineer-accept; 36% domain-relevant | borderline — mutant gen is heavy, but cargo-mutants engine itself is local & cheap |
| LLM-mutated source code (LLMorpheus) | LLMorpheus | StrykerJS comparison (arXiv:2404.09952) | finds wrong-method-call faults Stryker cannot | yes for mutation phase; test-kill requires existing suite |
| LLM as test oracle | LLM-judge | ASE 2025 (arXiv:2503.05061, 2601.05542) | TPR >96% / TNR <25% — over-accepting; +12.9% w/ class context | yes BUT must pair w/ deterministic check |
| LLM-guided fuzzing | Fuzz4All | C/C++/Go/SMT2/Java/Python (arXiv:2308.04748) | 98 bugs found, 64 confirmed-novel in GCC/Clang/Z3/CVC5/OpenJDK/Qiskit | yes — LLM only generates seeds; fuzzer runs locally |
| Verified codegen | Dafny+LLM | DafnyBench (arXiv:2406.08467) | 68% success generating hints | partial — Dafny tooling heavy; Lean/Verus alternatives lighter |

## Best techniques to adopt
Ranked by ROI for ELAI's current FSM:

1. **Property-Based Testing as second-axis EXECUTING gate** | grounded: arXiv:2506.18315 (+13.4% pass@1) + arXiv:2510.09907 (56% valid bug rate) | hook: EXECUTING currently requires `≥1 test flipped fail→pass`; extend to `(≥1 example flip) AND (≥1 proptest run with ≥10 cases, zero panics, ≥1 non-trivial property predicate)`. proptest in Rust crates, hypothesis in `.elai_cc/python/elai_inference/tests/`. Vacuous-property detection: reject `proptest!{|x| true}` style at PreToolUse on test files.

2. **cargo-mutants survival gate at AUDITING for changed crates** | grounded: arXiv:2501.12862 (Meta ACH 73% accept); cargo-mutants README. | hook: IntegrationTestRunner STEP level adds `cargo mutants --in-place --baseline skip --shard <touched_files>` with budget (60s STEP / 120s PHASE / 300s CONCEPT). Any MISSED mutant in a touched function = BLOCK FINDING (anti-rubber-stamp already requires ≥1 finding — this guarantees a real one). Cache survivor list in MistakeBook keyed by file+sha to avoid re-checking unchanged code.

3. **Anti-over-mock policy in MistakeBook** | grounded: arXiv:2602.00409 (36% vs 26% mock-add rate). | hook: PostToolUse on `*.test.ts|*.test.py|tests/*.rs`: count `mock.patch|vi.mock|mockall::|MockAll` insertions; if added without a matching `#[ignore]`-tagged integration test in the same diff, add a forbidden-diff hash. Mutation-test the mocked behavior — if all mutants of the real impl survive, BLOCK.

4. **PRM/multi-verifier consensus for D-3 promotion** | grounded: arXiv:2509.02360 (PRMs course-correct SWE agents), arXiv:2503.05061 (single-LLM judge TNR<25%). | hook: VerificationResult aggregator already requires ≥2 verifiers; require ≥1 of those to be a **non-LLM verifier** (cargo test, cargo-mutants survival, proptest panic-free). Pure LLM-LLM agreement under-counts negatives.

5. **Contract tests + bi-directional schema check at PHASE level** | grounded: Pact 2025 best-practices; arXiv:2504.17287 RBCTest. | hook: For ELAI's sidecar↔CLI↔dashboard boundaries, the existing `tests/contracts/` directory becomes the contract-broker analogue. At PHASE level, run schema-diff: SCIP-extracted public API from `elai.map` vs sibling project's consumed-symbol list. Use the already-tracked `SCHEMA_VERSION` bump as the can-i-deploy gate.

6. **Mutant-guided test gen as a sidecar agent role** | grounded: Meta ACH (arXiv:2501.12862); LLMorpheus (arXiv:2404.09952). | hook: New agent `mutant-hunter.md` (paired w/ existing auditor). When cargo-mutants surfaces a SURVIVED mutant, mutant-hunter prompts local Qwen-2.5-Coder-7B to author one test that kills it. Test must compile and flip the mutant red. Same FSM substate as AUDITING.

7. **Fuzz4All-style LLM-seeded fuzzing for parsers/serializers** | grounded: arXiv:2308.04748 (98 bugs / 64 novel). | hook: CONCEPT-level only; gated by file pattern (`crates/*/src/parser*.rs`, `crates/*/src/wire*.rs`, IPC v2 message serializers). LLM-seed + cargo-fuzz, 60-120s budget.

8. **Hypothesis/proptest oracle: round-trip + metamorphic + invariants** | grounded: arXiv:2510.09907 categorization. | hook: A `properties.md` per crate listing required (round-trip serde, idempotence, monotonicity, etc.). spec-grounder agent reads it; spec-link-check enforces presence for crates over a LOC threshold.

9. **Verified-codegen escape hatch (Dafny/Verus) for crypto/consensus paths** | grounded: arXiv:2406.08467 DafnyBench; arXiv:2509.22908 VeriCoding. | hook: ONLY for ELAI's blake3 wrappers / DKG verifier / consensus check paths; quarantined experiment, not in mainline FSM. Local 7B insufficient — defer to cloud Sonnet 4.6.

10. **Process-Reward-Model gating at AUDITING** | grounded: arXiv:2509.02360. | hook: Train tiny PRM on existing audit traces (BLOCK vs PASS); use to early-abort spiraling executor before 3-edit threshold.

## When MORE tests hurt
- **Data-quality > volume**: 43.52% noise in Methods2Test; filtering raised bug-detect +21.42% AND removed coverage-inflation (arXiv:2502.14212). Adding more LLM-generated tests without quality filter = adding more agree-with-the-implementation tests.
- **Over-mocking inflation**: agent-authored tests add mocks 36% vs human 26% (arXiv:2602.00409). A mock-heavy suite covers the test, not the impl — mutation testing reveals this (mutate impl, mock-tests pass).
- **SWE-bench-Verified rot**: UTBoost identifies 345 inflated passes, 24.4% of Verified leaderboard affected, 36 instances with insufficient tests (arXiv:2506.09289). Larger benchmark suite did not == higher fidelity; quality auditing did.
- **Maintenance cost scales linear w/ suite size**; QA spends 20-30% of work week on flake triage (industry data from Diffie / Rainforest QA blog). ELAI's auditor budget is bounded — each added test is a recurring tax on every AUDITING cycle.
- **LLM-judge sycophancy**: TPR>96% / TNR<25% on adversarial inputs (arXiv:2503.05061). More LLM-evaluated tests = more false-confidence; absolute gain only from deterministic kill-criteria (mutant-survival, panic, assertion).
- **Snapshot/golden-image fragility**: practitioner consensus (TeachMeIDEA, Shakacode, Qase 2025 sources) is that the maintenance cost of golden snapshots dominates resilience layer; recommend reserved use for rendered UI / serializer-output only.

## Anti-patterns
- **Vacuous tests** — `assert True`, `assert result is not None`, `assert len(x) >= 0`. UTBoost finds these inflate pass rates; ELAI MistakeBook should regex-ban (arXiv:2506.09289).
- **Test mirrors implementation** — auto-gen tests that copy the impl AST and assert it (Hora & Robbes 2026 over-mock pattern, arXiv:2602.00409).
- **LLM-judge as sole oracle** — calibration error compounds (arXiv:2503.05061); always pair w/ deterministic check.
- **Coverage-as-goal** — Meta ACH paper explicitly shifts to fault-as-goal; coverage % is gameable.
- **Single example-TDD without properties** — PGS shows example-TDD plateaus; properties recover +13.4% pass@1 (arXiv:2506.18315).
- **Mock-all-deps unit tests** — covers test infrastructure, not behavior; survives all impl mutations.
- **Self-debug without external verifier** — Reflexion's 91% HumanEval depends on external test feedback; pure self-critique loops cap lower (arXiv:2303.11366).
- **Trust SWE-bench-Verified pass count uncritically** — 24.4% leaderboard rate inflated (arXiv:2506.09289).

## Open questions
1. **Measure local-7B mutant-test-kill rate**: run cargo-mutants on current ELAI crates with Qwen-2.5-Coder-7B writing the kill-tests. Hypothesis: ≥40% kill rate makes it production-viable. Cost ceiling: 20K-token-context per mutant.
2. **Property-coverage as IntegrationTestRunner CONCEPT-level metric** — define "property coverage" (% of invariants in `properties.md` exercised w/ ≥100 cases) and measure vs branch-coverage on 3 ELAI crates; does property-cov correlate w/ mutant-kill rate r > 0.6?
3. **Anti-over-mock heuristic threshold**: at what mock-count-per-test does mutant-kill drop below 30%? Need empirical curve on ELAI's existing test suite before setting MistakeBook ban threshold.
4. **D-3 ≥2-verifier consensus calibration**: current setup may be 2× LLM. Measure TPR/TNR gap when forcing ≥1 non-LLM verifier on a held-out set of known-good and known-broken PRs. Target: TNR ≥ 70%.
5. **proptest seed corpus per FSM transition**: should EXECUTING cache a fixed-seed corpus per phase to avoid flake-then-pass, or randomize per run? Measure flake rate at 100 / 1K / 10K seeds.
