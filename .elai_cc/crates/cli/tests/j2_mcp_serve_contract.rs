use std::io::{BufRead, BufReader, Read, Write};
use std::path::Path;
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::Arc;

use elai_core::clock::ClockProvider;
use elai_orchestrator::ssb::{PhaseTransitionPayload, Ssb, SsbConfig, SsbPayload};
use serde_json::{json, Value as JsonValue};

struct McpFixture {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

impl McpFixture {
    fn spawn(project_root: &Path) -> Self {
        let mut child = Command::new(env!("CARGO_BIN_EXE_elai"))
            .arg("mcp-serve")
            .current_dir(project_root)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("spawn elai mcp-serve");
        let stdin = child.stdin.take().expect("server stdin");
        let stdout = child.stdout.take().expect("server stdout");
        Self {
            child,
            stdin,
            stdout: BufReader::new(stdout),
        }
    }

    fn request(&mut self, request: &JsonValue) -> JsonValue {
        let payload = serde_json::to_vec(request).expect("serialize request");
        self.write_payload(&payload);
        self.read_response()
    }

    fn write_payload(&mut self, payload: &[u8]) {
        write!(
            self.stdin,
            "Content-Length: {}\r\n\r\n",
            payload.len()
        )
        .expect("write frame header");
        self.stdin.write_all(payload).expect("write frame body");
        self.stdin.flush().expect("flush frame");
    }

    fn read_response(&mut self) -> JsonValue {
        let mut content_length = None;
        loop {
            let mut line = String::new();
            let read = self.stdout.read_line(&mut line).expect("read response header");
            assert_ne!(read, 0, "server closed stdout before response");
            if line == "\r\n" {
                break;
            }
            let header = line.trim_end_matches(['\r', '\n']);
            if let Some((name, value)) = header.split_once(':') {
                if name.trim().eq_ignore_ascii_case("Content-Length") {
                    content_length = Some(
                        value
                            .trim()
                            .parse::<usize>()
                            .expect("numeric Content-Length"),
                    );
                }
            }
        }
        let length = content_length.expect("response Content-Length");
        let mut payload = vec![0_u8; length];
        self.stdout.read_exact(&mut payload).expect("read response body");
        serde_json::from_slice(&payload).expect("parse response JSON")
    }
}

impl Drop for McpFixture {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn prepare_project_root() -> tempfile::TempDir {
    let root = tempfile::tempdir().expect("temp project root");
    let elai_dir = root.path().join(".elai_cc");
    std::fs::create_dir_all(elai_dir.join("sessions")).expect("create session dir");
    let policy_source = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../authz-policy.toml");
    std::fs::copy(policy_source, elai_dir.join("authz-policy.toml")).expect("copy authz policy");

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("tokio runtime");
    runtime.block_on(async {
        let config = SsbConfig {
            db_path: elai_dir.join("sessions/ssb.db"),
            pool_size: Some(1),
            project_id: "j2-contract".to_string(),
            session_id: "j2-session".to_string(),
        };
        let clock: Arc<dyn ClockProvider> = Arc::new(elai_core::OsClock);
        let mut ssb = Ssb::new(config, clock).expect("create SSB fixture");
        ssb.write_payload(
            "j2-contract",
            0,
            SsbPayload::PhaseTransition(PhaseTransitionPayload {
                from_state: "IDLE".to_string(),
                to_state: "PLANNING".to_string(),
                evidence_summary: "journal-derived-j2-fixture".to_string(),
            }),
            Vec::new(),
        )
        .await
        .expect("write phase transition");
        ssb.shutdown();
    });
    root
}

fn call_tool(id: u64, name: &str, arguments: JsonValue) -> JsonValue {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": "tools/call",
        "params": {"name": name, "arguments": arguments},
    })
}

#[test]
fn j2_mcp_serve_is_read_only_deny_default_and_recovers_from_malformed_frames() {
    let root = prepare_project_root();
    let mut server = McpFixture::spawn(root.path());

    let initialize = server.request(&json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2025-03-26",
            "capabilities": {},
            "clientInfo": {"name": "j2-contract", "version": "1"}
        }
    }));
    assert_eq!(initialize["result"]["serverInfo"]["name"], "elai-tier1-control-plane");

    let tools = server.request(&json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/list",
        "params": {}
    }));
    let names: Vec<&str> = tools["result"]["tools"]
        .as_array()
        .expect("tools array")
        .iter()
        .map(|tool| tool["name"].as_str().expect("tool name"))
        .collect();
    assert_eq!(names, vec!["fsm_get_state", "plan_status", "rra_show"]);
    for tool in tools["result"]["tools"].as_array().expect("tools array") {
        assert_eq!(tool["annotations"]["readOnlyHint"], true);
        assert_eq!(tool["_meta"]["authz"]["role"], "planner");
        assert_eq!(tool["_meta"]["authz"]["tool"], tool["name"]);
    }

    let fsm = server.request(&call_tool(3, "fsm_get_state", json!({})));
    assert_eq!(fsm["result"]["isError"], false);
    assert_eq!(fsm["result"]["structuredContent"]["source"], "ssb");
    assert_eq!(fsm["result"]["structuredContent"]["state"], "PLANNING");
    assert_eq!(
        fsm["result"]["structuredContent"]["phase"],
        "journal-derived-j2-fixture"
    );

    let denied = server.request(&call_tool(4, "task_submit", json!({"task": "must not run"})));
    assert_eq!(denied["result"]["isError"], true);
    assert_eq!(denied["result"]["structuredContent"]["code"], "AUTHZ_DENIED");
    assert_eq!(denied["result"]["structuredContent"]["status"], "denied");

    let policy_path = root.path().join(".elai_cc/authz-policy.toml");
    std::fs::remove_file(&policy_path).expect("remove policy to exercise deny-all fallback");
    let missing_policy = server.request(&call_tool(5, "fsm_get_state", json!({})));
    assert_eq!(missing_policy["result"]["isError"], true);
    assert_eq!(
        missing_policy["result"]["structuredContent"]["code"],
        "AUTHZ_DENIED"
    );
    let policy_source = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../authz-policy.toml");
    std::fs::copy(policy_source, &policy_path).expect("restore authz policy");

    server.write_payload(b"{not-json");
    let malformed = server.read_response();
    assert_eq!(malformed["error"]["code"], -32700);
    assert_eq!(malformed["error"]["data"]["type"], "MalformedFrame");
    assert_eq!(malformed["error"]["data"]["recoverable"], true);

    let fsm_after_error = server.request(&call_tool(6, "fsm_get_state", json!({})));
    assert_eq!(fsm_after_error["result"]["isError"], false);
    assert_eq!(
        fsm_after_error["result"]["structuredContent"]["state"],
        "PLANNING"
    );
}
