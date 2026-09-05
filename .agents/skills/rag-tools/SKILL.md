---
name: rag-tools
description: Rules for using the local-rag MCP server tools. Triggers when calling mcp_search_context, mcp_memorize, mcp_ingest_file, mcp_forget, mcp_list_topics, or any RAG-related operation.
license: internal
compatibility: "Requires .elai_cc/ layout, local-rag MCP server, FSM state machine"
metadata:
  version: "1.0.0"
  author: "elai"
---

RAG (local-rag MCP server)

You have access to a local offline knowledge base via MCP tools. Use them autonomously without asking the user.

All tools accept an optional project parameter for filtering (e.g. project="death-must-die"). Always pass the current project name when calling any RAG tool so results stay scoped. When ingesting or memorizing, always tag with the project name. When searching without a specific project need, omit the parameter to search across all projects.

mcp_search_context: call this during the search phase of every plan, before forming any hypothesis about a bug, and when entering unfamiliar code. Query with the concept, not the symptom. "velocity rotation hook pattern" not "crash".

mcp_memorize: call this after every completed plan phase and after fixing any bug. topic = short slug, tags = comma-separated categories. Do not wait until end of session. NEVER batch mcp_memorize with other tool calls. It triggers lazy model loading (280MB embedding model, 15-30 seconds on first call per session). Batching it blocks all queued tools and makes the UI appear frozen. Also: Qdrant uses an exclusive file lock -- never start a second Python process that imports the server while the MCP server is already running; it will deadlock with AlreadyLocked.

mcp_ingest_file: call this when a file is too large to read directly. Ingest it once, then query with mcp_search_context instead of reading it again.

mcp_forget: call this when a stored entry is confirmed wrong or outdated.

mcp_list_topics: call this at session start if the project uses the RAG heavily, to know what is already indexed.

Just-In-Time Verification Protocol: When retrieving archival memory or architectural rules via mcp_search_context, do not trust the memory blindly. Cross-reference stored citations against the live codebase. If the cited code has been altered and now contradicts the memory, or if the citations point to files that no longer exist, investigate the discrepancy and call mcp_memorize to update the archival memory before proceeding.

If the MCP server is not running or returns an error, continue without it and note the failure in the response.
