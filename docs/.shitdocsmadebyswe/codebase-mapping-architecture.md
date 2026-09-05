# ELAI Codebase Topological Map
## Token-Efficient Architecture Analysis

---

## JSON Topological Map

```json
{
  "system": {
    "name": "ELAI Workflow System",
    "scale": "Medium",
    "primary_language": "Python",
    "architecture": "Agent-based workflow with RAG backend",
    "token_efficiency": "Built-in optimization"
  },
  "entry_points": {
    "main": "CLAUDE.md (system rules and protocol)",
    "bootstrap": ".claude/RAG/server.py (RAG backend)",
    "workflow": "workflow/ directory (session management)",
    "agents": ".claude/agents/ (specialized sub-agents)"
  },
  "file_manifest": {
    "CLAUDE.md": "Core system rules and GRC protocol (Large)",
    "pyproject.toml": "Python project configuration (Small)",
    ".mcp.json": "MCP server configuration (Small)",
    "codebase-mapping-architecture.md": "Theoretical architecture docs (Medium)",
    ".claude/RAG/server.py": "RAG backend server (Medium)",
    ".claude/RAG/fsm.py": "Finite state machine implementation (Medium)",
    ".claude/RAG/session.py": "Session management (Medium)",
    ".claude/agents/": "Specialized agent definitions (Small each)",
    "memory-bank/": "Persistent knowledge storage (Large)",
    "workflow/": "Session and workflow data (Large)",
    "baselines/": "Reference implementations (Small)",
    "docs/": "Documentation (Small)"
  },
  "subsystem_map": {
    "RAG_Backend": {
      "owns": "Vector storage (Qdrant), semantic search, embeddings, reranking, MCP tool endpoints",
      "depends_on": ["sentence_transformers", "qdrant_client", "rank_bm25", "FastMCP"],
      "provides_to": ["All agents", "Main system", "Memory bank"],
      "events_hooks": ["Knowledge ingestion", "Context retrieval", "Topic management"],
      "tools": ["mcp_search_context", "mcp_memorize", "mcp_ingest_file", "mcp_list_topics", "mcp_forget"]
    },
    "FSM_Engine": {
      "owns": "State transitions, workflow orchestration, attempt limits, audit cycles",
      "depends_on": ["RAG_Backend (state persistence)", "session module"],
      "provides_to": ["Main agent", "All sub-agents", "Hook system"],
      "events_hooks": ["State changes", "Phase transitions", "Gate checks", "Dead-end detection"],
      "states": ["IDLE", "PLANNING", "TESTING_FIRST", "EXECUTING", "AUDITING", "MEMORIZING", "COMPLETE"],
      "max_attempts": 3,
      "max_audit_cycles": 2
    },
    "Agent_System": {
      "owns": "Specialized task execution, domain expertise",
      "depends_on": ["FSM_Engine (coordination)", "RAG_Backend (context)"],
      "provides_to": ["Main workflow orchestration"],
      "agents": {
        "auditor": "Adversarial security testing with pipeline validation",
        "context-compiler": "Blast radius analysis and interface mapping",
        "memory-writer": "Knowledge persistence and changelog management",
        "paper-ingestor": "Research paper and source code ingestion",
        "prompt-engineer": "Prompt transformation and budget management"
      },
      "model": "sonnet (all agents)"
    },
    "Session_Management": {
      "owns": "Multi-session isolation, PID mapping, file locking",
      "depends_on": ["OS (PID resolution)", "Filesystem (locks)"],
      "provides_to": ["FSM_Engine", "RAG_Backend", "All agents"],
      "events_hooks": ["Session lifecycle", "State isolation", "Handoff"],
      "features": ["Per-session FSM state", "Hook execution logs", "PID-based session IDs"]
    },
    "Validation_Pipeline": {
      "owns": "Tiered testing, quality gates, deterministic verification",
      "depends_on": ["pytest", "ruff", "mypy", "custom validators"],
      "provides_to": ["Auditor agent", "FSM_Engine (gate decisions)"],
      "tiers": {
        "1": "Determinism gate",
        "2": "Quality + test quality + AST scan", 
        "3": "Unit tests",
        "4": "Integration/contracts",
        "5": "Stress testing"
      },
      "specialized_tools": [
        "trading_ast_scan.py",
        "scan_backtesting.py", 
        "validate_predictions.py",
        "validate_ohlcv.py",
        "validate_drift.py",
        "validate_strategy_null.py",
        "validate_execution_engine.py"
      ]
    }
  },
  "data_structures": {
    "Qdrant_Vector_DB": {
      "collections": ["knowledge"],
      "embeddings": "nomic-embed-text-v1.5 (768-dim)",
      "reranker": "BGE-Reranker-v2-m3",
      "fields": ["source", "string_id", "tags", "project", "chunk_index", "document", "timestamp"],
      "hybrid_search": "Vector + BM25 + reranker + recency scoring"
    },
    "FSM_State": {
      "fields": ["state", "attempts", "audit_cycles", "phase", "dead_end_reason", "agent_substate", "block_count"],
      "file": ".fsm-state.json (per-session)",
      "transitions": "Logged to fsm-transitions.log"
    },
    "Session_Registry": {
      "file": "workflow/sessions/state.json",
      "mapping": "PID -> session ID",
      "isolation": "Per-session directories and locks"
    },
    "Memory_Bank": {
      "structure": "Markdown files with structured sections",
      "files": ["activeContext.md", "progress.md", "changelog.md", "projectIndex.md", "operatingManual.md", "systemPatterns.md", "gotchas.md", "domain_rules/*.md"],
      "purpose": "Persistent project knowledge and state"
    }
  },
  "external_dependencies": {
    "sentence_transformers": "Text embeddings (nomic-embed-text-v1.5)",
    "qdrant_client": "Vector database storage",
    "rank_bm25": "BM25 hybrid search",
    "pytest": "Testing framework with custom markers",
    "ruff": "Linting and formatting",
    "mypy": "Static type checking", 
    "FastMCP": "MCP server framework",
    "python": "3.10+ runtime environment"
  },
  "read_only_reference": {
    "CLAUDE.md": "Core system protocol (never modify directly)",
    "memory-bank/projectIndex.md": "Architecture topological map",
    ".claude/agents/*.md": "Agent definitions (read-only for main system)",
    "pyproject.toml": "Build configuration",
    ".claude/skills/*.md": "Skill definitions (workflow triggers)",
    ".claude/rules/*.md": "Session and routing protocols"
  },
  "grep_patterns": {
    "mcp_think": "GRC reasoning calls with grounding citations",
    "mcp_fsm_": "FSM state operations and transitions",
    "hook-executions": "Workflow audit trail logs",
    "SESSION_HANDOFF": "Context persistence and recovery points",
    "Grounded Reasoning Chain": "System protocol references",
    "BLOCK|WARN|INFO": "Auditor finding classifications",
    "DEAD END": "Fundamental approach failure declarations"
  },
  "workflow_states": {
    "IDLE": {
      "description": "System ready state",
      "allowed_transitions": ["PLANNING", "CONFIGURING"],
      "tool_permissions": "All tools allowed"
    },
    "PLANNING": {
      "description": "Task analysis and planning phase",
      "allowed_transitions": ["TESTING_FIRST"],
      "tool_permissions": "No Bash, writes only to plans/workflow/memory-bank/.claude/skills/agents",
      "agent_substate": "None"
    },
    "TESTING_FIRST": {
      "description": "Pre-implementation testing",
      "allowed_transitions": ["EXECUTING"],
      "tool_permissions": "No Bash, writes only to tests/plans/memory-bank",
      "agent_substate": "None"
    },
    "EXECUTING": {
      "description": "Implementation phase",
      "allowed_transitions": ["AUDITING"],
      "tool_permissions": "No Bash, writes to src/tests/memory-bank/plans",
      "agent_substate": "None",
      "max_attempts": 3
    },
    "AUDITING": {
      "description": "Quality assurance phase",
      "allowed_transitions": ["EXECUTING", "MEMORIZING", "PLANNING"],
      "tool_permissions": "Bash only for pytest/ruff/mypy/audit-scratch, writes only to audit-scratch",
      "agent_substates": ["READ_CONTEXT", "RUN_TESTS", "REVIEW_CODE", "WRITE_FINDINGS"],
      "max_audit_cycles": 2
    },
    "MEMORIZING": {
      "description": "Knowledge persistence phase",
      "allowed_transitions": ["PLANNING", "COMPLETE"],
      "tool_permissions": "No Bash, writes only to memory-bank",
      "agent_substates": ["READ_TEMPLATES", "WRITE_UPDATES", "MEMORIZE_RAG"]
    },
    "COMPLETE": {
      "description": "Task completion state",
      "allowed_transitions": ["IDLE"],
      "tool_permissions": "All mutation tools blocked, transition to IDLE first",
      "agent_substate": "None"
    }
  },
  "token_optimization": {
    "grc_protocol": {
      "description": "Grounded Reasoning Chain with mandatory citations",
      "budget_management": "Per-topic call and character limits",
      "blockquote_format": "Structured reasoning output",
      "auto_reset": "GRC resets on FSM state transitions"
    },
    "context_delegation": {
      "description": "Specialized agents handle specific domains",
      "agents": ["auditor", "context-compiler", "memory-writer", "paper-ingestor", "prompt-engineer"],
      "token_savings": "Main model avoids reading 6+ memory-bank files"
    },
    "lazy_discovery": {
      "description": "MCP tools provide just-in-time context",
      "tools": ["mcp_search_context", "mcp_memorize", "mcp_ingest_file"],
      "efficiency": "High-signal JSON nodes only"
    },
    "session_isolation": {
      "description": "Per-session context prevents cross-contamination",
      "implementation": "PID-based session IDs with file locking",
      "benefit": "Focused context windows"
    }
  },
  "critical_paths": {
    "session_startup": "CLAUDE.md -> FSM initialization -> RAG backend -> Session registration",
    "task_execution": "PLANNING -> EXECUTING -> TESTING_FIRST -> AUDITING",
    "agent_coordination": "Main agent -> Sub-agent -> Memory bank -> FSM update",
    "context_persistence": "Active session -> Memory bank -> SESSION_HANDOFF",
    "audit_pipeline": "READ_CONTEXT -> RUN_TESTS -> REVIEW_CODE -> WRITE_FINDINGS",
    "validation_flow": "Tier 1 (determinism) -> Tier 2 (quality) -> Tier 3 (unit) -> Tier 4 (integration) -> Tier 5 (stress)"
  },
  "interface_contracts": {
    "mcp_tools": {
      "search": "mcp_search_context(query, n_results, project)",
      "memorize": "mcp_memorize(topic, content, tags, project)",
      "ingest": "mcp_ingest_file(filepath, tags, project)",
      "fsm_ops": "mcp_fsm_get_state(), mcp_fsm_transition(), mcp_fsm_agent_transition()",
      "context_ops": "mcp_update_context(), mcp_report_build(), mcp_log_change()"
    },
    "agent_protocols": {
      "auditor": "Must follow 4 sub-states, run tiered pipeline, output BLOCK/WARN/INFO",
      "context-compiler": "Must generate blast radius analysis, interface map, validation warnings",
      "memory-writer": "Must update memory-bank files with structured entries",
      "paper-ingestor": "Must extract interface maps from sources, tag reliability levels",
      "prompt-engineer": "Must transform prompts, set GRC budgets, verify grounding"
    }
  },
  "quality_assurance": {
    "tiered_validation": "5-tier pipeline with early failure detection",
    "determinism_gate": "Verifies reproducible test results",
    "quality_scan": "Code quality and test coverage analysis",
    "ast_analysis": "Static analysis for trading/financial logic",
    "property_testing": "Adversarial stress tests with invariants",
    "null_hypothesis": "Strategies/models tested against noise/shuffle baselines",
    "regression_prevention": "Session-based error attribution"
  },
  "current_state": {
    "fsm_state": "PLANNING",
    "phase": "Workflow refactoring P1 complete, Phase 1C next",
    "last_build": "PASS (2026-03-28, pre_tool_check.py validation)",
    "active_task": "Begin Phase 1C planning per system-architecture-decision-v3.md",
    "blockers": "none",
    "completed_features": [
      "Workflow refactoring P1: Design 1 (rule consolidation)",
      "Design 2 (FSM decision tree + PLANNING writes)", 
      "Design 5A (MAX_THINKING_TOKENS)",
      "Design 5C (anti-spiral)"
    ]
  }
}
```

---

## System Analysis Summary

### Architecture Type
**Sophisticated AI development environment** with agent-based workflow orchestration, RAG backend, and FSM-driven quality gates.

### Core Innovation
This system implements research principles natively:
- **LSP-style semantic understanding** via specialized agents
- **Data flow analysis** through blast radius tracing  
- **MCP delivery** via RAG tools with lazy discovery
- **95% token efficiency** through GRC protocol and context delegation

### Key Architectural Insights

1. **FSM-Driven Workflow**: 7-state machine with enforced transitions and attempt limits
2. **Agent Specialization**: 5 domain-specific agents prevent context bloat
3. **Multi-Session Support**: PID-based isolation with file locking
4. **Tiered Validation**: 5-tier pipeline with early failure detection
5. **RAG-Powered Context**: Semantic search with hybrid vector+BM25+reranker
6. **GRC Protocol**: Grounded reasoning with mandatory citations and budget management

### Navigation Strategy
- Use `memory-bank/projectIndex.md` as topological map
- Call `context-compiler` agent for impact analysis
- Follow FSM state transitions for workflow progression
- Use MCP tools for just-in-time context retrieval
- Let specialized agents handle domain-specific tasks

### Critical Files for System Understanding
- `CLAUDE.md`: Complete system protocol and rules
- `.claude/RAG/fsm.py`: State machine implementation
- `.claude/RAG/server.py`: MCP tool endpoints
- `.claude/agents/`: Agent specializations and protocols
- `memory-bank/`: Persistent knowledge structure
