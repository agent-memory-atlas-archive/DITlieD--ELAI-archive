Deterministic Code Analysis and Rule Enforcement via Abstract Syntax Trees: A Comprehensive 2026 Evaluation

Source: Internal research document
Domain: Tree-sitter, AST-based linting, static analysis, rule engine architecture
Timeframe: 2025-2026

---

1. Problem Statement

Regex-based linting cannot comprehend nested hierarchical language structure — under-matches valid code, over-matches keywords in strings/comments. The modern paradigm uses Abstract Syntax Trees (ASTs) and Concrete Syntax Trees (CSTs) via Tree-sitter's incremental GLR parsing algorithm.

Goal: Deterministic rule enforcement engine evaluating structural patterns in <1ms per rule across Rust, Python, TypeScript, and Go. Rules stored in SQLite, compiled at runtime. The <1ms budget dictates exclusive reliance on syntactic pattern matching — no deep semantic analysis.

---

2. Tree-Sitter Query Language

2.1 Syntactic Capabilities

- Parenthesized patterns declaring node type + expected children
- Captures (@symbol): Extract isolated tokens from structural matches (e.g., @function.name)
- Field identifiers: Semantic roles for children (left:, right:) — precise targeting without fragile child-index ordering
- Quantifiers: *, +, ? applied to AST nodes (variadic params, repeated decorators, arbitrary statement sequences)
- Negation (!): Assert explicit absence of a field (e.g., !type_parameters for class without generics)
- Alternation ([]): Match one of several node types
- Predicates: #eq?, #match? — string equality and regex matching on captured nodes after structural match

2.2 Fundamental Limitations

Temporal Ordering Constraints:
Tree-sitter queries operate on lexical scope and tree hierarchy, NOT execution precedence. Can assert both function A and function C exist in same block, and C appears before A in source text (via sibling anchors). But lexical precedence != execution precedence. If C is inside an if_statement evaluating to false, or early return exists between C and A, structural query still matches. No CFG = blind to runtime execution paths. Result: false positives and false negatives whenever non-linear control flow exists.

Cross-Function Invariants:
Tree-sitter parses files into stateless isolated hierarchical trees. No symbol tables, no identifier scoping, no call graphs. Call site "validateAuth" treated as independent identifier node disconnected from its function_definition. Aliasing (const check = validateAuth; check()) defeats literal string matching. Cross-function invariants are mathematically impossible via isolated CST queries.

Data Flow and Taint Tracking:
Requires CFG, SSA form, and taint propagation algorithms. Async/await boundaries cause lexical AST to diverge from chronological runtime execution (microtask queue). Taint through callbacks/array methods (.map, .reduce) unfeasible without massive over-approximation. Re-exports and barrel files require cross-file resolution outside Tree-sitter's single-file design. Phi nodes at branch merges need program memory state. Raw Tree-sitter is structurally blind to data flow.

---

3. Tool Architectures

3.1 Semgrep: Generic AST (IL)

Pipeline: Source code -> Tree-sitter CST -> OCaml bridging (ocaml-tree-sitter-semgrep) -> unified language-agnostic Generic AST (Intermediate Language) -> YAML rule matching with metavariables ($X) and ellipsis operator (...).

Multi-language: Single rule targets identical constructs across languages via normalization. Enables broad cross-language taint tracking.

Tradeoffs: Inherently lossy — language-specific idiosyncrasies smoothed over during CST-to-Generic translation. Sacrifices ability to lint highly language-specific syntax that doesn't map to generic IL. Translation overhead necessitated 2025 multicore engine overhaul for large monorepo responsiveness.

3.2 ast-grep: Isomorphic CST Pattern Matching

Written entirely in Rust. Operates directly on native Tree-sitter CST — NO intermediate generic representation. Uses ignore crate for parallel file discovery across all CPU cores.

Core mechanism: Isomorphic pattern matching. Rules written in target language itself (e.g., Promise.all($A)). Parsed using same Tree-sitter grammar as target code. Highly optimized sub-graph isomorphism algorithm finds structural matches.

Multi-language: Single rule CANNOT span multiple languages. JS and TS produce structurally distinct CSTs. Rules must be explicitly language-specific. Mixed-language contexts (CSS in JS template literals) handled via Tree-sitter language injection.

3.3 difftastic: S-Expression DAG Shortest-Path

Uses Tree-sitter for structural differencing, not linting. Translates CST into simplified uniform S-expression (Lists + Atoms). Models differencing as shortest-path on DAG using Autochrome algorithm. Ignores whitespace/line breaks that don't alter S-expression topology.

Limitation: Scales poorly with highly divergent files (memory/combinatorial explosion). Lossy reduction discards metadata needed for semantic reasoning — cannot be repurposed for deep analysis or automated AST merging.

---

4. Performance Characteristics

4.1 Cold-Start Parse Time (Native Rust, O(N) with file size)

| File Size | Source Size | Parse Time | Throughput |
|---|---|---|---|
| 100 lines | ~3 KB | < 0.2 ms | ~100 MB/s |
| 500 lines | ~15 KB | ~0.6 ms | ~100 MB/s |
| 1,000 lines | ~30 KB | ~1.0 - 2.0 ms | ~100 MB/s |
| 5,000 lines | ~150 KB | ~5.0 - 10.0 ms | ~100 MB/s |

Critical: Cold-start parsing 5,000-line file = 5-10ms. Cannot cold-parse per rule evaluation — would categorically fail <1ms budget.

FFI penalty: Rust-native = fixed memory allocation cost. Node.js/Python wrappers suffer linear deserialization penalties growing with file size. Tree-sitter up to 40x faster than traditional parsers on small files but lags hyper-optimized native toolchains on files >5,000 lines due to FFI overhead. Engine MUST execute rules natively in Rust or Go.

4.2 Incremental Reparsing (Key to <1ms)

| File Size | Incremental Parse (small edit) | Query Execution (incremental) |
|---|---|---|
| 1,000 lines | ~0.1 - 0.2 ms | ~0.5 ms |
| 5,000 lines | ~0.2 - 0.4 ms | ~1.0 - 2.0 ms |

10x-20x speedup over cold parsing. Single statement modification in 5,000-line file: skips thousands of unaffected nodes, reallocates only modified branches, drops parse time to microseconds.

Requirement: Engine must maintain active memory state — hold parsed AST in RAM, incrementally update on filesystem events. Then dozens of structural rules evaluate within 1ms budget.

4.3 Query Complexity Scaling

Standard query throughput: ~200 MB/s, basic matches in <5ms for 1MB codebase.

Danger: Deeply nested alternations ([]) and unbounded quantifiers (*, +) without terminal anchors trigger catastrophic backtracking. In heavily nested syntax (chained generics in Rust, large C++ matrices), unbounded parent-child searches spike from microseconds to hundreds of milliseconds.

Requirement: Compile and optimize rules from SQLite; actively reject/warn against unanchored infinitely quantified patterns threatening the 1ms envelope.

---

5. The Expressiveness Gap

5.1 What AST Rules CAN Do (<1ms)

- Identify deprecated/banned/insecure APIs (eval(), MD5.new())
- Detect hardcoded secrets, IPs, config literals in function arguments
- Enforce syntactic completeness (missing catch blocks, omitted arguments)
- Audit naming conventions, stylistic inconsistencies, localized type misalignments

5.2 What AST Rules CANNOT Do

Defeated by basic indirection: const safeEval = eval; safeEval() — AST searches for literal "eval" at call site and misses. AST rules analyze shape of code as written, NOT behavior as executed.

5.3 Full Static Analysis (Beyond <1ms)

Semgrep Pro: Interprocedural analysis and data flow reachability across file boundaries. Tracks variables through reassignments, aliasing, method chains. Scan times: tens of seconds to minutes.

CodeQL: Compiles entire codebase into relational database (full AST + data flow graph + CFG). QL language (Datalog-inspired) queries pointer aliasing, control flow domination, path sensitivity. Database compilation and query: minutes to hours, up to 450MB memory per run, requires buildable environment.

5.4 Verdict for <1ms Engine

Must strictly rely on CST matching. Cannot dynamically build CFG or execute interprocedural taint tracking. Detection requiring knowledge of variable origination outside immediate lexical scope, or type inference not written in syntax, falls beyond the boundary. Defer to CodeQL, Semgrep Pro, or LLM agents out-of-band.

---

6. Property-Based Testing for AST Rules

6.1 Why PBT Over Example-Based Testing

Traditional example-based tests: Static code snippets + assert rule triggers/doesn't. Insufficient — cannot account for infinite structural variations permitted by grammars. Rules break on formatting/comment changes.

PBT: Define abstract invariants, generate thousands of randomized test cases, ensure property holds universally. proptest crate uses Strategy trait for random generation; on failure, automatically shrinks to minimal reproducible example.

6.2 Mutation Strategies

1. Trivial Mutations (Whitespace, Comments, Indentation):
Tree-sitter CSTs capture whitespace/comments as anonymous nodes. Robust queries use named nodes. Proptest injects random comments, line breaks, tab indentations between tokens. False positive after trivial mutation -> query relies on flawed proximity anchors or exact string matching.

2. Syntactic Equivalences (Variable Renaming):
Randomly rename identifiers across generated code. Ensures rules use metavariables ($X = $X) not hardcoded text. Prevents false negatives from different naming conventions.

3. Logical Permutations:
Swap arithmetic signs (+ to -), flip boolean operators (&& to ||), negate comparisons (> to <=). Ensures rule triggers precisely on defined anti-pattern without catching safe distinct operations.

4. Structural Nesting:
Wrap target anti-pattern inside deeply nested if/while/closures. If rule relies on ^ root anchor instead of unanchored descendant search, proptest immediately fails. Forces spatial flexibility refinement.

---

7. ast-grep YAML DSL vs Raw Tree-Sitter Queries

7.1 Raw S-Expressions

Require intimate knowledge of language-specific grammar node names (function_definition in Python, arrow_function in JS). Fragile across grammars.

7.2 YAML DSL Advantages

Isomorphic Patterns: pattern: console.log($ARG) instead of complex nested S-expression. Engine compiles string into appropriate Tree-sitter query dynamically.

Logical Composition: Relational predicates — all (AND), any (OR), has (descendant), inside (ancestor). Multi-layered constraints difficult/impossible in single S-expression.

Metavariable Filtering: Apply regex constraints to captures, ensure sub-nodes absent — without cluttering structural query.

Escape hatch: Drop to raw Tree-sitter matching when YAML abstractions insufficient.

For SQLite-managed rules: YAML/JSON DSL mapping to isomorphic patterns provides superior developer experience with identical sub-millisecond execution benefits.

---

8. Key Takeaways

1. Tree-sitter's incremental reparsing (microseconds for small edits) is the key to achieving <1ms per rule — engine MUST maintain parsed AST in memory
2. Cold-start parsing is too slow for per-rule evaluation; active memory state with filesystem event-driven incremental updates is mandatory
3. The practical ceiling of AST rules is intra-procedural syntactic matching — defeated by basic aliasing, blind to control flow, cannot track data flow
4. Engine must be native Rust/Go — FFI to Node.js/Python introduces linear deserialization penalties that break latency budget
5. Unanchored unbounded quantifiers in queries cause catastrophic backtracking; SQLite-stored rules must be validated/optimized at compile time
6. ast-grep's isomorphic pattern matching (rules in target language syntax) with YAML DSL is the optimal ergonomic layer for rule authoring
7. PBT with four mutation strategies (trivial, rename, logical permutation, structural nesting) mathematically proves rule determinism
8. Semantic analysis (cross-function invariants, taint tracking, temporal ordering) must be delegated to Semgrep Pro/CodeQL/LLM agents operating out-of-band
