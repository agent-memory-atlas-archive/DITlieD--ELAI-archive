plan: Architectural and Topological Workflow Improvement
created: 2026-04-01
status: pending

Goal: Eradicate "LLM Amnesia," "Monolithic Slop," and "Architectural Decay" by implementing deterministic codebase mapping (Topological Ingestor), automated Git-Governance, and an Evidence-Based Research Gate. This integrates existing sub-agents into a rigorous, multi-step validation cycle.


1. EVIDENCE-BASED RESEARCH GATE (PRE-PLANNING)
Source: chat 2, Paper Ingestor, Backtest Overfitting

Problem: LLMs often propose "happy path" solutions or "vibe-based" strategies that haven't been validated against real papers or proven engineering patterns.

Solution: The "Multi-Step Research Protocol." Before any PLANNING state, the `paper-ingestor` agent is triggered as a mandatory validator.
- Step 1: User Request (e.g., "How do I mod X game?").
- Step 2: LLM identifies high-level phases (e.g., Reverse Engineering, Patching, Hooking).
- Step 3: `paper-ingestor` (Mode: Source/Paper) gathers all up-to-date, validated information for each phase. It provides a "Cons vs. Pros" matrix for every option (e.g., DLL Injection vs. BepInEx).
- Step 4: The LLM generates a "System Physics" breakdown based ONLY on the validated research from Step 3.
- Step 5: User (Manager) reviews the validated options and the breakdown before approving the transition to PLANNING.


2. THE TOPOLOGICAL INGESTOR (DKB & RIG)
Source: SOTA Codebase Understanding, Rare Code Flaws, ingestcodebase.txt

Problem: Standard RAG (search) is "fuzzy" and misses "multi-hop" connections (e.g., Controller -> Service Interface -> Repository). This leads to architectural hallucinations in 500k+ LOC repos.

Solution: Deterministic AST-Derived Knowledge Graphs (DKB) & Repository Intelligence Graphs (RIG) with Cold-Start Detection.
- Automatic Scale Detection: On session start, the `context-compiler` runs a lightweight `glob` count.
    - COLD START (< 20 files): The Ingestor is bypassed; the agent uses direct file reads for maximum speed on new projects.
    - SCALE (>= 20 files): The Ingestor triggers a full AST scan to build/update the graph.
- Engine: Replace probabilistic search with a **Tree-sitter** parser to mathematically map all definitions and references.
- Graph Ontology (CBFDAE): Nodes are categorized into Components, Blocks, Functions (signatures), Data (schemas), Access, and Events.
- Multi-Hop Traversal: A Graph-based retrieval system that understands structural edges (inheritance, DI). When a function is touched, the system automatically identifies all "Callers" and "Callees" across file boundaries.
- Repository Intelligence (RIG): Ingests build-system metadata (e.g., pyproject.toml, requirements.txt) to map non-syntactic connections like linking and package dependencies.
- Skeletonization 2.0: For large codebases, provide sub-agents with "Skeletons"—files stripped of all logic but retaining imports, types, and function signatures—reducing token usage by 70% while maintaining 100% structural awareness.


3. AUTOMATED GIT-GOVERNANCE & TRACEABILITY
Source: gitcommitinworkflow.txt

Problem: Manual commits are skipped, losing the history of "how we got here." New files are often accidentally staged or missed in .gitignore.

Solution: Integrated Lifecycle Governance.
- Auto-Commit (MEMORIZING): Every successful FSM cycle (after the Adversarial Auditor passes Tier 1-4) triggers an automatic, semantic Git commit. Commits do NOT happen mid-execution to keep history clean.
- Automated Root Cause: If the POS-pipeline fails, a tool automatically `git grep` the history of the failing code to find the last known-good diff and presents it to the LLM for diagnosis.
- Auto-Gitignore: On every file creation, a hook checks the file type and size. Large data files, ML models, and logs are automatically added to `.gitignore` before the first commit.
- Push Logic: Commits are local-first. Pushing to remote is a manual user trigger or a final transition from COMPLETE -> IDLE.


4. CRISPY PIPELINE & SUB-AGENT DELEGATION
Source: chat 3 no slop ideas.txt, CLAUDE.md

Problem: "Instruction Budget" overload. Mega-prompts cause LLMs to write "slop."

Solution: Deterministic Sub-Agent Orchestration.
- The "Crispy" Sequence: Research (Paper Ingestor) -> Mapping (Context Compiler) -> Planning (Prompt Engineer) -> Implementation (Main) -> Audit (Auditor) -> Persistence (Memory Writer).
- Context Delegation: The **Context Compiler** sub-agent is the sole gatekeeper for information. It checks for a valid `.topological-map.db`. If found, it uses the Ingestor's graph to "compress" only the required multi-hop dependencies. If missing/new, it defaults to standard discovery tools.
- Instruction Shielding: Each sub-agent is given a task-specific, narrow instruction set (<50 instructions) to ensure 100% adherence and zero "attention decay."


IMPLEMENTATION PHASES

Phase 1: The Research & Breakdown Gate
- Trigger: Update `planning-workflow` to require `paper-ingestor` validation for all new projects/features.
- Deliverable: Multi-step "Validated Options" output before PLANNING starts.

Phase 2: Topological Ingestor (AST/Graph) with Scale-Check
- Implement `scripts/topological_ingestor.py` using Tree-sitter.
- Add scale-detection logic (file count threshold) to skip/trigger ingestion.
- Create the DAG-based SQLite database for structural mapping.
- Update `context-compiler` to query this graph for "Multi-Hop" dependencies.

Phase 3: Git-Governance & Auto-Gitignore
- Implement `scripts/git_lifecycle.py` to handle commits in the MEMORIZING state.
- Write the "New File -> Gitignore" check tool.
- Integrate `git grep` history analysis into the Auditor's failure loop.

Phase 4: Skeletonization & Context Compression
- Implement `scripts/skeletonize.py` to strip logic but keep signatures/types.
- Update `context-compiler` to provide skeletons for upstream dependencies.


KEY DECISIONS
1. Evidence First: No plan is valid without Paper/Source grounding.
2. Structure is Mathematical: Code mapping is AST-deterministic, never probabilistic.
3. Scale-Aware: Don't build expensive graphs for tiny projects.
4. Commit on Success: Git history reflects only verified, audited progress.
5. Delegate, Don't Bloat: Sub-agents own specific context layers to protect the instruction budget.
