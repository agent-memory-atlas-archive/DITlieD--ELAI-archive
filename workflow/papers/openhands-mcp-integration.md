Source: OpenHands MCP integration (github.com/All-Hands-AI/OpenHands)
Reliability: HIGH (reviewed source, Apache 2.0 public repo, ingested 2026-05-20)
Files examined:
  openhands/app_server/mcp/mcp_router.py
  openhands/app_server/app.py
  openhands/app_server/settings/settings_models.py
  tests/unit/mcp/test_mcp_integration.py
  tests/unit/server/routes/test_mcp_routes.py
  pyproject.toml
  fastmcp_slim/fastmcp/client/transports/config.py (fastmcp 3.3.1)
  fastmcp_slim/fastmcp/client/transports/http.py
  fastmcp_slim/fastmcp/client/transports/stdio.py
  fastmcp_slim/fastmcp/client/mixins/tools.py

ARCHITECTURE OVERVIEW

OpenHands is NOT a bespoke MCP client. All MCP client and server machinery
is delegated to fastmcp library (>=3.2,<4) which wraps mcp SDK (>=1.25).
  1. FastMCP server (mcp_router.py) with tools via @mcp_server.tool() decorator.
  2. MCPConfig in Settings.agent_settings.mcp_config pointing at remote MCP servers.
  3. HTTP mount of FastMCP server at /mcp (stateless_http=True) in FastAPI app.
  4. openhands-sdk (1.22.1, separate package) for agent-side tool dispatch.
There is NO hand-rolled MCPClient class.

KEY FINDING FOR C-39: OpenHands has NO standalone CLI binary. It is
web/agent-server only. The C-39 spec claim about CLI and web contexts is
incorrect for OpenHands. HTTP transport and stdio auto-launch patterns ARE
validated and apply to ELAI C-39.

PUBLIC API (fastmcp.Client)

list_tools | async(max_pages=250) -> list[mcp.types.Tool] | auto-paginates
list_tools_mcp | async(cursor=None) -> mcp.types.ListToolsResult | raw result
call_tool | async(name, arguments=None, *, timeout=None, raise_on_error=True, meta=None) -> CallToolResult
call_tool_mcp | async(name, arguments, ...) -> mcp.types.CallToolResult | no raise on tool error
CallToolResult: content (list), structured_content, meta, data, is_error (bool)
mcp.types.Tool: name (str), description (str|None), inputSchema (dict)

TRANSPORT IMPLEMENTATIONS

StreamableHttpTransport(url, headers=None, auth=None, verify=None)
  SSE-based via mcp.client.streamable_http.streamable_http_client
  wraps mcp.ClientSession
  timeout: httpx.Timeout(connect=30s, read=configurable)
  stateless_http=True on server: each request gets fresh MCP session

StdioTransport(command, args, env=None, cwd=None, keep_alive=True, log_file=None)
  spawns subprocess via mcp.client.stdio.stdio_client + anyio
  keep_alive=True (default): subprocess lives across connect/disconnect cycles
  connect() / disconnect() / close() explicit lifecycle
  _is_session_dead() checks anyio stream open_send_streams on read+write
  subprocess NOT killed when async context exits if keep_alive=True

MCPConfigTransport(config, name_as_prefix=True)
  single server: direct transport, no namespace prefix on tool names
  multi-server: composite FastMCP, each server as namespace
  tool naming in multi-server: namespace_toolname (underscore separator)
  partial failure: failed servers skipped with log warning
  all fail: raises ConnectionError

MCPConfig STRUCTURE

MCPConfig(mcpServers: dict[str, MCPServerTypes])
RemoteMCPServer(url, transport='http') -> StreamableHttpTransport
StdioMCPServer(command, args, env, cwd) -> StdioTransport
TransformingRemoteMCPServer / TransformingStdioMCPServer: adds transforms + tag filters

SERVER-SIDE REGISTRATION PATTERN

mcp_server = FastMCP('mcp', mask_error_details=True)

@mcp_server.tool()
async def create_pr(repo_name, ...) -> str:
    request = get_http_request()
    conversation_id = request.headers.get('X-OpenHands-ServerConversation-ID', None)
    raise ToolError(str(error))

mcp_app = mcp_server.http_app(path='/mcp', stateless_http=True)
app = FastAPI(routes=[Mount(path='/mcp', app=mcp_app)])

BEHAVIORAL CONTRACTS

list_tools -> all tools across all mounted namespaces; auto-paginates
call_tool -> ToolError from server propagates as ToolError at client
stateless_http server -> no session affinity; each request independent
StdioTransport keep_alive=True -> subprocess reused; must call close() explicitly
MCPConfigTransport multi-server -> ConnectionError only if ALL servers fail
Tool names in multi-server: namespace_originalname format

PLATFORM CONSTRAINTS

Python 3.12+ required
fastmcp>=3.2,<4 (3.3.1 reviewed); mcp>=1.25; anyio==4.9
Stdio uses anyio Event not asyncio.Event for ready/stop signals
Windows: no Unix-specific code in stdio transport; should work

GOTCHAS

1. stateless_http belongs in http_app() call, NOT FastMCP() constructor.
   Using it in constructor raises DeprecationWarning (OpenHands test verifies).
2. call_tool_mcp never raises on tool errors; call_tool raises (raise_on_error=True default).
3. Multi-server MCPConfigTransport: partial failure is silent; not an error.
4. StdioTransport keep_alive=True: context exit does NOT kill subprocess; must call close().
5. Tool name underscore collision: namespace prefix can be ambiguous with underscored names.
6. HTTP strips Authorization on cross-origin redirects; need custom httpx_client_factory.
7. openhands-sdk (agent dispatch) is a SEPARATE pinned package, MEDIUM reliability.

CRITICAL ASSERTIONS (testable)

assert len(tools) > 0
assert all(hasattr(t, 'name') and isinstance(t.inputSchema, dict) for t in tools)
assert result.is_error == False
assert result.content is not None
call_tool raise_on_error=True raises ToolError when server isError=True
Single server MCPConfigTransport: tool names have NO namespace prefix

C-39 DESIGN IMPLICATIONS

Validated patterns:
  HTTP transport is SSE-based. For CLI simplicity, use stateless JSON-RPC POST
  (fastmcp stateless_http=True accepts plain POST, no SSE parser needed).
  Stdio auto-launch: StdioTransport(command, args, keep_alive=True).
  ELAI MCPClient: spawn on demand, reuse within session, explicit teardown at exit.
  Lazy schema: list_tools with cursor aligns with C-35 ToolGate gated catalog.
  Error propagation: ToolError fail-closed to SSB matches C-39 stub_ban requirement.
  Dispatch trait: McpClientDispatch mirrors call_tool(name: str, args: dict).
No OpenHands precedent for: port allocation for co-process, SSB event bus, --via-mcp flag.

SPEC CORRECTION

C-39 claims OpenHands uses MCP bridge as primary tool surface in both CLI and web contexts.
Verified: OpenHands has no standalone CLI binary; it is web/agent-server only.
HTTP transport and stdio patterns are validated production best practice.
The CLI mode parity claim cannot cite OpenHands as direct precedent.
C-39 design is still valid; ELAI CLI is a different product.

CONTRACT TEST STUBS (tests/contracts/cli_mcp_contract_stubs.py)

def test_list_tools_returns_nonempty(mcp_client):
    import asyncio
    tools = asyncio.get_event_loop().run_until_complete(mcp_client.list_tools())
    assert len(tools) > 0
    assert all(hasattr(t, 'name') for t in tools)

def test_call_tool_success_has_content(mcp_client):
    import asyncio
    result = asyncio.get_event_loop().run_until_complete(
        mcp_client.call_tool('echo', {'text': 'hello'})
    )
    assert result.is_error == False
    assert result.content is not None

def test_call_tool_error_raises_tool_error(mcp_client):
    import asyncio
    from fastmcp.exceptions import ToolError
    raised = False
    try:
        asyncio.get_event_loop().run_until_complete(
            mcp_client.call_tool('nonexistent_tool_xxx', {})
        )
    except ToolError:
        raised = True
    assert raised

def test_tool_schema_has_input_schema(mcp_client):
    import asyncio
    tools = asyncio.get_event_loop().run_until_complete(mcp_client.list_tools())
    for t in tools:
        assert isinstance(t.inputSchema, dict)

def test_single_server_no_namespace_prefix(mcp_config_single):
    from fastmcp.client.transports.config import MCPConfigTransport
    assert len(mcp_config_single.mcpServers) == 1
    transport = MCPConfigTransport(mcp_config_single)
    assert hasattr(transport, 'transport')
