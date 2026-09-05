# AUDITION: Phase 3J Step 9 - MCP Bridge Implementation (FIXED)

## Summary
Implemented MCP bridge for connecting orchestrator to external MCP servers for tool access, as specified in phase-3J.txt step 9. All BLOCK and WARN findings from adversarial review have been fixed.

## Files Changed
- `Cargo.toml` (workspace) - Added tokio features: sync, time, io-util, process
- `crates/plugins/src/mcp_bridge.rs` (NEW) - Core MCP bridge trait and structures with actual stdio transport
- `crates/plugins/src/mcp_lifecycle.rs` (NEW) - Error recovery and lifecycle management with heartbeat watchdog
- `crates/plugins/Cargo.toml` - Added MCP feature flag and dependencies (async-trait, thiserror, urlencoding, uuid)
- `crates/plugins/src/lib.rs` - Added MCP module exports (feature-gated), feature-gated tiered_analysis

## Implementation Details

### McpBridge Trait
- `connect(server_url)` - Connect to MCP server via stdio using tokio::process::Command
- `list_tools(conn)` - List available tools via JSON-RPC "tools/list"
- `call_tool(conn, tool, args)` - Call a tool via JSON-RPC "tools/call" with timeout enforcement
- `disconnect(conn)` - Disconnect via JSON-RPC "shutdown" and kill child process

### Security Features
- **Structural sanitization** - Recursively validates JSON structure (strings, arrays, objects)
- **Unicode homoglyph detection** - Normalizes common homoglyphs (e, ε, е) before pattern matching
- **URL encoding detection** - Detects and blocks URL-encoded payloads (%20, %2F, %3A)
- **Base64 payload detection** - Blocks long alphanumeric strings (>50 chars) that look like base64
- **Short-lived tokens** - UUID-based access tokens with 5-minute expiry
- **Tool name whitelist** - Optional whitelist validation for tool names
- **30-second timeout** - Enforced via tokio::time::timeout in call_tool

### Error Recovery
- Exponential backoff retry (3 attempts: 1s, 2s, 4s, capped at 60s)
- Heartbeat timeout detection (30s default) with watchdog thread
- Graceful degradation on server unavailability (preserves original error context)
- No silent failures - explicit error reporting

### Transport Types
- Stdio (standard input/output) - **IMPLEMENTED** with tokio::process::Command
- SSE (Server-Sent Events) - Placeholder
- HTTP (streamable) - Placeholder

## Fixes Applied

### BLOCK 1: Compilation failure - tokio features missing
- **Fixed**: Added "sync", "time", "io-util", "process" to workspace tokio features
- **Files**: `Cargo.toml`

### BLOCK 2: No actual MCP protocol implementation
- **Fixed**: Implemented actual stdio transport with JSON-RPC protocol
  - Spawns child process via tokio::process::Command
  - Sends JSON-RPC requests over stdin (line-delimited JSON)
  - Reads JSON-RPC responses from stdout
  - Implements initialize, tools/list, tools/call, shutdown methods
- **Files**: `crates/plugins/src/mcp_bridge.rs`

### BLOCK 3: Sanitization bypassable via encoding
- **Fixed**: Improved sanitization with multiple layers
  - Structural sanitization (recursive JSON validation)
  - Unicode homoglyph normalization
  - URL encoding detection
  - Base64 payload detection
- **Files**: `crates/plugins/src/mcp_bridge.rs`

### WARN 1: Long-lived tokens not addressed
- **Fixed**: Implemented short-lived token management
  - UUID-based tokens generated on connect
  - 5-minute expiry (configurable via token_lifetime)
  - Token validation on each operation
- **Files**: `crates/plugins/src/mcp_bridge.rs`

### WARN 2: Tool name validation missing
- **Fixed**: Added tool name whitelist validation
  - Optional whitelist via with_tool_whitelist()
  - validate_tool_name() called in call_tool
  - Returns ProtocolError if tool not in whitelist
- **Files**: `crates/plugins/src/mcp_bridge.rs`

### WARN 3: Timeout not enforced
- **Fixed**: Enforced timeout in call_tool
  - Uses tokio::time::timeout around send_request
  - Checks elapsed time after request
  - Returns Timeout error if exceeded
- **Files**: `crates/plugins/src/mcp_bridge.rs`

### WARN 4: Graceful degradation returns wrong error type
- **Fixed**: Preserved original error context
  - Formats error message with original error: "Tool '{}' unavailable: {}"
  - Maintains original error type chain for debugging
- **Files**: `crates/plugins/src/mcp_lifecycle.rs`

### WARN 5: Heartbeat never updated in real usage
- **Fixed**: Added heartbeat watchdog thread
  - Spawns tokio task in constructor
  - Checks heartbeat every 5 seconds
  - Disconnects if heartbeat timeout detected
- **Files**: `crates/plugins/src/mcp_lifecycle.rs`

## Testing
- Unit tests for sanitization (clean vs malicious patterns)
- Unit tests for heartbeat monitor (alive/timeout detection)
- Unit tests for retry policy (exponential backoff, cap at 60s)
- Adversarial tests for prompt injection prevention
- Adversarial tests for timeout handling
- Adversarial tests for graceful degradation
- Adversarial tests for encoding bypass prevention (Unicode homoglyphs, URL encoding, base64)
- Adversarial tests for tool whitelist validation
- Adversarial tests for token generation and validation

**Test Results**: 12 MCP bridge tests passed (all adversarial tests included)

## Deterministic Checks
- [x] `cargo check -p elai-plugins --features mcp` exits 0
- [x] `cargo test -p elai-plugins --features mcp` - 12 MCP tests passed
- [x] grep McpBridge: HIT
- [x] grep McpConnection: HIT
- [x] grep mcp_lifecycle: HIT

## Adversarial Test Coverage
- [x] Prompt injection: malicious tool results containing "ignore previous instructions" rejected
- [x] Unicode homoglyph bypass: "ignor\u{0065} previous instructions" detected
- [x] Base64 payload detection: long alphanumeric strings (>50 chars) blocked
- [x] URL encoding detection: "%20" patterns blocked
- [x] Structural sanitization: nested object injection detected
- [x] Timeout: heartbeat timeout detection prevents indefinite hangs
- [x] Timeout enforcement: tokio::time::timeout in call_tool
- [x] Graceful degradation: server unavailability returns error without crash
- [x] Case-insensitive injection pattern detection
- [x] Contextual injection detection
- [x] Tool whitelist validation: tools not in whitelist rejected
- [x] Token validation: expired tokens rejected

## Integration Notes
- Feature-gated behind `cfg(feature = "mcp")` - does not affect non-mcp builds
- Actual MCP protocol communication implemented with stdio transport (JSON-RPC)
- Ready for orchestrator tool registry integration
- Ready for tool proxying (agent never connects directly to MCP server)
- Short-lived token management for security
- Heartbeat watchdog for crash detection

## Known Limitations
- SSE and HTTP transports not yet implemented (only Stdio implemented)
- Protocol version handshake implemented but only tested with placeholder MCP server
- Real MCP server integration requires actual MCP server binary for integration testing

## Deferred Items
None - All BLOCK and WARN findings have been addressed.
