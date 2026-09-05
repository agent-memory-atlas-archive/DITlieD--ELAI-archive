Title: Compiled AI: Deterministic Code Generation for LLM-Based Workflow Automation
Authors: Geert Trooskens, Aaron Karlsberg, Anmol Sharma, Lamara De Brouwer, Max Van Puyvelde, Matthew Young, John Thickstun, Gil Alterovitz, Walter A. De Brouwer
Affiliations: XY.AI Labs (Palo Alto), Stanford University School of Medicine, Cornell University CS, Brigham and Women's Hospital / Harvard Medical School
arXiv ID: 2604.05150
Published: April 2026

---

CORE CLAIM / CONTRIBUTION

The paper formalizes "Compiled AI" as a production deployment pattern where an LLM is invoked ONCE at workflow-design time to generate business-logic code, and all subsequent workflow executions are fully deterministic, zero-token static code. The three defining properties:

1. One-time LLM invocation -- model runs only at generation/compilation time, never at transaction time.
2. Zero-token deterministic execution -- deployed workflows are static code with no further LLM calls.
3. Mandatory multi-stage validation -- every generated artifact must pass security, syntax, execution, and accuracy stages before deployment.

Contributions: (1) system architecture for constrained code generation via templates + modules, (2) a four-stage generation-and-validation pipeline (Algorithm 1), (3) an evaluation framework measuring token amortization, determinism, reliability, security, and cost rather than just task accuracy.

---

PROBLEM

Runtime LLM agents (AutoGen, LangChain, etc.) invoke the model on every transaction, leading to:
- High token costs scaling linearly with transaction volume
- Variable latency (2000ms+ P50 for LLM calls vs sub-10ms for native code)
- Non-determinism even at temperature=0 (Atil et al. 2024: up to 15% accuracy variance; Ouyang et al. 2023: 18-75% output variance due to MoE routing and other architectural factors)
- Wide attack surface -- prompt injection is possible on every transaction
- Poor auditability (no traceable code path for compliance)

Multi-agent failure analysis: Cemri et al. 2025 found 79% of multi-agent failures stem from specification and coordination issues (41.8% specification failures, 36.9% coordination failures), not infrastructure. Salesforce CRMArena-Pro shows agent success degrading from 58% single-turn to 35% multi-turn.

Healthcare context is the primary motivation: prior authorization, billing reconciliation, and clinical document processing require determinism, auditability (CMS decision timeframes, FDA AI/ML medical device rules), and HIPAA compliance. These workflows are "intelligent to design but not to execute repeatedly."

---

ARCHITECTURE

System receives a YAML workflow specification and produces a validated Temporal activity (Temporal.io workflow framework) as output. Core components:

ORCHESTRATOR
- Receives YAML workflow spec
- Selects appropriate template and modules
- Assembles the prompt (template + modules + prompt blocks + spec)
- Invokes LLM once
- Sends assembled artifact through validation pipeline
- On failure: regenerates with error context (not retries from scratch)

TEMPLATE LIBRARY
- Pre-tested code patterns for: synchronous handlers, streaming processors, batch processors with checkpointing, input validators with fallback logic
- Templates provide the infrastructure scaffold; LLM only fills in the bounded business-logic function (20-50 lines)
- Bounding the output space reduces hallucination risk because API schemas, DB schemas, and type signatures come from pre-validated templates, not LLM generation

MODULE LIBRARY
- Reusable functional capabilities: database access with connection pooling, HTTP clients with retry logic, notification delivery
- Selected by the orchestrator based on workflow spec needs

PROMPT BLOCKS
- Encode domain constraints: HIPAA data handling rules, CMS compliance requirements, PCI-DSS, SOC 2
- Compliance-by-construction: generated code inherits compliance properties because constraints are in the prompt template, not left to model discretion

CODE GENERATION (LLM CALL)
- Single invocation of the LLM (paper uses Claude Opus 4.5 at temperature=0)
- Output is constrained to the bounded business-logic section of the template
- PydanticAI for structured orchestration and typed output extraction
- Custom DSL executor for workflow definitions

FOUR-STAGE VALIDATION PIPELINE (Algorithm 1)

Stage 1 -- Security:
- Static analysis using Bandit + Semgrep + custom rules
- Checks: SQL injection (CWE-89), command injection (CWE-78), path traversal, secrets exposure, CWE-94 (code injection), CWE-502 (deserialization), CWE-328 (hash)
- 100% precision (zero false positives on benign workflows)
- 75% recall on 20 known-vulnerable code fixtures

Stage 2 -- Syntax:
- AST parsing
- Type checking via mypy
- Linting via ruff
- First-pass rate: 100% on BFCL evaluation set

Stage 3 -- Execution:
- Sandboxed execution against test fixtures
- Verifies: successful completion, correct error handling, output structure conformance
- First-pass rate: 100% on BFCL evaluation set

Stage 4 -- Accuracy:
- Output comparison against golden datasets
- Task-specific thresholds
- First-pass rate: 96% on BFCL (the 4% failures are semantically incorrect but syntactically valid -- caught before deployment)

On any stage failure: regenerate with error context appended to prompt (not full restart). The loop is: Generate -> Assemble -> [Security -> Syntax -> Execution -> Accuracy] -> if FAIL then Regenerate with errors -> repeat until all pass or exhaust budget.

THREE-GATE SECURITY ARCHITECTURE (orthogonal to the validation pipeline)

Input Gate:
- DeBERTa-v3 for prompt injection detection
- Presidio for PII scanning
- 95.8% recall, 100% precision on 30 adversarial injection inputs

Code Gate:
- Static analysis for CWE-94, CWE-89, CWE-502, CWE-78, CWE-328

Output Gate:
- Cryptographic canary token injection for prompt leakage detection
- 12.5% recall in simulation (acknowledged as methodology artifact -- canaries only fire when attacker has specific access to prompt context)
- 100% precision

Dual LLM Pattern: privileged and quarantined model instances are separated to prevent the quarantined model from escaping its sandbox.

BOUNDED AGENTIC INVOCATION (Code Factory variant)
For semantically complex tasks that pure deterministic code cannot handle (e.g., extracting structured fields from noisy invoice text), the generated code itself calls an LLM for a narrow, schema-constrained subtask. This is still "compiled" because the calling code is static, the schema is fixed, fallback logic is deterministic, drift monitoring is defined, and human escalation thresholds are hardcoded. The LLM call per execution is narrow and bounded by Pydantic schema, with automatic retry on schema violations.

SAFETY SANDWICH PATTERN (from Appendix A healthcare example):
- Phase 1: Bounded agentic invocation (LLM extracts structured clinical factors from unstructured note, output validated against schema with range checks e.g., "3.0 < a1c < 20.0")
- Phase 2: Hallucination validation (sanity-check ranges on extracted values, trigger human review if suspicious)
- Phase 3: Deterministic decision (pure if/elif/else logic, no model involvement)
- Phase 4: Audit trail (log decision event with logic version to enable future re-audit)

WORKFLOW EXECUTION SUBSTRATE: Temporal.io is the production runtime -- workflows are Temporal activities, which provide durable execution, automatic retry, and state persistence.

---

IMPLEMENTATION DETAILS

Algorithm 1 (pseudocode from paper):
  t <- SelectTemplate(S, T)       // pick template from library based on spec
  m <- SelectModules(S, M)        // pick modules (db, http, notify)
  p <- AssemblePrompt(S, t, m)    // combine spec + template + modules + prompt blocks
  code <- LLMGenerate(p)          // ONE-TIME call
  A <- Assemble(t, m, code)       // slot generated code into template scaffold
  for stage in {Security, Syntax, Execution, Accuracy}:
      result <- Validate(A, stage)
      if result == FAIL:
          code <- Regenerate(p, result.errors)  // error context appended
          A <- Assemble(t, m, code)
  return A

Generated function size constraint: 20-50 lines. This is a hard design choice -- small enough that the LLM stays within a well-posed generation task, small enough for effective static analysis, small enough for full type annotation.

LLM used in evaluation: Claude Opus 4.5 (claude-opus-4-5-20251101), temperature=0.
Structured extraction: PydanticAI.
DSL executor: custom (not released in full detail in the paper).
Code quality tooling: mypy (type checking), ruff (linting), Bandit+Semgrep (security), cyclomatic complexity measured but not gated.

YAML spec format (from Appendix A healthcare example):
  metadata: name, compliance list
  inputs: named typed inputs
  logic_requirements: list of steps, each with:
    - type: "bounded_invocation" (LLM call with schema) or "deterministic_rule"
    - schema (for bounded_invocation): Pydantic-style field definitions with types
    - validation: list of range/constraint expressions
    - logic (for deterministic_rule): natural language IF/THEN/ELSE logic

Pydantic output validation on the Code Factory path includes automatic retry on schema violations (contributes to the 10% retry outlier tail -- median 2,695ms, mean 10,263ms, max 183s on DocILE).

TOKEN AMORTIZATION FORMULA:
  n* = GENTOKENS_compiled / (RUNTIMETOKENS_per-tx-runtime - RUNTIMETOKENS_per-tx-compiled)
  n* = 9,600 / (552 - 0) ≈ 17 transactions for BFCL task
  At n=1,000: compiled uses 9,600 total vs 552,000 for direct LLM -> 57x reduction
  At n=1M/month: compiled TCO $555 vs direct LLM $22,000 (40x ratio at Claude Opus 4.5 pricing: $15/1M input, $75/1M output)

DETERMINISM METRIC:
  Output entropy H = -sum(p_i * log(p_i)) over N=1,000 identical inputs
  Target: H=0 (zero entropy = fully deterministic)
  Compiled AI achieves H=0 empirically
  Runtime inference at temperature=0 achieves 95% reproducibility (not 100%) due to MoE routing and batch-order nondeterminism

CYCLOMATIC COMPLEXITY of LLM-generated code: average 23.8 vs 8 for human code. The paper attributes this to LLM tendency to generate switch-case structures. Flagged as "area for template refinement" but not addressed in evaluation.

CODE QUALITY METRICS:
  Type coverage: 99%
  Test pass rate: 96%
  Cyclomatic complexity: 23.8 (elevated, known LLM artifact)

---

EVALUATION SETUP

Benchmark 1 -- BFCL (Berkeley Function-Calling Leaderboard):
  n=400 instances
  Task: identify correct function + extract parameters from natural language query
  Evaluation: task completion rate, token counts, latency, determinism, code quality

Benchmark 2 -- DocILE (Document Intelligence):
  n=5,680 invoice documents
  Characteristics: degraded OCR, heterogeneous formats across hundreds of vendors, semantic ambiguity
  Metrics: KILE (Key Information Line-level Extraction -- exact match on invoice number, date, total amount, etc.), LIR (Line Item Recognition -- structured line items within documents)

Baselines for both:
  - Direct LLM (Claude Opus 4.5, one call per transaction)
  - LangChain (same LLM with orchestration overhead)
  - AutoGen (same LLM with orchestration overhead)
  - Compiled AI / Code Factory (LLM invoked once at compilation)
  - Deterministic variant (pure regex, no LLM) -- DocILE only

Infrastructure: Claude Opus 4.5 at temperature=0 for all LLM baselines. PydanticAI for structured output. Temporal.io for workflow execution substrate.

---

HEADLINE RESULTS

BFCL (function-calling):
  Task completion: 96% (384/400)
  All 16 failures are compilation-time failures (detectable before deployment)
  Zero execution-time failures on successfully compiled workflows
  Token consumption at 1,000 tx: Compiled AI 9,600 vs Direct LLM 552,000 (57x reduction)
  Token consumption vs AutoGen: 84x reduction at 1,000 tx
  Break-even point: ~17 transactions
  TCO at 1M tx/month: $555 vs $22,000 Direct LLM (40x reduction)
  Latency P50: 4.5ms Compiled AI vs 2,004ms Direct LLM (450x improvement)
  Latency jitter P99-P50: 10.5ms Compiled AI vs 1,123ms Direct LLM
  Reproducibility: 100% Compiled AI vs 95% Direct LLM
  Output entropy: H=0 Compiled AI vs non-zero runtime inference
  Validation first-pass rates: 100% Security, 100% Syntax, 100% Execution, 96% Accuracy

DocILE (document intelligence):
  Code Factory: KILE 80.0%, LIR 80.4%, latency 2,695ms median (10,263ms mean)
  Direct LLM: KILE 80.0%, LIR 74.5%, latency 6,339ms
  LangChain: KILE 80.0%, LIR 75.6%, latency 6,207ms
  AutoGen: KILE 77.8%, LIR 78.9%, latency 13,742ms
  Deterministic (regex): KILE 20.3%, LIR 59.7%, latency 0.6ms
  Code Factory achieves highest LIR (80.4%) and 2.3x lower latency than Direct LLM
  Key failure modes for regex on KILE: customer names 10.1%, currency amounts 9.2%

Security (135 total test cases):
  Overall accuracy: 96.7% prompt injection detection, 87.5% code safety
  Input Gate (DeBERTa-v3 injection): 95.8% recall, 100% precision (30 adversarial inputs)
  Code Gate (static analysis): 75% recall on 20 vulnerable fixtures, 100% precision (20 benign workflows)
  Output Gate (canary): 12.5% recall (acknowledged simulation artifact), 100% precision
  Combined precision: 96.1% across 135 cases; zero false positives throughout

---

LIMITATIONS (stated by authors)

1. Specification quality dependency: requires users to accurately express workflow in YAML. Iterative spec refinement is often necessary. The "specification problem" remains unsolved.

2. Bounded applicability: not suitable for tasks requiring genuine creativity, novel-situation adaptation, or open-ended reasoning. Only covers two task types; broader generalization needs further study.

3. Generation failures: 4% failure rate (16/400 on BFCL). All are accuracy failures (syntactically valid but semantically wrong), all detectable pre-deployment, but they represent a non-zero rejection rate at compile time.

4. Model dependence: generated code quality tied to underlying LLM. Model updates may require re-validation of all compiled artifacts.

5. Benchmark scope: their evaluation framework not yet adopted by other systems, limiting cross-system comparability. DocILE retry tail (max 183s) indicates robustness issues on the Code Factory path.

6. Output Gate canary recall (12.5%) is very low -- acknowledged as a simulation methodology gap, but production effectiveness for prompt leakage is unproven.

7. Elevated cyclomatic complexity (23.8 vs 8 for human code) in generated artifacts -- potential long-term maintainability concern not addressed in evaluation.

---

WHAT IS CONCRETELY REUSABLE FOR ELAI

1. COMPILE-ONCE, EXECUTE-MANY PATTERN FOR ELAI PLAN PHASES
The core insight maps directly to ELAI's orchestrator: tasks that are "intelligent to design but not to execute repeatedly" should be compiled at plan-creation time. Specifically, the prompt-hardening pipeline (B-plan I-47), scoring functions, routing policies, and extraction schemas could be compiled once via LLM and deployed as deterministic code rather than being re-evaluated per-invocation. The break-even formula (n*=GENTOKENS/RUNTIMETOKENS_saved) gives a principled threshold for deciding when compilation is worthwhile.

2. SAFETY SANDWICH PATTERN
Directly applicable to ELAI's agent-skill execution: bounded LLM invocation (e.g., code understanding or doc extraction) surrounded by (a) schema-constrained input validation, (b) hallucination range-check logic, (c) deterministic decision layer, (d) audit trail emission. This is the architecture for ELAI's trust-hardening track (H-17..H-29) -- the paper provides a concrete implementation reference.

3. FOUR-STAGE VALIDATION PIPELINE
ELAI already has wiring-enforcement layers (static, contract, runtime). The paper's SECURITY->SYNTAX->EXECUTION->ACCURACY sequencing with error-context regeneration (not retry-from-scratch) is the missing piece for ELAI's code-generation path when the sidecar or cloud model generates code. The Bandit+Semgrep+mypy+ruff toolchain is directly portable (Python) or has Rust equivalents (cargo clippy, cargo-audit, rustfmt).

4. DUAL LLM PATTERN FOR ISOLATION
Quarantining the model that processes user/external input from the privileged model that accesses internal state is directly applicable to ELAI's trust-hardening architecture. The sidecar (local Qwen) could serve as the quarantined model for processing external doc content, while the cloud model (Sonnet/Opus) remains the privileged orchestrator.

5. BOUNDED AGENTIC INVOCATION AS A FIRST-CLASS CONCEPT
The Code Factory variant formalizes what ELAI does informally: an orchestrator that sometimes calls a model for a narrow subtask. The key discipline is: fixed input schema (Pydantic), fixed output schema, automatic retry on schema violation, drift monitoring, human escalation threshold. This is the spec for ELAI's sidecar invocation protocol.

6. DEBERTA-V3 FOR PROMPT INJECTION DETECTION
95.8% recall, 100% precision on 30 adversarial inputs. Portable model, open-source, runs locally. Applicable as ELAI's Input Gate for any path where external content (user documents, web-fetched content) enters the agent context.

7. TOKEN AMORTIZATION ECONOMICS
The 40x TCO reduction at 1M tx/month validates ELAI's architectural direction of moving intelligence to plan time and keeping execution cheap. At $15/1M input + $75/1M output (Opus 4.5 pricing), the math is concrete. For ELAI's cloud-lane orchestrator running at scale, this framing supports the "compile routing policies once, execute deterministically" direction in E-31.

8. TEMPORAL.IO AS WORKFLOW EXECUTION SUBSTRATE
The paper uses Temporal for durable execution, retry, and state persistence. ELAI's current orchestrator does not use an external workflow substrate. Temporal provides exactly the "workflows survive process restarts" guarantee that ELAI's session/dag system tries to implement manually. Worth investigating as a replacement or complement for the FSM layer.

9. VALIDATION FIRST-PASS RATE AS A NEW METRIC CATEGORY
The paper explicitly flags "Validation pipeline first-pass rates" as a novel metric category not reported by prior work. ELAI's auditing state should emit: first-pass rate per gate (security/syntax/execution/accuracy), overall first-pass rate, and compile-time failure rate. These are the operational metrics that distinguish "does it work" from "is it production-ready."

10. CONTINUOUS OPTIMIZATION VIA RUNTIME METRICS
The paper positions compiled AI as "an evolutionary system: deterministic execution with adaptive improvement, where production metrics continuously inform artifact refinement." This is the operational architecture for ELAI's SAFEDELTA loop -- compile the orchestration logic, measure real outcomes, trigger targeted regeneration of underperforming artifacts (not the entire plan), re-validate, redeploy. The paper provides the operational framing that grounds ELAI's trust-measurement roadmap.

CAUTION / MISMATCH WITH ELAI:
- ELAI's task space includes genuinely novel/creative tasks where the paper explicitly says compiled AI does not apply.
- The 4% compile-time failure rate and the high cyclomatic complexity of generated code are real costs for any production adoption.
- The output gate recall of 12.5% means prompt leakage detection is essentially unproven in this paper's evaluation.
- DocILE retry tail (max 183s) is a serious latency outlier for the Code Factory path -- bounded agentic invocation with retries can be unpredictable.
- YAML specification authoring shifts the burden to the user/planner -- ELAI's autonomous planning loop would need to auto-generate the YAML spec from high-level intent, which the paper lists as future work.

---

ENGINE: mineru
EXTRACTION: C:\Users\user\Downloads\new\paper-2604_extracted\paper-2604.05150.md
