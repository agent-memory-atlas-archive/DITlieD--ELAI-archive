SUB-PHASE 1 — PART C BLOCKER FIXES (decision doc)
Session: s-1178780
Date: 2026-04-30

WHAT
Fix 5 pipeline shape bugs that block real-model E2E. Items 1.1–1.5 from
NEXT-SESSION-RESIDUALS-PROMPT.md §3, mirroring NEXT-SESSION-PART-C-PROMPT.md §2.

KEY DECISIONS
1.1 (--backend=local for CliAction::Prompt):
    Implement OPTION-A: explicit `--backend=local[:PORT]` flag for the Prompt
    action. Adds a BackendConfig with BackendMode::Local; build_runtime already
    routes that into BudgetPolicy::Local { ceiling=LOCAL_MODEL_MAX_CONTEXT,
    fraction=LOCAL_BUDGET_FRACTION }. Side-fix: also auto-detect when env
    ANTHROPIC_BASE_URL contains localhost/127.0.0.1 (covers existing scripts).
    No new dependency.

1.2 (Tool list filter — drop test/agent-only tools):
    Filter inside filter_tool_specs at cli/main.rs:1005. Drop names matching
    test/agent-runtime kinds: TestingPermission, RunTaskPacket, Worker* (any
    of {Create,Get,Observe,ResolveTrust,AwaitReady,SendPrompt,Restart,Terminate}),
    Team* (Create,Delete), Cron* (Create,Delete,List), RemoteTrigger, McpAuth,
    MCP, SendUserMessage, NotebookEdit, ListMcpResources, ReadMcpResource.
    Whitelist what stays for the model: bash, read_file, write_file, edit_file,
    glob_search, grep_search, WebFetch, WebSearch, TodoWrite, Skill, Agent,
    ToolSearch, AskUserQuestion, Sleep, EnterPlanMode, ExitPlanMode,
    StructuredOutput, REPL, PowerShell, LSP, git_commit, TaskCreate, TaskGet,
    TaskList, TaskStop, TaskUpdate, TaskOutput. (~26 model-visible tools, well
    below the 60K-char system prompt that triggered the prefill TDR.)
    Implementation: a const HashSet of MODEL_HIDDEN_TOOL_NAMES + a filter inside
    filter_tool_specs that runs after the existing allowed_tools filter.

1.3 (augment_with_knowledge ContextBudget cap):
    augment_with_knowledge at cli/main.rs:3516 currently appends raw retrieval
    output without a token budget. Cap by character length using the prompt
    budget for Local backend (LOCAL_MODEL_MAX_CONTEXT * LOCAL_BUDGET_FRACTION).
    For Local backend, cap appended retrieval at min(budget/4 chars, 8000 chars).
    For Api backend, keep current behavior (no cap). This is a guard, not the
    full ModelCapabilityProfile wiring (deferred to B-10).

1.4 (dep_audit gemma-4 path):
    dep_audit.rs currently references a removed gemma-4 path. Default to the
    Qwen2.5-Coder-7B-Instruct-abliterated local path; if missing, skip the
    model-load step gracefully (warn-and-continue, no crash).

1.5 (indexing.rs core::paths migration):
    init_indexing_service in cli/src/indexing.rs (only 5KB) presumably contains
    the .elai vs .elai_cc dual-path block flagged "// B-15D AUDIT". Migrate to
    core::paths::dkg_db_path() / bm25_db_path(). Cross-grep all .elai/dkg.db
    string literals in cli/src/main.rs and route through the same helper.

WIRING_CONTRACT
produces_symbols:
  - elai_cli::CliAction::Prompt.backend (new field)
  - elai_cli::filter_tool_specs hidden-name filter
  - elai_cli::LiveCli::augment_with_knowledge bounded variant
  - elai_cli::dep_audit::default_model_path (new fn)
consumes_symbols:
  - build_runtime (already uses backend_config branch)
  - LiveCli prompt loop
reachable_from:
  - elai.exe -p "..." entrypoint
  - elai.exe doctor / dep audit subcommands
stub_ban:
  - no `todo!()`, no `unimplemented!()`, no `# TODO` placeholders introduced
observable_effect:
  - sidecar.log system_chars < 10K for typical turn (down from ~60K)
  - sidecar.log prompt max_tokens cap respected (≤ LOCAL_MODEL_MAX_CONTEXT/4)
  - augment_with_knowledge does not push prompt_chars past model context

DETERMINISTIC_CHECK
  cargo check --workspace passes (auditor)
  grep -E '"TestingPermission"' .elai_cc/crates/cli/src returns hidden-name reference
  grep -n 'BudgetPolicy::ApiDefault' shows no extra new uses on Prompt path
  grep '\.elai/dkg\.db' .elai_cc/crates/cli/src/ returns only via core::paths

ACCEPTANCE
  cargo build --release -p elai-cli (≤ 5 min, exit 0)
  After sidecar boot, `elai.exe --backend=local -p "hi"` returns within ≤120s
  with a non-empty answer and SSB rows > 0.

ANTI-SPIRAL
  Each item one pass; if cargo error cascades >5 files, STOP and revisit.
