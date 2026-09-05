Tool Inventory — full registry, classification, FSM filtering, file locations

Files:
- Tool definitions: crates/tools/src/lib.rs — mvp_tool_specs() at line 377
- Tool classification/FSM filtering: crates/orchestrator/src/tool_filter.rs — ToolCategory::classify() at line 86
- FSM state -> allowed categories: crates/orchestrator/src/tool_filter.rs — allowed_tool_categories() at line 224
- Dispatch context/config: crates/orchestrator/src/dispatcher.rs — build_dispatch_context(), BackendConfig
- Local client tool filtering: crates/cli/src/main.rs — LocalRuntimeClient::stream() ~line 5742
- Sidecar server-side filter: python/elai_inference/server.py — CORE_TOOLS constant

All 46 registered tools (mvp_tool_specs):

 #  Tool Name             Category          Notes
 1  bash                  Bash              shell exec
 2  read_file             Read
 3  write_file            Edit
 4  edit_file             Edit
 5  glob_search           Read
 6  grep_search           Read
 7  WebFetch              Web
 8  WebSearch             Web
 9  TodoWrite             Meta
10  Skill                 Meta
11  Agent                 Meta
12  ToolSearch            Meta
13  NotebookEdit          Meta
14  Sleep                 Meta
15  SendUserMessage       Meta
16  Config                ConfigOnly
17  EnterPlanMode         Planning
18  ExitPlanMode          Planning
19  StructuredOutput      Meta
20  REPL                  Meta
21  PowerShell            Bash
22  AskUserQuestion       Meta
23  TaskCreate            TaskManagement
24  RunTaskPacket         TaskManagement
25  TaskGet               TaskManagement
26  TaskList              TaskManagement
27  TaskStop              TaskManagement
28  TaskUpdate            TaskManagement
29  TaskOutput            TaskManagement
30  WorkerCreate          TaskManagement
31  WorkerGet             TaskManagement
32  WorkerObserve         TaskManagement
33  WorkerResolveTrust    TaskManagement
34  WorkerAwaitReady      TaskManagement
35  WorkerSendPrompt      TaskManagement
36  WorkerRestart         TaskManagement
37  WorkerTerminate       TaskManagement
38  TeamCreate            TaskManagement
39  TeamDelete            TaskManagement
40  CronCreate            TaskManagement
41  CronDelete            TaskManagement
42  CronList              TaskManagement
43  LSP                   Meta
44  ListMcpResources      Mcp
45  ReadMcpResource       Mcp
46  McpAuth               Mcp

Additional classify() entries (no ToolSpec but recognized by FSM filter):
- read / glob / grep                -> Read (aliases)
- write / edit                      -> Edit (aliases)
- powershell                        -> Bash
- webfetch / web_fetch              -> Web
- websearch / web_search            -> Web
- enterplanmode / enter_plan_mode   -> Planning
- exitplanmode / exit_plan_mode     -> Planning
- testingpermission                 -> Testing
- config                            -> ConfigOnly
- fsm_get_state                     -> FsmInternal
- fsm_transition                    -> FsmInternal
- list_hidden_paths                 -> FsmInternal
- mcp__* (prefix match)             -> Mcp
- anything unrecognized             -> Meta (default fallback)

FSM state -> allowed categories:

State          Allowed Categories
Idle           ALL (but LocalRuntimeClient sends zero tools in local mode)
Planning       Read, Web, Planning, Knowledge, Meta, FsmInternal
TestingFirst   Read, Planning, Testing, Meta, FsmInternal
Executing      Read, Edit, Bash, Web, Testing, Git, Meta, FsmInternal, TaskManagement, Mcp
Auditing       Read, Bash, Testing, Validation, Meta, FsmInternal
Memorizing     Read, Meta, FsmInternal
Complete       Read, Meta, FsmInternal
Configuring    Read, ConfigOnly, Meta, FsmInternal

Local pipeline flow:
- Idle -> 0 tools sent (model listens first)
- FSM transitions to Executing -> up to 8 tools (max_tools_local default), filtered by Executing categories
- Default max_tools_local = 8, configurable via BackendConfig
- Tool selection is FIFO from registry order after FSM filter (no priority weighting)
