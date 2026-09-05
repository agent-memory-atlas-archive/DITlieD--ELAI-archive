# C-50: Contract-Coding Test Scaffolding Pattern

Objective: integrate the Contract-Coding pattern (arXiv:2604.13100) as a
greenfield project-init template: when ELAI initializes a fresh project,
emit precondition/postcondition contracts BEFORE implementation. Pattern
demonstrated 47% functional success on Greenfield-5 with near-perfect
structural integrity.

Pre-requisites: Phase 4F code completeness (FILE_PLAN + skeleton + MiniCritic
+ StructuralRetry) SHIPPED.
Parallel-safe with: C-35..C-49, C-51..C-53.

Paper grounding (arXiv:2604.13100, 2026-04-10, verified via prior oracle
2026-05-17): "Contract-Coding" pattern. Per prior oracle: "47% functional
success while maintaining near-perfect structural integrity" on the
Greenfield-5 benchmark. Open-source. MODERATE complexity.

## Context

ELAI's C-31/C-32 code-completeness layer enforces FILE_PLAN + skeleton-first.
Contract-Coding extends the pattern with PRE/POST CONDITION generation:
- For each function the FILE_PLAN declares, emit preconditions (input
  constraints) and postconditions (return guarantees) BEFORE writing
  the body.
- The MiniCritic checks body against these conditions, not just structure.
- StructuralRetry chain extends to ContractViolationRetry.

47% on Greenfield-5 is not high in absolute terms but is structurally
near-perfect — meaning the failed 53% are SEMANTIC failures (got the
wrong answer) not STRUCTURAL failures (broken syntax, wrong file shape).
For ELAI's deterministic-first philosophy, this is the right
optimization direction.

## Reads

- F:\.ELAI_workflow\docs\thefinalglance\C-plan\C-31_code_completeness_file_plan.md
- F:\.ELAI_workflow\docs\thefinalglance\C-plan\C-32_code_completeness_skeleton_first.md
- .elai_cc/crates/code_completeness/ (Phase 4F)
- workflow/papers/contract-coding-interface.md (paper-ingestor + mode=source, Task 1)

## Tasks

1. **Source-grounding** — paper-ingestor (mode=paper + mode=source) against
   arXiv:2604.13100 and its open-source reference.

2. **ContractSpec struct (crates/code_completeness/src/contracts.rs)** —
   per-function: preconditions: Vec<Predicate>, postconditions: Vec<Predicate>,
   contract_language: ContractLang (Rust assert!, Python assert,
   TS predicate). Predicates are STRING expressions in the target language,
   not abstract; validated by parsing through tree-sitter at emission.

3. **ContractEmitter** — runs after FILE_PLAN, before skeleton. For each
   planned function, prompts the model to emit preconditions + postconditions.
   Output validated for syntactic-parsability in target language. Stored
   alongside skeleton in .elai/contracts/{file_path}.toml.

4. **MiniCritic extension** — ContractViolationCheck runs at body-completion
   time: parses body AST, checks that asserted preconditions hold on
   entry paths and postconditions hold on return paths. Failures emit
   ContractViolationRetry into the existing retry chain.

5. **Greenfield-init template** — when `elai new project` runs, the
   project template ENABLES contracts by default. Existing projects opt
   in via elai_cc.toml [code_completeness.contracts.enabled].

6. **SSB action types** — contract_emitted, contract_violation_detected,
   contract_retry_resolved.

7. **Tests first (TDD)**:
   - 4F.X.3: ContractSpec round-trips via toml
   - 4F.X.4: Predicates parse via tree-sitter in target language
   - 4F.X.5: ContractViolationCheck flags a deliberately-violated body
   - 4F.X.6: ContractViolationRetry resolves on retry with correct body
   - 4F.X.7: Greenfield-5 fixture pilot: produce 47% functional success
     (target match), 95%+ structural success (target match)

8. **Contract tests** — tests/contracts/contract_emitter_contract.rs.

## WIRING_CONTRACT

produces_symbols:
  - ContractSpec, Predicate, ContractLang (crates/code_completeness/src/contracts.rs)
  - ContractEmitter, ContractViolationCheck (same module)
  - .elai/contracts/{file_path}.toml per-project artifacts
  - SSB action_types: contract_emitted, contract_violation_detected,
    contract_retry_resolved
consumes_symbols:
  - FILE_PLAN output (C-31)
  - skeleton output (C-32)
  - MiniCritic + StructuralRetry chain
  - tree-sitter (knowledge crate)
reachable_from:
  - elai new project (template enables)
  - elai_cc.toml [code_completeness.contracts.enabled]
stub_ban:
  - contract emission MUST validate-parse the predicate before storing
  - ContractViolationCheck MUST emit SSB before triggering retry
  - target-language predicate validation MUST NOT fall back to
    "permissive" mode (parse fail = contract reject)

## DETERMINISTIC_CHECK

```bash
test -f .elai_cc/crates/code_completeness/src/contracts.rs
rg "ContractSpec|ContractEmitter|ContractViolationCheck" .elai_cc/crates/code_completeness/src
rg "contract_emitted|contract_violation_detected|contract_retry_resolved" .elai_cc/crates/orchestrator/src
cargo test -p elai_code_completeness -- contracts 2>&1 | grep "test result.*ok"
test -f .elai_cc/crates/code_completeness/tests/greenfield5_fixture
```

## ACCEPTANCE

- [ ] paper-ingestor produces workflow/papers/contract-coding-interface.md
- [ ] ContractSpec implemented with target-language predicates
- [ ] ContractEmitter runs post-FILE_PLAN, pre-skeleton
- [ ] ContractViolationCheck wired into MiniCritic
- [ ] ContractViolationRetry extends the existing retry chain
- [ ] Greenfield-init template enables contracts by default
- [ ] All 3 SSB action types emit
- [ ] TDD tests pass
- [ ] Greenfield-5 pilot reproduces ≥40% functional success + ≥90%
      structural success (conservative vs paper's 47%/near-perfect)
- [ ] Contract tests pass

## Files

- ADD: .elai_cc/crates/code_completeness/src/contracts.rs
- ADD: .elai_cc/crates/code_completeness/src/contract_emitter.rs
- ADD: .elai_cc/crates/code_completeness/tests/contracts_tdd.rs
- ADD: .elai_cc/crates/code_completeness/tests/greenfield5_fixture/* (paper fixture)
- ADD: .elai_cc/crates/code_completeness/tests/contracts/contract_emitter_contract.rs
- ADD: workflow/papers/contract-coding-interface.md
- MODIFY: .elai_cc/crates/code_completeness/src/mini_critic.rs (ContractViolationCheck)
- MODIFY: .elai_cc/crates/code_completeness/src/retry.rs (ContractViolationRetry variant)
- MODIFY: .elai_cc/crates/orchestrator/src/ssb/schema.rs (3 new action types)
- MODIFY: .elai_cc/templates/new_project/elai_cc.toml ([code_completeness.contracts] default)

## Next

C-50 closure raises ELAI's greenfield baseline. Pairs with C-31/C-32: FILE_PLAN
declares functions, contracts declare their behavior, skeleton emits bodies
constrained by contracts. Three layers of completeness pressure.
