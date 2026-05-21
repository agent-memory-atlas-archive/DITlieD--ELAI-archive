// C-39: CLI MCP client wrapper.
//
// Thin synchronous adapter over DefaultMcpBridge for use in the CLI sync
// task-runner path.

pub struct CliMcpClient { port: u16, url: String }

impl CliMcpClient {
    pub fn new(port: u16) -> Self {
        Self { port, url: format!("http://localhost:{port}") }
    }
    pub fn default_port() -> Self { Self::new(8200) }
    pub fn port(&self) -> u16 { self.port }
}

#[cfg(feature = "mcp")]
impl CliMcpClient {
    pub fn list_tools(&self) -> Result<Vec<String>, String> {
        use elai_plugins::mcp_bridge::{DefaultMcpBridge, McpToolSummary};
        let bridge = DefaultMcpBridge::default();
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| format!("cli_mcp_client: runtime build failed: {}", e))?;
        let url = self.url.clone();
        let result: Result<Vec<McpToolSummary>, _> =
            rt.block_on(async move { bridge.http_list_tools(&url).await });
        result
            .map(|tools| tools.into_iter().map(|t| t.name).collect())
            .map_err(|e| format!("cli_mcp_client: list_tools failed: {}", e))
    }
    pub fn invoke_tool(&self, tool_name: &str, args: serde_json::Value) -> Result<serde_json::Value, String> {
        use elai_plugins::mcp_bridge::DefaultMcpBridge;
        let bridge = DefaultMcpBridge::default();
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| format!("cli_mcp_client: runtime build failed: {}", e))?;
        let url = self.url.clone();
        let tn = tool_name.to_owned();
        let result = rt.block_on(async move { bridge.http_call_tool(&url, &tn, args).await });
        result.map(|r| r.content).map_err(|e| format!("cli_mcp_client: invoke_tool failed: {}", e))
    }
    pub fn get_tool_schema(&self, tool_name: &str) -> Result<serde_json::Value, String> {
        // TODO[C39.stub]: full schema resolution deferred; returns name + available flag
        let tools = self.list_tools()?;
        let available = tools.iter().any(|t| t == tool_name);
        Ok(serde_json::json!({"tool_name": tool_name, "available": available, "port": self.port}))
    }
}

#[cfg(not(feature = "mcp"))]
impl CliMcpClient {
    pub fn list_tools(&self) -> Result<Vec<String>, String> {
        Err("cli_mcp_client: mcp feature not compiled in".to_string())
    }
    pub fn invoke_tool(&self, _tool_name: &str, _args: serde_json::Value) -> Result<serde_json::Value, String> {
        Err("cli_mcp_client: mcp feature not compiled in".to_string())
    }
    pub fn get_tool_schema(&self, _tool_name: &str) -> Result<serde_json::Value, String> {
        Err("cli_mcp_client: mcp feature not compiled in".to_string())
    }
}
