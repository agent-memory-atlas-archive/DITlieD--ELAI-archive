use std::io::{self, BufRead, BufReader, Read, Write};
use std::path::Path;

use elai_core::paths::sessions_dir_path;
use elai_orchestrator::ssb_viewer::SsbViewer;
use elai_orchestrator::read_rra;
use elai_plugins::authz::{check_authz, AuthzPolicy, AuthzVerdict};
use serde_json::{json, Value as JsonValue};

use crate::args::CliOutputFormat;

const AUTHZ_ROLE: &str = "planner";
const AUTHZ_POLICY_PATH: &str = ".elai_cc/authz-policy.toml";
const MCP_PROTOCOL_VERSION: &str = "2025-03-26";
const MAX_FRAME_BYTES: usize = 16 * 1024 * 1024;

const FSM_GET_STATE: &str = "fsm_get_state";
const PLAN_STATUS: &str = "plan_status";
const RRA_SHOW: &str = "rra_show";

pub fn run_mcp_serve(_output_format: CliOutputFormat) -> Result<(), String> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut reader = BufReader::new(stdin.lock());
    let mut writer = stdout.lock();

    loop {
        let frame = match read_frame(&mut reader) {
            Ok(Some(frame)) => frame,
            Ok(None) => return Ok(()),
            Err(error) => return Err(format!("mcp-serve: reading stdio frame: {error}")),
        };

        let payload = match frame {
            Ok(payload) => payload,
            Err(reason) => {
                write_response(&mut writer, &jsonrpc_error(JsonValue::Null, -32700, "Parse error", json!({
                    "type": "MalformedFrame",
                    "reason": reason,
                    "recoverable": true,
                })))?;
                continue;
            }
        };

        let request: JsonValue = match serde_json::from_slice(&payload) {
            Ok(request) => request,
            Err(error) => {
                write_response(&mut writer, &jsonrpc_error(JsonValue::Null, -32700, "Parse error", json!({
                    "type": "MalformedFrame",
                    "reason": error.to_string(),
                    "recoverable": true,
                })))?;
                continue;
            }
        };

        let Some(object) = request.as_object() else {
            write_response(&mut writer, &jsonrpc_error(JsonValue::Null, -32600, "Invalid Request", json!({
                "type": "InvalidRequest",
                "recoverable": true,
            })))?;
            continue;
        };

        let id = object.get("id").cloned();
        let method = object.get("method").and_then(JsonValue::as_str);
        let params = object.get("params").cloned().unwrap_or_else(|| json!({}));

        let Some(method) = method else {
            if let Some(id) = id {
                write_response(&mut writer, &jsonrpc_error(id, -32600, "Invalid Request", json!({
                    "type": "InvalidRequest",
                    "reason": "missing method",
                    "recoverable": true,
                })))?;
            }
            continue;
        };

        match method {
            "notifications/initialized" => {}
            "exit" => return Ok(()),
            "initialize" => {
                if let Some(id) = id {
                    write_response(&mut writer, &jsonrpc_result(id, initialize_result()))?;
                }
            }
            "ping" => {
                if let Some(id) = id {
                    write_response(&mut writer, &jsonrpc_result(id, json!({})))?;
                }
            }
            "tools/list" => {
                if let Some(id) = id {
                    write_response(&mut writer, &jsonrpc_result(id, tools_list_result()))?;
                }
            }
            "tools/call" => {
                if let Some(id) = id {
                    write_response(&mut writer, &jsonrpc_result(id, dispatch_tool_call(&params)))?;
                }
            }
            "shutdown" => {
                if let Some(id) = id {
                    write_response(&mut writer, &jsonrpc_result(id, JsonValue::Null))?;
                }
            }
            _ => {
                if let Some(id) = id {
                    write_response(&mut writer, &jsonrpc_error(id, -32601, "Method not found", json!({
                        "type": "MethodNotFound",
                        "method": method,
                    })))?;
                }
            }
        }
    }
}

fn dispatch_tool_call(params: &JsonValue) -> JsonValue {
    let Some(tool_name) = params.get("name").and_then(JsonValue::as_str) else {
        return tool_error("INVALID_ARGUMENTS", "tools/call requires a string `name`", json!({
            "status": "invalid_arguments",
        }));
    };
    let arguments = params.get("arguments").cloned().unwrap_or_else(|| json!({}));

    let policy = AuthzPolicy::load(AUTHZ_POLICY_PATH).unwrap_or_else(|_| AuthzPolicy::deny_all());
    match check_authz(AUTHZ_ROLE, tool_name, None, &policy) {
        AuthzVerdict::Deny { reason } => tool_error("AUTHZ_DENIED", &reason, json!({
            "status": "denied",
            "role": AUTHZ_ROLE,
            "tool": tool_name,
        })),
        AuthzVerdict::Allow => match tool_name {
            FSM_GET_STATE => fsm_get_state(),
            PLAN_STATUS => not_available(
                PLAN_STATUS,
                "I-39 plan-state derivation is not exposed as a callable read-only API",
            ),
            RRA_SHOW => rra_show(&arguments),
            _ => tool_error("AUTHZ_DENIED", "tool is not registered by the Tier-1 MCP server", json!({
                "status": "denied",
                "role": AUTHZ_ROLE,
                "tool": tool_name,
            })),
        },
    }
}

fn fsm_get_state() -> JsonValue {
    let project_root = match std::env::current_dir() {
        Ok(path) => path,
        Err(error) => return not_available(FSM_GET_STATE, &format!("cannot resolve project root: {error}")),
    };
    let db_path = sessions_dir_path(&project_root).join("ssb.db");
    if !db_path.is_file() {
        return not_available(FSM_GET_STATE, &format!("SSB journal not found at {}", db_path.display()));
    }
    let viewer = match SsbViewer::open(&db_path) {
        Ok(viewer) => viewer,
        Err(error) => return not_available(FSM_GET_STATE, &format!("cannot open SSB journal read-only: {error}")),
    };
    match viewer.latest_fsm_state() {
        Ok(snapshot) => tool_success(json!({
            "status": "available",
            "source": "ssb",
            "state": snapshot.state,
            "phase": snapshot.phase,
        })),
        Err(error) => not_available(FSM_GET_STATE, &format!("cannot derive FSM state from SSB: {error}")),
    }
}

fn rra_show(arguments: &JsonValue) -> JsonValue {
    let Some(path) = arguments.get("path").and_then(JsonValue::as_str) else {
        return tool_error("INVALID_ARGUMENTS", "rra_show requires a string `path`", json!({
            "status": "invalid_arguments",
            "tool": RRA_SHOW,
        }));
    };
    match read_rra(Path::new(path)) {
        Ok(artifact) => match serde_json::to_value(artifact) {
            Ok(payload) => tool_success(payload),
            Err(error) => tool_error("SERIALIZATION_ERROR", &error.to_string(), json!({
                "status": "error",
                "tool": RRA_SHOW,
            })),
        },
        Err(error) => tool_error("RRA_READ_ERROR", &error.to_string(), json!({
            "status": "error",
            "tool": RRA_SHOW,
            "path": path,
        })),
    }
}

fn not_available(tool: &str, reason: &str) -> JsonValue {
    tool_error("NOT_AVAILABLE", reason, json!({
        "status": "not_available",
        "tool": tool,
    }))
}

fn tool_success(payload: JsonValue) -> JsonValue {
    let text = payload.to_string();
    json!({
        "content": [{"type": "text", "text": text}],
        "structuredContent": payload,
        "isError": false,
    })
}

fn tool_error(code: &str, message: &str, mut details: JsonValue) -> JsonValue {
    if let Some(object) = details.as_object_mut() {
        object.insert("code".to_string(), json!(code));
        object.insert("message".to_string(), json!(message));
    }
    let text = details.to_string();
    json!({
        "content": [{"type": "text", "text": text}],
        "structuredContent": details,
        "isError": true,
    })
}

fn initialize_result() -> JsonValue {
    json!({
        "protocolVersion": MCP_PROTOCOL_VERSION,
        "capabilities": {"tools": {"listChanged": false}},
        "serverInfo": {
            "name": "elai-tier1-control-plane",
            "version": env!("CARGO_PKG_VERSION"),
        },
    })
}

fn tools_list_result() -> JsonValue {
    json!({
        "tools": [
            tool_definition(
                FSM_GET_STATE,
                "Read the current FSM state derived from the append-only SSB journal.",
                json!({"type": "object", "properties": {}, "additionalProperties": false}),
            ),
            tool_definition(
                PLAN_STATUS,
                "Read derived plan status when a callable read-only API is available.",
                json!({"type": "object", "properties": {}, "additionalProperties": false}),
            ),
            tool_definition(
                RRA_SHOW,
                "Read and decode a Reproducible-Run Artifact without replaying it.",
                json!({
                    "type": "object",
                    "properties": {"path": {"type": "string"}},
                    "required": ["path"],
                    "additionalProperties": false,
                }),
            ),
        ]
    })
}

fn tool_definition(name: &str, description: &str, input_schema: JsonValue) -> JsonValue {
    json!({
        "name": name,
        "description": description,
        "inputSchema": input_schema,
        "annotations": {
            "readOnlyHint": true,
            "destructiveHint": false,
            "idempotentHint": true,
            "openWorldHint": false,
        },
        "_meta": {
            "authz": {
                "role": AUTHZ_ROLE,
                "tool": name,
            }
        }
    })
}

fn jsonrpc_result(id: JsonValue, result: JsonValue) -> JsonValue {
    json!({"jsonrpc": "2.0", "id": id, "result": result})
}

fn jsonrpc_error(id: JsonValue, code: i64, message: &str, data: JsonValue) -> JsonValue {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": {"code": code, "message": message, "data": data},
    })
}

fn write_response(writer: &mut impl Write, response: &JsonValue) -> Result<(), String> {
    let payload = serde_json::to_vec(response)
        .map_err(|error| format!("mcp-serve: serializing response: {error}"))?;
    write!(writer, "Content-Length: {}\r\n\r\n", payload.len())
        .map_err(|error| format!("mcp-serve: writing response header: {error}"))?;
    writer
        .write_all(&payload)
        .map_err(|error| format!("mcp-serve: writing response body: {error}"))?;
    writer
        .flush()
        .map_err(|error| format!("mcp-serve: flushing response: {error}"))
}

fn read_frame(reader: &mut impl BufRead) -> io::Result<Option<Result<Vec<u8>, String>>> {
    let mut content_length: Option<usize> = None;
    let mut header_error: Option<String> = None;
    let mut saw_header = false;

    loop {
        let mut line = String::new();
        let bytes_read = reader.read_line(&mut line)?;
        if bytes_read == 0 {
            return if saw_header {
                Err(io::Error::new(io::ErrorKind::UnexpectedEof, "stdio closed in frame header"))
            } else {
                Ok(None)
            };
        }
        saw_header = true;
        if line == "\r\n" || line == "\n" {
            break;
        }
        let header = line.trim_end_matches(['\r', '\n']);
        let Some((name, value)) = header.split_once(':') else {
            header_error = Some(format!("malformed header `{header}`"));
            continue;
        };
        if name.trim().eq_ignore_ascii_case("Content-Length") {
            match value.trim().parse::<usize>() {
                Ok(length) => content_length = Some(length),
                Err(error) => header_error = Some(format!("invalid Content-Length: {error}")),
            }
        }
    }

    let Some(content_length) = content_length else {
        return Ok(Some(Err(header_error.unwrap_or_else(|| "missing Content-Length header".to_string()))));
    };
    if content_length > MAX_FRAME_BYTES {
        let copied = io::copy(&mut reader.take(content_length as u64), &mut io::sink())?;
        if copied != content_length as u64 {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "stdio closed in oversized frame"));
        }
        return Ok(Some(Err(format!(
            "frame length {content_length} exceeds {MAX_FRAME_BYTES}-byte limit"
        ))));
    }

    let mut payload = vec![0_u8; content_length];
    reader.read_exact(&mut payload)?;
    if let Some(error) = header_error {
        return Ok(Some(Err(error)));
    }
    Ok(Some(Ok(payload)))
}
