Architectural Violation Detection and Impact Analysis: Graph-Based Structural Validation in Modern Software Systems

Source: Internal research document
Domain: Dependency knowledge graphs, architectural drift detection, change impact analysis, SQLite graph queries
Timeframe: 2025-2026

---

1. Problem Statement

Architectural drift — divergence between intended design and implemented codebase — is mathematically intractable to detect via manual review at scale. Traditional static analysis operates at two extremes: per-file AST linting (fast but blind to system context) and full interprocedural dataflow analysis (deep but computationally prohibitive with state explosion and massive memory overhead).

Solution: Dependency Knowledge Graphs (DKGs) — parse ASTs via tree-sitter, extract structural metadata, store relationships (function call edges, import edges, type reference edges) in SQLite. Evaluate architectural constraints through graph reachability and topological querying without tracking variable state.

---

2. Cross-File Violation Detection Without Dataflow Analysis

2.1 Reverse Reachability for Un-Updated Callers

When a function definition's AST node is modified, reverse traverse call edges to find all callers. Compute abstract syntax hash of callers before/after commit. If target function hash changes but caller hash is identical -> structural inconsistency (caller neglected during refactoring).

2.2 Topological Chokepoints and Gatekeeper Evasion

Security requirement: "All paths to sensitive module X must pass through authorization module Y."

Modeled as dominator tree problem: node Y must dominate node X in the call graph. If a developer removes the authorization check, path-finding from any public entry point to sensitive sink discovers a route bypassing gatekeeper. Detectable instantly via graph topology without tracing a single variable.

2.3 Layer Isolation and DAG Constraints

Tag nodes by file paths/namespaces (e.g., all AST nodes in src/presentation/ tagged Presentation layer). Rule engine executes anti-join: if edge from Domain -> Presentation exists, layer violation triggered. Cycle detection via Tarjan's strongly connected components identifies mutual dependency tangles violating DAG topology.

2.4 ML Enhancement on Graph Topology

For legacy codebases without documented architectural contracts: Graph Neural Networks (GNNs) and graph-embedding anomaly detectors learn acceptable dependency clustering, indegree/outdegree density, directionality from existing graph. New edges bridging traditionally isolated clusters flagged as statistical architectural anomalies. Uses same underlying DKG data.

---

3. Graph Query Patterns: Industry Tool Analysis

3.1 ArchUnit (Java)

Fluent bytecode analysis. Isolation: noClasses().that().resideInAPackage("..presentation..").should().dependOnClassesThat().resideInAPackage("..persistence..")

Chokepoint: layeredArchitecture().whereLayer("Persistence").mayOnlyBeAccessedByLayers("Service"). At class level: classes in X should onlyHaveDependentClassesThat().resideInAnyPackage("..y..", "..x..")

3.2 dependency-cruiser (JavaScript/TypeScript)

Regex-driven JSON configuration. Isolation via forbidden array with from/to path regex blocks. Transitive reachability: reachable: true evaluates indirect "via via" relationships. reachable: false identifies dead wood (modules unreachable from entry points).

3.3 cargo-deny (Rust)

Macro-architecture over third-party crate dependency graph via deny.toml. Bans: deny = [{ name = "openssl-sys" }]. Graph shaping via skip-tree to mask acceptable violations in foundational library subtrees. Supply chain graph sanitization.

3.4 Semgrep Pro

interfile: true enables cross-file interprocedural analysis. Taint tracking paradigm: pattern-sources (entry points) + pattern-sinks (restricted functions). If path from source to sink doesn't traverse designated sanitizer/gatekeeper, violation flagged. Propagates constants and type inferences across files, follows ES6 exports / CommonJS requires.

3.5 Mapping to SQLite

| Tool Paradigm | Goal | SQLite DKG Implementation |
|---|---|---|
| ArchUnit (mayOnlyBeAccessedBy) | Restrict Indegree / Isolate Callers | SELECT target FROM edges WHERE target IN (X) AND source NOT IN (Y) |
| dependency-cruiser (reachable: true) | Transitive Path Prevention | Recursive CTE evaluating path connectivity between restricted node sets |
| cargo-deny ([bans]) | Macro Node Exclusion | SELECT id FROM nodes WHERE package_name IN (Banned_List) |
| Semgrep Pro (interfile Taint) | Cross-file Reachability | Depth-limited BFS CTE tracing call and import edges between Source and Sink |

---

4. Change Impact Analysis (CIA)

4.1 BFS and Proximity Evaluation

Start at modified AST node, expand outward in proximity order. Level-by-level traversal inherently categorizes impact severity: 1-hop = immediate high-probability risk; 3-4 hops = transitive, diluted by abstraction layers. Enables prioritized testing.

4.2 CIRank (Change Impact Rank)

PageRank-derived metric: probability of node being "infected" by change relates to direct connections AND probability of adjacent nodes being infected. Produces probabilistic heat map of blast radius. High CIRank propagation -> mandatory architectural review; isolated changes -> auto-approved.

4.3 Graph Condensation for Scale

Sink-reachability at enterprise scale (millions of nodes): Condensation operators (linear time) group strongly connected components, collapse linear paths. Empirical reduction: up to 99.74% vertices and 99.46% edges while perfectly preserving mathematical proof of reachability. CI/CD pinpoints affected endpoints/tests in milliseconds.

4.4 Matrix Multiplication for Demand-Driven Analysis

Summarize application code and each third-party dependency into distinct matrices. Stitch summaries demand-driven for instantaneous reachability between specific method pairs. Low time/memory overhead for real-time impact scoring during PR phase.

---

5. SQLite Performance for <10ms Architectural Checks

5.1 Recursive CTEs

SQLite natively supports hierarchical graph traversal. Constraints for performance:

Depth Limitations: Inject depth counter in recursive step (WHERE depth < 5) to prevent runaway recursion.

Cycle Detection: Maintain materialized path string (/NodeA/NodeB/NodeC/) during traversal. INSTR() function verifies target node doesn't already exist in path. Match found -> branch recursion halts.

5.2 NGQP Optimization

Next Generation Query Planner uses polynomial-time graph algorithm for optimal loop nesting and index utilization in multi-way joins.

Required: Covering B-Tree index on (source, target, edge_type). Enables index-only scans — VDBE resolves queries without loading data pages from main table.

Benchmark: 180,000+ node scans with complex WHERE filtering per second. Typical architectural validations easily within 10ms.

5.3 Pragmatic <10ms Checks

- Dead code detection: SELECT target FROM nodes WHERE reachable = 0 (instant)
- Layer enforcement: Anti-join on edges by namespace (fractions of ms, no recursion needed)
- Precision test execution: Reverse traversal from modified function to nearest test-suite node; execute only tests mathematically proven to intersect with change

---

6. Cargo Scan's Effects Model

6.1 Rust Effects Taxonomy

Three effect categories propagated upward through call graph:

| Category | Scope | Examples |
|---|---|---|
| Unsafe Effects | Bypass Rust memory safety | unsafe blocks, raw pointer deref, unsafe trait functions |
| System Effects | External environment interaction | Filesystem access, network I/O, OS syscalls |
| Higher-Order Effects | Dynamic control flow | Function pointers, closures, dynamic trait polymorphism |

Effects tagged at AST level, associated with call graph nodes, pushed up reverse-dependency chain via taint/effect analysis. Auditor instantly determines that a benign public API eventually triggers "Filesystem Write" three layers deep in a third-party crate — without tracking the exact file path variable.

Empirical result: Reduced manual auditing burden to median 0.2% of total lines of code.

6.2 Generalizing via DKGs

Tag functions during DKG insertion: open() -> Effect: Disk_IO; .query() on DB connector -> Effect: DB_Write.

Compliance query "Does any public API surface expose unauthenticated database write?": Reachability check from nodes tagged API_Router to nodes tagged Effect: DB_Write. If path exists without node tagged Auth_Middleware -> architectural violation.

Provides 80% of security/architectural insights of proprietary dataflow engines at fraction of cost.

---

7. Efficacy: Per-File vs Cross-File vs LLM Auditors

7.1 Per-File Linting Limitations

Per-file analyzers (ESLint, PMD, file-level SonarQube) miss 47-80% of critical vulnerabilities. Fundamental flaw: modern software is modularized, bugs manifest in component interactions (missing validation upstream + SQL execution downstream), not within single files.

7.2 Cross-File Graph Analysis

Significantly increases true positive detection by mapping relational pathways. Reduces false positives by up to 25% vs per-file linters (can verify whether dangerous function is actually reachable from untrusted input). Pre-computed DKG in SQLite eliminates traditional cross-file overhead (no recompilation/AST rebuild).

7.3 LLM Auditor Flaws

- Context window degradation: Cannot hold 100K-file dependency graph in working memory; forced to guess module interactions -> hallucinations about architecture
- Structural inconsistency: Without explicit exposure to structural patterns, approves PRs with unresolved references, cyclic dependencies, fragmented logic
- Cost/latency/non-determinism: Minutes instead of milliseconds per PR; high token costs; violation flagged Tuesday may be missed Wednesday

7.4 The Hybrid Architecture (Recommended)

Phase 1 — Deterministic Graph Pruning (DKG): SQLite engine handles all structural/topological constraints in <10ms. Catches circular dependencies, layer violations, unreachable code, basic effect propagation. Calculates precise blast radius.

Phase 2 — Semantic Context Injection: DKG extracts only the relevant localized subgraph (exact execution path source -> sink), ignoring rest of repository.

Phase 3 — LLM as Judge: Concise deterministic subgraph passed to LLM. Structural context is perfect and bounded -> LLM avoids topological hallucinations, focuses on semantic intent evaluation.

---

8. Key Takeaways

1. DKGs stored in SQLite bridge the gap between per-file linting (fast but blind) and full dataflow analysis (deep but prohibitive) — graph reachability provides cross-file validation in <10ms
2. Per-file rules miss 47-80% of critical vulnerabilities; cross-file graph analysis is mandatory for modern codebases
3. LLM auditors hallucinate architecture when given fragmented cross-file context; they must receive pre-computed structural subgraphs, not raw code
4. The hybrid DKG + LLM pipeline is strictly superior: deterministic graph catches topology violations, LLM judges semantic intent on the precise blast radius
5. Cargo Scan's effects taxonomy generalizes beyond Rust via DKG tagging, providing 80% of dataflow engine insights at negligible cost
6. Covering B-Tree index on (source, target, edge_type) is the critical SQLite optimization for sub-10ms query performance
7. Graph condensation achieves 99.74% vertex reduction while preserving reachability proofs — essential for enterprise-scale CI/CD
