# LLM Workflow for Vibes - Token-Efficient Codebase Summary

## PROJECT IDENTITY
**Name**: llm_workflow_for_vibes  
**Type**: AI workflow orchestration framework with adversarial auditing  
**Scale**: Medium (29 Python files, 28 Markdown files)  
**Primary Language**: Python 3.10+  

## CORE ARCHITECTURE

### Finite State Machine (FSM) Workflow
- **States**: IDLE → PLANNING → TESTING_FIRST → EXECUTING → AUDITING → MEMORIZING → COMPLETE
- **Enforcement**: Pre-tool hooks block actions per state, cross-session file locks
- **Dynamic Limits**: Audit cycles scale with block count (0→1, 1-2→2, 3+→3)

### Agent System (5 specialized agents)
- **auditor**: Hostile security auditor, runs tiered validation pipeline
- **context-compiler**: Blast radius analysis, interface mapping  
- **memory-writer**: Updates memory-bank, RAG ingestion
- **paper-ingestor**: Academic paper and source code ingestion
- **prompt-engineer**: Reformats user prompts for token efficiency

### Skill System (3 mandatory workflows)
- **planning-workflow**: Full plan creation, FSM execution, phase completion
- **prompt-transform**: Reformats prompts before implementation (triggered by code changes)
- **rag-tools**: RAG tool usage protocols

## VALIDATION PIPELINE

### 5-Tier System (scripts/run_pipeline.py)
1. **Determinism** (determinism_check.py) - detects non-deterministic patterns
2. **Quality** (test_quality_scan.py) - code quality, test quality, AST scan
3. **Unit Tests** - pytest unit tests (<10s, deterministic)
4. **Integration/Contracts** - cross-module interface tests
5. **Stress** - adversarial edge case tests

### Trading/ML Specialized Tools (.claude/tools/)
- **trading_ast_scan.py** - Static lookahead bias detector (587 lines)
- **validate_drift.py** - Concept drift detection (PSI > 0.25 = BLOCK)
- **validate_ohlcv.py** - OHLCV data quality validation
- **validate_predictions.py** - Prediction quality audit (PSR, direction imbalance)
- **validate_strategy_null.py** - Null hypothesis testing for strategies
- **validate_execution_engine.py** - Execution engine stress tests

## MEMORY BANK SYSTEM

### Core Files (memory-bank/)
- **projectIndex.md** - Topological system map (template structure, empty)
- **activeContext.md** - Current session state, SESSION_HANDOFF data
- **progress.md** - Build status, broken/untested modules
- **changelog.md** - Per-file change history
- **systemPatterns.md** - Locked architectural decisions
- **gotchas.md** - Known failure patterns
- **operatingManual.md** - Build/test commands, environment constraints

### Domain Rules
- **tradingRules.md** - Financial/trading system rules
- **gameRules.md** - Game/modding logic rules  
- **dashboardRules.md** - UI/dashboard rules
- **applicationRules.md** - General application rules

## KEY CONFIGURATION

### pyproject.toml
- **Test markers**: unit, integration, stress, conformance, operational, quarantine
- **Ruff config**: line-length 120, Python 3.10, comprehensive lint rules
- **pytest config**: tests/ directory, quarantine exclusion

### CLAUDE.md (55 lines, 8758 bytes)
- **GRC Protocol**: Grounded Reasoning Chain via mcp_think tool
- **Hard Rules**: No template deletion, anti-slopsquatting, context delegation
- **FSM Rules**: State transitions, hook enforcement, cleanup requirements
- **Skills**: Mandatory workflow triggers for prompt-transform, planning-workflow, rag-tools

## WORKFLOW ORCHESTRATION

### Session Management
- **Session IDs**: s-* format for tracking (e.g., s-109188, s-1628600)
- **Context Packages**: workflow/sessions/{id}/context-package.md
- **Handoff Protocol**: SESSION_HANDOFF in activeContext.md for context exhaustion

### File Locking System
- **Own/Lock**: mcp_session_claim() for file ownership
- **Conflict Detection**: Cross-session file lock enforcement
- **Release**: mcp_session_release() after phase completion

## TESTING INFRASTRUCTURE

### Test Categories
- **Unit**: <10s, no network, deterministic, gates commits
- **Integration**: <60s, cross-module, gates auditor pass  
- **Stress**: <5min, adversarial, auditor-only
- **Conformance**: Real-world behavioral, scheduled runs
- **Operational**: Live data, non-deterministic, never blocks

### Audit Scratch Space
- workflow/audit-scratch/ - verification scripts, property tests
- Adversarial stress tests required for trading/engine/portfolio code
- Invariant testing with numpy.random generators, explicit seeds

## SECURITY & VALIDATION FOCUS

### Anti-Patterns Detected
- Lookahead bias in trading pipelines
- Data leakage through future information
- Overfitting in backtesting results
- Silent data corruption

### Block Thresholds
- PSI > 0.20 (drift detection)
- Direction imbalance > 4:1 (predictions)
- ECE > 0.15 (prediction calibration)
- Zero slippage/commission (backtesting)

## INTEGRATION POINTS

### MCP (Model Context Protocol) Servers
- **memory**: Knowledge graph management
- **puppeteer**: Browser automation
- **sequential-thinking**: Reasoning chains

### External Dependencies
- Minimal: pytest, ruff, numpy, pandas
- Optional: scipy (fallback implementations included)
- No heavy frameworks - stdlib-first approach

## PHILOSOPHICAL APPROACH

### Core Principles
- **Zero Lazy Code**: No pass bodies, TODO, placeholder returns
- **Grounded Reasoning**: All claims must cite data sources
- **Adversarial Testing**: Assume code will fail, prove it works
- **Token Economy**: Every token costs context, maximize efficiency

### Development Workflow
1. Research & Planning (FSM PLANNING state)
2. Test-First Development (TESTING_FIRST state)
3. Implementation with GRC (EXECUTING state)  
4. Adversarial Audit (AUDITING state)
5. Memory Integration (MEMORIZING state)

## CURRENT STATE

### Template Structure
Most memory-bank files are templates (Format: headers only) - indicates early stage project setup
- projectIndex.md: Empty template structure
- projectBrief.md: Empty template structure  
- Domain rules files: Exist but likely template content

### Active Components
- Full agent system implemented
- Complete validation pipeline
- Sophisticated FSM workflow
- Comprehensive tooling for trading/ML validation

### Documentation
- docs_summary.md: Contains summaries of external documents about Claude Code leak, backtesting overfitting
- Multiple architectural planning documents in root
- ClaudeLeak documentation series in docs/ folder
