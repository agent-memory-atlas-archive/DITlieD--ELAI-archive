Source: Contract-Coding reference implementation
Repo: https://github.com/imliinyi/Contract-Coding (MIT license, 71 commits, main branch)
Ingested: 2026-06-24 via webfetch of raw source files
Reliability: HIGH (official repo, reviewed source — not decompiled)

================================================================
REPO STRUCTURE
================================================================
ContractCoding/
  config.py          — pydantic BaseModel config (model backend, workspace, limits)
  agents/
    base.py          — BaseAgent abstract class (run method)
    agent.py         — agent implementations
    forge.py         — AgentForge factory (create_agent + AgentCapability)
  llm/               — LLM client (OpenAI API or Codex CLI subprocess)
  memory/
    contract.py      — Contract kernel data model + markdown parser
    audit.py         — Contract interface auditor (AST-based)
    document.py      — DocumentManager (shared collaborative document)
    processor.py     — MemoryProcessor (conversation window)
  orchestration/
    engine.py        — Engine: registers agents, runs traversal, post-run audit
    runner.py        — AgentRunner: executes individual agents
    traverser.py     — GraphTraverser: HEG layer-based parallel scheduling
  prompts/
    system_prompt.py — core system prompt (document_action protocol)
    agents_prompt.py — per-agent prompts (Project_Manager, Critic, etc.)
  tools/             — file tools, document_action tool
  utils/
    log.py           — logger
    state.py         — GeneralState (pydantic model)
main.py              — CLI entry: --task, --workspace, --max-layers
requirements.txt     — langgraph, pydantic, openai, etc.

================================================================
CONTRACT DATA MODEL (memory/contract.py)
================================================================

ContractKernel:
  files: list[ContractFile]
  dependencies: dict[str, list[str]]   # path -> [dep_path, ...]
  by_path() -> dict[str, ContractFile]

ContractFile:
  path: str                            # normalized (no workspace/ prefix, no ./)
  owner: str                           # agent name (spaces -> underscores)
  status: ContractStatus               # TODO | IN_PROGRESS | DONE | ERROR | VERIFIED
  version: int                         # starts at 1, increments per sub-task
  classes: list[ContractClass]
  functions: list[ContractFunction]
  block: str                           # raw markdown block text

ContractClass:
  name: str
  attributes: list[ContractAttribute]  # name, type, description
  methods: list[ContractFunction]

ContractFunction:
  name: str
  params: list[ContractParam]          # name, type (optional)
  returns: str | None                  # return type annotation
  signature: str                       # raw "def name(params) -> returns"

ContractStatus = Literal["TODO", "IN_PROGRESS", "DONE", "ERROR", "VERIFIED"]
VALID_STATUSES = {"TODO", "IN_PROGRESS", "DONE", "ERROR", "VERIFIED"}

CRITICAL: The contract is STRUCTURAL — file paths, class/method names, parameter
names, type annotations, return types, ownership, dependencies. There are NO
preconditions, NO postconditions, NO behavioral assertions, NO Hoare triples.
The "contract" is a symbolic API specification (namespace + signatures), not a
Design-by-Contract behavioral specification.

================================================================
CONTRACT PARSING (memory/contract.py::parse_contract_kernel)
================================================================

Input: markdown string (the "Collaborative Document")
Output: ContractKernel or raises ContractParseError(issues)

Parser is REGEX-BASED (not AST-based for the contract itself):
1. Extract "### Symbolic API Specifications" section from markdown
2. Find all "**File:** `path`" blocks via regex
3. For each file block, extract:
   - Owner: regex match "**Owner:** value"
   - Status: regex match "**Status:** value" (must be in VALID_STATUSES)
   - Version: regex match "**Version:** N" (integer extraction)
4. Parse symbols within each file block:
   - Classes: "**Class:** `name`" or "Class Name `name`"
   - Attributes: lines under "**Attributes:**" section
   - Methods/Functions: Python-style "def name(params) -> returns" signatures
     extracted from backticks, quotes, or plain text via regex
5. Parse dependencies from "### Dependency Relationships" section:
   - Extract file paths from backticks
   - "depends on" / "requires" / "->" / "-->" keywords indicate edges

Validation (raises ContractParseError on failure):
- Missing file path, owner, status, or version → ContractParseIssue
- Invalid status string → ContractParseIssue
- Invalid version (non-numeric) → ContractParseIssue
- Any issue for a file → that file is dropped from the kernel

Path normalization:
- Strip backticks, strip "./" prefix, strip "workspace/" prefix
- Replace backslashes with forward slashes
- Strip leading/trailing slashes

================================================================
CONTRACT AUDIT (memory/audit.py::audit_contract_interfaces)
================================================================

Input: ContractKernel, workspace_path (str)
Output: list[AuditIssue]

AuditIssue:
  path: str
  severity: "error" | "warning"
  kind: str (missing_file | syntax_error | placeholder_text | placeholder_pass |
             missing_class | missing_method | missing_function |
             parameter_mismatch | parameter_type_mismatch |
             return_type_mismatch | missing_return_annotation)
  message: str
  expected: str | None
  actual: str | None

Audit flow per contract file:
1. Resolve file path in workspace (walk directory tree)
2. If file missing → AuditIssue(error, missing_file)
3. If not .py file → skip (only Python is audited)
4. Read source, ast.parse(source)
   - SyntaxError → AuditIssue(error, syntax_error)
5. _audit_placeholder_logic:
   - Source contains "todo" or "placeholder" (case-insensitive) → error
   - AST contains ast.Pass node → error (placeholder_pass)
6. _audit_python_symbols:
   - For each contract-declared class: check ast.ClassDef exists
     - Missing → AuditIssue(error, missing_class)
     - For each declared method: check ast.FunctionDef exists in class body
       - Missing → AuditIssue(error, missing_method)
       - Present → _compare_function (param names, param types, return types)
   - For each contract-declared function: check ast.FunctionDef exists at module level
     - Missing → AuditIssue(error, missing_function)
     - Present → _compare_function

_compare_function details:
- Strip self/cls from actual params if contract doesn't declare them
- Compare parameter NAMES (order-sensitive): mismatch → parameter_mismatch (early return)
- Compare parameter TYPES (normalized: strip quotes, remove whitespace): mismatch → parameter_type_mismatch
- Compare return types: mismatch → return_type_mismatch
- Expected return but actual missing → missing_return_annotation (warning)

Type normalization: strip quotes/whitespace, return None for empty.

Also audited (separate functions, called from Engine.run):
- audit_file_existence: checks all documented files exist in workspace
- audit_file_versions: checks "# version: N" first line of each file matches contract version

================================================================
HEG TRAVERSAL (orchestration/traverser.py::GraphTraverser)
================================================================

Layer-based parallel execution (ThreadPoolExecutor, max_workers=16):

Layer 0: Project_Manager
  - Generates the Collaborative Document (contract) from user intent
  - Document must contain: PRD, Directory Structure, Dependency Relationships,
    Symbolic API Specifications (file blocks with Owner/Version/Status)

Layer 1: Architect (contract review, NO implementation)
  - _schedule_architect_or_contract_repair:
    - Validates document structure (check_missing_specs, parse_contract_kernel)
    - If invalid → Project_Manager gets error message to fix document
    - If valid → Architect reviews/repairs contract, ensures consistency
    - Does NOT set any status to VERIFIED or DONE

Layer 2+: _schedule_from_contract
  - Parse kernel from document
  - If parse error → Project_Manager gets error to fix contract fields
  - If no files → Project_Manager gets "add file blocks" message
  - If missing specs → Project_Manager gets "add detailed file blocks" message
  - Run audit_contract_interfaces(kernel, workspace)
  - For each task in kernel.files:
    - Skip if VERIFIED
    - If DONE → add to done_tasks (schedule Critic + Code_Reviewer for review)
    - If TODO/IN_PROGRESS/ERROR:
      - Check dependencies (all deps must be VERIFIED and known)
      - If blocked → add to blocked_tasks
      - If unblocked → schedule to task.owner with contract description + issues
  - done_tasks → Critic + Code_Reviewer get batch review message
  - blocked_tasks with no other work → Project_Manager gets dependency fix message

Termination:
- When no pending tasks (all VERIFIED) → terminating_states
- When MAX_LAYERS exceeded → log error, build failure report
- When stalled (pending tasks but no agents scheduled) → Project_Manager gets critical message

================================================================
RETRY MECHANISM (the "ContractViolationRetry" equivalent)
================================================================

The reference implementation does NOT have an explicit "retry" class. Instead,
retry is implicit in the status model + scheduling loop:

1. Worker implements file → sets Status to DONE via document_action
2. Critic/Code_Reviewer review DONE files:
   - If correct → Status = VERIFIED (terminal, no more scheduling)
   - If incorrect → Status = ERROR + append issue bullets after Status line
3. Next layer: _schedule_from_contract sees ERROR status tasks:
   - Extracts issue text from file block (lines after "**Status**" line)
   - Builds worker message: "Fix {path}. Current Status: ERROR. ..."
   - Includes contract description + issue text
   - Schedules to task.owner
4. Worker fixes file → sets Status to DONE again
5. Loop continues until VERIFIED or MAX_LAYERS

The audit_contract_interfaces function provides DETERMINISTIC issue detection
(placeholder detection, missing symbols, signature mismatches) that supplements
the LLM-based Critic review. The traverser passes these audit issues to the
Critic/Code_Reviewer in the review message.

================================================================
DOCUMENT ACTION PROTOCOL (prompts/system_prompt.py)
================================================================

Agents communicate via a shared "Collaborative Document" using <document_action>:
- type: "update|add" (section-patch mode preferred)
- section: one of [Project Overview, User Stories, Constraints, Directory Structure,
  Global Shared Knowledge, Dependency Relationships, Symbolic API Specifications]
- content: markdown string (FULL REPLACEMENT of section body)

The document is the SINGLE SOURCE OF TRUTH. All agents read/write to it.
The contract kernel is parsed FROM this document, not from separate files.

================================================================
KEY INSIGHTS FOR C-50 INTEGRATION
================================================================

1. STRUCTURAL NOT BEHAVIORAL: The reference contract is file paths + class/method
   signatures + types + ownership + dependencies. C-50's plan says "precondition/
   postcondition contracts" — the reference does NOT implement these. C-50 must
   either:
   a) Rescope to structural contracts (matching the paper), OR
   b) Add behavioral contract generation as an ELAI extension beyond the paper

2. CONTRACT IS MARKDOWN: The contract lives in a markdown document parsed via
   regex. C-50's plan says ".elai/contracts/{file_path}.toml" — this is an ELAI
   design choice, not from the reference. The reference uses a single shared
   document, not per-file contract files.

3. AUDIT IS AST-BASED (Python only): audit_contract_interfaces uses Python's ast
   module. C-50 needs tree-sitter for multi-language support (Rust/Python/TS).
   The reference only audits .py files; non-Python files skip interface audit.

4. RETRY IS IMPLICIT: No explicit retry class. Retry = ERROR status + issue text
   + re-schedule to owner. C-50's "ContractViolationRetry extends the existing
   retry chain" should map to: contract violation → set ERROR → append issues →
   re-schedule through the existing StructuralRetry mechanism.

5. ONE-PASS AUDIT: The paper's "Discriminator Agent" is the Critic + Code_Reviewer
   + audit_contract_interfaces. The ablation (w/o Contract drops 100%→65%) is
   about removing the ENTIRE contract system, not just the audit. The audit is
   deterministic (AST-based); the Critic is LLM-based.

6. PARALLELISM: HEG = topological scheduling based on dependency graph. Tasks
   with unmet dependencies are blocked. Tasks with verified dependencies are
   scheduled in parallel via ThreadPoolExecutor. C-50 can reuse ELAI's existing
   BSP supersteps / JoinSet spawn_parallel for this.

7. NO SEPARATE CONTRACT EMISSION STEP: In the reference, the Project_Manager
   generates the contract AS PART OF the initial document. There is no separate
   "ContractEmitter" phase between FILE_PLAN and skeleton. C-50's plan to add
   a ContractEmitter post-FILE_PLAN pre-skeleton is an ELAI extension.

8. VERSION TRACKING: Each file has a version integer (starts at 1, increments
   per sub-task). The audit checks "# version: N" as the first line of each
   file. This is a simple but effective change-tracking mechanism.

================================================================
CRITICAL DISCREPANCY WITH C-50 PLAN
================================================================

C-50 plan says: "emit precondition/postcondition contracts BEFORE implementation"
and "Predicates are STRING expressions in the target language, not abstract;
validated by parsing through tree-sitter at emission."

The reference implementation does NOT do this. The contract is:
- File path + owner + status + version
- Class names + attribute names/types + method signatures
- Function signatures (name, params, return type)
- Dependency relationships

There are NO preconditions (input constraints) or postconditions (return
guarantees) in the reference. The "contract" is a structural API specification,
not a behavioral contract in the Design-by-Contract sense.

C-50 has two options:
A) RESCOPE: Implement structural contracts (signatures + file ownership +
   dependencies) matching the paper's actual approach. The audit checks
   signature conformance + placeholder detection. This is what the paper
   actually demonstrates.
B) EXTEND: Add behavioral pre/postcondition generation as an ELAI extension
   beyond the paper. This requires separate grounding (e.g., Daikon-family
   invariant inference, or LLM-generated JML/Eiffel-style assertions).

The existing paper interface document (contract-coding-interface.md) already
flags this: "Pre/postcondition emitter status: UNSUPPORTED by paper."

RECOMMENDATION: Option A (rescope to structural contracts) is grounded in
both the paper and the reference implementation. Option B requires additional
grounding sources. The 47% functional success + near-perfect structural
integrity result is from structural contracts, not behavioral ones.
