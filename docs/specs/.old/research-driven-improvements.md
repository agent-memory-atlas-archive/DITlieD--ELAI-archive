RESEARCH-DRIVEN SPEC IMPROVEMENTS
Date: 2026-04-08
Status: AUDITED (6 BLOCKs found, all resolved below)
Audit: docs/specs/research-improvements-audit.md
Context: Phases 0 through 2D.3 are DONE. Changes target Phase 2D.4+ onwards.

Sources:
  llm-json-schema-generation.md         — constrained decoding benchmarks and failure modes
  treesitter-ast-rule-engine.md          — tree-sitter performance, expressiveness ceiling, ast-grep
  auto-generated-linting-rules.md        — self-improving rule lifecycle, causal validation, decay
  architectural-rule-engine-vs-llm-auditor.md — cross-file analysis, DKG graph queries, Cargo Scan
  deterministic-simulation-testing.md    — madsim, HashMap, VCR pattern, SQLite VFS
  ic3-evolve-summary.md                 — proof-gated self-modification, slot-restricted patches

Bonus context:
  Automated Code Analysis Pipeline Brief.txt — two-phase architecture validation


===============================================================================
CRITICAL CHANGES (must apply before reaching affected phase)
===============================================================================

C1. CONSTRAINED DECODING PER-MODEL STRATEGY
    File: docs/specs/crates/providers.txt (section 3, LayerConfig)
    Phase affected: 3F (Tier 2 rule enforcement)

    Problem:
      The plan specifies DeepSeek-R1-Distill as the auditor model with
      "constrained output format" (providers.txt line 42). Research proves
      that R1-Distill models require <think>...</think> tags for reasoning.
      Setting format=json in ollama applies GBNF from the first token,
      clamping <think to zero probability. The model's reasoning capability
      is lobotomized — it must zero-shot the JSON without intermediate
      computation. Factual accuracy drops catastrophically.
      Source: llm-json-schema-generation.md section 2.3

    Current spec (providers.txt:42):
      "Auditor: low temperature (0.1-0.3), constrained output format."

    Change:
      Replace with per-model enforcement strategy:

      Tier 2 enforcement mode (new field in elai_cc.toml [agents.{role}.tier2_mode]):
        GRAMMAR:    GBNF/grammar constraints applied at generation time.
                    Use for: Gemma 4 (natively robust, has delimiter tokens).
                    NOT for: DeepSeek-R1-Distill, any model requiring CoT traces.
        INSTRUCTOR: Free generation + post-hoc JSON schema validation + retry.
                    Model generates freely (including <think> tags for R1).
                    Output parsed for JSON. If invalid: inject parsing error,
                    re-prompt (max 3 retries). Preserves full reasoning capability.
                    Use for: DeepSeek-R1-Distill, reasoning-heavy models.
        AUTO:       Select based on model metadata. Models with requires_cot=true
                    in model registry get INSTRUCTOR. Others get GRAMMAR.

      Default per role:
        executor (Gemma 4):       GRAMMAR
        auditor (R1-Distill):     INSTRUCTOR
        designer (Gemma 4):       GRAMMAR
        prompt-engineer (Qwen):   GRAMMAR (Qwen has native structured output)

    Evidence:
      "Using SGLang/vLLM JSON enforcement on R1 without complex multi-step
       prompting consistently produces semantic garbage." (source section 7)
      "Gemma 4: highly robust against zero-shot hallucination spikes...
       reliable high-speed structured extraction." (source section 7)
      RTX 3060 constrained decoding: <9 OTPS (source section 5, table)

    Additional constraint for GRAMMAR mode:
      All SCHEMA_REQUIRED predicates must be flattened to max 3 nesting levels.
      Recursive JSON Schema definitions banned (SIGSEGV on deep recursion
      in ollama GBNF engine). Predicate validation (rules.txt section 3A)
      must reject schemas with >3 nesting levels or $ref cycles.
      Source: llm-json-schema-generation.md section 2.2


C2. CROSS-FILE GRAPH PREDICATES IN RULES ENGINE
    File: docs/specs/crates/rules.txt (section 3, Predicate DSL)
    Phase affected: 3F (rules engine)

    Problem:
      All 5 current predicate types (AST_PATTERN, GREP_PATTERN, SCHEMA_REQUIRED,
      METRIC_GATE, COMPOSITE) operate on single files. Research shows per-file
      rules miss 47-80% of critical vulnerabilities (source: architectural-
      rule-engine.md section 7.1). The DKG already exists (Phase 1A) with
      call edges, import edges, type reference edges in SQLite. But the rules
      engine cannot query it.

    Change:
      Add 6th predicate type: GRAPH_REACHABILITY

      GRAPH_REACHABILITY:
        Evaluates graph queries on the DKG (knowledge/ crate's SQLite tables).
        Fields:
          source_pattern: glob or tag matching source nodes (e.g., "src/api/**")
          sink_pattern: glob or tag matching sink nodes (e.g., "src/db/**")
          must_traverse: node tag that must appear on every path (e.g., "auth_middleware")
          must_not_exist: if true, assert no path exists (layer isolation)
          max_depth: depth limit for recursive CTE (default 5, max 7)
          edge_types: which edge types to follow (call, import, type_ref)

        Query types (all implementable as SQLite recursive CTEs in <10ms):
          CHOKEPOINT:    All paths from source to sink must pass through must_traverse.
                         Models dominator tree. Detects auth bypass on refactor.
          ISOLATION:     No path from source to sink exists (layer enforcement).
                         Anti-join on edges by namespace/tag.
          BLAST_RADIUS:  On file change, reverse-traverse call edges, identify
                         callers whose AST hash did not change (un-updated callers).
          EFFECT_CHAIN:  Reachability from public API to node tagged with
                         effect type (DB_Write, Network_IO, Filesystem).
                         Cargo Scan effects model generalized via DKG tagging.

        Performance guarantee:
          SQLite covering B-Tree index on (source, target, edge_type) required.
          Depth-limited recursive CTE with cycle detection via materialized path.
          <10ms for depth<=5 on graphs up to 180,000 nodes (source: architectural-
          rule-engine.md section 5.2, NGQP benchmark). Depths 6-7: MEASURE during
          Phase 3F implementation. If >10ms at depth 7, reduce max to 5.
          Note: research benchmarks non-recursive queries and depth<=5 CTEs only.
          Depth 7 performance is an extrapolation that must be validated.

        Integration:
          rules/ crate defines trait DkgQuerier { fn query_reachability(...) -> ... }
          knowledge/ crate implements DkgQuerier.
          Injection: rules/ takes Box<dyn DkgQuerier> at construction (same pattern
          as existing Box<dyn AstEvaluator> in rules.txt section 3A). NO static
          crate dependency rules/ -> knowledge/. rules/ depends only on core/.
          Testable with MockDkgQuerier before knowledge/ crate is available.
          Update: Cargo.toml dependency diagram and projectIndex.md unchanged.
          Evaluation order: GRAPH_REACHABILITY after COMPOSITE (most expensive).

      Add to rules.txt section 3A (evaluation order):
        5. GRAPH_REACHABILITY -- recursive CTE on DKG, <10ms

      Add DKG effect tagging (new subsection in knowledge.txt):
        During DKG node insertion, tag functions with effect types:
          Effect::Unsafe      — contains unsafe block (Rust)
          Effect::Filesystem  — calls std::fs::*, open(), write() etc.
          Effect::Network     — calls std::net::*, reqwest::*, HTTP clients
          Effect::Database    — calls .query(), .execute() on DB connectors
          Effect::Auth        — calls auth/permission/session check functions
        Tagging is pattern-based (function name + import path matching).
        Configurable in elai_cc.toml [knowledge.effect_tags].
        Source: architectural-rule-engine.md section 6

    Evidence:
      "DKGs stored in SQLite bridge the gap between per-file linting (fast
       but blind) and full dataflow analysis (deep but prohibitive) — graph
       reachability provides cross-file validation in <10ms." (source takeaway 1)
      "Reduced manual auditing burden to median 0.2% of total LOC." (Cargo Scan)
      "Covering B-Tree index on (source, target, edge_type) is the critical
       SQLite optimization for sub-10ms query performance." (source takeaway 6)

    New seed rules (add to rules.txt section 8.6 ENGINEERING):
      TIER 1 -- GRAPH_REACHABILITY:
        no public API -> DB_Write without Auth node on path
        no Domain layer -> Presentation layer edge (layer isolation)
        file change -> un-updated callers flagged as WARN

    Contract tests: minimum 3 per GRAPH_REACHABILITY rule (BLOCK + ALLOW + EDGE).
    Add to section 9 contract test counts: Engineering += ~9 tests.


C3. HASHMAP DETERMINISM QUARANTINE
    File: docs/specs/crates/core.txt (section 4, deterministic entropy traits)
    Phase affected: 3I (DST) but should be enforced NOW (entropy quarantine is Phase 0.8)

    Problem:
      std::collections::HashMap/HashSet use SipHash with random OS seed.
      Iteration order is non-deterministic across runs. This breaks DST
      and any test relying on deterministic output ordering.
      Source: deterministic-simulation-testing.md section 8.1

    Change:
      Add to core.txt section 4 (after EntropySource):

      HashMap quarantine:
        Ban std::collections::HashMap and std::collections::HashSet in all crates.
        Replacements:
          BTreeMap/BTreeSet: when iteration order matters (deterministic by key order).
          indexmap::IndexMap/IndexSet: when insertion order matters (deterministic).
          FxHashMap (rustc-hash crate): when O(1) lookup needed AND iteration
            order does not matter AND determinism is not required for that usage.
        Enforced by: clippy.toml disallowed-methods for HashMap::new, HashSet::new,
          HashMap::with_capacity, HashSet::with_capacity.
        Exception: third-party crate internals (cannot control, but madsim patches
          entropy source for their SipHash seed).

      Add to section 6 (testable invariants):
        - No std::collections::HashMap or HashSet in any crate (clippy enforced)
        - All map iteration produces identical ordering given same input sequence

    Evidence:
      "std::collections::HashMap/HashSet are fundamentally non-deterministic
       (SipHash 1-3 with random OS seed). Iteration order changes every
       execution." (source section 8.1)
      "HashMap, SystemTime, thread scheduling are the three primary entropy
       sources in Rust that must be quarantined." (source takeaway 6)


===============================================================================
HIGH PRIORITY CHANGES (significant grounded value)
===============================================================================

H1. VCR/TRACEOPS LLM MOCKING FOR DST
    File: docs/specs/guide/steps/phase-3I.txt (DST framework)
    Phase affected: 3I

    Problem:
      temperature=0 + fixed seed does NOT guarantee LLM determinism (GPU
      float divergence, MoE routing, ollama caching anomalies). The plan
      mentions madsim but has no strategy for mocking LLM inference.
      Source: deterministic-simulation-testing.md section 7

    Change:
      Add VCR (Video Cassette Recorder) pattern to DST spec:

      During initial non-DST run against live ollama:
        1. Intercept all InferRequest/InferResponse IPC messages.
        2. Compute blake2s hash of InferRequest (prompt + config).
        3. Store (hash, response) pairs as cassette file (msgpack).

      During DST runs:
        1. HTTP transport layer intercepted (reqwest middleware or madsim
           network mock), NOT at LlmProvider trait level. This captures
           ollama-specific caching behavior and HTTP-level anomalies.
        2. Compute blake2s hash of full HTTP request body (prompt + config + model).
        3. Look up in cassette. Return recorded HTTP response with
           deterministic simulated latency from madsim timer.
        4. If hash not in cassette: test fails (flags prompt drift).

      Note: trait-level interception (mocking LlmProvider::complete()) is
      a separate, complementary pattern for unit testing the orchestrator
      without any transport concerns. VCR operates at transport level to
      catch ollama caching anomalies that trait mocking would miss.

      TraceOps layer (for complex multi-step agent interactions):
        Record entire agent execution trajectory (GRC steps, tool calls,
        branching decisions) as structured trace. During DST, feed
        deterministic trajectory instead of live LLM. Tests FSM/SSB logic
        without any LLM inference.

      Add to phase-3I testable invariants:
        - VCR cassette replay produces identical SSB state as live run
        - Prompt hash mismatch detected and reported (not silently wrong)
        - TraceOps trajectory replay exercises all FSM transitions

    Evidence:
      "Live LLM inference in a 10,000-step DST sequence will inevitably
       cause state divergence." (source section 7.1)
      "VCR cassettes transform probabilistic AI responses into pure
       deterministic data payloads mapped to prompt hashes." (source section 7.2)


H2. EFFECT TAGGING IN DKG
    File: docs/specs/crates/knowledge.txt (section 2, Layer 1 DKG)
    Phase affected: 1A (DKG) but can be added incrementally in 3F

    [Detailed in C2 above — the DKG effect tagging is part of the
     GRAPH_REACHABILITY predicate implementation.]


H3. EDCT COUNTERFACTUAL METHOD
    File: docs/specs/crates/rules.txt (section 5A, counterfactual shadow testing)
    Phase affected: 3F, 4H

    Problem:
      Current spec uses CEL-derived counterfactual testing (strip finding
      from auditor context, rerun, check if auditor catches independently).
      Research identifies EDCT (Explanation-Driven Counterfactual Testing)
      as a complementary method that operates on the RULE itself, not the
      auditor.

    Change:
      Add EDCT to section 5A alongside CEL:

      EDCT validation (runs during PROPOSED -> CANDIDATE transition):
        Given a PROPOSED rule that triggers BLOCK on code snippet S:
        1. Generate structurally identical snippet S' WITH the fix applied
           (e.g., add sanitization that the rule checks for).
        2. If rule still triggers BLOCK on S': rule is reacting to a spurious
           feature (function length, import presence) not the causal violation.
        3. Reject rule (do not promote to CANDIDATE).

        Generation of S': Orchestrator applies fix mutation (deterministic
        AST transform: add the missing call/check that the rule detects).
        If AST transform not possible for this predicate type: skip EDCT,
        rely on CEL counterfactual only.

      Add to rule_metrics schema:
        edct_pass  INTEGER NOT NULL DEFAULT 0  -- passed EDCT validation
        edct_fail  INTEGER NOT NULL DEFAULT 0  -- failed EDCT (spurious)

    Evidence:
      "If removing a condition eliminates an effect, direct causal connection
       is likely (Method of Difference)." (source section 3.2)


H4. PROOF-GATED SELF-MODIFICATION (IC3-EVOLVE PATTERN)
    File: docs/specs/integration/self-modification.txt (Layer 2)
    Phase affected: 4H

    Problem:
      Self-modification currently uses staged evaluation (10/50/200 tasks)
      with metric-based promotion. IC3-Evolve demonstrates that performance
      improvement alone is insufficient — a proposed change can show +9
      solved instances while producing incorrect classifications that happen
      to look good on metrics.

    Change:
      Add proof gate to Layer 2 staged evaluation:

      Before any config variant enters Stage 2 (50 tasks):
        Run DST reference model with the proposed config change against
        a FIXED set of 10 regression test traces (canonical cassettes).
        Gate checks SOUNDNESS, not behavioral identity:
          PASS: all evidence gates still trigger correctly, no BLOCK findings
            bypassed, no test regression (PASS_TO_FAIL), FSM reaches COMPLETE.
          FAIL: evidence gate bypassed (e.g., UNGROUNDED claim reaches EXECUTING),
            test regression detected, FSM stuck or crashes.
        The gate does NOT reject different FSM action sequences — a config
        change that correctly tightens rule thresholds will produce different
        BLOCK/ALLOW distributions, and that is expected and valid.
        Key distinction from IC3-Evolve: IC3's gate checks correctness
        (SAFE/UNSAFE classification matches ground truth). Our gate checks
        soundness (evidence gates not bypassed, no regressions), not
        behavioral identity.

      Add slot-restricted patches (IC3-Evolve pattern) to self-mod agent:
        Self-mod agent edits are restricted to specific config "slots":
          Slot 1: GRC budgets (max_calls, max_chars per role)
          Slot 2: Rule thresholds (FP rate, TP rate, promotion gates)
          Slot 3: Context budget percentages (repo_map, code_context, ssb, grc)
          Slot 4: Model parameters (temperature, top_p per role)
        Max 1 slot modified per proposal. Cross-slot changes require
        explicit Compass&Jump mode (higher evidence bar).
        Forbidden: FSM states, IPC protocol, security rules, evidence gates.

    Evidence:
      "Rounds r093-r094 proposed a clause-cleaning heuristic showing +9 solved
       instances... However, it failed independent witness validation — the
       heuristic was producing incorrect UNSAFE classifications." (IC3-Evolve
       section 10.1)
      "This separates optimization from soundness." (IC3-Evolve section 4)


H5. GBNF SCHEMA CONSTRAINTS DOCUMENTATION
    File: docs/specs/crates/providers.txt (section 3, LayerConfig)
    Phase affected: 3F

    Change:
      Add subsection to Phase 0-1 ollama config:

      GBNF limitations (must be enforced at schema compilation time):
        - Max 3 levels of nesting (deeper causes stack overflow SIGSEGV)
        - No recursive $ref definitions
        - No uniqueItems, contains, not, if/then/else (unsupported in CFGs)
        - Regex patterns must use ^ and $ anchors (complex patterns cause
          catastrophic backtracking and server crashes)
        - additionalProperties defaults to false; setting true causes
          unescaped newline hallucinations

      compile_schema() in rules/ crate must validate these constraints
      before inserting SCHEMA_REQUIRED predicate into rules.db.
      Violation at compile time = specific error, not runtime crash.

    Evidence:
      llm-json-schema-generation.md section 2.2 (full limitation table)


===============================================================================
MODERATE CHANGES (good improvement, lower urgency)
===============================================================================

M1. INCREMENTAL AST CACHE SHARED BETWEEN KNOWLEDGE AND RULES
    File: docs/specs/crates/tools.txt or knowledge.txt
    Phase affected: 3F

    Problem:
      Tree-sitter cold-start parse of 5000-line file = 5-10ms. Rules engine
      <1ms budget requires incremental reparsing. Knowledge/ crate already
      parses for DKG. Double-parsing is wasteful.

    Change:
      Add shared AST cache (in-memory, per-project):
        On file_write event: knowledge/ crate incrementally reparses,
        stores updated tree-sitter Tree in cache keyed by (file_path, content_hash).
        Rules/ crate reads from cache instead of cold-parsing.
        Cache invalidation: immediate on SSB file_write event (same as
        vector cache invalidation in knowledge.txt section 3).

      Requirement: rules/ must hold parsed AST in memory with filesystem
      event-driven incremental updates. Cold-parse per rule evaluation
      categorically fails <1ms budget.

    Evidence:
      "Engine MUST maintain active memory state — hold parsed AST in RAM,
       incrementally update on filesystem events." (treesitter section 4.2)
      Incremental parse small edit on 5k file: 0.2-0.4ms (vs 5-10ms cold)


M2. AST-GREP YAML DSL FOR PREDICATE AUTHORING
    File: docs/specs/crates/rules.txt (section 3, AST_PATTERN)
    Phase affected: 3F

    Problem:
      Raw tree-sitter S-expressions require intimate knowledge of
      language-specific grammar node names. Fragile across grammars.

    Change:
      EVALUATE ast-grep's YAML DSL as alternative predicate format for
      AST_PATTERN rules. ast-grep parses rules written in target language
      syntax (isomorphic pattern matching). Example:
        pattern: "console.log($ARG)" instead of complex S-expression.

      If adopted: AST_PATTERN predicate field format supports both:
        query_format: "sexp" (raw tree-sitter, current)
        query_format: "ast-grep" (YAML DSL, isomorphic patterns)
      ast-grep is Rust-native (no FFI penalty). Sub-millisecond execution.

      Decision gate: Phase 3F SKELETON step. Implement both, benchmark,
      pick the one with better ergonomics + performance.

    Evidence:
      "ast-grep's isomorphic pattern matching with YAML DSL is the optimal
       ergonomic layer for rule authoring." (treesitter section 7)
      "Engine compiled string into appropriate Tree-sitter query dynamically."


M3. ASYMMETRIC FP PENALTY AND TIME-AWARE DECAY
    File: docs/specs/crates/rules.txt (section 5, rule lifecycle)
    Phase affected: 3F, 4H

    Change:
      Add to rule_metrics schema:
        last_true_positive_at  INTEGER  -- epoch of last TP
        confidence_score       REAL DEFAULT 1.0  -- decays over time

      Time-aware confidence decay:
        Rules assigned 90-day half-life. If no true positives in 90 days,
        confidence halves. Below 0.25 = auto-deprecated to SHADOW.
        Resets on each true positive.

      Asymmetric harm multiplier:
        Each false positive penalizes confidence by 4x compared to
        true positive reward. Rationale: blocking valid code is more
        disruptive than missing a violation (auditor catches misses).

      Add to ACTIVE -> RETIRED transition triggers:
        confidence_score < 0.1 sustained over 5 pipelines

      Add to section 7 (theoretical limits, new subsection):
        "Fully autonomous closed-loop self-improvement is strictly
         unsustainable over long horizons (entropy decay + variance
         amplification). Periodic human review of rule set health
         is mandatory. Schedule: every 50 pipelines or 30 days,
         whichever comes first. Review: top 5 lowest-confidence ACTIVE
         rules + top 5 highest-composite PROPOSED rules."

    Evidence:
      "A single false positive penalizes confidence by 4x compared to
       positive weight from true detection." (auto-generated section 4.2)
      "Fully autonomous closed-loop self-improvement is strictly
       unsustainable over long horizons." (auto-generated section 7.4)


M4. CUSTOM SQLITE VFS FOR DST
    File: docs/specs/guide/steps/phase-3I.txt
    Phase affected: 3I

    Change:
      Add to DST infrastructure requirements:

      Custom SQLite VFS (Virtual File System) in Rust:
        Trap write()/fsync() syscalls, redirect to in-memory byte buffer
        managed by simulation. Simulator's deterministic PRNG dictates
        exact latency of read/write operations. When SQLite requests flush,
        VFS yields to simulation's timer queue.

        Enables:
          - Torn write simulation (truncate WAL at arbitrary byte boundary)
          - Deterministic I/O latency (no OS jitter)
          - WAL recovery testing (simulated crash + reboot + recovery)

      Implementation: ~200-400 LOC Rust implementing sqlite3_vfs struct.
      Reference: Limbo project (async SQLite rewrite with DST-native I/O).

    Evidence:
      "Custom SQLite VFS in Rust: Trap write()/fsync() syscalls, redirect
       to in-memory byte buffer managed by simulation." (DST section 6.1)


===============================================================================
M5. AUDITOR CONTEXT: DKG BLAST RADIUS INJECTION
    File: docs/specs/crates/orchestrator.txt (section 6, context budget)
    Phase affected: 3E (context budget allocator)
===============================================================================

    Problem:
      LLM auditors hallucinate architecture when given fragmented cross-file
      context from BM25/vector search. They need pre-computed structural
      subgraphs showing the blast radius of changes.
      Source: architectural-rule-engine.md section 7.3

    Change:
      When assembling Slot 5 (code_context) for auditor role:
        1. Compute blast radius via DKG reverse traversal of changed files.
        2. Include blast_radius_subgraph as a SEPARATE sub-section within
           Slot 5, clearly delimited from BM25/vector chunks.
        3. Subgraph budget: 30% of code_context allocation.
           BM25/vector chunks: 70% of code_context allocation.
        4. Configurable in elai_cc.toml [context_budget.auditor.subgraph_pct].

      Cache breakpoint 2 impact:
        Cache breakpoint 2 (after Slots 4-5) is currently described as
        "stable within same files." The DKG subgraph changes on every commit,
        not on file-set changes. Therefore: cache breakpoint 2 applies to
        the BM25/vector portion of Slot 5 only. The subgraph sub-section
        is AFTER cache breakpoint 2 (it is volatile per-commit content).
        This means Slot 5 is split: [BM25/vector (cacheable)] [subgraph (volatile)].

      Non-auditor roles: no subgraph injection. Executor, context-compiler,
      etc. continue using Slot 5 as-is (no split).

    Evidence:
      "LLM auditors hallucinate architecture when given fragmented cross-file
       context; they must receive pre-computed structural subgraphs, not raw
       code." (architectural-rule-engine.md section 7.3)


===============================================================================
M6. AST_PATTERN PREDICATE VALIDATION HARDENING
    File: docs/specs/crates/rules.txt (section 3A, compile_predicate)
    Phase affected: 3F
===============================================================================

    Change:
      Add to compile_predicate validation for AST_PATTERN:
        - Reject unanchored unbounded quantifiers (*, +) without terminal node.
          These cause catastrophic backtracking in tree-sitter query engine.
        - Validate query against each target language grammar at insertion time.
        - Warn on deeply nested alternations ([] > 3 levels).
        Critical for self-discovery: auto-generated predicates from violation
        patterns may be malformed. Must catch at insertion, not evaluation.

    Source: treesitter-ast-rule-engine.md section 4.3


===============================================================================
AUDIT RESOLUTION LOG (6 BLOCKs from adversarial audit)
===============================================================================

  BLOCK-1 (H3 EDCT lifecycle placement): FIXED. Moved from CANDIDATE->SHADOW
    to PROPOSED->CANDIDATE, matching cited source.
  BLOCK-2 (C2 undeclared dependency): FIXED. Specified Box<dyn DkgQuerier>
    injection pattern. No static crate dependency. Matches AstEvaluator pattern.
  BLOCK-3 (C2 depth=10 ungrounded): FIXED. Reduced max to 7, added MEASURE
    gate. Acknowledged extrapolation beyond research data.
  BLOCK-4 (H4 wrong gate logic): FIXED. Changed from behavioral identity to
    soundness check (evidence gates not bypassed, no regressions). Does not
    reject different BLOCK/ALLOW distributions.
  BLOCK-5 (H1 fabricated struct name): FIXED. VCR operates at HTTP transport
    layer (reqwest middleware), not LlmProvider trait. Matches research.
  BLOCK-6 (unnumbered cross-cutting section): FIXED. Numbered as M5,
    assigned to Phase 3E, documented cache breakpoint 2 impact.

  Additional: Predicate validation section numbered as M6, assigned Phase 3F.
  H2 is a cross-reference to C2 (not independent). Effective count: 14.


===============================================================================
SUMMARY: CHANGE COUNT BY FILE
===============================================================================

  providers.txt:     2 changes (C1 per-model strategy, H5 GBNF limits)
  rules.txt:         6 changes (C2 GRAPH_REACHABILITY, H3 EDCT, M2 ast-grep,
                                M3 decay/asymmetric, M6 predicate validation)
  core.txt:          1 change  (C3 HashMap ban)
  knowledge.txt:     1 change  (H2/C2 effect tagging)
  orchestrator.txt:  2 changes (M5 auditor context injection, context budget)
  self-modification: 1 change  (H4 proof-gated, slot-restricted)
  phase-3I.txt:      2 changes (H1 VCR/TraceOps, M4 SQLite VFS)

  Total: 15 items (14 independent + H2 cross-ref to C2).
  All grounded in research evidence. All audit BLOCKs resolved.

  WARNs from audit (not resolved here, noted for implementer):
    - C1: Add failure handler for INSTRUCTOR mode after 3 retries (ESCALATE to human)
    - C3: Consider banning FxHashMap too, or require #[allow] annotation
    - M3: Add SHADOW->ACTIVE recovery path for decayed rules (human review trigger)
    - M2: Consider just picking ast-grep now instead of dual implementation
    - M4: Spike first to check if madsim already covers SQLite I/O
    - H5: Merge into C1 (same source, same section)
    - H4 Slot 3: Enumerate specific config keys, flag auditor context % changes
